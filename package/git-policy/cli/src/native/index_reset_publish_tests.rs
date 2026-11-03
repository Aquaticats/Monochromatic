//! Controls for the lock proof, the temporary file and the file mode of the landing's private index.

use super::*;
use crate::service_index_lock::try_acquire;
use crate::test_support::{fixture, remove};

/// A directory with a live index file and the name of a private copy beside it.
fn directory(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let live: PathBuf = root.join("index");
    let built: PathBuf = root.join("index.cli-git-7");
    std::fs::write(&live, b"live").expect("live");
    std::fs::write(&built, b"copy").expect("copy");
    return (root, live, built);
}

/// The lock counts only when it is this process's, for this batch, and the private index is another file beside the live one.
#[test]
fn only_this_batchs_lock_counts() {
    let (root, live, built) = directory("publish-lock");
    let landing: Landing<'_> = Landing { live: live.as_path(), built: built.as_path(), seq: 7 };
    assert!(!holds_the_lock(&landing), "no lock");
    std::fs::write(lock_path_of(live.as_path()), b"someone else's lock").expect("foreign lock");
    assert!(!holds_the_lock(&landing), "a foreign lock");
    std::fs::remove_file(lock_path_of(live.as_path())).expect("removed");
    let held = try_acquire(live.as_path(), 7).expect("free");
    assert!(holds_the_lock(&landing), "our lock for this batch");
    assert!(!holds_the_lock(&Landing { seq: 8, ..landing }), "another batch");
    assert!(!holds_the_lock(&Landing { built: live.as_path(), ..landing }), "the live index itself");
    let elsewhere: PathBuf = root.join("sub").join("index.cli-git-7");
    assert!(!holds_the_lock(&Landing { built: elsewhere.as_path(), ..landing }), "a file in another directory");
    drop(held);
    assert!(!holds_the_lock(&landing), "the lock is gone once released");
    remove(root.as_path());
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

/// Many runs publishing to one target at once leave the whole file every time and nothing else in the directory.
#[test]
fn concurrent_runs_never_share_a_temporary_file() {
    let (root, _live, built) = directory("publish-concurrent");
    let bytes: Vec<u8> = (0..200_000u32).map(|number| { return (number % 251) as u8; }).collect();
    let barrier: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::new(std::sync::Barrier::new(4));
    let runs: Vec<std::thread::JoinHandle<()>> = (0..4)
        .map(|_| {
            let (target, content, gate) = (built.clone(), bytes.clone(), std::sync::Arc::clone(&barrier));
            return std::thread::spawn(move || {
                for _ in 0..25 {
                    gate.wait();
                    publish(target.as_path(), content.as_slice(), 0).expect("published");
                }
            });
        })
        .collect();
    for run in runs {
        run.join().expect("run");
    }
    assert_eq!(std::fs::read(&built).expect("target"), bytes);
    let mut names: Vec<String> = std::fs::read_dir(&root).expect("directory").map(|entry| { return entry.expect("entry").file_name().to_string_lossy().into_owned(); }).collect();
    names.sort();
    assert_eq!(names, vec!["index".to_string(), "index.cli-git-7".to_string()]);
    remove(root.as_path());
}

/// A failed step removes this run's file and leaves the target: the rename onto a directory fails, and so does a missing parent.
#[test]
fn a_failure_removes_only_this_runs_file() {
    let (root, _live, _built) = directory("publish-failure");
    let target: PathBuf = root.join("index.cli-git-9");
    std::fs::create_dir_all(target.join("occupied")).expect("a directory where the file should go");
    assert!(publish(target.as_path(), b"bytes", 0).is_err());
    assert!(target.join("occupied").is_dir());
    assert!(publish(root.join("missing").join("index.cli-git-9").as_path(), b"bytes", 0).is_err());
    let mut names: Vec<String> = std::fs::read_dir(&root).expect("directory").map(|entry| { return entry.expect("entry").file_name().to_string_lossy().into_owned(); }).collect();
    names.sort();
    assert_eq!(names, vec!["index".to_string(), "index.cli-git-7".to_string(), "index.cli-git-9".to_string()]);
    remove(root.as_path());
}

/// With a shared repository the published file has the widened mode whatever the umask, and without one it has the umask's.
#[test]
fn the_published_mode_is_gits() {
    // Each test runs in a process of its own under nextest, so the umask change reaches no other test.
    // SAFETY: `umask` only sets this process's file creation mask.
    unsafe { libc::umask(0o077) };
    let (root, _live, built) = directory("publish-mode");
    publish(built.as_path(), b"x", 0).expect("published");
    assert_eq!(std::fs::metadata(&built).expect("metadata").permissions().mode() & 0o7777, 0o600);
    publish(built.as_path(), b"x", 0o660).expect("published");
    assert_eq!(std::fs::metadata(&built).expect("metadata").permissions().mode() & 0o7777, 0o660);
    publish(built.as_path(), b"x", -0o640).expect("published");
    assert_eq!(std::fs::metadata(&built).expect("metadata").permissions().mode() & 0o7777, 0o640);
    remove(root.as_path());
}
