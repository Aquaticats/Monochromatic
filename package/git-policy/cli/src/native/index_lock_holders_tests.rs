//! Controls for open-holder scans:
//!  the `/proc/<pid>/fd` scan against a fixture proc root,
//!  a real
//! child process holding the lock open,
//!  the `lsof -F` and Restart Manager parsers,
//!  and the
//! number reader that `BigInt` text follows.

use super::*;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

/// Device and inode of a file,
///  as the scan matches them.
fn identity_of(path: &Path) -> (u64, u64) {
    let metadata: std::fs::Metadata = std::fs::metadata(path).expect("lock metadata");
    return (metadata.dev(), metadata.ino());
}

/// Number text as `BigInt` reads it,
///  decimal,
///  prefixed and malformed.
#[test]
fn lsof_numbers_follow_bigint_spelling() {
    assert_eq!(parse_lsof_number("42"), Some(42));
    assert_eq!(parse_lsof_number(" 42 \n"), Some(42));
    assert_eq!(parse_lsof_number("0x1000011"), Some(0x1000011));
    assert_eq!(parse_lsof_number("0X1f"), Some(0x1f));
    assert_eq!(parse_lsof_number("0o17"), Some(0o17));
    assert_eq!(parse_lsof_number("0b101"), Some(5));
    assert_eq!(parse_lsof_number(""), Some(0));
    assert_eq!(parse_lsof_number("0x"), None);
    assert_eq!(parse_lsof_number("+5"), None);
    assert_eq!(parse_lsof_number("-5"), None);
    assert_eq!(parse_lsof_number("4x"), None);
    assert_eq!(parse_lsof_number("1_000"), None);
}

/// Process records start at each `p` line;
///  their command and file descriptors follow.
#[test]
fn lsof_output_groups_files_under_their_process() {
    let output: &str = "p42\ncgit commit\nf3\nD0x1000011\ni77\nn/fixture/x\nf4\nD0x2\ni9\n\
                        p43\ncsh\nf5\nD0x1000011\ni77\n";
    let processes: Vec<LsofProcess> = parse_lsof_fields(output);
    assert_eq!(processes.len(), 2);
    assert_eq!(processes[0].pid, Some(42));
    assert_eq!(processes[0].command.as_deref(), Some("git commit"));
    assert_eq!(processes[0].files, vec![(Some(0x1000011), Some(77)), (Some(2), Some(9))]);
    assert_eq!(processes[1].pid, Some(43));
    assert_eq!(processes[1].files, vec![(Some(0x1000011), Some(77))]);
}

/// A process whose PID is not a number keeps no PID,
///  and a missing field stays absent.
#[test]
fn lsof_records_keep_unreadable_fields_absent() {
    let processes: Vec<LsofProcess> = parse_lsof_fields("pabc\ncx\nf1\ni\n");
    assert_eq!(processes.len(), 1);
    assert_eq!(processes[0].pid, None);
    assert_eq!(processes[0].command.as_deref(), Some("x"));
    assert_eq!(processes[0].files, vec![(None, Some(0))]);
}

/// Output without any `p` line names no process,
///  and text before the first `p` is ignored.
#[test]
fn lsof_output_without_a_process_names_nobody() {
    assert!(parse_lsof_fields("").is_empty());
    assert!(parse_lsof_fields("cgit\nf3\n").is_empty());
    assert_eq!(parse_lsof_fields("junk\np7\n").len(), 1);
}

/// Only the processes with a file of the same device and inode match;
///  a missing PID never matches.
#[test]
fn matching_holders_compare_device_and_inode() {
    let processes: Vec<LsofProcess> = vec![
        LsofProcess {
            pid: Some(10),
            command: Some(String::from("git")),
            files: vec![(Some(5), Some(77))],
        },
        LsofProcess {
            pid: Some(11),
            command: None,
            files: vec![(Some(6), Some(77))],
        },
        LsofProcess {
            pid: None,
            command: None,
            files: vec![(Some(5), Some(77))],
        },
    ];
    let holders: Vec<Holder> = matching_holders(processes.as_slice(), 5, 77);
    assert_eq!(
        holders,
        vec![Holder {
            pid: 10,
            command: Some(String::from("git")),
        }]
    );
}

/// Restart Manager lines carry the PID,
///  a tab and the application name;
///  the name may be absent.
#[test]
fn restart_manager_lines_are_parsed() {
    let holders: Vec<Holder> = parse_restart_manager_output("4242\tgit.exe\r\n\n  7  \nbad\tname\n");
    assert_eq!(
        holders,
        vec![
            Holder {
                pid: 4242,
                command: Some(String::from("git.exe")),
            },
            Holder {
                pid: 7,
                command: None,
            },
        ]
    );
}

/// The proc scan finds the holders of the lock by device and inode,
///  reads their command names,
/// skips the scanning process itself,
///  and reports only unreadable descriptor directories.
#[test]
fn the_proc_scan_matches_descriptors_by_identity() {
    let root: PathBuf = fixture("holders-proc-scan");
    let lock: PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("lock file");
    let other: PathBuf = root.join("other");
    std::fs::write(&other, b"").expect("other file");
    let proc_root: PathBuf = root.join("proc");
    let (device, inode): (u64, u64) = identity_of(lock.as_path());

    std::fs::create_dir_all(proc_root.join("100/fd")).expect("holder fd");
    std::os::unix::fs::symlink(&lock, proc_root.join("100/fd/3")).expect("holder link");
    std::fs::write(proc_root.join("100/comm"), "git\n").expect("holder comm");
    std::fs::create_dir_all(proc_root.join("101/fd")).expect("other fd");
    std::os::unix::fs::symlink(&other, proc_root.join("101/fd/3")).expect("other link");
    std::fs::create_dir_all(proc_root.join("7/fd")).expect("own fd");
    std::os::unix::fs::symlink(&lock, proc_root.join("7/fd/3")).expect("own link");
    std::fs::create_dir_all(proc_root.join("104/fd")).expect("dangling fd");
    std::os::unix::fs::symlink(root.join("vanished"), proc_root.join("104/fd/4")).expect("dangling link");
    std::fs::create_dir_all(proc_root.join("self")).expect("self directory");
    std::fs::create_dir_all(proc_root.join("105")).expect("exited process");
    std::fs::create_dir_all(proc_root.join("103")).expect("unreadable process");
    std::fs::write(proc_root.join("103/fd"), b"not a directory").expect("unreadable fd");

    let evidence: HolderEvidence = scan_proc_fd_holders(proc_root.as_path(), device, inode, 7);
    assert_eq!(evidence.method, "proc-fd");
    assert_eq!(
        evidence.holders,
        vec![Holder {
            pid: 100,
            command: Some(String::from("git")),
        }]
    );
    assert_eq!(evidence.partial.len(), 1);
    assert!(evidence.partial[0].starts_with("PID 103: "));
    remove(root.as_path());
}

/// A proc root that cannot be listed is partial evidence with its path,
///  never a silent absence.
#[test]
fn an_unlistable_proc_root_is_partial() {
    let root: PathBuf = fixture("holders-proc-missing");
    let evidence: HolderEvidence = scan_proc_fd_holders(root.join("no-proc").as_path(), 1, 2, 7);
    assert!(evidence.holders.is_empty());
    assert_eq!(evidence.partial.len(), 1);
    assert!(evidence.partial[0].contains("no-proc"));
    remove(root.as_path());
}

/// Start a child process that holds `lock` open on descriptor 3 until it is killed.
fn start_holder(lock: &Path) -> Child {
    return Command::new("sh")
        .args(["-c", "exec 3<\"$0\"; exec sleep 30", lock.to_str().expect("UTF-8 lock path")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("start holder");
}

/// Whether a holder list names this PID.
fn holds_pid(holders: &[Holder], pid: i64) -> bool {
    for holder in holders {
        if holder.pid == pid {
            return true;
        }
    }
    return false;
}

/// Whether a descriptor path names the lock file itself.
///  The dynamic loader opens shared libraries on descriptor 3 before the shell redirects it,
///  so the descriptor existing is not proof that the child holds the lock.
#[cfg(target_os = "linux")]
fn descriptor_is_lock(proc_fd: &Path, device: u64, inode: u64) -> bool {
    use std::os::unix::fs::MetadataExt;
    return match std::fs::metadata(proc_fd) {
        Ok(found) => found.dev() == device && found.ino() == inode,
        Err(_) => false,
    };
}

/// A real child holding the lock open is found by the host scan with its command name,
///  and the
/// test's own open descriptor never counts as a holder.
#[test]
#[cfg(target_os = "linux")]
fn a_real_child_holding_the_lock_is_found_on_linux() {
    let root: PathBuf = fixture("holders-real-child");
    let lock: PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("lock file");
    let (device, inode): (u64, u64) = identity_of(lock.as_path());
    let own_open: std::fs::File = std::fs::File::open(&lock).expect("own descriptor");
    let mut child: Child = start_holder(lock.as_path());
    let child_pid: i64 = i64::from(child.id());
    let proc_fd: PathBuf = PathBuf::from(format!("/proc/{child_pid}/fd/3"));
    let started: std::time::Instant = std::time::Instant::now();
    while !descriptor_is_lock(proc_fd.as_path(), device, inode) && started.elapsed().as_secs() < 10 {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(descriptor_is_lock(proc_fd.as_path(), device, inode), "the child opened the lock on descriptor 3 within 10 s");
    let evidence: HolderEvidence = scan_platform_holders(lock.as_path(), device, inode);
    let _ = child.kill();
    let _ = child.wait();
    drop(own_open);
    assert_eq!(evidence.method, "proc-fd");
    assert!(holds_pid(evidence.holders.as_slice(), child_pid), "the child holding the lock must be a holder: {evidence:?}");
    assert!(!holds_pid(evidence.holders.as_slice(), i64::from(std::process::id())));
    remove(root.as_path());
}

/// A lock nobody holds has no holders,
///  on the host scan of this platform.
#[test]
#[cfg(target_os = "linux")]
fn an_unheld_lock_has_no_holders() {
    let root: PathBuf = fixture("holders-unheld");
    let lock: PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("lock file");
    let (device, inode): (u64, u64) = identity_of(lock.as_path());
    let evidence: HolderEvidence = scan_platform_holders(lock.as_path(), device, inode);
    assert!(evidence.holders.is_empty());
    remove(root.as_path());
}

/// Only ASCII digit names are process identifiers,
///  so the proc scan skips `self`, empty names and names with other bytes.
#[test]
#[cfg(unix)]
fn only_digit_names_are_process_identifiers() {
    assert!(is_pid_name("1"));
    assert!(is_pid_name("4242"));
    assert!(!is_pid_name(""));
    assert!(!is_pid_name("self"));
    assert!(!is_pid_name("12a"));
    assert!(!is_pid_name("1 "));
    assert!(is_digit(b'0'));
    assert!(is_digit(b'9'));
    assert!(!is_digit(b'/'));
    assert!(!is_digit(b':'));
    assert!(!is_digit(b'a'));
}
