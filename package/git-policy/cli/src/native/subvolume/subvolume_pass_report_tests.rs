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
        left_out: vec![PathBuf::from("gone.txt"), PathBuf::from("gone2.txt"), PathBuf::from("gone3.txt"), PathBuf::from("gone4.txt")],
        holder_records_left_out: vec![PathBuf::from(".git/cli-git-lock-holders/4242.0f3a")],
        notices_carried: vec![PathBuf::from(".git/monochromatic-git-policy-cli/notices/push+main+0f3a.txt")],
        kept_at_the_destination: vec![PathBuf::from(".git/person-notes.txt")],
        changed_while_passing: vec![PathBuf::from("src/changed-kind.rs")],
        recopied: 5,
        ..SyncStats::default()
    };
    let lines: Vec<String> = what_a_pass_could_not_carry("copy", &stats);
    assert_eq!(lines.len(), 10, "{lines:#?}");
    assert_eq!(lines[0], "copy: not recreated (a socket or device node): agent.sock");
    assert_eq!(lines[1], "copy: not recreated (a socket or device node): dev/null");
    assert_eq!(
        lines[2],
        "copy: could not set the owner of 2 entries, the permission bits of 0, the times of 3, and 1 extended attributes (not running as the owner or as root)"
    );
    assert_eq!(lines[3], "copy: kept the newer or differently typed entry src/newer.rs in the destination; it was not replaced");
    assert_eq!(lines[4], "copy: 4 entr(ies) absent from the destination were left out because they predate the catch-up");
    assert_eq!(
        lines[5],
        "copy: 1 record(s) of this program's own lock holders stayed in the source, because the destination lacks them and they are older than the cut-off: .git/cli-git-lock-holders/4242.0f3a"
    );
    assert_eq!(
        lines[6],
        "copy: did not place .git/person-notes.txt because the destination came to hold another entry there, and that entry stays"
    );
    assert_eq!(lines[7], "copy: did not carry src/changed-kind.rs because it was another kind of entry when the copy reached it");
    assert_eq!(
        lines[8],
        "copy: carried 1 of this program's own notice(s) although the destination lacks them and they are older than the cut-off: .git/monochromatic-git-policy-cli/notices/push+main+0f3a.txt"
    );
    assert_eq!(lines[9], "copy: 5 file(s) changed while they were copied and were copied again");
}
