# Which unit mounts the checkout, and the ordering the boot gate needs

Design proposal by an agent,
revised a fifth time,
after a same-vendor read of the fourth revision,
**unapproved and non-final**.
It answers the two parts plan step 19 leaves open
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, "Step 19", "Open items"),
folds the findings of that read,
and measures on one throwaway guest the three things the read showed unsettled:
how many stop bounds a hung unit spends when its programs ignore SIGTERM,
where the stop-failure mode has to live for Fedora's own drop-in not to override it,
and what a per-dataset unit running `zfs mount` directly does at boot and after a provider restart.
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
and "round four" means `~/temp/agent/zfs-step-19-fourth-revision-2026-11-24/`.
A log named without a round is this round's.
Where a fact rests only on an earlier round's log, the round is named with it.

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
Folded in: option C now drops the ordering line from all four units and lets each consumer require
what it needs, which is the treatment option E had; the per-dataset and consumer units' bounds are
stated in "The units for the datasets under the checkout path"; and V8 is split into two questions,
whether the before-login ordering stays and where the bound values come from.

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

**M8, V2's option C contradicts this document's own drop-in section.** Held: the provider drop-in is
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

One throwaway guest on Fedora 43 Cloud Edition, `zfs-step-19-fifth-2026-11-24`, created and
destroyed through `mvm --backend libvirt` by the name its own create printed: systemd
`258.11-1.fc43`, kernel `6.17.1-300.fc43.x86_64`, OpenZFS `2.4.4-1` built from the release tarball
pinned at sha256 `2a3c70d55a37cc71618a95a60e81ad66530201eb118d37741dc92efcf848c8b1`, SELinux enforcing.
It booted four times: the first for the build and the fixture, then three unattended reboots, one per
arm, and one runtime arm ran inside the second of those boots without a reboot.
One guest was enough where round three needed three, because the facts that move with a systemd
version were measured there on 262.2 and this round adds no new one.

- `prepare.log`, `prepare-readback.log`, `fixture.log`, `units-install.log`: the guest's versions, its
  file-backed pool and datasets, and every unit and drop-in as systemd reads them.
- `state-green.log`, `evidence-green.log`: the unit text as this revision writes it, with the
  stop-failure mode in its own drop-in, and the green boot of it.
- `state-v9c.log`, `evidence-v9c.log`: the same boot with the per-dataset unit running
  `zfs mount <dataset>` directly, which is choice V9's option C.
- `restart-provider-v9c.log`: the provider restart that re-ran that unit on an already-mounted
  dataset.
- `state-stages.log`, `evidence-stages.log`: a start pass and a stop pass that ignore SIGTERM, with
  `TimeoutStartSec=8` and `TimeoutStopSec=4`, and the three stop stages each spending the stop bound.
- `stopmode-shapes.log`: the four placements of `TimeoutStopFailureMode=terminate` and the value
  systemd reports for each.
- `mvm-create-guest.log`, `mvm-destroy-guest.log`, `mvm-list-after-destroy.log`: the guest's own
  create and destroy records, through `mvm --backend libvirt`, by the name the create printed.
- `host-reads.log`, `apparmor-and-unit-reads.log`: every host fact this revision states, read again
  this round.

Three limits, so no result is read wider than it is:

- The programs are stand-ins. The gate, the probe, the stop passes and the consumers perform the
  reads this design names, because the wrapper's root modes are not built.
  Every result is about the units, their ordering edges, the failures they carry and systemd's and
  ZFS's own behaviour, never about the wrapper's handler.
- The checkout is its own encryption root on the guest, so one arm can make its mount fail by moving
  one key file. This host's checkout inherits the pool's key, which the host-shape arm models.
- No arm made the ZFS mount generator write a mount unit, and no arm ran ZED or AppArmor.
  Those two branches are source and policy readings.

## The answer in brief

- The provider is `zfs-mount.service` on a machine of this host's shape: this host's generator writes
  no unit for any dataset here, and `zfs mount -a` is what mounts them
  (host reads, `keep/host-reads.log`).
- The provider's exit status says nothing about the checkout **where the datasets under the checkout
  path carry `canmount=noauto`**: with the checkout's key unavailable and no key-loading unit, the
  provider finishes `Result=success`, `ExecMainStatus=0` while the checkout is not mounted at all
  (round three's `evidence-no-key-unit.log` and round four's `evidence-host-shape.log`).
  Without that property the provider says a great deal, and the next two bullets are why.
- With a dataset under the checkout path at `canmount=on` and the covered entry's flag **on**, the
  provider fails the whole boot's mount: it cannot create the mountpoint inside the immutable entry,
  `zfs-mount.service` ends `Result=exit-code`, `ExecMainStatus=1`, and the gate's job fails with
  `dependency` before its own check runs
  (round four's `evidence-hazard-flag-on.log`, boot `fcfae077-91df-49bf-bced-ff085a27b7ca`).
- With the same dataset at `canmount=on` and the flag **off**, the provider creates the mountpoint
  folder inside the exposed home underlay and mounts the dataset there
  (round three's `evidence-hazard-nested-auto.log`).
  The two together are why V4's property is a requirement and not a preference.
- What fails closed on a missing checkout key is the key-loading unit,
  `zfs-load-key@<escaped encryption root>.service`, required by the boot unit where one exists.
  Measured: the key unit exits `255/EXCEPTION`, and the gate, both account consumers and the
  per-dataset unit all fail with `dependency`
  (round three's `evidence-key-removed.log`, boot `1010ab84-5d06-4ede-8f5d-fd9880c6b700`).
- Where no key unit exists, this host's shape, the gate's own refusal is the signal:
  `SWEEP refused=checkout-not-the-dataset`, exit status two, and the same fail-closed job results
  (round four's `evidence-host-shape.log`, round three's `evidence-no-key-unit.log`).
- Never require a mount unit systemd learned from the mount table.
  Measured at a boot on systemd 258 and on 262.2, and again with the provider also required and both
  drop-ins absent: the requiring unit's start job leaves no journal line at all and the unit stays
  `inactive` with `Result=success`
  (round three's `evidence-learned-unit-only.log`, `boot-arm-learned-unit.log`,
  `boot-arm-learned-with-provider.log`).
- Order with `RequiresMountsFor=<checkout path>` rather than by hand. On the guest, one line put
  `After=` on the root mount and on the home dataset's mount unit, and `Requires=` only on the root
  mount, the one of them with a fragment (kept log `state-green.log`; round three's
  `systemd-262-arms.log` arm G).
  **On this host it would put only the `After=` on `-.mount`**, whose `FragmentPath` is empty, and the
  `Requires=` half lands on the provider's own line and not here (host read,
  `keep/host-reads.log`; the fragment rule, V3).
- The gate's ordering before `systemd-user-sessions.service` is real, and its cost is a start bound
  that is per phase, a reload that shares it, and a stop phase whose every stage the stop bound bounds.
  Measured in one boot, with a pass and a stop pass that both ignore SIGTERM: 8 s of start bound and
  three 4 s stop stages held logins 20.53 s (kept log `evidence-stages.log`, boot
  `fec7550a-5a0f-450c-802b-f9e004560004`).
  Measured in another boot, with a pass that dies at SIGTERM: 20 s of start bound and one 3 s stop
  stage held logins 23.54 s (round four's `evidence-bound-stop.log`, boot
  `ad2d3287-25f8-4086-819a-3ac4320671ea`).
- The boot unit text as this revision writes it, both bounds included and the stop-failure mode in a
  drop-in named for the unit, boots green with Fedora's own type-level drop-in installed and unmasked
  (kept log `evidence-green.log`, boot `de767eea-e7a9-4c4d-ad6a-e4a97803507b`), and the same text
  reports `TimeoutStopFailureMode=terminate` where Fedora's file says `abort`.
- A per-dataset unit that runs an explicit `zfs mount <dataset>` after the gate mounts the dataset and
  its consumer sees it, so that option ships as it stands (kept log `evidence-v9c.log`); it also needs
  the already-mounted rule, because a provider restart re-runs it on a mounted dataset and it ends
  `exit-code` with `filesystem already mounted` (kept log `restart-provider-v9c.log`).
- A consumer that requires the per-dataset unit sees that dataset mounted; one that names a unit the
  boot does not install gets nothing at all, silently
  (kept log `evidence-green.log`, round three's `evidence-v9b.log`).
- The per-dataset unit and both account consumers are unbounded oneshots: systemd reports
  `TimeoutStartUSec=infinity` and `TimeoutStopUSec=45s` for all three, and each carries its own
  `Before=systemd-user-sessions.service` (kept log `state-green.log`).
- The ZED hook is the root-owned wrapper file under a ZED-legal name, started with no argument, and
  on this host it waits on an AppArmor grant that does not exist yet, on a `zfs-zed.service` that is
  disabled and inactive, and on the `zed` profile's own exec rule naming a zedlet folder this host
  does not have (profile reading, `keep/apparmor-and-unit-reads.log`).

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
the checkout path, the propagation service, the provider's drop-in, the propagation drop-in, and the
ZED hook.
Two values wait on a measurement this design does not have, `<the measured ceiling of one reconcile>`
and `<the measured ceiling of one stop pass>`, and choices V8a and V8b settle what ships meanwhile.

Two placeholders stand in the unit text, and each is owned by another design:

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

### The boot and shutdown unit

```ini
# /etc/systemd/system/reconcile-git-worktree-clone-mounts.service
[Unit]
Description=Reconcile the managed worktree clone mounts
Requires=zfs-load-key@<escaped encryption root>.service
After=zfs-load-key@<escaped encryption root>.service
Requires=zfs-mount.service
After=zfs-mount.service zfs-import.target
RequiresMountsFor=<checkout path>
Before=systemd-user-sessions.service
Before=umount.target
Conflicts=umount.target

[Service]
Type=oneshot
RemainAfterExit=yes
TimeoutStartSec=<the measured ceiling of one reconcile>
TimeoutStopSec=<the measured ceiling of one stop pass>
ExecStartPre=-<root-file> <root-words> start-probe
ExecStart=<root-file> <root-words> boot-reconcile
ExecReload=<root-file> <root-words> boot-reconcile
ExecStopPost=-<root-file> <root-words> shutdown-unmount

[Install]
WantedBy=multi-user.target
```

**The two key lines are conditional.** A machine with no key-loading unit must not carry them, because
a unit that requires a unit the boot does not install stays `inactive` with `Result=success` and no
journal line at all (F31), which is a gate that silently never runs.
This host's unit, the shape such a machine installs, is the same text with those two lines removed,
and it ran on round four's host-shape boot, where the gate refused the checkout by its own rule
(round four's `state-host-shape.log`, `evidence-host-shape.log`):

```ini
# /etc/systemd/system/reconcile-git-worktree-clone-mounts.service, on a machine with no key-loading unit
[Unit]
Description=Reconcile the managed worktree clone mounts
Requires=zfs-mount.service
After=zfs-mount.service zfs-import.target
RequiresMountsFor=<checkout path>
Before=systemd-user-sessions.service
Before=umount.target
Conflicts=umount.target

[Service]
Type=oneshot
RemainAfterExit=yes
TimeoutStartSec=<the measured ceiling of one reconcile>
TimeoutStopSec=<the measured ceiling of one stop pass>
ExecStartPre=-<root-file> <root-words> start-probe
ExecStart=<root-file> <root-words> boot-reconcile
ExecReload=<root-file> <root-words> boot-reconcile
ExecStopPost=-<root-file> <root-words> shutdown-unmount

[Install]
WantedBy=multi-user.target
```

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
- `Requires=zfs-mount.service` and `After=zfs-mount.service zfs-import.target`.
  What the requirement buys is a pull-in, a dependency failure and restart propagation.
  The pull-in is weaker than it reads: the provider's
  `ConditionPathIsDirectory=/sys/module/zfs` turns a start without the module into a condition skip
  that succeeds and mounts nothing (source reading of the unit text, `keep/host-reads.log`), and
  where a dataset under the checkout path cannot be mounted the provider fails the whole boot's
  mount, which this unit's requirement then carries as a `dependency` failure instead of the unit's
  own refusal (round four's `evidence-hazard-flag-on.log`, quoted under "The failure controls").
  Restart propagation is load-bearing at runtime: measured on systemd 258 and on 262.2, a provider
  restart takes this unit, every per-dataset unit and every consumer that requires them through
  their stop pass and starts them again, four units in the 258 log
  (round three's `restart-provider.log`, `systemd-262-arms.log` arm F).
  At a boot the ordering edge is not what starts this unit: the brown-out arm of round two ran with
  no edge to the provider at all and this unit still started after it, because the propagation
  service's request started it (round two's `evidence-no-provider-edge.log:93` to `:96`:
  `Starting checkout-propagation.service` at `1795503075.469533`, `Starting reconcile-…` at
  `1795503075.501653`).
  Replace the name with the generated mount unit where the generator writes one; keep the
  `zfs-import.target` edge either way.
- `RequiresMountsFor=<checkout path>`.
  This replaces the two hand-written ordering lines.
  Measured on both systemd versions: one line put `After=` on `-.mount` and on the home dataset's
  learned mount unit, and `Requires=` only on the one of them that has a fragment
  (round four's `state-as-written-stopsec.log`, "the dependencies systemd derived":

  ```text
  Requires=zfs-mount.service sysinit.target system.slice "zfs-load-key@zfsstep19-home-monochromatic\\x2dgit\\x2dcheckout.service" -.mount
  After=zfs-mount.service zfs-import.target system.slice basic.target systemd-journald.socket -.mount "srv-zfs\\x2dstep\\x2d19\\x2dfixture-home.mount" "zfs-load-key@zfsstep19-home-monochromatic\\x2dgit\\x2dcheckout.service" sysinit.target
  ```

  and round three's `systemd-262-arms.log` arm G for the same line on 262.2).
  At the moment the unit loads, the checkout's own learned unit may not exist yet, so that edge
  orders the stop half rather than the first start; what orders the first start is the provider's
  requirement and `zfs-import.target`.
  What lands differs by machine and this host is the other shape: its root mount unit has an empty
  `FragmentPath`, so `Requires=` does not attach to it there and the line would leave an `After=` only
  (host read, `keep/host-reads.log`; the same rule V3 states).
- `Before=systemd-user-sessions.service`.
  What makes the checkout ready before an account session, and the line whose cost "What a hung start
  costs" measures. Measured in this round's green boot: the gate finished at `1795529986.223586` and
  the sessions unit started at `1795529986.582386`, after the account consumer at
  `1795529986.241818` and the per-dataset unit at `1795529986.262900`.
- `Before=umount.target` with `Conflicts=umount.target`.
  The stop half. Measured at runtime on 258 through the provider restart, which took the units down in
  reverse order and ran the stop pass (round three's `restart-provider.log`:
  `Stopping git-account-session-standin.service` at `1795507731.282283`,
  `Stopping reconcile-…` at `1795507731.284386`,
  `SWEEP stopped … stopped-slot=unmounted`).
  A shutdown arm was not run this round; round one's shutdown order on systemd 255 is in its
  `keep/guest-logs/journals-per-boot.log`, lines 83 to 90.
- `TimeoutStartSec=<the measured ceiling of one reconcile>` and
  `TimeoutStopSec=<the measured ceiling of one stop pass>`.
  Their values are not yet known: no run has measured one reconcile or one stop pass over this
  host's storage.
  What the pair buys is measured at test values: `TimeoutStartSec=8` on a hung start produced
  `start operation timed out. Terminating.`, then the stop phase, then `Failed with result 'timeout'`,
  then sessions; and `TimeoutStopSec=4` bounded each of the stop phase's three stages that a pass
  which ignores SIGTERM survives (kept log `evidence-stages.log`, quoted under
  "What a hung start costs").
  The bound is per phase, so the hold is the start bound plus the stop bound of each stage that has a
  process left to kill, which is what V8b prices.
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

## The units for the datasets under the checkout path

One unit per dataset whose mountpoint lies inside the checkout path, generated by setup and reported
by `doctor`:

```ini
# /etc/systemd/system/git-worktree-nested-<slot-or-name>-mount.service
[Unit]
Description=Mount a dataset whose mountpoint lies inside the verified checkout
Requires=reconcile-git-worktree-clone-mounts.service
After=reconcile-git-worktree-clone-mounts.service
Before=systemd-user-sessions.service

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=<root-file> <root-words> mount-path-descendant --dataset <dataset>

[Install]
WantedBy=multi-user.target
```

`mount-path-descendant` does not exist yet, and **it contradicts a principle the grammar already
states**. So the ask is a question for the root-modes design, not only an addition to a list:

> a mode is chosen by a leading word and nothing else, and no argument names a folder to mount or a
> dataset to change

(source reading, `handler_mode_arguments.rs:112` to `:115`, the `HandlerMode` doc).
That sentence's reach is narrower than it reads: `holder-scan` takes `--dataset` today, so an
argument may name a dataset. What the sentence forbids is an argument that makes the wrapper
**change** a dataset's state. `mount-path-descendant --dataset <dataset>` would do exactly that, so
either the mode list admits one mode that changes state by name, or this design's option A has no
call to make. The root-modes design owns that answer; this design records the conflict rather than
assuming its consent.

If that design admits the mode, this is what it needs, in its own words:

- the word `mount-path-descendant` and one option, `--dataset <dataset>`, read the way `holder-scan`
  reads its own `--dataset`;
- a refusal of its own for a dataset whose mountpoint lies outside the checkout path it verifies
  first: **`dataset-outside-the-checkout`**, a name nothing else uses.
  It is a new name because `checkout-not-the-dataset` already means something else in this design:
  the checkout's own mount row is not the checkout dataset.
  That refusal must name the dataset it refused, not only the path.
- **idempotence on an already-mounted dataset**: exit 0 and print what it found.
  This is measured as needed, not optional:
  a provider restart re-runs every descendant unit (round three's `restart-provider.log`), and the
  program a naive implementation would call returns status one for a second explicit mount of a
  mounted dataset. Measured, on the same dataset:
  round two's `propagation.log:99` holds
  `cannot mount 'zfsstep19/home/monochromatic-git-checkout': filesystem already mounted`;
  and the source gates that message on `explicit`
  (source reading, `zfs-2.4.4` `cmd/zfs/zfs_main.c`, the `OP_MOUNT` branch:
  `if (!explicit) return (0);` then `filesystem already mounted` and `return (1)`).

**Both units this design adds are unbounded oneshots by default**, and the reading matters for the
login hold: `systemctl show` reports `TimeoutStartUSec=infinity` and `TimeoutStopUSec=45s` for the
per-dataset unit and for both account consumers, each of which carries its own
`Before=systemd-user-sessions.service` (kept log `state-green.log`, "the stop-failure mode and the
bounds systemd reports, per unit").
So a hung `zfs mount` here, or a hung consumer command, holds every login with no start bound at all,
whatever the gate's own bounds say; that is why V8a's option B needed the treatment it now has, and it
is why the smallest complete answer bounds these units too or drops their ordering line.

For this host that is one unit, for `zpcachyos/nosnap/vm-builder-output`.
The measured stand-in ran the same call the mode would make, an explicit `zfs mount <dataset>`, after
checking that the checkout's own mount row is the checkout dataset, and it is idempotent
(round four's `evidence-as-written-stopsec.log`):

```text
SWEEP checkout-row=zfsstep19/home/monochromatic-git-checkout zfs
NESTED start 2026-11-24T12:07:21.221Z
NESTED mounted=zfsstep19/nosnap/vm-builder-output 2026-11-24T12:07:21.245Z
SESSION-DESCENDANT mounted=zfsstep19/nosnap/vm-builder-output
```

That stand-in **never exercises the refusal**: it ignores its arguments and reads the checkout's row
by its own fixed path, so nothing measured shows a dataset outside the checkout being refused
(the body is round four's `keep/guest-scripts/10-units.sh`, the `nested` function).
The refusal is a requirement with a source reading behind it and no arm.

The explicit form matters: `zfs mount -a` skips a `noauto` dataset and skips an unavailable key
silently, while the explicit form refuses with a named message (source reading,
`cmd/zfs/zfs_main.c`, `share_mount_one`'s `canmount` table and key branch, both gated on `explicit`;
measured for the key case in "The route-owned alternative").
`Before=systemd-user-sessions.service` is what closes the window the earlier rounds measured: without
it round one's `noauto` boot finished the descendant unit after the sessions unit
(round one's `keep/guest-logs/journals-per-boot.log`, `Finished systemd-user-sessions.service` at
`1795486959.129715`, `Finished git-worktree-nested-output-mount.service` at `1795486959.180427`);
with it, this round's green boot is gate, then the per-dataset unit and its consumer, then sessions
(kept log `evidence-green.log`, quoted in "The boot and shutdown unit").

### What choice V9's second option needs, which does not exist

The other way to mount these datasets is the boot unit's own pass, with no per-dataset unit, and
**that does not exist yet either**. `boot-reconcile` has no step that mounts a dataset under the
checkout: its order is its own checks, then the sweep, then the readiness record
(source reading, `boot_reconcile_mode.rs`, the module doc and `run_at`), and `rg 'zfs mount'` over
that folder matches comments only.
Round three measured a stand-in whose `zfs mount` exit status was echoed and whose last command was
an `echo`, so that unit exited zero whatever the mount did
(round three's `keep/guest-scripts/10-units.sh`, the `sweep-descendants` body).

That step needs three things this design now names, and they are the work option B costs:

- a source for the dataset list: the same list setup generates the per-dataset units from, read at
  boot rather than assumed;
- a failure rule: what a failed descendant mount does to the readiness record and to the unit's exit
  status, given that individual clone failures today still publish `Failed` and return `Ok`
  (`boot_reconcile_mode.rs:158` to `:164`);
- the same idempotence on `ExecReload=`, which re-runs the pass on an active checkout.

**So option A and option B both need code that does not exist**, and the difference between the three
options is what each delivers: B needs that step, A needs that mode, and C needs neither, which is why
this round booted C.

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
- an AppArmor grant, which no design owns yet and which the next section reads;
- the new mode `mount-path-descendant`, which choice V9's option A needs.

Three things on this host have to be true before the hook runs at all, and none is a design choice:

- `zfs-zed.service` is `disabled` and `inactive` here (F3), so no zedlet runs today;
- the `zed` profile's zedlet exec rule names a folder this host does not have, so even the packaged
  zedlets have no exec grant under it if it is enforced;
- the installed file would need an exec grant, a profile transition, or an unconfined start, and the
  AppArmor section is where each of those is priced.

## AppArmor on this host: what the hook and the pass it starts need

This is a **reading of profile text and of the host's readable policy**, not a measurement of
enforcement: the loaded-profile list, `/sys/kernel/security/apparmor/profiles`, is readable by root
only, so an unprivileged read answers `Permission denied`.
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

### What the hook and its pass would need

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

### The shapes, ranked by what each delivers

The profile offers exactly two inheritance classes: `ix`/`rix` stays inside `zed`, and `px`/`rpx` moves
to a named profile. Everything below is a way of arranging a grant inside those two, and only two of
the shapes have been weighed to the point of a ranking.

**Shape A, rules added to the profile through `/etc/apparmor.d/local/zed`.**
Every grant the hook needs lands in the `zed` profile: an exec rule for the installed file, `mount`
and `umount`, `capability dac_read_search` and possibly the `ptrace` read grant, and writable paths for
the slot, the record and the pass's own state.

What it delivers: one profile to load, one file to read, no transition to get wrong, and a start that
cannot fail for a missing target profile; and it is the only shape whose grants the owner's own
`local/` files already demonstrate, since `local/git` line 14 grants `rix` on the wrapper's exact path.

What it costs: every one of those grants also accrues to ZED itself and to every packaged zedlet that
runs `ix` into it, including `mount` over any path and a capability that reaches every `0700`
directory on the machine. The profile's own audit identity then names `zed` for a mount that ZED's
notification code never performs. And it is not a rule the file name limits: the exec rule it needs is
an `x` class rule on the installed file, which is the same kind of line shape B's `px` is.

**Shape B, a dedicated profile for the installed file, reached with one `px` rule in `local/zed`.**
The hook gets its own ruleset, and ZED's profile keeps the narrower set it has.

What it delivers: a separate audit label, so a denial names the hook and not ZED; its own mode, so an
experiment can put the hook in complain while ZED stays enforcing; a permission set that can be
reviewed without reading ZED's; and, for the one path a profile does confine here, a ruleset written
for a program that mounts as root rather than inherited by a notification daemon.

What it costs, in two kinds:
- **capability:** a `px` fails the exec closed if the target profile is not loaded, so a half-done
  install is a hook that cannot run while the boot gate still can, and the wrapper file then has a
  profile to keep in step with its real behaviour;
- **maintenance:** a second profile file and a name an installer has to know.
  That second half is the cost the owner's recorded answer is about, and it is real.

It is also **rules plus a profile**, not a profile instead of rules: the `px` line itself lives in
`local/zed`, and that file has to exist for the transition to happen at all.

**The four shapes neither this design nor the read has weighed.** They exist, they are reachable
inside the two classes the profile offers, and each is stated here with what is known about it so the
owner can ask for one:

- **a child profile declared inside `local/zed` and entered with `cx`.** The ruleset sits in the same
  file as the `zed` profile, as a nested profile, and the exec rule uses `cx`. It delivers B's separate
  ruleset and audit label without a second file to install, and it costs a nesting that
  `apparmor.d(5)` allows but that this host's other local files never use, so nothing here shows it
  loading.
- **a `pix` fallback.** `pix` transitions to the profile if it is loaded and stays in the current one
  if it is not, so it removes B's fail-closed cost. It also removes the guarantee B is chosen for: a
  machine where the profile did not load runs the hook with `zed`'s permissions and fails open into a
  broader set.
- **an unconfined start (`Ux`)** for the installed file. This is what the boot unit and the
  propagation service already do today, because no profile names their path. It delivers the least
  work and the least surprise, and it costs every guarantee the section is about: the hook's mounts and
  its `0700` reads would run with no audit label and no rule to review.
- **`px` to a profile that already exists**, which is what `/etc/apparmor.d/local/gh` line 2 does for
  the wrapper: `owner <the wrapper build> px -> git`. There the target is a profile the owner already
  maintains, so no new file is written and the wrapper is confined by rules that exist for its own
  program. For this hook the target would have to be a profile that already grants `mount fstype=zfs`
  and the capabilities the pass needs, and this host's `zfs` and `mount-zfs` profiles are written for
  the `zfs` and `mount.zfs` programs, not for the wrapper's own modes; whether they would do is not
  measured here.

**Ranking: B > A**, because the difference at the same grant is the audit identity and the review
surface: B's denials name the hook, and B's ruleset can be read without reading ZED's, while A widens a
notification daemon's permissions to cover a program that mounts. B's capability cost is a fail-closed
exec and one more file to keep in step; its maintenance cost is the one the owner's answer weighs.
A is not the lowest-privilege shape, and this design no longer calls the choice "least privilege for
the only program that mounts": **the same wrapper starts unconfined from the boot unit's `ExecStart=`,
`ExecReload=` and `ExecStopPost=` and from the propagation service**, and only the ZED path inherits a
profile, as "Only the ZED path is confined" says. What B buys is confinement for the hook's own path,
not for the wrapper.

**The owner's recorded answer, read for what it decided.** On 2026-11-23 the owner answered a
rule-or-profile question with "No, please do rules for everything instead of a dedicated profile that
agents would have to remember to use", and `/etc/apparmor/full-system-policy.md` records it under
"Local workload rules".
The session that asked was weighing a profile the wrapper's own tests would have to enter on purpose,
and the reason given was that agents would have to remember to use it.
That reason does not carry to this hook, and saying it did would misread the answer: nothing starts the
hook but ZED, and the kernel makes the transition on exec, so nobody has to remember anything.
What does carry is the direction: the owner prefers rules to profiles, and the two `local/` files this
host keeps for the wrapper's exact path are both rules, one `rix` and one `px` to an existing profile.
So the answer is a standing preference about maintaining this machine, this design puts it beside the
ranking rather than treating it as a measurement of what each shape delivers, and it does not decide
for the owner.

**The question waits for one host fact, and the fact is that nothing needs the grant today.**
`zfs-zed.service` is `disabled` and `inactive` here (F3), so no zedlet runs and no AppArmor grant is
needed until the owner enables ZED.
When that happens the question to put is: the owner's answer of 2026-11-23 was rules rather than a profile,
given while a session weighed a profile agents would have to remember to use; ZED starts this hook, so
nobody has to remember anything; if ZED is enabled, where should the hook's permissions live?
The options are A, B, the two shapes that change B's failure behaviour (`cx` inside `local/zed`, and
`pix`), the unconfined start, and `px` to an existing profile, each with the cost stated in the list
above it.

This design does not choose. It lists the grant under what the hook waits on, and `doctor` reports
what an unprivileged reader can see: whether `/etc/apparmor.d/zed` carries a `flags=` line, whether
`/etc/apparmor.d/local/zed` exists, whether `apparmor.service` is enabled and active, what
`/etc/apparmor/modes` says, whether `zfs-zed.service` is enabled, and whether the profile's zedlet rule
names a folder that exists. It cannot report the loaded mode, and it says so rather than guessing.

Two facts worth keeping beside the reading:

- The host's own `zfs` and `mount-zfs` profiles do allow `mount fstype=zfs` and `umount fstype=zfs`
  for the programs they name (`keep/apparmor-and-unit-reads.log`), which is how a person's
  `zfs mount` works. That does not help the hook: the hook's own process is not the `zfs` program, and
  only a `px` transition would put it there, which is the fourth unweighed shape in that list.
- The hazard arm's mountpoint creation by `zfs mount -a` is a profile question here as well, so the
  hazard's shape on this host is not settled by the guest, where no AppArmor runs.
  The `zfs` profile's `mount fstype=zfs` grants the mount, and creating the mountpoint folder is an
  ordinary directory write the profile must allow; the guest's arm was refused by the immutable flag
  before any policy question arose.

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

Every failure control of the four rounds, each with its own kept log.
Three arms ran this round, on one boot each, and one runtime arm ran inside the second of them;
the earlier rounds' arms are kept and named with their round.

**This round, on the guest of "What it ran": three arms and one runtime arm, all with SELinux
enforcing.**

- **The unit text as this revision writes it, with the stop-failure drop-in beside it: passes.**
  Kept logs `state-green.log`, `evidence-green.log`, boot
  `de767eea-e7a9-4c4d-ad6a-e4a97803507b`.
  The probe ran, the gate read the checkout row, the flag and the private propagation, attached the
  slot; the account consumer saw the checkout and the slot; the per-dataset unit mounted its dataset
  and its own consumer saw it; and sessions started last:
  gate `1795529986.185462` to `1795529986.223586`, account consumer `1795529986.226087` to
  `1795529986.241818`, per-dataset unit `1795529986.230947` to `1795529986.262900`, its consumer
  `1795529986.265039` to `1795529986.276250`, sessions `1795529986.582386`.
  Fedora's type-level `10-timeout-abort.conf` was installed and unmasked in that boot, and the gate
  reported `TimeoutStopFailureMode=terminate` with both drop-in paths listed in that order
  (`state-green.log`).
- **Choice V9's option C, one unit per dataset running `zfs mount <dataset>` directly, with the same
  `Requires=` and `After=` on the gate: passes.** Kept logs `state-v9c.log`, `evidence-v9c.log`, boot
  `14c29514-53e9-4fa5-9daa-fc0d742f810b`.
  The unit started at `1795530175.291209`, finished at `1795530175.309134`, the dataset was mounted,
  and its consumer ran at `1795530175.311884` and reported
  `SESSION-DESCENDANT mounted=zfsstep19/nosnap/vm-builder-output`.
  So this option needs no new mode to mount the dataset at a boot.
- **The same option under a provider restart: the unit fails, and its consumer's job with it.**
  Kept log `restart-provider-v9c.log`, in the same boot.
  `systemctl restart zfs-mount.service` took the gate, both consumers and the per-dataset unit through
  their stop half and started them again; the per-dataset unit then ran its explicit mount on the
  dataset that was still mounted:

  ```text
  1795530237.561479 zfs-step-19-fifth-2026-11-24 zfs[1335]: cannot mount 'zfsstep19/nosnap/vm-builder-output': filesystem already mounted
  1795530237.561874 zfs-step-19-fifth-2026-11-24 systemd[1]: git-worktree-nested-output-mount.service: Main process exited, code=exited, status=1/FAILURE
  1795530237.561960 zfs-step-19-fifth-2026-11-24 systemd[1]: git-worktree-nested-output-mount.service: Failed with result 'exit-code'.
  1795530237.562126 zfs-step-19-fifth-2026-11-24 systemd[1]: Failed to start git-worktree-nested-output-mount.service - Mount a dataset whose mountpoint lies inside the verified checkout.
  1795530237.562187 zfs-step-19-fifth-2026-11-24 systemd[1]: Dependency failed for git-vm-builder-standin.service - A vm-builder service of the account that needs the dataset under the checkout.
  1795530237.562198 zfs-step-19-fifth-2026-11-24 systemd[1]: git-vm-builder-standin.service: Job git-vm-builder-standin.service/start failed with result 'dependency'.
  ```

  The dataset stayed mounted, so this is a red unit for a correct state: the same already-mounted rule
  the mode needs applies to the direct option.
- **A pass and a stop pass that both ignore SIGTERM, with a short start bound and a short stop bound:
  three stop stages run, and the third is what ends it.** Kept logs `state-stages.log`,
  `evidence-stages.log`, boot `fec7550a-5a0f-450c-802b-f9e004560004`, quoted whole under
  "What a hung start costs".
  No `Aborting` line, no SIGABRT kill and no coredump appears in that boot, because the drop-in put
  the unit in `terminate`.

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
`keep/host-user-manager-stopsec.log`, and three boots, round three's, round four's and this round's.

The default: a oneshot has no start timeout.
`systemctl show` for a unit with no `TimeoutStartSec=` reports `TimeoutStartUSec=infinity`
(round three's `systemd-262-arms.log` arm A, and round two's `start-bound.log`; and this round's
per-dataset unit and both consumers, in `state-green.log`), so a hung reconcile ordered before the
sessions unit holds every login with no limit, and so does a hung per-dataset unit or consumer,
because they carry the same ordering line.

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

which is 15.35 s for a 20 s start bound and a 5 s stop bound in the worst shape, 20.53 s measured for
this round's 8 s and 4 s, and about 270 s for a 20 s start bound with the default 90 s stop bound.
The cooperative case is the same formula with one stage instead of three, which is 23.54 s for a 20 s
start bound and a 3 s stop bound.
A pre-phase that is unbounded rather than the probe adds one more start bound, because the start bound
is per phase.
That arithmetic is why the unit text carries `TimeoutStopSec=` as well as `TimeoutStartSec=`, why the
stop-failure drop-in matters, and why choice V8b prices the hold each answer leaves.

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

## The route-owned alternative, measured

Choice V1's second option is a unit the route owns, which mounts the checkout explicitly, with the
checkout dataset at `canmount=noauto` so the provider's scan skips it. Round two ran it three ways,
and its whole text is in that round's `guest-scripts/03-units.sh`.
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
- `state-green.log`, `evidence-green.log`: the unit text as this revision writes it, with the
  stop-failure drop-in, and the green boot of it, with the order gate, account consumer, per-dataset
  unit, its consumer, sessions; the bounds and modes of all four units; and the audit attribution.
- `state-v9c.log`, `evidence-v9c.log`: the same boot with the per-dataset unit running an explicit
  `zfs mount`.
- `restart-provider-v9c.log`: the provider restart that re-ran that unit and failed it on
  `filesystem already mounted`.
- `state-stages.log`, `evidence-stages.log`: the start bound and the three stop stages in one boot,
  with a pass and a stop pass that ignore SIGTERM.
- `stopmode-shapes.log`: the four placements of `TimeoutStopFailureMode=terminate` and the value
  systemd reports for each.
- `restart-provider.log` (round three): the provider restart that took four units through their stop
  pass, with `stopped-slot=unmounted`.
- `evidence-learned-unit-only.log`, `boot-arm-learned-unit.log`,
  `boot-arm-learned-with-provider.log` (round three): the learned-unit trap in three shapes.
- `systemd-262-arms.log` (round three): the bound facts, `RequiresMountsFor=`'s expansion, `Upholds=`
  and `BindsTo=` on a learned unit, and the four-unit restart chain, all on 262.2.
- `evidence-v9b.log` (round three): choice V9's option B, a stand-in whose exit status is not the
  mount's, and the consumer that names a per-dataset unit getting nothing.
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
- `mvm-create-guest.log`, `mvm-destroy-guest.log`, `mvm-list-after-destroy.log`: this round's guest's
  create and destroy records.
- `host-reads.log`, `apparmor-and-unit-reads.log`: every host fact this revision states, read this
  round.

Some logs named in this revision are not this round's, and each is named with its round:
`start-bound.log`, `dependencies-read.log`, `propagation-cycle.log` and `propagation.log` are round
two's; `restart-provider.log`, `systemd-262-arms.log`, `upholds-arm.log`, `escaping-arm.log` and every
`evidence-*` log of that round are round three's; and the nine logs this revision names with round
four are round four's.

## What the runs do not show

- Not the product handler. Every program is a stand-in; the wrapper's root modes, the readiness
  record, the start probe's own deadline and the unmount half never ran.
- Not this host's kernel or pool for the ZFS arms. The guest runs kernel `6.17.1-300.fc43`, OpenZFS
  `2.4.4-1` and a file-backed pool, and no host dataset was mounted, unmounted, imported, exported or
  changed.
- Not a generated mount unit. No arm made the ZFS mount generator write a unit, because that needs a
  root-written cache file on a machine whose ZED tracks the pool; the branch of the rule that names
  `<escaped checkout path>.mount` as the provider is a source reading plus the host reads, not a run.
- Not the generator's key units as generated: this round's guest had no key unit for the checkout at
  all, and its two key units were written by hand in the generator's names.
- Not ZED. No arm ran `zfs-zed.service`, and the hook is a text this design proposes.
- Not AppArmor. The guest runs Fedora's SELinux, not this host's AppArmor, so the hook's own profile
  question is a reading here, not a run, and none of the six shapes has been loaded.
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
- Not choice V9's option A or option B: the mode and the pass's step do not exist.
  Option C did run, at a boot and under a provider restart.
- Not V8a's option B: no arm has run with the ordering line removed from all four units.

## What else on this host touches the checkout's boot

`zfs-space-guard.service` is `After=zfs-mount.service` and its timer runs every five minutes,
deleting old `/home` snapshots when the pool is over 85 percent full, on a five-minute timer
(`systemctl cat` of both units in this round's `keep/host-reads.log`).
It has no edge to the boot unit and this design gives it none; what matters for a reader is that a
service ordered after the provider can run while the checkout is absent, which is another reason a
provider's own success is not a readiness signal.

## The choices this design puts to the owner

Each choice says what ships while it is open, whether it can be taken alone, and a concrete example
with the unit lines, the command and what it printed.
Each is ranked by what it delivers for this route's boot, never by effort and never by what an
incumbent does.

Two of the fourth revision's choices are now two choices each: the bounds question was a single
option set that mixed a value decision with an ordering decision, and the fourth revision's V8 is V8a
(does the before-login ordering stay) and V8b (where the bound values come from).
The numbering of the other choices is unchanged, so a reader of the fourth revision finds the same
headings.

### V1. Which unit is the provider

#### Option A: the unit that performs the mount

Unit lines, on this host:

```ini
[Unit]
Requires=zfs-mount.service
After=zfs-mount.service zfs-import.target
```

and, where the generator writes a mount unit,
`Requires=` and `After=` the generated `<escaped checkout path>.mount` instead.

The command and what it printed, with the checkout absent (round four's `evidence-host-shape.log`,
boot `773296e0-6fea-4ee0-94c1-1e486e063ce4`):

```text
$ systemctl --no-ask-password show --property=Id --property=ActiveState --property=CanReload \
    --property=Result --property=ExecMainStatus zfs-mount.service
Id=zfs-mount.service
ActiveState=active
CanReload=no
Result=success
ExecMainStatus=0
```

and, from the same boot's evidence file:

```text
SWEEP checkout-row=zfsstep19/home zfs
SWEEP refused=checkout-not-the-dataset 2026-11-24T12:09:24.209Z
```

What it delivers: one actor mounts each dataset once, so there is no second mount of the checkout
path and no property beyond V4's; the accepted lost-event and propagation designs name this unit; on a
machine with a key-loading unit the checkout-specific failure arrives through that unit's requirement;
and on a machine with none the gate's own refusal names the checkout path (both measured, the second
here).
It also carries a failure this design learned in round four: where a dataset under the checkout path
cannot have its mountpoint created, the provider fails and this unit's requirement turns that into a
`dependency` failure before its own check runs (round four's `evidence-hazard-flag-on.log`).

What it costs: the provider's own status says nothing about the checkout **when the descendants carry
`canmount=noauto`**, so a reader must know which unit to read; and the pull-in is not a proof of a
mount (`ConditionPathIsDirectory=/sys/module/zfs`, in "What decides the provider on another machine").

#### Option B: a route-owned unit that mounts the checkout explicitly

```ini
# /etc/systemd/system/mount-monochromatic-checkout.service
[Unit]
Description=Mount the checkout dataset explicitly
DefaultDependencies=no
Requires=zfs-load-key@<escaped encryption root>.service
After=zfs-load-key@<escaped encryption root>.service
After=zfs-import.target zfs-mount.service
Before=systemd-user-sessions.service
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

with the checkout dataset at `canmount=noauto`. The variant without the key requirement printed this
when the key was away (round two's `evidence-route-owned-bare.log`), the unit's own result first and
then the boot's journal:

```text
Id=mount-monochromatic-checkout.service
ActiveState=failed
CanReload=no
Result=exit-code
ExecMainStatus=1

1795503201.304968 zfs-step-19-revision-2026-11-24 zfs[838]: cannot mount 'zfsstep19/home/monochromatic-git-checkout': encryption key not loaded
1795503201.312202 zfs-step-19-revision-2026-11-24 systemd[1]: mount-monochromatic-checkout.service: Main process exited, code=exited, status=1/FAILURE
```

What it delivers: a unit whose status and journal entry are the checkout's, and, where the design does
not also require the key unit, a message that names the dataset and the reason.

What it costs: a `canmount=noauto` on the checkout dataset.
That is a property of a dataset **setup creates**, not a change to one that exists: on this host the
checkout is a directory inside the home dataset and no dataset is mounted at that path (F1, F7), so
the property is written when the checkout dataset is made.
A missing property makes the provider mount the checkout and this unit fail with
`filesystem already mounted` (source reading, `zfs_main.c`, the `OP_MOUNT` branch; measured on the
same message, round two's `propagation.log:99`), a red boot for a state `doctor` could have reported;
its `ExecStop=` unmounts the checkout independently of the provider's stop, which the design's stop
ordering does not want; and with the key requirement in place its failure is the same dependency
failure as option A's (round two's `evidence-route-owned-key-removed.log`).

#### Not an option: the conditional, which option A already is

The fourth revision offered "keep the conditional" as a third unit to pick, and it was not one: option
A's own text is that conditional, written out as two branches a machine's setup chooses between
(`Requires=zfs-mount.service` where no generator-written unit exists, the generated
`<escaped checkout path>.mount` where one does).
So there is no third unit text to compare, and the fourth revision's "B over C" had nothing on its left
and nothing on its right.

```ini
# what a unit file cannot say, and what setup decides instead
[Unit]
Requires=<whichever provider this machine has>
After=<the same>
```

What it costs to leave it as a conditional in prose: a reader cannot audit which prerequisite a given
boot had, which is why option A's two branches are stated as unit lines and `doctor` reports which one
this machine is.

#### Ranking of V1

Ranking: A > B.

A over B: B's whole measured advantage, a status that names the checkout dataset, appears only in the
variant that drops the key unit's requirement, and it costs a `noauto` on the checkout dataset plus a
second mount actor whose own stop unmounts the checkout; A reaches the same fail-closed boot through
the key unit where one exists and through the gate's refusal where none does, both measured.

What ships while it is open: nothing about V1 is unsettled for this host; A is what the document's
unit text writes, and the generated-unit branch of A has never run.

Separable: yes, this choice is the two lines that name the provider, and every other choice stands
with either answer.

### V2. What the boot unit requires of the provider

#### Option A: `Requires=` plus `After=`

The text this document writes today.

```ini
[Unit]
Requires=zfs-mount.service
After=zfs-mount.service zfs-import.target
```

The command and what it printed, on round three's guest (kept log `restart-provider.log`).
The log prints the units' ids and their invocation ids as two blocks, so this pairing is the
document's, in the log's own order:

```text
$ systemctl restart zfs-mount.service
$ systemctl --no-ask-password show --property=Id --property=InvocationID \
    reconcile-git-worktree-clone-mounts.service git-worktree-nested-output-mount.service \
    git-account-session-standin.service git-vm-builder-standin.service
Id=reconcile-git-worktree-clone-mounts.service              InvocationID=a845ba3c94244579b023b19ff4d42bb7
Id=git-worktree-nested-output-mount.service                InvocationID=c416a0899b184575962dac06b8f85f01
Id=git-account-session-standin.service                     InvocationID=01623935d40241018456f53f3927d3c9
Id=git-vm-builder-standin.service                          InvocationID=a1ec513011d944b68f5baca9c69398b0
```

(before the restart those four ids were `c48b243f1a6040a6b8966b9df592b22e`,
`419f427fa4d641a380c645b86e799a7f`, `e5a00fd9ea1f46ed9e3f4932d29e59ed` and
`ecafad6210754a4fa4f3bbc7806f28e9`, in the same order), and the stop pass ran inside it:

```text
SWEEP stopped 2026-11-24T08:08:51.298Z uptime=21.72
SWEEP stopped-slot=unmounted
```

What it delivers: a dependency failure when the provider fails, the pull-in on a machine where
nothing wanted the provider, and restart propagation, which the propagation design's sentence about
"the one job-driven case the unit covers today" describes.

What it costs: a provider restart restarts every consumer and every per-dataset unit, and the stop
pass unmounts every attached clone before the pass re-attaches them, all measured in
`restart-provider.log`; and the re-run the propagation design wants is also what the provider drop-in
already provides, so the propagation is a second path to the same re-run at the price of the
disruption.
Measured this round, the requirement is also what turns a provider failure into a `dependency`
failure before the gate's own named refusal can run (round four's `evidence-hazard-flag-on.log`): a boot
whose checkout is absent for a reason inside the provider's reach says `dependency` and not
`checkout-not-the-dataset`.

#### Option B: `Wants=` plus `After=`

The same lines with `Wants=` in place of `Requires=`. Measured in round two with the drop-in
installed: after `systemctl restart zfs-mount.service` the gate stayed active and the drop-in's
request ran its reload (round two's `wants-control.log`):

```text
1795503693.546036 zfs-step-19-revision-2026-11-24 systemd[1]: Reloading reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts (Wants= control)...
1795503693.566525 zfs-step-19-revision-2026-11-24 systemd[1]: Reloaded reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts (Wants= control).
```

What it delivers: the same re-run after a provider restart, through the drop-in, with no consumer or
descendant restart and no clone unmounted; the pull-in is kept; and the gate's own refusal runs where
it otherwise would have been replaced by a `dependency` failure.

What it costs: no restart propagation, so the two accepted designs' sentences describe a mechanism the
installed unit no longer has; and a provider failure no longer fails the boot unit as a dependency,
which a machine that wants that failure to name the provider would miss.
**Unmeasured:** that a provider failure leaves the gate green and refusing on its own.
The earlier revision asserted it; no arm has run it. The nearest measured fact is
`evidence-hazard-flag-on.log`, where the provider failed and the gate's job died of `dependency`
because that unit carries `Requires=`.

#### Option C: `After=` only

```ini
[Unit]
After=zfs-mount.service zfs-import.target
```

The arm this option's evidence came from had **no edge at all**, not `After=` only: the booted unit's
own description said `(no edge to the provider)`, and its journal is
(round two's `evidence-no-provider-edge.log:92` to `:95`):

```text
1795503075.003525 zfs-step-19-revision-2026-11-24 systemd[1]: Finished zfs-mount.service - Mount ZFS filesystems.
1795503075.469533 zfs-step-19-revision-2026-11-24 systemd[1]: Starting checkout-propagation.service - Ask the boot unit to re-verify the checkout's private propagation...
1795503075.501653 zfs-step-19-revision-2026-11-24 systemd[1]: Starting reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts (no edge to the provider)...
1795503075.526567 zfs-step-19-revision-2026-11-24 systemd[1]: Finished checkout-propagation.service - Ask the boot unit to re-verify the checkout's private propagation.
```

The gate started 32 ms after the propagation service's own start, which is why it ran after the
provider; the same confound round three found in the learned-unit arm.
So this arm shows neither that `After=` alone is enough nor that it is unnecessary.

What it delivers: the least coupling.

What it costs: nothing pulls the provider in, so a boot where nothing else wanted the provider runs
without it; and the pull-in caveat in "Why each line" applies: with the module not loaded, the provider
is a condition skip.
**The re-run after a provider restart is not part of the cost.** The provider drop-in in "The provider
drop-in, from the lost-event record" is installed under every answer to this choice, and its
`ExecStartPost=` runs `reload-or-restart` on the gate, so C re-runs the gate exactly as B does; B and C
differ by the pull-in alone.

#### Ranking of V2

Ranking: B > A > C. **This is a close call, and it is the owner's.**

B over A: both deliver the re-run after a provider restart, one through propagation and one through
the installed drop-in; B delivers it without unmounting every attached clone or restarting every
consumer and descendant, which is measured to be A's price, and B leaves room for the gate's own named
refusal where A replaces it with `dependency`.
That last half is **unmeasured**, as the option's own cost line says: no arm has run a provider
failure with `Wants=`, and the nearest measured fact is the hazard boot, where the gate's job died of
`dependency` because that unit carried `Requires=`.

A over C: A delivers the pull-in on a boot where nothing else wanted the provider, and C delivers
none of it; both re-run the gate after a restart through the drop-in.

What ships while it is open: A, the text this document writes, because it is the shape every green
arm booted and the shape the two accepted records describe. Taking B is an amendment for those two
records' owners, which is why this design puts A beside B rather than deleting A.

Separable: yes, these two lines; but B's saving is only visible beside V6, whose consumers are what
restart.

### V3. How the mount units are named in the ordering

#### Option A: `RequiresMountsFor=<checkout path>`

```ini
[Unit]
RequiresMountsFor=/home/user/Monochromatic
```

The command and what it printed, on round four's guest (kept log
`state-as-written-stopsec.log`, "the dependencies systemd derived"):

```text
$ systemctl --no-ask-password show --property=Requires --property=After reconcile-git-worktree-clone-mounts.service
Requires=zfs-mount.service sysinit.target system.slice "zfs-load-key@zfsstep19-home-monochromatic\\x2dgit\\x2dcheckout.service" -.mount
After=zfs-mount.service zfs-import.target system.slice basic.target systemd-journald.socket -.mount "srv-zfs\\x2dstep\\x2d19\\x2dfixture-home.mount" "zfs-load-key@zfsstep19-home-monochromatic\\x2dgit\\x2dcheckout.service" sysinit.target
```

and the same line on the host's own systemd release, over a mount learned from the mount table
(round three's `systemd-262-arms.log` arm G):

```text
Requires=sysinit.target -.mount system.slice
After=systemd-journald.socket basic.target "srv-step19\\x2d262-bind\\x2dtarget.mount" -.mount system.slice sysinit.target
```

What it delivers: every prefix mount unit derived from the path; both provider states covered without
a line per machine; `Requires=` only on units that have a fragment, which is the safe half of the rule
in "Never require a mount unit systemd learned"; one line instead of two.

What lands differs by machine, and on this host the `Requires=` half would not attach at all: its root
mount unit `-.mount` is `LoadState=loaded` with an empty `FragmentPath` and an empty `SourcePath`
(host read, `keep/host-reads.log`), and a requirement is added only to a mount unit that has a fragment
(round four's read of v262 `unit.c`, which adds `Requires=` only `if (m->fragment_path)`).
`[nothing]` This host cannot be booted to show it, so the claim rests on that source rule and this
host's file shape together, not on a run; the guest's 258 did put `-.mount` in `Requires=`
(`state-green.log`), and its root mount unit has a fragment.

What it costs: the derived dependency set is a manager report rather than a line a reader can check in
the unit file; a machine whose path prefix differs has a different set; and the checkout's own learned
unit enters the `After=` only once it exists, so the line orders the stop half rather than the first
start.

#### Option B: the two hand-written ordering lines

```ini
[Unit]
After=home.mount
After=<escaped checkout path>.mount
```

The command that shows the first line naming nothing, on a machine whose home dataset is mounted
elsewhere. `RequiresMountsFor=`'s own expansion on round four's guest names the home dataset's unit,
and it is not called `home.mount` (round four's `state-as-written-stopsec.log`):

```text
After=zfs-mount.service zfs-import.target system.slice basic.target systemd-journald.socket -.mount "srv-zfs\\x2dstep\\x2d19\\x2dfixture-home.mount" "zfs-load-key@zfsstep19-home-monochromatic\\x2dgit\\x2dcheckout.service" sysinit.target
```

`home.mount` is the unit name for a dataset mounted at `/home`, and this machine's home dataset is
mounted at `/srv/zfs-step-19-fixture/home`.

What it delivers: every name visible in the file, and no expansion to reason about.

What it costs: `home.mount` is not the home dataset's unit name when the home dataset sits at another
path, so the line silently orders nothing; it misses the prefix's other mounts; and the learned unit's
name must be computed per machine.

#### Option C: no mount-unit edge and a prose statement

```ini
[Unit]
# The checkout's mounts are ordered by the provider's requirement.
```

What it delivers: nothing to keep in step.

What it costs: the stop half loses its only statement, and `systemd-analyze verify` cannot check the
intent.

Not run: the two lines it drops are the ones `RequiresMountsFor=` expands to, and no shutdown arm has
run in any round.

#### Ranking of V3

Ranking: A > B > C.

A over B: A covers both provider states and every prefix mount with one line, and it cannot put a
`Requires=` on a learned unit, while B's `home.mount` name is measured to be wrong on a machine whose
home dataset sits elsewhere.

B over C: B states the stop half and C states nothing.

What ships while it is open: A, measured on both systemd releases.

Separable: yes, one line; its effect on the stop half is not measured, because no shutdown arm ran.

### V4. `canmount=noauto` on every dataset under the checkout path

#### Option A: exclude them and mount them after the gate

Three arms measure what the property decides: round three ran the first two, and round four added the
third.

```text
# the flag off, canmount=on (round three's evidence-hazard-nested-auto.log)
d /srv/zfs-step-19-fixture/home/account/Monochromatic/package/vm-builder/cli/output
zfsstep19/nosnap/vm-builder-output  .../Monochromatic/package/vm-builder/cli/output  zfs  shared

# the flag off, canmount=noauto (round three's evidence-hazard-v4.log)
d /srv/zfs-step-19-fixture/home
d /srv/zfs-step-19-fixture/home/account
d /srv/zfs-step-19-fixture/home/account/Monochromatic
zfsstep19/nosnap/vm-builder-output  .../Monochromatic/package/vm-builder/cli/output  noauto  no  available

# the flag ON, canmount=on (round four's evidence-hazard-flag-on.log)
1795522243.192568 ... zfs[818]: cannot mount '/srv/zfs-step-19-fixture/home/account/Monochromatic/package/vm-builder/cli/output': failed to create mountpoint: Operation not permitted
1795522243.193878 ... systemd[1]: zfs-mount.service: Failed with result 'exit-code'.
```

The first two blocks are the logs' own `findmnt` and `zfs list` lines with the paths shortened; the
third is the boot's journal, quoted whole under "The failure controls".

What it delivers: the provider never attempts a descendant, so its own status stays out of the
checkout's story and the gate's named refusal is what a reader sees; the gated unit's explicit
`zfs mount` refuses with a named message rather than skipping; and it is the only way the design's
"gate every path descendant" sentence can hold.

What it costs: a property change on existing datasets, which is the owner's act, and on this host there
is exactly one of them, `zpcachyos/nosnap/vm-builder-output`, mounted today at
`/home/user/Monochromatic/package/vm-builder/cli/output` (this round's read, `keep/host-reads.log`);
a dataset under the path that setup misses is mounted by the provider again, so `doctor` must read and
report it; and a dataset under the path is no longer mounted by `zfs mount -a` alone, so anything that
expected that must name it.

What it costs on this host until a mounter lands is stated in V9's last subsection: that dataset stops
being mounted at boot, and everything that writes under
`/home/user/Monochromatic/package/vm-builder/cli/output` writes into the checkout instead.

Step 20's snapshot coverage and the setup runbook are the two other places this property has to be
written down, and neither is changed by this design.

#### Option B: let the provider mount them and have the gate report them

The command that leaves it as it is, and what round three's two hazard logs read back:

```text
$ zfs get -H -o property,value canmount,mountpoint,mounted zpcachyos/nosnap/vm-builder-output
canmount	on
mountpoint	/home/user/Monochromatic/package/vm-builder/cli/output
mounted	yes
$ findmnt --output TARGET,SOURCE,FSTYPE,PROPAGATION --target /home/user/Monochromatic/package/vm-builder/cli/output
TARGET                                                 SOURCE                             FSTYPE PROPAGATION
/home/user/Monochromatic/package/vm-builder/cli/output zpcachyos/nosnap/vm-builder-output zfs    shared
```

(That is this round's read of this host, kept in `keep/host-reads.log`; the guest's own `on` is in
round three's `evidence-hazard-nested-auto.log`.)

What it delivers: no property and no extra units.

What it costs, measured three ways: with the flag off, an account-visible mount of the descendant
created by the provider under the exposed home underlay with no checkout mounted over it; with the
flag on, the same dataset's mountpoint cannot be created, so the provider unit fails and, because the
boot unit requires it, the gate's job dies of `dependency` instead of refusing with its own reason
(round four's `evidence-hazard-flag-on.log`, the provider's `status=1/FAILURE`).
What that boot also shows is that the failure is not the whole mount: the home dataset mounted before
the provider reached the dataset that failed, so datasets outside the failed mountpoint's subtree
still mount (same log, `zfs list` and the unit list).
Either way the design's gated-descendant claim becomes a report rather than a gate.

#### Ranking of V4

Ranking: A > B.

A over B: the two flag-off arms are the difference for a machine whose entry is not yet immutable, and
the flag-on arm is the difference for one that is: A keeps the provider green and the refusal the
gate's, while B makes any descendant whose mountpoint cannot be created fail the whole boot's mount.

What ships while it is open: A, the property setup reads and `doctor` reports.

Separable: yes; either mounter in V9 needs it, so it is not a consequence of V9's answer.

### V5. How the propagation window is closed

#### Option A: the gate's own first act makes the checkout private

```text
$ findmnt --noheadings --output PROPAGATION --target <checkout path>
private
```

Measured in every green boot this route has run, the step's own line being
`SWEEP propagation=private`; this round's green boot holds it
(kept log `evidence-green.log`), and the host's path is `shared` before any step
(`findmnt --noheadings --output PROPAGATION --target /home/user/Monochromatic` in
`keep/host-reads.log`).

What it delivers: one step, in the same pass that verifies, measured in every green boot.

What it costs: a descendant mounted by the provider before the step is a child of a shared parent for
that window, which V4 removes.

#### Option B: order each descendant after the gate's propagation step

```ini
# the per-dataset unit, ordering after the gate's propagation step rather than before sessions
[Unit]
After=reconcile-git-worktree-clone-mounts.service
```

What it delivers: the window closed by ordering rather than by a step.

What it costs: the provider cannot be ordered, so only its own scan can be prevented, which is V4,
and a per-dataset unit later mounts into whatever propagation the parent has.

Not run: the ordering edge is the one V9's option A already carries, and what it lacks is a step to
order, which no arm has.

#### Ranking of V5

Ranking: A > B.

A over B: B has no mechanism the provider obeys, and A is measured and already the design's.

What ships while it is open: A.

Separable: yes, one step; but B is not implementable without V4 and V9 together.

### V6. What carries the checkout's readiness to the account

#### Option A: the account's services require the boot unit, named one by one

```ini
# /etc/systemd/system/git-vm-builder-standin.service
[Unit]
Description=A vm-builder service of the account that needs the dataset under the checkout
Requires=git-worktree-nested-output-mount.service
After=git-worktree-nested-output-mount.service
Before=systemd-user-sessions.service

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=<the service's own command>
```

The journal of round four's green boot, where the consumer that requires the per-dataset unit ran
after it and saw the mount (round four's `evidence-as-written-stopsec.log`):

```text
1795522041.247040 zfs-step-19-fourth-2026-11-24 systemd[1]: Finished git-worktree-nested-output-mount.service - Mount a dataset whose mountpoint lies inside the verified checkout.
1795522041.249253 zfs-step-19-fourth-2026-11-24 systemd[1]: Starting git-vm-builder-standin.service - A vm-builder service of the account that needs the dataset under the checkout...
1795522041.261391 zfs-step-19-fourth-2026-11-24 systemd[1]: Finished git-vm-builder-standin.service - A vm-builder service of the account that needs the dataset under the checkout.
SESSION-DESCENDANT mounted=zfsstep19/nosnap/vm-builder-output
```

What it delivers: a real prerequisite, visible in the journal per service, and independent of login.

What it costs: it protects nothing about login; `boot-reconcile` exits zero when clones fail, so a
clone failure does not reach a consumer through the requirement; the service list is still to be
named; every consumer is an unbounded oneshot (`TimeoutStartUSec=infinity`, measured in
`state-green.log`), so a hung command of the account's holds logins through this unit's own
`Before=systemd-user-sessions.service`; and with V2's option A every consumer restarts when the
provider restarts, while with V2's option B it does not.
With V9's option B there is no per-dataset unit for a descendant consumer to name, so this option
needs a per-dataset object to exist.

#### Option B: gate the account's user manager

```ini
# /etc/systemd/system/user@<uid>.service.d/50-checkout.conf
[Unit]
Requires=reconcile-git-worktree-clone-mounts.service
After=reconcile-git-worktree-clone-mounts.service
```

What it delivers: everything the account starts is behind the gate.

What it costs: no design here for its ordering against `systemd-user-sessions.service`, or for a
refused boot that would leave the account entirely without a session; and a failed gate would remove
far more than the checkout.

Not run: dropping a `user@` instance's start on a failed gate is a boot with no session at all, which
this round did not want to leave a guest in.

#### Option C: rely on login ordering alone

No consumer requirement; sessions start after the gate's job by the gate's own `Before=`.
Measured in round four's host-shape boot: the gate's job failed with its own refusal and sessions
started anyway (round four's `evidence-host-shape.log`, `1795522164.523793`).

What it delivers: nothing to name.

What it costs: measured, sessions start after a failed gate
(round four's `evidence-host-shape.log`, `evidence-hazard-flag-on.log`; round three's
`evidence-key-removed.log`, `evidence-flag-removed.log`, `evidence-bound.log`), so a session begins
with no checkout and no signal.

#### Ranking of V6

Ranking: A > B > C.

A over B: A fails only what needs the checkout and leaves the session usable; B's blast radius is the
whole session and has no design for the ordering it would need.

B over C: C carries nothing, while B at least refuses something.
**That pair is a value judgment and the owner's**: fail closed against login stays available is a
choice about what a person finds after a bad boot, not a fact about what each delivers.

What ships while it is open: A, with the consumer list still to be named.

Separable: A is separable from V2 and V5; it is not separable from V9, because the unit a descendant
consumer names is what V9 decides.

### V7. What `doctor` and setup report

#### Option A: the whole set

This design's answer, in plain words:

- which of the three provider states the machine is in, and the unit that performs the mount;
- whether a key-loading unit exists, and its name, **and whether the installed boot unit's key lines
  match that answer**, because a unit that requires a unit the boot does not install never runs and
  says nothing;
- every dataset under the checkout path whose `canmount` is not `noauto`;
- the boot unit's start bound, stop bound, and **the stop-failure mode as `systemctl show` reports it**
  together with the drop-in paths systemd parsed, because the mode is decided by the drop-in file's
  name and a file that loses to the distribution's reads green while the bound still aborts;
- the provider drop-in and the boot unit's `ExecReload=`, with the note that a copy systemd has not
  reloaded is not seen;
- the propagation service and its drop-in, installed only beside a boot unit that can reload, with the
  escaped name spelled as `systemctl` prints it;
- the hook's name being one ZED's scan accepts, and whether `zfs-zed.service` is enabled and active;
- what an unprivileged reader can see of AppArmor: whether `/etc/apparmor.d/zed` carries a `flags=`
  line, whether `/etc/apparmor.d/local/zed` exists, whether `apparmor.service` is enabled and active,
  what `/etc/apparmor/modes` says, and whether the profile's zedlet rule names a folder that exists;
  it cannot read the loaded profile list and says so;
- the installed file's owner, mode, ancestors, `--version` and digest;
- whether the mode choice V9's option A needs exists in this build's grammar.

A run, on this host, unprivileged, from `keep/host-reads.log`:

```text
$ systemctl --no-ask-password is-enabled zfs.target zfs-import.target zfs-import-cache.service zfs-mount.service zfs-zed.service apparmor.service
enabled
enabled
enabled
enabled
disabled
enabled
$ systemctl --no-ask-password is-enabled zfs-load-key.service
masked
$ cat /etc/apparmor/modes
default enforce
include default
reload yes
fsp no
$ ls -la /etc/apparmor.d/local/zed
ls: cannot access '/etc/apparmor.d/local/zed': No such file or directory
$ ls -d /usr/lib/zfs-linux /usr/lib/zfs/zed.d
ls: cannot access '/usr/lib/zfs-linux': No such file or directory
/usr/lib/zfs/zed.d
```

That is the provider state (2), the key unit (none, masked), the AppArmor facts this design's hook
waits on, and the zedlet folder mismatch, from one unprivileged session.

What it delivers: every state a boot depends on is one command away, and each finding prints the
command that repairs it.

What it costs: a long report, and every line must be kept true as the design changes.

#### Option B: the provider state only

```ini
provider: zfs-mount.service
```

What it delivers: a short report.

What it costs: it reads green on a machine with the wrong `canmount`, no `ExecReload=`, a hook name ZED
ignores, a profile that cannot exec the hook, a disabled ZED, or key lines that name a unit the boot
does not install.

Not run: this option is one line of the whole-set report, and its own run would say nothing the
whole-set run does not.

#### Ranking of V7

Ranking: A > B.

A over B: the states that make a boot fail are exactly the ones B leaves unread, and this round added
three more of them: the key lines against the machine, the zedlet folder's exec grant, and a
`zfs-zed.service` that is off.

What ships while it is open: A, minus its last line, which needs V9's answer.

Separable: yes, the report is read-only.

### V8a. Does the ordering before `systemd-user-sessions.service` stay

This is the first of the two questions the fourth revision mixed into one.
The second, where the bound values come from, is V8b.
The two are separable in one direction only: V8a's answer about the ordering is what gives V8b's
numbers their meaning, and a machine can answer V8a while V8b is still open, which is the state this
document is in.

The fact the choice turns on, measured: the ordering is real and the login is **delayed**, never
refused. In this round's green boot the gate finished at `1795529986.223586` and sessions started at
`1795529986.582386`; in every failing arm, including this round's stages boot, sessions start after the
failed job (`Starting systemd-user-sessions.service` at `1795530394.711395`).

#### Option A: keep the ordering on all four units, with bounds on each

```ini
# the boot unit
Before=systemd-user-sessions.service
# the per-dataset unit and both consumers
Before=systemd-user-sessions.service
```

What it delivers: the checkout is ready, or known not to be, before any account session; the per-dataset
unit's mount is in place before an account's service that requires it starts; and the hold is bounded
where the bounds exist.

What it costs: a hung program in any of the four units holds every login for as long as its own bound,
and three of the four have no bound at all today (`TimeoutStartUSec=infinity`, measured in
`state-green.log`), so this option is only complete if the per-dataset unit and the consumers get
bounds too, which is V8b's job. It also holds logins for a failure that only the account's own services
need, which is the price of ordering before sessions rather than before the services.

#### Option B: drop the ordering from all four units, and let each consumer require what it needs

```ini
# the boot unit, the per-dataset unit and every consumer
# Before=systemd-user-sessions.service   <- removed from all of them
```

and every consumer that needs the checkout or a dataset under it requires the unit that provides it
(V6's option A).

The command and what it printed: this is a removal, so what is measured is the state it produces.
Every failing arm already shows a session starting with the checkout absent: this round's stages boot
has the gate failed at `1795530394.707449` and
`Starting systemd-user-sessions.service` at `1795530394.711395`, and round four's host-shape and
hazard boots do the same.

What it delivers: no login hold at all, whatever the pass does; the readiness reaches exactly the
services that name it, and a person can log in to look at a failed boot.

What it costs: an interactive session begins with no checkout until the services that need it fail,
so a person at the keyboard sees a directory with nothing mounted in it and no prompt-time signal; the
interactive shell is not one of the consumers, so nothing refuses it.
The ordering line also has to come off all four units at once, or the hold remains through whichever
unit kept it.

#### Ranking of V8a

Ranking: A > B.

A over B: A delivers the checkout before a session for every program of the account, interactive ones
included, and its cost is the bound arithmetic V8b prices; B delivers a boot a person can always log in
to, and its cost is a session that cannot tell whether the checkout is coming or gone, which is what
the whole design exists to answer before a service reads it.

**The pair is a value judgment about a bad boot, and it is the owner's:** a boot where nobody can log in
for the length of the bounds, against a login whose checkout may be missing.

What ships while it is open: A, because it is what every green arm booted and what V8b's numbers are
for.

Separable: no, in one direction. B is not available without V6's option A, since the readiness has to
reach something; A is available whatever V6 answers.

#### Not run

B is A's line removed from four units, and no arm has run with the line removed from all four: the
failing arms show sessions starting with the checkout absent while the line is still installed and the
ordering is satisfied by the failure, which is a different state.

### V8b. Where the bound values come from

#### Option A: the measured ceilings, once step 22 measures them

```ini
[Service]
TimeoutStartSec=<the measured ceiling of one reconcile>
TimeoutStopSec=<the measured ceiling of one stop pass>
```

and the same two lines on the per-dataset unit and on each consumer, sized to their own measured work.

The command and what it printed:

```text
$ systemctl --no-ask-password show --property=Id --property=TimeoutStartUSec --property=TimeoutStopUSec \
    reconcile-git-worktree-clone-mounts.service git-worktree-nested-output-mount.service \
    git-account-session-standin.service git-vm-builder-standin.service
Id=reconcile-git-worktree-clone-mounts.service
TimeoutStartUSec=20s
TimeoutStopUSec=5s
Id=git-worktree-nested-output-mount.service
TimeoutStartUSec=infinity
TimeoutStopUSec=45s
Id=git-account-session-standin.service
TimeoutStartUSec=infinity
TimeoutStopUSec=45s
Id=git-vm-builder-standin.service
TimeoutStartUSec=infinity
TimeoutStopUSec=45s
```

(kept log `state-green.log`; the gate's two values there are this round's test values, and the three
`infinity` lines are the state this option removes.)

What it delivers: every bound is one a run produced, and the hold is a measured number a reader can
check against the measurement; nothing ships as a guess.

What it costs: nothing bounds this boot until step 22 measures, and no measurement of one reconcile or
one stop pass exists today; a bound set at the measured ceiling kills a pass that is merely slower for
a reason the measurement did not include, and then runs the stop phase, whose own stages add up to
three stop bounds.

Not run: no measured value exists to boot.

#### Option B: numbers the owner decides, written down as decisions

```ini
TimeoutStartSec=5min
TimeoutStopSec=1min
```

on the gate, and the same kind of decision for the per-dataset unit and the consumers.

What it delivers: a bounded hold now, in the shape the project's rule for shipped limits allows, which
is a value with its reason beside it; and a bound loose enough that it only fires on a real hang.

What it costs: the numbers are decisions and not measurements, so the hold they permit is the largest
of the bounded options and the arithmetic behind it is the one in "What a hung start costs": the
probe's deadline plus five minutes plus one minute per stop stage that survives, which is about 6
minutes for the cooperative shape and about 8 minutes 20 s for the worst one. It also costs a second
decision per unit for the per-dataset unit and the consumers, each with its own work.
Not run: no arm has booted five-minute and one-minute bounds.

#### Option C: no bound on the gate, and the ordering dropped, so no number ships

```ini
# no TimeoutStartSec=
# no Before=systemd-user-sessions.service
```

What it delivers: no unsourced number, and no login hold.

What it costs: it is V8a's option B with the bounds question unanswered, so it gives up the checkout
before a session *and* leaves a hung pass unreaped until somebody stops the unit; the per-dataset unit
and the consumers are unbounded anyway (`infinity`), so dropping the gate's bound alone does not make
the boot bounded.

Not run, for the reason V8a's option B is not run.

#### Option D: no bound, and the ordering kept

```text
TimeoutStartUSec=infinity
```

(round three's `systemd-262-arms.log` arm A, round two's `start-bound.log`, and this round's
`state-green.log` for the three units that have no bound today.)

What it delivers: no number to defend, and no reconcile killed mid-attach.

What it costs: a hung reconcile holds every login with no limit; the stop phase's own default is 90 s
per stage; and the boot that results is one nobody can log in to, cleared only by a person with console
access.

#### Ranking of V8b

Ranking: A > B > C > D. **B is what ships until step 22 measures, and the owner decides it.**

A over B: A's numbers are measurements and B's are promises, which is the difference the project's rule
for shipped limits asks for; both bound the same stages, so the hold differs only by the values.

B over C: both leave a number to be supplied, and B bounds what can go wrong now while C leaves a hung
pass to a person; C's one advantage, that it ships no unsourced number, is bought by giving up both the
bound and the ordering.

C over D: both have no number, and C at least refuses to hold logins while it has none, which is the one
combination D must not have.

What ships while it is open, named exactly: **V8a's option A lines, no bound value on any of the four
units, and B's numbers as the owner's decision if the owner makes it.** This document does not dress
the guest's test values up as shipped values: `TimeoutStartSec=20` and `TimeoutStopSec=5` appear in the
arms and nowhere else.

Separable: yes, from every other choice, both the ordering question and the values question; but the
numbers only mean something once V8a is answered, and the per-dataset unit and the consumers need their
own bound or their own answer to V8a, which is why the two questions are posed together.

### V9. Who mounts a dataset under the checkout path

#### Option A: one unit per dataset, after the gate and before sessions, running the handler's mode

```ini
# /etc/systemd/system/git-worktree-nested-output-mount.service
[Unit]
Description=Mount a dataset whose mountpoint lies inside the verified checkout
Requires=reconcile-git-worktree-clone-mounts.service
After=reconcile-git-worktree-clone-mounts.service
Before=systemd-user-sessions.service

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=<root-file> <root-words> mount-path-descendant --dataset <dataset>

[Install]
WantedBy=multi-user.target
```

The command and what it printed, in this round's green boot, where the stand-in stands in that unit
exactly as the text writes it (kept logs `state-green.log`, `evidence-green.log`):

```text
$ systemctl --no-ask-password show --property=Id --property=ActiveState --property=CanReload \
    --property=Result git-worktree-nested-output-mount.service
Id=git-worktree-nested-output-mount.service
ActiveState=active
CanReload=no
Result=success

1795529986.230947 zfs-step-19-fifth-2026-11-24 systemd[1]: Starting git-worktree-nested-output-mount.service - Mount a dataset whose mountpoint lies inside the verified checkout...
1795529986.262900 zfs-step-19-fifth-2026-11-24 systemd[1]: Finished git-worktree-nested-output-mount.service - Mount a dataset whose mountpoint lies inside the verified checkout.
NESTED start 2026-11-24T14:19:46.235Z
NESTED mounted=zfsstep19/nosnap/vm-builder-output 2026-11-24T14:19:46.262Z
SESSION-DESCENDANT mounted=zfsstep19/nosnap/vm-builder-output
```

What it delivers: the mount after the verified checkout and before sessions, its own journal entry,
a per-dataset failure, and a consumer that requires exactly it and sees it.

What it costs: the mode `mount-path-descendant` does not exist, and its `--dataset` conflicts with the
grammar principle this design quotes under "The units for the datasets under the checkout path", so
this option ships only once the root-modes design admits both; one unit per dataset, generated by setup
and reported by `doctor`; a dataset under the path whose unit setup forgets is silently unmounted; and
one more unit for every new dataset.
Against C it buys the two rules C cannot express, the refusal of a dataset outside the checkout and the
already-mounted answer, and it pays for them with a mode, which C does not need.

#### Option B: the boot unit's own pass mounts them, one explicit `zfs mount` each

The pass mounts each dataset after it verifies the checkout, in the same process, with no per-dataset
unit. Round three measured a stand-in for that shape (kept log `evidence-v9b.log`, boot
`7f58994e-ac75-4789-a678-d16cebe76fe2`):

```text
SWEEP start 2026-11-24T08:10:27.493Z mode=boot-reconcile uptime=6.53
SWEEP checkout-row=zfsstep19/home/monochromatic-git-checkout zfs
SWEEP attached=registered 2026-11-24T08:10:27.526Z
SWEEP descendant-mount start 2026-11-24T08:10:27.536Z
SWEEP descendant-mount exit=0 2026-11-24T08:10:27.551Z
SWEEP descendant-mounted=zfsstep19/nosnap/vm-builder-output 2026-11-24T08:10:27.555Z
```

and the consumer that names a per-dataset unit in that same boot got nothing at all:

```text
$ systemctl --no-ask-password show --property=Id --property=ActiveState --property=CanReload --property=Result git-vm-builder-standin.service
Id=git-vm-builder-standin.service
ActiveState=inactive
CanReload=no
Result=success
```

What it delivers, once its step lands: one unit, one ordering edge, and the mount in the same process
as the verification that authorizes it, with no new mode and no per-dataset unit to keep in step; a
consumer that requires the boot unit runs after the pass, so every descendant is mounted by the time
it starts.

What it costs: **the step does not exist** (`boot-reconcile` has no such call), and it needs a dataset
list, a failure rule for the readiness record and the exit status, and idempotence on `ExecReload=`;
a consumer cannot require one descendant's readiness, only the whole pass's; a consumer that names a
per-dataset unit anyway gets nothing, silently, measured in `evidence-v9b.log`; and the pass's start
bound then has to
cover every descendant's mount.

#### Option C: one unit per dataset running `zfs mount <dataset>` directly, after the gate

This is the option that was printed as a strawman in the fourth revision.
With the same `[Unit]` lines the other two options carry, it mounts the dataset at a boot today, and
this round booted it.

```ini
# /etc/systemd/system/git-worktree-nested-output-mount.service
[Unit]
Description=Mount a dataset whose mountpoint lies inside the verified checkout
Requires=reconcile-git-worktree-clone-mounts.service
After=reconcile-git-worktree-clone-mounts.service
Before=systemd-user-sessions.service

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/usr/bin/zfs mount <dataset>

[Install]
WantedBy=multi-user.target
```

The command and what it printed, in this round's boot (kept log `evidence-v9c.log`, boot
`14c29514-53e9-4fa5-9daa-fc0d742f810b`; the guest's `zfs` is its source build's, at
`/opt/openzfs-2.4.4/sbin/zfs`, which is the same call):

```text
$ systemctl --no-ask-password show --property=InvocationID git-worktree-nested-output-mount.service
InvocationID=e72f8b120bb2443e8fb0496ac5a4c6fd
1795530175.291209 zfs-step-19-fifth-2026-11-24 systemd[1]: Starting git-worktree-nested-output-mount.service - Mount a dataset whose mountpoint lies inside the verified checkout...
1795530175.309134 zfs-step-19-fifth-2026-11-24 systemd[1]: Finished git-worktree-nested-output-mount.service - Mount a dataset whose mountpoint lies inside the verified checkout.
SESSION-DESCENDANT mounted=zfsstep19/nosnap/vm-builder-output
```

What it delivers: the mount after the verified checkout and before sessions, its own journal entry, a
per-dataset failure, and a consumer that requires exactly it and sees it, all with no new mode and no
new refusal to invent; the ordering is the gate's own, so a unit that starts early cannot mount under
an unverified checkout, which is what the fourth revision's print wrongly implied.

What it costs, measured in the same boot:
- **it is not idempotent.** After `systemctl restart zfs-mount.service` the unit re-ran its explicit
  mount on the dataset that was still mounted and ended `Result=exit-code`, `ExecMainStatus=1` with
  `cannot mount 'zfsstep19/nosnap/vm-builder-output': filesystem already mounted`, and its consumer's
  job failed with `dependency` (kept log `restart-provider-v9c.log`).
  Fixing that means a wrapper around `zfs mount` that treats "already mounted" as success, which is one
  more thing to install and test;
- it re-checks nothing: the unit mounts whatever dataset its file names, so a machine whose
  `/etc/systemd/system` copy names a dataset outside the checkout mounts it under a verified checkout,
  which is the refusal option A's mode would carry and C cannot;
- one unit per dataset, generated by setup and reported by `doctor`, with the same silent case A has: a
  dataset whose unit setup forgets is never mounted;
- and it leaves the two-mounters question open in the same way A does, since the provider still scans
  the path unless V4's property excludes these datasets.

#### Ranking of V9

Ranking: A > C > B. **A over C is a close call, and it is the owner's.**

A over C: A delivers a refusal for a dataset whose mountpoint is outside the checkout and a program
that treats an already-mounted dataset as success, both of which C has to be taught by a wrapper of its
own; C's whole advantage is that it ships today, measured in this round's boot.

C over B: both mount after the gate, and C mounts today while B does not exist at all: `boot-reconcile`
has no step that mounts a dataset under the checkout path, and `rg 'zfs mount'` over its folder matches
comments only (source reading, `boot_reconcile_mode.rs` and the folder). B's shape, measured in round
three, left a consumer that names a per-dataset unit with nothing at all.

What the ranking does not settle, and the owner should weigh: A's per-dataset failure and granularity
against C's shipping today with no new mode; and B's single actor, which has no per-dataset unit to
drift and no new mode, against both of them.

What ships while it is open, and the truth about this host: **nothing mounts a dataset under the
checkout path**, unless C is taken. C needs no new code, so "nothing ships" describes A and B only.

#### What V4 and V9 do to this host's one dataset under the path

This is the consequence the fourth revision stated only in passing, and it is the one an owner deciding
V4 needs in front of them.

This host has one dataset under the checkout path, `zpcachyos/nosnap/vm-builder-output`, and it is
`canmount=on`, `mounted yes`, at
`/home/user/Monochromatic/package/vm-builder/cli/output` right now
(this round's read, `keep/host-reads.log`).
What the two choices do to it:

- **V4-A (`canmount=noauto`) and neither mounter:** the provider skips it at every boot, and nothing
  else mounts it. `doctor` reports the dataset as not mounted, and a `vm-builder` run that writes
  under that path writes into the checkout instead; whether the directory is there to write into
  depends on the checkout's own tree, which this round did not read through the mount.
  `[nothing]` This is the state today's design ships while V9 is open.
- **V4-A and V9-C:** the per-dataset unit mounts it after the gate, before sessions; a provider restart
  fails that unit on the already-mounted message until C's wrapper lands.
- **V4-A and V9-A or V9-B:** the mode or the pass mounts it, and it stays unmounted until that code
  lands.
- **V4-B (leave it `on`):** the provider keeps mounting it at boot, with the exposure and the failure
  modes V4's two costs describe: with the covered entry's flag off it mounts into the exposed underlay;
  with the flag on the whole boot's mount fails and the gate's job dies of `dependency`.

So V4 and V9 are coupled in one direction: **any V4-A answer needs a mounter, and until one lands this
host's `vm-builder-output` is not mounted at boot at all.** V9-C is the only answer that restores it
without new code.

## What each part waits on

Named so a reader can see which piece is blocked, and on whom.

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
- The other mounter: a step inside `boot-reconcile` that mounts each dataset under the checkout path,
  with a dataset list, a failure rule, and idempotence on `ExecReload=`. That is step 19's build too.
- An AppArmor grant for the hook's file and for the pass it starts: no design owns it yet; the AppArmor
  section weighs the two shapes that were weighed and lists the four that were not.
  Whether ZED runs at all on this host is a host fact, not a waiting item: `zfs-zed.service` is
  `disabled` and `inactive`, so nothing waits on the grant until the owner enables ZED, and the
  question is put then.
- `TimeoutStartSec=` and `TimeoutStopSec=` values: a measurement of one reconcile and one stop pass, or
  the owner's ceilings, which is choice V8b; and the separate question of whether the ordering before
  sessions stays, which is V8a.
- Whether consumers restart on a provider restart: choice V2.
- Whether a per-dataset unit, the pass, or an explicit `zfs mount` mounts a descendant: choice V9, whose
  third answer needs no new code and no waiting.
- A wrapper for the explicit `zfs mount` if V9's C is taken, so an already-mounted dataset is success:
  V9's C's one piece of work.
- `canmount=noauto` on the datasets under the checkout path: an owner's property decision, taken on
  datasets setup creates (V4), and step 20's snapshot coverage and the setup runbook must state it too.
  Until a mounter lands, taking it leaves this host's `vm-builder-output` unmounted at boot.

## Edits this design suggests

Suggestions only; this round edits neither record.
Where an edit presumes a choice, the choice is named, so a reader can tell which parts wait.

To plan step 19, "Open items", which reads:

> Two parts the design leaves open block its acceptance arms:
> the checkout provider stays conditional in "One-time move and checkout boot protection", which names "the
> protected `home-user-Monochromatic.mount` if using a generated mount unit, or `zfs-mount.service` where it mounts
> the checkout", and "Checkout boot mount protection" says to prove the failure control and production ordering
> before enabling the gate.

the suggestion, presuming V1's option A, is to replace that sentence with the fixed provider, the key unit
and a pointer:

> The checkout provider is fixed on a machine of this host's shape as `zfs-mount.service`, whose `zfs mount -a`
> mounts the checkout because the ZFS mount generator writes no unit here; where the generator's cache file exists
> and names the checkout dataset, the provider is the generated `<escaped checkout path>.mount` instead.
> The checkout's key-loading unit is the generator's `zfs-load-key@<escaped encryption root>.service` where one
> exists, and the provider's status is not a readiness signal for the checkout where the datasets under the
> checkout path carry `canmount=noauto`.
> The failure control, the production ordering, the hazard arms and the route-owned alternative are measured in
> `doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`, which also fixes the units'
> whole text, their start and stop bounds, the key and provider drop-ins, and the exclusion every dataset mounted
> under the checkout path needs.

To plan step 19, "What it builds", the sentence "The gate that orders every path descendant after a verified
checkout mount" needs the condition this design measured:

> The gate that orders every path descendant after a verified checkout mount, where each such dataset carries
> `canmount=noauto` so that the automatic mount cannot mount it first, and where either a unit per descendant or
> the gate's own pass mounts it after the verification and before `systemd-user-sessions.service`.

To the design's "Boot and shutdown unit", the unit text in this document's "The boot and shutdown
unit", presuming V1's option A, V3's option A, V5's option A and either answer to V2, V8a, V8b and V9.

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

To the lost-event design's "The hook, in one form a sibling can cite", one amendment if choice V2's option B is
taken: its sentence that the boot unit's `Requires=` on the provider makes a provider restart re-run the pass
describes a unit this design would no longer install, and the re-run then rests on the same design's drop-in,
which the record already installs.

To the propagation design's "What becomes of `Type=oneshot` with `RemainAfterExit=yes`", the same amendment and
the same reason.

To the root-modes design, two additions to its mode list, and one question before them: whether a mode may change a
dataset's state named by an argument, which `HandlerMode`'s doc forbids today. If it may,
`mount-path-descendant --dataset <dataset>`, idempotent on an already-mounted dataset, refusing a dataset whose
mountpoint lies outside the checkout with `dataset-outside-the-checkout`, and the reason choice V9's option A needs
it. If it may not, V9's option A has no call to make and V9's option B's step is the only mounter that
needs new code there; V9's option C needs none.

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
  the gate's job fails with `dependency` before its own check. Basis: kept log
  `evidence-hazard-flag-on.log`.
- F13. The unit text as this design writes it, both bounds included, boots green with the order gate,
  per-dataset unit, its consumer, the account consumer and sessions. Basis: round four's
  `evidence-as-written-stopsec.log`.
- F14. The checkout's key unit failing fails the gate, both consumers and the per-dataset unit with
  `dependency`, and sessions start after. Basis: round three's `evidence-key-removed.log`, journal
  lines `1795507758.973672` to `1795507760.676330`.
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
- F27. A provider restart restarts the gate, the per-dataset unit and both consumers, and the stop pass
  unmounts the attached slot. Basis: round three's `restart-provider.log`, four units.
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
- F57. The per-dataset unit and both account consumers have `TimeoutStartUSec=infinity`,
  `TimeoutStopUSec=45s` and `TimeoutStopFailureMode=abort` from the distribution's own drop-in, and each
  carries `Before=systemd-user-sessions.service`. Basis: kept log `state-green.log`.
- F58. A per-dataset unit running an explicit `zfs mount <dataset>` after the gate mounts the dataset at
  a boot, and fails with `filesystem already mounted` after a provider restart, taking its consumer's
  job down with `dependency`. Basis: kept logs `evidence-v9c.log`, `restart-provider-v9c.log`.
- F59. This host has one dataset under the checkout path, `zpcachyos/nosnap/vm-builder-output`, at
  `canmount=on`, `mounted yes`, mountpoint
  `/home/user/Monochromatic/package/vm-builder/cli/output`, propagation `shared`. Basis: host read,
  `keep/host-reads.log`.
- F60. The green boot of the shipped text, with the stop-failure drop-in beside it, runs gate, account
  consumer, per-dataset unit, its consumer, then sessions, and reports
  `TimeoutStopFailureMode=terminate` while Fedora's `service.d/10-timeout-abort.conf` is installed and
  unmasked. Basis: kept logs `state-green.log`, `evidence-green.log`.

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

In the earlier rounds' kept copies: OpenZFS `zfs-2.4.4` `cmd/zfs/zfs_main.c`,
`etc/systemd/system-generators/zfs-mount-generator.c`, `cmd/zed/{zed_exec,zed_event}.c` and
`man/man8/zed.8`; systemd v258 and v262 `src/core/{job,mount,service,transaction,unit}.c` and the
`systemd.{service,exec,unit}` manuals; and this host's `apparmor.d(5)` for the exec modes `ix`, `px`
and the read permission an interpreted program needs.

## Measurements and logs

This round's folder: `~/temp/agent/zfs-step-19-fifth-revision-2026-11-24/`.

- `keep/guest-scripts/`, every guest step as its own script, so each arm is reproducible:
  `01-prepare.sh`, `02-fixture.sh`, `03-fix-fixture.sh`, `10-units.sh`, `11-state.sh`,
  `12-evidence.sh`, `13-reboot.sh` and `99-ready.sh`.
  `01-prepare.sh` and `02-fixture.sh` are round three's, unchanged and re-run; `03-fix-fixture.sh` and
  `13-reboot.sh` are round four's, unchanged; `10-units.sh`, `11-state.sh` and `12-evidence.sh` are
  round four's with this round's changes, and the differences are two stand-ins that ignore SIGTERM,
  the stop-failure drop-in and its name, three state names (`green`, `v9c`, `stages`), a per-dataset
  unit written in the direct shape, and the per-unit bounds read.
- `keep/logs/`, every step's whole output, named for its arm.
- `keep/host-reads.log`, the host reads with their exact output, every host fact this revision states.
- `keep/apparmor-and-unit-reads.log`, the AppArmor profile and policy reading, with its limits.
- `keep/state-so-far.md`, the round's running state.

The guest was created and destroyed through `mvm --backend libvirt` by the name its own create printed:
`mvm-create-guest.log` holds the create at 2026-11-24T13:40:27Z and `mvm-destroy-guest.log` the destroy,
and `mvm-list-after-destroy.log` reports no guest.

One log is a partial stream: `prepare.log` is the build's output as the guest agent delivered it, cut
short before the script's own read-back. `prepare-readback.log` is the same script's second run, which
repeats every read-back in full because the build was already done.
`fixture.log` ends at a busy unmount, which is the step round four documented; `fixture-fix.log` is the
completion.

## Revisions

The first revision was written after the first gate review; the second after a same-vendor re-read of
that revision; the third after a second gate review; the fourth after a same-vendor read of the third;
and this one after a same-vendor read of the fourth.
Earlier revisions' narratives of their own folds are folded into the text they changed and are not
repeated here.

This revision:
- states the worst case as the probe plus the start bound plus one stop bound per stop stage whose
  programs survive SIGTERM, and measures the three-stage case in a boot;
- moves the stop-failure mode out of the unit text into a drop-in whose file name sorts after the
  distribution's, after measuring that the directory is not what decides;
- splits V8 into the ordering question and the values question, gives option C the treatment option E
  had, and states the per-dataset and consumer units' unbounded defaults;
- writes V9's option C with the ordering lines the other two carry, boots it, and measures what a
  provider restart does to it;
- states what V4 and V9 together do to this host's one dataset under the checkout path;
- weighs the six AppArmor shapes instead of two, and corrects the ranking's "least privilege" claim;
- adds a fifth reading of the fail-closed question: the provider's failure leaves datasets outside the
  failed subtree mounted, which the hazard log holds and the OpenZFS source explains;
- takes every host read again in this round and keeps it;
- re-poses every choice with unit lines, a command and what it printed or a "Not run" line, a reason
  for each adjacent pair, and a separability statement.

No cross-vendor review of this revision has been received, so none is folded in, and the document
stays unapproved and non-final.
