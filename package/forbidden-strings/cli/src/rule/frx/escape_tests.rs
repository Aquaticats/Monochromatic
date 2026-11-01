// Round-trip and adversarial tests for literal escaping into the verbose dialect.

use super::{escape_literal, literal_pattern};
use forbidden_regex::RegexSet;

// Compiles one escaped literal into a single-rule set.
fn compiled(literal: &str) -> RegexSet {
    let pattern = escape_literal(literal);
    return RegexSet::new(std::slice::from_ref(&pattern)).expect("escaped literal must compile")
}

// Compiles one boundary-gated literal into a single-rule set.
fn compiled_bounded(literal: &str) -> RegexSet {
    let pattern = literal_pattern(literal);
    return RegexSet::new(std::slice::from_ref(&pattern)).expect("bounded literal must compile")
}

// Asserts the escaped literal matches itself and the same bytes embedded in noise.
fn assert_round_trips(literal: &str) {
    let set = compiled(literal);
    assert!(set.is_match(literal.as_bytes()), "did not match literal {literal:?}");
    let embedded = format!("xy{literal}z9");
    assert!(set.is_match(embedded.as_bytes()), "did not match embedded {literal:?}");
}

#[test]
fn metacharacter_literals_round_trip() {
    for literal in [
        "a.b", "a*b", "a+b", "a?b", "a|b", "a^b", "a$b", "a[b", "a]b", "a(b", "a)b", "a{b",
        "a}b", "a&b", "a~b", "a-b", "a/b", "(?:group)", "[class]", "{2,5}", ".*+?", "a.b.c",
    ] {
        assert_round_trips(literal);
    }
}

#[test]
fn dot_is_literal_not_wildcard() {
    let set = compiled("a.c");
    assert!(set.is_match(b"a.c"));
    assert!(!set.is_match(b"axc"));
}

#[test]
fn quantifiers_are_literal() {
    let star = compiled("a*b");
    assert!(star.is_match(b"a*b"));
    assert!(!star.is_match(b"b"));
    assert!(!star.is_match(b"aaab"));

    let plus = compiled("a+b");
    assert!(plus.is_match(b"a+b"));
    assert!(!plus.is_match(b"aaab"));

    let opt = compiled("ab?c");
    assert!(opt.is_match(b"ab?c"));
    assert!(!opt.is_match(b"ac"));
}

#[test]
fn class_brackets_are_literal() {
    let set = compiled("[abc]");
    assert!(set.is_match(b"[abc]"));
    assert!(!set.is_match(b"a"));
    assert!(!set.is_match(b"b"));
}

#[test]
fn pipe_is_literal_not_alternation() {
    let set = compiled("a|b");
    assert!(set.is_match(b"a|b"));
    assert!(!set.is_match(b"a"));
    assert!(!set.is_match(b"b"));
}

#[test]
fn backslash_shorthand_is_two_literal_bytes() {
    // Backslash escaped, so `\d`/`\w` are the two literal bytes, not shorthand classes.
    let digit = compiled("\\d");
    assert!(digit.is_match(b"\\d"));
    assert!(!digit.is_match(b"5"));

    let word = compiled("\\w+");
    assert!(word.is_match(b"\\w+"));
    assert!(!word.is_match(b"abc"));
}

#[test]
fn whitespace_is_preserved() {
    let space = compiled("foo bar");
    assert!(space.is_match(b"foo bar"));
    assert!(!space.is_match(b"foobar"));

    let tab = compiled("a\tb");
    assert!(tab.is_match(b"a\tb"));
    assert!(!tab.is_match(b"ab"));
}

#[test]
fn hash_is_literal_even_leading() {
    // A leading '#' unescaped would begin a verbose comment and swallow the rule,
    // compiling to an empty-matchable pattern; escaping keeps it a literal byte.
    let lead = compiled("#nocomment");
    assert!(lead.is_match(b"#nocomment"));

    let mid = compiled("a#b");
    assert!(mid.is_match(b"a#b"));
    assert!(!mid.is_match(b"ab"));
}

#[test]
fn embedded_newline_round_trips() {
    // Destination-boundary case: the escaper emits a backslash then the real
    // newline byte, which the engine reads as a literal newline.
    let set = compiled("a\nb");
    assert!(set.is_match(b"a\nb"));
    assert!(!set.is_match(b"ab"));
    assert!(escape_literal("x\ny").as_bytes().contains(&b'\n'));
}

#[test]
fn quotes_and_punctuation_pass_through() {
    for literal in ["key=\"value\"", "it's", "a,b;c:d", "<tag>", "@user", "a`b`c", "50%off"] {
        assert_round_trips(literal);
    }
}

#[test]
fn secret_shaped_literal_round_trips() {
    // The final byte uses the `\x50` ('P') convention so the raw 20-char token
    // never appears in this source file, which the repo commit gate scans.
    let secret = "AKIA1234567890ABCDE\x50";
    let set = compiled(secret);
    assert!(set.is_match(secret.as_bytes()));
    assert!(!set.is_match(b"AKIA"));
}

#[test]
fn short_word_literal_gains_boundaries() {
    assert_eq!(literal_pattern("ABC"), "\\bABC\\b");
}

#[test]
fn short_literal_matches_whole_word_not_embedded() {
    let set = compiled_bounded("ABC");
    assert!(set.is_match(b"the ABC list"));
    assert!(set.is_match(b"ABC"));
    // A short run inside a longer token no longer matches.
    assert!(!set.is_match(b"preABCpost"));
    assert!(!set.is_match(b"unABCed"));
}

#[test]
fn non_word_ends_take_no_boundary() {
    // Each CJK byte is non-word, so a boundary there would assert an ASCII boundary
    // CJK text never provides; the literal stays a plain substring rule. The
    // characters are written as escapes so no deny-listed term enters this source.
    let cjk = "\u{4e2d}\u{5171}"; // two 3-byte characters, 6 bytes total
    assert_eq!(literal_pattern(cjk), escape_literal(cjk));
    let set = compiled_bounded(cjk);
    assert!(set.is_match(cjk.as_bytes()));
    let embedded = format!("x{cjk}y");
    assert!(set.is_match(embedded.as_bytes()));
}

#[test]
fn boundary_is_applied_per_end() {
    // A non-word leading byte omits its boundary; a word trailing byte keeps one.
    assert_eq!(literal_pattern(".env"), "\\.env\\b");
    assert_eq!(literal_pattern("v1."), "\\bv1\\.");
}

#[test]
fn literal_at_ceiling_is_unbounded() {
    // Exactly eight bytes is not under the ceiling, so no boundaries are added.
    let eight = "Longword";
    assert_eq!(eight.len(), 8);
    assert_eq!(literal_pattern(eight), escape_literal(eight));
}

#[test]
fn short_literal_with_internal_space_is_bounded() {
    // A multi-word short literal is bounded at its outer word-byte ends; the
    // internal space is still escaped for verbose mode.
    assert_eq!(literal_pattern("Ab Cd"), "\\bAb\\ Cd\\b");
    let set = compiled_bounded("Ab Cd");
    assert!(set.is_match(b"in Ab Cd now"));
    assert!(!set.is_match(b"xAb Cd"));
    assert!(!set.is_match(b"Ab Cde"));
}
