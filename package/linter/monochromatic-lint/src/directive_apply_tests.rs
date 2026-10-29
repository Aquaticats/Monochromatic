//! What:
//!  Controls for the directive filter over synthetic host-position findings.
//! Why:
//!  The filter is rule-independent,
//!  so these controls feed it findings with registry codes at chosen positions
//!  and check which survive,
//!  which directive findings appear,
//!  and which directives are reported as unused.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('applyDirectives', () => { /* kinds, ranges, justification, suppressibility, unused */ });
//! ```

/// Import the filter and the models its inputs and outputs use.
use super::{DirectiveContext, apply_directives};
use crate::diagnostic::{Diagnostic, Severity};
use crate::directive_options::DirectiveOptions;
use crate::run_failure::{host_span, processing_failure};
use crate::run_paths::Language;

/// A suppressible Rust rule.
const SIDECARS: &str = "rust/tests-in-sidecars";
/// A suppressible rule of the other language,
/// reachable in a Rust file through rustdoc.
const BARE_URLS: &str = "markdown/no-bare-urls";
/// A Markdown rule used for Markdown fixtures.
const SINGLE_H1: &str = "markdown/single-h1";
/// The unsuppressible line budget.
const MAX_LINES: &str = "rust/max-lines";
/// The unsuppressible documentation requirement.
const RUSTDOC: &str = "rust/require-rustdoc";

/// A finding with a registry code at the first occurrence of a marker,
/// in the host's units.
fn finding(language: Language, source: &str, marker: &str, code: &str) -> Diagnostic {
    let offset: usize = source.find(marker).expect("the marker is in the fixture");
    return Diagnostic::new(
        code,
        Severity::Error,
        String::from("fixture finding"),
        String::from("fixture"),
        host_span(language, source, offset),
    );
}

/// Apply directives to findings with unused-directive reporting at the given severity.
fn apply(
    language: Language,
    source: &str,
    findings: Vec<Diagnostic>,
    unused: Option<Severity>,
) -> Vec<Diagnostic> {
    let context: DirectiveContext<'_> = DirectiveContext {
        language,
        filename: "fixture",
        source,
        options: DirectiveOptions { unused },
    };
    return apply_directives(&context, findings);
}

/// `code@line` for every finding,
///  in output order.
fn summary(found: &[Diagnostic]) -> Vec<String> {
    let mut lines: Vec<String> = Vec::<String>::new();
    for item in found {
        lines.push(format!("{}@{}", item.code, item.labels[0].span.line));
    }
    return lines;
}

/// Rust fixture findings:
/// one suppressible finding at each marker.
fn sidecar_findings(source: &str, markers: &[&str]) -> Vec<Diagnostic> {
    let mut found: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for marker in markers {
        found.push(finding(Language::Rust, source, marker, SIDECARS));
    }
    return found;
}

/// The next-line directive covers exactly the line after its comment.
#[test]
fn disable_next_line_covers_only_the_following_line() {
    let source: &str = "// monochromatic-lint-disable-next-line rust/tests-in-sidecars -- generated\nfn a() {}\nfn b() {}\n";
    let found: Vec<Diagnostic> = apply(
        Language::Rust,
        source,
        sidecar_findings(source, &["fn a", "fn b"]),
        None,
    );
    assert_eq!(summary(&found), ["rust/tests-in-sidecars@3"]);
}

/// The line directive covers its own line only,
///  not the line before or after.
#[test]
fn disable_line_covers_only_its_own_line() {
    let source: &str = "fn z() {}\nfn a() {} // monochromatic-lint-disable-line rust/tests-in-sidecars -- r\nfn b() {}\n";
    let found: Vec<Diagnostic> = apply(
        Language::Rust,
        source,
        sidecar_findings(source, &["fn z", "fn a", "fn b"]),
        None,
    );
    assert_eq!(
        summary(&found),
        ["rust/tests-in-sidecars@1", "rust/tests-in-sidecars@3"]
    );
}

/// A block comment works like a line comment for every kind.
#[test]
fn block_comments_carry_directives() {
    let source: &str = "/* monochromatic-lint-disable-next-line rust/tests-in-sidecars -- r */\nfn a() {}\nfn b() {} /* monochromatic-lint-disable-line rust/tests-in-sidecars -- r */\nfn c() {}\n";
    let found: Vec<Diagnostic> = apply(
        Language::Rust,
        source,
        sidecar_findings(source, &["fn a", "fn b", "fn c"]),
        None,
    );
    assert_eq!(summary(&found), ["rust/tests-in-sidecars@4"]);
}

/// A range lasts from its comment to the matching enable,
///  and without an enable to the end of the file.
#[test]
fn disable_lasts_until_enable_or_end_of_file() {
    let bounded: &str = "fn before() {}\n// monochromatic-lint-disable rust/tests-in-sidecars -- r\nfn inside() {}\n// monochromatic-lint-enable rust/tests-in-sidecars -- r\nfn after() {}\n";
    let found: Vec<Diagnostic> = apply(
        Language::Rust,
        bounded,
        sidecar_findings(bounded, &["fn before", "fn inside", "fn after"]),
        None,
    );
    assert_eq!(
        summary(&found),
        ["rust/tests-in-sidecars@1", "rust/tests-in-sidecars@5"]
    );
    let open: &str = "fn before() {}\n// monochromatic-lint-disable rust/tests-in-sidecars -- r\nfn inside() {}\nfn after() {}\n";
    let rest: Vec<Diagnostic> = apply(
        Language::Rust,
        open,
        sidecar_findings(open, &["fn before", "fn inside", "fn after"]),
        None,
    );
    assert_eq!(summary(&rest), ["rust/tests-in-sidecars@1"]);
}

/// A finding before the disable comment on the same line stays,
///  and one exactly at the comment's first byte is covered.
#[test]
fn range_start_is_the_comment_position() {
    let source: &str =
        "fn a() {} /* monochromatic-lint-disable rust/tests-in-sidecars -- r */ fn b() {}\n";
    let before: Diagnostic = finding(Language::Rust, source, "fn a", SIDECARS);
    let at: Diagnostic = finding(Language::Rust, source, "/* mono", SIDECARS);
    let after: Diagnostic = finding(Language::Rust, source, "fn b", SIDECARS);
    let found: Vec<Diagnostic> = apply(Language::Rust, source, vec![before, at, after], None);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].labels[0].span.offset, 0);
}

/// A bare disable covers every suppressible rule,
///  and a later enable of one rule re-enables only that rule.
#[test]
fn enable_of_one_rule_after_a_bare_disable_reenables_only_that_rule() {
    let source: &str = "// monochromatic-lint-disable -- r\nfn a() {}\n// monochromatic-lint-enable rust/tests-in-sidecars -- r\nfn b() {}\n";
    let findings: Vec<Diagnostic> = vec![
        finding(Language::Rust, source, "fn a", SIDECARS),
        finding(Language::Rust, source, "fn a", BARE_URLS),
        finding(Language::Rust, source, "fn b", SIDECARS),
        finding(Language::Rust, source, "fn b", BARE_URLS),
    ];
    let found: Vec<Diagnostic> = apply(Language::Rust, source, findings, None);
    assert_eq!(summary(&found), ["rust/tests-in-sidecars@4"]);
}

/// Overlapping ranges for different rules end independently,
///  and a bare enable ends everything still disabled.
#[test]
fn nested_and_overlapping_ranges_end_independently() {
    let source: &str = concat!(
        "// monochromatic-lint-disable rust/tests-in-sidecars -- r\n",
        "// monochromatic-lint-disable markdown/no-bare-urls -- r\n",
        "fn a() {}\n",
        "// monochromatic-lint-enable rust/tests-in-sidecars -- r\n",
        "fn b() {}\n",
        "// monochromatic-lint-enable -- r\n",
        "fn c() {}\n",
    );
    let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for marker in ["fn a", "fn b", "fn c"] {
        findings.push(finding(Language::Rust, source, marker, SIDECARS));
        findings.push(finding(Language::Rust, source, marker, BARE_URLS));
    }
    let found: Vec<Diagnostic> = apply(Language::Rust, source, findings, None);
    assert_eq!(
        summary(&found),
        [
            "rust/tests-in-sidecars@5",
            "rust/tests-in-sidecars@7",
            "markdown/no-bare-urls@7"
        ]
    );
}

/// A line directive wins inside a range that does not cover its rule,
///  and a later enable does not undo a line directive that precedes it.
#[test]
fn line_directives_apply_inside_and_outside_ranges() {
    let source: &str = concat!(
        "// monochromatic-lint-disable markdown/no-bare-urls -- r\n",
        "// monochromatic-lint-disable-next-line rust/tests-in-sidecars -- r\n",
        "fn a() {}\n",
        "// monochromatic-lint-enable markdown/no-bare-urls -- r\n",
        "fn b() {}\n",
    );
    let findings: Vec<Diagnostic> = vec![
        finding(Language::Rust, source, "fn a", SIDECARS),
        finding(Language::Rust, source, "fn a", BARE_URLS),
        finding(Language::Rust, source, "fn b", SIDECARS),
        finding(Language::Rust, source, "fn b", BARE_URLS),
    ];
    let found: Vec<Diagnostic> = apply(Language::Rust, source, findings, None);
    assert_eq!(
        summary(&found),
        ["rust/tests-in-sidecars@5", "markdown/no-bare-urls@5"]
    );
}

/// A directive without identifiers covers every suppressible rule and never the unsuppressible ones,
///  and it is not itself reported for that.
#[test]
fn a_bare_directive_leaves_unsuppressible_rules_alone() {
    let source: &str = "// monochromatic-lint-disable-next-line -- r\nfn a() {}\n";
    let findings: Vec<Diagnostic> = vec![
        finding(Language::Rust, source, "fn a", SIDECARS),
        finding(Language::Rust, source, "fn a", MAX_LINES),
        finding(Language::Rust, source, "fn a", RUSTDOC),
    ];
    let found: Vec<Diagnostic> = apply(Language::Rust, source, findings, Some(Severity::Warn));
    assert_eq!(
        summary(&found),
        ["rust/max-lines@2", "rust/require-rustdoc@2"]
    );
}

/// A directive aimed at an unsuppressible rule is reported at the identifier,
///  keeps the rule's finding,
///  and still applies its other identifiers.
#[test]
fn a_listed_unsuppressible_rule_is_refused_and_reported() {
    let source: &str = "// monochromatic-lint-disable-next-line rust/max-lines, rust/tests-in-sidecars -- r\nfn a() {}\n";
    let findings: Vec<Diagnostic> = vec![
        finding(Language::Rust, source, "fn a", MAX_LINES),
        finding(Language::Rust, source, "fn a", SIDECARS),
    ];
    let found: Vec<Diagnostic> = apply(Language::Rust, source, findings, None);
    assert_eq!(
        summary(&found),
        ["rust/max-lines@2", "directive/non-suppressible@1"]
    );
    let refusal: &Diagnostic = &found[1];
    assert_eq!(refusal.severity, Severity::Error);
    assert_eq!(
        refusal.labels[0].span.offset,
        source.find("rust/max-lines").expect("token")
    );
    assert_eq!(refusal.labels[0].span.length, "rust/max-lines".len());
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
    assert!(!refusal.processing_failure);
}

/// A list whose every entry was refused covers nothing,
///  rather than falling back to covering every rule.
#[test]
fn a_directive_whose_entries_were_all_refused_suppresses_nothing() {
    for list in [
        "rust/max-lines",
        "rust/require-rustdoc",
        "nonsense",
        "rust/max-lines, nonsense",
    ] {
        let source: String =
            format!("// monochromatic-lint-disable-next-line {list} -- r\nfn a() {{}}\n");
        let findings: Vec<Diagnostic> =
            vec![finding(Language::Rust, source.as_str(), "fn a", SIDECARS)];
        let found: Vec<Diagnostic> = apply(Language::Rust, source.as_str(), findings, None);
        assert_eq!(found[0].code, SIDECARS, "{list}");
        assert!(found.len() >= 2, "{list}");
    }
}

/// An identifier outside the registry is reported with a hint when a full identifier ends with it,
///  and the other identifiers still apply.
#[test]
fn an_unknown_rule_is_reported_with_a_hint() {
    let source: &str = "// monochromatic-lint-disable-next-line max-lines, rust/tests-in-sidecars, rust/nothing -- r\nfn a() {}\n";
    let findings: Vec<Diagnostic> = vec![finding(Language::Rust, source, "fn a", SIDECARS)];
    let found: Vec<Diagnostic> = apply(Language::Rust, source, findings, None);
    assert_eq!(
        summary(&found),
        ["directive/unknown-rule@1", "directive/unknown-rule@1"]
    );
    assert!(
        found[0]
            .help
            .as_deref()
            .expect("help")
            .contains("Did you mean \"rust/max-lines\" or \"shell/max-lines\"?")
    );
    assert!(
        found[1]
            .help
            .as_deref()
            .expect("help")
            .contains("monochromatic-lint --rules")
    );
    assert_eq!(
        found[1].labels[0].span.offset,
        source.find("rust/nothing").expect("token")
    );
}

/// A directive without a reason suppresses nothing and is reported,
///  for every kind and for every way to leave the reason out.
#[test]
fn a_missing_justification_voids_the_directive() {
    for tail in ["", " --", " --  ", " - dash", " because"] {
        for keyword in [
            "monochromatic-lint-disable-next-line",
            "monochromatic-lint-disable-line",
        ] {
            let source: String = format!(
                "// {keyword} rust/tests-in-sidecars{tail}\nfn a() {{}} // {keyword} rust/tests-in-sidecars{tail}\n"
            );
            let findings: Vec<Diagnostic> =
                vec![finding(Language::Rust, source.as_str(), "fn a", SIDECARS)];
            let found: Vec<Diagnostic> = apply(Language::Rust, source.as_str(), findings, None);
            assert_eq!(found[0].code, SIDECARS, "{keyword}{tail:?}");
            assert!(
                found
                    .iter()
                    .any(|item| return item.code == "directive/missing-justification"),
                "{keyword}{tail:?}"
            );
        }
    }
}

/// A void range directive does not start a range,
///  and a void enable does not end one.
#[test]
fn void_range_directives_do_nothing() {
    let void_disable: &str = "// monochromatic-lint-disable rust/tests-in-sidecars\nfn a() {}\n";
    let found: Vec<Diagnostic> = apply(
        Language::Rust,
        void_disable,
        sidecar_findings(void_disable, &["fn a"]),
        None,
    );
    assert_eq!(
        summary(&found),
        [
            "rust/tests-in-sidecars@2",
            "directive/missing-justification@1"
        ]
    );
    let void_enable: &str = "// monochromatic-lint-disable rust/tests-in-sidecars -- r\n// monochromatic-lint-enable rust/tests-in-sidecars\nfn a() {}\n";
    let rest: Vec<Diagnostic> = apply(
        Language::Rust,
        void_enable,
        sidecar_findings(void_enable, &["fn a"]),
        None,
    );
    assert_eq!(summary(&rest), ["directive/missing-justification@2"]);
}

/// A comment that starts like a directive but cannot be read is reported and suppresses nothing.
#[test]
fn malformed_comments_are_reported() {
    for comment in [
        "// monochromatic-lint-disable-nextline rust/tests-in-sidecars -- r",
        "// monochromatic-lint-disable-next-line rust/tests-in-sidecars,, -- r",
        "// monochromatic-lint-disable-next-line ,rust/tests-in-sidecars -- r",
    ] {
        let source: String = format!("{comment}\nfn a() {{}}\n");
        let found: Vec<Diagnostic> = apply(
            Language::Rust,
            source.as_str(),
            sidecar_findings(source.as_str(), &["fn a"]),
            None,
        );
        assert_eq!(
            summary(&found),
            ["rust/tests-in-sidecars@2", "directive/malformed@1"],
            "{comment}"
        );
        assert_eq!(found[1].severity, Severity::Error);
    }
}

/// A line directive written as a block comment over several lines is ambiguous and void,
///  while a range directive may span lines.
#[test]
fn multi_line_line_directives_are_malformed_but_ranges_may_span_lines() {
    let line_directive: &str =
        "/* monochromatic-lint-disable-next-line rust/tests-in-sidecars\n -- r */\nfn a() {}\n";
    let found: Vec<Diagnostic> = apply(
        Language::Rust,
        line_directive,
        sidecar_findings(line_directive, &["fn a"]),
        None,
    );
    assert_eq!(
        summary(&found),
        ["rust/tests-in-sidecars@3", "directive/malformed@1"]
    );
    assert!(found[1].message.contains("must be written on one line"));
    let range: &str = "/* monochromatic-lint-disable rust/tests-in-sidecars\n -- r */\nfn a() {}\n";
    let rest: Vec<Diagnostic> = apply(
        Language::Rust,
        range,
        sidecar_findings(range, &["fn a"]),
        None,
    );
    assert!(rest.is_empty());
}

/// Directive-looking text inside strings and doc comments of every form is not a directive.
#[test]
fn text_in_strings_and_doc_comments_is_not_a_directive() {
    let directive: &str = "monochromatic-lint-disable-next-line rust/tests-in-sidecars -- r";
    let sources: [String; 8] = [
        format!("const A: &str = \"// {directive}\";\nfn a() {{}}\n"),
        format!("const A: &str = r#\"\n// {directive}\n\"#;\nfn a() {{}}\n"),
        format!("/// {directive}\nfn a() {{}}\n"),
        format!("//! {directive}\nfn a() {{}}\n"),
        format!("/** {directive} */\nfn a() {{}}\n"),
        format!("/*! {directive} */\nfn a() {{}}\n"),
        format!("//// {directive}\nfn a() {{}}\n"),
        format!("// see {directive}\nfn a() {{}}\n"),
    ];
    for source in &sources {
        let found: Vec<Diagnostic> = apply(
            Language::Rust,
            source.as_str(),
            sidecar_findings(source.as_str(), &["fn a"]),
            Some(Severity::Warn),
        );
        let codes: Vec<&str> = found
            .iter()
            .map(|item| return item.code.as_str())
            .collect::<Vec<&str>>();
        assert_eq!(codes, [SIDECARS], "{source}");
    }
}

/// An unterminated block comment is not a directive,
///  and a file with syntax errors still honours the directives in it.
#[test]
fn unterminated_comments_and_syntax_errors() {
    let unterminated: &str =
        "fn a() {}\n/* monochromatic-lint-disable rust/tests-in-sidecars -- r\n";
    let found: Vec<Diagnostic> = apply(
        Language::Rust,
        unterminated,
        sidecar_findings(unterminated, &["fn a"]),
        None,
    );
    assert_eq!(summary(&found), ["rust/tests-in-sidecars@1"]);
    let broken: &str =
        "// monochromatic-lint-disable-next-line rust/tests-in-sidecars -- r\nfn {{{ a\n";
    let rest: Vec<Diagnostic> = apply(
        Language::Rust,
        broken,
        sidecar_findings(broken, &["fn {"]),
        None,
    );
    assert!(rest.is_empty());
}

/// CRLF files keep the directive and its target on adjacent lines,
///  and the carriage return never joins the reason or an identifier.
#[test]
fn crlf_rust_sources() {
    let source: &str = "// monochromatic-lint-disable-next-line rust/tests-in-sidecars -- r\r\nfn a() {}\r\nfn b() {}\r\n/* monochromatic-lint-disable-line rust/tests-in-sidecars -- r */\r\n";
    let found: Vec<Diagnostic> = apply(
        Language::Rust,
        source,
        sidecar_findings(source, &["fn a", "fn b"]),
        Some(Severity::Warn),
    );
    assert_eq!(
        summary(&found),
        ["rust/tests-in-sidecars@3", "directive/unused@4"]
    );
}

/// Markdown covers CRLF and bare CR line endings,
///  and its directive findings use UTF-16 columns after astral text.
#[test]
fn markdown_line_endings_and_columns() {
    let crlf: &str = "<!-- monochromatic-lint-disable-next-line markdown/no-bare-urls -- r -->\r\nhttp://a.example\r\nhttp://b.example\r\n";
    let found: Vec<Diagnostic> = apply(
        Language::Markdown,
        crlf,
        vec![
            finding(Language::Markdown, crlf, "http://a", BARE_URLS),
            finding(Language::Markdown, crlf, "http://b", BARE_URLS),
        ],
        None,
    );
    assert_eq!(summary(&found), ["markdown/no-bare-urls@3"]);
    let cr: &str = "<!-- monochromatic-lint-disable-next-line markdown/no-bare-urls -- r -->\rhttp://a.example\rhttp://b.example\r";
    let rest: Vec<Diagnostic> = apply(
        Language::Markdown,
        cr,
        vec![
            finding(Language::Markdown, cr, "http://a", BARE_URLS),
            finding(Language::Markdown, cr, "http://b", BARE_URLS),
        ],
        None,
    );
    assert_eq!(summary(&rest), ["markdown/no-bare-urls@3"]);
    let wide: &str =
        "\u{1f680}\u{1f680} <!-- monochromatic-lint-disable-line markdown/bogus -- r -->\n";
    let reported: Vec<Diagnostic> = apply(Language::Markdown, wide, Vec::<Diagnostic>::new(), None);
    let token: usize = wide.find("markdown/bogus").expect("token");
    let span = &reported[0].labels[0].span;
    assert_eq!(span.offset, token);
    // Two rockets are eight bytes and four UTF-16 units, so the column trails the byte offset by four.
    assert_eq!(span.column, token - 4 + 1);
    assert_eq!(span.length, "markdown/bogus".len());
}

/// Rust directive findings use byte columns after multibyte text.
#[test]
fn rust_columns_are_bytes() {
    let source: &str =
        "let \u{e9} = \"\u{1f680}\"; /* monochromatic-lint-disable-line rust/foo -- r */\n";
    let reported: Vec<Diagnostic> = apply(Language::Rust, source, Vec::<Diagnostic>::new(), None);
    let span = &reported[0].labels[0].span;
    assert_eq!(span.offset, source.find("rust/foo").expect("token"));
    assert_eq!(span.column, 53);
}

/// Markdown directives:
/// HTML comments suppress in the same ways,
///  and the same text in code is inert.
#[test]
fn markdown_directives_and_inert_text() {
    let source: &str = concat!(
        "<!-- monochromatic-lint-disable markdown/no-bare-urls -- r -->\n",
        "http://a.example\n",
        "<!-- monochromatic-lint-enable markdown/no-bare-urls -- r -->\n",
        "http://b.example\n",
        "```\n",
        "<!-- monochromatic-lint-disable markdown/single-h1 -- r -->\n",
        "```\n",
        "`<!-- monochromatic-lint-disable-next-line markdown/single-h1 -- r -->`\n",
        "# One\n",
    );
    let found: Vec<Diagnostic> = apply(
        Language::Markdown,
        source,
        vec![
            finding(Language::Markdown, source, "http://a", BARE_URLS),
            finding(Language::Markdown, source, "http://b", BARE_URLS),
            finding(Language::Markdown, source, "# One", SINGLE_H1),
        ],
        Some(Severity::Warn),
    );
    assert_eq!(
        summary(&found),
        ["markdown/no-bare-urls@4", "markdown/single-h1@9"]
    );
}

/// MDX has no directive form,
///  so its text is returned untouched.
#[test]
fn mdx_findings_are_returned_untouched() {
    let source: &str = "<!-- monochromatic-lint-disable markdown/single-h1 -- r -->\n# One\n";
    let found: Vec<Diagnostic> = apply(
        Language::Mdx,
        source,
        vec![finding(Language::Mdx, source, "# One", SINGLE_H1)],
        Some(Severity::Error),
    );
    assert_eq!(summary(&found), ["markdown/single-h1@2"]);
}

/// A file without the directive prefix anywhere is returned unchanged.
#[test]
fn files_without_the_prefix_are_returned_unchanged() {
    let source: &str = "fn a() {}\n";
    let findings: Vec<Diagnostic> = sidecar_findings(source, &["fn a"]);
    let found: Vec<Diagnostic> = apply(
        Language::Rust,
        source,
        findings.clone(),
        Some(Severity::Error),
    );
    assert_eq!(found, findings);
}

/// Processing failures,
///  refused fixes,
///  directive findings and unknown codes are never suppressed,
///  and an incomplete check is not judged for unused directives.
#[test]
fn only_registry_rule_findings_are_suppressed() {
    let source: &str = "// monochromatic-lint-disable -- r\n// monochromatic-lint-disable-next-line rust/tests-in-sidecars -- r\nfn a() {}\n";
    let span = host_span(Language::Rust, source, source.find("fn a").expect("marker"));
    let failure: Diagnostic =
        processing_failure("fixture", span.clone(), String::from("cannot check"));
    let refused: Diagnostic = Diagnostic::new(
        "core/fix-refused",
        Severity::Error,
        String::from("refused"),
        String::from("fixture"),
        span.clone(),
    );
    let unknown_code: Diagnostic = Diagnostic::new(
        "rust/not-a-rule",
        Severity::Error,
        String::from("unknown"),
        String::from("fixture"),
        span.clone(),
    );
    // A finding that carries a registry code but is flagged as incomplete processing is never suppressed.
    let mut flagged: Diagnostic = finding(Language::Rust, source, "fn a", SIDECARS);
    flagged.processing_failure = true;
    // A finding without a label has no position to match.
    let mut unlabelled: Diagnostic = finding(Language::Rust, source, "fn a", SIDECARS);
    unlabelled.labels.clear();
    let found: Vec<Diagnostic> = apply(
        Language::Rust,
        source,
        vec![
            failure,
            refused,
            unknown_code,
            flagged,
            unlabelled,
            finding(Language::Rust, source, "fn a", SIDECARS),
        ],
        Some(Severity::Warn),
    );
    let codes: Vec<&str> = found
        .iter()
        .map(|item| return item.code.as_str())
        .collect::<Vec<&str>>();
    assert_eq!(
        codes,
        [
            "core/processing-failure",
            "core/fix-refused",
            "rust/not-a-rule",
            SIDECARS,
            SIDECARS
        ]
    );
    assert!(found[3].processing_failure);
    assert!(found[4].labels.is_empty());
}

/// Unused directives are reported only on request,
///  at the requested severity,
///  and a used directive is never reported.
#[test]
fn unused_directives_are_reported_on_request() {
    let source: &str = "// monochromatic-lint-disable-next-line rust/tests-in-sidecars -- r\nfn a() {}\n// monochromatic-lint-disable-next-line -- r\nfn b() {}\n";
    let findings: Vec<Diagnostic> = sidecar_findings(source, &["fn a"]);
    let off: Vec<Diagnostic> = apply(Language::Rust, source, findings.clone(), None);
    assert!(off.is_empty());
    let warned: Vec<Diagnostic> = apply(
        Language::Rust,
        source,
        findings.clone(),
        Some(Severity::Warn),
    );
    assert_eq!(summary(&warned), ["directive/unused@3"]);
    assert_eq!(warned[0].severity, Severity::Warn);
    assert_eq!(
        warned[0].labels[0].span.offset,
        source
            .find("// monochromatic-lint-disable-next-line -- r")
            .expect("comment")
    );
    assert!(
        warned[0]
            .message
            .starts_with("Unused monochromatic-lint-disable-next-line directive:")
    );
    let errored: Vec<Diagnostic> = apply(Language::Rust, source, findings, Some(Severity::Error));
    assert_eq!(errored[0].severity, Severity::Error);
}

/// A listed directive is judged per identifier,
///  and the unused identifier is underlined.
#[test]
fn unused_identifiers_are_reported_individually() {
    let source: &str = "// monochromatic-lint-disable-next-line rust/tests-in-sidecars, markdown/no-bare-urls -- r\nfn a() {}\n";
    let found: Vec<Diagnostic> = apply(
        Language::Rust,
        source,
        sidecar_findings(source, &["fn a"]),
        Some(Severity::Warn),
    );
    assert_eq!(summary(&found), ["directive/unused@1"]);
    assert_eq!(
        found[0].labels[0].span.offset,
        source.find("markdown/no-bare-urls").expect("token")
    );
    assert!(found[0].message.contains("for markdown/no-bare-urls"));
}

/// An enable is used only when it re-enables something that was disabled,
///  and unused enables are reported like unused disables.
#[test]
fn unused_enables_are_reported() {
    let lone: &str = "// monochromatic-lint-enable rust/tests-in-sidecars -- r\nfn a() {}\n";
    let found: Vec<Diagnostic> = apply(
        Language::Rust,
        lone,
        sidecar_findings(lone, &["fn a"]),
        Some(Severity::Warn),
    );
    assert_eq!(
        summary(&found),
        ["rust/tests-in-sidecars@2", "directive/unused@1"]
    );
    assert!(
        found[1]
            .message
            .contains("enables nothing that was disabled")
    );
    let pair: &str = concat!(
        "// monochromatic-lint-disable rust/tests-in-sidecars, markdown/no-bare-urls -- r\n",
        "fn a() {}\n",
        "// monochromatic-lint-enable rust/tests-in-sidecars, rust/require-explicit-types -- r\n",
        "// monochromatic-lint-enable -- r\n",
        "// monochromatic-lint-enable -- r\n",
    );
    let rest: Vec<Diagnostic> = apply(
        Language::Rust,
        pair,
        sidecar_findings(pair, &["fn a"]),
        Some(Severity::Warn),
    );
    // The disable's second identifier matched no finding,
    //  the first enable's second identifier was never disabled,
    //  the bare enable then re-enables the remaining rule,
    //  and the last bare enable has nothing left.
    assert_eq!(
        summary(&rest),
        [
            "directive/unused@1",
            "directive/unused@3",
            "directive/unused@5"
        ]
    );
    assert!(
        rest[0]
            .message
            .contains("for markdown/no-bare-urls: it suppressed no finding")
    );
    assert_eq!(
        rest[1].labels[0].span.offset,
        pair.find("rust/require-explicit-types").expect("token")
    );
}

/// A later directive that covers a finding is credited,
///  so the earlier redundant one is reported,
///  and a line directive is credited ahead of an enclosing range.
#[test]
fn redundant_directives_are_reported_as_unused() {
    let source: &str = concat!(
        "// monochromatic-lint-disable rust/tests-in-sidecars -- r\n",
        "// monochromatic-lint-disable rust/tests-in-sidecars -- r\n",
        "fn a() {}\n",
        "// monochromatic-lint-disable-next-line rust/tests-in-sidecars -- r\n",
        "fn b() {}\n",
    );
    let found: Vec<Diagnostic> = apply(
        Language::Rust,
        source,
        sidecar_findings(source, &["fn a", "fn b"]),
        Some(Severity::Warn),
    );
    assert_eq!(summary(&found), ["directive/unused@1"]);
}

/// When two line directives cover one finding the later comment is credited,
///  so the earlier one is the unused one.
#[test]
fn the_latest_line_directive_is_credited() {
    let source: &str = "// monochromatic-lint-disable-next-line rust/tests-in-sidecars -- r\nfn a() {} // monochromatic-lint-disable-line rust/tests-in-sidecars -- r\n";
    let found: Vec<Diagnostic> = apply(
        Language::Rust,
        source,
        sidecar_findings(source, &["fn a"]),
        Some(Severity::Warn),
    );
    assert_eq!(summary(&found), ["directive/unused@1"]);
}

/// Unused directives are not judged after a processing failure,
///  because some rule's findings may be missing.
#[test]
fn unused_directives_are_not_judged_after_a_processing_failure() {
    let source: &str =
        "// monochromatic-lint-disable-next-line rust/tests-in-sidecars -- r\nfn a() {}\n";
    let span = host_span(Language::Rust, source, 0);
    let failure: Diagnostic = processing_failure("fixture", span, String::from("cannot check"));
    let found: Vec<Diagnostic> =
        apply(Language::Rust, source, vec![failure], Some(Severity::Error));
    assert_eq!(summary(&found), ["core/processing-failure@1"]);
}
