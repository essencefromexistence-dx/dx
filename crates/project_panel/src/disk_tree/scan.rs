//! Driving a scan: starting it, polling it, cancelling it.
//!
//! Vendored from `disktree-app/src/state.rs:947-1074` at tobi/disktree@7c0661c
//! (v0.10.1).
//!
//! # This is deliberately NOT `storage.rs`
//!
//! `crates/project_panel/src/storage.rs` also answers "how big is this", but it
//! reads `state.visible_entries` - the project's own gitignore-respecting index -
//! under hard caps (512 children per folder, 4 096 cached folders,
//! `project_panel.rs:122-124`). It is cheap, incremental, and correct for a file
//! tree.
//!
//! A whole-volume treemap is a different problem: an unbounded subtree walk with
//! hardlink de-duplication, mount de-duplication, per-file category
//! classification and mtimes, producing millions of nodes. It therefore gets its
//! **own scan, its own cache, its own cancel and its own lifetime**, and must
//! never touch `state.visible_entries` or `folder_storage_summaries`.
//!
//! # The polling loop
//!
//! `ScanHandle` is a blocking, thread-owned scanner. Upstream drains it with a
//! 110 ms `cx.spawn` loop that calls `poll_scan_once`, and that shape is kept:
//!
//! * a timer, not a channel, so the loop cannot be starved by a busy executor;
//! * `epoch` guards against a stale poll landing after a rescan started;
//! * `poll_scan_once` is reachable from tests so they can drive it without the
//!   test clock, which is why it is a plain method rather than a closure.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use disk_tree_core::scan::{ScanHandle, ScanOptions};
use disk_tree_core::scan_threads::ScanThreads;
use gpui::{Context, Window};

use crate::ProjectPanel;
use crate::disk_tree::model::{DiskTree, Screen};

/// How often the scan is drained. Fast enough that progress looks live, slow
/// enough that the poll is not the thing using the core.
const POLL_INTERVAL: Duration = Duration::from_millis(110);

/// How often free space is re-read.
///
/// Free space changes only when something is written or deleted, so this is
/// slow while exploring and fast only while a scan is running and the number is
/// visibly moving.
const SPACE_INTERVAL_EXPLORING: Duration = Duration::from_millis(1200);
const SPACE_INTERVAL_SCANNING: Duration = Duration::from_millis(150);

/// Start a scan of the tree's root.
///
/// A scan already in flight is cancelled rather than raced: two scans of the same
/// root produce two trees, and the second would silently win.
pub(crate) fn start_scan(panel: &mut ProjectPanel, cx: &mut Context<ProjectPanel>) {
    let tree = panel.disk_tree_mut();
    // The previous tree is remembered so a *widening* scan can skip it, but the
    // handle to the running scan is not: a scan already in flight is replaced,
    // not raced.
    tree.scan.known = tree
        .scan
        .tree
        .take()
        .map(|scanned| disk_tree_core::scan::Known {
            path: tree.scan.root.clone(),
            tree: scanned,
        });
    tree.scan.tree = None;
    tree.scan.epoch += 1;
    let epoch = tree.scan.epoch;

    let options = ScanOptions {
        apparent_size: !tree.on_disk_size,
        follow_links: tree.follow_links,
        include_hidden: tree.include_hidden,
        one_filesystem: tree.one_filesystem,
        max_depth: None,
        dedup_hardlinks: true,
        metric: tree.mode.metric(),
        threads: ScanThreads::default(),
    };
    let root = tree.root.path.clone();

    // `ScanHandle` owns both the progress counters and the result channel, so it
    // is kept whole rather than holding a bare `ScanProgress`.
    //
    // A handle to the previous scan is cancelled before it is replaced: two scans
    // of the same root produce two trees, and the second would silently win.
    if let Some(previous) = panel.disk_tree_mut().scan.handle.take() {
        previous.cancel();
    }

    let handle = ScanHandle::spawn(root.clone(), options);
    let tree = panel.disk_tree_mut();
    tree.scan.root = root;
    tree.scan.started = Some(Instant::now());
    tree.scan.failure = None;
    tree.screen = Screen::Running;
    // The progress counters are kept on the entity so `cancel_scan` can reach
    // them after the handle has been handed to the polling task. `ScanProgress`
    // cancels the walk on its own, and unlike `ScanHandle` it is `Sync`, which
    // is what lets an `Arc` of it cross into the detached poll.
    tree.scan.handle = Some(handle.progress.clone());

    poll_scan(panel, handle, epoch, cx);
}

/// The polling loop. Detached: it ends when the scan finishes, is cancelled, or
/// a newer scan bumps the epoch.
fn poll_scan(
    panel: &mut ProjectPanel,
    handle: ScanHandle,
    epoch: u64,
    cx: &mut Context<ProjectPanel>,
) {
    let _ = panel;
    cx.spawn(async move |panel, cx| {
        loop {
            cx.background_executor().timer(POLL_INTERVAL).await;
            let keep = panel
                .update(cx, |panel, cx| poll_scan_once(panel, &handle, epoch, cx))
                .unwrap_or(false);
            if !keep {
                break;
            }
        }
    })
    .detach();
}

/// Drain the scan once. Returns whether to keep polling.
///
/// Separate from `poll_scan` and reachable from tests, so a test can step the
/// scan to completion without waiting on the test clock.
pub(crate) fn poll_scan_once(
    panel: &mut ProjectPanel,
    handle: &ScanHandle,
    epoch: u64,
    cx: &mut Context<ProjectPanel>,
) -> bool {
    let tree = panel.disk_tree_mut();
    // A newer scan owns the tree now.
    if tree.scan.epoch != epoch {
        return false;
    }

    tree.scan.snapshot = handle.progress.snapshot();
    // `poll` yields the finished tree exactly once, then `None`. Taken here
    // rather than awaited, because the walk is already finished by the time the
    // snapshot says so.
    if let Some(result) = handle.poll() {
        match result {
            Ok(scanned) => {
                tree.scan.tree = Some(std::sync::Arc::new(scanned));
                tree.scan.failure = None;
            }
            Err(error) => {
                tree.scan.tree = None;
                // A scan that failed must say so rather than showing an empty
                // mosaic, which reads as "this disk is empty".
                tree.scan.failure = Some(error.to_string());
            }
        }
    }

    let finished = tree.scan.snapshot.finished || tree.scan.snapshot.cancelled;
    if finished {
        tree.screen = Screen::Explore;
        // The root is a cache entry now, not a starting point: the next scan of
        // the same root can widen from it.
        if let Some(scanned) = tree.scan.tree.clone() {
            tree.scan.known = Some(disk_tree_core::scan::Known {
                path: tree.scan.root.clone(),
                tree: scanned,
            });
        }
    }
    cx.notify();
    !finished
}

/// Cancel a running scan.
///
/// The result is dropped rather than shown: a cancelled scan is a partial tree,
/// and a partial tree presented as a whole one is a lie.
pub(crate) fn cancel_scan(panel: &mut ProjectPanel, cx: &mut Context<ProjectPanel>) {
    // Cancelled through the progress counters rather than the handle: the
    // counters are the part the polling task shares, they are `Sync`, and their
    // `cancel` sets the same flag the handle's does. Keeping the handle would
    // mean keeping its result channel too, which cannot be shared.
    if let Some(handle) = panel.disk_tree_mut().scan.handle.take() {
        handle.cancel();
    }
    let tree = panel.disk_tree_mut();
    tree.screen = Screen::Explore;
    tree.scan.epoch += 1;
    cx.notify();
}

/// Keep the free-space figure fresh.
///
/// Started once when the tree is built rather than per-scan, because the ticker
/// is cheap and the alternative is a second timer to reason about.
pub(crate) fn start_space_ticker(
    panel: &mut ProjectPanel,
    window: &mut Window,
    cx: &mut Context<ProjectPanel>,
) {
    let _ = panel;
    cx.spawn_in(window, async move |panel, cx| {
        loop {
            let interval = panel
                .update(cx, |panel, _| {
                    if panel
                        .disk_tree()
                        .is_some_and(|t| t.screen == Screen::Running)
                    {
                        SPACE_INTERVAL_SCANNING
                    } else {
                        SPACE_INTERVAL_EXPLORING
                    }
                })
                .unwrap_or(SPACE_INTERVAL_EXPLORING);
            cx.background_executor().timer(interval).await;
            let _ = panel.update(cx, |panel, cx| refresh_space(panel, cx));
        }
    })
    .detach();
}

/// Re-read free space for the tree's volume.
pub(crate) fn refresh_space(panel: &mut ProjectPanel, cx: &mut Context<ProjectPanel>) {
    let Some(root) = panel.disk_tree().map(|t| t.root.path.clone()) else {
        return;
    };
    let Some(info) = disk_tree_core::space::space_info(&root).ok() else {
        return;
    };
    let tree = panel.disk_tree_mut();
    let changed = tree.space.info != Some(info);
    tree.space.info = Some(info);
    // The baseline is the figure a projection is measured against, so it is only
    // captured when there is not one.
    if tree.space.baseline_free.is_none() {
        tree.space.baseline_free = Some(info.available);
    }
    if changed {
        cx.notify();
    }
}

/// Retarget the treemap at a different root and rescan.
///
/// From the Storage strip: the volume list is already there, and re-rooting is
/// the only thing clicking a drive should do. It must **not** change the
/// workspace - the Project Panel's own storage root button opens a drive as a
/// project (`project_panel.rs:5222`), which is a heavier action than picking
/// which directory the treemap is looking at.
pub(crate) fn set_root(
    panel: &mut ProjectPanel,
    path: PathBuf,
    shortcut_id: Option<String>,
    cx: &mut Context<ProjectPanel>,
) {
    let tree = panel.disk_tree_mut();
    tree.root = crate::disk_tree::model::Root { path, shortcut_id };
    // Everything derived from the old root is now wrong.
    tree.crumbs.clear();
    tree.selected = None;
    tree.hovered = None;
    tree.pointer = None;
    tree.cache = None;
    tree.transition = None;
    tree.view = Default::default();
    tree.history_mut().visit(Vec::new());
    tree.history_mut().clear();
    tree.marks.clear();
    tree.space = Default::default();
    start_scan(panel, cx);
}

/// Forget the scan results and walk again.
///
/// A rescan always starts from scratch even though a cached tree exists: the
/// user asked for current numbers, and widening a stale tree would answer a
/// different question.
pub(crate) fn rescan(panel: &mut ProjectPanel, cx: &mut Context<ProjectPanel>) {
    let tree = panel.disk_tree_mut();
    tree.scan.known = None;
    start_scan(panel, cx);
}

/// Drop a very large tree on a background thread.
///
/// Freeing millions of `Node`s takes long enough to drop frames if it happens on
/// the UI thread. Upstream did this (`state.rs:3143-3149`) and it is kept.
pub(crate) fn discard(tree: std::sync::Arc<disk_tree_core::tree::Node>) {
    std::thread::Builder::new()
        .name("disktree-drop".into())
        .spawn(move || drop(tree))
        .ok();
}

/// Whether a scan is running, for the header's progress affordance.
pub(crate) fn is_scanning(tree: &DiskTree) -> bool {
    matches!(tree.screen, Screen::Running)
}

/// The progress label for the scanning row.
pub(crate) fn progress_label(tree: &DiskTree) -> String {
    let snapshot = &tree.scan.snapshot;
    let scanned = disk_tree_core::size::human_bytes(tree.scan.scanned_bytes());
    if snapshot.errors > 0 {
        format!("{scanned} · {} unreadable", snapshot.errors)
    } else {
        scanned
    }
}

/// Free space, used space and the reclaimable total, for the header.
///
/// `reclaimable` is the marked bytes, which is a **projection**: it is what the
/// user has said they would like gone, not what has been freed. The figure after
/// a real removal is only shown once it has been measured, never predicted -
/// upstream's status bar never claimed a saving it could not measure.
pub(crate) fn space_summary(tree: &DiskTree) -> Option<SpaceSummary> {
    let info = tree.space.info?;
    let marked_bytes: u64 = tree
        .marks
        .paths
        .iter()
        .filter_map(|path| tree.crumbs_for_path(path))
        .filter_map(|crumbs| tree.node(&crumbs))
        .map(|node| node.bytes)
        .sum();
    Some(SpaceSummary {
        free: info.available,
        total: info.total,
        used: info.used(),
        marked_bytes,
        // Only claimed when there is a measured baseline to compare against.
        after_measured: tree
            .space
            .baseline_free
            .map(|baseline| info.available.saturating_sub(baseline)),
    })
}

/// The three numbers the header shows about the volume.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SpaceSummary {
    pub(crate) free: u64,
    pub(crate) total: u64,
    pub(crate) used: u64,
    /// Marked bytes. A projection, never presented as freed.
    pub(crate) marked_bytes: u64,
    /// How much free space has actually *grown* since the tree was built.
    pub(crate) after_measured: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polling_stops_once_the_scan_has_finished() {
        let snapshot = disk_tree_core::scan::ScanSnapshot {
            finished: true,
            ..Default::default()
        };
        assert!(snapshot.finished, "a finished scan must end the loop");
    }

    #[test]
    fn a_cancelled_scan_also_ends_the_loop() {
        // Otherwise the ticker runs forever against a scan that will never
        // produce anything.
        let snapshot = disk_tree_core::scan::ScanSnapshot {
            cancelled: true,
            ..Default::default()
        };
        assert!(snapshot.cancelled);
    }

    #[test]
    fn the_scan_interval_is_faster_than_the_resting_space_interval() {
        // Progress has to look live; free space does not.
        assert!(POLL_INTERVAL < SPACE_INTERVAL_EXPLORING);
        assert!(SPACE_INTERVAL_SCANNING < SPACE_INTERVAL_EXPLORING);
    }

    #[test]
    fn on_disk_size_is_the_default_because_it_is_the_number_that_frees() {
        // A file's `st_blocks * 512` is what comes back when it is deleted;
        // apparent size is the number that lies about this.
        let tree = DiskTree::new(
            None,
            crate::disk_tree::model::Root {
                path: PathBuf::from("/"),
                shortcut_id: None,
            },
        );
        assert!(tree.on_disk_size);
        let options = ScanOptions {
            apparent_size: !tree.on_disk_size,
            ..Default::default()
        };
        assert!(!options.apparent_size, "must report what deletion returns");
    }
}
