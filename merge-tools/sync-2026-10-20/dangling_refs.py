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


def main():
    tree, ours, theirs, base = sys.argv[1:5]
    max_hits = int(sys.argv[5]) if len(sys.argv) > 5 else 6

    changed = [ln.split("\t", 1)[1] for ln in git("diff", "--name-status", ours, tree).splitlines()]
    rs_files = [p for p in changed if p.endswith(".rs")]

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
        for name in re.findall(r"\b(" + "|".join(re.escape(n) for n in all_names) + r")\b", text):
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
