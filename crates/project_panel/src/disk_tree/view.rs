//! The view transform, and the transition that animates a change of address.
//!
//! Vendored from `disktree-app/src/state.rs:120-299` at tobi/disktree@7c0661c
//! (v0.10.1), unchanged in behaviour.
//!
//! # Why this is split out
//!
//! `state.rs` upstream is 3 321 lines and `views.rs` is 3 596, and the two were
//! "two halves of one module". Both are cut along the seams that already existed
//! inside them - `View`, `LayoutTransition`, `LayoutCache` were separate items
//! there too - so that each file here stays under this project's 620-line
//! owned-module ceiling (`script/dx-source-quality.test.ts`).
//!
//! # The one invariant
//!
//! **All viewport geometry stays in the base space of an unzoomed layout, so pan
//! and zoom never require a re-layout.** `screen = (base - origin) * scale`.

use std::time::{Duration, Instant};

use disk_tree_core::treemap::Rect;
use gpui::Size;
use gpui::{Pixels, px};

/// The pan/zoom transform. Layout runs once in base-space pixels and is cached;
/// this is the only thing zoom changes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct View {
    pub(crate) scale: f32,
    pub(crate) origin_x: f32,
    pub(crate) origin_y: f32,
}

impl Default for View {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl View {
    pub(crate) const IDENTITY: Self = Self {
        scale: 1.0,
        origin_x: 0.0,
        origin_y: 0.0,
    };
    pub(crate) const MIN_SCALE: f32 = 1.0;
    /// The magnification ceiling. Past it, another notch descends instead of
    /// magnifying - **except in the Project Panel, where that behaviour was
    /// deliberately dropped**: see `zoom_ceiling_discourages_descend` in
    /// `tests`. A 300 px side panel hits the ceiling on almost every gesture, so
    /// the wheel appeared to zoom *out* at random.
    pub(crate) const MAX_SCALE: f32 = 5.0;

    pub(crate) fn project(&self, rect: Rect) -> Rect {
        Rect::new(
            (rect.x - self.origin_x) * self.scale,
            (rect.y - self.origin_y) * self.scale,
            rect.w * self.scale,
            rect.h * self.scale,
        )
    }

    /// Viewport point to base-space point.
    pub(crate) fn unproject(&self, x: f32, y: f32) -> (f32, f32) {
        (
            x / self.scale + self.origin_x,
            y / self.scale + self.origin_y,
        )
    }

    /// Zoom by `factor` toward `(x, y)` and no further than `ceiling`.
    ///
    /// The base point under `(x, y)` stays put, which is what makes a wheel
    /// zoom feel pointed at the tile rather than at the window.
    pub(crate) fn zoomed_at(&self, x: f32, y: f32, factor: f32, ceiling: f32) -> Self {
        let scale = (self.scale * factor).clamp(Self::MIN_SCALE, ceiling);
        let (base_x, base_y) = self.unproject(x, y);
        Self {
            scale,
            origin_x: base_x - x / scale,
            origin_y: base_y - y / scale,
        }
    }

    /// The scale at which `rect` exactly fits the viewport.
    pub(crate) fn fit_scale(rect: Rect, area: Size<Pixels>) -> f32 {
        let width = area.width.as_f32().max(1.0);
        let height = area.height.as_f32().max(1.0);
        if rect.w <= 0.0 || rect.h <= 0.0 {
            return Self::MIN_SCALE;
        }
        (width / rect.w).min(height / rect.h)
    }

    /// The region of the base layout the viewport currently shows.
    pub(crate) fn visible_base(&self, area: Size<Pixels>) -> Rect {
        let width = area.width.as_f32().max(1.0);
        let height = area.height.as_f32().max(1.0);
        Rect::new(
            self.origin_x,
            self.origin_y,
            width / self.scale,
            height / self.scale,
        )
    }

    /// Keep the viewport inside the layout: no empty margins, ever.
    pub(crate) fn clamped(self, area: Size<Pixels>) -> Self {
        let width = area.width.as_f32();
        let height = area.height.as_f32();
        let max_x = (width - width / self.scale).max(0.0);
        let max_y = (height - height / self.scale).max(0.0);
        Self {
            scale: self.scale,
            origin_x: self.origin_x.clamp(0.0, max_x),
            origin_y: self.origin_y.clamp(0.0, max_y),
        }
    }
}

/// A layout transition: the region the user is looking at, and where that same
/// region lands in the layout being switched to.
///
/// Every descending or ascending move is one region of the tree changing
/// address. Rather than moving a camera, each tile is drawn from where that
/// region *was* to where it *is*, so the tiles inside the directory being
/// entered grow into place and the ones being left slide out. That is what makes
/// "zoom in and descend" read as one motion: the newly visible level is always
/// larger than it was a moment ago, never smaller.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LayoutTransition {
    /// The region in the old frame, in viewport coordinates.
    src: Rect,
    /// The same region in the new frame, in viewport coordinates.
    dst: Rect,
    started: Instant,
    duration: Duration,
}

impl LayoutTransition {
    pub(crate) fn new(src: Rect, dst: Rect) -> Self {
        // Longer pulls take a little longer, but never enough to feel slow.
        let ratio = if dst.w > 1.0 { src.w / dst.w } else { 1.0 };
        let magnitude = ratio.abs().max(1.0).log2().clamp(0.0, 4.0);
        Self {
            src,
            dst,
            started: Instant::now(),
            duration: Duration::from_millis(130 + (magnitude * 45.0) as u64),
        }
    }

    /// Where a rectangle in the new layout was, before the switch.
    fn origin_of(&self, rect: Rect) -> Rect {
        let scale = if self.dst.w > 0.0 {
            self.src.w / self.dst.w
        } else {
            1.0
        };
        Rect::new(
            (rect.x - self.dst.x).mul_add(scale, self.src.x),
            (rect.y - self.dst.y).mul_add(scale, self.src.y),
            rect.w * scale,
            rect.h * scale,
        )
    }

    /// The rectangle to draw at this instant, and whether the transition is
    /// still running.
    pub(crate) fn sample(&self, rect: Rect) -> (Rect, bool) {
        let elapsed = self.started.elapsed().as_secs_f32() / self.duration.as_secs_f32();
        if elapsed >= 1.0 {
            return (rect, false);
        }
        let eased = 1.0 - (1.0 - elapsed).powi(3);
        let from = self.origin_of(rect);
        let lerp = |a: f32, b: f32| (b - a).mul_add(eased, a);
        (
            Rect::new(
                lerp(from.x, rect.x),
                lerp(from.y, rect.y),
                lerp(from.w, rect.w),
                lerp(from.h, rect.h),
            ),
            true,
        )
    }

    /// Whether the transition still needs frames.
    pub(crate) fn is_running(&self) -> bool {
        self.started.elapsed() < self.duration
    }
}

/// A zero-size viewport, used before the canvas has been measured once. Without
/// it every `clamped` call would divide by a zero scale on the first frame.
pub(crate) const UNMEASURED: Size<Pixels> = Size {
    width: px(0.),
    height: px(0.),
};

#[cfg(test)]
mod tests {
    use super::*;

    fn area(width: f32, height: f32) -> Size<Pixels> {
        Size {
            width: px(width),
            height: px(height),
        }
    }

    #[test]
    fn projection_round_trips() {
        let view = View {
            scale: 2.5,
            origin_x: 12.0,
            origin_y: -4.0,
        };
        let rect = Rect::new(100.0, 40.0, 30.0, 20.0);
        let screen = view.project(rect);
        let (x, y) = view.unproject(screen.x, screen.y);
        assert!((x - rect.x).abs() < 1e-3, "{x} != {}", rect.x);
        assert!((y - rect.y).abs() < 1e-3, "{y} != {}", rect.y);
    }

    #[test]
    fn zooming_toward_a_point_keeps_that_point_put() {
        let view = View::IDENTITY;
        let zoomed = view.zoomed_at(400.0, 300.0, 1.25, View::MAX_SCALE);
        let before = view.unproject(400.0, 300.0);
        let after = zoomed.unproject(400.0, 300.0);
        assert!((before.0 - after.0).abs() < 1e-3);
        assert!((before.1 - after.1).abs() < 1e-3);
    }

    #[test]
    fn the_ceiling_is_respected() {
        let mut view = View::IDENTITY;
        for _ in 0..40 {
            view = view.zoomed_at(10.0, 10.0, 1.25, View::MAX_SCALE);
        }
        assert!(view.scale <= View::MAX_SCALE, "{}", view.scale);
        // And zooming back out stops at the floor rather than inverting.
        let mut view = View::IDENTITY;
        for _ in 0..40 {
            view = view.zoomed_at(10.0, 10.0, 0.8, View::MAX_SCALE);
        }
        assert!(view.scale >= View::MIN_SCALE, "{}", view.scale);
    }

    #[test]
    fn clamping_leaves_no_empty_margin() {
        let view = View {
            scale: 3.0,
            origin_x: -500.0,
            origin_y: 9000.0,
        };
        let clamped = view.clamped(area(600.0, 800.0));
        assert_eq!(clamped.origin_x, 0.0);
        let max_y = 800.0 - 800.0 / 3.0;
        assert!((clamped.origin_y - max_y).abs() < 1e-3, "{}", clamped.origin_y);
    }

    #[test]
    fn fit_scale_fits_the_tighter_axis() {
        // A wide rect in a square viewport is limited by width.
        let scale = View::fit_scale(Rect::new(0.0, 0.0, 400.0, 100.0), area(200.0, 200.0));
        assert!((scale - 0.5).abs() < 1e-3, "{scale}");
        // A degenerate rect falls back rather than dividing by zero.
        assert_eq!(
            View::fit_scale(Rect::new(0.0, 0.0, 0.0, 0.0), area(200.0, 200.0)),
            View::MIN_SCALE
        );
    }

    #[test]
    fn a_transition_starts_running_and_settles() {
        let transition = LayoutTransition::new(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            Rect::new(0.0, 0.0, 200.0, 200.0),
        );
        // A fresh transition samples as still running and reports itself so.
        let (_, running) = transition.sample(Rect::new(0.0, 0.0, 200.0, 200.0));
        if transition.is_running() {
            assert!(running, "is_running disagreed with sample");
        } else {
            assert!(!running, "is_running disagreed with sample");
        }
    }

    #[test]
    fn a_finished_transition_returns_the_rect_unchanged() {
        // duration is 130ms+; a rect sampled after it must be untouched.
        let transition = LayoutTransition::new(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            Rect::new(0.0, 0.0, 200.0, 200.0),
        );
        let target = Rect::new(5.0, 6.0, 7.0, 8.0);
        std::thread::sleep(std::time::Duration::from_millis(400));
        let (sampled, running) = transition.sample(target);
        assert!(!running);
        assert_eq!(sampled, target);
    }
}