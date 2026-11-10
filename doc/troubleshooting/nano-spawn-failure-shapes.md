# nano-spawn 2.1.0 reports every failure as one error object, and a failed start carries no status

## Symptom

A caller that forwards a child's captured output and status must decide what a
caught spawn value is.
Two outcomes look alike and are not:

- a command that ran and failed,
  which reports `exitCode` or `signalName`
  plus the captured streams;
- a command that never started,
  which reports neither a status nor a signal,
  so a caller that renders only those three fields prints nothing at all while
  the failure is real.

`task-depends` showed the second case as silence until
`package/monorepo-task/core-exec/src/depends-exec.ts` learned to report it,
run against a source file whose output check always reports staleness:

```text
$ task-depends -s 'src.txt' -o 'sh:echo -Infinity' -- no-such-command-for-the-probe-8f3a
$ echo $?
1
```

Nothing was printed,
on either stream,
and the process still exited 1.
The command line and the reason now arrive on standard error;
"Verified workarounds" has the line that produces them.

A timed-out child is a third case,
one layer up:
`package/monorepo-task/core-exec/src/depends-bounded-wait.ts` wraps the
subprocess error in `BoundedWaitTimeoutError.cause`,
so a caller that reads the thrown value's own fields sees no captured output
even when the child wrote some before the bound ended it.
That output was dropped for the same reason:
the
consumer read only the bound error's own fields.

## Root cause

### Every rejection is one error class

nano-spawn funnels each failure through `getResultError`
([`source/result.js:44-49`][nano-result-error]),
whose instance is always a `SubprocessError`
([`source/result.js:51-57`][nano-result-instance]):

- a non-zero status or an ending signal throws a `SubprocessError` directly
  ([`source/result.js:34-42`][nano-result-failure]);
- a start failure,
  a stream error,
  or any other rejection is wrapped in one
  ([`source/spawn.js:32`][nano-spawn-start],
  [`source/result.js:19-22`][nano-result-catch]).

So a caller never catches a primitive,
and never catches a bare Node error.
It also never catches anything else that carries a subprocess shape.

### A failed start has no status to report

`SubprocessError` is enriched from `getErrorOutput`
([`source/result.js`][nano-result-error]) and `getOutputs`
([`source/result.js:65-73`][nano-result-outputs]):

```js
const getErrorOutput = ({exitCode, signalCode}) => ({
  ...(exitCode < 1 ? {} : {exitCode}),
  ...(signalCode === null ? {} : {signalName: signalCode}),
});
```

A child that never started has `exitCode` undefined,
so the key is dropped,
and `signalCode` null,
so the signal key is dropped too.
What is left is `message` (`Command failed: <command>`),
`cause` (the `ENOENT`
error),
empty `stdout`/`stderr`,
and `isCanceled`.
A consumer that reads only `exitCode`,
`signalName`,
`stdout` and `stderr`
therefore has nothing to show,
and the failure text sits in fields it never
prints.

### The bound keeps the subprocess error one level down

`spawnBounded` rethrows the bound as its own error with the subprocess error in
`cause`:

```ts
throw new BoundedWaitTimeoutError({
  timeoutMs,
  cause: error,
},);
```

`BoundedWaitTimeoutError` declares only `timeoutMs`,
so the captured output the
subprocess error holds is not on the value a caller catches.

## Verification

Environment:
nano-spawn 2.1.0 (catalog range `>=2.1.0` in
`pnpm-workspace.yaml`),
Node 26.13.0,
Linux x86_64.

The probe imports
`package/monorepo-task/core-exec/src/depends-exec.ts` directly and runs real
children through `executeWithCollapsedOutput`,
capturing both process streams
and `process.exitCode`.
Run on 2026-11-09,
before the consumer reported either shape,
its catalog was:

```json
{"case":"exit 3 with output","exitCode":3,"stdout":["child-out\n"],"stderr":["child-err\n"]}
{"case":"missing executable","exitCode":1,"stdout":[],"stderr":[]}
{"case":"timeout","exitCode":1,"stdout":[],"stderr":["[task-depends] command timed out …"]}
{"case":"exit 4 with allowFailure","stdout":[],"stderr":[]}
{"case":"killed by signal","exitCode":1,"stdout":[],"stderr":["… by signal: SIGKILL\n"]}
{"case":"timeout after output","exitCode":1,"stdout":[],"stderr":["[task-depends] timed out …"]}
```

In that run the last two cases were the point:
the signal-ended child was reported,
and the
child that printed before its bound ended had its text dropped rather than
dumped.

A second harness runs the built entry
`package/monorepo-task/core-exec/dist/final/node/depends.mjs` the way a task
runs it,
in a disposable folder holding `src.txt`,
with `-s src.txt -o 'sh:echo -Infinity'` so the command after `--` always runs,
and reports its status and both streams.
Run on 2026-11-10 against the build that reports both shapes,
the same cases left:

```json
{"case":"exit 3 with output","exitCode":3,"stdout":["child-out"],"stderr":["child-err"]}
{"case":"missing executable","exitCode":1,"stdout":[],"stderr":["[task-depends] could not start `no-such-command-for-the-probe-8f3a`: spawn no-such-command-for-the-probe-8f3a ENOENT. Check that the executable is installed and on PATH, and that the task names it correctly."]}
{"case":"timeout","exitCode":1,"stdout":[],"stderr":["[task-depends] command timed out after 1 seconds and was ended (raise it with --command-timeout-seconds): sh -c sleep 60"]}
{"case":"exit 4 with allowFailure","exitCode":0,"stdout":["allowed-out"],"stderr":[]}
{"case":"killed by signal","exitCode":1,"stdout":[],"stderr":["[task-depends] command terminated by signal: SIGKILL"]}
{"case":"timeout after output","exitCode":1,"stdout":["printed-before-the-bound"],"stderr":["[task-depends] command timed out after 1 seconds and was ended (raise it with --command-timeout-seconds): sh -c echo printed-before-the-bound; echo warned-before-the-bound >&2; sleep 60","warned-before-the-bound"]}
```

`missing executable` names the command line,
the reason and what to check,
and keeps status 1.
`timeout after output` dumps both streams the child wrote before the bound
ended it,
and still ends the run as failed.

`package/monorepo-task/core-exec/src/depends-exec.unit.test.ts` pins the
reachable branches:
success discards output,
a failed child dumps it and keeps
its status,
`allowFailure` leaves the status alone,
a failed start is named and fails,
its missing file and its
permission reason are told apart,
a start the call allows still prints its
command,
the bound
and a signal are named,
and what a timed-out child printed is dumped.
The timeout line's note names the flag;
its status is the failed-run status,
because the bound's error carries no subprocess status of its own.

## Verified workarounds

Read the failure through a guard,
not through a cast of the caught value.
`package/monorepo-task/core-exec/src/tool-failure.ts:74` supplies one:
`isCapturedSubprocessFailure` accepts a value that carries a numeric
`exitCode` or a string `signalName`,
and refuses a failed start.
`package/monorepo-task/core-exec/src/depends-exec.ts:251` uses it to dump the
captured streams when there are any.

A failed start is reported from what it does carry.
`package/monorepo-task/core-exec/src/depends-exec.ts:320` names the command line
the task asked for,
takes the innermost message
(`cause.message`,
Node's own `spawn <command> ENOENT`,
which names the failed call and its code where nano-spawn's own message only
repeats `Command failed: <command>`),
and turns `cause.code` into a check a person can act on:
`ENOENT` asks about the installation and `PATH`,
`EACCES` about the file's permission and the account,
and any other code falls back to naming the command as the thing to check.
The run still ends with status 1,
because a failed start has no status of its own.

A timed-out child's captured output is kept by unwrapping the bound's `cause`
before reading the streams.
`package/monorepo-task/core-exec/src/depends-exec.ts:237` recognizes
`BoundedWaitTimeoutError`,
`package/monorepo-task/core-exec/src/depends-exec.ts:243` accepts the subprocess
error in its `cause`,
and `package/monorepo-task/core-exec/src/depends-exec.ts:294` dumps the streams.
No signal is named in that case,
because the bound ended the group itself and the caller has already printed the
line naming the bound.

Tradeoffs:
an `allowFailure` call prints the same diagnostics and leaves the exit status
alone,
and output still sitting in the child's own buffers when the group is killed is
lost,
because the bound ends the group before those writes happen.

## What does not work

- Guarding on `error !== null && typeof error === 'object'` and printing a
  fallback message in the else:
  the else never runs,
  because every rejection is
  the object above.
- Reading `message` alone for a failed start:
  it names the command line without
  saying the executable was missing.
- Relying on the catalog range for this shape:
  the wrapping is a documented
  implementation detail,
  not a contract,
  so re-run the probe when the resolved
  version changes.

## Upstream filing decision

Nothing to file.
The wrapper is deliberate:
one error class for callers,
with the original error
in `cause`.
The missing diagnostic is the consumer's,
and this repository's consumer now
handles the failed start instead of assuming it arrives with a status.

[nano-result-catch]: https://github.com/sindresorhus/nano-spawn/blob/cc231e2/source/result.js#L19-L22
[nano-result-error]: https://github.com/sindresorhus/nano-spawn/blob/cc231e2/source/result.js#L44-L49
[nano-result-failure]: https://github.com/sindresorhus/nano-spawn/blob/cc231e2/source/result.js#L34-L42
[nano-result-instance]: https://github.com/sindresorhus/nano-spawn/blob/cc231e2/source/result.js#L51-L57
[nano-result-outputs]: https://github.com/sindresorhus/nano-spawn/blob/cc231e2/source/result.js#L65-L73
[nano-spawn-start]: https://github.com/sindresorhus/nano-spawn/blob/cc231e2/source/spawn.js#L32
