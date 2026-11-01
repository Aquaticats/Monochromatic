# Handover: config-oxlint stale plugin bundle (issue #570)

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/task-util` is now `package/monorepo-task/core-exec`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `@monochromatic-dev/module-async-time` is now `@monochromatic-dev/async-time-core`;
> `package/module/or-throw` is now `package/or-throw/core`;
> `package/module/logger` is now `package/logger/core`;
> `@monochromatic-dev/module-logger` is now `@monochromatic-dev/logger-core`.

## Status (2026-09-25, grilling in progress, no code changed)

Issue:
 [#570](https://github.com/Aquaticat/Monochromatic/issues/570).
The user asked to resolve it,
then asked for a grilling session before any code change.

Next action:
 finish fact gathering,
 then ask the reframed design-tree questions under "Open questions".
No code edits until the user confirms shared understanding.

## Requirements from the user

- Grill the design before editing code.
- Do not plainly adopt the issue's framing or its listed remediations;
  look for fixes that dissolve the failure instead of patching one link.
- The first round of questions (which of the issue's remediations to do,
  and where a rolldown build guard lives) was rejected as unanswerable as posed.

## Rejected framing

Asking "which of the issue's remediations (build guard, wrapper guidance, post-build import check)"
and "shared versus local build guard" presupposes the issue's framing
that the bundler step is the thing to fix.
The user rejected that round.

## Evidence gathered

### Rolldown behavior (installed 1.2.9, catalog `>=1.2.5` in `pnpm-workspace.yaml`)

- An unresolved bare specifier emits `UNRESOLVED_IMPORT` as a warning
  and is kept as an external import;
  the build exits 0
  (upstream `crates/rolldown/src/module_loader/resolve_utils.rs` lines 118 to 143 at tag v1.2.9).
- Path-like unresolved specifiers are hard errors already.
- No strict mode;
  `checks.unresolvedImport` only silences;
  no `logLevel` value makes it an error.
- The input option `onLog` can escalate with `defaultHandler('error', log)`;
  a subagent probe showed `rolldown -c` then exits 1,
  with a positive control (non-matching filter exits 0 and emits the bare import).
- Log fields:
  `log.code`,
  `log.id` (importer path),
  `log.exporter` (unresolved specifier).
  `log.message` carries ANSI codes,
  so filter on fields.

### Repo structure

- `dist/` is gitignored (`.gitignore` line 53),
  so `package/config/oxlint/dist` is a local build artifact,
  not committed.
  The issue's "committed-to-disk" wording means "present on disk".
- The root `oxlint.config.ts` imports the built default export
  `@monochromatic-dev/config-oxlint`
  (`package/config/oxlint/package.json` `exports["."]` points at `dist/final/node/index.mjs`).
- `package/config/oxlint/src/index.ts` is an existing source entry (`./ts` export)
  that resolves plugin `/ts` source via `import.meta.resolve()`
  so in-repo linting tracks live source with no rebuild.
  The root config does not use it.
- `package/config/oxlint/src/index.node.ts` says the prebuilt sidecars
  avoid plugin resolution at lint time,
  an optimization tracked in issue #238
  (not yet read;
   `gh` hit a TLS timeout).
- All three shared rolldown flavors
  (`package/config/rolldown/src/index.ts`,
   `index.client.ts`,
   `index.node.ts`)
  use `packageExternals`,
  which bundles undeclared bare imports by omission,
  so every flavor shares the silent-external weak point.
- `config-oxlint` has no `version` and is not published yet;
  publishing is tracked by #523
  (`doc/handover/pnpr-publish-remaining-packages.md`).
- `mise.toml` has `prepare:pnpm:install` (`pnpm install`);
  build tasks do not depend on it.

### Existing freshness check backfires (verified by reading `mise.toml` lines 374 to 421)

- `lint:oxlint` and `format:oxlint` templates call `ensureOxlintConfig()`,
  which rebuilds `//package/config/oxlint:build:js:node`
  when `shouldBuildOxlintConfig()` finds a source newer than the oldest sidecar.
- Sources are `src`, `package.json`, `tsconfig.json`, `rolldown.node.config.ts`
  of `config-oxlint` and each `package/oxlint-plugin/*`,
  plus `pnpm-lock.yaml` and `pnpm-workspace.yaml`.
- In the #570 sequence:
  a merge or `pnpm install --lockfile-only` touches `pnpm-lock.yaml`;
  the next lint rebuilds before install,
  producing the broken bundle now newer than every source;
  a later `pnpm install` touches none of those sources,
  so the check never rebuilds again.
  This is why "install did not help" in the issue.
- Second hole:
  transitively inlined workspace packages
  (for example `package/module/logger` source)
  are not in the source list,
  so editing them never rebuilds the sidecars either.

### Install freshness (subagent report, not re-verified except where noted)

- No git hook or cli-git post-action runs `pnpm install`;
  hk is retired (`doc/decision/cli-git-policies-platform.md` line 870);
  `.git/hooks` holds only Git LFS shims.
- pnpm `verifyDepsBeforeRun` only fires on `pnpm run` and `pnpm exec`,
  which repo tasks never use (CM3).
- `node_modules/.pnpm/lock.yaml` is pnpm's record of the last installed lockfile;
  comparing it with `pnpm-lock.yaml` detects "lockfile changed, not installed".
  Every install rewrites its mtime.
- No-op `pnpm install --frozen-lockfile --offline` measured 0.39 s then 0.21 s
  (two runs, no spread band),
  and relinks six packages every run (cause not investigated).
- mise native `sources`/`outputs` exist; no repo task uses them.
  `task-util` `depends` helper is used only by `prepare:playwright`.

### Consumers, reproduction, wrapper (subagent report; starred items re-verified)

- No rolldown config in the repo sets `onLog`, `onwarn`, or `checks`.
  All three shared factories build from one base object
  (`package/config/rolldown/src/index.node.ts` `baseOptions`,
   `index.ts`,
   `index.client.ts`),
  so one `onLog` there reaches every consumer.
- With dependencies installed,
  `//package/config/oxlint:build` emits no `UNRESOLVED_IMPORT`.
- Issue #570 reproduced in a throwaway worktree:
  after removing the plugin's `module-logger` link,
  rolldown printed
  `[UNRESOLVED_IMPORT] Could not resolve '@monochromatic-dev/module-logger/ts' ... treating it as an external dependency`,
  exited 0,
  and oxlint then failed to load the config.
- A fresh-worktree full `mise run build` printed `UNRESOLVED_IMPORT` only from
  raw (non-`config-rolldown`) electron main configs
  and from `package/kwin/key-helper/src/nvim.ts` line 19 (starred),
  which imports bare `@monochromatic-dev/module-async-time`
  instead of the `/ts` subpath (ST3),
  so it depends on sibling dist build order.
  The full build also exited 1 partway with no error line (not investigated).
- Starred:
  `~/temp/agent/node_modules/@monochromatic-dev/module-logger` is a stray symlink (dated Sep 23)
  to the main checkout;
  any worktree under `~/temp/agent` resolves `module-logger` through it,
  masking this bug in reproductions.
- Starred:
  `package/oxlint-plugin/no-restricted-syntax/package.json` lists `module-logger`
  as a devDependency (the issue calls it "declared");
  workspace code is bundled at build time,
  so dev classification is consistent with the bundling policy.
- Wrapper source:
  `package/dev-script/task-util/src/oxlint-wrapper.ts`;
  `finalizeResult` keeps oxlint's exit code.
  oxlint 1.85.0 prints the config-load failure block on stdout, not stderr.
  The wrapper has no test;
  `oxlint-augment.ts` holds testable pure helpers.

### Lint speed: built sidecars versus `./ts` source entry (subagent report)

- Issue #238 (closed, commit `05cbf8fd1`) recorded,
  per oxlint process on a clean fixture,
  prebuilt 606.2±35.5 ms versus source 910.0±62.0 ms;
  savings multiply in the `--fix` loop
  (a three-pass fix chain spawns six oxlint processes).
- Re-measured through `mise run //package/<pkg>:lint:oxlint`,
  hyperfine one warmup plus five runs,
  on a host at load average 28.59 over 16 cores (other sessions),
  so the band is wide:
  - `package/module/or-throw` (58 files):
    built median 1.736 s (spread 0.242 s),
    `./ts` median 1.785 s (spread 0.100 s).
  - `package/git-policy/cli` (255 files):
    built median 3.837 s (spread 0.216 s),
    `./ts` median 3.970 s (spread 0.107 s).
  - Differences sit inside or near the run-to-run band;
    the `--fix` loop was not re-measured.
- Positive control confirmed the `./ts` config loads all five plugins
  (`no-restricted-syntax(no-regex)` fired on a probe file).
- Whole-repo lint took 369.6 s in one fresh-worktree trial
  with 385 TS2307 errors from unbuilt packages;
  not representative.
- Interleaved re-measurement (three arms rotated: built-A, built-B as noise band, `./ts`;
  3 warmup plus 15 runs each; load average 6 to 13 on 16 cores;
  fixture adapted from #238, confirmed three fix passes plus three oracle passes):
  - `format:oxlint` fix loop on `package/module/or-throw`:
    built medians 6.312 s and 6.309 s,
    `./ts` median 7.436 s (minimum 7.187 s),
    so +1.12 s (+18 %), outside the band.
  - Plain `lint:oxlint`:
    built medians 1.595 s and 1.634 s,
    `./ts` median 1.799 s,
    so about +0.2 s (+12 %), outside the band.
  - About 0.19 s per oxlint process;
    `strace` confirmed `./ts` opened plugin source and no dist files.

### oxlint and rolldown share blame (subagent report; details in troubleshooting docs)

- `doc/troubleshooting/oxlint-config-load-failure-exit-code.md`:
  oxlint 1.85.0 exits 1 both for "config or JS plugin failed to load" and for "lint found errors"
  (`apps/oxlint/src/result.rs` lines 27 to 51 collapse distinct variants);
  the failure prints on stdout before any formatter,
  so `--format json` does not change it.
  Tested consumer-side discriminators:
  unparsable `--format json` output,
  or the absence of the `Found N warnings and M errors.` summary line.
  ESLint documents exit 2 for this case.
  A prototype upstream patch (exit 2) is drafted, not filed.
- `doc/troubleshooting/rolldown-unresolved-import-external.md`:
  warn-and-externalize is deliberate Rollup-compatible behavior;
  the rolldown docs line claiming otherwise is wrong.
  A hard-error option was merged and reverted upstream (#9388, #9438; request #9362 open).
- Decisive for design:
  rolldown delivers warnings to `onLog` only after `bundle_write` has written every file,
  so an `onLog` escalation exits 1 but still leaves the broken sidecar on disk,
  newer than every input,
  which `shouldBuildOxlintConfig` then treats as current.
  The issue's first remediation, as worded, would not prevent the persisted artifact.
  A `resolveId` plugin calling `this.resolve(..., { skipSelf: true })`
  and `this.error` on null fails before writing (verified by the subagent).
  A prototype upstream fix (deliver warnings before writing) is drafted, not filed.

### mise share of blame (subagent report; details in `doc/troubleshooting/mise-dependency-freshness.md`)

- mise 2026.9.12 has `mise deps` (alias `prepare`),
  configured as `[deps.pnpm] auto = true`;
  experimental, but the repo already sets `experimental = true` (`mise.no-env.toml` line 225).
  The repo has no `[deps]` table.
- It hashes root `pnpm-lock.yaml` and `package.json` (blake3),
  records state only after a successful install, per worktree,
  and runs before every `mise run` and `mise x`, including child runs;
  a failed install aborts the run.
  Throwaway experiment:
  after `pnpm install --lockfile-only` it reported stale and installed first
  (positive control);
  the next run skipped install (negative control).
- Limits:
  workspace-member `package.json` files need a `sources` override;
  it never inspects `node_modules`,
  so the issue's reproduction (deleting one link) stays invisible;
  a manual `pnpm install` is not recorded,
  so the next `mise run` installs again;
  no cross-process lock.
  Per-run overhead unresolved within noise.
- Native task `sources`/`outputs` share the `ensureOxlintConfig` flaw
  unless an install marker (`node_modules/.modules.yaml`) is a source.
  Default mtime mode also skips a task whose failed first run wrote outputs;
  hash mode reruns correctly.
  Docs say oldest output, code uses newest.
- Repo policy forbids `depends` (`mise.toml` line 627 comment:
  "never use depends or post depends, use run only").
- Enter, cd, and `watch_files` hooks fire only under `mise activate`.
- The "fanout exited 1 with no error line" report did not reproduce:
  a fresh-worktree build ended with `package fanout failed:` naming four packages,
  each with its own earlier error (missing Android NDK, unresolved `canvg`,
  `libghostty` build order, a `git log` failure).
- Drafted, not posted:
  a comment for upstream discussion #8733 with a prototype fix and e2e test
  (`doc/troubleshooting/mise-dependency-freshness.patch`).

### pnpm share of blame (subagent report; details in `doc/troubleshooting/pnpm-stale-node-modules-detection.md`)

- `--lockfile-only` writes only `pnpm-lock.yaml` (documented);
  it skips linking and the installed-state files,
  and nothing warns afterwards (`pnpm list` exits 0 without the new dependency).
- No standalone status command exists in pnpm 12.5.1.
  The `verifyDepsBeforeRun` check compares settings, catalogs, project list,
  missing `node_modules`, member manifest mtimes,
  and wanted versus installed lockfile.
  Usable as a probe today:
  `pnpm --config.verify-deps-before-run=error exec true`,
  about 85 ms up to date or stale (30 runs, quiet host),
  versus 96.9 ms for a no-op `pnpm install --offline`
  and 215 ms for `--frozen-lockfile --offline` (frozen disables the fast path).
- The per-run "added 6" in the main checkout is a pnpm bug:
  49 dangling links to skipped optional platform packages inside
  typescript, rolldown, oxlint, oxlint-tsgolint, satteri, yuku-parser
  never satisfy the reinstall check;
  visible here because the repo sets `modulesCacheMaxAge: 0`.
  Local repair (not run):
  delete the dangling links under `node_modules/.pnpm`.
  Prototype upstream fix and draft issue exist, not filed.
- A repair install prints `Already up to date` even while it creates a sub-project workspace link.

## Failure chain with owners

Separate links;
a fix at an earlier link can make later ones moot,
but each link also fails for causes other than #570.

1. Install drift:
   `node_modules` falls behind manifests.
   pnpm leaves `--lockfile-only` drift silent and has no status command outside `pnpm run`/`pnpm exec`;
   mise ships `mise deps` but the repo does not enable it;
   nothing in the repo gates tasks on install state.
2. Silent bundling:
   rolldown warns and externalizes (deliberate, Rollup-compatible),
   and an `onLog` escalation fires only after output is written;
   only a `resolveId` guard fails before writing.
3. Stale persisted artifact:
   in-repo lint consumes gitignored `dist` sidecars;
   the repo's hand-rolled `ensureOxlintConfig` mtime check
   ignores install state and transitive workspace sources,
   and mise native `sources`/`outputs` would repeat that flaw.
4. Ambiguous failure signal:
   oxlint exits 1 for both config-load failure and findings;
   the wrapper passes that through;
   a caller filtering output read it as clean.

## Open questions

- Round 3 asked (awaiting answers):
  policy when a task starts on drifted `node_modules`;
  which drafted upstream reports to file;
  whether to repair the main checkout's dangling optional-platform links.
- Held until install policy is settled:
  install-gate mechanism (mise deps versus pnpm verify probe);
  in-repo lint artifact (fixed freshness check versus `./ts` source entry;
  measured cost of `./ts` is about 0.19 s per oxlint process);
  build guard (link 2);
  wrapper signal (link 4).
- Unrelated incidents seen during research,
  not in #570 scope:
  fresh-worktree full build failures
  (Android NDK, `canvg`, `libghostty` order, `git log`);
  `package/kwin/key-helper/src/nvim.ts` bare import violating ST3.

## Commits

- Handover and troubleshooting docs only; no code changed.
