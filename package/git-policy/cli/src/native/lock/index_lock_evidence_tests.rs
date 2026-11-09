//! Controls for foreign `index.lock` evidence:
//!  PID file text and what the process table says,
//!  lock metadata
//! without following links,
//!  classification precedence,
//!  the gather with injected sources,
//!  and the
//! prose the wrapper prints.
//!  Two of them pin the verdict against the wall clock (audit of 2026-11-09,
//! finding 3).

use super::*;
use crate::index_lock_holders::Holder;
use crate::test_support::{fixture, remove};
use crate::wait_ceilings::MERGE_LOCK_CEILING;
use std::path::PathBuf;

/// A lock whose change time is one second after the epoch,
///  with a 20 ms start resolution window.
fn lock_changed_at_1000() -> LockMetadata {
    return LockMetadata {
        device: 3,
        inode: 9,
        ctime_ms: 1000,
    };
}

/// The process table's answer for a PID that runs,
/// started at `started_at_ms`,
/// and named `command`.
fn process_of(started_at_ms: f64, resolution_ms: i64, command: Option<&str>) -> PidProcess {
    return PidProcess {
        start: ProcessStart::Running {
            started_at_ms,
            resolution_ms,
        },
        command: command.map(String::from),
    };
}

/// A process table reporting no process.
fn start_missing(_pid: i64) -> PidProcess {
    return PidProcess {
        start: ProcessStart::Missing,
        command: None,
    };
}

/// A Git that this host's clock puts before the lock's change time.
fn start_before(_pid: i64) -> PidProcess {
    return process_of(500.0, 10, Some("git"));
}

/// A Git that this host's clock puts after the lock's change time,
/// which a forward step of the wall clock produces for a live holder.
fn start_after(_pid: i64) -> PidProcess {
    return process_of(5000.0, 10, Some("git"));
}

/// A process whose start time cannot be read.
fn start_unknown(_pid: i64) -> PidProcess {
    return PidProcess {
        start: ProcessStart::Unknown(String::from("probe failed")),
        command: None,
    };
}

/// A scan that saw no process with the lock open and could see everything.
fn no_holders(_path: &Path, _device: u64, _inode: u64) -> HolderEvidence {
    return HolderEvidence {
        method: "proc-fd",
        holders: Vec::new(),
        partial: Vec::new(),
        unresolvable: Vec::new(),
    };
}

/// A holder scan that finds one process,
///  PID 55 named `git`.
fn scan_one(_path: &Path, _device: u64, _inode: u64) -> HolderEvidence {
    return HolderEvidence {
        method: "proc-fd",
        holders: vec![Holder {
            pid: 55,
            command: Some(String::from("git")),
        }],
        partial: Vec::new(),
        unresolvable: Vec::new(),
    };
}

/// A holder scan that must never run;
///  reaching it fails the test.
fn scan_never(_path: &Path, _device: u64, _inode: u64) -> HolderEvidence {
    panic!("the holder scan must not run once the PID file proves a live owner");
}

/// Evidence with the given PID file,
///  holders and the fixed lock.
fn evidence_with(pid_file: PidFile, holders: Option<HolderEvidence>) -> Evidence {
    return Evidence {
        lock_path: PathBuf::from("/repo/index.lock"),
        lock: lock_changed_at_1000(),
        pid_file,
        holders,
    };
}

/// Decimal PID text:
///  a trailing newline is allowed,
///  anything else around the digits is not.
#[test]
fn pid_file_text_names_a_positive_safe_integer() {
    assert_eq!(parse_pid_file_text("pid 42\n"), Some(42));
    assert_eq!(parse_pid_file_text("pid 42"), Some(42));
    assert_eq!(parse_pid_file_text("pid 42\n\n"), Some(42));
    assert_eq!(parse_pid_file_text("pid 9007199254740991"), Some(9_007_199_254_740_991));
    assert_eq!(parse_pid_file_text("pid 0"), None);
    assert_eq!(parse_pid_file_text("pid -1"), None);
    assert_eq!(parse_pid_file_text("pid 4x"), None);
    assert_eq!(parse_pid_file_text("pid 9007199254740992"), None);
    assert_eq!(parse_pid_file_text("42\n"), None);
    assert_eq!(parse_pid_file_text("PID 42\n"), None);
    assert_eq!(parse_pid_file_text("pid \n"), None);
    assert_eq!(parse_pid_file_text(""), None);
}

/// The start the evidence words is compared with the change time,
///  plus the start's resolution,
///  and it decides nothing:
///  it only says what this host's clock reads.
#[test]
fn the_worded_start_compares_with_the_change_time() {
    let lock: LockMetadata = lock_changed_at_1000();
    assert!(!process_of(500.0, 1, Some("git")).starts_after(&lock));
    assert!(!process_of(1020.0, 20, Some("git")).starts_after(&lock));
    assert!(process_of(1021.0, 20, Some("git")).starts_after(&lock));
    assert!(!start_missing(42).starts_after(&lock), "a gone process has no start to compare");
    assert!(!start_unknown(42).starts_after(&lock));
}

/// An absent PID file is absent evidence,
///  and a directory in its place is unreadable.
#[test]
fn the_pid_file_is_absent_or_unreadable() {
    let root: PathBuf = fixture("evidence-pid-file-kinds");
    let absent: PathBuf = root.join("index~pid.lock");
    assert_eq!(read_pid_file_evidence(absent.as_path(), start_missing), PidFile::Absent);
    std::fs::create_dir(&absent).expect("directory in place of the PID file");
    assert!(matches!(
        read_pid_file_evidence(absent.as_path(), start_missing),
        PidFile::Unreadable(_)
    ));
    remove(root.as_path());
}

/// A PID file naming a process carries what the process table says about that number.
#[test]
fn the_pid_file_carries_what_the_process_table_says() {
    let root: PathBuf = fixture("evidence-pid-file-owner");
    let path: PathBuf = root.join("index~pid.lock");
    std::fs::write(&path, "pid 42\n").expect("PID file");
    assert_eq!(read_pid_file_evidence(path.as_path(), start_missing), PidFile::Owner(42, start_missing(42)));
    assert_eq!(read_pid_file_evidence(path.as_path(), start_before), PidFile::Owner(42, start_before(42)));
    assert_eq!(read_pid_file_evidence(path.as_path(), start_after), PidFile::Owner(42, start_after(42)));
    assert_eq!(read_pid_file_evidence(path.as_path(), start_unknown), PidFile::Owner(42, start_unknown(42)));
    remove(root.as_path());
}

/// Text that is not `pid <n>` is malformed,
///  and its quoted prefix is at most 64 UTF-16 units.
#[test]
fn malformed_pid_files_quote_at_most_sixty_four_units() {
    let root: PathBuf = fixture("evidence-pid-file-malformed");
    let path: PathBuf = root.join("index~pid.lock");
    std::fs::write(&path, "x".repeat(100)).expect("long text");
    assert_eq!(
        read_pid_file_evidence(path.as_path(), start_missing),
        PidFile::Malformed("x".repeat(64))
    );
    std::fs::write(&path, "\u{1F600}".repeat(40)).expect("emoji text");
    assert_eq!(
        read_pid_file_evidence(path.as_path(), start_missing),
        PidFile::Malformed("\u{1F600}".repeat(32))
    );
    std::fs::write(&path, "short").expect("short text");
    assert_eq!(
        read_pid_file_evidence(path.as_path(), start_missing),
        PidFile::Malformed(String::from("short"))
    );
    remove(root.as_path());
}

/// A lock's metadata is its own link-free identity;
///  a symbolic link is judged by the link.
#[test]
fn lock_metadata_does_not_follow_links() {
    let root: PathBuf = fixture("evidence-lock-metadata");
    let lock: PathBuf = root.join("index.lock");
    let target: PathBuf = root.join("target");
    std::fs::write(&target, b"").expect("target");
    std::os::unix::fs::symlink(&target, &lock).expect("link as lock");
    let metadata: LockMetadata = read_lock_metadata(lock.as_path())
        .expect("readable link")
        .expect("present link");
    use std::os::unix::fs::MetadataExt;
    let link_inode: u64 = std::fs::symlink_metadata(&lock).expect("link metadata").ino();
    let target_inode: u64 = std::fs::metadata(&target).expect("target metadata").ino();
    assert_eq!(metadata.inode, link_inode);
    assert_ne!(metadata.inode, target_inode);
    assert!(metadata.ctime_ms > 0);
    remove(root.as_path());
}

/// A missing lock,
///  or a path under a regular file,
///  is no lock;
///  other failures are errors.
#[test]
fn an_absent_lock_is_not_an_error() {
    let root: PathBuf = fixture("evidence-lock-absent");
    assert_eq!(read_lock_metadata(root.join("index.lock").as_path()).expect("absent is Ok"), None);
    std::fs::write(root.join("file"), b"").expect("regular file");
    assert_eq!(
        read_lock_metadata(root.join("file").join("index.lock").as_path()).expect("ENOTDIR is Ok"),
        None
    );
    remove(root.as_path());
}

/// A proven live owner by descriptor beats every PID file;
///  then a PID file naming a running Git proves a live owner;
///  and a number this table does not have proves nothing.
#[test]
fn classification_prefers_a_descriptor_then_the_pid_file() {
    let holder: HolderEvidence = HolderEvidence {
        method: "proc-fd",
        holders: vec![Holder {
            pid: 55,
            command: Some(String::from("git")),
        }],
        partial: Vec::new(),
        unresolvable: Vec::new(),
    };
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, start_missing(42)), Some(holder))),
        Verdict::ProvenAlive {
            pid: 55,
            by_descriptor: true,
            command: Some(String::from("git")),
        }
    );
    let empty: HolderEvidence = HolderEvidence {
        method: "proc-fd",
        holders: Vec::new(),
        partial: Vec::new(),
        unresolvable: Vec::new(),
    };
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, start_before(42)), Some(empty.clone()))),
        Verdict::ProvenAlive {
            pid: 42,
            by_descriptor: false,
            command: Some(String::from("git")),
        }
    );
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, start_missing(42)), Some(empty.clone()))),
        Verdict::EvidenceFree,
        "a number this table does not have proves nothing: its writer may be in another PID namespace"
    );
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Absent, Some(empty))),
        Verdict::EvidenceFree
    );
}

/// A Git that runs is the holder whatever this host's clock says about its start.
/// Why:
///  A forward step of the wall clock moves a live holder's derived start past the lock's change time,
///  and the pre-fix rule read that as a reused number,
///  so the commit behind the lock was told that nothing proves a holder alive (audit of 2026-11-09,
/// finding 3).
#[test]
fn a_git_whose_start_the_clock_puts_after_the_lock_is_still_the_holder() {
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, start_after(42)), Some(no_holders(Path::new(""), 0, 0)))),
        Verdict::ProvenAlive {
            pid: 42,
            by_descriptor: false,
            command: Some(String::from("git")),
        },
        "a live Git's lock is held, not evidence-free, after a clock step"
    );
}

/// The other direction of the same step:
/// a program that is not named like Git is no proof of a holder,
/// before or after the lock's change time,
/// because a later program with that number reads the same way.
#[test]
fn a_program_that_is_not_named_like_git_is_no_holder_whatever_its_start() {
    for started_at_ms in [500.0_f64, 5000.0] {
        assert_eq!(
            classify_index_lock(&evidence_with(PidFile::Owner(42, process_of(started_at_ms, 10, Some("sleep"))), Some(no_holders(Path::new(""), 0, 0)))),
            Verdict::EvidenceFree,
            "a sleep that this host's clock puts at {started_at_ms} is no holder"
        );
    }
}

/// A number that this process table does not have is no proof that the lock's owner is gone.
/// Why:
///  Git writes its own number into the lock's PID file and no PID namespace,
///  and a number means a process only inside the namespace that wrote it:
///  a live Git in another PID namespace (a container with this repository mounted in)
///  writes a number this table has never had,
///  and through its editor it holds the lock with no descriptor open.
///  Reading that absence as a dead owner ended the queued commit at once with `IndexInUse`,
///  where a live holder is waited for (cross-vendor review of 2026-11-09,
/// finding 2).
#[test]
fn a_pid_file_number_this_table_does_not_have_is_no_proof_of_a_dead_owner() {
    let lock: &Path = Path::new("/repo/index.lock");
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, start_missing(42)), Some(no_holders(lock, 0, 0)))),
        Verdict::EvidenceFree,
        "a holder of another PID namespace has no process here, and it may be alive"
    );
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, start_missing(42)), None)),
        Verdict::EvidenceFree,
        "a number this table does not have proves nothing even without a descriptor scan"
    );
    // The same evidence with a live Git in the PID file is the holder, and nothing here read it as gone.
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, start_before(42)), Some(no_holders(lock, 0, 0)))),
        Verdict::ProvenAlive {
            pid: 42,
            by_descriptor: false,
            command: Some(String::from("git")),
        }
    );
}

/// Without a holder scan,
///  a PID file alone decides,
///  and a malformed,
///  unreadable or unknown one does not.
#[test]
fn a_skipped_scan_leaves_only_the_pid_file() {
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, start_before(42)), None)),
        Verdict::ProvenAlive {
            pid: 42,
            by_descriptor: false,
            command: Some(String::from("git")),
        }
    );
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, start_missing(42)), None)),
        Verdict::EvidenceFree
    );
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Malformed(String::from("x")), None)),
        Verdict::EvidenceFree
    );
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Unreadable(String::from("denied")), None)),
        Verdict::EvidenceFree
    );
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, start_unknown(42)), None)),
        Verdict::EvidenceFree
    );
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, process_of(500.0, 10, None)), None)),
        Verdict::EvidenceFree,
        "a running process whose command name could not be read proves nothing"
    );
}

/// No lock gives no evidence,
///  a PID file proving a live owner skips the scan,
///  and any other lock is scanned.
#[test]
fn gathering_skips_the_scan_only_for_a_proven_live_owner() {
    let root: PathBuf = fixture("evidence-gather");
    let real_index: PathBuf = root.join("index");
    assert_eq!(gather_with(real_index.as_path(), start_before, scan_never).expect("no lock is Ok"), None);

    std::fs::write(root.join("index.lock"), b"").expect("lock");
    std::fs::write(root.join("index~pid.lock"), "pid 42\n").expect("PID file");
    let proven: Evidence = gather_with(real_index.as_path(), start_before, scan_never)
        .expect("gather")
        .expect("lock present");
    assert!(proven.holders.is_none());
    assert_eq!(proven.lock_path, root.join("index.lock"));

    let scanned: Evidence = gather_with(real_index.as_path(), start_missing, scan_one)
        .expect("gather")
        .expect("lock present");
    assert_eq!(scanned.pid_file, PidFile::Owner(42, start_missing(42)));
    assert_eq!(scanned.holders, Some(scan_one(root.as_path(), 0, 0)));
    assert_eq!(classify_index_lock(&scanned), Verdict::ProvenAlive {
        pid: 55,
        by_descriptor: true,
        command: Some(String::from("git")),
    });
    remove(root.as_path());
}

/// The production gather reports no evidence for a repository without a lock.
#[test]
fn the_host_gather_reports_no_lock_when_none_exists() {
    let root: PathBuf = fixture("evidence-host-gather");
    assert_eq!(gather_index_lock_evidence(root.join("index").as_path()).expect("Ok"), None);
    remove(root.as_path());
}

/// The waiting line names the proof,
/// how long the wait lasts before it ends,
/// what becomes of the command then,
///  and ends with a newline;
/// a missing command is not invented.
#[test]
fn the_proven_holder_line_names_its_proof() {
    let end: WaitEnd<'_> = WaitEnd { within: MERGE_LOCK_CEILING, ending: "git merge meets Git's own refusal" };
    let descriptor: String = proven_holder_line(55, true, Some("git"), Path::new("/repo/index.lock"), end);
    let ceiling_seconds: u64 = MERGE_LOCK_CEILING.as_secs();
    assert_eq!(
        descriptor,
        format!(
            "monochromatic-git-policy-cli: waiting for PID 55 (git), which holds /repo/index.lock (proven by an open descriptor); this waits until that process lets go, for at most {ceiling_seconds} seconds; after that, git merge meets Git's own refusal, and the lock is left in place.\n"
        )
    );
    assert!(!descriptor.contains("giving up"), "a notice while the wait goes on never says the wait ended; only the ceiling's line does: {descriptor}");
    let file: String = proven_holder_line(9, false, None, Path::new("/repo/index.lock"), end);
    assert!(file.contains("waiting for PID 9, which holds"));
    assert!(file.contains("(proven by its Git lock PID file)"));
}

/// The change time is whole milliseconds since the epoch,
/// with the nanosecond part rounded down.
///  Permission changes move the change time,
/// and the test retries until the millisecond part is not zero.
#[test]
#[cfg(unix)]
fn the_change_time_rounds_nanoseconds_down_to_milliseconds() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let root: PathBuf = fixture("lock-ctime-milliseconds");
    let path: PathBuf = root.join("index.lock");
    std::fs::write(&path, b"x").expect("the lock file is written");
    let mut observed: bool = false;
    for attempt in 0..50_u32 {
        let mode: u32 = if attempt % 2 == 0 { 0o600 } else { 0o640 };
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).expect("the change time moves");
        let metadata: std::fs::Metadata = std::fs::symlink_metadata(&path).expect("the lock is stat-ed");
        let expected: i64 = metadata.ctime() * 1000 + metadata.ctime_nsec() / 1_000_000;
        let read: LockMetadata = read_lock_metadata(&path).expect("the lock is read").expect("the lock is present");
        assert_eq!(read.ctime_ms, expected);
        if metadata.ctime_nsec() / 1_000_000 != 0 {
            observed = true;
            break;
        }
    }
    assert!(observed, "a change time with a millisecond part was observed");
    remove(root.as_path());
}

/// A scan that could not look at hundreds of processes is summarized,
/// not listed.
/// Why:
///  On 2026-11-07 the notice named every process the account could not read,
/// which buried what a person needs to know.
#[test]
fn many_unreadable_processes_are_summarized_in_the_wait_line() {
    let reasons: Vec<String> = (0..200).map(|number| { return format!("PID {number}: Permission denied (os error 13)"); }).collect();
    let scan: HolderEvidence = HolderEvidence { method: "proc-fd", holders: Vec::new(), partial: reasons.clone(), unresolvable: Vec::new() };
    let evidence: Evidence = evidence_with(PidFile::Absent, Some(scan));
    let end: WaitEnd<'_> = WaitEnd { within: crate::wait_ceilings::LANDING_LOCK_CEILING, ending: crate::service_index_attempts::COMMIT_WITHDRAWN };
    let line: String = unproven_wait_line(Path::new("/repo/index.lock"), Some(&evidence), end);
    assert!(line.contains(format!("{} processes could not be looked at", reasons.len()).as_str()), "{line}");
    assert!(line.contains("PID 0: Permission denied"), "{line}");
    assert!(!line.contains("PID 100: Permission denied"), "a line that lists every reason is not a summary: {line}");
}

/// A scan that could not identify a descriptor whose record names the lock.
fn one_unresolvable(_path: &Path, _device: u64, _inode: u64) -> HolderEvidence {
    return HolderEvidence {
        method: "proc-fd",
        holders: Vec::new(),
        partial: Vec::new(),
        unresolvable: vec![String::from("PID 4242 fd 3: its mount is a `fuse` file system, whose attributes are not in the memory of this kernel, so the file is not read")],
    };
}

/// A PID file naming a number this table does not have is no proof that the owner is dead,
/// whether or not the scan could identify every descriptor that names the lock:
/// the holder may be alive in another PID namespace,
/// so the verdict is evidence-free and the commit waits within its ceiling as it does for a live holder
/// (cross-vendor review of 2026-11-09,
/// findings 2 and 4).
#[test]
fn an_unidentified_holder_and_an_absent_number_both_leave_the_owner_unproven() {
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, start_missing(42)), Some(one_unresolvable(Path::new(""), 0, 0)))),
        Verdict::EvidenceFree,
        "a live holder the scan could not identify is not dead because its number is absent here"
    );
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, start_missing(42)), Some(no_holders(Path::new(""), 0, 0)))),
        Verdict::EvidenceFree,
        "a number this table does not have is no proof of death, whatever the scan saw"
    );
}
