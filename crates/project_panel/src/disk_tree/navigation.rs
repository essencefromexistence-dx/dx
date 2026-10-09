//! Building the treemap geometry: layout, hit-testing, navigation.
//!
//! Vendored from `disktree-app/src/state.rs:1533-1700` and `1780-1970` at
//! tobi/disktree@7c0661c (v0.10.1).
//!
//! # The invariant that makes this cheap
//!
//! **Layout runs in base-space pixels and is cached.** `LayoutKey` holds the
//! crumbs, the viewport size, the options and the filter epoch; any change to
//! any of those drops the cache and re-lays out. Zoom and pan change
//! [`View`], never the layout, so magnifying a large tree costs one paint.
//!
//! # Crumbs are absolute
//!
//! Every crumbs value here is a path from the **scanned root**, never from the
//! current directory. `treemap::layout` is handed the drawn node's crumbs and
//! every tile extends them, so a tile resolves from the root at any depth.
//! Relative crumbs look correct at the root and silently point at other
//! directories after descending - including for marks.

use disk_tree_core::tree::Node;
use disk_tree_core::treemap::{self, LayoutOptions, Rect, Tile, TileKind};
use gpui::Pixels;

use crate::disk_tree::model::{
    ColorMode, DiskTree, Filtered, LayoutCache, LayoutKey, Mosaic, TileDeco,
};
use crate::disk_tree::view::View;

/// How many tiles a label is allowed to claim before the deepest ones are
/// dropped. A squarified treemap of a whole disk can produce thousands; drawing
/// a label for every one of them is what makes a treemap slow, not the quads.
const MAX_LABELS: usize = 400;

/// The layout options for the current depth and rem, plus the current area.
fn current_key(tree: &DiskTree, area: gpui::Size<Pixels>, rem: f32) -> LayoutKey {
    LayoutKey::new(
        &tree.crumbs,
        area.width.as_f32(),
        area.height.as_f32(),
        layout_options(tree.max_depth, rem),
        tree.filter_epoch,
    )
}

/// The cached layout for the current state, recomputing only on a key mismatch.
pub(crate) fn layout(tree: &mut DiskTree, area: gpui::Size<Pixels>, rem: f32) -> Vec<Tile> {
    let key = current_key(tree, area, rem);
    if let Some(cache) = tree.cache.as_ref()
        && cache.key == key
    {
        return cache.tiles.clone();
    }

    let tiles = compute_tiles(tree, area, rem);
    tree.cache = Some(LayoutCache {
        key,
        tiles: tiles.clone(),
    });
    tiles
}

fn compute_tiles(tree: &DiskTree, area: gpui::Size<Pixels>, rem: f32) -> Vec<Tile> {
    let Some(node) = tree.current() else {
        return Vec::new();
    };
    let options = layout_options(tree.max_depth, rem);
    let area =
        disk_tree_core::treemap::Rect::new(0.0, 0.0, area.width.as_f32(), area.height.as_f32());
    treemap::layout(node, &tree.crumbs, area, tree.metric(), &options)
}

/// Resolve every tile into its painted form, and choose which ones get a label.
///
/// This is the per-frame resolver. It runs before paint so the paint loop does
/// no lookups at all - that separation is upstream's, and it is the reason a
/// whole-disk mosaic stays smooth.
pub(crate) fn prepare(tree: &DiskTree, tiles: Vec<Tile>, area: gpui::Size<Pixels>) -> Mosaic {
    let now = tree.pointer;
    let mut decos = Vec::with_capacity(tiles.len());
    let mut labels = Vec::new();
    let mut label_budget = MAX_LABELS;

    // Walk in layout order so the largest tiles claim labels first. `layout`
    // emits parents before children and siblings biggest-first, which is exactly
    // the priority a reader wants.
    for tile in &tiles {
        let Some(tile_view) = TileView::new(tree, tile) else {
            continue;
        };

        if label_budget > 0 && tile_view.wants_label(tree.view, area) {
            if let Some(label) = tile_view.label(tree) {
                labels.push(label);
                label_budget -= 1;
            }
        }

        decos.push(tile_view.deco(tree));
    }

    // The cursor tooltip needs the pointer's tile even when it got no label, so
    // hit-testing stays independent of the label budget.
    let _ = now;

    Mosaic {
        tiles: decos,
        labels,
        view: tree.view,
    }
}

/// One tile, read against the tree.
struct TileView<'a> {
    tile: &'a Tile,
    node: &'a Node,
    depth: u32,
    filtered: Filtered,
    marked: bool,
    covered: bool,
    hovered: bool,
    selected: bool,
}

impl<'a> TileView<'a> {
    fn new(tree: &'a DiskTree, tile: &'a Tile) -> Option<Self> {
        let root = tree.tree()?;
        // `Tile::crumbs()` rather than a field: the two `TileKind` variants hold
        // their crumbs differently, and reading them uniformly here is the whole
        // reason the accessor exists.
        let crumbs = tile.crumbs();
        let node = root.resolve(crumbs)?;
        let path = disk_tree_core::tree::path_of(&tree.root.path, root, crumbs);
        let marked = tree.marks.paths.contains(&path);
        // A tile *inside* a marked directory is background only: the mark is
        // what the user asked about, and its children must not read as
        // separately removable. A marked tile is not covered - it is the thing
        // marked, and it needs to stand out.
        let covered = !marked && tree.covered_by_mark(crumbs);
        Some(Self {
            tile,
            node,
            depth: tile.depth,
            filtered: tree.filtered(tile),
            marked,
            covered,
            hovered: tree.hovered.as_deref() == Some(crumbs),
            selected: tree.selected.as_deref() == Some(crumbs),
        })
    }

    fn rect(&self) -> Rect {
        self.tile.rect
    }

    fn area(&self) -> f32 {
        self.rect().area()
    }

    /// A tile is worth labelling when it is big enough to hold text **at the current
    /// zoom**. This is why the test needs the view and not just the area: at
    /// `scale = 1` a 30 px tile is unreadable, and at `scale = 5` the same tile
    /// is the most legible thing on screen.
    fn wants_label(&self, view: View, _area: gpui::Size<Pixels>) -> bool {
        // Filtered-out tiles are dimmed past the point of reading, so they get no
        // label even when they are large.
        if matches!(self.filtered, Filtered::Out) {
            return false;
        }
        let screen = view.project(self.rect());
        const MIN_LABEL_WIDTH: f32 = 24.0;
        const MIN_LABEL_HEIGHT: f32 = 12.0;
        screen.w >= MIN_LABEL_WIDTH && screen.h >= MIN_LABEL_HEIGHT
    }

    fn label(&self, tree: &DiskTree) -> Option<crate::disk_tree::model::Label> {
        use crate::disk_tree::model::Label;
        let node = self.node;
        let size_text = match tree.mode {
            ColorMode::Age => String::new(),
            ColorMode::Files => disk_tree_core::size::human_count(node.files),
            ColorMode::Size => disk_tree_core::size::human_bytes_short(node.bytes),
        };
        // The band is present only when the layout subdivided this directory, so its
        // presence *is* the "drawn open as a directory" answer.
        let header = self.header_band();
        Some(Label {
            rect: self.rect(),
            header,
            text: node.name.to_string(),
            size_text,
            depth: self.depth,
        })
    }

    /// The band a directory keeps for its name, in base space. Read from the
    /// tile, which carries the rectangle the layout actually reserved, so a
    /// label always fills the band it was given.
    fn header_band(&self) -> Option<Rect> {
        self.tile.header
    }

    /// Whether this tile is reclaimable - a cache, a sync client's old versions, a
    /// package store, build output, trash, and so on.
    ///
    /// A *reason*, not a measurement: `classify::Reclaim` says what kind of thing
    /// it is. Nothing here decides whether removing it is safe; that is
    /// `disk_tree_core::removal`'s job, and it refuses paths it must.
    fn reclaimable(&self) -> bool {
        self.node.reclaim.is_some()
    }

    fn deco(&self, tree: &DiskTree) -> TileDeco {
        TileDeco {
            rect: self.rect(),
            depth: self.depth,
            category: self.node.category,
            covered: self.covered,
            filtered: self.filtered,
            hovered: self.hovered,
            marked: self.marked,
            reclaimable: self.reclaimable(),
            selected: self.selected,
            unreadable: self.node.read_error,
            age_bucket: tree.age_bucket(self.node),
            descendable: is_descendable(&self.tile.kind),
            crumbs: self.tile.crumbs().to_vec(),
        }
    }
}

/// The mosaic's layout options for a given depth and rem scale.
///
/// `header`/`header_inner` are the bands a directory keeps for its name. They are
/// multiplied by the interface's rem so they scale with zoom like every other
/// dimension - upstream did the same with `HEADER_REMS * self.rem`
/// (`state.rs:2935-2940`).
pub(crate) fn layout_options(depth: u32, rem: f32) -> LayoutOptions {
    let mut options = LayoutOptions::default();
    options.max_depth = depth;
    options.header = 1.5 * rem;
    options.header_inner = 1.125 * rem;
    options.rem = rem;
    options
}

/// A tile's index in the layout, for hit-testing.
pub(crate) fn tile_at(tiles: &[Tile], x: f32, y: f32) -> Option<&Tile> {
    disk_tree_core::treemap::hit(tiles, x, y)
}

/// Whether a tile can be entered. `Others` is a merged tail, not a directory,
/// so it is deliberately not descendable.
pub(crate) fn is_descendable(kind: &TileKind) -> bool {
    matches!(kind, TileKind::Node { .. })
}
