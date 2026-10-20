#!/usr/bin/env python3
"""Show one conflict block of a merged file with shared context around it.

Usage: showctx.py <flat-name> <block-index> [before] [after]
Byte-exact (binary read).
"""
import os
import sys
from pathlib import Path

CTX = Path(os.environ.get("DX_MERGE_CTX", r"G:\Temp\UserTemp\opencode\dxmerge\ctx"))


def parse(raw: bytes):
    lines = raw.split(b"\n")
    out = []
    i, n = 0, len(lines)
    while i < n:
        if lines[i].startswith(b"<<<<<<< "):
            start = i
            ol = lines[i][8:].decode("utf-8", "replace")
            i += 1
            ours = []
            while i < n and not lines[i].startswith((b"|||||||", b"=======")):
                ours.append(lines[i]); i += 1
            if i < n and lines[i].startswith(b"|||||||"):
                i += 1
                while i < n and not lines[i].startswith(b"======="):
                    i += 1
            i += 1
            theirs = []
            while i < n and not lines[i].startswith(b">>>>>>> "):
                theirs.append(lines[i]); i += 1
            tl = lines[i][8:].decode("utf-8", "replace") if i < n else "?"
            out.append((start, i, ol, ours, tl, theirs))
        i += 1
    return out, lines


name, idx = sys.argv[1], int(sys.argv[2])
before = int(sys.argv[3]) if len(sys.argv) > 3 else 40
after = int(sys.argv[4]) if len(sys.argv) > 4 else 40

bs, lines = parse((CTX / f"{name}.merged").read_bytes())
if not 1 <= idx <= len(bs):
    sys.exit(f"no block {idx} (have {len(bs)})")
start, end, ol, ours, tl, theirs = bs[idx - 1]
print(f"### {name} block {idx} | merged lines {start+1}..{end+1} | SIDE1={ol} ({len(ours)}) SIDE2={tl} ({len(theirs)})")
print(f"--- context before ---")
for j in range(max(0, start - before), start):
    print(f"  {j+1:>6}| " + lines[j].decode("utf-8", "replace"))
print(f"--- conflict ---")
print(f"  {start+1:>6}| <<<<<<< {ol}")
for j, ln in enumerate(ours):
    print(f"  {'' :>6}| 1|{j+1:>4} " + ln.decode("utf-8", "replace"))
print(f"  {'':>6}| =======")
for j, ln in enumerate(theirs):
    print(f"  {'':>6}| 2|{j+1:>4} " + ln.decode("utf-8", "replace"))
print(f"  {end+1:>6}| >>>>>>> {tl}")
print(f"--- context after ---")
for j in range(end + 1, min(len(lines), end + 1 + after)):
    print(f"  {j+1:>6}| " + lines[j].decode("utf-8", "replace"))
