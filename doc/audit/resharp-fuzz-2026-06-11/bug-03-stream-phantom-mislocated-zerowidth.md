# BUG-3 stream reports phantom zero-width matches at wrong positions

- Type:
   correctness,
   soundness.
   `Regex::stream` reports zero-width matches at
  positions where the pattern does not match,
   and misses the positions where it
  does.
- Phase:
   match time,
   the `stream` leftmost-shortest scan (`stream.rs`),
   a
  separate path from `find_all`.
- Severity:
   soundness.
   For a purely zero-width pattern,
   shortest and longest
  match lengths coincide,
   so `stream` and `find_all` must report the same set of
  match positions;
   they do not.
- Affected:
   default config (and others);
   shown on default below.
- Discovery:
   the STREAMPHANTOM oracle (`find_all` empty but `stream` non-empty)
  plus the 06-04 re-verification of stream correctness.

## Reproducer

```rust
//! Reproduction of the mislocated zero-width matches of `stream`:
//! three patterns whose spans differ from `find_all`,
//! with both results in the comments.

/// The regex type whose `stream` mislocates matches.
use resharp::Regex;

// lookahead-of-char: matches the empty string at offset 0 where 'c' follows.
let re = Regex::new(r"(?=c)").unwrap();
assert_eq!(re.find_all(b"c").unwrap().len(), 1);   // [0:0]  correct
assert_eq!(re.stream(b"c").unwrap(), re.find_all(b"c").unwrap()); // FAILS: stream = [1:1]

// word boundary: boundaries of "ab" are at 0 (start) and 2 (end), not 1.
let re = Regex::new(r"\b").unwrap();
// find_all("ab") = [0:0, 2:2]; stream("ab") = [1:1, 2:2]  (1 is NOT a boundary)

// negative lookahead of begin-anchor: matches everywhere except offset 0.
let re = Regex::new(r"(?!\A)").unwrap();
// find_all("ab") = [1:1, 2:2]; stream("ab") = [0:0, 2:2]  (0 is forbidden by \A)
```

Harness:

```sh
repro --show 283f3d6329 63 0   # (?=c) on "c": stream=1:1, find_all=0:0
repro --show 5c62 6162 0       # \b on "ab": stream=1:1,2:2, find_all=0:0,2:2
repro --show 283f215c4129 6162 0  # (?!\A) on "ab": stream=0:0,2:2, find_all=1:1,2:2
```

## Observed versus expected

- `(?=c)` on `"c"`:
   matches the empty string at offset 0 (where `c` follows).
  `find_all` and `find_anchored` agree on `0:0`.
   `stream` returns `1:1`,
   a
  position where `(?=c)` is false (nothing follows index 1).
   Phantom match,
   and
  the real `0:0` match is missed.
- `\b` on `"ab"`:
   real boundaries are at 0 (start before a word char) and 2 (end
  after a word char);
   position 1 (between two word chars) is not a boundary.
  `find_all` returns `[0:0, 2:2]`.
   `stream` returns `[1:1, 2:2]`:
   it reports the
  non-boundary 1 and misses the start boundary 0.
- `(?!\A)` on `"ab"`:
   `\A` is true only at offset 0,
   so the pattern matches at
  1 and 2,
   not 0.
   `find_all` returns `[1:1, 2:2]`.
   `stream` returns `[0:0, 2:2]`:
  it reports the forbidden 0 and misses 1.

In each case `stream` advances past the relevant byte before recognising the
zero-width assertion,
 mis-locating the match by one position at the leading edge.

## Related manifestation: stream returns empty while a match exists

The same stream defect also drops every match (not just mislocates) on
assertion-heavy patterns.
 `((?<=b+){2}&(\n{2,}\w{1,3}){0}^{0})` on `"b"` (and
`"bb"`,
 `"ba"`,
 ...) has `is_match = true` and `find_all = [1:1]` (and friends)
but `stream = []` (the STREAMINCONSIST oracle,
 3 distinct patterns of this shape
in the 40k corpus).
 These patterns also trip bug-02 (`find_anchored = Some(0:0)`
while `is_match = true` here but the anchored span is wrong).
 This is the 06-04
BUG-9 (stream drops matches) shape,
 verified fixed for `\A\z?` but live again on
the lookbehind-intersection family;
 it is the empty-result face of the same
stream zero-width / assertion mishandling,
 not a separate root cause.

## Relationship to other findings

Distinct path from arm-bug-01 (the SIMD `find_all` driver) and bug-07 (the
hardened `find_all`);
 all three mishandle consecutive/leading zero-width matches
but in separate drivers.
 Related to 06-04 BUG-9 ("stream drops matches"),
 whose
write-up explicitly listed "whether stream's shortest spans are themselves always
correct" as untested.
 This is the realisation of that gap:
 the spans are not
correct for leading zero-width assertions.
