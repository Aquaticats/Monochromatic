//! Controls for the capture store's files.

use super::*;
use crate::private_storage::take_directory_syncs;
use crate::test_support::{fixture, remove};

/// Sequence files hold canonical decimal safe integers and one newline.
#[test]
fn sequences_are_canonical_decimals() {
    let path: &Path = Path::new("/s");
    assert_eq!(parse_sequence(b"0\n", path), Ok(0));
    assert_eq!(parse_sequence(b"41\n", path), Ok(41));
    assert_eq!(
        parse_sequence(b"9007199254740991\n", path),
        Ok(MAX_SAFE_INTEGER)
    );
    let malformed: CaptureError =
        CaptureError(String::from("Capture sequence file is malformed: /s"));
    for bad in [
        &b"9007199254740992\n"[..],
        b"99999999999999999999\n",
        b"41",
        b"\n",
        b"041\n",
        b"+1\n",
        b"-1\n",
        b" 1\n",
        b"1 \n",
        b"1\n\n",
        b"1\r\n",
        b"",
        b"x\n",
    ] {
        assert_eq!(parse_sequence(bad, path), Err(malformed.clone()), "{bad:?}");
    }
    assert_eq!(
        malformed.to_string(),
        "Capture sequence file is malformed: /s"
    );
}

/// The next sequence is one past the file,
///  or 1 without one.
#[test]
fn next_sequences_follow_the_file() {
    let root: PathBuf = fixture("capture-store-sequence");
    assert_eq!(read_next_capture_sequence(root.as_path()), Ok(1));
    let store: PathBuf = ensure_capture_store(root.as_path()).expect("store");
    assert_eq!(store, root.join(CAPTURE_STORE_NAME));
    assert_eq!(capture_store_path(root.as_path()), store);
    assert_eq!(read_next_capture_sequence(root.as_path()), Ok(1));
    std::fs::write(store.join(SEQUENCE_FILENAME), b"4\n").expect("sequence");
    assert_eq!(read_next_capture_sequence(root.as_path()), Ok(5));
    std::fs::write(store.join(SEQUENCE_FILENAME), b"four\n").expect("malformed");
    assert!(read_next_capture_sequence(root.as_path()).is_err());
    std::fs::remove_file(store.join(SEQUENCE_FILENAME)).expect("remove");
    std::fs::create_dir(store.join(SEQUENCE_FILENAME)).expect("directory");
    assert!(
        read_next_capture_sequence(root.as_path())
            .expect_err("directory")
            .0
            .starts_with("reading ")
    );
    // A store path that is a file is unsafe.
    let other: PathBuf = root.join("other");
    std::fs::create_dir(&other).expect("other");
    std::fs::write(other.join(CAPTURE_STORE_NAME), b"").expect("file store");
    assert!(ensure_capture_store(other.as_path()).is_err());
    remove(root.as_path());
}

/// Store files are regular,
///  read without following links,
///  and decoded with replacements.
#[cfg(unix)]
#[test]
fn store_files_are_regular() {
    let root: PathBuf = fixture("capture-store-files");
    let file: PathBuf = root.join("record");
    assert_eq!(read_store_file(file.as_path()), Ok(None));
    std::fs::write(&file, b"a\xffb").expect("record");
    assert_eq!(
        read_store_file(file.as_path()),
        Ok(Some(String::from("a\u{fffd}b")))
    );
    let link: PathBuf = root.join("link");
    std::os::unix::fs::symlink(&file, &link).expect("link");
    assert_eq!(
        read_store_file(link.as_path()).expect_err("link").0,
        format!(
            "Capture store entry is not a regular file: {}",
            link.display()
        )
    );
    let large: PathBuf = root.join("large");
    std::fs::File::create(&large)
        .expect("large")
        .set_len(STORE_FILE_LIMIT + 1)
        .expect("sparse size");
    assert!(
        read_store_file(large.as_path())
            .expect_err("too large")
            .0
            .starts_with("Capture store entry is larger than ")
    );
    assert!(read_store_file(file.join("x").as_path()).is_ok());
    remove(root.as_path());
}

/// The worktree identity is trimmed,
///  absent before the first capture.
#[test]
fn worktree_identities_are_trimmed() {
    let root: PathBuf = fixture("capture-store-identity");
    assert_eq!(read_worktree_id(root.as_path()), Ok(None));
    let store: PathBuf = ensure_capture_store(root.as_path()).expect("store");
    std::fs::write(store.join(WORKTREE_ID_FILENAME), b" 0b6c \n").expect("identity");
    assert_eq!(
        read_worktree_id(root.as_path()),
        Ok(Some(String::from("0b6c")))
    );
    std::fs::remove_file(store.join(WORKTREE_ID_FILENAME)).expect("remove");
    std::fs::create_dir(store.join(WORKTREE_ID_FILENAME)).expect("directory");
    assert!(read_worktree_id(root.as_path()).is_err());
    remove(root.as_path());
}

/// Creating the store syncs the Git directory once, and finding the store syncs nothing.
#[test]
fn the_git_directory_is_synced_only_when_the_store_is_created() {
    let git_dir: PathBuf = fixture("capture-store-sync-on-create");
    take_directory_syncs();
    ensure_capture_store(git_dir.as_path()).expect("created");
    assert_eq!(take_directory_syncs(), vec![git_dir.clone()]);
    ensure_capture_store(git_dir.as_path()).expect("found");
    assert_eq!(take_directory_syncs(), Vec::<PathBuf>::new());
    remove(git_dir.as_path());
}
