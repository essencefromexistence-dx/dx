import sys

p = r"G:/Dx/code/crates/editor/src/inlays/inlay_hints.rs"
raw = open(p, encoding="utf-8", newline="").read()
lines = raw.split("\n")

CR = "\r" if raw.count("\r\n") > 0 else ""
print("EOL:", "CRLF" if CR else "LF")

def body(s):
    return s.rstrip("\r")

blocks = []
i = 0
while i < len(lines):
    if body(lines[i]).startswith("<<<<<<< HEAD"):
        s = i
        m = e = None
        j = i + 1
        while j < len(lines):
            b = body(lines[j])
            if b == "=======" and m is None:
                m = j
            elif b.startswith(">>>>>>> "):
                e = j
                break
            j += 1
        if m is None or e is None:
            sys.exit("unterminated block at line %d (m=%s e=%s)" % (s + 1, m, e))
        blocks.append((s, m, e))
        i = e + 1
    else:
        i += 1

print("blocks (1-based start/sep/end):", [(s + 1, m + 1, e + 1) for s, m, e in blocks])

def ours(b):
    return lines[b[0] + 1:b[1]]

def theirs(b):
    return lines[b[1] + 1:b[2]]

rep3 = [x + CR for x in [
    "        let invalidating = invalidate_cache.should_invalidate();",
    "        if invalidating {",
    "            let Some(mut visible_inlay_hint_ids) = collect_visible_inlay_hint_ids_for_buffer(",
    "                self.display_map.read(cx),",
    "                &multi_buffer_snapshot,",
    "                buffer_id,",
    "            ) else {",
    "                return;",
    "            };",
    "            for hint_id in &visible_inlay_hint_ids {",
    "                inlay_hints.added_hints.remove(hint_id);",
]]

out = []
prev = 0
for idx, b in enumerate(blocks, start=1):
    o, t = ours(b), theirs(b)
    print("  block %d: ours=%d theirs=%d | ours tail=%r | theirs head=%r"
          % (idx, len(o), len(t), body(o[-1]) if o else None, body(t[0]) if t else None))
    out.extend(lines[prev:b[0]])
    if idx in (1, 2, 4):
        rep = t
    elif idx == 3:
        rep = rep3
    elif idx == 5:
        rep = o + ["}" + CR] + ["" + CR] + t
    else:
        sys.exit("unexpected block count: %d" % idx)
    out.extend(rep)
    prev = b[2] + 1
out.extend(lines[prev:])

open(p, "w", encoding="utf-8", newline="").write("\n".join(out))
left = sum(1 for l in out if body(l).startswith("<<<<<<< ") or body(l).startswith(">>>>>>> "))
print("written; lines=%d markers_left=%d" % (len(out), left))
