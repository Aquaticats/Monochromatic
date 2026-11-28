//! What:
//!  The control of the removal's command path:
//!  the one moment it hands the origin cleanup record,
//!  taken through the very function the command calls.
//! Why:
//!  The record's reader accepts one spelling of a moment and refuses every other,
//!  and the command path is the writer no host control reached before this one:
//!  every other control plants `now` itself, so each of them agreed with the reader
//!  whatever the command's own clock wrote.
//!  A later change of that clock is caught here, because this control reads the moment
//!  through the command's own function and never restates the spelling.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const now = theMomentTheRecordsCarry();
//! folder.publish(OriginCleanupRecord.owed({ ...tuples, startedAt: now }));
//! expect(anExistingCleanupRecord(common, ownName).startedAt).toBe(now);
//! ```

use super::the_moment_the_records_carry;
use crate::worktree_clone_origin_cleanup::origin_cleanup_publication::CleanupFolder;
use crate::worktree_clone_origin_cleanup::origin_cleanup_reading::an_existing_cleanup_record;
use crate::worktree_clone_origin_cleanup::origin_cleanup_record::{
    CleanupBlock,
    OriginCleanupRecord,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_words::BlockReason;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_fixtures::{
    ORIGIN_OWN_NAME,
    a_fixture,
};
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{
    CLONE,
    CLONE_GUID,
    ORIGIN_CREATETXG,
    ORIGIN_DATASET,
    ORIGIN_DATASET_GUID,
    ORIGIN_SNAPSHOT_GUID,
    RemovalFixture,
    guid,
};
use crate::worktree_clone_removal_proof_and_plan::started_moment::is_seconds_with_offset;
use std::path::Path;

/// The slot's path,
/// which every record of this control names.
const SLOT_PATH: &str = "/home/an-account/a-checkout/.claude/worktrees/fix-flaky-test";

/// What:
///  One record of the fixture's own tuple,
///  whose two moments are the command's moment.
/// Why:
///  The removal writes the state lines twice:
///  an `owed` record before its first destroy and a `blocked` one when an attempt
///  could not finish,
///  so both spellings of the moment are what the reader must accept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = OriginCleanupRecord.owed({ origin, clone, slotPath, startedAt: now });
/// ```
fn a_record(now: &str) -> OriginCleanupRecord {
    return OriginCleanupRecord::owed(
        ORIGIN_DATASET,
        guid(ORIGIN_DATASET_GUID),
        ORIGIN_OWN_NAME,
        guid(ORIGIN_SNAPSHOT_GUID),
        ORIGIN_CREATETXG,
        CLONE,
        guid(CLONE_GUID),
        Path::new(SLOT_PATH),
        now,
    );
}

/// What:
///  The moment the removal's command path reads is published by the record's own
///  writer,
///  read back by its own reader,
///  and still the same moment.
/// Why:
///  A removal whose record its own reader refuses destroys nothing and never finishes,
///  which is what two releases shipped,
///  so the clock the command reads is held here against the reader that has to accept it.
#[test]
fn the_moment_the_command_reads_survives_the_records_write_and_read_back() {
    let fixture: RemovalFixture = a_fixture("the-moment-the-command-reads");
    let now: String = the_moment_the_records_carry();
    assert!(
        is_seconds_with_offset(now.as_str()),
        "the command's moment is the spelling this release's readers accept: {now}"
    );
    let folder: CleanupFolder =
        CleanupFolder::open_or_create(fixture.common()).expect("the cleanup folder opens");
    let owed: OriginCleanupRecord = a_record(now.as_str());
    folder
        .publish(&owed)
        .expect("the moment the command reads is one the record's writer publishes");
    let read: OriginCleanupRecord = an_existing_cleanup_record(fixture.common(), ORIGIN_OWN_NAME)
        .expect("the cleanup folder answers")
        .expect("the record it published stands");
    assert_eq!(
        read.started_at, now,
        "the record reads back the very moment the command read"
    );
}

/// What:
///  The same moment is what a blocked attempt writes as `attemptedAt`,
///  and the reader accepts it there too.
/// Why:
///  The removal hands one moment to both fields,
///  so a change of the clock that only one of the two readers tolerates would otherwise
///  stop a removal after its first destroy rather than before it.
#[test]
fn the_same_moment_reads_back_as_the_attempts_own_time() {
    let fixture: RemovalFixture = a_fixture("the-moment-the-attempt-records");
    let now: String = the_moment_the_records_carry();
    let folder: CleanupFolder =
        CleanupFolder::open_or_create(fixture.common()).expect("the cleanup folder opens");
    let blocked: OriginCleanupRecord = a_record(now.as_str()).blocked(
        CleanupBlock {
            reason: BlockReason::Dependency,
            errno: 0,
            clones: 1,
            holds: 1,
        },
        now.as_str(),
    );
    folder
        .publish(&blocked)
        .expect("the attempt's moment is one the record's writer publishes");
    let read: OriginCleanupRecord = an_existing_cleanup_record(fixture.common(), ORIGIN_OWN_NAME)
        .expect("the cleanup folder answers")
        .expect("the record it published stands");
    assert_eq!(
        read.attempted_at.as_deref(),
        Some(now.as_str()),
        "the record reads back the attempt's own moment"
    );
}
