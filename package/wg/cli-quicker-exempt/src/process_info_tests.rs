//! Verifies procfs parsing and the account lookup.

/// Account lookup under test.
use crate::user_account::{account_from_entry, lookup_user};
/// Parsers under test.
use crate::process_info::{list_pids, parse_cgroup, parse_status_id, parse_stat, read_own_pid_namespace, read_process, split_cmdline};

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

/// This process's own ids,
/// so fixtures stay valid for whoever runs the test.
fn own_ids() -> (u32, u32) {
    // SAFETY: getuid and getgid have no preconditions.
    return unsafe { (libc::getuid(), libc::getgid()) };
}

/// Reads this very test process through the real procfs,
/// reports a vanished pid as absent,
/// and reports a process of another user as absent.
#[test]
fn the_real_procfs_describes_this_process() {
    let pid = i32::try_from(std::process::id()).expect("pid fits");
    let (uid, _gid) = own_ids();
    let info = read_process(std::path::Path::new("/proc"), pid, uid).expect("readable").expect("alive");
    assert_eq!(info.pid, pid);
    assert!(info.exe.is_absolute());
    assert!(!info.args.is_empty());
    assert!(info.cgroup.starts_with('/'));
    assert!(info.pid_namespace.as_deref().is_some_and(|link| return link.starts_with("pid:[")));
    assert_eq!(read_own_pid_namespace(std::path::Path::new("/proc")), info.pid_namespace);
    assert!(list_pids(std::path::Path::new("/proc")).expect("list").contains(&pid));
    assert_eq!(read_process(std::path::Path::new("/proc"), i32::MAX, uid).expect("no error"), None);
    assert_eq!(read_process(std::path::Path::new("/proc"), pid, uid.wrapping_add(1)).expect("no error"), None);
}

/// Builds a fake process directory with readable files.
fn fake_process(name: &str) -> std::path::PathBuf {
    let (uid, gid) = own_ids();
    let root = crate::test_scratch::scratch_root().join(format!("wg-quicker-info-{name}-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&root);
    let base = root.join("5");
    std::fs::create_dir_all(&base).expect("make the process directory");
    std::os::unix::fs::symlink("/usr/bin/mise", base.join("exe")).expect("link the executable");
    std::fs::write(base.join("cmdline"), b"mise\0run\0").expect("write cmdline");
    std::fs::write(base.join("status"), format!("Uid:\t{uid}\t{uid}\t{uid}\t{uid}\nGid:\t{gid}\t{gid}\t{gid}\t{gid}\n")).expect("write status");
    std::fs::write(base.join("cgroup"), "0::/user.slice/x.scope\n").expect("write cgroup");
    let zeros = vec!["0"; 17].join(" ");
    std::fs::write(base.join("stat"), format!("5 (x) S 1 {zeros} 99 0\n")).expect("write stat");
    return root;
}

/// A readable fixture parses and a missing pid namespace link leaves the field absent.
#[test]
fn a_fixture_without_a_namespace_link_parses_with_the_field_absent() {
    let root = fake_process("plain");
    let (uid, _gid) = own_ids();
    let info = read_process(&root, 5, uid).expect("readable").expect("alive");
    assert_eq!(info.pid_namespace, None);
    assert_eq!(info.start_time, 99);
    assert_eq!(info.parent, 1);
}

/// A file the reader cannot open is an error that names the problem,
/// never the same answer as a process that exited.
#[test]
fn an_unreadable_file_is_an_error_and_not_a_vanished_process() {
    let root = fake_process("unreadable");
    let (uid, _gid) = own_ids();
    // A directory in place of the file makes the read fail with an error that is not a vanish.
    let cmdline = root.join("5/cmdline");
    std::fs::remove_file(&cmdline).expect("remove the file");
    std::fs::create_dir(&cmdline).expect("replace it by a directory");
    assert!(read_process(&root, 5, uid).is_err());
    // An executable link that cannot be read is an error too: a plain file is not a link.
    let fresh = fake_process("exe-error");
    std::fs::remove_file(fresh.join("5/exe")).expect("remove the link");
    std::fs::write(fresh.join("5/exe"), b"x").expect("write a plain file");
    assert!(read_process(&fresh, 5, uid).is_err());
    // A missing file is a vanished process.
    std::fs::remove_file(fresh.join("5/exe")).expect("remove the file");
    assert_eq!(read_process(&fresh, 5, uid).expect("a vanish is not an error"), None);
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

/// An entry whose home directory field is null is an error,
/// and an entry with a home directory yields the account.
#[test]
fn an_entry_without_a_home_directory_field_is_an_error() {
    // SAFETY: a zeroed passwd has only null pointers and zero ids.
    let mut entry: libc::passwd = unsafe { std::mem::zeroed() };
    let error = account_from_entry(&entry, 7).expect_err("a null home directory field");
    assert!(error.to_string().contains("no home directory"));
    let home = std::ffi::CString::new("/home/tester").expect("no NUL inside");
    entry.pw_dir = home.as_ptr().cast_mut();
    entry.pw_gid = 42;
    let account = account_from_entry(&entry, 7).expect("a filled entry");
    assert_eq!(account.gid, 42);
    assert_eq!(account.home, std::path::PathBuf::from("/home/tester"));
}
