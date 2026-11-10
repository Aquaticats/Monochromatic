# Handover: moving the Git wrapper onto tokio

## Current state

- Written:
  2026-11-07T13:30:00-05:00.
- Branch and worktree:
  `feat/git-policy-cli-on-tokio` in `.claude/worktrees/wrapper-on-tokio`.
  Main was last merged into it at `6c0ff76dc9` (the coordinator merged the branch at `48611f7e09`);
  the coordinator may merge gated checkpoints.
  Main merged into it by fast-forward at `cb0b734efa` (the coordinator had merged the branch at `5e441f25d3`);
  the coordinator may merge gated checkpoints.
- Plan and the owner's answers:
  `doc/planning/git-policy-cli-full-tokio.md`,
  sections "The owner's answers at 2026-11-07" and "Progress on the branch".
- Landed on the branch:
  the guard that a forwarded command makes no thread (T2),
  the budget,
  limits,
  plan,
  lazy runtime,
  blocking-job
  helper and test-only probe (T3),
  the digest,
  time stamp and memory-headroom swaps,
  the pidfd child helper and the captured child run,
  and its first two callers (T4,
  partly);
  then,
  by the second agent:
  the `decide` race closed (`4d372fdd80` red,
  `7f39a05469` green) and
  step T5,
  the child watcher as a task (`fa518c33a1`,
  `7e4dc3aefc`,
  `c5b4559b88`).
  Details and gate counts:
  the plan,
  "The `decide` race closed"
  and "Step T5".
- Vet report for safe file saving:
  `doc/audit/tech-safe-file-saving-published-crates-vet-2026-11-07.md` (recommended,
  not adopted).

## How to run the gates

- Host lib tests:
  `mise run //package/git-policy/cli:native:test:host -- <filter>`.
- Host binary tests:
  `mise run //package/git-policy/cli:native:test:binary:host -- <filter>`.
- Clippy:
  `mise run //package/git-policy/cli:native:clippy`;
  the package lint:
  `mise run //package/git-policy/cli:lint`.
- Container gate,
  in its own unit,
  one `mise run` per call:
  `systemd-run --user --unit=<name> --collect --property=MemoryMax=40G --property=CPUQuota=2400% --property=TasksMax=4096
  --property=IOWeight=10 --working-directory=<worktree> --setenv=PATH="$PATH" bash -c 'mise run
  //package/git-policy/cli:native:test:container > <log> 2>&1'`.
  A cold build in the container takes about half an hour at its 2 CPUs;
  the tests take about ten minutes;
  Clippy about four.
- Latency:
  `mise run //package/git-policy/cli:native:perf -- --output <json>`;
  run it twice on the same binary for the spread.
- A dependency change:
  edit the use site,
  `mise run sync:files`,
  `mise run //package/git-policy/cli:native:lock:members`.
  When the shadow cannot download a crate offline,
  complete the lock with the toolchain's own `cargo fetch` first,
  then run the
  `lock:members` task so the release-age and trust policy checks the new versions.

## Where steps T6 onward stand (2026-11-07T15:55:00-05:00)

- Branch `feat/git-policy-cli-tokio-steps-6-onward` in `.claude/worktrees/tokio-steps-6-onward`,
  created from the frozen tip `9afcac4a26`
  of `feat/git-policy-cli-on-tokio`;
  main is not merged into it (the coordinator says when,
  after the repository-lint branch and the tokio
  branch are in main;
  carry edits for `push/child_watch.rs` to its split file).
- Done:
  T6 (`start_retry`,
  `bounded_push`,
  `bounded_query`,
  the repair tool's wait and reaper) and T7 (the signal relay on a pidfd,
  no guard wait,
  a service's stop not relayed).
  Details,
  controls,
  measurements and gates:
  the plan,
  "Steps T6 and T7".
- Written for the owner:
  open choice 12 (what a stop of the service does to the Git child of a commit it is preparing),
  with a ranking.
- T10 (the service),
  2026-11-08T05:40:00-05:00:
  merged into main by the commit `merge(git-policy-cli): step T10 ...` of 2026-11-08.
  Gate of the head `4e3bde33c0`:
  5349 of 5349 passed,
  Clippy and lint clean.
  The failure of the second gate (`commit_timing::a_commit_that_reaches_a_foreign_index_holder_late_is_attributed_to_that_wait`) was the record's holder kind decided by the last notice;
  cause by instrumentation,
  safety answer (no lock is touched) and fix:
  the plan,
  "The holder kind of the timing record".
  Worker cap 4,
  lowered-limit sweep,
  refusal-path controls,
  resolver fix and bounded renewal stop:
  the plan,
  "Step T10,
  the worker cap,
  the sweep,
  the refusal paths and the bounded stop".
  Left for T11:
  the work loop thread and its `std::sync::mpsc` queue,
  the synchronous landing loop with its sleeps,
  its inotify wait and its queues,
  and the owner's pick for open choice 12 ("C:
  shutdown stops its Gits (Recommended)",
  2026-11-07T22:19:43-05:00):
  the stop handler only records the stop and the service's shutdown signals the Gits it waits on through a registry,
  with bounded waits then `SIGKILL`,
  a commit's work kept on its side ref,
  and the red test that a stop during a long commit Git leaves no Git and no `index.lock` within the bound.
  Open failures to read before trusting a green run:
  the five tests of `foundation::kept_names_in_the_git_directories::tests` that fold with the real `/proc` failed together once in a host run
  (`could not be looked at in time`,
  0.02 s;
  four instrumented repeats passed,
  no cause measured;
  the scan runs only when a kept name exists under its earlier spelling).
- T11 (choice 12 and the service queues),
  2026-11-07T23:40:00-05:00:
  branch `feat/git-policy-cli-tokio-t11`,
  gate 5371 of 5371,
  Clippy and lint clean.
  Done:
  the owner's "C:
  shutdown stops its Gits (Recommended)" (the handler records,
  the service's shutdown tells the waiters,
  each ends its own child,
  the process ends by the signal within `SETTLE_CEILING`;
  a stopped commit keeps its work on `refs/cli-git/kept/<n>`;
  no lock table),
  and the two queues of the service on tokio channels.
  Details,
  controls,
  judgment calls for the owner's veto and the list of remaining sleeps:
  the plan,
  "Step T11".
  Not done,
  and blocked:
  the landing loop,
  the work loop and their sleeps,
  the inotify wait,
  the client and provision sleeps.
  They wait for an `async` Git leaf (`run_git` is synchronous and the bridge refuses a call from inside the runtime;
  79 production files and 203 mentions of `run_git` or `run_git_checked`);
  the plan names the order.
  For T12:
  the parallel runner (`foundation/parallel_runner.rs`,
  `side_by_side_threads.rs`,
  `scoped_start.rs`;
  `drain_directories` of the work loop's round runs on scoped threads) and,
  before the loops,
  the leaf-first `async` Git.
  The signed-ceiling branch owns `landing/service_index_*` and `commit/native_commit*`;
  a stop does not wake a landing that waits for a foreign index lock (bounded by the settle bound).
  Host runs of the two askpass controls are skipped (`-- -- --skip askpass`),
  because they need `GIT_ASKPASS` unset.
- Not started:
  T11 (callers:
  the commit command and the policy run as `async fn`),
  T12 (parallel runner),
  T13 (removal,
  lint).
  T10 follows the owner's answer to choice 4
  ("Rewrite per-thread state"):
  the production `thread_local!` items to turn into task-local or context values are in
  `service/service_busy_step_record_on_this_thread.rs`,
  `service/service_busy.rs`,
  `landing/service_durable.rs`,
  `landing/service_index_*.rs`,
  `landing/service_built_checks.rs`,
  `landing/service_land_git_steps_and_import_stream.rs`,
  `transaction/transaction_git_step_bounds.rs`,
  `repo_read/repo_read_tree.rs` (an `Rc`),
  `repo_read/repo_read_odb.rs`,
  `foundation/private_storage.rs`,
  `foundation/scoped_start.rs`,
  `process/pid_namespace.rs`,
  `worktree/add_timing.rs`,
  `invocation/panic_notice.rs`;
  the remaining production `thread::sleep` calls
  (`landing/service_land_follow.rs`,
  `landing/service_index_hold.rs`,
  `landing/service_index_wait.rs`,
  `landing/service_index_attempts.rs`,
  `lock/index_holder_wait.rs`,
  `push/auto_push.rs`,
  `service/service_client*.rs`,
  `service/service_server.rs`,
  `service/service_provision.rs`,
  `transaction/transaction_git.rs` (`run_git_pausing`),
  `transaction/transaction_phase.rs`) are T10 and T11 work.
- The one `thread::sleep` that stays by design is the forwarded command's start pause in `push/start_retry.rs` (`sleep_for`):
  a forwarded command builds no runtime.

- Merges (2026-11-07):
  `feat/git-policy-cli-on-tokio` is in main as `6e8521c8fa` (gate 5222 of 5222,
  container Clippy clean,
  package lint 0 and 0).
  T6 and T7 followed it (gate 5229 of 5229,
  2 slow,
  container Clippy clean,
  package lint 0 and 0).
  No test failed in any of these gates,
  so there is no failure text to record as open.
  The lint branch's rules (named closures,
  the 300 code-line limit) found work in the tokio code at the merge:
  the closures of `systemd_dbus.rs` and `standards_review_client.rs` are named functions now,
  the stop and sweep of a push live in
  `push/stop_of_a_push_and_sweep_of_its_locks.rs`,
  and `end_with_its_group` lives in `testing/child_ended_with_its_whole_group.rs`.
- T10 waits for the commit-stall branch (the coordinator,
  2026-11-07),
  which changes the same service files.
## Open work, in order

- Done since the previous handover (details and figures:
  the plan,
  "The coordinator's three settlements after T5",
  "The two gate failures
  of main at `85e6d2ab99`",
  "Step T8" and "Step T9"):
  the T5 one-shot form on runtime pipes that keep all output,
  the futex wait of the
  name claim,
  the measured `HELD_NAME_WAIT` (60.3 s,
  because a hold of 6.03 s was seen) and `WATCHER_END_BOUND` (stop grace plus one
  second),
  the test-helper fix for the `DirectoryNotEmpty` gate failure,
  the measured cause of the left ref lock,
  T8 (`zbus` 6) and T9
  (`hyper`,
  `ureq` gone).
- Not started,
  in the plan's order:
  T6 (`start_retry`,
  `try_wait` loops,
  `sweep_after_exit`),
  T7 (the signal relay on a pidfd with the
  registry of children),
  T10 (the service),
  T11 (the callers become `async`,
  the synchronous bridges go),
  T12 (the parallel runner),
  T13 (removal of what nothing uses:
  the hand wire format `service/systemd_dbus_wire_format.rs` and its fuzz target,
  the thread helpers,
  the poll constants;
  the source guards;
  `native:perf` with a same-binary control;
  the mutation campaign).
  T6 to T12 convert code whose
  callers are synchronous,
  so they are in practice T11's leaf-first conversion.
- Left on purpose by the settlements:
  the `LeaveRunning` waiter thread and the `Unbounded` blocking `wait` of `output_watched_within`
  (a reaper that does not end the child when dropped is needed first).
- Settled by the owner,
  2026-11-07,
  "3x worst case" (plan,
  "The owner's pick on the waits"):
  `HELD_NAME_WAIT` and the stop's ceiling are three
  times the longest delay measured (about 18.1 s);
  a stop waits past its grace while a member is in state `D`.
  Still open:
  the cause of the 6.03 s
  (disk or scheduler) is not measured;
  the index-lock fallback belongs to another agent.
- The mutation campaign is the coordinator's.
  Files changed since the last list:
  `src/native/process/step_output_read_whole_through_pipes.rs`,
  `src/native/push/child_watch.rs`,
  `src/native/foundation/name_held_while_one_thread_publishes_it.rs`,
  `src/native/foundation/private_storage.rs`,
  `src/native/service/systemd_dbus.rs`,
  `src/native/standards_review/standards_review_client.rs`,
  `src/native/process/stop_waits_while_a_member_cannot_be_interrupted.rs`,
  `src/native/process/process_tree.rs`,
  `src/native/push/bounded_push.rs`,
  `src/native/push/child_watch_clock_marks_and_stopping.rs`,
  `src/native/foundation/worst_delay_seen_on_this_host.rs` (the test helper
  `src/native/testing/test_support.rs` is not mutated code).
- T4 is done on Linux (descriptor-limit sweeps landed).
  T5 landed as a task behind an unchanged synchronous face;
  its one-shot
  pidfd form (`output_watched_within`,
  `EndAt` reads pipes and returns all output,
  the pidfd helper keeps only a tail) needs an owner-visible
  decision first;
  the plan says why.
- Next,
  in the order that needs no async callers:
  T9 (`hyper`) and the `zbus` 6 client of T8;
  T6 to T12 are the leaf-first conversion of the
  callers (T11's work in practice),
  see the plan,
  "Where the remaining steps stand after T5".
- Running at this checkpoint:
  the user unit `tokio-gate-merge2` (container gate of the tree after the merge `d628f74005`;
  log
  `${HOME}/temp/agent/tokio-steps/gate-merge2.log`).
  It writes its own log;
  read it,
  then record its count in the plan beside
  "Main merged again at `d628f74005`".
  Previous gates:
  step 1 5133 of 5133,
  T5 5134 of 5134,
  both Clippy clean.
- Exact next step for the next agent:
  T9 (`hyper`) or the `zbus` 6 client of T8;
  the catalog already names `zbus >=6.2.1`,
  `hyper >=1.12.0`,
  `hyper-util`,
  `hyper-rustls`,
  and the local registry cache holds `zbus-6.2.1` and `hyper-1.12.0`,
  so the plan's "floor 5.19" is out of date.
  `native:perf` was not run (the plan asks for it after T10,
  T12,
  T13).
  Scratch logs:
  `${HOME}/temp/agent/tokio-steps/`.
- The `HELD_NAME_WAIT` and `WATCHER_END_BOUND` values (five seconds each,
  the start retry's budget) have no measurement beside them;
  they
  are open to the owner's veto.
  The name claim is a bounded wait,
  not lock-free,
  which deviates from "add no lock" for the coordinator to accept.
- T5 to T13 as listed in the plan,
  with the owner's answers applied (task-local state instead of a `LocalSet`,
  the registry of
  children for the relay,
  no inline fallback when no thread starts).
- `foundation/private_storage.rs`:
  the exclusive publish is `tempfile::persist_noclobber` with the owner's checked-rename fallback (G1 answered,
  "Keep the fallback (Recommended)");
  the module's other faces move to `tokio::fs` with the callers in T11.
  Two facts for the owner are in the plan ("Safe file saving:
  the publish replaced"):
  the in-process race of `decide` on a filesystem with neither,
  and the service's index-lock refusal there.
- `zbus` 6 for the service client (step T8):
  main's `toolchain.lock.json` now names `nightly-2026-11-07`,
  so it compiles;
  the catalog
  floor is 5.19 and the release-age exclusion names `zbus@5.19.0`,
  both to be moved to 6.
- The mutation campaign:
  the user told this agent not to run mutation campaigns and to delegate them to the coordinator.
  Scope for the coordinator:
  `src/native/foundation/name_held_while_one_thread_publishes_it.rs`,
  `src/native/foundation/lazy_runtime.rs`,
  `src/native/foundation/runtime_plan.rs`,
  `src/native/push/child_watch.rs`,
  `src/native/push/child_watch_clock_marks_and_stopping.rs`,
  `src/native/foundation/private_storage.rs`,
  `src/native/service/service_busy.rs`,
  `src/native/process/child_awaited_through_a_pidfd.rs`,
  `src/native/process/captured_child_run_to_its_end.rs` and the T3 files under `src/native/foundation/` (`runtime_limits.rs`,
  `runtime_plan.rs`,
  `lazy_runtime.rs`,
  `blocking_job.rs`),
  against a gate image of this branch.
  A scoped run for the first two was started and stopped at the planted-controls stage;
  it produced no mutant results.
- Not started in this session:
  T5 to T13.
  T5 (the watchers) follows the T4 pattern:
  the synchronous callers keep a bridge (`output_of_blocking` enters the lazy runtime and refuses from inside it) until their own callers become asynchronous at T11,
  as the plan's T4 text says.
- The owner's G1 answer was recorded at 2026-11-07T07:40:00-05:00 (the commit `1cff4935cd`).
- Main merged again at `ae740e9e6d` (toolchain lock,
  archive route,
  daily update):
  the only conflict was `package/git-policy/cli/Cargo.lock`;
  it was rebuilt from main's lock by the toolchain's `cargo fetch` (the offline index lacked the new gix releases) and `native:lock:members` (61 versions checked;
  `cpufeatures@0.3.1` set aside by the policy,
  0.3.0 locked).
  Gate after that merge:
  see the plan,
  "Gates after the merges".
- Main merged in at `9156ca885b` (wrapper 0.6.0 released;
  `child_environment.rs` from the background-never-prompts change came with it).
  Container gate after that merge:
  5127 tests run,
  5127 passed (1 slow),
  8 skipped;
  Clippy clean.

## Step T12, its first part, 2026-11-08T18:40:00-05:00

- Branch and worktree:
  `feat/git-policy-cli-tokio-steps-12-and-13` in
  `.claude/worktrees/pi-tokio-step-t12`,
  from main at `1a907ccfb9`.
  Not merged into main.
  Corrected 2026-11-08T21:05:00-05:00:
  this work is on main now,
  merged as `6c3d3ef739` from `feat/git-policy-cli-tokio-async-git-leaf-and-its-callers`;
  the branch name above is the one the record was written with.
- Done,
  and the whole of what T12 could be from this tree:
  the process's one bound on blocking jobs
  (`foundation/blocking_jobs_within_the_budget.rs`),
  and the ordered runner with each item as a job of the runtime under that bound
  (`foundation/parallel_runner_on_the_runtime.rs`),
  with `runtime_plan::budgeted_workers` as the one derivation of the worker count the
  runtime plan and the bound share.
  Commits `2b1f830568` and `c1e6a78a4c`.
  Details,
  the controls,
  the planted controls and the checks:
  the plan,
  "Step T12,
  the runner's jobs on the runtime".
- Not done,
  and blocked,
  with the evidence in the plan:
  the per-call-site conversion of the seven `run_ordered` sites and the nine
  `run_all_side_by_side`/`run_beside_caller` sites.
  Each must hand its items,
  lanes and work over as shared values (`Arc`) because a `spawn_blocking` job must own what it reads,
  and the service round (`landing/service_round.rs`) runs Git inside those jobs,
  so it needs the leaf-first `async` Git first.
  That leaf does not exist:
  `transaction/transaction_git.rs` has `pub fn run_git`,
  not an `async fn`,
  and T11 wrote down why it stopped there.
- The one thing this step found that the plan did not have:
  a refused `spawn_blocking` pushes the job into the pool's queue and then panics,
  and the queued job keeps everything it captured for the life of the process.
  A permit inside the job is therefore lost for good,
  and a sending end inside it keeps a channel open forever.
  Both were measured as planted controls (30 s and a 300 s timeout).
  The design that carries it:
  the place is held by the awaiting frame,
  and the consumer ends when every job has returned (a counter and a `Notify`),
  then drains what was already sent.
  The next agent must keep that shape in any new job helper.
- Exact next steps,
  in order:
  1.  The leaf-first `async` Git:
      an `async fn` beside `run_git` in `transaction/transaction_git.rs`
      (`run_git_once`/`run_git_watched`/`run_git_once_with`/`run_git_pausing` become
      `async` cores over the async pipe reader and `OwnedChild::wait`,
      with the synchronous
      faces kept as bridges),
      then its callers one folder at a time.
  2.  Then `landing/service_round.rs` (`drain_directories`) onto
      `run_ordered_on_the_runtime`,
      and the pure-file sites
      (`worktree/worktree_trash_entries.rs`,
      `subvolume/subvolume_index_refresh_of_every_worktree.rs`,
      `worktree/worktree_convert_command.rs`,
      the copy and repair sites of `side_by_side_threads.rs`) once their state is shared.
  3.  Then T13's removals and lint:
      no `std::thread`,
      no `std::process::Command` outside the guarded start,
      no `thread::sleep`,
      and the hand D-Bus wire format with its fuzz target.
      None of the three guards can be tightened before step 2 lands,
      because `foundation/scoped_start.rs`,
      `foundation/side_by_side_threads.rs` and the rest still start threads and sleep.
- Not run,
  for the next agent:
  the container gate of this branch (no gate image was made),
  `native:perf`,
  and the mutation campaign.

## Step T12, the leaf-first asynchronous Git and the awaited runner, 2026-11-08T19:29:56-05:00

- Branch and worktree:
  `feat/git-policy-cli-tokio-steps-12-and-13` in
  `.claude/worktrees/pi-tokio-async-git-leaf`,
  from `6240b51a7a`.
  Not merged into main.
  Corrected 2026-11-08T21:05:00-05:00:
  this work is on main now,
  merged as `6c3d3ef739` from `feat/git-policy-cli-tokio-async-git-leaf-and-its-callers`;
  the branch name above is the one the record was written with.
- Done:
  a Git step can now be awaited by a task of the runtime
  (`transaction/transaction_git_awaiting.rs`:
  `run_git_awaiting`,
  `run_git_checked_awaiting`,
  `push/child_watch_output_watched_on_the_runtime.rs`,
  `push/start_retry.rs`'s `RefusableAwaitingStart`),
  and an ordered round can run items that await one
  (`foundation/parallel_runner_on_the_runtime_awaiting.rs`:
  each lane as a task,
  a lane's items one after another,
  storage work on the blocking pool inside the item,
  the reporting shared with the blocking form through `finish_the_run`).
  Commits `e0359dc84c` and `a2f901dd92`.
  Details,
  controls,
  the measurement and the checks:
  the plan,
  "Step T12,
  the leaf-first asynchronous Git and the ordered runner that awaits".
- The synchronous faces are unchanged and stay that way until their last
  caller goes:
  a step with an `Unbounded` or `LeaveRunning` bound makes no pipes,
  so the synchronous `run_git` works today from inside the runtime for a
  branch write,
  and a bridge around the whole runner would take that from every pool job
  that moves a branch.
- The one rule this step measured,
  for any later helper:
  a synchronous `Drop` that waits for a task holds a worker,
  and while the workers of a small runtime are all inside such waits the tasks
  that would end them cannot run.
  Four awaited Git steps took 38.2 s that way and 10.8 ms through
  `ChildWatch::close_awaiting`,
  and one awaited step alone took the whole 19.1 s of `WATCHER_END_BOUND`.
- Not done:
  every production call site.
  None was converted,
  so no command changed behaviour.
  Converting one makes its function `async`,
  which makes its callers `async` or a bridge that enters the runtime,
  and that chain runs to the command or to the service loop.
  The 67 production files that call `run_git` are converted one folder at a
  time from the top of each chain.
- Exact next steps,
  in order:
  1.  Convert one folder whose chain to a boundary is short,
      keeping the package's tests green at each commit.
      The leaf files are `transaction/`,
      `push/` and `foundation/`,
      and a converted step calls `run_git_awaiting` where it called `run_git`.
  2.  Convert the landing's handlers and then `landing/service_round.rs`
      (`drain_directories`) onto `run_ordered_awaiting_on_the_runtime`,
      with each item's storage work inside
      `run_blocking_job_within_the_budget`.
  3.  Then the pure-file sites of the blocking runner
      (`worktree/worktree_trash_entries.rs`,
      `subvolume/subvolume_index_refresh_of_every_worktree.rs`,
      `worktree/worktree_convert_command.rs`)
      and the `run_all_side_by_side`/`run_beside_caller` sites.
  4.  Then T13's removals and lint.
- Checks,
  the whole suites once at the end:
  `native:test:host` 4794 run,
  4794 passed (one slow),
  8 skipped;
  `native:test:binary:host` 733 run,
  733 passed (two slow),
  3 skipped;
  `//package/git-policy/cli:lint` 0 errors and 0 warnings.
  Both suites ran with `CLI_GIT_TEST_ROOT` under a short path,
  without which `service_mode::the_same_start_with_the_real_git_listens` fails
  on this host for the `SUN_LEN` reason
  `doc/handover/cli-git-rust-implementation.md` already records,
  and both with the run's own control group at `TasksMax=4000`,
  because a smaller one refused `git-lfs`'s filter process to two LFS
  controls.
- Not run,
  for the next agent:
  the container gate of this branch,
  `native:perf`,
  and the mutation campaign (paused).

## Step T12, the first production folder on the awaited Git runner, 2026-11-08T21:05:00-05:00

- Branch and worktree:
  `feat/git-policy-cli-tokio-call-sites-from-the-top-of-each-chain` in
  `.claude/worktrees/pi-tokio-convert-the-first-call-sites`,
  from main at `bbadfac560`.
  Not merged into main.
- Done:
  `health/` is the first production folder whose Git reads are awaited on the
  runtime,
  and the report of `git monochromatic-git-policy-cli doctor` is built
  by one task instead of by steps that each enter the runtime for their pipes.
  Commits `a669e2a59a` (the control that does not compile) and `ae4e1a57c7`
  (the conversion).
  The scripts that measured the chains are
  `${HOME}/temp/agent/tokio-callsite-measure-20261109/callgraph.ts` (the call
  graph over the production sources),
  `ancestors.ts` (one function's ancestors,
  one level at a time) and
  `chains.ts` (the ancestor set of every site).
  Details,
  the measurement of every other folder's chain,
  the controls and the
  checks:
  the plan,
  "Step T12,
  the first production folder on the awaited Git runner".
- The shape a later conversion copies:
  the leaf functions become futures and call `run_git_awaiting`;
  the outermost synchronous function of the chain enters the runtime once
  (`doctor_action`,
  and `plan_doctor` in `invocation/management.rs`);
  a shared read that has callers in reserved folders gets a second face beside
  its synchronous one (`config_snapshot::read_config_snapshot_awaiting`) rather
  than a conversion that would reach `landing/` and `worktree_copying/`.
- Why `health/` and not another folder,
  measured:
  its chain is six functions above the folder
  (`run_process` > `plan_invocation` > `plan_management` >
  `plan_management_with_section` > `plan_doctor` >
  `doctor_action_with_section`) and then three functions inside it,
  and its
  three Git readouts are called by nothing but each other,
  the folder's own
  controls and `plan_doctor`.
  Every other folder that the brief does not reserve has either a longer chain
  through the commit path (`conflict/`,
  `commit/`,
  `merge/`,
  `push/`,
  `invocation/`),
  a caller in a reserved folder (`config/`,
  `repair/`,
  `shadow/`,
  `standards_review/`,
  `candidate/`),
  or items of the blocking runner (`worktree/`).
- What stays as it is,
  with the reason in the plan:
  `invocation/forwarded_command.rs::read_config_value`,
  because converting it
  builds the runtime on the forwarded-command path the owner's lazy-runtime
  choice and `binary_forwarded_command_makes_no_thread_tests.rs` keep free.
- Exact next steps,
  in order:
  1.  Item 3's pure-file sites,
      and read the plan's list of what they need
      first:
      each of the three lends the blocking runner state that has to be
      owned before the awaited runner can take it (`IndexLock<'a>`,
      `RewriteOne<'a>`,
      `&dyn Platform`,
      `&Interrupt`),
      and each says lines while
      it works,
      which a `'static` blocking job cannot do through the sink the
      awaited runner lends it.
      Decide that line path first;
      it is one decision for all three.
  2.  Then item 2:
      the landing's handlers and `landing/service_round.rs`.
  3.  Then item 4 (T13's guards),
      unchanged.
- Checks,
  both whole suites at the tip:
  `native:test:host` 4807 run,
  4807 passed (1 slow),
  8 skipped;
  `native:test:binary:host` 733 run,
  733 passed (10 slow),
  3 skipped;
  `native:clippy` clean;
  `//package/git-policy/cli:lint` 0 errors and 0 warnings.
  A lib run before the last two commits showed one failure,
  in a folder this branch does not touch,
  and the tip run of the same suite passed everything:
  `worktree_copying::worktree_copy_stage::tests::a_stamp_comes_only_from_a_clone_of_a_settled_file`
  asserts that a file written this instant is still inside the timestamp
  granularity,
  which a run of the whole suite under load cannot promise;
  the same binary passes it three times out of three when it is asked alone,
  from the same fixture root,
  which is three samples and not a measured cause:
  the suspected cause is the twenty milliseconds the control allows between the
  write and the clone,
  against the load of the whole suite.
  The folder is one another agent is changing,
  and three observations are what there is:
  it failed once inside a full run,
  passed three times out of three alone,
  and passed in the full run at the tip.
  Both suites ran with `CLI_GIT_TEST_ROOT=/var/tmp/cg/cli-git/fixtures/root`
  and with the run's own control group at `TasksMax=4000`,
  the conditions the "Step T12,
  the first production folder on the awaited Git runner" section records,
  with one addition the next agent needs:
  the fixture root has **two** length limits,
  not one.
  It must be short enough for a Unix socket under it to fit `SUN_LEN`,
  and it must be at least six path segments deep,
  because `worktree_add_binding`'s control `a_relative_pointer_without_links_is_bound`
  resolves `../../../../../../..` from a fixture directory and fails when that
  lands above `/`.
  Measured on this tree:
  `/var/tmp/cg-fixtures` (three segments) fails that control and
  `/var/tmp/cg/cli-git/fixtures/root` (six) passes it,
  the same test binary both times.
  `native:perf`,
  compared with the numbers the T11 section records,
  is in the plan's section for this step.
- Not run,
  for the next agent:
  the container gate of this branch (no gate image was made) and the mutation
  campaign (paused by the owner).

## Step T12, the line path and the three pure-file sites, 2026-11-08T23:05:00-05:00

- Branch and worktree:
  `feat/git-policy-cli-tokio-pure-file-sites-on-the-awaited-runner` in
  `.claude/worktrees/pi-tokio-pure-file-sites-on-the-awaited-runner`,
  from main's tree at `3db3002750` (origin/main has moved since;
  this branch is not merged).
  Commits `6718083e03`,
  `1a7a356e46`,
  `f91efa791b`,
  `f1da73cd9f`,
  `4712446a7c`,
  `a5bf18702f`,
  `19c78736dc`,
  `e96655c92e`,
  `2b0b47c964` and `c7ad6b145a`.
- Done,
  and the whole of what item 3 could be from this tree:
  the line path for an item whose work runs in a `'static` job,
  and the three pure-file sites on `run_ordered_awaiting_on_the_runtime`:
  `worktree/worktree_trash_entries.rs`,
  `subvolume/subvolume_index_refresh_of_every_worktree.rs` and
  `worktree/worktree_convert_command.rs`.
  No site of the blocking runner's scoped threads is left in those three files.
- The decisions,
  this agent's under the owner's delegation and open to the owner's veto
  (the plan,
  "Step T12,
  the line path and the state of the pure-file sites"):
  the state each site lends the runner is shared with `Arc`,
  and an item's lines are an owned `LinesToTheConsumer<R>` it moves into its job.
  Neither adds a lock,
  both keep the lines in input order and as they happen,
  and nothing of the three sites is reached by a forwarded command.
- Corrected record:
  the "Step T12,
  the first production folder on the awaited Git runner" section says the three sites "all say lines while they work".
  Read again,
  only `worktree/worktree_convert_command.rs` does;
  the other two say theirs at `Event::End`,
  on the consumer,
  so the line path had one user and not three.
- One fact for any later helper,
  measured:
  `tokio::runtime::Handle::try_current()` is `Ok` on a thread of the blocking pool
  (thread name `git-policy-worker`) and `Err` on a scoped thread of `parallel_runner.rs`,
  so a control can tell "this work is in the process's runtime" from "this work is on a thread of
  its own".
  What it cannot tell is a worker of the runtime from a job of its pool;
  the place control does that
  (`hold_the_only_place` in `testing/parallel_test_support.rs`:
  with the bound held to one place,
  work that takes a place waits and work that runs on a worker does not).
- Exact next steps,
  in order:
  1.  Item 2:
      the landing's handlers and `landing/service_round.rs` (`drain_directories`)
      onto `run_ordered_awaiting_on_the_runtime`,
      with each item's storage work inside `run_blocking_job_within_the_budget`.
      The landing's items say lines while they work,
      so they take the owned sink by moving it into the job,
      as `worktree_convert_command.rs` does.
      Another agent has an unmerged branch in the landing's index handling;
      read it before touching `landing/service_index_*` or `commit/native_commit*`.
  2.  Then the sites still on `parallel_runner.rs`:
      `worktree/worktree_copying/`,
      the copy and repair sites of `foundation/side_by_side_threads.rs`,
      and the rest of the seven `run_ordered` sites the plan lists.
  3.  Then item 4 (T13's guards),
      unchanged.
- Checks,
  the whole suites once at the tip,
  with `CLI_GIT_TEST_ROOT=/var/tmp/cg/cli-git/fixtures/root` and the run's own control group at
  `TasksMax=4000`:
  `native:test:host` 4826 run,
  4826 passed (one slow),
  8 skipped;
  `native:test:binary:host` 733 run,
  730 passed (three slow),
  3 failed,
  3 skipped,
  and the same suite once more at the same tip,
  after the review's corrections:
  733 run,
  733 passed (three slow),
  3 skipped.
  The three failures of the first run are the `service::the_settle_command_*` group,
  which passes 4 of 4 when asked alone and whose own reason is a deadline
  ("the sockets and processes of the earlier helper could not be looked at in time"),
  under a load average of 60 to 91 with another agent's whole suite running beside it;
  the same control is already recorded twice as a suite-only failure that passed alone and in the
  next full runs (`doc/handover/cli-git-rust-implementation.md`,
  and the plan's T4 section).
  `native:clippy` clean;
  `//package/git-policy/cli:lint` 0 warnings and 0 errors.
- `native:perf`,
  nine arms with two binaries over two windows,
  the second interleaved (main's,
  this tree,
  main's,
  this tree),
  so each binary has arms on both sides of the other's:
  every arm inside the 2,000 ms forwarded bound by three orders of magnitude
  (added medians 1.4 to 2.2 ms);
  commit through the service paired delta medians 146.8,
  111.1,
  119.2,
  39.5 and 60.5 ms for this tree and 45.5,
  56.7,
  171.4 and 58.5 ms for main's,
  and signed commit 167.8,
  99.5,
  95.2,
  161.5 and 89.4 ms against 108.5,
  93.6,
  123.3 and 111.9 ms.
  Each binary's own spread covers the whole difference (3.7 times on this tree's commit medians,
  3.8 times on main's),
  the first window's apparent separation reversed in the interleaved one,
  and the 5,000 ms bar was broken by both binaries under this load.
  So no arm here attributes anything to this step,
  and no file this step touches runs in a commit.
  The plan,
  "Step T12,
  the line path and the state of the pure-file sites",
  "Latency".
- One defect this step carried and the binary suite caught:
  a conversion,
  now a job of the blocking pool,
  could no longer learn where the system configuration and attributes files are,
  because the bridge that starts a child refuses from inside the runtime.
  Fixed by `LinuxPlatform::with_its_conversion_environment_settled`
  (the plan says what it pins).
- Not run,
  for the next agent:
  the container gate of this branch (no gate image was made) and the mutation campaign
  (paused by the owner).
- A review of this branch by another vendor (`${HOME}/temp/agent/reviews/tokio-pure-file-sites-sol.txt`)
  found two defects in what the code promises,
  both right,
  and both fixed in `c316f98b42` and `51e5958ff6`
  (the plan,
  "What a review of the branch found,
  and what changed"):
  a rewrite that panicked after installing its index was promised as unchanged
  (the kinds of end are counted apart now,
  and only a job that never started is promised that),
  and
  the place controls gave their place back on a clock instead of on an attempt
  (`jobs_waiting_for_a_place` in `blocking_jobs_within_the_budget.rs`,
  test-only,
  and the holder reports the ask each control asserts).
  Measured for the second:
  with the old helper and a conversion run on the worker after a 1,500 ms
  wait,
  the conversion control passed although no place was taken;
  with the ask it fails.
- Round 2's checks,
  only the folders a fix touches plus Clippy and the package lint,
  and no whole-suite run:
  `worktree_trash_entries` 7 of 7,
  `worktree_convert` 86 of 86,
  `subvolume_index` 45 of 45,
  `parallel_runner` 20 of 20,
  `native:clippy` clean,
  `//package/git-policy/cli:lint` 0 warnings and 0 errors.
- A second review (the same file,
  a third finding) held as well:
  the pool queues a blocking job before it asks for a thread
  (`tokio-1.54.1`,
  the version in `Cargo.lock`:
  `runtime/blocking/pool.rs:624` pushes,
  `:628` asks,
  `:499` returns `NoThreads`,
  `:386` panics,
  `run_worker` at `:657` drains),
  so "no thread" never proved a job would not run,
  and the index report's "those stay as they were" was measured to be false (4 of 4 rewrites ran
  after the report,
  in a child whose process limit is one).
  Fixed by `foundation/the_claim_a_job_takes_before_running_it.rs`:
  a job takes the claim before its first effect and the reporter takes it after the refusal,
  and one atomic exchange decides,
  so the promise holds and a job the report gave up on does nothing.
  The error texts of `BlockingJobError` and `OrderedRunError`,
  the trash disposal's sentence and two
  control frames were reworded,
  and no other site promises a file
  (the plan,
  "What a review of the branch found,
  and what changed").
  Round 3's checks:
  122 controls of the folders touched
  (`blocking_job`,
  `the_claim_a_job`,
  `subvolume_index`,
  `parallel_runner`,
  `worktree_trash`) 122 of 122,
  `native:clippy` clean,
  `//package/git-policy/cli:lint` 0 warnings and 0 errors.
- A third review (the same file) found three waits and bounds,
  all three right,
  fixed in `8a8ffaeea7` and `8881e8bc83` after `0bee2a3825` made the consumer's wait one function:
  the completion loop's wake branch could be switched off and then miss its final wake,
  the refresh read `.git/config` synchronously before its first await,
  and a refused job's place went back while the pool still held the job queued.
  The place and the claim travel together now
  (`foundation/the_place_of_one_blocking_job.rs`),
  a run reads what it needs on the blocking pool (`prepare_the_run`),
  and the wake branch is never switched off.
  `SPEC.md` records all three rules under "Every wait has a bound".
- Two controls that failed in a filtered suite and passed alone were the process's blocking bound against controls
  that ask for every item in flight at once:
  measured,
  the bound is 1,
  2 and 3 at CPU quotas of 100%,
  200% and
  400% while the request asks 4.
  `may_run_all_at_once` states each control's need now,
  and the two modules pass 68 of 68 at both quotas (`a3a924b37b`).
  The `SIGABRT at 125 s` was not reproduced;
  that control passed in 1.1 to 9.0 s,
  and the unbounded wait on its
  path is the one the first finding of this round removed.
- Round 4's checks:
  `blocking_job`,
  `the_place_of_one`,
  `the_claim_a_job`,
  `subvolume_index`,
  `parallel_runner` and `worktree_convert_command` 107 of 107;
  `native:clippy` clean;
  `//package/git-policy/cli:lint` 0 warnings and 0 errors.
- A fourth review found four things,
  all right,
  fixed in `37c5ac28ec` (a dropped caller revokes the claim),
  `e35cb2da53` (a conversion finds what it covers on the blocking pool),
  `9d73b723bc` (a preparation control's cleanup cannot wait for a reader that cannot come) and `6f032b9cc6`
  (a refused start says whether the step will run or already has),
  with the room seam of `a180366183`.
  The two controls the coordinator saw fail were measured:
  the index order control's waiting item could take
  the last place its peers needed (14.7 s in an 88-test selection,
  0.05 s alone,
  33.9 s in their run),
  and the
  conversion order control converted four worktrees at one place,
  which is four real conversions one after
  another at about 28 s each on a busy disk (105 s in a 19-test selection,
  1.1 s alone).
  A control now takes the room it needs (`room_for_blocking_jobs_in_a_control`),
  and the conversion controls
  convert only what their report needs.
  The plan,
  "A fourth review",
  and "The two controls the coordinator saw fail,
  measured".
- Round 5's checks,
  the whole selection at CPU quotas of 100% and 200%:
  218 run,
  218 passed.
  `native:clippy` clean;
  `//package/git-policy/cli:lint` 0 warnings and 0 errors.
- Main was merged into this branch at `69a0e7f75c` (the coordinator);
  the commits of this step are its own.

## Step T12, the landing round's controls, and the four things its move needs first

- Written:
  2026-11-09T11:40:00-05:00.
- Branch and worktree:
  `feat/git-policy-cli-tokio-landing-round-on-the-runtime` in
  `.claude/worktrees/pi-tokio-landing-round-on-the-runtime`,
  from main at `baf3d03fe6`,
  with main merged again at `5ffe9b2dcc` (it had moved to `6232f281d7`).
  Not merged into main.
  Commits `02601dbb76`,
  `8af808f1c0`,
  `ba49929c1c` and the merge `5ffe9b2dcc`.
- Done:
  the raw blocking runner's refusal claims only what it knows
  (the finding of `${HOME}/temp/agent/reviews/tokio-pure-file-sites-round-5-sol.txt`),
  and `landing/service_round.rs` has the controls its move must keep:
  the answer a drain of one directory after another gives,
  a lane's order,
  what a refused thread does partway through a round,
  and the measurement that says why the round cannot run its items as
  blocking jobs.
- Not done,
  and measured as not contained:
  the move of `drain_directories` onto the runtime.
  It needs four things first,
  and the plan's section
  "Step T12,
  the landing round:
  what blocks it and what now pins it"
  names each with its file and lines:
  an awaited push chain (about a dozen functions in six files,
  with the synchronous faces kept for `manual_push_probe` and `push_stacking`),
  an awaited maintenance (two functions),
  a claim renewal whose stop does not wait for a task,
  and the claims,
  the settles and the notices inside
  `run_blocking_job_within_the_budget`.
- The next agent starts with the awaited push chain,
  then the awaited maintenance,
  then the renewal's stop,
  then the round itself onto `run_ordered_awaiting_on_the_runtime`:
  `landing/service_round.rs` first,
  and `landing/service_work_run_round.rs` keeps a synchronous face that enters
  the runtime once.
- The refusal's three states are now separate values,
  which is what the review asked for:
  `MayStillRunAfterItsThreadWasRefused` (the runner that takes no claim),
  `NoThreadToStart` (a caller that took the claim before it answered),
  and `StartedAfterItsThreadWasRefused` (the job took the claim first).
  `OrderedRunError` carries the same three,
  and the index refresh counts the first with the ends whose result is not
  known.
- Checks at the tip,
  `CLI_GIT_TEST_ROOT=/var/tmp/cg/cli-git/fixtures/root` and the run's own
  control group at `TasksMax=4000`:
  `native:test:host` 5087 run,
  5087 passed (one slow),
  12 skipped;
  `native:test:binary:host` 744 run,
  736 passed (four slow),
  8 failed,
  3 skipped,
  all eight failures pre-existing and measured as such (the plan's section for
  this step names them and the base-arm run that failed the same way);
  at the merged tip `native:test:host` 5164 run,
  5164 passed (one slow),
  12 skipped,
  and `native:test:binary:host` 757 run,
  748 passed (three slow),
  9 failed,
  3 skipped,
  the ninth (`client_service::an_index_writer_waits_for_the_service_to_finish_landing_and_says_so`)
  being main's,
  measured the same way;
  `native:clippy` clean;
  `//package/git-policy/cli:lint` 0 errors and 0 warnings.
- Open failure to read before trusting a green binary run,
  measured and not this branch's:
  `fault_trace::a_worktree_copy_killed_at_any_call_leaves_a_recoverable_repository`
  fails 2 of 2 alone and once inside a full run,
  "6 of 47 kills left the repository unrecovered",
  each with "a staging directory stays after the next commands:
  `.cli-git-worktree-copy-<id>`",
  and it fails the same way with this branch's four production files restored
  to `baf3d03fe6`,
  which is the measurement that it predates this step.
  The folder is `worktree_copying/`,
  which another agent is changing.
- `native:perf`,
  five arms between 07:50 and 08:00 with this tree's binary and main's at
  `baf3d03fe6`,
  alternating and with two arms of one binary as the control:
  the same binary's commit medians ran 57.6 to 853.9 ms (14.8 times) and its
  signed medians 57.3 to 404.9 ms,
  which covers every difference between the two binaries;
  forwarded commands stayed 1.4 to 3.6 ms added;
  both binaries broke the 5,000 ms commit bar under a load average of 53 to 121
  on six CPUs.
  So no arm attributes anything to this step,
  and none of its files runs in a commit.
  The plan's section for this step has every arm.

## Step T12, the awaited push chain's four faces, and the merge's second pass, 2026-11-09T15:46:16-05:00

- Branch:
  `refactor/git-policy-cli-landing-round-awaits-its-push-chain`,
  not merged into main.
- Done:
  the awaited maintenance (the whole of the second thing the round needs),
  and the awaited query,
  the awaited push destination and the awaited Git LFS
  step (the reconcilable part of the first).
  Each has a face whose waits are awaited beside the synchronous one,
  and a control comparing the two.
  The four commits and their controls are in the plan's section
  "Step T12,
  the landing round awaits its push chain,
  and the merge's second pass is what stops it".
- The next agent's first decision is not code:
  the awaited push chain cannot end without the reconcile,
  and the reconcile resolves a conflicted merge through `merge/`,
  whose only seam to Git is a synchronous trait method
  (`GitRunner::run`,
  `merge/merge_second_pass_git_runner.rs:35`) taken by seven
  functions across 9,381 lines of source.
  The plan records the two ways out and what each costs the typed `git merge`
  path;
  the owner decides between them.
- Then,
  in order:
  the reconcile's awaited faces (`push/auto_push_reconcile*.rs`),
  `push_once_awaiting` and the two object-safe handler methods
  (`landing/service_work.rs`),
  the push handler's own synchronous reads
  (`read_config_snapshot`,
  `branch_exists`,
  `branch_is_gone_words`),
  the landing loop's claim renewal whose `Drop` waits for its task
  (`landing/service_claim_renewal_running.rs:275`),
  the claims and settles put on the blocking pool
  (`landing/service_work_drain_pending_claims.rs`),
  and last the round itself onto
  `run_ordered_awaiting_on_the_runtime`.
- Checks:
  the file selections of the touched folders are green,
  `native:clippy` is clean,
  and the package lint is clean on those folders;
  the whole `native:test:host` run and its failures are in the plan.

## Step T13, not started, 2026-11-08T18:40:00-05:00

Nothing of T13 is done and nothing of it can be done first:
its three source guards fail today
(35 files start threads through `foundation/scoped_start.rs`,
`foundation/side_by_side_threads.rs` and their callers,
and the remaining sleeps are listed in the plan,
"What T11 did not convert,
and why"),
and its removals (the hand wire format `service/systemd_dbus_wire_format.rs` and the fuzz
target `dbus_reply`,
the thread helpers,
the poll constants) wait for their last caller to go.
The `format:monochromatic-lint` fixer and the package lint are clean on this branch,
so the tree is ready for whoever takes the guards once the call sites are converted
("Step T12,
the leaf-first asynchronous Git and the awaited runner",
"Exact next steps").

## Rules that bit during this work

- A `mise run` task never runs from inside `package/git-policy/cli` for Git commands:
  the require-root policy refuses;
  use `git -C <worktree>`.
- A commit through the wrapper can queue behind other sessions' commits for minutes;
  do not start work that needs the commit to have landed.
