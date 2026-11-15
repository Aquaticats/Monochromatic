//! Controls for the places services take in one journal:
//!  a service that took one is alone,
//!  a second service of the same journal makes them shared,
//!  a place whose process is gone is not a holder and is removed,
//!  and a place nothing can be read from is unknown, which is never "alone".

use super::*;

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
    assert_eq!(holders(journal.as_path()), Holders::Alone, "one service writes this journal");
    let second: HeldPlace = take_ones_place(journal.as_path()).expect("the second place");
    assert_eq!(holders(journal.as_path()), Holders::Shared, "two services write this journal");
    drop(second);
    assert_eq!(holders(journal.as_path()), Holders::Alone, "the second service ended, so the first is alone again");
    drop(first);
    assert_eq!(holders(journal.as_path()), Holders::Alone, "a journal with no places at all has one service's worth of nothing to land");
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
    assert_eq!(holders(journal.as_path()), Holders::Alone, "a place whose process is gone holds nothing");
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
    assert_eq!(holders(journal.as_path()), Holders::Unknown, "a place with no process named in it is unknown");
    let _removed = std::fs::remove_file(unreadable.as_path());
    assert_eq!(holders(journal.as_path()), Holders::Alone);
    drop(mine);
}
