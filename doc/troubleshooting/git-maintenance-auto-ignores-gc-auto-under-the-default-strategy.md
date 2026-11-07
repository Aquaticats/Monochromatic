# Git 2.56.0 `maintenance run --auto` ignores `gc.auto` under the default strategy

With `gc.auto=1` and 3000 loose objects,
`git maintenance run --auto` packs nothing in Git 2.56.0,
and neither does the
automatic maintenance that `git commit` starts.
`git gc --auto` and `git maintenance run --auto --task=gc` pack them.
The commit service of `package/git-policy/cli` runs `git maintenance run --auto --quiet --no-detach` for its maintenance
work items (`src/native/maintenance_run.rs`),
so a control that sets only `gc.auto` sees the filler stay loose although the
item ran.

## Reproduction

In a scratch repository with `gc.auto=1`,
`maintenance.autoDetach=false` and 3000 one-line files committed with
`-c gc.auto=0`:

- `git maintenance is-needed --auto` exits 1,
  and `git maintenance run --auto --no-detach` starts no child (Trace2
  shows no `child_start`);
  `git count-objects -v` still says `count: 3003`.
- `git maintenance is-needed --auto --task=gc` exits 0,
  and `git maintenance run --auto --task=gc --no-detach` leaves
  `count: 0`.
- `git config maintenance.gc.enabled true` makes the plain `--auto` run pack them too.
- `git config maintenance.geometric-repack.auto 1` makes the plain `--auto` run pack them with the geometric task.

## Cause

`initialize_task_config` in `builtin/gc.c` at tag `v2.56.0` chooses `geometric_strategy` for an unscheduled run when
neither `--task` nor `maintenance.strategy` is given.
That strategy lists exactly `commit-graph`,
`geometric-repack`,
`pack-refs`,
`rerere-gc`,
`reflog-expire` and `worktree-prune`,
and not `gc`.
With `--auto` each listed task runs only
when its own condition holds.
The `gc` task has `gc_condition`;
the repack this strategy runs has
`geometric_repack_auto_condition`,
and the four probes above show that it follows `maintenance.geometric-repack.auto`
(default 6700 objects) and not `gc.auto`.
The manual page of this Git still says "By default,
only
maintenance.gc.enabled is true",
which the `initialize_task_config` code contradicts.

## Consequence for this repository

The service's maintenance item runs;
whether Git then packs anything is Git's decision from the threshold it reads.
A control that wants a repack must set `maintenance.geometric-repack.auto` (or `maintenance.gc.enabled`),
as
`the_maintenance_a_landing_hands_to_the_service_runs_once_the_work_settled` in `src/native/binary_commit_tests.rs` does.
The wrapper's own gate (`auto_maintenance_arguments`) still follows `maintenance.auto` and `gc.auto`,
so
`gc.auto=0` still turns the item into a no-op.
