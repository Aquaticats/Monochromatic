# `module-logger` 0.4.0 on QuickJS-ng 0.16.2: missing console methods and timers prevent logging

## Symptom

A neutral `module-logger` 0.4.0 bundle imported into the QuickJS-ng `qjs` executable
 has no verified default sink after its first log call.
`await logger.flush()` fails while reporting its own failure:

```text
TypeError: not a function
    at reportLoggerInternalError
    at flushAll
```

The issue reports a 2.3 ms VM startup difference when importing the logger into a helper,
 and the affected consumer temporarily uses `helpers/src/qjs-log.ts` in `Aquaticat/labwc-config`.
That consumer writes to stderr;
 QuickJS-ng's `console.log` writes to stdout.
These are different contracts,
 even if the exception is repaired.

## Root cause

QuickJS-ng v0.16.2's `quickjs-libc.c:4603-4615` constructs a console with only a `log` property:

```c
console = JS_NewObject(ctx);
JS_SetPropertyStr(ctx, console, "log",
                  JS_NewCFunction(ctx, js_print, "log", 1));
JS_SetPropertyStr(ctx, global_obj, "console", console);
```

Its `quickjs-libc.c:4595-4596` prints that method to standard output:

```c
fwrite(b.buf, 1, b.size, stdout);
fflush(stdout);
```

At the pre-fix repository commit `193dd7eab`,
 `package/logger/core/src/sink/console.ts:381-385` required a severity method,
 not `console.log`,
 to verify:

```ts
const testFn = hasProcessStderr() ? console.info : console.debug;
if ((typeof testFn) !== 'function')
  return Promise.resolve(false,);
```

The same pre-fix file at `package/logger/core/src/sink/console.ts:282-289`
 only called the severity method when present;
 a missing method silently lost the record:

```ts
const consoleFn = console[method];
if ((typeof consoleFn) === 'function') {
  consoleFn.call(
    console,
    text,
  );
}
```

The `qjs` CLI does not install global timer functions:
 `quickjs-libc.c:4444-4445` registers `setTimeout` and `setInterval` on the
 `qjs:os` module,
 while `quickjs-libc.c:4603-4624` adds console and print globals.
QuickJS-ng's `quickjs.c:57218` provides `queueMicrotask` as a global:

```c
JS_CFUNC_DEF("queueMicrotask", 1, js_global_queueMicrotask ),
```

Before the fix,
 `package/logger/core/src/create-logger.ts:351-358,560-565`
 called `withTimeout` during verification and flush.
The dependency `package/async-time/core/src/with-timeout.ts:56-61`
 requires a global timer even if the operation's promise has already resolved:

```ts
const timer = setTimeout(
  function onTimeout() {
    reject(new Error(`Timed out after ${String(ms,)}ms: ${label}`,),);
  },
  ms,
);
```

Every verification therefore failed in `qjs`,
 including a hypothetical console sink that accepted `console.log`.
The pre-fix `package/logger/core/src/error-format.ts:40-42`
 then assumed `console.warn` when reporting the caught error:

```ts
console.warn(
  `logger internal error: ${context}: ${caughtValueText(error,)}`,
);
```

This secondary exception obscured the first failure.
The runtime's limited console and timers are documented capabilities,
 not a QuickJS-ng regression.
The repository's logger made unsupported assumptions about them.

## Verification

The runtime probe used the official `qjs-linux-x86_64` release asset at tag `v0.16.2`,
 source commit `1ab8676f4b6d6d669baeb5f21790fb9734636a20`:

```sh
# In a private disposable directory, after downloading the release asset
chmod 700 qjs-linux-x86_64
./qjs-linux-x86_64 --eval "console.log(JSON.stringify({console:Object.keys(console),timer:typeof setTimeout,microtask:typeof queueMicrotask,promiseResolvers:typeof Promise.withResolvers,errorIsError:typeof Error.isError,dispose:typeof Symbol.dispose}))"
```

Observed:

```json
{"console":["log"],"timer":"undefined","microtask":"function","promiseResolvers":"function","errorIsError":"function","dispose":"symbol"}
```

The committed regression test `package/logger/core/src/quickjs-compat.unit.test.ts`
 simulates a console with only `log`,
 absent timers,
 and absent reporting methods
 against the built neutral artifact.
Build and run it with:

```sh
# From the Monochromatic repository root
mise run buildAndTest -- package/logger/core/src/quickjs-compat.unit.test.ts
```

The test failed at `193dd7eab` in the three reported paths and passes after the fix.
For a real end-user boundary check,
 place this source in a disposable `smoke.mjs` beside a `node_modules/@monochromatic-dev/logger-core`
 symlink to this package directory,
 then bundle it with Deno and run it with the release `qjs` executable:

```js
// smoke.mjs
import * as std from 'qjs:std';
import { createLogger, logger, sinks, tagged } from '@monochromatic-dev/logger-core';
std.err.puts('imported\n');
const l = tagged({ tag: 'qjs-smoke' });
l.info('consumer info');
l.warn('consumer warn');
l.error('consumer error');
await logger.flush();
const { initPromise } = createLogger({
  sinks: [
    { verify() { throw new Error('failure report'); }, write() { return Promise.resolve(); } },
    sinks.createConsoleSink(),
  ],
});
await initPromise;
std.err.puts('flushed-and-reported\n');
std.err.flush();
```

```sh
# Paths here assume smoke.mjs and its node_modules symlink are in $scratch
mise run //package/logger/core:build
deno bundle --platform=browser --external=qjs:* --node-modules-dir=manual --output "$scratch/smoke.bundle.mjs" "$scratch/smoke.mjs"
"$scratch/qjs-linux-x86_64" --module "$scratch/smoke.bundle.mjs" > "$scratch/stdout.txt" 2> "$scratch/stderr.txt"
```

On 2026-09-24,
 stdout carried the tagged info,
 warn,
 and error lines and the internal `failure report`;
 stderr carried `imported` and `flushed-and-reported`.
Import and `tagged()` produced no logger output before the first log call.
The final marker proves that `flush()` and the failing sink's verification completed.
Running an otherwise identical fixture with `logger.error('before immediate exit')`
 followed by `std.exit(0)` without `await logger.flush()` produced no log line:
 the console sink batches into a microtask and explicit exit bypasses it.

Working cases:

- `console.log` with `queueMicrotask` and no global timers now verifies and writes all unsuppressed levels.
- A complete severity-method console without `console.log` still verifies.
- A host with both timer primitives retains bounded verification and flush,
  as covered by the full logger unit suite.

Failing or deliberately unsupported cases:

- A partial severity-method console without `console.log` cannot cover every level and does not verify.
- An absent `queueMicrotask` cannot support the console sink's batching contract and does not verify.
- A timerless host with a custom sink whose promise never settles cannot enforce a deadline.
- An explicit `std.exit()` before the queued flush can discard buffered records.

For an indicative startup check,
 a mount-free `node:26-slim` container with the `qjs` binary and bundles baked in
 ran a bounded Node `spawnSync` harness under `podman run --memory=2g --cpus=2 --rm`.
Three unchanged-build rounds (five warmups and forty invocations each)
 gave medians of 3.403,
 3.534,
 and 3.562 ms for the bundled logger fixture,
 versus 1.650,
 1.643,
 and 1.697 ms for a bare three-line `console.log` control.
The logger fixture additionally creates a failing custom sink,
 so the difference is not an isolated import cost.
The unchanged-build median spread was 0.159 ms for the logger
 and 0.054 ms for the control.
This container is not the physical desktop and does not establish the consumer's full interaction budget.

## Verified workarounds

- For an unreleased package version,
  the consumer's current `qjs-log.ts` uses `qjs:std` and writes to stderr directly.
  The verified primitive is:

  ```js
  // Direct QuickJS-ng stderr logging
  import * as std from 'qjs:std';
  std.err.puts('[error] [1970-01-01T00:00:00.000Z] [helper] failed\n');
  std.err.flush();
  ```

  This preserves journal-oriented stderr and synchronous exit behavior,
  but duplicates formatting and omits the shared logger's multi-sink behavior.
- Use the fixed neutral artifact with `await logger.flush()` before an explicit exit.
  It retains tagging and level formatting without requiring global timers,
  but uses stdout on QuickJS-ng and adds parse/startup work.
  It is not a drop-in replacement for the stderr shim.

## What does not work

- Only changing the console verifier to accept `console.log` leaves
  `withTimeout` calling missing global `setTimeout` during verification and flush.
- Only guarding `console.warn` leaves sink verification failing,
  and a console-only host still drops missing severity methods.
- Importing `qjs:os` as a global timer substitute does not automatically install
  `setTimeout` on `globalThis`;
   its timer functions are module exports
  (`quickjs-libc.c:4444-4445`):

  ```c
  JS_CFUNC_MAGIC_DEF("setTimeout", 2, js_os_setTimeout, 0 ),
  JS_CFUNC_MAGIC_DEF("setInterval", 2, js_os_setTimeout, 1 ),
  ```

- The repaired console sink's microtask flush alone does not deliver a line
  before an immediate `std.exit()`;
  `await logger.flush()` did deliver it.

## Upstream filing decision

The repository's `.out-of-scope/` entries were checked;
 none exempts QuickJS-ng console or timers.
Searches of open and closed QuickJS-ng issues and PRs for
 `console.warn console.error` found no matching thread.
No QuickJS-ng report is warranted:

- **Upstream fault:**
   no.
  The QuickJS-ng standard-library documentation at `docs/docs/stdlib.md:24-26`
  explicitly lists `console.log`,
   and the implementation matches it:

  ```md
  ### `console.log(...args)`

  Same as `print()`.
  ```
- **Upstream fixability:**
   yes in principle,
  but adding browser console aliases or global timers would expand QuickJS-ng's API,
  not repair this logger's assumptions.
- **Supported use case:**
   no evidence that `qjs` promises a full browser console
  or global timers;
   its documented `qjs:os` module owns the timers.
- **Contribution welcome:**
   the tag's `README.md` and repository file list
  showed no contribution ban or special AI-report policy;
  that observation does not make an unrelated feature request appropriate.
- **Likely upstream fix:**
   no maintainer commitment was found for this request;
  the reported behavior follows the documented API,
  so this is not an upstream defect to fix.
- **Architecture-compatible upstream prototype:**
   not applicable because the
  correction belongs to `module-logger`;
  the verified package patch and regression test are committed here.

**Upstream filing artifact:**
 nothing to add to the QuickJS-ng tracker.
A new issue there would misattribute the failure.
The relevant downstream follow-up belongs in `Aquaticat/labwc-config`
 to evaluate migration without changing its hot-path budget or stderr/journal behavior.
