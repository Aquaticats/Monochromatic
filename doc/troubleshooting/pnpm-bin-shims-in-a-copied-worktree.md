# A worktree's `node_modules/.bin` shims name a store path counted from the checkout that wrote them

Observed on 2026-11-23 in the worktree
`.claude/worktrees/pi-driver-launcher-takes-the-first-open-pool`,
a `git worktree` of `/home/user/Monochromatic`,
with `oxlint` 1.94.0 and `oxlint-tsgolint` 7.0.2008
under the store root `/home/user/.local/share/pnpm/store/v11`.

## Symptom

`mise run //package/pi-shared/subagent-driver:lint:oxlint` failed
with a module path that does not exist:

```text
Error running tsgolint: "exit status: exit status: 1"
Error: Cannot find module '/home/user/Monochromatic/.claude/worktrees/.local/share/pnpm/store/v11/links/@/oxlint-tsgolint/7.0.2008/<hash>/node_modules/oxlint-tsgolint/bin/tsgolint.js'
```

`/home/user/Monochromatic/.claude/worktrees/.local` is nobody's store:
the store is `/home/user/.local/share/pnpm/store/v11`.

`lint:types` and `build` in the same worktree were unaffected.

## What was probed

`node_modules/.bin/oxlint` there is a regular file
(a pnpm cmd-shim), not a symlink, and its last line records the target it wants:

```text
# cmd-shim-target=/home/user/.local/share/pnpm/store/v11/links/@/oxlint/1.94.0/<hash>/node_modules/oxlint/bin/oxlint
```

The command it actually runs names the same store with a relative climb:

```sh
exec node "$basedir_abs"/'../../../.local/share/pnpm/store/v11/links/@/oxlint/1.94.0/<hash>/node_modules/oxlint/bin/oxlint' "$@"
```

`basedir_abs` is the shim's own folder, `<checkout>/node_modules/.bin`.
Three levels up from there is `/home/user/` only when the checkout sits directly under the home folder.
Here it is `/home/user/Monochromatic/.claude/worktrees`, so the climb lands on
`/home/user/Monochromatic/.claude/worktrees/.local/share/pnpm/store/...`.

Nine of the thirty-nine shims in that folder name a store path in a relative climb,
and each of the nine climbs resolved from that folder to a path that does not exist there.
The other two shims that resolve to nothing are workspace commands whose `dist` was never built here.
Worktrees whose `oxlint` is 1.93.0 carry the same store target with six levels
(`'../../../../../../.local/share/...'`), which resolves from a worktree as well as from the main checkout;
two worktrees made on 2026-11-23 carry three levels with 1.94.0, like this one.

## Mechanism

pnpm's cmd-shim writes the target as a path relative to the folder that ran the install,
so the number of `../` counts the depth of that folder below the home folder.
A shim written by an install in `/home/user/Monochromatic` therefore climbs three levels,
and a copy of it inside a worktree three levels deeper points at `.../worktrees/.local/...`.
The shim's own `cmd-shim-target=` comment stays correct,
which is why the failure names one path and the file names another.

## Repair

Rewrite the climb in the shim to the number of levels the worktree needs,
keeping the target and everything else the same:

```sh
# inside the worktree, for the two shims this task needs
node --input-type=module -e 'const fs = await import("node:fs");
for (const name of ["oxlint", "tsgolint"]) {
  const file = `node_modules/.bin/${name}`;
  const before = fs.readFileSync(file, "utf8");
  fs.writeFileSync(file, before.split("../../../.local/share/pnpm/store").join("../../../../../../.local/share/pnpm/store"));
}'
```

`./node_modules/.bin/oxlint --version` then answers `Version: 1.94.0` and
`./node_modules/.bin/tsgolint --version` answers its own unsupported-entrypoint warning instead of `MODULE_NOT_FOUND`;

Rerunning `pnpm install` in the worktree writes the shims for that depth itself,
which is the repair to prefer where an install is allowed.
That round was told not to run `pnpm install` in the worktree (issue 6339),
so the shims were rewritten as above, and the seven other shims carrying the same climb were left as they were.

## What else that folder lacked

Recorded because they were met in the same session and named different faults:

- `mise run //package/pi-shared/subagent-driver:format:dprint` answered
  `No files found to format with the specified plugins at <package folder>.`
  so no dprint formatting could run there; `format:oxlint` did run.
- The repo-wide Markdown linter `monochromatic-lint` has no executable in that worktree,
  so the README's Markdown was not linted.

## What is not established

Which program copied `node_modules` from the main checkout into the worktree.
No task in this repository's `mise.toml` copies it,
so the copy is made by whatever created the worktree.
