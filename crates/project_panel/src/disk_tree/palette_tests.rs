//! The palette's specification, as tests.
//!
//! Vendored from the `#[cfg(test)] mod tests` block in
//! `disktree-app/src/palette.rs:176-261` at tobi/disktree@7c0661c (v0.10.1),
//! moved into its own file so `palette.rs` stays under the project's
//! file-size ceiling. **Every assertion is upstream's, unchanged in substance.**
//!
//! One adaptation: upstream built its `Theme` from a real Omarchy theme
//! (`Theme::tokyo_night()` / `Theme::flexoki_light()`). There is no such thing
//! here, and a test that depends on whatever theme the developer has installed is
//! not a test. So these build [`DiskTreeTokens`] directly. That makes them
//! stronger than upstream's in one respect - they now also pin the behaviour of
//! *light* themes, which upstream only exercised incidentally through
//! `deeper_tiles_lift_away_from_the_background`.
//!
//! What each one protects:
//!
//! - `every_legend_category_has_its_own_hue` - no two legend entries read as one
//!   colour. If this fails, two categories became indistinguishable in the mosaic.
//! - `colourful_categories_share_one_level` - the saturated categories sit at one
//!   saturation/lightness, so no tile stands out by accident.
//! - `deeper_tiles_lift_away_from_the_background` - nesting reads without borders.
//! - `the_highlight_is_not_a_category_colour` - the one strong colour stays unique.
//! - `age_buckets_cover_every_age_in_order` - the ramp is monotonic.
//! - `mixing_toward_a_grey_keeps_the_hue` - the RGB-space lerp does not swing
//!   around the colour wheel.
//! - `mix_clamps_its_parameter` - `mix` is total.

use disk_tree_core::classify::Category;
use gpui::Hsla;
use theme::Appearance;

use crate::disk_tree::{
    palette::{
        AGE_BUCKETS, age_accent, age_bucket, age_fill, category_accent, category_fill, highlight,
        mix,
    },
    theme::DiskTreeTokens,
};

/// A fixed dark theme. The values are ours, but they only have to be *a* dark
/// theme: every assertion below is about relationships between colours, not
/// about the specific hexes, which is why they survive a theme change.
fn dark() -> DiskTreeTokens {
    DiskTreeTokens {
        surface: hsla(0.0, 0.0, 0.12),
        background: hsla(0.0, 0.0, 0.09),
        text: hsla(0.0, 0.0, 0.92),
        text_muted: hsla(0.0, 0.0, 0.70),
        accent: hsla(0.58, 0.70, 0.55),
        highlight: hsla(0.09, 0.85, 0.60),
        border: hsla(0.0, 0.0, 0.22),
        inset_border: hsla(0.0, 0.0, 0.13),
        appearance: Appearance::Dark,
    }
}

fn light() -> DiskTreeTokens {
    DiskTreeTokens {
        surface: hsla(0.0, 0.0, 0.99),
        background: hsla(0.0, 0.0, 1.0),
        text: hsla(0.0, 0.0, 0.15),
        text_muted: hsla(0.0, 0.0, 0.40),
        accent: hsla(0.58, 0.60, 0.50),
        highlight: hsla(0.09, 0.75, 0.45),
        border: hsla(0.0, 0.0, 0.80),
        inset_border: hsla(0.0, 0.0, 0.88),
        appearance: Appearance::Light,
    }
}

const fn hsla(h: f32, s: f32, l: f32) -> Hsla {
    Hsla { h, s, l, a: 1.0 }
}

#[test]
fn every_legend_category_has_its_own_hue() {
    let tokens = dark();
    let accents: Vec<Hsla> = Category::LEGEND
        .iter()
        .map(|&category| category_accent(&tokens, category))
        .collect();
    for (index, left) in accents.iter().enumerate() {
        for right in &accents[index + 1..] {
            let apart = (left.h - right.h).abs() > 0.03 || (left.s - right.s).abs() > 0.1;
            assert!(apart, "{left:?} and {right:?} read as one colour");
        }
    }
}

#[test]
fn colourful_categories_share_one_level() {
    let tokens = dark();
    let code = category_fill(&tokens, Category::Code, 0);
    let git = category_fill(&tokens, Category::Git, 0);
    assert!((code.l - git.l).abs() < 0.02);
    assert!((code.s - git.s).abs() < 0.03);
}

#[test]
fn deeper_tiles_lift_away_from_the_background() {
    for tokens in [dark(), light()] {
        let top = category_fill(&tokens, Category::Code, 0);
        let deep = category_fill(&tokens, Category::Code, 3);
        // Away from the background: lighter on dark, darker on light.
        let distance = |fill: Hsla| (fill.l - tokens.background.l).abs();
        assert!(
            distance(deep) > distance(top) + 0.05,
            "{:?}",
            tokens.appearance
        );
    }
}

#[test]
fn the_highlight_is_not_a_category_colour() {
    let tokens = dark();
    let highlight = highlight(&tokens);
    for category in Category::LEGEND {
        let fill = category_fill(&tokens, category, 0);
        assert!(highlight.s - fill.s > 0.2, "{category:?} competes");
    }
}

#[test]
fn age_buckets_cover_every_age_in_order() {
    assert_eq!(age_bucket(0), 0);
    assert_eq!(age_bucket(8), 1);
    assert_eq!(age_bucket(100), 2);
    assert_eq!(age_bucket(300), 3);
    assert_eq!(age_bucket(5000), 4);
    let tokens = dark();
    assert!(age_fill(&tokens, 0, 0).s > age_fill(&tokens, 4, 0).s);
}

#[test]
fn mixing_toward_a_grey_keeps_the_hue() {
    let tokens = dark();
    let orange = category_accent(&tokens, Category::AgentScratch);
    let mixed = mix(orange, tokens.surface, 0.3);
    assert!((mixed.h - orange.h).abs() < 0.02, "{mixed:?}");
}

#[test]
fn mix_clamps_its_parameter() {
    let tokens = dark();
    let clamped_low = mix(tokens.background, tokens.accent, -1.0).l;
    let clamped_high = mix(tokens.background, tokens.accent, 2.0).l;
    assert!((clamped_low - tokens.background.l).abs() < 1e-3);
    assert!((clamped_high - tokens.accent.l).abs() < 1e-3);
}

#[test]
fn the_age_ramp_is_monotonic_in_both_appearances() {
    // Upstream checked this on dark only. The ladder has separate dark and
    // light branches, so both are worth pinning.
    for tokens in [dark(), light()] {
        for pair in AGE_BUCKETS.windows(2) {
            let (newer, older) = (pair[0].0, pair[1].0);
            assert!(
                age_bucket(newer) < age_bucket(older),
                "{newer} should be newer than {older}"
            );
        }
        let swatches: Vec<f32> = (0..AGE_BUCKETS.len())
            .map(|bucket| age_accent(&tokens, bucket).s)
            .collect();
        for pair in swatches.windows(2) {
            assert!(
                pair[0] > pair[1],
                "age accents must drain as tiles go untouched: {swatches:?}"
            );
        }
    }
}

#[test]
fn depth_is_clamped_so_a_deep_tile_cannot_run_away() {
    // `depth.min(4)` in the ladders is what stops a pathologically deep tree
    // from producing a white tile on dark or a black one on light.
    let tokens = dark();
    let deepest = category_fill(&tokens, Category::Code, 4);
    for absurd in [5, 6, 40, 4000] {
        assert_eq!(
            category_fill(&tokens, Category::Code, absurd),
            deepest,
            "depth {absurd} should render identically to depth 4"
        );
    }
}

#[test]
fn is_dark_agrees_with_the_appearance_it_was_built_from() {
    assert!(dark().is_dark());
    assert!(!light().is_dark());
}
