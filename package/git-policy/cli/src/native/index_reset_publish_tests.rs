//! Controls for the temporary file, the file mode and the rename of the landing's private index, with the file system steps made
//! to fail or run short.

use super::*;
use crate::test_support::{fixture, remove};
use std::cell::RefCell;
use std::os::unix::fs::PermissionsExt as _;

/// A directory with a live index file and a private copy beside it.
fn directory(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let live: PathBuf = root.join("index");
    let built: PathBuf = root.join("index.cli-git-7");
    std::fs::write(&live, b"live").expect("live");
    std::fs::write(&built, b"copy").expect("copy");
    return (root, live, built);
}

/// The sorted names in a directory.
fn names(root: &Path) -> Vec<String> {
    let mut found: Vec<String> = std::fs::read_dir(root).expect("directory").map(|entry| { return entry.expect("entry").file_name().to_string_lossy().into_owned(); }).collect();
    found.sort();
    return found;
}

/// The permission bits of a file.
fn mode_of(path: &Path) -> u32 {
    return std::fs::metadata(path).expect("metadata").permissions().mode() & 0o7777;
}

/// A mask of 022, as the controls that fix the umask use it.
fn umask_022() -> std::io::Result<u32> {
    return Ok(0o022);
}

/// The private copy has one name, built from the live index and the batch, in the live index's directory.
#[test]
fn the_private_name_is_built_from_the_live_index_and_the_batch() {
    let live: PathBuf = PathBuf::from("/repository/.git/index");
    assert_eq!(crate::service_land_follow::private_index_path(live.as_path(), 7), PathBuf::from("/repository/.git/index.cli-git-7"));
    assert_ne!(crate::service_land_follow::private_index_path(live.as_path(), 7), crate::service_land_follow::private_index_path(live.as_path(), 8));
    let other: PathBuf = PathBuf::from("/repository/.git/index2");
    assert_eq!(crate::service_land_follow::private_index_path(other.as_path(), 7).parent(), other.parent());
}

/// Git's `calc_shared_perm`: positive values add bits, negative values replace them, a read-only file gets no write bit, and an
/// executable one gets execute where it gets read.
#[test]
fn shared_modes_follow_calc_shared_perm() {
    assert_eq!(shared_mode(0o660, 0o600), 0o660);
    assert_eq!(shared_mode(0o664, 0o600), 0o664);
    assert_eq!(shared_mode(0o660, 0o644), 0o664);
    assert_eq!(shared_mode(-0o640, 0o600), 0o640);
    assert_eq!(shared_mode(-0o640, 0o666), 0o640);
    assert_eq!(shared_mode(0o660, 0o400), 0o440);
    assert_eq!(shared_mode(0o660, 0o700), 0o770);
}

/// The mode is set before the first byte is written, also when `core.sharedRepository` is narrower than the mask gives: under a
/// mask of 022 a value of 0640 never has the file readable by the world (a value that adds bits adds them to the 0644 Git's file
/// has), and the file is empty when its mode is set.
#[test]
fn the_mode_is_set_before_any_byte_is_written() {
    for (shared, expected) in [(-0o640, 0o640), (0o660, 0o664), (0o664, 0o664), (-0o600, 0o600)] {
        let (root, _live, built) = directory("publish-order");
        let seen: RefCell<Vec<(&str, u32, u64)>> = RefCell::new(Vec::new());
        let chmod = |file: &std::fs::File, mode: u32| -> std::io::Result<()> {
            seen.borrow_mut().push(("chmod", file.metadata()?.permissions().mode() & 0o7777, file.metadata()?.len()));
            return (SYSTEM.chmod)(file, mode);
        };
        let write = |file: &mut std::fs::File, bytes: &[u8]| -> std::io::Result<usize> {
            seen.borrow_mut().push(("write", file.metadata()?.permissions().mode() & 0o7777, file.metadata()?.len()));
            return (SYSTEM.write)(file, bytes);
        };
        let steps: Steps<'_> = Steps { write: &write, chmod: &chmod, umask: &umask_022, ..SYSTEM };
        publish_with(&steps, built.as_path(), b"index bytes", shared).expect("published");
        let seen = seen.into_inner();
        // At the change of mode the file is empty and owner-only; at the first write it already has the final mode.
        assert_eq!(seen[0], ("chmod", 0o600, 0), "shared {shared:o}");
        assert_eq!(seen[1], ("write", expected, 0), "shared {shared:o}");
        assert_eq!(mode_of(built.as_path()), expected, "shared {shared:o}");
        remove(root.as_path());
    }
}

/// Writes that take a few bytes at a time still publish the whole file.
#[test]
fn short_writes_publish_the_whole_file() {
    let (root, _live, built) = directory("publish-short");
    let bytes: Vec<u8> = (0..1000u32).map(|number| { return (number % 253) as u8; }).collect();
    let write = |file: &mut std::fs::File, part: &[u8]| -> std::io::Result<usize> { return (SYSTEM.write)(file, &part[..part.len().min(3)]); };
    publish_with(&Steps { write: &write, ..SYSTEM }, built.as_path(), bytes.as_slice(), 0).expect("published");
    assert_eq!(std::fs::read(&built).expect("target"), bytes);
    remove(root.as_path());
}

/// A write that fails midway, a write that takes nothing, a mode that cannot be set, a flush that fails and a rename that fails
/// each end with an error, no file of this run left, and the target as it was.
#[test]
fn every_failed_step_leaves_the_target_and_no_temporary_file() {
    let broken = || -> std::io::Error { return std::io::Error::other("injected"); };
    let calls: RefCell<u32> = RefCell::new(0);
    let fails_after_a_while = |file: &mut std::fs::File, part: &[u8]| -> std::io::Result<usize> {
        *calls.borrow_mut() += 1;
        if *calls.borrow() > 2 { Err(broken()) } else { (SYSTEM.write)(file, &part[..part.len().min(5)]) }
    };
    let takes_nothing = |_: &mut std::fs::File, _: &[u8]| -> std::io::Result<usize> { return Ok(0); };
    let no_chmod = |_: &std::fs::File, _: u32| -> std::io::Result<()> { return Err(broken()); };
    let no_sync = |_: &std::fs::File| -> std::io::Result<()> { return Err(broken()); };
    let no_rename = |_: &Path, _: &Path| -> std::io::Result<()> { return Err(broken()); };
    let no_umask = || -> std::io::Result<u32> { return Err(broken()); };
    let cases: Vec<(&str, Steps<'_>, i32)> = vec![
        ("a write that fails midway", Steps { write: &fails_after_a_while, ..SYSTEM }, 0),
        ("a write that takes nothing", Steps { write: &takes_nothing, ..SYSTEM }, 0),
        ("a mode that cannot be set", Steps { chmod: &no_chmod, ..SYSTEM }, 0o660),
        ("a flush that fails", Steps { sync: &no_sync, ..SYSTEM }, 0),
        ("a rename that fails", Steps { rename: &no_rename, ..SYSTEM }, 0),
        ("a mask that cannot be read", Steps { umask: &no_umask, ..SYSTEM }, 0o660),
    ];
    for (label, steps, shared) in cases {
        let (root, _live, built) = directory("publish-failure-steps");
        *calls.borrow_mut() = 0;
        let before: Vec<String> = names(root.as_path());
        assert!(publish_with(&steps, built.as_path(), b"a long enough payload to need several writes", shared).is_err(), "{label}");
        assert_eq!(std::fs::read(&built).expect("target"), b"copy".to_vec(), "{label}: the target changed");
        assert_eq!(names(root.as_path()), before, "{label}: a file was left");
        remove(root.as_path());
    }
}

/// A rename onto a directory and a missing parent fail without leaving a file.
#[test]
fn a_target_that_cannot_be_replaced_leaves_nothing() {
    let (root, _live, _built) = directory("publish-target");
    let target: PathBuf = root.join("index.cli-git-9");
    std::fs::create_dir_all(target.join("occupied")).expect("a directory where the file should go");
    assert!(publish(target.as_path(), b"bytes", 0).is_err());
    assert!(target.join("occupied").is_dir());
    assert!(publish(root.join("missing").join("index.cli-git-9").as_path(), b"bytes", 0).is_err());
    assert_eq!(names(root.as_path()), vec!["index".to_string(), "index.cli-git-7".to_string(), "index.cli-git-9".to_string()]);
    remove(root.as_path());
}

/// A payload that checks itself: a 16-byte header naming the writer and the length, then the writer's byte to the end.
fn payload(writer: u8, length: usize) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::with_capacity(length);
    bytes.extend_from_slice(&(length as u64).to_be_bytes());
    bytes.extend_from_slice(&[writer; 8]);
    bytes.resize(length, writer);
    return bytes;
}

/// Whether `bytes` is a whole payload of one writer.
fn is_whole(bytes: &[u8]) -> bool {
    let Some(header) = bytes.get(..16) else {
        return false;
    };
    let length: usize = u64::from_be_bytes(header[..8].try_into().expect("eight bytes")) as usize;
    return length == bytes.len() && bytes[8..16] == [bytes[8]; 8] && bytes[16..].iter().all(|byte| { return *byte == bytes[8]; });
}

/// Writers publishing different payloads to one target at once, with a reader reading it all the while: every read is one whole
/// payload of one writer, the last one is whole, and nothing but the target and the live index is left.
#[test]
fn concurrent_publications_of_different_payloads_are_always_whole() {
    let (root, _live, built) = directory("publish-concurrent");
    std::fs::write(&built, payload(0, 4096)).expect("first payload");
    let barrier: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::new(std::sync::Barrier::new(4));
    let finished: std::sync::Arc<std::sync::atomic::AtomicUsize> = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let reader = {
        let (target, done) = (built.clone(), std::sync::Arc::clone(&finished));
        std::thread::spawn(move || {
            let mut reads: u32 = 0;
            // The reader pauses between reads and ends when the last writer has.
            while done.load(Ordering::SeqCst) < 4 {
                assert!(is_whole(std::fs::read(&target).expect("read").as_slice()), "a reader saw a payload that is not whole");
                reads += 1;
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            return reads;
        })
    };
    let writers: Vec<std::thread::JoinHandle<()>> = (1..=4u8)
        .map(|writer| {
            let (target, gate, done) = (built.clone(), std::sync::Arc::clone(&barrier), std::sync::Arc::clone(&finished));
            return std::thread::spawn(move || {
                for round in 0..20usize {
                    gate.wait();
                    publish(target.as_path(), payload(writer, 8192 + round * 64 + usize::from(writer)).as_slice(), 0).expect("published");
                }
                done.fetch_add(1, Ordering::SeqCst);
            });
        })
        .collect();
    for writer in writers {
        writer.join().expect("writer");
    }
    assert!(reader.join().expect("reader") > 0);
    assert!(is_whole(std::fs::read(&built).expect("target").as_slice()));
    assert_eq!(names(root.as_path()), vec!["index".to_string(), "index.cli-git-7".to_string()]);
    remove(root.as_path());
}

/// Under a real mask of 077, which the test sets in a child process, the system calls give Git's modes: the mask's for no shared
/// repository, the widened one for `group` and the replaced one for a file mode value.
#[test]
fn the_published_mode_under_a_real_umask_is_gits() {
    if crate::umask_child_test_support::ran_in_child(module_path!(), "the_published_mode_under_a_real_umask_is_gits", 0o077) {
        return;
    }
    assert_eq!(process_umask().expect("the mask"), 0o077);
    let (root, _live, built) = directory("publish-mode");
    for (shared, expected) in [(0, 0o600), (0o660, 0o660), (0o664, 0o664), (-0o640, 0o640)] {
        publish(built.as_path(), b"x", shared).expect("published");
        assert_eq!(mode_of(built.as_path()), expected, "shared {shared:o}");
    }
    remove(root.as_path());
}
