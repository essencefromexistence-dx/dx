# DISKTREE_PANEL.md - port `disktree` into the Project Panel's file tree

> **Plan, not work.** Nothing here is implemented. Sibling docs for *this* goal:
> `DISKTREE_TODO.md` (checklist) and `DISKTREE_CHANGELOG.md` (what shipped, and why).
> The root `PLAN.md` / `TODO.md` / `CHANGELOG.md` belong to the **upstream Zed merge** - a different
> job; do not mix them.
> Same discipline: verify against the tree before asserting, record what was proven.

---

## 1. The placement answer

**Not a new dock panel. A new module subtree inside `crates/project_panel/src/`, next to the
`storage*` files that already own this exact concept.**

```
crates/project_panel/src/
├── project_panel.rs            10 925   existing — dispatch + state fields only, do NOT grow
├── project_panel_settings.rs      150   existing
├── storage.rs                      369   existing — UNCHANGED (project-index storage)
├── storage_roots.rs                246   existing — UNCHANGED, reused as the root picker
├── storage_roots_view.rs           168   existing — +1 button that switches to the treemap
├── operation_status.rs              25   existing
├── media_preview/                       existing
└── disk_tree/                           NEW — the whole disktree port, one directory
    ├── mod.rs                    ~150
    ├── tokens.rs                 109   <- disktree ui.rs            VERBATIM
    ├── theme.rs                  ~ 90
    ├── palette.rs                261   <- disktree palette.rs       retargeted
    ├── palette_tests.rs          ~ 95   <- disktree palette.rs:176-261
    ├── model.rs                  ~330   <- disktree state.rs:286-458, 2945-3000
    ├── view.rs                   ~260   <- disktree state.rs:124-283 + zoom math
    ├── layout.rs                 ~200   <- disktree state.rs:1780-1970
    ├── navigation.rs             ~280   <- disktree state.rs:1533-1700
    ├── keys.rs                   ~300   <- disktree state.rs:2524-2779
    ├── mouse.rs                  ~180   <- disktree state.rs:2790-2919
    ├── scan.rs                   ~260   <- disktree state.rs:947-1074
    ├── canvas.rs                 550   <- disktree treemap_view.rs  NEAR-VERBATIM
    ├── header.rs                 ~330   <- disktree views.rs:41-445
    ├── breadcrumbs.rs            ~200   <- disktree views.rs:1190-1290
    ├── legend.rs                 ~180
    ├── widgets.rs                541   <- disktree widgets.rs        retargeted
    ├── overlays.rs               ~330   <- disktree views.rs:3228-3596
    ├── marks.rs                  235   <- disktree marks.rs         VERBATIM
    ├── git.rs                    407   <- disktree git.rs           VERBATIM
    └── tests.rs                  <620   ported #[gpui::test]s

crates/disk_tree_core/           NEW crate — vendored disktree-core, no gpui (§4)
```

`disktree-app`'s `state.rs` (3 321 L) + `views.rs` (3 596 L) = 6 917 lines become **21 focused
modules**, each under the repo's 620-line owned-module cap
(`script/dx-source-quality.test.ts:13`).

### 1.1 Why here and not a dock panel — five things that already exist

| # | existing code | what it already does | what disktree brings |
|---|---|---|---|
| 1 | `storage_roots.rs:111-146` `collect_drive_shortcuts` | enumerates every mounted volume via `sysinfo::Disks`, keeps `available_space()` + `total_space()` | replaces disktree's `space.rs` volume picker (1 041 L) — **we already have it** |
| 2 | `storage_roots.rs:11-18` `StorageRootKind` | `Drive`, `DxHub`, `OneDrive`, `GoogleDrive`, `Dropbox`, each with `path` + `capacity` + `available` | replaces disktree's "volume picker" screen outright |
| 3 | `storage.rs:136-166` `StorageSortMode` | `Size` / `FileCount` / `Modified` | **the same three metrics** as disktree's `Metric::{Bytes,Files}` + age mode |
| 4 | `storage.rs:244` `storage_heat_level` → `u8` 0..=4, `project_panel.rs:9820` `dx_explorer_storage_heat_color(heat_level) -> Color` | a 4-step categorical heat ramp for storage rows | proof the "colour by size class" idea is already house style |
| 5 | `storage_roots_view.rs:18` `render_storage_root_strip` | the **Storage** strip pinned above the tree, `DxUiIcon::Storage` → `IconName::DatabaseZap` (`crates/ui/src/dx_icons.rs:44,89`), element ids `dx-explorer-storage-root-*` | the natural home for a "show as treemap" affordance |

And `project_panel` **already depends on** `sysinfo`, `rayon`, `fs`, `notifications`, `ui`,
`theme`, `gpui` (`crates/project_panel/Cargo.toml`) — every dependency the scan needs.

A separate dock panel would have had to re-implement 1-5 and would have been a second, competing
"storage" surface in the same window.

### 1.2 The one hard constraint: `storage.rs` is NOT the same scanner

Do not "reuse" `storage.rs` for the treemap. They are different problems:

| | `storage.rs` (existing) | disktree `scan.rs` |
|---|---|---|
| input | `state.visible_entries` — the project's own **gitignore-respecting index** | an arbitrary directory on the OS filesystem |
| bounds | `MAX_PROJECT_PANEL_FOLDER_STORAGE_CHILD_FILES = 512`, `..._BACKGROUND_FOLDER_STORAGE_DIRS = 4_096`, `..._SUMMARY_CACHE = 4_096` (`project_panel.rs:122-124`) | bounded only by the disk |
| cost | incremental, cheap, per-visible-row | a full subtree walk; millions of `Node`s for `G:\` |
| output | `StorageFolderItem` / `StorageOverview` (flat, top-5) | a full `tree::Node` with `bytes`/`own_bytes`/`files`/`dirs`/`mtime`/`category`/`reclaim` |
| hardlink/mount dedup | none | `dedup_hardlinks`, `one_filesystem`, mount de-dup |

⇒ The treemap gets **its own scanner, its own cache, its own cancel, its own lifetime** — inside
`disk_tree/scan.rs`, over `disk_tree_core::scan`. It **reads** `storage_roots.rs` for the root
picker and **never touches** `state.visible_entries`, `folder_storage_summaries`, or
`storage_root_refresh_generation`.

### 1.3 The one place I could *not* refactor, and why

It would be tidier to lift `render_dx_explorer_storage_heat_indicator` /
`dx_explorer_storage_heat_indicator_width` / `dx_explorer_storage_heat_color`
(`project_panel.rs:9808-9825`) into a `storage_heat.rs` shared with the treemap palette.

**`script/dx-project-panel-source.test.ts:934-936` pins all three to `project_panel.rs` by regex.**
Moving them means rewriting three assertions in a 2 850-line guard. Not worth it.

And the treemap should **not** share the heat ramp anyway: that is a 4-step `Indicator::bar`
(width + colour) for one list row; the treemap needs a **9-category x 5-depth HSL fill ladder**
(disktree's `palette.rs`). Different problems. Leave a comment on `storage_heat_color` saying so,
so the next reader does not try to merge them.

---

## 2. The mode toggle — satisfies the guard unchanged

`script/dx-project-panel-source.test.ts` pins the existing storage surface hard:

- `:891` `storage_details_visible:\s*bool` must exist
- `:896` `storage_details_visible:\s*false` must be the default
- `:901` the header control must match
  `IconButton::new(storage_details_button_id, dx_icon(DxUiIcon::Storage))` … `Show storage details`
  … `this.toggle_storage_details_visible(window, cx)` — **the exact expression**
- `:906` `render_dx_explorer_storage_drilldown` must early-return on `!self.storage_details_visible`
- `:918-919` `fn dx_explorer_storage_drilldown_items(` and `fn dx_explorer_storage_overview(` must
  **not** exist

So: **do not replace the bool with an enum.** Add a sibling field and one resolver.

```rust
// project_panel.rs, next to storage_details_visible:326
disk_tree_visible: bool,          // default false — opt-in, like its sibling
```

```rust
// disk_tree/mod.rs
pub(crate) enum StorageSurface { Treemap, Details }

impl ProjectPanel {
    /// The one user-facing choice: treemap, largest-folders list, or neither.
    pub(crate) fn storage_surface(&self) -> Option<StorageSurface> {
        if self.disk_tree_visible {
            Some(StorageSurface::Treemap)
        } else if self.storage_details_visible {
            Some(StorageSurface::Details)
        } else {
            None
        }
    }
}
```

`toggle_disk_tree_visible` flips **both** fields, so they are mutually exclusive by construction
and there is exactly one user-visible choice. Every existing call site of
`storage_details_visible` (`project_panel.rs:5147, 5469, 5503, 5514, 5520, 5561, 5571, 6259, 6284,
6453, 6773, 6797`) keeps compiling and keeps its meaning. **Zero guard-test churn.**

Actions, in `crates/zed_actions/src/lib.rs` beside `project_panel::{Toggle, ToggleFocus}` (`:419-431`):

```rust
project_panel::{
    /// Toggles the disk usage treemap in place of the storage list.
    ToggleDiskTree,
    /// Toggles the largest-folders storage list.
    ToggleStorageDetails,
}
```

Registered in the existing `ProjectPanel::init` (`project_panel.rs:711-720` pattern) via
`cx.observe_new(|workspace: &mut Workspace, ..)| .. workspace.register_action(..))`. Palette
visibility is free (`Window::available_actions`, `crates/gpui/src/window.rs:6865-6878`).

**No default keybinding.** This repo merges upstream constantly and chords are the churn hot spot
(`ctrl-shift-k` = DeleteLine, `ctrl-shift-y` = ToggleAllDocks, `ctrl-shift-d` = `git::Diff`).
`ctrl-shift-8` is free on all three platforms if you want one. The header button + the palette is
a complete toggle on its own.

---

## 3. What renders where

`ProjectPanel::render` (`project_panel.rs:9938`) currently emits, in order: header → storage root
strip → drilldown → tree. The treemap slots in **exactly where the drilldown is**, so the Storage
strip stays visible above it and remains the root picker.

```
render_dx_explorer_header            (5247)   + 1 IconButton: dx_icon(DxUiIcon::Storage), toggles Treemap
render_dx_explorer_storage_root_strip (5160)  unchanged — now the treemap's root picker
  ├── disk_tree::render (mode == Treemap)      NEW   canvas + breadcrumbs + legend + key bar
  └── render_dx_explorer_storage_drilldown (4892)      unchanged, gated on Details
render_entry (8073) … the file tree            unchanged
```

Header button shape mirrors the existing one exactly (`:5469-5571`):

```rust
IconButton::new(disk_tree_button_id, dx_icon(DxUiIcon::Storage))
    .icon_color(if disk_tree_visible { Color::Accent } else { Color::Muted })
    .tooltip(Tooltip::text(if disk_tree_visible { "Show storage treemap" } else { "Show storage treemap" }))
    .on_click(cx.listener(|this, _, window, cx| this.toggle_disk_tree_visible(window, cx)))
```

Element ids follow the existing convention: `dx-explorer-disk-tree`, `dx-explorer-disk-tree-canvas`,
`dx-explorer-disk-tree-breadcrumbs`. `disktree-app`'s own `"disktree-treemap"` id is dropped — our
prefix is `dx-explorer-` (`storage_roots_view.rs:37,51,83`).

---

## 4. `crates/disk_tree_core` — copy, do not rewrite

A separate crate for one reason only: **it has no `gpui` dependency**, so an upstream re-sync is a
file copy. Copy all 16 files from `inspirations/disktree/crates/disktree-core/src/`:

```
access.rs 97   classify.rs 619   export.rs 252   filter.rs 249   insights.rs 245
lib.rs 23      mft.rs 1713       removal.rs 2567 scan.rs 1802      scan_threads.rs 733
size.rs 135    space.rs 1041      tree.rs 561     treemap.rs 725    windows.rs 1151
```

Only these changes:

| what | from | to | why |
|---|---|---|---|
| `chrono` | `0.4.45` | workspace `chrono` | lock has `0.4.42`; one call site (`removal.rs:1532`). Avoids a version bump. |
| `rayon`, `rustc-hash`, `rustix` | `"1"`, `"2"`, `"1.1"` | `.workspace = true` | lock already resolves `1.11.0` / `2.1.1` / `1.1.4` |
| `trash` | crates.io `5.2.9` | workspace `trash` (**zed-industries fork `41c6c80`, 5.2.5**) | §7 risk R1 |
| `windows-sys` | `0.61` | `windows` `0.62`, features added to the root `Cargo.toml` | one lock, one Windows shim |
| `sysinfo` | `0.31.4` | workspace `sysinfo` — **already a `project_panel` dep** | zero new crates |
| `winapi-util` | `0.1.11` | verbatim | already in `Cargo.lock` |
| `[lints]` | disktree's table | `workspace = true` | keep disktree's *reasoning* as a comment |
| `license` | `MIT` | `GPL-3.0-or-later` + `LICENSE-GPL` symlink + **`LICENSE-MIT-disktree`** verbatim | `script/check-licenses:58-84`; MIT requires the notice survives |

**Net new external crates: zero.** `Cargo.lock`: **never** `cargo generate-lockfile`
(`CHANGELOG.md:18-21` — the `dx-media` git dep is unfetchable).

**Phase it:** land `cfg(not(windows))` first. `mft.rs` (1 713 L) + `windows.rs` (1 151 L) are
Windows/FFI-only, and our root `[workspace.lints]` denies `unsafe_code` — R2/R3.

### 4.1 What is *not* copied, because we already have it

| disktree module | our equivalent |
|---|---|
| `space.rs` volume enumeration (1 041 L) | `storage_roots.rs:111-146` (`sysinfo::Disks`) |
| `size.rs` `human_bytes` (135 L) | `storage.rs:254 format_file_size` |
| the volume-picker screen (`views.rs:93cb8c6`) | the Storage strip + `open_dx_explorer_storage_root` |
| `power.rs` private TOML preset file | `settings` + `SettingsStore` |
| `appearance.rs` theme bootstrap | `window.window_appearance()` — ours are settings, not files |
| `app_menu.rs` native menus | Zed's menu + command palette |
| `main.rs` window / CLI / cmux / console | none — we are a panel, not an app |

`git.rs` and `marks.rs` come across verbatim (no GPUI). `git.rs` shells out to `git` with a
hardened `-c` list; **keep it** rather than wiring `git_ui`, because it runs per-checkout during a
scan and `git_ui`'s client is heavier than that.

---

## 5. The port itself — what ports verbatim and what doesn't

### 5.1 The load-bearing finding: the painter is *already* stock GPUI

The whole treemap is **one `canvas()` with two closures**, plus `window.paint_quad`, plus
`window.text_system().shape_line(...).paint(...)` under `window.with_content_mask`. Every one of
those exists in **our** `crates/gpui`, verified:

| disktree call | ours |
|---|---|
| `gpui::canvas(prepaint, paint)` | `crates/gpui/src/elements/canvas.rs:10` |
| `gpui::quad(bounds, corners, fill, border, color, style)` | `crates/gpui/src/window.rs:7938` |
| `window.paint_quad(..)` | `crates/gpui/src/window.rs:4669` |
| `gpui::pattern_slash(color, w, interval)` — the reclaimable hatch | `crates/gpui/src/color.rs:827` |
| `gpui::transparent_black()` | `crates/gpui/src/color.rs:444` |
| `window.with_content_mask(Some(ContentMask{..}), f)` | `crates/gpui/src/window.rs:4148` |
| `window.text_system()` | `crates/gpui/src/window.rs:2473` |
| `ShapedLine::paint(origin, line_height, align, align_width, window, cx)` | `crates/gpui/src/text_system/line.rs:108` |
| `line.ascent` / `line.descent` (deref to `LineLayout`) | `pub ascent`, `pub descent`, `crates/gpui/src/text_system/line.rs` |
| **`Rems` / `rems()`** — the whole `ui.rs` scale system | `crates/gpui/src/geometry.rs:3250`, `:3735` |
| **`window.rem_size()` / `set_rem_size()`** | `crates/gpui/src/window.rs:3081`, `:3090` |
| `.debug_selector(..)` | `crates/gpui/src/elements/div.rs:885` |
| `.on_any_mouse_down(..)` | `crates/gpui/src/elements/div.rs:162` |
| `MouseButton::Navigate(NavigationDirection)` | `crates/gpui/src/interactive.rs:465` |
| `ScrollDelta::{Lines, Pixels}` | `crates/gpui/src/interactive.rs:554` |
| `ClickEvent::click_count()` | `crates/gpui/src/interactive.rs:424` |
| `window.pixel_snap_bounds(..)` | `crates/gpui/src/window.rs:3163` |

Two independent confirmations that this is a fork of the same codebase: `Rems` + `rem_size()` +
`debug_selector` are **not** upstream Zed API and they are in ours; and disktree's own comment
calls its dependency "the Zed GPUI fork".

⇒ `canvas.rs` ports **near-verbatim**. So does `tokens.rs`.

### 5.2 The only two things that do **not** port

1. **`gpui_omarchy` — 15 items.** `init`, `Theme`, `ActiveTheme::omarchy()`,
   `button`/`button_group`/`checkbox`/`ChoiceItem`/`ButtonVariant`/`Status`, `alert_dialog`,
   `dialog_popup`/`dialog_title`/`dialog_description`/`dialog_button`, `separator`,
   `with_tooltip`, `keycap`, `icon`/`IconName`.
   → our `ui` crate + `theme::ActiveTheme` + `ui::components::modal::Modal`
   (`crates/ui/src/components/modal.rs`).
2. **`gpui_kit`'s application shell.** `application()`, `WindowOptions`/`WindowBounds`/
   `TitlebarOptions`, `actions!`, `relative()`, `prompt_for_new_path`, `observe_window_appearance`,
   `base::{ThemeAppearance, CheckboxState}`, `#[gpui_kit::test]`.
   → our workspace/window services and `#[gpui::test]`.

**Two of these turn out to be 1:1, which is worth knowing before planning around them:**
`App::prompt_for_paths(PathPromptOptions) -> oneshot::Receiver<Result<Option<Vec<PathBuf>>>>` is
already ours (`crates/gpui/src/app.rs:1771`), and `App::write_clipboard(ClipboardItem)` likewise.

Everything else is ordinary GPUI: the `Rc<Cell<Point<Pixels>>>` paint→state channel
(`state.rs:367-368`), `View::project`/`unproject`, the `LayoutKey` cache, the scan-poller loop,
`discard(tree)` on a raw thread.

### 5.3 Theming: the one real port

`palette.rs` is 261 lines of **pure functions** with no GPUI element in it — the ideal seam. It reads
exactly **7** `gpui_omarchy::Theme` members: `inset`, `accent.h`, `warning`, `background`, `bright`,
`foreground`, `appearance`. `widgets.rs` adds `font`, `surface`, `border`, `danger`, `success`,
`secondary`.

Plan: a **~90-line `disk_tree/theme.rs`** with a `DiskTreeTokens` struct resolved from
`cx.theme()`, and `palette.rs`'s signatures change from `&gpui_omarchy::Theme` to `&DiskTreeTokens`.
**Every hue table, depth ladder, age ramp and mixing rule stays byte-identical.**

| disktree | ours |
|---|---|
| `inset` | `colors().panel_background` |
| `background` | `colors().background` |
| `surface` | `colors().surface_background` |
| `border` | `colors().border` |
| `bright` / `foreground` | `colors().text` / `colors().text_muted` |
| `secondary` | `colors().text_placeholder` |
| `accent` | `theme.accents().color_for_index(0)` (hue only) |
| `warning` / `danger` / `success` | `theme.status().warning` / `.error` / `.success` — all three exist (`crates/theme/src/styles/status.rs:36,68,78`) |
| `font` | the editor UI font, resolved as the rest of our panels do |

Keep `Colors::new(&tokens)` resolving 9 categories x 5 depths + 5 age buckets x 5 depths **once per
frame** (`treemap_view.rs:186-223`) — that is why the paint loop never touches the palette.

**`theme.accents()`** (`crates/theme/src/styles/accents.rs:13-30`) already ships 13 theme-aware,
user-overridable, modulo-cycling hues, used by `git_graph.rs` for branch lanes. Offer it as an
override when the user's theme defines `accents`; keep disktree's fixed hues as the default
because it has tests asserting they do not collide. **Do not silently swap one for the other.**

### 5.4 Reuse over reinvention

| need | reuse |
|---|---|
| file reads / writes / listing / metadata | `fs::Fs::{read_dir, metadata, write, canonicalize, open_sync}` — `crates/fs/src/fs.rs:144,145,148,150,137,141`. **Not** `std::fs`. |
| delete | `fs::Fs::{trash, remove_dir, remove_file}` — `crates/fs/src/fs.rs:129,124,134`, all taking `RemoveOptions`. **Not** `std::fs::remove_dir_all`. |
| folder picker | `cx.prompt_for_paths(PathPromptOptions { directories: true, .. })` — already ours, `crates/gpui/src/app.rs:1771` |
| clipboard | `cx.write_clipboard(ClipboardItem)` |
| root picker | `storage_roots::collect_storage_root_shortcuts()` (`storage_roots.rs:68`) |
| icons | `dx_icon(DxUiIcon::Storage)` — consistent with the strip |
| toasts | `notifications::status_toast::StatusToast::new(text, cx, ..)` — `crates/notifications/src/status_toast.rs:34`, `crates/agent_ui/src/agent_panel.rs:6453` |
| confirm-before-delete | `ui::components::modal::Modal` — `crates/ui/src/components/modal.rs` |

### 5.5 Deletion: vendor `removal.rs`, deliberately

Zed already has `Fs::trash`. We vendor `disk_tree_core/removal.rs` anyway, because it does four
things `Fs` does not: (1) NTFS MFT sizes when elevated (`mft.rs`), (2) elevation
(`access::restart_as_administrator`, `file_table_readable` — `access.rs`, 97 L), (3) refusals —
profile folders, media exceptions under stacked mount points (`removal.rs:231-541, 955-1088`),
(4) post-removal flush of the volume's free space. All four come with tests.

The concession: its macOS path uses `trash::macos::{DeleteMethod, TrashContextExtMacos}`
(`removal.rs:1372`) and `trash::TrashContext` (`:1380`), and our `trash` is the zed-industries fork
at 5.2.5, whose `crates/fs` only exercises `delete_with_info`/`restore_all`/`TrashItem`/`Error`.
→ risk **R1**.

**Deletion stays out of scope until the read-only treemap ships.** Steps 1-7 cannot lose data.

### 5.6 Keyboard / mouse contract

The dispatcher ladder (`state.rs:2524-2635`) *is* the behaviour. Port in this order: find-field →
screen → help overlay → power menu → breadcrumb menu → **platform-modifier passthrough** (`⌘`/`Super`
must return or we steal the window's chords) → volume picker → confirm-dialog.

Explore keys (`state.rs:2663-2752`): `/`/`s` find · `enter` descend · `h/j/k/l` + arrows ·
`backspace`/`u` ascend · `esc` unwind · `space`/`x` mark · `tab` sibling · `[`/`]` depth ·
`-`/`=`/`0` zoom · `t` cycle Size/Files/Age · `r` rescan · `g` goto storage root · `i` hidden ·
`d` apparent size · `o` reveal · `?` help. **Note `t` already collides with nothing here, but
`c`/`d`/`i`/`o`/`p`/`q`/`s`/`v`/`g` must be re-checked against the project panel's own bindings
before shipping** — the project panel has its own key handling.

**One intentional divergence:** disktree's zoom-to-cursor *descends* at `MAX_SCALE`
(`zoom_target`, `state.rs:1326-1337`). In a ~300 px side panel that fires constantly and feels
broken. Keep the ceiling, **drop the auto-descend**, make `enter` the only way in — and record it
in `CHANGELOG.md`.

Keep the `pointer_active` arbitration (`state.rs:1533-1549`): regression-tested
(`tests.rs:922-972`), and it is the difference between "feels right" and "jumps when the mouse
passes over it".

---

## 6. Wiring (exact edits)

| # | file | edit |
|---|---|---|
| 1 | `Cargo.toml` | add `"crates/disk_tree_core"` to `[workspace] members`; `disk_tree_core = { path = "crates/disk_tree_core" }` to `[workspace.dependencies]`. **`disk_tree_panel` is not a crate** — the UI lives in `project_panel`. |
| 2 | `Cargo.toml` | add `windows` features for `mft.rs` to `[workspace.dependencies]` (`Cargo.toml:982-1043`) — features go there, not per-crate |
| 3 | `crates/disk_tree_core/Cargo.toml` | `[lints] workspace = true`, `license = "GPL-3.0-or-later"`, **no `license-file`** (`script/check-licenses:13`) |
| 4 | `crates/disk_tree_core/LICENSE-GPL` | symlink → `../../LICENSE-GPL`, depth-exact (`script/check-licenses:33-56`) |
| 5 | `crates/disk_tree_core/LICENSE-MIT-disktree` | verbatim upstream MIT text + a `NOTICE` naming upstream commit `7c0661c` |
| 6 | `crates/project_panel/Cargo.toml` | add `disk_tree_core.workspace = true` |
| 7 | `crates/project_panel/src/project_panel.rs:1-8` | add `mod disk_tree;` |
| 8 | `crates/project_panel/src/project_panel.rs:326` | add `disk_tree_visible: bool` + init `false` beside `storage_details_visible` (`:1139`) |
| 9 | `crates/project_panel/src/project_panel.rs` | add `toggle_disk_tree_visible`, `storage_surface`, `render_disk_tree`; one `IconButton` in `render_dx_explorer_header` (`:5469-5571`) |
| 10 | `crates/project_panel/src/storage_roots_view.rs` | nothing required — reuse as-is |
| 11 | `crates/zed_actions/src/lib.rs:419-431` | add `project_panel::{ToggleDiskTree, ToggleStorageDetails}` |
| 12 | `crates/project_panel/src/project_panel.rs:711-720` | register the two actions in the existing `init` |
| 13 | `crates/settings_content/src/settings_content.rs` + `assets/settings/default.json` | `disk_tree: { button, root, include_hidden, apparent_size }` — **every** field must be in `default.json` or `Settings::from_settings` panics (`crates/settings/src/settings_store.rs:70-74`) |
| 14 | `crates/project_panel/src/project_panel_settings.rs` | `DiskTreeSettings` with `#[derive(RegisterSetting)]`, mirroring `OutlinePanelSettings` |
| 15 | `script/disk-tree-panel-source.test.ts` | **new** guard: the §1 module split, every file < 620 lines, and `disk_tree_core` imports no `gpui` |

`initialize_panels` (`crates/zed/src/zed.rs:788`) and `main.rs:767` are **untouched** — no new
panel, no new `Panel::activation_priority`, no dock entry, no status-bar button to add.

---

## 7. Risks

| id | risk | resolution |
|---|---|---|
| **R1** | `trash::macos::TrashContextExtMacos` may not exist in the zed-industries fork (5.2.5) | compile on macOS early; fallback is `Fs::trash` for the macOS backend only |
| **R2** | `windows-sys 0.61` → `windows 0.62` in `mft.rs` (1 713 L of FFI) | `cfg(not(windows))` first; `mft.rs` in a separate change |
| **R3** | our root `[workspace.lints]` denies `unsafe_code`; `mft.rs` is FFI-heavy | same as R2 |
| **R4** | the panel is ~300 px wide; disktree assumes a window | the Storage strip already works narrow; `[`/`]` depth + `-`/`=` zoom; consider a zoom-to-fit on first paint |
| **R5** | scanning `G:\` takes tens of seconds and eats a core | keep disktree's `ScanThreads` admission budget; port `examples/scan_workers.rs` as a `[[bench]]` (`project_panel` already has `benches/sorting.rs` + `criterion`) |
| **R6** | the treemap's key map collides with the project panel's | audit before shipping (§5.6) |
| **R7** | `storage_roots.rs` refreshes on a 60 s interval (`project_panel.rs:125`) — too slow to pick a new drive | the treemap reads the cached `Vec<StorageRootShortcut>` and adds a "refresh" affordance; do not add a second poller |
| **R8** | upstream sync drift — disktree `main` moved **56 commits in one week** | `disk_tree_core` stays byte-identical to upstream so a re-sync is a file copy; all divergence lives in `project_panel/src/disk_tree/` |

---

## 8. Task order

1. `crates/disk_tree_core` — copy all 16 files, align the 6 deps, license + symlink, `cargo check -p disk_tree_core`, run the copied unit tests. **No gpui anywhere.**
2. `disk_tree/tokens.rs` + `theme.rs` + `palette.rs` + `palette_tests.rs` — the smallest end-to-end slice that proves the seam (gpui `Rems` + our theme tokens). Port the property tests green.
3. `disk_tree/canvas.rs` + `model.rs` + `view.rs` + `layout.rs` — painter over a synthetic `tree::Node`. First `VisualTestContext` screenshot.
4. `disk_tree/navigation.rs` + `keys.rs` + `mouse.rs` + `breadcrumbs.rs` + `header.rs` + `legend.rs` — the §5.6 contract, with the `pointer_active` test ported.
5. `disk_tree/scan.rs` — wire `ScanHandle` + the poller; measure with disktree's own benchmark **before** touching the tick interval.
6. Wire it in: `disk_tree/mod.rs`, the `disk_tree_visible` field, `storage_surface()`, the header button, the two actions, settings. **Read-only from here.**
7. `script/disk-tree-panel-source.test.ts`; `CHANGELOG.md` entry listing every divergence from upstream.
8. **Only then** `marks.rs`, `git.rs`, `widgets.rs`, `overlays.rs`, and finally the review/delete screen.

**Stop after step 6 and show a screenshot before step 8.** Steps 1-6 cannot lose data; step 8 can.
