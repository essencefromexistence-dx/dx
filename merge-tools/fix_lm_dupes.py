"""Remove the duplicated auth-error-message pair from language_model.rs, keeping OURS.

The merge emitted BOTH sides of a trait-default block:

  Block A (theirs) lines 427-451 : "in Settings > AI > LLM Providers"
  Block B (ours)   lines 453-477 : "via the Agent Panel settings"

Both define `authentication_error_message` and `missing_credentials_error_message`
-> E0428 "defined multiple times".

Policy: local 100 %. Ours' wording is the local change (it names our UI surface,
the Agent Panel), so Block B stays and Block A is removed.

Anchors on the THEIRS wording, so it cannot remove ours by mistake.
"""

import sys
from pathlib import Path

PATH = Path("G:/Dx/code/crates/language_model/src/language_model.rs")

THEIRS_AUTH = "Update your key in Settings > AI > LLM Providers to continue."
THEIRS_CRED = "Add your key in Settings > AI > LLM Providers to continue."
OURS_AUTH = "Update your key via the Agent Panel settings to continue."
OURS_CRED = "Add your key via the Agent Panel settings to continue."

raw = PATH.read_bytes()
eol = b"\r\n" if b"\r\n" in raw else b"\n"
lines = raw.decode("utf-8").split(eol.decode())

# --- locate the two blocks by their distinctive wording -------------------------
a_auth = [i for i, l in enumerate(lines) if THEIRS_AUTH in l]
a_cred = [i for i, l in enumerate(lines) if THEIRS_CRED in l]
b_auth = [i for i, l in enumerate(lines) if OURS_AUTH in l]
b_cred = [i for i, l in enumerate(lines) if OURS_CRED in l]

print(f"theirs-wording lines : auth={a_auth} cred={a_cred}")
print(f"ours-wording lines   : auth={b_auth} cred={b_cred}")

if not (len(a_auth) == len(a_cred) == len(b_auth) == len(b_cred) == 1):
    sys.exit("ABORT: expected exactly one occurrence of each wording marker")

# --- walk back from Block A's auth body to the start of its doc comment ---------
start = a_auth[0]
# first back to the `fn` header that owns this string body
while start > 0 and not lines[start].lstrip().startswith("fn "):
    start -= 1
# then over that fn's doc comment
while start > 0 and lines[start - 1].strip().startswith("///"):
    start -= 1
# include one preceding blank line so no double blank is left behind
if start > 0 and lines[start - 1].strip() == "":
    start -= 1

# --- walk forward from Block A's cred body to the closing brace of that fn ------
end = a_cred[0]
while not lines[end].rstrip().endswith("}"):
    end += 1
# include the trailing blank line
if end + 1 < len(lines) and lines[end + 1].strip() == "":
    end += 1

print(f"\ndeleting lines {start + 1}..{end + 1} (inclusive):")
for i in range(start, end + 1):
    print(f"  {i + 1:4d}| {lines[i]}")

removed = lines[start : end + 1]
for m in (THEIRS_AUTH, THEIRS_CRED):
    assert any(m in l for l in removed), f"refusing: {m!r} not inside the removed range"

kept = lines[:start] + lines[end + 1 :]
text = eol.decode().join(kept)

# --- verify exactly one definition of each fn remains --------------------------
for name in ("fn authentication_error_message", "fn missing_credentials_error_message"):
    n = text.count(name)
    print(f"\nafter: {name!r} occurrences = {n}")
    if n != 1:
        sys.exit(f"ABORT: expected 1, got {n}")

for m in (OURS_AUTH, OURS_CRED):
    assert m in text, f"ABORT: ours' wording {m!r} was lost"

PATH.write_bytes(eol.join(l.encode("utf-8") for l in kept))
print(f"\nWROTE {PATH}  (removed {len(removed)} lines)")
