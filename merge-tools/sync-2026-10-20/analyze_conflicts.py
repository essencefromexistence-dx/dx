#!/usr/bin/env python3
"""Dump conflict blocks from the dx-code merge scratch context.

Byte-exact: files are read as binary and split on b"\n" so CRLF/LF in the
source blob is preserved and never silently normalised.

Handles both merge-ort marker styles:
  2-way:  <<<<<<< ours / ======= / >>>>>>> theirs
  3-way:  <<<<<<< ours / ||||||| base / ======= / >>>>>>> theirs
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
            ours_label = lines[i][8:].decode("utf-8", "replace")
            i += 1
            ours = []
            while i < n and not lines[i].startswith((b"|||||||", b"=======")):
                ours.append(lines[i]); i += 1
            base = None
            if i < n and lines[i].startswith(b"|||||||"):
                i += 1
                base = []
                while i < n and not lines[i].startswith(b"======="):
                    base.append(lines[i]); i += 1
            i += 1  # skip =======
            theirs = []
            while i < n and not lines[i].startswith(b">>>>>>> "):
                theirs.append(lines[i]); i += 1
            end = i
            theirs_label = lines[i][8:].decode("utf-8", "replace") if i < n else "?"
            out.append((start + 1, end + 1, ours_label, ours, base, theirs_label, theirs))
        i += 1
    return out, lines


def show(name: str, max_side: int = 120):
    p = CTX / f"{name}.merged"
    bs, lines = parse(p.read_bytes())
    print(f"===== {name} : {len(bs)} block(s), {len(lines)} lines =====")
    for k, (s, e, ol, ours, base, tl, theirs) in enumerate(bs, 1):
        print(f"--- block {k}: merged lines {s}..{e} | SIDE1({ol}) {len(ours)} lines | "
              f"SIDE2({tl}) {len(theirs)} lines | base {0 if base is None else len(base)} lines")
        for label, side in (("1", ours), ("2", theirs)):
            body = side[:max_side]
            for j, ln in enumerate(body):
                print(f"  {label}{j+1:>4}| " + ln.decode("utf-8", "replace"))
            if len(side) > max_side:
                print(f"  {label} ... ({len(side) - max_side} more lines)")
    print()


if __name__ == "__main__":
    names = sys.argv[1:]
    if not names:
        names = sorted({p.name[:-7] for p in CTX.glob("*.merged")})
    for nm in names:
        show(nm)
