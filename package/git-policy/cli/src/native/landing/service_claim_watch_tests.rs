//! Controls for the Git directories a restarted service looks at by itself.

use super::*;

/// A scratch directory under the package's own build directory, removed by the caller.
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

/// A directory written down by one watch is read back by the watch of the service that starts next.
#[test]
fn a_remembered_directory_is_restored_by_the_next_watch() {
    let root: PathBuf = scratch("restored");
    let git_dir: PathBuf = git_dir_with_work(root.as_path(), "repo.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let mut first: ClaimWatch = ClaimWatch::restored(store.as_path());
    assert!(first.directories().is_empty(), "a new journal watches nothing");
    first.remember(git_dir.as_path());
    first.remember(git_dir.as_path());
    assert_eq!(first.directories(), vec![git_dir.clone()], "remembered once");
    assert_eq!(std::fs::read_dir(&store).expect("the store").count(), 1, "one file for one directory");
    assert_eq!(ClaimWatch::restored(store.as_path()).directories(), vec![git_dir]);
    let _removed = std::fs::remove_dir_all(&root);
}

/// Several directories are each written down and each restored.
#[test]
fn every_directory_has_its_own_entry() {
    let root: PathBuf = scratch("several");
    let one: PathBuf = git_dir_with_work(root.as_path(), "one.git");
    let two: PathBuf = git_dir_with_work(root.as_path(), "two.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let mut watch: ClaimWatch = ClaimWatch::restored(store.as_path());
    watch.remember(one.as_path());
    watch.remember(two.as_path());
    let mut restored: Vec<PathBuf> = ClaimWatch::restored(store.as_path()).directories();
    restored.sort();
    assert_eq!(restored, vec![one, two]);
    let _removed = std::fs::remove_dir_all(&root);
}

/// A directory that lost its work directory, and a file that names no absolute path, are dropped and their files removed, so a
/// removed worktree is not looked at at every start.
#[test]
fn a_directory_with_no_work_directory_and_a_file_with_no_path_are_dropped() {
    let root: PathBuf = scratch("dropped");
    let gone: PathBuf = git_dir_with_work(root.as_path(), "gone.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let mut watch: ClaimWatch = ClaimWatch::restored(store.as_path());
    watch.remember(gone.as_path());
    std::fs::write(store.join("junk.watch"), "relative/path\n").expect("a file that names no absolute path");
    std::fs::write(store.join("other.txt"), "left alone\n").expect("a file of another kind");
    std::fs::remove_dir_all(&gone).expect("the worktree is removed");
    assert!(ClaimWatch::restored(store.as_path()).directories().is_empty());
    let mut left: Vec<String> = std::fs::read_dir(&store).expect("the store").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    left.sort();
    assert_eq!(left, vec![String::from("other.txt")], "only the file of another kind stays");
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
    assert!(!root.join(WATCH_DIRECTORY).exists());
    let _removed = std::fs::remove_dir_all(&root);
}

/// A directory whose file cannot be written is not counted as kept, so the next wake tries again.
#[test]
fn a_directory_whose_file_cannot_be_written_is_tried_again_at_the_next_wake() {
    let root: PathBuf = scratch("unwritable");
    let git_dir: PathBuf = git_dir_with_work(root.as_path(), "repo.git");
    // A file where the store's directory should be makes the write fail.
    let blocker: PathBuf = root.join("blocker");
    std::fs::write(&blocker, "not a directory\n").expect("the blocker");
    let mut watch: ClaimWatch = ClaimWatch::restored(blocker.join("store").as_path());
    watch.remember(git_dir.as_path());
    assert!(watch.directories().is_empty(), "not written, so not known");
    let _removed = std::fs::remove_dir_all(&root);
}
