# Method and oracles

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Five lanes against resharp 0.6.13 (fuzzed at HEAD `f0ce60a`,
 tests-only delta from
`d89964b`),
 on x86_64 (AVX2) and Apple M1 (AArch64,
 NEON).
 All tooling is under
`/tmp/agent/resharp-denot-oracle` (the oracles),
 `/tmp/agent/resharp-upstream-investigate`
(the upstream clone + in-tree fuzz),
 and on the M1 under
`~/resharp-fuzz-2026-06-19`.

## Adjudication tiers

With the dotnet RE# reference being retired (see README),
 real-vs-by-design is
decided by,
 in order:

1.  Any panic,
     crash,
     or compile-time DoS on a compile-accepted pattern:
     always
    real.
2.  Violation of an asserted contract (the three `match_invariants` invariants:
    find_all ordering,
     is_match iff find_all non-empty,
     find_anchored = longest at
    0):
     real soundness bug.
3.  Positional wrongness on a faithful-fragment (trust0) pattern,
     established by
    the independent denotational oracle or the Lean oracle:
     real soundness bug.
4.  Cross-API inconsistency on an accepted-superset pattern where `api.md` already
    says the invariant is "stated intent,
     known to be violated":
     catalogued as a
    known gap,
     not inflated to a new finding.

The supported subset is the arbiter for "real bug vs engine correctly rejecting /
permissive by design":
 `doc/syntax.md`,
 `doc/features.md`,
 `doc/api.md` in the
upstream repo.
 `features.md` is explicit that the conservative fragment (no
lookaround-in-union,
 no anchors-under-complement) is verified,
 while the
accepted-superset "has less formal backing ... recent fuzzing found most of its
soundness issues exactly there".
 Both findings this campaign reports are in the
accepted superset,
 consistent with that warning.

## Lane 1: in-tree libFuzzer

The four upstream targets (`compile`,
 `diff_regex`,
 `match_invariants`,
`simd_diff`) built with `cargo fuzz build --target x86_64-unknown-linux-gnu` (the
gnu target avoids the musl/cc ASAN incompatibility) on x86,
 and `cargo fuzz build`
on the M1.
 Run unbounded-time,
 RSS-capped (8192 MB x86,
 4096 MB M1) to avoid host
OOM.

Result:
 0 crashes on either arch.
 Iterations on x86 by the harvest point:
`diff_regex` ~1.23M,
 `simd_diff` ~670k,
 `match_invariants` ~567k,
 `compile` ~76k
(the compile target is slow because of the `\w` compile-cost family).
 The only
artifacts produced anywhere were three slow-units (one `compile` and one
`match_invariants` on x86,
 one `match_invariants` on M1),
 all of the `\w`/`\b`
bounded-repeat compile-cost family (`compile-cost-recheck.md`),
 none of them
crashes.
 `simd_diff` (the SIMD fast-vs-slow differential) is clean on both arches.

## Lane 2: independent denotational oracle (headline external check)

`src/main.rs`.
 The point is an oracle whose implementation strategy differs from
resharp's,
 so disagreements are uncorrelated.
 resharp is Brzozowski symbolic
derivatives;
 a derivative oracle would share its blind spots.
 Instead,
 membership
of substring `s[i..j]` in `L(node)` is computed by direct structural recursion:

```text
Pred(c): j==i+1 and s[i] in c        Concat(a,b): exists k. a[i..k] and b[k..j]
Union(a,b): a[i..j] or b[i..j]       Inter(a,b): a[i..j] and b[i..j]
Compl(a): not a[i..j]                Star(a): i==j or exists k>i. a[i..k] and star[k..j]
Opt(a): i==j or a[i..j]
```

memoized on `(node, i, j)`.
 On top,
 leftmost-longest `find_all[0]` (== Lean
llmatch),
 `is_match`,
 and `find_anchored` are derived and differentiated against
resharp under `UnicodeMode::Ascii`.

Fragment only (no anchors,
 no lookarounds),
 where substring-language membership is
unambiguous;
 generation is biased into the accepted-superset danger zone reachable
without lookarounds:
 intersection with a nullable (zero-width) operand,
 and
complement.
 AST-first with fully-parenthesized rendering,
 so neither side's parser
precedence can drift.

Validation before trust:
 on the plain-regex subset (no `&`/`~`) the oracle's
EXISTENCE (is_match) is checked against the `regex` crate.
 Span/start are NOT
checked against the regex crate:
 the crate is leftmost-FIRST and was observed to
return non-leftmost-longest spans on greedy nested stars (e.g. `(2,3)` where Python
`re` and resharp both give `(1,4)`),
 so only existence is a sound cross-check.
After fixing an initial anchored-vs-unanchored harness bug,
 the oracle is at 0
existence disagreements vs the regex crate over millions of pairs.

Result:
 217,754,018 pattern-input pairs over 60 seeds on x86 (plus 20 seeds on the
M1),
 0 disagreements on llmatch,
 is_match,
 and find_anchored,
 and 0 crashes,
 in
both release (silent-mismatch class) and debug (assert/overflow class) profiles.
This is the strong clean signal for `find_all` correctness on the fragment plus
intersection/complement/zero-width.

## Lane 3: engine-internal self-consistency (the lane that found both bugs)

`src/bin/selfconsist.rs`.
 Covers the anchor/lookaround superset the denotational
oracle cannot reach,
 needing no external oracle by checking resharp's own asserted
contracts against itself across all four unicode modes and hardened/default:

```text
C1 is_match  <=> find_all non-empty
C2 find_anchored=Some(m) => m.start==0 and m is the longest find_all match at 0
C3 find_anchored=None    => no find_all match starts at 0
C4 default find_all == hardened find_all
C5 stream non-empty <=> find_all non-empty   (experimental; catalogued separately)
```

Result over 6,899,984 pairs:
 C1 surfaced the is_match false positive
(`bug-is-match-false-positive-inter-optional-end-anchor.md`);
 C2/C3 surfaced the
find_anchored bug (`bug-find-anchored-end-anchor-union.md`);
 C4 = 0 violations
(default and hardened always agree);
 C5 = the experimental `stream` phantom matches
only (`stream-experimental.md`).
 No crashes,
 in release or debug.

## Lane 4: Lean formal position differential

See `lean-differential.md`.
 The recovered `extended-regexes` Lean formalization
(`Regex.MatchingAlgorithm.llmatch`) rebuilt on the M1 (mathlib cache),
 with a
fresh AST-first paired generator (`gen_pairs.py`) emitting matched resharp strings
and Lean terms for the lookaround superset (faithful zone),
 `readpairs` for the
resharp side.
 Result:
 2310 comparable cases,
 0 disagreements (the rest resharp
rejects at compile).
 Formally confirms the bug-11/12/13 positional class fixed.

## Lane 5: compile-cost characterization + 06-11 re-verification

`src/bin/perf.rs` and `anchored_probe`/`trig`/`c1probe`/`c1min` minimizers;
 the
13 06-11 findings re-checked in `verification-vs-2026-06-11.md`.

## Stopping condition

Lanes ran until the denotational oracle exhausted the fragment with no new
disagreements over 200M+ pairs,
 the self-consistency families converged on the two
findings (both minimized to a stable trigger),
 the Lean lane reached 0
disagreements over thousands of comparable cases,
 and the in-tree fuzzers produced
only compile-cost slow-units across millions of executions.
 A mostly-clean result
at this depth is reported as raised confidence in 0.6.13,
 not as under-fuzzing.
