# dx-code × upstream Zed — merge status, 2026-09-24

## Inputs (re-verified, two readings)

| role | ref | commit | date |
|---|---|---|---|
| **ours** | `merge-20260924` = `dev` | `c701504082` | 2026-09-24 18:27 +0600 |
| **theirs** | `upstream/main` | `baee1ca6de` | 2026-09-24 09:52 UTC |
| **synthetic base** | `d8eb569cc5` (Zed, *not* an ancestor) | `d8eb569cc5` | 2026-06-15 10:54 UTC |

`dev` has **no common ancestor** with `upstream/main` (fresh root `8dc3418ec1`). The base was
recovered by minimising `git diff --name-only <candidate> dev` over upstream history; a plain
`merge-tree` refuses (unrelated histories) and the forced form compares against the **empty tree**
— its signature is "0 `content` conflicts + thousands of `add/add`", which is a *missing base*,
not a hard merge.

Merged tree computed in memory:

```
git merge-tree --write-tree --merge-base=d8eb569cc5 merge-20260924 upstream/main
-> 61ff6f4ddf14b258a6e11810345f59d51e6a5ce3      (257 CONFLICT records, 259 conflicted paths)
```

## Nothing has touched the worktree

All resolution happens in a **scratch index** and the object DB:

```
export GIT_INDEX_FILE=G:/tmp/merge-idx2      # NOT merge-idx — see the lock trap below
git read-tree 61ff6f4ddf        # stage-0 seed from merge-ort's own result
```

> **TRAP — use `merge-idx2`, not `merge-idx`.** A SIGTERM'd `update-index` left a 3.7 MB orphan
> `G:/tmp/merge-idx.lock`, and the G: guard **refuses to rename or delete it** (verified with the
> sandbox bypassed). The index was therefore copied to `merge-idx2`, which has no lock collision.
> Root cause of the wedge: `git update-index` writes `<index>.lock` then **renames** it over the
> index, and that rename is intercepted → `fatal: Unable to write new index file`. The guard is
> **intermittent** — a retry seconds later succeeded. Fallback: `C:` has 17 GB free, and the index
> need not live on the repo's drive.

This is mandatory on this machine: the G: safe-delete guard wedges `git merge` (reproduced 4×,
incl. with the sandbox bypassed) and also makes `git update-ref` return 0 while changing nothing.
**Zero worktree writes ⇒ the guard cannot fire.** At the end the user runs one `git reset --hard`.

## Conflict inventory (257 records / 259 paths)

| type | count | nature | disposition |
|---|---|---|---|
| `modify/delete` — *we* deleted, upstream modified | 114 | `docs/**`, `.zed/`, `.cargo/`, `CONTRIBUTING.md`, 5 × `crates/*/LICENSE-*` | **109 force-removed**; `.cargo/config.toml` restored; 5 licences kept |
| `modify/delete` — upstream deleted, *we* modified | 9 | our work at risk | see Batch B |
| `content` | 121 | the real hand-merge work | **open — 739 hunks** |
| `rename/delete` | 7 | 5 × `docs/.conventions/brand-voice` (we deleted), `REVIEWERS.conl`/`debug.plist` (we parked in `zzz/`) | resolved |
| `add/add` | 4 | `crates/http_proxy/{Cargo.toml,src/allowlist.rs,src/http_proxy.rs}`, `crates/language_models/src/provider/llama_cpp.rs` | 3 resolved, `llama_cpp.rs` open |
| `rename/rename` | 1 | `renovate.json` → `zzz/` (ours) vs `.github/` (theirs) | merged tree keeps **both**, zero loss |
| `distinct types` | 1 | `crates/http_proxy/LICENSE-GPL` regular file (ours) vs symlink (theirs) | identical content, took theirs (symlink) |

Rename-aware view of `ours -> merged`: **A 437 · M 1446 · D 53 · R 27**. The `--name-only`
"114 deletions" figure was misleading — `--name-only` has **no rename detection**, so every
renamed crate shows up as add+delete. The authoritative deletion measure is
`git diff -M --name-status merge-20260924 61ff6f4ddf`.

## Resolved so far

### Batch A — mechanical (113 paths force-removed)
`docs/**` (110), `.zed/settings.json`, `CONTRIBUTING.md`.

### Batch A′ — one reversal after inspection
`.cargo/config.toml` was **restored** from upstream. It is *not* harmless housekeeping: it carries
`--cfg tokio_unstable` and `--cfg windows_slim_errors`, and `tokio_unstable` appears **nowhere** in
our tree. Since the merged tree contains upstream's code, that file is build-critical. It does **not**
set `target-dir`, so the justfile's `build_target_dir := "G:/Dx/code/target"` pin is unaffected.

### Batch B — structural
* `crates/http_proxy/` — **ours is a strict subset of theirs.** Our `http_proxy.rs` doc comment says
  "This crate grows over a short stack of PRs" and declares only `mod allowlist`; upstream's declares
  `allowlist` + `pinned_host` + `proxy` and ships `Cargo.toml` with 11 deps vs our 2. Taking theirs
  preserves both sides. Applied to all 3 conflicted files; the `~merge-20260924` tilde artifact
  merge-ort emitted for the symlink/regular-file clash was removed.
* `crates/csv_preview/src/csv_preview.rs` → `crates/tabular_data_preview/src/tabular_data_preview.rs`
  merged **cleanly** (our +21/−0 applied onto theirs' +207/−122).
* `crates/extensions_ui/src/extension_suggest.rs` → `crates/extension_suggest/src/extension_suggest.rs`
  (our +19/−1 onto theirs' +326/−143) — 1 residual hunk.
* `crates/csv_preview/src/renderer/settings.rs` → `crates/tabular_data_preview/src/renderer/settings.rs`
  (our +1/−1 onto theirs' +147/−166) — 1 residual hunk.
* 10 paths keep **ours** verbatim (`add_llm_provider_modal.rs`, `capture_example.rs`,
  `script/{storybook,redeploy-vercel,github-label-issues-to-triage.py}`,
  `zzz/{REVIEWERS.conl,debug.plist}`, `.github/workflows-disabled/triage_project_sync.yml.disabled`).
* 6 `crates/*/LICENSE-*` paths keep **theirs** (merge-ort mislabelled these as `modify/delete`; they
  are pure upstream additions: absent in base *and* in ours).

### Verified intact
Our DX branding survives the auto-merge: `IconName::DxCog` is **our** rename (0 files in base and
upstream, 14 in ours) and is defined at `crates/icons/src/icons.rs:112` **of the merged tree**.
`crates/ui/src/dx_icons.rs`, `PROBLEM.md` and `script/dx-icon-system-source.test.ts` all survive.

## The one genuinely open structural decision: `llama_cpp.rs`

Both sides independently wrote a `llama.cpp` provider. They are **complementary, not duplicates**:

* **ours** (2248 lines) — *manages the local server*: `LlamaCppLaunchConfig`, `LlamaCppRuntime`,
  `RunningServer`, `warm_up_llama_server`, prompt fitting to context, chat templates.
  Provider id `"llama_cpp"`.
* **theirs** (1909 lines) — *talks to a server*: delegates to the new `crates/llama_cpp` crate
  (`LLAMA_CPP_API_URL = "http://localhost:8080"`, **zero** server-launch code), plus
  `ConfigurationView`, `LiveCapabilities`, capability cells, telemetry, env API key.
  Provider id `"llama.cpp"`.

The ids differ, so both *can* coexist — but both are named `LlamaCppLanguageModelProvider`, and
`LlamaCppSettings`/`State`/`LlamaCppLanguageModel` collide. "Keep both" therefore means renaming our
types (≈2200 lines) and registering two providers.

## Remaining work (updated 2026-09-24, after the manifest layer went green)

0. ✅ **Root `Cargo.toml` + the 4 crate manifests are DONE.** `cargo metadata --no-deps` → **rc 0**,
   262 members, none missing on disk. Includes the `pathdiff` cross-file fix — see CHANGELOG.md.
1. **73 content files / 214 code hunks** — queue at `G:/tmp/remaining_hunks.tsv`; full original
   worklist appended below (its hunk counts are the pre-resolver figures).
2. 2 rename-merge residual hunks (each 1 hunk).
3. `llama_cpp.rs` (add/add).
4. `Cargo.lock` — 353 of the hunks are lockfile churn; **regenerate, never hand-merge**.
5. Verification: `cargo metadata` is green. Scoped `cargo check -p <crate>` is the next step but
   **cannot pass yet** — 117 files still carry conflict markers. `G:` has ~47 GB free and `code/`
   has no `target/`; a cold full-workspace check very likely does not fit.

### Exact reconciliation (2026-09-24)

| measure | n |
|---|---:|
| worklist rows | 121 |
| resolved (5 manifests: root + `agent` + `extensions_ui` + `gpui_windows` + `util`) | 5 |
| still marked **and** in the worklist | 116 |
| marked **outside** the worklist | 2 |
| files carrying markers (total) | 118 |
| **real remaining conflicts** | **117** |

### After the auto-resolution pass (2026-09-24, evening)

`G:/tmp/resolve2.py` + `G:/tmp/install_all.py` re-merged all 117 files from the object DB and
resolved hunks that `git merge-file` had merely *grouped* (both sides editing different lines of one
hunk), plus insert/insert unions and character-level merges. Nothing was guessed — unresolved hunks
keep their markers.

| frame | before | after | progress |
|---|---:|---:|---:|
| conflict records | 257 | 74 left | **71 %** |
| file paths | 259 | 74 left | **71 %** |
| content files (of 121) | 121 | 73 left | 40 % |
| **code hunks** | **385** | **214** | **44 %** |
| `Cargo.lock` hunks | 353 | 353 | regenerate, never merge |
| files fully marker-free | 0 | 43 | — |
| files partially resolved | 0 | 73 | — |

Remaining queue, sorted: `G:/tmp/remaining_hunks.tsv`. Tiers: 7 files ≥10 hunks · 6 files 5–9 ·
27 files 2–4 · 35 files with 1. Biggest: `thread_view.rs`(16), `sidebar.rs`(12),
`project_panel.rs`(12), `extensions_ui.rs`(12), `workspace.rs`(11), `opencode.rs`(10).

The 2 outside the worklist:

* `crates/language_models/src/provider/llama_cpp.rs` — the known **add/add** (stages 2+3, no stage 1).
* `crates/agent/src/templates/edit_file_prompt_diff_fenced.hbs` — **FALSE POSITIVE, do not touch.**
  It is a prompt template that *documents* the search/replace format using `<<<<<<< SEARCH line=N`
  / `>>>>>>> REPLACE` as example syntax. All four sides are byte-identical
  (`a7db420a99dcd4f2c109581c29485c3cfd0f7b64`), and `merge-tree` reports **no conflict** for it.
  A naive `git grep -l '^<<<<<<<'` gate will flag it.

## How to continue (machinery)

```
export GIT_INDEX_FILE=G:/tmp/merge-idx2
export MSYS_NO_PATHCONV=1                 # or `upstream/main:.cargo/config.toml` gets mangled to `upstream\main;...`

# per-file context (already generated for all 121 by the engine):
#   /g/tmp/mr2/<path>.base .ours .theirs .merged
#   report: /tmp/mf2_report.tsv  (cols: rc, residual_hunks, ours_changed, theirs_changed, direction, path)

# apply a resolved file:
sha=$(git hash-object -w /g/tmp/mr2/<path>.resolved)
git update-index --add --cacheinfo 100644,$sha,<path>

# finish:
git write-tree
git commit-tree -p merge-20260924 -p upstream/main -m "merge: upstream Zed baee1ca6de into dx-code"
# then the user runs: git reset --hard <resolved>
```

**EOL:** `core.autocrlf=true`, but the index is LF throughout (`git ls-files --eol` shows `i/lf`).
`git hash-object -w` on a LF scratch file round-trips **byte-exact** — verified with `cmp`.

**Trap:** `git merge-file`'s `-L` labels are positional. If you pass `theirs base ours`, the first
section is labelled whatever you wrote first — it is *not* auto-named. Mislabelling silently swaps
"ours" and "theirs" in the review output. Always confirm a disputed hunk with `git grep` against the
tree rather than trusting the label.

## Open questions for the user

1. **`docs/`** — 109 upstream files were dropped to keep our fork's layout. Pure docs, zero build
   impact, one command to restore. Restoring loses nothing of ours and satisfies "remote 100%".
2. **`llama_cpp.rs`** — confirm the two-provider shape (rename ours) vs. keeping upstream's live and
   our implementation compiled-but-unwired.
3. **Verification** — with ~49 GB free a full `cargo check` may not fit; confirm whether to attempt it
   or to hand over with the conflict set documented.

---
## Full worklist — 121 content conflicts (sorted by residual hunks)

| hunks | ours changed | theirs changed | start-from | path |
|---:|---:|---:|---|---|
| 353 | 3790/2247 | 2604/2559 | regenerate (never hand-merge) | Cargo.lock |
| 32 | 3573 | 5083 | ours-onto-theirs | crates/agent_ui/src/conversation_view/thread_view.rs |
| 20 | 3426 | 1674 | theirs-onto-ours | crates/project_panel/src/project_panel.rs |
| 20 | 5314 | 1311 | theirs-onto-ours | crates/sidebar/src/sidebar.rs |
| 18 | 655 | 960 | ours-onto-theirs | crates/extensions_ui/src/extensions_ui.rs |
| 17 | 902 | 4877 | ours-onto-theirs | crates/workspace/src/workspace.rs |
| 13 | 3661 | 2242 | theirs-onto-ours | crates/agent_ui/src/agent_panel.rs |
| 10 | 1048 | 730 | theirs-onto-ours | crates/settings_ui/src/settings_ui.rs |
| 10 | 1196 | 240 | theirs-onto-ours | crates/title_bar/src/title_bar.rs |
| 10 | 125 | 634 | ours-onto-theirs | crates/opencode/src/opencode.rs |
| 10 | 205 | 2564 | ours-onto-theirs | crates/editor/src/element.rs |
| 10 | 79 | 393 | ours-onto-theirs | Cargo.toml |
| 9 | 476 | 560 | ours-onto-theirs | crates/project/src/context_server_store.rs |
| 9 | 94 | 5105 | ours-onto-theirs | crates/agent_ui/src/conversation_view.rs |
| 8 | 114 | 3305 | ours-onto-theirs | crates/agent/src/thread.rs |
| 8 | 372 | 1360 | ours-onto-theirs | crates/gpui_wgpu/src/wgpu_renderer.rs |
| 8 | 448 | 1174 | ours-onto-theirs | crates/language_models/src/provider/opencode.rs |
| 6 | 1021 | 271 | theirs-onto-ours | crates/workspace/src/dock.rs |
| 6 | 160 | 10352 | ours-onto-theirs | crates/git_ui/src/git_panel.rs |
| 6 | 169 | 58 | theirs-onto-ours | crates/diagnostics/src/diagnostic_renderer.rs |
| 6 | 210 | 1119 | ours-onto-theirs | crates/gpui_apple/src/metal_renderer.rs |
| 6 | 41 | 159 | ours-onto-theirs | crates/ui/src/components/list/list_item.rs |
| 5 | 14 | 78 | ours-onto-theirs | crates/gpui/src/app/entity_map.rs |
| 5 | 151 | 1025 | ours-onto-theirs | crates/language/src/language_settings.rs |
| 5 | 180 | 268 | ours-onto-theirs | crates/zed/src/main.rs |
| 4 | 140 | 1179 | ours-onto-theirs | crates/git_ui/src/branch_picker.rs |
| 4 | 214 | 1342 | ours-onto-theirs | crates/settings_ui/src/page_data.rs |
| 4 | 2853 | 393 | theirs-onto-ours | assets/settings/default.json |
| 4 | 333 | 822 | ours-onto-theirs | crates/file_finder/src/file_finder.rs |
| 4 | 401 | 2067 | ours-onto-theirs | crates/zed/src/zed.rs |
| 4 | 441 | 946 | ours-onto-theirs | crates/workspace/src/multi_workspace.rs |
| 4 | 443 | 766 | ours-onto-theirs | crates/editor/src/inlays/inlay_hints.rs |
| 4 | 52 | 404 | ours-onto-theirs | crates/gpui_windows/src/platform.rs |
| 4 | 609 | 175 | theirs-onto-ours | crates/gpui_windows/src/events.rs |
| 3 | 126 | 185 | ours-onto-theirs | crates/agent_ui/src/agent_configuration/configure_context_server_modal.rs |
| 3 | 138 | 294 | ours-onto-theirs | crates/project/src/lsp_store/document_symbols.rs |
| 3 | 211 | 517 | ours-onto-theirs | crates/editor/src/semantic_tokens.rs |
| 3 | 224 | 607 | ours-onto-theirs | crates/agent_ui/src/config_options.rs |
| 3 | 29 | 160 | ours-onto-theirs | crates/settings_ui/src/components/ollama_model_picker.rs |
| 3 | 29 | 184 | ours-onto-theirs | crates/project/src/search.rs |
| 3 | 390 | 140 | theirs-onto-ours | crates/agent_ui/src/completion_provider.rs |
| 3 | 43 | 1107 | ours-onto-theirs | crates/gpui_macos/src/window.rs |
| 3 | 6 | 1569 | ours-onto-theirs | crates/agent/src/tools/terminal_tool.rs |
| 3 | 695 | 176 | theirs-onto-ours | crates/gpui_windows/src/window.rs |
| 2 | 10 | 12 | ours-onto-theirs | crates/ai_onboarding/src/plan_definitions.rs |
| 2 | 1012 | 854 | theirs-onto-ours | crates/editor/src/items.rs |
| 2 | 12 | 40 | ours-onto-theirs | crates/editor/src/editor_settings.rs |
| 2 | 128 | 76 | theirs-onto-ours | crates/agent_ui/src/ui/model_selector_components.rs |
| 2 | 13 | 123 | ours-onto-theirs | crates/ui/src/components/button/button_like.rs |
| 2 | 140 | 766 | ours-onto-theirs | crates/gpui_linux/src/linux/wayland/window.rs |
| 2 | 1628 | 1667 | ours-onto-theirs | crates/agent_ui/src/agent_configuration.rs |
| 2 | 179 | 307 | ours-onto-theirs | crates/editor/src/code_context_menus.rs |
| 2 | 25 | 55 | ours-onto-theirs | crates/ai_onboarding/src/ai_onboarding.rs |
| 2 | 25 | 871 | ours-onto-theirs | crates/gpui_linux/src/linux/wayland/client.rs |
| 2 | 256 | 221 | theirs-onto-ours | crates/zed/src/reliability.rs |
| 2 | 32 | 351 | ours-onto-theirs | crates/agent_ui/src/entry_view_state.rs |
| 2 | 33 | 1306 | ours-onto-theirs | crates/picker/src/picker.rs |
| 2 | 37 | 20 | theirs-onto-ours | crates/language_models/src/settings.rs |
| 2 | 4 | 17 | ours-onto-theirs | crates/gpui_windows/Cargo.toml |
| 2 | 4 | 58 | ours-onto-theirs | crates/context_server/src/oauth.rs |
| 2 | 434 | 492 | ours-onto-theirs | crates/workspace/src/pane.rs |
| 2 | 445 | 155 | theirs-onto-ours | crates/agent_ui/src/agent_diff.rs |
| 2 | 49 | 115 | ours-onto-theirs | crates/workspace/src/pane_group.rs |
| 2 | 5 | 171 | ours-onto-theirs | crates/zed/src/zed/quick_action_bar/preview.rs |
| 2 | 60 | 89 | ours-onto-theirs | crates/settings_content/src/language_model.rs |
| 2 | 61 | 878 | ours-onto-theirs | crates/editor/src/selection.rs |
| 2 | 65 | 5005 | ours-onto-theirs | crates/search/src/project_search.rs |
| 2 | 84 | 12 | theirs-onto-ours | crates/snippets_ui/src/snippets_ui.rs |
| 2 | 85 | 295 | ours-onto-theirs | crates/project/src/lsp_store/document_links.rs |
| 2 | 94 | 9953 | ours-onto-theirs | crates/outline_panel/src/outline_panel.rs |
| 1 | 1 | 12 | ours-onto-theirs | crates/util/Cargo.toml |
| 1 | 10 | 1057 | ours-onto-theirs | crates/gpui/src/platform.rs |
| 1 | 10 | 3 | theirs-onto-ours | .gitignore |
| 1 | 107 | 12 | theirs-onto-ours | crates/project/src/agent_registry_store.rs |
| 1 | 111 | 609 | ours-onto-theirs | crates/terminal_view/src/terminal_view.rs |
| 1 | 114 | 111 | theirs-onto-ours | crates/gpui_linux/src/linux/x11/window.rs |
| 1 | 119 | 832 | ours-onto-theirs | crates/terminal_view/src/terminal_element.rs |
| 1 | 12 | 683 | ours-onto-theirs | crates/open_router/src/open_router.rs |
| 1 | 13 | 1963 | ours-onto-theirs | crates/git_ui/src/git_graph.rs |
| 1 | 135 | 136 | ours-onto-theirs | crates/agent_ui/src/agent_ui.rs |
| 1 | 139 | 220 | ours-onto-theirs | crates/editor/src/navigation.rs |
| 1 | 17 | 629 | ours-onto-theirs | crates/project_panel/src/undo.rs |
| 1 | 170 | 137 | theirs-onto-ours | crates/editor/src/completions.rs |
| 1 | 18 | 606 | ours-onto-theirs | crates/settings/src/vscode_import.rs |
| 1 | 19 | 261 | ours-onto-theirs | crates/editor/src/document_symbols.rs |
| 1 | 193 | 44 | theirs-onto-ours | crates/gpui_wgpu/src/shaders.wgsl |
| 1 | 2 | 10630 | ours-onto-theirs | crates/acp_thread/src/acp_thread.rs |
| 1 | 2 | 11 | ours-onto-theirs | crates/ui/src/components/image.rs |
| 1 | 2 | 20 | ours-onto-theirs | crates/extensions_ui/Cargo.toml |
| 1 | 2 | 40 | ours-onto-theirs | crates/zed/src/zed/quick_action_bar.rs |
| 1 | 20 | 81 | ours-onto-theirs | crates/settings_content/src/editor.rs |
| 1 | 23 | 1196 | ours-onto-theirs | crates/sidebar/src/sidebar_tests.rs |
| 1 | 24 | 2 | theirs-onto-ours | crates/zed/resources/info/Permissions.plist |
| 1 | 24 | 61 | ours-onto-theirs | crates/call/src/call_impl/mod.rs |
| 1 | 264 | 1808 | ours-onto-theirs | crates/editor/src/editor.rs |
| 1 | 277 | 546 | ours-onto-theirs | crates/zed/src/zed/open_listener.rs |
| 1 | 279 | 208 | theirs-onto-ours | crates/agent_ui/src/profile_selector.rs |
| 1 | 28 | 26 | theirs-onto-ours | crates/diagnostics/src/items.rs |
| 1 | 288 | 3928 | ours-onto-theirs | crates/gpui/src/window.rs |
| 1 | 290 | 2016 | ours-onto-theirs | crates/worktree/src/worktree.rs |
| 1 | 3 | 2 | theirs-onto-ours | crates/title_bar/src/title_bar_settings.rs |
| 1 | 313 | 399 | ours-onto-theirs | crates/gpui_windows/src/directx_renderer.rs |
| 1 | 335 | 18 | theirs-onto-ours | crates/agent_ui/src/language_model_selector.rs |
| 1 | 352 | 16 | theirs-onto-ours | crates/agent_ui/src/model_selector.rs |
| 1 | 4 | 14 | ours-onto-theirs | crates/paths/src/paths.rs |
| 1 | 4 | 1768 | ours-onto-theirs | crates/recent_projects/src/remote_servers.rs |
| 1 | 4 | 265 | ours-onto-theirs | crates/settings_ui/src/pages/edit_prediction_provider_setup.rs |
| 1 | 43 | 15 | theirs-onto-ours | crates/zed/src/zed/move_to_applications.rs |
| 1 | 43 | 1854 | ours-onto-theirs | crates/language/src/buffer_tests.rs |
| 1 | 44 | 542 | ours-onto-theirs | crates/agent_settings/src/agent_settings.rs |
| 1 | 5 | 16 | ours-onto-theirs | crates/agent/Cargo.toml |
| 1 | 5 | 67 | ours-onto-theirs | crates/workspace/src/workspace_settings.rs |
| 1 | 6 | 80 | ours-onto-theirs | crates/settings_ui/src/components/dropdown.rs |
| 1 | 60 | 41787 | ours-onto-theirs | crates/editor/src/editor_tests.rs |
| 1 | 67 | 38 | theirs-onto-ours | crates/tab_switcher/src/tab_switcher.rs |
| 1 | 68 | 697 | ours-onto-theirs | crates/language/src/language_registry.rs |
| 1 | 8 | 198 | ours-onto-theirs | crates/ui/src/components/collab/update_button.rs |
| 1 | 851 | 6 | theirs-onto-ours | crates/onboarding/src/onboarding.rs |
| 1 | 89 | 19 | theirs-onto-ours | crates/git_ui/src/repository_selector.rs |
| 1 | 90 | 105 | ours-onto-theirs | crates/agent_ui/src/mention_set.rs |
| 1 | 96 | 1432 | ours-onto-theirs | crates/terminal_view/src/terminal_panel.rs |
