//! `disk_tree_core`: scanning, layout, space accounting and deletion, with no
//! UI dependency.
//!
//! The split exists so the parts that must be *correct* — size accounting,
//! hardlink de-duplication, aspect-ratio layout, and what may be deleted — can
//! be built and tested without GPUI, a display, or a GPU.
//!
//! # Provenance
//!
//! Vendored verbatim from [`tobi/disktree`](https://github.com/tobi/disktree)
//! `crates/disktree-core`, at commit **`7c0661c`** (tag `v0.10.1`), MIT licensed,
//! `Copyright (c) 2026 Tobi Lütke` — see `LICENSE-MIT-disktree`. That licence
//! requires the notice to survive redistribution, which is why it is carried
//! here unmodified.
//!
//! **Every other file in this directory is byte-identical to upstream**, and that
//! is enforced: `script/disk-tree-core-vendored.test.ts` pins the SHA-256 of all
//! fourteen. The only edits made at vendoring time were to this file (this
//! header, plus the one lint allow below) and to `Cargo.toml`; see that file for
//! the dependency alignment and the two lint deviations, both deliberate.
//!
//! Do not refactor these files, and do not run `cargo fmt` over them: upstream
//! formats with its own rustfmt settings, and reformatting here would break the
//! byte-identity that makes re-syncing upstream a file copy. If `cargo fmt --check`
//! reports a diff in this crate, that diff is wrong.
//!
//! Do not refactor these files. They are the *correctness* half of the disk tree
//! — size accounting, hardlink de-duplication, aspect-ratio layout, and what may
//! be deleted — and keeping them byte-identical is what makes re-syncing upstream
//! a file copy instead of a merge. Local changes live in
//! `crates/project_panel/src/disk_tree/`, which owns every adaptation to our
//! GPUI and our theme.
//!
//! Upstream's `disktree-core` doc comment is quoted above rather than rewritten;
//! only the crate name changed.
//!
//! # Why one crate-level lint allow
//!
//! `clippy.toml:9-14` disallows `std::process::Command::{spawn, output, status,
//! stdin, stdout, stderr}` because they "can block the current thread for an
//! unknown duration", and `Cargo.toml:1253` turns that into a deny.
//!
//! disktree uses it in six places - `removal.rs:677` `/sbin/mount`,
//! `removal.rs:1416` the elevated re-exec, `removal.rs:1666` and
//! `scan.rs:1513` (tests), `space.rs:647` `mount`, `windows.rs:904` `cmd`.
//! All six are short, bounded system calls that this crate only ever makes from a
//! background removal thread or from a test - never from a UI thread. That is the
//! situation the lint exists to prevent, so the deny does not apply here.
//!
//! It is allowed once, here, rather than as seven `#[allow]` attributes spread
//! over four vendored files: one line is cheaper to carry across an upstream
//! re-sync, and it cannot rot into a second un-audited call site the way a
//! per-site attribute can.

#![allow(clippy::disallowed_methods)]

pub mod access;
pub mod classify;
pub mod export;
pub mod filter;
pub mod insights;
#[cfg(windows)]
mod mft;
pub mod removal;
pub mod scan;
pub mod scan_threads;
pub mod size;
pub mod space;
pub mod tree;
pub mod treemap;
#[cfg(windows)]
mod windows;
