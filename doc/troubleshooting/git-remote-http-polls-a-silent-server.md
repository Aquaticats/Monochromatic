# A silent server outlives the connected push window, because the HTTP helper keeps ticking

Status:
fixed on 2026-11-11 in `package/git-policy/cli` (`push_progress.rs`,
`bounded_push.rs`,
`stop_cause_and_words.rs`),
with controls in `push_progress_tests.rs` and `push_slow_server_tests.rs`.
Every face this audit measured,
the Git LFS upload that keeps reconnecting included,
is bounded;
the measurements are at the end of this file.

## Symptom

The commit service's own push (the auto-push after a landed commit) to a server that accepts the
TCP connection and never answers did not end.
Measured by the audit `alpha-10-merge-lfs-and-package-repair` (finding F8) against the wrapper
0.6.1-alpha.10,
with `CONNECTED_PUSH_QUIET_WINDOW` at 150 seconds:

- a plain `git push` over HTTP to a plain TCP listener:
  still running after about 290 seconds,
  one
  connection,
  no stop and no retry line in the service log;
- a Git LFS upload with `lfs.activitytimeout=100000`:
  still running after 330 seconds,
  one
  connection;
- a Git LFS upload with the default activity timeout:
  still running after 330 seconds,
  twelve
  connections,
  because git-lfs reconnected every 30 seconds.

Direct `git-lfs` alone retried for 900 seconds and was killed by `timeout`.

## What the clock saw

The clock reads three signs of work from `/proc` and a fourth from the kernel's socket
diagnostics (`push_progress.rs`):
bytes read and written,
processor time,
a task able to run or
waiting for the disk,
and bytes carried by a connected socket.

The audit could not tell why the window was never reached.
It was measured on 2026-11-11 with the
crate's own verbosity (`MONOCHROMATIC_VERBOSE=true`) and a shortened window,
against a listener that
accepts and reads and answers nothing:

- the tree was `git push`,
  `git remote-http` and `git-remote-http`;
- `git-remote-http`,
  the process that holds the connection,
  was charged **one processor tick every
  three seconds** while it polled the silent socket,
  and its byte counters did not move at all;
- every look that found such a tick counted the tree as working,
  so the silence the connected
  window measures never reached 150 seconds,
  and the wait line ("nothing has moved for 10
  seconds") was followed by nothing.

A tree of a single holder that only ticks (a shell holding a socket and burning a few milliseconds
between two timed reads of it) never ended at all.

The same verbosity over a Git LFS upload that stayed on one connection (the audit's
`ACTIVITY_TIMEOUT=100000` face) showed a second mover:

```text
sample connected=true progressed=true moved=76408 socket_moved=0 members={(1865491, ...): (76408, 1)}
sample connected=true progressed=true moved=76422 socket_moved=0 members={(1865491, ...): (76422, 1)}
sample connected=true progressed=true moved=76436 socket_moved=0 members={(1865491, ...): (76436, 1)}
```

Fourteen bytes about every ten seconds,
all of them `git-lfs`'s own upload meter on its standard error,
which the wrapper gives the child as a file of its own.
`Meter::look` read every byte the tree wrote as work,
so `waiting_since` was cleared every ten seconds and the window never fired in 260 seconds of
observation,
on one connection,
with no stop line in the service log.

## The cause

`Meter::observe` reported "work" for any of the three process signs,
and `Meter::look` let that
answer the connected-window question.
A helper that polls its socket is therefore
indistinguishable from a push that makes its pack,
and the window that exists for exactly this case
can never fire.
The holder's own bytes are the same defect one step on:
its meter is not progress of the transfer.

## The fix

While a connection is open,
neither the processor time nor the bytes of the process that holds it
are work:
that process polls the socket and prints its own account of waiting.
What it really sends is on the connection,
whose carried bytes are counted,
and so is
the work of every process that holds no connection,
which is where a
push keeps working while its connection stands open (`pack-objects` searching deltas,
for one).
A socket carries no progress on the look that first sees it,
because its counters then stand for
this side's handshake and request,
and that is also what bounds a client that reconnects:
each new connection starts from nothing and its client is its holder,
so a chain of reconnections that carries nothing still reaches the window.

A stop names the run that held the connection (`git push`,
`git-lfs push`,
`git fetch`) and the server,
as host and port:
the push's destination word is read
from the command,
a URL answers with its own host,
and a remote name with one bounded
`git remote get-url --push` query,
asked only when a stop needs it.
A push that lands therefore pays nothing for the name,
and the wait line a live run prints still says "the remote".

A control that cannot wait 150 seconds shortens the window with
`CLI_GIT_TEST_ONLY_CONNECTED_WINDOW_SECONDS`,
in the manner of the crate's other control-only
variables.

## How to see it again

`package/git-policy/cli/src/native/push/push_slow_server_tests.rs`,
`a_server_that_takes_the_connection_and_answers_nothing_is_ended_at_the_connected_window`:
a local
listener takes the connection and holds it for three windows while the push runs,
so an end before
that is the wrapper's,
and never the fixture's.

Against the service:

```sh
# a local stand-in that accepts and answers nothing, in a throwaway repository with its own HOME
node "${HOME}/temp/agent/f7-f8/repro/git_autopush_silent.ts" 200
```

Measured on this host on 2026-11-11 with the fix:

- `push_ms=151466`,
  one connection accepted;
- the stop the service logged,
  as one line of its push report:

```text
monochromatic-git-policy-cli: git push held its connection to the server 127.0.0.1:40953 open with
nothing moving for 150 seconds and was stopped (limit=connected, bytes_moved=62938,
elapsed_seconds=151); CLI_GIT_PUSH_DEADLINE_SECONDS does not change this window, which is fixed at
150 seconds.
```

- the retry line,
  `next_try=2 in_seconds=10`.

## A client that keeps reconnecting

`git-lfs` with its default thirty second activity timeout reconnects to a silent server every thirty
seconds,
and its reconnect writes a request,
which used to be a byte the tree moved.
With the holder's own bytes no longer counted,
the chain of connections is bounded like one
connection:
each one starts from nothing and each client is its own holder.

Measured on 2026-11-11 against a stand-in that accepts and answers nothing,
with the default
`lfs.activitytimeout`:

- six connections,
  thirty seconds apart,
  accepted `4, 34, 64, 94, 124, 154` seconds after the
  commit;
- one stop,
  as the service logged it:

```text
monochromatic-git-policy-cli: git-lfs push held its connection to the remote open with nothing
moving for 150 seconds and was stopped (limit=connected, bytes_moved=78053, elapsed_seconds=150)
```

- the push's own length in the service's report,
  `push_ms=150995`,
  and `outcome=retrying` after it.

What the window still cannot see is a remote that keeps acknowledging bytes for ever,
which by the recorded decision ("Bound it,
keep retrying",
and no ceiling for a run that shows progress) is work and is waited for.
