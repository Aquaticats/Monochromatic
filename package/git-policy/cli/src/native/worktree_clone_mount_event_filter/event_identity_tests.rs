//! What:
//!  Controls of the two names one ZED delivery carries:
//!  the variables ZED sets them from,
//!  a delivery that carried both,
//!  one that carried neither,
//!  and an empty value that is no name at all.
//! Why:
//!  The filter decides on these two strings alone,
//!  so the reading has to be right about the names ZED writes and about the
//!  value that is not a name
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  veto entry V2).
//!  The values are handed in through a lookup rather than set in this process,
//!  because the environment belongs to the whole test process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(EventIdentity.read(lookup).dataset()).toBe('zpcachyos/nosnap/clones/one');
//! ```

use super::{
    DATASET_VARIABLE,
    EventIdentity,
    POOL_VARIABLE,
};
use std::collections::BTreeMap;
use std::ffi::OsString;

/// The pool the route's parent lives in,
/// as every control of this folder names it.
const POOL: &str = "zpcachyos";

/// The dataset one clone has,
/// as every control of this folder names it.
const DATASET: &str = "zpcachyos/nosnap/zfsroute-clones/one";

/// What:
///  A lookup that answers the names it was given and nothing else.
/// Why:
///   A control hands in its own delivery without touching this process's
///  environment,
///  and a variable the control did not name stays absent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lookup = (names: Record<string, string>) => (name: string) => names[name];
/// ```
fn lookup_of(names: &[(&'static str, &str)]) -> impl Fn(&str) -> Option<OsString> {
    let mut values: BTreeMap<String, String> = BTreeMap::new();
    for (name, value) in names {
        values.insert(String::from(*name), String::from(*value));
    }
    return move |name: &str| return values.get(name).cloned().map(OsString::from);
}

/// The two variable names are the ones the distribution's own zedlets read.
/// Why:
///  a wrong spelling reads nothing and drops every real request in silence,
///  so the two names are pinned here against the zedlets installed beside them
///  (`/etc/zfs/zed.d/all-syslog.sh:24` and
///  `/etc/zfs/zed.d/history_event-zfs-list-cacher.sh:7,20`,
///  `zfs-utils` 2.4.4-3).
#[test]
fn the_two_variables_are_the_ones_the_installed_zedlets_read() {
    assert_eq!(POOL_VARIABLE, "ZEVENT_POOL");
    assert_eq!(DATASET_VARIABLE, "ZEVENT_HISTORY_DSNAME");
}

/// A delivery that carried both names reads them back.
#[test]
fn a_delivery_reads_its_pool_and_its_dataset() {
    let lookup = lookup_of(&[(POOL_VARIABLE, POOL), (DATASET_VARIABLE, DATASET)]);
    let event: EventIdentity = EventIdentity::read(&lookup);
    assert_eq!(event.pool(), Some(POOL));
    assert_eq!(event.dataset(), Some(DATASET));
}

/// A delivery that carried neither names nothing.
#[test]
fn a_delivery_that_named_nothing_reads_no_name() {
    let lookup = lookup_of(&[]);
    let event: EventIdentity = EventIdentity::read(&lookup);
    assert_eq!(event.pool(), None);
    assert_eq!(event.dataset(), None);
}

/// An empty value is no name,
/// which is what a cleared environment leaves.
#[test]
fn an_empty_value_is_no_name() {
    let event: EventIdentity = EventIdentity::of(Some(""), Some(""));
    assert_eq!(event.pool(), None);
    assert_eq!(event.dataset(), None);
}

/// A snapshot's own name stays on the delivery's dataset,
/// because taking it off is the rule's job and the control below pins it once.
#[test]
fn the_delivery_keeps_a_snapshots_suffix() {
    let snapshot: String = format!("{DATASET}@auto-2026-11-27");
    let event: EventIdentity = EventIdentity::of(Some(POOL), Some(snapshot.as_str()));
    assert_eq!(event.dataset(), Some(snapshot.as_str()));
}
