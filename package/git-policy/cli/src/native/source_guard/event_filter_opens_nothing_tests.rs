//! What:
//!  The ban on the event filter opening anything,
//!  taking a claim or asking the compatibility gate,
//!  and the ban on the merge device opening the ZFS device or the gate.
//! Why:
//!  A dropped event is safe only because the filter reads two strings and nothing
//!  else:
//!  the design puts it before the gate for exactly that reason,
//!  so a drop costs the fork ZED already paid and no device open
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "The event filter, in brief",
//!  and veto entry V2).
//!  The merge device is cost and never safety:
//!  it reads and writes three files under `/run`,
//!  and a device open or a gate verdict written beside them would make a cost
//!  record decide whether a request is read
//!  (V4,
//!  V7 and V12 in the same file).
//!  No module test sees either mistake:
//!  a filter that opened the device would still decide every case this crate
//!  tests,
//!  and so would a device that asked the gate.
//!  What this reading cannot see is named where it is written:
//!  a call a macro written outside the folder expands to,
//!  a name a glob import brings into scope without writing it,
//!  and a call whose name and `(` a line break separates.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(linesNamingOneOf(sourcesUnder('worktree_clone_mount_event_filter/'), DEVICE_AND_CLAIM_NAMES)).toEqual([]);
//! ```

use crate::source_scan_banned_lines_tests::{
    anchors_the_scope_no_longer_holds,
    lines_naming_one_of,
    release_sources_under,
};

/// The folder whose code must read its delivery and nothing else.
const THE_FILTER_FOLDER: &[&str] = &["worktree_clone_mount_event_filter/"];

/// The folder whose code must read and write `/run` files and open nothing.
const THE_DEVICE_FOLDER: &[&str] = &["worktree_clone_mount_sweep_merge/"];

/// The names the filter may not write:
///  the ZFS device the route opens before a raw request,
///  the compatibility gate it asks first,
///  the run claim the merge device takes,
///  and the receipts and readiness records the sweep reads.
const WHAT_THE_FILTER_MAY_NOT_NAME: &[&str] = &[
    "ZfsDevice",
    "OpenedDevice",
    "LiveGate",
    "BeforeRequests",
    "take_claim",
    "ReceiptsFolder",
    "ReadinessFile",
];

/// The names the merge device may not write:
///  the ZFS device,
///  the compatibility gate,
///  and the mounts a pass acts on.
const WHAT_THE_DEVICE_MAY_NOT_NAME: &[&str] =
    &["ZfsDevice", "OpenedDevice", "LiveGate", "BeforeRequests"];

/// What:
///  The filter folder names none of the things a decision must be made without.
/// Why:
///  The scope is the folder as a prefix,
///  so a file a later split makes inside it is covered without anyone remembering
///  it,
///  and the anchor proves the scope is still the filter's own rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(linesNamingOneOf(releaseSourcesUnder(['worktree_clone_mount_event_filter/']), names)).toEqual([]);
/// ```
#[test]
fn the_event_filter_names_nothing_it_would_have_to_open() {
    let sources: Vec<(String, String)> =
        release_sources_under(THE_FILTER_FOLDER).expect("the crate's release sources");
    let stale: Vec<String> = anchors_the_scope_no_longer_holds(
        sources.as_slice(),
        &[(
            "pub fn decide_about_the_event(",
            "the filter's rule is the code this ban is about",
        )],
    );
    assert!(
        stale.is_empty(),
        "a ban that reached nothing passes every tree, and these anchors are gone:\n{}",
        stale.join("\n")
    );
    let found: Vec<String> = lines_naming_one_of(sources.as_slice(), WHAT_THE_FILTER_MAY_NOT_NAME)
        .iter()
        .map(|held| {
            return format!(
                "{}:{}: `{}` holds `{}`",
                held.file, held.number, held.line, held.text
            );
        })
        .collect();
    assert!(
        found.is_empty(),
        "the event filter decides on two strings alone, before the compatibility gate and before `/dev/zfs` is \
         opened, so a drop costs the fork ZED already paid and nothing more; these lines name what it may not:\n{}",
        found.join("\n")
    );
}

/// What:
///  The merge device names the ZFS device and the gate nowhere.
/// Why:
///  Its whole job is how many passes run and who runs them,
///  and the pass a caller hands it is the only thing it reaches;
///  a device open written beside the device's records would turn a cost record
///  into a step that reads the pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(linesNamingOneOf(releaseSourcesUnder(['worktree_clone_mount_sweep_merge/']), names)).toEqual([]);
/// ```
#[test]
fn the_merge_device_names_neither_the_device_nor_the_gate() {
    let sources: Vec<(String, String)> =
        release_sources_under(THE_DEVICE_FOLDER).expect("the crate's release sources");
    let stale: Vec<String> = anchors_the_scope_no_longer_holds(
        sources.as_slice(),
        &[(
            "pub fn run_a_merge_pass(",
            "the device's taker is the code this ban is about",
        )],
    );
    assert!(
        stale.is_empty(),
        "a ban that reached nothing passes every tree, and these anchors are gone:\n{}",
        stale.join("\n")
    );
    let found: Vec<String> = lines_naming_one_of(sources.as_slice(), WHAT_THE_DEVICE_MAY_NOT_NAME)
        .iter()
        .map(|held| {
            return format!(
                "{}:{}: `{}` holds `{}`",
                held.file, held.number, held.line, held.text
            );
        })
        .collect();
    assert!(
        found.is_empty(),
        "the merge device decides how many passes run and who runs them, and the pass is the caller's; these \
         lines name what it may not:\n{}",
        found.join("\n")
    );
}
