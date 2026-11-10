# Node 26.13.0 `execFile` ends a child that writes 1 MiB to stdout or stderr

A program started with `child_process.execFile` is alive only while its combined
output stays under `maxBuffer`,
which defaults to 1 MiB.
A launch helper that
started a long-lived GUI with `execFile` therefore had the window it opened end
mid-session once that window had written 1 MiB,
and its promise pended until the
window closed.
`package/kwin/key-helper` hit this twice:
as `execFile` in
`src/launch.ts`,
then after that moved into `src/child-process.ts`,
where both
`ydotool` and the launcher shared the one buffered shape (issue 4621).

## Symptom

No error reaches the caller and nothing is logged at the moment of the kill.
A launched application disappears,
or a future write fails,
and the daemon's
await never settles for as long as the application would have run.

The failure surfaces differently depending on when the parent notices the bound.
Two probes against the same 4 MiB child,
differing only in how fast the child
writes:

- A child that writes in 64 KiB chunks with a 20 ms pause is ended with
  `SIGTERM` after 347 ms;
  the callback receives
  `error.code: ERR_CHILD_PROCESS_STDIO_MAXBUFFER` and `stderr` holding exactly
  `1048576` characters;
  a marker file the child would have written after its
  last write is absent.
- A child that writes the same 4 MiB as fast as the pipe accepts it finishes
  first:
  the marker records `reached the end after 4194304 chars`,
  the callback
  receives `error.code: 7` (the child's own exit status),
  and `stderr` holds
  only `196608` characters,
  silently truncated.

## Root cause

Node documents the bound in the `execFile` options:

> `maxBuffer` <number> Largest amount of data in bytes allowed on stdout or
> stderr.
> If exceeded,
> the child process is terminated and any output is
> truncated.
> See caveat at maxBuffer and Unicode.
> Default:
> `1024 * 1024`.

The same paragraph appears for `exec` and `execSync`;
`spawn` documents no
`maxBuffer` [node-child-process][].

The bound is enforced on the parent's side as output arrives,
and the parent
signals the child with `killSignal`,
`SIGTERM` by default,
then reports
`ERR_CHILD_PROCESS_STDIO_MAXBUFFER` instead of the child's own exit status.
Because enforcement is per read,
a child that writes and exits within one
parent turn can finish before the signal is sent,
which is the second probe
above:
the truncation is visible,
the kill is not.
A test or a reproduction that
writes one large block therefore does not show the kill.

Which shape fits:

- A command the daemon runs to completion and reads the failure of is well served
  by a bound:
  the buffer is the failure message.
- A program the daemon starts and leaves running outlives any bound.
  It needs a
  stream the parent drains and discards past a fixed point,
  and a call that
  returns once the program has started rather than when it ends.

## Verification

Version under test:
Node `v26.13.0`,
the toolchain binary at
`${HOME}/.local/share/monochromatic-dev/toolchain/node/26.13.0`.
Scratch scripts live outside this repository,
in
`${HOME}/temp/agent/pi-key-helper`.

Both probes use this same Node binary to run the child:

```js
// child that writes 40 chunks of 64 KiB to stderr with 20 ms between them
const chunk = 'y'.repeat(64 * 1024);
const pause = (ms) => Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms);
for (let index = 0; index < 40; index += 1) {
  process.stderr.write(chunk);
  pause(20);
}
require('node:fs').writeFileSync(marker, 'reached the end');
```

- `execFile(process.execPath, ['-e', childScript], { maxBuffer: 1024 * 1024 }, cb)`
  ends the child:
  `exit event after ms: 347 code: null signal: SIGTERM`,
  `error.code: ERR_CHILD_PROCESS_STDIO_MAXBUFFER`,
  marker absent.
- The same child without the pauses finishes:
  `error.code: 7`,
  `stderr length kept: 196608`,
  marker present.
  This is the catalog entry that
  looks like a counterexample and is not one.
- `spawn(command, args, { stdio: ['ignore', 'ignore', 'pipe'] })` with an
  `on('data')` reader that stops appending past 1 MiB keeps the same child
  running to its own exit;
  the package's
  `package/kwin/key-helper/src/child-process.ts` tests assert exactly that
  (`runCommand` reports `exited with code 7`,
  and a started launcher records its
  last line).

The text fragment on the Node documentation link was loaded in a headless
browser session and scrolled to the `execFile` options paragraph;
a fragment
naming text the page does not hold left the page at the top,
so the check tells
the two apart.

## Verified workarounds

- Start the program with `spawn` and pipe only stderr,
  keeping the first 1 MiB
  for a failure message and discarding the rest as it arrives,
  then resolve on
  the `spawn` event.
  `package/kwin/key-helper/src/child-process.ts` does this in
  `startCommand`.
  - Tradeoff:
    a started program's later nonzero exit is reported as a log line,
    not as a rejection,
    because the caller has already returned.
    The daemon's
    callers are D-Bus methods that answer immediately,
    so nothing else could
    carry it.
  - Tradeoff:
    stderr past 1 MiB is gone,
    and the message says so.
- Raise `maxBuffer` for a command that is meant to finish.
  `execFile` still ends
  the child at the new bound.
  - Tradeoff:
    the value is a guess at how much output a program may produce,
    and
    the kill comes back as soon as the guess is low.
- `maxBuffer: Infinity` removes the kill for a program that ends.
  - Tradeoff:
    memory grows with everything the child writes,
    so one talkative
    daemon's child can exhaust the host.

## What does not work

- Awaiting `close` on an `execFile`-started program that outlives the call.
  The
  promise pends for the whole lifetime,
  and the bound still ends it.
- Reproducing the kill with one large `process.stderr.write`.
  The child can exit
  before the parent enforces the bound;
  the marker in the second probe above is
  the evidence.
  A reproduction or a test has to write across several parent
  turns.
- Listening for a `close` code of `null` as the signal of a kill in a buffered
  shape:
  the rejection carries `ERR_CHILD_PROCESS_STDIO_MAXBUFFER` and `signal`
  is `undefined`,
  while `killSignal` names what was sent.

## Upstream filing decision

Nothing to file,
and the six constraints are walked here so a future session does
not repeat the audit.

1. Is it really upstream's fault?
   No. `maxBuffer` and `killSignal` are documented
   options with a documented effect,
   and the docs already carry a
   "maxBuffer and Unicode" section about byte counting.
   Our call site used a
   one-shot runner for a program that outlives a call.
2. Can upstream fix it?
   Not applicable:
   there is nothing broken to fix.
   `spawn`
   already provides the unbuffered shape.
3. Are they supporting this use case?
   Yes,
   `spawn` with `stdio` and `unref` is
   the documented way to start a detached program.
4. Would the repo welcome our contribution?
   Not reached,
   since there is nothing
   to contribute.
   No issue,
   pull request,
   or comment is drafted in this
   repository for this finding.
5. Will they likely fix it?
   Not applicable.
6. Have we prototyped a minimal fix?
   The fix belongs on our side and is in
   `package/kwin/key-helper/src/child-process.ts`.

[node-child-process]:
  https://nodejs.org/docs/latest-v26.x/api/child_process.html#:~:text=Largest%20amount%20of%20data%20in%20bytes%20allowed%20on%20stdout%20or%20stderr
