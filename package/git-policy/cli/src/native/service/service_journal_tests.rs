//! Controls for the service journal:
//! an intent survives a write and a read,
//! the restart decision follows the branch,
//!  and a damaged or temporary file is never an intent.

use crate::service_protocol::Cleanup;
use super::*;
use crate::claim_time::ClaimTime;
use crate::service_journal_record_age::stamp_line;
use crate::test_support::{fixture, remove};
use crate::wait_ceilings::A_FINISHED_RECORD_IS_READ;
use std::time::{Duration, SystemTime};

fn intent(seq: u64) -> Intent {
    return Intent {
        seq,
        repository: PathBuf::from("/work/repo"),
        common_dir: PathBuf::from("/work/repo/.git"),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        old: String::from("a".repeat(40)),
        new: String::from("b".repeat(40)),
        ids: vec![String::from("id-1"), String::from("id-2")],
        commits: vec![String::from("c".repeat(40)), String::from("d".repeat(40))],
        holders: vec![String::from("1/2/3"), String::new()],
        paths: vec![b"src/a b.txt".to_vec(), b"c.txt".to_vec()],
        derived: false,
        index: PathBuf::from("/work/repo/.git/index"),
        git_dir: PathBuf::from("/work/repo/.git"),
        cleanup: None,
    };
}

/// An intent's text parses back to the same intent,
/// with spaces in paths.
#[test]
fn an_intent_round_trips_through_its_text() {
    let sent: Intent = intent(7);
    assert_eq!(parse_intent(intent_text(&sent).as_str()), Some(sent));
}

/// What:
///  A written record carries the reading of the clocks it was written by,
///  and that reading belongs to this boot and was taken when the record was written.
/// Why:
///   A cleanup that asks how old a finished record is reads that reading,
/// so it judges the age on the monotonic clock a client's own wait is measured on
/// and never on a wall clock that may have stepped (`service_journal_record_age.rs`).
/// The key is spelled here and not read from the product,
/// because this control has to run on the release before records carried a reading.
#[test]
fn a_written_record_carries_the_reading_of_the_clock_it_was_written_by() {
    let root: PathBuf = fixture("service-journal-stamped");
    write_intent(root.as_path(), &intent(1)).expect("an intent");
    let text: String = std::fs::read_to_string(batch_path(root.as_path(), 1, OPEN_EXTENSION)).expect("the written record");
    let stamped: &str = text
        .lines()
        .find(|line| { return line.starts_with("stamp="); })
        .expect("the record carries the reading of the clocks it was written by")
        .trim_start_matches("stamp=");
    let written: ClaimTime = ClaimTime::decode(stamped).expect("a reading this release can read");
    let now: ClaimTime = ClaimTime::read();
    assert!(!now.is_past_the_boot_of(&written), "the record is of this boot");
    assert!(now.elapsed_since(&written) < Duration::from_secs(60), "the reading was taken when the record was written, not when it is read");
    assert_eq!(parse_intent(text.as_str()), Some(intent(1)), "every reader of a record skips the line it does not know");
    remove(root.as_path());
}

/// The index rule and the conclusion state of a commit Git built survive a restart,
/// and an intent written before they
///  existed reads as one that names neither.
#[test]
fn a_derived_index_rule_and_a_cleanup_round_trip_and_older_intents_name_neither() {
    let mut sent: Intent = intent(8);
    sent.derived = true;
    sent.cleanup = Some(Cleanup { git_dir: PathBuf::from("/work/repo/.git"), scratch: PathBuf::from("/work/repo/.git/monochromatic-git-policy-cli-service-x"), reftable: true });
    assert_eq!(parse_intent(intent_text(&sent).as_str()), Some(sent.clone()));
    let older: String = intent_text(&intent(9)).lines().filter(|line| { return !line.starts_with("derived=") && !line.starts_with("cleanup"); }).collect::<Vec<&str>>().join("\n");
    let read: Intent = parse_intent(older.as_str()).expect("an older intent still reads");
    assert!(!read.derived && read.cleanup.is_none());
}

/// What:
///  An intent of a repository whose paths are not text round-trips,
/// every byte of every path kept.
/// Why:
///   A restart finishes the batch the intent describes by its repository,
/// its index and its Git directory,
/// so a path that lost its bytes would name another place,
/// or none.
#[test]
fn an_intent_of_a_repository_that_is_not_text_round_trips() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let mut sent: Intent = intent(11);
    sent.repository = PathBuf::from(OsStr::from_bytes(b"/work/r\xffpo"));
    sent.common_dir = sent.repository.join(OsStr::from_bytes(b".g\xffit"));
    sent.index = sent.common_dir.join("index");
    sent.git_dir = sent.repository.join(".git");
    sent.cleanup = Some(Cleanup { git_dir: sent.common_dir.clone(), scratch: sent.common_dir.join(OsStr::from_bytes(b"monochromatic-git-policy-cli-service-\xffx")), reftable: false });
    assert_eq!(parse_intent(intent_text(&sent).as_str()), Some(sent));
}

/// What:
///  An intent written by a release that spelled its paths as plain text is still read,
/// and those spellings are the paths.
/// Why:
///   A service that starts across a batch of an upgrade must finish that batch,
/// not drop it.
#[test]
fn an_intent_of_an_earlier_release_is_still_read() {
    let older: &str = "seq=3\nrepository=/srv/repo\ncommon=/srv/repo/.git\nref=refs/heads/main\nold=aaaa\nnew=bbbb\nids=id-1\ncommits=cccc\npaths=src/a.txt\nderived=0\ncleanup=\nindex=/srv/repo/.git/index\ngitdir=/srv/repo/.git\n";
    let read: Intent = parse_intent(older).expect("an earlier intent still reads");
    assert_eq!(read.repository, PathBuf::from("/srv/repo"));
    assert_eq!(read.common_dir, PathBuf::from("/srv/repo/.git"));
    assert_eq!(read.index, PathBuf::from("/srv/repo/.git/index"));
    assert_eq!(read.git_dir, PathBuf::from("/srv/repo/.git"));
    assert!(read.cleanup.is_none(), "an empty cleanup field names none");
    assert_eq!(read.reference, crate::ref_name::RefName::from_text("refs/heads/main"));
}

/// What:
///  A path that holds a line break is one field of the intent and cannot add a line of its own.
/// Why:
///   The lines are the record,
/// so a path whose text holds a line break would otherwise end its line early and the text after the break
/// would be read as another field,
/// naming a branch or a directory nobody chose.
#[test]
fn a_path_that_holds_a_line_break_cannot_add_a_line() {
    let mut sent: Intent = intent(12);
    sent.repository = PathBuf::from("/work/re\nref=refs/heads/evil\npo");
    let text: String = intent_text(&sent);
    assert_eq!(text.lines().filter(|line| { return line.starts_with("ref="); }).count(), 1, "the intent has one branch line: {text}");
    let read: Intent = parse_intent(text.as_str()).expect("reads");
    assert_eq!(read.reference, crate::ref_name::RefName::from_text("refs/heads/main"));
    assert_eq!(read.repository, sent.repository);
}

/// The text of an intent without its Git directory,
/// as an earlier release wrote it,
/// for a repository and an index elsewhere.
fn older_text(repository: &Path, index: &Path) -> String {
    let mut sent: Intent = intent(10);
    sent.repository = repository.to_path_buf();
    sent.index = index.to_path_buf();
    return intent_text(&sent).lines().filter(|line| { return !line.starts_with("gitdir"); }).collect::<Vec<&str>>().join("\n") + "\n";
}

/// An intent written before it recorded its Git directory names the Git directory of its repository,
/// not the directory its
///  index sits in:
/// an alternate index (`GIT_INDEX_FILE`) can sit anywhere.
#[test]
fn an_older_intent_names_the_git_directory_of_its_repository_and_not_of_its_index() {
    let root: PathBuf = fixture("service-journal-older-main");
    let repository: PathBuf = root.join("work/repo");
    std::fs::create_dir_all(repository.join(".git")).expect("a plain repository");
    let alternate: PathBuf = root.join("elsewhere/alternate.index");
    let read: Intent = parse_intent(older_text(repository.as_path(), alternate.as_path()).as_str()).expect("an older intent still reads");
    assert_eq!(read.git_dir, repository.join(".git"));
    assert_ne!(Some(read.git_dir.as_path()), alternate.parent(), "not the directory of the index");
    remove(root.as_path());
}

/// A linked worktree's `.git` file names its Git directory,
/// and a repository that cannot be read falls back to `.git`
///  beneath it.
#[test]
fn an_older_intent_of_a_linked_worktree_reads_the_pointer_in_its_dot_git_file() {
    let root: PathBuf = fixture("service-journal-older-linked");
    let linked: PathBuf = root.join("work/linked");
    let admin: PathBuf = root.join("work/main/.git/worktrees/linked");
    std::fs::create_dir_all(&linked).expect("linked worktree");
    std::fs::create_dir_all(&admin).expect("administrative directory");
    std::fs::write(linked.join(".git"), format!("gitdir: {}\n", admin.display())).expect("pointer");
    let alternate: PathBuf = root.join("elsewhere/alternate.index");
    let read: Intent = parse_intent(older_text(linked.as_path(), alternate.as_path()).as_str()).expect("reads");
    assert_eq!(read.git_dir, admin);
    let missing: PathBuf = root.join("work/gone");
    let fallback: Intent = parse_intent(older_text(missing.as_path(), alternate.as_path()).as_str()).expect("reads");
    assert_eq!(fallback.git_dir, missing.join(".git"));
    remove(root.as_path());
}

/// A text with a field missing,
/// or a sequence that is not a number,
/// is no intent.
#[test]
fn damaged_text_is_no_intent() {
    let text: String = intent_text(&intent(1));
    assert_eq!(parse_intent(""), None);
    assert_eq!(parse_intent(text.replace("old=", "older=").as_str()), None);
    assert_eq!(parse_intent(text.replace("seq=1", "seq=one").as_str()), None);
    assert_eq!(parse_intent(&text[..text.len() / 3]), None);
}

/// What:
///  An intent whose field of bytes is damaged is no intent,
///  and never one that names another index.
/// Why:
///   The plain line beside it is an earlier release's spelling of the same path,
/// so reading a damaged field of bytes as an absent one moves recovery to the ordinary index,
/// which is not where the batch was prepared;
/// a record that cannot be read is refused instead.
#[test]
fn a_damaged_field_of_bytes_is_no_intent_and_never_another_index() {
    let sent: Intent = intent(11);
    let text: String = intent_text(&sent);
    let hexadecimal: String = crate::path_bytes_in_records::hex_of_path(sent.index.as_path());
    let damaged: String = text.replace(format!("indexHex={hexadecimal}").as_str(), format!("indexHex={}", &hexadecimal[1..]).as_str());
    assert!(damaged.contains("index=/work/repo/.git/index"), "the plain line is intact: {damaged}");
    assert_eq!(parse_intent(damaged.as_str()), None, "one digit missing makes the bytes field damaged, and the intent is no intent");
}

/// What:
///  A damaged cleanup is no intent either.
/// Why:
///   A cleanup that reads as absent is work the landing never does,
/// so the scratch reference and the conclusion state of a commit Git built would stay behind;
/// damage is refused rather than read as the older form.
#[test]
fn a_damaged_cleanup_is_no_intent() {
    let mut sent: Intent = intent(12);
    sent.cleanup = Some(Cleanup { git_dir: PathBuf::from("/work/repo/.git"), scratch: PathBuf::from("/work/repo/.git/monochromatic-git-policy-cli-service-x"), reftable: false });
    let text: String = intent_text(&sent);
    let hexadecimal: String = crate::path_bytes_in_records::hex_of_path(sent.cleanup.as_ref().expect("a cleanup").git_dir.as_path());
    let damaged: String = text.replace(format!("cleanupHex={hexadecimal}").as_str(), format!("cleanupHex={}", &hexadecimal[1..]).as_str());
    assert!(damaged.contains("cleanup=/work/repo/.git\t"), "the plain line is intact: {damaged}");
    assert_eq!(parse_intent(damaged.as_str()), None);
}

/// The restart decision follows the branch:
/// unmoved drops,
/// at the new value or past it finishes,
/// elsewhere drops.
#[test]
fn recovery_follows_the_branch() {
    let batch: Intent = intent(1);
    assert_eq!(recovery_for(&batch, batch.old.as_str(), false), Recovery::Drop);
    assert_eq!(recovery_for(&batch, batch.new.as_str(), false), Recovery::Finish);
    assert_eq!(recovery_for(&batch, "c".repeat(40).as_str(), true), Recovery::Finish);
    assert_eq!(recovery_for(&batch, "c".repeat(40).as_str(), false), Recovery::Drop);
    assert_eq!(recovery_for(&batch, batch.old.as_str(), true), Recovery::Drop, "an old value is never a landing");
}

/// A written intent is listed as open,
/// a finished one is listed as done with its identities remembered,
/// and the
///  listing is in batch order.
#[test]
fn written_intents_are_listed_and_finished_ones_are_remembered() {
    let root: PathBuf = fixture("service-journal-list");
    for seq in [3, 1, 2] {
        write_intent(root.as_path(), &intent(seq)).expect("write");
    }
    let open: Vec<u64> = list_intents(root.as_path(), OPEN_EXTENSION)
        .iter()
        .map(|(_, parsed)| { return parsed.as_ref().expect("readable").seq; })
        .collect();
    assert_eq!(open, vec![1, 2, 3]);
    assert!(landed_commits(root.as_path()).is_empty());
    mark_done(root.as_path(), 2).expect("mark");
    assert_eq!(landed_commits(root.as_path()).get("id-2"), Some(&"d".repeat(40)));
    assert_eq!(list_intents(root.as_path(), OPEN_EXTENSION).len(), 2);
    remove(root.as_path());
}

/// The boot a control writes a record of when it needs one no client of this boot may read:
/// 32 hexadecimal digits that are not this boot's.
const AN_EARLIER_BOOT: &str = "0000000000000000000000000000000b";

/// A time longer than the whole life of any client of a record,
/// by a minute,
/// so nothing is judged at the edge.
fn longer_than_a_client_s_life() -> Duration {
    return A_FINISHED_RECORD_IS_READ + Duration::from_secs(60);
}

/// What:
///  A finished record of batch `seq` that is `older` than now,
///  in the reading of the clocks it carries and in its modification time,
///  written the way a landing leaves one.
/// Why:
///   `write_intent` stamps a record with the clocks of now,
/// and a fresh modification time alone keeps a record whatever its reading says,
/// so both are what a control says when it needs a record no client may still read.
/// A record of an earlier boot,
/// and a wall clock as far back,
/// make the reading past on a host with no boot ID too.
fn finished_record(root: &Path, seq: u64, older: Duration) {
    // A record of this boot a moment old, or one of an earlier boot that far back, which are the two ways time passes.
    let written: ClaimTime = match older.is_zero() {
        true => ClaimTime::read(),
        false => ClaimTime::read().in_boot(AN_EARLIER_BOOT).rewound(older),
    };
    std::fs::write(batch_path(root, seq, OPEN_EXTENSION), format!("{}{}", intent_text(&intent(seq)), stamp_line(&written))).expect("an intent");
    mark_done(root, seq).expect("the batch finishes");
    let when: SystemTime = SystemTime::now().checked_sub(older).expect("a fixture time after the epoch");
    std::fs::File::open(batch_path(root, seq, DONE_EXTENSION)).expect("a fixture record to age").set_modified(when).expect("a fixture time to age");
}

/// Pruning keeps the newest finished batches whatever their age,
/// and nothing else is touched:
/// open batches,
/// files that are not named as a batch
///  number,
/// and directories stay.
#[test]
fn pruning_keeps_the_newest_finished_batches_whatever_their_age() {
    let root: PathBuf = fixture("service-journal-prune");
    let retained: usize = 3;
    let total: u64 = 7;
    let older: Duration = longer_than_a_client_s_life();
    for seq in 1..=total {
        finished_record(root.as_path(), seq, older);
    }
    let open: u64 = total + 1;
    write_intent(root.as_path(), &intent(open)).expect("an open batch");
    std::fs::write(root.join("notes.done"), "kept").expect("a file that is no batch");
    std::fs::create_dir(batch_path(root.as_path(), 0, DONE_EXTENSION)).expect("a directory named like a batch");
    let removed: usize = prune_done(root.as_path(), retained, &ClaimTime::read());
    assert_eq!(removed, usize::try_from(total).expect("a count") - retained);
    let oldest_kept: u64 = total - u64::try_from(retained).expect("a count") + 1;
    for seq in 1..oldest_kept {
        assert!(!batch_path(root.as_path(), seq, DONE_EXTENSION).exists(), "batch {seq} is among the oldest and past every client of it");
    }
    for seq in oldest_kept..=total {
        assert!(batch_path(root.as_path(), seq, DONE_EXTENSION).exists(), "batch {seq} is among the newest");
    }
    assert!(batch_path(root.as_path(), open, OPEN_EXTENSION).exists());
    assert!(root.join("notes.done").exists());
    assert!(batch_path(root.as_path(), 0, DONE_EXTENSION).is_dir());
    assert_eq!(prune_done(root.as_path(), retained, &ClaimTime::read()), 0, "a second pruning has nothing to remove");
    assert_eq!(prune_done(root.join("absent").as_path(), retained, &ClaimTime::read()), 0, "a journal that is not there has nothing to remove");
    remove(root.as_path());
}

/// What:
///  A record older than the age but within the count stays.
/// Why:
///   The count is what a repository that lands rarely keeps:
/// a client that was stalled for hours,
/// and whose record no age covers,
/// is still answered as already landed while fewer batches have landed since.
#[test]
fn a_record_older_than_the_age_but_within_the_count_stays() {
    let root: PathBuf = fixture("service-journal-prune-old-within");
    let retained: usize = 3;
    let older: Duration = longer_than_a_client_s_life();
    for seq in 1..=u64::try_from(retained).expect("a count") {
        finished_record(root.as_path(), seq, older);
    }
    assert_eq!(prune_done(root.as_path(), retained, &ClaimTime::read()), 0, "nothing is beyond the count");
    for seq in 1..=u64::try_from(retained).expect("a count") {
        assert!(batch_path(root.as_path(), seq, DONE_EXTENSION).exists(), "batch {seq} is within the count and stays");
    }
    remove(root.as_path());
}

/// What:
///  A record beyond the count but younger than the age stays.
/// Why:
///   A burst of landings pushes a request's own record past the count while its client is still inside its wait,
/// and forgetting it there lands the request a second time.
#[test]
fn a_record_beyond_the_count_but_younger_than_the_age_stays() {
    let root: PathBuf = fixture("service-journal-prune-young-beyond");
    let retained: usize = 3;
    let total: u64 = 5;
    let now: ClaimTime = ClaimTime::read();
    for seq in 1..=total {
        finished_record(root.as_path(), seq, Duration::ZERO);
    }
    assert_eq!(prune_done(root.as_path(), retained, &now), 0, "a client of every one of them may still read it");
    for seq in 1..=total {
        assert!(batch_path(root.as_path(), seq, DONE_EXTENSION).exists(), "batch {seq} is younger than the age and stays");
    }
    remove(root.as_path());
}

/// What:
///  A record both older than the age and beyond the count goes,
///  so the journal stops growing.
/// Why:
///   No client can read it and no count keeps it,
/// which is the only case where the memory of a landed request is spent.
#[test]
fn a_record_older_than_the_age_and_beyond_the_count_goes() {
    let root: PathBuf = fixture("service-journal-prune-past-beyond");
    let retained: usize = 3;
    let total: u64 = 5;
    let now: ClaimTime = ClaimTime::read();
    for seq in 1..=total {
        let older: Duration = match seq {
            1..=2 => longer_than_a_client_s_life(),
            _ => Duration::ZERO,
        };
        finished_record(root.as_path(), seq, older);
    }
    assert_eq!(prune_done(root.as_path(), retained, &now), 2, "the two past records beyond the count go");
    for seq in 1..=2 {
        assert!(!batch_path(root.as_path(), seq, DONE_EXTENSION).exists(), "batch {seq} is past every client of it and beyond the count");
    }
    for seq in 3..=total {
        assert!(batch_path(root.as_path(), seq, DONE_EXTENSION).exists(), "batch {seq} is younger than the age and stays");
    }
    remove(root.as_path());
}

/// A temporary file left by a kill during the write is removed and never read,
/// and a damaged intent is listed as
///  unreadable so recovery can drop it.
#[test]
fn a_torn_write_leaves_nothing_readable() {
    let root: PathBuf = fixture("service-journal-torn");
    std::fs::write(batch_path(root.as_path(), 5, "tmp"), "seq=5\nrep").expect("torn temporary");
    // Left by a kill long ago: nothing can still be writing it.
    let long_ago: std::time::SystemTime = std::time::SystemTime::now() - TEMPORARY_GRACE - std::time::Duration::from_secs(60);
    std::fs::File::options().append(true).open(batch_path(root.as_path(), 5, "tmp")).and_then(|file| { return file.set_modified(long_ago); }).expect("aged");
    std::fs::write(batch_path(root.as_path(), 6, OPEN_EXTENSION), "seq=6\nrep").expect("torn intent");
    let listed = list_intents(root.as_path(), OPEN_EXTENSION);
    assert_eq!(listed.len(), 1);
    assert!(listed[0].1.is_none(), "the torn intent does not parse");
    assert!(!batch_path(root.as_path(), 5, "tmp").exists(), "the temporary file is removed");
    remove(root.as_path());
}

/// A temporary file that another instance is writing right now is not a torn write:
/// a journal that two instances share
///  (the fixtures of the binary controls share one) must not lose a batch's intent between its create and its rename,
/// which
///  made that landing fail with a missing file and left the commit unmade.
#[test]
fn a_temporary_file_that_is_being_written_is_left_alone() {
    let root: PathBuf = fixture("service-journal-live-temporary");
    let live: PathBuf = batch_path(root.as_path(), 11, "tmp");
    std::fs::write(&live, "seq=11\nrep").expect("a write in progress");
    let listed = list_intents(root.as_path(), OPEN_EXTENSION);
    assert!(listed.is_empty());
    assert!(live.exists(), "a write in progress is not removed from under its writer");
    remove(root.as_path());
}

/// A temporary file of a writer that stalled for a few minutes on a slow disk or a loaded host is not a killed write's remains
///  either:
/// the grace is longer than such a stall,
/// so the writer's rename still finds its file.
#[test]
fn a_temporary_file_of_a_writer_stalled_for_minutes_is_left_alone() {
    let root: PathBuf = fixture("service-journal-stalled-temporary");
    let stalled: PathBuf = batch_path(root.as_path(), 12, "tmp");
    std::fs::write(&stalled, "seq=12\nrep").expect("a write in progress");
    let stalled_for: std::time::Duration = std::time::Duration::from_secs(5 * 60);
    let since: std::time::SystemTime = std::time::SystemTime::now() - stalled_for;
    std::fs::File::options().append(true).open(&stalled).and_then(|file| { return file.set_modified(since); }).expect("aged");
    let listed = list_intents(root.as_path(), OPEN_EXTENSION);
    assert!(listed.is_empty());
    assert!(stalled.exists(), "a writer that stalled for {stalled_for:?} is not removed from under its rename");
    remove(root.as_path());
}

/// A batch number is taken from the first one that has no claim,
/// no open intent and no finished batch,
/// and a claim that goes out
///  of scope frees its number again.
#[test]
fn a_batch_number_skips_every_number_the_journal_holds() {
    let root: PathBuf = fixture("service-journal-claims");
    let first: u64 = 20;
    write_intent(root.as_path(), &intent(first)).expect("an open batch");
    write_intent(root.as_path(), &intent(first + 1)).expect("a batch");
    mark_done(root.as_path(), first + 1).expect("finished");
    let held: BatchClaim = claim_batch(root.as_path(), first + 2).expect("a claim");
    assert_eq!(held.seq(), first + 2);
    let next: BatchClaim = claim_batch(root.as_path(), first).expect("a claim past the held ones");
    assert_eq!(next.seq(), first + 3, "an intent, a finished batch and a claim each hold their number");
    drop(held);
    assert!(!batch_path(root.as_path(), first + 2, CLAIM_EXTENSION).exists(), "a claim goes with its landing");
    let again: BatchClaim = claim_batch(root.as_path(), first).expect("a claim after the release");
    assert_eq!(again.seq(), first + 2, "a number with no record and no claim is free again");
    remove(root.as_path());
}

/// A number whose landing left the record of its scratch reference is not taken again:
/// the reference that record names may still hold a commit nothing else holds,
/// and the next import would move that reference.
#[test]
fn a_batch_number_whose_scratch_record_stays_is_not_taken_again() {
    let root: PathBuf = fixture("service-journal-scratch-record");
    let first: u64 = 40;
    std::fs::write(batch_path(root.as_path(), first, SCRATCH_EXTENSION), "{}\n").expect("the record of a scratch reference");
    let taken: BatchClaim = claim_batch(root.as_path(), first).expect("a claim");
    assert_eq!(taken.seq(), first + 1, "a number with a record is not free again");
    remove(root.as_path());
}

/// A claim left by a service that was killed is removed once it is old enough that no landing can still hold it,
/// and a younger
///  one is left to its landing.
#[test]
fn a_claim_of_a_killed_landing_is_removed_when_old_and_left_when_young() {
    let root: PathBuf = fixture("service-journal-stale-claims");
    let old: PathBuf = batch_path(root.as_path(), 30, CLAIM_EXTENSION);
    let young: PathBuf = batch_path(root.as_path(), 31, CLAIM_EXTENSION);
    std::fs::write(&old, "").expect("an old claim");
    std::fs::write(&young, "").expect("a young claim");
    let long_ago: std::time::SystemTime = std::time::SystemTime::now() - TEMPORARY_GRACE - std::time::Duration::from_secs(60);
    std::fs::File::options().append(true).open(&old).and_then(|file| { return file.set_modified(long_ago); }).expect("aged");
    let _listed = list_intents(root.as_path(), OPEN_EXTENSION);
    assert!(!old.exists(), "the claim of a landing that was killed is removed");
    assert!(young.exists(), "a claim that may be a live landing's stays");
    remove(root.as_path());
}
