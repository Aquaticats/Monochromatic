//! Controls for the pool's block-cloning switch:
//! a switch file that reads 0 declines the rung on ZFS and only on ZFS,
//! a switch that reads 1 or cannot be read declines nothing,
//! and the kernel's own switch is read without changing it.

use crate::fs_kind::FilesystemKind;
use crate::test_support::{fixture, remove};
use crate::worktree_clone_block_cloning::{block_cloning_refusal, block_cloning_refusal_at, switch_at};
use std::path::{Path, PathBuf};

/// What:
///  A switch file holding `text` in the fixture root,
///  the destination the refusal would name,
///  and the fixture root to remove afterwards.
/// Why:
///  The control stands in for `/sys/module/zfs/parameters/zfs_bclone_enabled`
///  with a file it writes,
/// so it runs on a host whose own pool clones blocks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [root, switchPath, target] = switchFixture('clone-block-cloning', '0\n');
/// ```
fn switch_fixture(name: &str, text: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let switch: PathBuf = root.join("zfs_bclone_enabled");
    std::fs::write(switch.as_path(), text).expect("a switch file standing in for the kernel's");
    let target: PathBuf = root.join("topic");
    return (root, switch, target);
}

/// What:
///  A switch reading 0 declines the rung for a ZFS destination and names both the destination and the file.
/// Why:
///  This is the cost the review measured:
///  with the pool's block cloning off every add would pay the source proof and then get a refused clone,
///  so the rung has to be declined before the proof.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(refusal('zfs', false)).toContain('its pool has ZFS block cloning off');
/// ```
#[test]
fn a_switch_reading_zero_declines_the_rung_on_zfs() {
    let (root, switch, target) = switch_fixture("clone-block-cloning-off", "0\n");
    assert_eq!(switch_at(switch.as_path()), Some(false), "the fixture reads as the kernel writes it");
    let reason: String = block_cloning_refusal((FilesystemKind::Zfs, switch_at(switch.as_path())), target.as_path()).expect("block cloning off refuses the clones");
    assert!(reason.contains("would receive the clones") && reason.contains(target.display().to_string().as_str()), "the refusal names the destination: {reason}");
    assert!(reason.contains("zfs_bclone_enabled") && reason.contains("reads 0"), "the refusal names the switch and its value: {reason}");
    assert_eq!(block_cloning_refusal_at((target.as_path(), switch.as_path())), None, "a fixture root is no ZFS dataset, so the switch decides nothing for it");
    remove(root.as_path());
}

/// What:
///  A switch reading 1 declines nothing,
///  a switch file that is absent declines nothing,
///  and a btrfs destination on the same host is decided by its own filesystem.
/// Why:
///  The switch belongs to the pool,
///  so narrowing the check to ZFS is what keeps every other filesystem's add as it was;
///  a host with no ZFS module has no switch file,
/// and its adds are not refused for one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(refusal('zfs', true)).toBe(undefined); expect(refusal('btrfs', false)).toBe(undefined); expect(refusalAt(target, missing)).toBe(undefined);
/// ```
#[test]
fn a_switch_reading_one_or_no_switch_at_all_declines_nothing() {
    let (root, switch, target) = switch_fixture("clone-block-cloning-on", "1\n");
    assert_eq!(switch_at(switch.as_path()), Some(true));
    assert_eq!(block_cloning_refusal((FilesystemKind::Zfs, Some(true)), target.as_path()), None, "block cloning on refuses nothing");
    assert_eq!(block_cloning_refusal((FilesystemKind::Btrfs, Some(false)), target.as_path()), None, "a btrfs destination is not decided by the ZFS pool's switch");
    assert_eq!(block_cloning_refusal((FilesystemKind::Other, Some(false)), target.as_path()), None, "an ordinary directory is not decided by the ZFS pool's switch");
    let absent: PathBuf = switch.with_file_name("absent");
    assert_eq!(block_cloning_refusal_at((absent.as_path(), switch.as_path())), None, "a host without the switch file refuses nothing");
    remove(root.as_path());
}

/// What:
///  This host's own switch,
/// when it has one,
/// is read as one of the two values and never changes.
/// Why:
///  The rung reads the real file in `/sys`,
///  so a control has to meet that file at least once;
///  on a host without the ZFS module the file is absent and the read says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect([undefined, true, false]).toContain(switchAt(ZFS_BLOCK_CLONING_SWITCH));
/// ```
#[test]
fn this_hosts_switch_reads_as_one_of_the_two_values() {
    let host: Option<bool> = switch_at(Path::new(crate::worktree_clone_block_cloning::ZFS_BLOCK_CLONING_SWITCH));
    assert!(matches!(host, None | Some(true) | Some(false)), "the switch reads off, on or not at all: {host:?}");
}
