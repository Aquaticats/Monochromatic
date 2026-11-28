//! What:
//!  The latest readiness record the handler published,
//!  with the boot,
//!  the pool and the checkout state it names,
//!  or why none was read.
//! Why:
//!  A caller waits for this record rather than for an installed executable,
//!  so a record from another boot or another pool admits nothing and is a
//!  finding a person acts on
//!  (`crate::doctor_zfs_route::handler_checks_readiness`;
//!  plan step 18,
//!  "Requests,
//!  results and ordering").
//!  The record is read through the handler's own reader,
//!  which proves the folder's owner and mode as it opens it,
//!  so a record another account could rewrite decides nothing,
//!  and a folder that is not there is an absence rather than an unreadable
//!  record.
//!  Nothing here publishes a record or makes the folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const readiness = theReadinessFacts(readinessFile); // { kind: 'read', boot, pool, checkout, sweep } | { kind: 'absent', why } | { kind: 'unreadable', why }
//! ```

/// What the latest record answered,
/// with the states the check compares.
use crate::doctor_zfs_route::handler_checks_readiness::ReadinessFacts;
/// The handler's own reader of the record,
/// and its refusal.
use crate::worktree_clone_mount_protocol::ReadinessFile;
/// The record the reader parsed,
/// and the word its sweep state is written as.
use crate::worktree_clone_mount_protocol::ReadinessRecord;

/// What:
///  The latest readiness record's fields,
///  or which of the two ways it was not read.
/// Why:
///  A missing record and one this account could not read send a person to
///  different work,
///  and a record that parsed keeps the boot and the two states the check
///  judges.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = theReadinessFacts(ReadinessFile.live());
/// ```
pub(crate) fn the_readiness_facts(readiness: &ReadinessFile) -> ReadinessFacts {
    let record: Option<ReadinessRecord> = match readiness.read() {
        Ok(filed) => filed,
        Err(refusal) => {
            return ReadinessFacts::Unreadable {
                why: refusal.message(),
            };
        }
    };
    let Some(filed) = record else {
        // What:     `path` is the one file the handler publishes into, so the absence names the place a person
        //           looks for it.
        // Why:      A person reading "no record" needs the file the handler would have written.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { kind: 'absent', why: `no readiness record is filed at ${readiness.record()}` };
        // ```
        let path: std::path::PathBuf = readiness.record();
        return ReadinessFacts::Absent {
            why: format!(
                "no readiness record is filed at {}, so no boot's readiness is proved",
                path.display()
            ),
        };
    };
    return ReadinessFacts::Read {
        boot: filed.boot,
        pool: filed.pool_state,
        checkout: filed.checkout_state,
        sweep: String::from(filed.sweep_state.word()),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_readiness_record_read_tests.rs"]
mod tests;
