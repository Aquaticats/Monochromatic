# Catalog specifier policy

## Purpose

`pnpm-workspace.yaml` sets `catalogMode: strict`, yet manifests with literal versions install silently.
This handover records the measured cause, the decisions reached in the grilling session, and the next action.
No code has been changed yet; the user has not confirmed the shared understanding.

## Measured facts

- pnpm documents `catalogMode` only for `pnpm add` and `pnpm update`.
  In a throwaway fixture on pnpm 12.16.1, `pnpm install` accepted `"is-even": "1.0.0"` and `"is-number": "^7.0.0"` with exit 0 and no warning.
  The positive control `pnpm add is-odd@1.0.0` (outside the catalog range `>=3.0.0`) failed with `ERR_PNPM_CATALOG_VERSION_MISMATCH`.
- The pnpm source consults the catalog mode only on add and update paths.
  No pnpm setting or subcommand enforces catalog use at install time.
- A `readPackage` hook in `.pnpmfile.mjs` receives the root and every workspace manifest on pnpm 12.16.1,
  with raw specifiers in `dependencies`, `devDependencies` and `peerDependencies`.
  The comment in `.pnpmfile.mjs` saying direct workspace dependencies are not read via `readPackage` is stale and must be corrected.
- The hook can import a `.ts` file (`./check.ts` was imported and called in a fixture).
- The hook runs on frozen installs, in CI, and on any install after a manifest edit.
  `readPackage` does not run on a plain no-op `pnpm install` that prints "Already up to date" (`optimisticRepeatInstall`).
  Correction after the reboot: the `updateConfig` hook does run there, and on every other verb; see "Research after the reboot".
- Workspace manifests and `file:` dependency manifests both reach the hook without a `dist` key,
  so a `dist` filter would police `file:` dependencies; the hook must use a workspace-name set built from `package/*/*`.
  (Moot under an `updateConfig` hook, which never sees dependency manifests.)
- `syncpack` 15.3.3 can express the policy only with workarounds and passes named catalogs.
  `eslint-plugin-pnpm` 1.9.1 leaks `workspace:*.0` and peers starting with `*`, defaults to a destructive autofix, and needs ESLint beside oxlint.
  Both were run in CachyOS containers; neither is adopted.
- The history scan found 260 commits that touched exact-version lines, covering 37 distinct dependencies.
  Today the repo has 248 tracked manifests and exactly one violation: `zod` at `^4.6.7` in `package/dsh-plugin/cost-dismissal/package.json`.
- The repo has no `catalogs:` section and no named-catalog references; 208 manifests use plain `catalog:`.

## Decisions

- Policy: `dependencies`, `devDependencies` and `optionalDependencies` accept only `catalog:`, `catalog:<name>` or exactly `workspace:*`.
  `peerDependencies` accept only `*`, `catalog:` or `catalog:<name>`.
  Any other specifier fails, including digits, ranges, prereleases, `workspace:^`, `link:`, `file:`, git and URL forms.
- No exception list; a dependency that needs a version belongs in the catalog.
- Catalog entries themselves are not validated by this check.
  Re-confirmed after the review: manifests only, even though `pnpm add --workspace` promotes a workspace package
  into the catalog as `workspace:^1.0.0` (measured).
- The lint checks reference syntax only: it does not verify that a `catalog:` or `catalog:<name>` reference has an entry.
  pnpm rejects a missing entry for installed packages, but nothing checks tracked manifests outside the workspace.
  This is documented as a known limit, by the user's choice after the review.
- Named catalogs are allowed; pnpm itself rejects a reference to a missing entry.
- Mechanism: one shared check, run by a mise lint task and by the `.pnpmfile.mjs` hook.
- Package home (the hook-facing validator must stay dependency-free, see the probes below):
  new `package/dependency-catalog/core-specifier-policy` (pure validator and manifest enumeration)
  and a thin `package/dependency-catalog/cli-specifier-policy` for the mise task.
  `core-pnpm` stays parsing-only.
- Lint scope: every tracked `package.json`, including the root and `package-paused/`.
  The hook covers only workspace manifests, identified by a workspace-name set.
  Superseded by "Research after the reboot": an `updateConfig` hook reads manifests from disk and enumerates the workspace
  from `config.workspacePackagePatterns`, so no name set and no `dist` classifier is needed; the root manifest is covered too.
- The check reports every violation in one run, naming the file, the section and the dependency.
- `zod` moves into the catalog as `>=4.6.7` and `cost-dismissal` switches from `^4.6.7` to `catalog:`.
  An intermediate edit of this handover wrongly said `zod` was already in the catalog;
  the `'>=4.6.6'` hit at `pnpm-workspace.yaml` line 461 is the CVE floor in the `overrides:` block, not a catalog entry.
  Measured: `pnpm-lock.yaml` records the `cost-dismissal` specifier as `'>=4.6.6'` (the override) resolving 4.6.7,
  so the override already governs and the catalog range changes no effective resolution.

## Pre-implementation probes

Run after the decisions, on pnpm 12.16.1 in a throwaway fixture, each with a positive control.

- Fresh install with a hook-imported file that imports a bare package fails for every install:
  `ERR_MODULE_NOT_FOUND: Cannot find package 'is-odd'`, exit 1, because the hook loads before `node_modules` exists.
  The hook-facing validator must therefore be one dependency-free file imported by relative path,
  with only `node:` builtins.
  The repo rules that normally require `@monochromatic-dev/logger-core` and `or-throw-core` need a documented exception in that file.
- A throwing hook does not break `pnpm add`: with `catalogMode: strict`, `pnpm add is-buffer@1.1.6` and `pnpm add is-buffer`
  both exit 0 and the hook receives the promoted `catalog:` form.
  The positive control (a manifest with `"is-number": "^7.0.0"`) exits 1 with `ERR_PNPM_PNPMFILE_FAIL` and the policy message.
- pnpm itself rejects a `catalog:` reference with no catalog entry: `ERR_PNPM_CATALOG_ENTRY_NOT_FOUND_FOR_SPEC`.
- Read from `.pnpmfile.mjs`, not probed: `applyBlocklist` rewrites `dependencies` and `optionalDependencies` in place to
  `workspace:@monochromatic-dev/pnpm-override-stub-*@*`.
  The specifier policy must therefore validate the manifest as received, before `applyBlocklist`,
  and a test must cover a workspace manifest that depends on a blocked name.

## New input after the decisions

The user plans to shadow `pnpm` with an own Rust and clap wrapper.
No document in `doc/` mentions it yet; the nearest precedent is `package/mpm-shadow/cli`,
which finds the real tool on `PATH`, runs it as a child with arguments untouched, then acts after it.
A `pnpm` shadow could run the specifier check before pnpm starts,
which would cover the no-op install that the `readPackage` hook cannot reach.
That is an inference from the shadow design, not a measurement.
Correction: the `updateConfig` hook reaches the no-op install too, so this was not a shadow-only advantage.
The user delegated the timing and reach decisions, so these are agent recommendations, open to veto.

## Independent review findings

A `pi` review (model `openai/gpt-6.1-sol`, whole files pasted) and the advisor reviewed the design after the decisions.
Verdict from `pi`: ship a source-manifest checker with CI enforcement, defer the hook and the Rust shadow.
Verified by probe or file read:

- A name-only workspace classifier misfires on a registry package that shares a workspace package's name:
  registry `is-odd@3.0.1` matched `nameOnly=true` with `dist` present, while name plus no `dist` matched only the workspace package.
  The hook must require both conditions, and the set must include the root manifest's name.
- `pnpm-workspace.yaml` sets `savePrefix: ''` and `saveWorkspaceProtocol: false` (lines 537 and 538).
  Under `catalogMode: strict`, `pnpm add @fx/b --workspace` writes `"@fx/b": "catalog:"` for both `false` and `rolling`,
  and promotes the workspace package into the catalog as `workspace:^1.0.0`, which the manifest policy allows
  but which differs from the repo's `workspace:*` convention.
- The existing hook exports `export const hooks`; the fixture used `export default { hooks }`.
  The implementation must be tested with the real export shape.
- Not probed: whether CI invokes the root `lint` task, and the `pnpmfileChecksum` effect of an edited imported validator.
- Lint checks syntax only; a `catalog:` reference to a missing entry in a tracked manifest outside the workspace
  (for example `package-paused/`) is not caught by pnpm either.
- A per-manifest hook throws on the first violating manifest, so "every violation in one run" holds for lint only.

## Shadow recommendations

- Timing: ship the TypeScript check, the hook and the lint task now; the shadow is a separate later project.
  Reason: the gap is real today, the shadow has no document or code, and the policy is language-independent.
- Port by corpus, not by code: keep the policy's test vectors (specifier, section, expected verdict)
  in one JSON file that the TypeScript tests and a later Rust nextest suite both read,
  so the shadow adopts the same rules without a second interpretation.
- Reach: the lint task stays as the CI backstop even after the shadow exists.
  Measured: `mise.toml` names `pnpm` 30 times and installs funnel through the `prepare:pnpm` task chain,
  so callers do not all share one `PATH` resolution.
- The hook is retired only after the shadow is shown to be first on `PATH` for CI, mise tasks and agent shells.
- The shadow, when built, follows the `package/mpm-shadow/cli` pattern: run the real `pnpm` as a child,
  arguments untouched, and run the check first.

## Why the Rust wrapper option was not listed in the hook-timing question

The hook-timing question (Q10) offered only TypeScript-first variants: lint then hook, both together, or lint plus a mise call.
It did not offer "build the whole policy in the Rust wrapper and drop the lint task and the hook".
That omission was a mistake of mine, not a measured exclusion:

- Earlier questions did offer the shape: the shadow-timing question had "shadow is the next thing built" and
  "standalone Rust crate", and the reach question had "the shadow is the only enforcement point", ranked last.
  The user answered both with "you suggest", delegating them, and I recorded my own recommendation as if it were settled.
  I then built the hook-timing options on that recommendation instead of re-asking, and did not widen the options
  after the review, which the repo's option-widening rule requires.
- The reasons I would have given for ranking it low are real but are not measurements:
  no document or code for the shadow exists (searched `doc/` and `package/`, found none),
  a Rust crate needs its own README, nextest tests and packaging first,
  a shadow only mediates callers that resolve it first on `PATH`,
  and a pre-flight check on every invocation needs verb, `--dir` and repair-command design (raised by the `pi` review).
- Its real advantages were also not weighed: one implementation in the language the shadow is written in,
  and a check that runs before pnpm starts, so even a no-op install is covered
  (inference from `package/mpm-shadow/cli`, not measured).

The user decides this when back; the option must be put to them with pros, cons and a ranking.
A suggested rule edit for `AGENTS.md`, not applied: after a delegated decision, treat the recommendation as open
for veto and re-offer the delegated alternative in every later option set that depends on it.

## Dependencies the design needs

Enumerated late; the repo's grilling rule asks for this first.
Nothing here is picked yet beyond the first item.

- The hook-facing validator file: `node:` builtins only (measured necessity, because the hook loads before `node_modules`).
- Superseded: the workspace-name set and the YAML question.
  `updateConfig` hands the hook `config.workspacePackagePatterns` (measured: `["pkg/*"]` in the fixture),
  so the hook expands those globs with `node:fs` and needs neither a name set nor a YAML parser.
  What remains open is how the hook expands `*`, `**` and `!` patterns, and failing closed on a pattern it cannot expand.
- The rest of `core-specifier-policy` and `cli-specifier-policy` would follow repo rules:
  `@monochromatic-dev/logger-core`, `@monochromatic-dev/or-throw-core`, `string-dedent` and the test harness.
  Open: how `git ls-files` is run (an existing `package/git-executable` may own that; not yet read).
- Rust shadow, if chosen: `clap` plus a JSON reader for manifests; no crates picked, and the catalog-policy rule for
  Rust dependencies (`doc/decision/rust-dependency-catalog.md`) governs the picks.

## State at the user's reboot

- Decided: policy, no exceptions, named catalogs allowed, lint scope (every tracked manifest),
  package home (two new packages), manifests-only, syntax-only references, zod into the catalog as `>=4.6.7`.
- Deferred by the user: hook timing, and whether the whole policy moves into the Rust wrapper.
- Nothing was implemented. Throwaway fixtures, probe scripts and the `pi` prompt and output remain in `~/temp/agent`.
- Commits made by this session trigger `standards-review-shadow-*` systemd units; they were left to finish.

## Research after the reboot

The user said the earlier research was clearly insufficient and asked for more before any further question.
Everything here was measured in throwaway fixtures under `~/temp/agent` unless marked otherwise.

### The `updateConfig` hook reaches every verb, which corrects the no-op claim

Probe `~/temp/agent/catalog-hook-matrix.ts` (fixture `hook-matrix-GFm9KN`), pnpm 12.16.1, one hook-logging `.pnpmfile.mjs`:

- `updateConfig` fired on: fresh install, no-op install, frozen install, `add`, `update`, `prune`, `run`, `exec`, `list`,
  `why`, `outdated`, `fetch`.
- It did not fire on `config get` and `--version`.
- `readPackage` fired only on fresh, frozen, `add`, `update`, `prune` and `fetch`; `preResolution` on fresh, frozen, `add`,
  `update`, `prune`; `afterAllResolved` on fresh, `add`, `update`.
- `updateConfig` receives the config object with `dir` and `workspaceDir`, and no manifests.
  A hook can still read manifests itself, because it is ordinary Node code.
- So the earlier statement that no hook reaches the no-op install was wrong for `updateConfig`.
  `doc/troubleshooting/pnpm-prune-before-install.md` already documented `updateConfig` as the earliest hook;
  I had not read it before writing the handover.

### An `updateConfig` check enforces the policy on every verb

Probe `~/temp/agent/catalog-updateconfig-probe.ts` (fixture `updateconfig-probe-6VGCn8`): an `updateConfig` hook using only
`node:fs` and `node:path` hand-parses the `packages:` key of `pnpm-workspace.yaml`, expands the `*` globs, reads each
`package.json` and throws on any policy violation.

- Clean workspace: install and no-op install exit 0.
- With `"is-number": "^7.0.0"` planted in `pkg/a`: no-op install, frozen install, `run`, `exec`, `list` and a `run` from the
  `pkg/a` subdirectory all exit 1.
  The message reaches the user: `running updateConfig hook ... Error: specifier policy: pkg/a/package.json dependencies is-number=^7.0.0`.
  It is hard-wrapped by pnpm at terminal width, so a grep for the full phrase on one line misses it.
- After repairing the manifest, the no-op install exits 0 again (the positive control moves in both directions).
- `pnpm install --ignore-pnpmfile` exits 0 with the violation present: the one bypass found.
  `config get` and `--version` are unaffected, so the repair path through pnpm itself is not blocked.
  No file in the repo uses `--ignore-pnpmfile` or `ignorePnpmfile`.
- Reach and bypass probe `~/temp/agent/catalog-updateconfig-reach.ts`, same fixture, violation planted
  (the message, not the exit code, is the discriminator; the control run without a violation printed no message):
  the check fired for `--dir`, `-C`, `--filter`, `-r`, `--workspace-root`, and `--dir <fixture>` run from an unrelated cwd.
  It was skipped for `pnpm_config_ignore_pnpmfile=true` in the environment and for `--config.ignore-pnpmfile=true`,
  as well as `--ignore-pnpmfile`; `npm_config_ignore_pnpmfile=true` did not skip it.
  `--pnpmfile ./missing.mjs` fails with exit 2 rather than skipping.
  A deliberate bypass therefore exists in three spellings (flag, `--config.` form, `pnpm_config_` environment variable),
  which is why the source lint stays as the backstop.
- Probe `~/temp/agent/catalog-updateconfig-add.ts` (same fixture, hook and manifest restored afterwards):
  - The `updateConfig` argument carries `workspacePackagePatterns` (value `["pkg/*"]` in the fixture), `packages`, `ignorePnpmfile`,
    `catalogs`, `catalogMode` and `workspaceDir`, among a long list of settings.
    The earlier probe logged only a fixed key list that omitted `workspacePackagePatterns`.
  - `updateConfig` and `readPackage` coexist in one `export const hooks` object; both ran.
  - With a violation planted, `pnpm add is-odd`, `pnpm update` and `pnpm remove is-number --filter @fx/a` all exit 1 with the
    policy message.
    The check runs before those verbs read anything, so pnpm cannot repair a violation: the repair is a hand edit of the
    manifest (or the bypass).
    This differs from the `readPackage` finding that a throwing hook leaves `pnpm add` working.
    The hook's message must therefore name the file, section, dependency and the hand-edit remedy.
- The hook cannot tell which verb is running.
  Probe `~/temp/agent/catalog-updateconfig-argv.ts` (fixture hook restored afterwards) logged `process.argv`, `process.execArgv` and
  the `npm_*` and `PNPM_*` environment keys inside `updateConfig` for `install`, `add` and `run` (the output for `update` and
  `remove` was not read): `argv` is empty after the script name, no `npm_command` or `PNPM_*` variable exists, and `execArgv` is
  pnpm's own bootstrap script, identical for every verb.
  pnpm starts the pnpmfile in a child Node process that talks to it over a JSON protocol on stdout and stdin,
  so a hook must never write to stdout, and the verb is not passed.
  Blocking is therefore all or nothing across verbs; selecting repair verbs would need a platform-specific read of the parent
  process command line, which is unmeasured and would not work on the Windows runners.
- Landing order: a throwing `updateConfig` makes every pnpm verb except `config get` and `--version` exit 1 while the
  `zod` violation exists, so the `zod` fix and the hook must land in the same commit.
- Cost: the same scan over the real repo's workspace manifests took a median of 4.9 ms over 15 in-process runs
  (minimum 4.4, maximum 9.0 on the first), and it found the one real violation, `zod` in `cost-dismissal`.
  This is the scan alone, not pnpm's own startup.
- Limits: the hand-written glob expansion supports only `*` segments; the real workspace uses `package/*/*`, which it handled.
  A `!` negation pattern or `**` in the workspace file would need handling.

### CI facts that bear on a PATH shadow

- Both CI systems install pnpm through the `task-cli-setup` action and run `task-run prepare:pnpm`
  (`.github/workflows/current-baseline.yml`, `.depot/workflows/current-baseline.yml`).
  The two `.depot/workflows` files pass `pnpm-version: '12.14.0'` explicitly, while the action reads `.tools.pnpm.version`
  from `toolchain.lock.json` (12.16.1) when the input is empty, as the `.github/workflows` files do.
  So CI runs two pnpm versions.
  A container probe (`~/temp/agent/pnpm-ci-version-probe.md`, CachyOS image, `--memory=2g --cpus=2`, no mounts, with a
  12.16.1 control that reproduced every host result) found `updateConfig` identical on 12.14.0:
  it fires on every verb except `config get` and `--version`, a throwing hook blocks no-op and frozen install, `run`, `exec`
  and `list` with the policy message, and `--ignore-pnpmfile` skips it.
  Only `preResolution` differs: it does not fire on frozen install or `prune` on 12.14.0.
  Not covered by that probe: node 24 (the image has 26.12.1) and recursive filters (covered on 12.16.1 only, above).
- The action puts node, pnpm and the task CLI on `PATH` (`.github/actions/task-cli-setup/action.yml`) and nothing about a
  shadow directory.
  The mpm shadow reaches callers because the toolchain plan links it into a gitignored directory first on `PATH`
  (`doc/decision/mpm-toolchain-adoption.md`); a pnpm shadow would need the same link, and CI would need a built Rust binary
  for each runner OS (the workflows include Windows steps).
- The Rust dependency catalog has `clap`, `serde` and `serde_json` but no YAML crate (`rust-catalog.toml`),
  so a Rust checker must hand-parse `packages:` too or add a crate.

### Upstream state (agent report `~/temp/agent/pnpm-upstream-research.md`, source reading of pnpm commit `c7b46aedef`, not run)

- Issue [8644](https://github.com/pnpm/pnpm/issues/8644) "Checks to enforce pnpm catalog protocol usage" is the direct request.
  It proposed "Or the `pnpm install` command fails if enforcement is enabled" and was closed `COMPLETED` on 2026-02-15
  with no closing PR, on the grounds that `catalogMode` "likely addresses this feature request".
  `catalogMode` does not: its code is wired to add and update only (`crates/package-manager/src/catalog_mode.rs:1`).
- Discussion [8567](https://github.com/orgs/pnpm/discussions/8567) holds an unanswered question from 2026-03-11 asking whether a
  hand-edited `package.json` should be covered; no maintainer replied.
  Install-time warnings were also suggested in 15022 (now a locked discussion); neither it nor 11742 has a maintainer reply.
  The maintainers point to `syncpack` as the external route.
  No upstream fix is on the horizon, so the repo needs its own check.
- No pnpm command or setting validates specifiers: no `pnpm catalog` or lint command exists, and `blockExoticSubdeps`,
  `trustPolicy`, `minimumReleaseAge` and `strictDepBuilds` act on resolved packages.
- The agent's source reading agrees with the probes: `updateConfig` receives settings (`dir`, `workspaceDir`,
  `workspacePackagePatterns`) and no manifests, runs before the command on almost every verb, and does not run on `dlx` or `doctor`.
  `pnpm dlx` is therefore an unchecked path, which is acceptable because it installs into a throwaway project.
- The no-op install fast path refuses to short-circuit when a pnpmfile is present, which is why `updateConfig` still runs.
- The agent's `gh search` first PR round used a quoted multi-word phrase that returns nothing, so only its rerun counts.
  It did not search `pnpm/rfcs`.
- The earlier `pi` verdict (defer the hook) was reached on my wrong premise that no hook reaches the no-op install and that the
  hook must wait for manifests through `readPackage`; the verdict should be treated as superseded on that point.

### What this changes

Reach is no longer an argument for the Rust shadow over a hook: an `updateConfig` hook covers the no-op install and every
other verb on any caller, with `--ignore-pnpmfile` as the only bypass found, while a shadow covers only callers that resolve
it first.
The shadow's remaining case is language unity and one owner for the rule, not coverage.

## Mechanism decision (user, after the reboot research)

The user chose option B, "Rust + clap pnpm wrapper first, everything in it", against my ranking of A (updateConfig hook plus lint),
C (Rust checker plus hook stub), D (lint and CI only) above B.
The ranking and the cons of each option were shown with the question, so this is an informed choice and stands.
The user also chose to wait for the second `pi` review before deciding; the review finished after that answer, and its findings
are below.
Consequences recorded, not yet confirmed by the user as the full design:

- The earlier decisions about the policy, no exceptions, named catalogs, manifests only, syntax only and `zod` into the catalog
  still stand, because they are about the rule, not the mechanism.
- The two TypeScript packages (`core-specifier-policy`, `cli-specifier-policy`), the `.pnpmfile.mjs` hook wiring and the mise lint
  task as the primary mechanism are not chosen.
  The `updateConfig` measurements remain valid facts, and a hook can still be added later as a second layer.
- A wrapper sees `argv`, so unlike the hook it can tell the verb and scope which verbs it blocks.
  It also reaches only callers that resolve it first on `PATH`; CI's setup action puts the real pnpm on `PATH`, so CI needs an
  explicit step that runs the checker (a `check` subcommand of the same binary, built without pnpm).
  Without that step the policy is not enforced in CI.

## Second pi review (`openai/gpt-6.1-sol`, prompt `~/temp/agent/pi-review2-prompt.txt`, output `~/temp/agent/pi-review2-output.txt`)

Told about the user's bad feeling and that the first verdict rested on the disproved no-op premise.
Verdict: do not ship A as written; proceed with an omitted option E, a dependency-free Node checker with mandatory CI source lint
and an advisory (non-blocking) `updateConfig` hook, no Rust requirement.
The user chose B before this arrived, so E is recorded as a considered alternative, not the plan.
Findings that bear on any design, including B (the reviewer's claims, not independently verified here):

- A hard-failing hook turns a manifest mistake into a repository-wide pnpm outage, and `--ignore-pnpmfile` also disables the
  existing dependency blocklist.
  A wrapper can avoid this by scoping verbs; the lint backstop must not need pnpm to run.
- CI enforcement is a proposed property, not an established one: whether any workflow runs the root `lint` task is unprobed, and
  `package-paused/` is outside the workspace.
  The checker needs an explicit CI step, repository-root-relative tracked paths including the root manifest, working-tree
  contents, NUL-delimited git output, and failure (not success) when enumeration fails.
- The probe's glob expansion was wrong outside its fixture: `**` taken literally, `!` exclusions ignored, partial-segment wildcards
  taken literally, and a missing directory throwing.
  Any enumerator must fail closed on a pattern it cannot expand, or reuse a maintained facility whose semantics are tested against pnpm.
- `isCatalog` used `RegExp.test`, which coerces an array such as `["catalog:team"]` to a matching string; a checker must require
  string values and define malformed dependency maps.
- Landing the `zod` change may need a regenerated lockfile and clean frozen-install tests on both pnpm versions, Node 24 and Windows.
- A preflight is not a postcondition: a script run from a valid workspace can write a literal specifier and exit 0.
- Several handover statements were stronger than their evidence: "every verb" (the agent's source reading found `dlx` and `doctor`
  exceptions, which this handover already records), "only bypass" (a valid alternative pnpmfile was not tested), "must hand-parse
  YAML" (a Rust stub or wrapper can be handed `workspacePackagePatterns` or read `pnpm-workspace.yaml` itself), and Node 26 probes
  do not cover Node 24.
- Option C has a bootstrap obligation, not a circular dependency: the checker binary must be built before the first hooked pnpm call.
- The bad feeling is not evidence for or against Rust; a structural reading is that the design grew from specifier enforcement
  into a prerequisite for most pnpm work while bootstrap, recovery and CI execution stayed unspecified.

## Precedent and inputs for option B (read, not built)

- `package/mpm-shadow/cli` (`Cargo.toml`, `src/real_binary.rs`, `src/shim.rs`, README):
  the mode is chosen by the name the binary is started under, forwarded arguments are never given to clap
  (clap owns only the shadow's own commands), the real tool is found by a PATH search that skips every candidate resolving to the
  shadow itself, the child inherits the standard streams, and the exit status is passed through (`128` plus the signal on a signal).
  It is Unix-only: `real_binary.rs` imports `std::os::unix::fs::PermissionsExt` and splits `PATH` on `:`;
  its release archives are Linux only (x86_64 and aarch64).
  Its dependencies for the shadow work are `clap` and `serde_json` (with `preserve_order`); the rest serve its other jobs.
  Per-crate pins come from `rust-catalog.toml` through file-enforcer, not by hand.
- The Rust catalog (`rust-catalog.toml`) holds `clap`, `serde`, `serde_json`, `anyhow`, `globset`, `ignore`, `libc`, `toml`,
  `tracing`; it holds no YAML crate.
  Adding a crate is governed by `doc/decision/rust-dependency-catalog.md`.
- The re-run scan finds 256 tracked `package.json` files (it found 248 earlier, so the count moved) and still exactly one
  violation, `zod` in `cost-dismissal`; no tracked `package.json5`, `package.yaml` or `package.yml` exists, so those pnpm manifest
  formats are a known limit, not a current gap.
- CI: the baseline workflows (`.github/workflows/current-baseline.yml`, `.depot/workflows/current-baseline.yml`) and
  `npm-release.yml` run on Linux runners; macOS and Windows runners appear only in Rust build and test workflows
  (`cargo-publish.yml`, `music-player-release.yml`, `fs-id.yml`, `readonly-semantic-bridge.yml`).
  Whether those non-Linux jobs run `pnpm install` was not checked.

## Dependency round for option B (user answers)

Adopted from the `mpm-shadow` precedent without asking, veto open (the user's answers did not veto any):
crate home `package/pnpm-shadow/cli` (package `monochromatic-pnpm-shadow`, binary `pnpm-shadow`, linked as `pnpm` the way
`mpm-shadow` is linked as `mpm`); mode chosen by the name the binary is started under; forwarded pnpm arguments never reach clap;
the real-pnpm search skips every candidate resolving to the shadow itself; dependencies `clap` and `serde_json` from
`rust-catalog.toml`, `tempfile` for tests, no `anyhow` or `tracing`; a `check` subcommand that CI and a mise lint task call without pnpm.

- Manifest enumeration: the user chose the `ignore` crate walk, against my ranking (git subprocess first).
  Consequence: the scope is the set of `package.json` files the walk sees, not strictly the tracked set.
  Measured today with ripgrep, which uses the same crate: the default walk, the walk with hidden files, and
  `git ls-files --cached` return the identical 257 manifests, so there is no difference yet.
  The sets can diverge when an untracked, non-ignored `package.json` exists (it would fail the check), or when a tracked file is
  gitignored (it would be skipped); the crate also needs a git work tree to apply `.gitignore` by default, which is unmeasured here.
  Probe `~/temp/agent/compare-enumeration.ts` (its worktree-list step failed, see below, so where linked worktrees live relative
  to the repo root is not established; a worktree inside the tree and not ignored would add its manifests to the walk).
- Workspace and YAML: no YAML, as recommended. The root is the nearest ancestor holding `pnpm-workspace.yaml` (existence only),
  and the preflight checks the same manifests as `check`.
  Known effect: a violation in `package-paused/` also blocks pnpm in the wrapper although pnpm ignores that manifest.
- Platforms: the user wrote "unix now, cross platform for a later version."
  Recorded as: Unix-only for now (as `mpm-shadow`), cross-platform planned for a later version.
  Whether `check` should already be portable now (my first-ranked option) was not separately answered.
- Observed during this round, not caused by this session: every `git` command in the main checkout currently fails through the
  git shadow with `config-invalid` for `cli-git.config.jsonc` (`policies.security/forbidden-strings[1].rulesFile must be a
  string, found an array`), which looks like another session's in-progress work on forbidden-strings.
  The last handover commit before it, `f6df1b5d3f`, succeeded.

## CI execution facts for the `check` step (measured by reading the workflows)

- No workflow runs the root `lint` task: the `task-run` calls across the workflows include `lint:monochromatic-lint` (three times)
  but never `lint`, so a new task added to the root `lint` list would not run in CI by itself.
  This confirms the `pi` finding that CI execution of a new lint is not automatic.
- `.github/workflows/current-baseline.yml` and `.depot/workflows/current-baseline.yml` trigger only on `workflow_dispatch`.
  `.github/workflows/advisory-core.yml` triggers on pushes and pull requests to `main`, runs `task-run prepare:pnpm`,
  and then runs named repository-invariant steps with `continue-on-error: true` (`Rust catalog drift`, `pnpr config drift`)
  that print `::error::` with the remedy; this is the existing precedent for a check step.
- `prepare:pnpm` runs `prepare:pnpm:install` (`pnpm install`, alias `pi`) then `prepare:pnpm:others:*` (`mise.toml`).
- CI builds Rust crates through `task-run //package/<crate>:build`, so a Rust `check` binary can be built in the same job.

## Behavior round for option B (user answers)

- Verb scope: the user chose "Warn only, never block", against my ranking (install-class verbs first, warn only last).
  The wrapper prints every violation to stderr and always runs the real pnpm with the arguments untouched.
  Consequences: no verb parsing and no fail-closed argument scan are needed for blocking; the second `pi` review's outage
  finding does not apply to the wrapper; the wrapper cannot stop a violation from landing locally; enforcement exists only where
  `check` is run and fails (CI, a lint task).
  Whether the warning appears on every invocation or only on install-class verbs is not decided; the first needs no verb parsing.
- Skip switch: none, as recommended. The only way around the wrapper stays calling the real pnpm directly.
- CI step: a named step before `task-run prepare:pnpm` in `.github/workflows/advisory-core.yml`, as recommended.
  Measured after the answer, which the question had listed as a check to make:
  that workflow states "results are advisory; no required checks and no branch-protection change"
  (`Summary` step), and its final `Enforce advisory result` step fails the job only when the `static`, `build` or `unit` step
  failed; the existing `Rust catalog drift` and `pnpr config drift` steps use `continue-on-error: true` and are not in that list,
  so their failures leave the job green.
  A new check step therefore shows a red step in the log but does not fail the job unless it is added to the `Enforce advisory result`
  condition (or run without `continue-on-error`), and no branch protection makes the job a merge gate either way.
  The line numbers in this workflow moved between two reads in this session, so another session edited it.
- Not asked yet: whether the new step is added to the `Enforce advisory result` condition; root discovery now that the wrapper does
  not parse `--dir` or `-C` (the nearest ancestor from the current directory is the simple answer); warning wording;
  exit-code behavior of `check` (nonzero on any violation); the test corpus layout (a JSON vectors file read by the crate's nextest
  suite, since no TypeScript implementation is planned); the `zod` landing order (the `check` step and the `zod` fix must land
  together or the new CI step fails from its first run); PATH linking through the toolchain plan.

## Two gaps found after the behavior round (my research gaps, found by reading)

- The CI step chosen above runs rarely on the push path.
  `advisory-core.yml` has a `gate` job and the `core` job runs only when `needs.gate.outputs.allowed == 'true'`.
  Pushes are admitted under budgets of 2 per minute, 4 per hour and 8 per day (counted from `gh run list`); pull requests and
  manual dispatch are always admitted.
  Commits on `main` per day over the last eight days were 436, 382, 722, 73, 5, 26, 1049, 268 and 2 (local commit counts; auto-push
  means many reach the remote), so on a normal day most pushes are not admitted and a check inside `core` would not run for them.
- I never examined the repository's own commit-time policy engine as a home for this check, although `package/git-policy/cli`
  (`cli-git`, the shadowing `git`) is where this repository enforces manifest invariants today:
  - `doc/decision/cli-git-policies-platform.md`: the wrapper enforces policies with severity `off`, `warn` or `error`, fails closed
    when an enabled policy cannot complete, reads candidate bytes of staged files, and treats CI as the authoritative layer that must
    stay independent of the local wrapper ("Local enforcement remains best-effort fast feedback; CI remains authoritative").
  - `package/git-policy/cli/README.md`, "Policies": the wrapper ships a fixed set of policies and no others; the config keys
    `plugins` and `trust` are refused, so a new policy means a native Rust policy inside that crate
    (`src/native/policy_registry.rs` and siblings), listed in `cli-git.config.jsonc` where the four optional ones are listed.
    `mono/dependent-version-bump` already reads package manifests at commit time and blocks at `error`.
  - A `mono/catalog-specifiers` policy would reject a commit whose staged `package.json` bytes violate the rule, at the point where a
    violation lands, with no directory walk and no PATH-order dependency beyond the git shadow every commit already uses.
    Costs: it is another session's active crate (versions 0.4.x, release ordering, and a toolchain sync from another worktree
    reinstalled an older `cli-git-native` into the shared `~/.cargo/bin` today and broke every git command until reinstalled);
    it acts only at commit time, so a local `pnpm install` stays silent without the wrapper's warning.

## Open items

- The user has not confirmed that the shared understanding is complete.
  Dependencies, enumeration, workspace discovery and platforms are answered above.
  Still to ask: which pnpm verbs the wrapper blocks (a wrapper sees `argv`, so this can be scoped, unlike the hook), how it finds
  the workspace root when `--dir`, `-C`, `--filter` or `--workspace-root` is passed, what it does when the `ignore` walk or the
  real-pnpm search fails (fail closed), the explicit CI step and its place before `prepare:pnpm`, `PATH` linking through the
  toolchain plan, how the policy test corpus is shared, the `--ignore-pnpmfile`-style bypass question (an environment variable
  or flag to skip the preflight, or none), the repair message wording, and the `zod` landing order.
- Precedent to read before asking: `package/mpm-shadow/cli` (`Cargo.toml`, `real_binary.rs`, `shim.rs`), the Rust dependency
  catalog (`rust-catalog.toml`, `doc/decision/rust-dependency-catalog.md`) and the `cli-git-native` shadow.
- Whether the `updateConfig` hook is kept as a second layer later is open; nothing is built.

## Next action

Read the shadow precedent and the Rust catalog, enumerate every dependency option for the wrapper and get each picked,
then continue the design questions for option B.
Implementation, including the `zod` fix, waits for the user's explicit confirmation that the shared understanding is complete.
