//! Controls for the report of the folder of saved copies:
//! what it reads,
//! what it says,
//! and that a folder which never answers does not hold the doctor.

use super::*;
use crate::discarded_prune::Limits;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// A day in milliseconds.
const DAY: i64 = 24 * 60 * 60 * 1000;

/// The clock the save names are made from.
const NOW: i64 = 1_793_660_000_000;

/// A directory name for a save made `age` before `NOW`,
/// the way the wrapper names it.
fn name_at(age: i64) -> String {
    return crate::iso_time::format_iso_milliseconds(NOW - age)
        .chars()
        .take(19)
        .map(|each| { return if each == ':' { '-' } else { each }; })
        .collect();
}

/// Make the directory of a save made `age` before `NOW` at `folder`,
/// holding one file,
/// and give its path.
fn make_save_at(folder: &Path, age: i64) -> PathBuf {
    let directory: PathBuf = folder.join(name_at(age));
    std::fs::create_dir_all(directory.as_path()).expect("save directory");
    std::fs::write(directory.join("one.txt"), b"copy").expect("copy");
    return directory;
}

/// What:
///  A survey that never answers ends at the ceiling with one line that says so.
/// Why:
///  The survey opens and lists the saved-copy directories,
/// and a hard-mounted network file system whose server
///  stopped answering holds such a call in the kernel with no end;
/// the owner's rule is "nothing should ever wait forever or
///  indefinitely",
/// and a section silently left out would read as "no saved copies here".
#[test]
fn a_saved_copy_folder_that_never_answers_gives_up_at_its_ceiling() {
    let root: PathBuf = fixture("discarded-report-stalled");
    let (done, finished) = std::sync::mpsc::channel::<String>();
    let named: PathBuf = root.clone();
    let worker = std::thread::spawn(move || {
        let text: String = doctor_section_with(stalled_survey, Duration::from_millis(50), named.as_path(), Limits::default());
        let _sent = done.send(text);
    });
    // The worker is left behind on purpose: a survey that never answers has no end, and the test process ends it.
    let answered: Result<String, _> = finished.recv_timeout(Duration::from_secs(10));
    let _abandoned = worker;
    let text: String = answered.unwrap_or_else(|_| { panic!("the survey never returned, so a folder on a mount that stopped answering holds the doctor") });
    assert!(text.contains("could not be read within"), "{text}");
    assert!(text.contains("discarded copies:"), "{text}");
    remove(root.as_path());
}

/// A survey that never answers,
/// as one on a mount whose server stopped answering does not.
///
/// The thread parks,
/// so it spends no processor time while it waits:
/// this is a stall,
/// not a load.
fn stalled_survey(_common_dir: &Path) -> SavedCopies {
    loop {
        std::thread::park();
    }
}

/// What:
///  The report names the folder it read and the copies it holds,
/// and says nothing when no folder holds a copy.
/// Why:
///  This is the wording the person reads,
/// and the reading is bounded but not changed.
#[test]
fn the_report_names_the_folder_and_what_it_holds() {
    let common: PathBuf = fixture("discarded-report-text");
    assert_eq!(doctor_section(common.as_path(), Limits::default()), "");
    make_save_at(common.join(SAVED_DIRECTORY).as_path(), 3 * DAY);
    let text: String = doctor_section(common.as_path(), Limits { max_age_days: 7, max_bytes: 2048 });
    for part in ["discarded copies:", "1 copies", "1 saves", "older than 7 days", "past 2048 bytes", "discardedCopies.maxAgeDays"] {
        assert!(text.contains(part), "{part} in {text}");
    }
    assert!(text.contains(common.join(SAVED_DIRECTORY).display().to_string().as_str()), "{text}");
    remove(common.as_path());
}

/// What:
///  Saved copies under both data-folder names are both reported,
/// including when the current folder exists and holds none.
/// Why:
///  The read-only report chose one folder,
/// and a repository that holds both names is a supported state:
/// the
///  earlier folder's copies are a person's recoverable data,
/// so a report that named only the current folder hid them from
///  the person who asked what is there.
#[test]
fn saved_copies_under_both_folder_names_are_both_reported() {
    use crate::wrapper_names_old_and_new::DATA_FOLDER;
    let common: PathBuf = fixture("discarded-report-both");
    let current: PathBuf = common.join(SAVED_DIRECTORY);
    let earlier: PathBuf = common.join(DATA_FOLDER.earlier).join("discarded");
    let kept: PathBuf = make_save_at(earlier.as_path(), 9 * DAY);
    // The current folder exists and holds no saved copy, which is the state that hid the older one.
    std::fs::create_dir_all(current.as_path()).expect("the current folder");
    let text: String = doctor_section(common.as_path(), Limits::default());
    assert!(text.contains(earlier.display().to_string().as_str()), "the earlier folder is named: {text}");
    assert!(text.contains("1 copies"), "the copies are counted: {text}");
    assert!(text.contains("1 saves"), "the save is counted: {text}");
    assert!(text.contains(kept.file_name().expect("name").to_string_lossy().as_ref()), "the kept save is named: {text}");
    assert!(!text.contains("could not be read"), "{text}");
    let merged: Report = report(common.as_path()).expect("a report of both folders");
    assert_eq!((merged.copies, merged.saves), (1, 1), "the copies of both folders are counted");
    remove(common.as_path());
}
