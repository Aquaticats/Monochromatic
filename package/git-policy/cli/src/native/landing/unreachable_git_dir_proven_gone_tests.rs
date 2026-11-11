//! What:
//!  Controls of what the filesystem proves about a Git directory the service can no longer resolve,
//!  and of what the
//!  drain does with the plans when it is asked.
//! Why:
//!  The two cases the release could not tell apart are the two the owner named:
//! a drive that is not mounted yet,
//!  whose push returns when it is,
//! and a repository that was deleted,
//! whose plans can never run.
//! One control is here for each
//!  side of that line,
//! and the drain's own answer is checked at the drain,
//! not at the proof alone:
//!  a plan that ends is a plan no later round keeps.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(prove(goneRepo, nothingMounted)).toEqual({ gone: { nearest, because: 'ordinary-directory' } });
//! expect((await drain(goneRepo, due)).changes).toEqual([{ forget: branch }]);
//! ```

use super::{Gone, GoneBecause, Mounts, NothingIsMounted, Proof, TheMountTable, gone_line, prove};
use crate::claim_time::ClaimTime;
use crate::push_retry::{RetryChange, Retrying};
use crate::service_claim_renewal_running::Renewal;
use crate::service_work::{SERVICE_KINDS, Verdict};
use crate::service_work_drains_awaiting::drain_planned_with_awaiting;
use crate::test_support::{fixture, remove};
use crate::work_item::{WorkItem, WorkKind};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The first pause of a plan,
/// as a service that retries states it.
const FIRST_PAUSE: Duration = Duration::from_secs(600);

/// The branch every control pushes.
const BRANCH: &str = "refs/heads/main";

/// What:
///  A directory that holds entries,
/// with the Git directory of a repository under it that is then deleted.
/// Why:
///   This is the repository another agent ran with the installed wrapper and then deleted:
/// the tree is gone,
/// and the
///  directory that held it still holds other things.
fn a_deleted_repository(root: &Path) -> (PathBuf, PathBuf) {
    let holder: PathBuf = root.join("work");
    let repository: PathBuf = holder.join("repository-a");
    std::fs::create_dir_all(repository.join(".git").as_path()).expect("the Git directory of a repository");
    std::fs::write(holder.join("kept.txt"), "another entry").expect("an entry beside it");
    std::fs::remove_dir_all(repository.as_path()).expect("the repository is deleted");
    return (repository.join(".git"), repository);
}

/// What:
///  A directory that holds nothing,
/// with a path under it that no repository has made yet.
/// Why:
///   This is what a mount point looks like while its drive is not mounted:
/// the directory is there,
/// and the
///  filesystem that would hold the repository is not.
fn a_path_behind_an_unmounted_mount_point(root: &Path) -> PathBuf {
    let mount_point: PathBuf = root.join("drive");
    std::fs::create_dir_all(mount_point.as_path()).expect("the mount point's directory");
    return mount_point.join("repository/.git");
}

/// What:
///  A mount table in which every directory is a mount point,
/// so a control can state the reading without a mount of its own.
/// Why:
///   A deleted repository on a mounted drive is the case the line's reason names;
/// the running system of a control
///  may have no such drive.
struct EverythingIsMounted;

impl Mounts for EverythingIsMounted {
    fn is_mount_point(&self, _path: &Path) -> bool {
        return true;
    }
}

/// What:
///  A repository that was deleted is proven gone,
/// by the directory that held it still holding other entries.
/// Why:
///   Nothing is missing but the repository itself,
/// so no try can reach it again and its plans can never run.
#[test]
fn a_deleted_repository_is_proven_gone() {
    let root: PathBuf = fixture("unreachable-deleted");
    let (git_dir, worktree): (PathBuf, PathBuf) = a_deleted_repository(root.as_path());
    assert_eq!(
        prove(git_dir.as_path(), Some(worktree.as_path()), &NothingIsMounted),
        Proof::Gone(Gone { nearest: root.join("work"), because: GoneBecause::OrdinaryDirectory }),
        "the directory that held the repository holds other entries, so the repository was deleted"
    );
    remove(root.as_path());
}

/// What:
///  A path behind a directory that holds nothing may still return,
/// so its plan is kept as the owner asked.
/// Why:
///   An empty directory is what a mount point looks like while its drive is not mounted,
/// and the plan is what tries the
///  push when the drive is back.
#[test]
fn a_path_behind_an_unmounted_mount_point_may_return() {
    let root: PathBuf = fixture("unreachable-unmounted");
    let git_dir: PathBuf = a_path_behind_an_unmounted_mount_point(root.as_path());
    assert_eq!(
        prove(git_dir.as_path(), Some(root.join("drive/repository").as_path()), &NothingIsMounted),
        Proof::MayReturn(super::MayReturn { nearest: root.join("drive") }),
        "nothing is mounted at the directory, and it holds nothing, so a drive may still arrive"
    );
    remove(root.as_path());
}

/// What:
///  The root of a mounted filesystem above a missing path proves the repository gone.
/// Why:
///   Every name below that root would live on a filesystem that is mounted and answering,
/// so only someone writing the
///  repository again brings it back,
/// and the line says which root it looked at.
#[test]
fn the_root_of_a_mounted_filesystem_proves_the_repository_gone() {
    let root: PathBuf = fixture("unreachable-mounted-root");
    let git_dir: PathBuf = a_path_behind_an_unmounted_mount_point(root.as_path());
    assert_eq!(
        prove(git_dir.as_path(), Some(root.join("drive/repository").as_path()), &EverythingIsMounted),
        Proof::Gone(Gone { nearest: root.join("drive"), because: GoneBecause::MountedFilesystemRoot }),
        "a mounted filesystem's root is answering, so the rest was deleted from it"
    );
    remove(root.as_path());
}

/// What:
///  The line names the path whose plans ended,
/// the directory the proof looked at,
/// and what proved it.
/// Why:
///   A person reading the journal has to be able to tell a drive that is not mounted from a repository the service gave
///  up on,
/// and the branches that lost their tries.
#[test]
fn the_line_names_the_path_the_proof_and_the_branches() {
    let root: PathBuf = fixture("unreachable-line");
    let (git_dir, worktree): (PathBuf, PathBuf) = a_deleted_repository(root.as_path());
    let _worktree: &Path = worktree.as_path();
    let gone: Gone = Gone { nearest: root.join("work"), because: GoneBecause::OrdinaryDirectory };
    let said: String = gone_line(git_dir.as_path(), &gone, &[String::from("refs/heads/main of 0123")]);
    assert!(said.contains(git_dir.display().to_string().as_str()), "the path is named: {said}");
    assert!(said.contains(root.join("work").display().to_string().as_str()), "the directory the proof looked at is named: {said}");
    assert!(said.contains("is a directory that holds other entries"), "the proof is named: {said}");
    assert!(said.contains("refs/heads/main of 0123"), "the branch and commit that lost their tries are named: {said}");
    assert!(said.contains("ends without another try"), "and that no try follows: {said}");
    remove(root.as_path());
}

/// What:
///  A drain of a repository that is gone ends its plan and keeps no new one.
/// Why:
///   This is the state the owner reported:
/// a plan kept for a directory nothing can reach is a try that runs forever.
/// The
///  change it asks the loop to apply is the one that drops the branch,
/// and no plan of that branch comes back.
#[test]
fn a_deleted_repository_ends_its_plans() {
    let root: PathBuf = fixture("unreachable-drain-gone");
    let (git_dir, worktree): (PathBuf, PathBuf) = a_deleted_repository(root.as_path());
    let drained: crate::service_work::Drained = crate::merge_second_pass::tests::await_it(drain_planned_with_awaiting(
        git_dir.as_path(),
        &Quiet,
        &ClaimTime::read(),
        crate::service_work::CLAIM_LEASE,
        Renewal::Timed,
        &retrying_for(git_dir.as_path(), worktree.as_path()),
        &SERVICE_KINDS,
    ));
    assert!(
        !drained.changes.iter().any(|change| return matches!(change, RetryChange::Plan { .. })),
        "no plan is kept for a repository that is gone: {:?}",
        drained.changes
    );
    assert_eq!(drained.changes.len(), 1, "one change, and it drops the branch: {:?}", drained.changes);
    assert!(
        matches!(&drained.changes[0], RetryChange::Forget { git_dir: named, branch_ref } if named.as_path() == git_dir.as_path() && branch_ref == BRANCH),
        "the plan of the branch ends where it was kept: {:?}",
        drained.changes
    );
    remove(root.as_path());
}

/// What:
///  A drain of a path behind an empty directory keeps its plan,
/// which is what an unmounted drive needs.
/// Why:
///   The control of the line's other side:
/// the same drain,
/// the same kind of missing path,
/// and a plan that stays because the
///  filesystem has not proven the repository gone.
#[test]
fn a_path_behind_an_unmounted_mount_point_keeps_its_plans() {
    let root: PathBuf = fixture("unreachable-drain-kept");
    let git_dir: PathBuf = a_path_behind_an_unmounted_mount_point(root.as_path());
    let drained: crate::service_work::Drained = crate::merge_second_pass::tests::await_it(drain_planned_with_awaiting(
        git_dir.as_path(),
        &Quiet,
        &ClaimTime::read(),
        crate::service_work::CLAIM_LEASE,
        Renewal::Timed,
        &retrying_for(git_dir.as_path(), root.join("drive/repository").as_path()),
        &SERVICE_KINDS,
    ));
    assert!(
        !drained.changes.iter().any(|change| return matches!(change, RetryChange::Forget { .. })),
        "nothing ends a plan for a directory that may return: {:?}",
        drained.changes
    );
    assert_eq!(drained.changes.len(), 1, "one change, and it keeps the plan: {:?}", drained.changes);
    assert!(
        matches!(&drained.changes[0], RetryChange::Plan { git_dir: named, attempt, .. } if named.as_path() == git_dir.as_path() && *attempt == 5),
        "the plan keeps its branch with the next try: {:?}",
        drained.changes
    );
    remove(root.as_path());
}

/// What:
///  The mount table of the running system names the root of this process's tree as a mount point.
/// Why:
///   The proof's first reading rests on the kernel's own table,
/// and a table that answered nothing would turn every
///  mounted filesystem's root into an ordinary directory,
/// so a deleted repository there would keep its plans.
#[cfg(target_os = "linux")]
#[test]
fn the_real_mount_table_names_the_root() {
    assert!(TheMountTable::default().is_mount_point(Path::new("/")), "the root of the tree is a mount point");
}

/// A drain whose directory may return asks no handler anything,
/// and one whose repository is gone asks none either.
struct Quiet;

impl crate::service_work::WorkHandlers for Quiet {
    fn push(&self, _items: &[WorkItem]) -> Verdict {
        panic!("a drain of an unreachable directory runs nothing");
    }
    fn maintenance(&self, _item: &WorkItem) -> Verdict {
        panic!("a drain of an unreachable directory runs nothing");
    }
    fn review(&self, _item: &WorkItem) -> Verdict {
        panic!("a drain of an unreachable directory runs nothing");
    }
}

/// What:
///  The retry a service hands a drain whose directory cannot be reached:
/// one push of the branch,
/// due now,
/// on its fourth try,
/// with a plan kept in memory.
/// Why:
///   The proof is asked about the plans a drain already has,
/// so the control states one exactly as a round that failed to push it did.
fn retrying_for(git_dir: &Path, worktree: &Path) -> Retrying {
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: worktree.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        branch_ref: String::from(BRANCH),
        oid: String::from("0123456789abcdef0123456789abcdef01234567"),
        parent: String::from("fedcba9876543210fedcba9876543210fedcba98"),
        created: String::from("2026-11-10T17:50:25.196Z"),
        tries: 3,
    };
    return Retrying { first: Some(FIRST_PAUSE), due: vec![crate::push_retry::Due { git_dir: git_dir.to_path_buf(), item, attempt: 4 }], plans: None };
}

/// What:
///  A Git directory that was moved away from a worktree that is still there may return.
/// Why:
///   Nothing of the repository was deleted:
/// its own Git directory is not where it was,
/// and a repair or a rename puts it
///  back,
/// so the plan waits as it always has.
#[test]
fn a_git_directory_that_moved_away_from_its_worktree_may_return() {
    let root: PathBuf = fixture("unreachable-moved-away");
    let worktree: PathBuf = root.join("worktree");
    let git_dir: PathBuf = worktree.join(".git");
    std::fs::create_dir_all(git_dir.as_path()).expect("the Git directory");
    std::fs::rename(git_dir.as_path(), root.join("git-dir-away").as_path()).expect("the Git directory goes away");
    assert_eq!(
        prove(git_dir.as_path(), Some(worktree.as_path()), &NothingIsMounted),
        Proof::MayReturn(super::MayReturn { nearest: worktree.clone() }),
        "the worktree is still there, so the repository was not deleted"
    );
    remove(root.as_path());
}
