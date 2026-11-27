# What `cargo-shadow` could do about a scope unit a killed shadow leaves

**Unapproved and non-final.**
This is a proposal for the project owner to decide from.
It was written by an agent in the Pi harness on 2026-11-27,
in the worktree of the branch
`fix/cargo-shadow-cli-issue-6578-scope-that-no-retire-follows-is-found-and-closed`.
It answers the fourth item of [issue 6578][issue],
changes no code,
no test and no SPEC for it,
and declares nothing complete:
the owner declares completion.
Each claim below says what it rests on:
the source of `package/cargo-shadow/cli` as the branch stands,
a real invocation of the installed `cargo-shadow` 0.1.5-alpha.12 on this host on 2026-11-27,
or `doc/troubleshooting/systemd-scope-adopting-an-exiting-pid-stays-running-empty.md`
(its census and its measured workarounds).

## The state this is about

A transient scope stays `active (running)` with an empty control group when the manager attaches
a process that is already exiting:
`cgroup.procs` accepts the write without moving anything,
the control group never becomes populated,
and the kernel therefore never sends the empty notification that ends a scope.
The root cause,
the reproduction and the manager-side prototype are the troubleshooting doc's.

Since 2026-11-27 a live run ends its unit itself,
after the kernel's own last `populated` line says the group holds nothing
(`retire_unit`,
called from `stop_everything` in `src/bounded_run_end.rs`).
That step is reached by every path a living wrapper controls:
a normal end,
a bound that passes,
a refusal after the program started,
and a launcher that is gone.

A wrapper killed by a signal reaches none of them.
No thread of it is left to ask the manager anything.
What remains is the unit,
and the question is what clears it.

Measured here on 2026-11-27,
one run of the installed `cargo-shadow` per arm,
a stand-in `cargo` that sleeps in the install step,
a throwaway `HOME`,
and `SIGKILL` sent to the shadow's own process:

- the shadow alive:
  every unit of the run ends,
  and one of them logs `Stopped [systemd-run] ...`,
  which is the retire step (`keep/installed-shadow-arm-normal-end.log`).
- the shadow killed while `cargo update` was in its scope:
  the unit stays `LoadState=loaded`,
  `ActiveState=active`,
  `SubState=running`,
  `TasksCurrent=2`,
  `CollectMode=inactive-or-failed`,
  and its journal holds only `Started`.
  Nothing retires it,
  because nothing of the shadow is left.
  When the program inside it ended,
  the manager ended and collected the unit (`not-found`,
  no stop line):
  a group that was populated and then emptied needs no help
  (`keep/installed-shadow-arm-sigkill.log`).
- a wrapper killed while its launcher never became a member:
  the unit stays `active (running)` with `TasksCurrent=0`,
  `populated 0`,
  an empty `cgroup.procs`,
  and only `Started` in its journal.
  That is the state issue 6578 is about,
  and no process of it is left to end it
  (`keep/left-unit-from-the-controls-red-run.txt`).

## What clears one today

Nothing in `cargo-shadow` does.

- `retire_unit` has one caller,
  `stop_everything`,
  which only a live run reaches (`src/bounded_run_end.rs:176`).
  A search for every `systemctl` use in `src` finds no other stop of a `cargo-shadow-*` unit.
- a recovery reads a recorded scope with `query` (`src/control_group_records.rs`) and answers
  `Membership::Gone` for a loaded unit whose group is empty,
  so it settles the journal and never asks the manager about the unit.
- `lean` sweeps build folders,
  run records,
  containers and toolchains
  (`src/lean_service_units.rs`,
  `src/lean_run_records.rs`,
  `src/lean_removal.rs`).
  It never lists a `cargo-shadow-*.scope` unit.
  Its one manager call is `systemctl --user daemon-reload` at install time
  (`src/lean_service_units.rs:269`),
  which the troubleshooting doc records as ending every stuck scope;
  that is a side effect of installing the service,
  not a sweep,
  and it happens once.
  On this host the service exists and is `disabled` (measured 2026-11-27).
- a run starting up does not look at the units other runs left.

So the unit lives until a person stops it or the manager is reloaded.
The journal cannot say it exists:
it reads `Gone`.

## Options

### A. A sweep when a run starts

Before a run asks for its own scope,
list the `cargo-shadow-*.scope` units,
take those whose control group holds no member and whose unit has been active longer than a
grace,
and stop them with the same step `stop_everything` uses.

- Pros:
  covers a unit from any run,
  including a run whose journal is already settled and removed;
  one listing per invocation;
  the gate is the kernel's own `populated` line;
  one implementation,
  `retire_unit`,
  already covered by controls.
- Cons:
  a listing cannot tell this account's cargo-shadow units from another session's,
  because the name prefix is shared;
  the grace is a guess;
  every invocation pays a `systemctl list-units` (4 ms to 5 ms per stop was measured,
  a listing was not);
  stopping a live run's scope that is momentarily empty would kill a build,
  and only the grace makes that unlikely rather than impossible;
  the package's own tests plant such units and would be swept by their neighbours unless a test
  hook excludes them.

### B. A unit property that makes the manager collect an empty scope

Ask the launcher for a property that ends or collects a scope whose group is empty.

- Pros:
  no sweep,
  no listing,
  no guessing;
  the manager does the work.
- Cons:
  no such property exists today.
  `CollectMode=inactive-or-failed` is already passed by every run and frees a unit only once it is
  inactive or failed,
  which a stuck scope never becomes.
  `RuntimeMaxSec` does end a stuck scope,
  but it also ends a legitimate run at that bound,
  leaves it `failed` for the whole bound,
  and the bound would have to exceed any real build.
  The manager-side fix is the prototype patch in the troubleshooting doc,
  which is upstream's decision and not a property a caller can set.
- Verdict:
  not available.

### C. Leave it, and document it

- Pros:
  no code,
  no new risk to a live run;
  each unit holds no process and costs the manager little;
  measured rarity is 12 of 11360 starts in one census,
  and two of the two runs this round made through the test that caused them.
- Cons:
  they accumulate (the `wg-quicker-exempt` watcher left 3696 in one night),
  nothing tells a person they exist,
  and the journal says only `Started`.

### D. A recovery stops the unit its journal recorded

`journal_recover` already asks `query` about the scope a journal records.
Where the answer is `Membership::Gone`,
also ask the manager to stop the unit the record's path names,
through `retire_unit`,
with the same gate and bound.

- Pros:
  no listing and no guessing,
  because the unit name is the run's own record;
  the gate is the same kernel line the end of a run uses;
  nothing of another run's can be stopped;
  it fires at the next run in that repository,
  which is when a leftover matters,
  and the journal is still there then because a recovery settles it after reading it.
- Cons:
  a recovery would ask the manager,
  which needs a route detection it does not do today (one `systemctl` question);
  it covers only runs that left a journal,
  so a unit whose journal was settled before the change stays;
  the recovery grows by one stop,
  which measured 4 ms to 5 ms for an empty scope.

### E. A sweep in the lean service

Add "stop loaded `cargo-shadow-*.scope` units with no member" to the pass the service already
runs.

- Pros:
  no cost on a run's own path;
  one place for housekeeping;
  it repeats every `PAUSE_BETWEEN_PASSES_SECONDS`.
- Cons:
  it shares option A's identification problem and its grace,
  at a scale that repeats on every pass;
  the service is optional and `disabled` on this host,
  so it fixes nothing where it is not installed;
  a pass that stops another session's momentarily empty scope is the same hazard as A,
  once per pass.

## Ranking

**D > A > E > C > B.**

- D above A:
  D names the unit from the run's own record and cannot touch another run's scope;
  A's one advantage is covering a unit whose journal is gone,
  which needs the journal to have been settled and removed first,
  and so is the rarer state.
- A above E:
  A runs wherever a leftover is made,
  whether or not the service is installed;
  E depends on a service this host has disabled.
- E above C:
  both leave the leftover to something that is not a run,
  and E clears it without a person noticing while C does not.
- C above B:
  C is a decision the owner can take today;
  B is unavailable,
  and its `RuntimeMaxSec` variant would end legitimate builds.

## Questions for the owner

1. Which option,
   if any?
   If a sweep is chosen (A or E),
   is stopping another session's momentarily empty scope an acceptable risk under a grace,
   or does the account need a marker in the unit name instead?
2. If D:
   may a recovery ask the manager to stop a unit,
   which is a write outside the repository and outside the journal?
3. If nothing is chosen:
   should the state be documented in the package's README where a person meeting such a unit
   would look?
4. The manager-side prototype in the troubleshooting doc is a human's filing decision,
   not an agent's;
   does the owner want the fact sheet kept for that?

## What was measured

Kept beside this round's report,
under `/home/user/temp/agent/issue-6578-scope-that-no-retire-follows-is-found-and-closed-scratch/keep/`
(the folder is not a repository record and expires a day after its last change):

- `installed-shadow-arm-normal-end.log`:
  the shadow alive,
  one `cargo check` through a stand-in `cargo`;
  three scopes were seen while it ran and all were `not-found` after it,
  one of them logging `Stopped`.
- `installed-shadow-arm-sigkill.log` and the manager's answer after it:
  the shadow killed inside `cargo update`;
  its scope stayed `active (running)` with `TasksCurrent=2` and only `Started`,
  and became `not-found` with no stop line once the program ended.
- `left-unit-from-the-controls-red-run.txt`:
  the unit a killed-launcher test left,
  `active (running)`,
  `TasksCurrent=0`,
  `populated 0`,
  an empty `cgroup.procs`,
  and one journal line,
  `Started [systemd-run] /usr/bin/true.`
- `probe-one-run-of-the-installed-shadow.sh`:
  the probe both the first two arms ran,
  with its stand-in `cargo` beside it.

[issue]: https://github.com/Aquaticat/Monochromatic/issues/6578
