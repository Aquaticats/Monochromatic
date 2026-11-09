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
use super::{ADD_ESCAPE_HATCH, ADD_TABLE, AddRegion, BulkKind, BulkMatch, caller_global_prefix, literal_pathspecs_enabled, parse_add_region, stages_in_bulk};
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

/// Every spelling Git resolves to the whole worktree is a bulk match;
///  other magic and ordinary names are not.
#[cfg(unix)]
#[test]
fn matches_every_whole_tree_pathspec_spelling() {
    use crate::command_test_support::byte_argument;
    for values in [
        vec![":/*"],
        vec![":/**"],
        vec![":(top)"],
        vec!["**"],
        vec![":(glob)**"],
    ] {
        assert_eq!(
            matches(values.as_slice()),
            vec![(BulkKind::Pathspec, 0)],
            "{values:?}"
        );
    }
    for values in [
        vec![":(top)sub"],
        vec![":(literal)**"],
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

/// The caller's mode is the last literal global option it wrote,
/// and otherwise the variable's own boolean reading;
/// an unset value and one Git would refuse to read as a boolean are no mode,
/// so the rejection stands for a spelling this reading does not model.
#[test]
fn the_caller_mode_is_its_last_literal_option_then_the_variable() {
    let literal = |values: &[&str], variable: Option<&str>| -> bool {
        let prefix: Vec<std::ffi::OsString> = os_arguments(values);
        return literal_pathspecs_enabled(prefix.as_slice(), variable.map(std::ffi::OsStr::new));
    };
    assert!(!literal(&[], None));
    assert!(literal(&["--literal-pathspecs"], None));
    assert!(!literal(&["--no-literal-pathspecs"], None));
    assert!(!literal(&["--literal-pathspecs", "--no-literal-pathspecs"], None), "the later option wins");
    assert!(literal(&["--no-literal-pathspecs", "--literal-pathspecs"], None));
    assert!(literal(&["-C", "dir", "-c", "core.pager=cat", "--literal-pathspecs"], None), "other global options do not disturb it");
    for (value, expected) in [("1", true), ("true", true), ("yes", true), ("ON", true), ("0", false), ("false", false), ("", false), ("banana", false)] {
        assert_eq!(literal(&[], Some(value)), expected, "GIT_LITERAL_PATHSPECS={value:?}");
        assert!(!literal(&["--no-literal-pathspecs"], Some(value)), "an option the caller wrote overrides the variable");
    }
}

/// `caller_global_prefix` is the tokens before the subcommand word and nothing after it.
#[test]
fn the_caller_global_prefix_stops_at_the_subcommand() {
    let prefix = |values: &[&str]| -> Vec<String> {
        return caller_global_prefix(os_arguments(values).as_slice()).iter().map(|word| { return word.to_string_lossy().into_owned(); }).collect();
    };
    assert_eq!(prefix(&["add", "--", "**"]), Vec::<String>::new());
    assert_eq!(prefix(&["--literal-pathspecs", "add", "--", "**"]), vec![String::from("--literal-pathspecs")]);
    assert_eq!(prefix(&["-C", "dir", "--no-pager", "add", "-A"]), vec![String::from("-C"), String::from("dir"), String::from("--no-pager")]);
}

/// Git's own answer for the spellings under literal mode,
/// which is what `stages_in_bulk` reads:
/// each names one file,
/// and the working directory still takes everything below it.
#[test]
fn literal_mode_stages_the_literal_file_in_git() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("add-literal-mode");
    for spelling in ["*", "**", ":(top)", ":(glob)**"] {
        std::fs::write(root.join(spelling), b"literal\n").expect("write the literal file");
        git(root.as_path(), &["--literal-pathspecs", "add", "--", spelling]);
        assert_eq!(staged(root.as_path()), spelling, "{spelling}");
        git(root.as_path(), &["reset", "--quiet"]);
        std::fs::remove_file(root.join(spelling)).expect("remove the literal file");
    }
    std::fs::write(root.join("top.txt"), b"top\n").expect("write top file");
    std::fs::create_dir(root.join("sub")).expect("sub directory");
    std::fs::write(root.join("sub/below.txt"), b"below\n").expect("write below file");
    for spelling in [".", "./"] {
        git(root.as_path(), &["--literal-pathspecs", "add", "--", spelling]);
        assert_eq!(staged(root.as_path()), "sub/below.txt\ntop.txt", "{spelling}");
        git(root.as_path(), &["reset", "--quiet"]);
    }
    assert!(stages_in_bulk(b"**", true) == false && stages_in_bulk(b"**", false) && stages_in_bulk(b".", true));
    remove(directory.as_path());
}

/// Git stages the whole worktree for each whole-tree spelling the parser names,
/// at the top level and below it.
#[test]
fn every_whole_tree_spelling_stages_the_whole_worktree_in_git() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("add-whole-tree");
    std::fs::write(root.join("top.txt"), b"top\n").expect("write top file");
    std::fs::create_dir(root.join("sub")).expect("sub directory");
    std::fs::write(root.join("sub/below.txt"), b"below\n").expect("write below file");
    for spelling in [":/*", ":/**", ":(top)", "**", ":(glob)**"] {
        git(root.as_path(), &["add", "--", spelling]);
        assert_eq!(
            staged(root.as_path()),
            "sub/below.txt\ntop.txt",
            "{spelling}"
        );
        git(root.as_path(), &["reset", "--quiet"]);
    }
    remove(directory.as_path());
}
