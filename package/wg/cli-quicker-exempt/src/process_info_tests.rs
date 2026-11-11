//! Verifies procfs parsing and the account lookup.

/// Account lookup under test.
use crate::user_account::{account_from_entry, lookup_user};
/// Parsers under test.
use crate::process_info::{list_pids, parse_cgroup, parse_stat, parse_stat_fields, parse_status_id, read_own_pid_namespace, read_process, read_whole, split_cmdline, StatFields};

/// Reads the parent and start time past a command name that contains spaces and parentheses.
#[test]
fn stat_parsing_skips_a_hostile_command_name() {
    let zeros = vec!["0"; 17].join(" ");
    let text = format!("42 (we ird) name) S 7 {zeros} 123456 0\n");
    assert_eq!(parse_stat(&text), Some((7, 123_456)));
    assert_eq!(parse_stat("42 (x)"), None);
    assert_eq!(parse_stat("garbage"), None);
}

/// Builds the text of a stat file whose fields after the command name are the given state,
/// flags and start time.
fn stat_text(state: &str, flags: u64) -> String {
    let mut fields: Vec<String> = vec!["0".to_string(); 20];
    fields[0] = state.to_string();
    fields[1] = "1".to_string();
    fields[6] = flags.to_string();
    fields[19] = "99".to_string();
    return format!("5 (x) {} \n", fields.join(" "));
}

/// The state letter and the task flags are read past a hostile command name,
/// and a text too short for them is rejected.
#[test]
fn stat_fields_carry_the_state_and_the_flags() {
    let text = stat_text("R", 4_194_368).replace("5 (x)", "42 (we ird) name)");
    assert_eq!(parse_stat_fields(&text), Some(StatFields { state: 'R', parent: 1, flags: 4_194_368, start_time: 99 }));
    assert_eq!(parse_stat_fields("42 (x) S 7 0 0"), None);
}

/// A zombie,
/// a dead process and a process flagged `PF_EXITING` are on their way out,
/// and a sleeping or running process,
/// even with other flags set,
/// is not.
#[test]
fn zombies_dead_processes_and_pf_exiting_processes_are_exiting() {
    let exiting = |state: &str, flags: u64| return parse_stat_fields(&stat_text(state, flags)).expect("parses").exiting();
    assert!(exiting("Z", 0));
    assert!(exiting("X", 0));
    assert!(exiting("S", 4));
    assert!(exiting("R", 4 | 0x0040_0000));
    assert!(!exiting("S", 0));
    assert!(!exiting("R", 0x0040_0000));
    assert!(!exiting("D", 0x0000_0200));
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

/// A process that is exiting is reported as absent,
/// whether it is a zombie or is flagged `PF_EXITING` while it still looks alive,
/// and a live process of the same fixture is reported.
#[test]
fn an_exiting_process_is_reported_as_absent() {
    let (uid, _gid) = own_ids();
    let root = fake_process("exiting");
    assert!(read_process(&root, 5, uid).expect("readable").is_some());
    std::fs::write(root.join("5/stat"), stat_text("Z", 0)).expect("write a zombie");
    assert_eq!(read_process(&root, 5, uid).expect("a zombie is not an error"), None);
    std::fs::write(root.join("5/stat"), stat_text("R", 4)).expect("write an exiting process");
    assert_eq!(read_process(&root, 5, uid).expect("an exiting process is not an error"), None);
    std::fs::write(root.join("5/stat"), stat_text("S", 0)).expect("write a sleeper");
    assert!(read_process(&root, 5, uid).expect("readable").is_some());
}

/// A command line larger than the first read buffer is read whole,
/// and the status head still yields the ids when the file is longer than it.
#[test]
fn large_files_are_read_whole() {
    let (uid, _gid) = own_ids();
    let root = fake_process("large");
    let mut cmdline: Vec<u8> = b"mise\0run\0".to_vec();
    cmdline.extend(std::iter::repeat_n(b'a', 20_000));
    cmdline.push(0);
    std::fs::write(root.join("5/cmdline"), &cmdline).expect("write a long command line");
    assert_eq!(read_whole(&root.join("5/cmdline")).expect("read").len(), cmdline.len());
    let info = read_process(&root, 5, uid).expect("readable").expect("alive");
    assert_eq!(info.args.len(), 3);
    assert_eq!(info.args[2].len(), 20_000);
    let (_uid, gid) = own_ids();
    let padding = "x".repeat(4_000);
    std::fs::write(root.join("5/status"), format!("Uid:\t{uid}\t{uid}\t{uid}\t{uid}\nGid:\t{gid}\t{gid}\t{gid}\t{gid}\nPadding:\t{padding}\n")).expect("write a long status");
    assert!(read_process(&root, 5, uid).expect("readable").is_some());
}

/// Counts the read calls this process has made,
/// from the `syscr` line of its own io accounting.
fn read_calls() -> u64 {
    let text = std::fs::read_to_string("/proc/self/io").expect("read the io accounting");
    for line in text.lines() {
        if let Some(count) = line.strip_prefix("syscr: ") {
            return count.trim().parse().expect("a number");
        }
    }
    return 0;
}

/// The read calls `work` makes beyond those of taking the two counts.
fn read_calls_during(work: impl FnOnce()) -> u64 {
    let idle_before = read_calls();
    let idle_after = read_calls();
    let before = read_calls();
    work();
    let after = read_calls();
    return (after - before) - (idle_after - idle_before);
}

/// A procfs file costs two read calls whole (the content,
/// then the end of the file),
/// and a process costs seven (the head of the status file once,
/// then stat,
/// command line and cgroup twice each):
/// a pass over a thousand processes multiplies every extra call by a thousand.
/// `std::fs::read` costs about five per file on procfs,
/// which reports a size of zero,
/// and that is what this guard keeps out.
/// The files are the real procfs files of this process,
/// because a regular file reports its size and is read in two calls whatever the buffer.
#[test]
fn a_file_is_read_with_two_calls_and_a_process_with_seven() {
    let (uid, _gid) = own_ids();
    let pid = i32::try_from(std::process::id()).expect("pid fits");
    let whole = read_calls_during(|| {
        let _bytes = read_whole(std::path::Path::new("/proc/self/stat")).expect("read");
    });
    assert_eq!(whole, 2, "a whole-file read made {whole} read calls");
    let process = read_calls_during(|| {
        let _info = read_process(std::path::Path::new("/proc"), pid, uid).expect("readable");
    });
    assert_eq!(process, 7, "reading one process made {process} read calls");
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

/// A permission error on a process of the watched user is an error,
/// because a reader that may not look cannot say the process is gone.
/// Root can read everything,
/// so the test has nothing to show there.
#[test]
fn a_permission_error_is_an_error_and_not_a_vanished_process() {
    // SAFETY: geteuid has no preconditions.
    if unsafe { libc::geteuid() } == 0 {
        return;
    }
    let root = fake_process("permission");
    let (uid, _gid) = own_ids();
    let cmdline = root.join("5/cmdline");
    std::fs::set_permissions(&cmdline, std::os::unix::fs::PermissionsExt::from_mode(0o000)).expect("lock the file");
    let error = read_process(&root, 5, uid).expect_err("a locked file");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
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

/// A process that was reaped while a file it left open was read answers `ESRCH`
/// (`No such process`),
/// and that is a vanished process and not a failure of the scan:
/// the control fails on the earlier reader, which treated only `ENOENT` as gone and
/// stopped the whole socket-marking rescan with "POLICY NOT IN FORCE".
#[test]
fn a_reaped_process_answers_esrch_and_counts_as_gone() {
    let reaped = std::io::Error::from_raw_os_error(libc::ESRCH);
    assert_eq!(reaped.raw_os_error(), Some(libc::ESRCH));
    assert!(crate::process_info::means_vanished(&reaped));
    assert!(crate::process_info::means_vanished(&std::io::Error::from(std::io::ErrorKind::NotFound)));
    assert!(!crate::process_info::means_vanished(&std::io::Error::from(std::io::ErrorKind::PermissionDenied)));
    assert!(!crate::process_info::means_vanished(&std::io::Error::from(std::io::ErrorKind::InvalidData)));
}
