//! Tests of the stage claim and the cleanup of stages an interrupted copy left behind.

use super::*;
use crate::private_storage::create_private_directory;
use crate::test_support::{fixture, git_context, remove, repository};
use crate::worktree_copy_journal::{JOURNAL_PARENT_NAME, create_journal, ensure_journal_root};
use crate::worktree_copy_selection::CopySelection;
use crate::worktree_copy_stage::stage_ignored_snapshot;
use std::os::unix::fs::PermissionsExt;

/// A common directory and a destination parent beside it, as a copy into `worktrees/topic` would use.
fn scene(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join("common");
    let worktrees: PathBuf = root.join("worktrees");
    std::fs::create_dir_all(common.as_path()).expect("common");
    std::fs::create_dir_all(worktrees.as_path()).expect("worktrees");
    return (root, common, worktrees.join("topic"));
}

/// A stage container with a payload and a read-only directory, the shape an interrupted copy leaves.
fn abandoned_stage(container: &Path) {
    create_private_directory(container).expect("container");
    let payload: PathBuf = container.join("payload");
    create_private_directory(payload.as_path()).expect("payload");
    let locked: PathBuf = payload.join("locked");
    create_private_directory(locked.as_path()).expect("locked");
    std::fs::write(locked.join("data"), b"staged").expect("data");
    std::fs::set_permissions(locked.as_path(), std::fs::Permissions::from_mode(0o500)).expect("read-only");
}

/// A claim names a container beside the destination, writes it under the journal root, and creates nothing else.
#[test]
fn a_claim_names_the_container_and_creates_no_container() {
    let (root, common, destination) = scene("claim-names");
    let claim: StageClaim = claim_stage(common.as_path(), destination.as_path()).expect("claim");
    assert_eq!(claim.container().parent(), destination.parent());
    assert!(claim.container().file_name().expect("name").to_string_lossy().starts_with(STAGE_PREFIX));
    assert!(!claim.container().exists(), "the claim precedes the container");
    let recorded: String = std::fs::read_to_string(claim.path.as_path()).expect("claim file");
    assert_eq!(recorded, claim.container().to_string_lossy());
    assert_eq!(claim.path.parent(), Some(journal_root(common.as_path()).as_path()));
    assert!(claim.path.to_string_lossy().ends_with(".stage"));
    assert_eq!(
        std::fs::metadata(claim.path.as_path()).expect("metadata").permissions().mode() & 0o777,
        0o600
    );
    remove(root.as_path());
}

/// Releasing a claim removes its file and nothing else; two claims have different containers.
#[test]
fn releasing_a_claim_removes_only_its_file() {
    let (root, common, destination) = scene("claim-release");
    let first: StageClaim = claim_stage(common.as_path(), destination.as_path()).expect("first");
    let second: StageClaim = claim_stage(common.as_path(), destination.as_path()).expect("second");
    assert_ne!(first.container(), second.container());
    let first_path: PathBuf = first.path.clone();
    let second_path: PathBuf = second.path.clone();
    first.release().expect("release");
    assert!(!first_path.exists());
    assert!(second_path.exists());
    second.release().expect("release");
    remove(root.as_path());
}

/// A claim failure names the claim and leaves no temporary file when the journal root cannot be created.
#[test]
fn a_claim_without_a_common_directory_fails() {
    let (root, common, destination) = scene("claim-no-common");
    let failure: WorktreeCopyError =
        claim_stage(common.join("absent").as_path(), destination.as_path()).expect_err("no common directory");
    assert!(!failure.0.is_empty());
    remove(root.as_path());
}

/// A claim exists for the pending check until it is released.
#[test]
fn claims_exist_until_released() {
    let (root, common, destination) = scene("claim-exist");
    assert!(!stage_claims_exist(common.as_path()).expect("no root"));
    ensure_journal_root(common.as_path()).expect("root");
    assert!(!stage_claims_exist(common.as_path()).expect("empty root"));
    let claim: StageClaim = claim_stage(common.as_path(), destination.as_path()).expect("claim");
    assert!(stage_claims_exist(common.as_path()).expect("claimed"));
    claim.release().expect("release");
    assert!(!stage_claims_exist(common.as_path()).expect("released"));
    remove(root.as_path());
}

/// Only claim files count: journals and unrelated files do not make the repository pending.
#[test]
fn other_files_are_not_claims() {
    let (root, common, _destination) = scene("claim-others");
    let journals: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    std::fs::write(journals.join("id.json"), b"{}").expect("journal-shaped file");
    std::fs::write(journals.join("notes.txt"), b"x").expect("note");
    assert!(!stage_claims_exist(common.as_path()).expect("others"));
    remove(root.as_path());
}

/// An abandoned stage is removed with its read-only directories, its claim goes, and one notice names the stage.
#[test]
fn an_abandoned_stage_is_removed_with_a_notice() {
    let (root, common, destination) = scene("claim-reap");
    let claim: StageClaim = claim_stage(common.as_path(), destination.as_path()).expect("claim");
    abandoned_stage(claim.container());
    let container: PathBuf = claim.container().to_path_buf();
    let claim_path: PathBuf = claim.path.clone();
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reaped");
    assert!(!container.exists());
    assert!(!claim_path.exists());
    assert_eq!(
        notices,
        vec![format!(
            "cli-git: removed the stage {} that an interrupted ignored-state copy left behind.",
            quote(container.to_string_lossy().as_ref())
        )]
    );
    remove(root.as_path());
}

/// A claim whose container was never created is removed quietly: nothing was staged.
#[test]
fn a_claim_without_a_container_is_removed_without_a_notice() {
    let (root, common, destination) = scene("claim-no-container");
    let claim: StageClaim = claim_stage(common.as_path(), destination.as_path()).expect("claim");
    let claim_path: PathBuf = claim.path.clone();
    assert!(reap_abandoned_stages(common.as_path()).expect("reaped").is_empty());
    assert!(!claim_path.exists());
    remove(root.as_path());
}

/// A stage that a pending journal names is kept for recovery; only its claim is removed.
#[test]
fn a_journaled_stage_is_kept() {
    let (root, common, destination) = scene("claim-journaled");
    let source: PathBuf = repository(root.as_path(), "source");
    std::fs::write(source.join(".gitignore"), b"cache/\n").expect("gitignore");
    std::fs::create_dir_all(source.join("cache")).expect("cache");
    std::fs::write(source.join("cache/data"), b"payload").expect("data");
    let claim: StageClaim = claim_stage(common.as_path(), destination.as_path()).expect("claim");
    let snapshot = stage_ignored_snapshot(
        &git_context(),
        source.as_path(),
        claim.container(),
        &[],
        &CopySelection::Ignored,
    )
    .expect("staged");
    let pending = create_journal(common.as_path(), destination.as_path(), &snapshot).expect("journal");
    let claim_path: PathBuf = claim.path.clone();
    assert!(reap_abandoned_stages(common.as_path()).expect("reaped").is_empty());
    assert!(snapshot.stage_container.exists(), "the journal owns the stage");
    assert!(pending.path.exists());
    assert!(!claim_path.exists());
    remove(root.as_path());
}

/// Every abandoned stage of a repository is removed in one pass, each with its own notice.
#[test]
fn every_abandoned_stage_is_removed() {
    let (root, common, destination) = scene("claim-reap-many");
    let first: StageClaim = claim_stage(common.as_path(), destination.as_path()).expect("first");
    let second: StageClaim = claim_stage(common.as_path(), destination.as_path()).expect("second");
    abandoned_stage(first.container());
    abandoned_stage(second.container());
    let notices: Vec<String> = reap_abandoned_stages(common.as_path()).expect("reaped");
    assert_eq!(notices.len(), 2);
    assert!(!first.container().exists());
    assert!(!second.container().exists());
    assert!(!stage_claims_exist(common.as_path()).expect("claims"));
    remove(root.as_path());
}

/// A claim that names a path outside the stage container naming is refused, and the named path survives.
#[test]
fn a_claim_naming_another_directory_is_refused() {
    let (root, common, destination) = scene("claim-foreign-name");
    let victim: PathBuf = destination.parent().expect("parent").join("precious");
    std::fs::create_dir_all(victim.as_path()).expect("victim");
    let claim: StageClaim = claim_stage(common.as_path(), destination.as_path()).expect("claim");
    std::fs::write(claim.path.as_path(), victim.to_string_lossy().as_bytes()).expect("rewrite claim");
    let failure: WorktreeCopyError = reap_abandoned_stages(common.as_path()).expect_err("refused");
    assert!(failure.0.contains("does not name a stage container"), "{}", failure.0);
    assert!(victim.exists());
    assert!(claim.path.exists(), "an unproven claim stays for inspection");
    remove(root.as_path());
}

/// A claim that names a relative path is refused even when its last component looks like a stage.
#[test]
fn a_claim_naming_a_relative_path_is_refused() {
    let (root, common, destination) = scene("claim-relative");
    let claim: StageClaim = claim_stage(common.as_path(), destination.as_path()).expect("claim");
    std::fs::write(claim.path.as_path(), b".cli-git-worktree-copy-relative").expect("rewrite claim");
    let failure: WorktreeCopyError = reap_abandoned_stages(common.as_path()).expect_err("refused");
    assert!(failure.0.contains("does not name a stage container"), "{}", failure.0);
    remove(root.as_path());
}

/// A claim that names a stage others can read is refused: the directory is not provably cli-git's own.
#[test]
fn a_shared_stage_is_refused_and_kept() {
    let (root, common, destination) = scene("claim-shared");
    let claim: StageClaim = claim_stage(common.as_path(), destination.as_path()).expect("claim");
    std::fs::create_dir(claim.container()).expect("container");
    std::fs::set_permissions(claim.container(), std::fs::Permissions::from_mode(0o755)).expect("shared");
    let failure: WorktreeCopyError = reap_abandoned_stages(common.as_path()).expect_err("refused");
    assert!(failure.0.contains("private stage is unsafe"), "{}", failure.0);
    assert!(claim.container().exists());
    remove(root.as_path());
}

/// A symbolic link in the container's place is refused, so a planted link never redirects the removal.
#[test]
fn a_linked_container_is_refused_and_its_target_kept() {
    let (root, common, destination) = scene("claim-linked");
    let target: PathBuf = root.join("target");
    std::fs::create_dir(target.as_path()).expect("target");
    std::fs::write(target.join("keep"), b"keep").expect("keep");
    let claim: StageClaim = claim_stage(common.as_path(), destination.as_path()).expect("claim");
    std::os::unix::fs::symlink(target.as_path(), claim.container()).expect("link");
    assert!(reap_abandoned_stages(common.as_path()).is_err());
    assert!(target.join("keep").exists());
    remove(root.as_path());
}

/// A claim file others can read is refused before it is believed.
#[test]
fn a_shared_claim_file_is_refused() {
    let (root, common, destination) = scene("claim-file-shared");
    let claim: StageClaim = claim_stage(common.as_path(), destination.as_path()).expect("claim");
    std::fs::set_permissions(claim.path.as_path(), std::fs::Permissions::from_mode(0o644)).expect("shared");
    let failure: WorktreeCopyError = reap_abandoned_stages(common.as_path()).expect_err("refused");
    assert!(failure.0.contains("journal file is unsafe"), "{}", failure.0);
    remove(root.as_path());
}

/// A claim write cut off before its rename leaves a temporary that the next pass removes without a notice.
#[test]
fn an_unfinished_claim_write_is_removed() {
    let (root, common, _destination) = scene("claim-temporary");
    let journals: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    let leftover: PathBuf = journals.join("abc.stage.def.tmp");
    let journal_temporary: PathBuf = journals.join("abc.json.def.tmp");
    std::fs::write(leftover.as_path(), b"cut off").expect("leftover");
    std::fs::write(journal_temporary.as_path(), b"a journal write").expect("journal temporary");
    assert!(reap_abandoned_stages(common.as_path()).expect("reaped").is_empty());
    assert!(!leftover.exists());
    assert!(journal_temporary.exists(), "journal temporaries belong to journal recovery");
    remove(root.as_path());
}

/// Without a journal root there is nothing to reap and nothing is created.
#[test]
fn a_repository_without_a_journal_root_reaps_nothing() {
    let (root, common, _destination) = scene("claim-no-root");
    assert!(reap_abandoned_stages(common.as_path()).expect("nothing").is_empty());
    assert!(!journal_root(common.as_path()).exists());
    remove(root.as_path());
}

/// A journal root that cannot be listed because a file stands where a directory belongs is a failure,
///  not an empty root: the claims and the unfinished writes under it were not looked at.
#[test]
fn an_unlistable_journal_root_is_a_failure_and_not_an_empty_one() {
    let (root, common, _destination) = scene("claim-root-not-a-directory");
    std::fs::write(common.join(JOURNAL_PARENT_NAME), b"a file where the directory belongs").expect("file");
    let unlistable: PathBuf = journal_root(common.as_path());
    claim_names(unlistable.as_path()).expect_err("the root cannot be listed");
    remove_unfinished_claims(unlistable.as_path()).expect_err("the root cannot be listed");
    remove(root.as_path());
}

/// Making a directory durable is fine when it succeeded or the directory was never created, and a fault otherwise.
#[test]
fn only_an_absent_directory_may_skip_its_sync() {
    absent_is_fine(Ok(())).expect("done");
    absent_is_fine(Err(std::io::Error::from(std::io::ErrorKind::NotFound))).expect("absent");
    absent_is_fine(Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied)))
        .expect_err("refused");
}

/// A claim whose container cannot be examined (a file stands in the way) fails and keeps its claim,
///  instead of being taken for a claim whose container was never created.
#[test]
fn a_container_that_cannot_be_examined_keeps_its_claim() {
    let (root, common, _destination) = scene("claim-container-blocked");
    let journals: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    let blocker: PathBuf = root.join("blocker");
    std::fs::write(blocker.as_path(), b"a file").expect("blocker");
    let claim: PathBuf = journals.join(format!("blocked{CLAIM_SUFFIX}"));
    let container: PathBuf = blocker.join(format!("{STAGE_PREFIX}blocked"));
    crate::private_storage::write_private_file(claim.as_path(), container.to_string_lossy().as_bytes())
        .expect("claim");
    reap_abandoned_stages(common.as_path()).expect_err("the container cannot be examined");
    assert!(claim.exists(), "the claim stays for inspection");
    remove(root.as_path());
}
