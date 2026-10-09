//! The seven theme values the palette actually reads, resolved once per frame.
//!
//! `palette.rs` is 200-odd lines of pure colour maths with no GPUI in it, and it
//! was written against `gpui_omarchy::Theme`. Ours is a different shape, so
//! rather than re-derive the palette against `theme::Theme` - which would drag
//! twelve colour roles and four helper methods through every signature - the
//! roles it uses are gathered here, once.
//!
//! That is the whole reason this file exists. Everything about *how* the colours
//! combine stays in `palette.rs`; this only answers "which of our tokens plays
//! the part Omarchy's `inset` used to".

use gpui::{App, Hsla};
use theme::ActiveTheme as _;
use theme::Appearance;

/// The theme values the disk tree's colour maths reads.
///
/// Resolved per frame rather than cached: the panel is a view, `cx.theme()` is a
/// pointer read, and a stale token after a theme switch is a bug nobody
/// reproduces twice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DiskTreeTokens {
    /// The panel surface every fill is mixed toward, so the mosaic sits inside
    /// the theme instead of on top of it. Omarchy's `inset`.
    pub(crate) surface: Hsla,
    /// The page behind the panel. Omarchy's `background`.
    pub(crate) background: Hsla,
    /// Primary text. Omarchy's `bright`.
    pub(crate) text: Hsla,
    /// Secondary text. Omarchy's `foreground`.
    pub(crate) text_muted: Hsla,
    /// The theme accent, used for hue only - the age ramp draws recent writes
    /// in it. Omarchy's `accent`.
    pub(crate) accent: Hsla,
    /// The one strong colour in the palette: selection, the main action,
    /// reclaimable totals, free space after a removal. Nothing else uses it.
    pub(crate) highlight: Hsla,
    /// The theme's borders, for tile outlines and chrome.
    pub(crate) border: Hsla,
    /// Hairlines that must stay quiet against a filled tile.
    pub(crate) inset_border: Hsla,
    /// Light or dark. The palette's lightness ladders branch on this.
    pub(crate) appearance: Appearance,
}

impl DiskTreeTokens {
    pub(crate) fn resolve(cx: &App) -> Self {
        let theme = cx.theme();
        let colors = theme.colors();

        Self {
            surface: colors.panel_background,
            background: colors.background,
            text: colors.text,
            text_muted: colors.text_muted,
            accent: theme.accents().color_for_index(0),
            highlight: theme.status().warning,
            border: colors.border,
            inset_border: colors.border.opacity(0.6),
            appearance: theme.appearance(),
        }
    }

    /// Whether the theme is dark. Every lightness ladder in `palette.rs` forks
    /// on this, so it is asked once here rather than matching on `Appearance`
    /// at each call site.
    pub(crate) fn is_dark(&self) -> bool {
        matches!(self.appearance, Appearance::Dark)
    }
}