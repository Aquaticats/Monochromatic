# `lint:types` exits 0 without running tsc after `task-util` testing seam

Once `task-util` was rebuilt after commit `b7ae0cb92`
(`feat(task-util): add built testing seam`, 2026-08-18),
every package `lint:types` task passed without type-checking anything
until `edd680e89`.
The broken `dist/final/node/tsc-filter.mjs` in use was built on 2026-09-09;
earlier rebuilds are not recorded.

## Symptom

`mise run //package/<path>:lint:types` printed only the dispatcher line
and exited 0,
even with a deliberate `const x: number = 'not a number'` in package source.

## Root cause

The `lint:types` task template in the root `mise.toml` runs
`runWorkspaceNode('package/monorepo-task/core-exec', 'tsc-filter', ['--build'])`,
which executes the built `package/monorepo-task/core-exec/dist/final/node/tsc-filter.mjs`
whenever it exists.

`src/tsc-filter.ts` runs its command-line body behind `if (import.meta.main) await main();`
so importing it stays side-effect free.
Commit `b7ae0cb92` made `src/testing.ts` re-export filter helpers from `./tsc-filter.ts`.
Both files are rolldown entries in `rolldown.node.config.ts`,
so rolldown moved the shared module,
including the `import.meta.main` guard,
into a hashed chunk and reduced the `tsc-filter.mjs` entry to re-exports.
Inside that chunk `import.meta.main` is false,
so `main()` never ran and `tsc` was never spawned.

The expensive CLI tests in `src/tsc-filter.expensive.unit.test.ts` ran only the source file,
where the guard is true,
so they kept passing.

## Fix

`edd680e89` moves the pure diagnostic helpers into `src/tsc-output-filter.ts`,
which `testing.ts` imports,
so nothing imports the command-line entry and it stays a standalone chunk.
A new expensive test runs the built `dist/final/node/tsc-filter.mjs`
against a type error and requires a non-zero exit;
it failed on the old build and passes on the rebuilt one.

## Prevention

Any rolldown entry whose body sits behind `import.meta.main` must not be imported by another entry.
Keep reusable helpers in sibling modules,
as `oxlint-wrapper.ts` does with `oxlint-augment.ts` and `oxlint-fix-loop.ts`.

## Consequences

Type errors introduced while the check was silent are now reported.
The Pi 0.87 breakage in [`pi-0-87-extension-api-breakage.md`](pi-0-87-extension-api-breakage.md)
is one case the check would have caught.
