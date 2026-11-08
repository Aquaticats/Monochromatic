# BUG-3 is_match returns false while find_all returns a match

## Classification

- Type:
   correctness,
   two public entry points disagree on the same input.
- Phase:
   match time.
- Severity:
   soundness.
   A caller using `is_match` as a gate before `find_all`
  (or vice versa) gets contradictory answers.

## Minimal reproducer

```rust
//! Reproduction of the disagreement:
//! `is_match` says there is no match while `find_all` returns one.

/// The regex type whose two entry points disagree.
use resharp::Regex;
let re = Regex::new(r"(\z|(?=a)\w)").unwrap();
let hay = b"0";
assert_eq!(re.is_match(hay).unwrap(), false);          // resharp says no match
assert_eq!(re.find_all(hay).unwrap().len(), 1);        // but find_all returns one
```

Command line:

```sh
repro '(\z|(?=a)\w)' --sweep | grep INCONSIST
```

Second independent trigger (found by the `match_invariants` target),
 opposite
direction is also possible;
 this one is `is_match` true,
 `find_all` empty:

```sh
repro '\BU' 5569695c --hex   # haystack "Uii\" ; INCONSIST in default, full, hardened
```

## Observed behaviour

For `(\z|(?=a)\w)`:
 `is_match` is false but `find_all` returns one match.
 It
reproduces in the default,
 full,
 flags,
 and hardened modes (not ascii),
 on many
haystacks including `0`,
 a single space,
 a tab,
 a newline,
 and `HELLO`.

For `\BU` on `Uii\`:
 `is_match` is true but `find_all` returns zero matches,
 in
the default,
 full,
 and hardened modes.

## Expected behaviour

`is_match(h)` must equal `!find_all(h).is_empty()` for every pattern and
haystack.
 This is one of the invariants the in-tree `match_invariants` fuzz
target asserts.

## Root cause

Not fully traced.
 `is_match` (`resharp-engine/src/lib.rs:1861`) has several
fast-path early returns keyed on `fwd_begin_anchored`,
 `rev_end_anchored`,
`has_bounded`,
 and the selected prefix,
 while `find_all` dispatches through a
different set of paths (`compute_find_all`).
 Both triggers involve a zero-width
or anchor-adjacent branch in an alternation (`\z` end anchor with a lookahead,
or `\B` non-word-boundary),
 which is exactly where the two entry points choose
different specialised scans.
 The minimal forms point at the anchor-or-lookahead
branch of the `is_match` fast path returning the wrong answer.

## Distinct triggers

- `(\z|(?=a)\w)`:
   is_match false,
   find_all one match.
- `((?=0)\S|\z)` on `a`:
   is_match false,
   find_all one match.
   The dotnet reference
  confirms a match exists (`im=1 fa=1:1`),
   so is_match is the wrong side.
- `\BU` on `Uii\`:
   is_match true,
   find_all empty (opposite direction).
- `\z\A(?:a){0,1}` on the empty string:
   is_match false,
   but both `\z` and `\A`
  hold at offset 0 and the optional group matches empty,
   so the empty match
  exists.
   Both the regex crate (ascii) and dotnet report a match;
   rust is_match
  is wrong.
   This is the anchor-concatenation ordering variant.
- `\z\A.*` on the empty string:
   is_match false and find_all empty,
   but both `\z`
  (end of input) and `\A` (start of input) hold at offset 0,
   so the pattern
  matches the empty string there.
   The `regex` crate confirms it
  (`DIVERGE|ascii|rs=false|rx=true|hay=`),
   independent of the Lean reference and
  dotnet.
   `\A\z` (start then end) is correct;
   only the reversed `\z\A` order is
  wrong,
   so this is an anchor-ordering defect in the same family as
  `\z\A(?:a){0,1}`.
   Surfaced by the anchor Lean round (this pattern also crashes
  the stream path,
   BUG-15,
   on non-empty input).
- `(?<=\D?[a-c]+0?)b` on `ba`:
   is_match false,
   find_all returns one match `1:2`.
  Here is_match is the correct side:
   the only `b` is at offset 0 and its
  lookbehind `\D?[a-c]+0?` requires at least one `[a-c]` before offset 0,
   which
  is impossible,
   so there is no match.
   dotnet confirms (`im=0`,
   find_all empty),
  and the Lean ground truth agrees (none).
   find_all is the wrong side,
   emitting a
  span over `a` (offset 1) that no `b` can occupy.
   This is a lookbehind-driven
  trigger (the others are anchor or lookahead driven) and the over-producing side
  is find_all,
   not is_match.
   Found by the Lean leftmost-longest position round.

## Notes

- `(\z|a)`,
   `(\z|\w)`,
   and `((?i:\z)|\w)` do not trigger;
   the lookahead plus the
  perl class is required:
   `(\z|(?=a)\w)`.
- Both directions (false-but-present and true-but-absent) and the distinct
  minimal shapes (lookahead-plus-class,
   end-then-start anchor) suggest more than
  one underlying is_match fast-path defect.
