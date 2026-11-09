//! Controls for the lines a tree pass says about what it could not carry.

use super::*;
use crate::subvolume_sync_attrs::AttributeLosses;
use std::path::PathBuf;

/// A pass that found nothing to say says nothing.
#[test]
fn a_pass_that_lost_nothing_says_nothing() {
    assert_eq!(what_a_pass_could_not_carry("copy", &SyncStats::default()), Vec::<String>::new());
}

/// Each kind names its own entry,
/// or its counts and the shape it kept or left out,
/// prefixed with the pass's label.
#[test]
fn each_kind_of_what_a_pass_could_not_carry_is_said_with_its_label() {
    let stats: SyncStats = SyncStats {
        special_skipped: vec![PathBuf::from("agent.sock"), PathBuf::from("dev/null")],
        losses: AttributeLosses { owner: 2, extended_attributes: 1, mode: 0, times: 3 },
        conflicts: vec![PathBuf::from("src/newer.rs")],
        left_out: 4,
        recopied: 5,
        ..SyncStats::default()
    };
    let lines: Vec<String> = what_a_pass_could_not_carry("copy", &stats);
    assert_eq!(lines.len(), 6, "{lines:#?}");
    assert_eq!(lines[0], "copy: not recreated (a socket or device node): agent.sock");
    assert_eq!(lines[1], "copy: not recreated (a socket or device node): dev/null");
    assert_eq!(
        lines[2],
        "copy: could not set the owner of 2 entries, the permission bits of 0, the times of 3, and 1 extended attributes (not running as the owner or as root)"
    );
    assert_eq!(lines[3], "copy: kept the newer or differently typed entry src/newer.rs in the destination; it was not replaced");
    assert_eq!(lines[4], "copy: 4 entr(ies) absent from the destination were left out because they predate the catch-up");
    assert_eq!(lines[5], "copy: 5 file(s) changed while they were copied and were copied again");
}
