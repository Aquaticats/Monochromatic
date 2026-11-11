//! Tests of the entries a snapshot an add installed is behind on,
//! and of the line that says so.

use super::*;

/// A repository with one ignored file that is older than the instant a control passes,
/// and one written after it.
fn source_with_an_entry_written_after(name: &str) -> (PathBuf, PathBuf, SnapshotSource, u128) {
    let root: PathBuf = crate::test_support::fixture(name);
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    std::fs::write(repo.join(".gitignore"), b"ignored/\n").expect("gitignore");
    std::fs::create_dir_all(repo.join("ignored")).expect("the ignored directory");
    std::fs::write(repo.join("ignored/older.txt"), b"the copy holds this one").expect("the older entry");
    crate::test_support::git(repo.as_path(), &["add", "-A"]);
    crate::test_support::git(repo.as_path(), &["commit", "--quiet", "-m", "init"]);
    // A change time and the instant a control names must be tellable apart, and the filesystem stamps whole seconds.
    std::thread::sleep(std::time::Duration::from_millis(1200));
    let instant: u128 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis();
    std::thread::sleep(std::time::Duration::from_millis(20));
    std::fs::write(repo.join("ignored/newer.txt"), b"written after the copy").expect("the newer entry");
    let source: SnapshotSource = crate::worktree_snapshot_source::source_or_dirty(&crate::test_support::git_context(), &crate::worktree_ladder_test_support::main_identity(repo.as_path())).expect("a clean source");
    return (root, repo, source, instant);
}

/// A file written after the copy is named and one written before it is not.
#[test]
fn the_entry_written_after_the_copy_is_named_and_the_older_one_is_not() {
    let (root, repo, source, instant) = source_with_an_entry_written_after("after-the-copy-newer");
    let newer: Vec<Vec<u8>> = entries_newer_than_the_copy(&crate::test_support::git_context(), &source, root.join("a-worktree-this-add-made").as_path(), instant).expect("the walk");
    assert!(
        newer.iter().any(|path: &Vec<u8>| return path.as_slice() == b"ignored/newer.txt"),
        "the entry written after the copy is named: {:?}",
        newer.iter().map(|path| return String::from_utf8_lossy(path).into_owned()).collect::<Vec<String>>()
    );
    assert!(!newer.iter().any(|path: &Vec<u8>| return path.as_slice() == b"ignored/older.txt"), "the entry the copy holds is not named: {newer:?}");
    assert!(
        !newer.iter().any(|path: &Vec<u8>| return path.as_slice() == b"ignored"),
        "the directory is not named: it holds no bytes of its own, and the files below it that changed are: {newer:?}"
    );
    std::fs::remove_dir_all(repo.as_path()).expect("the fixture's repository");
    crate::test_support::remove(root.as_path());
}

/// A copy newer than every entry is behind on nothing,
/// so nothing is said.
#[test]
fn a_copy_newer_than_every_entry_is_behind_on_nothing() {
    let (root, repo, source, _instant) = source_with_an_entry_written_after("after-the-copy-fresh");
    let now: u128 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis();
    let newer: Vec<Vec<u8>> = entries_newer_than_the_copy(&crate::test_support::git_context(), &source, root.join("a-worktree-this-add-made").as_path(), now).expect("the walk");
    assert!(newer.is_empty(), "nothing changed after a copy taken now: {newer:?}");
    std::fs::remove_dir_all(repo.as_path()).expect("the fixture's repository");
    crate::test_support::remove(root.as_path());
}

/// The age reads as whole seconds below a minute and as minutes above it,
/// and a copy made while the add ran says so.
#[test]
fn the_age_reads_in_the_words_a_person_uses() {
    assert_eq!(age_words(0), "less than a second");
    assert_eq!(age_words(1), "1 second");
    assert_eq!(age_words(59), "59 seconds");
    assert_eq!(age_words(60), "1 minute");
    assert_eq!(age_words(124), "2 minutes and 4 seconds");
    assert_eq!(age_words(61), "1 minute and 1 second");
}

/// The line names the source,
/// the destination,
/// the age,
/// the first paths,
/// the count of the rest,
/// and the file that holds every one of them.
#[test]
fn the_line_names_the_copy_its_age_the_first_paths_and_the_file_of_all_of_them() {
    let source: PathBuf = PathBuf::from("/a/source");
    let destination: PathBuf = PathBuf::from("/a/worktree");
    let list: PathBuf = PathBuf::from("/a/worktree/.git/ignored-state.txt");
    let newer: Vec<Vec<u8>> = (0..5).map(|index| return format!("node_modules/pkg{index}/index.js").into_bytes()).collect();
    let line: String = notice_line(source.as_path(), destination.as_path(), "37 seconds", newer.as_slice(), Some(list.as_path()));
    assert!(line.contains("\"/a/worktree\""), "{line}");
    assert!(line.contains("\"/a/source\""), "{line}");
    assert!(line.contains("37 seconds"), "{line}");
    assert!(line.contains("\"node_modules/pkg0/index.js\""), "{line}");
    assert!(line.contains("and 2 more"), "{line}");
    assert!(line.contains("Every path is on its own line of \"/a/worktree/.git/ignored-state.txt\""), "{line}");
    assert!(line.ends_with("makes them again here.\n"), "{line}");
}

/// The worktree the add just made is not part of the source's ignored state,
/// so none of its files is named even though every one of them was written after the copy.
#[test]
fn the_worktree_this_add_made_is_not_named() {
    let (root, repo, source, instant) = source_with_an_entry_written_after("after-the-copy-destination");
    let destination: PathBuf = repo.join("ignored/made-by-this-add");
    std::fs::create_dir_all(destination.as_path()).expect("the worktree this add made");
    std::fs::write(destination.join("a.txt"), b"written by this add").expect("a file of the new worktree");
    let newer: Vec<Vec<u8>> = entries_newer_than_the_copy(&crate::test_support::git_context(), &source, destination.as_path(), instant).expect("the walk");
    assert!(
        !newer.iter().any(|path: &Vec<u8>| return path.starts_with(b"ignored/made-by-this-add")),
        "no file of the worktree this add made is named: {:?}",
        newer.iter().map(|path| return String::from_utf8_lossy(path).into_owned()).collect::<Vec<String>>()
    );
    assert!(
        newer.iter().any(|path: &Vec<u8>| return path.as_slice() == b"ignored/newer.txt"),
        "the entry the source wrote after the copy is still named: {newer:?}"
    );
    std::fs::remove_dir_all(repo.as_path()).expect("the fixture's repository");
    crate::test_support::remove(root.as_path());
}
