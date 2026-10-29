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
  It also loads `package/shim/katex` as `katex` through the
  `'micromark-extension-math>katex': 'link:package/shim/katex'` override in `pnpm-workspace.yaml`,
  so that shim is held too and its `link:` path must move in the same commit.
  The peer will annotate its historical ledger mentions of the shim path after the move.
  One gate takes about 20 to 25 minutes.
  The cooldown branch landed as `b95d7cf83d`;
  round 8 is still in its gate.

Uncommitted main-worktree changes owned by none of these peers:
`toolchain.lock.json`,
 `mpm.managers.toml`,
 `mpm.packages.toml`,
 `.projenrc.ts`,
 `package/dev-script/file-enforcer/src/toolchain/plan.artifacts.ts`,
 `package/dev-script/file-enforcer/src/toolchain/plan.tools.ts`.
Leave them alone.

## Measured facts

Measured 2026-10-29 at `5b9ef0444f`;
full report and raw data in `~/temp/agent/package-category-elimination/blast-radius.md`.

- None of the rebalance plan's cluster moves (`mvm`,
   `oxlint`,
   `done`,
   `music-player/native-core`) has landed;
  `package/mcp/mvm` still imports `cli-mvm` internals.
- Every existing npm name already equals `@monochromatic-dev/<category>-<project>`.
- `package/cli/markdown-lint`,
   `package/rust-module/rust-linter-core`,
   and `package/rust-module/rust-linter-pattern` are untracked build debris,
   not packages.
- Published on npmjs:
   `module-logger` (0.4.5) and `module-fs-path` (git tags 0.1.0 to 0.2.1).
  Renaming them creates new npm packages;
  trusted publishing needs a manual first publish per name (`doc/decision/npm-publishing.md`).
  No `cli` package is published to npm.
- The private pnpr registry (`package/config/pnpr/config.yaml`,
   generated by `file-enforcer.config.ts`) lists 71 names from these categories;
  external repo `Aquaticat/labwc-config` depends on `module-or-throw`.
- `mise.toml`,
   `mise.no-env.toml`,
   and `.github/workflows/npm-publish.yml` hardcode `module-$NPM_MODULE` and `//package/module/$NPM_MODULE`.
- Rust crates without `publish = false`:
   `cli/forbidden-strings`,
   `cli/nested-wayland-session`,
   `cli/wg-quicker-exempt`,
   `rust-module/forbidden-regex`,
   `rust-module/jsonc-edit`;
  crate names stay,
   but `cargo-publish.yml` and `ide-release.yml` path filters change.
- External reference outside the repo:
  `~/.config/systemd/user/claude-cpu-watch.service` is a symlink into
  `package/dev-script/claude-cpu-cap/systemd/` and hardcodes that path.
- Hand-edit hotspots beyond `rg` of `package/<category>/`:
   `pnpm-workspace.yaml` `link:package/shim/*` overrides,
   `.pnpmfile.mjs` stub names,
   root `package.json`,
   `.changeset/config.json` ignores,
   `file-enforcer.config.ts` (including `CARGO_PROFILE_BY_DIR`),
   `Dockerfile`,
   `.gitignore`,
   `.cargo/config.toml`,
   `monochromatic-lint.config.jsonc`,
   12 `.github` files,
   `.depot/workflows/weekly-native.yml`,
   and joined path segments such as `'package', 'dev-script'` in 9 files.
- `package-paused/` reuses `desktop-daemon`,
   `dev-script`,
   and `module` names (4 packages);
  per precedent the frozen tree is not re-laid out,
   only stale references swept.

## Naming rules adopted during the `cli` round

- npm names rename in lockstep with paths (`AGENTS.md` rule SGD);
  this supersedes Rule 6 of the rebalance plan for this wave.
  No `cli` package is published on npmjs,
   and the pnpr list is generated,
   so `cli` has no compatibility cost.
- When one product holds two projects with the same role,
  the qualifier follows the role:
   `cli-allowedips`,
   `core-allowedips`,
   `cli-unused-export`
  (same order as `plugin-*`,
   `fixture-*`,
   `variant-*`).
- Sidecars keep sidecar form beside their subject:
   `mutation-test/cli.fixture`,
   `forbidden-strings/cli.fuzz`.
- Bin names and Rust crate names stay unchanged.
- The user does not own the WireGuard brand,
  so no category is named `wireguard`;
  `wg` is acceptable because WireGuard did not register it.

## Per-category mapping

Research reports:
 `~/temp/agent/package-category-elimination/` (`apps-and-tools.md`,
 `modules.md`,
 `blast-radius.md`).

### `cli` (settled 2026-10-29)

- `cli/android-exempt-unused` to `android-exempt-unused/cli`.
- `cli/forbidden-strings` to `forbidden-strings/cli`;
  `cli/forbidden-strings.fuzz` to `forbidden-strings/cli.fuzz`.
  The regex engine joins the same product:
  `rust-module/forbidden-regex` to `forbidden-strings/core`,
  with `.bench` and `.fuzz` to `core.bench` and `core.fuzz`.
  This reverses Phase 7 of the rebalance plan,
  which assumed `cli` stayed.
- `cli/fy` to `fy/cli` (zero consumers;
   the user chose to move rather than pause it).
- `cli/git-clone-size` to `git-clone-size/cli`.
- `cli/markdown-lint`:
   untracked build debris (source deleted in `5bc05682ec`);
  list with `git clean --dry-run -d -X`,
   then remove.
- `cli/mutation-test` to `mutation-test/cli`;
  `cli/mutation-test.fixture` to `mutation-test/cli.fixture`.
- `cli/unused-export` to `mutation-test/cli-unused-export`,
  because it imports `mutation-test` engine internals (shared code).
- `cli/mvm` to `mvm/cli` now;
  the `mvm/core` extraction becomes a later separate task,
  deviating from the rebalance plan's core-first order by user decision.
  `mcp/mvm` stays in `mcp` this wave and switches its import to the new name.
- `cli/nested-wayland-session` to `nested-wayland-session/cli`,
  after `monochromatic-ec` releases it.
- `cli/open-code-review-issue` to `open-code-review-issue/cli`.
- `cli/rgffplay` to `rgffplay/cli` (zero consumers;
   moved by user choice).
- `cli/terminal-exec` to `terminal-exec/cli`.
- `cli/vmsync` to `vmsync/cli`:
  its e2e drives `mvm` but it imports no `mvm` code.
- `wg` family (the shared library has no single owner):
  `module/wg-allowedips` to `wg/core-allowedips`,
  `cli/wg-allowedips` to `wg/cli-allowedips`,
  `cli/wg-quicker` to `wg/cli-quicker`,
  `cli/wg-quicker-exempt` to `wg/cli-quicker-exempt`.

The uncommitted toolchain refactor in main
 (`file-enforcer/src/toolchain`,
 `toolchain.lock.json`,
 `mpm.*.toml`,
 `.projenrc.ts`,
 `pnpm-workspace.yaml`,
 `pnpm-lock.yaml`,
 two `dsh-plugin` manifests)
belongs to the user,
who allowed committing it as-is so the batch can regenerate the lockfile.

Correction:
 commit `56178d85ed` says it holds the user's toolchain refactor,
 but only `src/toolchain/plan.artifacts.ts` and `plan.tools.ts` do;
 the other 25 files are an accidental oxlint autofix from the `monochromatic-fb` session
 (a corrective GitHub commit comment is posted).
Known break the user chose to fix later:
 `package/dev-script/file-enforcer/src/toolchain/update.ts:632-633` TS7006
 fails that package's `lint:types`;
 batch verification treats this as preexisting.

`monochromatic-fb` released Rust crate moves after `fd0bd03540`.
It also approved path-only edits in `check-mode.ts`,
 `cargo/catalog/*.ts`,
 `file-enforcer.config.ts`,
 `package/dev-script/file-enforcer/mise.toml`,
 `mise.no-env.toml`,
 `.github/workflows/advisory-core.yml`,
 and `rust-catalog.toml` comments,
and wants a message when the move commit lands.
Its notes for crate moves:
 `file-enforcer.config.ts` discovers crates via `package/*/*/Cargo.toml` and `package/*/*/*/Cargo.toml`;
 rewrite relative `path` dependencies and let the catalog sync re-derive versions;
 afterwards run `mise run sync:files`,
 `node package/dev-script/file-enforcer/src/cli.ts --check`,
 and `cargo metadata --locked --offline` per crate.

## Next action

Collect the per-package research,
propose each category's mapping,
and move the first unblocked category once the user settles it.
