#!/usr/bin/env python3
"""Static consistency check: every `self.X` inside an `impl` block must be a field of the
impl'd type or a method defined on it. Handles BOTH `impl T {` and `impl Trait for T {`.
Catches the marker-free class where a merge moves a method to the other half of a split impl.
Usage: check_impl_consistency2.py <file.rs>
"""
import re, sys

P = sys.argv[1]
lines = open(P, encoding="utf-8", newline="").read().replace("\r\n", "\n").split("\n")

RE_IMPL_PLAIN = re.compile(r"^impl(?:<[^>]*>)?\s+(\w+)(?:<[^>]*>)?\s*\{")
RE_IMPL_FOR = re.compile(r"^impl(?:<[^>]*>)?\s+.*?\bfor\s+(\w+)(?:<[^>]*>)?\s*\{")

fields = {}
for i, l in enumerate(lines):
    m = re.match(r"(?:pub )?struct (\w+)(?:<[^>]*>)? \{", l)
    if not m:
        continue
    f = set()
    j = i + 1
    while j < len(lines) and not lines[j].startswith("}"):
        fm = re.match(r"\s*(?:pub(?:\([^)]*\))? )?(\w+):", lines[j])
        if fm:
            f.add(fm.group(1))
        j += 1
    fields[m.group(1)] = f

blocks = []
for i, l in enumerate(lines):
    m = RE_IMPL_PLAIN.match(l) or RE_IMPL_FOR.match(l)
    if m:
        depth = 1
        j = i + 1
        while j < len(lines) and depth > 0:
            depth += lines[j].count("{") - lines[j].count("}")
            j += 1
        blocks.append((m.group(1), i, j - 1))
print(f"impl blocks: {len(blocks)}")

methods = {}
for t, s, e in blocks:
    for k in range(s + 1, e):
        m = re.match(r"\s*(?:pub(?:\([^)]*\))? )?(?:async )?fn (\w+)", lines[k])
        if m:
            methods.setdefault(t, set()).add(m.group(1))

total = 0
for t, s, e in blocks:
    f = fields.get(t, set())
    ms = methods.get(t, set())
    bad = {}
    for k in range(s + 1, e):
        for m in re.finditer(r"self\.(\w+)", lines[k]):
            n = m.group(1)
            if n in f or n in ms or n == "0":
                continue
            bad.setdefault(n, []).append(k + 1)
    if bad:
        total += len(bad)
        print(f"\n!! impl {t} (lines {s+1}-{e+1}) unresolved self.X:")
        for n, v in sorted(bad.items()):
            print(f"     {n} -> {v[:8]} (count {len(v)})")
print("\nTOTAL unresolved distinct names:", total)
sys.exit(1 if total else 0)
