# A control group's free memory counts the page cache its own build wrote, so a batch sized from it runs one item at a time

## Symptom

A test that asserts two items of a work directory run beside each other fails in a capped scope,
with one item inside its handler instead of two:

```text
thread 'landing::service_work_items_side_by_side::tests::two_worktrees_trash_records_run_beside_each_other' panicked at src/native/landing/service_work_items_side_by_side_tests.rs:327:5:
assertion `left == right` failed: the two worktrees' trashes were inside the handler at the same moment
  left: 1
 right: 2
```

Both items are settled and each ran to its end;
they simply never ran at the same moment.
The same failure appears in `two_branches_of_one_repository_are_pushed_at_once`
(issue 6567).

## Root cause

The width of a batch is the host's budget answer
(`worker_budget` in `package/git-policy/cli/src/native/foundation/parallel_budget.rs`),
and one of that answer's inputs is the memory free:

```text
by_memory = (memory_available / 2) / per_item_memory
```

`HostResources::read` takes `memory_available` as the smaller of the system's available memory
and the free bytes of this process's control group,
which the `sysinfo` crate answers as `memory.max - memory.current` at each level of the hierarchy and
then the least of those
(`sysinfo-0.40.3/src/unix/linux/cgroup.rs`,
`memory_limits`).

The page cache a process wrote while it lived inside the group is charged to that group's `memory.current`.
So a build run inside a memory-capped scope pushes the group's own cap,
`memory.current` reaches `memory.max`,
and `HostResources::read` answers **no free bytes at all** even though the machine has gigabytes free.
`worker_budget` then returns its floor of one item
(by design:
"a starved host still makes progress"),
so a round runs its batches one item at a time.

## What was measured

On 2026-11-27,
in
`/home/user/Monochromatic/.claude/worktrees/pi-issue-6567-side-by-side-work-item-controls-hold-threaded/package/git-policy/cli`,
each `cargo test --locked --offline --lib --features test-only-switches landing:: -- --skip guest --skip _arm_tests`
run in its own
`systemd-run --user --scope --slice=agent-work.slice -p MemoryMax=10G -p CPUQuota=700% -p TasksMax=8192 -p IOWeight=1`:

- The run whose scope also compiled the crate from scratch read
  `memory_available: Some(0)` with `parallelism: 7`,
  and the two controls above failed with `left: 1`.
- The same control alone,
  in the same caps,
  with a warm build,
  read
  `memory_available: Some(23564271616)` (23.5 GB) and passed with two items inside the handler.
- A fresh scope under the same caps answered
  `memory.max=10737418240 current=2486272`,
  so the cap itself is never the zero:
  the group's own charge is what fills it.

The crates are built into a directory every agent shares (`~/.cargo/build/<device>/<hash>`),
but the writes are made by the `cargo` process of the scope,
so the cache they leave is charged to the scope that built.

## What to do

A control that asserts two items run beside each other states the host its batch is sized by instead of
letting the machine answer:
`drain_pending_awaiting` and `drain_the_slow_work_awaiting` take a `&HostResources`,
the shipped round reads the machine once for the round,
and the controls use `A_HOST_THAT_BOUNDS_NO_BATCH` or `A_HOST_THAT_RUNS_ONE_ITEM_AT_A_TIME`
(`package/git-policy/cli/src/native/landing/service_work_items_side_by_side.rs`).
The rule is written down in the package guide,
"Nothing a test changes may be process-wide".

To tell this from a control that looked before its second item arrived,
print what the host answered where the width is taken:
a width of one beside `memory_available: Some(0)` is this trap,
and a width of two beside a count of one is the control's own window.
