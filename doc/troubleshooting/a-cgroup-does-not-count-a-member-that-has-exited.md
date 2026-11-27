# A cgroup does not count a member that has exited

`cargo-shadow`'s install step refused a command with
"a process started by cargo could not be stopped",
and the round that met it read the process from `ps` as a `git` that `cargo` had left as a zombie
(issue 6528,
2026-11-26,
host under heavy disk load).
A zombie holds no descriptor and can write nothing,
so if a stop that waits for `populated 0` counted one as a member that could not be stopped,
the refusal would be wrong for that case.

## What was measured

Run on this host on 2026-11-27,
each in a `systemd-run --user --scope --property=Delegate=yes` scope of its own:
a process forked a child,
the child exited,
and the parent never waited for it:

- The scope's own `cgroup.procs` listed the parent only;
  the zombie's id was absent,
  and the zombie's `/proc/<pid>/stat` still read `Z`.
- A nested group below the scope
  (`mkdir` in the scope's directory,
  the zombie moved into it,
  so it was the group's only task)
  read `populated 0` in its `cgroup.events`,
  `nr_subsys_pids 0` in its `cgroup.stat`,
  and no line in its `cgroup.procs`,
  although `/proc/<pid>/stat` still read `Z`.
- The scope above that nested group read `populated 1` while any live process stood anywhere in its
  subtree:
  `populated` counts populated descendant groups too,
  so a group whose own `cgroup.procs` is empty can still read `populated 1`.
- `sh -c 'sleep 0.2 & sleep 600'` reaped the background `sleep` before it could be observed as a zombie,
  so a shell cannot be the fixture for this case;
  a program that forks and never waits can.

## The rule

Read `populated 0` from `cgroup.events` as "no thread of this group or its descendants is left":
a process whose last thread has ended is gone from the kernel's count at once,
whether or not some process has reaped it yet.
So a stop that waits for `populated 0` never waits for,
or refuses on,
a member that has only exited,
and a zombie seen in `ps` is not the cause of such a refusal.

`cgroup.procs` is the right place to look for process ids,
and it lists a thread group whose leader exited only while another of its threads lives
(so that member can still write).
A group with an empty `cgroup.procs` and `populated 1` holds its tasks in a descendant group,
and `cgroup.kill` reaches them there.

## What the refusal says now

So that a reader does not have to measure this again,
`cargo-shadow`'s refusal carries the stop's own report:
how long it waited for `populated 0`,
whether the kernel still reported the group populated,
and every member it listed with the state of its threads
(alive,
a leader that exited behind a live thread,
exited and waiting to be reaped,
gone,
or unreadable).

The listing reads the scope's own group and every group below it,
because `populated` counts a descendant group's threads too:
a scope whose own `cgroup.procs` is empty can still hold a live process below it,
and the refusal names that process with the group it stands in (`1234 in nested`).
Each member's descriptor is opened before its threads are read,
so from that open onwards the number cannot name another process,
though the number was read from the list a moment earlier:
a number freed and reused inside that window would be named with the threads of whichever process holds it now,
and the stop's signal pass closes that window by listing the scope again after it opens its descriptors,
because a signal must name the process it acts on.
A `/proc/<pid>` entry that is missing is told from a hidden one by the pidfd:
readable means the process ended,
not readable means a mount option or another pid namespace hides it,
and only the first is read as gone.
The listing has a bound of its own (`LISTING_BOUND` in `src/control_group_stop.rs`), and it is looked at between reads,
at the top of each group and before each member of it, so a refusal names what it read and says when the bound cut the
rest short,
never that a member it did not reach can write nothing;
one read of the walk has no bound of its own.
The measurement beside the constant is
`control_group_stop::measure_tests::the_listing_of_a_scope_with_this_many_threads_takes`, ignored by default:
one member of `N` sleeping threads in a delegated scope, 37.7 microseconds with no thread, 11.6 milliseconds with 1000
and 29.4 with 3000 (medians of three rounds, measured 2026-11-27).

A refusal whose listing could not be read says so and keeps the members it read before the failure;
an empty listing is worded as an empty listing;
a scope whose directory was gone is worded as gone, not as a kernel that reported it empty, and says that its members
were not read and that the manager removes the directory of a scope its last member has left;
and the process-group reasons state the bound that was waited, or that no stop was tried.

The controls are `control_group_stop::report_tests` (in `src/control_group_stop_report_tests.rs`),
`bounded_run_containment::refusal_tests` (in `src/bounded_run_containment_refusal_tests.rs`),
`install_step_tests::a_refusal_of_a_run_left_with_a_live_member_says_how_long_the_stop_waited_and_names_it`,
and `control_group_stop::leftover_tests` (in `src/control_group_stop_leftover_tests.rs`),
which reaches the decision `refusal_of_scope` makes for a member that ended after the listing read it,
for a listing cut before it read one member and for one cut after it read the first,
and for a group below the scope that could not be read.
The first three carry a failpoint of their own, because neither outcome can be held open from outside: the kernel takes a
process out of `cgroup.procs` and out of `populated` at its last thread's end, so a member can only be read as one that has
ended in the microseconds between the read of the list and the read of its threads,
and the listing's bound is reached with a clock the control arms rather than with a scope of thousands of threads.
The group that cannot be read needs no failpoint: a mode of `000` on it makes the read answer `EACCES`.
