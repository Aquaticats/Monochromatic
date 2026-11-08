# BUG-15 stream() DFA construction panics on a broad pattern class

## Classification

- Type:
   panic,
   index out of bounds,
   crash.
- Phase:
   match time,
   lazy DFA state construction on the streaming match path.
- Severity:
   crash,
   and wide.
   The full 159257-pattern directed corpus streamed
  through every config produced 28688 distinct patterns that panic at this one
  site (the 12000-pattern subset gave 2396;
   the proportion holds at roughly a
  fifth).
   It fires in every option configuration.
   Any pattern using resharp's
  extended operators (intersection,
   complement,
   lookarounds,
   anchors) can hit it
  once a long enough input is streamed.
- A panic needs no oracle.
   Found while streaming the anchor and lean2 corpora
  through the `stream` API in the panic hunt.

## Minimal reproducer

```rust
//! Reproduction of the stream construction panic:
//! the intersection of two different letters is the empty language,
//! and building the stream automaton for it indexes out of bounds.

/// The regex type whose `stream` entry point panics.
use resharp::Regex;
let re = Regex::new("a&b").unwrap();      // intersection of {"a"} and {"b"}: the empty language
let _ = re.stream(b"aaa");
// panics: engine.rs:550 index out of bounds: the len is 2 but the index is 2
```

A fresh `Regex` and a single `stream` call are enough.
 The input must be at least
three bytes:
 `stream(b"aa")` is fine,
 `stream(b"aaa")` panics.
 The crash is
specific to `stream`;
 `is_match`,
 `find_all`,
 and `find_anchored` on the same
pattern and input do not reach it.

Command line (deterministic,
 all seven configs):

```sh
repro 'a&b' --sweep
# PANIC|engine.rs:550 index out of bounds: the len is 2 but the index is 2|mode=default|hay=616161|pat="a&b"
# (same for ascii, full, js, hardened, flags, unbounded)

repro --stream1 "$(printf '%s' 'a&b' | xxd -p)" "$(printf '%s' aaa | xxd -p)"
# PANIC engine.rs:550 index out of bounds: the len is 2 but the index is 2
```

## Scope: which patterns trigger it

A panic hunt streamed every distinct pattern in the lean2 and anchor corpora
(12000 patterns) across all seven configs.
 2396 distinct patterns panic at
`engine.rs:550`,
 by trigger family:

- Intersection `&`:
   1688 patterns.
   Minimal `a&b`,
   `( &c)`,
   `(a*&b)`.
   Empty or
  near-empty intersections trigger it;
   a trivial non-empty intersection like
  `(.&a)`,
   `(a&a)`,
   or `(\d&\w)` does not,
   so it depends on the minterm or state
  structure the intersection produces.
- Reversed and combined anchors:
   `\z\A`,
   `\z\A.*`,
   and many `$`/`\A`/`\b`
  combinations.
- Lookarounds with no intersection or anchor:
   413 patterns,
   for example
  `((?<! )\D)`,
   `((?![\w])1)`,
   `((?!a) )+`.

Plain regular patterns (literals,
 classes,
 alternation,
 star) do not trigger it;
it is confined to the extended-operator surface that is resharp's whole point.

## Observed behaviour

`create_state` (`resharp-engine/src/engine.rs:545`) reads
`self.state_nodes[state_id as usize]` at line 550 with `state_id == 2` when
`state_nodes` has length 2 (valid indices 0 and 1),
 so the access is out of
bounds and the engine panics.
 The streaming match path drives a DFA transition
into a state id that was never allocated in `state_nodes`.
 Two consumed bytes
build two states;
 the third byte's transition requests the unallocated third
state.

## Root cause localization

The crash is specific to `stream` because the streaming path finds each match
start with an incremental reverse scan that the block matchers do not run.
`try_emit_step` (`resharp-engine/src/stream.rs:247`) calls `scan_rev_from`,
 whose
reverse-transition loop is:

```rust
//! The transition step of the forward scan:
//! it looks the next state up in the center table and builds the missing state lazily,
//! which is the line the panic is reported at.
// resharp-engine/src/engine.rs:1245
let delta = (curr << self.mt_log | mt) as usize;
let next = self.center_table[delta];
if next == DFA_MISSING {
    curr = self.lazy_transition(b, curr as u16, mt)? as u32;
    self.create_state(b, curr as u16).ok();   // engine.rs:1249
} else {
    curr = next as u32;
}
```

Every other path pairs `ensure_capacity(sid)` immediately before
`create_state(b, sid)`:
 the forward `lazy_transition_slow` (`engine.rs:414` then
`:415`) and the block matchers (`engine.rs:415` to `:416` and `:441` to `:442`).
This reverse-scan site calls `create_state(b, curr)` with no preceding
`ensure_capacity(curr)`,
 so when `curr` is a state whose `state_nodes` slot was
not grown,
 the read of `self.state_nodes[curr]` at `engine.rs:550` is out of
bounds.
 `is_match` needs no match start and `find_all` uses a different reverse
pass,
 which is why only `stream` reaches this site.

The fix is to call `self.ensure_capacity(curr)` before the `create_state` at
`engine.rs:1249` (and to audit the other `.ok()` reverse-scan `create_state`
calls at `engine.rs:1098` and `:1185` for the same missing pairing).

## Code quality

The invariant "call `ensure_capacity(sid)` immediately before `create_state(b,
sid)`" is enforced by hand at every call site (the forward `lazy_transition_slow`
and both block matchers all pair them) and simply forgotten at the reverse-scan
site `engine.rs:1249`.
 An invariant maintained by copy-paste discipline across
several call sites will eventually be missed at one of them,
 which is exactly what
happened.
 `create_state` should grow `state_nodes` itself (or a single helper
should do both),
 so the bounds safety is not a convention a future edit can drop.
Compounding it,
 the reverse-scan call is `self.create_state(b, curr as u16).ok()`,
discarding the `Result`,
 so a failure here is silent rather than surfaced.
 A
fallible state allocation whose error is `.ok()`-swallowed and whose precondition is
unchecked is the shape of defect that turns into an out-of-bounds panic on input.

## Expected behaviour

No panic.
 The streaming DFA builder must allocate the state it transitions into,
or never request a `state_id` beyond `state_nodes`.

## Related correctness defect: reversed anchors `\z\A`

The pattern that first surfaced this,
 `\z\A.*`,
 also exposes a separate
correctness bug in the end-anchor-then-start-anchor order,
 documented as a BUG-3
trigger.
 On the empty string both `\z` and `\A` hold at offset 0,
 so `\z\A.*`
matches there,
 but resharp reports no match (`im=0`,
 empty `find_all`).
 The
`regex` crate confirms the match exists (`DIVERGE|ascii|rs=false|rx=true|hay=`),
independent of the Lean reference.
 `\A\z` (start then end) is handled correctly;
only the reversed order is wrong.
 That correctness bug and this stream crash are
distinct defects that happen to coincide on `\z\A.*`.

## Affected configurations

Every configuration the oracle sweeps,
 all seven:
 `default`,
 `unicode(Ascii)`,
`unicode(Full)`,
 `unicode(Javascript)`,
 `hardened(true)`,
`dot_matches_new_line(true)` with `multiline(false)`,
 and `unbounded_size(true)`.

## Notes

- The panic site `create_state` at `engine.rs:550` is distinct from the BUG-1
  re-entrancy panic (union and intersection rewrites) and the BUG-2 assert
  (`engine.rs:960`).
   The full-corpus panic hunt (159257 patterns x 7 configs)
  confirmed only these two crash sites in the entire corpus:
   `engine.rs:550`
  (28688 distinct patterns,
   165515 panic lines) and `engine.rs:960` (BUG-2,
   137
  panic lines).
   No third crash site exists.
- The `stream` path having a separate,
   more fragile DFA construction than the
  block matchers is the core issue.
   Because `stream` is the API for incremental
  and large-input matching,
   this is dangerous:
   a compiled pattern that works under
  `is_match` and `find_all` crashes the moment it is fed to `stream` on real
  input.
