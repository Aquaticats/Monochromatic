//! What:
//! Unit tests of the store's version directories in `store_versions.rs`:
//! which directories count as versions,
//! the size and the write time each is judged by,
//! the download files an earlier layout left,
//! and the removal markers a cut-short removal leaves.
//! Why:
//! The cleanup removes whatever this scan lists and nothing else,
//! so a directory it must never list (a staging directory, a `current` link, a version of a tool no lock names)
//! and one it must list (an old version nobody pins) are each a test.

/// Imports `BTreeSet` from `std::collections`.
use std::collections::BTreeSet;
/// Imports `fs` from `std`.
use std::fs;
/// Imports `FileTimes` from `std::fs`,
/// which sets a fixture's write time.
use std::fs::FileTimes;
/// Imports what these tests use from `std::path`.
use std::path::{Path, PathBuf};
/// Imports `Duration` and `SystemTime` from `std::time`.
use std::time::{Duration, SystemTime};

/// Imports the module under test.
use crate::toolchain::store_versions::{
    DOWNLOADS_DIRECTORY_NAME, SIZE_RECORDS_FILE_NAME, TOMBSTONE_PREFIX, install_directory_name, leftover_downloads,
    read_size_records, removal_markers, tombstone_of, tree_size, version_directories,
};
use crate::toolchain::model::Route;
use crate::test_scratch::scratch;

/// The names of the versions the scan lists for `tools`.
fn versions_of(root: &Path, tools: &[&str]) -> Vec<String> {
    let set: BTreeSet<String> = tools.iter().map(|tool| return (*tool).to_string()).collect();
    return version_directories(root, &set)
        .into_iter()
        .map(|version| return format!("{}-{}", version.tool, version.version))
        .collect();
}

/// Makes `<root>/<tool>/<version>` with one file of `bytes` bytes inside it.
fn make_version(root: &Path, tool: &str, version: &str, bytes: usize) -> PathBuf {
    let directory: PathBuf = root.join(tool).join(version);
    fs::create_dir_all(&directory).expect("make the version directory");
    fs::write(directory.join("program"), vec![b'a'; bytes]).expect("write the program");
    return directory;
}

/// Sets the write time of `path` (a file or a directory) to `seconds` before now.
fn written_ago(path: &Path, seconds: u64) {
    let when: SystemTime = SystemTime::now() - Duration::from_secs(seconds);
    let file = fs::File::open(path).expect("open the fixture for its times");
    file.set_times(FileTimes::new().set_modified(when).set_accessed(when)).expect("set the fixture's times");
}

/// Every route that keeps one directory per version names it after the tool's id,
/// and `cargo binstall` names it after the crate.
#[test]
fn names_the_directory_of_each_route() {
    assert_eq!(install_directory_name(Route::Archive, "node"), Some("node".to_string()));
    assert_eq!(install_directory_name(Route::Archive, "conda:bubblewrap"), Some("conda-bubblewrap".to_string()));
    assert_eq!(install_directory_name(Route::CargoGit, "cargo:below"), Some("cargo-below".to_string()));
    assert_eq!(install_directory_name(Route::CargoBinstall, "cargo:fd-find"), Some("cargo/fd-find".to_string()));
    assert_eq!(install_directory_name(Route::CargoBinstall, "fd-find"), Some("cargo/fd-find".to_string()));
    assert_eq!(install_directory_name(Route::GoInstall, "go:github.com/charmbracelet/crush"), Some("go-github.com-charmbracelet-crush".to_string()));
    assert_eq!(install_directory_name(Route::MpmPnpm, "npm:typescript"), None);
    assert_eq!(install_directory_name(Route::BinSourced, "rg"), None);
}

/// The scan lists the version directories of the tools it is asked about,
/// skips a hidden staging directory and the `current` link,
/// and counts each version's downloads with it.
#[test]
fn lists_versions_with_their_downloads_and_skips_staging() {
    let directory = scratch();
    let root: PathBuf = directory.path().join("toolchain");
    make_version(&root, "node", "26.12.0", 100);
    make_version(&root, "node", "26.13.1", 200);
    fs::create_dir_all(root.join("node/.26.12.1.staging.42")).expect("make a staging directory");
    std::os::unix::fs::symlink("26.13.1", root.join("node/current")).expect("make the current link");
    let downloads: PathBuf = root.join(DOWNLOADS_DIRECTORY_NAME).join("node").join("26.12.0");
    fs::create_dir_all(&downloads).expect("make the downloads directory");
    fs::write(downloads.join("node.tar.gz"), vec![b'z'; 50]).expect("write the archive");

    let found = version_directories(&root, &["node".to_string()].into_iter().collect());

    let listed: Vec<String> = found
        .iter()
        .map(|version| return format!("{}-{}-{}-{}", version.tool, version.version, version.bytes, version.downloads.is_some()))
        .collect();
    assert_eq!(listed, vec!["node-26.12.0-150-true", "node-26.13.1-200-false"]);
}

/// A tool whose directory the scan was not asked about is not listed,
/// so no version outside the locks' reach is ever removed.
#[test]
fn never_lists_a_tool_it_was_not_asked_about() {
    let directory = scratch();
    let root: PathBuf = directory.path().join("toolchain");
    make_version(&root, "node", "26.12.0", 100);
    make_version(&root, "aws", "2.40.2", 100);

    assert_eq!(versions_of(&root, &["node"]), vec!["node-26.12.0"]);
}

/// The listing is ordered by write time, oldest first,
/// because the rule removes in that order.
#[test]
fn orders_versions_by_write_time_oldest_first() {
    let directory = scratch();
    let root: PathBuf = directory.path().join("toolchain");
    let recent: PathBuf = make_version(&root, "node", "26.13.1", 10);
    let old: PathBuf = make_version(&root, "node", "26.12.0", 10);
    written_ago(&recent, 60);
    written_ago(&old, 60 * 60 * 24);

    assert_eq!(versions_of(&root, &["node"]), vec!["node-26.12.0", "node-26.13.1"]);
}

/// A tool's directory that is missing is not an error:
/// nothing is installed under it.
#[test]
fn lists_nothing_for_a_tool_that_is_not_installed() {
    let directory = scratch();
    assert!(versions_of(directory.path(), &["node"]).is_empty());
}

/// The download files an earlier layout left are listed for the tool their name begins with,
/// and a download still being written is not.
#[test]
fn lists_leftover_downloads_but_not_one_in_flight() {
    let directory = scratch();
    let root: PathBuf = directory.path().join("toolchain");
    let downloads: PathBuf = root.join(DOWNLOADS_DIRECTORY_NAME);
    fs::create_dir_all(&downloads).expect("make the downloads directory");
    fs::write(downloads.join("node-26.12.0-node-v26.12.0-linux-x64.tar.gz"), vec![b'n'; 40]).expect("write a leftover");
    fs::write(downloads.join("node-26.13.1-node-v26.13.1-linux-x64.tar.gz.download"), vec![b'n'; 10])
        .expect("write a download in flight");
    fs::write(downloads.join("something-else.tar.gz"), vec![b'x'; 10]).expect("write a file of nothing known");

    let found = leftover_downloads(&root, &["node".to_string()].into_iter().collect());

    let listed: Vec<String> = found
        .iter()
        .map(|leftover| return format!("{}-{}-{}", leftover.tool, leftover.bytes, leftover.file.file_name().unwrap().to_string_lossy()))
        .collect();
    assert_eq!(listed, vec!["node-40-node-26.12.0-node-v26.12.0-linux-x64.tar.gz"]);
}

/// A version moved out of its name while it is removed is listed as a removal marker and never as a version,
/// so a later pass cannot take a half version for a whole one.
#[test]
fn a_version_being_removed_is_a_marker_and_not_a_version() {
    let directory = scratch();
    let root: PathBuf = directory.path().join("toolchain");
    let version: PathBuf = make_version(&root, "node", "26.12.0", 100);
    let marker: PathBuf = tombstone_of(&version);
    fs::rename(&version, &marker).expect("move the version aside");

    assert!(versions_of(&root, &["node"]).is_empty());
    assert!(marker.file_name().unwrap().to_string_lossy().starts_with(TOMBSTONE_PREFIX));
    // The marker is found from the store itself, so no list of tool names has to be right for it to be found.
    let markers = removal_markers(&root);
    assert_eq!(markers.len(), 1, "{markers:?}");
    assert_eq!(markers[0].version_marker, marker);
    assert_eq!(markers[0].version, "26.12.0");
    assert_eq!(markers[0].version_directory, version);
    assert_eq!(markers[0].tool, "node");
}

/// A removal marker in a tool's downloads directory is found too,
/// so a removal cut short between the two moves is finished.
#[test]
fn finds_a_removal_marker_beside_the_downloads() {
    let directory = scratch();
    let root: PathBuf = directory.path().join("toolchain");
    let downloads: PathBuf = root.join(DOWNLOADS_DIRECTORY_NAME).join("node").join("26.12.0");
    fs::create_dir_all(&downloads).expect("make the downloads directory");
    let marker: PathBuf = tombstone_of(&downloads);
    fs::rename(&downloads, &marker).expect("move the downloads aside");

    let markers = removal_markers(&root);
    assert_eq!(markers.len(), 1, "{markers:?}");
    assert_eq!(markers[0].downloads_marker, Some(marker));
    assert_eq!(markers[0].downloads_directory, Some(downloads.clone()));
    assert_eq!(markers[0].version_directory, root.join("node").join("26.12.0"));
}

/// A version's size is recorded once,
/// and a version whose directory was written again is measured again.
#[test]
fn records_a_size_and_measures_again_once_the_version_is_written() {
    let directory = scratch();
    let root: PathBuf = directory.path().join("toolchain");
    let version: PathBuf = make_version(&root, "node", "26.13.1", 1000);
    let tools: BTreeSet<String> = ["node".to_string()].into_iter().collect();

    assert_eq!(version_directories(&root, &tools)[0].bytes, 1000);
    assert_eq!(read_size_records(&root).get("node/26.13.1").map(|record| return record.bytes), Some(1000));
    assert!(root.join(SIZE_RECORDS_FILE_NAME).is_file());

    // An install replaces a version as one rename, so the next entry changes the directory's own write time.
    fs::write(version.join("extra"), vec![b'e'; 24]).expect("add a file to the version");
    assert_eq!(version_directories(&root, &tools)[0].bytes, 1024);
}

/// A version whose directory is unchanged keeps its recorded size even when a file inside it was rewritten in place,
/// which is the one change a directory's write time does not show.
#[test]
fn trusts_a_record_while_the_version_directory_is_unchanged() {
    let directory = scratch();
    let root: PathBuf = directory.path().join("toolchain");
    let version: PathBuf = make_version(&root, "node", "26.13.1", 1000);
    let tools: BTreeSet<String> = ["node".to_string()].into_iter().collect();
    assert_eq!(version_directories(&root, &tools)[0].bytes, 1000);

    fs::write(version.join("program"), vec![b'b'; 10]).expect("rewrite the program in place");

    assert_eq!(version_directories(&root, &tools)[0].bytes, 1000);
}

/// A size record file that is not the document this release writes reads as no records,
/// so the store is measured again.
#[test]
fn measures_everything_when_the_size_records_are_not_readable() {
    let directory = scratch();
    let root: PathBuf = directory.path().join("toolchain");
    make_version(&root, "node", "26.13.1", 1000);
    fs::write(root.join(SIZE_RECORDS_FILE_NAME), b"not the document").expect("write a broken record file");
    let tools: BTreeSet<String> = ["node".to_string()].into_iter().collect();

    assert!(read_size_records(&root).is_empty());
    assert_eq!(version_directories(&root, &tools)[0].bytes, 1000);
}

/// The size of a version is the length of every file below it.
#[test]
fn measures_the_length_of_every_file_below() {
    let directory = scratch();
    let root: PathBuf = directory.path().join("toolchain");
    let version: PathBuf = make_version(&root, "node", "26.13.1", 1000);
    fs::create_dir_all(version.join("lib/node_modules")).expect("make a nested directory");
    fs::write(version.join("lib/node_modules/index.js"), vec![b'j'; 24]).expect("write a nested file");

    assert_eq!(tree_size(&version), 1024);
}
