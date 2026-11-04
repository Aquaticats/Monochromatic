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

/// The mountinfo ids tell a hidden mount from a visible one: a mount made at `/repo/work` and then covered by a mount at
///  `/repo` does not hold `/repo/work`, but a mount made at `/repo/work` after the cover does.
#[test]
fn a_mount_hidden_under_a_later_mount_does_not_hold_the_path() {
    let root: &str = "10 1 0:1 / / rw - btrfs /dev/a rw";
    let hidden: &str = "20 10 0:2 / /repo/work rw - btrfs /dev/b rw,user_subvol_rm_allowed";
    let cover: &str = "30 10 0:3 / /repo rw - btrfs /dev/c rw";
    let table: Vec<MountFacts> = parse_mount_table(format!("{root}\n{hidden}\n{cover}\n").as_bytes());
    let holder: &MountFacts = mount_holding(table.as_slice(), Path::new("/repo/work/file")).expect("a mount");
    assert_eq!(holder.source, "/dev/c", "the covered mount holds nothing");
    assert!(!holder.has_option("user_subvol_rm_allowed"));
    let later: &str = "40 30 0:4 / /repo/work rw - btrfs /dev/d rw,user_subvol_rm_allowed";
    let with_later: Vec<MountFacts> = parse_mount_table(format!("{root}\n{hidden}\n{cover}\n{later}\n").as_bytes());
    assert_eq!(mount_holding(with_later.as_slice(), Path::new("/repo/work")).expect("a mount").source, "/dev/d");
    assert_eq!(mount_holding(with_later.as_slice(), Path::new("/repo/other")).expect("a mount").source, "/dev/c");
}

/// A mount stacked over another at one mount point, and a table whose parent ids do not link, are both resolved: the
///  topmost stacked mount holds the path, and an unlinked table falls back to the longest mount point.
#[test]
fn stacked_mounts_resolve_to_the_top_and_an_unlinked_table_to_the_longest_mount_point() {
    let table: Vec<MountFacts> = parse_mount_table(
        "10 1 0:1 / / rw - btrfs /dev/a rw\n30 10 0:3 / /repo rw - btrfs /dev/c rw\n35 30 0:5 / /repo rw - tmpfs tmpfs rw\n".as_bytes(),
    );
    assert_eq!(mount_holding(table.as_slice(), Path::new("/repo/x")).expect("a mount").filesystem_type, "tmpfs");
    let unlinked: Vec<MountFacts> = parse_mount_table("100 1 0:1 / / rw - btrfs /dev/a rw\n200 999 0:2 / /data rw - xfs /dev/b rw\n".as_bytes());
    assert_eq!(mount_holding(unlinked.as_slice(), Path::new("/data/x")).expect("a mount").filesystem_type, "xfs");
    let no_root: Vec<MountFacts> = parse_mount_table("200 999 0:2 / /data rw - xfs /dev/b rw\n".as_bytes());
    assert_eq!(mount_holding(no_root.as_slice(), Path::new("/data/x")).expect("a mount").filesystem_type, "xfs");
    assert_eq!(mount_holding(no_root.as_slice(), Path::new("/elsewhere")), None);
}

/// A line whose ids are not numbers is not a mount line.
#[test]
fn a_line_with_non_numeric_ids_is_skipped() {
    assert!(parse_mount_table("x y 0:1 / / rw - btrfs /dev/a rw\n".as_bytes()).is_empty());
}

/// A line is skipped when the separator comes too early or when fewer than the three fields follow it, whichever of the two
///  holds; a line with exactly the three fields after the separator is read. Each short shape is its own case, because the
///  two checks must each refuse what the other would let through.
#[test]
fn a_line_is_read_only_with_the_separator_late_enough_and_three_fields_after_it() {
    let complete: &str = "10 1 0:1 / / rw - btrfs /dev/a rw";
    assert_eq!(parse_mount_table(complete.as_bytes()).len(), 1, "positive control");
    let separator_too_early: &str = "1 2 3 - btrfs /dev/a rw";
    let cut_after_the_separator: &str = "10 1 0:1 / / rw - btrfs /dev/a";
    let nothing_after_the_separator: &str = "10 1 0:1 / / rw -";
    for line in [separator_too_early, cut_after_the_separator, nothing_after_the_separator] {
        assert!(parse_mount_table(line.as_bytes()).is_empty(), "{line}");
    }
}

/// When the ids do not chain from any mount on the path (a mount whose parent is listed but sits elsewhere), the mount with
///  the longest mount point above the path holds it, and the later of two at one mount point wins.
#[test]
fn a_path_no_chain_of_ids_reaches_falls_back_to_the_longest_mount_point() {
    let elsewhere: &str = "10 1 0:1 / /a rw - btrfs /dev/a rw";
    let short: &str = "20 10 0:2 / /b rw - xfs /dev/b rw";
    let long: &str = "30 20 0:3 / /b/c rw - ext4 /dev/c rw";
    let later_short: &str = "40 10 0:4 / /b rw - tmpfs tmpfs rw";
    let table: Vec<MountFacts> = parse_mount_table(format!("{elsewhere}\n{short}\n{long}\n{later_short}\n").as_bytes());
    assert_eq!(longest_mount_holding(table.as_slice(), Path::new("/b/c/x")).expect("a mount").filesystem_type, "ext4", "the longer mount point wins");
    assert_eq!(longest_mount_holding(table.as_slice(), Path::new("/b/other")).expect("a mount").filesystem_type, "tmpfs", "the later of two at one point wins");
    assert_eq!(longest_mount_holding(table.as_slice(), Path::new("/nowhere")), None);
    assert_eq!(mount_holding(table.as_slice(), Path::new("/b/c/x")).expect("the fallback answers").filesystem_type, "ext4", "no walk reaches it");
}
