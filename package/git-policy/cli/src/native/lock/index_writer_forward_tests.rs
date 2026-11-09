//! Controls for the forward that stays alive around Git:
//!  a plain writer runs as a child and reports Git's status,
//!  a foreign `index.lock` is Git's to refuse and is not waited for,
//!  and a command that needs nothing around Git is handed back for an exec.

use super::*;
use std::path::Path;
use crate::forwarded_command::Route;
use crate::test_support::{REAL_GIT, executable, fixture, isolated_overlay, remove, repository};
use crate::worktree_identity::resolve_worktree_identity;
use crate::worktree_copy_forward::CopyFacts;

/// The forward for `git add <path>` in a repository.
fn add_forward(repo: &Path) -> CoordinatedForward {
    let prefix: Vec<OsString> = vec![OsString::from("-C"), repo.as_os_str().to_owned()];
    let identity: WorktreeIdentity = resolve_worktree_identity(
        Path::new(REAL_GIT),
        prefix.as_slice(),
        isolated_overlay().as_slice(),
    )
    .expect("identity");
    let arguments: Vec<OsString> = vec![
        OsString::from("-C"),
        repo.as_os_str().to_owned(),
        OsString::from("add"),
        OsString::from("a.txt"),
    ];
    let command: ResolvedCommand = ResolvedCommand {
        arguments: arguments.clone(),
        subcommand_index: 2,
        subcommand: Some(OsString::from("add")),
        route: Route::Direct,
    };
    return CoordinatedForward {
        real_git: PathBuf::from(REAL_GIT),
        arguments,
        overlay: isolated_overlay(),
        identity,
        command,
        environment: Vec::new(),
        copy: None,
    };
}

/// A writer runs real Git as a child and reports its status.
#[test]
fn a_coordinated_add_runs_git_and_reports_its_status() {
    let root: PathBuf = fixture("forward-add");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"content\n").expect("file to add");
    let forward: CoordinatedForward = add_forward(repo.as_path());
    assert_eq!(run_coordinated_forward(&forward), CoordinatedRun::Ran(0));
    let staged: String = crate::test_support::git_text(repo.as_path(), &["ls-files", "--", "a.txt"]);
    assert_eq!(staged, "a.txt");
    remove(root.as_path());
}

/// A foreign `index.lock` is Git's own refusal:
/// the writer runs Git at once,
/// Git fails as it fails without the wrapper,
/// and
///  nothing waits for the lock or removes it.
#[test]
fn a_foreign_index_lock_is_gits_refusal_and_is_not_waited_for() {
    let root: PathBuf = fixture("forward-foreign-lock");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"content\n").expect("file to add");
    std::fs::write(repo.join(".git/index.lock"), b"").expect("foreign lock");
    let forward: CoordinatedForward = add_forward(repo.as_path());
    let began: std::time::Instant = std::time::Instant::now();
    assert_eq!(run_coordinated_forward(&forward), CoordinatedRun::Ran(128), "Git's status for a lock it cannot create");
    assert!(began.elapsed() < std::time::Duration::from_secs(10), "no wait for a foreign holder");
    assert!(repo.join(".git/index.lock").exists(), "the foreign lock is left in place");
    let staged: String = crate::test_support::git_text(repo.as_path(), &["ls-files", "--", "a.txt"]);
    assert_eq!(staged, "", "Git could not take the lock, so nothing was staged");
    remove(root.as_path());
}

/// What:
///  A Git child that a signal ended keeps its signal through the coordinated forward,
/// which reports it as a value because it
///  keeps the wrapper alive around Git.
/// Why:
///  A run that flattens the death into the status a shell reports (128 + the signal) makes the caller see a normal
///  exit,
/// where native Git's caller sees `WIFSIGNALED` with that signal;
/// a script that tells "the reader went away"
///  from "Git failed" then takes the wrong branch
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 4 of the round-1
///  review).
/// The caller ends the wrapper by the same signal once the run has returned,
/// as the copy route does.
/// Every stream of the run is kept for the command's refusal,
/// which is the route an ordinary `git add` takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(runCoordinatedForward(addThroughAStandInThatKillsItself)).toEqual({ killed: SIGTERM });
/// ```
#[test]
fn a_git_child_that_a_signal_ended_keeps_its_signal_through_the_coordinated_forward() {
    assert_eq!(coordinated_death_of_a_git_killed_by_sigterm(true), CoordinatedRun::Killed(libc::SIGTERM));
}

/// What:
///  The same death,
/// through the path that has no place to keep the refused command's output.
/// Why:
///  Both of the run's ways of starting Git decide what a signal death is;
/// a fix that keeps the death in one of them
///  and not the other reports a status to one person and a death to the next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(runWithoutScratchOutput(addThroughAStandInThatKillsItself)).toEqual({ killed: SIGTERM });
/// ```
#[test]
fn a_git_child_that_a_signal_ended_keeps_its_signal_when_no_scratch_place_is_given() {
    assert_eq!(coordinated_death_of_a_git_killed_by_sigterm(false), CoordinatedRun::Killed(libc::SIGTERM));
}

/// What:
///  Runs a coordinated `git add` whose Git ends itself by `SIGTERM`,
/// with a home for the kept output when `home` is true,
/// and
///  answers how the run ended.
/// Why:
///   The two tests of the deaths above differ only in whether the run has a scratch place,
/// so one helper keeps them
///  the same in everything else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standIn = `#!/bin/sh\nfor token in "$@"; do if [ "$token" = "add" ]; then kill -TERM $$; fi; done; exec git "$@"`;
/// ```
fn coordinated_death_of_a_git_killed_by_sigterm(home: bool) -> CoordinatedRun {
    let root: PathBuf = fixture("forward-killed");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"content\n").expect("file to add");
    let mut forward: CoordinatedForward = add_forward(repo.as_path());
    let scripts: PathBuf = root.join("scripts");
    std::fs::create_dir_all(&scripts).expect("script directory");
    let stand_in: PathBuf = scripts.join("git");
    // The stand-in answers every query with the real Git and dies of SIGTERM where Git would write the index, which is
    //  what a person's `kill` or a supervisor mapping 128 + N does to Git alone.
    executable(
        stand_in.as_path(),
        format!("#!/bin/sh\nfor token in \"$@\"; do\nif [ \"$token\" = \"add\" ]; then kill -TERM $$; fi\ndone\nexec {REAL_GIT} \"$@\"\n").as_bytes(),
    );
    forward.real_git = stand_in;
    if home {
        // The scratch place of the kept output is read from the environment of the run's own variables.
        forward.environment = vec![(OsString::from("HOME"), root.join("home").into_os_string())];
    }
    let run: CoordinatedRun = run_coordinated_forward(&forward);
    remove(root.as_path());
    return run;
}

/// A non-writer that carries the linked-worktree copy still runs here,
/// as a settled child,
/// not as an exec.
#[test]
fn a_non_writer_with_a_copy_runs_settled() {
    let root: PathBuf = fixture("forward-status-copy");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut forward: CoordinatedForward = add_forward(repo.as_path());
    forward.command = ResolvedCommand {
        arguments: forward.arguments.clone(),
        subcommand_index: 2,
        subcommand: Some(OsString::from("status")),
        route: Route::Direct,
    };
    forward.arguments = vec![
        OsString::from("-C"),
        repo.as_os_str().to_owned(),
        OsString::from("status"),
        OsString::from("--short"),
    ];
    forward.copy = Some(CopyFacts {
        identity: forward.identity.clone(),
        creates: false,
        destination: None,
        add_plan: None,
        remove_plan: None,
    });
    assert_eq!(run_coordinated_forward(&forward), CoordinatedRun::Ran(0));
    remove(root.as_path());
}

/// A non-writer is not coordinated and is handed back for an ordinary exec.
#[test]
fn a_non_writer_is_not_coordinated() {
    let root: PathBuf = fixture("forward-status");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut forward: CoordinatedForward = add_forward(repo.as_path());
    forward.command = ResolvedCommand {
        arguments: forward.arguments.clone(),
        subcommand_index: 2,
        subcommand: Some(OsString::from("status")),
        route: Route::Direct,
    };
    assert_eq!(run_coordinated_forward(&forward), CoordinatedRun::NotCoordinated);
    remove(root.as_path());
}

/// A writer of another index file than the worktree's own has no hold of the service to wait for and is handed back for an exec.
#[test]
fn a_writer_of_another_index_file_is_not_coordinated() {
    let root: PathBuf = fixture("forward-other-index");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let forward: CoordinatedForward = add_forward(repo.as_path());
    let mut other: CoordinatedForward = forward.clone();
    other.overlay.push((OsString::from("GIT_INDEX_FILE"), repo.join("other-index").into_os_string()));
    assert_eq!(run_coordinated_forward(&other), CoordinatedRun::NotCoordinated);
    remove(root.as_path());
}
