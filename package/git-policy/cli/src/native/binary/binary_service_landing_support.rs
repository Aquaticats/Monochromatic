//! What:
//!  The twin harness of the landing controls:
//! one repository state made once,
//! copied byte for byte,
//! then the same command
//!  run by native Git in one copy and by the wrapper through the commit service in the other,
//! and every stored and left-behind
//!  field of the two compared.
//! Why:
//!  A byte copy has the same history,
//! and the dates of the new commit are pinned in both runs,
//! so a service that builds the
//!  same commit as native Git makes the very same object;
//! the fields are compared as well,
//! because a difference is easier to
//!  read as a field than as a hash.
//! What is left behind (the index entries,
//! the status,
//! the state files of a merge,
//! the
//!  references,
//! the head) is what the person's next command sees.
//! Every command the harness runs for its own oracle is bounded and names what it waited for,
//!  so a control that meets a Git or a copy that never answers fails saying which one it was waiting for
//! instead of holding the run until the runner ends it.

use super::commit::wrapped_commit;
use super::service_commit::{
    Running,
    runtime_for,
    start_worktree,
};
use super::support::{
    Fixture,
    NATIVE_GIT_BOUND_SECONDS,
    Observed,
    observe,
};
use git_policy_cli::service_journal::DONE_EXTENSION;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;

/// The date both runs give their commits.
const PINNED_DATE: &str = "2026-02-03T04:05:06+0000";

/// What:
///  What a repository holds after a command, as labelled text,
///  and the bounded Git reads it makes,
///  in a file of their own because this one is over the per-file line budget.
/// Why:
///  A control that named one of them through this module goes on working,
///  and the file that holds them describes the repository rather than the twin harness.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './binary_service_landing_support_what_a_repository_holds.rs';
/// ```
pub(super) use crate::service_landing_support_what_a_repository_holds::*;

/// One command line,
/// the state it starts from,
/// and what to compare.
pub(super) struct Case {
    /// Names the case in a failure.
    pub name: &'static str,
    /// The arguments after `git`.
    pub command: &'static [&'static str],
    /// Whether the line commits the index,
    /// which the wrapper's commit-only rule refuses unless it is told not to enforce.
    pub index_commit: bool,
    /// What both commands read on standard input.
    pub input: &'static [u8],
    /// Whether the whole of the output must agree,
    /// not only its first line (Git's own words for a refusal).
    pub full_output: bool,
}

/// A case with no input that compares the first line of the output.
pub(super) fn case(
    name: &'static str,
    command: &'static [&'static str],
) -> Case {
    return Case {
        name,
        command,
        index_commit: false,
        input: b"",
        full_output: false,
    };
}

/// What one run came to and what it left.
#[derive(Debug, Eq, PartialEq)]
pub(super) struct Outcome {
    pub code: Option<i32>,
    pub first_line: String,
    pub output: String,
    pub left: Vec<(String, String)>,
}

fn pinned(command: &mut Command) {
    command
        .env("GIT_AUTHOR_DATE", PINNED_DATE)
        .env("GIT_COMMITTER_DATE", PINNED_DATE)
        .env("GIT_EDITOR", "true");
}

/// A byte copy of `repo` beside it,
/// so both copies hold the same objects and the same configuration,
/// stopped with a failure that names the copy when it takes longer than `NATIVE_GIT_BOUND_SECONDS`.
pub(super) fn copy_beside(
    fixture: &Fixture,
    repo: &Path,
    name: &str,
) -> PathBuf {
    let copy: PathBuf = fixture.root.join(name);
    let status = Command::new("/usr/bin/timeout")
        .arg("--signal=KILL")
        .arg(NATIVE_GIT_BOUND_SECONDS)
        .arg("cp")
        .arg("--archive")
        .arg(repo)
        .arg(&copy)
        .status()
        .expect("cp starts");
    assert!(
        status.success(),
        "copying {} to {} for a control failed: it was ended at its {NATIVE_GIT_BOUND_SECONDS} s bound or cp refused ({status:?})",
        repo.display(),
        copy.display()
    );
    return copy;
}

/// Runs the case natively in `repo`,
/// under the bound of one command of this harness,
/// with the same input and the same settling the wrapper's own run gets.
pub(super) fn run_native(
    fixture: &Fixture,
    repo: &Path,
    entry: &Case,
) -> Outcome {
    let mut command: Command =
        native_git_command(fixture, repo, entry.command, NATIVE_GIT_BOUND_SECONDS);
    pinned(&mut command);
    // `observe` ends a command that was killed at its bound and names the command and the directory it ran in.
    return finish(fixture, repo, observe(&mut command, entry.input));
}

/// Runs the case through the wrapper and the service whose runtime directory `name` names.
pub(super) fn run_served(
    fixture: &Fixture,
    repo: &Path,
    name: &str,
    entry: &Case,
) -> Outcome {
    let mut arguments: Vec<&str> = entry.command.to_vec();
    if entry.index_commit {
        arguments.insert(1, "--no-enforce-only");
    }
    // A copy of the repository made after the service started is a worktree of its own, with a service of its own.
    let _service_of_this_worktree: Running =
        start_worktree(fixture.root.as_path(), runtime_for(name).as_path(), repo);
    let mut command: Command = wrapped_commit(fixture, repo, arguments.as_slice());
    command.env("XDG_RUNTIME_DIR", runtime_for(name));
    pinned(&mut command);
    return finish(fixture, repo, observe(&mut command, entry.input));
}

fn finish(
    fixture: &Fixture,
    repo: &Path,
    observed: Observed,
) -> Outcome {
    let stdout: String = String::from_utf8_lossy(observed.stdout.as_slice()).into_owned();
    return Outcome {
        code: observed.code,
        first_line: stdout.lines().next().unwrap_or("").to_string(),
        output: stdout,
        left: left_behind(fixture, repo),
    };
}

/// The number of batches the service has finished in the fixture.
pub(super) fn finished(fixture: &Fixture) -> usize {
    return std::fs::read_dir(fixture.root.join("journal")).map_or(0, |entries| {
        return entries
            .flatten()
            .filter(|entry| {
                return entry.path().extension().is_some_and(|extension| {
                    return extension == DONE_EXTENSION;
                });
            })
            .count();
    });
}

/// Runs a case in two byte copies of `base`,
/// which stays as it was,
/// and requires the same outcome,
/// and a landing by the service whenever native
///  Git committed.
/// Returns the two outcomes.
pub(super) fn compare(
    fixture: &Fixture,
    base: &Path,
    name: &str,
    entry: &Case,
) -> (Outcome, Outcome) {
    let native_repo: PathBuf = copy_beside(fixture, base, "native");
    let served_repo: PathBuf = copy_beside(fixture, base, "served");
    let before: usize = finished(fixture);
    let native: Outcome = run_native(fixture, native_repo.as_path(), entry);
    let served: Outcome = run_served(fixture, served_repo.as_path(), name, entry);
    assert_eq!(
        served.code, native.code,
        "{}: exit status\nservice output:\n{}",
        entry.name, served.output
    );
    assert_eq!(
        served.first_line, native.first_line,
        "{}: first line of the output",
        entry.name
    );
    if entry.full_output {
        assert_eq!(served.output, native.output, "{}: output", entry.name);
    }
    for ((label, expected), (_, found)) in native.left.iter().zip(&served.left) {
        assert_eq!(found, expected, "{}: {label}", entry.name);
    }
    if native.code == Some(0) {
        assert!(
            finished(fixture) > before,
            "{}: the service landed it",
            entry.name
        );
    } else {
        assert_eq!(finished(fixture), before, "{}: nothing landed", entry.name);
    }
    std::fs::remove_dir_all(&served_repo).expect("remove the served copy");
    std::fs::remove_dir_all(&native_repo).expect("remove the native copy");
    return (native, served);
}
