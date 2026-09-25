# merge-tools

Tooling and evidence from the `dx-code` × upstream Zed merge (2026-09-24 → 2026-09-26).

Preserved here because it lived in `G:/tmp/sync/`, and `G:` is volatile on this host — losing it would
lose the reproducibility of every repair below.

Full narrative: `../CHANGELOG.md` (§Session 33–35), `../TODO.md` §0.11, `../PLAN.md`.

> ⚠️ The merge docs reference these files by their original `G:/tmp/sync/…` paths. The copies here are
> byte-identical; where a doc names a `/g/tmp` path, this directory is the durable home.

## Tools (generic, reusable)

| file | what it does |
|---|---|
| `eolpatch.py` | **Assertion-guarded, EOL-aware multi-replacement patcher.** Detects the file's own line ending, builds every anchor with it, asserts each anchor's match count, and **refuses to write** if any anchor is off. Necessary because every worktree file here is **CRLF** while the index is LF (`git ls-files --eol` → `i/lf w/crlf`) and `grep -c $'\r'` **lies** (returns 0 on a CRLF file), so LF-built anchors match nothing. Usage: `python eolpatch.py spec.json`. |
| `nested_audit.py` | **Nested-item silent-loss audit.** Compares enum variants (`Enum::Variant`) and struct fields (`Struct.field`) across ours / theirs / base / merged for every changed `.rs` file, using `git cat-file --batch`. Reports items a side **added relative to the base** that the merged tree lacks. ⚠️ The base filter is mandatory: without it the report is ~98 % noise (952 items → 16). ⚠️ A struct **rename** makes every field of that struct look lost. |
| `resolve.py` | Session-32 sync resolver (the 7-conflict upstream sync). |

## Repairs (the three fix scripts)

| file | repair |
|---|---|
| `fix_entity_map.py` | `gpui/src/app/entity_map.rs` — a **duplicated** `let Some(entity) = …` block whose first copy had lost its `else { … };`, giving `expected ';', found keyword 'let'` plus an `E0282` cascade. |
| `fix_icons_enum.py` | `icons/src/icons.rs` — restores **9 of ours' `IconName` variants** the merge dropped, each at its alphabetical position. Asserts the enum is sorted (whitelisting the two pre-existing upstream anomalies `Bookmark>BoltFilled`, `TextWrap>TextUnwrap`) and refuses to guess. |
| `fix_lm_dupes.py` | `language_model/src/language_model.rs` — a duplicated trait-default pair (`authentication_error_message` + `missing_credentials_error_message`) whose two copies **differed in wording**. Keeps **ours** ("via the Agent Panel settings"), removes theirs, and anchors on the **theirs-only** strings so it cannot delete ours by mistake. |

## `specs/` — the exact edit record

One JSON per anchored replacement, in `eolpatch.py` format (`path` + `replacements[]` with `old`/`new`
line arrays and an expected match count). These are the audit trail for the fixes that did not need a
dedicated script:

`spec_scene_label.json` (the `LiquidGlass` arm on upstream's new `PrimitiveBatch::label()`) ·
`spec_window_atlaskey.json` (`AtlasKey` passed by value) · `spec_window_profiler_import.json` +
`spec_reflow.json` (the doubled `profiler` import and the rustfmt reflow it required) ·
`spec_lang_avail.json` / `spec_lang_registry.json` / `spec_lang_settings.json` (the four ports onto
upstream's new `SplicingVec` / `FileTypeMap` / `ExtendingSet` / `AvailableLanguages` types).

## `evidence/`

| file | what it proves |
|---|---|
| `parsegate.txt` | the full-tree parse gate result — **0 of 1256** changed `.rs` files fail to parse |
| `nested_audit2.txt` | the base-filtered silent-loss audit — 16 local losses in 2 files, all accounted for as renames |
| `enum_ours.txt` / `enum_final.txt` | ours' 304 `IconName` variants and the post-repair set — `comm -23` is **empty**, i.e. zero remaining losses |
| `upstream_docs.diff` | upstream's two `docs/` sections recorded verbatim, so honouring our deletion of that tree loses nothing silently |
| `Cargo.toml.bak` | the pre-fix root manifest (md5 `ca3e6b63c8c57830c0b8b36f3584da09`), so the `dx_media` pin is reversible |

## The merge policy these tools serve

A **genuine semantic merge**, never a "local / remote / both" pick: **local 100 % and remote 100 %**.
Where upstream restructured code our UI rewrote, keep **our** structure and port upstream's semantics onto
it. Where our UI has no surface for an upstream feature, keep upstream's code **compiled and wired, just
unsurfaced** — never deleted. **Never guess a conflict**, and **marker-free ≠ resolved**: both no-loss
audits must pass in both directions.
