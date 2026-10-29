# Plan: eliminate generic package categories

Status:
 grilling in progress (started 2026-10-29),
 executing per category as each category's mapping settles.

## Request

The user asked to eliminate these first-level categories under `package/`:
`cli`,
 `module`,
 `shim`,
 `stub`,
 `build-tool`,
 `dev-script`,
 `desktop-app`,
 `desktop-daemon`,
 `rust-module`,
 `webapp-productivity`,
 `git`.
Reason given:
 the words are too generic,
 and products should be owned by a product category.

This supersedes the "keep broad utility buckets broad" decision
(Rule 4 and the `module`,
 `dev-script`,
 `shim`,
 `stub` entries)
in [the package category rebalance plan](package-category-rebalance.md).
That plan's other rules still apply:
 category means ownership boundary,
 exactly `package/<category>/<project>`,
 adapter to core to neutral utility dependency direction.

## Decisions

### Scope

Only the eleven listed categories for now.
`config`,
 `mcp`,
 `test-fixture`,
 `linter`,
 `runtime-error`,
 and `learning` were offered;
the user will handle them later.
Consequence:
 a product cluster created here may start without its members from those categories
(for example `mvm` gets `cli` now while `mcp/mvm` stays until the later wave).

### Product versus host

When pieces of one product live in several categories and share code,
they belong to one product category.
Pieces that only share a name stay where they are.
The user cited `music-player` as the prime example of the target shape.
Host categories outside the eleven are out of scope for this wave.

### Shared libraries

A library moves into the product that is its main consumer.
With no main consumer,
 it joins a domain family category (for example `async`,
 `test`).
It becomes its own category only when no family fits.

### Project names

The second segment is an artifact role:
 `core` (library),
 `cli`,
 `mcp`,
 `app`,
 `daemon`,
 `plugin-*`,
 `fixture-*`,
 `variant-*`.
A language suffix (`core-ts`,
 `core-rust`) appears only when two implementations coexist.
Never repeat the product name (`package/done/done`).
npm names follow the path per `AGENTS.md` rule SGD;
Rust crate names stay unchanged,
 as in [the singular dir name plan](singular-dir-name-invariant.md).

### Execution

The user overrode the grilling default of acting only after every branch settles:
each category executes as soon as its own mapping is settled.

## Peer holds

Recorded 2026-10-29 from peer session replies.
Do not move these until the named session reports them idle.

- `monochromatic-ec` (IDE and compositor work):
  `package/desktop-app/ide` and `package/cli/nested-wayland-session`;
  release workflows (`ide-release.yml`,
   compositor tags) and its scratch branches assume current paths.
  It also edits `package/desktop-app/terminal/task.ts`.
- `monochromatic-fb` (issue #1800,
   Rust dependency catalog):
  `package/dev-script/file-enforcer` (uncommitted),
  `package/module/rust-catalog`,
  `package/dev-script/catalog-tighten`,
   a new `package/dev-script/` package,
   root `file-enforcer.config.ts`,
   `mise.no-env.toml`,
  and every first-party `Cargo.toml`
  (24 manifests across `cli`,
   `desktop-app`,
   `git-policy`,
   `linter`,
   `music-player`,
   `rust-module`,
   `codex-plugin`).
  Its sync derives internal path-dependency versions from relative paths,
  so moving any Rust package during that work breaks it.
- Translation-repair session:
  wait until its two branches (`83a854a2ab` cooldown,
   `a7bab35fac` round 8) land.
  It imports `module-async-time`,
   `module-caught-value`,
   `module-llm-type`,
   `module-logger`,
   `module-or-throw`,
   `module-test`,
   and `git-executable` by name;
  its docs cite `package/module/logger` by path.
  One gate takes about 20 to 25 minutes.

Uncommitted main-worktree changes owned by none of these peers:
`toolchain.lock.json`,
 `mpm.managers.toml`,
 `mpm.packages.toml`,
 `.projenrc.ts`,
 `package/dev-script/file-enforcer/src/toolchain/plan.artifacts.ts`,
 `package/dev-script/file-enforcer/src/toolchain/plan.tools.ts`.
Leave them alone.

## Per-category mapping

Pending research reports in `~/temp/agent/package-category-elimination/`.

## Next action

Collect the per-package research,
propose each category's mapping,
and move the first unblocked category once the user settles it.
