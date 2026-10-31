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
- The hook must build a workspace-name set (plus the root name) from `package/*/*`,
  but `pnpm-workspace.yaml` parsing normally needs the `yaml` library, which the hook cannot import.
  Open: scan `package/*/*/package.json` with `node:fs`, or parse the one `packages:` key by hand.
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
- Cost: the same scan over the real repo's workspace manifests took a median of 4.9 ms over 15 in-process runs
  (minimum 4.4, maximum 9.0 on the first), and it found the one real violation, `zod` in `cost-dismissal`.
  This is the scan alone, not pnpm's own startup.
- Limits: the hand-written glob expansion supports only `*` segments; the real workspace uses `package/*/*`, which it handled.
  A `!` negation pattern or `**` in the workspace file would need handling.

### CI facts that bear on a PATH shadow

- Both CI systems install pnpm through the `task-cli-setup` action and run `task-run prepare:pnpm`
  (`.github/workflows/current-baseline.yml`, `.depot/workflows/current-baseline.yml`).
  CI pins pnpm 12.14.0 while the local toolchain is 12.16.1.
  Whether the `updateConfig` results above hold on 12.14.0 is being probed in a container; recorded below when it lands.
- The action puts node, pnpm and the task CLI on `PATH` (`.github/actions/task-cli-setup/action.yml`) and nothing about a
  shadow directory.
  The mpm shadow reaches callers because the toolchain plan links it into a gitignored directory first on `PATH`
  (`doc/decision/mpm-toolchain-adoption.md`); a pnpm shadow would need the same link, and CI would need a built Rust binary
  for each runner OS (the workflows include Windows steps).
- The Rust dependency catalog has `clap`, `serde` and `serde_json` but no YAML crate (`rust-catalog.toml`),
  so a Rust checker must hand-parse `packages:` too or add a crate.

### What this changes

Reach is no longer an argument for the Rust shadow over a hook: an `updateConfig` hook covers the no-op install and every
other verb on any caller, with `--ignore-pnpmfile` as the only bypass found, while a shadow covers only callers that resolve
it first.
The shadow's remaining case is language unity and one owner for the rule, not coverage.

## Open items

- The user has not yet confirmed that the shared understanding is complete.
- The user may veto the shadow recommendations above.
- Hook timing (lint first then hook, both together, or lint plus a mise call) and the Rust-wrapper-first option are open.
- Pick the dependencies listed above before any other branch is explored.

## Next action

After confirmation: fix `zod`, build `core-specifier-policy` and `cli-specifier-policy` with adversarial boundary tests,
wire the lint task into the root `lint` task's repo-wide list,
wire the hook in `.pnpmfile.mjs`, correct its stale comment, then verify each failure path in a throwaway fixture.
