//! Controls for the unit-test fixtures: where they live and that they go away however their test ends (issue 1809).

use super::*;

/// A test thread that panics after creating a fixture leaves nothing behind.
#[test]
fn a_fixture_is_removed_when_its_test_thread_panics() {
    let (sender, receiver) = std::sync::mpsc::channel::<PathBuf>();
    let handle = std::thread::spawn(move || {
        let root: PathBuf = fixture("guard-panic");
        std::fs::write(root.join("file"), "content").expect("file");
        sender.send(root).expect("send the root");
        panic!("the test fails before it removes its fixture");
    });
    assert!(handle.join().is_err(), "the thread panics on purpose");
    let root: PathBuf = receiver.recv().expect("the root");
    assert!(!root.exists(), "{} was left behind", root.display());
}

/// A test thread that ends normally leaves nothing behind either, with or without an explicit `remove`.
#[test]
fn a_fixture_is_removed_when_its_test_thread_ends() {
    let (sender, receiver) = std::sync::mpsc::channel::<PathBuf>();
    std::thread::spawn(move || {
        let kept: PathBuf = fixture("guard-ends");
        let removed: PathBuf = fixture("guard-removed");
        remove(removed.as_path());
        sender.send(kept).expect("send the root");
        sender.send(removed).expect("send the root");
    })
    .join()
    .expect("the thread ends normally");
    for root in receiver.try_iter() {
        assert!(!root.exists(), "{} was left behind", root.display());
    }
}

/// Settling keeps a tree on request, removes it otherwise, and accepts a tree that is already gone.
#[test]
fn settling_keeps_removes_and_accepts_a_tree_that_is_gone() {
    let root: PathBuf = fixture("settle");
    std::fs::write(root.join("file"), "content").expect("file");
    settle_fixture(root.as_path(), true);
    assert!(root.join("file").exists(), "a kept fixture stays");
    settle_fixture(root.as_path(), false);
    assert!(!root.exists(), "a settled fixture goes");
    settle_fixture(root.as_path(), false);
}

/// Fixtures are created beside the build output, never in the system temporary directory, unless the person moved the
/// root on purpose.
#[test]
fn the_fixture_root_is_not_the_system_temporary_directory() {
    if std::env::var_os(FIXTURE_ROOT_VARIABLE).is_some() {
        return;
    }
    let root: PathBuf = fixture_root();
    assert!(root.ends_with("test-fixtures"), "{}", root.display());
    // The system temporary directory is `TMPDIR` when set and `/tmp` otherwise; reading the variable compares against it
    // without calling the temporary-directory function that the lint configuration bans.
    let mut system: Vec<PathBuf> = vec![PathBuf::from("/tmp"), PathBuf::from("/var/tmp")];
    system.extend(std::env::var_os("TMPDIR").map(PathBuf::from));
    for directory in system {
        assert!(!root.starts_with(&directory), "{} is under the system temporary directory {}", root.display(), directory.display());
    }
}

/// A process a test starts cannot write a core dump: tests make services abort or kill them on purpose, and every dump costs
///  the system's core-dump handler processor time and disk writes on a disk other sessions share (several hundred dumps of
///  `cli-git-service` from test runs on 2026-11-03). The limit is the harness's, set where fixtures are made; the product
///  sets none.
#[test]
fn a_process_a_test_starts_may_not_write_a_core_dump() {
    let _root: PathBuf = fixture("core-limit");
    let output = std::process::Command::new("sh").args(["-c", "ulimit -c; ulimit -H -c"]).output().expect("sh runs");
    assert_eq!(String::from_utf8_lossy(&output.stdout).split_whitespace().collect::<Vec<&str>>(), vec!["0", "0"]);
}
