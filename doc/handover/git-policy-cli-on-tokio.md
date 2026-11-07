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
- Not started:
  T10 (the service),
  T11 (callers),
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

## Rules that bit during this work

- A `mise run` task never runs from inside `package/git-policy/cli` for Git commands:
  the require-root policy refuses;
  use `git -C <worktree>`.
- A commit through the wrapper can queue behind other sessions' commits for minutes;
  do not start work that needs the commit to have landed.
