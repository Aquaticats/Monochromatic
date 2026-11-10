# Issue 397 checkout relocation: reviewed 2026-11-10

## Why

Issue 397 was queued against `package/async-time/core` as a `bug` naming a defect in that package.
Read in full (`gh issue view 397 --repo Aquaticats/Monochromatic --json body`,
no comments on the issue),
it proposes repository-wide checkout portability.
It names no defect in any package's code.
This records the verdict,
which of its claims still hold,
which citations have moved,
and what the work still needs.

## Verdict

Not a defect in `package/async-time/core`,
and not fixable from that package.
The issue asks for a new repository interface
(`mise run checkout:copy`,
`checkout:audit`,
`checkout:activate`),
for remediations in other packages,
for edits to user-global configuration outside the repository,
and for one blocking human action.
It needs the owner.

## Where the issue names this package

The issue names this package in one place:
the closing probe of its reproduction,
`mise run //package/module/async-time:buildAndTest`,
which reported the package building and both unit suites passing after a copy.
That path no longer exists.
Commit `592d78b143` (2026-10-29,
`refactor(*): eliminate the module, rust-module, shim, stub, dev-script, desktop-app, and remaining generic categories`)
moved it to `package/async-time/core`,
package `@monochromatic-dev/async-time-core`.
The suite holds three tests today
(`withTimeout`,
`wait`,
`settleWithin`),
all passing.

## Claims that still hold

### Repository-local Git configuration carries a credential

`.git/config` still holds exactly one `url` entry with userinfo.
Only the count was checked;
the value was not read into this record.

### Tracked operational files carry machine paths

- `.idea/gradle.xml` still stores an absolute module path,
  which now names `packages/linter/kotlin`,
  a directory that does not exist.
- `.idea/vcs.xml` still maps an absolute directory alongside `$PROJECT_DIR$`.
- `package/hall-monitor/daemon/hall-monitor.desktop` line 5 still launches
  `/var/home/user/Monochromatic/package/hall-monitor/daemon/hall-monitor`.
- `package/hall-monitor/daemon/src/analyze/llama.ts` lines 15,
  20 and 25 still hardcode a model,
  a projector and `llama-server` under `/var/home/user`.

### Nested worktrees still point at the main common directory

Of the 220 entries under `.claude/worktrees`,
214 hold a `.git` file naming this repository's common directory,
188 of them spelling it
`/var/mnt/encrypted/repository/Monochromatic/.git/worktrees/<name>`
and 26 spelling it
`/var/home/user/Monochromatic/.git/worktrees/<name>`,
which is the same directory reached through a symlink.
The remaining 6 entries hold no `.git` file.
A copied tree therefore keeps writing to the source repository's administration.

## Claims that have moved or no longer hold

- The `translation-repair` nested worktree the issue names is gone.
  The class of nested worktree it stands for remains.
- `package/desktop-daemon/hall-monitor` is now `package/hall-monitor/daemon`.
- `package/figma/kiwi/src/verify.ts` derives its export directory from `homedir()`,
  so "hardcodes one user's directory" no longer holds,
  though the data still lives outside the repository.
- `package/module/async-time` is now `package/async-time/core`.

## Existing owner for part of the proposal

`package/git-policy/cli/src/native/worktree_copying/` already owns ignored-state selection,
copy-on-write staging,
install with exact existing-entry checks and rollback,
journals,
recovery and publication,
recorded in `doc/handover/cli-git-native-worktree-copy.md`.
`AGENTS.md` rule `RCI` says to extend an owner that already holds a responsibility
rather than adding a second one,
so the shape of `checkout:copy` is the owner's call.

## Decisions the work needs

- Rotate the credential in the repository-local Git configuration,
  or decide `checkout:audit` may ship before rotation.
  The issue makes rotation a prerequisite.
- Say whether `checkout:copy` extends `worktree_copying` or becomes a new package,
  and where `checkout:audit` and `checkout:activate` live.
- Choose the contents of the `--state=developer` ignored-state profile.
- Authorize rewriting user-global integration files
  (`~/.pi/agent/settings.json`,
  `~/.claude/settings.json`)
  and the desktop or systemd entries,
  or keep those manual.
- Approve the `.idea`,
  Hall Monitor and Kiwi remediations,
  which land in those packages.

## Verification

Package checks on merged `main` (`6e2333edc7`),
each through `mise run //package/async-time/core:<task>`
in a `systemd-run --user --scope` with `CPUQuota=300%`,
`MemoryMax=8G`,
`TasksMax=512`:

- `build` builds `dist/final/neutral/index.mjs` and the declarations.
- `test:unit` runs the `withTimeout`,
  `wait` and `settleWithin` suites,
  all passing.
- `lint:types` exits 0.
- `lint:oxlint` reports `Found 0 warnings and 0 errors.`
