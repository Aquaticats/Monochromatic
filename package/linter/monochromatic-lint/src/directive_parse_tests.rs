//! What:
//!  Controls for the directive comment grammar.
//! Why:
//!  The same text must mean the same thing in a Rust and a Markdown comment,
//!  and every way to be almost a directive must be classified deliberately.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('parseComment', () => { /* keywords, rule lists, separator, malformed */ });
//! ```

/// Import the parser and its result types.
use super::{DirectiveKind, ParsedComment, ParsedDirective, RuleToken, parse_comment};

/// Parse text that must be a directive.
fn directive(content: &str) -> ParsedDirective {
    match parse_comment(content, 100) {
        ParsedComment::Directive(parsed) => return parsed,
        other => panic!("{content:?} should be a directive, got {other:?}"),
    }
}

/// The identifier texts of a parsed directive.
fn texts(parsed: &ParsedDirective) -> Vec<&str> {
    return parsed
        .rules
        .iter()
        .map(|token| return token.text.as_str())
        .collect::<Vec<&str>>();
}

/// Each of the four keywords selects its kind,
///  and the keyword needs no list or reason to be read.
#[test]
fn each_keyword_selects_its_kind() {
    let cases: [(&str, DirectiveKind); 4] = [
        ("monochromatic-lint-disable", DirectiveKind::Disable),
        ("monochromatic-lint-enable", DirectiveKind::Enable),
        (
            "monochromatic-lint-disable-line",
            DirectiveKind::DisableLine,
        ),
        (
            "monochromatic-lint-disable-next-line",
            DirectiveKind::DisableNextLine,
        ),
    ];
    for (text, kind) in cases {
        let parsed: ParsedDirective = directive(text);
        assert_eq!(parsed.kind, kind, "{text}");
        assert_eq!(parsed.kind.keyword(), text);
        assert!(parsed.rules.is_empty(), "{text}");
        assert_eq!(parsed.justification, None, "{text}");
    }
    assert!(DirectiveKind::DisableLine.is_line_scoped());
    assert!(DirectiveKind::DisableNextLine.is_line_scoped());
    assert!(!DirectiveKind::Disable.is_line_scoped());
    assert!(!DirectiveKind::Enable.is_line_scoped());
}

/// Ordinary comments are not directives,
///  including prose that mentions the tool or a keyword in the middle.
#[test]
fn ordinary_comments_are_not_directives() {
    for text in [
        "",
        "   ",
        " plain comment ",
        " see monochromatic-lint-disable for details",
        " monochromatic-lint is the tool",
        " Monochromatic-lint-disable -- r",
        "/ monochromatic-lint-disable -- r",
        " * monochromatic-lint-disable -- r",
    ] {
        assert_eq!(
            parse_comment(text, 0),
            ParsedComment::NotDirective,
            "{text:?}"
        );
    }
}

/// A first word that starts with the prefix but is not a keyword is malformed,
///  including near misses and a keyword fused to its dashes.
#[test]
fn near_miss_keywords_are_malformed() {
    for text in [
        " monochromatic-lint-disable-nextline -- r",
        " monochromatic-lint-disable-next-line-x -- r",
        " monochromatic-lint-disable-- r",
        " monochromatic-lint- ",
        " monochromatic-lint-DISABLE -- r",
        " monochromatic-lint-rules are described elsewhere",
    ] {
        match parse_comment(text, 0) {
            ParsedComment::Malformed(reason) => {
                assert!(reason.contains("is not a directive"), "{text:?}: {reason}");
            }
            other => panic!("{text:?} should be malformed, got {other:?}"),
        }
    }
}

/// The reason follows a run of two or more dashes set off by whitespace,
///  and is trimmed.
#[test]
fn the_justification_follows_the_dash_separator() {
    let cases: [(&str, Option<&str>); 10] = [
        (" monochromatic-lint-disable -- reason ", Some("reason")),
        (" monochromatic-lint-disable --- reason", Some("reason")),
        (
            " monochromatic-lint-disable \t--\tthe reason, with commas",
            Some("the reason, with commas"),
        ),
        (" monochromatic-lint-disable -- a -- b", Some("a -- b")),
        (" monochromatic-lint-disable", None),
        (" monochromatic-lint-disable --", None),
        (" monochromatic-lint-disable --   \n ", None),
        (" monochromatic-lint-disable - reason", None),
        (" monochromatic-lint-disable --reason", None),
        (" monochromatic-lint-disable rust/max-lines--reason", None),
    ];
    for (text, expected) in cases {
        let found: Option<String> = match parse_comment(text, 0) {
            ParsedComment::Directive(parsed) => parsed.justification,
            ParsedComment::Malformed(_) | ParsedComment::NotDirective => None,
        };
        assert_eq!(found.as_deref(), expected, "{text:?}");
    }
}

/// Dashes inside a word or after text without whitespace never separate a reason,
///  so such text stays part of the identifier list.
#[test]
fn dashes_without_surrounding_whitespace_are_not_a_separator() {
    let parsed: ParsedDirective = directive(" monochromatic-lint-disable rust/a--b, c -d -- r");
    assert_eq!(texts(&parsed), ["rust/a--b", "c -d"]);
    assert_eq!(parsed.justification.as_deref(), Some("r"));
    // A run that starts after text without a blank is skipped,
    //  and a later proper run is still found.
    let later: ParsedDirective = directive(" monochromatic-lint-disable x-- -- r");
    assert_eq!(texts(&later), ["x--"]);
    assert_eq!(later.justification.as_deref(), Some("r"));
}

/// Identifiers are split on commas,
///  trimmed,
///  and carry exact host byte positions even after multibyte blanks.
#[test]
fn rule_lists_are_split_trimmed_and_positioned() {
    let text: &str =
        " monochromatic-lint-disable-next-line  rust/max-lines ,markdown/single-h1\u{a0}, a -- r";
    let parsed: ParsedDirective = directive(text);
    assert_eq!(
        texts(&parsed),
        ["rust/max-lines", "markdown/single-h1", "a"]
    );
    for token in &parsed.rules {
        let RuleToken {
            text: name,
            start,
            end,
        } = token;
        // The content starts at host offset 100 in these controls.
        assert_eq!(&text[start - 100..end - 100], name.as_str());
    }
    assert_eq!(parsed.justification.as_deref(), Some("r"));
}

/// An empty list entry is malformed rather than silently dropped,
///  because dropping it could turn a typo into a wider directive.
#[test]
fn empty_list_entries_are_malformed() {
    for text in [
        " monochromatic-lint-disable a,,b -- r",
        " monochromatic-lint-disable a, -- r",
        " monochromatic-lint-disable ,a -- r",
        " monochromatic-lint-disable , -- r",
    ] {
        match parse_comment(text, 0) {
            ParsedComment::Malformed(reason) => assert!(reason.contains("empty entry"), "{reason}"),
            other => panic!("{text:?} should be malformed, got {other:?}"),
        }
    }
}

/// A blank list means every suppressible rule,
///  and surrounding whitespace of any kind is ignored.
#[test]
fn a_blank_list_means_no_identifiers() {
    let parsed: ParsedDirective = directive("\n\t monochromatic-lint-enable \r\n -- reason \r\n");
    assert!(parsed.rules.is_empty());
    assert_eq!(parsed.kind, DirectiveKind::Enable);
    assert_eq!(parsed.justification.as_deref(), Some("reason"));
}
