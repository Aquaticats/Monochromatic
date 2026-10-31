//! Verifies procfs parsing and the account lookup.

/// Account lookup under test.
use crate::user_account::lookup_user;
/// Parsers under test.
use crate::process_info::{list_pids, parse_cgroup, parse_status_id, parse_stat, read_process, split_cmdline};

/// Reads the parent and start time past a command name that contains spaces and parentheses.
#[test]
fn stat_parsing_skips_a_hostile_command_name() {
    let zeros = vec!["0"; 17].join(" ");
    let text = format!("42 (we ird) name) S 7 {zeros} 123456 0\n");
    assert_eq!(parse_stat(&text), Some((7, 123_456)));
    assert_eq!(parse_stat("42 (x)"), None);
    assert_eq!(parse_stat("garbage"), None);
}

/// Reads the real id and ignores the effective ids.
#[test]
fn status_ids_use_the_real_id() {
    let status = "Name:\tx\nUid:\t1000\t0\t0\t0\nGid:\t1001\t0\t0\t0\n";
    assert_eq!(parse_status_id(status, "Uid:"), Some(1000));
    assert_eq!(parse_status_id(status, "Gid:"), Some(1001));
    assert_eq!(parse_status_id("Name:\tx\n", "Uid:"), None);
}

/// Reads the unified hierarchy line only.
#[test]
fn only_the_unified_cgroup_line_counts() {
    assert_eq!(parse_cgroup("1:name=systemd:/a\n0::/b/c.scope\n"), Some("/b/c.scope".to_string()));
    assert_eq!(parse_cgroup("1:name=systemd:/a\n"), None);
}

/// Splits a command line on NUL and drops the trailing empty argument.
#[test]
fn command_lines_split_on_nul() {
    assert_eq!(split_cmdline(b"mise\0run\0x\0"), vec![b"mise".to_vec(), b"run".to_vec(), b"x".to_vec()]);
    assert_eq!(split_cmdline(b""), Vec::<Vec<u8>>::new());
}

/// Reads this very test process through the real procfs,
/// and reports a vanished pid as absent.
#[test]
fn the_real_procfs_describes_this_process() {
    let pid = i32::try_from(std::process::id()).expect("pid fits");
    let info = read_process(std::path::Path::new("/proc"), pid).expect("readable").expect("alive");
    assert_eq!(info.pid, pid);
    assert!(info.exe.is_absolute());
    assert!(!info.args.is_empty());
    assert!(info.cgroup.starts_with('/'));
    assert!(list_pids(std::path::Path::new("/proc")).expect("list").contains(&pid));
    assert_eq!(read_process(std::path::Path::new("/proc"), i32::MAX).expect("no error"), None);
}

/// Looks up this process's own account and an id that has none.
#[test]
fn the_account_lookup_finds_this_user_and_rejects_an_unknown_id() {
    // SAFETY: getuid and getgid have no preconditions.
    let (uid, gid) = unsafe { (libc::getuid(), libc::getgid()) };
    let account = lookup_user(uid).expect("this user has an account");
    assert!(account.home.is_absolute());
    assert_eq!(account.gid, gid);
    assert!(lookup_user(4_000_000_000).is_err());
}
