# BUG-4 find_all emits a match with end = usize::MAX

## Classification

- Type:
   correctness,
   out-of-bounds match result.
- Phase:
   match time.
- Severity:
   high.
   `find_all` returns a `Match` whose `end` is `usize::MAX`.
   Any
  caller that slices `haystack[m.start..m.end]` panics or reads out of bounds.
  This is the silent-leak sibling of BUG-2 (same sentinel,
   no assertion).

## Minimal reproducer

```rust
//! Reproduction of the no-match sentinel leak:
//! `find_all` returns a match whose end is the sentinel value.

/// The regex type and the option set that its construction takes.
use resharp::{Regex, RegexOptions};
let opts = RegexOptions::default()
    .case_insensitive(true).ignore_whitespace(true)
    .dot_matches_new_line(true).multiline(false);
let re = Regex::with_options(r"~(_*$)", opts).unwrap();
let matches = re.find_all(b"ab").unwrap();
// matches contains Match { start: 1, end: 18446744073709551615 }
```

Command line:

```sh
repro '~(_*$)' --sweep | grep BOUNDS
```

A stronger trigger of the same defect:

```sh
repro '~(_*\z)' --sweep | grep BOUNDS
```

## Observed behaviour

```text
BOUNDS|flags|m=1..18446744073709551615|len=2|hay=6162|pat="~(_*$)"
```

The match end is `18446744073709551615`,
 which is `usize::MAX`,
 which is the
engine's `NO_MATCH` sentinel.
 It appears for `~(_*$)` in the flags mode on
2-byte and 3-byte haystacks;
 `~(_*\z)` triggers it far more often.

## Expected behaviour

Every returned `Match` satisfies `start <= end <= haystack.len()`.

## Root cause

`resharp-engine/src/engine.rs`,
 the reverse-collect plus forward-verify
`find_all` path.
 Two push sites emit `Match { end: l_max_end }` where
`l_max_end` can still be `NO_MATCH`:

```rust
//! The two places in the engine that push a match without checking that the end is not the sentinel.
// engine.rs:1009
matches.push(Match { start: nulls[i], end: l_max_end });
// engine.rs:1022
matches.push(Match { start: nulls[i], end: l_max_end });
```

`NO_MATCH` is `usize::MAX` (`engine.rs:12`).
 The forward scan from a candidate
start found no valid end (it stayed at the sentinel),
 but a `Match` is pushed
anyway.
 The parallel path at `engine.rs:960` guards the same condition with an
assertion (BUG-2);
 these two push sites do not,
 so the sentinel escapes into the
public result.
 The pattern shape is the complement of "any string ending at end
of input",
 `~(_*$)` and `~(_*\z)`,
 which makes the forward language empty at the
chosen start while the reverse pass still proposed that start.

## Distinct triggers

The `usize::MAX` end leaks from several prefix shapes,
 not just the
end-anchor-complement one:

- end-anchor complement,
   default-off-multiline:
   `~(_*$)`,
   `~(_*\z)` (flags mode).
- non-word-boundary prefix:
   `\Bb+` on `ba` returns `1..usize::MAX` (default mode).
- lookbehind prefix:
   `(?<=[^a])b+` on `ba` returns `1..usize::MAX` (default mode).
- intersection:
   `\b\W{0}(b&\S{0,2})(c|1{0})`.

The `\B` and lookbehind triggers fire in the default option mode,
 so this is not
flags-only.
 dotnet rejects `\Bb+` (fail closed) and returns a normal result for
`(?<=[^a])b+`;
 rust accepts both and leaks the sentinel.

## Notes

- Fixing BUG-2 and BUG-4 together likely means making the reverse-proposed start
  and the forward-verified end agree,
   or dropping the candidate when the forward
  end is `NO_MATCH`.
