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
A second road to the same leftover was added on 2026-11-27
from the alpha.13 release read,
by an agent in the worktree of the branch
`docs/cargo-shadow-cli-cargo-shadow-alpha-13-release-pass-a-scope-whose-empty-notification-the-manager-lost`:
its section "The second road to the same leftover" is that round's,
and its claims rest on systemd 262.2's source,
a guest that ran that version,
and the released `cargo-shadow` 0.1.5-alpha.13 in that guest.
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

## The second road to the same leftover: the manager loses the empty notification

The state is that of the section "The state this is about",
reached by another road and with one reading that differs:
a transient scope whose control group held a process and then emptied,
which the manager never ended,
so the unit stays `ActiveState=active`,
`SubState=running`,
with `populated 0`,
an empty `cgroup.procs`,
`pids.current` 0 and `pids.peak` at least 1,
and the run that made it said nothing.
Here the run ended normally and no thread of it is left;
what failed is the notification the design rests on.

### The claim the step rests on

`cargo-shadow` 0.1.5-alpha.13 asks the manager nothing about a scope whose group held a process:
`control_group_retire.rs:213` to `214` reads `pids.peak`,
and `retire` answers `None` for `Emptiness::EmptiedAfterAProcess` at line 264.
The sentence that carries it is the package's own README:

> A group that held one changed its `populated` line,
> which is the change the manager watches and ends the unit on,
> so that run asks the manager nothing

(`README.md` lines 854 to 858),
and the troubleshooting file says the same
(`doc/troubleshooting/systemd-scope-adopting-an-exiting-pid-stays-running-empty.md` lines 916 to 921).
systemd's source states the reliance in one line,
`/* Unwatch all pids we've just added to cgroup. We rely on empty notifications there. */`
(`src/core/scope.c:417`,
in `scope_enter_running`,
which sets `SCOPE_RUNNING` at line 412).

### The three paths in systemd 262.2's source

The version read here is the one the guest ran,
`systemd 262 (262.2-1-arch)`,
so the review's line numbers and these are the same file at the same tag.
Each of the three was read in `src/core/cgroup.c` or `src/core/manager.c` at `v262.2`.

#### A queue overflow is skipped, and nothing rescans the groups it hid

```c
/* src/core/cgroup.c:3234-3236 at v262.2, in on_cgroup_inotify_event */
if (e->wd < 0)
        /* Queue overflow has no watch descriptor */
        continue;
```

The reading holds:
the kernel's own overflow marker is dropped without a look at any group,
so a scope that emptied inside the lost range is never queued.
Only read,
not shown:
filling `fs.inotify.max_queued_events` (16384 in the guest) needs a burst of events,
and no such burst was generated for it.

#### A watch that cannot be added is ignored

```c
/* src/core/cgroup.c:2038-2045 at v262.2, in unit_watch_cgroup */
crt->cgroup_control_inotify_wd = inotify_add_watch(u->manager->cgroup_inotify_fd, events, IN_MODIFY);
if (crt->cgroup_control_inotify_wd < 0) {

        if (errno == ENOENT) /* If the directory is already gone we don't need to track it, so this
                              * is not an error */
                return 0;

        return log_unit_error_errno(u, errno, "Failed to add control inotify watch descriptor for control group %s: %m", empty_to_root(crt->cgroup_path));
}
```

and the call that discards the answer,
in `unit_update_cgroup`:

```c
/* src/core/cgroup.c:2161-2162 at v262.2 */
/* Start watching it */
(void) unit_watch_cgroup(u);
(void) unit_watch_cgroup_memory(u);
```

The reading holds:
the failed watch is logged once and the answer is thrown away,
so the unit runs with no notification at all.
Shown in the guest,
under "What was shown in the guest".

#### A scope that empties between a re-execution's one read and the watch it re-adds

```c
/* src/core/cgroup.c:4020-4032 at v262.2 */
void unit_cgroup_catchup(Unit *u) {
        assert(u);

        if (!UNIT_HAS_CGROUP_CONTEXT(u))
                return;

        /* We dropped the inotify watch during reexec/reload, so we need to
         * check these as they may have changed.
         * Note that (currently) the kernel doesn't actually update cgroup
         * file modification times, so we can't just serialize and then check
         * the mtime for file(s) we are interested in. */
        (void) unit_check_cgroup_events(u);
        unit_add_to_cgroup_oom_queue(u);
}
```

`manager_catchup` calls that for every unit (`src/core/manager.c:1905`,
called from `manager_ready` at line 2128),
and the watch is added later,
when the realize queue is dispatched and `unit_update_cgroup` reaches line 2161.
A deserialized unit with a group is invalidated and queued for realization
(`src/core/unit-serialize.c:378`).
The only two readers of `cgroup.events` are the event handler and this catch-up,
both at `(void) unit_check_cgroup_events(u)`,
so a scope that empties after the catch-up read and before the watch is re-added is never queued.
The reading holds,
and it was shown in the guest by racing the two.

### What was shown in the guest

One `mvm` guest of the `cachyos-7.3.5-1-zfs-2.4.4` template:
kernel `7.3.5-1-cachyos`,
`systemd 262 (262.2-1-arch)`,
cgroup v2,
4 vCPUs,
`MemTotal` 8,121,404 kB and no swap,
root's own user manager started with `systemctl start user@0.service`,
`fs.inotify.max_user_watches` 524288,
`max_user_instances` 1024,
`max_queued_events` 16384,
and 82 watches held by root when the failed-watch arm began.
Every launcher command in the arms is byte for byte the shape
`cargo-shadow` builds (`control_group.rs:303` to `317`):

```sh
systemd-run --user --scope --quiet --collect --property=Delegate=yes \
  --unit=cargo-shadow-<token>.scope --slice=app.slice -- <program>
```

#### The control arms, with the watch quota untouched

`/usr/bin/true` as the program:
the launcher answered 0,
and the immediate look read `LoadState=loaded`,
`ActiveState=active`,
`SubState=running`,
`CollectMode=inactive-or-failed`,
`TasksCurrent=0`,
which is exactly the state the retire step judges;
one second later the same look read `not-found`,
and the group directory was gone.

`/bin/sleep 0.4` as the program,
looked at while it ran:
`populated 1`,
`pids_peak=1`,
one line in `cgroup.procs`;
after it ended,
`not-found` and the directory gone.
So the ordinary path is created,
populated,
ended and collected,
as the design says.

#### The failed watch, shown

`sysctl --write fs.inotify.max_user_watches=16` in the guest,
with 82 watches already held by the user,
then the same launcher with `/usr/bin/true`.
The manager logged this,
once per watch:

```text
cargo-shadow-lost-notification.scope: Failed to add control inotify watch descriptor for control group /user.slice/user-0.slice/user@0.service/app.slice/cargo-shadow-lost-notification.scope: No space left on device
cargo-shadow-lost-notification.scope: Failed to add memory inotify watch descriptor for control group /user.slice/user-0.slice/user@0.service/app.slice/cargo-shadow-lost-notification.scope: No space left on device
```

One second after the program ended,
and again two seconds after the quota had been restored:

```text
MARK look=1s-after-the-program-ended
loaded
active
running
Fri 2026-11-27 17:07:59 UTC
inactive-or-failed
success
0
MARK cgroup_events=populated 0 frozen 0
MARK pids_peak=1
MARK pids_current=0
MARK cgroup_procs_count=0
```

The eight values are `LoadState`,
`ActiveState`,
`SubState`,
`ActiveEnterTimestamp`,
`InactiveEnterTimestamp` (empty),
`CollectMode`,
`Result` and `TasksCurrent`,
in the order `systemctl` prints them.
The last look read `loaded`,
`active`,
`running`,
`0` again:
two seconds after the quota was restored the unit was still there.
What the guest shows is that restoring it changed nothing;
whether the manager ever re-adds that watch is not something these readings look at.

#### The released wrapper, in the same state

The same guest,
the linux-x64 asset of `cargo-shadow 0.1.5-alpha.13` linked where `cargo` is on `PATH`,
a stand-in `cargo` behind it that logs its arguments and its own control group,
and the fixture whose stale `Cargo.lock` makes one call start four scopes,
run from `package/x/y` of that fixture with the quota at 16:

```text
MARK wrapper_exit=0
MARK standin_cargo_log
args=update --workspace --locked --offline group=/user.slice/user-0.slice/user@0.service/app.slice/cargo-shadow-1cd23325725149a988b687f3f5c92c06.scope pids.peak=3 populated 1
args=update --workspace --offline group=/user.slice/user-0.slice/user@0.service/app.slice/cargo-shadow-a0d6c7c09bf0689e7a85ea8432f51b22.scope pids.peak=3 populated 1
args=metadata --locked --offline --format-version 1 group=/user.slice/user-0.slice/user@0.service/app.slice/cargo-shadow-c98f9a30f7fcb9b6c898ce5bea1b9a85.scope pids.peak=3 populated 1
args=metadata --locked --offline --format-version 1 group=/user.slice/user-0.slice/user@0.service/app.slice/cargo-shadow-433f579a90df7313893fc69b4401411a.scope pids.peak=3 populated 1
MARK units_of_this_run
  cargo-shadow-1cd23325725149a988b687f3f5c92c06.scope loaded active running [systemd-run] /root/wrapper-arm/bin/cargo update --workspace --locked --offline
  cargo-shadow-433f579a90df7313893fc69b4401411a.scope loaded active running [systemd-run] /root/wrapper-arm/bin/cargo metadata --locked --offline --format-version 1
  cargo-shadow-a0d6c7c09bf0689e7a85ea8432f51b22.scope loaded active running [systemd-run] /root/wrapper-arm/bin/cargo update --workspace --offline
  cargo-shadow-c98f9a30f7fcb9b6c898ce5bea1b9a85.scope loaded active running [systemd-run] /root/wrapper-arm/bin/cargo metadata --locked --offline --format-version 1
MARK loaded_count=4
```

Each of the four read `state=loaded active running 0`,
`cgroup_events=populated 0 frozen 0`,
`pids_peak=3` and an empty `cgroup.procs`,
and each had the manager's `Failed to add control inotify watch descriptor` line.
The wrapper's own output was its install-step lines and the toolchain warning;
it said nothing about the four units,
and its exit status was 0,
which is what the Medium finding says and what the gate's `pids.peak` branch predicts.

#### The re-execution, which cuts both ways

A leftover is collected by a re-execution,
because the catch-up reads `cgroup.events` for every unit once:
one leftover,
`systemctl --user daemon-reexec`,
and the same unit read `not-found` two seconds later.

The same read is the window of the third path.
Forty scopes were started at once,
each program waiting for an absolute instant so the launcher's own start cost could not move it,
on a 4 ms grid,
with one `daemon-reexec` fired into the middle:

```text
MARK reexec_measured_milliseconds=109
MARK fire_at_ms=309
MARK grid_from_ms=349
MARK grid_to_ms=505
MARK reexec_fires_at_ms=314
MARK reexec_returns_at_ms=489
MARK leftovers_after_the_attempt=2
MARK unit=cargo-shadow-reexec-2-36.scope state=loaded active running 0
MARK unit=cargo-shadow-reexec-2-36.scope cgroup_events=populated 0 frozen 0
MARK unit=cargo-shadow-reexec-2-36.scope pids_peak=2
MARK unit=cargo-shadow-reexec-2-37.scope state=loaded active running 0
MARK unit=cargo-shadow-reexec-2-37.scope cgroup_events=populated 0 frozen 0
MARK unit=cargo-shadow-reexec-2-37.scope pids_peak=2
```

Those two programs ended at 489 ms and 493 ms on the grid,
and the re-execution returned at 489 ms:
every other unit of the forty was collected.
Two more attempts left nothing,
one of them because its re-execution returned after the last program had ended.
So the state is reachable through a re-execution,
and one attempt in three hit it.
The manager's own catch-up read and watch add were not instrumented,
so what those two units show is a result the window in the source explains,
and no other cause was found for it.

### What such a unit costs, and what removes it

Each reading is from the guest,
after the arm that made it.

- It holds no process and no accounted memory:
  `TasksCurrent=0`,
  `MemoryCurrent=0`,
  an empty `cgroup.procs`,
  `pids.current` 0.
  Its group directory holds the standard cgroup files and `du --bytes --summarize` reads 0,
  which is cgroupfs's answer for every file it holds.
- What it does cost is the manager's own memory and its unit table:
  twenty leftovers took the manager's loaded units from 91 to 111,
  and `MemoryCurrent` of `user@0.service` from 6,004,736 to 8,675,328 bytes,
  so 2,670,592 bytes more for twenty,
  about 130 KiB each.
  That is one reading before and one after,
  with no control arm and no run-to-run spread,
  so the difference is the manager's growth across the arm and not a measured price of one unit.
- No per-unit limit counts it:
  the pids controller counts a unit's processes and this unit has none,
  and the manager sets no limit on how many units it loads.
  What it adds to the manager counts,
  the way any of the manager's own memory does,
  against a limit written on `user@0.service` (`MemoryMax` was not read here).
- `systemctl --user stop <unit>` ends it:
  exit status 0 in 4 ms,
  then `LoadState=not-found` and the group directory gone.
  That is the same command `retire_unit` would have sent.
- The user manager stopping ends all of them:
  `systemctl stop user@0.service` left no group directory and no loaded unit,
  and after the restart the nineteen leftovers read 0.
  That is the manager's exit and not a session's logout:
  the guest had no session for root and no linger,
  so the manager was started and stopped directly,
  and a machine where root lingers or stays logged in keeps its manager and its units.
- A `daemon-reexec` ends them,
  as "The re-execution,
  which cuts both ways" shows.
- A reboot ends them:
  a leftover read `loaded active running 0` before,
  the guest's boot id changed from `577ecf20-bf27-4964-8baa-1b68d7610bbf` to `80e9661a-4762-438d-8dbd-a46295c9d0c9`,
  and after the reboot the count of `cargo-shadow-*` units was 0.

So a person meets it in `systemctl --user list-units` and in nothing else,
and it goes when the manager is stopped,
re-executed or rebooted.
Nothing here says how long one stays:
a unit was still there two seconds after its quota was restored,
and that is the whole of what the readings say about its life.

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

## The options and the ranking, with the second road

The options in the section "Options" were weighed against the killed-run road.
This part says what each one does with the second road,
and whether the order the section "Ranking" gives still holds.

### What each option does with this state

- A,
  a sweep when a run starts:
  clears it.
  The unit is a `cargo-shadow-*.scope` with no member,
  which is what A lists and what its gate reads from the kernel,
  and it does not need the journal,
  which matters here because this run's journal is gone.
  Its cons stand as written,
  and one of them grows:
  the grace now also has to be longer than a live run's own scopes,
  which a sweep that starts another run in the same second could otherwise catch.
- B,
  a unit property:
  does not clear it,
  for the reason already recorded:
  no property ends a scope that never becomes inactive,
  and `RuntimeMaxSec` would end legitimate builds.
- C,
  leave it and document it:
  applies as written,
  and this state makes the documenting half more valuable,
  because a person meeting one in `systemctl --user list-units` cannot tell it from the stuck scope
  of issue 6578 there,
  and the one reading that separates them is the `pids.peak` of the group:
  0 for a group that never held a process,
  at least 1 here.
- D,
  a recovery stops the unit its journal recorded:
  reaches the killed-run road and not a run that finished.
  The recovery judges the journals it reads as open
  (`listing.open` in `src/journal_recover.rs`),
  and a journal whose completion marker is published is in `listing.done` instead,
  where the next run only finishes and removes it (`src/journal_recover.rs:285` to `294`)
  without reading any record from it.
  A run that finishes therefore puts its journal out of D's reach even when the removal itself fails,
  which `journal_finish.rs` records as a warning rather than a stop,
  and the README's sentence puts the `settled <scope path>` line in exactly that journal
  (`README.md` lines 741 to 742).
  What D still covers from this road is the window between the retire step returning `None`
  (`control_group_retire.rs:264`) and the transaction's completion marker
  (`journal_transaction.rs:532`):
  a run killed there leaves an empty scope whose journal is still open,
  and D would reach it.
  It cannot reach the state this section is about,
  where the run finished and the wrapper said nothing.
- E,
  a sweep in the lean service:
  clears it,
  with A's identification and A's grace,
  and only where the service is installed and enabled,
  which it is not on this host.

### Whether the ranking changes

**D > A > E > C > B,
unchanged**,
and each adjacent pair for a stated reason:

- D above A:
  D names the unit from the run's own record and cannot touch another session's scope,
  and that is still the only option with that property.
  What this state changes is the size of A's advantage,
  not its kind:
  A reaches both roads where D reaches the killed-run one and the window before a run's
  completion marker,
  and the journal-gone state is not the accident "Ranking" called it,
  since any run that finishes leaves one.
  The reason D still ranks first is what A costs and what its extra reach buys:
  A lists on every invocation,
  and its grace is the only thing between it and another run's momentarily empty scope;
  what it alone adds is a state with two rare triggers,
  an account whose inotify watches are used up and a race with a re-execution that one attempt
  in three engineered here,
  and both roads' leftovers go on the next re-execution,
  logout or reboot,
  as "What such a unit costs,
  and what removes it" measured,
  so A buys an earlier clear only where no such event comes before its next invocation.
  That makes this pair closer than "Ranking" put it,
  and D keeps the earlier place on its precision,
  not on any claim about which road clears itself.
- A above E:
  unchanged and now the stronger pair of the two:
  both clear this state,
  and A does it wherever a run is made while E needs a service this host
  has disabled.
- E above C:
  unchanged;
  both leave the work to something that is not a run,
  and E clears it.
- C above B:
  unchanged;
  C is a decision the owner can take today and B is unavailable.

One addition is available and not adopted here,
because its price is the opposite of what alpha.13 bought:
the run is alive when it leaves this unit,
so its own last step could look at the unit it just started,
with the name already in its hand and no listing and no grace,
and stop it or name it when the kernel says the group emptied after holding a process.
That is D's step on the live path.
Its pros:
it needs no listing,
no grace,
and no journal,
and it is the only way to learn about this road while the wrapper that made it is still running.
Its cons:
the condition is true for every ordinary scoped run,
because at the last look the manager's notification may simply not have arrived yet,
so it restores a `systemctl` process start per scoped run,
which is the cost alpha.13 removed by returning `None` at `control_group_retire.rs:264`;
and a stop that follows such a look can kill a member that joins in the window between the look
and the stop,
silently,
which is the Low finding of the alpha.13 read.
It is question 5 of "Questions for the owner",
not a rank taken here.

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
5. The second road's leftover is made by a run that is still alive:
   after each program ends,
   the run reads that scope's `populated` line and its `pids.peak` anyway.
   Should its last step also ask the manager about the unit it just started?
   Pro:
   the name is in hand,
   and no listing,
   no grace and no journal are needed,
   so this is the only option that reaches this road before the run ends.
   Con:
   the condition is true of every ordinary scoped run,
   because the manager's notification may still be in flight at that look,
   so it brings back a `systemctl` process start per scoped run,
   which alpha.13 removed;
   and a stop after such a look can kill a member that joins in between,
   silently,
   which the alpha.13 read records as a Low finding.
   The same two sides are under "The options and the ranking,
   with the second road".
6. The package's README and the troubleshooting file state the notification as a law
   ("so that run asks the manager nothing").
   Should the sentence say the manager can lose it,
   in whichever round holds the crate's README,
   or is this document enough?

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
