# deepmerge-ts historical recall audit, 2026-09-24

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/deepmerge-ts.fuzz` is now `package/upstream-fork/deepmerge-ts.fuzz`.

## Question

Would the sidecar in `package/module/deepmerge-ts.fuzz`,
as it stood at commit `e4237dc8f` (the start of `Method audit` in `doc/handover/deepmerge-ts-hardening.md`),
have detected the bugs deepmerge-ts actually shipped and later fixed?
This is the positive control for the search method as a whole:
the campaign's quiet rounds mean something only if the method catches real faults of the kinds upstream ships.

## Answer

Of 22 historical bugs that could be re-created and shown to reproduce,
the sidecar detects 16.

- Runtime:
   8 of 8,
   all in the fixed-seed bounded layer (`test:unit`),
   by files that specify behaviour (not only by files that pin current behaviour).
- Result types:
   8 of 10;
   7 in the bounded layer (`lint:types`),
   1 more only in a fresh draw of the declared-type campaign generator.
- Packaging and metadata:
   0 of 4.

Every miss sits on a surface the sidecar never loads:
exported utility types and the typed `utils` of custom merge functions,
module resolution other than `bundler`,
and the `engines` field.
No miss came from generator reach or from a model that copies the bug,
and no runtime bug needed the campaign.

The history is small:
upstream shipped 8 runtime fixes in total,
so "8 of 8" says the method catches the classes of runtime fault upstream has shipped,
not that it bounds how many unknown faults remain.
Most of upstream's shipped bugs were in result types and packaging,
and those are where the misses are.

A new current-version result-type defect came out of building the controls
(see `New finding on 8.0.2`).

## Ground truth

The ledger lives in `package/module/deepmerge-ts.fuzz/src/recall-ledger-*.ts`,
one row per fix, with commit, release, issue, what broke, and upstream's regression test.
Sources:
every `Bug Fixes` entry in upstream `CHANGELOG.md` (all versions),
every commit whose subject starts with `fix`,
every closed issue labelled `Type: Bug`,
and both published advisories (`gh api /advisories?affects=deepmerge-ts`):
`GHSA-r9w3-g83q-m6hq` (prototype pollution, fixed in 4.0.2)
and `GHSA-ggr8-5vv4-36mx` (stack exhaustion on cyclic input, fixed in 8.0.0 by a `feat` commit,
so absent from `Bug Fixes`).

34 rows:
8 runtime,
14 result-type,
12 packaging, install, engines, compiler-cost, and repo-internal.
Two repo-internal rows (`df4add2` missing dev dependency, `be28290` eslint comment) have no consumer effect
and are excluded,
leaving 32.

Closed bug issues that were never fixed (`#22` `PartialDeep` inputs, `#488` `index.d.cts` with `skipLibCheck: false`,
`#416` Safari and `Array.prototype.at`)
are outside the recall question;
`#488` does not reproduce against 8.0.2 in the `nodenext` CommonJS probe below.

## Method

### Runtime rows

`src/recall-runtime.ts` runs inside the capped mutation image (`podman run --memory=2g --cpus=2`),
with upstream trees mounted read-only.
For each row it builds a buggy and a fixed bundle with esbuild:

- Transplant rows re-apply the historical fault to the v8.0.2 source at the place the fix lives today
   (exact-text edits in `src/recall-ledger-runtime.ts`, each required to match a stated number of times),
   so the sidecar runs against today's API and only the fault differs;
   the fixed side is unmodified v8.0.2.
- Parent rows bundle the fix commit's parent and the fix commit itself.
   For `3984927` both trees predate the FastUnsafe entry points,
   so both get the same shim re-exporting the default entry points under those names.

Positive control per row:
`reproduces`, ported from upstream's regression test or the issue repro,
must return `true` on the buggy bundle and `false` on the fixed one,
or the run aborts.
Every sidecar `src/*.unit.test.ts` file then runs against both bundles;
a test counts as detecting the bug only if it fails on the buggy bundle and passes on the fixed one.
Bounded campaign rounds (5 seeds, 2 000 runs per property) were queued for rows the bounded layer missed;
none were needed.
Harness control:
the v8.0.2 bundle passes every sidecar file.

### Result-type rows

`src/recall-types.ts` type-checks the whole sidecar (`src/**/*.ts`, excluding `*.local.*`)
with `deepmerge-ts` mapped by `paths` to each npm release's declarations,
using the workspace TypeScript 7.0.2 and the repo's `tsconfig.dom.json` settings,
plus a fresh 1 500-case draw of the declared-type campaign generator (seed 20260930, runtime values from 8.0.2).
Diagnostics are keyed by file, position, and code;
a row's detections are the diagnostics present with the buggy release and absent with the fixed one.

Positive control per row:
upstream's `expectType` assertion (or the issue's snippet) as a standalone module,
with an identity check as strict as tsd's,
must fail against the buggy release and pass against the fixed one.
Harness control:
against 8.0.2 the sidecar has 0 diagnostics outside the fresh draw,
matching `lint:types`;
the fresh draw has 408 against 8.0.2 itself
(the committed corpus build drops such draws),
and they cancel out of every attribution.

### Packaging rows

`src/recall-packaging.ts` installs each release into its own consumer directory outside the repo
and loads it six ways:
the sidecar's two
(TypeScript 7 `bundler` resolution through the `exports` types condition, and a Node ESM named import)
and four it never uses
(TypeScript 6.0.2 `node10`, TypeScript 7 `nodenext` from `.mts` and from `.cts`, Node `require`).
A first run placed consumers inside the package,
and TypeScript's lookup walked up into the package's own `node_modules/deepmerge-ts` (8.0.2) and passed every probe;
the consumers now live under `/tmp` in the container.

### Scope

Only files present at `e4237dc8f` count
(`dist/recall/baseline-files.txt`, from `git ls-tree`),
so files added during this audit by any workstream cannot raise recall.
The fresh declared-type draw counts as the campaign layer, not the bounded layer.

## Results by row

### Runtime, 8 of 8 detected

- `105b646` (1.1.0) `deepmerge()` returned `{}`:
   detected only by one hand-written example in `src/type-record.unit.test.ts`.
   No property draws a zero-argument call (`mergeArgumentsArbitrary` uses `minLength: 1`).
- `0967070` (1.1.1, `#8`) inherited and non-enumerable properties joined the merge:
   13 tests, including the model, options-model, exotic, and pollution properties.
- `d637db7` (4.0.2, `GHSA-r9w3-g83q-m6hq`) `__proto__` assigned instead of defined:
   10 tests, including the invariant and model properties.
- `3363570` (4.0.4) non-enumerable symbol keys collected:
   11 tests, including the model and options-model properties.
- `6b04863` (6.0.3) `deepmergeInto` defined `__proto__` on its internal reference:
   3 tests (into model property, into options-model property, a mutation-derived example).
- `0784f63` (7.0.0, `#460`) an `undefined` among 3+ values sent the merge to `mergeOthers`:
   the invariant `undefined arguments are neutral` and a runtime example in `src/type-leaf.unit.test.ts`;
   the model properties did not draw a separating case in 400 runs.
- `2cd7824` (8.0.0) `deepmergeInto` wrote into a source's nested container:
   5 tests, including `deepmergeInto never mutates its sources` and the into model property.
- `3984927` (8.0.0, `GHSA-ggr8-5vv4-36mx`) cyclic input exhausted the stack:
   26 tests, including the alias-graph and cycle invariant properties.

### Result types, 8 of 10 measured rows detected

- `696a1b2` (1.1.7, `#16`) 3+ readonly tuples:
   control separates,
   but type-checking the sidecar against 1.1.6 did not finish in 900 s,
   so no attribution was possible.
   The sidecar's own type check did not finish against that release either.
- `944b428` (5.0.0, `#304`) `DeepMergeMergeFunctionUtils` required a generic:
   missed.
   The sidecar imports two exported types (`DeepMergeLeafURI`,
   `DeepMergeNoFilteringURI`) and never names the utils type.
- `fa9ace2` (6.0.0, `#451`) optional over required:
   detected in every type layer,
   but 5.1.0 to 6.0.0 is a major release whose other commits include two breaking type changes,
   so this attribution is not isolated.
- `5e8b9b6` (6.0.1, `#459`) index signatures gained `undefined`:
   detected by one committed declared-type case and 12 fresh-draw cases.
- `6b4ff3f` (6.0.2, `#465`) empty record argument:
   detected by 29 committed declared-type diagnostics and 104 fresh-draw diagnostics.
- `1832bd0` (7.0.1, `#476`) all-optional records returned `never`:
   detected by the committed declared-type corpus (27), hand-written `expectTypeOf` (3), and the fresh draw (170).
- `ca94270` (7.0.3, `#482`) bundled declarations gave `utils.defaultMergeFunctions` the into signatures:
   missed.
   No sidecar custom merge function calls `utils.defaultMergeFunctions`.
   The concurrent surface workstream's `src/surface-docs-example.unit.test.ts` (added after `e4237dc8f`) does catch it
   (4 buggy-only TS2554).
- `6d85163` (7.1.4, `#524`) leaf unions with `undefined` ignored filtering:
   detected by the committed declared-type corpus (2) and hand-written `expectTypeOf` (3).
- `349fd14` (7.1.5, `#529`) nested optional over required:
   missed by every committed specifying layer;
   detected by 4 fresh-draw cases, and 2 known-defect pins flip.
- `e86cd8c` (7.1.6, `#692`) index signature absorbed known keys:
   detected by the committed declared-type corpus (4) and the fresh draw (64).
- `b307b77` (8.0.1, `#714`) TypeScript 7 support:
   upstream's `tests/consumer-ts7/fixture.ts` compiles against 8.0.0 under TypeScript 7.0.2,
   so it cannot serve as the control;
   a leaf union of 120 members that may be `undefined` gives TS2589 against 8.0.0 and not 8.0.1,
   which matches the fix (tail-recursive `UnionToTuple`).
   Detected by one TS2321 (excessive stack depth) in `src/options.property.unit.test.ts`.

Unmeasured:
`ee59064` (2.0.0, `#17`), `fc85dfa` (3.0.1, `#60`), and `9a881d3` (4.0.0,
`#61`) have no consumer-level assertion upstream,
and probes of readonly records, tuples, Sets, and Maps under TypeScript 7.0.2 found no program that separates the
releases.

Release pairs are not single commits:
`git log` between each pair lists only the fix plus renames and style commits,
except 5.0.0 (repo re-initialisation chores), the 6.0.0 major above, and 7.1.6,
which also carries a build maintenance update.

### Packaging and metadata, 0 of 4 measured rows detected

- `063675e` (7.0.2, `#480`) no `types` fallback for `node10`:
   TypeScript 6 `node10` fails on 7.0.1 (TS2307) and passes on 7.0.2;
   both sidecar-way probes pass on both.
- `b875711` (4.2.1, `#145`) `typesVersions` named a missing file:
   `node10` fails on 4.2.0 and passes on 4.2.1;
   both sidecar-way probes pass on both.
- `a5f334b` (1.1.2, `#10`) current and legacy declarations overwrote each other:
   `node10` fails on 1.1.1 and passes on 1.1.2;
   the sidecar-way `bundler` probe fails on every 1.x release (TS7016, no `exports` types condition),
   so it cannot separate them.
- `ef54ea6` (8.0.2) `engines` allowed Node 16.0 to 16.8:
   8.0.1 declares `node >=16.0.0` while `dist/index.mjs` calls `Object.hasOwn` twice;
   the sidecar runs on one Node release and never reads `engines`.

Unmeasured:
`tsc-hang` (`eb4183e`, 4.0.3, `#94`) does not reproduce with TypeScript 7.0.2
(the sidecar checks against 4.0.2 in 3.5 s with the same 1 126 diagnostics as 4.0.3);
`4117460` (1.1.3, `#12`) changes only `exports` condition order, which none of the six probes observes;
`c7e1019` (1.1.1, `#6`) concerns TypeScript before 4.1;
`4b8ca98` and `86faf2a` concern the Deno build, which is not in the npm tarball;
`7102229` (1.0.1) fixed a `postinstall`, and 1.0.0 was never published to npm.

## Proposed changes

None of these are applied;
each touches shared files or adds tasks.

- Zero-argument calls:
   let `mergeArgumentsArbitrary` in `src/arbitraries.ts` draw empty argument lists
   (or add an arity property),
   so `105b646`-class faults are caught by a property, not one example.
- `undefined` between records:
   the `#460` class reached the model only through a top-level invariant;
   weight draws toward 3+ record inputs sharing a key where a middle value is `undefined`.
- Exported type surface:
   type tests that instantiate every exported type with its defaults,
   and custom merge functions typed through `utils` that call every `utils.defaultMergeFunctions` member with its
   documented signature,
   covering `944b428` and `ca94270`.
   Every documented `deepmergeCustom` example compiled verbatim covers `ca94270` too.
- Declared-type bounded corpus:
   `349fd14` was caught only by a fresh draw;
   regenerate the committed corpus from more seeds, or run a fresh draw in `test:unit`.
- Union size:
   draw leaf unions of 50 members and more, which would have found `New finding on 8.0.2`.
- Packaging matrix:
   run `src/recall-packaging.ts` against the installed release as a task,
   and fail when any of the six probes fails.
- `engines`:
   run the bundle under the lowest Node release `engines` admits, in a container,
   or check the built-ins the bundle calls against their first Node release.
- Tasks to add to the package `mise.toml`
   (the audit ran these as scratch drivers with the same `podman` arguments):
   `recall:setup` (`node src/recall-setup.ts <clone>` on the host),
   `recall:runtime` (mutation image, trees read-only at `/trees`, `dist/recall/runtime` at `/out`,
   `node src/recall-runtime.ts [ids]`),
   `recall:types` (`node:<major>-slim`, repo read-only, `dist/recall/types` writable at its host path,
   npm releases read-only with `RECALL_NPM_DIR`, `node src/recall-types.ts corpus|check|control`),
   and `recall:packaging` (mutation image, same mounts, `node src/recall-packaging.ts <versions>`).
   Reports: `node src/recall-runtime-report.ts` and `node src/recall-types-report.ts`.
- Formatting:
   the `recall-*` files still carry `chain-per-line` and `param-per-line` warnings that `format:oxlint` fixes;
   this workstream was not allowed to run it.

## New finding on 8.0.2

With the documented no-filtering setup,
`deepmergeCustom<unknown, { DeepMergeFilterValuesURI: DeepMergeNoFilteringURI }>({ filterValues: false })`,
merging two `{ status: Status }` records where `Status` is a string-literal union of 50 members
fails to type-check with TS2589 (type instantiation is excessively deep and possibly infinite),
and `status` types as `any`.
Measured on deepmerge-ts 8.0.2 with TypeScript 7.0.2 and 6.0.2.
A 45-member union type-checks;
a custom filter URI that returns its input fails from 60 members;
the default filter handles 120.
Not security-relevant.
Pinned in `src/recall-known-defect-union-depth.unit.test.ts` with `@ts-expect-error`;
removing the directive makes `lint:types` report TS2589 at that line.

## Evidence

- Commits:
   `bb05f7b58`, `127b84d9d`, `f3db5afe7`, `d353befe9`, `0ad31b9b6`, `91d913530`, `931132672`.
- Upstream history clone:
   `~/temp/agent/deepmerge-ts-history-2026-09-24` (push URL `DISABLED`).
- Outputs (gitignored):
   `package/module/deepmerge-ts.fuzz/dist/recall/runtime/runtime.jsonl`,
   `dist/recall/types/results/*.json`,
   `dist/recall/packaging/results.json`.
