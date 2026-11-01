# deepmerge-ts mutation testing, 2026-09-24

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/deepmerge-ts.fuzz` is now `package/upstream-fork/deepmerge-ts.fuzz`.

Mutation pass of the deepmerge-ts widening pass
(`doc/handover/deepmerge-ts-hardening.md`, section `Widening pass`).
It measures how strongly upstream's Vitest suite and the sidecar
(`package/module/deepmerge-ts.fuzz`) observe upstream's runtime source,
and turns every distinguishable surviving mutant into a sidecar test.

## Result

- Upstream's suite detects 800 of 1098 mutants (72.86 %).
- Upstream plus the sidecar as it stood during the sweep detects 907 (82.60 %).
   99 of those extra kills came from ordinary sidecar tests,
   8 only from known-defect tests.
- After this pass, 1017 are detected (92.62 %).
   The remaining 81 are argued equivalent from source,
   and a differential check found no input that separates any of them
   (see `Equivalent mutants`).
   Counting only non-equivalent mutants, detection is complete.
- New public findings for the combined issue draft: see `Findings`.
- No new security candidates.

## Method

### Upstream run

StrykerJS 10.0.0 with the Vitest runner,
over upstream `src/**/*.ts` except `src/types/**` (type declarations only),
against upstream's own suite at tag `v8.0.2` (commit `cbfd03b`).
The checkout, Stryker, Vitest 4.1.10, Vite 8.0.10, `vite-tsconfig-paths` 6.1.1, TypeScript 6.0.2, and esbuild 0.28.2
were baked into a local image from `docker.io/library/node:26-slim` plus `procps`
(Stryker's process cleanup calls `ps`).
Nothing was installed in this repo or in the fork.
Every run used `podman run --memory=2g --cpus=2 --rm --init`;
the run took 2 minutes 51 seconds.

```json
// stryker.config.json, baked into the image beside upstream's vitest.config.ts
{
  "testRunner": "vitest",
  "vitest": { "configFile": "vitest.config.ts" },
  "mutate": ["src/**/*.ts", "!src/types/**"],
  "coverageAnalysis": "perTest",
  "concurrency": 2,
  "timeoutMS": 10000,
  "reporters": ["json", "clear-text"],
  "jsonReporter": { "fileName": "/out/mutation.json" }
}
```

Stryker's result: 783 killed, 17 timed out, 238 survived, 60 without coverage, 0 errors.
Upstream's suite scored lowest on `src/defaults/into.ts` (51.32 %) and `src/defaults/vanilla.ts` (64.46 %),
the files holding `deepmergeInto`'s record, Map, and cycle-resolution logic.

### Sidecar sweep

For each of the 298 surviving or uncovered mutants,
a scratch driver applied Stryker's recorded replacement to the source text,
bundled `src/index.ts` with esbuild into one ES module,
and ran every sidecar `*.unit.test.ts` file with `DEEPMERGE_FUZZ_TARGET` pointing at that bundle,
two files at a time, 120 seconds per file, inside the same capped container
with this repo mounted read-only (`--security-opt label=disable`).
A mutant counts as killed when any file fails.
Kills only by `known-defect*` files or machine-local pinning files are reported separately,
because those files pin current behaviour instead of specifying it.

Controls:
the unmutated esbuild bundle passed every sidecar file,
so the bundle stands in for the npm release;
99 mutants were killed, so the override reached the tests.
Other widening forks were adding files during the sweep,
so the sidecar figures describe the files present then.

### Differential check

For the mutants still surviving after the new tests,
a second scratch harness compared each mutant bundle with the unmutated bundle
on 1000 generated cases in each of six categories:
alias graphs (`src/alias-graph.ts`) through `deepmerge` and `deepmergeInto`,
alias graphs with option plans (`src/options-arbitraries.ts`, `src/options-build.ts`) through both custom entry points,
exotic arguments (`src/exotic-arbitraries.ts`) through all four default entry points,
and sidecar trees with FastUnsafe option plans.
It compared a canonical serialization of every result and every input afterwards:
object identity (fresh, input, or back-reference), prototype, tag, own-key descriptors, entries,
and thrown error class.
Controls: the unmutated bundle against itself showed no difference,
and a mutant the sidecar kills (34) showed one.

## Where the sidecar's earlier kills came from

- `options.property.unit.test.ts` killed 68, 22 of them alone.
- `merge-model.property.unit.test.ts` killed 46, none alone.
- `merge-invariant.property.unit.test.ts` killed 23, 4 alone.
- `alias-graph.property.unit.test.ts` killed 22, 17 alone (3 of them by timing out).
- `exotic-merge.property.unit.test.ts` killed 14, none alone.
- `exotic-behaviour.unit.test.ts` killed 13, 8 alone.
- A machine-local file killed 1, none alone.

The first-pass files (model, invariant, type, helper, and accepted-behaviour tests) killed 52 of the 298;
the widening forks' new files account for the rest of the 99.
That matches the widening pass's premise that the first pass's generators were narrow.

Killed only by known-defect tests before this pass:
214, 225, 227, 229, 378, 380 (`known-defect-options.unit.test.ts`),
445 (`known-defect.unit.test.ts` and `known-defect-alias.unit.test.ts`),
488 (`type-known-defect.unit.test.ts`).
The new tests also kill 445 and 488.

## New tests

All pass against the npm release 8.0.2.
Each test's comment names the mutant ids it kills.
Mutant ids refer to this run's `mutation.json`.

### `src/mutation-custom.unit.test.ts`

- `filterValues` dropping every value at a key leaves the target value in place:
   19, 21, 107, 109, 201, 203, 220, 222.
- Implicit default merging is off unless enabled: 6, 366.
- Implicit default merging falls back for `mergeSets`, `mergeMaps`, and `mergeCircularReferences`: 75, 80, 500.
- `mergeCircularReferences` returning `actions.defaultMerge` runs the default: 499, 501.
- A custom `mergeSets` or `mergeMaps` replaces the default: 78, 488, 493.
- Into merge functions run the default only on request: 175, 180, 181, 324.
- Into `mergeCircularReferences` requesting the default, by return or by slot, runs it:
   337, 338, 339, 341, 343, 344.
- Into `filterValues: false` keeps `undefined`, and the FastUnsafe into default filters it: 99, 197, 199.
- `rootMetaData` reaches the root merge function: 358.
- `actions.skip` drops keys on the FastUnsafe three-record path and Map entries: 843, 845, 932, 934.
- `utils.mergeFunctions` exposes only the merge functions, not other or unset options: 559, 572.

### `src/mutation-cycle.unit.test.ts`

- A cyclic value in the second position is remapped onto the result: 279, 445.
- Three values where only the last is cyclic, or only the last is not, resolve by the last: 461, 989.
- `deepmergeInto` remaps nested cycles onto the target, never onto a source: 253, 255, 798, 801, 805, 808.
- `deepmergeInto` resolves a plain last value holding an ancestor reference onto the target:
   737, 741, 742, 744, 755, 757, 761, 762, 763, 765, 767, 769, 770.
- Resolution returns an unchanged plain value by reference: 764, 766, 768, 961, 963, 965.
- Resolution leaves an array holding an ancestor reference as that array: 760, 957.
- Resolution copies an own `__proto__` key as an own enumerable writable configurable key:
   773, 775, 777, 778, 779, 780, 970, 972, 974, 975, 976, 977.

### `src/mutation-record.unit.test.ts`

- Every record path stores `__proto__` as an own enumerable writable configurable key:
   671, 673, 705, 706, 707, 881, 915, 916, 917.
- `deepmergeInto` with two sources assigns through a target setter: 698.
- A Module-tagged object with a non-plain prototype merges as a record: 1054, 1056, 1080, 1081, 1082.
- An object whose constructor prototype is not Object-tagged is a leaf, even with `isPrototypeOf`: 1075, 1083, 1084.
- `objectHasProperty` is false for functions: 1035.
- `deepmergeInto` replaces a target Set's contents with the merged values when `filterValues` drops it: 590.

### `src/mutation-metadata.unit.test.ts`

- Every into remap falls back to the first input of the ancestor level
   when a custom `metaDataUpdater` omits `result`: 257, 759, 804, 811.

### `src/known-defect-mutation.unit.test.ts`

- Defect: `filterValues` removing every value at a key the target lacks leaves an empty container: 511.
- Intent question: invalid `maxDepth` values in `deepmergeIntoCustom` silently fall back to 1000: 207, 209.

### Machine-local test

One machine-local test file kills 4 further mutants (226, 228, 397, 399);
it is not part of the committed suite.

## Findings

### Defect: `deepmergeInto` assigns an empty container where `deepmerge` assigns `undefined`

With a custom `filterValues` that removes every value at a key the target lacks,
`deepmergeInto` sets that key to an empty container of the first value's kind,
while `deepmerge` with the same option sets it to `undefined`.
Verified on 8.0.2 with one source and with two:

```js
import { deepmergeCustom, deepmergeIntoCustom } from "deepmerge-ts";

const dropArrays = (values) => values.filter((value) => !Array.isArray(value));
console.log(deepmergeCustom({ filterValues: dropArrays })({ a: [1] }, { a: [2] })); // { a: undefined }
const target = {};
deepmergeIntoCustom({ filterValues: dropArrays })(target, { a: [1] });
console.log(target); // { a: [] }
```

Cause: `mergeRecordsInto` in `src/defaults/into.ts` seeds the slot with `emptyLike(propValues[0])` before
`mergeUnknownsInto` filters, and `mergeUnknownsInto` returns early on zero values, so the seed is assigned.
The same seeding causes the known first-value typing defect.
The pending `fix/into-first-value-typing` branch reseeds only when kinds differ,
so it does not cover this case.

### Intent question: invalid `maxDepth` in `deepmergeIntoCustom`

Same behaviour and question as the `deepmergeCustom` case the options fork pinned:
`-1`, `NaN`, and `"1"` silently become 1000.

### Characterization: arrays are not searched for ancestor references during cycle resolution

When a merge resolves a plain value against a cyclic sibling,
`resolveCyclicReferences` rebuilds records that reference an ancestor,
but returns arrays unchanged.
A source array that references a source ancestor therefore stays shared with the source,
and the result points into the source graph instead of at itself.
This overlaps the aliasing fork's cycles-through-containers area;
the new test only pins that the array is returned as is.

### Observation: into cycle remaps fall back to a source object under custom metadata

`deepmergeInto`'s remaps read `result ?? parents[0]`.
A custom `metaDataUpdater` that keeps `hierarchy` but omits `result` makes every remap
point at the first input of that level, which is a source record when the target lacked the key,
so the target ends up referencing a source object.
`deepmerge`'s remaps read `result` only.
Pinned in `mutation-metadata.unit.test.ts`; worth one line in the issue draft as a question
about whether `result` is a required part of custom metadata.

## Equivalent mutants

81 mutants survive every test.
Each is argued equivalent from the source below,
and none was separated by the differential check.
Arguments marked "environment" hold only under an unmodified `Object.prototype`
or unpatched primitive-wrapper `Symbol.toStringTag`.

- Kind guard forced true or joined with `||`
   (26, 28, 29, 31 in `deepmerge-fast.ts`; 114, 116, 117, 119 in `deepmerge-into-fast.ts`;
   258, 260, 261, 263 in `deepmerge-into.ts`; 424, 426, 427, 429 in `deepmerge.ts`).
   For a leaf or other first kind, the added checks route a kind mismatch to `mergeOthers`,
   which the `switch` default does anyway, and compute cyclic depths that are always 0:
   `parents` holds only the record or Map containers merged at each level,
   and in `deepmergeInto` a leaf target value only occurs at levels the user's target already had,
   where no ancestor container is a leaf. FastUnsafe computes no depths at all.
- Two-value fast path disabled (35, 123, 267, 433):
   the general loop performs the same checks for two values.
- Two-record fast paths removed (597, 599, 657, 659, 815, 817, 866, 868):
   the general record merge visits the same keys in the same order with the same metadata.
- `new Array(length)` becomes `new Array()` (283, 449): the array grows on assignment.
- `utils.maxDepth !== undefined` forced true (234, 405): these variants always set a number.
- `hierarchy !== undefined` forced true (244, 415, 749, 949):
   `getCyclicReferenceDepth` returns 0 without a hierarchy, and a positive depth implies one.
- Optional chaining removed where the entry always exists
   (254, 256, 758, 799, 800, 802, 803, 806, 807, 809, 810, 756, 955):
   a positive depth never exceeds `hierarchy.length`.
- Partial mutants of the primitive guard in cycle resolution
   (738, 739, 740, 743, 745, 938, 939, 940, 943, 945):
   primitives and `null` fall through to depth 0 and kind `NOT` and come back unchanged.
- `key === "__proto__"` forced true on fresh result records (771, 874, 908, 968), environment:
   defining a full data property equals assignment on a fresh ordinary object.
- Circular-merge loop mutants (785, 787, 789, 794, 981, 983, 985):
   with all depths equal, the "depths differ" branch returns the same ancestor result,
   and resolving a cyclic last value returns that same result.
- Plain-record check, constructor prototype null or not an object (1065, 1066, 1067, 1069, 1072), environment:
   the following `Object.prototype.toString` check rejects `null`, functions, and primitives.
- Symbol fast path (1017, 1019): iterating an empty array does nothing.
- `getCyclicReferenceDepth` shortcuts (1098, 1110): the loop and `includes` give the same answer.
- `defaultMetaDataUpdaterFast` body emptied (536): it still returns `undefined`.
- `shouldFallbackToDefault` forced to fall back when the function is the default (555):
   the default runs again on the same values and returns the same value.

## Follow-up integration

- Reproducible from the repo (user decision recorded in `doc/handover/deepmerge-ts-hardening.md`):
   the image, Stryker config, sweep driver, and differential harness are now
   `package/module/deepmerge-ts.fuzz/container/`
   plus `src/mutation-{score,sweep,sidecar,canon,differential,differential-case}.ts`,
   run by the `mutation`, `mutation:sweep`, and `mutation:differential` tasks (commands in the package `README.md`).
   Rerun from a `v8.0.2` checkout:
    800 of 1098 detected (782 killed and 18 timed out,
    against 783 and 17 in the original run,
    a timing difference),
   238 survived, 60 without coverage, with mutant ids matching this report.
   The sweep's baseline control passes all 36 sidecar files on this machine,
   and a sweep of mutants 34, 226, 283, and 511 reproduces their verdicts here
   (detected; machine-local only; survived; `known-defect-mutation` only).
   The differential task's controls hold at 1000 cases per category (baseline stable, mutant 34 separated);
   it finds no difference for 283, as argued, and none for 511 either,
   which the pinned test detects: its six categories never draw a drop-all filter at a key the target lacks,
   so a "no difference" there is bounded by those generators, not proof of equivalence.
- Coverage gate refrozen after the widened generators: 1765 of 1772 dist lines (was 1676);
   `defaultMetaDataUpdaterFast` is still never called.
- `README.md` lists the `mutation-*` tests and tasks.
- Generators: the listed gaps were closed by a follow-up fork (commits `1d2cc683f` to `035805aa8`),
   each new branch with a reach count and a control that must fail.
   It found one more intent question (into never passes a target-only Map entry to a custom function);
   a probe of the still-unexercised into slot writes found another
   (`actions.defaultMerge` written into the slot is ignored by the array, Set, and Map functions).
   Both are pinned in `known-defect-options.unit.test.ts`.
- Combined issue draft:
   all four findings in `Findings`,
   plus both intent questions,
   added after independent reproduction on 8.0.2.

## Not exercised

- TypeScript types: `src/types/**` has no runtime mutants, and Stryker does not mutate type-level code.
- Higher-order mutants and mutators outside Stryker's default set.
- The sidecar's own oracle (`src/model.ts`, `src/shape.ts`) was not mutation tested,
   so a lenient oracle can still hide a surviving behaviour.
- The 17 Stryker timeouts were counted as detected and not inspected.
- Upstream's rollup build: mutants ran as esbuild bundles,
   justified only by the unmutated esbuild bundle passing every sidecar file.
- Environments with a modified `Object.prototype` or patched wrapper `Symbol.toStringTag`,
   where the "environment" equivalents above could differ.
