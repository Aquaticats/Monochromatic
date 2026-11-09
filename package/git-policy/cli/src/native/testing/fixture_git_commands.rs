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
use std::ffi::OsStr;
/// `Path` is a borrowed filesystem path of raw OS bytes.
use std::path::Path;
/// The command builder and the finished child every fixture Git run is.
use std::process::{Command, Output};

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
