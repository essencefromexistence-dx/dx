//! Review-screen overlays: help, history, and removal progress.
//!
//! These are rows in the review list, not floating windows. The point of the
//! review screen is that every refused path stays visible, so nothing here may
//! cover the plan: help goes in the footer, history goes above the rows, and
//! progress replaces the rows only while the worker owns them.

use gpui::{AnyElement, IntoElement, prelude::*};

use super::widgets;

/// What the review screen's keys do, in one footer block.
pub(crate) fn review_help() -> AnyElement {
    let hints = [
        widgets::key_hint("enter", "confirm"),
        widgets::key_hint("t", "move to trash"),
        widgets::key_hint("d", "delete permanently"),
        widgets::key_hint("r", "run the confirmed removal"),
        widgets::key_hint("escape", "back to the treemap"),
    ];
    widgets::section(
        "Keys",
        gpui::div().flex().flex_col().children(hints).into_any_element(),
    )
}

/// Where the back/forward history stands, for the card upstream showed.
///
/// Crumbs here are index paths, not names, so the card reports position rather
/// than pretending to label entries it cannot name without the tree.
pub(crate) fn history_card(at: usize, len: usize) -> AnyElement {
    let text = if len == 0 {
        "No history yet. Descend once to start one.".to_owned()
    } else {
        format!(
            "History: position {} of {}.",
            at.saturating_add(1).min(len),
            len
        )
    };
    widgets::section(
        "History",
        gpui::div().px_2().py_1().child(text).into_any_element(),
    )
}

/// What a running removal has done so far, in its own terms.
pub(crate) fn removal_progress(removed: u64, freed: u64, failed: usize) -> AnyElement {
    let body = gpui::div()
        .flex()
        .flex_col()
        .child(format!("{removed} removed"))
        .child(format!("{} freed", widgets::human_bytes(freed)))
        .child(format!("{failed} failed"))
        .child(widgets::key_hint("escape/q", "ask the worker to stop"))
        .into_any_element();
    widgets::section("Removing", body)
}
