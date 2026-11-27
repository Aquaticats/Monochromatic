//! What:
//!  Real Git run in a fixture directory with the fixture's identity,
//!  and the two arms the controls compare:
//!  the update's own resets,
//!  and `git reset --no-refresh` over a byte list of names.
//! Why:
//!  A control that compares the in-process reset with Git's own command needs
//!  both to run under the same identity and the same cleared environment,
//!  and it needs the same index file handed to the second,
//!  so the identity,
//!  the variables and the standard input live in one place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const output = gitWith(dir, env, args, input);
//! gitReset(repo, commit, names);
//! const ok = gitResetStatus(repo, commit, names);
//! ```

/// The Git binary the fixtures run,
/// and the child owner every start goes through.
use crate::test_support::{
    REAL_GIT,
    spawn_owned,
};
/// The variables one run adds,
/// and the directory it runs in.
use std::ffi::OsString;
use std::path::Path;
/// The command,
/// its output and the standard input a run may be fed.
use std::process::{
    Command,
    Output,
    Stdio,
};
/// Run real Git in `dir` with the fixture identity,
/// extra variables and optional standard input.
pub(crate) fn git_with(
    dir: &Path,
    env: &[(&str, OsString)],
    args: &[&str],
    input: Option<&[u8]>,
) -> Output {
    let mut command: Command = Command::new(REAL_GIT);
    command
        .current_dir(dir)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (name, value) in env {
        command.env(name, value);
    }
    let mut child = spawn_owned(&mut command).expect("start Git");
    if let Some(bytes) = input {
        use std::io::Write as _;
        child
            .stdin
            .take()
            .expect("piped")
            .write_all(bytes)
            .expect("write input");
    }
    return child.wait_with_output().expect("Git output");
}

/// Real Git in a copy of `dir`'s environment with `index` as its index file,
/// requiring success;
/// its standard output.
pub(super) fn git_in_index(
    dir: &Path,
    index: &Path,
    args: &[&str],
    input: Option<&[u8]>,
) -> String {
    let output: Output = git_with(
        dir,
        &[("GIT_INDEX_FILE", index.as_os_str().to_os_string())],
        args,
        input,
    );
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    return String::from_utf8(output.stdout)
        .expect("UTF-8")
        .trim_end()
        .to_string();
}

/// Run `git reset --no-refresh <commit> -- <names>` in `repo` as the follow-up's fallback does.
pub(crate) fn git_reset(
    repo: &Path,
    commit: &str,
    names: &[Vec<u8>],
) {
    let mut input: Vec<u8> = Vec::new();
    for name in names {
        input.extend_from_slice(b":(top,literal)");
        input.extend_from_slice(name.as_slice());
        input.push(0);
    }
    let output: Output = git_with(
        repo,
        &[],
        &[
            "reset",
            "--no-refresh",
            commit,
            "--pathspec-from-file=-",
            "--pathspec-file-nul",
        ],
        Some(input.as_slice()),
    );
    assert!(
        output.status.success(),
        "git reset: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Whether `git reset --no-refresh <commit> -- <names>` succeeds in `repo`.
pub(crate) fn git_reset_status(
    repo: &Path,
    commit: &str,
    names: &[Vec<u8>],
) -> bool {
    let mut input: Vec<u8> = Vec::new();
    for name in names {
        input.extend_from_slice(b":(top,literal)");
        input.extend_from_slice(name.as_slice());
        input.push(0);
    }
    return git_with(
        repo,
        &[],
        &[
            "reset",
            "--no-refresh",
            commit,
            "--pathspec-from-file=-",
            "--pathspec-file-nul",
        ],
        Some(input.as_slice()),
    )
    .status
    .success();
}
