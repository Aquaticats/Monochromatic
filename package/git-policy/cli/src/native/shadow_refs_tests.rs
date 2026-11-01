//! Controls for the shadow ref store, on both ref backends.

use super::*;
use crate::private_storage::take_file_syncs;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// A bare shadow repository in the fixture, with the given ref backend.
fn bare_shadow(root: &Path, ref_format: &str, real_objects: &Path) -> PathBuf {
    let shadow: PathBuf = root.join("shadow");
    let format_argument: String = format!("--ref-format={ref_format}");
    git(
        root,
        &[
            "init",
            "--bare",
            "--quiet",
            "--template=",
            format_argument.as_str(),
            shadow.to_str().expect("fixture"),
        ],
    );
    std::fs::write(shadow.join("objects").join("info").join("alternates"), format!("{}\n", real_objects.display())).expect("alternates");
    return shadow;
}

/// A repository with a tag, a symbolic branch and a second branch, so the snapshot has every kind.
fn decorated_repository(root: &Path) -> (PathBuf, String) {
    let repo: PathBuf = repository(root, "repo");
    git(repo.as_path(), &["tag", "v1"]);
    git(repo.as_path(), &["branch", "Zeta"]);
    git(repo.as_path(), &["symbolic-ref", "refs/heads/alias", "refs/heads/main"]);
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    return (repo, head);
}

/// Packed text names the header, every direct ref except the target, and no symbolic ref.
#[test]
fn packed_refs_skip_the_target_and_symbolic_refs() {
    let refs: Vec<RefSnapshotEntry> = vec![
        RefSnapshotEntry {
            name: String::from("refs/heads/alias"),
            oid: String::new(),
            symref: String::from("refs/heads/main"),
        },
        RefSnapshotEntry {
            name: String::from("refs/heads/main"),
            oid: String::from("1111111111111111111111111111111111111111"),
            symref: String::new(),
        },
        RefSnapshotEntry {
            name: String::from("refs/tags/v1"),
            oid: String::from("2222222222222222222222222222222222222222"),
            symref: String::new(),
        },
    ];
    assert_eq!(
        format_packed_refs(&refs, "refs/heads/main"),
        "# pack-refs with: sorted \n2222222222222222222222222222222222222222 refs/tags/v1\n"
    );
}

/// The real ref listing sorts by raw bytes, so `Zeta` precedes `main`, and marks symbolic refs.
#[test]
fn real_refs_sort_by_bytes_and_mark_symbolic_refs() {
    let root: PathBuf = fixture("shadow-refs-list");
    let (repo, _head) = decorated_repository(root.as_path());
    let refs: Vec<RefSnapshotEntry> = list_real_refs(&git_context(), repo.as_path()).expect("refs");
    let names: Vec<&str> = refs.iter().map(|entry| { return entry.name.as_str(); }).collect();
    let zeta: usize = names.iter().position(|name| { return *name == "refs/heads/Zeta"; }).expect("Zeta");
    let main: usize = names.iter().position(|name| { return *name == "refs/heads/main"; }).expect("main");
    assert!(zeta < main, "{names:?}");
    let alias = refs.iter().find(|entry| { return entry.name == "refs/heads/alias"; }).expect("alias");
    assert_eq!(alias.symref, "refs/heads/main");
    remove(root.as_path());
}

/// A files shadow holds the snapshot, the loose target, the symbolic ref and `HEAD` on the branch.
#[test]
fn files_shadow_snapshots_refs_and_points_head_at_the_branch() {
    let root: PathBuf = fixture("shadow-refs-files");
    let (repo, head) = decorated_repository(root.as_path());
    let shadow: PathBuf = bare_shadow(root.as_path(), "files", repo.join(".git").join("objects").as_path());
    write_shadow_refs(
        &git_context(),
        repo.as_path(),
        shadow.as_path(),
        RefFormat::Files,
        "refs/heads/main",
        &SymbolicHead::Branch(String::from("refs/heads/main")),
        &Base::Commit(head.clone()),
    )
    .expect("refs");
    assert_eq!(git_text(shadow.as_path(), &["symbolic-ref", "HEAD"]), "refs/heads/main");
    assert_eq!(git_text(shadow.as_path(), &["rev-parse", "refs/heads/main"]), head);
    assert_eq!(git_text(shadow.as_path(), &["rev-parse", "refs/tags/v1"]), head);
    assert_eq!(git_text(shadow.as_path(), &["symbolic-ref", "refs/heads/alias"]), "refs/heads/main");
    remove(root.as_path());
}

/// The files snapshot is published whole and made no file sync: the shadow is thrown away, and nothing reads its refs
///  after a crash.
#[test]
fn the_files_snapshot_is_not_synced() {
    let root: PathBuf = fixture("shadow-refs-files-sync");
    let (repo, head) = decorated_repository(root.as_path());
    let shadow: PathBuf = bare_shadow(root.as_path(), "files", repo.join(".git").join("objects").as_path());
    take_file_syncs();
    write_shadow_refs(
        &git_context(),
        repo.as_path(),
        shadow.as_path(),
        RefFormat::Files,
        "refs/heads/main",
        &SymbolicHead::Branch(String::from("refs/heads/main")),
        &Base::Commit(head),
    )
    .expect("refs");
    assert!(shadow.join("packed-refs").is_file());
    assert_eq!(take_file_syncs(), Vec::<PathBuf>::new());
    remove(root.as_path());
}

/// A detached shadow has no symbolic `HEAD`, only the base as a direct value.
#[test]
fn detached_shadow_head_is_the_base() {
    let root: PathBuf = fixture("shadow-refs-detached");
    let (repo, head) = decorated_repository(root.as_path());
    let shadow: PathBuf = bare_shadow(root.as_path(), "files", repo.join(".git").join("objects").as_path());
    write_shadow_refs(
        &git_context(),
        repo.as_path(),
        shadow.as_path(),
        RefFormat::Files,
        "HEAD",
        &SymbolicHead::Detached,
        &Base::Commit(head.clone()),
    )
    .expect("refs");
    assert_eq!(git_text(shadow.as_path(), &["rev-parse", "HEAD"]), head);
    assert!(run_git_checked(
        &git_context(),
        &shadow_request(shadow.as_path(), shadow.as_path(), &["symbolic-ref", "-q", "HEAD"]),
    )
    .is_err());
    remove(root.as_path());
}

/// A detached `HEAD` without a base cannot be written, and the failure says why.
#[test]
fn detached_without_a_base_fails() {
    let root: PathBuf = fixture("shadow-refs-detached-unborn");
    let (repo, _head) = decorated_repository(root.as_path());
    let shadow: PathBuf = bare_shadow(root.as_path(), "files", repo.join(".git").join("objects").as_path());
    let error: GitFailure = write_shadow_refs(
        &git_context(),
        repo.as_path(),
        shadow.as_path(),
        RefFormat::Files,
        "HEAD",
        &SymbolicHead::Detached,
        &Base::Unborn,
    )
    .expect_err("no base");
    assert_eq!(error.0, "A detached HEAD has no unborn preparation base.");
    remove(root.as_path());
}

/// An unborn branch names the branch in `HEAD` but writes no target ref.
#[test]
fn unborn_branch_has_no_target_ref() {
    let root: PathBuf = fixture("shadow-refs-unborn");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repo");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    let shadow: PathBuf = bare_shadow(root.as_path(), "files", repo.join(".git").join("objects").as_path());
    write_shadow_refs(
        &git_context(),
        repo.as_path(),
        shadow.as_path(),
        RefFormat::Files,
        "refs/heads/main",
        &SymbolicHead::Branch(String::from("refs/heads/main")),
        &Base::Unborn,
    )
    .expect("refs");
    assert_eq!(git_text(shadow.as_path(), &["symbolic-ref", "HEAD"]), "refs/heads/main");
    assert!(run_git_checked(
        &git_context(),
        &shadow_request(shadow.as_path(), shadow.as_path(), &["rev-parse", "--verify", "refs/heads/main"]),
    )
    .is_err());
    remove(root.as_path());
}

/// A reftable shadow receives the same snapshot through one `update-ref --stdin` transaction.
#[test]
fn reftable_shadow_snapshots_refs_in_one_transaction() {
    let root: PathBuf = fixture("shadow-refs-reftable");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repo");
    git(repo.as_path(), &["init", "--quiet", "--ref-format=reftable", "--initial-branch=main"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    git(repo.as_path(), &["tag", "v1"]);
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let shadow: PathBuf = bare_shadow(root.as_path(), "reftable", repo.join(".git").join("objects").as_path());
    write_shadow_refs(
        &git_context(),
        repo.as_path(),
        shadow.as_path(),
        RefFormat::Reftable,
        "refs/heads/main",
        &SymbolicHead::Branch(String::from("refs/heads/main")),
        &Base::Commit(head.clone()),
    )
    .expect("refs");
    assert_eq!(git_text(shadow.as_path(), &["rev-parse", "refs/tags/v1"]), head);
    assert_eq!(git_text(shadow.as_path(), &["symbolic-ref", "HEAD"]), "refs/heads/main");
    remove(root.as_path());
}
