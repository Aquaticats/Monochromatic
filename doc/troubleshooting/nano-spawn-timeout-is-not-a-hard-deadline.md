# nano-spawn 2.1.0 timeout can outlive its limit when child handles SIGTERM

## Symptom

A caller can pass `timeout` to nano-spawn and still receive a successful result after that interval.
The trigger is a child process that handles or ignores the default `SIGTERM` signal and exits successfully later.

A 100 ms timeout around a child with a 500 ms natural exit produced:

```json
{"ignoreTermination":true,"deadlineMode":"timeout","outcome":"resolved","durationMs":517,"subprocessDurationMs":517}
```

This matters for callers that promise a hard child-process deadline rather than a request to terminate.

## Root cause

### nano-spawn forwards Node spawn options

Nano-spawn documents `timeout`,
`signal`,
and `killSignal` as ordinary `child_process.spawn()` options in
[`readme.md:86-88`][nano-readme]:

```md
##### options.stdio, options.shell, options.timeout, options.signal, options.cwd, options.killSignal

All `child_process.spawn()` options can be passed to `spawn()`.
```

[`source/index.js:13-15`][nano-index]
normalizes those options and passes them into its subprocess and result paths:

```js
const spawnOptions = getOptions(options);
const nodeChildProcess = spawnSubprocess(file, commandArguments, spawnOptions, context);
let subprocess = getResult(nodeChildProcess, spawnOptions, context);
```

[`source/spawn.js:20`][nano-spawn-source]
then delegates directly to Node:

```js
const instance = spawn(file, commandArguments, options);
```

Nano-spawn therefore adds no stronger deadline semantic around Node's option.

### Node timeout sends a signal rather than proving exit

Node's current [`child_process.spawn()` documentation][node-spawn]
states that `timeout` uses `killSignal`,
whose default is `SIGTERM`.
Its synchronous process documentation makes the consequence explicit:
if a process handles `SIGTERM` and does not exit,
the parent continues waiting for the child to exit.

The nano-spawn result path also awaits child completion.
[`source/result.js:10-21`][nano-result]
creates a `close` promise and settles only through process or stream completion:

```js
const onClose = once(instance, 'close');

try {
  await Promise.race([
    onClose,
    ...instance.stdio.filter(Boolean).map(stream => onStreamError(stream)),
  ]);
  checkFailure(context, getErrorOutput(instance));
  return getOutputs(context);
} catch (error) {
  await Promise.allSettled([onClose]);
  throw getResultError(error, instance, context, options);
}
```

A child that ignores the timeout's `SIGTERM` can therefore continue until its own later exit.
If that exit is successful,
nano-spawn resolves successfully.

### AbortSignal alone can settle while the child survives

[`source/result.js:44-49`][nano-result-cancel]
classifies cancellation from the caller's signal state:

```js
export const getResultError = (error, instance, context, {signal}) => Object.assign(
  getErrorInstance(error, context),
  getErrorOutput(instance),
  {isCanceled: signal?.aborted === true},
  getOutputs(context),
);
```

A default abort still uses Node's default `SIGTERM`.
The reproduction observed nano-spawn reject at the deadline with `isCanceled: true`,
while the child remained alive.
A promise rejection is therefore not sufficient evidence that the child deadline was enforced.

## Verification

### Environment

- nano-spawn 2.1.0,
  npm `gitHead` `cc231e2c7b1e434a96f25f907ca2cb2f7c596e90`;
- Node 22.18.0;
- Linux x86_64 container;
- 256 MiB memory,
  one CPU,
  64 processes,
  and no network.

The harness starts a Node child that either accepts or ignores `SIGTERM`.
Each child naturally exits after 500 ms.
The parent requests a 100 ms deadline.

```js
import spawn from 'nano-spawn';
import { setTimeout as delay } from 'node:timers/promises';

const childSource = `
  process.on('SIGTERM', () => {});
  setTimeout(() => {}, 500);
`;

const subprocess = spawn(process.execPath, [
  '--input-type=module',
  '--eval',
  childSource,
], {
  stdin: 'ignore',
  timeout: 100,
});

const child = await subprocess.nodeChildProcess;
const startedAt = performance.now();

try {
  await subprocess;
  console.log({ outcome: 'resolved', durationMs: performance.now() - startedAt });
} catch (error) {
  await delay(20);
  console.log({
    outcome: 'rejected',
    durationMs: performance.now() - startedAt,
    isCanceled: error.isCanceled,
    childAlive: child.exitCode === null && child.signalCode === null,
  });
}
```

### Working behavior catalog

- Child does not handle `SIGTERM`,
  `timeout: 100`:
  rejection after 125 ms including a 20 ms liveness wait,
  `signalName: "SIGTERM"`,
  child not alive.
- Child handles `SIGTERM`,
  `AbortSignal.timeout(100)` plus `killSignal: "SIGKILL"`:
  rejection after 120 ms including the same wait,
  `isCanceled: true`,
  child not alive,
  and partial stdout `before` plus stderr `problem` preserved on the error.
- Fast child with a fresh `AbortSignal.timeout(60_000)`:
  successful resolution with stdout `ok` after 18 ms;
  the unused deadline timer did not keep the parent alive.

### Failing behavior catalog

- Child handles `SIGTERM`,
  `timeout: 100`:
  successful resolution after 517 ms.
- Child handles `SIGTERM`,
  `signal: AbortSignal.timeout(100)` with default kill signal:
  rejection after 120 ms including the liveness wait,
  `isCanceled: true`,
  child still alive.

The positive control proves that the harness detects an ordinary timeout termination.
The liveness check distinguishes promise settlement from child termination.

## Verified workarounds

### Use an abort deadline with a non-interceptable kill signal

```js
import spawn from 'nano-spawn';

const result = await spawn('gh', ['api', '--include', '/rate_limit'], {
  stdin: 'ignore',
  stdout: 'pipe',
  stderr: 'pipe',
  signal: AbortSignal.timeout(60_000),
  killSignal: 'SIGKILL',
  windowsHide: true,
});
```

Create a fresh deadline signal for each invocation and do not also pass nano-spawn's `timeout` option.
The reproduction's child handled `SIGTERM`,
but this form rejected at the deadline and left no live child.
`SubprocessError.isCanceled` was `true`,
which lets a caller distinguish its dedicated deadline signal from an ordinary nonzero exit.
Partial captured output remained available for diagnostic classification.
A successful child settled promptly despite the unused long deadline.

Tradeoff:
forceful termination gives the child no cleanup interval.
The parent must own cleanup of request files and other resources.
The verification above covers Linux and direct-child termination.
It does not prove process-tree termination when a child creates descendants that inherit captured pipes.
A consumer claiming Windows support must run the same liveness fixture there.

### Add explicit liveness enforcement through nodeChildProcess

A caller can await `subprocess.nodeChildProcess`,
track completion,
and force-kill the child when its own deadline fires.

Tradeoff:
this duplicates process lifecycle state that `AbortSignal.timeout()` plus `killSignal` already expresses,
and creates timer-versus-close races the caller must test.
It is useful only when staged graceful and forceful termination is required.

## What does not work

### Passing timeout alone

The timeout sends the default `SIGTERM`.
A child can handle it and later exit successfully,
causing nano-spawn to resolve beyond the requested interval.

### Using AbortSignal.timeout with the default kill signal

Nano-spawn rejects with `isCanceled: true`,
but the verified child remained alive.
Promise settlement alone does not prove resource termination.

### Racing the subprocess promise without terminating the child

A `Promise.race()` can return control at the deadline while leaving the child,
its pipes,
and its GitHub request active.
That violates a child-process deadline rather than implementing one.

## Upstream filing decision

No `.out-of-scope/` entry covers nano-spawn or child-process deadlines.
Open and closed nano-spawn Issues and pull requests were searched for
`timeout hard deadline SIGTERM`;
no match was found.

1.  **Is it really upstream's fault?**
    No.
    Nano-spawn promises to pass Node spawn options through.
    Node defines timeout as sending `killSignal`,
    not as proving termination by that instant.
2.  **Can upstream fix it?**
    Yes in principle,
    by adding a separate hard-deadline or staged-kill option.
3.  **Are they supporting this use case?**
    No.
    The README supports Node's timeout option but does not claim a hard deadline.
4.  **Would the repo welcome our contribution?**
    No prohibition on external contributions or assisted reports was found.
    The repository has no contribution guide or Issue template;
    its security policy routes vulnerabilities through Tidelift.
5.  **Will they likely fix it?**
    There is no matching tracker discussion or stated plan.
    The repository is active and released 2.1.0 on 2026-04-01,
    but this consumer requirement is outside the documented contract.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    No upstream change is warranted because constraints 1 and 3 fail.
    The verified consumer-side option composition supplies the required hard behavior.

Nothing should be filed upstream.
The behavior is inherited and documented,
and the consumer workaround uses options nano-spawn intentionally forwards.

[nano-index]: https://github.com/sindresorhus/nano-spawn/blob/cc231e2/source/index.js#L13-L15
[nano-readme]: https://github.com/sindresorhus/nano-spawn/blob/cc231e2/readme.md#L86-L88
[nano-result]: https://github.com/sindresorhus/nano-spawn/blob/cc231e2/source/result.js#L10-L21
[nano-result-cancel]: https://github.com/sindresorhus/nano-spawn/blob/cc231e2/source/result.js#L44-L49
[nano-spawn-source]: https://github.com/sindresorhus/nano-spawn/blob/cc231e2/source/spawn.js#L20
[node-spawn]: https://nodejs.org/docs/latest-v22.x/api/child_process.html#child_processspawncommand-args-options
