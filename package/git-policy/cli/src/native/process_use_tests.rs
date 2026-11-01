//! Controls for the scan of processes that use a directory or a file.

use super::*;
use crate::test_support::{Sleeper, fixture, remove};
use std::os::unix::fs::MetadataExt;

/// A process whose working directory lies inside the directory is found, and a process elsewhere is not.
#[cfg(target_os = "linux")]
#[test]
fn a_working_directory_inside_the_directory_is_a_use() {
    let root: PathBuf = fixture("process-use-cwd").canonicalize().expect("canonical root");
    let inside: PathBuf = root.join("transaction/hooks");
    std::fs::create_dir_all(&inside).expect("inside");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir_all(&elsewhere).expect("elsewhere");
    let target: PathBuf = root.join("transaction");
    let bystander: Sleeper = Sleeper::start(elsewhere.as_path(), None);
    let before: UseScan = processes_using(&UseTarget::Inside(target.as_path()));
    assert!(!before.users.iter().any(|user| { return user.pid == bystander.pid(); }), "{before:?}");
    let holder: Sleeper = Sleeper::start(inside.as_path(), None);
    let after: UseScan = processes_using(&UseTarget::Inside(target.as_path()));
    assert!(after.users.iter().any(|user| { return user.pid == holder.pid(); }), "{after:?}");
    assert!(!after.users.iter().any(|user| { return user.pid == bystander.pid(); }), "{after:?}");
    drop(holder);
    drop(bystander);
    remove(root.as_path());
}

/// An open descriptor on a file inside the directory is a use, whatever the working directory.
#[cfg(target_os = "linux")]
#[test]
fn an_open_descriptor_inside_the_directory_is_a_use() {
    let root: PathBuf = fixture("process-use-fd").canonicalize().expect("canonical root");
    let target: PathBuf = root.join("transaction");
    std::fs::create_dir_all(&target).expect("transaction");
    std::fs::write(target.join("landing-1.json"), b"{}").expect("record");
    let holder: Sleeper = Sleeper::start(root.as_path(), Some(target.join("landing-1.json").as_path()));
    let scan: UseScan = processes_using(&UseTarget::Inside(target.as_path()));
    assert!(scan.users.iter().any(|user| { return user.pid == holder.pid(); }), "{scan:?}");
    assert_eq!(scan.users.iter().find(|user| { return user.pid == holder.pid(); }).and_then(|user| { return user.command.clone(); }), Some(String::from("sleep")));
    drop(holder);
    remove(root.as_path());
}

/// A file is recognized by device and inode, so a second name for the lock is the lock.
#[cfg(target_os = "linux")]
#[test]
fn an_open_descriptor_is_recognized_by_identity_under_any_name() {
    let root: PathBuf = fixture("process-use-identity").canonicalize().expect("canonical root");
    std::fs::write(root.join("index.lock"), b"").expect("lock");
    std::fs::hard_link(root.join("index.lock"), root.join("alias")).expect("alias");
    let metadata: std::fs::Metadata = std::fs::metadata(root.join("index.lock")).expect("lock metadata");
    let target: UseTarget = UseTarget::File { device: metadata.dev(), inode: metadata.ino() };
    assert!(processes_using(&target).users.is_empty());
    let holder: Sleeper = Sleeper::start(root.as_path(), Some(root.join("alias").as_path()));
    let scan: UseScan = processes_using(&target);
    assert!(scan.users.iter().any(|user| { return user.pid == holder.pid(); }), "{scan:?}");
    drop(holder);
    remove(root.as_path());
}

/// The scanning process is never its own user.
#[cfg(target_os = "linux")]
#[test]
fn the_scanning_process_is_not_reported() {
    let root: PathBuf = fixture("process-use-self").canonicalize().expect("canonical root");
    let held: std::fs::File = std::fs::File::create(root.join("held")).expect("held");
    let metadata: std::fs::Metadata = held.metadata().expect("metadata");
    let scan: UseScan = processes_using(&UseTarget::File { device: metadata.dev(), inode: metadata.ino() });
    assert!(!scan.users.iter().any(|user| { return user.pid == i64::from(std::process::id()); }), "{scan:?}");
    drop(held);
    remove(root.as_path());
}

/// A process table that cannot be listed is reported, not read as empty.
#[test]
fn an_unreadable_process_table_is_reported() {
    let scan: UseScan = processes_using_through(Path::new("/nonexistent-proc-root"), &UseTarget::Inside(Path::new("/")), 1);
    assert!(scan.users.is_empty());
    assert_eq!(scan.unreadable.len(), 1, "{scan:?}");
    assert!(scan.unreadable[0].contains("/nonexistent-proc-root"), "{scan:?}");
}
