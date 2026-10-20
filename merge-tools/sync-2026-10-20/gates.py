#!/usr/bin/env python3
"""Gate suite for the dx-code x upstream Zed merge.

All checks run against a MERGED TREE (git tree object), never the worktree.

  markers   - zero conflict markers anywhere in the tree
  deletion  - every file deleted vs ours must be deleted by upstream too
              (no unexplained loss of our work)
  noloss    - token-level no-loss audit in BOTH directions for every changed
              .rs file: tokens added by a side relative to base must survive
  items     - item-parity sweep: every fn/struct/enum/trait/const/type/mod
              defined by either side must be defined at least as often in the
              merged file
  balance   - brace/paren/bracket balance of the merged file must equal the
              balance of one of the sides (balance neutrality)

Usage: python gates.py <merged-tree> [ours] [theirs] [base]
"""
import os
import re
import subprocess
import sys

REPO = os.environ.get("DX_CODE_REPO", r"G:\Dx\code")
IDENT = r"[A-Za-z_][A-Za-z0-9_]*"
TOKEN_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*|[^\sA-Za-z0-9_]")
ITEM_RE = {
    "fn": re.compile(rf"\bfn\s+({IDENT})"),
    "struct": re.compile(rf"\bstruct\s+({IDENT})"),
    "enum": re.compile(rf"\benum\s+({IDENT})"),
    "trait": re.compile(rf"\btrait\s+({IDENT})"),
    "type": re.compile(rf"\btype\s+({IDENT})"),
    "const": re.compile(rf"\bconst\s+({IDENT})"),
    "static": re.compile(rf"\bstatic\s+({IDENT})"),
    "mod": re.compile(rf"\bmod\s+({IDENT})"),
}


class Batch:
    """One long-lived `git cat-file --batch` process."""

    def __init__(self):
        self.p = subprocess.Popen(["git", "-C", REPO, "cat-file", "--batch"],
                                  stdin=subprocess.PIPE, stdout=subprocess.PIPE)

    def get(self, spec):
        self.p.stdin.write((spec + "\n").encode())
        self.p.stdin.flush()
        header = self.p.stdout.readline().decode().strip()
        if header.endswith("missing"):
            return None
        _, _, size = header.split(" ")
        n = int(size)
        data = self.p.stdout.read(n)
        self.p.stdout.read(1)  # trailing LF
        return data.decode("utf-8", "replace")


def git(*args):
    p = subprocess.run(["git", "-C", REPO, *args], stdout=subprocess.PIPE,
                       stderr=subprocess.DEVNULL)
    return p.stdout.decode("utf-8", "replace")


def strip_code(text):
    """Remove comments, string literals and char literals from Rust source.

    Order matters (strings are handled before comments can truncate them) and
    char literals must be distinguished from lifetimes ('a vs 'x'). Raw
    strings (r"", r#"..\"#), byte strings and nested block comments are
    handled. This is the corrected version of the strip-order bug the repo
    recorded against its own braces.py.
    """
    out = []
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        # line comment
        if c == "/" and i + 1 < n and text[i + 1] == "/":
            while i < n and text[i] != "\n":
                i += 1
            continue
        # block comment (Rust allows nesting)
        if c == "/" and i + 1 < n and text[i + 1] == "*":
            depth = 1
            i += 2
            while i < n and depth:
                if text[i] == "/" and i + 1 < n and text[i + 1] == "*":
                    depth += 1
                    i += 2
                elif text[i] == "*" and i + 1 < n and text[i + 1] == "/":
                    depth -= 1
                    i += 2
                else:
                    i += 1
            continue
        # raw / byte / c-string prefixes
        if c in "rbc":
            j = i
            if text[j] in "bc":
                j += 1
            if j < n and text[j] == "r":
                j += 1
                hashes = 0
                while j < n and text[j] == "#":
                    hashes += 1
                    j += 1
                if j < n and text[j] == '"':
                    term = '"' + "#" * hashes
                    k = text.find(term, j + 1)
                    i = (k + len(term)) if k != -1 else n
                    out.append('""')
                    continue
        # normal string
        if c == '"':
            i += 1
            while i < n:
                if text[i] == "\\":
                    i += 2
                    continue
                if text[i] == '"':
                    i += 1
                    break
                i += 1
            out.append('""')
            continue
        # char literal vs lifetime
        if c == "'":
            j = i + 1
            if j < n and text[j] == "\\":
                k = text.find("'", j)
                if k != -1 and k - j <= 4:
                    i = k + 1
                    out.append("''")
                    continue
            elif j + 1 < n and text[j + 1] == "'":
                i = j + 2
                out.append("''")
                continue
            i += 1  # lifetime: drop the quote only
            continue
        out.append(c)
        i += 1
    return "".join(out)


def tokens(text):
    return TOKEN_RE.findall(strip_code(text))


def items(text):
    found = {}
    for kind, rx in ITEM_RE.items():
        for m in rx.finditer(text):
            k = f"{kind}:{m.group(1)}"
            found[k] = found.get(k, 0) + 1
    return found


def balance(text):
    s = strip_code(text)
    return (s.count("{") - s.count("}"),
            s.count("(") - s.count(")"),
            s.count("[") - s.count("]"))


def main():
    tree = sys.argv[1]
    ours = sys.argv[2] if len(sys.argv) > 2 else "forge"
    theirs = sys.argv[3] if len(sys.argv) > 3 else "upstream/main"
    base = sys.argv[4] if len(sys.argv) > 4 else None
    fails = []

    # ---- markers over the whole tree (single git grep) -------------------
    # .hbs files are excluded: `<<<<<<< SEARCH line=N` / `>>>>>>> REPLACE` is a
    # SEARCH/REPLACE prompt template, a recorded false positive.
    out = git("grep", "-I", "-n", "-E", r"^(<{7}|>{7}|\|{7}) |^=======$", tree,
              "--", ":(exclude)*.hbs")
    marked = [ln for ln in out.splitlines() if ln]
    print(f"[markers] {len(marked)} line(s) with conflict markers in the tree")
    for m in marked[:20]:
        print("   ", m)
    if marked:
        fails.append("markers")

    # ---- deletion audit --------------------------------------------------
    # Only files the FORK changed (base != ours) may not silently disappear:
    # a file the fork never touched being deleted is upstream's business.
    status = git("diff", "--name-status", ours, tree).splitlines()
    added = [ln.split("\t", 1)[1] for ln in status if ln.startswith("A\t")]
    deleted = [ln.split("\t", 1)[1] for ln in status if ln.startswith("D\t")]
    unexplained = []
    for path in deleted:
        fork_changed = git("diff", "--quiet", base, ours, "--", path) != ""
        still_upstream = git("cat-file", "-e", f"{theirs}:{path}") == ""
        if fork_changed and still_upstream:
            unexplained.append(f"{path}: FORK-modified, deleted in merge, still upstream")
    print(f"[deletion] vs ours: {len(added)} added, {len(deleted)} deleted; "
          f"{len(unexplained)} unexplained (fork-modified and still upstream)")
    for u in unexplained[:20]:
        print("   ", u)
    if unexplained:
        fails.append("deletion")

    # ---- per-file audits on changed .rs files ----------------------------
    changed = [ln.split("\t", 1)[1] for ln in status
               if ln.split("\t", 1)[1].endswith(".rs")]
    b = Batch()
    noloss_fail, item_fail, bal_fail = [], [], []
    ours_only, theirs_only = [], []
    for path in changed:
        m = b.get(f"{tree}:{path}")
        o = b.get(f"{ours}:{path}")
        t = b.get(f"{theirs}:{path}")
        if m is None or o is None or t is None:
            continue
        mt, ot, tt = set(tokens(m)), set(tokens(o)), set(tokens(t))
        if base:
            bs = b.get(f"{base}:{path}")
            bt = set(tokens(bs)) if bs is not None else set()
        else:
            bt = set()
        missing_ours = sorted((ot - bt - tt) - mt)
        missing_theirs = sorted((tt - bt - ot) - mt)
        if missing_ours or missing_theirs:
            noloss_fail.append((path, missing_ours[:15], missing_theirs[:15]))
        mo, mm, mti = items(o), items(m), items(t)
        for name in set(mo) & set(mti):
            # defined by BOTH sides: the merged file must not have fewer
            if mm.get(name, 0) < max(mo.get(name, 0), mti.get(name, 0)):
                item_fail.append((path, name, mo.get(name, 0), mti.get(name, 0),
                                  mm.get(name, 0)))
        for name in set(mo) - set(mti):
            # only ours defines it: upstream deleted or renamed it
            if mm.get(name, 0) < mo[name]:
                ours_only.append((path, name, mo[name], mm.get(name, 0)))
        for name in set(mti) - set(mo):
            # only theirs defines it: upstream added or renamed it
            if mm.get(name, 0) < mti[name]:
                theirs_only.append((path, name, mti[name], mm.get(name, 0)))
        if base:
            bs = b.get(f"{base}:{path}")
            if bs is not None:
                bo, bt2, bm = balance(o), balance(t), balance(m)
                if bm != bo and bm != bt2:
                    bal_fail.append((path, bo, bt2, bm))
    print(f"[noloss] {len(noloss_fail)}/{len(changed)} file(s) with tokens missing")
    for f in noloss_fail[:20]:
        print(f"    {f[0]}\n        missing-ours={f[1]}\n        missing-theirs={f[2]}")
    print(f"[items] {len(item_fail)} file(s) with a definition count below a side")
    for f in item_fail[:20]:
        print(f"    {f[0]}: {f[1]} ours={f[2]} theirs={f[3]} merged={f[4]}")
    print(f"[balance] {len(bal_fail)} file(s) with balance differing from both sides")
    for f in bal_fail[:20]:
        print(f"    {f[0]}: ours={f[1]} theirs={f[2]} merged={f[3]}")
    print(f"[items-ours-only] {len(ours_only)} definition(s) only ours has, "
          f"not in the merged file (upstream delete/rename - needs triage)")
    for f in ours_only[:60]:
        print(f"    {f[0]}: {f[1]} ours={f[2]} merged={f[3]}")
    print(f"[items-theirs-only] {len(theirs_only)} definition(s) only theirs has, "
          f"not in the merged file (upstream add/rename - needs triage)")
    for f in theirs_only[:60]:
        print(f"    {f[0]}: {f[1]} theirs={f[2]} merged={f[3]}")
    if noloss_fail:
        fails.append("noloss")
    if item_fail:
        fails.append("items")
    if bal_fail:
        fails.append("balance")

    print()
    print("GATES:", ("FAIL -> " + ",".join(fails)) if fails else "PASS")
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main())
