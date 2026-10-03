"""Nested-item silent-loss audit for the dx-code merge.

WHY THIS EXISTS
---------------
The merge's existing no-loss gates are token-based, and `itemparity.py` only compared
TOP-LEVEL items (fn/struct/enum/trait/type/const/static/mod). Neither can see a loss
of a NESTED item -- and that is exactly what happened with `IconName`: the merge dropped
9 of ours' 304 enum variants, yet the token `Sliders` was still present elsewhere in the
tree (crates/ui/src/dx_icons.rs), so every token-level check passed. Only 4 of the 9
broke the build.

This script compares NESTED items across ours / theirs / merged:
  * enum variants      (inside `enum X { ... }`)
  * struct fields      (inside `struct X { ... }`)

and reports anything present on a side but MISSING from the merged tree:
  * in OURS   but not merged -> LOCAL loss      (policy violation: local must be 100 %)
  * in THEIRS but not merged -> REMOTE loss     (policy violation: remote must be 100 %)

Usage:  python nested_audit.py [--limit N]
"""

import re
import subprocess
import sys
from pathlib import Path

REPO = Path("G:/Dx/code")
OURS = "b5142ba1aa"
THEIRS = "a84689073d"
BASE = "15dc0e9fa0"  # real merge-base: the 2nd parent of forge's previous merge

# THE CRUCIAL FILTER, per the merge docs:
#   "ours == base  =>  upstream's removal, NOT our loss"
# So an item only counts as a LOSS if a side ADDED it relative to the base and the
# merged tree then lacks it. Comparing ours-vs-merged alone floods the report with
# upstream restructures (e.g. `enum ContentBlock` exists in base AND ours, and
# upstream moved it elsewhere -- dropping it is upstream's change, not ours).

IDENT = r"[A-Za-z_][A-Za-z0-9_]*"


def git_batch(specs):
    """git cat-file --batch for a list of '<rev>:<path>' -> {spec: text|None}."""
    if not specs:
        return {}
    p = subprocess.run(
        ["git", "cat-file", "--batch"],
        cwd=REPO,
        input="\n".join(specs).encode(),
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
    )
    out = p.stdout
    result = {}
    pos = 0
    for spec in specs:
        nl = out.find(b"\n", pos)
        if nl == -1:
            break
        header = out[pos:nl]
        pos = nl + 1
        parts = header.split()
        if len(parts) == 2 and parts[1] == b"missing":
            result[spec] = None
            continue
        size = int(parts[2])
        result[spec] = out[pos : pos + size].decode("utf-8", "replace")
        pos += size + 1  # skip trailing newline
    return result


def block_body(text, header_re):
    """Yield (name, body) for each brace-matched block whose header matches header_re."""
    for m in header_re.finditer(text):
        i = text.find("{", m.end())
        if i == -1:
            continue
        depth = 0
        j = i
        while j < len(text):
            c = text[j]
            if c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        yield m.group(1), text[i + 1 : j]


ENUM_RE = re.compile(rf"\benum\s+({IDENT})")
STRUCT_RE = re.compile(rf"\bstruct\s+({IDENT})")

VARIANT_RE = re.compile(rf"^\s*({IDENT})\s*[,=({{]")
FIELD_RE = re.compile(rf"^\s*(?:pub(?:\([^)]*\))?\s+)?({IDENT})\s*:")


def items(text):
    """-> set of 'Enum::Variant' and 'Struct::field' strings."""
    found = set()
    for name, body in block_body(text, ENUM_RE):
        for line in body.splitlines():
            line = line.strip()
            if not line or line.startswith("//") or line.startswith("#") or line.startswith("///"):
                continue
            m = VARIANT_RE.match(" " + line)
            if m and m.group(1)[0].isupper():
                found.add(f"{name}::{m.group(1)}")
    for name, body in block_body(text, STRUCT_RE):
        for line in body.splitlines():
            line = line.strip()
            if not line or line.startswith("//") or line.startswith("#") or line.startswith("///"):
                continue
            m = FIELD_RE.match(" " + line)
            if m:
                found.add(f"{name}.{m.group(1)}")
    return found


def main():
    limit = None
    if "--limit" in sys.argv:
        limit = int(sys.argv[sys.argv.index("--limit") + 1])

    files = subprocess.run(
        ["git", "diff", "--name-only", OURS, "--", "*.rs"],
        cwd=REPO,
        stdout=subprocess.PIPE,
        text=True,
    ).stdout.split()
    if limit:
        files = files[:limit]
    print(f"changed .rs files: {len(files)}")

    specs = []
    for f in files:
        specs.append(f"{OURS}:{f}")
        specs.append(f"{THEIRS}:{f}")
        specs.append(f"{BASE}:{f}")
    blobs = git_batch(specs)
    print(f"blobs fetched: {sum(1 for v in blobs.values() if v is not None)}")

    local_loss, remote_loss, skipped = [], [], 0

    for f in files:
        merged_path = REPO / f
        if not merged_path.exists():
            continue
        try:
            merged = merged_path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            skipped += 1
            continue
        o = blobs.get(f"{OURS}:{f}")
        t = blobs.get(f"{THEIRS}:{f}")
        b = blobs.get(f"{BASE}:{f}")
        if o is None or t is None:
            skipped += 1
            continue
        base_items = items(b) if b is not None else set()

        mi = items(merged)
        # only items a side ADDED relative to the base can be that side's loss
        for label, side, bucket in ((f"ours", o, local_loss), (f"theirs", t, remote_loss)):
            added = items(side) - base_items
            missing = added - mi
            if missing:
                bucket.append((f, label, sorted(missing)))

    print()
    print("=" * 78)
    print(f"LOCAL losses (in ours, MISSING from merged): {sum(len(m) for _,_,m in local_loss)} items in {len(local_loss)} files")
    print("=" * 78)
    for f, label, missing in local_loss[:60]:
        print(f"  {f}")
        for x in missing[:14]:
            print(f"      - {x}")
        if len(missing) > 14:
            print(f"      ... +{len(missing) - 14} more")

    print()
    print("=" * 78)
    print(f"REMOTE losses (in theirs, MISSING from merged): {sum(len(m) for _,_,m in remote_loss)} items in {len(remote_loss)} files")
    print("=" * 78)
    for f, label, missing in remote_loss[:40]:
        print(f"  {f}")
        for x in missing[:10]:
            print(f"      - {x}")
        if len(missing) > 10:
            print(f"      ... +{len(missing) - 10} more")

    print(f"\nskipped (unreadable/absent): {skipped}")


if __name__ == "__main__":
    main()
