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

use super::{LandedPlace, LandedSweep, holds_its_own_commits, sweep_landed_worktrees};
use crate::held_commit_record::{Held, write};
use crate::shadow_pool::pool_root_for;
use crate::service_work::Verdict;
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

/// What:
///  The answer of a wake in a control,
///  which reaches no commit service.
/// Why:
///  A real wake provisions the service of this machine's manager,
///  and no control may start one;
///  the record a hand-over writes is what the control reads instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noServiceAnswers = () => Ok;
/// ```
fn no_service_answers(_context: &crate::transaction_git::GitContext, _git_dir: &Path) -> Result<(), String> {
    return Ok(());
}

/// The place one control sweeps,
/// with the fixture's own process table.
fn place_of<'a>(
    landed: &'a Landed,
    context: &'a crate::transaction_git::GitContext,
    registered_before: &'a RegisteredBefore,
    proc_root: &'a Path,
    git_dir: &'a Path,
) -> LandedPlace<'a> {
    return LandedPlace {
        context,
        cwd: landed.main.as_path(),
        common_dir: landed.common.as_path(),
        current_root: Some(landed.main.as_path()),
        proc_root,
        registered_before,
        git_dir,
        told: &no_service_answers,
    };
}

/// What:
///  The removals a sweep handed to the commit service,
///  performed by the call the service makes for each record.
/// Why:
///  Git's own removal reads every tracked file of a worktree and is a step that may not be run twice,
///  so the merge no longer waits for it:
///  every control that means "the worktree went" runs its record through that call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const letTheServiceFinish = (sweep) => sweep.handedOver.map((one) => deleteSubvolumeOf(one.record));
/// ```
fn let_the_service_finish(sweep: &LandedSweep) -> Vec<Verdict> {
    let mut verdicts: Vec<Verdict> = Vec::new();
    for one in &sweep.handed_over {
        verdicts.push(crate::worktree_remove_hand_over::delete_subvolume_of(&one.record));
    }
    return verdicts;
}

/// The registrations the fixture's repository has now.
fn registrations_now(landed: &Landed) -> RegisteredBefore {
    return registered_before(landed.common.as_path()).expect("the registrations of the fixture");
}

/// The head of the main checkout now.
fn head_of(main: &Path) -> String {
    return git_text(main, &["rev-parse", "HEAD"]);
}

/// The commit as the notice text names it,
/// with no abbreviation to resolve.
fn plain(commit: &str) -> String {
    return commit.to_string();
}

/// Whether real Git still has the commit `commit` in the fixture's repository.
/// Why:
///  A removal that took every reference to a commit leaves it to Git's own maintenance,
///  and only a Git that answered is evidence that the commit really went.
fn commit_is_there(repository: &Path, commit: &str) -> bool {
    return std::process::Command::new(REAL_GIT)
        .arg("-C")
        .arg(repository)
        .args(["cat-file", "-e", commit])
        .output()
        .expect("Git can be asked whether a commit is there")
        .status
        .success();
}

/// Plant the lock file a refused update leaves on the reference `name`,
/// as a killed or a concurrent Git leaves it.
fn plant_lock(common_dir: &Path, name: &str) {
    let lock: PathBuf = common_dir.join(format!("{name}.lock"));
    std::fs::create_dir_all(lock.parent().expect("the directory of the lock")).expect("the lock's directory");
    std::fs::write(lock.as_path(), b"").expect("the lock file");
}

/// The worktree's own Git directory,
/// as Git's pointer names it.
fn admin_of(worktree: &Path) -> PathBuf {
    let pointer: String = std::fs::read_to_string(worktree.join(".git")).expect("the worktree's pointer");
    let path: &str = pointer.trim().strip_prefix("gitdir: ").expect("a pointer to its Git directory");
    return PathBuf::from(path);
}

/// What:
///  The state a commit that met a reference lock leaves in the worktree:
///  a commit no branch names,
/// held by the worktree's own reference and named by a held record.
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
    return land_feature_hearing(landed, &no_service_answers);
}

/// What:
///  The same merge and sweep,
///  with the answer a wake gets given here.
/// Why:
///  A control about the wake itself needs to read what a hand-over told the service,
///  and no control may reach one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sweep = landFeatureHearing(landed, told); // the sweep, with every wake answered by told
/// ```
fn land_feature_hearing(landed: &Landed, told: &dyn Fn(&crate::transaction_git::GitContext, &Path) -> Result<(), String>) -> LandedSweep {
    let before: String = head_of(landed.main.as_path());
    // Taken before the merge starts, as the command that runs a merge takes it: a worktree added while the merge runs is
    // not one this merge ends.
    let registered_before: RegisteredBefore = registrations_now(landed);
    git(landed.main.as_path(), &["merge", "--quiet", "--no-ff", "--message=land the feature", "feature"]);
    let after: String = head_of(landed.main.as_path());
    let context: crate::transaction_git::GitContext = git_context();
    let table: PathBuf = an_empty_process_table(landed);
    let place: LandedPlace<'_> = LandedPlace {
        context: &context,
        cwd: landed.main.as_path(),
        common_dir: landed.common.as_path(),
        current_root: Some(landed.main.as_path()),
        proc_root: table.as_path(),
        registered_before: &registered_before,
        git_dir: landed.common.as_path(),
        told,
    };
    return sweep_landed_worktrees(&place, before.as_str(), after.as_str(), "main");
}

/// A repository,
/// a worktree on `feature` that a merge runs in,
/// and one on `task-z` that a person has just added and not started in:
/// its branch holds no commit of its own,
/// and its tip is main's.
/// Why:
///  Every control of a fresh worktree needs the two worktrees and the commit main moved by,
///  in one place,
/// so none of them drifts from the state a person leaves.
fn a_fresh_worktree_beside_the_one_a_merge_runs_in(name: &str) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let main: PathBuf = repository(root.as_path(), "main");
    let tracked: PathBuf = main.join("README.md");
    std::fs::write(tracked.as_path(), b"# fixture\n").expect("a tracked file");
    age(tracked.as_path());
    git(main.as_path(), &["add", "--", "README.md"]);
    git(main.as_path(), &["commit", "--quiet", "--message=a tracked file"]);
    let common: PathBuf = main.join(".git");
    let merged_in: PathBuf = root.join("x");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "feature", merged_in.to_str().expect("a UTF-8 fixture path")]);
    record(common.as_path(), merged_in.as_path()).expect("the record of a worktree this program made");
    // main moves, as a person landing a commit in the main checkout makes it move.
    let moved: PathBuf = main.join("more.txt");
    std::fs::write(moved.as_path(), b"more\n").expect("a file main commits");
    age(moved.as_path());
    git(main.as_path(), &["add", "--", "more.txt"]);
    git(main.as_path(), &["commit", "--quiet", "--message=main moves"]);
    // A new task starts at main's tip: its branch has no commit of its own.
    let fresh: PathBuf = root.join("z");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "task-z", fresh.to_str().expect("a UTF-8 fixture path")]);
    record(common.as_path(), fresh.as_path()).expect("the record of a worktree this program made");
    return (root, main, merged_in, fresh);
}

/// What:
///  A worktree added for a branch that holds no commit of its own is not removed by a merge of another branch.
/// Why:
///  The merge `git merge main` inside a feature worktree brought in main's commits,
///  and the sweep counted every branch contained in the new head and not in the head before it,
///  so a branch that merely pointed at main's tip looked landed:
///  the worktree a person had just added went,
///  though nothing of it was in any merge
///  (a cross-vendor audit of `574063773d`,
///  finding F4).
///  What the removal deletes of a worktree it takes is a separate proof (`worktree_landed_ignored_files.rs`),
///  so this control holds no ignored file:
/// nothing may keep it but the rule about the branch's own commits.
#[test]
fn a_worktree_whose_branch_holds_no_commit_of_its_own_is_not_removed_by_another_branchs_merge() {
    let (root, main, merged_in, fresh) = a_fresh_worktree_beside_the_one_a_merge_runs_in("landed-fresh-branch");
    let common: PathBuf = main.join(".git");
    let before: String = head_of(merged_in.as_path());
    let registered_before: RegisteredBefore = registered_before(common.as_path()).expect("the registrations of the fixture");
    git(merged_in.as_path(), &["merge", "--quiet", "--no-ff", "--message=update the branch from main", "main"]);
    let now: String = head_of(merged_in.as_path());
    let context: crate::transaction_git::GitContext = git_context();
    let table: PathBuf = root.join("proc");
    std::fs::create_dir_all(table.as_path()).expect("an empty process table");
    // The common Git directory, where a handed-over removal is recorded: the record a person's own removal writes names it too.
    let own_git_dir: PathBuf = common.clone();
    let place: LandedPlace<'_> = LandedPlace {
        context: &context,
        cwd: merged_in.as_path(),
        common_dir: common.as_path(),
        current_root: Some(merged_in.as_path()),
        proc_root: table.as_path(),
        registered_before: &registered_before,
        git_dir: own_git_dir.as_path(),
        told: &no_service_answers,
    };
    let sweep: LandedSweep = sweep_landed_worktrees(&place, before.as_str(), now.as_str(), "feature");
    assert!(fresh.exists(), "the worktree a person added and has not started in is still there: {sweep:?}");
    assert!(sweep.removed.is_empty(), "a branch with no commit of its own landed in no merge: {sweep:?}");
}

/// What:
///  The rule that keeps a fresh worktree out of the sweep answers both ways:
///  a branch that has moved since it was made holds a commit of its own,
///  and one that has not does not.
/// Why:
///  A rule that answered "no commit of its own" to every branch would remove nothing at all,
///  so the arm that keeps the worktree needs the arm that removes one beside it in the same control.
#[test]
fn a_branch_that_has_moved_since_it_was_made_holds_a_commit_of_its_own() {
    let (root, main, _merged_in, fresh) = a_fresh_worktree_beside_the_one_a_merge_runs_in("landed-branch-own-commits");
    let common: PathBuf = main.join(".git");
    let task: &str = "refs/heads/task-z";
    assert!(
        !holds_its_own_commits(common.as_path(), task, head_of(fresh.as_path()).trim()),
        "a branch a person added and committed nothing on holds no commit of its own"
    );
    git(fresh.as_path(), &["commit", "--quiet", "--allow-empty", "--message=the task's first work"]);
    assert!(
        holds_its_own_commits(common.as_path(), task, head_of(fresh.as_path()).trim()),
        "a branch that has a commit of its own is no longer a branch that never moved"
    );
    // A repository that keeps no reflog for the branch proves nothing, so the branch stays a candidate.
    let logs: PathBuf = common.join("logs/refs/heads/task-z");
    std::fs::remove_file(logs.as_path()).expect("the branch's own reflog");
    assert!(
        holds_its_own_commits(common.as_path(), task, head_of(fresh.as_path()).trim()),
        "without a reflog nothing proves the branch never moved, so it is examined as it was before"
    );
    drop(root);
}

/// What:
///  A landed worktree that holds another registered worktree is kept,
///  and the worktree inside it keeps its uncommitted work.
/// Why:
///  A worktree can be added from inside another one,
///  and the nested checkout is a directory of the outer worktree under an ignored name,
///  so the removal took its modified tracked file and its untracked file with it and left its registration naming a path
///  that was gone
///  (a cross-vendor audit of `574063773d`,
///  finding F2).
#[test]
fn a_landed_worktree_holding_another_registered_worktree_stays() {
    let landed: Landed = a_repository_with_a_worktree_and_ignore_rules("landed-nested-worktree", ".claude/\n");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let inside: PathBuf = landed.worktree.join(".claude/worktrees/b");
    git(
        landed.worktree.as_path(),
        &["worktree", "add", "--quiet", "-b", "sub", inside.to_str().expect("a UTF-8 fixture path")],
    );
    record(landed.common.as_path(), inside.as_path()).expect("the record of a worktree this program made");
    let changed: PathBuf = inside.join("README.md");
    std::fs::write(changed.as_path(), b"# the fixture, edited in the nested worktree\n").expect("a tracked file the person changed there");
    let notes: PathBuf = inside.join("notes.txt");
    std::fs::write(notes.as_path(), b"work the person has not committed\n").expect("an untracked file the person wrote there");
    let sweep: LandedSweep = land_feature(&landed);
    assert!(sweep.removed.is_empty(), "nothing is removed while a registered worktree is inside the one that landed: {sweep:?}");
    assert_eq!(sweep.kept.len(), 1, "the worktree that holds it is named as kept: {sweep:?}");
    assert!(sweep.kept[0].why.contains("inside it"), "the reason says what is inside it: {}", sweep.kept[0].why);
    assert!(sweep.kept[0].why.contains(inside.to_str().expect("a UTF-8 fixture path")), "the reason names the worktree inside: {}", sweep.kept[0].why);
    assert_eq!(std::fs::read(changed.as_path()).expect("the changed file is still there"), b"# the fixture, edited in the nested worktree\n");
    assert_eq!(std::fs::read(notes.as_path()).expect("the untracked file is still there"), b"work the person has not committed\n");
    assert!(inside.exists(), "the nested worktree is where it was");
}

/// What:
///  The unshown push failure of a worktree the sweep removes reaches the person from the next command,
///  as it does when a person removes that worktree.
/// Why:
///  A worktree keeps its records in its own Git directory,
/// which the removal deletes;
///  the sweep ran no removal snapshot,
/// so a failed push nobody had seen vanished with no word
///  (a cross-vendor audit of `574063773d`,
///  finding F3).
#[test]
fn a_swept_worktrees_unshown_push_failure_is_shown_by_the_next_command() {
    let landed: Landed = a_repository_with_a_worktree("landed-push-notice");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let admin: PathBuf = admin_of(landed.worktree.as_path());
    let commit: String = head_of(landed.worktree.as_path());
    crate::push_failure_notice::record_push_failure(
        admin.as_path(),
        "refs/heads/feature",
        commit.as_str(),
        crate::push_failure_notice::PushOutcome::Refused,
        "refused by test policy",
    )
    .expect("a failed push nobody has seen yet");
    let sweep: LandedSweep = land_feature(&landed);
    assert_eq!(sweep.handed_over.len(), 1, "the landed worktree waits for the commit service: {sweep:?}");
    assert_eq!(let_the_service_finish(&sweep), vec![Verdict::Done], "the service's own removal succeeds");
    assert!(!admin.exists(), "the removal took the record that named the failure");
    // What the next command does with the removal this one made.
    crate::worktree_removal_rescue::settle_removals(landed.common.as_path());
    let texts: Vec<String> = crate::test_support::shown_notices(crate::push_failure_notice::orphaned_push_failures(
        landed.common.as_path(),
        &plain,
    ));
    assert!(
        texts.iter().any(|text| { return text.contains("branch feature") && text.contains(commit.as_str()) && text.contains("refused by test policy"); }),
        "the push failure the removal would have destroyed is shown: {texts:?}"
    );
}

/// What:
///  A merge run again after a refused sweep looks at no landed worktree,
///  and writes no kept reference,
///  which is why the notice names the removal that finishes the work instead.
/// Why:
///  The refusal used to prescribe the same command again,
///  and the command that reaches the sweep is a merge:
///  its branch already landed,
///  so the head does not move and the sweep is skipped,
///  and the person following the advice got silence and no preserved commit
///  (a cross-vendor review of the landed sweep,
///  finding 2).
///  This control pins that fact:
/// if the sweep ever gains a retry,
///  it fails,
/// and the notice that says a merge again removes nothing is read again.
#[test]
fn a_merge_run_again_after_a_refusal_looks_at_no_landed_worktree() {
    let landed: Landed = a_repository_with_a_worktree("landed-refusal-retry");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let _held: String = hold_a_commit_in_the_worktree(&landed, 7);
    for name in ["refs/cli-git/kept/7", "refs/cli-git/kept/7-rescue"] {
        plant_lock(landed.common.as_path(), name);
    }
    let identity: crate::worktree_identity::WorktreeIdentity = crate::worktree_identity::WorktreeIdentity::MainWorktree {
        common_dir: landed.common.clone(),
        git_dir: landed.common.clone(),
        worktree_root: landed.main.clone(),
    };
    let (context, window) = crate::worktree_landed_after_a_merge::window(git_context(), &identity).expect("the branch as the merge finds it");
    git(landed.main.as_path(), &["merge", "--quiet", "--no-ff", "--message=land the feature", "feature"]);
    let first: LandedSweep = crate::worktree_landed_after_a_merge::after_a_merge(&context, &identity, &window, 0).expect("the merge landed the branch");
    assert!(first.failed.is_some(), "neither kept reference can be written: {first:?}");
    assert!(landed.worktree.exists(), "the refusal leaves the worktree where it is");
    for name in ["refs/cli-git/kept/7", "refs/cli-git/kept/7-rescue"] {
        std::fs::remove_file(landed.common.join(format!("{name}.lock"))).expect("the lock nothing holds is removed");
    }
    let (retry_context, retry_window) = crate::worktree_landed_after_a_merge::window(git_context(), &identity).expect("the branch as the retry finds it");
    git(landed.main.as_path(), &["merge", "--quiet", "--no-ff", "--message=land the feature", "feature"]);
    let retry: Option<LandedSweep> = crate::worktree_landed_after_a_merge::after_a_merge(&retry_context, &identity, &retry_window, 0);
    assert!(retry.is_none(), "a merge whose branch already landed moves nothing and looks at no worktree: {retry:?}");
    assert!(landed.worktree.exists(), "the worktree is where the refusal left it");
    assert!(!landed.common.join("refs/cli-git/kept/7").exists(), "the retry wrote no kept reference");
}

#[test]
fn a_landed_clean_worktree_is_removed_and_its_branch_stays() {
    let landed: Landed = a_repository_with_a_worktree("landed-clean");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let sweep: LandedSweep = land_feature(&landed);
    assert!(sweep.failed.is_none(), "{:?}", sweep.failed);
    assert!(sweep.removed.is_empty(), "no removal runs on the merge's own thread: {sweep:?}");
    assert_eq!(sweep.handed_over.len(), 1, "one landed worktree waits for the commit service: {sweep:?}");
    assert!(sweep.kept.is_empty() && sweep.unproven.is_empty(), "{sweep:?}");
    assert_eq!(let_the_service_finish(&sweep), vec![Verdict::Done], "the service's own removal succeeds");
    assert!(!landed.worktree.exists(), "the worktree directory is gone");
    let branch: String = git_text(landed.main.as_path(), &["rev-parse", "--verify", "refs/heads/feature"]);
    assert_eq!(branch.len(), 40, "the branch the worktree held stays: {branch}");
    assert!(landed.root.join("main").exists(), "the repository itself stays");
}

/// What:
///  A landed worktree whose removal the fast destroy cannot take is recorded for the commit service,
///  and the merge's own sweep removes nothing and runs no Git removal.
/// Why:
///  Git's own removal reads every tracked file of the worktree and is a step that may not be run twice,
///  so a merge that ran it waited for the whole deletion:
///  the coordinator's integration passes of 2026-11-11 printed
///  `still waiting for git worktree remove ..., a bulk step ... it is a step that may not be run twice`
///  twice and ended only when the deletion did.
///  The record is what outlives the command,
///  the service of the worktree the merge ran in performs the removal,
///  and the record's state is where its outcome is read.
#[test]
fn a_landed_worktree_is_handed_to_the_commit_service_instead_of_being_removed_by_the_merge() {
    let landed: Landed = a_repository_with_a_worktree("landed-handed-over");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let sweep: LandedSweep = land_feature(&landed);
    assert!(sweep.removed.is_empty(), "no removal runs on the merge's own thread: {sweep:?}");
    assert!(sweep.kept.is_empty(), "nothing keeps the worktree, so nothing hands it to a person: {sweep:?}");
    assert_eq!(sweep.handed_over.len(), 1, "the landed worktree waits for the commit service: {sweep:?}");
    let handed: &super::HandedOverWorktree = &sweep.handed_over[0];
    assert_eq!(handed.worktree, landed.worktree);
    assert_eq!(handed.branch, "refs/heads/feature");
    assert!(landed.worktree.exists(), "the worktree is still there when the merge ends");
    let item: &crate::work_item::WorkItem = &handed.record;
    assert_eq!(item.kind, crate::work_item::WorkKind::SubvolumeDelete);
    assert_eq!(item.parent, landed.worktree.display().to_string(), "the record names the worktree to remove");
    assert_eq!(item.git_dir, landed.common, "the record is read from the queue of the worktree the merge ran in");
    assert_eq!(item.branch_ref, crate::worktree_remove_hand_over::WORKTREE_REMOVAL_REFERENCE, "the record says the removal is Git's own");
    let pending: PathBuf = crate::worktree_remove_hand_over::pending_record_path(landed.common.as_path(), item);
    assert!(pending.is_file(), "the record outlives the command: {}", pending.display());
    assert_eq!(let_the_service_finish(&sweep), vec![Verdict::Done], "the service's own removal succeeds");
    assert!(!landed.worktree.exists(), "the worktree is gone once the service has removed it");
    let branch: String = git_text(landed.main.as_path(), &["rev-parse", "--verify", "refs/heads/feature"]);
    assert_eq!(branch.len(), 40, "the branch the worktree held stays: {branch}");
}

/// What:
///  The hand-over tells the commit service of the worktree the merge ran in,
///  which is the instance whose queue the record is read from.
/// Why:
///  One instance reads each worktree's queue,
///  and a record another instance reads is refused there,
///  so a wake that named the wrong directory would leave the removal to a command nobody runs.
#[test]
fn a_hand_over_wakes_the_service_of_the_queue_it_wrote_to() {
    let landed: Landed = a_repository_with_a_worktree("landed-hand-over-wake");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let woken: std::cell::RefCell<Vec<PathBuf>> = std::cell::RefCell::new(Vec::new());
    let told = |_context: &crate::transaction_git::GitContext, git_dir: &Path| -> Result<(), String> {
        woken.borrow_mut().push(git_dir.to_path_buf());
        return Ok(());
    };
    let sweep: LandedSweep = land_feature_hearing(&landed, &told);
    assert_eq!(sweep.handed_over.len(), 1, "the landed worktree waits for the commit service: {sweep:?}");
    assert_eq!(woken.borrow().clone(), vec![landed.common.clone()], "the service of the queue the record was written to is told");
}

/// What:
///  A landed worktree holding a file Git would keep is not removed,
///  and Git's own sentence for the refusal is the answer the service records.
/// Why:
///  The removal is Git's read of every tracked and untracked file of the worktree,
///  and it is that read which keeps a person's work:
///  the merge hands the removal over instead of running it,
///  so the control reads what the service's own call came to.
#[test]
fn a_landed_worktree_with_uncommitted_work_stays() {
    let landed: Landed = a_repository_with_a_worktree("landed-dirty");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let notes: PathBuf = landed.worktree.join("notes.md");
    std::fs::write(notes.as_path(), b"what I was in the middle of\n").expect("a file the person wrote");
    let sweep: LandedSweep = land_feature(&landed);
    assert!(sweep.removed.is_empty(), "nothing is removed while the worktree holds work: {sweep:?}");
    assert_eq!(sweep.handed_over.len(), 1, "the removal is Git's, so it waits for the commit service: {sweep:?}");
    let verdicts: Vec<Verdict> = let_the_service_finish(&sweep);
    assert!(
        matches!(verdicts.as_slice(), [Verdict::Failed(words)] if words.contains("modified or untracked files")),
        "Git refuses to remove a worktree holding a file it would keep, in its own words: {verdicts:?}"
    );
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
    assert_eq!(sweep.handed_over.len(), 1, "one landed worktree waits for the commit service: {sweep:?}");
    assert_eq!(let_the_service_finish(&sweep), vec![Verdict::Done], "the service's own removal succeeds");
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
    let place: LandedPlace<'_> = place_of(&landed, &context, &registered_before, table.as_path(), landed.common.as_path());
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
    let place: LandedPlace<'_> = place_of(&landed, &context, &registered_before, table.as_path(), landed.common.as_path());
    let sweep: LandedSweep = sweep_landed_worktrees(&place, before.as_str(), after.as_str(), "main");
    assert!(sweep.removed.is_empty(), "nothing is removed while a process may be working there: {sweep:?}");
    assert_eq!(sweep.kept.len(), 1, "{sweep:?}");
    assert!(sweep.kept[0].why.contains("process 4343"), "the reason names the process: {}", sweep.kept[0].why);
    assert!(sweep.kept[0].why.contains("hides its working folder"), "{}", sweep.kept[0].why);
    assert!(landed.worktree.exists(), "the worktree stays");
}

/// What:
///  A landed worktree whose process holds a file open in it, with its working folder elsewhere, stays.
/// Why:
///  A program that opened a file in a worktree and went on, an editor or a build started with `-C` from another
///  directory, works there without a working folder in it,
///  and the working-folder read alone removed the worktree with the file open
///  (a qwen4-max-preview audit of 0.6.1-alpha.10,
/// `574063773d`,
///  finding 2).
///  The process table is written by the control, so no program on this machine decides the answer.
#[test]
fn a_landed_worktree_whose_file_is_held_open_from_elsewhere_stays() {
    let landed: Landed = a_repository_with_a_worktree("landed-open-file");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let held: PathBuf = landed.worktree.join("work.txt");
    std::fs::write(held.as_path(), b"the file the process holds\n").expect("the file the process holds open");
    let elsewhere: PathBuf = landed.root.join("elsewhere");
    std::fs::create_dir(elsewhere.as_path()).expect("a folder the process was started in");
    let table: PathBuf = a_table_with_one_process(&landed, 4343, Some(elsewhere.as_path()), 1_000, 100);
    let descriptors: PathBuf = table.join("4343").join("fd");
    std::fs::create_dir_all(descriptors.as_path()).expect("a descriptor directory");
    std::os::unix::fs::symlink(held.as_path(), descriptors.join("3")).expect("the file the process holds open");
    let registered_before: RegisteredBefore = registrations_now(&landed);
    let before: String = head_of(landed.main.as_path());
    git(landed.main.as_path(), &["merge", "--quiet", "--no-ff", "--message=land the feature", "feature"]);
    let after: String = head_of(landed.main.as_path());
    let context: crate::transaction_git::GitContext = git_context();
    let place: LandedPlace<'_> = place_of(&landed, &context, &registered_before, table.as_path(), landed.common.as_path());
    let sweep: LandedSweep = sweep_landed_worktrees(&place, before.as_str(), after.as_str(), "main");
    assert!(sweep.removed.is_empty() && sweep.handed_over.is_empty(), "nothing is removed while a process holds a file there: {sweep:?}");
    assert_eq!(sweep.kept.len(), 1, "{sweep:?}");
    assert_eq!(
        sweep.kept[0].why,
        "process 4343 has a file open in it, though its working folder is elsewhere",
        "the reason names the process and what it holds"
    );
    assert!(landed.worktree.exists(), "the worktree stays");
}

/// What:
///  A landed worktree holding an unfinished bisect stays,
/// and so does its `BISECT_LOG`.
/// Why:
///  An unfinished bisect leaves `BISECT_LOG` in the worktree's own Git directory and moves nothing else:
///  the checkout stays on its branch and every file stays clean,
///  so no other condition of the sweep notices it,
///  and real Git's own `git worktree remove` deleted the worktree and the log without a word
///  (measured with Git 2.56 on a fixture:
/// exit 0,
/// the log gone).
///  The sweep's module doc lists a bisect among the operations that keep a worktree,
///  and the marks it checked did not include the one a bisect leaves
///  (a cross-vendor audit of the landed sweep).
#[test]
fn a_landed_worktree_with_a_bisect_in_progress_stays() {
    let landed: Landed = a_repository_with_a_worktree("landed-bisect");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    git(landed.worktree.as_path(), &["bisect", "start"]);
    let bisect_log: PathBuf = admin_of(landed.worktree.as_path()).join("BISECT_LOG");
    assert!(bisect_log.exists(), "the bisect leaves its log in the worktree");
    let sweep: LandedSweep = land_feature(&landed);
    assert!(sweep.failed.is_none(), "{:?}", sweep.failed);
    assert!(sweep.removed.is_empty(), "nothing is removed while a bisect is unfinished there: {sweep:?}");
    assert_eq!(sweep.kept.len(), 1, "{sweep:?}");
    assert!(sweep.kept[0].why.contains("a bisect is unfinished there"), "the reason says what holds it: {}", sweep.kept[0].why);
    assert!(bisect_log.exists(), "the bisect log stays with its worktree");
    assert!(landed.worktree.exists(), "the worktree stays");
}

/// What:
///  A landed worktree the command itself was started in stays.
/// Why:
///  `-C` names the repository a command works on,
/// never its own working folder,
///  so a program that spawns a wrapped merge with the landed worktree as its working folder
///  left that worktree protected by no path at all,
///  and the only process in it was the command's own,
///  which the process read skipped:
///  the sweep removed the folder the program had set the command loose in
///  (a cross-vendor audit of the landed sweep).
#[test]
fn a_landed_worktree_the_command_itself_was_started_in_stays() {
    let landed: Landed = a_repository_with_a_worktree("landed-own-process");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let own: u32 = std::process::id();
    let table: PathBuf = a_table_with_one_process(&landed, own, Some(landed.worktree.as_path()), 1_000, 100);
    let registered_before: RegisteredBefore = registrations_now(&landed);
    let before: String = head_of(landed.main.as_path());
    git(landed.main.as_path(), &["merge", "--quiet", "--no-ff", "--message=land the feature", "feature"]);
    let after: String = head_of(landed.main.as_path());
    let context: crate::transaction_git::GitContext = git_context();
    let place: LandedPlace<'_> = place_of(&landed, &context, &registered_before, table.as_path(), landed.common.as_path());
    let sweep: LandedSweep = sweep_landed_worktrees(&place, before.as_str(), after.as_str(), "main");
    assert!(sweep.removed.is_empty(), "nothing is removed while the command itself works there: {sweep:?}");
    assert_eq!(sweep.kept.len(), 1, "{sweep:?}");
    assert_eq!(
        sweep.kept[0].why,
        "this command is working in it, because it was started in that folder",
        "the reason names the command itself rather than a number"
    );
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
    assert_eq!(sweep.handed_over.len(), 1, "the worktree waits for the commit service: {sweep:?}");
    assert!(sweep.kept.is_empty(), "{sweep:?}");
    assert_eq!(let_the_service_finish(&sweep), vec![Verdict::Done], "the service's own removal succeeds");
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
    assert_eq!(sweep.handed_over.len(), 1, "the worktree waits for the commit service: {sweep:?}");
    assert_eq!(let_the_service_finish(&sweep), vec![Verdict::Done], "the service's own removal succeeds");
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
    assert_eq!(sweep.handed_over.len(), 1, "the landed worktree waits for the commit service: {sweep:?}");
    assert_eq!(sweep.kept_commits.len(), 1, "the person is told about the one held commit: {sweep:?}");
    let kept: String = String::from("refs/cli-git/kept/7");
    assert!(sweep.kept_commits[0].contains(kept.as_str()), "the line names where the commit is kept: {}", sweep.kept_commits[0]);
    assert_eq!(git_text(landed.main.as_path(), &["rev-parse", kept.as_str()]), held, "the kept reference names the held commit");
    git(landed.main.as_path(), &["gc", "--prune=now"]);
    assert_eq!(git_text(landed.main.as_path(), &["cat-file", "-t", held.as_str()]), "commit", "the commit is not lost with the worktree");
}

/// What:
///  A landed worktree whose batch kept reference already names another commit keeps its held commit on the reference
///  beside it,
/// so that commit survives the removal and Git's own maintenance.
/// Why:
///  A kept reference another batch's commit is kept under is left where it is,
///  and the sweep read the collision as a line the person reads rather than as a commit nothing holds:
///  the worktree went,
/// its per-worktree reference went with it,
///  and `git gc --prune=now` then took a commit this program said it kept
///  (a cross-vendor review of the landed sweep,
///  finding 1).
#[test]
fn a_landed_worktree_whose_kept_reference_names_another_commit_keeps_its_commit_beside_it() {
    let landed: Landed = a_repository_with_a_worktree("landed-held-commit-name-taken");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let held: String = hold_a_commit_in_the_worktree(&landed, 7);
    let elsewhere: String = head_of(landed.main.as_path());
    assert_ne!(elsewhere, held, "the control keeps two different commits");
    let taken: String = String::from("refs/cli-git/kept/7");
    let beside: String = String::from("refs/cli-git/kept/7-rescue");
    git(landed.main.as_path(), &["update-ref", taken.as_str(), elsewhere.as_str()]);
    let named_before: String = git_text(landed.main.as_path(), &["for-each-ref", "--contains", held.as_str(), "--format=%(refname)"]);
    let sweep: LandedSweep = land_feature(&landed);
    git(landed.main.as_path(), &["gc", "--prune=now"]);
    let survived: bool = commit_is_there(landed.main.as_path(), held.as_str());
    assert!(
        survived,
        "the commit {held} was held by the worktree alone ({named_before:?}), the sweep handed {} worktree(s) over, and a collection then deleted it: {sweep:?}",
        sweep.handed_over.len()
    );
    let holders: String = git_text(landed.main.as_path(), &["for-each-ref", "--contains", held.as_str(), "--format=%(refname)"]);
    assert_eq!(holders.trim(), beside, "the held commit is kept on the reference beside the taken one: {sweep:?}");
    assert_eq!(git_text(landed.main.as_path(), &["rev-parse", taken.as_str()]), elsewhere, "the commit the taken name already held stays named");
    assert_eq!(sweep.handed_over.len(), 1, "the landed worktree waits for the commit service once its commit is kept: {sweep:?}");
}

/// What:
///  A landed sweep that could keep a held commit on no reference names the command that finishes the removal,
///  and that command keeps the commit before Git removes the worktree.
/// Why:
///  The refusal told the person to run the same command again,
///  and a merge whose branch already landed is up to date:
/// the retry looked at no worktree,
///  wrote no kept reference and said nothing at all,
///  so the one remedy the notice named did nothing
///  (a cross-vendor review of the landed sweep,
///  finding 2).
#[test]
fn a_refused_landed_sweep_names_a_command_that_finishes_the_removal() {
    let landed: Landed = a_repository_with_a_worktree("landed-held-commit-refusal-remedy");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let held: String = hold_a_commit_in_the_worktree(&landed, 7);
    let taken: String = String::from("refs/cli-git/kept/7");
    let beside: String = String::from("refs/cli-git/kept/7-rescue");
    for name in [taken.as_str(), beside.as_str()] {
        plant_lock(landed.common.as_path(), name);
    }
    let sweep: LandedSweep = land_feature(&landed);
    assert!(sweep.failed.is_some(), "neither kept reference can be written, so the sweep refuses: {sweep:?}");
    assert!(landed.worktree.exists(), "the refusal leaves the worktree and its commit where they are");
    let notice: String = crate::worktree_landed_sweep_notice::landed_sweep_lines(&sweep).join("\n");
    println!("{notice}");
    let quoted: String = crate::shell_quote::shell_quote(landed.worktree.as_os_str().as_encoded_bytes());
    let finishes: String = format!("git worktree remove {quoted}");
    assert!(notice.contains(finishes.as_str()), "the notice names the command that finishes the removal, `{finishes}`: {notice}");
    assert!(notice.contains("up to date"), "the notice says a merge run again removes nothing: {notice}");
    // The remedy the notice names, run as the removal route runs it: the locks go, the records are snapshotted, the commit is
    //  kept, and real Git removes the worktree.
    for name in [taken.as_str(), beside.as_str()] {
        std::fs::remove_file(landed.common.join(format!("{name}.lock"))).expect("the lock nothing holds is removed");
    }
    let context: crate::transaction_git::GitContext = git_context();
    crate::worktree_removal_rescue::snapshot_linked_worktrees(landed.common.as_path());
    let kept: crate::worktree_removal_held_commits::KeptCommits = crate::worktree_removal_held_commits::keep_held_commits(landed.common.as_path(), &context);
    assert!(kept.refused.is_empty(), "the reference can be written again: {:?}", kept.refused);
    let linked: String = landed.worktree.display().to_string();
    git(landed.main.as_path(), &["worktree", "remove", linked.as_str()]);
    assert!(!landed.worktree.exists(), "the command the notice names removes the worktree");
    git(landed.main.as_path(), &["gc", "--prune=now"]);
    assert!(commit_is_there(landed.main.as_path(), held.as_str()), "the commit is kept before the worktree goes");
    assert_eq!(git_text(landed.main.as_path(), &["rev-parse", taken.as_str()]), held, "the kept reference names the held commit");
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
    let place: LandedPlace<'_> = place_of(&landed, &context, &registered_before, table.as_path(), landed.common.as_path());
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
    assert_eq!(sweep.handed_over.len(), 1, "only the branch the merge landed is handed over: {sweep:?}");
    assert_eq!(sweep.handed_over[0].worktree, landed.worktree);
    assert!(older.exists(), "a branch that sat on an older commit is no work this merge brought in");
}

/// The path of the host's Git,
/// named so a control can say which Git it used.
#[test]
fn the_fixture_git_is_the_host_git() {
    assert!(Path::new(REAL_GIT).exists(), "{REAL_GIT}");
}
