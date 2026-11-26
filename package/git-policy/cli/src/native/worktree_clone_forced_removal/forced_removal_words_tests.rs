//! Controls for the forced arm's words:
//!  every refusal reads back as itself,
//!  the receipt's own bound holds each word,
//!  the stages advance in the act's order,
//!  and the reserved names are the ones the design fixes.

use crate::worktree_clone_forced_removal::forced_removal_words::*;

/// Every refusal word reads back as the same refusal,
/// and a word this grammar does not carry parses to nothing.
#[test]
fn every_refusal_reads_back_as_itself() {
    let words: [RefusalWord; 7] = [
        RefusalWord::CallerProtectedHolds,
        RefusalWord::HolderUnendable,
        RefusalWord::HolderUnseen,
        RefusalWord::HolderBlind,
        RefusalWord::HolderLocked,
        RefusalWord::HolderStructural,
        RefusalWord::CallerGone,
    ];
    for word in words {
        assert_eq!(
            RefusalWord::parse(word.word()),
            Some(word),
            "word `{}`",
            word.word()
        );
    }
    assert_eq!(RefusalWord::parse("holder-busy"), None);
}

/// A refusal word fits the receipt's own failed-step bound,
/// which is what lets it travel in that field.
#[test]
fn every_refusal_fits_the_receipts_failed_step_bound() {
    let words: [RefusalWord; 7] = [
        RefusalWord::CallerProtectedHolds,
        RefusalWord::HolderUnendable,
        RefusalWord::HolderUnseen,
        RefusalWord::HolderBlind,
        RefusalWord::HolderLocked,
        RefusalWord::HolderStructural,
        RefusalWord::CallerGone,
    ];
    for word in words {
        assert!(
            word.word().len() <= FAILED_STEP_MAXIMUM_BYTES,
            "`{}` is {} bytes and the receipt's field holds {FAILED_STEP_MAXIMUM_BYTES}",
            word.word(),
            word.word().len()
        );
    }
    // The bound itself is the receipt's, and the longest word is well inside it.
    assert!(FAILED_STEP_MAXIMUM_BYTES >= "caller-protected-holds".len());
}

/// The stages rise in the act's own order,
/// so a progress file that went backwards is read as a rewritten file.
#[test]
fn the_stages_rise_in_the_acts_order() {
    let stages: [Stage; 5] = [
        Stage::Preflight,
        Stage::Structural,
        Stage::Signals,
        Stage::Unmount,
        Stage::Done,
    ];
    let orders: Vec<u8> = stages.iter().map(|stage| return stage.order()).collect();
    assert_eq!(orders, vec![0, 1, 2, 3, 4]);
    assert_eq!(Stage::Preflight.word(), "preflight");
    assert_eq!(Stage::Done.word(), "done");
}

/// The names the design fixes are the bytes this build writes,
/// and the two properties carry the route's own prefix.
#[test]
fn the_reserved_names_are_the_designs() {
    assert_eq!(SCAN_FILE_SUFFIX, ".holder-scan");
    assert_eq!(PROGRESS_FILE_SUFFIX, ".removal-progress");
    assert_eq!(DESTINATION_DIRECTORY_PREFIX, ".git-worktree-clone-removal-");
    assert_eq!(
        CALLER_PROPERTY,
        "org.monochromatic:git-worktree-clone-removal-caller"
    );
    assert!(FORCE_PROPERTY.starts_with("org.monochromatic:git-worktree-clone-removal-"));
    // The version word and the separator are the mount protocol's own, so one grammar spells them once.
    assert_eq!(
        crate::worktree_clone_mount_protocol::clone_mount_property_names::VALUE_VERSION,
        "v1"
    );
    assert_eq!(
        crate::worktree_clone_mount_protocol::clone_mount_property_names::FIELD_SEPARATOR,
        ':'
    );
}
