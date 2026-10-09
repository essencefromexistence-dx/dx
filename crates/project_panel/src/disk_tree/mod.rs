//! The disk tree: a treemap of what is actually filling a disk.
//!
//! Ported from [`tobi/disktree`](https://github.com/tobi/disktree) at commit
//! **`7c0661c`** (tag `v0.10.1`), MIT licensed - see
//! `crates/disk_tree_core/LICENSE-MIT-disktree`.
//!
//! # Why this lives in the Project Panel
//!
//! Because the Project Panel already owns this concept. `storage_roots.rs`
//! enumerates every mounted volume with its free/total bytes;
//! `storage.rs` has `StorageSortMode::{Size, FileCount, Modified}` - the same
//! three metrics the treemap offers - and a `storage_heat_level -> Color`
//! categorical ramp. A separate dock panel would have re-implemented all of that
//! and left two competing "storage" surfaces in one window.
//!
//! # What came from where
//!
//! | here | from | how close |
//! |---|---|---|
//! | the scan, the layout, the colours' *inputs* | `crates/disk_tree_core` | verbatim, MIT |
//! | `tokens.rs` | `disktree-app/src/ui.rs` | verbatim, `Rems` import repathed |
//! | `palette.rs` | `disktree-app/src/palette.rs` | every rule unchanged; only the theme type |
//! | `palette_tests.rs` | `disktree-app/src/palette.rs:176-261` | assertions unchanged |
//! | `theme.rs` | new | the adapter that replaces `gpui_omarchy::Theme` |
//! | `marks.rs` | new | the marked set, keyed by absolute path |
//! | `review.rs` | new | review screen plus the removal driver |
//! | `overlays.rs` | new | help, history, and removal-progress rows |
//! | `widgets.rs` | new | review rows and the permanent-delete modal |
//! | everything below this line | `disktree-app/src/{state,views}.rs` | task 3+ |
//!
//! # The one rule
//!
//! **Adaptations live here, never in `disk_tree_core`.** The correctness half -
//! size accounting, hardlink de-duplication, aspect-ratio layout, what may be
//! deleted - is vendored byte-identical and guarded by SHA-256 in
//! `script/disk-tree-core-vendored.test.ts`. Re-syncing upstream is a file copy
//! only for as long as that stays true.

#![allow(dead_code)]

pub(crate) mod canvas;
pub(crate) mod marks;
pub(crate) mod model;
#[cfg(test)]
mod model_tests;
pub(crate) mod mouse;
pub(crate) mod moves;
pub(crate) mod navigation;
pub(crate) mod overlays;
pub(crate) mod palette;
#[cfg(test)]
mod palette_tests;
pub(crate) mod review;
pub(crate) mod scan;
pub(crate) mod theme;
pub(crate) mod tokens;
pub(crate) mod view;
pub(crate) mod widgets;
