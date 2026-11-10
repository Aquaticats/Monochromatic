# @monochromatic-dev/monorepo-task-core-exec

CLI utilities for mise task orchestration:
 a command executor with `allowFailure` support,
a file append helper,
 a make-style dependency checker,
a `tsc` wrapper that filters `node_modules` diagnostics,
an `oxlint` wrapper that augments diagnostics while preserving every finding and exit status,
the pull-or-build step for the Playwright container image,
the runner of the lock's `mpm-shadow`,
and the daily update of every lock,
which checks every package before and after it and opens a fix-forward issue for
each package it breaks.

## Orchestration regression tests

`src/build-and-test.unit.test.ts` verifies the generated root `buildAndTest` task
through real mise subprocesses with disposable build and test fixtures.
It checks that tests still run after build failure,
that either phase failing makes the aggregate fail,
and that explicitly tolerant tasks remain tolerant.

`src/root-test-aggregate.unit.test.ts` verifies the root `test` aggregate over a
disposable repository.
It checks that a package declaring its own `test` task is reached without an edit to the root configuration,
that root unit discovery leaves that package's `*.unit.test.ts` files to that package,
that it keeps the files of a package that declares no `test` task,
that a failing package suite is named while every other suite still runs,
and that a failing root suite does not stop the package phase.
Both suites consume the hand-written root `mise.toml` rather than a bundled TypeScript function:
the configuration is the artifact under test.
Relative imports supply test-only fixture helpers.

```sh
# Repository root
mise run buildAndTest -- package/monorepo-task/core-exec/src/build-and-test.unit.test.ts
```

## Binaries

### task-command

Wraps command execution to work around task runner limitations where tasks with
`allowFailure: true` can't be dependencies.
 Controls exit codes while preserving
all output and errors.

```sh
# Exit with the command's exit code
task-command -- oxlint

# Always exit with 0 regardless of command result
task-command --allowFailure -- oxlint
task-command -a -- oxlint

# Execute through the shell (enables pipes, &&, etc.)
task-command --shell -- "echo hello && echo world"
task-command -s -- "echo hello && echo world"

# Set a timeout in milliseconds
task-command --timeout 5000 -- npm test
task-command -t 5000 -- npm test

# Combine flags
task-command -a -s -- "oxlint && dprint check"
```

The `--` separator is required to distinguish script flags from command arguments.

### task-append

Appends text lines to a file.
 Validates that the target file exists and has write
permissions before appending.

```sh
# Append a single line
task-append "new line" --to output.md

# Append multiple lines as separate arguments
task-append "line 1" "line 2" "line 3" --to output.md

# Short flag
task-append "new line" -t output.md
```

### task-depends

Make-style dependency checker that replaces mise's built-in `depends`.
Runs the command only when dependencies are stale,
 and collapses command output
(hidden on success,
 shown on failure).

Both `-s` and `-o` accept file globs or `sh:` prefixed shell commands.
`sh:` commands must output a parseable timestamp on stdout
(`Infinity`,
 `-Infinity`,
 unix epoch,
 or ISO 8601).
Non-zero exit codes are errors,
 not silent staleness signals.

Nothing waits forever.
Every `sh:` item and `sh:` time strategy is ended after `--query-timeout-seconds` (default 60),
and the command after `--` is ended after `--command-timeout-seconds` (default 3600;
a provisional value).
Both accept fractions of a second.
An ended child is a failure with exit code 1,
 and the message names the bound and the option that changes it.
The child runs in its own process group and the whole group is killed,
so a pipeline leaves no member behind.
The glob reads of `-s` and `-o` run inside task-depends itself and have no bound of their own.

```sh
# File-based: run build only when sources changed
task-depends -s "src/**/*.ts" -o "dist/**/*.js" -- mise run build

# Command-based gate: rebuild container only when image is missing
task-depends -s "sh:echo Infinity" -o "sh:podman image exists img && echo Infinity || echo -Infinity" -- podman build .

# Timestamp from command: use git commit time as source mtime
task-depends -s "sh:git log -1 --format=%ct" -o "dist/**" -- mise run build

# Catch missing outputs in mixed lists with oldest strategy
task-depends --output-time-strategy oldest -s "src/**" -o "dist/**" -o "sh:curl -sf localhost/health && echo Infinity || echo -Infinity" -- mise run deploy

# With verbose logging
task-depends -v -s "src/**" -o "dist/**" -- mise run build

# Allow command failure
task-depends -a -s "src/**" -o "dist/**" -- mise run build

# Longer bounds for a slow probe and a long build
task-depends --query-timeout-seconds 120 --command-timeout-seconds 7200 -s "src/**" -o "sh:slow-probe" -- mise run build
```

### task-tsc

Wrapper for `tsc` that cleans stale incremental caches and
filters out diagnostics originating from `node_modules/` paths.

**Incremental cache cleanup.
**
The shared tsconfig sets `composite: true`,
 which implies `incremental: true`.
This is intentional;
 `composite` provides valuable constraints
(rootDir defaults to the tsconfig directory,
 all source files must be matched by `include`,
and `declaration` defaults to true).
However,
 tsc's `--build` mode has a cache invalidation bug
([#2666](https://github.com/nicolo-ribaudo/tc39-proposal-structs/issues/2666))
where stale `.tsbuildinfo` files cause false negatives after dependency updates.
To work around this,
 `task-tsc` deletes all `dist/**/*.tsbuildinfo` files
before each invocation,
 forcing a clean check every time.

**`node_modules` diagnostic filtering.
**
JSR packages ship `.ts` source files instead of `.d.ts` declarations.
TypeScript's resolver prefers `.ts` siblings over `.js` exports,
and `skipLibCheck` only covers `.d.ts` files.
This causes `tsc --build` to type-check JSR package source
under the consumer's tsconfig,
 producing false positives
(e.g. `noUncheckedIndexedAccess` violations in `@jsr/zod__zod`).
The wrapper drops diagnostic lines whose file path contains `/node_modules/`
along with their continuation lines (indented context lines),
and exits 0 when only `node_modules` errors were found.

A compiler crash,
silent nonzero exit,
or unfamiliar remaining output preserves the failing process status.
Suppression requires that the existing filter actually removed known diagnostics;
only blank stdout lines and emitted-file `TSFILE: ` metadata may remain.
`test:tsc-failure` verifies the built wrapper through disposable compiler processes,
including a simulated Go panic that creates no actual thread pressure.

```sh
# Default: runs tsc --build with filtering
task-tsc --build

# Forward any tsc arguments
task-tsc --build --noEmit
task-tsc --noEmit -p tsconfig.json

# Without arguments, defaults to --build
task-tsc
```

**`TSC_SINGLE_THREADED` requests TypeScript single-threaded mode.
**
When set to a truthy value,
the wrapper injects `--singleThreaded` unless the caller already passed it.
The root `mise run lint` fanout sets this env var beside `OXLINT_THREADS`,
so package-local `lint:types` tasks do not stack TypeScript 7 worker pools on top
of mise's package-level parallelism.
Values `0`,
`false`,
`no`,
and `off` opt out.

See `TROUBLESHOOTING.typescript.md` section
"JSR packages ship `.ts` source files that `skipLibCheck` cannot skip"
for the full root cause analysis.

### task-playwright-image

Makes the Playwright container image available locally;
it is what `mise run prepare:playwright` runs.
CI publishes `playwright.Dockerfile` as `ghcr.io/aquaticats/monochromatic-playwright:<tag>`,
where `<tag>` is the first 16 hex digits of the SHA-256 of the file
(`.github/workflows/playwright-image.yml`,
`doc/runbook/publish-playwright-image.md`).
The command,
run from the repository root,
computes the same tag and then:

- keeps a local `monochromatic-playwright:<tag>` as it is,
- otherwise pulls the registry image and names it `monochromatic-playwright:<tag>` and `monochromatic-playwright`
  (the name the Playwright tasks run),
  but only when `podman info` reports `linux/amd64`,
  the platform CI publishes,
  because podman pulls a single-platform image of another platform with exit 0 and only a warning,
- and when the pull fails,
  or the machine reports another platform or none,
  prints why on one line and builds `playwright.Dockerfile` locally with the same two names.

Podman's own output stays hidden;
a failure prints the reason,
and the local build's output when that fails too,
and exits 1.
Any edit of `playwright.Dockerfile`,
comments included,
changes the tag.

```sh
# Repository root
task-playwright-image
```

### task-mpm-shadow

Runs `mpm-shadow` at least at the version `toolchain.lock.json` names;
it is what the root `toolchain:*` tasks run.
The command finds the repository root (the folder with the lock,
searched upward from the current directory)
and runs the newest installed copy of this repository's `mpm-shadow` whose `--version` is at or above the lock's version
for the `mpm-shadow` tool,
looking at `.cache/toolchain-bin/mpm-shadow` and at every finished version folder of
`<data home>/monochromatic-dev/toolchain/mpm-shadow/` and `<data home>/monochromatic-dev/toolchain-global/mpm-shadow/`
(a folder with the `.mpm-shadow-installed` marker and the program;
this repository's tools are one set across both roots,
and the newest copy wins wherever it is),
and saying `locked at X in <lock file>, Y runs; upgrade that lock with: mise run toolchain:update mpm-shadow` on standard
error when that copy is newer,
`<lock file>` being the lock it compared,
so a reader in a repository of worktrees can tell which worktree is behind.
A copy older than the lock never runs.
Only when no copy is at or above the lock's version does it download the lock's archive for the platform,
check the SHA-256 recorded beside it,
unpack it into a hidden staging folder,
require a program that prints the locked version,
write the marker last,
rename the folder to `<version>`,
point `current` at the newest finished version and remove the older version folders,
which is how `mpm-shadow install` lays out an archive-route tool that this repository publishes
(a folder at the version's name that is not finished,
or whose program prints another version,
is replaced,
and one that another task finished meanwhile is used;
staging folders and downloads that a killed task left are removed once their process is gone and they are ten minutes old:
a process id unknown here can belong to a live task in another process id namespace that shares the data home,
so the age is checked as well).
It then runs that copy with the arguments given and exits with its status;
the `mpm-shadow` on `PATH` is never run,
except on a platform for which the lock has no archive row and no installed copy,
where it is asked for its version first and runs only when that is at or above the lock's
(a note on standard error says so,
and an older one is refused in one line).
Versions are ordered by numeric core,
then a release above its pre-release,
then pre-releases by SemVer 2.0.0 precedence
(`rc.2` below `rc.10`).
A failure the command reports itself (an unreadable lock,
a failed download or checksum,
a copy that does not run)
prints one line and exits 1,
and asking a copy for its version gives up after 10 seconds.
The command needs Node 24 or newer
(`import.meta.main` and `Error.isError`;
type stripping must be on by default,
which it is from Node 22.18 and 24.3);
`fetch` in Node ignores `HTTP_PROXY` and `HTTPS_PROXY` unless `NODE_USE_ENV_PROXY=1` is set
(measured with Node 26.13.0),
so a machine behind a proxy sets that variable for the task.
Notes go to standard error so that standard output stays the program's (`bin --json` is read by other programs).
The modules behind it import only `node:` modules,
so the command runs from source on a checkout without `node_modules`;
`.github/actions/repo-tools-setup/action.yml` does the same for CI by running `src/repo-tools-setup.ts` from source.

```sh
# Repository root
task-mpm-shadow install --lockfile-only --frozen-lockfile
```

### task-daily-update

The daily update of the toolchain locks and the dependency locks,
and the report of what it breaks.
`mise run toolchain:update:daily` runs it once;
the systemd user timer installed by `mise run toolchain:timer:install` runs it every day.
The order of its steps,
the failure rules,
the measured caps and the check bounds are in
`doc/planning/toolchain-always-latest.md` ("The daily update" and "Checking every package after the update").

After the groups commit,
the update tells the owner what the update broke:

- Before anything is updated,
  every package is checked through its own `mise` task:
  `lint:clippy` for a Rust crate,
  `lint:types` for a TypeScript package.
  A package with neither task is not checked.
- After the groups commit,
  every package is checked again,
  when the update moved anything.
  The commits are never waited on and never undone by this.
- A check that passed before and fails after is newly red;
  one that failed before is already red and is never reported;
  one that timed out or never ran is unknown,
  and is named in the notification and never gets an issue.
- Each newly red package gets one GitHub issue through `gh` in this repository,
  labeled `needs-triage`
  (`doc/agent/triage-labels.md`).
  It names the update commits,
  what moved in the package's lock and in the toolchain locks,
  the first error with its
  code,
  and the command that reproduces it.
  An open issue with the same package,
  check and cause gets a comment instead.
- A `gh` failure is logged with its reason and the issue is kept in
  `.cache/toolchain-daily-update-pending-issues.json` for the next run;
  it never changes the exit status.
- The owner gets at most one desktop notification per run,
  with the issue addresses;
  the journal has the details.

Tests never reach a real desktop or a real GitHub:
`src/daily-update-package-checks.unit.test.ts` scripts every effect,
and `src/daily-update-whole-path.unit.test.ts` runs the real entry point over a disposable repository with a recording
stand-in for `gh` and the real `notify-send` on a private session bus,
behind a guard (`assertPrivateBus`) that refuses any other bus.

```sh
# Repository root
mise run toolchain:update:daily
mise run //package/monorepo-task/core-exec:test:unit -- package/monorepo-task/core-exec/src/daily-update-whole-path.unit.test.ts
```

### task-oxlint

Wrapper for `oxlint` that augments diagnostics with extra guidance
from `oxlint-augment.ts`.
The repository runs lint through this wrapper,
so the mise `lint:oxlint` task is the consumer boundary that matters.
The wrapper never removes diagnostics or converts Oxlint failures to success.

```sh
# Type-aware lint from the shared config (what the mise lint:oxlint task runs)
task-oxlint

# Auto-fix
task-oxlint --fix

# Any oxlint arguments are forwarded
task-oxlint src/
```

Several behaviors differ from invoking `oxlint` directly.
 They are intentional;
this list exists so they are not mistaken for bugs.

**Forces `--format=default`.
**
The wrapper prepends `--format=default` unless the caller passes an explicit
`--format` or `-f`.
Oxlint's piped default reporter is not stable across versions.
The guidance injector needs graphical diagnostic boundaries to place notes beside their matching findings.
Consequently,
`task-oxlint` uses the graphical reporter even when piped,
while an explicit format always wins.
Non-graphical formats may not receive added guidance,
but their diagnostics and exit status still pass through unchanged.

**`OXLINT_THREADS` overrides thread count.
**
When set,
 the wrapper injects `--threads <value>`.
 oxlint ignores
`RAYON_NUM_THREADS`,
 so this env var is the only way to pin threads without
editing every call site.

**A crashed analysis child fails the run.
**
Oxlint's type-aware pass runs helper programs beside itself:
`tsgolint`
for the `typescript/*` rules,
and the TypeScript API server a JavaScript plugin reaches through `typescript/sync`.
Both are Go programs,
so a panic inside one kills an analyzer while Oxlint itself carries on and exits on the
diagnostics the survivors produced.
A run like that looks clean and is not.
The wrapper reads the crash report the dying child writes to the inherited standard error
and throws instead of reporting success,
so the task fails with a message saying the run's findings are incomplete.
Recognition needs the crash header together with the Go runtime's goroutine dump or signal
line,
so output that merely mentions a panic is not treated as a crash.
Every pass forwards Oxlint's own output first,
so the findings that did survive are still shown.
A `--fix` run is judged by its final convergence pass:
a crash in an earlier pass that the final pass does not repeat goes unreported.

## Why task-depends over mise native `depends`

Mise's built-in `depends` has three problems that make it unsuitable for
environment prerequisites like container image builds.

**`depends` always re-runs.
**
Mise's `depends` field unconditionally re-executes every listed task on every invocation.
A `podman build` that takes 30 seconds runs before every test,
 even when the image already exists.
Mise does offer `sources`/`outputs` for staleness checking,
 but these only work with files;
they cannot check whether a container image,
 a system package,
 or a running service exists.

**`sources`/`outputs` is file-only.
**
Mise's staleness detection compares file modification times and metadata hashes.
This works for source-to-artifact builds but cannot express conditions like
"does this container image exist" (`podman image exists`) or
"is this service healthy" (`curl --fail http://localhost:8080/health`).
`task-depends` accepts `sh:` prefixed shell commands in both `-s` and `-o`.
Commands must output a parseable timestamp (`Infinity`,
 `-Infinity`,
unix epoch seconds/ms,
 or ISO 8601).
 Non-zero exit codes are treated as errors,
preventing silent misinterpretation when commands fail unexpectedly.

**`depends` output is noisy.
**
When a depended-on task runs,
 its full stdout/stderr streams into the parent task's output.
For long-running builds this drowns out the actual task output.
`task-depends` captures the command's output and only shows it when the command fails,
keeping the happy path clean.

### Staleness model

Everything resolves to timestamps.
 Stale when `strategy(sources) > strategy(outputs)`.

**File globs** resolve to file modification times.
Empty globs contribute no timestamps;
 the aggregation strategy returns `-Infinity`
("no information"),
 which means empty sources never trigger and empty outputs always trigger.

**`sh:` commands** must output a parseable timestamp on stdout:

- `Infinity` or `-Infinity` (gate pattern:
   exists/missing)
- Unix epoch seconds or milliseconds
- ISO 8601 date string

Non-zero exit codes throw an error.
 To handle command failures gracefully,
use the shell gate pattern:
 `sh:command && echo Infinity || echo -Infinity`.

**Aggregation strategies** reduce multiple timestamps per side to one value:

- `newest` (default):
   `Math.max`;
   any new source or any fresh output dominates
- `oldest`:
   `Math.min`;
   catches the oldest/missing item in mixed lists
- `mean`:
   arithmetic mean of all timestamps
- `median`:
   middle value,
   robust to outliers
- `sh:command`:
   custom shell command that receives millisecond timestamps
  on stdin (one per line) and must output a single millisecond timestamp,
  `Infinity`,
   or `-Infinity`

```sh
# Custom strategy using Unix sort (minimum = oldest)
task-depends --output-time-strategy "sh:sort -n | head -1" -s "src/**" -o "dist/**" -- mise run build

# Custom strategy using awk for weighted average
task-depends --source-time-strategy "sh:awk '{s+=$1;n++} END{print s/n}'" -s "src/**" -o "dist/**" -- mise run build
```

Commands run via `/bin/sh` (Node.
js `child_process` with `shell: true`).

## Flags

### task-command

- **-a,
   --allowFailure**:
   suppress the command's non-zero exit code (always exit 0)
- **-s,
   --shell**:
   execute the command string through the system shell
- **-t,
   --timeout \<ms\>**:
   kill the command after the given number of milliseconds

### task-append

- **-t,
   --to \<file\>**:
   target file to append to (required)

### task-depends

- **-s,
   --sources \<glob | sh:
  command\>**:
   source item (repeatable,
   optional)
- **-o,
   --outputs \<glob | sh:
  command\>**:
   output item (repeatable,
   at least one required)
- **--source-time-strategy \<strategy\>**:
   aggregation for source timestamps (default:
   `newest`)
- **--output-time-strategy \<strategy\>**:
   aggregation for output timestamps (default:
   `newest`)
- **-a,
   --allowFailure**:
   suppress the command's non-zero exit code (always exit 0)
- **-v,
   --verbose**:
   log staleness decision details to stderr

Strategies:
 `newest`,
 `oldest`,
 `mean`,
 `median`,
 or `sh:command`.

### task-tsc

No flags of its own.
 All arguments are forwarded directly to `tsc`.
Defaults to `--build` when no arguments are provided.

### task-playwright-image

No flags and no arguments;
it reads `playwright.Dockerfile` in the current directory and needs `podman` on `PATH`.

### task-mpm-shadow

No flags of its own;
every argument is passed to `mpm-shadow`.

### task-oxlint

No flags of its own;
 all arguments are forwarded to `oxlint`.
 The wrapper
prepends `--format=default` unless an explicit `--format`/`-f` is present,
and reads the `OXLINT_THREADS` env var to inject `--threads <value>`.
