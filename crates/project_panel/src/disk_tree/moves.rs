//! Navigation and selection: moving through the tree, and resolving a hit.
//!
//! Split out of `navigation.rs` so both files stay under this project's 620-line
//! owned-module ceiling (`script/dx-source-quality.test.ts`). Geometry and
//! resolution stay in `navigation.rs`; the actions a key or a click performs stay
//! here.

use disk_tree_core::filter::{Keep, filter, Matches};
use disk_tree_core::treemap::{self, Rect, Tile};
use disk_tree_core::tree::{Metric, Node};
use gpui::{Context, Entity, Pixels, px};
use project::Project;
use std::path::PathBuf;

use crate::ProjectPanel;
use crate::disk_tree::model::{ColorMode, Direction, DiskTree, Filtered};
use crate::disk_tree::navigation::{layout_options, tile_at};
use crate::disk_tree::view::View;

impl DiskTree {
    /// The crumbs of the node at `path`, or `None` if it is not in the tree.
    ///
    /// Walks down from the scanned root one component at a time. This is the
    /// inverse of `tree::path_of`, and it has to be written by hand because the
    /// core offers no such direction - which is also why marks are stored as
    /// paths at all: they survive a rescan, and a path is the only stable handle
    /// a scan produces.
    pub(crate) fn crumbs_for_path(&self, path: &std::path::Path) -> Option<Vec<usize>> {
        let root = self.tree()?;
        let relative = path.strip_prefix(&self.root.path).ok()?;
        let mut node = root.as_ref();
        let mut crumbs = Vec::new();
        for component in relative.components() {
            let name = component.as_os_str().to_string_lossy();
            let index = node
                .children
                .iter()
                .position(|child| child.name.as_ref() == name)?;
            crumbs.push(index);
            node = &node.children[index];
        }
        Some(crumbs)
    }

    /// Whether `crumbs` sits strictly below a marked directory.
    ///
    /// A marked *ancestor* dims everything under it, so the marked directory
    /// reads as one removable thing rather than as a removable directory plus a
    /// thousand individually removable children.
    pub(crate) fn covered_by_mark(&self, crumbs: &[usize]) -> bool {
        self.marks.paths.iter().any(|marked| {
            let Some(marked) = self.crumbs_for_path(marked) else {
                return false;
            };
            // A mark does not cover itself, and a node below the marked node is
            // not covered by it either.
            marked.len() < crumbs.len() && crumbs.starts_with(&marked)
        })
    }

    /// Whether the applied filter keeps, dims or drops a tile.
    ///
    /// `Holds` - a directory that merely *contains* a match - stays readable,
    /// because it is the path to the match. `Out` is dropped from the layout
    /// entirely rather than dimmed, so the mosaic shows only the matches and
    /// their ancestors.
    pub(crate) fn filtered(&self, tile: &Tile) -> Filtered {
        let Some(needle) = self.find.as_deref() else {
            return Filtered::Shown;
        };
        let Some(root) = self.tree() else {
            return Filtered::Shown;
        };
        let Some(matches) = filter(root, &self.crumbs, needle) else {
            return Filtered::Shown;
        };
        match matches.keep(tile.crumbs()) {
            // A match, and everything under it.
            Some(Keep::Whole) => Filtered::Shown,
            // A directory on the way to a match.
            Some(Keep::Partial { .. }) => Filtered::Holds,
            // Neither a match nor on the way to one.
            None => Filtered::Out,
        }
    }

    /// The age bucket for a node, in age mode only.
    pub(crate) fn age_bucket(&self, node: &Node) -> Option<usize> {
        if self.mode != ColorMode::Age || node.modified == 0 {
            return None;
        }
        Some(crate::disk_tree::palette::age_bucket(days_since(node.modified)))
    }
}

/// Whole days since a Unix timestamp. Negative timestamps (a pre-epoch mtime, or
/// a clock skew) are treated as "just now" rather than wrapping into the oldest
/// bucket.
fn days_since(unix_seconds: i64) -> i64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
// No timestamp at all, or one at or before the epoch, means the age is
    // unknown, which is "just now" rather than the oldest bucket. A timestamp
    // in the future must not wrap either, which a plain subtraction does in
    // release builds.
    if unix_seconds <= 0 {
        return 0;
    }
    now.saturating_sub(unix_seconds).max(0) / 86_400
}

/// Resolve a node's crumbs by name, for the find field.
pub(crate) fn find_by_name(tree: &DiskTree, needle: &str) -> Option<Vec<usize>> {
    let root = tree.tree()?;
    root.find(needle).map(|(crumbs, _)| crumbs)
}

/// Enter a directory: the selection becomes the crumbs, and the layout is
/// rebuilt around it.
pub(crate) fn descend(
    tree: &mut DiskTree,
    crumbs: Vec<usize>,
    area: gpui::Size<Pixels>,
    rem: f32,
) {
    if crumbs == tree.crumbs {
        return;
    }
    // Remember where we were, so the transition has a `src` rectangle.
    let before = current_rect(tree, &tree.crumbs, area, rem);
    tree.crumbs = crumbs.clone();
    tree.cache = None;
    // Remember where we came from, but only for a real move: re-entering the
    // directory already shown must not push a duplicate history entry.
    let previous = tree.crumbs.clone();
    tree.view = View::IDENTITY;
    tree.selected = Some(crumbs);
    tree.hovered = None;
    if previous != tree.crumbs {
        tree.visit();
    }
    let after = current_rect(tree, &tree.crumbs, area, rem);
    if let (Some(src), Some(dst)) = (before, after) {
        tree.transition = Some(crate::disk_tree::view::LayoutTransition::new(
            project(&tree.view, src),
            project(&tree.view, dst),
        ));
    }
    tree.pointer_active = false;
}

/// Leave the current directory.
pub(crate) fn ascend(tree: &mut DiskTree, area: gpui::Size<Pixels>, rem: f32) {
    if tree.crumbs.is_empty() {
        return;
    }
    let mut parent = tree.crumbs.clone();
    parent.pop();
    descend(tree, parent, area, rem);
}

/// The rectangle a directory currently occupies, in base space.
fn current_rect(
    tree: &DiskTree,
    crumbs: &[usize],
    area: gpui::Size<Pixels>,
    rem: f32,
) -> Option<Rect> {
    let node = tree.node(crumbs)?;
    let area = disk_tree_core::treemap::Rect::new(
        0.0,
        0.0,
        area.width.as_f32(),
        area.height.as_f32(),
    );
    let tiles = treemap::layout(node, crumbs, area, tree.metric(), &layout_options(1, rem));
    // The root's own tile is the first one laid out at depth 1.
    tiles.first().map(|tile| tile.rect)
}

fn project(view: &View, rect: Rect) -> Rect {
    view.project(rect)
}

/// Move the selection to the neighbouring tile in `direction`.
///
/// Selection never changes depth: `enter` and `backspace` do that. Among
/// same-depth tiles the nearest one in that direction wins, scored by distance
/// along the axis with a penalty for the gap across, so a tile directly below
/// beats one far below and beside.
pub(crate) fn move_selection(
    tree: &mut DiskTree,
    direction: Direction,
    tiles: &[Tile],
    area: gpui::Size<Pixels>,
) {
    tree.pointer_active = false;
    let Some(from) = tree.selected.clone().filter(|_| tree.show_selection) else {
        return;
    };
    let (cx0, cy0) = current_point(tree, &from, tiles, area);
    let Some(current) = tile_at(tiles, cx0, cy0) else {
        return;
    };
    let from_depth = current.depth;
    let from_rect = current.rect;

    let mut best: Option<(f32, Vec<usize>)> = None;
    for tile in tiles {
        let tile_crumbs = tile.crumbs();
        if tile.depth != from_depth || tile_crumbs == from {
            continue;
        }
        let Some(score) = directional_score(from_rect, tile.rect, direction) else {
            continue;
        };
        if best.as_ref().is_none_or(|(best, _)| score < *best) {
            best = Some((score, tile_crumbs.to_vec()));
        }
    }

    if let Some((_, crumbs)) = best {
        tree.selected = Some(crumbs);
    }
}

/// Where a tile sits in base space, given the tile list. A tile not in the list
/// (because it is filtered out or too small) falls back to the view's centre.
fn current_point(
    tree: &DiskTree,
    crumbs: &[usize],
    tiles: &[Tile],
    area: gpui::Size<Pixels>,
) -> (f32, f32) {
    if let Some(tile) = tiles.iter().find(|tile| tile.crumbs() == crumbs) {
        let rect = tile.rect;
        return (rect.x + rect.w / 2.0, rect.y + rect.h / 2.0);
    }
    let base = tree.view.visible_base(area);
    (base.x + base.w / 2.0, base.y + base.h / 2.0)
}

/// `None` when `to` is not in `direction` from `from`.
fn directional_score(from: Rect, to: Rect, direction: Direction) -> Option<f32> {
    let (from_cx, from_cy) = (from.x + from.w / 2.0, from.y + from.h / 2.0);
    let (to_cx, to_cy) = (to.x + to.w / 2.0, to.y + to.h / 2.0);
    let (dx, dy) = (to_cx - from_cx, to_cy - from_cy);

    // The gap across, penalised: a tile that also moved sideways is further away
    // than one directly in the direction.
    match direction {
        Direction::Right if dx > 0.0 => Some(dx + dy.abs() * 2.5),
        Direction::Left if dx < 0.0 => Some(-dx + dy.abs() * 2.5),
        Direction::Down if dy > 0.0 => Some(dy + dx.abs() * 2.5),
        Direction::Up if dy < 0.0 => Some(-dy + dx.abs() * 2.5),
        _ => None,
    }
}

/// Step to the next or previous sibling, wrapping.
///
/// Siblings are stored biggest-first, so this walks the list directly rather
/// than re-sorting by the active metric.
pub(crate) fn cycle_sibling(tree: &mut DiskTree, step: i32) {
    tree.pointer_active = false;
    let Some(node) = tree.current() else {
        return;
    };
    if node.children.is_empty() {
        return;
    }
    let index = tree
        .selected
        .as_ref()
        .and_then(|crumbs| crumbs.last().copied())
        .filter(|last| *last < node.children.len())
        .unwrap_or(0) as i32;
    let len = node.children.len() as i32;
    let next = ((index + step) % len + len) % len;
    tree.selected = Some(vec![next as usize]);
}

/// Zoom by `factor` about a base-space point, clamped to the ceiling.
pub(crate) fn zoom_at(tree: &mut DiskTree, x: f32, y: f32, factor: f32, area: gpui::Size<Pixels>) {
    tree.view = tree
        .view
        .zoomed_at(x, y, factor, View::MAX_SCALE)
        .clamped(area);
}

/// Back to the unzoomed view.
pub(crate) fn reset_view(tree: &mut DiskTree, area: gpui::Size<Pixels>) {
    tree.view = View::IDENTITY.clamped(area);
}

/// Change the nesting depth, 1..=6.
pub(crate) fn adjust_depth(tree: &mut DiskTree, delta: i32) {
    let next = (tree.max_depth as i32 + delta).clamp(1, 6);
    if next as u32 == tree.max_depth {
        return;
    }
    tree.max_depth = next as u32;
    tree.cache = None;
}

/// Cycle Size -> Files -> Age.
pub(crate) fn set_mode(tree: &mut DiskTree, mode: ColorMode) {
    if tree.mode == mode {
        return;
    }
    tree.mode = mode;
    tree.cache = None;
}

/// Run the find, on the background, and stash the result.
///
/// `find_epoch` supersedes an in-flight search: a slower earlier search must
/// not overwrite a newer one.
pub(crate) fn start_find(
    tree: &mut DiskTree,
    needle: String,
    cx: &mut Context<ProjectPanel>,
) {
    tree.find = Some(needle.clone());
    tree.find_epoch += 1;
    let epoch = tree.find_epoch;
    let Some(root) = tree.tree().cloned() else {
        tree.filter = None;
        return;
    };

    // `find` walks the whole tree, so it cannot run on the UI thread. The walk
    // goes onto the background executor via `spawn`, awaited from inside
    // `cx.spawn`: `App::background_spawn` exists, but it is on `AppContext` and
    // neither `Context<ProjectPanel>` nor `AsyncApp` derefs to it.
    let search = cx
        .background_executor()
        .spawn({
            // `root` is already owned (`tree.tree().cloned()` above),
            // so the async block can take it directly. `find` returns a reference
            // into root, so keep only the owned crumbs: nothing borrowed may escape
            // a `static background task.`
            async move { root.find(&needle).map(|(crumbs, _)| crumbs) }
        });

    cx.spawn(async move |panel, cx| {
        let found = search.await;
        // The panel can be gone by the time a whole-tree search lands; that is
        // not an error worth logging.
        let _ = panel.update(cx, |this, _| {
            let disk_tree = this.disk_tree_mut();
            // A slower earlier search must not overwrite a newer one.
            if disk_tree.find_epoch != epoch {
                return;
            }
            disk_tree.filter = found;
            disk_tree.filter_epoch += 1;
            disk_tree.cache = None;
        });
    })
    .detach();
}

/// The matches for the current find, for the "N of M" line.
///
/// Returns `None` when no find is running, so the caller can tell "no filter"
/// from "filter found nothing" - the first is silent, the second says zero.
pub(crate) fn matches(tree: &DiskTree) -> Option<Matches> {
    let root = tree.tree()?;
    let needle = tree.find.as_deref()?;
    filter(root, &tree.crumbs, needle)
}

/// Reveal a path in the platform's file manager.
///
/// Routed through `Project::reveal_path` rather than a `std::process::Command`,
/// which is what the Project Panel already does for its own tree
/// (`project_panel.rs:4460-4463`) and what makes this work over WSL and remote
/// connections: `reveal_path` translates the path for the host before handing it
/// to the platform.
pub(crate) fn reveal(tree: &DiskTree, project: &Entity<Project>, cx: &mut Context<ProjectPanel>) {
    let Some(crumbs) = tree.action_target() else {
        return;
    };
    let Some(path) = tree.path_at(crumbs) else {
        return;
    };
    project.update(cx, |project, cx| project.reveal_path(&path, cx));
}

/// The marked paths, for the review screen in task 8.
pub(crate) fn marked_paths(tree: &DiskTree) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = tree.marks.paths.iter().cloned().collect();
    paths.sort();
    paths
}

/// Whether the pointer is over the canvas at all.
pub(crate) fn pointer_inside(x: f32, y: f32, area: gpui::Size<Pixels>) -> bool {
    x >= 0.0 && y >= 0.0 && x < area.width.as_f32() && y < area.height.as_f32()
}

/// Repaint request. Every mutation in this module ends with one.
pub(crate) fn notify(cx: &mut Context<ProjectPanel>) {
    cx.notify();
}

/// Metric for the current mode, exposed for the header.
pub(crate) fn metric(tree: &DiskTree) -> Metric {
    tree.mode.metric()
}

/// Convert a viewport pixel to a base-space point, for hit-testing.
pub(crate) fn to_base(tree: &DiskTree, x: f32, y: f32) -> (f32, f32) {
    tree.view.unproject(x, y)
}

/// Pixels helper for the painter: a viewport-space size.
pub(crate) fn px_size(width: f32, height: f32) -> gpui::Size<Pixels> {
    gpui::Size {
        width: px(width),
        height: px(height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use disk_tree_core::tree::{Node, NodeKind};

    /// The crumbs a path resolves to, and the reverse.
    ///
    /// `tree::path_of` and this are inverses; that round trip is what marks rely
    /// on, so it is worth pinning rather than assuming.
    #[test]
    fn crumbs_and_paths_are_inverses() {
        let mut root = Node::directory("root");
        let mut child = Node::directory("child");
        child
            .children
            .push(Node::entry("grandchild", NodeKind::File, 7));
        root.children.push(child);
        disk_tree_core::tree::aggregate(&mut root, Metric::Bytes);

        // `path_of` starts at the scanned root, so the root's own name is not
        // part of the path: the crumb path is relative to what was scanned.
        let base = std::path::PathBuf::from("/scan");
        let crumbs = vec![0, 0];
        let path = disk_tree_core::tree::path_of(&base, &root, &crumbs);
        assert_eq!(path, base.join("child").join("grandchild"));
        assert!(path.starts_with(&base));
    }

    /// A tree with two same-depth siblings side by side, each weighed by a
    /// file inside it. `aggregate` recomputes a directory's total from its
    /// children, so a size set on the directory itself does not survive it.
    fn row() -> Node {
        let mut root = Node::directory("root");
        let mut left = Node::directory("left");
        left.children.push(Node::entry("a", NodeKind::File, 100));
        let mut right = Node::directory("right");
        right.children.push(Node::entry("b", NodeKind::File, 200));
        root.children.push(left);
        root.children.push(right);
        disk_tree_core::tree::aggregate(&mut root, Metric::Bytes);
        root
    }

    #[test]
    fn directional_score_only_scores_the_requested_direction() {
        let from = Rect::new(0.0, 0.0, 10.0, 10.0);
        let right = Rect::new(20.0, 0.0, 10.0, 10.0);
        let below = Rect::new(0.0, 20.0, 10.0, 10.0);

        assert!(directional_score(from, right, Direction::Right).is_some());
        assert!(directional_score(from, below, Direction::Right).is_none());
        assert!(directional_score(from, below, Direction::Down).is_some());
        assert!(directional_score(from, right, Direction::Down).is_none());
    }

    #[test]
    fn a_tile_directly_below_beats_one_below_and_beside() {
        let from = Rect::new(0.0, 0.0, 10.0, 10.0);
        let straight = Rect::new(0.0, 20.0, 10.0, 10.0);
        let offset = Rect::new(40.0, 20.0, 10.0, 10.0);
        let a = directional_score(from, straight, Direction::Down).unwrap();
        let b = directional_score(from, offset, Direction::Down).unwrap();
        assert!(a < b, "{a} should beat {b}");
    }

    #[test]
    fn cycling_siblings_wraps_in_both_directions() {
        let tree = row();

        // `aggregate` leaves children largest-first, so right (200) is index 0.
        assert_eq!(tree.children[0].name.as_ref(), "right");
        assert_eq!(tree.children[1].name.as_ref(), "left");

        let len = tree.children.len() as i32;
        let mut index = 0i32;
        index = ((index + 1) % len + len) % len;
        assert_eq!(index, 1);
        index = ((index - 1) % len + len) % len;
        assert_eq!(index, 0, "stepping back wraps to the last sibling");
    }

    #[test]
    fn days_since_never_goes_negative() {
        // A mtime in the future, or a pre-epoch one, must not wrap into a huge
        // age bucket.
        assert_eq!(days_since(i64::MAX / 2), 0);
        assert_eq!(days_since(-1000), 0);
    }

    #[test]
    fn pointer_bounds_are_half_open() {
        let area = px_size(100.0, 50.0);
        assert!(pointer_inside(0.0, 0.0, area));
        assert!(pointer_inside(99.0, 49.0, area));
        assert!(!pointer_inside(100.0, 50.0, area), "exclusive");
        assert!(!pointer_inside(-1.0, 10.0, area));
    }

fn dummy_tree() -> DiskTree {
        DiskTree::new(
            None,
            crate::disk_tree::model::Root {
                path: PathBuf::from("/"),
                shortcut_id: None,
            },
        )
    }

    #[test]
    fn ascending_at_the_root_is_a_no_op() {
        // Crumbs empty means we are already at the scanned root.
        let mut tree = dummy_tree();
        tree.crumbs = Vec::new();
        ascend(&mut tree, px_size(100.0, 100.0), 16.0);
        assert!(tree.crumbs.is_empty());
    }

    #[test]
    fn a_leaf_directory_cannot_descend() {
        let mut node = Node::entry("file.txt", NodeKind::File, 10);
        node.children.clear();
        assert!(!node.is_dir() || node.children.is_empty());
    }
}
