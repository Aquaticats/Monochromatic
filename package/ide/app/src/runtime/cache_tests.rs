//! The parser cache on disposable directories:
//!  first use,
//!  reuse,
//!  damage,
//!  concurrency,
//!  permissions.

use super::unpack;
use std::{
    fs,
    io::Read,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
    sync::{Arc, Barrier},
    thread,
};

/// Bytes standing in for a parser library;
///  large enough that a torn write would be visible.
fn library() -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::with_capacity(1_048_576);
    for index in 0..1_048_576u32 {
        bytes.push((index % 251) as u8);
    }
    return bytes;
}

/// Names in `directory` other than the final library files.
fn leftovers(directory: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for entry in fs::read_dir(directory).expect("cache directory") {
        let name = entry
            .expect("entry")
            .file_name()
            .to_string_lossy()
            .into_owned();
        if !name.ends_with(".so") {
            found.push(name);
        }
    }
    return found;
}

#[test]
fn first_use_writes_the_library_privately_and_later_uses_reuse_it() {
    let base = tempfile::tempdir().expect("disposable cache");
    let directory = base.path().join("runtime/key/grammars");
    let bytes = library();
    let path = unpack(&directory, "sql", &bytes).expect("first use");
    assert_eq!(path, directory.join("sql.so"));
    assert_eq!(fs::read(&path).expect("cached library"), bytes);
    let mode = fs::metadata(&path).expect("metadata").permissions().mode();
    assert_eq!(
        mode & 0o777,
        0o600,
        "the library is readable by the user only"
    );
    let directory_mode = fs::metadata(&directory)
        .expect("metadata")
        .permissions()
        .mode();
    assert_eq!(
        directory_mode & 0o777,
        0o700,
        "the cache directory is the user's only"
    );
    let modified = fs::metadata(&path)
        .expect("metadata")
        .modified()
        .expect("time");
    let again = unpack(&directory, "sql", &bytes).expect("second use");
    assert_eq!(again, path);
    assert_eq!(
        fs::metadata(&path)
            .expect("metadata")
            .modified()
            .expect("time"),
        modified,
        "an intact cached library was written again"
    );
    assert!(
        leftovers(&directory).is_empty(),
        "{:?}",
        leftovers(&directory)
    );
}

#[test]
fn a_damaged_or_shortened_cached_library_is_written_again() {
    let base = tempfile::tempdir().expect("disposable cache");
    let directory = base.path().join("grammars");
    let bytes = library();
    let path = unpack(&directory, "rust", &bytes).expect("first use");
    let mut damaged = bytes.clone();
    damaged[500_000] ^= 0x01;
    fs::write(&path, &damaged).expect("damage one bit");
    unpack(&directory, "rust", &bytes).expect("damaged cache");
    assert_eq!(
        fs::read(&path).expect("rewritten"),
        bytes,
        "one changed bit was not repaired"
    );
    fs::write(&path, &bytes[..1000]).expect("shorten");
    unpack(&directory, "rust", &bytes).expect("shortened cache");
    assert_eq!(
        fs::read(&path).expect("rewritten"),
        bytes,
        "a shortened file was not repaired"
    );
    fs::remove_file(&path).expect("remove");
    unpack(&directory, "rust", &bytes).expect("removed cache");
    assert_eq!(
        fs::read(&path).expect("rewritten"),
        bytes,
        "a removed file was not restored"
    );
    assert!(
        leftovers(&directory).is_empty(),
        "{:?}",
        leftovers(&directory)
    );
}

/// A rewrite puts a new file in place of the old one instead of writing into it:
///  a process that already
/// opened (or loaded) the old file keeps reading it whole,
///  and a reader of the name sees either file whole.
/// This is what makes concurrent first starts safe;
///  a write into the file in place would fail it.
#[test]
fn a_rewrite_replaces_the_file_instead_of_writing_into_it() {
    let base = tempfile::tempdir().expect("disposable cache");
    let directory = base.path().join("grammars");
    let bytes = library();
    let path = unpack(&directory, "toml", &bytes).expect("first use");
    let shortened = bytes[..4096].to_vec();
    fs::write(&path, &shortened).expect("shorten the cached library");
    let mut held = fs::File::open(&path).expect("open the shortened library");
    let inode = fs::metadata(&path).expect("metadata").ino();
    unpack(&directory, "toml", &bytes).expect("rewrite");
    let mut seen = Vec::new();
    held.read_to_end(&mut seen)
        .expect("read through the earlier handle");
    assert_eq!(
        seen, shortened,
        "the rewrite changed the file an earlier reader holds"
    );
    assert_ne!(
        fs::metadata(&path).expect("metadata").ino(),
        inode,
        "the name still points at the old file"
    );
    assert_eq!(fs::read(&path).expect("rewritten"), bytes);
}

/// Several first starts at once (threads stand in for processes;
///  the rename that makes a write
/// visible is atomic for both) all get a complete library,
///  and no partial file stays behind.
#[test]
fn concurrent_first_uses_all_get_the_complete_library() {
    let base = tempfile::tempdir().expect("disposable cache");
    let directory = Arc::new(base.path().join("grammars"));
    let bytes = Arc::new(library());
    let count = 8;
    let barrier = Arc::new(Barrier::new(count));
    let mut handles = Vec::new();
    for _ in 0..count {
        let shared_directory = Arc::clone(&directory);
        let shared_bytes = Arc::clone(&bytes);
        let start = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            start.wait();
            let path = unpack(&shared_directory, "typescript", &shared_bytes)
                .expect("concurrent first use");
            // Every caller reads back what it is about to load: always the whole library.
            return fs::read(path).expect("cached library") == *shared_bytes;
        }));
    }
    for handle in handles {
        assert!(
            handle.join().expect("thread"),
            "a caller saw an incomplete library"
        );
    }
    assert!(
        leftovers(&directory).is_empty(),
        "{:?}",
        leftovers(&directory)
    );
}

#[test]
fn an_unwritable_cache_names_the_directory_and_the_remedy() {
    let base = tempfile::tempdir().expect("disposable cache");
    // A file where the cache directory should be makes every write fail, whoever runs the test.
    let blocked = base.path().join("blocked");
    fs::write(&blocked, "not a directory").expect("blocking file");
    let error = unpack(&blocked.join("grammars"), "sql", &library()).expect_err("blocked cache");
    let message = format!("{error:#}");
    assert!(
        message.contains("Cannot unpack the language parser sql"),
        "{message}"
    );
    assert!(
        message.contains(&blocked.join("grammars").display().to_string()),
        "{message}"
    );
    assert!(message.contains("XDG_CACHE_HOME"), "{message}");
}
