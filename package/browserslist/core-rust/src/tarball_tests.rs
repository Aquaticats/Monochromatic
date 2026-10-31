//! What:
//!  Tests for in-memory tarball extraction: found, missing, duplicated, linked and corrupt entries.
//! Why:
//!  Downloaded archives are untrusted until their digest matches, but even a matching archive is read defensively.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('extractFiles', () => { /* success and refusals */ });
//! ```

use crate::fixture::{build_symlink_tgz, build_tgz};
use crate::tarball::extract_files;

/// Wanted files come back in the order requested, ignoring unrelated entries.
#[test]
fn extracts_in_requested_order() {
    let tgz = build_tgz(&[("package/a.txt", b"A"), ("package/skip.txt", b"S"), ("package/data/b.json", b"BB")]);
    let files = extract_files("pkg", &tgz, &["package/data/b.json", "package/a.txt"]).expect("extracts");
    assert_eq!(files, vec![b"BB".to_vec(), b"A".to_vec()]);
}

/// Names must match exactly: a prefix, a suffix, or a path elsewhere in the archive does not count.
#[test]
fn matches_paths_exactly() {
    let tgz = build_tgz(&[("package/a.txt.bak", b"x"), ("other/package/a.txt", b"y"), ("a.txt", b"z")]);
    let message = extract_files("pkg", &tgz, &["package/a.txt"]).expect_err("missing").to_string();
    assert_eq!(message, "pkg tarball: package/a.txt is missing");
}

/// A wanted path that appears twice is refused.
#[test]
fn rejects_duplicates() {
    let tgz = build_tgz(&[("package/a.txt", b"1"), ("package/a.txt", b"2")]);
    let message = extract_files("pkg", &tgz, &["package/a.txt"]).expect_err("duplicate").to_string();
    assert_eq!(message.contains("appears more than once"), true, "{message}");
}

/// A wanted path that is a symbolic link is refused rather than followed.
#[test]
fn rejects_links() {
    let tgz = build_symlink_tgz("package/a.txt");
    let message = extract_files("pkg", &tgz, &["package/a.txt"]).expect_err("link").to_string();
    assert_eq!(message.contains("not a regular file"), true, "{message}");
}

/// Bytes that are not gzip, and a truncated archive, fail with the tarball named.
#[test]
fn rejects_corrupt_input() {
    assert_eq!(extract_files("pkg", b"not gzip at all", &["x"]).is_err(), true);
    let mut tgz = build_tgz(&[("package/a.txt", &[7_u8; 4096])]);
    tgz.truncate(tgz.len() / 2);
    let message = extract_files("pkg", &tgz, &["package/a.txt"]).expect_err("truncated").to_string();
    assert_eq!(message.starts_with("pkg tarball: "), true, "{message}");
}

/// An empty request over a valid archive succeeds with nothing.
#[test]
fn empty_request() {
    let tgz = build_tgz(&[("package/a.txt", b"1")]);
    assert_eq!(extract_files("pkg", &tgz, &[]).expect("extracts"), Vec::<Vec<u8>>::new());
}
