# BUG-21 is_match/find_all results depend on prior queries (lazy DFA cache contamination)

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Classification

- Type:
   correctness,
   soundness.
   A reused `Regex` returns a different,
   wrong answer
  for the same input depending on which inputs were queried before it.
   The query
  methods are not pure functions of their argument.
- Phase:
   match time,
   the reverse lazy-DFA path shared across all query methods.
- Severity:
   high.
   `Regex` is built to be compiled once and queried many times;
   this
  is the documented and only sensible usage.
   Under that usage the primary API
  `is_match` silently returns wrong answers.
   The same `is_match(h)` call repeated on
  one instance flips `false` to `true`.
   The corruption also drives `find_all` to
  emit a span whose `end` is the raw `NO_MATCH` sentinel (`usize::MAX`),
   which a
  caller slicing `data[start..end]` would fault on.

## Minimal reproducer

```rust
//! Reproduction of the lazy automaton cache contamination:
//! an identical call,
//! or a call after a different query,
//! changes its own answer on one `Regex` instance,
//! and the no-match sentinel leaks into `find_all`.

/// The regex type whose cached automaton is contaminated.
use resharp::Regex;

// Repeating an identical call flips its own answer on a single instance.
let re = Regex::new(r"\Bb").unwrap();
assert_eq!(re.is_match(b"ba").unwrap(), false); // 1st call: correct (no match)
assert_eq!(re.is_match(b"ba").unwrap(), false); // 2nd call: FAILS, returns true

// Cross-query form: a prior NON-matching query corrupts a later one.
let re = Regex::new(r"\Bb").unwrap();
assert_eq!(re.is_match(b"aa").unwrap(), false); // ok, no match
assert_eq!(re.is_match(b"ba").unwrap(), false); // FAILS, returns true
// a freshly built Regex returns the correct false for b"ba".

// The contamination also leaks the in-band NO_MATCH sentinel into find_all.
let re = Regex::new(r"\B(\w&b?)").unwrap();
let _ = re.is_match(b"aa");
let spans = re.find_all(b"ba").unwrap();
// spans == [Match { start: 1, end: 18446744073709551615 }]  (end == usize::MAX)
```

`\Bb` on `"ba"`:
 `\B` (non-word-boundary) is false at offset 0 (true start-of-input
before a word char is a boundary),
 and offset 1 holds `a`,
 not `b`.
 So there is no
match anywhere and the correct answer is `false`.

## Observed behaviour

The standalone harness (`/tmp/agent/contam-test`) prints:

```text
is_match(ba) [reused]       = true     <- wrong
is_match(ba) [fresh regex]  = false    <- correct
ba,ba on fresh instance: first=false second=true

find_all(ba) reused = [Match { start: 1, end: 2 }]      <- phantom span
find_all(ba) fresh  = []                                <- correct
find_anchored(ba) reused = Some(Match { start: 0, end: 1 })
```

The first query against a freshly built instance is correct;
 the corruption appears
only on a subsequent query against the same instance.
 Discovered by the
`STREAMINCONSIST` and `BOUNDS` buckets of the directed `check_one` sweep
(`repro --checkbatch`),
 which reuse one `Regex` across a sequence of haystacks:
 24
`\B`+intersection patterns reported `is_match=true` with an empty `stream`,
 and
`\B(\w&b?)` reported a find_all span ending at `usize::MAX`.
 Both reproduce on
default,
 full,
 and hardened,
 so the defect is config-independent.

## Expected behaviour

`is_match`,
 `find_all`,
 `find_anchored`,
 and `stream` must be pure functions of the
input for a given `Regex`.
 Querying one input must not change the result of a later
query of a different (or the same) input.
 `\Bb` on `"ba"` must return `false` and an
empty `find_all` on every call,
 in any order.

## Independent corroboration

No external oracle is needed:
 the engine contradicts itself across calls,
 and a
freshly built instance disagrees with a reused one on identical input.
 The standard
semantics also agree with the fresh (correct) answer:
 the `regex` crate matches
`\Bb` against `"ba"` nowhere (`\B` is false at the leading word char),
 matching
resharp's first/fresh `false` and against its contaminated `true`.

## Root cause

`is_match` for a non-anchored pattern runs the reverse collector
(`resharp-engine/src/lib.rs:1894`,
 `rev_ts.collect_rev_first`).
 The reverse engine
`rev_ts` lives in the shared,
 mutex-guarded `inner` and persists across queries;
 its
lazy DFA tables (`begin_table`,
 `center_table`,
 `state_nodes`,
 `effects_id`,
`engine.rs:200`) are filled on demand by `create_state` (`engine.rs:545`) and reused
by later calls.

The terminal step of the reverse scan,
 `handle_rev_end` (`engine.rs:1470`),
 resolves
the begin-of-input boundary by taking a lazy transition on the first byte's minterm:

```rust
//! The lazy transition taken for the first byte of the data:
//! the transition table is read and extended while the cache that it shares with earlier queries may hold stale state.
// engine.rs:1477
let mt = self.mt_lookup[data[0] as usize] as u32;
let new_state = self.lazy_transition(b, sid, mt)?;   // engine.rs:1478
```

`lazy_transition` (`engine.rs:391`) returns `center_table[delta]` when present,
 so
this transition is keyed only on `(sid, minterm)`.
 The `debug`-feature trace of two
consecutive `is_match(b"ba")` calls on `\Bb` shows the two calls entering this step
with the identical state (`[rev pos=1] state=2`) and identical first byte,
 yet
diverging:

```text
CALL 1:  [pre end] state=3  eid=0   -> [rev end] state=3  eid=0    => false (correct)
CALL 2:  [pre end] state=12 eid=5   -> [rev end] state=12 eid=5    => true  (wrong)
                                 node= ... | (?=){1}
```

The same `(sid, mt)` lazy transition returns state 3 on the first call and state 12
on the second.
 State 12's node carries an extra alternative `(?=){1}`,
 a begin-anchor
lookahead resolved as satisfied;
 `collect_rev_complex` then records a null at the
begin and `is_match` returns `true`.
 The boundary transition is therefore not a
stable function of `(state, minterm)`:
 it is mutated by query history.
 The begin-of-
input context (`\B` resolved against the true start) is not part of the cached
state's identity at `handle_rev_end`,
 so a state cached during one scan supplies a
wrong begin-anchor nullability on the next.

The downstream `usize::MAX` span (`find_all` returning `Match { end: NO_MATCH }`)
is the same corruption surfacing through the forward driver after the reverse
collector seeds a phantom start position,
 compounded by the find_all push path not
guarding `max_end == NO_MATCH` before emitting a `Match` (the in-band sentinel
escaping into a public value,
 see `code-quality.md`).

## Affected configurations

Default,
 full,
 and hardened all reproduce (the `\B` boundary transition is shared by
every config).
 Config-independent;
 the limits-disabling `unbounded_size` config is
irrelevant since this is a correctness,
 not a size-limit,
 defect.

## Relationship to other findings

- Distinct from BUG-3 (`is_match` vs `find_all` on a fresh instance):
   that is a
  fixed per-input path divergence;
   this is history dependence,
   where the same
  instance and input change answer across calls.
- Distinct from BUG-20 (`find_anchored` ignores a leading begin assertion):
   BUG-20 is
  a deterministic begin-context omission in the forward `scan_fwd_slow`;
   this is a
  caching coherence failure in the reverse lazy DFA shared across queries.
   They share
  the theme that the begin-of-input boundary is mishandled,
   and are likely worth
  fixing together (make the begin/boundary context part of cached state identity).
- The `usize::MAX` leak makes concrete the in-band `NO_MATCH = usize::MAX` sentinel
  concern recorded in `code-quality.md`:
   the sentinel is reachable in a public
  `Match.end`,
   so the "proven scalar-local-only" assumption behind keeping the
  sentinel does not hold once contamination is in play.
- The contamination does not stop at wrong answers:
   it can drive the engine into a
  state that trips the `engine.rs:960` assertion (BUG-2) and panics.
   `\w+b` in the
  default config,
   `find_all` over the sequence `["ab", "ba"]`,
   returns a correct
  result for `"ab"` then panics on `"ba"` (a fresh `\w+b` on `"ba"` does not panic).
  That panic then poisons the shared `inner` mutex and permanently bricks the whole
  `Regex` (BUG-25).
   So this defect chains contamination -> abort -> permanent brick.

## Code quality

The transition cache treats `(state, minterm)` as a complete key while the begin-of-
input nullability is an additional,
 uncaptured dimension of the state at
`handle_rev_end`.
 A reasonable fix is to fold the begin/end boundary context into the
state identity (distinct begin-resolved states) rather than resolving it ad hoc at
the terminal step against a cache that cannot tell the contexts apart.
 The sentinel
escape additionally argues for replacing the in-band `usize::MAX` `NO_MATCH` with
`Option<usize>` at the find_all push boundary so a corrupted scan cannot surface as a
valid-looking span.
