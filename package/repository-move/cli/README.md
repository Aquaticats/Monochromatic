# repository-move-cli

One-off tools for moving the repository and its worktree subvolumes to the 860 drive.
The design and the reasons are in `doc/planning/repository-on-the-860-drive.md`;
the manual for running them is `doc/runbook/repository-on-the-860-drive-guided-runner.md`.

## Tools

- `src/move-repository.ts` (bin `repository-move-copy`):
  the fail-closed copy by `btrfs send` and `btrfs receive`.
  Without `--run` it prints every step and runs nothing.
  Passes:
  `first` and `final` (copy),
  `freeze` and `thaw` (read-only flag with a manifest),
  `external-first` and `external-final` (plain folders by `rsync`).
- `src/list-uncommitted-work.ts` (bin `repository-move-scan`):
  a table of what a move would strand in every worktree.
- `src/run-repository-move.ts` (bin `repository-move-run`):
  the guided runner the coordinator runs:
  `prepare`,
  `cutover`,
  `thaw`,
  `rollback` and `status`.

The tools import only node's own modules,
so they run from the TypeScript source (`node src/move-repository.ts`),
with the absolute path of node when a root step runs them
(`sudo node` fails when node comes from the version manager).
Run `mise run //package/repository-move/cli:build` and `mise run //package/repository-move/cli:test:unit` for the built
bins and the tests.

## Records

Every record is keyed by the subvolume's UUID plus its path.
A resume after worktrees were added,
removed,
moved or replaced continues each subvolume's own records and prints what changed;
a copy name (`s000`,
`f000`) is never reused.
The UUIDs come from `btrfs subvolume list -u` (root only).
Its `path` column is relative to the parent named by `top level`,
so the tool rebuilds each full path from the parent chain.

## Deadlines

Each transfer has a deadline,
the whole run has a total deadline,
and every quick command has its own.
A transfer gets the smaller of its deadline and what remains of the total.

## Unverified

The real `btrfs send` and `receive`,
and the output of `btrfs subvolume list -u` for a subvolume nested in another nested subvolume,
were not run here (they need root).
The tests use a btrfs test double (`src/btrfs-test-double.ts`).
