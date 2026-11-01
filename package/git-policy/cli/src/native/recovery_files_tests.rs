//! Controls for recovery file operations on disposable files.

use super::*;
use crate::test_support::{fixture, remove};
use crate::transaction_journal::FileIdentity;

/// The recorded identity of an existing file,
///  as a landing records it.
pub(crate) fn lock_identity_of(path: &Path) -> LockIdentity {
    let (device, inode, _) = file_identity_of(path).expect("identity");
    return LockIdentity {
        file: FileIdentity { device, inode },
        fs_id: filesystem_identity(path).expect("filesystem identity"),
    };
}

/// Existence probes tell missing from present and report other failures.
#[cfg(unix)]
#[test]
fn existence_probes_report_failures() {
    let root: PathBuf = fixture("recovery-files-exists");
    let file: PathBuf = root.join("file");
    std::fs::write(&file, b"x").expect("file");
    assert!(recovery_path_exists(file.as_path()).expect("probe"));
    assert!(!recovery_path_exists(root.join("missing").as_path()).expect("probe"));
    let under_file: PathBuf = file.join("child");
    assert!(
        recovery_path_exists(under_file.as_path())
            .expect_err("not a directory")
            .0
            .starts_with("probing ")
    );
    assert!(path_present(under_file.as_path()).is_err());
    let dangling: PathBuf = root.join("dangling");
    std::os::unix::fs::symlink(root.join("missing"), &dangling).expect("symlink");
    // A dangling link is present for `lstat` and absent for `access`.
    assert!(path_present(dangling.as_path()).expect("lstat"));
    assert!(!recovery_path_exists(dangling.as_path()).expect("access"));
    assert!(!path_present(root.join("missing").as_path()).expect("lstat"));
    remove(root.as_path());
}

/// Recovery files are read exactly,
///  never through links.
#[cfg(unix)]
#[test]
fn recovery_files_are_regular() {
    let root: PathBuf = fixture("recovery-files-read");
    let file: PathBuf = root.join("record.json");
    std::fs::write(&file, b"{}\n").expect("file");
    assert_eq!(read_recovery_file(file.as_path()).expect("read"), b"{}\n");
    assert_eq!(
        read_recovery_file(root.join("missing").as_path())
            .expect_err("missing")
            .0,
        format!(
            "Transaction recovery file is missing: {}",
            root.join("missing").display()
        )
    );
    let link: PathBuf = root.join("link");
    std::os::unix::fs::symlink(&file, &link).expect("symlink");
    assert_eq!(
        read_recovery_file(link.as_path()).expect_err("link").0,
        format!("Unsafe transaction recovery file: {}", link.display())
    );
    assert_eq!(
        read_recovery_file(root.as_path()).expect_err("directory").0,
        format!("Unsafe transaction recovery file: {}", root.display())
    );
    assert!(read_recovery_file(file.join("x").as_path()).is_err());
    assert_eq!(
        read_optional(file.as_path()).expect("optional"),
        Some(b"{}\n".to_vec())
    );
    assert_eq!(
        read_optional(root.join("missing").as_path()).expect("optional"),
        None
    );
    assert!(read_optional(root.as_path()).is_err());
    remove(root.as_path());
}

/// Comparison streams whole files and fails on missing ones.
#[test]
fn comparison_streams_whole_files() {
    let root: PathBuf = fixture("recovery-files-equal");
    let left: PathBuf = root.join("left");
    let right: PathBuf = root.join("right");
    let mut bytes: Vec<u8> = vec![7; COMPARISON_CHUNK * 2 + 5];
    std::fs::write(&left, &bytes).expect("left");
    std::fs::write(&right, &bytes).expect("right");
    assert!(files_equal(left.as_path(), right.as_path()).expect("equal"));
    // Exactly one chunk long on both sides.
    std::fs::write(&left, &bytes[..COMPARISON_CHUNK]).expect("left chunk");
    std::fs::write(&right, &bytes[..COMPARISON_CHUNK]).expect("right chunk");
    assert!(files_equal(left.as_path(), right.as_path()).expect("one chunk"));
    bytes[COMPARISON_CHUNK + 1] = 8;
    std::fs::write(&right, &bytes).expect("second chunk differs");
    std::fs::write(&left, vec![7; COMPARISON_CHUNK * 2 + 5]).expect("left");
    assert!(!files_equal(left.as_path(), right.as_path()).expect("differs"));
    std::fs::write(&right, vec![7; COMPARISON_CHUNK * 2 + 4]).expect("shorter");
    assert!(!files_equal(left.as_path(), right.as_path()).expect("shorter"));
    std::fs::write(&left, b"").expect("empty");
    std::fs::write(&right, b"").expect("empty");
    assert!(files_equal(left.as_path(), right.as_path()).expect("empty"));
    assert!(
        files_equal(left.as_path(), root.join("missing").as_path())
            .expect_err("missing")
            .0
            .starts_with("opening ")
    );
    assert!(files_equal(root.join("missing").as_path(), right.as_path()).is_err());
    remove(root.as_path());
}

/// The read limit and the comparison chunk keep their documented sizes.
///  The comparison test sizes its files from the chunk, and no test builds a file as large as the limit,
///  so only the values themselves pin the arithmetic that produces them.
#[test]
fn recovery_sizes_keep_their_documented_values() {
    assert_eq!(RECOVERY_FILE_LIMIT, 2_147_483_648);
    assert_eq!(COMPARISON_CHUNK, 65_536);
}

/// Owned links keep the inode,
///  and a link to another inode is removed and refused.
#[test]
fn owned_links_prove_their_inode() {
    let root: PathBuf = fixture("recovery-files-link");
    let source: PathBuf = root.join("source");
    std::fs::write(&source, b"index").expect("source");
    let (device, inode, regular) = file_identity_of(source.as_path()).expect("identity");
    assert!(regular);
    let linked: PathBuf = root.join("linked");
    std::fs::write(&linked, b"stale").expect("stale link name");
    create_owned_file_link(
        source.as_path(),
        linked.as_path(),
        device.as_str(),
        inode.as_str(),
    )
    .expect("link");
    assert_eq!(file_identity_of(linked.as_path()).expect("linked").1, inode);
    let failure: RecoveryError =
        create_owned_file_link(source.as_path(), linked.as_path(), device.as_str(), "1")
            .expect_err("other inode");
    assert_eq!(
        failure.0,
        format!(
            "Commit transaction file link identity changed: {}",
            linked.display()
        )
    );
    assert!(!linked.exists());
    let other_device: RecoveryError =
        create_owned_file_link(source.as_path(), linked.as_path(), "0", inode.as_str())
            .expect_err("other device");
    assert!(other_device.0.contains("link identity changed"));
    assert!(
        create_owned_file_link(
            root.join("missing").as_path(),
            linked.as_path(),
            device.as_str(),
            inode.as_str()
        )
        .expect_err("missing source")
        .0
        .starts_with("linking ")
    );
    remove_file_if_present(linked.as_path()).expect("absent is removed");
    assert!(remove_file_if_present(root.as_path()).is_err());
    remove(root.as_path());
}

/// Only the exact recorded lock is owned;
///  inspection failures are errors,
///  not mismatches.
#[test]
fn owned_locks_match_every_recorded_identity() {
    let root: PathBuf = fixture("recovery-files-owned");
    let lock: PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("lock");
    let journal: LockIdentity = lock_identity_of(lock.as_path());
    assert_eq!(
        owned_lock_mismatch(&journal, lock.as_path()).expect("owned"),
        None
    );
    assert!(assert_owned_lock(&journal, lock.as_path()).is_ok());
    let changed: String = format!("Index lock identity changed: {}", lock.display());
    let mut other_inode: LockIdentity = journal.clone();
    other_inode.file.inode = String::from("1");
    assert_eq!(
        owned_lock_mismatch(&other_inode, lock.as_path()).expect("checked"),
        Some(changed.clone())
    );
    let mut other_device: LockIdentity = journal.clone();
    other_device.file.device = String::from("1");
    assert_eq!(
        owned_lock_mismatch(&other_device, lock.as_path()).expect("checked"),
        Some(changed.clone())
    );
    let mut other_filesystem: LockIdentity = journal.clone();
    other_filesystem.fs_id = String::from("fs-uuid_other");
    assert_eq!(
        owned_lock_mismatch(&other_filesystem, lock.as_path()).expect("checked"),
        Some(changed.clone())
    );
    assert_eq!(
        assert_owned_lock(&other_filesystem, lock.as_path())
            .expect_err("foreign")
            .0,
        changed
    );
    let directory: PathBuf = root.join("dir.lock");
    std::fs::create_dir(&directory).expect("directory lock");
    assert_eq!(
        owned_lock_mismatch(&journal, directory.as_path()).expect("checked"),
        Some(format!(
            "Index lock is not a regular owned file: {}",
            directory.display()
        ))
    );
    assert!(
        owned_lock_mismatch(&journal, root.join("missing").as_path())
            .expect_err("missing lock")
            .0
            .starts_with("inspecting the index lock ")
    );
    remove(root.as_path());
}

/// Release removes only the owned lock and reports what it found.
#[test]
fn release_removes_only_owned_locks() {
    let root: PathBuf = fixture("recovery-files-release");
    let lock: PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("lock");
    let journal: LockIdentity = lock_identity_of(lock.as_path());
    let mut foreign: LockIdentity = journal.clone();
    foreign.file.inode = String::from("1");
    assert_eq!(
        release_owned_lock(&foreign, lock.as_path()).expect("foreign"),
        LockRelease::Foreign
    );
    assert!(lock.exists());
    assert_eq!(
        release_owned_lock(&journal, lock.as_path()).expect("owned"),
        LockRelease::Released
    );
    assert!(!lock.exists());
    assert_eq!(
        release_owned_lock(&journal, lock.as_path()).expect("absent"),
        LockRelease::Absent
    );
    let under_file: PathBuf = root.join("file");
    std::fs::write(&under_file, b"x").expect("file");
    assert!(release_owned_lock(&journal, under_file.join("index.lock").as_path()).is_err());
    remove(root.as_path());
}

/// Probe that removes the lock it is asked about and answers nothing,
///  as a second recoverer releasing the lock between the two inspections does.
fn identity_after_a_peer_removed_the_lock(path: &Path) -> Option<String> {
    std::fs::remove_file(path).expect("the peer removes the lock");
    return None;
}

/// Probe that removes the lock but still answers the real identity,
///  as a peer releasing the lock between the ownership proof and the removal does.
fn identity_before_a_peer_removed_the_lock(path: &Path) -> Option<String> {
    let identity: Option<String> = filesystem_identity(path);
    std::fs::remove_file(path).expect("the peer removes the lock");
    return identity;
}

/// Two recoverers release one dead transaction's lock at once:
///  the one that loses the race finds the lock gone and reports it absent instead of failing.
#[test]
fn a_lock_released_by_a_peer_mid_release_is_absent() {
    let root: PathBuf = fixture("recovery-files-peer-release");
    let lock: PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("lock");
    let journal: LockIdentity = lock_identity_of(lock.as_path());
    assert_eq!(
        release_owned_lock_with(&journal, lock.as_path(), identity_after_a_peer_removed_the_lock)
            .expect("the vanished lock is absent"),
        LockRelease::Absent
    );
    std::fs::write(&lock, b"").expect("lock again");
    let second_journal: LockIdentity = lock_identity_of(lock.as_path());
    assert_eq!(
        release_owned_lock_with(&second_journal, lock.as_path(), identity_before_a_peer_removed_the_lock)
            .expect("the removed lock is absent"),
        LockRelease::Absent
    );
    remove(root.as_path());
}

/// Probe for a mechanism that answers nothing.
fn no_identity(_path: &Path) -> Option<String> {
    return None;
}

/// A lock that still exists but cannot be identified stays an error:
///  nothing proves it is the transaction's own.
#[test]
fn an_unidentifiable_lock_is_still_an_error() {
    let root: PathBuf = fixture("recovery-files-unidentified");
    let lock: PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("lock");
    let journal: LockIdentity = lock_identity_of(lock.as_path());
    let error: RecoveryError = release_owned_lock_with(&journal, lock.as_path(), no_identity)
        .expect_err("unidentified");
    assert!(error.0.starts_with("unable to resolve the filesystem identity for "));
    assert!(lock.exists());
    remove(root.as_path());
}

/// A recovered index is installed through the owned lock with the post-index times.
#[test]
fn recovered_indexes_install_through_the_owned_lock() {
    let root: PathBuf = fixture("recovery-files-install");
    let transaction: PathBuf = root.join("transaction");
    std::fs::create_dir(&transaction).expect("transaction");
    let real_index: PathBuf = root.join("index");
    std::fs::write(&real_index, b"old index").expect("real index");
    let lock: PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"partial").expect("lock");
    let post: PathBuf = transaction.join("post-1.index.recovery");
    std::fs::write(&post, b"new index").expect("post index");
    let earlier: std::time::SystemTime =
        std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000);
    std::fs::File::options()
        .write(true)
        .open(&post)
        .expect("post handle")
        .set_times(
            std::fs::FileTimes::new()
                .set_modified(earlier)
                .set_accessed(earlier),
        )
        .expect("post times");
    let journal: LockIdentity = lock_identity_of(lock.as_path());
    let mut foreign: LockIdentity = journal.clone();
    foreign.file.inode = String::from("1");
    assert_eq!(
        install_recovered_index(
            lock.as_path(),
            real_index.as_path(),
            post.as_path(),
            &foreign
        )
        .expect_err("foreign lock")
        .0,
        format!("Index lock identity changed: {}", lock.display())
    );
    assert_eq!(std::fs::read(&lock).expect("lock untouched"), b"partial");
    install_recovered_index(
        lock.as_path(),
        real_index.as_path(),
        post.as_path(),
        &journal,
    )
    .expect("install");
    assert_eq!(std::fs::read(&real_index).expect("installed"), b"new index");
    assert!(!lock.exists());
    assert!(!transaction.join("install.index").exists());
    assert_eq!(
        std::fs::metadata(&real_index)
            .expect("metadata")
            .modified()
            .expect("mtime"),
        earlier
    );
    assert_eq!(
        file_identity_of(real_index.as_path()).expect("identity").1,
        journal.file.inode
    );
    assert!(
        install_recovered_index(
            lock.as_path(),
            real_index.as_path(),
            post.as_path(),
            &journal
        )
        .expect_err("no lock")
        .0
        .starts_with("opening the index lock ")
    );
    assert!(
        install_recovered_index(
            lock.as_path(),
            real_index.as_path(),
            root.join("missing").as_path(),
            &journal
        )
        .is_err()
    );
    remove(root.as_path());
}

/// Release outcomes carry the incumbent's names.
#[test]
fn release_outcomes_have_names() {
    assert_eq!(lock_release_name(LockRelease::Released), "released");
    assert_eq!(lock_release_name(LockRelease::Absent), "absent");
    assert_eq!(lock_release_name(LockRelease::Foreign), "foreign");
}

/// Git's PID file sits beside the lock.
#[test]
fn pid_files_sit_beside_the_lock() {
    assert_eq!(
        lock_pid_path(Path::new("/repo/.git/index")),
        PathBuf::from("/repo/.git/index~pid.lock")
    );
    assert_eq!(parent_of(Path::new("/repo/x")), Path::new("/repo"));
    assert_eq!(parent_of(Path::new("/")), Path::new("/"));
}

/// Timestamps copy from the source to an open file.
#[test]
fn timestamps_copy_to_open_files() {
    let root: PathBuf = fixture("recovery-files-times");
    let source: PathBuf = root.join("source");
    let target: PathBuf = root.join("target");
    std::fs::write(&source, b"s").expect("source");
    std::fs::write(&target, b"t").expect("target");
    let when: std::time::SystemTime =
        std::time::UNIX_EPOCH + std::time::Duration::from_secs(2_000_000);
    std::fs::File::options()
        .write(true)
        .open(&source)
        .expect("source handle")
        .set_times(
            std::fs::FileTimes::new()
                .set_modified(when)
                .set_accessed(when),
        )
        .expect("source times");
    let handle: std::fs::File = std::fs::File::options()
        .write(true)
        .open(&target)
        .expect("target handle");
    apply_timestamps(source.as_path(), &handle).expect("apply");
    assert_eq!(
        std::fs::metadata(&target)
            .expect("metadata")
            .modified()
            .expect("mtime"),
        when
    );
    assert!(apply_timestamps(root.join("missing").as_path(), &handle).is_err());
    remove(root.as_path());
}
