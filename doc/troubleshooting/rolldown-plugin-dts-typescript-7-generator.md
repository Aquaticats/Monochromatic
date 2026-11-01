# rolldown-plugin-dts 0.27.2 selects tsgo under TypeScript 7 and misses cross-package declarations

## Symptom

After `pnpm-lock.yaml` moved rolldown-plugin-dts from 0.27.1 to 0.27.2,
any package lint that needed to refresh the shared oxlint build failed before oxlint started:

```sh
mise run //package/pi-plugin/auto-mode:lint:oxlint
```

The lint task calls `ensureOxlintConfig()`,
which ran:

```sh
mise run //package/config/oxlint:build:js:node
```

Tsdown then failed with one error for each plugin sidecar source:

```text
[plugin rolldown-plugin-dts:generate]
Error: tsgo did not generate dts file for
.../package/oxlint-plugin/stylistic/src/index.ts, please check your tsconfig.

[plugin rolldown-plugin-dts:generate]
Error: tsgo did not generate dts file for
.../package/oxlint-plugin/tsdoc/src/index.ts, please check your tsconfig.

[plugin rolldown-plugin-dts:generate]
Error: tsgo did not generate dts file for
.../package/oxlint-plugin/no-restricted-syntax/src/index.ts, please check your tsconfig.
```

The failing declaration catalog is specific:
the config-oxlint project's own files receive declarations,
while the three workspace plugin entry files in the JavaScript bundle do not.
A package-local plugin build still passes because its declaration inputs remain inside its own tsconfig project.

## Root cause

The regression is an option-resolution and graph-boundary mismatch in rolldown-plugin-dts 0.27.2.
The installed TypeScript major version changes the declaration generator before the plugin has read
`isolatedDeclarations` from the package tsconfig.
The resulting tsgo pass emits the tsconfig project,
while Rolldown later asks the declaration plugin to load declarations for every TypeScript source module bundled into
JavaScript,
including source from other workspace packages.

Source was read from `sxzz/rolldown-plugin-dts` tag `v0.27.2`,
commit `dd7e8fc6a88aba4107dd20975682233df5e5d3a5`.

1. `src/tsgo.ts:21` classifies every installed TypeScript major version of 7 or newer as tsgo:

   ```typescript
   export function isTsgo(): boolean {
     const major = getTypeScriptMajor()
     return major != null && major >= 7
   }
   ```

2. `src/options.ts:285` resolves the tsgo option before loading the package tsconfig.
   Because the caller has not supplied an explicit `oxc` option,
   TypeScript 7 makes `tsgo` truthy:

   ```typescript
   // Resolve tsgo option
   if (tsgo == null) {
     tsgo = isTsgo() && !vue && !tsMacro && !oxc
   }
   if (tsgo === true) {
     tsgo = {}
   } else if (typeof tsgo === 'object' && tsgo.enabled === false) {
     tsgo = false
   }
   ```

3. The plugin reads the tsconfig only after choosing tsgo.
   `src/options.ts:329` would normally select Oxc when the resolved compiler options enable
   `isolatedDeclarations`,
   but its guard requires tsgo to be false:

   ```typescript
   oxc ??= !!(compilerOptions?.isolatedDeclarations && !vue && !tsgo && !tsMacro)
   ```

   The shared TypeScript configuration sets `isolatedDeclarations: true`.
   Under 0.27.1,
   that setting selected Oxc and generated each declaration from the module code handed to the plugin.
   Under 0.27.2,
   the earlier TypeScript 7 check makes the Oxc fallback unreachable.

4. `src/generate.ts:109` derives one declaration root from the selected tsconfig.
   `src/generate.ts:114` then launches a complete tsgo project emission before Rolldown has traversed its module graph:

   ```typescript
   const rootDir = tsconfig ? path.dirname(tsconfig) : cwd

   return {
     name: 'rolldown-plugin-dts:generate',

     async buildStart(options) {
       if (tsgo) {
         tsgoContext = await runTsgo(rootDir, tsconfig, sourcemap, tsgo.path)
       }
   ```

5. Rolldown's JavaScript traversal is broader than that tsconfig project.
   `src/generate.ts:185` registers every traversed TypeScript module outside `node_modules` in `dtsMap`:

   ```typescript
   if (!jsFile || emitJs) {
     const mod = this.getModuleInfo(id)
     const isEntry = entryMatcher
       ? entryMatcher(path.relative(cwd, id))
       : !!mod?.isEntry
     const dtsId = filename_to_dts(id)
     dtsMap.set(dtsId, { code, id, isEntry, jsFile })
   ```

   The three sidecars in `package/config/oxlint/src/plugin-*.ts` deliberately re-export plugin package `/ts`
   source subpaths.
   The JavaScript build uses `deps.alwaysBundle` for `@monochromatic-dev/**`,
   so Rolldown traverses and inlines those external workspace source files.
   Config-oxlint's package-scoped tsconfig does not emit declarations for them.

6. When Rolldown loads the declaration module,
   `src/generate.ts:237` assumes the earlier tsgo project pass emitted a matching file.
   `src/generate.ts:244` throws when it did not:

   ```typescript
   if (tsgo) {
     const dtsPath = path.resolve(
       tsgoContext!.path,
       path.relative(path.resolve(rootDir), filename_to_dts(id)),
     )
     if (!existsSync(dtsPath)) {
       debug('[tsgo]', dtsPath, 'is missing')
       throw new Error(
         `tsgo did not generate dts file for ${id}, please check your tsconfig.`,
       )
     }
   ```

### Upstream resolution

rolldown-plugin-dts 0.27.4,
released on 2026-07-09,
adds a first-class `generator` option.
Source was read from tag `v0.27.4`,
commit `efb7230fa7a7ef2d383112d48c6c80ac439563a2`.

`src/options.ts:38` accepts an explicit generator:

```typescript
generator?: 'tsc' | 'oxc' | 'tsgo'
```

`src/options.ts:338` also resolves inherited `isolatedDeclarations` before falling back to TypeScript 7:

```typescript
if (!generator) {
  if (vue || tsMacro) {
    generator = 'tsc'
  } else if (tsgo) {
    generator = 'tsgo'
  } else if (oxc || compilerOptions?.isolatedDeclarations) {
    generator = 'oxc'
  } else if (isTS7Installed()) {
    generator = 'tsgo'
  } else {
    generator = 'tsc'
  }
}
```

`src/generate.ts:262` then runs the per-module Oxc declaration path when selected:

```typescript
} else if (generator === 'oxc' && !RE_VUE.test(id)) {
  const result = isolatedDeclarationSync(id, code, oxc)
```

This restores the generator that the repository's `isolatedDeclarations: true` policy used before 0.27.2,
and the explicit option lets the shared tsdown presets preserve that choice across future TypeScript upgrades.

### tsgo stays project-bounded through 0.27.9

The explicit `generator` option changes which backend runs,
not how the tsgo backend scopes its emission.
Source was read from `sxzz/rolldown-plugin-dts` `main`,
grafted commit `7a08c944f7527cebc5a89439d006528ca64acd78` (version 0.27.9),
and matches the installed artifact
(`dist/index.mjs:961` in `rolldown-plugin-dts@0.27.9_rolldown@1.1.5_typescript@7.0.2`).

`src/generate.ts:114` still derives one declaration root from the selected tsconfig,
`src/generate.ts:121` still launches a whole-project tsgo emission at `buildStart`,
and `src/generate.ts:257` still throws for any traversed module the project pass did not emit:

```typescript
`tsgo did not generate dts file for ${id}, please check your tsconfig.`,
```

Selecting `generator: 'tsgo'` explicitly therefore fails any build that inlines
workspace source from outside the entry package's tsconfig project,
exactly as the implicit 0.27.2 selection did.
This is the project-boundary model upstream recorded in
[rolldown-plugin-dts #189](https://github.com/sxzz/rolldown-plugin-dts/issues/189):
tsgo build mode stays unsupported without an official API.

## Verification

Versions and source revisions:

- Node `26.4.0` in the disposable worktree.
- TypeScript `7.0.1-rc`.
- tsdown `0.22.4`.
- Rolldown `1.1.5`.
- Failing rolldown-plugin-dts `0.27.2`,
  tag commit `dd7e8fc6a88aba4107dd20975682233df5e5d3a5`.
- Passing rolldown-plugin-dts `0.27.4`,
  tag commit `efb7230fa7a7ef2d383112d48c6c80ac439563a2`.
- Consumer checkout commit `5922c4b0e` in a disposable Git worktree.

The minimized harness was:

```sh
mise run //package/config/oxlint:build:js:node
```

The user-facing harness was:

```sh
mise run //package/pi-plugin/auto-mode:lint:oxlint
```

The failure reproduced on consecutive minimized runs and once through the complete lint task.
Every run reported the same three missing paths;
only diagnostic ordering varied.

### Custom TSDoc tag preservation on the resolved version

A later disposable probe exercised the repository-resolved passing stack:

- Rolldown 1.1.5;
- `rolldown-plugin-dts` 0.27.4;
- `generator: 'oxc'`;
- TypeScript 7.0.2 installed;
- one function,
  one overload,
  and one type call signature carrying `@mutates target - description`;
- a separate entry file re-exporting every declaration.

The direct `rolldown/experimental` `isolatedDeclarationSync` result contained three `@mutates` blocks and no errors.
Bundling through `rolldown-plugin-dts` also contained three blocks,
including every target and description.
The re-exporting entry changed declaration linkage but did not strip custom TSDoc.

The verification task was a disposable `mise` task invoking a Node module that called
`isolatedDeclarationSync(sourcePath, source, { sourcemap: false })`,
then:

```javascript
const build = await rolldown({
  input: reexportingEntryPath,
  plugins: [dts({ generator: 'oxc' })],
});
const generated = await build.generate({ format: 'es' });
```

Verified counts:

```text
isolatedMutatesCount: 3
bundledMutatesCount: 3
isolatedErrors: []
```

This verifies custom-tag preservation for the installed versions and tested declaration forms.
It does not replace package-build and external-consumer publication tests.

### Explicit-generator backend bench on raw rolldown (2026-07-15)

A later disposable-worktree bench compared the `oxc` and `tsgo` backends
selected explicitly through `generator`,
driven by raw `rolldown` (no tsdown),
ahead of the planned tsdown removal.
TypeScript 6 and the `tsc` backend were excluded as deprecated for this repository.

Versions:

- Node `26.5.0`.
- Rolldown `1.1.5` from the workspace catalog.
  Rolldown `1.2.0` (published 2026-07-15T11:08Z) was blocked by the workspace's
  `minimumReleaseAge` supply-chain policy at bench time;
  the backends under test live in rolldown-plugin-dts,
  so the comparison is unaffected.
- rolldown-plugin-dts `0.27.9`,
  installed as `rolldown-plugin-dts@0.27.9_rolldown@1.1.5_typescript@7.0.2`.
- TypeScript `7.0.2`.

The harness was one config factory per bench cell replicating the shared tsdown flavor settings:
externals built from `package.json` `dependencies` plus `peerDependencies`
minus `@monochromatic-dev/**` (kept inline),
`transform.target` from the repository `browserslistTargets` helper,
`minify: { compress: true, mangle: false, codegen: true }`,
`entryFileNames: '[name].mjs'`,
`output.cleanDir: true`,
and `dts({ generator, tsconfig })` selected by a `BENCH_DTS` env var
(`off` | `oxc` | `tsgo`).
Timing ran through hyperfine 1.20.0 with one warmup and five measured runs per cell:

```sh
hyperfine --warmup 1 --runs 5 --parameter-list gen off,oxc,tsgo \
  'BENCH_DTS={gen} node_modules/.bin/rolldown --config bench/or-throw.config.ts'
```

Process wall-time means over five runs:

- `or-throw/core` (31 source files,
   zero runtime deps,
   neutral flavor):
  `off` 170.1 ms ± 4.7,
  `oxc` 195.1 ms ± 8.8,
  `tsgo` 275.5 ms ± 6.2.
  Declaration increment over the `off` control:
  25.0 ms for `oxc`,
  105.4 ms for `tsgo` (roughly four times `oxc`).
- `toml-edit/core` (75 source files,
   four inlined workspace deps,
   neutral flavor):
  `off` 150.9 ms ± 5.6,
  `oxc` 196.5 ms ± 6.5 (increment 45.6 ms).
  `tsgo`:
   build failure,
  no timing possible.
- `config/oxlint` (five entries,
   four inlined workspace plugin packages,
   node flavor):
  `off` 164.5 ms ± 4.7,
  `oxc` 269.2 ms ± 7.5 (increment 104.7 ms).
  `tsgo`:
   build failure,
  no timing possible.

The `tsgo` failure on both cross-package cells was:

```text
Error: tsgo did not generate dts file for
<worktree>/packages/ownership-marker/foreign-borrowed/src/index.ts,
please check your tsconfig.
```

Output quality on the one cell where both backends built (`or-throw/core`):
the `oxc` and `tsgo` `index.d.mts` differ by 46 diff lines,
all either string-literal quote style (`""` versus `''`)
or file-level `@module` TSDoc blocks that `tsgo` preserves and `oxc` drops.
No exported type surface differs.
All three `oxc` declaration bundles pass
`tsc --noEmit --strict --skipLibCheck --ignoreConfig` under TypeScript 7.0.2.

### Passing catalog

- rolldown-plugin-dts 0.27.1 with the original shared `dts: true` configuration:
  config-oxlint build passed.
- rolldown-plugin-dts 0.27.4 with the original shared `dts: true` configuration:
  config-oxlint build passed because inherited `isolatedDeclarations` selected Oxc.
- rolldown-plugin-dts 0.27.4 with `generator: 'oxc'` in both declaration-emitting shared presets:
  config-oxlint Node build passed.
- The same explicit configuration:
  the stylistic plugin Node build passed.
- The same explicit configuration:
  the module-const neutral build passed.
- The same explicit configuration:
  config-tsdown type lint and oxlint passed.
- With the independent auto-mode TSDoc findings corrected in the disposable worktree,
  the original user-facing auto-mode oxlint task reported zero warnings and zero errors.
- Config-oxlint's five JavaScript outputs still present in the main checkout matched the 0.27.4 Oxc rebuild by
  SHA-256.
  The failed 0.27.2 build had removed the five declaration outputs before throwing,
  so they had no remaining main-checkout byte baseline;
  the passing build restored all ten expected outputs.

The shared preset inventory found 64 Node config consumers,
24 neutral config consumers,
and 7 client config consumers.
The client preset has `dts: false` and is not affected.
A broader search found 95 shared-config imports across 95 tsdown config files,
matching those three groups.

### Failing catalog

- rolldown-plugin-dts 0.27.2 with the original shared `dts: true` configuration:
  three `tsgo did not generate dts file` errors.
- rolldown-plugin-dts 0.27.2 with `oxc: true`:
  the build failed earlier with:

  ```text
  Error: [rolldown-plugin-dts] TypeScript 7.0 is installed, but the `tsgo` option is disabled.
  Please enable it to use TypeScript 7.0 features.
  ```

  The source check at `src/options.ts:295` rejects disabled tsgo whenever TypeScript 7 is installed,
  even when Oxc was explicitly requested.
- rolldown-plugin-dts 0.27.9 with explicit `generator: 'tsgo'` on raw rolldown 1.1.5:
  any build inlining workspace source from outside the entry package's tsconfig project fails with the same
  `tsgo did not generate dts file` error
  (verified 2026-07-15 on `toml-edit/core` and `config/oxlint`;
  a zero-dependency single-package build passes).

## Verified workarounds

### Upgrade to 0.27.4 and select Oxc in the shared presets

This is the preferred repair.
Use rolldown-plugin-dts 0.27.4 or newer through tsdown's compatible dependency range,
then set:

```typescript
dts: {
  generator: 'oxc',
},
```

in `package/config/tsdown/src/index.node.ts` and `package/config/tsdown/src/index.ts`.

Tradeoffs:

- Oxc declaration generation requires source compatible with `isolatedDeclarations`.
  This repository already enforces that option globally.
- Oxc does not perform type-checker-dependent declaration inference.
  Package type lint remains the separate semantic check.
- The workspace must keep rolldown-plugin-dts at 0.27.4 or newer while the config uses `generator`.
- At the time of investigation,
  the release had not yet cleared the workspace's 1,440-minute supply-chain age policy.
  It becomes eligible at 2026-07-10 08:52:56-04:00.
  The user selected an immediate,
  exact-version `rolldown-plugin-dts@0.27.4` exception rather than waiting or temporarily rolling back.
  Pnpm 11.10.0 accepted that selector and passed the repository's 731-entry supply-chain check in the disposable
  prototype.

The implemented repair passed both Node and neutral builds,
kept all five comparable JavaScript output hashes unchanged,
restored the five declaration outputs removed by the failed build,
and restored the original lint path after the independent TSDoc findings were corrected.

The exact release-age exception applies only to 0.27.4 and never exempts a future release.
Removing it after 2026-07-10 08:52:56-04:00 is optional policy-file cleanup rather than a correctness change;
the exact override remains necessary until tsdown's own dependency floor requires a plugin release with
`generator`.

### Temporarily hold rolldown-plugin-dts 0.27.1

A disposable-worktree override to 0.27.1 restored the config-oxlint build without source changes.

Tradeoffs:

- Version 0.27.1 does not declare TypeScript 7 in its peer range.
- It relies on the older implicit Oxc path rather than an explicit generator contract.
- It delays adoption of the upstream fix and needs a follow-up removal.
- It is appropriate only when work must resume before 0.27.4 clears the supply-chain age policy and no exception is
  acceptable.

## What does not work

- Keeping 0.27.2 and setting `oxc: true`:
  verified to fail with the plugin's TypeScript 7 tsgo-disabled error.
- Keeping 0.27.2 and setting `tsgo: false`:
  reaches the same unconditional check in `src/options.ts:295`.
- Using `packageExtensions` to set
  `tsdown@0.22.4.dependencies.rolldown-plugin-dts: 0.27.4`:
  a clean pnpm 11.10.0 install retained rolldown-plugin-dts 0.27.2 because package extensions supplement package
  metadata rather than replacing this existing dependency edge.
  The aggregate build then reproduced all three missing declarations.
  An override is required to replace the resolved transitive version.
- Relying on config-oxlint's package tsconfig while using 0.27.2 tsgo:
  it cannot emit source files from the three external plugin package projects,
  even though Rolldown deliberately bundles them.
- Treating the failure as an auto-mode source diagnostic:
  oxlint never starts until the shared config build succeeds.
- Treating the package-local plugin build as proof the aggregate build works:
  that control remains inside one package project and does not exercise the failing cross-package sidecars.
- Hand-editing `pnpm-lock.yaml`:
  it bypasses package-manager resolution and repository policy.
  Regenerate the lock through the owning pnpm mise task.

## Upstream filing decision

No `.out-of-scope/` entry matches rolldown-plugin-dts,
tsdown,
or this declaration-generator regression.

Duplicate searches covered open and closed issues and pull requests for
`tsgo declaration monorepo`,
`did not generate dts file`,
and `generator option oxc TypeScript 7` in `sxzz/rolldown-plugin-dts`,
plus TypeScript 7 tsgo declaration searches in `rolldown/tsdown`.

Two related closed issues exist:

- [rolldown-plugin-dts #47](https://github.com/sxzz/rolldown-plugin-dts/issues/47) records an earlier monorepo
  `tsgo did not generate dts file` path-layout failure.
- [rolldown-plugin-dts #189](https://github.com/sxzz/rolldown-plugin-dts/issues/189) records tsgo's project-reference
  limitation and the maintainer's decision not to support tsgo build mode without an official API.

Neither needs a new comment for this incident.
Release 0.27.4 already contains the generator-selection mechanism needed by this repository.
A local maintenance issue tracks eventual removal of the compatibility pin:
[#359](https://github.com/Aquaticat/Monochromatic/issues/359).

1. **Upstream responsibility**
   Yes for 0.27.2.
   Its new default selected tsgo before reading inherited `isolatedDeclarations`,
   changing a previously passing supported configuration.
2. **Upstream ability to fix**
   Yes.
   Upstream added the generator selector in commit `4564ef53545456ce2a2e21029066db27ee99991a` and released it in 0.27.4.
3. **Supported use case**
   Yes.
   The 0.27.4 README documents `generator: 'oxc'` and automatic Oxc selection for
   `isolatedDeclarations`.
4. **Contribution policy**
   The repository welcomes contributions in general,
   but no contribution remains necessary here.
   The repository has issues enabled,
   its organization contribution guide asks for minimal reproductions and Conventional Commits,
   and no repository,
   organization guide,
   issue workflow,
   or code-of-conduct text bans AI-assisted reports.
5. **Likelihood of a fix**
   Already satisfied.
   Releases 0.27.3 and 0.27.4 followed 0.27.2 on 2026-07-09,
   and 0.27.4 contains the verified solution.
6. **Compatible minimal prototype**
   Verified at the consumer boundary.
   The released `generator: 'oxc'` path was integrated into both shared declaration-emitting presets,
   exercised through Node and neutral tsdown builds,
   type-checked,
   linted,
   and compared by SHA-256 against every main-checkout output that survived the failed 0.27.2 build.

### Re-check on 2026-07-15 (explicit tsgo at 0.27.9)

The backend bench re-confirmed the tsgo project-boundary failure with the explicit `generator: 'tsgo'` selector
at 0.27.9 on raw rolldown.
Nothing about this is additive upstream:
[rolldown-plugin-dts #189](https://github.com/sxzz/rolldown-plugin-dts/issues/189)
already records the project-boundary limitation and the maintainer's decision
not to support tsgo build mode without an official API,
which fails constraint 5 for a new filing on the same mechanism.
No new issue and no comment;
the repository's remedy remains `generator: 'oxc'`.

### Upstream filing artifact

No new upstream issue or comment is warranted:
upstream already released the required option,
and the related issue threads do not need another reproduction after the released fix passed the real consumer.

The local tracking issue is [#359](https://github.com/Aquaticat/Monochromatic/issues/359).
It records the separate maintenance action:
remove the exact override after tsdown raises its dependency floor or a fresh dependency graph independently resolves a
compatible release without it.
The exact release-age exception may be removed once 0.27.4 naturally clears the age gate;
that does not by itself authorize removing the override.

## References

- [rolldown-plugin-dts 0.27.2 release](https://github.com/sxzz/rolldown-plugin-dts/releases/tag/v0.27.2)
- [rolldown-plugin-dts 0.27.4 release](https://github.com/sxzz/rolldown-plugin-dts/releases/tag/v0.27.4)
- [Generator option commit](https://github.com/sxzz/rolldown-plugin-dts/commit/4564ef53545456ce2a2e21029066db27ee99991a)
- [Earlier monorepo tsgo issue](https://github.com/sxzz/rolldown-plugin-dts/issues/47)
- [Project-reference limitation](https://github.com/sxzz/rolldown-plugin-dts/issues/189)
- [Pnpm package extension and override settings](https://pnpm.io/settings#packageextensions)
- [Implementation plan](../planning/tsdown-typescript-7-cross-package-dts.md)
