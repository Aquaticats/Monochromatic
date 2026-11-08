# BUG-25 a panic inside the lock poisons the inner Mutex and permanently bricks the Regex

## Classification

- Type:
   robustness,
   soundness amplifier.
   Any internal panic during a query poisons
  the shared `inner` Mutex,
   after which every method on that `Regex` panics with a
  `PoisonError`.
   A single transient failure becomes a permanent,
   total denial of
  service for the instance.
- Phase:
   match time,
   the lock acquisition in every public query method.
- Severity:
   high.
   `Regex` is built to be compiled once and reused (and is `Send`/
  `Sync`-shaped behind a `Mutex`,
   so shared across threads).
   One bad input that trips
  any internal panic (an assertion,
   a capacity abort,
   a contamination-induced
  inconsistency) bricks the instance for all later calls and all threads holding it,
  even if the caller dutifully catches the first panic.

## Minimal reproducer

```rust
//! Reproduction of the poisoned lock:
//! a panic that the caller catches leaves the cache lock poisoned,
//! so every later call on the same `Regex` fails.

/// The regex type whose lock is poisoned.
use resharp::Regex;
/// The panic helpers that catch the first panic and show that the second call fails too.
use std::panic;

let re = Regex::new(r"\w+b").unwrap();
let _ = re.find_all(b"ab");                                   // ok, primes the cache
let first = panic::catch_unwind(panic::AssertUnwindSafe(|| re.find_all(b"ba")));
assert!(first.is_err());                                      // panics at engine.rs:960

// the caller caught the panic, but the Regex is now permanently dead:
let bricked = panic::catch_unwind(panic::AssertUnwindSafe(|| re.is_match(b"z")));
assert!(bricked.is_err());                                    // PoisonError at lib.rs:1723
```

A two-element `find_all` sequence `["ab", "ba"]` is enough.
 After it,
 all four query
methods are bricked.

## Observed behaviour

```text
find_all(ba) panicked: true
  is_match      bricked=true
  find_all      bricked=true
  find_anchored bricked=true
  stream        bricked=true
```

The first panic is `engine.rs:960` (`assertion left != right failed: correctness
issue found`,
 the BUG-2 site,
 here reached through the BUG-21 cache contamination
primed by `find_all("ab")`).
 Every subsequent call panics with
`lib.rs:1723: PoisonError`.
 The wider hardened-vs-default sweep (`repro --hardbatch`)
shows the same shape across `\w+b`,
 `\D+b`,
 `[^a]+b`,
 `(\w+)b`:
 138 `HARDPANIC_IM`
and 144 `HARDPANIC_FA` rows where the default config panics (and then cascades into
`PoisonError` on every later haystack in the reused sequence) while hardened does not.

## Expected behaviour

A panic on one input must not destroy the `Regex`.
 After catching it,
 a later query
of an unrelated input must run normally.
 Equivalently,
 a transient internal failure
should be isolated to the call that caused it,
 not promoted to a permanent brick of
the shared instance.

## Root cause

`inner` is a `std::sync::Mutex<RegexInner>` (`resharp-engine/src/lib.rs:89`,
`:280`,
 `:1125`).
 Std mutexes poison on panic:
 if a thread panics while holding the
lock,
 every later `lock()` returns `Err(PoisonError)`.
 Every public query method
acquires the lock with `.lock().unwrap()`,
 which turns that `Err` into a panic.
 There
are 16 such `self.inner.lock().unwrap()` sites in `lib.rs` (`find_all_dfa_inner` at
`:1723`,
 plus `is_match`,
 `find_anchored`,
 `stream`,
 and the prefix/bounded paths).

The query work,
 including the scan that hits the `engine.rs:960` assertion,
 runs
inside the locked scope (`find_all_dfa_inner` locks at `lib.rs:1723` and then calls
the scan).
 So when the assertion fires,
 the panic unwinds through the held lock and
poisons it.
 From then on,
 every `.lock().unwrap()` on that instance re-panics,
 so all
methods are dead.

This is orthogonal to whichever bug causes the first panic:
 fixing BUG-2 and BUG-21
removes this particular trigger,
 but any other panic inside the lock (a capacity
abort,
 a future defect) would still brick the instance the same way.
 The amplifier is
the `std::sync::Mutex` + `.lock().unwrap()` pattern.
 Fixes:
 recover from poison at the
lock sites (`.lock().unwrap_or_else(|e| e.into_inner())`),
 switch to a non-poisoning
mutex (`parking_lot::Mutex`),
 or run the fallible scan so a panic cannot escape with
the lock held (catch and restore consistent state before unwinding).
 The first is the
smallest change and makes a caught panic survivable.

## Affected configurations

The locking pattern is config-independent.
 The reproducer above panics in the default
config;
 hardened avoids the specific `engine.rs:960` trigger but shares the same
poison-amplification design,
 so any panic it does hit would brick it identically.

## Relationship to other findings

- The first panic here is the BUG-2 assertion (`engine.rs:960`),
   reached through
  BUG-21 contamination (the `find_all("ab")` prime corrupts the shared cache so
  `find_all("ba")` trips the assertion;
   a fresh `\w+b` on `"ba"` does not panic).
  BUG-25 is the distinct,
   orthogonal amplifier that converts that one panic into a
  permanent brick of the whole instance.
- Strengthens the `code-quality.md` note about library aborts on user input:
   the abort
  is not merely a crash of one call;
   via mutex poisoning it disables the compiled
  `Regex` for every subsequent call and every thread sharing it.

## Code quality

`.lock().unwrap()` at 16 sites encodes "treat lock poisoning as unreachable,
" which is
false the moment any code under the lock can panic,
 and this crate has several such
panics (BUG-2,
 BUG-15,
 capacity aborts).
 For a long-lived,
 shared,
 compiled object the
poison-tolerant lock (or a non-poisoning mutex) is the shape a reasonable author picks
once they accept that the guarded work can fail.
