//! Controls for the hook presence decision: which entries count as a hook, and which failures keep the entry.

use super::*;
use crate::hook_dispatch_plan::ConfigParameters;
use crate::test_support::{executable, fixture, remove};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// A plan whose only meaningful fields are the hooks directory and whether the configuration names hooks.
fn plan(hooks_path: &Path, named_hooks_possible: bool) -> HookDispatchPlan {
    return HookDispatchPlan {
        git_path: String::from(crate::test_support::REAL_GIT),
        hooks_path: hooks_path.display().to_string(),
        disabled_events: Vec::new(),
        config_parameters: ConfigParameters::Absent,
        worktree_root: String::new(),
        lease: String::new(),
        hook_lock_path: String::new(),
        skip_hook_lock: false,
        named_hooks_possible,
    };
}

/// A missing entry, and a missing directory, prove that no hook exists.
#[test]
fn a_missing_entry_or_directory_proves_absence() {
    let root: PathBuf = fixture("hook-presence-absent");
    assert!(!hook_entry_may_exist(root.as_path(), "pre-commit"));
    assert!(!hook_entry_may_exist(root.join("no-such-directory").as_path(), "pre-commit"));
    std::fs::write(root.join("pre-commit.sample"), b"#!/bin/sh\n").expect("sample");
    assert!(!hook_entry_may_exist(root.as_path(), "pre-commit"), "a sample is not a hook");
    std::fs::write(root.join("commit-msg"), b"#!/bin/sh\n").expect("other event");
    assert!(!hook_entry_may_exist(root.as_path(), "pre-commit"), "another event's hook is not this one");
    remove(root.as_path());
}

/// An entry of any file type or mode may be a hook, or earns Git's not-executable hint.
#[test]
fn an_entry_of_any_type_or_mode_may_exist() {
    let root: PathBuf = fixture("hook-presence-entries");
    executable(root.join("pre-commit").as_path(), b"#!/bin/sh\n");
    std::fs::write(root.join("commit-msg"), b"#!/bin/sh\n").expect("plain file");
    std::fs::create_dir(root.join("prepare-commit-msg")).expect("directory");
    std::os::unix::fs::symlink(root.join("nowhere"), root.join("post-commit")).expect("dangling link");
    for event in ["pre-commit", "commit-msg", "prepare-commit-msg", "post-commit"] {
        assert!(hook_entry_may_exist(root.as_path(), event), "{event}");
    }
    let mode: u32 = std::fs::metadata(root.join("commit-msg")).expect("metadata").permissions().mode();
    assert_eq!(mode & 0o111, 0, "the plain file is not executable");
    remove(root.as_path());
}

/// A failure to look that is not "not found" keeps the entry: a hooks path that is a file, and an unsearchable directory.
#[test]
fn a_lookup_failure_other_than_not_found_keeps_the_entry() {
    let root: PathBuf = fixture("hook-presence-errors");
    std::fs::write(root.join("hooks-is-a-file"), b"").expect("file");
    assert!(hook_entry_may_exist(root.join("hooks-is-a-file").as_path(), "pre-commit"));
    let sealed: PathBuf = root.join("sealed");
    std::fs::create_dir(sealed.as_path()).expect("directory");
    std::fs::set_permissions(sealed.as_path(), std::fs::Permissions::from_mode(0o000)).expect("seal");
    let searchable: bool = std::fs::symlink_metadata(sealed.join("pre-commit")).is_err_and(|error| {
        return error.kind() == ErrorKind::NotFound;
    });
    // A privileged user can search a sealed directory; the check then proves absence, as Git would see it.
    assert_eq!(hook_entry_may_exist(sealed.as_path(), "pre-commit"), !searchable);
    std::fs::set_permissions(sealed.as_path(), std::fs::Permissions::from_mode(0o755)).expect("unseal");
    remove(root.as_path());
}

/// A configuration that names hooks keeps every event, and otherwise the directory is looked at when asked.
#[test]
fn named_hooks_keep_every_event_and_the_directory_is_read_when_asked() {
    let root: PathBuf = fixture("hook-presence-plan");
    let named: HookDispatchPlan = plan(root.as_path(), true);
    assert!(event_may_have_user_hook(&named, "pre-commit"), "named hooks keep an event with no file");
    let unnamed: HookDispatchPlan = plan(root.as_path(), false);
    assert!(!event_may_have_user_hook(&unnamed, "post-commit"));
    executable(root.join("post-commit").as_path(), b"#!/bin/sh\n");
    assert!(event_may_have_user_hook(&unnamed, "post-commit"), "a hook created later is seen");
    remove(root.as_path());
}
