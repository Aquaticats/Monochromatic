//! Controls for the forward that stays alive around Git:
//!  a plain writer runs as a child and reports Git's status,
//!  a foreign `index.lock` is Git's to refuse and is not waited for,
//!  and a command that needs nothing around Git is handed back for an exec.

use super::*;
use std::path::Path;
use crate::forwarded_command::Route;
use crate::test_support::{REAL_GIT, executable, fixture, git, git_text, isolated_overlay, remove, repository};
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
        // These controls name a built-in, which no shell alias runs.
        shell_command: None,
        missing_word: false,
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


/// What:
///  A Git child that a signal ended keeps its signal on the plain child route,
/// where the command is not one the wrapper may
///  run again and no copy is settled.
/// Why:
///   `git add --patch` is not run again after the service took the index,
/// so its Git runs once through the plain route,
///  and that route reported the death as the status a shell prints for it (128 + the signal)
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 3 of the round-2
///  review).
/// Neither of the two routes the signal fix covered was reached for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(runCoordinatedForward(addPatchThroughAStandInThatKillsItself)).toEqual({ killed: SIGTERM });
/// ```
#[test]
fn a_git_child_of_a_command_that_is_not_run_again_keeps_its_signal() {
    assert_eq!(plain_route_death_of_a_git_killed_by_sigterm(), CoordinatedRun::Killed(libc::SIGTERM));
}

/// The death of a Git that ends itself by `SIGTERM`,
/// on the route a command the wrapper may not run again takes.
fn plain_route_death_of_a_git_killed_by_sigterm() -> CoordinatedRun {
    let root: PathBuf = fixture("forward-killed-once");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"content\n").expect("file to add");
    let mut forward: CoordinatedForward = add_forward(repo.as_path());
    // `--patch` reads the person's answers, so the command is never run again and its Git runs once.
    forward.arguments = vec![OsString::from("-C"), repo.as_os_str().to_owned(), OsString::from("add"), OsString::from("--patch")];
    forward.command = ResolvedCommand { arguments: forward.arguments.clone(), subcommand_index: 2, subcommand: Some(OsString::from("add")), route: Route::Direct, shell_command: None, missing_word: false };
    assert!(!may_run_again(forward.command.word(), forward.command.region()), "this control is the plain route only if the command may not run again");
    let scripts: PathBuf = root.join("scripts");
    std::fs::create_dir_all(&scripts).expect("script directory");
    let stand_in: PathBuf = scripts.join("git");
    executable(
        stand_in.as_path(),
        format!("#!/bin/sh\nfor token in \"$@\"; do\nif [ \"$token\" = \"add\" ]; then kill -TERM $$; fi\ndone\nexec {REAL_GIT} \"$@\"\n").as_bytes(),
    );
    forward.real_git = stand_in;
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
        // These controls name a built-in, which no shell alias runs.
        shell_command: None,
        missing_word: false,
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
        // These controls name a built-in, which no shell alias runs.
        shell_command: None,
        missing_word: false,
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

/// A Git that appends its arguments to `log` and then runs the real Git,
/// so a control can tell whether the forwarded command was started at all.
fn recording_git(root: &Path, log: &Path) -> PathBuf {
    let directory: PathBuf = root.join("recording-git");
    std::fs::create_dir(&directory).expect("recording directory");
    let script: String = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\nexec {REAL_GIT} \"$@\"\n",
        log = log.display()
    );
    let program: PathBuf = directory.join("git");
    std::fs::write(program.as_path(), script.as_bytes()).expect("recording Git");
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(program.as_path(), std::fs::Permissions::from_mode(0o755)).expect("an executable Git");
    return program;
}

/// Whether `log` holds a start of the merge `forward` would hand to native Git,
/// as its last words after any options the runner adds of its own.
fn forwarded_a_merge(log: &Path, forward: &CoordinatedForward) -> bool {
    let written: String = std::fs::read_to_string(log).unwrap_or_default();
    let invocation: String = forward
        .arguments
        .iter()
        .map(|argument| { return argument.to_string_lossy().into_owned(); })
        .collect::<Vec<String>>()
        .join(" ");
    let tail: String = format!(" {invocation}");
    return written.lines().any(|line| { return line == invocation || line.ends_with(tail.as_str()); });
}

/// A repository where `feature` and `main` have diverged and another session left `u.txt` staged,
/// which native Git refuses to merge over.
fn diverged_with_staged_work(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    for file in ["u.txt", "f.txt", "h.txt"] {
        std::fs::write(repo.join(file), format!("{file} base\nwith enough lines\nfor a rename to be found\n")).expect("file");
        git(repo.as_path(), &["add", "--", file]);
    }
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("f.txt"), b"f.txt feature\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=feature work"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("h.txt"), b"h.txt main\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=main work"]);
    std::fs::write(repo.join("u.txt"), b"u.txt staged by another session\n").expect("file");
    git(repo.as_path(), &["add", "--", "u.txt"]);
    return (root, repo);
}

/// The forward for `git merge` with `after` written behind the subcommand,
/// in the repository at `repo`,
/// started with `real_git`.
fn merge_forward(repo: &Path, after: &[&str], real_git: &Path) -> CoordinatedForward {
    let prefix: Vec<OsString> = vec![OsString::from("-C"), repo.as_os_str().to_owned()];
    let identity: WorktreeIdentity = resolve_worktree_identity(
        Path::new(REAL_GIT),
        prefix.as_slice(),
        isolated_overlay().as_slice(),
    )
    .expect("identity");
    let mut arguments: Vec<OsString> = prefix;
    arguments.push(OsString::from("merge"));
    arguments.extend(after.iter().map(OsString::from));
    let command: ResolvedCommand = ResolvedCommand {
        arguments: arguments.clone(),
        subcommand_index: 2,
        subcommand: Some(OsString::from("merge")),
        route: Route::Direct,
        shell_command: None,
        missing_word: false,
    };
    return CoordinatedForward {
        real_git: real_git.to_path_buf(),
        arguments,
        overlay: isolated_overlay(),
        identity,
        command,
        environment: Vec::new(),
        copy: None,
    };
}

/// What:
///  A merge over another session's staged work is completed by the wrapper itself:
/// the merge is made,
/// native Git is
///  never started for it,
/// and the staged work is carried,
/// so no status of a refusal is reported at all.
/// Why:
///   Native Git refuses this merge (`Your local changes ... would be overwritten`),
/// and the owner's answer is "Carry
///  edits forward (Recommended)"
/// (`SPEC.md`,
/// "Merges and commits never fail").
#[test]
fn a_merge_over_staged_work_is_completed_and_carries_it() {
    let (root, repo): (PathBuf, PathBuf) = diverged_with_staged_work("forward-merge-carried");
    std::fs::write(repo.join("f.txt"), b"f.txt staged by another session\n").expect("file");
    git(repo.as_path(), &["add", "--", "f.txt"]);
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let log: PathBuf = root.join("git-starts.log");
    let forward: CoordinatedForward = merge_forward(repo.as_path(), &["--no-edit", "feature"], recording_git(&root, log.as_path()).as_path());
    let ended: CoordinatedRun = run_coordinated_forward(&forward);
    assert_eq!(ended, CoordinatedRun::Ran(0), "the merge is completed");
    assert!(!forwarded_a_merge(log.as_path(), &forward), "native Git is not started for it");
    assert_ne!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), head, "the branch moved to the merge");
    assert_eq!(git_text(repo.as_path(), &["rev-list", "--count", "--merges", "HEAD"]), "1");
    for staged in ["f.txt", "u.txt"] {
        assert_eq!(
            git_text(repo.as_path(), &["diff", "--cached", "--name-only", "--", staged]),
            staged,
            "{staged} stays staged"
        );
    }
    // The person's bytes are in the file beside the merge's, between markers this time: both changed the whole line.
    let carried: String = std::fs::read_to_string(repo.join("f.txt")).expect("the file");
    assert!(carried.contains("f.txt staged by another session") && carried.contains("<<<<<<<"), "{carried}");
    remove(root.as_path());
}

/// A merge the wrapper defers to native Git runs here,
/// and the status is Git's:
/// the setting that defers it is read,
/// the command falls through,
/// and native Git refuses the merge,
/// because none of the commits it would merge carries a signature.
#[test]
fn a_merge_the_wrapper_defers_runs_as_native_git_and_reports_its_status() {
    let (root, repo): (PathBuf, PathBuf) = diverged_with_staged_work("forward-merge-deferred");
    git(repo.as_path(), &["config", "merge.verifysignatures", "yes"]);
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let log: PathBuf = root.join("git-starts.log");
    let forward: CoordinatedForward = merge_forward(repo.as_path(), &["--no-edit", "feature"], recording_git(&root, log.as_path()).as_path());
    let ended: CoordinatedRun = run_coordinated_forward(&forward);
    let CoordinatedRun::Ran(code) = ended else {
        panic!("the deferred merge runs as native Git: {ended:?}");
    };
    assert!(forwarded_a_merge(log.as_path(), &forward), "the deferred merge is handed to native Git");
    assert_ne!(code, 0, "native Git's refusal of a merge of unsigned commits");
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), head, "nothing was merged");
    assert_eq!(git_text(repo.as_path(), &["diff", "--cached", "--name-only"]), "u.txt", "the staged work stays staged");
    remove(root.as_path());
}
