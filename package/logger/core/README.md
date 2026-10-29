# module-logger

Zero-config multi-sink logger with tagged composition.
Works immediately with no setup call:
 the first log or flush call builds the default logger and auto-discovers available backends for the current runtime,
and records emitted while async backend verification is still pending replay to those
backends as soon as they verify.
Importing the package runs no discovery,
 no timers,
 and no I/O.

## Usage

```ts
import { tagged, } from '@monochromatic-dev/logger-core';

const l = tagged({ tag: 'http', },);
l.info('server started on port 3000',);
// console output: [info] [2026-03-11T...] [http] server started on port 3000
```

### Composing tags

Pass a parent logger via the `l` parameter to build hierarchical prefixes:

```ts
const l = tagged({ tag: 'http', },);
const rl = tagged({ tag: 'retry', l, },);
rl.warn('attempt 3 failed',);
// [warn] [...] [http] [retry] attempt 3 failed
```

Composed tags render root-first because each wrapper prepends before delegating to its parent logger.

Convention:
 use `myFn.name` as the tag so prefixes stay in sync with refactors.

```ts
function handleRequest({ l, }: { l: Logger; },): void {
  const rl = tagged({ tag: handleRequest.name, l, },);
  rl.info('received',);
}
```

### Default logger (no tags)

Import the singleton directly when tags are not needed:

```ts
import { logger, } from '@monochromatic-dev/logger-core';

logger.error('unexpected shutdown',);
```

## Runtime support

The [write completion and file ownership explanation](doc/write-completion.md)
documents completion tracking,
file batching,
ownership,
and the regressions motivating those internal parts.

Node 24 or newer (the build calls `Error.isError`),
 plus current browsers,
 Deno,
 Bun,
 and QuickJS-ng 0.16.2 for the sinks whose `verify` finds a backend there.
QuickJS-ng provides only `console.log` and no global timers:
 its default console sink sends all unsuppressed levels to stdout,
 not to stderr.
Call `await logger.flush()` before an explicit `std.exit()` so batched records
 are emitted before process termination.
For stderr output,
 supply a QuickJS-specific sink or keep a host-specific adapter.
### Global-scope-restricted runtimes

Cloudflare Workers (and any runtime that forbids timers,
 I/O,
 and random values in global scope) can import the root entry and `tagged` freely:
nothing is built at import,
 so no sink probe and no timer runs in global scope.
The first log or flush call inside a handler builds the default logger and verifies its sinks there.
A Worker that wants a logger scoped to one request
can still build its own with `createLogger`
over `sinks.createConsoleSink()`
and hand `flush()` to `ctx.waitUntil`.

The published package exposes the built artifact only.
The `/ts` source subpath used inside this workspace is stripped at publish time,
 because Node refuses `.ts` files under `node_modules`.

The root entry is platform-neutral and is built twice.
The `node` export condition serves a build whose default logger includes the file sink,
 with static `node:fs/promises` and `node:path` imports,
 and which carries no browser storage code.
Every other resolution (`default`) serves a build whose default logger includes the IndexedDB sink
 and which references no Node module.
Neither build contains a dynamic `import()`,
 and a unit test reads every chunk of both builds to keep it that way.
The root types are identical on both conditions.
A Node consumer whose bundler resolves the `default` condition gets the neutral build and no file logging.

## Log levels

Six levels,
 each mapping to a dedicated method:
 `trace`,
 `debug`,
 `info`,
 `warn`,
 `error`,
 `fatal`.

All methods accept a single `string` argument.
The caller owns serialization;
 template literals cover the common case
and keep the logger free of stringify opinions.

```ts
l.info(`status ${code} for ${url}`,);
```

The console sink silences `debug` and `trace` by default in non-browser environments.
Set `MONOCHROMATIC_VERBOSE=true` or pass `--verbose` to enable them.
In browsers,
 verbose mode is enabled automatically because DevTools
already provides its own log-level filtering.

## Sinks

The default logger writes to **all** available sinks simultaneously.
Availability is verified when the logger instance is created,
 with default-instance creation deferred until its first log or flush call.
Every sink is verified concurrently,
 each under its own time limit when host timers exist (`verifyTimeoutMs`,
 default `DEFAULT_VERIFY_TIMEOUT_MS`,
 5000 ms);
 a sink that does not answer in time counts as unavailable,
 so one hung backend probe cannot starve the others.
Records emitted while an async sink is
still being verified are replayed to that sink when it becomes available.
That startup buffer holds at most `STARTUP_BUFFER_CAP` records (10000,
 exported);
 past the cap the oldest buffered record is dropped,
 and once every sink has answered one `warn` record naming the dropped count is written to every available sink.

- **console**:
   formats as `[level] [ISO timestamp] message`;
  maps levels to corresponding `console.*` methods,
  falls back to `console.log` for a missing level method,
  and writes `debug` to `process.stderr` when `process.stderr.write` is available
- **file**:
  Node.js only.
  `createFileSink({ directory })` selects an explicit destination;
  otherwise `MONOCHROMATIC_LOG_DIRECTORY` selects it for default loggers too.
  An empty or unset destination walks up from `process.cwd()` to the nearest
  ancestor `node_modules/`,
  then appends JSONL records to
  `<that project>/node_modules/.monochromatic/{YYYY-MM-DD}/{timestamp}.{uuid}.log.jsonl`
  via `node:fs/promises`.
  One file per process under a UTC day directory,
  created on the first record:
  verification probes writability with a
  `{timestamp}.{uuid}.probe.tmp` round-trip it always removes,
  so a process that writes no records leaves no files.
  Appends run in invocation order within each sink;
  a large record finishes every filesystem write before the next record starts.
  Each returned write completion includes that wait,
  so `logger.flush()` also covers queued records.
  `MONOCHROMATIC_LOG_FILE=off` disables the file sink entirely,
  even with an explicit destination.
  Verification also sweeps the destination down to a retention window of
  14 days,
  512 MiB,
  and 50,000 files
  (`MONOCHROMATIC_LOG_MAX_AGE_DAYS`,
  `MONOCHROMATIC_LOG_MAX_BYTES`,
  and `MONOCHROMATIC_LOG_MAX_FILES` override;
  invalid or negative values fall back to the defaults),
  deleting only logger-owned names and at most one walk per five minutes
  per destination directory.
  Without an explicit or environment destination,
  a missing project or failed project-file probe falls back to persistent user state.
  One notice identifies the verified fallback directory and its reason.
  An explicit destination failure reports the error and disables that file sink without redirecting it.
  See the [file destination contract](doc/file-destinations.md)
  for layout,
  retention ownership,
  platform locations,
  and verification.
- **IndexedDB**:
   browser only;
   buffers records through the same shared policy and flush triggers as the
  sessionStorage sink and stores each newline-joined JSONL batch as one
  string value per transaction in the `monochromatic.log` database
  (`batch` store,
   auto-incremented keys,
   so concurrent tabs serialize
  without any key scheme);
   records are readable the moment their transaction settles,
   DevTools
  Application tab included,
   and survive tab close and browser restart;
   retention trims oldest-first past 2048 stored batches;
   `logger.flush()` settles every issued batch transaction
- **OPFS** (exported but not a default):
   browser only;
   buffers records and appends them to the Origin Private File System as
  newline-joined JSONL batches,
   one queued stream write per batch,
   under
  the same flush triggers as the sessionStorage sink;
   keeps a `FileSystemWritableFileStream` open for the session,
   and `logger.flush()` settles every issued batch write;
   staged stream content only becomes the file on a close a crash never
  performs,
   which is why the IndexedDB sink holds the default
  persistent-browser slot instead (see `DECISIONS.md`)
- **sessionStorage**:
   available wherever `globalThis.sessionStorage` exists (browsers,
   Node,
   Deno);
   buffers records and stores them as newline-joined JSONL batches under
  `monochromatic.log.{n}` keys with an auto-incrementing counter;
   one uniform write path on every runtime flushes a batch when it reaches
  32 KiB,
   when a record's severity is `warn` or worse,
   after 250 ms of quiet,
  on `pagehide`/document-hidden where those events exist,
   and on
  `logger.flush()`;
   caps its own footprint at half the runtime's default sessionStorage quota
  (a measured per-runtime heuristic:
   5 MiB on Node and the browser engines,
  10 MiB on Deno),
   evicting its oldest batches first and reclaiming further space reactively
  if the real store still overflows
- **localStorage**:
   available wherever `globalThis.localStorage` round-trips (browsers,
   Deno,
   Node launched with `--localstorage-file`;
   flagless Node skips the probe silently);
   buffers through the same shared policy and flush triggers as the
  sessionStorage sink,
   but stores each batch under a run-scoped key
  (`monochromatic.log.{stamp}.{nonce}.{index}`) because localStorage is shared
  across tabs and survives restarts;
   adopts entries left by earlier runs and evicts oldest-first,
   capping the combined footprint at half the runtime's default localStorage
  quota (5 MiB on Node and the browser engines,
   just under 10 MiB on Deno);
   the one web storage sink whose records remain inspectable after tab close
  or a full browser restart
- **noop**:
   discards all records;
   a stand-in that disables logging without removing log calls

Each sink is a factory.
The cross-platform ones,
 `createConsoleSink()`,
 `createSessionStorageSink()`,
 `createLocalStorageSink()`,
 and `createNoopSink()`,
 are exported under the `sinks` namespace of the root entry.
`createFileSink()` is exported from `@monochromatic-dev/logger-core/node`,
 and `createIndexedDbSink()` and `createOpfsSink()` from `@monochromatic-dev/logger-core/browser`,
 so importing a platform-only sink is the consumer's own assertion of the platform.
A sink instance keeps its own buffers,
 streams,
 and counters,
 so independent loggers never
share state.

Async sinks are fire-and-forget;
 the log call never blocks the caller.
Call `logger.flush()` before assertions or process shutdown to wait for startup
verification,
 pending sink writes,
 and sink-owned flush hooks.
A sink is dropped only when its `verify` reports the backend unavailable (resolves
`false` or rejects);
 a sink whose `flush` hook rejects is also dropped.
Individual `write` failures are the sink's own concern and do not disable the backend,
so one transient I/O hiccup never silently kills a sink for the rest of the run.

## Console output safety

Log text can carry attacker-influenced content,
 and a terminal treats control characters as commands (clear screen,
 set title,
 move the cursor,
 write the clipboard).
The console sink renders every C0 control except newline and tab,
 `DEL`,
 and every C1 control as a `\uXXXX` escape before the text reaches `console.*` or `process.stderr`,
 so the attempted sequence stays visible but inert.
Newlines and tabs pass through because multi-line messages are core.
The JSONL sinks need no such step:
 `JSON.stringify` already escapes control characters.

## Log record format

Every sink receives a `LogRecord`:

```ts
type LogRecord = {
  level: Level; // 'trace' | 'debug' | 'info' | 'warn' | 'error' | 'fatal'
  message: string;
  timestamp: number; // Date.now()
};
```

File,
 IndexedDB,
 OPFS,
 sessionStorage,
 and localStorage sinks write records as one JSON object per line (JSONL).

## Error handling

- The default logger is built by the first log or flush call,
   never at import;
   there is no readiness promise to await,
   because `flush()` awaits verification and startup replay internally
- `logger.flush()` awaits startup verification,
   pending sink writes,
   and sink-owned flush hooks,
   all under one deadline when the host provides `setTimeout` and
   `clearTimeout` (`flushDeadlineMs`,
   default `DEFAULT_FLUSH_DEADLINE_MS`,
   5000 ms).
   When the deadline elapses the logger reports one console breadcrumb,
   drops the in-flight writes from its view (sinks expose no cancellation,
   so the work continues in the background),
   and resolves,
   so a wedged backend cannot hang a shutdown in hosts with timers.
   Without both timer primitives,
   the logger awaits operations directly;
   a custom sink that never settles can prevent verification or flush from finishing
- Throws at log time once initialization has completed with no available backend.
   The console sink verifies wherever `queueMicrotask` and usable
   `console.log` or all required level methods exist,
   so this is reachable only through `createLogger` with sinks that all fail verification
- A sink is dropped when its `verify` reports unavailable,
   runs past `verifyTimeoutMs` in hosts with timers,
   or its flush hook rejects;
   remaining sinks continue,
   and a late verify answer after the limit is ignored
- Individual `write` failures are handled per sink and do not disable the backend
- Records logged before every sink has answered buffer under `STARTUP_BUFFER_CAP`;
   on overflow the oldest is dropped and the count is reported as one `warn` record after initialization,
   never silently

## Callback failure observation

Use `observeLoggerCallbacks(l)` when a caller-supplied logger must not replace an operation's outcome
with a telemetry exception.
Wrap it before adding operation tags:

```ts
// consumer.ts: caller supplies l: Logger.
import { observeLoggerCallbacks, tagged, } from '@monochromatic-dev/logger-core';

const observed = observeLoggerCallbacks(l);
const step = tagged({ tag: 'persist-result', l: observed.logger, });
step.info('output retained');
const abnormalCallbacks = observed.snapshot();
```

The facade performs each requested callback lookup and invocation lazily,
using the original receiver and message.
A failed callback does not prevent later requested messages.
It catches synchronous method/getter exceptions and rejection from an explicitly awaited `flush()`.
It never inspects or retains a thrown value,
adds a sink,
retries a message,
starts a timer or requests a flush on its own.

`snapshot()` returns a new frozen array of observed callback names,
deduplicated in this fixed order:
`debug`,
`error`,
`fatal`,
`flush`,
`info`,
`trace`,
`warn`.
Previously returned snapshots do not change.
The facade and observation object are also frozen.
Take the terminal snapshot after the operation's last requested log or flush callback.

These names report callback failures,
not whether a sink delivered or stored a message.
The operation still owns cancellation and its primary result or error.
The facade cannot contain blocking callbacks,
process exit or filesystem mutation.
Void level callbacks must remain synchronous;
unobserved rejected promises returned by those methods are outside the contract.

## Custom loggers

The default `logger` is `createLogger` applied to the default sink set,
 and it stays
zero-config.
 `createLogger` is also exported for building a logger over an explicit sink
list,
 for example to write to a fixed subset of backends or to inject a fake in tests.

```ts
import { createLogger, sinks, } from '@monochromatic-dev/logger-core';

const { logger, initPromise, } = createLogger({ sinks: [sinks.createNoopSink()], },);
logger.info('goes nowhere');
await initPromise; // optional; flush() awaits it internally
```

Raise the flush deadline for a slow but working backend,
 such as a network filesystem:

```ts
import { createLogger, } from '@monochromatic-dev/logger-core';
import { createFileSink, } from '@monochromatic-dev/logger-core/node';

const { logger, } = createLogger({
  sinks: [createFileSink(),],
  flushDeadlineMs: 30_000,
  verifyTimeoutMs: 30_000,
},);
```

A custom sink is any object satisfying the `Sink` interface,
 a single self-describing
adapter carrying `verify`,
 `write`,
 and an optional `flush`:

```ts
import type { LogRecord, Sink, } from '@monochromatic-dev/logger-core';

function createArraySink(): { records: LogRecord[]; sink: Sink; } {
  const records: LogRecord[] = [];
  const sink: Sink = {
    verify: () => Promise.resolve(true),
    write: (record) => {
      records.push(record);
      return Promise.resolve();
    },
  };
  return { records, sink, };
}
```

`verify`,
 `write`,
 and `flush` are all async (returning `Promise`) so the logger awaits
them uniformly.

## Design decisions

See [DECISIONS.md](DECISIONS.md) for rationale on:

- No sub-logger hierarchy;
   per-component filtering is a log viewer problem
- String-only messages;
   callers own serialization;
   no auto-stringify
- Sinks are self-describing factories;
   the logger owns availability
- Write failures do not disable a sink;
   only verify failure does
- localStorage and IndexedDB sink designs and measurements
- Console output neutralizes control characters
- `flush()` has a deadline
- Sinks verify concurrently under a time limit
- The startup buffer is bounded and overflow is reported
- Platform-specific sinks live behind `./node` and `./browser`
- Zero-config at import,
   no configure step (the logtape migration observations)

## Source files

- `src/types.ts`:
   `Logger`,
   `LogRecord`,
   `Sink`,
   `SinkFlush`,
   `Verify`,
   `Level` type definitions
- `src/create-logger.ts`:
   `createLogger({ sinks })` orchestration (verify,
   startup replay,
   flush)
- `src/logger.ts`:
   default singleton built by applying `createLogger` to the default sinks
- `src/node.ts`:
   the `./node` subpath entry,
   `createFileSink()`
- `src/browser.ts`:
   the `./browser` subpath entry,
   `createIndexedDbSink()` and `createOpfsSink()`
- `src/default-sinks.node.ts` and `src/default-sinks.neutral.ts`:
   the two default sink lists,
   selected at build time through the `#default-sinks` entry of `package.json` `imports`
- `src/artifact-platform-split.unit.test.ts`:
   guard that reads every chunk of both builds and rejects dynamic imports and cross-platform leaks
- `src/tagged.ts`:
   `tagged()` wrapper for composable prefixes
- `src/observe-callbacks.ts`:
   callback exception containment with detached,
   names-only observation snapshots
- `src/observe-callbacks.unit.test.ts`:
   built-artifact forwarding,
   lazy lookup,
   foreign throw and flush rejection controls
- `src/sink/console.ts`:
   `createConsoleSink()`,
   verbose-mode gating and microtask batching
- `src/sink/console-control-chars.ts`:
   control-character neutralization for console-bound text
- `src/sink/file.ts`:
   `createFileSink()`,
   Node.
  js file sink (JSONL via `appendFile`)
- `src/sink/indexed-db.ts`:
   `createIndexedDbSink()`,
   default persistent-browser sink (one transaction per batch,
   retention trim)
- `src/sink/indexed-db-util.ts`:
   promise bridges for the event-based IndexedDB API
- `src/sink/opfs.ts`:
   `createOpfsSink()`,
   opt-in browser OPFS sink with persistent writable stream (batched writes)
- `src/sink/record-buffer.ts`:
   buffering stage shared by the OPFS and sessionStorage sinks
   (byte cap,
   severity flush,
   quiet-period deadline,
   page lifecycle)
- `src/sink/session-storage.ts`:
   `createSessionStorageSink()`,
   cross-runtime web storage sink (buffering,
   flush triggers)
- `src/sink/session-storage-store.ts`:
   persistence engine behind it (key allocation,
   footprint accounting,
   quota eviction)
- `src/sink/local-storage.ts`:
   `createLocalStorageSink()`,
   cross-runtime persistent web storage sink (same buffering,
   run-scoped keys)
- `src/sink/local-storage-store.ts`:
   persistence engine behind it (run identity,
   prior-run adoption,
   cross-run oldest-first eviction)
- `src/sink/local-storage-key.ts`:
   run-scoped key building,
   strict parsing,
   and eviction ordering
- `src/sink/local-storage-quota.ts` and `src/sink/session-storage-quota.ts`:
   fill-probed per-runtime quota tables
- `src/sink/web-storage-runtime.ts` and `src/sink/web-storage-quota-error.ts`:
   host-runtime detection and quota-overflow recognition shared by both
  web storage engines
- `src/sink/noop.ts`:
   `createNoopSink()`,
   discards all records
