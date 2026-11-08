# BUG-5 find_all hits a reachable "this path should be eliminated" assertion

- Type:
   crash (debug-assertions builds) / reachable dead branch (release).
- Phase:
   match time,
   `find_all` for a `rev_trivial` pattern.
- Severity:
   medium-high.
   A `debug_assert!(false, ...)` the developer believed
  unreachable is reachable from user input,
   so any debug-assertions or test or
  fuzz build panics;
   in release the branch is compiled out and execution falls
  through a path the author intended to remove,
   with no guarantee the following
  code's invariants hold.
- Affected:
   non-hardened configs (the assertion is gated on `!self.hardened`).
- Architecture:
   confirmed byte-identical on aarch64 (Apple M1) and x86-64.
  `armprobe "_*(?!_)" "aa"` on the M1 panics at the same
  `resharp-engine/src/lib.rs:1824` site (the dotnet reference accepts `_*(?!_)`
  and returns `[(0,2),(2,2)]`,
   so this in-subset trigger reaches the dead branch;
  the site is arch-independent,
   the ARM run only makes "ARM-confirmed" airtight).
- Discovery:
   the `match_invariants` libFuzzer target.

## Reproducer

```rust
//! Reproduction of the reachable debug assertion:
//! a pattern with a trivial reverse automaton hits an assertion that claims the path is eliminated.

/// The regex type whose `find_all` panics in a debug-assertions build.
use resharp::Regex;
let re = Regex::new(r"_*$").unwrap();
let _ = re.find_all(b"\n\xfe*\xfe_*"); // panics in a debug-assertions build
// thread panicked at resharp-engine/src/lib.rs:1824:
//   found bug: this path should be eliminated
```

The fuzzer's exact unit was pattern `_*$`,
 haystack hex `0afe2afe5f2a`,
 default
config.
 `_*$` is `rev_trivial` (its reverse is `_*`),
 and the `find_all` path for
`rev_trivial && !hardened` reaches:

```rust
//! The debug assertion that states the trivial reverse path is eliminated,
//! which the reproduction reaches.
// resharp-engine/src/lib.rs:1824
if self.rev_trivial && !self.hardened {
    debug_assert!(false, "found bug: this path should be eliminated");
}
```

## Observed versus expected

Expected:
 `find_all` returns the matches with no panic.
 The engine instead hits a
guard the developer left as a should-never-happen marker,
 proving that the
`rev_trivial` non-hardened `find_all` path the author meant to eliminate is in
fact live.
 In a debug-assertions build (which includes the cargo-fuzz profile and
typical test profiles) this is a hard panic;
 in release the `debug_assert!` is a
no-op and control falls through to `collect_rev`,
 so the result is whatever that
path happens to compute,
 untested against this invariant.

## Source pointer

`resharp-engine/src/lib.rs:1824`.
 The branch sits just before the
`initial_nullability.has(END)` / `collect_rev` block in `find_all`.
 The fix is
either to handle the `rev_trivial` case explicitly (it should be cheap:
 the
reverse matches everything) or to prove and remove the branch;
 the assertion text
shows the author intended the latter but the elimination is incomplete.

## Relationship

Independent of the other findings (a distinct `find_all` branch).
 Shares the
"reverse pass / nullability" subsystem with the 06-04 BUG-4 sentinel family but is
a different defect.
