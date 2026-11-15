//! What:
//!  The two ways the controls start real Git in a fixture:
//! one that answers its output,
//! and one that requires it to have succeeded.
//! Why:
//!  Every control that needs a repository,
//!  a commit or an answer from Git starts it the same way,
//! with no system or global
//!  configuration and a fixed identity,
//!  so nothing of the account's own configuration reaches a fixture.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! spawnSync(REAL_GIT, args, { cwd, env: { PATH: '/usr/bin:/bin', GIT_CONFIG_NOSYSTEM: '1', ... } })
//! ```

/// The one path of the host's Git that the binary tests share.
use super::REAL_GIT;
/// `OsStr`/`OsString` are borrowed/owned operating-system text (siblings `&str`/`String` must be UTF-8).
use std::ffi::{OsStr, OsString};
/// `Write` fills the standard input of a query that reads one.
use std::io::Write;
/// `Path` is a borrowed filesystem path of raw OS bytes.
use std::path::Path;
/// The command builder, the child's pipes and the finished child every fixture Git run is.
use std::process::{Command, Output, Stdio};

/// Run real Git in a fixture with no system or global configuration and a fixed identity.
pub(crate) fn git_output<S: AsRef<OsStr>>(directory: &Path, arguments: &[S]) -> Output {
    return Command::new(REAL_GIT)
        .current_dir(directory)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .args(arguments)
        .output()
        .expect("native Git fixture command");
}

/// Run real Git in a fixture and require success.
pub(crate) fn git<S: AsRef<OsStr>>(directory: &Path, arguments: &[S]) -> Output {
    let output: Output = git_output(directory, arguments);
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        arguments.iter().map(AsRef::as_ref).collect::<Vec<&OsStr>>(),
        String::from_utf8_lossy(&output.stderr)
    );
    return output;
}

/// How long one real-Git query of a control may take before the bound ends it,
/// in seconds as `/usr/bin/timeout` reads them.
/// Why sixty:
///  The same bound the binary-level controls give one fixture Git run
/// (`NATIVE_GIT_BOUND_SECONDS`), and for the same measurement:
/// the longest disk stall one `fsync` of a trivial command showed on this host is 4.74 s
/// (`doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`),
/// so sixty is more than ten times it.
pub(crate) const FIXTURE_GIT_BOUND_SECONDS: &str = "60";

/// What:
///  Run one real-Git query under `bound_seconds` and answer its output,
///  or nothing when the bound ended it.
/// Why:
///  A control that ran Git directly would hold the suite for ever against a Git that never answers,
/// and report nothing at all;
/// `/usr/bin/timeout` sends `SIGKILL` at the bound and waits for the child,
/// so nothing of the query outlives this call.
/// The caller answers `None` with a verdict naming its own query,
/// because only it knows which query it made and what the query is for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = runUnderBound('git', ['--version'], 60); // undefined when the bound ended it
/// ```
pub(crate) fn real_git_within(
    program: &Path,
    directory: Option<&Path>,
    environment: &[(OsString, OsString)],
    arguments: &[OsString],
    input: Option<&[u8]>,
    bound_seconds: &str,
) -> Option<Output> {
    let mut command: Command = Command::new("/usr/bin/timeout");
    command.arg("--signal=KILL").arg(bound_seconds).arg(program).args(arguments);
    if let Some(directory) = directory {
        command.current_dir(directory);
    }
    command.env_clear();
    for (name, value) in environment {
        command.env(name, value);
    }
    command.stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() });
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child: std::process::Child = command.spawn().expect("the query starts");
    if let Some(input) = input {
        let mut standard_input: std::process::ChildStdin = child.stdin.take().expect("piped standard input");
        standard_input.write_all(input).expect("the query's input");
    }
    let output: Output = child.wait_with_output().expect("the query's output");
    // `timeout` ends the child at the bound and reports a status of its own: 124 with the default signal and 137
    // when the signal was KILL (measured on this host), and the signal itself on a host where it re-raises it.
    return match output.status.code() {
        None | Some(124) | Some(137) => None,
        Some(_) => Some(output),
    };
}
