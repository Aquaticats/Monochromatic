//! Controls of the synchronization of one created worktree:
//! the stage it fills,
//! the journal it leaves,
//! and the registration it is bound to.

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};
use crate::worktree_copy_journal::read_pending_journals;
use crate::worktree_copy_stage_claim::stage_claims_exist;

/// A source repository whose ignored `cache` holds one nested file,
/// and a destination directory beside it.
fn ignored_source(root: &Path) -> (PathBuf, PathBuf) {
    let source: PathBuf = repository(root, "source");
    std::fs::write(source.join(".gitignore"), b"cache/\n").expect("gitignore");
    std::fs::create_dir_all(source.join("cache/deep")).expect("cache");
    std::fs::write(source.join("cache/deep/data"), b"payload").expect("data");
    std::fs::write(source.join("cache/top"), b"top").expect("top");
    let destination: PathBuf = root.join("topic");
    return (
        std::fs::canonicalize(source).expect("canonical source"),
        destination,
    );
}

/// The common Git directory of a repository.
fn common_of(source: &Path) -> PathBuf {
    return std::fs::canonicalize(source.join(".git")).expect("common directory");
}

/// A complete synchronization installs the ignored state and leaves neither a journal nor a stage behind.
#[cfg(unix)]
#[test]
fn completed_synchronization_leaves_no_journal_or_stage() {
    let root: PathBuf = fixture("transaction-complete");
    let (source, destination) = ignored_source(root.as_path());
    git(
        source.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "topic",
            destination.to_str().expect("utf8"),
        ],
    );
    let destination_canonical: PathBuf =
        std::fs::canonicalize(&destination).expect("canonical destination");
    let common: PathBuf = common_of(source.as_path());
    let copied: usize = synchronize_created_worktree(
        &git_context(),
        common.as_path(),
        source.as_path(),
        destination_canonical.as_path(),
        &[],
        &CopySelection::Ignored,
    )
    .expect("synchronized");
    assert_eq!(copied, 4);
    assert_eq!(
        std::fs::read(destination_canonical.join("cache/deep/data")).expect("data"),
        b"payload"
    );
    assert!(
        read_pending_journals(common.as_path())
            .expect("pending")
            .is_empty()
    );
    let leftovers: Vec<String> = std::fs::read_dir(destination_canonical.parent().expect("parent"))
        .expect("list")
        .map(|entry| {
            return entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned();
        })
        .filter(|name| return name.starts_with(".cli-git-worktree-copy-"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
    assert!(
        !stage_claims_exist(common.as_path()).expect("claims"),
        "a settled copy releases its stage claim"
    );
    remove(root.as_path());
}

/// A copy that fails while staging removes its stage and releases its claim,
/// so nothing is left to reap.
#[cfg(unix)]
#[test]
fn failed_staging_releases_its_claim() {
    let root: PathBuf = fixture("transaction-staging-failure");
    let (source, _destination) = ignored_source(root.as_path());
    let common: PathBuf = common_of(source.as_path());
    let destination: PathBuf = root.join("absent").join("topic");
    let failure: WorktreeCopyError = synchronize_created_worktree(
        &git_context(),
        common.as_path(),
        source.as_path(),
        destination.as_path(),
        &[],
        &CopySelection::Ignored,
    )
    .expect_err("the stage parent is absent");
    assert!(!failure.0.is_empty());
    assert!(!stage_claims_exist(common.as_path()).expect("claims"), "the failed copy released its claim");
    assert!(read_pending_journals(common.as_path()).expect("pending").is_empty());
    remove(root.as_path());
}

/// A copy whose destination is another registration by the time it installs stops before the first entry,
/// names both registrations,
/// and leaves nothing pending.
#[cfg(unix)]
#[test]
fn a_copy_into_a_replaced_registration_stops_before_the_first_entry() {
    use crate::worktree_add_binding::bind_registration;
    let root: PathBuf = fixture("transaction-registration-replaced");
    let (source, destination) = ignored_source(root.as_path());
    git(
        source.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "topic",
            destination.to_str().expect("utf8"),
        ],
    );
    let destination_canonical: PathBuf = std::fs::canonicalize(&destination).expect("canonical destination");
    let common: PathBuf = common_of(source.as_path());
    let binding = bind_registration(common.as_path(), destination_canonical.as_path()).expect("bound").binding;
    // Another command removes this worktree and registers another one at the same path, without a checkout yet.
    git(
        source.as_path(),
        &[
            "worktree",
            "remove",
            "--force",
            destination_canonical.to_str().expect("utf8"),
        ],
    );
    git(
        source.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            "-b",
            "second",
            destination_canonical.to_str().expect("utf8"),
        ],
    );
    let failure: WorktreeCopyError = synchronize_created_worktree_noting(
        &git_context(),
        common.as_path(),
        source.as_path(),
        destination_canonical.as_path(),
        &[],
        &CopySelection::Ignored,
        None,
        Some(&binding),
        &mut Vec::new(),
    )
    .expect_err("the registration the copy was made for is gone");
    assert!(
        failure.0.contains(binding.admin.to_str().expect("utf8")),
        "the registration the copy was made for is named: {}",
        failure.0
    );
    assert!(failure.0.contains("Nothing in that worktree was changed"), "{}", failure.0);
    assert!(!destination_canonical.join("cache").exists(), "the replacement registration received nothing");
    assert!(read_pending_journals(common.as_path()).expect("pending").is_empty());
    assert!(!stage_claims_exist(common.as_path()).expect("claims"));
    remove(root.as_path());
}
