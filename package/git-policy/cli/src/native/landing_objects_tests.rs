//! Controls for migrating a shadow repository's objects into a kept pack of the real store.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::pack_keep::{remove_pack_keep, transaction_keep_message};
use crate::transaction_journal::Base;
use std::path::PathBuf;

/// Builds a shadow whose alternates name the real store, and commits `message` on `parent` inside it.
fn shadow_commit(root: &Path, repo: &Path, parent: Option<&str>, message: &str) -> (PathBuf, String) {
    let shadow: PathBuf = root.join("shadow");
    git(root, &["init", "--quiet", "--bare", shadow.to_str().expect("UTF-8 path")]);
    let alternates = shadow.join("objects").join("info").join("alternates");
    std::fs::create_dir_all(alternates.parent().expect("info directory")).expect("info directory");
    let real_objects = repo.join(".git").join("objects");
    std::fs::write(&alternates, format!("{}\n", real_objects.display())).expect("alternates");
    let tree: String = git_text(repo, &["rev-parse", "HEAD^{tree}"]);
    let git_dir: String = format!("--git-dir={}", shadow.display());
    let mut arguments: Vec<&str> = vec![git_dir.as_str(), "commit-tree", tree.as_str(), "-m", message];
    if let Some(parent_oid) = parent {
        arguments.push("-p");
        arguments.push(parent_oid);
    }
    let oid: String = git_text(root, &arguments);
    return (shadow, oid);
}

/// The new commit's objects reach the real store as a pack, which an index-pack report names.
#[test]
fn migrated_commit_is_readable_from_the_real_store() {
    let root = fixture("landing-objects-migrate");
    let repo = repository(root.as_path(), "repo");
    let initial = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let (shadow, commit) = shadow_commit(root.as_path(), repo.as_path(), Some(initial.as_str()), "second");
    let base = Base::Commit(initial.clone());
    let pack = migrate_shadow_objects(&git_context(), repo.as_path(), shadow.as_path(), &commit, &base, None)
        .expect("migration");
    assert_eq!(pack.len(), 40);
    assert!(repo.join(".git/objects/pack").join(format!("pack-{pack}.pack")).exists());
    assert_eq!(git_text(repo.as_path(), &["log", "-1", "--format=%s", commit.as_str()]), "second");
    remove(&root);
}

/// A keep message writes a `.keep` file naming the transaction, and removing the keep deletes it.
#[test]
fn keep_names_the_transaction_and_is_removed_after_the_swap() {
    let root = fixture("landing-objects-keep");
    let repo = repository(root.as_path(), "repo");
    let initial = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let (shadow, commit) = shadow_commit(root.as_path(), repo.as_path(), Some(initial.as_str()), "second");
    let base = Base::Commit(initial);
    let message = transaction_keep_message("0b6c");
    let pack = migrate_shadow_objects(&git_context(), repo.as_path(), shadow.as_path(), &commit, &base, Some(message.as_str()))
        .expect("migration");
    let keep = repo.join(".git/objects/pack").join(format!("pack-{pack}.keep"));
    assert_eq!(std::fs::read_to_string(&keep).expect("keep file").trim_end(), "cli-git 0b6c");
    remove_pack_keep(repo.join(".git/objects").as_path(), pack.as_str()).expect("keep removal");
    assert!(!keep.exists());
    remove(&root);
}

/// Without a keep message no `.keep` is written, so the pack is not protected.
#[test]
fn no_keep_message_writes_no_keep() {
    let root = fixture("landing-objects-nokeep");
    let repo = repository(root.as_path(), "repo");
    let initial = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let (shadow, commit) = shadow_commit(root.as_path(), repo.as_path(), Some(initial.as_str()), "second");
    let base = Base::Commit(initial);
    let pack = migrate_shadow_objects(&git_context(), repo.as_path(), shadow.as_path(), &commit, &base, None)
        .expect("migration");
    assert!(!repo.join(".git/objects/pack").join(format!("pack-{pack}.keep")).exists());
    remove(&root);
}

/// An unborn base packs the whole new history except what the real store already holds.
#[test]
fn unborn_base_migrates_a_root_commit() {
    let root = fixture("landing-objects-unborn");
    let repo = repository(root.as_path(), "repo");
    let (shadow, commit) = shadow_commit(root.as_path(), repo.as_path(), None, "root");
    let pack = migrate_shadow_objects(&git_context(), repo.as_path(), shadow.as_path(), &commit, &Base::Unborn, None)
        .expect("migration");
    assert!(repo.join(".git/objects/pack").join(format!("pack-{pack}.pack")).exists());
    assert_eq!(git_text(repo.as_path(), &["log", "-1", "--format=%s", commit.as_str()]), "root");
    remove(&root);
}

/// A commit the shadow does not hold fails the migration with the Git error, and migrates nothing.
#[test]
fn missing_commit_fails_the_migration() {
    let root = fixture("landing-objects-missing");
    let repo = repository(root.as_path(), "repo");
    let (shadow, _) = shadow_commit(root.as_path(), repo.as_path(), None, "root");
    let missing: String = "f".repeat(40);
    let result = migrate_shadow_objects(&git_context(), repo.as_path(), shadow.as_path(), &missing, &Base::Unborn, None);
    assert!(result.is_err());
    remove(&root);
}
