//! The disk tree's model: what is loaded, where the user is, what is selected.
//!
//! Vendored from `disktree-app/src/state.rs:286-458` and `2945-3000` at
//! tobi/disktree@7c0661c (v0.10.1).
//!
//! # Where this lives in the Project Panel
//!
//! `ProjectPanel` owns this as a **field**, not as its own `Entity`. Upstream
//! `Disktree` was the whole window and needed a standalone entity with two focus
//! handles; here it is a mode of an existing panel that already has a focus
//! handle, a key dispatcher and a place to draw. A separate entity would mean
//! forwarding every keystroke and mouse event across a boundary for no gain.
//!
//! That is why `focus_request`/`apply_focus`/`FocusTarget` from upstream are
//! **not** ported: they existed only because the key handler had no `&mut Window`
//! (`state.rs:2143-2153`). Every listener in this panel gets one, so focus moves
//! directly.

use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Instant;

use disk_tree_core::classify::Category;
use disk_tree_core::removal::RemovalMode;
use disk_tree_core::scan::{Known, ScanProgress, ScanSnapshot};
use disk_tree_core::space::SpaceInfo;
use disk_tree_core::tree::{Metric, Node};
use disk_tree_core::treemap::{LayoutOptions, Rect, Tile};
use gpui::{Entity, Pixels, Point, Size};
use std::cell::Cell;

use crate::disk_tree::marks::Marks;
use crate::disk_tree::view::{LayoutTransition, View};

/// Which number the mosaic is sized by.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ColorMode {
    /// Bytes on disk. The default, because it is what fills a volume.
    #[default]
    Size,
    /// File count. Answers "what is generating the most files".
    Files,
    /// Newest write, in five buckets. Answers "what changed recently".
    Age,
}

impl ColorMode {
    pub(crate) const ALL: [Self; 3] = [Self::Size, Self::Files, Self::Age];

    pub(crate) fn next(self) -> Self {
        match self {
            Self::Size => Self::Files,
            Self::Files => Self::Age,
            Self::Age => Self::Size,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Size => "Size",
            Self::Files => "Files",
            Self::Age => "Age",
        }
    }

    /// The metric `Size` and `Files` sort by. `Age` has no metric: it colours by
    /// modification time and leaves the areas alone.
    pub(crate) fn metric(self) -> Metric {
        match self {
            Self::Files => Metric::Files,
            _ => Metric::Bytes,
        }
    }
}

/// Whether the applied filter keeps a tile, holds it dimmed, or drops it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Filtered {
    Shown,
    Holds,
    Out,
}

impl Filtered {
    /// How far the fill is pulled toward the surface. `Holds` and `Out` are
    /// deliberately *not* two steps of "more faded": `Holds` still has to be
    /// readable, because it is a directory that *contains* a match.
    pub(crate) fn fade(self) -> f32 {
        match self {
            Self::Shown => 0.0,
            Self::Holds => 0.55,
            Self::Out => 0.82,
        }
    }
}

/// A label to shape and paint inside a tile.
#[derive(Clone, Debug)]
pub(crate) struct Label {
    /// Base-space rectangle. Projected by the view at paint time.
    pub(crate) rect: Rect,
    /// Set when the label belongs to a tile's header band rather than its body.
    pub(crate) header: Option<Rect>,
    pub(crate) text: String,
    pub(crate) size_text: String,
    /// Tiles at depth 0 are labelled boldly; deeper ones at 0.88 alpha.
    pub(crate) depth: u32,
}

/// The per-frame payload the painter consumes: geometry already resolved, no
/// lookups left to do inside the paint loop.
#[derive(Clone, Debug)]
pub(crate) struct Mosaic {
    pub(crate) tiles: Vec<TileDeco>,
    pub(crate) labels: Vec<Label>,
    pub(crate) view: View,
}

/// One tile, resolved for painting.
#[derive(Clone, Debug)]
pub(crate) struct TileDeco {
    /// Base-space geometry.
    pub(crate) rect: Rect,
    pub(crate) depth: u32,
    pub(crate) category: Category,
    /// A marked ancestor covers this tile, so it is drawn as background only.
    pub(crate) covered: bool,
    pub(crate) filtered: Filtered,
    pub(crate) hovered: bool,
    pub(crate) marked: bool,
    pub(crate) reclaimable: bool,
    pub(crate) selected: bool,
    /// A directory that could not be read. Drawn with a corner mark, because a
    /// silently missing subtree is worse than an ugly one.
    pub(crate) unreadable: bool,
    /// Age bucket, in age mode only.
    pub(crate) age_bucket: Option<usize>,
    /// Whether this tile has children, i.e. whether descending is possible.
    pub(crate) descendable: bool,
    /// The tile's crumbs from the scanned root, so a click can select it.
    pub(crate) crumbs: Vec<usize>,
}

/// A cached layout, valid for one (crumbs, area, options, filter).
pub(crate) struct LayoutCache {
    pub(crate) key: LayoutKey,
    pub(crate) tiles: Vec<Tile>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LayoutKey {
    pub(crate) crumbs: Vec<usize>,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) options: LayoutOptions,
    /// Bumped whenever the applied filter changes, so a filter change drops the
    /// cache instead of silently reusing a layout that no longer matches.
    pub(crate) filter: u64,
}

impl LayoutKey {
    pub(crate) fn new(
        crumbs: &[usize],
        width: f32,
        height: f32,
        options: LayoutOptions,
        filter: u64,
    ) -> Self {
        Self {
            crumbs: crumbs.to_vec(),
            width,
            height,
            options,
            filter,
        }
    }
}

/// Which screen the panel is showing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Screen {
    /// Drawing a tree, or waiting for the first one.
    Explore,
    /// A scan is running and there is nothing to show yet.
    Running,
    /// The marked set is being reviewed before anything is removed.
    Review,
    /// A removal the user confirmed is running.
    Removing,
}

/// Geometric direction, for arrow-key navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// The axis a move must stay on. Selection never jumps between depths with
    /// the arrow keys - that is `enter` and `backspace` - because a jump that
    /// changes depth has no geometric meaning to the user.
    pub(crate) fn horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    pub(crate) fn forwards(self) -> bool {
        matches!(self, Self::Down | Self::Right)
    }
}

/// A breadcrumb: one level of the path from the scanned root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Crumb {
    pub(crate) label: String,
    pub(crate) crumbs: Vec<usize>,
    pub(crate) is_last: bool,
}

/// The known history of visited directories, for the back/forward actions.
#[derive(Default)]
pub(crate) struct History {
    pub(crate) entries: Vec<Vec<usize>>,
    pub(crate) at: usize,
}

impl History {
    /// Record a new current location. `at` is the index of the location the
    /// user is at, so `at == entries.len() - 1` means "at the newest one".
    pub(crate) fn visit(&mut self, crumbs: Vec<usize>) {
        // Whatever was ahead of here is no longer reachable.
        self.entries.truncate(self.at + 1);
        if self.entries.last() != Some(&crumbs) {
            self.entries.push(crumbs);
        }
        self.at = self.entries.len() - 1;
    }

    pub(crate) fn back(&mut self) -> Option<&[usize]> {
        if self.at == 0 {
            return None;
        }
        self.at -= 1;
        self.entries.get(self.at).map(Vec::as_slice)
    }

    pub(crate) fn forward(&mut self) -> Option<&[usize]> {
        if self.at + 1 >= self.entries.len() {
            return None;
        }
        self.at += 1;
        self.entries.get(self.at).map(Vec::as_slice)
    }

    /// Forget everything, for a new root.
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.at = 0;
    }

    pub(crate) fn can_go_back(&self) -> bool {
        self.at > 0
    }

    pub(crate) fn can_go_forward(&self) -> bool {
        self.at + 1 < self.entries.len()
    }
}

/// The scan in flight, plus what is known about the last one.
pub(crate) struct ScanState {
    /// The tree being scanned into. `None` until the first result lands.
    pub(crate) tree: Option<std::sync::Arc<Node>>,
    /// The previous tree, handed to a widening scan so it does not start over.
    pub(crate) known: Option<Known>,
    /// The running scan, so it can be polled and cancelled. `None` when nothing
    /// is in flight.
    ///
    /// Held here rather than only in the polling task because that task is
    /// detached and cannot be told to stop from outside. This is the shared
    /// progress counter rather than the `ScanHandle`, which owns the result
    /// channel and so cannot be shared; cancelling only needs the counter.
    pub(crate) handle: Option<std::sync::Arc<ScanProgress>>,
    pub(crate) snapshot: ScanSnapshot,
    /// The root being scanned.
    pub(crate) root: PathBuf,
    /// Guards against a stale poll updating state after a rescan started.
    pub(crate) epoch: u64,
    /// When the scan started, for the elapsed label.
    pub(crate) started: Option<Instant>,
    /// Set when the scan ended in something the user should see.
    pub(crate) failure: Option<String>,
}

impl ScanState {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self {
            tree: None,
            known: None,
            handle: None,
            snapshot: ScanSnapshot::default(),
            root,
            epoch: 0,
            started: None,
            failure: None,
        }
    }

    pub(crate) fn is_running(&self) -> bool {
        !self.snapshot.finished && !self.snapshot.cancelled
    }

    /// Bytes counted so far, for the scanning screen.
    pub(crate) fn scanned_bytes(&self) -> u64 {
        self.tree
            .as_ref()
            .map(|tree| tree.bytes)
            .unwrap_or(self.snapshot.bytes)
    }
}

/// Free space on the volume being shown.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Space {
    pub(crate) info: Option<SpaceInfo>,
    /// Read before and after a removal, so the "you got this back" number is
    /// measured rather than predicted.
    pub(crate) baseline_free: Option<u64>,
}

impl Space {
    /// What a removal of `bytes` would leave free, if we can measure it.
    pub(crate) fn after_removing(&self, bytes: u64) -> Option<u64> {
        let info = self.info.as_ref()?;
        Some(info.available.saturating_add(bytes))
    }
}

/// What the treemap is currently showing. The canvas writes the measured bounds
/// back through these two cells, because a `canvas` prepaint closure cannot
/// return anything the paint closure wants.
#[derive(Default)]
pub(crate) struct Viewport {
    pub(crate) origin: Rc<Cell<Point<Pixels>>>,
    pub(crate) size: Rc<Cell<Size<Pixels>>>,
}

/// Which directory the treemap is rooted at, and what it is doing there.
pub(crate) struct Root {
    pub(crate) path: PathBuf,
    /// The Storage strip entry this root came from, if any, so the strip can
    /// show which one is active.
    pub(crate) shortcut_id: Option<String>,
}

/// Everything the disk tree needs, minus the parts that live on the panel.
pub(crate) struct DiskTree {
    /// The owning panel, kept so a later step can re-enter it. None in tests,
    /// which never need a live panel - that is why construction does not require
    /// an Entity at all.
    pub(crate) entity: Option<Entity<crate::ProjectPanel>>,
    /// The root being shown.
    pub(crate) root: Root,
    pub(crate) scan: ScanState,
    pub(crate) marks: Marks,
    pub(crate) space: Space,
    pub(crate) viewport: Viewport,

    /// Where the user is, as crumbs from the scanned root.
    pub(crate) crumbs: Vec<usize>,
    /// Visited directories, for the back and forward actions. Upstream kept this
    /// on the entity; here it is one more field of the same struct.
    pub(crate) history: History,
    /// What is selected, as crumbs from the scanned root.
    pub(crate) selected: Option<Vec<usize>>,
    /// What the pointer is over. `None` when the pointer is not over the canvas.
    pub(crate) hovered: Option<Vec<usize>>,
    /// Pointer position in treemap-local coordinates.
    pub(crate) pointer: Option<(f32, f32)>,
    /// **Arbitrates between pointer and keyboard.**
    ///
    /// If the mouse moved more recently than the last key, `action_target` is
    /// `hovered`; otherwise it is `selected`. Without this, arrowing the
    /// selection away from where the cursor rests makes the highlight appear to
    /// jump under the pointer. Upstream regression-tests this
    /// (`disktree-app/tests.rs:922-972`) and so do we - it is the difference
    /// between the treemap feeling right and feeling haunted.
    pub(crate) pointer_active: bool,

    pub(crate) view: View,
    pub(crate) transition: Option<LayoutTransition>,
    pub(crate) cache: Option<LayoutCache>,
    pub(crate) mode: ColorMode,
    pub(crate) screen: Screen,
    pub(crate) depth: u32,
    pub(crate) max_depth: u32,

    /// The find field's text, and whether it is open.
    pub(crate) find: Option<String>,
    pub(crate) find_epoch: u64,

    /// Include dotfiles. A rescan is needed for a change to take effect.
    pub(crate) include_hidden: bool,
    /// Report `st_blocks * 512` instead of apparent size.
    pub(crate) on_disk_size: bool,
    /// Follow symlinks. Off by default: a link cycle is a scan that never ends.
    pub(crate) follow_links: bool,
    /// Stay on one filesystem, so a bind mount is not counted twice.
    pub(crate) one_filesystem: bool,
    /// Highlight the selection even when the pointer is elsewhere.
    pub(crate) show_selection: bool,

    /// Bumped whenever the applied filter changes.
    pub(crate) filter_epoch: u64,
    /// Set while a find is running, cleared when it lands.
    pub(crate) filter: Option<Vec<usize>>,

    /// The review screen's state: what the plan says and, once the user has
    /// confirmed, the running removal.
    pub(crate) review: ReviewState,
}

/// What the review screen shows, and what the user has agreed to.
///
/// The plan is built from the marked paths every time the screen opens, so
/// editing the marks and reopening is the way to change it. Nothing here is a
/// licence to remove: [`super::review::start_removal`] is the only thing that
/// reads [`RemovalMode`], and it requires `confirmed` to have been set first.
pub(crate) struct ReviewState {
    /// The plan as last built, for rendering.
    pub(crate) plan: Option<disk_tree_core::removal::Plan>,
    /// What the user chose: trash, or delete for good.
    pub(crate) mode: RemovalMode,
    /// Set by the confirmation step. A plan is not acted on without it, which is
    /// what makes the review screen a gate rather than a preview.
    pub(crate) confirmed: bool,
    /// The running removal, if any.
    pub(crate) running: Option<disk_tree_core::removal::RemovalHandle>,
    /// Progress in the removal's own terms: targets done and bytes freed.
    pub(crate) removed: u64,
    pub(crate) freed: u64,
    pub(crate) failed: usize,
    /// Whether the removal finished, for the summary line.
    pub(crate) done: bool,
}

impl Default for ReviewState {
    fn default() -> Self {
        // `RemovalMode`'s own `Default` is `Permanent`, because that is the right
        // default for a programmatic caller. It is the *wrong* default for a
        // button: an unrecoverable delete should be the thing the user has to
        // ask for twice, so the screen opens on the recoverable one.
        Self {
            plan: None,
            mode: RemovalMode::Trash,
            confirmed: false,
            running: None,
            removed: 0,
            freed: 0,
            failed: 0,
            done: false,
        }
    }
}

impl DiskTree {
    pub(crate) fn new(entity: Option<Entity<crate::ProjectPanel>>, root: Root) -> Self {
        // Read the path before `root` moves into the struct below.
        let scan = ScanState::new(root.path.clone());
        Self {
            entity,
            scan,
            root,
            marks: Marks::default(),
            space: Space::default(),
            viewport: Viewport::default(),
            crumbs: Vec::new(),
            history: History::default(),
            selected: None,
            hovered: None,
            pointer: None,
            pointer_active: false,
            view: View::IDENTITY,
            transition: None,
            cache: None,
            mode: ColorMode::default(),
            screen: Screen::Explore,
            depth: 0,
            max_depth: 3,
            find: None,
            find_epoch: 0,
            include_hidden: true,
            on_disk_size: true,
            follow_links: false,
            one_filesystem: true,
            show_selection: true,
            filter_epoch: 0,
            filter: None,
            review: ReviewState::default(),
        }
    }

    /// The tree being drawn, if a scan has produced one.
    pub(crate) fn tree(&self) -> Option<&std::sync::Arc<Node>> {
        self.scan.tree.as_ref()
    }

    /// The node at `crumbs`, resolved from the scanned root. **Crumbs are
    /// absolute**: relative crumbs look right at the root and silently point at
    /// other directories after descending.
    pub(crate) fn node(&self, crumbs: &[usize]) -> Option<&Node> {
        self.tree()?.resolve(crumbs)
    }

    /// The node currently in view - the one whose children are drawn.
    pub(crate) fn current(&self) -> Option<&Node> {
        self.node(&self.crumbs)
    }

    /// The metric the current mode sorts by, for the header.
    pub(crate) fn metric(&self) -> Metric {
        self.mode.metric()
    }

    /// The crumbs one level up. `None` at the scanned root, where there is no
    /// further out.
    pub(crate) fn parent_crumbs(&self) -> Option<Vec<usize>> {
        (!self.crumbs.is_empty()).then(|| self.crumbs[..self.crumbs.len() - 1].to_vec())
    }

    /// Remember where the user is, so the back action has somewhere to go.
    pub(crate) fn visit(&mut self) {
        self.history.visit(self.crumbs.clone());
    }

    /// The absolute path at `crumbs`, for revealing, marking and the breadcrumb
    /// labels. `None` when the crumbs do not resolve, or resolve to something
    /// that is not a directory - a file has nothing to reveal or mark as a tree
    /// node.
    pub(crate) fn path_at(&self, crumbs: &[usize]) -> Option<PathBuf> {
        let tree = self.tree()?;
        let node = tree.resolve(crumbs)?;
        node.is_dir()
            .then(|| disk_tree_core::tree::path_of(&self.root.path, tree, crumbs))
    }

    /// The absolute path of any node, including a file.
    pub(crate) fn path_of_any(&self, crumbs: &[usize]) -> Option<PathBuf> {
        let tree = self.tree()?;
        tree.resolve(crumbs)?;
        Some(disk_tree_core::tree::path_of(&self.root.path, tree, crumbs))
    }

    /// The root's own path, always available even before a scan lands.
    pub(crate) fn root_path(&self) -> &Path {
        &self.root.path
    }

    /// What a keystroke acts on: the pointer's tile if the pointer moved most
    /// recently, otherwise the selection.
    pub(crate) fn action_target(&self) -> Option<&[usize]> {
        if self.pointer_active && self.hovered.is_some() {
            self.hovered.as_deref()
        } else {
            self.selected.as_deref()
        }
    }

    /// Move the selection onto whatever the pointer is over, before an action
    /// reads it. See `pointer_active`.
    pub(crate) fn adopt_pointer_target(&mut self) {
        if self.pointer_active
            && let Some(hovered) = self.hovered.clone()
        {
            self.selected = Some(hovered);
        }
    }

    /// The visited-directory history, for the back and forward actions.
    pub(crate) fn history(&self) -> &History {
        &self.history
    }

    pub(crate) fn history_mut(&mut self) -> &mut History {
        &mut self.history
    }

    /// Breadcrumbs for the trail, root first.
    pub(crate) fn breadcrumb_trail(&self) -> Vec<Crumb> {
        let Some(tree) = self.tree() else {
            return Vec::new();
        };
        let chain = tree.resolve_chain(&self.crumbs);
        let mut crumbs = Vec::with_capacity(chain.len());
        let mut out = Vec::with_capacity(chain.len());
        for (index, node) in chain.iter().enumerate() {
            crumbs.push(index);
            out.push(Crumb {
                label: node.name.to_string(),
                is_last: index + 1 == chain.len(),
                crumbs: crumbs.clone(),
            });
        }
        out
    }
}

/// The nesting levels the depth ladder offers, 1..=6.
///
/// One level draws only the current root's children; six is past the point where
/// a mosaic in a ~300 px panel can resolve anything, so it is the ceiling
/// upstream chose and the ceiling kept here.
pub(crate) const DEPTH_STEPS: std::ops::RangeInclusive<u32> = 1..=6;
