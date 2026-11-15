//! What:
//!  Controls of the record a command writes of itself and of the processes it starts:
//!  the one spelling the writer and the reader share,
//!  the two-space form older records carry,
//!  who may remove a record and on what proof,
//!  and the folder that must neither grow with every command nor outlive the last record it held.
//! Why:
//!  The writer wrote `self  <pid>` with two spaces and the reader parsed only one,
//!  so a forwarded command's own record named nobody and its lock could never be proved abandoned,
//!  and a record with no parseable holder was never swept:
//!  every forwarded command left a file (a review of `675771e759`,
//! finding 1).
//!  A removal decided on a reading of the records, with another writer between, lost the line of a live Git
//!  (a review of `2af2ecb879`, finding 3),
//!  so a record is removed only by its writer or after its writer is proved gone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(holdersOf(recordText([['self', 42, id]]))).toEqual([[42, id]]);
//! ```

use crate::lock_holder_record_reads::{
    Record, every_record, identities_recorded, proved_gone_writer, remove_records_no_one_needs, remove_this_process_record_if_no_lock_needs_it,
};
use crate::lock_holder_record_reads::MOST_RECORDS;
use crate::lock_holder_records::{
    CHILD_KIND, RECORD_HEADER, SELF_KIND, folder_of, identity_of_birth, identity_of_process, record_a_child_started, record_line, record_text,
    this_process_record_path, write_a_record_line_once,
};
use crate::git_directory_roots::CommandRepository;
use crate::record_proof_test_support::{a_dead_process, make_pid_file_after_the_lock};
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// What:
///  A record's own line is written in one spelling,
///  and that line reads back as the holder it names.
/// Why:
///  This is the whole defect:
///  the writer's `record_text` and the reader's parse must agree,
///  and the same line must be the one the append path writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(recordText([['self', 42, id]])).toContain(`self 42 ${id}`);
/// ```
#[test]
fn the_self_line_reads_back_as_the_process_it_names() {
    let identity: String = identity_of_birth("linux:4242");
    let text: String = record_text(&[(SELF_KIND, 42, identity.clone())]);
    assert!(text.contains(format!("{SELF_KIND} 42 {identity}").as_str()), "one space after the kind: {text:?}");
    let written: String = record_line(CHILD_KIND, 7, identity.as_str());
    assert_eq!(written, format!("{CHILD_KIND} 7 {identity}\n"), "the append path writes the same form");
    let root: PathBuf = fixture("record-self-line");
    let folder: PathBuf = folder_of(root.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the folder");
    std::fs::write(folder.join("record"), format!("{text}{written}").as_bytes()).expect("a record");
    let named: Vec<String> = identities_recorded(root.as_path(), 42);
    assert_eq!(named, vec![identity.clone()], "the self line names the process it was written for: {text:?}");
    let child: Vec<String> = identities_recorded(root.as_path(), 7);
    assert_eq!(child, vec![identity], "and a child line names its own");
    remove(root.as_path());
}

/// What:
///  A record of the two-space form,
/// which this branch wrote before the fix,
/// still names its holder.
/// Why:
///  Records of that form are on disk wherever the branch has run,
///  and a reader that dropped them would keep a killed command's lock for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(holdersOf(`self  ${pid} ${id}\n`)).toEqual([[pid, id]]);
/// ```
#[test]
fn the_two_space_form_of_a_record_still_names_its_holder() {
    let identity: String = identity_of_birth("linux:4343");
    let root: PathBuf = fixture("record-two-spaces");
    let folder: PathBuf = folder_of(root.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the folder");
    std::fs::write(folder.join("record"), format!("{RECORD_HEADER}\nself  99 {identity}\nchild  98 {identity}\n").as_bytes()).expect("a record of the older form");
    assert_eq!(identities_recorded(root.as_path(), 99), vec![identity.clone()], "the two-space self line reads back");
    assert_eq!(identities_recorded(root.as_path(), 98), vec![identity], "and so does the two-space child line");
    remove(root.as_path());
}

/// What:
///  A line that is not one of the two kinds,
///  and one whose number is not a number,
///  name nobody.
/// Why:
///  A record a person edited decides nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(holdersOf('other 1 x\nself nope x\n')).toEqual([]);
/// ```
#[test]
fn a_record_line_that_is_not_one_names_nobody() {
    let root: PathBuf = fixture("record-not-a-line");
    let folder: PathBuf = folder_of(root.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the folder");
    std::fs::write(folder.join("record"), format!("{RECORD_HEADER}\nother 1 {}\nself nope {}\nbogus\n", identity_of_birth("linux:1"), identity_of_birth("linux:2")).as_bytes()).expect("a record");
    assert!(identities_recorded(root.as_path(), 1).is_empty(), "a kind the reader does not know names nobody");
    assert!(identities_recorded(root.as_path(), 0).is_empty(), "a number that is not one names nobody");
    remove(root.as_path());
}

/// What:
///  A folder of records whose writers and holders are all gone and which no standing lock names is emptied,
///  the folder goes with its last record,
///  and a number a standing lock names keeps every record that names it.
/// Why:
///  The removal is what keeps the folder small enough that every lookup reads it whole;
///  records of the self form were never removed before,
/// so every command left a file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(selfRecordOfADeadProcess); removeRecordsNoOneNeeds(folder, [], []) ; expect(readdir(folder)).toEqual([]);
/// ```
#[test]
fn a_record_of_a_finished_command_is_removed_and_one_a_lock_needs_is_kept() {
    let root: PathBuf = fixture("record-sweep");
    let folder: PathBuf = folder_of(root.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the folder");
    let (gone, identity): (i64, String) = a_dead_process();
    // One record of a command that ended, and one whose number a standing lock still names.
    std::fs::write(folder.join("finished"), record_text(&[(SELF_KIND, gone, identity.clone()), (CHILD_KIND, gone, identity.clone())]).as_bytes())
        .expect("a finished command's record");
    std::fs::write(folder.join("needed"), record_text(&[(SELF_KIND, gone, identity.clone()), (CHILD_KIND, gone, identity)]).as_bytes()).expect("a record a lock needs");
    remove_records_no_one_needs(root.as_path(), every_record(root.as_path()).as_slice(), &[gone]);
    assert!(folder.join("finished").exists() && folder.join("needed").exists(), "a number a standing lock names keeps every record that names it");
    assert!(folder.exists(), "a folder that still holds a record stays, so a lock's proof is never removed with it");
    remove_records_no_one_needs(root.as_path(), every_record(root.as_path()).as_slice(), &[]);
    assert!(!folder.join("finished").exists(), "a record of a command that ended and that no lock names goes");
    assert!(!folder.join("needed").exists(), "and so does the other, once no lock names the number");
    assert!(!folder.exists(), "the folder goes with the last record it held, so a Git directory reads as the command found it");
    remove(root.as_path());
}

/// What:
///  The line a live writer appends after another command read its record is not lost:
///  a record whose reading names every process dead, and one that names no writer,
///  is left where it is.
/// Why:
///  This is the shape that loses a live Git's line (a review of `2af2ecb879`, finding 3):
///  a record an earlier release wrote names only its children,
///  a command reads it,
///  the writer appends its next child's line,
///  and a removal that acts on the reading unlinks the file with the new line in it.
///  The record is this test process's own, so the appended line is written by the shipped writer,
///  and the second actor is placed by the same seam the pass uses: the reading and the removal are two calls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(header + childLineOfADeadProcess); const records = everyRecord(folder); recordChild(liveChild);
/// removeRecordsNoOneNeeds(folder, records, []); expect(record).toContain(liveChild);
/// ```
#[test]
fn a_line_a_writer_appends_after_another_command_read_its_record_is_not_lost() {
    let root: PathBuf = fixture("record-writer-alive");
    let folder: PathBuf = folder_of(root.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the folder");
    let (gone, identity): (i64, String) = a_dead_process();
    let record: PathBuf = this_process_record_path(root.as_path()).expect("this process's record path");
    // The spelling a release before this one wrote: a header and a child's line, and no line naming the writer.
    std::fs::write(record.as_path(), record_text(&[(CHILD_KIND, gone, identity)]).as_bytes()).expect("a record whose only child is gone");
    // The reading another command makes of this repository's records.
    let read: Vec<Record> = every_record(root.as_path());
    // The writer's next child, still running, appended after that reading.
    let mut live: std::process::Child = std::process::Command::new("sleep")
        .arg("60")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("a live child starts");
    record_a_child_started(root.as_path(), live.id()).expect("the child's line is written");
    remove_records_no_one_needs(root.as_path(), read.as_slice(), &[]);
    let kept: String = std::fs::read_to_string(record.as_path()).expect("a record that names no writer is left where it is");
    assert!(kept.contains(format!("child {} ", live.id()).as_str()), "the line appended after the reading is still there: {kept:?}");
    let _killed = live.kill();
    let _waited = live.wait();
    remove(root.as_path());
}

/// What:
///  A line for a child creates the record with the writer's own line,
///  so every record a command writes names the one process that may append to it.
/// Why:
///  A record that names no writer may not be removed by any other command,
///  because nothing says whether more lines will follow;
///  naming the writer when the file is made is what lets every record be judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordChild(liveChild); expect(readFileSync(thisProcessRecordPath())).toContain(`self ${process.pid} `);
/// ```
#[test]
fn a_record_a_child_note_makes_names_its_writer() {
    let root: PathBuf = fixture("record-names-its-writer");
    let folder: PathBuf = folder_of(root.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the folder");
    let mut live: std::process::Child = std::process::Command::new("sleep")
        .arg("60")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("a live child starts");
    record_a_child_started(root.as_path(), live.id()).expect("the child's line is written");
    let record: PathBuf = this_process_record_path(root.as_path()).expect("this process's record path");
    let text: String = std::fs::read_to_string(record.as_path()).expect("the record a child note made");
    assert!(text.starts_with(RECORD_HEADER), "a record starts with its header: {text:?}");
    assert!(text.contains(format!("{SELF_KIND} {} ", std::process::id()).as_str()), "the record names the process that wrote it: {text:?}");
    assert!(text.contains(format!("{CHILD_KIND} {} ", live.id()).as_str()), "and the child that note was about: {text:?}");
    let _killed = live.kill();
    let _waited = live.wait();
    remove(root.as_path());
}

/// What:
///  A record that names no writer is kept when another command reads it.
/// Why:
///  Such a record was written by a release before the writer named itself,
///  so nothing says whether more lines may follow;
///  leaving it costs a file, and removing it could cost a lock its proof.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(header + childLineOfADeadProcess); removeRecordsNoOneNeeds(folder, [], []); expect(record).toBeThere();
/// ```
#[test]
fn a_record_that_names_no_writer_is_never_removed_by_another_command() {
    let root: PathBuf = fixture("record-no-writer");
    let folder: PathBuf = folder_of(root.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the folder");
    let (gone, identity): (i64, String) = a_dead_process();
    let record: PathBuf = folder.join("record-with-no-writer");
    std::fs::write(record.as_path(), record_text(&[(CHILD_KIND, gone, identity)]).as_bytes()).expect("a record an earlier release wrote");
    remove_records_no_one_needs(root.as_path(), every_record(root.as_path()).as_slice(), &[]);
    assert!(record.exists(), "a record that names no writer may still be appended to, so another command leaves it");
    remove(root.as_path());
}

/// What:
///  The writer removes its own record when no lock names what it recorded,
///  and keeps it while one does.
/// Why:
///  The writer is the only process that appends to its record and it will append nothing more,
///  which is the one thing no other command can know.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeOwnRecord(); removeMyRecordIfNoLockNeedsIt([deadChild]); expect(record).toBeThere(); removeMyRecordIfNoLockNeedsIt([]); expect(record).toBeGone();
/// ```
#[test]
fn the_writer_removes_its_own_record_only_when_no_lock_names_what_it_recorded() {
    let root: PathBuf = fixture("record-writer-removes");
    let folder: PathBuf = folder_of(root.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the folder");
    let (gone, identity): (i64, String) = a_dead_process();
    let own: i64 = i64::from(std::process::id());
    let own_identity: String = identity_of_process(own).expect("this process's identity");
    let record: PathBuf = this_process_record_path(root.as_path()).expect("this process's record path");
    std::fs::write(record.as_path(), record_text(&[(SELF_KIND, own, own_identity), (CHILD_KIND, gone, identity)]).as_bytes()).expect("this process's record");
    remove_this_process_record_if_no_lock_needs_it(root.as_path(), &[gone]);
    assert!(record.exists(), "a lock that names a process the record names keeps it");
    remove_this_process_record_if_no_lock_needs_it(root.as_path(), &[]);
    assert!(!record.exists(), "with every Git of this command ended and no lock naming one, the record goes");
    assert!(!folder.exists(), "and the folder goes with it");
    remove(root.as_path());
}

/// What:
///  The one attempt at writing a record reports a file it cannot make,
///  which is what `note` answers with a second attempt.
/// Why:
///  The second attempt exists because a sweep can remove the folder between the look at it and the file made in it,
///  and a retry that did not end would hold the command;
///  a first attempt that fails for any other reason reaches the caller at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(() => writeOnce(folder, join(folder, 'missing', 'file'), ...)).toThrow(/NotFound/);
/// ```
#[test]
fn an_attempt_whose_folder_is_not_under_the_file_reports_it_and_makes_nothing() {
    let root: PathBuf = fixture("record-attempt-fails");
    let folder: PathBuf = folder_of(root.as_path());
    let unreachable: PathBuf = folder.join("a-directory-nobody-made").join("record");
    let failed: std::io::Error = write_a_record_line_once(
        folder.as_path(),
        unreachable.as_path(),
        SELF_KIND,
        std::process::id(),
        identity_of_birth("linux:4242").as_str(),
        identity_of_birth("linux:4243").as_str(),
    )
    .expect_err("a file whose directory is not there cannot be made");
    assert_eq!(failed.kind(), std::io::ErrorKind::NotFound, "the failure says the folder or the file is not there: {failed}");
    assert!(!unreachable.exists(), "nothing was written");
    remove(root.as_path());
}

/// How many unrelated records the bound control plants before the record a lock needs.
/// It is derived from the bound, so a change to the bound never makes the control stale:
/// every one of them sorts before the holder's record, and the folder still fits in the bound whose records are all read.
const UNRELATED_RECORDS_A_PROOF_STILL_READS: usize = MOST_RECORDS - 2;

/// What:
///  A record a lock's PID file names is used even when every other record in the folder sorts before it and there are
///  thousands of them:
///  the proof reads every record the bound allows, whatever the order.
/// Why:
///  A bounded listing read in name order once answered "no record" for a lock whose record was there,
///  so whether a commit failed depended on lexical order
///  (a review of `4e9b9fff1d`,
/// finding 3).
///  The bound now answers "unknown" rather than "none" when it cuts the folder,
///  and this control holds the other half: under the bound, order decides nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let n = 0; n < MOST_RECORDS - 2; n++) writeRecord(folder, `${String(n).padStart(4, '0')}.unrelated`);
/// writeRecord(folder, `${deadPid}.record`); plantLock(indexLock, deadPid);
/// meetTheLocks(repository); expect(existsSync(indexLock)).toBe(false);
/// ```
#[test]
fn a_record_a_locks_number_names_is_found_past_the_bound_a_sweep_reads() {
    let root: PathBuf = fixture("record-past-the-bound");
    let folder: PathBuf = folder_of(root.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the folder");
    let (gone, identity): (i64, String) = a_dead_process();
    let unrelated_identity: String = identity_of_birth("linux:4242");
    for index in 0..UNRELATED_RECORDS_A_PROOF_STILL_READS {
        let name: String = format!("{index:04}.unrelated");
        std::fs::write(folder.join(name.as_str()), record_text(&[(SELF_KIND, 4242, unrelated_identity.clone())]).as_bytes()).expect("an unrelated record");
    }
    let record: PathBuf = folder.join(format!("{gone}.record"));
    std::fs::write(record.as_path(), record_text(&[(SELF_KIND, gone, identity.clone()), (CHILD_KIND, gone, identity)]).as_bytes()).expect("the record the lock needs");
    assert!(
        proved_gone_writer(root.as_path(), gone).is_some(),
        "the proof reads the record of the number the lock names, whatever else the folder holds and in whatever order"
    );
    let lock: PathBuf = root.join("index.lock");
    std::fs::write(lock.as_path(), b"a half-written index\n").expect("the lock a killed Git left");
    make_pid_file_after_the_lock(lock.as_path(), gone);
    // The pass over one plain repository: its Git directory is its common directory, and the index lies in it.
    crate::test_support::make_it_a_git_directory(root.as_path());
    let repository: CommandRepository = CommandRepository {
        start: root.clone(),
        git_dir: Some(root.clone()),
        common_dir: Some(root.clone()),
        index_file: Some(root.join("index")),
        shallow_file: Some(root.join("shallow")),
        object_dir: Some(root.join("objects")),
    };
    crate::abandoned_locks::meet_the_locks_of_the_repository_the_command_works_on(&repository);
    assert!(!lock.exists(), "the lock is cleared on the record the number names, past the bound a sweep reads");
    remove(root.as_path());
}

/// What:
///  A record a lock's number names at the usual place does not hide another record of the same number that contradicts
///  it:
///  every record of the folder takes part in the verdict,
///  and a lock whose holder may be alive in another PID namespace is kept.
/// Why:
///  The proof once read only the names that begin with the number when one of them existed,
///  so a renamed record naming another namespace was never read,
///  and the lock was unlinked on an incomplete proof
///  (a review of `429503238b`,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeRecord(folder, `${pid}.record`, deadLocalIdentity); writeRecord(folder, 'renamed', identityOfAnotherNamespace(pid));
/// plantLock(indexLock, pid); expect(provedGoneWriter(common, pid)).toBeUndefined(); expect(clearAbandonedLocks(...).cleared).toBe(0);
/// ```
#[test]
fn a_renamed_record_of_the_same_number_keeps_the_lock_its_usual_record_would_clear() {
    let root: PathBuf = fixture("record-renamed-contradiction");
    let folder: PathBuf = folder_of(root.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the folder");
    let (gone, identity): (i64, String) = a_dead_process();
    std::fs::write(folder.join(format!("{gone}.record")), record_text(&[(SELF_KIND, gone, identity.clone()), (CHILD_KIND, gone, identity)]).as_bytes())
        .expect("the usual record of a dead local process");
    // The same number, named by a record whose identity is a PID namespace this reader does not know: it may be a live
    //  Git of that namespace, so the proof must answer unknown.
    let unknown: String = another_namespaces_identity(gone);
    std::fs::write(folder.join("renamed-record"), record_text(&[(SELF_KIND, gone, unknown.clone())]).as_bytes()).expect("a renamed record of the same number");
    let lock: PathBuf = root.join("index.lock");
    std::fs::write(lock.as_path(), b"a half-written index\n").expect("the lock");
    make_pid_file_after_the_lock(lock.as_path(), gone);
    assert!(proved_gone_writer(root.as_path(), gone).is_none(), "the proof answers unknown while a record of another namespace names the number");
    crate::test_support::make_it_a_git_directory(root.as_path());
    let repository: CommandRepository = CommandRepository {
        start: root.clone(),
        git_dir: Some(root.clone()),
        common_dir: Some(root.clone()),
        index_file: Some(root.join("index")),
        shallow_file: Some(root.join("shallow")),
        object_dir: Some(root.join("objects")),
    };
    crate::abandoned_locks::meet_the_locks_of_the_repository_the_command_works_on(&repository);
    assert!(lock.exists(), "the lock is kept, because its holder may live in the namespace that record names");
    remove(root.as_path());
}

/// What:
///  An identity naming `pid` in a PID namespace this reader is not in,
///  built from the parts a real identity has.
/// Why:
///   The judgement answers unknown for another namespace,
///  which is what keeps a lock a Git of that namespace may hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${birth}#${boot}#12345#${machine}` // a namespace number this reader never has
/// ```
fn another_namespaces_identity(pid: i64) -> String {
    let real: String = identity_of_process(pid).unwrap_or_else(|| { return identity_of_birth("linux:1"); });
    let mut parts = real.split('#');
    let birth: &str = parts.next().unwrap_or("linux:1");
    let boot: &str = parts.next().unwrap_or("boot");
    let _scope: Option<&str> = parts.next();
    let machine: &str = parts.next().unwrap_or("machine");
    return format!("{birth}#{boot}#4026531999#{machine}");
}

/// What:
///  A named pipe at this process's record path fails the recording at once instead of holding the command,
///  and leaves the pipe where it was.
/// Why:
///  Opening a pipe for writing waits for a reader that never comes,
///  and the open happened before the ceiling of the command the child serves had begun,
///  so a Git start could wait for good
///  (a review of `429503238b`,
/// finding 4).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(thisProcessRecordPath(common)); expect(() => recordAChildStarted(common, pid)).toThrow(); expect(isFifo(record)).toBe(true);
/// ```
#[test]
fn a_named_pipe_at_the_records_path_fails_the_recording_and_is_left_alone() {
    let root: PathBuf = fixture("record-named-pipe");
    std::fs::create_dir_all(folder_of(root.as_path()).as_path()).expect("the folder");
    let record: PathBuf = this_process_record_path(root.as_path()).expect("this process's record path");
    let made = std::process::Command::new("mkfifo").arg(record.as_path()).status().expect("mkfifo runs");
    assert!(made.success(), "the control made a named pipe at the record's path");
    let mut live: std::process::Child = std::process::Command::new("sleep")
        .arg("60")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("a live child starts");
    let began: std::time::Instant = std::time::Instant::now();
    let failed: std::io::Error = record_a_child_started(root.as_path(), live.id()).expect_err("a pipe at the record's path cannot be written");
    let _killed = live.kill();
    let _waited = live.wait();
    assert!(began.elapsed() < std::time::Duration::from_secs(5), "the recording answers at once instead of waiting for a reader: {began:?}");
    assert_eq!(failed.kind(), std::io::ErrorKind::InvalidInput, "the failure says the name holds no plain file: {failed}");
    use std::os::unix::fs::FileTypeExt as _;
    assert!(std::fs::symlink_metadata(record.as_path()).is_ok_and(|metadata| { return metadata.file_type().is_fifo(); }), "the pipe is still where the person put it");
    remove(root.as_path());
}

/// What:
///  A folder holding more records than the bound a proof reads answers unknown and keeps the lock,
///  and the records it did not read stay where they are.
/// Why:
///  Every record can name any number in its own lines,
///  so a reader that could not read them all cannot say a lock's holder is gone:
///  the honest answer is unknown,
///  which keeps the lock
///  (a review of `429503238b`,
/// finding 2).
///  Nothing is guessed and nothing is removed on a listing the bound cut.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let n = 0; n <= MOST_RECORDS; n++) writeRecord(folder, `${String(n).padStart(5, '0')}.unrelated`);
/// writeRecord(folder, `${deadPid}.record`); plantLock(indexLock, deadPid);
/// expect(provedGoneWriter(common, deadPid)).toBeUndefined(); meetTheLocks(repository); expect(existsSync(indexLock)).toBe(true);
/// ```
#[test]
fn a_folder_past_the_bound_answers_unknown_and_keeps_the_lock() {
    let root: PathBuf = fixture("record-past-the-bound-unknown");
    let folder: PathBuf = folder_of(root.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the folder");
    let (gone, identity): (i64, String) = a_dead_process();
    let unrelated_identity: String = identity_of_birth("linux:4242");
    for index in 0..=MOST_RECORDS {
        let name: String = format!("{index:05}.unrelated");
        std::fs::write(folder.join(name.as_str()), record_text(&[(SELF_KIND, 4242, unrelated_identity.clone())]).as_bytes()).expect("an unrelated record");
    }
    let record: PathBuf = folder.join(format!("{gone}.record"));
    std::fs::write(record.as_path(), record_text(&[(SELF_KIND, gone, identity.clone()), (CHILD_KIND, gone, identity)]).as_bytes()).expect("the record the lock names");
    assert!(proved_gone_writer(root.as_path(), gone).is_none(), "the proof answers unknown when the folder is past the bound it reads");
    crate::test_support::make_it_a_git_directory(root.as_path());
    let lock: PathBuf = root.join("index.lock");
    std::fs::write(lock.as_path(), b"a half-written index\n").expect("the lock");
    make_pid_file_after_the_lock(lock.as_path(), gone);
    let repository: CommandRepository = CommandRepository {
        start: root.clone(),
        git_dir: Some(root.clone()),
        common_dir: Some(root.clone()),
        index_file: Some(root.join("index")),
        shallow_file: Some(root.join("shallow")),
        object_dir: Some(root.join("objects")),
    };
    crate::abandoned_locks::meet_the_locks_of_the_repository_the_command_works_on(&repository);
    assert!(lock.exists(), "the lock is kept, because the proof could not show it saw every record");
    remove(root.as_path());
}
