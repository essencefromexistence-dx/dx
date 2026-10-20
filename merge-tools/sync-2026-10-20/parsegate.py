#!/usr/bin/env python3
"""Parse gate: rustfmt must parse (and be format-clean) every changed .rs file
in the merged tree.

rustfmt is fed the file on STDIN: with a filename it tries to resolve `mod`
declarations and fails on files that are not on disk, which is a harness
artefact, not a defect.

A file that already failed at ours/theirs/base is reported as PRE-EXISTING;
only a file rustfmt accepts on a side but rejects in the merged tree is a
merge-introduced defect.

Usage: python parsegate.py <merged-tree> [ours] [theirs] [base]
"""
import os
import subprocess
import sys

REPO = os.environ.get("DX_CODE_REPO", r"G:\Dx\code")
RUSTFMT = r"D:\DevCaches\cargo\bin\rustfmt.exe"
CONFIG = os.path.join(REPO, "rustfmt.toml")


def git(*args):
    p = subprocess.run(["git", "-C", REPO, *args], stdout=subprocess.PIPE,
                       stderr=subprocess.DEVNULL)
    return p.stdout.decode("utf-8", "replace")


def blob(rev, path):
    p = subprocess.run(["git", "-C", REPO, "cat-file", "blob", f"{rev}:{path}"],
                       stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    return p.stdout if p.returncode == 0 else None


def rustfmt(text):
    """Return (parsed_ok, format_clean, first_error_line)."""
    p = subprocess.run([RUSTFMT, "--edition", "2024", "--emit", "stdout",
                        "--config-path", CONFIG],
                       input=text, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    err = p.stderr.decode("utf-8", "replace")
    first = ""
    for ln in err.splitlines():
        if ln.strip():
            first = ln.strip()
            break
    if p.returncode != 0:
        return False, False, first or f"rustfmt rc={p.returncode}"
    return True, p.stdout == text, first


def main():
    tree = sys.argv[1]
    ours = sys.argv[2] if len(sys.argv) > 2 else "forge"
    theirs = sys.argv[3] if len(sys.argv) > 3 else "upstream/main"
    base = sys.argv[4] if len(sys.argv) > 4 else None
    status = git("diff", "--name-status", ours, tree).splitlines()
    files = [ln.split("\t", 1)[1] for ln in status
             if ln.split("\t", 1)[1].endswith(".rs")]
    introduced, preexisting, unformatted, ok = [], [], [], 0
    for path in files:
        text = blob(tree, path)
        if text is None:
            continue
        parsed, clean, err = rustfmt(text)
        if parsed and clean:
            ok += 1
            continue
        sides = {}
        for label, rev in (("ours", ours), ("theirs", theirs), ("base", base)):
            if rev is None:
                continue
            t = blob(rev, path)
            sides[label] = None if t is None else rustfmt(t)
        side_ok = {k: (v[0] and v[1]) for k, v in sides.items() if v is not None}
        if not parsed:
            if any(v[0] for v in sides.values() if v is not None):
                introduced.append((path, err, sides))
            else:
                preexisting.append((path, err, sides))
        else:
            unformatted.append((path, sides))
    print(f"[parsegate] {len(files)} changed .rs file(s): {ok} parse+format clean, "
          f"{len(preexisting)} pre-existing parse failure(s), "
          f"{len(introduced)} merge-introduced parse failure(s), "
          f"{len(unformatted)} parse-ok but not format-clean")
    for p in preexisting[:40]:
        print(f"    PRE-EXISTING {p[0]}\n        {p[1]}")
    for p in introduced[:40]:
        print(f"    INTRODUCED    {p[0]}\n        {p[1]}\n        sides: {p[2]}")
    for p in unformatted[:40]:
        print(f"    UNFORMATTED   {p[0]}\n        sides: {p[1]}")
    return 1 if introduced else 0


if __name__ == "__main__":
    sys.exit(main())
