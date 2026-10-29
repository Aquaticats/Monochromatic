# tsdown DTS generation dominates build time (340-of-380 ms on module-es) because `rolldown-plugin-dts` re-bundles declarations through a single-threaded JS load hook

## Symptom

The [bundler-benchmark](https://github.com/gugustinette/bundler-benchmark)
shows tsdown completing builds in ~36 ms,
 but every package in
this monorepo takes 360+ ms. Even a 201-line CLI plugin
(`terminal-title`) takes 133 ms.

Wall-clock measurements feel anomalous compared to the published
benchmark;
 "tsdown is slow on our packages" is the surface
complaint.
 The root cause is that the benchmark disables the
features (DTS generation,
 minify,
 target lowering) that the
workspace requires.

## Root cause

DTS generation accounts for ~340 ms of the ~380 ms total on
`module-es`.
 Minification and `firefox140` target transforms are
negligible (~1 ms combined).

tsdown uses
[`rolldown-plugin-dts`](https://github.com/sxzz/rolldown-plugin-dts)
(by the same author as tsdown and rolldown,
 sxzz).
 This is
distinct from rolldown's builtin `isolatedDeclarationPlugin`:

**Builtin `isolatedDeclarationPlugin`** (Rust,
 parallel):

- Runs OXC `IsolatedDeclarations` per-file inside the
  `transform_ast` hook,
   fully in Rust.
- Emits each `.d.ts` as a **separate asset** via
  `ctx.emit_file()`.
- No bundling;
   output mirrors the source module graph.

**`rolldown-plugin-dts`** (JS plugin,
 what tsdown uses):

- Generates `.d.ts` per-file via OXC `isolatedDeclarationSync` (a
  sync NAPI call into native OXC).
- Feeds those `.d.ts` files back into rolldown as a **second
  bundle pass** via the `load` hook.
- Rolldown resolves imports between declarations,
   tree-shakes
  unused types,
   and emits a **single bundled declaration file**
  (e.g. `index.d.mts`).

tsdown is a library bundler.
 Its purpose is to produce a
self-contained package with a single JS entry and a single
`.d.ts` entry.
 If it emitted 483 separate `.d.ts` files,
consumers would need the entire internal module structure in
`node_modules`,
 defeating the purpose of bundling.

### Why the second pass is slow

Even though OXC isolated declarations is fast per-file
(sub-millisecond for small type files),
 the DTS bundling pass
has inherent overhead:

1. **JS event loop serialisation.
   ** Rolldown spawns concurrent
   tokio tasks per module (`tokio::spawn` in
   `module_loader.rs`),
    but each task calls the JS `load` hook
   via NAPI `ThreadsafeFunction.call_async()`.
    These callbacks
   execute one-at-a-time on the single JS event loop thread.
   The `isolatedDeclarationSync` call within each callback
   blocks that callback from yielding.
2. **Import resolution fanout.
   ** For `module-es`:
    483 DTS source
   files produce 1138 cross-file DTS import resolutions.
    Each
   resolution goes through the JS-side resolver plugin.
   Rolldown discovers imports incrementally;
    each layer of the
   dependency graph must resolve before the next layer is
   discovered.
3. **Bundle linking.
   ** After all load hooks complete,
    rolldown
   runs its standard linking phase on the DTS module graph to
   produce the final bundled output.

Verified via `DEBUG='rolldown-plugin-dts:*'` output:
 resolved
options confirm `oxc: { stripInternal: false, sourcemap: false
}`.
 OXC is auto-enabled because `tsconfig.json` has
`isolatedDeclarations: true` (detected at
[`rolldown-plugin-dts` options.ts line 944][rdpd]:
`oxc ??= !!(compilerOptions?.isolatedDeclarations && !vue && !tsgo && !tsMacro)`).
No tsc is involved.

[rdpd]: https://github.com/sxzz/rolldown-plugin-dts

## Verification

Version under test:

- tsdown / rolldown / `rolldown-plugin-dts` as pinned by
  `@monochromatic-dev/config-tsdown` at workspace HEAD.
- `module-es` workspace package (483 DTS source files,
   1138
  cross-file DTS import resolutions).
- `terminal-title` workspace package (8 DTS source files,
   201
  source lines).

Measured `module-es` with features toggled (times in ms):

- Full build (DTS + minify + target):
   367-389
- DTS only (no minify):
   365
- No DTS (minify + target):
   34-41
- No DTS,
   no minify:
   33-36

For `terminal-title` (8 DTS source files):
 55 ms with DTS,
 22 ms
without (~33 ms DTS overhead).

The benchmark disables every expensive feature:
 `dts` defaults
to false,
 `minify: false`,
 `target: false`,
`skipNodeModulesBundle: true`,
 and it uses the programmatic API
(no CLI startup).

### Measured timeline for module-es (debug timestamps)

- Config loading + tsconfig resolution:
   ~23 ms
- JS bundle (rolldown,
   Rust-side parallel):
   ~48 ms
- DTS:
   483 OXC calls + 1138 import resolutions (JS event loop):
  ~350 ms
- DTS bundle finalisation + dep detection:
   ~46 ms
- Total (reported by tsdown):
   ~380 ms

## Verified workarounds

### Set `dts: false` for packages that are not consumed as libraries

CLI tools,
 Claude Code plugins,
 and client bundles do not need
`.d.ts` files.
 Saves 33 ms (small packages) to 340 ms (large
packages) per build.

Tradeoff:
 cannot consume the package from another workspace as a
typed library.
 Acceptable for CLI tools and applications;
unacceptable for `module-*` library packages where types are
the public API.

### Reduce the number of source files that need DTS bundling

Cost scales with file count and import-graph depth.
 Flatter
module structures with fewer re-export layers reduce the
resolution fanout.
 Tradeoff:
 contradicts the workspace's
preference for small files split by region;
 the import graph is
shaped by the file layout we already want.

### Watch mode

`tsdown --watch` reuses the rolldown instance and only
re-processes changed files,
 avoiding the full DTS bundling pass
on each save.
 Tradeoff:
 only helps interactive workflows;
 CI
builds still pay the full cost.

## What does not work

- **Switching to tsc for DTS**:
   would be slower,
   not faster.
  The OXC path is already the fastest available.
- **Setting `dts: { oxc: true }` explicitly**:
   redundant;
  `rolldown-plugin-dts` auto-detects `isolatedDeclarations:
  true` from tsconfig.
- **Disabling minification**:
   saves ~1 ms,
   irrelevant.
- **Running with `DEBUG='rolldown-plugin-dts:*'` or
  `DEBUG='tsdown:*'` to measure**:
   debug logging adds
  significant overhead.
   For `module-es`,
   debug logging produces
  26,481 log lines and inflates reported wall time from ~480 ms
  to ~700 ms+ (~300 ms of pure logging overhead).
   Always
  measure with debug disabled.

## Why we do not file this upstream

1. **Is it really upstream's fault?
   ** No. The DTS bundling pass
   exists because library consumers expect a single `.d.ts`
   entry.
    The cost is inherent to the goal,
    not to a
   suboptimal implementation.
2. **Can upstream fix it?
   ** Possibly.
    Moving the load-hook
   fan-out to a Rust-side parallel walker would amortise the
   per-file cost,
    but `isolatedDeclarationSync` is itself a sync
   NAPI call,
    so the benefit depends on whether OXC's
   declaration generation can be invoked from rust threads
   safely.
    Non-trivial.
3. **Are they supporting this use case?
   ** Yes;
    bundled DTS is
   the documented tsdown feature set.
4. **Will they likely fix it?
   ** Maybe.
    sxzz maintains the
   plugin actively;
    performance improvements have landed in
   the past.
5. **Have we prototyped a minimal fix?
   ** No.

Decision:
 no upstream report.
 The cost is intrinsic to the
bundled-DTS feature;
 mitigate at our boundary by setting
`dts: false` for non-library packages.

## Upstream doc bug: default dependency bundling

**tsdown version:
** 0.9+ (current as of 2026-04-04)
**Severity:
** documentation contradiction (misleads users about default behavior)
**Upstream:
** [rolldown/tsdown](https://github.com/rolldown/tsdown)
**Commit tested:
** [`9471001`](https://github.com/rolldown/tsdown/commit/9471001)

### Summary

Three pages on [tsdown.dev](https://tsdown.dev) describe incompatible default behaviors
for how tsdown handles `dependencies`,
 `peerDependencies`,
 and `optionalDependencies`.
Two pages are correct;
 the FAQ is wrong.

### The contradiction

#### FAQ page ([tsdown.dev/guide/faq](https://tsdown.dev/guide/faq))

Source:
 `doc/guide/faq.md:52`

> "By default,
>  tsdown bundles all imported modules.
> To exclude dependencies (e.g.,
>  those listed in `package.json`),
>  use the `deps` configuration"

This claims the default is **bundle everything**,
 and the user must opt in to externalization.

#### Dependencies page ([tsdown.dev/options/dependencies](https://tsdown.dev/options/dependencies))

Source:
 `doc/options/dependencies.md:9`

> "By default,
>  `tsdown` **does not bundle dependencies** listed in your `package.json`
> under `dependencies`,
>  `peerDependencies`,
>  and `optionalDependencies`"

This claims the default is **externalize production dependencies** automatically.

#### How It Works page ([tsdown.dev/guide/how-it-works](https://tsdown.dev/guide/how-it-works))

Source:
 `doc/guide/how-it-works.md:30`

> "`dependencies`,
>  `peerDependencies`,
>  and `optionalDependencies` are **externalized**:
> they appear as `import` / `require` statements in the output and are not included in the bundle.
> "

Consistent with the Dependencies page.
 Production deps are external by default.

### Source code trace proving auto-externalization

The Dependencies and How It Works pages are correct.
The following trace through the source code shows exactly how tsdown
auto-externalizes production dependencies without any user configuration.

#### Step 1: `package.json` is read at config resolution time

`src/config/options.ts:96`:
 `resolveOptions()` reads `package.json` from the working directory:

```typescript
const pkg = await readPackageJson(cwd,);
```

`src/utils/package.ts:13-24`:
 `readPackageJson()` uses `empathic/package` to locate
the nearest `package.json` and parses it:

```typescript
export async function readPackageJson(
  dir: string,
): Promise<PackageJsonWithPath | undefined> {
  const packageJsonPath = findPackage({ cwd: dir, },);
  if (!packageJsonPath)
    return;
  const contents = await readFile(packageJsonPath, 'utf8',);
  return { ...JSON.parse(contents,), packageJsonPath, };
}
```

#### Step 2: `pkg` is passed into the resolved config

`src/config/options.ts:299`:
 the parsed `PackageJson` object is stored on the resolved config:

```typescript
const config: ResolvedConfig = {
  // ...
  pkg,
  // ...
};
```

#### Step 3: `DepsPlugin` is registered whenever `pkg` exists

`src/features/rolldown.ts:115-117`:
 the plugin is added if there is a `package.json`
**or** if `skipNodeModulesBundle` is set.
Since any real project has a `package.json`,
 this plugin is effectively always active:

```typescript
if (config.pkg || config.deps.skipNodeModulesBundle)
  plugins.push(DepsPlugin(config, bundle,),);
```

#### Step 4: `getProductionDeps` collects all production dependency names

`src/features/deps.ts:415-421`:
 on plugin creation,
 production deps are extracted
from `package.json`:

```typescript
/*
 * Production deps should be excluded from the bundle
 */
function getProductionDeps(pkg: PackageJson,): Set<string> {
  return new Set([
    ...Object.keys(pkg.dependencies || {},),
    ...Object.keys(pkg.peerDependencies || {},),
    ...Object.keys(pkg.optionalDependencies || {},),
  ],);
}
```

This is called at `src/features/deps.ts:169`:

```typescript
const deps = pkg && Array.from(getProductionDeps(pkg,),);
```

#### Step 5: every non-entry import is checked against the production deps list

`src/features/deps.ts:296-331`:
 `externalStrategy()` runs for every resolved import.
If the import ID matches a production dependency name (or starts with `<dep>/`),
it returns `true` (external):

```typescript
async function externalStrategy(
  id: string,
  importer: string | undefined,
  resolved: ResolvedId | null,
): Promise<boolean | [true, string,] | 'absolute' | 'no-external'> {
  if (id === shimFile)
    return false;

  if (alwaysBundle?.(id, importer,))
    return 'no-external';

  // ...skipNodeModulesBundle check...

  if (deps) {
    if (deps.includes(id,) || deps.some(dep => id.startsWith(`${dep}/`,))) {
      const resolvedDep = await resolveDepSubpath(id, resolved,);
      return resolvedDep ? [true, resolvedDep,] : true;
    }

    // ...@types fallback for DTS...
  }

  return false;
}
```

The `resolveId` hook at `src/features/deps.ts:173-208` calls `externalStrategy()`
and marks the module as external when it returns `true`:

```typescript
let shouldExternal = await externalStrategy(id, importer, resolved,);
// ...
if (shouldExternal === true || shouldExternal === 'absolute') {
  return {
    id,
    external: shouldExternal,
    moduleSideEffects,
  };
}
```

#### Step 6: bundled `node_modules` deps trigger a warning by default

`src/features/deps.ts:275-284`:
 when `onlyBundle` is not configured (the default),
the `generateBundle` hook scans output chunks for bundled `node_modules` deps
and emits a hint:

```typescript
} else if (onlyBundle == null && deps.size) {
  logger.info(
    nameLabel,
    `Hint: consider adding deps.onlyBundle option to avoid unintended bundling of dependencies...`
  )
}
```

A tool that bundles everything by default would not warn about bundling.

#### Summary of the trace

The call chain is:

1. `resolveOptions()` reads `package.json` -> `pkg`
2. `pkg` is stored on `ResolvedConfig`
3. `DepsPlugin(config, bundle)` is registered (always,
    when `pkg` exists)
4. `getProductionDeps(pkg)` extracts `dependencies` + `peerDependencies` + `optionalDependencies`
5. `externalStrategy()` marks any import matching a production dep as external
6. Anything from `node_modules` that slips through triggers a warning

No user configuration is required.
 The FAQ's claim that "tsdown bundles all imported modules"
by default is provably false.

### The FAQ's likely origin

The FAQ entry describes the behavior of a raw bundler (Rolldown,
 Rollup,
 webpack)
with zero configuration:
 everything imported gets bundled unless explicitly marked external.
tsdown's `DepsPlugin` overrides this raw default by reading `package.json`
and auto-externalizing production deps.
The FAQ appears to describe the underlying bundler's behavior
rather than tsdown's actual smart-default behavior.

The FAQ also recommends `deps.skipNodeModulesBundle` as the fix,
which is a **stricter** option that externalizes all `node_modules` imports
regardless of whether they are in `package.json`.
The actual default behavior already externalizes production deps;
`skipNodeModulesBundle` goes further by also externalizing `devDependencies`
that would otherwise be bundled.

### Impact

A user reading only the FAQ would:

- Set up `deps` configuration unnecessarily (it is already the default)
- Misunderstand why their production dependencies **are not** in the output
  (expected them to be bundled,
   per the FAQ)
- Potentially add `deps.alwaysBundle` for packages that should remain external,
  creating duplicate copies in consumer bundles
- Use `skipNodeModulesBundle` thinking it enables externalization,
  when it actually changes the externalization scope from "production deps only"
  to "all `node_modules`"

### Related tsdown configuration

For reference,
 the actual dependency handling options:

- `deps.alwaysBundle`:
   force-bundle specific packages even if listed in production deps (equivalent to tsup's `noExternal`)
- `deps.skipNodeModulesBundle`:
   externalize **all** `node_modules` imports regardless of `package.json` listing
- `deps.onlyBundle`:
   whitelist specific `node_modules` packages;
   warn on anything else being bundled

---

### Draft GitHub issue

**Title:
** docs(faq):
 "bundles all imported modules" contradicts Dependencies and How It Works pages

**Labels:
** documentation

**Body:
**

#### Description

The FAQ entry ["Why are my dependencies being bundled?"](https://tsdown.dev/guide/faq#dependencies-bundled) (`doc/guide/faq.md:50-62`) states:

> By default,
>  tsdown bundles all imported modules.
>  To exclude dependencies (e.g.,
>  those listed in `package.json`),
>  use the `deps` configuration

This contradicts two other documentation pages:

**Dependencies page** (`doc/options/dependencies.md:9`):

> By default,
>  `tsdown` **does not bundle dependencies** listed in your `package.json` under `dependencies`,
>  `peerDependencies`,
>  and `optionalDependencies`

**How It Works page** (`doc/guide/how-it-works.md:30`):

> `dependencies`,
>  `peerDependencies`,
>  and `optionalDependencies` are **externalized**:
>  they appear as `import` / `require` statements in the output and are not included in the bundle.

#### Source code confirms the Dependencies page is correct

`DepsPlugin` (`src/features/deps.ts`) is registered
whenever a `package.json` exists (`src/features/rolldown.ts:115-117`).
 On initialization,
 `getProductionDeps()` (`src/features/deps.ts:415-421`) collects all names from `dependencies`,
 `peerDependencies`,
 and `optionalDependencies`.
 The `externalStrategy()` function (`src/features/deps.ts:316-319`)
then marks any import matching those names as external,
 no user configuration required.

Additionally,
 when `deps.onlyBundle` is not set (the default),
 the `generateBundle` hook (`src/features/deps.ts:275-284`)
emits a warning if any `node_modules` dependencies end up bundled.
 A tool that bundles everything by default would not warn about bundling.

#### The FAQ also recommends a stricter-than-necessary fix

The FAQ suggests `deps.skipNodeModulesBundle: true` as the solution,
 which externalizes **all** `node_modules` imports (including `devDependencies`).
 The actual default already externalizes production deps.
 Users following the FAQ would unknowingly switch from "externalize production deps,
 bundle devDeps" to "externalize everything from node_modules",
 a meaningful behavioral change.

#### Suggested fix

Replace the FAQ entry with text consistent with the Dependencies page:

```markdown
### Why are my dependencies being bundled? {#dependencies-bundled}

By default, tsdown externalizes packages listed in your `package.json`
under `dependencies`, `peerDependencies`, and `optionalDependencies`.
Packages listed only in `devDependencies` are bundled if imported,
since consumers will not install them.

If you are seeing unexpected bundling of `node_modules` dependencies,
check that the package is listed in your production dependencies.
See [Dependencies](../options/dependencies.md) for fine-grained control
over which packages are bundled or externalized.
```

## Browser client bundle leaves package imports unresolved

### Symptom

The Done PostCSS page served its client module successfully,
but Chromium rejected bare imports before hydration:

```text
TypeError: Failed to resolve module specifier "postcss".
Relative references must start with either "/", "./", or "../".
```

Forcing only `postcss` into the bundle exposed a second bare import:

```text
TypeError: Failed to resolve module specifier
"@monochromatic-dev/hyperscript-core/ts".
```

The failed artifact was `package/done/variant-postcss/dist/client/inbox.js`.

### Root cause

The behavior was the composition of package metadata and tsdown's documented dependency policy,
not a Rolldown resolution failure.

`package/done/variant-postcss/package.json` listed `postcss` as a production dependency,
even though its source imported PostCSS only through `@monochromatic-dev/css-cli`.
tsdown `0.22.5` collects root production dependencies in `getProductionDeps()`
and `externalStrategy()` externalizes an exact package name or its subpaths.
The implementation is in [`src/features/deps.ts`][tsdown-deps] at revision
`940f65248715316b4087bb79e6bf05c77d101c10`.
Its source digest is
`403f204c5183ce8c264cae3d317a212bddbc06924d09286d3c7b215b93e6e6da`.

A config override that replaced `base.deps.alwaysBundle` with only `postcss`
also discarded the shared `@monochromatic-dev/**` rule.
That caused the subsequent workspace-package import failure.

PostCSS `8.5.16` publishes browser substitutions for `fs`,
`path`,
`url`,
`source-map-js`,
and its terminal highlighter.
Rolldown `1.1.5` enables package `browser` alias fields only for `platform: 'browser'`.
The resolver branch is in [`resolver_config.rs`][rolldown-resolver] at revision
`f09947ab017d6df74299f691853dcfc4f4f0f86e`.
Its source digest is
`30cde9f3fffb37c8b79180382a57b0819250d492ad135469a72b8abf883aafd1`.

### Verified repair

The repair keeps each ownership boundary explicit:

- Removed the redundant root `postcss` dependency from
  `package/done/variant-postcss/package.json`.
  `@monochromatic-dev/css-cli` remains the package that owns PostCSS.
- Added `package/css/cli/src/apply-mixins.ts` as a browser-compatible entry
  that excludes file-system and package-resolution code from the client graph.
- Changed `package/done/variant-postcss/src/client/css.ts`
  to import `@monochromatic-dev/css-cli/ts/apply-mixins`.
- Set `platform: 'browser'` in
  `package/done/variant-postcss/tsdown.client.config.ts`
  so PostCSS's published browser substitutions apply.
- Preserved `base.deps.alwaysBundle` and set `deps.onlyBundle`
  to `nanoid`,
  `picocolors`,
  and `postcss`,
  the dependencies observed in the generated bundle.

`mise run //package/done/variant-postcss:build` then completed
without unresolved imports or dependency hints.
A text scan found no remaining bare package imports in `dist/client`.

Browser verification used an in-memory database and exercised the generated page entries:

```text
DB_PATH=:memory: mise run //package/done/variant-postcss:serve:site
```

Chromium loaded Inbox,
In Progress,
Settings,
Search,
and task detail pages.
The run created and found a task,
started and stopped its timer,
and saved an updated title.
The final browser console contained application lifecycle logs only;
the page-error collection was empty.

### Why we do not file this upstream

No upstream issue is warranted.
tsdown behaved according to its production-dependency rules,
and Rolldown applied browser aliases once the config selected the browser platform.
The defects were the app's redundant dependency declaration,
a config override that replaced inherited bundling policy,
and a package entry that mixed browser and Node concerns.

[tsdown-deps]: https://github.com/rolldown/tsdown/blob/940f65248715316b4087bb79e6bf05c77d101c10/src/features/deps.ts
[rolldown-resolver]: https://github.com/rolldown/rolldown/blob/f09947ab017d6df74299f691853dcfc4f4f0f86e/crates/rolldown_resolver/src/resolver_config.rs
