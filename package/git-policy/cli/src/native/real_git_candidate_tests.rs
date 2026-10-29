//! What:
//!  Disposable-file controls for wrapper self-exclusion.
//! Why:
//!  The wrapper must recognise itself through every link,
//!  spelling and copy,
//!  and
//!      recognise the TypeScript wrapper's launchers,
//!  without misjudging real Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await classifyCandidate(linkToSelf, self)).toBe('wrapper');
//! ```
#![cfg(unix)]

/// Import the classification under test and shared fixtures.
use super::{CandidateKind, MAX_SCRIPT_INSPECTION_BYTES, classify_candidate};
use crate::test_support::{REAL_GIT, executable, fixture, remove};
use std::path::{Path, PathBuf};

/// Classify a candidate created with the given content against a fixed own executable.
fn classify_content(root: &Path, name: &str, content: &[u8]) -> CandidateKind {
    let candidate: PathBuf = root.join(name);
    executable(candidate.as_path(), content);
    return classify_candidate(candidate.as_path(), root.join("own").as_path());
}

/// This executable is a wrapper through every link,
///  relative spelling and byte-identical copy.
#[test]
fn own_executable_is_a_wrapper_through_links_and_copies() {
    let root: PathBuf = fixture("classify-self");
    let own: PathBuf = root.join("own");
    executable(own.as_path(), b"\x7fELF native wrapper bytes");
    let symlink: PathBuf = root.join("symlink");
    std::os::unix::fs::symlink(&own, &symlink).expect("symlink");
    let hardlink: PathBuf = root.join("hardlink");
    std::fs::hard_link(&own, &hardlink).expect("hard link");
    for candidate in [&own, &symlink, &hardlink] {
        assert_eq!(
            classify_candidate(candidate.as_path(), own.as_path()),
            CandidateKind::Wrapper,
            "{candidate:?}"
        );
    }
    assert_eq!(
        classify_content(root.as_path(), "copy", b"\x7fELF native wrapper bytes"),
        CandidateKind::Wrapper
    );
    // Same length, one different byte: a different native executable.
    assert_eq!(
        classify_content(root.as_path(), "sibling", b"\x7fELF native wrapper byteS"),
        CandidateKind::RealGit
    );
    remove(root.as_path());
}

/// Native executables and marker-free scripts are Git;
///  real Git 2.56.0 itself is Git.
#[test]
fn other_executables_are_real_git() {
    let root: PathBuf = fixture("classify-real");
    executable(root.join("own").as_path(), b"\x7fELF own");
    assert_eq!(
        classify_candidate(Path::new(REAL_GIT), root.join("own").as_path()),
        CandidateKind::RealGit
    );
    for (name, content) in [
        ("elf", b"\x7fELF other".as_slice()),
        ("pe", b"MZ"),
        ("script", b"#!/bin/sh\nexec /usr/bin/git \"$@\"\n"),
        ("empty", b""),
        ("short", b"#!"),
    ] {
        assert_eq!(
            classify_content(root.as_path(), name, content),
            CandidateKind::RealGit,
            "{name}"
        );
    }
    // A native executable is never searched for markers, whatever follows its header.
    assert_eq!(
        classify_content(
            root.as_path(),
            "native-with-marker-text",
            b"\x7fELF @monochromatic-dev/git-policy-cli"
        ),
        CandidateKind::RealGit
    );
    remove(root.as_path());
}

/// Scripts that start the TypeScript wrapper are wrappers,
///  wherever the marker sits within the bound.
#[test]
fn typescript_wrapper_launchers_are_wrappers() {
    let root: PathBuf = fixture("classify-shim");
    executable(root.join("own").as_path(), b"\x7fELF own");
    assert_eq!(
        classify_content(
            root.as_path(),
            "pnpm-shim",
            b"#!/bin/sh\nexec node \"$basedir/../@monochromatic-dev/git-policy-cli/dist/final/node/index.mjs\" \"$@\"\n"
        ),
        CandidateKind::Wrapper
    );
    assert_eq!(
        classify_content(
            root.as_path(),
            "cmd-shim",
            b"@ECHO off\r\nnode \"%~dp0\\..\\@monochromatic-dev\\git-policy-cli\\dist\\final\\node\\index.mjs\" %*\r\n"
        ),
        CandidateKind::Wrapper
    );
    // The marker begins at byte zero, spanning the four header bytes.
    assert_eq!(
        classify_content(
            root.as_path(),
            "marker-first",
            b"@monochromatic-dev/git-policy-cli"
        ),
        CandidateKind::Wrapper
    );
    // The marker ends exactly at the inspection bound.
    let marker: &[u8] = b"package/git-policy/cli/dist/final/node/index.mjs";
    let mut at_bound: Vec<u8> = vec![b'#'; MAX_SCRIPT_INSPECTION_BYTES - marker.len()];
    at_bound.extend_from_slice(marker);
    assert_eq!(at_bound.len(), MAX_SCRIPT_INSPECTION_BYTES);
    assert_eq!(
        classify_content(root.as_path(), "marker-last", at_bound.as_slice()),
        CandidateKind::Wrapper
    );
    remove(root.as_path());
}

/// The script inspection bound is exact:
///  at the bound is inspected,
///  one byte more is unusable.
#[test]
fn script_inspection_bound_is_exact() {
    let root: PathBuf = fixture("classify-bound");
    executable(root.join("own").as_path(), b"\x7fELF own");
    let at_bound: Vec<u8> = vec![b'#'; MAX_SCRIPT_INSPECTION_BYTES];
    assert_eq!(
        classify_content(root.as_path(), "at-bound", at_bound.as_slice()),
        CandidateKind::RealGit
    );
    let mut over_bound: Vec<u8> = vec![b'#'; MAX_SCRIPT_INSPECTION_BYTES + 1];
    assert_eq!(
        classify_content(root.as_path(), "over-bound", over_bound.as_slice()),
        CandidateKind::Unusable
    );
    // An oversized script is unusable even when it carries a marker within the bound.
    over_bound[..33].copy_from_slice(b"@monochromatic-dev/git-policy-cli");
    assert_eq!(
        classify_content(root.as_path(), "over-bound-marker", over_bound.as_slice()),
        CandidateKind::Unusable
    );
    // A large native executable is not subject to the script bound.
    let mut native: Vec<u8> = vec![0; MAX_SCRIPT_INSPECTION_BYTES * 2];
    native[..4].copy_from_slice(b"\x7fELF");
    assert_eq!(
        classify_content(root.as_path(), "large-native", native.as_slice()),
        CandidateKind::RealGit
    );
    remove(root.as_path());
}

/// The bound is the incumbent's 64 KiB (`package/git-executable/core/src/self-shim.ts`):
///  a launcher
/// padded to just under it,
///  with its marker in the last bytes,
///  is still recognised.
#[test]
fn script_inspection_bound_matches_the_incumbent() {
    assert_eq!(MAX_SCRIPT_INSPECTION_BYTES, 65_536);
    let root: PathBuf = fixture("classify-bound-value");
    executable(root.join("own").as_path(), b"\x7fELF own");
    let marker: &[u8] = b"@monochromatic-dev/git-policy-cli";
    let mut launcher: Vec<u8> = b"#!/bin/sh\n".to_vec();
    launcher.resize(65_536 - marker.len(), b'#');
    launcher.extend_from_slice(marker);
    assert_eq!(launcher.len(), 65_536);
    assert_eq!(
        classify_content(root.as_path(), "padded-launcher", launcher.as_slice()),
        CandidateKind::Wrapper
    );
    remove(root.as_path());
}

/// Entries that cannot be run as a file are unusable,
///  and a named pipe never blocks classification.
#[test]
fn unusable_candidates_are_skipped() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("classify-unusable");
    let own: PathBuf = root.join("own");
    executable(own.as_path(), b"\x7fELF own");
    assert_eq!(
        classify_candidate(root.join("missing").as_path(), own.as_path()),
        CandidateKind::Unusable
    );
    let directory: PathBuf = root.join("directory");
    std::fs::create_dir(&directory).expect("directory");
    assert_eq!(
        classify_candidate(directory.as_path(), own.as_path()),
        CandidateKind::Unusable
    );
    let plain: PathBuf = root.join("not-executable");
    std::fs::write(&plain, b"\x7fELF other").expect("plain file");
    std::fs::set_permissions(&plain, std::fs::Permissions::from_mode(0o644)).expect("mode");
    assert_eq!(
        classify_candidate(plain.as_path(), own.as_path()),
        CandidateKind::Unusable
    );
    // Any single execute bit makes a file a candidate.
    for mode in [0o100, 0o010, 0o001] {
        let path: PathBuf = root.join(format!("mode-{mode:o}"));
        std::fs::write(&path, b"\x7fELF other").expect("file");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644 | mode))
            .expect("mode");
        assert_eq!(
            classify_candidate(path.as_path(), own.as_path()),
            CandidateKind::RealGit,
            "{mode:o}"
        );
    }
    let dangling: PathBuf = root.join("dangling");
    std::os::unix::fs::symlink(root.join("missing"), &dangling).expect("dangling symlink");
    assert_eq!(
        classify_candidate(dangling.as_path(), own.as_path()),
        CandidateKind::Unusable
    );
    let unreadable: PathBuf = root.join("unreadable");
    std::fs::write(&unreadable, b"#!/bin/sh\n").expect("file");
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o111)).expect("mode");
    assert_eq!(
        classify_candidate(unreadable.as_path(), own.as_path()),
        CandidateKind::Unusable
    );
    let pipe: PathBuf = root.join("pipe");
    let made = std::process::Command::new("mkfifo")
        .arg(&pipe)
        .status()
        .expect("mkfifo");
    assert!(made.success());
    std::fs::set_permissions(&pipe, std::fs::Permissions::from_mode(0o755)).expect("mode");
    assert_eq!(
        classify_candidate(pipe.as_path(), own.as_path()),
        CandidateKind::Unusable
    );
    remove(root.as_path());
}
