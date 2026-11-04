//! What:
//!  `git add` bulk-staging facts in every spelling Git 2.56.0 accepts,
//!  with real-Git
//!       controls for the table and for the spellings the incumbent missed.
//! Why:
//!  The add-explicit policy is bypassed by any bulk form the parser does not see.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseAddRegion(['-A']).bulkMatches).toEqual(['-A']);
//! ```

/// The parser,
///  its table,
///  the oracles and the real-Git fixture helpers.
use super::{ADD_ESCAPE_HATCH, ADD_TABLE, AddRegion, BulkKind, BulkMatch, parse_add_region};
use crate::command_options::OptionErrorKind;
use crate::command_test_completion::{git_completion, render_completion};
use crate::command_test_support::{
    assert_table_invariants, git, os_arguments, output_text, remove, repository_with_tracked_file,
};
use std::path::{Path, PathBuf};

/// Parse a region Git accepts,
///  with no other wrapper flags.
fn region(values: &[&str]) -> AddRegion {
    return parse_add_region(os_arguments(values).as_slice(), &[]).expect("valid region");
}

/// The matches of a region as `(kind, token)` pairs.
fn matches(values: &[&str]) -> Vec<(BulkKind, usize)> {
    let mut pairs: Vec<(BulkKind, usize)> = Vec::<(BulkKind, usize)>::new();
    for found in region(values).bulk_matches {
        pairs.push((found.kind, found.token));
    }
    return pairs;
}

/// The incumbent's literal tokens:
///  `.`,
///  `./`,
///  `*`,
///  `:/`,
///  `-A`,
///  `--all`,
///  `-u`,
///  `--update`.
#[test]
fn detects_the_incumbent_bulk_tokens() {
    for (values, kind) in [
        (vec!["."], BulkKind::Pathspec),
        (vec!["./"], BulkKind::Pathspec),
        (vec!["*"], BulkKind::Pathspec),
        (vec![":/"], BulkKind::Pathspec),
        (vec!["-A"], BulkKind::AllFlag),
        (vec!["--all"], BulkKind::AllFlag),
        (vec!["-u"], BulkKind::UpdateFlag),
        (vec!["--update"], BulkKind::UpdateFlag),
    ] {
        assert_eq!(matches(values.as_slice()), vec![(kind, 0)], "{values:?}");
    }
    assert!(matches(&["file.ts", "dir/", "-v", "--", "other.ts"]).is_empty());
    // Matches come in argument order, before and after `--`.
    assert_eq!(
        matches(&["-u", ".", "-A", "--", "*", "x"]),
        vec![
            (BulkKind::UpdateFlag, 0),
            (BulkKind::Pathspec, 1),
            (BulkKind::AllFlag, 2),
            (BulkKind::Pathspec, 4)
        ]
    );
}

/// A token in a value position or after `--` is read by position,
///  as the incumbent did.
#[test]
fn reads_values_and_paths_by_position() {
    assert!(matches(&["--pathspec-from-file", "-A"]).is_empty());
    assert!(matches(&["--pathspec-from-file", "."]).is_empty());
    assert!(matches(&["--pathspec-from-file=."]).is_empty());
    assert!(matches(&["--chmod", "."]).is_empty());
    assert_eq!(
        matches(&["--chmod", "+x", "."]),
        vec![(BulkKind::Pathspec, 2)]
    );
    // After `--` the flags are file names; only bulk pathspecs still match.
    assert!(matches(&["--", "-A", "--all", "-u"]).is_empty());
    assert_eq!(matches(&["--", "-A", "."]), vec![(BulkKind::Pathspec, 2)]);
}

/// Divergence:
///  clusters,
///  abbreviations and the shared `--ignore-removal` variable.
#[test]
fn detects_bulk_flags_in_every_git_spelling() {
    for values in [
        vec!["-vA"],
        vec!["-Av"],
        vec!["-nA"],
        vec!["--al"],
        vec!["--no-ignore-removal"],
        vec!["--no-all", "-A"],
        vec!["--ignore-removal", "--no-ignore-removal"],
    ] {
        assert_eq!(
            matches(values.as_slice()),
            vec![(BulkKind::AllFlag, values.len() - 1)],
            "{values:?}"
        );
    }
    for values in [
        vec!["-fu"],
        vec!["-uv"],
        vec!["--upd"],
        vec!["--no-update", "-u"],
    ] {
        assert_eq!(
            matches(values.as_slice()),
            vec![(BulkKind::UpdateFlag, values.len() - 1)],
            "{values:?}"
        );
    }
    // Git's last option wins.
    for values in [
        vec!["-A", "--no-all"],
        vec!["-A", "--ignore-removal"],
        vec!["--no-ignore-removal", "--no-all"],
        vec!["-u", "--no-update"],
        vec!["--ignore-removal"],
    ] {
        assert!(matches(values.as_slice()).is_empty(), "{values:?}");
    }
}

/// Only the four literal pathspecs are bulk matches;
///  other magic is not,
///  as shipped.
#[cfg(unix)]
#[test]
fn matches_only_the_literal_bulk_pathspecs() {
    use crate::command_test_support::byte_argument;
    for values in [
        vec![":(top)"],
        vec![":/*"],
        vec!["**"],
        vec![".."],
        vec!["./."],
        vec![":!x"],
        vec![""],
        vec!["-"],
    ] {
        assert!(matches(values.as_slice()).is_empty(), "{values:?}");
    }
    let bytes: AddRegion =
        parse_add_region(&[byte_argument(b".\xff"), byte_argument(b"\xff")], &[]).expect("valid");
    assert!(bytes.bulk_matches.is_empty());
}

/// The escape hatch counts in option position only,
///  and `--resolved` is reported.
#[test]
fn reports_the_escape_hatch_and_resolved() {
    assert_eq!(region(&[ADD_ESCAPE_HATCH, "."]).wrapper.escape, vec![0]);
    assert_eq!(region(&[".", ADD_ESCAPE_HATCH]).wrapper.escape, vec![1]);
    for values in [
        vec!["--pathspec-from-file", ADD_ESCAPE_HATCH],
        vec!["--chmod", ADD_ESCAPE_HATCH],
        vec!["--", ADD_ESCAPE_HATCH],
        vec!["."],
    ] {
        assert!(
            region(values.as_slice()).wrapper.escape.is_empty(),
            "{values:?}"
        );
    }
    assert!(region(&["--resolved"]).resolved);
    assert!(!region(&["--resolved", "--no-resolved"]).resolved);
    assert!(region(&["--resolved"]).bulk_matches.is_empty());
    assert_eq!(
        region(&["-A"]).bulk_matches,
        vec![BulkMatch {
            kind: BulkKind::AllFlag,
            token: 0
        }]
    );
}

/// A region Git refuses is reported as refused.
#[test]
fn reports_what_git_refuses() {
    for (values, kind) in [
        (vec!["--unknown"], OptionErrorKind::UnknownOption),
        (vec!["-a"], OptionErrorKind::UnknownOption),
        (vec!["-all"], OptionErrorKind::SingleDashLongOption),
        (vec!["--pathspec-from-file"], OptionErrorKind::MissingValue),
        (vec!["--all=1"], OptionErrorKind::UnexpectedValue),
        (vec!["--i"], OptionErrorKind::AmbiguousOption),
    ] {
        assert_eq!(
            parse_add_region(os_arguments(values.as_slice()).as_slice(), &[])
                .expect_err("refused")
                .kind,
            kind,
            "{values:?}"
        );
    }
}

/// Paths staged against `HEAD`,
///  one per line.
fn staged(root: &Path) -> String {
    return output_text(&git(root, &["diff", "--cached", "--name-only"]));
}

/// The copied table matches the binary,
///  and Git stages everything for the spellings the
/// incumbent did not recognize.
#[test]
fn table_and_bulk_spellings_match_git() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("add-table");
    assert_table_invariants(ADD_TABLE);
    assert_eq!(
        render_completion(ADD_TABLE),
        git_completion(root.as_path(), &["add"])
    );
    for spelling in ["-vA", "--al", "--no-ignore-removal"] {
        std::fs::write(root.join("new.txt"), b"new\n").expect("write untracked file");
        std::fs::write(root.join("tracked.txt"), b"two\n").expect("modify tracked file");
        git(root.as_path(), &["add", spelling]);
        assert_eq!(staged(root.as_path()), "new.txt\ntracked.txt", "{spelling}");
        assert_eq!(
            matches(&[spelling]),
            vec![(BulkKind::AllFlag, 0)],
            "{spelling}"
        );
        git(root.as_path(), &["reset", "--quiet"]);
    }
    // `-A` in the value position of `--pathspec-from-file` names a file to read.
    std::fs::write(root.join("-A"), b"new.txt\n").expect("write pathspec file");
    git(root.as_path(), &["add", "--pathspec-from-file", "-A"]);
    assert_eq!(staged(root.as_path()), "new.txt");
    remove(directory.as_path());
}
