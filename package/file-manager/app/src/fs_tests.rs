// Unit tests for `crate::fs` (directory reads + sort). Exempt from require-rustdoc/max-lines
// because the file name ends in `_tests.rs`.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::fs::{read_directory, sort_entries};
use crate::types::{EntryKind, FileEntry};

fn entry(name: &str, kind: EntryKind) -> FileEntry {
    return FileEntry {
        name: name.to_owned(),
        path: PathBuf::from(name),
        kind,
        size: None,
        modified: None,
    }
}

fn unique_tempdir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("fm-{tag}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    return dir
}

#[test]
fn sorts_directories_before_files_then_case_insensitive() {
    let mut entries = vec![
        entry("zebra.txt", EntryKind::File),
        entry("Alpha", EntryKind::Directory),
        entry("beta.txt", EntryKind::File),
        entry("Zulu", EntryKind::Directory),
        entry("Apple.txt", EntryKind::File),
    ];
    sort_entries(&mut entries);
    let names: Vec<&str> = entries.iter().map(|item| return item.name.as_str()).collect();
    assert_eq!(names, ["Alpha", "Zulu", "Apple.txt", "beta.txt", "zebra.txt"]);
}

#[test]
fn reads_directory_with_kinds_and_metadata() {
    let root = unique_tempdir("read");
    fs::create_dir(root.join("subdir")).expect("create subdir");
    fs::write(root.join("file.txt"), b"hello").expect("write file");

    let snapshot = read_directory(&root, 7).expect("read temp dir");
    assert_eq!(snapshot.generation, 7);
    assert_eq!(snapshot.entries.len(), 2);
    assert_eq!(snapshot.entries[0].name, "subdir");
    assert_eq!(snapshot.entries[0].kind, EntryKind::Directory);
    assert_eq!(snapshot.entries[1].name, "file.txt");
    assert_eq!(snapshot.entries[1].kind, EntryKind::File);
    assert_eq!(snapshot.entries[1].size, Some(5));
    assert!(snapshot.entries[1].modified.is_some());

    fs::remove_dir_all(&root).expect("cleanup temp dir");
}

#[test]
fn missing_directory_is_an_error() {
    let missing = std::env::temp_dir()
        .join(format!("fm-missing-{}", std::process::id()))
        .join("does-not-exist");
    assert!(read_directory(&missing, 0).is_err());
}
