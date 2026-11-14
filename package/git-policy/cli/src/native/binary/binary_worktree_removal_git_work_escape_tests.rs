//! What:
//!  Consumer-level controls for the opt-out of the guard that refuses a removal taking Git work,
//!  through the built executable:
//!  a typed removal that would destroy such work is refused and the refusal names the opt-out,
//!  and the command it prints runs the removal as real Git runs it.
//! Why:
//!  The rule is the owner's
//!  (`SPEC.md`, "What never discarding a person's data means"):
//!  a command that could destroy a person's data is refused,
//!  and the person who means it runs it with the guard's opt-out,
//!  which then runs exactly as real Git runs it.
//!  Only a command that starts the executable over a disposable repository shows the refusal,
//!  the spellings the wrapper reads, and what each of them ends as.
//!  The worktrees here are plain directories real Git made,
//!  so the controls need no btrfs and show the guard rather than the destroy.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refused = runWrapped(fixture, repo, ['worktree', 'remove', task]); // exit 1, names the opt-out
//! const ran = runWrapped(fixture, repo, printedWords(refused)); // exit 0, the worktree is gone
//! ```

use super::support::{Fixture, Observed, fixture, git, hermetic, observe, remove, repository, run_direct, run_wrapped, stderr_of, wrapper_first_path};
use git_policy_cli::worktree_added_by_the_wrapper::record;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;

/// The file a tracked edit changes.
const TRACKED: &str = "base.txt";

/// The branch the worktree is added on.
const BRANCH: &str = "task";

/// What one fixture holds.
struct Scene {
    /// The fixture whose wrapper link every command starts.
    fixture: Fixture,
    /// The main checkout.
    repo: PathBuf,
    /// The worktree every control removes.
    task: PathBuf,
}

impl Scene {
    /// What the wrapper said and answered for one typed removal.
    fn remove_with(&self, words: &[&str]) -> Observed {
        return run_wrapped(&self.fixture, self.repo.as_path(), words);
    }

    /// The command a refusal printed,
    /// taken out of its own text between the backticks.
    fn printed_command(&self, refusal: &str) -> String {
        let marker: &str = "`";
        let start: usize = refusal.find("`git worktree --no-enforce-worktree remove ").expect("the refusal prints the command that runs the removal") + marker.len();
        let rest: &str = &refusal[start..];
        let end: usize = rest.find('`').expect("the printed command ends with a backtick");
        return rest[..end].to_owned();
    }

    /// Run a printed command the way a person does:
    /// through a shell,
    /// with the fixture's own environment and the wrapper first on its `PATH`.
    /// Why:
    ///  Splitting the text on spaces is not what a shell does with quotes,
    /// and the quoting is exactly what these controls are about.
    ///  `bash` is the shell the quoting's own controls use,
    /// and the one whose ANSI-C form a name with a line break needs.
    fn run_through_a_shell(&self, printed: &str) -> Observed {
        let mut shell: Command = Command::new("bash");
        shell.arg("-c").arg(printed);
        hermetic(&mut shell, &self.fixture, wrapper_first_path(&self.fixture).as_os_str());
        // `git worktree remove <path>` names a worktree of the repository the command stands in, so the paste happens there.
        shell.current_dir(self.repo.as_path());
        return observe(&mut shell, b"");
    }

    /// Mill the worktree's `HEAD` off its branch and commit there.
    fn detach_and_commit(&self) -> String {
        let added: PathBuf = self.task.join("more.txt");
        std::fs::write(added.as_path(), "more work\n").expect("a file in the worktree");
        run_direct(&self.fixture, self.task.as_path(), &["checkout", "--quiet", "--detach"]);
        run_direct(&self.fixture, self.task.as_path(), &["add", "more.txt"]);
        run_direct(&self.fixture, self.task.as_path(), &["commit", "--quiet", "--message=work on a detached head"]);
        let named: std::process::Output = git(&self.fixture, self.task.as_path(), &["rev-parse", "HEAD"]);
        return String::from_utf8_lossy(named.stdout.as_slice()).trim().to_owned();
    }

    /// Hide an edit to a tracked file behind `assume-unchanged`.
    fn hide_an_edit(&self) {
        run_direct(&self.fixture, self.task.as_path(), &["update-index", "--assume-unchanged", TRACKED]);
        std::fs::write(self.task.join(TRACKED), "my edit, only here\n").expect("an edit to a tracked file");
    }
}

/// What:
///  A repository with one linked worktree real Git made,
///  which the wrapper has recorded as its own.
/// Why:
///  The controls are about the guard a removal asks,
///  not about the subvolume a destroy needs:
///  a plain directory Git made goes to Git's own removal,
///  where the escape's promise is exactly Git's ending,
///  and the record is what lets a sweep examine the worktree at all.
fn a_repository_with_a_recorded_worktree(name: &str) -> Scene {
    return a_repository_with_a_recorded_worktree_in(name, "task");
}

/// What:
///  The same repository,
///  with the worktree at a directory of the caller's naming.
/// Why:
///  A control of the quoting needs a path holding a space,
///  a quote,
///  a dollar sign and a line break,
///  and only the directory's own name can hold those bytes.
fn a_repository_with_a_recorded_worktree_in(name: &str, directory: &str) -> Scene {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join(TRACKED), "base\n").expect("a tracked file");
    git(&fixture, &repo, &["add", TRACKED]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=a tracked file"]);
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("the common Git directory");
    let added: PathBuf = fixture.root.join(directory);
    let task_text: &str = added.to_str().expect("a UTF-8 fixture path");
    run_direct(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", BRANCH, task_text]);
    let task: PathBuf = std::fs::canonicalize(added.as_path()).expect("the worktree's real path");
    record(common.as_path(), task.as_path()).expect("the record that this program made the worktree");
    // A commit of the branch's own, so a sweep reads the branch as one that landed work.
    std::fs::write(task.join("task-file.txt"), "task work\n").expect("a file in the worktree");
    run_direct(&fixture, task.as_path(), &["add", "task-file.txt"]);
    run_direct(&fixture, task.as_path(), &["commit", "--quiet", "--message=task work"]);
    return Scene { fixture, repo, task };
}

/// What:
///  The refusal of a typed removal names what it would destroy and the opt-out,
///  and the command it prints runs the removal as real Git runs it.
/// Why:
///  Naming an escape nobody can type leaves the person holding a command that fails,
///  and a guard that stands on after the opt-out breaks the owner's rule.
#[test]
fn a_hidden_edit_is_refused_and_the_printed_opt_out_runs_the_removal() {
    let scene: Scene = a_repository_with_a_recorded_worktree("removal-escape-hidden-edit");
    let task: String = scene.task.display().to_string();
    scene.hide_an_edit();
    let refused: Observed = scene.remove_with(&["worktree", "remove", task.as_str()]);
    let said: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "the guard refuses the removal: {said}");
    assert!(said.contains("keeps Git work"), "{said}");
    assert!(said.contains("base.txt"), "{said}");
    let printed: String = format!("`git worktree --no-enforce-worktree remove {task}`");
    assert!(said.contains(printed.as_str()), "the refusal prints the opt-out in a spelling that runs: {said}");
    assert!(scene.task.join(TRACKED).exists(), "nothing was removed");
    let printed: String = scene.printed_command(said.as_str());
    let ran: Observed = scene.run_through_a_shell(printed.as_str());
    assert_eq!(ran.code, Some(0), "the printed command runs as real Git runs it: {}", stderr_of(&ran));
    assert!(!scene.task.exists(), "and the worktree is gone");
    remove(&scene.fixture);
}

/// What:
///  A commit only the worktree's `HEAD` names is refused the same way,
///  and the printed opt-out deletes it too.
/// Why:
///  This is the state the guard was added for,
///  and the person who means it must be able to say so.
#[test]
fn a_stranded_commit_is_refused_and_the_printed_opt_out_runs_the_removal() {
    let scene: Scene = a_repository_with_a_recorded_worktree("removal-escape-stranded-commit");
    let task: String = scene.task.display().to_string();
    let commit: String = scene.detach_and_commit();
    let refused: Observed = scene.remove_with(&["worktree", "remove", task.as_str()]);
    let said: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "the guard refuses the removal: {said}");
    assert!(said.contains(&commit[..12]), "the refusal names the commit: {said}");
    assert!(said.contains("`git worktree --no-enforce-worktree remove "), "the refusal prints the opt-out: {said}");
    assert!(scene.task.exists(), "nothing was removed");
    let printed: String = scene.printed_command(said.as_str());
    let ran: Observed = scene.run_through_a_shell(printed.as_str());
    assert_eq!(ran.code, Some(0), "the printed command runs as real Git runs it: {}", stderr_of(&ran));
    assert!(!scene.task.exists(), "and the worktree is gone");
    remove(&scene.fixture);
}

/// What:
///  Every spelling the wrapper reads for `git worktree remove` runs the removal,
///  and the one after `remove` is real Git's own refusal of the option.
/// Why:
///  The record says the opt-out is written directly after the subcommand,
///  and for `git worktree remove` that is after `worktree`:
///  the wrapper's reader scans controls there and in global-option position,
///  and hands everything after `remove` to Git.
///  A person who follows a refusal must not have to guess which of the three it meant.
#[test]
fn the_spellings_the_wrapper_reads_run_and_the_one_after_remove_is_real_gits() {
    let read: [[&str; 4]; 4] = [
        ["worktree", "--no-enforce-worktree", "remove", ""],
        ["--no-enforce-worktree", "worktree", "remove", ""],
        ["worktree", "--no-enforce-linked-worktree-only", "remove", ""],
        ["--no-enforce-linked-worktree-only", "worktree", "remove", ""],
    ];
    for (index, words) in read.iter().enumerate() {
        let scene: Scene = a_repository_with_a_recorded_worktree(format!("removal-escape-spelling-{index}").as_str());
        let task: String = scene.task.display().to_string();
        scene.hide_an_edit();
        let mut arguments: Vec<&str> = words.to_vec();
        *arguments.last_mut().expect("a place for the path") = task.as_str();
        let ran: Observed = scene.remove_with(arguments.as_slice());
        assert_eq!(ran.code, Some(0), "the opt-out at {words:?} runs the removal: {}", stderr_of(&ran));
        assert!(!scene.task.exists(), "and the worktree is gone at {words:?}");
        remove(&scene.fixture);
    }
    let scene: Scene = a_repository_with_a_recorded_worktree("removal-escape-after-remove");
    let task: String = scene.task.display().to_string();
    scene.hide_an_edit();
    let after: Observed = scene.remove_with(&["worktree", "remove", "--no-enforce-worktree", task.as_str()]);
    assert_eq!(after.code, Some(129), "the option after `remove` reaches real Git: {}", stderr_of(&after));
    assert!(scene.task.exists(), "and nothing was removed");
    remove(&scene.fixture);
}

/// What:
///  The sweep keeps a worktree that holds a commit nothing else names,
///  even though its branch landed and the worktree is one this program made.
/// Why:
///  Nobody typed anything in a merge,
///  so no opt-out exists there:
///  the automatic route must keep the worktree, as it did before the opt-out existed.
///  The commit sits on a reference this worktree keeps,
///  so its `HEAD` still names the branch and the sweep examines it as the candidate its landing made it:
///  a worktree detached after its commit is no candidate at all, which is the quiet keep the sweep's own
///  candidate rule gives.
#[test]
fn the_sweep_keeps_a_worktree_that_holds_a_commit_nothing_else_names() {
    let scene: Scene = a_repository_with_a_recorded_worktree("removal-escape-sweep-keeps");
    let asked: std::process::Output = git(&scene.fixture, scene.task.as_path(), &["rev-parse", "HEAD"]);
    let head: String = String::from_utf8_lossy(asked.stdout.as_slice()).trim().to_owned();
    let made: std::process::Output = git(&scene.fixture, scene.task.as_path(), &["commit-tree", format!("{head}^{{tree}}").as_str(), "-p", head.as_str(), "-m", "a commit only a worktree reference names"]);
    let commit: String = String::from_utf8_lossy(made.stdout.as_slice()).trim().to_owned();
    let kept: std::process::Output = git(&scene.fixture, scene.task.as_path(), &["update-ref", "refs/worktree/person-keep", commit.as_str()]);
    assert!(kept.status.success(), "the person's reference is written");
    let merged: Observed = scene.remove_with(&["merge", "--no-edit", BRANCH]);
    let said: String = stderr_of(&merged);
    assert_eq!(merged.code, Some(0), "the merge itself goes through: {said}");
    assert!(scene.task.exists(), "the sweep left the worktree: {said}");
    assert!(said.contains(&commit[..12]), "and named the commit it would have destroyed: {said}");
    remove(&scene.fixture);
}

/// What:
///  A removal that skips the copy is refused for a hidden edit,
///  and the opt-out runs it,
///  with the same skip in place.
/// Why:
///  The guard lives where the removal is made,
///  and `--no-worktree-copy` sends the command to real Git before that place:
///  the question is asked before the route is chosen,
///  so a flag that skips another step cannot skip this one.
#[test]
fn a_removal_that_skips_the_copy_is_refused_and_its_opt_out_runs_it() {
    let scene: Scene = a_repository_with_a_recorded_worktree("removal-skip-copy-hidden-edit");
    let task: String = scene.task.display().to_string();
    scene.hide_an_edit();
    let refused: Observed = scene.remove_with(&["--no-worktree-copy", "worktree", "remove", task.as_str()]);
    let said: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "a removal that skips the copy is still refused: {said}");
    assert!(said.contains("base.txt"), "and names the edit: {said}");
    assert!(scene.task.join(TRACKED).exists(), "nothing was removed");
    let ran: Observed = scene.remove_with(&["--no-worktree-copy", "worktree", "--no-enforce-worktree", "remove", task.as_str()]);
    assert_eq!(ran.code, Some(0), "the opt-out runs it as real Git runs it: {}", stderr_of(&ran));
    assert!(!scene.task.exists(), "and the worktree is gone");
    remove(&scene.fixture);
}

/// What:
///  The same for a commit only the worktree's `HEAD` names.
/// Why:
///  A commit is the data a removal cannot bring back,
///  and the route that skips the copy destroyed it before this control existed.
#[test]
fn a_removal_that_skips_the_copy_is_refused_for_a_stranded_commit_too() {
    let scene: Scene = a_repository_with_a_recorded_worktree("removal-skip-copy-stranded-commit");
    let task: String = scene.task.display().to_string();
    let commit: String = scene.detach_and_commit();
    let refused: Observed = scene.remove_with(&["--no-worktree-copy", "worktree", "remove", task.as_str()]);
    let said: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "a removal that skips the copy is still refused: {said}");
    assert!(said.contains(&commit[..12]), "and names the commit: {said}");
    assert!(scene.task.exists(), "nothing was removed");
    let ran: Observed = scene.remove_with(&["--no-worktree-copy", "worktree", "--no-enforce-worktree", "remove", task.as_str()]);
    assert_eq!(ran.code, Some(0), "the opt-out runs it as real Git runs it: {}", stderr_of(&ran));
    assert!(!scene.task.exists(), "and the worktree is gone");
    remove(&scene.fixture);
}

/// What:
///  The command a refusal prints runs as printed for a worktree path holding a space,
///  a single quote,
///  a double quote,
///  a dollar sign and a line break.
/// Why:
///  The report hands the person a command to copy and run,
///  and a raw path turns it into several words or into an expansion
///  (a review of `bab700404d`, finding 2).
///  The text is run through a shell here,
///  because splitting it on spaces is not what a shell does with quotes.
#[test]
fn the_printed_opt_out_runs_for_a_hostile_path() {
    let scene: Scene = a_repository_with_a_recorded_worktree_in("removal-hostile-path", "my task's $HOME \"q\"\nafter");
    let task: String = scene.task.display().to_string();
    scene.hide_an_edit();
    let refused: Observed = scene.remove_with(&["worktree", "remove", task.as_str()]);
    let said: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "the guard refuses the removal: {said}");
    let printed: String = scene.printed_command(said.as_str());
    assert!(!printed.contains("\n"), "the printed command holds no line break of its own: {printed}");
    let ran: Observed = scene.run_through_a_shell(printed.as_str());
    assert_eq!(ran.code, Some(0), "the printed command runs as printed: {} [{printed}]", stderr_of(&ran));
    assert!(!scene.task.exists(), "and the worktree is gone");
    remove(&scene.fixture);
}
