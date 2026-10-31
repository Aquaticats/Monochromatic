# Oxlint package tasks fail in a new linked worktree because it has no `node_modules`

## Symptom

In a new linked worktree under `.claude/worktrees/`,
`mise run //package/browserslist/core-rust:lint:scripts:oxlint bin/oracle-container.ts` stopped before linting.
Its first step builds the oxlint config bundle,
and that step failed:

```text
[//package/config/oxlint:build:js:node] $ rolldown --configLoader native --config rolldown.node.config.ts
 ERROR  Failed to load the config file "rolldown.node.config.ts" using the "native" config loader.
  [cause]: Cannot find package '@monochromatic-dev/config-rolldown' imported from
  .../package/config/oxlint/rolldown.node.config.ts
```

The emitting tool was rolldown 1.2.15 loading a config through Node.js module resolution.

## Root cause

A new worktree holds only tracked files,
so it has no `node_modules` directories,
including the workspace links such as `package/config/oxlint/node_modules/@monochromatic-dev/config-rolldown`.
Node.js resolution then walks up from the worktree into the main checkout's root `node_modules`,
which holds no link for a workspace package of another checkout's path.
The failure is therefore about the missing install,
not about the task or the rolldown version.

## Fix

Install into the worktree from the committed lockfile,
without lifecycle scripts so nothing outside the worktree changes:

```sh
# worktree root
pnpm install --frozen-lockfile --offline --ignore-scripts
```

This took seconds on 2026-10-30 because pnpm hard-links from its content store.
After a branch adds or changes a workspace dependency,
run `pnpm install --offline --ignore-scripts` again so the new links exist,
and commit the `pnpm-lock.yaml` change it makes.
The `prepare:pnpm:install` task runs a plain `pnpm install` with scripts,
which this note does not replace for a developer checkout.
