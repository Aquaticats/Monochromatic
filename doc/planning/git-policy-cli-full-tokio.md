# Moving the Git wrapper onto tokio as a whole: migration plan

## Status and how to read this

- Status:
  plan for the owner's review;
  nothing in `package/git-policy/cli` is changed by it.
- Written:
  2026-11-07T05:20:00-05:00
- Wrapper measured:
  `package/git-policy/cli`,
  crate `monochromatic-git-policy-cli` 0.5.34,
  source at `d62451d5919b3250f388157ec4acdf2a3c2f688c` (the `Cargo.toml` still says 0.5.34,
  so the release that the
  split plan calls step 0 has not been made at that commit).
- Purpose:
  decide whether the order of steps in section "The order of the move" and the answers to section "Open choices for the
  owner" are what the owner wants before any code moves.
- Inspect first:
  section "What tokio cannot do as the code needs" (five findings that change the design),
  section "The runtime" (the measured cost and the two failure modes of a multi-thread runtime),
  and section "Open choices for the owner".
- Respond by:
  a one-line answer per open choice;
  the ranking in each choice is the recommendation,
  and a veto of the top option is enough.
- Product changes:
  none.
  Every build and run was in a throwaway worktree (`.claude/worktrees/full-tokio-plan`,
  removed at the end) or in
  `${HOME}/temp/agent/full-tokio/`.

## The rulings this plan follows

The owner,
2026-11-07,
verbatim and in the order given:

- "Adopt tokio,
  do not adopt ureq,
  use the `hyper` set of https libs."
- On the runtime the owner picked "Multi-thread,
  sized to budget":
  the worker count comes from the wrapper's existing budget for parallel work,
  as many threads as the computer can spare at that moment.
- "Full on tokio.
  Multi-thread."
- "Rust nightly should always be latest."
  The pin moves in `doc/planning/rust-nightly-always-latest.md` (another agent;
  the file did not exist when this was
  written).
- Standing rules that bind every step:
  "commits vs native git should not take 5s longer.";
  "Commits should never fail.";
  "never discard user data";
  "nothing should ever wait forever or indefinitely";
  "Use as few locks as possible.";
  "We should never invoke ourselves recursively.
  That's bad design.
  Audit,
  remove,
  and ban that design aggressively.";
  "Disk pressure being high is not an excuse.";
  "load is not an excuse.";
  "make sure we're running everything that can be run in parallel in parallel";
  "Ignore Win and mac until I say so.";
  "(debuginfo level,
  incremental off for dependencies - no messing with these."
  and "other profile settings (split-debuginfo,
  strip,
  codegen-units) should be off-limits too".

Reading of "Full on tokio" used here:
everything in the wrapper that waits on input or output,
on a child process,
on a timer,
on a socket or on a signal is
written as asynchronous code on tokio,
and runs on one runtime that the process owns.
Where a part of that cannot work,
or makes a standing rule worse,
the section "What tokio cannot do as the code needs" says
so with evidence,
and the plan does not plan around it in silence.

The toolchain note of the coordinator (2026-11-07):
the build failure of `zbus` on `nightly-2026-11-01` was a compiler regression (rust-lang/rust issue 165263,
fixed by
revert PR 165266) that exists only in that nightly,
and every nightly from 2026-11-02 through 2026-11-07 compiles `zbus` 5.19.0 and 6.2.1
(`doc/research/wrapper-git-free-jobs-and-published-crates.md` at `7956f6c4c9`).
So this plan has no step that works around the old pin,
and every number here comes from
`nightly-2026-11-07` (`rustc 1.101.0-nightly (c980177d8 2026-11-06)`,
`cargo 1.101.0-nightly (111443f3d 2026-11-05)`).

## How the numbers were made

- Host:
  28 hardware threads,
  about 61 GiB of memory,
  shared with other sessions;
  the load average was 24 to 34 during every timing run,
  so the runs say what holds under the load there is.
- Every build and run used `systemd-run --user --scope` with `CPUQuota`,
  `MemoryMax`,
  `TasksMax` and `IOWeight=10`
  sized to it (builds 1200% and 30G;
  timing runs 200% and 4G).
  No synthetic load was made.
- Throwaway build:
  the worktree at `d62451d591` with one cargo feature `full_tokio_probe`
  (`tokio` 1.54.1,
  the newest release,
  optional dependency;
  a block in `main.rs` that builds a runtime from the
  environment variable `FULL_TOKIO_PROBE` before anything else of the wrapper runs,
  then leaks the runtime).
  With the feature off the code is today's code.
  Both binaries were built with the release profile unchanged (`cargo build --release --offline`).
- Arms of the timing,
  `rev-parse HEAD` in a scratch repository,
  rotating order,
  600 rounds per arm per run,
  three runs:
  real Git directly;
  today's binary (feature off);
  a byte copy of today's binary (the same-binary control);
  the probe binary with the variable unset (so the runtime code is linked and unused);
  and the probe binary building a current-thread runtime,
  a multi-thread runtime with tokio's default worker count,
  and a multi-thread runtime whose workers come from the wrapper's budget.
- Scratch tools (not in the repository):
  `${HOME}/temp/agent/full-tokio/classify.ts` and `count.sh` (the inventory),
  `bench.ts` and `asmin.ts` (timing and address-space runs),
  `probe-unix/` (small tokio probes for sockets,
  blocking pool,
  children and memory).

## Inventory of blocking and thread-based constructs

### How the files and lines were counted

- Non-test source:
  `src/native/**/*.rs` minus every module a `#[cfg(test)] mod x;` line or a `#[path = "x_tests.rs"]` line declares,
  minus the `testing/` and `source_guard/` folders,
  and minus `*_tests.rs` and `*_test_support.rs`:
  623 files (the folder holds 1288 `.rs` files;
  659 are test files).
  The brief said about 580;
  the count by this rule is 623 (`classify.ts` prints it).
- Counts are lines that match,
  outside lines that start with `//`,
  found with
  `rg --pcre2 --count-matches '^(?!\s*//).*(?:PATTERN)'` over that file list (`count.sh`).
  A line counts once however many matches it has.
  Doc comments and `//` comments do not count;
  string literals and attributes do.
- Some non-test files carry `#[cfg(test)]` items (test hooks).
  Of them only the `thread_local!` blocks matter to the plan,
  and they are split in the thread-local item.
- There is no `async fn`,
  `.await`,
  `tokio` or `futures` in non-test source at all (0 files):
  the move starts from no asynchronous code.

### Counts by construct

Each line gives files and matching lines,
then the files with the most matches.
"Commit path" means the process that a person waits on when they run `git commit`:
the short-lived command process (client) or the resident commit service (service) that lands the commit.

- Child processes started (`Command::new`):
  9 files,
  12 lines (`process/process_start.rs` 2,
  `lock/index_lock_holders.rs` 2,
  `lfs/lfs_upload.rs` 2,
  `repo_read/git_system_config.rs`,
  `invocation/forwarding.rs`,
  `invocation/management_service_repository_selection.rs`,
  `push/auto_push_single_flight.rs`,
  `repair/package_repair_command.rs`,
  `process/process_identity.rs`).
  Every program started is real Git,
  `ps`,
  `lsof`,
  `git-lfs` or a package-repair tool;
  none is the wrapper itself.
  Real Git is built in one place (`invocation/forwarding.rs`,
  `git_command`) and started through `start_retry.rs`
  (`status_retrying`,
  `output_retrying`,
  `spawn_retrying`,
  `exec_retrying`;
  12 files,
  20 lines call them).
  Commit path:
  both (the client prepares the commit with Git children;
  the service runs `fast-import` and `update-ref`).
- Child waits (`.wait()`,
  `.wait_with_output()`,
  `.try_wait()`,
  `.output()`,
  `.status()`):
  15 files,
  28 lines (`push/bounded_push.rs` 4,
  `repair/package_repair_command.rs` 3,
  `push/child_watch.rs` 3);
  `try_wait` polling loops:
  2 files,
  5 lines (`push/bounded_push.rs`,
  `repair/package_repair_command.rs`).
  Commit path:
  both.
- Threads (`thread::spawn`,
  `thread::Builder`,
  `spawn_scoped`,
  `thread::scope`):
  12 files,
  17 lines;
  `thread::scope` is 1 file,
  2 lines (`foundation/side_by_side_threads.rs`);
  all uses of the `thread` module:
  35 files,
  73 lines.
  Main sites:
  `service/service_server.rs` (work thread,
  acceptor thread,
  one thread per connection),
  `push/child_watch.rs` (one watcher thread per watched wait),
  `landing/service_claim_renewal.rs`,
  `landing/service_durable.rs` (background sync thread),
  `landing/service_index_prestage.rs`,
  `landing/earlier_service_process_look.rs`,
  `standards_review/standards_review_run.rs`,
  `process/thread_quiet.rs`,
  `foundation/scoped_start.rs` and `foundation/side_by_side_threads.rs` (scoped workers with the run-in-turn fallback),
  `foundation/parallel_runner.rs`,
  `repair/package_repair_command.rs`.
  Commit path:
  service (acceptor,
  per-connection threads,
  landing loop,
  durable-sync thread,
  claim renewal) and client
  (`ChildWatch` thread,
  guarded Markdown linter thread).
- Sleeps and polling:
  `thread::sleep` and `park_timeout`:
  18 files,
  26 lines
  (`push/bounded_push.rs` 6,
  `service/service_client_reach_service.rs` 2,
  `repair/package_repair_command.rs` 2,
  `landing/service_index_wait.rs` 2,
  and one each in `service/service_client.rs`,
  `service/service_provision.rs`,
  `commit/commit_preparation.rs`,
  `transaction/transaction_git.rs`,
  `transaction/transaction_phase.rs`,
  `push/start_retry.rs`,
  `lock/index_holder_wait.rs` and others);
  the shared pause constant `POLL_PAUSE`:
  5 files,
  14 lines.
  Commit path:
  client (`commit_preparation.rs` waits for the signal guard in a sleep loop;
  the service reach loop;
  the
  start retry) and service (index wait,
  hold retry).
- Deadline and no-progress watchers:
  `push/child_watch.rs` (473 lines),
  `push/child_watch_clock_marks_and_stopping.rs`,
  `push/push_progress.rs`,
  `push/wait_progress.rs`,
  `process/thread_quiet.rs`,
  `process/quiet_ledger.rs`,
  `markdown/markdown_linter_guard.rs`,
  `push/bounded_push.rs`,
  `push/bounded_query.rs`;
  `recv_timeout` is 9 files,
  11 lines;
  all channel use (`mpsc`) is 12 files,
  23 lines.
  Commit path:
  client (every watched Git step of the preparation,
  the Markdown linter guard) and service (landing steps).
- Atomics and locks inside the process:
  `Atomic*`:
  32 files,
  100 lines (`push/child_watch.rs` 14);
  `Mutex`,
  `RwLock`,
  `Condvar`:
  7 files,
  17 lines
  (`foundation/scoped_start.rs`,
  `service/journald_native.rs`,
  `landing/service_durable.rs`,
  `worktree/add_timing.rs`,
  `standards_review/standards_review_run.rs`,
  `process/quiet_ledger.rs`,
  `candidate/candidate_store_commit_listings.rs`).
- Unix sockets (`UnixListener`,
  `UnixStream`,
  `UnixDatagram`):
  12 files,
  54 lines;
  `UnixListener` is 2 files,
  9 lines (`service/service_server_socket_binding.rs`,
  `service/service_client_listen_and_ask_to_retire.rs`);
  the accept loop is one site,
  `service/service_server.rs` (`listener.incoming()`,
  1 file,
  4 `accept` lines);
  `UnixDatagram` is `service/journald_native.rs` (non-blocking `sendto`,
  `poll` with a zero timeout).
  Commit path:
  client (`service_client*.rs`) and service (all of it).
- Signal handling:
  `sigaction` and relatives:
  1 file,
  12 lines (`process/signal_relay.rs`,
  494 lines);
  `libc::kill`:
  8 files,
  8 lines;
  the service's own last-line handler is `service/service_signal.rs` (259 lines,
  only calls that `signal-safety(7)` lists).
  `SignalGuard` is used by `commit/commit_preparation.rs` (4 lines) and `lfs/lfs_upload.rs` (2 lines).
  Commit path:
  client (the guard wraps the Git commit of the preparation).
- Process groups and `pre_exec`:
  `process_group(0)` in 1 file (`repair/package_repair_command.rs`);
  `CommandExt::exec` in 1 file (`push/start_retry.rs`,
  which `forwarding.rs` calls through `exec_retrying`;
  the package-repair
  file imports the trait for `process_group`);
  `pre_exec`,
  `setsid`,
  `setpgid`,
  `fork`,
  `vfork`:
  0 files.
  `pre_exec` appears only in test files (`binary/binary_fault_limits_tests.rs` and others),
  where a closure sets resource
  limits on a child before it starts the built wrapper.
  Commit path:
  none (the package-repair command is not a commit step).
- File input and output:
  `std::fs`,
  `File::open`,
  `OpenOptions` and similar:
  205 files,
  916 lines;
  `sync_all`,
  `sync_data`,
  `fsync`:
  15 files,
  44 lines
  (`config/config_snapshot.rs` 7,
  `landing/service_land.rs` 6,
  `landing/service_durable.rs` 6,
  `landing/service_land_objects_of_batch.rs` 5);
  directory walks (`read_dir`,
  `walkdir`):
  59 files,
  77 lines;
  `/proc/` reads:
  16 files,
  25 lines.
  Commit path:
  both;
  the service's fsyncs are already queued to a background thread by `landing/service_durable.rs`
  ("Force at leaving,
  else background").
- Inotify:
  1 file,
  3 lines (`landing/service_index_watch.rs`,
  248 lines,
  a read loop over `libc` with `poll`;
  the research
  document already picks `rustix::fs::inotify` for it).
  Commit path:
  service (waiting for a holder of `index.lock`).
- File locks and exclusive creation:
  `libc::flock`:
  1 line (`worktree/worktree_convert_journal.rs:397`,
  non-blocking);
  `File::try_lock`:
  2 lines (`landing/service_land_copy.rs`,
  non-blocking);
  `O_EXCL` and `create_new`:
  18 files,
  21 lines (claims,
  journals,
  staging files,
  `private_storage.rs`).
  None blocks.
  Commit path:
  service (`service_land_copy.rs`,
  `service_journal.rs`,
  `service_index_staging.rs`) and both for
  `private_storage.rs`.
- Panic boundaries:
  `catch_unwind` and `resume_unwind`:
  5 files,
  8 lines
  (`markdown/markdown_linter_guard.rs`,
  `invocation/panic_notice.rs`,
  `foundation/call_catching_panics.rs`,
  `foundation/parallel_runner.rs`,
  `repair/package_repair_cargo_concurrent.rs`).
  Commit path:
  client (the Markdown linter runs under a guard that catches its panic).
- Netlink socket diagnostics:
  1 file,
  8 lines (`push/tcp_sockets.rs`,
  318 lines,
  raw `libc` socket,
  one request and one reply per sample).
  Commit path:
  none directly (the push watcher;
  the service's after-commit work).
- HTTPS and D-Bus:
  `ureq`:
  1 file,
  5 lines (`standards_review/standards_review_client.rs`);
  the hand D-Bus client `service/systemd_dbus.rs` (395 lines) and its wire-format file;
  they are replaced by `hyper` and `zbus` 6 on tokio by the owner's rulings and the measured prototype.
  Commit path:
  client reaches the service through the unit manager when nothing answers (`systemd_dbus`);
  the HTTPS call is the shadow review after landing (not on the wait path).
- Thread-affine state:
  `thread_local!` in 17 non-test files (18 lines).
  Ten are production state:
  `worktree/add_timing.rs`,
  `service/service_busy_step_record_on_this_thread.rs` (the landing board and detail),
  `repo_read/repo_read_tree.rs` (an `Rc` cache,
  so a `!Send` value),
  `repo_read/repo_read_odb.rs`,
  `landing/service_land_git_steps_and_import_stream.rs`,
  `landing/service_index_prestage.rs`,
  `landing/service_index_ownership.rs`,
  `landing/service_index_claim.rs`,
  `landing/service_durable.rs`,
  `landing/service_built_checks.rs`.
  Seven sit under `#[cfg(test)]` (fault hooks such as `foundation/scoped_start.rs` `STARTS_FAIL_ON_THIS_THREAD`).
- Unsafe:
  43 files,
  131 lines;
  the signal relay,
  the sock-diag reader,
  the btrfs and `openat2` calls and the socket activation
  `from_raw_fd` are the I/O ones.
- Process exit:
  `std::process::exit`:
  6 files,
  7 lines (`main.rs` 2).

### What the inventory says about the shape of the move

- The commit service is a thread-per-connection server over `std::sync::mpsc` queues with a serialized landing loop;
  its natural tokio form is an accept loop that spawns a task per connection and one landing task.
- The wrapper already separates its waits from its watchers (`ChildWatch` marks the begin and the end of a wait and a
  watcher judges it),
  which is the shape of a `select!` over the wait and a timer,
  so the watcher threads go away rather than being translated line by line.
- The thread-affine state and the `!Send` cache are the largest obstacle to a multi-thread runtime,
  not the child,
  socket or timer code (section "Thread-affine state and the landing loop").
- The forwarded commands (`Action::Forward` in `invocation/entry.rs`) replace the process with Git and never reach any
  construct in this list;
  a strace of `rev-parse HEAD` and of `status --short` on today's binary shows one `execve` of
  the wrapper and one of Git and no `clone`,
  `clone3`,
  `fork` or `vfork`.

## What tokio cannot do as the code needs

All claims about tokio are read from tokio 1.54.1 (the newest release;
crates.io `max_stable_version`,
published
2026-10-21) in the registry copy
`.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.54.1`,
or measured with `probe-unix`.

### A multi-thread runtime panics, then hangs, when the system refuses it threads

This breaks "Commits should never fail."
and "nothing should ever wait forever" for a wrapper that builds the runtime
eagerly.

- Worker threads are started through the blocking pool:
  `Launch::launch` calls `runtime::spawn_blocking(move || run(worker))` for each worker
  (`src/runtime/scheduler/multi_thread/worker.rs:515`),
  and `Spawner::spawn_blocking` ends in `panic!("OS can't spawn worker thread: {e}")` when the OS refuses a thread
  and no blocking thread exists (`src/runtime/blocking/pool.rs:385-387`).
  So `Builder::build()` of a multi-thread runtime panics instead of returning an error.
- Measured under `prlimit --nproc=1` on the probe binary:
  with a current-thread runtime `rev-parse HEAD` printed the hash;
  with a multi-thread runtime (default workers and budget workers) the process exited with status 101 and
  "internal error at ... worker.rs:515:13".
- Measured under `prlimit --as=<size>` (address-space limit) on the probe binary,
  in two sweeps with a timeout:
  with 28 workers six runs at each of 600,
  800,
  1000 and 1500 MiB exited 0,
  and six runs at each of 1030 and 1100 MiB
  hung until `timeout` ended them;
  one run at each of sixteen sizes from 900 to 2000 MiB hung at 900,
  950,
  1020,
  1025,
  1030,
  1100 and 1150 MiB with default
  workers and at the same sizes except 1100 with the 25 workers of the budget,
  and worked at the others
  (the band is not a threshold;
  it depends on where the mappings fall).
  The same binary with the variable unset,
  and with a current-thread runtime,
  never failed at those sizes
  (today's binary needs about 20 MiB).
  A hung process had 24 threads,
  and its main thread sat in a futex wait after the panic message of `worker.rs:515`
  (`eu-stack`;
  the binary is stripped,
  so the frames have no names).
  Reading:
  the panic unwinds out of `build()` while the already started workers are waiting,
  and dropping the half-launched runtime waits for them (tokio's runtime drop "waits forever",
  `src/runtime/runtime.rs`
  struct documentation,
  section "Shutdown").
  That reading is an inference from the stack and the documentation;
  the cause was not stepped through.
  Because the hang is inside the unwind,
  a `catch_unwind` around `build()` cannot guard it.
- Why the address space runs out:
  each worker adds 67,608 KiB of virtual size (measured by reading `/proc/self/status` after `build()`:
  28 workers add 1,892,944 KiB),
  and each blocking-pool thread adds 67,600 KiB.
  With `MALLOC_ARENA_MAX=1` the same 28 workers add 57,936 KiB and a blocking thread 2,064 KiB,
  so the weight is the glibc per-thread malloc arena (64 MiB of reserved address space),
  not tokio's 2 MiB stack.
  With `MALLOC_ARENA_MAX=1`,
  28 workers ran at 100,
  200,
  400,
  900,
  950,
  1020 and 1150 MiB,
  and hung again at 30,
  40
  and 60 MiB.
- Consequence for the design:
  the worker count must be chosen with the address-space and process limits in view before `build()`
  (`getrlimit` for `RLIMIT_AS` and `RLIMIT_NPROC`,
  which `foundation/parallel_budget.rs` does not read today),
  the arena count must be capped before the first worker starts (`mallopt(M_ARENA_MAX, ...)`,
  to be measured for
  allocation contention),
  and a limit that leaves no room must select the current-thread runtime.
  The existing suites already run the wrapper under such limits
  (`binary/binary_fault_limits_tests.rs`:
  descriptor,
  address-space and process limits on the wrapper and on the
  service),
  so the proof exists once the runtime is in the binary.

### Blocking-pool work panics, not errors, when no thread can start

- `tokio::task::spawn_blocking` panics with the same message (`pool.rs:385-387`);
  `tokio::fs` calls it (`src/fs/mod.rs:323-330`,
  `asyncify`),
  so every `tokio::fs` operation panics under the same limit.
- Measured under `prlimit --nproc=1` with a current-thread runtime:
  `spawn_blocking` panicked and the panic could be caught with `catch_unwind` (the closure is not returned);
  `tokio::fs::metadata` panicked outside any guard and ended the process with status 101.
- The wrapper's rule for this exists in `foundation/scoped_start.rs`:
  a thread is only a way to work sooner,
  so the work runs in turn where a thread cannot start,
  and the closure is handed over through a slot that either side
  can empty.
  The tokio form is one helper that puts the job in the same kind of slot,
  calls `spawn_blocking` inside
  `catch_unwind`,
  and runs the job on the calling task when the call panics.
  Plain `tokio::fs` is not used.
  The existing child-process harness that refuses every thread (`testing/no_threads_child_test_support.rs`) proves it.

### The signal relay cannot be tokio's signal handling

- The relay forwards a signal to the child only when a process sent it,
  not the terminal,
  by reading `si_code`
  (`process/signal_relay.rs`:
  `SA_SIGINFO` handler,
  `is_process_sender`),
  and it restores the previous dispositions when the guard drops,
  and it leaves an ignored signal ignored.
- `tokio::signal::unix` registers with `signal_hook_registry::register(signal, move || action(globals, signal))`
  (`src/signal/unix.rs:286`),
  the variant that gives the action no `siginfo_t`;
  the variant that does (`register_sigaction`) is not what tokio calls.
- tokio never restores:
  "Once a signal handler is registered with the process the underlying libc signal handler is never unregistered"
  (`src/signal/unix.rs`,
  documentation of `signal`),
  and `signal_hook_registry::unregister` leaves the signal ignored,
  not at its default:
  "the program will effectively ignore `SIGTERM` signals from now on" (`signal-hook-registry` 1.4.8,
  `src/lib.rs`
  documentation of `unregister`).
  The registry also chains the previous handler and installs a handler even where the previous disposition was ignore.
- So the relay stays a raw `sigaction` handler that calls `kill`,
  and stays small.
  It does not need the runtime at all,
  because the handler does the relay itself.
  What changes is the wait:
  `waitid(WNOWAIT)` blocks a thread,
  so the relay is rewritten to hold a pidfd of the child
  (`pidfd_open`,
  available through `rustix`,
  which the owner picked) and to signal it with `pidfd_send_signal`
  from the handler;
  a signal sent after the child was reaped then fails with `ESRCH` instead of reaching a reused pid,
  which is the hazard the `WNOWAIT` dance guards against today.
  That removes the blocking wait,
  not the guard.
  The design is proposed here,
  not built or tested;
  the existing `process/signal_relay_tests.rs` and
  `binary/binary_fault_signal_tests.rs` are its controls.
- tokio's own child reaping does not conflict:
  on Linux `tokio::process` opens a pidfd for each child (`src/process/unix/pidfd_reaper.rs`) and falls back to a
  `SIGCHLD` listener only where `pidfd_open` is missing (`src/process/unix/mod.rs`,
  `build_child`).
  The `SIGCHLD` fallback would install a handler that is never removed;
  it is harmless for `SIGCHLD`,
  which the
  relay does not touch.

### `tokio::process` can report a failed start after the child has started

- `Command::spawn` is the standard library's spawn followed by `build_child`
  (`src/process/mod.rs:863-866`).
  `build_child` converts the pipes and registers the pidfd with the reactor,
  and either failing returns an error while
  the standard library child is already running
  (`src/process/unix/mod.rs`,
  `build_child`:
  `Err((Some(err), _child)) => return Err(err)`;
  `src/process/unix/pidfd_reaper.rs:151-162`,
  the `PollEvented::new_with_interest` failure).
  Pipe conversion cannot fail for the wrapper's children,
  which use null or file descriptors (`output_capture.rs`);
  the reactor registration (`epoll_ctl`) can fail under a memory or watch limit.
  This was read from source,
  not triggered.
- The wrapper's start retry repeats a start that failed with `EAGAIN` for a bounded time (`push/start_retry.rs`).
  A start that reports an error while Git is running leaves an unsupervised Git beside a wrapper that goes on to its
  failure cleanup (in the commit preparation,
  that cleanup removes the shadow repository Git is writing).
- tokio 1.54.1 has no public way to hand it a child that was started elsewhere:
  `from_std` exists only for the three pipe types (`src/process/mod.rs:1512,1527,1542`),
  and `spawn_with` needs
  `tokio_unstable` (`src/process/mod.rs:930-940`).
- Plan (open choice 7):
  start with the standard library (`start_retry.rs`,
  unchanged,
  so a failed start leaves no child),
  open a pidfd (`rustix::process::pidfd_open`),
  wait for it with `tokio::io::unix::AsyncFd` (a pidfd is readable when
  the process exits) and reap with the standard library child's `try_wait`;
  where `pidfd_open` is missing,
  wait on the blocking pool.
  That is about a hundred lines of code,
  it reuses the pidfd the signal relay needs,
  and it is proposed,
  not built.
  The alternative is plain `tokio::process::Command`,
  which is simpler and has the failure just described.

### A non-blocking connect to a full Unix socket queue fails at once

- mio treats only `EINPROGRESS` as a pending connect (`mio-1.2.4/src/sys/unix/uds/stream.rs`,
  `connect_addr`),
  and tokio's `UnixStream::connect` returns the other errors.
- Measured with `probe-unix backlog` (a listener with backlog 0 that never accepts):
  the first blocking `std` connect had returned at the 300 ms check,
  the second was still blocked (a full queue);
  `tokio::net::UnixStream::connect` failed in 31 microseconds with `Resource temporarily unavailable (os error 11)`,
  kind `WouldBlock`.
- Today a blocking connect waits in the kernel until the service accepts.
  The service measured bursts of 128 simultaneous clients (`doc/planning/cli-git-resident-service.md`),
  so a full
  queue is a case the design already meets.
  The tokio form is a retry loop on `WouldBlock` with `tokio::time::sleep` and the client's existing deadline
  (`service/service_client_reach_service.rs`),
  which is the polling the file already has for "the service is starting".

### Blocking files, fsync and the runtime's own shutdown

- tokio offers no asynchronous `fsync` on stable.
  `tokio::fs` runs "ordinary blocking file operations behind the scenes" on the blocking pool,
  and io_uring is used
  only "when compiled with `tokio_unstable`" (`src/fs/mod.rs` module documentation).
  So the fsyncs of the commit path stay blocking calls.
  Measured with `probe-unix hop` (`nightly-2026-11-07`,
  release):
  handing an empty job to the blocking pool and back
  costs 7.4 microseconds on a current-thread runtime and 16.6 on a 4-worker multi-thread runtime;
  the first hop,
  which starts the pool thread,
  185 and 83 microseconds;
  `tokio::fs::metadata("/")` takes 17.7 and 10.0 microseconds against 2.0 to 2.4 for the inline call.
  The hop is small against an fsync,
  large against a `stat`:
  file calls that take microseconds stay inline,
  and calls
  that can wait on the disk (`sync_all`,
  large writes,
  directory walks of unbounded size) go to `spawn_blocking` in
  batches,
  as tokio's documentation recommends ("batch your operations into as few `spawn_blocking` calls as possible",
  `src/fs/mod.rs`).
- Dropping a runtime "blocks until all spawned work has been stopped.
  This can take an indefinite amount of time.
  The `Drop` implementation waits forever for this."
  (`src/runtime/runtime.rs`,
  section "Shutdown").
  A blocking task cannot be cancelled,
  and cancelling a `tokio::fs` future leaves the operation running
  (`src/fs/mod.rs`,
  section "Cancellation").
  So no runtime of the wrapper is ever dropped;
  each process ends with `std::process::exit` (as `main.rs` does now),
  or `Runtime::shutdown_timeout` where a runtime must go before the process ends.
- Consequence for "nothing should wait forever":
  a step that waits on a blocking call is bounded by the same watcher as today (`tokio::time::timeout` around the
  `JoinHandle`),
  and the call itself keeps running in the pool until it returns,
  which is the behavior of today's abandoned guarded thread (`markdown/markdown_linter_guard.rs`).
- Consequence for the Markdown linter and the scanner (CPU-bound,
  in process):
  they run on the blocking pool (`spawn_blocking`),
  never on a worker,
  because "code that spends a long time without
  reaching an `.await` will prevent other tasks from running" (`src/lib.rs`,
  section "CPU-bound tasks and blocking
  code").
  `task::block_in_place` is not an option on a current-thread runtime ("This function panics if called from a
  `current_thread` runtime",
  `src/task/blocking.rs`),
  which the fallback to a current-thread runtime can select.

### Thread-affine state and the landing loop

- Ten production `thread_local!` items (inventory) assume that a piece of work stays on one thread,
  and `repo_read/repo_read_tree.rs` holds an `Rc` there.
  On a multi-thread runtime a task can move between workers at each `.await`,
  and a future that holds an `Rc` across an
  `.await` is not `Send`,
  so `tokio::spawn` refuses it at compile time.
- The serialized landing step (`landing/service_land.rs`) is serial by design (`doc/planning/cli-git-resident-service.md`,
  "The serialized step,
  group commit,
  FIFO and backpressure"),
  so asynchronous code gives it no concurrency:
  its Git children and fsyncs are in sequence.
  The tokio forms that keep its thread-affine state valid are a dedicated current-thread runtime with a
  `tokio::task::LocalSet` on one thread for the landing step ("Runs `!Send` futures on the current thread",
  `src/task/local.rs`),
  or turning each `thread_local!` into a `tokio::task_local!` or an explicit context value.
  The first changes no state code;
  the second is the larger rewrite.
  This is open choice 4.

### Things that are fine as they are

- `pre_exec` and fork safety:
  `tokio::process::Command::pre_exec` exists and wraps the standard library's (`src/process/mod.rs:749-756`),
  with the
  same constraint ("normal operations like `malloc` or acquiring a mutex are not guaranteed to work" in the forked
  child,
  lines 735-738),
  and non-test source has no `pre_exec` call,
  so there is nothing to port.
  The standard library spawns with `posix_spawn` unless a user,
  a group,
  a `PATH` override with a bare program name,
  a `chroot` or a `pre_exec` closure is set (`library/std/src/sys/process/unix/unix.rs`,
  `posix_spawn`,
  lines 469-476
  of the `nightly-2026-11-07` source),
  and supports `process_group` in that path (`POSIX_SPAWN_SETPGROUP`,
  line 742),
  so starting Git while worker threads run uses the same spawn as today.
  tokio's `Command::process_group` exists (`src/process/mod.rs:790`).
- A child that must outlive the command or the runtime:
  `kill_on_drop` defaults to false and a dropped `Child` keeps running;
  "The tokio runtime will,
  on a best-effort basis,
  attempt to reap" it (`src/process/mod.rs`,
  section "Dropping/Cancellation" and "Unix Processes").
  Output capture in anonymous files (`process/output_capture.rs`) hands the child plain file descriptors,
  so nothing
  of the runtime is in the child's way.
- `exec` of the forwarded command:
  the forwarded path never builds a runtime (section "The runtime"),
  and a probe that built 28 workers and then forwarded
  `rev-parse HEAD` exited 0,
  so `exec` over live worker threads works.
- Exclusive creation and the three non-blocking lock calls cost microseconds and stay inline.
- The journal writer (`service/journald_native.rs`) is a non-blocking `sendto` made so that "a commit never waits for the
  journal";
  an `await` would add a suspension point where none must be.
  It stays synchronous and callable from any context.
- `catch_unwind` across `.await`:
  `std::panic::catch_unwind` takes a closure and cannot wrap an `.await`.
  A task spawned with `tokio::spawn` has its panic caught by the runtime and returned as `JoinError`
  (`is_panic`,
  `into_panic`,
  `src/runtime/task/error.rs:63,93`),
  so the task boundary is the panic boundary;
  in the same task the form is `futures::FutureExt::catch_unwind` with `AssertUnwindSafe`.
  The release profile already has `panic = "unwind"`,
  which the linked scanner needs.
  The guarded Markdown linter becomes `spawn_blocking` plus the `JoinError`,
  and its no-progress judge
  (`process/thread_quiet.rs`) reads `/proc/self/task/<tid>/` for the pool thread's id,
  recorded inside the job.
- Standing rule about recursion:
  the search of `Command::new` and `current_exe` shows no site that starts the wrapper itself;
  the only `current_exe` call is `invocation/real_git.rs:569`,
  which finds the wrapper in `PATH` to skip it,
  and the
  service is started by the operating system's socket unit,
  not by the wrapper.
  Nothing in the move adds a self-start,
  and a source-guard test that fails on `Command::new` of the current executable
  is step 1 of the order (it needs no tokio).

## The runtime

### Eager at start or lazy at first need: measured

On a command that needs nothing,
`rev-parse HEAD` forwarded to Git
(`nightly-2026-11-07`;
three runs of 600 rounds per arm,
rotating order;
medians in milliseconds;
the load average was 24 to 34):

- Real Git directly:
  2.77,
  3.01 and 2.60.
- Today's binary:
  4.71,
  5.01 and 4.39.
- Byte copy of today's binary (the control):
  4.86,
  5.10 and 4.63.
  The spread of the same bytes against today's binary is 0.09 to 0.24 ms,
  so a difference smaller than that is not a signal.
- Probe binary,
  runtime code linked and unused:
  5.00,
  5.19 and 4.63.
  (It is within the spread of the control:
  linking `tokio` costs nothing at start.)
- Probe,
  current-thread runtime built at start:
  +0.05,
  +0.12 and +0.08 over the unused probe.
- Probe,
  multi-thread runtime with tokio's default 28 workers built at start:
  +0.30,
  +0.42 and +0.30.
- Probe,
  multi-thread runtime with the budget's 25 workers:
  +0.47,
  +0.52 and +0.53.
- Probe,
  only reading the budget (`HostResources::read`) without building a runtime:
  +0.20,
  +0.32 and +0.36.
  The read costs more than the three fewer threads save:
  reading `/proc/meminfo`,
  `/proc/self/cgroup` and
  `memory.max` and `memory.current` of every cgroup on the way up to the root is the price of "as many as the computer
  can spare at that moment".
- Cold start,
  five runs each after dropping the binary's cached pages,
  minor page faults of the whole `rev-parse HEAD`:
  today's binary 475 to 483;
  the probe with unused runtime code 477 to 483;
  the multi-thread build 604 to 610 (about +125).
  Major faults were 4 for today's binary,
  6 for the probe and 1 to 2 for the multi-thread arms;
  they moved with what
  the preceding runs had left in the cache and are not a signal.
- Size:
  today's binary 16,744,056 bytes;
  the probe,
  with the cargo feature that links a builder only,
  16,954,272 (+210,216,
  +1.3%).
  The real conversion links much more of tokio;
  the prototype of the research document,
  which also added `zbus` and `hyper`,
  measured
  +1.2 to +1.3 MB (`doc/research/wrapper-git-free-jobs-and-published-crates.md`,
  section "Measured inside the wrapper").
- Lockfile:
  `tokio` with `rt`,
  `rt-multi-thread`,
  `net`,
  `time`,
  `process`,
  `signal`,
  `fs`,
  `io-util`,
  `sync` and `macros` adds 5
  registry packages (`mio`,
  `signal-hook-registry`,
  `socket2`,
  `tokio`,
  `tokio-macros`);
  without `macros`, 4.
  The `hyper` and `zbus` sets are counted in the research document.

So an eager multi-thread runtime costs 0.3 to 0.5 ms and about 125 minor faults on every command:
about 6 to 11% of the wrapper's own time on a forwarded command (4.4 to 5.0 ms),
10 to 20% of real Git's time on the
same command (2.6 to 3.0 ms),
and 0.01% of the 5 s bound.
Both readings are true;
the owner's rule of 5 s is not threatened,
but the forwarded command is
the wrapper's commonest path and it can have none of this cost.
So the plan is lazy:
the runtime is built by the first code that needs it,
never at `main`.

Where the runtime is built:
`invocation/entry.rs` `run_process` plans an `Action` first;
`Action::Forward` replaces the process with Git (`exec`),
and every other action runs wrapper code.
`run_process` builds the runtime only on the arms that run wrapper code
(`ForwardCoordinated`,
`ForwardAnnounced`,
`ForwardSettled`,
`ForwardPush`,
and the commands that do their own work),
through one function (`run_on_the_runtime(future)` in a new `foundation` file) that owns a `OnceLock` of the runtime,
as the prototype of `lazy_tokio_runtime.rs` did.
The planning that comes before the action spawns no child on the common path (the strace in the inventory).
A planning step that does spawn (`invocation/git_version_cache.rs`,
`repo_read/git_system_config.rs`,
`management_service_repository_selection.rs`:
one-shot `Command` waits with no concurrency and no watcher) stays on the
standard library until step 9,
when the planning is made `async` and enters the same function;
it is the last I/O to move because it is on the path of every command that is not a pure `exec`,
and gains nothing.
This is the one place where "full on tokio" is read as a goal for the end state,
not for every line at once.

### How the worker count is chosen, and where the budget is read

- Today the budget is `foundation/parallel_budget.rs`:
  `HostResources::read()` returns `available_parallelism` (which honors cgroup CPU quota and affinity) and the memory
  available (the lesser of `MemAvailable` and the cgroup headroom of every ancestor),
  and `worker_budget(items, per_item_memory, ceiling, cap)` is `min(items, parallelism - margin, memory/2/per_item,
  ceiling, cap)` with a margin of one eighth of the CPUs when there are more than two.
  It is read at nine call sites,
  each when a job starts (the service reads it for each round,
  `landing/service_work_run_round.rs`).
- For the runtime the request is `items` unbounded,
  `per_item_memory` the weight of one worker,
  and a `ceiling` from the limits the budget does not read today (section "A multi-thread runtime panics,
  then hangs"):
  `RLIMIT_AS` divided by the measured virtual cost of a worker (67,608 KiB with glibc arenas,
  2,064 KiB with
  `MALLOC_ARENA_MAX=1`),
  `RLIMIT_NPROC` minus the processes of the user,
  and the cgroup `pids.max`.
  The gate container (`native:test:container`) has 2 CPUs and 512 pids by default,
  so the budget there is 2 workers.
- One value per runtime:
  tokio cannot resize workers after `build()`.
  For the short-lived command process that is the moment of the command,
  which is the owner's "at that moment".
  For the service,
  which lives up to `IDLE_EXIT` (3 hours),
  it is the moment it started.
  What the owner's wording can be honored for at each moment is the concurrency of the work,
  not the thread count:
  a `tokio::sync::Semaphore` sized by `worker_budget` at the start of each round bounds how many blocking jobs
  and children the round runs at once,
  and the runtime's workers only poll tasks.
  This is open choice 2.
- The worker threads are cheap to keep idle:
  28 workers:
  29 threads,
  3,836 KiB resident,
  1,892,944 KiB virtual (arenas included).
  The pool for blocking work is separate and grows on demand up to 512 threads unless `max_blocking_threads` is set,
  which it must be,
  from the same budget and `pids.max`,
  with a `thread_keep_alive` short enough that a command process
  does not hold threads it no longer needs.

### The short-lived command process against the service

- The command process lives for the length of one command and exits with `std::process::exit`.
  Its runtime is built once,
  lazily,
  current-thread when the budget says one worker or the limits say no threads,
  multi-thread otherwise.
  Its work is a few Git children,
  one conversation with the service and the policy run;
  extra workers help only if the commit preparation runs independent children in parallel (the owner's
  "make sure we're running everything that can be run in parallel in parallel"),
  which `join!` over children does
  on any runtime,
  so worker count matters little here.
- The service is a long-lived process started by the operating system with a passed-in socket.
  It builds its runtime at `service_main::run` (it is already running by decision,
  no cost to a forwarded command) and
  keeps it for hours.
  The listener is made a `tokio::net::UnixListener` with `from_std` after `set_nonblocking(true)`;
  tokio documents that
  "the caller is responsible for ensuring that the listener is in non-blocking mode" and that a blocking listener is
  "always erroneous" (`src/net/unix/listener.rs`).
  `service/service_server_socket_binding.rs` makes the listener from file descriptor 3 (`LISTEN_FDS`) or from a bound
  path,
  so the change is one line there.
- A service started under the limits the wrapper faces must start its acceptor or exit loudly;
  the test
  `a_commit_service_at_its_process_limit_fails_plainly_and_serves_again_when_it_is_raised` and the
  start-under-limit sweeps already require it,
  and the runtime selection must keep them green.

### Recommendation

Lazy runtime,
built by the first wrapper code that needs it,
never in `main` before the action is planned.
Multi-thread when the budget and the limits allow more than one worker,
current-thread otherwise;
workers from the budget with the new limit inputs;
`MALLOC_ARENA_MAX` capped to a small number before the first worker starts (a number to be measured);
no runtime is ever dropped.

## Libraries in the crate split

The planned libraries and their tokio forms.
The owner has not said how many remain ("I'm not ready to answer this specific question yet."),
so each answer holds for the crate if it exists and for the folder if it is folded into the program.
The stand-alone rule is the owner's of 2026-11-07:
"assume every crate must stand on its own and no one else is going to use our crates to build a git wrapper",
tightened to "Assume no one else is going to use anything related to git from us."

- The rule does not forbid a library that depends on tokio:
  standing on its own means building and testing without the program,
  and a Git-free library has outside users only inside this repository.
  What it implies is that a library must not start a runtime of its own,
  call `block_on`,
  or require a particular
  runtime flavor of its caller,
  because `Runtime::block_on` "panics ... if called within an asynchronous execution
  context" (`src/runtime/runtime.rs`),
  and two crates each building a runtime would count each other's threads against
  one budget.
- `starting-and-watching-programs`:
  async only.
  It takes a `tokio::process::Command` equivalent,
  a no-progress policy and a signal relay handle,
  and returns futures.
  A sync face would be a second implementation of the watcher;
  the program and the service both run on the runtime.
- `parallel-work-within-a-budget`:
  becomes the owner of the budget and of the runtime builder (worker count from the limits,
  the arena cap,
  the
  fallback to current-thread,
  the semaphore).
  The ordered runner (`foundation/parallel_runner.rs`,
  scoped threads,
  `mpsc`) becomes a function over a stream of
  tasks that yields results in order;
  the start-or-run-here fallback becomes the `spawn_blocking` helper.
  It is the only library that names `Runtime`.
- `talking-to-the-systemd-user-manager`:
  async only,
  on `zbus` 6 (the owner's pick);
  the deadline test of the prototype is its control.
- `safe-file-saving`:
  sync only.
  Its calls are `stat`-sized or one `fsync`;
  wrapping each in a future costs more than it saves
  (hop 7 to 17 microseconds against 2 microseconds inline) and puts `spawn_blocking`,
  which can panic when no thread
  starts,
  inside a library that is meant never to fail.
  The program decides which of its calls go to the blocking pool,
  in batches.
- The pure libraries (text merge,
  shell quoting,
  digests,
  time stamps,
  the dependent-version check) stay as they are;
  none waits on anything.
- Libraries with the service in them (`landing`,
  `service_*`) are the program,
  not libraries,
  by the owner's second
  ruling,
  so their async form is decided by this plan,
  not by a library API.

## The order of the move

Every step is one commit or one branch,
is green on the container gate (`mise run //package/git-policy/cli:native:test:container`),
and can be reverted alone.
Nothing here runs before step 0 of the split plan (the release of the last crates.io version of the unsplit crate;
`doc/planning/cli-git-source-layout-target-tree.md`,
section "Move order"),
because the split plan requires that release before anything depends on an unpublished crate.

How it meets the layout moves:
the owner ranked "Layout + linting fixes are the biggest priority" (2026-11-04).
The conversions change function signatures in files that the split renames and moves,
so the plan keeps the
signature changes out of the files in motion:
steps T1 to T3 add files and tests and change no existing signature,
and can start right after the release;
the conversions from T4 on start after the grouping and the `uses/` links are in place (split steps 7 and 8) because
a conversion in the old flat layout is moved by the split with no benefit,
and because the layout lint then guards that
the new runtime owner does not become a dependency of everything.
If the owner prefers tokio first,
T4 to T13 can run on the flat layout and the split rebases over them;
the cost is a
larger rewrite of imports in split step 6.

- T1,
  the first step.
  A source-guard test that fails on any `Command::new` of the current executable,
  on `current_exe` outside
  `invocation/real_git.rs`,
  and (planted control) on a test file that plants such a call.
  This is the owner's "ban that design aggressively" and needs no tokio,
  so it is first.
  Revert:
  delete the test.
- T2.
  Add `tokio` 1.54.1 or newer to the catalog-managed dependencies with the features of the steps that follow and no
  use (`doc/runbook/rust-dependencies.md`:
  add to `rust-catalog.toml`,
  run `file-enforcer write`,
  regenerate the lock
  through the package manager).
  A source-guard test that the binary starts no thread and builds no runtime on a forwarded command (the strace
  expectation of the inventory,
  as a test over `/proc/<pid>/status` or `strace`).
  Revert:
  remove the dependency.
- T3.
  The budget and the runtime builder,
  with no caller:
  the limit inputs of `HostResources` (`RLIMIT_AS`,
  `RLIMIT_NPROC`,
  `pids.max`),
  the arena cap,
  the choice of flavor and
  worker count,
  the lazy `OnceLock`,
  the `spawn_blocking`-with-slot helper,
  and tests under `prlimit` for each limit
  that bites (the cases measured in section "A multi-thread runtime panics,
  then hangs").
  The tests start the built binary under the limits,
  as `binary_fault_limits_tests.rs` does.
  Revert:
  remove the files.
- T4.
  One-shot child waits with no watcher move to the child helper of open choice 7 (standard-library start,
  pidfd,
  `AsyncFd`):
  `process_start.rs`,
  `process_identity.rs`,
  `lock/index_lock_holders.rs` (`ps`,
  `lsof`),
  `repo_read/git_system_config.rs`,
  `invocation/management_service_repository_selection.rs`.
  Those are called from synchronous code;
  the file keeps a synchronous face that enters the runtime lazily,
  until their callers are async in T11.
  Revert:
  git revert of the step.
- T5.
  The watchers:
  `ChildWatch`,
  `Clock`,
  `wait_progress`,
  `quiet_ledger` as one `select!` over the child wait and a timer
  with the marks as atomics or a `watch` channel;
  the watcher thread,
  `recv_timeout` and `Condvar`-style signalling go.
  Control:
  the existing `push/child_watch*_tests.rs` run unchanged against the async form.
- T6.
  `bounded_push`,
  `bounded_query`,
  `start_retry` (sleep becomes `tokio::time::sleep`;
  the retry budget stays 5 s),
  the
  `try_wait` polling loops,
  the sweep thread after a push (`sweep_after_exit`).
- T7.
  The signal relay on a pidfd (section "The signal relay cannot be tokio's signal handling"),
  with the guard wait
  loop in `commit_preparation.rs` (`install_guard_when_free`,
  a sleep-and-retry) replaced by a registry that holds
  every active child,
  so that two concurrent preparations in one process (the service) need no one-at-a-time guard.
  This removes a polling loop and a refusal type;
  it is a design change the owner has not seen (open choice 6).
- T8.
  Service client:
  `UnixStream` connect with the `WouldBlock` retry,
  request and reply with timeouts,
  `client_journal`,
  the unit start through `zbus` (this is the owner's D-Bus ruling and merges with it).
- T9.
  The standards-review call on `hyper` (the owner's ruling,
  prototyped in the research document) with its
  `Marking` quiet-window wrapper,
  the `ureq` removal,
  and the guard test literal.
  It can go anywhere after T3;
  it is listed here because it needs the runtime and no other step needs it.
- T10.
  The service:
  the accept loop to `tokio::net::UnixListener`,
  a task per connection,
  the work loop and the claim renewal
  as tasks,
  the durable-sync thread as a batched `spawn_blocking`,
  the `mpsc` queues as tokio channels or the same
  queues,
  the inotify wait on `rustix` through `AsyncFd`,
  and the `thread_local!` decision of open choice 4.
  This is the largest step;
  it is split by file,
  one commit each,
  with the landing loop last.
- T11.
  Callers:
  the commit command and the policy run become `async fn`,
  the planning enters the runtime,
  the synchronous faces of T4 go.
- T12.
  The parallel runner and the scoped threads (`side_by_side_threads`,
  `scoped_start`,
  `parallel_runner`) become
  tasks and `spawn_blocking` jobs under the semaphore of the round;
  the nine `HostResources::read` call sites read the
  budget through the new owner.
- T13.
  Remove what nothing uses (hand D-Bus client and its fuzz target,
  `std::thread` helpers,
  the poll constants),
  tighten
  the lint (no `std::thread`,
  no `std::process::Command` outside the guarded start,
  no `thread::sleep`,
  as source
  guards),
  and run the mutation campaign and `native:perf` for the whole move.

Windows and macOS code (`#[cfg(not(unix))]`,
`is_process_sender` on macOS) is not converted ("Ignore Win and mac until I
say so.");
the gate's Windows clippy task keeps type-checking it.

## Risks and how a test proves each is handled

- Commits fail or hang when the system refuses threads or address space.
  Proof:
  the `prlimit` cases of T3 (process limit one,
  address-space sizes from the measured hang band,
  descriptor limits)
  against the built binary,
  expecting the same exit and recovery the suite expects today
  (`binary_fault_limits_tests.rs`:
  "the commit fails with a message or lands,
  and the repository recovers either way");
  plus `testing/no_threads_child_test_support.rs` for the `spawn_blocking` helper.
  Planted control:
  remove the limit inputs from the budget and the test must fail (guard test shown to fail,
  per the
  repository's rule).
- A child is started twice,
  or a started child is reported as failed.
  Proof:
  a test with a descriptor limit that makes the reactor registration fail,
  counting Git starts through the recording
  Git of `testing/recording_git.rs`;
  the count must equal the number of successful starts.
- A killed process leaves state that recovery cannot finish ("killed at any instruction").
  Proof:
  the phase seam `CLI_GIT_TEST_ONLY_PHASE_SIGNAL=<phase>:kill` (`transaction/transaction_phase.rs`) and the existing kill
  tests keep running;
  each new `.await` in a journaled sequence is a new instruction boundary,
  so every conversion of a
  landing or journal step adds its phase names to the seam (the list is generated from the step names).
- Slow `fsync` or a hung write delays everything.
  Proof:
  `landing/service_land_fsync_tests.rs` and `landing/service_land_hung_write_tests.rs` run against the async landing;
  a test that holds a blocking job in the pool and checks that a second client is still answered and that the process
  ends by `exit` with the job still running (the runtime is never dropped).
- A signal is relayed twice,
  lost,
  or sent to a reused pid.
  Proof:
  `process/signal_relay_tests.rs` and `binary/binary_fault_signal_tests.rs`,
  plus a new test that signals after the child
  was reaped and expects `ESRCH` and no effect on a process started with the freed pid (a fixture that forces pid
  reuse is needed;
  its feasibility is the first check of T7).
- Thread-affine state is shared across tasks.
  Proof:
  the code does not compile (`!Send`) on a multi-thread runtime,
  which is the check;
  and the landing step's tests run the loop on the chosen thread-affine form.
- The commit is more than 5 s slower than native Git.
  Proof:
  `mise run //package/git-policy/cli:native:perf` (added p95 under 2,000 ms on forwarded commands;
  every paired commit delta
  under 5,000 ms),
  run before T3 and after T10,
  T12 and T13,
  with alternating arms and a same-binary control.
  The latency harness is the gate;
  this plan's own numbers (sub-millisecond) are not.
- The runtime's footprint breaks the budget.
  Proof:
  a test over `/proc/<pid>/status` that a command process has no more threads than the budget says plus the pool's
  idle limit,
  after the runtime has been used and before exit.
- Mutation runs find unguarded new branches (the fallbacks).
  Proof:
  `native:mutation:scoped` over each new file and the planted controls (`native:mutation:planted`) per file,
  as the
  existing campaigns do;
  every fallback branch has a test that only passes when the branch is taken.
- The Markdown guard or the scanner on the blocking pool leaks threads.
  Proof:
  the guard's existing hang test,
  plus a count of pool threads after an abandoned job,
  bounded by `max_blocking_threads`.

## Open choices for the owner

### 1. When the runtime is built

- A.
  Lazy,
  at the first wrapper code that needs it (recommended).
  Pros:
  forwarded commands pay nothing (measured:
  +0.3 to +0.5 ms and about +125 minor faults avoided);
  a command that cannot get threads never builds a multi-thread runtime.
  Cons:
  one `OnceLock` owner and a rule that no code outside it builds a runtime.
- B.
  Eager in `main`.
  Pros:
  simplest to read,
  one place.
  Cons:
  measured cost on every command;
  on a limited system the build can hang (section "A multi-thread runtime
  panics,
  then hangs") before any check could run.
- C.
  Eager only for the service,
  lazy for the command.
  Pros:
  the service's runtime is built at its start.
  Cons:
  the same as A,
  listed because the service's choice is independent;
  A already includes it.
- Ranking:
  A > C > B,
  because A and C cost nothing on the forwarded path while B pays 0.3 to 0.5 ms on every command
  (A over C because C is A plus a statement about the service that A already makes);
  C > B because B cannot be made safe under the address-space limit without the checks that A also needs.

### 2. What "sized to budget" fixes

- A.
  The runtime's worker count comes from the budget with the limit inputs,
  and the concurrency of each round is bounded
  by a semaphore sized from a fresh budget read per round (recommended).
  Pros:
  honors "as many as the computer can spare at that moment" where it matters,
  per round;
  the service does not keep a size chosen hours ago.
  Cons:
  two sizes to explain.
- B.
  Worker count from the budget at build,
  nothing else.
  Pros:
  one rule,
  the owner's words taken literally.
  Cons:
  a service started on a quiet machine keeps that size for hours;
  the owner's "at that moment" holds only at start.
- C.
  A runtime per round for the service.
  Pros:
  the size is fresh each time.
  Cons:
  thread start and stop cost on every round,
  and a dropped runtime waits forever for blocking work.
- Ranking:
  A > B > C,
  because A is B plus the per-round bound that gives the sentence its meaning;
  B > C because C pays a runtime build per round and needs the drop that the documentation says can wait forever.

### 3. The address-space fix

- A.
  Cap malloc arenas (`mallopt`) and read the limits before choosing workers (recommended).
  Pros:
  measured:
  28 workers fit in 100 MiB with the cap and hung in a 900 to 1150 MiB band without it.
  Cons:
  the arena cap may raise allocation contention (to be measured in T3);
  `mallopt` is process-wide.
- B.
  Read the limits only,
  never cap arenas.
  Pros:
  no allocator change.
  Cons:
  needs a larger number of reserved bytes per worker in the sizing;
  one worker costs about 66 MiB of address space.
- C.
  Use the current-thread runtime for the command process and multi-thread only for the service.
  Pros:
  removes the worry for the commonest process.
  Cons:
  contradicts "Full on tokio.
  Multi-thread."
  for the command.
- Ranking:
  A > B > C,
  because A and B respect the owner's multi-thread ruling and A has the smaller footprint;
  B > C because C overrides the owner's ruling.

### 4. The landing loop and thread-affine state

- A.
  A dedicated current-thread runtime with a `LocalSet` for the landing step (recommended).
  Pros:
  ten `thread_local!` items and one `Rc` keep working unchanged;
  the step is serial anyway.
  Cons:
  two runtimes in the service;
  a `LocalSet` task that blocks a call blocks the whole landing runtime,
  which is today's behavior.
- B.
  Turn each thread-local into a task-local or a context value.
  Pros:
  one runtime,
  no thread affinity.
  Cons:
  the largest rewrite of the plan and every test that sets a thread-local fault hook changes.
- C.
  Run the whole landing batch as one `spawn_blocking` job in synchronous code.
  Pros:
  nothing in the step changes.
  Cons:
  the landing step stays outside "Full on tokio",
  and a hung write holds a pool thread for good.
- Ranking:
  A > B > C,
  because A keeps the state and moves the waits;
  B > C because B finishes what C declines,
  at a cost that only pays if landing ever runs concurrently.

### 5. Library API

- A.
  Async for the program-facing I/O libraries and sync for `safe-file-saving` (recommended;
  the section "Libraries in
  the crate split").
  Pros:
  no `block_on` anywhere;
  no runtime started by a library.
  Cons:
  a sync caller of an async library needs its own runtime.
- B.
  Every I/O library offers both through a cargo feature.
  Pros:
  reusable by a sync caller.
  Cons:
  two implementations to keep equal,
  and the owner said no one else will use the libraries.
- C.
  Everything async.
  Pros:
  uniform.
  Cons:
  `safe-file-saving` pays the hop for stat-sized calls and can panic when no thread starts.
- Ranking:
  A > C > B,
  because A matches the measured costs and the owner's no-outside-user rule;
  C > B because C is one implementation that works,
  where B doubles the code for a caller that does not exist.

### 6. The signal registry

- A.
  Replace the one-guard-at-a-time rule with a registry of active children and a pidfd per child (recommended).
  Pros:
  removes a polling loop and a refusal path;
  concurrent preparations in the service are supported.
  Cons:
  changes the documented limit "one guard may be active at a time" and the retry in
  `commit_preparation.rs`;
  needs the pid-reuse fixture.
- B.
  Keep the one-guard rule and wait on a tokio semaphore instead of a sleep loop.
  Pros:
  no semantics change.
  Cons:
  one more lock-like wait,
  against "Use as few locks as possible."
- Ranking:
  A > B,
  because A drops a wait that B only changes;
  the cost of A is the new proof in the risk list.

### 7. How children are started and awaited

- A.
  Standard-library start,
  pidfd,
  `AsyncFd`,
  reap with `try_wait` (recommended).
  Pros:
  a failed start leaves no child (the guarantee of `start_retry.rs` is kept);
  the pidfd is shared with the signal relay;
  the wait is on tokio's reactor.
  Cons:
  about a hundred lines of our own code where tokio has a finished type;
  the fallback where `pidfd_open` is missing is a blocking-pool wait;
  none of it is built.
- B.
  Plain `tokio::process::Command`.
  Pros:
  published code with nothing to maintain;
  the owner's word `tokio::process` taken as written.
  Cons:
  a start can fail after the child has started (read from source,
  not triggered),
  which `start_retry` cannot undo.
- C.
  Plain `tokio::process::Command` with every child started with `kill_on_drop(true)`.
  Pros:
  the stray child dies with the handle that the failed start drops.
  Cons:
  that is a stop by drop,
  not by no progress,
  against "Stop only on no progress";
  the steps that leave a child running (`LeaveRunning`) cannot use it.
- Ranking:
  A > B > C,
  because A keeps the one guarantee the start retry rests on;
  B > C because C adds a kill that the no-progress rule does not allow and does not cover the steps that leave a child
  running.

### 8. When tokio starts relative to the layout moves

- A.
  Add-only steps T1 to T3 after the release,
  conversions after the grouping and links of split steps 7 and 8
  (recommended).
  Pros:
  no conflict with the files in motion.
  Cons:
  the conversions wait for the layout work.
- B.
  All steps on the flat layout first.
  Pros:
  tokio sooner.
  Cons:
  the split's import rewrite grows,
  against the owner's "Layout + linting fixes are the biggest priority".
- C.
  Layout first,
  all of tokio after.
  Pros:
  simplest ordering.
  Cons:
  T1 to T3 are add-only and could land now for no cost.
- Ranking:
  A > C > B,
  because A takes C's safety and adds the steps that cannot conflict;
  C > B because B delays the owner's stated first priority.

## Cross-vendor review

Reviewed by `openai/gpt-6.1-sol` (no tools,
reading only this document);
the findings and what was done are listed in
the section "Review findings and what was done",
written after the review returned.

## Review findings and what was done

Pending at the time of writing.

## Limits

- Every tokio claim is read from tokio 1.54.1;
  no claim was checked against the repository's future lockfile pin.
- The eager-runtime timing is a single scratch repository,
  one commit,
  `rev-parse HEAD`;
  cold wall time was not measured
  (the research document found cold wall times disagree across runs) and page faults stand in for it.
- The child-start comparison (`probe-unix children`:
  standard library 618,
  tokio current-thread 632,
  tokio multi-thread
  with 4 workers 645 microseconds median,
  600 rounds,
  fixed order,
  no same-binary control) is indicative only;
  the
  differences are within what the order could cause.
- The hang under the address-space limit was measured with a probe that builds the runtime before the wrapper's other
  work;
  its cause inside tokio is an inference from the panic site and a stripped stack.
- Not measured:
  `native:perf` (the harness was not run),
  the mutation campaign,
  any real conversion,
  `hyper` and `zbus`
  on `nightly-2026-11-07` (the research document's numbers were made on `nightly-2026-10-28`),
  the cost of `MALLOC_ARENA_MAX` on allocation-heavy work,
  and `tokio_unstable` io_uring file operations
  (a `RUSTFLAGS` cfg,
  not a profile setting,
  but unstable and not proposed).
- The pidfd relay,
  the registry of active children and the standard-library start with a pidfd wait are designs,
  not
  prototypes;
  each has a first check in the step that builds it.
- The failed start after a started child (section "`tokio::process` can report a failed start") was read from source and
  was not triggered.
