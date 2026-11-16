//! Controls for the places services take in one journal:
//!  a service that took one is alone,
//!  a second service of the same journal makes them shared,
//!  a place whose process is gone is not a holder and is removed,
//!  and a place nothing can be read from is unknown, which is never "alone".

use super::*;
use std::time::Duration;

/// One empty journal.
fn journal(name: &str) -> PathBuf {
    let journal: PathBuf = crate::test_support::fixture(name).join("journal");
    std::fs::create_dir_all(journal.as_path()).expect("the journal");
    return journal;
}

/// What:
///  The first service of a journal is alone,
///  a second one makes them shared,
///  and the first alone again when the second ends.
/// Why:
///  The promise that a request never lands is one service's to keep only when nothing else can land it,
/// so the count is what a withdrawal answer rests on.
#[test]
fn a_journal_is_alone_until_a_second_service_takes_a_place_in_it() {
    let journal: PathBuf = journal("holders-alone");
    let first: HeldPlace = take_ones_place(journal.as_path()).expect("the first place");
    assert_eq!(holders(journal.as_path(), &first), Holders::Alone, "one service writes this journal");
    let second: HeldPlace = take_ones_place(journal.as_path()).expect("the second place");
    assert_eq!(holders(journal.as_path(), &first), Holders::Shared, "two services write this journal");
    drop(second);
    assert_eq!(holders(journal.as_path(), &first), Holders::Alone, "the second service ended, so the first is alone again");
    // What: A service holding no place of its own is never told it is alone.
    // Why:  `Alone` is the answer a promise rests on, and one live place that belongs to another service is not the
    //       caller's (`service_busy.rs`, `may_promise_a_request_never_lands`).
    let path: PathBuf = first.path().to_path_buf();
    drop(first);
    let none_of_ours: HeldPlace = HeldPlace { path };
    assert_eq!(holders(journal.as_path(), &none_of_ours), Holders::Unknown, "a place that is not the caller's own is not evidence that it is alone");
}

/// What:
///  A place left by a process that is gone is removed and not counted.
/// Why:
///  A killed service must not keep the next one from promising anything,
/// and its record is the only thing it left.
#[test]
fn a_place_whose_process_is_gone_is_taken_away_and_not_counted() {
    let journal: PathBuf = journal("holders-gone");
    let mine: HeldPlace = take_ones_place(journal.as_path()).expect("this service's place");
    // A place of a process that cannot be there: this system's process numbers stop below nine million, and the birth
    // identity is asked for a process that is not there anyway.
    let stale: PathBuf = mine.path().parent().expect("the places directory").join("9000001-stale.service");
    std::fs::write(stale.as_path(), "9000001\nlinux:gB\n").expect("a place of a process that is gone");
    assert_eq!(holders(journal.as_path(), &mine), Holders::Alone, "a place whose process is gone holds nothing");
    assert!(!stale.exists(), "and it is taken away as it is found");
    drop(mine);
}

/// What:
///  A place that cannot be read, and a place whose writer cannot be asked about, make the answer unknown.
/// Why:
///  Nothing that cannot be read is evidence that nobody else is there,
/// and a service that guesses would promise what it cannot keep.
#[test]
fn a_place_that_cannot_be_read_is_unknown() {
    let journal: PathBuf = journal("holders-unknown");
    let mine: HeldPlace = take_ones_place(journal.as_path()).expect("this service's place");
    let unreadable: PathBuf = mine.path().parent().expect("the places directory").join("2-torn.service");
    std::fs::write(unreadable.as_path(), "not a place of any service\n").expect("a place with no process in it");
    assert_eq!(holders(journal.as_path(), &mine), Holders::Unknown, "a place with no process named in it is unknown");
    let _removed = std::fs::remove_file(unreadable.as_path());
    assert_eq!(holders(journal.as_path(), &mine), Holders::Alone);
    drop(mine);
}

/// What:
///  A place that is being made is never a record a reader cannot read: at each instant of making one, another service
///  of the journal reads the places there are and gets an answer about whole places, never `Unknown`.
/// Why:
///  A place written under its final name and filled afterwards is, for a moment, a record nobody can read,
///  and a kill makes that moment last:
///  every later service would then read `Unknown` for good and would make no early withdrawal at all
///  (`doc/planning/cli-git-no-unbounded-waits.md`, and the second review of this branch, finding 2).
///  The control ends the writer at each step instead of a kill, at the package's own seams for a pause at a named point.
#[test]
fn a_place_that_is_being_made_is_never_a_record_a_reader_cannot_read() {
    for point in [IN_THE_PLACE_AFTER_ITS_FILE, IN_THE_PLACE_AFTER_HALF_OF_IT] {
        let journal: PathBuf = journal(format!("holders-under-construction-{point}").as_str());
        let reader: HeldPlace = take_ones_place(journal.as_path()).expect("the reader's own place");
        let (reached, let_go) = crate::service_fault::hooks::hold_the_point(point);
        let writing_journal: PathBuf = journal.clone();
        let writing = std::thread::spawn(move || {
            return take_ones_place(writing_journal.as_path());
        });
        let reached_the_step: Option<bool> = crate::test_support::finished_within_at(Duration::from_secs(30), move || { return reached.blocking_recv().is_ok(); });
        let seen: Holders = holders(journal.as_path(), &reader);
        let _released = let_go.send(());
        let made: Option<HeldPlace> = writing.join().expect("the writing thread ends");
        assert_eq!(reached_the_step, Some(true), "the writer reached {point}");
        // What: A place that is being made is not read as a record nobody can read.
        // Why:  Until the writer gives it its final name the journal holds the reader's one service, so either answer
        //       of a count is true; `Unknown` is the one that would disable the promise for good.
        assert_ne!(seen, Holders::Unknown, "at {point} the journal's places are readable");
        let published: HeldPlace = made.expect("the place was taken once the writer went on");
        assert_eq!(holders(journal.as_path(), &reader), Holders::Shared, "and it is a place once it is whole");
        drop(published);
        drop(reader);
    }
}

/// What:
///  A place written by an earlier build of this branch, whose content names no process at all, stops disabling the
///  promise once it is older than the moment it could be being written, and is taken away then; one that is younger is
///  left where it is and answers `Unknown`.
/// Why:
///  A record that names no process is not evidence of a live service, and it must not keep every later service from
///  answering an early withdrawal for ever;
///  a record that may be being written this instant is left, because removing it would take a live service's place.
#[test]
fn a_torn_place_of_an_earlier_build_is_left_while_it_could_be_being_written_and_taken_away_after() {
    let journal: PathBuf = journal("holders-torn-of-an-earlier-build");
    let mine: HeldPlace = take_ones_place(journal.as_path()).expect("this service's place");
    let torn: PathBuf = mine.path().parent().expect("the places directory").join(format!("{}-torn.service", std::process::id()));
    std::fs::write(torn.as_path(), b"").expect("a place of no bytes, as a kill between its name and its bytes leaves");
    assert_eq!(holders(journal.as_path(), &mine), Holders::Unknown, "a place that could be being written this instant is left");
    assert!(torn.exists(), "and it is not removed");
    // Older than any write of a place takes: the file's own time says when it was last written.
    let long_ago: std::time::SystemTime = std::time::SystemTime::now().checked_sub(Duration::from_secs(2 * 60 * 60)).expect("two hours before now");
    let held: std::fs::File = std::fs::File::options().write(true).open(torn.as_path()).expect("the torn place");
    held.set_times(std::fs::FileTimes::new().set_modified(long_ago)).expect("the record's own time");
    drop(held);
    assert_eq!(holders(journal.as_path(), &mine), Holders::Alone, "a record nothing could have been writing for two hours is not a holder");
    assert!(!torn.exists(), "and it is taken away, so the early withdrawal of this journal comes back by itself");
    drop(mine);
}

/// What:
///  A reader of the places answers within a bound when one of their names is a pipe nobody writes to,
///  and answers that it cannot tell.
/// Why:
///  `may_promise_a_request_never_lands` runs on the landing thread, the one thread that lands and answers,
///  so a read there that storage can hold stops every commit behind it
///  (the fourth review of this branch, finding 2):
///  a name that is a pipe is the deterministic way to reach such a read, and a journal on a stalled device does the same.
#[cfg(unix)]
#[test]
fn a_place_that_is_a_pipe_is_answered_within_a_bound() {
    use std::os::unix::ffi::OsStrExt as _;
    let journal: PathBuf = journal("holders-a-pipe");
    let mine: HeldPlace = take_ones_place(journal.as_path()).expect("this service's place");
    let pipe: PathBuf = mine.path().parent().expect("the places directory").join("another.service");
    let named: std::ffi::CString = std::ffi::CString::new(pipe.as_os_str().as_bytes()).expect("no NUL in a fixture path");
    // SAFETY: `named` is a NUL-terminated path that outlives the call, and the mode is the record's own.
    assert_eq!(unsafe { libc::mkfifo(named.as_ptr(), 0o600) }, 0, "a pipe where a place of another service belongs");
    let (answered, heard) = std::sync::mpsc::channel::<Holders>();
    let journal_for_the_reader: PathBuf = journal.clone();
    let mine_for_the_reader: PathBuf = mine.path().to_path_buf();
    let asking: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        // The place is asked for by its path here, because a control may not move this service's place into a thread.
        let _sent = answered.send(holders_beside(journal_for_the_reader.as_path(), mine_for_the_reader.as_path()));
    });
    let waited: Result<Holders, std::sync::mpsc::RecvTimeoutError> = heard.recv_timeout(Duration::from_secs(5));
    // What: The pipe is taken away whatever the answer was.
    // Why:  Opening its writing end would wait for a reader that this control never keeps, and a control that hung
    //       there would hide the very answer it measures.
    let _removed = std::fs::remove_file(pipe.as_path());
    let answer: Holders = waited.expect("the places are answered without waiting for a writer");
    asking.join().expect("the asking thread ends");
    // What: A name that is no record is no place, and the caller's own is the only one.
    // Why:  Every place is written whole under a staging name and given its final name in one step, so a pipe under a
    //       place name is another program's and never a service that is being written now.
    assert_eq!(answer, Holders::Alone, "a pipe under a place name is no place, and the caller's own is the only one");
    drop(mine);
}
