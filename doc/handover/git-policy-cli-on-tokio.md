# Handover: moving the Git wrapper onto tokio

## Current state

- Written:
  2026-11-07T07:05:00-05:00.
- Branch and worktree:
  `feat/git-policy-cli-on-tokio` in `.claude/worktrees/wrapper-on-tokio`.
  Not merged into main until the coordinator says the wrapper's release is out; keep it current by merging main into it.
- Plan and the owner's answers:
  `doc/planning/git-policy-cli-full-tokio.md`, sections "The owner's answers at 2026-11-07" and "Progress on the branch".
- Landed on the branch: the guard that a forwarded command makes no thread (T2), the budget, limits, plan, lazy runtime, blocking-job
  helper and test-only probe (T3), the digest, time stamp and memory-headroom swaps, the pidfd child helper and the captured child run,
  and its first two callers (T4, partly).
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

- T4 remainder: the descriptor-limit sweeps for the child helper; nothing else on Linux.
- T5 to T13 as listed in the plan, with the owner's answers applied (task-local state instead of a `LocalSet`, the registry of
  children for the relay, no inline fallback when no thread starts).
- The replacement of `foundation/private_storage.rs` by the crates of the vet report; it waits for the owner's answer on item G1 (facts in the vet report, section "Facts for the owner's answer on G1"); today's checked rename stays until then.
- `zbus` 6 for the service client (step T8): main's `toolchain.lock.json` now names `nightly-2026-11-07`, so it compiles; the catalog
  floor is 5.19 and the release-age exclusion names `zbus@5.19.0`, both to be moved to 6.
- The mutation campaign (`native:mutation:scoped`) over the new files and the planted controls.

## Rules that bit during this work

- A `mise run` task never runs from inside `package/git-policy/cli` for Git commands: the require-root policy refuses; use `git -C <worktree>`.
- A commit through the wrapper can queue behind other sessions' commits for minutes; do not start work that needs the commit to have landed.
