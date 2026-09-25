# Research: `dx-code` vs latest upstream Zed — how hard is the merge?

Date: 2026-09-24
Ours: `merge-20260924` = `c701504082` (identical to `dev`)
Theirs: `upstream/main` = `baee1ca6de` (zed-industries/zed, fetched 2026-09-24)

---

## Verdict

**Hard, but bounded and doable — nowhere near as bad as the raw numbers suggest.**

The "1,784 conflicts" figure is an artefact of comparing two histories with no common ancestor. It is
not a measure of work. Measured properly against the real Zed base, the merge is:

| | |
|---|---|
| **true conflicts** | **257** |
| conflict hunks | **756** |
| of which `Cargo.lock` (regenerate, no thinking) | 353 |
| **real source hunks** | **~403**, concentrated in ~20 files |
| conflicts needing *zero* judgement | ~114 (files we deliberately deleted) |

---

## 1. Why the 1,784 number was misleading

`dev` has **no common ancestor** with upstream — it is a fresh root commit (`8dc3418ec1`,
2026-07-17) carrying a whole tree. `git merge-base merge-20260924 upstream/main` returns empty.

With no base, git falls back to the **empty tree**, so *every file that differs becomes an
`add/add` collision*:

| | count |
|---|---|
| `add/add` | 1,543 |
| `distinct types` (LICENSE symlink vs regular) | 241 |
| `content` | **0** |
| total records | 1,784 |
| affected paths | 2,025 |

Zero `content` conflicts is the tell: a real merge never produces 0 content conflicts and 1,543
whole-file collisions. This output is a comparison, not a merge.

## 2. How the real base was found

The base is the upstream commit whose tree is closest to ours. Searched upstream commits from
2026-04-15 to 2026-07-25 and minimised `git diff --name-only <candidate> merge-20260924`:

| base | date | differing files |
|---|---|---|
| `b259b346bf` | 2026-06-01 | 25,922 |
| `8807ac4f3b` | 2026-06-05 | 25,713 |
| `c32c037c6b` | 2026-06-12 | 25,482 |
| **`d8eb569cc5`** | **2026-06-15** | **25,461** ← minimum |
| `51fac82ddd` | 2026-06-22 | 25,728 |
| `3856272188` | 2026-06-30 | 25,898 |

A sharp minimum with the curve rising on both sides — the signature of a real fork point. The base
is a **synthetic** one (not an ancestor of either side), which is exactly what
`git merge-tree --merge-base=<commit>` accepts.

Conflict count is stable around it, so 257 is trustworthy:

| base date | conflicts |
|---|---|
| 2026-06-01 | 515 |
| 2026-06-05 | 390 |
| 2026-06-12 | 289 |
| **2026-06-15** | **257** |
| 2026-06-22 | 415 |
| 2026-06-30 | 436 |

```
git merge-tree --write-tree --merge-base=d8eb569cc5 merge-20260924 upstream/main
→ merged tree 61ff6f4ddf, exit 1, 257 CONFLICT records
```

## 3. Tree-level facts

| | files |
|---|---|
| ours (`merge-20260924`) | **28,216** |
| theirs (`upstream/main`) | **4,323** |
| shared by name | 3,686 |
| …identical content | 1,902 |
| …differing | 1,784 |
| ours only | 24,504 |
| theirs only | 611 |

**The single most important number: only 3,686 files are shared.** Our 24,504 private files — almost
all of `web/**` (Router, Design, Graphics, Metasearch, Shader, Whiteboard, Presentations, Video, …) —
can never conflict. Upstream's 611 new files simply arrive.

Of the 1,784 shared-but-differing files, **1,527 changed on only one side**, so they merge cleanly.
Only 257 changed on both sides.

## 4. The 257 conflicts, by kind

| type | count | effort |
|---|---|---|
| `modify/delete` — **we deleted, upstream kept** | **114** | mechanical: keep deleted |
| `content` | 121 | **the real work** |
| `modify/delete` — upstream deleted, we kept | 9 | decide per file |
| `rename/delete` | 7 | low |
| `add/add` | 4 | low |
| `rename/rename` | 1 | low |
| `distinct types` | 1 | trivial |

### The 114 "we deleted" files — not work at all

These are files our fork removed that upstream still has and still edits. Confirmed sample:

```
.cargo/config.toml          docs/README.md            docs/src/ai/agents.md
.zed/settings.json          docs/book.toml            docs/src/ai/agent-panel.md
CONTRIBUTING.md             docs/src/SUMMARY.md       docs/src/account/billing.md
docs/.rules                 docs/.gitignore           docs/src/ai/edit-prediction.md
crates/*/LICENSE-GPL, LICENSE-APACHE (symlinks we dropped)
```

**110 of the 259 conflicted paths are `docs/`** — the entire Zed documentation tree, which this fork
deleted. Resolution is `--force-remove` on the whole batch. Zero judgement.

Note `.cargo/config.toml` is one of them: it exists in the base and upstream, but **not** in our tree.
That is why the `justfile` comment claiming "Cargo output is pinned to G:/Dx/code/target in
.cargo/config.toml" is stale — there is no such file here. (The target dir is just the default
`target/`.)

### The 9 upstream deletions we still carry

```
.github/workflows-disabled/triage_project_sync.yml.disabled
crates/agent_ui/src/agent_configuration/add_llm_provider_modal.rs
crates/csv_preview/src/csv_preview.rs
crates/csv_preview/src/renderer/settings.rs
crates/edit_prediction/src/capture_example.rs
crates/extensions_ui/src/extension_suggest.rs
script/github-label-issues-to-triage.py
script/redeploy-vercel
script/storybook
```

Upstream removed these; we still have them. Either accept the deletion (and fix any references) or
keep ours. Small, but each needs a decision.

## 5. Conflict hunks — where the real work is

756 `<<<<<<<` hunks across 126 files (fixed-string count; the usual figlet/test-fixture false
positives apply).

| file | hunks |
|---|---|
| `Cargo.lock` | **353** |
| `crates/agent_ui/src/conversation_view/thread_view.rs` | 28 |
| `crates/project_panel/src/project_panel.rs` | 20 |
| `crates/sidebar/src/sidebar.rs` | 18 |
| `crates/workspace/src/workspace.rs` | 15 |
| `crates/extensions_ui/src/extensions_ui.rs` | 12 |
| `crates/title_bar/src/title_bar.rs` | 10 |
| `crates/language_models/src/provider/llama_cpp.rs` | 10 |
| `crates/agent_ui/src/agent_panel.rs` | 10 |
| `Cargo.toml` | 10 |
| `crates/settings_ui/src/settings_ui.rs` | 9 |
| `crates/language_models/src/provider/opencode.rs` | 9 |
| `crates/project/src/context_server_store.rs` | 8 |
| `crates/agent/src/thread.rs` | 8 |
| `crates/gpui_wgpu/src/wgpu_renderer.rs` | 7 |
| `crates/git_ui/src/git_panel.rs` | 7 |
| `crates/editor/src/element.rs` | 7 |
| `crates/gpui_apple/src/metal_renderer.rs` | 6 |
| `crates/agent_ui/src/conversation_view.rs` | 6 |
| `crates/zed/src/main.rs` | 5 |

`Cargo.lock` (353 hunks) is regenerated, not merged — drop it from the effort estimate.
That leaves **~403 source hunks**.

The hotspots are **exactly the fork's UI rewrite**: `sidebar`, `title_bar`, `settings_ui`,
`project_panel`, `thread_view`, `agent_panel`, `workspace`, `extensions_ui`. This matches the warning
already recorded in this repo's handoff — a previous upstream merge *destroyed* Dx code in
`thread_view.rs` and `agent_panel.rs`, and restored only 6 files. **These files must stay ours.**

## 6. Recommended strategy

### Do not run a plain `git merge`
With unrelated histories it produces the useless 1,784-conflict merge, and a large `git merge` on the
G: volume has historically **wedged** (the safe-delete guard stalls on git's bulk deletions).

### Resolve in the object DB against the synthetic base
`git merge-tree --write-tree --merge-base=<base>` already works and writes nothing to the worktree.

1. `git merge-tree --write-tree --merge-base=d8eb569cc5 merge-20260924 upstream/main` → tree
   `61ff6f4ddf` (done; 257 conflicts, 756 hunks).
2. Seed a **scratch index** (`GIT_INDEX_FILE=<temp>`), `read-tree` that tree.
3. Resolve in three batches:
   - **batch A (114 + 1)** — the files we deleted: `update-index --force-remove`. Pure script.
   - **batch B (Cargo.lock + Cargo.toml)** — take ours, then regenerate `Cargo.lock` with cargo.
   - **batch C (121 content + 9 modify/delete + renames)** — the real review, prioritised by hunk
     count, starting with the ~20 files listed above.
4. `write-tree` + `commit-tree -p merge-20260924 -p upstream/main`.
5. Hand the user `git reset --hard <resolved>` (or just create a branch at it).

This touches zero worktree files until the final step, so the deletion guard cannot interfere.

### Fix the root cause: re-root the fork
Every future upstream merge will need this synthetic-base dance unless the histories are joined.
Once the merge above is done and reviewed, **grafting our root commit onto the Zed base**
(`git replace --graft`, then a `filter-repo`/`replace` bake-in) turns `dx-code` into an ordinary
fork where `git merge upstream/main` just works. Worth doing once, right after this merge lands.

## 7. Caveats

- The base `d8eb569cc5` is **inferred**, not recorded — the fork's real base commit is not in this
  history. 257 is accurate *for that base*; the true figure is unlikely to be far off given the
  sharp minimum.
- Rename detection was **skipped** for the 24k-file diff (`diff.renameLimit` too low), so the
  `rename/delete` and `rename/rename` counts (7 + 1) may be understated. Raise
  `diff.renameLimit` above 24,500 before trusting those.
- `Cargo.lock` must be regenerated, not merged; `Cargo.toml` (10 hunks) needs a careful manual pass.
- **Nothing here was compiled.** A Zed-scale build was not attempted, so the merge cannot yet be
  called correct — only computed.
