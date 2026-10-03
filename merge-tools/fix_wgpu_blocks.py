#!/usr/bin/env python3
"""Resolve the 4 remaining conflict blocks in gpui_wgpu/src/wgpu_renderer.rs.

Each block's "theirs" side is the already-integrated (upstream-structure + our-feature)
content; the merged file adopted upstream's RendererState/WgpuRendererCore split, so the
HEAD side is the stale pre-restructure text. Choices below are deliberate, one per block.
"""
import io, sys

P = r"G:/Dx/code/crates/gpui_wgpu/src/wgpu_renderer.rs"
CHOICES = ["theirs", "theirs", "theirs", "theirs"]

raw = open(P, "r", encoding="utf-8", newline="").read()
lines = raw.split("\n")
crlf = raw.count("\r\n")
print(f"file: {len(lines)} lines, crlf={crlf}, lf_only={raw.count(chr(10))-crlf}")

def clean(s):
    return s[:-1] if s.endswith("\r") else s

blocks = []
i = 0
while i < len(lines):
    if clean(lines[i]).startswith("<<<<<<<"):
        s = i; m = None; e = None
        j = i + 1
        while j < len(lines):
            c = clean(lines[j])
            if c == "=======" and m is None:
                m = j
            elif c.startswith(">>>>>>>"):
                e = j; break
            j += 1
        assert m is not None and e is not None, f"unterminated block at line {s+1}"
        blocks.append((s, m, e))
        i = e + 1
    else:
        i += 1

print(f"found {len(blocks)} conflict blocks at lines: {[(s+1, m+1, e+1) for s, m, e in blocks]}")
assert len(blocks) == len(CHOICES), f"expected {len(CHOICES)} blocks, found {len(blocks)}"

# Apply bottom-up so earlier indices stay valid.
for (s, m, e), choice in sorted(zip(blocks, CHOICES), key=lambda x: -x[0][0]):
    ours = lines[s + 1:m]
    theirs = lines[m + 1:e]
    body = ours if choice == "ours" else theirs
    print(f"  block {s+1}-{e+1}: chose {choice} "
          f"(ours {len(ours)} lines, theirs {len(theirs)} lines) -> {len(body)} lines")
    lines[s:e + 1] = body

out = "\n".join(lines)
open(P, "w", encoding="utf-8", newline="").write(out)
print(f"wrote {P}: {len(lines)} lines")
