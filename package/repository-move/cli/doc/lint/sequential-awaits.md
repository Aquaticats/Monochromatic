# `no-await-in-loop` suppressions in the repository move tools

## Rule and configuration

`package/config/oxlint/src/rule/correctness.ts` enables `eslint/no-await-in-loop` as a warning, and this package's lint
exits 1 on any warning.
The Oxlint rule has no options and no allow-list (the same finding is recorded in
`package/pi-plugin/advisor/lint-boundaries.md`, which inspected the rule source and its page).
Turning it off for this package would loosen a lint rule, which needs prior approval,
so each loop that must wait for one step before the next carries one statement-scoped `oxlint-disable-next-line` comment
on the `await` line.
Every independent set of operations in the package already runs with `Promise.all` instead.

## Where the order is the design

- Copying subvolumes (`src/copy-pass.ts`):
  a subvolume's snapshot, send, receive and placement are recorded before the next subvolume starts.
  The order names the copies (`s000`, `s001`, ...),
  every send names the earlier copies as clone sources,
  the free-space guard reads the destination between two subvolumes,
  and one total deadline bounds the pass,
  so a stop leaves every earlier subvolume recorded and a rerun continues.
  Running them together would break all four.
- External folders (`src/external-pass.ts`):
  each folder is created, copied, checked for deletions and recorded in turn under one total deadline.
- Freeze and thaw (`src/freeze-pass.ts`):
  one `btrfs property set` or read-back at a time, in manifest order.
  The runner prints each command as it runs it, so a parallel run would print a different order than it ran.
- Verification (`src/verify-pass.ts`):
  each `rsync` reads two trees from the same disks, so parallel runs would only slow each other,
  and the record written after each one lets a rerun resume.
- Stages of the guided runner (`src/guided-engine.ts`):
  a stage depends on what the earlier stage did and the run stops at the first failure.
- The scan (`src/uncommitted-work-table.ts`):
  one `git status` at a time keeps the load of the scan bounded while agents work in the worktrees.
- Test worktrees (`src/move-fixture.ts`):
  `git worktree add` holds a lock on the repository.

## Why the other shapes do not work

- Recursion keeps a frame per subvolume for a run that lasts hours.
- A promise chain is forbidden by the repository rule that allows `async` and `await` only.
- `Promise.all` over the subvolumes starts every send at once, which the deadlines, the clone sources and the free-space
  guard all forbid.
