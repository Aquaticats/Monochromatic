//! What:
//!  The one allowance the wrapper's own steps of a `git worktree add` share,
//!  through the built executable,
//!  against a stand-in for real Git that stalls at one named child.
//! Why:
//!   A step whose every try is bounded and whose number of tries is not waits without a bound:
//!  with a pipe stood at
//!  `.git/info/exclude`,
//!  every Git child of an add stalls in the same read,
//!  each child is ended at its own bound,
//!  the copy step is
//!  tried again,
//!  and on 2026-11-13 an add had not answered after 240 seconds,
//!  one child ended at ten seconds and another
//!  following,
//!  with nothing bounding the whole.
//!  The project's requirement is that nothing waits forever or indefinitely,
//!  so the add ends within
//!  one total allowance,
//!  says what it was waiting for and how long,
//!  and leaves no half-made worktree and no record of it
//!  behind.
//!  A child that stalls once and answers the second time is the other side of the same rule:
//!  the add still
//!  lands,
//!  because its own work is far inside the allowance.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const add = wrappedAdd({ allowanceSeconds: 15 }); // the stand-in stalls the untracked scan every time
//! expect(add.ending).toBe('itself'); expect(add.stderr).toContain('was not made');
//! ```

/// Import the shared fixtures, the bounded process helpers and the reading of who ended a command.
use super::support::{Fixture, Observed, bounded_for, executable, fixture, git, remove, repository, run_wrapped, stderr_of, wrapper_first_path};
use crate::support_observation_bound::{Answer, Ending, wait_under_bound};
use git_policy_cli::action::ENGINE_FAILURE_EXIT_CODE;
use git_policy_cli::worktree_copy_stage_claim::stage_claims_exist;
use std::ffi::OsStr;
use std::os::unix::process::CommandExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

/// The environment variable that sets the allowance of one add's own steps, in whole seconds.
const ALLOWANCE_VARIABLE: &str = "CLI_GIT_ADD_OWN_WORK_SECONDS";

/// The variable that sets how long one Git step of the wrapper may show no sign of work, in whole seconds.
const CHILD_WAIT_VARIABLE: &str = "CLI_GIT_CHILD_WAIT_SECONDS";

/// The allowance the control of a child that stalls every time gives the add, in seconds.
/// It is a healthy add's own work many times over and far below the window of the step it ends, so
/// the add ends in seconds and the control says so.
const STALLING_ALLOWANCE_SECONDS: &str = "15";

/// The allowance the control of a child that stalls once gives the add, in seconds.
/// Wide enough that a step's own quiet window is what ends the first try, not the allowance,
/// so the second try has the allowance to answer in.
const ONCE_ALLOWANCE_SECONDS: &str = "120";

/// How long the first try of the untracked scan may show no sign of work before the step is ended,
/// in seconds, for the control of a child that stalls once.
const ONCE_QUIET_WINDOW_SECONDS: &str = "2";

/// The bound this control gives the wrapped add, in seconds:
/// wide enough for an add that ends at its allowance and its clean-up.
/// The wrapped command's own `timeout` and this control's wait both take it,
/// so whichever ends the command first is the bound's doing.
const CONTROL_BOUND_SECONDS: &str = "60";

/// The words of the child the stand-in stalls:
/// the untracked scan of the clean check,
/// which is the step that was left in the read on 2026-11-13.
const STALLED_CHILD: &str = "ls-files --others --exclude-standard --directory --no-empty-directory -z";

/// What:
///  A `git` placed after the wrapper on `PATH`,
///  so it is the real Git the wrapper starts,
///  which runs the real Git for every call except the untracked scan,
///  which it turns into a Git that shows
///  no sign of work.
/// Why:
///   A pipe stood at `.git/info/exclude` makes every Git child of an add wait in the same read,
///  and a mount that stopped
///  answering does the same on this machine.
///  The stand-in does it at one named child and no other,
///  so the control names
///  the step that stalled and every other child of the add answers.
///  A child that sleeps shows no sign of work:
///  no bytes move,
///  it spends no processor time and no task of it is able to run,
///  which is what
///  the quiet window and the ceiling end a step on.
///  `stalls` says which tries stall:
///  `every` for a child that never answers,
///  `once` for one whose second try runs the real Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shim = standInGit({ stalls: 'every', started }); // a directory to put after the wrapper on PATH
/// ```
fn stand_in_git(fixture: &Fixture, stalls: &str, started: &Path) -> PathBuf {
    let directory: PathBuf = fixture.root.join("shim");
    std::fs::create_dir_all(directory.as_path()).expect("shim directory");
    let program: PathBuf = directory.join("git");
    // The count of asks goes in a file beside the marker, so a second try reads what the first wrote and the shell
    // keeps no state between two starts of the stand-in.
    let script: String = format!(
        "#!/bin/sh
case \" $* \" in
  *\" {STALLED_CHILD} \"*)
    tries=$(($(cat '{started}.tries' 2>/dev/null || echo 0) + 1))
    echo \"$tries\" > '{started}.tries'
    if [ '{stalls}' = every ] || [ \"$tries\" = 1 ]; then
      printf stalled > '{started}'
      exec /usr/bin/sleep 600
    fi
    ;;
esac
exec /usr/bin/git \"$@\"
",
        started = started.display()
    );
    executable(program.as_path(), script.as_bytes());
    return directory;
}

/// A repository with two tracked files and an ignored directory,
/// so an add has something to check out and something to copy.
fn populated_repository(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::create_dir_all(repo.join("src")).expect("source directory");
    std::fs::write(repo.join("README.md"), b"readme\n").expect("readme");
    std::fs::write(repo.join("src/leaf.txt"), vec![b'x'; 5_000]).expect("leaf");
    std::fs::write(repo.join(".gitignore"), b"ignored/\n").expect("gitignore");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=files"]);
    std::fs::create_dir_all(repo.join("ignored")).expect("ignored directory");
    std::fs::write(repo.join("ignored/cache"), b"copied state\n").expect("ignored file");
    return repo;
}

/// The avenue the wrapped add is given:
/// the wrapper's link first and then the stand-in,
/// and no `/usr/bin`,
/// because the wrapper tries the
///  conventional locations of Git ahead of the rest of `PATH`,
///  so a `/usr/bin/git` there would be chosen over the
///  stand-in,
/// which runs the real one by its absolute path.
fn stand_in_avenue(fixture: &Fixture, shim: &Path) -> String {
    return format!("{}:{}", fixture.root.join("bin").display(), shim.display());
}

/// One wrapped `git worktree add` of `destination` from `repo`,
/// under this control's bound and the add's own allowance.
fn wrapped_add(fixture: &Fixture, avenue: &str, repo: &Path, destination: &Path, allowance: &str) -> Command {
    let mut command: Command = bounded_for(fixture, fixture.root.join("bin/git").as_path(), OsStr::new(avenue), CONTROL_BOUND_SECONDS);
    command.current_dir(repo);
    command.args(["worktree", "add", "-b", "topic"]).arg(destination);
    command.env(ALLOWANCE_VARIABLE, allowance);
    return command;
}

/// Run a prepared wrapped command under this control's bound and say what came of it,
/// without panicking on a command the bound ended:
/// a control that must show an add still going at its bound reads the ending itself.
fn run_under_the_control(command: &mut Command) -> Answer {
    // The command leads a group of its own, so the bound ends the whole tree the add started and not this test.
    command.process_group(0);
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the wrapped add starts");
    let group: libc::pid_t = libc::pid_t::try_from(child.id()).expect("a process ID fits the pid type");
    let bound: Duration = Duration::from_secs(CONTROL_BOUND_SECONDS.parse().expect("whole seconds"));
    return wait_under_bound(child, Some(group), bound, b"");
}

/// The registrations of a repository's worktrees,
/// as the porcelain listing names them.
fn worktree_registrations(fixture: &Fixture, repo: &Path) -> String {
    return String::from_utf8_lossy(git(fixture, repo, &["worktree", "list", "--porcelain"]).stdout.as_slice()).into_owned();
}

/// The names a directory holds,
/// sorted,
/// so a failure can say what an add left where the worktree was not made.
fn entries_of(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(directory)
        .map(|listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); })
        .unwrap_or_default();
    names.sort();
    return names;
}

/// What:
///  The whole point of the control:
/// an add whose own child stalls every time it is asked ends by itself within its allowance,
/// says what it was waiting for and how long,
/// and leaves neither the worktree nor a record of the add behind.
/// Why:
///   The failing shape is the one the brief records from 2026-11-13:
///  every child is ended at its own bound and another
///  follows,
/// so the add outlives every bound it is given.
///  This control's own bound is what a failing run ends at,
///  and the
///  wrapped add must end before it,
///  so a run that waits without a bound and a run that ends are told apart
///  in one control.
#[test]
fn an_add_whose_own_child_stalls_every_time_ends_within_its_allowance_and_leaves_no_worktree() {
    let fixture: Fixture = fixture("add-allowance-stalls");
    let repo: PathBuf = populated_repository(&fixture);
    let destination: PathBuf = fixture.root.join("topic");
    let started: PathBuf = fixture.root.join("the-stalled-child-started");
    let shim: PathBuf = stand_in_git(&fixture, "every", started.as_path());
    let avenue: String = stand_in_avenue(&fixture, shim.as_path());
    let mut command: Command = wrapped_add(&fixture, avenue.as_str(), repo.as_path(), destination.as_path(), STALLING_ALLOWANCE_SECONDS);
    let answer: Answer = run_under_the_control(&mut command);
    let said: String = String::from_utf8_lossy(answer.stderr.as_slice()).into_owned();
    assert!(started.exists(), "the stand-in Git was never asked for the untracked scan, so nothing stalled: {said}");
    assert_eq!(answer.ending, Ending::Itself, "the add was still running at this control's {CONTROL_BOUND_SECONDS}-second bound: {said}");
    assert_eq!(answer.status.code(), Some(ENGINE_FAILURE_EXIT_CODE), "an add that ended at its allowance reports the wrapper's own failure: {said}");
    assert!(said.contains(ALLOWANCE_VARIABLE), "the line names the variable that sets the allowance: {said}");
    assert!(said.contains(STALLED_CHILD), "the line names the step it was waiting for: {said}");
    assert!(said.contains("was not made"), "the line says what it left: {said}");
    assert!(!destination.exists(), "nothing of the worktree is left: {}", entries_of(fixture.root.as_path()).join(", "));
    let registrations: String = worktree_registrations(&fixture, repo.as_path());
    assert!(!registrations.contains(destination.to_string_lossy().as_ref()), "the worktree is not registered: {registrations}");
    assert!(
        !stage_claims_exist(repo.join(".git").as_path()).expect("the repository's claims are listed"),
        "the record of what the add owed is released: {}",
        entries_of(fixture.root.as_path()).join(", ")
    );
    let stages: Vec<String> = entries_of(fixture.root.as_path()).into_iter().filter(|name: &String| { return name.starts_with(".cli-git-worktree-copy-"); }).collect();
    assert!(stages.is_empty(), "no stage of the stopped add is left: {stages:?}");
}

/// What:
///  A second add of the same worktree,
///  after one that ended at its allowance,
///  works,
///  with the same allowance and nothing of the stopped add in its way.
/// Why:
///   An add that ends at its bound must leave nothing the next command trips on:
///  a destination that is there,
///  a registration nobody made,
///  or a record that says a worktree is still being filled.
///  The control also keeps the cost claim honest,
///  because a healthy add of this fixture runs far inside the allowance.
#[test]
fn a_second_add_of_the_same_worktree_after_one_ended_at_its_allowance_works() {
    let fixture: Fixture = fixture("add-allowance-second");
    let repo: PathBuf = populated_repository(&fixture);
    let destination: PathBuf = fixture.root.join("topic");
    let started: PathBuf = fixture.root.join("the-stalled-child-started");
    let shim: PathBuf = stand_in_git(&fixture, "every", started.as_path());
    let avenue: String = stand_in_avenue(&fixture, shim.as_path());
    let stopped: Answer = run_under_the_control(&mut wrapped_add(&fixture, avenue.as_str(), repo.as_path(), destination.as_path(), STALLING_ALLOWANCE_SECONDS));
    assert_eq!(stopped.ending, Ending::Itself, "the first add ended before this control's bound: {}", String::from_utf8_lossy(stopped.stderr.as_slice()));
    assert_ne!(stopped.status.code(), Some(0), "the first add did not make the worktree: {}", String::from_utf8_lossy(stopped.stderr.as_slice()));
    // The second add meets real Git everywhere: the child the stopped add was waiting for answers now.
    let avenue_again: String = wrapper_first_path(&fixture).to_string_lossy().into_owned();
    let second: Answer = run_under_the_control(&mut wrapped_add(&fixture, avenue_again.as_str(), repo.as_path(), destination.as_path(), STALLING_ALLOWANCE_SECONDS));
    let said: String = String::from_utf8_lossy(second.stderr.as_slice()).into_owned();
    assert_eq!(second.ending, Ending::Itself, "the second add ran past this control's bound: {said}");
    assert_eq!(second.status.code(), Some(0), "the second add made the worktree: {said}");
    assert!(destination.join("README.md").is_file(), "the worktree holds the tracked files: {}", entries_of(destination.as_path()).join(", "));
    assert!(destination.join(".git").exists(), "the worktree is a registered worktree: {}", entries_of(destination.as_path()).join(", "));
    assert!(worktree_registrations(&fixture, repo.as_path()).contains(destination.to_string_lossy().as_ref()), "the repository names the new worktree");
    let status: Observed = run_wrapped(&fixture, destination.as_path(), &["status", "--porcelain"]);
    assert_eq!(status.code, Some(0), "the new worktree answers a Git command: {}", stderr_of(&status));
    remove(&fixture);
}

/// What:
///  An add whose own child stalls once and answers the second time still lands.
/// Why:
///   The allowance exists to end an add that cannot finish,
///  not to fail one whose step was slow once:
///  a step
///  ended for showing no sign of work is run again,
///  and the second try has the rest of the allowance.
///  The
///  line the wrapper prints when it runs a step again is the evidence that the first try was really ended
///  and the second really answered.
#[test]
fn an_add_whose_own_child_stalls_once_and_answers_the_second_time_still_lands() {
    let fixture: Fixture = fixture("add-allowance-once");
    let repo: PathBuf = populated_repository(&fixture);
    let destination: PathBuf = fixture.root.join("topic");
    let started: PathBuf = fixture.root.join("the-stalled-child-started");
    let shim: PathBuf = stand_in_git(&fixture, "once", started.as_path());
    let avenue: String = stand_in_avenue(&fixture, shim.as_path());
    let mut command: Command = wrapped_add(&fixture, avenue.as_str(), repo.as_path(), destination.as_path(), ONCE_ALLOWANCE_SECONDS);
    command.env(CHILD_WAIT_VARIABLE, ONCE_QUIET_WINDOW_SECONDS);
    let answer: Answer = run_under_the_control(&mut command);
    let said: String = String::from_utf8_lossy(answer.stderr.as_slice()).into_owned();
    assert!(started.exists(), "the stand-in Git was never asked for the untracked scan, so nothing stalled: {said}");
    assert_eq!(answer.ending, Ending::Itself, "the add ran past this control's bound: {said}");
    assert_eq!(answer.status.code(), Some(0), "the add landed after one ended try: {said}");
    assert!(said.contains("showed no sign of work and was ended; running it again"), "the ended try was run again: {said}");
    assert!(destination.join("README.md").is_file(), "the worktree holds the tracked files: {}", entries_of(destination.as_path()).join(", "));
    remove(&fixture);
}
