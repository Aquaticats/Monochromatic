//! What:
//! Unit tests of the `grep` and `find` refusal rules and of the dispatcher that picks a rule by tool name.
//! Why:
//! Both rules read another tool's argument grammar and decide on real directories,
//! so each boundary needs a case:
//! what recurses and what names a path and what bounds a `find` and which places count as wide.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // policy-walk.unit.test.ts
//! describe('grepRefusal', () => { ... }); describe('findRefusal', () => { ... });
//! ```

/// Imports the dispatcher and the record it reads.
use crate::policy::{Invocation, refusal_for};

/// Imports the `find` and `bfs` rule and its two parsers.
use crate::policy_find::{find_max_depth, find_refusal, find_start_points};

/// Imports the `grep` rule and its parser and the record the parser returns.
use crate::policy_grep::{GrepDescription, describe_grep, grep_refusal};

/// Imports the file-system module (make fixture directories).
use std::fs;

/// Imports the borrowed path type.
use std::path::Path;

/// What:
/// `fn texts(items: &[&str]) -> Vec<String>` declares a helper turning a borrowed list of text slices into an owned
/// list of `String`s.
/// Why:
/// Test cases read better as short literals.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const texts = (items: string[]): string[] => [...items];
/// ```
fn texts(items: &[&str]) -> Vec<String> {
    let mut owned: Vec<String> = Vec::new();
    for item in items {
        // `(*item).to_string()` reads through the reference and makes an owned copy of the text.
        owned.push((*item).to_string());
    }
    return owned;
}

/// What:
/// `fn repository() -> tempfile::TempDir` declares a helper that makes a disposable directory with a `.git` inside.
/// Why:
/// Most cases run "at a repository root".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repository(): string { const dir = mkdtemp(); mkdir(join(dir, '.git')); return dir; }
/// ```
fn repository() -> tempfile::TempDir {
    let directory = crate::test_scratch::scratch();
    fs::create_dir(directory.path().join(".git")).expect("make .git");
    fs::create_dir(directory.path().join("package")).expect("make a subdirectory");
    return directory;
}

/// What:
/// `#[test]` marks the next function as a test case.
/// Why:
/// The runner finds the tests by it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('recognizes the recursive flags', () => { ... });
/// ```
#[test]
fn grep_recursion_is_detected_in_every_spelling() {
    // `assert!(condition)` stops the test when the condition is false (a macro, hence `!`).
    assert!(describe_grep(&texts(&["-r", "foo"])).recursive);
    assert!(describe_grep(&texts(&["-R", "foo"])).recursive);
    assert!(describe_grep(&texts(&["-rn", "foo"])).recursive);
    assert!(describe_grep(&texts(&["-ir", "foo"])).recursive);
    assert!(describe_grep(&texts(&["--recursive", "foo"])).recursive);
    assert!(describe_grep(&texts(&["--dereference-recursive", "foo"])).recursive);
    assert!(describe_grep(&texts(&["-d", "recurse", "foo"])).recursive);
    assert!(describe_grep(&texts(&["--directories=recurse", "foo"])).recursive);
    assert!(describe_grep(&texts(&["--directories", "recurse", "foo"])).recursive);
    assert!(!describe_grep(&texts(&["-n", "foo", "file"])).recursive);
}

/// Values of value-taking flags and the pattern are not file operands.
#[test]
fn grep_operands_exclude_the_pattern_and_flag_values() {
    assert_eq!(
        describe_grep(&texts(&["-r", "foo", "src", "doc"])),
        GrepDescription { recursive: true, operands: texts(&["src", "doc"]) }
    );
    assert_eq!(describe_grep(&texts(&["-e", "foo", "src"])).operands, texts(&["src"]));
    assert_eq!(describe_grep(&texts(&["-A", "3", "foo", "src"])).operands, texts(&["src"]));
    assert_eq!(describe_grep(&texts(&["-A3", "foo"])).operands.len(), 0);
    assert_eq!(describe_grep(&texts(&["--include", "*.ts", "foo", "src"])).operands, texts(&["src"]));
    assert_eq!(describe_grep(&texts(&["--include=*.ts", "foo"])).operands.len(), 0);
    assert_eq!(describe_grep(&texts(&["foo", "--", "-r"])).operands, texts(&["-r"]));
}

/// A recursive grep from a repository root is refused whether the root is implied or named.
#[test]
fn grep_refuses_recursion_from_a_repository_root() {
    let root = repository();
    let home = crate::test_scratch::scratch();

    let implied = grep_refusal("grep", &texts(&["-rn", "foo"]), false, root.path(), home.path());
    let dot = grep_refusal("grep", &texts(&["-r", "foo", "."]), false, root.path(), home.path());

    assert!(implied.is_some());
    let text = implied.unwrap_or_default();
    assert!(text.contains("refused"));
    assert!(text.contains("a whole repository"));
    assert!(text.contains("SEARCH_SHADOW_ALLOW_WIDE"));
    assert!(dot.is_some());
}

/// The home directory and `/` are wide too,
/// from anywhere.
#[test]
fn grep_refuses_recursion_over_home_and_the_filesystem_root() {
    let work = crate::test_scratch::scratch();
    let home = crate::test_scratch::scratch();
    let home_text = home.path().to_string_lossy().to_string();

    assert!(grep_refusal("grep", &texts(&["-r", "foo", &home_text]), false, work.path(), home.path()).is_some());
    assert!(grep_refusal("grep", &texts(&["-r", "foo", "/"]), false, work.path(), home.path()).is_some());
}

/// What is bounded or deliberate must pass.
#[test]
fn grep_lets_bounded_and_deliberate_calls_through() {
    let root = repository();
    let home = crate::test_scratch::scratch();

    // Not recursive, a named subdirectory, escape hatch, and stdin-only input (the '-' operand).
    assert_eq!(grep_refusal("grep", &texts(&["foo", "file"]), false, root.path(), home.path()), None);
    assert_eq!(grep_refusal("grep", &texts(&["-r", "foo", "package"]), false, root.path(), home.path()), None);
    assert_eq!(grep_refusal("grep", &texts(&["-r", "foo"]), true, root.path(), home.path()), None);
    assert_eq!(grep_refusal("grep", &texts(&["-r", "foo", "-"]), false, root.path(), home.path()), None);
    // Recursion from a directory that is not a repository root is bounded by that directory.
    assert_eq!(grep_refusal("grep", &texts(&["-r", "foo"]), false, root.path().join("package").as_path(), home.path()), None);
}

/// Start points are read after the leading global options and before the first expression.
#[test]
fn find_start_points_are_read_before_the_first_expression() {
    assert_eq!(find_start_points(&texts(&["-name", "x"])), texts(&["."]));
    assert_eq!(find_start_points(&texts(&["src", "doc", "-name", "x"])), texts(&["src", "doc"]));
    assert_eq!(find_start_points(&texts(&["-L", "src", "-type", "f"])), texts(&["src"]));
    assert_eq!(find_start_points(&texts(&["-H", "-D", "tree", "src", "!", "-name", "x"])), texts(&["src"]));
    assert_eq!(find_start_points(&texts(&[])), texts(&["."]));
}

/// The smallest `-maxdepth` wins,
/// and a missing or non-numeric one is none.
#[test]
fn find_max_depth_takes_the_smallest_numeric_value() {
    assert_eq!(find_max_depth(&texts(&[".", "-maxdepth", "2"])), Some(2));
    assert_eq!(find_max_depth(&texts(&[".", "-maxdepth", "5", "-maxdepth", "1"])), Some(1));
    assert_eq!(find_max_depth(&texts(&[".", "-maxdepth", "x"])), None);
    assert_eq!(find_max_depth(&texts(&["."])), None);
}

/// A find from a repository root or home or `/` with no small depth limit is refused.
#[test]
fn find_refuses_unbounded_walks_from_wide_places() {
    let root = repository();
    let work = crate::test_scratch::scratch();
    let home = crate::test_scratch::scratch();
    let home_text = home.path().to_string_lossy().to_string();

    let implied = find_refusal("find", &texts(&["-name", "*.ts"]), false, root.path(), home.path());
    assert!(implied.is_some());
    let text = implied.unwrap_or_default();
    assert!(text.contains("refused"));
    assert!(text.contains("-maxdepth 3"));
    assert!(find_refusal("find", &texts(&[".", "-type", "f"]), false, root.path(), home.path()).is_some());
    assert!(find_refusal("bfs", &texts(&[&home_text, "-name", "x"]), false, work.path(), home.path()).is_some());
    assert!(find_refusal("find", &texts(&["/", "-name", "x"]), false, work.path(), home.path()).is_some());
    // A depth over the bound does not help.
    assert!(find_refusal("find", &texts(&[".", "-maxdepth", "9"]), false, root.path(), home.path()).is_some());
}

/// A bounded depth or a narrower start or a non-root directory or the escape hatch all pass.
#[test]
fn find_lets_bounded_walks_through() {
    let root = repository();
    let home = crate::test_scratch::scratch();

    assert_eq!(find_refusal("find", &texts(&[".", "-maxdepth", "3", "-name", "x"]), false, root.path(), home.path()), None);
    assert_eq!(find_refusal("find", &texts(&["package", "-name", "x"]), false, root.path(), home.path()), None);
    assert_eq!(find_refusal("find", &texts(&["-name", "x"]), false, root.path().join("package").as_path(), home.path()), None);
    assert_eq!(find_refusal("find", &texts(&["."]), true, root.path(), home.path()), None);
}

/// `--version` and `--help` never walk,
/// so they pass even where the implied start point is a repository root.
#[test]
fn find_lets_the_information_flags_through() {
    let root = repository();
    let home = crate::test_scratch::scratch();

    assert_eq!(find_refusal("find", &texts(&["--version"]), false, root.path(), home.path()), None);
    assert_eq!(find_refusal("bfs", &texts(&["--help"]), false, root.path(), home.path()), None);
    assert_eq!(find_refusal("find", &texts(&["-version"]), false, root.path(), home.path()), None);
}

/// What:
/// `fn invocation(...) -> Invocation` declares a helper that builds the record the dispatcher reads.
/// Why:
/// The dispatcher tests need one for each tool name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const invocation = (name, args, workingDirectory, home): Invocation => ({ name, arguments: args, allowWide: false, ... });
/// ```
fn invocation(name: &str, arguments: &[&str], working_directory: &Path, home: &Path) -> Invocation {
    return Invocation {
        name: name.to_string(),
        arguments: texts(arguments),
        allow_wide: false,
        stdin_has_data: false,
        working_directory: working_directory.to_path_buf(),
        home: home.to_path_buf(),
    };
}

/// The dispatcher picks the rule by tool name and leaves the other shadowed tools alone.
#[test]
fn the_dispatcher_picks_the_rule_by_tool_name() {
    let root = repository();
    let home = crate::test_scratch::scratch();

    assert!(refusal_for(&invocation("rg", &["--hidden", "foo"], root.path(), home.path())).is_some());
    assert!(refusal_for(&invocation("rg", &["-rl", "foo"], root.path(), home.path())).is_some());
    for name in ["grep", "egrep", "fgrep"] {
        assert!(refusal_for(&invocation(name, &["-r", "foo"], root.path(), home.path())).is_some(), "{name}");
    }
    for name in ["find", "bfs"] {
        assert!(refusal_for(&invocation(name, &["-name", "x"], root.path(), home.path())).is_some(), "{name}");
    }
    // Throttled only: these tools are never refused.
    for name in ["fd", "ag", "ack", "ugrep", "ug", "tree", "du", "ncdu", "tokei", "cloc", "fdfind"] {
        assert!(refusal_for(&invocation(name, &["-r", "--hidden", "."], root.path(), home.path())).is_none(), "{name}");
    }
}
