# Mise eliminated from CI

Status:
accepted on 2026-10-25 under `AGENTS.md` `DRR`.
This decision is in force.

## Decision

Mise is no longer installed, configured or invoked by any GitHub Actions or
Depot CI workflow in this repository.
Every workflow installs its tools directly (Node through
`actions/setup-node`, Rust through `rustup`, Java through
`actions/setup-java`, pnpm through `corepack`) and runs the repository's tasks
through `task-run`, the node-native task CLI at
`package/dev-script/task-cli/src/run.ts`.
The root `mise.toml`, `mise.no-env.toml` and `mise.lock` remain as the local
developer task system and the pinned-tool source of truth; `mise.lock` is
still read for version pins, but mise itself is never executed in CI.

## Scope

- Applies to every workflow under `.github/workflows/` and
  `.depot/workflows/`.
- Does not change local development: `mise run` keeps working unchanged on
  developer machines.
- Does not change publishing issuers, release producers or the trust
  boundaries recorded in `depot-ci-adoption.md`.
- Keeps `mise.lock` as the pinned version record CI reads for Rust, Node,
  pnpm, the Android SDK components and the conformance tool.

## Replacement

`@monochromatic-dev/dev-script-task-cli` (`package/dev-script/task-cli`)
executes the same task graph the `mise.toml` tree declared, from `task.ts`
files that mirror each package's `mise.toml` task names, bodies, `extends`,
`depends` and `usage` options.

- Root templates (`build`, `build:js:*`, `test`, `test:unit`, `lint`,
  `lint:oxlint`, `lint:types`, `watch:*`) live in `task.ts` at the repo root
  and carry the shared bodies the old `[task_templates.*]` held.
- Root tasks (`prepare:pnpm*`, `lint:monochromatic-lint`, `npm:module:*`,
  `changeset:*`) are inlined in `task.ts`.
- Package tasks live in `package/**/task.ts`, one per package the CI
  workflows reference, generated from the matching `mise.toml` and verified
  by running each body through `task-run`.
- `task-run` resolves `//package/<category>/<name>:<task>` and root task
  names, honours `-- <options> [-- passthrough]`, `usage_*` environment
  variables and `depends` chains, and fans out umbrella tasks over every
  package `task.ts` carrying the suffix.

The shared composite action `.github/actions/task-cli-setup` pins Node and
pnpm to the versions `mise.lock` records and exposes `task-run` on `PATH`.

## Why

- Every workflow previously installed mise, then a subset of tools through
  mise, then ran tasks through mise; the action, its version pin and its
  toolchain installer were three moving parts between the runner and the
  task body (`doc/troubleshooting/mise-action-version-ahead-of-release.md`,
  `doc/troubleshooting/mise-2026-10-19-cmake-lock-resolution.md`).
- Direct tool setup removes the action, the pin and the auto-install footgun
  (`MISE_AUTO_INSTALL=false` guards in twelve workflows) in one step.
- `task-run` runs bodies with the runner's node directly, so a task failure
  reports the body's own diagnostic instead of a mise wrapper frame.

## Verification

- Local: `task-run //package/module/logger:build`,
  `//package/module/logger:test:unit`, `//package/module/toml-edit:build`,
  `//package/module/toml-edit:test:unit`, `//package/module/p-map-fork:build`,
  `//package/cli/forbidden-strings:build` and
  `//package/linter/monochromatic-lint:build` all pass through the CLI on the
  developer machine.
- Remote (Namespace `nsc` ephemeral instance, Wolfi base): the CLI runs the
  same `//package/module/logger:build` and `//package/module/logger:test:unit`
  bodies on a clean instance after `apk add libatomic`, Node 26.12.0 and pnpm
  12.14.0; the dependency-resolution step needs a real `pnpm install` over the
  workspace lockfile, which the CI workflows run through `task-cli-setup`.
- Workflow files were checked with `rg 'mise-action|mise run|mise install|
  mise exec|MISE_AUTO_INSTALL|MISE_JOBS' .github/workflows .depot/workflows`
  and return no matches.

## Limits

- `mise.toml`, `mise.no-env.toml` and `mise.lock` are unchanged and remain
  the local developer surface; removing mise from local development is a
  separate decision.
- The `toml-edit` conformance task now fetches the upstream `toml-test`
  runner through `npx --yes toml-test` instead of
  `mise exec github:toml-lang/toml-test@latest`; the moving-oracle property
  is preserved and the version is still logged.
- Rust toolchains in CI come from the runner's `rustup` (`nightly`, matching
  `mise.lock`'s `tools.rust` entry) instead of `mise install rust`; the
  dated-nightly pin remains recorded in `mise.lock` for local use and for the
  `music-player-release` jobs that still parse it for cache keys.
