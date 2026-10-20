# sync-2026-10-20 — tooling for the `e70e0250a0` upstream sync

Tooling and the resolution record for merging `upstream/main`
(`e70e0250a0ac997f52c3ba3c9b0bf55f60ee09c4`, 2026-10-20) into `forge`.

Result: commit **`58e0c651c54`**, tree **`9a7d30e0aed`**, **14 conflicted paths**
(12 content + 2 `docs/**` modify/delete), all resolved.

> The earlier sessions' tools (`parsegate.py`, `noloss3.py`, `braces.py`,
> `dangle.py`, `drift.py`, `refile.py`, `check_cargo.py`, `check_ws_inherit.py`,
> `merge-engine2.sh`) lived in `G:/tmp` and were **lost** — `G:` is volatile on
> this host. They are rebuilt here, and the traps they cost are recorded in
> `../README.md` and in the `CHANGELOG.md` session entry.

## Run order

```powershell
$env:DX_CODE_REPO = 'G:\Dx\code'          # the repo (default)
$env:DX_MERGE_ROOT = 'G:\Temp\...\dxmerge' # scratch root (default)
$env:DX_MERGE_CTX  = "$env:DX_MERGE_ROOT\ctx"

# 1. measure the merge in memory - no worktree writes
git -C $env:DX_CODE_REPO merge-tree --write-tree --name-only upstream/main forge

# 2. explode the conflict context (base/ours/theirs/merged per path)
#    -> $env:DX_MERGE_CTX/<flat>.{base,ours,theirs,merged}
#    NOTE: write the blobs with `cmd /c "git cat-file blob <spec> > file"`.
#    PowerShell's native-command redirection converts LF to CRLF and the
#    extracted files no longer match the blobs (see the traps section).

# 3. read a conflict with context
python show_conflicts.py <flat-name> <block-index> [before] [after]
python analyze_conflicts.py <flat-name>...            # block sizes only

# 4. resolve it from a spec, byte-exact
python resolve_conflicts.py <flat-name>...             # writes <flat>.resolved

# 5. install the resolved blobs into the scratch index
$env:GIT_INDEX_FILE = "$env:DX_MERGE_ROOT\idx"
git -C $env:DX_CODE_REPO read-tree <merge-tree>
#    per file:
$sha = git -C $env:DX_CODE_REPO hash-object -w <flat>.resolved
git -C $env:DX_CODE_REPO update-index --add --cacheinfo "644,$sha,<repo-path>"

# 6. gates
python gates.py <tree> <ours> <theirs> <base>
python parsegate.py <tree> <ours> <theirs> <base>

# 7. commit
$tree = git -C $env:DX_CODE_REPO write-tree
git -C $env:DX_CODE_REPO commit-tree $tree -p <ours> -p upstream/main
```

## Tools

| file | what it does |
|---|---|
| `gates.py` | tree-wide gate suite: conflict markers, deletion audit, token-level no-loss audit **in both directions**, item-parity sweep, brace-balance neutrality |
| `parsegate.py` | rustfmt parse gate over every changed `.rs` file, classifying each failure as **pre-existing** (a side already failed) or **merge-introduced** |
| `resolve_conflicts.py` | spec-driven, byte-exact conflict resolver; refuses to write if a marker survives or the spec's block count does not match |
| `show_conflicts.py` | one conflict block with shared context before and after |
| `analyze_conflicts.py` | per-file block inventory (block index, side sizes) |
| `specs/*.json` | the resolution record: one file per conflicted path |

## Spec format

`specs/<flat-name>.json`:

```json
{
  "file": "crates/gpui/src/scene.rs",
  "blocks": {
    "1": ["            .sort_by_key(...);",
          "        self.liquid_glass",
          "            .sort_by_key(...);"]
  },
  "edits": [
    {"label": "what this does", "expect": 1,
     "old": ["..."], "new": ["..."]}
  ]
}
```

* `blocks` — the replacement lines for each **1-based** conflict block, in
  order. An **empty list** deletes the block.
* `@SIDE1` / `@SIDE2` — expand to that side's lines **verbatim**. Used for
  `Cargo.lock`, where a 110-line side must not be retyped by hand.
* `edits` — anchored replacements **outside** the conflict blocks, applied
  after the block resolutions. This is how an upstream semantic change is
  re-attached onto the fork's structure when the two sides' code does not
  textually overlap. `expect` is the exact match count; the tool refuses to
  write if it does not hold.

## Traps this sync paid for

| trap | symptom | rule |
|---|---|---|
| **PowerShell native-command redirection mangles blobs** | `git show … > file` adds one byte per line (LF→CRLF); the extracted file no longer matches the blob and byte-counts differ | extract with `cmd /c "git cat-file blob <spec> > file"`, or use `git cat-file --batch` |
| **merge-ort emits 2-way markers** | a parser that expects `\|\|\|\|\|\|\|` base sections consumes the whole conflict as "ours" | handle both `<<<<<<< / ======= / >>>>>>>` and the 3-way form |
| **`cargo metadata --locked` is the lock's only oracle** | a hand-merged lock that does not match the merged manifests fails with "cannot update the lock file" | resolve the lock with cargo; the hand-merge differed from cargo's answer by exactly one version-qualified dependency and one package ordering |
| **`git diff --no-index` with a relative path** | the diff compares the **repo's** file, not the intended one, because `-C` changes the process cwd | always pass absolute paths to `--no-index` |
| **rustfmt with a filename resolves `mod` declarations** | "failed to resolve mod `x`" on files that are not on disk — a harness artefact that looks like 45 parse failures | feed rustfmt the file on **stdin**: `rustfmt --edition 2024 --emit stdout --config-path rustfmt.toml < file.rs` |
| **a comment/string stripper must handle char literals and lifetimes** | `'a` (lifetime) vs `'x'` (char) confusion swallows the rest of the file; the audit then reports tokens that are present | strip strings and char literals, skip lifetimes, handle nested block comments and raw strings |
| **the `.hbs` marker false positive** | `git grep '^(<<<<<<< …)'` reports `edit_file_prompt_diff_fenced.hbs` | that file is a SEARCH/REPLACE **prompt template**; exclude `*.hbs` from the marker gate |
| **a rename reads as a lost definition** | `ScrollWheelEvent` (ours) vs `ScrollEvent` (theirs) | the item-parity sweep must separate *both sides define it* (a real loss) from *one side defines it* (a delete or a rename) |

## Verification for this sync

| check | result |
|---|---|
| conflict markers in the merged tree | **0** |
| parse gate over the 166 changed `.rs` files | **0 parse failures**; 14 not format-clean, **all pre-existing fork drift**, none merge-introduced |
| deletion audit | 11 added / 2 deleted; **0 unexplained** |
| no-loss audit (both directions) | 4 items, all **documented supersessions or renames** |
| brace-balance neutrality | **0** files differ from both sides |
| `cargo metadata --locked` | **rc 0** |
| `git ls-files --eol` | **0** `i/crlf`, **0** `i/mixed` |
| compile | **not run** — a full build needs more disk than `G:` has free |
