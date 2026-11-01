# Implementation plan: strengthen fuzzing coverage for module-toml-edit

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/file-enforcer` is now `package/file-enforcer/core`;
> `package/module/toml-edit.fuzz` is now `package/toml-edit/core.fuzz`;
> `package/module/toml-edit` is now `package/toml-edit/core`;
> `@monochromatic-dev/module-toml-edit` is now `@monochromatic-dev/toml-edit-core`;
> `@monochromatic-dev/module-logger` is now `@monochromatic-dev/logger-core`;
> `@monochromatic-dev/module-test` is now `@monochromatic-dev/test-harness-core`.

> Sidecar note:
> on 2026-09-06 the campaign moved from `package/module/toml-edit/src/fuzz/` into this sidecar package,
> `package/module/toml-edit.fuzz`,
>  following the `logger.fuzz` layout;
> `src/fuzz/` paths below now read as this package's `src/`,
> and the `fuzz` and `fuzz:coverage` tasks run as
> `//package/module/toml-edit.fuzz:fuzz`
> and `//package/module/toml-edit.fuzz:fuzz:coverage`.

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

This is the source-of-truth plan for the toml-edit fuzzing work.
 It replaces the
brief-only version with resolved sequencing,
 explicit oracle design,
 and pass
criteria for each layer.

## Implementation status

Updated 2026-06-12.
 Complete:
 phases 1 to 6 and 8 are landed,
 phase 7
(differential oracle) was attempted against the BurntSushi reference decoder and
then dropped (see below),
 and phase 9 is done.
 The `toml-edit-fuzz` CI workflow
is green end to end (run 27390878270:
 build,
 type-check,
 unit suite,
 fuzz smoke,
conformance,
 and the coverage gate),
 the path filter is proven (a push under
`package/module/toml-edit/**` triggered it),
 and issue #198 is closed.
 Getting
CI green required four workflow fixes the prior session's untested workflow had
masked:
 `mise-action` `install: false`,
 a job-level `MISE_AUTO_INSTALL=false`
(the root config auto-installs the slint git crate,
 which fails on the runner)
with node,
 pnpm,
 and toml-test installed explicitly,
 building the bundle before
type-check,
 and importing `module-logger` via `/ts` source so no logger build is
needed.
 Most work lives under `package/module/toml-edit/src/fuzz/` and
`src/conformance/`,
plus package-source fixes (`src/parse-toml-edit.ts`,
 `src/emit-value-string.ts`,
the shared `src/basic-escape.ts`,
 `src/value-encoders.ts`,
 `src/keys.ts`),
 the
seam exports in `src/index.ts`,
 the decision doc
`doc/decision/toml-edit-fuzzing.md`,
 and the workflow
`.github/workflows/toml-edit-fuzz.yml`.

Six real bugs found:
 four fixed in scope (the `RangeError` parse contract,
 the
parsed-node basic-string control-character escaping,
 the same gap on the
from-scratch value and key encoders,
 and the bare carriage-return parser
laxity),
 and two deeper edit-machinery defects deferred to #252 (repeated
path-create duplicate key,
 implicit-parent delete read/byte mismatch).
 Phase 6
also changed the newline policy:
 `CRLF` normalizes to `LF` on parse with a
warning (suppressible via `MONOCHROMATIC_WARN=false`,
 a mechanism added to
`@monochromatic-dev/module-logger`),
 and a bare `CR` is rejected.
 A follow-up
landed full `LF` symmetry:
 the canonical builder's `lineBreak` output option was
dropped,
 so the package is `LF`-only end to end (commit 7c915f16).

### Phase 7 dropped: BurntSushi proved unstable

The phase 7 differential oracle (decode the same documents through our decoder
and the BurntSushi Go `toml-test-decoder`,
 then compare) was built and run.
 It
worked:
 it immediately found a real divergence,
 which minimized to a defect in
the **reference**,
 not ours.
 BurntSushi v1.6.0's tagged decoder output loses data
on an empty key (`""`) near array structure:
 `[ { "" = 1 }, "z" ]` drops `"z"`,
`[ [ { "" = 1 } ] ]` collapses a nesting level,
 and `"" = [ [ {} ] ]` diverges.
Our parser is correct (no TOML reading drops `"z"`);
 a strip-and-recheck proof
confirmed empty-key was the sole cause.
 Decision (owner):
 drop BurntSushi
entirely rather than fight a growing carve-out against an unstable oracle (empty
keys appear in ~60 percent of generated docs;
 the bug has several shapes).
 The
differential adapters,
 the `go:` tool pin,
 and the `test:differential` task were
removed.
 Phase 6 conformance (both TOML versions,
 zero failures,
 no allow-list)
remains the parser-correctness oracle.
 A differential oracle against a different,
stable reference is possible future work.
 Full detail in the decision doc.

### Convention: naming an expensive fuzz/property test

A property test that is genuinely a unit test but too expensive for the default
suite (spawns external processes,
 heavy I/O) must be named
`*.expensive.unit.test.ts`.
 The `*.unit.test.ts` suffix gives it the test-file
lint relaxations (arrow callbacks,
 `require-await`,
 `require-tsdoc` on locals) and
normal harness discovery;
 the `.expensive.` segment excludes it from the default
`test:unit` run (only `--all` includes it),
 so routine CI never pays its cost.
 Do
not name such a file `*.property.ts` (it then falls under full production lint
rules) nor plain `*.property.unit.test.ts` (it then runs in the default suite and
CI).
 The everyday fuzz properties stay `*.property.unit.test.ts`;
 only the
external-dependency ones take `.expensive.`.

### Landed

Phase 1 (commit cd727e97),
 campaign scaffold:

- `src/fuzz-budget.ts`:
   `TOML_EDIT_FUZZ_BUDGET_MS`,
   `fuzzRunPlan`,
   and
  `isCampaignMode`.
- `fuzz` task in `mise.toml`:
   builds the bundle first,
   then runs every
  `src/**/*.property.unit.test.ts` under node at a per-property budget (default
  60000),
   so a stale dist never hides a fix.
- `src/fuzz/smoke.property.unit.test.ts`:
   imports the built package entry point
  and proves both bounded and campaign mode run.
- Also fixed two pre-existing low-information symbol descriptions in
  `src/effective-value.ts` so the package lints clean.

Phase 2 (commit 05a039d6),
 generators and oracle:

- `src/fuzz/equality.ts`:
   `semanticModel` (the `getStaticTOMLValue` projection)
  and `semanticEquals` (an iterative work-stack deep-equal handling nan,
   signed
  infinities,
   minus zero,
   `Date` by instant,
   and key order).
- `src/fuzz/escape.ts`:
   an independent basic-string and literal-string escaper,
  so generator string encoding never reuses the emitter under test.
- `src/fuzz/arb-*.ts`:
   grammar arbitraries for every scalar,
   key,
   compound,
   and
  document shape,
   split across files to stay under max-lines.
- `src/fuzz/mutators.ts`:
   structure-aware corruption mutators plus
  `corruptedDocumentArbitrary`.
- `src/fuzz/corpus.ts`:
   committed-fixture loaders plus campaign-only live
  repository discovery.
- `src/fuzz/generators.property.unit.test.ts`:
   proves every generator parses and
  means what it predicts.
   It caught a real generator bug,
   where a bare key
  emitted after a table header is captured by that table;
   the fix emits all
  key-value blocks before any sectioned block.

Phase 3 (commits 3a298325 and da3a8c59),
 parser and round-trip:

- `src/fuzz/parse-toml-edit.property.unit.test.ts`:
   totality,
   valid acceptance,
  invalid rejection,
   the TOML 1.0 versus 1.1 inline-table-newline split,
   and a
  pinned deep-nesting regression.
- `src/fuzz/round-trip.property.unit.test.ts`:
   splice byte identity,
   canonical
  semantic round-trip,
   and metamorphic comment and blank-line invariance.
- Real bug found and fixed in `src/parse-toml-edit.ts`:
   pathologically deep `[`
  or `{` nesting made the underlying parser throw a raw `RangeError` (stack
  overflow) that escaped the `ParseError`-only catch,
   breaking the documented
  `@throws TomlEditError` contract.
   `safeParse` now wraps every non-`ParseError`
  throw,
   preserving the cause.
   The totality oracle was then sharpened to assert
  the wrapped cause is a `ParseError` or a `RangeError`,
   so a future input that
  makes the parser throw a different class still surfaces rather than being
  silently wrapped.

Phase 4 (commits 363945e5 and 98443361),
 emitter and seams:

- Internal encoders and emitters re-exported from `src/index.ts` as unstable
  `_`-prefixed seams (`_encodeKey`,
   `_jsValueToTomlText`,
   `_emitContentNode`,
  `_emitStringValue`,
   `_spliceEmit`),
   documented in TSDoc,
   the README,
   and the
  decision doc.
- `src/fuzz/emit.property.unit.test.ts`:
   key round-trip,
   value round-trip,
  value-kind preservation across re-emission (the load-bearing datetime check),
  string re-emission,
   and splice byte identity.
- Second real bug found and fixed in `src/emit-value-string.ts`:
   the basic-string
  escaper emitted control scalars (NUL,
   U+001F,
   U+007F) raw,
   producing invalid
  TOML;
   now every control scalar below U+0020 plus U+007F is escaped as `\uXXXX`,
  single-line and multiline.

Phase 5 (commit 974207b9),
 stateful edit model:

- `src/fuzz/stateful.property.unit.test.ts`:
   random `tomlSet` / `tomlDelete`
  sequences over `emptyTomlEdit`,
   materialized by reparsing between operations,
  checked against an in-memory model;
   plus idempotence on the reparsed result.
- Surfaced two deeper edit-machinery defects now tracked in #252 (repeated
  path-create duplicate key,
   implicit-parent delete read/byte mismatch).
   The
  property stays on single top-level segments to avoid those edges.

Phase 6 (commits da1d3f7c,
 2f42cf02,
 bd94b330,
 5e8ae6d9,
 36b18731,
 plus
`@monochromatic-dev/module-logger` fa785bfb),
 toml-test conformance:

- `src/conformance/`:
   decode and encode node adapters satisfying the upstream
  runner's interfaces,
   with the kind-aware tagged-JSON model deferred from
  phase 2 (`decode-leaf.ts`,
   `decode-to-tagged.ts`,
   `encode-from-tagged.ts`).
- `test:conformance` task:
   follows the latest `toml-test` through mise's
  `github:` backend (attestation and SLSA provenance verified on install,
  version logged),
   runs decoder and encoder for TOML 1.0 and 1.1 under
  `MONOCHROMATIC_WARN=false`,
   and fails on any non-zero runner exit.
   Both versions pass every
  valid,
   encoder,
   and invalid case with no allow-list.
- Surfaced and fixed two more real bugs (the from-scratch value and key encoders
  emitted control scalars raw;
   the parser accepted a bare carriage return) and
  changed the newline policy (`CRLF` normalizes to `LF` with a warning,
   bare `CR`
  rejected).
   See the decision doc.

Phase 9 CI smoke (commit 7c5abf2a):

- `.github/workflows/toml-edit-fuzz.yml`:
   path-filtered to the package,
   its
  fixtures,
   and the decision doc;
   type-check,
   build,
   the bounded unit suite,
   then
  a short per-property fuzz campaign.
   Not yet proven by a real PR run.

Phase 8 (commit b376c3b6),
 V8 coverage gate:

- A deterministic reachability driver (`src/fuzz/coverage-driver.ts`) imports the
  package source (not the bundle),
   replays the generators and committed corpus
  through every entry point and `_` seam at a fixed seed,
   and runs under
  `NODE_V8_COVERAGE`.
   The reader (`src/fuzz/coverage-v8.ts`) projects block ranges
  to per-file covered lines;
   the gate (`src/fuzz/coverage-report.ts`) fails on any
  per-file regression from `coverage-baseline.json`.
   Operation spread,
   harness,
  and edit machinery are split across `coverage-exercise.ts`,
  `coverage-harness.ts`,
   `coverage-edits.ts`,
   and `coverage-probes.ts` for
  max-lines.
   New `fuzz:coverage` task (`--write` refreezes);
   no build needed.
- The driver was validated against the real property suite (a throwaway
  source-remap `--import` hook):
   it covers a superset of the suite,
   and the
  covered set is saturated across seeds and run counts (baseline 5287 lines,
   39
  files).
   The reusable five-question fuzz-target checklist lives in the decision
  doc.

### Decisions and deviations from the original plan

- The semantic oracle is `getStaticTOMLValue`-based,
   not the toml-test
  tagged-JSON model the plan placed in phase 2.
   Both sides of every comparison
  pass through the same projection,
   so its lossiness (datetime kinds collapse to
  a host-zone `Date`,
   integers past 2^53 lose precision) is symmetric and safe
  for round-trip and metamorphic equality.
   The kind-aware tagged-JSON converter
  is deferred to phase 6,
   where the toml-test decoder needs it and an external
  oracle makes the datetime-kind distinction necessary.
- Property files import the built package entry point
  (`@monochromatic-dev/module-toml-edit`),
   per the plan's import-boundary
  section,
   so the `fuzz` task and any campaign build first.
- Live corpus discovery excludes curated fixture sets (`test-fixture`,
  `toml-test`,
   `/fixtures/`),
   which deliberately hold invalid and
  version-specific inputs.
   The round-trip oracles additionally keep only sources
  the package parses,
   so a parse failure classifies corpus rather than failing a
  valid-requiring property.

### Carry-forward for phase 4 and later

- The `getStaticTOMLValue` oracle is blind to an emitter that changes a
  datetime's kind or spelling (a local date and an offset datetime can project
  to the same instant).
   Phase 4's `emitContentNode` spelling-preservation
  property must therefore assert datetime spelling directly;
   it is load-bearing,
  not redundant with the round-trip oracle.
- The round-trip valid-corpus filter (`parsesUnderDefault`) excludes TOML
  1.1-only constructs (unicode bare keys and similar) from canonical
  round-tripping.
   Phases 4 and 5 should parameterize `tomlVersion` so 1.1 shapes
  round-trip too.
   The 1.0 versus 1.1 difference itself is already asserted in the
  parser property.

### Remaining

- Phase 7 (differential oracle):
   dropped.
   BurntSushi v1.6.0 proved unstable (the
  empty-key data-loss bug above).
   Not pursued further against that reference;
   a
  different stable reference could revive it later.
- Phase 8 (V8 coverage gate and the reusable five-question checklist):
   landed in
  commit b376c3b6.
   See the Phase 8 entry above and the decision doc.
- Phase 9:
   done.
   `test:conformance` and `fuzz:coverage` are steps in
  `.github/workflows/toml-edit-fuzz.yml` (commit e0676ba5);
   the workflow's mise
  setup,
   build order,
   and the `module-logger` import were fixed (commits
  510c1256,
   e6b4f25a) so the run is green;
   the path filter is proven;
   and #198 is
  closed.
   The Scorecard FuzzingID alert (code-scanning/17) auto-closes on the next
  Scorecard scan,
   which is the scanner's own action.

## Evidence checked

This plan was sharpened against current repository state,
 not written from memory:

- `package/dev-script/file-enforcer/mise.toml` and
  `package/dev-script/file-enforcer/src/fuzz-budget.ts` show the existing
  env-parameterized property-test campaign pattern.
- `package/dev-script/file-enforcer/src/pipeline/toml.property.unit.test.ts`
  shows the weak TOML wrapper properties this plan must not copy as the whole
  target.
- `package/module/toml-edit/src/index.ts` lists the public API surface to cover.
- `package/module/toml-edit/src/fixtures.unit.test.ts` and
  `package/test-fixture/toml-edit/src/` already provide 91 valid and 108 invalid
  TOML fixture files.
- GitHub issue #198 is still open and asks for a bounded toml-edit fuzz task,
   a
  committed seed corpus entry,
   CI wiring for toml-edit changes,
   and explicit issue
  closure.
- `toml-test` was inspected at `/tmp/agent/toml-test-20260606` commit
  `af5f8052e9109206ad3977508263c97907f0797d`;
   its README documents the valid and
  invalid corpus split,
   tagged JSON format,
   TOML 1.0 and 1.1 file lists,
   and the
  option to reimplement the runner inside another language's test suite.
   The
  implementation still needs to pin a release or commit deliberately;
   the scratch
  clone path is research evidence only.
- A local node v26.3.0 smoke run with `NODE_V8_COVERAGE` produced V8 JSON containing
  covered and uncovered source ranges,
   so the coverage gate can start from raw V8
  coverage without first adding a coverage dependency.

No root `CONTEXT.md` or `CONTEXT-MAP.md` exists.
 No glossary file was created,
because the terms here are TOML and fuzzing terms,
 not project-domain language.

## Core diagnosis

The file-enforcer TOML properties were green but weak because every factor in
this product was too small:

```txt
reachable(generator)  x  detectable(oracle)  x  present(target layer)
```

- `present(target layer)` was small because the tests hit file-enforcer wrappers,
  not `@monochromatic-dev/module-toml-edit`,
   where the parser,
   editor,
   splicer,
  and emitter logic live.
- `detectable(oracle)` was small because “returns a string or throws
  `TomlEditError`” cannot catch silent value corruption.
- `reachable(generator)` was small because the generators mostly produced strings,
  single bare keys,
   and empty base documents.
- There was no measurement layer,
   so a green run said nothing about grammar reach,
  branch reach,
   or oracle strength.

The implementation must strengthen all three factors and must add feedback that
measures whether the suite is still strong.

## Goal

Improve fuzzing coverage of `@monochromatic-dev/module-toml-edit` and install a
repeatable method so future fuzzing cannot ship green-but-weak again.
 The work
should close issue #198 after implementation and verification.

The goal is not to request a hidden bug list.
 If the new properties,
 conformance
runs,
 or differential checks discover real toml-edit bugs,
 fix every surfaced bug
in this scope,
 then pin each counterexample as a permanent regression example.
 The
original “not fixing known bugs” rule means “do not depend on a supplied bug
list,
” not “defer bugs that the suite finds.
”

Related issue boundaries:

- #198 is the issue this plan should close.
   Its original acceptance criteria ask
  for smoke fuzzing,
   CI,
   and Scorecard FuzzingID closure.
   This plan optimizes for
  real bug-finding first and treats the Scorecard result as a post-implementation
  scanner check,
   not as the design target.
   The stronger coverage gate exists
  because the wrapper-only fuzzing failure proved smoke fuzzing is not enough.
- #165 tracks broader editor completeness and workspace adoption.
   Do not widen this
  fuzzing plan into replacing every TOML edit caller.
- #244 tracks sentinel conversion in file-enforcer wrappers.
   It can add wrapper
  regressions,
   but it does not replace direct toml-edit fuzzing.

## Non-goals

- Do not add mutation-testing tooling in this plan.
   Mutation testing remains the
  intended follow-up for measuring oracle strength more directly,
   and this work
  should create or update a follow-up issue for it.
- Fuzzing may widen `package/module/toml-edit/src/index.ts` with underscored seam
  exports such as `_...` helpers when that is the cleanest way to exercise the
  built artifact.
   These exports are explicitly unstable and carry no compatibility
  promise.
- Do not use the file-enforcer wrapper properties as proof that toml-edit itself
  is covered.
   They remain wrapper coverage only.
- Do not create an `AGENTS.md` pointer or global agent rule unless the user asks.

## Constraints

- Runtime is node.
   node v26 runs `.ts` files directly,
   the harness exits non-zero
  on failure,
   and campaign tasks must invoke node,
   not bun.
- Tests use `@monochromatic-dev/module-test/ts` and `fast-check`.
- Normal unit runs stay bounded.
   Campaign runs use a per-property time budget.
- Test files remain subject to `invocation-depth-per-line`,
   `no-mixed-operators`,
  and numeric-separator rules.
- New third-party parser dependencies require the `choosing-technology` skill's
  source audit before selection.
- The `toml-test` corpus is data,
   but this plan deliberately follows the latest
  upstream runner through mise rather than pinning it.
   Every conformance run must
  log the runner version and release asset digest so failures can be traced to an
  upstream corpus change or a local code change.
- Reference source files by repo-relative path.
   Keep prose free of em dashes.

## Resolved planning decisions

### Scope relative to issue #198

Optimize for preventing another green-but-weak fuzz suite,
 not merely for closing
issue #198's original smoke-fuzz checklist.
 The smoke fuzzer and CI step remain
required,
 but they are the floor below grammar-complete generators,
 stronger
oracles,
 coverage feedback,
 and the reusable checklist.

### Campaign shape

Use the file-enforcer pattern,
 but package-local:

- Add `package/module/toml-edit/src/fuzz-budget.ts` with
  `TOML_EDIT_FUZZ_BUDGET_MS`.
- Add a `fuzz` task to `package/module/toml-edit/mise.toml`,
   mirroring
  file-enforcer's task,
   with `--budget <ms>` defaulting to `60000`.
- Keep the same `*.property.unit.test.ts` files in both normal and campaign modes.
- Co-locate property files beside the source seam they exercise.
- Put shared arbitraries,
   semantic-normalization helpers,
   and corpus loaders under
  `package/module/toml-edit/src/fuzz/` so individual test files do not exceed the
  max-lines budget.

### Import boundary

Use built-artifact imports for fuzzing:

- Property files should exercise `@monochromatic-dev/module-toml-edit` through the
  built package entry point,
   not sibling source imports.
- If an internal seam needs direct fuzzing,
   export it from `src/index.ts` with an
  underscored name.
   The underscore marks an unstable seam export that exists for
  observability and fuzzing,
   not a compatibility promise.
- Do not skip seam fuzzing to avoid API growth;
   expose the seam with a `_` export
  when the seam matters.
- Document `_` seam exports in three places:
   TSDoc at each export declaration,
  `package/module/toml-edit/README.md`,
   and
  `doc/decision/toml-edit-fuzzing.md`.

Document this choice in `doc/decision/toml-edit-fuzzing.md`,
 because it is a
surprising deviation from the previous plan and from hiding internals.

### Corpus source

Use three corpus tiers:

1. Existing fixture package:
   `package/test-fixture/toml-edit/src/valid/` and
   `package/test-fixture/toml-edit/src/invalid/`.
2. Real repository TOML files in two forms:
    reviewed snapshot seeds for normal
   tests,
    plus dynamic discovery during campaigns.
    Both exclude generated,
   dependency,
    build-output,
    and secret-looking paths.
3. `toml-test` valid and invalid files,
    driven through the latest upstream runner
   acquired by mise.

The existing fixture package remains the right home for reusable TOML corpus data
that is useful without the upstream runner.
 Do not vendor the toml-test corpus as
this plan's primary integration path.
 Instead,
 add package-local decoder and
encoder adapter commands that the upstream toml-test runner can invoke.
 BurntSushi
TOML is the default differential oracle after conformance is wired;
 the decision
doc still must record source-audit evidence and rejected alternatives.
 Keep one-off shrunk
counterexamples in the owning property via fast-check `examples`.

### External oracle order

Bring up the external oracles in this order:

1. `toml-test` conformance first,
    through a dedicated conformance task using the
   upstream runner acquired by mise.
    It already specifies tagged JSON comparison
   semantics for integers,
    floats,
    booleans,
    datetimes,
    arrays,
    tables,
    and
   invalid inputs.
2. Differential parser second,
    after the source audit.
    Compare only normalized semantic values,
   not raw formatting.
3. Mutation testing later,
    in a separate plan.

Default differential parser:
 BurntSushi TOML,
 through its `toml-test-decoder` and
`toml-test-encoder` command tools.
 Acquire both Go and the BurntSushi tools through
mise's Go support without pinning either version.
 The decision-doc record must call out this
moving-oracle trade-off and log the concrete tool versions used in every campaign
run so later disagreements can be reproduced.
 Initial source evidence already
found those commands,
 an internal toml-test runner,
 Go CI across Linux,
 macOS,
Windows,
 and OSS-Fuzz CIFuzz wiring.
 The implementation still must compare and
reject at least `smol-toml`,
 `@iarna/toml`,
 `@ltd/j-toml`,
 and any current npm TOML
parser that appears from a fresh search;
 record the chosen path and rejected
alternatives in the decision doc.

### Coverage gate

Use raw V8 coverage first:

- Add a node-driven coverage campaign that runs the property files with
  `NODE_V8_COVERAGE` pointed at a temporary output directory.
- Summarize coverage for target files under `package/module/toml-edit/src/`,
  especially `parse-toml-edit.ts`,
   `emit-value.ts`,
   `emit-value-string.ts`,
  `splice.ts`,
   `comments.ts`,
   `toml-set-aot.ts`,
   `path-create.ts`,
  `toml-set.ts`,
   `toml-delete.ts`,
   `effective-value.ts`,
   and `resolve.ts`.
- Treat V8 uncovered ranges as a reachability signal.
   If a branch-like range in
  parser,
   emitter,
   comments,
   AOT,
   splice,
   or path-create code is untouched,
   add a
  generator case or a seed before calling the campaign strong.
- Gate on no regression from a committed baseline after the first complete
  campaign lands.
   Do not set an arbitrary percentage threshold before the baseline
  exists.
- If raw V8 ranges cannot give enough human-readable signal,
   vet a coverage
  summarizer as a separate dependency decision rather than silently adding one.

Call the task `fuzz:coverage` or make `fuzz --coverage` available.
 The task should
produce a text summary suitable for CI logs and a machine-readable baseline file
under the package,
 not under a generated output directory.

### Reusable checklist

Record the reusable fuzz-target checklist as a section in
`doc/decision/toml-edit-fuzzing.md` first.
 Do not create a project-local
`fuzzing` skill in this plan.
 A skill becomes worthwhile after the checklist is
used on at least one more target and the wording proves stable.

## Implementation phases

### Phase 1: Scaffold the campaign

Deliverables:

- `package/module/toml-edit/src/fuzz-budget.ts`.
- `package/module/toml-edit/mise.toml` `fuzz` task.
- One smoke property file that imports from the built package entry point and
  proves bounded mode and campaign mode both run under node.
- A fast-check seed and counterexample policy copied from the file-enforcer
  decision:
   random seeds in normal and campaign modes,
   pinned `examples` for every
  discovered counterexample.

Pass criteria:

- `mise run //package/module/toml-edit:test:unit` still passes.
- `mise run //package/module/toml-edit:fuzz --budget 1000` exits cleanly.

### Phase 2: Build semantic helpers and generators

Deliverables:

- A normalized TOML value model matching the `toml-test` tagged JSON shape.
- Equality helpers that handle `nan`,
   signed infinities,
   integer spelling,
   float
  equivalence,
   boolean case,
   datetime equivalence,
   arrays,
   inline tables,
   standard
  tables,
   and array-of-tables.
- Grammar-focused arbitraries for:
  - basic,
     literal,
     multiline-basic,
     and multiline-literal strings;
  - integers in decimal,
     hex,
     octal,
     binary,
     signed,
     and underscore forms;
  - floats with decimal points,
     exponents,
     `inf`,
     `-inf`,
     `nan`,
     and `-nan`;
  - booleans;
  - offset datetimes,
     local datetimes,
     local dates,
     and local times;
  - arrays,
     nested arrays,
     inline tables,
     standard tables,
     nested tables,
     and
    array-of-tables;
  - bare,
     quoted,
     empty,
     dotted,
     unicode,
     escaped,
     numeric-looking,
     and
    float-looking keys;
  - comments before keys,
     same-line trailing comments,
     header comments,
     blank
    lines,
     and CRLF variants;
  - duplicate keys,
     table collisions,
     path-create-through-scalar cases,
     deep
    nesting,
     huge strings,
     and unicode edge cases.
- Structure-aware mutators for valid corpus inputs,
   so invalid and near-invalid
  documents are not just arbitrary byte noise.
- A split repo-corpus loader:
   deterministic snapshot seeds in bounded unit mode,
  dynamic discovery of current repo TOML only in campaign mode.

Pass criteria:

- Every generator has at least one deterministic `examples` value for each grammar
  family above.
- The generator file set stays below max-lines by splitting helpers rather than
  compressing code.

### Phase 3: Parser and stringify properties

Properties:

- Arbitrary text either parses to a state or throws `TomlEditError`,
   never a raw
  `ParseError` or unrelated exception.
- Valid generated documents parse successfully.
- Invalid generated and corpus documents reject with `TomlEditError`.
- Splice mode with no deltas is byte-identical for every accepted source.
- `parseTomlEdit` then `tomlStringify` then `parseTomlEdit` preserves the
  normalized semantic model.
- `emptyTomlEdit` plus setters emits TOML that reparses and matches the intended
  model.
- `tomlVersion` is part of the generated case.
   TOML 1.0 and 1.1 differences are
  asserted explicitly rather than treated as flake.
- Aggressive metamorphic transforms reorder keys,
   reflow whitespace,
   add or strip
  comments,
   perturb table placement,
   and introduce near-collision shapes.
   Their
  oracle classifies each transformed document as same semantics,
   changed semantics,
  or invalid output instead of assuming every rewrite preserves meaning.

Pass criteria:

- The parser properties cover both current fixture-package files and generated
  grammar cases.
- A failing parse on a supposedly valid generator case shrinks to a useful example
  and is added to the property examples before the fix lands.

### Phase 4: Emitter and seam properties

Properties:

- `encodeKey` output reparses as one key segment and never creates an unintended
  dotted path.
- `jsValueToTomlText` output reparses under a synthetic key for every accepted JS
  and wrapped TOML input.
- `emitContentNode` preserves parse-time spelling for unchanged strings,
   numbers,
  floats,
   booleans,
   datetimes,
   arrays,
   and inline tables when the AST carries that
  spelling.
- `emitStringValue` covers quote styles,
   multiline trimming,
   escapes,
   backslashes,
  control characters,
   unicode,
   and source-escaped variants.
- `spliceEmit` preserves untouched byte ranges and only replaces the ranges marked
  by edits,
   insertions,
   deletions,
   or header-comment changes.
- Comment helpers preserve the documented distinction between attached preceding
  comments,
   blank-line-separated comments,
   same-line trailing comments,
   and header
  comments.

Pass criteria:

- Every seam named in the target package reference section has at least one
  property file or a named property block.
- Coverage output shows each seam file is exercised by the campaign.

### Phase 5: Stateful edit model

Model commands:

- Start from either `parseTomlEdit({ source })` over generated or corpus TOML,
   or
  `emptyTomlEdit()`.
- Set existing scalar,
   array element,
   inline-table entry,
   table body,
   and
  array-of-tables collection.
- Path-create under top-level,
   under a standard table,
   and inside an inline table.
- Delete key-values,
   table blocks,
   array elements at multiple depths,
   and
  array-of-tables collections.
- Read with `tomlGet`,
   `tomlGetValue`,
   `tomlHas`,
   `tomlKeys`,
   `tomlGetRaw`,
  `tomlGetNode`,
   and comment accessors after each mutation.
- Insert comments before and after keys,
   and set header comments.

Oracles:

- The in-memory model predicts effective reads after every command.
- `tomlStringify` output reparses after every command sequence unless the command
  is expected to throw.
- Repeating an identical set or delete is idempotent when the API contract says it
  should be.
- Operations that should reject do so with the documented `TomlEditError` subclass.
- Parse-time views (`tomlGetNode` and `tomlGetRaw`) stay parse-time views after
  deltas;
   delta-aware reads reflect pending edits.

Pass criteria:

- The command generator includes successful and rejecting paths.
- The model covers AOT,
   inline tables,
   comments,
   cross-path effective reads,
   and
  dotted-key collisions.

### Phase 6: toml-test conformance

Deliverables:

- A mise-managed acquisition path for the upstream toml-test runner using the
  `github:` backend for `toml-lang/toml-test` at `latest`.
   The implementation must
  smoke-test the `asset_pattern` and `.gz` handling before relying on it,
   and
  record the resolved release,
   asset digest,
   and any provenance verification in the
  decision doc and campaign logs.
- Package-local node adapter commands that satisfy the upstream runner's decoder
  and encoder interfaces:
   TOML on stdin to tagged JSON on stdout for decode,
  tagged JSON on stdin to TOML on stdout for encode,
   and non-zero exit on
  rejection.
- A dedicated `toml-test` or `test:conformance` mise task.
   Do not fold the binary
  into the normal unit suite.
   The task should invoke the mise-installed
  `toml-test` runner,
   not a checked-in binary or ad hoc downloader.
- Valid and invalid conformance runs for TOML 1.0 and 1.1,
   using the upstream
  binary's own selection mechanism rather than hand-copying all corpus files.
- Encoder-style conformance where tagged JSON is converted through toml-edit
  setters,
   stringified,
   reparsed by the runner,
   and compared semantically.

Pass criteria:

- The conformance task exits non-zero on any valid,
   invalid,
   decoder,
   or encoder
  failure.
- Legitimate TOML 1.0 versus TOML 1.1 differences are named and tested under the
  right `tomlVersion`.
- Unsupported encoder cases are fixed in this scope unless a narrow allow-list is
  justified.
   Every allow-list entry must name the exact corpus case,
   the toml-edit
  contract gap,
   and the follow-up issue if it represents missing package behavior.

### Phase 7: Differential parser oracle

Deliverables:

- Source-audit record in `doc/decision/toml-edit-fuzzing.md` for the selected parser and
  rejected alternatives.
- A semantic normalizer for the selected parser's output.
- A disagreement classifier with three outcomes:
  - toml-edit wrong;
  - reference parser wrong or less capable;
  - spec ambiguity or intentional version difference.
- A stable allow-list for intentional disagreements,
   with each entry tied to a
  source path,
   spec clause,
   or issue.

Pass criteria:

- Differential properties fail on unexplained disagreements.
- Explained disagreements do not become broad suppressions.

### Phase 8: Coverage and meta-checks

Deliverables:

- `fuzz:coverage` or `fuzz --coverage` task.
- Coverage summary for parser,
   emitter,
   splice,
   comments,
   AOT,
   path-create,
  effective-value,
   and resolve files.
- Committed baseline after the first complete strong campaign.
- Checklist section in `doc/decision/toml-edit-fuzzing.md`.

Pass criteria:

- Coverage task fails when target-file coverage regresses from baseline,
   and CI runs
  that gate for relevant toml-edit changes.
- The checklist requires all five questions for future targets:
  - Is the tested layer where the logic and bugs live?
  - Are all public entry points and internal seams covered?
  - Is every oracle stronger than no-crash or returns-a-string?
  - Do generators cover the full grammar and boundary cases?
  - Are real corpus seeds,
     counterexamples,
     and coverage feedback wired in?

### Phase 9: CI and issue closure

Deliverables:

- CI workflow or workflow step running a short bounded toml-edit fuzz smoke and
  the coverage no-regression gate on PRs and merge groups that touch
  `package/module/toml-edit/**`,
   `package/test-fixture/toml-edit/**`,
   or the
  toml-edit fuzz decision doc.
- CI uses a total smoke budget for the package,
   not 60 seconds per property.
   Local
  campaign mode may keep a deeper per-property budget.
- CI command uses the package tasks,
   not raw `node` commands pasted into workflow
  logic.
- Explicit `gh issue close 198` after the verified implementation is merged.
   Do
  not rely on a commit message alone,
   and do not wait for the next Scorecard scan
  before closing the implementation issue.
   If the next Scorecard scan still flags
  FuzzingID,
   file or update a follow-up about the scanner heuristic without
  weakening the fuzzing design.

Pass criteria:

- Local verification includes:
  - `mise run //package/module/toml-edit:lint:types`;
  - `mise run //package/module/toml-edit:test:unit`;
  - the toml-test conformance task;
  - `mise run //package/module/toml-edit:fuzz --budget 60000`;
  - the coverage task.
- CI path filtering is proven with an actual workflow run or a documented dry-run
  mechanism before issue #198 is closed.

## Target package reference

Public entry points from `package/module/toml-edit/src/index.ts`:

- `parseTomlEdit`
- `emptyTomlEdit`
- `tomlHas`
- `tomlGet`
- `tomlGetValue`
- `tomlGetRaw`
- `tomlGetNode`
- `tomlKeys`
- `tomlGetComments`
- `tomlGetCommentsBefore`
- `tomlGetCommentAfter`
- `tomlSet`
- `tomlDelete`
- `tomlSetHeaderComment`
- `tomlInsertCommentBefore`
- `tomlInsertCommentAfter`
- `tomlStringify`
- re-exported `getStaticTOMLValue` and `parseTOML` from `toml-eslint-parser`

Internal seams that need direct properties or named coverage blocks:

- `package/module/toml-edit/src/parse-toml-edit.ts`
- `package/module/toml-edit/src/emit-value.ts`
- `package/module/toml-edit/src/emit-value-string.ts`
- `package/module/toml-edit/src/values.ts`
- `package/module/toml-edit/src/value-encoders.ts`
- `package/module/toml-edit/src/keys.ts`
- `package/module/toml-edit/src/splice.ts`
- `package/module/toml-edit/src/comments.ts`
- `package/module/toml-edit/src/toml-set.ts`
- `package/module/toml-edit/src/toml-set-aot.ts`
- `package/module/toml-edit/src/path-create.ts`
- `package/module/toml-edit/src/path-create-merge.ts`
- `package/module/toml-edit/src/toml-delete.ts`
- `package/module/toml-edit/src/effective-value.ts`
- `package/module/toml-edit/src/resolve.ts`
- `package/module/toml-edit/src/walk.ts`
- `package/module/toml-edit/src/collision.ts`

## Definition of done

- toml-edit has property coverage across all scalar values,
   key forms,
   nesting,
  inline tables,
   standard tables,
   AOT,
   comments,
   path-create,
   delete,
   and splice
  behavior.
- The oracle set includes round-trip,
   aggressive metamorphic,
   conformance,
  differential,
   and stateful model oracles.
- The normal unit suite runs bounded properties;
   the `fuzz` task runs a budgeted
  campaign;
   the coverage task reports and gates target-file coverage.
- Every discovered counterexample from fuzzing,
   conformance,
   or differential
  testing is fixed in this scope and pinned in `examples` or corpus fixtures.
  Separate issues are reserved for explicit package-contract changes,
   not ordinary
  discovered bugs.
- `doc/decision/toml-edit-fuzzing.md` records the method,
   the dependency vetting
  record,
   rejected alternatives,
   the reusable checklist,
   and mutation testing as
  deferred follow-up with a linked issue.
- CI runs a short toml-edit fuzz smoke for relevant changes.
- Issue #198 is closed explicitly with `gh issue close 198` after verification.
