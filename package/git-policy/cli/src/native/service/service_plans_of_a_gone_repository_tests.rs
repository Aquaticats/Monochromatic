//! What:
//!  Controls for the pushes a service owes a repository that no longer exists:
//!  the plans of a deleted repository end,
//!  the plans of a repository that is there and of one behind a mount point that holds nothing stay,
//!  and no other record of the journal is touched.
//! Why:
//!  A plan that stays is a try a service started later spends on a directory nothing can reach,
//! and a journal whose records a cleanup removed is work a person may still want,
//! so each side of both lines has a control here.

use super::*;
use crate::push_retry_store::{StoredPlan, load, save};
use crate::test_support::{fixture, remove};
use crate::work_item::{WorkItem, WorkKind};

/// The branch every control's plan pushes.
const BRANCH: &str = "refs/heads/main";

/// The commit every control's plan names,
/// a different one per plan so the lines it gives back can be told apart.
fn commit(named: &str) -> String {
    return format!("{named:0<40}");
}

/// A plan of one branch of one Git directory,
/// as a round that failed to push wrote it.
fn plan(git_dir: &Path, repository: &Path, oid: &str) -> StoredPlan {
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: repository.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        branch_ref: String::from(BRANCH),
        oid: String::from(oid),
        parent: String::from("fedcba9876543210fedcba9876543210fedcba98"),
        created: String::from("2026-11-11T08:55:00.000-05:00"),
        tries: 148,
    };
    return StoredPlan { git_dir: git_dir.to_path_buf(), item, attempt: 149 };
}

/// One repository under `holder` that is then deleted,
/// beside an entry that keeps `holder` holding something.
fn a_deleted_repository(holder: &Path, name: &str) -> (PathBuf, PathBuf) {
    let repository: PathBuf = holder.join(name);
    let git_dir: PathBuf = repository.join(".git");
    std::fs::create_dir_all(git_dir.as_path()).expect("the Git directory of a repository");
    std::fs::write(holder.join("kept.txt"), "another entry").expect("an entry beside it");
    std::fs::remove_dir_all(repository.as_path()).expect("the repository is deleted");
    return (git_dir, repository);
}

/// What:
///  The plan of a deleted repository ends,
///  and the plans of a repository that is there and of one behind a mount point that holds nothing stay.
/// Why:
///   This is the state the owner reported:
/// a unit of a `/tmp` repository deleted hours before,
/// retrying at try 149.
/// The two plans that stay are what the rule must not take:
/// a repository whose push can still run,
/// and a drive that is not mounted yet.
#[test]
fn a_plan_of_a_deleted_repository_ends_and_the_plans_of_repositories_that_are_there_stay() {
    let root: PathBuf = fixture("plans-of-a-gone-repository");
    let journal: PathBuf = root.join("journal");
    let plans: PathBuf = journal.join(crate::push_retry::PLAN_DIRECTORY);
    // A repository that is there, whose push a later round must still be able to take.
    let alive_repository: PathBuf = root.join("alive");
    let alive_git_dir: PathBuf = alive_repository.join(".git");
    std::fs::create_dir_all(alive_git_dir.as_path()).expect("a repository that is there");
    // A repository that was deleted under a directory that holds other entries.
    let (gone_git_dir, gone_repository) = a_deleted_repository(root.join("work").as_path(), "repository-a");
    // A path behind a mount point that is there and holds nothing, as a drive that is not mounted leaves it.
    let mount_point: PathBuf = root.join("drive");
    std::fs::create_dir_all(mount_point.as_path()).expect("mount point");
    let unmounted_git_dir: PathBuf = mount_point.join("repository/.git");
    let unmounted_repository: PathBuf = mount_point.join("repository");
    for kept in [
        plan(alive_git_dir.as_path(), alive_repository.as_path(), &commit("aaaa"),),
        plan(gone_git_dir.as_path(), gone_repository.as_path(), &commit("bbbb"),),
        plan(unmounted_git_dir.as_path(), unmounted_repository.as_path(), &commit("cccc"),),
    ] {
        save(plans.as_path(), &kept).expect("the plan is kept");
    }
    let ended: Vec<String> = end_plans_of_gone_repositories(journal.as_path());
    assert_eq!(ended, vec![format!("{BRANCH} of {}", commit("bbbb"))], "the push of the deleted repository is the one that ends");
    let mut still_planned: Vec<PathBuf> = load(plans.as_path()).into_iter().map(|plan| { return plan.git_dir; }).collect();
    still_planned.sort();
    assert_eq!(
        still_planned,
        vec![alive_git_dir.clone(), unmounted_git_dir.clone()],
        "a repository that is there and a drive that may arrive keep their plans"
    );
    remove(root.as_path());
}

/// What:
///  The other records of the journal are left alone:
///  the records of the batches that landed and the folders of the service's work.
/// Why:
///   Ending a plan must not be the moment a commit a person made is discarded,
/// and the journal is removed only when nothing but the records of landed batches is in it.
#[test]
fn ending_a_plan_leaves_the_other_records_of_the_journal_alone() {
    let root: PathBuf = fixture("plans-of-a-gone-journal");
    let journal: PathBuf = root.join("journal");
    let plans: PathBuf = journal.join(crate::push_retry::PLAN_DIRECTORY);
    let (gone_git_dir, gone_repository) = a_deleted_repository(root.join("work").as_path(), "repository-a");
    save(plans.as_path(), &plan(gone_git_dir.as_path(), gone_repository.as_path(), &commit("bbbb"),)).expect("the plan is kept");
    let landed: PathBuf = journal.join(format!("7.{}", crate::service_journal::DONE_EXTENSION));
    std::fs::write(landed.as_path(), "seq=7\n").expect("a finished batch");
    let watched: PathBuf = journal.join(crate::service_claim_watch::WATCH_DIRECTORY);
    std::fs::create_dir_all(watched.as_path()).expect("the folder of the repositories an earlier service was woken for");
    std::fs::write(watched.join("repository"), "a wake it was asked for\n").expect("a note of a wake");
    assert_eq!(end_plans_of_gone_repositories(journal.as_path()).len(), 1);
    assert!(landed.exists(), "the record of a batch that landed stays");
    assert!(watched.exists(), "the folder of the repositories the service was woken for stays");
    assert_eq!(load(plans.as_path()).len(), 0, "only the plan of the deleted repository ended");
    remove(root.as_path());
}
