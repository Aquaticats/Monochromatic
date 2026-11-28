//! What:
//!  The event filter's rule:
//!  whether one ZED delivery is about the route's clone parent or a dataset
//!  below it,
//!  decided on the delivery's names alone.
//! Why:
//!  ZED forks the installed zedlet for every history event of every pool,
//!  including every sanoid snapshot of the checkout and the home datasets and
//!  every `zfs set` another tool makes,
//!  and one pass per such event is the cost the filter exists to hold
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "The event filter,
//! in brief";
//!  veto entry V2 of the same file).
//!  The rule is a reading of two strings:
//!  it opens no file,
//!  takes no claim and asks the compatibility gate nothing,
//!  so a dropped event costs the fork ZED already paid and nothing more,
//!  and `source_guard/event_filter_opens_nothing_tests.rs` refuses a call from
//!  this folder to the device,
//!  the gate or the run claim.
//!  Two kinds of delivery pass that the route's scope does not name:
//!  one whose parent name could not be read,
//!  and one that carried no pool or no dataset.
//!  Both are V2's own rule that the filter "never fails closed":
//!  a name the delivery did not carry proves nothing,
//!  and the handler's own checks run against live facts and refuse or pass.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const decision = decideAboutTheEvent(delivery, parentScope); // dropped unless it is the parent or below it
//! ```

use crate::worktree_clone_mount_event_filter::event_filter_decision::{
    DroppedWhy,
    FilterDecision,
    PassedWhy,
};
use crate::worktree_clone_mount_event_filter::event_identity::EventIdentity;
use crate::worktree_clone_mount_event_filter::parent_scope::ParentScope;

/// The character that starts a ZFS snapshot's name inside a dataset's.
/// Value:
///  `@`,
/// which `/etc/zfs/zed.d/history_event-zfs-list-cacher.sh` cuts a
///  delivery's dataset name at on its line 20 before it compares the dataset,
///  and which no dataset name may contain
///  (`zfs(8)`,
///  "Datasets").
pub const SNAPSHOT_MARKER: char = '@';

/// What:
///  Whether this delivery is the route's to act on.
/// Why:
///   See the file's own description:
///  the answer is decided from the delivery's pool and dataset and the parent's
///  two names,
///  and nothing else is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decision = decideAboutTheEvent({ pool: "zpcachyos", dataset: "zpcachyos/nosnap/clones/one@snap" }, parent);
/// ```
pub fn decide_about_the_event(
    event: &EventIdentity,
    parent: Option<&ParentScope>,
) -> FilterDecision {
    let Some(parent) = parent else {
        return FilterDecision::Passed(PassedWhy::TheParentCouldNotBeResolved);
    };
    let Some(pool) = event.pool() else {
        return FilterDecision::Passed(PassedWhy::TheEventNamedNoPool);
    };
    if pool != parent.pool() {
        return FilterDecision::Dropped(DroppedWhy::AnotherPool {
            parent_pool: String::from(parent.pool()),
            event_pool: String::from(pool),
        });
    }
    let Some(dataset) = event.dataset() else {
        return FilterDecision::Passed(PassedWhy::TheEventNamedNoDataset);
    };
    let named: &str = before_its_snapshot(dataset);
    if named == parent.dataset() {
        return FilterDecision::Passed(PassedWhy::TheParentsOwnDataset);
    }
    if is_below(named, parent.dataset()) {
        return FilterDecision::Passed(PassedWhy::BelowTheParent);
    }
    return FilterDecision::Dropped(DroppedWhy::OutsideTheParent {
        parent: String::from(parent.dataset()),
        dataset: String::from(named),
    });
}

/// What:
///  The dataset name a delivery named,
///  with a snapshot's own name taken off.
/// Why:
///   A snapshot of a clone is a change to the clone's own dataset,
///  and a `zfs set` on the clone and a snapshot of it are the same dataset's
///  events as far as the sweep is concerned;
///  the distribution's own list-cacher zedlet cuts the name the same way before
///  it decides,
/// at its line 20.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dataset = beforeItsSnapshot('zpcachyos/nosnap/clones/one@auto-2026-11-27');
/// ```
fn before_its_snapshot(dataset: &str) -> &str {
    return match dataset.split_once(SNAPSHOT_MARKER) {
        Some((named, _snapshot)) => named,
        None => dataset,
    };
}

/// What:
///  Whether a dataset name lies below another one.
/// Why:
///   A name that merely begins with the parent's characters is another dataset:
///  `…/clones-two` is no child of `…/clones`,
///  so the comparison is on the separator that follows the parent's name and
///  never on the letters alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const below = name.startsWith(parent + '/');
/// ```
fn is_below(
    name: &str,
    parent: &str,
) -> bool {
    return name
        .strip_prefix(parent)
        .is_some_and(|rest| return rest.starts_with('/'));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "event_filter_scope_tests.rs"]
mod tests;
