//! What:
//!  Read one request's answer as the outcome of every name it submitted,
//! with the
//!  kernel's per-name list kept exactly as it answered.
//! Why:
//!  The three answers a write request can carry are kept apart:
//! a success,
//! a
//!  refusal with per-name failures,
//! and a failure with no per-name list at all.
//!  Only the first two let a name be settled,
//! and a list that cannot be the
//!  complete one leaves the names it does not mention unknown rather than
//!  intact,
//! because the kernel shortens a list too large for the caller's buffer
//!  (`module/zfs/zfs_ioctl.c`, `nvlist_smush`) and the binding's reader keeps
//!  at most [`MAXIMUM_ELEMENTS_KEPT`] entries
//!  (`requests/element_error.rs`).
//!  A reply that contradicts the request,
//! or a success that names a failure,
//!  settles nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = outcomeOfAnswer(pool, submitted, answer);
//! outcome.names(); // [{ name, outcome: { kind: 'destroyed' | 'refused' | 'intact' | 'unknown' } }]
//! ```

/// The failure a live call returned,
/// the kernel's per-name element type and
///  the binding reader's cap,
/// and the reply list and value types.
use monochromatic_zfs_request_to_openzfs_kernel_device::requests::{
    ElementError,
    MAXIMUM_ELEMENTS_KEPT,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DeviceError,
    Error,
    NvData,
    NvList,
};

/// One request's outcome as a value.
use super::snapshot_batch_outcome::BatchOutcome;
/// The per-name vocabulary the outcome is read into.
use super::snapshot_name_outcome::{
    NameOutcome,
    SubmittedName,
};

/// The kernel's own marker that its per-name list was shortened.
///
/// `ZPROP_N_MORE_ERRORS` is `"N_MORE_ERRORS"`
/// (`include/sys/fs/zfs.h`, OpenZFS `zfs-2.4.4`):
/// a list too large for the
///  caller's buffer is smushed,
/// the entries it lost are removed and one pair
///  named this carries their count
///  (`module/zfs/zfs_ioctl.c`, `nvlist_smush` and
///  `zfs_ioc_destroy_snaps`'s registration with `smush_outnvlist` set).
///  A full reply is what this route's write buffer holds,
/// so the marker should never appear;
/// if it does, the list is short and the names it does not
///  name are unknown rather than intact.
pub const MORE_ERRORS_KEY: &str = "N_MORE_ERRORS";

/// What:
///  Read one request's answer as the outcome of every name it submitted.
/// Why:
///  A success is every name destroyed,
/// a refusal is its named failures plus the
///  names the kernel's no-destruction-on-failure rule leaves intact,
/// and any
///  other answer is unknown for every name.
pub fn outcome_of_answer(
    pool: &str,
    submitted: &[String],
    answer: Result<Option<NvList>, DeviceError>,
) -> BatchOutcome {
    return match answer {
        Ok(reply) => from_success(pool, submitted, reply.as_ref()),
        Err(error) => from_failure(pool, submitted, error),
    };
}

/// The outcome of a request the kernel answered without an error.
fn from_success(
    pool: &str,
    submitted: &[String],
    reply: Option<&NvList>,
) -> BatchOutcome {
    let said: String = match reply.map(read_success_reply) {
        None => String::new(),
        Some(Ok(())) => String::new(),
        Some(Err(reason)) => reason,
    };
    if said.is_empty() {
        return every(
            pool,
            submitted,
            NameOutcome::Destroyed,
            Vec::new(),
            true,
            None,
        );
    }
    return every(
        pool,
        submitted,
        NameOutcome::Unknown {
            said,
        },
        Vec::new(),
        false,
        None,
    );
}

/// Whether a success reply names no failure at all.
///
/// A success answers the program's empty error table,
/// so a nonzero number or a
///  shortened-list marker in it contradicts the kernel,
/// and neither the success nor
///  the failure is trusted.
fn read_success_reply(reply: &NvList) -> Result<(), String> {
    let mut failures: usize = 0;
    for pair in reply.iter() {
        if pair.name == MORE_ERRORS_KEY {
            return Err(String::from(
                "the kernel answered success with a shortened per-name list, which a success reply never carries",
            ));
        }
        let errno: i32 = match &pair.data {
            NvData::Int32 {
                value,
            } => *value,
            NvData::Int64 {
                value,
            } => i32::try_from(*value).map_err(|_| {
                return format!(
                    "the success reply's `{}` holds {value}, which is not an error number",
                    pair.name
                );
            })?,
            NvData::Uint64 {
                value,
            } => i32::try_from(*value).map_err(|_| {
                return format!(
                    "the success reply's `{}` holds {value}, which is not an error number",
                    pair.name
                );
            })?,
            _ => {
                return Err(format!(
                    "the success reply's `{}` is not an error number, so this is not the per-name list this request answers",
                    pair.name
                ));
            }
        };
        if errno != 0 {
            failures += 1;
        }
    }
    if failures != 0 {
        return Err(format!(
            "the kernel answered success and named {failures} failed snapshot(s), so the answer is not trusted"
        ));
    }
    return Ok(());
}

/// The outcome of a request the kernel refused.
fn from_failure(
    pool: &str,
    submitted: &[String],
    error: DeviceError,
) -> BatchOutcome {
    let Some(Error::Elements {
        errno,
        elements,
        ..
    }) = error.request()
    else {
        let request_errno: Option<i32> = error.request().and_then(Error::errno);
        let said: String =
            format!("{error}, and its answer names no snapshot, so no name can be settled");
        return it_is_unknown(pool, submitted, said, request_errno);
    };
    let (failures, shortened): (Vec<ElementError>, bool) = split_failures(elements);
    let complete: bool = !shortened;
    let named: Vec<&ElementError> = failures
        .iter()
        .filter(|failure| {
            return !submitted.iter().any(|name| {
                return *name == failure.name;
            });
        })
        .collect();
    if !named.is_empty() {
        let said: String = format!(
            "the kernel's per-name list names `{}`, which this request did not submit, so no name can be settled",
            named[0].name
        );
        return every(
            pool,
            submitted,
            NameOutcome::Unknown {
                said,
            },
            failures,
            complete,
            Some(*errno),
        );
    }
    let names: Vec<SubmittedName> = submitted
        .iter()
        .map(|name| {
            let named_failure: Option<&ElementError> = failures.iter().find(|element| {
                return element.name == *name;
            });
            let outcome: NameOutcome = match named_failure {
                Some(element) => NameOutcome::Refused { errno: element.errno },
                None if shortened => NameOutcome::Unknown {
                    said: String::from(
                        "the kernel's per-name list was shortened, so it cannot show whether this name failed",
                    ),
                },
                None => NameOutcome::Intact,
            };
            return SubmittedName {
                name: name.clone(),
                outcome,
            };
        })
        .collect();
    return BatchOutcome::new(pool.to_string(), names, failures, complete, Some(*errno));
}

/// Split the reader's elements into the per-name failures and whether the list
/// is provably complete.
///
/// The kernel's shortened-list marker is not a snapshot name,
/// and the binding's
///  reader keeps at most [`MAXIMUM_ELEMENTS_KEPT`] entries,
///  so a list at that cap may
/// be short of the kernel's own.
fn split_failures(elements: &[ElementError]) -> (Vec<ElementError>, bool) {
    let mut shortened: bool = elements.len() >= MAXIMUM_ELEMENTS_KEPT;
    let mut failures: Vec<ElementError> = Vec::new();
    for element in elements {
        if element.name == MORE_ERRORS_KEY {
            shortened = true;
            continue;
        }
        failures.push(element.clone());
    }
    return (failures, shortened);
}

/// The outcome every submitted name shares,
/// with the request's own facts.
fn every(
    pool: &str,
    submitted: &[String],
    outcome: NameOutcome,
    failures: Vec<ElementError>,
    failures_complete: bool,
    errno: Option<i32>,
) -> BatchOutcome {
    let names: Vec<SubmittedName> = submitted
        .iter()
        .map(|name| {
            return SubmittedName {
                name: name.clone(),
                outcome: outcome.clone(),
            };
        })
        .collect();
    return BatchOutcome::new(pool.to_string(), names, failures, failures_complete, errno);
}

/// Say the same unknown outcome to every name.
fn it_is_unknown(
    pool: &str,
    submitted: &[String],
    said: String,
    errno: Option<i32>,
) -> BatchOutcome {
    return every(
        pool,
        submitted,
        NameOutcome::Unknown {
            said,
        },
        Vec::new(),
        false,
        errno,
    );
}

/// The controls of the per-name outcome.
#[cfg(test)]
#[path = "snapshot_batch_answer_tests.rs"]
mod tests;
