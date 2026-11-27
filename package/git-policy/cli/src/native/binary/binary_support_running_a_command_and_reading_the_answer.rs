//! What:
//!  Running a command in a fixture and reading what its caller saw:
//!  real Git under the bound of one fixture command,
//!  a repository with one empty commit,
//!  the wrapper and a direct run,
//!  and the exit status and two streams a finished command left.
//! Why:
//!  Every control reads a run the same way,
//!  so the readers sit together and name one shape (`Observed`);
//!  a control that only needs the bytes says so with `stderr_of` rather than reaching into the struct.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const seen = runWrapped(fixture, repo, ['status']); // { code, stdout, stderr }
//! ```

use crate::support::{
    Fixture,
    wrapped,
};
use crate::support_native_git_command::{
    native_git,
    native_git_within,
};
use crate::support_observation::{
    Observed,
    observe,
    without_wait_notices,
};
use crate::support_program_names_and_time_bound::NATIVE_GIT_BOUND_SECONDS;
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Output;

/// Run real Git directly inside a fixture under the bound of one fixture command,
/// and require success.
/// # Panics
///  When Git failed,
///  or was ended at the bound:
///  the failure names the command,
///  the directory and the bound.
pub fn git<S: AsRef<OsStr>>(
    fixture: &Fixture,
    directory: &Path,
    arguments: &[S],
) -> Output {
    return git_within(fixture, directory, arguments, NATIVE_GIT_BOUND_SECONDS);
}

/// Run real Git directly inside a fixture under `bound` seconds,
/// and require success.
pub fn git_within<S: AsRef<OsStr>>(
    fixture: &Fixture,
    directory: &Path,
    arguments: &[S],
    bound: &str,
) -> Output {
    let output: Output = native_git_within(fixture, directory, arguments, bound);
    let words: Vec<String> = arguments
        .iter()
        .map(|argument| {
            return argument.as_ref().to_string_lossy().into_owned();
        })
        .collect();
    assert!(
        output.status.success(),
        "native Git {} in {} failed (bound {bound} s): {}",
        words.join(" "),
        directory.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    return output;
}

/// Create a repository with one empty commit on `main`,
///  using real Git directly.
pub fn repository(
    fixture: &Fixture,
    name: &OsStr,
) -> PathBuf {
    let root: PathBuf = fixture.root.join(name);
    std::fs::create_dir(&root).expect("repository directory");
    git(
        fixture,
        root.as_path(),
        &["init", "--quiet", "--initial-branch=main"],
    );
    git(
        fixture,
        root.as_path(),
        &["commit", "--quiet", "--allow-empty", "--message=initial"],
    );
    return root;
}

/// Run a wrapped command in a directory,
///  with empty standard input,
///  and return what the caller saw.
pub fn run_wrapped(
    fixture: &Fixture,
    directory: &Path,
    arguments: &[&str],
) -> Observed {
    return observe(wrapped(fixture).current_dir(directory).args(arguments), b"");
}

/// Run real Git directly in a directory under the bound of one fixture command,
///  with empty standard input,
///  and return what the caller saw.
pub fn run_direct(
    fixture: &Fixture,
    directory: &Path,
    arguments: &[&str],
) -> Observed {
    let output: Output = native_git(fixture, directory, arguments);
    return Observed {
        code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    };
}

/// Real Git's porcelain status of a repository,
///  untracked files included,
///  as text.
pub fn porcelain(
    fixture: &Fixture,
    repo: &Path,
) -> String {
    let output: Output = git(
        fixture,
        repo,
        &["status", "--porcelain=v1", "--untracked-files=all"],
    );
    return String::from_utf8_lossy(&output.stdout).into_owned();
}

/// What a caller sees when a command stops with exit status 2 and this text on standard error.
pub fn stopped_with(stderr: &str) -> Observed {
    return Observed {
        code: Some(2),
        stdout: Vec::<u8>::new(),
        stderr: stderr.as_bytes().to_vec(),
    };
}

/// What a caller sees when a command succeeds without printing anything.
pub fn silent_success() -> Observed {
    return Observed {
        code: Some(0),
        stdout: Vec::<u8>::new(),
        stderr: Vec::<u8>::new(),
    };
}

/// Standard error of an observed command as text.
pub fn stderr_of(observed: &Observed) -> String {
    return String::from_utf8_lossy(&observed.stderr).into_owned();
}

/// What:
///  Standard error of an observed command as text,
///  without the notices that say a step of the wrapper waited
///  (`without_wait_notices`).
/// Why:
///  A command whose landing ran past a notice grace prints one however healthy it is,
/// and a loaded host
///  decides that,
/// so a control that means the words the command said compares these.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = withoutWaitNotices(observed.stderr).toString();
/// ```
pub fn stderr_without_wait_notices(observed: &Observed) -> String {
    return String::from_utf8_lossy(without_wait_notices(&observed.stderr).as_slice()).into_owned();
}
