//! Controls for the Git directories a restarted service looks at by itself.

use super::*;
use std::ffi::OsString;

/// The first pause of the controls.
const FIRST: Duration = Duration::from_secs(10);

/// A scratch directory under the package's own build directory,
/// removed by the caller.
fn scratch(name: &str) -> PathBuf {
    let directory: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join("claim-watch-tests").join(format!("{name}-{}", std::process::id()));
    let _cleared = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("the scratch directory");
    return directory;
}

/// A Git directory with the work directory the wrapper makes.
fn git_dir_with_work(root: &Path, name: &str) -> PathBuf {
    let git_dir: PathBuf = root.join(name);
    std::fs::create_dir_all(git_dir.join(WORK_DIRECTORY)).expect("the work directory");
    return git_dir;
}

/// The files of a store,
/// by name.
fn files_of(store: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(store).map_or_else(|_| { return Vec::new(); }, |listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); });
    names.sort();
    return names;
}

/// A directory written down by one watch is read back by the watch of the service that starts next.
#[test]
fn a_remembered_directory_is_restored_by_the_next_watch() {
    let root: PathBuf = scratch("restored");
    let git_dir: PathBuf = git_dir_with_work(root.as_path(), "repo.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let mut first: ClaimWatch = ClaimWatch::restored(store.as_path(), None);
    assert!(first.directories().is_empty(), "a new journal watches nothing");
    first.remember(git_dir.as_path());
    first.remember(git_dir.as_path());
    assert_eq!(first.directories(), vec![git_dir.clone()], "remembered once");
    assert_eq!(files_of(store.as_path()).len(), 1, "one file for one directory");
    assert!(!first.needs_retry());
    assert_eq!(ClaimWatch::restored(store.as_path(), None).directories(), vec![git_dir]);
    let _removed = std::fs::remove_dir_all(&root);
}

/// Several directories are each written down and each restored.
#[test]
fn every_directory_has_its_own_entry() {
    let root: PathBuf = scratch("several");
    let one: PathBuf = git_dir_with_work(root.as_path(), "one.git");
    let two: PathBuf = git_dir_with_work(root.as_path(), "two.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let mut watch: ClaimWatch = ClaimWatch::restored(store.as_path(), None);
    watch.remember(one.as_path());
    watch.remember(two.as_path());
    let mut restored: Vec<PathBuf> = ClaimWatch::restored(store.as_path(), None).directories();
    restored.sort();
    assert_eq!(restored, vec![one, two]);
    let _removed = std::fs::remove_dir_all(&root);
}

/// A directory that lost its work directory,
/// a file that holds no path,
/// and a work directory that is a link are certainly of no
/// use:
/// their files are removed,
/// so a removed worktree is not looked at at every start.
/// A file of another kind stays.
#[test]
fn files_that_are_certainly_of_no_use_are_removed() {
    let root: PathBuf = scratch("dropped");
    let gone: PathBuf = git_dir_with_work(root.as_path(), "gone.git");
    let linked: PathBuf = root.join("linked.git");
    std::fs::create_dir_all(linked.join("monochromatic-git-policy-cli")).expect("the parent of the link");
    std::os::unix::fs::symlink(gone.join(WORK_DIRECTORY), linked.join(WORK_DIRECTORY)).expect("the link");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let mut watch: ClaimWatch = ClaimWatch::restored(store.as_path(), None);
    watch.remember(gone.as_path());
    watch.remember(linked.as_path());
    std::fs::write(store.join("junk.watch"), "relative/path\n").expect("a file that names no absolute path");
    std::fs::write(store.join("other.txt"), "left alone\n").expect("a file of another kind");
    std::fs::remove_dir_all(&gone).expect("the worktree is removed");
    let restored: ClaimWatch = ClaimWatch::restored(store.as_path(), None);
    assert!(restored.directories().is_empty());
    assert!(!restored.needs_retry(), "nothing failed to be read");
    assert_eq!(files_of(store.as_path()), vec![String::from("other.txt")], "only the file of another kind stays");
    let _removed = std::fs::remove_dir_all(&root);
}

/// A watch kept in memory only writes nothing and still knows what it was told.
#[test]
fn an_in_memory_watch_writes_nothing() {
    let root: PathBuf = scratch("memory");
    let git_dir: PathBuf = git_dir_with_work(root.as_path(), "repo.git");
    let mut watch: ClaimWatch = ClaimWatch::in_memory();
    watch.remember(git_dir.as_path());
    assert_eq!(watch.directories(), vec![git_dir]);
    assert!(!watch.needs_retry());
    assert!(!root.join(WATCH_DIRECTORY).exists());
    let _removed = std::fs::remove_dir_all(&root);
}

/// A file that cannot be written is not lost:
/// the watch keeps the directory,
/// asks to be tried again,
/// and a try after the fault is
/// mended writes it and hands the directory back,
/// so that a service killed afterwards finds it.
#[test]
fn a_file_that_could_not_be_written_is_written_by_a_later_try() {
    let root: PathBuf = scratch("unwritable");
    let git_dir: PathBuf = git_dir_with_work(root.as_path(), "repo.git");
    // A file where the store's directory should be makes the write fail.
    let blocker: PathBuf = root.join("blocker");
    std::fs::write(&blocker, "not a directory\n").expect("the blocker");
    let store: PathBuf = blocker.join("store");
    let mut watch: ClaimWatch = ClaimWatch::restored(store.as_path(), None);
    watch.remember(git_dir.as_path());
    assert!(watch.directories().is_empty(), "not written, so not yet known to the next service");
    assert!(watch.needs_retry());
    assert!(watch.retry().is_empty(), "the fault is still there");
    assert!(watch.needs_retry());
    assert!(watch.pause(FIRST) > FIRST, "the pause grows with the failures in a row: {:?}", watch.pause(FIRST));
    std::fs::remove_file(&blocker).expect("the fault is mended");
    assert!(watch.retry().is_empty(), "the directory was this process's own, so it is not new to the caller");
    assert!(!watch.needs_retry());
    assert_eq!(ClaimWatch::restored(store.as_path(), None).directories(), vec![git_dir], "the next service finds it");
    let _removed = std::fs::remove_dir_all(&root);
}

/// A note that cannot be read is kept,
/// not removed,
/// and read again once the fault is over;
/// the watch asks to be tried again
/// meanwhile,
/// so that the service neither forgets the repository nor exits idle.
#[test]
fn a_note_that_could_not_be_read_is_kept_and_found_by_a_later_try() {
    let root: PathBuf = scratch("unreadable");
    let git_dir: PathBuf = git_dir_with_work(root.as_path(), "repo.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    ClaimWatch::restored(store.as_path(), None).remember(git_dir.as_path());
    let refuse = |_: &Path| -> std::io::Result<Vec<u8>> { return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied)); };
    let mut watch: ClaimWatch = ClaimWatch::restored_reading(store.as_path(), None, &refuse);
    assert!(watch.directories().is_empty());
    assert!(watch.needs_retry(), "a refused read is tried again");
    assert_eq!(files_of(store.as_path()).len(), 1, "the note stays");
    assert!(watch.retry_reading(&refuse).is_empty());
    assert!(watch.needs_retry());
    assert_eq!(watch.retry(), vec![git_dir.clone()], "the read that works hands the directory over");
    assert!(!watch.needs_retry());
    assert_eq!(watch.directories(), vec![git_dir]);
    let _removed = std::fs::remove_dir_all(&root);
}

/// A store that cannot be listed is tried again;
/// one that does not exist is a journal that never held a note.
#[test]
fn a_store_that_cannot_be_listed_is_tried_again() {
    let root: PathBuf = scratch("unlistable");
    let git_dir: PathBuf = git_dir_with_work(root.as_path(), "repo.git");
    let blocker: PathBuf = root.join("blocker");
    std::fs::write(&blocker, "not a directory\n").expect("the blocker");
    let mut watch: ClaimWatch = ClaimWatch::restored(blocker.join("store").as_path(), None);
    assert!(watch.needs_retry(), "a store under a file cannot be listed");
    assert!(!ClaimWatch::restored(root.join("never-written").as_path(), None).needs_retry(), "a store that is not there holds nothing");
    std::fs::remove_file(&blocker).expect("the fault is mended");
    std::fs::create_dir_all(blocker.join("store")).expect("the store");
    std::fs::write(blocker.join("store").join(watch_name(git_dir.as_path(), 0)), encode(git_dir.as_path())).expect("a note");
    assert_eq!(watch.retry(), vec![git_dir], "listed again, the note is found");
    assert!(!watch.needs_retry());
    let _removed = std::fs::remove_dir_all(&root);
}

/// A note that names a directory of another instance is not this instance's to look at:
/// it is removed,
/// and a directory of another
/// instance is not written down.
#[test]
fn a_directory_of_another_instance_is_neither_kept_nor_written() {
    let root: PathBuf = scratch("foreign");
    let mine: PathBuf = git_dir_with_work(root.as_path(), "mine.git");
    let other: PathBuf = git_dir_with_work(root.as_path(), "other.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let instance: String = instance_of(mine.as_path());
    let mut watch: ClaimWatch = ClaimWatch::restored(store.as_path(), Some(instance.as_str()));
    watch.remember(other.as_path());
    watch.remember(mine.as_path());
    assert_eq!(watch.directories(), vec![mine.clone()]);
    assert_eq!(files_of(store.as_path()).len(), 1, "only this instance's directory is written");
    // A note another instance's directory was put into (a copied journal) is removed at the start.
    std::fs::write(store.join("copied.watch"), encode(other.as_path())).expect("a copied note");
    let restored: ClaimWatch = ClaimWatch::restored(store.as_path(), Some(instance.as_str()));
    assert_eq!(restored.directories(), vec![mine]);
    assert_eq!(files_of(store.as_path()).len(), 1, "the copied note is gone");
    let _removed = std::fs::remove_dir_all(&root);
}

/// A path that was redirected through a link to another repository names another instance,
/// so the note that holds it is removed
/// and the other repository is not looked at.
#[test]
fn a_path_redirected_to_another_repository_is_not_followed() {
    let root: PathBuf = scratch("redirected");
    let mine: PathBuf = git_dir_with_work(root.as_path(), "mine.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let instance: String = instance_of(mine.as_path());
    ClaimWatch::restored(store.as_path(), Some(instance.as_str())).remember(mine.as_path());
    // The path is replaced by a link to another repository that has a work directory of its own.
    let other: PathBuf = git_dir_with_work(root.as_path(), "other.git");
    std::fs::remove_dir_all(&mine).expect("the repository is removed");
    std::os::unix::fs::symlink(&other, &mine).expect("the path now leads to another repository");
    let restored: ClaimWatch = ClaimWatch::restored(store.as_path(), Some(instance.as_str()));
    assert!(restored.directories().is_empty(), "the other repository is not this instance's");
    assert!(files_of(store.as_path()).is_empty());
    let _removed = std::fs::remove_dir_all(&root);
}

/// A path that is not text,
/// or that ends in a newline,
/// is kept exactly and found again.
#[test]
fn a_path_is_kept_byte_for_byte() {
    let root: PathBuf = scratch("bytes");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let odd: PathBuf = root.join(OsString::from_vec(vec![b'o', 0xff, b'd', b'\n']));
    std::fs::create_dir_all(odd.join(WORK_DIRECTORY)).expect("a directory with a name that is not text and ends in a newline");
    let plain: PathBuf = root.join("od");
    std::fs::create_dir_all(plain.join(WORK_DIRECTORY)).expect("a directory whose name a lossy copy of the odd one would spell");
    let mut watch: ClaimWatch = ClaimWatch::restored(store.as_path(), None);
    watch.remember(odd.as_path());
    watch.remember(plain.as_path());
    assert_eq!(files_of(store.as_path()).len(), 2, "two different paths are two notes");
    let mut restored: Vec<PathBuf> = ClaimWatch::restored(store.as_path(), None).directories();
    restored.sort();
    let mut expected: Vec<PathBuf> = vec![odd, plain];
    expected.sort();
    assert_eq!(restored, expected);
    let _removed = std::fs::remove_dir_all(&root);
}

/// A note that holds another directory's path under the name this directory would take is left alone and the next name is used;
/// a damaged note under the name is replaced.
#[test]
fn a_taken_name_is_checked_not_trusted() {
    let root: PathBuf = scratch("taken");
    let git_dir: PathBuf = git_dir_with_work(root.as_path(), "repo.git");
    let other: PathBuf = git_dir_with_work(root.as_path(), "other.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    std::fs::create_dir_all(&store).expect("the store");
    // Another directory's whole note sits under the first name of this one.
    std::fs::write(store.join(watch_name(git_dir.as_path(), 0)), encode(other.as_path())).expect("a note of another directory");
    let mut watch: ClaimWatch = ClaimWatch::restored(store.as_path(), None);
    watch.remember(git_dir.as_path());
    let mut restored: Vec<PathBuf> = ClaimWatch::restored(store.as_path(), None).directories();
    restored.sort();
    let mut expected: Vec<PathBuf> = vec![git_dir.clone(), other.clone()];
    expected.sort();
    assert_eq!(restored, expected, "both are kept");
    // A torn note under the first name of a third directory is replaced.
    let third: PathBuf = git_dir_with_work(root.as_path(), "third.git");
    let mut again: ClaimWatch = ClaimWatch::restored(store.as_path(), None);
    std::fs::write(store.join(watch_name(third.as_path(), 0)), "claim-watch-1 2f7468").expect("a note that was cut short, left after the scan");
    again.remember(third.as_path());
    assert!(ClaimWatch::restored(store.as_path(), None).directories().contains(&third), "the cut note was replaced");
    let _removed = std::fs::remove_dir_all(&root);
}

/// What:
///  A repository the landing wrote down is looked at by the service that starts after it,
/// and one of another instance is not.
/// Why:
///   This is the whole of the promise that a kill is followed by the work being run with no commit needed:
/// the
///  wrapper's wake is what would have written the note,
/// and a wrapper killed between promoting a push and that wake
///  leaves the note the landing wrote instead.
#[test]
fn a_note_written_by_the_landing_is_looked_at_by_the_next_service() {
    let root: PathBuf = scratch("noted-by-the-landing");
    let git_dir: PathBuf = git_dir_with_work(root.as_path(), "repo.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let instance: String = instance_of(git_dir.as_path());
    // The landing knows the repository and its instance; no wake has happened for it.
    note_for_the_next_service(store.as_path(), Some(instance.as_str()), git_dir.as_path());
    assert_eq!(ClaimWatch::restored(store.as_path(), Some(instance.as_str())).directories(), vec![git_dir.clone()], "the next service looks at it");
    // A note of another instance's repository is neither written nor read, so the file stays one.
    let other: PathBuf = git_dir_with_work(root.as_path(), "other.git");
    note_for_the_next_service(store.as_path(), Some(instance.as_str()), other.as_path());
    assert_eq!(files_of(store.as_path()).len(), 1, "another instance's repository is not written down");
    let _removed = std::fs::remove_dir_all(&root);
}
