# Mise-free CI migration

Purpose:
 remove mise from every CI workflow in this repository while keeping the
 local developer task system unchanged.

Changes:
- New `package/dev-script/task-cli`: node-native `task-run` CLI that executes
  the task graph the root `mise.toml` declared, from `task.ts` files.
- New root `task.ts` (templates + root tasks) and one `task.ts` per
  CI-referenced package, mirroring the matching `mise.toml` task names,
  bodies, `extends`, `depends` and `usage` specs.
- New shared composite action `.github/actions/task-cli-setup` that installs
  Node and pnpm at the `mise.lock` versions with plain shell (vendor archives
  plus `npm install --global`) and exposes `task-run` on `PATH`.
- All 13 `.github/workflows/*.yml` and both `.depot/workflows/*.yml` files
  rewritten: `jdx/mise-action` removed, `mise install`/`mise run`/`mise exec`
  replaced with direct tool setup and `task-run`.
- `doc/decision/mise-free-ci.md` records the decision, scope, replacement
  surface, verification evidence and limits.

What to inspect:
- `rg 'mise-action|mise run|mise install|mise exec|MISE_AUTO_INSTALL|MISE_JOBS' .github/workflows .depot/workflows`
  returns no matches (the only remaining `mise` mentions are comments and the
  `mise.lock` version reads in `music-player-release.yml`).
- `node package/dev-script/task-cli/src/run.ts //package/module/logger:build`
  and `//package/module/logger:test:unit` run green on the developer machine.
- Remote verification on a Namespace `nsc` ephemeral instance (Wolfi,
  `apk add libatomic`, Node 26.12.0, pnpm 12.14.0) ran the same `build` and
  `test:unit` bodies; dependency resolution needs a real `pnpm install` over
  the workspace lockfile, which CI runs through `task-cli-setup`.

How to respond:
- A workflow failure after this lands is a task-body porting bug: open the
  matching `task.ts` (root or package) and compare its `run` body against the
  `mise.toml` body it replaced; the old body is still in git history at
  `d26ed6d2d8` and earlier.
- To add a new package task for CI, add its `task.ts` next to the package's
  `mise.toml`; the fanout and `//package/...:task` form pick it up
  automatically.
- Local development still runs `mise run` unchanged; removing mise from local
  development is a separate decision.

## Second pass: third-party actions and the task contract (2026-10-26)

Purpose:
 honor the user's correction to avoid third-party actions where possible, and
 repair the `task-run` contracts and workflow files whose breakage the first
 pass left on `main`.

Changes:
- `739807b5a2` restores the mise task contract in `task-run`: `node_modules/.bin`
  directories join the task `PATH` (CI died with `rolldown: command not found`,
  exit 127), one leading `--` separates options so `-- --budget 3000` reaches
  `usage_budget` (the fuzz budgets silently ran at the 60000ms default), flag
  names lose their leading dashes (`--all` and `--write` were unreachable),
  `usage_args` is shell-quoted so paths keep their spaces, and tasks without a
  `usage` spec get post-`--` argv appended to their shell bodies the way mise
  did (`lint:monochromatic-lint` was linting the whole repository instead of the
  paths `advisory-core` passes). Adds `package/dev-script/task-cli/task.ts`
  with `lint:types`; that task found three `parseUsageSpec` strictness errors,
  fixed in the same commit.
- `830d02b2cb` removes the tool-installing third-party actions: the composite
  action downloads Node and installs pnpm with `npm install --global`, the
  kotlin and music-player JDK jobs download the pinned
  `Temurin-21.0.12+8.0.LTS` archive, and `fs-id` reuses the composite action,
  so `actions/setup-node` and `actions/setup-java` are gone (the latter was
  also never SHA-pinned). The action writes `task-run` shims (bash plus
  `task-run.cmd`) into a `GITHUB_PATH` directory; the old
  `ln -sf ... /usr/local/bin` failed on Windows because that directory does
  not exist (run 40247019224).
- `f0c1309427` drops the empty `env:` keys the `MISE_*` removals left behind,
  which made `logger-fuzz.yml` and `toml-edit-fuzz.yml` invalid workflow files
  (every push to `main` failed at startup).

What to inspect:
- `rg 'actions/setup-node|actions/setup-java|mise-action|corepack' .github .depot`
  returns no matches.
- `node package/dev-script/task-cli/src/run.ts //package/module/logger.fuzz:fuzz -- --budget 1`
  prints `Fuzz campaign over 2 property files at 1ms each`, and
  `node package/dev-script/task-cli/src/run.ts lint:monochromatic-lint -- 'a b.txt'`
  reaches `monochromatic-lint` as one path word.
- A bare Namespace instance ran the whole setup recipe, the dependency step and
  the `logger-fuzz` bodies through the shim (`nsc` instance `no9tlksh0a3s0`),
  and a second instance installed the JDK (`lrscuvar00f7m`).
- `fs-id` re-runs its three-OS matrix on this push and exercises the new setup
  action and the task-run shims on Linux, macOS and Windows.

Completed follow-ups (2026-10-26, the user's three picks):
- `logger.fuzz`'s file-sink property is repaired (`299ce2fcd1` and the
  empty-batch fix after it). It was red since 2026-10-22 on three stale
  fixture assumptions around a sink whose 0.4.5 write path moved log files
  under a UTC day directory, dropped the verify probe record, and started
  honouring `MONOCHROMATIC_LOG_DIRECTORY` (which `test:unit` sets): the
  throwaway package now clears that override, the reader collects
  `*.log.jsonl` under the day directories and treats zero files as the empty
  round trip the sink documents, and the property expects no probe line. A
  four-case control (message empty or not, log-directory override on or
  off) pinned each cause; the sink itself was correct throughout.
- The `toml-edit.fuzz` coverage gate is green (`75a1f2dfb2`): the
  `src/parse-toml-edit.ts` 186 to 143 drop was the parser-error helper's move
  into `src/safe-parse.ts` (`e2924f2564`), and the same run covers both files
  at 100%, so the baseline was refrozen as the gate instructs for an intended
  drop.
- The `music-player` task ports are repaired (`e53e86bc80` and the
  empty-batch-adjacent fixes): node-inline bodies run under node again,
  `dir = {{config_root}}/rust` is honoured through the new `dir` field,
  `{{vars.android_clippy_dispatch}}` is inlined, `{{config_root}}` and
  `{{arg(...)}}` map to the `TASK_REPO_ROOT`/`TASK_PACKAGE_PATH` and
  `usage_*` forms, multi-line `run = [` arrays keep their steps (the port
  had collapsed them to zero, so `notices` silently did nothing), and Gradle
  packages resolve in `//package/...` names. The regenerated bodies come
  from `translate-tasks.ts` in
  `${HOME}/temp/agent/nsc-mise-elim/translate-tasks.ts`, kept for future
  mise ports.
- The user decided to keep the remaining platform actions (`actions/checkout`,
  `actions/cache`, the artifact and attestation actions, `changesets/action`,
  `rust-lang/crates-io-auth-action`, `github/codeql-action`,
  `ossf/scorecard-action`, `anthropics/claude-code-action`); they are
  SHA-pinned, user-approved and implement GitHub platform services.

Open items:
- `package/dev-script/task-cli` has no README, tests, `lint` or `build` tasks;
  only `lint:types` landed.
- `readonly-semantic-bridge` and `env-local-json` were red from this migration
  (Windows shim, missing `node_modules/.bin` on `PATH`) and re-run on their
  next path touch; their causes are fixed and proven.
- The action allow-list can drop `jdx/mise-action` and `actions/setup-node`;
  `actions/setup-java` never had a pinned SHA to list.
- One `logger.fuzz` file-sink property failure was observed once at 835 tests
  before the empty-batch fix and never reproduced in ten replays (eight 3s,
  one 30s, one 60s campaign); the empty-batch cause is fixed and covered.
  If the property fails again, keep the counterexample line.
