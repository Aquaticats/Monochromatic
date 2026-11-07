//! Controls for the probe of writers:
//! each case in its own disposable folder,
//! the refused grant and the granted one both shown.

use super::*;
use std::path::PathBuf;

/// One empty fixture folder for a control.
fn fixture(name: &str) -> PathBuf {
    return crate::test_support::fixture_with_prefix("native-writer-probe", name);
}

/// A file with a line in it,
/// in a fresh fixture folder.
fn planted(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let file: PathBuf = root.join("log");
    std::fs::write(file.as_path(), "line\n").expect("planted file");
    return (root, file);
}

/// A file with a writer open is reported held,
/// and the same file with the writer closed is reported free:
/// the positive control
///  and the negative control of one probe.
#[test]
fn a_file_with_a_writer_open_is_held_and_the_same_file_after_it_closed_is_free() {
    let (root, file) = planted("writer");
    let writer = std::fs::OpenOptions::new().append(true).open(file.as_path()).expect("writer");
    assert_eq!(writer_check(file.as_path()).expect("probe with a writer"), WriterCheck::WriterHolds);
    drop(writer);
    assert_eq!(writer_check(file.as_path()).expect("probe after the writer closed"), WriterCheck::NoWriter);
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// Programs that only read the file do not hold it:
/// a reader never keeps a moved log from being removed.
#[test]
fn readers_do_not_hold_a_file() {
    let (root, file) = planted("readers");
    let first = std::fs::File::open(file.as_path()).expect("first reader");
    let second = std::fs::File::open(file.as_path()).expect("second reader");
    assert_eq!(writer_check(file.as_path()).expect("probe"), WriterCheck::NoWriter);
    drop((first, second));
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// A probe leaves nothing held:
/// the file opens for writing at once afterwards,
/// and a second probe is answered the same way.
#[test]
fn a_probe_leaves_no_lease_behind() {
    let (root, file) = planted("no-lease-left");
    assert_eq!(writer_check(file.as_path()).expect("first probe"), WriterCheck::NoWriter);
    let writer = std::fs::OpenOptions::new().append(true).open(file.as_path()).expect("writer opens without waiting");
    drop(writer);
    assert_eq!(writer_check(file.as_path()).expect("second probe"), WriterCheck::NoWriter);
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// A name that does not exist and a name that is a symbolic link are errors,
/// never "no writer".
#[test]
fn a_missing_file_and_a_symbolic_link_are_errors() {
    let (root, file) = planted("not-a-file");
    let missing: std::io::Error = writer_check(root.join("absent").as_path()).expect_err("missing");
    assert_eq!(missing.kind(), std::io::ErrorKind::NotFound);
    let link: PathBuf = root.join("link");
    std::os::unix::fs::symlink(file.as_path(), link.as_path()).expect("link");
    assert!(writer_check(link.as_path()).is_err(), "a final symbolic link is not followed");
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// How long the control waits for the kernel to start a lease break after a thread opens the file:
/// the break starts inside the
///  `open` call,
/// so this is far above the time it needs and below `nextest`'s per-test stop (`.config/nextest.toml`).
const BREAK_START_CEILING: std::time::Duration = std::time::Duration::from_secs(10);

/// A writer that opens the file while a lease is held starts a lease break,
/// and the process that holds the lease survives it:
///  the signal that the kernel sends is the harmless one the probe chose,
/// not the default one that ends the process.
#[cfg(target_os = "linux")]
#[test]
fn a_writer_opening_during_the_lease_does_not_end_the_process_that_holds_it() {
    use std::os::fd::AsRawFd;
    let (root, file) = planted("break");
    let held = std::fs::File::open(file.as_path()).expect("reader");
    assert!(read_lease_granted(&held).expect("lease"), "no writer exists, so the lease is granted");
    let target: PathBuf = file.clone();
    let opener = std::thread::spawn(move || {
        return std::fs::OpenOptions::new().append(true).open(target.as_path()).expect("writer opens once the lease is given back");
    });
    let began: std::time::Instant = std::time::Instant::now();
    // SAFETY: the descriptor is open for the call, which reads or writes no memory.
    while unsafe { libc::fcntl(held.as_raw_fd(), libc::F_GETLEASE) } != libc::F_UNLCK {
        assert!(began.elapsed() < BREAK_START_CEILING, "the writer's open started no lease break");
        std::thread::yield_now();
    }
    lease_given_back(&held).expect("give back");
    drop(opener.join().expect("the opening thread finished"));
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}
