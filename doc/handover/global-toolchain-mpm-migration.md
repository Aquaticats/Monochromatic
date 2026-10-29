# Global toolchain mpm migration close-out

Status:
 2026-10-28,
 the machine-global toolchain migration landed
 (`doc/planning/mpm-global-toolchain-migration.md`,
 `doc/decision/mpm-toolchain-adoption.md` section `Machine-global scope`).

## Done on the maintainer's machine

- `~/.config/mise/config.toml` lost `[tools]`,
 `[settings.pipx]`,
 and `[settings.npm]`;
 it keeps `[env]`
 (`GH_PATH` repointed at the farm),
 the `_.path` farm entry,
 and `[settings]`.
- `mise run toolchain:sync:global` provisions the machine-local stack in
 `~/.config/monochromatic-dev/toolchain/`
 (plan,
 lock,
 mpm manifests)
 into `~/.local/share/monochromatic-dev/toolchain-global/`,
 exposed through `exposure/`
 (142 links).
- `toolchain:status:global` reports `59 of 59` in sync.
- `mise self-update --yes` exits clean and `mise plugins ls` is empty
 (the 2026-10-27 incident's user-boundary check,
 now true after the cutover),
 and the `linked-worktree-only` guard still rejects `git checkout --force` in
 main worktrees through `~/.local/bin/git`.
- Old Mise installs stay as the rollback.

## Open items

- `npm:typeslayer` was removed from the global plan at the maintainer's
 decision ("Remove it entirely",
 2026-10-28):
 the pinned build `0.1.32-aquaticat.0319832` is unpublished
 (33 registry versions,
 none of them `aquaticat`),
 so nothing under the lock can install it.
 Its retired Mise install stays under the standing
 "leave the old installs" decision;
 deleting it would be unrecoverable,
 which the maintainer may still order.
- The VM-builder and Containerfiles still seed a global Mise config with
 `[tools]`
 (`doc/planning/mise-removal-coverage.md`,
 `Editors, VM images, and containers`);
 they should seed the machine-local plan shape instead.
- macOS and Windows provisioning parity stays open
 (issues 1797 and 1798),
 for both scopes.
- `update.resolvers.ts` grew past the max-lines preference
 (1315 lines);
 a sibling split is a candidate follow-up.

## Machine facts worth keeping

- `bin install` honors a `.gz` payload's gzip header name,
 and `--force` renames a colliding directory aside to `.<name>.old`
 (the tree-sitter incident recorded in the planning spec);
 `tree-sitter` now rides the archive route.
- Two scopes must not share one `bin` install database;
 each runs `bin` with `BIN_CONFIG=<toolchainRoot>/bin.config.json`.
