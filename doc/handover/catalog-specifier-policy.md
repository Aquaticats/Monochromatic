# Catalog specifier policy

> References below this note that no longer resolve in the tree.
> Each entry gives the value that identifier has now:
> `package/dependency-catalog/core-specifier-policy` is not a path this repository held;
> the specifier policy is `package/catalog-specifier/core-rust` today
> (2026-11-01 `d3a44a3f6a`).
> `package/dependency-catalog/cli-specifier-policy` is not a path this repository held;
> the specifier policy is `package/catalog-specifier/core-rust` today
> (2026-11-01 `d3a44a3f6a`).

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

## Purpose

`pnpm-workspace.yaml` sets `catalogMode: strict`,
yet manifests with literal versions install silently.
This handover records the measured cause,
the decisions reached in the grilling session,
and the next action.
No code has been changed yet;
the user has not confirmed the shared understanding.

## Measured facts

- pnpm documents `catalogMode` only for `pnpm add` and `pnpm update`.
  In a throwaway fixture on pnpm 12.16.1,
  `pnpm install` accepted `"is-even": "1.0.0"` and `"is-number": "^7.0.0"` with exit 0 and no warning.
  The positive control `pnpm add is-odd@1.0.0` (outside the catalog range `>=3.0.0`) failed with `ERR_PNPM_CATALOG_VERSION_MISMATCH`.
- The pnpm source consults the catalog mode only on add and update paths.
  No pnpm setting or subcommand enforces catalog use at install time.
- A `readPackage` hook in `.pnpmfile.mjs` receives the root and every workspace manifest on pnpm 12.16.1,
  with raw specifiers in `dependencies`,
  `devDependencies` and `peerDependencies`.
  The comment in `.pnpmfile.mjs` saying direct workspace dependencies are not read via `readPackage` is stale and must be corrected.
- The hook can import a `.ts` file (`./check.ts` was imported and called in a fixture).
- The hook runs on frozen installs,
  in CI,
  and on any install after a manifest edit.
  `readPackage` does not run on a plain no-op `pnpm install` that prints "Already up to date" (`optimisticRepeatInstall`).
  Correction after the reboot:
  the `updateConfig` hook does run there,
  and on every other verb;
  see "Research after the reboot".
- Workspace manifests and `file:` dependency manifests both reach the hook without a `dist` key,
  so a `dist` filter would police `file:` dependencies;
  the hook must use a workspace-name set built from `package/*/*`.
  (Moot under an `updateConfig` hook,
  which never sees dependency manifests.)
- `syncpack` 15.3.3 can express the policy only with workarounds and passes named catalogs.
  `eslint-plugin-pnpm` 1.9.1 leaks `workspace:*.0` and peers starting with `*`,
  defaults to a destructive autofix,
  and needs ESLint beside oxlint.
  Both were run in CachyOS containers;
  neither is adopted.
- The history scan found 260 commits that touched exact-version lines,
  covering 37 distinct dependencies.
  Today the repo has 248 tracked manifests and exactly one violation:
  `zod` at `^4.6.7` in `package/dsh-plugin/cost-dismissal/package.json`.
- The repo has no `catalogs:` section and no named-catalog references;
  208 manifests use plain `catalog:`.

## Decisions

- Policy:
  `dependencies`,
  `devDependencies` and `optionalDependencies` accept only `catalog:`,
  `catalog:<name>` or exactly `workspace:*`.
  `peerDependencies` accept only `*`,
  `catalog:` or `catalog:<name>`.
  Any other specifier fails,
  including digits,
  ranges,
  prereleases,
  `workspace:^`,
  `link:`,
  `file:`,
  git and URL forms.
- No exception list;
  a dependency that needs a version belongs in the catalog.
- Catalog entries themselves are not validated by this check.
  Re-confirmed after the review:
  manifests only,
  even though `pnpm add --workspace` promotes a workspace package
  into the catalog as `workspace:^1.0.0` (measured).
- The lint checks reference syntax only:
  it does not verify that a `catalog:` or `catalog:<name>` reference has an entry.
  pnpm rejects a missing entry for installed packages,
  but nothing checks tracked manifests outside the workspace.
  This is documented as a known limit,
  by the user's choice after the review.
- Named catalogs are allowed;
  pnpm itself rejects a reference to a missing entry.
- Mechanism:
  one shared check,
  run by a mise lint task and by the `.pnpmfile.mjs` hook.
- Package home (the hook-facing validator must stay dependency-free,
  see the "Pre-implementation probes" section):
  new `package/dependency-catalog/core-specifier-policy` (pure validator and manifest enumeration)
  and a thin `package/dependency-catalog/cli-specifier-policy` for the mise task.
  `core-pnpm` stays parsing-only.
- Lint scope:
  every tracked `package.json`,
  including the root and `package-paused/`.
  The hook covers only workspace manifests,
  identified by a workspace-name set.
  Superseded by "Research after the reboot":
  an `updateConfig` hook reads manifests from disk and enumerates the workspace
  from `config.workspacePackagePatterns`,
  so no name set and no `dist` classifier is needed;
  the root manifest is covered too.
- The check reports every violation in one run,
  naming the file,
  the section and the dependency.
- `zod` moves into the catalog as `>=4.6.7` and `cost-dismissal` switches from `^4.6.7` to `catalog:`.
  An intermediate edit of this handover wrongly said `zod` was already in the catalog;
  the `'>=4.6.6'` hit at `pnpm-workspace.yaml` line 461 is the CVE floor in the `overrides:` block,
  not a catalog entry.
  Measured:
  `pnpm-lock.yaml` records the `cost-dismissal` specifier as `'>=4.6.6'` (the override) resolving 4.6.7,
  so the override already governs and the catalog range changes no effective resolution.

## Pre-implementation probes

Run after the decisions,
on pnpm 12.16.1 in a throwaway fixture,
each with a positive control.

- Fresh install with a hook-imported file that imports a bare package fails for every install:
  `ERR_MODULE_NOT_FOUND: Cannot find package 'is-odd'`,
  exit 1,
  because the hook loads before `node_modules` exists.
  The hook-facing validator must therefore be one dependency-free file imported by relative path,
  with only `node:` builtins.
  The repo rules that normally require `@monochromatic-dev/logger-core` and `or-throw-core` need a documented exception in that file.
- A throwing hook does not break `pnpm add`:
  with `catalogMode: strict`,
  `pnpm add is-buffer@1.1.6` and `pnpm add is-buffer`
  both exit 0 and the hook receives the promoted `catalog:` form.
  The positive control (a manifest with `"is-number": "^7.0.0"`) exits 1 with `ERR_PNPM_PNPMFILE_FAIL` and the policy message.
- pnpm itself rejects a `catalog:` reference with no catalog entry:
  `ERR_PNPM_CATALOG_ENTRY_NOT_FOUND_FOR_SPEC`.
- Read from `.pnpmfile.mjs`,
  not probed:
  `applyBlocklist` rewrites `dependencies` and `optionalDependencies` in place to
  `workspace:@monochromatic-dev/pnpm-override-stub-*@*`.
  The specifier policy must therefore validate the manifest as received,
  before `applyBlocklist`,
  and a test must cover a workspace manifest that depends on a blocked name.

## New input after the decisions

The user plans to shadow `pnpm` with an own Rust and clap wrapper.
No document in `doc/` mentions it yet;
the nearest precedent is `package/mpm-shadow/cli`,
which finds the real tool on `PATH`,
runs it as a child with arguments untouched,
then acts after it.
A `pnpm` shadow could run the specifier check before pnpm starts,
which would cover the no-op install that the `readPackage` hook cannot reach.
That is an inference from the shadow design,
not a measurement.
Correction:
the `updateConfig` hook reaches the no-op install too,
so this was not a shadow-only advantage.
The user delegated the timing and reach decisions,
so these are agent recommendations,
open to veto.

## Independent review findings

A `pi` review (model `openai/gpt-6.1-sol`,
whole files pasted) and the advisor reviewed the design after the decisions.
Verdict from `pi`:
ship a source-manifest checker with CI enforcement,
defer the hook and the Rust shadow.
Verified by probe or file read:

- A name-only workspace classifier misfires on a registry package that shares a workspace package's name:
  registry `is-odd@3.0.1` matched `nameOnly=true` with `dist` present,
  while name plus no `dist` matched only the workspace package.
  The hook must require both conditions,
  and the set must include the root manifest's name.
- `pnpm-workspace.yaml` sets `savePrefix: ''` and `saveWorkspaceProtocol: false` (lines 537 and 538).
  Under `catalogMode: strict`,
  `pnpm add @fx/b --workspace` writes `"@fx/b": "catalog:"` for both `false` and `rolling`,
  and promotes the workspace package into the catalog as `workspace:^1.0.0`,
  which the manifest policy allows
  but which differs from the repo's `workspace:*` convention.
- The existing hook exports `export const hooks`;
  the fixture used `export default { hooks }`.
  The implementation must be tested with the real export shape.
- Not probed:
  whether CI invokes the root `lint` task,
  and the `pnpmfileChecksum` effect of an edited imported validator.
- Lint checks syntax only;
  a `catalog:` reference to a missing entry in a tracked manifest outside the workspace
  (for example `package-paused/`) is not caught by pnpm either.
- A per-manifest hook throws on the first violating manifest,
  so "every violation in one run" holds for lint only.

## Shadow recommendations

- Timing:
  ship the TypeScript check,
  the hook and the lint task now;
  the shadow is a separate later project.
  Reason:
  the gap is real today,
  the shadow has no document or code,
  and the policy is language-independent.
- Port by corpus,
  not by code:
  keep the policy's test vectors (specifier,
  section,
  expected verdict)
  in one JSON file that the TypeScript tests and a later Rust nextest suite both read,
  so the shadow adopts the same rules without a second interpretation.
- Reach:
  the lint task stays as the CI backstop even after the shadow exists.
  Measured:
  `mise.toml` names `pnpm` 30 times and installs funnel through the `prepare:pnpm` task chain,
  so callers do not all share one `PATH` resolution.
- The hook is retired only after the shadow is shown to be first on `PATH` for CI,
  mise tasks and agent shells.
- The shadow,
  when built,
  follows the `package/mpm-shadow/cli` pattern:
  run the real `pnpm` as a child,
  arguments untouched,
  and run the check first.

## Why the Rust wrapper option was not listed in the hook-timing question

The hook-timing question (Q10) offered only TypeScript-first variants:
lint then hook,
both together,
or lint plus a mise call.
It did not offer "build the whole policy in the Rust wrapper and drop the lint task and the hook".
That omission was a mistake of mine,
not a measured exclusion:

- Earlier questions did offer the shape:
  the shadow-timing question had "shadow is the next thing built" and
  "standalone Rust crate",
  and the reach question had "the shadow is the only enforcement point",
  ranked last.
  The user answered both with "you suggest",
  delegating them,
  and I recorded my own recommendation as if it were settled.
  I then built the hook-timing options on that recommendation instead of re-asking,
  and did not widen the options
  after the review,
  which the repo's option-widening rule requires.
- The reasons I would have given for ranking it low are real but are not measurements:
  no document or code for the shadow exists (searched `doc/` and `package/`,
  found none),
  a Rust crate needs its own README,
  nextest tests and packaging first,
  a shadow only mediates callers that resolve it first on `PATH`,
  and a pre-flight check on every invocation needs verb,
  `--dir` and repair-command design (raised by the `pi` review).
- Its real advantages were also not weighed:
  one implementation in the language the shadow is written in,
  and a check that runs before pnpm starts,
  so even a no-op install is covered
  (inference from `package/mpm-shadow/cli`,
  not measured).

The user decides this when back;
the option must be put to them with pros,
cons and a ranking.
A suggested rule edit for `AGENTS.md`,
not applied:
after a delegated decision,
treat the recommendation as open
for veto and re-offer the delegated alternative in every later option set that depends on it.

## Dependencies the design needs

Enumerated late;
the repo's grilling rule asks for this first.
Nothing here is picked yet beyond the first item.

- The hook-facing validator file:
  `node:` builtins only (measured necessity,
  because the hook loads before `node_modules`).
- Superseded:
  the workspace-name set and the YAML question.
  `updateConfig` hands the hook `config.workspacePackagePatterns` (measured:
  `["pkg/*"]` in the fixture),
  so the hook expands those globs with `node:fs` and needs neither a name set nor a YAML parser.
  What remains open is how the hook expands `*`,
  `**` and `!` patterns,
  and failing closed on a pattern it cannot expand.
- The rest of `core-specifier-policy` and `cli-specifier-policy` would follow repo rules:
  `@monochromatic-dev/logger-core`,
  `@monochromatic-dev/or-throw-core`,
  `string-dedent` and the test harness.
  Open:
  how `git ls-files` is run (an existing `package/git-executable` may own that;
  not yet read).
- Rust shadow,
  if chosen:
  `clap` plus a JSON reader for manifests;
  no crates picked,
  and the catalog-policy rule for
  Rust dependencies (`doc/decision/rust-dependency-catalog.md`) governs the picks.

## State at the user's reboot

- Decided:
  policy,
  no exceptions,
  named catalogs allowed,
  lint scope (every tracked manifest),
  package home (two new packages),
  manifests-only,
  syntax-only references,
  zod into the catalog as `>=4.6.7`.
- Deferred by the user:
  hook timing,
  and whether the whole policy moves into the Rust wrapper.
- Nothing was implemented.
  Throwaway fixtures,
  probe scripts and the `pi` prompt and output remain in `~/temp/agent`.
- Commits made by this session trigger `standards-review-shadow-*` systemd units;
  they were left to finish.

## Research after the reboot

The user said the earlier research was clearly insufficient and asked for more before any further question.
Everything here was measured in throwaway fixtures under `~/temp/agent` unless marked otherwise.

### The `updateConfig` hook reaches every verb, which corrects the no-op claim

Probe `~/temp/agent/catalog-hook-matrix.ts` (fixture `hook-matrix-GFm9KN`),
pnpm 12.16.1,
one hook-logging `.pnpmfile.mjs`:

- `updateConfig` fired on:
  fresh install,
  no-op install,
  frozen install,
  `add`,
  `update`,
  `prune`,
  `run`,
  `exec`,
  `list`,
  `why`,
  `outdated`,
  `fetch`.
- It did not fire on `config get` and `--version`.
- `readPackage` fired only on fresh,
  frozen,
  `add`,
  `update`,
  `prune` and `fetch`;
  `preResolution` on fresh,
  frozen,
  `add`,
  `update`,
  `prune`;
  `afterAllResolved` on fresh,
  `add`,
  `update`.
- `updateConfig` receives the config object with `dir` and `workspaceDir`,
  and no manifests.
  A hook can still read manifests itself,
  because it is ordinary Node code.
- So the earlier statement that no hook reaches the no-op install was wrong for `updateConfig`.
  `doc/troubleshooting/pnpm-prune-before-install.md` already documented `updateConfig` as the earliest hook;
  I had not read it before writing the handover.

### An `updateConfig` check enforces the policy on every verb

Probe `~/temp/agent/catalog-updateconfig-probe.ts` (fixture `updateconfig-probe-6VGCn8`):
an `updateConfig` hook using only
`node:fs` and `node:path` hand-parses the `packages:` key of `pnpm-workspace.yaml`,
expands the `*` globs,
reads each
`package.json` and throws on any policy violation.

- Clean workspace:
  install and no-op install exit 0.
- With `"is-number": "^7.0.0"` planted in `pkg/a`:
  no-op install,
  frozen install,
  `run`,
  `exec`,
  `list` and a `run` from the
  `pkg/a` subdirectory all exit 1.
  The message reaches the user:
  `running updateConfig hook ... Error: specifier policy: pkg/a/package.json dependencies is-number=^7.0.0`.
  It is hard-wrapped by pnpm at terminal width,
  so a grep for the full phrase on one line misses it.
- After repairing the manifest,
  the no-op install exits 0 again (the positive control moves in both directions).
- `pnpm install --ignore-pnpmfile` exits 0 with the violation present:
  the one bypass found.
  `config get` and `--version` are unaffected,
  so the repair path through pnpm itself is not blocked.
  No file in the repo uses `--ignore-pnpmfile` or `ignorePnpmfile`.
- Reach and bypass probe `~/temp/agent/catalog-updateconfig-reach.ts`,
  same fixture,
  violation planted
  (the message,
  not the exit code,
  is the discriminator;
  the control run without a violation printed no message):
  the check fired for `--dir`,
  `-C`,
  `--filter`,
  `-r`,
  `--workspace-root`,
  and `--dir <fixture>` run from an unrelated cwd.
  It was skipped for `pnpm_config_ignore_pnpmfile=true` in the environment and for `--config.ignore-pnpmfile=true`,
  as well as `--ignore-pnpmfile`;
  `npm_config_ignore_pnpmfile=true` did not skip it.
  `--pnpmfile ./missing.mjs` fails with exit 2 rather than skipping.
  A deliberate bypass therefore exists in three spellings (flag,
  `--config.` form,
  `pnpm_config_` environment variable),
  which is why the source lint stays as the backstop.
- Probe `~/temp/agent/catalog-updateconfig-add.ts` (same fixture,
  hook and manifest restored afterwards):
  - The `updateConfig` argument carries `workspacePackagePatterns` (value `["pkg/*"]` in the fixture),
    `packages`,
    `ignorePnpmfile`,
    `catalogs`,
    `catalogMode` and `workspaceDir`,
    among a long list of settings.
    The earlier probe logged only a fixed key list that omitted `workspacePackagePatterns`.
  - `updateConfig` and `readPackage` coexist in one `export const hooks` object;
    both ran.
  - With a violation planted,
    `pnpm add is-odd`,
    `pnpm update` and `pnpm remove is-number --filter @fx/a` all exit 1 with the
    policy message.
    The check runs before those verbs read anything,
    so pnpm cannot repair a violation:
    the repair is a hand edit of the
    manifest (or the bypass).
    This differs from the `readPackage` finding that a throwing hook leaves `pnpm add` working.
    The hook's message must therefore name the file,
    section,
    dependency and the hand-edit remedy.
- The hook cannot tell which verb is running.
  Probe `~/temp/agent/catalog-updateconfig-argv.ts` (fixture hook restored afterwards) logged `process.argv`,
  `process.execArgv` and
  the `npm_*` and `PNPM_*` environment keys inside `updateConfig` for `install`,
  `add` and `run` (the output for `update` and
  `remove` was not read):
  `argv` is empty after the script name,
  no `npm_command` or `PNPM_*` variable exists,
  and `execArgv` is
  pnpm's own bootstrap script,
  identical for every verb.
  pnpm starts the pnpmfile in a child Node process that talks to it over a JSON protocol on stdout and stdin,
  so a hook must never write to stdout,
  and the verb is not passed.
  Blocking is therefore all or nothing across verbs;
  selecting repair verbs would need a platform-specific read of the parent
  process command line,
  which is unmeasured and would not work on the Windows runners.
- Landing order:
  a throwing `updateConfig` makes every pnpm verb except `config get` and `--version` exit 1 while the
  `zod` violation exists,
  so the `zod` fix and the hook must land in the same commit.
- Cost:
  the same scan over the real repo's workspace manifests took a median of 4.9 ms over 15 in-process runs
  (minimum 4.4,
  maximum 9.0 on the first),
  and it found the one real violation,
  `zod` in `cost-dismissal`.
  This is the scan alone,
  not pnpm's own startup.
- Limits:
  the hand-written glob expansion supports only `*` segments;
  the real workspace uses `package/*/*`,
  which it handled.
  A `!` negation pattern or `**` in the workspace file would need handling.

### CI facts that bear on a PATH shadow

- Both CI systems install pnpm through the `task-cli-setup` action and run `task-run prepare:pnpm`
  (`.github/workflows/current-baseline.yml`,
  `.depot/workflows/current-baseline.yml`).
  The two `.depot/workflows` files pass `pnpm-version: '12.14.0'` explicitly,
  while the action reads `.tools.pnpm.version`
  from `toolchain.lock.json` (12.16.1) when the input is empty,
  as the `.github/workflows` files do.
  So CI runs two pnpm versions.
  A container probe (`~/temp/agent/pnpm-ci-version-probe.md`,
  CachyOS image,
  `--memory=2g --cpus=2`,
  no mounts,
  with a
  12.16.1 control that reproduced every host result) found `updateConfig` identical on 12.14.0:
  it fires on every verb except `config get` and `--version`,
  a throwing hook blocks no-op and frozen install,
  `run`,
  `exec`
  and `list` with the policy message,
  and `--ignore-pnpmfile` skips it.
  Only `preResolution` differs:
  it does not fire on frozen install or `prune` on 12.14.0.
  Not covered by that probe:
  node 24 (the image has 26.12.1) and recursive filters (covered on 12.16.1 only,
  above).
- The action puts node,
  pnpm and the task CLI on `PATH` (`.github/actions/task-cli-setup/action.yml`) and nothing about a
  shadow directory.
  The mpm shadow reaches callers because the toolchain plan links it into a gitignored directory first on `PATH`
  (`doc/decision/mpm-toolchain-adoption.md`);
  a pnpm shadow would need the same link,
  and CI would need a built Rust binary
  for each runner OS (the workflows include Windows steps).
- The Rust dependency catalog has `clap`,
  `serde` and `serde_json` but no YAML crate (`rust-catalog.toml`),
  so a Rust checker must hand-parse `packages:` too or add a crate.

### Upstream state (agent report `~/temp/agent/pnpm-upstream-research.md`, source reading of pnpm commit `c7b46aedef`, not run)

- Issue [8644](https://github.com/pnpm/pnpm/issues/8644) "Checks to enforce pnpm catalog protocol usage" is the direct request.
  It proposed "Or the `pnpm install` command fails if enforcement is enabled" and was closed `COMPLETED` on 2026-02-15
  with no closing PR,
  on the grounds that `catalogMode` "likely addresses this feature request".
  `catalogMode` does not:
  its code is wired to add and update only (`crates/package-manager/src/catalog_mode.rs:1`).
- Discussion [8567](https://github.com/orgs/pnpm/discussions/8567) holds an unanswered question from 2026-03-11 asking whether a
  hand-edited `package.json` should be covered;
  no maintainer replied.
  Install-time warnings were also suggested in 15022 (now a locked discussion);
  neither it nor 11742 has a maintainer reply.
  The maintainers point to `syncpack` as the external route.
  No upstream fix is on the horizon,
  so the repo needs its own check.
- No pnpm command or setting validates specifiers:
  no `pnpm catalog` or lint command exists,
  and `blockExoticSubdeps`,
  `trustPolicy`,
  `minimumReleaseAge` and `strictDepBuilds` act on resolved packages.
- The agent's source reading agrees with the probes:
  `updateConfig` receives settings (`dir`,
  `workspaceDir`,
  `workspacePackagePatterns`) and no manifests,
  runs before the command on almost every verb,
  and does not run on `dlx` or `doctor`.
  `pnpm dlx` is therefore an unchecked path,
  which is acceptable because it installs into a throwaway project.
- The no-op install fast path refuses to short-circuit when a pnpmfile is present,
  which is why `updateConfig` still runs.
- The agent's `gh search` first PR round used a quoted multi-word phrase that returns nothing,
  so only its rerun counts.
  It did not search `pnpm/rfcs`.
- The earlier `pi` verdict (defer the hook) was reached on my wrong premise that no hook reaches the no-op install and that the
  hook must wait for manifests through `readPackage`;
  the verdict should be treated as superseded on that point.

### What this changes

Reach is no longer an argument for the Rust shadow over a hook:
an `updateConfig` hook covers the no-op install and every
other verb on any caller,
with `--ignore-pnpmfile` as the only bypass found,
while a shadow covers only callers that resolve
it first.
The shadow's remaining case is language unity and one owner for the rule,
not coverage.

## Mechanism decision (user, after the reboot research)

The user chose option B,
"Rust + clap pnpm wrapper first,
everything in it",
against my ranking of A (updateConfig hook plus lint),
C (Rust checker plus hook stub),
D (lint and CI only) above B.
The ranking and the cons of each option were shown with the question,
so this is an informed choice and stands.
The user also chose to wait for the second `pi` review before deciding;
the review finished after that answer,
and its findings
are below.
Consequences recorded,
not yet confirmed by the user as the full design:

- The earlier decisions about the policy,
  no exceptions,
  named catalogs,
  manifests only,
  syntax only and `zod` into the catalog
  still stand,
  because they are about the rule,
  not the mechanism.
- The two TypeScript packages (`core-specifier-policy`,
  `cli-specifier-policy`),
  the `.pnpmfile.mjs` hook wiring and the mise lint
  task as the primary mechanism are not chosen.
  The `updateConfig` measurements remain valid facts,
  and a hook can still be added later as a second layer.
- A wrapper sees `argv`,
  so unlike the hook it can tell the verb and scope which verbs it blocks.
  It also reaches only callers that resolve it first on `PATH`;
  CI's setup action puts the real pnpm on `PATH`,
  so CI needs an
  explicit step that runs the checker (a `check` subcommand of the same binary,
  built without pnpm).
  Without that step the policy is not enforced in CI.

## Second pi review (`openai/gpt-6.1-sol`, prompt `~/temp/agent/pi-review2-prompt.txt`, output `~/temp/agent/pi-review2-output.txt`)

Told about the user's bad feeling and that the first verdict rested on the disproved no-op premise.
Verdict:
do not ship A as written;
proceed with an omitted option E,
a dependency-free Node checker with mandatory CI source lint
and an advisory (non-blocking) `updateConfig` hook,
no Rust requirement.
The user chose B before this arrived,
so E is recorded as a considered alternative,
not the plan.
Findings that bear on any design,
including B (the reviewer's claims,
not independently verified here):

- A hard-failing hook turns a manifest mistake into a repository-wide pnpm outage,
  and `--ignore-pnpmfile` also disables the
  existing dependency blocklist.
  A wrapper can avoid this by scoping verbs;
  the lint backstop must not need pnpm to run.
- CI enforcement is a proposed property,
  not an established one:
  whether any workflow runs the root `lint` task is unprobed,
  and
  `package-paused/` is outside the workspace.
  The checker needs an explicit CI step,
  repository-root-relative tracked paths including the root manifest,
  working-tree
  contents,
  NUL-delimited git output,
  and failure (not success) when enumeration fails.
- The probe's glob expansion was wrong outside its fixture:
  `**` taken literally,
  `!` exclusions ignored,
  partial-segment wildcards
  taken literally,
  and a missing directory throwing.
  Any enumerator must fail closed on a pattern it cannot expand,
  or reuse a maintained facility whose semantics are tested against pnpm.
- `isCatalog` used `RegExp.test`,
  which coerces an array such as `["catalog:team"]` to a matching string;
  a checker must require
  string values and define malformed dependency maps.
- Landing the `zod` change may need a regenerated lockfile and clean frozen-install tests on both pnpm versions,
  Node 24 and Windows.
- A preflight is not a postcondition:
  a script run from a valid workspace can write a literal specifier and exit 0.
- Several handover statements were stronger than their evidence:
  "every verb" (the agent's source reading found `dlx` and `doctor`
  exceptions,
  which this handover already records),
  "only bypass" (a valid alternative pnpmfile was not tested),
  "must hand-parse
  YAML" (a Rust stub or wrapper can be handed `workspacePackagePatterns` or read `pnpm-workspace.yaml` itself),
  and Node 26 probes
  do not cover Node 24.
- Option C has a bootstrap obligation,
  not a circular dependency:
  the checker binary must be built before the first hooked pnpm call.
- The bad feeling is not evidence for or against Rust;
  a structural reading is that the design grew from specifier enforcement
  into a prerequisite for most pnpm work while bootstrap,
  recovery and CI execution stayed unspecified.

## Precedent and inputs for option B (read, not built)

- `package/mpm-shadow/cli` (`Cargo.toml`,
  `src/real_binary.rs`,
  `src/shim.rs`,
  README):
  the mode is chosen by the name the binary is started under,
  forwarded arguments are never given to clap
  (clap owns only the shadow's own commands),
  the real tool is found by a PATH search that skips every candidate resolving to the
  shadow itself,
  the child inherits the standard streams,
  and the exit status is passed through (`128` plus the signal on a signal).
  It is Unix-only:
  `real_binary.rs` imports `std::os::unix::fs::PermissionsExt` and splits `PATH` on `:`;
  its release archives are Linux only (x86_64 and aarch64).
  Its dependencies for the shadow work are `clap` and `serde_json` (with `preserve_order`);
  the rest serve its other jobs.
  Per-crate pins come from `rust-catalog.toml` through file-enforcer,
  not by hand.
- The Rust catalog (`rust-catalog.toml`) holds `clap`,
  `serde`,
  `serde_json`,
  `anyhow`,
  `globset`,
  `ignore`,
  `libc`,
  `toml`,
  `tracing`;
  it holds no YAML crate.
  Adding a crate is governed by `doc/decision/rust-dependency-catalog.md`.
- The re-run scan finds 256 tracked `package.json` files (it found 248 earlier,
  so the count moved) and still exactly one
  violation,
  `zod` in `cost-dismissal`;
  no tracked `package.json5`,
  `package.yaml` or `package.yml` exists,
  so those pnpm manifest
  formats are a known limit,
  not a current gap.
- CI:
  the baseline workflows (`.github/workflows/current-baseline.yml`,
  `.depot/workflows/current-baseline.yml`) and
  `npm-release.yml` run on Linux runners;
  macOS and Windows runners appear only in Rust build and test workflows
  (`cargo-publish.yml`,
  `music-player-release.yml`,
  `fs-id.yml`,
  `readonly-semantic-bridge.yml`).
  Whether those non-Linux jobs run `pnpm install` was not checked.

## Dependency round for option B (user answers)

Adopted from the `mpm-shadow` precedent without asking,
veto open (the user's answers did not veto any):
crate home `package/pnpm-shadow/cli` (package `monochromatic-pnpm-shadow`,
binary `pnpm-shadow`,
linked as `pnpm` the way
`mpm-shadow` is linked as `mpm`);
mode chosen by the name the binary is started under;
forwarded pnpm arguments never reach clap;
the real-pnpm search skips every candidate resolving to the shadow itself;
dependencies `clap` and `serde_json` from
`rust-catalog.toml`,
`tempfile` for tests,
no `anyhow` or `tracing`;
a `check` subcommand that CI and a mise lint task call without pnpm.

- Manifest enumeration:
  the user chose the `ignore` crate walk,
  against my ranking (git subprocess first).
  Consequence:
  the scope is the set of `package.json` files the walk sees,
  not strictly the tracked set.
  Measured today with ripgrep,
  which uses the same crate:
  the default walk,
  the walk with hidden files,
  and
  `git ls-files --cached` return the identical 257 manifests,
  so there is no difference yet.
  The sets can diverge when an untracked,
  non-ignored `package.json` exists (it would fail the check),
  or when a tracked file is
  gitignored (it would be skipped);
  the crate also needs a git work tree to apply `.gitignore` by default,
  which is unmeasured here.
  Probe `~/temp/agent/compare-enumeration.ts` (its worktree-list step failed,
  so where linked worktrees live relative
  to the repo root is not established;
  a worktree inside the tree and not ignored would add its manifests to the walk).
- Workspace and YAML:
  no YAML,
  as recommended.
  The root is the nearest ancestor holding `pnpm-workspace.yaml` (existence only),
  and the preflight checks the same manifests as `check`.
  Known effect:
  a violation in `package-paused/` also blocks pnpm in the wrapper although pnpm ignores that manifest.
- Platforms:
  the user wrote "unix now,
  cross platform for a later version."
  Recorded as:
  Unix-only for now (as `mpm-shadow`),
  cross-platform planned for a later version.
  Whether `check` should already be portable now (my first-ranked option) was not separately answered.
- Observed during this round,
  not caused by this session:
  every `git` command in the main checkout currently fails through the
  git shadow with `config-invalid` for `cli-git.config.jsonc` (`policies.security/forbidden-strings[1].rulesFile must be a
  string, found an array`),
  which looks like another session's in-progress work on forbidden-strings.
  The last handover commit before it,
  `f6df1b5d3f`,
  succeeded.

## CI execution facts for the `check` step (measured by reading the workflows)

- No workflow runs the root `lint` task:
  the `task-run` calls across the workflows include `lint:monochromatic-lint` (three times)
  but never `lint`,
  so a new task added to the root `lint` list would not run in CI by itself.
  This confirms the `pi` finding that CI execution of a new lint is not automatic.
- `.github/workflows/current-baseline.yml` and `.depot/workflows/current-baseline.yml` trigger only on `workflow_dispatch`.
  `.github/workflows/advisory-core.yml` triggers on pushes and pull requests to `main`,
  runs `task-run prepare:pnpm`,
  and then runs named repository-invariant steps with `continue-on-error: true` (`Rust catalog drift`,
  `pnpr config drift`)
  that print `::error::` with the remedy;
  this is the existing precedent for a check step.
- `prepare:pnpm` runs `prepare:pnpm:install` (`pnpm install`,
  alias `pi`) then `prepare:pnpm:others:*` (`mise.toml`).
- CI builds Rust crates through `task-run //package/<crate>:build`,
  so a Rust `check` binary can be built in the same job.

## Behavior round for option B (user answers)

- Verb scope:
  the user chose "Warn only,
  never block",
  against my ranking (install-class verbs first,
  warn only last).
  The wrapper prints every violation to stderr and always runs the real pnpm with the arguments untouched.
  Consequences:
  no verb parsing and no fail-closed argument scan are needed for blocking;
  the second `pi` review's outage
  finding does not apply to the wrapper;
  the wrapper cannot stop a violation from landing locally;
  enforcement exists only where
  `check` is run and fails (CI,
  a lint task).
  Whether the warning appears on every invocation or only on install-class verbs is not decided;
  the first needs no verb parsing.
- Skip switch:
  none,
  as recommended.
  The only way around the wrapper stays calling the real pnpm directly.
- CI step:
  a named step before `task-run prepare:pnpm` in `.github/workflows/advisory-core.yml`,
  as recommended.
  Measured after the answer,
  which the question had listed as a check to make:
  that workflow states "results are advisory;
  no required checks and no branch-protection change"
  (`Summary` step),
  and its final `Enforce advisory result` step fails the job only when the `static`,
  `build` or `unit` step
  failed;
  the existing `Rust catalog drift` and `pnpr config drift` steps use `continue-on-error: true` and are not in that list,
  so their failures leave the job green.
  A new check step therefore shows a red step in the log but does not fail the job unless it is added to the `Enforce advisory result`
  condition (or run without `continue-on-error`),
  and no branch protection makes the job a merge gate either way.
  The line numbers in this workflow moved between two reads in this session,
  so another session edited it.
- Not asked yet:
  whether the new step is added to the `Enforce advisory result` condition;
  root discovery now that the wrapper does
  not parse `--dir` or `-C` (the nearest ancestor from the current directory is the simple answer);
  warning wording;
  exit-code behavior of `check` (nonzero on any violation);
  the test corpus layout (a JSON vectors file read by the crate's nextest
  suite,
  since no TypeScript implementation is planned);
  the `zod` landing order (the `check` step and the `zod` fix must land
  together or the new CI step fails from its first run);
  PATH linking through the toolchain plan.

## Two gaps found after the behavior round (my research gaps, found by reading)

- The CI step chosen above runs rarely on the push path.
  `advisory-core.yml` has a `gate` job and the `core` job runs only when `needs.gate.outputs.allowed == 'true'`.
  Pushes are admitted under budgets of 2 per minute,
  4 per hour and 8 per day (counted from `gh run list`);
  pull requests and
  manual dispatch are always admitted.
  Commits on `main` per day over the last eight days were 436,
  382,
  722,
  73,
  5,
  26,
  1049,
  268 and 2 (local commit counts;
  auto-push
  means many reach the remote),
  so on a normal day most pushes are not admitted and a check inside `core` would not run for them.
- I never examined the repository's own commit-time policy engine as a home for this check,
  although `package/git-policy/cli`
  (`cli-git`,
  the shadowing `git`) is where this repository enforces manifest invariants today:
  - `doc/decision/cli-git-policies-platform.md`:
    the wrapper enforces policies with severity `off`,
    `warn` or `error`,
    fails closed
    when an enabled policy cannot complete,
    reads candidate bytes of staged files,
    and treats CI as the authoritative layer that must
    stay independent of the local wrapper ("Local enforcement remains best-effort fast feedback;
    CI remains authoritative").
  - `package/git-policy/cli/README.md`,
    "Policies":
    the wrapper ships a fixed set of policies and no others;
    the config keys
    `plugins` and `trust` are refused,
    so a new policy means a native Rust policy inside that crate
    (`src/native/policy/policy_registry.rs` and siblings),
    listed in `cli-git.config.jsonc` where the four optional ones are listed.
    `mono/dependent-version-bump` already reads package manifests at commit time and blocks at `error`.
  - A `mono/catalog-specifiers` policy would reject a commit whose staged `package.json` bytes violate the rule,
    at the point where a
    violation lands,
    with no directory walk and no PATH-order dependency beyond the git shadow every commit already uses.
    Costs:
    it is another session's active crate (versions 0.4.x,
    release ordering,
    and a toolchain sync from another worktree
    reinstalled an older `cli-git-native` into the shared `~/.cargo/bin` today and broke every git command until reinstalled);
    it acts only at commit time,
    so a local `pnpm install` stays silent without the wrapper's warning.

## Layers round (user answers)

- Layers:
  the user selected only the `cli-git` commit policy `mono/catalog-specifiers` (a native Rust policy in
  `package/git-policy/cli`,
  severity `error` by default),
  alongside the warn-only pnpm wrapper.
  The separate always-run workflow and the `advisory-core` step were not selected.
- The second question (should a CI step fail the job) was answered "Fail the job on a violation",
  but it was conditional on a CI
  step being kept and the first answer kept none,
  so the CI position is ambiguous:
  either no CI step now,
  or a job-failing step.
  Recorded as unresolved;
  asked again below.
  Without a CI step there is no layer independent of the git shadow,
  which `doc/decision/cli-git-policies-platform.md` says CI must
  provide ("CI remains authoritative").
- Read after the answer:
  `git cli-git check --all --policy <id>` already runs one policy over the whole repository and prints JSON
  lines,
  and `--no-enforce-<policy id>` is the platform's built-in per-invocation skip
  (`package/git-policy/cli/README.md`,
  "Commands" and "Wrapper-only options").
  So a policy in `cli-git` also gives a whole-repository check command and a skip switch without new code.
  That skip switch exists for every policy;
  the "no skip switch" answer applied to the pnpm wrapper only.

## Clarification round (user answers)

- CI:
  a job-failing CI step that runs `git cli-git check --all --policy mono/catalog-specifiers`,
  as recommended.
  This resolves the earlier ambiguity:
  a CI step exists and its failure fails the job.
  Home (a step in `advisory-core.yml`,
  which skips most pushes under its budget gate and blocks no merge,
  or a separate always-run
  workflow) is still to be decided;
  CI must be able to build or install the cli-git native binary.
- Rule home:
  the user chose a shared Rust library crate used by both the cli-git policy and the pnpm wrapper,
  against my ranking
  (wrapper calls `git cli-git check` first).
  Consequences:
  a new library crate with README and nextest tests;
  both consumers take it through path-and-version dependencies
  (the `mpm-shadow` to `monochromatic-jsonc-edit` precedent);
  the two crates release in step,
  and cli-git's own
  `mono/dependent-version-bump` governs the bump order;
  the wrapper then needs its own manifest enumeration (the `ignore` walk the
  user chose earlier) and JSON reading,
  because it no longer calls cli-git.
- Coordination:
  the user answered "You figure out amongst yourselves.
  You have the send message tool."
  So this session may message the session that owns `package/git-policy/cli` to agree who adds the policy;
  it does not edit that
  crate on its own.
  Peer sessions seen by `ListAgents`:
  `monochromatic-7a` (idle),
  `monochromatic-6e` (busy,
  the file-enforcer rewrite,
  which holds
  the merged rules-list feature and cutover config switch for forbidden-strings and git-policy),
  and a translation session.
  Which one owns git-policy was not known when the messages were sent;
  see the "Progress log" section.

## Last design round (user answers) and the peers' replies

- Wrapper warning scope:
  every invocation,
  except arguments that are only `--version`,
  `-v`,
  `--help` or `-h`;
  no verb parsing.
- CI home:
  a separate always-run workflow on push and pull request to `main` with `cancel-in-progress`,
  failing the job on a violation.
- Adopted with a veto window,
  not vetoed:
  the shared library crate is `package/catalog-specifier/core-rust`,
  crate
  `monochromatic-catalog-specifier`,
  following the `jsonc-edit/core-rust` precedent.
- Confirmation:
  the user did not confirm.
  They answered:
  "Add an AGENTS.md rule to say re-reviews must be ran before asking to confirm shared understanding for grilling
  workflows."
  Done:
  rule `RRG` added to `AGENTS.md` (192 characters,
  29 words),
  with `CLAUDE.md` regenerated by `mise run sync:files`.
  So the design as it now stands needs a fresh independent re-review before confirmation is asked;
  the earlier `pi` reviews predate
  the cli-git policy,
  the shared library crate,
  the separate CI workflow and the warn-only wrapper.
- `monochromatic-7a` owns `package/git-policy/cli` and its releases and will build the policy from the draft spec below once the
  design is confirmed;
  it will not start before then.
  Its release order (from its message,
  a peer's report,
  not verified here):
  1.  Publish the shared library crate first.
      The wrapper pins workspace library crates by exact version in its `Cargo.toml`
      (for example `forbidden-strings = { path, version = "=0.6.0" }`),
      and publishing the wrapper fails unless that exact version is
      already on crates.io.
      The shared crate needs inline pins in dependent manifests (the release tool
      `package/crate-release/cli/bin/release.ts` moves only inline `name = { ... version = "=x" }` lines,
      not table-style
      `[dependencies.name]`) and its own `release` and `release:wait` mise tasks like `package/forbidden-strings/cli/mise.toml`.
  2.  7a releases the wrapper (`native:release`,
      `native:release:wait`) once the policy,
      its tests and the mutation and fuzz gates pass.
      Since 2026-11-01 those tasks only dispatch the `crate-release` workflow and follow its run,
      and nothing compiles on the
      developer machine (`doc/decision/releases-run-in-ci.md`);
      the shared crate of step 1 also needs an entry in
      `package/crate-release/cli/releasable-crates.json` and in the `crate` choices of `.github/workflows/crate-release.yml`.
  3.  `toolchain:update` bumps the wrapper in `toolchain.lock.json`;
      it has no per-tool filter,
      so it also moves every other floating
      tool,
      and the user decides each time.
  4.  `toolchain:sync` installs it as the production `git`.
  The CI step must land only after the lock names the new wrapper and every runner installs from that lock,
     because
  `git cli-git check --all --policy mono/catalog-specifiers` fails on a wrapper that predates the policy.
  The wrapper installs into the shared `~/.cargo/bin`,
     and a worktree whose lock pins an older wrapper downgrades it for every tree
  on its next sync (observed 2026-10-31;
  the downgrade is now refused by `mpm-shadow`,
  see
  `package/mpm-shadow/cli/README.md`,
  "Repository-published tools keep only the newest version").
  7a's queue holds nothing that touches policy registration.
- `monochromatic-6e` confirmed 7a owns the cli-git work and added constraints:
  cli-git will pin `monochromatic-jsonc-edit =0.1.2`
  so that crate must be on crates.io before any new git-policy-cli publish;
  forbidden-strings releases before cli-git when both change;
  a repository-published tool is never downgraded and a tree with an older lock runs the newest installed one,
  so a new policy
  release must keep reading older `cli-git.config.jsonc` files.

## Draft spec for the cli-git policy and the shared library (not built; for the owning session and the user)

- Policy id `mono/catalog-specifiers`,
  default severity `error`,
  no options,
  listed in `cli-git.config.jsonc`
  next to `mono/dependent-version-bump`.
- Scope:
  every staged or checked `package.json` at any path (the root and `package-paused/` included),
  read as strict JSON from the
  candidate bytes the platform provides,
  never from the worktree.
- Rule:
  in `dependencies`,
  `devDependencies` and `optionalDependencies` a value must be a string equal to `catalog:`,
  matching
  `catalog:<name>` with a non-empty name,
  or exactly `workspace:*`;
  in `peerDependencies` a value must be `*`,
  `catalog:` or
  `catalog:<name>`.
  Anything else is a finding:
  digits,
  ranges,
  prereleases,
  `workspace:^`,
  `link:`,
  `file:`,
  git and URL forms.
  A non-string value,
  a non-object section or malformed JSON is a finding or a failure,
  never a pass (the second `pi` review showed
  a coercion bug where an array `["catalog:team"]` matched).
  `catalog:<name>` is syntax only:
  no catalog entry lookup,
  and catalog entries themselves are not checked.
- Findings name the file,
  the section and the dependency,
  and give the remedy:
  move the version into the `catalog:` section of
  `pnpm-workspace.yaml` and write `catalog:`,
  or use `workspace:*` for a workspace package.
- Library crate:
  pure functions over parsed JSON and over bytes,
  no file or process access;
  home and name to be chosen under
  `package/` with the singular-segment rule;
  consumed by the cli-git policy and the pnpm wrapper.
- Tests the library needs:
  a vectors file (specifier,
  section,
  verdict),
  adversarial values (arrays,
  objects,
  numbers,
  null,
  empty
  string,
  `catalog:` with trailing space,
  `workspace:*.0`,
  `*` in install sections,
  peers starting with `*`),
  malformed JSON,
  and
  duplicate keys.
- Landing order:
  the `zod` fix (move `zod` into the `pnpm-workspace.yaml` catalog as `>=4.6.7`,
  switch `cost-dismissal` to
  `catalog:`) must land before or with the CI step,
  or the step fails from its first run;
  the cli-git policy at `error` would also
  block any commit that touches `cost-dismissal/package.json` until then.

## Third pi review (re-review required by rule RRG before confirmation is asked)

Prompt `~/temp/agent/pi-review3-prompt.txt` (this handover,
the cli-git policies platform decision,
`advisory-core.yml` and
`cli-git.config.jsonc`,
with a header listing the design as it then stood),
output `~/temp/agent/pi-review3-output.txt`,
model `openai/gpt-6.1-sol`.
It returned.
Verdict:
proceed with named changes,
and do not confirm the design as complete yet.
Single strongest reason:
the design does not guarantee that the supposedly authoritative CI command runs the intended policy at
error severity.
The reviewer's claims below are its own unless marked measured here.

Findings,
most severe first:

- Blocker,
  CI provisioning is missing:
  a fresh runner has ordinary Git,
  so `git cli-git ...` is not a subcommand;
  an older shadow
  rejects the new policy;
  updating `toolchain.lock.json` provisions nothing unless the workflow installs the binary from the lock
  before the check.
  Also the notes still carry two incompatible bootstrap plans ("build or install").
  Required:
  choose one concrete bootstrap (a pinned,
  verified native release into an isolated runner directory,
  or a build from
  committed sources with a pinned toolchain and locked dependencies),
  not requiring `prepare:pnpm` first and using no floating
  update;
  verify on a fresh home with ordinary Git,
  no Cargo binaries,
  no `node_modules`,
  and a planted literal that must fail
  for a catalog finding,
  not for a missing tool.
- High,
  `--policy` is documented as a filter:
  the platform document says "A repeated `--policy` filters the enabled set",
  and native
  optional policies run only when listed;
  removing the entry or setting `off` or `warn` could leave the CI command green or
  checking nothing.
  Measured here:
  an unknown policy id fails with exit 2 (`config-invalid`,
  listing the shipped policies),
  and a listed error policy
  (`mono/forbidden-root-context`) on the clean repository exits 0 with no output.
  Not measured:
  what `--policy` does for a shipped policy that is absent,
  `off` or `warn` (needs a disposable fixture).
  Required:
  CI forces the policy at `error` or verifies it exists and is error-enabled;
  an empty selection must not count as success.
- High,
  the rollout can break git before the repair runs:
  a new config entry,
  an older shadow synced into the shared
  `~/.cargo/bin` by another worktree,
  and config-loading git commands failing;
  the handover reports a real downgrade and a real
  `rulesFile` array rejection today.
  No dependency cycle exists in library,
  then cli-git release,
  then pinned install,
  then config activation,
  but activating the
  config before the install is a practical trap,
  and `monochromatic-jsonc-edit =0.1.2` belongs in the release graph.
  Required:
  version-scoped installs per checkout,
  or proof of monotonic installs and backward config compatibility;
  a documented
  recovery through an absolute real-git path and an out-of-band install (a policy skip flag does not help when config loading fails).
- High,
  a failing workflow is not a landing gate:
  a push that bypasses the shadow lands before the workflow runs,
  a red check
  that is not required does not stop a merge,
  and `cancel-in-progress` under hundreds of commits a day means not every pushed
  revision gets a completed check.
  Required:
  state the guarantee honestly as detection (a completed run reports violations) or prevention (a required PR check),
  and
  call a cancelling push workflow a latest-tip check;
  inspect branch rules and the concurrency key.
- Medium,
  the warn-only wrapper lacks an operational contract:
  an unreadable manifest,
  a JSON parse error,
  a walk error or no
  workspace root must produce an advisory diagnostic and then delegate to pnpm,
  never stop it;
  `pnpm --dir /repo-b` from `/repo-a`
  scans the wrong tree;
  child status and signals must be preserved;
  only failing to find any real pnpm is fatal.
  Fixed here:
  the "fail closed" wording in Open items was stale blocking-design language and is replaced below.
- Medium,
  the two enumerations are different scopes:
  the policy sees staged or tracked bytes,
  the wrapper sees whatever the
  `ignore` walk finds,
  so they diverge on tracked-but-ignored manifests,
  untracked manifests,
  nested checkouts,
  partially staged
  files,
  nested workspace markers,
  a copy without `.git` and machine-specific ignore settings.
  My equivalence probe used ripgrep,
  not the actual `WalkBuilder` configuration.
  Required:
  do not describe the wrapper as the same repository check;
  a fixture should compare the real walk with the policy scope.
- Medium,
  the predicate contract is incomplete:
  duplicate keys,
  `"catalog: "`,
  `"catalog:team "`,
  `"catalog:team:other"`,
  embedded
  whitespace and `null`,
  array or scalar sections have no stated expected result;
  a caller that parses into a map first discards
  duplicate keys.
  Required:
  one normative byte-to-findings contract written before implementation,
  without inventing catalog-name rules that pnpm
  does not have.
- Medium,
  the guarantee is narrower than central version control:
  catalog indirection (`catalog:` pointing at
  `workspace:^1.0.0`),
  `overrides`,
  package extensions and `.pnpmfile.mjs` rewrites are not checked;
  tracked manifests outside the
  workspace get no pnpm validation;
  a newly tracked `package.json5` or `package.yaml` would escape a JSON-only rule;
  published
  manifests legitimately carry literal versions.
  Required:
  document it as a source-`package.json` reference-syntax rule,
  and consider a cheap filename-level rejection of
  unsupported manifest formats without a YAML parser.
- The `zod` catalog range:
  `^4.6.7` to `>=4.6.7` removes the upper major bound;
  the existing override masks it today;
  the measured
  fact is only that the locked resolution does not change,
  not that the ranges are equivalent.
  Measured here:
  the catalog uses `>=` floors for 155 entries and `^` for one,
  so `>=4.6.7` follows the repository convention,
  but
  the non-equivalence stays stated.
- The wrapper is not on the enforcement critical path:
  it gives feedback about uncommitted mistakes and a warning before an
  install,
  when it is actually selected;
  it gives no postcondition,
  no coverage of callers that bypass `PATH`,
  no staged-byte
  correctness and no independent enforcement.
  The reviewer advises landing the commit policy and CI first and shipping the wrapper afterwards or independently;
  the choice to
  build it stays the user's.

Sequencing the reviewer proposes (a review proposal,
not decided):

1.  Fix `zod` first,
    with the intended range and any lockfile update,
    for a zero-violation baseline.
2.  Fix the library's byte-to-findings contract,
    publish it with inline exact-version dependencies and release tasks.
3.  Satisfy the other cli-git publication prerequisites,
    including the `jsonc-edit` pin.
4.  Add and release the native policy without activating its config entry yet;
    test the new binary against the old config.
5.  Establish deterministic installation and the CI bootstrap;
    update the locked version by a targeted operation if possible,
    otherwise review the whole fleet-update diff.
6.  Activate the `error` policy and land the independently provisioned CI workflow.
7.  Ship the advisory pnpm wrapper independently.
It also notes that the staged-only commit policy does not need `zod` and activation in one commit (that was the abandoned hook's
constraint),
   though fixing `zod` first avoids a tripwire and the whole-repository CI check needs a clean baseline.

The bad feeling,
per the reviewer:
a one-rule source check has accumulated a published library,
two consumers,
native git release
coordination,
a fleet toolchain update,
shared executable state,
a PATH shadow and a new workflow,
while the authoritative execution
path stays underspecified.
The observed git breakage is integration risk,
not evidence against Rust.
It asks for stronger warnings on two choices that went against my rankings:
the `ignore` enumeration (a scope choice,
not just a
faster walk) and wrapper-first delivery (its marginal value dropped once the commit policy and CI existed).

## Confirmation and the review-driven decisions (user answers, 2026-11-01)

The user confirmed the shared understanding and the decision order,
after the third review was folded in.

- CI bootstrap:
  the user chose "Install a pinned release from the lock",
  against my ranking (build from committed sources first).
  Consequence,
  stated plainly:
  the CI workflow can land only after the cli-git release that contains the policy is named in
  `toolchain.lock.json` (a `toolchain:update`,
  which moves every floating tool and needs the user's decision each time) and the
  runner installs it from the lock into an isolated directory before the check;
  a stale lock turns CI red for infrastructure reasons.
  The review also asked CI to force the policy at `error` or verify it is error-enabled;
  not yet designed,
  and not measured.
- Other manifest formats:
  the policy also rejects tracked `package.json5`,
  `package.yaml` and `package.yml` (the recommended option).
- Wrapper timing:
  after the commit policy and CI (recommended);
  the wrapper stays wanted.
- Adopted items were not objected to:
  `zod` catalog entry `>=4.6.7`;
  wrapper scan errors are advisory and pnpm still runs;
  the
  wrapper never parses `--dir` or `-C`;
  the wrapper is a separate advisory check;
  CI promises detection,
  not prevention;
  the
  library's byte-to-findings contract is written before code.
- Order confirmed:
  `zod` fix,
  library contract and crate,
  cli-git policy built but not activated,
  CI bootstrap,
  activate the `error`
  policy and CI,
  then the wrapper.

## Progress log

- 2026-11-01,
  step 1 done:
  `zod` moved into the `pnpm-workspace.yaml` catalog as `'zod': '>=4.6.7'` and `cost-dismissal` switched to
  `"zod": "catalog:"`.
  Measured:
  `pnpm install --lockfile-only` (also with `--config.optimistic-repeat-install=false`) left `pnpm-lock.yaml` byte-identical,
  because the `overrides` entry `zod: '>=4.6.6'` replaces the specifier that the lockfile records,
  and
  `pnpm install --frozen-lockfile --lockfile-only` printed no error (its exit status was masked by a pipe to `tail`,
  so the absence
  of an error line is the evidence,
  not a status).
  The policy scan (`~/temp/agent/scan-specifiers.ts`) now finds 257 manifests and zero violations.
  Not run:
  a full `pnpm install` into `node_modules` and any test of `cost-dismissal`.

- 2026-11-01,
  step 2 started:
  the contract is written as the README of `package/catalog-specifier/core-rust` with its executable
  vectors in `fixtures/vectors.json`,
  before any code.
  Decisions I made while writing it,
  veto open (small,
  all about syntax the user did not specify):
  duplicate dependency names and duplicate checked sections are findings and every copy is still checked;
  a catalog name is
  `[A-Za-z0-9._-]+`,
  deliberately narrower than pnpm (which trims and takes any text);
  a leading byte order mark is removed;
  the
  parser is `parse_jsonc` from `monochromatic-jsonc-edit` (it keeps duplicates visible and accepts comments and trailing commas,
  which pnpm itself may reject);
  the path check refuses `package.json5`,
  `package.yaml` and `package.yml` by last segment,
  case sensitive.
- 2026-11-01,
  `monochromatic-7a` answered on `--policy` (measured on wrapper 0.4.6 and read from `management.rs` and `policy_engine.rs`;
  a
  peer's report,
  not re-measured here):
  an unknown id exits 2;
  for a shipped policy `--policy` only filters,
  so an `off` or absent
  optional policy runs nothing and exits 0 with no output (measured with `mono/forbidden-root-context` in a repository whose config
  did not list it),
  so a bare `--policy` run can pass vacuously today.
  Unlisted policies get severity `off` when their registry row has `off_unless_listed: true` (the four optional ones),
  else their
  default;
  `warn` runs and reports without failing.
  Consequences 7a recorded:
  `mono/catalog-specifiers` is registered with `off_unless_listed: true`,
  so shipping it does not activate
  it and an older `cli-git.config.jsonc` keeps working;
  a direct `--policy <id>` run whose policy is `off` will exit 2 with a message
  naming the config key (this lands with the policy work);
  lifecycle runs keep skipping `off` policies.
  This answers the third review's vacuous-pass finding for the planned CI step once that change lands.

- 2026-11-01,
  step 2 built:
  crate `monochromatic-catalog-specifier` 0.1.0 at `package/catalog-specifier/core-rust`
  (commits `d3a44a3f6a`,
  `9cb8b4b9a3`):
  `finding.rs`,
  `grammar.rs`,
  `check.rs`,
  README contract,
  `fixtures/vectors.json`.
  24 nextest tests passed in debug mode;
  clippy with warnings denied initially failed on `implicit_return` for closures
  (fixed by replacing closures with loops) and then passed.
  The user said mutation runs do not block publish and release,
  so publication is not held for them.
  Mutation guard check (`~/temp/agent/catalog-specifier-mutations.ts`,
  13 intended mutations against the committed sources,
  each
  restored from its original bytes;
  the tree was clean afterwards):
  8 were killed by failing tests (peer star,
  widened `workspace:*`,
  duplicate names,
  duplicate sections,
  byte order mark,
  peer
  section dropped,
  lone surrogate accepted,
  unsupported bytes still read);
  1 survived (a non-string finding whose path was blanked,
  because no test asserted that path);
  1 counted as killed only by a compile error (removing `package.yml` from a fixed-size array
  changed its type),
  so it did not exercise the test;
  3 were skipped because their pattern matched twice.
  I added a test asserting the path,
  section and name of a non-string finding,
  which covers the survivor;
  the compile-error mutation
  and the three skipped ones were not re-run.
  Publication steps prepared,
  not run:
  `publish` and `publish:dry-run` mise tasks,
  and jobs `cs-detect` and `cs-publish-crate`
  (plus the dispatch choice and the push path) in `.github/workflows/cargo-publish.yml`,
  copied from the `xe-*` jobs;
  they are not
  chained to other crates because the only dependency,
  `monochromatic-jsonc-edit` 0.1.2,
  is already on crates.io (measured today).
  The first publish of a new crate needs the owner's personal crates.io token (`doc/runbook/publish-crate-first-time.md`:
  the
  agent cannot create one,
  and a stored token answered 403 in earlier sessions),
  then Trusted Publishing is created by API as for the
  other crates.
  That is the current blocker for step 2's publication.

- 2026-11-01,
  step 2 published:
  `monochromatic-catalog-specifier` 0.1.0 was published to crates.io with the owner's bootstrap token,
  after the user answered that the token was logged in.
  `cargo package` packaged 15 files (115.7 KiB,
  34.0 KiB compressed) and verified by compile;
  `cargo publish --dry-run` passed with no
  403;
  the real `cargo publish` uploaded and waited for availability.
  Registry check:
  `max_stable_version` `0.1.0`,
  one version,
  not yanked,
  keywords and categories present.
  Consumer check:
  a disposable crate (`~/temp/agent/cs-consumer`) depending on `=0.1.0` from the registry compiled,
  and its
  assertions passed (a clean manifest gave no findings;
  `"zod": "^4.6.7"` gave one,
  with the remedy text).
  Trusted Publishing was created through the crates.io API with the token's `trusted-publishing` scope,
  printing statuses only:
  config `30143`,
  owner `Aquaticats` (id 343218767),
  repository `Monochromatic`,
  workflow `cargo-publish.yml`,
  no environment;
  the
  listing shows exactly one row.
  Trusted Publishing exercised:
  the dispatch `gh workflow run cargo-publish.yml --ref main -f crate=monochromatic-catalog-specifier
  -f dry-run=false` (run `41274797417`) concluded `success`;
  `cs-detect` and `cs-publish-crate` succeeded,
  including
  `Authenticate with crates.io (Trusted Publishing)` and `Publish (skips if this version is already on crates.io)`,
  with
  `Publish (dry run)` skipped.
  The log of `Publish (skips if this version is already on crates.io)` carries the annotation
  `monochromatic-catalog-specifier 0.1.0 already on crates.io; skipping publish.` and no upload line.
  The user answered that the token `monochromatic-catalog-specifier-bootstrap` was revoked;
  the agent then ran `cargo logout`,
  which printed that the `crates-io` token was removed from local storage,
  and `~/.cargo/credentials.toml` holds no token entry.
  The revocation itself happened in the dashboard and was not checked from here.
  `monochromatic-7a` was told the crate is published and given the API,
  the pin form and the remaining policy work.

- 2026-11-01,
  CI bootstrap measured while waiting for `monochromatic-7a` (task:
  the separate CI workflow,
  not yet written).
  The user chose "install a pinned release from the lock".
  `toolchain.lock.json` records `cargo:monochromatic-git-policy-cli` at
  `0.4.6` (route `cargo-binstall`,
  bin `cli-git-native`,
  linked as `git` into `.cache/cli-git-bin`,
  `artifacts` empty) and
  `cargo-binstall` at `1.26.2`;
  the repository's precedent for using them in CI is `.github/workflows/music-player-release.yml`
  (`jq` the version from the lock,
  `cargo install cargo-binstall --locked --version`,
  then `cargo binstall --no-confirm --version`).
  Probe `~/temp/agent/cs-ci-bootstrap/Containerfile` (CachyOS `latest`,
  `podman build --memory=2g --cpu-quota=200000`,
  nothing
  mounted,
  ordinary system git 2.56.0,
  fresh home,
  no cargo-installed binaries,
  no `node_modules`,
  a fixture repository holding one
  literal dependency),
  log `build.log`,
  exit 0:
  - `cargo install cargo-binstall --locked --version 1.26.2` compiled in 6 minutes 47 seconds on 2 CPUs.
  - `cargo binstall --no-confirm --disable-strategies compile --root /opt/cli-git --version 0.4.6 monochromatic-git-policy-cli` took
    2.8 seconds and logged `has been downloaded from github.com`,
    so the release archive exists and no compile was needed.
  - `cli-git-native --version` printed `monochromatic-git-policy-cli 0.4.6, shadowing git at /usr/bin/git`;
    the binary runs by its
    own name,
    so CI needs no `git` link:
    `cli-git-native cli-git check --all --policy <id>` is the whole command.
  - A whole-repository check of an existing policy exited 0;
    naming `mono/catalog-specifiers` against 0.4.6 exited 2 with
    `engine-failure` `config-invalid` (`Unknown policy ID`,
    listing the nine shipped ids),
    which is exactly the failure a lock
    that predates the policy produces,
    so a lagging lock fails loudly instead of passing.
  Correction after the user's challenge ("Doesn't cargo-binstall download gh releases too?"):
    yes,
    it does,
    and it fetched this same
  release archive in 2.8 s in the `~/temp/agent/cs-ci-bootstrap/Containerfile` probe.
    The 6m47s was only the compile of cargo-binstall itself (`cargo install`,
    the
  music-player precedent),
    and the "direct download against cargo-binstall" comparison that follows compared two different things.
  The right comparison is how cargo-binstall itself is obtained.
    The lock already pins a prebuilt one:
    row `cargo-binstall`,
    route
  `bin-sourced`,
    `artifacts.linux-x64` with `url`
  `https://github.com/cargo-bins/cargo-binstall/releases/download/v1.26.2/cargo-binstall-x86_64-unknown-linux-gnu.tgz` and a `sha256`
  (keys also for `linux-arm64`,
    `linux-arm64-musl`,
    `linux-x64-musl`).
  Measured (`~/temp/agent/cs-binstall-from-lock.ts`,
    host,
    empty scratch directory):
    downloading that archive took 0.78 s and its
  sha256 matched the lock;
    extraction 0.08 s;
    `cargo-binstall --no-confirm --disable-strategies compile --root <dir> --version 0.4.6
  monochromatic-git-policy-cli` took 6.60 s and installed `cli-git-native`;
    `--version` printed
  `monochromatic-git-policy-cli 0.4.6, shadowing git at /usr/bin/git`.
    About 7.5 s in total,
    with the cargo-binstall archive
  verified by the lock's checksum.
  Decision for the draft:
    use this lock-pinned route (prebuilt cargo-binstall from the lock row,
    verified by sha256,
    then binstall for
  the pinned cli-git),
    not `cargo install cargo-binstall` and not a hand-rolled download of the cli-git archive.
  The earlier direct-download measurement is kept as a measurement of that alternative only:
  The direct route was measured afterwards
  (`~/temp/agent/cs-direct-download.ts`,
    host,
    empty scratch directory):
    downloading
  `monochromatic-git-policy-cli-0.4.6-x86_64-unknown-linux-gnu.tar.gz` from the release URL that the crate's
  `[package.metadata.binstall]` `pkg-url` pattern gives took 0.67 s (2,685,208 bytes);
    `gh attestation verify <archive> --repo
  Aquaticats/Monochromatic` exited 0 in 9.69 s;
    extraction took 0.03 s and holds `cli-git-native` and `README.md`.
  The 0.4.6 release holds only the two Linux archives (x86_64 and aarch64) and no checksums file,
    so the attestation is the
  integrity check,
    and the lock's `artifacts` stay empty for this route;
    the step needs `gh` and a token (`GH_TOKEN`) on the runner.
  So the direct route costs about 11 seconds against about 7 minutes for the cargo-binstall route.
  CI design so far (draft,
    not written):
    a separate workflow on push and pull request to `main` with `cancel-in-progress`;
  read the two versions from the lock;
    install cargo-binstall and the pinned cli-git into `$RUNNER_TEMP`;
    run the check;
    plus a
  negative-control step that checks a throwaway fixture repository with one literal dependency and requires a nonzero exit,
    so the
  job cannot pass vacuously;
    the job fails on a violation (detection,
    not prevention).
  Not yet measured:
    the policy itself (not built),
    a check in a shallow CI checkout,
    and the workflow on a GitHub runner.

- 2026-11-01,
  rule changes after the user's challenges about the bootstrap measurement:
  - `QJ1` now says to split a measured total by its logged steps before blaming one and to compare options step for step
    (commit `f04bcf2590`).
  - `CM7` now says to install tools only from `toolchain.lock.json` (in containers and CI use its row's archive and sha256) and never to
    `cargo install` or compile a locked tool (commit `57f5078f6f`).
  - Cause recorded:
    the probe container compiled cargo-binstall because I copied `cargo install cargo-binstall --locked` from the
    music-player workflow without reading the lock's own `cargo-binstall` row,
    which pins a prebuilt archive with a sha256.
- 2026-11-01,
  the same fault fixed in `.github/workflows/music-player-release.yml` (user answered "Fix it now"):
  job `notices`
  (runner `ubuntu-24.04`) now reads the `linux-x64` url and sha256 from the lock,
  guards the url prefix,
  downloads,
  runs
  `sha256sum --check --strict`,
  extracts into `$RUNNER_TEMP/binstall`,
  adds it to `GITHUB_PATH` and `PATH`,
  then runs the unchanged
  `cargo binstall ... cargo-about`.
  Checked here (`~/temp/agent/test-music-player-binstall-step.ts`,
  the new lines extracted from the workflow and run with a scratch
  `RUNNER_TEMP` and `GITHUB_PATH` against the real lock):
  url guard,
  download,
  checksum check,
  extraction and the `GITHUB_PATH`
  write succeeded;
  the extracted `cargo-binstall -V` printed `1.26.2` and its archive sha256 equals the lock's.
  Not run:
  the workflow on a GitHub runner (the user chose to skip the full multi-platform dry run;
  there is no notices-only trigger),
  and the rustup and cargo-about parts of the step.
  The next music-player dry run or release is the step's first runner test.
  A push of the edit does not start the workflow:
  its push trigger watches only the two version `Cargo.toml` files.

- 2026-11-01,
  user decision about the toolchain update:
  `monochromatic-7a` had said the `toolchain:update` after its cli-git release
  moves every floating tool and needs a user decision each time;
  the user answered "I decide?
  No,
  just do it."
  So the agent runs `toolchain:update` itself once the policy release is on crates.io,
  reviews the `toolchain.lock.json` diff
  (the repository's review surface for it),
  and then lands the CI workflow;
  the user is not asked for that step.
  7a was told,
  and asked to message the released version.
  State when this was recorded:
  crates.io newest `monochromatic-git-policy-cli`
  is 0.4.6 (the lock's version);
  `package/git-policy/cli` has no catalog-specifiers code;
  the waiter `~/temp/agent/wait-cli-git-release.ts`
  ends when crates.io lists a newer version (four-hour deadline).

- 2026-11-01,
  `monochromatic-7a` status (a peer's report):
  wrapper 0.4.7 (30-second default push deadline,
  macOS signal fixes,
  mutation-test additions) is being released and is not the catalog-specifiers release;
  7a runs the lock update for 0.4.7 itself and
  it lands before mine.
  `mono/catalog-specifiers` is being built against `monochromatic-catalog-specifier` 0.1.0 on a separate,
  unmerged branch
  (`off_unless_listed`,
  old configs still read,
  a direct `--policy` on an off policy exits 2,
  not listed in `cli-git.config.jsonc`).
  7a will message the released version and release commit when the wrapper carrying the policy is on crates.io with its archives.
  Consequence for the waiter `~/temp/agent/wait-cli-git-release.ts`:
  it fires on any version above 0.4.6,
  so it will fire on 0.4.7,
  which is not the signal wanted;
  the agent restarts a waiter that ignores 0.4.7,
  and acts on 7a's message rather than on the waiter.

- 2026-11-01,
  waiter result:
  the four-hour waiter ended at its deadline with crates.io still at 0.4.6,
  so not even the 0.4.7 wrapper
  had been published.
  A waiter that ignores a wrapper release without the policy replaced it:
  `~/temp/agent/wait-cli-git-policy-release.ts` ends only when a version above 0.4.6 is on crates.io and its source archive
  contains the text `mono/catalog-specifiers` (eight-hour deadline).
  The last twelve releases of the crate came 25 minutes to about
  24.5 hours apart (measured from crates.io),
  so no completion time can be forecast;
  7a has given none.

- 2026-11-01,
  `monochromatic-8c` status (a peer's report,
  now the session sending cli-git release news):
  the wrapper branch for `mono/catalog-specifiers` (against `monochromatic-catalog-specifier` 0.1.0,
  `off_unless_listed`,
  default
  severity `error`,
  no options,
  not listed in `cli-git.config.jsonc`) is being merged to main after fixing a gap (a selected policy
  that is on but not run for the command still exited 0).
  The 0.4.7 cut was abandoned;
  the policy ships in the next wrapper release with the 30-second push deadline,
  the commit-path
  stat-index and timeout fixes,
  the layered gate image,
  and,
  when they merge,
  the LFS rework,
  the removal of external hook support
  and the in-process linter.
  8c will message the version and release commit once it is on crates.io with archives.
  8c asked who owns the crate-release path;
  the answer sent:
  git history names only the author Aquaticat,
  and
  `doc/decision/releases-run-in-ci.md` records the owner's choice "Release from CI",
  with `native:release` dispatching the workflow.

- 2026-11-01,
  `monochromatic-8c` after reading all of `doc/decision/releases-run-in-ci.md` (a peer's report):
  `native:release` dispatches
  the CI workflow and `native:release:wait` follows it through `cargo-publish.yml` to crates.io and the GitHub release.
  The workflow has never run on a runner (the first release is its first run),
  and three things are unproven:
  that a runner
  compiles the crates,
  that the token may call `createCommitOnBranch` on the protected branch,
  and that GitHub signs that commit
  under `GITHUB_TOKEN`.
  8c dry-runs first,
  so the first real wrapper release may need a retry.
  The linter crate is not in `releasable-crates.json` (only forbidden-strings and the wrapper are);
  if the wrapper embeds the linter,
  the linter publishes first by the old route.
  Consequence for my wait:
  no release time can be forecast,
  and a retry may delay it.

- 2026-11-01,
  `monochromatic-8c` relaying `monochromatic-6e` (file-enforcer and toolchain verbs),
  a peer's advice that I checked against the repo:
  scope the update to the wrapper's row,
  `mise run toolchain:update -- cargo:monochromatic-git-policy-cli`,
  because the task appends its
  arguments to `mpm-shadow update` (read in `mise.toml`,
  task `toolchain:update`,
  `usage ... var=#true`) and an unscoped run moves every
  `@latest` and channel tool (rule `UPN` names the tool).
  The row id exists in `toolchain.plan.jsonc` (`request`
  `cargo:monochromatic-git-policy-cli@latest`,
  route `cargo-binstall`).
  Adopted:
  this narrows the earlier plan of an unscoped `toolchain:update`;
  the user's delegation ("I decide?
  No,
  just do it.")
  still covers running it.
  Steps:
  the scoped update,
  `mise run toolchain:install`,
  review the `toolchain.lock.json` diff,
  commit only
  that file by explicit pathspec,
  and report separately any other row the diff shows.
  Also from 6e:
  the CI release workflow has a known bug (the bump commit is refused when main moves during the run,
  seen on
  forbidden-strings run 41303293802;
  nothing pushed) and a fix is in progress,
  so the wrapper release waits for it.

- 2026-11-01,
  `monochromatic-8c` result report (a peer's report;
  the policy is merged to main,
  not released):
  id
  `mono/catalog-specifiers`,
  default severity `error`,
  no options,
  `off_unless_listed: true`,
  not listed in this repository's
  `cli-git.config.jsonc`,
  older configs load unchanged.
  It reads staged or index bytes (never the worktree) of every `package.json`;
  the allowed specifiers match the agreed policy;
  tracked `package.json5`,
  `package.yaml` and `package.yml` are reported by file name without being read.
  JSONL codes are `mono/catalog-specifiers/<code>`;
  the six codes it lists equal the six `as_str()` values in
  `package/catalog-specifier/core-rust/src/finding.rs` lines 96 to 101 (checked),
  so nothing differs from the specification.
  It runs at commit and add,
  after a landed commit,
  on a manual push and for `check`,
  never for `fix`;
  `--no-enforce-mono/catalog-specifiers` skips it.
  A direct `git cli-git check|fix --policy <id>` that would run nothing now exits 2
  with a `config-invalid` event.
  It depends on `monochromatic-catalog-specifier` `=0.1.0` by path plus exact pin.
  The release waits for the LFS and hook-removal branches and the CI release fix.
  Correction (measured,
  then confirmed by 8c):
  the claim above that the policy never reads the worktree is wrong for direct
  commands.
  In a throwaway repository a clean staged `package.json` plus a violating unstaged one made
  `check --all --policy mono/catalog-specifiers` exit 1 (`~/temp/agent/cs-policy-050-probe.ts`,
  case `violation-unstaged-only`).
  8c confirmed:
  `check` and `fix` read the worktree files their scope selects;
  the commit and push paths check the index or
  commit content (`package/git-policy/cli/src/native/invocation/management.rs`).
  The CI checkout has index equal to worktree,
  so the
  workflow is unaffected.

- 2026-11-01,
  cli-git 0.5.0 landed in the lock and the CI workflow with the policy activation landed (task 3 done):
  - `monochromatic-git-policy-cli` 0.5.0 is on crates.io (2026-11-01T13:21:46Z,
    not yanked) with release commit `069e45a200` (8c).
    `mise run toolchain:update -- cargo:monochromatic-git-policy-cli` moved exactly one row (0.4.6 to 0.5.0,
    plus the file digest),
    committed alone as `efba845e0b`;
    `mise run toolchain:install` reported 1 installed,
    63 unchanged,
    0 failed,
    and the installed
    binary prints 0.5.0.
  - Probes with the installed binary in throwaway repositories (`~/temp/agent/cs-policy-050-probe.ts`):
    a clean staged manifest
    exits 0;
    a staged violation,
    a nested staged violation and a staged `package.yaml` each exit 1 with the expected codes;
    an unlisted policy exits 2 with `config-invalid`.
  - `.github/workflows/catalog-specifiers.yml` runs on pull requests,
    merge groups and pushes to `main` with `cancel-in-progress`.
    It installs cargo-binstall from the lock row (sha256-checked),
    installs the lock's cli-git version with
    `--disable-strategies compile`,
    runs `cli-git-native cli-git check --all --policy mono/catalog-specifiers`,
    then a negative
    control in a throwaway repository that must exit exactly 1 with the `disallowed-specifier` finding.
  - Evidence for the workflow:
    its three `run:` blocks were extracted and executed in a mount-free podman build on an image with
    no Rust toolchain (`~/temp/agent/cs-workflow-test.ts`,
    log `cs-workflow-test/build.log`):
    the sha256 check printed OK,
    the cli-git
    install took 1.99 s,
    the check exited 0 and the negative control was rejected as expected.
    `zizmor` v1.31.1 reports no findings;
    its one suppressed finding is the pedantic `anonymous-definition` (job `check` has no `name:`),
    the same as sibling workflows.
    First GitHub run:
    run 41311571968 on `d0d1a8bc35` (push,
    created 2026-11-01T13:30:21Z) completed with conclusion success.
  - Before activation:
    257 manifests scanned,
    0 parse failures,
    0 violations,
    and no tracked `package.json5`,
    `package.yaml` or
    `package.yml`.
    With the policy listed,
    the real repository check exits 0.
  - Activation:
    `cli-git.config.jsonc` lists `"mono/catalog-specifiers": "error"`.
    The workflow fails with exit 2 while the policy is
    unlisted,
    so the workflow and the listing landed together in one commit,
    `d0d1a8bc35`,
    which already passed the policy at
    commit time and is on `origin/main`.
    The tree already satisfied the policy,
    so the commit-time and CI runs had no violation to find.
  - Observed with 0.5.0:
    external hooks are switched off,
    and the commit printed the `external-hooks-unsupported` warning for the
    configured `hook.standards-review-shadow` (event `post-commit`).
    That hook produced the "Running as unit:
    standards-review-shadow-..." lines after earlier commits;
    it no longer runs.
    This is outside the catalog-specifier work and is
    for the user and the hook's owner to decide.

- 2026-11-01,
  `pnpm-shadow` built (task 4):
  crate `monochromatic-pnpm-shadow` 0.1.0 at `package/pnpm-shadow/cli`,
  commits
  `3637230e60` (sources),
  `32ba0bbd2e` (tests),
  `31b9f343bc` (fix,
  generated dependency table,
  lockfile),
  `82c2a88ba5` (parallel walk,
  README).
  - Design as agreed:
    Rust with clap,
    Unix only (the build fails elsewhere on purpose),
    `ignore` crate walk,
    no YAML (only the
    presence of `pnpm-workspace.yaml` is tested),
    no skip switch,
    warns on every invocation,
    advisory report on standard error
    then delegation to the real pnpm,
    never blocks.
    Started as `pnpm` it is the shadow;
    under any other name clap answers only
    `--help` and `--version`.
    The rule is the published `monochromatic-catalog-specifier` 0.1.0,
    so the shadow,
    the cli-git
    policy and CI cannot disagree.
    Exit codes:
    pnpm's own,
    `127` no real pnpm,
    `126` could not start it.
  - Verification:
    `cargo clippy --release --all-targets -- --deny warnings` clean;
    `mise run //package/pnpm-shadow/cli:test`
    29 tests run,
    29 passed (unit tests for the PATH lookup,
    the scan,
    the report and the launch,
    plus integration tests of the
    built binary against a fake pnpm:
    warning on stderr with pnpm's own stdout and status,
    silence on a clean workspace and
    outside any workspace,
    the same warning on every run,
    `127` with no real pnpm,
    own-name `--help` and `--version`).
  - A defect the tests found:
    clap's derive turned the struct's doc comment into `--help` text and its long form beat the
    explicit `about`;
    fixed with `long_about = None` (`31b9f343bc`).
  - Measured on this repository (22,508 walked entries,
    257 `package.json`,
    0 findings,
    silent):
    serial walk,
    whole call:
    about 95 ms to 114 ms mean (two identical shadow arms differed by 19 ms,
    the noise floor),
    against
    2.1 ms to 2.7 ms for the fake pnpm alone.
    The split (`~/temp/agent/pnpm-shadow-split`):
    the walk alone took 98 ms to 138 ms
    and the full scan 126 ms to 137 ms,
    so the walk is most of the cost and reading and checking the 257 manifests adds about
    15 ms to 30 ms. Serial against parallel walk alternating 30 rounds on the same tree and filters:
    serial median 78.9 ms
    (min 72.0,
    max 91.6),
    parallel median 10.0 ms (min 7.9,
    max 14.8),
    ranges not overlapping.
    The parallel walk was adopted.
    Whole call after the change (`~/temp/agent/pnpm-shadow-bench.ts`,
    30 runs per arm,
    hyperfine):
    14.3 ms and 14.5 ms in the two
    identical shadow arms against 2.3 ms and 2.4 ms for the fake pnpm alone,
    so the shadow adds about 12 ms to a trivial call.
    Those arms ran one after another,
    not interleaved,
    on a host other sessions also load.
  - Not done,
    by design of the agreed scope:
    the crate is not published,
    not in the crate-release registry,
    not linked as
    `pnpm` by the toolchain plan,
    and no mutation run was made (the user said mutation runs do not block publish and release).

- 2026-11-01,
  user answers on the two `pnpm-shadow` follow-ups (asked with the question tool;
  the recommended option was "not yet"
  for both and the user chose against it):
  "Publish it now",
  and "Yes,
  ask the toolchain session to link it".
  So:
  publish `monochromatic-pnpm-shadow` 0.1.0 (the first publication needs the owner's personal token,
  which the user logs in
  themselves and never sends in chat;
  then Trusted Publishing is created by API and CI takes over,
  as for
  `monochromatic-catalog-specifier`),
  register it for CI releases,
  and ask the toolchain session (the verbs' owner) to link
  the binary as `pnpm` through the toolchain plan once it is published.

- 2026-11-01,
  user directions on `pnpm-shadow`,
  three messages in order,
  each a direct user instruction:
  1.  "only say it's the wrapper on version and help commands."
      Done:
      the warning lines carry the prefix `warning: ` and never name
      the wrapper;
      the wrapper names itself only after a lone `--version`/`-v` (one line on standard output) and a lone
      `--help`/`-h`/`help` (a section),
      as `mpm-shadow` does;
      its two own failures (no real pnpm,
      could not start it) keep the
      `pnpm-shadow:` prefix because they report the wrapper's own fault.
      That exception is mine,
      not the user's:
      veto open.
  2.  "The wrapper also shouldn't take 12ms for trival pnpm commands where the wrapper doesn't even do work,
      because pnpm itself is
      rust tool.
      Profile and optimize until we don't add more than what pnpm itself adds for them.
      If you think clap is the problem,
      provide evidence and tell me what to use instead."
  3.  "still scans" - didn't we already determine to only do scans on install-class commands?,
      then "for wrapper warning scope" -
      only when the warning would make sense.
  The record before these:
     the user chose "Warn only,
     never block" against my ranking of install-class first,
     line "Whether the
  warning appears on every invocation or only on install-class verbs is not decided",
     then in the last design round "every
  invocation,
     except arguments that are only `--version`,
     `-v`,
     `--help` or `-h`".
     My build had omitted that exception,
     so it
  scanned on `--version` (my defect,
     reported to the user).
     The user's third message supersedes the recorded scope:
     the scan and
  warning happen only for install-class commands,
     and the user's own rule for the rest is that the wrapper does no work.
  - Implemented in `package/pnpm-shadow/cli/src/verb.rs`:
    the command word is the first word that is not an option or an option's
    value (value options read from `pnpm --help` of pnpm 12.16.1),
    looking through one `recursive` word.
    Warning commands,
    with
    aliases:
    `add`,
    `install`/`i`,
    `install-test`/`it`,
    `update`/`up`/`upgrade`,
    `remove`/`rm`/`uninstall`/`un`/`uni`,
    `dedupe`,
    `import`,
    `link`/`ln`,
    `unlink`/`dislink`,
    `ci`/`clean-install`/`ic`/`install-clean`.
    The list is my reading of "makes sense"
    (commands that resolve or change dependencies from `package.json`):
    `fetch`,
    `prune`,
    `deploy` and read-only commands are
    outside it;
    veto open.
    A help or version word before `--` cancels the warning;
    an unrecognised command line gives no warning.
  - For every command line except a lone version or help word the shadow now `exec`s pnpm (no resident wrapper;
    exit status and
    signals are pnpm's own).
    A test pins that a SIGTERM that kills the fake pnpm reaches the caller as that signal.
  - Evidence on clap (profile,
    `~/temp/agent/pnpm-shadow-profile.ts`,
    60 runs per arm,
    hyperfine):
    `pnpm-shadow --version` under its
    own name (the only path that runs clap) took 1.8 ms to 1.9 ms against 0.5 ms for `/usr/bin/true`,
    so clap costs about 1.3 ms
    there.
    In shadow mode `main.rs` returns before clap runs,
    and the shadow outside any workspace (no scan) took 8.5 ms against
    7.5 ms for the real `pnpm --version` in the quiet round,
    a difference inside that round's 1.5 ms to 1.7 ms spread.
    The 12 ms
    was the scan of 22,508 entries,
    not clap;
    no replacement for clap is proposed.
  - Result after the change,
    interleaved pairs (`~/temp/agent/pnpm-shadow-pairs.ts`,
    100 pairs per command,
    real pnpm and shadow back to
    back,
    order alternating,
    paired-difference median):
    `pnpm --version` +0.85 ms over 6.6 ms;
    `pnpm root` +0.59 ms over 11.8 ms;
    `pnpm install --help` +0.78 ms over 6.6 ms;
    `pnpm list` +1.32 ms over 143.6 ms with a p10 to p90 of -14.0 to +22.8 ms,
    so
    noise.
    The first profile's sequential arms were noisy (sigma up to 27 ms) because other sessions load the host;
    the paired
    method was chosen because that load moves both members of a pair together.
    Not measured:
    the wrapper in front of
    `pnpm install` itself (it would change real state);
    an install-class command pays the walk,
    about 10 ms.
  - Checks:
    clippy clean;
    54 tests pass,
    among them the verb rules (every listed verb,
    every value option,
    `--`,
    help anywhere),
    which commands warn and which stay silent in a workspace with a violation,
    and the signal pass-through.

- 2026-11-01,
  `monochromatic-pnpm-shadow` 0.1.0 published (the user answered "Publish it now",
  logged in with their own token
  through the terminal,
  then answered "Token is logged in,
  publish now"):
  - `mise run //package/pnpm-shadow/cli:publish:dry-run` passed first (20 files,
    111.5 KiB,
    verified by compile),
    then
    `:publish` uploaded;
    the crates.io API lists `0.1.0`,
    not yanked.
    cargo warned that `tests/shim.rs` is not in the package
    because `include` omits `tests/`;
    `mpm-shadow` has the same `include` and the same warning.
  - Trusted Publishing was created by API with the crate's token read in-process and never printed
    (`~/temp/agent/pnpm-shadow-trusted-publishing.ts`):
    config id 30204,
    owner `Aquaticats` (id 343218767),
    repository
    `Monochromatic`,
    workflow `cargo-publish.yml`,
    no environment;
    the list after showed one entry.
  - `.github/workflows/cargo-publish.yml` gained five jobs `ps-detect`,
    `ps-publish-crate`,
    `ps-release-state`,
    `ps-build-binary`,
    `ps-create-release`,
    the dispatch option `monochromatic-pnpm-shadow` and the push path `package/pnpm-shadow/cli/Cargo.toml`.
    They were generated from the `ms-*` jobs by a checked rename (`~/temp/agent/wire-pnpm-shadow-publish.ts`),
    and
    `~/temp/agent/check-publish-workflow.ts` parsed the YAML and showed each `ps-*` job equals its `ms-*` sibling after the rename and
    that every `needs` names a real job.
    `zizmor` reports only `artipacked` mediums,
    the same finding every checkout of this file
    already has.
    The crate is not in `package/crate-release/cli/releasable-crates.json`:
    `doc/decision/releases-run-in-ci.md`
    says only crates with release tasks are listed and the others release by a version bump pushed to `main`.
  - Dry run of the workflow:
    run 41315224517 (dispatch `crate=monochromatic-pnpm-shadow`,
    `dry-run=true`);
    it builds both Linux
    targets,
    smoke-runs the binary,
    verifies the archive layout and uploads nothing to crates.io or GitHub.
    The real dispatch
    (`dry-run=false`) follows a green dry run,
    because a GitHub release is immutable here.
  - Still the user's:
    revoke the token on crates.io and tell the agent,
    who then runs `cargo logout` (the credential file holds
    the token until then).
  - Still to do after the release:
    ask the toolchain session to add a `cargo:monochromatic-pnpm-shadow` row (route
    `cargo-binstall`,
    bin `pnpm-shadow`) and link the binary as `pnpm` (the user's answer:
    "Yes,
    ask the toolchain session to link it").

- 2026-11-01,
  `monochromatic-8c` reported `monochromatic-git-policy-cli` 0.5.1 published (crates.io 2026-11-01T14:19:49Z,
  both Linux
  assets,
  release commit `b0d0b14a01`,
  publish run 41315248565) and said the lock bump is mine.
  Declined for now and told 8c:
  the
  user's delegation covered the bump for the release carrying the policy (0.5.0,
  done in `efba845e0b`);
  0.5.1 is not needed by this
  work and changes the wrapper every session commits through.
  Behavior changes 8c lists since 0.5.0:
  embedded Markdown linter,
  `git-lfs push --all <destination> <refs>`,
  aliases cannot re-enable hooks,
  `core.fsmonitor` left as configured,
  commits go through
  the commit service by default once `git cli-git service install` was run (`CLI_GIT_COMMIT_SERVICE=0` keeps the native path).
  To put to the user:
  whether to run `mise run toolchain:update -- cargo:monochromatic-git-policy-cli` and commit only the lock.
- 2026-11-01,
  token cleanup:
  the user said the crates.io token is revoked;
  `cargo logout` printed that the token was removed from
  local storage,
  and `${CARGO_HOME:-$HOME/.cargo}/credentials.toml` went from 57 to 11 bytes (the length of a bare `[registry]` line;
  its content was not read,
  a read of it was denied by the auto-mode classifier).
  The server-side revoke is the user's step and
  was not checked from here.

- 2026-11-01,
  `monochromatic-pnpm-shadow` 0.1.0 released and verified as a consumer:
  - Dry run 41315224517 passed (both targets built and smoke-run),
    then the real run 41315727800 passed:
    `ps-publish-crate`
    authenticated through Trusted Publishing with no token and skipped the existing upload,
    both `ps-build-binary` targets passed and
    `ps-create-release` created the release `monochromatic-pnpm-shadow-v0.1.0` (immutable) with the x86_64 and aarch64 Linux
    archives (854,983 and 774,034 bytes) and a sha256sums file.
  - `cargo-binstall --no-confirm --disable-strategies compile --root <throwaway> --version 0.1.0 monochromatic-pnpm-shadow` installed
    `pnpm-shadow` from the archive in 4.6 s and `pnpm-shadow --version` printed `pnpm-shadow 0.1.0`.
  - Asked `monochromatic-6e` (the toolchain session,
    the owner of the plan and the link mechanism) to add and link the shadow as
    `pnpm`,
    with the facts above and the one open question:
    the real pnpm is already exposed as `.cache/toolchain-bin/pnpm`,
    so the link
    named `pnpm` must come first on `PATH` and the real pnpm must stay found after it.
    No reply yet.
- 2026-11-01,
  user answers to three questions (question tool):
  bump the lock to cli-git 0.5.1 now ("Yes,
  bump to 0.5.1 now",
  against the recommended "not now");
  keep the `pnpm-shadow:` prefix on the wrapper's two failure messages;
  keep the warning
  command set as listed.
  Peer reports after that:
  `monochromatic-8c` ran the scoped `toolchain:update` for 0.5.1 under what it called
  the user's standing authorization,
  and `monochromatic-6e` staged the plan row `cargo:monochromatic-pnpm-shadow` (`cargo-binstall`,
  bin `pnpm-shadow`,
  `os: linux`,
  a postinstall link of `pnpm-shadow` as `pnpm` into `.cache/pnpm-shadow-bin`) and an edit of
  `.config/mise/conf.d/20-env.toml` putting that directory ahead of `.cache/toolchain-bin`,
  and locked `0.1.0`;
  6e holds the install and
  the commit until 8c commits,
  then checks `command -v pnpm`,
  `pnpm --version` and `pnpm root` and reports.
  Checked from here with
  `git diff` of `toolchain.lock.json`:
  three changes only (the 0.5.0 to 0.5.1 version,
  the new shadow row,
  the digest).
  Not
  committed by this session on purpose:
  the file holds both sessions' uncommitted work.

- 2026-11-01,
  defect in `monochromatic-pnpm-shadow` 0.1.0 found by the toolchain session (`monochromatic-6e`) and fixed in the tree:
  - What 6e measured:
    after `toolchain:install` linked the shadow as `pnpm` through a launcher (an ELF that `mpm-shadow` writes for
    every postinstall link),
    `pnpm --version` exited 126 with "pnpm-shadow:
    could not start .../pnpm-shadow-bin/pnpm:
    Res...",
    and
    `pnpm root` and `pnpm --help` ran past a 60-second timeout.
    6e disabled the link and took the PATH entry out;
    the plan row and the
    lock row 0.1.0 stay.
  - Cause:
    the shadow skips PATH candidates whose resolved path equals its own executable path.
    The link resolves to the
    launcher,
    a different file,
    so the shadow took its own link as the real pnpm and started itself again:
    an endless exec chain,
    and
    on the version and help path (a child process) nested children until the process limit.
    My tests had linked the binary directly,
    never through a launcher.
  - Fix,
    commits `fix(pnpm-shadow/cli)` (loop guard) and `fix(pnpm-shadow/cli)` (notice handshake),
    README section "Launchers and the
    loop guard":
    each instance records the resolved path of the candidate it picks in the environment value `PNPM_SHADOW_GUARD`,
    and
    the instance that candidate starts skips every recorded path,
    so the chain ends at the real pnpm;
    hard limit of eight hops.
    The value
    is bound to a process (an exec guard to the process with the writer's id,
    a spawn guard to the writer's direct child),
    so a script
    that the real pnpm runs never inherits it;
    an instance started by another does not warn again;
    for version and help the
    innermost instance prints the notice and leaves a marker file named for the outer instance's process id in `XDG_RUNTIME_DIR`,
    which the outer instance claims and stays silent (without that variable a launcher chain prints the notice twice).
    6e suggested skipping every candidate in the directory the shadow was found in;
    that needs the invoked path,
    and argv[0] is the
    bare word `pnpm` when the shell finds the link on PATH,
    so the guard records resolved candidates instead.
  - Tests (68 pass,
    clippy clean):
    a launcher world (a bash script that runs the shadow under the name `pnpm`) for the exec and the
    spawn path,
    a violation warning exactly once,
    a pnpm that runs `pnpm` again still finding the real one,
    and a launcher with no real
    pnpm behind it ending with 127;
    plus unit tests of the guard text,
    the process binding,
    the skip list and the marker.
    The tests were first wrong in their harness:
    `timeout` was not found because the child `PATH` held only the shadow and fake
    directories (fixed with its absolute path).
  - Proof that the tests can fail (rule on guard tests):
    in a podman container with `--memory=2g --cpus=2 --pids-limit=200`,
    inputs
    baked in from `git archive HEAD`,
    with `is_meant_for` rewritten to always return false (`~/temp/agent/ps-guard-proof.ts`,
    log
    `ps-guard-proof/run.log`),
    both launcher tests failed:
    "the exec path looped" (exit 124,
    killed by `timeout`) and the warning
    count 9326 against the expected 1.
    Nothing ran on the host.
  - Latency after the guard,
    paired medians against real pnpm 12.16.1 (`~/temp/agent/pnpm-shadow-pairs.ts`,
    100 pairs):
    on a host that
    other sessions loaded heavily (real medians up to 184.6 ms,
    several spreads of 13 ms to 60 ms) only `install --help` was clean
    (+0.78 ms,
    p10 to p90 -1.7 to +3.3 ms,
    the same as before the guard);
    `--version` +1.48 ms and `root` -0.64 ms had spreads far above
    those values and prove nothing either way.
  - Not yet done:
    the fix is not released.
    0.1.0 on crates.io and in the GitHub release is immutable and still carries the defect,
    so
    a fixed release is a new version,
    0.1.1.
    `monochromatic-6e` asked to be told when a fixed release is out,
    then puts the PATH entry back
    and re-runs `command -v pnpm` plus `pnpm --version`,
    `root` and `--help` with timeouts.
- 2026-11-01,
  `monochromatic-8c` said pushes may fail before `git push` with a `git-lfs` batch HTTP 503 since cli-git 0.5.1 (about 22 batch
  requests per push,
  the server answers some with an intermittent 503);
  commits stay local and go out with the next successful push;
  a
  0.5.2 hotfix is being built;
  `--no-verify` and `GIT_LFS_SKIP_PUSH=1` must not be used for pushes that add LFS files.
  Seen from here:
  eight local commits ahead of `origin/main`,
  one of them mine (the README commit),
  the rest merges from other sessions.

- 2026-11-01,
  after a system reboot (the waiter on the 0.1.1 release was stopped by it):
  the user answered "Release 0.1.1 now" and the
  version bump is committed,
  `6751b15dbc` (`Cargo.toml` and `Cargo.lock` change by exactly the two version lines),
  and survived the
  reboot.
  It is not on `origin/main`:
  69 local commits,
  mostly other sessions',
  are ahead of it because pushes fail with the
  cli-git 0.5.1 regression (`git-lfs push --all` sends about 22 batch requests and the LFS server answers some with HTTP 503).
  A manual `git push origin main` at 10:12 local time failed the same way:
  "git-lfs push exited with status 2,
  so this push was not
  started and nothing was published".
  No bypass was used:
  `--no-verify` and `GIT_LFS_SKIP_PUSH=1` would publish pointers without
  objects,
  and 8c asked that they not be used for pushes that add LFS files.
  What clears it:
  the cli-git 0.5.2 hotfix that 8c is building (bounded upload,
  retry on 5xx) and its lock bump,
  or a push that
  happens to succeed.
  When any push lands,
  `6751b15dbc` goes with it,
  the path filter of `cargo-publish.yml` starts the release
  (`ps-detect` sees 0.1.0 to 0.1.1),
  and Trusted Publishing publishes and creates the release `monochromatic-pnpm-shadow-v0.1.1`.
  The waiter `~/temp/agent/wait-shadow-release-011.ts` (90-minute deadline) was restarted and ends at the full result.
  Then:
  tell the toolchain session,
  which asked to be told when a fixed release is out so it can put the PATH entry back and
  re-run `command -v pnpm`,
  `pnpm --version`,
  `pnpm root` and `pnpm --help` with timeouts,
  and then install 0.1.1 from the lock
  (its `toolchain:update` row).
  After the reboot `ListAgents` showed no `monochromatic-6e` or `monochromatic-8c`;
  the peers were
  `monochromatic-2a` and `monochromatic-6c`,
  and which of them owns the toolchain was not known,
  so no status was sent;
  a message to
  the stale socket of 6e failed with ENOENT.
  Ask the user,
  or read `ListAgents` again,
  for the current owner before sending.

- 2026-11-01,
  `monochromatic-pnpm-shadow` 0.1.1 released (the waiter ended with the full result):
  `6751b15dbc` reached `origin/main`
  after the git-lfs 503 cleared,
  crates.io lists 0.1.1,
  and the release `monochromatic-pnpm-shadow-v0.1.1` has the x86_64 and
  aarch64 Linux archives and a sha256sums file;
  the latest push-triggered `cargo-publish.yml` run (41320323471,
  headed by
  `11ad7efdf7`) succeeded.
  - Consumer check on the published archive:
    `cargo-binstall --no-confirm --disable-strategies compile --version 0.1.1` installed
    `pnpm-shadow 0.1.1` in 3.96 s.
    Behind a launcher with a fake pnpm after it (`~/temp/agent/ps-published-launcher-check.ts`,
    each
    run under `timeout 30`):
    `install` exit 0 with one warning block;
    `--version` exit 0 with pnpm's line and one notice naming the
    fake pnpm;
    `root` exit 0 with no warning;
    `nested` (a pnpm that runs `pnpm` again) reached the real pnpm.
    No loop.
  - Told the two peers listed after the reboot (`monochromatic-2a`,
    `monochromatic-6c`),
    each message addressed to whichever of them
    owns the toolchain,
    because which one does was not known.
    Still the toolchain owner's:
    lock 0.1.1,
    put `.cache/pnpm-shadow-bin`
    back ahead of `.cache/toolchain-bin`,
    and check `command -v pnpm`,
    `pnpm --version`,
    `root` and `--help` with timeouts.

## Open items

- Done:
  the lock bump to 0.5.0,
  the CI workflow,
  the policy activation and its first green run (run 41311571968).
- Done:
  `pnpm-shadow` 0.1.1 built,
  tested,
  published,
  released with archives,
  installed from the archive as a consumer,
  and shown
  not to loop behind a launcher (the 0.1.0 defect),
  on the published binary and on the test build.
- Done,
  by `monochromatic-2a` (the toolchain session;
  it answered the conditional message):
  locked `cargo:monochromatic-pnpm-shadow`
  0.1.1 (lock diff:
  that row and the digest),
  `toolchain:install` reported "1 installed,
  64 unchanged,
  0 failed",
  checked a throwaway PATH
  first,
  then `.config/mise/conf.d/20-env.toml` put `.cache/pnpm-shadow-bin` ahead of `.cache/toolchain-bin`;
  its commit was in flight
  when it reported.
  Checked from here in the repository's mise environment (`mise exec`,
  each command under `timeout 30`):
  `command -v pnpm` is `.cache/pnpm-shadow-bin/pnpm`;
  `pnpm --version` exit 0 with `12.16.1` and one `pnpm-shadow 0.1.1, shadowing pnpm at
  .cache/toolchain-bin/pnpm` line;
  `pnpm root` exit 0,
  no shadow line;
  `pnpm --help` exit 0 with pnpm's help then the shadow section.
- Done by the user's answer ("Yes,
  bump to 0.5.1 now"):
  read with `git show HEAD:toolchain.lock.json | jq` after the reboot,
  the
  committed lock names cli-git 0.5.1 and the `cargo:monochromatic-pnpm-shadow` row at 0.1.0 (so 0.1.1 is not locked yet).
- Push state after the reboot:
  the handover commit `07c02186b1` was saved locally and its auto-push failed with the git-lfs 503
  ("commit saved locally,
  but auto-push to origin failed");
  the earlier release commit had already gone out with a push that worked.
- For the user:
  the `hook.standards-review-shadow` post-commit hook no longer runs under cli-git 0.5.0 and later.
- Measured on 2026-11-01 with read-only `gh api`:
  `main` protection has `required_status_checks` with `strict: true` and empty `checks` and
  `contexts`,
  `required_signatures: true`,
  `enforce_admins: false` and no required pull request reviews;
  the one active ruleset
  (`Copilot review active branch`,
  id 9126851) has the single rule type `copilot_code_review`.
  So no status check is required and a failing
  `catalog-specifiers.yml` run blocks nothing;
  this is the detection-only guarantee the user chose ("Fail the job on a violation").
  Making it required would be the owner's setting;
  nothing in the repository's direct pushes to `main` would satisfy a required check.
- Settled by the user:
  the two `pnpm-shadow` failure messages keep the `pnpm-shadow:` prefix,
  and the warning-command list stays as built.
- Mutation run gaps never re-run:
  the `package.yml` mutation failed by compile error,
  and three mutations were skipped because their
  pattern matched twice.
- Not measured:
  the effect of `--policy` on a shipped policy that is absent,
  `off` or `warn`;
  whether branch protection exists;
  whether
  the crates.io token is revoked (the user said so;
  not checkable from here);
  the wrapper in front of `pnpm install` itself.

## Next action

Nothing is queued.
The whole design is built and live:
the `zod` fix,
the library crate,
the cli-git policy at `error`,
the CI
workflow,
and the `pnpm` shadow in front of the real pnpm.
Remaining are only items for the user,
in Open items (the hook that no longer
runs,
token revocation not checkable from here;
branch protection was measured,
see Open items).
