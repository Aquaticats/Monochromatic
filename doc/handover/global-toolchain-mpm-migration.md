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
- `toolchain:status:global` reports `59 of 60` in sync;
 the one drift is the `npm:typeslayer` pin below.
- `mise self-update --yes` exits clean and `mise plugins ls` is empty
 (the 2026-10-27 incident's user-boundary check,
 now true after the cutover),
 and the `linked-worktree-only` guard still rejects `git checkout --force` in
 main worktrees through `~/.local/bin/git`.
- Old Mise installs stay as the rollback.

## Open items

- `npm:typeslayer` pins `0.1.32-aquaticat.0319832`,
 which the public registry no longer carries
 (unpublished;
 33 versions listed,
 none of them `aquaticat`).
 The retired Mise install still runs it.
 The maintainer chooses:
 keep the pin and the old install as its only source,
 repin to `0.1.32`,
 or republish the prerelease and re-run `toolchain:sync:global`.
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
