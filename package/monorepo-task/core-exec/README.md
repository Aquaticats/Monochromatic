# @monochromatic-dev/monorepo-task-core-exec

CLI utilities for mise task orchestration:
 a command executor with `allowFailure` support,
a file append helper,
 a make-style dependency checker,
a `tsc` wrapper that filters `node_modules` diagnostics,
and an `oxlint` wrapper that augments diagnostics while preserving every finding and exit status.

## Orchestration regression tests

`src/build-and-test.unit.test.ts` verifies the generated root `buildAndTest` task
through real mise subprocesses with disposable build and test fixtures.
It checks that tests still run after build failure,
that either phase failing makes the aggregate fail,
and that explicitly tolerant tasks remain tolerant.

This suite consumes generated `mise.toml` rather than a bundled TypeScript function:
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

### task-oxlint

No flags of its own;
 all arguments are forwarded to `oxlint`.
 The wrapper
prepends `--format=default` unless an explicit `--format`/`-f` is present,
and reads the `OXLINT_THREADS` env var to inject `--threads <value>`.
