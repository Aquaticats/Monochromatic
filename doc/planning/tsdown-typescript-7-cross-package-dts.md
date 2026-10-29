# Restore config-oxlint declaration bundling with TypeScript 7

## Status

Implemented and verified on 2026-07-09.

The repair is:

- An exact rolldown-plugin-dts 0.27.4 override and exact-version release-age exception in `pnpm-workspace.yaml`.
- A package-manager-generated lockfile resolution at 0.27.4.
  Regeneration also registered the already-landed fs-id package importer;
  the lock commit reports that unrelated generated drift separately.
- Explicit Oxc declaration generation in config-tsdown's Node and neutral presets.
- Config-tsdown README guidance for declaration generation and separate semantic type lint.
- Correct destructured parameter and void-return TSDoc in auto-mode's notification helpers.

The original command now reaches real oxlint and reports zero warnings and zero errors in a clean disposable worktree
at implementation commit `5bdc39e4a`.
Local issue [#359](https://github.com/Aquaticat/Monochromatic/issues/359)
tracks eventual override removal after an upstream
or tsdown dependency floor makes it safe.

## Goal

Restore package-scoped oxlint tasks on a fresh or stale checkout without weakening declaration generation,
without requiring developers to prebuild the three oxlint plugin packages manually,
and without making the generated oxlint configuration depend on untracked build state.

The implementation must preserve these existing behaviors:

- `ensureOxlintConfig()` rebuilds the shared configuration when any bundled oxlint package source is newer than its
  outputs.
- `package/config/oxlint/dist/final/node/index.mjs` and its three plugin sidecars remain self-contained.
- Source exports under each plugin package's `/ts` subpath remain usable for in-repository development.
- Package builds and lint tasks continue to run through mise.

## Observed task chain

The current task graph is:

1. `package/pi-plugin/auto-mode/mise.toml` extends the root `lint:oxlint` task template.
2. The template in `mise.toml` calls `ensureOxlintConfig()` before invoking the oxlint wrapper.
3. `ensureOxlintConfig()` considers source and configuration files from `package/config/oxlint` and every package under
   `package/oxlint-plugin`.
4. A missing or stale output triggers
   `mise run //package/config/oxlint:build:js:node`.
5. `package/config/oxlint/tsdown.node.config.ts` builds one config entry and three plugin sidecar entries.
6. Each sidecar re-exports a plugin package's TypeScript `/ts` source subpath so the JavaScript bundle can inline it.
7. Declaration generation fails for the external plugin package `src/index.ts` files before the lint process starts.

## Evidence collected

The failing lockfile resolved:

- TypeScript `7.0.1-rc`.
- tsdown `0.22.4`.
- rolldown-plugin-dts `0.27.2`.

The implemented lockfile keeps TypeScript and tsdown unchanged and resolves rolldown-plugin-dts `0.27.4`.

Commit `797061b59ea222281a19d89ccbad6200ea526a70` updated only `pnpm-lock.yaml` and moved
rolldown-plugin-dts from `0.27.1` to `0.27.2`.
The update added TypeScript 7 to the plugin's peer range.

rolldown-plugin-dts `0.27.2` selects its `tsgo` path when the installed TypeScript major version is 7.
Its generated `dist/index.mjs` does the following:

- Runs tsgo once with the current tsconfig,
  a temporary output directory,
  and `rootDir` set to the tsconfig directory.
- Registers every TypeScript module traversed by the JavaScript bundle in a declaration map.
- For every registered module,
  expects tsgo to have emitted a corresponding declaration into the temporary directory.
- Throws `tsgo did not generate dts file for <source>` when that expected file is absent.

`package/config/oxlint/tsconfig.json` extends the shared configuration whose `include` paths are scoped through
`${configDir}` to the consuming package.
The three plugin source files reported by the failure are outside that package directory.
The user-provided TSFILE list contains declarations for `package/config/oxlint` files,
but none for the three external plugin package entry files.

An independent reproduction ran at commit `ebc4a04d3c02e7dbe2c4268d14846566749a4d1d`
in a disposable Git worktree.
The minimized command was:

```sh
mise run //package/config/oxlint:build:js:node
```

It failed on two consecutive runs with the same three missing declaration paths.
The complete user-facing command also reproduced independently:

```sh
mise run //package/pi-plugin/auto-mode:lint:oxlint
```

The full task reached `ensureOxlintConfig()`,
ran the minimized command,
and propagated its nonzero exit before invoking the oxlint wrapper.
The failing build wrote declarations for the config-oxlint project into a fresh temporary directory on every run,
but did not write declarations for the three plugin package entry files.

The differential probes confirmed an upstream option-resolution regression:

- rolldown-plugin-dts 0.27.1 passed the aggregate build under the original config.
- rolldown-plugin-dts 0.27.2 failed the aggregate build under the original config.
- A package-local stylistic plugin build passed under 0.27.2,
  disproving a broad TypeScript 7 incompatibility.
- Explicit `oxc: true` under 0.27.2 failed with the plugin's unconditional
  `TypeScript 7.0 is installed, but the tsgo option is disabled` error.
- rolldown-plugin-dts 0.27.4 passed the aggregate build under both inferred and explicit Oxc selection.
- Explicit Oxc selection through the shared Node and neutral presets also passed representative package builds,
  type lint,
  and oxlint.
- The five JavaScript outputs still present in the main checkout matched the 0.27.4 Oxc rebuild by SHA-256.
  The failed 0.27.2 rebuild had removed the five declaration outputs,
  so those files had no remaining main-checkout byte baseline;
  the passing build restored all ten expected outputs.

The complete source trace,
version catalog,
and upstream-filing decision are recorded in
`doc/troubleshooting/rolldown-plugin-dts-typescript-7-generator.md`.

## Hypothesis results

1. **Confirmed**
   rolldown-plugin-dts 0.27.2 chooses tsgo before reading inherited `isolatedDeclarations`.
   Tsgo emits the package tsconfig project,
   while Rolldown later requests declarations for external workspace source modules included by the JavaScript bundle.
2. **Rejected as a repair**
   an aggregate declaration tsconfig would model Rolldown's dynamic bundle graph as a static TypeScript project.
   Upstream 0.27.4 instead restores the repository's per-module Oxc path without coupling config-oxlint to transitive
   source directories.
3. **Rejected as a repair**
   consuming prebuilt plugin declarations adds build-order and freshness dependencies that the current source-driven
   sidecars intentionally avoid.
   The 0.27.4 Oxc path preserves source inlining and requires no plugin prebuild.
4. **Disproved**
   TypeScript 7 is not broadly incompatible with repository declaration settings.
   Package-local 0.27.2 builds pass,
   and Node plus neutral builds pass under the explicit 0.27.4 Oxc generator.

## Investigation completed

All mutating reproduction and build probes ran in disposable Git worktrees.
The main worktree's unrelated concurrent changes remained untouched.

Completed probes:

1. Reproduced the original lint-triggered failure and the minimized config-oxlint build failure.
2. Compared rolldown-plugin-dts 0.27.1,
   0.27.2,
   and 0.27.4 with the same consumer source.
3. Verified the package-local stylistic build as a passing 0.27.2 control.
4. Verified that explicit `oxc: true` cannot bypass 0.27.2's TypeScript 7 tsgo guard.
5. Read rolldown-plugin-dts 0.27.2 and 0.27.4 source at their release tags.
6. Read tsdown 0.22.4's pass-through from `UserConfig.dts` to rolldown-plugin-dts.
7. Searched upstream releases,
   commit history,
   issues,
   pull requests,
   contribution policy,
   and repository `.out-of-scope/` policy.
8. Prototyped inferred and explicit Oxc selection under 0.27.4.
9. Exercised Node and neutral shared presets,
   package type lint,
   package oxlint,
   and the original user-facing lint task.
10. Verified pnpm 11.10.0's exact-version `minimumReleaseAgeExclude` syntax with
    `rolldown-plugin-dts@0.27.4` through the repository install task and its 731-entry supply-chain check.

## Strategy decision

The selected strategy is rolldown-plugin-dts 0.27.4 plus explicit Oxc selection in both declaration-emitting shared
presets.
The user selected an exact-version release-age exception so implementation can proceed before the normal eligibility
time.

### Selected: 0.27.4 with explicit shared Oxc generator

Pros:
uses the released upstream mechanism;
restores the pre-regression generator;
keeps source-driven cross-package bundling;
protects all 64 Node and 24 neutral preset consumers from an implicit switch to experimental tsgo;
requires no plugin prebuild;
and produced the expected aggregate declarations in the prototype.

Cons:
requires a transitive version floor while tsdown 0.22.4 still allows older plugin versions;
adds an exact-version supply-chain age exception;
and keeps semantic type checking as a separate lint step because Oxc declaration generation is syntax-driven.

### Rejected: 0.27.4 with inferred Oxc

Pros:
passes today with no shared preset source change because 0.27.4 prioritizes inherited `isolatedDeclarations`.

Cons:
leaves the intended generator implicit;
allows a future option-order change to repeat the 0.27.2 class of regression;
and does not explain why this repository deliberately avoids experimental tsgo for declaration bundling.

### Rejected: temporary 0.27.1 rollback

Pros:
passed the failing build immediately without a release-age exception.

Cons:
does not declare TypeScript 7 peer support;
relies on old implicit behavior;
requires another lock transition;
and postpones adoption of the released upstream selector.

### Rejected: aggregate tsgo project

Pros:
keeps tsgo and can model the three immediate plugin entry files.

Cons:
turns Rolldown's dynamic bundle graph into a static TypeScript project;
must follow every transitive workspace source package in `alwaysBundle`;
conflicts with the repository policy against cross-package TypeScript project references;
and is unnecessary once per-module Oxc generation is selectable.

### Rejected: prebuilt plugin package boundary

Pros:
keeps each declaration build inside one package project.

Cons:
adds plugin build-order and freshness dependencies to every lint prerequisite;
requires generated plugin artifacts before config-oxlint can build;
and reverses the current source-driven sidecar design.

Ranking:
explicit 0.27.4 Oxc > inferred 0.27.4 Oxc > temporary 0.27.1 > aggregate tsgo project > prebuilt plugin boundary.
Explicit selection beats inference by making the generator an owned repository decision;
inference beats rollback by using the fixed release without follow-up churn;
rollback beats an aggregate project because it restores known output without introducing cross-package project
coupling;
and the aggregate project beats prebuild orchestration because it at least preserves source-driven bundling.

## Implementation plan

### Dependency policy and lockfile

1. Confirm `file-enforcer.config.ts` still does not manage `pnpm-workspace.yaml`.
2. In `pnpm-workspace.yaml`,
   add the exact selector `rolldown-plugin-dts@0.27.4` to `minimumReleaseAgeExclude`.
   This applies the user's approved exception only to the reviewed release,
   not to future rolldown-plugin-dts versions.
   Retain it as the audit record for this exact reviewed release:
   after the release ages past the gate it has no behavioral effect,
   while removing it before then would make clean frozen installs fail.
3. Add an exact `rolldown-plugin-dts: 0.27.4` override with a comment that:
   0.27.2 selects experimental tsgo before inherited `isolatedDeclarations`;
   0.27.4 adds `generator`;
   and the override can be removed when tsdown's own dependency floor requires a release carrying that option.
4. Run:

   ```sh
   mise run prepare:pnpm:install -- --no-frozen-lockfile
   ```

   This exact command passed with the exact-version age exception in the disposable prototype.
5. In a clean disposable worktree carrying the generated lockfile,
   verify the frozen path:

   ```sh
   mise run prepare:pnpm:install -- --frozen-lockfile
   ```

6. Inspect the generated `pnpm-lock.yaml` diff.
   It must resolve rolldown-plugin-dts 0.27.4 with its registry integrity and tarball URL,
   retain tsdown 0.22.4 and Rolldown 1.1.5 unless an independently required lock update is explained,
   and contain no hand-edited lock content.

### Shared declaration generator

1. In `package/config/tsdown/src/index.node.ts`,
   replace `dts: true` in `baseOptions` with:

   ```typescript
   dts: {
     generator: 'oxc',
   },
   ```

2. Apply the same change to the neutral preset in `package/config/tsdown/src/index.ts`.
3. Leave `package/config/tsdown/src/index.client.ts` at `dts: false`.
4. Update `package/config/tsdown/README.md` to state that Node and neutral presets explicitly use Oxc because the
   shared TypeScript policy enforces `isolatedDeclarations`,
   while type lint remains responsible for semantic checking.
5. Do not add an isolated unit test for the option object.
   Config-tsdown is a source-exported preset package with no self-build task,
   while the regression occurs only when tsdown loads its transitive declaration plugin against a cross-package
   Rolldown graph.
   The config-oxlint build is the correct integration seam;
   it is itself a four-entry consumer and root or package lint invokes it on every fresh checkout because generated
   output is absent.
   Node and neutral consumer builds cover both changed presets.

### Auto-mode diagnostics exposed after the build repair

Once config-oxlint builds,
the original lint command reaches three pre-existing TSDoc diagnostics in
`package/pi-plugin/auto-mode/src/ask-user.ts`.
The implementation must fix them so the requested user-facing command is actually green:

1. For `invokeTerminalNotification`,
   replace `@param invocation` with separate `@param command` and `@param args` entries matching its destructured
   fields.
2. Remove its `@returns` tag because the function returns `Promise<void>`.
3. Remove `notifyAsk`'s `@returns` tag for the same reason.
4. Preserve the notification behavior and tests;
   only documentation changes are needed.

The exact edits passed auto-mode oxlint and type lint in the disposable prototype.

### Documentation and commits

1. Keep `doc/troubleshooting/rolldown-plugin-dts-typescript-7-generator.md` aligned with the implemented dependency
   selector,
   age exception,
   and verification results.
2. Update this plan's status to implemented only after every acceptance criterion passes.
3. Commit dependency policy and generated lock changes with explicit pathspecs before continuing.
4. Commit shared preset and README changes with explicit pathspecs.
5. Commit the auto-mode TSDoc correction separately so the independent diagnostics remain distinguishable from the
   declaration fix.
6. Commit final documentation evidence with explicit pathspecs.

## Verification procedure

All fresh-output and freshness probes run in a disposable worktree created from the implementation commits,
not against the main checkout's ignored build outputs.

1. Confirm resolved versions in `pnpm-lock.yaml` and installed package metadata:
   TypeScript 7.0.1-rc,
   tsdown 0.22.4,
   Rolldown 1.1.5,
   and rolldown-plugin-dts 0.27.4.
2. Run config-tsdown checks:

   ```sh
   mise run //package/config/tsdown:lint:types
   mise run //package/config/tsdown:lint:oxlint
   mise run //package/config/tsdown:test:unit
   ```

3. Before each aggregate build in the disposable worktree,
   remove only its generated output directory:

   ```sh
   rm --recursive --force package/config/oxlint/dist/final/node
   mise run //package/config/oxlint:build:js:node
   ```

   Run this fresh-output sequence twice.
   Both runs must produce `index.mjs`,
   its declaration entry and shared declaration chunk,
   plus JavaScript and declaration files for all three plugin sidecars.
4. Exercise one direct Node consumer and one direct neutral consumer:

   ```sh
   mise run //package/oxlint-plugin/stylistic:build:js:node
   mise run //package/const/core:build:js:browser
   ```

5. Run config-oxlint checks:

   ```sh
   mise run //package/config/oxlint:lint:types
   mise run //package/config/oxlint:lint:oxlint
   ```

6. Run auto-mode checks after the TSDoc correction:

   ```sh
   mise run //package/pi-plugin/auto-mode:lint:types
   mise run //package/pi-plugin/auto-mode:test:unit
   mise run //package/pi-plugin/auto-mode:lint:oxlint
   ```

   The final command is the end-user boundary from the original report and must report zero warnings and zero errors.
7. In the disposable worktree,
   record the config output modification time,
   touch a bundled plugin source,
   rerun the auto-mode oxlint task,
   and assert the output modification time increased.
   This verifies `ensureOxlintConfig()` still rebuilds stale sidecars.
8. Confirm the repaired build creates ten config-oxlint output files.
   Compare the five JavaScript outputs against the recorded pre-repair SHA-256 values;
   all must match.
   Confirm the five declarations removed by the failed 0.27.2 build are present.
9. Run Markdown lint on the plan,
   troubleshooting document,
   and config-tsdown README.
10. Inspect `git status` and the complete diff.
    No unrelated concurrent file,
    ignored build output,
    or disposable-worktree artifact may be staged.

## Implementation verification

Verification ran in a clean disposable worktree at commit `5bdc39e4a`:

- Frozen install passed and pnpm verified all 731 lock entries against supply-chain policy.
- Installed versions were Node 26.5.0,
  TypeScript 7.0.1-rc,
  tsdown 0.22.4,
  Rolldown 1.1.5,
  and rolldown-plugin-dts 0.27.4.
- Two builds after removing `package/config/oxlint/dist/final/node` each produced ten files with identical SHA-256
  catalogs.
- All five comparable JavaScript hashes matched the pre-repair Oxc outputs.
- Config-tsdown type lint,
  oxlint,
  and unit tests passed.
- Config-oxlint type lint and oxlint passed.
- Stylistic's Node build and module-const's neutral build passed.
- Auto-mode type lint and unit tests passed.
- The original auto-mode oxlint command reported zero warnings and zero errors.
- Touching stylistic plugin source increased config output modification time from `1783651532` to `1783651629`,
  proving `ensureOxlintConfig()` rebuilt before linting.
- Markdown lint passed for config-tsdown README,
  this plan,
  and the troubleshooting document.
- The verification worktree remained clean after ignored generated outputs were excluded.
- No real neutral multi-entry preset consumer exists:
  all 24 neutral config files are one-line re-exports.
  The four-entry config-oxlint build covers multi-entry generator behavior on the Node preset.

A clean `packageExtensions` experiment was also rejected:
adding rolldown-plugin-dts 0.27.4 to `tsdown@0.22.4.dependencies` left 0.27.2 resolved,
and the aggregate build reproduced the same three missing declarations.
The exact override is therefore required to replace tsdown's existing dependency edge.

## Acceptance criteria

The implementation is complete because:

- `pnpm-workspace.yaml` exempts only `rolldown-plugin-dts@0.27.4` from release age and forces that exact transitive
  version with documented rationale.
- Pnpm's supply-chain verification passes with the exact-version exception.
- The generated lockfile resolves rolldown-plugin-dts 0.27.4 without unrelated unexplained drift.
- Node and neutral shared presets select `generator: 'oxc'` explicitly;
  the client preset still emits no declarations.
- Config-tsdown README documents declaration generation and the separate semantic type-lint boundary.
- Config-tsdown lint,
  type lint,
  and unit tests pass with zero warnings or errors.
- `mise run //package/config/oxlint:build:js:node` succeeds twice from a fresh-output fixture and produces all ten
  expected outputs.
- Representative Node and neutral consumer builds pass.
- Config-oxlint lint and type lint pass.
- Auto-mode's exposed TSDoc diagnostics are corrected without runtime changes;
  its type lint and unit tests pass.
- `mise run //package/pi-plugin/auto-mode:lint:oxlint` reaches real oxlint and reports zero warnings and zero errors.
- Touching a bundled plugin source makes `ensureOxlintConfig()` rebuild the shared outputs.
- No plugin prebuild is required.
- Every comparable JavaScript output retains its recorded SHA-256 hash,
  and all declaration outputs are restored.
- Markdown lint passes for every changed document.
- No unrelated concurrent change or ignored build output is staged.
- The final plan and troubleshooting document match implemented behavior and verification evidence.
