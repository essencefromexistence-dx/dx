#!/usr/bin/env python3
"""Nested-item silent-loss audit for the CURRENT sync (e91b82c106 into forge).

Adapted from merge-tools/nested_audit.py: ours is HEAD (33bbb8fe13), theirs is the
merge target (e91b82c106), base is the merge-base (f8adffe9d1). Scoped to the two files
this session resolved.
"""
import re, subprocess, sys
from pathlib import Path

REPO = Path("G:/Dx/code")
OURS = "33bbb8fe13"
THEIRS = "e91b82c106"
BASE = "f8adffe9d1"
FILES = [
    "crates/gpui_wgpu/src/wgpu_renderer.rs",
    "crates/language_models/src/provider/opencode.rs",
]

IDENT = r"[A-Za-z_][A-Za-z0-9_]*"
ENUM_RE = re.compile(rf"\benum\s+({IDENT})")
STRUCT_RE = re.compile(rf"\bstruct\s+({IDENT})")
VARIANT_RE = re.compile(rf"^\s*({IDENT})\s*[,=({{]")
FIELD_RE = re.compile(rf"^\s*(?:pub(?:\([^)]*\))?\s+)?({IDENT})\s*:")


def show(rev, path):
    p = subprocess.run(["git", "show", f"{rev}:{path}"], cwd=REPO,
                       stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    return p.stdout.decode("utf-8", "replace") if p.returncode == 0 else None


def block_body(text, header_re):
    for m in header_re.finditer(text):
        i = text.find("{", m.end())
        if i == -1:
            continue
        depth, j = 0, i
        while j < len(text):
            c = text[j]
            if c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        yield m.group(1), text[i + 1:j]


def items(text):
    found = set()
    for name, body in block_body(text, ENUM_RE):
        for line in body.splitlines():
            line = line.strip()
            if not line or line.startswith(("//", "#", "///")):
                continue
            m = VARIANT_RE.match(" " + line)
            if m and m.group(1)[0].isupper():
                found.add(f"{name}::{m.group(1)}")
    for name, body in block_body(text, STRUCT_RE):
        for line in body.splitlines():
            line = line.strip()
            if not line or line.startswith(("//", "#", "///")):
                continue
            m = FIELD_RE.match(" " + line)
            if m:
                found.add(f"{name}.{m.group(1)}")
    return found


for f in FILES:
    merged = (REPO / f).read_text(encoding="utf-8", errors="replace")
    o, t, b = show(OURS, f), show(THEIRS, f), show(BASE, f)
    if o is None or t is None:
        print(f"{f}: skipped (missing side)")
        continue
    bi = items(b) if b else set()
    mi = items(merged)
    print("=" * 78)
    print(f)
    for label, side in (("OURS", o), ("THEIRS", t)):
        added = items(side) - bi
        missing = sorted(added - mi)
        print(f"  {label}: added-vs-base {len(added)}, MISSING from merged: {len(missing)}")
        for x in missing[:40]:
            print(f"      - {x}")
        if len(missing) > 40:
            print(f"      ... +{len(missing)-40} more")
