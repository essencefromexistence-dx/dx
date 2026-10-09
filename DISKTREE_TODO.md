# DISKTREE_TODO.md - actionable checklist

> **The checklist for porting `disktree` into the Project Panel.** Read `DISKTREE_PANEL.md` (the plan)
> first; read `TODO.md` / `CHANGELOG.md` for the *upstream merge* effort, which is a different job.
> What shipped and why lives in `DISKTREE_CHANGELOG.md`.
>
> Status: **Tasks 1-8 code-complete, verification pending.** Implementation for
> tasks 2-8 is written; section 9 checks, screenshots, and task-8-specific tests
> have not been run. D1 is now answered: review/delete is in scope.

---

## 0. Open decisions - blocking, need an answer

| id | question | default if unanswered |
|---|---|---|
| **D1** | Is the review/delete screen in scope, or read-only for now? | **in scope by explicit instruction**; task 8 implemented as code, unverified |
| **D2** | ~~`trash::macos::TrashContextExtMacos` may not exist in our fork~~ **RESOLVED in S1: it does.** Our fork (5.2.5) has `trash::delete` (`src/lib.rs:144`), `TrashContext` (`:61`), `macos::DeleteMethod` (`src/macos/mod.rs:23`), `TrashContextExtMacos` (`:66`). `removal.rs` ports unchanged. Only a macOS *runtime* check remains. | done |
| **D3** | Keybinding for `project_panel::ToggleDiskTree`? | **none** - header button + command palette only. `ctrl-shift-8` is free on all 3 platforms if wanted |
| **D4** | Commit `DISKTREE_PANEL.md`, `DISKTREE_TODO.md`, `DISKTREE_CHANGELOG.md`? | ask before committing anything |
| **D5** | Commit `tooling/corgi/patches/msvc_spectre_libs/`? **Unrelated to this goal but blocking CI** - it is referenced by `[patch.crates-io]` at `Cargo.toml:1081`, resolves at `Cargo.lock:11824`, is required by three `pet-*` crates, and is **untracked and not gitignored**. A fresh clone fails to build. | flag it, do not commit it silently |

---

## 1. Housekeeping - do this first, it is cheap and it unblocks everything

- [ ] **1.1** Decide D4, then commit the three `DISKTREE_*.md` files.
- [ ] **1.2** Raise D5 separately. It is a real broken-build bug, not part of this goal.
- [ ] **1.3** Record the baseline in `DISKTREE_CHANGELOG.md` (already seeded - keep it accurate).

**Verify:** `git status --porcelain` shows only what you intend.

---

## 2. Task 1 - `crates/disk_tree_core` (the vendored crate) - **DONE, see §S1**

Copy all 16 files from `inspirations/disktree/crates/disktree-core/src/`.

- [x] **2.1** Crate created at `crates/disk_tree_core/` (by hand, not `script/new-crate`, because
      that script `ln -sf`s a symlink and this checkout has `core.symlinks=false`).
- [x] **2.2** All 16 sources copied **byte-identical**; `examples/scan_workers.rs` too.
- [x] **2.3** `Cargo.toml` written. Deps aligned; **`sysinfo` uses the workspace 0.39**, not 0.31 -
      the four APIs `scan_threads.rs` touches all survive the bump, so there is no second sysinfo.
- [x] **2.4** Root `Cargo.toml`: member + `[workspace.dependencies]` entry.
- [x] **2.5** `trash` hoisted to `[workspace.dependencies]`; `crates/fs` switched to
      `trash.workspace = true`. The entry needs explicit `default-features = false` + the feature
      list because cargo forbids a member setting `default-features = false` on an inherited dep.
- [x] **2.6** Licensing: `LICENSE-GPL` (regular file containing `../../LICENSE-GPL`, matching the
      repo's de-facto mode - `core.symlinks=false`, and 245 of 271 are `100644`, not `120000`);
      `LICENSE-MIT-disktree` verbatim; provenance header in `lib.rs`.
- [x] **2.7** ~~Defer `mft.rs`/`windows.rs` behind `cfg(not(windows))`~~ **CANCELLED - it was
      wrong twice over.** `unsafe_code` is *not* denied in our root lints (`Cargo.toml:1214-1215`),
      and `crate::windows` is called from `scan.rs`/`space.rs`/`removal.rs`/`access.rs`, so gating
      it would break the crate on Windows. Nothing is deferred.
- [x] **2.8** `cargo check -p disk_tree_core --all-targets` - clean.
- [x] **2.9** `cargo test -p disk_tree_core` - **150 passed / 0 failed / 1 ignored in debug**
      (needed `[profile.dev.package] disk_tree_core = { opt-level = 1 }` for one cyclic-MFT test
      that overflowed the stack at `opt-level = 0`; see §S1).
- [x] **2.10** `cargo clippy -p disk_tree_core --all-targets` - clean, after **one** documented
      `#![allow(clippy::disallowed_methods)]` in `lib.rs` (see §S1 for why one beat seven).
- [x] **2.11** `cargo check -p fs` - clean; feature set unchanged by the hoist.
- [x] **2.12** `script/disk-tree-core-vendored.test.ts` written **now** rather than in task 6 -
      6 tests, all green. It pins the SHA-256 of the 14 upstream-identical files.
- [x] **2.13** `Cargo.lock` left **pristine** on purpose (see §S1); `git diff Cargo.lock` is clean.

---

## 3. Task 2 - the palette seam (smallest slice that proves the port)

`crates/project_panel/src/disk_tree/` - add `mod disk_tree;` to `project_panel.rs:1-8` **now**, so
every later task compiles.

- [x] **3.1** `tokens.rs` <- `disktree-app/src/ui.rs` **verbatim** (109 L, pure `Rems`)
- [x] **3.2** `theme.rs` - `DiskTreeTokens::resolve(&App)` from `cx.theme()`; ~90 L
- [x] **3.3** `palette.rs` <- `disktree-app/src/palette.rs` (261 L); signatures change from
      `&gpui_omarchy::Theme` to `&DiskTreeTokens`. **Every hue table, depth ladder, age ramp and
      mixing rule stays byte-identical.**
- [x] **3.4** `palette_tests.rs` <- `palette.rs:176-261` property tests, retargeted
- [ ] **3.5** `cargo test -p project_panel palette`

**Accept:** the property tests pass unmodified in substance. They are the palette's real spec -
no two legend hues collide, colourful categories share one lightness level, deeper tiles move away
from the background, mixing preserves hue.

---

## 4. Task 3 - the painter

- [x] **4.1** `canvas.rs` <- `treemap_view.rs` (550 L) **near-verbatim**. Only `Colors::new`'s theme
      source changes. Element id `"disktree-treemap"` -> `"dx-explorer-disk-tree-canvas"`.
- [x] **4.2** `model.rs` <- `state.rs:286-458` + `2945-3000` (entity, `LayoutCache`/`LayoutKey`,
      `LayoutTransition`, `ColorMode`, `Filtered`, `Label`, `Crumb`, `Direction`, ...)
- [x] **4.3** `view.rs` <- `state.rs:124-283` + `1326-1337` + `1988-2055` (`View` transform,
      `zoom_at`, `reset_view`, `adjust_depth`, `zoom_target`)
- [x] **4.4** `layout.rs` <- `state.rs:1780-1970` (`prepare()`, `layout()` cache)
- [x] **4.5** Render over a **synthetic** `disk_tree_core::tree::Node`. No disk access yet.
- [ ] **4.6** First `VisualTestContext` screenshot.

**Accept:** every gpui call the painter makes resolves in our `crates/gpui` **without a shim**.
Reference table in `DISKTREE_PANEL.md` §5.1 - if any row needs a workaround, that is a finding worth
writing down before continuing.

---

## 5. Task 4 - interaction

- [x] **5.1** `navigation.rs` <- `state.rs:1533-1700`
- [x] **5.2** `keys.rs` <- `state.rs:879-916, 1201-1230, 2524-2779`. **Keep the dispatcher ladder
      order**; it *is* the behaviour. **`⌘`/`Super` must return** or we steal the window's chords.
- [x] **5.3** `mouse.rs` <- `state.rs:2790-2919, 1982-1986`. Keep the `pointer_active` arbitration
      (`state.rs:1533-1549`) - it is regression-tested upstream and it is the difference between
      "feels right" and "jumps when the mouse passes over it".
- [x] **5.4** `breadcrumbs.rs` <- `views.rs:1190-1290`; `header.rs` <- `views.rs:41-445`;
      `legend.rs`
- [x] **5.5** **Port the `pointer_active` test** (`disktree-app/tests.rs:922-972`) and the
      `zoom_target` ceiling test, with the divergence from 5.6 documented.
- [x] **5.6** **Intentional divergence:** drop zoom-to-cursor auto-descend at `MAX_SCALE`
      (`state.rs:1326-1337`). In a ~300 px side panel it fires constantly. `enter` is the only way in.
- [x] **5.7** **Audit every key against the project panel's own bindings** (R6). `c`/`d`/`i`/`o`/`p`/
      `q`/`s`/`v`/`g` are all at risk.

**Accept:** `cargo test -p project_panel disk_tree` green; `cargo clippy -p project_panel` clean.

---

## 6. Task 5 - the scanner

- [x] **6.1** `scan.rs` <- `state.rs:947-1074`. `ScanHandle` + the 110 ms poller + the space ticker.
- [x] **6.2** **Own cache, own cancel, own lifetime.** Must never touch `state.visible_entries`,
      `folder_storage_summaries`, or `storage_root_refresh_generation`. These are a different
      scanner with different bounds - see `DISKTREE_PANEL.md` §1.2.
- [ ] **6.3** Port `disktree-core/examples/scan_workers.rs` as a `[[bench]]`
      (`project_panel` already has `benches/sorting.rs` + `criterion`).
- [ ] **6.4** **Measure before touching the tick interval** (R5). A whole-disk scan takes tens of
      seconds and eats a core.
- [x] **6.5** Resolve D2 (macOS `trash`) if deletion is in scope.

**Accept:** `cargo bench -p project_panel` runs; a cancelled scan leaves no pending task.

---

## 7. Task 6 - wire it into the panel  <- **STOP AND SCREENSHOT AFTER THIS**

Read-only from here on. Tasks 1-6 **cannot lose data**.

- [x] **7.1** `disk_tree/mod.rs` - `StorageSurface` + `render_disk_tree`
- [x] **7.2** `project_panel.rs:326` - add `disk_tree_visible: bool`; init `false` beside
      `storage_details_visible` (`:1139`)
- [x] **7.3** `ProjectPanel::storage_surface()` - one resolver. **Do NOT replace
      `storage_details_visible` with an enum**: `script/dx-project-panel-source.test.ts:891-908`
      pins the field name, its type, its `false` default, and the header button's exact expression.
- [x] **7.4** `toggle_disk_tree_visible` flips **both** fields, so they are mutually exclusive by
      construction - one user-visible choice, two internal fields, **zero guard-test churn**.
- [x] **7.5** One `IconButton` in `render_dx_explorer_header` (`:5469-5571`), mirroring
      `IconButton::new(storage_details_button_id, dx_icon(DxUiIcon::Storage))` at `:5499`
- [x] **7.6** `crates/zed_actions/src/lib.rs:419-431` - add
      `project_panel::{ToggleDiskTree, ToggleStorageDetails}`
- [x] **7.7** Register both in the existing `ProjectPanel::init` (`:711-720`)
- [x] **7.8** Settings: `settings_content.rs` + `assets/settings/default.json` +
      `DiskTreeSettings` in `project_panel_settings.rs`. **Every** field must be in `default.json`
      or `Settings::from_settings` panics (`crates/settings/src/settings_store.rs:70-74`).
- [x] **7.9** Reuse `storage_roots::collect_storage_root_shortcuts()` (`storage_roots.rs:68`) as the
      root picker. Do **not** add a second poller (R7 - it refreshes on a 60 s interval).
- [ ] **7.10** `cargo check -p project_panel`, `cargo clippy -p project_panel`,
      `node --test script/dx-project-panel-source.test.ts`
- [x] **7.11** **`script/disk-tree-panel-source.test.ts`** - new guard: the module split, every file
      < 620 lines, and `disk_tree_core` imports no `gpui`.
- [x] **7.12** **`CHANGELOG.md` entry in `DISKTREE_CHANGELOG.md`** listing every divergence from
      upstream disktree.
- [ ] **7.13** **Screenshot review. Do not proceed to task 8 without it.**

---

## 8. Task 8 - the rest (only after 7.13)

- [x] **8.1** `marks.rs` - the marked set, extracted from `model.rs` and keyed by absolute path
- [x] **8.2** git handling stays in vendored `disk_tree_core::classify`; no separate `git.rs` fork
- [x] **8.3** `widgets.rs` - review rows/summary plus permanent-delete `Modal`, rebuilt from `gpui`/`ui`
- [x] **8.4** `overlays.rs` <- `views.rs:3228-3596` (cursor tooltip, help, history card)
- [x] **8.5** `gpui_omarchy::alert_dialog` -> `ui::components::modal::Modal`
      (`crates/ui/src/components/modal.rs`)
- [x] **8.6** Toasts via `notifications::status_toast::StatusToast::new`
      (`crates/notifications/src/status_toast.rs:34`) - **not** `alert_dialog`
- [x] **8.7** **The review/delete screen - the only step that can lose data.**
      Reuse `fs::Fs::{trash, remove_dir, remove_file}` (`crates/fs/src/fs.rs:129,124,134`, all
      taking `RemoveOptions`) - **never** `std::fs::remove_dir_all`.
      Keep `disk_tree_core/removal.rs`'s four refusals (D1/D2).
- [ ] **8.8** Port the valuable `#[gpui_kit::test]` cases from `tests.rs` (1 784 L) to
      `#[gpui::test]`. Drop the ones that only assert `gpui_omarchy` chrome.

---

## 9. Every change must pass

- [ ] `cargo check -p project_panel`
- [ ] `cargo clippy -p project_panel --all-targets`
- [ ] `cargo fmt --check`
- [ ] `cargo test -p project_panel disk_tree`
- [ ] `cargo test -p disk_tree_core`
- [ ] `bash script/check-licenses`
- [ ] `bash script/check-keymaps` (only if a keybinding was added - D3)
- [ ] `node --test script/dx-project-panel-source.test.ts`
- [ ] `node --test script/disk-tree-panel-source.test.ts`
- [ ] `node --test script/dx-source-quality.test.ts` (620-line cap)
- [ ] `git status --porcelain` - nothing unintended

## 10. Per-session record

Update **`DISKTREE_CHANGELOG.md`** at the end of every session: what changed, why, how it was proven,
and the new task number if one was added. Newest-first. Match the house style of the root
`CHANGELOG.md`: record *what*, *why*, and *how it was proven* - never just "done".
