//! What:
//!  Controls for the descriptor-relative calls:
//!  a directory is opened by descriptor,
//!  an entry below it is created once and never twice,
//!  a link in an entry's place is refused,
//!  a directory is made and removed,
//!  a directory with a file in it is not removed,
//!  and a rename puts the whole file at the name it means.
//! Why:
//!  These are the calls the act and the progress file both make,
//!  and each control is one rule of the design's boundary:
//!  no name resolved after a proof,
//!  no second writer,
//!  no removed directory that still holds something.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a staging name is created once', () => { expect(() => createAt(dir, 'a', 0o600)).toThrow(); });
//! ```

use super::{
    create_at,
    make_directory_at,
    open_directory,
    open_directory_below,
    open_file_below,
    remove_directory_at,
    remove_entry_at,
    rename_at,
    sync_directory,
};
use crate::testing::test_support::fixture;
use std::io::Write;
use std::path::PathBuf;

#[test]
fn a_directory_is_opened_and_an_entry_below_it_is_read() {
    let folder: PathBuf = fixture("descriptor_calls_open");
    std::fs::write(folder.join("a-file"), b"the whole file\n").expect("the fixture file");
    let directory: std::fs::File =
        open_directory(folder.as_path()).expect("the fixture folder is opened by descriptor");
    let mut entry: std::fs::File =
        open_file_below(&directory, "a-file").expect("the entry below it is opened");
    let mut text: String = String::new();
    std::io::Read::read_to_string(&mut entry, &mut text).expect("the entry is read");
    assert_eq!(text, "the whole file\n");
    assert!(
        open_directory_below(&directory, "a-file").is_err(),
        "a plain file is not a directory and the open says so"
    );
    assert!(
        open_file_below(&directory, "absent").is_err(),
        "an absent entry is refused"
    );
}

#[test]
fn a_link_in_an_entry_s_place_is_refused() {
    let folder: PathBuf = fixture("descriptor_calls_link");
    std::fs::write(folder.join("real"), b"x\n").expect("the fixture file");
    std::os::unix::fs::symlink("real", folder.join("link")).expect("the fixture link");
    let directory: std::fs::File =
        open_directory(folder.as_path()).expect("the fixture folder is opened");
    let refused =
        open_file_below(&directory, "link").expect_err("an entry that is a link is refused");
    assert_eq!(refused.raw_os_error(), Some(libc::ELOOP));
}

#[test]
fn a_staging_name_is_created_once_and_never_over_another() {
    let folder: PathBuf = fixture("descriptor_calls_create");
    let directory: std::fs::File =
        open_directory(folder.as_path()).expect("the fixture folder is opened");
    let mut first: std::fs::File =
        create_at(&directory, "a-name.staging", 0o600).expect("the name is free");
    first
        .write_all(b"one whole file\n")
        .expect("the staging write");
    first.sync_all().expect("the staging sync");
    drop(first);
    let refused = create_at(&directory, "a-name.staging", 0o600)
        .expect_err("a name another writer holds is never truncated");
    assert_eq!(refused.raw_os_error(), Some(libc::EEXIST));
    use std::os::unix::fs::MetadataExt;
    let facts: std::fs::Metadata =
        std::fs::metadata(folder.join("a-name.staging")).expect("the file's own facts");
    assert_eq!(
        facts.mode() & 0o7777,
        0o600,
        "the name carries the mode the caller asked for"
    );
}

#[test]
fn a_rename_puts_the_whole_file_at_the_name_it_means() {
    let folder: PathBuf = fixture("descriptor_calls_rename");
    let directory: std::fs::File =
        open_directory(folder.as_path()).expect("the fixture folder is opened");
    let mut staged: std::fs::File =
        create_at(&directory, "a-name.staging", 0o600).expect("the staging name is free");
    staged
        .write_all(b"the whole answer\n")
        .expect("the staging write");
    staged.sync_all().expect("the staging sync");
    drop(staged);
    rename_at(&directory, "a-name.staging", "a-name")
        .expect("the rename is one step in one directory");
    sync_directory(&directory).expect("the directory is synced");
    assert_eq!(
        std::fs::read_to_string(folder.join("a-name")).expect("the file is read back"),
        "the whole answer\n"
    );
    assert!(
        !folder.join("a-name.staging").exists(),
        "the staging name is gone after the rename"
    );
}

#[test]
fn a_directory_is_made_empty_and_removed_when_it_is_still_empty() {
    let folder: PathBuf = fixture("descriptor_calls_directory");
    let directory: std::fs::File =
        open_directory(folder.as_path()).expect("the fixture folder is opened");
    make_directory_at(&directory, ".git-worktree-clone-removal-1", 0o700)
        .expect("the reservation is free");
    use std::os::unix::fs::MetadataExt;
    let facts: std::fs::Metadata = std::fs::metadata(folder.join(".git-worktree-clone-removal-1"))
        .expect("the folder's own facts");
    assert_eq!(facts.mode() & 0o7777, 0o700);
    assert!(facts.is_dir());
    let inside: std::fs::File = open_directory_below(&directory, ".git-worktree-clone-removal-1")
        .expect("the folder is opened below the checkout");
    let mut file: std::fs::File = create_at(&inside, "a-file", 0o600).expect("a file inside it");
    file.write_all(b"x").expect("the write");
    drop(file);
    drop(inside);
    let refused = remove_directory_at(&directory, ".git-worktree-clone-removal-1")
        .expect_err("a directory something filled is not removed");
    assert_eq!(refused.raw_os_error(), Some(libc::ENOTEMPTY));
    let absent: std::io::Error = remove_entry_at(&directory, "a-file-that-is-not-there")
        .expect_err("an entry that is not there is refused");
    assert_eq!(absent.raw_os_error(), Some(libc::ENOENT));
}

#[test]
fn an_empty_directory_is_removed_by_its_name() {
    let folder: PathBuf = fixture("descriptor_calls_remove_empty");
    let directory: std::fs::File =
        open_directory(folder.as_path()).expect("the fixture folder is opened");
    make_directory_at(&directory, "reserved", 0o700).expect("the reservation is free");
    let inside: std::fs::File =
        open_directory_below(&directory, "reserved").expect("the folder is opened");
    let mut file: std::fs::File = create_at(&inside, "a-file", 0o600).expect("a file inside it");
    file.write_all(b"x").expect("the write");
    drop(file);
    drop(inside);
    remove_entry_at(&directory, "a-file")
        .expect_err("a file of another directory is not one of this directory's entries");
    std::fs::remove_file(folder.join("reserved").join("a-file"))
        .expect("the file the control made is removed by its own path");
    remove_directory_at(&directory, "reserved").expect("an empty directory is removed by its name");
    assert!(!folder.join("reserved").exists());
}
