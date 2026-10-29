# Mise eliminated from CI

Status:
accepted on 2026-10-25 under `AGENTS.md` `DRR`.
This decision is in force.

## Decision

Mise is no longer installed, configured or invoked by any GitHub Actions or
Depot CI workflow in this repository.
Every workflow installs its tools directly (Node and pnpm through the vendor
archives `.github/actions/task-cli-setup` downloads, Java through the pinned
Temurin archive, Rust through `rustup`) and runs the repository's tasks
through `task-run`, the node-native task CLI at
`package/dev-script/task-cli/src/run.ts`.
No external GitHub action installs a tool: the user asked to avoid
third-party actions where possible, because every one of them is an
organization and repository allow-list entry
(`doc/runbook/depot-trial-action-policy.md`).
The root `mise.toml`, `mise.no-env.toml` and `mise.lock` remain as the local
developer task system and the pinned-tool source of truth; `mise.lock` is
still read for version pins, but mise itself is never executed in CI.
Moved on 2026-10-26:
 the pinned-tool source of truth is now `toolchain.lock.json`
 (`doc/decision/mpm-toolchain-adoption.md`) and `mise.lock` is retired.

## Scope

- Applies to every workflow under `.github/workflows/` and
  `.depot/workflows/`.
- Does not change local development: `mise run` keeps working unchanged on
  developer machines.
- Does not change publishing issuers, release producers or the trust
  boundaries recorded in `depot-ci-adoption.md`.
- Keeps `mise.lock` as the pinned version record CI reads for Rust, Node,
  pnpm, the Android SDK components and the conformance tool.
  Moved on 2026-10-26:
  the record CI reads is `toolchain.lock.json`
  (`doc/decision/mpm-toolchain-adoption.md`).

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

The shared composite action `.github/actions/task-cli-setup` installs Node
and pnpm at the versions `mise.lock` records with plain shell steps (vendor
archives plus `npm install --global`) and exposes `task-run` on `PATH` through
a shim directory it adds to `GITHUB_PATH`, so it works on the Windows runners
where `/usr/local/bin` does not exist.

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
- Remote (Namespace `nsc` ephemeral instance, Wolfi base): after `apk add
  libatomic`, the `task-cli-setup` recipe (Node 26.12.0 archive, pnpm 12.14.0
  through `npm install --global`, `task-run` shims) and
  `pnpm install --frozen-lockfile` run on a bare instance, and
  `task-run //package/module/logger:build`, `//package/module/logger.fuzz:lint:types`
  and `//package/module/logger.fuzz:fuzz -- --budget 3000` pass there through
  the shim.
- The JDK recipe the kotlin and music-player jobs run was verified on a bare
  instance: it installs `Temurin-21.0.12+8`, the `mise.lock` `tools.java` pin.
- Every tool archive URL the setup code uses answers HTTP 200
  (`nodejs.org/dist` for Node 26.12.0 and 26.5.0, the Adoptium release assets
  for `jdk-21.0.12+8`).
- `task-run //package/module/logger.fuzz:fuzz -- --budget 1` reports the
  `1ms` budget in its campaign line, and
  `task-run lint:monochromatic-lint -- 'a b.txt'` reaches the linter as one
  path word, which proves the `--` option mapping and argv forwarding.
- Workflow files were checked with `rg 'mise-action|mise run|mise install|
  mise exec|MISE_AUTO_INSTALL|MISE_JOBS' .github/workflows .depot/workflows`
  and return no matches; `rg 'actions/setup-node|actions/setup-java|corepack'`
  also returns none, and `actionlint` reports only the pre-existing
  `depot-ubuntu-24.04-8` runner labels.

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
- Tool setup carries no external action, but the workflows still use the
  approved platform actions (`actions/checkout`, `actions/cache`, the artifact
  and attestation actions, the publishing actions, `github/codeql-action`,
  `ossf/scorecard-action`, `anthropics/claude-code-action`). Replacing those
  with shell and API calls is open work, not decided here.
- `logger.fuzz`'s file-sink property has been red since 2026-10-22 on its own
  cause (`doc/handover/mise-free-ci-migration.md` names the counterexample and
  the reproduction); it is not a migration defect and it keeps the
  `logger-fuzz` fuzz smoke step red.
- The `package/music-player/*/task.ts` bodies still need the port repairs the
  handover lists before `music-player-release` runs them.
