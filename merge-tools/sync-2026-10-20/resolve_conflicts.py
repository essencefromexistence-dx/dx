#!/usr/bin/env python3
"""Resolve conflict blocks in the dx-code merge scratch context.

Spec-driven and byte-exact. A spec names the flat context name and, per
1-based conflict-block index, the exact replacement lines.

    specs/<flat>.json = {
      "file": "crates/gpui/src/scene.rs",
      "blocks": {
        "1": ["line one", "line two"]
      }
    }

The tool:
  * parses the .merged blob into blocks (binary read, no EOL normalisation),
  * replaces each named block with its resolution,
  * REFUSES to write if a conflict marker survives, or if the spec's block
    count does not match the merged blob's block count,
  * writes <flat>.resolved and prints a per-block report.

Usage: python resolve.py <flat-name> [<flat-name> ...]
"""
import json
import os
import sys
from pathlib import Path

ROOT = Path(os.environ.get("DX_MERGE_ROOT", r"G:\Temp\UserTemp\opencode\dxmerge"))
CTX = ROOT / "ctx"
SPECS = Path(os.environ.get("DX_MERGE_SPECS", str(ROOT / "specs")))

MARKER_PREFIXES = (b"<<<<<<< ", b">>>>>>> ", b"||||||| ")


def parse(raw: bytes):
    """Return (blocks, lines). blocks = [(start_line_idx, end_line_idx, side1, side2)]."""
    lines = raw.split(b"\n")
    blocks = []
    i, n = 0, len(lines)
    while i < n:
        if lines[i].startswith(b"<<<<<<< "):
            start = i
            i += 1
            side1 = []
            while i < n and not lines[i].startswith((b"||||||| ", b"=======")):
                side1.append(lines[i]); i += 1
            if i < n and lines[i].startswith(b"||||||| "):
                i += 1
                while i < n and not lines[i].startswith(b"======="):
                    i += 1
            i += 1
            side2 = []
            while i < n and not lines[i].startswith(b">>>>>>> "):
                side2.append(lines[i]); i += 1
            blocks.append((start, i, side1, side2))
        i += 1
    return blocks, lines


def marker_count(raw: bytes) -> int:
    n = 0
    for ln in raw.split(b"\n"):
        if ln.startswith(MARKER_PREFIXES) or ln == b"=======":
            n += 1
    return n


def resolve(flat: str) -> int:
    spec = json.loads((SPECS / f"{flat}.json").read_text(encoding="utf-8"))
    raw = (CTX / f"{flat}.merged").read_bytes()
    blocks, _ = parse(raw)
    want = spec["blocks"]
    print(f"=== {flat}  ({spec['file']}) : {len(blocks)} block(s) in merged, "
          f"{len(want)} in spec")
    if len(want) != len(blocks):
        print(f"  ABORT: spec covers {len(want)} of {len(blocks)} blocks")
        return 1

    out = raw
    for idx in sorted((int(k) for k in want), reverse=True):
        blocks2, lines2 = parse(out)
        s2, e2, side1, side2 = blocks2[idx - 1]
        off = 0
        for j in range(s2):
            off += len(lines2[j]) + 1
        end_off = off
        for j in range(s2, e2 + 1):
            end_off += len(lines2[j]) + 1
        old = out[off:end_off]
        lines_want = want[str(idx)]
        # "@SIDE1" / "@SIDE2" expand to that side's lines, verbatim
        expanded = []
        for ln in lines_want:
            if ln == "@SIDE1":
                expanded.extend(x.decode("utf-8", "replace") for x in side1)
            elif ln == "@SIDE2":
                expanded.extend(x.decode("utf-8", "replace") for x in side2)
            else:
                expanded.append(ln)
        lines_want = expanded
        if lines_want:
            replacement = ("\n".join(lines_want) + "\n").encode("utf-8")
        else:
            replacement = b""
        out = out[:off] + replacement + out[end_off:]
        print(f"  block {idx}: {old.count(chr(10).encode())} "
              f"old lines -> {len(lines_want)} new lines")

    # anchored edits outside the conflict blocks (eolpatch.py semantics)
    for i, rep in enumerate(spec.get("edits", []), 1):
        label = rep.get("label", f"edit {i}")
        expect = rep.get("expect", 1)
        old_b = ("\n".join(rep["old"]) + "\n").encode("utf-8")
        new_b = ("\n".join(rep["new"]) + "\n").encode("utf-8")
        n = out.count(old_b)
        if n != expect:
            print(f"  ABORT: edit {i} ({label}): found {n} matches, expected {expect}")
            return 1
        out = out.replace(old_b, new_b, expect)
        print(f"  edit {i}: OK ({n} match) - {label}")

    left = marker_count(out)
    if left:
        print(f"  ABORT: {left} conflict marker(s) survive")
        return 1
    dst = CTX / f"{flat}.resolved"
    dst.write_bytes(out)
    print(f"  WROTE {dst}  ({len(out)} bytes)")
    return 0


if __name__ == "__main__":
    rc = 0
    for flat in sys.argv[1:]:
        rc |= resolve(flat)
    sys.exit(rc)
