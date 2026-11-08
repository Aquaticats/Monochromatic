# Code-quality issues in resharp (rust): definitely-rewrite tier

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Issues found while reading the engine,
 algebra,
 and parser source during the fuzz
campaign that any reasonable Rust author would rewrite on sight.
 resharp is a young,
fast-moving crate,
 so an unenforced invariant will break across churn and a
host-aborting guard on a "cannot happen" state will eventually be hit as the surface
grows;
 a reasonable maintainer hardens both now rather than trusting today's layout.
That puts invariant-protected `unsafe`,
 narrowing casts,
 and unproven-reachable
aborts in this tier alongside the proven defects.
 Genuine design tradeoffs (where a
maintainer might reasonably leave the current choice) are in
`code-quality.recommendations.md`.
 Every location is in the pristine clone
`/tmp/agent/resharp-fuzz-20260604`.

## A library aborts the host process on user input

```rust
//! The assertion that reports a correctness issue as a panic,
//! and the match push that follows it.
// resharp-engine/src/engine.rs:960
assert_ne!(NO_MATCH, l_max_end, "correctness issue found");
matches.push(Match { start: 0, end: l_max_end });
```

`find_all` aborts the whole process on a two-byte user input:
 `Regex::new(".\\W*b+")
.find_all(b"ba")` panics here (BUG-2),
 as do 43 to 46 distinct patterns in the
directed sweep's `HARDPANIC_FA` bucket,
 for example `(c|\D+)bx*` on `ba`.
 A library
must not `assert!`/`panic!` on user-supplied data reachable from a public API;
 it
returns its `Error`.
 The `correctness issue found` message shows the authors know
the state is reachable but invalid,
 so the assert documents a known bug instead of
handling it.
 Rewrite:
 return `Err(...)`.

## In-band `usize::MAX` sentinel for "no match" that buys no measurable perf

`NO_MATCH` is `usize::MAX` (`resharp-engine/src/engine.rs:12`),
 carried in the same
`usize` as real match ends,
 which is how the sentinel reaches a `Match` (BUG-4) or
the assertion above (BUG-2).
 The usual defense of an in-band sentinel is the cost of
`Option<usize>` (sixteen bytes against eight,
 no niche on `usize`).
 The evidence
says that cost is not paid here:
 every use of `NO_MATCH` is a scalar local,
 a return
value,
 or a comparison,
 never a field of a `Vec`,
 array,
 or struct stored in bulk.
The sites are `engine.rs:825`,
 `:924`,
 `:985`,
 `:1140`,
 `:1145`,
 `:1217`,
 `:1223`,
`:1226`,
 `:1635`,
 `:1650`,
 `:1804`,
 `:1820`,
 the eight checks in `fwd.rs`,
 the
`lib.rs` scan locals,
 and `stream.rs:249`;
 `Match { start, end }` itself stores only
real ends.
 For a value that lives only in registers,
 `Option<usize>` is free (the
`!= NO_MATCH` checks become `.is_some()`,
 the same discriminant test),
 so the
sentinel saves nothing and in exchange makes "no match" pushable into a `Match` and
trippable into a process abort.
 A perf tradeoff that costs zero and risks a
soundness bug plus a host abort is sour.
 Rewrite:
 make the absent end an
`Option<usize>` (or a two-variant enum) so the leak is a compile error.

## Host-aborting guards on states assumed unreachable

Several `panic!`/`unreachable!`/`.expect` guards abort the process if their "cannot
happen" assumption is ever wrong:

- `get_fixed_length(lb_stripped).expect("AnchoredFwdLb requires fixed-length lb")`
  (`engine/src/lib.rs:1076`):
   aborts if a variable-length lookbehind reaches the
  anchored-forward-lookbehind prefix path.
- `unreachable!("FwdPrefix without AnchoredFwd prefix")` and its sibling
  (`engine/src/lib.rs:1285`,
   `:1291`).
- `panic!` at `algebra/src/lib.rs:2595` and `:2891`;
   `assert!(node_id !=
  NodeId::MISSING)` at `algebra/src/lib.rs:4435`.

I could not reach these from a public API on user input in the time available
(variable-length lookbehinds like `(?<=a+)b` are currently rejected at parse).
 But
`engine.rs:960` proves this exact pattern,
 a guard assumed unreachable,
 is in fact
reachable from `find_all`;
 in a crate whose pattern surface (anchors,
 lookbehind,
intersection,
 complement) is still growing,
 the remaining "unreachable" guards are
the same latent abort waiting on the next feature.
 Rewrite:
 return `Err` (or encode
the invariant in the type,
 for example a fixed-length-lookbehind newtype so the
`.expect` becomes unnecessary),
 rather than aborting the host.

## Unchecked raw-pointer indexing keyed on a lazy state id

```rust
//! The two unchecked reads of the effect tables:
//! the state index and then the effect index,
//! each taken from the previous read without a bounds check.
// resharp-engine/src/engine.rs:1799 (inside `unsafe fn fwd_update`)
let eid = unsafe { *effect_id.add(state as usize) };   // state: u32 lazy DFA id
// ...
let v = unsafe { &*effects.add(eid as usize) };        // eid from that read
```

`state` is a lazily-grown DFA state id,
 the read has no bounds check,
 and
`fwd_update` is on the BUG-2 / BUG-4 path (`engine.rs:950`,
 `:1000`).
 The campaign
fuzzed under AddressSanitizer without tripping it,
 so the invariant "every reachable
state has an `effect_id` slot" holds today,
 but it is maintained by construction
order,
 not by the type system,
 exactly the kind of invariant that breaks when the
lazy DFA growth is next refactored.
 The cost of a check on this path is one
comparison.
 Rewrite:
 index a slice (`effect_id[state as usize]`) so a violated
invariant is a clean panic with a real `Error` boundary,
 not undefined behaviour,
 or
at minimum gate the `unsafe` behind a `debug_assert!` on the bound.

## Narrowing `as u16` / `as u8` casts on ids and lengths

`self.create_state(b, curr as u16)` (`engine.rs:1098`,
 `:1185`,
 `:1249`) narrows a
`u32` to the engine's `u16` state id,
 and `lb_fixed as u8` (`engine/src/lib.rs
:1078`) narrows a lookbehind length to `u8`.
 Today state ids fit in `u16` by design
and lookbehind lengths are small,
 so no truncation is observed;
 but `unbounded_size`
removes the state cap,
 and a silent `as` truncation there yields a different,
valid-looking state id and corrupts the transition with no error.
 A narrowing cast
on a value that another mode is allowed to grow past the target width is a defect
even before it triggers.
 Rewrite:
 `u16::try_from(curr)?` (and `u8::try_from`),
turning the boundary into an `Error` instead of silent wraparound.

## `.ok()` discards a fallible result the next statement depends on

```rust
//! The lazy transition and the state creation after it,
//! which drops the result of the creation.
// resharp-engine/src/engine.rs:1249
curr = self.lazy_transition(b, curr as u16, mt)? as u32;
self.create_state(b, curr as u16).ok();   // result dropped
```

`create_state` allocates `state_nodes[curr]`,
 which the next transition reads;
dropping its `Result` lets the code proceed when the state was not built,
 and the
following read is the out-of-bounds panic in BUG-15 (`engine.rs:550`).
 The same
shape repeats at `engine.rs:1098` and `:1185` (other reverse-scan paths,
 latently
the same crash) and at `algebra/src/lib.rs:2491` and `:2497`
(`mk_lookbehind_internal(...).ok()`,
 where a dropped rewrite failure could later
surface as a wrong match set).
 `.ok()` on an operation whose success the next line
relies on is wrong regardless of any other guard.
 Rewrite:
 propagate with `?`,
 and
have `create_state` ensure its own backing capacity so the precondition is not a
per-call-site convention.

## O(n^2) where the O(n) version sits in the same file

`find_all` routes nullable patterns to `find_all_nullable_slow`
(`resharp-engine/src/lib.rs:1794`),
 which restarts `scan_fwd_slow(pos, input)` from
every position with no state carried forward,
 giving O(n^2) (BUG-18).
 The hardened
config takes `find_all_dfa` (`:1713`),
 a single stateful pass,
 and matches the same
`~(a+)` on 64 KB in 3 ms where the nullable path takes 4.6 s,
 a 1500x gap on
identical input.
 When the linear implementation of the same function is in the same
file and demonstrably correct on the input,
 the per-position rescan is an immediate
rewrite.
 Rewrite:
 carry the forward state across positions,
 or route through the DFA
driver.

## One language, three internal representations, 300x cost spread

`\w`,
 `[\w]`,
 and `[A-Za-z0-9_]` denote the identical class.
 Bare `\w` and explicit
`[A-Za-z0-9_]` compile in microseconds;
 `[\w]` (the shorthand inside a class) takes
1.76 s for `[\w]{3,5}` and 15.3 s for `([\w]{3,5}){3,3}` (BUG-17),
 and the same
un-folded set makes BUG-11 and BUG-19 expensive.
 A class set should normalize to one
canonical minterm predicate at parse time regardless of how it was written;
 the fast
path already existing for two of the three spellings proves the normalization is
simply missing for the third.
 A 300x cost difference between two spellings of one
regex is something a maintainer fixes the day they see it.
 Rewrite:
 fold every
character-class set (including bracketed perl shorthands) to a single predicate
before it can be repeated.
