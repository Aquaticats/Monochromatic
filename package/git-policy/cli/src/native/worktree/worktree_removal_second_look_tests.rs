//! What:
//!  Controls of the proof a removal makes again immediately before it destroys anything.
//! Why:
//!   Every reading the proof takes must be shown to keep a worktree when it fails,
//!  and the reading that lets the removal go must be shown to exist,
//!  or a proof that always answered "keep" would pass every control but the last one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const why = whyTheWorktreeStays(look, worktree, admin); expect(why).toContain('process 4242 is working in it');
//! ```

use super::{SecondLook, why_the_worktree_stays};
use crate::test_support::{age, fixture, git, git_context, repository};
use std::path::{Path, PathBuf};

/// A repository and one linked worktree of it,
/// with the ignore rules a control writes in its first commit.
struct Fixture {
    /// The fixture directory that holds both.
    root: PathBuf,
    /// The main checkout,
    /// which is also the worktree the ignored files are compared with.
    main: PathBuf,
    /// The common Git directory.
    common: PathBuf,
    /// The linked worktree.
    worktree: PathBuf,
}

impl Fixture {
    /// The registration of the linked worktree.
    fn admin(&self) -> PathBuf {
        return self.common.join("worktrees").join("topic");
    }
}

/// What:
///  The repository every control here works on.
/// Why:
///  The proof reads the registrations,
///  the worktree's own Git directory and its ignored files,
///  so a control needs all three in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fixture = aRepositoryWithAWorktree('.claude/\n*.log\n');
/// ```
fn a_repository_with_a_worktree(name: &str, rules: &str) -> Fixture {
    let root: PathBuf = fixture(name);
    let main: PathBuf = repository(root.as_path(), "main");
    let tracked: PathBuf = main.join("README.md");
    std::fs::write(tracked.as_path(), b"# fixture\n").expect("a tracked file");
    let ignored: PathBuf = main.join(".gitignore");
    std::fs::write(ignored.as_path(), rules.as_bytes()).expect("the ignore rules");
    age(tracked.as_path());
    age(ignored.as_path());
    git(main.as_path(), &["add", "--", "README.md", ".gitignore"]);
    git(main.as_path(), &["commit", "--quiet", "--message=a tracked file"]);
    let worktree: PathBuf = root.join("worktree");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "feature", worktree.to_str().expect("a UTF-8 fixture path")]);
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("the common Git directory");
    return Fixture { root, main, common, worktree: std::fs::canonicalize(worktree.as_path()).expect("the worktree's real path") };
}

/// What:
///  A process table holding one process whose working folder is `working`,
///  and the machine's uptime beside it.
/// Why:
///  The proof reads a process table the caller names,
///  so a control gives its own and its answer never depends on the machine's programs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const table = aTableWithOneProcess(root, 4242, worktree);
/// ```
fn a_table_with_one_process(root: &Path, pid: u32, working: &Path) -> PathBuf {
    let table: PathBuf = root.join("proc");
    let entry: PathBuf = table.join(pid.to_string());
    std::fs::create_dir_all(entry.as_path()).expect("a process entry");
    std::os::unix::fs::symlink(working, entry.join("cwd")).expect("the working folder of the process");
    std::fs::write(table.join("uptime"), b"1000.00 0.00\n").expect("the machine's uptime");
    let mut fields: Vec<String> = vec![String::from("R")];
    for _unused in 0..18 {
        fields.push(String::from("0"));
    }
    fields.push(String::from("100"));
    std::fs::write(entry.join("stat"), format!("{pid} (a helper) {}\n", fields.join(" "))).expect("the process's own row");
    return table;
}

/// What:
///  A proof nobody stands in the way of answers nothing,
///  which is what lets a removal go.
/// Why:
///   Every other control here proves a worktree stays,
///  and a proof that answered "keep" for ever would pass all of them:
///  this is the one that says the question can be answered "go".
#[test]
fn nothing_in_the_way_of_a_clean_worktree_is_no_reason_to_keep_it() {
    let fixture: Fixture = a_repository_with_a_worktree("second-look-clean", ".claude/\n");
    let context: crate::transaction_git::GitContext = git_context();
    let empty: PathBuf = fixture.root.join("proc");
    std::fs::create_dir(empty.as_path()).expect("an empty process table");
    let look: SecondLook<'_> = SecondLook { context: &context, common_dir: fixture.common.as_path(), proc_root: empty.as_path(), source: Some(fixture.main.as_path()) };
    assert_eq!(why_the_worktree_stays(&look, fixture.worktree.as_path(), fixture.admin().as_path()), None, "a clean worktree with nothing inside it may go");
}

/// What:
///  A worktree a process is working in stays,
///  and the reason names the process.
/// Why:
///   A process whose working folder is the worktree is a person's shell or a build,
///  and removing it pulls the files out from under them.
///  The service reads the machine's process table,
///  which is the same reading this control makes with a table of its own.
#[test]
fn a_worktree_a_process_is_working_in_stays_on_the_second_look() {
    let fixture: Fixture = a_repository_with_a_worktree("second-look-busy", ".claude/\n");
    let context: crate::transaction_git::GitContext = git_context();
    let table: PathBuf = a_table_with_one_process(fixture.root.as_path(), 4242, fixture.worktree.as_path());
    let look: SecondLook<'_> = SecondLook { context: &context, common_dir: fixture.common.as_path(), proc_root: table.as_path(), source: Some(fixture.main.as_path()) };
    let why: Option<String> = why_the_worktree_stays(&look, fixture.worktree.as_path(), fixture.admin().as_path());
    assert_eq!(why, Some(String::from("process 4242 is working in it")));
}

/// What:
///  A worktree holding another registered worktree inside it stays,
///  and the reason names the nested one.
/// Why:
///  The nested checkout is a directory of the outer worktree,
///  usually under a name Git ignores,
///  so a removal of the outer one takes the nested worktree's files and its own record with it.
#[test]
fn a_worktree_another_worktree_was_added_inside_stays_on_the_second_look() {
    let fixture: Fixture = a_repository_with_a_worktree("second-look-nested", ".claude/\n");
    let inside: PathBuf = fixture.worktree.join(".claude/worktrees/b");
    git(fixture.worktree.as_path(), &["worktree", "add", "--quiet", "-b", "sub", inside.to_str().expect("a UTF-8 fixture path")]);
    let context: crate::transaction_git::GitContext = git_context();
    let empty: PathBuf = fixture.root.join("proc");
    std::fs::create_dir(empty.as_path()).expect("an empty process table");
    let look: SecondLook<'_> = SecondLook { context: &context, common_dir: fixture.common.as_path(), proc_root: empty.as_path(), source: Some(fixture.main.as_path()) };
    let why: Option<String> = why_the_worktree_stays(&look, fixture.worktree.as_path(), fixture.admin().as_path());
    let said: String = why.expect("a worktree holding another registered worktree stays");
    assert!(said.contains("another worktree this repository registers is inside it"), "{said}");
    assert!(said.contains(inside.display().to_string().as_str()), "the reason names the nested worktree: {said}");
}

/// What:
///  A worktree holding an ignored file the worktree the proof was made from does not hold stays,
///  and the reason names the file.
/// Why:
///  Git deletes what it ignores without a word,
///  so the ignored files are the one hole Git's own refusal leaves.
#[test]
fn a_worktree_holding_an_ignored_file_the_compared_worktree_does_not_hold_stays() {
    let fixture: Fixture = a_repository_with_a_worktree("second-look-ignored", "*.log\n");
    std::fs::write(fixture.worktree.join("person.log").as_path(), b"what the person wrote\n").expect("an ignored file");
    let context: crate::transaction_git::GitContext = git_context();
    let empty: PathBuf = fixture.root.join("proc");
    std::fs::create_dir(empty.as_path()).expect("an empty process table");
    let look: SecondLook<'_> = SecondLook { context: &context, common_dir: fixture.common.as_path(), proc_root: empty.as_path(), source: Some(fixture.main.as_path()) };
    let why: Option<String> = why_the_worktree_stays(&look, fixture.worktree.as_path(), fixture.admin().as_path());
    let said: String = why.expect("an ignored file nothing else holds keeps the worktree");
    assert!(said.contains("person.log"), "the reason names the file: {said}");
}

/// What:
///  A worktree with no worktree the ignored files could be compared with stays,
///  and the reason says there is none.
/// Why:
///  A merge that ran in a repository without a worktree made no comparison,
///  and nothing then proves the ignored files are the repository's.
#[test]
fn a_worktree_with_no_compared_worktree_stays_on_the_second_look() {
    let fixture: Fixture = a_repository_with_a_worktree("second-look-no-source", ".claude/\n");
    let context: crate::transaction_git::GitContext = git_context();
    let empty: PathBuf = fixture.root.join("proc");
    std::fs::create_dir(empty.as_path()).expect("an empty process table");
    let look: SecondLook<'_> = SecondLook { context: &context, common_dir: fixture.common.as_path(), proc_root: empty.as_path(), source: None };
    let why: Option<String> = why_the_worktree_stays(&look, fixture.worktree.as_path(), fixture.admin().as_path());
    assert!(
        matches!(why, Some(said) if said.contains("without a worktree")),
        "nothing proves the ignored files are not the person's: {why:?}"
    );
}
