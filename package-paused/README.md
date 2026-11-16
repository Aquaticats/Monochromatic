# packages-paused

Packages here are paused: temporarily set aside, not abandoned.
They are kept in the repository for reference and easy resumption,
but they are not part of the active workspace.

This differs from `packages-deprecated/`, which is for packages that stay
installable for outside consumers but are no longer maintained
(see `doc/howto/deprecate-package.md`).
Paused means the team intends to come back to it; deprecated means it is done.

## What "paused" means operationally

Unlike `packages/*/*` and `packages-deprecated/*/*`, the `packages-paused/`
tree is deliberately not wired into:

- `pnpm-workspace.yaml` globs, so paused packages are not workspace members:
  pnpm does not install or link them, and their `workspace:*` dependencies do
  not resolve.
- `mise.toml` `config_roots`, so their mise tasks are not discoverable and they
  drop out of the `//packages/...` build, lint, and test fan-out.

A package may be paused only when nothing in the active workspace depends on it.
Verify with a reverse-dependency search before moving one in.

## How to pause a package

1. Confirm no active package depends on it
   (`rg '"@monochromatic-dev/<name>"' packages/*/*/package.json`).
2. `git mv packages/<category>/<name> packages-paused/<category>/<name>`
   (or move a whole `<category>` directory at once).
3. Remove any root-level convenience tasks, env entries, or deploy config that
   reference the moved package, then regenerate derived files
   (`mise run prepare:pnpm:install` to reconcile the lockfile,
   `mise run sync:files` to regenerate `mise.toml` and `CLAUDE.md`).

## How to un-pause a package

1. `git mv packages-paused/<category>/<name> packages/<category>/<name>`.
2. Recreate any root-level convenience tasks that were removed when it was paused
   (for example the `forge:*` and `prepare:garage` tasks for `webapp-forge`).
3. `mise run prepare:pnpm:install` to relink it as a workspace member,
   then `mise run sync:files`.

## Currently paused

- `webapp-content/` (`messages-demo`)
- `webapp-search/` (`ai-tree`, `exa-search`)
- `webapp-forge/` (`seed`, `server`, `stress`)
- `webapp-edu/` (`paper2vn`)
- `stylesheet/` (`monochromatic`)
- `desktop-daemon/editord/`
- `hall-monitor/` (`daemon`)
