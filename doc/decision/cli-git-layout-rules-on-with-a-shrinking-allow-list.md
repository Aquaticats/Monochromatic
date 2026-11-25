# The git wrapper keeps the layout rules from now on, with today's loops on a list that only shrinks

Status: accepted by the owner on 2026-11-25, answering item 16 of `package/git-policy/cli/doc/debt-ledger-2026-11-25.md`.

## Decision

The layout rules `AGENTS.md` sets for every crate (LYD, a `uses/` folder of links in every source folder; LYL, no two folders
using each other; LYF, no loop among a folder's files) are switched on for `package/git-policy/cli` now, through the
repository linter's rules that enforce them (`rust/folder-uses-links` and `rust/file-order-in-folder`, which
`monochromatic-lint.config.jsonc` turns on only for the ZFS binding package today).
The loops and mutual uses that exist today go on an allow-list that may only shrink, so new code cannot add one, and later
rounds untangle the crate folder by folder.

The owner chose "Enable now, untangle over time (Recommended)" over untangling everything before other work lands, which would
pause the ZFS route, and over recording the crate as an exception.

## Why

The ledger measured, on 2026-11-25: 29 of the crate's 55 source folders in one strongly connected set, 97 pairs of folders
that use each other, and a `uses/` folder in 19 of the 55.
The base folder `foundation` uses six higher folders and is used by `landing` in 115 files and `worktree` in 142.
Switching the rules on with a shrinking list stops the growth at once, keeps the ZFS route (the owner's first priority)
moving, and ends with the rules holding.

## What it costs

An allow-list of today's pairs to keep until it is empty, `uses/` folders for the 36 folders that lack one, and the untangling
rounds after it.
