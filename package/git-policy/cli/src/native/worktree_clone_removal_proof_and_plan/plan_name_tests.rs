//! What:
//!  The controls of the three names a plan's grammar rests on:
//!  the file's GUID name,
//!  a dataset's name and a snapshot's.
//! Why:
//!  A name that carried a newline or an `=` would forge a line,
//!  and a file named by anything but the clone's canonical GUID could not be found by the removal
//!  that needs it
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The record's name grammar" and `F2`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(isPlanName('007')).toBe(false);
//! ```

use super::{is_a_snapshot_of, is_canonical_plan_name, is_dataset_name, is_snapshot_name};

#[test]
fn a_plan_name_is_the_clones_canonical_guid_and_nothing_else() {
    assert!(is_canonical_plan_name("7828944690444299989"));
    assert!(is_canonical_plan_name("0"));
    assert!(is_canonical_plan_name("78289446904442999890"), "a twenty-digit GUID is canonical");
    for refused in ["", "007", "+1", "7a", ".7828944690444299989"] {
        assert!(!is_canonical_plan_name(refused), "`{refused}` is not a plan name");
    }
    let twenty_one_digits: String = format!("1{}", "0".repeat(20));
    assert!(!is_canonical_plan_name(twenty_one_digits.as_str()), "21 digits are past the bound");
}

#[test]
fn a_dataset_name_holds_no_newline_no_equals_and_no_empty_component() {
    assert!(is_dataset_name("tank"));
    assert!(is_dataset_name("tank/wt/fix-flaky-test"));
    assert!(is_dataset_name("tank/mono checkout"));
    for refused in ["", "tank/", "/tank", "tank//wt", "tank/../wt", "tank/.", "tank/wt=x", "tank/wt\nx", "tank/wt@origin"] {
        assert!(!is_dataset_name(refused), "`{refused}` is not a dataset name");
    }
    let too_long: String = "a".repeat(256);
    assert!(!is_dataset_name(too_long.as_str()), "a name longer than the kernel's bound is refused");
}

#[test]
fn a_snapshot_name_is_one_dataset_and_one_snapshot() {
    assert!(is_snapshot_name("tank/wt/fix-flaky-test@worktree-own"));
    assert!(is_snapshot_name("tank/mono-checkout@origin"));
    for refused in ["tank/wt", "@origin", "tank/wt@", "tank/wt@a@b", "tank//wt@a", "tank/wt@a=b", "tank/wt@a\nb"] {
        assert!(!is_snapshot_name(refused), "`{refused}` is not a snapshot name");
    }
}

#[test]
fn a_snapshot_belongs_to_the_dataset_its_prefix_names() {
    assert!(is_a_snapshot_of("tank/wt", "tank/wt@own"));
    assert!(!is_a_snapshot_of("tank/wt", "tank/other@own"));
    assert!(!is_a_snapshot_of("tank/wt", "tank/wt"));
}
