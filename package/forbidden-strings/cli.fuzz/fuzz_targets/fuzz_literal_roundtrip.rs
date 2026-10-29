// What:  drive the scanner's real literal-to-verbose-dialect escaper (`escape_literal`)
//        over arbitrary strings, compile its output as a single-pattern `RegexSet`, and
//        assert the round-trip: the escaped pattern compiles, matches the literal
//        itself, and never matches empty input. A fixed adversarial battery runs once
//        per process so the always-verbose boundary cases (spaces, leading `#`, quotes,
//        backslashes, metacharacters, escape sequences, newlines) are always exercised.
// Why:   literal-to-dialect escaping is a syntax-boundary transformer: it must turn any
//        input bytes into destination-grammar (verbose regex) that matches exactly those
//        bytes. A dropped escape would either fail to compile (caught by the compile
//        assertion) or, for `#`, swallow the rule into a verbose comment that compiles
//        to an empty-matchable pattern (caught by the empty-input negative). The engine
//        swap (#385) deleted the old walker this target used to check; the escaper is the
//        surviving syntax-boundary surface, so this is where the round-trip now lives.

#![no_main]

use libfuzzer_sys::fuzz_target;

use forbidden_strings::fuzz_api::{escape_literal, RegexSet};
use forbidden_strings_fuzz::generators::redacted_fingerprint;

// Cap on literal length: a pure literal compiles linearly, so any bounded length is
// safe, but capping keeps each iteration cheap. Matches the pre-swap target's `256`.
const MAX_LITERAL_CHARS: usize = 256;

// The adversarial always-verbose cases the escaper must survive. Each is a destination-
// grammar boundary: unescaped, it would change meaning in the engine's verbose,
// multiline dialect. The battery runs once per process (see `run_battery`) so a bounded
// smoke pass exercises every one even with an empty corpus.
const ADVERSARIAL: &[&str] = &[
    // Whitespace: swallowed by verbose mode unless escaped.
    " ",
    "a b c",
    "  leading spaces",
    "trailing spaces  ",
    "tab\there",
    // `#`: a leading one begins a verbose comment (would swallow the whole rule); an
    // interior one begins a trailing comment. Both must stay literal bytes.
    "#comment",
    "a#b",
    "###",
    // Quotes and ordinary punctuation: not metacharacters, must pass through and match.
    "key=\"value\"",
    "it's",
    "a,b;c:d <tag> @user 50% off",
    // Backslashes and escape-sequence lookalikes: a backslash is itself escapable, so
    // `\n` in the input is two literal bytes, not a newline.
    "\\",
    "\\\\",
    "back\\slash",
    "\\n",
    "\\t",
    "\\x50",
    // Regex metacharacters: each must become a literal, not its special form.
    ".*+?|()[]{}^$&~",
    "a.b.c",
    "(?:group)",
    "[a-z]",
    "{2,5}",
    // Newlines: escaped to a backslash then the real byte, read as a literal newline.
    "line1\nline2",
    "\r\n",
    "a\nb\nc",
    // Non-ASCII: multibyte UTF-8 is never in the escape set, passes through unchanged
    // (two-byte é, three-byte € and ¥).
    "café",
    "price €5 or ¥100",
];

// What:  check the escape round-trip for one literal.
// Why:   one code path for both the fuzzer's arbitrary input and the fixed battery, so
//        the invariants and their redacted messages stay identical.
fn check(literal: &str) {
    // An empty literal escapes to an empty pattern (empty-matchable); it is not a
    // meaningful rule, so skip it rather than assert on the engine's empty handling.
    if literal.is_empty() {
        return;
    }

    // What:  escape the literal, then compile it as a lone-pattern set.
    // Why:   `RegexSet::new` and `RegexSet::from_bytes` are the only non-logging engine
    //        entry points, so a failing pattern's bytes never reach a subscriber; on
    //        failure we report only the redacted fingerprint, never the pattern.
    let pattern = escape_literal(literal);
    let set = match RegexSet::new(std::slice::from_ref(&pattern)) {
        Ok(set) => set,
        Err(_) => {
            panic!(
                "escape_literal produced a non-compiling verbose-dialect pattern: {}",
                redacted_fingerprint(literal.as_bytes()),
            )
        }
    };

    // Positive: the escaped pattern matches the exact literal bytes it came from.
    assert!(
        set.is_match(literal.as_bytes()),
        "escaped literal did not match itself: {}",
        redacted_fingerprint(literal.as_bytes()),
    );

    // Negative: a one-or-more-byte literal cannot match empty input. A leading `#`
    // swallowed into a verbose comment compiles to an empty-matchable pattern that
    // WOULD match empty input, so this is the direct comment-swallow regression guard.
    assert!(
        !set.is_match(b""),
        "escaped literal matched empty input (comment-swallow / empty-matchable regression): {}",
        redacted_fingerprint(literal.as_bytes()),
    );
}

// What:  run the fixed adversarial battery exactly once per fuzz process.
// Why:   guarantees the boundary cases are exercised on every campaign (including a
//        bounded smoke pass) without a per-iteration cost that would slow the fuzzer.
fn run_battery() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        for literal in ADVERSARIAL {
            check(literal);
        }
    });
}

fuzz_target!(|s: String| {
    run_battery();

    // Bound the literal so a single iteration stays cheap; a pure literal compiles
    // linearly regardless of length, so truncation loses no invariant.
    let literal: String = s.chars().take(MAX_LITERAL_CHARS).collect();
    check(&literal);
});
