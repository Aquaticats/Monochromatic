# `bun install` upstream bugs

This project does not file or track `bun install` (Bun's package manager) bugs
as GitHub issues, and does not invest in local workarounds for upstream Bun
package-manager defects.

## Why this is out of scope

The workspace's package manager is **pnpm**, not Bun. `mise.toml` and
`mise.no-env.toml` define the install entry point as `task-pnpm install`
(see `tasks."prepare:pnpm:install"`); the fix task is `fix:reinstall` (`rm -rf
node_modules; task-pnpm install`). The pnpm migration landed in commit
`e424ffde build(*): migrate package manager from vlt to pnpm`.

Bun is still the **runtime** for scripts, tests, and CLI binaries (`#!/usr/bin/
env bun` shebangs, the `@monochromatic-dev/test-harness-core` harness, etc.); that
scope is unchanged. The out-of-scope-ness here is narrowly about Bun's
`install` subcommand and the package-manager pieces around it (catalog
resolution, lockfile, HTTP cache).

Once `bun install` is not on the install path, its install-time bugs cannot
affect a build, test, or deploy here. Filing tracking issues against
`bun install` defects produces issue clutter without changing any local
outcome.

The defect that motivated the original tracking is moot:

- Bun's HTTP cache sends `If-None-Match` on subsequent fetches, which combined
  with JSR's bridge 502 produced install failures for JSR-hosted packages.
  Affects `bun install` only; pnpm does not exhibit the same conditional-GET
  cache pattern in a way that triggers the JSR bug.

## What we do instead

- **For installs**: run `mise run prepare:pnpm:install` (alias `pi`) or
  `mise run fix:reinstall` for a clean rebuild.
- **For runtime**: Bun continues to execute TypeScript directly, run the test
  harness, and back per-package CLIs; that path is unaffected.
- **For per-package README quickstarts that still say `bun install`**: treat
  as historical README copy; the workspace-level installer is pnpm. README
  fixes can land opportunistically, not via a tracking issue.

## Examples of this category

The following local tracking issues were closed as out-of-scope per this policy:

- `#161` Track upstream JSR 502 on conditional GET (jsr-io/jsr#1323) for
  `bun install`

## Exception

If the workspace ever returns to `bun install` as the workspace-level
installer, revisit. The trigger is a deliberate package-manager switch
recorded in `PHILOSOPHY.tool-choices.md` (or a successor doc), not a passive
"Bun fixed the bug" upstream change.

## Re-evaluation

If pnpm degrades to the point where switching back is attractive, the
re-evaluation starts with a fresh tool-selection decision (measure install
time, lockfile stability, catalog support, monorepo features), not with
re-opening this file.
