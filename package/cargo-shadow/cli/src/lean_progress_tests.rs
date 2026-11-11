//! The tests of the record of where a pass stopped:
//! a position is read back,
//! a list continues after it,
//! a record that is missing or unreadable means the front of the list,
//! and neither name of the record is ever followed through a link or opened on a pipe.

use std::{fs, path::{Path, PathBuf}, sync::mpsc, time::Duration};

use rustix::fs::{CWD, FileType, Mode, mknodat};

use crate::lean_progress::{LongPass, Progress, PROGRESS_FILE_NAME, STAGING_FILE_NAME, resume_index, state_directory};
use crate::test_scratch::scratch;

/// The longest a save or a read of the record may take in these tests:
/// a record that is a pipe must answer at once rather than wait for the other end.
const ANSWER_WITHIN: Duration = Duration::from_secs(10);

/// What:
/// `fn make_pipe(path: &Path)` makes a named pipe with no reader and no writer.
/// Why:
/// A pipe at one of the record's names is the shape that holds a blocking open forever,
/// and only a pipe tests what a symbolic link or another file cannot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const makePipe = (path: string): void => execFileSync('mkfifo', [path]);
/// ```
fn make_pipe(path: &Path) {
    mknodat(CWD, path, FileType::Fifo, Mode::from_raw_mode(0o600), 0).expect("a pipe at the record's name");
}

/// What:
/// `fn within<F, T>(work: F) -> T` is the work's answer,
/// and a failure when the work has not answered inside `ANSWER_WITHIN`.
/// Why:
/// The defect these controls guard is a call that never returns,
/// so the control cannot wait for it in the test's own thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function within<T>(work: () => T, limitMs: number): Promise<T>
/// ```
fn within<F, T>(work: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(work());
    });
    return receiver.recv_timeout(ANSWER_WITHIN).expect("the call answered instead of waiting for the other end of a pipe");
}

/// Three keys in the order a pass walks them.
fn keys() -> Vec<PathBuf> {
    return vec![PathBuf::from("/base/00/aaaa"), PathBuf::from("/base/00/bbbb"), PathBuf::from("/base/01/cccc")];
}

#[test]
fn with_no_record_a_pass_starts_at_the_front() {
    assert_eq!(resume_index(&keys(), None), 0);
}

#[test]
fn a_pass_continues_after_the_key_the_last_run_stopped_at() {
    assert_eq!(resume_index(&keys(), Some(Path::new("/base/00/aaaa"))), 1);
    assert_eq!(resume_index(&keys(), Some(Path::new("/base/00/bbbb"))), 2);
}

#[test]
fn a_key_that_is_gone_still_orders_the_list() {
    // The tree a run stopped at may have been removed by that run, so the key need not be in the list.
    assert_eq!(resume_index(&keys(), Some(Path::new("/base/00/aaab"))), 1);
}

#[test]
fn a_key_past_the_end_means_the_list_is_finished() {
    assert_eq!(resume_index(&keys(), Some(Path::new("/base/02/zzzz"))), keys().len());
    assert_eq!(resume_index(&keys(), Some(Path::new("/base/01/cccc"))), keys().len());
}

#[test]
fn a_position_survives_being_written_and_read() {
    let root = scratch();
    let mut progress: Progress = Progress::default();
    progress.set(LongPass::Trees, Some(PathBuf::from("/base/00/bbbb")));
    progress.set(LongPass::Units, Some(PathBuf::from("/base/0b/ab24900891e355/debug")));
    progress.save(root.path()).expect("saved");
    let (read, notes) = crate::lean_progress::read(root.path());
    assert!(notes.is_empty(), "{notes:?}");
    assert_eq!(read.after(LongPass::Trees), Some(Path::new("/base/00/bbbb")));
    assert_eq!(read.after(LongPass::Units), Some(Path::new("/base/0b/ab24900891e355/debug")));
}

#[test]
fn a_pass_that_finished_its_list_leaves_no_position_behind() {
    let root = scratch();
    let mut progress: Progress = Progress::default();
    progress.set(LongPass::Trees, Some(PathBuf::from("/base/00/bbbb")));
    progress.save(root.path()).expect("saved");
    progress.set(LongPass::Trees, None);
    progress.save(root.path()).expect("saved");
    let (read, _notes) = crate::lean_progress::read(root.path());
    assert_eq!(read.after(LongPass::Trees), None, "the next run starts at the front");
}

#[test]
fn a_missing_record_is_the_first_run_and_is_not_a_failure() {
    let root = scratch();
    let (read, notes) = crate::lean_progress::read(root.path());
    assert_eq!(read.after(LongPass::Trees), None);
    assert_eq!(read.after(LongPass::Units), None);
    assert!(notes.is_empty(), "a first run says nothing about a record it never had: {notes:?}");
}

#[test]
fn an_unreadable_record_means_the_front_and_says_so() {
    let root = scratch();
    fs::write(root.path().join(PROGRESS_FILE_NAME), b"\xff\xfe not a record\n").expect("a record nothing can read");
    let (read, notes) = crate::lean_progress::read(root.path());
    assert_eq!(read.after(LongPass::Trees), None);
    assert_eq!(read.after(LongPass::Units), None);
    assert_eq!(notes.len(), 1, "{notes:?}");
    assert!(notes[0].contains("starts at the front"), "{notes:?}");
}

#[test]
fn a_record_of_another_shape_means_the_front() {
    let root = scratch();
    fs::write(root.path().join(PROGRESS_FILE_NAME), b"trees /base/00/bbbb\n").expect("a record of no version this build knows");
    let (read, notes) = crate::lean_progress::read(root.path());
    assert_eq!(read.after(LongPass::Trees), None);
    assert_eq!(notes.len(), 1, "{notes:?}");
}

#[test]
fn a_path_with_a_newline_in_it_is_one_line_in_the_record() {
    let root = scratch();
    let odd: PathBuf = PathBuf::from("/base/00/line\nbreak\\back");
    let mut progress: Progress = Progress::default();
    progress.set(LongPass::Trees, Some(odd.clone()));
    progress.save(root.path()).expect("saved");
    let text: String = fs::read_to_string(root.path().join(PROGRESS_FILE_NAME)).expect("the record");
    assert_eq!(text.lines().count(), 2, "the header and one tree line: {text:?}");
    let (read, _notes) = crate::lean_progress::read(root.path());
    assert_eq!(read.after(LongPass::Trees), Some(odd.as_path()));
}

#[test]
fn the_state_directory_is_the_state_home_or_the_home_for_that() {
    // The reader takes both variables from the environment, so this test states what it read.
    if let Some(found) = state_directory() {
        assert!(found.ends_with("cargo-shadow"), "{}", found.display());
    }
}

/// A link at the staging name is the tool's own name to use,
/// and never a way into the file it names.
#[test]
fn a_link_at_the_staging_name_never_rewrites_the_file_it_names() {
    let root = scratch();
    let victim: PathBuf = root.path().join("a-file-of-the-user");
    let original: &str = "unrelated user data that must survive\n";
    fs::write(&victim, original).expect("the user's file");
    std::os::unix::fs::symlink(&victim, root.path().join(STAGING_FILE_NAME)).expect("a link at the staging name");
    let mut progress: Progress = Progress::default();
    progress.set(LongPass::Trees, Some(PathBuf::from("/base/00/bbbb")));
    progress.save(root.path()).expect("the position is still recorded");
    assert_eq!(fs::read_to_string(&victim).expect("the user's file"), original, "the record is never written through a link: {}", victim.display());
    let (read, notes) = crate::lean_progress::read(root.path());
    assert!(notes.is_empty(), "{notes:?}");
    assert_eq!(read.after(LongPass::Trees), Some(Path::new("/base/00/bbbb")), "the staging name the link was at became the record");
}

/// A pipe at the staging name is removed,
/// so the record is still written and no open waits for a reader.
#[test]
fn a_pipe_at_the_staging_name_does_not_wait_for_a_reader() {
    let root = scratch();
    make_pipe(&root.path().join(STAGING_FILE_NAME));
    let mut progress: Progress = Progress::default();
    progress.set(LongPass::Trees, Some(PathBuf::from("/base/00/bbbb")));
    let directory: PathBuf = root.path().to_path_buf();
    let saved = within(move || { return progress.save(&directory); });
    saved.expect("the position is still recorded");
    let (read, notes) = crate::lean_progress::read(root.path());
    assert!(notes.is_empty(), "{notes:?}");
    assert_eq!(read.after(LongPass::Trees), Some(Path::new("/base/00/bbbb")));
}

/// A pipe at the record's own name is not a record this version reads,
/// and reading it waits for nothing.
#[test]
fn a_pipe_at_the_record_name_does_not_wait_for_a_writer() {
    let root = scratch();
    make_pipe(&root.path().join(PROGRESS_FILE_NAME));
    let directory: PathBuf = root.path().to_path_buf();
    let (read, notes) = within(move || { return crate::lean_progress::read(&directory); });
    assert_eq!(read.after(LongPass::Trees), None, "a pipe says nothing about where a pass stopped");
    assert_eq!(notes.len(), 1, "{notes:?}");
    assert!(notes[0].contains("starts at the front"), "{notes:?}");
}
