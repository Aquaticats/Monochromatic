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
  the other 665 are test files).
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
- Consequence for the design (the cross-vendor review found the first version of this item too confident):
  reading the limits before `build()` lowers the chance of the failure and cannot remove it.
  Other processes use threads and memory between the check and the build;
  `RLIMIT_NPROC` counts the threads of all
  processes of the user,
  not only processes;
  the address-space figure must leave room for what the build and the
  blocking pool allocate after it;
  and cgroup `pids.max` counts every ancestor.
  So the build also needs a bound that does not depend on the check being right:
  the multi-thread runtime is built on a helper thread,
  and the caller waits for it with a deadline;
  when the deadline passes,
  the caller selects the current-thread runtime and leaves the helper thread (it may be hung
  in the unwind) running until the process ends.
  Where not even the helper thread can start,
  the current-thread runtime is selected at once.
  That makes the wait bounded,
  which is "nothing should ever wait forever";
  it does not make the build infallible,
  and nothing in this plan claims it does.
  The inputs of the check are the limits `foundation/parallel_budget.rs` does not read today (`getrlimit` for
  `RLIMIT_AS` and `RLIMIT_NPROC`,
  with the threads of the user counted,
  and the `pids.max` of the cgroup and its
  ancestors),
  and the arena count is capped before the first worker starts (`mallopt(M_ARENA_MAX, ...)`).
  An in-process `mallopt(M_ARENA_MAX, 1)` called before the build gave the same virtual sizes in the probe as the
  environment variable (28 workers:
  +57,936 KiB;
  a blocking thread +2,064 KiB),
  but it was not tried in the wrapper,
  and
  neither its contention cost nor the deadline-bounded build is measured.
  The existing suites already run the wrapper under such limits
  (`binary/binary_fault_limits_tests.rs`:
  descriptor,
  address-space and process limits on the wrapper and on the
  service),
  so a proof harness exists once the runtime is in the binary (open choice 9).

### Blocking-pool work panics, not errors, when no thread can start

- `tokio::task::spawn_blocking` panics with the same message (`pool.rs:385-387`) when the OS refuses a thread and no
  blocking thread exists;
  `tokio::fs` calls it (`src/fs/mod.rs:323-330`,
  `asyncify`),
  so a `tokio::fs` operation panics in that same state
  (with a pool thread alive it queues instead;
  the probe shows the first state only).
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
  The existing child-process harness that refuses every thread (`testing/no_threads_child_test_support.rs`) is the
  control for the helper.
- Limit of that fallback (cross-vendor finding,
  accepted):
  a job run on the calling task cannot be ended by a timer of the same runtime.
  On a current-thread runtime an inline job that hangs stops the timers and the sockets with it,
  and on a multi-thread
  runtime enough inline jobs occupy every worker.
  Today's code has the same property where a thread cannot start (the work runs on the caller),
  so the fallback is no
  worse than the code it replaces,
  but it is not bounded by the in-process watchers either.
  Two rules follow.
  The inline fallback is taken only when the OS refused a thread,
  never when a pool is merely busy
  (a busy pool queues the job,
  and a timeout around its `JoinHandle` stops the wait,
  not the job).
  Calls that can wait on a disk,
  a remote file system or a FUSE mount (`sync_all`,
  large writes,
  metadata of files in
  the repository) are the calls most exposed to it;
  whether they must have an external bound in that state (the service
  manager's watchdog,
  the client's own deadline on the service) is open choice 11.

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
  A pidfd keeps a pid from being reused,
  but not a descriptor number:
  if the handler can read the number while another thread closes the pidfd and the number is reused,
  the handler
  signals something else.
  So the descriptor is closed only after the handler can no longer read it (a slot cleared by an atomic store,
  and the
  close deferred until no handler can be inside it,
  which needs a proof or a different mechanism,
  such as keeping the
  pidfd open until the process ends in the short-lived command process).
  Also an `errno` of the interrupted code must be preserved by the handler.
- tokio's own child reaping does not conflict:
  on Linux `tokio::process` opens a pidfd for each child (`src/process/unix/pidfd_reaper.rs`) and falls back to a
  `SIGCHLD` listener only where `pidfd_open` is missing (`src/process/unix/mod.rs`,
  `build_child`).
  The `SIGCHLD` fallback would install a process-wide handler that is never removed.
  The relay does not touch `SIGCHLD`,
  but that does not make the handler harmless:
  a code path that relies on the inherited `SIGCHLD` disposition (automatic reaping with `SIG_IGN`,
  for example) changes
  behavior.
  The wrapper sets no such disposition today (the search for `sigaction` finds only `process/signal_relay.rs`,
  and it
  handles four other signals);
  whether Git or a hook relies on it is not checked,
  and the fallback only runs where
  `pidfd_open` is missing.

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
- The invariant the helper must keep (cross-vendor finding,
  accepted):
  once the standard-library spawn has succeeded,
  every failure after the spawn (`pidfd_open`,
  the reactor registration,
  a failed
  allocation of the supervision state,
  no thread for a fallback) leaves the child owned by the helper until it is reaped
  or deliberately handed over,
  and none of them is reported as a failed start that may be retried.
  The fallback for a missing or failed pidfd is therefore a timer-driven `try_wait` loop on the helper's own child
  (`tokio::time::interval`,
  no pool thread,
  no descriptor),
  not a blocking-pool wait.
  The cleanup order matters as well:
  the commit preparation must not remove the shadow repository while its Git child is
  alive,
  so the helper's drop ends or waits for the child before the cleanup runs.
  This invariant needs a test seam in the helper (an injected failure at each step after the spawn),
  because the
  reactor failure of `tokio::process` cannot be injected from outside;
  the descriptor-limit sweeps reach
  `pidfd_open` and descriptor creation,
  not the `epoll_ctl` registration itself.
- `kill_on_drop` does not rescue the plain `tokio::process` start:
  the drop guard that carries the flag is built after `imp::build_child(child)?` returns
  (`src/process/mod.rs`,
  `Command::build_child`),
  so a start that fails inside `build_child` has no guard and the
  standard-library child is dropped,
  which neither kills nor waits for it.

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

- tokio has an asynchronous API for `fsync` (`File::sync_all`,
  `sync_data`) but no kernel-asynchronous implementation on
  stable:
  `tokio::fs` runs "ordinary blocking file operations behind the scenes" on the blocking pool,
  and io_uring is used
  only "when compiled with `tokio_unstable`" (`src/fs/mod.rs` module documentation).
  So the fsyncs of the commit path stay blocking calls on a pool thread,
  with the pool's capacity and cancellation
  limits.
  Measured with `probe-unix hop` (`nightly-2026-11-07`,
  release):
  handing an empty job to the blocking pool and back
  costs 7.4 microseconds on a current-thread runtime and 16.6 on a 4-worker multi-thread runtime;
  the first hop,
  which starts the pool thread,
  185 and 83 microseconds;
  `tokio::fs::metadata("/")` takes 17.7 and 10.0 microseconds against 2.0 to 2.4 for the inline call.
  The hop is small against an fsync,
  large against a `stat`.
  The measured `stat` of `/` does not bound any other file call,
  because a metadata call on a repository file can wait on
  a disk,
  a lock or a remote or FUSE mount.
  So the rule is by what a call can wait on,
  not by what one measurement took:
  reads of `/proc`,
  `/sys` and anonymous scratch files may stay inline,
  and calls on repository or user files that can
  wait on storage (`sync_all`,
  large writes,
  directory walks of unbounded size,
  `stat` of user paths in a loop) go to
  `spawn_blocking` in batches,
  as tokio's documentation recommends ("batch your operations into as few `spawn_blocking`
  calls as possible",
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
- Ending the process is not a statement about data (cross-vendor finding,
  accepted).
  Work divides into three kinds,
  and the exit path treats them differently:
  work that may be abandoned (the shadow review,
  the Markdown linter after its no-progress verdict,
  a push that was
  stopped for no progress);
  work whose files and ownership must stay intact until it finishes (a landing batch,
  the journal,
  the claims and the
  index lock marker);
  and durability that must be complete or recoverable before success is reported ("Force at leaving,
  else background").
  The service already waits for the third kind before it ends:
  `serve` calls `crate::service_durable::drain()` before it
  returns in every exit arm (`service/service_server.rs`).
  The tokio form keeps that:
  the durable-sync jobs are tracked (a counter or a `JoinSet` of the pool jobs) and the exit
  path waits for them under the no-progress watcher,
  never under a runtime drop.
  The test that the process ends with a blocking job still running proves bounded termination only;
  the data test is that a job of the second or third kind is never started without being tracked.
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
  On a multi-thread runtime a task can move between workers at each `.await`.
  A future that holds an `Rc` across an `.await` is not `Send`,
  so `tokio::spawn` refuses it at compile time;
  that is a partial check only.
  A task that reads a thread-local,
  drops every reference before the `.await`,
  migrates and reads another worker's copy
  compiles,
  and so do two tasks that interleave on one thread and overwrite state that belonged to one operation.
  So the proof is by tests that migrate and interleave tasks with distinguishable operation state,
  and the test fault
  hooks (seven `#[cfg(test)]` thread-locals) need a positive control that they still reach work that moved to a worker
  or a pool thread.
- The serialized landing step (`landing/service_land.rs`) is serial by design (`doc/planning/cli-git-resident-service.md`,
  "The serialized step,
  group commit,
  FIFO and backpressure"),
  so asynchronous code gives it no concurrency:
  its Git children and fsyncs are in sequence.
  The tokio forms that keep its thread-affine state valid are a `tokio::task::LocalSet` ("Runs `!Send` futures on the
  current thread",
  `src/task/local.rs`) driven on one thread,
  or turning each `thread_local!` into a
  `tokio::task_local!` or an explicit context value (which does not by itself make a task `Send`:
  a task that keeps a
  `!Send` value still cannot be spawned).
  The `LocalSet` needs no second runtime (cross-vendor finding,
  accepted):
  `Runtime::block_on` runs its future on the calling thread,
  so the service's single multi-thread runtime can run the
  landing loop in a `LocalSet` on the thread that called `block_on`,
  while its accept loop,
  connection tasks and
  durable-sync jobs run on the workers.
  That keeps one runtime for the process and the landing loop's thread affinity,
  and is not yet tested in the wrapper.
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
  so a difference smaller than that is not
  resolved by this experiment (which is weaker than "no cost").
- Probe binary,
  runtime code linked and unused:
  5.00,
  5.19 and 4.63.
  (It is within the spread of the control:
  this experiment does not resolve a cost of linking `tokio`.)
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

So an eager multi-thread runtime costs 0.3 to 0.5 ms and about 125 minor faults on every forwarded command.
Against what the wrapper adds over real Git on the same command (4.71 less 2.77,
5.01 less 3.01 and 4.39 less 2.60:
1.8
to 2.0 ms) that is 15 to 28%;
against the whole command (4.4 to 5.0 ms) 6 to 11%;
against the 5 s bound 0.01%.
The 5 s rule is not threatened by it,
but the wrapper's added time on a forwarded command is,
and a forwarded command is
what the wrapper does most (the usage evidence for "most" was not collected here;
the inventory only shows that the
forward arm needs none of the constructs).
These timings are of `rev-parse HEAD` start-up only:
they say nothing of commit latency,
child supervision,
shutdown or
allocator contention,
and the 5 s rule remains unverified until `native:perf` runs (open choice 10).
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
standard library until T11.
At the end state the decision of what to do stays synchronous:
`plan_invocation` runs without a runtime,
and only an
action that is not `Forward` enters the runtime,
so the forwarded fast path never builds one.
A planning step that must spawn then either uses a synchronous bridge or is moved after the decision.
This is the one place where "full on tokio" is read as a goal for the end state,
not for every line at once.
A synchronous bridge that enters the runtime must not panic when called from inside it:
every bridge checks `tokio::runtime::Handle::try_current()` first and returns an error,
and the order of the steps puts
callees before callers (leaf I/O first),
so that a bridge has only synchronous callers for as long as it exists.
`Runtime::block_on` panics "if called within an asynchronous execution context" (`src/runtime/runtime.rs`).

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
  `RLIMIT_AS` less what is already mapped,
  divided by the virtual cost of a worker as measured on this host with this
  glibc (67,608 KiB with arenas,
  2,064 KiB with `MALLOC_ARENA_MAX=1`;
  not constants of tokio),
  `RLIMIT_NPROC` less the threads of the user,
  and the cgroup `pids.max` less the tasks in it and its ancestors.
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
  a `tokio::sync::Semaphore` sized by `worker_budget` bounds how many blocking jobs and children run at once,
  and the
  runtime's workers only poll tasks.
  For the bound to mean anything the semaphore is one object of the process (not one per round),
  shared by the rounds,
  the client work and the durable-sync jobs,
  and a permit is owned by the blocking closure,
  so it is released when the job
  ends and not when the awaiting task times out and stops waiting.
  Whether the semaphore is resized per round (a new object per round bounds only that round) is part of open choice 2.
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
  Whether it implies more is a design proposal here,
  not a consequence of the owner's words:
  that a library does not start a runtime of its own,
  call `block_on`,
  or require a particular runtime flavor of its
  caller,
  because `Runtime::block_on` "panics ... if called within an asynchronous execution
  context" (`src/runtime/runtime.rs`),
  and two crates each building a runtime would count each other's threads against
  one budget.
- `starting-and-watching-programs`:
  async only.
  It takes a `tokio::process::Command` equivalent,
  a no-progress policy and a signal relay handle,
  and returns futures.
  A sync face is possible without a second watcher (a synchronous kernel with an asynchronous scheduling adapter),
  but the
  program and the service both run on the runtime,
  so no sync caller exists to pay for it.
- `parallel-work-within-a-budget`:
  becomes the owner of the budget and exposes the runtime-building policy (worker count from the limits,
  the arena
  cap,
  the deadline-bounded build,
  the fallback to current-thread,
  the semaphore);
  the process still owns and starts the
  one runtime,
  so the library is policy and the program is the owner.
  The ordered runner (`foundation/parallel_runner.rs`,
  scoped threads,
  `mpsc`) becomes a function over a stream of
  tasks that yields results in order;
  the start-or-run-here fallback becomes the `spawn_blocking` helper.
  It is the only library that names `Builder`.
- `talking-to-the-systemd-user-manager`:
  async only,
  on `zbus` 6 (the owner's pick);
  the deadline test of the prototype is its control.
- `safe-file-saving`:
  sync only,
  proposed.
  Its small calls (stat-sized:
  a rename,
  a link,
  a mode change) cost more as futures than as calls (hop 7 to 17
  microseconds against about 2 inline,
  for the measured `stat` only),
  and its `fsync` calls can wait on a disk,
  which the
  metadata measurement says nothing about.
  The library stays synchronous so it never calls `spawn_blocking`,
  which can panic when no thread starts;
  the program
  decides which of its calls go to the blocking pool,
  in batches,
  through the helper that catches that panic.
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
and is reverted newest first:
a step can be reverted alone only while no dependent step depends on its helper or its async
signature,
and a revert of T3 after a consumer has landed does not leave a working program.
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
and can start right after the release (they still
touch shared surfaces:
the catalog,
the lockfile and `HostResources`,
so they are not conflict-free,
only signature-free);
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
  A test that the binary starts no thread and builds no runtime on a forwarded command:
  it counts `clone` and `clone3`
  calls of the process with `strace -f` (the measurement of the inventory),
  because a snapshot of
  `/proc/<pid>/status` cannot show that a thread was never started.
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
  The builder has no production caller yet,
  so the tests reach it through a test-only entry in the built binary (a hidden
  command word or an environment variable read only in test builds,
  as `CLI_GIT_TEST_ONLY_PHASE_SIGNAL` is),
  and start
  that binary under the limits,
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
  the file keeps a synchronous bridge that enters the runtime lazily and refuses
  (an error,
  not a panic) when called from inside it,
  until their callers are async in T11.
  The order of T4 to T12 is by call graph,
  leaf I/O first,
  so that no asynchronous caller reaches a bridge.
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
  the existing `push/child_watch*_tests.rs` are ported to the async API and must keep their assertions
  (they cannot run unchanged against an asynchronous API);
  a planted control per assertion shows each one can fail.
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
  This removes a polling loop and a refusal type,
  and it changes which children a signal reaches (a signal to the
  service would reach every active child unless a recipient rule says otherwise) and needs a handler-safe lifetime for the
  registry;
  it is a design change the owner has not seen and not ranked first (open choice 6).
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
  the commit command and the policy run become `async fn`;
  `plan_invocation` stays synchronous and the runtime is
  entered after it,
  only for actions that are not `Forward`;
  the synchronous bridges of T4 go.
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
  a test of the child helper with a failure injected at each step after the spawn (the `epoll_ctl` registration of plain
  `tokio::process` cannot be reached from outside,
  and a descriptor limit reaches `pidfd_open` and descriptor creation,
  not
  that call),
  counting Git starts through the recording Git of `testing/recording_git.rs`.
  It asserts one Git per logical command,
  that the helper still owns the child,
  that the shadow repository is not removed
  while the child lives,
  and that the child is reaped or handed over.
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
  compilation catches only the futures that hold a `!Send` value across an `.await`;
  tests that migrate and interleave tasks with distinguishable operation state catch the rest;
  the seven test fault hooks get a positive control that they still reach work that moved to another thread;
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
  a test that samples `/proc/<pid>/status` at the end of a command (after the runtime has been used,
  before exit) and
  finds no more threads than the budget says plus the pool's idle limit;
  a snapshot proves the final count,
  not that
  none were ever started.
- A job is abandoned that must not be,
  or a durable write is lost on exit.
  Proof:
  a test that a job of the second or third kind (landing batch,
  journal,
  claim,
  index marker,
  durable-sync) is never
  started untracked,
  that the exit path waits for the tracked durable-sync jobs under the no-progress watcher,
  and that a
  process killed at each exit step leaves a state that recovery finishes.
- The build of the multi-thread runtime hangs.
  Proof:
  the deadline-bounded build under the address-space sizes of the hang band (with and without the arena cap):
  the process
  must select the current-thread runtime within the deadline and finish the command.
  A planted control removes the deadline and the test must fail by timeout.
- The gate accepts a failure that "Commits should never fail."
  rejects.
  Proof:
  the fault suites assert "fails with a message or lands,
  and the repository recovers",
  which is recoverability;
  open choice 10 asks the owner for the stronger assertion (a commit that native Git would complete completes).
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

## The owner's answers at 2026-11-07

Picked in the question tool,
verbatim labels:

1.  When the runtime is built:
    "Lazily,
    on first need (Recommended)".
2.  What "sized to budget" fixes:
    "Build size + job limit (Recommended)".
3.  Address-space protection:
    "Cap pools + read limits (Recommended)".
4.  The landing loop and thread-affine state:
    "Rewrite per-thread state"
    (option C:
    each `thread_local!` becomes a task-local or a context value).
5.  Library API:
    "Everything async,
    and safe-file-saving shouldn't be something we own at all."
    So every I/O library is async,
    and saving files safely is done with published crates,
    not with a crate or module of ours.
6.  The signal relay:
    "Registry of children",
    with the recipient rule
    "Only that operation's children (Recommended)":
    a signal reaches only the children of the operation it belongs to,
    and a stop of the service is handled as an operation of its own.
7.  How children are started and awaited:
    "std start + pidfd (Recommended)".
8.  When tokio starts relative to the layout moves:
    "All tokio now".
    The conversions run on today's layout,
    before the layout moves.
9.  A bound on the runtime build:
    "Helper thread + deadline (Recommended)".
10. What "Commits should never fail."
    means for the gate:
    "Strong:
    what Git completes,
    we complete (Recommended)";
    each fault case gets a native-Git control.
11. Calls that can wait on storage when no thread can start:
    "Fail plainly".
    No inline fallback;
    the command fails with a plain message in that state,
    which the owner chose over option A.
12. What a stop of the service does to the Git child of a commit it is preparing
    (answered 2026-11-07T22:19:43-05:00):
    "C:
    shutdown stops its Gits (Recommended)",
    built in T11 (see open choice 12).

## Progress on the branch `feat/git-policy-cli-on-tokio`

Written:
2026-11-07T07:00:00-05:00.
The branch is not merged into main (the coordinator holds that until the wrapper's release is out).
The order follows the owner's answers:
the conversions run on today's layout (choice 8),
so no step waits for the layout moves.

### Step T1, the self-invocation ban: already in force, audited

- The three clauses of the step are covered by `source_guard/no_self_invocation_tests.rs` and its rules file:
  rule `own-executable` (`current_exe` and `/proc/self/exe` outside the excused `invocation/real_git.rs`),
  rule `spawn-own-executable` (a `Command::new(` whose argument names the wrapper's own path),
  and the planted controls (`each_rule_fires_on_its_own_forbidden_text`,
  and the fixture tests that plant a violation in a
  nested production file and in a file with a test-support name).
- `clippy.toml` also denies `std::env::current_exe`.
- `tokio::process::Command::new(` would match the same rule text,
  and the child helper of step T4 starts children with the
  standard library,
  so no new spelling needs a new rule.
- No code commit was needed.
  The broader lint of step T13 (no `std::process::Command` outside the guarded start) stays at T13.

### Step T2, tokio in the manifest and the no-clone guard

- `9d9fc7ec37`:
  `binary/binary_forwarded_command_makes_no_thread_tests.rs` runs a forwarded `rev-parse HEAD` and `status --short` under
  `strace` and counts the `clone`-family calls of the wrapper's own program (`wrapper_spawns` in `binary_budget_trace.rs`).
  It is green before tokio is linked,
  so it is a guard,
  not a red test.
  The planted control is `the_count_sees_threads_and_children_made_by_the_wrapper_and_only_those`,
  a trace in which the wrapper
  starts a thread and a child,
  which the counter must count (and the harness's `timeout` and Git must not be).
- `tokio` joined the manifest with the features `rt`,
  `rt-multi-thread`,
  `net`,
  `time`,
  `fs`,
  `io-util`,
  `sync` and `macros`.
  `process` and `signal` stay off by design (choices 6 and 7).
  `rustix` (feature `process`) joined for `getrlimit` and,
  in the later steps,
  the pidfd and the inotify watch.
  The dependency landed together with its first use in step T3,
  because an unused dependency is a cargo warning.

### Step T3, the budget, the limits and the lazy runtime

- Files (all under `src/native/foundation/`,
  each with a sidecar test file):
  `runtime_limits.rs` (address space left under `RLIMIT_AS`,
  threads left under `RLIMIT_NPROC` and under the `pids.max` of the
  control group and every group above it),
  `runtime_plan.rs` (workers from the budget,
  bounded by the limits;
  current-thread when fewer than two fit),
  `lazy_runtime.rs` (one `OnceLock` owner,
  malloc arenas capped to one before the first worker,
  the multi-thread build on a helper
  thread with a deadline,
  the current-thread runtime as the stand-in,
  a nested call refused with an error),
  `blocking_job.rs` (a blocking call on the pool;
  "the system refused a thread" is the error `NoThreadToStart`,
  never an inline run),
  `runtime_probe.rs` (the test-only entry,
  `CLI_GIT_TEST_ONLY_RUNTIME_PROBE`,
  for the controls that start the built executable).
- `invocation/panic_notice.rs` gained `while_panics_are_expected`:
  tokio panics,
  instead of returning an error,
  when a thread is refused;
  the callers catch that panic and report their own plain
  error,
  so the hook's "report this location" notice (which would tell a person of a defect on a command that went on to succeed)
  is silenced for the panics of those two calls only.
- Values chosen under the owner's answers,
  each open to the owner's veto and each cited beside the value in the code:
  deadline `BUILD_DEADLINE` one second (a working build is well under a few milliseconds);
  `ADDRESS_SPACE_RESERVE_BYTES` 64 MiB (three times the 20 MiB the whole process mapped);
  `THREAD_VIRTUAL_BYTES` the 2 MiB stack plus 64 KiB (measured 2,064 KiB a blocking thread,
  2,069 KiB a worker with one arena);
  `MALLOC_ARENA_LIMIT` 1.
- Departures from the text of this plan:
  the limits are a value of their own (`RuntimeLimits`) beside `HostResources`,
  not new fields of it,
  so the nine call sites and
  every test that builds a `HostResources` by hand stay unchanged;
  the user's threads are bounded by one read of the system's thread total in `/proc/loadavg`,
  an upper bound that only lowers the
  worker count,
  instead of a scan of every process;
  the arena cap is always applied,
  as choice 3 says,
  not only under an address-space limit.
- Controls (`binary/binary_runtime_build_under_limits_tests.rs`) start the built executable with the probe under a process limit
  of one and under address-space limits from 48 MiB to 1500 MiB (the measured hang band included),
  each with real Git started
  under the same limit as the native control (choice 10).
  Planted controls,
  run by hand and reverted:
  removing the limit inputs from the plan failed both the process-limit control (flavor `current-thread-after-failure`) and the
  address-space control (a stand-in after a deadline at 48 MiB);
  the unit control `a_build_that_outlasts_the_deadline_is_replaced_by_the_current_thread_runtime` fails if the wait loses its deadline.
- Finding,
  not the runtime's:
  below 48 MiB of address space the debug build of the wrapper prints nothing,
  with or without the probe,
  while real Git
  completes at 24 and 32 MiB.
  The sweep starts at 48 MiB for that reason.
  Whether the release build has the same floor was not measured.

### The published-crate swaps

- Digests (`47219dc182` red,
  `1aea0ae238` green):
  `subvolume/subvolume_sha.rs` is two calls into `sha1` 0.11.0 and `sha2` 0.11.1;
  the hand-written compression functions are gone.
  The published test vectors,
  the reference comparison at every length to a few blocks and the 512 MiB comparison still pass.
- Time stamps (`7474c396d2` red,
  `c4a834d4da` green):
  `foundation/iso_time.rs` formats through `jiff` and `conflict/discarded_prune_stamps.rs` parses through `jiff`;
  the hand-written calendar (`civil_from_days`,
  `days_from_civil`) is gone.
  The text shape is unchanged (`YYYY-MM-DDTHH:MM:SS.mmmZ`).
  An instant past the end of year 9999 is clamped to the crate's last instant (`9999-12-30T22:00:00.999Z`),
  where the old code
  printed a meaningless date.
  `jiff` was first kept at the already locked 0.2.24:
  the releases from 0.2.32 name `defmt` 1.x as an optional dependency,
  which
  locks `defmt-parser` 1.1.0,
  and the trust policy refused that release (published by account 414585 where 0.4.1 was published
  by account 84779).
  The owner,
  2026-11-07,
  verbatim:
  "I'm approving the publisher change for jiff."
  `rust-catalog.toml` now lists `defmt-parser@1.1.0` in `trustPolicyExclude` (that one release,
  not the crate,
  with the sentence
  beside it),
  `jiff` moved to 0.2.39 through `cargo update -p jiff` under the shadow (seven versions newly locked,
  all checked:
  `bitflags` 1.3.2,
  `defmt-macros` 1.2.0,
  `defmt-parser` 1.1.0,
  `defmt` 1.2.0,
  `jiff-core` 0.1.2,
  `jiff-static` 0.2.39,
  `jiff` 0.2.39),
  and the catalog floor is raised to 0.2.39.
  The guard is `the_lock_no_longer_holds_jiff_at_the_release_kept_while_the_trust_policy_refused_the_newer_ones` in
  `foundation/iso_time_tests.rs`.
  With the exclusion in place the daily dependency update has no refusal to trip on at this path;
  before it,
  the shadow's update
  rule would have kept nothing:
  it sets a refused release aside for an older one in the same caret class and,
  when that fails,
  restores every lockfile of the run (`package/cargo-shadow/cli/README.md`,
  the `cargo update` line of the flag table).
- Memory headroom (`8955195670` red,
  `0a59ee9272` green):
  `HostResources::read` takes the available memory and the control-group headroom from `sysinfo` 0.40.1 (feature `system` only);
  the readers of `/proc/meminfo`,
  `memory.max` and `memory.current` and their tests are gone.
  Locking `sysinfo` needed the crates of its Windows and macOS targets in the local registry cache;
  the shadow's offline install
  could not download them,
  so the lock was first completed with the toolchain's own `cargo fetch`,
  and then
  `mise run //package/git-policy/cli:native:lock:members` checked the 17 newly locked versions against the release-age and trust policy (all passed).
- Linux calls (`rustix`):
  started with `getrlimit` and the pidfd;
  the file calls move with the safe-file-saving replacement in the `Safe file saving` section.

### Step T4, children without a blocking wait (partly)

- `process/child_awaited_through_a_pidfd.rs`:
  the helper of choice 7.
  `adopt` cannot fail;
  a failed `pidfd_open` or a failed registration only changes the wait to a timer-driven `try_wait`.
  Injected failures (`InjectedFailure`) prove the invariant:
  after each,
  the live child is still owned,
  reapable,
  and ended by a drop;
  `into_std` hands it over.
- `process/captured_child_run_to_its_end.rs`:
  `output_of` runs a command to its end like `Command::output`,
  with the output in anonymous files (no reader,
  no pipe).
  Its blocking face `output_of_blocking` refuses with an error,
  not a panic,
  from inside the runtime.
- The two Linux one-shot callers use it (`repo_read/git_system_config.rs`,
  `invocation/management_service_repository_selection.rs`).
  The other one-shot waits named in the step (`process/process_start.rs`,
  `process/process_identity.rs`,
  `lock/index_lock_holders.rs`)
  start `ps`,
  `lsof` and PowerShell only on macOS and Windows,
  which the owner said to ignore;
  they were not converted.
- Not yet done in T4:
  `AsyncFd` registration is exercised only through `adopt`;
  the descriptor-limit sweeps of the plan's risk list for the helper are not written.

### Safe file saving

The vet report is `doc/audit/tech-safe-file-saving-published-crates-vet-2026-11-07.md`.
Summary:
`tempfile` (`persist_noclobber`) for the whole publish,
`tokio::fs` for create-new,
no-follow reads and private folders,
`rustix::fs::fsync` for the directory sync.
The one behavior no crate gives is the fallback to a checked rename on a filesystem that supports neither `renameat2`
with `RENAME_NOREPLACE` nor hard links (evidence:
`tempfile-3.27.0/src/file/imp/unix.rs`,
function `persist`,
lines 94 to 133).
`openat2` with `RESOLVE_NO_SYMLINKS` was measured and is stricter than the wrapper's `O_NOFOLLOW` (it refused every path on this host because the scratch folder sits behind a link),
so it is not adopted for reads.
The replacement lands with the conversion of the callers (the 38 files that use `foundation/private_storage.rs`),
not before.
The coordinator asked for facts on the one item in conflict with "Commits should never fail" (G1);
they are in the vet report,
section "Facts for the owner's answer on G1".
In short:
no in-kernel filesystem read lacks both `RENAME_NOREPLACE` and hard links (FAT and exFAT have no links but accept the flag;
NFS rejects the flag but has links);
FUSE servers such as `s3fs-fuse` lack both;
with `tempfile` only,
a plain commit still lands
(only the post-commit work item is lost),
but a commit during or concluding an unfinished merge fails with
"writing packed-refs failed:
Operation not supported (os error 95)" or
"copying a conclusion entry into the shadow failed:
Operation not supported (os error 95)";
the fallback is ten code lines.
The owner answered G1,
2026-11-07,
in the question tool.
The question said:
"Commits keep working on those disks.
We own 10 lines plus a 15-line test.
On those disks only,
two programs
saving the same file at the same instant could overwrite one another."
The owner picked,
verbatim:
"Keep the fallback (Recommended)".
So the replacement of `private_storage.rs` is built on `tempfile::persist_noclobber`,
and the checked rename stays as the fallback
for that one case only:
the publish answered by neither `renameat2` with `RENAME_NOREPLACE` nor a hard link.

### Safe file saving: the publish replaced, 2026-11-07T07:55:00-05:00

- The owner's answer on G1 is recorded in the Safe file saving section,
  verbatim:
  "Keep the fallback (Recommended)".
- `1cff4935cd` records it;
  `bcaf8579e4` is the red test of the defect the previous agent found
  (`binary/binary_fault_trace_tests.rs`,
  `a_filesystem_that_refuses_hard_links_does_not_stop_a_commit`:
  every `link` and `linkat`
  of the service and of each Git it starts answers `EOPNOTSUPP`;
  real Git lands the same commit under the same fault as the native
  control,
  the wrapper fails with "The commit service could not finish this commit,
  so it was not made.");
  `3ec166d146` is the guard red for the module;
  `2a503fc9e1` is the replacement;
  `baf34e09d3` makes `decide` in
  `service/service_busy.rs` publish its decision file through the same exclusive publish,
  which turns the first red green.
- What changed in `foundation/private_storage.rs`:
  the staging file is `tempfile::Builder` (prefix `.<name>.`,
  suffix `.partial`),
  written,
  synced when durable,
  chmod to 0600,
  then `persist_noclobber`;
  `publish_by_checked_rename` and
  `settle_exclusive_publish` keep the owner's fallback for any refusal that is not `AlreadyExists`;
  the directory sync is
  `rustix::fs::fsync` (feature `fs` added to the manifest,
  no new locked package).
  Gone:
  the hand-built staging name,
  the link-then-rename order,
  `staging_removal_report`,
  the uuid use.
  The controls:
  `a_filesystem_with_neither_no_replace_rename_nor_links_gets_the_checked_rename`,
  `the_checked_rename_refuses_a_taken_name_and_leaves_no_staging_file` (planted control:
  the fallback replaced by an error failed
  both),
  `a_taken_name_is_refused_with_the_operating_systems_error`,
  `a_published_record_is_private_and_a_failed_publish_leaves_nothing`.
- Not done,
  by design:
  the public faces of the module stay synchronous (the 38 callers become asynchronous in T11),
  so the reads
  (`O_NOFOLLOW`),
  the exclusive scratch create and the private folders still use the standard library;
  their move to `tokio::fs`
  lands with the callers,
  as the vet report's ledger says.
  Other `hard_link` users outside this module
  (`binary/binary_wrapper_store.rs`,
  `conflict/discard_audit*.rs`,
  `lock/single_owner_claim.rs`,
  `lock/stale_ref_lock_retire.rs`,
  `landing/service_index_placement.rs`) were not part of the step and were not changed.
- Two facts for the owner,
  not decided here:
  1. On a filesystem with neither no-replace rename nor links,
     `decide` is a check then rename.
     Its racers are the landing thread and
     a connection thread of one service,
     not two programs,
     so both can see themselves as the winner:
     a client could be told its
     request was withdrawn while the commit lands.
     The owner's question described the race as two programs saving one file;
     it
     did not say this case.
  2. A commit through the service on such a filesystem still fails for another reason:
     with `link`,
     `linkat` and `renameat2` all
     refused (injected under `strace`),
     the service answers "cannot take the index lock ...:
     the btrfs filesystem mounted at ... can
     neither rename a file without replacing another nor make a hard link ... so the lock cannot be created with its marker in one
     step,
     and the service takes no lock that a kill could leave empty",
     and the commit fails;
     real Git,
     under the same injection,
     lands the commit.
     That refusal is a deliberate earlier design in `landing/service_index_placement.rs`.
     Whether it should give way
     to "Commits should never fail" (and what a kill could then leave) needs the owner.
     The state directory is the subject of the
     fix above;
     the repository's Git directory on such a filesystem is this second case.
- Settled by the coordinator,
  2026-11-07T09:12:04-05:00,
  from owner rules already given (no new owner question):
  1. The `decide` race is inside one process,
     so it closes with an in-memory atomic claim (compare-and-swap on the decision's
     state) taken before the check-then-rename;
     no lock is added ("Use as few locks as possible.";
     `CLAUDE.md` rule NLK,
     atomic
     first).
     The cross-process case stays the checked rename the owner kept ("Keep the fallback (Recommended)").
  2. The index lock on a filesystem with neither no-replace rename nor hard links falls back to the way Git itself takes it:
     Git 2.56.0 creates `index.lock` with `open(..., O_RDWR | O_CREAT | O_EXCL ...)` (`tempfile.c` line 142,
     called from
     `lock_file` in `lockfile.c` lines 168 to 190) and needs neither call.
     Reason:
     "Commits should never fail."
     and the owner's
     G1 pick to support such filesystems;
     the fallback is never less safe than plain Git on the same filesystem,
     which commits
     there today while the wrapper refuses.
     What a kill can leave:
     a lock without its marker,
     as plain Git leaves one
     (`unable_to_lock_message`,
     `lockfile.c` lines 254 to 267,
     tells the user to remove it).
     The wrapper never removes such a
     lock unless the holder is proven gone,
     and its message names the file and the fix.
     Clearing it by itself needs a proof
     that holds on that filesystem (for example,
     that file locks are honored there);
     that is a measurement for the agent,
     and
     only a proven mechanism ships.
     Sources fetched to `~/temp/agent/git-prompt-src/lockfile-v2.56.0.c` and
     `tempfile-v2.56.0.c` (tag `v2.56.0`).
- The index lock without links,
  branch `fix/git-policy-cli-index-lock-without-links`:
  item 2 of the coordinator's settlements is built.
  `907416dba7` is the red test
  (`a_filesystem_with_neither_no_replace_rename_nor_links_does_not_stop_a_commit`:
  `EOPNOTSUPP` on `link`,
  `linkat` and `renameat2`,
  real Git
  as the native control);
  `place_lock` in `landing/service_index_placement.rs` takes `index.lock` by exclusive creation only when both the
  rename that does not replace and the hard link were refused,
  writes and flushes the marker,
  and hands the created file's descriptor to the
  hold (`Placed::Created`),
  because the ownership check compares the lock's inode.
  The claim chain keeps `place_marked` (a claim must be placed
  whole).
  Self-clearing is not shipped:
  a file lock cannot tell a killed service's empty lock from a live Git's,
  since the lock file must
  exist before it can be locked and Git never locks it;
  the empty lock stays,
  and the ceiling's words name the file and the fix.
- Merged into main as `5e441f25d3` (2026-11-07,
  coordinator),
  with steps T1 to T4 and the publish swap;
  main's only wrapper
  changes since the branch's last main merge were rename step 3's README and two TypeScript test files.

### Step T4, the rest: descriptor-limit sweeps

- `b028cc43c0`,
  `0e51e63ae4`:
  `process/child_awaited_through_a_pidfd_descriptor_limit_tests.rs` lowers `RLIMIT_NOFILE` of the test's
  own process (nextest gives each test a process) in three sweeps:
  a limit that leaves no descriptor for `pidfd_open` changes the
  wait to the timer and nothing else (with one free the pidfd is used,
  which proves the limit reached the call);
  a refused start
  is `EMFILE` and leaves no child;
  a start that worked is awaited to its status;
  the captured run is whole or the plain `EMFILE`
  error;
  in every case the descriptor count returns to its baseline and `waitpid(-1)` finds no child.
  Planted control:
  `pidfd_open` made to `expect` failed the first sweep.
  The sweep of the captured run cannot reach a failing
  `pidfd_open` (a start needs more descriptors than the pidfd,
  so a start that worked leaves room),
  so it does not fail under that
  plant;
  it guards leaks and the error kind.
- `OwnedChild::waits_on_a_pidfd` was added for these controls.

### The `decide` race closed, 2026-11-07T09:47:00-05:00

- Red `4d372fdd80`:
  `on_a_disk_without_exclusive_publish_the_landing_thread_and_a_connection_thread_never_both_win_a_decision`
  (`service/service_busy_tests.rs`) feeds every publish of two threads the refusal of a filesystem with neither no-replace rename nor
  links (test-only seams in `foundation/private_storage.rs`:
  `THE_FILESYSTEM_ANSWERS_NEITHER_EXCLUSIVE_PUBLISH` and
  `BETWEEN_THE_CHECK_AND_THE_RENAME`),
  pauses the landing thread between the check and the rename,
  and decides the same request from a
  connection thread.
  It failed with "both threads saw themselves the winner;
  the file says `landing`".
- Green `7f39a05469`:
  `foundation/name_held_while_one_thread_publishes_it.rs` holds the name in a fixed table of atomic flags (a hash
  of the path picks the slot) from before the check until the rename is done,
  and `publish_by_checked_rename` takes it.
  `aeb2948629`
  corrects the wording.
- Deviation from the coordinator's wording ("compare-and-swap on the decision's state ... no lock is added"),
  for the coordinator to accept
  or send back:
  the claim is taken by compare-and-swap,
  but a thread that finds it taken waits for the holder (a bounded yield-then-sleep
  loop,
  five seconds,
  `HELD_NAME_WAIT` in `private_storage.rs`,
  then `TimedOut` and nothing published).
  It is not lock-free for that
  thread,
  so the module and the code say so.
  The wait exists so that the loser can read the winner's file whole afterwards;
  a loser that
  learned from the flag alone could read the file before the rename.
  A one-shot claim that carries the outcome has an open obstacle:
  two
  names sharing a slot.
  `source_guard/service_lock_inventory_tests.rs` names mutexes,
  read-write locks,
  condition variables and `flock`,
  so the flag is not an entry there.
  The wait adds a `thread::sleep` that step T13's source guard must either except by path or this module
  must lose by moving to a futex wait (`rustix::thread::futex`).
- Placement:
  the claim is in the checked rename of `private_storage.rs`,
  so it covers every publisher in the process,
  not only `decide`.
- `HELD_NAME_WAIT` (five seconds) has no measurement beside it;
  it is the start retry's budget,
  open to the owner's veto.
- Gate:
  host lib tests 4432 of 4432;
  Clippy and the package lint clean;
  `a_filesystem_that_refuses_hard_links_does_not_stop_a_commit`
  passes;
  container gate 5133 run,
  5133 passed (1 slow),
  8 skipped,
  Clippy clean (snapshot at `aeb2948629`).

### Step T5, the child watcher as a task, 2026-11-07T09:47:27-05:00

- `fa518c33a1` red,
  `7e4dc3aefc` green:
  a budget of one worker now builds a multi-thread runtime of one worker;
  the current-thread runtime
  is the plan only where a limit leaves no thread.
  Reason:
  the watcher of a wait that blocks inside a read or a wait is a task that must
  run beside the blocked thread,
  and a current-thread runtime runs tasks only while some thread calls `block_on`.
  This changes the T3
  plan text "current-thread when fewer than two fit";
  the owner said "Full on tokio.
  Multi-thread."
  and nothing about one worker.
- `c5b4559b88`:
  `ChildWatch` keeps its synchronous face (`start`,
  `start_with_locks`,
  `begin`,
  `end`,
  `expired`,
  `never_stop`,
  `within`,
  `kept_as`,
  `ceiling_hit`) so its callers (`transaction/transaction_git.rs`,
  `candidate/candidate_reader.rs`) and the 24 controls in
  `push/child_watch_tests.rs` are unchanged and keep every assertion.
  The watcher is a task started by the new
  `foundation/lazy_runtime.rs` `spawn_on_runtime` (a task on the runtime's workers;
  nothing where the runtime has none,
  as before when the
  watcher thread could not start),
  fed by an unbounded tokio channel;
  the mark receiver,
  the watcher thread,
  `recv_timeout` in the
  loop and the `thread::sleep` loops of `stop_process` are gone.
  The notice is written on the blocking pool (a terminal that does not take
  a line must not hold a worker),
  and the lock sweep of a stopped step runs on the blocking pool;
  where no thread can start the sweep is
  not run and is logged (owner,
  11,
  "Fail plainly"),
  which leaves such a step's `.lock` files for the next Git command to name.
- Still in the file after T5,
  deliberately:
  dropping a `ChildWatch` waits for its task on a standard-library channel with a five-second bound
  (`WATCHER_END_BOUND`,
  the start retry's budget,
  open to the owner's veto);
  the commit message says "channel timeouts are gone",
  which
  is wrong for that one.
  `output_watched_within` still waits for its child with the standard library,
  and its `LeaveRunning` waiter is
  still a thread.
- Not done in T5,
  and why:
  the one-shot `output_watched_within` on the pidfd helper.
  Its `EndAt` form reads the child's output through
  pipes with `wait_with_output` and returns all of it;
  the pidfd helper reads anonymous files through `Capture::snapshot`,
  which keeps
  only the last `STEP_OUTPUT_LIMIT` bytes and appends notes.
  Moving `EndAt` steps (the commit preparation's Git steps) onto files would
  change what a large output returns,
  so it needs an owner-visible decision or an `AsyncFd` pipe reader,
  not a silent swap.
- Planted control,
  run by hand and reverted:
  `spawn_on_runtime` made to drop the task and return nothing (the watch watches nothing)
  failed (six) or timed out (three) among the 18 controls of `push/child_watch_tests.rs` that ran before nextest cancelled the run,
  including (`a_wait_past_the_bound_is_announced_then_the_child_is_stopped`,
  `a_command_past_the_bound_is_stopped_and_reported`,
  `a_silent_child_is_ended_after_the_quiet_window_and_a_sibling_is_left_alone`,
  `a_command_past_the_bound_is_stopped_with_the_processes_it_started`,
  `a_wait_that_has_ended_is_announced_no_more`,
  and the three ceiling
  controls,
  which waited out nextest's 300 second timeout).
- New control:
  `a_spawned_task_runs_while_the_caller_is_blocked` (`foundation/lazy_runtime_tests.rs`).
- Gate:
  host lib tests 4433 of 4433;
  all 700 host binary tests passed (one earlier run stopped at the first failure,
  `service::the_settle_command_names_a_batch_it_could_not_examine_and_exits_nonzero`,
  while a container gate was compiling beside it;
  it
  passed alone three times and in the full run that followed;
  no cause was measured,
  so it is an unexplained one-off);
  Clippy and the
  package lint clean.
  Container gate (snapshot at `c5b4559b88`):
  5134 run,
  5134 passed (2 slow),
  8 skipped,
  Clippy clean.
- Main merged again at `d628f74005` (the fold of the earlier data names `38d54f9b2b`,
  the verbose masking `870962b7f2`):
  no conflict,
  nothing of
  this branch touches `landing/earlier_service_process_look.rs`.
  After the merge:
  host lib tests 4464 of 4464,
  package lint clean;
  the container
  gate of the merged tree ran as the user unit `tokio-gate-merge2` (log `${HOME}/temp/agent/tokio-steps/gate-merge2.log`):
  5165 tests run,
  5165 passed (1 slow),
  8 skipped,
  Clippy clean.
  The coordinator merged that tree into main as `48611f7e09`.
- Files T5 changed,
  for the mutation campaign:
  `src/native/foundation/lazy_runtime.rs`,
  `src/native/foundation/runtime_plan.rs`,
  `src/native/push/child_watch.rs`,
  `src/native/push/child_watch_clock_marks_and_stopping.rs`;
  and for the decide fix
  `src/native/foundation/name_held_while_one_thread_publishes_it.rs` and `src/native/foundation/private_storage.rs`.

### The coordinator's three settlements after T5, 2026-11-07T10:20:00-05:00 to 2026-11-07T11:10:00-05:00

Each is a red commit,
then a green commit;
none is a new owner question.

- (a) The T5 one-shot form on an `AsyncFd` pipe reader that keeps all the output.
  Red `8c1e3a2c2b`,
  green `79b21d6382` (Clippy fix
  `3c707c0b04`).
  `process/step_output_read_whole_through_pipes.rs`:
  the two pipes are made with `std::io::pipe` and their read ends are
  registered with the reactor (`tokio::net::unix::pipe::Receiver::from_owned_fd`,
  which sets them non-blocking) before the child starts,
  so a refusal (a descriptor limit,
  a call from inside the runtime) starts no child;
  after the start the command's write ends are
  replaced (`release_write_ends`),
  because a write end held by this process keeps the readers from ever seeing the end of the output.
  `read_until_the_child_ends` joins both reads to their end with the pidfd wait of `OwnedChild`,
  as `wait_with_output` does (a child
  that fills a pipe is never waited for first),
  closes the child's input first as `wait_with_output` does,
  and ends the child when a read
  fails.
  `output_watched_within` uses it for `StepBound::EndAt`;
  `wait_with_output` is gone from `push/child_watch.rs`.
  Not changed,
  by
  design of the settlement:
  the `Capture` files of `LeaveRunning` and `Unbounded`,
  the `Unbounded` blocking `wait`,
  and the `LeaveRunning`
  waiter thread (leaving a child running past a command needs a reaper that does not end the child when dropped,
  which the one-shot
  settlement did not ask for).
  Controls (`push/child_watch_one_shot_through_pipes_tests.rs`):
  both streams of more than the kept tail come
  back whole (planted control,
  run by hand and reverted:
  the read truncated to `STEP_OUTPUT_LIMIT` failed it),
  a call from inside the
  runtime is refused with no child started (red before),
  no descriptor stays open after a run,
  and a source guard that `wait_with_output`
  is gone (red before).
  The 24 controls of `push/child_watch_tests.rs` pass unchanged (28 of 28 with the new four).
- (b) The name claim's wait is a wake on the flag.
  Red `b171ba0d9e`,
  green `d60cf655ca`.
  `foundation/name_held_while_one_thread_publishes_it.rs`:
  each slot is an `AtomicU32` (free,
  held,
  held with a sleeper);
  a thread that finds the name held marks a sleeper and sleeps on the word
  with `rustix::thread::futex::wait` (private,
  with the time left of the bound);
  the release swaps the word to free and wakes one sleeper
  if one is marked.
  `yield_now` and `thread::sleep` are gone from the module (a source control guards it),
  so the T13 guard needs no
  exception for it.
  The first version took the name after a sleep with a compare-and-swap to plain held,
  which loses the wake of the
  other sleepers;
  two of the new controls failed by hanging (60 and 90 seconds) and caught it,
  so a thread that has slept takes the name
  with the word still marked.
  Controls added:
  the bound holds with no one to wake the sleeper,
  and no wake is lost among eight threads
  taking and giving back one name three hundred times.
  The `thread` feature of `rustix` was added to `Cargo.toml` (no new locked package;
  `sync:files` wrote nothing).
  Where the platform has no futex (macOS,
  Windows) the sleeper parks for a millisecond slice and looks again;
  the crate does not build for Windows (README,
  Requirements) and the owner said to ignore both.
- (c) `HELD_NAME_WAIT` and `WATCHER_END_BOUND`,
  measured.
  Measurement and red controls `e15dd2bfb5`,
  values `9dee521668`.
  Both measurements are
  `#[ignore]` tests run by hand (`--run-ignored only --no-capture measure_how_long`),
  on this host's btrfs under the load there was
  (the logs are `${HOME}/temp/agent/tokio-steps/c-measure1.log` to `c-measure3.log` and the earlier run that stalled):
  - A name's hold (claim,
    check,
    rename):
    20,000 publishes by one thread under a load average of about 70:
    median 181 us,
    99.9th percentile
    312143 us,
    longest 6031986 us (the 120 second watchdog then ended that run before its crowd part).
    Three runs of 4,000 under a load
    average of 15 to 22:
    medians 24 to 25 us,
    longest 93,
    179 and 296227 us.
    Eight threads at once (2,000 publishes):
    median 143 to 156 us,
    longest 497 to 1126 us.
    So a hold of 6.03 s of wall time happened (the cause is not measured:
    the run was inside a scope capped at 1600% of the processors while the load average was about 70,
    so a throttled or descheduled thread gives the same figure as a stalled disk;
    a later run of 4,000 in a scope capped at 2400%,
    load average about 20,
    had a longest of 94 us,
    which excludes neither),
    and the previous five seconds was below it:
    a thread that waited for that holder
    would have given up and published nothing.
    `HELD_NAME_WAIT` is now `LONGEST_HOLD_SEEN` times `HOLD_MARGIN` (ten,
    because the tail moved
    by a factor of twenty between runs),
    60.3 seconds.
    No filesystem with neither exclusive publish was available (an S3 bucket through
    `s3fs-fuse`);
    a measurement there can only raise the figure.
    The factor ten is the agent's choice,
    open to the owner's veto.
  - Dropping a watch:
    between waits median 28 to 32 us and longest 505 us;
    in the middle of a wait median 72 to 94 us and longest 389 us;
    the lock sweep alone in a repository of forty branches median 309 to 340 us and longest 569 us;
    a whole stop of a child that answers the
    request,
    past the window until the call returns:
    median 30.6 to 37.2 ms,
    longest 41680 us.
    The task can legitimately outlast these only
    inside a stop's grace (`STOP_GRACE`,
    five seconds,
    the wait for a descendant that ignores the request before the kill),
    and a drop that
    aborted the task earlier would cancel that kill for a descendant that holds no pipe.
    `WATCHER_END_BOUND` is now `STOP_GRACE` plus
    `STOP_TAIL_ALLOWANCE` (one second,
    twenty-four times the longest tail measured),
    six seconds;
    before,
    it was five seconds,
    which equals
    the grace and so could cut it.
    A notice written to a terminal that never takes the line is the other stall that bound exists for;
    it
    has no measurable length,
    so it is the bound itself.
  - Controls:
    `the_wait_for_a_held_name_outlasts_the_longest_hold_ever_measured_with_its_margin` and
    `the_end_bound_covers_a_stop_with_its_whole_grace_and_the_longest_tail_measured` (both red before the values changed).
  - What the host load did:
    nothing was waited for to go quiet;
    the load average read 15 to 70 during these runs and is part of the figures.

### The two gate failures of main at `85e6d2ab99`, 2026-11-07T11:40:00-05:00

The coordinator's gate under a load average of about 70 failed two controls that passed 5165 of 5165 in `tokio-gate-merge2`
(log `${HOME}/temp/agent/tokio-mutants-gate.log`).
Neither is a T5 regression.

- `push::push_progress::tests::a_write_of_a_process_the_root_started_is_progress` (`DirectoryNotEmpty`):
  a test-helper defect from before
  T5.
  `end()` called `Child::kill`,
  which signals the leader only;
  `spawn_owned` ends the child's group only when the test thread's
  fixtures drop,
  after `remove()`;
  the control's background subshell (`(sleep 0.6; echo a >> log; sleep 60) & wait`) outlived `end()`
  and created `log` while the folder was removed.
  Any byte moved sets `seen` (the meter counts reads),
  so the control also stopped
  before the write.
  Deterministic reproduction,
  red `push_progress_tests.rs` `ending_a_child_ends_the_processes_it_started_too` (the
  subshell writes 0.3 s after it starts;
  `log` existed after `end()`),
  then the fix:
  `testing/test_support.rs` `end_with_its_group`
  (`killpg`,
  reap,
  look until `kill(-group, 0)` says the group is empty) used by `end()`,
  and the descendant control waits for `log`
  itself.
  Other leader-only `kill()` calls in tests were listed (binary service tests kill a service program with no subshell);
  none of them was changed,
  and a search for `& wait` in test files found nothing else of this shape.
- `lock::git_stop_lock_tests::a_push_stop_of_a_responsive_holder_leaves_no_lock` (a lock left without its pid file):
  the stop path of that
  control is `stop_push` in `push/bounded_push.rs`,
  which the T5 commits did not touch (`git diff 5e441f25d3 48611f7e09` lists no
  `bounded_push.rs`),
  and it uses no runtime,
  no `ChildWatch` and none of the new files.
  Evidence for the cause,
  not a reproduction:
  Git 2.56's signal handler removes the lock's pid file first and the lock second (`tempfile.c`,
  `remove_tempfiles`,
  the list is last
  registered first;
  a `strace` of `git update-ref --stdin` told to prepare and sent `SIGTERM` showed `unlink(x~pid.lock)` and then
  `unlink(x.lock)` 130 microseconds apart),
  so a `SIGKILL` between the two unlinks leaves exactly the failed state (a 41-byte lock,
  `pid_file_beside=false`),
  which the sweep leaves alone because nothing proves its holder gone (`stale_ref_lock_retire.rs`,
  rule:
  never
  remove an unproven lock).
  The log says the escalation did happen:
  the sweep that warned ran 12 s after the lock was born,
  about 8 s
  after the stop returned,
  so the killed tree took seconds to be gone,
  as a process held in a stalled disk call does,
  or one that the
  scheduler did not run (this host once took 6.03 s of wall time for a check and a rename,
  section c under the heading
  `The coordinator's three settlements after T5, 2026-11-07T10:20:00-05:00 to 2026-11-07T11:10:00-05:00`;
  whether disk or scheduler is not measured).
  A load of 70 on the gate's few CPUs is the trigger;
  the cause is the escalation
  at the end of the grace cutting a handler that the host had not let finish.
  Reproduced deterministically for the cut itself,
  not for
  the delay:
  `a_push_stop_of_a_holder_slow_to_finish_its_cleanup_leaves_no_lock` (`lock/git_stop_lock_tests.rs`,
  `#[ignore]` because it fails
  today and would hold the gate) starts the holder under `strace -f -e trace=unlink -e inject=unlink:delay_exit=5000000:when=1`,
  which
  holds Git's handler for five seconds after the pid file was removed;
  the stop's grace of two seconds ends inside that hold,
  the kill
  follows,
  and the run ends with `main.lock` of 41 bytes and `pid_file_beside=false`,
  the failed state of the gate,
  after 24.6 s (run by
  hand,
  `--run-ignored only`).
  What is not reproduced is why the handler was slow in the gate (a stalled disk or a processor not given to
  it).
  The control's grace for a responsive holder is now twice the longest delay seen (`RESPONSIVE_GRACE`,
  `git_stop_lock_tests.rs`);
  a responsive holder ends at once,
  so the longer grace costs nothing when it works.
  Exposure that remains in production,
  for the coordinator and the owner,
  not fixed here:
  `STOP_GRACE` is five seconds,
  below the
  measured 6.03 s stall,
  so the same cut can leave a ref lock that the next Git command must name,
  which weighs against "Commits
  should never fail."
  A fix that stays inside the rules would not escalate to `SIGKILL` while a member of the tree is in an
  uninterruptible call (state `D` in `/proc/<pid>/stat`,
  where `SIGKILL` cannot end it anyway and only cuts what follows),
  bounded
  by a further wait;
  it needs a decision on that bound,
  and a way to test it,
  which the injection above does not give.

### Step T8, the user-manager D-Bus client on `zbus` 6, 2026-11-07T12:40:00-05:00

- Red `afa89f5ffc`,
  green `959a79f6ac`,
  lint fixes `768f215ae3`.
  `service/systemd_dbus.rs`:
  the hand-written connection (authentication
  lines,
  serial counting,
  message reading,
  the reply decoder's use) is replaced by `zbus` 6.2.1 (`default-features = false`,
  feature
  `tokio`;
  the prototype of `doc/research/wrapper-git-free-jobs-and-published-crates.md`,
  "The prototype",
  was the model).
  `Bus` keeps its
  synchronous face (`connect`,
  `within`,
  `reload`,
  `reset_failed`,
  `stop_unit`,
  `stop_unit_alone`,
  `start_unit`,
  `active_state`) so
  `service/service_provision.rs` and `service/service_client_settings_and_watches.rs` are unchanged;
  each call enters the process's runtime
  through `lazy_runtime::run_on_runtime` under `tokio::time::timeout` of what is left of the exchange's deadline,
  connect and `Hello`
  included.
  A call from inside the runtime is refused with a plain connection error.
  The "lack of a blocking api" is worked around by
  that bridge,
  which goes when the callers become asynchronous (T11).
  The `zbus` errors map to the old `CallError` (`MethodError` to
  `Manager`,
  handshake to `Authentication`,
  address and connect failures to `Connection`,
  a socket timeout and an elapsed deadline to `TimedOut`).
- Locked:
  `zbus` 6.2.1 with `zbus_macros`,
  `zbus_utils`,
  `zcheapstr`,
  `zruntime`,
  `winnow`,
  `uuid` 1.29.0 (1.29.1 set aside by the release-age and
  trust policy) and the crates for the platforms the owner said to ignore (`uds_windows`,
  four `wasm-bindgen` crates,
  `js-sys`);
  25 versions
  were listed by `native:lock:members` (the whole list is in `${HOME}/temp/agent/tokio-steps/t8-lock1.log`),
  every one found in the local
  registry cache,
  so no `cargo fetch` was needed.
- Controls:
  the fake bus of `service/systemd_dbus_tests.rs` follows `zbus`'s handshake (`NEGOTIATE_UNIX_FD`,
  signals with a path and an
  interface,
  a reply before `NameAcquired`,
  the serial of each call echoed) and its byte-for-byte call comparisons became checks of the
  member,
  the unit and the job mode (the plain stop keeps `replace`;
  the stop that spares dependents names `ignore-dependencies`);
  a bus that
  accepts and never answers ends at the deadline (red-proof of the bound:
  it passed with the hand client as well,
  so it is a preservation
  control);
  `the_client_speaks_through_zbus_and_no_longer_writes_the_wire_itself` was the red one.
  The hand wire format
  (`service/systemd_dbus_wire_format.rs`) stays for the fake bus,
  its own controls and the fuzz target `dbus_reply` until T13;
  its two
  message-type constants are now `#[cfg(test)]`.
- Not run:
  the ignored control against the real user manager (`the_real_user_manager_loads_and_starts_a_runtime_only_probe_unit`;
  it
  writes unit files to the account's real service manager).

### Step T9, the standards-review call on `hyper`, 2026-11-07T13:20:00-05:00

- Red `84404ca225`,
  green `e4773d1976`.
  `standards_review/standards_review_client.rs`:
  `hyper` 1.12 with `hyper-util`'s legacy client,
  `hyper-rustls` 0.27.10 (`http1`,
  `ring`,
  `webpki-tokio`,
  `tls12`;
  no system roots),
  `http-body-util`,
  `rustls` with the `ring` provider,
  `webpki-roots` for the embedded roots;
  the trusted authority is read with `rustls`'s PEM reader.
  `ureq` and its packages are gone from the
  manifest and the lock.
  The call runs through `run_on_runtime` (the plan's inventory puts the review after landing,
  not on the
  wait path).
  The quiet window is one clock:
  `Marking` wraps every connection the connector makes and marks every read or write that moves
  bytes,
  the connect and TLS handshake included,
  and the call is raced against `silence` (the call ends with "The operation was aborted due to
  timeout" after a quiet window);
  the ceiling is `tokio::time::timeout` and ends with the same sentence as before.
  `ureq` set one timeout per phase;
  this is one clock over "no byte moved",
  which is the rule the module's doc states.
  No redirect is followed,
  no proxy variable is read,
  plain
  `http` is accepted for the loopback mock,
  the error text carries the whole cause chain.
- Catalog:
  `rustls`,
  `webpki-roots`,
  `bytes` and `tower-service` joined `rust-catalog.toml` (the other `hyper` entries were there already).
- Controls:
  the existing client controls pass unchanged on `hyper` (loopback mock:
  request line,
  headers and body,
  error status as a reply,
  chunked body,
  redirect not followed,
  silent server ends at the window,
  refused connection,
  unreadable authority,
  a reply that keeps
  arriving is never cut off and is cut at the ceiling);
  new:
  `a_tls_server_is_reached_only_through_a_trusted_authority` (a throwaway authority
  and a leaf made by the `openssl` tool,
  which the gate image has,
  and a `rustls` server in the test;
  the trusted authority is reached with the
  bearer header,
  and the embedded roots refuse it;
  it passed with `ureq` first,
  so it proves the control reaches TLS),
  the guard literal
  of `source_guard/standards_review_boundary_tests.rs` (the library paths `hyper::`,
  `hyper_util::`,
  `hyper_rustls::`,
  `http_body_util::`
  replace `ureq`,
  with its fixture controls),
  and `the_client_is_built_on_hyper_and_no_longer_on_ureq` (red before).
- Known from the research and not changed:
  `hyper-util`'s resolver runs `getaddrinfo` on tokio's blocking pool when the URL has a host name,
  and that thread stays idle for tokio's keep-alive;
  the endpoint is one fixed host.

### Gates of the settlements, T8 and T9, 2026-11-07T14:10:00-05:00

- Settlements (a) to (c),
  snapshot at `9dee521668` (unit `tokio-gate-settle`,
  log `${HOME}/temp/agent/tokio-steps/gate-settle.log`):
  5174 tests
  run,
  5174 passed (3 slow),
  11 skipped;
  the container Clippy then failed on shadowed bindings in `push/child_watch_drop_time_measurement_tests.rs`
  (fixed in `768f215ae3`).
- T8 and T9 together,
  first snapshot at `e4773d1976` (unit `tokio-gate-t8t9`,
  `gate-t8t9.log`):
  5179 run,
  5172 passed,
  7 failed,
  all of them
  the stand-in session bus of the binary controls (`binary/binary_service_provision_tests.rs`),
  which did not agree to `NEGOTIATE_UNIX_FD` or
  answer `Hello` with a name,
  so `zbus` waited out the 20 second exchange.
  Fixed in the commit that names them.
- T8 and T9 together,
  after that fix (unit `tokio-gate-t8t9b`,
  `gate-t8t9b.log`):
  5179 run,
  5179 passed (1 slow),
  11 skipped,
  container
  Clippy clean.
  Added after that snapshot,
  all tests or comments:
  `the_quiet_gap_kept_for_a_call_is_the_calls_own_and_not_the_idle_waiting_thread`
  (passes:
  the quiet clock counts the process's established sockets as work,
  so the idle waiting thread of the `hyper` call does not make
  the ledger figure hollow),
  the ignored strace reproduction,
  and the reworded citation of the 6.03 s;
  the package lint of that tree is clean
  and the quiet-gap control passed on the host.
- Not measured:
  `native:perf` (the plan asks for it after T10,
  T12 and T13).
  Settlement (a) is the first change on the commit wait path (every
  `EndAt` Git step of `run_git_watched` now enters the runtime twice,
  `attach_pipes` and `read_until_the_child_ends`);
  its cost against "commits
  vs native git should not take 5s longer" is an unmeasured exposure until then.

### The owner's pick on the waits, and the stop that waits for a member that cannot be interrupted, 2026-11-07T15:10:00-05:00

- The question (asked by the coordinator in the question tool):
  how much longer than the measured worst case (a check and a rename that took 6.03 s
  of wall time once,
  cause not measured) the wrapper waits before giving up,
  for (1) the wait before force-killing a Git asked to stop and (2) the
  held-name wait.
  The owner picked,
  2026-11-07,
  verbatim:
  "3x worst case".
- Applied in `foundation/worst_delay_seen_on_this_host.rs` (`LONGEST_DELAY_SEEN` 6.031986 s,
  `OWNER_MARGIN` 3,
  `WORST_CASE_WITH_MARGIN` about 18.1 s):
  `HELD_NAME_WAIT` in `private_storage.rs` is that value (it was 60.3 s with the agent's factor ten);
  `STOP_CEILING` is the same value.
- The stop:
  `process/stop_waits_while_a_member_cannot_be_interrupted.rs` `escalation_waits`.
  After `SIGTERM` the stop waits its grace (`STOP_GRACE`,
  5 s);
  past it,
  while some member of the stopped tree is in uninterruptible sleep (`/proc/<pid>/stat` state `D`,
  `process_tree.rs` `in_uninterruptible_sleep`)
  it keeps waiting,
  and it forces the end no later than `STOP_CEILING` after the request;
  with no member in `D` it forces at the grace,
  as before.
  The
  look is at the interval each stop's own loop already used (10 ms,
  `POLL_PAUSE` and `STOP_POLL`),
  so no loop was added.
  Stops that escalate
  `SIGTERM` to `SIGKILL`:
  `push/bounded_push.rs` `stop_push` (also the stop of `push/bounded_query.rs`,
  which calls it) and
  `push/child_watch_clock_marks_and_stopping.rs` `stop_process` (every watched step).
  Other kills in production code send `SIGKILL` with no prior
  request (`candidate/candidate_reader.rs`,
  `push/auto_push_reconcile_merge_commit.rs`,
  `repair/package_repair_command.rs`,
  `process/child_awaited_through_a_pidfd.rs`) and are not stops of this kind.
- `WATCHER_END_BOUND` is `STOP_CEILING` plus `STOP_TAIL_ALLOWANCE` (one second,
  twenty-four times the longest tail measured),
  about 19.1 s;
  it was
  `STOP_GRACE` plus the allowance.
  The earlier text of this plan that gives 60.3 s and 6 s is superseded by this section.
- Controls:
  red `c73ca8c76e` (the exact-value control of the held-name wait,
  the end-bound control,
  and the strace reproduction,
  now running);
  green:
  the reproduction `a_push_stop_of_a_holder_slow_to_finish_its_cleanup_leaves_no_lock`
  passes;
  `a_member_held_for_ever_is_ended_at_the_ceiling_and_the_lock_is_swept` (a stop whose member never leaves the blocked state returns one
  ceiling after the request and the lock is swept),
  and four unit controls of `escalation_waits` (inside the grace,
  past it with no member blocked,
  a
  member that never leaves,
  the real ceiling is three times the delay and past the grace).
  A planted control (the ceiling removed from `escalation_waits`)
  failed the held-for-ever unit control.
- A deviation to know:
  `strace` holds the delayed handler in a tracing stop (state `t`),
  not in `D`,
  and a `D` cannot be made on demand.
  The reproduction and
  the held-for-ever control therefore install,
  on their own thread,
  a stand-in rule (`D` or `t` counts;
  a test-only seam of `escalation_waits`);
  the
  production rule looks for `D` only.
  So the controls prove the loop's waiting and its ceiling,
  and that waiting through the handler's cleanup leaves no lock;
  they do not prove that a real `D` is recognised beyond `the_state_look_reads_a_real_process_and_refuses_a_gone_one`,
  which reads this test's own process.

### Main merged again at `c23b85309c`, and the gate after the owner's pick, 2026-11-07T16:05:00-05:00

- The merge of main (the fold late-line fix,
  the index-lock fallback `landing/service_index_*`,
  the settle-control retry,
  the held-message work):
  one
  conflict,
  in `rust-catalog.toml` (the default catalog's `branches` and `bytes` lines against main's named catalogs).
  Resolved by keeping `bytes` and the
  entries this branch added (`rustls`,
  `webpki-roots`,
  `tower-service`) and main's `branches-0-4` and `zbus-5` catalogs as they came;
  the default
  catalog's `branches` line,
  which main removed,
  stays removed.
  `mise run sync:files` wrote nothing;
  `native:lock:members` found the lock satisfying the
  manifest with no change;
  the merge changed neither `Cargo.toml` nor `Cargo.lock` of the wrapper against this branch's tip.
  No source conflict.
- First gate of the final tree (unit `tokio-gate-final`,
  `gate-final.log`):
  5209 run,
  5207 passed,
  2 failed,
  neither in code this branch changed.
  `lock::index_holder_wait::tests::an_owner_nothing_proves_alive_is_waited_for_like_a_proven_one`:
  `Released(2.49917834s)` against a hold of 2.5 s,
  because the
  hold was counted from the sleeper thread's own start,
  which is before the wait's clock starts,
  so the wait could read a little less than the hold (the
  mechanism is in the test,
  0.8 ms short here;
  two controls of the file had it,
  both fixed to count the hold from the wait's first notice).
  Passed alone and
  in the earlier gates.
  `repair::package_repair_command::tests::a_helper_that_keeps_writing_after_the_leader_exits_does_not_hold_the_run`:
  `leader-output` missing
  from the snapshot;
  `repair/` and `process/output_capture.rs` are byte for byte those of main and of `5e441f25d3`,
  the control passed in the five earlier gates
  (0.31 to 0.32 s) and alone (0.345 s),
  and failed once at 0.506 s;
  the cause was not found and is not claimed.
- Second gate of the final tree (unit `tokio-gate-final2`,
  `gate-final2.log`):
  5209 run,
  5209 passed (1 slow),
  11 skipped,
  container Clippy clean.

### The five failures of the gate of `944bdd30b5`, 2026-11-07T17:30:00-05:00

The coordinator's gate (log `${HOME}/temp/agent/tokio-merge-gate-944bdd30b5.log`):
5216 run,
5211 passed,
5 failed,
under a load average of about 70;
the
branch's own gate on `ad314ca6eb` had passed 5209 of 5209.
The coordinator's hypothesis,
unmeasured,
was that members of stopped trees are often in state
`D` under disk pressure,
so every stop now waits up to its ceiling and tests sized to the old five-second escalation run out their own bounds.

- Measured against the hypothesis (Summary of what was measured,
  not guessed):
  1. The stop controls took the same time in that gate as in the clean gate and in the gate before the change (for example 2.6,
     4.6,
     5.1 and 8.1 s for the
     `git_stop_lock_tests` controls in all three;
     `child_watch` stop controls 1.2 to 3.1 s);
     none waited toward an 18 s ceiling.
  2. A probe in `escalation_waits` (temporary,
     reverted) counted the looks that found a member in state `D` in 86 stop-related controls
     (`git_stop_lock`,
     `child_watch`,
     `bounded_push`,
     `bounded_query`,
     `push_wait`,
     `signal_relay`) run on this host at a load average of 24 to 47:
     zero.
     The unresponsive-holder controls do reach the looks past the grace,
     so the look was made;
     the holder was asleep (`S`),
     not in `D`.
  3. Production code reaches `escalation_waits` only through a stop (`stop_push`,
     `stop_process`);
     the three aborted push controls log no stop (the slow-server
     one logs only the first "waiting for the server" notice,
     which says the push is waited for and not stopped).
  The hypothesis is therefore not supported for these failures.
     It is not refuted for real disk pressure,
     which could not be produced here.
- The measured cause of four of the five is a host stall episode,
  not a wrapper change.
  Against the clean gate,
  15 tests took more than ten seconds longer,
  and the
  ones that need no wait took the longest:
  `auto_push_trigger::a_branch_that_grew_ahead_of_its_upstream_is_handed_over` 0.04 s to 122.3 s,
  `bounded_query::a_finished_query_keeps_code_and_streams_apart` 0.02 s to 35.0 s,
  `worktree_copy_stage::copied_file_has_identical_bytes` 0.01 s to 18.9 s,
  `auto_push_reconcile::what_the_fetch_and_the_fast_forward_say_is_kept_in_the_journal` 0.28 s to 174.5 s (aborted by the 120 s watchdog),
  `push_slow_server_tests` two controls 4.4 s to 120.1 s and 12.4 s to 150.6 s (aborted);
  the sum of test seconds was 2108 against 1289.
  The three aborted
  push controls,
  run alone afterwards at a load average of 24 to 47,
  took 0.31,
  4.16 and 12.15 s,
  as in the clean gate.
  What main added since the clean gate
  (`landing/earlier_service_*`,
  the settle command) does not touch push.
  The stall's resource (disk or processor) was not measured,
  and no fix in the wrapper
  follows from it;
  the controls were not changed.
- `bounded_push::a_tree_still_running_after_the_grace_is_swept_when_it_ends` (81.9 s,
  "a tree that still runs is not swept") is one of those victims whose
  assertion depended on timing:
  its member was a `sleep 1`,
  which had ended by the time the control looked.
  Reproduced deterministically (a 1.5 s hold of the
  control's thread before `settle_locks` gives the same message,
  run by hand and reverted),
  then fixed:
  the member is a `cat` that lives until the control closes
  its input,
  and the control holds its own thread for 1.5 s on purpose,
  so the delay stays covered.
- `service::service_reply::tests::a_queued_answer_is_written_before_the_connection_closes`:
  not this branch (the coordinator read the code:
  `write_until_answered_noting`
  beats whenever 20 ms pass with nothing queued,
  and beats before an answer are allowed).
  Positive control first:
  the same control with the sender held for three
  beats failed with `left: Heartbeat` every time (red commit);
  the control now skips beats,
  asserts the first other message is the answer and that only beats follow,
  and the late-sender variant stays as a permanent control.
- Proposal,
  not implemented (nothing measured says the wait is a problem,
  so the owner's pick stays as applied):
  to tell a member that is finishing its cleanup
  from one that is only doing input and output,
  the usable evidence is on disk.
  Measured on Git 2.56:
  a Git that holds a ref lock keeps no descriptor open on the
  lock file (`/proc/<pid>/fd` listed only the three standard streams and no lock),
  and an unlink moves no `/proc/<pid>/io` byte counter,
  so open files and byte
  counters cannot say it.
  What can:
  wait for a member in `D` only while the repository holds a lock file born since the step began that the sweep would judge
  unproven (the `LockLedger` already names the repositories and the birth time),
  because the cut that leaves a lock is exactly the state after the pid file is
  gone and before the lock is;
  with no such lock a member in `D` is only doing input and output and the kill goes at the grace as before.
  It needs a decision
  on whether the extra look (a directory walk of the ledger's roots at the stop's 10 ms interval,
  or once per 100 ms) is worth it.

### Gate after the fixes, with main merged again at `8b8c9440de`, 2026-11-07T18:20:00-05:00

- Main merged again:
  no conflict,
  no source file of the wrapper changed besides one version line in its `Cargo.toml` and `Cargo.lock`;
  `sync:files` left the tree clean.
- Unit `tokio-gate-final3` (`gate-final3.log`),
  load average about 66 during the run:
  5217 run,
  5217 passed (3 slow),
  11 skipped,
  container Clippy clean.
  The package lint
  before it was clean.

### Steps T6 and T7 on `feat/git-policy-cli-tokio-steps-6-onward`, 2026-11-07T15:33:44-05:00

- Branch and worktree:
  created from `9afcac4a26`,
  the frozen tip of `feat/git-policy-cli-on-tokio`,
  in `.claude/worktrees/tokio-steps-6-onward`.
  Main is not merged into it yet (the coordinator will say when).
- What the two steps are,
  as they were built.
  The pattern of T4 and T5 held:
  each conversion is an `async` core plus the synchronous face that keeps today's name and signature and enters the runtime,
  so the controls of the callers are unchanged.
  The synchronous faces go when the callers are converted (T11).
- Open choice 6 has the owner's answer in "The owner's answers at 2026-11-07" (choice 6,
  "Registry of children",
  with
  "Only that operation's children (Recommended)" and a stop of the service as an operation of its own),
  so T7 was done,
  and open choice 4 ("Rewrite per-thread state") is the answer T10 will follow.

#### T6, commits `4065cb84a5` (red), `2a6b9f66e9`, `347d570501`, `cc30b3f5f3`, `ea44eae518`

- `push/start_retry.rs`:
  the schedule of a refused start (`RefusalSchedule`,
  the 10 ms first pause doubling to 500 ms,
  the 5 s budget unchanged) is one value that
  the sleeping loop and the new awaiting loop share;
  `retry_refused_start_awaiting_within` and `spawn_retrying_without_blocking` pause on `tokio::time::sleep`.
  `captured_child_run_to_its_end.rs` (`output_of`) starts that way.
  Decision,
  the agent's and open to the owner's veto:
  the synchronous pause stays for the callers that have no runtime.
  A forwarded command (`invocation/forwarding.rs`,
  `exec_retrying` and `status_retrying`) builds no runtime
  (`binary/binary_forwarded_command_makes_no_thread_tests.rs`,
  the owner's "commits vs native git should not take 5s longer."
  applies to it first),
  so its pause is the one `std::thread::sleep` in `push/start_retry.rs` (`sleep_for`),
  and the T13 source guard against `thread::sleep` must allow exactly that function.
  Every other synchronous caller of `spawn_retrying` and `output_retrying` (13 files) keeps the sleeping pause until T11.
- `push/bounded_push.rs`,
  `push/bounded_query.rs`:
  `run_bounded_push_awaiting`,
  `run_bounded_query_awaiting`,
  `wait_until_done`,
  `wait_until_done_noting`,
  `stop_push`,
  `settle_locks`
  and `wait_for_tree` are `async`;
  `run_bounded_push_once` and `run_bounded_query` are the synchronous faces (`run_on_runtime`,
  a refusal becomes `SingleFlightError::Start`).
  The exit of the push is awaited on its pidfd (`OwnedChild::wait`);
  the loop's pause is `tokio::time::timeout(POLL_PAUSE, ...)` around that wait,
  because the output size and the clock of signs of work
  have no event to wait on and are looked at every `POLL_PAUSE` as before.
  `stop_push` takes the `OwnedChild`,
  and a child that was collected has nothing to hand over
  (`OwnedChild::into_std_when_unreaped`,
  `OwnedChild::kill`,
  both new).
  `sweep_after_exit` is a task of the runtime (`spawn_on_runtime`,
  `None` where the runtime has no worker,
  as when the thread could not start)
  that adopts the standard-library child inside the task,
  so a task that never runs leaves the child as it was and never waits for it
  (an `OwnedChild` dropped alive kills it and waits,
  which must not happen to a child in uninterruptible sleep);
  its sweep of directories runs on the blocking pool.
  `settle_locks` takes `Option<Child>` (`None` is a collected child).
- `repair/package_repair_command.rs`:
  `run_tool_reaping` is `async` and takes an `AsyncFn` reaper;
  `run_tool` is the synchronous face;
  `reap_within` is `async` and hands a leader that has not ended in its budget to a task.
  The wait seam (`WAIT_FAILS_ONCE_THIS_FILE_EXISTS`) and the stop seam (`CONTROL_RULES` of `escalation_waits`) are thread-locals of the test thread,
  and they still reach the code,
  because `Runtime::block_on` runs the future on the calling thread;
  the proof is that `a_member_held_for_ever_is_ended_at_the_ceiling_and_the_lock_is_swept` takes the same time as before
  (4.2 s against 4.2 to 4.8 s),
  where an unreached seam would wait the real 18 s ceiling.
  Only the two sweeps and the reaper of a late leader are spawned,
  and neither reads a seam.
- Files T6 changed:
  `src/native/push/start_retry.rs`,
  `src/native/push/bounded_push.rs`,
  `src/native/push/bounded_query.rs`,
  `src/native/process/child_awaited_through_a_pidfd.rs`,
  `src/native/process/captured_child_run_to_its_end.rs`,
  `src/native/repair/package_repair_command.rs`,
  and the new test helper `src/native/testing/ticker_beside_a_wait.rs`.
  `push/child_watch.rs` was not touched.
- Reachability of the `Nested` refusal (the synchronous faces refuse from inside the runtime,
  so a path from runtime code to one is a defect):
  the code that runs inside the runtime today is the bodies of `spawn_on_runtime` tasks (the watcher of `child_watch.rs`,
  the sweep task),
  `run_blocking_job` jobs (the stopped-step sweep,
  the sweep job,
  the runtime probe) and the futures of `run_on_runtime`
  (`output_of`,
  the pipe reads,
  the `zbus` calls,
  the standards review).
  `rg` for `run_bounded_push`,
  `run_bounded_query`,
  `run_tool`,
  `run_child`,
  `SignalGuard` and `run_clearing_locks` outside tests names only
  `commit_preparation.rs`,
  `lfs_upload.rs`,
  `lfs_bounded_push.rs`,
  `auto_push_reconcile_basics_and_limits.rs`,
  `auto_push_single_flight.rs`,
  `push_lock_clearing.rs`,
  `bounded_push.rs`,
  `bounded_query.rs`,
  the repair files and `signal_relay.rs`;
  none of the runtime bodies above calls into them (`LockLedger::sweep` reaches `clear_stale_ref_locks` and the judge,
  which read files and `/proc`).
  The chain stays synchronous at `run_clearing_locks`,
  which clears locks with code that is not asynchronous yet,
  so the bridge is placed below it.
- Controls.
  Ported unchanged in assertions:
  all of `bounded_push_tests.rs`,
  `bounded_query_tests.rs`,
  `start_retry_tests.rs`,
  `package_repair_command_tests.rs`,
  `git_stop_lock_tests.rs`,
  `push_slow_server_tests.rs`,
  `push_patience_tests.rs`;
  the two controls that called `sweep_after_exit` and `settle_locks` directly now join a task (`a_child_released_late_has_its_lock_swept_when_it_ends`,
  `a_tree_still_running_after_the_grace_is_swept_when_it_ends`),
  and the three repair controls that gave the reaper a closure give an async closure.
  New,
  each a task that counts every millisecond on the same thread as the wait
  (`a_pause_between_refused_starts_lets_other_tasks_run`,
  `the_wait_for_a_running_push_lets_other_tasks_run`,
  `the_stop_of_a_push_that_ignores_the_request_lets_other_tasks_run` (the stop run alone,
  so the ticks of the wait before it cannot count for it),
  `the_wait_for_a_running_query_lets_other_tasks_run`,
  `the_wait_for_a_running_tool_lets_other_tasks_run`,
  `the_reaping_of_a_leader_that_does_not_end_lets_other_tasks_run`).
  Planted controls,
  each run once and reverted:
  the red commit `4065cb84a5` ran the retry's pause as `thread::sleep` and the start-retry control read zero ticks over three pauses;
  with the old blocking shape planted in `wait_with_clock` and `stop_push` (a `thread::sleep` and a `try_wait` in place of the awaits)
  all three push and query controls failed;
  with a 400 ms `thread::sleep` in `wait_status` and a `thread::sleep` of the budget in `reap_within`,
  both repair controls failed.
- Measured,
  the seven file sets `bounded_push`,
  `bounded_query`,
  `start_retry`,
  `git_stop_lock`,
  `push_slow_server`,
  `push_patience` and
  `package_repair_command` (68 tests before),
  three host runs each,
  on the branch tip before any edit
  (`${HOME}/temp/agent/tokio-steps6/baseline-run*.log`,
  load average 40 to 76),
  after the push and query step
  (`after-b-run*.log`,
  74 tests,
  load average 27 to 31) and after the repair step and T7 (`after-c-run*.log`,
  93 tests with the `signal_relay` set added,
  load average 38 to 72):
  summary 72.3 to 73.0 s before and 72.3 to 72.4 s after the push step and 72.9 to 73.0 s after the last,
  and the per-test times agree within the spread of the three baseline runs
  (the longest controls,
  `a_push_over_http_waits_past_the_old_low_speed_cutoff_and_lands` 65.07 to 65.14 s before and 65.08 to 65.09 s after;
  `a_watched_stop_of_an_unresponsive_holder_leaves_no_lock` 8.20 to 8.31 s before and 8.20 to 8.70 s after).
  The controls are dominated by waits that the tests themselves set,
  so they show no regression,
  and they cannot show a gain;
  the exit of a push is now seen at once and not at the next 10 ms look,
  a difference below the spread.
- Gate,
  snapshot `ea44eae518` (unit `t6-gate`,
  log `${HOME}/temp/agent/tokio-steps6/gate-t6.log`):
  5225 tests run,
  5225 passed (1 slow),
  11 skipped,
  summary 570.7 s;
  container Clippy clean.
  Host Clippy and the package lint were clean on the tree after the last T6 commit.

#### T7, commit `0a19f2cdc4`

- Design (the agent's,
  built as the plan's choice 6 left it open;
  open to the owner's veto):
  - The registry is one slot,
    because a process runs one operation with a signal of its own.
    The first `SignalGuard` of a process owns the four handlers and publishes its child;
    a guard made while another owns them is made all the same,
    relays nothing and waits for no one
    (the polling `install_guard_when_free` of `commit_preparation.rs`,
    `GUARD_WAIT_LIMIT`,
    `retry_while_active` and the `SignalGuardActive` refusal type are gone).
  - The recipient rule:
    a signal reaches only the child of the guard that owns the handlers.
    A service calls `signal_relay::leave_signals_to_the_service` where it installs its own stop handler
    (`service/service_signal.rs` `install`),
    after which no guard in that process takes the handlers or relays.
    Before this change a commit prepared inside the service replaced the service's stop handler for the length of the Git step
    and relayed a stop of the service to that Git;
    now the service's own handling runs and the Git child is not signalled.
    The consequence for the owner is the new open choice 12.
  - The pidfd and the descriptor number.
    The handler signals through `rustix::process::pidfd_send_signal` (a raw system call of the Linux backend:
    no allocation,
    no `errno` write),
    on one descriptor number that is made on the first publish (`/dev/null`,
    `RELAY_FD`) and never closed.
    Publishing and retiring replace what the number refers to with `dup3`,
    atomically,
    so the handler never reads a closed number,
    the number is never another process's pidfd (nothing else can be handed it),
    and a signal after the child was reaped finds `/dev/null` or a reaped pidfd and fails harmlessly.
    This is a proof by construction of what the plan said "needs a proof or a different mechanism";
    it costs one descriptor for the life of a process that relays.
    The wait is `OwnedChild::wait` on a second pidfd,
    so `waitid(WNOWAIT)` and its blocked thread are gone.
  - A failure to make the descriptor or the pidfd leaves the child unrelayed and logs it (`signalRelay`):
    "Commits should never fail."
    outranks the relay.
  - The non-Linux Unix path keeps the number-based `kill` (not converted:
    "Ignore Win and mac until I say so.").
- `run_child` is the synchronous face of `run_child_awaiting`.
- Files T7 changed:
  `src/native/process/signal_relay.rs`,
  `src/native/commit/commit_preparation.rs`,
  `src/native/service/service_signal.rs`.
- Controls.
  Unchanged and green:
  `process_signal_is_relayed_and_the_child_outcome_is_reported`,
  `signal_during_start_is_relayed_after_the_child_starts`,
  `terminal_interrupt_is_not_relayed_a_second_time`,
  `term_during_a_held_pre_commit_ends_git_and_reports_143`,
  `dropping_the_guard_restores_the_previous_dispositions`,
  and at the binary level the twelve `signal` controls
  (`a_termination_sent_to_the_wrapper_alone_is_relayed_to_git` and its three siblings among them).
  Removed with what they tested:
  the guard wait of `commit_preparation_tests.rs` (two controls),
  `a_second_guard_is_refused_until_the_first_is_dropped`,
  `only_the_refusal_of_a_second_guard_is_recognized_as_such`,
  and the three controls of `wait_without_reaping`
  (the call is gone;
  their two intents are kept as `waiting_ends_at_the_childs_exit_and_reports_it` and
  `an_interrupted_wait_continues_until_the_child_exits`,
  on the new wait).
  New:
  `a_second_guard_is_made_and_relays_nothing_while_the_first_owns_the_handlers`,
  `the_child_of_a_guard_without_handlers_is_never_published`,
  `the_relay_descriptor_names_the_child_while_it_runs_and_nothing_after`,
  `a_stop_of_a_service_is_not_relayed_to_the_child_of_a_commit`.
  Planted controls,
  each run once and reverted:
  the guard without handlers made to publish (`run_relayed(command, true)`) failed the publish control;
  `leave_signals_to_the_service` made a no-op failed the service control;
  the retire left out failed the descriptor control;
  a second guard that claims to relay failed the second-guard control.
- Gate,
  snapshot `0a19f2cdc4`,
  T6 and T7 together (unit `t7-gate`,
  log `${HOME}/temp/agent/tokio-steps6/gate-t7.log`):
  5224 tests run,
  5224 passed (1 slow),
  11 skipped,
  summary 756.6 s;
  container Clippy clean.
  The gate read longer than the T6 gate (the sum of per-test seconds is 1479 against 1140),
  and a failure or a slowness is a defect until measured:
  the ten tests that slowed most against the T6 gate are in code the two steps do not touch
  (for example `push::auto_push::tests::a_matching_push_of_a_branch_the_push_remote_has_goes_to_the_same_name` 0.11 s to 47.43 s and,
  beside it,
  `a_matching_push_of_a_branch_the_push_remote_lacks_publishes_nothing` 0.07 s to 47.40 s,
  two figures equal to the hundredth,
  which two tests that stalled together on the host show and a change in one of them does not),
  and six of them,
  run alone on the host afterwards at a load
  average of 46,
  took 0.26 to 1.29 s.
  So the cause of the extra time is a host stall episode,
  as in "The five failures of the gate of `944bdd30b5`",
  not these steps;
  the stall's resource was not measured.
- Not measured,
  and why:
  `native:perf` (the plan asks for it after T10,
  T12 and T13).
  T7 puts the runtime on the path of every commit's Git step (`run_child` enters it,
  building it on the first need,
  about 0.3 to 0.5 ms measured in
  "The runtime"),
  plus one `pidfd_open` and one `dup3`;
  the exposure against "commits vs native git should not take 5s longer."
  is unmeasured
  until then and is a few milliseconds by construction.

### Step T10, the service on tokio, 2026-11-07T20:40:00-05:00

Branch `feat/git-policy-cli-tokio-t10-service`,
from main at `f2a90d3fe2`,
not yet merged (see "Gate of T10").

#### What changed

- The landing's per-thread state is carried by the landing's scope (owner's choice 4,
  "Rewrite per-thread state").
  `landing/landing_state_carried_by_its_task.rs` holds one `LandingState` of `Cell`s and `RefCell`s in a `tokio::task_local!`,
  entered once by `serve` (`in_a_landing_with`) and readable by `on_the_landing_state(arguments, named_function)`;
  a task entered with `in_a_landing_task` carries it across workers.
  Ten production `thread_local!` items of the service and landing folders went into it:
  the board and the detail of a never-ended write,
  the ceiling step,
  the abandoned claims,
  the refused lock,
  the unsettled locks,
  the pre-staged marker,
  and the three of the queued syncs.
  Outside a scope nothing is remembered and a read answers its default,
  which is what a thread-local gave a thread that never began a landing;
  the wrapper's own commands never enter a scope (the index lock is the service's alone in production;
  `landing_reset_copy` in `index/index_reset_support.rs` is a test helper).
  The two counters only controls read (`name_steps`,
  `loose_looks`) are counted in test builds only.
  What stays a thread-local,
  with the reason:
  `#[cfg(test)]` fault hooks (they set a value on the test thread and the code under test runs there),
  the panic hook's count in `invocation/panic_notice.rs` (a hook runs on the thread that panics,
  so the thread is the right owner),
  `worktree/add_timing.rs` (`ACTIVE`,
  the wrapper's `worktree add`,
  with worker threads that share a `Mutex`;
  T12),
  `repo_read/repo_read_tree.rs` (`SHARED_LISTS`,
  an `Rc` cache of the wrapper's reads;
  T11).
- Connections are tasks.
  `service/service_server_accept_connections_as_tasks.rs` starts one acceptor task on the process's runtime
  (`spawn_on_runtime`;
  no worker thread ends the service with status 4 and a plain line,
  as the start of a thread did)
  and a task per connection;
  `handle_connection` is `async` on `tokio::net::UnixStream` halves,
  the request line is read under `REQUEST_READ_TIMEOUT`,
  `Reply` is an unbounded `tokio::sync::mpsc` sender (never waits,
  so the landing thread cannot block on a client),
  and `write_until_answered_noting` is an `async fn` whose beat is `tokio::time::timeout` on the channel.
  The board's file reads,
  the withdrawal decision (files in the journal),
  and the instance name of a Git directory (a path resolution) run on the blocking pool (`run_blocking_job`),
  by the plan's rule "by what a call can wait on".
  A pool that cannot start a thread answers the client `Unavailable` (instance check),
  hands a withdrawal to the landing thread (as for a request that began to land),
  or leaves out one beat's note (the beat is still written).
- A claim renewal is a task (`landing/service_claim_renewal.rs`):
  `select!` over the control channel and `sleep_until`,
  the renames on the blocking pool with the renewer going there and back,
  and `Drop` still waits until the task has ended (the same wait as the join it replaces;
  it is not bounded,
  which is open below).
- The landing's background syncs are a task that hands each batch to the blocking pool (`landing/service_durable.rs`);
  `drain` still waits for a barrier (the exit path's wait for durability).
- The marker made ahead is a blocking job awaited by a task (`service_index_prestage.rs`);
  the look at the processes is a blocking job awaited under its ceiling (`landing/scan_awaited_under_a_ceiling.rs`).
- Two closure-lint suppressions were added for the two tokio methods that accept only a callable
  (`package/git-policy/cli/doc/closure-lint-suppressions.md`).

#### What did not change, and why

- `serve` is still a synchronous loop on the main thread.
  Its Git steps reach the runtime through the synchronous bridges (`run_on_runtime`),
  which refuse a call from inside the runtime;
  an `async` landing loop is T11 (callers `async`,
  bridges gone).
  Its sleeps (`landing/service_land_follow.rs`,
  `service_index_hold.rs`,
  `service_index_wait.rs`,
  `service_index_attempts.rs`,
  `lock/index_holder_wait.rs`),
  the inotify wait and the `std::sync::mpsc` queues to the landing and the work loop stay with it.
- The work loop thread (`service/service_server_work_loop.rs`) and `landing/service_work.rs`,
  `service_work_handlers_replanning_and_acceptance.rs` are the coordinator's W3 files and were not touched.
- The client side of the service (`service/service_client*.rs`,
  `service_provision*.rs`) belongs to the synchronous commit command,
  so it waits for T11.

#### Threads of the service and their stacks (before the worker cap; with it an idle service has 6, see "the worker cap")

Measured with the release binary on this 28-CPU host,
one service,
idle (`${HOME}/temp/agent/t10-threads.py`):
before T10 four threads (the main thread that lands,
the work loop,
the acceptor,
one more) and one thread more for each open connection (36 with 32 connections);
after T10 27 threads (the main thread,
the work-loop thread and 25 runtime workers) whatever the number of connections (27 with 32 connections).
The workers are the runtime plan's (`foundation/runtime_plan.rs`),
built when the acceptor starts (before,
the first call of `spawn_on_runtime` or `run_on_runtime` built it:
the startup warm of a remembered repository,
or the first Git step of the first landing):
workers up to the budget,
fewer under an address-space limit,
`RLIMIT_NPROC` or `pids.max`,
each with a 2 MiB stack (`THREAD_STACK_BYTES`) and 64 KiB more of address space,
and the blocking pool grows on demand up to the plan's `blocking_threads` with the same stack.
The main thread has the system's 8 MiB stack and the work-loop thread Rust's default 2 MiB.
Many services run at once on one host (28 were counted on 2026-11-03),
so 25 idle workers each is a real cost of "sized to budget" for a service;
whether a service should cap its workers (its tasks wait on sockets and the pool,
and the landing is its own thread) is a question for the owner.

#### Controls

Red first,
then green:
`landing_state_carried_by_its_task_tests.rs` (two tasks that interleave on one thread keep their own state;
a task polled on one thread and finished on another keeps it;
a scope inside a scope shadows;
a thread started inside a scope is not inside a landing;
outside a scope a write is not kept),
`binary_service_connections_without_a_thread_each_tests.rs` (32 connections held open added 32 threads,
now none),
`service_claim_renewal_tests.rs` (16 renewals added 17 threads),
`service_durable_tests.rs` (a thread named `cli-git-durable`),
`service_index_prestage_tests.rs` (a thread named `cli-git-prestag`,
the kernel's 15-character name),
and `earlier_service_process_look_tests.rs` (the awaited look beside a ticker,
and a look that outlasts its ceiling).
The tests of `service_reply_tests.rs` were ported to tasks on a current-thread runtime with their assertions
and a counting task beside the writer.
Forty-six tests that read state across calls were wrapped in `in_a_landing`;
two of them start their landing on a thread of their own and enter the scope there.

#### Gate of T10

Container gate (`--cpus 8 --memory 8g --pids-limit 2048`) of the tree at `0ab4fa764b`:
5314 run,
5313 passed (3 slow),
1 failed,
11 skipped;
host Clippy clean,
package lint 0 errors and 0 warnings with the closure check run.
The failure is `fault_limits::a_memory_limit_close_to_exhaustion_on_the_service_leaves_a_recoverable_repository`,
`no address-space limit stopped a commit, so the limits tested nothing (service starts at 110, commit lands at 110)`.
It is not the failure main had.
Main's was a bisect that met the one lethal limit where the last thread's alternative signal stack cannot be allocated
(fixed in main at `5083511eed`,
merged into this branch at `325e2608ea`);
T10's persists after that merge (same test,
`starts at 113, commit lands at 113` on the merged tree),
so the two are separate incidents.
The cause of T10's is its design:
the runtime is built when the acceptor starts,
its plan keeps `ADDRESS_SPACE_RESERVE_BYTES` (64 MiB) back when it sizes workers,
and so at the smallest limit under which the service greets a client the headroom for everything a commit needs afterward is already reserved;
the band between "starts" and "lands" that the sweep needs is empty.
This is a choice for the owner or the coordinator (the brief says the gate must pass it),
with a ranking in the report that closes this step:
B (the sweep accepts an empty band,
because the control `a_service_that_ends_before_it_listens_leaves_a_recoverable_repository` now in main proves the below-start case) >
A (move the exhaustion to a resource T10 added a refusal path for,
a blocking-pool thread refused under `pids.max`,
with a positive control that the refusal fires) >
C (shrink the reserve until a band exists,
which weakens the service under exhaustion for the sake of a control).
The merge of main into the branch was not re-gated:
the host runs of the fault controls and the connection control on the merged tree are the evidence (9 of 10 pass,
the one above fails).

Latency (`native:perf`'s own tool,
three alternating rounds,
`${HOME}/temp/agent/t10-perf/`),
added median of a commit through the service:
before 18.4,
17.5,
17.2 ms (spread 1.2 ms),
after 18.9,
18.7,
18.7 ms;
p95 of the added time 26.8,
22.5,
26.5 before and 22.3,
27.1,
22.5 after;
signed commit 24.4,
23.9,
23.7 before and 23.5,
24.8,
23.4 after.
Forwarded commands (`rev-parse`,
`status`,
`log`,
`diff --stat`) added 1.2 to 1.7 ms before and after,
inside their spread.
After minus before is 1.05 ms on the mean of the added medians (18.8 against 17.7),
against a same-binary spread of 1.2 ms in the before runs,
so the difference is within the spread and not a measured regression.
A candidate cause,
not measured:
the instance check's path resolution now hops to the blocking pool before the request is queued,
and the first connection pays the start of a pool thread.
The bar (every paired delta far under five seconds) holds.

#### Step T10, the worker cap, the sweep, the refusal paths and the bounded stop, 2026-11-08T04:00:00-05:00

The coordinator's rulings on the three picks and the owed control,
applied on the same branch after main (W3 included) was merged into it.

Two answers first.
An existing fault-point suite does kill the service at each landing step and prove recovery:
`service/service_fault.rs` arms a `SIGKILL` at `after-objects`,
`after-intent`,
`after-ref`,
`after-index` and `before-ack`,
and at two instants of the index lock (`lock-created-unmarked`,
`lock-marked-unsynced`),
and `binary/binary_service_replay_tests.rs` and `binary_fault_trace_lock_without_links_tests.rs` restart the service and check the repository.
Those prove recovery from an end at a named point.
An allocation failure ends the process with an abort at any instruction,
and in a Git program the landing starts,
which no named point reaches;
the address-space sweep samples that.
And "9 of 10 pass" meant 9 of the 10 tests the filter `fault_limits connections_held_open` selected,
not 9 of 10 runs of the one test:
the one failing test was run once.

- The sweep lowers the running service's `RLIMIT_AS` with `prlimit` (`rustix::process::prlimit` exists in the locked 1.1.x;
  the control uses the test support's `LimitKind::lift_for`,
  which calls `prlimit`),
  after the service has greeted.
  The first design,
  the mapped size plus a headroom from none up to the reserve,
  never stopped a commit (every headroom landed):
  the service's heap is mostly free by then,
  and the Git programs it starts inherit a limit far above what they need.
  The sweep now sets the limit to each of eleven parts of the mapped size plus the reserve.
  The first limits that bite are the ones under which the Git programs of the landing cannot map their own space;
  whether the service's own allocation also fails in some of them was not measured.
  The test keeps `refused > 0` and `landed > 0` and checks recovery after every limit
  (`a_memory_limit_close_to_exhaustion_on_the_service_leaves_a_recoverable_repository`,
  passed in the gate below).
  Main's `a_service_that_ends_before_it_listens_leaves_a_recoverable_repository` keeps the below-start case.
- The refusal paths of a connection are controlled with a system that refuses every thread
  (`service/service_server_connection_handling_tests.rs`,
  each in a child with no thread to start):
  a message about a Git directory is answered `Unavailable` (with a positive control that the same message is `Rejected` where threads exist),
  a withdrawal that the pool cannot decide reaches the landing thread's queue,
  and a beat's note is left out.
  The existing `a_commit_service_at_its_process_limit_fails_plainly_and_serves_again_when_it_is_raised` lowers `RLIMIT_NPROC` on the running service and ran in the gate (not skipped).
- tokio's `spawn_blocking` panics with "OS can't spawn worker thread" when no thread can start and none is idle.
  Every blocking call of the service and the landing goes through `run_blocking_job`,
  which catches that panic (`foundation/blocking_job.rs`);
  a search of `service/`,
  `landing/` and `foundation/` finds no other `spawn_blocking`,
  `block_in_place` or `tokio::fs`.
  The search found one way in that the code did not control:
  hyper-util's own resolver calls `tokio::task::spawn_blocking` inside its `call`,
  in the standards-review call that the service's work loop drives with `block_on`,
  so the panic would have unwound into the work loop's thread.
  The call now uses `PoolResolver` (`standards_review/standards_review_client_name_lookup_on_the_blocking_pool.rs`),
  which looks the name up through `run_blocking_job` and gives an error;
  its control is the lookup in a child with no thread to start.
- The worker cap is `SERVICE_WORKER_CAP` = 4 (`service/service_main.rs`,
  with the measurement beside it).
  Four rounds alternating release binaries that differ only in the constant,
  added median of a commit through the service:
  uncapped 19.0,
  18.4,
  18.4,
  18.5 ms (spread 0.6);
  2 workers 25.6,
  22.7,
  23.0,
  22.4 (4.9 above the uncapped mean,
  outside the spread);
  4 workers 18.4,
  18.3,
  18.1,
  18.7;
  8 workers 18.6,
  18.6,
  18.2,
  18.5.
  The signed commit and the forwarded commands show the same shape.
  An idle service has 6 threads (two of its own and four workers) with or without 32 more connections,
  where it had 27.
  The smallest address-space limit under which the service starts is 93 MiB at every cap and 27 MiB before T10:
  the cap does not move it;
  the runtime plan's 64 MiB reserve does (a plan whose workers do not fit leaves the current-thread stand-in,
  and a service with no worker thread ends with status 4).
  The 110 MiB the container gate reported is the same figure on the gate's host and build.
- `Drop` of a claim renewal waits for its task for at most `RENEWAL_STOP_CEILING`,
  which is `WORST_CASE_WITH_MARGIN` (about 18.1 s:
  the owner's three times the longest delay of a check and a rename seen,
  and a renewal is one rename).
  One renewal measured far less than a thousandth of that in its control.
  On the timeout the task is left behind,
  a warning is logged,
  and the claim expires by its lease;
  the rename the task finishes later finds the claim moved on and changes nothing,
  because a renewal renames only from the name it last wrote.
  The coordinator's suggested design (the task releases the claim itself on a stop message) was not needed for that:
  the existing settlement already finds a claim by its id and generation and asks again when a renewal renamed it meanwhile.
  The control keeps the blocking pool busy so the renewal's job waits for a thread,
  and shows the drop returns at the ceiling (red first:
  it waited until the pool freed).
  The running renewal moved to `landing/service_claim_renewal_running.rs`,
  which depends on the renewer and not the reverse (the file had passed the line limit).

#### Gate of T10 after the rulings

Container gate (`--cpus 8 --memory 8g --pids-limit 2048`) of the tree at `2c88301e85`,
which has main at `f981e1b493` (W3) merged in:
5344 run,
5343 passed (3 slow),
1 failed,
11 skipped;
host Clippy clean,
package lint 0 errors and 0 warnings (the closure check ran;
no `skipped` line).
The service-memory sweep,
the process-limit control,
the thread cap control and the bounded drop control passed.
The failure,
recorded as open and not rerun:
`commit_timing::a_commit_that_reaches_a_foreign_index_holder_late_is_attributed_to_that_wait`,
`assertion left == right failed: the record says what kind of holder the wait was for` with `left: "unproven"` and `right: "proven-alive"`
(the service's answer line named `lock_holder="Process 80482"` and `lock_found="foreign no-marker size=281 age_ms=8036"`,
`lock_wait_ms=4004`;
the wrapper's record carries the holder kind from the words the service sent,
and `proven-alive` needs the text `(proven by `).
The gate before the rulings (an uncapped runtime,
the same test) passed it,
so the cap,
the resolver,
the bounded drop,
the sweep and the main merge were all candidates;
the cause is found below ("The holder kind of the timing record").

Latency of the final binary against the one before T10 (three alternating rounds;
`${HOME}/temp/agent/t10-perf2/`),
added median of a commit through the service:
before 19.9,
18.4,
18.6 ms (spread 1.5),
after 19.7,
19.0,
18.2 ms (spread 1.5);
signed commit 31.1,
30.3,
31.0 before and 30.2,
30.1,
31.5 after;
forwarded commands 1.2 to 1.9 ms before and after.

#### The holder kind of the timing record: safety, cause and fix, 2026-11-08T05:00:00-05:00

The gate's one failure after the rulings,
`commit_timing::a_commit_that_reaches_a_foreign_index_holder_late_is_attributed_to_that_wait` (`unproven` for `proven-alive`),
was diagnosed by instrumentation and bounded single-test runs,
not by gate reruns.

- Safety:
  the failing value is only the attribution in the wrapper's timing record.
  `lockHolderKind` is set in `service/service_client.rs` from the text of the waiting notices (`holder_kind_of`) and no remediation reads it.
  The service's own look (`look_at_holder`,
  `landing/service_index_attempts.rs`) answers `Proved::Alive`,
  `Dead` or `Nothing`;
  `Nothing` and `Alive` are waited for until the ceiling and then refused with `IndexInUse`;
  `Dead` ends the wait with a failure and removes nothing.
  The only removal is `reclaim_if` (`landing/service_index_reclaim.rs`):
  a lock with no marker returns `Kept` at `marker_of_file`,
  and a marked lock goes only when its marker's process identity judges it stale
  (a foreign or earlier-format lock is told to the person and left).
  So `unproven` can neither remove a lock a live process holds nor step over it:
  not a P1.
- Not starvation of the look:
  the service's answer line of the failing gate run named `lock_holder="Process 80482"` with `lock_looks=1`,
  so the one look returned `ProvenAlive`,
  and in every instrumented run under "Counts of the one test under the gate's limits"
  the only holder notice was proven.
  The look did not reach its ceiling in any run.
  The pool's busy count was not recorded,
  because no look waited on the pool.
- Cause,
  confirmed:
  the client overwrote `trace.holder_kind` on every waiting notice and every inside-a-write notice,
  so the last notice decided the kind.
  The control releases the holder 4 s after the first notice,
  which is the same instant as the 4th heartbeat of the connection (beats at +1.0,
  +2.0,
  +3.0 and +4.0 s after the first notice).
  The beat asks the board for a note,
  and when the landing has just begun its first write the answer is an inside-a-write notice,
  whose text has no `(proven by `:
  it arrives after the proof and the kind becomes `unproven`.
  A message writer that logged each message and each landing write showed it in each failing run:
  one proven waiting notice,
  then a heartbeat,
  and an inside-a-write notice with `proven=false` at +4005 or +4006 ms,
  on the 4th beat;
  no run without that notice failed.
- Counts of the one test under the gate's limits (the host test binary in a scope with `CPUQuota=800%`,
  `MemoryMax=8G`,
  `TasksMax=2048`;
  the gate image takes no test filter,
  so the image was not used),
  20 runs each:
  cap 4,
  3 failed (run 3,
  8,
  19,
  without the message log);
  cap 4 with the message log,
  2 failed (both with the inside-a-write notice at the 4th beat);
  uncapped,
  1 failed (the same notice at +4005 ms).
  So the cap is not the cause.
  Whether T10 raised the rate over the tree before it was not measured:
  the coincidence of the beat and the release is in the old writer as well,
  and only its offset (a few milliseconds of task and pool hops against a thread) differs.
- The other fields of the timing record:
  `waitingNotices` counts and `positions` appends;
  `last_blocker` and the holder's words are last-wins by design (the error names the latest blocker),
  and `lockHolderKind` was the only last-wins field of the record.
- Fix,
  test first (`binary_service_heartbeat_tests.rs`:
  a proven holder followed by an inside-a-write notice,
  or by a notice that names no proof,
  stays `proven-alive`;
  an unproven wait that a later notice proves is `proven-alive`;
  an inside-a-write notice alone records no kind;
  three of the four were red):
  `holder_kind_after` never undoes a proof (`commit/commit_timing.rs`),
  and an inside-a-write notice no longer sets the kind (`service/service_client.rs`).
  No capacity was reserved for the look,
  because nothing starved it.

#### Gate of T10, third and last, 2026-11-08T05:40:00-05:00

Container gate (`--cpus 8 --memory 8g --pids-limit 2048`,
image tag `t10-liveness`) of the tree at `4e3bde33c0`:
5349 run,
5349 passed (3 slow),
11 skipped,
no failure;
host Clippy clean;
package lint 0 errors and 0 warnings (the closure check ran,
no `skipped` line).
Latency of the final binary against the one before T10 (three alternating rounds,
`${HOME}/temp/agent/t10-perf3/`),
added median of a commit through the service:
before 20.4,
20.0,
19.1 ms (spread 1.3),
after 20.6,
20.4,
19.4 (spread 1.2);
signed commit 27.2,
26.3,
25.6 before and 25.5,
25.4,
25.7 after;
forwarded commands 1.0 to 1.9 ms either way.

### Step T11, a stop ends the Gits the service waits on, and the service queues on tokio, 2026-11-07T23:40:00-05:00

Branch `feat/git-policy-cli-tokio-t11`,
from main at `d4eb891f37`,
main merged in at `7c3cd01c49` (clean).
The owner's pick for choice 12 ("C:
shutdown stops its Gits (Recommended)",
2026-11-07T22:19:43-05:00) is built.
The loops of the service (the landing loop,
the work loop),
their sleeps,
the inotify wait and the client's and the provision's sleeps are not converted:
they wait for the conversion of the Git leaf (see "What T11 did not convert,
and why").

#### The design of choice 12

- The handler only records the stop (`service/service_signal.rs`):
  it stores the signal in an atomic (`process/stop_of_the_service_noticed_by_its_children.rs`),
  writes the exit line as before,
  writes one byte to a non-blocking pipe,
  and arms `alarm` for the last resort.
  A second stop signal ends the process at once by the default action.
  A process that could not make the pipe ends in the handler as before.
- The service's own shutdown is a task of the runtime (`service/service_server_shutdown_after_a_stop.rs`):
  it reads the byte,
  raises a level that every waiter holds an arm of,
  waits for the unfinished work to reach zero for at most `SETTLE_CEILING`,
  and ends the process by the recorded signal
  (default disposition and `raise`,
  so the exit still reads as "killed by SIGTERM").
  It starts only in a process that prepared to notice a stop:
  the first version of this commit ended the in-process services of the controls
  (three `hung_write_tests` failed in the first container gate with exit code 1 and no text;
  the fix and a control are in the branch).
- No table of children (owner:
  "Use as few locks as possible."):
  every wait owns its child and ends it.
  The watched steps (`output_watched_within`,
  that is every Git the landing runs) end their child in `watch_loop`,
  through `stop_process` (the grace,
  the kill after it,
  the sweep of the locks the child left),
  the push and the query (`bounded_push.rs`,
  `bounded_query.rs`) look at the arm at each of their existing ticks and stop with the existing `stop_push`,
  and the Git step of a commit under a signal guard ends in `process/child_awaited_until_it_exits_or_the_service_is_stopped.rs`.
  The shutdown only counts the work in flight (an atomic,
  raised before the stop is read,
  so a start is either refused or counted).
- A step begun after the stop is refused,
  except a step the landing thread starts while it unwinds:
  that is how the commit's work is kept.
  A refused push is answered as a stopped push (the queued line,
  retried by the next service),
  a refused step as `EndedByStop`.
- The landing treats a stopped step like a step ended at its ceiling
  (`service_land_git_steps_and_import_stream.rs`,
  `run`):
  `StepTooSlow`,
  the sentence kept for the clients,
  the built commit kept on `refs/cli-git/kept/<n>` by the existing `keep_unlanded`,
  the scratch branch removed,
  and the service's own `index.lock` given up when the lock guard drops.
  The loop lands nothing more once the stop is recorded;
  the work loop starts no round.
- Bounds (each beside its measurement or the owner's figure):
  `SETTLE_CEILING` = `STOP_CEILING` (3 times the longest delay measured,
  the owner's "3x worst case") plus `STOP_GRACE` plus one second for the unwinding,
  24.096 s;
  `LAST_RESORT` (the alarm) = `SETTLE_CEILING` plus `STOP_GRACE`,
  29.096 s;
  the manager's default stop timeout was taken to be 90 s and this crate's unit set none (wrong,
  see the "Owner answer of 2026-11-08 and what it changed" section:
  the host's default is 15 s,
  and the unit now sets its own).

#### Controls

Red first (`9ff3110660`):
`binary_service_stop_ends_its_gits_tests.rs`,
a real service with a stand-in Git that stalls at the branch move with the service's lock held,
`SIGTERM` sent to the service.
Before the wiring three of four failed (`no Git is left running after 10 ms`,
`ended after 10 ms` for the Git that ignores `SIGTERM`,
and no commit on a side ref);
the idle one passed.
After the wiring all four pass:
no Git and no `index.lock` within the bound,
the commit on `refs/cli-git/kept/<n>` and the branch unmoved,
a Git that ignores `SIGTERM` killed after `STOP_GRACE` and before `SETTLE_CEILING` (6.0 s in the run),
an idle service ended at once by the signal.
Unit controls:
the stop module (counting,
refusal and its count,
first signal only,
arm told only when the shutdown tells,
the settle bound reached,
the bounds below the manager's timeout,
noticing only after preparing),
a push,
a query and a commit step running when the stop is told
(ended,
queued or reported as ended by the stop),
a push begun after the stop not started,
and the queue wait.
Planted control:
the arm of every piece of work removed (`begin_unfinished_work` giving none) failed the arm control at once and the push and commit-step controls only at their 60-second child (reverted).
The existing control that the signal guard does not relay a service's stop to a commit's child is kept (the relay still does not;
the shutdown ends the Gits).
One failure found by an existing control:
the shutdown logged through standard error,
which blocks on a full pipe,
and `a_service_stopped_while_standard_error_is_a_full_pipe_still_exits` failed (10 s);
the shutdown now writes its one line without waiting (`send_without_waiting`).

#### Owner answer of 2026-11-08 and what it changed

The coordinator put the first judgment call in the `Judgment calls open to the owner's veto` section
to the owner (question tool,
verbatim answer "B:
let a branch write finish (Recommended)").
The option read:
"A stop ends every other Git at once,
but waits for a running branch write up to the stop limit.
systemd signals only the service itself (KillMode=mixed) and kills what's left only at that limit."
It replaces the first judgment call.
Built on `fix/git-policy-cli-stop-lets-branch-write-finish`:

- The landing's branch move (an `update-ref` that is not a side-ref write,
  `is_branch_write` in `transaction_git_step_rerun_and_end_rules.rs`) takes no stop arm
  (`output_watched_within_for_a_stop` in `push/child_watch.rs`):
  its count stays,
  so the shutdown waits for it up to `SETTLE_CEILING`,
  and nothing signals it.
  Every other Git of the landing,
  and every push,
  query and commit step,
  is ended by the stop as T11 built it.
  The import of the commit's objects (`fast-import`) is also a step that is never ended on the landing thread,
  and it is still ended by the stop:
  only the branch write waits.
- The service unit sets `KillMode=mixed` and `TimeoutStopSec` from `STOP_TIMEOUT_SECONDS`
  (`LAST_RESORT` plus one `STOP_GRACE`,
  whole seconds rounded up;
  35 s with the bounds of T11),
  so the manager signals the service alone,
  kills what is left when the service exits or at the timeout,
  and never uses the host's default.
  The margin of one grace is the coordinator's pick;
  the owner's veto is open.
- A stop during a branch write that finishes lands the commit
  (`a_stop_during_a_slow_branch_write_lets_it_finish_and_lands_the_commit`).
  A stop during one that never ends makes the service end at `SETTLE_CEILING` by the signal,
  with the Git still running (nothing here signals it;
  the manager's final kill ends it),
  the branch unmoved,
  the commit on its scratch branch `refs/worktree/cli-git-service/<seq>`
  and the open intent for recovery
  (`a_stop_during_a_hung_branch_write_ends_the_service_within_the_bound_and_keeps_the_commit_on_its_side_ref`).
  The commit is not copied to `refs/cli-git/kept/<n>` in that case,
  because the landing is still inside the write when the service ends and cannot unwind;
  recovery reads the open intent as it does after any kill.
  Whether the commit should also be copied to a kept ref at the limit is decided (2026-11-08,
  branch `fix/git-policy-cli-kept-ref-for-a-branch-write-past-the-stop`):
  it is,
  by the recovery that settles the batch,
  never by the stop.
  What that touched was read first:
  a start of the service does not settle its own journal
  (only an earlier release's shared journal is settled at a start),
  the next batch settles it,
  and that drop left the commit on the landing's scratch branch alone,
  which a commit Git built never has.
  The drop now writes `refs/cli-git/kept/<n>` before the intent goes
  (`keep_what_the_branch_does_not_show` in `service_land_finish.rs`),
  it is a side-ref write,
  so a stop cuts it like any other Git of a landing,
  and a drop whose kept write fails is logged with Git's words and still drops,
  so a ref nobody can write does not leave every later commit waiting on the batch's index lock.
  The stop waits for nothing more than it did,
  so the ref is named once the worktree commits again.
  Main's own sweep of a dropped batch's scratch reference came in the merge (`f293f8f58d`,
  `service_scratch_ref_of_a_dropped_batch.rs`),
  so the drop keeps this commit and then removes that reference;
  a kept reference that already names another commit is never replaced,
  and the scratch reference then stays too.
  Controls,
  red first (`70d04ae08a`):
  `a_restart_keeps_the_commit_of_a_batch_whose_branch_never_moved_on_a_kept_ref`,
  `a_restart_keeps_a_commit_git_built_that_landed_no_ref_on_a_kept_ref`
  and
  `the_commit_of_a_branch_write_that_outlived_the_stop_is_kept_on_a_kept_ref_by_the_next_commit`.
- The measured host facts behind the unit:
  the live unit had `KillMode=control-group` and `TimeoutStopUSec=15s`
  from `DefaultTimeoutStopSec=15s` in `/etc/systemd/user.conf.d/timeout.conf`,
  both below `SETTLE_CEILING` (24.096 s) and `LAST_RESORT` (29.096 s),
  so the manager would have killed the service in the middle of its cleanup.
- Existing services pick up the new text at the next provisioning
  (`in_place` reads the installed template and finds it different,
  so `provision_with` rewrites it and asks the manager for a reload).
  A reload is needed and the existing mechanism does it.
  Probed on this host with a throwaway unit in the runtime unit directory:
  after the file was rewritten and `systemctl --user daemon-reload` ran,
  the running service kept its process and `systemctl --user show` gave the new `KillMode` and `TimeoutStopUSec`
  (`control-group` and 15 s before,
  `mixed` and 35 s after).
- The control `the_bounds_end_before_the_units_own_stop_timeout` replaces the one that compared the bounds with an assumed ninety seconds.
  The three controls of the first version that stalled the branch write
  (`ends at the first SIGTERM`,
  `keeps its work on its side ref`,
  `ignores SIGTERM`) now stall the import,
  or became the two controls above:
  a stop no longer ends a stalled branch write,
  by the owner's answer.

#### Judgment calls open to the owner's veto

- Superseded on 2026-11-08 by the owner's answer in the `Owner answer of 2026-11-08 and what it changed` section:
  a stop ends a branch write too (the landing's `Unbounded` step),
  with `SIGTERM` first,
  the kill after `STOP_GRACE`,
  and a wait past the grace while a member is in uninterruptible sleep.
  The owner's 2026-11-04 "Never cut a branch write (Recommended)" was about the wrapper's own ceilings while the service lives;
  choice 12 says the service's stop stops the Gits it waits on.
  A cut `update-ref` leaves its lock to the sweep and the branch where it was,
  the batch's intent stays open and recovery reads it,
  and the commit is on its side ref.
- A landing that waits for a foreign index lock (`service_index_attempts.rs`,
  another agent's file) is not woken by the stop:
  its group is unfinished work,
  so the service ends at `SETTLE_CEILING` at the latest.
- The stop does not drain the queued durability syncs (as before;
  a signal ended the service at once).
- A client whose group was stopped gets `StepTooSlow` when its connection is still open when the process ends,
  and a closed connection otherwise;
  the commit is on its side ref either way.

#### What changed for the queues

The two queues of the service,
the landing's requests and the after-commit wakes,
are `tokio::sync::mpsc` unbounded channels
(`service/service_server.rs`,
`service_server_connection_handling.rs`,
`service_server_work_loop.rs`);
a task sends without a thread,
and the loops that read them wait on the runtime's own timer and wake through `service/service_queue_received_on_the_runtime.rs`
(an item,
the bound,
or the end of the queue).
`std::sync::mpsc` is gone from the service's two queues.
It remains in `landing/service_durable.rs` (the barrier),
`landing/service_claim_renewal_running.rs`,
`commit/native_commit_waiting_notice.rs`,
`push/child_watch.rs` (the left-running waiter),
`standards_review/standards_review_run.rs`,
`markdown/markdown_linter_guard.rs` and `foundation/lazy_runtime.rs`.

#### What T11 did not convert, and why

- The landing loop,
  the work loop and their sleeps stay threads.
  Every Git step they run goes through `run_git` and its siblings,
  which are synchronous and enter the runtime through `run_on_runtime`,
  and that bridge refuses a call from inside the runtime (a blocking-pool thread included:
  `Handle::try_current` is `Ok` there).
  A task that ran a round would meet `RuntimeError::Nested` at its first Git.
  Counted on this tree:
  79 production files name `run_git`,
  `run_git_checked`,
  `output_watched`,
  `output_of_blocking` or `run_on_runtime`,
  and `run_git` or `run_git_checked` has 203 production mentions.
  The order that works is leaf first:
  an `async` `run_git` beside the synchronous one,
  then `transaction_git` callers one folder at a time,
  then the landing's steps,
  then the loop.
  Letting a blocking-pool thread call the bridge was judged and not done:
  it needs a thread-local marker for "this thread is a pool job"
  and it removes the refusal that has found real defects.
- The remaining production sleeps and bounded receives
  (`rg 'thread::sleep|park_timeout|recv_timeout'` outside tests and `binary/`):
  `landing/service_index_hold.rs:147`,
  `landing/service_index_attempts.rs:300` and `landing/service_index_wait.rs:118`,
  `:309`,
  `landing/service_land_follow.rs:280`,
  `lock/index_holder_wait.rs:236`,
  `push/auto_push.rs:278`,
  `push/start_retry.rs:200` (the forwarded command's start pause,
  by design),
  `transaction/transaction_phase.rs:391`,
  `transaction/transaction_git.rs:162` (`run_git_pausing`),
  `service/service_client.rs:290`,
  `service/service_client_reach_service.rs:137`,
  `:204`,
  `service/service_provision_earlier_service_retirement.rs:243`,
  `commit/native_commit_waiting_notice.rs:160`,
  `standards_review/standards_review_run.rs:533`,
  `:543`,
  `markdown/markdown_linter_guard.rs:222`,
  `process/thread_quiet.rs:299` and `foundation/name_held_while_one_thread_publishes_it.rs:189`.
  The client and provision sleeps belong to the synchronous commit command and the provisioning,
  which become `async` with the commit command (T11 as the plan wrote it),
  and the inotify wait (`landing/service_index_watch.rs`) belongs to the landing's lock wait (`service_index_attempts.rs`,
  which the signed-ceiling branch edits).
- Files the signed-ceiling branch edits were not touched:
  `landing/service_index_reclaim.rs`,
  `landing/service_index_attempts.rs`,
  `commit/native_commit*`.

#### Gate of T11

Container gate (`--cpus 8 --memory 8g --pids-limit 2048`,
unit `t11-gate2`,
log `${HOME}/temp/agent/t11/gate2.log`) of the tree at `fc12edecb4`,
main merged in:
5371 run,
5371 passed (3 slow),
11 skipped,
no failure;
container Clippy clean;
package lint (host) 0 errors and 0 warnings (the closure check ran,
no `skipped` line).
The first gate of the branch (`16bb774ce3`,
5370 run,
3 failed) is the shutdown that ended in-process services (above).
Host:
the lib suite minus the two askpass controls (they need `GIT_ASKPASS` unset,
the owner's rule sets it empty) 4640 passed.
Latency (`native:perf`'s own tool,
three alternating rounds against the binary before T11,
`${HOME}/temp/agent/t11/perf/`;
the container gates ran on the host through all of it,
which the alternation covers),
added median of a commit through the service:
before 19.6,
19.9,
23.6 ms (spread 4.0),
after 22.4,
20.3,
26.0 ms;
after minus before is 2.8,
0.4 and 2.4 ms by round,
mean 1.9 ms against the same-binary spread of 4.0 ms,
so within the spread and not a measured regression;
signed commit 28.4,
28.1,
30.3 before and 30.0,
28.3,
35.4 after;
forwarded commands 1.4 to 2.2 ms either way.
Two before runs of the earlier (idle host) pair gave 20.6 and 20.1 ms added.
The bar (every paired delta far under five seconds) holds.
The after binary was built before the fix of the shutdown's start;
the fix changes nothing a commit through the service runs.

### Step T12, the runner's jobs on the runtime and one bound for the process, 2026-11-08T18:40:00-05:00

Branch `feat/git-policy-cli-tokio-steps-12-and-13`,
from main at `1a907ccfb9`.
Two commits:
`2b1f830568` (the bound and the runner) and
`c1e6a78a4c` (the lint and Clippy fixes).

#### Where the plan and the code disagree, told as the coordinator asked

The plan's T12 reads "the parallel runner and the scoped threads ... become tasks and `spawn_blocking` jobs under
the semaphore of the round;
the nine `HostResources::read` call sites read the budget through the new owner".
The code does not allow that as one step,
for two reasons:

- A job of the blocking pool must own what it reads (`spawn_blocking` requires `'static`),
  and the scoped threads of
  `parallel_runner.rs` and `side_by_side_threads.rs`
  borrow the caller's items,
  lanes and work for the length of one thread scope.
  So every call site must hand over shared values (`Arc`)
  instead of borrows,
  and the seven `run_ordered` call sites
  (`worktree/worktree_convert_command.rs`,
  `worktree/worktree_trash_entries.rs`,
  `push/manual_push_probe.rs`,
  `push/manual_push_lifecycle.rs`,
  `subvolume/subvolume_index_refresh_of_every_worktree.rs`,
  `landing/service_round.rs`,
  and the controls) plus the nine `run_all_side_by_side`/`run_beside_caller` sites must change that way,
  one at a time,
  since a change of the runner's signature reaches all of them at once.
- The work of `landing/service_round.rs` runs Git:
  `drain_directories` calls
  `drain_planned` → `run_due_push_retries` and `drain_pending` → the `WorkHandlers` of the landing
  (`landing/service_work.rs`),
  and those start Git children.
  A pool job reaches `run_git`,
  which enters the runtime through `run_on_runtime`,
  and that bridge refuses a call from inside the runtime (`RuntimeError::Nested`).
  So the round's conversion needs the leaf-first `async` Git that T11 named and did not build:
  there is no `async fn run_git` in `transaction/transaction_git.rs` today
  (T11 left the note in
  "What T11 did not convert,
  and why";
  79 production files and 203 mentions).

So this step is the part that needs neither:
the two pieces every later call-site conversion rests on.

#### What landed

- `foundation/blocking_jobs_within_the_budget.rs`:
  the process's one bound on blocking jobs,
  a `tokio::sync::Semaphore` made once,
  sized by
  `runtime_plan::budgeted_workers` (budget ∩ `RLIMIT_NPROC`/`pids.max` ∩ address space)
  and by `hold_blocking_jobs_to`,
  whose permits a job takes before the pool is asked for a thread.
  The size is the same derivation the runtime's worker count uses
  (`runtime_plan.rs` now has one function for both),
  so more jobs than threads cannot run.
- `foundation/parallel_runner_on_the_runtime.rs`:
  the same ordered run as `parallel_runner.rs` (input order,
  the live lines of the head item,
  a lane's items one after another)
  with each item as one job of the pool under that bound,
  the consumer on the calling task,
  and `run_ordered_and_wait_here` as the face a caller that is still synchronous uses
  (it enters the runtime and refuses from inside it).
  The reporting is shared with the scoped runner:
  `Cursor`,
  the message type and the lane-coverage check are `pub(crate)` in `parallel_runner.rs`,
  so the two cannot report differently.
- `service/service_main.rs`:
  the service holds its blocking jobs to `SERVICE_WORKER_CAP`,
  as it holds its workers.

The seven call sites are not converted;
the scoped runner stays beside the new one until they are.
The nine `HostResources::read` sites still read the budget themselves;
the bound is what now limits how much of it runs at once.

#### A refused `spawn_blocking` swallows the job, measured

The design of a permit was changed by a measurement,
not by a preference.
When the system refuses a thread,
`spawn_blocking` pushes the job into the pool's queue and *then* panics
(`tokio-1.54.1/src/runtime/blocking/pool.rs`,
`spawn_task`:
the queue push precedes the thread start,
and `Err(NoThreads)` leaves the task queued with no thread to run it).
The queued job keeps everything it captured for the life of the process,
so:

- a permit moved into the job is lost for good.
  Planted control (the permit put back into the closure):
  `where_no_thread_can_start_the_jobs_fail_plainly_and_give_their_places_back` failed with
  "the place of a job refused a thread was never given back:
  Elapsed(())" after 30 s.
- a sending end moved into the job never drops,
  so a consumer that ends when the channel closes never ends.
  Planted control (the consumer reading until the channel closed):
  `where_no_thread_can_start_the_run_is_a_plain_error` hung to nextest's 300 s timeout
  (`TIMEOUT [ 300.007s]`).

So the place is held by the awaiting frame
(no shipped caller stops waiting for one of these jobs;
the doc beside it says so),
and the consumer ends when every job has *returned*,
counted by `SlotsStillRunning`,
and then drains what the jobs already sent.
Both controls pass in 0.016 s and 0.025 s once the design carries this.

#### Controls

`foundation/blocking_jobs_within_the_budget_tests.rs` (5):
`a_job_gives_its_value`,
`the_budget_is_at_least_one_place_and_a_cap_holds_it_lower`,
`no_more_jobs_run_at_once_than_the_budget_allows` (a burst of six held to the two places it asked for;
reports the most places held at once),
`a_job_that_panics_gives_its_place_back`,
`where_no_thread_can_start_the_jobs_fail_plainly_and_give_their_places_back` (the child that is refused every thread,
two jobs,
both refused,
the job never ran,
no thread started for the second wait).

`foundation/parallel_runner_on_the_runtime_tests.rs` (5):
`every_item_runs_and_the_report_keeps_input_order` (the first item ends last and the report keeps input order),
`items_run_at_the_same_time` (a rendezvous of the jobs the runner promises at once;
a serial run can never pass it),
`the_process_bound_holds_the_items_to_one_at_a_time`,
`where_no_thread_can_start_the_run_is_a_plain_error`,
`a_lane_set_that_leaves_an_item_out_is_refused`.

#### Checks

`mise run //package/git-policy/cli:native:test:host -- foundation::blocking_jobs_within_the_budget foundation::parallel_runner foundation::parallel_runner_on_the_runtime foundation::runtime_plan`:
27 tests run,
27 passed.
The scoped runner's 12 controls and the side-by-side,
scoped-start and blocking-job controls ran unchanged in an earlier run of the same filter (37 tests run,
37 passed before the two no-thread fixes below landed as they are).
`native:clippy` clean;
package `lint` 0 errors and 0 warnings with the closure check run.
Not run:
the container gate (it builds the gate image;
the tree has not been through it),
`native:perf` (the plan asks for it after T12),
and the mutation campaign.

#### The overlap control expects the runner's bound, not a typed four, 2026-11-08T20:30:54-05:00

`items_run_at_the_same_time` asked one run for four workers and expected four items at a single point.
The jobs of this runner are bounded by the process's bound on blocking jobs,
which `budgeted_workers` sizes from `available_parallelism`,
so a container's CPU quota lowers it:
measured in the test's own cgroup,
`CPUQuota=200%` gave `available_parallelism` 2 and `budgeted_blocking_jobs()` 2,
and a probe of four items kept at most 2 inside their job at once (51.6 ms in two waves),
against `available_parallelism` 6 and a bound of 5 (26.8 ms) at the session's 600% allowance.
The old control failed there in 10.011 s with `left: 2` against `right: 4`,
which is the two items that did start waiting out `PATIENCE`.
So the control was wrong and not the runner:
the runner says in its module doc that it runs its items under that bound,
and its `workers` field names the bound now too,
where the typed four had read as a guarantee.
The control derives the count from the runner's promise (`jobs_that_may_overlap`:
the workers it asks for,
the lanes and `budgeted_blocking_jobs()`) and runs exactly that many items at a rendezvous of that size;
where the bound is one it prints why it has nothing to measure and returns,
because one job at a time can never meet another.
Planted control:
with `at_once` in `run_ordered_on_the_runtime` set to one,
the control fails in 10.016 s with `left: 1` against `right: 2`,
so it still catches a runner that runs its items one after another.
Both allowances:
`mise run //package/git-policy/cli:native:test:host -- foundation::parallel_runner_on_the_runtime`
passed 9 of 9 at `CPUQuota=200%` and at the session's 600%,
with `items_run_at_the_same_time` in 0.007 s and in 0.009 s.
Commit `520d8991c2` on `fix/git-policy-cli-parallel-runner-control-holds-at-any-cpu-count`,
with an independent review's corrections to three sentences that claimed more than the bound guarantees
in `98f51f9d37`:
the bound is a ceiling every job of the process shares and not a reservation one run keeps,
and it comes from the CPU allowance with the thread,
address-space and cap limits.

### Step T12, the leaf-first asynchronous Git and the ordered runner that awaits, 2026-11-08T19:29:56-05:00

Branch `feat/git-policy-cli-tokio-steps-12-and-13` in
`.claude/worktrees/pi-tokio-async-git-leaf`,
from `6240b51a7a` (the merge of "Step T12,
the runner's jobs on the runtime").
Two commits:
`e0359dc84c` (the asynchronous Git runner and the awaited watched
run) and `a2f901dd92` (the ordered runner for work that awaits a Git child).

#### What the previous step left, and what this one had to build first

The section "Step T12,
the runner's jobs on the runtime" ends with one leaf
missing:
`transaction/transaction_git.rs` had no `async fn`,
so the round of the landing could not move onto the runtime runner,
because a
pool job that reaches `run_git` meets the refusal of
`run_on_runtime` (`RuntimeError::Nested`).
This step builds that leaf and the runner the round needs after it.

The synchronous face of a Git step enters the runtime for one reason:
a step that is ended at its ceiling (`StepBound::EndAt`) registers its pipes
with the reactor,
and `attach_pipes` does that through
`run_on_runtime`.
Everything else of a step is already a wait of the
synchronous caller's thread or of the watcher task.

#### The two faces, and why both exist

- `transaction/transaction_git_awaiting.rs` is the same runner with the
  entering taken out:
  the pipes are registered on the runtime the caller
  already runs on,
  the child is started by `spawn_retrying_without_blocking`
  (the awaiting start retry that already existed),
  its output is read by
  `StepPipes::read_until_the_child_ends_awaiting`,
  and its end is awaited
  through the pidfd of `OwnedChild`.
  `run_git_awaiting` and `run_git_checked_awaiting` are its two entries.
- The synchronous face stays exactly as it was,
  and for a measured reason:
  it is not only the callers that are still synchronous that need it.
  A step
  whose bound is `Unbounded` or `LeaveRunning` makes no pipes at all,
  so the
  synchronous face works today from inside the runtime for those bounds,
  which the branch write of a landing is.
  A bridge that entered the runtime
  around the whole runner would have taken that away from every pool job that
  moves a branch,
  so the leaf is added beside the face and not under it.
- What the two cannot differ in is shared rather than copied:
  `build_command`,
  `input_file`,
  `step_subject`,
  `terms_for_request`,
  `step_bound`,
  `checked_from` and `refused_by_the_system` are the synchronous file's items,
  made `pub(crate)` for the awaiting file;
  `ending_of_a_watched_step`
  (`push/child_watch_step_bounds_and_errors.rs`) words the stop,
  the ceiling
  and the quiet window once for both;
  `watched_timeout_text` moved beside it
  so a file that words an ending does not use the file that runs the wait.
- `push/child_watch_output_watched_on_the_runtime.rs` is the watched run of a
  step as a future:
  `attach_pipes_awaiting`,
  the awaiting start,
  the pipe read
  and the child's end.
  A step that is left running (a write that may not be
  cut) is waited for by a task of the runtime with the runtime's timer,
  and
  the thread the synchronous face starts for that is not started here.

#### The place a worker was held, measured

The synchronous face closes a watch by dropping it,
and `Drop` waits for the
watcher task with `recv_timeout` on the calling thread.
That is right for a
synchronous caller and wrong on a worker:
while the workers of a small
runtime are inside those waits,
the very tasks that would end them cannot
run,
so each watch pays the whole `WATCHER_END_BOUND` (19.1 s).

Measured before the fix (`foundation/parallel_runner_on_the_runtime_awaiting_tests.rs`,
`an_item_may_await_a_git_child`):
four awaited Git steps of 3.6 ms each took
38.2 s,
and one awaited step alone took 19.1 s.
`ChildWatch::close_awaiting` closes the same watch from a task,
awaiting the
watcher's own end with the same bound,
and the four steps take 10.8 ms.
The synchronous face keeps its blocking drop,
which the measurement of
`WATCHER_END_BOUND` was made under.
The rule this gives any later helper:
a synchronous `Drop` that waits for a
task must not run on a worker,
so a face that runs on the runtime closes its
own watches.

#### The ordered runner that awaits

`foundation/parallel_runner_on_the_runtime.rs` runs each item as a job of the
blocking pool,
and a job cannot await anything,
so the round of the landing
could not use it whatever the Git leaf looked like.
`foundation/parallel_runner_on_the_runtime_awaiting.rs` runs each lane as a
task of the runtime instead (`run_ordered_awaiting_on_the_runtime`,
`OrderedAwaitingWork`):

- a lane's items run one after another inside one task,
  which is what keeps
  two items of one worktree apart,
  and lanes run side by side,
  bounded by
  `workers`;
- an item that also waits on storage puts that one call on the blocking pool
  inside its own work (`run_blocking_job_within_the_budget`),
  so the
  process's one bound still covers the storage and the worker awaiting Git
  stays free;
- the consumer,
  the drain of what the jobs already sent and the join are
  `finish_the_run`,
  shared with the blocking form,
  so the input order,
  the
  live lines of the head item and the counting of jobs that returned stay one
  code.
  The blocking form's behaviour is unchanged.
- `OrderedAwaitingWork::run` hands back a boxed future rather than being an
  `async fn`,
  because the work travels as `Arc<dyn OrderedAwaitingWork>` and a
  trait with an `async` method cannot be made into a `dyn` value.

#### What is not converted, and why that is not a gap of this step

No production call site was converted,
so no command changes behaviour.
Converting one call site makes its function `async`,
which makes its callers
either `async` or a bridge that enters the runtime,
and that chain runs up to
the command or to the service loop.
The 67 production files that call
`run_git` are therefore converted one folder at a time from the top of their
own chains,
and the round of the landing is the last of them:
its items run
the landing's handlers (`landing/service_work.rs`),
so every step below them
becomes a future first.

#### Controls

`transaction/transaction_git_awaiting_tests.rs` (7):
`a_query_awaited_on_the_runtime_gives_gits_answer`,
`an_input_awaited_on_the_runtime_reaches_git_and_comes_back_hashed` (a 4 MiB
input),
`a_failure_awaited_on_the_runtime_names_the_command`,
`a_branch_write_awaited_in_a_landing_task_moves_the_ref` (the `Unbounded`
bound,
from a task that carries a landing's state),
`a_run_leaves_no_descriptor_open`,
`the_synchronous_face_of_the_same_query_is_refused_from_inside_the_runtime`
(the positive control:
without it the first control could pass for a reason
other than the one it claims),
and
`a_branch_write_of_the_landing_asked_of_the_synchronous_face_inside_the_runtime_still_lands`,
which pins why the synchronous face was not turned into a bridge:
a step with an `Unbounded` bound makes no pipes,
and the pipes are the only reason that face enters the runtime.

`push/child_watch_output_watched_on_the_runtime_tests.rs` (5):
`a_step_awaited_on_the_runtime_returns_every_byte_of_both_streams` (a step
that writes beyond the kept tail on both streams),
`the_synchronous_face_of_the_same_step_is_refused_from_inside_the_runtime`,
`a_step_that_is_never_ended_is_awaited_to_its_own_end`,
`a_step_awaited_on_the_runtime_leaves_no_descriptor_open`,
`a_step_on_a_directory_of_its_own_runs_and_leaves_its_marker`.

`foundation/parallel_runner_on_the_runtime_awaiting_tests.rs` (4):
`every_item_runs_and_the_report_keeps_input_order`,
`items_run_at_the_same_time` (a rendezvous of four on a `tokio::sync::Barrier`),
`a_lane_runs_its_items_one_after_another`,
`an_item_may_await_a_git_child` (a real `git rev-parse` per item,
which is
also the measurement of the watch's drop above).

#### Checks

`mise run //package/git-policy/cli:native:check`:
clean.
`native:clippy`:
clean.
`native:test:host -- foundation:: parallel_runner:: transaction:: push::child_watch`:
335 tests run,
335 passed.
`mise run lint:monochromatic-lint` on every file this step touched:
clean,
after `format:monochromatic-lint` inserted the line breaks it asks for in
three of them.
The whole lib suite,
once,
at the end
(`native:test:host`,
with `CLI_GIT_TEST_ROOT` under a short path so a Unix
socket fits `SUN_LEN`):
4794 tests run,
4794 passed (one slow),
8 skipped.
The whole binary-level suite,
once
(`native:test:binary:host`,
same short root):
733 tests run,
733 passed (two
slow),
3 skipped.
The package lint (`//package/git-policy/cli:lint`,
`monochromatic-lint
--max-warnings 0` over the package plus Clippy):
0 errors,
0 warnings.
Two runs before those look like defects and are not:
- with a small `TasksMax` on the run's own control group,
  two LFS controls
  failed with "cannot fork() for git-lfs filter-process:
  Resource temporarily
  unavailable",
  which is the bound the run was given,
  not the code;
- without the short fixture root,
  `service_mode::the_same_start_with_the_real_git_listens` fails as
  `doc/handover/cli-git-rust-implementation.md` recorded on a clean checkout:
  its socket path under `CARGO_TARGET_DIR` is longer than `SUN_LEN`.
Not run:
the container gate (it builds the gate image and this tree has not been
through it),
`native:perf`,
and the mutation campaign (paused by the owner).

### Step T12, the first production folder on the awaited Git runner, 2026-11-08T21:05:00-05:00

Branch `feat/git-policy-cli-tokio-call-sites-from-the-top-of-each-chain` in
`.claude/worktrees/pi-tokio-convert-the-first-call-sites`,
from main at `bbadfac560`.
Two commits:
`a669e2a59a` (the control that fails to compile) and
`ae4e1a57c7` (the conversion that turns it green).

#### Which folder, measured first

The handover's "Exact next steps" item 1 asks for one folder whose chain to a
boundary is short.
The chains were measured,
not guessed:

- A call graph over the production sources (every `*.rs` under `src/native`
  minus `*_tests.rs` and the word-merge corpus,
  comments and string literals
  stripped,
  each `fn` body brace-matched) found 79 distinct
  (file,
  function) pairs that call a `run_git*` entry,
  in 49 files.
  A plain text count of the same entries gives 111 mentions in 68 files,
  which includes doc comments.
- Each such function's ancestor set was then read back with `rg` on its own
  name,
  one level at a time,
  so a chain is a list of functions one can check
  by hand rather than a number from the graph.

What that gave,
for the folders the brief does not reserve:

- `health/` (3 sites):
  `fsmonitor_report` is reached by
  `main` > `run_process` > `plan_invocation` > `plan_management` >
  `plan_management_with_section` > `plan_doctor` > `doctor_action_with_section`,
  and inside the folder the three Git readouts are called by nothing but each
  other and the folder's own controls.
  Six functions above the folder,
  three
  inside it,
  and the chain closes.
- `conflict/` (7 sites):
  `paths_with_markers` and `marker_sizes` hang below
  `report_landing` > `service_commit` > `run_timed` > `run_commit_command` >
  `run_commit_if_real` > `run_wrapped_command` > `plan_with_real_git` >
  `plan_invocation`,
  which is the commit path,
  and the commit path is the one
  the five-second bar and the budget controls measure.
- `commit/` (19),
  `landing/` (27),
  `merge/` (15):
  9 to 14 hops,
  all of them
  through the commit or the landing,
  and `landing/` is reserved by the brief.
- `standards_review/` (4):
  `read_patch` is reached through
  `service/service_review.rs` and `repository_link_check/`,
  16 hops.
- `shadow/` (10):
  `list_real_refs` and `write_shadow_refs` are reached through
  `create_shadow_repository_carrying` > `build_with_git` in `landing/`.
- `config/` (1):
  `read_config_snapshot` has eight production callers,
  one in
  `landing/` and one in `worktree_copying/`,
  both reserved by the brief.
- `repair/` (1):
  `manifest_directories` sits under `repair_cargo` >
  `repair_worktree` in `worktree_copying/`.
- `worktree/` (5):
  `untracked_file` and `trash_location_is_ignored` are items
  of `run_beside_caller` (`side_by_side_threads.rs`),
  so they move with the
  runner and not before it.
- `candidate/` (5):
  the landing's attempt rounds,
  which is item 2.
- `push/` (4),
  `invocation/` (5):
  `manual_push_range` and `git_version` are
  reached from the commit path and from `lock/`.

So `health/` is the folder item 1 asks for:
the shortest chain,
and the only
one whose subtree ends where it starts.
`landing/`,
`lock/`,
the `worktree_copy` files and the printed wait messages
were not touched.

#### The design

- `health/doctor.rs`:
  `query`,
  `locate_hooks` and
  `doctor_action_with_section` are futures and ask Git through
  `run_git_awaiting`,
  so no step of the report enters the runtime for the
  pipes of a step that is ended at its ceiling.
  `doctor_action` keeps its signature and is the synchronous face:
  it enters
  the runtime once for the whole report,
  which is the shape
  `captured_child_run_to_its_end::output_of_blocking` and
  `bounded_query::run_bounded_query` already use.
- `health/doctor_config.rs`:
  `config_findings` and `top_level` are futures.
- `health/doctor_fsmonitor.rs`:
  `fsmonitor_report` is a future.
- `config/config_snapshot.rs`:
  `read_config_snapshot_awaiting` is the second
  face of one read.
  The request,
  the parse and the debug lines are
  `config_listing_request` and `snapshot_of`,
  shared with the synchronous face,
  so the two cannot differ in what they read or what they record.
  The synchronous read keeps every caller it has,
  because two of them are in
  reserved folders;
  a conversion of the read itself is item 2's work.
- `invocation/management.rs`:
  `plan_doctor` enters the runtime once around the
  report and turns a refused entry into the command's own failure action
  (`doctor::doctor_failure`,
  whose prefix is the doctor's),
  so a report that could not be started reads like every other doctor failure.

Two rules from the previous agent are kept:

- No synchronous `Drop` that waits for a task runs on a worker.
  The report is
  one task,
  and its watches are closed by `run_git_awaiting`,
  which closes them
  with `ChildWatch::close_awaiting`.
- Nothing waits for a job.
  The report starts no blocking job,
  and the one
  runtime entry is made by the synchronous face,
  on the calling thread.

One limit of this step,
said rather than hidden:
the report's storage reads (the hooks directory scan,
the configuration file and the
`/proc` reads of the filesystem section) run on the worker that carries the
report rather than on the blocking pool.
Each is one short read of a known small file or directory,
they happen once per doctor run,
and putting them on the pool would need owned copies of what they read for a
`'static` job;
the landing's round is where that shape is decided,
so this step does not set a second one.

A second contract this step changes,
said plainly:
`doctor_action_with_section` takes its filesystem section as
`&dyn Fn(&Path, Option<&Path>) -> String`,
and that function is now called **on** the runtime instead of before it.
The production callback
(`invocation/management.rs`,
`filesystem_section_with_link_line` >
`doctor_filesystem::host_section` >
`repository_link_check::doctor_link_line`)
enters no runtime and starts no thread,
which was read in those files and is what the binary suite's
`subvolume::the_doctor_ends_with_the_filesystem_section_and_keeps_its_status`
exercises,
but a caller may no longer hand the doctor a section that enters the runtime.
The callback is also not `Sync`,
so the report's future is not `Send`;
it is driven by `block_on` on the calling thread and is not spawnable,
which is what the bridge does and all it does.

#### What could not be converted, and why

- `invocation/forwarded_command.rs::read_config_value` stays as it is.
  Converting it would build the process's runtime on the forwarded-command
  path,
  and the guard `binary_forwarded_command_makes_no_thread_tests.rs`
  plus the owner's choice 1 (lazy,
  at the first wrapper code that needs it;
  measured at +0.3 to +0.5 ms and about 125 minor faults on every forwarded
  command) exist to keep that path free of the runtime.
  A conversion here is a decision about that measurement,
  not a call-site edit.
- The three "pure-file" sites of item 3 (`worktree/worktree_trash_entries.rs`,
  `subvolume/subvolume_index_refresh_of_every_worktree.rs`,
  `worktree/worktree_convert_command.rs`) stay on the blocking runner for now,
  and the next agent should read this before starting them.
  The awaited runner takes `Arc<[T]>` items,
  `Arc<[Vec<usize>]>` lanes and
  `Arc<dyn OrderedAwaitingWork<T, R>>` work,
  and a task spawned on the runtime
  must be `'static`.
  Each of the three lends the runner something that is not owned:
  - `subvolume/` lends `rewrite: &RewriteOne<'a>`,
    a borrowed closure,
    and
    `IndexLock<'a>`,
    which holds a borrowed token from the caller;
  - `worktree/worktree_convert_command.rs` lends `platform: &dyn Platform` and
    `interrupt: &Interrupt`.
  So converting one means owning those first:
    `Arc<dyn Fn ... + Send + Sync>`,
  an owned `IndexLock`,
    and `Arc<dyn Platform + Send + Sync>`.
  That changes the signatures `subvolume_migrate.rs`,
    `subvolume_rollback.rs`
  and `subvolume_index_parallel_tests.rs` are written against,
    and the
  controls that pass closures borrowing a `Rendezvous` and an `AtomicUsize`.
- There is a second,
  larger obstacle on the same three sites:
  an item that says
  its lines while it works.
  `run_blocking_job_within_the_budget` takes `J: FnOnce() -> T + Send + 'static`,
  and the sink an item says its lines through (`LinesToTheConsumer`) is lent to
  the item's future for the length of the run,
  so it cannot travel into a
  `'static` job.
  A site whose items say lines as they go therefore needs either a buffered
  report or a new way for a blocking job to hand a line back to the consumer.
  The landing's round and the three pure-file sites all say lines while they
  work,
  so this is a decision the next agent has to make before item 3 moves.

#### Controls

`health/doctor_tests.rs` (three new):

- `the_report_is_built_on_the_runtime_it_is_awaited_on`:
  the whole report,
  awaited on the runtime,
  with a hook in the default directory,
  ends with
  status 1 and lists that hook by path.
  It is the conversion's proof:
  it can pass only when no step of the report
  calls the synchronous face.
  The in-process readers are switched off in that control
  (`repo_read_hook::switch_off`),
  because a reader that answered a question without a child would let the
  control hold whatever face the step used.
  Planted control:
  with `query` put back to the synchronous `run_git`,
  the control fails with
  "git rev-parse --path-format=absolute --git-common-dir --absolute-git-dir
  could not start:
  the runtime that runs this command's waits was asked for from inside itself"
  and is restored.
  Its red half,
  `a669e2a59a`,
  is a compile failure:
  the compiler answers
  "the trait `std::future::Future` is not implemented for
  `invocation::action::Action`" and suggests making `doctor_action_with_section`
  asynchronous.
- `the_two_faces_of_the_report_agree`:
  the synchronous face and the awaited
  report give the same status,
  standard output and standard error,
  field for
  field,
  on a repository with a configured hook,
  a traditional hook and the
  fsmonitor note.
  It compares the bridge with the report it wraps,
  which is what the bridge has to keep;
  what pins the report against **before** this step is the exact-output
  controls that were already here
  (the folder's 66 others and the binary suite's nine),
  every one of which compares a whole action,
  and none of which changed.
- `the_synchronous_face_of_the_report_is_refused_from_inside_the_runtime`:
  the positive control,
  so the first control cannot pass for another reason.
  The refusal reaches the caller as a doctor failure (status 2) whose message
  names the runtime asked for from inside itself,
  not as a panic.

The existing controls were moved onto the runtime and say so in one helper per
file:
`located` (`doctor_tests.rs`),
`findings_of` (`doctor_config_tests.rs`),
`fsmonitor_of` (`doctor_fsmonitor_tests.rs`) and `with_section`
(`doctor_filesystem_tests.rs`).

#### Checks

`mise run //package/git-policy/cli:native:test:host -- health::`:
69 tests run,
69 passed,
three of them new here.
`native:test:binary:host -- doctor`:
9 tests run,
9 passed,
including
`doctor_outside_a_repository_still_exits_two`.
`native:clippy`:
clean.
`monochromatic-lint --max-warnings 0` on the nine changed files:
0 findings
after `//:format:monochromatic-lint` inserted the three line breaks it asks for.
The whole suites,
at the tip,
with `CLI_GIT_TEST_ROOT=/var/tmp/cg/cli-git/fixtures/root` and the run's own
control group at `TasksMax=4000`:
`native:test:host` 4807 run,
4807 passed (1 slow),
8 skipped;
`native:test:binary:host` 733 run,
733 passed (10 slow),
3 skipped.
A lib run before the last two commits showed one failure,
`worktree_copying::worktree_copy_stage::tests::a_stamp_comes_only_from_a_clone_of_a_settled_file`,
which asserts that a file written this instant is still inside the timestamp
granularity;
the same binary passed it three times out of three when asked alone,
and the tip run of the whole suite passed it too.
It is in a folder this branch does not touch,
and the suspected cause is the twenty milliseconds the control allows between
the write and the clone,
against the load of the whole suite,
which is a suspicion and not a measured cause.
The fixture root has two length limits,
not one,
which the earlier sections did not record:
it must be short enough for a Unix socket under it to fit `SUN_LEN`,
**and** at least six path segments deep,
because `worktree_add_binding`'s control
`a_relative_pointer_without_links_is_bound` resolves `../../../../../../..`
from a fixture directory and fails when that lands above `/`.
Measured on this tree:
`/var/tmp/cg-fixtures` (three segments) fails that control and
`/var/tmp/cg/cli-git/fixtures/root` (six) passes it,
the same test binary both times.

#### Latency, and why this host cannot answer the commit bar today

Three `native:perf` runs,
on a host where other sessions and these two suites had been running,
the third arm a binary without this step's change
(main's tip `bbadfac560`,
the nine files put back,
built and measured minutes after the second arm,
then restored):

- Arm 1,
  this tree (2026-11-08T20:55:14-05:00):
  forwarded commands added median 1.7,
  1.9,
  1.6 and 1.9 ms;
  commit through the service added median 83.2 ms and paired delta maximum
  1220.1 ms;
  signed commit 45.8 ms,
  maximum 137.5 ms;
  no bound broken.
- Arm 2,
  the same binary
  (2026-11-08T20:59:22-05:00):
  forwarded 1.6,
  1.9,
  1.6 and 1.8 ms;
  commit 213.8 ms,
  maximum 5437.5 ms;
  signed 178.1 ms,
  maximum 2188.6 ms;
  the commit bound broken.
- Arm 3,
  without this step's change
  (2026-11-08T21:01:22-05:00):
  forwarded 1.8,
  2.1,
  1.7 and 1.5 ms;
  commit 382.7 ms,
  maximum 10801.4 ms;
  signed 59.9 ms,
  maximum 687.1 ms;
  the commit bound broken,
  by twice as much as arm 2.

What that says,
and what it does not:

- Forwarded commands are inside the record either way (1.4 to 2.2 ms in the T11
  section;
  1.5 to 2.1 ms here),
  and no arm broke that bound.
- The commit bar is **broken in this window**,
  by arm 2 (maximum 5437.5 ms against the 5000 ms bound)
  and by arm 3 (10801.4 ms),
  and it was met by arm 1 (1220.1 ms).
  The unchanged tree broke it twice as far,
  which says the branch does not carry the violation,
  and it does not make the violation go away:
  the bar is the owner's requirement on the commit path,
  and on this host,
  at this time,
  the tree does not meet it.
- Two runs of the **same** binary differ by 2.6 times (83.2 against 213.8 ms)
  and the run of the **unchanged** tree is 382.7 ms,
  so the run-to-run spread is larger than any plausible effect of this step.
  That is the whole of what these three arms can attribute.
  The one file this step changes that the commit path does run is
  `config/config_snapshot.rs`:
  its synchronous read keeps its command line,
  its parse and its order,
  with the body split into `config_listing_request` and `snapshot_of`,
  and the commit path's other readers are untouched.
  The doctor has no caller outside `health/` and `plan_doctor`,
  so a doctor run cannot be in a commit at all.
- The T11 record of 19.6 to 26.0 ms was taken under much less load.
  The next step for these numbers is the one the earlier sections used:
  alternate the two binaries arm by arm,
  a same-binary pair for the spread,
  and repeat until the arms separate,
  under whatever load the host carries then.
  It is not a matter of waiting for a quiet host.

### Where the remaining steps stand after T5

- T6 to T12 convert code that waits one thing after another (the retry sleeps of `start_retry.rs`,
  the `try_wait` loops of
  `bounded_push.rs` and `package_repair_command.rs`,
  the service's per-connection threads,
  the guard wait of `commit_preparation.rs`).
  The pattern that worked for T4 and T5 (a synchronous face that enters the runtime,
  tasks only where something must run beside a blocked
  thread) does not shorten these:
  a sleep or a poll in sequential code becomes a `tokio::time` call only when its caller is itself
  asynchronous,
  so they are in practice the T11 conversion of the callers,
  done leaf first.
  They were not started.
- Two steps need no async callers and could go first:
  T9 (the standards review on `hyper`,
  which needs new locked crates through the
  toolchain tasks) and the `zbus` 6 client of T8 (the catalog already names `zbus >=6.2.1` and `hyper >=1.12.0`;
  the "floor 5.19" of the first plan text and of the earlier handover is gone,
  and no release-age exclusion names `zbus@5.19.0`).

### What is left

Steps T5 (the watcher task landed;
its one-shot pidfd form and the `LeaveRunning` thread did not) to T13 (the watchers,
`bounded_push`,
the signal relay on a pidfd with the registry of children,
the service client and
the `zbus` move,
the standards-review call on `hyper`,
the service,
the callers,
the parallel runner,
the final removal and lint),
the replacement of `private_storage.rs`,
the descriptor-limit sweeps of the child helper,
and the mutation campaign.
- After T12's first part (2026-11-08T18:40:00-05:00,
"Step T12,
the runner's jobs on the runtime"):
  the bound and the runtime runner exist and are tested;
  what is left of T12 is the per-call-site conversion of seven `run_ordered` sites and nine
  `run_all_side_by_side`/`run_beside_caller` sites to shared state,
  and that conversion of the service round needs the leaf-first `async` Git first.
  T13's lint (no `std::thread`,
  no `std::process::Command` outside the guarded start,
  no `thread::sleep`) cannot be tightened until those call sites are converted,
  since `scoped_start.rs`,
  `side_by_side_threads.rs` and 30-odd other files still start threads.
- After the leaf-first asynchronous Git and the awaited runner
  (2026-11-08T19:29:56-05:00,
  "Step T12,
  the leaf-first asynchronous Git and the ordered runner that awaits"):
  a Git step can be awaited by a task of the runtime
  (`transaction_git_awaiting.rs`) and a round can run items that await one
  (`parallel_runner_on_the_runtime_awaiting.rs`),
  and no production call site uses either.
  What is left of T12 is the conversion itself,
  one folder at a time from the top of each chain,
  and `landing/service_round.rs` last,
  because its items run the landing's handlers.
- After the first production folder
  (2026-11-08T21:05:00-05:00,
  "Step T12,
  the first production folder on the awaited Git runner"):
  `health/` awaits its Git reads on the runtime and the report of
  `git monochromatic-git-policy-cli doctor` is built by one task,
  with `plan_doctor` and `doctor_action` as the synchronous faces that enter
  the runtime once.
  A shared read whose other callers are in reserved folders gets a second face
  beside its synchronous one
  (`config_snapshot::read_config_snapshot_awaiting`).
  What is left is unchanged in order,
  and item 3 now has two measured obstacles recorded in that section:
  the borrowed state each pure-file site lends the blocking runner,
  and the live lines an item says,
  which a `'static` blocking job cannot carry.
- After the three pure-file sites
  (2026-11-08T21:47:00-05:00,
  "Step T12,
  the line path and the state of the pure-file sites"):
  an item of an awaited run owns its line sink (`LinesToTheConsumer<R>`,
  which it may move into a `'static` job),
  and the three sites that lend the blocking runner borrowed state are on
  `run_ordered_awaiting_on_the_runtime`
  (`worktree/worktree_trash_entries.rs`,
  `subvolume/subvolume_index_refresh_of_every_worktree.rs`,
  `worktree/worktree_convert_command.rs`),
  each with the synchronous face its callers still use.
  What is left of item 3 is `worktree/worktree_copying/` and the copy and repair sites of
  `foundation/side_by_side_threads.rs`,
  then item 2 (the landing's handlers and `landing/service_round.rs`),
  then T13's guards.
  One thing a later conversion must copy:
  a job of the blocking pool is inside the runtime,
  so a bridge that starts a child (`output_of_blocking`) refuses there;
  `LinuxPlatform::with_its_conversion_environment_settled` is the shape that answers such a
  question before the run instead.
- After the snapshot source and the clean check
  (2026-11-09T14:35:00-05:00,
  "Step T12,
  the snapshot source and the clean check move to the runtime"):
  `worktree/worktree_clean.rs` and
  `worktree/worktree_snapshot_source.rs` start no thread and no scoped job:
  the untracked scan of the clean check and the configuration read and trash rule of the source
  check are awaits of the same task,
  and the proof of the tracked files is a job of the blocking pool beside them.
  The production files of `worktree/` therefore hold no `std::thread` at all;
  what still starts one is
  `worktree_copying/` (`worktree_copy_compare.rs`,
  `worktree_copy_after_lock.rs`,
  `worktree_copy_install.rs`,
  `worktree_copy_stage_leaf_copying.rs`,
  `worktree_copy_journal_removal.rs`),
  `commit/commit_index_stat_staleness.rs`,
  `landing/service_durable.rs`
  and the scoped-thread runner of `foundation/parallel_runner.rs` itself.
  What is left of item 3 is unchanged after those,
  then item 2 (the landing's handlers and `landing/service_round.rs`),
  then T13's guards.
  A conversion of a site whose Git half is a job rather than an await must copy the one rule
  this step measured:
  the scan and both source reads had to become awaiting faces
  (`run_git_awaiting`),
  because a job of the pool is inside the runtime and the synchronous face refuses there.
- After the worktree copy's five thread sites
  (2026-11-09T14:50:45-05:00,
  "Step T12,
  the worktree copy's threads become waits of the runtime"):
  the final check's two walks,
  the copy stage's slices,
  the installation's slices and the split removal's workers are jobs of the blocking pool,
  and a snapshot's trash and the package repair beside it are two branches of one task of
  the runtime.
  `worktree/worktree_copying/` holds no production scoped thread;
  the one left is a control that runs two adds at once.
  What is left of item 3 is the two `run_beside_caller` sites of
  `worktree/worktree_snapshot_source.rs` and the one of `worktree/worktree_clean.rs`,
  then item 2 (the landing's handlers and `landing/service_round.rs`),
  then T13's guards.
- The snapshot-source step and the worktree-copy step are both on main at 2026-11-09T16:01:04-05:00,
  so each entry's list of what is left was written before the other landed:
  the `run_beside_caller` sites of `worktree/worktree_snapshot_source.rs`,
  the site of `worktree/worktree_clean.rs` and the `worktree_copying/` sites are done.
- After the merge's second pass
  (2026-11-09T17:35:00-05:00,
  "Step T12,
  the second pass awaits its own Git,
  and the reconcile's chain stops at the branch move"):
  `merge/` awaits its own Git,
  so the reconcile's merge commit is `commit_merge_awaiting`,
  and a person's merge reaches the same body through one entry of the runtime.
  What is left of item 2 is the recovery folder's awaited Git seam
  (19 synchronous start sites across six files),
  then `advance_local_branch_awaiting`,
  `integrate_and_push_awaiting` with `MergePusherAwaiting`,
  `reconcile_divergent_push_awaiting`,
  `push_once_awaiting`,
  and the two handlers;
  then `landing/service_round.rs`,
  then T13's guards.

### Step T12, the line path and the state of the pure-file sites, 2026-11-08T21:47:00-05:00

- Branch and worktree:
  `feat/git-policy-cli-tokio-pure-file-sites-on-the-awaited-runner` in
  `.claude/worktrees/pi-tokio-pure-file-sites-on-the-awaited-runner`,
  from main at `3db3002750`.
  Not merged into main.
- This section decides the two things the "What is left" section left open for item 3
  before any of its code moves:
  the line path of an item whose work runs in a `'static` job,
  and the owning of the state each site lends the runner.
  Both decisions are
  this agent's under the owner's delegation,
  and both are open to the owner's veto.

#### What each site really lends, and which of them says lines while it works

The T12 step record says the three pure-file sites "all say lines while they work".
Read again,
only one of them does:

- `worktree/worktree_trash_entries.rs`:
  `RemoveEachEntry::run` takes `_say` and says nothing;
  the reasons are said at `Event::End` by `KeepReasonsOfEntriesThatStay`,
  on the consumer.
- `subvolume/subvolume_index_refresh_of_every_worktree.rs`:
  `RewriteEachIndex::run` takes `_say` and says nothing;
  one line per index is said at `Event::End` by `SayEachIndex`,
  on the consumer.
- `worktree/worktree_convert_command.rs`:
  `ConvertEachWorktree::run` hands `say` to `convert_worktree`,
  which says its progress as it goes,
  and `ConversionReport` relays each `Event::Line`.
  This is the only site of the three that needs a line path at all.

So the line path is one decision,
but only its third site exercises it;
the first two say their lines where every runner already says them.

#### Owning the borrowed state

- A.
  Share it:
  `Arc` the state the job reads.
  Pros:
  it is what both runtime runners already take (`Arc<[T]>` items,
  `Arc<dyn ... + Send + Sync>` work),
  one copy for the whole run,
  no lock,
  and no file descriptor per item.
  Cons:
  the sites' own signatures change
  (`EntryRemoval` owns its fields,
  `IndexLock` owns its token,
  `ConvertRun` takes `Arc<dyn Platform + Send + Sync>` and `Arc<Interrupt>`),
  and the controls that lend a closure over a
  `Rendezvous` or an `AtomicUsize` must own it in an `Arc` first.
- B.
  Clone it per item.
  Pros:
  the sites keep their borrowed signatures.
  Cons:
  a clone of the manifest or the target list per item,
  a duplicated descriptor per entry (`File::try_clone`),
  and memory that grows with the item count for state that never changes.
- C.
  Leave these three on the blocking runner of scoped threads.
  Pros:
  nothing changes and the controls stay as they are.
  Cons:
  the scoped threads stay,
  so T13's guard against `std::thread` cannot be tightened,
  and the folder carries two runners for longer.
- D.
  Leak the state (`Box::leak`) so it is `'static`.
  Pros:
  no signature change and no clone.
  Cons:
  memory that no caller can size or free lives for the life of the process,
  and the ownership the type system would have shown is hidden in a comment.

Ranking:
A > B > C > D.
A beats B because it keeps one copy of state that never changes and adds no descriptor.
B beats C because it reaches the runtime runner the plan already requires for item 3.
C beats D because leaking state to avoid a signature change is the one option that hides a cost
from the person who will later read the code.

#### The line path

- A.
  Hand each item an owned line handle.
  The runner gives an item a sink it owns
  (a clone of the consumer's sending end and the item's index)
  instead of a sink borrowed for the length of the run,
  so the item may move it into a `'static` blocking job.
  The lines travel the one channel the consumer already reads.
  Pros:
  lines appear as they happen and in input order,
  no lock is added,
  no second channel and no per-item loop,
  and the shape is the one the blocking runner of
  `parallel_runner_on_the_runtime.rs` already gives its own jobs
  (`LinesToTheConsumer` is built inside the job from an owned sender).
  Cons:
  `OrderedAwaitingWork::run` changes its third parameter,
  so every implementation of it changes,
  including the ones the landing's handlers will add.
- B.
  A forwarding channel inside the item.
  The item's future owns an unbounded channel,
  the job is given its sending end,
  and the future selects between the job's end and each arriving line,
  saying each through the borrowed sink.
  Pros:
  the same liveness and order as A,
  no lock,
  and no change to the trait.
  Cons:
  a second channel and a select loop in every site that says lines,
  and each line makes one hop through the worker that awaits the job.
- C.
  Buffer the lines and return them with the result.
  Pros:
  the least code;
  the job owns a `Vec<String>` and nothing else changes.
  Cons:
  an item's lines appear only when it ends,
  so the first unfinished worktree,
  whose lines are the only live ones today,
  loses its progress;
  this is the behaviour `parallel_runner.rs` exists to keep.
- D.
  Run the item's work on the worker instead of the blocking pool.
  Pros:
  the borrowed sink keeps working with no change at all.
  Cons:
  a worker that polls tasks is blocked for the length of a conversion,
  which is the case the plan's own rule forbids
  ("Blocking files,
  fsync and the runtime's own shutdown").

Ranking:
A > B > C > D.
A beats B because it adds neither a hop nor a loop and reuses the one channel of the run.
B beats C because it keeps the progress of the first worktree live.
C beats D because a buffered report still keeps every item on the blocking pool.

#### The choices, as this agent's, open to the owner's veto

- The state each site lends is shared with `Arc` (option A),
  and its synchronous face keeps its signature by owning what it needs
  (`EntryRemoval` with owned fields,
  `IndexLock` with an owned token,
  `ConvertRun` and `execute` taking an `Arc`).
- The line path is the owned line handle of option A:
  `parallel_runner_on_the_runtime_awaiting.rs` hands each item an
  owned `LinesToTheConsumer<R>`,
  which the item moves into its `'static` job,
  and the consumer keeps the input order it already keeps.
  No lock,
  no buffering and no runtime on the forwarded-command path
  (nothing of the three sites is reached by a forwarded command).

#### What changed, per site

- `foundation/parallel_runner_on_the_runtime.rs`:
  `LinesToTheConsumer<R>` owns a clone of the run's one sending end instead of borrowing it,
  and both runtime runners hand it to their items
  (the blocking form builds it inside its own job,
  as it already did).
  `OrderedAwaitingWork::run` therefore takes the sink by value.
- `worktree/worktree_trash_entries.rs`:
  each entry is an item of the awaited runner and each removal one job of the pool.
  `EntryRemoval` owns a `PathBuf`,
  an `Arc<File>`,
  an `Arc<TrashManifest>` and an `Arc<RemoveOne>`;
  `RemoveOne` gains `Send` and loses its lifetime;
  `delete_trash` holds its trash handle in an `Arc`.
  `remove_entries` keeps its signature and enters the runtime once;
  `remove_entries_awaiting` is the face an asynchronous caller uses.
- `subvolume/subvolume_index_refresh_of_every_worktree.rs`:
  each index is an item of the awaited runner and each rewrite one job of the pool.
  `AllIndexes` owns its root,
  its references and an `Arc<RewriteOne>`;
  `IndexLock::Held` shares its token in an `Arc<str>`;
  `RewriteOne` gains `Send` and loses its lifetime.
  `refresh_all_indexes` and `refresh_all_indexes_with` return `Result<usize, MigrateError>`
  and enter the runtime once,
  `MigrateError` gains `Runtime`,
  and `refresh_all_indexes_awaiting` is the asynchronous face.
- `worktree/worktree_convert_command.rs`:
  each worktree is an item of the awaited runner and each conversion one job of the pool,
  which is where the owned line sink is needed:
  `convert_worktree` says its progress through it while the job works.
  `ConvertRun` takes an `Arc<dyn Platform + Send + Sync>` and an `Arc<Interrupt>`;
  `execute` takes both by value and `convert_action` builds them.
  `execute_run` keeps its signature and enters the runtime once,
  `execute_run_awaiting` is the asynchronous face,
  and a run the runtime ends early says so before the summary and exits 2.
- `testing/parallel_test_support.rs`:
  `hold_the_only_place` holds the process's one place on blocking jobs on a task of the
  runtime,
  so a control can show that its work waits for a place,
  which only a job of the pool does.

What stayed as it is,
and why:

- The waits:
  none of the three sites waits without a bound it already had.
  The runner's consumer ends when every lane has returned (`finish_the_run`),
  and a lane that a refused `spawn_blocking` swallowed is counted out by `CountedOut`.
- The two measured rules of the previous step:
  nothing of this step has a synchronous `Drop` that waits for a task,
  and no job of the pool awaits anything,
  so nothing waits on a job from inside a job.
- `worktree/worktree_copying/` and the copy and repair sites of
  `foundation/side_by_side_threads.rs` are not part of this step.
  The handover's item 3 lists them with these three sites;
  the brief this work follows reserved that folder,
  so they stay on the blocking runner for the next agent.

#### Controls

Every site has a control that fails on the code before it,
and the third site has two,
planted separately,
because they catch different defects:

- The runtime control:
  `the_removals_run_on_the_processs_runtime`,
  `the_rewrites_run_on_the_processs_runtime`,
  `the_conversions_run_on_the_processs_runtime`.
  Each records `tokio::runtime::Handle::try_current().is_ok()` at the site's own work:
  `[false, false, false, false]` before the conversion,
  where a scoped thread is in no runtime,
  and `[true, true, true, true]` after it.
  Measured rather than assumed:
  a probe in `blocking_jobs_within_the_budget_tests.rs` printed
  `PROBE inside a job: current=true thread=Some("git-policy-worker")`
  and `PROBE outside: current=false`,
  so a job of the pool is in the runtime and a scoped thread is not.
- The place control:
  `the_removals_take_places_of_the_processs_bound_on_blocking_jobs`,
  `the_rewrites_take_places_of_the_processs_bound_on_blocking_jobs`,
  `the_conversions_take_places_of_the_processs_bound_on_blocking_jobs`.
  With the bound held to one place by another job of the pool,
  no item's work starts until that job gives the place back.
  The holder gives it back because a job of the work under test is waiting for a place
  (`blocking_jobs_within_the_budget::jobs_waiting_for_a_place`,
  test-only),
  and it reports that ask,
  which each control asserts,
  so a work path that never takes a place fails the control.
  A review of the first version of these controls found the holder giving the place back after a fixed
  second instead,
  and it is right:
  with that helper and a conversion run on the worker after a 1,500 ms wait,
  the conversion control passed although no place was ever taken.
  With the ask it fails,
  and the plant is recorded in the commit that changed it.
  Planted for the third site by running the conversion on the worker instead of as a job,
  the place control fails with
  "a conversion started while another job held the only place of the process's bound,
  so it did
  not take one",
  while the runtime control still passes,
  which is the reason both exist.
  The plant was reverted and the folder passes again.
- The line path:
  `a_line_said_inside_a_blocking_job_reaches_the_consumer_as_it_happens` in
  `parallel_runner_on_the_runtime_awaiting_tests.rs`.
  It does not compile before the owned sink
  (`error[E0521]: borrowed data escapes outside of method`,
  "argument requires that `'a` must outlive `'static`"),
  and with the sink holding its lines until it is dropped it fails after its patience with
  "the consumer saw the line only after the job ended".
- The output:
  one control per site pins the whole report the code gave before the step,
  and each was written and passed on the tree before its conversion,
  then again after it.
  `the_lines_of_a_run_are_the_ones_recorded_before_the_step` exists twice
  (`subvolume/subvolume_index_parallel_tests.rs` and
  `worktree/worktree_convert_parallel_tests.rs`):
  the checkout's index first and then each linked worktree's by path,
  and for a conversion the risk notice,
  each worktree's name,
  its copy line,
  its index line,
  its verdict,
  and the summary.
  The counts they pin are named constants of the fixture
  (`CHECKOUT_ENTRIES_REFRESHED`,
  `LINKED_ENTRIES_REFRESHED`,
  `COPIED_ENTRIES`,
  `INDEX_ENTRIES_REFRESHED`),
  so a legitimate change to the fixture breaks them on purpose.
  The trash site's own output is the reasons of the entries that stay,
  in entry order,
  and its three existing controls compare that list exactly;
  they pass unchanged.

#### Latency

Nine `native:perf` arms over two windows on 2026-11-08 (22:53 to 22:57 and 23:45 to 23:47),
with two binaries:
this tree,
and main's tree at the base commit `3db3002750`
(the files of this step restored with `git restore --source=3db3002750`,
built,
measured,
and restored,
whose rebuild produced a binary byte for byte the one the arms of this tree measured).
The host carried a load average between 60 and 91 on 12 CPUs throughout,
with another agent's whole host suite running in the second window,
so the arms alternate and each binary has arms on both sides of the other's.

The arms in the order they ran,
commit through the service and signed commit through the service,
each the paired delta against native Git:

- 22:53,
  this tree:
  commit median 146.8 ms,
  maximum 4,486.9 ms;
  signed median 167.8 ms,
  maximum 5,108.5 ms.
- 22:54,
  this tree:
  commit median 111.1 ms,
  maximum 2,277.4 ms;
  signed median 99.5 ms,
  maximum 457.2 ms.
- 22:55,
  main's tree:
  commit median 45.5 ms,
  maximum 284.4 ms;
  signed median 108.5 ms,
  maximum 7,658.5 ms.
- 22:56,
  this tree:
  commit median 119.2 ms,
  maximum 999.0 ms;
  signed median 95.2 ms,
  maximum 2,386.7 ms.
- 22:57,
  main's tree:
  commit median 56.7 ms,
  maximum 2,842.3 ms;
  signed median 93.6 ms,
  maximum 296.3 ms.
- 23:45,
  main's tree:
  commit median 171.4 ms,
  maximum 8,028.0 ms;
  signed median 123.3 ms,
  maximum 380.0 ms.
- 23:46,
  this tree:
  commit median 39.5 ms,
  maximum 1,228.1 ms;
  signed median 161.5 ms,
  maximum 9,664.7 ms.
- 23:46,
  main's tree:
  commit median 58.5 ms,
  maximum 3,533.3 ms;
  signed median 111.9 ms,
  maximum 1,039.8 ms.
- 23:47,
  this tree:
  commit median 60.5 ms,
  maximum 2,769.2 ms;
  signed median 89.4 ms,
  maximum 221.0 ms.

Forwarded commands,
the added medians of `rev-parse`,
`status`,
`log` and `diff-stat`,
stayed between 1.4 and 2.2 ms in every arm,
three orders of magnitude under the 2,000 ms bound.
One arm of main's tree (23:45) reported `status` at a p95 of 3,843.1 ms with a median of 2.2 ms,
a stall in one sample;
no arm of this tree did.

What that says,
and what it does not:

- The first window looked like a separation on the commit median
  (this tree 111.1 to 146.8 ms,
  main's 45.5 to 56.7 ms),
  and that reading does not survive the second:
  the first window's arms were this tree,
  this tree,
  main's,
  this tree,
  main's,
  so a slow half hour for one binary can look like an effect of the other.
  The second window interleaves the two (main's,
  this tree,
  main's,
  this tree),
  and there this tree's commit medians are 39.5 and 60.5 ms against main's 171.4 and 58.5 ms,
  the same order reversed.
- The same-binary spread covers the whole difference:
  this tree's five commit medians run from 39.5 to 146.8 ms (3.7 times),
  main's four from 45.5 to 171.4 ms (3.8 times),
  and each binary's signed medians run 89.4 to 167.8 and 93.6 to 123.3 ms.
- The 5,000 ms bar was broken in this window by this tree (signed 5,108.5 and 9,664.7 ms) and by
  main's tree (signed 7,658.5 ms and commit 8,028.0 ms),
  and met by the other arms of both.
  The bar is broken by both binaries under this load,
  and no arm attributes that to this step.
- Nothing here is evidence that this step made the commit path slower or faster,
  and there is a reason to expect it cannot:
  none of the files this step touches runs in a commit.
  The trash entry removal is reached from the copy path of a new worktree
  (`worktree_copying/worktree_copy_after_lock_disposal.rs`)
  and from the service's work loop for a trash item,
  the index refresh of every worktree is reached from the subvolume migration,
  and the conversion command and the runner's trait are reached from neither.
  A commit in these fixtures has no trash item and no migration.
- What would settle the commit bar is the earlier sections' advice with more arms than this:
  interleave the two binaries one arm each,
  many times,
  under whatever load the host carries then.

#### What a review of the branch found, and what changed

Two findings,
both about what the code promises rather than about the move,
both read from the committed code and both right:

- A rewrite that panicked after it installed its index was counted with the rewrites that never
  started,
  and the error said "the indexes stay as they were".
  The kinds of end are counted apart now (`RewritesThatGaveNoResult`),
  and the message keeps the uncertainty:
  "1 of 4 index file(s) ended without a known result,
  so whether those were rewritten is not known;
  run the command again to rewrite them",
  with the line of that index saying that it "may have been written or may be as it was".
  The control installs the checkout's index and then panics,
  and fails on the old message.
- The place controls gave their place back after a fixed second instead of on an attempt,
  so a work path that bypasses the bound could start after that second and pass.
  The holder now waits for the ask and reports it (see the controls above),
  and the review's own scenario was reproduced as a false pass before the change.

A third finding came from a second reviewer,
on the branch after those two,
and it holds as well:

- `NoThreadToStart` was read as proof that a job never ran,
  and `subvolume/subvolume_index_refresh_of_every_worktree.rs` promised "those stay as they were" on it.
  Tokio queues the job first:
  in the locked version (`Cargo.lock`,
  `tokio-1.54.1`),
  `runtime/blocking/pool.rs` pushes the task at line 624 and asks for a thread only at line 628,
  returns `SpawnError::NoThreads` at line 499,
  and panics with "OS can't spawn worker thread" at line 386,
  so the job stays queued and the pool's next thread runs it (`run_worker`,
  line 657).
  This plan's own note in `blocking_jobs_within_the_budget.rs` already said so for the place,
  and `blocking_job.rs`'s comment that "the closure it consumed is gone,
  so nothing ran" was wrong.
  Measured,
  in a child whose process limit is one:
  the report said "4 of 4 index file(s) were not rewritten,
  because the work of each one could not be
  started,
  so those stay as they were",
  a thread was allowed again,
  and **all four rewrites ran** afterwards.
  The fix is `foundation/the_claim_a_job_takes_before_running_it.rs`:
  one atomic exchange decides,
  between the job taking the claim before its first effect and the
  reporter taking it after the refusal,
  so a job the report gave up on does nothing and the promise is a fact,
  and a job that took the claim first makes the report say the result is not known.
  The control is
  `a_report_that_says_an_index_could_not_be_started_leaves_no_job_that_can_rewrite_it`,
  which counts the runs of its own rewrite after the drain,
  and it fails with the plant that removes the job's side of the claim.

Every other place that read the refusal as proof,
and what was decided for each:

- `foundation/blocking_job.rs`,
  the error itself and its text:
  reworded,
  because the type cannot know:
  "it had not started when that was reported;
  the pool had already queued it,
  so it may still run
  later".
- `foundation/parallel_runner_on_the_runtime.rs`,
  `OrderedRunError`:
  reworded the same way,
  and it now says that the item the step held may still run later.
- `worktree/worktree_trash_entries.rs`:
  no claim.
  Its reason list says an entry was not removed now,
  and a removal that runs later is identity-checked and is what the person asked for,
  so no data can be lost by it.
- `worktree/worktree_convert_command.rs`:
  no claim.
  A conversion that runs later is journaled and resumable,
  the report says the reason rather than any
  state of the worktree,
  and the claim would buy nothing the journal does not already give.
- The two controls that measured a job had not run (`blocking_jobs_within_the_budget_tests`,
  `parallel_runner_on_the_runtime_tests`) now say that they measure it at that moment.

#### A third review: three waits and bounds, and two controls that failed in a suite

All three findings held;
each has a control that failed first,
and each
control fails again with the fix removed.

- The completion loop could miss its final wake
  (`foundation/parallel_runner_on_the_runtime.rs`).
  The notification branch was guarded by `still_running.running > 0`,
  so a lane that returned between the count the consumer read and the wait it built left the consumer with a
  switched-off branch,
  a stored wake it could not use (`Notify::notify_one` stores one permit;
  `tokio-1.54.1/src/sync/notify.rs`),
  and a channel kept open by a sending end a refused job holds for the life of the process:
  a wait for ever.
  The wait is now one function,
  `wait_for_a_message_or_a_return`,
  whose wake branch is never switched off,
  the count is read at the top of each turn,
  and a closed channel leaves the count as the only thing to wait for (no spinning).
  The control,
  `a_wake_that_arrived_before_the_wait_still_ends_it_although_the_channel_stays_open`,
  builds the state the race leaves,
  since the two-instruction window itself cannot be forced from a test:
  it failed with
  "the wait was not ended by a wake that had already arrived,
  because the branch that reads wakes was switched
  off while the channel stayed open".
- The refresh could block the runtime before its first await.
  `subvolume_index_files::repository_hash_length` opens the repository's configuration with a synchronous
  `std::fs::read` (line 485),
  and a named pipe there blocks that open until a writer comes,
  on the thread that polls the runtime:
  no timeout could expire.
  What a run reads before its first rewrite is now `prepare_the_run`,
  one job of the blocking pool,
  with what it reads owned.
  The control,
  `a_preparation_read_that_hangs_leaves_the_runtime_free_and_its_timeout_expires`,
  failed with "the refresh answered within the control's wait:
  Timeout".
- Refused jobs could run outside the blocking-job bound.
  The frame gave the place back as soon as it stopped waiting,
  so a job the pool had queued at that moment ran later on no place at all — measured in a child whose process
  limit is one,
  with the bound held to two:
  the job the report gave up on ran.
  The place and the claim now travel together
  (`foundation/the_place_of_one_blocking_job.rs`):
  the job takes the claim before its first effect and holds the place until its work ends (or unwinds),
  the frame gives the place back only when it takes the claim,
  which proves the job will do nothing,
  and the slot is taken once so the second give-back is a no-op.
  The claim a site puts on its own report stays as it is:
  it is what lets a site say "given up on before it could start" rather than "not known".
  The control,
  `a_job_the_pool_queued_after_a_refusal_does_not_run`,
  failed with "the job the report gave up on ran afterwards with no place of the bound held for it".
  The version of this control that measured how many jobs run at once passed on the first attempt,
  because the process's own pool may allow fewer blocking threads than the bound,
  and the queue then runs one job at a time (measured:
  `peak=1`);
  its own runtime with a deeper pool is what makes the state reachable.

Two controls that failed in a filtered suite of 122 tests and passed alone were measured in one scope of their
two modules at CPU quotas of 100% and 200%:

- `the_indexes_are_rewritten_side_by_side` (10.09 s),
  `worktrees_are_converted_side_by_side` (11.38 s) and
  `the_report_keeps_input_order_whichever_worktree_finishes_first` (22.17 s) failed at 200%,
  every one of them a rendezvous that needs its items in flight at once;
- the review's `the_report_keeps_the_order_of_the_checkout_then_its_worktrees` needs two places (the checkout's
  rewrite holds one and waits for the others),
  so it fails at 100%,
  where the process allows one.

The cause is the process's bound on blocking jobs,
which comes from the CPUs the process may use,
not from the host a control states for its own budget request.
Measured with `budgeted_blocking_jobs()` against `worker_budget` on the same fixture:
the request asks 4 and the bound is 1,
2 and 3 at quotas of 100%,
200% and 400%.
`parallel_test_support::may_run_all_at_once` now states each control's need and passes without running where the
machine cannot allow it,
naming the control,
the places it needs and the bound it saw:
68 of 68 pass at both quotas.
The `SIGABRT at 125 s` the review saw for
`the_conversions_take_places_of_the_processs_bound_on_blocking_jobs` was not reproduced:
it passed in 1.1 to 9.0 s at both quotas,
and 125 s is nextest's watchdog,
which means the test never returned.
The only unbounded wait on that path was the completion loop's,
which the first finding of this round removed;
a refused job on the path cannot be ruled out under the load the review's run carried.

#### A fourth review: a revoked claim, a pool-side target search, two controls' cleanups, and a truthful refusal

- Cancelling a queued job did not revoke its claim.
  The claim was taken only on the returned error,
  so a future dropped at its `.await` left it free and the
  closure the pool still held queued ran afterwards:
  a conversion could change a worktree after its caller
  reported a timeout,
  with no reader left for its progress or its result.
  `TheCallOnAJob` holds the claim and the place of one job and acts when the call is dropped before it
  settles.
  The control,
  `a_job_whose_caller_stopped_waiting_does_not_run_afterwards`,
  kept the pool's only thread inside another job so the measured closure was queued at the moment the timeout
  dropped it,
  and failed with "the job whose caller stopped waiting ran afterwards,
  so a step can act after its
  caller reported a timeout".
- Worktree path resolution blocked the runtime before its first await:
  `execute_run_awaiting` ran `targets` (Git for `--all`,
  a scan of the linked worktrees) and `lanes_of`
  (a synchronous `std::fs::canonicalize` of each target) on the polling thread.
  `find_the_targets_and_lanes` is one job of the blocking pool now,
  like the index refresh's
  `prepare_the_run`.
  The control keeps the process's only place inside another job,
  registers a linked worktree while the run
  waits,
  and shows the run converts it;
  planted (the finding done on the awaiting task) it fails with "the
  worktree registered while the run waited was not converted,
  so the run had decided what it covers before it
  first waited".
- The preparation control's cleanup could wait for ever on its named pipe,
  and its first rewrite also joined
  the blocked thread before its assertion (which the plant caught as a 310 s timeout).
  The cleanup opens the pipe with `O_NONBLOCK`,
  runs on its own thread,
  and is waited for with a bound;
  the
  control now fails in five seconds with "the control's cleanup did not end within 5000 ms:
  opening the named
  pipe for a reader that can never come waits for ever" (planted) instead of holding the test process to the
  watchdog.
- The refusal diagnostic said a step "had not started",
  which is false when the job lost the claim race and
  the pool ran it on another thread.
  `a_refused_thread_start` decides in the one place that takes the claim:
  `NoThreadToStart` now says the step will not run and changed nothing,
  and the new
  `StartedAfterItsThreadWasRefused` says it has started and that whether it finished is not known.
  `OrderedRunError` carries the same pair,
  and the index report counts such a job as one whose result is not
  known.
  The control drives both orders and failed to compile before the fix (no decision,
  no variant).

#### The two controls the coordinator saw fail, measured

Both are controls that put their items in flight at once,
and the measured cause was the same in each:
the
process's room for blocking jobs,
and a control that had the item which *waits* hold the last place its peers
need.

- `the_report_keeps_the_order_of_the_checkout_then_its_worktrees` needs the checkout's rewrite to hold a place
  while it waits for the other three,
  so with room for exactly four the waiting item could take the last place
  its peers needed and the wait then ended only at its 10 s patience:
  measured at 14.7 s inside an 88-test
  selection and 0.05 s alone,
  and 33.9 s in the coordinator's run.
- `the_report_keeps_input_order_whichever_worktree_finishes_first` took 105 s (once past the 120 s watchdog)
  inside a 19-test selection and 1.1 s alone;
  with the host's own bound of two it failed at 3.8 s on its 3 s
  patience.
  What grew was not a wait but the storage work:
  it held the process to one place,
  so four worktrees
  were four conversions one after another,
  and one conversion takes about 28 s at a 2-CPU allowance while the
  host's disk is busy.

The seam is `room_for_blocking_jobs_in_a_control`:
the room a control asks for is what `budgeted_workers`
answers while it runs,
and that one derivation feeds both the runtime's blocking pool and the bound,
so a
control runs at any CPU quota instead of passing without running.
The two order controls ask for one place more
than their items (the waiting item never holds the last place its peers need),
the conversion controls convert
as little as their report needs (`WORKTREES` is two,
the bound controls make one worktree more than the overlap
they expect),
and the slowest conversion control measured 16.1 s at a 2-CPU allowance,
leaving the watchdog a
margin of seven.
The whole selection passes 218 of 218 at 100% and at 200%.

#### Checks

The folders this step touches,
each run on its own while it was written:

- `worktree::worktree_trash_entries` 7 of 7,
  `worktree::worktree_convert` 86 of 86,
  `subvolume::subvolume_index_files` 43 of 43 and its parallel controls 7 of 7,
  `subvolume::` 268 of 268 (one slow),
  `foundation::parallel_runner` 20 of 20 (the three runner forms).
- `native:clippy` clean;
  `//package/git-policy/cli:lint` 0 warnings and 0 errors
  (it found four line breaks and one avoidable closure,
  which the commits fix).
- The whole suites at the tip,
  `CLI_GIT_TEST_ROOT=/var/tmp/cg/cli-git/fixtures/root` and the run's own control group at
  `TasksMax=4000`:
  `native:test:host` 4826 run,
  4826 passed (1 slow),
  8 skipped;
  `native:test:binary:host` 733 run,
  730 passed (3 slow),
  3 failed,
  3 skipped;
  the same suite once more at the same tip,
  after the review's corrections landed:
  733 run,
  733 passed (3 slow),
  3 skipped.
- The three binary failures of the first run are one group,
  `service::the_settle_command_*`:
  `the_settle_command_finishes_a_batch_the_earlier_helper_left_open`,
  `the_settle_command_names_a_batch_it_could_not_examine_and_exits_nonzero` and
  `the_settle_command_is_not_held_by_a_service_another_control_is_starting`.
  All four controls of that group pass when they are asked alone
  (`native:test:binary:host -- service::the_settle_command`:
  4 run,
  4 passed),
  and the reason the command printed is a deadline,
  not a wrong answer:
  "the sockets and processes of the earlier helper could not be looked at in time",
  so it is not known whether one runs.
  The host carried a load average of 60 to 91 on 12 CPUs through the run,
  with another agent's whole host suite running beside it,
  and the same control is already recorded twice as a suite-only failure that passed alone and in
  the next full runs (`doc/handover/cli-git-rust-implementation.md`,
  and this plan's T4 section).
  No cause is measured here either,
  and the second run of the same suite at the same tip passed all 733,
  so the shape is the recorded one:
  a full run under heavy load fails that group and a full run
  does not.
  The path it exercises (the settle command's scan of `/proc`) is not one this step changes.

One defect this step did carry,
and the binary suite caught it rather than a control of its own:
with the conversion as a job of the blocking pool,
the content decision could no longer learn where the system configuration and attributes files are,
because the bridge that starts a child refuses to start one from inside the runtime.
`convert::the_system_attributes_file_is_located_through_the_real_git_too` and
`convert::a_worktree_the_stat_data_cannot_vouch_for_is_judged_by_content_not_refused_for_not_knowing_git`
failed with "the location of the system attributes file cannot be established without running Git",
and the fix is `LinuxPlatform::with_its_conversion_environment_settled`,
with the control `a_platform_settled_early_reads_the_facts_the_late_answer_gives`
(the plan's next section says what it pins).

## Open choices for the owner

Rankings are the recommendation;
each lists the reason for every adjacent pair.

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
  Eager in `main`,
  after the resource checks and the bounded build of choice 9.
  Pros:
  simplest to read,
  one place;
  the checks can run before the build.
  Cons:
  measured cost on every command,
  15 to 28% of what the wrapper adds over real Git on a forwarded command.
- C.
  Eager only for the service,
  lazy for the command.
  Pros:
  the service's runtime is built at its start.
  Cons:
  the same as A for the command;
  A already includes it for the service.
- Ranking:
  A > C > B,
  because A and C cost nothing on the forwarded path while B pays 0.3 to 0.5 ms on every command
  (A over C because C is A plus a statement about the service that A already makes);
  C > B because B carries the cost on the commonest command and C does not.

### 2. What "sized to budget" fixes

- A.
  Worker count from the budget (with the limit inputs) at build,
  and the concurrency of the work bounded by one
  process-wide semaphore whose permits travel with the jobs and whose size is refreshed by a fresh budget read when a
  round starts (recommended).
  Pros:
  honors "as many as the computer can spare at that moment" for the work,
  not only at start;
  permit lifetime follows the real job.
  Cons:
  the thread count is still the size at start (a service keeps it for hours),
  and resizing a shared semaphore
  needs a rule for overlapping rounds.
- B.
  Worker count from the budget at build,
  nothing else.
  Pros:
  one rule,
  the owner's words taken literally.
  Cons:
  concurrency is not bounded by the budget at all after the build.
- C.
  A runtime per round for the service.
  Pros:
  the size is fresh each time.
  Cons:
  thread start and stop cost on every round,
  and a dropped runtime waits forever for blocking work.
- Ranking:
  A > B > C,
  because A is B plus a bound on the work that gives the sentence its meaning;
  B > C because C pays a runtime build per round and needs the drop that the documentation says can wait forever.

### 3. Address-space protection

- A.
  Cap malloc arenas and read the limits (address space,
  threads,
  pids) before choosing workers (recommended).
  Pros:
  measured with the environment variable:
  28 workers fit in 100 MiB with the cap and hung in a band between 900
  and 1150 MiB without it.
  Cons:
  the in-process `mallopt` call matched the environment variable in the probe but was not tried in the wrapper,
  the
  cost in allocation contention is not measured,
  and the checks are advisory (choice 9 is what bounds the wait).
- B.
  Read the limits only.
  Pros:
  no allocator change.
  Cons:
  one worker is about 66 MiB of address space on this glibc,
  so the sizing allows few workers under a limit.
- C.
  Current-thread runtime for the command process,
  multi-thread only for the service.
  Pros:
  removes the worry from the commonest process.
  Cons:
  contradicts "Full on tokio.
  Multi-thread."
  for the command.
- Ranking:
  A > B > C,
  because A and B respect the owner's multi-thread ruling and A fits more workers in the same limit,
  on measurements that are not yet repeated in the real program;
  B > C because C overrides the owner's ruling.
  Neither A nor B makes construction failure-safe;
  choice 9 does the bounding.

### 4. The landing loop and thread-affine state

- A.
  One multi-thread runtime in the service,
  with the landing loop in a `LocalSet` on the thread that calls
  `Runtime::block_on` (recommended).
  Pros:
  one runtime;
  ten `thread_local!` items and one `Rc` keep working unchanged;
  the step is serial anyway.
  Cons:
  not yet tested in the wrapper;
  a landing task that blocks a call blocks that thread,
  as today;
  the proof of
  isolation (migration and interleaving tests) is still needed.
- B.
  A second,
  dedicated current-thread runtime for the landing step.
  Pros:
  the same state code;
  isolation by construction.
  Cons:
  two runtimes and two thread budgets in one process,
  against the owner's single-runtime reading.
- C.
  Turn each thread-local into a task-local or a context value.
  Pros:
  no thread affinity at all.
  Cons:
  the largest rewrite;
  a task that keeps a `!Send` value is still not spawnable;
  every test that sets a
  thread-local fault hook changes.
- D.
  Run the whole landing batch as one `spawn_blocking` job of synchronous code.
  Pros:
  nothing in the step changes.
  Cons:
  the step stays outside "Full on tokio",
  and a hung write holds a pool thread for good.
- Ranking:
  A > B > C > D,
  because A is B without the second runtime;
  B > C because B keeps the state code and C rewrites it for a step that gains no concurrency;
  C > D because C finishes the move that D declines.

### 5. Library API

- A.
  Async for the program-facing I/O libraries,
  sync for `safe-file-saving` (recommended).
  Pros:
  no `block_on` in a library;
  the runtime is started once by the program.
  Cons:
  a sync caller of an async library needs its own runtime.
- B.
  Every I/O library offers both,
  over one synchronous kernel with an asynchronous scheduling adapter.
  Pros:
  reusable by a sync caller;
  one implementation of the logic.
  Cons:
  two surfaces to test,
  and the owner said no one else will use the libraries.
- C.
  Everything async.
  Pros:
  uniform.
  Cons:
  `safe-file-saving` pays the hop on small calls and can reach a panicking `spawn_blocking`.
- Ranking:
  A > B > C,
  because A matches the measured costs and the owner's no-outside-user rule;
  B > C because B keeps the cheap calls synchronous and C does not,
  at the price of a second surface.

### 6. The signal relay between the existing rule and a registry

- A.
  Keep the one-guard-at-a-time rule and replace the sleep-and-retry with an asynchronous wait for the guard (a
  semaphore or a `Notify`),
  and move the child wait to the pidfd (recommended for now).
  Pros:
  signal semantics do not change;
  the polling loop goes.
  Cons:
  one more wait,
  against "Use as few locks as possible.";
  the service still runs one preparation's relay at a time.
- B.
  A registry of active children with a pidfd each.
  Pros:
  removes the refusal and supports concurrent preparations.
  Cons:
  needs a recipient rule (a signal to the service reaches every child,
  one operation's children,
  or none),
  a
  handler-safe publication and retirement protocol,
  and `errno` preservation;
  none is designed.
- Ranking:
  A > B,
  because A changes the wait without changing which child a signal reaches,
  where B changes both and has
  two open designs;
  B stays the target once the owner rules on recipients.

### 7. How children are started and awaited

- A.
  Standard-library start,
  pidfd,
  `AsyncFd`,
  timer-driven `try_wait` as the fallback,
  reap with the standard-library
  child,
  with the helper owning the child through every failure after the spawn (recommended).
  Pros:
  a failed start leaves no child (the guarantee of `start_retry.rs` is kept);
  the pidfd is shared with the signal
  relay;
  the fallback needs no descriptor and no thread.
  Cons:
  about a hundred lines of our own code where tokio has a finished type;
  the fault-injection seam is ours to
  build;
  none of it is built.
- B.
  Plain `tokio::process::Command`.
  Pros:
  published code with nothing to maintain;
  the owner's word `tokio::process` taken as written.
  Cons:
  a start can fail after the child has started (read from source,
  not triggered),
  which `start_retry` cannot undo,
  and the failure cannot be injected from outside to test it.
- C.
  Plain `tokio::process::Command` with `kill_on_drop(true)`.
  Pros:
  none that holds:
  the drop guard is built after the failing call.
  Cons:
  a stop by drop is not a stop by no progress,
  and the steps that leave a child running cannot use it.
- D.
  Report the gap upstream and wait for a tokio fix,
  using B meanwhile.
  Pros:
  no code of ours.
  Cons:
  waiting,
  and B's failure stays until then.
- Ranking:
  A > B > D > C,
  because A keeps the guarantee the start retry rests on;
  B > D because D is B plus a wait that changes nothing today;
  D > C because C offers no protection and adds a kill the no-progress rule does not allow.

### 8. When tokio starts relative to the layout moves

- A.
  Add-only steps T1 to T3 after the release,
  conversions after the grouping and links of split steps 7 and 8
  (recommended).
  Pros:
  no signature conflict with the files in motion.
  Cons:
  the conversions wait for the layout work;
  T1 to T3 still touch the catalog,
  the lockfile and `HostResources`.
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
  T1 could land at once and costs nothing in the layout's way.
- Ranking:
  A > C > B,
  because A takes C's safety and adds the steps that change no signature;
  C > B because B delays the owner's stated first priority.

### 9. A bound on the runtime build

- A.
  Build the multi-thread runtime on a helper thread,
  wait with a deadline,
  fall back to the current-thread runtime
  (recommended).
  Pros:
  the wait is bounded whatever the limits do;
  measured hangs in the unwind would not hold the command.
  Cons:
  a possibly hung helper thread stays until exit;
  the deadline is a number that needs a measured basis
  or an owner decision recorded beside the value;
  not yet built or measured.
- B.
  Checks only (choice 3).
  Pros:
  no extra thread.
  Cons:
  the checks are advisory;
  a build that still fails hangs inside the unwind.
- C.
  Current-thread runtime only.
  Pros:
  no thread,
  no hang.
  Cons:
  contradicts "Multi-thread".
- Ranking:
  A > B > C,
  because A bounds the wait that B can only make less likely;
  B > C because C overrides the owner's ruling.

### 10. What "Commits should never fail." means for the gate

- A.
  A commit that native Git would complete completes,
  under the limits of the fault suites (the strong reading).
  Pros:
  matches the sentence;
  makes the fallbacks necessary.
  Cons:
  some limits (a refused process start) make native Git fail too,
  so the suite needs a native-Git control per
  case.
- B.
  A commit fails only with a plain message and the repository recovers (what the suites assert today).
  Pros:
  already built.
  Cons:
  accepts failures the sentence rejects.
- Ranking:
  A > B,
  because A is the sentence and B is its weakest part;
  the cost is the native control per case.

### 11. Calls that can wait on storage when no thread can start

- A.
  Inline fallback allowed,
  with an external bound (the client's deadline on the service,
  the service manager's
  watchdog) for the calls that can wait on storage (recommended).
  Pros:
  keeps "a thread is only a way to work sooner";
  commits still land where no thread starts.
  Cons:
  the in-process timers cannot end an inline call;
  the external bounds are not designed.
- B.
  No inline fallback for calls that can wait on storage;
  they fail with a plain message when no thread starts.
  Pros:
  nothing waits unbounded in process.
  Cons:
  breaks "Commits should never fail."
  in that state.
- Ranking:
  A > B,
  because A keeps a commit landing where B gives it up,
  at the price of an external bound that today's
  code does not have either.

### 12. What a stop of the service does to the Git child of a commit it is preparing

Raised by T7 (written 2026-11-07T15:50:00-05:00).
The owner's answer to choice 6 says a signal reaches only the children of the operation it belongs to,
and that a stop of the service is an operation of its own.
Before T7 a commit prepared inside the service swapped the service's stop handler for the length of the Git step,
so a stop of the service reached that one Git (and the service went on running).
T7 builds the owner's rule literally (option A),
which leaves that Git running when the service is stopped.

- A.
  The stop of the service has its own handling and signals no commit's Git child (as built).
  Pros:
  it is the owner's recipient rule as worded;
  the service's stop handler is never replaced,
  so a stop is always the service's;
  nothing in the handler touches a child.
  Cons:
  a Git that the service was running when it was stopped is left to end by itself,
  and holds its index lock for as long as it takes (a commit step is bounded by the wrapper's own limits,
  not by the stop).
- B.
  The stop of the service signals every child of every operation the service runs.
  Pros:
  no Git outlives the service,
  as in the single-commit case before T7.
  Cons:
  it is the recipient rule the owner did not pick ("a signal to the service reaches every child");
  the one slot of T7 becomes a table that a handler reads,
  with a bound on its size
  (a shipped limit that needs a measured or owner-decided figure) and a proof for its publication and retirement.
- C.
  The stop of the service is an operation that ends the children it names:
  the handler only records the stop,
  and the service's own shutdown code,
  not the handler,
  signals the children of the operations it waits for,
  through a registry of those children.
  Pros:
  the stop is an operation of its own,
  as the owner said;
  no Git outlives the service;
  the handler stays minimal and safe.
  Cons:
  not designed:
  the registry is a table,
  the shutdown path must wait a bounded time for the children it signalled,
  and "nothing should ever wait forever" applies to that wait.
- Ranking:
  C > A > B,
  because C ends no stray Git and still treats the stop as an operation of its own,
  which A gives up for simplicity;
  A > B because A is what the owner's rule says and B is the rule the owner declined.
  The agent built A because it is the only option that needed no new design,
  and it is the narrowest;
  a veto of A in favour of C is enough to start C.

The owner's pick,
2026-11-07T22:19:43-05:00,
verbatim label:
"C:
shutdown stops its Gits (Recommended)".
Option C is the design:
the stop handler only records the stop,
and the service's own shutdown signals the Git processes it is waiting on,
through a registry of those children.
Built in T11,
after T10 merges,
under the owner's rules:
"Use as few locks as possible."
(one owner per piece of state,
not a shared table),
"nothing should ever wait forever or indefinitely" (every wait after the signal is bounded,
then `SIGKILL`),
and "never discard user data" (a commit whose Git is stopped keeps its work on its side ref,
as withdrawals do).
Red tests first:
a stop during a long-running commit Git leaves no Git process and no `index.lock` behind within the bound.

Built in T11 (2026-11-07T23:40:00-05:00):
see "Step T11".

## Cross-vendor review

Reviewed by `openai/gpt-6.1-sol` (a different vendor from the author;
no tools,
reading only this document as committed
at `687074ecf2`;
output in `${HOME}/temp/agent/full-tokio/sol-review.txt`).
It judged the plan "not ready for approval as a correctness-preserving migration sequence" and named five blockers.
Findings are grouped by what was done.

Taken,
and the text changed:

- Resource checks cannot make a multi-thread build failure-safe.
  The section "A multi-thread runtime panics,
  then hangs" now says the checks lower the chance and cannot remove it,
  counts threads and ancestor cgroups,
  and adds a deadline-bounded build on a helper thread with a fall back to the
  current-thread runtime (new open choice 9);
  the planted control in the risks is now the removal of the deadline.
- The inline fallback is not bounded by in-process timers.
  The section "Blocking-pool work panics" states the limit,
  restricts the fallback to the state where the OS refused a
  thread,
  and moves the exposed calls to an external bound (new open choice 11).
- The child helper must keep the child through every failure after the spawn.
  The section on `tokio::process` states the invariant,
  makes the fallback a timer-driven `try_wait` instead of a
  blocking-pool wait,
  orders the cleanup,
  and asks for an injection seam in the helper.
- `kill_on_drop` does not protect a failed start.
  Checked in the source:
  the drop guard is built after `imp::build_child(child)?` returns,
  so the claim for option C was
  false;
  option C now has no pro and ranks last,
  and an upstream-report option (D) was added.
- The signal registry lacks a recipient policy and a handler-safe lifetime.
  The section on the relay names the descriptor-reuse hazard and `errno`;
  choice 6 is re-ranked so the guard rule with an
  asynchronous wait comes first and the registry stays the target.
- Runtime shutdown does not establish data safety.
  The section on shutdown separates work that may be abandoned,
  work that must stay intact,
  and durability that must be
  complete;
  it cites that `serve` already drains the durable-sync queue before every exit;
  a risk was added.
- `Send` checking is a partial check for thread-local state.
  The thread-affine section and the risk now ask for migration and interleaving tests and for a positive control of the
  test fault hooks.
- A `LocalSet` needs no second runtime.
  Taken:
  choice 4 now ranks a `LocalSet` on the thread that calls `block_on` of the one multi-thread runtime first;
  that design is not tested in the wrapper.
- Synchronous bridges can nest a runtime.
  The runtime section and the steps now require a try-current check that refuses instead of panicking,
  order the steps
  leaf first,
  and keep `plan_invocation` synchronous so the forward path never builds a runtime.
- A per-round semaphore bounds only its round.
  The budget section and choice 2 now use one process-wide object with permits owned by the job.
- Library conclusions were overstated.
  The section is reworded as proposals,
  the claim that both surfaces need two implementations is dropped,
  and the
  `fsync` case is separated from the metadata measurement.
- "No asynchronous fsync" was misleading,
  and "every `tokio::fs` operation panics" exceeded the probe.
  Both reworded.
- The `SIGCHLD` fallback is not proven harmless,
  and the allocator measurements are not universal costs.
  Both reworded;
  the in-process `mallopt` was then measured in the probe (same sizes as the environment variable).
- The reactor-failure test did not target the cited failure,
  some tests could pass without exercising the property,
  and
  the thread-count test used a snapshot.
  The risk list is rewritten (an injection seam;
  a test-only entry for the builder in T3;
  `strace` clone counts in T2;
  ported assertions with planted controls in T5).
- The timing denominators.
  Corrected:
  the wrapper's own added time over real Git is 1.8 to 2.0 ms,
  so the 0.3 to 0.5 ms is 15 to 28% of it;
  "costs nothing" became "not resolved by this experiment";
  "commonest path" is marked as unmeasured usage;
  the file
  count is reconciled (623 non-test and 665 test files make 1288).
- The acceptance criteria are weaker than "Commits should never fail."
  Added as open choice 10.
- Steps are not independently reversible,
  and T1 to T3 are not conflict-free.
  Reworded to "newest first" and "signature-free,
  not conflict-free".
- Rankings of choices 1 to 8 were corrected where the review found a wrong pro or con (choice 1 B no longer says the
  build cannot be made safe,
  choice 2 distinguishes the thread count from the concurrency,
  choice 3 carries the
  unmeasured caveat,
  choice 5 drops the duplicate-implementation claim).

Not taken as written:

- "A syntactic check for `current_exe` is not a complete proof against indirect self-invocation."
  Agreed as a statement;
  the guard stays as step T1 and the plan does not claim it is a proof.
  The existing runtime marker (`started_as_git.rs`) is the other half and is unchanged.

## Review findings and what was done

The list in the section "Cross-vendor review" is the record.
Nothing the reviewer raised was left without a change or a stated reason.

## Step T12, the landing round: what blocks it and what now pins it

- Branch and worktree:
  `feat/git-policy-cli-tokio-landing-round-on-the-runtime` in
  `.claude/worktrees/pi-tokio-landing-round-on-the-runtime`,
  from main at `baf3d03fe6`,
  and main merged into it again at `5ffe9b2dcc` when it had moved to
  `6232f281d7`.
  Commits `02601dbb76`,
  `8af808f1c0`,
  `ba49929c1c` and the merge `5ffe9b2dcc`.
  Not merged into main.
- Done:
  the wording of the raw runner's refusal (the finding of
  `${HOME}/temp/agent/reviews/tokio-pure-file-sites-round-5-sol.txt`),
  and the controls the move of `landing/service_round.rs` has to keep.
- Not done:
  the move itself.
  It is not a contained change and the reasons are below,
  measured.

### Why the round did not go onto the blocking runner

The brief for this step named `run_ordered_on_the_runtime`
(`foundation/parallel_runner_on_the_runtime.rs`).
Its items are jobs of the blocking pool,
and a job of the pool is inside the
runtime:
`tokio::runtime::Handle::try_current()` is `Ok` on a thread named
`git-policy-worker`,
which the three controls named `..._run_on_the_processs_runtime` record
(`subvolume/subvolume_index_parallel_tests.rs:199`,
`worktree/worktree_trash_entries_tests.rs:145` and
`worktree/worktree_convert_parallel_tests.rs:454`),
against `false` on a scoped thread.
Every Git step the round runs reads its child through the runtime:

- a step ended at a ceiling reads its pipes through
  `read_until_the_child_ends`
  (`process/step_output_read_whole_through_pipes.rs:145`:
  `run_on_runtime`),
  and
- a push goes through `run_bounded_push_once`
  (`push/bounded_push.rs:388`:
  `run_on_runtime`),
  which the auto-push,
  the reconcile and `git-lfs`'s upload all use.

Both refuse from inside the runtime with `RuntimeError::Nested`,
so a round
whose items were jobs would fail every push and every maintenance run,
and
`drain_pending`'s verdicts would requeue them for ever.
The control that measures this:
`landing::service_round::tests::a_git_read_asked_inside_a_blocking_job_of_the_pool_is_refused`.

The form the round can use is
`run_ordered_awaiting_on_the_runtime`
(`foundation/parallel_runner_on_the_runtime_awaiting.rs`),
whose module comment already says the round cannot use the blocking form.
That form needs four things that do not exist yet:

1. An awaited push chain.
   `WorkHandlers::push` reaches `push_once` (`push/auto_push.rs:262`),
   which reaches `run_query` (`push/auto_push_single_flight.rs:230`),
   `plan_named_push` (`:216`,
   through `read_destination` and `named_push` of
   `push/auto_push_destination.rs`),
   `run_push` (`lfs/lfs_bounded_push.rs:98`,
   whose `GitQuery` trait and `read_settings`/`plan_upload` are synchronous)
   and `reconcile_divergent_push` (`push/auto_push_reconcile.rs:75`,
   with the `MergePusher` trait).
   Awaiting twins must keep the synchronous faces,
   because `manual_push_probe`
   and `push_stacking` call the same functions from plain threads.
   About a dozen functions in six files,
   plus the two traits and their doubles.
2. An awaited maintenance.
   `read_maintenance_config` and `run_auto_maintenance_with`
   (`landing/maintenance_run.rs:156` and `:227`)
   run `git config` and `git maintenance run`,
   both ended at a ceiling.
   Two functions.
3. A claim renewal whose stop does not wait for a task.
   `drain_pending` starts one (`ClaimRenewal::start`,
   `landing/service_claim_renewal_running.rs:275`)
   and its `Drop` waits for the task up to `stop_ceiling`.
   That is allowed on the service's work thread and forbidden on a worker:
   the measured rule of the earlier step says a synchronous `Drop` that waits
   for a task holds a worker,
   and a drain that is a task would hold one.
4. The claims,
   the settles and the notices put on the blocking pool.
   `drain_pending` (`landing/service_work_drain_pending_claims.rs`) interleaves
   `claim_all_of`,
   `settle_all`,
   `record_push_failure`,
   `clear_push_failures_before` and
   `next_claim_expiry` with the handler calls,
   and the plan's item 2 asks for each item's storage work inside
   `run_blocking_job_within_the_budget`,
   which needs the held claims owned by the job.

Items 1 and 2 are the leaf-first conversion's own next folders.
Items 3 and 4 are the landing loop's (T11's remainder).
A step that did any one of them alone would leave the round where it is,
and a
step that did all four is the largest piece left in T12,
on the path that
pushes the person's backup.

### What the controls pin, and the defects they catch

`landing/service_round_tests.rs`:

- `the_answer_of_a_round_is_the_one_a_drain_of_one_directory_after_another_gives`
  compares the round's `RoundAnswer` with an oracle that runs the same item
  work (`DrainEachDirectory::run`) serially on the calling thread,
  over two fixtures with one claim another instance holds and one push that
  could pass each.
  The waits are compared by directory and within five seconds (two readings of
  one lease),
  the plan changes exactly.
  Planted,
  the consumer attributing a wait to the last directory instead of its own:
  `left: ["wait 3", "wait 2", "wait 1", "wait 0", ...]`
  against
  `right: ["wait 0", "wait 1", "wait 2", "wait 3", ...]`.
- `a_lane_drains_its_directories_in_the_order_they_were_named`
  gives one repository a linked worktree and shows the lane drains the common
  directory and then the linked one.
  Planted,
  the lane taking its items last first:
  `left: [.../linked, .../common]` against `right: [.../common, .../linked]`.
- `a_refused_thread_partway_through_a_round_leaves_no_directory_undrained`
  refuses every scoped start on the calling thread
  (`scoped_start::RefusedStarts`) and shows the answer and the order of the
  pushes are the ones a round with every thread gives:
  every lane runs on the
  calling thread,
  no directory is lost,
  and the caller is told nothing.
  Planted,
  the runner dropping the lanes a refused thread would have run:
  `left: []` against the whole answer.
  What the converted round must say instead is `OrderedRunError`:
  the items after the refused one in its lane do not run,
  their claims are left
  to the lease of their holder
  (`service_work::CLAIM_LEASE`),
  and the next round of the work loop finds them
  again.
  Nothing is lost,
  and the caller is told the refresh did not finish.

`a_git_read_asked_inside_a_blocking_job_of_the_pool_is_refused` has no plant:
it measures a state of the code rather than guarding a behaviour,
and the same read as a task of the runtime answers,
which is the state its refutation would be.

The wording of a refusal:

- `a_refusal_from_this_runner_says_the_step_may_still_run_and_claims_no_state_of_it`
  fails on the old sentence (quoted in the commit) and passes on
  `MayStillRunAfterItsThreadWasRefused`.
- `the_refusal_that_promises_no_change_names_the_claim_that_makes_it_sure`
  pins the answer `run_blocking_job_within_the_budget` gives,
  and that it names
  the claim as what makes it sure.

### Checks

- `foundation::blocking_job`,
  `foundation::blocking_jobs_within_the_budget`,
  `foundation::parallel_runner` (the three runner forms) and
  `foundation::scoped_start`:
  35 of 35 and 3 of 3.
- `landing::service_round`:
  10 of 10.
- `native:clippy` clean;
  `//package/git-policy/cli:lint` 0 errors and 0 warnings,
  after `format:monochromatic-lint` wrote four line breaks it asked for.
- The whole suites at the tip,
  `CLI_GIT_TEST_ROOT=/var/tmp/cg/cli-git/fixtures/root` and the run's own
  control group at `TasksMax=4000`:
  `native:test:host` 5087 run,
  5087 passed (one slow),
  12 skipped,
  and `native:test:binary:host` 744 run,
  736 passed (four slow),
  8 failed,
  3 skipped.
  At the merged tip (`5ffe9b2dcc`) the same two suites were run again:
  `native:test:host` 5164 run,
  5164 passed (one slow),
  12 skipped,
  and `native:test:binary:host` 757 run,
  748 passed (three slow),
  9 failed,
  3 skipped,
  the nine being the eight below and
  `client_service::an_index_writer_waits_for_the_service_to_finish_landing_and_says_so`,
  which fails alone and fails the same way with this branch's four production
  files taken from `origin/main`,
  so it came with the merge and not with the step.
- The eight binary failures are not this branch's,
  measured:
  `fault_trace::a_worktree_copy_killed_at_any_call_leaves_a_recoverable_repository`,
  `package_repair::pnpm_runs_after_the_snapshot_rung`,
  `service_provision::the_service_pushes_after_the_commit_returned`,
  `version_cache::a_commit_records_and_then_trusts_the_version_text`,
  `worktree_copy::a_filter_that_creates_a_worktree_through_the_wrapper_settles_it_itself`,
  `worktree_dirty_snapshot::quiet_silences_the_note_about_left_out_changes`,
  `worktree_snapshot::a_clean_main_checkout_is_snapshotted_and_equals_gits_own_checkout`
  and `worktree_snapshot::a_snapshot_of_main_sets_aside_what_the_rule_leaves_out`.
  All eight fail alone as well as in the suite (8 run,
  8 failed),
  and all eight fail the same way with this branch's four production files
  restored to `baf3d03fe6` (8 run,
  8 failed),
  which is the measurement that they predate the step.
  The folders are `worktree_copying/`,
  `version_cache`,
  `package_repair`,
  `service_provision` and `worktree_copy`,
  none of which this branch reaches.
  Two shapes are worth the next agent's look,
  and neither is diagnosed here:
  three of them fail on a sentence that says
  "received no ignored-state copy:
  the worktree this command registered is no
  longer at that path",
  and `version_cache::a_commit_records_and_then_trusts_the_version_text` found
  two cache files where it expects one,
  the second being `git-system-attributes-<hash>`,
  which `LinuxPlatform::with_its_conversion_environment_settled` writes.
- `native:perf`,
  five arms on 2026-11-09 between 07:50 and 08:00,
  with two binaries:
  this tree at the tip,
  and main's tree at `baf3d03fe6` (its four production files restored,
  built,
  and the tree restored,
  whose rebuild produced a binary byte for byte the one this tree's arms
  measured).
  The arms alternate (main's,
  this tree,
  main's,
  this tree,
  this tree),
  so the last two are the same-binary control for the spread.
  The host carried a load average of 53 to 121 on six CPUs throughout.
  Paired deltas against native Git,
  commit and signed commit through the service:

  - 07:50 main's:
    commit median 2,568.8 ms,
    maximum 27,094.7 ms;
    signed median 1,503.8 ms,
    maximum 16,886.2 ms.
  - 07:52 this tree:
    commit median 853.9 ms,
    maximum 13,714.4 ms;
    signed median 404.9 ms,
    maximum 10,262.4 ms.
  - 07:55 main's:
    commit median 508.5 ms,
    maximum 25,570.1 ms;
    signed median 499.9 ms,
    maximum 12,497.2 ms.
  - 07:58 this tree:
    commit median 57.6 ms,
    maximum 480.7 ms;
    signed median 189.1 ms,
    maximum 17,658.5 ms.
  - 07:59 this tree:
    commit median 90.3 ms,
    maximum 3,857.3 ms;
    signed median 57.3 ms,
    maximum 5,897.3 ms.

  What that says:
  the same binary's own commit medians ran from 57.6 to 853.9 ms (14.8 times)
  and its signed medians from 57.3 to 404.9 ms (7.1 times),
  which covers every difference between the two binaries,
  and main's own arms ran 508.5 to 2,568.8 ms,
  so the order of the arms is the load's,
  not the binaries'.
  Both binaries broke the 5,000 ms bar under this load:
  main's at 25.6 s and 27.1 s (commit) and 12.5 s and 16.9 s (signed),
  this tree's at 13.7 s (commit) and 10.3 s and 17.7 s (signed),
  with the last arm's 5.9 s signed the only one near it,
  and no file this step touches runs in a commit.
  Forwarded commands,
  the added medians of `rev-parse`,
  `status`,
  `log` and `diff-stat`,
  stayed between 1.4 and 3.6 ms in every arm,
  three orders of magnitude under the 2,000 ms bound.
- Two controls of `landing::service_work::push_retry_tests` failed inside one
  folder selection
  (`a_push_plan_due_later_does_not_hide_the_startup_look` and
  `a_stopped_push_is_still_tried_again_after_the_service_restarts`) and pass
  alone:
  measured with both arms of the change,
  two runs each,
  2 of 2 in four runs,
  under a load average of 62 to 73 on six CPUs,
  and the failing run's own push took 10.2 s against a patience of about 10 s.
  Their shape is the recorded one:
  a witness with a time window that a loaded host reaches.

### Step T12, the repair folder's threads become waits of the runtime, 2026-11-09T10:55:00-05:00

- Branch and worktree:
  `refactor/git-policy-cli-repair-threads-on-the-runtime` in
  `.claude/worktrees/pi-tokio-repair-threads-on-the-runtime`,
  from main at `4da5949330`.
  Not merged into main.
  Commits `875d4a208a` (the fan-out as tasks and pool jobs) and `3803520ca0` (the three tools as three waits).
- Converted,
  in `package/git-policy/cli/src/native/repair/`:
  `package_repair.rs` (three tools,
  once on scoped threads),
  `package_repair_tree_clone.rs` (one scoped thread per slice),
  `package_repair_cargo_concurrent.rs` (one scoped thread per worker).
  Each is now a wait of a task:
  the three tools run under `tokio::join!`,
  and both fan-outs take items from a queue of tasks.
  Every storage step is `run_blocking_job_within_the_budget`:
  the clone's walk,
  its slices and its modes,
  and the search path and real path of the Cargo repair.
- The synchronous faces stay (`repair_worktree`,
  `repair_pnpm`,
  `repair_toolchain`,
  `repair_cargo`,
  `clone_tree`,
  `read_workspace`),
  because `worktree_copying/` calls `repair_worktree` and is reserved by this step's brief,
  and their bodies are the awaited ones entered once
  (`run_on_runtime`).
  The awaiting faces beside them are `repair_worktree_awaiting`,
  `repair_pnpm_awaiting`,
  `repair_toolchain_awaiting`,
  `repair_cargo_awaiting`,
  `clone_tree_awaiting`,
  `read_workspace_awaiting` and
  `manifest_directories_awaiting`;
  the tool bridge gained `run_tool_awaiting` (`run_tool_reaping` awaited directly)
  and `run_refused` keeps the words a tool that could not be run is reported with.
- The ordered runners are not used by this folder,
  and that is measured:
  `OrderedRunOfAwaitingOwned::on_event` is `&'run mut dyn OrderedEvents<R>`,
  which is not `Send`,
  so `run_ordered_awaiting_on_the_runtime` cannot be awaited inside a task
  (`E0277: ... cannot be sent between threads safely`),
  and every lane or item of this repair is a task,
  the worktree repair's own task included.
  A later step that wants nested runs either makes that consumer `Send` or keeps a run at the top
  of a chain.
- Two rules the repair keeps:
  `run_a_job_or_in_turn` runs a refused job's work on the awaiting task,
  because `BlockingJobError::NoThreadToStart` promises the job will not run and changed nothing
  (`foundation/blocking_jobs_within_the_budget.rs`),
  and holds a place of the process's bound across that work
  (the first review round,
  below);
  `call_catching_panics_awaiting` turns a panic of one item's awaited work into that item's error,
  because a task that panics ends the run
  (`foundation/call_catching_panics_awaiting.rs`).
  Each was shown to fail before it was restored:
  the first by `left: Err(NoThreadToStart)` in `a_job_refused_a_thread_runs_in_turn` and by
  "source could not be cloned" in
  `a_parallel_clone_where_no_thread_can_start_is_made_in_turn`;
  the second by `item 1: left: None, right: Some(10)` in
  `a_panicking_item_is_a_failure_naming_it_and_the_others_keep_their_results`.
- Measured with a temporary probe (deleted before the last commit),
  the two builds alternating,
  three rounds,
  under a load average of about 50 on eight CPUs;
  each figure is five runs of one process:

  the three tools (two fixtures sleeping 40 ms):
  before 42.6 to 44.0 ms,
  after 42.7 to 46.0 ms,
  so no separation.
  The six Cargo workspaces (three tool runs of 40 ms each):
  before 133.3 to 138.5 ms,
  after 132.4 to 135.2 ms,
  a difference under the spread.
  The clone of 1,200 small files in 40 directories:
  before 20.9 to 57.5 ms,
  after 32.7 to 44.2 ms:
  the after arm's spread is narrower and its median about 3 ms higher,
  which is the cost of one pool job per slice,
  and the before arm's spread is the host's disk.
- Controls:
  `repair::` 350 tests passed with `foundation::call_catching`,
  `foundation::blocking_jobs_within_the_budget` and `worktree_copying::`;
  the new ones are `the_tools_run_beside_each_other`,
  `a_refused_system_is_reported_in_plain_words_and_changes_nothing`,
  `a_job_refused_a_thread_runs_in_turn`,
  `a_panicking_item_is_a_failure_naming_it_and_the_others_keep_their_results` and
  `no_more_items_are_inside_their_work_at_once_than_the_run_asks_for`.
  `native:clippy` clean;
  the package `lint` and the container gate were not both seen green here
  (see the agent's report).
- Not converted:
  `worktree_copying/` (its `finish_trash_and_repair` keeps one scoped thread for the trash beside the repair)
  and `commit/`,
  both reserved by the brief for branches that have not landed.

### Step T12, the first review round, 2026-11-09T12:46:31-05:00

- Branch and worktree:
  `refactor/git-policy-cli-repair-threads-on-the-runtime`,
  with main at `ab2839b414` merged as `4004f6b562` (45 commits,
  no conflict).
  The review read source only
  (`~/temp/agent/reviews/tokio-repair-round-1-sol.txt`),
  and every finding was reproduced before it was fixed:
  each fix has the control that failed without it,
  and the failing output is quoted in its commit message.
- `repair/package_repair_toolchain.rs`,
  the record read at `lock_named_by_record` and the awaiting face.
  Held:
  the read ran on the task of `tokio::join!` in `package_repair.rs`,
  so a `.cache/toolchain-launchers/*.launch` entry that is a named pipe with no writer
  stopped the pnpm repair's quiet window and ceiling together with the read.
  This is new to the branch:
  the toolchain repair had a thread of its own before it.
  Fixed by `91bb19ccab`:
  only a regular file is opened,
  so an entry of any other kind is left alone,
  and both the decision before the install and the count after it are jobs of the blocking pool
  (`prepare_install_within_the_budget`,
  `count_stale_within_the_budget`).
  A count that could not be read turns a success into a failure that names the reason,
  so nothing reports a success it did not check.
  The synchronous face keeps its caller-thread reads:
  it runs on the caller's own thread before and after its one entry into the runtime and polls no wait there.
  Controls,
  written first in `19181978d8`:
  `a_record_that_is_a_named_pipe_is_left_alone` and
  `the_launcher_records_are_read_as_a_job_of_the_pool`.
  The first failed with "nothing ended the wait" after ten seconds,
  and the second with "the launcher records were read without asking for a place of the process's bound on blocking jobs".
- `foundation/blocking_jobs_within_the_budget.rs`,
  the fallback of `run_a_job_or_in_turn`.
  Held:
  `a_refused_thread_start` gives the refused job's place back before answering,
  so the work the fallback then runs in turn held no place,
  and a second job took that free place and ran beside it.
  Fixed by `bdb21a6329`:
  the fallback takes a place again and holds it across the work,
  which is the one lock-free place the process already has and no new lock.
  Control,
  written first in `7d98660e8e`:
  `a_fallback_that_runs_in_turn_holds_a_place_of_the_bound`
  reads the bound while the fallback's work is open,
  and reads it free before that work starts,
  so the measurement is shown to see both states.
  It failed with `left: 1, right: 0`.
- `repair/package_repair_cargo_concurrent.rs`,
  the build of an item's future.
  Held:
  `queue.job.run(item)` is evaluated before `call_catching_panics_awaiting` exists,
  so a job that panicked there ended the worker task,
  which dropped the answers it had finished,
  discarded its panic text at `join_next`,
  and never took the items after it.
  Fixed by `8da095c454`:
  the build travels through `while_panics_are_expected` and becomes that item's error,
  every answer is handed to the run as its item ends instead of travelling in the task's return value,
  and a task's own panic is recorded rather than dropped.
  Control,
  written first in `1c04a543b1`:
  `a_panic_while_an_items_future_is_built_stays_that_items`,
  which failed with `[Err("item 0 gave no result"), Err("item 1 gave no result"), Err("item 2 gave no result")]`.
- Controls of the round:
  `repair::`,
  `worktree_copying::`,
  `foundation::blocking_jobs_within_the_budget`,
  `foundation::call_catching` and the two place and claim modules beside them,
  392 tests passed;
  `native:clippy` clean (it caught one shadowed binding the fix introduced,
  which is renamed);
  `lint:monochromatic-lint` reports nothing in the six files this round touched.
- Not done here:
  the container gate (`native:test:container`) was not run,
  the mutation campaigns stay paused by the owner,
  and Win and mac were left alone.

### Step T12, the second review round, 2026-11-09T13:21:18-05:00

- Review:
  `~/temp/agent/reviews/tokio-repair-round-2-sol.txt`,
  source only.
  Main was already merged into the branch as `f7e02da83e` when the round started.
- `repair/package_repair_cargo_workspaces.rs`,
  Held:
  `manifest_directories_awaiting` ran `directories_of_the_listing`,
  and so `path.is_file()` for every tracked `Cargo.toml`,
  on the task that polls the three repairs under `tokio::join!`;
  a filesystem that does not answer a lookup would hold the pnpm repair's quiet window and ceiling with it.
  Fixed by `5f257d9bf1`:
  the listing travels as a job of the blocking pool through `run_a_job_or_in_turn`,
  and a listing that cannot be read is an error naming the refusal,
  which the Cargo repair reports as that workspace's failure.
  Control,
  written first in `4f96ad5a4b`:
  `the_manifest_directories_are_read_as_a_job_of_the_pool`,
  which failed with "the tracked manifests were read without asking for a place of the process's bound",
  and passed after,
  with all 79 tests of `repair::`.
- `foundation/blocking_jobs_within_the_budget.rs`,
  the fallback of `run_a_job_or_in_turn`,
  is now a recorded limit and not a defect:
  `SPEC.md` (`7a1fe48708`,
  "Package-manager repair of a new worktree") says that where the system refuses to start a pool thread
  the job's work runs on the shared task,
  so the person sees the add wait with no tool line and no ceiling firing until that filesystem answers,
  and raising `ulimit -u` or the control group's `pids.max` and running the command again is the remedy.
  No code change,
  by the owner's word of 2026-11-09.
- `repair/package_repair_toolchain.rs`,
  the `metadata` then `read_to_string` of a launcher record,
  is dismissed by the owner on 2026-11-09:
  nothing on this host replaces a record with a pipe inside that window,
  and the round-1 type check stays as it is.
  The owner's rule for these rounds:
  "If you ask GPT to find issues,
  it will find issues.
  Be sure to dismiss them via your own judgement at some point."
- Found by the round's search,
  and **not** changed,
  because the round's brief named one site and the owner asked for that site and the `SPEC.md` note:
  three synchronous filesystem calls still run on the task that polls the repairs.
  `package_repair_pnpm.rs` lines 116 and 121:
  `prepare_relink` asks `is_file()` and `is_dir()` of the worktree,
  and walks the search path (`find_program`),
  before the relink is awaited.
  `package_repair_cargo.rs` lines 229,
  242,
  246 and 247:
  `repair_workspace_awaiting` asks `is_file()` for the source manifest,
  and `exists()` for three state directories.
  `package_repair_cargo.rs` lines 268 and 270:
  `read_lock` reads the workspace's `Cargo.lock`,
  and `restore_lock` puts that file back around the awaited clean.
  Each is a candidate for the same job treatment,
  and the pnpm one is the largest,
  since a worktree that answers no lookup holds the other two repairs' limits through it.
- Not done:
  the container gate (`native:test:container`) was not run,
  and no mutation campaign was started (paused by the owner).

## Step T12, the snapshot source and the clean check move to the runtime, 2026-11-09T14:35:00-05:00

- Branch and worktree:
  `refactor/git-policy-cli-worktree-snapshot-and-clean-threads-on-the-runtime` in
  `.claude/worktrees/pi-tokio-worktree-snapshot-and-clean`,
  from main at `a2d927fb55`,
  with main merged as `4781ab04a2` when it had moved to `33562f4ff7` (233 commits,
  no conflict).
  Not merged into main.
  Commits `3cb3e4efbc` (the controls),
  `a334dfe771` (the clean check),
  `f7ae9e8f09` (the source's three checks) and
  `41dd04d1c5` (the two splits the line limit asked for).
- Converted,
  in `package/git-policy/cli/src/native/worktree/`:
  `worktree_clean.rs` (the untracked scan and the tracked proof,
  once one scoped thread beside the calling thread) and
  `worktree_snapshot_source.rs` (three reads,
  once two levels of `run_beside_caller`).
  Each is now a wait of a task:
  the scan is
  `scan_for_untracked_file_awaiting` over `run_git_awaiting`,
  the tracked proof is `run_a_job_or_in_turn` of
  `check_tracked_files_in_a_job`,
  and the source's configuration read and trash rule are
  `read_config_snapshot_awaiting` and `trash_location_is_ignored_awaiting`.
  The synchronous faces stay (`check_clean`,
  `scan_for_untracked_file`,
  `untracked_file`,
  `read_configuration_timed`,
  `trash_location_ignored_timed`,
  `the_checks_of_the_source_in_turn`),
  because `worktree_remove.rs` and `worktree_ladder_run.rs` call them from plain threads,
  and each enters the runtime once (`run_on_runtime`).
  The awaiting faces beside them are `check_clean_awaiting` and
  `the_checks_of_the_source`.
- What each wait is bounded by did not change:
  every Git child is the step its class gives it
  (`terms_for_request`,
  the quiet window and the ceiling),
  and the proof of the tracked files was unbounded and still is,
  now as a job of the pool.
  What did change is one diagnostic:
  the three steps `check_tracked` enters with `add_timing::begin_step`
  (`readIndexAndTree`,
  `indexAgainstTree`,
  `walkSource`)
  are a thread-local recorder,
  so a job of the pool does not reach the add's timing record and their time lands in
  `checkSource`'s own share.
  The `untrackedScan`,
  `readConfig` and `trashIgnored` work names still arrive,
  because that ledger is the process's.
- Two rules the sites keep,
  both recorded rather than invented here:
  `run_a_job_or_in_turn` runs a refused job's work on the awaiting task under a place of the
  process's bound,
  which is the fallback `SPEC.md` already records for the package-manager repair
  (owner,
  2026-11-09),
  and each synchronous face falls back to running its checks one after another on the calling
  thread where no runtime can be built,
  as `repair_worktree` does.
- Controls:
  `worktree_clean_tests.rs`,
  `the_tracked_files_are_proven_as_a_job_of_the_processs_blocking_pool` (written first,
  failed before the move),
  `a_worktree_that_is_not_clean_is_still_refused_where_no_thread_can_start` (replaces
  `a_clean_worktree_is_clean_where_no_thread_can_start`,
  whose seam this move takes away from the site;
  the body runs in a child refused every thread) and
  `worktree_snapshot_source_tests.rs`,
  `btrfs_the_checks_of_the_source_take_a_place_of_the_processs_bound` (written first,
  failed before the move).
  Both place controls fail without the move and read:
  "the tracked files were proven without asking for a place of the process's bound on blocking
  jobs" and
  "the files of the source were proven clean without asking for a place of the process's bound
  on blocking jobs".
  Proof by removal:
  with the job planted as `run_blocking_job_within_the_budget` (so a refusal is answered instead
  of the work being run),
  the child control fails with
  "the tracked files could not be proven (the system refused a thread for a blocking step ...)".
- Counted before the move,
  `worktree_clean::tests` and `worktree_snapshot_source::tests` together:
  24 run,
  22 passed,
  2 failed.
  After it:
  16 of 16 and 8 of 8,
  and 163 of 163 over the clean,
  source,
  trash,
  ladder and remove filters;
  91 of 91 over the same set with `worktree_ladder`;
  159 of 159 over convert,
  clone,
  remove and checkout workers.
- `native:clippy` clean;
  `monochromatic-lint` over the nine files this step touches reports nothing,
  after the two files it pushed over 300 code lines were split
  (`worktree_clean_untracked_scan.rs`,
  `worktree_trash_ignore_rule.rs`)
  and one doc line was broken.
  The package-wide lint is not green on this tree and was not before this step:
  `rust/max-lines` and `markdown/semantic-line-breaks` findings stand in
  `push/`,
  `testing/` and `worktree_copying/`,
  none of them touched here.
- Not done:
  `worktree_copying/` (another agent holds it),
  the container gate (`native:test:container`),
  `native:perf` and the mutation campaigns (paused by the owner),
  and Win and mac were left alone.

### Step T12, the worktree copy's threads become waits of the runtime, 2026-11-09T14:50:45-05:00

- Branch and worktree:
  `refactor/git-policy-cli-worktree-copy-threads-on-the-runtime` in
  `.claude/worktrees/pi-tokio-worktree-copy-thread-sites`,
  from main at `0c4282dc2c`,
  with main merged as `9d559d3571`.
  Not merged into main.
  Commits:
  `694b0f7ddf` (the side-by-side runner on the runtime),
  `5910e490a7` (a removed stage's subtrees),
  `be158c8607` (a staged copy's slices),
  `ec0d6d73e9` (an installation's slices),
  `8bc979be0d` (the final check's source walk),
  and `da9bba44dc` (the snapshot's trash beside the repair).
- New foundation module
  `foundation/side_by_side_on_the_runtime.rs`,
  the mirror of `side_by_side_threads.rs`:
  `run_all_side_by_side_on_the_runtime` runs every item as one job of the blocking pool
  (`run_a_job_or_in_turn`) and answers in the order of the states,
  and `run_beside_the_caller_on_the_runtime` runs one job beside the calling task's own
  work.
  Both take owned state,
  so `S: Clone + Send + 'static`,
  and both are awaited faces a task of the runtime can use.
- The five sites the brief named now run on the runtime,
  each keeping the synchronous face its callers still call,
  which enters the runtime once:
  - `worktree_copying/worktree_copy_compare.rs`:
    the source's manifest walk is a job of the pool and the stage's walk runs beside it on
    the calling task,
    as the thread and the caller did.
  - `worktree_copying/worktree_copy_stage_leaf_copying.rs`:
    each slice of leaves is one job of the pool,
    which owns its share of the manifest and answers with the stamps of its own
    positions instead of writing into one shared list.
    `copy_leaves` returns the stamps of its slice,
    and the caller places them in slice order,
    so the fan-out writes nothing shared but the early-stop flag
    (`Arc<AtomicBool>`).
  - `worktree_copying/worktree_copy_install.rs`:
    each slice of a batch's leaves is one job of the pool,
    named by positions in the batch it shares;
    the batch travels as an `Arc<[WorktreeCopyEntry]>` made once per batch.
  - `worktree_copying/worktree_copy_journal_removal.rs`:
    the workers that share the subtrees below the split depth are jobs of the pool;
    the cursor and the subtrees travel as shares.
  - `worktree_copying/worktree_copy_after_lock.rs`:
    the disposal of a snapshot's trash and the package repair are two branches of one task
    (`tokio::join!`),
    each of which is a chain of jobs of the pool:
    the disposal on the new awaited faces of the trash,
    the repair on `repair_worktree_awaiting`.
- The trash's removal gained an awaited face rather than a second removal:
  `worktree_trash_dispose.rs` now holds one `prepare_trash_removal`
  (the lookups,
  the association check and the descent),
  one `EntryRemoval` per face,
  and one `settle_trash_removal`
  (the strangers' check,
  the directory's own removal and the parent's),
  with `delete_trash` calling them here on the caller's thread and
  `delete_trash_awaiting` calling them as jobs of the pool.
  The service's trash handler still calls the synchronous face,
  because its own wait is not on the runtime yet (item 2).
- What each site's wait is bounded by is unchanged:
  the copy's slice count is the host's budget up to eight
  (`copy_worker_count`),
  the install's is the same ceiling from `PARALLEL_BREAK_EVEN_ENTRIES` leaves,
  the removal's is that ceiling with one worker per eight subtrees below the split depth,
  the final check's two walks end with their trees,
  the trash's entries end with the manifest's list,
  and every job of all of them waits for a place of the process's one bound on blocking
  jobs before it starts.
- The pattern for a refused thread is the one the earlier sites used:
  `run_a_job_or_in_turn` does that item's work on the awaiting task where the pool answers
  `NoThreadToStart`,
  because `spawn_blocking` queues the job and then panics
  (`tokio-1.54.1/src/runtime/blocking/pool.rs`,
  `spawn_task`),
  and the place of the bound is taken again for the work done in turn.
  The existing children that are refused every thread prove it:
  `a_split_removal_where_no_thread_can_start_removes_the_whole_tree`,
  `a_parallel_stage_copy_where_no_thread_can_start_copies_in_turn`,
  `a_parallel_install_where_no_thread_can_start_is_made_in_turn`,
  `the_final_check_where_no_thread_can_start_walks_in_turn`,
  and `where_no_thread_can_start_every_item_runs_in_turn` for the new runner.
- A trash cannot be emptied in a process with no thread to start,
  and that is not new to this step:
  `remove_entries` on either face removes its entries through jobs of the pool,
  so a refused pool leaves them with their reasons and the add reports what stayed.
  The synchronous face does the same,
  which is why `nothing_to_repair` still runs the disposal here with no runtime at all,
  and why an add whose runtime cannot be built does the disposal and the repair one after
  the other on its own thread.
- Controls,
  each shown to fail before its change by putting the old file back:
  `the_source_walk_takes_a_place_of_the_processs_bound`,
  `the_copy_slices_take_places_of_the_processs_bound`,
  `the_install_slices_take_places_of_the_processs_bound`,
  `the_removal_workers_take_places_of_the_processs_bound` (all four through
  `hold_the_only_place`:
  they read no ask for the one place of the process's bound on the old code),
  and `a_disposal_beside_a_repair_runs_inside_the_processs_runtime`
  (the old disposal ran on a scoped thread outside the runtime:
  a probe in a build of the old code printed
  `PROBE disposal inside_the_runtime=false thread=ThreadId(4)` against
  `PROBE caller thread=ThreadId(2)`).
  Replaced controls:
  `a_disposal_beside_a_repair_runs_on_a_thread_of_its_own` asserted the thread this step
  removes,
  so it became the runtime control,
  and the lifecycle control
  `a_snapshot_trash_is_emptied_before_the_add_ends_and_a_failure_of_it_is_not_the_adds`
  now drives the awaiting body with the shipped disposal,
  because the test-only `Disposal` stand-in and the `ScopeBody` it filled are gone.
- Checks:
  `mise run //package/git-policy/cli:native:test:host -- worktree_copying:: foundation::side_by_side_on_the_runtime worktree::worktree_trash`:
  370 tests run,
  370 passed,
  on the tree with main merged at `9d559d3571`.
  `native:clippy` is clean on this branch's files;
  the task itself answers one error on main,
  `clippy::manual_map` at `foundation/lazy_runtime.rs` line 434,
  which came in with `fa57b9a574` before this branch merged it,
  so Clippy was run with `-A clippy::manual_map` to show no other finding.
  `monochromatic-lint` on the seventeen files this step touched:
  no finding.
  Not run:
  the container gate,
  `native:perf`,
  and the mutation campaigns (paused by the owner).
- What is left in `worktree_copying/`:
  no production scoped thread.
  The one left in the folder is `worktree_copy_recovery_tests.rs`'s control that runs two
  adds at once,
  which is a control of the concurrent worktree copy and not a site of the copy path.
- What is left beside it:
  `worktree/worktree_snapshot_source.rs` (two `run_beside_caller` sites) and
  `worktree/worktree_clean.rs` (one),
  which are not part of the copy path this step was given;
  then item 2 (the landing's handlers and `landing/service_round.rs`),
  then T13's guards.
- Measured,
  and not measured:
  no timing was taken for this step,
  because every face it changes is the copy path of `git worktree add`,
  and the harness for it is `native:perf`,
  which this step did not run.
  The one cost the step adds is one copy of the manifest per call:
  `Arc::from(entries)` in the copy stage and in the install's leaves,
  and the small selected and excluded root lists of the final check,
  which is what a job of the pool that owns what it reads needs.
  A stage of 22,717 entries pays that once,
  against the 25 to 100 ms its slices already take in the measurements beside
  `PARALLEL_COPY_MIN_ENTRIES`.
### Step T12, the landing round awaits its push chain, and the merge's second pass is what stops it, 2026-11-09T15:46:16-05:00

- Branch and worktree:
  `refactor/git-policy-cli-landing-round-awaits-its-push-chain` in
  `.claude/worktrees/pi-tokio-awaited-push-chain`,
  from main at `d713744706`,
  with main merged in at `0ef371ee15` (it had moved to `5045e9c01c`).
  Not merged into main.
- The goal:
  the landing round's move onto `run_ordered_awaiting_on_the_runtime`,
  which needs the four things the section
  "Step T12,
  the landing round:
  what blocks it and what now pins it" lists.
  This run did the whole of the second (the awaited maintenance) and the
  reconcilable part of the first (the awaited push chain),
  and measured where the first stops being reconcilable.
- Done,
  one commit each,
  every one compiling and passing:
  - `153b270b7a`,
    the awaited maintenance.
    `run_auto_maintenance_with_awaiting` is
    `run_auto_maintenance_with` with `run_git_awaiting` for its two Git steps,
    and the decision,
    the journal lines and the pre-2.47 retry are one code
    (`config_from_snapshot`,
    `planned_arguments`,
    `maintenance_config_of`,
    `ended`,
    `reported_fallback`).
    Controls:
    `the_wait_for_maintenance_lets_other_tasks_run` and
    `the_awaited_face_answers_what_the_synchronous_face_answers`,
    over the file's 16 tests.
  - `c838d5190a`,
    the awaited query.
    `run_query_awaiting` and `git_output_awaiting`
    (`push/auto_push_single_flight.rs`) ask the same question with the wait on
    the runtime,
    and `output_text` holds the one copy of how an answer is read.
  - `780f8f895e`,
    the awaited push destination.
    `read_push_settings_awaiting`,
    `named_push_awaiting` and
    `read_destination_awaiting` (`push/auto_push_destination.rs`),
    with `push_settings_of`,
    `destination_of`,
    `mirror_skip`,
    `named_by_mode`,
    `refspec_query` and `named_by_refspecs_of` shared with the synchronous
    faces.
    Control:
    `the_awaited_reads_answer_what_the_synchronous_reads_answer` runs both faces
    over six configurations of one fixture repository.
  - `6ab80e14b2`,
    the awaited Git LFS step.
    `lfs_upload_plan_awaiting.rs` (a sibling file,
    because both faces of the
    plan together pass the package's length limit) holds
    `plan_upload_awaiting`;
    `read_settings_awaiting` and `current_branch_awaiting` are in
    `lfs_upload_settings.rs`;
    `run_upload_bounded_awaiting` is in `lfs_upload.rs`;
    `run_push_awaiting` and `PushQueryAwaiting` are in `lfs_bounded_push.rs`.
    `GitQueryAwaiting` is the synchronous `GitQuery` behind a boxed future,
    because a trait with an `async` method cannot be a `dyn` value.
    Controls:
    `the_awaited_face_of_the_plan_asks_and_answers_as_the_synchronous_face_does`
    (twelve shapes of a push,
    two environments,
    and a failing question) and
    `the_awaited_upload_answers_what_the_synchronous_upload_answers`.
  - `021afd91dc`,
    the filesystem reads of those faces put on the blocking pool.
    The awaited upload looked `git-lfs` up on the caller's `PATH` and read the
    worktree's layout on the task,
    and the awaited push's pre-check scanned that
    `PATH` the same way,
    so a filesystem that answers no lookup would have held
    every other directory of the round.
    Control:
    `the_awaited_upload_asks_for_a_place_of_the_processs_bound_before_it_reads_the_path`,
    which holds the process's only place (`testing/parallel_test_support.rs`) and
    failed with
    "the upload read the caller's PATH without asking for a place of the
    process's bound" when the lookup went back onto the task.
- Measured,
  and this is what remains:
  the awaited push chain cannot be finished without the merge's second pass,
  and the second pass is a folder of its own.
  - `push_once` (`push/auto_push.rs:262`) ends a rejected push with
    `reconcile_divergent_push`,
    and `integrate_and_push`
    (`push/auto_push_reconcile.rs:187`) publishes a merge it computes with
    `commit_merge` (`push/auto_push_reconcile_merge_commit.rs:150`),
    whose
    conflicted merge is resolved by `resolve_tree`
    (`merge/merge_second_pass_tree.rs:359`).
  - That path runs Git through one trait:
    `GitRunner::run`
    (`merge/merge_second_pass_git_runner.rs:35`),
    a synchronous method,
    and the reconcile's `PushRunner`
    (`push/auto_push_reconcile_merge_commit.rs:91`) implements it with
    `run_bounded_git`.
    Seven functions take `&dyn GitRunner`
    (`second_pass`,
    `merge_attributes`,
    `read_blobs`,
    `write_blob`,
    `tree_with_files`,
    `resolve_tree` and `install`),
    and the folder that holds them is 9,381 lines of source,
    14,916 with its controls (`merge/`),
    of which `merge_second_pass_worktree.rs` also serves a person's own
    `git merge`.
  - So the reconcile is not "about a dozen functions in six files":
    awaiting it means the second pass's runner hands back a future,
    and
    everything that takes `&dyn GitRunner` becomes a future too.
    Two ways,
    and the choice is the owner's because it decides how the typed
    `git merge` path keeps its synchronous face:
    - A.
      Await the second pass:
      `GitRunner::run` hands back a boxed future;
      `second_pass`,
      `merge_attributes`,
      `read_blobs`,
      `write_blob`,
      `tree_with_files`,
      `install` and `resolve_tree` become `async`;
      the forwarded `git merge` path keeps one synchronous face that enters the
      runtime once (`run_on_runtime`).
      No duplication,
      but it rewrites `merge/`'s seam and every stub runner in
      its controls.
    - B.
      Duplicate the second pass as awaited twins:
      no comment about the typed path,
      and two bodies that must not diverge,
      in a folder whose conflict resolution is the one thing here a person's
      data depends on.
  - `778fc95c23`,
    the reconcile's own questions awaited:
    `is_ancestor_awaiting`,
    `sweep_private_fetch_refs_awaiting` and
    `fetch_remote_tip_awaiting`
    (`push/auto_push_reconcile_basics_and_limits.rs`),
    and `common_dir_of_awaiting` and `signs_commits_awaiting`
    (`push/auto_push_reconcile_merge_commit.rs`),
    with `ancestry_of` and `refs_to_sweep` as the one copy each of how an
    ancestry exit status is read and which private fetch refs may be swept.
    These need no decision about the second pass.
    Control:
    `the_awaited_questions_of_the_reconcile_answer_what_the_synchronous_ones_answer`
    (`push::auto_push_reconcile`,
    79 of 79).
  - What the awaited chain still lacks once the second pass is settled,
    in order:
    `reconcile_divergent_push_awaiting` and `integrate_and_push_awaiting` with a
    `MergePusherAwaiting` (`push/auto_push_reconcile.rs`),
    `commit_merge_awaiting` (`push/auto_push_reconcile_merge_commit.rs`),
    `advance_local_branch_awaiting` and its helpers
    (`push/auto_push_reconcile_branch_advance.rs`),
    then `push_once_awaiting` (`push/auto_push.rs`) and the two handlers
    `WorkHandlers::push_awaiting` and `WorkHandlers::maintenance_awaiting`
    (`landing/service_work.rs`),
    which must be object-safe methods handing back a boxed future,
    with the
    synchronous ones as their default for the doubles.
  - The push's own synchronous reads are not all covered yet either:
    `GitHandlers::push_git` (`landing/service_work.rs:380`) calls
    `read_config_snapshot`,
    a synchronous filesystem read,
    and must call
    `read_config_snapshot_awaiting` inside a job of the pool when the push runs
    on a task;
    and `branch_exists` and `branch_is_gone_words` (`:449`,
    `:481`) read the
    repository with the synchronous `run_query`.
  - The third and fourth things the round needs are untouched:
    `ClaimRenewal`'s `Drop` still waits for its task
    (`landing/service_claim_renewal_running.rs:275`),
    and `drain_pending` (`landing/service_work_drain_pending_claims.rs`) still
    interleaves `claim_all_of`,
    `settle_all` and the push-failure records with
    the handler calls on one thread.
- Checks:
  - The file selections this run touched,
    all green:
    `landing::maintenance_run` 16 of 16,
    `push::auto_push_single_flight` 5 of 5,
    `push::auto_push_destination` 7 of 7,
    `lfs::` 71 of 71 (and `lfs::lfs_upload` 13 of 13 after the pool change),
    `push::auto_push` 165 of 165.
  - `native:clippy` clean at the last commit of this run's own work
    (`021afd91dc`).
    and `//package/git-policy/cli:lint` 0 errors and 0 warnings on the folders
    this run touched
    (`landing/maintenance_run.rs`,
    `landing/maintenance_run_tests.rs`,
    `push/auto_push_single_flight.rs`,
    `push/auto_push_single_flight_tests.rs`,
    `push/auto_push_destination.rs`,
    `push/auto_push_destination_tests.rs`,
    `lfs/`),
    after `format:monochromatic-lint` wrote the breaks it asked for.
    At the merged tip `native:clippy` reports one finding this run does not
    touch:
    `foundation/lazy_runtime.rs:434` is "manual implementation of
    `Option::map`",
    and that file's last change is main's `6e27b3add3`
    (`style(git-policy-cli): lint at zero on main`).
    It was left alone rather than fixed here,
    because another branch owns that file.
    The package-wide `//package/git-policy/cli:lint` is not clean on this base
    and was not made clean here:
    `worktree_copying/worktree_copy_stage_claim_reaping.rs` is over the
    package's length limit,
    and `foundation/lazy_runtime.rs` holds one closure finding,
    neither of which this run touches.
  - The whole `native:test:host` suite,
    once before main was merged
    (`CLI_GIT_TEST_ROOT=/var/tmp/cg/cli-git/fixtures/root`,
    the run's own control group at `TasksMax=4000`):
    5707 run,
    5703 passed (one slow),
    4 failed,
    14 skipped.
    Three of the four were main's,
    not this branch's,
    and the merge fixed them
    (`c07ee38a08` names the shell shim `health/lfs_shims.rs` writes,
    which the
    no-self-invocation guard flagged at `d713744706`).
    The fourth,
    `service::service_review::tests::a_silent_gateway_ends_the_review_when_it_has_been_silent_for_the_quiet_window`,
    took 22.4 s and is the recorded shape of a time window a loaded host
    reaches.
    At the merged tip the same suite was run again:
    5816 run,
    5815 passed (one slow),
    1 failed,
    14 skipped.
    The one failure is
    `source_guard::service_lock_inventory_tests::a_lock_stands_only_in_a_file_on_the_inventory`,
    which reports `commit/commit_prepare_part_durations.rs` as a lock outside
    the inventory;
    that file's last change is main's `e6a69463ed`,
    so it came with the merge and not with this run.
    `source_guard` is 46 of 47 for that reason,
    and every other selection of the merged tip passed.
  - Not run:
    `native:test:binary:host`,
    the container gate
    (`native:test:container`),
    `native:perf` and any mutation campaign
    (paused by the owner).
    None of the files this run changes runs in a commit or in a forwarded
    command,
    so `native:perf` was not run to say what it would cost.

## A refused thread ends with the sentence that names the limit, 2026-11-09T17:30:00-05:00

- What a command does when the system refuses a thread,
  on every path a commit,
  merge,
  push or worktree command takes,
  had two halves in this tree:
  the work runs on a thread the command already has
  (`foundation/scoped_start.rs::start_or_run_with`,
  which the worktree copy,
  the worktree clean check,
  the repair clone,
  the ordered runner and the side-by-side runner all use),
  or the command ends with a sentence.
  The sentence was missing at five places,
  and one of them called the refusal a defect of this program:
  `push/bounded_query_input.rs` (the write half of a push query),
  `push/child_watch.rs` (the waiter of a step left running),
  `markdown/markdown_linter_guard.rs` (the linked linter of a commit),
  `service/service_server.rs` (the service's after-commit work thread,
  which ended the service),
  and
  `standards_review/standards_review_run.rs`,
  which answered a refused thread as `Contained::Panicked`,
  so `service/service_review.rs` told the person
  "the review of <oid> stopped with an internal error".
- `foundation/scoped_start.rs::refused_thread_sentence` is the one sentence now:
  "no thread could be started to <what>:
  the system refused the thread (<the system's error>).
  Hint:
  check the process limit (`ulimit -u`),
  the address-space limit (`ulimit -v`) and the control group's `pids.max`,
  free some,
  and run the command again".
- `Contained` gained `NoThread(String)`,
  and gave up `Copy` with it because the sentence is owned;
  nothing in the tree copied a `Contained`.
- The controls,
  each refused a thread for real in a child whose `RLIMIT_NPROC` is one
  (`testing/no_threads_child_test_support.rs`):
  `push::bounded_query::tests::where_no_thread_can_start_the_input_writer_names_the_limit`,
  `markdown::markdown_linter_guard::tests::where_no_thread_can_start_the_guard_names_the_limit`,
  `service::service_review::tests::where_no_thread_can_start_the_review_names_the_limit`,
  and,
  at the binary level,
  `fault_limits::a_service_whose_work_thread_is_refused_says_which_limit_refused_it`
  (the service under `RLIMIT_NPROC` one,
  and its own log line is what the control reads).
- `push::child_watch::tests::a_step_left_running_whose_waiter_is_refused_names_the_limit`
  arms the refusal instead of making one,
  and has to:
  the step is a process of the very process whose thread start is refused,
  so `RLIMIT_NPROC` refuses the fork too;
  a limit lowered once the child exists does not arrive in time either,
  because the waiter's start follows the child's within a millisecond while the step's number is visible only
  after 30.4 ms (measured on 2026-11-09).
- Measured task counts,
  the gate's own settings
  (`--cpus 2 --memory 2g`,
  nextest's concurrency of two test binaries and two tests each,
  the image built from `776281f4ee`):
  a container running `--lib` and `--test native_binary` peaks at 192 tasks
  (`/sys/fs/cgroup/pids.peak` inside it,
  against `pids.max` 8192),
  and a person's commit through the wrapper peaks at 14 tasks in the scope that held it together with
  the test harness that drove it,
  its fixture service and Git.
  The container run failed the same four tests the 2026-11-09T16:14 gate failed
  (`wait_notice` three,
  `word_merge` one) on a tree that does not hold this branch's changes,
  so those failures are not this cap and not these changes.
- `--pids-limit 512` stays what it is:
  192 measured leaves it 2.7 times the room,
  and the thread refusals in the 16:14 gate log came from the tests that set `RLIMIT_NPROC`,
  `RLIMIT_AS` and `RLIMIT_NOFILE` on the wrapper and on the service themselves
  (`binary/binary_fault_limits_tests.rs`),
  not from the container's cap.
  Raising the cap would hide none of them and fix none.
- What no in-process guard can answer:
  a thread that starts and then cannot map its signal stack ends the process with `rtabort!`
  (`library/std/src/sys/pal/unix/stack_overflow.rs`,
  outside any unwind),
  so the runtime plan keeps `runtime_plan.rs`'s `ADDRESS_SPACE_RESERVE_BYTES` back for it and
  `a_memory_limit_close_to_exhaustion_on_the_service_...` accepts the band where nothing can start.
### Step T12, the second pass awaits its own Git, and the reconcile's chain stops at the branch move, 2026-11-09T17:35:00-05:00

- Branch and worktree:
  `refactor/git-policy-cli-merge-second-pass-awaited` in
  `.claude/worktrees/pi-tokio-merge-second-pass-awaited`,
  from `c0bb93d036` (the merge of the previous round).
  Not merged into main.
- What this run carried out:
  option A of the previous section,
  which the coordinator picked with the owner's veto open
  (`doc/handover/questions-for-the-owner.md`,
  "the merge's second pass becomes awaitable as one body").
  A start of the second pass hands back a boxed future,
  and every caller that takes a `GitRunner` is awaited or enters the runtime once.
  One body,
  no twin:
  the folder a person's conflict resolution depends on does not diverge from itself.
- Done,
  one commit,
  `403254deeb`,
  compiling and passing:
  - `merge/merge_second_pass_git_runner.rs`:
    `GitRunner::run` returns
    `Pin<Box<dyn Future<Output = Result<GitReply, String>> + Send + '_>>`,
    as the LFS step's `GitQueryAwaiting` does,
    because a trait with an `async` method cannot be a `dyn` value;
    the shipped `ContextRunner` awaits `run_git_awaiting`,
    so a start's pipes are registered on the runtime the caller already runs on.
  - `merge/merge_second_pass.rs`:
    `second_pass`,
    `merge_attributes`,
    `read_blobs`,
    `write_blob` and `tree_with_files` are `async`.
  - `merge/merge_second_pass_tree.rs`:
    `resolve_tree`,
    `merge_attribute_source`,
    `attribute_choice`,
    `configured_attribute_tree` and `attribute_tree_variables` are `async`,
    because the attribute tree is a question put to the same runner.
    `resolve_tree_on_the_runtime` and `attribute_tree_variables_on_the_runtime`
    are the faces the thread-side callers use:
    `landing/service_replay.rs`,
    `shadow/replay_merge.rs`,
    and the merge tree of `merge/merge_isolation_merge_tree.rs`.
  - `merge/merge_second_pass_worktree.rs`:
    `install` and `second_pass_parts` are `async`,
    and `resolve_after` stays synchronous and enters the runtime once for the pass,
    because the finish after it
    (`merge_second_pass_finish.rs`)
    still starts Git synchronously and is a folder of its own.
  - `merge/merge_isolation_apply.rs`:
    `merge_tree` keeps its signature and enters the runtime once,
    `merge_tree_awaiting` is the asynchronous face.
  - `push/auto_push_reconcile_merge_commit.rs`:
    `PushRunner` answers through `run_bounded_git_awaiting`,
    `commit_merge` enters the runtime once,
    and `commit_merge_awaiting` is the face the reconcile's loop will use.
    The two single synchronous questions it no longer asks
    (`common_dir_of`,
    `signs_commits`) are `#[cfg(test)]`,
    for the control that compares the two faces.
  - Two sibling files,
    because the folder crossed the package's 300-code-line limit:
    `merge/merge_isolation_merge_tree.rs` (the merge computed from objects and the pass over its
    conflicts) and
    `merge/merge_second_pass_worktree_install.rs` (the staging and the write of the files a
    forwarded pass merged).
    Both keep their names through a `pub use` in the file they left.
- Controls:
  - `the_second_pass_answers_from_a_task_of_the_runtime`
    (`merge/merge_second_pass_tree_tests.rs`)
    runs `second_pass` and `resolve_tree` inside `run_on_runtime` over a real conflicted
    repository and requires the merged tree.
    Planted,
    by putting the synchronous `run_git` back where `ContextRunner` awaits,
    it fails with
    "git check-attr -z --stdin merge could not start:
    the runtime that runs this command's waits
    was asked for from inside itself,
    where waiting for it would never end;
    nothing was run".
    Restored.
  - `the_synchronous_start_is_refused_from_a_task_of_the_runtime`
    is the positive control of that one:
    the two faces differ in exactly the way the awaited one is written for.
  - Every existing control passes unchanged,
    with its stub runner's start now handing back a boxed future and its call awaited on the
    runtime;
    the two counting stubs hold a `Mutex` and an `AtomicUsize` instead of a `RefCell` and a `Cell`,
    because a start's future is `Send`.
  - The person's own path is the built executable:
    `binary_word_merge`,
    `binary_word_merge_finish`,
    `binary_merge_isolation`,
    `binary_merge_never_fails` and
    `binary_commit_during_merge`,
    87 controls,
    all passing.
- Measured,
  and this is where the awaited push chain stops:
  `advance_local_branch`
  (`push/auto_push_reconcile_branch_advance.rs`)
  moves the branch through `fast_forward_recorded`,
  whose refusal path settles the intent record with `settle_failed_merge`
  (`merge/merge_isolation_recovery.rs:253`).
  That function,
  and the folder it settles through,
  read Git at 19 synchronous start sites
  (`run_git`,
  `run_git_checked`) across six files
  of 2,161 lines
  (`merge_isolation_recovery.rs`,
  `merge_isolation_recovery_steps.rs`,
  `merge_isolation_recovery_reads.rs`,
  `merge_isolation_recovery_cleanup.rs`,
  `merge_isolation_recovery_complete.rs`,
  `merge_isolation_moved.rs`,
  `merge_isolation_git_reads.rs`).
  A synchronous start is refused inside the runtime,
  and a job of the blocking pool is inside the runtime too,
  so there is no bridge to it:
  `advance_local_branch_awaiting` needs that folder converted first,
  leaf first,
  as `merge/` was here.
  Leaving the record for the next wrapper command to settle instead was rejected:
  it changes what a person sees after a refused fast-forward.
- What the chain still lacks,
  in the order the dependency allows:
  the recovery folder's awaited Git seam,
  then `advance_local_branch_awaiting` and its helpers
  (`push/auto_push_reconcile_branch_advance.rs`),
  then `integrate_and_push_awaiting` with `MergePusherAwaiting` and
  `reconcile_divergent_push_awaiting` (`push/auto_push_reconcile.rs`),
  then `push_once_awaiting` (`push/auto_push.rs`),
  then the two handlers `WorkHandlers::push_awaiting` and
  `WorkHandlers::maintenance_awaiting` (`landing/service_work.rs`).
  `commit_merge_awaiting` is done.
  The push's own synchronous reads that the handlers will need are unchanged:
  `GitHandlers::push_git` (`landing/service_work.rs:380`) calls `read_config_snapshot`,
  and `branch_exists` and `branch_is_gone_words` read with the synchronous `run_query`.
- Checks:
  - `merge`,
    `auto_push_reconcile` and `auto_push` in one selection:
    347 of 347 passed,
    run twice,
    once before the two sibling files were split out and once after.
  - `native:clippy` clean on every target,
    and `//package/git-policy/cli:lint` 0 errors and 0 warnings on the folders this run touched,
    after `format:monochromatic-lint` wrote the breaks it asked for.
  - `native:test:binary:host --no-run` builds the binary test target.
  - The binary selection above:
    87 of 87.
    One control of the pair that watches the pass's own `git check-attr` from `/proc`
    (`word_merge::an_edit_made_while_the_pass_merges_is_not_replaced` and
    `word_merge::a_same_length_resolution_with_a_restored_time_is_not_replaced`)
    fails under host load at its 15-second window,
    and it does so on the base commit too:
    at `c0bb93d036` the first of the two failed at 16.06 s with the same assertion,
    and here the second failed at 16.29 s.
    Each passes alone,
    and both pass in the 87-control run,
    so the flake is in the control's `/proc` polling and not in the pass;
    it is not fixed here,
    because a proper fix is a shim Git that writes a marker instead of a window
    (fixed 2026-11-09,
    see "The two controls of the second pass wait on the pass itself" below).
- Not run:
  the whole `native:test:host` suite,
  the container gate,
  `native:perf` (nothing of this run is in a commit or a forwarded command's path)
  and any mutation campaign (paused by the owner).

## The two controls of the second pass wait on the pass itself, 2026-11-09T19:04:27-05:00

Branch `test/git-policy-cli-word-merge-controls-wait-on-a-marker`,
from main at `283d7d9053`.
Three commits:
`76b18e0f3b` (the stand-in Git and the two controls),
`3e9133dbd5` (a warning in the stand-in's pipe opener) and
`47f4caf239` (two doc lines the linter broke).

### What replaced the window

The section "Step T12,
the second pass awaits its own Git,
and the reconcile's chain stops at the branch move"
records the failure and names the fix:
each of the pair walked `/proc` for the pass's own `git check-attr` child inside a fifteen-second window,
and under host load the pass had not reached the query before the window closed.
The window is gone.
`binary_word_merge_pass_gate.rs` puts a stand-in `git` on the run's `PATH`:
the wrapper's link directory,
then the stand-in,
and nothing else,
because real Git at a conventional location outranks every other candidate the wrapper considers.
The stand-in passes every start to real Git by absolute path,
except the pass's own `git check-attr -z --stdin merge`,
which it holds:
it writes `reached.marker` and the word `reached` on one pipe,
and it reads the control's word on a second pipe before it goes on.
The commit policy's own `check-attr` names its paths as arguments,
so the two flags tell the pass's query apart from every other start,
and the marker file makes the hold happen once per run.
The control waits on that pipe and releases the pass after its edit,
so neither side is raced against a clock.
Two details make the wait clockless:
a pipe held at both ends waits for nothing to be opened and never reads the end of its input,
and a control still waiting when the command ends is woken by the command's own end,
so a pass that never reaches the query fails the control with its reason instead of hanging it.
The supervisor's idle rule is suspended for the hold itself,
which is the test's own controlled wait.
The controls also conflict four files instead of forty:
the gate is what puts the edit inside the pass,
so the count no longer buys a window.
Every start the stand-in takes is appended to `starts.log`,
so the gate's own control can say the stand-in was the run's Git instead of assuming it,
and `reached.marker` can be read once the run ended.

### What each control still proves, and the plants that showed it

- `word_merge::an_edit_made_while_the_pass_merges_is_not_replaced`:
the edit's bytes survive and the line names the path the pass left.
Planted by reading the worktree baseline at install time instead of when the conflicts were read
(`stillness` in `merge_second_pass_worktree_unchanged.rs`),
as it was before the guard:
the control failed at 4.160 s with
"assertion `left == right` failed:
the other command's bytes survive",
left holding the merged file's bytes and right the resolution's.
Restored,
and it passes.
- `word_merge::a_same_length_resolution_with_a_restored_time_is_not_replaced`:
a resolution of the same length whose modification time was put back survives.
Planted by comparing lengths instead of bytes in `stillness`,
which is the defect the bytes replaced:
the control failed at 10.739 s with "the other command's bytes survive" as its words,
and the 'R'-filled file gone.
Restored,
and it passes.
- `word_merge_pass_gate_guard::a_command_that_never_asks_for_the_attribute_wakes_the_control`:
a forwarded `rev-parse --git-dir` starts the stand-in Git and never asks for the merge attribute,
so the control must be woken with `false` by the command's own end,
and the start record must name the command,
while `reached.marker` stays absent.
Planted by leaving that word unsaid (`held_run` in `binary_word_merge_pass_gate.rs`):
the control waited,
and nextest's own bound ended the test
(`TIMEOUT` at 300.007 s),
which is the hang the word exists to prevent.
Restored,
and it passes in 1.004 s.

### What this change ran

- `//package/git-policy/cli:native:test:host -- word_merge merge_second_pass`:
57 of 57 passed in 1.987 s,
and 57 of 57 again in 2.349 s at the final commit.
- `//package/git-policy/cli:native:test:binary:host -- word_merge merge_second_pass`:
39 of 39 passed in 23.251 s,
with the pair at 1.186 s and 1.224 s
under a load average of 148 on 8 CPUs,
and again 39 of 39 in 30.851 s at a load average of 68 with the pair at 1.297 s and 1.313 s.
At the final commit the same selection is 40 controls
(the gate's own control joined them)
and 40 of 40 passed in 138.881 s at a load average of 57,
six of them slow,
with the pair at 1.004 s and 1.020 s:
the whole selection follows the host,
the pair no longer does.
  One run of the same selection failed,
  and it is recorded here because it is not this pair:
  with the scope's memory held to 8 GiB instead of the 16 GiB of the passing runs,
  the same 39 controls took 156.260 s at a load average of 148,
  and `word_merge_finish::a_cherry_pick_range_is_resolved_and_continued`
  was ended by the harness supervisor's five-second idle rule,
  its tree being `bin/git cherry-pick main..topic` asleep in `unix_stream_read_generic`
  while the commit service answered a landing in 5,140 ms.
  Both of this pair passed in that run,
  at 1.2 s each.
  The idle rule's own limit is stated in `binary_supervisor.rs`:
  it cannot tell an endless wait from a finite one on something outside the tree,
  and the service is such a thing.
- `//package/git-policy/cli:native:clippy` clean on every target.
- `//package/git-policy/cli:lint:monochromatic-lint`:
nothing in the files this branch touched.
The run still fails on eleven errors and eight warnings in files this branch does not touch
(`landing/service_commit.rs` and `testing/test_support.rs` over 300 code lines,
the rest `markdown/semantic-line-breaks` and `rust/avoidable-closure`),
which stand at `283d7d9053` too.
- Not run:
the whole `native:test:host` suite,
the container gate,
`native:perf` and any mutation campaign (paused by the owner).

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
- The in-process `mallopt(M_ARENA_MAX, 1)` was measured in `probe-unix` only (28 workers:
  +57,936 KiB,
  the same as the
  environment variable);
  it was not measured in the wrapper,
  and allocation contention was not measured.
- `Runtime::block_on` with a `LocalSet` on the calling thread of a multi-thread runtime is read from the documentation
  of `Runtime::block_on` and `LocalSet`,
  and was not run.
- The review file and the probe sources are scratch;
  the probe patch of the wrapper is kept as
  `${HOME}/temp/agent/full-tokio/probe.patch`.
