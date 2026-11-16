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
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::fs_kind::is_subvolume_root;
use crate::lock_birth_times::{BirthTimes, with_birth_times};
use crate::test_support::{age, fixture, git, git_context, remove, repository};
use crate::worktree_copy::{GitStatus, GitStep};
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_ladder_run::{Ladder, LadderResult, begin_ladder};
use crate::worktree_ladder_test_support::{main_identity, plan_to, subvolume_repository};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

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
    let worktree: PathBuf = root.join("topic");
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
        matches!(&why, Some(said) if said.contains("without a worktree")),
        "nothing proves the ignored files are not the person's: {why:?}"
    );
}

/// What:
///  A runner for the ladder that must never be asked for a Git step.
/// Why:
///  A worktree whose tree is the source's needs no step after real Git ran without a checkout,
///  and a step this control did not expect would be a different add from the one it measures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noRunner = () => { throw new Error('no Git step is expected') };
/// ```
fn no_runner() -> impl FnMut(&GitStep<'_>) -> Result<GitStatus, WorktreeCopyError> {
    return |_step| { panic!("the tree is the source's, so the ladder runs no Git step") };
}

/// What:
///  A linked worktree of `repo` made the way the wrapper makes one on a btrfs volume:
///  real Git adds it without a checkout,
///  and the ladder puts a snapshot of the source at the destination.
/// Why:
///  The folder of that worktree carries the birth time of the subvolume the snapshot was taken from,
///  which is not the time the worktree was made,
///  so a control of that time needs a worktree made this way and no other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worktree = aSnapshotWorktree(repo, root); // the destination, a snapshot subvolume
/// ```
fn a_snapshot_worktree(repo: &Path, root: &Path) -> PathBuf {
    let destination: PathBuf = root.join("topic");
    let plan = plan_to(destination.clone());
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo));
    assert_eq!(ladder.first_step(), GitStep::WithoutCheckout(plan.insert_at), "a clean subvolume source is snapshotted");
    git(repo, &["worktree", "add", "--no-checkout", "-b", "topic", "../topic"]);
    let mut runner = no_runner();
    let result: LadderResult = ladder.finish(GitStatus { code: Some(0) }, &mut runner, &git_context(), &mut std::io::sink(), &mut std::io::sink());
    assert_eq!(result.git, GitStatus { code: Some(0) });
    assert!(is_subvolume_root(destination.as_path()).expect("probe"), "the destination is the snapshot's own subvolume");
    assert_eq!(result.carried, Some(std::fs::canonicalize(destination.as_path()).expect("canonical destination")), "the destination received the source's ignored state with the snapshot");
    return std::fs::canonicalize(destination.as_path()).expect("the canonical worktree");
}

/// What:
///  A process table holding one process whose working folder cannot be read,
///  started at `started`.
/// Why:
///  The start of a process that hides its working folder against the time the worktree was made is the whole question,
///  and a control writes both numbers instead of waiting for a process to be born at the moment it needs.
///  The machine is said to have been up for `UPTIME_SECONDS`,
///  and the start is written in the hundredths of a second the process table counts in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const table = aTableOfAHiddenProcessStartedAt(root, 4242, started);
/// ```
fn a_table_of_a_hidden_process_started_at(root: &Path, pid: u32, started: SystemTime) -> PathBuf {
    // Long enough that a start written as ticks of a second stays a small number.
    const UPTIME_SECONDS: u64 = 100_000;
    // The kernel counts a process's start in hundredths of a second here (`sysconf(_SC_CLK_TCK)`).
    const TICKS_IN_A_SECOND: u64 = 100;
    let table: PathBuf = root.join("proc");
    let entry: PathBuf = table.join(pid.to_string());
    std::fs::create_dir_all(entry.as_path()).expect("a process entry");
    // A file where a link belongs is a working folder nothing can read, which is this account's answer to a hidden one.
    std::fs::write(entry.join("cwd"), b"not a link, so nothing can be read from it").expect("a working folder that cannot be read");
    let age: Duration = SystemTime::now().duration_since(started).expect("a start in the past");
    let age_ticks: u64 = u64::try_from(age.as_millis()).expect("an age that fits the ticks") / 10;
    let started_ticks: u64 = UPTIME_SECONDS * TICKS_IN_A_SECOND - age_ticks;
    let mut fields: Vec<String> = vec![String::from("R")];
    for _unused in 0..18 {
        fields.push(String::from("0"));
    }
    fields.push(started_ticks.to_string());
    std::fs::write(table.join("uptime"), format!("{UPTIME_SECONDS}.00 0.00\n")).expect("the machine's uptime");
    std::fs::write(entry.join("stat"), format!("{pid} (a helper) {}\n", fields.join(" "))).expect("the process's own row");
    return table;
}

/// What:
///  A process table holding one process of this account whose working folder cannot be read.
/// Why:
///  The answer about a hidden process whose worktree has no readable made time does not rest on its start,
///  so this table names no start at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const table = aTableWithAHiddenProcess(root, 5656);
/// ```
fn a_table_with_a_hidden_process(root: &Path, pid: u32) -> PathBuf {
    let table: PathBuf = root.join("proc");
    let entry: PathBuf = table.join(pid.to_string());
    std::fs::create_dir_all(entry.as_path()).expect("a process entry");
    std::fs::write(entry.join("cwd"), b"not a link, so nothing can be read from it").expect("a working folder that cannot be read");
    return table;
}

/// What:
///  A landed worktree made as a snapshot is removed when the only process that could work in it started before the
///  worktree was made,
///  and kept when that process started after.
/// Why:
///   A btrfs snapshot keeps the birth time of the subvolume it was taken from,
///  so the worktree's folder reads as old as the checkout the worktree came from,
///  and a sweep that judged a process that hides its working folder by the folder's birth time kept every snapshot
///  worktree for as long as the service manager lived
///  (seen on this machine 2026-11-13:
/// the folder carried `2026-11-08T08:33:27.265Z` while the worktree's record was born eleven hours after the service
///  manager started,
///  and the sweep left the worktree in place).
///  The record the add writes is not a snapshot's,
///  so the time it names separates the worktree's own work from every process older than it.
///  The process table is written rather than read from the machine,
///  because no control can choose when this machine's own programs started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const why = whyTheWorktreeStays(look, snapshotWorktree, admin); // undefined before, the process after
/// ```
#[test]
fn btrfs_a_worktree_made_as_a_snapshot_is_removed_when_the_process_that_hides_its_folder_started_before_it() {
    let Some(root) = btrfs_fixture("second-look-snapshot-made-time") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let source_born: SystemTime = std::fs::metadata(repo.as_path()).expect("the source subvolume").created().expect("a birth time for the source");
    // The worktree's folder carries the source's birth time, so the add is made a second after it:
    // a process started between the two started before the worktree was made and after the source was.
    std::thread::sleep(Duration::from_millis(1_200));
    let worktree: PathBuf = a_snapshot_worktree(repo.as_path(), root.as_path());
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    let made: SystemTime = std::fs::metadata(admin.as_path()).expect("the registration").created().expect("a birth time for the registration");
    assert_eq!(
        std::fs::metadata(worktree.as_path()).expect("the worktree").created().expect("a birth time for the worktree"),
        source_born,
        "the folder the snapshot became carries the source's birth time, which is the reading this control exists for"
    );
    assert!(source_born < made, "the record that says when the worktree was made is younger than the source it was copied from");
    let context: crate::transaction_git::GitContext = git_context();
    let common: PathBuf = repo.join(".git");
    let older: SystemTime = source_born + made.duration_since(source_born).expect("the record is younger than the source") / 2;
    let older_table: PathBuf = a_table_of_a_hidden_process_started_at(root.as_path(), 6666, older);
    let look: SecondLook<'_> = SecondLook { context: &context, common_dir: common.as_path(), proc_root: older_table.as_path(), source: Some(repo.as_path()) };
    assert_eq!(
        why_the_worktree_stays(&look, worktree.as_path(), admin.as_path()),
        None,
        "a process that started before the worktree was made proves nothing about it, so a landed worktree goes"
    );
    let younger: SystemTime = made + SystemTime::now().duration_since(made).expect("the worktree was made a moment ago") / 2;
    let younger_table: PathBuf = a_table_of_a_hidden_process_started_at(root.as_path(), 6666, younger);
    let younger_look: SecondLook<'_> = SecondLook { context: &context, common_dir: common.as_path(), proc_root: younger_table.as_path(), source: Some(repo.as_path()) };
    let why: Option<String> = why_the_worktree_stays(&younger_look, worktree.as_path(), admin.as_path());
    let said: String = why.expect("a process that started after the worktree was made may be working inside it");
    assert!(said.contains("process 6666"), "the reason names the process: {said}");
    assert!(said.contains("hides its working folder"), "{said}");
    assert!(said.contains("nothing shows it started before this worktree was made"), "{said}");
    remove(root.as_path());
}

/// What:
///  A worktree whose made time no record names is kept,
///  and the reason says that time is unknown rather than that the process is young.
/// Why:
///   A file system that records no birth time says nothing about when the worktree existed,
///  and a notice that said the process started after the worktree was made would state a fact nothing read,
///  so a person would look for a program that started there instead of at the worktree's own records.
///  The arm that hides birth times is the one this package already has for such a file system,
///  and it is set for the length of one call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const why = withAbsentBirthTimes(() => whyTheWorktreeStays(look, worktree, admin)); // '... nothing shows when this worktree was made ...'
/// ```
#[test]
fn a_worktree_whose_made_time_no_record_names_is_kept_and_the_reason_says_the_time_is_unknown() {
    let fixture: Fixture = a_repository_with_a_worktree("second-look-made-time-unknown", ".claude/\n");
    let table: PathBuf = a_table_with_a_hidden_process(fixture.root.as_path(), 5656);
    let context: crate::transaction_git::GitContext = git_context();
    let look: SecondLook<'_> = SecondLook { context: &context, common_dir: fixture.common.as_path(), proc_root: table.as_path(), source: Some(fixture.main.as_path()) };
    let why: Option<String> = with_birth_times(BirthTimes::Hidden, || {
        return why_the_worktree_stays(&look, fixture.worktree.as_path(), fixture.admin().as_path());
    });
    let said: String = why.expect("a worktree whose made time no record names stays");
    assert!(said.contains("process 5656"), "the reason names the process: {said}");
    assert!(said.contains("hides its working folder"), "{said}");
    assert!(said.contains("nothing shows when this worktree was made"), "{said}");
    assert!(!said.contains("started before this worktree was made"), "the reason does not claim the process is young: {said}");
}
