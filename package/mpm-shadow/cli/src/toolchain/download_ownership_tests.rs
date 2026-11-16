//! Unit tests of who owns a version's downloads directory:
//! an install owns it before its first byte and a removal sees that,
//! and an install that starts while a removal holds the right does not write under it.

/// Imports `fs` from `std`.
use std::fs;
/// Imports what these tests use from `std::path`.
use std::path::PathBuf;

/// Imports what these tests use from the module under test.
use super::download_ownership::{DownloadsOwner, RemovalRight, live_owner, take_removal_right};
/// Imports `downloads_of` from `super::store_versions`,
/// which names the directory these rights are about.
use super::store_versions::downloads_of;
/// Imports the archive removal,
/// which is the removal that used to take a directory an install was writing in.
use super::published_archive::remove_downloads;
use crate::test_scratch::scratch;

/// What:
/// An install that owns a version's downloads keeps them from the removal that would delete them,
/// and the archive is still readable afterwards,
/// which is what the install does next.
/// Why:
/// The review of this change found the in-use check and the recursive delete were a check and then a delete:
/// an install that wrote its archive after the check lost it and failed.
/// The ownership is taken before the install's first byte and seen by the removal,
/// so there is no instant between a
/// check and a delete in which an install can start.
#[test]
fn an_install_that_owns_its_downloads_keeps_them() {
    let directory = scratch();
    let root: PathBuf = directory.path().join("store");
    let ownership: DownloadsOwner = DownloadsOwner::take(&root, "bin", "0.29.4").expect("take the downloads");
    let archive: PathBuf = downloads_of(&root, "bin", "0.29.4").join("bin-0.29.4.tar.gz");
    fs::write(&archive, b"the archive an install is unpacking").expect("write the archive");
    // Every file here, the owner file included, is older than the in-use floor:
    // an install that has been unpacking for over an hour is the case the owner file is read for,
    // and the age rule must not be what keeps the archive in this test.
    let downloads: PathBuf = downloads_of(&root, "bin", "0.29.4");
    for entry in fs::read_dir(&downloads).expect("read the downloads").flatten() {
        crate::toolchain::fixtures::age_file(&entry.path(), std::time::Duration::from_secs(2 * 60 * 60));
    }

    remove_downloads(&root, "bin", "0.29.4").expect("nothing to do");

    assert!(archive.is_file(), "the removal took the archive of an install that owns the directory");
    assert_eq!(fs::read(&archive).expect("read it back"), b"the archive an install is unpacking");

    // Control: once the install is done, the same removal takes the directory, so the ownership is not a leak.
    drop(ownership);
    remove_downloads(&root, "bin", "0.29.4").expect("removed");
    assert!(!archive.exists(), "the archives stayed after the install gave them up");
}

/// What:
/// An install that starts while a removal holds the right to its downloads does not write under it:
/// it waits for the removal,
/// and if the right is still held when its wait ends it refuses and writes nothing.
/// Why:
/// The other half of the same rule:
/// a removal takes the right before it judges the directory,
/// so an install that starts afterwards finds the claim and keeps out,
/// which is what makes the removal's delete safe without a window.
#[test]
fn an_install_does_not_write_under_a_removal_that_holds_the_right() {
    let directory = scratch();
    let root: PathBuf = directory.path().join("store");
    let downloads: PathBuf = downloads_of(&root, "bin", "0.29.4");
    fs::create_dir_all(&downloads).expect("make the downloads directory");
    fs::write(downloads.join("old-archive.tar.gz"), b"old").expect("write an old archive");
    let right: RemovalRight = take_removal_right(&root, "bin", "0.29.4").expect("ask for the right");
    let RemovalRight::Held { claim } = right else {
        panic!("the removal did not get the right to a directory nobody else holds");
    };

    let refused = DownloadsOwner::take(&root, "bin", "0.29.4");

    assert!(refused.is_err(), "an install took a directory a removal holds the right to");
    let message: String = refused.expect_err("the reason").message;
    assert!(message.contains("removing the archives"), "the refusal says what is happening: {message}");
    assert!(message.contains("run the command again"), "the refusal says what to do next: {message}");
    assert_eq!(
        fs::read_dir(&downloads).expect("read the downloads").flatten().count(),
        1,
        "the refused install wrote into the directory all the same"
    );

    // Control: the right being given up lets the next install take the directory.
    drop(claim);
    let ownership: DownloadsOwner = DownloadsOwner::take(&root, "bin", "0.29.4").expect("take the downloads");
    assert!(live_owner(&downloads).is_some(), "the owner file a live install wrote is not seen");
    drop(ownership);
    assert!(live_owner(&downloads).is_none(), "the owner file outlived the ownership");
}

/// What:
/// A removal claim a killed remover left does not block an install for good:
/// the claim names its maker,
/// and a claim whose maker has ended is cleared before the right is taken again.
/// Why:
/// The review of this change showed one kill leaving the claim behind and every later install of that version waiting
/// two seconds and failing,
/// for ever,
/// with nothing to point at but an internal file.
#[test]
fn a_claim_a_killed_remover_left_does_not_block_an_install() {
    let directory = scratch();
    let root: PathBuf = directory.path().join("store");
    let downloads: PathBuf = downloads_of(&root, "bin", "0.29.4");
    fs::create_dir_all(&downloads).expect("make the downloads directory");
    let claim: PathBuf = downloads.with_file_name(".0.29.4.download-removal");
    // A claim naming a process that has ended, as a kill between the create and the drop leaves it.
    fs::write(&claim, format!("{}.{}\n", u32::MAX - 1, 12_345)).expect("leave the stale claim");
    let aged: std::time::SystemTime = std::time::SystemTime::now() - std::time::Duration::from_secs(60);
    fs::File::options()
        .write(true)
        .open(&claim)
        .expect("open the claim to age it")
        .set_times(std::fs::FileTimes::new().set_modified(aged))
        .expect("age the claim");

    let first: Result<DownloadsOwner, _> = DownloadsOwner::take(&root, "bin", "0.29.4");
    assert!(first.is_ok(), "a dead remover's claim blocked the install: {:?}", first.err());
    drop(first.expect("the first install owns the downloads"));

    let second: Result<DownloadsOwner, _> = DownloadsOwner::take(&root, "bin", "0.29.4");
    assert!(second.is_ok(), "the same stale claim still blocked a retry: {:?}", second.err());
    assert!(live_owner(&downloads).is_some(), "the install owns the directory after taking it");
}

/// What:
/// Taking a stale claim removes exactly the claim it read, never a live claim another remover put there in the
/// meantime.
/// Why:
/// The review of this change read a claim naming a dead maker, had another remover replace it with a live one before
/// the unlink, and the unlink removed the live claim: both removers then held the right to one downloads directory.
/// The harness holds the reader on a FIFO at the claim's name, so the replacement lands between the read and the
/// unlink by construction.
#[cfg(unix)]
#[test]
fn a_stale_claim_is_taken_by_removing_exactly_the_claim_it_read() {
    use std::io::Write;
    use std::os::unix::fs::MetadataExt;

    let directory = scratch();
    let root: PathBuf = directory.path().join("store");
    let downloads: PathBuf = downloads_of(&root, "bin", "0.29.4");
    fs::create_dir_all(&downloads).expect("make the downloads directory");
    let claim: PathBuf = downloads.with_file_name(".0.29.4.download-removal");
    let fifo: PathBuf = downloads.with_file_name("claim-fifo");
    let status = std::process::Command::new("mkfifo").arg(&fifo).status().expect("run mkfifo");
    assert!(status.success(), "mkfifo failed");
    fs::rename(&fifo, &claim).expect("put a FIFO where the claim belongs");
    let root_for_thread: PathBuf = root.clone();
    let remover: std::thread::JoinHandle<_> = std::thread::spawn(move || return take_removal_right(&root_for_thread, "bin", "0.29.4"));
    let mut writer: fs::File = fs::OpenOptions::new().write(true).open(&claim).expect("open the claim's writer");
    // The live claim another remover would put there, naming this process.
    let live: PathBuf = downloads.with_file_name("live-replacement-claim");
    fs::write(&live, format!("{}.0\n", std::process::id())).expect("write the live claim");
    let live_inode: u64 = fs::symlink_metadata(&live).expect("read the live claim").ino();
    fs::rename(&live, &claim).expect("replace the old claim with the live one");
    writer.write_all(b"4294967294.12345\n").expect("release the reader with a dead maker");
    drop(writer);
    let right = remover.join().expect("the remover thread ended").expect("the removal right");

    assert!(matches!(&right, RemovalRight::Held { .. }), "the probe did not reach the stale-claim branch");
    let after_inode: u64 = fs::symlink_metadata(&claim).expect("the claim after the take").ino();
    drop(right);
    assert_eq!(after_inode, live_inode, "stale cleanup removed the live replacement claim");
}
