# BUG-9 the stream path drops matches that is_match and find_all report

## Classification

- Type:
   correctness,
   the `stream` matcher under-reports.
- Phase:
   match time,
   the `Regex::stream` / `stream_with` path (leftmost-shortest,
  a separate scan from `find_all`).
- Severity:
   soundness for any caller using the stream API.

## Minimal reproducers

```rust
//! Reproduction of the dropped stream match:
//! `is_match` and `find_all` find the empty match,
//! `stream` finds none.

/// The regex type whose `stream` entry point drops the match.
use resharp::Regex;
let re = Regex::new(r"\A\z?").unwrap();
assert!(re.is_match(b"a").unwrap());          // true
assert_eq!(re.find_all(b"a").unwrap().len(), 1); // one match (0,0)
assert_eq!(re.stream(b"a").unwrap().len(), 0);   // BUG: stream returns nothing
```

Command line,
 with the violated invariant printed as `STREAMINCONSIST`:

```sh
repro '\A\z?'  --sweep | grep STREAM
repro '(^|b)'  --sweep | grep STREAM   # is_match true on "a", stream empty
repro '(?<!b)' --sweep | grep STREAM   # is_match true on "b", stream empty
repro '\Bb'    --sweep | grep STREAM
repro '^\D*'   --sweep | grep STREAM
```

## Observed behaviour

```text
STREAMINCONSIST|default|im=true|streamlen=0|hay=61|pat="(^|b)"
STREAMINCONSIST|default|im=true|streamlen=0|hay=62|pat="(?<!b)"
```

For `(?<!b)` on `b`,
 the dotnet reference confirms the match exists:
`im=1 fa=0:0 le=0`.
 resharp `is_match` and `find_all` agree that a match exists,
but `stream` returns an empty list.

## Expected behaviour

`stream(h)` must be non-empty whenever `is_match(h)` is true.
 The stream path is
just a different traversal of the same language.

## Root cause

`resharp-engine/src/stream.rs`,
 the `stream_with_inner` path.
 The triggers are
all zero-width or anchor matches at a boundary:
 a leading `^` or `\A`,
 a
trailing `\z`,
 a non-word-boundary `\B`,
 or a zero-width negative lookbehind.
The stream traversal advances past or never emits the zero-width match that the
forward and reverse passes used by `find_all` do emit.
 This is the stream
analogue of the already-tracked hardened find_all zero-width drop.

## Distinct triggers

707 distinct patterns hit `STREAMINCONSIST` in the 159k-pattern sweep.
 The
shortest forms cluster on anchors and boundaries:
 `\Bb`,
 `\B\W`,
 `^\D*`,
`(^|b)`,
 `(b|^)`,
 `(^|c)`,
 `(?<!b)`,
 `(\b|^)`,
 `(^&b?)`,
 `\A\z?`.
