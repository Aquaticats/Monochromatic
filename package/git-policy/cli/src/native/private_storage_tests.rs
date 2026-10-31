//! Controls for the private file primitives,
//!  each in its own disposable directory.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// The permission bits of a path,
///  without its file type.
#[cfg(unix)]
fn mode_of(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    return std::fs::symlink_metadata(path)
        .expect("metadata")
        .permissions()
        .mode()
        & 0o7777;
}

/// A private file holds exactly the bytes,
///  mode `0600`,
///  and is never replaced.
#[test]
fn a_private_file_is_created_once_with_its_exact_bytes() {
    let root: PathBuf = fixture("private-write");
    let path: PathBuf = root.join("record.json");
    write_private_file(path.as_path(), b"{\"a\":1}\n").expect("first write");
    assert_eq!(std::fs::read(&path).expect("read back"), b"{\"a\":1}\n");
    #[cfg(unix)]
    assert_eq!(mode_of(path.as_path()), 0o600);
    let second: std::io::Error =
        write_private_file(path.as_path(), b"other").expect_err("second write must fail");
    assert_eq!(second.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(std::fs::read(&path).expect("unchanged"), b"{\"a\":1}\n");
    remove(root.as_path());
}

/// Neither writer follows a planted final link,
///  so the link's target is never created.
#[cfg(unix)]
#[test]
fn writers_refuse_a_planted_link() {
    let root: PathBuf = fixture("private-link-write");
    let target: PathBuf = root.join("target");
    let link: PathBuf = root.join("record.json");
    std::os::unix::fs::symlink(&target, &link).expect("plant link");
    assert!(write_private_file(link.as_path(), b"x").is_err());
    assert!(write_exclusive_synced(link.as_path(), b"x").is_err());
    assert!(!target.exists(), "the link target must not be created");
    remove(root.as_path());
}

/// The synced writer keeps the creation mode and the bytes.
#[test]
fn the_synced_writer_creates_an_exclusive_file() {
    let root: PathBuf = fixture("private-synced");
    let path: PathBuf = root.join("owner.json");
    write_exclusive_synced(path.as_path(), b"abc").expect("write");
    assert_eq!(std::fs::read(&path).expect("read"), b"abc");
    #[cfg(unix)]
    assert_eq!(mode_of(path.as_path()) & 0o077, 0, "no group or other bits");
    assert!(write_exclusive_synced(path.as_path(), b"abc").is_err());
    remove(root.as_path());
}

/// The scratch writer keeps the creation mode and the bytes,
///  refuses an existing file and a planted link,
///  and is the one writer that never syncs.
#[test]
fn the_scratch_writer_creates_an_exclusive_private_file() {
    let root: PathBuf = fixture("private-scratch");
    let path: PathBuf = root.join("input.bin");
    write_exclusive_scratch(path.as_path(), b"abc").expect("write");
    assert_eq!(std::fs::read(&path).expect("read"), b"abc");
    #[cfg(unix)]
    assert_eq!(mode_of(path.as_path()) & 0o077, 0, "no group or other bits");
    assert!(write_exclusive_scratch(path.as_path(), b"abc").is_err());
    #[cfg(unix)]
    {
        let link: PathBuf = root.join("link.bin");
        std::os::unix::fs::symlink(root.join("target"), &link).expect("plant link");
        assert!(write_exclusive_scratch(link.as_path(), b"x").is_err());
        assert!(!root.join("target").exists(), "the link target must not be created");
    }
    remove(root.as_path());
}

/// A private directory has no group or other bits and its parent must exist.
#[test]
fn a_private_directory_is_owner_only() {
    let root: PathBuf = fixture("private-directory");
    let directory: PathBuf = root.join("lock");
    create_private_directory(directory.as_path()).expect("create");
    assert!(directory.is_dir());
    #[cfg(unix)]
    assert_eq!(mode_of(directory.as_path()) & 0o077, 0);
    assert!(
        create_private_directory(directory.as_path()).is_err(),
        "exists already"
    );
    assert!(
        create_private_directory(root.join("a/b").as_path()).is_err(),
        "no parent"
    );
    remove(root.as_path());
}

/// `protect_path` sets exactly the requested bits.
#[cfg(unix)]
#[test]
fn protection_sets_the_requested_mode() {
    let root: PathBuf = fixture("private-protect");
    let path: PathBuf = root.join("file");
    std::fs::write(&path, b"x").expect("write");
    protect_path(path.as_path(), 0o640).expect("protect");
    assert_eq!(mode_of(path.as_path()), 0o640);
    protect_path(path.as_path(), PRIVATE_FILE_MODE).expect("protect again");
    assert_eq!(mode_of(path.as_path()), 0o600);
    assert!(protect_path(root.join("missing").as_path(), 0o600).is_err());
    remove(root.as_path());
}

/// Child of the controls below: write a record larger than the process's file-size limit with each writer.
///  Both writers must fail, because only part of the record fits.
#[cfg(unix)]
#[test]
#[ignore = "started as a child process, under a file-size limit, by the controls of the writers"]
fn child_writes_a_record_past_the_size_limit() {
    let directory: PathBuf = PathBuf::from(std::env::var_os(crate::test_support::LIMITED_DIRECTORY_VARIABLE).expect("directory"));
    let record: Vec<u8> = vec![b'x'; 1000];
    assert!(write_private_file(directory.join("private.json").as_path(), record.as_slice()).is_err());
    assert!(write_exclusive_synced(directory.join("synced.json").as_path(), record.as_slice()).is_err());
}

/// A record whose write is cut half way by a full disk or a size limit leaves nothing at its name, not the part
///  that fitted: a record that recovery finds must be whole, and a half-written one stops every later command.
#[cfg(unix)]
#[test]
fn a_write_cut_half_way_leaves_nothing_behind() {
    let root: PathBuf = fixture("private-cut-write");
    let status = crate::test_support::run_child_under_file_size_limit(
        "private_storage::tests::child_writes_a_record_past_the_size_limit",
        100,
        root.as_path(),
    );
    assert!(status.success(), "the child's writers must both fail: {status:?}");
    let left: Vec<String> = std::fs::read_dir(&root)
        .expect("directory")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .collect();
    assert_eq!(left, Vec::<String>::new(), "no record and no half-written staging file stays");
    remove(root.as_path());
}

/// A filesystem that cannot sync a directory (procfs answers `EINVAL`) is not a failure of the write: the entry cannot
///  be made durable there, and every other filesystem call of the commit works.
#[cfg(target_os = "linux")]
#[test]
fn a_directory_that_cannot_be_synced_is_not_an_error() {
    let procfs: &Path = Path::new("/proc/self");
    if !procfs.is_dir() {
        eprintln!("skipped: no procfs on this system");
        return;
    }
    let direct: std::io::Result<()> = std::fs::File::open(procfs).and_then(|handle| { return handle.sync_all(); });
    assert!(direct.is_err(), "the control: procfs refuses to sync a directory");
    sync_directory(procfs).expect("an unsupported directory sync is skipped");
}

/// Syncing works on a directory and fails on a missing one.
#[test]
fn directory_sync_needs_the_directory() {
    let root: PathBuf = fixture("private-sync");
    sync_directory(root.as_path()).expect("sync");
    #[cfg(unix)]
    assert!(sync_directory(root.join("missing").as_path()).is_err());
    remove(root.as_path());
}

/// Removing a tree removes it,
///  and an absent tree counts as removed.
#[test]
fn tree_removal_tolerates_an_absent_tree() {
    let root: PathBuf = fixture("private-remove");
    let tree: PathBuf = root.join("tree");
    std::fs::create_dir_all(tree.join("inner")).expect("tree");
    std::fs::write(tree.join("inner/file"), b"x").expect("file");
    remove_tree(tree.as_path()).expect("remove");
    assert!(!tree.exists());
    remove_tree(tree.as_path()).expect("already absent");
    remove(root.as_path());
}

/// A file inside a file-named parent cannot be removed as a tree:
///  the error is reported.
#[cfg(unix)]
#[test]
fn tree_removal_reports_other_errors() {
    let root: PathBuf = fixture("private-remove-error");
    let file: PathBuf = root.join("file");
    std::fs::write(&file, b"x").expect("file");
    assert!(remove_tree(file.as_path()).is_err(), "a file is not a tree");
    remove(root.as_path());
}

/// Reads return exact bytes up to the limit and refuse everything unsafe or oversized.
#[test]
fn reads_refuse_unsafe_and_oversized_paths() {
    let root: PathBuf = fixture("private-read");
    let path: PathBuf = root.join("record");
    std::fs::write(&path, b"12345").expect("write");
    assert_eq!(
        read_regular_file(path.as_path(), 5).expect("at the limit"),
        b"12345"
    );
    assert!(matches!(
        read_regular_file(path.as_path(), 4),
        Err(ReadRefusal::TooLarge)
    ));
    assert!(matches!(
        read_regular_file(root.join("missing").as_path(), 5),
        Err(ReadRefusal::Missing)
    ));
    assert!(matches!(
        read_regular_file(root.as_path(), 5),
        Err(ReadRefusal::NotRegular)
    ));
    #[cfg(unix)]
    {
        let link: PathBuf = root.join("link");
        std::os::unix::fs::symlink(&path, &link).expect("link");
        assert!(matches!(
            read_regular_file(link.as_path(), 5),
            Err(ReadRefusal::NotRegular)
        ));
        // A path through a regular file names nothing, like a missing file.
        assert!(matches!(
            read_regular_file(path.join("inner").as_path(), 5),
            Err(ReadRefusal::Missing)
        ));
    }
    remove(root.as_path());
}

/// An unreadable file is an input-output refusal,
///  distinct from a missing one.
#[cfg(unix)]
#[test]
fn an_unreadable_file_is_an_io_refusal() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("private-unreadable");
    let path: PathBuf = root.join("record");
    std::fs::write(&path, b"x").expect("write");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).expect("chmod");
    // A process with the capability to override permissions can still read it; the
    // refusal is asserted only where the operating system enforces the mode.
    if std::fs::read(&path).is_err() {
        assert!(matches!(
            read_regular_file(path.as_path(), 5),
            Err(ReadRefusal::Io(_))
        ));
    }
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).expect("restore");
    remove(root.as_path());
}

/// A file smaller than the limit is read whole.
#[test]
fn a_file_below_the_limit_is_read_whole() {
    let root: PathBuf = fixture("private-read-small");
    let path: PathBuf = root.join("record");
    std::fs::write(&path, b"abc").expect("write");
    assert_eq!(
        read_regular_file(path.as_path(), 1024).expect("read"),
        b"abc"
    );
    remove(root.as_path());
}

/// A file whose size reads as zero but which yields more than the limit (as a growing file
/// would) is still refused:
///  the read itself is capped,
///  not only the size check.
#[cfg(target_os = "linux")]
#[test]
fn the_read_is_capped_beyond_the_reported_size() {
    let path: &Path = Path::new("/proc/self/stat");
    assert_eq!(std::fs::metadata(path).expect("metadata").len(), 0);
    assert!(matches!(
        read_regular_file(path, 10),
        Err(ReadRefusal::TooLarge)
    ));
    let whole: Vec<u8> = read_regular_file(path, 4096).expect("whole record");
    assert!(whole.len() > 10);
}
