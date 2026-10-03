import sys

p = r"G:/Dx/code/crates/extensions_ui/src/extensions_ui.rs"
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
            sys.exit("unterminated block at %d" % (s + 1))
        blocks.append((s, m, e))
        i = e + 1
    else:
        i += 1

print("blocks:", [(s + 1, m + 1, e + 1) for s, m, e in blocks])

def ours(b):
    return lines[b[0] + 1:b[1]]

def theirs(b):
    return lines[b[1] + 1:b[2]]

union1 = [x + CR for x in [
    "use editor::{Editor, EditorElement, EditorStyle, MultiBufferOffset};",
    "use extension_host::{ExtensionIndexEntry, ExtensionManifest, ExtensionStore};",
]]

out = []
prev = 0
for idx, b in enumerate(blocks, start=1):
    o, t = ours(b), theirs(b)
    print("  block %d: ours=%d theirs=%d | ours head=%r"
          % (idx, len(o), len(t), body(o[0]) if o else None))
    out.extend(lines[prev:b[0]])
    if idx == 1:
        rep = union1
    elif idx == 2:
        rep = list(t)
        hits = 0
        for k, ln in enumerate(rep):
            if body(ln) == "                            match_candidates.len(),":
                rep[k] = "                            MAX_FILTERED_EXTENSION_RESULTS.min(match_candidates.len())," + CR
                hits += 1
        if hits != 1:
            sys.exit("match_candidates.len() limit: expected 1 hit, got %d" % hits)
        print("    -> re-attached MAX_FILTERED_EXTENSION_RESULTS cap in update_local_search_results")
    elif idx in (3, 4, 5, 6):
        rep = t
    else:
        sys.exit("unexpected block %d" % idx)
    out.extend(rep)
    prev = b[2] + 1
out.extend(lines[prev:])

# keep ours' now-unsurfaced helper + const compiled
def annotate(text, needle, attr):
    n = 0
    for k, ln in enumerate(text):
        if body(ln) == needle:
            if body(text[k - 1]) != attr:
                text.insert(k, attr + CR)
                n += 1
    return n

a1 = annotate(out, "fn bounded_remote_extension_entries(", "#[allow(dead_code)]")
a2 = annotate(out, "const MAX_DISPLAYED_EXTENSION_RESULTS: usize = MAX_FILTERED_EXTENSION_RESULTS;", "#[allow(dead_code)]")
print("annotated: bounded_remote_extension_entries=%d MAX_DISPLAYED_EXTENSION_RESULTS=%d" % (a1, a2))
if a1 != 1 or a2 != 1:
    sys.exit("annotation failed")

open(p, "w", encoding="utf-8", newline="").write("\n".join(out))
left = sum(1 for l in out if body(l).startswith("<<<<<<< ") or body(l).startswith(">>>>>>> "))
print("written; lines=%d markers_left=%d" % (len(out), left))
