//! The model's tests.
//!
//! Split out of `model.rs` so both files stay under this project's 620-line
//! owned-module ceiling (`script/dx-source-quality.test.ts`).

//! Every assertion here is about a rule the rest of the disk tree relies on:
//! what changes a layout cache key, what counts as a real mark change, and what
//! a projection is allowed to claim.

// super is mod.rs, so the items under test are named explicitly.
use std::path::PathBuf;

use disk_tree_core::space::SpaceInfo;
use disk_tree_core::tree::Metric;

use super::marks::Marks;
use super::model::*;
use super::navigation::layout_options;

#[test]
fn color_mode_cycles_through_all_three() {
    let mut mode = ColorMode::Size;
    let mut seen = vec![mode];
    for _ in 0..2 {
        mode = mode.next();
        seen.push(mode);
    }
    assert_eq!(seen, ColorMode::ALL);
    assert_eq!(mode.next(), ColorMode::Size, "must wrap");
}

#[test]
fn only_files_mode_changes_the_metric() {
    assert_eq!(ColorMode::Size.metric(), Metric::Bytes);
    assert_eq!(ColorMode::Files.metric(), Metric::Files);
    // Age colours by mtime and leaves the areas alone.
    assert_eq!(ColorMode::Age.metric(), Metric::Bytes);
}

#[test]
fn filtered_fades_in_readable_steps() {
    assert_eq!(Filtered::Shown.fade(), 0.0);
    assert!(Filtered::Holds.fade() < Filtered::Out.fade());
    // A held match must stay readable: it is a directory containing a hit.
    assert!(Filtered::Holds.fade() < 0.7, "held tiles are still content");
}

#[test]
fn history_walks_back_and_forward_without_duplicating() {
    let mut history = History::default();
    assert!(history.back().is_none());
    assert!(history.forward().is_none());

    history.visit(vec![]);
    history.visit(vec![1]);
    history.visit(vec![1, 2]);
    assert!(history.can_go_back());
    assert!(!history.can_go_forward());

    // Revisiting the current entry is not a new place.
    history.visit(vec![1, 2]);
    assert_eq!(history.entries.len(), 3);

    assert_eq!(history.back(), Some(&[1][..]));
    assert_eq!(history.back(), Some(&[][..]));
    assert!(!history.can_go_back());
    assert_eq!(history.forward(), Some(&[1][..]));
    // `[1, 2]` is still ahead, because going back never forgets the tip.
    assert!(history.can_go_forward());
    assert_eq!(history.forward(), Some(&[1, 2][..]));
    assert!(!history.can_go_forward());
}

#[test]
fn going_back_from_the_end_discards_the_forward_branch() {
    let mut history = History::default();
    history.visit(vec![]);
    history.visit(vec![1]);
    history.visit(vec![2]);
    history.back();
    history.visit(vec![3]);
    assert!(!history.can_go_forward(), "the old branch is gone");
}

#[test]
fn marks_bump_the_epoch_only_on_a_real_change() {
    let mut marks = Marks::default();
    assert!(marks.is_empty());
    assert_eq!(marks.epoch, 0);

    assert!(marks.toggle(PathBuf::from("/a")));
    assert_eq!(marks.epoch, 1);
    assert_eq!(marks.len(), 1);

    // A removal is a change too: it changes what the painter draws.
    assert!(!marks.toggle(PathBuf::from("/a")));
    assert_eq!(marks.epoch, 2);
    assert!(marks.is_empty());

    marks.toggle(PathBuf::from("/b"));
    marks.clear();
    assert!(marks.is_empty());
    assert_eq!(marks.epoch, 4);
}

#[test]
fn a_removed_path_reclaims_only_what_can_be_measured() {
    let space = Space::default();
    assert_eq!(space.after_removing(1024), None, "no measurement, no claim");

    let space = Space {
        info: Some(SpaceInfo {
            total: 1000,
            free: 150,
            available: 100,
        }),
        baseline_free: Some(100),
    };
    assert_eq!(space.after_removing(250), Some(350));
}

#[test]
fn only_left_and_right_are_horizontal() {
    assert!(Direction::Left.horizontal());
    assert!(Direction::Right.horizontal());
    assert!(!Direction::Up.horizontal());
    assert!(!Direction::Down.horizontal());
    assert!(Direction::Down.forwards());
    assert!(!Direction::Up.forwards());
}

#[test]
fn the_layout_key_changes_when_the_filter_does() {
    let base = LayoutKey::new(&[0], 10.0, 10.0, layout_options(3, 16.0), 0);
    let mut bumped = base.clone();
    bumped.filter = 1;
    assert_ne!(base, bumped, "a filter change must drop the layout cache");

    let resized = LayoutKey::new(&[0], 11.0, 10.0, layout_options(3, 16.0), 0);
    assert_ne!(base, resized, "a resize must drop the layout cache");
}

#[test]
fn header_bands_scale_with_the_rem() {
    // A header band is in layout pixels, so it has to follow interface zoom
    // or the names detach from their directories at other rem sizes.
    let small = layout_options(3, 12.0);
    let large = layout_options(3, 24.0);
    assert!((large.header - small.header * 2.0).abs() < 1e-3);
    assert!((large.header_inner - small.header_inner * 2.0).abs() < 1e-3);
    assert!(small.header_inner < small.header, "inner bands are slimmer");
}
