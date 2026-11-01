//! Controls for foreign `index.lock` evidence:
//!  PID file text and owner comparison,
//!  lock metadata
//! without following links,
//!  classification precedence,
//!  the gather with injected sources,
//!  and the
//! prose the wrapper prints.

use super::*;
use crate::index_lock_holders::Holder;
use crate::test_support::{fixture, remove};
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

/// A start source reporting no process.
fn start_missing(_pid: i64) -> ProcessStart {
    return ProcessStart::Missing;
}

/// A start source reporting a process that started before the lock changed.
fn start_before(_pid: i64) -> ProcessStart {
    return ProcessStart::Running {
        started_at_ms: 500.0,
        resolution_ms: 10,
    };
}

/// A start source reporting a process that started after the lock changed.
fn start_after(_pid: i64) -> ProcessStart {
    return ProcessStart::Running {
        started_at_ms: 5000.0,
        resolution_ms: 10,
    };
}

/// A start source that cannot read the start time.
fn start_unknown(_pid: i64) -> ProcessStart {
    return ProcessStart::Unknown(String::from("probe failed"));
}

/// A holder scan that finds nothing and reports nothing partial.
fn scan_nothing(_path: &Path, _device: u64, _inode: u64) -> HolderEvidence {
    return HolderEvidence {
        method: "proc-fd",
        holders: Vec::new(),
        partial: Vec::new(),
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

/// A live process that started no later than the change,
///  plus its resolution,
///  owns the lock.
#[test]
fn owner_evidence_compares_start_with_the_change_time() {
    let lock: LockMetadata = lock_changed_at_1000();
    assert_eq!(owner_evidence(ProcessStart::Missing, &lock), PidOwner::Missing);
    assert_eq!(
        owner_evidence(ProcessStart::Unknown(String::from("r")), &lock),
        PidOwner::StartUnknown(String::from("r"))
    );
    let before: ProcessStart = ProcessStart::Running {
        started_at_ms: 500.0,
        resolution_ms: 1,
    };
    assert_eq!(owner_evidence(before, &lock), PidOwner::StartedBeforeLock(500.0));
    let at_edge: ProcessStart = ProcessStart::Running {
        started_at_ms: 1020.0,
        resolution_ms: 20,
    };
    assert_eq!(owner_evidence(at_edge, &lock), PidOwner::StartedBeforeLock(1020.0));
    let past_edge: ProcessStart = ProcessStart::Running {
        started_at_ms: 1021.0,
        resolution_ms: 20,
    };
    assert_eq!(owner_evidence(past_edge, &lock), PidOwner::StartedAfterLock(1021.0));
}

/// An absent PID file is absent evidence,
///  and a directory in its place is unreadable.
#[test]
fn the_pid_file_is_absent_or_unreadable() {
    let root: PathBuf = fixture("evidence-pid-file-kinds");
    let lock: LockMetadata = lock_changed_at_1000();
    let absent: PathBuf = root.join("index~pid.lock");
    assert_eq!(read_pid_file_evidence(absent.as_path(), &lock, start_missing), PidFile::Absent);
    std::fs::create_dir(&absent).expect("directory in place of the PID file");
    assert!(matches!(
        read_pid_file_evidence(absent.as_path(), &lock, start_missing),
        PidFile::Unreadable(_)
    ));
    remove(root.as_path());
}

/// A PID file naming a process is classified by the process's start against the lock.
#[test]
fn the_pid_file_names_an_owner_classified_by_its_start() {
    let root: PathBuf = fixture("evidence-pid-file-owner");
    let path: PathBuf = root.join("index~pid.lock");
    std::fs::write(&path, "pid 42\n").expect("PID file");
    let lock: LockMetadata = lock_changed_at_1000();
    assert_eq!(
        read_pid_file_evidence(path.as_path(), &lock, start_missing),
        PidFile::Owner(42, PidOwner::Missing)
    );
    assert_eq!(
        read_pid_file_evidence(path.as_path(), &lock, start_before),
        PidFile::Owner(42, PidOwner::StartedBeforeLock(500.0))
    );
    assert_eq!(
        read_pid_file_evidence(path.as_path(), &lock, start_after),
        PidFile::Owner(42, PidOwner::StartedAfterLock(5000.0))
    );
    assert_eq!(
        read_pid_file_evidence(path.as_path(), &lock, start_unknown),
        PidFile::Owner(42, PidOwner::StartUnknown(String::from("probe failed")))
    );
    remove(root.as_path());
}

/// Text that is not `pid <n>` is malformed,
///  and its quoted prefix is at most 64 UTF-16 units.
#[test]
fn malformed_pid_files_quote_at_most_sixty_four_units() {
    let root: PathBuf = fixture("evidence-pid-file-malformed");
    let path: PathBuf = root.join("index~pid.lock");
    let lock: LockMetadata = lock_changed_at_1000();
    std::fs::write(&path, "x".repeat(100)).expect("long text");
    assert_eq!(
        read_pid_file_evidence(path.as_path(), &lock, start_missing),
        PidFile::Malformed("x".repeat(64))
    );
    std::fs::write(&path, "\u{1F600}".repeat(40)).expect("emoji text");
    assert_eq!(
        read_pid_file_evidence(path.as_path(), &lock, start_missing),
        PidFile::Malformed("\u{1F600}".repeat(32))
    );
    std::fs::write(&path, "short").expect("short text");
    assert_eq!(
        read_pid_file_evidence(path.as_path(), &lock, start_missing),
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
///  then a PID file that started before the
/// lock proves a live owner;
///  then one naming a gone process proves a dead owner.
#[test]
fn classification_prefers_a_descriptor_then_the_pid_file() {
    let holder: HolderEvidence = HolderEvidence {
        method: "proc-fd",
        holders: vec![Holder {
            pid: 55,
            command: Some(String::from("git")),
        }],
        partial: Vec::new(),
    };
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, PidOwner::Missing), Some(holder))),
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
    };
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, PidOwner::StartedBeforeLock(1.0)), Some(empty.clone()))),
        Verdict::ProvenAlive {
            pid: 42,
            by_descriptor: false,
            command: None,
        }
    );
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, PidOwner::Missing), Some(empty.clone()))),
        Verdict::Dead(42)
    );
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, PidOwner::StartedAfterLock(9.0)), Some(empty.clone()))),
        Verdict::EvidenceFree
    );
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Absent, Some(empty))),
        Verdict::EvidenceFree
    );
}

/// Without a holder scan,
///  a PID file alone decides,
///  and a malformed,
///  unreadable or unknown one does not.
#[test]
fn a_skipped_scan_leaves_only_the_pid_file() {
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, PidOwner::StartedBeforeLock(1.0)), None)),
        Verdict::ProvenAlive {
            pid: 42,
            by_descriptor: false,
            command: None,
        }
    );
    assert_eq!(
        classify_index_lock(&evidence_with(PidFile::Owner(42, PidOwner::Missing), None)),
        Verdict::Dead(42)
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
        classify_index_lock(&evidence_with(PidFile::Owner(42, PidOwner::StartUnknown(String::from("r"))), None)),
        Verdict::EvidenceFree
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
    assert_eq!(scanned.pid_file, PidFile::Owner(42, PidOwner::Missing));
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

/// Each PID file state has its own sentence,
///  quoting malformed text.
#[test]
fn pid_file_prose_names_each_state() {
    assert_eq!(
        describe_pid_file(&PidFile::Absent),
        "no PID file (the owner did not run with core.lockfilePid=true)"
    );
    assert_eq!(describe_pid_file(&PidFile::Malformed(String::from("x"))), "a malformed PID file \"x\"");
    assert_eq!(
        describe_pid_file(&PidFile::Owner(42, PidOwner::Missing)),
        "PID file names PID 42, which no longer runs"
    );
    assert_eq!(
        describe_pid_file(&PidFile::Owner(42, PidOwner::StartedBeforeLock(1.0))),
        "PID file names PID 42, which runs and started before the lock changed"
    );
    assert_eq!(
        describe_pid_file(&PidFile::Owner(42, PidOwner::StartUnknown(String::from("r")))),
        "PID file names PID 42, which runs but whose start time is unreadable (r)"
    );
    let reused: String = describe_pid_file(&PidFile::Owner(42, PidOwner::StartedAfterLock(0.0)));
    assert!(reused.contains("1970-01-01T00:00:00.000Z"));
    assert!(reused.ends_with("so the PID was reused"));
}

/// Holder prose names the holders with commands,
///  the scan method,
///  and only the first partial notes.
#[test]
fn holder_prose_names_holders_and_limits_partial_notes() {
    let none: Evidence = evidence_with(PidFile::Absent, Some(scan_nothing(Path::new("/"), 0, 0)));
    assert_eq!(describe_holders(&none), "proc-fd scan found no process holding it open");
    let one: Evidence = evidence_with(PidFile::Absent, Some(scan_one(Path::new("/"), 0, 0)));
    assert_eq!(describe_holders(&one), "proc-fd scan found PID 55 (git)");
    let skipped: Evidence = evidence_with(PidFile::Absent, None);
    assert_eq!(describe_holders(&skipped), "no open-holder scan");
    let partial: HolderEvidence = HolderEvidence {
        method: "lsof",
        holders: Vec::new(),
        partial: vec![String::from("a"), String::from("b"), String::from("c"), String::from("d")],
    };
    assert_eq!(
        describe_holders(&evidence_with(PidFile::Absent, Some(partial))),
        "lsof scan found no process holding it open; 4 partial-evidence note(s): a; b; c; ..."
    );
}

/// The evidence line names the lock,
///  its identity,
///  its change time and both sources.
#[test]
fn the_evidence_line_names_the_lock_and_its_sources() {
    let line: String = describe_index_lock_evidence(&evidence_with(PidFile::Absent, Some(scan_nothing(Path::new("/"), 0, 0))));
    assert!(line.starts_with("/repo/index.lock (device 3, inode 9, changed 1970-01-01T00:00:01.000Z); "));
    assert!(line.contains("no PID file"));
    assert!(line.ends_with("proc-fd scan found no process holding it open"));
}

/// The waiting line names the proof and ends with a newline;
///  a missing command is not invented.
#[test]
fn the_proven_holder_line_names_its_proof() {
    let descriptor: String = proven_holder_line(55, true, Some("git"), Path::new("/repo/index.lock"), 1800);
    assert_eq!(
        descriptor,
        "cli-git: waiting for PID 55 (git), which holds /repo/index.lock (proven by an open descriptor); giving up after 1800 seconds (CLI_GIT_LOCK_WAIT_SECONDS changes this), leaving the lock in place.\n"
    );
    let file: String = proven_holder_line(9, false, None, Path::new("/repo/index.lock"), 1800);
    assert!(file.contains("waiting for PID 9, which holds"));
    assert!(file.contains("(proven by its Git lock PID file)"));
}

/// The change time is whole milliseconds since the epoch, with the nanosecond part rounded down.
///  Permission changes move the change time, and the test retries until the millisecond part is not zero.
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
