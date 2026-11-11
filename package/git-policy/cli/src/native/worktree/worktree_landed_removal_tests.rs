//! What:
//!  Controls for the removal of the worktrees whose branches a merge landed.
//! Why:
//!   The removal happens without a person asking for it,
//!  so each condition that keeps a worktree needs a control that would catch it going,
//!  and the clean case needs one that proves the worktree really goes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fixture = aRepositoryWithAWorktree(); const sweep = sweepLandedWorktrees(...); expect(sweep.removed).toHaveLength(1);
//! ```

use super::{LandedPlace, LandedSweep, sweep_landed_worktrees};
use crate::held_commit_record::{Held, write};
use crate::shadow_pool::pool_root_for;
use crate::test_support::{REAL_GIT, age, fixture, git, git_context, git_text, repository};
use crate::worktree_added_by_the_wrapper::record;
use crate::worktree_landed_registrations::{RegisteredBefore, registered_before};
use std::path::{Path, PathBuf};

/// A repository with one linked worktree whose branch holds a commit of its own.
struct Landed {
    /// The fixture directory that holds both.
    root: PathBuf,
    /// The main checkout,
    /// where the merge runs.
    main: PathBuf,
    /// Its common Git directory.
    common: PathBuf,
    /// The linked worktree,
    /// on branch `feature`.
    worktree: PathBuf,
}

/// What:
///  A repository whose branch `feature` is checked out in a linked worktree and holds one commit.
/// Why:
///   Every control starts from a repository where one merge lands one branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const landed = aRepositoryWithAWorktreeWhoseBranchHasACommit(name);
/// ```
fn a_repository_with_a_worktree(name: &str) -> Landed {
    return a_repository_with_a_worktree_and_ignore_rules(name, "");
}

/// What:
///  The same repository,
///  whose first commit also carries the ignore rules a control writes.
/// Why:
///   An ignored file the person wrote is the one state a landed sweep may not take,
///  and Git only calls a file ignored where a rule says so,
///  which both trees must carry for the worktree's own listing to see it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const landed = aRepositoryWithAWorktreeAndIgnoreRules(name, '.env\n');
/// ```
fn a_repository_with_a_worktree_and_ignore_rules(name: &str, rules: &str) -> Landed {
    let root: PathBuf = fixture(name);
    let main: PathBuf = repository(root.as_path(), "main");
    let tracked: PathBuf = main.join("README.md");
    std::fs::write(tracked.as_path(), b"# fixture\n").expect("a tracked file");
    // A file as new as the index proves nothing by its stat fields, and the cleanliness proof would then refuse
    // every worktree below (`test_support::age`).
    age(tracked.as_path());
    let mut committed: Vec<&str> = vec!["README.md"];
    if !rules.is_empty() {
        let ignored: PathBuf = main.join(".gitignore");
        std::fs::write(ignored.as_path(), rules.as_bytes()).expect("the ignore rules");
        age(ignored.as_path());
        committed.push(".gitignore");
    }
    let mut added: Vec<&str> = vec!["add", "--"];
    added.extend(committed);
    git(main.as_path(), added.as_slice());
    git(main.as_path(), &["commit", "--quiet", "--message=a tracked file"]);
    let worktree: PathBuf = root.join("worktree");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "feature", worktree.to_str().expect("a UTF-8 fixture path")]);
    git(worktree.as_path(), &["commit", "--quiet", "--allow-empty", "--message=the feature work"]);
    let common: PathBuf = main.join(".git");
    return Landed { root, main, common, worktree };
}

/// What:
///  An empty stand-in process table for one fixture.
/// Why:
///   The sweep reads the machine's processes,
///  and a control that read the real ones would depend on every program every other session runs:
///  a program started in between that hides its working folder would keep a worktree the control expects removed.
///  The one control about a process working inside writes the entry it needs instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const table = anEmptyProcessTable(landed); // path to the table
/// ```
fn an_empty_process_table(landed: &Landed) -> PathBuf {
    let table: PathBuf = landed.root.join("proc");
    std::fs::create_dir_all(table.as_path()).expect("an empty process table");
    return table;
}

/// The place one control sweeps,
/// with the fixture's own process table.
fn place_of<'a>(landed: &'a Landed, context: &'a crate::transaction_git::GitContext, registered_before: &'a RegisteredBefore, proc_root: &'a Path) -> LandedPlace<'a> {
    return LandedPlace {
        context,
        cwd: landed.main.as_path(),
        common_dir: landed.common.as_path(),
        current_root: Some(landed.main.as_path()),
        proc_root,
        registered_before,
    };
}

/// The registrations the fixture's repository has now.
fn registrations_now(landed: &Landed) -> RegisteredBefore {
    return registered_before(landed.common.as_path()).expect("the registrations of the fixture");
}

/// The head of the main checkout now.
fn head_of(main: &Path) -> String {
    return git_text(main, &["rev-parse", "HEAD"]);
}

/// The worktree's own Git directory, as Git's pointer names it.
fn admin_of(worktree: &Path) -> PathBuf {
    let pointer: String = std::fs::read_to_string(worktree.join(".git")).expect("the worktree's pointer");
    let path: &str = pointer.trim().strip_prefix("gitdir: ").expect("a pointer to its Git directory");
    return PathBuf::from(path);
}

/// What:
///  The state a commit that met a reference lock leaves in the worktree:
///  a commit no branch names, held by the worktree's own reference and named by a held record.
/// Why:
///   Both of those go with the removal,
///  so a sweep that removes the worktree without keeping the commit loses it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = holdACommitInTheWorktree(landed, 7);
/// ```
fn hold_a_commit_in_the_worktree(landed: &Landed, seq: u64) -> String {
    let tree: String = git_text(landed.worktree.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let held: String = git_text(landed.worktree.as_path(), &["commit-tree", tree.as_str(), "-p", "HEAD", "-m", "the held edit"]);
    let reference: String = format!("refs/worktree/cli-git-service/{seq}");
    git(landed.worktree.as_path(), &["update-ref", "--no-deref", reference.as_str(), held.as_str()]);
    // The branch, its reflog and every other ref were expired by the hold, so the worktree's own reference is the only thing
    //  that names the commit.
    git(landed.main.as_path(), &["reflog", "expire", "--expire=now", "--all"]);
    let admin: PathBuf = admin_of(landed.worktree.as_path());
    let record: Held = Held {
        seq,
        journal: landed.root.join("journal"),
        repository: landed.worktree.clone(),
        common_dir: landed.common.clone(),
        git_dir: admin.clone(),
        reference: String::from("refs/heads/feature"),
        old: head_of(landed.worktree.as_path()),
        new: held.clone(),
        commits: vec![held.clone()],
        index: admin.join("index"),
        reflog: b"commit: the held edit".to_vec(),
        locks: vec![landed.common.join("refs/heads/feature.lock")],
        created: String::from("2026-01-01T00:00:00+00:00"),
    };
    write(admin.as_path(), &record).expect("the record of the hold");
    assert_eq!(
        git_text(landed.worktree.as_path(), &["for-each-ref", "--contains", held.as_str(), "--format=%(refname)"]).trim(),
        reference,
        "nothing but the worktree's own reference names the held commit"
    );
    return held;
}

/// What:
///  The shadow pool of the fixture's linked worktree,
///  holding one finished member as a fill leaves it.
/// Why:
///   A pool is named from the administrative directory of the worktree it serves,
///  which Git names after the worktree's own last folder and the removal deletes,
///  so this is the folder a removal either takes or leaves behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pool = poolRootFor(common, join(common, 'worktrees', basename(worktree))); mkdirSync(join(pool, 'member', 'objects'), { recursive: true });
/// ```
fn planted_pool(landed: &Landed) -> PathBuf {
    let last: &std::ffi::OsStr = landed.worktree.file_name().expect("the worktree's last folder names its administrative directory");
    let admin: PathBuf = landed.common.join("worktrees").join(last);
    let pool: PathBuf = pool_root_for(landed.common.as_path(), admin.as_path());
    std::fs::create_dir_all(pool.join("member").join("objects")).expect("a pool as a fill leaves it");
    return pool;
}

/// What:
///  Land `feature` on `main` and answer the sweep of the worktrees it landed.
/// Why:
///   The merge is the event the sweep hangs on,
///  and its two heads are what the sweep compares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sweep = landFeature(landed);
/// ```
fn land_feature(landed: &Landed) -> LandedSweep {
    let before: String = head_of(landed.main.as_path());
    // Taken before the merge starts, as the command that runs a merge takes it: a worktree added while the merge runs is
    // not one this merge ends.
    let registered_before: RegisteredBefore = registrations_now(landed);
    git(landed.main.as_path(), &["merge", "--quiet", "--no-ff", "--message=land the feature", "feature"]);
    let after: String = head_of(landed.main.as_path());
    let context: crate::transaction_git::GitContext = git_context();
    let table: PathBuf = an_empty_process_table(landed);
    let place: LandedPlace<'_> = place_of(landed, &context, &registered_before, table.as_path());
    return sweep_landed_worktrees(&place, before.as_str(), after.as_str(), "main");
}

#[test]
fn a_landed_clean_worktree_is_removed_and_its_branch_stays() {
    let landed: Landed = a_repository_with_a_worktree("landed-clean");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let sweep: LandedSweep = land_feature(&landed);
    assert!(sweep.failed.is_none(), "{:?}", sweep.failed);
    assert_eq!(sweep.removed.len(), 1, "one landed worktree is removed: {sweep:?}");
    assert!(sweep.kept.is_empty() && sweep.unproven.is_empty(), "{sweep:?}");
    assert!(!landed.worktree.exists(), "the worktree directory is gone");
    let branch: String = git_text(landed.main.as_path(), &["rev-parse", "--verify", "refs/heads/feature"]);
    assert_eq!(branch.len(), 40, "the branch the worktree held stays: {branch}");
    assert!(landed.root.join("main").exists(), "the repository itself stays");
}

#[test]
fn a_landed_worktree_with_uncommitted_work_stays() {
    let landed: Landed = a_repository_with_a_worktree("landed-dirty");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let notes: PathBuf = landed.worktree.join("notes.md");
    std::fs::write(notes.as_path(), b"what I was in the middle of\n").expect("a file the person wrote");
    let sweep: LandedSweep = land_feature(&landed);
    assert!(sweep.removed.is_empty(), "nothing is removed while the worktree holds work: {sweep:?}");
    assert_eq!(sweep.kept.len(), 1, "{sweep:?}");
    assert!(!sweep.kept[0].why.is_empty(), "the reason is named: {sweep:?}");
    assert!(landed.worktree.exists(), "the worktree stays");
    assert_eq!(std::fs::read(notes.as_path()).expect("the file is still there"), b"what I was in the middle of\n");
}

/// What:
///  A worktree the sweep removes takes its shadow pool with it,
///  and one it keeps keeps its pool.
/// Why:
///   The pool of a worktree is that worktree's own and sits in the common Git directory,
///  which no removal deletes,
///  so a removal that left it behind would hold a finished tree of every worktree this
///  sweep removes for as long as the repository lives
///  (`worktree_removal_clears_the_pool.rs`).
#[test]
fn a_removed_worktrees_shadow_pool_goes_with_it() {
    let landed: Landed = a_repository_with_a_worktree("landed-pool-gone");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let pool: PathBuf = planted_pool(&landed);
    assert!(pool.exists(), "the pool is there before the sweep: {}", pool.display());
    let sweep: LandedSweep = land_feature(&landed);
    assert_eq!(sweep.removed.len(), 1, "one landed worktree is removed: {sweep:?}");
    assert!(!pool.exists(), "the pool of the removed worktree goes with it: {}", pool.display());
}

#[test]
fn a_kept_worktrees_shadow_pool_stays() {
    let landed: Landed = a_repository_with_a_worktree("landed-pool-kept");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let pool: PathBuf = planted_pool(&landed);
    std::fs::write(landed.worktree.join("notes.md").as_path(), b"what I was in the middle of\n").expect("a file the person wrote");
    let sweep: LandedSweep = land_feature(&landed);
    assert!(sweep.removed.is_empty(), "nothing is removed while the worktree holds work: {sweep:?}");
    assert!(pool.exists(), "the worktree stays, so its pool stays: {}", pool.display());
}

/// What:
///  A stand-in process table with one process,
///  whose working folder is `working` when the table reports one and unreadable when it does not.
/// Why:
///   A control writes the state the sweep must read,
///  and the machine's own processes must not decide what a control sees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const table = aTableWithOneProcessIn(landed, 4242); // path to the table
/// ```
fn a_table_with_one_process(landed: &Landed, pid: u32, working: Option<&Path>, up_seconds: u64, start_ticks: u64) -> PathBuf {
    let table: PathBuf = landed.root.join("proc");
    let entry: PathBuf = table.join(pid.to_string());
    std::fs::create_dir_all(entry.as_path()).expect("a process entry");
    match working {
        Some(directory) => {
            std::os::unix::fs::symlink(directory, entry.join("cwd")).expect("the working folder of the process");
        }
        None => {
            std::fs::write(entry.join("cwd"), b"not a link, so nothing can be read from it").expect("a working folder that cannot be read");
        }
    }
    // The start time is what decides whether a process that hides its folder could be inside the worktree.
    std::fs::write(table.join("uptime"), format!("{up_seconds}.00 0.00\n")).expect("the machine's uptime");
    std::fs::write(entry.join("stat"), stat_with_start_ticks(pid, start_ticks)).expect("the process's own row");
    return table;
}

/// The words `/proc/<pid>/stat` uses for a process that started after `start_ticks` clock ticks of uptime.
fn stat_with_start_ticks(pid: u32, start_ticks: u64) -> String {
    let mut fields: Vec<String> = vec![String::from("R")];
    // Every field between the state and the start time is a number this read never looks at.
    for _unused in 0..18 {
        fields.push(String::from("0"));
    }
    fields.push(start_ticks.to_string());
    return format!("{pid} (a helper) {}\n", fields.join(" "));
}

#[test]
fn a_landed_worktree_with_a_process_inside_stays() {
    let landed: Landed = a_repository_with_a_worktree("landed-busy");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let table: PathBuf = a_table_with_one_process(&landed, 4242, Some(landed.worktree.as_path()), 1_000, 100);
    let registered_before: RegisteredBefore = registrations_now(&landed);
    let before: String = head_of(landed.main.as_path());
    git(landed.main.as_path(), &["merge", "--quiet", "--no-ff", "--message=land the feature", "feature"]);
    let after: String = head_of(landed.main.as_path());
    let context: crate::transaction_git::GitContext = git_context();
    let place: LandedPlace<'_> = place_of(&landed, &context, &registered_before, table.as_path());
    let sweep: LandedSweep = sweep_landed_worktrees(&place, before.as_str(), after.as_str(), "main");
    assert!(sweep.removed.is_empty(), "nothing is removed while a process works there: {sweep:?}");
    assert_eq!(sweep.kept.len(), 1, "{sweep:?}");
    assert!(sweep.kept[0].why.contains("process 4242 is working in it"), "{}", sweep.kept[0].why);
    assert!(landed.worktree.exists(), "the worktree stays");
}

/// What:
///  A process of this account that hides its working folder inside a landed worktree keeps it.
/// Why:
///   Nothing proves such a process is outside the worktree,
///  and skipping it removed the folder under a live process,
///  which a cross-vendor review of the landed sweep reproduced.
#[test]
fn a_landed_worktree_with_a_hidden_process_inside_stays() {
    let landed: Landed = a_repository_with_a_worktree("landed-hidden-process");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    // Started at the very moment the machine came up, so it is younger than the worktree however young the worktree is.
    let table: PathBuf = a_table_with_one_process(&landed, 4343, None, 1_000, 100_000);
    let registered_before: RegisteredBefore = registrations_now(&landed);
    let before: String = head_of(landed.main.as_path());
    git(landed.main.as_path(), &["merge", "--quiet", "--no-ff", "--message=land the feature", "feature"]);
    let after: String = head_of(landed.main.as_path());
    let context: crate::transaction_git::GitContext = git_context();
    let place: LandedPlace<'_> = place_of(&landed, &context, &registered_before, table.as_path());
    let sweep: LandedSweep = sweep_landed_worktrees(&place, before.as_str(), after.as_str(), "main");
    assert!(sweep.removed.is_empty(), "nothing is removed while a process may be working there: {sweep:?}");
    assert_eq!(sweep.kept.len(), 1, "{sweep:?}");
    assert!(sweep.kept[0].why.contains("process 4343"), "the reason names the process: {}", sweep.kept[0].why);
    assert!(sweep.kept[0].why.contains("hides its working folder"), "{}", sweep.kept[0].why);
    assert!(landed.worktree.exists(), "the worktree stays");
}

/// What:
///  An ignored file the person wrote in the worktree after the add keeps that worktree,
///  named in the reason the person reads.
/// Why:
///   This is the state Git's own removal deletes without a word,
///  and the owner's rule is that no command of this program discards a person's data.
#[test]
fn a_landed_worktree_holding_an_ignored_file_the_person_wrote_stays() {
    let landed: Landed = a_repository_with_a_worktree_and_ignore_rules("landed-ignored-own", ".env\n");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let secret: PathBuf = landed.worktree.join(".env");
    std::fs::write(secret.as_path(), b"PASSWORD=the person's own\n").expect("a file the person wrote after the add");
    let sweep: LandedSweep = land_feature(&landed);
    assert!(sweep.removed.is_empty(), "nothing is removed while the worktree holds the person's ignored file: {sweep:?}");
    assert_eq!(sweep.kept.len(), 1, "{sweep:?}");
    assert!(sweep.kept[0].why.contains(".env"), "the reason names the file: {}", sweep.kept[0].why);
    assert_eq!(std::fs::read(secret.as_path()).expect("the file is still there"), b"PASSWORD=the person's own\n");
}

/// What:
///  An ignored file whose bytes this repository's own worktree holds too does not keep it.
/// Why:
///   The bytes are not lost with the worktree,
///  and a sweep that kept every worktree holding an ignored file would free nothing.
#[test]
fn a_landed_worktree_whose_ignored_file_the_repository_also_holds_is_removed() {
    let landed: Landed = a_repository_with_a_worktree_and_ignore_rules("landed-ignored-shared", ".env\n");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    std::fs::write(landed.main.join(".env").as_path(), b"A=1\n").expect("the repository's own ignored file");
    std::fs::write(landed.worktree.join(".env").as_path(), b"A=1\n").expect("the copy the add carries into the worktree");
    let sweep: LandedSweep = land_feature(&landed);
    assert_eq!(sweep.removed.len(), 1, "the worktree is removed: {sweep:?}");
    assert!(sweep.kept.is_empty(), "{sweep:?}");
    assert!(!landed.worktree.exists(), "the worktree directory is gone");
}

/// What:
///  An ignored file of the worktree the repository holds with other bytes keeps it.
/// Why:
///   The person edited it in the worktree,
///  and the bytes that go with the worktree are not the ones this repository keeps.
#[test]
fn a_landed_worktree_whose_ignored_file_the_person_changed_stays() {
    let landed: Landed = a_repository_with_a_worktree_and_ignore_rules("landed-ignored-changed", ".env\n");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    std::fs::write(landed.main.join(".env").as_path(), b"PORT=3000\n").expect("the repository's own ignored file");
    std::fs::write(landed.worktree.join(".env").as_path(), b"PORT=3001\n").expect("the person's own port in the worktree");
    let sweep: LandedSweep = land_feature(&landed);
    assert!(sweep.removed.is_empty(), "the edited file stays with its worktree: {sweep:?}");
    assert_eq!(sweep.kept.len(), 1, "{sweep:?}");
    assert!(sweep.kept[0].why.contains(".env"), "{}", sweep.kept[0].why);
}

/// What:
///  An ignored directory a build wrote into the worktree alone does not keep it.
/// Why:
///   The ignored build state is what the sweep exists to give back,
///  so the removal line says directories go and the space comes back.
#[test]
fn a_landed_worktree_holding_an_ignored_build_directory_is_removed() {
    let landed: Landed = a_repository_with_a_worktree_and_ignore_rules("landed-ignored-build", "build/\n");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    std::fs::create_dir(landed.worktree.join("build").as_path()).expect("a build directory of the worktree's own");
    std::fs::write(landed.worktree.join("build").join("output.o").as_path(), b"build output\n").expect("a file of that directory");
    let sweep: LandedSweep = land_feature(&landed);
    assert_eq!(sweep.removed.len(), 1, "the worktree is removed: {sweep:?}");
    assert!(!landed.worktree.exists(), "the worktree directory is gone");
}

/// What:
///  The commit a reference lock holds in a landed worktree reaches a reference of this repository
///  before the sweep removes that worktree,
///  and real Git's maintenance then leaves it alone.
/// Why:
///   The hold's record and its per-worktree reference live in the worktree's own Git directory,
///  which the removal deletes,
///  so a sweep that skipped this lost a commit the wrapper said it kept
///  (a cross-vendor review of the landed sweep).
#[test]
fn a_landed_worktree_holding_a_commit_keeps_it_on_a_side_reference() {
    let landed: Landed = a_repository_with_a_worktree("landed-held-commit");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let held: String = hold_a_commit_in_the_worktree(&landed, 7);
    let sweep: LandedSweep = land_feature(&landed);
    assert_eq!(sweep.removed.len(), 1, "the landed worktree is removed: {sweep:?}");
    assert_eq!(sweep.kept_commits.len(), 1, "the person is told about the one held commit: {sweep:?}");
    let kept: String = String::from("refs/cli-git/kept/7");
    assert!(sweep.kept_commits[0].contains(kept.as_str()), "the line names where the commit is kept: {}", sweep.kept_commits[0]);
    assert_eq!(git_text(landed.main.as_path(), &["rev-parse", kept.as_str()]), held, "the kept reference names the held commit");
    git(landed.main.as_path(), &["gc", "--prune=now"]);
    assert_eq!(git_text(landed.main.as_path(), &["cat-file", "-t", held.as_str()]), "commit", "the commit is not lost with the worktree");
}

/// What:
///  A worktree a command adds for the branch a merge just landed,
///  after that merge ended,
///  is not the sweep's to remove.
/// Why:
///   The sweep reads the registrations after the merge commit exists,
///  so a registration that was not there when the merge landed was in its candidate set
///  and the worktree it names went with a merge that could not have landed it
///  (a cross-vendor review of the landed sweep).
#[test]
fn a_worktree_added_after_the_merge_is_not_removed_by_its_sweep() {
    let landed: Landed = a_repository_with_a_worktree("landed-late-add");
    // The branch must be free for the late add, so the worktree the fixture made goes before the merge.
    git(landed.main.as_path(), &["worktree", "remove", landed.worktree.to_str().expect("a UTF-8 fixture path")]);
    let before: String = head_of(landed.main.as_path());
    let registered_before: RegisteredBefore = registrations_now(&landed);
    git(landed.main.as_path(), &["merge", "--quiet", "--no-ff", "--message=land the feature", "feature"]);
    let now: String = head_of(landed.main.as_path());
    let late: PathBuf = landed.root.join("late");
    git(landed.main.as_path(), &["worktree", "add", "--quiet", late.to_str().expect("a UTF-8 fixture path"), "feature"]);
    record(landed.common.as_path(), late.as_path()).expect("the record of a worktree this program made");
    let context: crate::transaction_git::GitContext = git_context();
    let table: PathBuf = an_empty_process_table(&landed);
    let place: LandedPlace<'_> = place_of(&landed, &context, &registered_before, table.as_path());
    let sweep: LandedSweep = sweep_landed_worktrees(&place, before.as_str(), now.as_str(), "main");
    assert!(sweep.removed.is_empty(), "a worktree added after the merge landed is not removed: {sweep:?}");
    assert!(sweep.kept.is_empty() && sweep.unproven.is_empty(), "it was no candidate at all: {sweep:?}");
    assert!(late.exists(), "the worktree the late add made is still there");
}

#[test]
fn a_landed_worktree_this_program_cannot_prove_it_made_is_left() {
    let landed: Landed = a_repository_with_a_worktree("landed-unproven");
    let sweep: LandedSweep = land_feature(&landed);
    assert!(sweep.removed.is_empty(), "nothing is removed without the record: {sweep:?}");
    assert_eq!(sweep.unproven.len(), 1, "{sweep:?}");
    assert_eq!(sweep.unproven[0].branch, "refs/heads/feature");
    assert!(landed.worktree.exists(), "the worktree stays");
}

#[test]
fn a_branch_that_merely_sat_on_an_older_commit_is_no_candidate() {
    let landed: Landed = a_repository_with_a_worktree("landed-older");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let older: PathBuf = landed.root.join("older");
    let first: String = git_text(landed.main.as_path(), &["rev-parse", "HEAD"]);
    git(
        landed.main.as_path(),
        &["worktree", "add", "--quiet", "-b", "older", older.to_str().expect("a UTF-8 fixture path"), first.as_str()],
    );
    record(landed.common.as_path(), older.as_path()).expect("the record of a worktree this program made");
    let sweep: LandedSweep = land_feature(&landed);
    assert_eq!(sweep.removed.len(), 1, "only the branch the merge landed is removed: {sweep:?}");
    assert_eq!(sweep.removed[0].worktree, landed.worktree);
    assert!(older.exists(), "a branch that sat on an older commit is no work this merge brought in");
}

/// The path of the host's Git,
/// named so a control can say which Git it used.
#[test]
fn the_fixture_git_is_the_host_git() {
    assert!(Path::new(REAL_GIT).exists(), "{REAL_GIT}");
}

/// What:
///  A landed worktree whose files the sweep removed and whose registration it could not is not reported as left in
///  place.
/// Why:
///   A removal that destroyed the files and could not empty the registration is what a sweep reported as "left the
///  worktree ... in place" while the path was gone,
/// which sends the person to a worktree that is no longer
///  there and says nothing about the registration the printer would have to finish (a Sonnet finding, F1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = landedSweepLines(sweep).join('\n'); expect(lines).not.toContain('in place');
/// ```
#[test]
fn a_landed_worktree_whose_files_went_and_whose_registration_stayed_is_not_reported_in_place() {
    use std::os::unix::fs::PermissionsExt as _;
    let Some(landed) = a_repository_with_a_subvolume_worktree("landed-unfinished") else { return };
    if !crate::fs_btrfs_test_support::removal_allowed(landed.root.as_path()) {
        // Nothing on this mount can destroy a subvolume, so the sweep is Git's and no registration is left behind.
        return;
    }
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let admin: PathBuf = admin_of(landed.worktree.as_path());
    let worktrees: PathBuf = admin.parent().expect("the directory of the registrations").to_path_buf();
    // A registration that cannot be removed after the destroy: its files are gone and it stays, which is the state no
    // line may call "in place".
    std::fs::set_permissions(worktrees.as_path(), std::fs::Permissions::from_mode(0o555)).expect("read-only registrations");
    let sweep: LandedSweep = land_feature(&landed);
    std::fs::set_permissions(worktrees.as_path(), std::fs::Permissions::from_mode(0o755)).expect("restore");
    let lines: String = crate::worktree_landed_sweep_notice::landed_sweep_lines(&sweep).join("\n");
    assert!(!landed.worktree.exists(), "the destroy took the files of the worktree: {lines}");
    assert!(admin.is_dir(), "the registration stayed: {lines}");
    assert!(!lines.contains("in place"), "a worktree whose files are gone is not reported as left in place: {lines}");
    assert!(lines.contains(admin.to_str().expect("UTF-8")), "the line names the registration that stayed: {lines}");
    assert!(lines.contains("refs/heads/feature"), "and the branch that landed: {lines}");
}

/// What:
///  A repository whose branch `feature` is checked out in a linked worktree that is a subvolume of its own,
///  so that a
///  sweep may take the destroy route.
/// Why:
///   The destroy route is the one the sweep takes on btrfs,
/// and it is the one that can leave a registration behind:
///  a worktree that is an ordinary directory is always Git's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const landed = aRepositoryWithASubvolumeWorktree('landed-unfinished'); // undefined off btrfs
/// ```
fn a_repository_with_a_subvolume_worktree(name: &str) -> Option<Landed> {
    let root: PathBuf = crate::fs_btrfs_test_support::btrfs_fixture(name)?;
    let main: PathBuf = repository(root.as_path(), "main");
    let tracked: PathBuf = main.join("README.md");
    std::fs::write(tracked.as_path(), b"# fixture\n").expect("a tracked file");
    age(tracked.as_path());
    git(main.as_path(), &["add", "--", "README.md"]);
    git(main.as_path(), &["commit", "--quiet", "--message=a tracked file"]);
    let worktree: PathBuf = root.join("worktree");
    crate::fs_btrfs_ioctl::subvolume_create_at(worktree.as_path()).expect("the subvolume of the worktree");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "feature", worktree.to_str().expect("a UTF-8 fixture path")]);
    git(worktree.as_path(), &["commit", "--quiet", "--allow-empty", "--message=the feature work"]);
    age(worktree.join("README.md").as_path());
    git(worktree.as_path(), &["update-index", "--refresh"]);
    let common: PathBuf = main.join(".git");
    return Some(Landed { root, main, common, worktree });
}
