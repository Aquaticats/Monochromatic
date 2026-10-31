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
    rules_candidate_path, rules_candidate_paths, rules_source,
};
use crate::candidate_object::{CandidateMode, parse_object_id};
use crate::config_schema::RulesFileEntry;
use forbidden_strings::{RulesEntry, parse_rules_list};
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

/// One entry for a path and an optional flag.
fn entry(path: &str, optional: bool) -> RulesEntry {
    return RulesEntry { path: PathBuf::from(path), optional };
}

/// One configured file for a path and an optional flag.
fn configured(path: &str, optional: bool) -> RulesFileEntry {
    return RulesFileEntry { path: String::from(path), optional };
}

/// The source of a variable or default, which cannot fail for a value that parses.
fn source(variable: Option<&OsStr>, root: &Path) -> RulesSource {
    return rules_source(None, variable, root).expect("a valid source");
}

/// The configured files first,
///  then the variable,
///  then the default file in the root,
///  which alone is tolerated when missing;
///  every name is resolved against the root.
#[test]
fn rules_file_precedence_matches_the_standalone_scanner() {
    let root: &Path = Path::new("/repo/root");
    assert_eq!(
        source(None, root),
        RulesSource {
            entries: vec![entry("/repo/root/forbidden-strings.local.txt", false)],
            explicit: false,
        }
    );
    assert_eq!(
        source(Some(OsStr::new("rules/own.txt")), root),
        RulesSource {
            entries: vec![entry("/repo/root/rules/own.txt", false)],
            explicit: true,
        }
    );
    // An absolute setting is used as it is.
    assert_eq!(
        source(Some(OsStr::new("/elsewhere/rules.txt")), root),
        RulesSource {
            entries: vec![entry("/elsewhere/rules.txt", false)],
            explicit: true,
        }
    );
    // A configured file wins over the variable and is explicit; it is joined to the root.
    assert_eq!(
        rules_source(
            Some([configured(".cache/rules.txt", false)].as_slice()),
            Some(OsStr::new("/elsewhere/rules.txt")),
            root
        ),
        Ok(RulesSource {
            entries: vec![entry("/repo/root/.cache/rules.txt", false)],
            explicit: true,
        })
    );
}

/// The variable is a rules list in the scanner's grammar:
///  comma-separated, a trailing `?` marking an optional entry,
///  each entry resolved against the root and an absolute entry kept as it is.
#[test]
fn the_variable_is_read_as_the_scanners_rules_list() {
    let root: &Path = Path::new("/repo/root");
    assert_eq!(
        source(Some(OsStr::new("shared.txt,/abs/local.txt?,sub/x.txt")), root),
        RulesSource {
            entries: vec![
                entry("/repo/root/shared.txt", false),
                entry("/abs/local.txt", true),
                entry("/repo/root/sub/x.txt", false),
            ],
            explicit: true,
        }
    );
    // The same value yields the same entries as the scanner's own parser, so the two tools cannot drift.
    let value: &str = "a.txt?,b.txt,c??";
    let parsed: Vec<RulesEntry> = parse_rules_list(value).expect("valid list");
    let joined: Vec<RulesEntry> = source(Some(OsStr::new(value)), root).entries;
    assert_eq!(joined.len(), parsed.len());
    for (resolved, original) in joined.iter().zip(&parsed) {
        assert_eq!(resolved.optional, original.optional);
        assert_eq!(resolved.path, root.join(&original.path));
    }
    // A value without a comma or a trailing question mark is one required file, as it was before lists existed.
    assert_eq!(source(Some(OsStr::new("odd?name.txt")), root).entries, vec![entry("/repo/root/odd?name.txt", false)]);
}

/// A malformed variable is a refusal, not a guess.
#[test]
fn a_malformed_variable_is_refused() {
    let root: &Path = Path::new("/repo/root");
    for value in ["", ",", "a,,b", "a,", ",a", "?", "a.txt,a.txt?"] {
        assert!(rules_source(None, Some(OsStr::new(value)), root).is_err(), "{value:?}");
    }
    // The configuration is never parsed as a list, so a comma in a configured path stays in the path.
    let configured_commas: Result<RulesSource, _> =
        rules_source(Some([configured("dir/a,b.txt", false)].as_slice()), None, root);
    assert_eq!(
        configured_commas.expect("configured path").entries,
        vec![entry("/repo/root/dir/a,b.txt", false)]
    );
}

/// A setting that is not UTF-8 cannot be split, so it is still one required path.
#[test]
fn a_variable_that_is_not_utf8_is_one_required_path() {
    let root: &Path = Path::new("/repo/root");
    let bytes: OsString = OsString::from_vec(b"r\xffu,les.txt?".to_vec());
    let selected: RulesSource = source(Some(bytes.as_os_str()), root);
    assert_eq!(selected.entries.len(), 1);
    assert!(!selected.entries[0].optional);
    assert_eq!(selected.entries[0].path.as_os_str().as_bytes(), b"/repo/root/r\xffu,les.txt?");
    assert!(selected.explicit);
}

/// Configured files keep their order and their optional flags.
#[test]
fn configured_files_keep_order_and_flags() {
    let root: &Path = Path::new("/repo/root");
    assert_eq!(
        rules_source(
            Some([configured("shared.txt", false), configured("local.txt", true)].as_slice()),
            None,
            root
        ),
        Ok(RulesSource {
            entries: vec![entry("/repo/root/shared.txt", false), entry("/repo/root/local.txt", true)],
            explicit: true,
        })
    );
}

/// Every listed file inside the repository is excluded from the scan, not only the first.
#[test]
fn every_rules_file_inside_the_repository_has_a_candidate_pathname() {
    let root: &Path = Path::new("/repo/root");
    let rules: RulesSource = RulesSource {
        entries: vec![
            entry("/repo/root/shared.txt", false),
            entry("/elsewhere/outside.txt", false),
            entry("/repo/root/dir/local.txt", true),
        ],
        explicit: true,
    };
    assert_eq!(
        rules_candidate_paths(&rules, root),
        vec![b"shared.txt".to_vec(), b"dir/local.txt".to_vec()]
    );
    assert!(rules_candidate_paths(&RulesSource { entries: Vec::new(), explicit: true }, root).is_empty());
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
    let named: Vec<Vec<u8>> = vec![rules.to_vec()];
    for mode in [
        CandidateMode::Regular,
        CandidateMode::Executable,
        CandidateMode::Symlink,
        CandidateMode::Gitlink,
    ] {
        for change in [CandidateChange::Added, CandidateChange::Modified] {
            assert!(is_scannable(
                &candidate(b"src/file.rs", change, mode),
                named.as_slice()
            ));
            assert!(is_scannable(&candidate(b"src/file.rs", change, mode), &[]));
        }
        assert!(!is_scannable(
            &candidate(b"src/file.rs", CandidateChange::Deleted, mode),
            &[]
        ));
    }
    for path in SCANNER_SELF_MATCH_PATHS {
        assert!(!is_scannable(
            &candidate(path, CandidateChange::Modified, CandidateMode::Regular),
            &[]
        ));
    }
    let own: Candidate = candidate(rules, CandidateChange::Modified, CandidateMode::Regular);
    assert!(!is_scannable(&own, named.as_slice()));
    // Any of several rules files is excluded, not only the first.
    assert!(!is_scannable(&own, &[b"shared.txt".to_vec(), rules.to_vec()]));
    // The same pathname is scanned when it is not the rules file, and near misses of every exclusion are scanned.
    assert!(is_scannable(&own, &[]));
    assert!(is_scannable(&own, &[b"config/rules.txt.bak".to_vec()]));
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
                named.as_slice()
            ),
            "{:?}",
            String::from_utf8_lossy(path)
        );
    }
}
