//! What:
//!  Controls for the byte and identity primitives behind wrapper self-exclusion.
//! Why:
//!  Classification is only as exact as its header,
//!  marker,
//!  identity and content
//!      comparisons,
//!  so each one is pinned at its boundaries on disposable files.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await sameFile(linkToSelf, self)).toBe(true);
//! ```
#![cfg(unix)]

/// Import the primitives under test and shared fixtures.
use super::{has_wrapper_marker, identical_content, is_native_header, same_file};
use crate::test_support::{executable, fixture, remove};
use std::path::PathBuf;

/// Every supported native signature is recognised,
///  including two-byte PE.
#[test]
fn native_headers_are_recognised_by_prefix() {
    for header in [
        b"\x7fELF".as_slice(),
        b"MZ",
        b"MZ\x90\x00",
        b"\xfe\xed\xfa\xce",
        b"\xfe\xed\xfa\xcf",
        b"\xce\xfa\xed\xfe",
        b"\xcf\xfa\xed\xfe",
        b"\xca\xfe\xba\xbe",
        b"\xbe\xba\xfe\xca",
        b"\xca\xfe\xba\xbf",
        b"\xbf\xba\xfe\xca",
    ] {
        assert!(is_native_header(header), "{header:?}");
    }
    for header in [
        b"".as_slice(),
        b"M",
        b"\x7fEL",
        b"#!/b",
        b"ELF\x7f",
        b"\xfe\xed\xfa",
        b"zM",
    ] {
        assert!(!is_native_header(header), "{header:?}");
    }
}

/// Each marker is found at the start,
///  middle and end;
///  near misses are not markers.
#[test]
fn wrapper_markers_are_found_anywhere_in_script_bytes() {
    for marker in [
        "@monochromatic-dev/git-policy-cli",
        "package/git-policy/cli/dist/final/node/index.mjs",
        "@monochromatic-dev\\git-policy-cli",
        "package\\git-policy\\cli\\dist\\final\\node\\index.mjs",
    ] {
        assert!(has_wrapper_marker(marker.as_bytes()), "{marker}");
        assert!(has_wrapper_marker(
            format!("#!/bin/sh\nexec node {marker}\n").as_bytes()
        ));
        assert!(has_wrapper_marker(format!("\u{ff}\n{marker}").as_bytes()));
        // One byte short at either end is not the marker.
        assert!(!has_wrapper_marker(&marker.as_bytes()[1..]), "{marker}");
        assert!(
            !has_wrapper_marker(&marker.as_bytes()[..marker.len() - 1]),
            "{marker}"
        );
    }
    for content in [
        "",
        "#!/bin/sh\nexec /usr/bin/git \"$@\"\n",
        "@monochromatic-dev/git-policy-api",
        "monochromatic-dev/git-policy-cli",
    ] {
        assert!(!has_wrapper_marker(content.as_bytes()), "{content:?}");
    }
}

/// The same file is recognised through symbolic links,
///  hard links and relative spellings.
#[test]
fn same_file_holds_through_links_and_spellings() {
    let root: PathBuf = fixture("same-file");
    let own: PathBuf = root.join("own");
    executable(own.as_path(), b"\x7fELF own");
    let other: PathBuf = root.join("other");
    executable(other.as_path(), b"\x7fELF own");
    let directory: PathBuf = root.join("dir");
    std::fs::create_dir(&directory).expect("directory");
    let symlink: PathBuf = directory.join("git");
    std::os::unix::fs::symlink(&own, &symlink).expect("symlink");
    let chained: PathBuf = root.join("chained");
    std::os::unix::fs::symlink(&symlink, &chained).expect("chained symlink");
    let hardlink: PathBuf = root.join("hardlink");
    std::fs::hard_link(&own, &hardlink).expect("hard link");
    let dotted: PathBuf = directory.join("..").join("own");
    for same in [&own, &symlink, &chained, &hardlink, &dotted] {
        assert!(same_file(same.as_path(), own.as_path()), "{same:?}");
        assert!(same_file(own.as_path(), same.as_path()), "{same:?}");
    }
    assert!(!same_file(other.as_path(), own.as_path()));
    assert!(!same_file(root.join("missing").as_path(), own.as_path()));
    assert!(!same_file(own.as_path(), root.join("missing").as_path()));
    assert!(!same_file(
        root.join("missing").as_path(),
        root.join("missing").as_path()
    ));
    let dangling: PathBuf = root.join("dangling");
    std::os::unix::fs::symlink(root.join("missing"), &dangling).expect("dangling symlink");
    assert!(!same_file(dangling.as_path(), own.as_path()));
    remove(root.as_path());
}

/// Only equal length and equal bytes make two files identical.
#[test]
fn identical_content_requires_equal_length_and_bytes() {
    let root: PathBuf = fixture("identical");
    let own: PathBuf = root.join("own");
    std::fs::write(&own, b"abcdef").expect("own");
    for (name, content, expected) in [
        ("copy", b"abcdef".as_slice(), true),
        ("same-length", b"abcdeX", false),
        ("first-differs", b"Xbcdef", false),
        ("shorter", b"abcde", false),
        ("longer", b"abcdefg", false),
        ("empty", b"", false),
    ] {
        let path: PathBuf = root.join(name);
        std::fs::write(&path, content).expect("candidate");
        assert_eq!(
            identical_content(path.as_path(), own.as_path()),
            expected,
            "{name}"
        );
        assert_eq!(
            identical_content(own.as_path(), path.as_path()),
            expected,
            "{name}"
        );
    }
    assert!(!identical_content(
        root.join("missing").as_path(),
        own.as_path()
    ));
    assert!(!identical_content(
        own.as_path(),
        root.join("missing").as_path()
    ));
    let empty_one: PathBuf = root.join("empty-one");
    std::fs::write(&empty_one, b"").expect("empty");
    assert!(identical_content(
        empty_one.as_path(),
        root.join("empty").as_path()
    ));
    remove(root.as_path());
}
