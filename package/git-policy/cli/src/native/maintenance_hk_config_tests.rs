//! Controls for the hk Git-config cleanup: the listing parser and the removal against real Git.

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The listing parser keeps only keys of the requested scope that start with the hk prefix.
#[test]
fn listing_selects_hk_keys_of_the_requested_scope() {
    let listing: &[u8] = b"local\0hook.hk-a.command\0global\0hook.hk-c.command\0local\0user.name\0";
    assert_eq!(
        hk_keys_in_scope(listing, HkConfigScope::Local).expect("parsed"),
        vec![String::from("hook.hk-a.command")]
    );
    assert_eq!(
        hk_keys_in_scope(listing, HkConfigScope::Global).expect("parsed"),
        vec![String::from("hook.hk-c.command")]
    );
}

/// Duplicate keys collapse, and the result is sorted in UTF-16 code-unit order as JavaScript's sort is.
#[test]
fn duplicate_keys_collapse_and_sort_by_utf16_units() {
    let listing: &[u8] = "local\0hook.hk-b.command\0local\0hook.hk-a.command\0local\0hook.hk-b.command\0local\0hook.hk-\u{1F600}.command\0local\0hook.hk-\u{FFFD}.command\0".as_bytes();
    assert_eq!(
        hk_keys_in_scope(listing, HkConfigScope::Local).expect("parsed"),
        vec![
            String::from("hook.hk-a.command"),
            String::from("hook.hk-b.command"),
            String::from("hook.hk-\u{1F600}.command"),
            String::from("hook.hk-\u{FFFD}.command"),
        ]
    );
}

/// The prefix match is exact: a key that only resembles it is left alone.
#[test]
fn prefix_match_is_exact() {
    let listing: &[u8] = b"local\0hook.hkx-a.command\0local\0hook.hk-\0local\0hook.h\0";
    assert_eq!(
        hk_keys_in_scope(listing, HkConfigScope::Local).expect("parsed"),
        vec![String::from("hook.hk-")]
    );
}

/// An empty listing has no keys.
#[test]
fn empty_listing_has_no_keys() {
    assert_eq!(hk_keys_in_scope(b"", HkConfigScope::Local).expect("empty"), Vec::<String>::new());
}

/// A listing that is only a NUL leaves one unpaired field, which the incumbent throws for as well.
#[test]
fn listing_of_only_a_nul_is_malformed() {
    assert_eq!(
        hk_keys_in_scope(b"\0", HkConfigScope::Local),
        Err(HkConfigCleanupError::MalformedListing)
    );
}

/// An empty field in the middle of the listing is kept as a field, and it is not an hk key.
#[test]
fn middle_empty_field_is_kept_as_a_field() {
    let listing: &[u8] = b"local\0\0local\0hook.hk-x\0";
    assert_eq!(
        hk_keys_in_scope(listing, HkConfigScope::Local).expect("parsed"),
        vec![String::from("hook.hk-x")]
    );
}

/// A scope without a key is malformed listing output, as the incumbent throws for it.
#[test]
fn scope_without_key_is_malformed() {
    assert_eq!(
        hk_keys_in_scope(b"local\0hook.hk-a.command\0global\0", HkConfigScope::Global),
        Err(HkConfigCleanupError::MalformedListing)
    );
}

/// A failed removal names the key and the exit status.
#[test]
fn failed_removal_error_names_the_key() {
    let error: HkConfigCleanupError = HkConfigCleanupError::Failed {
        key: String::from("hook.hk-a.command"),
        code: 5,
    };
    assert_eq!(
        error.to_string(),
        "git config --unset-all hook.hk-a.command exited 5"
    );
}

/// Real Git: the local scope loses its hk keys and keeps every other key, and a second run is a no-op.
#[test]
fn local_cleanup_removes_only_hk_keys_and_is_idempotent() {
    let root: PathBuf = fixture("hk-local");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["config", "hook.hk-pre-commit.command", "echo hk"]);
    git(repo.as_path(), &["config", "--add", "hook.hk-pre-commit.command", "echo again"]);
    git(repo.as_path(), &["config", "hook.keep.command", "echo keep"]);
    let context: GitContext = git_context();
    let removed: HkConfigCleanupResult = cleanup_hk_git_config(
        &context,
        HkConfigScope::Local,
        repo.as_path(),
    )
    .expect("cleanup ran");
    assert_eq!(removed.removed_keys, vec![String::from("hook.hk-pre-commit.command")]);
    assert_eq!(
        crate::test_support::git_text(repo.as_path(), &["config", "--get", "hook.keep.command"]),
        "echo keep"
    );
    let again: HkConfigCleanupResult = cleanup_hk_git_config(
        &context,
        HkConfigScope::Local,
        repo.as_path(),
    )
    .expect("second cleanup ran");
    assert_eq!(again.removed_keys, Vec::<String>::new());
    remove(root.as_path());
}

/// Real Git: the global scope is cleaned in its own file and the local scope is not touched by a global run.
#[test]
fn global_cleanup_reads_and_writes_only_the_global_file() {
    let root: PathBuf = fixture("hk-global");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let global: PathBuf = root.join("global.gitconfig");
    write_global(global.as_path());
    git(repo.as_path(), &["config", "hook.hk-local.command", "echo local"]);
    let mut context: GitContext = git_context();
    context.overlay.retain(|(name, _)| return name.as_os_str() != OsString::from("GIT_CONFIG_GLOBAL").as_os_str());
    context.overlay.push((OsString::from("GIT_CONFIG_GLOBAL"), global.clone().into_os_string()));
    let removed: HkConfigCleanupResult =
        cleanup_hk_git_config(&context, HkConfigScope::Global, repo.as_path()).expect("global ran");
    assert_eq!(removed.removed_keys, vec![String::from("hook.hk-global.command")]);
    assert!(!std::fs::read_to_string(global.as_path()).expect("global file").contains("hk-global"));
    assert_eq!(
        crate::test_support::git_text(repo.as_path(), &["config", "--get", "hook.hk-local.command"]),
        "echo local"
    );
    remove(root.as_path());
}

/// Write a global configuration file holding one hk key.
fn write_global(path: &Path) {
    std::fs::write(path, "[hook \"hk-global\"]\n\tcommand = echo global\n").expect("global file");
}

/// Real Git: a damaged configuration fails the listing with Git's status, and nothing is removed.
#[test]
fn damaged_configuration_fails_the_listing() {
    let root: PathBuf = fixture("hk-damaged");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(".git").join("config"), b"[core\n\tbogus\n").expect("damage config");
    assert!(matches!(
        cleanup_hk_git_config(&git_context(), HkConfigScope::Local, repo.as_path()),
        Err(HkConfigCleanupError::ListingFailed(code)) if code != 0
    ));
    remove(root.as_path());
}

/// A working directory that does not exist means Git cannot start there.
#[test]
fn missing_working_directory_is_a_start_failure() {
    let root: PathBuf = fixture("hk-missing");
    assert!(matches!(
        cleanup_hk_git_config(&git_context(), HkConfigScope::Local, root.join("absent").as_path()),
        Err(HkConfigCleanupError::Start(_))
    ));
    remove(root.as_path());
}
