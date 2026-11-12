//! Controls for the temporary file,
//! the file mode and the rename of the landing's private index,
//! with the file system steps made
//! to fail or run short.

use super::*;
use crate::test_support::{fixture, remove};
use std::cell::RefCell;

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

/// Each private copy has a directory of its own beside the live index,
/// named at random,
/// so two copies never share a name.
#[test]
fn the_private_copy_has_a_directory_of_its_own_beside_the_live_index() {
    let root: PathBuf = fixture("index-reset-publish-copy-directory");
    let live: PathBuf = root.join("index");
    std::fs::write(&live, b"live").expect("live");
    let lock = crate::service_index_lock::try_acquire(live.as_path(), 7).expect("the lock is free");
    let first = crate::service_land_follow::PrivateCopy::make(&lock, 7).expect("a copy");
    let second = crate::service_land_follow::PrivateCopy::make(&lock, 7).expect("another copy of the same batch");
    assert_eq!(first.directory().parent(), Some(root.as_path()));
    assert_ne!(first.directory(), second.directory());
    assert_eq!(std::fs::read(first.path()).expect("copy"), b"live".to_vec());
    drop(first);
    drop(second);
    drop(lock);
    remove(root.as_path());
}

/// Git's `calc_shared_perm`:
/// positive values add bits,
/// negative values replace them,
/// a read-only file gets no write bit,
/// and an
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

/// The mode is set before the first byte is written,
/// also when `core.sharedRepository` is narrower than the mask gives:
/// under a
/// mask of 022 (the test runs in a child that starts with it) a value of 0640 never has the file readable by the world (a value
/// that adds bits adds them to the 0644 Git's file has),
/// and the file is empty when its mode is set.
#[test]
fn the_mode_is_set_before_any_byte_is_written() {
    if crate::umask_child_test_support::ran_in_child(module_path!(), "the_mode_is_set_before_any_byte_is_written", 0o022) {
        return;
    }
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
        let steps: Steps<'_> = Steps { write: &write, chmod: &chmod, ..SYSTEM };
        publish_with(&steps, built.as_path(), b"index bytes", shared, None).expect("published");
        let order = seen.into_inner();
        // At the change of mode the file is empty and has the mode Git's file has; at the first write it has the final mode.
        assert_eq!(order[0], ("chmod", 0o644, 0), "shared {shared:o}");
        assert_eq!(order[1], ("write", expected, 0), "shared {shared:o}");
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
    publish_with(&Steps { write: &write, ..SYSTEM }, built.as_path(), bytes.as_slice(), 0, None).expect("published");
    assert_eq!(std::fs::read(&built).expect("target"), bytes);
    remove(root.as_path());
}

/// A write that fails midway,
/// a write that takes nothing,
/// a mode that cannot be set,
/// a flush that fails and a rename that fails
/// each end with an error,
/// no file of this run left,
/// and the target as it was.
#[test]
fn every_failed_step_leaves_the_target_and_no_temporary_file() {
    let broken = || -> std::io::Error { return std::io::Error::other("injected"); };
    let calls: RefCell<u32> = RefCell::new(0);
    let fails_after_a_while = |file: &mut std::fs::File, part: &[u8]| -> std::io::Result<usize> {
        *calls.borrow_mut() += 1;
        if *calls.borrow() > 2 {
            return Err(broken());
        }
        return (SYSTEM.write)(file, &part[..part.len().min(5)]);
    };
    let takes_nothing = |_: &mut std::fs::File, _: &[u8]| -> std::io::Result<usize> { return Ok(0); };
    let no_chmod = |_: &std::fs::File, _: u32| -> std::io::Result<()> { return Err(broken()); };
    let no_sync = |_: &std::fs::File| -> std::io::Result<()> { return Err(broken()); };
    let no_rename = |_: &Path, _: &Path| -> std::io::Result<()> { return Err(broken()); };
    let no_mode = |_: &std::fs::File| -> std::io::Result<u32> { return Err(broken()); };
    let cases: Vec<(&str, Steps<'_>, i32)> = vec![
        ("a write that fails midway", Steps { write: &fails_after_a_while, ..SYSTEM }, 0),
        ("a write that takes nothing", Steps { write: &takes_nothing, ..SYSTEM }, 0),
        ("a mode that cannot be set", Steps { chmod: &no_chmod, ..SYSTEM }, 0o660),
        ("a flush that fails", Steps { sync: &no_sync, ..SYSTEM }, 0),
        ("a rename that fails", Steps { rename: &no_rename, ..SYSTEM }, 0),
        ("a mode that cannot be read", Steps { mode: &no_mode, ..SYSTEM }, 0o660),
    ];
    for (label, steps, shared) in cases {
        let (root, _live, built) = directory("publish-failure-steps");
        *calls.borrow_mut() = 0;
        let before: Vec<String> = names(root.as_path());
        assert!(publish_with(&steps, built.as_path(), b"a long enough payload to need several writes", shared, None).is_err(), "{label}");
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
    assert!(publish(target.as_path(), b"bytes", 0, None).is_err());
    assert!(target.join("occupied").is_dir());
    assert!(publish(root.join("missing").join("index.cli-git-9").as_path(), b"bytes", 0, None).is_err());
    assert_eq!(names(root.as_path()), vec!["index".to_string(), "index.cli-git-7".to_string(), "index.cli-git-9".to_string()]);
    remove(root.as_path());
}

/// A payload that checks itself:
/// a 16-byte header naming the writer and the length,
/// then the writer's byte to the end.
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

/// Writers publishing different payloads to one target at once,
/// with a reader reading it all the while:
/// every read is one whole
/// payload of one writer,
/// the last one is whole,
/// and nothing but the target and the live index is left.
#[test]
fn concurrent_publications_of_different_payloads_are_always_whole() {
    let (root, _live, built) = directory("publish-concurrent");
    std::fs::write(&built, payload(0, 4096)).expect("first payload");
    let barrier: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::new(std::sync::Barrier::new(4));
    let finished: std::sync::Arc<std::sync::atomic::AtomicUsize> = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    // The writers start only after the reader has read once, so the reads overlap the publications.
    let reader_started: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::new(std::sync::Barrier::new(2));
    let reader = {
        let (target, done, started) = (built.clone(), std::sync::Arc::clone(&finished), std::sync::Arc::clone(&reader_started));
        std::thread::spawn(move || {
            assert!(is_whole(std::fs::read(&target).expect("first read").as_slice()));
            started.wait();
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
    reader_started.wait();
    let writers: Vec<std::thread::JoinHandle<()>> = (1..=4u8)
        .map(|writer| {
            let (target, gate, done) = (built.clone(), std::sync::Arc::clone(&barrier), std::sync::Arc::clone(&finished));
            return std::thread::spawn(move || {
                for round in 0..20usize {
                    gate.wait();
                    publish(target.as_path(), payload(writer, 8192 + round * 64 + usize::from(writer)).as_slice(), 0, None).expect("published");
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

/// A reader that reads the target while a publication is in flight (during its writes,
/// at its flush,
/// on both sides of its rename)
/// sees the old payload whole until the rename and the new one whole after it.
/// The reads are made from inside the publication's own
/// steps,
/// on another thread each time,
/// so each provably overlaps it:
/// the stress test above can finish without one doing so.
#[test]
fn a_read_inside_each_step_of_a_publication_sees_one_whole_payload() {
    let (root, _live, built) = directory("publish-overlap");
    let old: Vec<u8> = payload(1, 4096);
    let new: Vec<u8> = payload(2, 9000);
    std::fs::write(&built, old.as_slice()).expect("first payload");
    let during: RefCell<Vec<(&'static str, Vec<u8>)>> = RefCell::new(Vec::new());
    let read_on_another_thread = |label: &'static str| {
        let target: PathBuf = built.clone();
        let bytes: Vec<u8> = std::thread::scope(|scope| { return scope.spawn(|| { return std::fs::read(&target).expect("read"); }).join().expect("reader"); });
        during.borrow_mut().push((label, bytes));
    };
    let write = |file: &mut std::fs::File, part: &[u8]| -> std::io::Result<usize> {
        let taken: usize = file.write(&part[..part.len().min(1000)])?;
        read_on_another_thread("write");
        return Ok(taken);
    };
    let sync = |file: &std::fs::File| -> std::io::Result<()> {
        read_on_another_thread("flush");
        return file.sync_all();
    };
    let rename = |from: &Path, to: &Path| -> std::io::Result<()> {
        read_on_another_thread("before rename");
        std::fs::rename(from, to)?;
        read_on_another_thread("after rename");
        return Ok(());
    };
    publish_with(&Steps { write: &write, sync: &sync, rename: &rename, ..SYSTEM }, built.as_path(), new.as_slice(), 0, None).expect("published");
    let seen = during.into_inner();
    assert!(seen.iter().filter(|(label, _)| { return *label == "write"; }).count() > 1, "the reader overlapped several writes: {}", seen.len());
    for (label, bytes) in &seen {
        let expected: &Vec<u8> = if *label == "after rename" { &new } else { &old };
        assert!(bytes == expected, "a reader at {label} saw something other than the whole {} payload", if *label == "after rename" { "new" } else { "old" });
    }
    assert!(std::fs::read(&built).expect("target") == new);
    remove(root.as_path());
}

/// Under a real mask of 077,
/// which the test sets in a child process,
/// the system calls give Git's modes:
/// the mask's for no shared
/// repository,
/// the widened one for `group` and the replaced one for a file mode value.
#[test]
fn the_published_mode_under_a_real_umask_is_gits() {
    if crate::umask_child_test_support::ran_in_child(module_path!(), "the_published_mode_under_a_real_umask_is_gits", 0o077) {
        return;
    }
    let (root, _live, built) = directory("publish-mode");
    for (shared, expected) in [(0, 0o600), (0o660, 0o660), (0o664, 0o664), (-0o640, 0o640)] {
        publish(built.as_path(), b"x", shared, None).expect("published");
        assert_eq!(mode_of(built.as_path()), expected, "shared {shared:o}");
    }
    remove(root.as_path());
}

/// A write that is interrupted by a signal is tried again,
/// as `write_all` does,
/// and the whole file is published.
#[test]
fn an_interrupted_write_is_tried_again() {
    let (root, _live, built) = directory("publish-interrupted");
    let tries: RefCell<u32> = RefCell::new(0);
    let write = |file: &mut std::fs::File, part: &[u8]| -> std::io::Result<usize> {
        *tries.borrow_mut() += 1;
        if *tries.borrow() % 2 == 1 {
            return Err(std::io::Error::from(std::io::ErrorKind::Interrupted));
        }
        return (SYSTEM.write)(file, &part[..part.len().min(4)]);
    };
    publish_with(&Steps { write: &write, ..SYSTEM }, built.as_path(), b"a payload that needs several writes", 0, None).expect("published");
    assert_eq!(std::fs::read(&built).expect("target"), b"a payload that needs several writes".to_vec());
    assert!(*tries.borrow() > 4);
    remove(root.as_path());
}
