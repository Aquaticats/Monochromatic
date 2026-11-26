# Step 22's guest half, measured on 2026-11-25

**Non-final draft by an agent,
unapproved.**
It records one round's guest measurements with their controls and spread,
and the values each would support.
Only the owner declares a value settled,
and no shipped constant changed in the round that wrote this.

The plan's step 22 asks for the mount and unmount budgets,
the sweep's cost and lost-event behaviour,
the add against ordinary Git stage by stage,
what expiration and removal free,
the forced arm's three waits,
and the snapshot-destroy request budget
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
"Step 22:
The measurements that size waits and retention").
The storage half stays the owner's act on the owner's authorisation,
and it is not measured here.

## The fixture and the wrapper under test

One disposable `mvm` libvirt guest,
Ubuntu 24.04 on the recipe's kernel,
with OpenZFS `2.4.4-1` built from source in the guest,
four vCPUs,
7.75 GiB of memory and a six-gibibyte swap file.
The guest was destroyed through `mvm` when the arm ended,
and its logs are kept in the round's `keep/guest/` folder.

The pool is `zfsstep22` on a two-gibibyte file.
Every number here is the guest's semantics on a file-backed pool,
never this host's storage.

The checkout is the dataset the step 19 design describes.
The arm makes it rather than migrating one,
because the owner's one-time migration has not run:
`zfsstep22/ROOT/cos/home/monochromatic-git-checkout` mounted at a folder of the arm's own,
with `monochromatic-git-worktree-clone-datasets` beside it as a legacy dataset,
and the nine permissions the design scopes granted to the account the arm runs as.

The workload is the wrapper's own source tree,
3,327 files and 43,729,083 bytes.
It is copied into the checkout and committed once,
so both arms fill the same tracked tree.
The tree's `uses/` folders are links between source folders rather than fill work,
and the copy leaves them out.

The wrapper is the branch tip built inside the guest from commit `7e6a51baad`'s tree,
by `cargo build --release --tests --locked` with one job.
Every arm starts that build's own executable:
the add as `git` through a link named `git`,
the sweep under the installed ZED history event's name,
and the holder scan as the root mode's words.

The add's five rounds alternate an ordinary `git worktree add` and the wrapper's clone add,
control first in each pair,
so each pair is one interleaved comparison.
Even rounds let the built sweep attach the clone,
which is what ZED's start does.
Odd rounds mount it at once with `mount -t zfs`,
which is the smallest wait the route can see,
so the two shapes bracket what a handler adds.

Every round of either arm starts from nothing.
Its slot,
clone dataset,
origin snapshot,
mount and records are taken away before the next round.

## The add against ordinary Git

Five interleaved pairs on the fixture's workload.

Ordinary Git's `git worktree add`,
in milliseconds:

- round 0 was 605,
- round 1 was 352,
- round 2 was 351,
- round 3 was 342,
- round 4 was 357.

The first round is the fixture's cold one.
The four after it span 15 milliseconds,
which is the run-to-run spread of unchanged input.

The wrapper's clone add,
in milliseconds:

- round 0 was 647,
- round 1 was 468,
- round 2 was 536,
- round 3 was 400,
- round 4 was 534.

Split by the attach shape,
the rounds the kernel mounted at once took 468 and 400 milliseconds,
and the rounds the built sweep attached took 647,
536 and 534.

The medians are 352 milliseconds for ordinary Git and 534 for the clone add.
The median of the five paired ratios is 133 per cent,
and the pairs range from 107 to 153 per cent.
The ratio of the two medians is 152 per cent,
which is not the same number,
because the median of five ratios and the ratio of two medians rank differently.

The binding record's phases time the stages inside the clone add,
in milliseconds from the command's start.

The claim and the slot were caught at 161 in round 0,
and the rounds after the first missed them,
because the record is rewritten per phase.
`registered` was reached at 185 then 161 then 161 then 140 then 141,
so Git's registration and the preflight take 140 to 185.
`snapshotted` was reached at 233 then 208 then 208 then 187 then 189,
so the checkout snapshot and its inventory take 47 to 48.
`cloned` was reached at 265 then 239 then 238 then 217 then 219,
so the clone and its inventory take 30 to 32.
`mounted` was reached at 484 then 285 then 395 then 259 then 373,
so the mount wait takes 42 to 46 with the kernel's own mount,
and 154 to 219 through the built sweep.
`filled` was reached at 647 then 447 then 516 then 380 then 514,
so the fill,
the pointer's exchange,
the proof and the copied administration's disposal take 121 to 163.

The disposal ordinary Git has no stage for is measured on a copy of the same administration the route discards,
3,366 files and 15,092,999 bytes.
In milliseconds:

- round 0 was 104,
- round 1 was 103,
- round 2 was 102,
- round 3 was 102,
- round 4 was 110.

The median is 103 milliseconds,
which is 29 per cent of ordinary Git's median total,
and it is the one contribution the control arm cannot have.

Ordinary Git's 352 milliseconds buys the registration and the checkout.
The clone add's 534 buys the registration at 140 to 185,
the snapshot at 47 to 48,
the clone at 30 to 32,
the mount wait at 42 to 219,
the fill at 121 to 163,
and the disposal at 103.
The clone and its snapshot are what the ordinary arm does not create.

## The mount request's wait

The kernel's own mount call took 13 and 12 milliseconds when the arm made it directly.
The built sweep attached the add's clone in 164,
127 and 124 milliseconds,
which includes the sweep's read of the clone's intent and the attach.

The add's own wait,
from the record's `cloned` to its `mounted`,
was 42 and 46 milliseconds when the arm mounted at
once,
and 154,
157 and 219 milliseconds when the sweep did.

ZED's delivery of one property event to a test zedlet was 10,218 to 10,581 microseconds,
and to the product's receipt 11,171 to 12,113.
Those are the three kept rounds of the receipt-path measurement in
`package/git-policy/cli/doc/zfs-mount-receipt-deadline-measurement-2026-11-21.md`.
That document is another round's,
and it is cited here for the event's share only.

The largest handler path this round measured is a full sweep over 64 clones at 853 milliseconds plus a mount call at
13.
The largest path the add actually waited through was 219 milliseconds.
A request that arrives while a parent's sweep runs waits for that sweep first.

## The unmount's wait and the holder scan

The kernel's unmount by path,
with no holder,
in milliseconds:

- round 0 was 9,
- round 1 was 7,
- round 2 was 6,
- round 3 was 6,
- round 4 was 7.

The median is 7 milliseconds.

The same unmount with a process whose working directory is inside the mount was refused in 1 millisecond in all five
rounds.
A busy unmount answers rather than waits,
so the deadline is not a retry bound.

The holder left after its signal in 2,
0,
0,
0 and 1 milliseconds,
median 0.
A holder that ignores the polite signal was still alive 500 milliseconds after it,
and left 1 millisecond after the kill.
That is why the forced arm needs both waits rather than one.

A transient unit left its cgroup in 15,
7,
7,
6 and 6 milliseconds after its manager was asked to stop it,
median 7.

Plan step 9's holder scan is the all-namespace scan an unmount request's handler makes.
It timed at 4 milliseconds over 282 live tasks and 7 over 481,
from the same guest and the same run,
as `MEASUREMENT holder-scan-ms=small=4 large=7 tasks-small=282 tasks-large=481` records.

## The sweep's cost and a lost event

One sweep over clones of the checkout's snapshot that all ask for their mount:

- one clone took 120 milliseconds,
- sixteen clones took 324 milliseconds and all sixteen attached,
- sixty-four clones took 853 milliseconds and all sixty-four attached.

Beyond the base,
a clone costs about 12 milliseconds.

A request written with no caller waiting and no trigger fired stood unmounted for a watched window of 5,000
milliseconds.
The next sweep attached it in 148 milliseconds.
The window is an observation and not a bound.
What it shows is that nothing happens until a trigger,
and that one sweep settles a pending request.

A bound of one sweep per period at a parent holding 64 clones would cost the handler about 853 milliseconds of work per
period.
That is the number a rate rule would be written against.

## What expiration, removal and the guard free

The arm read ZFS's own accounting on the design's datasets,
in bytes.

The checkout dataset's `used` was 39,523,840 before the clone,
and the clone's `used` was 1,024 after the attach.
Twelve mebibytes written into an ignored folder of the checkout did not raise `usedbydataset` above 39,523,840,
because the arm's output was one repeated byte and the pool stored it at almost no cost.
The hourly snapshot taken after that write pinned 38,912 bytes of `usedbysnapshots`.
Destroying the clone freed its 1,024 bytes,
and destroying the hourly freed the 38,912.

A compressible fixture therefore shows what a snapshot's own accounting costs,
and nothing about retained build output.
This round did not obtain the pinned-output number it set out to read.
A probe that writes incompressible bytes and overwrites them after the snapshot was prepared,
and the guest's agent stopped answering before it could run.

The guard's current scope is a code fact rather than a measurement.
`zfs-space-guard` prunes only `<dataset>@autosnap_*` of its configured dataset,
as `/usr/local/sbin/zfs-space-guard:7` and `:44` and
[the design](cli-git-zfs-worktree-design-proposal.md)
read it under "Snapshots and setup reporting".
A descendant's hourly and a clone's pinned origin are therefore outside its reach,
and nothing this round measured changes that reach.

## The forced arm's three waits

`T_wait` is the wait between `SIGTERM` and `SIGKILL`.
It was 0 to 2 milliseconds for a process that cooperates,
and unbounded for one that ignores the signal.

`T_h` is the handler's heartbeat period.
This round measured the work one period carries rather than the period itself,
which is the sweep at 120 to 853 milliseconds depending on the clones a parent holds.

`T_stop` is the wait for a locked unit to leave its cgroup.
It was 6 to 15 milliseconds.

The harness's own provisional values were `T_wait` 10 seconds,
`T_h` 1 second and `T_g` 4 seconds,
from "Tier 2" of
[the forced removal design](cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md).
They are the harness's,
not shipped.

## The snapshot-destroy request budget

One `ZFS_IOC_DESTROY_SNAPS` request through the binding carried every name of one dataset's snapshots,
and no `defer` member.

- 64 names were 3,652 bytes of encoded input,
  accepted in 117 milliseconds with none left;
- 512 names were 28,740 bytes,
  accepted in 2,198 milliseconds with none left;
- 2,048 names were 114,756 bytes,
  accepted in 22,112 milliseconds with none left.

A name costs about 56 bytes of encoded input.
The binding sizes a mutation's reply at `max(256 KiB, twice the input)`,
in `requests/reply.rs` at `initial_size`.
So 114,756 bytes of input asks for 229,512 bytes of reply,
which is far below the 128 MiB ceiling.
A per-name answer is one int32 per name,
which 2,048 names fit in easily.

## The values this measurement would support

Each proposed value carries its citation.
None is set,
and the constant in code stays as it is until the owner accepts one.

`PROVISIONAL_MOUNT_REQUEST_DEADLINE` in
`package/git-policy/cli/src/native/worktree_clone_mount_protocol/mount_request_wait.rs`
is 30,000 milliseconds now.
The proposal is 5,000 milliseconds.
The largest handler path measured is a 64-clone sweep at 853 plus a mount call at 13.
The add's own wait was 42 to 219 milliseconds,
and the event's share from the receipt-path record is 11 to 12.
So 5,000 is about 5.7 times the largest measured path,
and about 23 times the largest wait the add paid.

Step 9's unmount deadline has no constant yet.
The proposal is 5,000 milliseconds.
Its measured parts are the holder scan at 4 to 7,
the unmount at 6 to 9,
and the read-back.
A handler busy with a 64-clone sweep at 853 delays the request before any of them,
so the same 5,000 covers both halves with the same margin.

`TERMINATION_WAIT` in
`package/git-policy/cli/src/native/worktree_clone_forced_removal/forced_arm_numbers.rs`
has no value.
The proposal is 2,000 milliseconds.
A cooperating holder left in 0 to 2 milliseconds,
and a refusing one never leaves,
so the value is a ceiling on how long a polite exit is waited for.
Two thousand is about a thousand times the measured exit,
and a fifth of the harness's 10 seconds.

`HEARTBEAT_PERIOD` and its four-period give-up window have no values.
The proposal is 1,000 milliseconds and 4,000,
which keeps the harness's provisional pair.
A 64-clone sweep is 853 milliseconds,
so the four-period window is 4.7 times the largest sweep this round measured.
The handler still writes within a period while it works.
A parent that holds more clones than 64 would need the period raised with the sweep's measured curve,
and nothing measured a larger parent.

`STOP_WAIT` in the same module has no value.
The proposal is 2,000 milliseconds.
The measured stop was 6 to 15 milliseconds,
so 2,000 is over a hundred times the largest measured wait,
and it still bounds a manager that never answers.

The removal's request budget is what step 16's report names as the command-path switch's need.
The proposal is `RequestBudget::of_input_bytes(65_536)`,
that is 64 KiB.
The largest request the VM accepted was 114,756 bytes,
and the 64 KiB budget is 2.3 times the 512-name request and under the largest accepted one.
Its reply is `max(256 KiB, 128 KiB)`,
well inside the binding's ceiling.

## What this did not show

- This host's storage,
  because the pool is a file in a guest as the plan's storage half says.
- A real ZED-attached handler,
  because the arm mounts with the kernel's own call or the built sweep,
  and no `zfs-zed` delivered an event in this run.
  The event's share is cited from the 2026-11-21 record and is not this round's measurement.
- The handler process's own start,
  its readiness check and its receipts' write,
  which step 8's arm covers and this round does not.
- Downstream preparation after an add,
  such as pnpm's import,
  Cargo's prewarm or an editor's watch rebinding,
  which
  [the design](cli-git-zfs-worktree-design-proposal.md)
  also says the research did not measure.
- A holder that flushes state before leaving,
  such as an editor or a package manager.
  The measured holders were sleeping processes and a shell that ignored the signal.
- A parent with more than 64 clones,
  and a sweep of the whole reconciliation rather than one parent's clones.
- Pinned incompressible output,
  because the arm's output was one repeated byte,
  and the correcting probe did not run because the guest's agent stopped answering.
- A lost event's real delay,
  because the 5,000-millisecond window is an observation and no trigger's loss was induced.
- This host's load,
  because other rounds' guests were running throughout.
  The recorded spread is the spread under contention,
  not a quiet machine's.

## Evidence

- `keep/guest/step-22-measurements-arm.log` is the arm's whole run at `7e6a51baad`'s tree,
  with the `STEP22` lines this record quotes.
- `keep/guest/holder-scan-arm.log` is plan step 9's scan in the same guest and run.
- `keep/guest/step-22-measurements-arm-run-1.log` is the first run.
  Its workload was inflated through the `uses/` links,
  and the product defect it then found is fixed by
  `fix(git-policy-cli): a capture longer than its prefix reads back` (`302ecbcc9a`).
- `package/git-policy/cli/src/native/binary/binary_zfs_step_22_measurement_guest_tests.rs` is the arm.
- `package/git-policy/cli/e2e/zfs-step-22-measurements-guest.ts` is the driver.
- The guest `zfs-step-22-measurements` was destroyed through `mvm` when the arm ended,
  and `mvm --backend libvirt list` no longer names it.
