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
- New shared composite action `.github/actions/task-cli-setup` that pins
  Node and pnpm to the `mise.lock` versions and exposes `task-run` on `PATH`.
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
