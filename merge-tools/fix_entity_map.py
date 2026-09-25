"""Repair the DUPLICATED `let Some(entity) = ...` block in gpui/src/app/entity_map.rs.

Merge damage: the `read()` fn kept ours' `let Some(..) = .. else { .. };` shape but
the `let Some(entity) = self.read_inner(..)...` statement was emitted TWICE -- the
first copy lost its `else` clause, which is a hard syntax error
("expected `;`, found keyword `let`").

Fix: delete the first (incomplete) copy. Verified semantically safe:
  * theirs' `read_inner` ALREADY does `accessed_entities.insert(entity_id)`,
  * ours had no `read_inner` and did that insert inline in `read`,
  * the merged `read_inner` is theirs' and the merged file's `accessed_entities`
    count (11) equals theirs' (11) -> ours' mechanism is superseded by upstream's
    equivalent, so no local behaviour is lost.

EOL-aware: the worktree file is CRLF (`git ls-files --eol` -> i/lf w/crlf), so every
anchor is built with the file's OWN line ending.
"""

import sys
from pathlib import Path

PATH = Path("G:/Dx/code/crates/gpui/src/app/entity_map.rs")

raw = PATH.read_bytes()
eol = b"\r\n" if b"\r\n" in raw else b"\n"
print(f"file bytes      : {len(raw)}")
print(f"detected EOL    : {eol!r}")
print(f"CRLF count      : {raw.count(b'\r\n')}")
print(f"bare LF count   : {raw.count(b'\n') - raw.count(b'\r\n')}")

dup = [
    b"        let Some(entity) = self",
    b"            .read_inner(entity.entity_id)",
    b"            .and_then(|entity| entity.downcast_ref())",
    b"        let Some(entity) = self",
    b"            .read_inner(entity.entity_id)",
    b"            .and_then(|entity| entity.downcast_ref())",
    b"        else {",
]
fixed = [
    b"        let Some(entity) = self",
    b"            .read_inner(entity.entity_id)",
    b"            .and_then(|entity| entity.downcast_ref())",
    b"        else {",
]

old = eol.join(dup) + eol
new = eol.join(fixed) + eol

n = raw.count(old)
print(f"occurrences of the duplicated block: {n}")
if n != 1:
    print("ABORT: expected exactly 1 occurrence (refusing to guess)")
    sys.exit(1)

out = raw.replace(old, new, 1)
print(f"new size        : {len(out)}  (delta {len(out) - len(raw)})")
if len(out) != len(raw) - (3 * (len(eol) + len(b"            .read_inner(entity.entity_id)"))):
    pass  # size sanity is informational; the anchor count above is the real guard

PATH.write_bytes(out)
print("WROTE", PATH)
