# @monochromatic-dev/upstream-fork-deepmerge-ts.fuzz

Consumer-side verification of the third-party
[`deepmerge-ts`](https://github.com/RebeccaStevens/deepmerge-ts) package:
a fast-check property campaign,
a reference model of its documented semantics,
a known-defect registry,
type-level tests,
and a V8 coverage-reachability gate.

Non-runtime sidecar:
nothing here ships,
and deepmerge-ts is a dev dependency only.
Decisions,
evidence,
and open items:
`doc/handover/deepmerge-ts-hardening.md`.

## What is checked

- `src/merge-model.property.unit.test.ts`:
   `deepmerge`,
   `deepmergeCustom({ maxDepth })`,
   `deepmergeInto`,
   and both `FastUnsafe` variants agree with `src/model.ts` on generated trees
   (accessors,
   hidden keys,
   null prototypes,
   frozen records,
   prototype-named keys,
   symbols,
   Sets,
   Maps).
- `src/merge-invariant.property.unit.test.ts`:
   inputs are never mutated,
   `Object.prototype` is never polluted,
   `undefined` arguments are neutral,
   and same-chain cycles merge into a result with that cycle.
- `src/model.unit.test.ts` and `src/json-case.ts`:
   the model against documented examples,
   independent of deepmerge-ts,
   plus a JSON-serializable corpus intended for the Rust unified-linter merge.
- `src/realm.ts`:
   realm-independent record,
   Set,
   and Map checks shared by the model and `src/shape.ts`,
   pinned by `src/realm.unit.test.ts`.
- `src/exotic-*.ts`:
   subclasses,
   typed arrays,
   boxed primitives,
   Proxies,
   and other-realm objects against the model,
   plus characterization of `isRecord` branches and non-writable `deepmergeInto` targets.
- `src/options-*.ts`:
   the four `*Custom` entry points against an options-aware model over generated option plans
   (custom and disabled merge functions,
   `actions.skip` and `actions.defaultMerge`,
   filters,
   `enableImplicitDefaultMerging`,
   `maxDepth`).
- `src/alias-*.ts`:
   shared references,
   cycles through records,
   arrays,
   Sets,
   and Maps,
   and `deepmergeInto` aliasing,
   checked by a graph bisimulation oracle under upstream's cycle rule.
- `src/scale-width.unit.test.ts`:
   linear scaling of wide two-input merges.
- `src/known-defect*.unit.test.ts`:
   each known runtime divergence asserted to still reproduce,
   so an upstream fix turns it red.
   The comment on each test names the generator region it excludes.
- `src/accepted-behaviour.unit.test.ts`, `src/exotic-behaviour.unit.test.ts`, and `src/alias-accepted.unit.test.ts`:
   accepted but otherwise untested behaviour
   (the default `maxDepth` fallback,
   getter flattening,
   frozen targets,
   single-input reference sharing).
- `src/type-*.unit.test.ts`:
   `expectTypeOf` pins of result types beside runtime values;
   `src/type-known-defect.unit.test.ts` pins result types the runtime contradicts.
- `src/type-soundness.generated.ts`:
   1000 generated calls (600 on widened literals, 400 on `as const` literals checked for exact literal and tuple types)
   whose runtime results must type-check against their static result types;
   `src/type-soundness.unit.test.ts` replays them at runtime.
- `src/declared-type-*.ts`:
   the same soundness check over generated declared types
   (unions,
   optional keys,
   index signatures,
   tuples,
   `deepmergeInto` targets);
   `src/declared-type-soundness.generated.ts` keeps the fixed-seed draws outside pinned classes,
   and `src/type-known-defect-declared.unit.test.ts` pins the classes it found.
- `src/mutation-*.unit.test.ts`:
   example tests that detect the upstream mutants the rest of the suite missed
   (custom merge functions, cycle resolution, record paths, custom metadata),
   each naming the mutant ids it kills;
   method and results in `doc/audit/deepmerge-ts-mutation-2026-09-24.md`.
- `src/recall-*.ts`:
   historical recall,
   the positive control for the whole method:
   a ledger of every bug upstream shipped and fixed,
   rebuilt buggy and fixed,
   with the sidecar run against both
   (runtime bundles, result types against each npm release, and six ways of loading each release);
   `src/recall-known-defect-union-depth.unit.test.ts` pins the result-type defect found on the way;
   method and results in `doc/audit/deepmerge-ts-recall-2026-09-24.md`.
- `src/surface-*.ts`:
   surfaces no other layer loads:
   every documented example and claim run literally
   (`surface-docs-example`, `surface-docs-claim`, `surface-docs-type`),
   export, collection-view, spread, and interface result types (`surface-known-defect`),
   and container scripts for the TypeScript 4.7 to 7.0 flag matrix,
   type-checker cost sweeps,
   and Bun and Deno runs;
   method and results in `doc/audit/deepmerge-ts-surface-2026-09-24.md`.

Machine-local files matching `*.local.*` (gitignored)
hold embargoed security findings until upstream publishes an advisory.

## Running

```bash
# Bounded, fixed-seed layer (also what the coverage gate measures)
mise run //package/upstream-fork/deepmerge-ts.fuzz:test:unit

# Result types, including the generated soundness corpus
mise run //package/upstream-fork/deepmerge-ts.fuzz:lint:types

# Unbounded campaign in a 2 GiB / 2 CPU podman container; stops at the first
# counterexample and writes a replay record under dist/fuzz-failure/
mise run //package/upstream-fork/deepmerge-ts.fuzz:fuzz --round-runs 10000

# Coverage-reachability gate over the installed dist; --write ratchets the baseline
mise run //package/upstream-fork/deepmerge-ts.fuzz:fuzz:coverage

# Regenerate the type-level soundness corpus after a generator or version change
mise run //package/upstream-fork/deepmerge-ts.fuzz:generate:type-cases

# Declared-type campaign, positive control, and fixed-seed corpus, each in a 2 GiB / 2 CPU podman container;
# the campaign report lands in dist/declared-type/<seed>/report.json
mise run //package/upstream-fork/deepmerge-ts.fuzz:fuzz:declared-types --seed 11 --size 1500
mise run //package/upstream-fork/deepmerge-ts.fuzz:fuzz:declared-types:control
mise run //package/upstream-fork/deepmerge-ts.fuzz:generate:declared-type-cases
```

Mutation testing of a deepmerge-ts checkout
(StrykerJS, Vitest, and esbuild live only in the image from `container/mutation.Containerfile`;
every run is capped at 2 GiB / 2 CPUs with the checkout and this repo read-only and `dist/mutation/` writable):

```bash
# Stryker over the checkout with upstream's own Vitest suite; writes dist/mutation/mutation.json
mise run //package/upstream-fork/deepmerge-ts.fuzz:mutation --checkout /absolute/path/to/deepmerge-ts

# Control: every sidecar file passes against the unmutated esbuild bundle
mise run //package/upstream-fork/deepmerge-ts.fuzz:mutation:sweep --checkout /absolute/path/to/deepmerge-ts --baseline

# Run each Survived and NoCoverage mutant (or the listed ids) through every sidecar file;
# one line per mutant in dist/mutation/sweep.jsonl
mise run //package/upstream-fork/deepmerge-ts.fuzz:mutation:sweep --checkout /absolute/path/to/deepmerge-ts

# Compare swept bundles with the baseline on fixed-seed generated cases;
# fails unless the baseline matches itself and the control mutant differs
mise run //package/upstream-fork/deepmerge-ts.fuzz:mutation:differential --control 34 283 449
```

Historical recall
(inputs live under `~/temp/agent`, outside this repo;
the runtime and packaging runs use the mutation image,
and every run except setup and report is capped at 2 GiB / 2 CPUs
with this repo read-only and only its `dist/recall/<area>/` writable):

```bash
# Every npm release and the upstream trees the ledger names (host; downloads and git archive only)
mise run //package/upstream-fork/deepmerge-ts.fuzz:recall:setup --clone /absolute/path/to/full-history/deepmerge-ts

# Runtime rows: control per row, then every sidecar file on the buggy and fixed bundles;
# one line per row in dist/recall/runtime/runtime.jsonl
mise run //package/upstream-fork/deepmerge-ts.fuzz:recall:runtime undefined-middle

# Result-type rows: a fresh declared-type draw, the sidecar against releases, or one row's control
mise run //package/upstream-fork/deepmerge-ts.fuzz:recall:types corpus 20260930 1500
mise run //package/upstream-fork/deepmerge-ts.fuzz:recall:types check 6.0.1 6.0.2
mise run //package/upstream-fork/deepmerge-ts.fuzz:recall:types control empty-record

# Six ways of loading each release; dist/recall/packaging/results.json
mise run //package/upstream-fork/deepmerge-ts.fuzz:recall:packaging 7.1.5 8.0.2

# Summaries over files present when the audit started (--baseline-commit, default e4237dc8f)
mise run //package/upstream-fork/deepmerge-ts.fuzz:recall:report
```

Unsearched surfaces
(the toolchain installs into `dist/surface/scratch`, never this workspace;
every run is capped at 2 GiB / 2 CPUs with this repo read-only and only `dist/surface/` writable):

```bash
# TypeScript 4.7 to 7.0 and probe tools
mise run //package/upstream-fork/deepmerge-ts.fuzz:surface:install

# Type tests per release and flag set
mise run //package/upstream-fork/deepmerge-ts.fuzz:surface:types --configs base,isolated,no-strict-null ts60 ts70

# Instantiations and check time per case family; measure the band with --repeat first
mise run //package/upstream-fork/deepmerge-ts.fuzz:surface:cost --alias ts60 --prefix wide --repeat 7

# Every sidecar unit file under Bun or Deno (DEEPMERGE_FUZZ_TARGET passes through)
mise run //package/upstream-fork/deepmerge-ts.fuzz:surface:runtime --runtime bun
mise run //package/upstream-fork/deepmerge-ts.fuzz:surface:runtime --runtime deno
```

Replay a campaign record on the host:

```bash
# package/upstream-fork/deepmerge-ts.fuzz
DEEPMERGE_FUZZ_SEED=<seed> DEEPMERGE_FUZZ_NUM_RUNS=<runs> node <file>
```

## Testing another build

Set `DEEPMERGE_FUZZ_TARGET` to a module path
to run every runtime test and the campaign against it instead of the npm release.
Relative paths resolve against this package directory.
The type-level tests always use the installed npm release's types.

For a deepmerge-ts checkout (the fork lives at <https://github.com/Aquaticat/deepmerge-ts>),
`fork:build` builds a read-only copy with source maps inside a capped container,
and `fuzz:coverage` then reports coverage per `src/*.ts` file with uncovered line ranges
instead of gating the npm baseline:

```bash
mise run //package/upstream-fork/deepmerge-ts.fuzz:fork:build --checkout /absolute/path/to/deepmerge-ts
DEEPMERGE_FUZZ_TARGET=dist/fork-build/index.mjs mise run //package/upstream-fork/deepmerge-ts.fuzz:test:unit
DEEPMERGE_FUZZ_TARGET=dist/fork-build/index.mjs mise run //package/upstream-fork/deepmerge-ts.fuzz:fuzz:coverage
```
