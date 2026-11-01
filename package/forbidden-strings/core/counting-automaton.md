# Counting-automaton architecture for forbidden-regex

This note records the counting-automaton rebuild and the performance work that
followed.
 It is historical architecture context,
 not an active handover.

The counting back-end is a Glushkov-style counting NFA (`CountingNfa`):
 it covers
concatenation,
 alternation,
 class repetition,
 and anchors,
 and intersection with
complement runs as a synchronized product of such NFAs.
 So the bounded-repetition
blowup is gone for plain patterns,
 alternation patterns,
 and `&`/`~` alike,
including the real AWS rule.
 Counts live in fixed-width bitsets and the byte step
reuses ping-pong buffers,
 so matching never allocates per byte.
 Only a repetition of
a non-class body (e.g. an optional group `(?:ab)?`) and nested set algebra inside an
operand still route to the eager DFA.

## Throughput status: BEAT regex on BOTH arches (x86 ~1.30-1.32x, arm64/m1 ~1.21-1.23x)

Latest (post-u16):
 x86 dev box ~95-96M lines/s vs regex ~72-74M = ~1.30 to 1.32x;
 Apple
m1 arm64 (hot,
 back-to-back) ~56M vs regex ~46M = ~1.21 to 1.23x.
 The serialized matcher
is 1,383,726 bytes (was 2,376,598;
 a 42% drop from the u16 state-id change,
 verified
neutral-to-better on both arches).
 Three tricks got here from the 1.07x single-pass fold:

0. u16 DFA state ids (`dfa/table.rs`):
    `trans`/`start`/`num_states`/`dead` are u16,
    so the
   hot transition table is half the width.
    No throughput change (the gate's aho-corasick
   DFA prefilter dominates,
    not the table reads) but 42% smaller serialized + half the
   table memory,
    free.
    Engine DFAs cap at 20000 < 65534;
    `from_parts` now takes u16 ids
   directly (callers `build_dfa_within`/`minimize` narrow at the boundary the cap makes
   safe);
    `build_dfa_within` clamps its cap to 65534.

1. DFA aho-corasick kind for the gate's which-rule matcher (`gate.rs`):
    the default NFA
   chases failure links per byte on every flagged line,
    a hidden scalar bottleneck on
   BOTH arches and the reason arm lost before.
    DFA-kind is one lookup per byte (x86
   candidates 90->118M,
    full 72->87M,
    1.07->1.25x;
    arm full 41->53M,
    flipping the loss
   to a win).
    This was the decisive cross-arch fix.
2. Dedup non-anchored gate checks + skip their redundant prefilter (`regex.rs`,
   `engine.rs`):
    a non-anchored rule's whole-line check ignores the hit position,
    so it
   runs once per line via a stack-only `CheckedFull` 256-bit set even when its seed
   recurs;
    and the gate already proved the seed present,
    so `matches_only` skips the
   engine's own prefilter rescan.
    x86 1.25->1.31x,
    arm 1.16->1.22x.

The historical milestones below trace the journey from 0.20x.
 The u16 state-id
shrink,
 fuzzing,
 and mutation testing are already done.
 The remaining practical
optimization from this note is combining the marker line-start DFAs for the boolean
path;
 the all-rules monolithic automaton was measured and rejected in "What is
deliberately not pursued".

## Batch API + across-lines SIMD exploration (measured; one opt-in win)

`is_match_batch(&[&[u8]]) -> Vec<bool>` on `Regex`/`RegexSet` ships,
 scalar per-line by
default.
 On top of it every across-lines layout was raced against the per-line loop on the
real corpus,
 single table-backed pattern with no required literal,
 on x86 (AVX-512) and arm64/m1 (NEON):

- Vertical SIMD across lines (one line per lane,
   gather the transition):
   LOSES both arches
  (x86 0.15 to 0.85x,
   m1 0.43 to 0.88x).
   x86 has no 16-bit gather (a u16 table scalarizes);
  even native u32 `vpgatherdd`,
   and NEON which has no gather at all,
   lose to scalar loads.
- Interleaved scalar (N independent chains,
   no gather):
   parity only with length sorting.
- Branchless equal-length kernel on exact-length buckets,
   N=32:
   the winner.
   Dropping the
  per-lane early-exit branches lets the N chains pipeline.
   Low-match regime:
   x86 1.04x
  ({20}) / 1.09x ({32});
   m1 1.32x / 1.37x.
   LOSES at 47% match (x86 0.75x) because the
  per-line loop early-exits on first match,
   so it is opt-in not default.
   m1 wins far more
  (wide out-of-order rewards the 32-lane memory-level parallelism).
- Bucket-width sweep:
   N=32 sweet spot both arches (16 behind,
   64 regresses).
- Sorting:
   cost ~8ms / 751k refs (~0.5% of a scalar pass),
   near-linear on already-sorted
  input.
   Public `Regex::is_match_batch_bucketed` sorts internally;
   it wins only when the
  caller feeds length-sorted lines (x86 api presorted {32} 1.02x vs unsorted 0.81x:
   the
  per-call sort on unsorted input eats x86's modest kernel win).
   m1's bigger kernel win
  clears the sort.
   The scanner is expected to pre-bucket.
- Set-level concat-buffer gate sweep:
   LOSES both (x86 0.79 to 0.80x,
   m1 0.74x);
  `regex-automata`'s per-line prefilter is already SIMD-optimal on short lines.

Scope:
 the win is for a single full-scan DFA (one `Regex`,
 or a ruleset's seedless group
DFAs).
 The shipped `RegexSet` is gate-dominated with the seedless groups empty,
 so the
bucketed kernel does not help the current set path.

Kernels live in `src/dfa/batch.rs` (scalar/interleaved/tight,
 width-generic),
 `src/dfa/sheng.rs`
and `src/dfa/sheng2.rs` (the permute kernels),
 and `src/regex/batch.rs` (public plus
`#[doc(hidden)]` per-kernel hooks);
 bench sweep in `forbidden-regex.bench/src/kernels.rs`.
The losing vertical SIMD gather kernel was deleted along with its `#![feature(portable_simd)]`,
so the lib now builds on stable Rust again (`cargo +stable check --lib` is clean);
 only the
fuzz crate needs nightly.
 The Sheng kernels use explicit `std::arch` intrinsics,
 which are
stable.

- Sheng in-register transition (`src/dfa/sheng.rs`):
   THE winner,
   and the default for a
  table-backed pattern with no required literal and at most 64 states over a batch of at least 512 lines (the floor
  amortizes a one-time 16 KiB permute-table build).
   Each input byte's transition column is
  one 64-byte permute,
   so the per-byte step is `vpermb` (AVX-512VBMI) / `vqtbl4q` (NEON)
  instead of a dependent table load.
   The state lives in a register (all lanes start equal
  and share each permute,
   so they stay equal and lane 0 is the state);
   position-dependent
  acceptance accumulates in-register as `acc |= accept_mask[state] & ctx_bit[byte]`,
   tested
  once at the end.
   Measured x86 1.45x ({8},
   47% match) / 2.08x ({20}) / 2.19x ({32});
   arm64
  1.13x / 1.54x / 1.64x.
   Wins at every match rate,
   needs no sorting.
   MUST be the explicit
  intrinsic:
   portable SIMD's `swizzle_dyn` scalarizes a 64-byte dynamic shuffle (0.03x).
  Runtime-gated (AVX-512VBMI on x86,
   NEON baseline on arm64);
   scalar fallback otherwise.
  It dominates the bucketed kernel everywhere,
   so the bucketed path stays only for the
  rarer over-64-state seedless DFAs.

- Two-byte composed Sheng (`src/dfa/sheng2.rs`):
   the BEST kernel,
   and the first tier of the
  `is_match_batch` default (cascades sheng2 -> sheng -> scalar).
   One `vpermb`/`vqtbl4q`
  advances the state by TWO bytes via a class-pair transition table,
   halving the one-byte
  Sheng's critical chain.
   Gated to position-independent acceptance (every accept mask all-set
  or all-clear),
   at most 64 states and 16 classes (the pair table is `nc*nc` columns).
  Acceptance over the pair folds into a second table off the chain;
   a trailing odd byte and
  end-of-input use one-byte steps.
   Measured x86 2.23x ({8}) / 3.12x ({20}) / 3.35x ({32});
  arm64 1.49x / 2.06x / 2.19x,
   about 1.3x to 1.5x over the one-byte kernel.

Still open if more is wanted:
 the composition ladder continues (Sheng3/Sheng4 with `nc^k`
class-tuple tables,
 each halving the chain again with diminishing returns and tighter class
limits),
 and Sheng over 64 states via paired permutes.
 None of this touches the
gate-dominated `RegexSet` path;
 it accelerates single-pattern / seedless-rule scanning.

## Correctness infrastructure (fuzzing + mutation testing) complete

Fuzzing and mutation testing were completed before later risky changes,
 so future
larger builds inherit a hardened suite.
 User-directed order,
 fuzzing plus mutation
testing first,
 is satisfied.

Fuzzing (`package/forbidden-strings/core.fuzz`,
 a SIBLING crate like `.bench`;
mirrors `package/forbidden-strings/cli.fuzz`;
 cargo-fuzz / libFuzzer,
 nightly from the
repo-root toolchain):
- `fuzz_compile`:
   arbitrary bytes as pattern + ruleset;
   compile/`compile_lenient` must
  reject,
   never panic or hang.
- `fuzz_from_bytes`:
   arbitrary bytes through `Regex`/`RegexSet::from_bytes`;
   a decoded
  automaton that passes `validate` must run without OOB/panic (the security boundary).
- `fuzz_roundtrip`:
   a generated valid pattern compiled -> `to_bytes` -> `from_bytes`
  must give the SAME verdict on the generated content (serialization preserves meaning).
- `fuzz_differential`:
   a generated NON-algebra pattern vs `regex::bytes` with
  `unicode(false)` (so `\d \w \s \b` are ASCII like ours) on single-line content;
   byte
  verdicts must agree.
- Structured generator in `src/generators.rs` (`PatternAndContent`,
   bounded depth/
  repeat,
   records `uses_algebra` so the differential skips `&`/`~`).
- mise tasks mirror the sibling:
   `list`,
   `build`,
   `smoke`,
   `run`;
   plus a `decode_artifact`
  bin (reconstruct a pattern from a libFuzzer artifact).
   Generator caps repeats to
  simple-atom bases so it explores realistic patterns fast.
- STATUS:
   TWO real bugs found and fixed,
   then re-fuzzed CLEAN:
  1. `fuzz_from_bytes`:
      a decoded RegexSet with inconsistent parallel vectors panicked at
     match time -> `RegexSet::validate_structure` (1.2M runs clean after the fix).
  2. `fuzz_roundtrip`/`fuzz_differential`:
      deeply nested bounded repetition blew up the
     DFA build's residuals -> compile OOM -> `RESIDUAL_NODE_CAP` guard in `dfa/build.rs`
     (re-fuzzed clean:
      roundtrip 26k,
      differential 62k,
      compile 36k,
      from_bytes 1.2M,
      all
     exit 0,
      no OOM/crash/mismatch).
      Both have regression tests in `tests/integration.rs`.
  Two clean campaigns (~millions of runs) = enough fuzzing;
      proceeded to mutation.
- User directive (satisfied):
   do ENOUGH fuzzing before starting mutation testing.

Test suite:
 grew from ~20 ("laughably few") to 170,
 all sidecar `*_tests.rs` (max-lines
exempt) per module:
 charset,
 countset,
 parse,
 ast/smart,
 nullable,
 prefilter,
 derivative,
gate,
 dfa/table,
 dfa/build,
 dfa/minimize,
 counting/element,
 counting/build,
 counting/nfa,
engine,
 error,
 build (routing),
 group,
 plus integration (from_bytes adversarial,
 fold
routing,
 products,
 regex public surface,
 dialect semantics).
 The pathological-compile
repro is `#[ignore]`d so it does not slow the default suite or per-mutant mutation runs.

Mutation testing (cargo-mutants on the engine crate) -- DONE:
- Runs inside the Podman container (mutated = arbitrary code,
   must not run on the host);
  `mutants.Containerfile` + mise `test:mutation:image`/`test:mutation`.
   Mount the crate
  with `--security-opt label=disable` (no host relabel);
   `mutants.out/` gitignored at root.
- Three full runs (711 mutants,
   ~17 min each).
   First run 483 caught / 140 missed (75%);
  after adding targeted sidecar tests,
   FINAL run 563 caught / 55 missed / 70 unviable /
  23 timeouts = 88% caught,
   ~91% detected (timeouts are infinite-loop mutations,
   detected).
- The 55 remaining survivors are deliberately left:
   behaviorally-EQUIVALENT mutants
  (CheckedFull dedup is a per-line optimization with the same matched set;
   `empty()` vs
  derived `Default::default()`;
   carry `|` vs `^` where the shifted bit is always 0;
  `clear_above` boundary that is unreachable;
   `line_start_candidate`/`candidates_only`
  whose result is unchanged),
   bench-only DIAGNOSTICS (lib `debug_seedless`),
   and
  parser/validate-INTERNAL arm-routing and bound-arithmetic mutants that change an error
  path or an internal index but not the observable accept/reject.
   None is a real bug.
- Tests that killed survivors target:
   every `validate` decode rejection branch (Dfa,
   Nfa,
  Product,
   Engine -- the decode security boundary),
   the residual guard,
   intersection-seed
  extraction,
   countset higher-word scan + sizing,
   all rule-routing classifications,
   the
  regex public surface + diagnostic hooks,
   minimize's distinct_count,
   error rendering,
   the
  repetition cap,
   and dialect semantics.

## Earlier status: 1.07x of regex on the real-repo corpus (BEAT regex; was 0.20x)

Honest stable bench (16 threads;
 forbidden-regex SHARED across threads because it is
immutable,
 regex CLONED per-thread so its lazy-DFA cache scales not contends;
 both
deployed at their best is the fair comparison).
 Corpus = EVERY non-gitignored line in
this repo (~747k lines) via `ignore::WalkBuilder`,
 the same walk the real scanner uses
(`bench/src/corpus.rs`).
 Ruleset = the shipped rules,
 ported to our dialect
(`bench/src/port.rs` + `normalize.rs`),
 compile-filtered to ~251 rules both engines
accept.
 Build+serialize ~28s (budget 60s;
 the non-anchorable rules now build DFAs).

- forbidden-regex:
   ~72M lines/s.
   regex:
   ~68-73M lines/s (noisy).
   Ratio ~1.07x on the
  winning run;
   gate-only ~76M is robustly above regex's range.
   Confirm robustness with
  repeated runs (regex noise band is ~67-73M;
   full must stay above it).

The win came from collapsing the TWO-pass design (gate + a second literal-free pass,
which capped at ~39M) into ONE gate pass,
 then making that pass faster than regex.

Per-line profile (`prefilter_only` / `candidates` / `anchored_only` / `gate_only` /
`seedless_only` / `csa_union_only` hooks on `RegexSet`,
 surfaced in the bench):

- prefilter-only (regex-automata SIMD literal prefilter):
   ~160M (faster than regex).
- candidates (prefilter + aho-corasick enumeration,
   no per-rule check):
   ~94M.
- anchored-only (gate,
   counting-fallback rules skipped):
   ~92M.
- gate-only (the one real pass:
   prefilter + per-rule checks):
   ~76M.
- full (gate + the near-free line-start first-byte reject):
   ~72M.
- seedless-only:
   empty (every literal-free rule folded into the gate).
- csa-union-only:
   ~0.8M (the counting union,
   kept only as the fold's oracle).

### What got us from 0.55x to 1.07x: the single-pass fold (all committed)

The seedless second pass was the whole gap (gate alone already beat regex).
 Folding
all literal-free rules into the one gate pass,
 then three perf fixes,
 crossed regex.
See the "fold" section below for the full account;
 in short:
 weak/leading/line-start
seeds put every rule in the gate;
 non-anchorable rules use the eager DFA (not the slow
counting NFA) for their on-hit full-line check;
 the DFA match loop early-exits on the
dead sink;
 and the line-start marker checks are skipped by a first-byte set.

### Cross-arch: WIN on both (the arm loss was the AC NFA, now fixed)

The arm loss was diagnosed and FIXED.
 Diagnosis (m1 profile,
 before the fix):
prefilter-only 95M (2.08x regex,
 our SIMD prefilter wins on arm),
 but candidates 49M
(the aho-corasick `find_overlapping` NFA walk ate the whole lead,
 a 95->49 cliff),
gate-only 38M,
 full 41M (0.90x,
 LOSE).
 The scalar AC failure-link walk on every flagged
line was the bottleneck (worst on arm,
 but it cost ~15M on x86 too).
 Fix:
 force the DFA
aho-corasick kind (one lookup per byte).
 After:
 m1 candidates 66-75M,
 gate 46-50M,
 full
49-53M,
 1.11-1.16x (WIN,
 hot,
 across 3 back-to-back runs);
 x86 candidates 118M,
 full
87M,
 1.25x.
 So the same trick fixed both arches.

Methodology note:
 m1 is not sufficiently cooled,
 so keep it maximally HOT (no cooldown
between runs) and read the back-to-back ratio,
 which is stable (~1.11-1.16x) even as
absolute throughput throttles down over consecutive runs.
 The m1 clone is at
`/Volumes/MacData/Monochromatic-bench` (cloned from origin,
 which the auto-pushing `git`
wrapper keeps current,
 including FORCE pushes;
 m1's own `git` is stock `/usr/bin/git`,
updated with `git fetch --depth 1 origin main && git reset --hard origin/main`).

### What got us from 0.20x to 0.55x (all committed)

1. Seedless grouping (`src/group.rs`):
    the literal-free rules collapse into a few
   union DFAs (greedy,
    cap-bounded) instead of N scalar scans.
    2 groups at cap 6000.
2. SIMD gate (`src/gate.rs`):
    `regex_automata::util::prefilter::Prefilter` for the
   negative-line fast reject;
    aho-corasick only maps a hit back to rules.
3. Anchored DFA fast-check:
    a seeded rule whose seed is its LEADING literal compiles
   to a bare (no-`Σ*`) anchored DFA,
    run at the gate's hit position instead of the slow
   counting/product sim.
    `leading_seeds` (gate on the leading literal so the hit
   position is the rule start);
    `anchored_engine`;
    `RegexSet.anchored: Vec<Option<Engine>>`;
   gate passes the hit position to `matches_rule`.
4. Leading-literal extension (`prefilter.rs::concat_leading`):
    extend a leading
   alternation by the mandatory bytes after it,
    so `(?:sk|rk)_(?:test|live|prod)_...`
   gates on `sk_`/`rk_` (rare,
    anchorable) not inner `test`/`live`/`prod` (common).
5. DFA-safe routing (`regex.rs`):
    a non-anchorable seeded rule with only small
   repetitions (`max_repeat <= 3`,
    e.g. the line-anchored marker alternations) routes
   to the fast eager DFA,
    not counting.
6. Bench:
    combined the ~109 shortcode-label marker rules in
   `forbidden-strings.append.txt` into two `^(?:CODE|...):` alternation rules (one fast
   automaton each,
    not 109 counting rules);
    porter keeps a leading `^` (line-start
   anchor) and strips a class-repeat nested after a leading `(?:` group.

### Why two passes cannot beat regex, and the one-pass path

`full = 1/(1/gate + 1/seedless)`.
 seedless is pinned at ~75M:
 the 2 union DFAs cannot
merge into one (the `{n,m}` overlap blows the eager DFA past any cap;
 measured at cap
60000 it still needs 2 groups).
 With seedless fixed at ~75M,
 NO gate value crosses
regex's ~73M.
 Both maxed (gate->~95M,
 seedless->~75M) caps at ~48M.
 The only way past
is ONE automaton over ALL rules,
 one lookup/byte,
 like regex's combined lazy DFA.

### CsA finding (MEASURED, settled): a seedless second pass cannot win, at all

Built and measured a single counting NFA over the union of all 5 seedless rules
(`RegexSet.seedless_union`,
 `csa_only_is_match` hook,
 corpus-wide oracle check in the
bench).
 Result:
 the union pass is 0.82M lines/s (90x SLOWER than the 75M DFA groups),
and it agreed with the DFA groups on all 747479 corpus lines (correct,
 just slow).
 The
46-position union is dominated by the per-byte closure over the `^`-anchored marker
rule's ~25 literal positions.

The determinized CsA would remove the closure-recompute but not the per-byte counter
work,
 so at absolute best it MATCHES the DFA groups (a DFA is 1 lookup/byte;
 a CsA is 1
lookup + counter ops/byte).
 And that ceiling is irrelevant,
 because the arithmetic
forbids ANY second-pass win:
 with the measured gate at 82M and
`full = 1/(1/gate + 1/seedless)`,
 beating regex (72M) needs
`1/seedless < 1/72 - 1/82 = 0.001694`,
 i.e. seedless > 590M lines/s (~15.8 GB/s).
 No
byte-scanning automaton reaches that (the SIMD literal prefilter,
 which only SKIPS,
peaks at 5.6 GB/s;
 a real per-byte DFA tops ~3 GB/s).
 So the seedless CsA is a
dead end for the goal,
 and so is a single union DFA,
 and so is any second pass.

### The winning lever (DONE): fold every literal-free rule into the ONE gate pass

The gate path alone (the seeded rules with the SIMD prefilter) already ran at ~82M,
above regex.
 The entire deficit was the second literal-free pass.
 Folding every
literal-free rule into the single gate pass (`src/build.rs`,
 `route_rule`/`fold_seedless`)
deleted that pass;
 three perf fixes then made the one pass faster than regex.
 The 5
literal-free rules and how each folds in (signals measured by `rg` over the repo):

- azure `[delim][..]{3}\dQ~[..]{31,34}`:
   gated on required inner literal `Q~` (8 lines);
  no leading literal,
   so the full rule engine runs on a hit.
   `seeds_from_node_min(.,1)`.
- SK `SK[hex]{32}`:
   leading `SK` (393 lines) -> anchored-at-hit.
   `leading_seeds_min(.,2)`.
- vault `(hvs\.[..]{90,120})|(s\.[..]{24})`:
   leading `hvs.`/`s.` (35650) -> anchored-at-hit.
- facebook `\d{15,16}(\||%)[..]{27,40}`:
   gated on its 1-char `|`/`%` (~8000 lines),
   full
  DFA on a hit.
   `WEAK_INNER_SEED_LEN = 1` (only literal-free rules reach this floor).
- RULE_A `^(?:INF|...):` and RULE_B `^(?:PR|...)[0-9]:`:
   `^`-anchored deny-code markers.
  Routed to a LINE-START check (`route_rule` handles `starts_with_line_anchor` FIRST,
  before seeds),
   gated by a first-byte set so the check is skipped on almost every line.

The three fixes that made `full = gate'` clear regex (all in the perf commit
"beat regex on the bench"):

1. Non-anchorable rules build the eager DFA (`build_table_kind`,
    fallback to counting on
   blowup),
    not the counting NFA.
    Their engine runs over the whole line on every hit,
    and
   the inner-keyword rules hit common keywords;
    a per-byte DFA beats the counting scan.
   Gate 70M -> 75M.
    Cost:
    build+serialize 9s -> 28s (the inner-keyword DFAs).
2. `Dfa::is_match` early-exits on the dead sink (`Dfa::dead`,
    found at build,
    validated on
   decode so a hostile blob cannot name an accepting state dead).
    Without it every
   anchored/line-start check walked the whole line after dying.
    Full 47M -> 66M.
3. Line-start first-byte reject:
    `RegexSet.line_start_first: ByteSet` (rebuilt in
   `prepare`,
    via `Dfa::mark_first_bytes`) is the union of the markers' possible first
   bytes;
    `line_start_candidate` skips the anchored marker checks unless `line[0]` is in
   it.
    The markers begin almost no lines,
    so this is near-free.
    Full 68M -> 72M.

Oracle (kept):
 `RegexSet.seedless_union` is the counting NFA over the original 5
literal-free rules;
 the bench asserts `seedless_union.is_match(line) => is_match(line)`
on all 747k lines (no literal-free match missed).
 The 246 seeded rules are unchanged by
construction (adding other rules' seeds to the shared gate never changes a rule's own
check).
 Parity vs regex stays 15/15 on the sample.

### Still in the tree (cleanup candidates)

- `RegexSet.seedless_union` + `csa_only_is_match` + `seedless_union_size`:
   now only the
  fold's oracle (serialized,
   ~adds to size).
   Keep while validating,
   or drop once settled.
- `forbidden_regex::debug_seedless` (doc-hidden) + bench `bin/seedless.rs`:
   one-shot
  diagnostics for the literal-free shapes.
   Removable.
- bench `csa-union-only` row:
   measures the oracle pass (0.8M),
   not a real path.

### If more headroom is wanted (not required; we already win)

- Combine the two marker line-start DFAs into one engine (one call instead of two on a
  first-byte candidate line).
   Closes the small remaining `full` (72M) vs `gate` (76M) gap.
- The all-rules single-pass CsA (AC literal skeleton + counter registers on prefix-complete
  states) remains the theoretical ceiling,
   but is unneeded now.
   The user is holding
  additional tricks for further gains.

### Constraints (do not violate)

- Immutable / `Send + Sync`:
   NO per-thread mutable state for our engine (the bench
  shares one instance across threads;
   regex is cloned per-thread).
   A lazy DFA with a
  per-thread cache is therefore disallowed;
   the CsA must be eager (built at compile,
  serialized).
- Win requirement:
   beat regex on this bench,
   on this machine and `ssh m1`,
   when
  neither is under heavy load.
   Quiet-system regex is ~73M on the full corpus.
- Build+serialize <= 60s.
   Bench is hard-bounded (10s/engine,
   bounded parity sample).
- AKIA-style fixtures in the corpus are deliberate test inputs,
   not real secrets.
  If a hook flags them,
   handle them under the repository's current forbidden-strings fixture policy.

## What this crate is

A restricted regular-expression engine for the `forbidden-strings` secret
scanner.
 It must support intersection (`&`) and complement (`~(...)`),
 which
plain NFA/backtracking engines do not handle as first-class operations.
 It matches one
line as raw bytes and answers a boolean.
 A whole ruleset compiles into one `RegexSet`
that combines a set-level literal gate with per-rule exact engines.

## Locked design decisions (from the requirements interview)

Do not relitigate these;
 they were settled with the user.

- Output:
   boolean `is_match` only (no offsets).
   `RegexSet` also reports which rule
  ids matched via `matches`.
- Character model:
   byte-oriented (`&[u8]`).
   Classes are 256-bit byte sets.
  `.` = any byte except `b'\n'`.
   Word set for `\b`/`\w` is ASCII `[A-Za-z0-9_]`.
- Semantics:
   unanchored search (match any substring).
   `^ $ \b` anchor to line and
  word boundaries.
- Flags:
   `multiline` and verbose (`x`) are always on;
   there is no flags argument.
  Verbose mode ignores unescaped whitespace (including newlines) outside classes,
  so one rule may span many lines.
   A line whose first character is `#` is a
  comment to end-of-line;
   a `#` elsewhere is literal.
   Whitespace and `#` inside
  `[...]` are literal.
- Grammar:
   operands of `&` and `|` must each be a single atom (a literal,
   a class,
  `.`,
   an anchor,
   a `(?:...)` group,
   or a `~(...)`).
   Concatenations and quantified
  atoms must be wrapped in `(?:...)` to be operands,
   so there is no operator
  precedence.
   `(?:a|b)` is valid;
   `(?:ab|cd)` and `\bX\b & ~(Y)` are rejected
  until wrapped.
   `~(...)` is always parenthesized.
- Supported constructs:
   `\t`,
   `[abc]`,
   `[a-z]`,
   `[a-zA-Z]`,
   `[^...]`,
   `.`,
  `(?:a|b)`,
   `a?`,
   `a{3}`,
   `a{3,6}`,
   `^`,
   `$`,
   `\b`,
   `&`,
   `~(...)`,
   and the
  shorthands `\d \w \s \D \W \S`.
- Strict rejection (`CompileError::Syntax`):
   `*`,
   `+`,
   unbounded `{n,}`,
   `\xNN`,
  capturing `(`,
   lookaround and inline flags (`(?` not followed by `:`),
  backreferences,
   unknown escapes,
   unbalanced brackets,
   stacked quantifiers,
   and
  `{n,m}` with `n > m`.
- Empty-matchable patterns are rejected at compile (`CompileError::EmptyMatchable`)
  because under unanchored search they would match every input.
- Serialization:
   serde derive plus bincode 1.
  x (`to_bytes` / `from_bytes`).
  `from_bytes` validates structure before the automaton is ever run.
- Package:
   `package/forbidden-strings/core/`,
   crate `forbidden_regex`,
  edition 2024.
   New `package/rust-module/` category for Rust library crates.

## The problem that forced the rebuild

The first back-end compiled patterns to an eager byte-class DFA by desugaring
`{n,m}` into unrolled copies and determinizing.
 That is correct but explodes
exponentially in the repeat count whenever a pattern's literal prefix overlaps its
repeated class.
 Measured serialized sizes (release):

- `AKIA[A-Z]{2}` is 670 B,
   `{8}` is 3.7 KB,
   `{16}` is 72 KB (exponential).
- `xyzw[A-Z2-7]{16}` (disjoint prefix) is 1 KB;
   a pure literal is under 0.5 KB.
- A two-rule `RegexSet` already serialized to 1.8 MB.

This is the classic `.*X.{k}` DFA blowup,
 and AWS-style keys
(`AKIA[A-Z2-7]{16}`) are exactly that shape.
 Minimization (Moore,
 implemented)
does not help:
 those states are genuinely Myhill-Nerode-distinct because the
determinized automaton tracks the set of overlapping active counts.

## The chosen fix: counting automaton (DFA structure + counter registers)

User directive:
 "do NOT let DFA own the countings.
" Decision:
 the resharp/RE#
counting-set approach.
 Keep a small serializable automaton over byte-classes,
 but
represent `{n,m}` as a runtime counter register,
 never as unrolled states.
 The
overlapping active counts under unanchored search live in a counter-SET register
(updated per byte),
 not in the structural state,
 so the structural automaton stays
small and the count cannot blow it up.

Sketch of the target model:

- Structural states form a small DFA over byte-classes.
   A `Repeat{C, min, max}`
  region is one structural state with a self-loop on bytes in `C`.
- Each repeat owns a counter register holding the SET of active counts.
   On a byte
  in `C`:
   increment all counts (cap at `max`,
   drop above).
   On entry:
   add a fresh
  count.
   The exit transition to the continuation is guarded by
  `register ∩ [min, max] != ∅`.
- Acceptance is a structural accept plus any counter guards,
   and stays
  position-dependent (the 4-bit mask over `(word_after, line_end)` from the old
  table generalizes).
- `&`:
   run operands together;
   the gate accepts when both accept.
   `~(R)`:
   the
  complement targets in real rules are simple (fixed strings such as
  `AKIA2{16}` which is the literal "AKIA" then sixteen '2's),
   so determinize and
  complement that small,
   counter-free operand and run it alongside.

This is essentially reimplementing the counting core of RE#;
 build it
incrementally.

## Current code state

Front-end (done):

- `src/charset.rs`:
   `ByteSet` (256-bit) plus the shorthand sets and
  `is_word_byte`;
   now derives serde so a counting program can carry byte sets.
- `src/error.rs`:
   `CompileError`.
- `src/context.rs`:
   `Ctx` (line_start,
   line_end,
   word_before,
   word_after).
- `src/ast/node.rs`:
   `Node` algebra including `Repeat { node, min, max }`.
- `src/ast/smart.rs`:
   smart constructors;
   `repeat(node, min, max)` and `optional`.
- `src/parse/*`:
   cursor (verbose-mode skipping),
   escape,
   class,
   repeat (emits a
  `Repeat` node,
   no longer desugars),
   grammar,
   and `parse.rs` (empty-match guard).
- `src/nullable.rs`:
   handles `Repeat` (`min == 0 || nullable(body)`).
- `src/derivative.rs`:
   interim countdown derivative for `Repeat` (still used by the
  eager DFA path;
   correct but blows up,
   which is why linear patterns now skip it).
- `src/dfa/classes.rs`:
   `collect_sets` recurses into `Repeat`.

Back-ends:

- Leaf and bitset:
   `src/counting/element.rs` holds `Element` (the position kinds
  `Class`,
   `Counted`,
   `LineStart`,
   `LineEnd`,
   `WordBoundary`) and `validate_element`.
  Counts live in `src/counting/countset.rs` (`CountSet`):
   a fixed-width bitset
  indexed by count (`bit i` set means count `i` is live),
   sized to the element's
  `max`.
   Entry is a bit-or,
   the exit guard a shift-and-test,
   and a matched byte
  advances every live count at once with one multi-word left shift.
- Shared simulation core:
   `src/counting/sim.rs` holds `State` (active positions
  `0..=len`,
   where `len` is the virtual accept,
   plus per-position count bitsets),
  `closure`,
   `step_into` (fills a reused destination,
   no allocation),
   and
  `boundary_ctx`.
   It walks follow sets,
   not an implicit chain,
   so it serves any NFA.
- Counting NFA (DONE):
   `src/counting/nfa.rs` (`CountingNfa { elements, follow,
  start }`,
   `is_match`,
   `validate`) and `src/counting/build.rs` (`build_nfa`,
   a
  Glushkov first/last/follow walk;
   tests in `src/counting/run_tests.rs`).
   Covers
  concatenation,
   alternation,
   class repetition,
   and anchors;
   returns `None` for
  intersection,
   complement,
   `Top`,
   or a repetition of a non-class body.
   The search
  in `src/counting/run.rs` ping-pongs two `State` buffers and re-seeds `start` every
  boundary (the `Σ*` prefix).
   `AKIA[A-Z2-7]{16}` is under 2 KB.
- Synchronized-product back-end (DONE):
   `src/counting/product.rs` (`ProductProgram`,
  `build_product`,
   `validate`;
   tests in `src/counting/product_tests.rs`).
   A
  `Node::Inter` whose operands each `build_nfa` splits into positives (must match the
  same span) and negatives (the `~(...)` operands,
   none may match that span).
   One
  thread per start runs every operand NFA in lockstep over the same bytes,
   so the
  same-span set algebra holds;
   each thread reuses a spare buffer side,
   pruned in
  place via `retain_mut`.
   The real AWS rule
  `(?:\b(?:(?:A3T[A-Z0-9])|(?:AKIA)|(?:ASIA))[A-Z2-7]{16}\b) & ~(AKIA2{16})` is under
  4 KB and decides keys exactly per prefix branch.
- Eager DFA (still used for a repetition of a non-class body,
   e.g. an optional group
  `(?:ab)?`,
   and nested set algebra inside an operand):
   `src/dfa/build.rs`,
  `src/dfa/table.rs` (`Dfa` table,
   match loop,
   `validate`),
   `src/dfa/minimize.rs`.
  The product-union gate (`src/dfa/union.rs`) and the `Dfa::step`/`start_state`/
  `accept_mask_of` helpers it used were REMOVED;
   a counting-aware gate replaces it.
- `src/engine.rs`:
   `Engine { Nfa(CountingNfa), Product(ProductProgram), Table(Dfa) }`
  picks the back-end (`is_match`,
   `validate`).
- `src/build.rs`:
   selects back-ends for single patterns and routes set rules into the gate,
  anchored-at-hit checks,
   line-start checks,
   or seedless union groups.
- `src/gate.rs`:
   `SetGate` owns the SIMD literal prefilter plus DFA aho-corasick which-rule
  matcher over seeded rules.
- `src/regex.rs`:
   `Regex` wraps one `Engine`;
   `RegexSet` stores per-rule engines for exact
  checks plus `anchored`,
   `line_start`,
   `seedless_groups`,
   `seedless_union`,
  and the rebuilt `SetGate`.
   `compile`,
   `new`,
   `compile_lenient`,
  `from_ruleset`,
   `is_match`,
   `matches`,
   `to_bytes`,
   and `from_bytes` are
  the public surface.

Core regression coverage:

- `tests/integration.rs`:
   end-to-end (literals,
   anchors,
   classes/shorthands,
  repetition bounds,
   alternation/intersection/complement,
   empty-match rejection,
  `RegexSet` + serialize round-trip,
   `from_ruleset`).
- `src/counting/run_tests.rs`:
   differential vs the eager DFA oracle,
   the
  serialized-size proof,
   and exact `{16}` bound checks.

## Working notes

Use mise tasks when editing this crate.
 Direct `cargo` commands drift from the repo task
contracts and should not be used for normal iteration.

- Fast tests:
   `mise run //package/forbidden-strings/core:test:debug`.
- Debug build:
   `mise run //package/forbidden-strings/core:build:debug`.
- Release test gate:
   `mise run //package/forbidden-strings/core:test`.
- Lint gates before declaring a milestone done:
   `mise run //package/forbidden-strings/core:lint:rust` and
  `mise run //package/forbidden-strings/core:lint:clippy`.
- Stage scoped pathspecs (`git commit package/forbidden-strings/core ...`),
  never `-A`/`.`.

TDD targets:

1. DONE.
    Front-end:
    `Repeat` flows through parser/nullable/derivative;
   `tests/integration.rs` green.
2. DONE (oracle is the eager DFA).
    The counting simulation is diffed against it in
   `run_tests.rs::linear_agrees_with_oracle`.
3. DONE.
    Counting back-end for linear `class{n,m}` under search;
    the size proof
   (`counted_key_stays_small`,
    under 2 KB) and exact-bound checks pass.
4. DONE (linear operands).
    `&` and `~` are layered onto the counting model via the
   synchronized product in `src/counting/product.rs`.
    The hard constraint held:
   under `Σ*·(A & B)` the SAME substring must satisfy both operands,
    so the operands
   are not run as independent search automata (that would match different spans);
   instead one thread per start runs every operand in lockstep and accepts only when
   all positives and no negatives accept at the same boundary.
   `product_tests.rs::product_agrees_with_oracle` diffs it against the eager DFA.
5. DONE.
    The operand IR is a counting NFA (`build_nfa`),
    so alternation costs only
   follow edges and the real AWS rule
   `(?:\b(?:(?:A3T[A-Z0-9])|(?:AKIA)|(?:ASIA))[A-Z2-7]{16}\b) & ~(AKIA2{16})` leaves
   the eager DFA (`product_tests.rs::aws_rule_with_alternation_stays_small`,
    under
   4 KB).
    Standalone alternation patterns are diffed against the oracle in
   `run_tests.rs::linear_agrees_with_oracle`.
    Note the grammar requires each `&`/`|`
   operand to be a single atom,
    so every multi-character alternation branch must be
   wrapped:
    `(?:(?:AKIA)|(?:ASIA))`,
    not `(?:AKIA|ASIA)`.
    A repetition of a non-class
   body (an optional group `(?:ab)?`,
    `(?:ab){2,4}`) is the remaining shape that
   `build_nfa` rejects;
    it routes to the eager DFA.
    To lift that,
    `build_repeat`
   would unroll a small-bound non-class body into fresh position copies (mandatory
   copies for `min`,
    optional copies up to `max`),
    falling back to the eager DFA when
   the bound is large;
    the counter-set trick does not apply to a multi-position body.
6. DONE.
    `RegexSet` gate and seedless fold.
    The current set path uses one SIMD
   required-literal gate for seeded rules,
    anchored-at-hit DFAs for leading-literal
   rules,
    line-start DFAs for marker rules,
    and capped union DFAs only for rules that
   remain truly literal-free.
    The all-rules monolithic automaton was measured and
   rejected because it state-explodes or loses the prefilter advantage.

## Lint and style reminders specific to this crate

- 300 code-line budget per `.rs` (`monochromatic-rust-linter`);
   split into sibling
  modules,
   never disable.
   Comments do not count,
   so verbose docs are free.
- Rustdoc on every documentable item (`use`,
   fields,
   variants,
   impl blocks,
  modules,
   the file).
   Write the `What:`/`Why:` explainer as `///`;
   the documented
  fixture density (one to three lines) is enough.
- No recursion over flat input;
   recursion only over the node tree (structural).
- Functional style,
   `const`/immutable where reasonable.

## Benchmark scope and target hardware

The throughput goal is narrow and explicit:
 beat the `regex` crate on the bench
sidecar (`package/forbidden-strings/core.bench`),
 on two machines,
 measured
only when each is otherwise idle.
 We do NOT need to beat `regex` universally.

- This machine:
   AMD Ryzen 8700F,
   64 GB RAM.
- `ssh m1`:
   Apple M1,
   16 GB RAM.
- Both CPUs have many hardware threads,
   and the bench is only meaningful when the
  machine is not under heavy load.
   Many threads plus the immutable,
   `Send + Sync`
  `RegexSet` (no `Mutex`,
   unlike resharp) mean a parallel scan is a legitimate lever;
  a fair bench parallelizes both engines,
   so threading alone does not move the ratio.
- `resharp` is deferred indefinitely as a baseline:
   its serialized matcher is too
  slow to be a useful comparison.
   Bench only against `regex`.
- The bench loads the real shipped ruleset (`forbidden-strings.local.example.txt`
  plus `forbidden-strings.append.txt`),
   ports each rule into this dialect,
   and
  compile-filters to the subset both engines accept;
   our scanner gets the `&`/`~`
  versions,
   `regex` gets the complement-stripped positives.
   Each engine is timed for
  a fixed wall-clock budget (10 s) and reported in lines/s and MB/s.

## Verification end state (unchanged from the plan)

- Lint clean (`lint:rust`,
   `lint:clippy`),
   all nextest tests pass.
- `tests/integration.rs` crosses the crate boundary:
   build a `RegexSet`,
  serialize,
   reload via `from_bytes`,
   and match.
- Differential correctness vs `regex` in a throwaway worktree on the plain rules
  (set-algebra rules have no `regex` equivalent).
- Throughput:
   the `bench/` sidecar measuring lines/second on a pre-serialized
  `RegexSet` for `forbidden_regex` vs `regex`,
   beating `regex` on both machines.
