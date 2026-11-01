# deepmerge-ts unsearched surfaces, 2026-09-24

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/deepmerge-ts.fuzz` is now `package/upstream-fork/deepmerge-ts.fuzz`.

Unsearched-surfaces workstream of the method audit
(`doc/handover/deepmerge-ts-hardening.md`,
 section `Method audit`).
Every earlier layer checked runtime values or result types under one tsconfig;
this pass searched deepmerge-ts 8.0.2 for defects on the surfaces none of them reached:
upstream's documentation,
 compiler variation,
 type-checker cost,
 packaging,
 and other runtimes.
Each check has a positive control that shows it can fail (`Controls`).

Target:
 the npm release 8.0.2,
with docs read at upstream `main` `17fc99cb`,
whose `src/` and `README.md` are checksum-identical to the JSR release `@rebeccastevens/deepmerge` 8.0.2
(compared against `https://jsr.io/@rebeccastevens/deepmerge/8.0.2_meta.json`).
Heavy runs used `podman run --memory=2g --cpus=2 --rm`.

## Result

- Docs conformance:
   four TypeScript snippets in the docs do not type-check against the published declarations
   on any release from 4.7 to 7.0,
   four behavioural statements do not match the build,
   and the documented `ObjectType` export does not exist at runtime.
   Every documented runtime result holds.
- Compiler variation:
   the declarations compile with `skipLibCheck: false` on every release from 4.7.4 to 7.0.2,
   and every pinned result type holds on each release where the harness compiles.
   New result-type divergences:
   read-only collection types that are not `Set` or `Map` instances,
   a fixed argument followed by a spread,
   and every `null` under `strictNullChecks: false`.
- Type-checker cost:
   linear in width,
   depth,
   tuple length,
   union size,
   and `deepmergeInto` width;
   superlinear in the number of statically typed arguments,
   with TS2589 at 200 literal arguments or 50 recursive-typed ones.
- Packaging:
   publint and `@arethetypeswrong/cli` report nothing;
   the engines floor (Node 16.9.0) holds for both builds.
- Runtimes:
   the whole bounded sidecar layer passes under Bun 1.4.2,
   Deno 2.9.7,
   and against the CJS build on Node.
- One embargo candidate,
   recorded only in `doc/handover/deepmerge-ts-embargo-surface.local.md`.

## Findings

### Defect: `ObjectType` is declared as a value export that no build provides

- `docs/API.md` "Utility Functions" lists `ObjectType` among the utilities
   "exported for use in custom merge functions".
- `src/index.ts` re-exports it as `type ObjectType` (a `const enum` in `src/utils.ts`),
   so neither `dist/index.mjs` nor `dist/index.cjs` exports it,
   while `dist/index.d.mts` and `dist/index.d.cts` declare `export declare const enum ObjectType`.
- Consequences,
   each reproduced:
   plain `tsc` inlines the members and works;
   `isolatedModules` and `verbatimModuleSyntax` report TS2748 on 6.0.2 and 7.0.2;
   Node type stripping fails at load
   (`SyntaxError: The requested module 'deepmerge-ts' does not provide an export named 'ObjectType'`);
   `esbuild --bundle` fails
   (`No matching export in "node_modules/deepmerge-ts/dist/index.mjs" for import "ObjectType"`);
   CJS reads `undefined` and throws on `.OTHER`;
   and JSR consumers,
   who get the source types,
   see TS1362 (`exported using 'export type'`).
- publint and `@arethetypeswrong/cli` do not check value exports against ESM declarations,
   so neither reports it.

```ts
// enum-use.ts, from API.md's description
import { deepmergeCustom, getObjectType, ObjectType } from "deepmerge-ts";

const merge = deepmergeCustom({
  mergeOthers: (values, utils) =>
    values.every((value) => getObjectType(value) === ObjectType.OTHER) ? values : utils.actions.defaultMerge,
});
console.log(merge({ d: new Date(0) }, { d: new Date(1) }));
```

### Defect: read-only collection types that are not instances merge as `Set` and `Map` in the type only

- `IsSet` and `IsMap` in `index.d.mts` test `extends ReadonlySet` and `extends ReadonlyMap`;
   `getObjectType` (`src/utils.ts`) tests `instanceof Set` and `instanceof Map`.
- A value typed `ReadonlySet<T>` or `ReadonlyMap<K, V>` that is not an instance
   (a read-only view,
   or a collection class from an observable or immutable library)
   is typed as a merged `Set<A | B>` or `Map<K, A | B>`,
   while the runtime keeps the last value by reference,
   so `add` and `set` type-check and throw.

```ts
// view.ts
import { deepmerge } from "deepmerge-ts";

class Bag<T> implements ReadonlySet<T> { /* each member delegates to a private Set; run in full on 8.0.2 */ }
const merged = deepmerge({ tags: new Bag(["a"]) }, { tags: new Bag([1]) });
// merged.tags: Set<string | number>; runtime: the second Bag
merged.tags.add(2); // TypeError: merged.tags.add is not a function
```

- Related imprecision:
   with a lib before ES2025 (every `target` up to ES2024,
   and `esnext` on TypeScript 5.4 and older;
   5.6 has the methods,
   5.5 was not installed),
   `ReadonlySet` lacks the ES2025 set methods,
   so a `Map` type is structurally a `ReadonlySet<unknown>`;
   identical `Map` input types then skip the same-type shortcut
   and come back as `Map<K, V | V>` with two structurally identical value types.
   Sound,
   but `expectTypeOf(...).toEqualTypeOf<Map<K, V>>()` fails
   (`src/type-collection.unit.test.ts` on TypeScript 4.7 to 5.4).

### Defect: a fixed argument followed by a spread array is typed as the fixed argument alone

- `docs/API.md` says inputs that are not a tuple type make the result `unknown`.
   `[First, ...Rest[]]` passes `IsTuple`,
   so the result is typed from `First` alone;
   `deepmergeInto(target, ...rest)` keeps the target type for the same reason.

```ts
// spread.ts
import { deepmerge } from "deepmerge-ts";

const rest: { b: string; list: string[] }[] = [{ b: "x", list: ["s"] }];
const merged = deepmerge({ a: 1, list: [1] }, ...rest);
// { a: number; list: number[] }; runtime { a: 1, list: [1, "s"], b: "x" }
```

### Intent question: result types under `strictNullChecks: false`

- `DeepMergeLeafElement` filters `E extends undefined ? never : E`;
   with `strictNullChecks` off `null extends undefined` holds,
   so every `null` is filtered in the type while the runtime keeps it.
- `deepmerge({ a: { x: 1 } }, { a: null })` is typed `{ a: { x: number } }` (runtime `{ a: null }`),
   and `deepmerge({ a: 1 }, null)` is typed `{ a: number }` (runtime `null`),
   so `merged.a.x` compiles and throws.
- Upstream states no `strict` requirement.
   The other differences under `strict: false` come from `undefined` being unrepresentable
   and are not specific to deepmerge-ts.

### Intent question: interface-typed inputs

- README "TypeScript Interfaces" says interfaces "may not appear to merge correctly".
   The result type is the last input alone:
   `deepmerge(config as Config, { list: ["s"] })` with `interface Config { list: number[]; name: string }`
   is `{ list: string[] }`,
   while the runtime value is `{ list: [1, "s"], name: "a" }`.
   The type drops keys and claims string elements for an array holding numbers,
   which is more than the caveat suggests.

### Docs errors: snippets the declarations reject

Each fails identically on TypeScript 4.7.4,
 6.0.2,
 and 7.0.2 with `--strict --skipLibCheck false`:

- `docs/deepmergeCustom.md` dates example and "Customizing the Meta Data" example write `DeepMergeLeaf<Ts>`;
   the declaration requires three type arguments (TS2314).
- "Customizing the Meta Data":
   the key-path `metaDataUpdater` returns `{ keyPath: ({} | null)[] }`
   because `DeepMergeMergeInfo.key` is `unknown`,
   not `PropertyKey` (TS2322).
- `src/types/options.ts` `MetaDataUpdater` TSDoc example
   calls `deepmergeCustom({ metaDataUpdater })` without type arguments,
   so `path` exists on neither side (TS2769,
   TS2339).
- "Deepmerge Into Custom" `CustomizedDeepmergeInto` omits `DeepMergeFilterValuesURI`,
   which `DeepMergeFunctionsURIs` requires (TS2344;
   TS2741 on 7.0),
   and uses `DeepMergeHKT` and `DeepMergeFunctionsDefaultURIs` without importing them.

### Docs errors: statements the build contradicts

- `src/deepmerge.ts` `deepmergeCustom` TSDoc example says
   "Merge arrays by concatenation instead of last-wins";
   the default `mergeArrays` already concatenates,
   so the example changes nothing.
- `objectHasProperty` "Returns whether the given object has the given property";
   it returns `false` for own non-enumerable properties and for any function object
   (`typeof object === "object" && propertyIsEnumerable`).
- FastUnsafe docs and TSDoc:
   "Circular structures will result in a stack overflow."
   One cyclic input merged with plain ones returns normally (passed through by reference);
   the overflow needs cyclic values in two inputs at one position.
- The FastUnsafe custom variants are documented as taking the same options minus three;
   they also always pass `meta` as `undefined` (typed and at runtime),
   so `deepmergeCustom.md`'s `meta`-based examples neither compile nor act there.
   The docs say only "No metadata tracking".

### Intent question: type-checker cost grows superlinearly with argument count

- `deepmerge` with N literal record arguments:
   instantiations 26118 at 25,
   131218 at 50,
   833293 at 100,
   2607868 at 150,
   4053593 at 175
   (about N to the power 2.6),
   and TS2589 ("Type instantiation is excessively deep and possibly infinite") at 200,
   on 6.0.2 and 7.0.2.
- 45 arguments of distinct recursive types check;
   50 give TS2589 when the result is read.
- Separate from the TS2589 case the recall workstream pinned
   (a 50-member union with `DeepMergeNoFilteringURI`,
   `src/recall-known-defect-union-depth.unit.test.ts`):
   these cases use default options and grow with argument count or recursive input types.
- The runtime has no such limit;
   `deepmerge(...array)` is typed `unknown` and costs nothing.
- Linear families,
   for comparison (6.0.2):
   width 1600 keys 127856 instantiations,
   depth 70 levels 53277,
   `deepmergeInto` width 800 keys 175486 (check time 1.6 s,
   against 0.5 s for `deepmerge` at 800),
   tuples of 800 elements 3984,
   50 Sets 13401,
   unions of 64 members 9431.
   TypeScript 7.0.2 checks the largest of these in under 0.1 s.
- A 100-level nested literal crashes TypeScript 5.9 and 6.0 (`Debug Failure`) and gives TS2321 on 7.0,
   but so does `declare function pair<A, B>(a: A, b: B): [A, B]`,
   so that limit belongs to TypeScript.

## Checked without finding a defect

- Every runtime result printed in README.md and `docs/deepmergeCustom.md`
   (default config,
   `mergeArrays: false`,
   dates,
   `filterValues: false`,
   `null` filter,
   sum,
   product,
   and mean,
   key path,
   `skipme`),
   the three "Default Merging" variants on seven input shapes,
   `deepmergeInto` values including the target first,
   into `actions` holding only `defaultMerge`,
   `rootMetaData` reaching root merges,
   `utils.deepmerge` and `utils.deepmergeInto` being the customized functions,
   default filtering (`undefined` dropped,
   `null` kept),
   mixed kinds going to `mergeOthers`,
   `getKeysOfObjects` (enumerable string and symbol keys),
   `getObjectType` numbering,
   and FastUnsafe `enableImplicitDefaultMerging` and `actions.skip`.
- Declarations under `isolatedModules`,
   `verbatimModuleSyntax`,
   `noUncheckedIndexedAccess`,
   and `moduleResolution` `node10`,
   `node16`,
   `nodenext`,
   and `bundler`
   (both the `.d.mts` and `.d.cts` paths),
   with the whole type corpus (1000 inferred and 439 declared cases).
- `publint --strict` (0.3.24):
   "All good".
   `@arethetypeswrong/cli --from-npm` (0.18.5):
   no problems for node10,
   node16 from CJS and ESM,
   and bundler.
   `dist/index.d.cts` is byte-identical to `dist/index.d.mts`.
- `engines: >=16.9.0`:
   the builds use `Object.hasOwn` and `Array.prototype.at` (both in Node 16.9);
   an ESM and a CJS smoke run of all eight entry points and the utilities
   print identical output on Node 16.9.0,
   18,
   20,
   22.19,
   24,
   and 26.
- The bounded sidecar layer (every `src/*.unit.test.ts`) under Bun 1.4.2,
   Deno 2.9.7,
   and on Node against `dist/index.cjs`.

## Controls

- Docs runtime tests:
   a copy of `dist/index.mjs` whose default `mergeArrays` reverses its output
   fails the README and TSDoc-concatenation tests.
- Docs type pins:
   each `@ts-expect-error` would report TS2578 if the declarations accepted the snippet;
   none does on any release from 4.7.4 to 7.0.2.
- Surface pins (`src/surface-known-defect.unit.test.ts`):
   a copy exporting `ObjectType` fails the first test,
   a copy classifying set views as Sets fails the collection test,
   and the reversing copy fails the spread and interface tests.
- Compiler matrix:
   `base` on 6.0.2 is clean;
   the same files on 4.7.4 report the lib-dependent `Map` case and harness errors,
   so the matrix reports diagnostics it finds.
- Cost:
   seven runs of the 400-key case gave identical instantiation counts
   and check times from 0.16 s to 0.26 s,
   so only instantiation counts and differences well past that band are read.
- `@arethetypeswrong/cli`:
   0.18.2 failed on every package,
   including the control `deepmerge@4.3.1`;
   0.18.5 reports that control's known "Named exports" problem,
   so its clean result for deepmerge-ts counts.
- Runtimes:
   the reversing CJS copy fails the same 13 files on Node,
   Bun,
   and Deno
   (16 once the surface tests existed),
   and the unmodified CJS build passes all of them.

## New tests and scripts

In `package/module/deepmerge-ts.fuzz/src/`:

- `surface-known-defect.unit.test.ts` with `surface-collection-view.ts`:
   `ObjectType`,
   collection views,
   spread arguments,
   interface inputs.
- `surface-docs-example.unit.test.ts`:
   documented runtime results.
- `surface-docs-claim.unit.test.ts`:
   behavioural statements,
   conformance and docs errors.
- `surface-docs-type.unit.test.ts`:
   snippets the declarations reject.
- `surface-toolchain.ts`,
   `surface-type-matrix.ts`,
   `surface-type-cost.ts`,
   `surface-runtime.ts`:
   container scripts for the compiler matrix,
   cost sweeps,
   and runtimes.
- Machine-local:
   `surface-embargo.local.unit.test.ts`.

Reproduction,
 from the package directory,
with this repo mounted read-only at its host path and `dist/surface` writable:

```sh
# package/module/deepmerge-ts.fuzz, inside docker.io/library/node:26-slim
node src/surface-toolchain.ts dist/surface/scratch
(cd dist/surface/scratch && npm install)
node src/surface-type-matrix.ts "$PWD/dist/surface/scratch" base
FLAGS=base,isolated,verbatim,node10,node16,nodenext,no-unchecked-index,no-exact-optional,strict-false,no-strict-null
node src/surface-type-matrix.ts "$PWD/dist/surface/scratch" "$FLAGS" ts60
node src/surface-type-cost.ts "$PWD/dist/surface/scratch" ts60
# inside docker.io/oven/bun:1 and docker.io/denoland/deno:latest
bun src/surface-runtime.ts bun
DENO_RUN="deno run --allow-all --no-lock --node-modules-dir=manual"
$DENO_RUN src/surface-runtime.ts $DENO_RUN
```

## Requested shared-file changes

- `mise.toml`:
   capped tasks `surface:install`,
   `surface:types`,
   `surface:cost`,
   and `surface:runtime` (Bun and Deno)
   wrapping the reproduction commands,
   with the repo mounted read-only and `dist/surface` writable.
- `README.md`:
   a "What is checked" entry for the `surface-*` files and a command block for the tasks.
- `doc/handover/deepmerge-ts-issue.local.md`:
   sections for the public findings in `Findings`.

## Not exercised

- Docs:
   the codesandbox example,
   CHANGELOG statements,
   the performance claims in README "Performance",
   and JSR's rendered symbol pages (their text is the TSDoc checked here).
- Compiler:
   TypeScript 5.1,
   5.3,
   5.5,
   and 5.7;
   the declared-type corpus without `exactOptionalPropertyTypes` (its own declarations need it,
   TS2411);
   `lib` below ES2022;
   `module: preserve`;
   consumers emitting declarations of inferred merge results (`declaration` and `isolatedDeclarations`).
- Cost:
   custom HKT URIs beyond one,
   many-argument `deepmergeInto`,
   and editor latency in `tsserver`.
- Packaging:
   bundlers other than esbuild,
   Yarn Plug'n'Play,
   and package managers other than npm.
- Runtimes:
   browsers,
   edge runtimes,
   Node before 16.9.0 (outside `engines`),
   and the unbounded campaign under Bun and Deno.
