# bug-11: find_all reverse pass proposes a null start the forward pass rejects

> Secondary class (see `dotnet-adjudication.md`):
>  the dotnet reference REJECTS
> this pattern at compile ("lookarounds inside union not supported").
>  rust's own
> `ensure_supported_rec` guard accepts it instead,
>  then panics.
>  The crash is a
> defect regardless of subset (a compile-accepted pattern must never panic;
>  rust
> should reject it as the reference does,
>  or handle it),
>  but it is out-of-subset.

Severity:
 crash (debug builds) / soundness (release builds).
 Found by the Lean
position-level differential (the lane the internal oracles cannot reach),
 case
R1612,
 trust0 (anchor-free,
 translator-faithful).
 Confirmed on the unmodified
v0.6.12 stock crate.
 This is a THIRD distinct crash site,
 separate from bug-04
(`resharp-algebra/src/lib.rs:2724`) and bug-05
(`resharp-engine/src/lib.rs:1824`).

Architecture:
 confirmed byte-identical on aarch64 (Apple M1) and x86-64.
`armprobe "((?!a)|b)&(~((c)))" "abca"` on the M1 panics at the same
`resharp-engine/src/ldfa.rs:906` site (the reverse/forward null mismatch is in
the scalar ldfa path,
 not a SIMD path;
 the ARM run makes "ARM-confirmed"
demonstrated rather than inferred).

## Minimal reproducer

```rust
//! Reproduction of the mismatch between the reverse and the forward null checks of the lazy automaton:
//! two inputs make a debug build panic and a release build drop a match,
//! each described in the comments.
// stock resharp v0.6.12, default config
/// The regex type and the option set that its construction takes.
use resharp::{Regex, RegexOptions};

let re = Regex::with_options(r"((?!a)|b)&(~((c)))", RegexOptions::default()).unwrap();
let _ = re.find_all(b"abca");
//        ^ debug-assertions build: panics at resharp-engine/src/ldfa.rs:906
//          release build: returns [(1,2),(4,4)], silently dropping (2,2)
let _ = re.find_all(b"ca");
//        ^ debug-assertions build: panics at resharp-engine/src/ldfa.rs:833
//          release build: returns [(2,2)], dropping the LEFTMOST match (0,0)
```

The pattern is `((?!a) | b) & ~(c)`:
 an alternation whose left branch `(?!a)` is
zero-width (nullable),
 intersected with the complement `~(c)`.
 All three
ingredients are required:
 replacing `(?!a)` with `()` or `(?=a)` removes the
fault,
 and dropping the `|b` alternation removes it (`(?!a)&(~((c)))` is fine).

## Panic

```txt
thread 'main' panicked at resharp-engine/src/ldfa.rs:906:17:
assertion `left != right` failed: find_all: forward scan found no end for reverse-proposed start
  left: 18446744073709551615
 right: 18446744073709551615
   4: <resharp::ldfa::LDFA>::scan_fwd_all
   5: <resharp::Regex>::find_all_dfa
   6: <resharp::Regex>::find_all
```

`18446744073709551615` is `usize::MAX` = `NO_MATCH`.

## Root cause

`find_all` (`find_all_dfa`) is a two-pass driver.
 A reverse scan
(`collect_rev_inner` / `scan_rev_from`) collects candidate match-START positions
into `nulls: &[usize]`;
 `scan_fwd_all` (`ldfa.rs:774`) then runs a forward scan
from each proposed start to find the match end.
 Three sites assert the coupling
invariant "a reverse-proposed start must have a forward end":

- `ldfa.rs:833` (the `nulls.last() == 0` start-at-0 path),
- `ldfa.rs:887` (the in-loop cache-miss path),
- `ldfa.rs:906` (the in-loop non-cache-miss path),

each `debug_assert_ne!(NO_MATCH, l_max_end, "find_all: forward scan found no end
for reverse-proposed start")`,
 guarded by `if l_max_end != NO_MATCH { push }`.

For `(nullable-alternation) & (complement)` this is a FORWARD-pass fault:
 the
reverse pass proposes a LEGITIMATE start (a real match exists there,
 confirmed by
the Lean ground truth below),
 but the forward derivative of the same intersection
is computed non-nullable at that position,
 so `fwd_update` returns `NO_MATCH` and
the forward scan fails to confirm a start that should match.
 The assert message
("forward scan found no end for reverse-proposed start") states exactly this.
 The
`nulls` slice handed to `scan_fwd_all` confirms the reverse pass is correct here:
for `((?!b)|ba)&(aa)*` on `"abab"` it is `[4, 2, 0]` (offset 0 included,
 and Lean
gives `0:0`),
 and the forward scan then rejects offset 0 at `ldfa.rs:837`.
 (This
is the mirror image of bug-12,
 where the reverse pass is the one at fault and
never proposes the legitimate start.
) The invariant is violated:

- Debug-assertions builds (cargo-fuzz,
   any `debug_assertions` consumer,
   `cargo
  test`,
   dev builds):
   the `debug_assert_ne!` fires,
   a hard panic.
   This reaches
  BOTH `find_all` AND `is_match` (`is_match` routes through `scan_fwd_all` for
  this intersection class):
   `is_match(b"abca")` panics at `ldfa.rs:906`,
  `is_match(b"ca")` at `ldfa.rs:833`.
   Any caller on such a pattern crashes the
  process (DoS),
   not just `find_all` callers.
- Release builds (default `debug-assertions = false`):
   the assert is compiled
  out and the `if l_max_end != NO_MATCH` guard drops the unconfirmed start.
  `find_all` then silently omits a real match (soundness;
   see below).
   `is_match`
  is unaffected in release because some other match still exists,
   so the existence
  answer stays correct (`true`);
   the release soundness impact is `find_all`-only.

## Soundness (release), against the Lean ground truth

`llmatch` (leftmost-longest first match) over the same pattern:

```txt
hay     Lean llmatch (truth)   rust release find_all     verdict
"ca"    0:0                    [(2,2)]                   drops leftmost 0:0
"c"     0:0                    [(1,1)]                   drops leftmost 0:0
"abca"  1:2                    [(1,2),(4,4)]             first ok, drops (2,2)
""      0:0                    [(0,0)]                   ok
```

On `"ca"` and `"c"` the engine drops the leftmost match entirely,
 so even
`find_all().first()` is wrong,
 not merely a trailing zero-width omission.

## Provenance and distinctness

Surfaced by the reconstructed Lean position-level lane:
 the AST corpus case
R1612 (`((((?!a))|b)&(~((.&[cd]))))` on `"abca"`) was the only trust0
disagreement in a 1954-case run (the other two disagreements were nested
lookbehind-of-`\A`,
 the documented-unfaithful translator shape).
 The earlier
oracle and libFuzzer runs saw only two panic sites (`algebra:2724` = bug-04,
`engine/lib.rs:1824` = bug-05);
 the `ldfa.rs` site did not appear because the
random/adversarial pattern corpora did not sample this exact shape (a NEGATIVE
lookahead in an alternation,
 intersected with a complement of a char that occurs
in the haystack and differs from the lookahead char).
 The Lean AST generator,
which mixes `Intersection`/`Negation`/`NegLookahead` nodes explicitly,
 did.
 The
release-mode soundness half (`find_all` dropping a match while staying internally
self-consistent) is additionally invisible to the internal oracles and needs the
external position reference;
 the crash half is engine-internal (the engine trips
its own assert).

Trigger shape (all required):
 a negative lookahead `(?!x)` as one branch of an
alternation `(?!x)|...`,
 intersected `&` with a complement `~(y)` where `y`
occurs in the haystack and `y != x`.
 Removing the alternation
(`(?!x)&~(y)`),
 using a positive/optional branch instead of negative lookahead,
or making `y == x` or absent from the haystack,
 all suppress it.

Distinct from:

- bug-04 (`resharp-algebra/src/lib.rs:2724`,
   reentrant union rewrite) -- compile
  path,
   different file.
- bug-05 (`resharp-engine/src/lib.rs:1824`,
   `rev_trivial` find_all dead branch)
  -- different file/line,
   different trigger (`_*$`).
- arm-bug-01 (`fwd.rs:123`,
   SIMD prefilter zero-width drop) -- different driver
  (prefilter vs lazy-DFA `scan_fwd_all`),
   not SIMD-gated (fires with SIMD on).

## Suggested fix direction

The reverse null-collection and the forward derivative must agree on nullability
for `Intersection`/`Negation` nodes.
 Here the reverse pass is correct (it
proposes a start where Lean confirms a match),
 so the fix is on the forward side:
`fwd_update` / the forward derivative under-reports a null end for the
intersection-with-complement node,
 returning `NO_MATCH` where the reverse pass
(and the language) say a zero-width match ends.
 The assert the author wrote
("forward scan found no end for reverse-proposed start") already encodes the
expectation that the forward pass confirm every reverse-proposed start;
 the
intersection-with-complement forward-nullability path is where that breaks.
