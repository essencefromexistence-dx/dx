//! Drawing the mosaic: one canvas, resolved colours, shaped labels.
//!
//! Vendored from `disktree-app/src/treemap_view.rs` (550 lines) at
//! tobi/disktree@7c0661c (v0.10.1). **Near-verbatim** - this was the finding
//! the whole port rested on.
//!
//! # Why this file needed almost no work
//!
//! Upstream's painter is stock GPUI. Every call it makes exists in our
//! `crates/gpui` at the same shape, including three that are not upstream Zed
//! API and so were worth checking individually:
//!
//! | call | ours |
//! |---|---|
//! | `canvas(prepaint, paint)` | `crates/gpui/src/elements/canvas.rs:10` |
//! | `quad(bounds, corners, fill, border, color, style)` | `crates/gpui/src/window.rs:7938` |
//! | `window.paint_quad(..)` | `crates/gpui/src/window.rs:4669` |
//! | `pattern_slash(color, width, interval)` - the hatch | `crates/gpui/src/color.rs:827` |
//! | `transparent_black()` | `crates/gpui/src/color.rs:444` |
//! | `with_content_mask(..)` | `crates/gpui/src/window.rs:4148` |
//! | `text_system()` | `crates/gpui/src/window.rs:2473` |
//! | `ShapedLine::paint(origin, h, align, w, window, cx)` | `crates/gpui/src/text_system/line.rs:108` |
//! | `Rems` | `crates/gpui/src/geometry.rs:3250` |
//! | `Window::rem_size` / `set_rem_size` | `crates/gpui/src/window.rs:3081`, `:3090` |
//! | `debug_selector` | `crates/gpui/src/elements/div.rs:885` |
//! | `on_any_mouse_down` | `crates/gpui/src/elements/div.rs:162` |
//!
//! Two upstream spellings do *not* compile here, and both are noted where they
//! appear: `Rems::as_f32` (ours exposes the `.0` field and `to_pixels`), and
//! `canvas(..).absolute()` (a `Canvas` implements `Styled` only, so the absolute
//! positioning lives on a wrapping `div`).
//!
//! # The two rules that make it fast
//!
//! **The mosaic is painted, not composed of elements.** Thousands of rectangles
//! belong in one canvas callback.
//!
//! **Every colour is resolved before paint.** `Colors::new` resolves 9 categories
//! x 5 depths plus 5 age buckets x 5 depths *once per frame*, so the paint loop
//! never touches the palette. Upstream's comment says so; it is the reason this
//! file is worth reading twice.

use disk_tree_core::classify::Category;
use gpui::{
    App, Bounds, ContentMask, Corners, Edges, FocusHandle, Font, FontWeight, Hsla,
    InteractiveElement, MouseDownEvent, MouseMoveEvent, ParentElement, Pixels, Point, ScrollEvent,
    SharedString, Size, StatefulInteractiveElement, Styled, TextAlign, TextRun, Window, canvas,
    pattern_slash, px, quad, transparent_black,
};
use std::cell::Cell;
use std::rc::Rc;

use crate::ProjectPanel;
use crate::disk_tree::model::{Filtered, Label, Mosaic, TileDeco};
use crate::disk_tree::palette;
use crate::disk_tree::theme::DiskTreeTokens;
use crate::disk_tree::view::View;

/// How deep the fill ladder goes. A tile deeper than this renders as depth 4 -
/// see `category_fill`, which clamps - so a very deep tree does not fade to
/// nothing.
pub(crate) const DEPTHS: usize = 5;

/// The mosaic, as a GPUI element.
///
/// All four mouse listeners live on the wrapping `div`, **not** on the canvas:
/// a `Canvas` has no element id and is not independently interactive
/// (`crates/gpui/src/elements/canvas.rs:41-43`). This is the same arrangement
/// `git_graph.rs` uses (`crates/git_ui/src/git_graph.rs:4563-4604`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn render(
    mosaic: Mosaic,
    tokens: DiskTreeTokens,
    font: SharedString,
    rem: f32,
    focus_handle: &FocusHandle,
    origin: Rc<Cell<Point<Pixels>>>,
    measured: Rc<Cell<Size<Pixels>>>,
    cx: &gpui::Context<ProjectPanel>,
) -> impl gpui::IntoElement {
    let colors = Colors::new(&tokens);
    // Upstream wrote `tokens::text::BODY.as_f32()`; our `Rems` exposes the `.0`
    // field (and `to_pixels`, which needs a `Pixels` rem size). The `.0` read is
    // the same value multiplied by `rem` below.
    let name_size = rem_px(crate::disk_tree::tokens::text::BODY.0, rem);
    let size_size = rem_px(crate::disk_tree::tokens::text::CAPTION.0, rem);

    let Mosaic {
        tiles,
        labels,
        view,
    } = mosaic;

    let canvas_origin = origin;
    let canvas_measured = Rc::clone(&measured);

    gpui::div()
        .id("dx-explorer-disk-tree")
        .debug_selector(|| "disk-tree".into())
        .relative()
        .flex_1()
        .min_h_0()
        .min_w_0()
        .overflow_hidden()
        .bg(tokens.surface)
        .track_focus(focus_handle)
        .on_hover(cx.listener(|panel, hovered: &bool, _window, cx| {
            // Clearing hover on leave is what stops a stale highlight lingering
            // after the pointer leaves the panel.
            if !hovered {
                crate::disk_tree::mouse::on_leave(panel, cx);
            }
        }))
        .on_mouse_move(cx.listener(|panel, event: &MouseMoveEvent, window, cx| {
            crate::disk_tree::mouse::on_move(panel, event, window, cx);
        }))
        .on_any_mouse_down(cx.listener(|panel, event: &MouseDownEvent, window, cx| {
            crate::disk_tree::mouse::on_down(panel, event, window, cx);
        }))
        .on_scroll(cx.listener(|panel, event: &ScrollEvent, window, cx| {
            crate::disk_tree::mouse::on_scroll(panel, event, window, cx);
        }))
        .child(
            // A `Canvas` implements `Styled` but not `Div`, so it has neither
            // `absolute()` nor an element id. Upstream chained
            // `.absolute().inset_0()` onto the canvas itself; here the absolute
            // positioning lives on a wrapping `div` and the canvas fills it.
            gpui::div().absolute().inset_0().child(canvas(
                // Closure 1: prepaint / measure. This is the only channel from
                // the paint pass back to state - a `canvas` prepaint closure's
                // return value goes to paint, not to the entity.
                move |bounds: Bounds<Pixels>, window: &mut Window, _| {
                    // Both cells are the ones the tree holds, so a mouse event
                    // later in this frame maps back to the same coordinates the
                    // paint pass used.
                    canvas_origin.set(bounds.origin);
                    if canvas_measured.get() != bounds.size {
                        canvas_measured.set(bounds.size);
                        // The mosaic depends on the viewport size (clamping,
                        // label sizes), so a resize needs a second frame.
                        window.request_animation_frame();
                    }
                    bounds.size
                },
                // Closure 2: paint.
                move |bounds: Bounds<Pixels>,
                      _: Size<Pixels>,
                      window: &mut Window,
                      cx: &mut App| {
                    paint_tiles(&tiles, bounds, view, &colors, window);
                    paint_labels(
                        &labels, bounds, view, &colors, &font, name_size, size_size, window, cx,
                    );
                },
            )),
        )
}

fn rem_px(rems: f32, rem: f32) -> Pixels {
    px(rems * rem)
}

/// Every colour the paint loop needs, resolved once per frame.
///
/// Indexed `[category][depth]` and `[bucket][depth]`. Keeping the palette out of
/// the paint loop is the point: `category_fill` is three float ops and a mix,
/// and calling it 4 000 times per frame is how a treemap drops frames.
struct Colors {
    label: [Hsla; 2],
    label_dim: Hsla,
    hover_border: Hsla,
    selected_border: Hsla,
    marked_border: Hsla,
    marked_label: Hsla,
    warning: Hsla,
    hatch: Hsla,
    /// Per category, per depth.
    fill: Vec<[Hsla; DEPTHS]>,
    /// Per category: the accent strip over a top-level directory.
    strip: Vec<Hsla>,
    /// Per age bucket, per depth.
    age: Vec<[Hsla; DEPTHS]>,
    marked_fill: Hsla,
    inset: Hsla,
}

impl Colors {
    fn new(tokens: &DiskTreeTokens) -> Self {
        let mut fill = Vec::with_capacity(Category::LEGEND.len());
        let mut strip = Vec::with_capacity(Category::LEGEND.len());
        for &category in &Category::LEGEND {
            let mut column = [Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 1.0,
            }; DEPTHS];
            for (depth, slot) in column.iter_mut().enumerate() {
                *slot = palette::category_fill(tokens, category, depth as u32);
            }
            fill.push(column);
            strip.push(palette::category_accent(tokens, category));
        }

        let mut age = Vec::with_capacity(palette::AGE_BUCKETS.len());
        for bucket in 0..palette::AGE_BUCKETS.len() {
            let mut column = [Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 1.0,
            }; DEPTHS];
            for (depth, slot) in column.iter_mut().enumerate() {
                *slot = palette::age_fill(tokens, bucket, depth as u32);
            }
            age.push(column);
        }

        Self {
            label: [
                palette::label_color(tokens, 0),
                palette::label_color(tokens, 1),
            ],
            label_dim: tokens.text_muted.opacity(0.7),
            hover_border: tokens.border,
            selected_border: palette::highlight(tokens),
            marked_border: palette::highlight(tokens),
            marked_label: palette::on_highlight(tokens),
            warning: tokens.highlight,
            hatch: palette::hatch(tokens),
            fill,
            strip,
            age,
            marked_fill: palette::highlight(tokens).opacity(0.28),
            inset: tokens.inset_border,
        }
    }

    /// The fill for a tile, honouring the filter's fade.
    fn fill(&self, tile: &TileDeco) -> Hsla {
        let depth = (tile.depth as usize).min(DEPTHS - 1);
        let base = match tile.age_bucket {
            Some(bucket) => self
                .age
                .get(bucket)
                .and_then(|column| column.get(depth))
                .copied()
                .unwrap_or(self.label[0]),
            None => self
                .fill
                .get(category_index(tile.category))
                .and_then(|column| column.get(depth))
                .copied()
                .unwrap_or(self.label[0]),
        };
        match tile.filtered {
            Filtered::Shown => base,
            other => palette::mix(base, self.inset, other.fade()),
        }
    }
}

/// A category's index in the fill tables.
fn category_index(category: Category) -> usize {
    Category::LEGEND
        .iter()
        .position(|&candidate| candidate == category)
        .unwrap_or(0)
}

/// The five quads per tile, then the outline rings on top.
fn paint_tiles(
    tiles: &[TileDeco],
    bounds: Bounds<Pixels>,
    view: View,
    colors: &Colors,
    window: &mut Window,
) {
    let none = Edges::all(px(0.));
    let solid = gpui::BorderStyle::Solid;
    let scale = window.scale_factor();

    // Rings are collected and painted after every fill, so a selection ring is
    // never painted under a later sibling's body.
    let mut rings: Vec<(u8, Bounds<Pixels>, f32, Hsla)> = Vec::new();

    for tile in tiles {
        let rect = view.project(tile.rect);
        if rect.w <= 0.5 || rect.h <= 0.5 {
            continue;
        }
        let tile_bounds = snap(to_window(&rect, bounds), scale);

        // 1. body fill, or the mark's wash when marked.
        let body = if tile.marked {
            colors.marked_fill
        } else {
            colors.fill(tile)
        };
        window.paint_quad(quad(
            tile_bounds,
            Corners::default(),
            body,
            none,
            colors.hover_border,
            solid,
        ));

        // 2. reclaimable hatch. A pattern, not a hue, so "what is it" and "can it
        //    go" read independently.
        if tile.reclaimable && !tile.marked && !tile.covered && tile.filtered == Filtered::Shown {
            window.paint_quad(quad(
                tile_bounds,
                Corners::default(),
                pattern_slash(colors.hatch, 1.0, 6.0),
                none,
                colors.hatch,
                solid,
            ));
        }

        // 3. the accent strip over a top-level directory. Skipped in age mode,
        //    where hue carries the age rather than the category.
        if tile.depth == 0
            && tile.age_bucket.is_none()
            && tile.filtered != Filtered::Out
            && let Some(accent) = colors.strip.get(category_index(tile.category))
        {
            let strip = Bounds::new(
                tile_bounds.origin,
                Size::new(
                    tile_bounds.size.width,
                    px(2.0_f32.min(tile_bounds.size.height.as_f32())),
                ),
            );
            window.paint_quad(quad(
                strip,
                Corners::default(),
                *accent,
                none,
                colors.hover_border,
                solid,
            ));
        }

        // 4. a directory that could not be read. A corner mark, because a
        //    silently missing subtree is worse than an ugly one.
        if tile.unreadable && rect.w > 12.0 && rect.h > 12.0 {
            let mark = Bounds::new(
                Point::new(
                    tile_bounds.origin.x + tile_bounds.size.width - px(6.),
                    tile_bounds.origin.y + px(2.),
                ),
                Size::new(px(4.), px(4.)),
            );
            window.paint_quad(quad(
                mark,
                Corners::default(),
                colors.warning,
                none,
                colors.warning,
                solid,
            ));
        }

        // 5. the ring. Ranked so the most important wins and goes on top.
        let ring = if tile.selected {
            Some((3u8, 2.0, colors.selected_border))
        } else if tile.hovered {
            Some((2, 1.0, colors.hover_border))
        } else if tile.marked {
            Some((1, 2.0, colors.marked_border))
        } else {
            None
        };
        if let Some((rank, width, color)) = ring {
            rings.push((rank, tile_bounds, width, color));
        }
    }

    rings.sort_by_key(|(rank, ..)| *rank);
    for (_, ring, width, color) in rings {
        window.paint_quad(quad(
            ring,
            Corners::default(),
            transparent_black(),
            Edges::all(px(width)),
            color,
            solid,
        ));
    }
}

/// Snap a rectangle to whole device pixels.
///
/// Without this, a mosaic of a few thousand adjacent rectangles shows seams
/// between them, because each is rasterised into its own pixel span.
fn snap(bounds: Bounds<Pixels>, scale: f32) -> Bounds<Pixels> {
    let round = |value: Pixels| px((value.as_f32() * scale).round() / scale);
    let left = round(bounds.origin.x);
    let top = round(bounds.origin.y);
    let right = round(bounds.origin.x + bounds.size.width);
    let bottom = round(bounds.origin.y + bounds.size.height);
    Bounds::new(
        Point::new(left, top),
        Size::new((right - left).max(px(0.)), (bottom - top).max(px(0.))),
    )
}

/// Shape and paint the labels, each clipped to its own tile.
///
/// Labels are shaped here rather than composed as elements so they clip to their
/// tile: a `div` cannot clip its text to an arbitrary rectangle cheaply, and an
/// unclipped label bleeding across a tile boundary is exactly the artefact a
/// treemap must not have.
#[allow(clippy::too_many_arguments)]
fn paint_labels(
    labels: &[Label],
    bounds: Bounds<Pixels>,
    view: View,
    colors: &Colors,
    font: &SharedString,
    name_size: Pixels,
    size_size: Pixels,
    window: &mut Window,
    cx: &mut App,
) {
    let font = Font {
        family: font.clone(),
        ..Font::default()
    };
    let name_line_height = name_size * 1.35;
    let text_system = window.text_system().clone();
    let text_padding = px(3.);

    for label in labels {
        // A header label fills the band the layout reserved; a body label sits in
        // the tile's own corner, over nothing but its own fill.
        let owned = label.header.unwrap_or(label.rect);
        let rect = view.project(owned);
        if px(rect.w) < text_padding * 2.0 || px(rect.h) < name_size {
            continue;
        }
        let mask = to_window(&rect, bounds);
        let origin = Point::new(
            mask.origin.x + text_padding,
            mask.origin.y + name_size * 0.15,
        );

        let weight = if label.depth == 0 {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        };
        let name_run = TextRun {
            len: label.text.len(),
            font: Font {
                weight,
                ..font.clone()
            },
            color: colors.label[(label.depth.min(1)) as usize],
            ..TextRun::default()
        };
        let line = text_system.shape_line(
            SharedString::from(label.text.clone()),
            name_size,
            &[name_run],
            None,
        );

        window.with_content_mask(Some(ContentMask { bounds: mask }), |window| {
            let _ = line.paint(origin, name_line_height, TextAlign::Left, None, window, cx);

            if label.size_text.is_empty() {
                return;
            }

            // The size sits under the name, vertically centred against it.
            let size_run = TextRun {
                len: label.size_text.len(),
                font: font.clone(),
                color: colors.label_dim,
                ..TextRun::default()
            };
            let size_line = text_system.shape_line(
                SharedString::from(label.size_text.clone()),
                size_size,
                &[size_run],
                None,
            );
            let baseline = origin.y
                + ((line.ascent - line.descent) - (size_line.ascent - size_line.descent)) * 0.5;
            let size_origin = Point::new(origin.x, baseline);
            let _ = size_line.paint(
                size_origin,
                name_line_height,
                TextAlign::Left,
                None,
                window,
                cx,
            );
        });
    }
}

/// Base space to window space.
fn to_window(rect: &disk_tree_core::treemap::Rect, bounds: Bounds<Pixels>) -> Bounds<Pixels> {
    Bounds::new(
        Point::new(px(rect.x) + bounds.origin.x, px(rect.y) + bounds.origin.y),
        Size::new(px(rect.w), px(rect.h)),
    )
}
