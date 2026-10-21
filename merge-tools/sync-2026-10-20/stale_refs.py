#!/usr/bin/env python3
"""Stale-rename sweep.

The merge's nastiest failure mode is a *newly added* line that references an
item upstream renamed or deleted:

    the fork adds `fn should_block_mouse_passthrough()` containing
    `self.scroll_wheel_listeners`
    upstream renames the field to `scroll_listeners`
    -> the merge applies upstream's rename to every line that existed in base,
       but the fork's line was an ADDITION, so upstream's rename cannot reach
       it. No conflict, no marker, and the identifier is still *present* -
       so a no-loss audit sees nothing wrong.

The result is a compile error (E0609) that no structural gate can see.

This sweep finds it: it lists every item and struct-field name that upstream
REMOVED or renamed in the window, and reports where the merged tree still
refers to the old name.

Usage: python stale_refs.py <merged-tree> <ours> <theirs> <base>
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
FIELD_RE = re.compile(rf"^\s+(?:pub(?:\([^)]*\))?\s+)?([a-z_][A-Za-z0-9_]*)\s*:\s*[^:]", re.M)


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
    for m in FIELD_RE.finditer(text):
        found.add(m.group(1))
    return found


def main():
    tree, ours, theirs, base = sys.argv[1:5]
    changed = [ln.split("\t", 1)[1] for ln in git("diff", "--name-status", theirs, base).splitlines()
               if ln.split("\t", 1)[1].endswith(".rs")]

    removed = {}
    for path in changed:
        b = blob(base, path)
        t = blob(theirs, path)
        if b is None or t is None:
            continue
        for name in items(b) - items(t):
            removed.setdefault(name, set()).add(path)

    # names the FORK also uses - those are the fork's own, not stale
    fork_names = set()
    for path in changed:
        o = blob(ours, path)
        if o is not None:
            fork_names |= items(o)

    names = sorted(removed)
    if not names:
        print("[stale] no removed names")
        return 0
    pat = r"\b(" + "|".join(re.escape(n) for n in names) + r")\b"
    hits = {}
    for line in git("grep", "-n", "-w", "-E", pat, tree, "--", "*.rs").splitlines():
        m = re.match(r"^[^:]+:([^:]+):(\d+):(.*)$", line)
        if not m:
            continue
        path, ln, text = m.group(1), int(m.group(2)), m.group(3)
        for name in re.findall(r"\b(" + "|".join(re.escape(n) for n in names) + r")\b", text):
            hits.setdefault(name, []).append((path, ln, text.strip()[:140]))

    # a name is STALE if the merged tree has NO definition of it anywhere
    defpat = (r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+|unsafe\s+)*"
              r"(?:fn|struct|enum|trait|type|const|static)\s+("
              + "|".join(re.escape(n) for n in names) + r")\b")
    defs = set()
    for line in git("grep", "-h", "-P", defpat, tree, "--", "*.rs").splitlines():
        m = re.search(r"\b(fn|struct|enum|trait|type|const|static)\s+([A-Za-z_][A-Za-z0-9_]*)", line)
        if m:
            defs.add(m.group(2))
    # struct fields (declaration form `name: Type`, not `name: value`)
    fieldpat = (r"^\s+(?:pub(?:\([^)]*\))?\s+)?("
                + "|".join(re.escape(n) for n in names) + r")\s*:\s*[A-Za-z_&]")
    for line in git("grep", "-h", "-P", fieldpat, tree, "--", "*.rs").splitlines():
        m = re.match(r"^\s+(?:pub(?:\([^)]*\))?\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:", line)
        if m:
            defs.add(m.group(1))
    # type aliases and `use` re-exports count as definitions too
    for line in git("grep", "-h", "-P",
                    r"^\s*(?:pub(?:\([^)]*\))?\s+)?type\s+([A-Za-z_][A-Za-z0-9_]*)\s*=",
                    tree, "--", "*.rs").splitlines():
        m = re.search(r"\btype\s+([A-Za-z_][A-Za-z0-9_]*)", line)
        if m:
            defs.add(m.group(1))
    for line in git("grep", "-h", "-P",
                    r"\b(" + "|".join(re.escape(n) for n in names) + r")\s*(?:,|\};)",
                    tree, "--", "*.rs").splitlines():
        m = re.search(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*(?:,|\};)", line)
        if m and ("use " in line or "pub use " in line):
            defs.add(m.group(1))

    stale = {n: h for n, h in hits.items() if n not in defs}
    print(f"[stale] upstream removed/renamed {len(removed)} name(s); "
          f"{len(hits)} still appear in the merged tree; "
          f"**{len(stale)} have no definition anywhere**")
    for name in sorted(stale):
        print(f"\n  !! {name}   (removed by upstream in: "
              f"{', '.join(sorted(removed[name])[:3])})")
        for path, ln, text in stale[name][:8]:
            print(f"       {path}:{ln}: {text}")
        if len(stale[name]) > 8:
            print(f"       ... and {len(stale[name]) - 8} more")
    return 1 if stale else 0


if __name__ == "__main__":
    sys.exit(main())
