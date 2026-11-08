# BUG-2 "correctness issue found" assertion at engine.rs:960

## Classification

- Type:
   internal correctness assertion fires during matching.
- Phase:
   match time,
   inside the bounded or reverse-collect `find_all` path.
- Severity:
   panic (abort under the fuzzer).
   The assertion exists precisely
  because the engine knows this state should be unreachable.
   See BUG-4 for the
  sibling path that hits the same bad state without an assertion and leaks it.

## Minimal reproducer

```rust
//! Reproduction of the correctness assertion:
//! `find_all` panics in the default option mode on a two byte input.

/// The regex type whose `find_all` panics.
use resharp::Regex;
// panics in the default option mode on a two byte input:
let re = Regex::new(r".\W*b+").unwrap();
let _ = re.find_all(b"ba"); // panics at engine.rs:960
```

Command line:

```sh
repro '.\W*b+' 6261 --hex
repro '\S+b'   62275f --hex   # also panics, in default/full/js/flags modes
```

`.\W*b+` on `ba` is the cleanest trigger:
 default option mode,
 a two byte ascii
input.
 The dotnet reference returns `no match` for it.

A second independent trigger:

```sh
repro '(\d|_)b(?:a)*' --sweep   # panics in mode=full on haystack "ba"
```

## Observed behaviour

```text
thread panicked at engine.rs:960:
assertion `left != right` failed: correctness issue found
  left: 18446744073709551615   (== usize::MAX == NO_MATCH)
  right: <the same>
```

For `\S+b` on `b'_` the assertion fires in the default,
 full,
 js,
 and flags
modes.
 In the hardened mode the same pattern and haystack instead produce an
`is_match` versus `find_all` disagreement (see BUG-3 class),
 and in ascii mode it
produces a real cross-engine `DIVERGE` (`is_match` true in resharp,
 false in the
`regex` crate;
 see notes).

## Expected behaviour

`find_all` returns the correct match set without aborting.

## Root cause

`resharp-engine/src/engine.rs:960`:

```rust
//! The assertion that fires,
//! and the match push that follows it,
//! in the matcher of the failing case.
assert_ne!(NO_MATCH, l_max_end, "correctness issue found");
matches.push(Match { start: 0, end: l_max_end });
```

The reverse pass identified a candidate match start,
 but the forward scan from
that start returns `NO_MATCH` (`usize::MAX`,
 defined at `engine.rs:12`) for the
end.
 The forward and reverse views of the language disagree,
 so the sentinel
reaches the point where a `Match` end is required.
 The assertion catches it
here;
 the path in BUG-4 does not.

## Code quality

`NO_MATCH` is an in-band sentinel,
 `usize::MAX` (`engine.rs:12`),
 threaded through
the same `usize` channel that carries real match ends.
 That is the root code smell:
"no end" and "an end of 18446744073709551615" are indistinguishable by type,
 so the
sentinel can be pushed into a `Match { end }` (BUG-4) or trip an assertion (here)
instead of being unrepresentable.
 A `usize` for a value that is sometimes absent
should be an `Option<usize>` or a dedicated two-variant enum,
 which would make
"sentinel reaches a Match" a compile error rather than a runtime abort.

The guard chosen is also wrong for a library:
 `assert_ne!` aborts the process.
 A
regex engine that hits an internal inconsistency on user input should surface an
`Error`,
 not abort the host.
 The `correctness issue found` message confirms the
authors know the state is reachable-but-invalid;
 an assert documents the bug
instead of handling it.
 The hardened path does not abort on the same input (it
returns a wrong-but-non-crashing answer,
 BUG-3 class),
 so the two paths disagree on
whether this state is fatal,
 which says neither has a principled handling of it.

## Distinct triggers and the hardened divergence

The default,
 full,
 js,
 flags,
 and unbounded configs all abort at `:960` on this
pattern family;
 the directed sweep's `HARDPANIC_FA` oracle (default panics,
hardened does not) is this same assert:
 43 to 46 distinct patterns where the
default `find_all`/`is_match` aborts at `:960` while the hardened scan does not,
for example `(c|\D+)bx*` on `ba`.
 So BUG-2 is not a stream-only or rare crash;
 it
is reachable from `find_all` on short ascii inputs across every non-hardened
config.

## Notes

- The `\S+b` trigger is also a genuine ascii-mode soundness bug on its own:
   on
  the pure-ascii haystack `b'_`,
   resharp reports `is_match = true` while the
  `regex` crate with `.unicode(false)` reports false.
   `\S+b` requires a non-space
  run ending in a literal `b`,
   and `b'_` has its only `b` at offset 0 with
  nothing before it,
   so there is no match.
   This overlaps BUG-7 (the `\S` family).
- Distinct triggers:
   21 panic hits at this site in the first 80k sweep,
   all at
  `engine.rs:960`.
