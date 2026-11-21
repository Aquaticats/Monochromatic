//! What:
//!  Controls of the sweep's refusal:
//!  its mode name,
//!  its plan step,
//!  and that running it answers the refusal rather than a sweep's line.
//! Why:
//!  The default invocation is what the installed ZED event starts,
//!  so a build that answered it with success would report a sweep nobody ran
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//! step 8).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(AWAITING).toEqual({ mode: 'sweep (no argument)', step: 8, does: expect.any(String) });
//! ```

use super::{AWAITING, run, sweep_at};
use crate::test_support::{fixture, remove};
use crate::worktree_clone_mount::{PARENT_RECORD_NAME, STORE_HEADER, StoredIdentity};
use std::path::PathBuf;

/// The refusal names the default invocation and step 8.
#[test]
fn the_refusal_names_the_default_invocation_and_its_step() {
    assert_eq!(AWAITING.mode, "sweep (no argument)");
    assert_eq!(AWAITING.step, 8);
    assert!(AWAITING.message().contains("plan step 8"), "{}", AWAITING.message());
}

/// Running the mode answers the refusal,
/// never a success line.
#[test]
fn running_the_mode_answers_the_refusal() {
    assert_eq!(run(), Err(AWAITING));
}

/// What:
///  The sweep removes the staging names a killed writer left and nothing else,
///  and still answers its step's refusal.
/// Why:
///  A machine that never adopts must not keep a killed writer's litter forever,
///  and a removers arm that also touched a record or an unknown dot entry
///  would hide the state `doctor` exists to report.
#[test]
fn the_sweep_removes_the_staging_names_a_killed_writer_left() {
    let root: PathBuf = fixture("sweep-staging-removal");
    let folder: PathBuf = root.join("slot-identities");
    std::fs::create_dir(folder.as_path()).expect("the store folder fixture");
    let parent: String = format!("{STORE_HEADER}\n{}\n", StoredIdentity::Parent(crate::worktree_clone_mount::CloneFolderIdentity { device: 1, inode: 2, owner: 1000, mode: 0o755, access_acl: None, default_acl: None }).encode());
    std::fs::write(folder.join(PARENT_RECORD_NAME), parent).expect("the parent record");
    let slot_staging: String = format!(".{}", "e".repeat(32));
    let parent_staging: String = String::from("..parent.0123456789abcdef.partial");
    for name in [slot_staging.as_str(), parent_staging.as_str(), ".a-third-form"] {
        std::fs::write(folder.join(name), b"staged").expect("a staging entry");
    }
    assert_eq!(sweep_at(folder.as_path()), Err(AWAITING), "the sweep still refuses its own step");
    let mut left: Vec<String> = std::fs::read_dir(folder.as_path()).expect("the folder lists").flatten().map(|entry| return entry.file_name().to_string_lossy().into_owned()).collect();
    left.sort();
    assert_eq!(left, vec![String::from(".a-third-form"), String::from(PARENT_RECORD_NAME)], "the two known forms are gone, the record and the unknown dot entry stay");
    remove(root.as_path());
}
