# BUG-13 a lookahead leaks its body width into a zero-width match span

## Classification

- Type:
   correctness,
   wrong match length,
   over-long `find_all` span.
- Phase:
   match time,
   `find_all` span computation.
- Severity:
   soundness.
   `is_match` is correct (a zero-width match does exist),
   so
  the `is_match` vs `find_all` self-consistency oracle does not fire.
   The defect
  is purely in the reported span length.
   Found by the Lean ground-truth oracle
  (the non-anchor leftmost-longest position round) and corroborated by the dotnet
  engine and by resharp's own `find_anchored`.

## Minimal reproducer

```rust
//! Reproduction of the lookahead width leak:
//! a pattern that is one lookahead must match with zero width,
//! but the match returned is one byte wide.

/// The regex type,
/// its options and the match type that `find_all` returns.
use resharp::{Regex, RegexOptions, Match};
let re = Regex::with_options(r"(?=(?=c)c{1,3})", RegexOptions::default()).unwrap();
// the whole pattern is a lookahead, a zero-width assertion, so any match is zero-width
assert_eq!(re.find_all(b"c").unwrap(), vec![Match { start: 0, end: 0 }]);
// FAILS: resharp returns [Match { start: 0, end: 1 }]
```

Command line:

```sh
repro --pair "$(printf '%s' '(?=(?=c)c{1,3})' | xxd -p | tr -d '\n')" "$(printf '%s' c | xxd -p)"
# rust:   im=1|fa=0:1|le=0   (find_all says 0:1, find_anchored end says 0)
# dotnet: im=1|fa=0:0|le=0   (correct)
# Lean llmatch: 0:0          (correct)
```

## Observed behaviour

The whole pattern `(?=(?=c)c{1,3})` is a single lookahead,
 so a successful match
consumes nothing and must be zero-width wherever the lookahead body succeeds.
resharp's `find_all` instead returns a span whose end is advanced past the start
by the length the lookahead body consumed internally.
 On longer input the error
is systematic,
 one extra unit per span:

```text
hay="c"    rust fa=0:1        dotnet fa=0:0
hay="cc"   rust fa=0:1,1:2    dotnet fa=0:0,1:1
hay="ccc"  rust fa=0:1,1:2,2:3 dotnet fa=0:0,1:1,2:2
```

resharp's own `find_anchored` returns end 0 for `"c"` (the correct zero-width
end),
 so `find_anchored` and `find_all` disagree on the end of the same match
that both place at offset 0.

## Expected behaviour

A pattern that is entirely a lookahead matches the empty string,
 so every span is
`(p, p)`.
 Lean and the dotnet engine both return zero-width spans.

## Distinct triggers and isolation

The leak needs two ingredients in the lookahead body:
 a leading nested lookahead,
and a following factor that can match a variable length.
 Removing either makes
`find_all` correct again.

```text
trigger (rust find_all wrong, dotnet and Lean give zero-width):
  (?=(?=c)c{1,3})       on "c"   -> rust 0:1
  (?=(?=c)c{1,2})       on "c"   -> rust 0:1
  (?=(?=c)c?c)          on "c"   -> rust 0:1
  (?=(?=[a-z])(?:a)?c)  on "c"   -> rust 0:1
  (?=(?=[a-z])[a-z]{0}(?:a)?c{1,3}) on "abc" -> rust 2:3

correct (no leak):
  (?=(?=c)c)            on "c"   -> 0:0   (no variable factor)
  (?=(?=c)cc?)          on "c"   -> 0:0   (variable factor is not leading)
  (?=(?:a)?c)           on "c"   -> 0:0   (no nested lookahead)
  (?=[a-z]{0}c{1,3})    on "c"   -> 0:0   (leading factor is a repeat, not a lookahead)
  (?=c{1,3})            on "ccc" -> 0:0,1:1,2:2  (plain lookahead, no nesting)
```

## Root cause

The match-end computation for a top-level lookaround treats the lookaround as if
it consumes its body when the body begins with a nested zero-width assertion
followed by a nullable-prefixed factor.
 The lookaround node handling lives around
`resharp-engine/src/engine.rs:128` (`Kind::Lookahead | Kind::Lookbehind`) and the
reverse-scan match-end machinery (`collect_rev_inner`,
 `handle_rev_end`,
`scan_rev_from`).
 `find_anchored` computes the same match's end as 0,
 so the two
end-finding paths diverge;
 the exact divergent line is not yet pinned.

## Notes

- This is a ground-truth-only find,
   like BUG-12:
   `is_match` is right and the two
  `find_all` spans are internally consistent with each other,
   so the INCONSIST,
  BOUNDS,
   OVERLAP,
   HARDDIFF,
   and STREAM oracles all stay silent.
   Only the verified
  Lean semantics and the dotnet reference expose the wrong span length.
   The one
  internal signal is `find_anchored` end (0) disagreeing with `find_all` end (1).
- Distinct from BUG-3 (existence disagreement),
   BUG-10 (a dropped trailing
  zero-width match),
   and BUG-12 (wrong nullability).
   Here the match exists,
   is
  found,
   and is at the right offset;
   only its length is wrong.
