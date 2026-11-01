# tsdown 0.22: pointing two configs at one outDir lets each run's default clean delete the other's output; self-inflicted here, and the reason this repo gives every bundling config its own `dist/` subdir

When a package builds two bundles into the same directory with two tsdown configs (here:
Electron main process ESM and sandboxed preload CJS,
 both into `dist/app`),
 the second run's
default clean pass removes the first run's artifacts.
The build reports success;
 the missing file is only discovered when the app fails to start or
a later stage misses an input.

This is not a tsdown defect.
The repo's existing convention of one `dist/` subdir per bundling config
(`dist/app`,
 `dist/tools`,
 `dist/final` in the electron packages) exists precisely to keep each
config's clean scope disjoint;
 the collision happened because
`package/file-manager/variant-electron`'s preload config was pointed at `dist/app`,
which the main-process config already owns,
 instead of getting its own subdir.

Found while building `package/file-manager/variant-electron`:
 the preload build printed
`Cleaning 1 files` and deleted `dist/app/main.mjs` produced seconds earlier.

## Symptom

```txt
ℹ Build start
ℹ Cleaning 1 files
✔ Build complete in 11ms
```

in the SECOND tsdown invocation,
 followed by the prior bundle being absent from the shared
outDir.
 No error,
 no warning naming the deleted file.

## Root cause

Two layers,
 and the actionable one is ours.

The mechanical layer:
 tsdown resolves `clean` to `true` by default and expands `true` to the whole outDir.
From the installed package (tsdown 0.22.4 read from the pnpm store;
 behavior observed
identically under 0.22.3,
 which the builds print):

`tsdown/dist/options-BFfWGQT3.mjs:720` (option defaults):

```js
let { entry, format, plugins = [], clean = true, ... , outDir = "dist", ... } = ...
```

`tsdown/dist/options-BFfWGQT3.mjs:57`:

```js
if (clean === true) clean = [slash(outDir)];
else if (!clean) clean = [];
```

and `cleanOutDir` (`options-BFfWGQT3.mjs:30`) globs those paths and removes matches.
Each tsdown invocation knows only its own config,
 so a sibling config's artifacts in the same
outDir are indistinguishable from stale output.

The causal layer:
 the repo already structures electron packages so this cannot happen.
`dist/app` is the staged runtime directory with several producers by design
(the main-bundle tsdown config,
 the renderer `tsc` emit,
 and the `build:stage` copy step),
 but only ONE of those producers cleans:
 the main-bundle tsdown config,
 which runs first,
 immediately after the package's explicit `build:clean` task,
 so its clean is a no-op on an already-empty directory.
Every other tsdown config gets its own subdir (`dist/tools` for package tools,
 and now `dist/preload` for the preload bundle),
 so each config's default clean only ever touches artifacts that config itself produced.
Pointing the preload config at `dist/app` broke that invariant;
 the deletion followed from documented option behavior.

## Verification

Environment:
 tsdown 0.22.3 at build time (catalog `>=0.22.3`),
 configs
`tsdown.main.config.ts` (esm,
 `outDir: 'dist/app'`) and `tsdown.preload.config.ts` (cjs).

- Preload config pointed at `dist/app` with both configs on default clean:
   running main then preload leaves only `preload.cjs`;
  `dist/app/main.mjs` is gone (the symptom log above).
- Preload config on its own `outDir: 'dist/preload'` with default clean (the shipped fix):
   `mise run //package/file-manager/variant-electron:build:js:preload` alone prints
  `Cleaning 1 files` scoped to `dist/preload` and `dist/app/main.mjs` survives;
   the full package `test` task (unit suites plus the nested-Wayland boundary test) exits 0.

## Verified workarounds

- The shipped fix,
   matching the repo convention:
   give the second config its own subdir (`outDir: 'dist/preload'`),
   keep default clean,
   and let the package's `build:stage` step copy `preload.cjs` into the staged
  `dist/app` directory (`src/build-stage.ts`),
   with `build:clean` also removing `dist/preload`.
  Tradeoff:
   one extra copy in the stage step and the bundle existing twice on disk;
   in exchange every config's clean scope is disjoint and rebuild order stops mattering.
- The first-shipped,
   since-replaced patch:
   `clean: false` on every config after the first one writing into a shared outDir,
   with directory hygiene owned by the up-front `build:clean` task.
  Tradeoff:
   stale files from renamed entries linger until the explicit clean step runs,
   and the trap stays armed:
   any future config pointed at the shared directory with default clean,
   or a reordering that runs the cleaning config later,
   re-breaks silently.

## What does not work

- Relying on invocation order or timing:
   the clean is unconditional per run;
   the second run
  always wins.

## Upstream filing decision

`.out-of-scope/` was checked:
 no tsdown exemption exists.

1. Really upstream's fault?
    No;
    `clean: true` defaulting to the config's own outDir is
   reasonable,
    documented option behavior,
    and the multi-config-one-outDir arrangement violated this repo's own
   one-subdir-per-config convention.
2. Can upstream fix it?
    Conceivably (e.g. cleaning only files the config would emit),
    but that
   is a design change,
    not a defect fix.
3. Supported use case?
    Multiple configs are supported;
    sharing one outDir between them with
   default clean is not a documented pattern.
4. Would the repo welcome the contribution?
    Not evaluated;
    constraint 1 fails.
5. Will they likely fix it?
    Nothing to fix.
6. Prototyped minimal fix?
    Not applicable;
    the consumer-side arrangement recorded above removes the collision.

Decision:
 nothing to file.
