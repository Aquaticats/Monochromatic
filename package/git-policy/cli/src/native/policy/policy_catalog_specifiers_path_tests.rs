//! What:
//!  Controls for the catalog-specifier check on awkward candidates:
//!  a name that is not UTF-8,
//! a symbolic link named like a manifest,
//!  and the hint every finding kind of the shared rule carries.
//! Why:
//!  Git allows any bytes in a path and a link where a file is expected;
//!  the policy must neither panic nor let either pass,
//!  and a kind the rule adds later must fail a control here until it has a hint.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await check([Buffer.from('caf\xe9/package.json')])).toEqual([{ code: 'disallowed-specifier', pathBytes: ... }]);
//! ```
#![cfg(unix)]

use super::{add_outcome, code_and_path, findings_of, os, write};
use crate::event_hints::finding_hint;
use crate::event_path::EventPath;
use crate::policy_engine::PolicyFinding;
use crate::test_support::{fixture, remove, repository};
use monochromatic_catalog_specifier::FindingKind;
/// Raw bytes of a name.
use std::ffi::OsString;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;

/// A directory name that is not UTF-8:
///  Latin-1 `é` is the lone byte `0xE9`.
const LATIN_ONE_DIRECTORY: &[u8] = b"caf\xe9";

/// A manifest below a directory whose name is not UTF-8 is checked like any other,
///  and a finding keeps the exact bytes of the name in its pathname.
#[test]
fn a_manifest_below_a_name_that_is_not_utf8_is_still_checked() {
    let root: PathBuf = fixture("catalog-bytes");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let directory: PathBuf = repo.join(OsString::from_vec(LATIN_ONE_DIRECTORY.to_vec()));
    std::fs::create_dir(&directory).expect("directory");
    std::fs::write(directory.join("package.json"), br#"{"dependencies": {"zod": "^4.6.7"}}"#).expect("manifest");
    std::fs::write(directory.join("package.yaml"), b"name: x\n").expect("yaml");
    std::fs::write(directory.join("package.json.bak"), b"{}").expect("backup");
    let found: Vec<PolicyFinding> = findings_of(add_outcome(
        repo.as_path(),
        &[OsString::from_vec(LATIN_ONE_DIRECTORY.to_vec())],
    ));
    assert_eq!(
        code_and_path(found.as_slice()),
        vec![
            (
                "disallowed-specifier",
                EventPath::from_git_bytes(b"caf\xe9/package.json")
            ),
            (
                "unsupported-manifest-format",
                EventPath::from_git_bytes(b"caf\xe9/package.yaml")
            ),
        ]
    );
    // The exact bytes travel in the pathname; the message shows the replacement character.
    let named: EventPath = found[0].path.clone().expect("path");
    assert_eq!(named.exact(), Some(&b"caf\xe9/package.json"[..]));
    assert!(found[0].message.starts_with("caf\u{fffd}/package.json: "), "{}", found[0].message);
    // A name made only of text bytes needs no extra field.
    assert_eq!(EventPath::from_git_bytes(b"ok/package.json").exact(), None);
    assert_eq!(OsString::from_vec(LATIN_ONE_DIRECTORY.to_vec()).as_bytes(), LATIN_ONE_DIRECTORY);
    remove(root.as_path());
}

/// A symbolic link named `package.json` hides no manifest from the policy:
///  its blob is the link target,
/// which is not a JSON object,
/// so it is a finding and not a pass.
#[test]
fn a_symbolic_link_named_like_a_manifest_is_reported() {
    let root: PathBuf = fixture("catalog-link");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), "real/package.json", br#"{"dependencies": {"zod": "^4.6.7"}}"#);
    std::fs::create_dir(repo.join("linked")).expect("directory");
    std::os::unix::fs::symlink("../real/package.json", repo.join("linked/package.json")).expect("link");
    std::os::unix::fs::symlink("../real/package.json", repo.join("linked/package.yaml")).expect("yaml link");
    let found: Vec<PolicyFinding> = findings_of(add_outcome(repo.as_path(), &os(&["linked"])));
    assert_eq!(
        code_and_path(found.as_slice()),
        vec![
            ("invalid-manifest", EventPath::from_git_bytes(b"linked/package.json")),
            ("unsupported-manifest-format", EventPath::from_git_bytes(b"linked/package.yaml")),
        ]
    );
    remove(root.as_path());
}

/// The rule's kind and its spelling,
/// with an exhaustive match so a new kind stops compilation here.
fn every_kind() -> Vec<FindingKind> {
    let all: Vec<FindingKind> = vec![
        FindingKind::UnsupportedManifestFormat,
        FindingKind::InvalidManifest,
        FindingKind::MalformedSection,
        FindingKind::DuplicateKey,
        FindingKind::NonStringSpecifier,
        FindingKind::DisallowedSpecifier,
    ];
    for kind in &all {
        match kind {
            FindingKind::UnsupportedManifestFormat
            | FindingKind::InvalidManifest
            | FindingKind::MalformedSection
            | FindingKind::DuplicateKey
            | FindingKind::NonStringSpecifier
            | FindingKind::DisallowedSpecifier => {}
        }
    }
    return all;
}

/// Every kind of finding the rule has carries a hint under this policy's name,
///  and the code is the rule's kebab-case spelling.
#[test]
fn every_kind_of_the_rule_has_a_hint() {
    for kind in every_kind() {
        for fix_available in [false, true] {
            let hint: &str = finding_hint("mono/catalog-specifiers", kind.as_str(), fix_available)
                .unwrap_or_else(|| panic!("{} has no hint", kind.as_str()));
            assert!(hint.starts_with("Hint: "), "{}: {hint}", kind.as_str());
        }
    }
    assert_eq!(finding_hint("mono/catalog-specifiers", "unknown", false), None);
}
