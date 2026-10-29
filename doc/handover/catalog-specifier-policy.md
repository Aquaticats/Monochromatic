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
  It does not run on a plain no-op `pnpm install` that prints "Already up to date" (`optimisticRepeatInstall`).
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
- Named catalogs are allowed; pnpm itself rejects a reference to a missing entry.
- Mechanism: one shared check, run by a mise lint task and by the `.pnpmfile.mjs` hook.
- Package home (the hook-facing validator must stay dependency-free, see the probes below):
  new `package/dependency-catalog/core-specifier-policy` (pure validator and manifest enumeration)
  and a thin `package/dependency-catalog/cli-specifier-policy` for the mise task.
  `core-pnpm` stays parsing-only.
- Lint scope: every tracked `package.json`, including the root and `package-paused/`.
  The hook covers only workspace manifests, identified by a workspace-name set.
- The check reports every violation in one run, naming the file, the section and the dependency.
- `zod` is already in the catalog as `'>=4.6.6'` (`pnpm-workspace.yaml`), so the fix is manifest-only:
  `cost-dismissal` switches from `^4.6.7` to `catalog:`.
  This corrects an earlier statement in this handover that the catalog entry had to be added.

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
The user delegated the timing and reach decisions, so these are agent recommendations, open to veto.

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

## Open items

- The user has not yet confirmed that the shared understanding is complete.
- The user may veto the shadow recommendations above.

## Next action

After confirmation: fix `zod`, build `core-specifier-policy` and `cli-specifier-policy` with adversarial boundary tests,
wire the lint task into the root `lint` task's repo-wide list,
wire the hook in `.pnpmfile.mjs`, correct its stale comment, then verify each failure path in a throwaway fixture.
