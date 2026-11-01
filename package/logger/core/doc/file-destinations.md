# File destinations

## Consumer contract

Default Node logging first tries the nearest ancestor `node_modules/.monochromatic` directory.
A missing project or a failed project-file probe falls back to persistent user state.
The fallback is verified before one console notice identifies the destination and why it was needed.
The notice uses a direct host console because logging through the sink itself would recurse.
Both fallback notices and direct internal error diagnostics use the console sink's terminal-control neutralizer.
The adversarial filename catalog fails on the published `0.4.3` artifact before that repair.
Logger `0.4.4` includes the repair;
its build,
fresh-cache types,
full units and full package lint pass locally.

An explicit `createFileSink({ directory })` argument or nonempty `MONOCHROMATIC_LOG_DIRECTORY`
selects a strict destination.
Failure disables that file sink and reports the error;
it never redirects caller-selected diagnostics to another location.
An empty string preserves default discovery.

On Unix systems,
fallback files use `$XDG_STATE_HOME/monochromatic/log`,
or `~/.local/state/monochromatic/log` when that variable is missing,
empty or relative.
The [XDG base directory specification][] requires absolute environment paths and identifies logs as persistent state.
On macOS,
the destination is `~/Library/Logs/monochromatic`.
On Windows,
it is `%LOCALAPPDATA%\monochromatic\Log`,
with `~/AppData/Local` as the local application-data fallback.
Those platform layouts follow the existing ecosystem [env-paths implementation][].
The package does not add a dependency to resolve these paths.
The repository's `module-conf-fork` owns config paths,
which are a different storage lifetime from log state.

Destinations stay project-local (proposal option L2,
recorded in `doc/decision/module-logger-log-accumulation.md`).
The day-directory nesting under that destination is a deliberate layout change
shipped in `0.4.5`,
not a location change.
Moving the defaults to user state (proposal option L1)
remains the recorded escape
if the retained window ever re-triggers the cli-git linked-worktree copy stall
of issue 571,
which carries the ignored log tree into every new worktree.

## File layout

Each process owns one log file per destination directory,
`{YYYY-MM-DD}/{timestamp}.{uuid}.log.jsonl`,
nested under a UTC day directory named for the file's day key.
The real file appears on the first record;
verification only probes writability with a
`{timestamp}.{uuid}.probe.tmp` round-trip it always removes before returning,
so a process that writes no records leaves no log files.
The probe file carries its suffix on purpose:
a crashed probe never looks like a log record.
A `.logger-retention-sweep` stamp file in the destination root
throttles retention sweeps to one walk per five minutes per directory.
It is housekeeping state,
not a log file.
Legacy flat `*.log.jsonl` strays in the destination root
age out alongside the day directories,
keyed by their name prefix.

## Retention window

Verification sweeps the destination down to a retention window:
14 days of history,
536,870,912 bytes (512 MiB),
and 50,000 files,
matching `DEFAULT_LOG_MAX_AGE_DAYS`,
`DEFAULT_LOG_MAX_BYTES`,
and `DEFAULT_LOG_MAX_FILES`.
`MONOCHROMATIC_LOG_MAX_AGE_DAYS`,
`MONOCHROMATIC_LOG_MAX_BYTES`,
and `MONOCHROMATIC_LOG_MAX_FILES` override those bounds;
a non-numeric or negative value falls back to its default,
so a typo cannot disable retention or make it delete everything.
Age expiry keys off the UTC day name prefix,
so one linear pass ages a whole directory,
and a kept day directory owns its contents wholesale.
The budget pass then deletes the oldest files first
until the size and count bounds hold again,
never selecting a file modified within the last hour,
because a running process may still append to it.
Sweeps run off the logging path,
report their own failures through the internal error reporter,
and never block a record.
`flush` settles the sweep so exits and assertions can await it.

## Sweep ownership

Sweeps and the backlog purge touch only logger-owned names:
`YYYY-MM-DD` day directories,
timestamp `*.log.jsonl` files,
and timestamp `*.probe.tmp` residue.
Deliberate neighbors,
such as the `translation-repair-runs*` run directories
and comparison-evidence directories,
are never deleted.

## Inflow control

`MONOCHROMATIC_LOG_FILE=off` disables the file sink entirely,
including an explicit `directory` argument
or a nonempty `MONOCHROMATIC_LOG_DIRECTORY`,
so no filesystem probe or write happens at all.
`MONOCHROMATIC_LOG_DIRECTORY` reroutes the destination
for default loggers and for sinks without their own `directory` argument;
an empty string preserves ancestor discovery.
Test harnesses point it at a per-run scratch directory
to keep runs hermetic even without the kill switch.
Documented defaults for normal runs do not change.

## Backlog purge

The new layout never revisits the old flat files,
so the retention window is applied once,
by hand,
to the `*.log.jsonl` backlog in `node_modules/.monochromatic`
of every worktree of this repository.
Backlog logs older than the 14-day window are purged
and the deliberate run directories stay.
That purge lands with `0.4.5`,
carried out by the release that introduces the automated sweeps.

## Responsibility boundaries

`src/sink/file-project.ts` owns ancestor discovery and preserves its existing exported test seam.
`src/sink/file-state.ts` resolves per-user paths without filesystem work at import.
`src/sink/file-destination.ts` owns project-first selection and visible fallback.
`src/sink/file-open.ts` round-trips the probe and creates one log file
on the first record before publishing it to the append queue.
`src/sink/file.ts` owns instance-local verification,
lazy file opening,
and serialized record admission.
`src/sink/file-append.ts` retains batching and write completion ownership.
`src/sink/file-retention-policy.ts` names the retention bounds and their environment overrides.
`src/sink/file-retention-name.ts` classifies logger-owned names by shape,
without regular expressions.
`src/sink/file-retention-sweep.ts` throttles each destination's sweep and drives it off the logging path.
`src/sink/file-retention-clean.ts` measures candidates and trims oldest-first within the fresh-file grace.

Discovery remains lazy on first use.
Concurrent verification shares one promise and one file.
The browser and neutral artifacts do not import these Node modules.
Directory creation uses mode `0700` and new files use `0600` where the platform honors Unix modes.
Existing directory permissions are not changed.

## Evidence and remaining checks

Regression commit `301671d3e2` precedes the implementation.
The old built artifact fails project fallback,
no-project fallback and user-state failure diagnostics,
while writable-project and strict explicit-destination controls pass.
The log is `logger-default-fallback-red.log` in the session's private verification directory.
Post-change build,
fresh-cache types and the complete unit suite pass.
Full package lint reports zero warnings and errors on 69 files with 486 rules.
The real read-only mounted consumer proves an independent write rejects with `EROFS`,
then verifies default-logger persistence in private user state and strict explicit-destination failure.
The recursive mkdir diagnostic is `ENOENT` on that fixture;
the independent write establishes read-only state without relying on that diagnostic's spelling.

Removing project priority and explicit-destination strictness independently makes the maintained tests fail
after successful private builds.
Restoring each private source and rebuilding passes the same catalog.
Production source and output hashes remain unchanged during both mutations.
The receipt is `logger-destination-mutants-receipt.json` in the private verification directory.
Linux consumers exercise real filesystem behavior.
The platform layout catalog also checks injected macOS and Windows paths;
it does not claim execution on those operating systems.

[XDG base directory specification]: https://specifications.freedesktop.org/basedir/latest/
[env-paths implementation]: https://github.com/sindresorhus/env-paths/blob/main/index.js
