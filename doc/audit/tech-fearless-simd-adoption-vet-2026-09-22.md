# Fearless SIMD adoption vet

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/rust-module/forbidden-regex.bench` is now `package/forbidden-strings/core.bench`;
> `package/rust-module/forbidden-regex` is now `package/forbidden-strings/core`.

- Status: in progress; no adoption recommendation or score.
- Lifecycle phase: targeted evidence, not validated.
- Subject: `fearless_simd` adoption for existing Rust packages in this monorepo.
- Scope: identify a concrete consumer and compare retaining its current implementation with SIMD abstraction candidates.
- Started and last updated: 2026-09-22.
- Governing skill: `.agents/skills/choosing-technology/SKILL.md` at `a05818ad70a40e5769a36de669697ba109891b31`.
- Governing skill SHA-256: `393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`.
- Compatibility fingerprint (schema 1): `4d98a75047162e9871d9d05491221898c181feb5467cded2ba5a64d46779ff18`.
- Fingerprint input:
- `{"baseCategories":["inspectable open-source local technology"],"deployment":{"platforms":["Android arm64","Linux x64"
- ,"macOS arm64","Windows x64"]},"hardConstraints":["No product mutations without adoption authorization"],"incumbent":n
- ull,"overlays":["multi-platform","native"],"schemaVersion":1,"scope":"Rust SIMD abstractions for current monorepo cons
- umers","subject":"fearless_simd adoption","trustBoundary":"native Rust library"}`.
- Active audit owner: current Pi session.
- Prior compatible report: none found at report creation.
- Base category: inspectable open-source local Rust crate; native-code and multi-platform overlays apply.
- Hard constraints: do not mutate product files on evaluation; retain required consumer behavior and supported targets.
- Criteria frozen provisionally before scoring: measured consumer throughput, correctness, safety proof surface,
- platform fit, maintenance, dependency and build surface, readability. Unspecified priorities have weight 1. No scores
- before validation.
- Unresolved preferences: none yet. A specific consumer and measured regression threshold must be determined before a
- candidate ranking.

## Context and initial evidence

- `package/rust-module/forbidden-regex/src/dfa/sheng.rs:226-249,281-303` and `sheng2.rs:248-285,319-345` use a
- runtime-gated AVX-512VBMI kernel on x86 and NEON on arm64, with scalar fallback. These are architecture-specific
- `vpermb` and `vqtbl4q` algorithms, not merely a portable lane-wise loop.
- `package/rust-module/forbidden-regex/OPTIMIZATIONS.md:329-391` records measured regressions from a `std::simd`
- dynamic shuffle and from a cross-line SIMD gather; the latter was removed. This is prior measurement, not a fresh
- reproduction. It does **not** prove a regression for the different `fearless_simd` shuffle implementation.
- `package/rust-module/forbidden-regex/Cargo.toml:31-57` already delegates literal scanning to `memchr`,
- `aho-corasick`, and `regex-automata`; replacing their internal SIMD dispatch is not an API-level change to this crate.
- `package/music-player/truepeak-core/src/meter.rs:196-231,310-418` uses a stateful, interleaved, per-channel
- four-sample interpolation. This is a potential numerical kernel, but SIMD benefit and floating-point policy parity
- have not been measured. Contrary to stale migration prose in the core manifest, **both consumers are integrated**:
- `package/music-player/desktop-app/Cargo.toml:138` and `package/music-player/android-app/rust/Cargo.toml:72` depend on
- the core, while their `src/truepeak.rs` files import it. Android's JNI entry invokes the shared meter at
- `android-app/rust/src/lib.rs:700-711`.
- Root `mise.toml:81-96` uses nightly Rust repo-wide. The forbidden-regex README describes stable-compatible production
- SIMD at `:202`.
- A source-and-manifest search of `package/` for `core::arch`, `std::arch`, target-feature annotations,
- feature-detection intrinsics, and SIMD library imports found owned explicit Rust SIMD in
- `forbidden-regex/src/dfa/sheng.rs` and `sheng2.rs`. This is a scoped inventory of searched Rust sources, not a claim
- that no transitive crate uses SIMD. `package/rust-module/forbidden-regex/Cargo.toml:49-65` delegates literal
- prefilters to `memchr`, `aho-corasick`, and `regex-automata`; the desktop and Android music-player manifests depend
- on `gxhash` (`package/music-player/desktop-app/Cargo.toml:49`, `android-app/rust/Cargo.toml:38`). Adopting a library
- for owned kernels would not switch those libraries' internal algorithms.
- `package/music-player/truepeak-core/src/meter.rs:196-231,310-418` is currently a scalar stateful meter. Vectorizing
- it is a separate design and measurement question, not a migration of an existing SIMD implementation.
- The current scanner's public batch path goes through `package/rust-module/forbidden-regex/src/regex/batch.rs:76-79`
- to `src/engine.rs:318-324`, which selects the two-byte Sheng kernel, then one-byte Sheng or scalar fallback in
- `src/dfa/sheng2.rs:251-297` and `src/dfa/sheng.rs:222-259`. Replacing a vector kernel alone does not replace the
- entire scanner pipeline. Because this scanner protects against secret leakage in CI, a targeted replacement must
- receive the high-trust overlay before adoption; the current compatibility fingerprint models only the broader
- library-level scope and must be revised if a scanner-specific vet becomes the decision scope.

## Discovery query schedule and ledger

Frozen initial schedule for this scope, prior to candidate ratings:

- crates.io, exact/category queries: `portable SIMD`, `SIMD runtime dispatch`, `fearless_simd`, `pulp`, `wide`.
- GitHub repositories and code: `Rust portable SIMD runtime dispatch`, `fearless_simd`, `pulp`, `wide`, `safe_arch`.
- Web: `Rust stable portable SIMD runtime dispatch alternatives fearless_simd pulp wide safe_arch multiversion crates.io
- 2026` and `fearless_simd alternative Rust SIMD`.
- Local: `fearless_simd|std::simd|portable_simd|core::arch|is_x86_feature_detected|target_feature|pulp|wide::|simdeez|si
- md|vectoriz` in Rust packages and associated decision/optimization records.

Completed searches so far:

- Local `rg` over Rust packages, unfiltered after an initial overbroad capped prose search: current owned SIMD code
- appeared in `forbidden-regex`; Cargo locks for file-manager packages include `pulp` transitively. No package manifest
- searched had `fearless_simd`.
- Web search with literal query in schedule, Exa fast with fallback: found the Linebender release, crate documentation,
- `pulp`, and `simply-simd`; provider output had no pagination metadata. This does not saturate a source class.
- Direct provided URLs, primary documentation: Linebender v1.0 post dated 2026-09-22, docs.rs 1.0.0 and companion
- macros 0.1.0. Cached crates.io page/API still described 0.6.0/0.7.0; direct `curl` against
- `/api/v1/crates/fearless_simd/1.0.0` returned HTTP 403 on 2026-09-22. The 1.0 version claim uses docs.rs and the
- tagged repository; published artifact checksum and resolver availability remain unverified.
- GitHub repository clone: `~/temp/agent/fearless-simd-2026-09-22`, commit `dcbd824a897a098ec0124c088f971ac6d890d0cd`,
- tag `fearless_simd-v1.0.0`.
- Expansion round and registry pagination: pending. Source-class saturation: pending, not blocked or complete.

Candidates and discovery sources:

### Retain current implementation

- Discovered from local `forbidden-regex` source and optimization record.
- Category: existing owned implementation, native/multi-platform overlays.
- Screening: pass for current consumer; it does not directly replace unrelated libraries' internals.

### `fearless_simd` 1.0.0, optional `fearless_simd_macros` 0.1.0

- Discovered from supplied URLs, docs.rs and the Linebender GitHub tag.
- Category: inspectable open-source local library, native/multi-platform overlays.
- Screening: pending targeted verification, no known hard-gate failure. Core and companion macro require distinct
- audits.

### `pulp`

- Discovered from docs.rs/web search and transitive Cargo locks under file-manager packages.
- Category: inspectable open-source local library, native/multi-platform overlays.
- Screening: pending metadata/source audit.

### `wide` and `safe_arch`

- Discovered from existing SIMD notes in `doc/audit/tech-meow-cache-key-hash-vet-2026-09-17.md:553`.
- Category: inspectable open-source local libraries, native/multi-platform overlays.
- Screening: pending category fit and source audit.

### `simply-simd`

- Discovered from web search for runtime-dispatch alternatives.
- Category: inspectable open-source local library, native/multi-platform overlays.
- Screening: pending metadata/source audit.

## Targeted `fearless_simd` evidence records

- `fearless_simd` 1.0.0, license and dependency surface, primary `fearless_simd/Cargo.toml:1-43` and root
- `Cargo.toml:1-25` at pinned tag, accessed 2026-09-22: Apache-2.0 OR MIT, MSRV 1.89; core has an optional `libm`
- dependency and defaults to `std`. License appears compatible with current crates but has not received a complete
- dependency or provenance audit.
- Optional macro 0.1.0, primary `fearless_simd_macros/Cargo.toml:1-23` and
- [macro documentation](https://docs.rs/fearless_simd_macros/0.1.0/fearless_simd_macros/), accessed 2026-09-22: uses
- `proc-macro2`, `quote`, and `syn` 3; rejects async and const functions, and limits target-feature scope to execution
- inside the annotated function. This extends the audit surface. The core crate does not depend on the macro.
- Dispatch behavior, primary `fearless_simd/src/lib.rs:286-369,639-650` at pinned tag: the `Avx512` auto-detection
- requires the full Ice Lake feature group including VBMI2, VAES, SHA, and other features, while current Sheng kernels
- gate only F, BW, and VBMI. Thus automatic `Avx512` dispatch can select a lower level on some CPUs supporting the
- existing kernel; this is a source-derived conditional, not a measured regression. `as_avx512` only returns an
- existing proof token; constructing one from the narrower guard without meeting its full contract would be unsound.
- Exact shuffle mapping, primary `fearless_simd/src/generated/avx512.rs:13009-13021` and `generated/neon.rs:6383-6404`
- at pinned tag: the fast 64-byte dynamic shuffle directly uses `_mm512_permutexvar_epi8` on AVX-512 and four
- `vqtbl4q_u8` operations on NEON. The current Sheng NEON kernel in
- `package/rust-module/forbidden-regex/src/dfa/sheng.rs:326-343` uses one `vqtbl4q_u8` for a broadcast 16-byte state. A
- portable rewrite may therefore duplicate shuffles on arm64 unless optimized; its code generation and speed have not
- been measured.
- Provenance, primary tag and
- [security policy](https://github.com/linebender/fearless_simd/blob/main/fearless_simd/SECURITY.md), accessed
- 2026-09-22: upstream promises security backports to newest releases for each supported MSRV for at least three years
- from MSRV release. Its exact policy is not a promise to patch every 1.0.x version independently.
- Test matrix, primary `.github/workflows/ci.yml` at pinned tag, accessed 2026-09-22: CI includes stable MSRV, native
- Linux/Windows/macOS, emulated SSE2/AVX2/AVX-512 and baseline arm64, wasm, sanitizer and macro UI jobs. Their presence
- is not proof of a passing run at this revision.
- Public tracker sample, `gh issue list --repo linebender/fearless_simd --state all --limit 12` and corresponding
- `gh pr list`, accessed 2026-09-22: issues #381 (open SSE2 rounding performance), #364 (closed NEON `fract` edge
- correctness), and #353 (open swizzle operand shape) illustrate active maintenance and remaining behavior gaps. PR
- #389 merged for the v1.0 release on 2026-09-21. Maintainer comments versus actions and full issue/PR review remain
- pending; activity alone is not a quality verdict.
- Core source quality, fuzzing and mutation evidence, complete dependency tree, all CI commands, release build
- provenance, and consumer-boundary execution: pending. A text search for `fuzz|proptest|quickcheck|mutation` in owned
- Rust source and `.github` found no hits; this does not prove absence of external fuzz campaigns. No third-party code
- executed in this evaluation.

## Validation, scores and decision status

- Execution manifest: none yet; no candidate build, test, install, macro expansion, or benchmark was run.
- Hard-gate exits: none established. Unknowns are pending, not failures.
- Finalists: none promoted after complete discovery and targeted audit. No equal-depth validation performed.
- Scoring, sensitivity, pros/cons ranking: not applicable yet. Do not infer a winner from publication or README claims.
- Current decision: no adoption authorization, no product changes, and no recommendation until discovery, audit, and
- consumer-boundary comparisons are complete.
- Next actions: finish registry and GitHub discovery, audit each serious alternative, inspect execution trees, run
- upstream and bounded throwaway consumer tests, compare both real kernels against existing behavior and noise-banded
- timing, then score and perform sensitivity analysis.

## Proposed forbidden-regex worktree trial, not an adoption decision

User asked on 2026-09-22 whether to try an isolated comparison and how to establish strict improvement.
A disposable worktree is suitable for the experiment; this question does not authorize product edits in the main
worktree.
The comparison below is a proposed protocol, not evidence of improved safety or performance.

### Frozen variants and integration boundary

- A: untouched incumbent from one pinned commit, with separate disposable baseline build outputs.
- B: use core `u8x64::load_array_ref` with a valid lower-tier x86 token or NEON token, convert to
- `__m512i`/`uint8x16x4_t`, and retain the existing CPU check, native 16-byte broadcast state, and raw lookup. Source
- supports this load-only design without constructing an Ice Lake token. It removes local pointer-load obligations but
- **not** the x86 call's target-feature obligation; code generation and actual speed remain unverified.
- C: independently test an algorithmic rewrite using `u8x64::swizzle_dyn`. It is not a drop-in substitution for the
- 64-byte table plus 16-byte output on NEON, nor for the narrow x86 dispatch tier. The optional `fearless_simd_macros`
- crate is unnecessary for the B or C design; audit its transitive surface only if later used.
- Upstream `fearless_simd/src/generated/avx512.rs:35-51` requires the full Ice Lake feature bundle for its proof token.
- Replacing the current x86 F/BW/VBMI gate with `Level::new().as_avx512()` **cannot** preserve accelerated-path
- eligibility for a CPU that has the current features but lacks other Ice Lake features. Do not create an `Avx512`
- token from that narrower CPU check. Retaining the old kernel for such hosts changes the comparison to a hybrid rather
- than a full replacement.
- Hold input corpus, compiler and flags, optimization settings, crate artifact checksums, host CPU, and package
- revisions fixed. B and C each get one change class; do not conflate a rewrite with a safe-intrinsics wrapper.

### Correctness and reachability gate

- Compare per-line verdicts and `(line, rule)` attribution plus CLI exit status, not just the total hit count measured
- by the existing `forbidden-regex.bench/README.md:21-27`. A security scanner cannot accept any new false negative or
- false positive.
- Exercise the public route in `src/engine.rs:318-329`: seeded versus seedless, table versus counting, batches just
- below and at `SHENG_BATCH_FLOOR` (512), one-byte Sheng, two-byte Sheng, scalar fallback, and host-feature branches.
- Existing `src/regex/batch_tests.rs:86-102` exercises the large seedless route, while the explicit hooks in
- `src/dfa/sheng_tests.rs` and `sheng2_tests.rs` can exercise qualifying and falling-back shapes. Verify which kernel
- actually ran rather than inferring it from a passing result.
- Differentially compare against the incumbent and an independent semantic oracle on adversarial inputs: empty lines,
- odd pair tails, matches ending at either byte position, acceptance/context boundaries, 16/64/65-state eligibility,
- out-of-range shuffle indices, long and short lines, and Unicode encoded as bytes. Fuzz and mutation controls must
- fail when the changed kernel deliberately returns a wrong verdict; a clean null result without this positive control
- is inconclusive.
- Compile/run on native supported CPU feature tiers, including arm64 NEON, x86 SSE2/AVX2, and an x86 AVX-512VBMI host;
- use bounded emulation for a narrow F/BW/VBMI-only CPU if native access is unavailable. Emulation establishes
- dispatch/correctness, not native throughput. Baseline-target compilation must not globally enable features absent on
- the emulated CPU.

### Performance and engineering gate

- Validate first that the microbenchmark observes a deliberate, known slower alternative. Measure the unchanged
- baseline's run-to-run spread before A/B comparisons. Interleave A/B runs on identical hardware and input, with
- resource bounds and no ambient credentials, to limit drift. Record distributions and confidence intervals; absence of
- a statistically detected slowdown alone is not equivalence.
- Use `package/rust-module/forbidden-regex.bench/src/kernels.rs:73-114` for explicit Sheng and Sheng2 microbenchmarks,
- including their table-build cost, then measure the real ruleset and caller boundary using
- `src/kernels.rs:131-145,201-237`. The latter spends time outside these kernels, so both measurements are necessary.
- Include per-feature-tier checks; an x86 dev host without AVX-512 cannot establish AVX-512 speed.
- Inspect the generated loop, not just the presence of `vpermb` or `vqtbl4q`. The upstream
- `generated/neon.rs:6383-6404` 64-byte-result implementation contains four table lookups; the incumbent
- `src/dfa/sheng.rs:326-343` uses one for its replicated 16-byte state. Unused-result elimination is a code-generation
- question; note surviving shuffles, spills, loads, code size, vector-width conversion, and fallback behavior.
- Freeze a zero-regression performance aim for each consumed native tier before looking at candidate timings, with an
- explicit **inconclusive** outcome whenever intervals cannot exclude a slowdown. A material advantage must be
- demonstrated in at least one predeclared dimension, such as measured throughput or a removed safety obligation,
- without a verified loss in correctness, platform coverage, other tested throughput, maintainability, or
- operational/dependency surface. Finite tests cannot prove universal dominance over all CPUs and inputs.
- A safety win means identifying exact feature, pointer, bounds, alignment, and target-feature proof obligations
- eliminated from this package, which remain, and which move into the new dependency. Count the audit burden of
- generated code and any transitive procedural macros actually used. Less `unsafe` text by itself does not prove
- greater safety.
- Even if the trial passes, upstream source, issue/PR, security, provenance, platform, and consumer-boundary gates in
- this report remain prerequisites to recommending adoption. The experiment is not permission to merge the trial.

## Source-only resolution after the user's correction

The user correctly challenged the implication that all remaining questions required a worktree.
At pinned tag `fearless_simd-v1.0.0` (`dcbd824a897a098ec0124c088f971ac6d890d0cd`), the following conclusions are
available without building either crate.

- **Full Ice Lake proof versus narrow gate:** `fearless_simd/src/lib.rs:286-369` checks features including VBMI2, VAES
- and SHA in addition to F/BW/VBMI; `src/generated/avx512.rs:35-51` gives the token the same full requirement;
- `src/kernel_macros.rs:152-202,340-383` accepts only fixed proof types and attaches that full feature set. In contrast
- `package/rust-module/forbidden-regex/src/dfa/sheng.rs:226-240` and `sheng2.rs:251-266` require only F/BW/VBMI. The
- drop-in `as_avx512` rewrite is ruled out as an equivalent dispatch replacement, not proven slower on any measured
- host.
- **Safe-load-only candidate exists:** `fearless_simd/src/generated/simd_types.rs:7990-8001,8081-8083` creates
- `u8x64<S>` from an exactly 64-byte array for any valid `S: Simd`, using checked transmutation;
- `src/generated/simd_trait.rs:9652-9654` exposes `load_array_ref`. Native conversions
- `src/generated/avx512.rs:17687-17700` and `src/generated/neon.rs:7644-7657` also accept any `S: Simd`. The existing
- table entries are `[u8; 64]` (`package/rust-module/forbidden-regex/src/dfa/sheng.rs:99-117,173-202`,
- `sheng2.rs:110-138,202-237`). `fearless_simd/src/lib.rs:555-580` safely supplies an SSE2 token even when the detected
- level is stronger. A safe table load with this token can feed an `__m512i` inside the existing narrowly guarded
- intrinsic function; the analogous NEON load feeds `uint8x16x4_t`. This would move the byte-load safety obligation to
- an external dependency that still requires a full audit, while retaining current dispatch and state width. It does
- not show identical emitted loads or eliminate the unsafe x86 call.
- **Shape of portable shuffle:** `fearless_simd/src/generated/simd_trait.rs:510-513,6303-6306` requires same-width
- table, index and result for `u8x16` or `u8x64`. It has no 64-byte-input/16-byte-output primitive.
- `src/generated/avx512.rs:13009-13021` uses `_mm512_permutexvar_epi8`, unlike the historical `std::simd` scalarization
- in `package/rust-module/forbidden-regex/OPTIMIZATIONS.md:343-352`. `src/generated/neon.rs:6383-6404` calculates four
- 16-byte outputs; both current NEON kernels maintain only a replicated 16-byte state (`sheng.rs:330-343`,
- `sheng2.rs:369-388`). A changed-width algorithm or native `kernel!` is required to express the old NEON state/output
- shape.
- **Out-of-range semantics are not a blocker for validated DFAs:** the fast x86 shuffle may wrap indices and the NEON
- variant zeroes them (`src/generated/avx512.rs:13009-13021`, `generated/neon.rs:6383-6404`). Validated `Dfa` instances
- satisfy start and transition target `< num_states` in `package/rust-module/forbidden-regex/src/dfa/table.rs:314-344`;
- `Regex::from_bytes` validates before use (`src/regex.rs:211-222`), and both Sheng table builders require at most 64
- states (`src/dfa/sheng.rs:173-202`, `sheng2.rs:202-237`). This does not make unvalidated `Dfa::from_parts` data valid
- by construction.
- **What `kernel!` can and cannot remove:** `fearless_simd/src/kernel_macros.rs:4-21,68-99,340-383` safely calls a
- target-feature-scoped inner function when given a genuine token; pointer intrinsics with extra requirements still
- need local `unsafe`. The x86 `Avx512` variant cannot replace the existing narrow gate without changing eligibility.
- Safe table loading is a separate possible improvement; counting `unsafe` keywords is not the safety proof.
- **Proc macro cost:** core `fearless_simd/Cargo.toml:23-43` has only optional `libm` and defaults to `std`.
- `fearless_simd_macros/Cargo.toml:14-23` brings `syn`, `quote` and `proc-macro2` only if separately added; `kernel!`
- is defined in core `src/kernel_macros.rs:68-99`.

**Remaining empirical questions:** whether the load-only design compiles against the published 1.0 artifact and
preserves the expected machine code, its throughput and code size on actual x86 and arm64 targets, and whether the
operational safety benefit outweighs a new dependency. These cannot be inferred from method names or source structure
alone. A full portable rewrite need not be built merely to rediscover the known token and operand-shape constraints;
retain it only as a separate changed-algorithm experiment if it promises an independently useful benefit.
