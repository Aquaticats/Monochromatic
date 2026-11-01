# Node.js 26.7.0 `spawnSync()` default `maxBuffer` turns large test output into truncated `ENOBUFS` evidence

## Symptom

A Node.js wrapper used `child_process.spawnSync()` to run a build and full test suite,
capture both output streams,
and decide whether a guard mutation had failed.
The wrapper did not set `maxBuffer` and checked `result.status` without first checking `result.error`.

Once one child invocation's combined piped output crossed the default 1,048,576-byte limit,
`spawnSync()` terminated or interrupted that child and returned an `ENOBUFS` error object.
The original wrapper did not persist `result.error`,
so the historical logs alone could not name that error.
They measured 1,073,564 and 1,073,294 bytes,
ended before the intended Candidate G assertion,
and contained no test-runner `FAIL` line.

The same project test command was reproduced while recording the complete result object.
On Linux x64 it returned:

```text
error.code = ENOBUFS
status = 1
signal = null
stdout bytes = 951944
stderr bytes = 98080
```

A standalone child that wrote 1,048,577 stdout bytes returned a different termination shape on the same host:

```text
error.code = ENOBUFS
status = null
signal = SIGTERM
```

The exact `status` and `signal` therefore depend on the child command and platform.
`result.error.code` is the stable evidence for this failure.

Setting `maxBuffer: 100 * 1024 * 1024` made subsequent logs complete.
Those first complete reruns revealed that two mutations actually survived,
so the lifecycle controls were strengthened in commits `4d25cc73d` and `f2c64fc7e`.
After those control changes,
the complete 2,028,382-byte and 2,027,739-byte runs contained the intended Candidate G failures.

The emitting API is Node.js `child_process.spawnSync()`.
`ENOBUFS` is exposed through `result.error.code`,
not through a thrown exception from `spawnSync()` itself.

## Root cause

All source citations are from `nodejs/node` [tag `v26.7.0`][node-source-v26],
commit `b4f23d3619c98bed09af93a21192f6080197a8c6`,
dated 2026-08-05.
The clone used for this trace had origin `https://github.com/nodejs/node.git`.

### `spawnSync()` installs a 1 MiB default

`lib/child_process.js:96` defines the shared limit:

```js
const MAX_BUFFER = 1024 * 1024;
```

`lib/child_process.js:883-913` applies that value unless the caller supplies another one,
validates it,
normalizes standard input and output,
and calls the internal binding:

```js
function spawnSync(file, args, options) {
  options = {
    __proto__: null,
    maxBuffer: MAX_BUFFER,
    ...normalizeSpawnArguments(file, args, options),
  };

  validateMaxBuffer(options.maxBuffer);
  // ...
  return child_process.spawnSync(options);
}
```

The [Node 26 API documentation][node-docs] states the same default and behavior at
`doc/api/child_process.md:1171-1175`:

```md
* `maxBuffer` {number} Largest amount of data in bytes allowed on stdout or
  stderr. If exceeded, the child process is terminated. See caveat at
  [`maxBuffer` and Unicode][]. **Default:** `1024 * 1024`.
```

### Every piped read contributes to one native counter

`lib/internal/child_process.js:1129-1131` crosses into the native synchronous runner:

```js
function spawnSync(options) {
  const result = spawn_sync.spawn(options);
```

Each synchronous standard-I/O pipe calls the same process-level counter after a read.
`src/spawn_sync.cc:285-294`:

```cpp
} else {
  last_output_buffer_->OnRead(buf, nread);
  process_handler_->IncrementBufferSizeAndCheckOverflow(nread);
}
```

`src/spawn_sync.cc:648-654` adds every read length to `buffered_output_size_`.
When the aggregate exceeds `max_buffer_`,
Node records libuv `UV_ENOBUFS` and kills the child:

```cpp
void SyncProcessRunner::IncrementBufferSizeAndCheckOverflow(ssize_t length) {
  buffered_output_size_ += length;

  if (max_buffer_ > 0 && buffered_output_size_ > max_buffer_) {
    SetError(UV_ENOBUFS);
    Kill();
  }
}
```

This counter is shared across piped output descriptors.
A 614,400-byte stdout plus a 614,400-byte stderr therefore exceeds the default,
even though neither stream individually reaches 1 MiB.
Closed Node pull request
[nodejs/node#4035][node-pr-4035]
explicitly discussed sharing `maxBuffer` between stdout and stderr.

### The result carries an error instead of a test exit status

`lib/internal/child_process.js:1140-1148` attaches stdout and stderr,
then converts the native error into an `ErrnoException`:

```js
result.stdout = result.output?.[1];
result.stderr = result.output?.[2];

if (result.error) {
  result.error = new ErrnoException(result.error, 'spawnSync ' + options.file);
  result.error.path = options.file;
  result.error.spawnargs = ArrayPrototypeSlice(options.args, 1);
}
```

The child did not reach an ordinary exit,
so `status` is `null`.
Checking only whether `status` equals zero conflates an `ENOBUFS` transport failure with a real test failure.

### Captured output can exceed the configured limit

Node checks after receiving a native read chunk.
The upstream test explains the observable overshoot at
`test/parallel/test-child-process-spawnsync-maxbuf.js:25-27`:

```js
// We can have buffers larger than maxBuffer because underneath we alloc 64k
// that matches our read sizes.
```

The same upstream test explicitly checks the emitted error at
`test/parallel/test-child-process-spawnsync-maxbuf.js:20-24`:

```js
const ret = spawnSync(process.execPath, args, { maxBuffer: 1 });

assert.ok(ret.error, 'maxBuffer should error');
assert.strictEqual(ret.error.code, 'ENOBUFS');
assert.strictEqual(getSystemErrorName(ret.error.errno), 'ENOBUFS');
```

A truncated log can therefore be somewhat larger than 1 MiB.
Its size is not proof that the entire child output was captured.

### Historical intent

The finite default is intentional memory protection,
not an accidental v26 regression.
Commit [`eb8a51a35c96`][node-finite-default-commit] introduced finite defaults for synchronous APIs to avoid
out-of-memory conditions.
Commit [`652877e3a9ee`][node-one-mib-commit],
associated with closed pull request
[nodejs/node#27179][node-pr-27179],
raised all defaults to 1 MiB in 2019.

An earlier hypothesis was that the mutation test itself failed without printing its assertion.
That reading was wrong.
The positive control with an explicit 100 MiB cap produced the complete log and the expected assertion,
while the default-cap run returned `ENOBUFS` and stopped near the native buffer boundary.

## Verification

Verified on 2026-08-31 with mise-managed Node.js `v26.7.0`
on Bazzite `44.20260825.0`,
Linux `7.2.0-ogc6.1.fc44.x86_64`,
architecture `x64`.
The source trace used tag `v26.7.0` at commit
`b4f23d3619c98bed09af93a21192f6080197a8c6`.

Use this standalone harness:

```js
// /tmp/spawnsync-maxbuffer-repro.mjs
import assert from 'node:assert/strict';
import { closeSync, openSync, rmSync, statSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const KiB = 1024;
const MiB = KiB * KiB;
const child = String.raw`
  const [stdoutBytes, stderrBytes] = process.argv.slice(1).map(Number);
  process.stdout.write(Buffer.alloc(stdoutBytes, 120));
  process.stderr.write(Buffer.alloc(stderrBytes, 121));
`;

function run(label, stdoutBytes, stderrBytes, options = {}) {
  const result = spawnSync(
    process.execPath,
    ['-e', child, String(stdoutBytes), String(stderrBytes)],
    { encoding: 'buffer', ...options },
  );
  return {
    label,
    errorCode: result.error?.code ?? null,
    status: result.status,
    signal: result.signal,
    stdoutBytes: result.stdout?.length ?? null,
    stderrBytes: result.stderr?.length ?? null,
  };
}

const redirectedPath = join(tmpdir(), `spawnsync-maxbuffer-${process.pid}.log`);
const redirectedFd = openSync(redirectedPath, 'w');
const rows = [
  run('default-under', MiB - 1, 0),
  run('default-exact', MiB, 0),
  run('default-over', MiB + 1, 0),
  run('default-combined-over', 600 * KiB, 600 * KiB),
  run('explicit-4MiB', 2 * MiB, 0, { maxBuffer: 4 * MiB }),
  run('infinity', 2 * MiB, 0, { maxBuffer: Infinity }),
  run('redirected-fd', 2 * MiB, 0, {
    stdio: ['ignore', redirectedFd, 'ignore'],
  }),
];
closeSync(redirectedFd);
rows.at(-1).redirectedBytes = statSync(redirectedPath).size;
assert.equal(rows[0].errorCode, null);
assert.equal(rows[1].errorCode, null);
assert.equal(rows[2].errorCode, 'ENOBUFS');
assert.equal(rows[3].errorCode, 'ENOBUFS');
assert.equal(rows[4].errorCode, null);
assert.equal(rows[5].errorCode, null);
assert.equal(rows[6].redirectedBytes, 2 * MiB);
console.log(JSON.stringify({ node: process.version, rows }, null, 2));
rmSync(redirectedPath);
```

Run it with:

```bash
node /tmp/spawnsync-maxbuffer-repro.mjs
```

### Clean catalog

- `default-under`,
  1,048,575 stdout bytes:
  `errorCode: null`,
  `status: 0`.
- `default-exact`,
  1,048,576 stdout bytes:
  `errorCode: null`,
  `status: 0`.
- `explicit-4MiB`,
  2,097,152 stdout bytes:
  `errorCode: null`,
  `status: 0`.
- `infinity`,
  2,097,152 stdout bytes:
  `errorCode: null`,
  `status: 0`.
- `redirected-fd`,
  2,097,152 stdout bytes:
  `errorCode: null`,
  `status: 0`,
  captured stdout absent,
  and the destination file measured 2,097,152 bytes.

### `ENOBUFS` catalog

- `default-over`,
  1,048,577 requested stdout bytes:
  `errorCode: ENOBUFS`,
  `status: null`,
  `signal: SIGTERM`.
- `default-combined-over`,
  614,400 requested bytes on each output stream:
  `errorCode: ENOBUFS`,
  `status: null`,
  `signal: SIGTERM`.
  The returned chunks totaled more than 1 MiB because the overflow check runs after a native read.

### Project incident control

The original `spawnSync()` wrapper used its default buffer.
The two truncated logs contained no intended Candidate G failure and did not persist `result.error`.
A direct reproduction of the same `mise run //package/translation-repair/<package>:test:unit` child command
recorded `error.code: ENOBUFS`,
`status: 1`,
`signal: null`,
and 1,050,024 captured bytes across stdout and stderr.
Each sequential build or test `spawnSync()` call has its own 1 MiB budget;
the final wrapper log can be larger because it concatenates outputs from separate child invocations.

The corrected wrapper set:

```js
const result = spawnSync('mise', ['run', task], {
  encoding: 'utf8',
  maxBuffer: 100 * 1024 * 1024,
});
```

Complete reruns first showed that the lease-history and all-settled mutations survived.
After commits `4d25cc73d` and `f2c64fc7e` strengthened those controls,
the corrected runs exited nonzero for the intended mutations and included these exact controls:

```text
realization reclaim election history retention control failed
realization author concurrent exact abort control failed
```

Restored runs reported 874 passing suites and no failures.

## Verified workarounds

### Set a finite cap from the expected evidence envelope

For synchronous callers that need the complete output in memory,
set an explicit finite cap and inspect `result.error` before interpreting `result.status`:

```js
const result = spawnSync('mise', ['run', task], {
  encoding: 'utf8',
  maxBuffer: 100 * 1024 * 1024,
});

if (result.error !== undefined)
  throw result.error;

const mutationWasRejected = result.status !== 0;
```

The 100 MiB cap was verified against the project test output,
which measured about 2 MiB for the affected runs.
The tradeoff is bounded but potentially substantial memory use because synchronous output remains buffered in the parent.
The cap must include stdout and stderr together plus headroom for run-to-run output variability.
Native read-chunk overshoot affects how much truncated output may be returned after failure;
it does not require headroom when actual aggregate output remains within the cap.

### Redirect large output to files

Pass file descriptors in `stdio` when the parent does not need an in-memory return value:

```js
import { open } from 'node:fs/promises';

await using output = await open(outputPath, 'w');
const result = spawnSync('mise', ['run', task], {
  stdio: ['ignore', output.fd, output.fd],
});

if (result.error !== undefined)
  throw result.error;
```

The 2 MiB verification arm completed without `ENOBUFS` and wrote every byte.
The tradeoffs are explicit file lifecycle management,
loss of `result.stdout` and `result.stderr`,
and merged streams when one descriptor is reused.
Use separate descriptors when stream identity matters.

### Stream with asynchronous `spawn()`

`child_process.spawn()` exposes streams and does not accumulate output behind a `maxBuffer` option.
Maintainer discussion in [nodejs/node#39933][node-issue-39933] confirms that distinction.
Pipe each stream to bounded processing or files and await child close.

The tradeoff is an asynchronous lifecycle rewrite:
callers must own stream backpressure,
error events,
close ordering,
cancellation,
and output-file cleanup.
For a synchronous mutation harness with measured 2 MiB output,
an explicit finite `maxBuffer` is the narrower workaround.

## What does not work

- Checking only `result.status`:
  `status: null` also represents `spawnSync()` failure.
  Read `result.error` first and report `error.code`.
- Treating absence of a test assertion in captured output as a null result:
  the log may have ended at `maxBuffer`,
  not at test completion.
- Changing `encoding`:
  the native limit counts bytes before JavaScript converts returned buffers to strings.
- Sizing the cap from stdout alone or from one prior run:
  stdout and stderr share the counter,
  and test diagnostics vary between runs.
  The harness confirms that exactly 1 MiB succeeds when actual aggregate output is exactly 1 MiB.
- `stdio: 'ignore'`:
  it avoids buffering by discarding the evidence the mutation run exists to inspect.
- `maxBuffer: Infinity`:
  it worked in the harness,
  but removes the runaway-output circuit breaker and permits unbounded parent memory growth.
- Raising shell pipe or terminal scrollback limits:
  the terminating counter is inside Node's synchronous child-process runner.

## Upstream filing decision

### Duplicate search

Open and closed Node issues and pull requests were searched for
`spawnSync`,
`maxBuffer`,
and `ENOBUFS`.
[nodejs/node#9829][node-issue-9829],
“maxBuffer default too small”,
already contains the default-size debate and maintainer rationale.
[nodejs/node#23027][node-pr-23027] and
[nodejs/node#27179][node-pr-27179] record the finite-default and 1 MiB decisions.
There is no additive defect report from this incident.
Post nothing upstream.

### Six-constraint check

- Constraint 1,
  really upstream's fault:
  **no**.
  Node documents the 1 MiB default,
  tests `ENOBUFS`,
  and intentionally uses a finite cap as memory protection.
  The consumer wrapper omitted a required envelope choice and ignored `result.error`.
- Constraint 2,
  can upstream fix it:
  **yes in principle**,
  by changing the default or diagnostic surface,
  but no upstream correction is required for documented behavior.
- Constraint 3,
  is the use case supported:
  **yes**.
  `spawnSync()` documents `maxBuffer`,
  supports finite values and `Infinity`,
  and has dedicated tests in `test/parallel/test-child-process-spawnsync-maxbuf.js`.
- Constraint 4,
  would the repository welcome this contribution:
  **no for an agent-authored interaction without prior authorization**.
  `CONTRIBUTING.md:57-66` requires external automation to obtain authorization in `nodejs/admin`
  before creating issues,
  comments,
  pull requests,
  or reviews.
  No matching `.out-of-scope/` exemption exists,
  but the upstream automation policy independently forbids filing this artifact.
- Constraint 5,
  will upstream likely fix it:
  **no for this requested behavior**.
  The finite default and later 1 MiB value were deliberate,
  reviewed changes,
  and the exact default-size issue is closed.
- Constraint 6,
  was a minimal upstream fix prototyped:
  **no,
  and not required**.
  Constraints 1,
  4,
  and 5 fail,
  so the auto-prototype gate does not trigger.
  The verified consumer fix is to set a measured finite cap and handle `result.error`.

### Upstream filing artifact

Nothing to add to [nodejs/node#9829][node-issue-9829].
Do not file a new issue or comment.

[node-docs]: https://nodejs.org/docs/latest-v26.x/api/child_process.html#child_processspawnsynccommand-args-options
[node-source-v26]: https://github.com/nodejs/node/tree/v26.7.0
[node-issue-9829]: https://github.com/nodejs/node/issues/9829
[node-issue-39933]: https://github.com/nodejs/node/issues/39933
[node-pr-4035]: https://github.com/nodejs/node/pull/4035
[node-pr-23027]: https://github.com/nodejs/node/pull/23027
[node-pr-27179]: https://github.com/nodejs/node/pull/27179
[node-finite-default-commit]: https://github.com/nodejs/node/commit/eb8a51a35c96
[node-one-mib-commit]: https://github.com/nodejs/node/commit/652877e3a9eee3f863314382f64f8ac1e5b27186
