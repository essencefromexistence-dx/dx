"""Restore the IconName variants the merge silently dropped.

The merge kept 295 of ours' 304 IconName variants and all 14 of upstream's additions,
but DROPPED 9 of ours. Only 4 were caught by the compiler (they are the ones
crates/ui/src/dx_icons.rs happens to reference); the other 5 are silent losses.

This script inserts each missing variant at its alphabetical position in the merged
enum, so the enum stays sorted exactly as it was.

Dry run:  python fix_icons_enum.py --dry
Apply:    python fix_icons_enum.py --apply
"""

import sys
from pathlib import Path

PATH = Path("G:/Dx/code/crates/icons/src/icons.rs")
MISSING = [
    "FolderOpenAdd",
    "Library",
    "ListFilter",
    "LockOutlined",
    "MenuAltTemp",
    "NewThread",
    "OpenFolder",
    "Sliders",
    "ToolFolder",
]

raw = PATH.read_bytes()
eol = b"\r\n" if b"\r\n" in raw else b"\n"
text = raw.decode("utf-8")
lines = text.split(eol.decode())

start = next(i for i, l in enumerate(lines) if l.startswith("pub enum IconName {"))
end = next(i for i in range(start + 1, len(lines)) if lines[i] == "}")

# the enum body: variant lines only, in file order
body_idx = [
    i for i in range(start + 1, end) if lines[i].startswith("    ") and lines[i].endswith(",")
]
present = [lines[i].strip().rstrip(",") for i in body_idx]

print(f"enum body: lines {start + 1}..{end + 1}, {len(body_idx)} variants")
print(f"EOL: {eol!r}")
print()

# sanity: the enum is essentially sorted, with exactly two pre-existing upstream
# anomalies that are unrelated to this repair. Whitelist those and refuse anything else.
KNOWN_UNSORTED = {("Bookmark", "BoltFilled"), ("TextWrap", "TextUnwrap")}
violations = [
    (present[k], present[k + 1])
    for k in range(len(present) - 1)
    if present[k] > present[k + 1] and (present[k], present[k + 1]) not in KNOWN_UNSORTED
]
assert not violations, f"unexpected ordering violations: {violations}"
print(f"ordering: OK (2 known upstream anomalies whitelisted)")
print()

planned = []
for name in MISSING:
    if name in present:
        print(f"  SKIP {name}: already present")
        continue
    # first existing variant that sorts after `name` -> insert before it
    ins = next((k for k, v in enumerate(present) if v > name), None)
    if ins is None:
        raise SystemExit(f"cannot place {name}: no successor found")
    print(f"  {name:16s} -> before {present[ins]:16s} (body line {body_idx[ins] + 1})")
    planned.append((body_idx[ins], name))

print()
if "--dry" in sys.argv or "--apply" not in sys.argv:
    print("DRY RUN - nothing written")
    sys.exit(0)

# insert bottom-up so earlier indices stay valid
for idx, name in sorted(planned, reverse=True):
    lines.insert(idx, f"    {name},")

PATH.write_bytes(eol.join(l.encode("utf-8") for l in lines))
print(f"WROTE {PATH}  (+{len(planned)} variants)")
