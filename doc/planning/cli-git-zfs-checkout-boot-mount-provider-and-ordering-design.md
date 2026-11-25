# Which unit mounts the checkout, and the ordering the boot gate needs

Design proposal by an agent,
revised a sixth time,
which writes the owner's answers to step 19's choices into the design,
**unapproved and non-final**.
It answers the two parts plan step 19 leaves open
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, "Step 19", "Open items"),
carries the owner's answers from
`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`,
keeps every option the owner did not choose in "The options not chosen" with its evidence,
and measures on one throwaway guest the four things the answers left unmeasured:
what a provider restart does to the chosen units,
what makes the nested dataset's unit tolerate an already-mounted dataset,
what a provider failure does to the gate now that the gate merely wants the provider,
and what logins do while the gate hangs and fails.
It designs no wrapper code,
edits nothing under `doc/decision/` or `doc/handover/`,
and applies nothing to any machine.

Every claim in this document carries a label for what it rests on:

- a kept log under this round's `keep/logs/`, named in backticks;
- a journal line, quoted with its Unix timestamp and the boot it belongs to;
- a host read, kept under this round's `keep/`, with the command that printed it;
- a source reading, with the file and the lines;
- `[nothing]`, where a sentence is an inference, a recommendation or an open question.

This round's folder is `~/temp/agent/zfs-step-19-fifth-revision-2026-11-24/`.
"Round one" means `~/temp/agent/zfs-step-19-checkout-boot-mount-provider-and-ordering-2026-11-23/`,
"round two" means `~/temp/agent/zfs-step-19-design-after-its-gate-review-2026-11-24/`,
"round three" means `~/temp/agent/zfs-step-19-third-revision-2026-11-24/`,
"round four" means `~/temp/agent/zfs-step-19-fourth-revision-2026-11-24/`,
and "round five" means this folder's own first half, the fifth revision's round.
A log named without a round is this round's.
Where a fact rests only on an earlier round's log, the round is named with it.

## The owner's answers, and how this revision writes them

The owner answered every choice on 2026-11-24T08:20:00-05:00 through the question tool, and
`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md` records them.
That record is the owner's and this revision neither edits nor paraphrases it as its own:
every answer below is quoted by what it settles, with the record as its basis, and the design's own
sentences follow from it.

- **V1: `zfs-mount.service` is the provider.** The gate names that unit, which runs `zfs mount -a`.
- **V2: `Wants=` plus `After=`.** A provider restart leaves the gate up, and the provider drop-in
  re-runs it, so no attached clone is unmounted.
  The record says this amends the accepted lost-event and propagation records, which describe
  `Requires=`, and that what the gate does when the provider fails was unmeasured.
- **V3: `RequiresMountsFor=/home/user/Monochromatic`.** The gate carries the one line.
- **V4 and V9: `canmount=noauto` on `zpcachyos/nosnap/vm-builder-output`, mounted now by a unit running
  `/usr/bin/zfs mount`, replaced by the wrapper's `mount-path-descendant` mode once it exists.**
  The unit is ordered after the gate.
- **V5: the gate makes the checkout's mount private as its first act.**
- **V6: each service that reads the checkout requires the gate**, or one nested dataset's unit.
- **V7: `doctor` prints the whole boot setup**, as the design's V7 list has it.
- **V8a: logins proceed.** The `Before=systemd-user-sessions.service` line is dropped from the gate,
  from the nested dataset's unit and from the consumers.
- **V8b: `TimeoutStartSec=5min` and `TimeoutStopSec=1min`** as the owner's numbers, replaced by
  measured ceilings when plan step 22 measures the workload.
- **The ZED hook's AppArmor permissions are rules in `/etc/apparmor.d/local/zed`**, which every
  packaged zedlet inherits, and the owner accepted that cost.

### What the answers change in the unit texts

Three blocks change shape, and each is written out whole under "The units, whole":

- **the boot unit** carries `Wants=zfs-mount.service` in place of `Requires=`, and no
  `Before=systemd-user-sessions.service`;
- **the nested dataset's unit** runs an explicit `zfs mount`, is ordered after the gate, and carries a
  condition that skips the start when the dataset is already mounted at its mountpoint;
- **the two account consumers** require the gate or that unit, and carry no before-login line.

The two bounds are the owner's numbers, so the placeholders the earlier revisions carried are filled
in this text, and step 22 replaces them with measured ceilings.

### What the answers left unmeasured, and what this round measured

The record names one gap itself: "What the gate does when the provider fails is still unmeasured", and
that describes `Requires=`. Four questions followed from the answers, and this round answered each in
one throwaway guest (kept logs, one boot each):

- **Does the nested dataset's unit restart when the provider restarts?** No.
  With `Wants=`, the gate is reloaded through the provider drop-in and the nested unit's invocation is
  untouched (kept log `restart-provider-chosen.log`).
- **What happens when something else restarts the nested unit while its dataset is still mounted?**
  Its condition skips the start: a gate restart restarts it, and the journal holds
  `skipped, unmet condition check ConditionPathIsMountPoint=!…` (kept log `restart-gate-chosen.log`).
  The same skip happens at a boot where the provider mounted the dataset first
  (kept log `evidence-chosen-mounted-first.log`).
- **What does a provider failure do to the gate now that it only wants the provider?** Nothing: the
  provider ends `Result=exit-code`, `ExecMainStatus=1`, and the gate runs and finishes green beside it
  (kept log `evidence-chosen-provider-fails.log`).
- **What do logins do while the gate hangs?** They proceed, immediately: sessions start 0.44 s after
  the gate and 20.4 s before its failure (kept log `evidence-chosen-stages.log`).

The chosen set then booted green, failed closed on a missing checkout key, and spent its bounds, each in
its own boot: `evidence-chosen-green.log`, `evidence-chosen-key-removed.log`,
`evidence-chosen-stages.log`.

## What the read of the fourth revision changed

The read
(`~/temp/agent/gate-review-zfs-step-19-fourth-revision-2026-11-24/keep/read-of-the-fourth-step-19-revision.md`,
a same-vendor second pass, not an independent review)
found no high defect, eight medium ones and twelve lows.
All eight held as findings and are folded here.
One of them, the second, proposed a remedy that does not work, which this round measured; the fold
answers it with the measurement rather than with agreement.
The read numbered eleven lows and counted twelve; each numbered low, and each item of its format
check, has an entry here.

### The medium findings

**M1, the worst case is understated.** Held, and this round measured a case worse than the read's own
correction.
The fourth revision's formula was the probe's deadline plus `TimeoutStartSec` plus `TimeoutStopSec`,
about 110 s for a 20 s start bound and the default 90 s stop bound.
The read said "one stop bound per stage the processes survive" and doubted its own earlier smaller fix.
This round's arm settles the count.
A gate whose start pass and whose stop pass both ignore SIGTERM, with `TimeoutStartSec=8` and
`TimeoutStopSec=4`, spent the start bound once and the stop bound **three** times
(kept log `evidence-stages.log`, boot `fec7550a-5a0f-450c-802b-f9e004560004`, one contiguous run):

```text
1795530374.179243 zfs-step-19-fifth-2026-11-24 systemd[1]: Starting reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts...
1795530382.442884 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: start operation timed out. Terminating.
1795530386.539982 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: State 'stop-sigterm' timed out. Killing.
1795530386.540052 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Killing process 911 (sweep-hang-term) with signal SIGKILL.
1795530386.540176 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Killing process 1075 (sleep) with signal SIGKILL.
1795530386.540434 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Main process exited, code=killed, status=9/KILL
1795530390.623415 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: State 'stop-post' timed out. Terminating.
1795530394.707074 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: State 'final-sigterm' timed out. Killing.
1795530394.707136 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Killing process 1076 (sweep-stop-hang) with signal SIGKILL.
1795530394.707231 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Killing process 1092 (sleep) with signal SIGKILL.
1795530394.707449 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Failed with result 'timeout'.
1795530394.707667 zfs-step-19-fifth-2026-11-24 systemd[1]: Failed to start reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts.
1795530394.711395 zfs-step-19-fifth-2026-11-24 systemd[1]: Starting systemd-user-sessions.service - Permit User Sessions...
1795530394.724362 zfs-step-19-fifth-2026-11-24 systemd[1]: Finished systemd-user-sessions.service - Permit User Sessions.
```

The three stages are the SIGTERM the start bound's failure sends, the `ExecStopPost=` the failure then
runs, and the final SIGTERM before the final SIGKILL.
SIGKILL is the one signal a program cannot refuse, which is why three bounds were spent and not four:
the final SIGKILL stage ended in a third of a millisecond.
From the unit's start to the sessions unit is 20.53 s, against 8 s of start bound and 4 s of stop bound,
and the boot's own evidence file agrees on when each pass began
(`SWEEP-TERM start … uptime=6.21`, `SWEEP-STOP-TERM start … uptime=18.56`).
Round four's 23.54 s is the **cooperative** case and is labelled so here: that arm's main process died
at SIGTERM within a millisecond, so only the stop-post stage ran
(round four's `evidence-bound-stop.log`, boot `ad2d3287-25f8-4086-819a-3ac4320671ea`).
Folded in: "What a hung start costs" states both arms, the stage count and the arithmetic, and V8b
states the hold each answer leaves rather than a formula with one stop bound in it.

**M2, the stop-failure mode cannot live where it stood, and the placement the read proposed does not
work either.** Held in substance; its remedy answered with the measurement.
Held: no booted unit text ever carried the line, and the same line inside the unit file is overridden
by Fedora's type-level drop-in. This round measured that directly
(kept log `stopmode-shapes.log`, the block "the line in the unit file itself, no drop-in" prints
`DropInPaths=/usr/lib/systemd/system/service.d/10-timeout-abort.conf` and
`TimeoutStopFailureMode=abort` although the file holds `TimeoutStopFailureMode=terminate`).
What does not hold is the remedy "move it to a name-specific drop-in directory": **the file name
decides the parse order, not the directory.** The same line at
`reconcile-git-worktree-clone-mounts.service.d/10-terminate.conf` lost:

```text
DropInPaths=/etc/systemd/system/reconcile-git-worktree-clone-mounts.service.d/10-terminate.conf /usr/lib/systemd/system/service.d/10-timeout-abort.conf
TimeoutStopFailureMode=abort
```

Renamed to `zz-checkout-stop-failure-mode.conf` in the same directory, the same line won, because the
type-level file is then parsed first:

```text
DropInPaths=/usr/lib/systemd/system/service.d/10-timeout-abort.conf /etc/systemd/system/reconcile-git-worktree-clone-mounts.service.d/zz-checkout-stop-failure-mode.conf
TimeoutStopFailureMode=terminate
```

and masking the type-level file for the whole machine won as well, which is the undo that file's own
comment gives an administrator:

```text
DropInPaths=/etc/systemd/system/service.d/10-timeout-abort.conf
TimeoutStopFailureMode=terminate
```

So a kind number is not enough and a directory is not enough: a file name that sorts after the
distribution's is what decides, and a distribution that later adds a later-sorting file changes the
answer again.
Folded in: the line left the unit text, the drop-in `zz-checkout-stop-failure-mode.conf` carries it,
this round's green boot ran with Fedora's type-level drop-in installed and unmasked and reported
`TimeoutStopFailureMode=terminate` (kept log `evidence-green.log`, boot
`de767eea-e7a9-4c4d-ad6a-e4a97803507b`), and V7's report reads the effective value back with
`systemctl show` instead of trusting the file it installed.
The purpose the line serves still holds and is measured: with `terminate` in force, the stages arm's
stop bounds ended in SIGKILL with no `Aborting` line, no `SIGABRT` kill and no coredump in that boot,
against round four's arm where Fedora's `abort` left two coredumps.

**M3, the fourth revision's V8 option C kept the login hold through other units.** Held, and measured.
The per-dataset unit and both account consumers carry their own
`Before=systemd-user-sessions.service`, and none of them has a start bound: systemd reports
`TimeoutStartUSec=infinity` for all three
(kept log `state-green.log`, "the stop-failure mode and the bounds systemd reports, per unit").
So C's "no login hold at all, and separable" was true of the gate's own line and false of the boot,
and the per-dataset unit is itself an unbounded oneshot ordered before sessions: a hung descendant
mount holds every login under every option.
Folded in: the fourth revision's option C is the answer the owner gave (V8a: logins proceed), which
drops the ordering line from all four units and lets each consumer require what it needs, the treatment
the fourth revision's option E had; the nested unit's and the consumers' bounds are stated under "The
nested dataset's own unit"; and V8 was split into two questions, V8a for the ordering and V8b for the
bound values, which the owner then answered.

**M4, unsourced numbers shipped in "what ships".** Held: the fourth revision's "what ships while it is
open" printed `TimeoutStartSec=20` and `TimeoutStopSec=5`, which are the guest's own test values, while
that paragraph's option B said five minutes and one minute. Folded in: no number ships here without a
measurement or the owner's decision beside it, and V8b names exactly which values ship and where each
comes from.

**M5, V9's option C was printed as a strawman, and its real cost is larger than the read says.** Held.
The fourth revision printed C with `[Service]` lines only, leaving out the `Requires=` and `After=` on
the gate that its own option A carries, so "no verification handshake" described the print and not the
option. This round wrote C with A's `[Unit]` lines and booted it: the unit mounted the dataset, its
consumer saw it, and sessions started last (kept log `evidence-v9c.log`, boot
`14c29514-53e9-4fa5-9daa-fc0d742f810b`).
The read's own summary of C, "it lacks only the mode's own re-read of the checkout row and its
refusal", understates it, and this round measured the rest: after `systemctl restart zfs-mount.service`
the unit re-ran its explicit mount on the dataset that was still mounted and ended

```text
1795530237.561479 zfs-step-19-fifth-2026-11-24 zfs[1335]: cannot mount 'zfsstep19/nosnap/vm-builder-output': filesystem already mounted
1795530237.561874 zfs-step-19-fifth-2026-11-24 systemd[1]: git-worktree-nested-output-mount.service: Main process exited, code=exited, status=1/FAILURE
1795530237.561960 zfs-step-19-fifth-2026-11-24 systemd[1]: git-worktree-nested-output-mount.service: Failed with result 'exit-code'.
1795530237.562126 zfs-step-19-fifth-2026-11-24 systemd[1]: Failed to start git-worktree-nested-output-mount.service - Mount a dataset whose mountpoint lies inside the verified checkout.
1795530237.562187 zfs-step-19-fifth-2026-11-24 systemd[1]: Dependency failed for git-vm-builder-standin.service - A vm-builder service of the account that needs the dataset under the checkout.
1795530237.562198 zfs-step-19-fifth-2026-11-24 systemd[1]: git-vm-builder-standin.service: Job git-vm-builder-standin.service/start failed with result 'dependency'.
```

with that consumer's job failing `dependency` while the dataset stayed mounted
(kept log `restart-provider-v9c.log`; one contiguous run of that boot).
So C needs the already-mounted rule as much as the mode does, and C's advantage is smaller than "it
ships today" alone suggests.
Folded in: V9's C is written with A's lines, ranked again on what each delivers, and the section
"What V4 and V9 do to this host's one dataset under the path" states the consequence.

**M6, "nothing else the provider would have mounted is mounted either" is false.** Held.
In that same boot the home dataset was mounted: `zfs list` reports `zfsstep19/home` with
`MOUNTED yes`, and the arm's unit list shows the learned `srv-zfs…-home.mount` active
(round four's `evidence-hazard-flag-on.log`, boot `fcfae077-91df-49bf-bced-ff085a27b7ca`).
The provider did fail, which is what the section measures.
Folded in: the sentence now says that the provider unit fails and that datasets outside the failed
mountpoint's subtree still mount, and the OpenZFS source that decides it is cited where the claim
appears.

**M7, the AppArmor ranking.** Held, and folded under "AppArmor on this host": the four shapes the read
names are weighed as alternatives with their costs, "least privilege" is narrowed to the one path it
covers, B is stated as rules plus a profile rather than a profile instead of rules, the slips the read
lists are corrected, and the owner's recorded answer is read for what it actually decided.

**M8, the fourth revision's V2 option C contradicts this document's own drop-in section.** Held: the provider drop-in is
installed under every answer to V2 and its `ExecStartPost=` re-runs the gate, so "nothing re-runs the
gate after a provider restart" is false for C, and "C delivers neither" is wrong about the re-run.
Folded in: C's cost says the drop-in re-runs the gate and that B and C differ by the pull-in alone, and
the sentence that ranks B over A marks its fail-closed half unmeasured, which is how the option itself
already labels it.

### The low findings

- **L-1, quotes that were not contiguous.** Held. Every journal block in this revision is one
  contiguous run of one log. Where two runs are joined, the text says so and the joined part is
  quoted separately, which is what the boot-unit block now does.
- **L-2, the style fold was incomplete.** Held. There is no em-dash or en-dash in this revision, no
  pointer by position or by another revision's number, and no bare date: the sentence the read calls
  garbled is rewritten as the two monotonic uptimes and the 17 ms it measures.
- **L-3, the audit fold was cited loosely.** Held. The print in
  `evidence-as-written-stopsec.log` holds thirteen records where the same log's own counts say
  fourteen, so this revision says "every record but one" and names what the counts show. The sentence
  that claimed the arm's boot held four records and that its own `daemon-reload` wrote them is gone:
  the green boot's window holds none, the file's records all fall in the earlier boot's window, and the
  guest's stand-in units ran in `unconfined_service_t`, which the evidence block does print.
- **L-4, `-.mount`.** Held. On this host `-.mount` is `LoadState=loaded` with empty `FragmentPath` and
  `SourcePath`, so by the fragment rule `RequiresMountsFor=` would put only `After=` on it here, and
  the guest's 258 put it in `Requires=` because that machine's root mount unit has a fragment
  (kept log `host-reads.log`, and `state-green.log` for the derived set).
  Corrected in V3, in `F26` and in the sentence in "The answer in brief".
- **L-5, AppArmor text slips.** Held, all four, and corrected: `zpool` has its own profile at
  `/etc/apparmor.d/zpool` line 11, in complain mode, so the `rpx` on `zpool` reaches that profile and
  not `mount-zfs`; the `/run` write list now carries
  `@{PROC}/@{pid}/task/@{tid}/comm rw` at line 56; the phrase "no `px` of its own" is replaced by the
  two `rpx` rules it meant; and B's cost is split into a capability cost and a maintenance cost.
- **L-6, host facts without a kept read.** Held. Every host fact this revision states is read again
  this round and kept: `stat` of the path's three directories, `systemctl --version`, `pacman -Qo` of
  the masked unit, the generator and its cache, the pool's `keyformat` and `keylocation`,
  `/etc/apparmor.d/disable/`, the unprivileged profile read, and `-.mount`
  (kept logs `host-reads.log`, `apparmor-and-unit-reads.log`).
- **L-7, V1's slips.** Held on all three. Option C is no longer offered as a pickable unit, because
  option A's own text already carries the conditional; B's cost now says that the property belongs to
  a dataset setup creates, and that today's checkout is a directory in the home dataset; and A's
  printed output now carries its command line.
- **L-8, V4-B and V5-A.** Held. V4-B prints this round's read of the host's own dataset with its
  command and its output (`canmount on`, `mounted yes`), and V5-A shows the measured
  `SWEEP propagation=private` from a boot instead of a templated `findmnt`.
- **L-9, citation slips.** Held. `handler_mode_arguments.rs:113` to `:115` becomes `:112` to `:115`,
  and the two different ranges for the same journal block become the one block's own line numbers.
- **L-10, one cause stated as the cause.** Held. The boot's own start of the gate is now stated as
  what the arm shows, with the unit's default `After=basic.target` named as the other cause the arm
  does not exclude.
- **L-11, the hazard arm's unit set.** Held. The hazard control now says that both drop-ins were absent
  and that the per-dataset unit and both consumers were disabled in that boot
  (kept log `state-hazard-flag-on.log` in round four's folder).
- **The format check.** Held: V1's "B over C" was empty, V9's "B over C" rested on a strawman, V8's
  "E over C" rested on C being unbounded, and four options lacked a command line or a "Not run" line.
  Every option in this revision has unit lines, a command and what it printed or a "Not run" line
  with the reason, and a reason for each adjacent pair by what each option delivers.

## What it ran, and how to read this document

Two throwaway guests on Fedora 43 Cloud Edition, each created and destroyed through
`mvm --backend libvirt` by the name its own create printed, both with systemd `258.11-1.fc43`,
kernel `6.17.1-300.fc43.x86_64`, OpenZFS `2.4.4-1` built from the release tarball pinned at sha256
`2a3c70d55a37cc71618a95a60e81ad66530201eb118d37741dc92efcf848c8b1`, and SELinux enforcing:

- `zfs-step-19-fifth-2026-11-24`, the fifth revision's guest: four boots, three arms and one runtime
  arm;
- `zfs-step-19-chosen-2026-11-24`, this revision's guest: five boots for the chosen unit set, two of
  them with a runtime arm inside, plus the build and fixture boot.

One guest would have been enough for the facts that move with a systemd version, which round three
measured on 262.2; the second exists because the first was destroyed when the fifth revision's round
ended and the branch merged.

The fifth revision's logs, kept:

- `prepare.log`, `prepare-readback.log`, `fixture.log`, `units-install.log`: that guest's versions, its
  file-backed pool and datasets, and every unit and drop-in as systemd reads them.
- `state-green.log`, `evidence-green.log`, `state-v9c.log`, `evidence-v9c.log`,
  `restart-provider-v9c.log`: the fifth revision's green boot, the explicit-mount unit, and the
  provider restart that failed it.
- `state-stages.log`, `evidence-stages.log`, `stopmode-shapes.log`: the three stop stages, and the four
  placements of the stop-failure mode.

This revision's logs, kept:

- `prepare-chosen.log`, `prepare-chosen-readback.log`, `fixture-chosen.log`, `fixture-fix-chosen.log`,
  `units-install-chosen.log`: the second guest's build, pool, datasets and installed stand-ins.
- `state-chosen-green.log`, `evidence-chosen-green.log`: the chosen unit set as systemd reads it
  (`Wants=`, the owner's bounds, the condition, no before-login line) and its green boot.
- `restart-provider-chosen.log`: the provider restart, with invocation ids before and after.
- `restart-gate-chosen.log`: the gate restart that restarts the nested unit, and the skip that follows.
- `state-chosen-green-mounted-first.log`, `evidence-chosen-mounted-first.log`: the boot where the
  provider mounts the nested dataset first.
- `state-chosen-key-removed.log`, `evidence-chosen-key-removed.log`: the checkout's key away.
- `state-chosen-provider-fails.log`, `evidence-chosen-provider-fails.log`: a provider failure with a
  healthy checkout.
- `state-chosen-stages.log`, `evidence-chosen-stages.log`: the hung pass with the bounds scaled down
  inside that arm only.
- `mvm-create-guest-chosen.log`, `mvm-destroy-guest-chosen.log`, `mvm-list-after-destroy-chosen.log`:
  that guest's own create and destroy records, by the name the create printed.
- `host-reads.log`, `apparmor-and-unit-reads.log`: every host fact this revision states, read in the
  fifth revision's round and unchanged since, except where a line says otherwise.

Three limits, so no result is read wider than it is:

- The programs are stand-ins. The gate, the probe, the stop passes and the consumers perform the reads
  this design names, because the wrapper's root modes are not built.
  Every result is about the units, their ordering edges, the failures they carry and systemd's and
  ZFS's own behaviour, never about the wrapper's handler.
- The checkout is its own encryption root on the guest, so one arm can make its mount fail by moving
  one key file. This host's checkout inherits the pool's key, which round four's host-shape arm models.
- No arm made the ZFS mount generator write a mount unit, and no arm ran ZED or AppArmor.
  Those two branches are source and policy readings.

## The answer in brief, as the owner's answers make it

- The provider is `zfs-mount.service` (V1): this host's generator writes no unit for any dataset here,
  and `zfs mount -a` is what mounts them (host reads, `keep/host-reads.log`).
  The gate **wants** it and orders after it (V2), so a provider restart leaves the gate up.
- The nested dataset under the checkout, `zpcachyos/nosnap/vm-builder-output`, carries
  `canmount=noauto` (V4) and is mounted by its own unit, ordered after the gate, which runs an explicit
  `zfs mount` (V9), until the wrapper's `mount-path-descendant` mode replaces it.
- That unit skips itself when its dataset is already mounted, through
  `ConditionPathIsMountPoint=!<its mountpoint>`, and the skip is measured three ways: at a boot where
  the provider mounted the dataset first, at a gate restart, and never in the green boot, where it
  mounts (`evidence-chosen-mounted-first.log`, `restart-gate-chosen.log`, `evidence-chosen-green.log`).
- A provider restart under `Wants=` does not restart the gate, the nested unit or the consumers: the
  gate is **reloaded** through the provider drop-in and every invocation id stays the same
  (`restart-provider-chosen.log`).
- A provider failure under `Wants=` does not fail the gate: the provider ended `Result=exit-code`,
  `ExecMainStatus=1` while the gate ran and finished `Result=success`, its consumers and the nested
  unit with it (`evidence-chosen-provider-fails.log`).
- The provider's own status still says nothing about the checkout **where the datasets under the
  checkout path carry `canmount=noauto`**: with the checkout's key unavailable and no key-loading unit,
  the provider finishes `Result=success`, `ExecMainStatus=0` while the checkout is not mounted at all
  (round three's `evidence-no-key-unit.log` and round four's `evidence-host-shape.log`).
- What fails closed on a missing checkout key is the key-loading unit, which the gate still requires
  where one exists: measured again with the chosen units, the key unit exits `255/EXCEPTION` and the
  gate, both consumers and the nested unit fail with `dependency`, while logins proceed
  (`evidence-chosen-key-removed.log`).
- Where no key unit exists, this host's shape, the gate's own refusal is the signal:
  `SWEEP refused=checkout-not-the-dataset`, exit status two, and the same fail-closed job results
  (round four's `evidence-host-shape.log`, round three's `evidence-no-key-unit.log`).
- Never require a mount unit systemd learned from the mount table.
  Measured at three boots: the requiring unit's start job leaves no journal line at all and the unit
  stays `inactive` with `Result=success`
  (round three's `evidence-learned-unit-only.log`, `boot-arm-learned-unit.log`,
  `boot-arm-learned-with-provider.log`).
- Order with `RequiresMountsFor=<checkout path>` (V3). On this host it puts only an `After=` on
  `-.mount`, whose `FragmentPath` is empty, and the `Requires=` half lands nowhere
  (host read, `keep/host-reads.log`).
- **Logins do not wait for the gate** (V8a). Measured with the chosen text and a hung pass: the gate
  started at `1795546177.439225` and sessions at `1795546177.881849`, 0.44 s later, while the gate
  failed at `1795546198.303748` (kept log `evidence-chosen-stages.log`).
- The gate's bounds are the owner's numbers (V8b): `TimeoutStartSec=5min`, `TimeoutStopSec=1min`,
  which step 22 replaces with measured ceilings. A hung pass still spends the start bound once and the
  stop bound once per stage a program survives SIGTERM, so the arithmetic under "What a hung start
  costs" is the price of the gate's own failure, no longer of a held login.
- The boot unit text as this revision writes it boots green with Fedora's own type-level drop-in
  installed and unmasked, reports `TimeoutStopFailureMode=terminate` through a drop-in whose file name
  sorts after the distribution's, and orders gate, then the consumers and the nested unit, then
  sessions (kept log `evidence-chosen-green.log`, boot
  `d611cc4c-27da-4607-91c9-38e47140c8f1`).
- A consumer that requires the nested unit sees that dataset mounted; one that names a unit the boot
  does not install gets nothing at all, silently (round three's `evidence-v9b.log`).
- The nested unit and both account consumers are unbounded oneshots: `TimeoutStartUSec=infinity`,
  `TimeoutStopUSec=45s` in this round's state log (`state-chosen-green.log`).
  With no before-login line that no longer holds a login; it holds only the services that require them.
- The ZED hook is the root-owned wrapper file under a ZED-legal name, started with no argument, and its
  permissions are **rules in `/etc/apparmor.d/local/zed`**, the owner's answer, with every packaged
  zedlet inheriting them (profile reading, `keep/apparmor-and-unit-reads.log`).
  It still waits on a `zfs-zed.service` that is `disabled` and `inactive` here, and on the `zed`
  profile's own zedlet exec rule naming a folder this host does not have.

## The provider on this host, and how the reads decide it

Read-only, on this host, 2026-11-24; the whole output is in this round's `keep/host-reads.log`.

- The checkout path has no mount of its own; `/home` is the home dataset, mounted `shared`:

  ```text
  TARGET SOURCE                  FSTYPE PROPAGATION
  /home  zpcachyos/ROOT/cos/home zfs    shared
  ```

- The ZFS mount generator is installed, `/etc/zfs/zfs-list.cache` does not exist, and
  `/run/systemd/generator` holds no ZFS mount unit.
  `zfs-mount-generator(8)` says the generator reads only that cache file.
- `zfs-zed.service` is `disabled` and `inactive`, so nothing keeps a cache file either.
- What mounts the datasets is the pair the OpenZFS package installs:
  `zfs-import-cache.service` runs `zpool import -c /etc/zfs/zpool.cache -aN $ZPOOL_IMPORT_OPTS`,
  and `zfs-mount.service` runs `/usr/bin/zfs mount -a`, with `ConditionPathIsDirectory=/sys/module/zfs`,
  `Before=local-fs.target` and `WantedBy=zfs.target`.
  `zfs.target`, `zfs-import.target`, `zfs-import-cache.service` and `zfs-mount.service` are all
  enabled, and `/etc/default/zfs` leaves `ZPOOL_IMPORT_OPTS` empty.
- `/usr/lib/systemd/system/zfs-load-key.service` is a symlink to `/dev/null` owned by
  `zfs-utils 2.4.4-3` (`ls -la` in `keep/host-reads.log`), there is no `zfs-load-key@.service` file,
  and `systemctl is-enabled zfs-load-key.service` answers `masked`.
  That unit is the package's mask, not a key loader, and never was.
- The host's own key path is a person's: `zpcachyos` has `keyformat=passphrase` and
  `keylocation=prompt`, and every dataset below it, the home dataset included, reads
  `encryptionroot  zpcachyos`, `keylocation  none` (inherited) and `keystatus  available`.
- `home.mount` is learned from the mount table (`FragmentPath=` empty,
  `SourcePath=/proc/self/mountinfo`), and the checkout's own unit name
  (`systemd-escape --suffix=mount --path /home/user/Monochromatic`) is `not-found`.

The path the covered entry sits in, read the same way:

```text
/home                      root:root     0755
/home/user                 user:user     0700
/home/user/Monochromatic   user:user     0755
```

## What decides the provider on another machine

The rule, in the order the facts decide it:

1.  The provider is the generated mount unit `<escaped checkout path>.mount`
    when the distribution installs the ZFS mount generator and
    `/etc/zfs/zfs-list.cache/<pool>` exists and lists the checkout dataset.
    That unit has a fragment, so `Requires=` and `After=` both work on it,
    and systemd mounts the checkout as that unit's own job.
    Its ordering is the generator's: `After=zfs-import.target` and the key-loading unit,
    `Before=zfs-mount.service`.
2.  Otherwise the provider is the unit that runs `zfs mount -a` at boot: `zfs-mount.service` in every
    OpenZFS packaging this route's records name.
    It mounts the checkout as a side effect of one command over every dataset, so the route names
    the actor and not a mount unit, because a mount unit appears only after the actor ran.
3.  Where neither is enabled, nothing mounts the checkout at boot, and nothing loads the checkout's
    key unless the pool's import does.
    The route refuses at boot and names what is missing; `doctor` reports which of the three states
    the machine is in, and whether a key-loading unit exists.

Two consequences a reader must carry:

- A `Requires=` on the unit that runs `zfs mount -a` is a **pull-in and a dependency, not a proof of
  a mount**. The unit carries `ConditionPathIsDirectory=/sys/module/zfs`, so where the module is not
  loaded the start is a condition-skip that counts as success and mounts nothing
  (source reading of the unit text in `keep/host-reads.log`, `systemctl cat zfs-mount.service`).
- A mount unit systemd learns from the mount table cannot be required, only ordered.

## The provider's status is not the checkout's status

This section replaces a control that read a provider exit status one as a checkout failure.
It is not one, and a boot where the descendants carry `canmount=noauto` never produces it.

Measured, this host's shape (round four's `evidence-host-shape.log`, boot
`773296e0-6fea-4ee0-94c1-1e486e063ce4`): the checkout's dataset has
`canmount=on`, its key is unavailable, no key-loading unit exists for it, and the dataset under the
path carries `canmount=noauto`, so nothing loads the key and `zfs mount -a` skips both. The provider's
own result, from `systemctl show` in the same boot:

```text
Id=zfs-mount.service
ActiveState=active
CanReload=no
Result=success
ExecMainStatus=0
```

What the boot then shows:

```text
SWEEP start 2026-11-24T12:09:24.202Z mode=boot-reconcile uptime=6.21
SWEEP checkout-row=zfsstep19/home zfs
SWEEP refused=checkout-not-the-dataset 2026-11-24T12:09:24.209Z
1795522164.523793 zfs-step-19-fourth-2026-11-24 systemd[1]: Starting systemd-user-sessions.service - Permit User Sessions...
```

Three things are true at once, and the design says all three:

- the provider can finish green with the checkout absent, because `zfs mount -a` treats an
  unavailable key as a dataset to skip and a `noauto` dataset as one to leave alone
  (source reading, `cmd/zfs/zfs_main.c`, `share_mount_one`'s key and `canmount` branches, both gated
  on `explicit`);
- the checkout-specific failure arrives through the key unit where one exists, and through the gate's
  own refusal where none does;
- nothing was written into the covered entry: the home underlay kept its inventory
  (same log, "the home underlay's inventory").

The same green provider over a checkout that cannot mount is not a property of the provider; it is a
property of what it is asked to mount, and the hazard section measures the other answer.

Measured the other way, with a key unit present and its key file away
(round three's `evidence-key-removed.log`, boot `1010ab84-5d06-4ede-8f5d-fd9880c6b700`):

```text
1795507758.973672 zfs-step-19-third-2026-11-24 systemd[1]: zfs-load-key@zfsstep19-home-monochromatic\x2dgit\x2dcheckout.service: Main process exited, code=exited, status=255/EXCEPTION
1795507758.976797 zfs-step-19-third-2026-11-24 systemd[1]: Dependency failed for reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts.
1795507758.983254 zfs-step-19-third-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Job reconcile-git-worktree-clone-mounts.service/start failed with result 'dependency'.
1795507760.676330 zfs-step-19-third-2026-11-24 systemd[1]: Starting systemd-user-sessions.service - Permit User Sessions...
```

The provider in that same boot finished `Result=success`, `ExecMainStatus=0`, which is the whole
point of the section.

## The key-loading unit, and what happens where there is none

The unit to name is the generator's own: `zfs-load-key@<escaped encryption root>.service`,
written by `zfs-mount-generator` beside the mount unit it serves
(source reading, `zfs-2.4.4` `etc/systemd/system-generators/zfs-mount-generator.c`, the block that
builds `keyloadunit` with `systemd_escape(p_encroot, "zfs-load-key@", ".service")`).
Its whole shape, as the generator writes it:

```ini
[Unit]
Description=Load ZFS key for <dataset>
SourcePath=/etc/zfs/zfs-list.cache/<pool>
Documentation=man:zfs-mount-generator(8)
DefaultDependencies=no
Wants=zfs-import.target
After=zfs-import.target

[Service]
Type=oneshot
RemainAfterExit=yes
StandardOutput=null
StandardError=null
ExecStart=/bin/sh -euc '[ "$(<zfs> get -H -o value keystatus "<dataset>")" = "unavailable" ] || exit 0; exec <zfs> load-key "<dataset>"'
ExecStop=/bin/sh -euc '[ "$(<zfs> get -H -o value keystatus "<dataset>")" = "available" ] || exit 0; exec <zfs> unload-key "<dataset>"'
```

It has no `[Install]` section, and the `Before=zfs-mount.service` line belongs to the **mount** unit
the generator writes beside it, not to the key unit (F39).

Two consequences:

- Where such a unit exists, the boot unit requires and orders after it:
  `Requires=zfs-load-key@<escaped encryption root>.service` and the same in `After=`, with the name
  taken from the checkout dataset's own encryption root where it has one and from the home dataset's
  where the checkout inherits.
  Measured: with the key file away, the key unit failed and the gate's start job failed with
  `dependency` before its `ExecStart` ran (round three's `evidence-key-removed.log`).
- Where none exists, nothing loads a key at boot except the pool's import.
  That is this host's state, and the measured consequence is the gate's refusal
  `checkout-not-the-dataset` (round four's `evidence-host-shape.log`).
  `doctor` reports that no key-loading unit exists and that the checkout's key must be loaded before
  the provider runs.

Round three's guest wrote two key units by hand, in the generator's names but not in its shape:
they carried `Before=zfs-mount.service` and `[Install] WantedBy=zfs.target`, which the generator's
does not, so the failure arm's provider started after the gate's job had already failed
(round three's `units-install.log:31`, `:41`; its `evidence-key-removed.log:103`,
`1795507758.983254`, against the key unit's exit at `1795507758.973672`).
The failure that arm measures is the requirement's, which is what the design needs; the shape is the
guest's and is not evidence about the generator's unit.

## The units, whole

The files this design installs are the boot unit, one unit per dataset whose mountpoint lies inside
the checkout path, the account's consumers, the propagation service, the provider's drop-in, the
propagation drop-in, the stop-failure drop-in and the ZED hook.
Two values are the owner's numbers, `TimeoutStartSec=5min` and `TimeoutStopSec=1min`, which plan step 22
replaces with measured ceilings (V8b).
Every placeholder the unit texts carry is named where its text is, and the two that belong to another
design are these:

- `<root-file>` is the root-owned regular file the modes are installed as, which the root-modes
  design's choice 3 fixes: one file under the ZED hook's own name (its option A), one file named for
  the modes (option B), one file named for the wrapper (option C), or a copy or hard link per name
  (option D).
- `<root-words>` is the **two words** the units spell before the mode word:

  ```text
  monochromatic-git-policy-cli zfs-root
  ```

  The start rule is the root-modes design's choice 1, option A, and the word itself is its choice 2,
  whose constant is `ROOT_GROUP_WORD`
  (source reading,
  `doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`,
  "Choice 1" option A and `src/native/worktree_clone_mount_handler/root_mode_names.rs` in that
  design's module text).
  A single group word is not enough: `handler_mode_arguments.rs` reads one word as the mode word, so
  `<root-file> zfs-root boot-reconcile` is refused with status 2 as an unknown mode, and on a file
  whose name rule does not read the words, the same line would run the wrapper as Git with root.

Every unit line in this document holds under any answer to those choices:
the root-modes design drops the words when the start is the hook's own name and reads them otherwise,
and this design spells both the file and the words.
The placeholders this design owns are the checkout path (`/home/user/Monochromatic`, V3), the nested
dataset and its mountpoint (V4 and V9), the escaped encryption root behind the two key lines, and the
owner's two bounds.

### The boot and shutdown unit

This is the gate the owner's answers write: `Wants=` for the provider (V2), the one
`RequiresMountsFor=` line (V3), no before-login ordering (V8a), and the owner's two bounds (V8b).

```ini
# /etc/systemd/system/reconcile-git-worktree-clone-mounts.service
[Unit]
Description=Reconcile the managed worktree clone mounts
Requires=zfs-load-key@<escaped encryption root>.service
After=zfs-load-key@<escaped encryption root>.service
Wants=zfs-mount.service
After=zfs-mount.service zfs-import.target
RequiresMountsFor=/home/user/Monochromatic
Before=umount.target
Conflicts=umount.target

[Service]
Type=oneshot
RemainAfterExit=yes
TimeoutStartSec=5min
TimeoutStopSec=1min
ExecStartPre=-<root-file> <root-words> start-probe
ExecStart=<root-file> <root-words> boot-reconcile
ExecReload=<root-file> <root-words> boot-reconcile
ExecStopPost=-<root-file> <root-words> shutdown-unmount

[Install]
WantedBy=multi-user.target
```

The placeholders, each owned where it says:

- `<escaped encryption root>` is the checkout dataset's encryption root, or the home dataset's where
  the checkout inherits one, escaped as `systemctl` prints it; the two key lines are present only on a
  machine that has such a unit, as the next block shows.
- `/home/user/Monochromatic` is the checkout path, which V3's line names as this host's.
- `<root-file>` is the root-owned regular file the modes are installed as, fixed by the root-modes
  design's choice 3 (one file under the ZED hook's own name, one named for the modes, one named for the
  wrapper, or a copy or hard link per name).
- `<root-words>` is the two words the units spell before the mode word,
  `monochromatic-git-policy-cli zfs-root`, which is that design's choice 1 option A and its choice 2's
  constant `ROOT_GROUP_WORD`
  (source reading,
  `doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`,
  "Choice 1" option A and `root_mode_names.rs` in that design's module text).
- `5min` and `1min` are the owner's numbers from
  `doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`, "V8, split in two",
  and plan step 22 replaces them with measured ceilings.

**The two key lines are conditional.** A machine with no key-loading unit must not carry them, because
a unit that requires a unit the boot does not install stays `inactive` with `Result=success` and no
journal line at all (F31), which is a gate that silently never runs.
This host's unit, the shape such a machine installs, is the same text with those two lines removed, and
it ran on round four's host-shape boot, where the gate refused the checkout by its own rule
(round four's `state-host-shape.log`, `evidence-host-shape.log`):

```ini
# /etc/systemd/system/reconcile-git-worktree-clone-mounts.service, on a machine with no key-loading unit
[Unit]
Description=Reconcile the managed worktree clone mounts
Wants=zfs-mount.service
After=zfs-mount.service zfs-import.target
RequiresMountsFor=/home/user/Monochromatic
Before=umount.target
Conflicts=umount.target

[Service]
Type=oneshot
RemainAfterExit=yes
TimeoutStartSec=5min
TimeoutStopSec=1min
ExecStartPre=-<root-file> <root-words> start-probe
ExecStart=<root-file> <root-words> boot-reconcile
ExecReload=<root-file> <root-words> boot-reconcile
ExecStopPost=-<root-file> <root-words> shutdown-unmount

[Install]
WantedBy=multi-user.target
```

**This text, with the placeholders filled, the stop-failure drop-in beside it and the two key lines
present, is what booted green in this round's guest.** The bounds stood at the owner's values, and the
only substitutions are the stand-in program's name where `<root-file> <root-words>` stands, its
argument, which is the mode word, and the guest's own paths
(kept logs `state-chosen-green.log`, `evidence-chosen-green.log`, boot
`d611cc4c-27da-4607-91c9-38e47140c8f1`).
That boot's order and the values systemd read, one contiguous run of its journal with the unit's own
`systemctl show` output appended:

```text
1795545978.001546 zfs-step-19-chosen-2026-11-24 systemd[1]: Starting zfs-mount.service - Mount ZFS filesystems...
1795545978.062661 zfs-step-19-chosen-2026-11-24 systemd[1]: Finished zfs-mount.service - Mount ZFS filesystems.
1795545978.824420 zfs-step-19-chosen-2026-11-24 systemd[1]: Starting reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts...
1795545978.882730 zfs-step-19-chosen-2026-11-24 systemd[1]: Finished reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts.
1795545978.886534 zfs-step-19-chosen-2026-11-24 systemd[1]: Starting git-account-session-standin.service - A service of the account that needs the checkout...
1795545978.898076 zfs-step-19-chosen-2026-11-24 systemd[1]: Starting git-worktree-nested-output-mount.service - Mount zfsstep19/nosnap/vm-builder-output under the verified checkout...
1795545978.912087 zfs-step-19-chosen-2026-11-24 systemd[1]: Finished git-account-session-standin.service - A service of the account that needs the checkout.
1795545978.925234 zfs-step-19-chosen-2026-11-24 systemd[1]: Finished git-worktree-nested-output-mount.service - Mount zfsstep19/nosnap/vm-builder-output under the verified checkout.
1795545978.928715 zfs-step-19-chosen-2026-11-24 systemd[1]: Starting git-vm-builder-standin.service - A vm-builder service of the account that needs the dataset under the checkout...
1795545978.941876 zfs-step-19-chosen-2026-11-24 systemd[1]: Finished git-vm-builder-standin.service - A vm-builder service of the account that needs the dataset under the checkout.
1795545979.363744 zfs-step-19-chosen-2026-11-24 systemd[1]: Starting systemd-user-sessions.service - Permit User Sessions...
1795545979.371295 zfs-step-19-chosen-2026-11-24 systemd[1]: Finished systemd-user-sessions.service - Permit User Sessions.

TimeoutStartUSec=5min
TimeoutStopUSec=1min
TimeoutStopFailureMode=terminate
DropInPaths=/usr/lib/systemd/system/service.d/10-timeout-abort.conf /etc/systemd/system/reconcile-git-worktree-clone-mounts.service.d/zz-checkout-stop-failure-mode.conf
Wants=zfs-mount.service
```

Nothing is omitted from that run.
Sessions start last here because the gate is fast, not because anything orders them after it: the
stages arm under "The failure controls" shows sessions starting while the gate is still running.

### The stop-failure drop-in the unit needs beside it

The line that decides what a bound does when it fires cannot live in the unit file, and it also
cannot live in a drop-in whose file name sorts before the distribution's own.
Every drop-in is parsed in one order sorted by file name, so Fedora's
`/usr/lib/systemd/system/service.d/10-timeout-abort.conf` wins against a `10-` file of this design's
and loses against a `zz-` file (measured four ways, kept log `stopmode-shapes.log`,
quoted under this round's second finding).
So the shape this design installs is the unit text in "The boot and shutdown unit" plus one drop-in:

```ini
# /etc/systemd/system/reconcile-git-worktree-clone-mounts.service.d/zz-checkout-stop-failure-mode.conf
[Service]
TimeoutStopFailureMode=terminate
```

The name is load-bearing, so V7's report reads the effective value back with `systemctl show`
rather than trusting the file.

**This text, with the placeholders filled and that drop-in beside it, is what booted green.**
The two bounds stood at this round's test values, `TimeoutStartSec=20` and `TimeoutStopSec=5`,
because no measurement of one reconcile or one stop pass exists yet; and the only substitutions this
design makes for the stand-ins are the stand-in program's name where `<root-file> <root-words>`
stands and its argument, which is the mode word
(kept logs `state-green.log`, `evidence-green.log`, boot
`de767eea-e7a9-4c4d-ad6a-e4a97803507b`).
That boot's order and the value systemd read, one contiguous run of its journal with this boot's own
`systemctl show` output appended:

```text
1795529986.176303 zfs-step-19-fifth-2026-11-24 systemd[1]: Starting checkout-propagation.service - Ask the boot unit to re-verify the checkout's private propagation...
1795529986.185462 zfs-step-19-fifth-2026-11-24 systemd[1]: Starting reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts...
1795529986.206818 zfs-step-19-fifth-2026-11-24 systemd[1]: Finished checkout-propagation.service - Ask the boot unit to re-verify the checkout's private propagation.
1795529986.223586 zfs-step-19-fifth-2026-11-24 systemd[1]: Finished reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts.
1795529986.226087 zfs-step-19-fifth-2026-11-24 systemd[1]: Starting git-account-session-standin.service - A service of the account that needs the checkout...
1795529986.230947 zfs-step-19-fifth-2026-11-24 systemd[1]: Starting git-worktree-nested-output-mount.service - Mount a dataset whose mountpoint lies inside the verified checkout...
1795529986.241818 zfs-step-19-fifth-2026-11-24 systemd[1]: Finished git-account-session-standin.service - A service of the account that needs the checkout.
1795529986.262900 zfs-step-19-fifth-2026-11-24 systemd[1]: Finished git-worktree-nested-output-mount.service - Mount a dataset whose mountpoint lies inside the verified checkout.
1795529986.265039 zfs-step-19-fifth-2026-11-24 systemd[1]: Starting git-vm-builder-standin.service - A vm-builder service of the account that needs the dataset under the checkout...
1795529986.276250 zfs-step-19-fifth-2026-11-24 systemd[1]: Finished git-vm-builder-standin.service - A vm-builder service of the account that needs the dataset under the checkout.
1795529986.582386 zfs-step-19-fifth-2026-11-24 systemd[1]: Starting systemd-user-sessions.service - Permit User Sessions...
1795529986.589728 zfs-step-19-fifth-2026-11-24 systemd[1]: Finished systemd-user-sessions.service - Permit User Sessions.

DropInPaths=/usr/lib/systemd/system/service.d/10-timeout-abort.conf /etc/systemd/system/reconcile-git-worktree-clone-mounts.service.d/zz-checkout-stop-failure-mode.conf
TimeoutStartUSec=20s
TimeoutStopUSec=5s
TimeoutStopFailureMode=terminate
```

Nothing is omitted from that run: the account consumer and the per-dataset unit start together after
the gate, because the first requires the gate and the second requires the per-dataset unit, and
sessions start last.

### Why each line

- `Requires=zfs-load-key@<escaped encryption root>.service` and its `After=`.
  The prerequisite whose failure is the checkout's, on a machine that has one.
  Measured: with the key file away, the key unit failed and this unit's start job failed with
  `dependency` before its `ExecStart` ran, and so did both account consumers and the per-dataset
  unit (round three's `evidence-key-removed.log`, journal lines `1795507758.973672` to
  `1795507758.983254`).
  Where there is no such unit these two lines are absent, as the second unit text in
  "The boot and shutdown unit" shows.
- `Wants=zfs-mount.service` and `After=zfs-mount.service zfs-import.target`, the owner's V2 answer.
  What `Wants=` buys is a pull-in and the ordering, and what it gives up is the dependency failure and
  the restart propagation `Requires=` had: measured, a provider restart leaves the gate active and the
  drop-in reloads it (kept log `restart-provider-chosen.log`), and a provider failure leaves the gate
  to run and finish green beside it (kept log `evidence-chosen-provider-fails.log`).
  The pull-in is weaker than it reads: the provider's
  `ConditionPathIsDirectory=/sys/module/zfs` turns a start without the module into a condition skip
  that succeeds and mounts nothing (source reading of the unit text, `keep/host-reads.log`), and
  where a dataset under the checkout path cannot be mounted the provider fails the whole boot's
  mount, which this unit's requirement then carries as a `dependency` failure instead of the unit's
  own refusal (round four's `evidence-hazard-flag-on.log`, quoted under "The failure controls").
  **That price was `Requires=`'s, and the owner's answer pays it no more.** Measured with
  `Requires=`, a provider restart took this unit, the nested unit and both consumers through their stop
  pass and started them again (round three's `restart-provider.log`, `systemd-262-arms.log` arm F;
  round five's `restart-provider-v9c.log`).
  At a boot the ordering edge is not what starts this unit: the brown-out arm of round two ran with
  no edge to the provider at all and this unit still started after it, because the propagation
  service's request started it (round two's `evidence-no-provider-edge.log:93` to `:96`:
  `Starting checkout-propagation.service` at `1795503075.469533`, `Starting reconcile-…` at
  `1795503075.501653`).
  Replace the name with the generated mount unit where the generator writes one; keep the
  `zfs-import.target` edge either way.
- `RequiresMountsFor=<checkout path>`, the owner's V3 answer.
  This replaces the two hand-written ordering lines.
  Measured on the chosen text in this round's guest: one line put `After=` on `-.mount` and on the home
  dataset's learned mount unit, and `Requires=` only on the one of them that has a fragment
  (kept log `state-chosen-green.log`, "the dependencies systemd derived"):

  ```text
  Requires=-.mount sysinit.target "zfs-load-key@zfsstep19-home-monochromatic\\x2dgit\\x2dcheckout.service" system.slice
  After="srv-zfs\\x2dstep\\x2d19\\x2dfixture-home.mount" systemd-journald.socket system.slice basic.target zfs-import.target zfs-mount.service "zfs-load-key@zfsstep19-home-monochromatic\\x2dgit\\x2dcheckout.service" -.mount sysinit.target
  ```

  (and round three's `systemd-262-arms.log` arm G for the same line on 262.2).
  The provider's name is in the `After=` and in `Wants=`, not in `Requires=`, which is what V2's answer
  asks for.
  At the moment the unit loads, the checkout's own learned unit may not exist yet, so that edge
  orders the stop half rather than the first start; what orders the first start is the provider's edge
  and `zfs-import.target`.
  What lands differs by machine and this host is the other shape: its root mount unit has an empty
  `FragmentPath`, so `Requires=` does not attach to it there and the line would leave an `After=` only
  (host read, `keep/host-reads.log`; the same rule V3 gives).
- **No `Before=systemd-user-sessions.service`, which is the owner's V8a answer.**
  The line is absent from this unit, from the nested dataset's unit and from both consumers, so a
  login never waits for the gate. Measured: with a hung pass, sessions started 0.44 s after the gate
  and 20.4 s before the gate failed (kept log `evidence-chosen-stages.log`), and in the green boot the
  gate had already finished when sessions started (`evidence-chosen-green.log`).
  The cost is that a session can begin with no checkout, which "What a hung start costs" prices.
- `Before=umount.target` with `Conflicts=umount.target`.
  The stop half. Measured at runtime on 258 through a *gate* restart, which took the units down in
  reverse order: the consumers, then the nested unit, then the gate (kept log
  `restart-gate-chosen.log`), and through the `Requires=` provider restart of round three, which did the
  same (round three's `restart-provider.log`:
  `Stopping git-account-session-standin.service` at `1795507731.282283`,
  `Stopping reconcile-…` at `1795507731.284386`,
  `SWEEP stopped … stopped-slot=unmounted`).
  A shutdown arm was not run this round; round one's shutdown order on systemd 255 is in its
  `keep/guest-logs/journals-per-boot.log`, lines 83 to 90.
- `TimeoutStartSec=5min` and `TimeoutStopSec=1min`, the owner's V8b numbers, which step 22 replaces
  with measured ceilings.
  What the pair buys, measured with the chosen text and its bounds scaled to 8 s and 4 s inside the
  arm only: the start bound fired at `1795546185.553034`, the stop phase spent its bound once per stage
  a program survived, and the unit failed at `1795546198.303748`, 20.86 s after its start
  (kept log `evidence-chosen-stages.log`).
  The bound is per phase, so the gate's own failure can last the start bound plus one stop bound per
  stop stage that survives, which "What a hung start costs" prices; a login no longer waits for it.
- The drop-in `zz-checkout-stop-failure-mode.conf` with `TimeoutStopFailureMode=terminate`.
  It exists because of a measured difference between machines: a machine whose packaging sets
  `TimeoutStopFailureMode=abort` sends SIGABRT to a stop stage that overruns and leaves a coredump.
  Fedora 43 does exactly that, and round four's bound-stop arm holds
  `State 'stop-post' timed out. Aborting.`, two `Killing process … with signal SIGABRT` lines and two
  `systemd-coredump` dumps (round four's `evidence-bound-stop.log`).
  With the drop-in in force, this round's stages arm ended every stage in SIGKILL instead: no
  `Aborting` line, no SIGABRT kill and no coredump anywhere in that boot
  (kept log `evidence-stages.log`).
  This host's own systemd has no `service.d/10-timeout-abort.conf` at either path
  (`ls` in `keep/host-reads.log`), so here the drop-in restates the default, and on Fedora's shape it
  is what keeps the default.
- `ExecStartPre=-<root-file> <root-words> start-probe`.
  The start-probe design's placement, adopted here because plan step 19 installs the unit: one root
  process per boot, in the service manager's own child, before the reconcile.
  The probe's own deadline is built and bounds the probe: `PROBE_DEADLINE` is 350 ms with a 231 ms
  margin over the slowest measured worker phase
  (source reading, `package/git-policy/cli/src/native/start_probe/probe_deadline.rs`, the two
  constants and the measurement its doc comment cites). So the deferral round two recorded is gone and the line
  ships with the unit.
  The dash prefix is load-bearing and measured: a failing `ExecStartPre=` under the prefix left the
  unit `active` and ran its `ExecStart`, the same line without the prefix failed the unit and skipped
  it, and a timeout inside `ExecStartPre=` sent the unit to its stop action whatever the prefix said
  (round two's `start-bound.log`, three units, each read back).
  The unit's bound is a second, larger bound over the same phase, not the probe's deadline.
- `ExecStart=<root-file> <root-words> boot-reconcile`.
  The pass itself: it verifies the checkout's mount identity, the covered entry's flag and the private
  propagation, then attaches what the intent asks for.
- `ExecReload=<root-file> <root-words> boot-reconcile`.
  The line the lost-event record accepts and the propagation design's printed lever needs.
  Measured in round two: `systemctl reload-or-restart` on the active unit ran this command and no stop
  pass (round two's `reload-gate.log`, `Reloading reconcile-…` at `1795503559.285343`, `Reloaded` at
  `1795503559.315252`).
  A reload re-runs the reconcile on an active checkout, so the mode must be idempotent; the stand-in
  was made so after a first run showed a second bind of the same slot.
- `ExecStopPost=-<root-file> <root-words> shutdown-unmount`.
  The stop pass, and the dash prefix is this design's answer to the round one review's M1.
  The built `shutdown-unmount` always refuses
  (`package/git-policy/cli/src/native/worktree_clone_mount_handler/shutdown_unmount_mode.rs`,
  `run` returning `AWAITING`), so without the prefix every stop fails the unit for a mode whose step
  has not landed.
  With the prefix the refusal is logged and the stop stays clean, and the line loses the prefix when
  the unmount half lands.
- `WantedBy=multi-user.target` with default dependencies.
  This is what makes the unit part of the boot at all, and it carries the implicit `After=basic.target`.

### What a stop does today

Two source readings of `main`:

- `shutdown-unmount` refuses, so the stop line carries `-` until that mode lands.
- `boot-reconcile` exits **zero** when individual clones fail: `boot_reconcile_mode.rs` publishes a
  readiness record with `SweepState::Failed` and still returns `Ok`
  (`boot_reconcile_mode.rs:158` to `:164`), and only a refused prerequisite returns `Err`.
  So a consumer's `Requires=` on this unit carries a boot refusal and not a per-clone sweep failure.
  A clone's failure reaches a consumer through the readiness record, not through the unit's result.

### The nested dataset's own unit

V4 and V9's answer: `canmount=noauto` on every dataset whose mountpoint lies inside the checkout, and
one unit per such dataset, ordered after the gate, which runs `/usr/bin/zfs mount` now and the
wrapper's `mount-path-descendant` mode once it exists.
On this host the list is one dataset, `zpcachyos/nosnap/vm-builder-output`, mounted at
`/home/user/Monochromatic/package/vm-builder/cli/output` (this round's host read, and V4's).

```ini
# /etc/systemd/system/git-worktree-nested-<slot-or-name>-mount.service
[Unit]
Description=Mount <dataset> under the verified checkout
Requires=reconcile-git-worktree-clone-mounts.service
After=reconcile-git-worktree-clone-mounts.service
ConditionPathIsMountPoint=!<the dataset's own mountpoint>

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/usr/bin/zfs mount <dataset>

[Install]
WantedBy=multi-user.target
```

Every placeholder: `<slot-or-name>` and `<dataset>` are the dataset the unit serves, and
`<the dataset's own mountpoint>` is the path its `mountpoint` property names, which `doctor` reports
and setup writes from the same list the unit is generated from.

**The condition is what makes the unit tolerate an already-mounted dataset.** `zfs mount` of a mounted
dataset prints `filesystem already mounted` and returns 1, which the source gates on `explicit`
(source reading, `zfs-2.4.4` `cmd/zfs/zfs_main.c`, the `OP_MOUNT` branch: `if (!explicit) return (0);`
then `filesystem already mounted` and `return (1)`).
`ConditionPathIsMountPoint=!<path>` skips the unit's start instead, and a skipped unit counts as
success for a consumer that requires it.
Measured three ways in this round's guest:

- **at a boot where the provider mounted the dataset first** (`canmount=on`, the shape a machine has
  before the owner's property change): the unit is skipped and its consumer still runs

  ```text
  1795546081.684890 zfs-step-19-chosen-2026-11-24 systemd[1]: git-worktree-nested-output-mount.service - Mount zfsstep19/nosnap/vm-builder-output under the verified checkout skipped, unmet condition check ConditionPathIsMountPoint=!/srv/zfs-step-19-fixture/home/account/Monochromatic/package/vm-builder/cli/output
  ```

  with `ActiveState=inactive`, `ConditionResult=no`, no invocation id at all, and the dataset mounted
  (kept log `evidence-chosen-mounted-first.log`, boot `fc7d3967-e3fc-46c1-a85c-1d191020f932`);
- **at a restart of the gate**, which this unit requires and which therefore restarts it: the same
  skip, the consumer runs, and the dataset stays mounted (kept log `restart-gate-chosen.log`);
- **in the green boot**, where the dataset is not mounted: the condition passes and the unit mounts it,
  `ActiveState=active`, `Result=success` (kept log `evidence-chosen-green.log`).

That closes the failure the fifth revision measured: a provider restart used to restart this unit
through `Requires=` and fail it with `filesystem already mounted` (round five's
`restart-provider-v9c.log`). Under the owner's `Wants=` the unit is not restarted by a provider restart
at all, and where something does restart it the condition skips it.

**When the mode replaces the explicit mount**, this unit's `[Unit]` block stays as it is and only
`ExecStart=` changes to `<root-file> <root-words> mount-path-descendant --dataset <dataset>`.
The mode must then carry two things the explicit call cannot:

- **idempotence on an already-mounted dataset**: exit 0 and print what it found.
  A measured line holds the message a naive implementation returns, on the same dataset
  (round two's `propagation.log:99`:
  `cannot mount 'zfsstep19/home/monochromatic-git-checkout': filesystem already mounted`).
- **a refusal for a dataset whose mountpoint lies outside the checkout**: `dataset-outside-the-checkout`,
  a name nothing else uses.
  It is a new name because `checkout-not-the-dataset` already means something else here: the checkout's
  own mount row is not the checkout dataset.
  That refusal must name the dataset it refused, not only the path.

And it contradicts a principle the grammar states, so the mode is a question for the root-modes design
rather than only an addition to a list:

> a mode is chosen by a leading word and nothing else, and no argument names a folder to mount or a
> dataset to change

(source reading, `handler_mode_arguments.rs:112` to `:115`, the `HandlerMode` doc).
That sentence's reach is narrower than it reads: `holder-scan` takes `--dataset` today, so an argument
may name a dataset; what the sentence forbids is an argument that makes the wrapper **change** a
dataset's state, which `mount-path-descendant --dataset` would do.
The root-modes design owns that answer.

**The unit is unbounded, and that is now cheap.** systemd reports `TimeoutStartUSec=infinity` and
`TimeoutStopUSec=45s` for it (`state-chosen-green.log`), so a hung `zfs mount` is reaped only by a
person; with no before-login line (V8a) it no longer holds a login, only the services that require it.
The chosen bounds (V8b) are the gate's, and neither answer gives this unit one; a later revision may
size it, and the same is true of the consumers below.

### The account's consumers

V6's answer: any service that reads the checkout carries `Requires=` and `After=` on the gate, or on
one nested dataset's unit. The two stand-ins this design measured are the ones its own arms use:

```ini
# /etc/systemd/system/git-account-session-standin.service
[Unit]
Description=A service of the account that needs the checkout
Requires=reconcile-git-worktree-clone-mounts.service
After=reconcile-git-worktree-clone-mounts.service

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=<the service's own command>

[Install]
WantedBy=multi-user.target
```

```ini
# /etc/systemd/system/git-vm-builder-standin.service
[Unit]
Description=A vm-builder service of the account that needs the dataset under the checkout
Requires=git-worktree-nested-output-mount.service
After=git-worktree-nested-output-mount.service

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=<the service's own command>

[Install]
WantedBy=multi-user.target
```

Neither carries `Before=systemd-user-sessions.service` (V8a), and neither carries a start bound.
Measured in the green boot: the checkout consumer finished at `1795545978.912087` after the gate at
`1795545978.882730`, and the nested consumer at `1795545978.941876` after its unit at
`1795545978.925234` (kept log `evidence-chosen-green.log`).

A consumer that names a unit the boot does not install gets nothing at all, silently: the fifth
revision's and round three's arms hold that (`evidence-v9b.log`).

## The provider drop-in, from the lost-event record

```ini
# /etc/systemd/system/zfs-mount.service.d/reconcile-git-worktree-clone-mounts-after-checkout.conf
[Service]
ExecStartPost=-/usr/bin/systemctl --no-block --no-ask-password reload-or-restart reconcile-git-worktree-clone-mounts.service
```

Measured in round two on systemd 258, with the gate's requirement written as `Wants=` so the restart
would not move it: after `systemctl restart zfs-mount.service` the gate stayed active and then
`Reloading reconcile-…` and `Reloaded` appeared (round two's `wants-control.log`, journal lines
`1795503693.546036` and `1795503693.566525`), and the `-` prefix kept the provider `active` with
`Result=success`.
With the gate on `Requires=`, the same restart already restarted the gate and that boot's journal
holds no separate reload (round two's `dropin.log`), which is the whole of that result.
The install is the record's, and the record's reason stands: the request is what reconciles after a
post-key-load checkout mount, including a provider started by something other than the boot.

## The propagation service and its drop-in, from the propagation record

```ini
# /etc/systemd/system/checkout-propagation.service
[Unit]
Description=Ask the boot unit to re-verify the checkout's private propagation
BindsTo=<escaped checkout path>.mount
After=<escaped checkout path>.mount

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/usr/bin/systemctl --no-block --no-ask-password reload-or-restart reconcile-git-worktree-clone-mounts.service
```

```ini
# /etc/systemd/system/<escaped checkout path>.mount.d/10-upholds-propagation.conf
[Unit]
Upholds=checkout-propagation.service
```

Measured on systemd 258 and again on 262.2:

- The drop-in is read for a mount unit systemd learned: `systemctl show` reports the `Upholds=` and
  names the drop-in in `DropInPaths`, and the unit's `FragmentPath` is empty with
  `SourcePath=/proc/self/mountinfo` (round two's `dependencies-read.log`; round three's
  `upholds-arm.log`).
- `Upholds=` starts the service when the mount unit activates, and `BindsTo=` stops it when the mount
  unit deactivates: on 262.2 the service started at `1795508243.805597` and was stopped at
  `1795508245.851061`, while the consumer the service's request had started stayed active
  (round three's `upholds-arm.log`).
- The service starts before the gate on a boot where both are wanted: in this round's green boot the
  service started at `1795529986.176303` and the gate at `1795529986.185462`, because the learned
  unit's activation submits the upheld service immediately (kept log `evidence-green.log`).

The escaped name must be spelled as `systemctl` prints it, backslashes and all; doubling the
backslashes names a unit that does not exist (measured, round three's `escaping-arm.log`).
`systemd-analyze verify` reports one line for this pair, and it is an artifact of verify, which has no
live manager to find a learned unit:

```text
checkout-propagation.service: Failed to create checkout-propagation.service/start:
Unit <escaped checkout path>.mount not found.
```

So `doctor`'s list of known lines must carry it rather than treat it as a defect, and a reader must
not conclude from it that the dependency is broken: the same spelling resolved once the unit was
loaded (round three's `escaping-arm.log`, `upholds-arm.log`).

Neither drop-in is installed without the boot unit's `ExecReload=`, which is the propagation record's
own condition: without that line the same request is a restart, which runs the stop pass and unmounts
every attached clone.

## The ZED hook, and what it waits on

The hook is **the root-owned regular file under a ZED-legal name**, and nothing else.

```text
/usr/libexec/monochromatic-git-policy-cli/<the installed file>   regular, 0755, root
/etc/zfs/zed.d/history_event-<hook-name>                          symlink to the file
```

- ZED starts a zedlet by its file name, with no arguments: `execle(path, prog, NULL, env)`
  (source reading, `zfs-2.4.4` `cmd/zed/zed_exec.c:168`), with the environment built in
  `zed_event.c`'s block that fills it before the exec.
- A zedlet's name must carry the zevent class as a prefix followed by a non-alphabetic character
  (`zed.8`); `history_event-…` satisfies that.
- No argument is the attaching sweep: `handler_mode_arguments.rs`'s `parse` returns
  `HandlerMode::AttachingSweep` for an empty argument list.
  There is no `sweep` word in the grammar, and this design writes none.
- No script is installed. The earlier text's `exec <root-mode-entry> <sweep>` file is deleted with its
  justification: "every zedlet here is a `.sh` file" described the packaged zedlets, not a
  requirement of ZED's, and the one-program record retires an installed program of the route's own.

What each part waits on in the root-modes design, by whose choice:

- the file's own name, and whether the file is named for the hook or for the modes: choice 3;
- the hook name's exact spelling, which is the source constant
  `ZEDLET_INSTALL_PATH = "/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets"`
  (`handler_install_paths.rs:38`, read in `keep/host-reads.log`) unless that design renames it;
  it is a constant of `main`, not a decision a record made;
- the two words in every unit line: choice 2, whose constant this design spells;
- whether a root start under another name runs the wrapper: choice 4's placement;
- the words `--version` and the refusals print: choice 5, which only `doctor`'s reading depends on;
- the AppArmor rules, which the owner's answer places in `/etc/apparmor.d/local/zed` and the next
  section writes out;
- the mode `mount-path-descendant`, which the nested dataset's unit runs once that mode exists.

Three things on this host have to be true before the hook runs at all, and none is a design choice:

- `zfs-zed.service` is `disabled` and `inactive` here (F3), so no zedlet runs today;
- the `zed` profile's zedlet exec rule names a folder this host does not have, so even the packaged
  zedlets have no exec grant under it if it is enforced;
- the installed file would need an exec grant, which the owner's answer gives it as a rule in
  `/etc/apparmor.d/local/zed`, and nothing applies that rule until the owner enables ZED.

## AppArmor on this host: the hook's rules, as the owner chose

**The answer is "rules in `local/zed`"**: the hook's needs are added to
`/etc/apparmor.d/local/zed`, which the `zed` profile includes if it exists, and every packaged zedlet
inherits them.
The record carries the owner's acceptance of that cost:
`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`,
"The ZED hook's AppArmor permissions".

Nothing applies the grant yet: `zfs-zed.service` is `disabled` and `inactive` here (F3), so no zedlet
runs and no rule is needed until the owner enables ZED.
What the rules are, and why each is needed, is stated under "The rules to add, and the reading each
rests on".
The shapes the owner did not choose are kept, with their own costs, in "The options not chosen".

This section's basis is a **reading of profile text and of the host's readable policy**, not a
measurement of enforcement: the loaded-profile list, `/sys/kernel/security/apparmor/profiles`, is
readable by root only, so an unprivileged read answers `Permission denied`.
The whole output is in this round's `keep/apparmor-and-unit-reads.log`.

### What the `zed` profile's text grants

`/etc/apparmor.d/zed` is root-owned, mode 0644, and 1339 bytes.
It carries no `flags=` line, is not in `/etc/apparmor.d/disable/`, and its
`include if exists <local/zed>` at line 60 has no local file beside it
(`ls: cannot access '/etc/apparmor.d/local/zed'`), so the shipped text is what would apply.

Its exec rules, in its own words:

- `@{exec_path} mr` at line 19, where `@{exec_path}` is `/{usr/,}{local/,}{s,}bin/zed`:
  map and read of the program itself, **with no exec permission**;
- `rix` on fourteen utility rules, lines 21 to 34: `@{bin}/{m,g,}awk`, `basename`, `cat`, `diff`,
  `expr`, `flock`, `{,e}grep`, `hostname`, `logger`, `ls`, `mktemp`, `realpath`, `rm`, `sort`.
  `ix` inherits, so every one of them runs inside the `zed` profile, and the leading `r` is the read
  permission that `apparmor.d(5)` calls for beside an interpreted program.
- `rpx` on `/{usr/,}{local/,}{s,}bin/zpool` and `/{usr/,}{local/,}{s,}bin/zfs`, lines 36 and 37.
  `px` is the discrete-profile transition mode, so each of the two runs in the profile named for its
  own path: `zpool` in `/etc/apparmor.d/zpool`, which is `profile zpool … flags=(complain)` at its
  line 11, and `zfs` in `/etc/apparmor.d/zfs`, which carries `mount fstype=zfs, umount fstype=zfs,`
  with `capability sys_admin` and `capability dac_read_search`.
  `/etc/apparmor.d/mount-zfs` is the profile for the `mount.zfs` helper, which neither rule names.
- `rix` on `/{usr/,}{local/,}lib/zfs-linux/zed.d/*.sh` at line 38: the only rule that grants exec to
  a zedlet.

Apart from those two `rpx` rules, the profile carries no plain `x`, no `ux` and no `cx`, so a program
ZED starts either stays in `zed` through an `ix` or `rix` rule, or lands in the one profile a `px`
rule names.

Its capabilities are one line: `capability sys_admin,` (line 15). No other capability.
Its write rules are these, and it does have them for `/run`:

```text
@{run}/zed.pid rwlk                     line 43
@{run}/zed.state rwlk                   line 44
@{run}/zfs-list.cache@* rw              line 45
/etc/zfs/zfs-list.cache/{,*} rwk        line 41
owner @{tmp}/tmp.* rw                   line 47
@{sys}/module/zfs/parameters/zfs_zevent_len_max rw   line 52
@{PROC}/@{pid}/task/@{tid}/comm rw      line 56
/dev/zfs rw                             line 58
```

It has no write rule under the account's paths, and none under `/etc/zfs/zed.d`.

### The zedlet folder the rule names is not this host's

Line 38 names `lib/zfs-linux/zed.d`, and this host's zedlets are somewhere else:

```text
/{usr/,}{local/,}lib/zfs-linux/zed.d/*.sh   the rule (line 38)
ls: cannot access '/usr/lib/zfs-linux': No such file or directory
/usr/lib/zfs/zed.d                          sixteen entries, the packaged zedlets
/etc/zfs/zed.d                              symlinks into /usr/lib/zfs/zed.d
```

So on this host the packaged zedlets have no exec grant from the profile, whichever of the two paths
the kernel matches: the rule's folder does not exist and the real folder is not in the rule.
That is a defect of the shipped profile on Arch packaging, not of this design, and it means the
profile as shipped would deny ZED's own zedlets if it were enforced. ZED being disabled is why nobody
has seen it.

### What the hook and its pass need, read from the profile

Under `ix`, which is the only inheritance the profile offers, the hook's process would be in `zed`
and would need, in addition to what the profile carries:

- an exec grant on the installed file; line 38 is the only zedlet exec rule and it names another
  folder, so there is none today;
- `mount` and `umount` rules: the profile has none, and the `zfs` profile's `mount fstype=zfs`
  belongs to the `zfs` program's own profile, not to the hook's process;
- `capability dac_read_search`, to read and to search `/home/user`, which is the account's, mode
  `0700` (F8). The file's owner can traverse it; root can traverse it only through the capability,
  and a profile that does not grant the capability denies its use, so a root hook under `zed` could
  not stat the checkout at all. The `zfs` profile grants it and `mount-zfs` grants it beside
  `sys_admin`, which is why the `zfs` program itself works here.
  `[nothing]`: a holder scan that reads another process's `/proc` entries most likely also needs a
  `ptrace` read grant, which the profile does not carry.
- writes where the pass writes: the clone slot under the checkout, the readiness record under
  `/var/lib`, and its own state under `/run`; `zed` grants only `zed.pid`, `zed.state` and
  `zfs-list.cache@*` there.

### The profile's mode, as the readable policy derives it

The live list cannot be read unprivileged, and `[nothing]` cannot settle the mode either way on its
own. The host's own readable files make "the `zed` profile is not loaded" near-impossible here:

- `apparmor.service` is `enabled` and `active` (`systemctl is-enabled`, `is-active`);
- `/etc/apparmor/modes` reads `default enforce`, `include default`, `reload yes`, `fsp no`, and is
  root-owned mode 0644, so an unprivileged reader can read it;
- `zed` is in `/etc/apparmor/install.db` (line 706, `… zed`), so the managed policy installs it;
- `/usr/share/apparmor/flags.d/` holds no entry for `zed` or for `zfs`: its `00-main.conf` carries 413
  complain entries, `zpool complain` among them (`00-main.conf:419`), and nothing for either of the
  two profiles this route depends on;
- `/etc/apparmor/full-system-policy.md` records 710 installed profiles, zero drift, and modes
  verified against source.

So `enforce` is derived, not read, and `doctor` must say which of the two it did.

### Only the ZED path is confined

AppArmor attaches a profile by the path of the program run
(source reading, `doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`, "Root cause"),
so `/usr/bin/zfs` and `/usr/bin/zpool` land in their own profiles wherever they are started from,
including `zfs-mount.service` and the per-dataset unit.
The wrapper's installed file has no profile, and no unit names one: this host has no profile file for
`systemd` or for `systemd-user` (`ls` in `keep/apparmor-and-unit-reads.log`), and the `flags.d` entries
named for them are inert without a profile file.
The boot unit's `ExecStart=`, `ExecReload=` and `ExecStopPost=` therefore start the wrapper unconfined;
only a start under `zed`, which is the hook's own path, inherits a profile.
The AppArmor need is the hook's alone.

### The rules to add, and the reading each rests on

The file to write is `/etc/apparmor.d/local/zed`, which the profile includes at its line 60 and which
does not exist yet.
Each rule below answers one need the reading found, and the reading is the one the section above
records:

```text
# /etc/apparmor.d/local/zed
# ZED starts the hook, and every packaged zedlet inherits these rules (the owner's accepted cost).
/usr/libexec/monochromatic-git-policy-cli/<the installed file> rix,
# The pass mounts and unmounts datasets; the zfs profile's own rules are these two.
mount fstype=zfs,
umount fstype=zfs,
# Root searches the account's 0700 home and reads the checkout; only this capability lets it.
capability dac_read_search,
# The holder scan reads other processes' /proc entries, which the kernel checks as a ptrace read.
ptrace (read),
# Where the pass writes: the clone slot under the checkout, the readiness record, its own state.
owner /home/user/Monochromatic/.claude/worktrees/** rwlk,
owner /var/lib/monochromatic-git-policy-cli/** rwlk,
owner /run/monochromatic-git-policy-cli/** rwlk,
```

Each line, and what it rests on:

- **the wrapper's file, `rix`.** The hook is a zedlet: ZED execs it, and `ix` is the inheritance the
  profile offers. Without an exec rule the hook cannot start at all under `zed`.
  `rix` rather than `rix`-plus-something is what `local/git` line 14 already grants the same file for
  the Git path.
  `[nothing]` The exact file name is the root-modes design's choice 3.
- **`mount fstype=zfs` and `umount fstype=zfs`.** The profile has no mount rule today, and the `zfs`
  profile's `mount fstype=zfs` belongs to the `zfs` program's profile, not to the hook's process
  (`keep/apparmor-and-unit-reads.log`: `zfs` line 18, `mount-zfs` lines 26 to 32).
- **`capability dac_read_search`.** `/home/user` is the account's, mode `0700` (F8); the file's owner
  can traverse it, root only through the capability, and a profile that does not grant it denies the
  use. The `zfs` profile grants it for the same reason.
- **`ptrace (read)`.** The holder scan reads other processes' `/proc` entries.
  `[nothing]` This is the one rule whose need is inferred rather than measured; the narrower
  `ptrace (read) peer=unconfined,` form, which `local/git` line 33 uses, is available if a denial
  names a narrower peer set.
- **the three writable paths.** The pass writes the clone slot under the checkout, the readiness record
  under `/var/lib`, and its own state under `/run`; `zed` grants only `zed.pid`, `zed.state` and
  `zfs-list.cache@*` there (its lines 43 to 45), and no rule at all under the account's paths.

**Two things about this file are not this design's to fix.** The `zed` profile's only zedlet exec rule
names `/{usr/,}{local/,}lib/zfs-linux/zed.d/*.sh` (line 38), and this host's zedlets are in
`/usr/lib/zfs/zed.d`, which does not exist under that name (`ls: cannot access '/usr/lib/zfs-linux'`);
so the packaged zedlets have no exec grant from the profile either. That is a defect of the shipped
profile on Arch packaging, and the owner's answer does not reach it: the rules above are the hook's,
and ZED's own zedlets need a line for the folder they are actually in.
`doctor` reports the folder mismatch and whether `local/zed` exists.

**What this answer buys and costs**, in the terms the ranking used: it buys one profile to load, one
file to read, no transition to get wrong, and a start that cannot fail for a missing target profile.
It costs every packaged zedlet the needs that make a mount work: `mount` over any path and a capability
that reaches every `0700` directory on the machine, and the audit identity is `zed` for a mount ZED's
notification code never performs.
The owner accepted that cost when answering.

**The AppArmor need is the hook's alone.** The wrapper starts unconfined from the boot unit's
`ExecStart=`, `ExecReload=` and `ExecStopPost=` and from the propagation service, because this host has
no profile file for `systemd` or `systemd-user` (the section above says why), so the grant above is
needed only for a start under ZED's own path.

## The covered entry, and the ancestors above it

Measured as the account, with the checkout unmounted and the entry immutable
(round one's `keep/guest-logs/guest-ancestor.log`, unchanged since and re-read for this text):

```text
mv ~/Monochromatic ~/Monochromatic.moved          -> Operation not permitted     (the flag)
rmdir ~/Monochromatic                              -> Operation not permitted     (the flag)
touch ~/Monochromatic/planted                       -> Operation not permitted     (the flag)
mv <home>/account <home>/account.moved              -> Permission denied           (the parent, root-owned)
mv ~/projects ~/projects.moved                      -> succeeds                    (the parent, account-owned)
   then ~/projects/Monochromatic/planted exists     (planted by root in the arm, not by the account)
zfs mount zfsstep19/home/monochromatic-git-checkout -> Insufficient privileges    (the account)
```

The condition the design needs:

> the covered entry is protected while every directory **above the entry's parent**, up to the home
> dataset's root, is one the account cannot write.

A looser wording, "every directory between it and the home dataset's root is one the account cannot
write", fails at this host's own path, because `/home/user` is the account's directory
(mode `0700`, owner `user:user`, read from the host), and that wording would then claim the condition
fails there. What holds at `/home/user/Monochromatic` is the corrected condition: `/home` is root's,
the entry is the account's, and the account cannot rename `/home/user` because that needs write on
`/home`. At a deeper path the account owns, such as `~/projects/Monochromatic`, the account renames
`~/projects` and the flag on the entry protects nothing, which the arm shows.

The flag has a second effect this round measured: with the checkout unmounted, the entry is an
immutable directory in the home underlay, so `zfs mount -a` cannot create a mountpoint beneath it and
the provider fails instead of mounting a dataset into the exposed underlay
(round four's `evidence-hazard-flag-on.log`).

## The failure controls

Every failure control of every round so far, each with its own kept log.
Five arms ran this round, on one boot each, plus two runtime arms inside the first of them:
the chosen unit set green, the same with the dataset already mounted, the same with the checkout's key
away, the same with the provider failing, and a hung pass with the bounds scaled down.
The earlier rounds' arms are kept and named with their round.

**This round, on the guest of "What it ran", all with SELinux enforcing.**

- **The chosen unit set, boots green.** Kept logs `state-chosen-green.log`,
  `evidence-chosen-green.log`, boot `d611cc4c-27da-4607-91c9-38e47140c8f1`.
  The provider mounted the checkout, the gate verified it and attached the slot, the account consumer
  saw the checkout and its slot, the nested unit mounted its dataset and its own consumer saw it, and
  sessions started last:
  provider `1795545978.001546` to `1795545978.062661`, gate `1795545978.824420` to
  `1795545978.882730`, account consumer `1795545978.886534` to `1795545978.912087`, nested unit
  `1795545978.898076` to `1795545978.925234`, its consumer `1795545978.928715` to
  `1795545978.941876`, sessions `1795545979.363744`.
  The unit reported `TimeoutStartUSec=5min`, `TimeoutStopUSec=1min`,
  `TimeoutStopFailureMode=terminate`, and `Wants=zfs-mount.service` (`state-chosen-green.log`).
- **A provider restart leaves the gate reloaded and the nested unit untouched.** Kept log
  `restart-provider-chosen.log`, in the same boot: every invocation id unchanged, the journal's three
  lines `Stopped`/`Finished`/`Reloading`/`Reloaded`, and the dataset still mounted.
  The chosen answer's whole saving, measured.
- **A gate restart skips the nested unit instead of failing it.** Kept log `restart-gate-chosen.log`,
  in the same boot: `git-worktree-nested-output-mount.service - Mount … skipped, unmet condition check
  ConditionPathIsMountPoint=!…` at `1795546036.984167`, `ActiveState=inactive`, `ConditionResult=no`,
  its consumer running, and the dataset still mounted.
- **A boot where the provider mounted the nested dataset first: the nested unit is skipped and its
  consumer still runs.** Kept log `evidence-chosen-mounted-first.log`, boot
  `fc7d3967-e3fc-46c1-a85c-1d191020f932`: the skip lands at `1795546081.684890`, the unit has no
  invocation id at all (`ActiveState=inactive`, `ConditionResult=no`), the dataset is mounted by the
  provider, and both consumers finish.
- **The checkout's key away: the gate, both consumers and the nested unit fail closed, and logins
  proceed.** Kept logs `state-chosen-key-removed.log`, `evidence-chosen-key-removed.log`, boot
  `cde4dece-de1e-4aa1-8fff-0242f0f4f261`, one contiguous run:

  ```text
  1795546126.857861 zfs-step-19-chosen-2026-11-24 systemd[1]: zfs-load-key@zfsstep19-home-monochromatic\x2dgit\x2dcheckout.service: Main process exited, code=exited, status=255/EXCEPTION
  1795546126.858291 zfs-step-19-chosen-2026-11-24 systemd[1]: Failed to start zfs-load-key@zfsstep19-home-monochromatic\x2dgit\x2dcheckout.service - Load ZFS key for zfsstep19/home/monochromatic-git-checkout (step 19 stand-in for the generated unit).
  1795546126.861234 zfs-step-19-chosen-2026-11-24 systemd[1]: Dependency failed for reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts.
  1795546126.862744 zfs-step-19-chosen-2026-11-24 systemd[1]: Dependency failed for git-account-session-standin.service - A service of the account that needs the checkout.
  1795546126.864096 zfs-step-19-chosen-2026-11-24 systemd[1]: git-account-session-standin.service: Job git-account-session-standin.service/start failed with result 'dependency'.
  1795546126.864116 zfs-step-19-chosen-2026-11-24 systemd[1]: Dependency failed for git-worktree-nested-output-mount.service - Mount zfsstep19/nosnap/vm-builder-output under the verified checkout.
  1795546126.865437 zfs-step-19-chosen-2026-11-24 systemd[1]: Dependency failed for git-vm-builder-standin.service - A vm-builder service of the account that needs the dataset under the checkout.
  1795546126.866776 zfs-step-19-chosen-2026-11-24 systemd[1]: git-vm-builder-standin.service: Job git-vm-builder-standin.service/start failed with result 'dependency'.
  1795546126.866799 zfs-step-19-chosen-2026-11-24 systemd[1]: git-worktree-nested-output-mount.service: Job git-worktree-nested-output-mount.service/start failed with result 'dependency'.
  1795546126.866804 zfs-step-19-chosen-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Job reconcile-git-worktree-clone-mounts.service/start failed with result 'dependency'.
  ```

  and sessions at `1795546128.417573`, 1.5 s later and unaffected.
- **A provider failure with a healthy checkout: the provider fails and the gate stays green.** Kept logs
  `state-chosen-provider-fails.log`, `evidence-chosen-provider-fails.log`, boot
  `49f8f811-942d-4d48-8cc6-46f3ddee2b3f`: the provider printed
  `cannot mount '/srv/zfs-step-19-fixture/blocked/child': failed to create mountpoint: Operation not permitted`,
  ended `Result=exit-code` with `ExecMainStatus=1`, twice in that boot (once from `zfs.target`, once
  from the gate's `Wants=`), while the gate finished green and every consumer and the nested unit
  succeeded.
  That is the record's own "still unmeasured" sentence, measured for the shape the owner chose.
- **A hung pass with the chosen ordering and the bounds scaled inside the arm: logins proceed and the
  gate fails 20.86 s later.** Kept logs `state-chosen-stages.log`, `evidence-chosen-stages.log`, boot
  `7f0fa2eb-a49f-4025-8633-5ee6256dd6d9`: the bounds were `TimeoutStartSec=8` and
  `TimeoutStopSec=4` in this arm only, because the owner's 5 min and 1 min would not fit a boot, and
  the pass and stop pass were the stand-ins that ignore SIGTERM, one contiguous run:

  ```text
  1795546177.439225 zfs-step-19-chosen-2026-11-24 systemd[1]: Starting reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts...
  1795546177.881849 zfs-step-19-chosen-2026-11-24 systemd[1]: Starting systemd-user-sessions.service - Permit User Sessions...
  1795546177.890506 zfs-step-19-chosen-2026-11-24 systemd[1]: Finished systemd-user-sessions.service - Permit User Sessions.
  1795546185.553034 zfs-step-19-chosen-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: start operation timed out. Terminating.
  1795546189.802997 zfs-step-19-chosen-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: State 'stop-sigterm' timed out. Killing.
  1795546194.053098 zfs-step-19-chosen-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: State 'stop-post' timed out. Terminating.
  1795546198.303094 zfs-step-19-chosen-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: State 'final-sigterm' timed out. Killing.
  1795546198.303748 zfs-step-19-chosen-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Failed with result 'timeout'.
  1795546198.304671 zfs-step-19-chosen-2026-11-24 systemd[1]: Failed to start reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts.
  ```

  Sessions started 0.44 s after the gate and finished 20.4 s before it failed.
  No `Aborting` line, no SIGABRT kill and no coredump appears in that boot, because the drop-in put
  the unit in `terminate`.

**Round five's arms, still standing.**

- The unit text of the fifth revision, with the stop-failure drop-in and the same bounds as test values:
  passes (round five's `evidence-green.log`, boot `de767eea-e7a9-4c4d-ad6a-e4a97803507b`).
- A per-dataset unit running `zfs mount` under `Requires=` on the provider: mounts at a boot, and fails
  with `filesystem already mounted` after a provider restart (round five's `evidence-v9c.log`,
  `restart-provider-v9c.log`). This is the arm the chosen `Wants=` and the condition answer.
- A pass and a stop pass that ignore SIGTERM with `Requires=` in place: the three stop stages, 20.53 s
  (round five's `evidence-stages.log`).
- The four placements of `TimeoutStopFailureMode=terminate` (round five's `stopmode-shapes.log`).

**Round four's arms, still standing, on the same image and the same stand-ins.**

- The unit text with both bounds and no stop-failure line: passes, boot
  `5b8c1894-4729-48ba-a307-ac0982ba40da` (round four's `evidence-as-written-stopsec.log`).
- This host's shape, no key-loading unit for the checkout and no key lines in the unit: the gate
  refuses `checkout-not-the-dataset` and the boot fails closed
  (round four's `state-host-shape.log`, `evidence-host-shape.log`, boot
  `773296e0-6fea-4ee0-94c1-1e486e063ce4`).
- The hazard with the covered entry's flag on, `canmount=on`, and the checkout's mount failing: the
  provider cannot create the mountpoint and the boot's mount fails.
  The set of units that boot ran is the arm's own:
  `zfs.target`, `zfs-mount.service`, the gate and the pool's key unit were enabled, the per-dataset
  unit and both consumers were disabled, the checkout's key unit and its file were moved aside, and
  both drop-ins were absent (round four's `state-hazard-flag-on.log`, the blocks "== enabled" and
  "== drop-in directories"; round four's `evidence-hazard-flag-on.log`, boot
  `fcfae077-91df-49bf-bced-ff085a27b7ca`).
  One contiguous run of that boot:

  ```text
  1795522243.168284 zfs-step-19-fourth-2026-11-24 systemd[1]: Starting zfs-mount.service - Mount ZFS filesystems...
  1795522243.192568 zfs-step-19-fourth-2026-11-24 zfs[818]: cannot mount '/srv/zfs-step-19-fixture/home/account/Monochromatic/package/vm-builder/cli/output': failed to create mountpoint: Operation not permitted
  1795522243.193788 zfs-step-19-fourth-2026-11-24 systemd[1]: zfs-mount.service: Main process exited, code=exited, status=1/FAILURE
  1795522243.193878 zfs-step-19-fourth-2026-11-24 systemd[1]: zfs-mount.service: Failed with result 'exit-code'.
  1795522243.194032 zfs-step-19-fourth-2026-11-24 systemd[1]: Failed to start zfs-mount.service - Mount ZFS filesystems.
  1795522243.195780 zfs-step-19-fourth-2026-11-24 systemd[1]: Dependency failed for reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts.
  1795522243.197004 zfs-step-19-fourth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Job reconcile-git-worktree-clone-mounts.service/start failed with result 'dependency'.
  1795522244.547960 zfs-step-19-fourth-2026-11-24 systemd[1]: Starting systemd-user-sessions.service - Permit User Sessions...
  ```

  **What that boot does not show is a boot with nothing mounted.** The home dataset mounted before the
  provider reached the dataset that failed, and the mount table proves it: `zfs list` reports
  `zfsstep19/home` with `MOUNTED yes` and `findmnt` prints it as `zfsstep19/home`, while the unit list
  holds `srv-zfs…-home.mount` active (same log).
  So the sentence is: **the provider unit fails, and datasets outside the failed mountpoint's subtree
  still mount.** The OpenZFS task that decides it skips only the failed dataset's mountpoint
  descendants and carries on with the rest, then exits nonzero
  (source reading, `zfs-2.4.4` `lib/libzfs/libzfs_mount.c`, `zfs_mount_task`).
  The home underlay's inventory in that boot is the three directories it started with: no `package`
  folder was created, and no dataset was mounted under the checkout.
- A hung start with both bounds, where the main process dies at SIGTERM: a 20 s start bound and a 3 s
  stop bound held logins 23.54 s. That is the cooperative case
  (round four's `evidence-bound-stop.log`, boot `ad2d3287-25f8-4086-819a-3ac4320671ea`).

**Round three's arms, still standing, on the same image and the same stand-ins.**

- The checkout's key file away: the key unit fails and the gate's job fails closed
  (round three's `evidence-key-removed.log`).
- The covered entry's flag removed: refuses and fails closed; `SWEEP covered=----------------------`,
  `SWEEP refused=covered-entry-not-immutable`, `ExecMainStatus=2`, and both consumers' and the
  per-dataset unit's jobs failed with `dependency` (round three's `evidence-flag-removed.log`, boot
  `e36e69bf-8441-46b0-9972-7d7409090919`).
- A 20 s start bound on a hung pass: the bound fires, the stop pass runs, sessions start after the
  failure (round three's `evidence-bound.log`, `evidence-bound-late.log`).
- The two hazard arms with the flag **off**: `canmount=on` mounts the descendant into the exposed
  underlay and creates the folder (round three's `evidence-hazard-nested-auto.log`, boot
  `ce693fde-d03d-4583-a357-78db392ea64e`); `canmount=noauto` mounts nothing and creates nothing
  (round three's `evidence-hazard-v4.log`).
- The learned-unit trap, three shapes: round three's `evidence-learned-unit-only.log`,
  `boot-arm-learned-unit.log`, `boot-arm-learned-with-provider.log`, quoted under "Never require a
  mount unit systemd learned".

**The login is not gated by the gate's success, only delayed by its end.** In every failing arm,
`Starting systemd-user-sessions.service` follows the failed job
(round three's three failure logs, round four's `evidence-host-shape.log`,
`evidence-hazard-flag-on.log`, and this round's `evidence-stages.log`).
That is why the design puts `Requires=` on the services of the account that need the checkout, which
is choice V6.

**SELinux was enforcing in every arm, and the audit log is attributed rather than counted.**
This round's evidence step reads three things: `ausearch --message AVC,USER_AVC --start boot`, every
record the audit file holds, and the count that falls inside each boot's journal window.
The result in this round's green arm (kept log `evidence-green.log`, "the boots this log holds"):

```text
boot 0d127f12a43d4a1dbea9f0d39f396d62 from 11/24/2026 08:02:38 to 11/24/2026 08:02:44 avc-records=0
boot f0f892098bf346d0aaa0b04bc509dc3a from 11/24/2026 13:40:30 to 11/24/2026 14:19:37 avc-records=24
boot de767eeae7a94c4dad6ae4a97803507b from 11/24/2026 14:19:41 to 11/24/2026 14:20:17 avc-records=0
```

So this round's boot has no denial, and the twenty-four records the file holds all belong to this
guest's own first boot, the one that built the packages and made the fixture, which ran from
`13:40:30` to `14:19:37`.
Every one of them is the same denial, and none names a unit of this design:

```text
type=AVC msg=audit(1795528107.779:196): avc:  denied  { dac_override } for  pid=1176 comm="systemd-getty-g" capability=1  scontext=system_u:system_r:systemd_getty_generator_t:s0 tcontext=system_u:system_r:systemd_getty_generator_t:s0 tclass=capability permissive=0
```

In this round's log the print and the counts agree, twenty-four either way.
They did not agree in the fourth round's log, whose print held thirteen of the fourteen its counts
reported, which is why this revision says "the records the file holds" and quotes the two counts
beside the print instead of calling the print complete.
The earlier rounds' "avc count in the audit log this boot" was this cumulative file count under a
label that said "this boot", and that label is why it read as a contradiction: the count rises with
each boot of the same guest while `--start boot` stays empty, because the records belong to boots
before the arm's.
Nothing in them names this design's programs, and the guest's stand-in units ran in
`unconfined_service_t`, which the evidence block prints (kept log `evidence-green.log`, the `selinux`
block), and which is `mvm`'s own drop-in for a confining policy
(`doc/troubleshooting/mvm-fedora-guest-confined-by-selinux.md`).

## What a hung start costs

Every number here is measured: the phases of one start on systemd 262.2 in round three's
`systemd-262-arms.log`, the stop bound on the host's own user manager in round three's
`keep/host-user-manager-stopsec.log`, and boots of rounds three, four, five and this one.

**These are the gate's own failure times, not a held login.** The owner's V8a answer drops the
before-login line from every unit, so nothing here delays a session; the stages arm under
"The failure controls" shows sessions starting 0.44 s after the gate and 20.4 s before its failure.

The default: a oneshot has no start timeout.
`systemctl show` for a unit with no `TimeoutStartSec=` reports `TimeoutStartUSec=infinity`
(round three's `systemd-262-arms.log` arm A, and round two's `start-bound.log`; and this round's
nested unit and both consumers, in `state-chosen-green.log`), so a hung gate, nested unit or consumer
is reaped only by its own bound or by a person; with the ordering line gone it no longer holds a login.

The start bound is **per phase**, not for the start as a whole:

```text
arm B1: TimeoutStartSec=4, ExecStartPre=-/bin/sleep 6   -> start-exit=1, elapsed-ms=4033,  Result=timeout
arm B2: TimeoutStartSec=4, ExecStartPre=-/bin/sleep 3, ExecStart=/bin/sleep 3
                                                       -> start-exit=0, elapsed-ms=6037,  Result=success
```

So the ceiling covers each of the start's phases separately: a pass that spends its pre-phase running
the probe and its main phase running the reconcile can take twice the line's value.
A pre-phase that overruns the bound kills the unit before its `ExecStart` runs (arm B1), which is why
the two-phase worst case needs a pre-phase that finishes inside the bound.

The same bound covers a **reload**, and a reload that overruns does not fail the unit:

```text
arm C: TimeoutStartSec=3, ExecStart=/bin/true, RemainAfterExit=yes, ExecReload=/bin/sleep 6
reload-exit=1, elapsed-ms=3102, TimeoutStartUSec=3s, Result=success
```

(`systemd.service(5)` at v262: "the start timeout is also applied to service reloads, regardless of
whether implemented through `ExecReload=` or via the reload logic enabled via `Type=notify-reload`";
and `service.c` arms the same `s->timeout_start_usec` per phase).

The stop phase **extends the hold**, because it runs inside the start job's failure path and the
sessions unit is ordered after the job, not after the kill:

```text
arm D: TimeoutStartSec=3, ExecStart=/bin/sleep 10, ExecStopPost=/bin/sleep 5, dependent After= it
1795508155.691091 Starting arm-d.service...
1795508158.748077 arm-d.service: start operation timed out. Terminating.
1795508158.749128 arm-d.service: Main process exited, code=killed, status=15/TERM
1795508163.759644 arm-d.service: Failed with result 'timeout'.
1795508163.761897 Starting arm-d-dependent.service...
```

That is 3.06 s to the kill and 5.01 s more before the unit reached `failed` and the dependent started.
The stop phase is itself bounded by `TimeoutStopSec=` (default 90 s): on the host's own user manager a
two second `TimeoutStopSec=` killed a five second `ExecStopPost=` after 2.07 s, with
`State 'stop-post' timed out. Terminating.` and `Control process exited, code=killed, status=15/TERM`
(round three's `keep/host-user-manager-stopsec.log`).

**The stop phase is more than one stage, and the bound is spent once per stage a program survives.**
This round's arm measured the whole sequence with a start pass and a stop pass that both ignore
SIGTERM, `TimeoutStartSec=8` and `TimeoutStopSec=4`, and the mode in `terminate`
(kept log `evidence-stages.log`, boot `fec7550a-5a0f-450c-802b-f9e004560004`, one contiguous run):

```text
1795530374.170753 zfs-step-19-fifth-2026-11-24 systemd[1]: Starting checkout-propagation.service - Ask the boot unit to re-verify the checkout's private propagation...
1795530374.179243 zfs-step-19-fifth-2026-11-24 systemd[1]: Starting reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts...
1795530374.198372 zfs-step-19-fifth-2026-11-24 systemd[1]: Finished checkout-propagation.service - Ask the boot unit to re-verify the checkout's private propagation.
1795530382.442884 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: start operation timed out. Terminating.
1795530386.539982 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: State 'stop-sigterm' timed out. Killing.
1795530386.540052 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Killing process 911 (sweep-hang-term) with signal SIGKILL.
1795530386.540176 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Killing process 1075 (sleep) with signal SIGKILL.
1795530386.540434 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Main process exited, code=killed, status=9/KILL
1795530390.623415 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: State 'stop-post' timed out. Terminating.
1795530394.707074 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: State 'final-sigterm' timed out. Killing.
1795530394.707136 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Killing process 1076 (sweep-stop-hang) with signal SIGKILL.
1795530394.707231 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Killing process 1092 (sleep) with signal SIGKILL.
1795530394.707449 zfs-step-19-fifth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Failed with result 'timeout'.
1795530394.707667 zfs-step-19-fifth-2026-11-24 systemd[1]: Failed to start reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts.
1795530394.711395 zfs-step-19-fifth-2026-11-24 systemd[1]: Starting systemd-user-sessions.service - Permit User Sessions...
1795530394.724362 zfs-step-19-fifth-2026-11-24 systemd[1]: Finished systemd-user-sessions.service - Permit User Sessions.
```

The stages and what each cost, from that run:

- the start: 8.26 s (`1795530374.179243` to `1795530382.442884`), the bound's 8 s plus the probe's
  microseconds and the manager's own work;
- the SIGTERM the failure sends to the main process: 4.10 s, because that pass ignored it, then SIGKILL;
- the `ExecStopPost=` the failure then runs: 4.08 s, because the stop pass ignored it too, then a move
  to the final stage;
- the final SIGTERM to everything left: 4.08 s, then SIGKILL;
- the final SIGKILL stage: a third of a millisecond, because no program can refuse it.

From the unit's start to the sessions unit is 20.53 s, and the boot's own evidence file agrees on when
each pass began (`SWEEP-TERM start … uptime=6.21`, `SWEEP-STOP-TERM start … uptime=18.56`).
No `Aborting` line, no SIGABRT kill and no coredump appears anywhere in that boot.

**Round four's arm is the cooperative case**, and this revision labels it so: its main process died at
SIGTERM within a millisecond, so only the stop-post stage ran
(round four's `evidence-bound-stop.log`, boot `ad2d3287-25f8-4086-819a-3ac4320671ea`):

```text
1795522327.244840 zfs-step-19-fourth-2026-11-24 systemd[1]: Starting reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts...
1795522347.454452 zfs-step-19-fourth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: start operation timed out. Terminating.
1795522347.455101 zfs-step-19-fourth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Main process exited, code=killed, status=15/TERM
1795522350.705097 zfs-step-19-fourth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: State 'stop-post' timed out. Aborting.
1795522350.779011 zfs-step-19-fourth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Failed with result 'timeout'.
1795522350.788178 zfs-step-19-fourth-2026-11-24 systemd[1]: Starting systemd-user-sessions.service - Permit User Sessions...
```

20.21 s to the timeout, 3.25 s more to the stop bound, 23.54 s to the sessions unit; the evidence
file's own two lines agree (`SWEEP hung … uptime=6.24`, `SWEEP stop-hung … uptime=26.44`); and that
boot's `abort` mode is why the stop bound ended in SIGABRT and two coredumps rather than a kill
(round four's `timeouts-read-back.log`).

So the worst case a reader should carry is:

```text
the probe's own deadline (0.35 s)
  + TimeoutStartSec
  + TimeoutStopSec × (the stop stages whose processes ignore SIGTERM: at most 3)
```

which is 20.53 s measured in round five for an 8 s start bound and a 4 s stop bound, 20.86 s measured
this round for the same scaled pair, 8 min 0.35 s at the owner's 5 min and 1 min with all three stop
stages surviving, and about 270 s for a 20 s start bound with the default 90 s stop bound.
The cooperative case is the same formula with one stage instead of three, which is 23.54 s for a 20 s
start bound and a 3 s stop bound.
A pre-phase that is unbounded rather than the probe adds one more start bound, because the start bound
is per phase.
That arithmetic is why the unit text carries `TimeoutStopSec=` as well as `TimeoutStartSec=`, why the
stop-failure drop-in matters, and what step 22's measured ceilings replace the owner's numbers with.

Round three's boot where the clock stepped, kept for what it says about reading this arithmetic
(round three's `evidence-bound.log`, `evidence-bound-late.log`):
`SWEEP hung … uptime=6.39` and `SWEEP stopped … uptime=26.57`, so 20.18 s of monotonic time from the
pass's start to the stop pass's own line, against 21.31 s of wall clock, because the wall clock
stepped by about a second during that boot.
From the timeout line to the unit's failure is 17 ms
(`1795507884.711975` to `1795507884.728790`), with the stop pass finished inside it.
systemd's timer is monotonic, so uptimes are what date a bound.

The prefix's limit, measured separately (round two's `start-bound.log`):

- a failing `ExecStartPre=` under `-`: unit `active`, `ExecStart` ran;
- the same line without `-`: unit `failed`, `ExecStart` did not run;
- a **timeout** in `ExecStartPre=` under `-`: `start-pre operation timed out`, `Result=timeout`, and
  the unit's `ExecStopPost=` ran.

The consequence for the design is that the probe must enforce its own deadline, which it does, and
must not rely on the prefix to survive a timeout.

## Never require a mount unit systemd learned

Measured three times on three system managers, in the shape the earlier rounds used.

**systemd 258, at a boot, the learned unit alone required.** The gate's start job never appeared in
the boot's journal, the unit was left `inactive` with `Result=success`, the sweep never ran, and
sessions opened (round three's `evidence-learned-unit-only.log`, boot
`867cba26-983a-4291-933b-31fcd17243bb`, with the provider's drop-in and the propagation drop-in both
off).

**systemd 262.2, at a boot, the learned unit alone required.** A provider unit made a bind mount at
boot, so the learned unit did not exist when the boot transaction was built; the requiring unit's
result was `LoadState=loaded`, `ActiveState=inactive`, `Result=success`, and the boot's journal holds
no line for its start at all (round three's `boot-arm-learned-unit.log`, boot
`941589a1-33ab-4e02-a73a-f5bbc16764bb`).

**systemd 262.2, at a boot, the provider required beside the learned unit, both drop-ins absent.**
The provider finished `Result=success` and made its mount, the learned mount unit was
`ActiveState=active`, and the unit requiring both was `LoadState=loaded`, `ActiveState=inactive`,
`Result=success`, with no journal line for its start (round three's
`boot-arm-learned-with-provider.log`, boot `8e07e08b-ee3f-44ca-9827-b999a57c365e`).
That is the control round three's own review asked for, and it settles the confound: requiring the
provider does not make the boot transaction start a unit that requires the learned mount unit.

**Round two's "second shape", now explained.** There the learned unit was required beside a required
provider **with both drop-ins on**, and the gate did run. Its journal puts the gate's first start ten
milliseconds after `Finished checkout-propagation.service`, while the provider's own request had
failed with `Unit <escaped checkout path>.mount not found` (round two's
`evidence-learned-unit-required.log`). With the control's measured silence, the start that ran was
the propagation service's request, the one component the control removes.

So the rule the design states is:
**order with `After=` or with `RequiresMountsFor=`,
and never write `Requires=` on a unit systemd learned from the mount table.**
The same rule covers a unit a boot does not install at all: a consumer that requires the per-dataset
unit on a boot without it stayed `inactive (dead)` with `Result=success` and no journal line
(round three's `evidence-v9b.log`).

The source says why the failure is quiet: a dependency job that cannot be added for `-ENOENT` is
logged at debug level (systemd v262 `src/core/transaction.c`, the block that logs
`Cannot add dependency job, ignoring` through `log_unit_full_errno` with `LOG_DEBUG`), and a learned
mount unit cannot be started anyway: its own start answers `No mount parameters to operate on`
(round one's `keep/guest-logs/journals-per-boot.log`, `No mount parameters to operate on` at
`1795486467.319399`, then `Failed with result 'resources'` at `1795486467.319417`).

The corrected V1 reason follows from it: the lost-event record rejected a mount unit **as a
trigger**, not as a provider (`doc/decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md`,
"What was not chosen"), and the account-writable-path exposure applies to `zfs mount -a` just as much
as to a route-owned unit.

## The route-owned alternative, measured, kept as V1's unchosen option

This is the option the owner did not choose for V1 (it is restated under "The options not chosen"),
kept here because it is the design's only measurement of a route-owned mount.
A unit the route owns mounts the checkout explicitly, with the checkout dataset at `canmount=noauto` so
the provider's scan skips it. Round two ran it three ways, and its whole text is in that round's
`guest-scripts/03-units.sh`.
The program path is this host's, `/usr/bin/zfs`; the guest that ran it used its own
`/opt/openzfs-2.4.4/sbin/zfs`, which is the same call.

- Green: it mounts the checkout. Kept log (round two) `evidence-route-owned.log`: the provider
  finished at `1795503146.401532` with the checkout skipped, the route unit started at
  `1795503146.404749`, finished at `1795503146.434234`, and the gate verified afterwards.
- With the key unit required and the key file away: a dependency failure, like the provider shape
  (round two's `evidence-route-owned-key-removed.log`).
- With no key requirement and the key file away: a status that names the dataset and the reason
  (round two's `evidence-route-owned-bare.log`):

  ```text
  zfs[838]: cannot mount 'zfsstep19/home/monochromatic-git-checkout': encryption key not loaded
  mount-monochromatic-checkout.service: Main process exited, code=exited, status=1/FAILURE
  mount-monochromatic-checkout.service: Failed with result 'exit-code'
  ```

That third result is the option's only measured advantage, and it exists exactly when the design does
**not** put the key unit's requirement on the route unit. Choice V1 weighs it.

## What the runs show, one arm per log

This round's logs are one guest's, on systemd 258.
Earlier rounds' logs are named with their round.

- `prepare.log`: the guest's package install and the pinned tarball's sha256, cut short by the guest
  agent, which is why the same script's read-back is kept separately.
- `prepare-readback.log`: the same script, second run: Fedora Linux 43, systemd `258.11-1.fc43`,
  kernel `6.17.1-300.fc43.x86_64`, SELinux enforcing, module `2.4.4-1`, userspace `zfs-2.4.4-1`, and
  the source install's unit list with its `zfs-load-key.service -> /dev/null`.
- `fixture.log`, `fixture-fix.log`: the pool, the datasets, the cache file the import unit reads, the
  immutable entry, and `zfs-mount-a-exit=0` with the checkout and the dataset under it mounted; then
  the busy unmount `03-fix-fixture.sh` completes.
- `units-install.log`: every unit, drop-in and stand-in as systemd reads them, and the two escaped
  names.
- `state-green.log`, `evidence-green.log`: the fifth revision's unit text, with the stop-failure
  drop-in, and the green boot of it, with the order gate, account consumer, nested unit, its consumer,
  sessions; the bounds and modes of all four units; and the audit attribution.
- `state-v9c.log`, `evidence-v9c.log`: the same boot with the per-dataset unit running an explicit
  `zfs mount`.
- `restart-provider-v9c.log`: the provider restart that re-ran that unit and failed it on
  `filesystem already mounted`.
- `state-stages.log`, `evidence-stages.log`: the start bound and the three stop stages in one boot,
  with a pass and a stop pass that ignore SIGTERM.
- `stopmode-shapes.log`: the four placements of `TimeoutStopFailureMode=terminate` and the value
  systemd reports for each.
- `state-chosen-green.log`, `evidence-chosen-green.log`: this revision's chosen unit set, as systemd
  reads it and as it boots: `Wants=`, the owner's bounds, the nested unit's condition, no before-login
  line, and the order provider, gate, consumers and nested unit, sessions.
- `restart-provider-chosen.log`: the provider restart under `Wants=`, with invocation ids before and
  after and the gate's reload.
- `restart-gate-chosen.log`: the gate restart, which restarts the nested unit and skips it on its
  condition.
- `state-chosen-green-mounted-first.log`, `evidence-chosen-mounted-first.log`: the boot where the
  provider mounted the nested dataset first, so the nested unit skips.
- `state-chosen-key-removed.log`, `evidence-chosen-key-removed.log`: the checkout's key away, the
  fail-closed jobs, and logins proceeding.
- `state-chosen-provider-fails.log`, `evidence-chosen-provider-fails.log`: a provider failure with a
  healthy checkout, and the gate green beside it.
- `state-chosen-stages.log`, `evidence-chosen-stages.log`: the hung pass with the bounds scaled down
  inside that arm.
- `prepare-chosen.log`, `prepare-chosen-readback.log`, `fixture-chosen.log`, `fixture-fix-chosen.log`,
  `units-install-chosen.log`: the second guest's build and fixture.
- `restart-provider.log` (round three): the provider restart that took four units through their stop
  pass, with `stopped-slot=unmounted`.
- `evidence-learned-unit-only.log`, `boot-arm-learned-unit.log`,
  `boot-arm-learned-with-provider.log` (round three): the learned-unit trap in three shapes.
- `systemd-262-arms.log` (round three): the bound facts, `RequiresMountsFor=`'s expansion, `Upholds=`
  and `BindsTo=` on a learned unit, and the four-unit restart chain, all on 262.2.
- `evidence-v9b.log` (round three): the mounter the owner did not choose, a stand-in whose exit status
  is not the mount's, and the consumer that names a per-dataset unit getting nothing.
- `evidence-hazard-nested-auto.log`, `evidence-hazard-v4.log` (round three): the two hazard arms with
  the flag off.
- `evidence-key-removed.log`, `evidence-flag-removed.log`, `evidence-bound.log`,
  `evidence-bound-late.log`, `evidence-no-key-unit.log`, `evidence-as-written.log` (round three): the
  five failure controls of that round.
- `state-as-written-stopsec.log`, `evidence-as-written-stopsec.log`, `state-host-shape.log`,
  `evidence-host-shape.log`, `state-hazard-flag-on.log`, `evidence-hazard-flag-on.log`,
  `state-bound-stop.log`, `evidence-bound-stop.log`, `timeouts-read-back.log` (round four): the unit
  text with both bounds, this host's shape, the hazard with the flag on, and the cooperative
  hung-start arm.
- `mvm-create-guest.log`, `mvm-destroy-guest.log`, `mvm-create-guest-chosen.log`,
  `mvm-destroy-guest-chosen.log`: both guests' create and destroy records, through
  `mvm --backend libvirt`, by the names the creates printed.
- `host-reads.log`, `apparmor-and-unit-reads.log`: every host fact this revision states, read this
  round.

Some logs named in this revision are not this round's, and each is named with its round:
`start-bound.log`, `dependencies-read.log`, `propagation-cycle.log` and `propagation.log` are round
two's; `restart-provider.log`, `systemd-262-arms.log`, `upholds-arm.log`, `escaping-arm.log` and every
`evidence-*` log of that round are round three's; the nine logs this revision names with round four are
round four's; and the logs this revision names with round five are round five's.

## What the runs do not show

- Not the product handler. Every program is a stand-in; the wrapper's root modes, the readiness
  record, the start probe's own deadline and the unmount half never ran.
- Not this host's kernel or pool for the ZFS arms. The guest runs kernel `6.17.1-300.fc43`, OpenZFS
  `2.4.4-1` and a file-backed pool, and no host dataset was mounted, unmounted, imported, exported or
  changed.
- Not a generated mount unit. No arm made the ZFS mount generator write a unit, because that needs a
  root-written cache file on a machine whose ZED tracks the pool; the branch of the rule that names
  `<escaped checkout path>.mount` as the provider is a source reading plus the host reads, not a run.
- Not the generator's key units as generated: the guests have no generator-written key unit, and the
  two key units both guests ran were written by hand in the generator's names.
- Not ZED. No arm ran `zfs-zed.service`, and the hook is a text this design proposes.
- Not AppArmor. The guest runs Fedora's SELinux, not this host's AppArmor, so the hook's rules are a
  reading here, not a run, and no shape has been loaded.
- Not this host's key path. The guest's checkout is an encryption root of its own with a file key;
  this host's inherits the pool's passphrase, which round four's host-shape arm models but does not
  reproduce in its prompt.
- Not the refusal `dataset-outside-the-checkout`: the stand-in ignores its arguments.
- Not a failed import, a delayed import, or a delayed key load.
  Plan step 19's acceptance text asks for the delayed cases too, and they remain unmeasured.
- Not a busy shutdown. The stop pass ran with one slot attached and nothing holding it; a busy clone,
  a failed stop and an interrupted boot are step 9's arms. The stand-in's stop pass does not force or
  detach, and reports a busy target, which is the design's requirement.
- Not a shutdown arm this round: the reverse stop order was seen at runtime through the provider
  restart, and the boot-time shutdown order is round one's on systemd 255.
- Not the host's own systemd release for the ZFS arms: this round's guest runs 258, and the facts that
  move with a version were measured in round three on 262.2, the host's release.
- Not the mode `mount-path-descendant`, and not the pass's own step: neither exists, and the chosen
  text runs `/usr/bin/zfs mount` until the mode does.
- Not the gate's own refusal under `Wants=` where the checkout itself cannot mount: the key-removed arm
  fails at the key unit first, and the provider-failure arm has a healthy checkout.
- Not the owner's 5 min and 1 min bounds at their real values: the stages arm scaled them to 8 s and
  4 s inside the arm, and said so.
- Not a login held by the gate: the owner's answer removed the ordering, and the stages arm shows
  sessions starting while the gate still runs.

## What else on this host touches the checkout's boot

`zfs-space-guard.service` is `After=zfs-mount.service` and its timer runs every five minutes,
deleting old `/home` snapshots when the pool is over 85 percent full, on a five-minute timer
(`systemctl cat` of both units in this round's `keep/host-reads.log`).
It has no edge to the boot unit and this design gives it none; what matters for a reader is that a
service ordered after the provider can run while the checkout is absent, which is another reason a
provider's own success is not a readiness signal.

## The choices, and the owner's answers

Each subsection states the owner's answer, cites the record, says what it settles in the text, and
gives the measurement the answer rests on.
Every "Record:" line quotes
`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`,
the owner's record of 2026-11-24T08:20:00-05:00.
The options the owner did not choose are kept in "The options not chosen", with the evidence they
carried and what each would have cost.
Every answer is the owner's; this design's part is to write it and to measure what it needs.

### V1. Which unit is the provider: `zfs-mount.service`

Record: "V1: `zfs-mount.service` is the provider",
`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`.
The boot gate names that unit, the one that runs `zfs mount -a` on this machine.

What it settles: the two lines in the boot unit that name the provider, `Wants=zfs-mount.service` and
`After=zfs-mount.service zfs-import.target`, with the dependency type V2's answer fixes.
The rule for another machine is unchanged, because the record names this machine's provider:
where the ZFS mount generator writes `<escaped checkout path>.mount` and lists the checkout dataset,
that unit is named instead, and its own ordering is the generator's
("What decides the provider on another machine").

Measured: with the checkout absent and no key unit, the provider finishes `Result=success`,
`ExecMainStatus=0` while the checkout is not mounted (round four's `evidence-host-shape.log`,
`evidence-no-key-unit.log`); with the covered entry's flag on and a descendant at `canmount=on`, the
provider fails and the datasets outside the failed mountpoint's subtree still mount
(round four's `evidence-hazard-flag-on.log`); and with the owner's `Wants=`, the provider failing
leaves the gate green (kept log `evidence-chosen-provider-fails.log`, quoted under V2).

What it costs, and the design says it wherever a reader meets it: a green provider is not the
checkout's readiness where the datasets under the path carry `canmount=noauto`, and a `Wants=` pull-in
is not a proof of a mount, because `zfs-mount.service` carries
`ConditionPathIsDirectory=/sys/module/zfs`.

What stays open: the branch that names a generator-written mount unit has still never run.

### V2. What the gate requires of the provider: `Wants=` plus `After=`

Record: "`Wants=` + `After=` (Recommended)", which also says the record's own consequence:
"On a provider restart the gate stays up and its drop-in re-runs it, so no attached worktree clone is
unmounted. What the gate does when the provider fails is still unmeasured, and the accepted lost-event
and propagation records, which describe `Requires=`, are amended by this answer."

What it settles: `Wants=zfs-mount.service` in the boot unit, and an amendment to two accepted records:
`doc/decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md` and
`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`, which describe the
gate's `Requires=` as the mechanism that re-runs the pass after a provider restart.
The re-run now rests on the provider drop-in that the lost-event record already installs
("The provider drop-in, from the lost-event record").

Measured this round, with the chosen units, closing the record's own gap:

- **a provider restart leaves everything up.** `systemctl restart zfs-mount.service` took only the
  provider through its stop and start; the provider drop-in then **reloaded** the gate, and every unit's
  invocation id is the same before and after, the nested dataset's included:

  ```text
  == before: invocation ids                    == after: invocation ids
  reconcile-…           45d89811c30c47d29404ac3343544aeb   45d89811c30c47d29404ac3343544aeb
  git-worktree-nested-… 39baa5e6267e4e36a21d00d5d72b954c   39baa5e6267e4e36a21d00d5d72b954c
  git-account-session-… ef82917374fb4a089d17558a11a70ae1   ef82917374fb4a089d17558a11a70ae1
  git-vm-builder-…      9db05a2c52424a59abf8c261660cf321   9db05a2c52424a59abf8c261660cf321
  ```

  and the journal for that restart is three lines (kept log `restart-provider-chosen.log`):

  ```text
  1795546019.758727 zfs-step-19-chosen-2026-11-24 systemd[1]: Stopped zfs-mount.service - Mount ZFS filesystems.
  1795546019.811794 zfs-step-19-chosen-2026-11-24 systemd[1]: Finished zfs-mount.service - Mount ZFS filesystems.
  1795546019.814072 zfs-step-19-chosen-2026-11-24 systemd[1]: Reloading reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts...
  1795546019.846363 zfs-step-19-chosen-2026-11-24 systemd[1]: Reloaded reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts.
  ```

  So **the nested dataset's unit does not restart after a provider restart**, and nothing unmounts.
- **a provider failure no longer fails the gate.** With a fixture dataset whose mountpoint cannot be
  created beneath an immutable directory, the provider ends `Result=exit-code`, `ExecMainStatus=1`
  while the gate runs and finishes green beside it, and both consumers and the nested unit succeed
  (kept log `evidence-chosen-provider-fails.log`, boot `49f8f811-942d-4d48-8cc6-46f3ddee2b3f`):

  ```text
  1795546252.883519 zfs-step-19-chosen-2026-11-24 zfs[817]: cannot mount '/srv/zfs-step-19-fixture/blocked/child': failed to create mountpoint: Operation not permitted
  1795546252.900481 zfs-step-19-chosen-2026-11-24 systemd[1]: zfs-mount.service: Main process exited, code=exited, status=1/FAILURE
  1795546252.900817 zfs-step-19-chosen-2026-11-24 systemd[1]: Failed to start zfs-mount.service - Mount ZFS filesystems.
  1795546253.614650 zfs-step-19-chosen-2026-11-24 systemd[1]: Starting reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts...
  1795546253.681643 zfs-step-19-chosen-2026-11-24 systemd[1]: Finished reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts.
  ```

  (The provider starts twice in that boot, once from `zfs.target` and once from the gate's own
  `Wants=`, and both starts fail while the gate stays green.)

What it costs: no restart propagation, and no dependency failure. A machine that wanted the gate's job
to die with the provider has to read the provider's own unit instead, which V7's `doctor` output
reports.

What stays open: what the gate does when the provider fails **and the checkout itself cannot mount**.
The two arms that could show it each fail earlier: the key-removed arm fails at the key unit, and the
provider-failure arm has a healthy checkout.
The record says the same thing in its own words, and this design does not claim more.

### V3. How the mount units are named in the ordering: `RequiresMountsFor=`

Record: "V3: `RequiresMountsFor=<checkout>`", and the gate carries
`RequiresMountsFor=/home/user/Monochromatic`.

What it settles: the one line in the boot unit, and nothing else in the ordering.
Measured on the guest: one line put `After=` on the root mount and on the home dataset's learned mount
unit, and `Requires=` only on the one that has a fragment (kept log `state-chosen-green.log`; round
three's `systemd-262-arms.log` arm G).
On this host the `Requires=` half attaches to nothing: `-.mount` is `LoadState=loaded` with empty
`FragmentPath` and `SourcePath` (host read, `keep/host-reads.log`), and a requirement is added only to a
mount unit with a fragment.
`[nothing]` This host cannot be booted to show it, so that sentence rests on the file shape and the
rule together, not on a run.

What it costs: the derived set is a manager report rather than a line a reader can check, and the
checkout's own learned unit enters the `After=` only once it exists, so the line orders the stop half
rather than the first start.

### V4 and V9. How the nested datasets are mounted: `noauto`, this unit now, the mode later

Record: "`noauto`, zfs-mount unit now, mode later (Recommended)", with its own detail:
the dataset is `zpcachyos/nosnap/vm-builder-output` at
`/home/user/Monochromatic/package/vm-builder/cli/output`, set to `canmount=noauto`, which the record
names as "the owner's act on an existing dataset", and mounted by a unit that runs
`/usr/bin/zfs mount zpcachyos/nosnap/vm-builder-output` ordered after the gate, replaced by the
wrapper's `mount-path-descendant` mode once it exists.

What it settles: the `canmount=noauto` property, the nested unit's whole text
("The nested dataset's own unit"), and the mode's requirement list, which that section states.

Measured this round, on the chosen unit:

- **it mounts at a boot**: `ActiveState=active`, `Result=success`, `SESSION-DESCENDANT mounted=…`
  (kept log `evidence-chosen-green.log`);
- **it does not restart after a provider restart**, so nothing there can fail it
  (`restart-provider-chosen.log`);
- **it tolerates an already-mounted dataset** in the two ways that can reach it, both by skipping:
  a gate restart, and a boot where the provider mounted the dataset first
  (`restart-gate-chosen.log`, `evidence-chosen-mounted-first.log`);
- **its job fails closed when the checkout's key is away**, with `dependency`, because it requires the
  gate (kept log `evidence-chosen-key-removed.log`).

What it costs: the property change is the owner's act on an existing dataset, and the unit is a second
mounter of the path that the mode is meant to replace; the mode must then carry idempotence and the
`dataset-outside-the-checkout` refusal, which the explicit call cannot.

What stays open: the mode, and therefore the replacement, is the root-modes design's answer about
whether a mode may change a dataset's state named by an argument
(`handler_mode_arguments.rs:112` to `:115`).

### V5. How the propagation window is closed: the gate's first act makes it private

Record: "V5: the gate makes the checkout private first".

What it settles: nothing in the unit text; the step is the pass's own first act, and the design states
it as a requirement on `boot-reconcile`.

Measured in every green boot of every round, the step's own line being `SWEEP propagation=private`;
this round's green boot holds it (kept log `evidence-chosen-green.log`), and the host's path reads
`shared` before any step (`findmnt` in `keep/host-reads.log`).

What it costs: a descendant mounted by the provider before the step is a child of a shared parent for
that window, which the `noauto` property removes.

### V6. What carries the checkout's readiness to the account: each service requires what it reads

Record: "Each service requires the gate (Recommended). Any service that reads the checkout carries
`Requires=` and `After=` on the gate, or on one nested dataset's unit."

What it settles: the two consumer texts under "The account's consumers", and the list `doctor` reports.

Measured: the checkout's consumer finishes after the gate, and the nested consumer after its unit
(kept log `evidence-chosen-green.log`); on a missing key both consumers' jobs fail with `dependency`
while logins proceed (`evidence-chosen-key-removed.log`).

What it costs: `boot-reconcile` exits zero when clones fail, so a clone failure reaches a consumer
through the readiness record and not through the requirement; and each consumer is unbounded, which
now holds only its own dependents.

What stays open: the service list, which is the account's to name.

### V7. What `doctor` and setup report: the whole boot setup

Record: "V7: doctor prints the whole boot setup", with the list the choice carried.

What it settles: `doctor`'s list, which stands as the choice wrote it, with two additions this round's
answers make necessary:

- **the nested unit's condition against the dataset's state**, because a skipped unit and a mounted
  dataset are the same good state, and a mounted dataset with no unit at all is not;
- **the effective `TimeoutStopFailureMode`, read with `systemctl show`**, because the drop-in's file
  name decides whether it wins against the distribution's.

Measured: one unprivileged run of the whole set is kept in round four's `keep/host-reads.log`, and this
round's reads repeat every host fact it needs (`keep/host-reads.log`,
`keep/apparmor-and-unit-reads.log`).

What it costs: a long report whose every line must be kept true as the design changes.

### V8a. Does the before-login ordering stay: no, logins proceed

Record: "Logins proceed (Recommended). The 'before login' ordering
(`Before=systemd-user-sessions.service`) is dropped from the gate, the nested-dataset units and the
consumers."

What it settles: the line is gone from four units; the gate keeps `WantedBy=multi-user.target` and its
default `After=basic.target`, and the readiness reaches the services that require the gate.

Measured with the chosen text and a hung pass, bounds scaled to 8 s and 4 s inside the arm only:
the gate starts at `1795546177.439225`, sessions at `1795546177.881849` and finish at
`1795546177.890506`, and the gate fails at `1795546198.303748`, 20.86 s after its start
(kept log `evidence-chosen-stages.log`, boot `7f0fa2eb-a49f-4025-8633-5ee6256dd6d9`).
So **a login never waits for the gate**, and the price the earlier revisions priced as a held login is
now the gate's own failure arriving after the login.

What it costs: a session can begin with no checkout, and the interactive shell is not one of the
consumers, so nothing refuses it. The record's answer accepts that.

### V8b. Where the bound values come from: the owner's numbers now, measured ceilings at step 22

Record: "5 min start, 1 min stop, then measured (Recommended)", which says the units ship
`TimeoutStartSec=5min` and `TimeoutStopSec=1min` as the owner's decision and step 22 replaces them.

What it settles: the two lines in the boot unit, with the owner's decision beside them, which is what
the project's rule for shipped limits asks.
Measured with those lines and with the bounds scaled to 8 s and 4 s inside the arm: the start bound
fired at `1795546185.553034`, the stop phase spent its bound once per stage a program survived, and the
unit failed at `1795546198.303748` (kept log `evidence-chosen-stages.log`).
At the owner's values the same arithmetic gives the gate's own worst case: 5 min plus one 1 min stop
bound per stage that survives, and no login is held for any of it.

What stays open: the measured ceilings, which step 22 supplies.

### The ZED hook's AppArmor permissions: rules in `local/zed`

Record: "Rules in local/zed", with the owner's acceptance of the cost that every zedlet inherits them.
The rules and the reading behind each are under "AppArmor on this host".

## The options not chosen

Each option the owner did not pick is kept here with the evidence it carried, so a later reader can see
what was weighed without reading four revisions.
Nothing here ships; the unit texts above are the chosen ones.

### V1's other option: a route-owned unit that mounts the checkout explicitly

The accepted lost-event record's and round two's shape:

```ini
# /etc/systemd/system/mount-monochromatic-checkout.service
[Unit]
Description=Mount the checkout dataset explicitly
DefaultDependencies=no
Requires=zfs-load-key@<escaped encryption root>.service
After=zfs-load-key@<escaped encryption root>.service
After=zfs-import.target zfs-mount.service
Before=umount.target
Conflicts=umount.target

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/usr/bin/zfs mount <checkout dataset>
ExecStop=/usr/bin/zfs unmount <checkout dataset>

[Install]
WantedBy=multi-user.target
```

with the checkout dataset at `canmount=noauto`.
The variant without the key requirement printed this when the key was away
(round two's `evidence-route-owned-bare.log`):

```text
zfs[838]: cannot mount 'zfsstep19/home/monochromatic-git-checkout': encryption key not loaded
mount-monochromatic-checkout.service: Main process exited, code=exited, status=1/FAILURE
```

What it would have delivered: a unit whose status and journal entry are the checkout's, and, without
the key requirement, a message naming the dataset and the reason.
What it would have cost: a `noauto` on the checkout dataset, which is a property of a dataset setup
creates (the checkout is a directory in the home dataset today, F1 and F7), plus a second mount actor
whose own stop unmounts the checkout independently of the provider's.
With the key requirement in place its failure is the same dependency failure as the chosen shape's
(round two's `evidence-route-owned-key-removed.log`).

### V2's other options

**`Requires=` plus `After=`**, the text the earlier revisions wrote and the two accepted records
describe. Measured on systemd 258 and 262.2: a provider restart took the gate, the nested unit and both
consumers through their stop pass and started them again, and the stop pass unmounted the attached slot
(round three's `restart-provider.log`, `systemd-262-arms.log` arm F; round five's
`restart-provider-v9c.log` for the nested unit's `filesystem already mounted` failure).
What it would have delivered: the dependency failure when the provider fails, and restart propagation.
What it would have cost: the unmount of every attached clone on every provider restart, which is what
the owner's answer removes.

**`After=` only.** No pull-in, no dependency, no propagation; the drop-in still re-runs the gate after a
provider restart, so it differs from the chosen answer by the pull-in alone.
The arm the earlier revisions cited for it had no edge to the provider at all
(round two's `evidence-no-provider-edge.log:92` to `:95`), so it never showed `After=` alone.

### V3's other options

**The two hand-written ordering lines** (`After=home.mount` and
`After=<escaped checkout path>.mount`): every name visible in the file, but `home.mount` is wrong on a
machine whose home dataset sits elsewhere.
Measured on the guest, whose home dataset is mounted at
`/srv/zfs-step-19-fixture/home` and whose unit is `srv-zfs-step-19-fixture-home.mount`
(round four's `state-as-written-stopsec.log`), and the learned unit's name must be computed per
machine.

**No mount-unit edge and a prose statement.** Nothing to keep in step, and the stop half loses its only
statement. No shutdown arm has run in any round, so its effect is unmeasured either way.

### V4's other option: leave the datasets to the provider

Measured three ways: with the covered entry's flag off and the descendant at `canmount=on`, the
provider creates the mountpoint folder inside the exposed home underlay and mounts the descendant there
(round three's `evidence-hazard-nested-auto.log`); with `canmount=noauto` it mounts nothing
(`evidence-hazard-v4.log`); with the flag on, the provider cannot create the mountpoint beneath the
immutable entry, so it fails, and the datasets outside the failed subtree still mount (round four's
`evidence-hazard-flag-on.log`).
What it would have delivered: no property and no unit.
What it would have cost: an account-visible mount created by the provider under the exposed home
underlay whenever the flag is off, and a provider failure whenever it is on.

### V5's other option: order each descendant after the gate's propagation step

The window closed by ordering rather than by a step. It has no mechanism the provider obeys, because
the provider cannot be ordered to a step inside another unit; only the `noauto` property (V4) stops it
mounting a descendant first. Not run: there is no step to order, which is why the chosen answer keeps
the step.

### V6's other options

**Gating the account's user manager** (`user@<uid>.service` with `Requires=` on the gate): everything
the account starts is behind the gate, and a refused gate leaves the account with no session at all,
which is why it was not run.

**Relying on login ordering alone**: measured, sessions start after a failed gate
(round four's `evidence-host-shape.log`, `evidence-hazard-flag-on.log`; round three's
`evidence-key-removed.log`, `evidence-flag-removed.log`, `evidence-bound.log`), so a session would begin
with no checkout and no signal. With V8a's answer the ordering line is gone anyway, so this option is
not available any more.

### V7's other option: the provider state only

```ini
provider: zfs-mount.service
```

It reads green on a machine with the wrong `canmount`, no `ExecReload=`, a hook name ZED ignores, a
profile that cannot exec the hook, a disabled ZED, key lines naming a unit the boot does not install, or
a stop-failure drop-in that loses to the distribution's.

### V8a's other option: keep the before-login ordering

The fifth revision's option A: the line on all four units, with bounds on each.
Measured in round five with a hung pass and a hung stop pass: a 20 s start bound and a 3 s stop bound
held logins 23.54 s where the pass died at SIGTERM (round four's `evidence-bound-stop.log`), and 20.53 s
where it did not (round five's `evidence-stages.log`); and the per-dataset unit and both consumers have
no start bound at all (round five's `state-green.log`), so the hold through them would be unlimited.
What it would have delivered: the checkout ready, or known not to be, before any account session.
What it would have cost: a login held for as long as four units' bounds, one of them unbounded, which is
what the owner's answer removes.

### V8b's other options

**The measured ceilings, once step 22 measures them** is where the chosen answer lands anyway, so it is
not a rival: the owner's numbers are this answer's first half.
**No bound and the ordering dropped**: this answer's ordering with no number, leaving a hung pass to be
reaped by a person.
**No bound and the ordering kept**: the combination no boot unit ordered before sessions may have; it is
gone with V8a's answer.

### V9's other options

**A mode per dataset, `mount-path-descendant --dataset <dataset>`**, is what the chosen answer becomes
once the mode exists; the design keeps its requirement list under "The nested dataset's own unit".
**The boot unit's own pass mounts them**, with no per-dataset unit: `boot-reconcile` has no such step
(`rg 'zfs mount'` over its folder matches comments only), and it needs a dataset list, a failure rule
for the readiness record and the exit status, and idempotence on `ExecReload=`.
Round three measured a stand-in for that shape whose exit status was an `echo`'s, and the consumer that
names a per-dataset unit got nothing at all (`evidence-v9b.log`).

### The AppArmor shapes the owner did not choose

**A dedicated profile for the installed file, entered by one `px` rule in `local/zed`**: a separate
audit label, its own mode, and a ruleset reviewable without reading ZED's; it costs a second profile to
install and keep in step, a `px` that fails the exec closed when the target profile is not loaded, and
the same kind of rule in `local/zed` anyway, so it is rules plus a profile rather than a profile
instead of rules.

**A child profile inside `local/zed`, entered with `cx`**: the same separate ruleset without a second
file; it costs a nesting this host's other local files never use, so nothing here shows it loading.

**`pix` instead of `px`**: it removes the fail-closed cost and the guarantee, because a machine where the
profile did not load runs the hook with the current profile's permissions.

**An unconfined start (`Ux`)**: what the boot unit and the propagation service already do; it costs
every guarantee the section is about.

**`px` to a profile that already exists**, as `/etc/apparmor.d/local/gh` line 2 does for the wrapper
with `px -> git`: no new file, and the hook would run under rules written for the `zfs` or `mount.zfs`
program, not for the wrapper's own modes; whether they would do is unmeasured.

## What each part waits on

Named so a reader can see which piece is blocked, and on whom.
The choices are settled; what waits is the build each chosen piece needs.

- The installed file's own name, and whether it is named for the hook or for the modes: the root-modes
  design's choice 3.
- The hook's exact spelling: that design's choice 2 and the source constant `ZEDLET_INSTALL_PATH`.
- The two words `<root-words>` in every unit line: that design's choice 1, option A, and its choice 2,
  whose constant is `ROOT_GROUP_WORD`.
- Whether a root start under another name runs the wrapper: that design's choice 4.
- The mode `mount-path-descendant`, with `--dataset <dataset>` and idempotence on an already-mounted
  dataset: that design's mode list, and step 19's build. It needs that design to say whether a mode
  may change a dataset's state by name at all, and it needs the new refusal
  `dataset-outside-the-checkout`.
  Until it lands, the nested dataset's unit runs `/usr/bin/zfs mount` (V4 and V9's answer).
  (Written 2026-11-25:
  the mode is built and its refusals are named,
  `no-recorded-checkout` among them,
  which refuses until root has run `record-checkout`;
  the unit's line becomes one line naming the mode once the checkout migration has run and the record
  stands,
  `ExecStart=<the root-owned file> monochromatic-git-policy-cli zfs-root mount-path-descendant --dataset <dataset>`,
  and the order is
  `doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`,
  "The order the nested dataset's unit changes in".)
- `canmount=noauto` on `zpcachyos/nosnap/vm-builder-output`: the owner's act on an existing dataset
  (V4), and step 20's snapshot coverage and the setup runbook must state it too.
  With this design's unit installed the dataset is mounted at boot; nothing else mounts it.
- The measured ceilings that replace `TimeoutStartSec=5min` and `TimeoutStopSec=1min`: plan step 22
  (V8b).
- The rules in `/etc/apparmor.d/local/zed`: nothing applies them until the owner enables
  `zfs-zed.service`, which is `disabled` and `inactive` here (the ZED hook's answer).
- The account's own service list under `Requires=`/`After=` on the gate or a nested unit (V6).
- The amendment to the two records that describe `Requires=` on the provider (V2):
  `doc/decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md` and
  `doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`.
  Those are their owners' to amend; "Edits this design suggests" says what to change.

## Edits this design suggests

Suggestions only; this round edits neither record.
Every edit below now presumes the owner's answers, and the record that carries them is
`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`.

To plan step 19, "Open items", which reads:

> Two parts the design leaves open block its acceptance arms:
> the checkout provider stays conditional in "One-time move and checkout boot protection", which names "the
> protected `home-user-Monochromatic.mount` if using a generated mount unit, or `zfs-mount.service` where it mounts
> the checkout", and "Checkout boot mount protection" says to prove the failure control and production ordering
> before enabling the gate.

the suggestion is to replace that sentence with the fixed provider the owner chose, the key unit
and a pointer:

> The checkout provider is fixed on a machine of this host's shape as `zfs-mount.service`, whose `zfs mount -a`
> mounts the checkout because the ZFS mount generator writes no unit here; where the generator's cache file exists
> and names the checkout dataset, the provider is the generated `<escaped checkout path>.mount` instead.
> The checkout's key-loading unit is the generator's `zfs-load-key@<escaped encryption root>.service` where one
> exists, and the provider's status is not a readiness signal for the checkout where the datasets under the
> checkout path carry `canmount=noauto`.
> The failure control, the production ordering, the hazard arms and the provider-failure arms are measured in
> `doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`, which also fixes the units'
> whole text, the owner's start and stop bounds, the key and provider drop-ins, the nested dataset's unit and its
> already-mounted condition, and the exclusion every dataset mounted under the checkout path needs.
> The gate wants the provider rather than requiring it, so a provider restart reloads the gate instead of
> restarting it, and a provider failure no longer fails the gate.

To plan step 19, "What it builds", the sentence "The gate that orders every path descendant after a verified
checkout mount" needs the condition this design measured, and no longer names the sessions unit:

> The gate that orders every path descendant after a verified checkout mount, where each such dataset carries
> `canmount=noauto` so that the automatic mount cannot mount it first, and where a unit per descendant, ordered
> after the gate, runs an explicit `zfs mount` (later the handler's `mount-path-descendant` mode) and skips itself
> when its dataset is already mounted at its own mountpoint.

To the design's "Boot and shutdown unit", the unit text in this document's "The boot and shutdown
unit", which is the text the owner's answers settle: `Wants=` on the provider, `RequiresMountsFor=`,
no before-login line, and the owner's 5 min and 1 min bounds.

To the design's "One-time move and checkout boot protection", the paragraph that begins "The account can replace
`~/Monochromatic` while it is unmounted, regardless of the entry's owner" needs the ancestor condition:

> The account can replace an ancestor of the checkout path while the checkout is unmounted, wherever it can write
> that ancestor's parent; the immutable covered entry refuses the entry's own rename, removal and inner writes, so
> the protection holds only while the account cannot write any directory from the entry's parent up to the home
> dataset's root. At `/home/user/Monochromatic` that holds: `/home` is root's, and `/home/user` is the account's
> but its own parent is not. At a deeper account-owned path such as `~/projects/Monochromatic` it does not hold, and
> the entry belongs at a path where the account cannot write those directories.

and the paragraph ending "The probe did not replace an ancestor of the protected entry or exercise unattended boot."
can say what has now been run:

> Item 8's two gaps are measured in the guests of
> `doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`:
> an unattended boot with the checkout's mount made to fail mounts nothing under the covered entry while the
> immutable flag is set and leaves the provider failing, the same boot with the flag removed created the folders in
> the home underlay and mounted the output dataset into them, and an account-owned ancestor was renamed with a
> file planted at the entry's path while a root-owned ancestor refused it.

To the design's "Checkout boot mount protection" decision, whose last sentence reads "Failed ordinary mount and
unattended boot remain unmeasured.", the measurement now exists and the sentence becomes a pointer to this design's
"The failure controls".

To the design's `ExecStart`, `ExecStopPost` and hook text, the program name follows the one-program decision
(`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`); this design writes them against
`<root-file>` and `<root-words>` and says so.

To the lost-event design's "The hook, in one form a sibling can cite", one amendment, because the owner took
`Wants=`: its sentence that the boot unit's `Requires=` on the provider makes a provider restart re-run the pass
describes a unit this design no longer installs, and the re-run now rests on the same design's drop-in, which the
record already installs (measured: `restart-provider-chosen.log`).

To the propagation design's "What becomes of `Type=oneshot` with `RemainAfterExit=yes`", the same amendment and
the same reason, with the same measurement behind it.

To the root-modes design, two additions to its mode list, and one question before them: whether a mode may change a
dataset's state named by an argument, which `HandlerMode`'s doc forbids today. If it may,
`mount-path-descendant --dataset <dataset>`, idempotent on an already-mounted dataset, refusing a dataset whose
mountpoint lies outside the checkout with `dataset-outside-the-checkout`, and the reason the nested dataset's unit
needs it. If it may not, the unit keeps running `/usr/bin/zfs mount`, which is what the owner's answer installs
until then, and the wrapper needs no new mode at all.

## Facts and what each rests on

- F1. The checkout path has no mount of its own; `/home` is the home dataset, mounted `shared`.
  Basis: host read, `keep/host-reads.log`, `findmnt`.
- F2. The ZFS mount generator is installed, `/etc/zfs/zfs-list.cache` is absent, and
  `/run/systemd/generator` holds no ZFS mount unit. Basis: host read.
- F3. `zfs-zed.service` is `disabled` and `inactive`. Basis: host read, `systemctl is-enabled`,
  `is-active`.
- F4. `zfs-mount.service` runs `/usr/bin/zfs mount -a` with `ConditionPathIsDirectory=/sys/module/zfs`,
  `Before=local-fs.target` and `WantedBy=zfs.target`; `zfs-import-cache.service` runs
  `/usr/bin/zpool import -c /etc/zfs/zpool.cache -aN $ZPOOL_IMPORT_OPTS`; `ZPOOL_IMPORT_OPTS` is empty;
  the four ZFS units are enabled. Basis: host read, `systemctl cat` of both units and
  `/etc/default/zfs` in `keep/host-reads.log`.
- F5. `/usr/lib/systemd/system/zfs-load-key.service` is a `/dev/null` symlink owned by
  `zfs-utils 2.4.4-3`, there is no `zfs-load-key@.service` file, and the unit is `masked`. Basis: host read.
- F6. The pool's encryption root is `zpcachyos`, its key is a passphrase prompted at import
  (`keylocation=prompt`), and the home dataset reads `encryptionroot=zpcachyos`, `keylocation=none`,
  `keystatus=available`. Basis: host read, `zfs get … encryptionroot,keylocation,keystatus` of
  `zpcachyos`, `zpcachyos/ROOT/cos` and `zpcachyos/ROOT/cos/home` in `keep/host-reads.log`.
- F7. `home.mount` is learned from the mount table (`LoadState=loaded`, `FragmentPath=` empty,
  `SourcePath=/proc/self/mountinfo`), and the checkout's own unit name is `not-found`.
  Basis: host read, `systemctl show` for both names in `keep/host-reads.log`.
- F8. `/home` is root's `0755`, `/home/user` is the account's `0700`, the checkout path is the account's `0755`.
  Basis: host read.
- F9. `zfs-space-guard.service` is `After=zfs-mount.service`, and its timer is `OnCalendar=*:0/5`.
  Basis: host read, `keep/host-reads.log`.
- F10. This host runs systemd `262.2-1-arch`, and has no `service.d` directory at either
  `/usr/lib/systemd/system/` or `/etc/systemd/system/`, so nothing here sets
  `TimeoutStopFailureMode` for every service. Basis: host read, `keep/host-reads.log`.
- F11. The guest runs Fedora Linux 43, systemd `258.11-1.fc43`, kernel `6.17.1-300.fc43.x86_64`,
  OpenZFS `2.4.4-1` from the pinned tarball, SELinux enforcing, agent in `unconfined_service_t`.
  Basis: kept logs `prepare-readback.log`; round four's `evidence-as-written-stopsec.log`.
- F12. The hazard with the covered entry's flag on and `canmount=on` makes the provider fail:
  `failed to create mountpoint: Operation not permitted`, `Result=exit-code`, `ExecMainStatus=1`, and
  under the `Requires=` the earlier revisions wrote, the gate's job failed with `dependency` before its
  own check. Basis: round four's `evidence-hazard-flag-on.log`.
- F13. The fourth revision's unit text, both bounds included, boots green with the order gate,
  nested unit, its consumer, the account consumer and sessions. Basis: round four's
  `evidence-as-written-stopsec.log`.
- F14. The checkout's key unit failing fails the gate, both consumers and the nested unit with
  `dependency`, and sessions start after (round three's `evidence-key-removed.log`, journal lines
  `1795507758.973672` to `1795507760.676330`); the chosen units do the same, measured this round
  (`evidence-chosen-key-removed.log`).
- F15. With the checkout unmountable and the dataset under the path at `canmount=noauto`, the provider
  finishes `Result=success`, `ExecMainStatus=0`, both with a failing key unit and with no key unit at
  all. Basis: round three's `evidence-key-removed.log`, `evidence-no-key-unit.log`; round four's
  `evidence-host-shape.log`.
- F16. With no key-loading unit, the gate refuses `checkout-not-the-dataset` with status two.
  Basis: round four's `evidence-host-shape.log`.
- F17. With the flag removed, the gate refuses `covered-entry-not-immutable` with status two and its
  dependents' jobs fail with `dependency`. Basis: round three's `evidence-flag-removed.log`.
- F18. With the flag absent, the checkout's mount failing and the descendant at `canmount=on`, the
  provider creates the mountpoint folder in the home underlay and mounts the descendant there.
  Basis: round three's `evidence-hazard-nested-auto.log`.
- F19. The same with `canmount=noauto` mounts nothing and creates nothing. Basis: round three's
  `evidence-hazard-v4.log`.
- F20. A 20 s start bound kills a hung pass, records `timeout`, runs the stop pass, and sessions start
  after the failure. Basis: round three's `evidence-bound.log`, `evidence-bound-late.log`.
- F21. A oneshot's default start timeout is `infinity`. Basis: round three's `systemd-262-arms.log`
  arm A, and round two's `start-bound.log`.
- F22. The start bound is per phase: a 4 s bound killed a 6 s pre-phase at 4033 ms, and a 4 s bound let a
  3 s pre plus a 3 s start finish at 6037 ms. Basis: round three's `systemd-262-arms.log` arms B1, B2.
- F23. A reload shares the start bound: a 3 s bound failed a 6 s `ExecReload=` at 3102 ms while the unit
  stayed `Result=success`. Basis: round three's `systemd-262-arms.log` arm C, and `systemd.service(5)`'s
  `TimeoutStartSec=` sentence at v262 (F41).
- F24. The stop pass extends the hold: after a 3 s bound fired, a 5 s `ExecStopPost=` delayed the unit's
  failure and its dependent's start by 5.01 s. Basis: round three's `systemd-262-arms.log` arm D.
- F25. `TimeoutStopSec=` bounds the stop pass: a 2 s value killed a 5 s `ExecStopPost=` after 2.07 s on
  the host's user manager, and a 3 s value cut a ten-minute stop pass 3.25 s after the start bound fired
  in a boot. Basis: round three's `keep/host-user-manager-stopsec.log`; round four's
  `evidence-bound-stop.log`.
- F26. `RequiresMountsFor=` puts `After=` on every prefix mount unit and `Requires=` only on a unit
  that has a fragment. On the guest that was `-.mount`; on this host `-.mount` is `LoadState=loaded`
  with empty `FragmentPath` and `SourcePath`, so the requirement does not attach here.
  Basis: kept log `state-green.log`; round three's `systemd-262-arms.log` arm G; host read,
  `keep/host-reads.log`; the fragment rule as round four's read of v262 `unit.c` gives it.
- F27. Under `Requires=`, a provider restart restarted the gate, the nested unit and both consumers,
  and the stop pass unmounted the attached slot. Basis: round three's `restart-provider.log`, four
  units; round five's `restart-provider-v9c.log` for the nested unit's failure. The chosen `Wants=`
  changes it: `restart-provider-chosen.log`.
- F28. The same propagation reaches a four-unit chain on 262.2. Basis: round three's
  `systemd-262-arms.log` arm F.
- F29. `Requires=` on the learned mount unit alone, with both drop-ins off, leaves the gate's boot job
  unrun on 258: no journal line, `inactive`, `Result=success`. Basis: round three's
  `evidence-learned-unit-only.log`.
- F30. The same on 262.2 at a boot, with the learned unit created during the transaction: the requiring
  unit is `inactive` with `Result=success` and no journal line. Basis: round three's
  `boot-arm-learned-unit.log`.
- F30b. Requiring the provider beside the learned unit, with both drop-ins absent, still leaves the
  requiring unit unrun at a boot. Basis: round three's `boot-arm-learned-with-provider.log`.
- F31. A consumer that requires a unit the boot does not install gets nothing: `inactive`,
  `Result=success`, no journal line. Basis: round three's `evidence-v9b.log`.
- F32. The stand-in pass mounting a descendant in its own process works: `descendant-mount exit=0`, then
  `descendant-mounted=…`. Basis: round three's `evidence-v9b.log`. Its exit status is the echo's, so it
  is not evidence that a failed mount fails the pass.
- F33. A consumer requiring the per-dataset unit runs after it and sees the mount. Basis: round four's
  `evidence-as-written-stopsec.log`.
- F34. `Upholds=` and `BindsTo=` work on a mount unit learned from the mount table: the service starts
  with the mount unit's activation and stops with its deactivation. Basis: round three's
  `upholds-arm.log`; round two's `dependencies-read.log`, `propagation-cycle.log`.
- F35. An escaped unit name must be spelled as `systemctl` prints it; doubling the backslashes names a
  unit that does not exist, and `systemd-analyze verify`'s "not found" line is a verify-time artifact
  for a learned unit. Basis: round three's `escaping-arm.log`.
- F36. ZED starts a zedlet with no arguments and passes its own environment: `execle(path, prog, NULL,
  env)`. Basis: source reading, `zfs-2.4.4` `cmd/zed/zed_exec.c:168` and `zed_event.c`'s environment
  block.
- F37. `MODE_WORDS` holds five words; `HandlerMode`'s doc says a mode is chosen by a leading word and
  that no argument names a folder to mount or a dataset to change, while `holder-scan` already takes
  `--dataset`; no argument is the attaching sweep; a single group word is refused as an unknown mode.
  Basis: source reading, `handler_mode_arguments.rs`, `MODE_WORDS`, `parse` and the `HandlerMode` doc at
  `:113` to `:115`.
- F38. An explicit `zfs mount` of an already-mounted dataset prints `filesystem already mounted` and
  returns 1, while `-a` returns 0; `-a` skips a `noauto` dataset and an unavailable key, while the
  explicit form refuses. Basis: source reading, `zfs-2.4.4` `cmd/zfs/zfs_main.c`, `share_mount_one` and
  the `OP_MOUNT` branch; measured on the message in round two's `propagation.log:99`.
- F39. The generator's key unit carries `DefaultDependencies=no`, `Wants=`/`After=zfs-import.target`,
  `StandardOutput=null`, an `ExecStop` that unloads the key and no `[Install]`; `Before=zfs-mount.service`
  is on the generated mount unit. Basis: source reading, `zfs-mount-generator.c`, the `keyloadunit`
  block.
- F40. The start probe's deadline is 350 ms with a 231 ms margin over the slowest measured worker phase.
  Basis: source reading, `probe_deadline.rs`, `PROBE_DEADLINE` and `PROBE_DEADLINE_MARGIN`.
- F41. The start timeout also applies to reloads, and `TimeoutStopSec=` bounds the stop phase.
  Basis: source readings, `systemd.service(5)` at v262 (the `TimeoutStartSec=` and `TimeoutStopSec=`
  entries) and `service.c`'s per-phase `service_arm_timer(s, ..., s->timeout_start_usec)`.
- F42. `boot-reconcile` exits zero when clones fail and only a refusal returns an error; it has no step
  that mounts a dataset under the checkout path; `shutdown-unmount` always refuses.
  Basis: source readings, `boot_reconcile_mode.rs:158` to `:164` and its module doc,
  `shutdown_unmount_mode.rs`'s `run`.
- F43. The root-modes design's start rule is its choice 1, option A: the two words
  `monochromatic-git-policy-cli zfs-root` followed by a mode word, on any file; under the hook's own
  name, no argument is the sweep. `ROOT_GROUP_WORD` is its choice 2's constant.
  Basis: source reading,
  `doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`, "Choice 1" and its
  `root_mode_names.rs` module text.
- F44. Plan line 2569 names no unit per path descendant. Basis: source reading,
  `doc/planning/cli-git-zfs-worktree-implementation-plan.md`.
- F45. The one-program record holds no rule that root's mounts come from the one program; it fixes one
  executable and leaves the modes' reach to a design. Basis: source reading,
  `doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`.
- F46. The lost-event design states the provider restart's propagation as a fact and names its price,
  and the propagation design calls it "the one job-driven case the unit covers today".
  Basis: source readings, `doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`, its `F17` and
  the section around it;
  `doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
  "What becomes of `Type=oneshot` with `RemainAfterExit=yes`" and its `F39`.
- F47. The `zed` profile's exec permissions are `mr` on itself, `rix` on fourteen utilities, `rpx` on
  `zfs` and `zpool`, and `rix` on `lib/zfs-linux/zed.d/*.sh` only; it grants `capability sys_admin` and
  no other capability; it has write rules for three `/run` paths, `/etc/zfs/zfs-list.cache`, its own
  temp files, one sysfs parameter, one `/proc` comm path and `/dev/zfs`; it has no mount rule, no rule
  under the account's paths, and no exec rule matching this host's zedlet folder. Basis: profile
  reading, `keep/apparmor-and-unit-reads.log`.
- F47b. The two `rpx` rules land in different profiles: `zpool`'s own file is
  `profile zpool … flags=(complain)` at `/etc/apparmor.d/zpool:11`, and `/etc/apparmor.d/mount-zfs` is
  the profile for `mount.zfs`, which neither rule names. Basis: profile reading,
  `keep/apparmor-and-unit-reads.log`.
- F48. The built-in `mvm` registry's newest systemd is Fedora 43's 258, and a custom template reaches
  the Rawhide image with systemd 262.2. Basis: source reading, `package/mvm/cli/src/registry.ts`, and
  round three's `mvm-create-guest2.log`, `systemd-262-arms.log`.
- F49. `apparmor.service` is enabled and active, `/etc/apparmor/modes` is `default enforce` with
  `fsp no`, `zed` is in `install.db`, `/usr/share/apparmor/flags.d/` carries no entry for `zed` or
  `zfs` and does carry `zpool complain`, and there is no `/etc/apparmor.d/local/zed`. Basis: host read,
  `keep/apparmor-and-unit-reads.log`.
- F50. The loaded-profile list is root-only: an unprivileged read of
  `/sys/kernel/security/apparmor/profiles` answers `Permission denied`. Basis: host read.
- F51. `/usr/lib/zfs-linux` does not exist and this host's zedlets are `/usr/lib/zfs/zed.d`'s sixteen
  entries, with `/etc/zfs/zed.d` holding symlinks to them. Basis: host read.
- F52. This host has no profile file for `systemd` or `systemd-user`, so a unit's program starts under
  whatever profile its own path carries and none other. Basis: host read, and the attachment mechanism
  in `doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`, "Root cause".
- F53. The boot unit's reload re-runs the reconcile, and a provider restart with the gate on
  `Requires=` restarts it rather than reloading it. Basis: round two's `reload-gate.log`, `dropin.log`.
- F54. Every audit record this round's guest's log held was `systemd-getty-generator`'s
  `dac_override` denial, all twenty-four inside that guest's own first boot, and the arm's own boot had
  none. Basis: kept log `evidence-green.log`, "the boots this log holds" and the record listing.
- F54b. The fourth round's log printed thirteen records where its own counts, and `grep --count`, said
  fourteen. Basis: round four's `evidence-as-written-stopsec.log`.
- F55. A drop-in's **file name** decides the parse order across all drop-in directories: the same
  `TimeoutStopFailureMode=terminate` is overridden as `10-terminate.conf`, wins as
  `zz-checkout-stop-failure-mode.conf`, and loses when it stands in the unit file; masking the
  type-level file wins as well. Basis: kept log `stopmode-shapes.log`, four shapes.
- F56. A start bound's failure runs a stop phase of three bounded stages when the programs ignore
  SIGTERM: the SIGTERM stage, the `ExecStopPost=` stage and the final SIGTERM stage, each bounded by
  `TimeoutStopSec=`, with the final SIGKILL stage ending at once. Measured: 8 s + 4 s + 4 s + 4 s, and
  20.53 s from the unit's start to the sessions unit. Basis: kept log `evidence-stages.log`.
- F57. The nested dataset's unit and both account consumers have `TimeoutStartUSec=infinity` and
  `TimeoutStopUSec=45s`, and carry no before-login line under the owner's answer.
  Basis: kept log `state-chosen-green.log`; round five's `state-green.log` for the same bounds before
  the before-login line was dropped.
- F58. A per-dataset unit running an explicit `zfs mount <dataset>` after the gate mounts the dataset at
  a boot (kept log `evidence-v9c.log`), and under `Requires=` on the provider it fails with
  `filesystem already mounted` after a provider restart, taking its consumer's job down with
  `dependency` (round five's `restart-provider-v9c.log`). The chosen `Wants=` and the condition
  answer that arm.
- F59. This host has one dataset under the checkout path, `zpcachyos/nosnap/vm-builder-output`, at
  `canmount=on`, `mounted yes`, mountpoint
  `/home/user/Monochromatic/package/vm-builder/cli/output`, propagation `shared`. Basis: host read,
  `keep/host-reads.log`.
- F60. The fifth revision's green boot of its own text, with the stop-failure drop-in beside it, runs
  gate, account consumer, nested unit, its consumer, then sessions, and reports
  `TimeoutStopFailureMode=terminate` while Fedora's `service.d/10-timeout-abort.conf` is installed and
  unmasked. Basis: round five's `state-green.log`, `evidence-green.log`.

- F61. Under the chosen `Wants=`, a provider restart leaves the gate, the nested unit and both consumers
  with their invocation ids unchanged, and the provider drop-in reloads the gate.
  Basis: kept log `restart-provider-chosen.log`.
- F62. A provider failure with a healthy checkout leaves the gate green: the provider ends
  `Result=exit-code` with `ExecMainStatus=1` while the gate finishes `Result=success` and every consumer
  and the nested unit succeed. Basis: kept log `evidence-chosen-provider-fails.log`.
- F63. `ConditionPathIsMountPoint=!<path>` skips a unit whose path is already a mountpoint, a skipped
  unit counts as success for the consumer that requires it, and systemd logs
  `skipped, unmet condition check ConditionPathIsMountPoint=!…`.
  Basis: kept logs `restart-gate-chosen.log`, `evidence-chosen-mounted-first.log`.
- F64. With the before-login line dropped, sessions start 0.44 s after the gate's start and 20.4 s
  before its failure under a hung pass. Basis: kept log `evidence-chosen-stages.log`.
- F65. The chosen unit set boots green with `Wants=` and the owner's bounds, ordering provider, gate,
  the consumers and the nested unit, then sessions. Basis: kept log `evidence-chosen-green.log`.
- F66. With the checkout's key away, the chosen set fails closed at the key unit: the gate, both
  consumers and the nested unit fail `dependency`, and sessions start 1.5 s later.
  Basis: kept log `evidence-chosen-key-removed.log`.

## Sources read

On this host, read-only, 2026-11-24; the whole output is in this round's `keep/host-reads.log` and
`keep/apparmor-and-unit-reads.log`:
`zfs get` for `zpcachyos`, `zpcachyos/ROOT/cos` and `zpcachyos/ROOT/cos/home` (encryption,
keyformat, keylocation, keystatus, encryptionroot, canmount), `findmnt` for `/home`,
`systemctl is-enabled` and `is-active` for the ZFS units, `zfs-zed.service` and `apparmor.service`,
`systemctl is-enabled zfs-load-key.service`, `ls` of `/etc/zfs/zfs-list.cache`,
`/run/systemd/generator`, `/usr/lib/systemd/system/zfs-load-key*`, `/usr/lib/zfs-linux` and
`/usr/lib/zfs/zed.d`, `/etc/apparmor.d/zed` with line numbers, `/etc/apparmor.d/zfs`,
`/etc/apparmor.d/mount-zfs`, `/etc/apparmor/modes`, `/etc/apparmor/install.db`,
`/usr/share/apparmor/flags.d/`, `/etc/apparmor.d/local/`, `/etc/apparmor.d/disable/`,
`/sys/kernel/security/apparmor/`, and `/proc/self/attr/current`.

The record this revision writes: `doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`,
read whole, and the two records it amends,
`doc/decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md` and
`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`.

In `main`: `package/git-policy/cli/src/native/worktree_clone_mount_handler/` (the module doc,
`boot_reconcile_mode.rs`, `handler_mode_arguments.rs`, `handler_install_paths.rs`,
`shutdown_unmount_mode.rs`), `handler_mode_arguments.rs`'s `MODE_WORDS` and `parse`,
`doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`,
`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`,
`doc/decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md`,
`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`,
`doc/troubleshooting/mvm-fedora-guest-confined-by-selinux.md`, and
`package/git-policy/cli/src/native/start_probe/probe_deadline.rs`.

In this round's guest and its scripts: `systemd.unit(5)`'s `ConditionPathIsMountPoint=` entry, as the
installed systemd `258.11-1.fc43` implements it, measured rather than quoted
(`restart-gate-chosen.log`, `evidence-chosen-mounted-first.log`).

In the earlier rounds' kept copies: OpenZFS `zfs-2.4.4` `cmd/zfs/zfs_main.c`,
`etc/systemd/system-generators/zfs-mount-generator.c`, `cmd/zed/{zed_exec,zed_event}.c` and
`man/man8/zed.8`; systemd v258 and v262 `src/core/{job,mount,service,transaction,unit}.c` and the
`systemd.{service,exec,unit}` manuals; and this host's `apparmor.d(5)` for the exec modes `ix`, `px`
and the read permission an interpreted program needs.

## Measurements and logs

This round's folder: `~/temp/agent/zfs-step-19-fifth-revision-2026-11-24/`, which holds both the fifth
revision's round and this continuation of it.

- `keep/guest-scripts/`, every guest step as its own script, so each arm is reproducible:
  `01-prepare.sh`, `02-fixture.sh`, `03-fix-fixture.sh`, `10-units.sh`, `11-state.sh`,
  `12-evidence.sh`, `13-reboot.sh`, `99-ready.sh` from the fifth revision's round, and
  `21-chosen-state.sh` and `22-chosen-evidence.sh` from this one.
  `01-prepare.sh` and `02-fixture.sh` are round three's, unchanged and re-run; `03-fix-fixture.sh` and
  `13-reboot.sh` are round four's, unchanged; `21-chosen-state.sh` writes the chosen gate, the nested
  unit with its condition and the two consumers, and `22-chosen-evidence.sh` reads the invocation ids,
  the condition results and the sessions timing beside the fifth revision's evidence step.
- `keep/logs/`, every step's whole output, named for its arm and its guest.
- `keep/host-reads.log` and `keep/apparmor-and-unit-reads.log`, the host reads with their exact output,
  every host fact this revision states.
- `keep/state-so-far.md`, the round's running state.

Two guests, each created and destroyed through `mvm --backend libvirt` by the name its own create
printed: `zfs-step-19-fifth-2026-11-24` (create in `mvm-create-guest.log`, 2026-11-24T13:40:27Z, destroy
in `mvm-destroy-guest.log`) and `zfs-step-19-chosen-2026-11-24` (create in
`mvm-create-guest-chosen.log`, 18:41:38Z, destroy in `mvm-destroy-guest-chosen.log`, 18:51:37Z).
`mvm-list-after-destroy-chosen.log` reports no guest.

Two logs are partial streams: `prepare.log` and `prepare-chosen.log` are each build's output as the
guest agent delivered it, cut short before the script's own read-back; `prepare-readback.log` and
`prepare-chosen-readback.log` are the same scripts' second runs, which repeat every read-back in full
because the build was already done.
`fixture.log` and `fixture-chosen.log` end at the busy unmount round four documented, and each is
completed by its `03-fix-fixture.sh` run.

## Revisions

The first revision was written after the first gate review; the second after a same-vendor re-read of
that revision; the third after a second gate review; the fourth after a same-vendor read of the third;
the fifth after a same-vendor read of the fourth; and this one after the owner answered every choice.
Earlier revisions' narratives of their own folds are folded into the text they changed and are not
repeated here.

The fifth revision, kept for its measurements:
- stated the worst case as the probe plus the start bound plus one stop bound per stop stage whose
  programs survive SIGTERM, and measured the three-stage case in a boot;
- moved the stop-failure mode into a drop-in whose file name sorts after the distribution's, after
  measuring that the directory is not what decides;
- split V8 into the ordering question and the values question, and stated the nested and consumer
  units' unbounded defaults;
- wrote V9's option C with the ordering lines the other two carried, booted it, and measured what a
  provider restart did to it under `Requires=`;
- weighed six AppArmor shapes, and added the reading that a provider's failure leaves datasets outside
  the failed subtree mounted;
- re-posed every choice with unit lines, a command and what it printed or a "Not run" line, a reason
  for each adjacent pair, and a separability statement.

This revision:
- writes the owner's eight answers and the AppArmor answer into the text, each citing
  `doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`;
- rewrites the units whole as the chosen ones, every placeholder named, with the nested dataset's own
  unit and the two consumers in their chosen shapes;
- moves every option the owner did not choose into "The options not chosen", with its evidence;
- re-runs the provider-restart arm with the chosen units and reports that the nested unit does not
  restart at all under `Wants=`;
- gives that unit a measured already-mounted tolerance, `ConditionPathIsMountPoint=!<path>`, and shows
  it skipping at a gate restart and at a boot where the provider mounted the dataset first;
- measures what the record itself called unmeasured: a provider failure with a healthy checkout leaves
  the gate green;
- measures the login cost of V8a's answer: none, because sessions start 0.44 s after the gate and 20.4 s
  before it fails;
- boots the chosen set green, fail-closed on a missing key, and with the bounds scaled down inside the
  stages arm only.

No cross-vendor review of this revision has been received, so none is folded in, and the document
stays unapproved and non-final.
