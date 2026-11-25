//! What:
//!  Controls for the staged state's settlement:
//!  the four answers,
//!  the words a report prints,
//!  and the mapping from step 9's cleanup outcome.
//! Why:
//!  A kept folder is the state the command refuses on and the cancelling removal reports,
//!  and a folder that holds a mount is never removed
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  its "The act" section).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(ofTheCleanup({ kind: 'kept', step: 'mount', said: 'x' }).isRetained()).toBe(true);
//! ```

use super::{StagedSettlement, of_the_cleanup};
use crate::worktree_clone_unmount::staging_cleanup::CleanupOutcome;

#[test]
fn only_a_kept_folder_is_retained() {
    assert!(!StagedSettlement::NoStagedFolder.is_retained());
    assert!(!StagedSettlement::Removed { record: true, claim: true }.is_retained());
    assert!(!StagedSettlement::ReleasedNoFolder.is_retained());
    assert!(StagedSettlement::Retained { step: String::from("mount"), said: String::from("a mount covers the folder") }.is_retained());
}

#[test]
fn the_cleanups_removal_maps_to_the_settlement_that_names_what_left() {
    match of_the_cleanup(CleanupOutcome::Removed { record: true, claim: false }) {
        StagedSettlement::Removed { record, claim } => {
            assert!(record, "the record left with the folder");
            assert!(!claim, "the claim was not there");
        }
        other => panic!("a removal maps to a removal, not {other:?}"),
    }
    assert_eq!(of_the_cleanup(CleanupOutcome::ReleasedNoFolder), StagedSettlement::ReleasedNoFolder);
}

#[test]
fn a_kept_folder_names_the_step_that_kept_it() {
    let settled: StagedSettlement = of_the_cleanup(CleanupOutcome::Kept { step: String::from("mount"), said: String::from("a row of this process's mount table names the folder") });
    let retained: bool = settled.is_retained();
    match &settled {
        StagedSettlement::Retained { step, said } => {
            assert_eq!(step, "mount");
            assert!(said.contains("mount table"), "{said}");
        }
        other => panic!("a kept folder maps to a kept settlement, not {other:?}"),
    }
    assert!(retained);
}
