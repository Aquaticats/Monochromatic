//! What:
//!  Controls for resolving comments into active directives.
//! Why:
//!  Which line a line directive covers,
//! and whether a list means "these rules",
//! "every rule" or "nothing",
//!  decide what is suppressed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('resolveDirectives', () => { /* covered lines, targets, void directives */ });
//! ```

/// Import the resolver and the pieces it is checked against.
use super::{Directive, Targets, resolve_directives};
use crate::diagnostic::Diagnostic;
use crate::directive_findings::Reporter;
use crate::directive_parse::DirectiveKind;
use crate::directive_scan::{ScannedComments, scan_comments};
use crate::run_paths::Language;

/// Resolve the directives of a Rust source,
///  returning them with the codes of the problems found.
fn resolve(source: &str) -> (Vec<Directive>, Vec<String>) {
    let scanned: ScannedComments = scan_comments(Language::Rust, "f.rs", source).expect("rust");
    let reporter: Reporter<'_> = Reporter {
        filename: "f.rs",
        scanned: &scanned,
    };
    let mut problems: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    let directives: Vec<Directive> = resolve_directives(&reporter, source, &mut problems);
    let codes: Vec<String> = problems
        .iter()
        .map(|problem| return problem.code.clone())
        .collect::<Vec<String>>();
    return (directives, codes);
}

/// A line directive covers its own line,
///  a next-line directive the line after its last line,
///  and range directives cover no line.
#[test]
fn covered_lines_follow_the_kind() {
    let source: &str = "fn a() {} // monochromatic-lint-disable-line -- r\n/* monochromatic-lint-disable-next-line -- r */\n// monochromatic-lint-disable -- r\n// monochromatic-lint-enable -- r\n";
    let (directives, problems) = resolve(source);
    assert!(problems.is_empty());
    let shape: Vec<(DirectiveKind, usize)> = directives
        .iter()
        .map(|directive| return (directive.kind, directive.covered_line))
        .collect::<Vec<(DirectiveKind, usize)>>();
    assert_eq!(
        shape,
        [
            (DirectiveKind::DisableLine, 1),
            (DirectiveKind::DisableNextLine, 3),
            (DirectiveKind::Disable, 0),
            (DirectiveKind::Enable, 0),
        ]
    );
}

/// No list covers everything,
///  a list covers its accepted entries,
///  and a list whose entries were all refused covers nothing.
#[test]
fn lists_resolve_to_all_listed_or_none() {
    let source: &str = "// monochromatic-lint-disable -- r\n// monochromatic-lint-disable rust/tests-in-sidecars -- r\n// monochromatic-lint-disable rust/max-lines -- r\n";
    let (directives, problems) = resolve(source);
    assert_eq!(problems, ["directive/non-suppressible"]);
    assert_eq!(directives[0].targets, Targets::All);
    assert_eq!(directives[0].slot_count(), 1);
    assert_eq!(directives[0].slot_for("anything"), Some(0));
    assert_eq!(directives[1].slot_count(), 1);
    assert_eq!(directives[1].slot_for("rust/tests-in-sidecars"), Some(0));
    assert_eq!(directives[1].slot_for("markdown/single-h1"), None);
    assert_eq!(directives[2].targets, Targets::Listed(Vec::new()));
    assert_eq!(directives[2].slot_count(), 0);
    assert_eq!(directives[2].slot_for("rust/tests-in-sidecars"), None);
}

/// A directive without a reason is not returned,
///  and neither is a malformed one,
///  while ordinary comments produce nothing.
#[test]
fn void_directives_are_not_returned() {
    let source: &str = "// monochromatic-lint-disable\n// monochromatic-lint-bogus -- r\n// ordinary\n/* monochromatic-lint-disable-line\n -- r */\n";
    let (directives, problems) = resolve(source);
    assert!(directives.is_empty());
    assert_eq!(
        problems,
        [
            "directive/missing-justification",
            "directive/malformed",
            "directive/malformed"
        ]
    );
}
