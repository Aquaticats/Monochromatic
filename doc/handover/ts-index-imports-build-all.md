# Handover: index-only /ts cross-package imports, and build every package

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/deps-cube` is now `package/dependency-catalog/cli-cube`;
> `package/module/or-throw` is now `package/or-throw/core`;
> `@monochromatic-dev/module-fs-path` is now `@monochromatic-dev/fs-path-core`;
> `package/module/logger` is now `package/logger/core`;
> `@monochromatic-dev/module-logger` is now `@monochromatic-dev/logger-core`.

Live progress for the refactor described in the approved plan at
`/home/user/.claude/plans/make-all-imports-between-keen-piglet.md`.
 Read that plan first;
 this doc tracks what is
done,
 what is next,
 and the gotchas a fresh session needs.

Branch:
 `refactor/ts-index-imports-build-all` (created off `main`).
 Commit after each package;
 use explicit scoped
pathspecs (`cli-git` rejects `git add -A`/`.`).
 Commit trailer:
 `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`
(the harness-specified form;
 an earlier "(1M context)" variant appears in older commits,
 harmless).

## Goal and locked decisions

Make every non-exempt cross-package import in `package/` resolve to TypeScript source via the package index only:
`@monochromatic-dev/<pkg>/ts`.
 Then build every package that has a source-to-dist transformation.

1. Import form:
    index only (`<pkg>/ts`).
    Anti-patterns to rewrite:
    bare (`<pkg>`),
    feature-named (`/tagged`,
   `/scope`),
    `<feature>/ts` (`/scope/ts`),
    and deep (`/ts/<file>.ts`).
    All collapse to `<pkg>/ts`.
2. Export maps:
    keep `./ts` and `./ts/*`;
    remove feature-named entries (Phase B2).
    Keep `.` (built dist).
3. Build shape:
    only `neutral` and `node` bundles,
    from a single `src/index.ts`.
    Per package pick neutral-only
   (exemplar `module-test`;
    `.` = `{types,default}` both neutral),
    node-only (exemplar `pi-shared-model-selection`,
   `module-matrix`,
    the `pi-*`;
    `.` = `{types,default}` both node),
    or both (exemplar `module-es`,
    `module-logger`;
   `.` = `{types: neutral, node, default: neutral}`).
    No per-feature bundles.
    Client bundles only for webapps,
   bin bundles only for CLIs.
4. Everything with a source-to-dist transformation must build a dist,
    even though the workspace consumes `/ts` source.
5. Tree scope:
    `package/` only.
6. `config-oxlint` family:
    builds a `node` dist like any source-to-dist package (decision 4 applies).
    Its consumers,
   notably the repo-root `oxlint.config.ts`,
    import the built `.` on purpose:
    loading the prebuilt config is 20% to
   40% faster than evaluating `/ts` source on every lint run.
    It is exempt only from the import-rewrite-to-`/ts` rule
   (decision 1),
    not from building,
    so it is NOT source-only.
7. CLI/app packages lacking a build task:
    add bundled-bin builds.
8. Shims (`shim-*`) and stubs (`stub-*`):
    leave hand-written `.cjs`/`.d.cts` as-is.
9. Executable-index packages (`cli-mvm`,
    `dev-script-inference-canary`):
    restructure to a library barrel `index.ts`
   plus a separate executable file (done in Phase A).

## Exemptions (imports of them NOT rewritten to `/ts`)

The codemod skips these targets;
 their ~72 imports stay verbatim.
 Two distinct reasons,
 do not conflate them:

Not built AND not rewritten (pure data or curated source,
 no source-to-dist transformation):
`config-typescript`,
 `config-dprint`,
 `config-stylelint`,
 `config-cosign`,
 `config-tofu`,
 `config-dotfiles`;
`config-tsdown` (circular);
 `claude-code-plugin-source` (curated source named subpaths consumed by sibling plugin
bins);
 `test-fixture-*`;
 `shim-*`;
 `stub-*`.

Built BUT not rewritten:
 the `config-oxlint` family.
 It has a real source-to-dist build
(`tsdown.node.config.ts`,
 dist under `dist/final/node`),
 so decision 4 applies and it must build.
 Its consumers
import the built `.` deliberately because the prebuilt config loads 20% to 40% faster than `/ts` source per lint run.
Its imports are not rewritten to `/ts`,
 but it is not source-only.

## Verification helpers

- Per-package type-check:
   `mise run //package/<path>:lint:types`.
   Lint:
   `:lint:oxlint`.
- `mise.toml` is GENERATED from `mise.no-env.toml` by file-enforcer.
   Never edit `mise.toml` directly;
   edit
  `mise.no-env.toml` then run `mise run file-enforcer` to regenerate.
   (`CLAUDE.md` is likewise generated from `AGENTS.md`.
  )
- Resolution analysis (the measurement script used throughout):
   a Node script that walks `package/**/*.ts`,
   resolves
  each `@monochromatic-dev/*` specifier against the target's `package.json` `exports`,
   classifies src/dist,
   and skips
  self-imports.
   Re-run it after Phase B to confirm 0 dist-resolving and 0 unresolved cross-package imports.
   The Phase A
  gate ("every non-exempt cross-package import target exposes .
  /ts") already PASSES.

## Phase A: DONE

Commits on the branch (newest last):

- `build(<pkg>): expose ./ts index source subpath` for 14 source-only libraries (added `./ts` + `./ts/*`):
  `module-or-throw`,
   `module-const`,
   `module-async-time`,
   `agent-harness-shared-current-time-context` (renamed from `module-current-time-context`),
   `module-numeric-format`,
  `module-function-arity`,
   `module-i18n-compose`,
   `module-toml-edit`,
   `module-image-diff`,
   `module-token-count`,
  `module-fs-path` (also keeps its `./find-monorepo-root`/`./find-package-root` feature entries for now),
  `mcp-stdio`,
   `claude-code-plugin-hook-type`,
   `cli-terminal-exec` (its `.`/`./ts` point at `src/launch.ts`,
   the
  library entry;
   `index.ts` is its bin).
   `module-dom` already had `./ts` + `./ts/*` (no change needed).
- `refactor(pi-shared-model-selection): re-export all feature modules from index` (index now `export *` of core,
  scope,
   cost,
   budget,
   pi-coding-agent;
   overlapping types resolve to identical `types.ts` bindings).
- `refactor(cli-mvm): split library barrel from CLI executable` (`src/index.ts` -> barrel of the 10 command modules;
  executable moved to `src/cli.ts`;
   `bin` + package `mise run` repointed;
   added `.`/`./ts`/`./ts/*`;
   feature subpaths
  like `/clone` kept for Phase B2 removal).
- `refactor(dev-script-inference-canary): split library barrel from canary executable` (`src/index.ts` -> barrel of
  `linter-artifacts` + `runner-types`;
   executable moved to `src/canary.ts`;
   package + root `mise.no-env.toml` `canary`
  task repointed and `mise.toml` regenerated;
   added `.`/`./ts`/`./ts/*` PLUS a TEMPORARY `./src/*` passthrough so the
  viewer's existing deep `/src/*.ts` imports keep resolving until Phase B2).

Recommended confirmation before Phase B:
 a repo-wide `mise run lint:types` (per-package checks passed for every
touched package and the inference-canary-viewer consumer;
 additive exports cannot change existing type-checking).

## Phase B: DONE — every non-exempt cross-package import resolves to `/ts`

Approach decided WITH the user:
 NOT a heavy AST codemod.
 A minimal literal-replace tool did the specifier swap;
 the
duplicate-import merges (36 sites across 4 targets) were hand-merged.
 Final resolution sweep:
 0 cross-package imports
resolve to non-`/ts`.
 The only remaining bare `@monochromatic-dev/*` specifiers are SELF-imports (a package importing
itself by name,
 plan-excluded;
 includes the intentional "tests import the built dist" pattern) and TSDoc/README
`@example` comments (deferred,
 see "Doc examples" below).

### The tools (repo root, UNTRACKED, throwaway)

`mise.rewrite-ts-imports.ts` — run per target:
 `bun mise.rewrite-ts-imports.ts <target-unscoped> [--dry]`.
 Prints the
changed-file list to stdout,
 a summary to stderr.
 Walks `package/**/*.{ts,tsx,mts}` (skips `dist`,
 `node_modules`),
skips files OWNED by the target (self-imports),
 rewrites `@monochromatic-dev/<target>[/sub]` to `.../ts` ONLY in
import/export context (a `from`/`import`/`import(` with a word-boundary before it).
 Critical safety guard:
 never
touches bare string-literal package names (verified skips:
 `config/oxlint/.../allow-pkg.ts`,
`pi/auto-mode/.../git-worktree-read-allowlist.ts`).
 Boundary-safe,
 idempotent.

`mise.detect-merge-sites.ts` — run per target:
 reports files that would have DUPLICATE import statements after collapse
(condition:
 `valueCount + typeCount > 1` for the target's `/ts`;
 see the rule finding below).
 Used as the merge oracle
(detector -> 0 after merging),
 cross-checked against real scoped oxlint.

Both decide-on-keep-or-delete at the very end (after Phase C).

### Staging: paths contain SPACES (module/es dir names like `t object/...`)

`git add $(cat list)` / `$(cat list)` WORD-SPLITS and breaks.
 Use `git commit --pathspec-from-file=<list> -m ...`
(one path per line,
 spaces preserved;
 cli-git accepts it as explicit pathspecs).
 For module-logger the full set was
captured with `git diff --name-only` (tool's 38 + 26 hand-merges + the cli-git pilot = 65 files;
 untracked scratch
tools excluded automatically).

### The `/dom` prerequisite (NO new switches were needed during Phase B)

Importing a built package's `/ts` drags its dependency SOURCE into the consumer's type-check;
 `module-logger`'s
`src/sinks/opfs.ts` uses DOM APIs (`FileSystemWritableFileStream`,
 `navigator.storage`),
 so base-config consumers
without the DOM lib fail TS2304/TS2339.
 Fix (doc/troubleshooting/`typescript.md` §"All packages must extend
config-typescript/dom"):
 switch the consumer tsconfig to `config-typescript/dom`.
 The 9 base-extenders importing
module-test were switched in Phase A's prep (commit d4536cf8).
 NO further switches were needed:
 every built-package
flip (module-kv-store,
 figma-kiwi,
 ...,
 module-logger itself) kept repo-wide `lint:types` GREEN,
 because the remaining
module-logger consumers were already on `/dom`.
 (If a future flip DOES break,
 the lint:
types error cites `opfs.ts`;
 the
reported consumer package varies run-to-run due to tsgo `--build` incremental caching,
 but the fix is always "switch
that consumer to /dom" and commit it with the target.
)

### The duplicate-import rule — EMPIRICAL findings (correct the earlier guess)

The active rule is `eslint/no-duplicate-imports`,
 NOT `import/no-duplicates`,
 and the earlier "separate value/type is
fine under prefer-inline:
false" note was WRONG.
 Validated by running scoped `mise run //package/git-policy/cli:lint:oxlint`
on a hand-merged pilot file:

- A separate `import type { X } from '.../ts'` alongside a value `import { ... } from '.../ts'` (same specifier) IS a
  duplicate.
   The fix is ONE merged statement with type bindings folded inline as `type X` specifiers
  (e.g. `import { logger, tagged, type Logger } from '@monochromatic-dev/module-logger/ts';`).
   This is
  valid under `verbatimModuleSyntax: true`.
   Aliases preserved as `type Logger as ModuleLogger`,
   `logger as defaultLogger`.
- Therefore a file with `value=1 type=1` (one value import + one type import collapsing to the same `/ts`) IS a merge
  site.
   The detector condition is `value + type > 1`.
- `includeExports` is effectively false:
   an `export ... from '.../ts'` re-export does NOT conflict with an `import`
  from the same specifier (validated:
   `inference-canary-viewer/src/data/viewer-types.ts` has both,
   oxlint clean).
- dprint `importDeclaration.forceMultiLine: "whenMultiple"` (package/config/dprint/`index.json`):
   a multi-specifier
  import MUST be multi-line,
   one binding per line.
   Single-binding imports stay inline.
   Merged blocks follow this;
   `mise
  run lint:dprint` confirmed none of the merged files were flagged.

PRE-EXISTING oxlint debt is IGNORED (user instruction);
 the gate is repo-wide `lint:types` GREEN.
 The only oxlint
constraint observed is "do not ADD a `no-duplicate-imports` violation",
 handled by the merges above.

### Progress — ALL targets committed (newest last)

Pilot + prep (prior session):
 `06b6afca` favicon dynamic->static;
 `d4536cf8` 9 tsconfigs to /dom;
 `35459241`
module-test (307 files).

This session,
 in dependency-risk order (each its own commit,
 repo-wide lint:
types GREEN after each):
- Wave 1,
   pure src->src no-op source libs (no merge):
   `module-or-throw`,
   `module-const`,
   `module-async-time`,
  `agent-harness-shared-current-time-context` (renamed from `module-current-time-context`),
   `module-numeric-format`,
   `module-toml-edit`,
   `mcp-stdio`,
  `claude-code-plugin-hook-type` (note:
   package NAME is `@monochromatic-dev/claude-code-plugin-hook-type`),
  `cli-terminal-exec`.
- Wave 2,
   bare imports of built packages (dist->src flip,
   no merge):
   `module-kv-store`,
   `module-async-iter`,
  `module-matrix`,
   `module-zip-writer`,
   `module-memoize`,
   `figma-kiwi`.
- Wave 3,
   feature/deep/merge targets:
   `module-fs-path` (`/find-monorepo-root`),
   `dev-script-inference-canary`
  (deep `/src/*.ts`),
   `build-tool-css` (`/ts/process-shim`;
   the redundant process-shim side-effect import was dropped,
  since importing applyMixins from the index runs the shim transitively),
   `cli-mvm` (8 features,
   3 merges),
  `pi-shared-model-selection` (5 features,
   6 merges),
   `module-logger` (108 specifiers / 65 files / 26 merges;
   commit
  `e671ac97`).

Targets with 0 cross-package specifiers (nothing to do):
 `module-es`,
 `module-observable`,
 `module-function-arity`,
`module-i18n-compose`,
 `module-image-diff`,
 `module-token-count`,
 `module-hyperscript`.
 (Their only bare specifiers are
self-imports.
)

### Doc examples — DEFERRED (not code imports; out of the rewrite scope, fix in a docs pass)

TSDoc/README `@example` blocks still show old import forms.
 Known:
 `module/or-throw/README.md` (bare),
`module/logger/src/tagged.ts:17` (`/tagged`,
 escaped `\@`),
 `claude-code-plugin/hook-type/src/index.ts:11`
(`@monochromatic-dev/claude-code-hook-types` — ALSO a WRONG package name,
 missing `plugins`).
 These do not affect
lint:
types or oxlint.
 Update them when convenient (the `/tagged` one becomes unimportable after Phase B2).

## Phase B2: DONE (5 of 7 packages; 2 deferred to Phase C)

Removed feature-named (and `<feature>/ts`) export entries,
 leaving `.`,
 `./ts`,
 `./ts/*` (where it existed).
 Verified by
the resolution sweep (still 0 in-scope non-`/ts` cross-package imports) + a repo-wide non-`.ts` reference grep (0
references in `package/` to any removed entry;
 the only hits are `package-paused/` (out of scope),
 `doc/`,
 READMEs,
and the throwaway tool) + a JSON-parse/keys check on each edited file.
 NOTE:
 `lint:types` is a WEAK B2 gate (a
`package.json`-only edit may not invalidate a consumer's tsbuildinfo,
 so green can be stale);
 the sweep + 0-consumer proof
is the real verification.
 It was green anyway.

Done (one commit each):
 `module-logger` (`./logger`,
 `./tagged`,
 `./types`);
 `pi-shared-model-selection` (all 5
per-feature dist entries `./core`/`./scope`/`./cost`/`./budget`/`./pi-coding-agent` AND their `/ts` twins — leaving
`.`+`./ts`;
 the per-feature dist BUNDLES are collapsed to a single index in Phase C);
 `cli-mvm` (10 feature entries);
`module-fs-path` (`./find-monorepo-root`,
 `./find-package-root`);
 `dev-script-inference-canary` (the temporary `./src/*`
passthrough).

DEFERRED to Phase C (structural,
 not simple dead-entry pruning):

- `cli-vmsync`:
   its `exports` is feature-ONLY (`./import`,
   `./boot`,
   `./sync`,
   ...),
   with NO `.`/`./ts`/`./ts/*`.
   It was
  never a Phase B target (0 cross-package consumers) and never got Phase A's `./ts`.
   Removing its only exports needs the
  Phase A `./ts` add + the Phase C bin-build treatment together.
   All its feature exports are confirmed unimported.
- `build-tool-css`:
   has explicit `./ts/fs-registry` + `./ts/process-shim` (a `/ts/<file>` form) and NO `./ts/*`
  wildcard.
   Both are confirmed unimported (Phase B collapsed the one `done-postcss` consumer to `/ts`).
   Normalize to the
  canonical `.`+`./ts`+`./ts/*` shape (replace the two explicit entries with a single `./ts/*`) during its Phase C build
  review.
   (The advisor's "build-tool-css needs no B2 change" assumed it resolved via an existing `./ts/*`;
   the file has
  no `./ts/*`,
   so the explicit entries are the thing to normalize,
   just not in B2.
  )

## Phase C: IN PROGRESS — convert source-only and bin packages to builds

Everything imports `/ts`,
 so flipping `.` from src to dist affects no in-repo consumer.
 Precedent:
 commit `7f4b5e0`
(`markdownlint-no-pipe-tables`,
 node-only).
 Neutral exemplar:
 `module-test`.

### PROVEN recipe per neutral source-only lib (validated on `module-async-time` + `module-const`)

1. Add `tsdown.browser.config.ts` (one line):
    `export { default, } from '@monochromatic-dev/config-tsdown/.ts';`
   (neutral).
    For node use `tsdown.node.config.ts` -> `'@monochromatic-dev/config-tsdown/.node.ts'`.
2. `mise.toml`:
    add `build`,
    `build:js`,
    `build:js:browser` (or `build:js:node`) + the three `watch:*` twins via
   `extends`,
    and a `buildAndTest` task.
    CAVEAT:
    the test-task wiring VARIES per package — some have
   `[tasks."test:unit"] extends = "test:unit"` (then `buildAndTest = "mise run build; mise run test:unit"`),
    others a
   custom `[tasks.test] run = "bun src/self.unit.test.ts"` (then `buildAndTest = "mise run build; mise run test"`).
   Read the existing `mise.toml` and preserve its test task;
    only ADD build + buildAndTest.
3. `package.json`:
    `"module"` -> `dist/final/neutral/index.mjs`;
    `exports["."]` -> `{ "types":
   "./dist/final/neutral/index.d.mts", "default": "./dist/final/neutral/index.mjs" }` (node:
    swap `neutral`->`node`);
   keep `./ts` + `./ts/*`;
    `files` -> add `"dist/final"` before `"src"`;
    add `"@monochromatic-dev/config-tsdown":
   "workspace:*"` as the first devDep.
4. Switch the package's own unit tests to import the BUILT dist instead of sibling source.
    PREFERRED form:
    switch the
   specifier to the package BY NAME (`@monochromatic-dev/<pkg>`),
    which resolves to `.` = dist after the flip.
    This is
   UNIFORM (one target string regardless of the test file's directory depth,
    so it works for nested-`src/` packages
   like `module-es` too,
    unlike a relative `../dist/...` path which the precedent used).
    KEY:
    tests that ALREADY import
   by name (e.g. `module-function-arity`'s) need NO edit at all -- they follow `.` automatically.
    Only relative source
   imports (`} from './foo.ts';`) need switching,
    to `} from '@monochromatic-dev/<pkg>';`.
    Leave test-aggregator
   imports (`import './foo.unit.test.ts'`),
    `module-test/ts`,
    and other-package `/ts` imports alone.
    The dist index
   must re-export every symbol the tests use;
    if not,
    `buildAndTest` fails loudly naming the symbol -> add the
   re-export to the index (finishing Phase A) OR keep that one test on source if it tests a genuine internal.
    If a
   single test file imports TWO+ own source modules,
    switching both to by-name creates a duplicate import -> merge them
   into one statement (same as Phase B's eslint/no-duplicate-imports merges).

   SCALING:
    packages with one test file per function (`module-or-throw` = 25,
    `module-toml-edit` = 18,
   `module-i18n-compose` = 9) make manual per-file switching expensive.
    For those,
    consider a small throwaway tool
   (`*.unit.test.ts` only:
    rewrite a relative import of an own non-test source module to `@monochromatic-dev/<pkg>`,
   merging duplicates) and let `buildAndTest` be the oracle.
    Tests already importing by name cost nothing.
5. `pnpm install` (links the new config-tsdown devDep;
    updates root `pnpm-lock.yaml`).
6. Gate:
    `mise run //package/<path>:buildAndTest` (build emits `dist/final/neutral`,
    tests pass against it).
7. Commit:
    `git add <tsdown.config>` (untracked),
    then `git commit <package.json> <mise.toml> <tsdown.config>
   <switched test files> pnpm-lock.yaml -m "build(<pkg>): produce neutral dist"`.

### Findings (de-risk the fan-out)

- isolatedDeclarations:
   0 new errors.
   `config-typescript` already runs a declaration-emitting types build under
  `lint:types`,
   so the source already satisfies it.
   The plan's "expect to add return types" caution is STALE.
- No build/test cycle stall:
   `module-test` is consumed via `/ts` source,
   so building a lib never needs module-test's
  dist and vice versa.
- `dist/` is gitignored;
   commits never include built artifacts (only `files: [dist/final, src]` for publish).
- GATE IS build + buildAndTest PER PACKAGE,
   not lint:
  types.
   Do NOT run repo-wide `lint:types` mid-batch:
   converted
  tests import `../dist/.../index.mjs` whose `.d.mts` only exists after a build,
   so lint:
  types FALSE-FAILS on any
  package edited-but-not-yet-built.
   Run repo-wide lint:
  types only as a FINAL check after all dist exist.
   (Same reason
  `git clean -dX` then lint:
  types breaks until rebuild:
   inherent to the precedent's pattern.
  )
- First-with-deps bundling CONFIRMED:
   the neutral config (`package/config/tsdown/src/index.ts`) sets
  `deps: { alwaysBundle: ['@monochromatic-dev/**'] }`,
   so workspace deps are INLINED into the self-contained bundle
  (verified:
   `module-logger`/`numeric-format`/`toml-edit` dist `.mjs` have zero `@monochromatic-dev` imports).
   External
  npm deps (e.g. `toml-eslint-parser`) stay EXTERNAL (one bare import in the dist,
   resolved from the consumer's
  node_modules).
   Consequence:
   build order is irrelevant;
   a lib builds standalone regardless of whether its workspace
  deps are built yet.
- DEP CLASSIFICATION decides the bundle shape (tsdown defaults):
   `dependencies`/`peerDependencies` are
  EXTERNALIZED (bare import in dist,
   consumer installs);
   `devDependencies` are AUTO-BUNDLED inline (consumers never
  install devDeps).
   The config's `alwaysBundle`/`neverBundle` override this.
   So a runtime dep that must ship inlined
  belongs in devDeps OR `alwaysBundle`;
   a dep meant to stay external stays in `dependencies`.
   For each dep-heavy
  package,
   classify accordingly and VERIFY by grepping the built dist's `import ... from` lines (the method used for
  toml-edit/module-logger).
   Do NOT reclassify a package's existing deps without reason;
   respect its split + the config
  overrides.
- NODE config (`package/config/tsdown/src/index.node.ts`):
   `platform: 'node'`,
   `outDir: dist/final/node`,
  `alwaysBundle: ['@monochromatic-dev/**', 'find-up', 'nano-spawn']` (find-up/nano-spawn force-bundled despite being
  `dependencies`),
   `neverBundle: ['@earendil-works/pi-coding-agent', 'typebox', '@earendil-works/pi-ai']` (pi runtime
  peer deps,
   provided at load time -- bundling them causes CJS/ESM "exports is not defined").
   Relevant to pi-shared and
  any pi-* / CLI package consuming those peers.
- THE BY-NAME SMOKE-TEST LEVER (sharpens step 4):
   the dist's ONLY new risk vs source is re-export/bundle integrity;
  the source unit tests already cover the logic and bundling adds no code paths.
   So you do NOT need every public test
  routed through the dist -- you need the public surface exercised BY-NAME once (executes the bundle) PLUS the dts emit
  (which fails the build if any index re-export is missing,
   covering re-export completeness for free).
   Practical rule:
  if a package already has a by-name test (e.g. a `public-api.unit.test.ts`),
   it IS the smoke test -- flip with ZERO
  test edits.
   If not,
   switch ONE or TWO broad round-trip tests to by-name and leave the rest on source.
   Internal-symbol
  tests (symbols NOT in the index) CANNOT route through dist and MUST stay on source -- this is forced,
   not a choice.
- A THIRD throwaway tool exists:
   `mise.switch-tests-byname.ts` (repo root,
   UNTRACKED).
   `bun mise.switch-tests-byname.ts
  <pkgDir> <pkgName> [--dry]`.
   Switches a `*.unit.test.ts`'s relative own-source import to by-name,
   but ONLY for files
  with EXACTLY ONE own-source import (no-merge tier);
   files with 2+ are reported and SKIPPED for hand-merge (the
  `eslint/no-duplicate-imports` case).
   No regex (pure string scan),
   idempotent.
   Used for `module-or-throw` (25 files,
  all single-binding public).
   buildAndTest is the oracle for internal-symbol mistakes.

### Done (neutral unless noted; ALL conversions complete — repo-wide lint:types PASSED, build+test+resolution gate pending, see Final verification)

- `module-async-time` (`test:unit` variant;
   tests switched to relative-to-dist) — `build(module-async-time): produce neutral dist (Phase C pilot)`.
- `module-const` (custom `test` aggregator variant;
   5 tests switched to relative-to-dist) — `build(module-const): produce neutral dist`.
- `module-function-arity` (`test:unit`;
   test already by-name,
   NO test edit) — `build(module-function-arity): produce neutral dist`.
- `agent-harness-shared-current-time-context` (`test:unit`;
   1 test switched to by-name) — `build(module-current-time-context): produce neutral dist`.
- `module-numeric-format` (custom `test` aggregator;
   3 leaf tests switched to by-name;
   FIRST built lib WITH workspace
  deps -- module-const + module-or-throw inlined) — `build(module-numeric-format): produce neutral dist`.
- `module-or-throw` (`test:unit`;
   25 single-binding public tests switched to by-name via `mise.switch-tests-byname.ts`)
  — `build(module-or-throw): produce neutral dist`.
- `module-i18n-compose` (`test:unit`;
   `public-api.unit.test.ts` already by-name = the smoke test,
   ZERO test edits)
  — `build(module-i18n-compose): produce neutral dist`.
- `module-toml-edit` (`test:unit`;
   two clean all-public round-trips `canonical`+`toml-delete` hand-merged to by-name;
  toml-eslint-parser stays external;
   internal `tomlFloat`/`tomlInteger`/`encodeKey`/`isAttachedGap` tests stay on source)
  — `build(module-toml-edit): produce neutral dist`.
- `mcp-stdio` (NODE bundle — uses process.
  stdin/stdout;
   `private: true`;
   FIRST node build) — per user direction the
  index was EXPANDED to re-export the previously-internal `JSON_RPC_*` codes + `PROTOCOL_VERSION` + `readLines`,
   so all
  4 tests route through dist:
   json-rpc + line-reader via the tool,
   transport (3) + server (5) hand-merged;
   added the
  missing `test:unit` task — `build(mcp-stdio): produce node dist`.
- `cli-fy` (NODE bin;
   FIRST pure-CLI bin) — bin repointed to `dist/final/node/index.mjs`;
   `cli.unit.test.ts` repointed
  to spawn the built bin (13-scenario smoke-run) and its latent cwd bug fixed via `findMiseMonorepoRootCached`;
   unit
  tests (resolve,
   coerce) stay on source — `build(cli-fy): produce node bin dist` + `fix(cli-fy): resolve spawn cwd ...`.
- `module-image-diff` (NODE lib+bin;
   FIRST proven dual-entry,
   commit `943e0f1a`) — `tsdown.node.config.ts` dual-entry
  (`['./src/index.ts', './src/cli.ts']`) emits `index.mjs` (lib) + `cli.mjs` (bin);
   `.` -> node dist,
   `bin` -> `cli.mjs`.
  `similarity` switched to by-name (lib bundle smoke test);
   ADDED `cli.unit.test.ts` spawning `cli.mjs --help` (bin
  execution — `dts` can't verify `cli.mjs` since `cli.ts` has no exports);
   network test renamed to
  `client.expensive.unit.test.ts`
  (excluded from default `test:unit`) —
  `build(module-image-diff): produce dist (lib + bin, node)`.
- `cli-rgffplay` (NODE pure-CLI,
   commit `86e55896`) — bin -> `dist/final/node/index.mjs`.
   ADDED `index.unit.test.ts`
  spawning the built bin with NO args (prints usage + exits non-zero BEFORE any rg/ffplay spawn — the only inert path;
  a real query plays audio).
   nano-spawn inlined by the node config's alwaysBundle.
- `module-token-count` (NODE lib+bin,
   commit ~`token-count`) — dual-entry;
   ADDED two tests (had none):
   `index.unit.test.ts`
  imports the pure `DEFAULT_MODEL` const by-name (lib smoke,
   no Anthropic API),
   `cli.unit.test.ts` spawns `cli.mjs --help`
  (optique exits before any API call).
   Counting itself is paid/network so no counting test.
- `cli-mvm` (NODE lib+bin,
   commit `dd02f9dd`) — dual-entry;
   `list.unit.test.ts` (`splitOnWhitespace`,
   public via index
  `export *`) switched to by-name (lib smoke);
   `iso9660.unit.test.ts` stays on source (`createIso` is internal).
   ADDED
  `cli.unit.test.ts` spawning `mvm --help` (only inert path;
   every subcommand mutates KVM VMs).
- `cli-terminal-exec` (NODE lib+bin,
   INVERTED split,
   commit `50dda7b5`) — LIBRARY entry is `src/launch.ts` -> `launch.mjs`
  (the `.` export),
   BIN entry is `src/index.ts` -> `index.mjs`.
   ADDED `launch.unit.test.ts` importing `launchTerminal`
  by-name and asserting `typeof === 'function'` WITHOUT calling it (executes the lib bundle,
   spawns no terminal;
   verified
  the import is side-effect-free).
   The bin is NEVER executed by any test (running it opens a terminal).
   Internal helper
  tests stay on source.
- `mcp-mvm` (NODE pure-CLI,
   MCP stdio server,
   commit `05e86703`) — bin -> dist;
   cli-mvm + mcp-stdio inlined.
   ADDED
  `index.unit.test.ts` spawning the bin with `stdin: 'ignore'` (EOF):
   `serve` reads zero JSON-RPC lines and exits 0,
  registering all 8 tools but firing no tool call (no VM).
   EOF is the only inert invocation.
- `dev-script-vm-builder` (NODE pure-CLI,
   custom entry,
   commit `ffe93377`) — no `src/index.ts`;
   entry overridden to the
  bin `src/build-and-import.ts`.
   NO smoke test:
   it parses no args and runs `sudo podman build` immediately at top level,
  and importing it triggers the VM pipeline.
   Verification is build + dts emit.
   `import.ts`/`sign-and-push.ts` stay source-run.
- `dev-script-task-util` (NODE multi-bin,
   SIX entries,
   HIGHEST RISK,
   commit `8d719386`) — six bins
  (command/append/depends/oxlint-wrapper/pnpm-filter/tsgo-filter).
   BOOTSTRAP ORDER MATTERS:
   task-oxlint/task-tsgo are
  invoked by the mise lint templates repo-wide via `.bin` shims,
   so BUILD FIRST (bin still at src),
   verify the dist bins
  work standalone,
   THEN flip bin + `pnpm install` (shims now point at existing dist).
   Verified repo-wide:
   lint:
  types +
  lint:
  oxlint pass through the dist bins on task-util AND module-const.
   8 internal tests stay on source.
   RELOCATED three
  flaky-under-parallel-load tsgo-filter CLI integration tests to `tsgo-filter.expensive.unit.test.ts` (pre-existing
  flakiness — leftover dirs from a 2026-05-23 run;
   `par-each` starves the tsgo subprocess -> empty stdout).
   Fast file is
  deterministic 3/3.
- `module-fs-path` (NEUTRAL cross-runtime,
   commit `3650453d`) — neutral dist;
   TLA dynamic `node:path`/`node:fs/promises`
  imports survive as runtime externals;
   happy-opfs external.
   `path-ops.unit.test.ts` switched its `./index.ts` import to
  by-name (the `.` dist) as the lib smoke;
   find-* tests stay on source.
   DOCUMENTED in README:
   rolldown constant-folds the
  source's obfuscated `` `node${':path'}` `` specifier back to a literal `import('node:path')`,
   so the built `.` bundle is
  node/bun-only;
   browser cross-runtime support is via `/ts` source (nothing in-repo loads the `.` dist in a browser).
- `pi-shared-model-selection` (NODE single-index normalize,
   commit `2715d0bd`) — collapsed the per-feature tsdown entries
  (core/scope/cost/budget/pi-coding-agent) to a single `src/index.ts`;
   `package.json`/export-map were already single-index
  correct from B2.
   `pi-coding-agent.unit.test.ts` switched its single own-source import (`estimateAdvisorInputTokens`) to
  by-name as the collapsed-bundle smoke.
   Build cleaned 16 stale per-feature files;
   bundle imports with all 37 exports.
- `cli-vmsync` (NODE pure-CLI,
   deferred-B2 finish,
   commit `519c2f66`) — `src/index.ts` IS the bin (shebang),
   so it took
  the cli-git shape:
   DROPPED all nine orphaned feature exports (no consumer;
   `./ts`->`index.ts` would expose the bin as a
  lib),
   bin flips to dist,
   added missing build/buildAndTest tasks.
   ADDED `index.unit.test.ts` spawning `vmsync --help`
  (only inert path).
   Feature-module unit tests + lifecycle.
  expensive e2e stay on source.
- `build-tool-css` (NODE lib+bin,
   deferred-B2 finish,
   commit `3f4dd3f9`) — dual-entry (`index.ts` + `cli.ts`) so build-css
  bin emits `cli.mjs`;
   FIXED the broken `.`/main/module/typings pointers (they referenced `.js`/`.d.ts` but the build
  emits `.mjs`/`.d.mts` — a dangling `.` export nothing consumed);
   consolidated `./ts/fs-registry`+`./ts/process-shim`
  into `./ts/*`;
   `build.unit.test.ts` switched to by-name (`.` lib smoke) + ADDED `cli.unit.test.ts` (`--help`).
   ALSO
  fixed a pre-existing race in build.
  unit.
  `test.ts` (left `M` at session start):
   integration fixtures shared one output
  path and `cleanup()` rm'd it,
   so module-test's CONCURRENT it-blocks deleted the file mid-read (ENOENT).
   Each test now
  owns a distinct `dist/test-output-<fixture>-<test>.css` path;
   deterministic 4/4.
   (module-test runs it-blocks within a
  describe concurrently — a key gotcha for any test sharing mutable disk/registry state.
  )
- `claude-code-plugin-hook-type` — DECIDED EXEMPT (pure types,
   zero value exports;
   the `.d.mts` IS the source,
   no
  source-to-dist transformation;
   same rationale as the pure-data configs).
   Source-only;
   no build,
   no `.` flip.
   Decided by the
  exemption rule,
   not asked.

(Note:
 async-time/const used relative-to-dist `../dist/final/neutral/index.mjs`;
 later conversions use the cleaner
by-name form.
 Both work;
 no need to retrofit the first two.
)

### Smoke-test ladder by inertness (decision guide for any remaining/future bin)

Pick the lightest rung the bin's safety allows;
 each executes the built artifact:
1. By-name lib import + `typeof` check,
    never calling the side-effecting export (cli-terminal-exec) — when the only
   public symbol has side effects but its module is side-effect-free at import.
2. `--help`/`--version` spawn (image-diff,
    token-count,
    cli-mvm) — when the parser short-circuits before any handler.
3. No-args spawn asserting usage + non-zero exit (cli-rgffplay) — when no-args errors BEFORE the dangerous action.
4. EOF-stdin spawn asserting clean exit (mcp-mvm) — when a server loop terminates on closed stdin without acting.
5. NO execution,
    build + dts only (dev-script-vm-builder) — when every entrypoint path has an unavoidable side effect
   (sudo build) and the module can't be imported inertly.
    Document WHY in the commit + buildAndTest description.
Relocate any flaky/slow/network/VM test that the new `test:unit` glob activates to `<name>.expensive.unit.test.ts`.

### Remaining — bundle sets determined (deps + `node:` built-in scan)

NEUTRAL,
 no bin:
 ALL DONE.
 `claude-code-plugin-hook-type` is DECIDED EXEMPT:
 its `src/` is pure type declarations
(`common.ts` + event/tool-input types;
 verified zero `export const|function|class|let|var|default` value exports).
 A
build would emit only a `.d.mts` (or an empty `.mjs`) with no runtime code -- there is no source-to-dist
TRANSFORMATION,
 exactly the pure-data config exemption rationale.
 It keeps `.` + `./ts` + `./ts/*` at source;
 no
build,
 no `.` flip.
 This is determined by the exemption rule (pure types = the `.d.mts` IS the source),
 not a
preference,
 so it was decided rather than asked.

VERIFY-THEN-ROUTE:
 `mcp-stdio` — DONE (node bundle;
 see Done above).
 Confirmed it uses `process` stdin/stdout.

NEUTRAL but cross-runtime/OPFS:
 `module-fs-path` (uses a computed `node:path` dynamic import + happy-opfs;
 its tests
may need a non-node/DOM env).
 Do as its own mini-pilot.

### BIN sub-batch — recipes + per-package safety (read before fanning out)

SHEBANG PRE-FLIGHT (done):
 all 9 bin entries ALREADY carry `#!/usr/bin/env bun` on line 1 (checked via `head -1`).
tsdown PRESERVES a shebang but does NOT add one;
 a missing shebang builds green and fails silently only when the
installed bin runs (`/bin/sh` fallback).
 If a future bin entry lacks it,
 add it (in-scope:
 the bin is non-functional
without it).

Two bin shapes:

PURE-CLI (bin only,
 NO library `.` export — exemplar `cli-git`):
 ALL DONE — `cli-fy`,
 `cli-rgffplay`,
 `mcp-mvm`,
`dev-script-task-util` (multi-bin),
 `dev-script-vm-builder` (custom entry).
 Recipe (PROVEN on cli-fy,
 commit `f747a476`):
1. `tsdown.node.config.ts` = `export { default, } from '@monochromatic-dev/config-tsdown/.node.ts';`
2. `mise.toml`:
    keep the package's existing `run`/lint tasks;
    ADD build + `build:js` + `build:js:node` + the three
   `watch:*` twins + `test:unit` (if missing) + `buildAndTest` (`mise run build; mise run test:unit`).
3. `package.json`:
    `bin` target `src/<entry>.ts` -> `dist/final/node/<entry>.mjs`;
    add `@monochromatic-dev/config-tsdown`
   devDep.
    Keep `main` at src (matches cli-git);
    NO `exports`,
    NO `files` for a private pure-CLI (cli-git has neither).
4. NO by-name test routing:
    a pure CLI has no `.` dist to import.
    The bin's verification is a SMOKE-RUN of the built
   bin (the by-name lever applied to an executable).
    If an integration test already spawns the CLI (cli-fy's
   `cli.unit.test.ts`),
    REPOINT its spawn from `src/index.ts` to `dist/final/node/index.mjs` -> it becomes the
   automated smoke-run inside buildAndTest.
    Unit tests of internal modules stay on source.
5. CWD GOTCHA (bit cli-fy):
    a spawn-the-CLI integration test that used repo-root-relative paths with
   `cwd: process.cwd()` only worked from repo root;
    run as a PACKAGE task,
    cwd is the package dir and the paths
   misresolve (bun "Module not found").
    FIX:
    set the spawn `cwd` to the monorepo root via
   `import { findMiseMonorepoRootCached } from '@monochromatic-dev/module-fs-path/ts'` (async,
    no args,
    cached;
   `const REPO_ROOT = await findMiseMonorepoRootCached();` at module top -- TLA is fine).
    Do NOT hand-roll
   `'../../../..'` (brittle;
    the user corrected this).
    The bug was latent because the package had no test task before.
6. SMOKE INVOCATION MUST BE PROVEN INERT (read the parser first):
    cli-fy's scenarios (node:
   path ops,
    --help,
    error
   cases) are inert.
    For the remaining pure-CLIs:
    `mcp-mvm` + `dev-script-vm-builder` spin VMs -> ONLY `--help`/
   `--version` after confirming help is inert;
    `cli-terminal-exec` (lib+bin below) must NOT be run at all (`AGENTS.md`:
   probing terminal-exec opens a terminal) -- verify it by build + shebang + parser-read only.

LIB+BIN (BOTH `.`/`./ts` exports AND a bin from a SEPARATE entry -- DUAL-ENTRY):
 ALL DONE — `module-image-diff` (commit
`943e0f1a`),
 `module-token-count`,
 `cli-mvm`,
 `cli-terminal-exec` (INVERTED:
 lib `launch.ts`,
 bin `index.ts`;
 bin NEVER
run).
 Recipe (PROVEN on module-image-diff;
 in-repo precedent
`package/dev-script/deps-cube/tsdown.node.config.ts` is the same dual-entry shape):
1. Confirm lib AND bin share a platform (else two configs -- the hardest shape;
    avoid for the first pilot).
    Check with
   the node-builtins grep.
    `module-image-diff` is node+node (lib uses `node:fs/promises`+`node:path` in encoding.
   *;
   bin uses `@optique/run`),
    `--help` inert (optique prints help before any network/handler).
2. DUAL-ENTRY config.
    The base default export must be re-wrapped with `defineConfig`,
    and under `isolatedDeclarations`
   that needs an explicit `UserConfig` type on the const (a bare `export default defineConfig(...)` fails
   `TS9037: Default exports can't be inferred with --isolatedDeclarations`).
    PROVEN form:
   ```ts
   import base from '@monochromatic-dev/config-tsdown/.node.ts';
   import { defineConfig, type UserConfig, } from 'tsdown';
   const config: UserConfig = defineConfig({ ...base, entry: ['./src/index.ts', './src/cli.ts',], },);
   export default config;
   ```
   Emits `dist/final/node/index.mjs` (lib) + `dist/final/node/cli.mjs` (bin),
    each with `.d.mts`.
    tsdown chmod +x's
   `cli.mjs` (shebang detected,
    logged "Granting execute permission");
    `index.mjs` gets NO shebang.
3. `package.json`:
    `.` -> `{ types: index.d.mts, default: index.mjs }`;
    `module` -> `index.mjs`;
    `bin` -> `cli.mjs` (NOT
   `index.mjs`);
    keep `./ts`+`./ts/*`;
    `files` += `dist/final`;
    ensure `@monochromatic-dev/config-tsdown` devDep.
4. Lib tests:
    route public ones through `.` by-name (the proven lever;
    tool for single-import files,
    hand-merge for
   2+).
    `module-image-diff` used `similarity` (both symbols public) as the lib smoke test;
    internals stay on source.
5. BIN MUST BE EXECUTED,
    not just emitted (advisor catch).
    The lib by-name test loads `index.mjs`,
    never `cli.mjs`;
   and `dts` cannot verify `cli.mjs` because the bin entry has NO exports (its `.d.mts` is an empty `export {}`).
    So
   `cli.mjs` is an UNVERIFIED artifact until run.
    ADD a committed `cli.unit.test.ts` that spawns
   `bun dist/final/node/cli.mjs --help`,
    asserting exit 0 + observed help text (model on cli-fy:
    `findMiseMonorepoRootCached`
   for `cwd`,
    `nano-spawn` + `module-fs-path` as devDeps + `pnpm install`).
    Read the actual `--help` output FIRST,
    then
   assert against it (test-assumptions-before-encoding).
    `--help` is inert -> safe in the default gate.
6. NETWORK / VM / SLOW TESTS:
    rename to `<name>.expensive.unit.test.ts` (precedent `cli-vmsync/lifecycle.expensive.unit.test.ts`;
   the `test:unit` template excludes `str contains ".expensive."` unless `--all`).
    Adding `test:unit` ACTIVATES the
   `*.unit.test.ts` glob,
    so an unrenamed network test would fire paid APIs on every `buildAndTest` AND the final
   repo-wide `mise run test`.
    Leave the renamed file on SOURCE imports (no by-name switch -> zero paid calls during the
   conversion;
    the lib smoke test already covers the bundle).
    `module-image-diff`'s `client` test was renamed this way.

GENERAL:
 `cli-mvm`'s bin is `src/cli.ts`;
 `dev-script-inference-canary`'s would be `src/canary.ts` (already split in
Phase A,
 but inference-canary is a dev tool -- assess separately).
 (`claude-code-plugin-source` is exempt.
)

NORMALIZE:
 `pi-shared-model-selection` -> node-only single-index:
 collapse the tsdown entry config to `src/index.ts`
only (emit `dist/final/node`),
 drop the separate scope/cost/budget/pi-coding-agent bundles.
 Its export map was already
pruned to `.`+`./ts` in B2,
 so only the build config + the `.` dist shape need finishing.

DEFERRED B2:
 BOTH DONE (see Done list).
 `cli-vmsync` took the cli-git pure-CLI shape (dropped feature exports,
 NOT
`./ts`+`index.ts` which would expose the bin);
 `build-tool-css` consolidated to `./ts/*`,
 fixed `.js`->`.mjs`,
 built the
bin,
 and fixed a pre-existing concurrent-it-block disk race.

## Final verification

ALL per-package conversions are DONE and committed.
 Remaining = the repo-wide gate only.

Invocation note:
 there is NO root `lint:types`/`build`/`test:unit` task by that bare name;
 use the package glob form
`mise run '//package/...:lint:types'` (and `:build`,
 `:test:unit`).
 The root `//:lint`/`//:build`/`//:test` fan out too.

Gate status:
- Repo-wide `mise run '//package/...:lint:types'`:
   PASS (exit 0,
   ~21s;
   all packages incl.
   build-tool-css consumers
  messages-demo/done-postcss/ssg-test — no cross-package regression from the cli-vmsync export drop or build-tool-css
  changes).
   RE-RUN this once more at the very end as the final confirmation.
- TODO:
   repo-wide `mise run '//package/...:build'` (every package emits its dist;
   watch for any isolatedDeclarations
  gaps newly exercised by dist emit).
- TODO:
   repo-wide `mise run '//package/...:test:unit'` (default tier excludes `*.expensive.*`;
   the relocated
  tsgo-filter + build-tool-css races are out of this tier).
   Some packages use a custom `test` aggregator instead of
  `test:unit` (numeric-format,
   const,
   fs-path) — those run via `mise run '//package/...:test'` or their own task.
- TODO:
   re-run the resolution analysis.
   Criterion:
   0 NON-self cross-package imports resolve to dist.
  CRITICAL caveat (advisor):
   the by-name lib smokes added this session (path-ops->fs-path,
   list->cli-mvm,
  similarity->image-diff,
   build->build-tool-css,
   estimateAdvisorInputTokens->pi-shared-model-selection,
  DEFAULT_MODEL->token-count,
   launchTerminal->cli-terminal-exec) are SELF-imports (package X's test importing X's own
  `.` dist).
   The original analysis EXCLUDED self-imports;
   the re-run MUST exclude them too,
   or they show as false-positive
  "dist-resolving" violations.
   The real check:
   any CROSS-package (importer pkg !
  = target pkg) bare/feature import of a
  NON-exempt target whose `.` resolves to dist.
   Expected:
   zero (Phase B rewrote every cross-package import to `/ts`).

Close-out TODO (not yet done):
 decide the 3 throwaway tools at repo root — `mise.rewrite-ts-imports.ts`,
`mise.detect-merge-sites.ts`,
 `mise.switch-tests-byname.ts` (untracked) — commit or delete.
 Surface in the final
summary:
 the hook-types exemption and the fs-path `.`-dist-is-node-only README note.

## Gate RESULT (all green, 2026-05-29)

The whole repo-wide gate now passes.
 The locked task is DONE.

- Resolution analysis:
   PASS.
   Wrote `mise.analyze-resolution.ts` (root,
   untracked):
   builds a package-name -> exports
  map from every `package/**/package.json`,
   scans every `@monochromatic-dev/<pkg>[/sub]` import-context specifier,
  resolves each against the target's `exports`,
   and buckets it.
   Result over 940 specifiers:
   706 cross->src,
   138
  exempt-target,
   96 self-import (94 of which resolve to dist == the intentional by-name lib smokes,
   correctly
  excluded),
   and CROSS->DIST violations:
   0,
   UNRESOLVED:
   0,
   unknown-target:
   0.
   The only initial finding was a
  documentation typo:
   hook-types' `src/index.ts` `@example` imported `@monochromatic-dev/claude-code-hook-types`
  (a name no package owns;
   real name is `...-plugins-hook-types`).
   Fixed to the canonical `/ts` form (commit d452c277).
- Repo-wide `mise run '//package/...:build'`:
   PASS (exit 0,
   ~43s).
   Two non-import issues surfaced and were fixed:
  - `module-image-diff`:
     a byte-identical duplicate `client.unit.test.ts` was left behind when the conversion
    relocated it to `client.expensive.unit.test.ts` (paid Voyage/Gemini/OpenRouter calls).
     The duplicate would have
    fired the paid test under the default gate.
     Removed (commit c07865ab).
  - `dev-script-inference-canary-viewer`:
     PRE-EXISTING build break,
     unrelated to the import refactor.
     Its build runs
    `bun src/build.ts`,
     which spawns `git diff --no-index`;
     `git` on PATH is the cli-git wrapper,
     whose repo-root
    guard rejects the package-dir cwd that mise package-tasks use.
     Fixed by resolving the root via
    `findGitRepoRootCached` (from `@monochromatic-dev/module-fs-path/ts`) and passing it as the spawn `cwd`;
     the
    `--no-index` paths are absolute so the cwd shift is inert to the diff (commit fa4e18ed).
     User authorized this
    out-of-task-set fix.
  - `claude-code-plugin-terminal-title`:
     the gate regenerated this tracked hook artifact from unchanged source;
     the
    committed copy was stale (older toolchain downleveled `using` via a `_usingCtx()` polyfill,
     current tsdown emits
    native `using`).
     Behaviorally identical;
     refreshed so a clean checkout matches a fresh build (commit 6f5ab59e).
    User authorized the scope expansion.
- Repo-wide `mise run '//package/...:test:unit'`:
   PASS (exit 0,
   ~27s;
   default tier,
   excludes `*.expensive.*`).
- Repo-wide `mise run '//package/...:lint:types'`:
   PASS (exit 0,
   ~16s) as final confirmation.
   oxlint on the one
  edited package (inference-canary-viewer) also clean (0 warnings,
   0 errors).
