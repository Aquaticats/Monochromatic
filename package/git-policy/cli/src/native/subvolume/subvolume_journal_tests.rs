//! Controls for the migration journal:
//! an empty start,
//! the states and their predicates,
//! the atomic write,
//! the reload,
//!  every shape of a file that is not a journal,
//! and the deletion.

use super::*;
use crate::test_support::{fixture, remove};

/// A fresh journal path in a fixture.
fn journal_path(name: &str) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let path: PathBuf = parent.join(".journal.json");
    return (parent, path);
}

/// No file means an empty journal that is not persisted and answers "not begun" to every step.
#[test]
fn a_missing_file_is_an_empty_journal() {
    let (parent, path) = journal_path("journal-empty");
    let journal: Journal = Journal::load(path.as_path()).expect("load");
    assert!(!journal.is_persisted() && journal.state("copy").is_none());
    assert!(!journal.is_done("copy") && !journal.has_begun("copy"));
    assert_eq!(journal.path(), path.as_path());
    remove(parent.as_path());
}

/// `started` has begun but is not done;
/// `done` is both;
/// `rolled-back` is neither.
#[test]
fn the_states_answer_the_predicates() {
    let (parent, path) = journal_path("journal-states");
    let mut journal: Journal = Journal::load(path.as_path()).expect("load");
    journal.mark("copy", "started").expect("mark");
    assert!(journal.has_begun("copy") && !journal.is_done("copy"));
    journal.mark("copy", "done").expect("mark");
    assert!(journal.has_begun("copy") && journal.is_done("copy"));
    journal.mark("exchange", "rolled-back").expect("mark");
    assert!(!journal.has_begun("exchange") && !journal.is_done("exchange"));
    assert_eq!(journal.state("exchange"), Some("rolled-back"));
    assert!(journal.is_persisted());
    remove(parent.as_path());
}

/// What was marked is read back by a new load,
/// and no temporary file stays beside the journal.
#[test]
fn a_marked_step_survives_a_reload_and_leaves_no_temporary_file() {
    let (parent, path) = journal_path("journal-reload");
    let mut journal: Journal = Journal::load(path.as_path()).expect("load");
    journal.mark("create", "done").expect("mark");
    journal.mark("copy", "started").expect("mark");
    let reloaded: Journal = Journal::load(path.as_path()).expect("reload");
    assert!(reloaded.is_persisted() && reloaded.is_done("create") && reloaded.has_begun("copy") && !reloaded.is_done("copy"));
    assert_eq!(reloaded.started, journal.started);
    let names: Vec<String> = std::fs::read_dir(parent.as_path()).expect("listing").flatten().map(|item| { return item.file_name().to_string_lossy().into_owned(); }).collect();
    assert_eq!(names, vec![String::from(".journal.json")]);
    remove(parent.as_path());
}

/// Every file that is not a journal of this format is an error that names the file;
/// nothing is guessed.
#[test]
fn a_file_that_is_not_a_journal_is_refused_by_name() {
    let (parent, path) = journal_path("journal-broken");
    let cases: [(&str, &str); 6] = [
        ("not json", "is not valid JSON"),
        ("{\"schema\": 2, \"started\": \"x\", \"steps\": {}}", "unknown schema version"),
        ("{\"started\": \"x\", \"steps\": {}}", "unknown schema version"),
        ("{\"schema\": 1, \"steps\": {}}", "has no start time"),
        ("{\"schema\": 1, \"started\": \"x\"}", "has no steps"),
        ("{\"schema\": 1, \"started\": \"x\", \"steps\": {\"copy\": {}}}", "step without a state"),
    ];
    for (text, why) in cases {
        std::fs::write(path.as_path(), text).expect("write");
        let error: MigrateError = Journal::load(path.as_path()).expect_err("refused");
        let shown: String = error.to_string();
        assert!(shown.contains(why) && shown.contains(path.to_str().expect("path")), "{text}: {shown}");
    }
    remove(parent.as_path());
}

/// A path that cannot be read as a file (a directory) is an input and output error,
/// not an empty journal.
#[test]
fn an_unreadable_journal_is_an_error_not_an_empty_one() {
    let (parent, path) = journal_path("journal-directory");
    std::fs::create_dir(path.as_path()).expect("a directory where the journal would be");
    let error: MigrateError = Journal::load(path.as_path()).expect_err("refused");
    assert!(error.to_string().contains("cannot read the journal"), "{error}");
    remove(parent.as_path());
}

/// A write that cannot happen is reported with the journal's path and leaves the memory state recorded.
#[test]
fn a_journal_that_cannot_be_written_is_reported() {
    let parent: PathBuf = fixture("journal-unwritable");
    let path: PathBuf = parent.join("absent-directory").join(".journal.json");
    let mut journal: Journal = Journal::load(path.as_path()).expect("load");
    let error: MigrateError = journal.mark("copy", "started").expect_err("no directory to write in");
    assert!(error.to_string().contains("cannot write the journal"), "{error}");
    assert!(!journal.is_persisted());
    remove(parent.as_path());
}

/// Deleting removes the file and forgets the steps;
/// deleting twice is not an error.
#[test]
fn delete_forgets_the_steps_and_is_repeatable() {
    let (parent, path) = journal_path("journal-delete");
    let mut journal: Journal = Journal::load(path.as_path()).expect("load");
    journal.mark("copy", "done").expect("mark");
    journal.delete().expect("delete");
    assert!(!path.exists() && !journal.is_persisted() && !journal.is_done("copy"));
    journal.delete().expect("a second delete");
    remove(parent.as_path());
}

/// A path that is a non-empty directory cannot be deleted as a file,
/// and the error names it.
#[test]
fn a_journal_path_that_cannot_be_removed_is_reported() {
    let (parent, path) = journal_path("journal-undeletable");
    std::fs::create_dir(path.as_path()).expect("directory");
    std::fs::write(path.join("inside"), b"x").expect("file");
    let mut journal: Journal = Journal { path: path.clone(), started: String::from("x"), states: BTreeMap::new(), times: BTreeMap::new(), persisted: true };
    let error: MigrateError = journal.delete().expect_err("a directory is not a file");
    assert!(error.to_string().contains("cannot delete the journal"), "{error}");
    remove(parent.as_path());
}

/// Each step keeps the time it was recorded at when a later step is marked,
/// and a record written without a time reads as
///  having none.
#[test]
fn a_step_keeps_its_own_time_and_an_old_record_has_none() {
    let directory = crate::test_support::fixture("journal-times");
    let path = directory.join("journal.json");
    let mut journal: Journal = Journal::load(path.as_path()).expect("empty");
    assert_eq!(journal.seconds_of("copy"), None);
    journal.mark("copy", "done").expect("mark");
    let first: i64 = journal.seconds_of("copy").expect("time");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    journal.mark("delta1", "done").expect("mark");
    let reloaded: Journal = Journal::load(path.as_path()).expect("reload");
    assert_eq!(reloaded.seconds_of("copy"), Some(first), "marking another step does not move this step's time");
    assert!(reloaded.seconds_of("delta1") > Some(first));
    std::fs::write(path.as_path(), br#"{"schema":1,"started":"x","steps":{"copy":{"state":"done","at":"y"}}}"#).expect("an older journal");
    assert_eq!(Journal::load(path.as_path()).expect("load").seconds_of("copy"), None);
    crate::test_support::remove(directory.as_path());
}

/// The start time is the one the journal file recorded,
/// a resumed run keeps it,
/// and a fresh journal has a time of its own
///  (ISO text),
/// not an empty or a placeholder one.
#[test]
fn the_start_time_is_the_recorded_one_and_a_fresh_journal_has_its_own() {
    let (parent, path) = journal_path("journal-started");
    let fresh: Journal = Journal::load(path.as_path()).expect("load");
    let began: &str = fresh.started();
    assert!(began.len() >= "2026-01-01T00:00:00Z".len() && began.contains('T') && began.ends_with('Z'), "{began}");
    let recorded: &str = "2026-10-31T12:34:56.789Z";
    std::fs::write(path.as_path(), format!("{{\"schema\":1,\"started\":\"{recorded}\",\"steps\":{{}}}}")).expect("a journal");
    let mut loaded: Journal = Journal::load(path.as_path()).expect("load");
    assert_eq!(loaded.started(), recorded);
    loaded.mark("copy", "started").expect("mark");
    assert_eq!(Journal::load(path.as_path()).expect("reload").started(), recorded, "a resumed run keeps the start time");
    remove(parent.as_path());
}

/// The fixed sibling name that releases up to 0.5.20 wrote the journal through:
/// the journal's path with `.tmp` appended.
fn old_fixed_sibling(journal: &Path) -> PathBuf {
    let mut name: std::ffi::OsString = journal.as_os_str().to_os_string();
    name.push(".tmp");
    return PathBuf::from(name);
}

/// A leftover of a killed run in the shape the writer names its temporary files:
/// the journal's path,
/// `.tmp.`,
/// a process
///  number and a clock reading.
fn leftover_sibling(journal: &Path, process: &str, clock: &str) -> PathBuf {
    let mut name: std::ffi::OsString = journal.as_os_str().to_os_string();
    name.push(format!(".tmp.{process}.{clock}"));
    return PathBuf::from(name);
}

/// Red until the write is exclusive:
/// a symbolic link planted at the old fixed sibling name must not be followed,
/// so the
///  file it points at keeps its bytes and the journal still lands.
#[test]
fn a_link_planted_at_the_old_sibling_name_is_not_written_through() {
    let (parent, path) = journal_path("journal-planted-link");
    let victim: PathBuf = parent.join("victim.txt");
    let precious: &[u8] = b"user data that no journal write may touch";
    std::fs::write(victim.as_path(), precious).expect("victim");
    let planted: PathBuf = old_fixed_sibling(path.as_path());
    std::os::unix::fs::symlink(victim.as_path(), planted.as_path()).expect("plant the link");
    let mut journal: Journal = Journal::load(path.as_path()).expect("load");
    journal.mark("copy", "started").expect("the journal is written beside the planted link");
    assert_eq!(std::fs::read(victim.as_path()).expect("victim"), precious, "the planted link was written through");
    assert_eq!(std::fs::read_link(planted.as_path()).expect("the link is left where it was"), victim);
    assert!(Journal::load(path.as_path()).expect("reload").has_begun("copy"));
    remove(parent.as_path());
}

/// Red until the write is exclusive:
/// a dangling link planted at the old fixed sibling name must not make the write create
///  the file it points at.
#[test]
fn a_dangling_link_planted_at_the_old_sibling_name_creates_nothing() {
    let (parent, path) = journal_path("journal-dangling-link");
    let aimed_at: PathBuf = parent.join("created-through-the-link.txt");
    let planted: PathBuf = old_fixed_sibling(path.as_path());
    std::os::unix::fs::symlink(aimed_at.as_path(), planted.as_path()).expect("plant the link");
    let mut journal: Journal = Journal::load(path.as_path()).expect("load");
    journal.mark("copy", "started").expect("the journal is written beside the planted link");
    assert!(std::fs::symlink_metadata(aimed_at.as_path()).is_err(), "the write created the file the link names");
    assert!(Journal::load(path.as_path()).expect("reload").has_begun("copy"));
    remove(parent.as_path());
}

/// Red until leftovers are swept:
/// the plain files a killed run left under the writer's own temporary names are removed once
///  a later write lands,
/// while a file under any other name,
/// a directory under a temporary name and a link under one stay.
#[test]
fn leftovers_of_a_killed_run_are_swept_and_nothing_else_is() {
    let (parent, path) = journal_path("journal-leftovers");
    let leftover: PathBuf = leftover_sibling(path.as_path(), "4242", "1700000000000000000");
    let old_fixed: PathBuf = old_fixed_sibling(path.as_path());
    std::fs::write(leftover.as_path(), b"half a journal").expect("leftover");
    std::fs::write(old_fixed.as_path(), b"half a journal").expect("old leftover");
    let notes: PathBuf = leftover_sibling(path.as_path(), "notes", "by-hand");
    std::fs::write(notes.as_path(), b"somebody's notes").expect("notes");
    let directory: PathBuf = leftover_sibling(path.as_path(), "4243", "1700000000000000001");
    std::fs::create_dir(directory.as_path()).expect("directory under a temporary name");
    let link: PathBuf = leftover_sibling(path.as_path(), "4244", "1700000000000000002");
    std::os::unix::fs::symlink(notes.as_path(), link.as_path()).expect("link under a temporary name");
    let mut journal: Journal = Journal::load(path.as_path()).expect("load");
    journal.mark("copy", "started").expect("mark");
    assert!(std::fs::symlink_metadata(leftover.as_path()).is_err(), "the leftover plain file stayed");
    assert!(std::fs::symlink_metadata(old_fixed.as_path()).is_err(), "the old fixed sibling stayed");
    assert_eq!(std::fs::read(notes.as_path()).expect("notes"), b"somebody's notes");
    assert!(directory.is_dir(), "a directory is never swept");
    assert!(std::fs::symlink_metadata(link.as_path()).expect("link").file_type().is_symlink(), "a link is never swept");
    remove(parent.as_path());
}

/// The exclusive write refuses every kind of path that already exists,
/// and leaves it as it was:
/// a plain file keeps its
///  bytes,
/// a link is not followed and a dangling link creates nothing.
#[test]
fn the_exclusive_write_refuses_anything_already_there() {
    let (parent, _path) = journal_path("journal-exclusive");
    let plain: PathBuf = parent.join("plain");
    std::fs::write(plain.as_path(), b"kept").expect("plain file");
    let victim: PathBuf = parent.join("victim");
    std::fs::write(victim.as_path(), b"kept").expect("victim");
    let link: PathBuf = parent.join("link");
    std::os::unix::fs::symlink(victim.as_path(), link.as_path()).expect("link");
    let aimed_at: PathBuf = parent.join("never-created");
    let dangling: PathBuf = parent.join("dangling");
    std::os::unix::fs::symlink(aimed_at.as_path(), dangling.as_path()).expect("dangling link");
    for occupied in [&plain, &link, &dangling] {
        let error: std::io::Error = write_exclusively(occupied.as_path(), b"journal").expect_err("refused");
        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists, "{}", occupied.display());
    }
    assert_eq!(std::fs::read(plain.as_path()).expect("plain"), b"kept");
    assert_eq!(std::fs::read(victim.as_path()).expect("victim"), b"kept");
    assert!(std::fs::symlink_metadata(aimed_at.as_path()).is_err());
    remove(parent.as_path());
}

/// The temporary file is private to its owner,
/// and the names of the writer's leftovers are recognized exactly:
/// the old fixed
///  name and the name with a process number and a clock reading,
/// nothing longer,
/// shorter or non-numeric.
#[test]
fn the_temporary_file_is_private_and_only_the_writers_own_names_are_leftovers() {
    let (parent, _path) = journal_path("journal-shapes");
    let written: PathBuf = parent.join("written");
    write_exclusively(written.as_path(), b"journal").expect("written");
    let mode: u32 = std::os::unix::fs::PermissionsExt::mode(&std::fs::metadata(written.as_path()).expect("metadata").permissions());
    assert_eq!(mode & 0o777, PRIVATE_FILE_MODE);
    let name: &[u8] = b"j.json";
    for own in ["j.json.tmp", "j.json.tmp.1.2", "j.json.tmp.123456.1700000000000000000"] {
        assert!(is_leftover_name(own.as_bytes(), name), "{own}");
    }
    for foreign in ["j.json", "j.json.tmp.", "j.json.tmp.1", "j.json.tmp.1.", "j.json.tmp..2", "j.json.tmp.1.2.3", "j.json.tmp.a.2", "j.json.tmpx", "j.json.old", "x.j.json.tmp", "j.json.tmp.1.2x"] {
        assert!(!is_leftover_name(foreign.as_bytes(), name), "{foreign}");
    }
    remove(parent.as_path());
}

/// What:
///  A named pipe where the journal belongs is refused at once,
///  naming the file,
/// instead of waiting for a writer
///  that never comes.
/// Why:
///   Opening a named pipe for reading waits until something writes to it,
/// and the command that waits holds the
///  claim of the checkout,
/// so every later command on it is refused as busy with a process that never ends.
/// The
///  conversion's reader already refuses such a file;
/// the migration's must too.
#[test]
fn a_named_pipe_where_the_journal_belongs_is_refused_at_once() {
    let parent: PathBuf = fixture("journal-fifo");
    let path: PathBuf = parent.join(".cli-git-migrate-checkout.journal.json");
    crate::subvolume_sync_attrs::make_fifo(path.as_path(), 0o600).expect("named pipe");
    let read: Option<Result<Journal, MigrateError>> = crate::test_support::finished_within(move || { return Journal::load(path.as_path()); });
    let Some(loaded) = read else { panic!("reading the journal did not finish within five seconds") };
    let reason: String = loaded.expect_err("a named pipe is not a journal").to_string();
    assert!(reason.contains("not a plain file"), "{reason}");
    remove(parent.as_path());
}
