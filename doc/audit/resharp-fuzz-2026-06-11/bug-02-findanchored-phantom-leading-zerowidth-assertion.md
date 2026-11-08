# BUG-2 find_anchored returns a phantom match for a leading zero-width assertion false at offset 0

- Type:
   correctness,
   soundness.
   `find_anchored` returns `Some(match)` for inputs
  that do not match at all.
- Phase:
   match time,
   the `find_anchored` path.
- Severity:
   soundness.
   `find_anchored` contradicts `is_match` and `find_all` on
  the same engine and input:
   `find_anchored = Some` implies a match exists,
   so
  `is_match` must be true,
   but it is false.
- Affected:
   all six limits-enabled configs (config-independent).
- Discovery:
   the FANINCONSIST oracle (`find_anchored = Some` while
  `is_match = false`) over the 40k adversarial corpus:
   122 distinct triggers.

## Reproducer

```rust
//! Reproduction of the phantom leading assertion:
//! `find_anchored` returns a match for patterns that begin with an assertion that cannot hold at offset zero,
//! where `is_match` and `find_all` find none.

/// The regex type and the match type of the spans.
use resharp::{Regex, Match};

// leading lookbehind that fails at offset 0 (nothing precedes the start):
let re = Regex::new(r"(?<=a)").unwrap();
assert_eq!(re.is_match(b"b").unwrap(), false);          // no match anywhere
assert_eq!(re.find_all(b"b").unwrap(), vec![]);         // none
assert_eq!(re.find_anchored(b"b").unwrap(), None);      // FAILS: returns Some(0:0)

// leading non-word-boundary that fails at a word char at start of input:
let re = Regex::new(r"\BU").unwrap();
assert_eq!(re.is_match(b"U").unwrap(), false);          // \B is false before 'U' at start
assert_eq!(re.find_anchored(b"U").unwrap(), None);      // FAILS: returns Some(0:1)
```

Harness:

```sh
# 283f3c3d6129 = "(?<=a)", 62 = "b"
repro --show 283f3c3d6129 62 0
# compile=ok|im=Ok(false)|fa=|fan=Ok(Some((0, 0)))|stream=
```

## Observed versus expected

`find_anchored(hay)` is the longest match anchored at offset 0.
 If the pattern's
leading assertion is false at offset 0 (a lookbehind at the start of input,
 `\B`
before a word char at the start,
 etc.),
 there is no anchored match and the result
must be `None`.
 resharp returns `Some(0:_)`,
 which both `is_match` and `find_all`
contradict (both report no match).
 The body of the pattern is evaluated but the
leading zero-width assertion's begin-context is not,
 so `find_anchored` accepts a
position the assertion forbids.

## Scope

122 distinct triggering patterns in the corpus.
 93 are a leading lookbehind
`(?<=...)...`;
 the rest are `\b` / `\B` led.
 The minimal forms are
`(?<=a)` (returns `Some(0:0)`),
 `(?<=a)b` on `"b"` (returns `Some(0:1)`),
 and
`\BU` on `"U"` (returns `Some(0:1)`).
 Each fires `find_anchored = Some` while
`is_match = false`.

## Relationship to 2026-06-04 BUG-20

This is BUG-20 ("find_anchored ignores a leading assertion that fails at the start
of input"),
 reported fixed.
 It is only partially fixed:
 the specific `\B0` on
`"00"` case is now correct (`find_anchored = None`),
 but the general case (leading
lookbehind,
 and `\B` before any other word char) is still live.
 Treating the
narrow patch as a full fix is the documentation gap the user flagged.

## Source pointer and mechanism

`find_anchored` is `resharp-engine/src/lib.rs:1891`.
 v0.6.12 added a leading-
lookbehind guard (the partial BUG-20 fix):

```rust
//! The anchored search with the guard for a leading lookbehind,
//! whose context-free fallback ignores other leading assertions.
// resharp-engine/src/lib.rs:1901
if self.has_lb && !self.rev_trivial && !self.always_nullable {
    let first = inner.fwd_ts.scan_fwd_first_null_from(&mut inner.b,
        ldfa::DFA_INITIAL as u32, 0, input)?;
    if first.2 { return Ok(None); }
}
Ok(inner.fwd.scan_fwd_slow(&mut inner.b, 0, input)?
    .map(|end| Match { start: 0, end }))   // line 1908: context-free fallback
```

Two ways the leading assertion is missed:

- the guard excludes `always_nullable` and `rev_trivial` patterns,
   so a nullable
  leading lookbehind like `(?<=a)` (always_nullable) skips the check entirely and
  falls through to the context-free `scan_fwd_slow(0)` at line 1908,
   which scans
  from offset 0 without evaluating the lookbehind,
   returning `Some(0:_)`;
- for `\B`-led patterns the guard may be entered but the
  `scan_fwd_first_null_from` null-scan check does not detect that `\B` is false
  before a word char at the start,
   so `first.2` is false and it again falls
  through to line 1908.

The fix is to make the leading-assertion / begin-context evaluation
unconditional in `find_anchored` (or to fold `find_anchored` into the same
match-enumeration core as `find_all`,
 which gets these cases right),
 rather than
gating it on `has_lb && !rev_trivial && !always_nullable`.
 This is the same
"each driver re-derives the assertion logic" structural issue as `code-quality.md`
describes.
