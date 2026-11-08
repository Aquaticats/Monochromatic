# BUG-20 find_anchored ignores a leading assertion that fails at the start of input

## Classification

- Type:
   correctness,
   soundness.
   `find_anchored` returns a match at offset 0 that
  the pattern's leading zero-width assertion forbids there.
- Phase:
   match time,
   the `find_anchored` path.
- Severity:
   soundness.
   `find_anchored` is documented as the longest match anchored
  at position 0;
   here it reports a match at 0 that does not satisfy the pattern,
   and
  `find_all` on the same pattern and input correctly reports the match elsewhere.
  The two public APIs contradict,
   and `find_anchored` is the wrong side.

## Minimal reproducer

```rust
//! Reproduction of the ignored leading assertion:
//! `find_all` correctly skips offset 0,
//! `find_anchored` returns a match there.

/// The regex type whose two entry points disagree.
use resharp::Regex;
let re = Regex::new(r"\B0").unwrap();
let hay = b"00";
// \B (non-word-boundary) is FALSE at offset 0 (start-of-input before a word char),
// so there is no match at 0; the only match is at offset 1 (word-to-word boundary).
assert_eq!(re.find_all(hay).unwrap(), vec![/* 1..2 */]);     // correct: matches at 1:2
assert!(re.find_anchored(hay).unwrap().is_none());           // FAILS: returns Some(0..1)
```

Command line (`repro --pair <hexpat> <hexhay>` prints `im=|fa=|le=`,
 where `le` is
`find_anchored().end`,
 `-1` for `None`):

```sh
repro --pair "$(printf '%s' '\B0' | xxd -p)" "$(printf '%s' 00 | xxd -p)"
# im=1|fa=1:2|le=1   -> find_all 1:2 (correct), find_anchored 0:1 (wrong)
```

## Observed behaviour

`find_anchored` distinguishes neither the polarity of a word boundary nor the
satisfiability of a leading lookbehind at offset 0.
 On `"00"`,
 default config:

```text
\b0       fa=0:1  le=1     both correct: \b holds at start (none-to-word)
\B0       fa=1:2  le=1     find_all correct (match at 1); find_anchored WRONG (0:1)
^0        fa=0:1  le=1     both correct: ^ holds at start
\A0       fa=0:1  le=1     both correct: \A holds at start
(?<=0)0   fa=1:2  le=1     find_all correct (a 0 precedes offset 1); find_anchored WRONG (0:1)
```

`\b0` and `\B0` give the identical `find_anchored` answer (`le=1`) even though `\b`
holds at offset 0 and `\B` does not,
 so `find_anchored` is not evaluating the
boundary at the start at all.
 The same over-match happens for a leading lookbehind
`(?<=0)`,
 which cannot hold at offset 0 (nothing precedes it) yet `find_anchored`
matches there.
 The breadth holds across the class shapes:
 `\B0+` (`le=2`),
`\B[0-9]`,
 and `\B\d` all return a spurious offset-0 match while `find_all` returns
the correct later span.

## Expected behaviour

`find_anchored(h)` must return a match only if the whole pattern,
 including any
leading zero-width assertion,
 holds at offset 0.
 For `\B0` on `"00"` it must return
`None`;
 for `(?<=0)0` on `"00"` it must return `None`.

## Independent corroboration

`find_all` is the correct side,
 so this needs no external oracle:
 `find_anchored`
contradicts `find_all` on the same engine.
 The `\B` semantics are also standard,
 and
the `regex` crate matches `\B0` on `"00"` at offset 1,
 agreeing with resharp's
`find_all` (`1:2`) and against resharp's `find_anchored` (`0:1`).

## Root cause

`find_anchored` (`resharp-engine/src/lib.rs:1838`) scans forward from position 0
with the generic forward scanner:

```rust
//! The anchored search:
//! it scans forward from offset 0 and returns the span it finds,
//! without first checking that a leading assertion holds at offset 0.
// resharp-engine/src/lib.rs:1847
let max_end = inner.fwd.scan_fwd_slow(&mut inner.b, 0, input)?;
if max_end != engine::NO_MATCH {
    Ok(Some(Match { start: 0, end: max_end }))
} else { Ok(None) }
```

`scan_fwd_slow(b, 0, input)` is entered without seeding the begin-of-input context
into the initial state,
 so a leading assertion whose truth depends on that context,
`\B` (false at a true start before a word char) or a lookbehind requiring prior
input,
 is evaluated as if the context were neutral and the assertion is effectively
skipped.
 The `find_all` path seeds the begin context (its initial nullability is
keyed on `pos_begin == 0`,
 see `engine.rs:818` and `:921`),
 which is why `find_all`
gets `\B0` and `(?<=0)0` right while `find_anchored` does not.

The fix is to start `find_anchored`'s scan from the same begin-anchored initial
state `find_all` uses at position 0,
 so the leading assertion is evaluated against a
real start of input.

## Affected configurations

Reproduces in the default and ascii configs (the word boundary is the trigger);
`\B` exists in every config,
 so the begin-context omission is config-independent.

## Relationship to other findings

- Distinct from BUG-3 (`is_match` vs `find_all`):
   there `find_all` is the wrong
  side;
   here `find_all` is correct and `find_anchored` is wrong.
- Distinct from BUG-13 (find_anchored leaks a lookahead's width into a zero-width
  span):
   that is a span-width error on a matching pattern;
   this is `find_anchored`
  reporting a match that should not exist because a leading assertion fails.
   Both are
  `find_anchored` mishandling leading zero-width assertions,
   so they may share the
  begin-context handling and are worth fixing together.
- The directed `find_anchored`-versus-`find_all` consistency oracle (`FANDIFF`)
  surfaced this;
   its `fa=empty|fan=0:N` and `fafirst=1:2|fan=0:1` buckets are
  dominated by `\b`/`\B` and leading-lookbehind patterns of this family.
