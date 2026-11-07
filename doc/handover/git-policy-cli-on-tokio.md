# Handover: moving the Git wrapper onto tokio

## Current state

- Written:
  2026-11-07T09:58:00-05:00.
- Branch and worktree:
  `feat/git-policy-cli-on-tokio` in `.claude/worktrees/wrapper-on-tokio`.
  Main merged into it by fast-forward at `cb0b734efa` (the coordinator had merged the branch at `5e441f25d3`); the coordinator may merge gated checkpoints.
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

- T4 is done on Linux (descriptor-limit sweeps landed). T5 landed as a task behind an unchanged synchronous face; its one-shot
  pidfd form (`output_watched_within`, `EndAt` reads pipes and returns all output, the pidfd helper keeps only a tail) needs an owner-visible
  decision first; the plan says why.
- Next, in the order that needs no async callers: T9 (`hyper`) and the `zbus` 6 client of T8; T6 to T12 are the leaf-first conversion of the
  callers (T11's work in practice), see the plan, "Where the remaining steps stand after T5".
- The `HELD_NAME_WAIT` and `WATCHER_END_BOUND` values (five seconds each, the start retry's budget) have no measurement beside them; they
  are open to the owner's veto. The name claim is a bounded wait, not lock-free, which deviates from "add no lock" for the coordinator to accept.
- T5 to T13 as listed in the plan, with the owner's answers applied (task-local state instead of a `LocalSet`, the registry of
  children for the relay, no inline fallback when no thread starts).
- `foundation/private_storage.rs`: the exclusive publish is `tempfile::persist_noclobber` with the owner's checked-rename fallback (G1 answered, "Keep the fallback (Recommended)"); the module's other faces move to `tokio::fs` with the callers in T11. Two facts for the owner are in the plan ("Safe file saving: the publish replaced"): the in-process race of `decide` on a filesystem with neither, and the service's index-lock refusal there.
- `zbus` 6 for the service client (step T8): main's `toolchain.lock.json` now names `nightly-2026-11-07`, so it compiles; the catalog
  floor is 5.19 and the release-age exclusion names `zbus@5.19.0`, both to be moved to 6.
- The mutation campaign: the user told this agent not to run mutation campaigns and to delegate them to the coordinator. Scope for the coordinator: `src/native/foundation/name_held_while_one_thread_publishes_it.rs`, `src/native/foundation/lazy_runtime.rs`, `src/native/foundation/runtime_plan.rs`, `src/native/push/child_watch.rs`, `src/native/push/child_watch_clock_marks_and_stopping.rs`, `src/native/foundation/private_storage.rs`, `src/native/service/service_busy.rs`, `src/native/process/child_awaited_through_a_pidfd.rs`, `src/native/process/captured_child_run_to_its_end.rs` and the T3 files under `src/native/foundation/` (`runtime_limits.rs`, `runtime_plan.rs`, `lazy_runtime.rs`, `blocking_job.rs`), against a gate image of this branch. A scoped run for the first two was started and stopped at the planted-controls stage; it produced no mutant results.
- Not started in this session: T5 to T13. T5 (the watchers) follows the T4 pattern: the synchronous callers keep a bridge (`output_of_blocking` enters the lazy runtime and refuses from inside it) until their own callers become asynchronous at T11, as the plan's T4 text says.
- The owner's G1 answer was recorded at 2026-11-07T07:40:00-05:00 (the commit `1cff4935cd`).
- Main merged again at `ae740e9e6d` (toolchain lock, archive route, daily update): the only conflict was `package/git-policy/cli/Cargo.lock`; it was rebuilt from main's lock by the toolchain's `cargo fetch` (the offline index lacked the new gix releases) and `native:lock:members` (61 versions checked; `cpufeatures@0.3.1` set aside by the policy, 0.3.0 locked). Gate after that merge: see the plan, "Gates after the merges".
- Main merged in at `9156ca885b` (wrapper 0.6.0 released; `child_environment.rs` from the background-never-prompts change came with it). Container gate after that merge: 5127 tests run, 5127 passed (1 slow), 8 skipped; Clippy clean.

## Rules that bit during this work

- A `mise run` task never runs from inside `package/git-policy/cli` for Git commands: the require-root policy refuses; use `git -C <worktree>`.
- A commit through the wrapper can queue behind other sessions' commits for minutes; do not start work that needs the commit to have landed.
