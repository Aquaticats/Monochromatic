//! What:
//! Unit tests of the ripgrep walk policy in `policy.rs`.
//! Why:
//! The parser reads another tool's argument grammar,
//! so each flag family needs a case that proves the boundary:
//! what is a wide walk,
//! what names a path and what only prints information.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // policy.unit.test.ts
//! describe('describeWalk', () => { ... });
//! ```

/// What:
/// Imports the functions and the record type under test from this crate.
/// Why:
/// The tests call them directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { describeWalk, refusalFor } from './policy.ts';
/// ```
use crate::policy::{WalkDescription, describe_walk, rg_refusal};

/// Imports the kind of standard input an invocation carries.
use crate::stdin_kind::StdinKind;

/// What:
/// `fn texts(items: &[&str]) -> Vec<String>` declares a helper turning a borrowed list of text slices into an
/// owned list of `String`s (the shape the policy functions take).
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
/// `fn walk(...) -> WalkDescription` declares a helper that describes a list of arguments.
/// Why:
/// Shortens every case below.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const walk = (items: string[]) => describeWalk(items);
/// ```
fn walk(items: &[&str]) -> WalkDescription {
    return describe_walk(&texts(items));
}

/// What:
/// `#[test]` marks the next function as a test case.
/// Why:
/// The runner finds the tests by it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('a plain search is not wide', () => { ... });
/// ```
#[test]
fn a_plain_search_is_not_wide_and_names_no_path() {
    // `assert_eq!(a, b)` stops the test with a message when `a != b` (a macro, hence `!`).
    assert_eq!(
        walk(&["foo"]),
        WalkDescription { widened: false, informational: false, path_count: 0, short_replace: false }
    );
}

/// The short `-r` is ripgrep's `--replace`.
/// It must be detected alone,
/// with an attached value and inside clusters.
#[test]
fn the_short_r_flag_is_detected_alone_attached_and_in_clusters() {
    assert!(walk(&["-r", "X", "foo"]).short_replace);
    assert!(walk(&["-rl", "foo"]).short_replace);
    assert!(walk(&["-ir", "X", "foo"]).short_replace);
    assert!(walk(&["-nr", "X", "foo"]).short_replace);
}

/// Things that look like `-r` but are not the flag must not be banned.
#[test]
fn lookalikes_of_the_short_r_flag_are_not_flags() {
    // The long form is explicit replacement and stays available.
    assert!(!walk(&["--replace", "X", "foo"]).short_replace);
    assert!(!walk(&["--replace=X", "foo"]).short_replace);
    // `r` as the value of `-g` (a glob), after `-e` (a pattern), or after `--` is not a flag.
    assert!(!walk(&["-gr", "foo"]).short_replace);
    assert!(!walk(&["-e", "-r"]).short_replace);
    assert!(!walk(&["foo", "--", "-r"]).short_replace);
    assert!(!walk(&["-i", "foo"]).short_replace);
}

/// The ban applies to rg only and has no escape hatch,
/// repository root or path condition.
#[test]
fn refuses_the_short_r_flag_for_rg_everywhere() {
    let message = rg_refusal("rg", &texts(&["-rl", "foo"]), false, false, StdinKind::None, "/anywhere");

    assert!(message.is_some());
    let text = message.unwrap_or_default();
    assert!(text.contains("refused"));
    assert!(text.contains("--replace"));
    assert!(text.contains("-l"));
    // The wide-walk escape hatch must not lift this ban, and a piped stdin must not either.
    assert!(rg_refusal("rg", &texts(&["-r", "X", "foo"]), true, true, StdinKind::Pipe, "/repo").is_some());
    // Other tools keep their own meaning of `-r` (grep's is "recursive").
    assert_eq!(rg_refusal("grep", &texts(&["-rl", "foo"]), false, true, StdinKind::None, "/repo"), None);
    // The explicit long form is not refused.
    assert_eq!(rg_refusal("rg", &texts(&["--replace", "X", "foo"]), false, false, StdinKind::None, "/repo"), None);
}

/// The first positional is the pattern,
/// so only later ones are paths.
#[test]
fn the_first_positional_is_the_pattern_and_the_rest_are_paths() {
    assert_eq!(walk(&["foo", "src", "doc"]).path_count, 2);
}

/// Every flag family that widens the walk must be recognized.
#[test]
fn hidden_and_no_ignore_flags_widen_the_walk() {
    assert!(walk(&["--hidden", "foo"]).widened);
    assert!(walk(&["--no-ignore", "foo"]).widened);
    assert!(walk(&["--no-ignore-vcs", "foo"]).widened);
    assert!(walk(&["--unrestricted", "foo"]).widened);
    assert!(walk(&["-u", "foo"]).widened);
    assert!(walk(&["-uu", "foo"]).widened);
    assert!(walk(&["-.", "foo"]).widened);
    assert!(walk(&["-iu", "foo"]).widened);
}

/// A pattern given by `-e` or `-f` means every positional is a path.
#[test]
fn a_pattern_from_a_flag_makes_every_positional_a_path() {
    assert_eq!(walk(&["-e", "foo", "--hidden"]).path_count, 0);
    assert_eq!(walk(&["-e", "foo", "src"]).path_count, 1);
    assert_eq!(walk(&["--regexp=foo", "src", "doc"]).path_count, 2);
}

/// With `--files` there is no pattern,
/// so every positional is a path.
#[test]
fn files_mode_has_no_pattern() {
    assert_eq!(walk(&["--files", "--hidden"]).path_count, 0);
    assert_eq!(walk(&["--files", "src"]).path_count, 1);
}

/// A flag's value must not be mistaken for a path or a pattern.
#[test]
fn a_flags_value_is_not_a_positional() {
    assert_eq!(walk(&["--glob", "*.ts", "--hidden", "foo"]).path_count, 0);
    assert_eq!(walk(&["-g", "*.ts", "foo", "src"]).path_count, 1);
    assert_eq!(walk(&["--glob=*.ts", "foo"]).path_count, 0);
    assert_eq!(walk(&["-A3", "foo"]).path_count, 0);
    assert_eq!(walk(&["-A", "3", "foo", "src"]).path_count, 1);
}

/// Everything after `--` is positional,
/// even when it looks like a flag.
#[test]
fn everything_after_a_double_dash_is_positional() {
    let description = walk(&["--hidden", "--", "--odd-pattern", "src"]);
    assert!(description.widened);
    assert_eq!(description.path_count, 1);
}

/// Help,
/// version and list flags only print,
/// so they have nothing to refuse.
#[test]
fn information_flags_are_informational() {
    assert!(walk(&["--help"]).informational);
    assert!(walk(&["--version"]).informational);
    assert!(walk(&["-h"]).informational);
    assert!(walk(&["-V"]).informational);
    assert!(walk(&["--type-list"]).informational);
}

/// The refusal fires only for the exact combination:
/// rg,
/// wide,
/// no path,
/// repository root,
/// no piped input.
#[test]
fn refuses_a_wide_rg_walk_with_no_path_at_a_repository_root() {
    let message = rg_refusal("rg", &texts(&["--hidden", "foo"]), false, true, StdinKind::None, "/repo");

    // `message.is_some()` asks whether a refusal came back; `unwrap_or_default()` then takes the text out.
    assert!(message.is_some());
    let text = message.unwrap_or_default();
    assert!(text.contains("refused"));
    assert!(text.contains("/repo"));
    assert!(text.contains("SEARCH_SHADOW_ALLOW_WIDE"));
    assert!(text.contains("narrower path"));
}

/// What:
/// `fn piped_rg(items: &[&str]) -> Option<String>` declares a helper that asks the policy about one piped invocation
/// (standard input is a pipe,
/// the working directory is not a repository root).
/// Why:
/// Every case below differs only in the arguments,
/// and the lost-input risk does not depend on the directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pipedRg = (items: string[]) => rgRefusal('rg', texts(items), false, false, 'pipe', '/anywhere');
/// ```
fn piped_rg(items: &[&str]) -> Option<String> {
    return rg_refusal("rg", &texts(items), false, false, StdinKind::Pipe, "/anywhere");
}

/// A pipe on standard input with a named search path loses the piped lines:
/// ripgrep reads the paths and never the pipe.
/// It is refused wherever it runs, repository root or not.
#[test]
fn refuses_a_piped_search_that_names_a_search_path() {
    for arguments in [&["--invert-match", "X", "."][..], &["X", "some/dir"], &["-l", "X", "a", "b"]] {
        let message = piped_rg(arguments);

        assert!(message.is_some(), "{arguments:?} with a pipe must be refused");
        let text = message.unwrap_or_default();
        // The message names the command, the input it would drop, and both ways forward (rule DGT).
        assert!(text.contains("refused"), "{text}");
        assert!(text.contains(&arguments.join(" ")), "the message must name the command: {text}");
        assert!(text.contains("standard input is a pipe"), "the message must name the affected input: {text}");
        assert!(text.contains("drop the search path"), "the message must offer dropping the path to filter the pipe: {text}");
        assert!(text.contains("without a pipe"), "the message must offer dropping the pipe to search the path: {text}");
    }
}

/// Every shape where ripgrep does read the pipe,
/// or reads no input at all,
/// must still run.
#[test]
fn lets_a_piped_search_through_when_the_pipe_is_read_or_no_search_runs() {
    // No search path: the pipe is the input.
    assert_eq!(piped_rg(&["X"]), None);
    // `-` is ripgrep's explicit "read standard input", alone or beside other paths.
    assert_eq!(piped_rg(&["X", "-"]), None);
    assert_eq!(piped_rg(&["X", "-", "extra/path"]), None);
    // `--files` lists files and reads no input.
    assert_eq!(piped_rg(&["--files", "."]), None);
    // Informational flags only print.
    assert_eq!(piped_rg(&["--version"]), None);
    assert_eq!(piped_rg(&["--help"]), None);
    // A redirected file and a socket are deliberate, and an unpiped run reads nothing.
    assert_eq!(rg_refusal("rg", &texts(&["X", "."]), false, false, StdinKind::File, "/anywhere"), None);
    assert_eq!(rg_refusal("rg", &texts(&["X", "."]), false, false, StdinKind::Socket, "/anywhere"), None);
    assert_eq!(rg_refusal("rg", &texts(&["X", "."]), false, false, StdinKind::None, "/anywhere"), None);
    // Other tools keep their own argument grammar.
    assert_eq!(rg_refusal("grep", &texts(&["X", "."]), false, false, StdinKind::Pipe, "/anywhere"), None);
}

/// Each condition that makes the walk bounded or deliberate must let it through.
#[test]
fn lets_through_everything_that_is_not_a_wide_unbounded_rg_walk() {
    let wide = texts(&["--hidden", "foo"]);
    assert_eq!(rg_refusal("rg", &texts(&["--hidden", "foo", ".github"]), false, true, StdinKind::None, "/repo"), None);
    assert_eq!(rg_refusal("rg", &texts(&["foo"]), false, true, StdinKind::None, "/repo"), None);
    assert_eq!(rg_refusal("rg", &texts(&["--help", "--hidden"]), false, true, StdinKind::None, "/repo"), None);
    assert_eq!(rg_refusal("grep", &wide, false, true, StdinKind::None, "/repo"), None);
    assert_eq!(rg_refusal("rg", &wide, true, true, StdinKind::None, "/repo"), None);
    assert_eq!(rg_refusal("rg", &wide, false, false, StdinKind::None, "/repo"), None);
    assert_eq!(rg_refusal("rg", &wide, false, true, StdinKind::Pipe, "/repo"), None);
}
