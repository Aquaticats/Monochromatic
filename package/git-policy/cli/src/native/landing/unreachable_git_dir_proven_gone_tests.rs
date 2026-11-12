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

use super::{Gone, GoneBecause, Mounts, NothingIsMounted, Proof, gone_line, prove, the_mounts};
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
///  A path behind a directory this process may not search may return,
///  because an access error proves nothing about it.
/// Why:
///  "never discard user data":
/// a directory whose mode,
/// an access control list,
/// or a filesystem that stopped answering refuses the search is not a directory that is gone;
/// the repository behind it may be there all along,
/// and a plan read as gone there would delete a live repository's only retry record.
/// The proof is asked both ways it is asked in the release:
/// with the worktree a plan names,
/// and without one,
/// as the service's own question asks it.
#[test]
fn a_path_behind_a_directory_that_cannot_be_searched_may_return() {
    use std::os::unix::fs::PermissionsExt;
    if unsafe { libc::geteuid() } == 0 {
        eprintln!("skipped: a directory that refuses the search does not stop root, so this control proves nothing here");
        return;
    }
    let root: PathBuf = fixture("unreachable-denied-search");
    let private: PathBuf = root.join("private");
    let repository: PathBuf = private.join("repository");
    let git_dir: PathBuf = repository.join(".git");
    std::fs::create_dir_all(git_dir.as_path()).expect("the Git directory of a repository");
    // The directory that holds the repository refuses the search, and the directory above it holds other entries, so a walk that
    //  read the refusal as absence would reach an ordinary directory and prove the repository gone.
    std::fs::set_permissions(private.as_path(), std::fs::Permissions::from_mode(0o000)).expect("the directory refuses the search");
    assert_eq!(
        prove(git_dir.as_path(), Some(repository.as_path()), &NothingIsMounted),
        Proof::MayReturn(super::MayReturn { nearest: repository.clone() }),
        "a refused search is not a deletion, so the worktree's repository may still be there"
    );
    assert_eq!(
        prove(git_dir.as_path(), None, &NothingIsMounted),
        Proof::MayReturn(super::MayReturn { nearest: git_dir.clone() }),
        "a walk that was refused at its first step proves nothing about the path"
    );
    std::fs::set_permissions(private.as_path(), std::fs::Permissions::from_mode(0o700)).expect("the directory is searchable again");
    assert!(git_dir.exists(), "the repository was there the whole time");
    remove(root.as_path());
}

/// What:
///  A directory that is there but whose entries cannot be listed may return.
/// Why:
///  An empty directory is what a mount point looks like while its drive is not mounted,
/// and whether this one holds anything was refused;
/// reading the refusal as "it holds other entries" would end the plan of a live repository deleted from it.
#[test]
fn a_path_behind_a_directory_whose_entries_cannot_be_listed_may_return() {
    use std::os::unix::fs::PermissionsExt;
    if unsafe { libc::geteuid() } == 0 {
        eprintln!("skipped: a directory that refuses the read does not stop root, so this control proves nothing here");
        return;
    }
    let root: PathBuf = fixture("unreachable-denied-listing");
    let repository: PathBuf = root.join("repository");
    let git_dir: PathBuf = repository.join(".git");
    std::fs::create_dir_all(repository.as_path()).expect("the directory the repository would be in");
    // Search without read: the Git directory can be looked for, and the entries of the directory it would be in cannot be listed.
    std::fs::set_permissions(repository.as_path(), std::fs::Permissions::from_mode(0o100)).expect("the directory refuses the listing");
    assert_eq!(
        prove(git_dir.as_path(), None, &NothingIsMounted),
        Proof::MayReturn(super::MayReturn { nearest: repository.clone() }),
        "a directory whose entries were refused is not proven to hold the other entries of a deleted repository"
    );
    std::fs::set_permissions(repository.as_path(), std::fs::Permissions::from_mode(0o700)).expect("the directory is listable again");
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
        matches!(&drained.changes[0], RetryChange::Forget { git_dir: named, branch_ref, .. } if named.as_path() == git_dir.as_path() && branch_ref == BRANCH),
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
    assert!(the_mounts().is_mount_point(Path::new("/")), "the root of the tree is a mount point");
}

/// Environment variable that makes the control below run its inner half,
/// inside a mount namespace of its own.
#[cfg(target_os = "linux")]
const MOUNT_SIDE: &str = "UNREACHABLE_GIT_DIR_TEST_MOUNT_SIDE";

/// What:
///  The mount table is read again for every question,
///  so a drive mounted after an earlier check,
///  whose only repository is then deleted,
///  is proven gone by the next.
/// Why:
///   A serving service asks whether its worktree is still there every minute
/// (`service_server::OWN_WORKTREE_CHECK`),
/// and a drive may be mounted between two of those questions.
/// A table kept from the first check kept saying the drive was not mounted,
/// so the empty root of a mounted filesystem read as a mount point waiting for its drive,
/// the plan never ended,
/// and the service of a deleted repository never asked for its socket to be stopped.
/// The sequence needs a mount,
/// so the inner half runs under `unshare` in a private mount namespace,
/// and the control skips where the host lets no unprivileged process mount.
#[cfg(target_os = "linux")]
#[test]
#[allow(clippy::disallowed_methods, reason = "starts this test binary, not the wrapper, under `unshare`; the ban on `current_exe` guards the wrapper's own start")]
fn the_mount_table_is_read_again_after_a_drive_is_mounted() {
    if std::env::var_os(MOUNT_SIDE).is_none() {
        let probe: std::process::Output = std::process::Command::new("unshare")
            .args(["--user", "--map-root-user", "--mount", "--propagation", "private", "--", "mount", "--version"])
            .output()
            .expect("unshare runs");
        if !probe.status.success() {
            eprintln!("skipped: this host lets no unprivileged process mount in a namespace of its own, so no table changes here");
            return;
        }
        let done: std::process::Output = std::process::Command::new("unshare")
            .args(["--user", "--map-root-user", "--mount", "--propagation", "private"])
            .arg(std::env::current_exe().expect("this test binary"))
            .args(["--exact", "landing::unreachable_git_dir_proven_gone::tests::the_mount_table_is_read_again_after_a_drive_is_mounted", "--nocapture"])
            .env(MOUNT_SIDE, "1")
            .output()
            .expect("the inner half runs");
        let said: String = String::from_utf8_lossy(done.stdout.as_slice()).to_string() + String::from_utf8_lossy(done.stderr.as_slice()).as_ref();
        assert!(done.status.success(), "the inner half failed:\n{said}");
        assert!(said.contains("1 passed"), "the inner half did not run:\n{said}");
        return;
    }
    let root: PathBuf = fixture("unreachable-table-again");
    let drive: PathBuf = root.join("drive");
    std::fs::create_dir_all(drive.as_path()).expect("a mount point that is there");
    let repository: PathBuf = drive.join("repository");
    let git_dir: PathBuf = repository.join(".git");
    // The drive is not mounted and its mount point holds nothing, so a repository may still arrive with it.
    assert_eq!(
        prove(git_dir.as_path(), None, &the_mounts()),
        Proof::MayReturn(super::MayReturn { nearest: drive.clone() }),
        "an empty directory that no filesystem is mounted on may still receive the repository"
    );
    let mounted: MountedTmpfs = MountedTmpfs::at(drive.as_path());
    // The drive is mounted, and the only repository on it is deleted, so its root holds nothing at all.
    std::fs::create_dir_all(git_dir.as_path()).expect("a repository on the mounted drive");
    std::fs::remove_dir_all(repository.as_path()).expect("the repository is deleted");
    assert_eq!(
        prove(git_dir.as_path(), None, &the_mounts()),
        Proof::Gone(Gone { nearest: drive.clone(), because: GoneBecause::MountedFilesystemRoot }),
        "the table is read again, so the mounted root that is answering proves the repository gone"
    );
    assert!(
        crate::service_units_of_removed_worktrees::worktree_is_gone(git_dir.as_path()),
        "the question the service asks reads the same table and answers the same"
    );
    drop(mounted);
    remove(root.as_path());
}

/// A tmpfs this control mounted,
/// unmounted when it goes however the control ends,
/// so an assertion that failed does not leave the fixture's mount point busy.
#[cfg(target_os = "linux")]
struct MountedTmpfs {
    /// The directory the filesystem was mounted on.
    at: PathBuf,
}

#[cfg(target_os = "linux")]
impl MountedTmpfs {
    /// Mount a tmpfs on `path`,
    /// which this process may only do as root in a mount namespace of its own.
    fn at(path: &Path) -> Self {
        let mounted: std::process::Output = std::process::Command::new("mount").args(["-t", "tmpfs", "tmpfs"]).arg(path).output().expect("the mount command runs");
        assert!(
            mounted.status.success(),
            "a tmpfs is mounted at {}: {}",
            path.display(),
            String::from_utf8_lossy(mounted.stderr.as_slice())
        );
        return Self { at: path.to_path_buf() };
    }
}

#[cfg(target_os = "linux")]
impl Drop for MountedTmpfs {
    fn drop(&mut self) {
        let _unmounted: std::io::Result<std::process::Output> = std::process::Command::new("umount").arg(self.at.as_path()).output();
    }
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

/// What:
///  A repository renamed away and back before the change lands keeps the push plan its drain ended,
///  even though the round took that plan out of the running service's schedule to hand it to the drain.
/// Why:
///   The proof that ends a plan is a reading of the filesystem,
///  and what it asks for is applied after that reading:
///  a repository that is back by then is reachable,
///  so its plan is not a try that can never run,
///  and dropping it leaves the repository's only scheduled backup gone.
/// The plan the drain ended is the one the round removed before the drain ran,
///  so keeping it means putting that plan back:
///  a control that schedules a plan of its own hides the loss,
///  because nothing then removes the plan the running service holds.
#[test]
fn a_repository_renamed_back_before_the_change_lands_keeps_its_plans() {
    let root: PathBuf = fixture("unreachable-renamed-back");
    let holder: PathBuf = root.join("work");
    let repository: PathBuf = holder.join("repository-a");
    let moved: PathBuf = holder.join("repository-away");
    let git_dir: PathBuf = repository.join(".git");
    std::fs::create_dir_all(git_dir.as_path()).expect("the Git directory");
    std::fs::write(holder.join("kept.txt"), "another entry").expect("an entry beside it");
    let plans: PathBuf = root.join("plans");
    std::fs::create_dir_all(plans.as_path()).expect("the plan directory");
    let mut retries: crate::push_retry::PushRetries = crate::push_retry::PushRetries::restored(FIRST_PAUSE, plans.as_path());
    let stored: Retrying = retrying_for(git_dir.as_path(), repository.as_path());
    for due in &stored.due {
        let _pause: Duration = retries.schedule(git_dir.as_path(), &due.item, 1, std::time::Instant::now());
    }
    // The round of the service's work takes the due plan out of the schedule before it drains the directory, and hands the plan
    //  to the worker; the plan the drain ends is therefore this one, and it is the one the repository's return must keep
    //  (`service_work_run_round.rs`, `plan_the_round`).
    let taken: Vec<crate::push_retry::Due> = retries.take_due(git_dir.as_path(), std::time::Instant::now() + FIRST_PAUSE + Duration::from_secs(1));
    assert_eq!(taken.len(), 1, "the due plan left the running service's schedule, as it does before a round's drain");
    let retrying: Retrying = Retrying { first: Some(FIRST_PAUSE), due: taken, plans: Some(plans.clone()) };

    std::fs::rename(repository.as_path(), moved.as_path()).expect("the whole repository is renamed, not deleted");
    let drained: crate::service_work::Drained = crate::merge_second_pass::tests::await_it(drain_planned_with_awaiting(
        git_dir.as_path(),
        &Quiet,
        &ClaimTime::read(),
        crate::service_work::CLAIM_LEASE,
        Renewal::Timed,
        &retrying,
        &SERVICE_KINDS,
    ));
    assert!(
        matches!(&drained.changes[0], RetryChange::Forget { .. }),
        "the drain proves the path gone while the repository is away: {:?}",
        drained.changes
    );

    std::fs::rename(moved.as_path(), repository.as_path()).expect("the repository is renamed back");
    retries.apply(drained.changes);

    assert!(
        retries.next_wait(git_dir.as_path(), std::time::Instant::now()).is_some(),
        "the running service still has a wait for the repository whose plan its drain ended on a reading that went stale"
    );
    let due: Vec<crate::push_retry::Due> = retries.take_due(git_dir.as_path(), std::time::Instant::now() + FIRST_PAUSE + Duration::from_secs(1));
    assert_eq!(due.len(), 1, "the repository is back, so its one scheduled push stays");
    remove(root.as_path());
}
