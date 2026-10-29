# Remove tsdown in favor of raw rolldown

## Status

Completed 2026-07-15.
Decided after a grilled interview and a backend bench,
piloted and swept on the `tsdown-removal` branch,
merged to `main` (merge commit `3a5fe22e6`).
tsdown is absent from the catalog,
lockfile,
mise files,
and every active package;
`package/config/tsdown` is deleted.
Outstanding follow-ups live under "Open items".

## Driver

Layer and dependency reduction:
tsdown is a roughly 5,000-line config translator and plugin orchestrator
whose value to this repository compresses into small repo-owned glue.
Supporting evidence from the user:
tsdown carries hidden timers and lifecycle management
that have caused unaccountable delays.
Not the driver:
build performance or churn fatigue alone;
those were considered and ranked below layer reduction.

## Decision record

- End state:
  raw rolldown drives every build flavor through a new repo-owned
  `@monochromatic-dev/config-rolldown` package
  (name follows the `config-<tool>` sibling convention).
  Config files rename from `tsdown.<flavor>.config.ts` to `rolldown.<flavor>.config.ts`;
  mise task names stay,
  task bodies swap to the rolldown CLI.
- Declarations:
  keep `rolldown-plugin-dts`,
  wired explicitly with `generator: 'oxc'`.
  The 2026-07-15 bench (see
  [the troubleshooting doc](../troubleshooting/rolldown-plugin-dts-typescript-7-generator.md))
  showed `tsgo` cannot build any package inlining workspace sources
  and `oxc` costs a quarter of `tsgo`'s declaration time where both pass,
  with type-surface-equivalent output that type-checks clean under TypeScript 7 strict.
  TypeScript 6 and the `tsc` backend are deprecated for this repository.
- Scope:
  active `package/*/*` plus root infra and docs.
  `package-paused/` (eight stale configs) and `package-deprecated/` (one) stay untouched;
  they are outside the workspace glob,
  so catalog removal cannot break their installs.
  Paused packages migrate at resume time.
- `@monochromatic-dev/config-tsdown` is deleted outright:
  it is not on npm (checked 2026-07-15,
   404),
  so the deprecation howto's outside-consumer preconditions do not apply.
- No forbidden-strings guard against tsdown reintroduction:
  catalog absence plus the superseding decision doc make reintroduction a loud,
   deliberate act.
- Rolldown version:
  the pilot runs on catalog-resolved 1.1.5.
  The move to 1.2.0 lands separately once it clears the `minimumReleaseAge` supply-chain gate
  (published 2026-07-15T11:08Z,
   gate is roughly one day);
  execution does not wait for it.

## Behavior inventory config-rolldown must replicate

Read from `package/config/tsdown/src/` on 2026-07-15:

- Workspace-dep inlining:
  bundle `@monochromatic-dev/**` always;
  node flavor also inlines `find-up` and `nano-spawn`;
  pi runtime peers (`@earendil-works/pi-coding-agent`,
  `typebox`,
  `@earendil-works/pi-ai`)
  stay external.
  Raw rolldown idiom:
  externals as a regex array built from `package.json`
  `dependencies` plus `peerDependencies`,
  omitting force-bundled names
  (function-form `external` carries documented per-module overhead).
- Browserslist-derived `transform.target` via the existing `browserslistTargets` helper,
  which already emits rolldown-compatible engine strings
  (rolldown rejects raw browserslist queries).
- Minify `{ compress: true, mangle: false, codegen: true }` under `output`.
- `.mjs` output via `entryFileNames: '[name].mjs'`
  (rolldown has no `fixedExtension`);
  `rolldown-plugin-dts` derives `.d.mts` from that template.
- Per-entry self-contained builds for committed and published bundles
  (array configs replace tsdown's `perEntryNodeConfig`;
  no hash-named shared chunks).
- Out dirs:
  `dist/final/node`,
  `dist/final/neutral`,
  `dist/client` as today.
- Clean:
  `output.cleanDir` is not watch-safe and per-config;
  shared-outdir per-entry builds get a mise pre-clean step
  (`run` array form) instead of `cleanDir`.
- Client flavor:
  `dts: false`,
  bundle everything,
  `canvg` stub alias,
  plain `.js` extensions in `dist/client`.
- `with { type: 'text' }` asset imports:
  keep the repo-owned `@monochromatic-dev/rolldown-plugin-import-attributes`
  (rolldown parses but ignores attributes for loader selection).
- Electron preload:
  `format: 'cjs'` is first-class in rolldown;
  translate `tsdown.preload.config.ts` directly.
- Shebang preservation for bin entries is native to rolldown entry chunks
  (verified live 2026-07-15).

## Execution plan

1. Pilot in a fresh worktree off `main`:
   create `package/config/rolldown`,
   migrate six representative packages,
   verify each at the user boundary (VUB):
   - `oxlint-plugin/tsdoc`:
      node flavor,
      published,
      dist types consumed by `oxlint.config.ts`.
   - `claude-code-plugin/correction-reminder`:
      committed `bundle/node` output,
      exercised through Claude Code.
   - `git-policies/cli`:
      bin with shebang,
      run a real command.
   - `toml-edit/core`:
      neutral flavor,
      inlined workspace deps,
      tests green.
   - `wc/app`:
      client flavor,
      page loaded via `agent-browser`.
   - `file-manager/variant-electron`:
      preload CJS,
      exercised through the app.
2. Sweep per flavor on `main` after pilot passes,
   each flavor one atomic commit including its mise task swap:
   node (70 configs),
    browser (25),
    client (7),
   outliers (`tools`,
   `main`,
   `preload`;
    five configs).
3. Final commits:
   remove `tsdown` from the pnpm catalog,
   delete `package/config/tsdown`,
   update docs.
4. Docs updates:
   supersede "Bundler:
    tsdown > raw rolldown" in `doc/philosophy/tool-choices.md`
   (rolldown 1.0-stable revisit trigger fired 2026-05-07;
   note tsdown moved into the rolldown org as the official library layer,
   and that this repo removes it anyway for layer reduction
   plus the hidden-timer lifecycle delays);
   retain tsdown troubleshooting docs
   (root causes live on in `rolldown-plugin-dts`);
   add a resume-time migration note for paused packages.

## Pilot results (2026-07-15, branch `tsdown-removal`)

All six representative cells pass on raw rolldown 1.1.5 with
`rolldown-plugin-dts` 0.27.9 (`generator: 'oxc'`):

- `toml-edit/core` (neutral):
   built,
   dist smoke-verified (parse plus
  byte-exact roundtrip),
   unit tests green.
- `oxlint-plugin/tsdoc` (node):
   plugin dist shape valid (22 rules),
  unit tests green,
   repo oxlint chain clean end to end.
- `claude-code-plugin/correction-reminder` (committed bundle):
  output byte- and mode-identical to the tsdown artifact,
  hook exercised directly with valid JSON response.
- `git-policies/cli` (bin):
   unminified single chunk,
   shebang and exec
  bit preserved,
   real commands exercised from the built bin.
- `wc/app` (client):
   page assembled and driven via
  `agent-browser`,
   zero console errors,
   live stats computed.
- `file-manager/variant-electron` (preload CJS):
   built through the
  swapped mise task;
   the wayland boundary test passes on the merged
  tree (four consecutive exit-0 runs with the rolldown-built
  `preload.cjs` staged).
   The sweep-time failures were load-induced:
  the harness's ten-second observed-state deadline missed while the
  repo-wide build fanout and cargo release builds saturated the
  machine,
   on the tsdown-built main checkout and the branch alike.
  The earlier "blocked by agent-session display access" reading was
  wrong:
   the session has real Wayland access,
   and the
  `Fatal Wayland communication error: Broken pipe` lines appear in
  passing runs too;
   they are electron teardown noise after the
  compositor's `quit`,
   not a failure signal.

Pilot findings folded into the design:

- Raw rolldown does not set the executable bit on shebang outputs;
  `config-rolldown` ships a `writeBundle` chmod plugin.
- `--configLoader native` is required:
   the default bundle loader's temp
  file resolves bare imports from the package dir,
  which strict pnpm isolation rejects;
  the native loader mirrors tsdown's per-file resolution chain.
- `config-rolldown` quality gates pass:
  zero oxlint findings,
   types clean,
   ported browserslist tests green.
  Unit tests for the new `package-externals` and `shebang-executable`
  helpers are still owed before the package counts as complete (PKG).

## Open items

- Identify the source of truth for `mise.toml` versus `mise.no-env.toml`
  before editing task bodies
  (`file-enforcer.config.ts` does not manage them;
  the duplication mechanism is unidentified).
- Bump rolldown catalog floor to `>=1.2.0` once the age gate clears.
- Follow-up audit (separate from this migration):
  shrink `dts` to packages whose dist types are actually consumed;
  requires auditing the 268 bare-name workspace imports.

## Rejected alternatives

- Split emit/bundle architecture (tsgo per-file emit for libraries,
   rolldown for bundles):
  adds a second build mechanism and forces a packaging redesign;
  not fewer layers than rolldown alone.
  The bench later confirmed tsgo also cannot emit across inlined workspace sources.
- No-build for private libraries (exports pointing at src):
  breaks non-TS consumers
  (oxlint plugin host,
   bare-name imports,
   runtime text imports),
  and Node refuses type stripping under `node_modules` for npm consumers.
- A different third-party wrapper (unbuild,
   tsup,
   vite lib mode):
  swaps a layer instead of removing one.
- Owning declaration bundling via `oxc-transform` directly:
  reimplements the hard part of `rolldown-plugin-dts`;
  opposite of layer reduction.
