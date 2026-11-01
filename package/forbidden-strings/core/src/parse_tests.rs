// What:  unit tests for the parser front door: every accepted construct parses, and
//        every rejection path returns the right `CompileError` variant.
// Why:     This file groups the parse test cases so behavior changes fail near the code path
//          they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("parse", () => {
//   // test cases below
// });
// ```

use super::parse;
use crate::error::CompileError;

// What:    Asserts a pattern parses successfully.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function ok(pattern: string): void {
//   // Rust body below is the implementation.
// }
// ```
fn ok(pattern: &str) {
    assert!(parse(pattern).is_ok(), "expected {pattern:?} to parse, got {:?}", parse(pattern));
}

// What:    Asserts a pattern is rejected as a syntax / unsupported-construct error.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function syntax_err(pattern: string): void {
//   // Rust body below is the implementation.
// }
// ```
fn syntax_err(pattern: &str) {
    assert!(
        matches!(parse(pattern), Err(CompileError::Syntax { .. })),
        "expected {pattern:?} to be a syntax error, got {:?}",
        parse(pattern),
    );
}

#[test]
fn accepts_literals_and_escapes() {
    ok("abc");
    ok("A3T");
    ok("\\.");
    ok("\\[");
    ok("\\]");
    ok("\\(");
    ok("\\&");
    ok("\\~");
    ok("\\t");
    ok("a\\\\b");
}

#[test]
fn accepts_classes() {
    ok("[abc]");
    ok("[a-z]");
    ok("[a-zA-Z0-9_]");
    ok("[^0-9]");
    ok("[a-f0-9]");
    // What:    a leading `]` is a class member, not a close.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    ok("x[]a]");
}

#[test]
fn accepts_shorthands_and_dot() {
    ok("\\d");
    ok("\\w");
    ok("\\s");
    ok("\\D");
    ok("\\W");
    ok("\\S");
    ok(".");
}

#[test]
fn accepts_alternation_and_groups() {
    ok("(?:a|b)");
    ok("(?:a|b|c)");
    ok("(?:abc)");
    ok("(?:a|b)x");
}

#[test]
fn accepts_bounded_repetition() {
    ok("a{3}");
    ok("a{3,6}");
    ok("[a-z]{16}");
    // What:    optional needs a mandatory neighbour to stay non-nullable.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    ok("xa?");
    ok("AKIA[A-Z2-7]{16}");
}

#[test]
fn accepts_anchors() {
    ok("^abc");
    ok("abc$");
    ok("\\bword\\b");
    ok("^a[0-9]:");
}

#[test]
fn accepts_intersection_and_complement() {
    ok("[a-z]&[a-c]");
    ok("[a-z]&~(m)");
    ok("(?:[a-z]{4})&~(abcd)");
    ok("(?:AKIA[A-Z2-7]{16})&~(AKIA2{16})");
}

#[test]
fn rejects_unbounded_and_greedy_quantifiers() {
    syntax_err("a*");
    syntax_err("a+");
    syntax_err("a{2,}");
    syntax_err("a{,5}");
}

#[test]
fn rejects_stacked_quantifiers() {
    syntax_err("a??");
    syntax_err("a{2}{3}");
    syntax_err("a*?");
}

#[test]
fn rejects_bad_repetition_bounds() {
    syntax_err("a{3,1}");
    syntax_err("a{x}");
}

#[test]
fn repetition_count_at_the_cap_is_allowed_but_beyond_is_rejected() {
    // What:    The desugared-count cap is 1024: exactly at the cap parses, one past is
    //          rejected.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    ok("a{1024}");
    syntax_err("a{1025}");
    // What:    the upper bound is also capped.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    syntax_err("a{1,1025}");
}

#[test]
fn rejects_unbalanced_brackets() {
    syntax_err("(?:abc");
    syntax_err("[abc");
    syntax_err("abc)");
    syntax_err("~(abc");
}

#[test]
fn rejects_unsupported_groups() {
    // What:    capturing group.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    syntax_err("(abc)");
    // What:    lookahead.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    syntax_err("(?=a)");
    // What:    lookbehind.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    syntax_err("(?<=a)");
    // What:    named capture.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    syntax_err("(?P<n>a)");
}

#[test]
fn rejects_unknown_escapes() {
    syntax_err("\\q");
    syntax_err("\\xff");
}

#[test]
fn rejects_empty_matchable_patterns() {
    assert_eq!(parse("a?"), Err(CompileError::EmptyMatchable));
    assert_eq!(parse("a{0}"), Err(CompileError::EmptyMatchable));
    assert_eq!(parse("~(abc)"), Err(CompileError::EmptyMatchable));
    assert_eq!(parse("(?:a)?"), Err(CompileError::EmptyMatchable));
}

#[test]
fn rejects_trailing_and_stray_operators() {
    syntax_err("a|");
    syntax_err("|a");
    syntax_err("a&");
}

#[test]
fn verbose_mode_ignores_unescaped_whitespace() {
    // What:    Whitespace outside classes is ignored, so these all parse to the same node.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let base = parse("abc").expect("parse the baseline pattern");
    assert_eq!(parse("a b c").expect("parse with spaces between literals"), base);
    assert_eq!(parse("a\tb\tc").expect("parse with tabs between literals"), base);
    assert_eq!(parse("a\nb\nc").expect("parse with newlines between literals"), base);
    assert_eq!(parse("  abc  ").expect("parse with leading and trailing whitespace"), base);
}

#[test]
fn first_column_hash_is_a_comment_to_end_of_line() {
    // What:    A `#` at the start of a line comments out the rest of that line.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert_eq!(
        parse("# a header comment\nabc").expect("parse a pattern with a leading comment"),
        parse("abc").expect("parse the baseline pattern"),
    );
    assert_eq!(
        parse("abc\n# trailing comment line\n").expect("parse a pattern with a trailing comment"),
        parse("abc").expect("parse the baseline pattern"),
    );
}

#[test]
fn escaped_and_class_whitespace_stays_literal() {
    // What:    An escaped space and a space inside a class are real bytes, not ignored.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert_ne!(
        parse("a\\ b").expect("parse a pattern with an escaped space"),
        parse("ab").expect("parse a pattern without the escaped space"),
    );
    assert_ne!(
        parse("[ ]x").expect("parse a pattern with a space inside a class"),
        parse("x").expect("parse a pattern without the class space"),
    );
}

// What:    Asserts a pattern is a syntax error reported at exactly `pos`.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function syntax_at(pattern: string, pos: number): void {
//   // Rust body below is the implementation.
// }
// ```
fn syntax_at(pattern: &str, pos: usize) {
    match parse(pattern) {
        Err(CompileError::Syntax { pos: got, .. }) => {
            assert_eq!(got, pos, "wrong error offset for {pattern:?}");
        }
        other => panic!("expected a syntax error at {pos} for {pattern:?}, got {other:?}"),
    }
}

// What:    Asserts a pattern is a syntax error whose message contains `needle`.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function syntax_msg(pattern: string, needle: string): void {
//   // Rust body below is the implementation.
// }
// ```
fn syntax_msg(pattern: &str, needle: &str) {
    match parse(pattern) {
        Err(CompileError::Syntax { message, .. }) => {
            assert!(message.contains(needle), "{pattern:?} message {message:?} lacks {needle:?}");
        }
        other => panic!("expected a syntax error mentioning {needle:?} for {pattern:?}, got {other:?}"),
    }
}

#[test]
fn syntax_errors_report_the_offending_offset() {
    // What:    The cursor's reported position must be the real byte offset of the problem, not
    //          a constant: a `*` after `abc` is at offset 3, and a `+` after a five-byte group
    //          at 5.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    syntax_at("abc*", 3);
    syntax_at("(?:a)+", 5);
}

#[test]
fn trailing_and_unmatched_input_is_rejected() {
    // What:    A stray close after a complete expression is a syntax error, never silently
    //          dropped.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    syntax_err("a)");
    syntax_err("(?:a)b]");
}

#[test]
fn class_range_endpoints_and_lookahead() {
    // What:    A trailing `-` before `]` is a literal (the range lookahead must read the `]`
    //          exactly one byte ahead), and an equal-endpoint range `[a-a]` is the single
    //          byte, not "out of order"; only a genuinely descending range is rejected.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    ok("[a-]");
    ok("[a-a]");
    ok("[-a]");
    syntax_err("[z-a]");
    // What:    `[a-]` and `[-a]` are both the set {`a`, `-`}, regardless of where the `-`
    //          sits.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert_eq!(
        parse("[a-]").expect("parse a trailing-dash class"),
        parse("[-a]").expect("parse a leading-dash class"),
    );
}

#[test]
fn repetition_bounds_accept_equal_and_reject_descending() {
    // What:    `{n,n}` is a valid exact count; the bound test is strict `<`, so `m == n` is
    //          allowed and only `n > m` is rejected.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    ok("a{2,2}");
    syntax_err("a{3,2}");
    syntax_msg("a{1,}", "unbounded");
}

#[test]
fn unsupported_quantifiers_carry_their_own_message() {
    // What:    `*`/`+` are rejected in postfix position (after an atom) with a message
    //          pointing at the `{m,n}` replacement, and in atom position (no preceding atom)
    //          as plain errors.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    syntax_msg("a*", "{0,n}");
    syntax_msg("a+", "{1,n}");
    syntax_err("*");
    syntax_err("+");
    syntax_err("(?:*)");
}

#[test]
fn metacharacters_without_an_operand_are_rejected() {
    // What:    A leading operator, a bare brace with no preceding atom, and a mix of `&`/`|`
    //          at one level are all errors. `{` is tested alone, not `{3}`: a trailing `}`
    //          would itself be rejected as unmatched, masking whether the `{` arm fired.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    syntax_err("|a");
    syntax_err("&a");
    syntax_err("{");
    syntax_msg("(?:a)|(?:b)&(?:c)", "mix");
}

#[test]
fn empty_matchable_patterns_are_rejected_only_when_realizable() {
    // What:    `^$` matches the empty line, so it is empty-matchable and rejected. `^\b$` is
    //          nullable only in the impossible context (a word boundary at an empty line,
    //          which has no word byte on either side), so it is NOT empty-matchable and must
    //          compile.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(matches!(parse("^$"), Err(CompileError::EmptyMatchable)), "^$ should be rejected");
    assert!(parse("^\\b$").is_ok(), "^\\b$ is not realizably empty-matchable: {:?}", parse("^\\b$"));
    // What:    A bare word boundary is realizably empty and stays rejected.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(matches!(parse("\\b"), Err(CompileError::EmptyMatchable)));
}

#[test]
fn mid_line_hash_is_a_literal_not_a_comment() {
    // What:    A `#` that is not the first column of its line is a literal byte; only the
    //          first-column comment rule (guarded by `at_line_start`) strips to end of line.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let with_hash = parse("a#b").expect("parse a pattern with a mid-line hash");
    // What:    identical to the explicitly-escaped form.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    assert_eq!(with_hash, parse("a\\#b").expect("parse a pattern with an escaped hash"));
    // What:    the `#b` was NOT comment-stripped away.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    assert_ne!(with_hash, parse("a").expect("parse a pattern without the hash"));
}
