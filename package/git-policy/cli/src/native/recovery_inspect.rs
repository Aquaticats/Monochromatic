//! What:
//!  Owner inspection of one transaction registry entry,
//!  and the oldest-first recovery order.
//! Why:
//!  Live owners are skipped,
//!  dead owners recovered,
//!  a published directory with a malformed owner record fails closed,
//!  and one without an owner record is recovered once abandoned (`ownerless_directory.rs`) and fails closed before;
//!  recovery runs oldest first because a later transaction was
//!      prepared against the state an earlier one left
//!      (`src/policy-engine/commit-transaction-recovery-inspect.ts`,
//!  `commit-landing-lock.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const inspected = oldestFirst(await Promise.all(entries.map(inspectRegistryEntry)));
//! ```

/// Debug diagnostics.
use super::diagnostic_log::debug;
/// Whether a directory without an owner record is abandoned, and the message when it is not.
use super::ownerless_directory::{OwnerlessStanding, ownerless_refusal, ownerless_standing};
/// Bounded no-follow reads.
use super::private_storage::{ReadRefusal, read_regular_file};
/// The fail-closed recovery failure.
use super::recovery_error::{RecoveryError, io_failure};
/// The recovery read limit and existence probes.
use super::recovery_files::{RECOVERY_FILE_LIMIT, recovery_path_exists};
/// Owner records and liveness.
use super::transaction_owner::{
    OwnerLiveness, OwnerLivenessError, TransactionOwner, classify_transaction_owner_record,
    parse_transaction_owner,
};
/// Registry entries.
use super::transaction_registry::{EntryKind, OWNER_FILENAME, RegistryEntry};
/// `PathBuf` is an owned filesystem path.
use std::path::{Path, PathBuf};

/// What:
///  An entry's owner evidence:
///  its record and liveness,
///  or why none can be attributed.
/// Why:
///   `InspectedEntry['owner']`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Owner = { record; liveness } | 'vanished' | 'unattributed';
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OwnerEvidence {
    /// A complete owner record and whether its process still runs.
    Owned(TransactionOwner, OwnerLiveness),
    /// The directory disappeared while recovery listed it:
    ///  its owner finished.
    Vanished,
    /// A staging candidate without a complete owner record.
    Unattributed,
    /// A published directory without an owner record that nothing has changed or used for the patience:
    ///  recovered like a dead owner's, with no process number to match a PID file against.
    Abandoned,
}

/// The process number recovery passes for an abandoned directory, whose owner record is lost.
///  No PID file names it, because a process number is at least one.
pub const NO_OWNER_PID: i64 = 0;

/// What:
///  One registry entry with its owner evidence.
/// Why:
///   `InspectedEntry`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InspectedEntry = { entry: TransactionRegistryEntry; owner: Owner };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectedEntry {
    /// The classified entry.
    pub entry: RegistryEntry,
    /// Its owner evidence.
    pub owner: OwnerEvidence,
}

/// Methods over an inspected entry.
impl InspectedEntry {
    /// What:
    ///  The owner record of a dead owner,
    ///  or nothing.
    /// Why:
    ///   Only dead owners are recovered.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// typeof owner !== 'string' && owner.liveness === 'dead' ? owner.record : undefined
    /// ```
    pub fn dead_owner(&self) -> Option<&TransactionOwner> {
        match &self.owner {
            OwnerEvidence::Owned(record, OwnerLiveness::Dead) => return Some(record),
            _ => return None,
        }
    }

    /// What:
    ///  The process number to recover under:
    ///  a dead owner's, or `NO_OWNER_PID` for an abandoned directory.
    /// Why:
    ///   Both are recovered as a dead owner's directory is;
    ///  only the PID file match needs the number.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// dead ? owner.ownerPid : abandoned ? NO_OWNER_PID : undefined
    /// ```
    pub fn recoverable_owner_pid(&self) -> Option<i64> {
        match &self.owner {
            OwnerEvidence::Owned(record, OwnerLiveness::Dead) => return Some(record.owner_pid),
            OwnerEvidence::Abandoned => return Some(NO_OWNER_PID),
            _ => return None,
        }
    }
}

/// What:
///  The liveness of an owner whose existence probe was refused.
///  A refusal for lack of permission names a process that exists but is not this account's to signal,
///  so the owner is alive; any other refusal is a recovery failure.
/// Why:
///   Recovery must never touch a transaction that may be live, and it must not fail an unrelated read either:
///  the PID may belong to another account, or to a process of another PID namespace (issue 1784).
///  Treating the owner as alive leaves its transaction alone and lets the command that scanned the registry go on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (error.code === 'EPERM') return 'alive'; throw error;
/// ```
fn liveness_after_refused_probe(owner_pid: i64, error: &std::io::Error) -> Result<OwnerLiveness, RecoveryError> {
    if error.raw_os_error() == Some(libc::EPERM) {
        debug(
            "inspectRegistryEntry",
            format!("transaction owner {owner_pid} cannot be signaled ({error}); it is treated as alive").as_str(),
        );
        return Ok(OwnerLiveness::Alive);
    }
    return Err(RecoveryError(format!(
        "probing transaction owner {owner_pid} failed: {error}"
    )));
}

/// What:
///  Classify an owner,
///  reporting a failed probe as a recovery failure unless the refusal only means the process is not ours to signal.
/// Why:
///   Recovery cannot decide liveness without the probe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await classifyTransactionOwner({ ownerPid, ownerIdentity })
/// ```
fn liveness(record: &TransactionOwner, owner_path: &Path) -> Result<OwnerLiveness, RecoveryError> {
    match classify_transaction_owner_record(record, owner_path) {
        Ok(found) => return Ok(found),
        Err(OwnerLivenessError::Probe(error)) => {
            return liveness_after_refused_probe(record.owner_pid, &error);
        }
        Err(OwnerLivenessError::Identity(error)) => {
            return Err(RecoveryError(format!(
                "reading the identity of transaction owner {} failed: {error}",
                record.owner_pid
            )));
        }
    }
}

/// What:
///  Read and classify an entry's owner,
///  tolerating a live owner removing its own directory.
/// Why:
///   `inspectRegistryEntry`:
///  a published directory needs a valid matching owner record;
///  a
///       staging candidate may hold a torn one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function inspectRegistryEntry(entry: TransactionRegistryEntry): Promise<InspectedEntry>;
/// ```
pub fn inspect_registry_entry(entry: &RegistryEntry) -> Result<InspectedEntry, RecoveryError> {
    let owner_path: PathBuf = entry.path.join(OWNER_FILENAME);
    let staging: bool = entry.kind == EntryKind::Staging;
    let bytes: Vec<u8> = match read_regular_file(owner_path.as_path(), RECOVERY_FILE_LIMIT) {
        Ok(found) => found,
        Err(ReadRefusal::Missing) => {
            debug(
                "inspectRegistryEntry",
                format!("owner record absent at {}", owner_path.display()).as_str(),
            );
            if staging {
                return Ok(unattributed(entry));
            }
            if !recovery_path_exists(entry.path.as_path())? {
                return Ok(InspectedEntry {
                    entry: entry.clone(),
                    owner: OwnerEvidence::Vanished,
                });
            }
            return match ownerless_standing(entry.path.as_path())? {
                OwnerlessStanding::Abandoned => Ok(InspectedEntry {
                    entry: entry.clone(),
                    owner: OwnerEvidence::Abandoned,
                }),
                OwnerlessStanding::Unproven(reason) => Err(ownerless_refusal(entry.path.as_path(), reason.as_str())),
            };
        }
        Err(ReadRefusal::NotRegular | ReadRefusal::TooLarge) => {
            return Err(RecoveryError(format!(
                "Unsafe transaction recovery file: {}",
                owner_path.display()
            )));
        }
        Err(ReadRefusal::Io(error)) => {
            return Err(io_failure("reading", owner_path.as_path(), &error));
        }
    };
    let Some(record) = parse_transaction_owner(bytes.as_slice()) else {
        if staging {
            debug(
                "inspectRegistryEntry",
                format!("staging owner record incomplete: {}", entry.path.display()).as_str(),
            );
            return Ok(unattributed(entry));
        }
        return Err(RecoveryError(format!(
            "Transaction owner record is malformed: {}",
            entry.path.display()
        )));
    };
    if record.transaction_id != entry.transaction_id {
        return Err(RecoveryError(format!(
            "Transaction owner record names another transaction: {}",
            entry.path.display()
        )));
    }
    let state: OwnerLiveness = liveness(&record, owner_path.as_path())?;
    return Ok(InspectedEntry {
        entry: entry.clone(),
        owner: OwnerEvidence::Owned(record, state),
    });
}

/// What:
///  An entry whose owner cannot be attributed.
/// Why:
///   Shared by the missing and torn staging cases.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ entry, owner: 'unattributed' })
/// ```
fn unattributed(entry: &RegistryEntry) -> InspectedEntry {
    return InspectedEntry {
        entry: entry.clone(),
        owner: OwnerEvidence::Unattributed,
    };
}

/// What:
///  An entry's recovery-order key:
///  `<createdAt> <path>` when attributed,
///  otherwise
///       `~<path>`,
///  after every ISO time.
/// Why:
///   `creationKey`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// typeof owner === 'string' ? `~${entry.path}` : `${owner.record.createdAt} ${entry.path}`
/// ```
pub fn creation_key(inspected: &InspectedEntry) -> String {
    let path: String = inspected.entry.path.to_string_lossy().into_owned();
    match &inspected.owner {
        OwnerEvidence::Owned(record, _) => return format!("{} {path}", record.created_at),
        OwnerEvidence::Vanished | OwnerEvidence::Unattributed | OwnerEvidence::Abandoned => return format!("~{path}"),
    }
}

/// What:
///  Order inspected entries oldest first,
///  stably.
/// Why:
///   `oldestFirst`:
///  JavaScript compares the keys by UTF-16 code units;
///  for the ASCII
///       timestamps and registry paths the wrappers write,
///  byte order is the same order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entries.toSorted((left, right) => compare(creationKey(left), creationKey(right)))
/// ```
pub fn oldest_first(entries: Vec<InspectedEntry>) -> Vec<InspectedEntry> {
    // `mut` allows sorting the keyed entries in place.
    let mut keyed: Vec<(String, InspectedEntry)> = Vec::with_capacity(entries.len());
    for entry in entries {
        keyed.push((creation_key(&entry), entry));
    }
    keyed.sort_by(compare_keys);
    let mut ordered: Vec<InspectedEntry> = Vec::with_capacity(keyed.len());
    for (_, entry) in keyed {
        ordered.push(entry);
    }
    return ordered;
}

/// What:
///  Compare two keyed entries by key.
/// Why:
///   A named comparator keeps the sort free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (left, right) => left.key < right.key ? -1 : left.key > right.key ? 1 : 0
/// ```
fn compare_keys(
    left: &(String, InspectedEntry),
    right: &(String, InspectedEntry),
) -> std::cmp::Ordering {
    return left.0.cmp(&right.0);
}

/// Inspection controls stay out of the release executable.
#[cfg(test)]
#[path = "recovery_inspect_tests.rs"]
pub(crate) mod tests;
