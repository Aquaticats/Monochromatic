//! What:
//!  Controls of the device's folder:
//!  a folder a fresh boot does not have yet is made at its designed mode,
//!  a path another account owns is refused in words rather than taken,
//!  and the three names the design gives the folder are the ones it uses.
//! Why:
//!  The claim,
//! the mark and the budget share one folder,
//!  and the folder's proof is what stops a pass acting inside something another
//!  account made
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "The records",
//!  and veto entry V6).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const folder = SweepsFolder.at(fixture, process.getuid()); folder.prepare();
//! ```

use super::{
    PASS_MARK_MODE,
    PASS_MARK_NAME,
    PASS_MARK_TEXT,
    RUN_CLAIM_PREFIX,
    SWEEPS_FOLDER,
    SWEEPS_FOLDER_MODE,
    SweepsFolder,
};
use crate::test_support::fixture;
use std::os::unix::fs::{
    MetadataExt,
    PermissionsExt,
};

/// The account this test runs as,
/// which is the owner a fixture folder has.
fn owner() -> u32 {
    return rustix::process::geteuid().as_raw();
}

/// The three names the design gives the folder are the ones this build uses.
#[test]
fn the_designed_names_are_the_ones_this_build_uses() {
    assert_eq!(SWEEPS_FOLDER, "/run/git-worktree-clone-mount-sweeps");
    assert_eq!(RUN_CLAIM_PREFIX, "run");
    assert_eq!(PASS_MARK_NAME, "another-pass-is-due");
    assert_eq!(PASS_MARK_TEXT, "v1");
    assert_eq!(SWEEPS_FOLDER_MODE, 0o755);
    assert_eq!(PASS_MARK_MODE, 0o644);
}

/// A folder a fresh boot does not have is made at its designed mode.
#[test]
fn a_folder_a_fresh_boot_lacks_is_made_at_its_mode() {
    let root: std::path::PathBuf = fixture("sweeps-folder-fresh");
    let folder: SweepsFolder = SweepsFolder::at(root.join("run").join("sweeps"), owner());
    folder.prepare().expect("a folder this account may make");
    let facts: std::fs::Metadata =
        std::fs::metadata(folder.directory()).expect("the folder that was made");
    assert!(facts.is_dir(), "the folder is a directory");
    assert_eq!(facts.permissions().mode() & 0o7777, SWEEPS_FOLDER_MODE);
    assert_eq!(facts.uid(), owner());
    assert_eq!(
        folder.pass_mark_path(),
        folder.directory().join(PASS_MARK_NAME)
    );
    assert_eq!(
        folder.run_claim_file(3),
        folder.directory().join(format!("{RUN_CLAIM_PREFIX}.3"))
    );
    crate::test_support::remove(root.as_path());
}

/// A folder another account owns is refused in words,
/// and the refusal says what cannot be used and why.
#[test]
fn a_folder_another_account_owns_is_refused_in_words() {
    let root: std::path::PathBuf = fixture("sweeps-folder-owner");
    let directory: std::path::PathBuf = root.join("sweeps");
    std::fs::create_dir_all(directory.as_path()).expect("the fixture folder");
    // The proof is asked for an owner this account is not, which is the state a folder another account made leaves.
    let folder: SweepsFolder = SweepsFolder::at(directory.clone(), owner() + 1);
    let refusal: String = folder.prepare().expect_err("another account's folder");
    assert!(
        refusal.contains("is owned by user"),
        "the refusal names the owner it found: {refusal}"
    );
    assert!(
        refusal.contains(directory.to_string_lossy().as_ref()),
        "the refusal names the folder: {refusal}"
    );
    assert_eq!(
        std::fs::metadata(directory.as_path())
            .expect("the folder that was refused")
            .permissions()
            .mode()
            & 0o7777,
        0o755,
        "the refused folder keeps the mode it was made with"
    );
    crate::test_support::remove(root.as_path());
}
