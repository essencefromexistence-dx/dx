//! Pointer handling for the mosaic: hover, click, wheel-zoom.
//!
//! Vendored from `disktree-app/src/state.rs:2790-2919` and `1982-1986` at
//! tobi/disktree@7c0661c (v0.10.1).
//!
//! # How a mouse event becomes a tile
//!
//! 1. The event's window position is made treemap-local by subtracting the
//!    canvas origin the paint pass recorded.
//! 2. It is unprojected through [`View`] into base space, because the layout is
//!    cached in base space and hit-testing must not know about zoom.
//! 3. `treemap::hit` returns the deepest tile there.
//!
//! Upstream kept the canvas origin and size in `Rc<Cell<..>>` fields on the
//! entity. They are still fields - [`crate::disk_tree::model::Viewport`] - and the
//! canvas prepaint still writes them; that part of the design is unchanged.

use gpui::{
    Context, MouseButton, MouseDownEvent, MouseMoveEvent, NavigationDirection, ScrollDelta,
    ScrollWheelEvent, Window,
};

use crate::ProjectPanel;
use crate::disk_tree::model::{Direction, DiskTree};
use crate::disk_tree::moves;
use crate::disk_tree::navigation;
use crate::disk_tree::navigation::{is_descendable, tile_at};
use crate::disk_tree::view::View;

/// Wheel zoom per notch. 1.15 is a whole-feeling step: at 1.1 the mosaic appears
/// not to respond, at 1.3 it jumps.
const WHEEL_ZOOM: f32 = 1.15;

/// Pan, in base-space pixels, per wheel notch with shift held.
const WHEEL_PAN: f32 = 40.0;

/// The current area of the treemap, or `None` before the first frame measured it.
///
/// `None` rather than a zero size, because "not measured yet" and "measured as
/// nothing" must not both be handled as a degenerate viewport: the first would
/// divide by a zero scale.
fn area(tree: Option<&DiskTree>) -> Option<gpui::Size<gpui::Pixels>> {
    let size = tree?.viewport.size.get();
    (size.width > gpui::px(0.) && size.height > gpui::px(0.)).then_some(size)
}

/// Window position to treemap-local, in base space.
///
/// `None` when the pointer is outside the canvas: a mousemove that arrives while
/// the pointer is over the header must not select anything.
fn to_base(tree: &DiskTree, position: gpui::Point<gpui::Pixels>) -> Option<(f32, f32)> {
    let size = tree.viewport.size.get();
    let area = (size.width > gpui::px(0.) && size.height > gpui::px(0.)).then_some(size)?;
    let origin = tree.viewport.origin.get();
    let local = gpui::Point::new(position.x - origin.x, position.y - origin.y);
    if local.x < gpui::px(0.)
        || local.y < gpui::px(0.)
        || local.x > area.width
        || local.y > area.height
    {
        return None;
    }
    Some(tree.view.unproject(local.x.as_f32(), local.y.as_f32()))
}

/// Drop the hover state. Called when the pointer leaves the canvas.
pub(crate) fn on_leave(panel: &mut ProjectPanel, cx: &mut Context<ProjectPanel>) {
    let tree = panel.disk_tree_mut();
    if tree.hovered.take().is_some() || tree.pointer.is_some() {
        tree.pointer = None;
        cx.notify();
    }
}

/// Track the pointer, and what it is over.
pub(crate) fn on_move(
    panel: &mut ProjectPanel,
    event: &MouseMoveEvent,
    window: &mut Window,
    cx: &mut Context<ProjectPanel>,
) {
    let area = match area(panel.disk_tree()) {
        Some(area) => area,
        None => return,
    };
    let rem = panel.disk_tree_rem(window);
    let tiles = navigation::layout(panel.disk_tree_mut(), area, rem);
    let Some(tree) = panel.disk_tree() else {
        return;
    };
    let Some((x, y)) = to_base(tree, event.position) else {
        on_leave(panel, cx);
        return;
    };

    let tree = panel.disk_tree_mut();
    tree.pointer = Some((x, y));
    let hovered = tile_at(&tiles, x, y).map(|tile| tile.crumbs().to_vec());
    let changed = tree.hovered != hovered;
    // The pointer moving counts as pointer-driven, which is what makes
    // `action_target` follow the cursor until a key is pressed.
    tree.pointer_active = true;
    if changed {
        cx.notify();
    }
}

/// Click to select, double-click or click-an-already-selected-directory to
/// descend, middle-click to mark, and the side buttons to walk history.
pub(crate) fn on_down(
    panel: &mut ProjectPanel,
    event: &MouseDownEvent,
    window: &mut Window,
    cx: &mut Context<ProjectPanel>,
) {
    let area = match area(panel.disk_tree()) {
        Some(area) => area,
        None => return,
    };

    // The side buttons are history, exactly like the browser's. Handled before
    // position so a click outside the mosaic still navigates.
    match event.button {
        MouseButton::Navigate(NavigationDirection::Back) => {
            go_back(panel, area, window, cx);
            return;
        }
        MouseButton::Navigate(NavigationDirection::Forward) => {
            go_forward(panel, area, window, cx);
            return;
        }
        _ => {}
    }

    let rem = panel.disk_tree_rem(window);
    let tiles = navigation::layout(panel.disk_tree_mut(), area, rem);
    let Some(tree) = panel.disk_tree() else {
        return;
    };
    let Some((x, y)) = to_base(tree, event.position) else {
        return;
    };
    let Some(tile) = tile_at(&tiles, x, y) else {
        return;
    };
    let crumbs = tile.crumbs().to_vec();
    let descendable = is_descendable(&tile.kind);

    // The tile under the pointer must be readable at this moment: a hit-test
    // against a stale layout selects the wrong thing.
    let already_selected = panel
        .disk_tree()
        .is_some_and(|t| t.selected.as_deref() == Some(crumbs.as_slice()));

    match event.button {
        MouseButton::Left => {
            if event.click_count >= 2 {
                let tree = panel.disk_tree_mut();
                tree.selected = Some(crumbs.clone());
                tree.pointer_active = false;
                if descendable {
                    moves::descend(tree, crumbs, area, rem);
                }
            } else if event.modifiers.control || event.modifiers.alt {
                // Ctrl/⌘ toggles the mark without moving the selection, so a
                // multi-mark does not require re-aiming each time.
                toggle_mark(panel, &crumbs, cx);
            } else if already_selected && descendable {
                // A second single click on a selected directory opens it, the way
                // a file manager does.
                moves::descend(panel.disk_tree_mut(), crumbs, area, rem);
            } else {
                let tree = panel.disk_tree_mut();
                tree.selected = Some(crumbs);
                tree.pointer_active = false;
            }
            cx.notify();
        }
        MouseButton::Middle => {
            toggle_mark(panel, &crumbs, cx);
            cx.notify();
        }
        _ => {}
    }
}

/// Wheel: zoom, or pan with shift held.
pub(crate) fn on_scroll(
    panel: &mut ProjectPanel,
    event: &ScrollWheelEvent,
    _window: &mut Window,
    cx: &mut Context<ProjectPanel>,
) {
    let area = match area(panel.disk_tree()) {
        Some(area) => area,
        None => return,
    };

    // Lines on a trackpad-free mouse, pixels on a trackpad. Normalising to lines
    // is what makes one notch mean the same thing on both.
    //
    // `Lines` carries a `Point<f32>` and `Pixels` a `Point<Pixels>`, so the two
    // arms differ: the pixel arm is divided by the 24 px line height.
    let lines = match event.delta {
        ScrollDelta::Lines(delta) => delta.y,
        ScrollDelta::Pixels(delta) => delta.y.as_f32() / 24.0,
    };
    if lines == 0.0 {
        return;
    }

    let Some(tree) = panel.disk_tree() else {
        return;
    };
    let Some((x, y)) = to_base(tree, event.position) else {
        return;
    };

    if event.modifiers.shift {
        let tree = panel.disk_tree_mut();
        tree.view.origin_y = (tree.view.origin_y - lines * WHEEL_PAN).max(0.0);
        tree.view = tree.view.clamped(area);
        cx.notify();
        return;
    }

    // Zoom toward the cursor. Past the ceiling this stops dead rather than
    // descending: in a ~300 px panel the ceiling is two notches away, and
    // silently changing directory made the wheel feel broken. `enter` is the
    // only way in.
    let factor = if lines > 0.0 {
        WHEEL_ZOOM
    } else {
        1.0 / WHEEL_ZOOM
    };
    let tree = panel.disk_tree_mut();
    tree.view = tree
        .view
        .zoomed_at(x, y, factor, View::MAX_SCALE)
        .clamped(area);
    cx.notify();
}

fn toggle_mark(panel: &mut ProjectPanel, crumbs: &[usize], cx: &mut Context<ProjectPanel>) {
    let Some(path) = panel.disk_tree().and_then(|t| t.path_at(crumbs)) else {
        return;
    };
    panel.disk_tree_mut().marks.toggle(path);
    cx.notify();
}

fn go_back(
    panel: &mut ProjectPanel,
    area: gpui::Size<gpui::Pixels>,
    window: &mut Window,
    cx: &mut Context<ProjectPanel>,
) {
    let rem = panel.disk_tree_rem(window);
    let tree = panel.disk_tree_mut();
    let crumbs = tree.history_mut().back().map(<[usize]>::to_vec);
    if let Some(crumbs) = crumbs {
        moves::descend(panel.disk_tree_mut(), crumbs, area, rem);
        panel.refresh_disk_tree_header(window);
        cx.notify();
    }
}

fn go_forward(
    panel: &mut ProjectPanel,
    area: gpui::Size<gpui::Pixels>,
    window: &mut Window,
    cx: &mut Context<ProjectPanel>,
) {
    let rem = panel.disk_tree_rem(window);
    let tree = panel.disk_tree_mut();
    let crumbs = tree.history_mut().forward().map(<[usize]>::to_vec);
    if let Some(crumbs) = crumbs {
        moves::descend(panel.disk_tree_mut(), crumbs, area, rem);
        panel.refresh_disk_tree_header(window);
        cx.notify();
    }
}

/// Move the selection with an arrow key. Never changes depth.
pub(crate) fn move_selection(
    panel: &mut ProjectPanel,
    direction: Direction,
    window: &mut Window,
    cx: &mut Context<ProjectPanel>,
) {
    let Some(area) = area(panel.disk_tree()) else {
        return;
    };
    let rem = panel.disk_tree_rem(window);
    panel.disk_tree_mut().adopt_pointer_target();
    let tiles = navigation::layout(panel.disk_tree_mut(), area, rem);
    moves::move_selection(panel.disk_tree_mut(), direction, &tiles, area);
    cx.notify();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wheel_step_is_a_whole_feeling_one() {
        // Too small reads as no response; too large reads as a jump.
        assert!(WHEEL_ZOOM > 1.1, "{WHEEL_ZOOM}");
        assert!(WHEEL_ZOOM < 1.3, "{WHEEL_ZOOM}");
    }

    #[test]
    fn a_zoom_out_notch_is_exactly_a_zoom_in_notch_reversed() {
        // Asymmetric steps make panning back feel broken.
        assert_eq!(WHEEL_ZOOM, 1.0 / WHEEL_ZOOM.powi(-1));
    }

    #[test]
    fn lines_and_pixels_normalise_to_the_same_notch() {
        // A trackpad reports pixels; dividing by the 24 px line height is what
        // makes one gesture mean one step on both kinds of device.
        let from_lines = ScrollDelta::Lines(gpui::point(0.0_f32, 1.0_f32));
        let from_pixels = ScrollDelta::Pixels(gpui::point(gpui::px(24.), gpui::px(24.)));
        let normalise = |delta: ScrollDelta| match delta {
            ScrollDelta::Lines(y) => y.y,
            ScrollDelta::Pixels(y) => y.y.as_f32() / 24.0,
        };
        assert_eq!(normalise(from_lines), normalise(from_pixels));
    }
}
