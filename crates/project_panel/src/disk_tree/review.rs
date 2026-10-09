//! The review screen: the only path in this panel that can lose data.
//!
//! Upstream's `review` step (`disktree-app/src/views.rs:3228-3596`) is the
//! safety gate for removal, and it is reproduced here in substance: a plan is
//! built from the marked paths, everything the plan refuses is shown rather than
//! hidden, and **nothing is removed until the user has confirmed a mode**.
//!
//! # What actually removes bytes
//!
//! [`disk_tree_core::removal::spawn`], on its own worker thread. It applies the
//! four refusals the vendored core has - a mount point, an ancestor of the
//! scanned root, the user's home directory, and a path outside the root - and
//! reports each one as a [`Blocked`] row instead of touching it. That logic is
//! vendored byte-identical and guarded by SHA-256, so re-implementing any part of
//! it here would only be a way to get it wrong.
//!
//! `std::fs::remove_dir_all` is never used: it has no trash, no confirmation, and
//! no notion of which paths the user actually consented to.
//!
//! # Why the mode has to be chosen before the plan is built
//!
//! [`RemovalMode::Permanent`] is not recoverable. Defaulting the screen to it
//! would make "I pressed the review key" equivalent to "I asked for this to be
//! deleted for ever", which is not a reasonable reading of any gesture. So the
//! default is [`RemovalMode::Trash`], and [`start_removal`] refuses to run a
//! permanent removal unless [`ReviewState::confirmed`] was set by
//! [`confirm`].
//!
//! [`Blocked`]: disk_tree_core::removal::Blocked
//! [`confirm`]: confirm

use std::time::Duration;

use disk_tree_core::removal::{self, RemovalEvent, RemovalMode, Target};
use disk_tree_core::tree::{Metric, Node};
use gpui::{App, Context};
use notifications::status_toast::StatusToast;

use crate::ProjectPanel;

use super::model::{DiskTree, Screen};
use super::{overlays, widgets};

/// Build the review plan from the current marks and show the review screen.
///
/// This does not remove anything. It is safe to call from any key handler.
pub(crate) fn open(panel: &mut ProjectPanel, cx: &mut Context<ProjectPanel>) {
    let tree = panel.disk_tree_mut();
    let targets = marked_targets(tree);
    let plan = removal::plan(&targets, tree.root.path.as_path());
    tree.review.plan = Some(plan);
    // Every visit starts on the recoverable mode. A previous permanent choice
    // must not survive into a new review.
    tree.review.mode = RemovalMode::Trash;
    tree.review.confirmed = false;
    tree.review.removed = 0;
    tree.review.freed = 0;
    tree.review.failed = 0;
    tree.review.done = false;
    tree.screen = Screen::Review;
    cx.notify();
}

/// The marked paths, as removal targets.
///
/// A marked path that is no longer in the tree (a rescan dropped it, or it was
/// deleted underneath us) is skipped rather than planned: there is nothing to
/// remove and reporting it would be noise. Its bytes are still worth knowing, so
/// the projection stays honest by measuring what we actually have.
fn marked_targets(tree: &DiskTree) -> Vec<Target> {
    let mut targets = Vec::new();
    let Some(scanned) = tree.tree() else {
        return targets;
    };
    for path in tree.marks.paths.iter() {
        let Some(crumbs) = crumbs_for_path(tree.root.path.as_path(), scanned, path) else {
            // The mark no longer names anything in the tree. A rescan dropped
            // it, or it was deleted underneath us. There is nothing to remove,
            // so planning it would only produce a target that fails at run time.
            continue;
        };
        let Some(node) = node_at(scanned, &crumbs) else {
            continue;
        };
        targets.push(Target {
            path: path.clone(),
            bytes: metric_bytes(tree, node),
            is_dir: node.kind.is_dir(),
            hidden: is_hidden_path(path),
        });
    }
    targets.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.path.cmp(&b.path)));
    targets
}

fn node_at<'a>(root: &'a Node, crumbs: &[usize]) -> Option<&'a Node> {
    let mut node = root;
    for &index in crumbs {
        node = node.children.get(index)?;
    }
    Some(node)
}

/// Whether the last component begins with a dot.
fn is_hidden_path(path: &std::path::Path) -> bool {
    path.file_name()
        .map(|name| name.to_string_lossy().starts_with('.'))
        .unwrap_or(false)
}

/// Pick how the removal happens. Choosing a mode is not consent; [`confirm`] is.
pub(crate) fn set_mode(
    panel: &mut ProjectPanel,
    mode: RemovalMode,
    cx: &mut Context<ProjectPanel>,
) {
    let tree = panel.disk_tree_mut();
    // Changing the mode invalidates any previous confirmation: the user has not
    // seen *this* mode's warning yet.
    tree.review.mode = mode;
    tree.review.confirmed = false;
    cx.notify();
}

/// The user has read the plan and chosen to go ahead.
///
/// Split from [`start_removal`] so the confirm gesture and the run gesture can
/// never be the same keypress.
pub(crate) fn confirm(panel: &mut ProjectPanel, cx: &mut Context<ProjectPanel>) {
    let tree = panel.disk_tree_mut();
    tree.review.confirmed = true;
    cx.notify();
}

/// Go back to the treemap, abandoning the review.
pub(crate) fn cancel(panel: &mut ProjectPanel, cx: &mut Context<ProjectPanel>) {
    let tree = panel.disk_tree_mut();
    // A running removal is *not* cancelled here: the worker is mid-flight and
    // abandoning the screen must not silently leave a half-removed tree behind
    // with nobody watching. Cancel is explicit, via [`cancel_running`].
    tree.review.plan = None;
    tree.review.confirmed = false;
    tree.screen = Screen::Explore;
    cx.notify();
}

/// Ask the worker to stop before its next target.
///
/// Targets already removed stay removed; `removal::spawn` checks the flag between
/// targets, so this is a clean boundary rather than a partial write.
pub(crate) fn cancel_running(panel: &mut ProjectPanel, cx: &mut Context<ProjectPanel>) {
    if let Some(handle) = panel.disk_tree_mut().review.running.take() {
        handle.cancel();
    }
    let tree = panel.disk_tree_mut();
    tree.screen = Screen::Explore;
    cx.notify();
}

/// Start the removal the review screen describes.
///
/// Refuses unless the plan exists, is non-empty, and the user confirmed. The
/// refusals themselves live in the vendored core; this only refuses to *call* it.
pub(crate) fn start_removal(panel: &mut ProjectPanel, cx: &mut Context<ProjectPanel>) {
    let tree = panel.disk_tree_mut();
    if !tree.review.confirmed {
        return;
    }
    let Some(plan) = tree.review.plan.take() else {
        return;
    };
    if plan.is_empty() {
        // Everything the user marked was refused. Nothing to do, and saying so
        // by returning to the treemap is better than showing an empty progress
        // bar that finishes instantly.
        tree.review.done = true;
        tree.screen = Screen::Explore;
        cx.notify();
        return;
    }

    let mode = tree.review.mode;
    tree.review.removed = 0;
    tree.review.freed = 0;
    tree.review.failed = 0;
    tree.review.done = false;
    tree.screen = Screen::Removing;

    let handle = removal::spawn(plan, mode);
    tree.review.running = Some(handle);
    poll_removal(panel, cx);
}

/// Drain the worker's available events into the review state.
///
/// Everything already in the channel is applied this tick, then the ticker
/// re-arms itself while a worker is still running.
fn poll_removal(panel: &mut ProjectPanel, cx: &mut Context<ProjectPanel>) {
    let mut finished = false;
    let mut repaint = false;
    loop {
        // Take one event at a time without holding the tree borrow across the
        // mutation below: the handle is borrowed only for the `poll` call.
        let event = panel
            .disk_tree()
            .and_then(|tree| tree.review.running.as_ref())
            .and_then(|handle| handle.poll());
        let Some(event) = event else {
            break;
        };
        repaint = true;
        match event {
            RemovalEvent::Start { .. } => {}
            RemovalEvent::Item { bytes, outcome, .. } => {
                let tree = panel.disk_tree_mut();
                tree.review.removed += 1;
                match outcome {
                    Ok(()) => tree.review.freed += bytes,
                    Err(_) => tree.review.failed += 1,
                }
            }
            RemovalEvent::Done { .. } => finished = true,
        }
    }

    if finished {
        let tree = panel.disk_tree_mut();
        tree.review.running = None;
        tree.review.done = true;
        // The marks described paths that no longer exist. Leaving them would
        // make the next review plan target paths nothing on disk can find.
        tree.marks.clear();
        repaint = true;

        let (removed, freed, failed) = (tree.review.removed, tree.review.freed, tree.review.failed);
        announce(panel, removed, freed, failed, cx);
    }

    if repaint {
        cx.notify();
    }

    // Re-arm only while there is still a worker. When it is gone the Done branch
    // above has already run and this returns for good.
    let still_running = panel
        .disk_tree()
        .map(|tree| tree.review.running.is_some())
        .unwrap_or(false);
    if still_running {
        let task = cx.spawn(async move |panel, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(REMOVAL_POLL_MS))
                .await;
            let _ = panel.update(cx, |panel, cx| {
                if panel
                    .disk_tree()
                    .map(|tree| tree.review.running.is_some())
                    .unwrap_or(false)
                {
                    poll_removal(panel, cx);
                }
            });
        });
        task.detach();
    }
}

/// How often a running removal is drained, in milliseconds.
///
/// Slower than the scanner's poll because removal is a handful of large
/// operations rather than a continuous stream.
const REMOVAL_POLL_MS: u64 = 110;

/// Render the review screen.
///
/// Rows, not a dialog: the point of the screen is that every refused path is
/// visible, and a modal that truncates its list hides exactly the thing the
/// user needs to see.
pub(crate) fn render(
    tree: &DiskTree,
    tokens: super::theme::DiskTreeTokens,
    _cx: &mut App,
) -> impl gpui::IntoElement {
    use gpui::prelude::*;

    let _ = tokens;
    if tree.screen == Screen::Removing {
        return overlays::removal_progress(
            tree.review.removed,
            tree.review.freed,
            tree.review.failed,
        );
    }

    let Some(plan) = tree.review.plan.as_ref() else {
        return gpui::div().into_any_element();
    };
    let mode = tree.review.mode;

    let summary = gpui::div()
        .flex()
        .flex_col()
        .child(format!("{} selected", plan.targets.len()))
        .child(format!(
            "{} to be freed",
            widgets::human_bytes(plan.bytes())
        ))
        .child(format!("{} mode: {}", mode.label(), mode.detail()))
        .into_any_element();
    let rows = plan
        .targets
        .iter()
        .map(|target| widgets::path_row(&target.path, widgets::human_bytes(target.bytes), false))
        .collect::<Vec<_>>();
    let covered = plan
        .covered
        .iter()
        .map(|target| {
            widgets::path_row(
                &target.path,
                "covered by a selection above".to_owned(),
                true,
            )
        })
        .collect::<Vec<_>>();
    let blocked = plan
        .blocked
        .iter()
        .map(|blocked| widgets::warning_row(blocked.reason.clone(), &blocked.path))
        .collect::<Vec<_>>();

    // `.id(..)` is required, not decorative: the overflow methods live on
    // `StatefulInteractiveElement`, which is implemented for `Stateful<E>`
    // (crates/gpui/src/elements/div.rs:4100) and not for a bare `Div`. Giving the
    // container an id is how it becomes `Stateful<Div>`.
    let mut body = gpui::div()
        .flex()
        .flex_col()
        .id("dx-explorer-disk-tree-review-scroll")
        .overflow_y_scroll()
        .child(widgets::section("Review", summary))
        .child(overlays::history_card(
            tree.history.at,
            tree.history.entries.len(),
        ))
        .child(
            gpui::div()
                .flex()
                .flex_col()
                .children(rows)
                .children(covered)
                .children(blocked),
        )
        .child(overlays::review_help());
    if mode == RemovalMode::Permanent {
        body = body.child(widgets::permanent_confirm_modal(tree.review.confirmed));
    }
    body.into_any_element()
}

/// Tell the user what the worker did, once, in the status area.
///
/// A removal that finishes silently is indistinguishable from a removal that
/// never ran. The toast carries the same three numbers the progress rows
/// showed, and nothing else: the plan itself is gone by now.
fn announce(
    panel: &mut ProjectPanel,
    removed: u64,
    freed: u64,
    failed: usize,
    cx: &mut Context<ProjectPanel>,
) {
    let message = if failed == 0 {
        format!(
            "Removed {removed} paths, {} freed.",
            widgets::human_bytes(freed)
        )
    } else {
        format!(
            "Removed {removed} paths, {} freed, {failed} failed.",
            widgets::human_bytes(freed)
        )
    };
    let toast = StatusToast::new(message, cx, |this, _| this.dismiss_button(true));
    let _ = panel.workspace.update(cx, |workspace, cx| {
        workspace.toggle_status_toast(toast, cx);
    });
}

/// The same numbers the screen shows, for a toast once the removal is done.
pub(crate) fn summary_of(tree: &DiskTree) -> (usize, u64, usize) {
    (
        tree.review.removed as usize,
        tree.review.freed,
        tree.review.failed,
    )
}

/// Bytes a node contributes to the tree, honouring the metric in use.
///
/// The removal plan's projection has to use the same number the treemap was
/// drawing, or "this will free N" disagrees with the tile the user clicked.
fn metric_bytes(tree: &DiskTree, node: &Node) -> u64 {
    match tree.metric() {
        Metric::Bytes => node.bytes,
        Metric::Files => node.files,
    }
}

/// The crumbs for an absolute path under the scanned root.
///
/// The treemap addresses nodes by index path, and marks are keyed by absolute
/// path so they survive a rescan. That means the review screen has to turn one
/// back into the other, which is a walk down the root's children matching names
/// one component at a time - `Path::components`, not string prefixes, so
/// `C:\a` never matches `C:\ab`.
fn crumbs_for_path(
    root_path: &std::path::Path,
    root: &Node,
    path: &std::path::Path,
) -> Option<Vec<usize>> {
    let rest = path.strip_prefix(root_path).ok()?;
    let mut node = root;
    let mut crumbs = Vec::new();
    for component in rest.components() {
        let name = component.as_os_str().to_string_lossy();
        let index = node
            .children
            .iter()
            .position(|child| child.name.as_ref() == name.as_ref())?;
        crumbs.push(index);
        node = &node.children[index];
    }
    Some(crumbs)
}
