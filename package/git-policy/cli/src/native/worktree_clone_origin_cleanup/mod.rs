//! What:
//!  The record that carries an origin snapshot's cleanup past the clone and the registration:
//!  its folder,
//!  its name grammar,
//!  its lines and states,
//!  its bound,
//!  the identity proof one attempt makes,
//!  the one single-name destroy it sends,
//!  and the look the worktree commands make at it.
//! Why:
//!  The project decided the origin cleanup runs under the account's descendants-only
//!  `mount,destroy` on the checkout,
//!  that the record is separate and durable,
//!  and that the registration is settled after the clone is destroyed even while the origin
//!  cleanup is blocked
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Project decisions",
//!  "Origin cleanup authority" and "Git registration when origin cleanup is blocked";
//!  `doc/planning/cli-git-zfs-origin-cleanup-record-design.md`).
//!  The record's format is the one the project accepted for the three removal-side records:
//!  `key=value` lines
//!  (`doc/decision/cli-git-zfs-removal-side-records-and-the-adds-record-accepted.md`).
//!  Nothing here reads Git,
//!  takes a claim or waits for a person:
//!  an attempt re-reads the world and a completion is an absence proof,
//!  never an intention.
//!  The look destroys a snapshot,
//!  so it is reached from the removal's own step and from the worktree commands the project
//!  named,
//! and from nothing else.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `origin_cleanup_refusal.rs`:
//!   why bytes are not a record,
//!   and the sentence a person reads.
//! - `origin_cleanup_words.rs`:
//!   the header line,
//!   the fifteen keys,
//!   the two states,
//!   the four reasons and the folder's name.
//! - `origin_cleanup_bounds.rs`:
//!   the record's 9,217 bytes and each field's widest value.
//! - `origin_cleanup_record.rs`:
//!   the record as a value,
//!   its two state constructors and its bytes.
//! - `origin_cleanup_attempt_identity.rs`:
//!   the identity proof of the origin dataset and the recorded snapshot.
//! - `origin_cleanup_attempt_counts.rs`:
//!   the clone and hold counts and the mark that block before any request.
//! - `origin_cleanup_attempt.rs`:
//!   the seven steps of one attempt.
//! - `origin_cleanup_lines.rs`:
//!   the lines it prints and when a block speaks.
//! - `origin_cleanup_parse.rs`:
//!   reading those bytes back.
//! - `origin_cleanup_reading.rs`:
//!   the folder,
//!   the name grammar,
//!   the listing and the owner-and-mode read.
//! - `origin_cleanup_publication.rs`:
//!   the folder's creation,
//!   one write and one removal.
//! - `origin_cleanup_look.rs`:
//!   the look the five worktree commands make.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = OriginCleanupRecord.owed({ origin, clone, slotPath, startedAt });
//! folder.publish(record);
//! const outcome = attemptTheCleanup({ record, reads, device, attemptedAt });
//! ```

pub mod origin_cleanup_attempt;
pub mod origin_cleanup_bounds;
pub mod origin_cleanup_lines;
pub mod origin_cleanup_look;
pub mod origin_cleanup_parse;
pub mod origin_cleanup_publication;
pub mod origin_cleanup_reading;
pub mod origin_cleanup_record;
pub mod origin_cleanup_refusal;
pub mod origin_cleanup_words;

/// The healthy record every control of this folder starts from.
/// Never part of the release executable.
#[cfg(test)]
#[path = "origin_cleanup_fixtures_tests.rs"]
pub(crate) mod origin_cleanup_fixtures;

pub use origin_cleanup_attempt::{
    CleanupAttempt,
    CleanupOutcome,
    attempt_the_cleanup,
};
pub use origin_cleanup_lines::{
    line_of_a_block,
    line_of_a_destruction,
    line_of_a_refusal,
    the_reason_changed,
};
pub use origin_cleanup_look::{
    LookAnswer,
    OriginCleanupLook,
    the_origin_cleanup_look,
};
pub use origin_cleanup_publication::CleanupFolder;
pub use origin_cleanup_reading::{
    CleanupListing,
    an_existing_cleanup_record,
    check_the_private_file,
    check_the_private_folder,
    common_directory_owner,
    is_a_record_name,
    is_a_staging_name,
    list_the_origin_cleanups,
    origin_cleanups_folder,
    read_the_cleanup_record,
};
pub use origin_cleanup_record::{
    CleanupBlock,
    OriginCleanupRecord,
};
pub use origin_cleanup_refusal::{
    CleanupRefusal,
    CleanupRefusalCause,
};
pub use origin_cleanup_words::{
    BlockReason,
    CleanupState,
    ORIGIN_CLEANUP_HEADER,
    ORIGIN_CLEANUP_KEYS_IN_ORDER,
    ORIGIN_CLEANUPS_FOLDER,
    ORIGIN_SNAPSHOT_PREFIX,
};
