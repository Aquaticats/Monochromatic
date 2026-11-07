# Handover: moving the Git wrapper onto tokio

## Current state

- Written:
  2026-11-07T13:30:00-05:00.
- Branch and worktree:
  `feat/git-policy-cli-on-tokio` in `.claude/worktrees/wrapper-on-tokio`.
  Main was last merged into it at `6c0ff76dc9` (the coordinator merged the branch at `48611f7e09`); the coordinator may merge gated checkpoints.
- Plan and the owner's answers:
  `doc/planning/git-policy-cli-full-tokio.md`, sections "The owner's answers at 2026-11-07" and "Progress on the branch".
- Landed on the branch: the guard that a forwarded command makes no thread (T2), the budget, limits, plan, lazy runtime, blocking-job
  helper and test-only probe (T3), the digest, time stamp and memory-headroom swaps, the pidfd child helper and the captured child run,
  and its first two callers (T4, partly); then, by the second agent: the `decide` race closed (`4d372fdd80` red, `7f39a05469` green) and
  step T5, the child watcher as a task (`fa518c33a1`, `7e4dc3aefc`, `c5b4559b88`). Details and gate counts: the plan, "The `decide` race closed"
  and "Step T5".
- Vet report for safe file saving:
  `doc/audit/tech-safe-file-saving-published-crates-vet-2026-11-07.md` (recommended, not adopted).

## How to run the gates

- Host lib tests: `mise run //package/git-policy/cli:native:test:host -- <filter>`.
- Host binary tests: `mise run //package/git-policy/cli:native:test:binary:host -- <filter>`.
- Clippy: `mise run //package/git-policy/cli:native:clippy`; the package lint: `mise run //package/git-policy/cli:lint`.
- Container gate, in its own unit, one `mise run` per call:
  `systemd-run --user --unit=<name> --collect --property=MemoryMax=40G --property=CPUQuota=2400% --property=TasksMax=4096
  --property=IOWeight=10 --working-directory=<worktree> --setenv=PATH="$PATH" bash -c 'mise run
  //package/git-policy/cli:native:test:container > <log> 2>&1'`.
  A cold build in the container takes about half an hour at its 2 CPUs; the tests take about ten minutes; Clippy about four.
- Latency: `mise run //package/git-policy/cli:native:perf -- --output <json>`; run it twice on the same binary for the spread.
- A dependency change: edit the use site, `mise run sync:files`, `mise run //package/git-policy/cli:native:lock:members`.
  When the shadow cannot download a crate offline, complete the lock with the toolchain's own `cargo fetch` first, then run the
  `lock:members` task so the release-age and trust policy checks the new versions.

## Open work, in order

- Done since the previous handover (details and figures: the plan, "The coordinator's three settlements after T5", "The two gate failures
  of main at `85e6d2ab99`", "Step T8" and "Step T9"): the T5 one-shot form on runtime pipes that keep all output, the futex wait of the
  name claim, the measured `HELD_NAME_WAIT` (60.3 s, because a hold of 6.03 s was seen) and `WATCHER_END_BOUND` (stop grace plus one
  second), the test-helper fix for the `DirectoryNotEmpty` gate failure, the measured cause of the left ref lock, T8 (`zbus` 6) and T9
  (`hyper`, `ureq` gone).
- Not started, in the plan's order: T6 (`start_retry`, `try_wait` loops, `sweep_after_exit`), T7 (the signal relay on a pidfd with the
  registry of children), T10 (the service), T11 (the callers become `async`, the synchronous bridges go), T12 (the parallel runner),
  T13 (removal of what nothing uses: the hand wire format `service/systemd_dbus_wire_format.rs` and its fuzz target, the thread helpers,
  the poll constants; the source guards; `native:perf` with a same-binary control; the mutation campaign). T6 to T12 convert code whose
  callers are synchronous, so they are in practice T11's leaf-first conversion.
- Left on purpose by the settlements: the `LeaveRunning` waiter thread and the `Unbounded` blocking `wait` of `output_watched_within`
  (a reaper that does not end the child when dropped is needed first).
- Open for the coordinator and the owner: the exposure that `STOP_GRACE` (5 s) is below the longest measured disk stall (6.03 s), so a `SIGKILL`
  at the end of a grace can cut Git's cleanup between its two unlinks and leave a ref lock (plan, "The two gate failures"); the factor
  ten of `HELD_NAME_WAIT`; the index-lock fallback belongs to another agent.
- The mutation campaign is the coordinator's. Files changed since the last list: `src/native/process/step_output_read_whole_through_pipes.rs`,
  `src/native/push/child_watch.rs`, `src/native/foundation/name_held_while_one_thread_publishes_it.rs`, `src/native/foundation/private_storage.rs`,
  `src/native/service/systemd_dbus.rs`, `src/native/standards_review/standards_review_client.rs` (the test helper
  `src/native/testing/test_support.rs` is not mutated code).

## Rules that bit during this work

- A `mise run` task never runs from inside `package/git-policy/cli` for Git commands: the require-root policy refuses; use `git -C <worktree>`.
- A commit through the wrapper can queue behind other sessions' commits for minutes; do not start work that needs the commit to have landed.
