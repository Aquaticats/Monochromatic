# Preventing mistakes: tasks, builds and bulk output

Rules for running the repository's tasks and builds:
where a task runs from,
which install a build uses,
which tools a gate tree runs,
and where large output goes.

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.
It held the section of the same name in
[Preventing mistakes: shell commands, edits, commits and agents](mistake-prevention-shell-edits-commits-and-agents.md)
until 2026-10-19,
when that file reached the 1,000-line limit.

## Tasks, builds and bulk output

What happened:
a source-map build was named `build:coverage`,
and the package's `build` fan-out starts every `build:*` child at once,
so each plain build ran it beside the normal one into the same directory (M63);
the census first chose `tmpdir()` for about 8 GB of raw coverage,
where `/tmp` is a tmpfs held in memory (M64);
a test's stand-in command carried a literal `[FAIL]`,
which a warning quoted into the suite's own log,
where the census counts it (M65);
a scratch sweep walked the package and `doc/` with a recursive `readdirSync`
and held 6.7 GB resident after 16 minutes,
when it was stopped (M96).
A background suite written as `<suite> > <log> || echo "exit $?"` printed its fallback into the task output,
not the log,
and the task reported exit 0 while the log held a failing scan (ledger B103).
With that rule written,
a script making agent worktrees failed at its first worktree twice behind the same fallback,
its task reported exit 0,
its log went unread,
and an agent was sent to a worktree that did not exist (ledger M114).

The type check ran before the build on a fix tree and reported ten errors in two test files,
read against the old built declarations (ledger M132).
Two listings printed several hundred file names into a session where a count was wanted (ledger M132).
A message that resumed stopped agents said they had been stopped for a usage limit,
and one read it as being cut short and handed back part of its work (ledger M127).
The type check reads the built declarations,
so it runs after the build;
a count comes before a listing;
and a message that resumes an agent says the whole task stands.
A package task started from the main repository's root,
not the worktree holding the branch,
found no such task (ledger M134);
every package task names its worktree as its directory in the call.
A gate started as a user unit under `mise exec` met a `git` without the worktree flag its setup uses,
failed in its first seconds,
and was found about twenty minutes later,
since the waiter armed on its log's last line never fired;
and a unit wrote its log into a folder not made yet,
and failed at setting up its output (ledger M141).
The agent porting the runner observations change ran a lint beside the source scans,
which rebuilt `dist` under it,
and counted that lint void (ledger B366),
the shared build directory of M63 again:
a run that reads `dist` never runs beside one that rebuilds it.
A reviewer's `verify-published` rebuilds `dist` too,
and ran during probe reruns that read it,
so five probes failed with a missing module and were run again (ledger M182);
every task that builds counts as a rebuild.
The eighth review ran its replay with `--skip-deps` for the same reason (ledger M185),
and two commands that read `dist` together were started in one block in two more rounds (ledger M184,
M186).
The first lines of a started run's log are read before a waiter is armed,
a waiter watches the failure line as well as the success line,
and the folder a unit writes to exists before the unit starts;
a gate runs in the plain shell,
since it makes worktrees with the shell's `git`.
The lead's brief for the runner leftovers handed on ten observations from a task title and two old notes,
three of them with no source text in the notes,
so the agent had to find each again from the code and the ledger (ledger M142).
A brief names the source of each observation it hands on
(a file and line,
a ledger code or a report),
or says it holds none,
so the agent knows whether it is checking a claim or searching for one.
On 2026-10-11 (UTC) the lead again started a package task from the main repository's root,
ran the type check while the named-test runner was rebuilding `dist`,
and piped `git apply --check` into a filter,
so the shell reported the filter's exit and a patch that applied read as refused (ledger M150);
a command whose exit decides the next step runs unpiped.
An agent waiting on its suite read a background task's output file in the harness's task folder,
which its brief forbids,
then stopped and read its own scratch logs instead (ledger M149);
a continuation brief restates every binding rule of the brief it continues,
since a fresh agent has not read the old one's corrections.

The rule:
a task that must not run with a fan-out parent is named outside the parent's prefix,
and a new task's first run is read in its log for every task it started.
Output that can reach gigabytes goes under the package's cache directory on disk,
after `df` has shown the filesystem.
A fixture printing a marker that tooling counts builds the marker when it runs,
and a new test file's first run is read for markers,
not only for its exit.
A background run appends its exit status to its own log
(`<suite> > <log> 2>&1 || echo "suite exit $?" >> <log>`)
and is read by its `[FAIL]` markers and that line,
never by the task's reported status.
A command that sets up what a later step uses
(a worktree,
a build,
an install)
is read by its log before that step starts,
and a path an agent is sent to is listed in the call that sends it.
A scratch scan lists its files from git's index,
or walks with a bound that skips `node_modules`,
and runs with a heap cap (`node --max-old-space-size=512`);
one that runs past its expected time is checked once by process status.
A task that depends on the build,
run while other work reads `dist`,
is run without it
(`mise run --skip-deps <task>`),
and the brief names that form (ledger M185).
The katex shim landed as a new workspace package with no file-enforcer run,
and the private registry's config,
which file-enforcer derives from the workspace,
left it out until a later run for another change wrote it in (ledger M286).
A change that adds,
removes or renames a workspace package runs `mise run file-enforcer` in its worktree before it lands
and commits what that run writes to the files derived from the workspace;
the rewrite of `_.path` in the root `mise.toml` follows the worktree's installs,
so it is read against them first,
and a scratch worktree's partial install is never committed as that list.

The corpus-pass package's test runner prints no count of cases or files (ledger M293).
Its gate reads the exit code,
the `[FAIL]` lines (zero),
and the `[PASS]` lines of the suites the change added or touched,
by name,
because a suite with failing cases still prints its pass line.

What enforces it:
the coverage census,
which refuses a suite whose log carries a failing marker,
and one that printed no pass marker (ledger B384);
habit for the names,
the directories,
the sources a brief names
and the file-enforcer run before a package lands.

## Gate trees and their tools

What happened:
after the repository moved its tools to MPM,
gate g219 on the merge of main passed 3181 cases and failed 694,
and its source scans exited 1.
The typography package runs `monochromatic-lint` by name,
and on the gate's `PATH` that name reached the shim of `mise`,
which printed `mise ERROR No version is set for shim`.
The tools now live in a directory of each worktree,
`.cache/toolchain-bin`,
which `mise run toolchain:sync` makes.
The gate's `PATH` builder had dropped the entries of the main checkout,
and the fresh gate trees had never been synced.
Round 8's own red cases in `tr-w357` were the same missing program in a tree that predated the cutover (ledger B980).

A rotation sweep started in `tr-g224` while gate g224 rebuilt `dist` in the same tree,
and its salt-9 canary failed with `ERR_MODULE_NOT_FOUND` on `translation-repair-markup/dist/final/node/index.mjs`
(ledger M297).
Gate g223 on the merge `b12cdc5d23` of main into the cooldown branch went red in the type check,
the oxlint run and the unit tests of `pass-entry`,
though the merge itself was clean:
the branch's begin-entry case at `pass-entry.unit.test.ts:1501` lacked `attempt`,
which the stamps that main carried at `112f6274fc` had made required (ledger M298).
Gate g215 failed `run-client-create.unit.test.ts:480` under host load,
and the case passed alone:
the rig counted turns of the event loop (fixed in `df2350a6ee`,
ledger B965 and M299).
Gate g228 ran beside two rotation sweeps,
and its `probe.test-unit` step exited 1 with 11 failing built-command cases
whose stderr carried the line
`logger internal error: sink verification failed ... Timed out after 5000ms`.
The step run alone in the same tree passed 296 cases with none failing (ledger M309).
An install on the current main rewrote `pnpm-lock.yaml` in the gate trees `tr-g227` and `tr-g228`,
149 changed lines in the first and a 229-line diff in the second.
The `git checkout --detach` of gate g227 was refused by cli-git over the dirty tree,
and each gate tree was dirty again when it was retired,
so the retire script refused to remove it (ledger M308).
No launch line of a gate was recorded in the notes before gate g239,
and none for a sweep stands in them now (ledger M316).

The rule:
a gate tree is set up in this order before its first gate:
`mise run prepare:pnpm:install`,
then `mise run toolchain:sync`,
which needs the installed `node_modules`.
The gate's `PATH` puts `<tree>/.cache/toolchain-bin`,
`<tree>/.cache/cli-git-bin` and `<tree>/node_modules/.bin` first
and leaves out the directory of the `mise` shims.
A subagent never runs `toolchain:sync`:
after a subagent merges main across a change to the toolchain,
the lead syncs that subagent's tree.
A failure that names a missing program or a shim with no version is read as a tree with no sync
before it is read as a case that is red.
A rotation sweep runs beside a gate only in another worktree:
in the gate's own tree it starts after the gate ends,
since the gate rebuilds `dist` and the sweep reads it.
At most one sweep runs beside a gate,
even in another worktree (ledger M309).
After main is merged into a branch,
`lint:types` runs on the merged head before a full gate,
because a merge with no conflict can still make a field required that the branch's cases leave out.
A red case is run alone before it is judged:
a case that fails under the load of a gate and passes alone is looked at for a rig that depends on timing,
and one that fails alone is read for its message.
A red step of a gate is run alone in the same tree before it is judged,
as a red case is:
`probe.test-unit` of gate g228 was red with 11 cases beside two sweeps,
and passed 296 cases with none failing alone (ledger M309).
After a gate tree's install,
`git status --short` is read for a rewritten `pnpm-lock.yaml`,
since an install on the current main rewrites it (ledger M308).
The tree setup saves that diff and restores the file with `git restore --no-enforce-worktree -- pnpm-lock.yaml`,
and the restore is repeated before the tree is retired.
That restore is the lead's,
in scratch tooling:
a subagent leaves the file as it is and reports it in its hand-back.
The lockfile belongs to another session,
and no gate tree and no round commits it or changes it.
A gate is launched from the plain shell as a user unit of its own,
in the form the notes recorded on 2026-10-29 (UTC) while gate g239 ran.
The four variables that `env` passes hold no key,
and the user bus needs the last two of them (ledger M317).
`<tr-fan>` is the lead's scratch folder,
`/var/home/user/temp/agent/tr-fan`,
and `gate-family.ts` is a scratch script of the lead's outside the repository.
The hash it takes is the full hash of the head,
copied from a git command that printed it (see "Hashes given to a command" in
`mistake-prevention-commit-checks-and-figures-told-to-others.md`).

```bash
# launch form of a gate; the four variables of env pass through as they stand
env --ignore-environment PATH="${PATH}" HOME="${HOME}" \
  XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR}" DBUS_SESSION_BUS_ADDRESS="${DBUS_SESSION_BUS_ADDRESS}" \
  systemd-run --user --unit=tr-gate-gNNN --collect \
  --property=MemoryMax=32G --property=CPUQuota=2400% \
  --working-directory=<tr-fan> \
  --setenv=PATH="$(cat tree-path-gNNN.txt)" --setenv=HOME \
  --property=StandardOutput=file:<tr-fan>/gate-gNNN.out \
  --property=StandardError=file:<tr-fan>/gate-gNNN.err \
  <node from the tree path> s245/split/gate-family.ts <full hash> gate-gNNN <tree>
```

The gate's verdict is in `gate-gNNN/summary.txt`:
`gate-gNNN.out` holds only the wrapper lines of systemd.
The limits are sized to the host (28 threads and about 41 GiB available when they were set),
and a gate waits on its unit's process id with a bound of 7,200 seconds.
A rotation sweep is a unit of its own too,
named `tr-sweep-gNNN`
(`tr-sweep-wNNN` when it runs in a round's own tree),
started in the tree it tests once the gate of that tree has ended,
with its log in `sweep-gNNN.log` of the scratch folder.
Its limits were `MemoryMax=8G` and `CPUQuota=800%` beside a gate,
and 16G and 2000% once the gate had ended,
and the one note that gives its wait bound gives 3,600 seconds.
The notes hold no full command for a sweep.
The next launch writes its full line into the handover,
and the line is added here from there.

What enforces it:
habit,
and the lead's `PATH` builder for gates and the tree setup that restores the lockfile,
which are scratch tooling outside the repository.
