//! Controls for the read-only comparison of the old tree with the installed one.

use super::*;
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// How much a control extends each file per step while the comparison reads it.
/// Why:
///  A sparse extension of this size lands faster than a reader passes the bytes already there,
/// so a reader with no bound
///  never reaches the end of either file.
const GROWTH_BYTES: u64 = 1024 * 1024 * 1024;

/// Two trees with the same file,
/// link and directory,
/// the second one made by copying the first with its times.
fn twin_trees(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let old: PathBuf = root.join("old");
    let installed: PathBuf = root.join("installed");
    for tree in [&old, &installed] {
        std::fs::create_dir_all(tree.join("dir")).expect("directory");
        std::fs::write(tree.join("dir/file"), b"same\n").expect("file");
        std::os::unix::fs::symlink("dir/file", tree.join("link")).expect("link");
    }
    let mtime = std::fs::metadata(old.join("dir/file")).expect("metadata").modified().expect("mtime");
    std::fs::OpenOptions::new().write(true).open(installed.join("dir/file")).expect("open").set_modified(mtime).expect("same time");
    return (root, old, installed);
}

/// Trees that hold the same entries have no difference,
/// and the installed tree may hold more.
#[test]
fn equal_trees_have_no_difference_and_more_in_the_installed_tree_is_fine() {
    let (root, old, installed) = twin_trees("compare-equal");
    assert_eq!(first_difference(old.as_path(), installed.as_path()).expect("compared"), None);
    std::fs::write(installed.join("added-since"), b"new\n").expect("added");
    assert_eq!(first_difference(old.as_path(), installed.as_path()).expect("compared"), None);
    remove(root.as_path());
}

/// A missing entry,
/// another kind of entry,
/// another size or time,
/// and another link target are each named,
/// and nothing is changed.
#[test]
fn each_kind_of_difference_is_named_and_nothing_is_changed() {
    let (root, old, installed) = twin_trees("compare-differences");
    std::fs::write(installed.join("dir/file"), b"another length\n").expect("rewrite");
    let size = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(size.contains("dir/file") && size.contains("differs"), "{size}");
    assert_eq!(std::fs::read(old.join("dir/file")).expect("old"), b"same\n", "the old tree is unchanged");
    assert_eq!(std::fs::read(installed.join("dir/file")).expect("installed"), b"another length\n", "the installed tree is unchanged");
    std::fs::remove_file(installed.join("dir/file")).expect("remove");
    let missing = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(missing.contains("dir/file") && missing.contains("missing"), "{missing}");
    std::fs::create_dir(installed.join("dir/file")).expect("a directory in its place");
    let kind = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(kind.contains("not the same kind"), "{kind}");
    std::fs::remove_dir(installed.join("dir/file")).expect("remove");
    std::fs::write(installed.join("dir/file"), b"same\n").expect("restore");
    std::fs::remove_file(installed.join("link")).expect("remove link");
    std::os::unix::fs::symlink("elsewhere", installed.join("link")).expect("another target");
    let target = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(target.contains("link") && target.contains("points elsewhere"), "{target}");
    remove(root.as_path());
}

/// What:
///  Files that something keeps extending while they are read do not hold the comparison open.
/// Why:
///   The comparison read until both sides met their end,
/// and a writer that stayed ahead of the reader kept that
///  end away for good,
/// so a conversion waited inside it with its claim held and never finished.
/// The lengths the two sides had when
///  they were opened bound the read,
/// and a file that grew past its own is a difference the conversion reports rather than
///  waits on.
#[test]
fn a_file_that_keeps_growing_does_not_hold_the_comparison_open() {
    let (root, old, installed) = twin_trees("compare-growing");
    let first_watched: PathBuf = old.join("dir/file");
    let second_watched: PathBuf = installed.join("dir/file");
    let first_compared: PathBuf = old.clone();
    let second_compared: PathBuf = installed.clone();
    let growing: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    let started: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let writer: Arc<std::sync::Mutex<Option<std::thread::JoinHandle<()>>>> = Arc::new(std::sync::Mutex::new(None));
    let stop: Arc<AtomicBool> = Arc::clone(&growing);
    let once: Arc<AtomicBool> = Arc::clone(&started);
    let spawned: Arc<std::sync::Mutex<Option<std::thread::JoinHandle<()>>>> = Arc::clone(&writer);
    let (extended, first_extension) = std::sync::mpsc::channel::<()>();
    // The callback is installed on the thread that compares, because `finished_within` runs the work on a thread of its own.
    let comparison = crate::test_support::finished_within(move || {
        watch_contents_opened(Some(Box::new(move |_path: &Path| {
            if once.swap(true, Ordering::SeqCst) {
                return;
            }
            let stopping: Arc<AtomicBool> = Arc::clone(&stop);
            let telling: std::sync::mpsc::Sender<()> = extended.clone();
            let first_extended: PathBuf = first_watched.clone();
            let second_extended: PathBuf = second_watched.clone();
            let handle: std::thread::JoinHandle<()> = std::thread::spawn(move || {
                let first: std::fs::File = std::fs::OpenOptions::new().write(true).open(first_extended.as_path()).expect("the old file");
                let second: std::fs::File = std::fs::OpenOptions::new().write(true).open(second_extended.as_path()).expect("the installed file");
                let mut length: u64 = first.metadata().expect("metadata").len();
                while stopping.load(Ordering::SeqCst) {
                    length += GROWTH_BYTES;
                    // A sparse extension: the bytes a reader passes afterwards are zeros the kernel supplies without storing them.
                    first.set_len(length).expect("extend the old file");
                    second.set_len(length).expect("extend the installed file");
                    let _ = telling.send(());
                }
            });
            *spawned.lock().expect("the slot of the writer") = Some(handle);
            // The control waits for that first extension, so the growth the read reports is there before its first byte.
            let _ = first_extension.recv_timeout(std::time::Duration::from_secs(2));
        })));
        let decided = first_difference(first_compared.as_path(), second_compared.as_path());
        watch_contents_opened(None);
        return decided;
    });
    growing.store(false, Ordering::SeqCst);
    let stopped: Option<std::thread::JoinHandle<()>> = writer.lock().expect("the slot of the writer").take();
    if let Some(handle) = stopped {
        handle.join().expect("the writer stopped");
    }
    let Some(outcome) = comparison else { panic!("the comparison did not finish within five seconds although the files only grew; its read has no bound") };
    let difference: String = outcome.expect("compared").expect("a difference");
    assert!(difference.contains("dir/file"), "{difference}");
    remove(root.as_path());
}

/// What:
///  An edit of the same length whose modification time was put back is a difference,
/// because the contents are what a person changed and the old tree is the copy that holds them.
/// Why:
///   Size and modification time alone cannot see such an edit;
/// the contents of a file are compared where the
///  comparison can decide the deletion of it.
#[test]
fn a_same_size_edit_with_the_time_put_back_is_a_difference() {
    let (root, old, installed) = twin_trees("compare-same-size");
    let file: PathBuf = old.join("dir/file");
    let before: std::fs::Metadata = std::fs::metadata(&file).expect("metadata");
    let bytes: Vec<u8> = std::fs::read(&file).expect("bytes");
    let mut edited: Vec<u8> = bytes.clone();
    edited[0] = b'D';
    std::fs::write(&file, &edited).expect("an edit of the same length");
    std::fs::OpenOptions::new().write(true).open(&file).expect("open the edited file").set_times(std::fs::FileTimes::new().set_modified(before.modified().expect("modified"))).expect("put the time back");
    assert_eq!(std::fs::metadata(&file).expect("metadata").len(), before.len(), "the length is unchanged");
    let difference: String = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(difference.contains("dir/file") && difference.contains("contents"), "{difference}");
    remove(root.as_path());
}

/// How far after an edit the proving pass of a control is recorded as having finished.
/// Why:
///  The edit and the pass read the same clock,
/// so the margin puts the pass after the edit whatever the clock's resolution is,
/// which is the sequence the review describes.
const PROOF_MARGIN_MS: i64 = 1_000;

/// What:
///  An edit of the same length whose modification time was put back is a difference even when it was made before the
///  proving pass started.
/// Why:
///   The comparison skipped the contents of a file whose change time was older than the proof,
/// because the proving pass
///  set the copy's own change time before that instant;
/// an edit made after the first copy and before the pass leaves a
///  change time older than the proof,
/// so the old tree was declared equivalent and the only copy of the edited bytes was
///  deleted with it.
/// The third review of this route names it.
#[test]
fn a_same_size_edit_before_the_proving_pass_is_still_a_difference() {
    let (root, old, installed) = twin_trees("compare-before-the-proof");
    let file: PathBuf = old.join("dir/file");
    let before: std::fs::Metadata = std::fs::metadata(&file).expect("metadata");
    let mut edited: Vec<u8> = std::fs::read(&file).expect("bytes");
    edited[0] = b'D';
    std::fs::write(&file, &edited).expect("an edit of the same length");
    std::fs::OpenOptions::new().write(true).open(&file).expect("open the edited file").set_times(std::fs::FileTimes::new().set_modified(before.modified().expect("modified"))).expect("put the time back");
    let proved_at: i64 = crate::worktree_convert_journal::now_milliseconds() + PROOF_MARGIN_MS;
    let difference: String = first_difference_after(old.as_path(), installed.as_path(), Some(proved_at)).expect("compared").expect("a difference");
    assert!(difference.contains("dir/file") && difference.contains("contents"), "{difference}");
    assert_eq!(std::fs::read(&file).expect("old"), edited, "the old tree still holds the edited bytes");
    // An edit made after the instant the proof finished is named as one made after the proof, which is what tells a person
    // reading the report whether the copy or the edit came first.
    let proved_before_it: i64 = crate::worktree_convert_journal::now_milliseconds() - PROOF_MARGIN_MS;
    edited[0] = b'E';
    std::fs::write(&file, &edited).expect("a later edit of the same length");
    std::fs::OpenOptions::new().write(true).open(&file).expect("open the edited file").set_times(std::fs::FileTimes::new().set_modified(before.modified().expect("modified"))).expect("put the time back again");
    let later: String = first_difference_after(old.as_path(), installed.as_path(), Some(proved_before_it)).expect("compared").expect("a difference");
    assert!(later.contains("dir/file") && later.contains("after the copy was proved"), "{later}");
    remove(root.as_path());
}

/// What:
///  A mode change of the old root directory itself is a difference.
/// Why:
///   The old root's own metadata supplied only the device number,
/// so a `chmod` of the old tree's top
///  directory was never compared and the tree that held it was deleted.
#[test]
fn a_mode_change_of_the_old_root_is_a_difference() {
    let (root, old, installed) = twin_trees("compare-root-mode");
    std::fs::set_permissions(old.as_path(), std::fs::Permissions::from_mode(0o700)).expect("a narrowed old root");
    let difference: String = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(difference.contains("the old tree itself") && difference.contains("permission bits"), "{difference}");
    remove(root.as_path());
}

/// What:
///  A mode change of a named pipe is a difference.
/// Why:
///   A FIFO enters none of the directory,
/// link and regular-file branches,
/// so its mode,
/// owner,
/// group
///  and carried attributes were never compared and its distinct permissions were lost with the old tree.
#[test]
fn a_mode_change_of_a_named_pipe_is_a_difference() {
    let (root, old, installed) = twin_trees("compare-fifo-mode");
    crate::subvolume_sync_attrs::make_fifo(old.join("queue").as_path(), 0o600).expect("the old queue");
    crate::subvolume_sync_attrs::make_fifo(installed.join("queue").as_path(), 0o644).expect("the installed queue");
    let difference: String = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(difference.contains("queue") && difference.contains("permission bits"), "{difference}");
    remove(root.as_path());
}

/// What:
///  A named pipe planted at the old entry after the walk read its metadata does not block the
///  comparison.
/// Why:
///  The contents were opened through `File::open`,
/// which blocks on a FIFO and follows a link,
/// and nothing around the call has
/// a deadline;
/// the conversion holds its claim while it waits.
/// The comparison opens both sides through the plain-file open,
///  which refuses anything but a plain file.
#[test]
fn a_named_pipe_planted_before_the_contents_are_read_does_not_block_the_comparison() {
    let (root, old, installed) = twin_trees("compare-fifo-race");
    let target: PathBuf = old.join("dir/file");
    let planted: PathBuf = target.clone();
    let old_tree: PathBuf = old.clone();
    let installed_tree: PathBuf = installed.clone();
    // The callback is installed on the thread that compares, because `finished_within` runs the work on a thread of its own.
    let comparison = crate::test_support::finished_within(move || {
        watch_contents_open(Some(Box::new(move |path: &Path| {
            if path != planted {
                return;
            }
            std::fs::remove_file(planted.as_path()).expect("replace the file");
            crate::subvolume_sync_attrs::make_fifo(planted.as_path(), 0o600).expect("a named pipe");
        })));
        let decided = first_difference_after(old_tree.as_path(), installed_tree.as_path(), Some(0));
        watch_contents_open(None);
        return decided;
    });
    let Some(outcome) = comparison else { panic!("the comparison did not finish within five seconds") };
    let error: std::io::Error = outcome.expect_err("the comparison reports what it cannot read");
    assert!(error.to_string().contains("not a plain file"), "{error}");
    remove(root.as_path());
}

/// What:
///  Give `path` the user attribute `user.cli-git-control`,
///  and answer whether the filesystem took it.
/// Why:
///   A filesystem or a mount without user attributes cannot carry the difference this control makes,
/// which says so
///  instead of failing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function giveUserAttribute(path: string): boolean;
/// ```
fn give_user_attribute(path: &Path) -> bool {
    let name = std::ffi::CString::new("user.cli-git-control").expect("a name without NUL");
    let text = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("a path without NUL");
    let value: &[u8] = b"only in the old tree";
    // SAFETY: both strings are NUL-terminated and outlive the call, and `value` is readable for its own length.
    let result: libc::c_int = unsafe { libc::lsetxattr(text.as_ptr(), name.as_ptr(), value.as_ptr().cast(), value.len(), 0) };
    return result == 0;
}

/// Whether the tests run as root,
/// which can give a file another owner.
fn running_as_root() -> bool {
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    return unsafe { libc::geteuid() } == 0;
}

/// A change of the permission bits of a file or of a directory,
/// of the owner or the group,
/// and of a carried
///  extended attribute is each a difference,
/// although the size and modification time stay the same.
#[test]
fn a_change_of_the_mode_an_owner_or_an_attribute_is_a_difference() {
    let (root, old, installed) = twin_trees("compare-attributes");
    let file: PathBuf = old.join("dir/file");
    let original: std::fs::Permissions = std::fs::metadata(installed.join("dir/file")).expect("installed").permissions();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).expect("a narrowed file");
    let bits = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(bits.contains("dir/file") && bits.contains("permission bits"), "{bits}");
    std::fs::set_permissions(&file, original).expect("the same bits again");
    assert_eq!(first_difference(old.as_path(), installed.as_path()).expect("compared"), None, "the control can report no difference");
    std::fs::set_permissions(old.join("dir"), std::fs::Permissions::from_mode(0o700)).expect("a narrowed directory");
    let directory = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(directory.contains("dir differs") && directory.contains("permission bits"), "{directory}");
    std::fs::set_permissions(old.join("dir"), std::fs::Permissions::from_mode(0o755)).expect("the same bits again");
    if !give_user_attribute(&file) {
        eprintln!("XATTR CONTROL NOT RUN: a_change_of_the_mode_an_owner_or_an_attribute_is_a_difference (the filesystem took no user attribute)");
        remove(root.as_path());
        return;
    }
    let attribute = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(attribute.contains("dir/file") && attribute.contains("extended attributes"), "{attribute}");
    assert_eq!(std::fs::read(installed.join("dir/file")).expect("installed"), b"same\n", "the installed tree is unchanged");
    remove(root.as_path());
}

/// A file given another owner in the old tree is a difference.
/// Giving one needs root,
/// so the control says it did not run elsewhere.
#[test]
fn an_owner_change_is_a_difference() {
    if !running_as_root() {
        eprintln!("OWNER CONTROL NOT RUN: an_owner_change_is_a_difference (giving a file another owner needs root)");
        return;
    }
    let (root, old, installed) = twin_trees("compare-owner");
    let file: PathBuf = old.join("dir/file");
    let name = std::ffi::CString::new(file.as_os_str().as_encoded_bytes()).expect("a path without NUL");
    // SAFETY: `name` is NUL-terminated and outlives the call.
    let result: libc::c_int = unsafe { libc::lchown(name.as_ptr(), 65534, 65534) };
    assert_eq!(result, 0, "give the old tree's file another owner");
    let found = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(found.contains("dir/file") && found.contains("owner or group"), "{found}");
    remove(root.as_path());
}

/// A subvolume inside the old tree is named as something the command does not delete,
/// so the old tree is kept with the path.
#[test]
fn btrfs_a_nested_subvolume_is_named_as_not_deleted() {
    let Some(base) = btrfs_fixture("compare-nested") else { return };
    let old: PathBuf = base.join("old");
    let installed: PathBuf = base.join("installed");
    std::fs::create_dir_all(&old).expect("old");
    std::fs::create_dir_all(installed.join("inner")).expect("installed");
    subvolume_create_at(old.join("inner").as_path()).expect("nested subvolume");
    let found = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(found.contains("inner") && found.contains("subvolume or a mount point") && found.contains("does not delete"), "{found}");
    remove(base.as_path());
}
