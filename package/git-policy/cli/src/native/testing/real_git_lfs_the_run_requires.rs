//! What:
//!  The real `git-lfs` a control proves the upload against,
//! and what a control does in a run that has none.
//! Why:
//!  Controls prove the LFS upload against the real program and against an LFS server written inline,
//! and one compares the pointer decoder with the program's own,
//! and each of them printed a line and checked nothing where `PATH` found no `git-lfs`,
//! so the container gate counted them as passes without an upload ever running
//! (`doc/troubleshooting/git-policy-cli-gate-skips-the-real-git-lfs-controls.md`).
//!  A run that must have the program sets `CLI_GIT_REQUIRE_REAL_GIT_LFS`,
//! which the gate image does,
//! and a control whose program is then absent fails with that variable named
//!  rather than passing;
//! the gate image installs `git-lfs`,
//! so such a failure means an image that lost it.
//! A run that sets nothing keeps the old behaviour:
//! the control prints the line that says it did not run,
//! and the gate log shows that line because the suite runs with `--success-output=final`.
//!  One copy of the decision serves the four binary-level controls and the one library control that ask it,
//! and the binary target includes this file by path
//!  (`src/native/binary/binary_tests.rs`),
//! the way it includes
//! `testing/test_alive.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const program = realGitLfsOrTheLineThatSaysTheControlDidNotRun('the real upload against the server');
//! ```

/// Owned operating-system text,
/// because a `PATH` is bytes and not text.
use std::ffi::OsStr;
/// An owned filesystem path of raw OS bytes.
use std::path::PathBuf;

/// What:
///  The name of the program these controls need.
/// Why:
///   This file is compiled into two targets and imports nothing of the crate,
/// as `binary_support_program_names_and_time_bound.rs` does,
/// so the name is written here;
/// `testing/real_git_lfs_the_run_requires_tests.rs` fails the run where it and `lfs_pre_push::LFS_PROGRAM`
///  disagree,
/// so the copy cannot drift from the wrapper's own name.
pub const PROGRAM: &str = "git-lfs";

/// The environment variable that states this run must have the real `git-lfs`.
/// The container gate image sets it (`bin/native-test-containerfile.ts`);
/// nothing else does,
/// so a developer's run that has no `git-lfs` keeps the line and the pass.
pub const REQUIRED_VARIABLE: &str = "CLI_GIT_REQUIRE_REAL_GIT_LFS";

/// What:
///  Whether this run states that the real `git-lfs` must be there.
/// Why:
///   The variable's presence is the gate image's statement,
/// and an empty value states nothing,
/// as the btrfs and end-to-end acknowledgements read theirs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const required = (process.env.CLI_GIT_REQUIRE_REAL_GIT_LFS ?? '') !== '';
/// ```
pub fn the_run_requires_it() -> bool {
    return std::env::var_os(REQUIRED_VARIABLE).is_some_and(|value| { return !value.is_empty(); });
}

/// What:
///  The real `git-lfs` the given search list names,
/// or nothing.
/// Why:
///   The list is the one the control's own command searches,
/// and a directory on it that holds no program of that name is not a program:
/// the lookup is the file's presence,
/// never a guess from the list's shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const found = path?.split(':').map((folder) => join(folder, 'git-lfs')).find((candidate) => statSync(candidate, { throwIfNoEntry: false })?.isFile());
/// ```
pub fn found_in(path: Option<&OsStr>) -> Option<PathBuf> {
    let list: &OsStr = path?;
    return std::env::split_paths(list)
        .map(|directory| { return directory.join(PROGRAM); })
        .find(|candidate| { return candidate.is_file(); });
}

/// What:
///  The real `git-lfs` this process's `PATH` names,
/// or nothing.
/// Why:
///   Every control that uploads an object needs the real program,
/// and its own `PATH` is the list its upload searches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const found = foundIn(process.env.PATH);
/// ```
pub fn on_the_path() -> Option<PathBuf> {
    return found_in(std::env::var_os("PATH").as_deref());
}

/// What:
///  The program when it is there,
///  `None` after one line naming the control when this run states it may be absent,
///  and a failed control when the run states it must be there.
/// Why:
///   A control that needs the program must not pass without it.
/// A run that states the absence is known gets the line,
/// which `--success-output=final` brings into the gate's log;
///  a run that states the program must be there gets a failure that names the variable and the image
/// that installs it,
/// because that run is a gate image that lost the program or a `PATH` that names none of its folders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const program = required('the real upload'); if (program === undefined) return;
/// ```
pub fn required(what: &str) -> Option<PathBuf> {
    return decided(what, on_the_path(), the_run_requires_it());
}

/// What:
///  The same decision,
/// with the program found and the run's statement given rather than read.
/// Why:
///   A control cannot change this process's environment beside the other controls of its binary,
/// so the arm that fails is reached through this function,
/// with the absence and the statement a gate image has.
/// The environment itself is proved where a child process can set it
/// (`testing/real_git_lfs_the_run_requires_tests.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decided = (what, found, required) => found ?? (required ? fail(what) : noteSkipped(what));
/// ```
pub fn decided(what: &str, found: Option<PathBuf>, required: bool) -> Option<PathBuf> {
    if let Some(program) = found {
        return Some(program);
    }
    assert!(
        !required,
        "the control {what} found no `{PROGRAM}`, and this run states it must have the program: `{REQUIRED_VARIABLE}` is set, so \
         the control fails rather than passes. The container gate image installs `{PROGRAM}` \
         (`native-test-base.Containerfile`), so this is an image that lost it, or a `PATH` that names none of its folders."
    );
    eprintln!(
        "REAL GIT LFS CONTROL NOT RUN: {what} (`{PROGRAM}` is not on this run's `PATH`, and nothing states that this run must have it; \
         the container gate image installs it and sets `{REQUIRED_VARIABLE}`, so its absence there fails)"
    );
    return None;
}
