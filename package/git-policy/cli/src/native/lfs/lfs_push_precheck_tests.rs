//! Controls for the refusal made before a push that carries the upload hook:
//! when `git-lfs` is missing from `PATH`,
//! and
//! the push shapes that never need it.

use super::*;
use crate::test_support::{executable, fixture, remove};

/// An argument list from words.
fn words(items: &[&str]) -> Vec<OsString> {
    return items.iter().map(OsString::from).collect();
}

/// An environment whose `PATH` is the given directory.
fn path_environment(directory: &std::path::Path) -> Vec<(OsString, OsString)> {
    return vec![(OsString::from("PATH"), directory.as_os_str().to_owned())];
}

/// A `PATH` without the program refuses a push that carries the hook,
/// with the explanation,
/// and a `PATH` with it does not.
#[test]
fn a_push_is_refused_only_when_the_program_is_missing() {
    let root = fixture("lfs-precheck-missing");
    let empty = root.join("empty");
    let full = root.join("full");
    std::fs::create_dir(&empty).expect("directory");
    std::fs::create_dir(&full).expect("directory");
    executable(full.join("git-lfs").as_path(), b"#!/bin/sh\n");
    let region: Vec<OsString> = words(&["origin", "main"]);
    let refusal: String = lfs_refusal(&[], region.as_slice(), path_environment(&empty).as_slice()).expect("refused");
    let search: crate::lfs_pre_push::ProgramSearch = crate::lfs_pre_push::ProgramSearch::of(path_environment(&empty).as_slice());
    assert_eq!(refusal, missing_program_message(&search));
    assert!(refusal.contains(&empty.display().to_string()), "the refusal names the folder it searched: {refusal}");
    assert_eq!(lfs_refusal(&[], region.as_slice(), path_environment(&full).as_slice()), None);
    assert!(lfs_refusal(&[], region.as_slice(), &[]).is_some(), "an unset PATH finds nothing");
    remove(root.as_path());
}

/// A push that bypasses the hook,
/// sends nothing,
/// or switched the hook off never needs the program.
#[test]
fn a_push_that_runs_no_hook_is_never_refused() {
    let root = fixture("lfs-precheck-bypass");
    let environment = path_environment(root.as_path());
    for region in [
        words(&["--no-verify", "origin", "main"]),
        words(&["--no-ver", "origin", "main"]),
        words(&["origin", "main", "--no-verify"]),
        words(&["--dry-run", "origin", "main"]),
    ] {
        assert_eq!(lfs_refusal(&[], region.as_slice(), environment.as_slice()), None, "{region:?}");
    }
    let off: Vec<OsString> = words(&["-c", "hook.cli-git-lfs-pre-push.enabled=false"]);
    assert_eq!(lfs_refusal(off.as_slice(), words(&["origin"]).as_slice(), environment.as_slice()), None);
    let region_after_terminator: Vec<OsString> = words(&["origin", "--", "--no-verify"]);
    assert!(
        lfs_refusal(&[], region_after_terminator.as_slice(), environment.as_slice()).is_some(),
        "a word after -- is a refspec, not an option"
    );
    remove(root.as_path());
}

/// A region Git itself would refuse is left to Git.
#[test]
fn a_region_git_would_refuse_is_left_to_git() {
    let environment: Vec<(OsString, OsString)> = Vec::new();
    assert_eq!(lfs_refusal(&[], words(&["--no-such-option"]).as_slice(), environment.as_slice()), None);
}
