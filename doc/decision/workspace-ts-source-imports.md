# Workspace packages import each other's TypeScript source through the `/ts` subpath

## Status

Accepted 2026-09-06.
Owner decision during the issue #486 grilling recorded in `doc/planning/issue-486-workspace-ts-source-imports.md`.
Rule ST3 in `AGENTS.md` points here.
Complements `.out-of-scope/typescript-project-references.md`,
which records why project references are not the type-checking topology (rejected 2026-05-14 and again 2026-09-06).

## Context

Every workspace package exports `"./ts": "./src/index.ts"`,
61 of them also `"./ts/*": "./src/*"`,
and cross-package imports name that subpath (1789 sites on 2026-09-06).
The convention entered `AGENTS.md` in commit `e5bc11cbc`;
its rationale lived only in the out-of-scope note until now.
Issue #486 asked whether oxlint and tsc actually handle it,
prompted by the cost of the project-owned oxlint rule `prefer-readonly-parameter-types`.

Measured on 2026-09-06 (details and run captures in the planning doc):

- 148 package programs hold a median of 95 sibling source files;
  122 hold more sibling files than their own.
- `lint:types` for a 126-file program:
   0.46 s;
  for `file-enforcer` (137 own,
   151 sibling):
   0.9 s with source siblings and 0.9 s with declarations.
- Warm `lint:oxlint` per package with source siblings versus built declarations:
  `file-enforcer` 1.4 s against 1.4 s,
  `pi-plugin/auto-mode` 3.1 s against 3.2 s,
  `mcp/mvm` 1.2 s against 1.1 s.
  Finding sets byte-identical in all three.
- Cold `lint:oxlint` per package:
  `auto-mode` 22.0 s against 5.4 s,
  `file-enforcer` 8.1 s against 5.2 s,
  `mvm` 7.4 s against 7.4 s.
- Cold whole-repo `lint:oxlint`:
   261 s to 305 s with the rule on,
  13 s to 15 s with it off by config;
  warm 63 s to 105 s with it on.

## Decision

Keep `/ts`.
Cross-package imports resolve to TypeScript source for tsc,
oxlint's type-aware runs,
rolldown bundles,
tests,
Node at runtime,
and editors.
No build precedes type-check or lint.

## Why source imports are the right default here

- **Buildless cross-package work.**
  A sibling edit is visible to every consumer's type-check,
  lint,
  test,
  and bundle without a build step or a dependency order.
  The root fan-out in `mise.toml` (`fanout_packages`) runs package tasks in listing order with no ordering,
  and nothing in the repo needs one because nothing consumes a sibling's artifact.
- **Type-checking the closure is cheap under TypeScript 7.**
  The classic reason to consume declarations is cold type-check time.
  Here the whole 126-file program checks in under half a second,
  and swapping 151 sibling sources for 5 declaration bundles changed `file-enforcer`'s `lint:types` by nothing.
  This re-measures,
  at 151 packages,
  the reasoning the out-of-scope note recorded at 95.
- **Live effect analysis for the readonly rule.**
  The rule proves parameter effects from callee implementations.
  Sibling source lets it prove workspace calls mechanically,
  which is what retired the hand-maintained effect catalogs
  (`doc/planning/prefer-readonly-traversal-narrowing.md`,
   2026-07-15).
  Built declarations would push workspace calls onto the shipped-implementation path,
  which needs the sibling's dist present before lint.
- **Self-contained artifacts without publishing every sibling.**
  Builds inline `@monochromatic-dev/**` from source (`deps.alwaysBundle`),
  and `./ts` is stripped at publish (`doc/decision/npm-publishing.md`),
  so a published package carries what it needs and never advertises source it does not ship.
- **A subpath works in every resolver without flags.**
  Node ignores unknown export conditions unless `--conditions` is passed,
  and TypeScript honours `customConditions` only under `bundler`,
  `node16`,
  or `nodenext` resolution.
  A subpath resolves identically in Node,
  tsc,
  `oxc_resolver`,
  rolldown,
  and editors,
  and it is greppable:
  the `test-import` rule can tell a package's own `/ts` from a sibling's.
- **Node runs it.**
  Node refuses `.ts` under `node_modules`,
  but pnpm workspace links resolve to real paths outside it,
  so tests importing `@monochromatic-dev/test-harness-core/ts` run under Node 26 unchanged
  (`doc/decision/npm-publishing.md`).

## Prior art

Surveyed at pinned commits in `doc/research/typescript-monorepo-cross-package-imports.md`.

- **Effect** exports `"."` as `./src/index.ts` in-repo and swaps to `dist` through `publishConfig.exports` at publish.
  Same shape as this repo's `./ts` plus publish-time stripping;
  Effect checks types with references,
  this repo with per-package programs.
- **Turborepo** documents "just-in-time" internal packages whose `exports` reference `.ts` directly with no build,
  naming the consequence this repo accepts:
  a dependent's type-check reports errors in the internal dependency.
- **Nx** (`@myorg/source`),
  **Vitest** (`__vitest_source__`),
  and **Babel** (`babel-src`) give editors,
  bundlers,
  and tests source resolution through a project-private export condition.
  This repo gets the same through a subpath,
  which also reaches Node and tsc without configuration.
- **JSR** publishes TypeScript source directly and maps `exports` straight to `.ts`.
- No surveyed repo checks sibling source once per consumer;
  those with type-aware lint (Sentry,
   typescript-eslint,
   Babel) read sibling declarations after a build.
  The measured cost of doing otherwise here is under a second per package,
  which is why the difference is accepted rather than engineered away.

## Costs accepted

- **Sibling source is checked under the consumer's tsconfig.**
  A lib mismatch once broke node-only consumers of DOM-touching siblings,
  so every package extends the `/dom` config (`doc/troubleshooting/typescript.md`);
  8 packages on the plain base are the standing exception list.
  A sibling's type error fails every consumer's `lint:types`.
  Accepted 2026-09-06 (round 5 of the grilling).
- **The readonly rule has a cold-only term that grows with sibling source in the program.**
  Measured per package on 2026-09-06 (`auto-mode` 22 s against 5 s with declarations;
  `mvm` unchanged);
  warm cost does not depend on the closure.
  The first hypothesis,
  that the persistent cache re-summarizes every sibling once per consuming package,
  overstated it:
  a cache census after a cold sweep found 2454 entries for 1834 distinct files,
  so about a quarter are duplicates across projects,
  and a prototype that re-keyed entries by analysis environment produced no cross-project hits
  because the envelope also validates whole-program surfaces
  (file list,
  declaration surface,
  compiler options) that differ per program
  (`doc/planning/issue-486-workspace-ts-source-imports.md`,
   prototype section,
   and `doc/planning/issue-486-summary-share.patch`).
  The remedy stays inside the rule and is not yet identified;
  tracked under issue #374.
- **`./ts` is stripped at publish**;
  consumers outside the repo use built artifacts only.
- **Declaration bundling of sibling source under TypeScript 7** needed rolldown-plugin-dts 0.27.4 with explicit Oxc
  (`doc/troubleshooting/rolldown-plugin-dts-typescript-7-generator.md`).
- **A custom test-import rule** exists because oxlint's native import restriction cannot tell own `/ts` from a sibling's
  (`package/oxlint-plugin/test-import/README.md`).

## Alternatives rejected

- **Project references**:
   `.out-of-scope/typescript-project-references.md`.
- **Compiled boundary** (tsc and type-aware lint read sibling declarations;
   the Sentry,
   typescript-eslint,
   and Babel pattern):
  buys isolation,
  not speed;
  warm cost did not move in three packages,
  and it needs the build ordering the fan-out lacks plus 1789 rewritten sites or a condition rollout.
  Isolation was accepted instead.
- **Rule-only boundary** (siblings treated like locked packages):
  reverses the 2026-07-15 live-analysis decision for a cold-only gain that the cache fix targets without it.
- **Single root program with `paths`** (Vue core,
   Vitest):
  does not shrink the per-package closure the rule analyses,
  and collides with oxlint's root-tsconfig discovery trap (`doc/troubleshooting/oxlint.md`).
- **Repo-wide measurement of the compiled boundary**:
   declined 2026-09-06 as not decision-relevant once isolation was accepted.

## Revisit triggers

Measure first,
then reopen;
never reopen from best-practice reasoning.

- Warm single-package `lint:oxlint` or `lint:types` above 10 s whose difference against the built-declaration variant
  (recipe in the planning doc) exceeds the run-to-run band.
- Cold whole-repo `lint:oxlint` still above target after the rule's cross-project summary sharing lands.
- A measured need for owner-attributed type errors that the `/dom`-everywhere convention cannot meet.
