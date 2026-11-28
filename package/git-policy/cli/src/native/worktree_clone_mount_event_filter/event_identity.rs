//! What:
//!  The two names one ZED delivery carries that decide whether the clone-mount
//!  route looks at it:
//!  the pool and the dataset,
//!  read from the environment ZED builds for a zedlet.
//! Why:
//!  ZED starts the installed zedlet for every history event of every pool,
//!  and the route wants only the events about its own clone parent and the
//!  datasets below it,
//!  so the filter needs those two names before it opens anything
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  the section "The event filter,
//! in brief",
//!  and veto entry V2 in the same file).
//!  ZED writes each pair of the event's nvlist into the child's environment as
//!  `ZEVENT_` and the pair's name in capitals,
//!  which is what the distribution's own zedlets read:
//!  `/etc/zfs/zed.d/history_event-zfs-list-cacher.sh` takes the pool from
//!  `ZEVENT_POOL` on its line 7 and the dataset from
//!  `ZEVENT_HISTORY_DSNAME` on its line 20,
//!  and `/etc/zfs/zed.d/all-syslog.sh` prints the pool from `ZEVENT_POOL` on
//!  its line 24
//!  (both installed by `zfs-utils` 2.4.4-3 on this machine).
//!  Nothing here opens a file,
//!  a device or a socket:
//!  a delivery that names nothing under the parent is dropped before the
//!  compatibility gate runs,
//!  which is the whole saving of the filter.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const event = EventIdentity.live(); // { pool: process.env.ZEVENT_POOL, dataset: process.env.ZEVENT_HISTORY_DSNAME }
//! ```

use std::ffi::OsString;

/// The variable ZED sets from the event's `pool` pair.
/// Value:
///  `ZEVENT_POOL`,
/// the name `/etc/zfs/zed.d/all-syslog.sh` prints and
///  `/etc/zfs/zed.d/history_event-zfs-list-cacher.sh` names a pool by.
pub const POOL_VARIABLE: &str = "ZEVENT_POOL";

/// The variable ZED sets from the event's `history_dsname` pair.
/// Value:
///  `ZEVENT_HISTORY_DSNAME`,
/// the name
///  `/etc/zfs/zed.d/history_event-zfs-list-cacher.sh` reads and strips a
///  snapshot's `@` suffix from.
pub const DATASET_VARIABLE: &str = "ZEVENT_HISTORY_DSNAME";

/// What:
///  The names one ZED delivery carries,
///  either of them absent when the event did not carry that pair or carried
///  an empty value.
/// Why:
///   The filter decides on these two strings alone,
///  so a delivery read once is one value a control can hand in without setting
///  any variable of this process,
///  and an absent name stays absent rather than becoming an empty name that no
///  dataset has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EventIdentity = { pool?: string; dataset?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventIdentity {
    /// The event's pool name,
    /// when it carried one.
    pool: Option<String>,
    /// The event's dataset name,
    /// with any `@snapshot` suffix still on it,
    /// when it carried one.
    dataset: Option<String>,
}

impl EventIdentity {
    /// What:
    ///  The names one delivery carries,
    ///  each absent when it is missing or empty.
    /// Why:
    ///   A body that reads ZED's environment and a control that hands in
    ///  fixture names make the same value,
    ///  so the rule is tested on the strings a real delivery gives it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const event = EventIdentity.of(process.env.ZEVENT_POOL, process.env.ZEVENT_HISTORY_DSNAME);
    /// ```
    pub fn of(
        pool: Option<&str>,
        dataset: Option<&str>,
    ) -> Self {
        return EventIdentity {
            pool: a_name(pool),
            dataset: a_name(dataset),
        };
    }

    /// What:
    ///  The names ZED handed this process.
    /// Why:
    ///   A ZED-started handler reads the delivery it was started for and
    ///  nothing else,
    ///  so the environment is its only source.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const event = EventIdentity.live();
    /// ```
    pub fn live() -> Self {
        return Self::read(&|name| return std::env::var_os(name));
    }

    /// What:
    ///  The names a lookup answers,
    ///  which is what a control hands in instead of setting this process's
    ///  environment.
    /// Why:
    ///   The environment belongs to the whole test process,
    ///  so a control that changed it would change every other control of the
    ///  run;
    ///  a lookup keeps the reading testable and the process untouched.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const event = EventIdentity.read((name) => fixtureEnvironment.get(name));
    /// ```
    pub fn read(lookup: &dyn Fn(&str) -> Option<OsString>) -> Self {
        let pool: Option<String> =
            lookup(POOL_VARIABLE).and_then(|value| return value.into_string().ok());
        let dataset: Option<String> =
            lookup(DATASET_VARIABLE).and_then(|value| return value.into_string().ok());
        return EventIdentity {
            pool: a_name(pool.as_deref()),
            dataset: a_name(dataset.as_deref()),
        };
    }

    /// The event's pool name,
    /// when it carried one.
    pub fn pool(&self) -> Option<&str> {
        return self.pool.as_deref();
    }

    /// The event's dataset name,
    /// with any `@snapshot` suffix still on it.
    pub fn dataset(&self) -> Option<&str> {
        return self.dataset.as_deref();
    }
}

/// What:
///  One name,
///  or nothing when the delivery carried no value or an empty one.
/// Why:
///  An empty variable is what a caller who cleared the environment leaves,
///  and an empty string is no dataset and no pool,
///  so it is read as absent rather than compared as a name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = (value?: string) => (value === undefined || value === '') ? undefined : value;
/// ```
fn a_name(value: Option<&str>) -> Option<String> {
    let value: &str = value?;
    if value.is_empty() {
        return None;
    }
    return Some(String::from(value));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "event_identity_tests.rs"]
mod tests;
