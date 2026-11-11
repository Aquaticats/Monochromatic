//! What:
//!  `git add` bulk-staging facts in every spelling Git 2.56.0 accepts,
//!  with real-Git
//!       controls for the table,
//!  for the spellings the incumbent missed and for the selection the policy reads.
//! Why:
//!  The add-explicit policy is bypassed by any bulk form the parser does not see,
//!  and it refuses a spelling only for what Git itself says the spelling selects.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseAddRegion(['-A']).allToken).toBe(0);
//! ```

/// The parser,
///  its table,
///  the oracles and the real-Git fixture helpers.
use super::{
    ADD_ESCAPE_HATCH, ADD_TABLE, AddRegion, caller_global_prefix, is_ordinary_named_path,
    literal_pathspecs_enabled, names_no_literal_character, parse_add_region,
};
use crate::command_options::OptionErrorKind;
use crate::command_test_completion::{git_completion, render_completion};
use crate::command_test_support::{
    assert_table_invariants, byte_argument, fixture, git, git_output, os_arguments, output_text,
    remove, repository, repository_with_tracked_file,
};
/// The argument lists the policy's own queries carry.
use crate::rule_add_explicit::{selection_query_arguments, whole_worktree_query_arguments};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Parse a region Git accepts,
///  with no other wrapper flags.
fn region(values: &[&str]) -> AddRegion {
    return parse_add_region(os_arguments(values).as_slice(), &[]).expect("valid region");
}

/// The positional tokens one region must be asked about,
///  which the parser keeps in argument order.
fn questioned(values: &[&str]) -> Vec<usize> {
    return region(values).pathspec_tokens;
}

/// The incumbent's flag tokens:
///  `-A`,
///  `--all`,
///  `-u`,
///  `--update`.
#[test]
fn detects_the_incumbent_bulk_flags() {
    for values in [vec!["-A"], vec!["--all"]] {
        assert_eq!(region(values.as_slice()).all_token, Some(0), "{values:?}");
        assert!(region(values.as_slice()).update_token.is_none());
    }
    for values in [vec!["-u"], vec!["--update"]] {
        assert_eq!(region(values.as_slice()).update_token, Some(0), "{values:?}");
        assert!(region(values.as_slice()).all_token.is_none());
    }
    // A flag is found wherever the caller wrote it, before or after `--`.
    assert_eq!(region(&["file.ts", "-A"]).all_token, Some(1));
    assert_eq!(region(&["-u", "file.ts"]).update_token, Some(0));
    assert_eq!(region(&["-A", "-u"]).all_token, Some(0));
    assert_eq!(region(&["-A", "-u"]).update_token, Some(1));
}

/// A token in a value position or after `--` is read by position,
///  as the incumbent did.
#[test]
fn reads_values_and_paths_by_position() {
    assert!(questioned(&["--pathspec-from-file", "-A"]).is_empty());
    assert!(questioned(&["--pathspec-from-file", "."]).is_empty());
    assert!(questioned(&["--pathspec-from-file=."]).is_empty());
    assert!(questioned(&["--chmod", "."]).is_empty());
    assert_eq!(questioned(&["--chmod", "+x", "."]), vec![2]);
    // After `--` the flags are file names; only pathspecs are still read.
    assert!(questioned(&["--", "-A", "--all", "-u"]).is_empty());
    assert!(region(&["--", "-A", "--all", "-u"]).all_token.is_none());
    assert_eq!(questioned(&["--", "-A", "."]), vec![2]);
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
            region(values.as_slice()).all_token,
            Some(values.len() - 1),
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
            region(values.as_slice()).update_token,
            Some(values.len() - 1),
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
        let parsed: AddRegion = region(values.as_slice());
        assert!(
            parsed.all_token.is_none() && parsed.update_token.is_none(),
            "{values:?}"
        );
    }
}

/// An ordinary named path stages what it names,
///  so the region asks Git nothing about it;
///  every other spelling is a question,
///  whatever the caller meant by it.
#[test]
fn only_an_ordinary_named_path_is_no_question() {
    for spelling in ["file.ts", "dir/", ".hidden", "-", "a:b", ""] {
        assert_eq!(
            questioned(&[spelling]),
            Vec::<usize>::new(),
            "{spelling:?} names one path"
        );
    }
    assert!(is_ordinary_named_path(b"dir/"));
    assert!(is_ordinary_named_path(b"a:b"));
    assert!(!is_ordinary_named_path(b"./file"));
    assert!(!is_ordinary_named_path(b".."));
    assert!(!is_ordinary_named_path(b"sub/.."));
    assert!(!is_ordinary_named_path(b"*"));
    assert!(!is_ordinary_named_path(b"a?b"));
    assert!(!is_ordinary_named_path(b"a[b]"));
    assert!(!is_ordinary_named_path(b"a\\b"));
    assert!(!is_ordinary_named_path(b":!x"));
}

/// Every spelling the incumbent's fixed list never held is a question,
///  and every one of them that reaches by pattern alone names no character of a name.
#[test]
fn every_whole_tree_spelling_is_a_question_git_answers() {
    for spelling in [
        ".", "./", ".//", "././", ".*", "..", "sub/..", "sub/../", "sub/../.", "*", "**", "./**",
        ":/*", ":/**", ":/", ":(top)", ":(glob)**", ":(top,glob)**", ":(top)**", ":(icase)*",
        ":(attr:!nope)", ":!nothing-here", ":^nothing-here", "*.txt", "sub/*",
    ] {
        assert_eq!(
            questioned(&[spelling]),
            vec![0],
            "{spelling:?} is no ordinary name"
        );
    }
    // A spelling that names at least one character of a name is not reach-only.
    for spelling in ["*.txt", "sub/*", "sub/../a.txt", "[a-z]*.txt", ":(glob)a*"] {
        assert!(
            !names_no_literal_character(spelling.as_bytes(), false),
            "{spelling:?} names characters"
        );
    }
    // A spelling that names none of them reaches by pattern alone.
    for spelling in [
        ".",
        "./",
        ".//",
        "./**",
        "sub/..",
        "sub/../.",
        "*",
        "**",
        ":/",
        ":/*",
        ":/**",
        ":(top)",
        ":(glob)**",
        ":(top,glob)**",
        ":(icase)*",
        ":(attr:!nope)",
        ":!nothing-here",
        ":^nothing-here",
    ] {
        assert!(
            names_no_literal_character(spelling.as_bytes(), false),
            "{spelling:?} reaches only"
        );
    }
    // Longhand magic that makes the pattern a name reads as a name, as Git reads it.
    assert!(!names_no_literal_character(b":(literal)**", false));
    assert!(names_no_literal_character(b":(literal)", false));
    // A longhand form with no closing parenthesis is no reading at all, so nothing is refused on it.
    assert!(!names_no_literal_character(b":(top", false));
}

/// A caller that reads pathspecs literally has no magic and no wildcard:
/// a spelling is a name unless it resolves to a directory.
#[test]
fn a_literal_caller_names_by_its_own_spelling() {
    for spelling in ["*", "**", ":(top)", ":(glob)**", ":!x", "*.txt", "./**"] {
        assert!(
            !names_no_literal_character(spelling.as_bytes(), true),
            "{spelling:?} is a file name under literal pathspecs"
        );
    }
    for spelling in [".", "./", ".//", "sub/..", "././"] {
        assert!(
            names_no_literal_character(spelling.as_bytes(), true),
            "{spelling:?} still names a directory"
        );
    }
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
    assert!(region(&["--resolved"]).pathspec_tokens.is_empty());
    assert_eq!(region(&["-A"]).all_token, Some(0));
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

/// A token that is not UTF-8 is a path like any other.
#[cfg(unix)]
#[test]
fn undecodable_paths_are_judged_by_their_bytes() {
    let ordinary: AddRegion =
        parse_add_region(&[byte_argument(b".\xff"), byte_argument(b"\xff")], &[]).expect("valid");
    assert!(ordinary.pathspec_tokens.is_empty());
    let bulk: AddRegion = parse_add_region(&[byte_argument(b"\xff*")], &[]).expect("valid");
    assert_eq!(bulk.pathspec_tokens, vec![0]);
}

/// Paths staged against `HEAD`,
///  one per line.
fn staged(root: &Path) -> String {
    return output_text(&git(root, &["diff", "--cached", "--name-only"]));
}

/// The paths one query lists,
///  each as Git printed it.
fn listed(directory: &Path, arguments: &[OsString]) -> Vec<String> {
    let answer: std::process::Output = git_output(directory, arguments);
    assert!(
        answer.status.success(),
        "the listing failed: {}",
        String::from_utf8_lossy(&answer.stderr)
    );
    let mut paths: Vec<String> = Vec::<String>::new();
    for path in answer.stdout.split(|byte| return *byte == 0) {
        if !path.is_empty() {
            paths.push(String::from_utf8_lossy(path).into_owned());
        }
    }
    paths.sort();
    return paths;
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
        assert_eq!(region(&[spelling]).all_token, Some(0), "{spelling}");
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
/// so the reading stays for a spelling this reading does not model.
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
    assert!(!literal(&["-C", "--literal-pathspecs"], None), "the value of -C is no option of the caller's");
    assert!(!literal(&["-c", "--no-literal-pathspecs"], None), "nor is the value of -c");
    assert!(literal(&["-C", "--literal-pathspecs", "--literal-pathspecs"], None), "the option after that value still counts");
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

/// The two queries the policy reads are built here:
/// the caller's own spellings with the caller's own mode,
/// and the whole worktree with a mode the wrapper wrote.
#[test]
fn the_queries_carry_the_caller_mode_and_their_own() {
    let prefix: Vec<OsString> = os_arguments(&["-C", "dir", "--literal-pathspecs"]);
    let spellings: Vec<OsString> = os_arguments(&[":(top)", "a*b"]);
    let selection: Vec<String> = selection_query_arguments(prefix.as_slice(), spellings.as_slice())
        .iter()
        .map(|word| { return word.to_string_lossy().into_owned(); })
        .collect();
    assert_eq!(
        selection,
        vec![
            String::from("-C"),
            String::from("dir"),
            String::from("--literal-pathspecs"),
            String::from("ls-files"),
            String::from("-z"),
            String::from("--cached"),
            String::from("--others"),
            String::from("--exclude-standard"),
            String::from("--"),
            String::from(":(top)"),
            String::from("a*b"),
        ]
    );
    let whole: Vec<String> = whole_worktree_query_arguments(prefix.as_slice())
        .iter()
        .map(|word| { return word.to_string_lossy().into_owned(); })
        .collect();
    assert_eq!(
        whole,
        vec![
            String::from("-C"),
            String::from("dir"),
            String::from("--literal-pathspecs"),
            String::from("--no-literal-pathspecs"),
            String::from("ls-files"),
            String::from("-z"),
            String::from("--cached"),
            String::from("--others"),
            String::from("--exclude-standard"),
            String::from("--full-name"),
            String::from("--"),
            String::from(":/"),
        ]
    );
}

/// Git reads the value of a global option as that option's value,
/// so `git -C --literal-pathspecs add -- '**'` changes into the directory of that name
/// and expands the pattern over the whole worktree:
/// the caller's mode is no mode at all,
/// and the bulk rejection stands.
#[test]
fn a_directory_named_like_a_mode_option_is_no_mode_in_git() {
    let directory: PathBuf = fixture("add-global-value");
    let root: PathBuf = repository(directory.as_path(), "--literal-pathspecs");
    std::fs::write(root.join("a.txt"), b"a\n").expect("write a.txt");
    std::fs::write(root.join("b.txt"), b"b\n").expect("write b.txt");
    git(directory.as_path(), &["-C", "--literal-pathspecs", "add", "--", "**"]);
    assert_eq!(staged(root.as_path()), "a.txt\nb.txt", "Git stages everything below the directory -C named");
    remove(directory.as_path());
}

/// What the policy reads is what Git stages:
/// for every spelling that sweeps the tree,
/// the listing holds every path,
/// and `git add` stages exactly the listed paths.
#[test]
fn the_selection_query_lists_exactly_what_git_add_stages() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("add-selection");
    std::fs::write(root.join("top.txt"), b"top\n").expect("write top file");
    std::fs::create_dir(root.join("sub")).expect("sub directory");
    std::fs::write(root.join("sub/below.txt"), b"below\n").expect("write below file");
    std::fs::write(root.join(".gitignore"), b"*.log\n").expect("write ignore file");
    std::fs::write(root.join("ignored.log"), b"gone\n").expect("write ignored file");
    git(root.as_path(), &["add", "--", ".gitignore", "sub/below.txt", "top.txt"]);
    git(root.as_path(), &["commit", "--quiet", "--message=tree"]);
    // Every tracked file differs from the index, so each spelling below stages all of them.
    for path in ["sub/below.txt", "top.txt", "tracked.txt"] {
        std::fs::write(root.join(path), b"changed\n").expect("modify a tracked file");
    }
    let whole: Vec<String> = listed(root.as_path(), whole_worktree_query_arguments(&[]).as_slice());
    assert_eq!(whole, vec![".gitignore", "sub/below.txt", "top.txt", "tracked.txt"]);
    // A tracked file that differs from the index is staged by every spelling below.
    std::fs::write(root.join(".gitignore"), b"*.log\n# changed\n").expect("modify the ignore file");
    for spelling in [
        ".", "./", ".//", "././", "./**", "*", "**", ":/*", ":/**", ":/", ":(top)",
        ":(glob)**", ":(top,glob)**", ":(top)**", ":(icase)*", ":(attr:!nope)", ":!nothing-here",
        ":^nothing-here", "sub/..", "sub/../",
    ] {
        let spellings: Vec<OsString> = vec![OsString::from(spelling)];
        assert_eq!(
            listed(root.as_path(), selection_query_arguments(&[], spellings.as_slice()).as_slice()),
            whole,
            "{spelling:?} selects the whole tree"
        );
        git(root.as_path(), &["add", "--", spelling]);
        assert_eq!(staged(root.as_path()), whole.join("\n"), "{spelling:?}");
        git(root.as_path(), &["reset", "--quiet"]);
    }
    // A pattern that happens to fit every path selects them all, and names a character of a name.
    let pattern: Vec<OsString> = vec![OsString::from("*.*")];
    assert_eq!(
        listed(root.as_path(), selection_query_arguments(&[], pattern.as_slice()).as_slice()),
        whole,
        "every path of this tree holds a dot"
    );
    assert!(!names_no_literal_character(b"*.*", false));
    // The ignored file is no path `git add` stages, so no query lists it.
    assert!(!whole.contains(&String::from("ignored.log")));
    remove(directory.as_path());
}

/// Git's own answer for the spellings under literal mode,
/// which is what the policy reads:
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
    assert!(!names_no_literal_character(b"**", true) && names_no_literal_character(b".", true));
    remove(directory.as_path());
}
