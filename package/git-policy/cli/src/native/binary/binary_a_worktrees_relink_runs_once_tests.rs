//! What:
//!  One worktree's relinking runs once,
//!  whichever of the two parties who can relink it gets there first.
//! Why:
//!   The add relinks a new worktree itself when no service read its wake,
//!  and the commit service relinks it from the record the add wrote,
//!  and the two used to run the two installers at once over one `node_modules`
//!  (the review of 2026-11-13,
//!  finding 1:
//!  two runs of one worktree 1.5 s apart on this branch;
//!  and on `main` when a service starts during the add's own run).
//!  What two concurrent installers do to one worktree's links nothing measured,
//!  so the record's own claim is what decides whose work it is
//!  (`package_repair_relink_claim.rs`):
//!  a rename out of `pending`, which only one consumer of the directory wins.
//!  This control runs the built wrapper with nothing listening when its wake is written,
//!  and a real service of the same build started while the add's own relink runs.
//!  The other way in, a service that reads the wake later than the client's close bound, is the reviewer's own script
//!  (`s2-slow-real-service-worktree-add.ts`): this file's claim is what stops it too.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await run(wrappedGit, ['worktree', 'add', '-b', 'one', '../one'], { XDG_RUNTIME_DIR: runtime }); expect(pnpmStarts()).toHaveLength(1)
//! ```

use super::commit::remove_settled;
use super::support::{Fixture, Observed, executable, fixture, git, hermetic, observe_without_waiting_for_the_after_commit_work, repository, stderr_of, wrapper_first_path};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::Duration;

/// How long the fixture's pnpm takes,
/// so two runs that overlap are two starts before one end.
const PNPM_SECONDS: u32 = 3;

/// How long the add is given before a service is started in the second control:
/// after the add's own wake has come to nothing and it holds its record.
const START_A_SERVICE_AFTER: Duration = Duration::from_secs(4);

/// The bound of one command this file starts itself,
/// so a control that waits for it is never held by it.
const COMMAND_BOUND_SECONDS: &str = "180";

/// A repository whose tracked files hold a lockfile and whose ignored directory holds a `node_modules` to copy.
fn repository_with_node_modules(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(repo.join(".gitignore"), b"node_modules/\n").expect("gitignore");
    std::fs::write(repo.join("pnpm-lock.yaml"), b"lockfileVersion: '9.0'\n").expect("lockfile");
    git(fixture, repo.as_path(), &["add", "--", ".gitignore", "pnpm-lock.yaml"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=lockfile"]);
    std::fs::create_dir_all(repo.join("node_modules/dep")).expect("node_modules");
    std::fs::write(repo.join("node_modules/dep/index.js"), b"module.exports = 1;\n").expect("dependency");
    return repo;
}

/// A fixture `pnpm` that records each start and each end with its process number,
/// so a run that overlaps another is visible in the record alone.
fn fake_pnpm(fixture: &Fixture) -> PathBuf {
    let record: PathBuf = fixture.root.join("pnpm-record");
    executable(
        fixture.root.join("bin/pnpm").as_path(),
        format!(
            "#!/bin/sh\necho \"start $(date +%s.%N) $$ in $(pwd -P)\" >> '{}'\nsleep {PNPM_SECONDS}\necho \"end $(date +%s.%N) $$ in $(pwd -P)\" >> '{}'\nexit 0\n",
            record.display(),
            record.display()
        )
        .as_bytes(),
    );
    return record;
}

/// The lines the fixture's pnpm recorded.
fn recorded(record: &Path) -> Vec<String> {
    return std::fs::read_to_string(record).map(|text: String| { return text.lines().map(str::to_owned).collect(); }).unwrap_or_default();
}

/// The starts in a pnpm record.
fn starts(lines: &[String]) -> Vec<String> {
    return lines.iter().filter(|line| { return line.starts_with("start"); }).cloned().collect();
}

/// How many relink lines the add printed,
/// for the tools it reported either way.
fn relinks_reported(printed: &str) -> usize {
    return printed.lines().filter(|line| { return line.contains("work at its new location"); }).count();
}

/// A command of the wrapper with the runtime directory this control chose,
/// so it reaches the service this control put there and no other,
/// and under a bound of its own.
fn wrapped_in(fixture: &Fixture, runtime: &Path, directory: &Path, arguments: &[&str]) -> Command {
    let mut command: Command = Command::new("/usr/bin/timeout");
    command.arg("--signal=KILL").arg(COMMAND_BOUND_SECONDS).arg(fixture.root.join("bin/git"));
    hermetic(&mut command, fixture, wrapper_first_path(fixture).as_os_str());
    command.env("XDG_RUNTIME_DIR", runtime).current_dir(directory).args(arguments);
    return command;
}

/// What:
///  A service started while the add relinks the worktree itself does not run that relink again.
/// Why:
///   This is `main`'s own way into two relinks of one worktree:
///  the add leaves its record `pending` while its own run goes on,
///  and a service that starts or looks in that window claims the record and runs the tools beside the add.
///  The claim closes it: the record is the add's, so the service finds nothing to claim,
///  and a record it did find would be its own to run, never both.
#[test]
fn a_service_started_during_the_adds_own_relink_relinks_the_worktree_once() {
    let fixture: Fixture = fixture("relink-once-service-later");
    let repo: PathBuf = repository_with_node_modules(&fixture);
    let record: PathBuf = fake_pnpm(&fixture);
    // Nothing listens when the add wakes, so the add relinks the worktree itself.
    let runtime: PathBuf = super::support::short_runtime_directory(&fixture);
    let adding: std::process::Child = {
        let mut command: Command = wrapped_in(&fixture, runtime.as_path(), repo.as_path(), &["worktree", "add", "-b", "one", "../one"]);
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        command.spawn().expect("the add runs")
    };
    std::thread::sleep(START_A_SERVICE_AFTER);
    let service = super::support::start_fixture_service_in(runtime.clone());
    service.serve(&fixture, repo.join(".git").as_path());
    // A commit of the repository wakes it, which is what a person's next command does.
    std::fs::write(repo.join("a.txt"), "one\n").expect("a change");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    let mut committing: Command = wrapped_in(&fixture, runtime.as_path(), repo.as_path(), &["commit", "--message=a change", "--", "a.txt"]);
    let committed: Observed = observe_without_waiting_for_the_after_commit_work(&mut committing, b"");
    assert_eq!(committed.code, Some(0), "{}", stderr_of(&committed));
    let finished: Output = adding.wait_with_output().expect("the add ends");
    let printed: String = String::from_utf8_lossy(finished.stderr.as_slice()).into_owned();
    assert_eq!(finished.status.code(), Some(0), "{printed}");
    assert_eq!(relinks_reported(printed.as_str()), 1, "the add reports one relink of the worktree:\n{printed}");
    // The service's own run, if it made one, ends after the add returned.
    std::thread::sleep(Duration::from_secs(PNPM_SECONDS as u64 + 12));
    let lines: Vec<String> = recorded(record.as_path());
    assert_eq!(starts(lines.as_slice()).len(), 1, "one worktree's relink runs once, whichever party holds the record:\n{lines:?}\n{printed}");
    remove_settled(&fixture);
}
