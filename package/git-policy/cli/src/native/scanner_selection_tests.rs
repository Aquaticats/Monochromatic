//! What:
//!  Controls for rules-file selection and candidate eligibility.
//! Why:
//!  A wrong rules path scans with the wrong rules or none,
//!  and a wrong eligibility
//!      decision either skips a file that must be checked or scans a rule source
//!      against itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(rulesSource(undefined, '/repo')).toEqual({ path: '/repo/forbidden-strings.local.txt', explicit: false });
//! ```
#![cfg(unix)]

/// Import the functions under test and the values they judge.
use super::{
    DEFAULT_RULES_FILE, RULES_VARIABLE, SCANNER_SELF_MATCH_PATHS, is_scannable,
    rules_candidate_path, rules_source,
};
use crate::candidate_object::{CandidateMode, parse_object_id};
use crate::candidate_record::CandidateChange;
use crate::candidate_version::{Candidate, CandidateIdentity};
use crate::scanner_adapter::RulesSource;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

/// One candidate with a pathname,
///  a change kind and a mode.
fn candidate(path: &[u8], change: CandidateChange, mode: CandidateMode) -> Candidate {
    return Candidate {
        identity: CandidateIdentity {
            generation: 0,
            index: 0,
        },
        path: path.to_vec(),
        mode,
        change,
        object: parse_object_id(&[b'a'; 40]),
    };
}

/// The names shared with the standalone scanner are exactly its own.
#[test]
fn names_match_the_standalone_scanner() {
    assert_eq!(RULES_VARIABLE, "FORBIDDEN_STRINGS_RULES");
    assert_eq!(DEFAULT_RULES_FILE, "forbidden-strings.local.txt");
    assert_eq!(
        SCANNER_SELF_MATCH_PATHS,
        [
            b"package/forbidden-strings/cli/data/betterleaks-default-config.toml".as_slice(),
            b"package/forbidden-strings/cli/data/builtin-rules.txt",
            b"package/forbidden-strings/cli/src/port-betterleaks-relaxations.ts",
        ]
    );
}

/// The configured file first,
///  then the variable,
///  then the default file in the root,
///  which alone is tolerated when missing;
///  every name is resolved against the root.
#[test]
fn rules_file_precedence_matches_the_standalone_scanner() {
    let root: &Path = Path::new("/repo/root");
    assert_eq!(
        rules_source(None, None, root),
        RulesSource {
            path: PathBuf::from("/repo/root/forbidden-strings.local.txt"),
            explicit: false,
        }
    );
    assert_eq!(
        rules_source(None, Some(OsStr::new("rules/own.txt")), root),
        RulesSource {
            path: PathBuf::from("/repo/root/rules/own.txt"),
            explicit: true,
        }
    );
    // An absolute setting is used as it is.
    assert_eq!(
        rules_source(None, Some(OsStr::new("/elsewhere/rules.txt")), root),
        RulesSource {
            path: PathBuf::from("/elsewhere/rules.txt"),
            explicit: true,
        }
    );
    // A setting that is not UTF-8 is still a path, and an empty one is still explicit.
    let bytes: OsString = OsString::from_vec(b"r\xffules.txt".to_vec());
    assert_eq!(
        rules_source(None, Some(bytes.as_os_str()), root)
            .path
            .as_os_str()
            .as_bytes(),
        b"/repo/root/r\xffules.txt"
    );
    assert!(rules_source(None, Some(OsStr::new("")), root).explicit);
    // A configured file wins over the variable and is explicit; it is joined to the root.
    assert_eq!(
        rules_source(
            Some(".cache/rules.txt"),
            Some(OsStr::new("/elsewhere/rules.txt")),
            root
        ),
        RulesSource {
            path: PathBuf::from("/repo/root/.cache/rules.txt"),
            explicit: true,
        }
    );
    assert_eq!(
        rules_source(Some("configured.txt"), None, root),
        RulesSource {
            path: PathBuf::from("/repo/root/configured.txt"),
            explicit: true,
        }
    );
}

/// A rules file inside the repository has a candidate pathname;
///  one outside,
///  or the root itself,
///  has none.
#[test]
fn rules_candidate_path_is_the_repository_relative_name() {
    let root: &Path = Path::new("/repo/root");
    for (rules, expected) in [
        (
            "/repo/root/forbidden-strings.local.txt",
            Some(b"forbidden-strings.local.txt".as_slice()),
        ),
        (
            "/repo/root/config/rules.txt",
            Some(b"config/rules.txt".as_slice()),
        ),
        (
            "/repo/root/./config/../rules.txt",
            Some(b"rules.txt".as_slice()),
        ),
        ("/repo/root/a/b/../../c.txt", Some(b"c.txt".as_slice())),
        (
            "/repo/root/../root/again.txt",
            Some(b"again.txt".as_slice()),
        ),
        ("/repo/root", None),
        ("/repo/root/.", None),
        ("/repo/root/sub/..", None),
        ("/repo/root/../outside.txt", None),
        ("/repo/rootless/rules.txt", None),
        ("/repo", None),
        ("/elsewhere/rules.txt", None),
        ("/../../repo/root/rules.txt", Some(b"rules.txt".as_slice())),
    ] {
        assert_eq!(
            rules_candidate_path(Path::new(rules), root).as_deref(),
            expected,
            "{rules}"
        );
    }
    // Pathname bytes are carried exactly.
    let rules: PathBuf = PathBuf::from(OsString::from_vec(b"/repo/root/d\xff/r\xfe.txt".to_vec()));
    assert_eq!(
        rules_candidate_path(rules.as_path(), root).as_deref(),
        Some(b"d\xff/r\xfe.txt".as_slice())
    );
}

/// Deleted paths,
///  the rule sources and the rules file are not scanned;
///  everything else is,
///  whatever its mode.
#[test]
fn eligibility_excludes_only_deletions_and_rule_sources() {
    let rules: &[u8] = b"config/rules.txt";
    for mode in [
        CandidateMode::Regular,
        CandidateMode::Executable,
        CandidateMode::Symlink,
        CandidateMode::Gitlink,
    ] {
        for change in [CandidateChange::Added, CandidateChange::Modified] {
            assert!(is_scannable(
                &candidate(b"src/file.rs", change, mode),
                Some(rules)
            ));
            assert!(is_scannable(&candidate(b"src/file.rs", change, mode), None));
        }
        assert!(!is_scannable(
            &candidate(b"src/file.rs", CandidateChange::Deleted, mode),
            None
        ));
    }
    for path in SCANNER_SELF_MATCH_PATHS {
        assert!(!is_scannable(
            &candidate(path, CandidateChange::Modified, CandidateMode::Regular),
            None
        ));
    }
    let own: Candidate = candidate(rules, CandidateChange::Modified, CandidateMode::Regular);
    assert!(!is_scannable(&own, Some(rules)));
    // The same pathname is scanned when it is not the rules file, and near misses of every exclusion are scanned.
    assert!(is_scannable(&own, None));
    assert!(is_scannable(&own, Some(b"config/rules.txt.bak")));
    for path in [
        b"config/rules.tx".as_slice(),
        b"other/config/rules.txt",
        b"package/forbidden-strings/cli/data/builtin-rules.txt.orig",
        b"vendor/package/forbidden-strings/cli/data/builtin-rules.txt",
        b"package/forbidden-strings/cli/data",
    ] {
        assert!(
            is_scannable(
                &candidate(path, CandidateChange::Added, CandidateMode::Regular),
                Some(rules)
            ),
            "{:?}",
            String::from_utf8_lossy(path)
        );
    }
}
