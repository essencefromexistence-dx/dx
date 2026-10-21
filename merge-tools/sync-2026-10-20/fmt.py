#!/usr/bin/env python3
r"""Show rustfmt's proposed changes for a file in a tree, and optionally write
the formatted blob back into the scratch index.

rustfmt is fed the file on STDIN: with a filename it tries to resolve `mod`
declarations and fails on files that are not on disk, which is a harness
artefact, not a defect.

Usage: python fmt.py <tree> <path> [--apply]
Env: DX_CODE_REPO (default G:\Dx\code), DX_MERGE_INDEX (default: none)
"""
import difflib
import os
import subprocess
import sys

REPO = os.environ.get("DX_CODE_REPO", r"G:\Dx\code")
RUSTFMT = os.environ.get("DX_RUSTFMT", r"D:\DevCaches\cargo\bin\rustfmt.exe")
CONFIG = os.path.join(REPO, "rustfmt.toml")


def blob(rev, path):
    p = subprocess.run(["git", "-C", REPO, "cat-file", "blob", f"{rev}:{path}"],
                       stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    return p.stdout if p.returncode == 0 else None


def fmt(text):
    p = subprocess.run([RUSTFMT, "--edition", "2024", "--emit", "stdout",
                        "--config-path", CONFIG],
                       input=text, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    return p.stdout, p.returncode, p.stderr.decode("utf-8", "replace")


def main():
    tree, path = sys.argv[1], sys.argv[2]
    apply = "--apply" in sys.argv
    text = blob(tree, path)
    if text is None:
        sys.exit(f"no blob for {path}")
    out, rc, err = fmt(text)
    if rc != 0:
        sys.exit(f"rustfmt failed: {err}")
    if out == text:
        print(f"{path}: already format-clean")
        return
    diff = list(difflib.unified_diff(
        text.decode("utf-8", "replace").splitlines(),
        out.decode("utf-8", "replace").splitlines(),
        fromfile="merged", tofile="rustfmt", lineterm="", n=2))
    print(f"{path}: {len(diff)} diff line(s)")
    for ln in diff[:400]:
        print("   " + ln)
    if apply:
        index = os.environ.get("DX_MERGE_INDEX")
        if not index:
            sys.exit("set DX_MERGE_INDEX to the scratch index before --apply")
        sha = subprocess.run(["git", "-C", REPO, "hash-object", "-w", "--stdin"],
                             input=out, stdout=subprocess.PIPE).stdout.decode().strip()
        env = dict(os.environ, GIT_INDEX_FILE=index)
        r = subprocess.run(["git", "-C", REPO, "update-index", "--add",
                            "--cacheinfo", f"644,{sha},{path}"],
                           stdout=subprocess.PIPE, stderr=subprocess.STDOUT, env=env)
        print(f"installed {sha} rc={r.returncode} {r.stdout.decode().strip()}")


if __name__ == "__main__":
    main()
