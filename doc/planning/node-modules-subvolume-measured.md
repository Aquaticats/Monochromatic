# Whether `node_modules/.monochromatic` can be a btrfs subvolume

Status:
measurement,
2026-11-10T13:17:53-05:00,
branch `docs/planning-node-modules-subvolume-measured` at `62994776ec`,
on a throwaway clone and never the main checkout,
no repository file changed and no tool built outside that clone,
Linux only,
and `package-paused/` untouched.

The question this answers is the first of the two the owner was handed in
[what a worktree add pays for the directories its snapshot leaves out](cli-git-worktree-add-cost-of-its-trash.md),
under the coordinator's pick in ["Picks made for the owner, veto open"](../handover/questions-for-the-owner.md):

> Whether `node_modules/.monochromatic` can be a subvolume is measured on a throwaway checkout before anything changes:
> it would remove roughly 165,000 of the 269,000 deletions a snapshot's trash costs,
> and nothing yet shows that the tools leave a subvolume in place.

So the subject here is only the second half of that sentence:
whether the tools that write the directory leave a subvolume in place,
or replace it with a plain folder.
The snapshot's own behaviour was measured earlier and is re-confirmed here as a control.

## How it was measured

The checkout:
a `git clone --single-branch --branch docs/planning-node-modules-subvolume-measured` of this worktree,
23,006 tracked files plus 233 MiB of LFS content,
`.git` holding 1,010 MB of hardlinked objects,
at `${HOME}/temp/agent/node-modules-subvolume-probe/clone`,
on the btrfs volume `/dev/mapper/crypt_sda` mounted at `/var/mnt/encrypted`,
kernel `7.2.15-ogc1.1.fc45.x86_64`.
Its push URL was set to `DISABLED` with `git remote set-url --push origin DISABLED` before anything ran,
so nothing in it can reach a remote.

The arrangement:
a subvolume stood where a plain directory stands today before the first install,
and before every arm that follows the directory itself,
except the last,
which measured what a writer does when it is absent.

```sh
cd -- "$HOME/temp/agent/node-modules-subvolume-probe/clone"
mkdir --parents node_modules
btrfs subvolume create node_modules/.monochromatic      # before the first pnpm install
```

Every arm that starts from a copy of the arrangement made it again the same way,
at an empty path.

The reading:
a btrfs subvolume root reports inode 256 and a device ID of its own,
which is what `package/git-policy/cli/src/native/subvolume/fs_kind.rs` decides with
(`BTRFS_SUBVOLUME_ROOT_INODE`).
Every arm reads it with `stat --format='inode=%i dev=%d %n'` before and after.

The instrument was shown able to tell the two apart in both directions:
the same `stat` call read `plain-directory inode=505313385` for a plain directory created beside the first
subvolume and `subvolume inode=256 dev=2097266` for the subvolume,
and the fixture arms below read `plain-directory` for their controls.
A tool that swapped the arrangement for a plain directory would therefore change both numbers.

A canary went further:
a file `.probe-canary.txt` was written inside the root subvolume and inside a package's one,
and the arms that followed were read against whether the canary still existed,
which no reused inode could fake.

Every command ran under `timeout` and inside its own cgroup:

```sh
systemd-run --user --scope --collect \
  --property=CPUQuota=... --property=MemoryMax=... \
  --property=TasksMax=... --property=IOWeight=... -- <command>
```

and each arm states its exit status.

The probe scripts are under `${HOME}/temp/agent/node-modules-subvolume-probe/`:
`launcher-sweep.ts`,
`deleter-fixture.ts`,
`count-entries.ts`.
They are scratch,
not repository files.

## `pnpm install`

Through the repository's own launcher,
`mise exec -- pnpm install --offline --frozen-lockfile`,
pnpm 12.18.0,
with the subvolume already standing in an otherwise empty `node_modules`.

The cold install created every dependency and left the arrangement alone
(the block quotes its last lines):

```text
Scope: all 241 workspace projects
…
Done in 12s using pnpm v12.18.0
PNPM_INSTALL_EXIT=0
AFTER inode=256 dev=2097266 node_modules/.monochromatic
```

`node_modules/.monochromatic` received no entry at all from that install:
`ls -A` printed nothing.

The repeat install,
the shape most commands actually pay for,
did the same:

```text
BEFORE inode=256 dev=2097266 node_modules/.monochromatic
Scope: all 241 workspace projects
[WARN] There are cyclic workspace dependencies: …/package/async-time/core, …/package/test-harness/core, …/package/logger/core
✓ Lockfile passes supply-chain policies (verified 6h ago)
Lockfile is up to date, resolution step is skipped
Already up to date
Agent skills awaiting approval: @optique/core.
Run "pnpm approve" to review them.
Done in 14.8s using pnpm v12.18.0
PNPM_INSTALL_REPEAT_EXIT=0
AFTER inode=256 dev=2097266 node_modules/.monochromatic
```

A third install ran with the canary file in place,
and both the canary and the inode survived it:

```text
PNPM_INSTALL_CANARY_EXIT=0
=== root canary
canary
root inode=256 dev=2097266
```

Nothing in pnpm's own behaviour was adjusted for this:
the installs are the repository's ordinary offline frozen-lockfile installs.

## The mpm launchers

The launchers in `.cache/toolchain-launchers/` are why the owner's brief names them.
Measured one by one with `--version`,
from the clone's root,
reading the arrangement before and after each:

```text
before: subvolume inode=256 dev=3145940
cargo-shadow: exit 0, first line "cargo-shadow 0.1.5-alpha.9"; .monochromatic subvolume inode=256 dev=3145940; entries 0 -> 0
file-enforcer: exit 0, first line "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"version\",\"code\":\"version\",\"message\":\"file-enforcer 0.1.3\"}"; .monochromatic subvolume inode=256 dev=3145940; entries 0 -> 0
monochromatic-git-policy-cli: exit 0, first line "git version 2.56.0"; .monochromatic subvolume inode=256 dev=3145940; entries 0 -> 0
monochromatic-lint: exit 0, first line "monochromatic-lint 0.5.2-alpha.8"; .monochromatic subvolume inode=256 dev=3145940; entries 0 -> 0
mpm-shadow: exit 0, first line "mpm-shadow 0.2.7-alpha.10"; .monochromatic subvolume inode=256 dev=3145940; entries 0 -> 0
pnpm-shadow: exit 0, first line "pnpm-shadow 0.1.3-alpha.1"; .monochromatic subvolume inode=256 dev=3145940; entries 0 -> 0
search-shadow: exit 0, first line "search-shadow 0.2.3-alpha.1, a shadow of rg, grep, egrep, fgrep, ugrep, ug, ag, ack, find, fd, fdfind, bfs, tree, du, ncdu, tokei, cloc"; .monochromatic subvolume inode=256 dev=3145940; entries 0 -> 0
final: subvolume inode=256 dev=3145940
```

`monochromatic-nested-wayland-session` was left out of that sweep because it opens a window,
and this machine's desktop shows nothing.

The launcher install path itself,
`mise run toolchain:install`,
which runs the `mpm-shadow` launcher the lock names:

```text
toolchain install: platform linux-x64, root /home/user/.local/share/monochromatic-dev/toolchain, exposure …/clone/.cache/toolchain-bin
toolchain install: 0 installed, 66 unchanged, 0 failed
toolchain install: exported to mise: ANDROID_HOME, ANDROID_SDK_ROOT, JAVA_HOME, RUSTUP_TOOLCHAIN
TOOLCHAIN_INSTALL_EXIT=0
inode=256 dev=2097266 node_modules/.monochromatic
```

That run built the clone's own `.cache/toolchain-bin` and `.cache/toolchain-launchers`,
so it did real work in the checkout,
and it wrote no entry into the arrangement either.

The reason is where the writing lives:
every one of those launchers is a Rust program
(a 5.4 MB ELF under each name,
with the `.launch` JSON beside it naming what it runs),
and the code that creates `node_modules/.monochromatic` is the TypeScript logger,
`package/logger/core/src/sink/file-destination.ts`,
which creates it with `mkdir(directory, { recursive: true })` and
`package/logger/core/src/sink/file-open.ts`,
which removes only its own probe file.
A search over the 4,066 `.rs` files finds the path in tests,
in the leave-out's prune rule and in an ELF section name,
never in a launcher that writes it.

## A cold `mise run`

Three arms,
each a first run in this clone.

`mise run //package/logger/core:build`:
exit 0,
both the root arrangement and the package's own one unchanged,
and no entry written.

`mise run //package/logger/core:test:unit`:
the first attempt failed before running anything,
because the clone had not been built (the path is abbreviated):

```text
Error [ERR_MODULE_NOT_FOUND]: Cannot find module '…/clone/package/logger/core/dist/final/node/index.mjs'
TEST_UNIT_EXIT=1
```

After the build it ran the suite and failed one test file,
`package/logger/core/src/startup.unit.test.ts`
("delivers startup records to file sink without awaiting initPromise"),
exit 1.
That file's failure is not this page's subject and was not diagnosed;
the other test files of the run reported their passes.
What matters for this measurement is that it wrote nothing into either arrangement,
and the reason is worth recording:
the test runner points `MONOCHROMATIC_LOG_DIRECTORY` at `${HOME}/temp/agent/module-test-logs/<stamp>-<pid>`
(`mise.toml`,
`testRunEnv`),
so repository tests never write `node_modules/.monochromatic` at all.

`mise run //package/const/core:lint:oxlint` is the arm that does write it,
because the type-aware Oxlint plugins are TypeScript and log through the repository's logger.
With a subvolume at `package/const/core/node_modules/.monochromatic`:

```text
BEFORE const pkg inode=256 dev=3145945
Found 6 warnings and 5 errors.
Finished in 1.9s on 14 files with 490 rules using 3 threads.
CONST_LINT_OXLINT_EXIT=1
AFTER const pkg inode=256 dev=3145945
=== entries
2026-11-10
.logger-retention-sweep
=== nested
2026-11-10T18-17-43.905Z.30e69bc4-eb58-44ea-ad8f-bc39ffd09a8c.log.jsonl
```

The subvolume stayed a subvolume,
the logger created its day directory and its file inside it,
and the retention sweep left its own stamp inside it,
which is the sweep running against that directory without replacing it.

The exit 1 is the clone's,
not the arrangement's:
the same task in this worktree printed `Found 0 warnings and 0 errors.` and exited 0,
and the clone's findings come from packages the clone never built.

A second lint run with a canary inside the package's arrangement left both intact:

```text
CONST_LINT_CANARY_EXIT=1
=== const canary
canary
const inode=256 dev=3145945
```

## The repository's own logger at a checkout's root

`mise run` starts monorepo tasks in a package,
so a root-cwd task was not what writes the root directory.
The writer was run directly instead,
from the clone's root,
so that `node_modules/.monochromatic` is the destination the logger resolves:

```sh
node --input-type=module-typescript -e "import { logger } from '@monochromatic-dev/logger-core/ts'; logger.info('…'); await logger.flush()"
```

```text
BEFORE root inode=256 dev=2097266
[info] [2026-11-10T18:18:34.743Z] probe: one record from the clone root, written by the repository own logger
logged from …/clone
ROOT_LOGGER_EXIT=0
AFTER root inode=256 dev=2097266 node_modules/.monochromatic
=== entries
2026-11-10
.logger-retention-sweep
=== day contents
2026-11-10T18-18-34.751Z.20576e5e-94f7-45d1-9360-fbbe96c761ee.log.jsonl
```

The arrangement took the records and stayed a subvolume.

## What deletes and what moves

Measured in a fixture of its own,
`deleter-fixture.ts`,
which builds a fresh project per arm with the same 200 entries inside,
and whose first arm is the control that the deleter still deletes a plain directory.
Abbreviated only where a path repeats:

```text
A rm -rf node_modules, plain directory (control): exit 0; node_modules absent
B rm -rf node_modules, subvolume inside: exit 0; node_modules absent; .monochromatic absent
C rm -rf node_modules/.monochromatic alone, subvolume: exit 0; .monochromatic absent
D rmSync(node_modules, recursive, force) of mise task fix:reinstall, subvolume inside: no throw; node_modules absent; .monochromatic absent
E rename .monochromatic into a trash beside it, subvolume: no throw; source absent; trash subvolume inode=256 entries=1
F rename .monochromatic into a trash beside it, plain directory (control): no throw; source absent; trash plain-directory inode=505939635 entries=1
G btrfs subvolume delete of the subvolume, as the owning user: exit 0, output "Delete subvolume 74918 (no-commit): '…'"; .monochromatic absent
I btrfs subvolume create onto an existing plain directory, empty then holding a file: empty: exit 1, stderr "ERROR: Could not create subvolume: File exists"; non-empty: exit 1, stderr "ERROR: Could not create subvolume: File exists"
J btrfs snapshot of a tree holding a nested subvolume: source nested subvolume inode=256 entries=1; snapshot plain plain-directory inode=257 entries=1; snapshot nested plain-directory inode=2 entries=0
K rename the empty stand-in a snapshot shows where a nested subvolume stands: code ENOTEMPTY, message "ENOTEMPTY: directory not empty, rename …"; snapshot nested still plain-directory inode=2 entries=0; trash absent
```

What those settle:

- On this kernel,
  `rm -rf` of a directory above a subvolume removes the subvolume too,
  so nothing has to delete the arrangement by name.
- The same is true of `rmSync(…, { recursive: true, force: true })`,
  which is the first line of the repository's `fix:reinstall` task in `mise.toml`,
  and the owning user can also end it with `btrfs subvolume delete` without root.
- A subvolume can be renamed within its own tree (arm E),
  so a trash rename of the arrangement itself is not refused.
- A snapshot shows an empty stand-in where the nested subvolume stands (arm J),
  and that stand-in cannot be renamed even though it lists no entries (arm K);
  the leave-out already knows this,
  `package/git-policy/cli/src/native/worktree/worktree_trash.rs`
  ("A btrfs stand-in holds nothing and btrfs refuses to rename it").
- `btrfs subvolume create` refuses a path that already exists,
  empty or not (arm I),
  so an arrangement can only be made at an empty path:
  an existing populated `.monochromatic` has to be copied into a fresh subvolume,
  never converted in place.

The cost of making the arrangement,
measured twice in the same arm:

```text
H mkdir, then the same path as an empty subvolume, 100 times each: mkdir 3.7 ms, btrfs subvolume create 502.3 ms for 100 each, so 5.02 ms against 0.04 ms each
H mkdir, then the same path as an empty subvolume, 100 times each: mkdir 2132.1 ms, btrfs subvolume create 4211.3 ms for 100 each, so 42.11 ms against 21.32 ms each
```

The host's load moved both numbers by more than the difference between them,
so no ratio beyond "a few milliseconds against a few hundredths" is claimed.

The repository's own `fix:reinstall` task,
run in the clone with the arrangement at the root and with `pnpm_config_offline=true`,
so its own `pnpm install` could finish offline:

```text
Done in 16.7s using pnpm v12.18.0
FIX_REINSTALL_EXIT=0
=== root after
stat: cannot statx 'node_modules/.monochromatic': No such file or directory
```

It succeeded,
and it removed the arrangement with `node_modules`.
Nothing then recreated it:
an install does not create that directory.

What a writer does when the directory is absent is the counter-case to everything above,
and it is the one way a plain folder replaces the arrangement:

```text
$ rm -rf node_modules/.monochromatic     # the state fix:reinstall left
$ node --input-type=module-typescript -e "…logger.info(…); await logger.flush()"
[info] [2026-11-10T18:27:46.372Z] probe: one record where the logger found no directory at all
RECREATE_EXIT=0
AFTER inode=505984460 dev=108 node_modules/.monochromatic
2026-11-10
.logger-retention-sweep
```

The logger made a plain directory there,
on the parent's own device and not inode 256,
because that is what `mkdir(dir, { recursive: true })` does.
The same is true of the tools that take that directory as their scratch parent
(`package/git-clone-size/cli/src/temp-dir.ts`,
`package/mvm/cli/src/scratch-parent.ts`,
`package/search-fetch/core/src/scratch-parent.ts`,
`package/mutation-test/cli/src/scratch-parent.ts`,
`package/open-code-review-issue/cli/src/scratch-parent.ts`,
`package/deps-update/cli/src/scratch.ts`),
which create their own subdirectory inside it,
as read from those sources rather than run here.
So the arrangement survives every tool that finds it and is lost by any command that runs after something removed it.

## Today's counts

Measured over the main checkout read-only,
with `count-entries.ts`
(215 directories,
a 1.6 s walk):

```text
directories holding a .monochromatic under a node_modules: 215
their entries together: 168825
   136192 package/git-policy/cli/node_modules/.monochromatic
    28941 node_modules/.monochromatic
      686 package/claude-code-plugin/source/node_modules/.monochromatic
      276 package/oxlint-plugin/prefer-readonly-parameter-type/node_modules/.monochromatic
```

The unfinished walk in
[what a worktree add pays for the directories its snapshot leaves out](cli-git-worktree-add-cost-of-its-trash.md)
counted 165,161 entries in these directories at 40% of its walk on 2026-11-10;
the complete count today is 168,825,
so that figure holds.

One finding changes what the largest single number means:
135,889 of the 136,192 entries under the largest directory sit in one subdirectory named `git-policy/`,
a tool's scratch tree that shares the directory with the logger's records,
and not in day directories of `*.log.jsonl` files.
The logger's own day directories in that directory on 2026-11-10 held 295 entries across six days,
plus one retention stamp.
Whether the leave-out should move the scratch tree with the records is a separate question from this page.

## What this settles

- None of the three writers replaces the arrangement with a plain folder:
  the cold and repeated `pnpm install`,
  every launcher,
  the launcher install,
  a cold `mise run` of a build,
  a test and a lint,
  and the repository's logger itself all left the subvolume a subvolume.
- Two of those write nothing into it at all:
  `pnpm install` and the launchers.
  Every writer of that directory is TypeScript:
  the logger,
  and the tools that take it as their scratch parent.
  The tool that does write it thousands of times,
  the type-aware Oxlint plugins,
  keeps it.
  Repository tests write elsewhere on purpose:
  their runner points `MONOCHROMATIC_LOG_DIRECTORY` at agent scratch.
- A writer that finds the directory missing makes a plain directory,
  not a subvolume,
  so the arrangement depends on nothing removing it between runs.
- The arrangement does not break the deleters this repository has:
  `rm -rf`,
  `rmSync` in `fix:reinstall`,
  a trash rename of the arrangement,
  and `btrfs subvolume delete`
  all worked on this kernel.
- A subvolume cannot be made at an existing path,
  so any route has to create it empty,
  once,
  and every later route has to keep it through the tools above.

## What this does not settle

- Any kernel other than `7.2.15-ogc1.1.fc45`:
  whether `rm -rf` over a subvolume,
  the `rmSync` of `fix:reinstall`
  and the rename in arm E all behave the same elsewhere was not measured,
  and the repository supports more than one Linux host.
- `pnpm install --force`,
  an install whose lockfile changed,
  `pnpm store prune` and `pnpm approve`:
  only the offline frozen-lockfile installs above were run.
  pnpm's own pruning of a lockfile change is the shape most likely to reach into `node_modules`,
  and it was not exercised.
- How the arrangement would come to exist in the main checkout and in every new worktree,
  which is a design step,
  not a tool's behaviour,
  and was not built.
  Today's `git worktree add` does not create it,
  and the clone shows nothing recreates it after a reinstall.
- The conversion of a populated directory into a subvolume:
  arm I only shows it cannot be done in place,
  and copying 168,825 entries into a fresh subvolume while the logger keeps appending to them
  was neither timed nor attempted.
- What the arrangement costs in btrfs metadata and quota across 215 directories with 168,825 entries.
- Windows and macOS,
  which the owner's pause covers.

## The decision this leaves open, with what each route costs

Nothing of it was built,
and `doc/decision/` is untouched.

- Make the directory once,
  by hand or by a one-off task,
  in the main checkout only.
  Pro:
  no tool change,
  and everything in those directories stops being carried into a snapshot,
  which today's count puts at 168,825 entries in 215 directories.
  Con:
  the conversion needs a copy of whatever stands there (arm I),
  every new worktree still gets the empty stand-in (arm J) so the step is needed per worktree,
  and one `fix:reinstall` ends it silently.
- Have a wrapper make it a subvolume again whenever it finds a plain folder.
  Pro:
  self-healing,
  so a `fix:reinstall` or a fresh worktree does not end the arrangement,
  and the cost measured here is a few milliseconds per creation (arm H).
  Con:
  it is a new owner in the wrapper for a directory the logger owns,
  it still has to copy whatever stands there (arm I),
  and it decides for 215 directories on every command.
- Leave the snapshot as it is and remove the entries from what the leave-out moves instead,
  for example by leaving scratch trees out of the trash.
  Pro:
  no filesystem arrangement at all,
  and nothing to lose to a reinstall.
  Con:
  it catches the 135,889 entries of one scratch tree and not the run records
  (295 entries in the largest directory,
  28,941 at a checkout's root).

Ranking:
the wrapper route over the leave-out route,
and the leave-out route over the one-off route,
because the wrapper keeps the whole benefit and survives what removes the directory,
the leave-out route keeps most of today's measured share without any filesystem arrangement,
and the one-off route's benefit is ended silently by an ordinary `fix:reinstall` and is needed again in every worktree.
That ranking is a suggestion for the owner to veto,
not a change made here.
