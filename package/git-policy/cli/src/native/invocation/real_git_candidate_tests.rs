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
use super::{CandidateKind, MAX_SCRIPT_INSPECTION_BYTES, classify_candidate, is_wrapper_launcher, record_names_this_wrapper};
use crate::test_support::{REAL_GIT, executable, fixture, remove};
use crate::wrapper_names_old_and_new::PROGRAM;
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

/// A launcher padded to the last byte under the bound,
///  with its marker in the last bytes,
///  is still recognised:
///  the case is sized from the constant, so the bound may move without breaking it.
#[test]
fn script_inspection_bound_matches_the_incumbent() {
    let root: PathBuf = fixture("classify-bound-value");
    executable(root.join("own").as_path(), b"\x7fELF own");
    let marker: &[u8] = b"@monochromatic-dev/git-policy-cli";
    let mut launcher: Vec<u8> = b"#!/bin/sh\n".to_vec();
    launcher.resize(MAX_SCRIPT_INSPECTION_BYTES - marker.len(), b'#');
    launcher.extend_from_slice(marker);
    assert_eq!(launcher.len(), MAX_SCRIPT_INSPECTION_BYTES);
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

/// The record of a launcher of this wrapper, as `mpm-shadow` writes it, for a tool id and a `real` path.
fn record(tool_id: &str, real: &Path) -> String {
    return format!(
        "{{\"toolId\":\"{tool_id}\",\"lockFile\":\"/nonexistent/toolchain.lock.json\",\"real\":\"{}\",\"scope\":\"repo\"}}",
        real.display()
    );
}

/// A native executable with a launch record naming this wrapper's tool is a wrapper, through its own name and through a
///  symlink to it; no process is started, so the native branch is shown without running a launcher.
#[test]
fn a_native_launcher_with_a_record_of_this_wrapper_is_a_wrapper() {
    let root: PathBuf = fixture("classify-launcher");
    let own: PathBuf = root.join("own");
    executable(own.as_path(), b"\x7fELF native wrapper bytes");
    let launcher: PathBuf = root.join("cli-git-native");
    executable(launcher.as_path(), b"\x7fELF native launcher bytes, a different file");
    std::fs::write(root.join("cli-git-native.launch"), record("cargo:monochromatic-git-policy-cli", Path::new("/elsewhere/release/cli-git-native")))
        .expect("record");
    assert_eq!(classify_candidate(launcher.as_path(), own.as_path()), CandidateKind::Wrapper, "the launcher itself");
    let link: PathBuf = root.join("git");
    std::os::unix::fs::symlink(&launcher, &link).expect("symlink");
    assert_eq!(classify_candidate(link.as_path(), own.as_path()), CandidateKind::Wrapper, "a symlink named git");
    let chain: PathBuf = root.join("chain-git");
    std::os::unix::fs::symlink(&link, &chain).expect("second symlink");
    assert_eq!(classify_candidate(chain.as_path(), own.as_path()), CandidateKind::Wrapper, "a chain of symlinks");
    // The record is also found beside the candidate's own name: a launcher copied under the name `git` with its record.
    let named: PathBuf = root.join("named");
    std::fs::create_dir(&named).expect("directory");
    executable(named.join("git").as_path(), b"\x7fELF native launcher bytes, another copy");
    std::fs::write(named.join("git.launch"), record("cargo:monochromatic-git-policy-cli", Path::new("/elsewhere"))).expect("record");
    assert_eq!(classify_candidate(named.join("git").as_path(), own.as_path()), CandidateKind::Wrapper, "a record beside the name");
    remove(root.as_path());
}

/// A launcher is a wrapper whichever spelling of the program's name it carries:
///  the current name and the name an older install gave the file, each with the record beside it.
#[test]
fn a_native_launcher_named_like_either_spelling_of_the_program_is_a_wrapper() {
    for program in [PROGRAM.current, PROGRAM.earlier] {
        let root: PathBuf = fixture(format!("classify-launcher-named-{program}").as_str());
        let own: PathBuf = root.join("own");
        executable(own.as_path(), b"\x7fELF native wrapper bytes");
        let launcher: PathBuf = root.join(program);
        executable(launcher.as_path(), b"\x7fELF native launcher bytes, a different file");
        std::fs::write(root.join(format!("{program}.launch")), record("cargo:monochromatic-git-policy-cli", Path::new("/elsewhere/release"))).expect("record");
        assert_eq!(classify_candidate(launcher.as_path(), own.as_path()), CandidateKind::Wrapper, "{program}");
        remove(root.as_path());
    }
}

/// A record decides by the tool it names or by the file it leads to; any other record, or none, leaves real Git alone.
#[test]
fn a_launcher_of_another_tool_or_a_broken_record_is_real_git() {
    let root: PathBuf = fixture("classify-launcher-others");
    let own: PathBuf = root.join("own");
    executable(own.as_path(), b"\x7fELF native wrapper bytes");
    // A launcher of some other tool that leads to a real Git is Git.
    let other: PathBuf = root.join("other-git");
    executable(other.as_path(), b"\x7fELF launcher of a Git from a toolchain");
    std::fs::write(root.join("other-git.launch"), record("cargo:some-other-tool", Path::new(REAL_GIT))).expect("record");
    assert_eq!(classify_candidate(other.as_path(), own.as_path()), CandidateKind::RealGit);
    // Whatever the tool id says, a record whose `real` is this very executable names this wrapper.
    let leads: PathBuf = root.join("leads-here");
    executable(leads.as_path(), b"\x7fELF launcher of something that leads here");
    std::fs::write(root.join("leads-here.launch"), record("cargo:some-other-tool", own.as_path())).expect("record");
    assert_eq!(classify_candidate(leads.as_path(), own.as_path()), CandidateKind::Wrapper);
    // A record that is not JSON, or has no tool, names nothing.
    let broken: PathBuf = root.join("broken");
    executable(broken.as_path(), b"\x7fELF candidate with a broken record");
    std::fs::write(root.join("broken.launch"), b"not json at all").expect("record");
    assert_eq!(classify_candidate(broken.as_path(), own.as_path()), CandidateKind::RealGit);
    assert!(!record_names_this_wrapper(b"{}", Some(own.as_path())));
    assert!(!record_names_this_wrapper(b"{\"toolId\":5,\"real\":7}", Some(own.as_path())));
    // Without an own executable (the service) only the tool id can decide.
    assert!(record_names_this_wrapper(record("cargo:monochromatic-git-policy-cli", Path::new("/x")).as_bytes(), None));
    assert!(!record_names_this_wrapper(record("cargo:some-other-tool", own.as_path()).as_bytes(), None));
    // The tool id may carry any prefix before the colon; the crate name after the last colon decides.
    assert!(record_names_this_wrapper(record("npm:monochromatic-git-policy-cli", Path::new("/x")).as_bytes(), None));
    assert!(!record_names_this_wrapper(record("cargo:monochromatic-git-policy-cli-extra", Path::new("/x")).as_bytes(), None));
    // Real Git and a missing file have no record.
    assert!(!is_wrapper_launcher(Path::new(REAL_GIT), Some(own.as_path())));
    assert!(!is_wrapper_launcher(root.join("missing").as_path(), None));
    remove(root.as_path());
}
