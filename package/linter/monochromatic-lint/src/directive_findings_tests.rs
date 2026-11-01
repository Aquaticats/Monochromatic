//! What:
//!  Controls for the findings that directives produce.
//! Why:
//!  The identifiers are stable output,
//!  and each message must name the affected input and the ways to resolve the problem.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('Reporter', () => { /* codes, severities, spans, hints */ });
//! ```

/// Import the reporter and the pieces needed to position its findings.
use super::{MALFORMED, MISSING_JUSTIFICATION, NON_SUPPRESSIBLE, Reporter, UNKNOWN_RULE, UNUSED};
use crate::diagnostic::{Diagnostic, Severity};
use crate::directive_parse::{DirectiveKind, RuleToken};
use crate::directive_scan::{CommentSite, ScannedComments, scan_comments};
use crate::run_paths::Language;

/// The shared fixture:
/// one Rust comment.
const SOURCE: &str = "// monochromatic-lint-disable-next-line -- r\n";

/// Run a check with a reporter over the fixture.
fn with_reporter<Check: FnOnce(&Reporter<'_>, CommentSite)>(check: Check) {
    let scanned: ScannedComments = scan_comments(Language::Rust, "f.rs", SOURCE).expect("rust");
    let site: CommentSite = scanned.sites[0];
    let reporter: Reporter<'_> = Reporter {
        filename: "f.rs",
        scanned: &scanned,
    };
    check(&reporter, site);
}

/// The codes are the stable identifiers that are documented.
#[test]
fn codes_are_stable() {
    assert_eq!(MISSING_JUSTIFICATION, "directive/missing-justification");
    assert_eq!(UNKNOWN_RULE, "directive/unknown-rule");
    assert_eq!(NON_SUPPRESSIBLE, "directive/non-suppressible");
    assert_eq!(MALFORMED, "directive/malformed");
    assert_eq!(UNUSED, "directive/unused");
}

/// Comment-level findings underline the whole comment,
///  are errors,
///  and name the keyword and the remedies.
#[test]
fn comment_findings_underline_the_comment() {
    with_reporter(|reporter: &Reporter<'_>, site: CommentSite| {
        let missing: Diagnostic = reporter.missing_justification(site, DirectiveKind::DisableLine);
        assert_eq!(missing.code, MISSING_JUSTIFICATION);
        assert_eq!(missing.severity, Severity::Error);
        assert_eq!(missing.filename, "f.rs");
        assert_eq!(missing.labels[0].span.offset, 0);
        assert_eq!(missing.labels[0].span.length, site.end - site.start);
        assert!(
            missing
                .message
                .contains("monochromatic-lint-disable-line has no justification")
        );
        assert!(
            missing
                .help
                .as_deref()
                .expect("help")
                .contains("or remove the directive")
        );
        let malformed: Diagnostic = reporter.malformed(site, "a reason");
        assert_eq!(malformed.code, MALFORMED);
        assert!(malformed.message.contains("a reason"));
        assert!(!malformed.processing_failure);
        assert!(malformed.fix.is_none());
    });
}

/// An unknown identifier gets the full-identifier hint only when exactly its suffix matches.
#[test]
fn unknown_rule_hints() {
    with_reporter(|reporter: &Reporter<'_>, _site: CommentSite| {
        let token = |text: &str| -> RuleToken {
            return RuleToken {
                text: String::from(text),
                start: 3,
                end: 3 + text.len(),
            };
        };
        let near: Diagnostic = reporter.unknown_rule(&token("single-h1"));
        assert_eq!(near.code, UNKNOWN_RULE);
        assert!(
            near.help
                .as_deref()
                .expect("help")
                .contains("Did you mean \"markdown/single-h1\"?")
        );
        let far: Diagnostic = reporter.unknown_rule(&token("single"));
        assert!(
            far.help
                .as_deref()
                .expect("help")
                .contains("monochromatic-lint --rules")
        );
        assert_eq!(far.labels[0].span.offset, 3);
        assert_eq!(far.labels[0].span.length, "single".len());
    });
}

/// A refusal names the rule and says that configuration cannot change it.
#[test]
fn refusal_names_the_rule() {
    with_reporter(|reporter: &Reporter<'_>, _site: CommentSite| {
        let token: RuleToken = RuleToken {
            text: String::from("rust/max-lines"),
            start: 3,
            end: 17,
        };
        let refusal: Diagnostic = reporter.non_suppressible(&token);
        assert_eq!(refusal.code, NON_SUPPRESSIBLE);
        assert!(
            refusal
                .message
                .contains("\"rust/max-lines\" cannot be suppressed")
        );
        assert!(
            refusal
                .help
                .as_deref()
                .expect("help")
                .contains("no configuration option changes that")
        );
    });
}

/// Unused findings carry the requested severity,
///  and their wording distinguishes enables from disables and listed rules from whole directives.
#[test]
fn unused_wording() {
    with_reporter(|reporter: &Reporter<'_>, site: CommentSite| {
        let whole: Diagnostic = reporter.unused(
            Severity::Warn,
            DirectiveKind::Disable,
            (site.start, site.end),
            None,
        );
        assert_eq!(whole.code, UNUSED);
        assert_eq!(whole.severity, Severity::Warn);
        assert_eq!(
            whole.message,
            "Unused monochromatic-lint-disable directive: it suppressed no finding."
        );
        let listed: Diagnostic = reporter.unused(
            Severity::Error,
            DirectiveKind::Enable,
            (3, 9),
            Some("rust/a"),
        );
        assert_eq!(listed.severity, Severity::Error);
        assert_eq!(
            listed.message,
            "Unused monochromatic-lint-enable directive for rust/a: it enables nothing that was disabled at that point."
        );
        assert_eq!(listed.labels[0].span.length, 6);
        assert!(
            listed
                .help
                .as_deref()
                .expect("help")
                .contains("--report-unused-disable-directives")
        );
    });
}
