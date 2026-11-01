# `module-logger` at a43e52f: changing `trace` to `debug` does not establish a suppressed-entry speedup

## Symptom

[Issue #130](https://github.com/Aquaticat/Monochromatic/issues/130) proposed changing function-entry
`trace` calls to `debug` because it expected an advantage even with the console sink disabled;
the issue specifically mentioned caller-side string formatting.
The former version of this file separately claimed that every trace call captures a stack
and asserted a 5 to 20 times improvement without a runnable measurement.
That claim confused the logger call with console emission.

The production entry call is `parseCss` in `package/css/core-edit/src/parse.ts:42-48`.
The other production trace in `package/jsonc-edit/core-ts/src/parse-jsonc.ts:42-52` reports
an exception-driven parser fallback,
 not function entry.
 `package/logger/core/src/tagged.ts:67-68`
forwards the public trace method and is not a call site to migrate.

## Root cause

The logger creates the same kind of record for either level and dispatches it to every
available sink.
 `package/logger/core/src/create-logger.ts:455-472,579-585`:

```ts
function createMethod(level: Level,): (message: string,) => void {
  return function logAtLevel(message: string,): void {
    if ((!state.hasAvailableSink) && state
      .initialized)
      throw new Error('No logging backends available',);

    /**
     Shared LogRecord forwarded to every available sink; built once per call so all sinks see the same timestamp.
     */
    const record: LogRecord = {
      level,
      message,
      timestamp: Date.now(),
    };
```

`package/logger/core/src/create-logger.ts:488-493` dispatches the record:

```ts
      availableIndices.forEach(function writeToSink(entryIndex,) {
        writeRecordToEntry({
          entryIndex,
          record,
        },);
      },);
```

`package/logger/core/src/create-logger.ts:578-586` binds both levels through that factory:

```ts
const logger: Logger = {
  debug: createMethod('debug',),
  error: createMethod('error',),
  fatal: createMethod('fatal',),
  flush: flushAll,
  info: createMethod('info',),
  trace: createMethod('trace',),
  warn: createMethod('warn',),
};
```

The entry caller constructs the message before calling the logger;
`package/css/core-edit/src/parse.ts:42-48` contains:

```ts
tagged({
  tag: parseCss.name,
  l,
},)
  .trace(`parsing ${String(source.length,)} characters`,);
```

The tagged wrapper only prepends its tag,
 regardless of the level;
`package/logger/core/src/tagged.ts:48-68` contains:

```ts
debug: function debug(message: string,): void {
  l.debug(`${prefix}${message}`,);
},
```

`package/logger/core/src/tagged.ts:67-69` similarly forwards trace:

```ts
trace: function trace(message: string,): void {
  l.trace(`${prefix}${message}`,);
},
```

The console sink suppresses **both** levels when verbose output is off;
`package/logger/core/src/sink/console.ts:26-29,497-504` contains:

```ts
const SILENT_LEVELS: ReadonlySet<string> = new Set([
  'debug',
  'trace',
],);
```

`package/logger/core/src/sink/console.ts:497-504` checks this set:

```ts
if ((!getVerbose()) && SILENT_LEVELS.has(record.level,))
  return Promise.resolve();
```

This suppression occurs **after** message formatting and logger dispatch.
 It applies to the
console sink,
 not every sink:
 `package/logger/core/src/default-sinks.node.ts:34-42`
includes `createFileSink()`,
 whose `write` method appends JSON records
(`package/logger/core/src/sink/file.ts:266-279`):

```ts
createConsoleSink(),
createSessionStorageSink(),
createLocalStorageSink(),
createFileSink(),
```

```ts
await appendFile(
  filePath,
  `${JSON.stringify(record,)}\n`,
);
```

Therefore a suppressed console does not imply a zero-cost log call or
suppressed file output.

With verbose console output enabled,
 `trace` and `debug` deliberately diverge.
`package/logger/core/src/sink/console.ts:143-151,255-286` maps trace to `console.trace`;
debug writes to process stderr when present and otherwise uses `console.debug`:

```ts
const LEVEL_TO_CONSOLE_METHOD: Record<Level,
  'debug' | 'error' | 'info' | 'trace' | 'warn'> = {
    debug: 'debug',
    error: 'error',
    fatal: 'error',
    info: 'info',
    trace: 'trace',
    warn: 'warn',
  };
```

`package/logger/core/src/sink/console.ts:264-286` selects stderr for debug and
calls the mapped console method for trace:

```ts
  if ((level === 'debug') && writeDebugRunToProcessStderr(text,))
    return;
  try {
    /**
     Name (not the function reference) of the matching `console.*` method; resolved lazily so post-import hot patches still apply.
     */
    const method = LEVEL_TO_CONSOLE_METHOD[level];
    /**
     Resolved console method looked up by name; may be missing or non-callable in stripped runtimes, which the guard handles.
     */
    const consoleFn = console[method];
    if ((typeof consoleFn) === 'function') {
      consoleFn.call(
        console,
        text,
      );
    }
```

Node 26.8.2 emits stack frames for `console.trace('trace probe')`,
 unlike
`console.debug('debug probe')` (probed with both calls in a bounded Node container).
That occurs when the console emits the trace,
 not at every `logger.trace()` invocation.
Changing the `parseCss` call would also change its record level and formatted prefix;
`package/logger/core/src/sink/console.ts:168-173` formats the prefix as:

```ts
return `[${record.level}] [${
  new Date(record.timestamp,)
    .toISOString()
}] ${neutralizeControlCharacters(record.message,)}`;
```

The verbose console output and its diagnostic stack would change too.

## Verification

The logger source was inspected at `a43e52f03bb3021f2b085f480bcde27364cc14b8`.
The fixture first appears in `e205c9e42`;
 check out that commit or a descendant,
then build the unchanged logger with `mise run //package/logger/core:build:js:node`.
The repeatable fixture is
[performance.logging.bench.mjs](performance.logging.bench.mjs).
 It calls a tagged logger
with the same message construction as `parseCss`,
 alternates level order after warmup,
reports individual times in microseconds per call,
 and separately tests a forced stack
capture as a positive control.
 The container is limited to two CPUs,
 2 GiB,
 no network,
and read-only mounts of the fixture and built logger.
 Run each mode separately:

```bash
# From the repository root: no-op sink
podman run --rm --memory=2g --cpus=2 --pids-limit=128 --network=none \
  --security-opt=label=disable \
  --volume="$PWD/doc/troubleshooting:/work/doc/troubleshooting:ro" \
  --volume="$PWD/package/logger/core/dist/final/node:/work/package/logger/core/dist/final/node:ro" \
  --workdir=/work docker.io/library/node:26-slim \
  node doc/troubleshooting/performance.logging.bench.mjs noop
```

Run the suppressed and emitted modes with the same container bounds and mounts:

```bash
# From the repository root: suppressed console
podman run --rm --memory=2g --cpus=2 --pids-limit=128 --network=none \
  --security-opt=label=disable \
  --volume="$PWD/doc/troubleshooting:/work/doc/troubleshooting:ro" \
  --volume="$PWD/package/logger/core/dist/final/node:/work/package/logger/core/dist/final/node:ro" \
  --workdir=/work docker.io/library/node:26-slim \
  node doc/troubleshooting/performance.logging.bench.mjs suppressed
```

```bash
# From the repository root: emitted console, writing timing JSON to stdout
podman run --rm --memory=2g --cpus=2 --pids-limit=128 --network=none \
  --security-opt=label=disable \
  --volume="$PWD/doc/troubleshooting:/work/doc/troubleshooting:ro" \
  --volume="$PWD/package/logger/core/dist/final/node:/work/package/logger/core/dist/final/node:ro" \
  --workdir=/work --env=MONOCHROMATIC_VERBOSE=true docker.io/library/node:26-slim \
  node doc/troubleshooting/performance.logging.bench.mjs emitted 2>/dev/null
```

The emitted mode drains each record in its own microtask so console stack costs are visible;
it does not model bursts grouped into one console call.
The redirect discards the benchmark's diagnostic console output,
 not its timing JSON or process exit code.
Before trusting that redirected output,
 a separate integration probe initialized the same console sink in the same bounded Node image,
logged one tagged trace and one tagged debug record,
 and flushed both.
Its stdout was empty;
 its stderr held both entries,
 trace stack frames,
 and no logger-internal error.
Both methods emitted to stderr in this Node runtime,
 so the observed difference is not a change of output stream.
Reproduce the output check without redirecting stderr:

```bash
# From the repository root: verify both enabled console paths
podman run --rm --memory=2g --cpus=2 --pids-limit=128 --network=none \
  --security-opt=label=disable \
  --volume="$PWD/package/logger/core/dist/final/node:/logger:ro" \
  --env=MONOCHROMATIC_VERBOSE=true docker.io/library/node:26-slim \
  node --input-type=module -e '
    import { createLogger, sinks, tagged } from "/logger/index.mjs";
    const { logger, initPromise } = createLogger({ sinks: [sinks.createConsoleSink()] });
    await initPromise;
    const entry = tagged({ tag: "probe", l: logger });
    entry.trace("trace entry");
    await logger.flush();
    entry.debug("debug entry");
    await logger.flush();
  '
```

On Node 26.8.2 in `node:26-slim`,
 the ten samples per level were:

- No-op sink:
   trace 0.399 to 1.296 µs/call;
   debug 0.389 to 1.650 µs/call.
  The explicit-stack positive control was 2.994 to 6.926 µs/call and produced
  a nonzero checksum.
   The probe can detect stack-capture overhead.
- Suppressed console:
   trace 0.335 to 1.286 µs/call;
   debug 0.333 to 1.930 µs/call.
  Their bands overlap,
   so this run does not establish a suppressed-console gain.
- Emitted console,
   one flush per record:
   trace 7.357 to 8.039 µs/call;
  debug 3.475 to 4.130 µs/call.
   This comparison changes the emitted stack and
  recorded level,
   so it does not establish an equivalent-output speedup.

These numbers cover logger-entry calls,
 **not** full CSS parsing throughput,
 browser
console behavior,
 or real file-sink throughput.
 The source comparison explains why
switching severity cannot avoid caller-side formatting or record construction.

## Verified resolution

Keep the function-entry trace in `parseCss` and the fallback trace in `parseJsonOrMiss`.
There is no supported suppressed-console improvement from replacing either with debug;
retaining trace also preserves the diagnostic level and verbose console stack.
 The tradeoff
is that verbose console emission continues to pay for its stack,
 as designed.
 If a future
specific caller favors a stack-free debug message,
 compare that workload and accept the
change in level/output explicitly rather than applying a workspace-wide substitution.
No logging-performance rule was added to `AGENTS.md`,
 at the user's explicit request despite that issue acceptance criterion.

## What does not work

- Blindly replacing `l.trace(`:
   the actual `parseCss` entry uses `tagged(...).trace(`;
  the literal `l.trace(` in `tagged.ts` is the forwarding implementation.
- Treating `console.trace()` benchmarks as measurements of a suppressed `logger.trace()`:
  `package/logger/core/src/create-logger.ts:455-490` creates and forwards a record;
  `package/logger/core/src/sink/console.ts:497-504` suppresses it before emission.
- Treating an overlapping timing band as proof of equivalence:
   the positive control
  confirms sensitivity to stack work,
   but this benchmark cannot prove that all
  implementations or workloads have identical timing.

## Upstream filing decision

This is a repository-owned documentation correction,
 not a third-party logger defect.
The `.out-of-scope/` entries contain no matching logger exemption.
 Issue #130 already
tracks this exact premise;
 there is no second issue or additive upstream comment to file.

1. **Upstream fault?**
    No:
    the severity paths and sink behavior are working as implemented;
   the former troubleshooting note was wrong.
2. **Fixable upstream?**
    Yes:
    this repository can correct its own documentation.
3. **Supported use case?**
    Yes:
    tagged severity methods and console filtering are public APIs.
4. **Contribution welcome?**
    This is our own repository,
    not an outside contribution.
5. **Likely fix?**
    The existing issue requests verification and correction if the premise fails.
6. **Minimal fix prototyped?**
    Not applicable to a logger defect:
    the measured resolution is
   to retain the calls and correct the documentation.

There is nothing to add to a separate upstream filing.
