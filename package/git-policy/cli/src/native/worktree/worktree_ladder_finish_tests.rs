//! Controls for the steps after real Git returned:
//! the snapshot rung serves a clean source with or without a tree
//!  difference and prints Git's line;
//! every refusal destroys the stage and completes the worktree with `reset --hard`;
//!  `--quiet` is honored on both;
//! and no step runs a hook.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::fs_kind::is_subvolume_root;
use crate::private_storage::take_file_syncs;
use crate::test_support::{REAL_GIT, fixture, git, git_context, git_text, remove, repository};
use crate::worktree_copy::GitStep;
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_ladder_run::begin_ladder;
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_add_binding::Bound;
use crate::worktree_copy_journal::journal_root;
use crate::test_support::git_output;
use crate::worktree_ladder_test_support::{OLDER_TARGET_OLD_PATHS, commit_all, main_identity, paths_below, plan_to, plan_to_at, subvolume_repository, tracked, tracked_sparse};
use std::cell::RefCell;
use std::process::{Command, Stdio};

/// Every step the runner was asked for,
/// as the words of its arguments.
type Steps = RefCell<Vec<Vec<String>>>;

/// A runner that records each step and runs the commands of `Other` steps with real Git in an isolated environment.
fn recording_runner(steps: &Steps, fail_reset: bool) -> impl FnMut(&GitStep<'_>) -> Result<GitStatus, WorktreeCopyError> + '_ {
    return move |step| {
        let GitStep::Other { arguments, environment } = step else { panic!("only the ladder's own steps are expected") };
        steps.borrow_mut().push(arguments.iter().map(|word: &OsString| { return word.to_string_lossy().into_owned(); }).collect());
        if fail_reset && arguments.iter().any(|word: &OsString| { return word == "reset"; }) {
            return Ok(GitStatus { code: Some(1) });
        }
        let status = Command::new(REAL_GIT)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
            .env("GIT_AUTHOR_NAME", "Fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
            .env("GIT_COMMITTER_NAME", "Fixture")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
            .envs(environment.iter().cloned())
            .args(arguments.iter())
            .stdin(Stdio::null())
            .status()
            .expect("Git starts");
        return Ok(GitStatus { code: status.code() });
    };
}

/// Run Git without a checkout the way the lifecycle does,
/// then the ladder's `finish`;
/// return the result,
/// the steps and
///  what was printed.
fn add_through_the_ladder(repo: &Path, root: &Path, extra: &[&str], fail_reset: bool, spoil: impl FnOnce(&Path)) -> (LadderResult, Vec<Vec<String>>, String) {
    return add_through_the_ladder_at(repo, root, None, extra, fail_reset, spoil);
}

/// `add_through_the_ladder` with the commit the add targets named,
/// so a fixture can add a worktree at a tree that differs from the source's `HEAD`.
/// The plan names that commit too,
/// because the words this helper gives Git name it (`-b topic ../topic <commit>`):
/// a plan without it would be the in-process twin of an add whose commit-ish the skip cannot read,
/// which none of these controls means to make.
fn add_through_the_ladder_at(repo: &Path, root: &Path, target: Option<&str>, extra: &[&str], fail_reset: bool, spoil: impl FnOnce(&Path)) -> (LadderResult, Vec<Vec<String>>, String) {
    let destination: PathBuf = root.join("topic");
    let mut plan = plan_to_at(destination.clone(), target);
    plan.quiet = extra.contains(&"--quiet");
    let ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo));
    assert_eq!(ladder.first_step(), GitStep::WithoutCheckout(plan.insert_at), "the source is eligible");
    let mut words: Vec<&str> = vec!["worktree", "add", "--no-checkout"];
    words.extend_from_slice(extra);
    words.extend(["-b", "topic", "../topic"]);
    if let Some(commit) = target {
        words.push(commit);
    }
    git(repo, words.as_slice());
    spoil(destination.as_path());
    let steps: Steps = RefCell::new(Vec::new());
    let mut printed: Vec<u8> = Vec::new();
    let result: LadderResult = {
        let mut runner = recording_runner(&steps, fail_reset);
        ladder.finish(GitStatus { code: Some(0) }, &mut runner, &git_context(), &mut printed, &mut std::io::sink())
    };
    return (result, steps.into_inner(), String::from_utf8_lossy(printed.as_slice()).into_owned());
}

/// The first,
/// unchanged tree needs no Git step at all and prints Git's line.
#[test]
fn btrfs_a_source_at_the_target_commit_is_served_with_no_git_step() {
    let Some(root) = btrfs_fixture("finish-same") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (result, steps, printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &[], false, |_| {});
    assert_eq!(result.git, GitStatus { code: Some(0) });
    assert_eq!(result.carried, Some(root.join("topic")));
    assert!(result.claim.is_some(), "the record of the add travels on to the copy");
    // The swap put the snapshot where Git's directory stood, so the destination is the object the rung made and not the one that was reserved before Git ran.
    assert_eq!(result.placed, crate::worktree_add_binding::path_identity(root.join("topic").as_path()), "the destination is the snapshot the rung swapped in");
    // The snapshot's own `.git` is a copy of the whole common directory, and it goes into the trash even when the leave-out moved nothing, so the add hands one record over whatever its source holds.
    let trash: PendingTrash = result.trash.expect("the snapshot's own .git is handed over");
    assert_eq!(trash.manifest.entries.len(), 1, "the snapshot's own .git is the one entry");
    assert!(steps.is_empty(), "{steps:?}");
    let head: String = git_text(repo.as_path(), &["log", "-1", "--format=HEAD is now at %h %s"]);
    assert_eq!(printed, format!("{head}\n"));
    assert!(is_subvolume_root(root.join("topic").as_path()).expect("probe"));
    assert!(root.join("topic/ignored/pkg/index.js").is_file());
    remove(root.as_path());
}

/// The names of the files this thread flushed to disk that carry `infix` in their name,
/// in call order.
fn flushed_with(infix: &str) -> Vec<String> {
    return take_file_syncs().into_iter().map(|path: PathBuf| { return path.file_name().map(|name| { return name.to_string_lossy().into_owned(); }).unwrap_or_default(); }).filter(|name: &String| { return name.contains(infix); }).collect();
}

/// A snapshot swapped into place hands its record to the lifecycle without a flush:
/// the swap of a subvolume makes every flush
///  after it in the same btrfs transaction a commit of the whole filesystem,
/// which waits for every other writer's data;
/// the record travels to the copy,
/// which settles it without a flush and writes no completion marker,
/// so the record of the add never disappears between the swap and the copy.
/// The add's own records are published by one rename each and flushed by nobody,
/// before the swap as well as after it,
/// and the control that the recorder reads this process's flushes is a write of the test's own.
#[test]
fn btrfs_a_served_snapshot_hands_its_record_over_without_a_flush() {
    let Some(root) = btrfs_fixture("finish-lazy-marker") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    take_file_syncs();
    let (result, _steps, _printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &[], false, |_| {});
    assert!(result.carried.is_some(), "the snapshot served the add");
    let flushed: Vec<String> = take_file_syncs().into_iter().map(|path: PathBuf| { return path.file_name().map(|name| { return name.to_string_lossy().into_owned(); }).unwrap_or_default(); }).collect();
    assert_eq!(flushed.iter().filter(|name: &&String| { return name.contains(".add."); }).count(), 0, "neither the binding record nor the destination record is flushed, so a loaded host cannot hold the add here: {flushed:?}");
    assert_eq!(flushed.iter().filter(|name: &&String| { return name.contains(".done."); }).count(), 0, "{flushed:?}");
    // The recorder's own control: a write this test flushes is seen, so the empty answers above are the add's and not the
    // recorder's.
    let control: PathBuf = root.join("recorder-control");
    crate::private_storage::write_private_file(control.as_path(), b"x").expect("a flushed control");
    let seen: Vec<PathBuf> = take_file_syncs();
    assert!(seen.iter().any(|path: &PathBuf| { return path.ends_with("recorder-control"); }), "the recorder sees a flushed write: {seen:?}");
    assert!(result.claim.is_some(), "the record travels on to the lifecycle");
    assert!(crate::worktree_copy_stage_claim::stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the record stands until its owner settles it");
    result.claim.expect("the record").settle_unsynced().expect("settled without a flush");
    assert!(std::fs::read_dir(journal_root(repo.join(".git").as_path())).map(|listing| { return listing.count(); }).unwrap_or(0) == 0, "settling the handed record leaves nothing");
    remove(root.as_path());
}

/// An add that Git's own checkout completed after the snapshot was refused hands its record over as well:
/// no completion marker is written,
/// so a command killed before the copy's journal still leaves the record naming the worktree.
#[test]
fn btrfs_a_checkout_completed_by_git_hands_its_record_over() {
    let Some(root) = btrfs_fixture("finish-flushed-marker") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    take_file_syncs();
    let (result, _steps, _printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &[], false, |destination| {
        std::fs::write(destination.join("stray"), b"x").expect("a stray file");
    });
    assert_eq!(result.carried, None, "Git's checkout served the add");
    assert_eq!(flushed_with(".done.").len(), 0, "no completion marker is written before the copy");
    assert!(result.claim.is_some(), "the record travels on to the copy");
    remove(root.as_path());
}

/// `--quiet` prints nothing.
#[test]
fn btrfs_quiet_prints_no_line() {
    let Some(root) = btrfs_fixture("finish-quiet") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (result, _steps, printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &["--quiet"], false, |_| {});
    assert!(result.carried.is_some());
    assert_eq!(printed, "");
    remove(root.as_path());
}

/// A destination that holds more than Git's pointer is not served:
/// the stage is destroyed,
/// `reset --hard` completes the
///  worktree with the work tree and git directory Git gives its own checkout,
/// and nothing is carried.
#[test]
fn btrfs_a_refused_step_destroys_the_stage_and_completes_the_checkout_like_git() {
    let Some(root) = btrfs_fixture("finish-fallback") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (result, steps, printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &[], false, |destination| {
        std::fs::write(destination.join("stray"), b"x").expect("a stray file");
    });
    assert_eq!(result.git, GitStatus { code: Some(0) });
    assert_eq!(result.carried, None);
    assert_eq!(result.placed, None, "Git's own checkout left the directory that was reserved");
    assert_eq!(result.trash, None);
    assert!(result.claim.is_some(), "the record of the add travels on to the copy");
    assert_eq!(printed, "", "Git's own reset prints the line, not the ladder");
    assert_eq!(steps.len(), 1, "{steps:?}");
    assert_eq!(&steps[0][2..], ["reset", "--hard", "--no-recurse-submodules"], "{steps:?}");
    assert!(root.join("topic/shared.txt").is_file(), "the checkout completed");
    assert!(!is_subvolume_root(root.join("topic").as_path()).expect("probe"), "the stage was not swapped in");
    let leftovers: Vec<String> = std::fs::read_dir(root.as_path()).expect("listing").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).filter(|name: &String| { return name.starts_with(".cli-git"); }).collect();
    assert_eq!(leftovers, Vec::<String>::new(), "the stage is gone");
    remove(root.as_path());
}

/// A failing `reset --hard` gives its status to the person,
/// as Git's own would,
/// and the worktree and its registration stay as
///  they are for the person to finish or remove.
#[test]
fn btrfs_a_failing_checkout_step_gives_its_status() {
    let Some(root) = btrfs_fixture("finish-reset-fails") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (result, _steps, _printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &["--quiet"], true, |destination| {
        std::fs::write(destination.join("stray"), b"x").expect("a stray file");
    });
    assert_eq!(result.git, GitStatus { code: Some(1) });
    assert_eq!(result.carried, None);
    assert_eq!(result.placed, None, "Git's own checkout left the directory that was reserved");
    assert_eq!(result.trash, None);
    assert!(result.claim.is_some(), "the record of the add travels on to the copy");
    assert!(root.join("topic/.git").exists() && root.join("topic/stray").exists(), "nothing was deleted");
    assert!(repo.join(".git/worktrees/topic").exists(), "nothing was unregistered");
    remove(root.as_path());
}

/// What:
///  A main checkout that is an ordinary directory on btrfs,
/// with two commits and a tracked sparse file above the clone rung's size limit.
/// Why:
///   An ordinary directory is what the clone rung serves,
/// and its size limit declines a source of too few tracked bytes,
/// so every control on this fixture would otherwise stop at Git's own checkout
///  before the ending it is about;
/// the sparse anchor is the same one the `worktree_ladder_run` controls use
///  (`worktree_ladder_test_support::tracked_sparse`).
fn ordinary_repository(root: &Path) -> PathBuf {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "repo")).expect("canonical");
    tracked_sparse(repo.as_path(), "bulk/kept.bin", crate::worktree_clone_rung_limits::THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG + 1_000);
    tracked(repo.as_path(), "shared.txt", b"first\n");
    commit_all(repo.as_path(), "first");
    tracked(repo.as_path(), "shared.txt", b"second, longer\n");
    commit_all(repo.as_path(), "second");
    return repo;
}

/// A file another process put into the destination after Git made it is no clone's to remove:
/// the destination holds more
///  than Git's pointer,
/// so the rung is refused,
/// and Git's own checkout leaves a file it has no entry for.
/// The fixture's sparse anchor puts the source above the clone rung's size limit,
/// so the clone rung is reached and refused on purpose.
#[test]
fn btrfs_a_refused_clone_rung_leaves_a_file_it_did_not_create() {
    let Some(root) = btrfs_fixture("finish-clone-stray") else { return };
    let repo: PathBuf = ordinary_repository(root.as_path());
    let (result, steps, _printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &[], false, |destination| {
        std::fs::write(destination.join("notes"), b"mine\n").expect("a file of someone else");
    });
    assert_eq!(result.git, GitStatus { code: Some(0) });
    assert_eq!(steps.len(), 1, "{steps:?}");
    assert_eq!(std::fs::read(root.join("topic/notes")).expect("kept"), b"mine\n", "a non-conflicting file is not deleted");
    assert!(root.join("topic/shared.txt").is_file(), "the checkout completed");
    remove(root.as_path());
}

/// When the checkout that completes a refused clone rung fails,
/// the worktree and its registration stay,
/// including the file
///  that was in it,
/// and the failing status is the person's.
/// The fixture's sparse anchor puts the source above the clone rung's size limit,
/// so the clone rung is reached and refused on purpose.
#[test]
fn btrfs_a_failing_checkout_after_a_refused_clone_rung_leaves_the_worktree_and_its_registration() {
    let Some(root) = btrfs_fixture("finish-clone-rollback") else { return };
    let repo: PathBuf = ordinary_repository(root.as_path());
    let (result, _steps, _printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &["--quiet"], true, |destination| {
        std::fs::write(destination.join("notes"), b"x").expect("a stray file");
    });
    assert_eq!(result.git, GitStatus { code: Some(1) });
    assert_eq!(std::fs::read(root.join("topic/notes")).expect("kept"), b"x");
    assert!(repo.join(".git/worktrees/topic").exists(), "the registration stays");
    assert!(result.claim.is_some(), "the record of this add is handed to the lifecycle, which settles it");
    assert!(crate::worktree_copy_stage_claim::stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the record stands until its owner settles it");
    remove(root.as_path());
}

/// A snapshot of the main checkout empties nothing itself:
/// the trash the leave-out made travels out of the ladder,
/// so the lifecycle
///  empties it beside the package repair of the new worktree.
#[test]
fn btrfs_a_snapshot_hands_its_trash_to_the_caller_instead_of_emptying_it() {
    let Some(root) = btrfs_fixture("finish-trash-handed-over") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    // The trash parent and the cache must be ignored in the source, so the source is clean; the cache is what the leave-out sets
    //  aside.
    tracked(repo.as_path(), ".gitignore", b"ignored/\n.claude/\n.pnpm-store/\n");
    commit_all(repo.as_path(), "ignore the trash parent");
    std::fs::create_dir_all(repo.join(".pnpm-store")).expect("a cache");
    std::fs::write(repo.join(".pnpm-store/data"), b"payload").expect("cache data");
    let (result, steps, _printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &[], false, |_| {});
    assert_eq!(result.carried, Some(root.join("topic")), "the snapshot served the add");
    assert!(steps.is_empty(), "{steps:?}");
    let pending: PendingTrash = result.trash.expect("the snapshot left a trash for the disposal");
    assert_eq!(pending.destination, root.join("topic"));
    assert_eq!(pending.manifest.entries.len(), 2, "the cache and the snapshot's own .git were set aside");
    assert!(!pending.quiet);
    let trash: PathBuf = root.join("topic").join(crate::worktree_trash::TRASH_PARENT).join(&pending.manifest.name);
    assert!(trash.join("d0/.pnpm-store/data").is_file() || trash.join("d0/data").is_file(), "the trash is still in the worktree, for the lifecycle to empty: {}", trash.display());
    remove(root.as_path());
}

/// What:
///  A trash whose record cannot be written is emptied before the rung returns,
/// and no trash nothing names travels
///  on to the hand-over.
/// Why:
///   The record is the only thing that empties a trash after the command,
/// so a record that cannot be written leaves
///  the trash with nothing that would ever remove it;
/// the add pays the emptying itself in that case,
/// which no
///  ordinary add reaches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFileSync(join(common, WORK_DIRECTORY), 'not a directory');
/// expect(recordBeforeTheRungReturns(pending)).toBeUndefined();
/// expect(existsSync(trashParent)).toBe(false);
/// ```
#[test]
fn a_trash_whose_record_cannot_be_written_is_emptied_before_the_rung_returns() {
    let root: PathBuf = fixture("finish-trash-unrecordable");
    let worktree: PathBuf = root.join("topic");
    let common: PathBuf = root.join("common");
    std::fs::create_dir_all(common.join("worktrees/topic")).expect("admin");
    std::fs::create_dir_all(worktree.join(crate::worktree_trash::TRASH_PARENT)).expect("the trash parent");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", common.join("worktrees/topic").display())).expect(".git file");
    let name: String = "f".repeat(40);
    std::fs::create_dir_all(worktree.join(crate::worktree_trash::TRASH_PARENT).join(&name).join("d0/payload")).expect("an entry");
    let held: std::fs::File = crate::fs_beneath::open_root(worktree.as_path()).expect("the worktree");
    let names: Vec<std::ffi::OsString> = crate::fs_beneath::plain_components(Path::new(crate::worktree_trash::TRASH_PARENT)).expect("a plain parent");
    let parent: std::fs::File = crate::fs_beneath_walk_chain::open_directory_beneath(&held, names.as_slice()).expect("the trash parent");
    let trash: std::fs::File = crate::fs_beneath::open_directory_at(&parent, std::ffi::OsStr::new(&name)).expect("the trash");
    let manifest: TrashManifest = TrashManifest {
        name: name.clone(),
        trash: crate::fs_beneath::identity_of(&trash).expect("the trash identity"),
        entries: vec![crate::fs_beneath::identity_at(&trash, std::ffi::OsStr::new("d0")).expect("the entry identity")],
    };
    // The queue has nowhere to go: the directory the records are written in is a plain file.
    std::fs::create_dir_all(common.join("monochromatic-git-policy-cli")).expect("the queue's parent");
    std::fs::write(common.join(crate::work_item::WORK_DIRECTORY), b"not a directory\n").expect("a file where the queue's directory would be");
    let pending: PendingTrash = PendingTrash { destination: worktree.clone(), common_dir: common.clone(), manifest, taken_over: 0, quiet: false };
    assert_eq!(record_before_the_rung_returns(pending), None, "a trash with no record travels no further");
    assert!(!worktree.join(crate::worktree_trash::TRASH_PARENT).exists(), "and the add empties it before it returns");
    remove(root.as_path());
}

/// What:
///  The manifest of a snapshot's trash is a durable record by the time the rung returns,
/// so a command killed after the
///  worktree is published still leaves a trash the service empties.
/// Why:
///   The manifest is what says which directories the leave-out may delete,
/// and the trash's name alone cannot
///  reconstruct it.
/// A record first written by the hand-over that runs after the copy and its repairs leaves every kill
///  between the swap and that step with a trash nothing will ever empty,
/// which is disk the person has to find
///  themselves.
/// The record is written the moment the trash stands in the published worktree,
/// and the rung still empties
///  nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { trash } = await ladder.finish(...); expect(pendingRecord(commonDir, trash.manifest.name)).toBeDefined(); expect(existsSync(trashPath)).toBe(true);
/// ```
#[test]
fn btrfs_a_snapshot_records_its_trash_before_the_rung_returns() {
    let Some(root) = btrfs_fixture("finish-trash-recorded") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    tracked(repo.as_path(), ".gitignore", b"ignored/\n.claude/\n.pnpm-store/\n");
    commit_all(repo.as_path(), "ignore the trash parent");
    std::fs::create_dir_all(repo.join(".pnpm-store")).expect("a cache");
    std::fs::write(repo.join(".pnpm-store/data"), b"payload").expect("cache data");
    let (result, _steps, _printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &[], false, |_| {});
    let pending: PendingTrash = result.trash.expect("the snapshot left a trash for the disposal");
    let record: PathBuf = crate::work_item::state_directory(crate::service_work::work_directory(pending.common_dir.as_path()).as_path(), crate::work_item::PENDING).join(format!("trash-{}", pending.manifest.name));
    let item: crate::work_item::WorkItem = crate::work_item::WorkItem::decode(std::fs::read_to_string(record.as_path()).expect("the record is written before the rung returns").as_str()).expect("a work item");
    assert_eq!(item.repository, pending.destination, "the record names the worktree that holds the trash");
    assert_eq!(item.git_dir, pending.common_dir, "and the common directory whose service empties it");
    assert_eq!(item.parent, pending.manifest.encode(), "and it carries the identities the leave-out wrote");
    let trash: PathBuf = pending.destination.join(crate::worktree_trash::TRASH_PARENT).join(&pending.manifest.name);
    assert!(trash.is_dir(), "and the rung empties nothing of it: {}", trash.display());
    remove(root.as_path());
}

/// A source file whose bytes were written again after the source was proven clean keeps the snapshot:
/// the entry the stat data
///  cannot vouch for is settled by content,
/// so Git's own checkout never runs.
#[test]
fn btrfs_a_source_file_written_again_with_its_own_bytes_keeps_the_snapshot() {
    let Some(root) = btrfs_fixture("finish-content-settled") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let source: PathBuf = repo.clone();
    let (result, steps, _printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &[], false, move |_| {
        std::fs::write(source.join("shared.txt"), b"second, longer\n").expect("the same bytes again");
    });
    assert_eq!(result.carried, Some(root.join("topic")), "the snapshot served the add");
    assert!(steps.is_empty(), "no Git step, so the slow route never ran: {steps:?}");
    assert_eq!(std::fs::read(root.join("topic/shared.txt")).expect("the file"), b"second, longer\n");
    assert!(is_subvolume_root(root.join("topic").as_path()).expect("probe"), "the snapshot stands in place");
    remove(root.as_path());
}

/// A main checkout whose file changed in content after the source was proven clean keeps the snapshot and is restored to the new
///  `HEAD`,
/// instead of discarding it for Git's own checkout.
#[test]
fn btrfs_a_main_source_changed_under_the_copy_is_restored_instead_of_copied_in_full() {
    let Some(root) = btrfs_fixture("finish-restore-under-copy") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let source: PathBuf = repo.clone();
    let (result, steps, _printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &[], false, move |_| {
        std::fs::write(source.join("shared.txt"), b"edited while the copy was made\n").expect("a change after the check");
    });
    assert_eq!(result.carried, Some(root.join("topic")), "the snapshot served the add");
    assert!(is_subvolume_root(root.join("topic").as_path()).expect("probe"), "the snapshot stands in place");
    let ran = |word: &str| -> bool { return steps.iter().any(|words: &Vec<String>| { return words.iter().any(|each: &String| { return each == word; }); }); };
    assert!(ran("read-tree"), "the restore ran in the stage: {steps:?}");
    assert!(!ran("reset"), "Git's own checkout did not run: {steps:?}");
    assert_eq!(std::fs::read(root.join("topic/shared.txt")).expect("the committed bytes"), b"second, longer\n", "the worktree holds what Git's own checkout holds");
    assert!(root.join("topic/ignored/pkg/index.js").is_file(), "the ignored state is still carried");
    remove(root.as_path());
}

/// A writer that reports what the add's records held when the line for the person was written.
struct RecordProbe {
    /// The directory of the claims.
    claims: PathBuf,
    /// Whether a completion marker existed at the first write.
    marker_seen: Option<bool>,
    /// Whether the add's record named the directories Git made at the first write.
    bound_seen: Option<bool>,
}

impl Write for RecordProbe {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let marker: bool = std::fs::read_dir(self.claims.as_path()).is_ok_and(|listing| { return listing.flatten().any(|entry| { return entry.file_name().to_string_lossy().ends_with(".done"); }); });
        self.marker_seen.get_or_insert(marker);
        let bound: bool = std::fs::read_dir(self.claims.as_path()).is_ok_and(|listing| {
            return listing.flatten().filter(|entry| { return entry.file_name().to_string_lossy().ends_with(".add"); }).any(|entry| { return std::fs::read_to_string(entry.path()).is_ok_and(|text: String| { return text.contains("phase=registered"); }); });
        });
        self.bound_seen.get_or_insert(bound);
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// Both rungs hand the add's record to the lifecycle instead of releasing it,
/// and write no completion marker:
/// the record of an add that owes a copy must outlive the checkout,
/// or a command killed before the copy's journal would leave a worktree with no ignored state and nothing naming it.
/// Both fixtures hold a sparse anchor above the clone rung's size limit,
/// and the ordinary one is an ordinary directory,
/// so each control reaches its own rung on purpose.
#[test]
fn btrfs_each_rung_hands_the_adds_record_over_before_it_prints_and_marks_nothing() {
    for (name, ordinary) in [("finish-marker-clone", true), ("finish-marker-snapshot", false)] {
        let Some(root) = btrfs_fixture(name) else { return };
        let repo: PathBuf = if ordinary { ordinary_repository(root.as_path()) } else { subvolume_repository(root.as_path()) };
        let plan = plan_to(root.join("topic"));
        let claims: PathBuf = journal_root(repo.join(".git").as_path());
        let ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
        let records: Vec<String> = std::fs::read_dir(claims.as_path()).expect("claims").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
        assert!(records.iter().any(|record: &String| { return record.ends_with(".add"); }), "{name}: the add is recorded before Git runs: {records:?}");
        git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
        let mut probe: RecordProbe = RecordProbe { claims: claims.clone(), marker_seen: None, bound_seen: None };
        let steps: Steps = RefCell::new(Vec::new());
        let result: LadderResult = {
            let mut runner = recording_runner(&steps, false);
            ladder.finish(GitStatus { code: Some(0) }, &mut runner, &git_context(), &mut probe, &mut std::io::sink())
        };
        assert_eq!(result.git, GitStatus { code: Some(0) }, "{name}");
        assert_eq!(probe.marker_seen, Some(false), "{name}: no completion marker is written while a copy is still owed");
        assert_eq!(probe.bound_seen, Some(true), "{name}: the record names the directories Git made when the line is printed");
        assert!(result.claim.is_some(), "{name}: the record is handed to the lifecycle");
        let left: Vec<String> = std::fs::read_dir(claims.as_path()).expect("claims").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
        assert!(left.iter().any(|entry: &String| { return entry.ends_with(".add"); }), "{name}: the record stands until the copy owns it: {left:?}");
        remove(root.as_path());
    }
}

/// A stage name that holds somebody else's directory after a failed swap back is not the wrapper's to destroy.
#[test]
fn an_error_from_a_failed_swap_back_does_not_own_the_stage() {
    let foreign: Ended = Ended::from(PlaceError { reason: String::from("x"), stage_is_ours: false });
    let own: Ended = Ended::from(PlaceError { reason: String::from("y"), stage_is_ours: true });
    assert!(!foreign.stage_is_ours && own.stage_is_ours);
    assert!(Ended::from(String::from("a plain reason")).stage_is_ours, "an error before the exchange left the stage ours");
}

/// When the records cannot be written,
/// the add still succeeds:
/// the marker only keeps a later report from calling a whole
///  worktree interrupted,
/// and nothing is deleted either way.
/// The fixture's sparse anchor puts the source above the clone rung's size limit,
/// so the clone rung serves the add on purpose.
#[test]
fn btrfs_an_add_whose_records_cannot_be_written_still_succeeds_and_keeps_everything() {
    let Some(root) = btrfs_fixture("finish-records-unwritable") else { return };
    let repo: PathBuf = ordinary_repository(root.as_path());
    let plan = plan_to(root.join("topic"));
    let claims: PathBuf = journal_root(repo.join(".git").as_path());
    let ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    std::fs::set_permissions(claims.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o500)).expect("read-only records");
    let steps: Steps = RefCell::new(Vec::new());
    let mut printed: Vec<u8> = Vec::new();
    let result: LadderResult = {
        let mut runner = recording_runner(&steps, false);
        ladder.finish(GitStatus { code: Some(0) }, &mut runner, &git_context(), &mut printed, &mut std::io::sink())
    };
    std::fs::set_permissions(claims.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("writable again");
    assert_eq!(result.git, GitStatus { code: Some(0) });
    assert!(!printed.is_empty(), "Git's line is printed");
    assert!(root.join("topic/shared.txt").is_file() && repo.join(".git/worktrees/topic").exists());
    remove(root.as_path());
}

/// A worktree that was moved away after the add was bound,
/// with another worktree made at its path,
/// is never reset:
/// the fallback
///  checkout verifies the binding and stops,
/// the replacement keeps its edits,
/// the moved worktree keeps its registration,
/// and the
///  add fails with a report.
/// The fixture's sparse anchor puts the source above the clone rung's size limit,
/// so the clone rung serves the add on purpose.
#[test]
fn btrfs_a_fallback_never_resets_a_replacement_at_the_destination() {
    let Some(root) = btrfs_fixture("finish-fallback-replaced") else { return };
    let repo: PathBuf = ordinary_repository(root.as_path());
    let plan = plan_to(root.join("topic"));
    let ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "--detach", "../topic", "HEAD~1"]);
    // The first step the ladder asks for is the tree difference: while it runs, the worktree is moved and replaced.
    let replaced: std::cell::Cell<bool> = std::cell::Cell::new(false);
    let steps: Steps = RefCell::new(Vec::new());
    let mut inner = recording_runner(&steps, false);
    let mut swapping = |step: &GitStep<'_>| {
        if !replaced.replace(true) {
            git(repo.as_path(), &["worktree", "move", "../topic", "../moved"]);
            git(repo.as_path(), &["worktree", "add", "--quiet", "--detach", "../topic", "HEAD"]);
            std::fs::write(root.join("topic/edited.txt"), b"work in the replacement").expect("their edit");
            return Ok(GitStatus { code: Some(1) });
        }
        return inner(step);
    };
    let mut printed: Vec<u8> = Vec::new();
    let result: LadderResult = ladder.finish(GitStatus { code: Some(0) }, &mut swapping, &git_context(), &mut printed, &mut std::io::sink());
    assert_eq!(result.git, GitStatus { code: Some(1) });
    assert!(printed.is_empty());
    assert_eq!(std::fs::read(root.join("topic/edited.txt")).expect("the replacement is untouched"), b"work in the replacement");
    assert!(root.join("moved").exists(), "the moved worktree stays");
    assert!(steps.borrow().iter().all(|words: &Vec<String>| { return !words.iter().any(|word: &String| { return word == "reset"; }); }), "no reset was run: {:?}", steps.borrow());
    remove(root.as_path());
}

/// A checkout that succeeded while the worktree was being moved and replaced is not taken for the add's:
/// the status is a
///  failure,
/// the replacement is untouched by anything the wrapper does afterwards,
/// and no completion marker is written.
/// The fixture's sparse anchor puts the source above the clone rung's size limit,
/// so the clone rung serves the add on purpose.
#[test]
fn btrfs_a_checkout_that_ran_against_a_replacement_is_not_taken_for_the_adds() {
    let Some(root) = btrfs_fixture("finish-checkout-replaced") else { return };
    let repo: PathBuf = ordinary_repository(root.as_path());
    let plan = plan_to(root.join("topic"));
    let ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    std::fs::write(root.join("topic/notes"), b"x").expect("a stray file, so the clone rung is refused");
    let steps: Steps = RefCell::new(Vec::new());
    let mut inner = recording_runner(&steps, false);
    let mut swapping = |step: &GitStep<'_>| {
        let outcome = inner(step);
        git(repo.as_path(), &["worktree", "move", "../topic", "../moved"]);
        git(repo.as_path(), &["worktree", "add", "--quiet", "--detach", "../topic", "HEAD"]);
        return outcome;
    };
    let mut printed: Vec<u8> = Vec::new();
    let result: LadderResult = ladder.finish(GitStatus { code: Some(0) }, &mut swapping, &git_context(), &mut printed, &mut std::io::sink());
    assert_eq!(result.git, GitStatus { code: Some(1) }, "the reset succeeded, but not against the bound worktree");
    assert!(root.join("moved/shared.txt").exists() && root.join("topic/shared.txt").exists(), "both worktrees stand");
    remove(root.as_path());
}

/// A registration replaced after the add was bound stops the clone rung before it writes anything:
/// the destination keeps only
///  Git's pointer,
/// and the clones never reach the replacement's neighbourhood.
#[test]
fn btrfs_the_clone_rung_writes_nothing_when_the_registration_was_replaced() {
    let Some(root) = btrfs_fixture("finish-clone-registration-replaced") else { return };
    let repo: PathBuf = ordinary_repository(root.as_path());
    let plan = plan_to(root.join("topic"));
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let source = crate::worktree_snapshot_source::common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    let bound: Bound = crate::worktree_add_binding::bind_registration(source.common_dir.as_path(), plan.destination.as_path()).expect("bound");
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    std::fs::rename(admin.as_path(), admin.with_file_name("topic.moved")).expect("registration moved");
    std::fs::create_dir(admin.as_path()).expect("another registration");
    let steps: Steps = RefCell::new(Vec::new());
    let mut runner = recording_runner(&steps, false);
    let reason: String = serve_clones(&source, crate::worktree_clone_conversion_beside_the_clone::CloneFence::Beside(crate::worktree_clone_conversion_fence::fence_the_attributes(&source, &git_context()).expect("the attributes")), &plan, &bound, &mut runner, &git_context(), None).expect_err("refused");
    assert!(reason.contains("no longer the one Git made"), "{reason}");
    let entries: Vec<String> = std::fs::read_dir(root.join("topic")).expect("listing").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    assert_eq!(entries, vec![String::from(".git")], "nothing was written into the worktree");
    remove(root.as_path());
}

/// What:
///  A clean source whose attributes convert one tracked path gets Git's own bytes on that path from the snapshot rung,
///  and the worktree it publishes is clean.
/// Why:
///   A snapshot carries the source's working bytes,
/// so where a line-ending rule applies the stage holds the blob and Git's checkout
///  would write the converted form (issue 6323).
/// The fence is decided before the snapshot,
/// so the index the snapshot installs records that path without stat data,
///  and the one `git checkout-index` after the difference writes it.
/// The index control in `worktree_snapshot_index_step_tests.rs` proves the first half.
/// The control proves the served bytes are the ones real Git's own checkout writes for the same commit,
/// and that they are not the source's:
/// the source file holds the blob (LF) and Git's checkout writes carriage returns,
/// so a rung that carried the snapshot's bytes would fail here.
/// The fence's write goes through the crate's own Git runner and not the ladder's,
/// so the control reads the bytes rather than the step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const (result) = addThroughTheLadder(repo, root);
/// expect(readFileSync('topic/notes.txt')).toEqual(readFileSync('reference/notes.txt'));
/// expect(readFileSync('topic/notes.txt')).not.toEqual(readFileSync('repo/notes.txt'));
/// expect(status('topic')).toBe('');
/// ```
#[test]
fn btrfs_a_converting_path_is_written_by_git_and_the_snapshot_rung_is_clean() {
    let Some(root) = btrfs_fixture("finish-conversion") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    tracked(repo.as_path(), ".gitattributes", b"*.txt text eol=crlf\n");
    tracked(repo.as_path(), "notes.txt", b"line one\nline two\n");
    commit_all(repo.as_path(), "converting");
    let (result, _steps, _printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &[], false, |_| {});
    assert_eq!(result.git, GitStatus { code: Some(0) });
    assert_eq!(result.carried, Some(root.join("topic")), "the snapshot rung served the add");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "reference", "../reference"]);
    let served: Vec<u8> = std::fs::read(root.join("topic/notes.txt")).expect("the served path");
    let gits: Vec<u8> = std::fs::read(root.join("reference/notes.txt")).expect("Git's checkout");
    assert_eq!(served, b"line one\r\nline two\r\n", "the served path holds Git's conversion");
    assert_eq!(served, gits, "the served bytes are the ones Git's own checkout writes");
    assert_ne!(served, std::fs::read(root.join("repo/notes.txt")).expect("the source's file"), "the source's file holds the blob, so a rung that carried it would fail here");
    assert_eq!(git_text(root.join("topic").as_path(), &["status", "--porcelain=v1"]), "", "the published worktree is clean");
    remove(root.as_path());
}

/// What:
///  The same control for a source whose conversion comes from the configuration rather than from an attributes rule:
///  `core.autocrlf=true` set after the files were committed,
///  which is the class the conversion controls call
///  `a_config_rule_gone_after_the_add_is_written_by_git`'s neighbour.
/// Why:
///   The fence reads `core.autocrlf` from the source's configuration,
/// and the snapshot rung carries the source's bytes either way,
///  so a rung that did not write these paths through Git would leave LF where Git's checkout writes CRLF.
/// The control reads the served bytes against a real Git checkout of the same commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git(repo, ['config', 'core.autocrlf', 'true']);
/// const (result) = addThroughTheLadder(repo, root);
/// expect(readFileSync('topic/notes.txt')).toEqual(readFileSync('reference/notes.txt'));
/// ```
#[test]
fn btrfs_a_configuration_rule_writes_gits_bytes_in_the_snapshot_rung() {
    let Some(root) = btrfs_fixture("finish-conversion-config") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    tracked(repo.as_path(), "notes.txt", b"line one\nline two\n");
    commit_all(repo.as_path(), "plain endings");
    git(repo.as_path(), &["config", "core.autocrlf", "true"]);
    let (result, _steps, _printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &[], false, |_| {});
    assert_eq!(result.git, GitStatus { code: Some(0) });
    assert_eq!(result.carried, Some(root.join("topic")), "the snapshot rung served the add");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "reference", "../reference"]);
    let served: Vec<u8> = std::fs::read(root.join("topic/notes.txt")).expect("the served path");
    let gits: Vec<u8> = std::fs::read(root.join("reference/notes.txt")).expect("Git's checkout");
    assert_eq!(served, gits, "the served bytes are the ones Git's own checkout writes under core.autocrlf=true");
    assert_ne!(served, b"line one\nline two\n", "Git's checkout converts both text lines, so a rung that carried the source would fail here");
    assert_eq!(git_text(root.join("topic").as_path(), &["status", "--porcelain=v1"]), "", "the published worktree is clean");
    remove(root.as_path());
}

/// What:
///  Every tracked path of a worktree with its bytes and whether it is executable,
/// by path.
/// Why:
///   A control that compares two worktrees compares what each holds,
/// and a path one holds and the other does not is as much a difference as another path's bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// trackedBytes(worktree); // [path, bytes, executable]
/// ```
fn tracked_bytes(worktree: &Path) -> Vec<(String, Vec<u8>, bool)> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt as _;
    use std::os::unix::fs::PermissionsExt as _;
    let listing: Vec<u8> = git_output(worktree, &["ls-files", "-z"]).stdout;
    let mut files: Vec<(String, Vec<u8>, bool)> = Vec::new();
    for name in listing.split(|byte: &u8| { return *byte == 0; }).filter(|name: &&[u8]| { return !name.is_empty(); }) {
        let path: PathBuf = worktree.join(OsStr::from_bytes(name));
        let executable: bool = std::fs::metadata(path.as_path()).map(|held: std::fs::Metadata| { return held.permissions().mode() & 0o111 != 0; }).unwrap_or(false);
        files.push((String::from_utf8_lossy(name).into_owned(), std::fs::read(path.as_path()).unwrap_or_default(), executable));
    }
    return files;
}

/// What:
///  An add to a commit whose tree lacks the paths the source's `HEAD` holds ends after the snapshot was taken,
/// and the worktree it publishes equals Git's checkout of that commit:
/// every tracked path's bytes and mode,
/// a clean status,
/// and no path of the snapshot the target tree lacks.
/// Why:
///   The snapshot rung writes the fence's paths in the stage before the exchange,
/// so the same target tree ends the rung with the destination holding only Git's pointer and no path of the copies can stay;
/// the control requires that of the rung as of the clone rung,
/// so the two rungs' endings are held to the same tree
/// (`m05` of the class controls leaves the clone rung with its copies).
/// The commit-ish is `-`,
/// which Git reads as this worktree's previous branch and the skip's one name-only read cannot,
/// so the ending is reached on purpose:
/// a commit-ish the read resolves is skipped before the snapshot,
/// which would leave this rung's ending untested.
#[test]
fn btrfs_a_target_tree_that_lacks_a_fenced_path_leaves_no_path_of_the_snapshot_behind() {
    let Some(root) = btrfs_fixture("finish-older-target") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    tracked(repo.as_path(), ".gitattributes", b"*.txt text eol=crlf\n");
    tracked(repo.as_path(), "a.txt", b"line one\nline two\n");
    tracked(repo.as_path(), "keep.dat", b"line one\n");
    commit_all(repo.as_path(), "conversion");
    git(repo.as_path(), &["update-ref", "refs/heads/other", "HEAD"]);
    tracked(repo.as_path(), "extra.txt", b"line one\nline two\n");
    for index in 0..OLDER_TARGET_OLD_PATHS {
        tracked(repo.as_path(), format!("old/file-{index}.dat").as_str(), b"line one\n");
    }
    commit_all(repo.as_path(), "second");
    // `@{-1}`, which `-` names, is the branch this worktree was on before the current one, so the reflog has to hold
    //  that checkout. Those two checkouts rewrite the files, and a file as new as the index proves nothing, so the
    //  index is refreshed after the second has passed: only a provably clean source reaches the ending.
    git(repo.as_path(), &["checkout", "--quiet", "other"]);
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    crate::worktree_ladder_test_support::wait_for_a_new_second();
    git(repo.as_path(), &["update-index", "--really-refresh"]);
    let (result, steps, _printed) = add_through_the_ladder_at(repo.as_path(), root.as_path(), Some("-"), &[], false, |_| {});
    // The ending has a Git step of its own, and a skipped add has none: this is how the control says it reached it.
    assert!(steps.iter().any(|words: &Vec<String>| return words.iter().any(|word: &String| return word == "read-tree")), "the two-tree update ran: {steps:?}");
    assert_eq!(result.git, GitStatus { code: Some(0) }, "Git's own checkout completed the worktree");
    assert_eq!(result.carried, None, "the snapshot rung published no worktree");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "reference", "../reference", "other"]);
    let topic: PathBuf = root.join("topic");
    let reference: PathBuf = root.join("reference");
    assert_eq!(tracked_bytes(topic.as_path()), tracked_bytes(reference.as_path()), "every tracked path is Git's checkout's");
    assert_eq!(git_text(topic.as_path(), &["status", "--porcelain=v1"]), "", "the worktree is clean");
    // The snapshot's own trash folder is its `.claude`, which the reference has no reason to hold.
    assert_eq!(paths_below(topic.as_path(), &[".git", ".claude"]), paths_below(reference.as_path(), &[".git"]), "a path the snapshot carried and the target tree lacks stays behind");
    assert!(!topic.join("extra.txt").exists(), "the fenced path the older tree lacks is gone");
    assert!(!topic.join("old").exists(), "the folder the older tree lacks is gone");
    remove(root.as_path());
}

/// What:
///  A repository two commits deep under a `*.txt text eol=crlf` rule,
///  the older of which lacks `extra.txt`,
/// with a worktree at the older commit prepared the way the rung's steps leave one:
/// the newer commit's files on disk,
/// and its index written under the rung's own name,
///  by a `git read-tree` that records no stat data.
/// Why:
///   The shape is the rung's last two states before the two-tree update refuses,
/// so a control can run the exact fallback the rung runs without a clone-capable filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { repo, destination, admin, pending, newer } = failedTreeDifferenceShape(root, steps);
/// ```
fn failed_tree_difference_shape(root: &Path, runner: &mut impl FnMut(&GitStep<'_>) -> Result<GitStatus, WorktreeCopyError>) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "repo")).expect("canonical");
    tracked(repo.as_path(), ".gitattributes", b"*.txt text eol=crlf\n");
    tracked(repo.as_path(), "notes.txt", b"line one\nline two\n");
    commit_all(repo.as_path(), "older");
    let older: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    tracked(repo.as_path(), "extra.txt", b"line one\nline two\n");
    commit_all(repo.as_path(), "newer");
    let newer: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let destination: PathBuf = root.join("topic");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", destination.to_str().expect("utf8"), older.as_str()]);
    // The clones' output on disk: the newer commit's tracked files, byte for byte.
    for name in [".gitattributes", "notes.txt", "extra.txt"] {
        std::fs::copy(repo.join(name), destination.join(name)).expect("a copied file");
    }
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    let pending: PathBuf = admin.join("index.pending");
    let environment: Vec<(OsString, OsString)> = vec![(OsString::from("GIT_INDEX_FILE"), pending.as_os_str().to_os_string())];
    let arguments: Vec<OsString> = ["-C"].iter().map(OsString::from).chain([destination.as_os_str().to_os_string()]).chain(["read-tree"].iter().map(OsString::from)).chain([OsString::from(newer.as_str())]).collect();
    runner(&GitStep::Other { arguments: arguments.as_slice(), environment: environment.as_slice() }).expect("the index the rung wrote");
    assert!(pending.exists(), "the rung's index waits under its own name");
    assert!(!admin.join("index").exists(), "and the worktree has no index yet");
    return (repo, destination, admin, pending);
}

/// What:
///  The fallback checkout runs against the rung's waiting index and rewrites the entries that hold no stat data,
/// and the worktree's own `index` does not exist while it runs.
/// Why:
///   `git read-tree -m -u` refuses a zeroed entry,
/// so the rung's fallback is Git's own `reset --hard` against the index the rung wrote under its own name;
/// if that run merely refreshed such an entry instead of rewriting the file,
/// the source's bytes would stay where Git's checkout writes others (issue 6323),
/// and a published index before the run would vouch for them.
/// The control is the same shape with the entry's stat data recorded by `git update-index --refresh`:
/// the same measurement then shows the source's bytes and a clean status,
///  so it can see the difference between a rewrite and a refresh.
#[test]
fn the_fallback_checkout_rewrites_a_zeroed_entry_and_leaves_a_recorded_one_trusted() {
    for arm in ["zeroed", "recorded"] {
        let root: PathBuf = fixture(format!("finish-fallback-{arm}").as_str());
        let steps: Steps = RefCell::new(Vec::new());
        let mut runner = recording_runner(&steps, false);
        let (repo, destination, admin, pending) = failed_tree_difference_shape(root.as_path(), &mut runner);
        if arm == "recorded" {
            // The entry of `notes.txt` is refreshed over the source's LF bytes, which is the state a published index would
            //  have reached before the write (the defect the waiting name prevents).
            let environment: Vec<(OsString, OsString)> = vec![(OsString::from("GIT_INDEX_FILE"), pending.as_os_str().to_os_string())];
            let arguments: Vec<OsString> = ["-C"].iter().map(OsString::from).chain([destination.as_os_str().to_os_string()]).chain(["update-index", "--refresh"].iter().map(OsString::from)).collect();
            runner(&GitStep::Other { arguments: arguments.as_slice(), environment: environment.as_slice() }).expect("refresh");
        }
        let plan = plan_to(destination.clone());
        let bound: Bound = crate::worktree_add_binding::bind_registration(repo.join(".git").as_path(), destination.as_path()).expect("bound");
        let status: GitStatus = super::checkout_like_git_with((&plan, &bound, &mut runner), Some(pending.as_path()), true);
        assert_eq!(status, GitStatus { code: Some(0) }, "Git's own checkout ran ({arm})");
        assert!(!admin.join("index").exists(), "the worktree's own name is the rung's to publish, after this run ({arm})");
        // The publication the rung makes after the run, so `git status` here reads a published index as the person's would.
        std::fs::rename(pending.as_path(), admin.join("index")).expect("the publication");
        let served: Vec<u8> = std::fs::read(destination.join("notes.txt")).expect("the served path");
        let entry: String = git_text(destination.as_path(), &["ls-files", "--debug", "--", "notes.txt"]);
        if arm == "zeroed" {
            assert_eq!(served, b"line one\r\nline two\r\n", "an entry without stat data is rewritten, not refreshed");
            assert!(!destination.join("extra.txt").exists(), "an entry the target tree lacks is removed");
            assert!(entry.contains("size: 20"), "the rewritten file's own stat data is recorded: {entry}");
        } else {
            assert_eq!(served, b"line one\nline two\n", "a recorded entry is trusted, so the source's bytes stay: the defect the ordering prevents");
            assert!(entry.contains("size: 18"), "the recorded entry describes the source's bytes, not Git's: {entry}");
            assert_eq!(git_text(destination.as_path(), &["status", "--porcelain=v1"]), "", "and that state reads as a clean worktree, so the probe sees a refresh where a rewrite was needed");
        }
        remove(root.as_path());
    }
}

/// What:
///  An add at a commit whose tree lacks a path the snapshot rung's fence must have Git write is skipped before the stage
///  name is claimed and before the snapshot,
///  so the add is Git's own checkout and the snapshot pays nothing.
/// Why:
///   The index the snapshot installs holds no stat data for a fenced path,
/// and the tree difference refuses such an entry,
///  so the rung would end after the snapshot and pay Git's whole checkout over it.
/// The fence is decided before the snapshot in this rung,
///  so its own answer and one name-only read settle it there,
///  and the same fence decides the clone rung's skip.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ladder = beginLadder(planAt('other'), context, identity);
/// expect(ladder.firstStep()).toBe('person'); expect(ladder.rung).toBe('git'); expect(ladder.slow.word()).toBe('declines');
/// ```
#[test]
fn btrfs_a_target_tree_that_changes_a_fenced_path_is_served_by_gits_checkout_without_a_snapshot() {
    let Some(root) = btrfs_fixture("finish-older-target-skip") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    tracked(repo.as_path(), ".gitattributes", b"*.txt text eol=crlf\n");
    tracked(repo.as_path(), "a.txt", b"line one\nline two\n");
    commit_all(repo.as_path(), "conversion");
    git(repo.as_path(), &["update-ref", "refs/heads/other", "HEAD"]);
    tracked(repo.as_path(), "extra.txt", b"line one\nline two\n");
    commit_all(repo.as_path(), "second");
    let plan = crate::worktree_ladder_test_support::plan_to_at(root.join("topic"), Some("other"));
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    assert_eq!(ladder.first_step(), GitStep::Person, "Git's own checkout writes the worktree");
    assert!(matches!(ladder.rung, Rung::Git(_)), "the snapshot rung steps aside before the snapshot");
    let route = ladder.slow.as_ref().expect("the sentence that says why");
    assert_eq!(route.word(), "declines", "{route:?}");
    assert!(route.line().contains("differs from the source's index at extra.txt"), "{}", route.line());
    assert!(!crate::worktree_copy_stage_claim::stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "no stage name was claimed, so nothing was snapshotted");
    remove(root.as_path());
}

/// Give every file below `root` the fixture's own past time,
/// so the index a checkout just wrote is newer than all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const entry of readdirSync(root, { withFileTypes: true })) { entry.isDirectory() && entry.name !== '.git' ? ageBelow(join(root, entry.name)) : utimesSync(join(root, entry.name), past, past); }
/// ```
fn age_below(root: &Path) {
    for entry in std::fs::read_dir(root).expect("the checkout").flatten() {
        let path: PathBuf = entry.path();
        if entry.file_type().expect("a file type").is_dir() {
            if path.file_name() != Some(std::ffi::OsStr::new(".git")) {
                age_below(path.as_path());
            }
            continue;
        }
        crate::test_support::age(path.as_path());
    }
}

/// What:
///  A linked-worktree source whose snapshot is blocked after the check takes rung 2
///  with the early read already started,
///  and the add's end does not wait for that read before the clone rung's own work.
/// Why:
///   The read starts before `checkSource` on rung 1's condition alone;
///  a blocker found after the check (`snapshot_blocker`) sends the add to rung 2,
///  and the integration pass's Medium 2 of 2026-11-22 found the settle waiting there
///  before the copies,
/// where `main` has no read at all.
///  The held read makes the wait visible:
/// a wait would last its 30 s hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ladder = beginLadder(plan, context, linkedIdentity); expect(ladder.rung.kind).toBe('clone'); const began = now(); ladder.finish({ code: 1 }, ...); expect(now() - began).toBeLessThan(2_000);
/// ```
#[test]
fn btrfs_a_rung_two_fall_through_does_not_wait_for_the_read_before_its_copies() {
    let Some(root) = btrfs_fixture("rung-two-fall-through") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let linked: PathBuf = root.join("linked");
    crate::fs_btrfs_ioctl::subvolume_create_at(linked.as_path()).expect("the linked worktree's subvolume");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", linked.to_str().expect("utf8")]);
    age_below(linked.as_path());
    git(linked.as_path(), &["update-index", "--really-refresh"]);
    let nested: PathBuf = linked.join("ignored").join("nested");
    std::fs::create_dir_all(nested.parent().expect("the ignored directory")).expect("the ignored directory");
    git(linked.as_path(), &["worktree", "add", "--quiet", "--detach", nested.to_str().expect("utf8")]);
    let common: PathBuf = repo.join(".git");
    let identity = WorktreeIdentity::LinkedWorktree { worktree_root: linked.clone(), git_dir: common.join("worktrees").join("linked"), common_dir: common.clone() };
    let plan: AddPlan = crate::worktree_ladder_test_support::plan_to(root.join("topic"));
    let hold = crate::worktree_clone_conversion_size_read::a_control_holds_the_read();
    crate::add_timing::begin(repo.join(".git"), true);
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &identity);
    let why = ladder.slow.as_ref().map(|route| return route.detail()).unwrap_or_default();
    assert!(matches!(ladder.rung, Rung::Clone(..)), "the snapshot is blocked after the check, so the clone rung serves it (slow route: {why})");
    assert!(ladder.sizes_beside.is_none(), "the read rung 2 never uses is ended, not left running beside the copies");
    let mut runner = |_step: &GitStep<'_>| -> Result<GitStatus, WorktreeCopyError> { panic!("a failed Git run serves nothing here") };
    let began = std::time::Instant::now();
    let _result: LadderResult = ladder.finish(GitStatus { code: Some(1) }, &mut runner, &git_context(), &mut std::io::sink(), &mut std::io::sink());
    let elapsed: std::time::Duration = began.elapsed();
    crate::add_timing::finish(0);
    drop(hold);
    assert!(elapsed < std::time::Duration::from_secs(2), "the add's end does not wait for the unused read before rung 2: {elapsed:?}");
    let record: String = std::fs::read_to_string(repo.join(".git").join(crate::add_timing::ADD_TIMING_FILE)).unwrap_or_default();
    assert!(record.contains("\"fenceSizesBesideTheCheck\":\"started and not joined\""), "the add states the read it did not join: {record}");
    remove(root.as_path());
}
