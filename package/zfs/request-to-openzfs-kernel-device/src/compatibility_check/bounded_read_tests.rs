//! What:
//!  The bounded read's decisions:
//! a file inside its bound reads whole,
//! one past the bound
//!  refuses,
//! a link refuses and a missing file refuses,
//! and a failure names the step
//!  the caller asked for.
//! Why:
//!  Every file the gate reads is root's,
//! so a link below `/sys` is a path this
//!  process must not follow,
//! and a file that grew must not fill this process before a
//!  byte is judged.

use super::blob_fixture::fixture_directory;
use super::bounded_read::{IDENTITY_FILE_BOUND, ReadRequest, read_bounded};
use super::refusal::{Refusal, Step};
use std::path::PathBuf;

/// A file under the build directory with the given bytes.
fn written(name: &str, file: &str, bytes: &[u8]) -> PathBuf {
    let path: PathBuf = fixture_directory(name).join(file);
    std::fs::write(&path, bytes).expect("the fixture file is written");
    return path;
}

/// The refusal one read must get.
fn refused(request: &ReadRequest<'_>) -> Refusal {
    let answer: Result<Vec<u8>, Refusal> = read_bounded(request);
    let Err(refusal): Result<Vec<u8>, Refusal> = answer else {
        panic!("the read was accepted");
    };
    return refusal;
}

#[test]
fn a_file_inside_its_bound_reads_whole() {
    let bytes: &[u8] = b"2.4.4-1\n";
    let path: PathBuf = written("bounded-read-fits", "version", bytes);
    let read: Vec<u8> = read_bounded(&ReadRequest {
        path: path.as_path(),
        bound: IDENTITY_FILE_BOUND,
        step: Step::Identity,
    })
    .expect("the file is read");
    assert_eq!(read, bytes);
}

#[test]
fn a_file_past_its_bound_refuses_naming_it() {
    let bytes: Vec<u8> = vec![0; 32];
    let path: PathBuf = written("bounded-read-past", "large", &bytes);
    let refusal: Refusal = refused(&ReadRequest {
        path: path.as_path(),
        bound: 16,
        step: Step::ModuleBtf,
    });
    assert_eq!(refusal.step, Step::ModuleBtf);
    assert!(refusal.detail.contains("16 bytes"), "{}", refusal.detail);
    assert_eq!(refusal.file.as_deref(), Some(path.as_path()));
}

#[test]
fn a_link_refuses_rather_than_being_followed() {
    let target: PathBuf = written("bounded-read-link", "target", b"ok");
    let link: PathBuf = fixture_directory("bounded-read-link").join("link");
    let _ = std::fs::remove_file(&link);
    std::os::unix::fs::symlink(&target, &link).expect("the link is made");
    let refusal: Refusal = refused(&ReadRequest {
        path: link.as_path(),
        bound: IDENTITY_FILE_BOUND,
        step: Step::Identity,
    });
    assert_eq!(refusal.step, Step::Identity);
    assert!(refusal.detail.contains("could not be opened"), "{}", refusal.detail);
}

#[test]
fn a_missing_file_refuses_naming_the_path() {
    let path: PathBuf = fixture_directory("bounded-read-missing").join("absent");
    let refusal: Refusal = refused(&ReadRequest {
        path: path.as_path(),
        bound: IDENTITY_FILE_BOUND,
        step: Step::BaseBtf,
    });
    assert_eq!(refusal.step, Step::BaseBtf);
    assert_eq!(refusal.file.as_deref(), Some(path.as_path()));
    assert!(refusal.to_string().contains("base type information refused whilst reading"));
}
