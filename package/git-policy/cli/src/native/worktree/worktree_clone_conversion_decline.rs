//! What:
//!  How many tracked paths a rule of their own converts,
//!  and the share of them past which the rung declines instead of writing them through Git.
//! Why:
//!   The count is the decline rule's whole input,
//! read from the attributes alone before any blob is read and before anything is cloned,
//! so a source whose paths Git would have to write pays one query and Git's own checkout and no copy at all;
//! an entry's kind is read here because a link is never converted by its path's rules.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const converting = entries.filter((entry) => !isLink(entry) && checkoutConverts(attributes.get(entry.path), rules)).length;
//! ```

use crate::worktree_clean::StatEntry;
use crate::worktree_clone_conversion_rules::{ConversionRules, PathAttributes, checkout_converts};
use std::collections::HashMap;

/// The share of the tracked paths past which the rung declines instead of writing them through Git,
/// as `1 / DECLINE_SHARE_DIVISOR`.
/// Why this share:
///  the write step prices a path through Git's own rules at about 0.12 ms for a text path and 0.31 to 0.57 ms for an
///  LFS path across kept runs (`measure-recheck2-md-*` and `-real-*`,
///  `measure-gdc-real-*`;
/// both classes in one `git checkout-index`),
///  and the rule's threshold is above the break-even those prices give:
///  the fence stays under `main` while the paths it writes cost less than `main`'s checkout minus the fence's non-write
///  work,
/// which lies between this repository's 0.065 and the measured 0.199 share.
///  The part of that band the rule does not decline is a cost the proposal states.
///  One measured source above the threshold is the converting one (0.644 of its paths) and it declines.
pub const DECLINE_SHARE_DIVISOR: usize = 4;

/// The counted paths below which the share is not worth a decline.
/// Why:
///  the write of a thousand text paths (60 to 80 ms) is inside the run-to-run spread of an add this size,
/// so a small repository with a converting rule should keep its clone rung,
/// and the class controls of four paths must still reach the fence.
pub const DECLINE_PATHS_FLOOR: usize = 1000;


/// What:
///  Whether the rung declines the add instead of writing its converting paths through Git.
/// Why:
///   The rule is two boundaries a control can stand on:
/// a repository below the path floor keeps its rung whatever its
/// share,
/// and a share must be strictly more than `1 / DECLINE_SHARE_DIVISOR` of the tracked paths.
/// The share is counted from the attributes alone,
/// before any blob is read and before anything is cloned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const declines = (converting, tracked) => converting > DECLINE_PATHS_FLOOR && converting * DECLINE_SHARE_DIVISOR > tracked;
/// ```
pub fn declines(converting: usize, tracked: usize) -> bool {
    return converting > DECLINE_PATHS_FLOOR && converting * DECLINE_SHARE_DIVISOR > tracked;
}


/// The mode masks that name an entry's type,
/// and the type a symbolic link has.
pub(crate) const MODE_TYPE_MASK: u32 = 0o170000;
pub(crate) const MODE_SYMLINK: u32 = 0o120000;


/// What:
///  How many tracked paths a rule of their own would convert,
/// from the attributes and the configuration alone.
/// Why:
///   This is the count the decline rule prices before it reads a blob,
/// and a link is never converted by its path's
/// rules;
/// a path Git answers nothing for keeps the defaults,
/// which the configuration then decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const convertingPaths = (entries, attributes, rules) => entries.filter((entry) => !isLink(entry) && checkoutConverts(attributes.get(entry.path), rules)).length;
/// ```
pub fn converting_paths(entries: &[StatEntry], attributes: &HashMap<Vec<u8>, PathAttributes>, rules: &ConversionRules) -> usize {
    let mut converting: usize = 0;
    for entry in entries {
        let held: PathAttributes = attributes.get(entry.path.as_slice()).cloned().unwrap_or_default();
        if entry.mode & MODE_TYPE_MASK != MODE_SYMLINK && checkout_converts(&held, rules) {
            converting += 1;
        }
    }
    return converting;
}
