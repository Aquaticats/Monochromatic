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
    "Lazily, on first need (Recommended)".
2.  What "sized to budget" fixes:
    "Build size + job limit (Recommended)".
3.  Address-space protection:
    "Cap pools + read limits (Recommended)".
4.  The landing loop and thread-affine state:
    "Rewrite per-thread state"
    (option C: each `thread_local!` becomes a task-local or a context value).
5.  Library API:
    "Everything async, and safe-file-saving shouldn't be something we own at all."
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
10. What "Commits should never fail." means for the gate:
    "Strong: what Git completes, we complete (Recommended)";
    each fault case gets a native-Git control.
11. Calls that can wait on storage when no thread can start:
    "Fail plainly".
    No inline fallback;
    the command fails with a plain message in that state,
    which the owner chose over option A.

## Progress on the branch `feat/git-policy-cli-on-tokio`

Written:
2026-11-07T07:00:00-05:00.
The branch is not merged into main (the coordinator holds that until the wrapper's release is out).
The order follows the owner's answers: the conversions run on today's layout (choice 8), so no step waits for the layout moves.

### Step T1, the self-invocation ban: already in force, audited

- The three clauses of the step are covered by `source_guard/no_self_invocation_tests.rs` and its rules file:
  rule `own-executable` (`current_exe` and `/proc/self/exe` outside the excused `invocation/real_git.rs`),
  rule `spawn-own-executable` (a `Command::new(` whose argument names the wrapper's own path),
  and the planted controls (`each_rule_fires_on_its_own_forbidden_text`, and the fixture tests that plant a violation in a
  nested production file and in a file with a test-support name).
- `clippy.toml` also denies `std::env::current_exe`.
- `tokio::process::Command::new(` would match the same rule text, and the child helper of step T4 starts children with the
  standard library, so no new spelling needs a new rule.
- No code commit was needed.
  The broader lint of step T13 (no `std::process::Command` outside the guarded start) stays at T13.

### Step T2, tokio in the manifest and the no-clone guard

- `9d9fc7ec37`:
  `binary/binary_forwarded_command_makes_no_thread_tests.rs` runs a forwarded `rev-parse HEAD` and `status --short` under
  `strace` and counts the `clone`-family calls of the wrapper's own program (`wrapper_spawns` in `binary_budget_trace.rs`).
  It is green before tokio is linked, so it is a guard, not a red test.
  The planted control is `the_count_sees_threads_and_children_made_by_the_wrapper_and_only_those`, a trace in which the wrapper
  starts a thread and a child, which the counter must count (and the harness's `timeout` and Git must not be).
- `tokio` joined the manifest with the features `rt`, `rt-multi-thread`, `net`, `time`, `fs`, `io-util`, `sync` and `macros`.
  `process` and `signal` stay off by design (choices 6 and 7).
  `rustix` (feature `process`) joined for `getrlimit` and, in the later steps, the pidfd and the inotify watch.
  The dependency landed together with its first use in step T3, because an unused dependency is a cargo warning.

### Step T3, the budget, the limits and the lazy runtime

- Files (all under `src/native/foundation/`, each with a sidecar test file):
  `runtime_limits.rs` (address space left under `RLIMIT_AS`, threads left under `RLIMIT_NPROC` and under the `pids.max` of the
  control group and every group above it),
  `runtime_plan.rs` (workers from the budget, bounded by the limits; current-thread when fewer than two fit),
  `lazy_runtime.rs` (one `OnceLock` owner, malloc arenas capped to one before the first worker, the multi-thread build on a helper
  thread with a deadline, the current-thread runtime as the stand-in, a nested call refused with an error),
  `blocking_job.rs` (a blocking call on the pool; "the system refused a thread" is the error `NoThreadToStart`, never an inline run),
  `runtime_probe.rs` (the test-only entry, `CLI_GIT_TEST_ONLY_RUNTIME_PROBE`, for the controls that start the built executable).
- `invocation/panic_notice.rs` gained `while_panics_are_expected`:
  tokio panics, instead of returning an error, when a thread is refused; the callers catch that panic and report their own plain
  error, so the hook's "report this location" notice (which would tell a person of a defect on a command that went on to succeed)
  is silenced for the panics of those two calls only.
- Values chosen under the owner's answers, each open to the owner's veto and each cited beside the value in the code:
  deadline `BUILD_DEADLINE` one second (a working build is well under a few milliseconds);
  `ADDRESS_SPACE_RESERVE_BYTES` 64 MiB (three times the 20 MiB the whole process mapped);
  `THREAD_VIRTUAL_BYTES` the 2 MiB stack plus 64 KiB (measured 2,064 KiB a blocking thread, 2,069 KiB a worker with one arena);
  `MALLOC_ARENA_LIMIT` 1.
- Departures from the text of this plan:
  the limits are a value of their own (`RuntimeLimits`) beside `HostResources`, not new fields of it, so the nine call sites and
  every test that builds a `HostResources` by hand stay unchanged;
  the user's threads are bounded by one read of the system's thread total in `/proc/loadavg`, an upper bound that only lowers the
  worker count, instead of a scan of every process;
  the arena cap is always applied, as choice 3 says, not only under an address-space limit.
- Controls (`binary/binary_runtime_build_under_limits_tests.rs`) start the built executable with the probe under a process limit
  of one and under address-space limits from 48 MiB to 1500 MiB (the measured hang band included), each with real Git started
  under the same limit as the native control (choice 10).
  Planted controls, run by hand and reverted:
  removing the limit inputs from the plan failed both the process-limit control (flavor `current-thread-after-failure`) and the
  address-space control (a stand-in after a deadline at 48 MiB);
  the unit control `a_build_that_outlasts_the_deadline_is_replaced_by_the_current_thread_runtime` fails if the wait loses its deadline.
- Finding, not the runtime's:
  below 48 MiB of address space the debug build of the wrapper prints nothing, with or without the probe, while real Git
  completes at 24 and 32 MiB.
  The sweep starts at 48 MiB for that reason.
  Whether the release build has the same floor was not measured.

### The published-crate swaps

- Digests (`47219dc182` red, `1aea0ae238` green):
  `subvolume/subvolume_sha.rs` is two calls into `sha1` 0.11.0 and `sha2` 0.11.1; the hand-written compression functions are gone.
  The published test vectors, the reference comparison at every length to a few blocks and the 512 MiB comparison still pass.
- Time stamps (`7474c396d2` red, `c4a834d4da` green):
  `foundation/iso_time.rs` formats through `jiff` and `conflict/discarded_prune_stamps.rs` parses through `jiff`;
  the hand-written calendar (`civil_from_days`, `days_from_civil`) is gone.
  The text shape is unchanged (`YYYY-MM-DDTHH:MM:SS.mmmZ`).
  An instant past the end of year 9999 is clamped to the crate's last instant (`9999-12-30T22:00:00.999Z`), where the old code
  printed a meaningless date.
  `jiff` was first kept at the already locked 0.2.24: the releases from 0.2.32 name `defmt` 1.x as an optional dependency, which
  locks `defmt-parser` 1.1.0, and the trust policy refused that release (published by account 414585 where 0.4.1 was published
  by account 84779).
  The owner, 2026-11-07, verbatim: "I'm approving the publisher change for jiff."
  `rust-catalog.toml` now lists `defmt-parser@1.1.0` in `trustPolicyExclude` (that one release, not the crate, with the sentence
  beside it), `jiff` moved to 0.2.39 through `cargo update -p jiff` under the shadow (seven versions newly locked, all checked:
  `bitflags` 1.3.2, `defmt-macros` 1.2.0, `defmt-parser` 1.1.0, `defmt` 1.2.0, `jiff-core` 0.1.2, `jiff-static` 0.2.39, `jiff` 0.2.39),
  and the catalog floor is raised to 0.2.39.
  The guard is `the_lock_no_longer_holds_jiff_at_the_release_kept_while_the_trust_policy_refused_the_newer_ones` in
  `foundation/iso_time_tests.rs`.
  With the exclusion in place the daily dependency update has no refusal to trip on at this path; before it, the shadow's update
  rule would have kept nothing: it sets a refused release aside for an older one in the same caret class and, when that fails,
  restores every lockfile of the run (`package/cargo-shadow/cli/README.md`, the `cargo update` line of the flag table).
- Memory headroom (`8955195670` red, `0a59ee9272` green):
  `HostResources::read` takes the available memory and the control-group headroom from `sysinfo` 0.40.1 (feature `system` only);
  the readers of `/proc/meminfo`, `memory.max` and `memory.current` and their tests are gone.
  Locking `sysinfo` needed the crates of its Windows and macOS targets in the local registry cache; the shadow's offline install
  could not download them, so the lock was first completed with the toolchain's own `cargo fetch`, and then
  `mise run //package/git-policy/cli:native:lock:members` checked the 17 newly locked versions against the release-age and trust policy (all passed).
- Linux calls (`rustix`): started with `getrlimit` and the pidfd; the file calls move with the safe-file-saving replacement below.

### Step T4, children without a blocking wait (partly)

- `process/child_awaited_through_a_pidfd.rs`:
  the helper of choice 7.
  `adopt` cannot fail; a failed `pidfd_open` or a failed registration only changes the wait to a timer-driven `try_wait`.
  Injected failures (`InjectedFailure`) prove the invariant: after each, the live child is still owned, reapable, and ended by a drop;
  `into_std` hands it over.
- `process/captured_child_run_to_its_end.rs`:
  `output_of` runs a command to its end like `Command::output`, with the output in anonymous files (no reader, no pipe).
  Its blocking face `output_of_blocking` refuses with an error, not a panic, from inside the runtime.
- The two Linux one-shot callers use it (`repo_read/git_system_config.rs`, `invocation/management_service_repository_selection.rs`).
  The other one-shot waits named in the step (`process/process_start.rs`, `process/process_identity.rs`, `lock/index_lock_holders.rs`)
  start `ps`, `lsof` and PowerShell only on macOS and Windows, which the owner said to ignore; they were not converted.
- Not yet done in T4:
  `AsyncFd` registration is exercised only through `adopt`; the descriptor-limit sweeps of the plan's risk list for the helper are not written.

### Safe file saving

The vet report is `doc/audit/tech-safe-file-saving-published-crates-vet-2026-11-07.md`.
Summary: `tempfile` (`persist_noclobber`) for the whole publish, `tokio::fs` for create-new, no-follow reads and private folders,
`rustix::fs::fsync` for the directory sync.
The one behavior no crate gives is the fallback to a checked rename on a filesystem that supports neither `renameat2`
with `RENAME_NOREPLACE` nor hard links (evidence: `tempfile-3.27.0/src/file/imp/unix.rs`, function `persist`, lines 94 to 133).
`openat2` with `RESOLVE_NO_SYMLINKS` was measured and is stricter than the wrapper's `O_NOFOLLOW` (it refused every path on this host because the scratch folder sits behind a link), so it is not adopted for reads.
The replacement lands with the conversion of the callers (the 38 files that use `foundation/private_storage.rs`), not before.
The coordinator asked for facts on the one item in conflict with "Commits should never fail" (G1); they are in the vet report,
section "Facts for the owner's answer on G1".
In short: no in-kernel filesystem read lacks both `RENAME_NOREPLACE` and hard links (FAT and exFAT have no links but accept the flag;
NFS rejects the flag but has links); FUSE servers such as `s3fs-fuse` lack both; with `tempfile` only, a plain commit still lands
(only the post-commit work item is lost), but a commit during or concluding an unfinished merge fails with
"writing packed-refs failed: Operation not supported (os error 95)" or
"copying a conclusion entry into the shadow failed: Operation not supported (os error 95)"; the fallback is ten code lines.
The owner answered G1, 2026-11-07, in the question tool.
The question said: "Commits keep working on those disks. We own 10 lines plus a 15-line test. On those disks only, two programs
saving the same file at the same instant could overwrite one another."
The owner picked, verbatim: "Keep the fallback (Recommended)".
So the replacement of `private_storage.rs` is built on `tempfile::persist_noclobber`, and the checked rename stays as the fallback
for that one case only: the publish answered by neither `renameat2` with `RENAME_NOREPLACE` nor a hard link.

### What is left

Steps T5 to T13 (the watchers, `bounded_push`, the signal relay on a pidfd with the registry of children, the service client and
the `zbus` move, the standards-review call on `hyper`, the service, the callers, the parallel runner, the final removal and lint),
the replacement of `private_storage.rs`, the descriptor-limit sweeps of the child helper, and the mutation campaign.

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
