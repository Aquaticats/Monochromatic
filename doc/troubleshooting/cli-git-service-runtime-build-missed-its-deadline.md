# cli-git: a service whose runtime build missed its deadline

## Symptom

The wrapper's own gate of 2026-11-11T10:29-05:00
(`gpc-gate-alpha-11-on-fc9f7f5f4b-20261111T102937-0500.log`)
failed three tests that passed when each ran alone.
One of them,
`fault_trace::a_service_killed_at_each_ftruncate_leaves_a_recoverable_repository`,
failed because one of its services ended before it listened:

```text
monochromatic-git-policy-cli-service: started release="0.6.1-alpha.9" protocol=11 pid=140204 ...
monochromatic-git-policy-cli-service: ready at_ms=1794412368765
monochromatic-git-policy-cli-service: cannot start the acceptor of connections: the runtime that runs it
  has no worker thread (the system may refuse threads; check `ulimit -u` and the control group's `pids.max`)
  at_ms=1794412370526

thread '<unnamed>' panicked at src/native/binary/binary_support_fixture_service.rs:345:17:
the fixture's commit service ended (exit status: 4) before it listened at
  /home/tester/temp/agent/rt-139054-3/monochromatic-git-policy-cli/b559f4159c3a78be995125257398e39d.sock
```

## Emitter

`package/git-policy/cli/src/native/service/service_server_accept_connections_as_tasks.rs`,
which started the acceptor with `spawn_on_runtime`,
and answered nothing when the process's runtime had no worker:
the service then wrote the line and exited with status 4.
The same pattern sat in the after-commit work
(`service_server.rs`) and the shutdown (`service_server_shutdown_after_a_stop.rs`).

The runtime itself is built by
`package/git-policy/cli/src/native/foundation/lazy_runtime.rs`:
on a helper thread,
under `BUILD_DEADLINE` (1 s),
and a build that misses that deadline is kept for the work it was built for
(`LATE_RUNTIME`).

## Working diagnosis

The two timestamps are the measurement.
`ready` was written before the acceptor started,
and the refusal came 1.76 s later.
A plan that was the current-thread one from the start,
or a build the system refused,
answers at once:
the message would have come in the time the rest of the start took.
The one path that spends exactly a second is the missed deadline,
so the multi-thread build outlasted `BUILD_DEADLINE`
and had still not arrived in the remaining 0.7 s.

A healthy build takes milliseconds,
so what made this one slow is the container's own load:
the gate runs four test threads at once,
each fault-trace control runs `SWEEP_WIDTH` (six) cases at once,
and each case starts a service,
an `strace` and the Gits they run.
That is enough to make a build that hands out worker threads wait.

What this does not establish:
how long the late build would have taken to arrive,
and whether the same service would have served had it waited.
The fix assumes it would,
because `LATE_RUNTIME` is exactly the runtime such a build exists for.

## The fix (2026-11-11)

A service now waits `LATE_BUILD_PATIENCE` (five times `BUILD_DEADLINE`) for a build that
missed its deadline before it gives up,
in all three places that need a worker,
and says which way its build went either way
(`package/git-policy/cli/src/native/service/service_runtime_build_report.rs`):

```text
this service serves from a runtime that is not the multi-thread one: it was built as a current-thread
stand-in, because the multi-thread build outlasted its 1s deadline in 1000 ms, and the build of a
service's own runtime is held to 1s so that one the system refuses cannot hold a commit. ...
```

A service that still finds no worker writes the task,
the build,
the wait and the limits to check,
and exits 4 as before.

## Guard

`package/git-policy/cli/src/native/binary/binary_service_late_runtime_build_tests.rs`
starts a service whose build is made slower than the deadline on purpose
(`CLI_GIT_TEST_ONLY_RUNTIME_BUILD_MILLIS`,
inert unless set),
and commits through it.
With the acceptor back on `spawn_on_runtime` the control fails exactly as the gate did:

```text
cannot start the acceptor of connections: this process has no worker thread to run it on, so it ends
here and the manager starts it again. the runtime was built as a current-thread stand-in, because the
multi-thread build outlasted its 1s deadline in 1000 ms, and it had still given no worker 5000 ms later
```

## What this does not establish

The gate's own pressure was not reproduced in a bounded subset on this host:
37 to 41 fault-trace controls at three or eight test threads in a container of the gate's own bounds
(the gate's 2 GB,
four CPUs and 1024 processes) peaked at 268 and 432 tasks and at 495 MB and 632 MB,
and no service there missed its build deadline.
So the deadline's miss is argued from the two timestamps above,
not from a rerun of the gate's own contention.

Two neighbours of that failure in the same gate failed for their own causes
(the beat a starved connection writer does not write,
and a landing the machine stalled for 92 s outside the wait it accounted for);
each is fixed where it lives,
and neither is evidence about this one.
