#!/usr/bin/env python3
r"""Dangling-reference gate.

The tree-wide gate suite proves that nothing the fork *added* was lost. It
cannot see the opposite failure: upstream DELETED a definition, the merge
applied the deletion, and some other file in the merged tree still calls it.
That is a compile error, and it is invisible to a marker/token/balance audit.

This gate lists every item definition that exists on a side but not in the
merged tree, and then searches the whole merged tree for a whole-word
reference to it.

Usage: python dangling_refs.py <merged-tree> <ours> <theirs> <base> [max-hits]
Env: DX_CODE_REPO (default G:\Dx\code)
"""
import os
import re
import subprocess
import sys

REPO = os.environ.get("DX_CODE_REPO", r"G:\Dx\code")
IDENT = r"[A-Za-z_][A-Za-z0-9_]*"
ITEM_RE = {
    "fn": re.compile(rf"\bfn\s+({IDENT})"),
    "struct": re.compile(rf"\bstruct\s+({IDENT})"),
    "enum": re.compile(rf"\benum\s+({IDENT})"),
    "trait": re.compile(rf"\btrait\s+({IDENT})"),
    "type": re.compile(rf"\btype\s+({IDENT})"),
    "const": re.compile(rf"\bconst\s+({IDENT})"),
    "static": re.compile(rf"\bstatic\s+({IDENT})"),
}


def git(*args):
    p = subprocess.run(["git", "-C", REPO, *args], stdout=subprocess.PIPE,
                       stderr=subprocess.DEVNULL)
    return p.stdout.decode("utf-8", "replace")


def blob(rev, path):
    p = subprocess.run(["git", "-C", REPO, "cat-file", "blob", f"{rev}:{path}"],
                       stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    return p.stdout.decode("utf-8", "replace") if p.returncode == 0 else None


def items(text):
    found = set()
    for rx in ITEM_RE.values():
        for m in rx.finditer(text):
            found.add(m.group(1))
    return found


def strip_comments_and_strings(line: str) -> str:
    """Drop `//` comments and string/char literals from one line.

    A mention of an identifier inside a comment or a literal is documentation,
    not a reference - and reporting it as a dangling reference is exactly the
    kind of noise that makes a gate get ignored.
    """
    out = []
    i, n = 0, len(line)
    while i < n:
        c = line[i]
        if c == "/" and i + 1 < n and line[i + 1] == "/":
            break
        if c == '"':
            i += 1
            while i < n:
                if line[i] == "\\":
                    i += 2
                    continue
                if line[i] == '"':
                    i += 1
                    break
                i += 1
            out.append('""')
            continue
        out.append(c)
        i += 1
    return "".join(out)


def main():
    tree, ours, theirs, base = sys.argv[1:5]
    max_hits = int(sys.argv[5]) if len(sys.argv) > 5 else 6
    # optional 6th arg: "all" widens the sweep to every .rs file in the tree.
    # The default scope is the files the MERGE changed - that is the merge's
    # blast radius, and the only place a merge can introduce a break. A
    # pre-existing fork break in a file the merge did NOT touch is outside
    # that radius, and only a compile finds it.
    sweep_all = len(sys.argv) > 6 and sys.argv[6] == "all"

    changed = [ln.split("\t", 1)[1] for ln in git("diff", "--name-status", ours, tree).splitlines()]
    rs_files = [p for p in changed if p.endswith(".rs")]
    if sweep_all:
        rs_files = [p for p in git("ls-tree", "-r", "--name-only", tree).splitlines()
                    if p.endswith(".rs")]
        print(f"[dangling] sweeping ALL {len(rs_files)} .rs files in the tree")

    merged_items = {}
    merged_files = {}
    for path in rs_files:
        t = blob(tree, path)
        if t is None:
            continue
        merged_files[path] = t
        merged_items[path] = items(t)

    # every identifier defined on a side but absent from the merged file
    suspects = {}
    for path in rs_files:
        for rev, label in ((ours, "ours"), (theirs, "theirs"), (base, "base")):
            t = blob(rev, path)
            if t is None:
                continue
            gone = items(t) - merged_items.get(path, set())
            for name in gone:
                suspects.setdefault(name, []).append(f"{label}:{path}")

    # whole-word references anywhere in the merged tree
    all_names = sorted(suspects)
    if not all_names:
        print("[dangling] no suspects")
        return 0
    pat = r"\b(" + "|".join(re.escape(n) for n in all_names) + r")\b"
    out = git("grep", "-n", "-w", "-E", pat, tree, "--", "*.rs")

    hits = {}
    for line in out.splitlines():
        m = re.match(r"^[^:]+:([^:]+):(\d+):(.*)$", line)
        if not m:
            continue
        path, ln, text = m.group(1), int(m.group(2)), m.group(3)
        # a mention inside a comment or a string literal is NOT a reference
        code = strip_comments_and_strings(text)
        for name in re.findall(r"\b(" + "|".join(re.escape(n) for n in all_names) + r")\b", code):
            hits.setdefault(name, []).append((path, ln, text.strip()[:150]))

    print(f"[dangling] {len(suspects)} identifier(s) defined on a side but absent "
          f"from the merged tree; {len(hits)} of them still referenced there")
    for name in sorted(hits):
        places = hits[name]
        print(f"\n  !! {name}   (defined at: {', '.join(sorted(set(suspects[name]))[:4])})")
        for path, ln, text in places[:max_hits]:
            print(f"       {path}:{ln}: {text}")
        if len(places) > max_hits:
            print(f"       ... and {len(places) - max_hits} more")
    if not hits:
        print("  (none referenced - clean)")
    return 1 if hits else 0


if __name__ == "__main__":
    sys.exit(main())
