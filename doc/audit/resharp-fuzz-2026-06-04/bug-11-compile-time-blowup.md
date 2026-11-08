# BUG-11 super-linear compile time on small intersection patterns

## Classification

- Type:
   performance,
   compile-time blowup (denial-of-service relevant).
- Phase:
   compile time,
   inside `Regex::new` / `with_options`.
- Severity:
   a roughly 20 character pattern takes multiple seconds to compile,
  and the cost grows steeply with pattern size,
   so a modestly larger pattern can
  hang the engine.
   The dotnet reference compiles the same patterns instantly.

## Minimal reproducers

```rust
//! Reproduction of the compile-time blowup:
//! two patterns whose construction takes several seconds each.

/// The regex type whose construction is slow.
use resharp::Regex;
// each of these blocks for several seconds inside Regex::new:
let _ = Regex::new(r"[\w]{3,5}[\w]([^a]&a+)");        // about 4 seconds
let _ = Regex::new(r".[\w]{3,5}([^a]{1,3})(\w?|\w?)"); // about 6.5 seconds
```

Command line (empty haystack,
 so only compilation runs):

```sh
time repro --pair "$(printf '%s' '[\w]{3,5}[\w]([^a]&a+)' | xxd -p | tr -d '\n')' ''
```

## Observed behaviour

Compile time measured with an empty haystack (matching short-circuits on empty,
so this is pure `Regex::new`):

- `[\w]{3,5}[\w]([^a]&a+)`:
   about 4.1 seconds.
- `.[\w]{3,5}([^a]{1,3})(\w?|\w?)`:
   about 6.6 seconds.

Adding a haystack does not change the time,
 confirming the cost is in
compilation,
 not matching.
 The dotnet reference returns immediately for both.

## Expected behaviour

Compilation of a small pattern completes in well under a second.
 The paper
advertises input-linear matching;
 that claim is about match time on the
compiled automaton,
 but the compile-time derivative or state construction here
is super-linear in the pattern,
 which is a separate and exploitable cost.

## Root cause

The patterns combine bounded repeats of character classes with intersection
(`&`) and small alternations.
 The derivative or minterm construction appears to
enumerate a state space that grows steeply with the class and repeat structure.
This is the same effect that made a directed compile sweep stall on a handful of
inputs:
 most patterns compile in milliseconds,
 a few take many seconds.

## Notes

- This surfaced as the `RUST_TIMEOUT` class in the dotnet differential (rust
  exceeds a 2 second per pattern watchdog while dotnet is fast) and as the
  hang that stalled the directed oracle sweep.
- Distinct triggers seen include `(([^a]&.{0,2})|[\w]{2,2})`,
  `[\w]{3,5}[\w]([^a]&a+)`,
   `(?-i:(?m:([^\w]{3,5})))`,
   and
  `.[\w]{3,5}([^a]{1,3})(\w?|\w?)`.
   The common factor is bounded class repeats
  combined with intersection or nested optional groups.
