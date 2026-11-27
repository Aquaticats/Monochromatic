//! What:
//!  The latest readiness record and whether it admits an attach on this boot.
//! Why:
//!  A caller waits for this record rather than for an installed executable,
//!  so a record from another boot,
//!  a pool or checkout that is not ready,
//!  and a state the handler recorded as unknown are three different answers,
//!  and the one the handler could not read leaves the answer unknown because
//!  the handler itself refused to guess.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkReadiness(facts, liveBoot); // pass | finding | unknown
//! ```

/// One check's answer,
/// and the three constructors every check builds it from.
use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
/// The three states the handler records for a pool or a checkout.
use crate::worktree_clone_mount_protocol::ReadyState;
//region The readiness record

/// What:
///  The latest readiness record's fields,
///  or why none was read.
/// Why:
///  A record from another boot or another pool admits nothing,
///  and the checks need each state beside the boot the record names,
///  so the whole record travels rather than a boolean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadinessFacts = { kind: 'read'; boot; pool; checkout; sweep } | { kind: 'absent'; why } | { kind: 'unreadable'; why };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadinessFacts {
    /// The record parsed.
    Read {
        /// The boot the record belongs to.
        boot: String,
        /// Whether the pool was ready.
        pool: ReadyState,
        /// Whether the checkout was ready.
        checkout: ReadyState,
        /// What the sweep was doing,
        /// which gates no attach.
        sweep: String,
    },
    /// No record is filed,
    /// or it was filed for another boot.
    Absent {
        /// What the read answered.
        why: String,
    },
    /// The record could not be read.
    Unreadable {
        /// What the read answered.
        why: String,
    },
}

/// What:
///  Whether this boot's handler published a readiness record that admits an
///  attach.
/// Why:
///  A caller waits for this record rather than for an installed executable,
///  so a record from another boot or a pool or checkout that is not ready is a
///  finding,
///  and one state the handler could not read leaves the answer unknown
///  because the handler itself refused to guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = check_readiness({ kind: 'read', boot, pool, checkout, sweep }, liveBoot); // pass | finding('boot'|'pool'|'checkout') | unknown
/// ```
pub fn check_readiness(
    facts: &ReadinessFacts,
    live_boot: &str,
) -> CheckOutcome {
    return match facts {
        ReadinessFacts::Absent {
            why,
        } => finding(
            "readiness",
            format!(
                "no readiness record answers this boot ({why}), so a caller waits for nothing; the handler writes it on its first qualifying run"
            ),
        ),
        ReadinessFacts::Unreadable {
            why,
        } => unknown(format!("the readiness record could not be read ({why})")),
        ReadinessFacts::Read {
            boot,
            pool,
            checkout,
            sweep,
        } => {
            if boot != live_boot {
                return finding(
                    "boot",
                    format!(
                        "the readiness record belongs to boot {boot} and this boot is {live_boot}, so it admits no attach here"
                    ),
                );
            }
            if *pool == ReadyState::Unknown || *checkout == ReadyState::Unknown {
                let unknown_state: &str = if *pool == ReadyState::Unknown {
                    "pool"
                } else {
                    "checkout"
                };
                return unknown(format!(
                    "the handler could not read the {unknown_state}, and it records that as unknown rather than guessing; no attach is admitted on it"
                ));
            }
            if *pool != ReadyState::Ready {
                return finding(
                    "pool",
                    format!(
                        "the record says the pool is {}, and only a ready pool admits an attach",
                        pool.word()
                    ),
                );
            }
            if *checkout != ReadyState::Ready {
                return finding(
                    "checkout",
                    format!(
                        "the record says the checkout is {}, and only a ready checkout admits an attach",
                        checkout.word()
                    ),
                );
            }
            pass(format!(
                "this boot's record says the pool and checkout are ready (sweep: {sweep})"
            ))
        }
    };
}

//endregion
