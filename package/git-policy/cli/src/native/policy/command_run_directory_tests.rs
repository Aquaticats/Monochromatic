//! What:
//!  Controls for the directory a command runs in:
//!  the directory Git puts the shell of a `!` alias in,
//!  and the caller's own directory moved by its `-C` chain for every other command.
//! Why:
//!  A rule that reads a relative path of a command,
//!  or starts a measurement on its behalf,
//!  names another file or another subtree with either answer wrong,
//!  and a directory that cannot be named must never fall back to the caller's.

use super::command_run_directory;
use crate::test_support::{fixture, remove};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Arguments as the wrapper receives them.
fn args(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(OsString::from).collect();
}

/// A command the shell of a `!` alias runs is measured in the shell's directory,
///  which Git named before the guard read the command,
///  and the caller's own `-C` chain does not move it a second time.
#[test]
fn a_command_a_shell_runs_is_measured_in_the_shells_directory() {
    let shell: PathBuf = PathBuf::from("/worktree/top");
    for prefix in [Vec::<&str>::new(), vec!["-C", "below"], vec!["--no-optional-locks", "-C", "/elsewhere"]] {
        assert_eq!(
            command_run_directory(args(prefix.as_slice()).as_slice(), Some(shell.as_path())),
            Ok(shell.clone()),
            "{prefix:?}"
        );
    }
}

/// A command of the caller's own runs where its `-C` chain ends,
///  which is this process's directory moved by every `-C` in the order written.
#[test]
fn a_command_of_the_callers_own_runs_where_its_chain_ends() {
    // An absolute `-C` reaches one directory whatever the process's own is, and the one after it moves from there.
    assert_eq!(
        command_run_directory(args(&["-C", "/work", "-C", "sub"]).as_slice(), None),
        Ok(PathBuf::from("/work/sub")),
        "an absolute chain"
    );
    // A relative one is read from the directory the move before it left, which the process's own directory starts.
    // A relative one is read from the directory the move before it left, which the process's own directory starts.
    let root: PathBuf = fixture("command-run-directory");
    let below: PathBuf = root.join("below");
    std::fs::create_dir_all(below.as_path()).expect("a directory below the fixture");
    let started_in: PathBuf = std::env::current_dir().expect("the directory this test started in");
    std::env::set_current_dir(root.as_path()).expect("the fixture is where the wrapper starts");
    // The directory the system reports for the process is the one every answer starts from, links resolved.
    let here: PathBuf = std::env::current_dir().expect("the directory this test now stands in");
    assert_eq!(
        command_run_directory(args(&["-C", "below"]).as_slice(), None),
        Ok(here.join("below")),
        "a relative chain"
    );
    assert_eq!(command_run_directory(&[], None), Ok(here.clone()), "no chain at all");
    // A `-C` whose value a later option's reader must not swallow is stepped over with its value.
    assert_eq!(
        command_run_directory(args(&["-C", "below", "--no-optional-locks"]).as_slice(), None),
        Ok(here.join("below")),
        "an option after the chain"
    );
    std::env::set_current_dir(started_in.as_path()).expect("leave the fixture before removing it");
    remove(root.as_path());
}

/// What:
///  A process directory that no longer exists names no command directory,
///  and the answer is an error rather than the shell's directory or the caller's assumption.
/// Why:
///  A rule that used a guessed directory would read a relative path of a command that never ran there;
///  the failure is what the policy reports as an unavailable fact.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // process.chdir(gone); commandRunDirectory([], undefined) throws
/// ```
#[test]
fn a_process_directory_that_is_gone_is_an_error() {
    let root: PathBuf = fixture("command-run-directory-gone");
    let gone: PathBuf = root.join("gone");
    std::fs::create_dir(gone.as_path()).expect("a directory to remove");
    std::env::set_current_dir(gone.as_path()).expect("the wrapper stands in the directory that goes");
    std::fs::remove_dir(gone.as_path()).expect("remove the directory the process stands in");
    let refused: Result<PathBuf, String> = command_run_directory(&[], None);
    let message: String = refused.expect_err("a directory that is gone named one");
    assert!(message.contains("could not name the directory this command runs in"), "{message}");
    std::env::set_current_dir(root.as_path()).expect("leave the directory that is gone");
    remove(root.as_path());
}

/// A shell's directory is taken as Git named it,
/// and never re-read from the process's own.
#[test]
fn a_shells_directory_is_not_read_again() {
    let gone: &Path = Path::new("/gone/shell");
    assert_eq!(command_run_directory(args(&["-C", "below"]).as_slice(), Some(gone)), Ok(gone.to_path_buf()));
}
