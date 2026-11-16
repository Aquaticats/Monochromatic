//! What:
//!  The record a commit leaves in the service journal,
//!  run through the built wrapper on a real repository:
//!  paths that hold every byte a name may hold,
//!  the record as bytes,
//!  a restart that finishes the batch it names,
//!  and the temporary files a kill leaves.
//! Why:
//!   A record is the only thing a restart has after a kill,
//! so every byte of it has to name what the commit said and nothing else,
//!  and a file a kill left has to be found by the wrapper itself later.
//! These controls run the program a person runs,
//!  in a fixture with its own home and runtime directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const landed = await runCommit(fixture, repo, ['commit', '-m', 'hostile paths']); await readJournalBytes(fixture)
//! ```

use super::commit::{COMMIT_BOUND_SECONDS, assert_nothing_left, remove_settled};
use super::support::{Fixture, Observed, bounded_for, fixture, git, observe, repository, stderr_of, wrapper_first_path};
use git_policy_cli::service_fault::{BEFORE_A_CACHE_RENAME, BEFORE_AN_INTENT_RENAME, FAULT_VARIABLE};
use git_policy_cli::service_journal::{DONE_EXTENSION, Intent, OPEN_EXTENSION, batch_path, parse_intent};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The instant a landing has moved the branch and not yet installed the index.
/// Spelled here rather than read from the product, because the batch's points are the product's own list
/// (`service_fault::FAULT_POINTS`) and a control that killed at one of them by name would fail on a list that changed.
const AFTER_THE_BRANCH_MOVED: &str = "after-ref";

/// The names one commit stages.
/// What each holds:
/// a tab and a carriage return, which the protocol carries,
/// an `=`, a space, a quote, a backslash, a leading dash, a `%`,
/// a byte that is not text,
/// and a letter that is not ASCII.
fn hostile_names() -> Vec<Vec<u8>> {
    return vec![
        b"a\ttab.txt".to_vec(),
        b"carriage\rreturn.txt".to_vec(),
        b"equals=sign.txt".to_vec(),
        b"a space.txt".to_vec(),
        b"quote'and\"apostrophe.txt".to_vec(),
        b"back\\slash.txt".to_vec(),
        b"-leading-dash.txt".to_vec(),
        b"100%done.txt".to_vec(),
        b"not\xfftext.txt".to_vec(),
        "caf\u{e9}.txt".as_bytes().to_vec(),
        b"plain.txt".to_vec(),
    ];
}

/// The name of a file, as the operating system holds it.
fn name_of(bytes: &[u8]) -> OsString {
    use std::os::unix::ffi::OsStrExt;
    return OsStr::from_bytes(bytes).to_owned();
}

/// What:
///  A file with `name` in `repo`,
///  staged with real Git,
///  and nothing else.
/// Why:
///   The commit through the wrapper reads the index the person staged,
/// so the fixture stages exactly the names the control is about.
fn stage_a_file(fixture: &Fixture, repo: &Path, name: &[u8]) {
    use std::os::unix::ffi::OsStrExt;
    let path: PathBuf = repo.join(OsStr::from_bytes(name));
    std::fs::write(&path, b"content\n").expect("the file");
    let arguments: Vec<OsString> = vec![OsString::from("add"), OsString::from("--"), name_of(name)];
    git(fixture, repo, arguments.as_slice());
}

/// A commit of the named files through the wrapper, under the commit bound.
fn commit_of(fixture: &Fixture, repo: &Path, names: &[Vec<u8>], message: &str) -> Observed {
    use std::os::unix::ffi::OsStrExt;
    let mut arguments: Vec<OsString> = vec![OsString::from("commit"), OsString::from(format!("--message={message}")), OsString::from("--")];
    for name in names {
        arguments.push(OsStr::from_bytes(name).to_owned());
    }
    let mut command: std::process::Command = bounded_for(fixture, fixture.root.join("bin/git").as_path(), wrapper_first_path(fixture).as_os_str(), COMMIT_BOUND_SECONDS);
    command.current_dir(repo).args(arguments);
    return observe(&mut command, b"");
}

/// A commit through the wrapper, under the commit bound.
fn commit(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> Observed {
    let mut command: std::process::Command = bounded_for(fixture, fixture.root.join("bin/git").as_path(), wrapper_first_path(fixture).as_os_str(), COMMIT_BOUND_SECONDS);
    command.current_dir(repo).args(arguments);
    return observe(&mut command, b"");
}

/// The names in an order two lists of them can be compared in.
/// Why:
///   The record names the paths in the index's order, which is the order Git sorts them in,
/// so a control compares the names themselves and not the order Git chose.
fn sorted(mut names: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    names.sort();
    return names;
}

/// What:
///  Waits, bounded, for a path to be gone,
///  and answers whether it is.
/// Why:
///   The sweep runs on another process's own thread after the service starts,
/// so the event is the removal and never a pause of a guessed length.
fn wait_until_gone(path: &Path) -> bool {
    let began: Instant = Instant::now();
    while began.elapsed() < Duration::from_secs(60) {
        if !path.exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    return false;
}

/// What:
///  The one record of a batch in the journal of a fixture,
///  with its bytes.
/// Why:
///   A batch that landed is a finished record and one a kill interrupted is an open one,
/// so the control says which of the two it means and never guesses.
fn the_batch_record(fixture: &Fixture, extension: &str) -> (PathBuf, Vec<u8>) {
    let journal: PathBuf = fixture.root.join("journal");
    let mut open: Vec<PathBuf> = std::fs::read_dir(journal.as_path())
        .expect("the journal")
        .flatten()
        .map(|entry| { return entry.path(); })
        .filter(|path| { return path.extension().and_then(OsStr::to_str) == Some(extension); })
        .collect();
    assert_eq!(open.len(), 1, "one batch of one commit is {extension}: {open:?}");
    let found: PathBuf = open.remove(0);
    let bytes: Vec<u8> = std::fs::read(&found).expect("the record's bytes");
    return (found, bytes);
}

/// What:
///  A commit of every name the protocol carries,
///  and the record it left,
///  read as bytes.
/// Why:
///   A record is lines and fields,
///  so a name that holds one of those ends its line or its field;
///  the record has to name every byte of the name anyway,
///  and it must not read as a field the commit never wrote.
#[test]
fn a_commit_of_hostile_names_leaves_a_record_that_names_each_of_them() {
    let fixture: Fixture = fixture("journal-hostile-names");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let names: Vec<Vec<u8>> = hostile_names();
    for name in &names {
        stage_a_file(&fixture, repo.as_path(), name);
    }
    let landed: Observed = commit_of(&fixture, repo.as_path(), names.as_slice(), "hostile names");
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    let (_record, bytes) = the_batch_record(&fixture, DONE_EXTENSION);
    let text: String = String::from_utf8_lossy(bytes.as_slice()).into_owned();
    // The record as it is written: one line per field, so a name that holds a tab adds no line, and no line it did not write is
    //  one a reader takes for a field of the record.
    let forged: Vec<&str> = text.lines().filter(|line| { return line.starts_with("cleanup") || line.starts_with("index=") || line.starts_with("gitdir="); }).collect();
    assert_eq!(forged.len(), 2, "the record holds the index and the Git directory lines it wrote and no cleanup line: {forged:?}");
    let read: Intent = parse_intent(text.as_str()).expect("the record reads");
    assert_eq!(sorted(read.paths.clone()), sorted(names.clone()), "every byte of every name is a path the record names; the record is:\n{text}");
    assert_eq!(read.index, repo.join(".git/index"), "the index is the one the commit was prepared against");
    assert_eq!(read.git_dir, repo.join(".git"), "the Git directory is the one the commit was made in");
    assert_eq!(read.cleanup, None, "the commit named no conclusion state to remove");
    // And the file the person made is the file the commit holds, under its own name.
    let listed: Observed = super::support::run_direct(&fixture, repo.as_path(), &["ls-tree", "--name-only", "-z", "HEAD"]);
    let mut shown: Vec<Vec<u8>> = listed.stdout.split(|byte| { return *byte == b'\0'; }).filter(|name| { return !name.is_empty(); }).map(<[u8]>::to_vec).collect();
    shown.sort();
    let mut expected: Vec<Vec<u8>> = names.clone();
    expected.sort();
    assert_eq!(shown, expected, "the commit holds every name, byte for byte");
    remove_settled(&fixture);
}

/// What:
///  A name that holds a line break cannot travel in the request at all,
///  so the batch that lands such a commit carries no list of paths,
///  and the journal has no line a name could have broken.
/// Why:
///   This is the answer to the reading that a committed path could end the `paths=` line:
/// the protocol refuses a path with a line break,
///  the commit travels as one Git built in a scratch repository (`derived`),
///  and the service reads its index entries from the difference between the two commits.
#[test]
fn a_name_that_holds_a_line_break_travels_as_a_commit_git_built() {
    let fixture: Fixture = fixture("journal-newline-name");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let name: &[u8] = b"line\nbreak.txt";
    stage_a_file(&fixture, repo.as_path(), name);
    let landed: Observed = commit_of(&fixture, repo.as_path(), &[name.to_vec()], "a line break in a name");
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    let (_record, bytes) = the_batch_record(&fixture, DONE_EXTENSION);
    let read: Intent = parse_intent(String::from_utf8_lossy(bytes.as_slice()).as_ref()).expect("the record reads");
    assert!(read.derived, "the batch names no list of paths, so its index entries come from the difference between the commits");
    assert!(read.paths.is_empty(), "no path travelled, so no line of the record could have been broken: {:?}", read.paths);
    let listed: Observed = super::support::run_direct(&fixture, repo.as_path(), &["ls-tree", "--name-only", "-z", "HEAD"]);
    assert_eq!(listed.stdout, b"line\nbreak.txt\0".to_vec(), "and the commit holds the name whole");
    remove_settled(&fixture);
}

/// What:
///  A service killed after its branch moved leaves the batch's record open,
///  and the next service finishes it:
///  the commit is on the branch once,
///  the index shows it,
///  and the record named every byte of the hostile names.
/// Why:
///   The record is all a restart has,
/// so the paths, the index and the Git directory it names are what the index update is made for;
/// a restart that read a name of its own would update another entry.
#[test]
fn a_service_killed_after_the_branch_moved_finishes_the_batch_of_hostile_names() {
    let fixture: Fixture = fixture("journal-recovery-hostile");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let names: Vec<Vec<u8>> = hostile_names();
    for name in &names {
        stage_a_file(&fixture, repo.as_path(), name);
    }
    // The fault is armed before any service of this fixture starts, so the service this commit reaches is the one it kills.
    fixture
        .service_environment
        .lock()
        .expect("the fixture's service environment")
        .push((OsString::from(FAULT_VARIABLE), OsString::from(AFTER_THE_BRANCH_MOVED)));
    let killed: Observed = commit_of(&fixture, repo.as_path(), names.as_slice(), "killed after the branch moved");
    assert_ne!(killed.code, Some(0), "the service was killed inside the landing: {}", stderr_of(&killed));
    // The branch moved and the record is still open: the state a kill at that instant leaves.
    let head: Observed = super::support::run_direct(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    assert_eq!(String::from_utf8_lossy(head.stdout.as_slice()).trim().len(), 40, "the branch holds a commit");
    let (record, bytes) = the_batch_record(&fixture, OPEN_EXTENSION);
    let read: Intent = parse_intent(String::from_utf8_lossy(bytes.as_slice()).as_ref()).expect("the open record reads");
    assert_eq!(sorted(read.paths.clone()), sorted(names.clone()), "the killed batch named every byte of every name");
    assert_eq!(read.new, String::from_utf8_lossy(head.stdout.as_slice()).trim(), "and the commit the branch holds");
    // The service of the next command settles what the kill left; the work records of the killed batch go with it.
    fixture.service_environment.lock().expect("the fixture's service environment").clear();
    super::support::restart_ended_services(&fixture);
    assert!(wait_until_gone(record.as_path()), "the restarted service finished the batch and its record is no longer open");
    let status: String = String::from_utf8_lossy(super::support::run_direct(&fixture, repo.as_path(), &["status", "--short"]).stdout.as_slice()).into_owned();
    assert_eq!(status, "", "the index shows the commit the branch holds: {status}");
    let count: String = String::from_utf8_lossy(super::support::run_direct(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).stdout.as_slice()).into_owned();
    assert_eq!(count.trim(), "2", "the commit landed once");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// What:
///  A service killed with a batch's record written whole and not yet published leaves the file it wrote,
///  and the next listing of that journal proves its writer gone and removes it.
/// Why:
///   A rename publishes a record,
/// so a kill between the write and the rename leaves a file nothing names;
/// its name carries the writing process,
///  and the kernel's answer that no such process exists is what lets the next service remove it.
#[test]
fn a_record_written_and_not_published_is_removed_by_the_next_service() {
    let fixture: Fixture = fixture("journal-temporary-of-a-killed-write");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    fixture
        .service_environment
        .lock()
        .expect("the fixture's service environment")
        .push((OsString::from(FAULT_VARIABLE), OsString::from(BEFORE_AN_INTENT_RENAME)));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a file");
    let arguments: Vec<OsString> = vec![OsString::from("add"), OsString::from("--"), OsString::from("a.txt")];
    git(&fixture, repo.as_path(), arguments.as_slice());
    let killed: Observed = commit_of(&fixture, repo.as_path(), &[b"a.txt".to_vec()], "killed before the record was published");
    assert_ne!(killed.code, Some(0), "the service was killed between the write and its rename: {}", stderr_of(&killed));
    let journal: PathBuf = fixture.root.join("journal");
    let temporaries: Vec<PathBuf> = std::fs::read_dir(journal.as_path())
        .expect("the journal")
        .flatten()
        .map(|entry| { return entry.path(); })
        .filter(|path| { return path.extension().and_then(OsStr::to_str) == Some("tmp"); })
        .collect();
    assert_eq!(temporaries.len(), 1, "the kill left the write's own file: {temporaries:?}");
    let name: String = temporaries[0].file_name().and_then(OsStr::to_str).expect("a name").to_owned();
    assert!(git_policy_cli::temporary_file_of_a_writer::writer_of(name.as_str()).is_some(), "the name carries the process that wrote it: {name}");
    assert!(!batch_path(journal.as_path(), 1, OPEN_EXTENSION).exists(), "and no record was published");
    // The next service lists the journal at its start, which is where the remains of a killed write are found.
    fixture.service_environment.lock().expect("the fixture's service environment").clear();
    super::support::restart_ended_services(&fixture);
    assert!(wait_until_gone(temporaries[0].as_path()), "the next service removed the file a writer that is gone left");
    remove_settled(&fixture);
}

/// What:
///  A wrapper killed between writing the version cache and publishing it leaves its temporary,
///  and the next command that asks Git the version removes it.
/// Why:
///   The cache is a file the wrapper writes on its own path with no service at all,
/// so a kill there is the one no service can settle,
///  and the write that follows is the one place that finds it.
#[test]
fn the_version_cache_a_kill_left_is_removed_by_the_next_command() {
    let fixture: Fixture = fixture("cache-temporary-of-a-killed-write");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let cache: PathBuf = fixture.root.join("home/.cache/monochromatic-git-policy-cli");
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a file");
    let arguments: Vec<OsString> = vec![OsString::from("add"), OsString::from("--"), OsString::from("a.txt")];
    git(&fixture, repo.as_path(), arguments.as_slice());
    let mut armed: std::process::Command = bounded_for(&fixture, fixture.root.join("bin/git").as_path(), wrapper_first_path(&fixture).as_os_str(), COMMIT_BOUND_SECONDS);
    armed.current_dir(repo.as_path()).args(["commit", "--no-only", "--message=killed before the cache was published"]).env(FAULT_VARIABLE, BEFORE_A_CACHE_RENAME);
    let killed: Observed = observe(&mut armed, b"");
    assert_ne!(killed.code, Some(0), "the wrapper was killed between the write and its rename: {}", stderr_of(&killed));
    let temporaries: Vec<PathBuf> = std::fs::read_dir(cache.as_path())
        .expect("the cache directory")
        .flatten()
        .map(|entry| { return entry.path(); })
        .filter(|path| { return path.extension().and_then(OsStr::to_str) == Some("tmp"); })
        .collect();
    assert_eq!(temporaries.len(), 1, "the kill left the write's own file: {temporaries:?}");
    let landed: Observed = commit(&fixture, repo.as_path(), &["commit", "--no-only", "--message=the next command"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert!(!temporaries[0].exists(), "the next command removed the file a writer that is gone left");
    let entries: Vec<String> = std::fs::read_dir(cache.as_path())
        .expect("the cache directory")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .collect();
    assert!(entries.iter().any(|name| { return name.starts_with("git-version-") && !name.ends_with(".tmp"); }), "and wrote the entry it asked for: {entries:?}");
    remove_settled(&fixture);
}
