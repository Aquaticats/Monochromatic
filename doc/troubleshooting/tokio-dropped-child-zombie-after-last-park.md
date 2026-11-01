# tokio 1.53.2 leaves a killed language server as a zombie when the runtime that dropped its handle stops parking before the process ends

helix-lsp at `ba40e547426b0f9896c8bdc699a4ab11f2b37dbc` starts every language server with tokio's `kill_on_drop`
and never awaits the process handle.
tokio collects the exit status of a dropped handle ("reaps" the process) only when one of its runtimes
returns from parking after the kernel reported that a child ended.
The Language worker of `package/ide/app` ended its runtime 50 ms after it dropped the handles,
so a server that needed longer to end stayed in the process table as a zombie until the application exited.

tokio documents this reaping as best effort, and the worker's ending was ours.
The worker now drops its runtime first and then reaps on a second, short-lived runtime.
Nothing is filed upstream.

## Symptom

`lifecycle::dropping_the_worker_leaves_no_child_process` failed once in a gate run and passed when run again.
It panicked at `tests/language/support.rs:439` (line 447 since the file grew) with:

```text
child processes remain after the worker was dropped: [(3512, 'Z', "ide-scripted-ls")]
```

Each tuple is the process number, the kernel's state letter, and the command name of one child of the test process,
read from `/proc/<pid>/stat` by `children` in `tests/language/support.rs`.
State `Z` is a zombie: the process has ended, and its parent has not collected its exit status.
The test had polled for five seconds (`children_until_none` in the same file) before it gave up.

In some measurements the same assertion also failed with a child that was still running:

```text
child processes remain after the worker was dropped: [(141, 'D', "ide-scripted-ls")]
```

State `D` is an uninterruptible wait inside the kernel.
A process in that state does not act on any signal, `SIGKILL` included, until the wait is over.

## Root cause

### helix-lsp hands the process to tokio's kill on drop

`helix-lsp/src/client.rs:230` spawns the server and keeps the handle in a field that nothing awaits:

```rust
// helix-lsp/src/client.rs:230
        let process = Command::new(cmd)
            .envs(server_environment)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(&root_path)
            // make sure the process is reaped on drop
            .kill_on_drop(true)
            .spawn();
```

```rust
// helix-lsp/src/client.rs:56
pub struct Client {
    id: LanguageServerId,
    name: String,
    _process: Child,
```

The process handle is dropped when the last `Arc<Client>` is dropped.

### tokio queues a dropped handle whose process has not ended yet

On Linux with process file descriptors, dropping the handle runs `PidfdReaper::drop`.
It asks once, without waiting, whether the process has ended, and otherwise moves the handle to a queue:

```rust
// tokio-1.53.2/src/process/unix/pidfd_reaper.rs:203
    fn drop(&mut self) {
        let mut orphan = self.inner.take().expect("inner has gone away").inner;
        if let Ok(Some(_)) = orphan.try_wait() {
            return;
        }

        self.orphan_queue.push_orphan(orphan);
    }
```

`Reaper::drop` at `src/process/unix/reap.rs:122` does the same on kernels without process file descriptors.
A process that was sent `SIGKILL` an instant before has normally not ended yet, so a killed server goes to the queue.

The queue is one per process, not one per runtime:

```rust
// tokio-1.53.2/src/process/unix/mod.rs:78
    fn get_orphan_queue() -> &'static OrphanQueueImpl<StdChild> {
        static ORPHAN_QUEUE: OrphanQueueImpl<StdChild> = OrphanQueueImpl::new();

        &ORPHAN_QUEUE
    }
```

### The queue is drained only when a runtime returns from parking

The process driver of a runtime looks at the queue after each park and nowhere else.
Its `shutdown` does not:

```rust
// tokio-1.53.2/src/runtime/process.rs:31
    pub(crate) fn park(&mut self, handle: &driver::Handle) {
        self.park.park(handle);
        GlobalOrphanQueue::reap_orphans(&self.signal_handle);
    }

    pub(crate) fn park_timeout(&mut self, handle: &driver::Handle, duration: Duration) {
        self.park.park_timeout(handle, duration);
        GlobalOrphanQueue::reap_orphans(&self.signal_handle);
    }

    pub(crate) fn shutdown(&mut self, handle: &driver::Handle) {
        self.park.shutdown(handle);
    }
```

`reap_orphans` drains the queue when a `SIGCHLD` arrived since the previous drain.
It starts listening for `SIGCHLD` lazily, the first time a park returns while the queue is not empty,
and drains once at that moment:

```rust
// tokio-1.53.2/src/process/unix/orphan.rs:85
        if let Some(mut sigchild_guard) = self.sigchild.try_lock() {
            match &mut *sigchild_guard {
                Some(sigchild) => {
                    if sigchild.try_has_changed().and_then(Result::ok).is_some() {
                        drain_orphan_queue(self.queue.lock());
                    }
                }
                None => {
                    let queue = self.queue.lock();

                    // Be lazy and only initialize the SIGCHLD listener if there
                    // are any orphaned processes in the queue.
                    if !queue.is_empty() {
                        // An errors shouldn't really happen here, but if it does it
                        // means that the signal driver isn't running, in
                        // which case there isn't anything we can
                        // register/initialize here, so we can try again later
                        if let Ok(sigchild) = signal_with_handle(SignalKind::child(), handle) {
                            *sigchild_guard = Some(sigchild);
                            drain_orphan_queue(queue);
                        }
                    }
                }
            }
        }
```

So a queued process is reaped only if some runtime in the process parks again after that process ended.

### The worker stopped parking 50 ms after it dropped the handles

On commit `ec7bdc39b`, the worker's shutdown ended like this,
and the worker thread, with its runtime, ended right after:

```rust
// package/ide/app/src/language/worker.rs:324, commit ec7bdc39b
        // Dropping the session and the registry releases every client; helix-lsp kills on drop.
        drop(self.session);
        drop(self.registry);
        tokio::time::sleep(REAP_PAUSE).await;
```

`REAP_PAUSE` was 50 ms (line 65 of the same file).
A server that ended more than 50 ms after its handle was dropped was never reaped.

### A server that has not answered `initialize` is released only when the runtime is dropped

`start_client` keeps a second `Arc<Client>` inside the task that awaits the `initialize` answer:

```rust
// helix-lsp/src/lib.rs:944
    // Initialize the client asynchronously
    let _client = client.clone();
    tokio::spawn(async move {
        use futures_util::TryFutureExt;
        let value = _client
            .capabilities
            .get_or_try_init(|| {
                _client
                    .initialize(enable_snippets)
                    .map_ok(|response| response.capabilities)
            })
            .await;
```

While that task is pending, dropping the registry does not drop the process handle.
The handle goes when the runtime drops its tasks, which is after the runtime's last park.
Such a server is reaped only if it had already ended at that moment, by the one `try_wait` in the drop.

### tokio documents the behavior

```rust
// tokio-1.53.2/src/process/mod.rs:218
//! The tokio runtime will, on a best-effort basis, attempt to reap and clean up
//! any process which it has spawned. No additional guarantees are made with regard to
//! how quickly or how often this procedure will take place.
//!
//! It is recommended to avoid dropping a [`Child`] process handle before it has been
//! fully `await`ed if stricter cleanup guarantees are required.
```

The documentation of `Command::kill_on_drop` (`src/process/mod.rs:645` to `663`) repeats it for killed processes.

### A reading that was wrong

The first attempt kept the worker's own runtime and waited inside it,
up to one second, until `/proc/thread-self/children` was empty.
Zombies remained: 14 and 32 of 600 runs failed in two measurements of that design.
A build that printed the state of every child still listed at that bound
(`ide-language-verify-zombie-probe-state-bVeTNk/runs/`, see the section "Evidence")
found 26 such children in 600 runs: 21 in state `D`, 5 in state `R`, none in state `Z`.
The children were still running.
They were the servers held by the `initialize` task,
which are killed only when the runtime is dropped, so no wait before that drop can see them end.

## Verification

Versions under test:
tokio 1.53.2 (crates.io checksum `e95f91fcc7a621e8b030f6aa23c71fe9838ae2fb4d8118b75602a328f5144044`),
helix-lsp at `ba40e547426b0f9896c8bdc699a4ab11f2b37dbc`,
Linux 7.2.7 with `CONFIG_PROC_CHILDREN`,
the `localhost/monochromatic/ide` test container.

### Harness

One run of the failing test:

```sh
# package/ide/app
mise run //package/ide/app:test:language lifecycle::dropping_the_worker_leaves_no_child_process
```

The failure is rare in a single run, so the harness is a rate.
`inspect:language-reap-rate` builds the test binary twice in a disposable copy,
once as it is and once with the former fixed pause,
and starts many test processes at a time inside one container bounded to two processors:

```sh
# package/ide/app
mise run //package/ide/app:inspect:language-reap-rate <target-cache> [cargo-home] [runs] [parallel] [rounds]
```

It counts each failed run by what it left:
a zombie,
a process still running,
or no child (for example a server that was not ready in time).
The counting was checked with a copy of the task whose second variant removes the reaping and adds no pause:
of 10 runs each, 7 and 8 were counted as zombies, and none with the reaping.
A short run can show nothing:
60 runs at sixteen at a time found no leftover process with either variant on 2026-10-05,
while the measurements listed in "Measurements" used 600 or 150 runs each
with a separate script and prebuilt binaries.

The guard control removes the reaping call and expects a zombie from the test whose server must be killed:

```sh
# package/ide/app
mise run //package/ide/app:inspect:language-lifecycle-guards <target-cache> [cargo-home] [helix-clone] sources
```

### Measurements

All runs are `lifecycle::dropping_the_worker_leaves_no_child_process` unless a line names another test,
one test process per run, in one container bounded to two processors and 2 GiB.
"Before" is the fixed 50 ms pause; "after" is the runtime drop followed by reaping for at most two seconds.
The machine has 16 processors and was shared with other work throughout.
Each line gives the load average and the ten-second "some" figure of `/proc/pressure/io`
that were read when the measurement started;
"not recorded" marks the measurements taken before those were read.
A failed run is counted by what it left:
a zombie (state `Z`),
a process still running (state `D`),
or neither (a server that was not ready within twenty seconds, or one that was killed before it read `shutdown`).

#### Sixteen test processes at a time, before

- 34 of 600 failed, all with a zombie. Load not recorded.
- 55 of 600 failed, all with a zombie. Load not recorded.
- 53 of 600 failed: 29 zombies, 17 still running, 7 neither.
  Load 85 and pressure 88 percent, read once while this measurement and the next "after" one ran.
- 69 of 600 failed: 52 zombies, 16 still running, 1 neither. Load 21, pressure 1 percent.
- 61 of 600 failed, all with a zombie. Load 97, pressure 6 percent.

A measurement before these reported 1 of 600 and is discarded:
a quoting mistake in the measuring script made all 600 runs write the same log and the same failure marker
(`ide-language-verify-zombie-baseline-contended-0r07T4/runs/` holds one `.log` and one `.failed`),
so its count says only that at least one run failed.

#### Sixteen test processes at a time, after

- 0 of 600 failed in a probe build whose bound was ten seconds. Load not recorded.
  Its 1200 shutdowns waited a median of 0 ms until no child was listed,
  more than 50 ms in 81 of them,
  more than one second in 3,
  and never more than 1457 ms, which is where the two seconds come from.
- 2 of 600 failed: 1 zombie, 1 neither. Load not recorded.
- 71 of 600 failed: 24 zombies, 35 still running, 12 neither. Load 85, pressure 88 percent.
- 0 of 600 failed. Load 46, pressure 17 percent.
- 42 of 600 failed: 16 zombies, 4 still running, 22 neither. Load 69, pressure 39 percent.
  This measurement took 359 s; the others at this level took 116 to 176 s.
- `lifecycle::server_that_ignores_exit_is_killed_and_reaped_before_the_drop_returns`:
  4 of 600 failed, all neither (four servers not ready in time, in runs active at the same moment).
  Load 76, pressure 36 percent.

#### Two test processes at a time, the test suite's own level

Before:

- 0 of 150 failed. Pressure 14 percent, load not recorded.
- 4 of 150 failed: 3 zombies, 1 still running. Pressure 59 percent, load not recorded.
- 5 of 150 failed: 3 zombies, 2 still running. Load 58, pressure 37 percent.

After:

- 4 of 150 failed: 2 zombies, 2 still running. Pressure 54 percent, load not recorded.
- 2 of 150 failed: 1 zombie, 1 still running. Pressure 50 percent, load not recorded.
- 1 of 150 failed: neither. Load 56, pressure 57 percent.

One at a time, before: 0 of 20 failed. Load not recorded.

#### What the measurements show

At sixteen at a time, "before" left zombies in every measurement, 29 to 61 of 600.
"After" left 0 or 1 of 600 in three measurements,
and 16 and 24 of 600 in the two measurements that also had processes still running and servers not ready in time.
In those two, processes stalled for seconds,
and a killed process that needs longer than the two seconds to end
ends after the worker thread is gone, where nothing reaps it.
That is the limit of any bounded wait.

At two at a time the measurements differ from each other by as much as the variants do:
zombies in 0, 3, and 3 of 150 runs before, and in 2, 1, and 0 after.
No difference is shown at that level.

Runs that left a process still running came in groups that were active at the same moment
(runs 33 to 41 of one measurement, for example),
which fits a stall of the whole container or machine and not something inside one run.
What those processes waited for inside the kernel was not traced.

### Cases without a leftover process

- The server ends before its handle is dropped: the `try_wait` in the drop reaps it.
- The server ends while a runtime in the process still parks: the park after `SIGCHLD` reaps it.
- The server is killed when the worker's runtime is dropped and ends within the reaping allowance:
  the second runtime reaps it.

### Cases with a zombie

- Before the change: the server ends more than 50 ms after the registry was dropped.
- Before the change: the server had not answered `initialize`, and has not ended when the runtime is dropped.
- After the change: the killed server needs more than two seconds to end,
  which happened only in measurements where processes stalled for seconds.

### Cases with a process still running

- The killed server is in state `D` for longer than the test waits (five seconds).
  No change in the worker can end such a process sooner.

## Verified workarounds

### Drop the runtime first, then reap on a second runtime until the kernel lists no child

This is what `package/ide/app/src/language/worker.rs` and `src/language/reap.rs` do now:

```rust
// package/ide/app/src/language/worker.rs, end of the worker thread
            drop(runtime);
            reap::finish(REAP_GRACE);
```

`reap::finish` builds a current-thread runtime and sleeps in steps of 2 ms,
so that runtime parks and drains tokio's process-wide queue,
until `/proc/thread-self/children` is empty or `REAP_GRACE` (two seconds) has passed.

Tradeoffs:

- The longest shutdown grows from 1.05 s to 3 s.
  The extra time is spent only while a killed process has not ended;
  when every server ended by itself the wait is over at the first look, sooner than the fixed pause was.
- It relies on tokio draining its process-wide queue from any runtime that parks.
  That is how tokio 1.53.2 is built, not a documented interface.
  `lifecycle::dropping_the_worker_leaves_no_child_process`
  and `lifecycle::server_that_ignores_exit_is_killed_and_reaped_before_the_drop_returns` fail if it changes.
- `/proc/thread-self/children` exists on Linux kernels built with `CONFIG_PROC_CHILDREN`.
  Without it the wait cannot see completion and uses the whole allowance.
- A killed process that needs longer than the allowance to end still becomes a zombie.
  Measured: 0 or 1 of 600 runs in three measurements,
  and 16 and 24 of 600 in two measurements in which processes stalled for seconds.

## What does not work

- A fixed pause before the runtime ends. 50 ms was too short in 81 of 1200 measured shutdowns,
  and no pause covers the server held by the `initialize` task.
- Waiting inside the worker's own runtime before dropping it. See "A reading that was wrong".
- A longer bounded wait alone.
  In the measurements with stalls, a killed process stayed in state `D` for more than five seconds.
- A runtime that parks without a timeout as a permanent reaper.
  This follows from `orphan.rs:92` and was not run:
  tokio starts listening for `SIGCHLD` only when a park returns while the queue is not empty,
  so a runtime that sleeps until a signal would not be woken for the first queued process.
- Collecting the exit status by process number with `waitpid`.
  The standard library offers it only through the handle, which helix-lsp owns;
  calling it directly needs a new direct dependency or `unsafe`, neither of which this package has adopted.
- A longer wait in the test, or ignoring state `Z` there. It would hide the processes this document is about.

## Upstream filing decision

`.out-of-scope/` has no entry for tokio, helix-lsp, or process reaping
(entries present: `bun-install.md`, `cargo-workspace.md`, `claude-code-upstream-bugs.md`, `codex-harness.md`, `jsr.md`,
`lightningcss.md`, `low-impact-typescript-formatting.md`, `module-es-monolith.md`, `pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`, `typescript-project-references.md`).

Duplicate search, on 2026-10-05:
`gh search issues --repo tokio-rs/tokio -- zombie` and `-- kill_on_drop`,
and `gh search issues --repo helix-editor/helix -- zombie`.

- [tokio-rs/tokio#2685][tokio-2685], closed, is this behavior:
  "tokio::process::Command leaves zombies when child future is dropped".
  A maintainer answered that it "is a known limitation that we do not call `waitpid` on drop",
  and closed it after the documentation quoted in "tokio documents the behavior" was written.
- [helix-editor/helix#4068][helix-4068], closed, is about a server's own child process that kept running after the
  editor quit, not about an unreaped process.

The constraints:

1.  Upstream's fault: no. tokio documents best-effort reaping and recommends awaiting the handle.
    The Helix editor stops its servers as the last step of closing (`helix-term/src/application.rs:1362`,
    `self.editor.close_language_servers(None).await;`) and then exits, and a process that exits leaves no zombies.
2.  Upstream can fix it: yes for tokio in principle
    (a final drain when a runtime shuts down, or a public call that drains).
    Not traced further, because the first constraint already decides.
3.  Supported use: tokio supports dropping a handle only with the stated best-effort cleanup.
    The worker here uses helix-lsp inside a process that keeps running after its servers are gone,
    which the editor itself does not do.
4.  Contribution welcome: tokio's `CONTRIBUTING.md` has no statement about assisted reports
    (searched for "AI", "LLM", "generated", "machine" on 2026-10-05). Not decisive here.
5.  Likely to be fixed: leaning no. The maintainer answer in tokio-rs/tokio#2685 calls it a known limitation
    and settled it with documentation.
6.  Prototype: none. The first and fifth constraints fail, so the prototype step does not apply.

Decision: do not file, and do not comment.
There is nothing to add to tokio-rs/tokio#2685: it is closed,
and this case is the limitation it already describes.

## Evidence

Paths are inside `~/temp/agent/` on the investigating machine and are not kept in the repository.

- `ide-language-verify-zombie-*/summary.json`: one summary per measurement of 2026-10-05,
  and `runs/` beside it with one log per run and a `.failed` marker for each failed run.
- `ide-language-verify-zombie-probe-state-bVeTNk/runs/`: the build that printed child states at the bound.
- `ide-language-verify-zombie-probe-bound10s-hnqbjk/` and `ide-language-verify-r1/reap-waits.txt`:
  the probe build with the ten-second bound and its 1200 reaping waits in milliseconds.
- `ide-language-verify-r1/zombie-two.json`: the two-at-a-time measurements with the pressure figures at each start.
- `ide-language-verify-r1/zombie-quiet.json`: the measurements that recorded load and pressure at each start.

[tokio-2685]: https://github.com/tokio-rs/tokio/issues/2685
[helix-4068]: https://github.com/helix-editor/helix/issues/4068
