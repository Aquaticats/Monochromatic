//! What:
//!  Controls of the refusal a plain `git worktree remove` gets while a process works in the worktree.
//! Why:
//!   The refusal is the guard against deleting the folder a program is working in,
//!  so each ending needs a control that would catch it going,
//!  and one control has to prove the question can also be answered "go":
//!  a guard that refused every removal would pass every control but that one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(whyARemovalIsRefused(removal)).toContain('process 4242 (`sleep`) is working in it');
//! ```

use super::{REMOVAL_ESCAPE, RemovalInQuestion, why_a_removal_is_refused};
use crate::test_support::{age, fixture, git, git_context, repository};
use std::path::{Path, PathBuf};

/// A repository and one linked worktree of it.
struct Fixture {
    /// The fixture directory that holds both.
    root: PathBuf,
    /// The common Git directory.
    common: PathBuf,
    /// The linked worktree.
    worktree: PathBuf,
}

/// What:
///  The repository every control here works on.
/// Why:
///  The guard reads the worktrees the repository registers,
///  the process table and a nested worktree's own state,
///  so a control needs all three in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fixture = aRepositoryWithAWorktree('nested/\n');
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
    let worktree: PathBuf = root.join("topic");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "feature", worktree.to_str().expect("a UTF-8 fixture path")]);
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("the common Git directory");
    return Fixture { root, common, worktree: std::fs::canonicalize(worktree.as_path()).expect("the worktree's real path") };
}

/// What:
///  A process table holding no process at all.
/// Why:
///  A control about something else gives an empty table,
///  so its answer never depends on the machine's own programs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const table = anEmptyProcessTable(root);
/// ```
fn an_empty_process_table(root: &Path) -> PathBuf {
    let table: PathBuf = root.join("proc");
    std::fs::create_dir_all(table.as_path()).expect("an empty process table");
    return table;
}

/// What:
///  A process table holding one process whose working folder is `working`.
/// Why:
///  The guard reads a process table the caller names,
///  and the one process is the whole answer it is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const table = aTableWithOneProcess(root, 4242, worktree, 'sleep');
/// ```
fn a_table_with_one_process(root: &Path, pid: u32, working: &Path, program: &str) -> PathBuf {
    let table: PathBuf = root.join("proc");
    let entry: PathBuf = table.join(pid.to_string());
    std::fs::create_dir_all(entry.as_path()).expect("a process entry");
    std::os::unix::fs::symlink(working, entry.join("cwd")).expect("the working folder of the process");
    std::fs::write(entry.join("comm"), format!("{program}\n")).expect("the program the process runs");
    std::fs::write(table.join("uptime"), b"1000.00 0.00\n").expect("the machine's uptime");
    let mut fields: Vec<String> = vec![String::from("R")];
    for _unused in 0..18 {
        fields.push(String::from("0"));
    }
    fields.push(String::from("100"));
    std::fs::write(entry.join("stat"), format!("{pid} (a helper) {}\n", fields.join(" "))).expect("the process's own row");
    return table;
}

#[test]
fn a_worktree_nothing_is_working_in_may_be_removed() {
    let fixture: Fixture = a_repository_with_a_worktree("refusal-clean", "nested/\n");
    let table: PathBuf = an_empty_process_table(fixture.root.as_path());
    let context: crate::transaction_git::GitContext = git_context();
    let refusal: Option<String> = why_a_removal_is_refused(&RemovalInQuestion {
        context: &context,
        common_dir: fixture.common.as_path(),
        proc_root: table.as_path(),
        target: fixture.worktree.as_path(),
        force: false,
    });
    assert_eq!(refusal, None, "a worktree with nothing of the person's in it may go");
}

/// What:
///  A worktree a process is working in is refused,
///  and the words name the process by its number and its program.
/// Why:
///   A person has to know which program to stop,
///  and the program's name is the kernel's record of the running image,
///  never a command line.
#[test]
fn a_process_working_in_the_worktree_is_refused_by_its_number_and_its_program() {
    let fixture: Fixture = a_repository_with_a_worktree("refusal-busy", "nested/\n");
    let table: PathBuf = a_table_with_one_process(fixture.root.as_path(), 4242, fixture.worktree.as_path(), "sleep");
    let context: crate::transaction_git::GitContext = git_context();
    let refusal: String = why_a_removal_is_refused(&RemovalInQuestion {
        context: &context,
        common_dir: fixture.common.as_path(),
        proc_root: table.as_path(),
        target: fixture.worktree.as_path(),
        force: false,
    })
    .expect("a worktree a process is working in stays");
    assert!(refusal.contains("process 4242 (`sleep`) is working in it"), "{refusal}");
    assert!(refusal.contains(fixture.worktree.display().to_string().as_str()), "the worktree is named: {refusal}");
    assert!(refusal.contains(REMOVAL_ESCAPE), "the escape is named: {refusal}");
    assert!(refusal.contains("Stop that program"), "the reader is told what clears the way: {refusal}");
}

/// What:
///  A forced removal is never refused,
///  whatever is working in the worktree.
/// Why:
///  `--force` is the person's own decision,
///  and real Git deletes the worktree that way,
///  so the wrapper must not stand between the person and it.
#[test]
fn a_forced_removal_is_not_refused_by_a_process_working_in_the_worktree() {
    let fixture: Fixture = a_repository_with_a_worktree("refusal-forced", "nested/\n");
    let table: PathBuf = a_table_with_one_process(fixture.root.as_path(), 4343, fixture.worktree.as_path(), "sleep");
    let context: crate::transaction_git::GitContext = git_context();
    assert_eq!(
        why_a_removal_is_refused(&RemovalInQuestion {
            context: &context,
            common_dir: fixture.common.as_path(),
            proc_root: table.as_path(),
            target: fixture.worktree.as_path(),
            force: true,
        }),
        None,
        "a forced removal runs exactly as real Git runs it"
    );
}

/// What:
///  A process table that cannot be read refuses,
///  and says which question went unanswered.
/// Why:
///  Nothing a missing table proves lets a removal take the folder a process is working in.
#[test]
fn a_process_table_that_cannot_be_read_refuses() {
    let fixture: Fixture = a_repository_with_a_worktree("refusal-no-table", "nested/\n");
    let context: crate::transaction_git::GitContext = git_context();
    let refusal: String = why_a_removal_is_refused(&RemovalInQuestion {
        context: &context,
        common_dir: fixture.common.as_path(),
        proc_root: fixture.root.join("no-such-proc").as_path(),
        target: fixture.worktree.as_path(),
        force: false,
    })
    .expect("nothing proves the worktree is empty");
    assert!(refusal.contains("cannot be read"), "{refusal}");
    assert!(refusal.contains(REMOVAL_ESCAPE), "the escape is named: {refusal}");
}
