//! Controls for the choice of rung:
//! a plan on an eligible source claims a stage and runs Git without a checkout;
//! a plan on
//!  any other source prepares an empty subvolume;
//! no plan prepares nothing;
//! and a failed Git is settled quietly.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::fs_kind::{FilesystemKind, filesystem_kind, is_subvolume_root};
use crate::test_support::{fixture, git_context, remove, repository};
use crate::worktree_copy_stage_claim::stage_claims_exist;
use crate::worktree_ladder_test_support::{commit_all, main_identity, plan_to, subvolume_repository, tracked, tracked_sparse};

/// A failed Git run,
/// as the lifecycle reports it.
const FAILED: GitStatus = GitStatus { code: Some(128) };

/// A runner that must not be called.
fn no_runner() -> impl FnMut(&GitStep<'_>) -> Result<GitStatus, crate::worktree_copy_error::WorktreeCopyError> {
    return |_step| { panic!("no Git run is expected"); };
}

/// Whether a file on this fixture root can be reflinked,
/// said once as a control that did not run when it cannot.
fn can_clone_a_file(root: &Path) -> bool {
    let from: PathBuf = root.join("probe-from");
    let to: PathBuf = root.join("probe-to");
    std::fs::write(from.as_path(), b"probe\n").expect("the probe's file");
    std::fs::write(to.as_path(), b"").expect("the probe's clone");
    let opened = (std::fs::File::open(from.as_path()), std::fs::OpenOptions::new().write(true).open(to.as_path()));
    let cloned: bool = match opened {
        (Ok(source), Ok(destination)) => crate::fs_clone_exchange::clone_file(&source, &destination, to.as_path()).is_ok(),
        _unreadable => false,
    };
    let _removed = (std::fs::remove_file(from.as_path()), std::fs::remove_file(to.as_path()));
    if !cloned {
        eprintln!("CLONE CONTROL NOT RUN: a dirty source's routing (the fixture root cannot reflink a file)");
    }
    return cloned;
}

/// What:
///  Every tracked file of `worktree` with its bytes and its executable bit,
/// in path order.
/// Why:
///   Byte equality with Git's own checkout is the promise both size rules make,
/// and a name list alone would
///  miss the bytes a conversion or a clone could change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const content = git('ls-files', '-z').split('\0').map((name) => [name, read(name), isExecutable(name)]);
/// ```
fn tracked_content_of(worktree: &Path) -> Vec<(String, Vec<u8>, bool)> {
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::PermissionsExt;
    let listing: Vec<u8> = crate::test_support::git_output(worktree, &["ls-files", "-z"]).stdout;
    let mut files: Vec<(String, Vec<u8>, bool)> = Vec::new();
    for name in listing.split(|byte: &u8| { return *byte == 0; }).filter(|name| { return !name.is_empty(); }) {
        let path: PathBuf = worktree.join(std::ffi::OsStr::from_bytes(name));
        let executable: bool = std::fs::symlink_metadata(path.as_path()).expect("the file's own metadata").permissions().mode() & 0o111 != 0;
        files.push((String::from_utf8_lossy(name).into_owned(), std::fs::read(path.as_path()).expect("the file's bytes"), executable));
    }
    return files;
}

/// What:
///  Add a worktree through Git's own checkout from `repo`,
/// the way the person's run does once the ladder has chosen the Git rung.
/// Why:
///   A declined add leaves the worktree to Git,
/// so the control runs exactly the run the person's command makes
///  and compares its bytes with a second one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('adds through Git\'s own checkout', () => run(['worktree', 'add', '--quiet', '-b', name, '../' + name]));
/// ```
fn gits_own_worktree(repo: &Path, name: &str, destination: &Path) -> PathBuf {
    crate::test_support::git(repo, &["worktree", "add", "--quiet", "-b", name, destination.to_str().expect("a UTF-8 destination")]);
    return destination.to_path_buf();
}

/// Without a plan the ladder asks for the person's own run and settles to Git's status.
#[test]
fn no_plan_means_the_persons_run() {
    let ladder: Ladder = begin_ladder(None, &git_context(), &WorktreeIdentity::OutsideWorktree);
    assert_eq!(ladder.first_step(), GitStep::Person);
    let mut runner = no_runner();
    let result: LadderResult = ladder.finish(GitStatus { code: Some(0) }, &mut runner, &git_context(), &mut std::io::sink(), &mut std::io::sink());
    assert_eq!(result.git, GitStatus { code: Some(0) });
    assert_eq!(result.carried, None);
    assert_eq!(result.placed, None, "no rung put an object at the destination");
    assert!(result.claim.is_none(), "no plan means no record");
}

/// A source neither rung serves gets an empty subvolume at the destination,
/// removed again when Git failed.
/// What:
///  The source is an ordinary directory on btrfs in the middle of a merge,
/// so the snapshot rung refuses it for its filesystem and
///  both rungs refuse it for the unfinished operation,
///  and the reason the person reads is the structural one.
/// Why:
///  An untracked file was this fixture's source while a source with changes ended the clone rung;
/// a source with changes
///  now clones the paths its proof settles,
///  so the fixture needs a state no rung can serve
///  (`doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`).
#[test]
fn btrfs_a_refused_source_gets_a_subvolume_that_a_failed_git_leaves_no_trace_of() {
    let Some(root) = btrfs_fixture("run-bootstrap") else { return };
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "ordinary")).expect("canonical");
    let head: String = String::from_utf8_lossy(&crate::test_support::git(repo.as_path(), &["rev-parse", "HEAD"]).stdout).trim().to_string();
    std::fs::write(repo.join(".git/MERGE_HEAD"), format!("{head}\n")).expect("an unfinished merge rules every source out");
    let plan = plan_to(root.join("topic"));
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    assert_eq!(ladder.first_step(), GitStep::Person);
    assert!(is_subvolume_root(plan.destination.as_path()).expect("probe"));
    assert_eq!(ladder.slow.as_ref().map(|route| { return route.word(); }), Some("not-subvolume"), "an ordinary directory on btrfs is told to be converted: {:?}", ladder.slow);
    let mut runner = no_runner();
    let result: LadderResult = ladder.finish(FAILED, &mut runner, &git_context(), &mut std::io::sink(), &mut std::io::sink());
    assert_eq!(result.git, FAILED);
    assert_eq!(result.carried, None);
    assert_eq!(result.placed, None);
    assert!(result.claim.is_none(), "Git never ran, so no rung took a claim");
    assert!(!plan.destination.exists());
    remove(root.as_path());
}

/// A clean source that is an ordinary directory gets the clone rung:
/// Git runs without a checkout,
/// into an empty subvolume.
#[test]
fn btrfs_a_clean_ordinary_source_gets_the_clone_rung_and_a_subvolume() {
    let Some(root) = btrfs_fixture("run-clone") else { return };
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "ordinary")).expect("canonical");
    // Above the clone rung's size limit, which declines a tree of too few bytes before this rung is chosen.
    tracked_sparse(repo.as_path(), "bulk/kept.bin", crate::worktree_clone_rung_limits::THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG + 1_000);
    commit_all(repo.as_path(), "a tree above the size limit");
    let plan = plan_to(root.join("topic"));
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    assert_eq!(ladder.first_step(), GitStep::WithoutCheckout(plan.insert_at));
    assert!(is_subvolume_root(plan.destination.as_path()).expect("probe"));
    assert!(stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the clone rung records the add durably, so a killed add can be removed");
    assert!(!plan.destination.with_file_name(".cli-git-worktree-copy-stage").exists(), "the clone rung stages nothing");
    let mut runner = no_runner();
    let result: LadderResult = ladder.finish(FAILED, &mut runner, &git_context(), &mut std::io::sink(), &mut std::io::sink());
    assert_eq!(result.git, FAILED);
    assert_eq!(result.carried, None);
    assert_eq!(result.placed, None);
    assert!(!plan.destination.exists(), "the empty subvolume is removed after a failed Git");
    assert!(result.claim.is_some(), "the record of the add is handed to the lifecycle rather than dropped");
    assert!(stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the record stands until its owner settles it");
    result.claim.expect("the record").settle().expect("settled");
    remove(root.as_path());
}

/// An eligible source claims its stage name and runs Git without a checkout;
/// a failed Git releases the claim and leaves
///  the destination as Git left it.
#[test]
fn btrfs_an_eligible_source_runs_git_without_a_checkout_and_a_failure_releases_the_claim() {
    let Some(root) = btrfs_fixture("run-snapshot") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let plan = plan_to(root.join("topic"));
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    assert_eq!(ladder.first_step(), GitStep::WithoutCheckout(plan.insert_at));
    assert_eq!(ladder.slow, None, "a snapshot has nothing to explain");
    assert!(!plan.destination.exists(), "a snapshot rung makes no subvolume before Git");
    assert!(stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the stage name is claimed durably");
    let mut runner = no_runner();
    let result: LadderResult = ladder.finish(FAILED, &mut runner, &git_context(), &mut std::io::sink(), &mut std::io::sink());
    assert_eq!(result.git, FAILED);
    assert_eq!(result.carried, None);
    assert_eq!(result.placed, None);
    assert!(result.claim.is_none(), "a Git that failed before the snapshot released the claim");
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the claim is released");
    remove(root.as_path());
}

/// A subvolume main checkout with an uncommitted change gets the snapshot rung with its list of changes and nothing to explain;
///  the same checkout in the middle of a merge keeps its reason for the person and gets Git's checkout.
#[test]
fn btrfs_a_dirty_subvolume_source_is_snapshotted_and_a_merging_one_keeps_its_reason() {
    let Some(root) = btrfs_fixture("run-dirty-reason") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    std::fs::write(repo.join("shared.txt"), b"edited\n").expect("edit");
    let plan = plan_to(root.join("topic"));
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    assert_eq!(ladder.first_step(), GitStep::WithoutCheckout(plan.insert_at), "a snapshot serves a source with a change");
    assert_eq!(ladder.slow, None, "a snapshot has nothing to explain");
    let Rung::Snapshot(source, _claim, _fence) = &ladder.rung else { panic!("the snapshot rung is expected") };
    let changed = source.dirty.as_ref().expect("the changes are listed");
    assert_eq!(changed.first.as_deref(), Some("shared.txt"));
    let mut runner = no_runner();
    ladder.finish(FAILED, &mut runner, &git_context(), &mut std::io::sink(), &mut std::io::sink());
    let head: String = String::from_utf8_lossy(&crate::test_support::git(repo.as_path(), &["rev-parse", "HEAD"]).stdout).trim().to_string();
    std::fs::write(repo.join(".git/MERGE_HEAD"), format!("{head}\n")).expect("merge mark");
    let merging: Ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    assert_eq!(merging.first_step(), GitStep::Person, "an unfinished merge stays a refusal");
    let line: String = merging.slow.as_ref().expect("a reason").line();
    assert!(line.contains("MERGE_HEAD"), "{line}");
    let mut second = no_runner();
    merging.finish(FAILED, &mut second, &git_context(), &mut std::io::sink(), &mut std::io::sink());
    remove(root.as_path());
}

/// A stage is destroyed only while the name still holds the object this process made:
/// another directory at the name,
/// and a
///  stage that was never identified,
/// are left in place.
#[test]
fn btrfs_a_stage_is_discarded_only_while_it_is_the_one_that_was_made() {
    let Some(root) = btrfs_fixture("run-discard-identity") else { return };
    let base: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical");
    let made = |name: &str| -> (PathBuf, crate::fs_beneath::ObjectId) {
        let stage: PathBuf = base.join(name);
        crate::fs_btrfs_ioctl::subvolume_create_at(stage.as_path()).expect("stage");
        std::fs::write(stage.join("file"), b"x").expect("content");
        let parent = crate::fs_beneath::open_root(base.as_path()).expect("parent");
        let id = crate::fs_beneath::identity_at(&parent, std::ffi::OsStr::new(name)).expect("identity");
        return (stage, id);
    };
    let (stage, id) = made("stage-one");
    discard_stage(stage.as_path(), Some(id));
    assert!(!stage.exists(), "the stage that was made goes");
    let (other, id_two) = made("stage-two");
    std::fs::create_dir(base.join("stage-three")).expect("somebody's directory");
    std::fs::write(base.join("stage-three/theirs"), b"theirs").expect("their file");
    discard_stage(base.join("stage-three").as_path(), Some(id_two));
    assert!(base.join("stage-three/theirs").exists(), "a directory that is not the one made stays");
    discard_stage(other.as_path(), None);
    assert!(other.join("file").exists(), "an unidentified stage stays");
    remove(root.as_path());
}

/// What:
///  A source whose attributes convert more than a quarter of its paths is declined before the add's record is claimed,
///  so no copy and no snapshot is paid for an add that Git's own checkout will write.
/// Why:
///   The decline needs `git check-attr` and the count over its answer and nothing else,
/// so deciding it after the copies made a declined add pay the copy and then Git's whole checkout over it
///  (the 0.644 fixture measured 1.67 to 1.75 of `main`,
/// `doc/planning/cli-git-clone-rung-conversion-fence.md`,
///  "What it costs").
/// The count is one past the floor and a quarter,
/// so the fixture is the smallest that reaches the rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ladder = beginLadder(plan, context, identity);
/// expect(ladder.rung).toBe('git'); expect(ladder.slow.word).toBe('declines'); expect(claims()).toBe(0);
/// ```
#[test]
fn a_converting_source_is_declined_before_anything_is_claimed_or_copied() {
    let root: PathBuf = crate::test_support::fixture("run-decline-before-the-copy");
    let repo: PathBuf = repository(root.as_path(), "repo");
    crate::worktree_ladder_test_support::tracked(repo.as_path(), ".gitattributes", b"*.txt text eol=crlf\n");
    // One path past the floor and more than a quarter of the tracked files.
    for index in 0..=crate::worktree_clone_conversion_fence::DECLINE_PATHS_FLOOR {
        crate::worktree_ladder_test_support::tracked(repo.as_path(), format!("file-{index}.txt").as_str(), b"line\n");
    }
    // Above the clone rung's size limit, so the fence is the limit this shape reaches and not the size.
    tracked_sparse(repo.as_path(), "bulk/kept.bin", crate::worktree_clone_rung_limits::THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG + 1_000);
    crate::worktree_ladder_test_support::commit_all(repo.as_path(), "fixture");
    let plan = plan_to(root.join("topic"));
    let identity = main_identity(repo.as_path());
    // The fence is only reached where a fast rung applies: a destination whose filesystem cannot clone refuses the clone
    // rung before the fence runs, and the gate image's fixture root is a tmpfs, so this control names its own skip there.
    if let Some(reason) = crate::worktree_snapshot_source::clone_refusal(&identity, &plan) {
        eprintln!("CLONE RUNG NOT RUN: a_converting_source_is_declined_before_anything_is_claimed_or_copied ({reason})");
        remove(root.as_path());
        return;
    }
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &identity);
    assert_eq!(ladder.first_step(), GitStep::Person, "the person's own run checks the worktree out");
    assert!(matches!(ladder.rung, Rung::Git(_)), "a source above the quarter takes Git's checkout");
    assert_eq!(ladder.slow.as_ref().map(|route| { return route.word(); }), Some("declines"), "{:?}", ladder.slow);
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "nothing was claimed, so nothing can have been copied into or out of a stage");
    // The Git rung prepares an empty subvolume at the destination when the filesystem allows one, so what says nothing was copied is that it holds no entry.
    let prepared: usize = if plan.destination.exists() { std::fs::read_dir(plan.destination.as_path()).expect("the destination").count() } else { 0 };
    assert_eq!(prepared, 0, "the rung copied nothing into the destination");
    remove(root.as_path());
}

/// A source with uncommitted changes takes the clone rung and runs Git without a checkout,
/// where the filesystem clones its files:
/// the change is no longer the refusal that sent the add to Git's whole checkout.
#[test]
fn a_dirty_source_gets_the_clone_rung_where_files_can_be_cloned() {
    let root: PathBuf = fixture("run-dirty-clone");
    if !can_clone_a_file(root.as_path()) {
        return;
    }
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "ordinary")).expect("canonical");
    tracked(repo.as_path(), "kept.txt", b"the commit's bytes\n");
    // Above the clone rung's size limit and with enough tracked paths that the changed-file cut does not take this one
    //  change, so the change is what brings the source to the clone rung and neither limit is the reason.
    tracked_sparse(repo.as_path(), "bulk/kept.bin", crate::worktree_clone_rung_limits::THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG + 1_000);
    for index in 0..crate::worktree_clone_conversion_fence::DECLINE_SHARE_DIVISOR {
        tracked(repo.as_path(), format!("plain-{index}.txt").as_str(), b"the commit's bytes\n");
    }
    commit_all(repo.as_path(), "the commit the person edits");
    std::fs::write(repo.join("kept.txt"), b"a change nobody committed\n").expect("the person's change");
    let plan = plan_to(root.join("topic"));
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    assert_eq!(ladder.first_step(), GitStep::WithoutCheckout(plan.insert_at), "a dirty source clones rather than taking Git's checkout");
    let slow: String = format!("{:?}", ladder.slow);
    assert!(slow.contains("subvolume"), "the reason the snapshot is not taken is the filesystem's, not the source's change: {slow}");
    assert!(!slow.contains("uncommitted"), "the source's change is nothing to explain any more: {slow}");
    assert!(stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the clone rung records the add durably");
    let mut runner = no_runner();
    let result: LadderResult = ladder.finish(FAILED, &mut runner, &git_context(), &mut std::io::sink(), &mut std::io::sink());
    assert_eq!(result.git, FAILED);
    assert!(result.claim.is_some(), "the record of the add is handed to the lifecycle rather than dropped");
    result.claim.expect("the record").settle().expect("settled");
    remove(root.as_path());
}

/// What:
///  An add at a commit whose tree lacks a path the attributes convert is skipped before the add's record is claimed,
///  so no copy is paid for a worktree that Git's own checkout writes.
/// Why:
///   The index this rung installs holds no stat data for a path the fence hands to Git,
/// and the tree difference refuses such an entry,
///  so the rung would end after its copies and pay Git's whole checkout over them,
///  as the "Limitations" section of
///  `doc/planning/cli-git-clone-rung-conversion-fence.md` says.
/// The attributes are read before the claim,
///  so the fence's own answer and one name-only read settle it here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ladder = beginLadder(planAt('other'), context, identity);
/// expect(ladder.rung).toBe('git'); expect(ladder.slow.line()).toContain('extra.txt'); expect(claims()).toBe(0);
/// ```
#[test]
fn a_target_tree_that_changes_a_fenced_path_is_skipped_before_anything_is_claimed_or_copied() {
    let root: PathBuf = crate::test_support::fixture("run-target-tree-skip");
    let repo: PathBuf = repository(root.as_path(), "repo");
    crate::worktree_ladder_test_support::tracked(repo.as_path(), ".gitattributes", b"*.txt text eol=crlf\n");
    crate::worktree_ladder_test_support::tracked(repo.as_path(), "a.txt", b"line one\nline two\n");
    crate::worktree_ladder_test_support::commit_all(repo.as_path(), "conversion");
    crate::test_support::git(repo.as_path(), &["update-ref", "refs/heads/other", "HEAD"]);
    // Above the clone rung's size limit: a tree of too few bytes is declined by the size before the skip is asked, and this
    //  control is about the skip the attributes decide.
    tracked_sparse(repo.as_path(), "bulk/kept.bin", crate::worktree_clone_rung_limits::THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG + 1_000);
    crate::worktree_ladder_test_support::tracked(repo.as_path(), "extra.txt", b"line one\nline two\n");
    crate::worktree_ladder_test_support::commit_all(repo.as_path(), "second");
    let plan = crate::worktree_ladder_test_support::plan_to_at(root.join("topic"), Some("other"));
    let identity = main_identity(repo.as_path());
    // A destination whose filesystem cannot clone refuses the clone rung before the fence runs, so this control names its
    // own skip there, as the decline control beside it does.
    if let Some(reason) = crate::worktree_snapshot_source::clone_refusal(&identity, &plan) {
        eprintln!("CLONE RUNG NOT RUN: a_target_tree_that_changes_a_fenced_path_is_skipped_before_anything_is_claimed_or_copied ({reason})");
        remove(root.as_path());
        return;
    }
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &identity);
    assert_eq!(ladder.first_step(), GitStep::Person, "Git's own checkout writes the worktree");
    assert!(matches!(ladder.rung, Rung::Git(_)), "the rung steps aside before the copies");
    let route: &SlowRoute = ladder.slow.as_ref().expect("the sentence that says why");
    assert_eq!(route.word(), "declines", "{route:?}");
    assert!(route.line().contains("differs from the source's index at extra.txt"), "{}", route.line());
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "nothing was claimed, so nothing can have been copied into or out of a stage");
    let prepared: usize = if plan.destination.exists() { std::fs::read_dir(plan.destination.as_path()).expect("the destination").count() } else { 0 };
    assert_eq!(prepared, 0, "the rung copied nothing into the destination");
    remove(root.as_path());
}

/// What:
///  A source below the size limit takes Git's own checkout,
/// says which sum and which limit declined it,
/// claims nothing,
/// and the add's worktree equals Git's own checkout byte for byte.
/// Why:
///   The size limit exists because a clone of too little data costs more than Git's write of it;
/// a
///  control that only checked the reason would stay green if the limit moved to Git's checkout for another
/// reason,
///  and one that only checked the route would not show that the person gets Git's result.
/// Removing the limit makes the rung clone the fixture and the worktree holds a shared extent,
///  which this control's byte comparison alone would not catch,
///  so the clone-capable skip and the extent are what make it red.
/// The limit is chosen by the filesystem,
///  so this control runs where the fixture's filesystem has the rule,
///  and `a_source_below_the_size_limit_is_cloned_where_the_filesystem_has_no_whole_tree_rule` is its other half
///  (`the_whole_tree_limit_is_the_subject_filesystems_own_measurement` pins both).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(ladder.rung).toBe('git'); expect(ladder.slow.word).toBe('declines');
/// expect(trackedContent(topic)).toEqual(trackedContent(control));
/// ```
#[test]
fn a_source_below_the_size_limit_gets_gits_own_checkout_and_its_bytes() {
    let root: PathBuf = fixture("run-size-limit");
    if !can_clone_a_file(root.as_path()) {
        remove(root.as_path());
        return;
    }
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "repo")).expect("canonical");
    let kind: FilesystemKind = filesystem_kind(repo.as_path()).expect("the fixture's filesystem");
    if crate::worktree_clone_rung_limits::the_fewest_tracked_bytes_that_keep_the_clone_rung(kind).is_none() {
        // A filesystem with no whole-tree rule clones this source, and
        // `a_source_below_the_size_limit_is_cloned_where_the_filesystem_has_no_whole_tree_rule` is that control.
        remove(root.as_path());
        return;
    }
    tracked(repo.as_path(), "kept.txt", b"the commit's bytes\n");
    tracked(repo.as_path(), "edited.txt", b"the commit's bytes\n");
    // Enough tracked paths that the changed-file cut does not take this one change, so the size is the reason.
    for index in 0..crate::worktree_clone_conversion_fence::DECLINE_SHARE_DIVISOR {
        tracked(repo.as_path(), format!("plain-{index}.txt").as_str(), b"the commit's bytes\n");
    }
    commit_all(repo.as_path(), "a tree far below the limit");
    std::fs::write(repo.join("edited.txt"), b"a change nobody committed\n").expect("the person's change");
    let plan: AddPlan = plan_to(root.join("topic"));
    let identity: WorktreeIdentity = main_identity(repo.as_path());
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &identity);
    assert_eq!(ladder.first_step(), GitStep::Person, "Git's own checkout writes the worktree");
    assert!(matches!(ladder.rung, Rung::Git(_)), "a tree below the size limit takes Git's checkout");
    let route: &SlowRoute = ladder.slow.as_ref().expect("the sentence that says why");
    assert_eq!(route.word(), "declines", "{route:?}");
    assert!(route.line().contains("would copy"), "the sentence names the bytes the clones would carry: {}", route.line());
    assert!(route.line().contains(crate::worktree_clone_rung_limits::THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG.to_string().as_str()), "and the limit it is below: {}", route.line());
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "nothing was claimed");
    let prepared: usize = if plan.destination.exists() { std::fs::read_dir(plan.destination.as_path()).expect("the destination").count() } else { 0 };
    assert_eq!(prepared, 0, "the rung copied nothing into the destination");
    let topic: PathBuf = gits_own_worktree(repo.as_path(), "topic", plan.destination.as_path());
    let control: PathBuf = gits_own_worktree(repo.as_path(), "control", root.join("control").as_path());
    assert_eq!(tracked_content_of(topic.as_path()), tracked_content_of(control.as_path()), "the add's worktree is Git's own checkout");
    assert_eq!(std::fs::read(topic.join("edited.txt")).expect("the commit's bytes"), b"the commit's bytes\n", "the person's change reaches no worktree");
    assert_eq!(crate::test_support::git_text(topic.as_path(), &["status", "--porcelain=v1"]), "", "the first status of the worktree is clean");
    remove(root.as_path());
}

/// What:
///  A source below the size limit gets the clone rung on a filesystem whose whole-tree rule does not apply,
///  runs Git without a checkout,
///  and declines nothing.
/// Why:
///   The guest round that measured the dirty shapes on btrfs found the clone no slower than Git's own checkout on
///   every shape it ran,
///   so the project dropped the limit there
///   (`doc/planning/cli-git-clone-rung-dirty-shapes-on-btrfs-measured.md`).
/// The fixture is the one `a_source_below_the_size_limit_gets_gits_own_checkout_and_its_bytes` declines,
/// so this control is red without the change:
/// it expects the clone rung where that control expects Git's checkout,
/// and no sentence,
/// because a cloned add has nothing to decline
/// (`worktree_slow_route::explain_decline`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(ladder.rung).toBe('clone'); expect(ladder.slow).toBeUndefined();
/// ```
#[test]
fn a_source_below_the_size_limit_is_cloned_where_the_filesystem_has_no_whole_tree_rule() {
    let root: PathBuf = fixture("run-size-limit-without-a-rule");
    if !can_clone_a_file(root.as_path()) {
        remove(root.as_path());
        return;
    }
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "repo")).expect("canonical");
    let kind: FilesystemKind = filesystem_kind(repo.as_path()).expect("the fixture's filesystem");
    if crate::worktree_clone_rung_limits::the_fewest_tracked_bytes_that_keep_the_clone_rung(kind).is_some() {
        // A filesystem with the whole-tree rule declines this source, and
        // `a_source_below_the_size_limit_gets_gits_own_checkout_and_its_bytes` is that control.
        remove(root.as_path());
        return;
    }
    tracked(repo.as_path(), "kept.txt", b"the commit's bytes\n");
    tracked(repo.as_path(), "edited.txt", b"the commit's bytes\n");
    // Many more tracked paths than changed ones, so the changed-file cut takes none of this one change.
    for index in 0..crate::worktree_clone_conversion_fence::DECLINE_SHARE_DIVISOR {
        tracked(repo.as_path(), format!("plain-{index}.txt").as_str(), b"the commit's bytes\n");
    }
    commit_all(repo.as_path(), "a tree far below the limit");
    std::fs::write(repo.join("edited.txt"), b"a change nobody committed\n").expect("the person's change");
    let plan: AddPlan = plan_to(root.join("topic"));
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    assert_eq!(ladder.first_step(), GitStep::WithoutCheckout(plan.insert_at), "the clone rung runs Git without a checkout on a tree this small");
    assert!(matches!(ladder.rung, Rung::Clone(..)), "the rung is the clone rung");
    assert!(ladder.slow.is_none(), "a cloned add declines nothing, so no sentence says a rule applied: {:?}", ladder.slow);
    assert!(stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the clone rung records the add durably");
    let mut runner = no_runner();
    let result: LadderResult = ladder.finish(FAILED, &mut runner, &git_context(), &mut std::io::sink(), &mut std::io::sink());
    assert_eq!(result.git, FAILED);
    assert!(result.claim.is_some(), "the record of the add is handed to the lifecycle rather than dropped");
    result.claim.expect("the record").settle().expect("settled");
    remove(root.as_path());
}

/// What:
///  A source above the size limit with more changed paths than the cut takes Git's own checkout,
/// says how many of how many changed,
/// claims nothing,
/// and the add's worktree equals Git's own checkout byte for byte.
/// Why:
///   The cut exists because the write step costs one run a path,
/// so the shape must be above the size limit or the
///  size limit would be the one that declined it and this control would stay green with the cut removed;
/// the
///  fixture's files are sparse,
///  so being above the limit costs it no disk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(ladder.rung).toBe('git'); expect(ladder.slow.line()).toContain('have changes');
/// expect(trackedContent(topic)).toEqual(trackedContent(control));
/// ```
#[test]
fn a_source_above_the_changed_file_cut_gets_gits_own_checkout_and_its_bytes() {
    let root: PathBuf = fixture("run-changed-file-cut");
    if !can_clone_a_file(root.as_path()) {
        remove(root.as_path());
        return;
    }
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "repo")).expect("canonical");
    // Five tracked files, together just above the size limit, so the cut is the limit this shape reaches.
    let files: usize = 5;
    let per_file: u64 = crate::worktree_clone_rung_limits::THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG / files as u64 + 1_000;
    for index in 0..files {
        tracked_sparse(repo.as_path(), format!("bulk/{index}.bin").as_str(), per_file);
    }
    commit_all(repo.as_path(), "a tree above the size limit");
    let cut: usize = crate::worktree_clone_rung_limits::the_changed_file_cut(files);
    // One path more than the cut takes, and the cut is the share of the tracked paths the fence declines at.
    for index in 0..=cut {
        let path: PathBuf = repo.join(format!("bulk/{index}.bin").as_str());
        let mut file: std::fs::File = std::fs::OpenOptions::new().append(true).open(path.as_path()).expect("the file");
        std::io::Write::write_all(&mut file, b"one changed byte\n").expect("the change");
    }
    let plan: AddPlan = plan_to(root.join("topic"));
    let identity: WorktreeIdentity = main_identity(repo.as_path());
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &identity);
    assert_eq!(ladder.first_step(), GitStep::Person, "Git's own checkout writes the worktree");
    assert!(matches!(ladder.rung, Rung::Git(_)), "a source above the cut takes Git's checkout");
    let route: &SlowRoute = ladder.slow.as_ref().expect("the sentence that says why");
    assert_eq!(route.word(), "declines", "{route:?}");
    assert!(route.line().contains("have changes"), "the sentence names the changes it found: {}", route.line());
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "nothing was claimed");
    let prepared: usize = if plan.destination.exists() { std::fs::read_dir(plan.destination.as_path()).expect("the destination").count() } else { 0 };
    assert_eq!(prepared, 0, "the rung copied nothing into the destination");
    let topic: PathBuf = gits_own_worktree(repo.as_path(), "topic", plan.destination.as_path());
    let control: PathBuf = gits_own_worktree(repo.as_path(), "control", root.join("control").as_path());
    assert_eq!(tracked_content_of(topic.as_path()), tracked_content_of(control.as_path()), "the add's worktree is Git's own checkout");
    assert_eq!(crate::test_support::git_text(topic.as_path(), &["status", "--porcelain=v1"]), "", "the first status of the worktree is clean");
    remove(root.as_path());
}
