//! Controls for the mountinfo reader: line shapes, escapes, the longest-mount rule and the option lookup.

use super::*;

/// The live line of this machine's `/var/home` (from `doc/planning/cli-git-btrfs-worktree.md`), with the option the
///  doctor asks about among the super options.
const BTRFS_HOME_LINE: &str = "181 177 0:48 /home /var/home rw,relatime shared:1 - btrfs /dev/dm-1 \
rw,seclabel,compress=zstd:1,ssd,discard=async,space_cache=v2,user_subvol_rm_allowed,subvolid=257,subvol=/home";

/// A root mount without the option.
const BTRFS_ROOT_LINE: &str = "100 1 0:30 /root / rw,relatime shared:2 - btrfs /dev/dm-1 rw,seclabel,ssd,space_cache=v2,subvolid=256,subvol=/root";

/// An overlay root with an optional field absent and a mount point holding an escaped space.
const OVERLAY_LINE: &str = "55 40 0:50 / /mnt/a\\040b rw - overlay overlay rw,lowerdir=/l,upperdir=/u,userxattr";

/// A btrfs line with the option in the super options is found, and the option is not found on the line without it.
#[test]
fn the_option_is_read_from_the_super_options() {
    let table: Vec<MountFacts> = parse_mount_table(format!("{BTRFS_ROOT_LINE}\n{BTRFS_HOME_LINE}\n").as_bytes());
    assert_eq!(table.len(), 2);
    assert!(!table[0].has_option("user_subvol_rm_allowed"));
    assert!(table[1].has_option("user_subvol_rm_allowed"));
    assert_eq!(table[1].filesystem_type, "btrfs");
    assert_eq!(table[1].source, "/dev/dm-1");
    assert!(table[1].has_option("relatime"), "a per-mount option is found too");
    assert!(table[1].has_option("compress=zstd:1"));
    assert!(!table[1].has_option("compress"), "an option is matched whole, not by prefix");
}

/// The octal escape in a mount point is decoded, and an optional-field-free line parses.
#[test]
fn an_escaped_mount_point_is_decoded() {
    let table: Vec<MountFacts> = parse_mount_table(OVERLAY_LINE.as_bytes());
    assert_eq!(table.len(), 1);
    assert_eq!(table[0].mount_point, PathBuf::from("/mnt/a b"));
    assert_eq!(table[0].filesystem_type, "overlay");
}

/// A line without the separator, a short line and an empty line are skipped, and the readable lines remain.
#[test]
fn unreadable_lines_are_skipped() {
    let text: String = format!("garbage\n\n1 2 3 - btrfs\n{BTRFS_HOME_LINE}\n");
    let table: Vec<MountFacts> = parse_mount_table(text.as_bytes());
    assert_eq!(table.len(), 1);
    assert_eq!(table[0].mount_point, PathBuf::from("/var/home"));
}

/// The longest ancestor mount wins, a sibling that shares a name prefix does not count, and a stacked mount at the
///  same mount point is decided by the later line.
#[test]
fn the_longest_ancestor_mount_holds_the_path() {
    let stacked: &str = "200 100 0:60 / /var/home rw - tmpfs tmpfs rw";
    let table: Vec<MountFacts> = parse_mount_table(format!("{BTRFS_ROOT_LINE}\n{BTRFS_HOME_LINE}\n{stacked}\n").as_bytes());
    let inside: &MountFacts = mount_holding(table.as_slice(), Path::new("/var/home/user/x")).expect("a mount");
    assert_eq!(inside.filesystem_type, "tmpfs", "the later mount at the same point is the visible one");
    let sibling: &MountFacts = mount_holding(table.as_slice(), Path::new("/var/home2/x")).expect("a mount");
    assert_eq!(sibling.mount_point, PathBuf::from("/"), "/var/home2 is not under /var/home");
    let exact: &MountFacts = mount_holding(table.as_slice(), Path::new("/var/home")).expect("a mount");
    assert_eq!(exact.mount_point, PathBuf::from("/var/home"));
    assert_eq!(mount_holding(&[], Path::new("/x")), None);
}

/// The live table answers for the working directory of the test: some mount holds it, and its type is not empty.
#[test]
fn the_live_table_answers_for_an_existing_path() {
    let facts: MountFacts = read_mount_facts(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("readable").expect("a mount holds it");
    assert!(!facts.filesystem_type.is_empty());
    assert!(facts.mount_point.is_absolute());
}

/// A path that does not exist is an error that names the path, never a guess.
#[test]
fn a_missing_path_is_an_error() {
    let message: String = read_mount_facts(Path::new("/definitely/not/here")).expect_err("no such path");
    assert!(message.contains("/definitely/not/here"), "{message}");
}
