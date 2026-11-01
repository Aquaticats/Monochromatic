//! What:
//!  Check every fixture's compiler output against the verdict markers written in the fixture itself.
//! Why:
//!  The `.stderr` golden files are produced by running the lint,
//!  so by themselves they would record a wrong verdict as readily as a right one;
//!  the markers are written by hand before the golden file exists and decide independently.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const fixture of fixtures('tests/ui')) {
//!   expect(observedReports(fixture.stderr)).toEqual(markedVerdicts(fixture.source));
//! }
//! ```

/// Import an ordered map so differences print in line order.
use std::collections::BTreeMap;
/// Import the borrowed operating-system string type,
/// to compare a file extension without a closure.
use std::ffi::OsStr;
/// Import file reading and directory listing.
use std::fs;
/// Import the path type used for fixture files.
use std::path::PathBuf;

/// What:
///  The kinds of compiler diagnostics a fixture may legitimately produce.
///  `Ord` lets the kind be part of a sorted map key.
/// Why:
///  Anything outside this list in a `.stderr` file is a surprise and fails the check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Kind = 'closure' | 'unfulfilled' | 'unknown-lint' | { error: string };
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Kind {
    /// A report of this lint.
    Closure,
    /// rustc's `unfulfilled_lint_expectations` warning.
    Unfulfilled,
    /// rustc's `unknown_lints` warning.
    UnknownLint,
    /// A compiler error with its code,
    /// such as `E0658`.
    Error(String),
}

/// A count of diagnostics per source line and kind.
type Tally = BTreeMap<(usize, Kind), usize>;

/// What:
///  Read the verdict markers of one fixture:
///  a comment line `// verdict: <word>` describes the next line.
///  Words:
///  `allowed` and `suppressed` (no report),
///  `reported` or `reported N`,
///  `unfulfilled`,
///  `unknown-lint`,
///  `error CODE`.
/// Why:
///  Markers sit beside the code they judge,
///  so a reader sees the expectation without opening the golden file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function markedVerdicts(source: string): Tally;
/// ```
fn parse_markers(source: &str) -> Tally {
    let mut tally: Tally = BTreeMap::new();
    for (index, line) in source.lines().enumerate() {
        let Some(word) = line.trim().strip_prefix("// verdict: ") else {
            continue;
        };
        let target: usize = index + 2;
        let mut parts = word.split_whitespace();
        let name: &str = parts
            .next()
            .expect("a verdict word follows the marker prefix");
        let argument: Option<&str> = parts.next();
        match name {
            "allowed" | "suppressed" => {}
            "reported" => {
                let count: usize = match argument {
                    Some(text) => text.parse().expect("a report count is a number"),
                    None => 1,
                };
                tally.insert((target, Kind::Closure), count);
            }
            "unfulfilled" => {
                tally.insert((target, Kind::Unfulfilled), 1);
            }
            "unknown-lint" => {
                tally.insert((target, Kind::UnknownLint), 1);
            }
            "error" => {
                let code: &str = argument.expect("an error marker names its code");
                tally.insert((target, Kind::Error(String::from(code))), 1);
            }
            other => panic!("unknown verdict word {other:?} on line {}", index + 1),
        }
    }
    return tally;
}

/// What:
///  Decide the kind of a diagnostic header line (`warning: ...` or `error[E0000]: ...` at column zero).
///  `None` means the line is a summary such as `warning: 3 warnings emitted`.
/// Why:
///  Unknown headers panic,
///  so an unexpected diagnostic cannot slip into a golden file unnoticed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function headerKind(line: string): Kind | undefined;
/// ```
fn header_kind(line: &str) -> Option<Kind> {
    if line.starts_with("warning: this closure captures") {
        return Some(Kind::Closure);
    }
    if line.starts_with("warning: this lint expectation is unfulfilled") {
        return Some(Kind::Unfulfilled);
    }
    if line.starts_with("warning: unknown lint") {
        return Some(Kind::UnknownLint);
    }
    if let Some(rest) = line.strip_prefix("error[") {
        let code: &str = rest
            .split(']')
            .next()
            .expect("an error header has a closing bracket");
        return Some(Kind::Error(String::from(code)));
    }
    if line.starts_with("error: aborting") || line.contains(" emitted") {
        return None;
    }
    if line.starts_with("warning: ") || line.starts_with("error") {
        panic!("unexpected diagnostic in a fixture's output: {line}");
    }
    return None;
}

/// What:
///  Count the diagnostics of a normalised `.stderr` file by primary line and kind,
///  for one fixture file name.
/// Why:
///  The first `--> $DIR/<file>:<line>:<column>` after a header is the diagnostic's primary location.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function observedReports(stderr: string, fixtureName: string): Tally;
/// ```
fn parse_stderr(stderr: &str, fixture_name: &str) -> Tally {
    let mut tally: Tally = BTreeMap::new();
    let mut pending: Option<Kind> = None;
    let location_prefix: String = format!("--> $DIR/{fixture_name}:");
    for line in stderr.lines() {
        if !line.starts_with(' ') && !line.is_empty() {
            pending = header_kind(line);
            continue;
        }
        let Some(location) = line.trim().strip_prefix(location_prefix.as_str()) else {
            continue;
        };
        let Some(kind) = pending.take() else {
            continue;
        };
        let digits: &str = location
            .split(':')
            .next()
            .expect("a location has a line number");
        let number: usize = digits.parse().expect("a line number is numeric");
        *tally.entry((number, kind)).or_insert(0) += 1;
    }
    return tally;
}

/// What:
///  List the differences between marked and observed diagnostics as readable lines;
///  empty means they agree.
/// Why:
///  A report without a marker,
///  or a marker without a report,
///  both mean the lint and the fixture's author disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function differences(marked: Tally, observed: Tally): string[];
/// ```
fn differences(marked: &Tally, observed: &Tally) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for (key, count) in observed {
        let wanted: usize = marked.get(key).copied().unwrap_or(0);
        if wanted != *count {
            lines.push(format!(
                "line {} {:?}: observed {count}, marked {wanted}",
                key.0, key.1
            ));
        }
    }
    for (key, count) in marked {
        if !observed.contains_key(key) {
            lines.push(format!(
                "line {} {:?}: observed 0, marked {count}",
                key.0, key.1
            ));
        }
    }
    return lines;
}

/// What:
///  The fixture source files under `tests/ui`,
///  sorted by name.
/// Why:
///  The check must cover every fixture without a hand-kept list that could go stale.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fixtures = readdirSync('tests/ui').filter((name) => name.endsWith('.rs')).sort();
/// ```
fn fixture_paths() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir("tests/ui").expect("tests/ui exists") {
        let path: PathBuf = entry.expect("a directory entry reads").path();
        if path.extension() == Some(OsStr::new("rs")) {
            paths.push(path);
        }
    }
    paths.sort();
    return paths;
}

/// Every fixture's golden output must equal its hand-written verdict markers.
#[test]
fn golden_output_equals_the_verdict_markers_of_every_fixture() {
    let paths: Vec<PathBuf> = fixture_paths();
    assert!(
        paths.len() >= 6,
        "expected at least six fixtures, found {}",
        paths.len()
    );
    let mut failures: Vec<String> = Vec::new();
    for path in paths {
        let name: String = path
            .file_name()
            .expect("file name")
            .to_string_lossy()
            .into_owned();
        let source: String = fs::read_to_string(&path).expect("fixture reads");
        let stderr: String =
            fs::read_to_string(path.with_extension("stderr")).expect("golden output exists");
        assert!(
            source.contains("// verdict: "),
            "{name} has no verdict markers"
        );
        let found: Vec<String> =
            differences(&parse_markers(&source), &parse_stderr(&stderr, &name));
        for line in found {
            failures.push(format!("{name}: {line}"));
        }
    }
    assert!(
        failures.is_empty(),
        "fixtures disagree with their markers:\n{}",
        failures.join("\n")
    );
}

/// Positive control:
/// an unmarked report is a difference.
#[test]
fn an_unmarked_report_is_flagged() {
    let stderr = "warning: this closure captures `n`, but x\n  --> $DIR/a.rs:7:5\n";
    let found: Vec<String> = differences(
        &parse_markers("// verdict: allowed\nlet f = 1;\n"),
        &parse_stderr(stderr, "a.rs"),
    );
    assert_eq!(found.len(), 1, "{found:?}");
}

/// Positive control:
/// a marker whose report is missing is a difference.
#[test]
fn a_report_marker_without_a_report_is_flagged() {
    let found: Vec<String> = differences(
        &parse_markers("// verdict: reported\nlet f = 1;\n"),
        &parse_stderr("", "a.rs"),
    );
    assert_eq!(found.len(), 1, "{found:?}");
}

/// Control:
/// a matching marker and report agree,
/// including a count.
#[test]
fn matching_markers_and_reports_agree() {
    let stderr = "warning: this closure captures `n`, but x\n  --> $DIR/a.rs:2:5\n\nwarning: this closure captures `m`, but x\n  --> $DIR/a.rs:2:20\n";
    let found: Vec<String> = differences(
        &parse_markers("// verdict: reported 2\nlet f = 1;\n"),
        &parse_stderr(stderr, "a.rs"),
    );
    assert!(found.is_empty(), "{found:?}");
}

/// Control:
/// an unknown diagnostic header aborts the check.
#[test]
#[should_panic(expected = "unexpected diagnostic")]
fn an_unknown_diagnostic_header_aborts() {
    let _ = parse_stderr("warning: unused variable\n  --> $DIR/a.rs:1:1\n", "a.rs");
}
