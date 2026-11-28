# What the wrapper's `0.6.1-alpha.48` release passes found

Round of 2026-11-27,
unapproved and non-final.

Two passes read the release `0.6.1-alpha.48`:
a behaviour pass,
which exercised alpha.47 and alpha.48 side by side,
and a splits pass,
which read what the 344 refactor commits of the range did.
This record keeps the findings and the measurements that no other file holds,
in the passes' own words where they had them.
Every figure is the pass's own measurement,
taken on 2026-11-27;
nothing here was re-measured for this record.

The behaviour the release rests on is the code,
the gate,
and the service-manager install proof named under
"No change of behaviour is not demonstrated" below.

## The behaviour pass found no High finding

Neither pass raised a High finding.
The behaviour pass's two Mediums and six Lows were all fixed by rounds merged on 2026-11-27,
each with a control that fails without the fix;
what each said is history,
and the controls pin the fixed words.

The splits pass's Mediums and Lows are below.

## The behaviour pass: what held

These held,
exercised with both binaries:

- the slot refusals,
  for the spellings they name;
- the merge pass's union of deletions;
- the self-check's memory,
  62.7 to 66.1 MB in alpha.47 and 9.3 to 11.0 MB in alpha.48;
- named pipes,
  answered and not waited on;
- the typed ssh relay at a terminal,
  which ended about 100 ms sooner.

## The behaviour pass: read and not run

Three surfaces were read only,
because nothing outside the process can start them:
the prune race,
the conversion environment's precedence,
and the service-only changes.

## The splits pass: no change of behaviour is not demonstrated

Its verdict is that "no change of behaviour" is not demonstrated.
Of the items compared at both ends,
97.9 percent have identical text,
no test-only switch is in either release binary,
no static became two and no trait impl was lost;
but 318 of the 344 refactor commits carry changed lines that are not moves,
and the pass read three of them in full.

Neither pass ran `git commit` in a release binary,
because without a session bus every commit form stops at one refusal.
So for the service paths the evidence is the reading,
the gate,
and the service-manager install proof,
which issue 6587's second round ran to its end (`fdd9ce0e83`):
alpha.48 installed over alpha.47 in a container of the gate image,
and the proof printed

```text
THE PROOF PASSED: the instance the older release loaded started as this release, and the pair of commits each landed once
```

## The splits pass: reach

At least 31 definitions became nameable from outside the crate,
seven of them private helpers a split made bare `pub`,
one of which removes directories.
The list of published names is
`src/native/source_guard/items_nameable_from_outside_release.txt`,
and its control is
`src/native/source_guard/nothing_becomes_nameable_from_outside_tests.rs`,
which fails on a published name that has no line there.

## The splits pass: refactor commits read and not read

The extract-function refactors are unread but for ten commits.
A second read of the seven the pass named first,
all on the client's and the service's commit paths,
found each equivalent by reading
(`e90482f100`,
`dde5c4be3b`,
`bad45d0420`,
`01c2f32e2c`,
`6a24c6342b`,
`f4e8d2fb9b`,
`8af8f56255`).
One guard covers less than its name says:
`src/native/source_guard/hook_self_invocation_tests.rs` reads 263 of the 699 code lines
the commit preparation spans,
which the list-guards round took first.
About three hundred refactor commits stay unread.

## The splits pass: guards that name moved files

The pass found 71 pairs of a source guard and a file it names,
where code left the file in a split and the guard does not name where it went.
Two were checked,
and neither was a hole.
The list-guards round holds the pairs now.

## The splits pass: low findings

Eight points,
as the pass put them:

- the sweep's warnings carry four new log tags;
- a commit labelled `docs` changed the `shutdown-unmount` help row
  (`066e33a47d`,
  noted on the commit);
- the panic notice's source locations moved;
- 34 items lost a platform gate on split-off files,
  with no effect on x86_64 Linux;
- tag constants are copied into more files;
- the README's allow-list counts are stale;
- two commits in the range cannot build alone;
- the linter the binary carries moved four alphas.

The platform gates are fixed:
a split-off file keeps the platform gate of the file it left (`4d6f09f1f2`).

## The splits pass: a split that was not a pure move

Not every split is a pure move.
The landing split's last commit (`3b9ada357b`) boxed the commit inside the service's answers
and made the commit command pass its own copy of its client configuration,
both for Clippy after the move,
which the release pass that reads the range reads as a change.

## The read of `0.6.1-alpha.52`: the guest drivers' shared steps

Round of 2026-11-28,
unapproved and non-final.
This file also holds the passes over `0.6.1-alpha.48`;
the section here is one read of `0.6.1-alpha.52` that the session weighed,
and the round that answered four of its findings and checked the fifth.
Only the owner declares it complete.

The read is a second pass by another agent of the same vendor as some of the authors,
so it is no independent review.
Its limits are the ones it states:
it started no guest,
it built no binary,
and it drove the drivers of the package against a stand-in `mvm`,
`node` and `tar`.
The round below kept to the same ground:
every reproduction runs a stand-in for `mvm`,
and no guest of this machine was used.

The read rated two Mediums and three Lows on the shared steps that every guest driver runs
(`bin/mvm-guest-step-command.ts`,
`bin/mvm-guest-detached-script.ts`,
`bin/mvm-guest-command-line.ts`,
`bin/mvm-guest-whole-run.ts`).
The Mediums were both consequences of the merge `98996296fb`,
which replaced three sets of guest helpers with one,
and the release said of that merge only that what every driver sends its guest is unchanged.
Two of those claims were about the drivers' own process and about the status of a command,
and both were false.

### What a real `mvm exec` answers for a command that exits nonzero

The refusal of a refused launcher rests on this,
so it was taken from the producer's source and its controls,
not from memory.
`package/mvm/cli/src/cli.ts` answers the guest command's own status:
`const result = await backend.exec({ command, name })`,
then `if (result.exitCode !== 0) process.exitCode = result.exitCode`.
The status itself is the guest agent's answer,
`exitCodeOf` in `package/mvm/cli/src/guest-exec-status.ts`,
which is `exitcode` when the answer has one and `128 + signal` when a signal ended the command.
The controls beside it agree:
`guest-exec.unit.test.ts` returns `exitCode: 3` for a command ending `exit 3`,
and `137` for one that ended `kill -KILL $$`.

So a launcher whose first line is `test "$(hostname)" = NAME || exit 1`
ends `mvm exec` with status 1 on another machine's guest,
and a status other than zero is exactly "the launcher was refused".

### M1: an interrupted driver left the step it had started running

Every step command is started in a process group of its own (`detached: true`),
which is what lets a step's bound stop everything the step started.
The same separation is what a terminal's interrupt key cannot reach:
SIGINT goes to the driver's own group,
so the driver ended and the step went on working with nobody reading it,
bounded at the two hours a guest build may take (7,200,000 ms),
and no driver handled a signal.

Reproduced before the change with the real `forced-act-guest.ts create`
and a stand-in `node` for the recipe step that records its own process number and sleeps,
SIGINT sent to the driver's group as a terminal sends it:

```text
before-main e2e/forced-act-guest.ts create: driver pid 1146505 leads group 1146505; the step it started is pid 1146521
before-main: before the signal, the step is alive=true
before-main: after SIGINT to the driver's group: driver exit code null signal SIGINT; step alive=true
before-main: the step outlived the driver, and this reading killed it
```

`01cb2ed4c9` holds the step commands this process is still waiting for in
`bin/mvm-guest-live-steps.ts`,
and a signal that ends the driver stops each through the group stop its own deadline uses
(`stopProcessGroup`),
says one line naming the command and its process group,
and then ends the driver the way the signal would have:
the handlers go first,
so the re-raised signal has its default action.
SIGHUP,
SIGINT and SIGTERM are the three signals,
because a terminal that went away is the same
hazard as one that was interrupted.
The same driver,
the same signal,
after the change:

```text
after e2e/forced-act-guest.ts create: driver pid 1257141 leads group 1257141; the step it started is pid 1257237
after: after SIGINT to the driver's group: driver exit code null signal SIGINT; step alive=false
git-policy guest step: SIGINT, so this run stopped every step command it had started and still had running: node .../e2e/run-fixture.ts create --name m1-after-guest --log-dir .../logs (process group 1257237)
```

The controls are three cases of `bin/mvm-guest-step-command.unit.test.ts`,
each running the driver as a program of its own,
because a signal's disposition belongs to the whole process:
SIGINT and SIGTERM each stop the step and name it,
and a step that had already ended is neither stopped again nor named.
Shown red by taking the hold out of `awaitStepCommand`:

```text
[FAIL] stops the step it started when SIGINT ends the driver, and names what it stopped
  AssertionError: expected false to equal true  -- interrupted.stepIsGone
[FAIL] stops the step it started when SIGTERM ends the driver, and names what it stopped
[PASS] stops and names no step that had already ended
```

What the change does not do,
and the record of it says so:
a guest the interrupted run had made is left as it was,
because taking a guest away is a step of the run's own,
so the guest's name and `mvm destroy` are what a person needs next.

### M2: a refused launcher was waited out instead of refused

`startDetachedGuestScript` called `runQuietStepCommand` for the `mvm exec`
that starts the launcher and threw the status away,
while its own doc promised a throw "when the guest could not be asked to start the script".
The adoption and start-probe drivers then waited for a log no guest would write,
for `crateBuildTimeoutMilliseconds + buildFinishMarginMilliseconds`
(2,400,000 + 180,000 ms,
43 minutes,
`e2e/adoption-guest-arms/crate-build.ts:88` and `e2e/start-probe-arms/crate-build.ts:200`),
and the step 22 driver for its own crate-build bound.
Before the merge the three drivers failed at once,
with the guest's name,
through `e2e/guest-host/guest-side-scripts.ts:187`.

Reproduced against a stand-in `mvm` that accepts both pushes and exits 1 for the launcher,
the driver program then doing what every driver does next:

```text
before-main: the driver program exited with 0
the start returned after 129 ms, although the launcher exited 1
the wait refused after 15205 ms: build.log: the guest script did not finish inside 3000 ms; see build.log
```

The wait's bound was 3,000 ms and it answered after 15,205 ms,
because a look that finds no status line waits one `POLL_INTERVAL_MS` (15,000 ms) first:
the real drivers' wait would be the 43 minutes of their own bound.

`09e47a5285` reads the status and refuses a nonzero one at once,
naming the launcher's command,
its status and the guest's name:

```text
after: the start refused after 66 ms: mvm --backend libvirt exec guest-a -- sh /mnt/shared/build.log.launch.sh exited 1: the launcher was refused, so 'guest-a' started none of build.log.step.sh
```

The controls are two cases of `bin/mvm-guest-detached-script.unit.test.ts`:
one reads the refusal,
the guest's name,
the status,
the three calls the stand-in was given and
that nothing came after the launcher,
and the other is the case where the launcher exits zero,
which must reach the wait,
so the probe is shown able to tell the two apart.
Shown red by dropping the status again:

```text
[FAIL] refuses at once when the launcher exited nonzero, naming the guest and the status
  expected 'the start returned\nthe wait refused:…' to include 'the start refused:'
[PASS] waits for the script when the launcher exited zero
```

The removal-lineage and settling drivers carried `nameCheck: 'none'` and ignored the launcher's
status before the merge too,
so what changes for them is new behaviour and not restored behaviour:
a launcher that `mvm` could not run,
or a guest the command could not reach,
refuses at once with the launcher's status and the guest's name
instead of leaving them to wait out the build's bound for a log that would never appear.
`e2e/README.md` says that,
and says which drivers carry no name check at all.

### L3: the step-shaped name check guarded nothing

`bin/mvm-guest-command-line.ts` wrote the `step` shape as `test "$(hostname)" = NAME`
with no `|| exit` and no `set -e`,
so on a machine of another name the step file ran the script anyway,
while the shape's doc said the detached script refuses itself.
`e2e/step-22-arms/detached-step.ts` is the driver that sends this shape,
and its check was the only one its scripts carried.

Reproduced by running the package's own step text with `sh` and a stand-in `hostname`:

```text
## before-main: the step-shaped check, run with sh on a machine of another name
sh exit status: 0
the log the step file left: "the script itself ran\nDONE=0\n"
## before-main: the same file on the guest it names, which is the positive control
sh exit status: 0
the log the step file left: "the script itself ran\nDONE=0\n"
```

`85213b9623` makes the shape refuse,
and writes the status line the host already waits for before it exits,
because a step file that exited without one would leave the host asking until the script's bound
passed:

```text
test "$(hostname)" = guest-a || { echo "DONE=1" >> /mnt/shared/build.log; exit 1; }
...
## after: the step-shaped check, run with sh on a machine of another name
sh exit status: 1
the log the step file left: "DONE=1\n"
## after: the same file on the guest it names, which is the positive control
sh exit status: 0
the log the step file left: "the script itself ran\nDONE=0\n"
```

So the host's next look finds `DONE=1` and refuses with the status the step wrote,
at once rather than at the end of the script's bound.
The shape was kept rather than dropped,
because the step 22 driver's record says the check has always sat in its step file,
and because the one place the three helper sets disagreed is worth keeping as a distinction.
Its controls are two cases of `bin/mvm-guest-command-line.unit.test.ts`,
which run the step file with `sh` on another machine's name and on the guest's own,
and the two text cases beside them.
Shown red by writing the bare check again:

```text
[FAIL] puts a check that refuses, and its status line, in the step when the launcher carries none
[FAIL] bounds the script, sends both streams to the shared log and appends its status
[FAIL] refuses at once, and writes the status line the host waits for, on a machine of another name
  expected +0 to equal 1
[PASS] runs the script on the guest it names, which is what the refusal is told apart from
```

### L4: the sentences whose wording the merge changed, checked and left

The read argued from the source that two failure sentences changed with the merge:
`git-policy guest run: destroying '<name>' also failed` (from the old lifecycle's own label)
and `exited -1` where the old helper printed `exited null` for a command a signal ended.
Nothing in the package or in this repository's records reads either sentence:
`rg 'exited -1'` and `rg 'exited null'` find nothing at all,
`rg 'also failed'` finds unrelated prose and the slot-state driver's own line,
and no test asserts either form.
So both sentences are left as they are,
and this record says they were checked.

### L5: two sentences that did not say what their own code says

`e2e/README.md` said four drivers "hold no plain long command at all"
over a list that names three plain `guest()` commands of one of them,
each bounded at 900 s and none timed on its own
(`e2e/adoption-guest-arms/store-dataset-and-fsync.ts:115`,
`delegation-arm.ts:155`,
`snapdir-arm.ts:149`).
The sentence names those three now and says what the section's own rule owes them:
a command bounded above five minutes runs detached,
nothing has timed these three,
so a round that keeps them plain owes a measurement under five minutes or a detached script.

The package guide said the start probe's shared header "is quoted now",
which reads as one of the merges of this release.
It was quoted before the release:
`dacca205b6` (2026-11-27T11:57:11-05:00) quoted it,
and that commit is an ancestor of `0.6.1-alpha.51`.
The guide names the commit and its time now.

### What the read's own ground did not cover

No guest was started,
so none of these readings says what a real `mvm` does with a signal
sent to its own client,
and the two scripts a launcher runs were never executed inside a guest.
The step-shaped check was read with `sh` of this host,
with the guest's shared folder replaced by a folder of the round's own.
What a `mvm exec` answers for a nonzero command is the producer's source and its controls,
not a guest of this round.
