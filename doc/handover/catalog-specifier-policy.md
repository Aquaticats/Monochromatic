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
- Package home: new `package/dependency-catalog/core-specifier-policy` (pure validator and manifest enumeration)
  and a thin `package/dependency-catalog/cli-specifier-policy` for the mise task.
  `core-pnpm` stays parsing-only.
- Lint scope: every tracked `package.json`, including the root and `package-paused/`.
  The hook covers only workspace manifests, identified by a workspace-name set.
- The check reports every violation in one run, naming the file, the section and the dependency.
- `zod` moves into the catalog as `>=4.6.7`, and `cost-dismissal` switches to `catalog:`.

## New input after the decisions

The user plans to shadow `pnpm` with an own Rust and clap wrapper.
No document in `doc/` mentions it yet; the nearest precedent is `package/mpm-shadow/cli`,
which finds the real tool on `PATH`, runs it as a child with arguments untouched, then acts after it.
A `pnpm` shadow could run the specifier check before pnpm starts,
which would cover the no-op install that the `readPackage` hook cannot reach.
That is an inference from the shadow design, not a measurement.
The mechanism, package-home and validator-language decisions above are under revision until the user states
the shadow's timeline and which callers (CI, mise tasks, agents) would go through it.

## Open items

- The user has not yet confirmed that the shared understanding is complete.
- Timeline and reach of the `pnpm` shadow, and whether the validator moves to Rust.

## Next action

After confirmation: fix `zod`, build `core-specifier-policy` and `cli-specifier-policy` with adversarial boundary tests,
wire the lint task into the root `lint` task's repo-wide list,
wire the hook in `.pnpmfile.mjs`, correct its stale comment, then verify each failure path in a throwaway fixture.
