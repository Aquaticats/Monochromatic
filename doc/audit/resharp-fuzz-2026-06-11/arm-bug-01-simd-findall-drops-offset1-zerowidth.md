# ARM-BUG-1 SIMD find_all drops the offset-1 match after a leading zero-width match

- Type:
   correctness,
   soundness.
   The SIMD-accelerated `find_all` omits a real
  match;
   the scalar path returns it.
- Phase:
   match time,
   the prefilter-accelerated leading-lookbehind `find_all`
  driver `fwd_lb_prefix_impl`.
- Severity:
   soundness.
   Any caller of `find_all` on an anchored zero-width pattern
  over input with consecutive empty matches silently loses the second match.
- Affected:
   all seven configs;
   both NEON (M1) and AVX2 (x86),
   identically.
- Discovery:
   the `simd_diff` libFuzzer target on the M1 (NEON),
   corroborated by
  the same-machine `has_simd()` on-vs-off differential,
   and reproduced on AVX2.

## Reproducer

```rust
//! Reproduction of the dropped empty match at offset one:
//! the vectorized scan misses it,
//! and the scalar fallback finds it.

/// The regex type whose `find_all` drops the match.
use resharp::Regex;

let re = Regex::new(r"^$").unwrap();
let got = re.find_all(b"\n\n").unwrap();
// SIMD on  (stock, NEON or AVX2): [Match{0,0}, Match{2,2}]   -- MISSING 1:1
// SIMD off (scalar fallback):     [Match{0,0}, Match{1,1}, Match{2,2}]  -- correct
assert_eq!(got.len(), 3); // FAILS on a stock build
```

`"\n\n"` is three empty lines (positions 0,
 1,
 2),
 so `^$` matches at `0:0`,
`1:1`,
 and `2:2`.
 The accelerated path drops exactly the offset-1 match and gets
every other position right.

Differential harness:

```sh
# 5e24 = "^$", 0a0a = "\n\n"; prints a SIMDDIFF line because on != off
repro --pair 5e24 0a0a 0
# SIMDDIFF|cfg=0|fa on=[(0,0),(2,2)] off=[(0,0),(1,1),(2,2)]|pf=1,1,1,0|pat=5e24|hay=0a0a
```

Other witnesses found by the fuzzer (same root cause):
 `^\0?` on
`"\n\x06.\n\x00"` and `^\x01?` on `"\n\x00\xa5\xa4\xa5"` both drop their `1:1`
match.

## Observed versus expected

Expected `find_all(^$, "\n\n") = [0:0, 1:1, 2:2]`.
 The scalar path (and the
hardened path,
 and `regex` crate semantics for `(?m)^$`) all agree.
 The
SIMD-accelerated default path returns `[0:0, 2:2]`.
 On N consecutive newlines the
accelerated path always returns positions `0, 2, 3, ..., N`,
 missing only
position 1.

## Root cause

`resharp-engine/src/fwd.rs`,
 function `fwd_lb_prefix_impl` (the `find_all` driver
used when a forward prefix was built,
 which only happens with SIMD on).
 For a
pattern with a leading lookbehind (`^` lowers to a line-start lookbehind),
 the
begin-nullable block emits the offset-0 match,
 then advances:

```rust
//! The restart position after a match:
//! a match that ends at offset zero restarts the search at offset one.
// resharp-engine/src/fwd.rs:123
search_start = if max_end == 0 { 1 } else { max_end };
```

`search_start` is then used as the lower bound for the lookbehind-candidate
search:
 `fwd_prefix.find_fwd(input, search_start)`,
 where a candidate at byte `c`
produces a body match at `body_start = c + lb_len`.
 After the zero-width begin
match (`max_end == 0`),
 this sets `search_start = 1`,
 so `find_fwd` skips the
`\n` candidate at byte 0,
 whose body position `0 + lb_len = 1` is exactly the
missed `1:1` match.
 The candidate loop never emits `body_start = 0` anyway
(`body_start >= lb_len >= 1`),
 so advancing to `1` is an off-by-`lb_len`
overshoot.

The scalar path takes no prefix (`build_fwd_prefix` returns `None` when
`has_simd()` is false),
 so it scans every position with `scan_fwd_slow` and is
correct.
 This is why the bug is exposed only by the SIMD-on path,
 even though the
faulty line is in the architecture-independent driver,
 not in `neon.rs` or the
AVX2 intrinsics (which only locate the `\n` bytes,
 correctly).

## Fix (verified)

Changing `{ 1 }` to `{ 0 }` at `fwd.rs:123` makes the SIMD-on path return
`[0:0, 1:1, 2:2]` for `^$` and removes the divergence (verified against the
adversarial corpus with no new divergences).
 The candidate loop cannot re-emit
the offset-0 match because its body positions start at `lb_len >= 1`,
 so starting
the candidate search at `0` is safe.

## Relationship to other findings

Distinct from the stream bug (bug-03) and the hardened bug (bug-07),
 which miss
zero-width matches in different drivers (the stream scan and the hardened
`find_all` respectively).
 The shared theme across arm-bug-01 / bug-03 / bug-07 is
mishandling of consecutive zero-width / assertion matches,
 but each lives in a
separate code path.
