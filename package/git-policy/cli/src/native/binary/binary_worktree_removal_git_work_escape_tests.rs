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
///  Without `core.ignorecase`, Git itself refuses the case-differing name,
///  and the guard is not what stops it.
/// Why:
///  The guard stands down exactly where Git names no worktree,
///  and this is the state that shows the stand-down is Git's refusal rather than a hole.
#[test]
fn without_ignorecase_git_itself_refuses_the_case_differing_name() {
    let scene: Scene = a_repository_with_a_recorded_worktree("removal-ignorecase-off");
    scene.hide_an_edit();
    run_direct(&scene.fixture, scene.repo.as_path(), &["config", "core.ignorecase", "false"]);
    let refused: Observed = scene.remove_with(&["worktree", "remove", "TASK"]);
    let said: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(128), "Git answers that TASK is not a working tree: {said}");
    assert!(scene.task.join(TRACKED).exists(), "and nothing was removed");
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
    let refused: Observed = observe(wrapped_within(&scene.fixture, "30").current_dir(scene.repo.as_path()).args(["worktree", "remove", task.as_str()]).env("CLI_GIT_TEST_ONLY_JOURNAL_SOCKET", "@cli-git-review-journal-nobody"), b"");
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
    let scene: SharedSuffix = a_repository_whose_worktrees_share_a_suffix("removal-ambiguous-stranded");
    std::fs::write(scene.inside.join("more.txt"), "more work\n").expect("a file in the worktree");
    run_direct(&scene.fixture, scene.inside.as_path(), &["checkout", "--quiet", "--detach"]);
    run_direct(&scene.fixture, scene.inside.as_path(), &["add", "more.txt"]);
    run_direct(&scene.fixture, scene.inside.as_path(), &["commit", "--quiet", "--message=work on a detached head"]);
    let named: std::process::Output = git(&scene.fixture, scene.inside.as_path(), &["rev-parse", "HEAD"]);
    let commit: String = String::from_utf8_lossy(named.stdout.as_slice()).trim().to_owned();
    let refused: Observed = run_wrapped(&scene.fixture, scene.repo.as_path(), &["worktree", "remove", "proj/task"]);
    let said: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "the shared suffix is refused on the stranded commit too: {said}");
    assert!(said.contains(&commit[..12]), "and names the commit: {said}");
    assert!(scene.inside.exists(), "nothing was removed");
    remove(&scene.fixture);
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
