//! What:
//!  The forced arm of a ZFS worktree clone's removal:
//!  who the command's caller and its session are,
//!  which holders a forced removal spares,
//!  which holders it may end and which it refuses,
//!  what it reads before it ends anything,
//!  and every word it answers with.
//! Why:
//!  Plan step 17 builds, with `--force`,
//!  "the ending of the holders other than the caller and its session, once the owed measurement shows it
//!  works; the identification of the caller and its session among the holders"
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  "Step 17").
//!  The design is
//!  `doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  accepted with its V10 answer "Stop that unit" by
//!  `doc/decision/cli-git-zfs-step-17-forced-and-cancelled-removal-designs-accepted.md`.
//!  Nothing in this folder is reached by a person's command:
//!  the removal's command path stays behind the cut
//!  (`src/native/source_guard/removal_command_path_writes_nothing_tests.rs`),
//!  no sentence of a release offers `--force`,
//!  and the only callers are this folder's controls and the guest arms.
//!  The cancelled removal and the `cancel-worktree-removal` command are the sibling design's and not built
//!  here.
//!  Every ending this folder would make is gated twice:
//!  by the evidence the probe rounds recorded for that kind of holder
//!  (`holder_endings.rs`),
//!  and by the constant that sizes its wait,
//!  which ships with no value (`forced_arm_numbers.rs`).
//!  A kind the evidence does not show is identified and refused, never ended.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `forced_removal_words.rs`:
//!   every word this arm writes or answers,
//!   the property names and the reserved file and directory suffixes.
//! - `forced_arm_numbers.rs`:
//!   the waits and the heartbeat this arm would need,
//!   each a constant with no value until the owner decides one or step 22's measurement sizes it.
//! - `process_session_record.rs`:
//!   one process record as the kernel prints it,
//!   the fields the caller's chain and the protected set are read from,
//!   and which read failures are a blind spot and which are a task that has gone.
//! - `caller_property.rs`:
//!   the caller property and the force property,
//!   their values and their reading.
//! - `caller_chain_walk.rs`:
//!   the walk from the command to the first process,
//!   the two checks each step makes,
//!   and the protected set it proves.
//! - `started_by_the_service_manager.rs`:
//!   the refusal of a `--force` command the service manager started,
//!   and the plain words it prints.
//! - `holder_scan_file.rs`:
//!   how root's finding reaches the account's wrapper:
//!   the second root-owned file beside the receipt,
//!   read under the request's own GUID and nonce,
//!   and the refusal a missing file is.
//! - `receiver_closure.rs`:
//!   the mounts that receive an unmount's propagation,
//!   the kept receivers that have a mount at the slot,
//!   and the covered ones.
//! - `holder_endings.rs`:
//!   what this arm may do to each kind of holder,
//!   with the round that showed it or the refusal it answers instead.
//! - `holder_preflight.rs`:
//!   the reads a forced removal makes before it ends anything,
//!   and the refusal each incomplete or unusable answer is.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const forced = planTheForcedRemoval({ scan, callerProperty, forceProperty, tables });
//! forced.refusal; forced.namespaces;
//! ```

/// Every word this arm writes or answers.
pub mod forced_removal_words;

/// The waits and the heartbeat, each a constant with no value.
pub mod forced_arm_numbers;

/// One process record and what a read failure proves.
pub mod process_session_record;

/// The caller property and the force property.
pub mod caller_property;

/// The walk to the first process and the protected set.
pub mod caller_chain_walk;

/// The refusal of a command the service manager started.
pub mod started_by_the_service_manager;

/// How root's finding reaches the wrapper.
pub mod holder_scan_file;

/// The mounts an unmount reaches.
pub mod receiver_closure;

/// What this arm may do to each kind of holder.
pub mod holder_endings;

/// The reads before the first act, and their refusals.
pub mod holder_preflight;

/// Every word this arm writes or answers.
pub use forced_removal_words::{
    CALLER_PROPERTY, DESTINATION_DIRECTORY_PREFIX, FAILED_STEP_MAXIMUM_BYTES, FORCE_PROPERTY, PROGRESS_FILE_SUFFIX,
    RefusalWord, SCAN_FILE_SUFFIX, Stage,
};
/// The waits and the heartbeat.
pub use forced_arm_numbers::{
    GIVE_UP_HEARTBEATS, HEARTBEAT_PERIOD, RefusedClock, STOP_WAIT, TERMINATION_WAIT, TimeoutWindow, the_stop_wait,
};
/// One process record and what a read failure proves.
pub use process_session_record::{
    CommandWord, GROUP_FIELD, PARENT_FIELD, ProcessRecord, ReadOutcome, SESSION_FIELD, STARTED_FIELD, STAT_FIELDS,
    STATE_FIELD, cause_of, parse_process_record, read_process_record,
};
/// The caller property and the force property.
pub use caller_property::{
    BOOT_ID_BYTES, CallerIdentity, CallerPropertyRefusal, ForceWordRefusal, MAXIMUM_CALLER_PROPERTY_BYTES, MAXIMUM_FORCE_PROPERTY_BYTES,
    caller_property_value, force_property_value, is_boot_id, parse_caller_property, parse_force_property,
};
/// The walk to the first process and the protected set.
pub use caller_chain_walk::{CallerChain, ChainRefusal, MOST_ANCESTORS_ASKED_FOR, SparedBy, walk_the_caller_chain};
/// The refusal of a command the service manager started.
pub use started_by_the_service_manager::{
    MANAGER_COMMAND, ManagerKind, ManagerPlace, ManagerReads, ManagerRefusal, THE_FIRST_PROCESS, command_word, manager_of,
    started_by_the_service_manager, the_refusal_text, unified_cgroup_path,
};
/// How root's finding reaches the wrapper.
pub use holder_scan_file::{ScanFile, ScanFileRefusal};
/// The mounts an unmount reaches.
pub use receiver_closure::{
    MountRow, NamespaceTable, Receiver, ReceiverSet, parent_of, plan_the_receivers,
};
/// What this arm may do to each kind of holder.
pub use holder_endings::{
    Ending, HolderKind, IN_PLACE_SHOWN_BY, KindEvidence, LOCKED_NOT_BUILT, MOVE_SHOWN_BY, MissingKind,
    PATH_HOLDER_SHOWN_BY, TASKLESS_NOT_BUILT, THE_KINDS, ending_of, ends_blockers,
};
/// The reads before the first act, and their refusals.
pub use holder_preflight::{ForcedPlan, ForcedRefusal, PreflightReads, Route, plan_the_forced_removal};
