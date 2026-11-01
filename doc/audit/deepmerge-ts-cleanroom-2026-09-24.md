# deepmerge-ts 8.0.2 clean-room docs oracle

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/deepmerge-ts.fuzz` is now `package/upstream-fork/deepmerge-ts.fuzz`.

## Purpose

This audit checks npm `deepmerge-ts@8.0.2` against a specification derived only from its documentation.
It is independent of the implementation-derived model kept elsewhere in this repo:
no file under `package/module/deepmerge-ts.fuzz/`, `doc/handover/deepmerge-ts-*`, `doc/audit/deepmerge-ts-*`,
or any `*.local.*` file was read,
and implementation source was read only after every expectation had been written and executed.

## Method

### Sources used for expectations

- `README.md` (identical to the copy shipped in the npm tarball).
- `docs/API.md` and `docs/deepmergeCustom.md` from `RebeccaStevens/deepmerge-ts` at commit `17fc99c`
  (the tarball ships no `docs/`; the changelog tops out at 8.0.2, so no later release is documented).
- `CHANGELOG.md` (identical to the shipped copy).
- TSDoc comments and declared types in the shipped `dist/index.d.mts`.

### Strength labels

- Stated: the docs say it directly; a mismatch counts.
- Implied: a doc example, type declaration, or changelog entry implies it; a mismatch needs judgement.
- Probe: the docs are silent or ambiguous; the outcome is recorded and never counted.

### Execution

- Scratch project: `~/temp/agent/deepmerge-ts-cleanroom-2026-09-24/cleanroom/` inside a `--depth 1` clone
  with pushes disabled.
- Every run used `podman run --memory=2g --cpus=2 --rm --init --network=none` on `docker.io/library/node:24-slim`.
- Type checks used TypeScript 5.9.3 with `strict`, `module: nodenext`.
- Runtime harness: hand-written runner with a structural equality check
  that compares prototypes, own enumerable string and symbol keys, key presence, and `Set`/`Map` order.
- Reference model: `refMerge` in `lib.mjs`, a direct transcription of the default-merge rules in
  "Derived specification", used for kind matrices and property checks.

### Case counts

- 1329 registered runtime cases, of which 3 are positive controls.
  They include a 512-case kind matrix (16 value kinds, every ordered pair, at root and nested),
  a 343-case three-input kind-change matrix,
  and 16 property checks of 400 seeded random trials each (6400 trials).
- 35 type-level expectations checked with `tsc`, plus 1 type control.
- 50 follow-up executions after reading the source
  (30 `deepmergeInto` kind-selection runs, 13 probe lines, 1 `ObjectType` import, 6 type probes);
  their expectations still come from the docs.

## Derived specification

### Default merge by kind

- Values are classified as records ("vanilla" objects), arrays, sets, maps, or others (`docs/API.md`).
- Records: result has the union of enumerable own keys, including symbol keys;
  each key's values are merged recursively.
- Records: a key whose only values are `undefined` stays present with value `undefined` (README example `prop4`).
- Arrays: concatenated in argument order; elements are never merged element-wise (README example).
- Sets: union (README example).
- Maps: union of keys; colliding values are deep merged (CHANGELOG 8.0.0), later leaf values win (README example).
- Others (primitives, `null`, `Date`, `RegExp`, functions, class instances, boxed primitives, typed arrays, `Error`):
  the last value wins (`mergeOthers`, `DeepMergeLeaf`).
- Mixed kinds (for example a map with an array, or a record with a number): handled by `mergeOthers`, so last wins,
  even when records sit on both sides of the other value (`deepmerge({a:{x:1}}, {a:5}, {a:{y:2}})` is `{a:{y:2}}`).
- `deepmerge` returns a new object when it merges containers (README contrasts it with `deepmergeInto`).
- Lone values (present in one input) go to `mergeOthers` and are returned as-is (CHANGELOG 3.0.0).
- Module namespace objects are records (CHANGELOG 4.1.0).
- Inputs are not mutated.

### Argument order and arity

- Later arguments win for leaves; arrays, sets, maps, and record keys keep first-appearance order.
- All values of a key are passed to a merge function at once ("smart merge"),
  so a custom `mergeOthers` sees every value.
- Zero inputs: `undefined` (`DeepMergeHKT<[]>` is `undefined`).
- One input: that input (`DeepMergeHKT<[T]>` is `T`).
- Many inputs (tested with 50 and 200): all merged.

### Undefined and null handling

- `undefined` values are filtered out before merging by default, so `undefined` never replaces a defined value
  (CHANGELOG 6.0.0, `filterValues` docs).
- `null` is kept and replaces earlier values.
- Filtering also applies at the root (`FilterValuesHKT` is applied to the whole argument tuple).
- Array and set elements that are `undefined` are kept: filtering acts on values being merged, not on elements.

### Keys

- Only enumerable keys are merged (CHANGELOG 1.1.1); symbol keys are included (`getKeysOfObjects` docs).
- Keys named like `Object.prototype` members (`toString`, `constructor`) are ordinary data.
- Prototype pollution is prevented in the standard variants (README "Security safeguards", CHANGELOG 4.0.2).

### `deepmergeInto`

- Mutates and keeps the identity of the target; returns `undefined`.
- The target ends up equal to the merge result of the target followed by the inputs (README "Merging into a Target").
- Non-target inputs, including their nested containers, are not mutated (CHANGELOG 8.0.0).
- Root targets can be records, arrays, sets, or maps.
- Types: the target is asserted to the merged type; `deepmergeIntoCustom` makes no assertion (README).

### `deepmergeCustom` and `deepmergeIntoCustom` options

- `mergeRecords`, `mergeArrays`, `mergeSets`, `mergeMaps`: `false` means that kind is not merged (last wins);
  a function replaces the default for that kind and receives only values of that kind.
- `mergeOthers`: receives everything else, including mixed kinds.
- `filterValues`: `false` keeps `undefined`; a function replaces the default filter and receives `(values, meta)`.
- `enableImplicitDefaultMerging`: returning `undefined` from any custom merge function means default merging;
  without it, `undefined` is the merged value.
- `utils`: `mergeFunctions` (custom or default), `defaultMergeFunctions`, `metaDataUpdater`,
  the customized `deepmerge`,
  `useImplicitDefaultMerging`, `filterValues` (the configured function), `maxDepth` (the configured limit), `actions`.
- Into variants: merge functions mutate `target.value`;
  `values` includes the target's value when there is one;
  only `actions.defaultMerge` exists (no `skip`);
  there is no implicit default merging, so returning `undefined` leaves the target slot as the function left it.

### Actions

- `actions.skip` omits the property from the result (docs example with `skipme` dates).
- `actions.defaultMerge` is equivalent to calling the default merge function for that kind.

### Metadata

- Built-in metadata carries the key the values were under (`meta.key`), including symbol keys.
- Root-level merges receive `rootMetaData` (default `undefined`).
- `metaDataUpdater(previousMeta, mergeInfo)` produces the metadata for nested merges;
  `mergeInfo` has `key`, `parents` (the inputs being merged at that step), `values` (candidates for `key`), `result`.
- Doc examples with fixed outputs: sum/product/mean gives `{ sum: 22, product: 880, mean: 7.5 }`;
  the key-path example gives the exact nested output shown in `docs/deepmergeCustom.md`.

### `maxDepth`

- Default `1000`; configurable; nested objects beyond the limit are not deeply merged, and merging does not throw.
- The limit prevents stack exhaustion on untrusted input (tested with 100000-deep record, map, and mixed chains).

### Circular references

- Supported by default (README); merging cyclic inputs does not throw.
- A custom `mergeCircularReferences` receives the values and one cyclic depth per value.
- Shared but acyclic references are not circular and merge normally.

### FastUnsafe variants

- Same results as the standard variants on trusted, acyclic data.
- No circular detection (cyclic input overflows the stack), no `maxDepth`,
  no metadata (merge functions get `undefined`),
  no prototype-pollution interception.
- Options exclude `metaDataUpdater`, `maxDepth`, and `mergeCircularReferences` (type level and runtime).

### Utility functions

- `getKeysOfObjects(objects)` and deprecated `getKeys(objects)`: a `Set` of all enumerable keys, symbol keys included.
- `getObjectType(value)`: one of `NOT`, `RECORD`, `ARRAY`, `SET`, `MAP`, `OTHER`.
- `objectHasProperty(object, property)`: whether the object has the property.
- `ObjectType`: listed under "Utility Functions ... exported for use in custom merge functions".

### Type-level results

- Records merge key-wise; arrays concatenate (tuples stay tuples); sets and maps union their element types.
- Mixed kinds and others resolve to the last type; `undefined` is filtered; zero inputs give `undefined`.
- Non-tuple spreads give `unknown` (`docs/API.md`).
- Every custom-return-type example in `docs/deepmergeCustom.md` produces the shown result type.
- README: "Merged output has correct typing."

## Controls

- Runtime control: `deepmerge({a:1},{a:2})` expected to equal `{a:1}`; it failed as required.
- Harness sensitivity control: a first-wins model was distinguished from the library within 400 seeded trials.
- Mutation-detector control: a deliberate nested mutation was detected by the snapshot comparison.
- Type control: `Equal<typeof deepmerge({a: number}, {a: string}), {a: number}>` produced `TS2344` as required.

## Results overview

- Stated cases: 1225 passed, 2 failed (both are my misreadings, listed under "Own misreadings").
- Implied cases: 89 passed, 1 failed (M4).
- Probe cases: 7 passed, 2 differed (recorded under "Ambiguities").
- Types: 34 of 35 expectations held; T19 differed (ambiguity), and a follow-up found M3.
- Source-informed follow-ups found M1 and M2.
- Every prototype-pollution case passed for the standard variants
  (45 payload and entry-point combinations plus a `Map` key case).
- Every stack-exhaustion case passed for the standard variants.

## Mismatches

### M1: `deepmergeInto` keeps only the last value when the first slot is `undefined`

Classification: defect.
The docs say `undefined` never replaces defined values and that the target receives the merge result;
`deepmerge` on the same inputs merges both records.
Affects `deepmergeInto`, `deepmergeIntoCustom`, and `deepmergeIntoFastUnsafe`, for records, arrays, sets, and maps.

```js
// ~/temp/agent/deepmerge-ts-cleanroom-2026-09-24/cleanroom/probe/into-kind.mjs (reduced)
import { deepmerge, deepmergeInto } from "deepmerge-ts";

const t1 = {};
deepmergeInto(t1, { k: undefined }, { k: { x: 1 } }, { k: { y: 1 } });
console.log(t1); // { k: { y: 1 } }, docs imply { k: { x: 1, y: 1 } }

const t2 = { k: undefined };
deepmergeInto(t2, { k: [1] }, { k: [2] });
console.log(t2); // { k: [2] }, docs imply { k: [1, 2] }

console.log(deepmerge({ k: undefined }, { k: [1] }, { k: [2] })); // { k: [1, 2] }
```

Source explanation: `mergeUnknownsInto` in `dist/index.mjs` picks the container kind from `mut_target.value`,
which is the target's value or `emptyLike(propValues[0])` built from the unfiltered first value.
When that value is `undefined`, the kind is `NOT`,
so the call falls through to `mergeOthersInto` and keeps the last value.
`mergeUnknowns` (non-Into) reads the kind from `filteredValues[0]` instead, which is why `deepmerge` is correct.

### M2: `deepmergeInto` merges a non-record's own properties into a record

Classification: defect.
The docs say mixed kinds are handled by `mergeOthers` (last wins) and records are only vanilla objects.

```js
// ~/temp/agent/deepmerge-ts-cleanroom-2026-09-24/cleanroom/probe/into-kind.mjs (reduced)
import { deepmerge, deepmergeInto } from "deepmerge-ts";

class K {
  constructor() {
    this.v = 1;
  }
}
const t = {};
deepmergeInto(t, { k: new K() }, { k: { x: 1 } });
console.log(t); // { k: { v: 1, x: 1 } }, docs imply { k: { x: 1 } }
console.log(deepmerge({ k: new K() }, { k: { x: 1 } })); // { k: { x: 1 } }
```

Also reproduced with an `Error` carrying an own `code` property, and with three inputs.
Source explanation: when the target lacks the key, `emptyLike` returns `{}` for any non-container object,
so the slot's kind is `RECORD`; the mixed-kind check then compares only values from index 1 onward against that kind,
so the first value's kind is never checked.

### M3: optional nested record types claim properties that can be missing

Classification: defect (type level), against README "Merged output has correct typing".

```ts
// ~/temp/agent/deepmerge-ts-cleanroom-2026-09-24/cleanroom/probe/optional-unsound.ts (reduced)
import { deepmerge } from "deepmerge-ts";

const r = deepmerge({} as { a?: { x: number } }, { a: { y: "s" } });
// Declared type: { a?: { x: number; y: string } | undefined }
// Runtime value: { a: { y: "s" } }, so r.a.x is typed number but is undefined.
const x: number = r.a!.x;
```

Explanation from `dist/index.d.mts` and the observed types: the merged value type combines the value types
of every input that declares the key, without a branch for the optional declaration being absent,
and the key's optionality comes from the first declaring input
(`DeepMergeRecordPropertyMetaDefaultHKTGetOptional` reads only `First`).
The same rule makes `deepmerge({} as { a?: number }, { a: "x" })` typed `{ a?: string }`,
which is loose but not unsound (listed under "Ambiguities").

### M4: TSDoc example calls default array merging "last-wins"

Classification: docs error.
The `deepmergeCustom` TSDoc example in `dist/index.d.mts` says
"Merge arrays by concatenation instead of last-wins" and passes `mergeArrays: (values) => values.flat()`.
The README example, `DeepMergeArraysDefaultHKT`, and runtime all concatenate by default,
so the example's custom function reproduces the default.

```js
import { deepmerge } from "deepmerge-ts";

console.log(deepmerge({ tags: ["a"] }, { tags: ["b"] })); // { tags: ["a", "b"] }
```

### M5: `ObjectType` is documented as an exported utility but has no runtime export

Classification: docs error (or packaging defect; the docs do not say it is type-only).
`docs/API.md` lists `ObjectType` under "Utility Functions",
"exported for use in custom merge functions".
The declarations use `export declare const enum ObjectType`, and `dist/index.mjs` does not export it.

```js
// ~/temp/agent/deepmerge-ts-cleanroom-2026-09-24/cleanroom/probe/objecttype.mjs
import { ObjectType, getObjectType } from "deepmerge-ts";
// SyntaxError: The requested module 'deepmerge-ts' does not provide an export named 'ObjectType'
console.log(getObjectType({}) === ObjectType.RECORD);
```

TypeScript accepts the same code only without `isolatedModules`, where the const enum is inlined;
with `isolatedModules` it reports `TS2748: Cannot access ambient const enums when 'isolatedModules' is enabled.`

## Own misreadings

- The `skipme` example (`docs/deepmergeCustom.md`) relies on `meta.key`.
  I also ran it through `deepmergeFastUnsafeCustom`, whose docs say metadata is bypassed,
  so `meta` is `undefined` and nothing is skipped. The library matches its docs.
- I expected `deepmergeFastUnsafe` to merge 1500-deep chains because it has no `maxDepth`.
  The docs say deeply nested input overflows the stack in the fast variants.
  In the container the fast variant overflowed above 1562 levels; the standard variant handled 20000 (search cap).

## Ambiguities and observations (not counted)

- Objects whose prototype is a plain object (`Object.create({ inherited: 1 })`) and null-prototype objects
  are merged as records; the result has `Object.prototype`, so the original prototype is dropped.
  `getObjectType(Object.create(null))` is `RECORD`.
- Array and `Set` subclass instances merge into plain `Array` and `Set` results.
- `deepmerge(x)` returns `x` itself, and lone nested containers are returned by reference,
  consistent with CHANGELOG 3.0.0; a frozen single input therefore yields a frozen result.
- Returning `actions.skip` at the root returns the internal symbol `Symbol(deepmerge-ts: skip)` to the caller.
- `deepmergeInto` with a root kind mismatch (`deepmergeInto({a:1}, [1])`), or with `mergeRecords: false`,
  silently leaves the target unchanged, while the type assertion still claims the merged type.
- `deepmergeInto` into a frozen target throws `TypeError`.
- A `filterValues` function returning `[]` for a key leaves the key present with `undefined`.
- `utils.maxDepth` is `1000` under default options and `undefined` in FastUnsafe variants.
- Built-in metadata objects carry `key`, `parents`, `values`, `result`, and `hierarchy`.
- Default circular handling rebuilds the cycle in the result: for self-cycles, `result.self === result`.
- A `__proto__` own key (from `JSON.parse`) becomes an own data property of the result; no prototype changes.
- `getObjectType(null)` and `getObjectType(function)` are `NOT`.
- `objectHasProperty` is true only for own enumerable properties (inherited and `toString` are false).
- Record, map, and set key order follows first appearance across inputs (probe passed).
- Smart merge differs from a classic pairwise fold when kinds change mid-list
  (for example a record, then `0`, then two records): this follows from the documented mixed-kind rule.
- `deepmerge({} as { a?: number }, { a: "x" })` is typed `{ a?: string }` although `a` is always present.

## Areas not tested

- The README caveat about `interface` types (microsoft/TypeScript#15300).
- TypeScript versions other than 5.9.3, including TypeScript 7 (CHANGELOG 8.0.1).
- The CommonJS entry point, the jsr package, Deno, and Bun.
- Performance claims (smart merge speed and memory versus classic merge).
- Exact `maxDepth` boundary: only 990 levels (merged), 1500 levels (not merged), and `maxDepth: 3` at depth 10.
- Numeric meaning of `cyclicDepths`, the `hierarchy` metadata, and `mergeInfo.result`.
- The `CustomizedDeepmergeInto` assertion-type example in `docs/deepmergeCustom.md` (not compiled).
- Prototype pollution in the FastUnsafe variants (documented as unprotected).
- Cross-realm objects and module namespaces beyond a single fixture module.

## Embargo

One embargoed item exists. It is not described in this report.
