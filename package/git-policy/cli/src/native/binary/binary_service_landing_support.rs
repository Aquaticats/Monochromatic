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
use super::service_commit::{Running, runtime_for, start_worktree};
use super::support::{Fixture, Observed, REAL_GIT, TIMED_OUT_SIGNAL, hermetic, observe};
use git_policy_cli::service_journal::DONE_EXTENSION;
use std::ffi::OsStr;
use std::os::unix::process::ExitStatusExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The date both runs give their commits.
const PINNED_DATE: &str = "2026-02-03T04:05:06+0000";

/// The longest one native Git command of this harness runs before the control stops waiting for it,
/// in seconds.
/// Why not a smaller number:
///  The oracle is one native `git commit` or one read of a fixture repository,
/// and the slowest of the 1,036 reads and 28 native commits the conclusion controls made took 463 ms on 2026-11-11
/// (`git-policy-cli-service-landing-state-control-times-out-alone`).
/// Sixty seconds is 130 times that,
/// so an ordinary read never reaches it even on a host other sessions load,
/// and what does reach it is a Git waiting on something a control cannot see.
const NATIVE_GIT_BOUND_SECONDS: &str = "60";

/// The state files a merge,
/// a cherry-pick or a revert leaves in the Git directory,
/// which a conclusion removes.
const STATE_FILES: [&str; 9] = ["MERGE_HEAD", "MERGE_MSG", "MERGE_MODE", "SQUASH_MSG", "AUTO_MERGE", "CHERRY_PICK_HEAD", "REVERT_HEAD", "MERGE_RR", "ORIG_HEAD"];

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
pub(super) fn case(name: &'static str, command: &'static [&'static str]) -> Case {
    return Case { name, command, index_commit: false, input: b"", full_output: false };
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
    command.env("GIT_AUTHOR_DATE", PINNED_DATE).env("GIT_COMMITTER_DATE", PINNED_DATE).env("GIT_EDITOR", "true");
}

/// A byte copy of `repo` beside it,
/// so both copies hold the same objects and the same configuration,
/// stopped with a failure that names the copy when it takes longer than `NATIVE_GIT_BOUND_SECONDS`.
pub(super) fn copy_beside(fixture: &Fixture, repo: &Path, name: &str) -> PathBuf {
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
    assert!(status.success(), "copying {} to {} for a control failed: it was ended at its {NATIVE_GIT_BOUND_SECONDS} s bound or cp refused ({status:?})", repo.display(), copy.display());
    return copy;
}

/// A command for native Git in `repo` under `bound`,
/// without the wrapper and without the person's own configuration,
/// ready for the environment the caller adds.
fn native_git_command(fixture: &Fixture, repo: &Path, arguments: &[&str], bound: &str) -> Command {
    let mut command: Command = Command::new("/usr/bin/timeout");
    command.arg("--signal=KILL").arg(bound).arg(REAL_GIT).args(arguments);
    hermetic(&mut command, fixture, OsStr::new("/usr/bin:/bin"));
    command.current_dir(repo);
    return command;
}

/// Runs one prepared native Git command to its end,
/// failing with the command,
/// the repository and the bound when Git was ended at that bound instead of answering.
fn native_git_run(command: &mut Command, repo: &Path, arguments: &[&str], bound: &str) -> std::process::Output {
    let output: std::process::Output = command.output().expect("native Git starts");
    assert_ne!(
        output.status.signal(),
        Some(TIMED_OUT_SIGNAL),
        "native Git {} in {} was ended at its {bound} s bound: the control was waiting for the oracle it compares the service against, so no field of this case can be compared",
        arguments.join(" "),
        repo.display()
    );
    return output;
}

/// The text of a Git query,
/// whether or not Git succeeded,
/// under `bound`,
/// as the bytes Git wrote them.
pub(super) fn text_of_within(fixture: &Fixture, repo: &Path, arguments: &[&str], bound: &str) -> String {
    let output: std::process::Output = native_git_run(&mut native_git_command(fixture, repo, arguments, bound), repo, arguments, bound);
    return String::from_utf8_lossy(output.stdout.as_slice()).into_owned() + &String::from_utf8_lossy(output.stderr.as_slice());
}

/// The text of a Git query,
/// whether or not Git succeeded,
/// under the bound every ordinary read of this harness gets.
pub(super) fn text_of(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> String {
    return text_of_within(fixture, repo, arguments, NATIVE_GIT_BOUND_SECONDS);
}

/// What the repository holds that a commit decides,
/// as labelled text.
pub(super) fn left_behind(fixture: &Fixture, repo: &Path) -> Vec<(String, String)> {
    let mut left: Vec<(String, String)> = Vec::new();
    let mut add = |label: &str, value: String| { left.push((label.to_string(), value)); };
    add("head", text_of(fixture, repo, &["rev-parse", "--verify", "--quiet", "HEAD"]));
    add("symbolic head", text_of(fixture, repo, &["symbolic-ref", "--quiet", "HEAD"]));
    add("references", text_of(fixture, repo, &["for-each-ref", "--format=%(refname) %(objectname)"]));
    let formats: [&str; 9] = ["%T", "%B", "%an <%ae>", "%ad", "%cn <%ce>", "%cd", "%P", "%H", "%G?"];
    for format in formats {
        add(format!("stored {format}").as_str(), text_of(fixture, repo, &["log", "-1", "--date=iso-strict", format!("--format={format}").as_str()]));
    }
    add("count", text_of(fixture, repo, &["rev-list", "--count", "--all"]));
    add("status", text_of(fixture, repo, &["status", "--short", "--untracked-files=all"]));
    add("staged", text_of(fixture, repo, &["diff", "--cached", "--raw", "--no-abbrev"]));
    add("index", text_of(fixture, repo, &["ls-files", "--stage"]));
    add("unstaged", text_of(fixture, repo, &["diff", "--raw", "--no-abbrev"]));
    let git_dir: PathBuf = repo.join(".git");
    for name in STATE_FILES {
        let state: String = std::fs::read(git_dir.join(name)).map_or_else(|_| { return String::from("(absent)"); }, |bytes| { return String::from_utf8_lossy(bytes.as_slice()).into_owned(); });
        add(format!("state {name}").as_str(), state);
    }
    let sequencer: String = std::fs::read_dir(git_dir.join("sequencer")).map_or_else(
        |_| { return String::from("(absent)"); },
        |entries| {
            let mut names: Vec<String> = entries
                .flatten()
                .map(|entry| { return format!("{}={}", entry.file_name().to_string_lossy(), std::fs::read_to_string(entry.path()).unwrap_or_default()); })
                .collect();
            names.sort();
            return names.join("\n");
        },
    );
    add("state sequencer", sequencer);
    return left;
}

/// Runs the case natively in `repo`,
/// under the bound of one command of this harness,
/// with the same input and the same settling the wrapper's own run gets.
pub(super) fn run_native(fixture: &Fixture, repo: &Path, entry: &Case) -> Outcome {
    let mut command: Command = native_git_command(fixture, repo, entry.command, NATIVE_GIT_BOUND_SECONDS);
    pinned(&mut command);
    // `observe` ends a command that was killed at its bound and names the command and the directory it ran in.
    return finish(fixture, repo, observe(&mut command, entry.input));
}

/// Runs the case through the wrapper and the service whose runtime directory `name` names.
pub(super) fn run_served(fixture: &Fixture, repo: &Path, name: &str, entry: &Case) -> Outcome {
    let mut arguments: Vec<&str> = entry.command.to_vec();
    if entry.index_commit {
        arguments.insert(1, "--no-enforce-only");
    }
    // A copy of the repository made after the service started is a worktree of its own, with a service of its own.
    let _service_of_this_worktree: Running = start_worktree(fixture.root.as_path(), runtime_for(name).as_path(), repo);
    let mut command: Command = wrapped_commit(fixture, repo, arguments.as_slice());
    command.env("XDG_RUNTIME_DIR", runtime_for(name));
    pinned(&mut command);
    return finish(fixture, repo, observe(&mut command, entry.input));
}

fn finish(fixture: &Fixture, repo: &Path, observed: Observed) -> Outcome {
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
            .filter(|entry| { return entry.path().extension().is_some_and(|extension| { return extension == DONE_EXTENSION; }); })
            .count();
    });
}

/// Runs a case in two byte copies of `base`,
/// which stays as it was,
/// and requires the same outcome,
/// and a landing by the service whenever native
///  Git committed.
/// Returns the two outcomes.
pub(super) fn compare(fixture: &Fixture, base: &Path, name: &str, entry: &Case) -> (Outcome, Outcome) {
    let native_repo: PathBuf = copy_beside(fixture, base, "native");
    let served_repo: PathBuf = copy_beside(fixture, base, "served");
    let before: usize = finished(fixture);
    let native: Outcome = run_native(fixture, native_repo.as_path(), entry);
    let served: Outcome = run_served(fixture, served_repo.as_path(), name, entry);
    assert_eq!(served.code, native.code, "{}: exit status\nservice output:\n{}", entry.name, served.output);
    assert_eq!(served.first_line, native.first_line, "{}: first line of the output", entry.name);
    if entry.full_output {
        assert_eq!(served.output, native.output, "{}: output", entry.name);
    }
    for ((label, expected), (_, found)) in native.left.iter().zip(&served.left) {
        assert_eq!(found, expected, "{}: {label}", entry.name);
    }
    if native.code == Some(0) {
        assert!(finished(fixture) > before, "{}: the service landed it", entry.name);
    } else {
        assert_eq!(finished(fixture), before, "{}: nothing landed", entry.name);
    }
    std::fs::remove_dir_all(&served_repo).expect("remove the served copy");
    std::fs::remove_dir_all(&native_repo).expect("remove the native copy");
    return (native, served);
}
