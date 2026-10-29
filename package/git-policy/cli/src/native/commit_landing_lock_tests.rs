//! Controls for the landing lock helpers.

use super::*;
use crate::owner_lock::{DEFAULT_POLL_DELAY, SilentWait, acquire_owner_lock};
use crate::transaction_lease::LANDING_LEASE_VARIABLE;
use crate::test_support::{fixture, remove};
use crate::transaction_journal::RefFormat;

/// A layout whose registry lives under `root`.
fn layout(root: &Path) -> InvocationLayout {
    return InvocationLayout {
        symbolic_head: crate::transaction_journal::SymbolicHead::Branch(String::from("refs/heads/main")),
        target_ref: String::from("refs/heads/main"),
        conclusion: crate::transaction_journal::Conclusion::None,
        repository_root: root.to_path_buf(),
        git_dir: root.join(".git"),
        common_dir: root.join(".git"),
        real_index_path: root.join(".git/index"),
        object_directory: root.join(".git/objects"),
        registry_root: root.join(".git/cli-git-transactions"),
        ref_format: RefFormat::Files,
        object_format: String::from("sha1"),
        empty_tree_oid: "4".repeat(40),
        invoked_at: String::from("2026-10-11T00:00:00.000Z"),
        config_snapshot: None,
    };
}

/// The lock lives in the registry under its fixed name.
#[test]
fn the_lock_directory_is_in_the_registry() {
    assert_eq!(
        landing_lock_directory(&layout(Path::new("/r"))),
        PathBuf::from("/r/.git/cli-git-transactions/landing.lock")
    );
}

/// A lease naming a lock that its token really owns makes a landing nested, and the landing then takes the ancestor's
///  view of that lock; any other lease or none leaves it a landing of its own.
#[test]
fn a_valid_lease_makes_the_landing_nested_and_hands_back_the_ancestors_lock() {
    use crate::test_support::{REAL_GIT, repository};
    use crate::transaction_lease::format_landing_lease;
    let root: PathBuf = fixture("landing-lock-nested");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut layout: InvocationLayout = layout(repo.as_path());
    layout.registry_root = repo.join(".git/cli-git-transactions");
    std::fs::create_dir_all(layout.registry_root.as_path()).expect("registry");
    let directory: PathBuf = landing_lock_directory(&layout);
    let mut silent: SilentWait = SilentWait;
    let ancestor: OwnerLock = acquire_owner_lock(directory.as_path(), DEFAULT_POLL_DELAY, &mut silent).expect("ancestor lock");
    let lease: String = format_landing_lease(directory.to_str().expect("path"), ancestor.token());
    let environment: Vec<(OsString, OsString)> =
        vec![(OsString::from(LANDING_LEASE_VARIABLE), OsString::from(lease))];
    assert!(landing_is_nested(&layout, environment.as_slice()));
    assert!(!landing_is_nested(&layout, &[]));
    let forged: String = format_landing_lease(directory.to_str().expect("path"), "not-the-token");
    assert!(!landing_is_nested(
        &layout,
        &[(OsString::from(LANDING_LEASE_VARIABLE), OsString::from(forged))]
    ));
    let context: GitContext = GitContext {
        real_git: PathBuf::from(REAL_GIT),
        overlay: Vec::new(),
        global_prefix: Vec::new(),
    };
    let inherited: OwnerLock =
        acquire_landing_lock(&context, repo.as_path(), &layout, environment.as_slice()).expect("the ancestor's view");
    assert_eq!(inherited.directory(), directory.as_path());
    assert_eq!(inherited.token(), ancestor.token());
    drop(inherited);
    assert!(directory.exists(), "the ancestor's lock is not released by the nested view");
    ancestor.release().expect("release");
    remove(root.as_path());
}
