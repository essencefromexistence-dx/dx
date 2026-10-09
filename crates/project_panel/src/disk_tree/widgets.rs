//! Small review-screen components, in this panel's own `ui` language.
//!
//! Upstream's widgets (`disktree-app/src/widgets.rs`) are not copied: they are
//! written against `gpui_omarchy`, which this repo does not have. What is kept
//! is the shape - a summary, one row per path, refused rows that say why, and a
//! separate irreversible-action confirmation - rebuilt from `gpui` and `ui`.
//!
//! The one deliberate mapping is the permanent-delete confirmation: upstream's
//! `gpui_omarchy::alert_dialog` becomes `ui::Modal` (re-exported flat from
//! `ui`'s private `components` module - `crates/ui/src/ui.rs:11,19`). Keys
//! still drive it, because a panel action must stay keyboard-first; the modal is
//! the visual confirmation, not a second consent mechanism.

use std::path::Path;

use gpui::{AnyElement, IntoElement, prelude::*};
use ui::{Modal, ModalHeader, Section};

/// A titled block: summary text above, rows below.
pub(crate) fn section(title: &str, body: AnyElement) -> AnyElement {
    gpui::div()
        .flex()
        .flex_col()
        .gap_1()
        .p_2()
        .child(title.to_owned())
        .child(body)
        .into_any_element()
}

/// One path and what acting on it means.
pub(crate) fn path_row(path: &Path, detail: String, dimmed: bool) -> AnyElement {
    gpui::div()
        .flex()
        .gap_2()
        .px_2()
        .py_1()
        .when(dimmed, |this| this.opacity(0.6))
        .child(path.display().to_string())
        .child(detail)
        .into_any_element()
}

/// A refused path: the reason first, because that is what the user must read.
pub(crate) fn warning_row(reason: String, path: &Path) -> AnyElement {
    gpui::div()
        .flex()
        .gap_2()
        .px_2()
        .py_1()
        .child(reason)
        .child(path.display().to_string())
        .into_any_element()
}

/// One key binding, in the `keys: action` shape the review footer uses.
pub(crate) fn key_hint(keys: &str, action: &str) -> AnyElement {
    gpui::div()
        .px_2()
        .py_1()
        .opacity(0.7)
        .child(format!("{keys}: {action}"))
        .into_any_element()
}

/// The permanent-mode confirmation, as a modal rather than an alert dialog.
///
/// It is shown, not popped up: the review screen is already the confirmation
/// surface, and a second overlapping dialog would hide the plan it is asking
/// about. `enter` still confirms and `r` still runs; this is the visual form of
/// that warning.
pub(crate) fn permanent_confirm_modal(confirmed: bool) -> AnyElement {
    let state = if confirmed {
        "Confirmed. Press r to delete permanently."
    } else {
        "Not confirmed. Press enter to confirm, escape to go back."
    };
    Modal::new("disk-tree-permanent-confirm", None)
        .header(
            ModalHeader::new()
                .headline("Delete permanently")
                .description("This cannot be undone. The trash is not involved."),
        )
        .section(
            Section::new_contained()
                .child("The paths below will be removed for good.".to_owned())
                .child(state.to_owned()),
        )
        .into_any_element()
}

/// `12 MiB` rather than `12582912`, because this is read at a glance.
pub(crate) fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
