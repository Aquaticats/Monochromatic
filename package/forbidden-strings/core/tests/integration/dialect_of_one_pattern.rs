//! What:
//!     End-to-end tests of the pattern dialect:
//!  which constructs parse,
//!  which are rejected,
//!  and what
//!  each accepted construct matches.
//! Why:
//!      This file is the Rust module that groups the dialect tests,
//!  so the test binary gives
//!          them one namespace and the shared key stays in `integration.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe("the pattern dialect", ...) with the cases below.
//! ```

/// What:
///     Imports the key each AWS-shaped rule is checked against,
///  defined once for the whole binary.
/// Why:
///      The tests below assert against that one literal;
///  importing it keeps one definition
///          instead of a copy per test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { AKIA_KEY } from "../integration";
/// ```
use super::AKIA_KEY;

/// What:
///     Imports the one-pattern compiler and the error type its rejections carry.
/// Why:
///      The tests below call `compile` and match `CompileError` variants directly;
///  importing
///          them keeps each test focused on the dialect instead of the full path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CompileError, compile } from "forbidden_regex";
/// ```
use forbidden_regex::{
    CompileError,
    compile,
};
#[test]
fn literal_and_search() {
    let re = compile("AKIA[A-Z2-7]{16}").expect("compile a leading-literal AKIA pattern");
    assert!(re.is_match(b"key=AKIAABCDEFGHIJKLMNO\x50;"));
    assert!(!re.is_match(b"no key here"));
    // What:    Wrong alphabet: '0' and '1' are not in [A-Z2-7].
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(!re.is_match(b"AKIA0123456789ABCDEF"));
}

#[test]
fn anchors_and_word_boundary() {
    let start = compile("^abc").expect("compile a start-anchored pattern");
    assert!(start.is_match(b"abcdef"));
    assert!(!start.is_match(b"xabcdef"));

    let end = compile("abc$").expect("compile an end-anchored pattern");
    assert!(end.is_match(b"xxabc"));
    assert!(!end.is_match(b"abcd"));

    let word = compile("\\bcat\\b").expect("compile a word-boundary pattern");
    assert!(word.is_match(b"a cat sat"));
    assert!(!word.is_match(b"category"));
}

#[test]
fn classes_and_shorthands() {
    let re = compile("[a-z]\\d[^0-9]").expect("compile a class-and-shorthand pattern");
    assert!(re.is_match(b"x5!"));
    assert!(!re.is_match(b"x55"));
    let neg = compile("a[^bc]d").expect("compile a negated-class pattern");
    assert!(neg.is_match(b"axd"));
    assert!(!neg.is_match(b"abd"));
}

#[test]
fn repetition_bounds() {
    let re = compile("a{2,4}").expect("compile a bounded-repetition pattern");
    assert!(!re.is_match(b"bab"));
    assert!(re.is_match(b"baab"));
    assert!(re.is_match(b"aaaa"));
    let exact = compile("x{3}").expect("compile an exact-repetition pattern");
    assert!(exact.is_match(b"xxx"));
    assert!(!exact.is_match(b"yxxy"));
}

#[test]
fn unsupported_constructs_rejected() {
    assert!(matches!(compile("a*"), Err(CompileError::Syntax { .. })));
    assert!(matches!(compile("a+"), Err(CompileError::Syntax { .. })));
    assert!(matches!(compile("a{2,}"), Err(CompileError::Syntax { .. })));
    assert!(matches!(compile("(abc)"), Err(CompileError::Syntax { .. })));
    assert!(matches!(compile("\\x41"), Err(CompileError::Syntax { .. })));
    // What:    Multi-atom operand of '&' must be wrapped.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(matches!(compile("ab&cd"), Err(CompileError::Syntax { .. })));
}

#[test]
fn alternation_intersection_complement() {
    // What:    Operands must be single atoms, so multi-letter branches are wrapped.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let alt = compile("(?:(?:cat)|(?:dog))").expect("compile an alternation pattern");
    assert!(alt.is_match(b"a dog"));
    assert!(!alt.is_match(b"a bird"));
    // What:    The unwrapped form is rejected by the grammar.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    assert!(matches!(
        compile("(?:cat|dog)"),
        Err(CompileError::Syntax { .. })
    ));

    // What:    The canonical set-algebra case, wrapped to the single-atom-operand grammar: an
    //          AWS-style key, excluding the documented all-2s placeholder.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let aws = compile("(?:\\b(?:(?:A3T[A-Z0-9])|(?:AKIA)|(?:ASIA))[A-Z2-7]{16}\\b)&~(AKIA2{16})")
        .expect("compile the AWS set-algebra pattern");
    assert!(aws.is_match(AKIA_KEY.as_bytes()));
    assert!(aws.is_match(format!("prefix {AKIA_KEY} suffix").as_bytes()));
    assert!(!aws.is_match(b"AKIA222222222222222\x32"));
}

#[test]
fn empty_matchable_rejected() {
    assert!(matches!(compile("a?"), Err(CompileError::EmptyMatchable)));
    assert!(matches!(
        compile("~(abc)"),
        Err(CompileError::EmptyMatchable)
    ));
}

#[test]
fn dialect_constructs_have_their_semantics() {
    // What:    `.` is any byte except newline.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let dot = compile(".").expect("compile the dot pattern");
    assert!(dot.is_match(b"x"));
    assert!(dot.is_match(b"\t"));
    assert!(!dot.is_match(b"\n"));
    assert!(!dot.is_match(b""));

    // What:    `{n}` is an exact count, not a literal brace.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let three = compile("a{3}").expect("compile an exact-count pattern");
    assert!(three.is_match(b"aaa"));
    // What:    contains a 3-run.
    // Why:     The nearby assertion or value needs this note so the test records the exact
    //          behavior being pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same assertion or value, with the important expectation named above.
    // ```
    assert!(three.is_match(b"aaaa"));
    assert!(!three.is_match(b"aa"));
    assert!(!three.is_match(b"a{3}"));

    // What:    `{n,m}` is a bounded range.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let range = compile("xa{2,3}").expect("compile a bounded-range pattern");
    assert!(range.is_match(b"xaa"));
    assert!(range.is_match(b"xaaa"));
    assert!(!range.is_match(b"xa"));

    // What:    `?` is zero-or-one of the preceding atom.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let opt = compile("xa?b").expect("compile an optional-atom pattern");
    assert!(opt.is_match(b"xb"));
    assert!(opt.is_match(b"xab"));
    assert!(!opt.is_match(b"xaab"));
}

#[test]
fn nested_group_repetition_compiles_and_matches() {
    // What:    Reasonable nested group repetition compiles fast and matches correctly.
    // Why:     The test uses this setup or assertion to pin the behavior named by the test
    //          function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
    // ```
    let triple = compile("(?:ab){3}").expect("compile a repeated-group pattern");
    assert!(triple.is_match(b"ababab"));
    assert!(!triple.is_match(b"abab"));
    let nested = compile("(?:[a-z]{2}){2}").expect("compile a nested-repeated-group pattern");
    assert!(nested.is_match(b"abcd"));
    assert!(!nested.is_match(b"ab1d"));
}

// What:    libFuzzer (roundtrip/differential) found these deeply-nested bounded repetitions
//          (and complement over them) blow up the DFA build's residuals. With the
//          residual-size guard, compile now TERMINATES (rejects or falls back to counting)
//          instead of OOM. Marked #[ignore]: even guarded, the worst case takes seconds (more
//          in a debug build), which would dominate the default suite and per-mutant mutation
//          runs; run on demand with `cargo test -- --ignored`. The engine guard plus the fuzz
//          corpus are the live regression protection.
// Why:     The program needs this named step so callers can reuse the behavior without copying
//          its body.
//
// In TS you'd write (pseudocode):
// ```ts
// function pathological_nested_repetition_terminates_without_oom(): void {
//   // Rust body below is the implementation.
// }
// ```
#[test]
#[ignore = "slow pathological compile; the residual guard is exercised by fuzzing"]
fn pathological_nested_repetition_terminates_without_oom() {
    let _ = compile("(?:(?:(?:(?:[^0-9]a\\|?){3}){2}){3,4}){4,4}");
    let _ = compile("~((?:(?:(?:\\S{4,4}\\d{4,4})a?){4,4}a?){3}a)a");
}
