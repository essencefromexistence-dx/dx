"""EOL-aware, assertion-guarded multi-replacement patcher.

Why this exists: every worktree file in G:/Dx/code is CRLF while the index is LF
(`git ls-files --eol` -> `i/lf w/crlf`), and `grep -c $'\\r'` LIES about it. A plain
text replacement built with LF newlines therefore matches NOTHING. This tool detects
the file's own EOL, builds every anchor with it, and refuses to write unless each
anchor matches its expected occurrence count exactly.

Usage:  python eolpatch.py <spec.json>

spec.json:
{
  "path": "G:/Dx/code/...",
  "replacements": [
    {"label": "what this does", "expect": 1, "old": ["line", "..."], "new": ["line", "..."]}
  ]
}
"""

import json
import sys
from pathlib import Path


def main(spec_path: str) -> int:
    spec = json.loads(Path(spec_path).read_text(encoding="utf-8"))
    path = Path(spec["path"])
    raw = path.read_bytes()

    eol_bytes = b"\r\n" if b"\r\n" in raw else b"\n"
    eol = eol_bytes.decode()
    crlf = raw.count(b"\r\n")
    bare_lf = raw.count(b"\n") - crlf
    print(f"file            : {path}")
    print(f"size            : {len(raw)}")
    print(f"EOL             : {eol!r}  (CRLF={crlf}, bare LF={bare_lf})")

    out = raw
    total_delta = 0
    failures = []

    for i, rep in enumerate(spec["replacements"], 1):
        label = rep.get("label", f"replacement {i}")
        expect = rep.get("expect", 1)
        old = eol.join(rep["old"]) + eol
        new = eol.join(rep["new"]) + eol
        old_b = old.encode()
        new_b = new.encode()

        n = out.count(old_b)
        status = "OK" if n == expect else "FAIL"
        print(f"  [{i}] {status} matches={n} expect={expect}  {label}")
        if n != expect:
            failures.append(f"[{i}] {label}: found {n}, expected {expect}")
            continue

        out = out.replace(old_b, new_b, expect)
        total_delta += len(new_b) - len(old_b)

    if failures:
        print("\nABORT — nothing written. Unmet anchors:")
        for f in failures:
            print("   " + f)
        return 1

    path.write_bytes(out)
    print(f"\nWROTE {path}")
    print(f"size delta      : {total_delta}  ({len(raw)} -> {len(out)})")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
