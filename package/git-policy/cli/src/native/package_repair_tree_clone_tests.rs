//! The tree clone: contents, modes, times, links kept verbatim, an existing destination refused, parallel and serial agree.

use super::*;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::path::PathBuf;

/// More files than any single worker slice, so the parallel path really splits the work.
const MANY_FILES: usize = 64;

/// A source with nested directories, an executable, a link with a relative target, and an old modification time.
fn build_source(root: &Path) -> PathBuf {
    let source: PathBuf = root.join("source");
    std::fs::create_dir_all(source.join("deps/nested")).expect("directories");
    std::fs::write(source.join("deps/nested/lib.rlib"), b"library bytes").expect("file");
    std::fs::write(source.join("tool"), b"binary bytes").expect("tool");
    std::fs::set_permissions(source.join("tool"), std::fs::Permissions::from_mode(0o750)).expect("mode");
    symlink("../outside/target", source.join("deps/link")).expect("link");
    for index in 0..MANY_FILES {
        std::fs::write(source.join(format!("deps/file-{index}")), format!("{index}")).expect("many");
    }
    let old: std::time::SystemTime = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000_000);
    let file: std::fs::File = std::fs::File::options().write(true).open(source.join("deps/nested/lib.rlib")).expect("open for times");
    file.set_modified(old).expect("set modified");
    return source;
}

/// Contents, modes and modification times are the source's; a link keeps its exact target text.
#[test]
fn clones_contents_modes_times_and_links() {
    let root: PathBuf = fixture("tree-clone-faithful");
    let source: PathBuf = build_source(&root);
    let destination: PathBuf = root.join("destination");
    let made: usize = clone_tree(&source, &destination, 1).expect("clone");
    assert!(made > MANY_FILES, "made {made}");
    assert_eq!(std::fs::read(destination.join("deps/nested/lib.rlib")).expect("lib"), b"library bytes");
    let tool = std::fs::metadata(destination.join("tool")).expect("tool metadata");
    assert_eq!(tool.permissions().mode() & 0o777, 0o750);
    let before = std::fs::metadata(source.join("deps/nested/lib.rlib")).expect("source");
    let after = std::fs::metadata(destination.join("deps/nested/lib.rlib")).expect("destination");
    assert_eq!((after.mtime(), after.mtime_nsec()), (before.mtime(), before.mtime_nsec()));
    assert_eq!(std::fs::read_link(destination.join("deps/link")).expect("link"), PathBuf::from("../outside/target"));
    remove(&root);
}

/// The clone shares no hard link with the source: changing the clone leaves the source alone.
#[test]
fn a_changed_clone_leaves_the_source_alone() {
    let root: PathBuf = fixture("tree-clone-independent");
    let source: PathBuf = build_source(&root);
    let destination: PathBuf = root.join("destination");
    clone_tree(&source, &destination, 1).expect("clone");
    std::fs::write(destination.join("tool"), b"changed in the clone").expect("write clone");
    assert_eq!(std::fs::read(source.join("tool")).expect("source tool"), b"binary bytes");
    remove(&root);
}

/// An existing destination is refused and left as it was.
#[test]
fn refuses_an_existing_destination() {
    let root: PathBuf = fixture("tree-clone-existing");
    let source: PathBuf = build_source(&root);
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir_all(&destination).expect("destination");
    std::fs::write(destination.join("keep"), b"kept").expect("keep");
    assert!(clone_tree(&source, &destination, 1).is_err());
    assert_eq!(std::fs::read(destination.join("keep")).expect("kept"), b"kept");
    assert!(!destination.join("tool").exists());
    remove(&root);
}

/// Several workers make the same tree as one worker.
#[test]
fn parallel_and_serial_clones_agree() {
    let root: PathBuf = fixture("tree-clone-parallel");
    let source: PathBuf = build_source(&root);
    let serial: PathBuf = root.join("serial");
    let parallel: PathBuf = root.join("parallel");
    let one: usize = clone_tree(&source, &serial, 1).expect("serial");
    let several: usize = clone_tree(&source, &parallel, 4).expect("parallel");
    assert_eq!(one, several);
    for index in 0..MANY_FILES {
        let name: String = format!("deps/file-{index}");
        assert_eq!(std::fs::read(parallel.join(&name)).expect("parallel file"), format!("{index}").into_bytes());
    }
    remove(&root);
}

/// A missing source is an error naming it.
#[test]
fn a_missing_source_is_an_error() {
    let root: PathBuf = fixture("tree-clone-missing");
    let error: String = clone_tree(root.join("absent").as_path(), root.join("destination").as_path(), 1).expect_err("missing source");
    assert!(error.contains("absent"), "{error}");
    remove(&root);
}
