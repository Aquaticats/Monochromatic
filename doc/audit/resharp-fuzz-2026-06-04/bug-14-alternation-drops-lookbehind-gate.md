# BUG-14 a nullable alternation sibling drops a lookbehind gate in find_all

## Classification

- Type:
   correctness,
   wrong match length,
   `find_all` returns a span a lookbehind
  forbids.
- Phase:
   match time,
   `find_all` longest-end determination.
- Severity:
   soundness.
   `is_match` is correct (the nullable branch genuinely
  matches),
   so the `is_match` vs `find_all` self-consistency oracle stays silent.
  The defect is the reported span length.
   Found by the Lean leftmost-longest
  position round and corroborated two ways:
   the Lean verified semantics,
   and
  resharp's own behaviour on the lookbehind branch in isolation.

## Minimal reproducer

```rust
//! Reproduction of the dropped lookbehind gate:
//! an alternation takes the branch whose lookbehind cannot hold at the start of the input.

/// The regex type,
/// its options and the match type that `find_all` returns.
use resharp::{Regex, RegexOptions, Match};
let re = Regex::with_options(r"(|(?<=[a-z])b)", RegexOptions::default()).unwrap();
// branch 2's lookbehind cannot hold at offset 0, so the only legal match there is
// the empty branch; leftmost-longest gives the zero-width match
assert_eq!(re.find_all(b"b").unwrap()[0], Match { start: 0, end: 0 });
// FAILS: resharp returns Match { start: 0, end: 1 }, taking branch 2 and dropping
// the lookbehind gate
```

Command line:

```sh
repro --pair "$(printf '%s' '(|(?<=[a-z])b)' | xxd -p | tr -d '\n')" "$(printf '%s' b | xxd -p)"
# rust:   im=1|fa=0:1,1:1|le=1   (find_all first span 0:1, wrong)
# Lean llmatch: 0:0              (correct)
```

## Observed behaviour

`(|(?<=[a-z])b)` is `(empty | (?<=[a-z])b)`.
 On `"b"` the second branch needs a
letter immediately before offset 0;
 there is none,
 so that branch cannot match at
offset 0 and the only match there is the empty branch,
 a zero-width match.
 resharp
instead returns the span `0:1`,
 the length of the second branch,
 as if the
lookbehind held.
 The same pattern on `"xb"` is handled correctly (`0:0` then
`1:2`),
 because at offset 1 the lookbehind genuinely holds;
 only the
start-of-input position,
 where the gate must reject the longer branch,
 is wrong.

## Expected behaviour

Leftmost-longest still has to honour the lookbehind.
 The longest legal match at
offset 0 is the empty branch,
 so `find_all`'s first span is `0:0`.

## Why this is a real bug, two independent confirmations

- Lean ground truth,
   with a longest-preference control that rules out the
  alternative explanation.
   `(ε | b)` on `"b"` is `0:1` in Lean (it does prefer the
  longer branch),
   while `(ε | (?<=[a-z]) b)` on `"b"` is `0:0` (the lookbehind
  makes the longer branch illegal).
   So Lean prefers longest in general yet returns
  the empty match here precisely because of the gate.
- resharp self-consistency.
   The branch in isolation,
   `(?<=[a-z])b` on `"b"`,
  returns no match (`im=0`,
   empty `find_all`),
   so resharp itself enforces the
  lookbehind there.
   Only inside the alternation,
   where a nullable sibling makes
  offset 0 a valid match origin,
   does the gate get dropped.
   The composition
  changes a sub-result that should not change.

## Distinct triggers and boundary

The trigger needs a nullable or empty sibling branch next to a branch that begins
with a lookbehind.
 A non-nullable sibling does not trigger it.

```text
trigger (find_all takes the longer span, dropping the lookbehind):
  (|(?<=[a-z])b)        on "b"  -> rust 0:1, want 0:0
  (a*|(?<=[a-z])b)      on "b"  -> rust 0:1
  (a?|(?<=[a-z])b)      on "b"  -> rust 0:1
  ((?<=[a-z])b|)        on "b"  -> rust 0:1   (order independent)
  ([\w]{0}|(?<=[\w])[a-c]) on "a" -> rust 0:1 (original Lean-round trigger)
  (b{0,2}|(?<=\w)_{0,1})   on "a" -> rust 0:1

not a trigger:
  (c|(?<=[a-z])b)       on "b"  -> no match   (non-nullable sibling)
  (?<=[a-z])b           on "b"  -> no match   (lookbehind enforced in isolation)
  (|b)                  on "b"  -> 0:1 correct (no lookbehind; longest wins legally)
```

## Root cause

`find_all`'s longest-end search from a start position does not re-apply the
lookbehind that gates the longer branch when a nullable sibling branch has already
made that position a valid match origin.
 The engine finds the longest reachable
end (here offset 1,
 via the `b` literal) without enforcing that the path reaching
it satisfies its lookbehind.
 The lookbehind and lookahead node handling lives
around `resharp-engine/src/engine.rs:128` (`Kind::Lookahead | Kind::Lookbehind`);
the exact longest-end site that skips the gate is not yet pinned.

## Notes

- Same underlying defect as the BUG-3 lookbehind trigger `(?<=\D?[a-c]+0?)b`,
   seen
  from the other side.
   When no nullable sibling exists,
   `is_match` is false while
  `find_all` still emits the lookbehind-forbidden span,
   so it surfaces as an
  INCONSIST (is_match vs find_all).
   When a nullable sibling makes `is_match` true,
  the same dropped-gate defect surfaces here as a wrong span length instead,
   which
  no internal oracle catches.
- Ground-truth-only at the span level,
   like BUG-12 and BUG-13:
   `is_match` is right
  and the two `find_all` spans are mutually consistent,
   so only the verified Lean
  semantics (plus the isolation argument) expose it.
