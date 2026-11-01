# Is resharp bulletproof? Robustness assessment, 2026-06-19

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`.

Assessment of whether `ieviev/resharp` (RE#,
 the Rust crate this workspace
depends on via `package/cli/forbidden-strings`) is "bulletproof,
" at
`resharp 0.6.13` (the published version we ship).
 Synthesizes the consumer-side
audit trail (`doc/troubleshooting/resharp.md`,
 `doc/audit/resharp-fuzz-*`,
`doc/handover/resharp-*`),
 an upstream source-and-history investigation,
 and a
fresh empirical re-verification of every known reproducer against v0.6.13
(`doc/audit/resharp-fuzz-2026-06-11/verification-0.6.13.md`).

Superseded in part by the deeper multi-oracle campaign
`doc/audit/resharp-fuzz-2026-06-19/`,
 which ran an independent denotational
oracle (217M pattern-input pairs),
 the recovered Lean formally-verified position
oracle (0 disagreements over 2310 lookaround-superset cases),
 an engine-internal
self-consistency lane,
 an anchor-extended denotational oracle,
 and the in-tree
libFuzzer targets on x86 (AVX2) and M1 (NEON).
 That campaign confirmed the prior
crash/soundness fixes hold but found THREE new live soundness residuals that update
the "one live item" claim below:
 a `find_anchored` phantom/missing span on
`(\z|$)$`,
 an `is_match` false positive on `_&(?:[ab]|$)?`,
 and (most seriously) a
`find_all` FALSE NEGATIVE on `.&a(?:$|b)` where both operands match (0,1) but the
intersection drops it.
 All three are arch-independent and trace to anchors dropped
by forward simplification;
 the `find_all` one is the bug-02/08/10 family reaching
the production API itself (fail-open).
 The third was found only after extending the
denotational oracle to anchors,
 because the earlier lanes were structurally blind
to a coordinated find_all/is_match false negative.
 `find_all` is therefore NOT
sound on the intersection-with-anchor zone,
 correcting the line below.
 See that
directory's `README.md`.

## Verdict

No,
 not bulletproof.
 Much closer than it was,
 and for our constrained usage it is
safe,
 but "bulletproof" is the wrong word for a 0.
x derivative-based engine that
was surfacing reachable panics and silent mis-matches as recently as eight days
before this assessment.

The honest shape of the answer has two readings,
 kept separate on purpose.

- resharp the crate,
   on arbitrary patterns:
   not bulletproof.
   Strong and rapidly
  hardening,
   with a real safety floor still below "bulletproof.
  "
- forbidden-strings,
   our consumer,
   on our rule set:
   safe,
   but that safety is
  achieved by wrapping resharp (catch_unwind,
   overflow-checks,
   pre-validators),
  not by resharp being sound underneath.
   The wrapper's safety must not be read as
  the crate's soundness.

## What "we contributed a lot" actually means

The contributions are real and substantial,
 and they are the reason the gap can
be measured at all.

- This project (GitHub `Aquaticat`,
   commit email `an@aquati.cat`) is the single
  largest external contributor to resharp:
   8 merged pull requests (#12,
   #14,
   #15,
  #16,
   #20,
   #24,
   #25,
   #26,
   #28) plus one maintainer commit carrying a
  `Co-authored-by: Aquaticat` trailer (`af6f2a5`),
   against a maintainer baseline
  of ~156 commits.
- The contributions are not only bug fixes.
   They include the fuzz harness (#12),
  the CI workflows that run tests plus deterministic fuzz-replay on every push
  and random fuzzing nightly (#24),
   the SIMD-vs-scalar differential target (#25),
  and the cross-API contract docs (#26).
   The differential oracle against the
  battle-tested `regex` crate and the Lean position-level ground-truth pipeline
  are this project's apparatus.
- Seven substantive issues were filed (#5,
   #17,
   #18,
   #19,
   #21,
   #22,
   #27).
   Only
  two remain open,
   and both are improvement requests,
   not defects:
   #22
  (unify the find_all / find_anchored / stream drivers) and #19 (split source
  files under 300 lines).
   The round-two correctness issue (#21) is closed.

So the apparatus this project built is precisely what keeps demonstrating resharp
is not yet bulletproof.
 That is a more useful answer than either "yes" or a vague
"no."

Note on identity:
 repo commits use `an@aquati.cat`;
 the account email on file is
`old.key0210@fastmail.com`.
 The GitHub-handle link to `Aquaticat` is solid;
 the
two emails are taken to be the same person (this is their monorepo) but that
equivalence is not independently verified from repo artifacts.

## Empirical state at v0.6.13 (the version we ship)

Full per-finding detail and reproduction steps:
`doc/audit/resharp-fuzz-2026-06-11/verification-0.6.13.md`.
 Summary:

- Every known crash reproducer (bug-04,
   bug-05,
   bug-11,
   and the two older
  panic-fix-handover shapes) no longer panics,
   in debug or release.
- Every checkable soundness reproducer (bug-02,
   bug-07,
   bug-08,
   bug-10,
   bug-12,
  bug-13,
   arm-bug-01) is resolved,
   either by returning the previously-established
  correct match set or by failing closed at compile.
- One performance finding is still live:
   bug-06,
   full-unicode `\w{N}` bounded
  repeat,
   a compile-time cost (1.5s at N=24 optimized) that scales toward a DoS
  within the `{0,500}` cap.
   It is a resource cost,
   not a crash or wrong answer.

This was a re-run of known reproducers.
 It establishes "no known bug fires on
v0.6.13,
" not "v0.6.13 is sound.
" No fresh fuzzing was run against v0.6.13 here.

## Why "bulletproof" is the wrong bar (structural)

Several facts hold regardless of the reproducer results,
 and they cap how strong
any robustness claim can be.

- It is 0.
  x (38 releases,
   no 1.0),
   with `rust_version = 1.70`.
   Upstream signals
  pre-stability by its own versioning.
- It is a deliberate PCRE subset,
   not a drop-in:
   no backreferences,
   no lazy
  quantifiers,
   no `find` / `captures`,
   leftmost-longest (not leftmost-greedy),
  multiline on by default.
   Several of "our" findings come from resharp being more
  permissive than its own dotnet reference:
   it accepts construct classes the
  reference rejects (lookaround-in-union,
   anchor-in-complement) and then has to
  get them right.
- No internal panic boundary:
   `catch_unwind` appears only in resharp's tests,
   not
  its library.
   Any `unwrap()` / `assert!` / `panic!` reached at runtime unwinds
  into the caller.
   There are roughly 38 library `unwrap()` sites in the engine.
- No `#![forbid(unsafe_code)]` anywhere;
   ~114 `unsafe` blocks,
   concentrated in
  the SIMD paths of `resharp-engine`.
   That is normal for a SIMD engine,
   but it
  means memory-safety rests on the correctness of hand-written intrinsics,
   which
  is exactly where arm-bug-01 lived.
- Invariants behind `debug_assert!` corrupt silently in release rather than
  panicking.
   This is why our consumer build sets `overflow-checks = true` and
  keeps the pre-validators:
   the worst case for a secret scanner is a rule that
  fails open.
- One architectural restriction is permanent by design:
   lookaround inside a
  complement body (`A&~(.*\bX\b.*)` shapes) is rejected,
   because reversing a
  complemented language with position-sensitive constraints does not compose in
  the symbolic-derivative algebra.
   Held by the consumer guard
  `engine::lookaround_in_complement`.
   See `doc/troubleshooting/resharp.md`.

## Residual risk and what would raise the bar

- Run a fresh fuzz campaign against v0.6.13 (the differential,
   oracle,
   and Lean
  lanes from the 06-11 method),
   since the last campaign targeted v0.6.12.
   The
  27-then-13 trend means a new version is the right time to re-measure unknowns.
- Adjudicate the secondary permissive class (bug-11 and kin) with the Lean and
  dotnet oracles:
   confirm whether v0.6.13 returns correct spans or merely stopped
  contradicting itself.
- Bound full-unicode `\w{N}` compile cost upstream,
   or avoid `unicode = Full` on
  attacker-supplied bounded repeats (bug-06).
- Keep the consumer defense stack.
   It is load-bearing by design for unknown
  shapes:
   `panic = "unwind"`,
   `overflow-checks = true`,
   the `catch_unwind`
  wrappers,
   and the pre-validators
  (`package/cli/forbidden-strings/Cargo.toml`,
  `package/cli/forbidden-strings/src/rule/engine.rs`).
   None should be reverted
  on the strength of "the known bugs are fixed.
  "

## Bottom line

For forbidden-strings:
 safe to ship on v0.6.13.
 The fail-closed wrappers mean a
future regression degrades to a synthetic hit,
 not a silent CI pass,
 and the one
new severe defect (a `find_all` false negative) does not reach our rule set,
 which
uses no intersection-with-anchor patterns.
 For resharp as a general-purpose engine
on untrusted patterns:
 substantially hardened and actively maintained,
 with
`find_all` matching an independent oracle across a 217M-pair search on the
anchor-free fragment,
 but with the known-bug list at v0.6.13 standing at THREE
narrow soundness residuals on the accepted-superset end-anchor zone (`find_anchored`
phantom,
 `is_match` false positive,
 and a `find_all` false negative on
intersection-with-anchor that is fail-open and algebra-deep,
 issue #22) plus one
acknowledged compile-cost item (the deeper `doc/audit/resharp-fuzz-2026-06-19/`
campaign).
 `find_all` is therefore sound on anchor-free and simple-anchor patterns
but NOT on intersection-with-anchor.
 It is a young 0.
x engine with `unsafe` SIMD,
no internal panic
boundary,
 and an unmeasured unknown-bug frontier.
 Robust and improving fast;
 not
bulletproof.
