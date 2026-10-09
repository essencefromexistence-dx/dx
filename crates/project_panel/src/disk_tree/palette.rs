//! Colour that means something.
//!
//! A tile's hue says what kind of data it is ([`Category`]); every hue sits at
//! the same muted saturation and lightness, so no block stands out by accident,
//! and deeper tiles lift slightly so nesting reads without borders. Reclaimable
//! space is a hatch, not a colour, so "what is it" and "can it go" are read
//! independently.
//!
//! One strong colour is kept apart: the highlight, the theme's warning amber.
//! It marks the selection, the main action, reclaimable totals and the free
//! space after a removal, and nothing else, so the eye goes straight to it.
//!
//! Lightness, saturation and the surface every fill is pulled toward come from
//! the active theme, so the mosaic sits inside it, light or dark.
//!
//! Vendored from `disktree-app/src/palette.rs` at tobi/disktree@7c0661c
//! (v0.10.1). **Every hue, ladder, ramp and mixing rule is unchanged.** The only
//! edit is the parameter type: `&gpui_omarchy::Theme` became
//! `&DiskTreeTokens` (see `theme.rs` for the role-by-role mapping). Upstream's
//! seven theme reads map as: `inset` -> `surface`, `background` -> `background`,
//! `bright` -> `text`, `foreground` -> `text_muted`, `accent` -> `accent`,
//! `warning` -> `highlight`, `appearance` -> `is_dark()`.
//!
//! The property tests in `palette_tests.rs` are upstream's, moved here so this
//! file stays under the project's file-size ceiling. They are the palette's real
//! specification - if one of them fails, the colour rules changed, not the
//! expectations.

use disk_tree_core::classify::Category;
use gpui::{Hsla, Rgba};

use crate::disk_tree::theme::DiskTreeTokens;

/// The hue a category is drawn in, and how much colour it carries. The
/// neutral kinds (documents, unknown) carry almost none.
const fn hue(category: Category) -> (f32, f32) {
    match category {
        Category::Code => (0.605, 1.0),
        Category::AgentScratch => (0.065, 1.0),
        Category::Toolchain => (0.415, 1.0),
        Category::Synced => (0.535, 1.0),
        Category::Git => (0.955, 1.0),
        Category::Media => (0.745, 1.0),
        Category::Cache => (0.125, 0.95),
        Category::Documents => (0.6, 0.18),
        Category::Other => (0.6, 0.08),
    }
}

/// The fill for a tile of `category`, `depth` levels into the view.
pub(crate) fn category_fill(
    tokens: &DiskTreeTokens,
    category: Category,
    depth: u32,
) -> Hsla {
    let (h, chroma) = hue(category);
    let step = depth.min(4) as f32;
    let (s, l) = if tokens.is_dark() {
        (0.26 * chroma, step.mul_add(0.028, 0.215))
    } else {
        (0.30 * chroma, step.mul_add(-0.03, 0.84))
    };
    // Pulled a little toward the theme surface, so each theme tints it.
    mix(
        Hsla { h, s, l, a: 1.0 },
        tokens.surface,
        0.12,
    )
}

/// The saturated version of a category's hue: the strip over a top-level
/// directory and the legend swatch.
pub(crate) fn category_accent(tokens: &DiskTreeTokens, category: Category) -> Hsla {
    let (h, chroma) = hue(category);
    let (s, l) = if tokens.is_dark() {
        (0.42 * chroma, 0.52)
    } else {
        (0.45 * chroma, 0.46)
    };
    Hsla { h, s, l, a: 1.0 }
}

/// The age ramp, newest first: this week, this month, this half-year, this
/// year, older.
pub(crate) const AGE_BUCKETS: [(i64, &str); 5] = [
    (7, "This week"),
    (30, "This month"),
    (182, "Six months"),
    (365, "This year"),
    (i64::MAX, "Older"),
];

/// Which [`AGE_BUCKETS`] entry an age in days falls in.
pub(crate) fn age_bucket(days: i64) -> usize {
    AGE_BUCKETS
        .iter()
        .position(|(limit, _)| days <= *limit)
        .unwrap_or(AGE_BUCKETS.len() - 1)
}

/// The fill for age mode: recent writes carry the theme accent, and colour
/// drains out of a tile as it goes untouched.
pub(crate) fn age_fill(tokens: &DiskTreeTokens, bucket: usize, depth: u32) -> Hsla {
    let fade = bucket.min(AGE_BUCKETS.len() - 1) as f32 / 4.0;
    let step = depth.min(4) as f32;
    let (s, l) = if tokens.is_dark() {
        (
            (1.0 - fade).mul_add(0.34, 0.03),
            step.mul_add(0.028, 0.29 - fade * 0.09),
        )
    } else {
        (
            (1.0 - fade).mul_add(0.36, 0.04),
            step.mul_add(-0.03, 0.74 + fade * 0.1),
        )
    };
    mix(
        Hsla {
            h: tokens.accent.h,
            s,
            l,
            a: 1.0,
        },
        tokens.surface,
        0.1,
    )
}

/// The age swatch for the legend.
pub(crate) fn age_accent(tokens: &DiskTreeTokens, bucket: usize) -> Hsla {
    let fade = bucket.min(AGE_BUCKETS.len() - 1) as f32 / 4.0;
    let l = if tokens.is_dark() {
        0.55 - fade * 0.25
    } else {
        0.45 + fade * 0.25
    };
    Hsla {
        h: tokens.accent.h,
        s: (1.0 - fade).mul_add(0.45, 0.04),
        l,
        a: 1.0,
    }
}

/// The one strong colour: selection, the main action, what can be had back./// Not a const fn, unlike upstream: is_dark matches on our Appearance, and a/// const fn cannot call one.
pub(crate) fn highlight(tokens: &DiskTreeTokens) -> Hsla {
    tokens.highlight
}

/// Text on a filled highlight.
pub(crate) fn on_highlight(tokens: &DiskTreeTokens) -> Hsla {
    if tokens.is_dark() {
        tokens.background
    } else {
        tokens.text
    }
}

/// The diagonal hatch over reclaimable space: quiet enough to leave the hue
/// readable, visible on every fill.
pub(crate) fn hatch(tokens: &DiskTreeTokens) -> Hsla {
    if tokens.is_dark() {
        tokens.text.opacity(0.16)
    } else {
        tokens.text_muted.opacity(0.18)
    }
}

/// Linear interpolation between two colours, in RGB: interpolating hue
/// would drag a colour around the wheel on its way to a grey.
pub(crate) fn mix(from: Hsla, to: Hsla, t: f32) -> Hsla {
    let t = t.clamp(0.0, 1.0);
    let lerp = |a: f32, b: f32| (b - a).mul_add(t, a);
    let (a, b) = (from.to_rgb(), to.to_rgb());
    Hsla::from(Rgba {
        r: lerp(a.r, b.r),
        g: lerp(a.g, b.g),
        b: lerp(a.b, b.b),
        a: lerp(a.a, b.a),
    })
}

/// A tile's name on top of its fill.
pub(crate) fn label_color(tokens: &DiskTreeTokens, depth: u32) -> Hsla {
    let base = if tokens.is_dark() {
        tokens.text
    } else {
        tokens.text_muted
    };
    if depth == 0 { base } else { base.opacity(0.88) }
}