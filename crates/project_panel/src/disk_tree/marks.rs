//! The marked set: what the user has picked out to act on.
//!
//! Marks are keyed by **absolute path**, not by crumbs. That is the whole design
//! and it is upstream's: a rescan rebuilds every index in the tree, so anything
//! addressed by index path would silently start pointing at a different
//! directory. A path survives a rescan, a rename of something above it does not
//! (and should not - the user marked that name), and it survives closing and
//! reopening the panel, which is what makes the review screen trustworthy.
//!
//! The cost is that every use has to turn a path back into a node. That is done
//! once, in [`super::review`], rather than in the painter.

use std::collections::HashSet;
use std::path::PathBuf;

/// A set of marked paths, plus an epoch the painter can compare cheaply.
#[derive(Default)]
pub(crate) struct Marks {
    pub(crate) paths: HashSet<PathBuf>,
    /// Bumped whenever the set changes - additions and removals alike, because
    /// both change what the painter draws - so it can skip the compare cheaply.
    pub(crate) epoch: u64,
}

impl Marks {
    /// Add or remove `path`. Returns whether it is now marked.
    pub(crate) fn toggle(&mut self, path: PathBuf) -> bool {
        let added = if self.paths.contains(&path) {
            self.paths.remove(&path);
            false
        } else {
            self.paths.insert(path);
            true
        };
        // Any change to the set is worth a repaint: a removal changes what the
        // painter draws just as much as an addition does.
        self.epoch += 1;
        added
    }

    /// Forget everything, and say so only if there was something to forget.
    pub(crate) fn clear(&mut self) {
        if !self.paths.is_empty() {
            self.paths.clear();
            self.epoch += 1;
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    pub(crate) fn len(&self) -> usize {
        self.paths.len()
    }
}