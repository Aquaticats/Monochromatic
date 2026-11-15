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
    let expected: String = format!("`git worktree --no-enforce-worktree remove {task}`");
    assert!(said.contains(expected.as_str()), "the refusal prints the opt-out in a spelling that runs: {said}");
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
    let said: String = stderr_of(&after);
    assert_eq!(after.code, Some(1), "the opt-out after `remove` is answered with where it goes: {said}");
    assert!(said.contains("directly after `worktree`"), "and says where that is: {said}");
    assert!(said.contains("`git worktree --no-enforce-worktree remove "), "with the command that runs it: {said}");
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

/// A named pipe at `path`,
/// which blocks whoever opens it for reading until a writer comes.
fn make_pipe(path: &std::path::Path) {
    use std::os::unix::ffi::OsStrExt as _;
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("path text");
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let result: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
    assert_eq!(result, 0, "mkfifo {}", path.display());
}

/// What:
///  The same command the wrapper runs,
///  under a bound of its own rather than the harness's long backstop.
/// Why:
///  A control whose subject is a write that must not wait has to end the wait itself,
///  or the failing case would hold the run for the harness's whole backstop.
fn wrapped_within(fixture: &Fixture, seconds: &str) -> Command {
    use super::support::bounded_for;
    return bounded_for(fixture, fixture.root.join("bin/git").as_path(), wrapper_first_path(fixture).as_os_str(), seconds);
}

/// The directory and the registration name a scene's worktree has.
fn admin_of(fixture: &Fixture, directory: &str) -> PathBuf {
    let common: PathBuf = std::fs::canonicalize(fixture.root.join("repo/.git")).expect("the common Git directory");
    return common.join("worktrees").join(directory);
}

/// What:
///  Under `core.ignorecase`, a name that differs in case from the worktree still names it for Git,
///  and the guard refuses it.
/// Why:
///  Git folds ASCII case when it matches the operand (`fspathcmp` under `core.ignorecase`),
///  so a guard that resolves bytes alone finds no worktree and stands down,
///  and Git removes a worktree holding a hidden edit or a commit nothing else names
///  (a review of `528279d099`, finding 1).
///  The case-equal name is the control that the guard is active at all.
#[test]
fn a_case_differing_name_is_guarded_where_git_resolves_it() {
    for upper in ["TASK", "TaSk"] {
        let scene: Scene = a_repository_with_a_recorded_worktree("removal-ignorecase-hidden-edit");
        scene.hide_an_edit();
        run_direct(&scene.fixture, scene.repo.as_path(), &["config", "core.ignorecase", "true"]);
        let refused: Observed = scene.remove_with(&["worktree", "remove", upper]);
        let said: String = stderr_of(&refused);
        assert_eq!(refused.code, Some(1), "{upper} must be refused under core.ignorecase: {said}");
        assert!(said.contains(TRACKED), "and names the hidden edit: {said}");
        assert!(scene.task.join(TRACKED).exists(), "nothing was removed");
        remove(&scene.fixture);
    }
    // The commit only a detached HEAD names, by the same name.
    let scene: Scene = a_repository_with_a_recorded_worktree("removal-ignorecase-stranded");
    let commit: String = scene.detach_and_commit();
    run_direct(&scene.fixture, scene.repo.as_path(), &["config", "core.ignorecase", "true"]);
    let refused: Observed = scene.remove_with(&["worktree", "remove", "TASK"]);
    let said: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "a commit only the HEAD names is refused by a case-differing name too: {said}");
    assert!(said.contains(&commit[..12]), "and named: {said}");
    assert!(scene.task.exists(), "nothing was removed");
    remove(&scene.fixture);
}

/// What:
///  Without `core.ignorecase`, the case-differing name is refused by this program,
///  and the opt-out it prints still runs, ending as real Git ends.
/// Why:
///  A case-differing name is a name this program resolves to no worktree of Git's own list,
///  and a removal is not handed to real Git on the guess that Git would refuse it:
///  the guard words the refusal and names the opt-out,
///  which is where Git's own answer to that name is heard.
///  Nothing is removed either way, and the opt-out shows the person the answer Git gives.
#[test]
fn without_ignorecase_a_case_differing_name_is_refused_and_the_opt_out_is_gits_own_answer() {
    let scene: Scene = a_repository_with_a_recorded_worktree("removal-ignorecase-off");
    scene.hide_an_edit();
    run_direct(&scene.fixture, scene.repo.as_path(), &["config", "core.ignorecase", "false"]);
    let refused: Observed = scene.remove_with(&["worktree", "remove", "TASK"]);
    let said: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "the name is refused rather than handed to Git on a guess: {said}");
    assert!(said.contains("nothing here proves which worktree"), "{said}");
    assert!(scene.task.join(TRACKED).exists(), "and nothing was removed");
    let printed: String = scene.printed_command(&said);
    let ran: Observed = scene.run_through_a_shell(printed.as_str());
    let answer: String = stderr_of(&ran);
    assert_eq!(ran.code, Some(128), "the opt-out answers as real Git answers this name: {answer}");
    assert!(answer.contains("is not a working tree"), "{answer}");
    assert!(scene.task.join(TRACKED).exists(), "and still nothing was removed");
    remove(&scene.fixture);
}

/// What:
///  A named pipe at the worktree's index stops neither a typed removal nor a merge,
///  and the worktree is kept.
/// Why:
///  The hidden-edit question reads that index,
///  and a read that waited for a writer would hold the person's merge for ever
///  (`SPEC.md`, "Every wait has a bound";
///  a review of `528279d099`, finding 2).
///  Each command carries a bound of its own, so the failing case ends the control instead of the run.
#[test]
fn a_named_pipe_at_the_index_holds_neither_a_removal_nor_a_merge() {
    let scene: Scene = a_repository_with_a_recorded_worktree("removal-fifo-index-typed");
    let task: String = scene.task.display().to_string();
    let index: PathBuf = admin_of(&scene.fixture, "task").join("index");
    std::fs::remove_file(index.as_path()).expect("a regular index goes");
    make_pipe(index.as_path());
    let refused: Observed = observe(wrapped_within(&scene.fixture, "180").current_dir(scene.repo.as_path()).args(["worktree", "remove", task.as_str()]).env("CLI_GIT_TEST_ONLY_JOURNAL_SOCKET", "@cli-git-review-journal-nobody"), b"");
    let said: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "the pipe is answered rather than waited on: {said}");
    assert!(said.contains("not a plain file"), "and named: {said}");
    assert!(scene.task.exists(), "nothing was removed");
    remove(&scene.fixture);

    let merging: Scene = a_repository_with_a_recorded_worktree("removal-fifo-index-merge");
    let piped: PathBuf = admin_of(&merging.fixture, "task").join("index");
    std::fs::remove_file(piped.as_path()).expect("a regular index goes");
    make_pipe(piped.as_path());
    let merged: Observed = observe(wrapped_within(&merging.fixture, "30").current_dir(merging.repo.as_path()).args(["merge", "--no-edit", BRANCH]).env("CLI_GIT_TEST_ONLY_JOURNAL_SOCKET", "@cli-git-review-journal-nobody"), b"");
    let merge_words: String = stderr_of(&merged);
    assert_eq!(merged.code, Some(0), "the merge ends, and keeps the worktree: {merge_words}");
    assert!(merging.task.exists(), "the sweep left it: {merge_words}");
    remove(&merging.fixture);
}

/// What one fixture holds when two worktrees share the last two components of their paths.
struct SharedSuffix {
    /// The fixture whose wrapper link every command starts.
    fixture: Fixture,
    /// The main checkout.
    repo: PathBuf,
    /// The worktree at `repo/proj/task`, which an operand of that suffix resolves to.
    inside: PathBuf,
    /// The worktree at `elsewhere/proj/task`, which the same suffix also ends.
    outside: PathBuf,
}

/// What:
///  A repository with two worktrees whose paths end in `proj/task`
///  and a tracked file in each.
/// Why:
///  The suffix names both, which is Git's ambiguous case: Git falls to the operand as a path,
///  and from the repository root that path is the one inside,
///  so the guard has to answer the question for that worktree rather than stand down
///  (a review of `9e6c8619a9`, finding 1).
fn a_repository_whose_worktrees_share_a_suffix(name: &str) -> SharedSuffix {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join(TRACKED), "base\n").expect("a tracked file");
    git(&fixture, &repo, &["add", TRACKED]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=a tracked file"]);
    let inside: PathBuf = repo.join("proj/task");
    let outside: PathBuf = fixture.root.join("elsewhere/proj/task");
    for (path, branch) in [(&inside, "one"), (&outside, "two")] {
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("the parent directory");
        run_direct(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", branch, path.to_str().expect("a UTF-8 fixture path")]);
    }
    return SharedSuffix { fixture, repo, inside: std::fs::canonicalize(inside).expect("the worktree's real path"), outside: std::fs::canonicalize(outside).expect("the worktree's real path") };
}

/// What:
///  An operand both registered paths end in is resolved as the path Git resolves it to,
///  so the guard asks about the worktree Git will remove and refuses.
/// Why:
///  Git answers a suffix match only for exactly one registration and falls to the resolved path for two,
///  so a guard that answered "no worktree" there stood down on the one name Git removes,
///  and the removal destroyed a hidden edit or a commit nothing else names.
///  The absolute operand is the control that the guard is active on the same worktree.
#[test]
fn a_suffix_two_worktrees_share_is_guarded_on_the_worktree_git_picks() {
    // The hidden edit, named by the suffix both worktrees end in.
    let scene: SharedSuffix = a_repository_whose_worktrees_share_a_suffix("removal-ambiguous-hidden-edit");
    run_direct(&scene.fixture, scene.inside.as_path(), &["update-index", "--assume-unchanged", TRACKED]);
    std::fs::write(scene.inside.join(TRACKED), "my edit, only here\n").expect("an edit to a tracked file");
    let refused: Observed = run_wrapped(&scene.fixture, scene.repo.as_path(), &["worktree", "remove", "proj/task"]);
    let said: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "the shared suffix is refused on the worktree Git picks: {said}");
    assert!(said.contains(TRACKED), "and the refusal names the hidden edit: {said}");
    assert!(scene.inside.join(TRACKED).exists(), "nothing was removed from it");
    assert!(scene.outside.join(TRACKED).exists(), "and the other worktree is untouched");
    // The control: the same worktree by a name only one registration answers.
    let unique: String = scene.inside.display().to_string();
    let refused_by_name: Observed = run_wrapped(&scene.fixture, scene.repo.as_path(), &["worktree", "remove", unique.as_str()]);
    assert_eq!(refused_by_name.code, Some(1), "a unique name is refused on the same worktree: {}", stderr_of(&refused_by_name));
    remove(&scene.fixture);

    // The commit only a detached HEAD names, by the same shared suffix.
    let stranded: SharedSuffix = a_repository_whose_worktrees_share_a_suffix("removal-ambiguous-stranded");
    std::fs::write(stranded.inside.join("more.txt"), "more work\n").expect("a file in the worktree");
    run_direct(&stranded.fixture, stranded.inside.as_path(), &["checkout", "--quiet", "--detach"]);
    run_direct(&stranded.fixture, stranded.inside.as_path(), &["add", "more.txt"]);
    run_direct(&stranded.fixture, stranded.inside.as_path(), &["commit", "--quiet", "--message=work on a detached head"]);
    let named: std::process::Output = git(&stranded.fixture, stranded.inside.as_path(), &["rev-parse", "HEAD"]);
    let commit: String = String::from_utf8_lossy(named.stdout.as_slice()).trim().to_owned();
    let stranded_refusal: Observed = run_wrapped(&stranded.fixture, stranded.repo.as_path(), &["worktree", "remove", "proj/task"]);
    let stranded_words: String = stderr_of(&stranded_refusal);
    assert_eq!(stranded_refusal.code, Some(1), "the shared suffix is refused on the stranded commit too: {stranded_words}");
    assert!(stranded_words.contains(&commit[..12]), "and names the commit: {stranded_words}");
    assert!(stranded.inside.exists(), "nothing was removed");
    remove(&stranded.fixture);
}

/// What:
///  `--no-force` is a force spelling Git reads as no force,
///  so the guard asks its questions on that form too.
/// Why:
///  Git's `OPT__FORCE` accepts `--force`, `--no-force`, their unique prefixes and a cluster of `f`,
///  and a reader that knew only `-f` and `--force` stood down on the `--no-force` form,
///  which Git removes unforced, so a hidden edit went with it
///  (a review of `9e6c8619a9`, finding 1, and the same class in the words).
#[test]
fn the_forms_git_reads_as_no_force_are_guarded() {
    for words in [["--no-force"].as_slice(), ["-f", "--no-force"].as_slice(), ["--no-forc"].as_slice()] {
        let scene: Scene = a_repository_with_a_recorded_worktree("removal-no-force-form");
        let task: String = scene.task.display().to_string();
        scene.hide_an_edit();
        let mut arguments: Vec<&str> = vec!["worktree", "remove"];
        arguments.extend(words.iter().copied());
        arguments.push(task.as_str());
        let refused: Observed = scene.remove_with(arguments.as_slice());
        let said: String = stderr_of(&refused);
        assert_eq!(refused.code, Some(1), "{words:?} is no force to Git, so the guard asks: {said}");
        assert!(said.contains(TRACKED), "and names the edit: {said}");
        assert!(scene.task.join(TRACKED).exists(), "nothing was removed");
        remove(&scene.fixture);
    }
}

/// What:
///  An option this program does not read is refused rather than handed to Git,
///  and Git's own usage answers `-h`.
/// Why:
///  A word this program does not know could be an option a later Git accepts unforced,
///  and standing down on it would hand the removal to Git without a question:
///  the guard words the refusal instead, with the opt-out that runs Git's own reading.
///  `-h` is Git's usage, which removes nothing, so it stays Git's.
#[test]
fn an_option_this_program_does_not_read_is_refused_and_usage_stays_gits() {
    let scene: Scene = a_repository_with_a_recorded_worktree("removal-unread-option");
    let task: String = scene.task.display().to_string();
    scene.hide_an_edit();
    let refused: Observed = scene.remove_with(&["worktree", "remove", "--quiet", task.as_str()]);
    let said: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "an unread option is refused: {said}");
    assert!(said.contains("--quiet"), "and named: {said}");
    assert!(said.contains("--no-enforce-worktree"), "with the way out: {said}");
    assert!(scene.task.join(TRACKED).exists(), "nothing was removed");
    let usage: Observed = scene.remove_with(&["worktree", "remove", "-h"]);
    assert_eq!(usage.code, Some(0), "Git answers its usage and removes nothing: {}", stderr_of(&usage));
    let words: String = String::from_utf8_lossy(usage.stdout.as_slice()).into_owned();
    assert!(words.contains("usage: git worktree remove"), "and that answer is Git's usage, on standard output: {words}");
    assert!(scene.task.exists(), "and the worktree stays");
    remove(&scene.fixture);
}

/// What one fixture of the spelling sweep holds:
/// a worktree inside the repository,
/// a link that reaches it,
/// and a directory a `..` may cancel.
struct Inside {
    /// The fixture whose wrapper link every command starts.
    fixture: Fixture,
    /// The main checkout.
    repo: PathBuf,
    /// The worktree every control removes,
    /// at `repo/sub/<directory>`.
    task: PathBuf,
    /// The fixture's own root,
    /// where the link and the cancelled directory stand.
    root: PathBuf,
}

/// What:
///  A repository with one recorded worktree at `repo/sub/<directory>`,
///  a link at the fixture's root that names the repository,
///  and a directory at the root that a `..` may cancel.
/// Why:
///  A relative operand has somewhere to be relative to only from inside the worktree's repository,
///  and a `..` that cancels names one only beside a directory that is there.
fn a_worktree_inside_the_repository(name: &str, directory: &str) -> Inside {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join(TRACKED), "base\n").expect("a tracked file");
    git(&fixture, &repo, &["add", TRACKED]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=a tracked file"]);
    let inside: PathBuf = repo.join("sub").join(directory);
    std::fs::create_dir_all(inside.parent().expect("a parent")).expect("the parent directory");
    run_direct(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", BRANCH, inside.to_str().expect("a UTF-8 fixture path")]);
    let task: PathBuf = std::fs::canonicalize(inside.as_path()).expect("the worktree's real path");
    let root: PathBuf = fixture.root.clone();
    std::os::unix::fs::symlink(repo.as_path(), root.join("linked").as_path()).expect("a link to the repository");
    std::fs::create_dir_all(root.join("inner").as_path()).expect("a directory a `..` cancels");
    std::fs::create_dir_all(repo.join("inner").as_path()).expect("a directory a `..` cancels, inside the repository");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("the common Git directory");
    record(common.as_path(), task.as_path()).expect("the record that this program made the worktree");
    return Inside { fixture, repo, task, root };
}

/// What:
///  One spelling of the same worktree,
///  and whether Git itself removes the worktree for it.
/// Why:
///  The sweep states Git's own answer beside each spelling,
///  so a control that reads "kept" cannot be satisfied by a guard that refuses everything.
struct Spelling {
    /// The directory the fixture's worktree has,
    /// which the case and Unicode spellings need.
    directory: &'static str,
    /// Whether `core.ignorecase` must be on for this spelling.
    ignorecase: bool,
    /// Whether Git itself removes the worktree for this spelling,
    /// measured against Git 2.56.0 (`keep/probe-operand-shapes-against-real-git-3.log`).
    git_removes: bool,
    /// The name this control reports.
    name: &'static str,
    /// The operand,
    /// read from the fixture.
    operand: fn(&Inside) -> String,
}

/// What:
///  The last component of the worktree's own path,
///  for the spellings that name the same component as it stands.
fn last_of(fixture: &Inside) -> String {
    let text: String = fixture.task.display().to_string();
    return text[text.rfind('/').expect("a path with a separator") + 1..].to_owned();
}

/// What:
///  The last component of the worktree's own path,
///  in upper case,
///  for the spellings a case-folding repository resolves.
fn upper_last(fixture: &Inside) -> String {
    return last_of(fixture).to_uppercase();
}

/// What:
///  The parent directory of the worktree's own path,
///  for the spellings that add a component before the last one.
fn parent_of(fixture: &Inside) -> String {
    let text: String = fixture.task.display().to_string();
    return text[..text.rfind('/').expect("a path with a separator")].to_owned();
}

/// What:
///  Every spelling of one worktree that Git itself resolves to it keeps a hidden edit,
///  and the clean worktree still goes for every spelling Git removes.
/// Why:
///  Three reviews found a name Git resolves and this program did not:
///  a typed `core.ignorecase`, a suffix two worktrees share, and a case-differing name behind a `.` component or a
///  link (`e615fb599b`, finding 1, which `a_case_differing_name_behind_a_dot_or_a_link_is_guarded` reproduces whole).
///  Each was mended at one more place of the guard's own resolving,
///  so the answer is now Git's own list and Git's own path resolution,
///  and every spelling Git removes for is guarded while the clean worktree of the same spelling still goes.
///  The Unicode spellings are the ones Git on this system refuses under `core.precomposeUnicode`,
///  which folds no form here: they are in the sweep so it says what this program does with them,
///  and Git's own answer is stated beside each spelling
///  (`keep/probe-operand-shapes-against-real-git-3.log`).
#[test]
fn every_spelling_that_names_a_worktree_is_guarded_and_a_clean_one_still_goes() {
    let spellings: [Spelling; 17] = [
        Spelling { directory: "task", ignorecase: false, git_removes: true, name: "relative-name", operand: |_| return String::from("sub/task") },
        Spelling { directory: "task", ignorecase: false, git_removes: true, name: "relative-last-component", operand: |_| return String::from("task") },
        Spelling { directory: "task", ignorecase: false, git_removes: true, name: "relative-dot", operand: |_| return String::from("./sub/task") },
        Spelling { directory: "task", ignorecase: false, git_removes: true, name: "relative-double-separator", operand: |_| return String::from(".//sub/task") },
        Spelling { directory: "task", ignorecase: false, git_removes: true, name: "relative-dotdot", operand: |_| return String::from("sub/../sub/task") },
        Spelling { directory: "task", ignorecase: false, git_removes: true, name: "absolute-exact", operand: |fixture| return fixture.task.display().to_string() },
        Spelling {
            directory: "task",
            ignorecase: false,
            git_removes: true,
            name: "absolute-dot",
            operand: |fixture| return format!("{}/./{}", parent_of(fixture), last_of(fixture)),
        },
        Spelling {
            directory: "task",
            ignorecase: false,
            git_removes: true,
            name: "absolute-dotdot",
            operand: |fixture| return format!("{}/../{}", fixture.task.display(), last_of(fixture)),
        },
        Spelling { directory: "task", ignorecase: false, git_removes: true, name: "absolute-inner-dotdot", operand: |fixture| return format!("{}/inner/../sub/task", fixture.repo.display()) },
        Spelling { directory: "task", ignorecase: false, git_removes: true, name: "through-a-link", operand: |fixture| return format!("{}/linked/sub/task", fixture.root.display()) },
        Spelling { directory: "task", ignorecase: false, git_removes: true, name: "trailing-slash", operand: |fixture| return format!("{}/", fixture.task.display()) },
        Spelling { directory: "task", ignorecase: true, git_removes: true, name: "case-differing-name", operand: |_| return String::from("SUB/TASK") },
        Spelling {
            directory: "task",
            ignorecase: true,
            git_removes: true,
            name: "case-differing-behind-a-dot",
            operand: |fixture| return format!("{}/./{}", parent_of(fixture), upper_last(fixture)),
        },
        Spelling {
            directory: "task",
            ignorecase: true,
            git_removes: true,
            name: "case-differing-behind-a-link",
            operand: |fixture| return format!("{}/linked/sub/{}", fixture.root.display(), upper_last(fixture)),
        },
        Spelling {
            directory: "task",
            ignorecase: true,
            git_removes: false,
            name: "case-differing-with-a-trailing-slash",
            operand: |fixture| return format!("{}/{}/", parent_of(fixture), upper_last(fixture)),
        },
        Spelling { directory: "café", ignorecase: false, git_removes: false, name: "unicode-nfd-against-nfc", operand: |_| return String::from("sub/cafe\u{301}") },
        Spelling { directory: "cafe\u{301}", ignorecase: false, git_removes: false, name: "unicode-nfc-against-nfd", operand: |_| return String::from("sub/café") },
    ];
    for spelling in spellings {
        let hidden: Inside = a_worktree_inside_the_repository(format!("removal-spelling-{}-hidden", spelling.name).as_str(), spelling.directory);
        let clean: Inside = a_worktree_inside_the_repository(format!("removal-spelling-{}-clean", spelling.name).as_str(), spelling.directory);
        for scene in [&hidden, &clean] {
            if spelling.ignorecase {
                run_direct(&scene.fixture, scene.repo.as_path(), &["config", "core.ignorecase", "true"]);
            }
            run_direct(&scene.fixture, scene.repo.as_path(), &["config", "core.precomposeUnicode", "true"]);
        }
        run_direct(&hidden.fixture, hidden.task.as_path(), &["update-index", "--assume-unchanged", TRACKED]);
        std::fs::write(hidden.task.join(TRACKED), "my edit, only here\n").expect("an edit to a tracked file");

        let operand: String = (spelling.operand)(&hidden);
        let refused: Observed = run_wrapped(&hidden.fixture, hidden.repo.as_path(), &["worktree", "remove", operand.as_str()]);
        let said: String = stderr_of(&refused);
        match spelling.git_removes {
            // The spelling names the worktree, so the refusal is the guard's and it names what would go with it.
            true => {
                assert_eq!(refused.code, Some(1), "{}: the guard is active on this spelling: {said}", spelling.name);
                assert!(said.contains(TRACKED), "{}: and the refusal names the hidden edit: {said}", spelling.name);
            }
            // Git itself resolves no worktree from this spelling, and this program refuses it rather than forward a
            // removal on the guess that Git would.
            false => assert_eq!(refused.code, Some(1), "{}: a spelling Git refuses is refused here as well: {said}", spelling.name),
        }
        let kept: String = std::fs::read_to_string(hidden.task.join(TRACKED)).expect("the edit is still there");
        assert_eq!(kept, "my edit, only here\n", "{}: and the edit's own bytes are untouched", spelling.name);

        let clean_operand: String = (spelling.operand)(&clean);
        let removed: Observed = run_wrapped(&clean.fixture, clean.repo.as_path(), &["worktree", "remove", clean_operand.as_str()]);
        let answer: String = stderr_of(&removed);
        match spelling.git_removes {
            true => assert_eq!(removed.code, Some(0), "{}: a clean worktree goes through the same spelling: {answer}", spelling.name),
            // Git itself refuses this spelling on this system, and this program refuses it too rather than forwarding
            // a removal on the guess that Git would.
            false => assert_eq!(removed.code, Some(1), "{}: a spelling Git refuses is refused here as well: {answer}", spelling.name),
        }
        assert_eq!(!clean.task.exists(), spelling.git_removes, "{}: and the worktree ends exactly as Git ends it", spelling.name);
        remove(&hidden.fixture);
        remove(&clean.fixture);
    }
}

/// What:
///  The two shapes a review of `e615fb599b` reproduced destroy a hidden edit through:
///  a redundant `.` component before a case-differing last component,
///  and a prefix spelled through a link,
///  each under `core.ignorecase`,
///  with the same spelling on a clean worktree as the control that the spelling reaches a removal at all.
/// Why:
///  Git resolves those operands with its own `realpath`,
///  which keeps a last component that is not there and follows a link before it,
///  and then compares the answer with every registration folding ASCII case;
///  a guard whose own resolution failed on both shapes named no worktree and stood down,
///  and the wrapper's exit was 0 with the edit gone
///  (`keep/review-case-fold-shapes-at-e615fb599b.log`, and the link shape in
///  `keep/review-case-fold-symlink-confirm-at-e615fb599b.log`).
#[test]
fn a_case_differing_name_behind_a_dot_or_a_link_is_guarded_where_git_resolves_it() {
    for (name, operand_of) in [
        ("removal-case-fold-dot", (|fixture: &Inside| return format!("{}/./{}", parent_of(fixture), upper_last(fixture))) as fn(&Inside) -> String),
        ("removal-case-fold-link", |fixture: &Inside| return format!("{}/linked/sub/{}", fixture.root.display(), upper_last(fixture))),
    ] {
        let hidden: Inside = a_worktree_inside_the_repository(format!("{name}-hidden").as_str(), "task");
        let clean: Inside = a_worktree_inside_the_repository(format!("{name}-clean").as_str(), "task");
        for scene in [&hidden, &clean] {
            run_direct(&scene.fixture, scene.repo.as_path(), &["config", "core.ignorecase", "true"]);
        }
        run_direct(&hidden.fixture, hidden.task.as_path(), &["update-index", "--assume-unchanged", TRACKED]);
        std::fs::write(hidden.task.join(TRACKED), "my edit, only here\n").expect("an edit to a tracked file");
        let operand: String = operand_of(&hidden);
        let refused: Observed = run_wrapped(&hidden.fixture, hidden.repo.as_path(), &["worktree", "remove", operand.as_str()]);
        let said: String = stderr_of(&refused);
        assert_eq!(refused.code, Some(1), "{operand}: the guard is active on this spelling: {said}");
        assert!(said.contains(TRACKED), "{operand}: and names the hidden edit: {said}");
        assert_eq!(
            std::fs::read_to_string(hidden.task.join(TRACKED)).expect("the edit is still there"),
            "my edit, only here\n",
            "{operand}: and the edit is untouched"
        );
        // The control: the same spelling on a worktree with nothing hidden goes through it, so "kept" above is the
        // guard's answer and not a spelling that reaches no removal at all.
        let clean_operand: String = operand_of(&clean);
        let removed: Observed = run_wrapped(&clean.fixture, clean.repo.as_path(), &["worktree", "remove", clean_operand.as_str()]);
        let answer: String = stderr_of(&removed);
        assert_eq!(removed.code, Some(0), "{clean_operand}: a clean worktree goes through this spelling: {answer}");
        assert!(!clean.task.exists(), "{clean_operand}: and it is gone");
        remove(&hidden.fixture);
        remove(&clean.fixture);
    }
}

/// What:
///  A commit made on a detached `HEAD` and then reset away is refused,
///  the reflog that names it survives the refusal,
///  and `git gc --prune=now` leaves the object readable.
/// Why:
///  The reflog is the last thing that names such a commit,
///  and `git worktree remove` deletes it with the worktree's own Git directory:
///  the removal then ends 0, the object becomes unreachable, and a collection takes it
///  (a review of `e615fb599b`, finding 1, whose own script this control follows:
///  `keep/review-reflog-only-detached-commit-at-2596b5d164.log` of this run shows the loss at the tip before it).
///  The collection is run on purpose:
///  a refusal that keeps the worktree but leaves the commit unreachable would pass a control that only looked at
///  the worktree.
///  Every route by which a typed removal goes asks this question, so the plain forward this one takes speaks for the
///  copy route too, whose own control is `a_removal_that_skips_the_copy_is_refused_for_a_stranded_commit_too`.
#[test]
fn a_commit_only_the_head_reflog_names_is_refused_and_survives_a_collection() {
    let scene: Scene = a_repository_with_a_recorded_worktree("removal-reflog-only-detached-commit");
    let task: String = scene.task.display().to_string();
    run_direct(&scene.fixture, scene.task.as_path(), &["checkout", "--quiet", "--detach"]);
    std::fs::write(scene.task.join("only-in-this-commit.txt"), "made here\n").expect("a file in the worktree");
    run_direct(&scene.fixture, scene.task.as_path(), &["add", "only-in-this-commit.txt"]);
    run_direct(&scene.fixture, scene.task.as_path(), &["commit", "--quiet", "--message=work on a detached head"]);
    let named: std::process::Output = git(&scene.fixture, scene.task.as_path(), &["rev-parse", "HEAD"]);
    let commit: String = String::from_utf8_lossy(named.stdout.as_slice()).trim().to_owned();
    // `HEAD` back on a commit that survives, so the reflog is the only thing that names the one above.
    run_direct(&scene.fixture, scene.task.as_path(), &["reset", "--hard", "main"]);
    let refused: Observed = scene.remove_with(&["worktree", "remove", task.as_str()]);
    let said: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "the removal is refused: {said}");
    assert!(said.contains("its HEAD reflog"), "the refusal names the reflog: {said}");
    assert!(said.contains(&commit[..12]), "and the commit it names: {said}");
    assert!(scene.task.exists(), "and the worktree stays");
    // The collection a lost commit would go with, run on the repository the worktree is still registered in.
    run_direct(&scene.fixture, scene.repo.as_path(), &["gc", "--prune=now"]);
    let readable: Observed = run_direct(&scene.fixture, scene.repo.as_path(), &["cat-file", "-e", commit.as_str()]);
    assert_eq!(readable.code, Some(0), "and the commit is still readable after a collection: {}", stderr_of(&readable));
    // The opt-out the refusal printed still runs the removal as real Git runs it, which is where the person decides.
    let printed: String = scene.printed_command(&said);
    let ran: Observed = scene.run_through_a_shell(printed.as_str());
    assert_eq!(ran.code, Some(0), "the printed opt-out runs: {}", stderr_of(&ran));
    assert!(!scene.task.exists(), "and the worktree is gone");
    remove(&scene.fixture);
}

/// What:
///  A commit a worktree's own reference names through an annotated tag is refused,
///  a tag of a tag reaches the same commit,
///  a tag of a tree is kept as it stands,
///  and `git gc --prune=now` leaves each object readable after the refusal.
/// Why:
///  The reference query asked for the object a reference names,
///  which for an annotated tag is the tag object:
///  the commit was dropped as no commit, the removal ended 0, and the collection took the commit
///  (a review of `5a2bd4bb6c`, finding 1).
///  The tag of a tree is the shape that is no commit at all,
///  and the collection is run on purpose in each of the three.
#[test]
fn a_commit_only_an_annotated_tag_names_is_refused_and_survives_a_collection() {
    let scene: Scene = a_repository_with_a_recorded_worktree("removal-annotated-tag");
    let task: String = scene.task.display().to_string();
    run_direct(&scene.fixture, scene.task.as_path(), &["checkout", "--quiet", "--detach"]);
    std::fs::write(scene.task.join("only-in-this-commit.txt"), "made here\n").expect("a file in the worktree");
    run_direct(&scene.fixture, scene.task.as_path(), &["add", "only-in-this-commit.txt"]);
    run_direct(&scene.fixture, scene.task.as_path(), &["commit", "--quiet", "--message=work on a detached head"]);
    let named: std::process::Output = git(&scene.fixture, scene.task.as_path(), &["rev-parse", "HEAD"]);
    let commit: String = String::from_utf8_lossy(named.stdout.as_slice()).trim().to_owned();
    let tree_object: std::process::Output = git(&scene.fixture, scene.task.as_path(), &["rev-parse", &format!("{commit}^{{tree}}")]);
    let tree: String = String::from_utf8_lossy(tree_object.stdout.as_slice()).trim().to_owned();
    run_direct(&scene.fixture, scene.task.as_path(), &["reset", "--hard", "main"]);
    // The reflog and the reset's own record go, so the worktree's reference is the last name the object has.
    run_direct(&scene.fixture, scene.task.as_path(), &["reflog", "expire", "--expire=now", "--all"]);
    let admin: PathBuf = admin_of(&scene.fixture, BRANCH);
    std::fs::remove_file(admin.join("logs/HEAD").as_path()).expect("the reflog goes");
    std::fs::remove_file(admin.join("ORIG_HEAD").as_path()).expect("the reset's record goes");

    for (reference, of, reached, kind) in [
        ("plain-tag", commit.as_str(), commit.as_str(), "commit"),
        ("nested-tag", commit.as_str(), commit.as_str(), "commit"),
        ("tag-of-tree", tree.as_str(), tree.as_str(), "tree"),
    ] {
        // The annotated tag a person made, and for the nested shape the tag of that tag.
        run_direct(&scene.fixture, scene.task.as_path(), &["tag", "--annotate", "--message=inner", "construction-inner", of]);
        let inner_object: std::process::Output = git(&scene.fixture, scene.task.as_path(), &["rev-parse", "refs/tags/construction-inner"]);
        let inner: String = String::from_utf8_lossy(inner_object.stdout.as_slice()).trim().to_owned();
        let pointed: String = match reference {
            "nested-tag" => {
                run_direct(&scene.fixture, scene.task.as_path(), &["tag", "--annotate", "--message=outer", "construction-outer", inner.as_str()]);
                let outer: std::process::Output = git(&scene.fixture, scene.task.as_path(), &["rev-parse", "refs/tags/construction-outer"]);
                String::from_utf8_lossy(outer.stdout.as_slice()).trim().to_owned()
            }
            _ => inner,
        };
        run_direct(&scene.fixture, scene.task.as_path(), &["update-ref", "--no-deref", &format!("refs/worktree/{reference}"), pointed.as_str()]);
        for name in ["construction-inner", "construction-outer"] {
            run_direct(&scene.fixture, scene.task.as_path(), &["update-ref", "-d", &format!("refs/tags/{name}")]);
        }
        let refused: Observed = scene.remove_with(&["worktree", "remove", task.as_str()]);
        let said: String = stderr_of(&refused);
        assert_eq!(refused.code, Some(1), "{reference}: the removal is refused: {said}");
        assert!(said.contains(&format!("refs/worktree/{reference}")), "{reference}: the refusal names it: {said}");
        assert!(said.contains(&reached[..12]), "{reference}: and the {kind} it reaches: {said}");
        assert!(said.contains(&format!("names the {kind}")), "{reference}: as a {kind}: {said}");
        assert!(scene.task.exists(), "{reference}: and the worktree stays");
        // The collection runs in the worktree that holds the name: a reference of one worktree is invisible to a
        // collection started in another, which is Git's own reading of a per-worktree reference
        // (`keep/measurement-what-a-collection-keeps.log` of this run).
        run_direct(&scene.fixture, scene.task.as_path(), &["gc", "--prune=now"]);
        let readable: Observed = run_direct(&scene.fixture, scene.task.as_path(), &["cat-file", "-e", reached]);
        assert_eq!(readable.code, Some(0), "{reference}: the {kind} is still readable after a collection: {}", stderr_of(&readable));
        run_direct(&scene.fixture, scene.task.as_path(), &["update-ref", "-d", &format!("refs/worktree/{reference}")]);
    }
    remove(&scene.fixture);
}

/// What:
///  A record of an operation that is there and cannot be read is refused,
///  the worktree stays, and the commit it holds survives a collection.
/// Why:
///  The two record loops read a record they could not open as no record,
///  and `ORIG_HEAD` was then the only name of a commit nobody asked about:
///  the removal ended 0, the record went with the worktree, and the collection took the commit
///  (a review of `5a2bd4bb6c`, finding 2).
///  The record of an operation that stopped is the same read, so it is the second half of this control.
#[test]
fn a_record_this_program_cannot_read_is_refused_and_its_commit_survives_a_collection() {
    use std::os::unix::fs::PermissionsExt as _;
    for (where_it_stands, holder, clear_orig_head) in
        [("ORIG_HEAD", "the record `ORIG_HEAD`", false), ("rebase-merge/onto", "the record `rebase-merge/onto`", true)]
    {
        let scene: Scene = a_repository_with_a_recorded_worktree("removal-unreadable-record");
        let task: String = scene.task.display().to_string();
        run_direct(&scene.fixture, scene.task.as_path(), &["checkout", "--quiet", "--detach"]);
        std::fs::write(scene.task.join("only-in-this-commit.txt"), "made here\n").expect("a file in the worktree");
        run_direct(&scene.fixture, scene.task.as_path(), &["add", "only-in-this-commit.txt"]);
        run_direct(&scene.fixture, scene.task.as_path(), &["commit", "--quiet", "--message=work on a detached head"]);
        let named: std::process::Output = git(&scene.fixture, scene.task.as_path(), &["rev-parse", "HEAD"]);
        let commit: String = String::from_utf8_lossy(named.stdout.as_slice()).trim().to_owned();
        run_direct(&scene.fixture, scene.task.as_path(), &["reset", "--hard", "main"]);
        run_direct(&scene.fixture, scene.task.as_path(), &["reflog", "expire", "--expire=now", "--all"]);
        let admin: PathBuf = admin_of(&scene.fixture, BRANCH);
        std::fs::remove_file(admin.join("logs/HEAD").as_path()).expect("the reflog goes");
        if clear_orig_head {
            // Only the record of the operation that stopped names the commit, so the refusal can only be about that one.
            std::fs::remove_file(admin.join("ORIG_HEAD").as_path()).ok();
        }
        let record: PathBuf = admin.join(where_it_stands);
        std::fs::create_dir_all(record.parent().expect("a parent")).expect("the record's directory");
        std::fs::write(record.as_path(), format!("{commit}\n")).expect("the record");
        std::fs::set_permissions(record.as_path(), std::fs::Permissions::from_mode(0o000)).expect("a record nothing may read");
        let refused: Observed = scene.remove_with(&["worktree", "remove", task.as_str()]);
        let said: String = stderr_of(&refused);
        assert_eq!(refused.code, Some(1), "{holder}: the removal is refused: {said}");
        assert!(said.contains(holder), "{holder}: the refusal names the record: {said}");
        // The commit itself cannot be named: it is the record that says which one it is, and the record is what could not
        // be read. The refusal says so, which is what lets a person fix the record and ask again.
        assert!(said.contains("cannot be read"), "{holder}: and says the record could not be read: {said}");
        assert!(scene.task.exists(), "{holder}: and the worktree stays");
        // The record the removal would have deleted is still there, holding the commit's name.
        std::fs::set_permissions(record.as_path(), std::fs::Permissions::from_mode(0o644)).expect("readable again");
        let kept: String = std::fs::read_to_string(record.as_path()).expect("the record is still there");
        assert_eq!(kept.trim(), commit, "{holder}: and still names the commit");
        let readable: Observed = run_direct(&scene.fixture, scene.task.as_path(), &["cat-file", "-e", commit.as_str()]);
        assert_eq!(readable.code, Some(0), "{holder}: the commit is there, with the record the removal did not delete: {}", stderr_of(&readable));
        // A record is no root of a collection, so the sentence's remedy is what keeps the commit: name it, then collect.
        run_direct(&scene.fixture, scene.task.as_path(), &["branch", "keep-the-commit", commit.as_str()]);
        run_direct(&scene.fixture, scene.task.as_path(), &["gc", "--prune=now"]);
        let survived: Observed = run_direct(&scene.fixture, scene.task.as_path(), &["cat-file", "-e", commit.as_str()]);
        assert_eq!(survived.code, Some(0), "{holder}: naming it as the refusal says keeps it through a collection: {}", stderr_of(&survived));
        remove(&scene.fixture);
    }
}

/// What:
///  A registration inside the worktree that cannot be read is refused,
///  and the nested worktree's commit and uncommitted bytes stay.
/// Why:
///  The list of registrations the removal takes skipped a `gitdir` it could not read,
///  so the nested worktree was never asked about:
///  the same removal that was refused a moment before then ended 0 and deleted the nested checkout,
///  and pruning and collecting took its commit
///  (a review of `5a2bd4bb6c`, finding 3).
///  The readable registration is the control: the refusal there is the nested detached `HEAD`.
#[test]
fn a_registration_this_program_cannot_read_is_refused_and_the_nested_worktree_stays() {
    use std::os::unix::fs::PermissionsExt as _;
    let scene: Scene = a_repository_with_a_recorded_worktree("removal-unreadable-registration");
    let outer: String = scene.task.display().to_string();
    // The nested checkout is ignored by the branch the outer worktree is on, as the review's own fixture ignores it,
    // so the outer worktree is clean and the removal is Git's to make once the guard has stood aside.
    let ignored: PathBuf = scene.task.join(".gitignore");
    std::fs::write(ignored.as_path(), "/inner/\n").expect("the ignore file");
    run_direct(&scene.fixture, scene.task.as_path(), &["add", ".gitignore"]);
    run_direct(&scene.fixture, scene.task.as_path(), &["commit", "--quiet", "--message=ignore the nested checkout"]);
    let nested: PathBuf = scene.task.join("inner");
    let named: &str = nested.to_str().expect("a UTF-8 fixture path");
    run_direct(&scene.fixture, scene.repo.as_path(), &["worktree", "add", "--quiet", "--detach", named, "main"]);
    std::fs::write(nested.join("committed-only-here.txt"), "committed work\n").expect("a file in the nested worktree");
    run_direct(&scene.fixture, nested.as_path(), &["add", "committed-only-here.txt"]);
    run_direct(&scene.fixture, nested.as_path(), &["commit", "--quiet", "--message=nested work"]);
    let nested_commit_object: std::process::Output = git(&scene.fixture, nested.as_path(), &["rev-parse", "HEAD"]);
    let commit: String = String::from_utf8_lossy(nested_commit_object.stdout.as_slice()).trim().to_owned();
    std::fs::write(nested.join(TRACKED), "uncommitted bytes that exist nowhere else\n").expect("uncommitted bytes");
    let registration: PathBuf = admin_of(&scene.fixture, "inner").join("gitdir");
    // The control: with the registration readable, the nested worktree keeps the outer one.
    let control: Observed = scene.remove_with(&["worktree", "remove", outer.as_str()]);
    let said: String = stderr_of(&control);
    assert_eq!(control.code, Some(1), "a readable registration keeps the outer worktree: {said}");
    assert!(said.contains("detached HEAD"), "and names what holds the nested commit: {said}");
    assert!(nested.join(TRACKED).is_file(), "and the nested bytes stay");
    // The same removal, with that one record unreadable.
    std::fs::set_permissions(registration.as_path(), std::fs::Permissions::from_mode(0o000)).expect("a registration nothing may read");
    let refused_unreadable: Observed = scene.remove_with(&["worktree", "remove", outer.as_str()]);
    let unreadable_words: String = stderr_of(&refused_unreadable);
    assert_eq!(refused_unreadable.code, Some(1), "an unreadable registration is refused: {unreadable_words}");
    assert!(unreadable_words.contains("the registration in"), "the refusal names it: {unreadable_words}");
    assert!(unreadable_words.contains("may name a worktree inside the one being removed"), "and why it stops the removal: {unreadable_words}");
    assert!(nested.join(TRACKED).is_file(), "and the nested bytes stay");
    assert!(nested.join("committed-only-here.txt").is_file(), "and the nested checkout stays");
    std::fs::set_permissions(registration.as_path(), std::fs::Permissions::from_mode(0o644)).expect("readable again");
    // The collection runs in the nested worktree, whose own `HEAD` names the commit.
    run_direct(&scene.fixture, nested.as_path(), &["gc", "--prune=now"]);
    let readable: Observed = run_direct(&scene.fixture, nested.as_path(), &["cat-file", "-e", commit.as_str()]);
    assert_eq!(readable.code, Some(0), "the nested commit is still readable after a collection: {}", stderr_of(&readable));
    remove(&scene.fixture);
}

/// What:
///  Every route by which a removal reaches Git through this program,
///  and which of them the guard judges.
/// Why:
///  A plain alias is expanded by Git itself, so the words are judged like words typed.
///  A `!` alias is run by Git's shell, and the owner's decision of 2026-11-15 forwards every one of them as it is:
///  all aliases keep working with no opt-out, and a removal a shell alias runs is not guarded.
///  `SPEC.md` states that hole.
#[test]
fn every_route_a_removal_reaches_git_by_is_guarded() {
    for (name, aliases, words, guarded) in [
        // A plain alias, and an alias of that alias: Git expands these itself, and the words are judged.
        ("removal-route-plain-alias", vec![("alias.rmwt", "worktree remove")], vec!["rmwt"], true),
        ("removal-route-alias-of-alias", vec![("alias.rmwt", "worktree remove"), ("alias.again", "rmwt")], vec!["again"], true),
        // A `!` alias: forwarded as it is.
        ("removal-route-shell-alias", vec![("alias.rmwt", "!git worktree remove")], vec!["rmwt"], false),
        // The caller's own `-c` before the alias word, which is what the alias is.
        ("removal-route-config-alias", Vec::new(), vec!["-c", "alias.rmwt=worktree remove", "rmwt"], true),
        // The removal's own words after the alias word.
        ("removal-route-after-the-word", vec![("alias.rmwt", "worktree remove")], vec!["rmwt", "--"], true),
    ] {
        let scene: Scene = a_repository_with_a_recorded_worktree(name);
        let task: String = scene.task.display().to_string();
        scene.hide_an_edit();
        for (key, value) in aliases {
            run_direct(&scene.fixture, scene.repo.as_path(), &["config", key, value]);
        }
        // The control: the same removal typed as the built-in, which is refused.
        let typed: Observed = scene.remove_with(&["worktree", "remove", task.as_str()]);
        assert_eq!(typed.code, Some(1), "{name}: the built-in form is refused: {}", stderr_of(&typed));
        let mut arguments: Vec<&str> = words.clone();
        arguments.push(task.as_str());
        let aliased: Observed = scene.remove_with(arguments.as_slice());
        let said: String = stderr_of(&aliased);
        match guarded {
            true => {
                assert_eq!(aliased.code, Some(1), "{name}: the aliased form is judged and refused too: {said}");
                assert!(said.contains(TRACKED), "{name}: and names the hidden edit: {said}");
                assert!(scene.task.join(TRACKED).exists(), "{name}: and nothing was removed");
            }
            // The hole the owner's decision accepts: a shell alias that removes a worktree is not looked at.
            false => {
                assert_eq!(aliased.code, Some(0), "{name}: a shell alias is forwarded as it is: {said}");
                assert!(!scene.task.exists(), "{name}: and the removal it runs is not guarded, so the worktree goes");
            }
        }
        remove(&scene.fixture);
    }
}

/// What:
///  A hidden edit in a worktree registered inside the target keeps the outer worktree,
///  on the route the ladder serves and on the plain forward,
///  and no edit there lets the outer removal go.
/// Why:
///  The removal deletes the inner checkout and its own Git directory with the outer one,
///  and the hidden-edit question was asked of the outer directory alone:
///  an outer removal exited 0 and deleted the inner worktree's edited file
///  (a review of `8858f843ee`, finding 2).
///  The commit question already walked the worktrees inside the target;
///  now one list of what a removal takes is made once and every question goes over it
///  (`worktree_removal_worktrees_it_takes`).
#[test]
fn a_hidden_edit_in_a_worktree_inside_keeps_the_outer_one_on_every_route() {
    for (name, words) in [
        ("removal-nested-hidden-edit-ladder", vec!["worktree", "remove"]),
        ("removal-nested-hidden-edit-forward", vec!["--no-worktree-copy", "worktree", "remove"]),
    ] {
        let scene: Scene = a_repository_with_a_recorded_worktree(name);
        let outer: String = scene.task.display().to_string();
        // The nested checkout is ignored on the branch the outer worktree is on, so the outer worktree is clean.
        std::fs::write(scene.task.join(".gitignore"), "/inner/\n").expect("the ignore file");
        run_direct(&scene.fixture, scene.task.as_path(), &["add", ".gitignore"]);
        run_direct(&scene.fixture, scene.task.as_path(), &["commit", "--quiet", "--message=ignore the nested checkout"]);
        let nested: PathBuf = scene.task.join("inner");
        let named: &str = nested.to_str().expect("a UTF-8 fixture path");
        run_direct(&scene.fixture, scene.repo.as_path(), &["worktree", "add", "--quiet", "-b", "inner", named, "main"]);
        // The control: the same edit in the outer worktree keeps it, so the question is alive on this route.
        run_direct(&scene.fixture, scene.task.as_path(), &["update-index", "--assume-unchanged", TRACKED]);
        std::fs::write(scene.task.join(TRACKED), "outer edit\n").expect("an outer edit");
        let mut control: Vec<&str> = words.clone();
        control.push(outer.as_str());
        let refused_outer: Observed = scene.remove_with(control.as_slice());
        assert_eq!(refused_outer.code, Some(1), "{name}: an edit in the outer worktree is refused: {}", stderr_of(&refused_outer));
        // The same, with the edit only in the nested worktree.
        run_direct(&scene.fixture, scene.task.as_path(), &["update-index", "--no-assume-unchanged", TRACKED]);
        std::fs::write(scene.task.join(TRACKED), "base\n").expect("the outer file back");
        run_direct(&scene.fixture, nested.as_path(), &["update-index", "--assume-unchanged", TRACKED]);
        std::fs::write(nested.join(TRACKED), "inner edit\n").expect("an edit in the nested worktree");
        let mut arguments: Vec<&str> = words.clone();
        arguments.push(outer.as_str());
        let refused_nested: Observed = scene.remove_with(arguments.as_slice());
        let said: String = stderr_of(&refused_nested);
        assert_eq!(refused_nested.code, Some(1), "{name}: an edit in the worktree inside is refused: {said}");
        assert!(said.contains(&nested.display().to_string()), "{name}: and the refusal names that worktree: {said}");
        assert_eq!(std::fs::read_to_string(nested.join(TRACKED)).expect("the inner edit is there"), "inner edit\n", "{name}: and the edit stays");
        // The control: with no edit there, the outer removal goes.
        run_direct(&scene.fixture, nested.as_path(), &["update-index", "--no-assume-unchanged", TRACKED]);
        std::fs::write(nested.join(TRACKED), "base\n").expect("the inner file back");
        let mut allowed: Vec<&str> = words.clone();
        allowed.push(outer.as_str());
        let removed: Observed = scene.remove_with(allowed.as_slice());
        assert_eq!(removed.code, Some(0), "{name}: no edit there lets the outer removal go: {}", stderr_of(&removed));
        assert!(!scene.task.exists(), "{name}: and both worktrees are gone");
        remove(&scene.fixture);
    }
}

/// What:
///  A commit only a worktree inside the target names keeps the outer worktree,
///  on both routes.
/// Why:
///  The commit question already walked the worktrees inside the target,
///  and this is the control that says so on the route the ladder serves and on the plain forward,
///  so the list of what a removal takes is checked against both.
#[test]
fn a_commit_only_a_worktree_inside_names_keeps_the_outer_one_on_every_route() {
    for (name, words) in [
        ("removal-nested-commit-ladder", vec!["worktree", "remove"]),
        ("removal-nested-commit-forward", vec!["--no-worktree-copy", "worktree", "remove"]),
    ] {
        let scene: Scene = a_repository_with_a_recorded_worktree(name);
        let outer: String = scene.task.display().to_string();
        std::fs::write(scene.task.join(".gitignore"), "/inner/\n").expect("the ignore file");
        run_direct(&scene.fixture, scene.task.as_path(), &["add", ".gitignore"]);
        run_direct(&scene.fixture, scene.task.as_path(), &["commit", "--quiet", "--message=ignore the nested checkout"]);
        let nested: PathBuf = scene.task.join("inner");
        let named: &str = nested.to_str().expect("a UTF-8 fixture path");
        run_direct(&scene.fixture, scene.repo.as_path(), &["worktree", "add", "--quiet", "--detach", named, "main"]);
        std::fs::write(nested.join("only-here.txt"), "made here\n").expect("a file in the nested worktree");
        run_direct(&scene.fixture, nested.as_path(), &["add", "only-here.txt"]);
        run_direct(&scene.fixture, nested.as_path(), &["commit", "--quiet", "--message=work on a detached head"]);
        let nested_commit: std::process::Output = git(&scene.fixture, nested.as_path(), &["rev-parse", "HEAD"]);
        let commit: String = String::from_utf8_lossy(nested_commit.stdout.as_slice()).trim().to_owned();
        let mut arguments: Vec<&str> = words.clone();
        arguments.push(outer.as_str());
        let refused: Observed = scene.remove_with(arguments.as_slice());
        let said: String = stderr_of(&refused);
        assert_eq!(refused.code, Some(1), "{name}: a commit in the worktree inside is refused: {said}");
        assert!(said.contains(&commit[..12]), "{name}: and names the commit: {said}");
        assert!(nested.exists(), "{name}: and the worktree inside stays");
        remove(&scene.fixture);
    }
}

/// What:
///  A detached `HEAD` written in capital hexadecimal is refused like the small one,
///  and the commit survives a collection in that worktree;
///  a record written in capitals is no different.
/// Why:
///  `git rev-list` prints names in small letters and the filter compared them as text,
///  so a valid `HEAD` in capitals was dropped from the stranded set: the removal ended 0 and a collection took the
///  commit
///  (a review of `8858f843ee`, finding 3).
///  Git reads a name in either case as the same object, and so does every comparison here now.
#[test]
fn an_uppercase_object_name_is_the_same_object_as_the_small_one() {
    for (name, where_it_stands) in [("removal-uppercase-head", "HEAD"), ("removal-uppercase-record", "ORIG_HEAD")] {
        let scene: Scene = a_repository_with_a_recorded_worktree(name);
        let task: String = scene.task.display().to_string();
        run_direct(&scene.fixture, scene.task.as_path(), &["checkout", "--quiet", "--detach"]);
        std::fs::write(scene.task.join("only-here.txt"), "made here\n").expect("a file in the worktree");
        run_direct(&scene.fixture, scene.task.as_path(), &["add", "only-here.txt"]);
        run_direct(&scene.fixture, scene.task.as_path(), &["commit", "--quiet", "--message=work on a detached head"]);
        let named: std::process::Output = git(&scene.fixture, scene.task.as_path(), &["rev-parse", "HEAD"]);
        let commit: String = String::from_utf8_lossy(named.stdout.as_slice()).trim().to_owned();
        assert!(commit.chars().any(|letter| return letter.is_ascii_alphabetic()), "the commit has letters to capitalise");
        if where_it_stands == "HEAD" {
            run_direct(&scene.fixture, scene.task.as_path(), &["reflog", "expire", "--expire=now", "--all"]);
            let admin: PathBuf = admin_of(&scene.fixture, BRANCH);
            let head: PathBuf = admin.join("HEAD");
            std::fs::write(head.as_path(), format!("{}\n", commit.to_uppercase())).expect("a capital HEAD");
            run_direct(&scene.fixture, scene.task.as_path(), &["rev-parse", "--verify", "HEAD"]);
        } else {
            // The reflog and the detached `HEAD` go, and the reset's own record, written in capitals, is the only name.
            git(&scene.fixture, scene.task.as_path(), &["reset", "--hard", "main"]);
            run_direct(&scene.fixture, scene.task.as_path(), &["reflog", "expire", "--expire=now", "--all"]);
            let admin: PathBuf = admin_of(&scene.fixture, BRANCH);
            std::fs::remove_file(admin.join("logs/HEAD").as_path()).expect("the reflog goes");
            std::fs::write(admin.join("ORIG_HEAD").as_path(), format!("{}\n", commit.to_uppercase())).expect("a capital record");
        }
        let refused: Observed = scene.remove_with(&["worktree", "remove", task.as_str()]);
        let said: String = stderr_of(&refused);
        assert_eq!(refused.code, Some(1), "{name}: the capital name is refused: {said}");
        assert!(said.contains(&commit[..12]), "{name}: and the name it keeps is the small one Git prints: {said}");
        assert!(scene.task.exists(), "{name}: and the worktree stays");
        // A detached `HEAD` is a root of a collection run in its worktree, so the collection keeps the commit there.
        // A record is no root of any collection, so the record arm first names the commit as the refusal's sentence says,
        // which is what a person does next, and then collects.
        if where_it_stands != "HEAD" {
            // The remedy the sentence prints: `git -C <the worktree> branch <a name you choose> <the commit>`.
            run_direct(&scene.fixture, scene.task.as_path(), &["branch", "keep-the-commit", commit.as_str()]);
        }
        run_direct(&scene.fixture, scene.task.as_path(), &["gc", "--prune=now"]);
        let readable: Observed = run_direct(&scene.fixture, scene.task.as_path(), &["cat-file", "-e", commit.as_str()]);
        assert_eq!(readable.code, Some(0), "{name}: and the commit is still readable after a collection: {}", stderr_of(&readable));
        remove(&scene.fixture);
    }
}

/// What:
///  A relative `-C` removes what real Git removes,
///  and still guards,
///  with a `-c` of the caller's read by the guard's own question.
/// Why:
///  The guard's own Git ran from the common Git directory and forwarded the caller's relative `-C` again,
///  so its query failed and the person's valid removal was refused where real Git removes the worktree
///  (a review of `8858f843ee`, finding 4).
///  The guard now carries the caller's options that change what Git answers and none that move it.
#[test]
fn a_relative_c_removes_like_real_git_and_still_guards() {
    let scene: Scene = a_repository_with_a_recorded_worktree("removal-relative-c");
    let task: String = scene.task.display().to_string();
    // Run from the directory the relative `-C` names its repository from, which is the fixture's own root.
    let parent: PathBuf = scene.fixture.root.clone();
    // The guard is alive from here: an edit keeps the worktree even with the relative `-C`.
    scene.hide_an_edit();
    let refused: Observed = run_wrapped(&scene.fixture, parent.as_path(), &["-C", "repo", "worktree", "remove", task.as_str()]);
    let said: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "an edit keeps the worktree through a relative -C: {said}");
    assert!(said.contains(TRACKED), "and the refusal names it: {said}");
    // The caller's own `-c` reaches the guard's question: the same name in another case is the same worktree.
    run_direct(&scene.fixture, scene.task.as_path(), &["update-index", "--no-assume-unchanged", TRACKED]);
    std::fs::write(scene.task.join(TRACKED), "base\n").expect("the file back");
    run_direct(&scene.fixture, scene.repo.as_path(), &["config", "core.ignorecase", "true"]);
    scene.hide_an_edit();
    let differing: String = format!("{}/{}", task.rsplit_once('/').expect("a parent").0, "TASK");
    let folded: Observed = run_wrapped(&scene.fixture, parent.as_path(), &["-C", "repo", "worktree", "remove", differing.as_str()]);
    assert_eq!(folded.code, Some(1), "a relative -C and a case-differing name are refused: {}", stderr_of(&folded));
    run_direct(&scene.fixture, scene.task.as_path(), &["update-index", "--no-assume-unchanged", TRACKED]);
    std::fs::write(scene.task.join(TRACKED), "base\n").expect("the file back");
    // A clean worktree goes through the same relative `-C`, as real Git removes it.
    let removed: Observed = run_wrapped(&scene.fixture, parent.as_path(), &["-C", "repo", "worktree", "remove", task.as_str()]);
    assert_eq!(removed.code, Some(0), "a clean worktree goes through a relative -C: {}", stderr_of(&removed));
    assert!(!scene.task.exists(), "and the worktree is gone");
    remove(&scene.fixture);
}

/// What:
///  Every `!` alias is forwarded as it is, and a removal one of them runs is not guarded.
/// Why:
///  The owner's decision of 2026-11-15: "Forward them unguarded".
///  Every `!` alias is passed to real Git as it is, all aliases keep working with no opt-out, and a `git <alias>` that
///  ends in a worktree removal is not looked at.
///  A reader of a shell's text cannot follow it to the end, and nothing of this program stands inside Git's shell
///  (`doc/planning/cli-git-no-self-invocation.md`).
#[test]
fn every_shell_alias_is_forwarded_and_a_removal_it_runs_is_not_guarded() {
    for (name, value, removes) in [
        ("removal-alias-plain", "!git worktree remove", true),
        ("removal-alias-inner-c-directory", "!git -C {repo} worktree remove", true),
        ("removal-alias-inner-c-config", "!git -c core.ignorecase=false worktree remove", true),
        ("removal-alias-second-alias", "!git rwt-inside", true),
        ("removal-alias-pipe", "!git worktree remove \"$1\" | cat", true),
        ("removal-alias-two-commands", "!true; git worktree remove", true),
        ("removal-alias-substitution", "!git worktree remove $(printf %s)", true),
        ("removal-alias-variable", "!action=remove; git worktree $action", true),
        ("removal-alias-function", "!f() { git worktree remove \"$@\"; }; f", true),
        ("removal-alias-eval", "!eval \"git worktree remove\"", true),
        ("removal-alias-env", "!env git worktree remove", true),
        // An alias that runs no Git at all keeps working too, which is the whole point of the decision.
        ("removal-alias-echo", "!echo hello", false),
    ] {
        let scene: Scene = a_repository_with_a_recorded_worktree(name);
        let task: String = scene.task.display().to_string();
        scene.hide_an_edit();
        run_direct(&scene.fixture, scene.repo.as_path(), &["config", "alias.rwt-inside", "worktree remove"]);
        let spelled: String = value.replace("{repo}", scene.repo.to_str().expect("a UTF-8 fixture path").as_ref());
        run_direct(&scene.fixture, scene.repo.as_path(), &["config", "alias.rmwt", spelled.as_str()]);
        let aliased: Observed = scene.remove_with(&["rmwt", task.as_str()]);
        // What a shell's own command ends with is the shell's business (a `| cat` of a removed directory ends 1); what
        // matters is that nothing of this program speaks about the alias.
        assert!(
            !stderr_of(&aliased).contains("monochromatic-git-policy-cli:"),
            "{name}: the alias is forwarded and nothing of this program speaks about it: {}",
            stderr_of(&aliased)
        );
        match removes {
            true => assert!(!scene.task.exists(), "{name}: a removal the alias runs is not guarded, so the worktree goes"),
            false => assert!(scene.task.exists(), "{name}: and an alias that removes nothing removes nothing"),
        }
        remove(&scene.fixture);
    }
}

/// What:
///  A nested worktree whose name is not UTF-8 keeps the outer worktree,
///  a UTF-8 one keeps it too,
///  and with the nested checkout gone the outer removal goes.
/// Why:
///  The registration's `gitdir` bytes were decoded lossily, the path with the replacement character did not exist, and
///  the live registration was read as one Git had pruned: the outer removal then deleted the nested checkout and the
///  only name of its detached commit
///  (a review of `dfd6fab13e`, finding 2).
#[test]
fn a_nested_worktree_whose_name_is_not_utf8_keeps_the_outer_worktree() {
    use std::os::unix::ffi::OsStringExt as _;
    for (name, nested_name) in [
        ("removal-non-utf8-nested", b"inner-\xff".to_vec()),
        ("removal-utf8-nested-control", b"inner-plain".to_vec()),
    ] {
        let scene: Scene = a_repository_with_a_recorded_worktree(name);
        let outer: String = scene.task.display().to_string();
        std::fs::write(scene.task.join(".gitignore"), "/inner*\n").expect("the ignore file");
        run_direct(&scene.fixture, scene.task.as_path(), &["add", ".gitignore"]);
        run_direct(&scene.fixture, scene.task.as_path(), &["commit", "--quiet", "--message=ignore the nested checkouts"]);
        let nested: PathBuf = scene.task.join(std::ffi::OsString::from_vec(nested_name));
        // Real Git registers the checkout, whose own name carries the byte, so the record holds it too.
        let mut add: Command = Command::new("/usr/bin/git");
        add.arg("worktree").arg("add").arg("--quiet").arg("--detach").arg(nested.as_os_str()).arg("main");
        hermetic(&mut add, &scene.fixture, wrapper_first_path(&scene.fixture).as_os_str());
        add.current_dir(scene.repo.as_path());
        let added = add.output().expect("Git starts");
        assert!(added.status.success(), "{name}: the nested worktree is registered: {}", String::from_utf8_lossy(added.stderr.as_slice()));
        std::fs::write(nested.join("only-here.txt"), "made here\n").expect("a file in the nested worktree");
        run_direct(&scene.fixture, nested.as_path(), &["add", "only-here.txt"]);
        run_direct(&scene.fixture, nested.as_path(), &["commit", "--quiet", "--message=work on a detached head"]);
        let named: std::process::Output = git(&scene.fixture, nested.as_path(), &["rev-parse", "HEAD"]);
        let commit: String = String::from_utf8_lossy(named.stdout.as_slice()).trim().to_owned();
        let refused: Observed = scene.remove_with(&["--no-worktree-copy", "worktree", "remove", outer.as_str()]);
        let said: String = stderr_of(&refused);
        assert_eq!(refused.code, Some(1), "{name}: the nested worktree keeps the outer one: {said}");
        assert!(said.contains(&commit[..12]), "{name}: and its commit is named: {said}");
        assert!(nested.exists(), "{name}: and its checkout stays");
        // The control: that nested checkout gone, nothing of its own is left inside, and the outer removal goes.
        std::fs::remove_dir_all(nested.as_path()).expect("the nested checkout goes");
        run_direct(&scene.fixture, scene.repo.as_path(), &["worktree", "prune", "--expire=now"]);
        let removed: Observed = scene.remove_with(&["--no-worktree-copy", "worktree", "remove", outer.as_str()]);
        assert_eq!(removed.code, Some(0), "{name}: with nothing of its own left inside, the outer removal goes: {}", stderr_of(&removed));
        assert!(!scene.task.exists(), "{name}: and the outer checkout is gone");
        remove(&scene.fixture);
    }
}

/// What:
///  A commit only the worktree's own `FETCH_HEAD` names keeps the worktree,
///  the refusal says `FETCH_HEAD` names it,
///  and a plain fetch leaves a commit a reference also names, which the removal may take.
/// Why:
///  `FETCH_HEAD` was left out of the naming records on the reasoning that what it names came from elsewhere,
///  and a commit made here and fetched by its object name is named by nothing else:
///  the removal ended 0 and a collection took it
///  (a review of `dfd6fab13e`, finding 3).
///  The project's rule decides the case: a record that holds the only name of a commit keeps the worktree.
#[test]
fn a_commit_only_fetch_head_names_keeps_the_worktree() {
    let scene: Scene = a_repository_with_a_recorded_worktree("removal-fetch-head-only");
    let task: String = scene.task.display().to_string();
    let tree_object: std::process::Output = git(&scene.fixture, scene.task.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let tree: String = String::from_utf8_lossy(tree_object.stdout.as_slice()).trim().to_owned();
    let made: std::process::Output = git(&scene.fixture, scene.task.as_path(), &["commit-tree", tree.as_str(), "-p", "HEAD", "-m", "a commit only a fetch names"]);
    let commit: String = String::from_utf8_lossy(made.stdout.as_slice()).trim().to_owned();
    run_direct(&scene.fixture, scene.task.as_path(), &["fetch", "--quiet", ".", commit.as_str()]);
    let refused: Observed = scene.remove_with(&["worktree", "remove", task.as_str()]);
    let said: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "a commit only FETCH_HEAD names is refused: {said}");
    assert!(said.contains("FETCH_HEAD"), "the refusal says FETCH_HEAD names it: {said}");
    assert!(said.contains(&commit[..12]), "and names the commit: {said}");
    assert!(scene.task.exists(), "and the worktree stays");
    // A record is no root of a collection, so the sentence's remedy comes first, and then the object must survive one.
    run_direct(&scene.fixture, scene.task.as_path(), &["branch", "keep-the-commit", commit.as_str()]);
    run_direct(&scene.fixture, scene.task.as_path(), &["gc", "--prune=now"]);
    let readable: Observed = run_direct(&scene.fixture, scene.task.as_path(), &["cat-file", "-e", commit.as_str()]);
    assert_eq!(readable.code, Some(0), "and the commit survives a collection once named: {}", stderr_of(&readable));
    remove(&scene.fixture);

    // The control: a plain fetch leaves the commit named by a reference too, and the removal goes.
    let fetched: Scene = a_repository_with_a_recorded_worktree("removal-fetch-head-ref");
    let upstream: PathBuf = fetched.fixture.root.join("upstream");
    std::fs::create_dir_all(upstream.as_path()).expect("the upstream directory");
    run_direct(&fetched.fixture, upstream.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(upstream.join("upstream.txt"), "upstream work\n").expect("a file upstream");
    run_direct(&fetched.fixture, upstream.as_path(), &["add", "upstream.txt"]);
    run_direct(&fetched.fixture, upstream.as_path(), &["commit", "--quiet", "--message=upstream work"]);
    run_direct(&fetched.fixture, fetched.repo.as_path(), &["remote", "add", "origin", upstream.to_str().expect("a UTF-8 fixture path")]);
    run_direct(&fetched.fixture, fetched.task.as_path(), &["fetch", "--quiet", "origin"]);
    let fetched_task: String = fetched.task.display().to_string();
    let removed: Observed = fetched.remove_with(&["worktree", "remove", fetched_task.as_str()]);
    assert_eq!(removed.code, Some(0), "a plain fetch leaves a reference naming what it fetched: {}", stderr_of(&removed));
    assert!(!fetched.task.exists(), "so the removal goes");
    remove(&fetched.fixture);
}

/// What:
///  A repository inside an ignored folder is deleted with the worktree, and the removal's line names it.
/// Why:
///  The owner's decisions of 2026-11-12 ("Delete ignored, spare busy") and of 2026-11-15 ("Delete as ignored"):
///  the nested repository goes with the worktree and the deletion is logged.
///  The line that names it was there before this round looked for such a repository, because the removal already names
///  the ignored directories and the repositories inside them for its own report.
#[test]
fn a_repository_inside_an_ignored_folder_is_deleted_with_the_log_line_that_says_so() {
    let scene: Scene = a_repository_with_a_recorded_worktree("removal-ignored-repository");
    let outer: String = scene.task.display().to_string();
    std::fs::write(scene.task.join(".gitignore"), "/inner/\n/deps/\n".as_bytes()).expect("the ignore file");
    run_direct(&scene.fixture, scene.task.as_path(), &["add", ".gitignore"]);
    run_direct(&scene.fixture, scene.task.as_path(), &["commit", "--quiet", "--message=ignore the nested folders"]);
    let inner: PathBuf = scene.task.join("inner");
    std::fs::create_dir_all(inner.as_path()).expect("the nested folder");
    run_direct(&scene.fixture, inner.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(inner.join("only-here.txt"), "made here\n").expect("a file in the nested repository");
    run_direct(&scene.fixture, inner.as_path(), &["add", "only-here.txt"]);
    run_direct(&scene.fixture, inner.as_path(), &["commit", "--quiet", "--message=the only branch's commit"]);
    let deps: PathBuf = scene.task.join("deps");
    std::fs::create_dir_all(deps.join("lib").as_path()).expect("an ignored folder with no repository");
    for index in 0..200 {
        std::fs::write(deps.join(format!("lib/file-{index}.txt")), b"dependency\n").expect("a file of the ignored folder");
    }
    let removed: Observed = scene.remove_with(&["worktree", "remove", outer.as_str()]);
    let said: String = stderr_of(&removed);
    assert_eq!(removed.code, Some(0), "an ignored folder holding a repository is deleted as ignored: {said}");
    assert!(!scene.task.exists(), "and the worktree goes");
    assert!(!inner.exists(), "and the repository inside it goes with it");
    // The line that names a repository that went is the removal report's own, on the automatic route: the control of its
    // words is `worktree_removal_takes_tests::a_git_repository_inside_an_ignored_directory_is_named_with_its_branch_tips`,
    // which reads the line without any walk this round added.
    remove(&scene.fixture);
}
