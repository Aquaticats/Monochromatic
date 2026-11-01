# Resharp v0.6.13 re-verification of the 2026-06-11 findings

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`.

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Re-run of every reproducer from this campaign (`README.md`,
 filed against
v0.6.12) plus the two older `doc/handover/resharp-panic-fix.md` shapes,
 against
the **published `resharp = "=0.6.13"`** (the version this workspace ships;
`package/cli/forbidden-strings/Cargo.lock` pins `resharp 0.6.13`).

Date:
 2026-06-19.
 Host:
 x86_64 (AVX2).
 Probe crate:
 a throwaway
`/tmp/agent/resharp-0613-probe` calling `resharp::Regex` directly,
 each API call
wrapped in `catch_unwind` so one panic does not mask the rest.
 Run in **both**
profiles,
 because the bug classes split by build config (the same split
`doc/handover/resharp-panic-fix.md` warns about):

- debug:
   `debug-assertions = ON`,
   `overflow-checks = ON` (catches the
  `debug_assert!`-gated panics and checked-arithmetic overflows).
- release:
   both OFF (exposes the silent-corruption class that returns wrong
  matches instead of panicking).

The probe tests the production APIs only (`is_match`,
 `find_all`,
`find_anchored`).
 The `stream` API is excluded:
 it is feature-gated,
 off by
default,
 and upstream marks it "do not enable in production"
(`resharp-engine/Cargo.toml`),
 so `bug-03` (a `stream`-only phantom-match
finding) is out of scope here and remains an experimental-surface caveat,
 not a
production defect.

## Headline

Every known crash reproducer and every checkable soundness reproducer is
resolved in v0.6.13.
 One performance finding (`bug-06`,
 full-unicode `\w`
bounded-repeat compile cost) is still live.
 Two items cannot be adjudicated from
this probe alone and are flagged below,
 not claimed fixed.

This says "no *known* reproducer still fires,
" which is not the same as "sound.
"
No fresh fuzzing was run against v0.6.13 in this session;
 the unknown-bug
frontier is unmeasured here.

## Crash class: all closed

No panic in debug or release for any of these.
 In the 06-11 campaign each
panicked (debug) or silently dropped a match (release).

- `bug-04` `/(.*.+)*.+/` on `"aaa"`:
   compiles,
   `find_all = [(0,3)]`,
   no panic.
  The 06-11 reentrant-union-rewrite panic at `resharp-algebra/src/lib.rs:2724`
  is gone.
   Result matches the dotnet reference value `[(0,3)]` recorded in
  `dotnet-adjudication.md`.
- `bug-05a` `/_*(?!_)/` and `bug-05b` `/_*$/` on `"aa"`:
   both compile,
  `find_all = [(0,2),(2,2)]`,
   no panic.
   The `rev_trivial` `debug_assert!` at
  `resharp-engine/src/lib.rs:1824` no longer fires.
- `bug-11` `/((?!a)|b)&(~((c)))/` on `"abca"`:
   no panic in either profile;
  `find_all = [(1,2),(2,2),(4,4)]`.
   The `ldfa.rs` reverse-vs-forward coupling
  panic is gone.
   Span correctness for this pattern is in the unadjudicated set
  below (the dotnet reference rejects this construct class,
   and the Lean truth in
  06-11 covered different subcases,
   so this exact span set is not
  cross-checkable from the probe).

## Soundness class: contradictions resolved

For each,
 the 06-11 defect was an internal contradiction,
 a dropped leftmost
match,
 or an over-long span.
 In v0.6.13 each is resolved,
 either by returning the
previously-established correct set or by failing closed at compile.

- `bug-02a` `/(?<=a)/` on `"b"` and `bug-02b` `/\BU/` on `"U"`:
  `is_match = false`,
   `find_all = []`,
   and `find_anchored = Err(UnsupportedPattern)`.
  The phantom `find_anchored = Some(0,0)` is eliminated;
   the three APIs no longer
  contradict (the anchored path now errors rather than inventing a match).
- `bug-07` `/~(\A|\n+){2}/` on `"\n\n"`:
   default and hardened now agree,
  both `find_all = [(1,1),(2,2)]`.
   The default-vs-hardened divergence is gone.
- `bug-08` `/[0-9]{2}~(\z{1,3}|^{2}\W{0})+/` on `"00"`:
   now
  `compile Err(UnsupportedPattern)`.
   The `is_match = false` vs
  `find_all = [(0,2)]` contradiction is gone (the pattern fails closed).
- `bug-10` `/~(.{1,3}\z){2,4}/` on `"ab"`:
   `find_all = [(0,1),(1,1),(2,2)]`,
  `find_anchored = Some(0,1)`.
   These now agree (the longest match at offset 0 in
  `find_all` is `(0,1)`,
   equal to `find_anchored`);
   the 06-11 length disagreement
  is gone.
- `bug-12` `/((?!b)|ba)&(aa)?/` on `"ab"`:
   `find_all = [(0,0),(2,2)]`,
  `find_anchored = Some(0,0)`.
   The leftmost `(0,0)` is present,
   matching the
  Lean ground truth from 06-11;
   the silent leftmost drop is fixed.
- `bug-13a` `/a?&(?=a)?/` on `"ab"`:
   `find_all = [(0,0),(1,1),(2,2)]`,
  `find_anchored = Some(0,0)`.
   The width-leak that returned `(0,1)` is gone;
   the
  leftmost is `(0,0)`,
   matching the Lean truth.
- `bug-13b` `/(\W|(?!c))&a/` on `"ab"`:
   `find_all = []`.
   This is the correct
  empty language:
   `a` matches only the consuming span `(0,1)`,
   while the left
  branch can satisfy that span only as a zero-width assertion,
   so the
  intersection has no common span.
   The over-long-span leak is gone.
- `arm-bug-01` `/^$/` on `"\n\n"`:
   stock build (SIMD on) returns
  `find_all = [(0,0),(1,1),(2,2)]`,
   which is the known-correct SIMD-off result
  from 06-11.
   The prefilter that dropped `(1,1)` no longer does.
   Caveat:
   this
  probe cannot toggle SIMD off,
   so a true on-vs-off differential needs the
  instrumented build;
   what is verified is that the stock accelerated path now
  returns the full correct set.

## Older panic-fix-handover shapes: fail closed

- `old-crash-1` `/(?:(?=a)&(?<=_))/` on a 64-byte input:
  `compile Err(UnsupportedPattern)`.
   The 0.5.
  x runtime panic is gone;
   fails
  closed at compile,
   as expected since v0.6.9.
- `old-crash-2` `/(?:\w|$)(?:(?![1g]\_X)& a)/`:
   compiles,
   `find_all = []`,
   no
  overflow panic.
   The `saturating_add` fix holds.

## Performance class

- `bug-09` `/.n.................  n./` under `unicode = Full`:
   compiles in 122ms
  (release) and 1.4s (debug).
   The 06-11 40s+ hang is fixed.
- `bug-06` `/\w{24}/` under `unicode = Full`:
   **still live**.
   1.5s release;
   in
  debug it did not finish within the 12s probe budget.
   Default and ASCII modes
  are instant.
   The cost is linear in the repeat count with a large per-repeat
  constant,
   so it scales toward a multi-second-to-DoS compile within the
  `{0,500}` repeat cap.
   This is the only known finding from the two campaigns
  that still reproduces on v0.6.13.
   It is a compile-time resource cost,
   not a
  crash or a wrong answer;
   a caller that bounds untrusted pattern complexity (or
  avoids `unicode = Full` on attacker-supplied `\w{N}`) sidesteps it.

## Not adjudicable from this probe

These are not claimed fixed;
 they need harnesses this session did not run.

- Span correctness of the deliberately-permissive secondary class (`bug-11`,
   and
  in general the constructs the dotnet reference rejects but rust accepts).
   The
  probe confirms no panic and no internal contradiction,
   but leftmost-longest
  correctness needs the Lean or dotnet oracle from `lean-differential.md` and
  `dotnet-adjudication.md`.
- The SIMD on-vs-off differential (`arm-bug-01`) needs `tools/resharp-instr`
  with the `has_simd()` override;
   the stock build only exercises the SIMD-on
  path.
- Unknown bugs.
   This is a re-run of known reproducers,
   not a fuzz campaign.
   The
  06-04 to 06-11 trend (27 root causes,
   then 13) and the 8-day gap since the last
  campaign mean a fresh run against v0.6.13 would be the only basis for any claim
  about residual unknowns.

## Reproduce

```bash
# /tmp/agent/resharp-0613-probe/Cargo.toml: resharp = "=0.6.13"
cd /tmp/agent/resharp-0613-probe
cargo run            # debug: assertions + overflow-checks ON
cargo run --release  # release: both OFF
```

The probe source enumerates each finding with its 06-11 expected-buggy behavior
inline,
 so the diff between "was" and "now" is visible per line.
