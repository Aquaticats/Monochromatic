//! What:
//!  Every command the wrapper prints as a remedy runs as printed:
//! through the built executable,
//! in a disposable
//!  repository put into the state that prints it,
//! and with the words a shell reads back.
//! Why:
//!  A message that names a command the wrapper's own policy then refuses leaves the person with no
//!  way forward,
//! and that happened:
//! the interrupted add's `git -C <path> reset --hard`,
//! `git stash branch`,
//! and
//!  `git clean -fd` were all printed as ways forward and all refused.
//! The controls read the command out of the printed
//!  text rather than restating it,
//! so a message that changes keeps being checked.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const printed = backtickedCommands(stderr).find((one) => one.includes(' reset '));
//! // expect(runAsPrinted(printed).stderr).not.toContain('"policyId"');
//! ```

use super::support::{
    Fixture, Observed, dead_copy_owner, fixture, git, observe_raw, porcelain, remove, repository,
    run_wrapped, stderr_of, wrapped,
};
use git_policy_cli::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
use git_policy_cli::worktree_copy_stage_claim::claim_stage_for_add_as;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The text between backticks in a message,
/// in the order the message prints it.
fn backticked_commands(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut rest: &str = text;
    while let Some(opening) = rest.find('`') {
        let after: &str = &rest[opening + 1..];
        let Some(closing) = after.find('`') else { break };
        found.push(String::from(&after[..closing]));
        rest = &after[closing + 1..];
    }
    return found;
}

/// What:
///  The words of one printed command,
///  read as a POSIX shell reads them.
/// Why:
///  The wrapper quotes a path that needs it with single quotes and leaves a plain one bare,
/// so a reader that only
///  splits on spaces would run a different command than the person pastes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shellWords(command: string): string[]; // 'a b' => ['a b']
/// ```
fn shell_words(command: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    let mut current: String = String::new();
    let mut quoted: bool = false;
    let mut started: bool = false;
    for character in command.chars() {
        match character {
            '\'' => {
                quoted = !quoted;
                started = true;
            }
            ' ' | '\t' if !quoted => {
                if started {
                    words.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            other => {
                current.push(other);
                started = true;
            }
        }
    }
    if started {
        words.push(current);
    }
    return words;
}

/// What:
///  Run one printed command,
///  as printed,
///  through the wrapper from `directory`.
/// Why:
///  The printed line starts with the `git` a person types,
///  and a shell resolves that word to the wrapper on the
///  fixture's `PATH`,
///  so the program word is the wrapper itself and the rest are its arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runAsPrinted(fixture: Fixture, directory: string, printed: string): Observed;
/// ```
fn run_as_printed(fixture: &Fixture, directory: &Path, printed: &str) -> Observed {
    let words: Vec<String> = shell_words(printed);
    let arguments: &[String] = match words.first().map(String::as_str) {
        Some("git") => &words[1..],
        _ => words.as_slice(),
    };
    return observe_raw(wrapped(fixture).current_dir(directory).args(arguments), b"");
}

/// Whether the wrapper's own policy refused the run.
fn refused_by_policy(observed: &Observed) -> bool {
    return stderr_of(observed).contains("\"policyId\"");
}

/// A repository with `tracked.txt` committed and the branch `topic`.
fn repository_with_a_tracked_file(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(fixture, repo.as_path(), &["add", "--", "tracked.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    git(fixture, repo.as_path(), &["branch", "topic"]);
    return repo;
}

/// What:
///  A repository,
///  a worktree registered there whose checkout never ran
///  (the shape a kill between Git's registration and its checkout leaves),
///  and a second linked worktree to run commands from.
/// Why:
///   Only a command in a linked worktree or a bare repository runs the recovery that reports the
///  worktree,
///  which is where the person reads the line;
///  the main worktree bypasses it by design.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { repo, destination, watcher } = worktreeWhoseCheckoutNeverRan(fixture);
/// ```
fn worktree_whose_checkout_never_ran(fixture: &Fixture) -> (PathBuf, PathBuf, PathBuf) {
    let repo: PathBuf = repository_with_a_tracked_file(fixture, "repo");
    let destination: PathBuf = fixture.root.join("half-added");
    let shown: &str = destination.to_str().expect("the fixture path is text");
    git(fixture, repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", shown, "topic"]);
    let watcher: PathBuf = fixture.root.join("watcher");
    git(fixture, repo.as_path(), &["worktree", "add", "--quiet", "--detach", watcher.to_str().expect("text")]);
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    // The record the wrapper's own add writes before real Git runs; its owner is a process that is gone, which is what
    // an add killed between the registration and the checkout leaves.
    let _claim = claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_copy_owner())
        .expect("the destination record of an add");
    return (repo, destination, watcher);
}

/// The command that finishes an interrupted add's checkout runs as printed,
/// and leaves the worktree whole.
#[test]
fn the_interrupted_adds_finish_command_runs_as_printed() {
    let fixture: Fixture = fixture("printed-interrupted-add");
    let (repo, destination, watcher): (PathBuf, PathBuf, PathBuf) = worktree_whose_checkout_never_ran(&fixture);
    // Every later command in a linked worktree reports the worktree the interrupted add left.
    let noticed: Observed = run_wrapped(&fixture, watcher.as_path(), &["rev-parse", "--is-inside-work-tree"]);
    let text: String = stderr_of(&noticed);
    assert!(text.contains("an interrupted `git worktree add` left"), "{text}");
    let printed: Vec<String> = backticked_commands(text.as_str());
    let finish: &String = printed
        .iter()
        .find(|one| return one.contains(" reset "))
        .unwrap_or_else(|| panic!("the report names the command that finishes the checkout: {text}"));
    let ran: Observed = run_as_printed(&fixture, repo.as_path(), finish.as_str());
    assert!(
        !refused_by_policy(&ran),
        "the command the report prints was refused: {finish}\n{}",
        stderr_of(&ran)
    );
    assert_eq!(ran.code, Some(0), "{finish}: {}", stderr_of(&ran));
    // What the command promises: the checkout is there, so nothing differs from `HEAD` any more.
    assert_eq!(porcelain(&fixture, destination.as_path()), "");
    remove(&fixture);
}

/// The `git stash drop` report's branch command runs as printed where the report says it does,
/// and the report says where that is.
#[test]
fn the_stash_drops_branch_command_runs_as_printed() {
    let fixture: Fixture = fixture("printed-stash-branch");
    let repo: PathBuf = repository_with_a_tracked_file(&fixture, "repo");
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", linked.to_str().expect("text"), "topic"]);
    std::fs::write(linked.join("tracked.txt"), b"two\n").expect("edit");
    git(&fixture, linked.as_path(), &["stash", "push", "--message=parked"]);
    // The report of a stash drop in the main worktree, where every stash form is refused.
    let dropped: Observed = run_wrapped(&fixture, repo.as_path(), &["stash", "drop"]);
    let text: String = stderr_of(&dropped);
    assert!(text.contains("git stash branch <branch>"), "{text}");
    assert!(
        !text.contains("git stash branch <branch> (allowed)"),
        "the report must not call a refused command allowed: {text}"
    );
    // The report's own words say a clean linked worktree runs it, so the command runs there.
    let ran: Observed = run_wrapped(&fixture, linked.as_path(), &["stash", "branch", "from-stash"]);
    assert!(!refused_by_policy(&ran), "git stash branch was refused in a clean linked worktree: {}", stderr_of(&ran));
    assert_eq!(ran.code, Some(0), "{}", stderr_of(&ran));
    remove(&fixture);
}

/// The `git clean -x` report's way to delete only untracked files runs as printed,
/// and keeps the ignored files.
#[test]
fn the_clean_ignored_reports_delete_runs_as_printed() {
    let fixture: Fixture = fixture("printed-clean-ignored");
    let repo: PathBuf = repository_with_a_tracked_file(&fixture, "repo");
    std::fs::write(repo.join(".gitignore"), b"ignored.txt\n").expect("ignore");
    git(&fixture, repo.as_path(), &["add", "--", ".gitignore"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=ignore"]);
    std::fs::write(repo.join("ignored.txt"), b"kept\n").expect("ignored file");
    std::fs::write(repo.join("untracked.txt"), b"new\n").expect("untracked file");
    let refused: Observed = run_wrapped(&fixture, repo.as_path(), &["clean", "-f", "-x"]);
    let text: String = stderr_of(&refused);
    assert!(text.contains("git clean"), "{text}");
    assert!(
        !text.contains("(allowed while no untracked file is precious)"),
        "the report must not call a refused clean allowed: {text}"
    );
    let printed: Vec<String> = backticked_commands(text.as_str());
    let delete: &String = printed
        .iter()
        .find(|one| return one.contains(" clean ") && one.contains(WORKTREE_ENFORCEMENT_ESCAPE_HATCH))
        .unwrap_or_else(|| panic!("the report names a clean that deletes untracked files only: {text}"));
    let ran: Observed = run_as_printed(&fixture, repo.as_path(), delete.as_str());
    assert!(!refused_by_policy(&ran), "the command the report prints was refused: {delete}\n{}", stderr_of(&ran));
    assert_eq!(ran.code, Some(0), "{delete}: {}", stderr_of(&ran));
    // The untracked file went and the ignored one stayed, which is what the report promises.
    assert!(!repo.join("untracked.txt").exists(), "the untracked file was deleted");
    assert!(repo.join("ignored.txt").exists(), "the ignored file was kept");
    remove(&fixture);
}
