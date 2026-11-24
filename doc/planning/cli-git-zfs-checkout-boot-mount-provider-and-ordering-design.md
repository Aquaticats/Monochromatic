# Which unit mounts the checkout, and the ordering the boot gate needs

Design proposal by an agent,
revised a fourth time,
after a same-vendor read of the third revision,
**unapproved and non-final**.
It answers the two parts plan step 19 leaves open
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, "Step 19", "Open items"),
folds the findings of that read,
and measures on one throwaway guest the three things the read showed unmeasured:
the unit text with its stop bound,
this host's unit without the key lines,
and the hazard with the covered entry's flag on.
It designs no wrapper code,
edits nothing under `doc/decision/` or `doc/handover/`,
and applies nothing to any machine.

Every claim in this document carries a label for what it rests on:

- a kept log under this round's `keep/logs/`, named in backticks;
- a journal line, quoted with its Unix timestamp and the boot it belongs to;
- a host read, kept under this round's `keep/`, with the command that printed it;
- a source reading, with the file and the lines;
- `[nothing]`, where a sentence is an inference, a recommendation or an open question.

This round's folder is `~/temp/agent/zfs-step-19-fourth-revision-2026-11-24/`.
"Round one" means `~/temp/agent/zfs-step-19-checkout-boot-mount-provider-and-ordering-2026-11-23/`,
"round two" means `~/temp/agent/zfs-step-19-design-after-its-gate-review-2026-11-24/`,
and "round three" means `~/temp/agent/zfs-step-19-third-revision-2026-11-24/`.
A log named without a round is this round's.
Where a fact rests only on an earlier round's log, the round is named with it.

## What the read of the third revision changed

The read
(`~/temp/agent/gate-review-zfs-step-19-third-revision-2026-11-24/keep/read-of-the-third-step-19-revision.md`,
a same-vendor second pass, not an independent review)
found no high defect, nine medium ones and twelve lows.
Each held when this round checked it against `main`, the kept logs and the sources;
none is answered with a rebuttal,
and each is named here with what became of it.

### The medium findings

**1. Choice V9's option B does not ship either.**
Held: `boot-reconcile` has no step that mounts a dataset under the checkout path.
Its module doc fixes the order as its own checks, then the sweep, then the readiness record
(source reading,
`package/git-policy/cli/src/native/worktree_clone_mount_handler/boot_reconcile_mode.rs`,
the module doc and `run_at`),
and `rg 'zfs mount'` over that folder finds comments only: no call mounts a dataset that is not a
clone.
What round three measured as B is a stand-in:
`sweep-descendants` runs `zfs mount`, echoes its exit status, and its last command is an `echo`,
so the unit exits zero whatever the mount did
(round three's `keep/guest-scripts/10-units.sh`, the `sweep-descendants` body).
Folded in: option B is stated as **a new step inside `boot-reconcile`**,
which needs three things this design now names:
a source for the dataset list,
a rule for what a failed descendant mount does to the readiness record and the exit status,
and the same idempotence on `ExecReload=`.
The document says plainly that **nothing ships until that step or option A's mode lands**,
and "What each part waits on" lists it.

**2. `mount-path-descendant` contradicts a written grammar principle.**
Held: `HandlerMode`'s doc says "a mode is chosen by a leading word and nothing else, and no argument
names a folder to mount or a dataset to change"
(source reading,
`.../handler_mode_arguments.rs:113` to `:115`).
The proposal answered it with `holder-scan`'s `--dataset`,
which only reads a dataset's holders, so it never carried that principle's consent.
The proposed refusal name was also the gate's own name for a different condition
(`checkout-not-the-dataset` means the checkout's mount row is not the checkout dataset),
so the new mode would have reused it for "this dataset's mountpoint is outside the checkout".
Folded in: the conflict is stated where the mode is asked for,
the new refusal gets its own name, `dataset-outside-the-checkout`,
and the "outside the checkout" property is marked unmeasured,
because the stand-in ignores its arguments (same script, the `nested` body).
Idempotence now rests on a measured line rather than only on the source:
round two's `keep/logs/propagation.log:99` holds
`cannot mount 'zfsstep19/home/monochromatic-git-checkout': filesystem already mounted`,
which is the status the mode must not return.

**3. A quote that exists nowhere.**
Held: the sentence "The root-modes design lists the same point as open ('no profile exists yet')"
cited a phrase that appears only in this proposal,
and `rg -i 'profile|apparmor'` over
`doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md` matches nothing.
The same document says "no design owns it yet" in its own list,
so it contradicted itself.
Folded in: the false attribution is gone,
and the AppArmor section says what the host's own readable policy shows instead.

**4. The "as written" unit never booted.**
Held: the text promised `TimeoutStopSec=`, and no state log held one:
`state-as-written.log` prints `TimeoutStartSec=20` and no stop bound,
and no script wrote one (round three's `keep/guest-scripts/11-state.sh`).
Folded in by booting the text with both bounds:
`TimeoutStartSec=20` and `TimeoutStopSec=5`,
green, with the unit as systemd read it (kept logs `state-as-written-stopsec.log`,
`evidence-as-written-stopsec.log`, boot `5b8c1894-4729-48ba-a307-ac0982ba40da`).
The only substitutions are still the placeholders, the stand-in program's name and its argument,
and the document lists them.

**5. The hazard arms ran with the flag off, which the design never ships.**
Held: `lsattr` in both of round three's hazard logs prints `----------------------`
(round three's `state-hazard-nested-auto.log:5`, `state-hazard-v4.log:5`).
Folded in by running the hazard with the covered entry's flag on, the checkout's mount failing and
the dataset under the path at `canmount=on`: the provider cannot create the mountpoint beneath the
immutable entry, ends `Result=exit-code` with `ExecMainStatus=1`, and the gate's job fails with
`dependency` before its own check runs (kept log `evidence-hazard-flag-on.log`, boot
`fcfae077-91df-49bf-bced-ff085a27b7ca`).
So the hazard the earlier rounds measured needs the flag **off**, and the state this design installs
converts it into a provider failure.
The earlier rounds' two arms are kept and labelled for the state they ran.

**6. V8's option E left the consumers' ordering in place.**
Held: the per-dataset unit and both account consumers carry
`Before=systemd-user-sessions.service`
(round three's `keep/guest-scripts/10-units.sh`, the three unit bodies; and
`state-as-written.log:79` for the per-dataset unit),
so dropping the gate's line alone still holds logins through them.
Folded in: option E now drops the line from the consumers and the per-dataset unit too,
says which of those it must drop, and is labelled not separable from V6 and V9.

**7. The hold arithmetic was wrong, and one measured number was an artifact.**
Held: in `evidence-bound.log` the gate started at `1795507884`-side wall clock
`Starting reconcile-…` at `1795507863.398217`,
while the same boot's own evidence file prints the hung pass at uptime `6.39` and the stop pass at
uptime `26.57`, a monotonic difference of 20.18 s.
systemd's timer is monotonic, so the bound fired at 20.18 s and the wall clock stepped by about a
second during the boot.
From the timeout line to `Failed with result 'timeout'` is 17 ms
(`1795507884.711975` to `1795507884.728790`), not "about a second".
Folded in: "What a hung start costs" quotes monotonic time, the 17 ms, and restates the worst case
as the start bound plus the stop bound,
with the probe's own deadline as the pre-phase's bound and a second start bound named as the
pre-phase's ceiling if anything ever puts an unbounded program there.
This round measures the pair in one boot: a 20 s start bound, a 3 s stop bound and a stop pass that
would have slept ten minutes held logins 23.54 s, and the stop bound cut the pass 3.25 s after the
start bound fired (kept log `evidence-bound-stop.log`, boot
`ad2d3287-25f8-4086-819a-3ac4320671ea`).

**8. The unit text's key lines are unconditional, which matters on this host.**
Held: this host has no key-loading unit at all (F5),
and this document's own rule says a unit that requires a unit the boot does not install stays
`inactive` with `Result=success` and no journal line (F31).
Round three's no-key arm booted a gate without those two lines and the document never said so
(round three's `state-no-key-unit.log`, the gate body).
Folded in by printing **this host's unit, without the key lines**,
and booting it (kept logs `state-host-shape.log`, `evidence-host-shape.log`,
boot `773296e0-6fea-4ee0-94c1-1e486e063ce4`),
and by adding a `doctor` line that compares the installed unit's key lines with the machine.

**9. The SELinux claim left contradictory data in the cited logs.**
Held: the line "avc count in the audit log this boot" is a whole-file count
(`grep --count 'avc:' /var/log/audit/audit.log`,
round three's `keep/guest-scripts/12-evidence.sh:63`),
and it rose with each boot of the same guest: 17, 20, 23, 27, 27, 27, 31, 35, 39,
while every `ausearch --message AVC,USER_AVC --start boot` printed `<no matches>`.
Folded in by attributing the whole file's records to boots:
this round's evidence step prints every record the log holds and the count that falls inside each
boot's journal window (kept log `evidence-as-written-stopsec.log`, the block
"the boots this log holds").
The result settles it: the arm's own boot holds no record, and every record in the file is
`systemd-getty-generator`'s `dac_override` denial, inside the earlier boot that built the packages.
The earlier rounds' line counted the whole file and labelled it "this boot", which is why it read as
a contradiction.
**One detail of the finding does not hold.** It infers that the records "may belong to earlier boots'
shutdowns (not shown by this evidence), which would include the stop pass".
They do not: all fourteen fall inside the first boot's own window, between 12:02:49 and 12:06:58,
while packages were being installed and systemd reloaded, and the helper that writes them runs on
those reloads.
The arm's boot held four, and its own `daemon-reload` is what wrote them.
Nothing in the file belongs to a shutdown.

### The low findings

- **L1, the propagation service's invocation.** "Every one of them a new invocation" described five
  units while `restart-provider.log` holds four pairs and no line for the propagation service.
  Corrected: four units changed, named one by one, and the propagation service's own id is not in
  that log. The quoted ids are also relabelled as this document's pairing of the log's two blocks,
  because the log prints the ids without their unit names.
- **L2, quotes that were not contiguous.** The L6 fold of round three said no quote was
  elided-looking; several were. Every block in this revision is one contiguous run of one log, and
  where a block is assembled from two sections, it says so beside the block.
- **L3, `1795507717.422`.** Cited and in no log; the per-dataset unit finished at
  `1795507717.419298` (`evidence-as-written.log:116`, round three). Corrected everywhere.
- **L4, unqualified earlier-round logs.** `start-bound.log`, `dependencies-read.log` and
  `propagation-cycle.log` live in round two's folder, not round three's. Each is now named with its
  round.
- **L5, smaller inconsistencies.** Three guests, not two; `mvm-create-guest2b.log` is
  `08:29:12.486Z`; the per-log claim now names the logs that do carry a boot id; the script list
  includes every script.
- **L6, the host key read.** `host-reads.log` (round three) asked
  `zfs get … zpcachyos/ROOT/home`, which does not exist, and the answer is
  `cannot open 'zpcachyos/ROOT/home': dataset does not exist`.
  Corrected with a fresh read of the real dataset,
  `zpcachyos/ROOT/cos/home`, in `keep/host-reads.log` of this round, quoted under F6.
- **L7, V2's option C.** Its cited arm had no edge to the provider at all
  (round two's `evidence-no-provider-edge.log:94`, the unit's own description says so), so it never
  showed "`After=` only", and that boot's gate started 32 ms after the propagation service's start,
  the confound round three's own M7 found.
  Corrected, and the sentence "at boot the ordering edge adds nothing" is now labelled for what
  that arm shows: nothing either way.
- **L8, V2's option B.** "A provider failure leaves the gate to run and refuse" was unmeasured.
  Now labelled unmeasured, and the option's measured half is named exactly
  (round two's `wants-control.log`, the reload after a provider restart).
- **L9, the key units' shape.** Round three's guest key units carry `Before=zfs-mount.service` and
  `[Install] WantedBy=zfs.target` (round three's `units-install.log:31`, `:41`, `:65`, `:75`),
  which is not the generator's shape (F39).
  Corrected where the guest's units are described, and the failure arm's ordering is stated as what
  it is: the provider started after the gate's job had already failed.
- **L10, examples.** Every option now carries unit lines, a command and what it printed, or says
  which of the three it lacks and why. V7 gains a run.
- **L11, style.** No bare dates, no "the earlier text" pointers.
- **L12, three omissions.** The hook name is a source constant
  (`ZEDLET_INSTALL_PATH`), not something a record fixes;
  the suggested plan edits say which of them depend on open choices;
  and `zfs-zed.service` is `disabled` and `inactive` here (F3), so the hook cannot run at all today,
  which is now on the list of what the hook waits on and in V7.

## What it ran, and how to read this document

One throwaway guest on Fedora 43 Cloud Edition, `zfs-step-19-fourth-2026-11-24`, created and
destroyed through `mvm --backend libvirt` by the name its own create printed: systemd
`258.11-1.fc43`, kernel `6.17.1-300.fc43.x86_64`, OpenZFS `2.4.4-1` built from the release tarball
pinned at sha256 `2a3c70d55a37cc71618a95a60e81ad66530201eb118d37741dc92efcf848c8b1`, SELinux enforcing.
It booted five times: the first for the build and the fixture, then four unattended reboots, one per
arm.
One guest was enough where round three needed three, because the facts that move with a systemd
version were measured there on 262.2 and this round adds no new one.

- `prepare.log`, `fixture.log`, `units-install.log`: the guest's versions, its file-backed pool and
  datasets, and every unit and drop-in as systemd reads them.
- `state-as-written-stopsec.log`, `evidence-as-written-stopsec.log`: the unit text with both bounds,
  and the green boot of it.
- `state-host-shape.log`, `evidence-host-shape.log`: this host's shape, no key-loading unit and no key
  lines, with the gate's own refusal.
- `state-hazard-flag-on.log`, `evidence-hazard-flag-on.log`: the hazard with the covered entry's flag
  on, whose result is the provider's failure.
- `state-bound-stop.log`, `evidence-bound-stop.log`: a hung start and a hung stop pass, bounded.
- `timeouts-read-back.log`: the two bounds and the stop-failure mode as systemd read them.
- `mvm-create-guest.log`, `mvm-destroy-guest.log`: the guest's own create and destroy records,
  through `mvm --backend libvirt`, by the name the create printed.

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
  (round three's `evidence-no-key-unit.log` and this round's `evidence-host-shape.log`).
  Without that property the provider says a great deal, and the next two bullets are why.
- With a dataset under the checkout path at `canmount=on` and the covered entry's flag **on**, the
  provider fails the whole boot's mount: it cannot create the mountpoint inside the immutable entry,
  `zfs-mount.service` ends `Result=exit-code`, `ExecMainStatus=1`, and the gate's job fails with
  `dependency` before its own check runs
  (kept log `evidence-hazard-flag-on.log`, boot `fcfae077-91df-49bf-bced-ff085a27b7ca`).
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
  (this round's `evidence-host-shape.log`, round three's `evidence-no-key-unit.log`).
- Never require a mount unit systemd learned from the mount table.
  Measured at a boot on systemd 258 and on 262.2, and again with the provider also required and both
  drop-ins absent: the requiring unit's start job leaves no journal line at all and the unit stays
  `inactive` with `Result=success`
  (round three's `evidence-learned-unit-only.log`, `boot-arm-learned-unit.log`,
  `boot-arm-learned-with-provider.log`).
- Order with `RequiresMountsFor=<checkout path>` rather than by hand. One line put `After=` on the
  root mount and the home dataset's mount unit, and `Requires=` only on the fragmented root mount
  (this round's `state-as-written-stopsec.log`; round three's `systemd-262-arms.log` arm G).
- The gate's ordering before `systemd-user-sessions.service` is real, and its cost is two bounds:
  a start's bound is per phase, a reload shares it, and the stop pass runs inside the start job's
  failure path, where `TimeoutStopSec=` bounds it.
  Measured in one boot: a 20 s start bound with a 3 s stop bound held logins 23.54 s
  (kept log `evidence-bound-stop.log`, boot `ad2d3287-25f8-4086-819a-3ac4320671ea`).
- The boot unit text as this document writes it, both bounds included, boots green
  (kept log `evidence-as-written-stopsec.log`, boot `5b8c1894-4729-48ba-a307-ac0982ba40da`),
  and this host's shape of it, with no key lines, boots too (kept log `evidence-host-shape.log`).
- A consumer that requires the per-dataset unit sees that dataset mounted; one that names a unit the
  boot does not install gets nothing at all, silently
  (this round's `evidence-as-written-stopsec.log`, round three's `evidence-v9b.log`).
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

Measured, this host's shape (kept log `evidence-host-shape.log`, boot
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
  `checkout-not-the-dataset` (kept log `evidence-host-shape.log`).
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
and `<the measured ceiling of one stop pass>`, and choice V8 settles what ships meanwhile.

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
TimeoutStopFailureMode=terminate
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
and it ran on this round's host-shape boot, where the gate refused the checkout by its own rule
(kept logs `state-host-shape.log`, `evidence-host-shape.log`):

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
TimeoutStopFailureMode=terminate
TimeoutStopSec=<the measured ceiling of one stop pass>
ExecStartPre=-<root-file> <root-words> start-probe
ExecStart=<root-file> <root-words> boot-reconcile
ExecReload=<root-file> <root-words> boot-reconcile
ExecStopPost=-<root-file> <root-words> shutdown-unmount

[Install]
WantedBy=multi-user.target
```

This text, with only the placeholders filled at this round's provisional values
(`TimeoutStartSec=20`, `TimeoutStopSec=5`), is what booted green
(kept logs `evidence-as-written-stopsec.log`, boot `5b8c1894-4729-48ba-a307-ac0982ba40da`;
the unit as systemd read it is printed in `state-as-written-stopsec.log`),
with two substitutions this design makes for the stand-ins:
the stand-in program's name stands where `<root-file> <root-words>` stands, and its argument is the
mode word.
That boot's order, one contiguous run of the journal:

```text
1795522041.162053 zfs-step-19-fourth-2026-11-24 systemd[1]: Starting checkout-propagation.service - Ask the boot unit to re-verify the checkout's private propagation...
1795522041.170076 zfs-step-19-fourth-2026-11-24 systemd[1]: Starting reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts...
1795522041.184678 zfs-step-19-fourth-2026-11-24 systemd[1]: Finished checkout-propagation.service - Ask the boot unit to re-verify the checkout's private propagation.
1795522041.207146 zfs-step-19-fourth-2026-11-24 systemd[1]: Finished reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts.
1795522041.216007 zfs-step-19-fourth-2026-11-24 systemd[1]: Starting git-worktree-nested-output-mount.service - Mount a dataset whose mountpoint lies inside the verified checkout...
1795522041.247040 zfs-step-19-fourth-2026-11-24 systemd[1]: Finished git-worktree-nested-output-mount.service - Mount a dataset whose mountpoint lies inside the verified checkout.
1795522041.249253 zfs-step-19-fourth-2026-11-24 systemd[1]: Starting git-vm-builder-standin.service - A vm-builder service of the account that needs the dataset under the checkout...
1795522041.585504 zfs-step-19-fourth-2026-11-24 systemd[1]: Starting systemd-user-sessions.service - Permit User Sessions...
```

(The account consumer's own two lines and the vm-builder service's "Finished" line fall between
these ones and are omitted here; the log holds them at `:114`, `:116` and `:119`.)

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
  own refusal (kept log `evidence-hazard-flag-on.log`, quoted under "The failure controls").
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
  learned mount unit, and `Requires=` only on `-.mount`, the one of them with a fragment
  (kept log `state-as-written-stopsec.log`, "the dependencies systemd derived":

  ```text
  Requires=zfs-mount.service sysinit.target system.slice "zfs-load-key@zfsstep19-home-monochromatic\\x2dgit\\x2dcheckout.service" -.mount
  After=zfs-mount.service zfs-import.target system.slice basic.target systemd-journald.socket -.mount "srv-zfs\\x2dstep\\x2d19\\x2dfixture-home.mount" "zfs-load-key@zfsstep19-home-monochromatic\\x2dgit\\x2dcheckout.service" sysinit.target
  ```

  and round three's `systemd-262-arms.log` arm G for the same line on 262.2).
  At the moment the unit loads, the checkout's own learned unit may not exist yet, so that edge
  orders the stop half rather than the first start; what orders the first start is the provider's
  requirement and `zfs-import.target`.
- `Before=systemd-user-sessions.service`.
  What makes the checkout ready before an account session, and the line whose cost "What a hung start
  costs" measures. Measured in the green boot: the gate finished at `1795522041.207146` and the
  sessions unit started at `1795522041.585504`, after the per-dataset unit at `1795522041.247040`.
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
  What the pair buys is measured at provisional values: `TimeoutStartSec=20` on a hung start produced
  `start operation timed out. Terminating.`, then the stop pass, then `Failed with result 'timeout'`,
  then sessions (round three's `evidence-bound.log`); a `TimeoutStopSec=3` killed a stop pass that
  would have slept ten minutes, 3.25 s after the start bound fired, and the whole login hold was
  23.54 s for a 20 s start bound (kept log `evidence-bound-stop.log`, quoted under
  "What a hung start costs"). Choice V8 ranks the alternatives and says what ships while the values
  are missing.
- `TimeoutStopFailureMode=terminate`.
  New here, and it exists because of a measured difference between machines: with the default,
  a machine whose packaging sets `TimeoutStopFailureMode=abort` sends SIGABRT to the stop pass and
  leaves a coredump. Fedora 43 does exactly that, and the bound-stop arm's journal holds
  `State 'stop-post' timed out. Aborting.`, two `Killing process … with signal SIGABRT` lines and two
  `systemd-coredump` dumps (kept log `evidence-bound-stop.log`).
  This host's own systemd has no `service.d/10-timeout-abort.conf` (`ls` of both `service.d`
  folders in `keep/host-reads.log`), so the line changes nothing here and makes other machines
  behave the same way.
- `ExecStartPre=-<root-file> <root-words> start-probe`.
  The start-probe design's placement, adopted here because plan step 19 installs the unit: one root
  process per boot, in the service manager's own child, before the reconcile.
  The probe's own deadline is built and bounds the probe: `PROBE_DEADLINE` is 350 ms with a 231 ms
  margin over the slowest measured worker phase
  (source reading, `package/git-policy/cli/src/native/start_probe/probe_deadline.rs`, the two
  constants and the measurement its doc comment cites). So the earlier deferral is gone and the line
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

(source reading, `handler_mode_arguments.rs:113` to `:115`, the `HandlerMode` doc).
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

For this host that is one unit, for `zpcachyos/nosnap/vm-builder-output`.
The measured stand-in ran the same call the mode would make, an explicit `zfs mount <dataset>`, after
checking that the checkout's own mount row is the checkout dataset, and it is idempotent
(kept log `evidence-as-written-stopsec.log`):

```text
SWEEP checkout-row=zfsstep19/home/monochromatic-git-checkout zfs
NESTED start 2026-11-24T12:07:21.221Z
NESTED mounted=zfsstep19/nosnap/vm-builder-output 2026-11-24T12:07:21.245Z
SESSION-DESCENDANT mounted=zfsstep19/nosnap/vm-builder-output
```

That stand-in **never exercises the refusal**: it ignores its arguments and reads the checkout's row
by its own fixed path, so nothing measured shows a dataset outside the checkout being refused
(the body is round three's `keep/guest-scripts/10-units.sh`, the `nested` function).
The refusal is a requirement with a source reading behind it and no arm.

The explicit form matters: `zfs mount -a` skips a `noauto` dataset and skips an unavailable key
silently, while the explicit form refuses with a named message (source reading,
`cmd/zfs/zfs_main.c`, `share_mount_one`'s `canmount` table and key branch, both gated on `explicit`;
measured for the key case in "The route-owned alternative").
`Before=systemd-user-sessions.service` is what closes the window the earlier rounds measured: without
it round one's `noauto` boot finished the descendant unit after the sessions unit
(round one's `keep/guest-logs/journals-per-boot.log`, `Finished systemd-user-sessions.service` at
`1795486959.129715`, `Finished git-worktree-nested-output-mount.service` at `1795486959.180427`);
with it, this round's green boot is gate, then descendant, then sessions
(kept log `evidence-as-written-stopsec.log`, quoted in "Why each line").

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

**So nothing ships while both are missing**, and the difference between the options is what each
would deliver once its piece lands, not which one exists.

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
  service started at `1795522041.162053` and the gate at `1795522041.170076`, because the learned
  unit's activation submits the upheld service immediately (kept log
  `evidence-as-written-stopsec.log`).

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
  `px` is the discrete-profile transition mode, so these two run in their own profiles:
  `/etc/apparmor.d/zfs` and `/etc/apparmor.d/mount-zfs` exist here, and the first carries
  `mount fstype=zfs, umount fstype=zfs,` with `capability sys_admin` and
  `capability dac_read_search`.
- `rix` on `/{usr/,}{local/,}lib/zfs-linux/zed.d/*.sh` at line 38: the only rule that grants exec to
  a zedlet.

There is no plain `x`, no `px` of its own, no `ux` and no `cx` anywhere in the profile, so a program
ZED starts either stays in `zed` or lands in a profile it names.

Its capabilities are one line: `capability sys_admin,` (line 15). No other capability.
Its write rules are these, and it does have them for `/run`:

```text
@{run}/zed.pid rwlk                     line 43
@{run}/zed.state rwlk                   line 44
@{run}/zfs-list.cache@* rw              line 45
/etc/zfs/zfs-list.cache/{,*} rwk        line 41
owner @{tmp}/tmp.* rw                   line 47
@{sys}/module/zfs/parameters/zfs_zevent_len_max rw   line 52
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

The live list cannot be read unprivileged, and the earlier revision offered "the `zed` profile is not
loaded" as a live possibility. The host's own files make that near-impossible here:

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

### The two shapes, ranked by what each delivers

**Shape A, rules added to the profile through `/etc/apparmor.d/local/zed`.**
Every grant the hook needs lands in the `zed` profile: an exec rule for the installed file, `mount`
and `umount`, `capability dac_read_search` and possibly the `ptrace` read grant, and writable paths for
the slot, the record and the pass's own state.

What it delivers: one profile to load, one file to read, no transition to get wrong, and a start that
cannot fail for a missing target profile.

What it costs: every one of those grants also accrues to ZED itself and to every packaged zedlet that
runs `ix` into it, including `mount` over any path and a capability that reaches every `0700`
directory on the machine. The profile's own audit identity then names `zed` for a mount that ZED's
notification code never performs.

**Shape B, a dedicated profile for the installed file, reached with `px`.**
The hook gets its own ruleset, and ZED's profile keeps the narrower set it has.

What it delivers: least privilege for the only program in this route that mounts and writes as root;
a separate audit label, so a denial names the hook and not ZED; its own mode, so an experiment can put
the hook in complain while ZED stays enforcing; and a permission set that can be reviewed without
reading ZED's.

What it costs: a second profile to install and keep in step with the wrapper's real behaviour; a name
that whoever installs the wrapper has to know, which is exactly what the owner's recorded preference
rejects; and a `px` that fails the exec closed if the target profile is not loaded, so a half-done
install is a hook that cannot run while the boot gate still can.

**Ranking: B > A**, because the difference is what each delivers for a root program that mounts: B
confines it to its own rules and its own audit identity while A widens a notification daemon's
permissions to cover it, and the cost B carries is maintenance, not capability.

**The owner's recorded preference stands beside that ranking and is theirs to weigh.** On
2026-11-23 the owner answered a rule-or-profile question with "No, please do rules for everything
instead of a dedicated profile that agents would have to remember to use", and
`/etc/apparmor/full-system-policy.md` records it under "Local workload rules".
That preference is a cost estimate about the people and agents who maintain this machine; it is not
a statement about what each shape delivers, and this design does not treat it as one.

This design does not choose between the two. It lists the grant under what the hook waits on, and
`doctor` reports what an unprivileged reader can see: whether `/etc/apparmor.d/zed` carries a
`flags=` line, whether `/etc/apparmor.d/local/zed` exists, whether `apparmor.service` is enabled and
active, what `/etc/apparmor/modes` says, and whether the profile's zedlet rule names a folder that
exists. It cannot report the loaded mode, and it says so rather than guessing.

Two facts worth keeping beside the reading:

- The host's own `zfs` and `mount-zfs` profiles do allow `mount fstype=zfs` and `umount fstype=zfs`
  for the programs they name (`keep/apparmor-and-unit-reads.log`), which is how a person's
  `zfs mount` works. That does not help the hook: the hook's own process is not the `zfs` program, and
  only a `px` transition would put it there.
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
(kept log `evidence-hazard-flag-on.log`).

## The failure controls

Every failure control of the three rounds, each with its own kept log.
Four ran this round; the earlier rounds' arms are kept and named with their round.

**This round, on the guest of "What it ran" — four arms, all with SELinux enforcing.**

- **The unit as written, both bounds included: passes.** Kept log
  `evidence-as-written-stopsec.log`, boot `5b8c1894-4729-48ba-a307-ac0982ba40da`.
  The probe ran, the gate read the checkout row, the flag and the private propagation, attached the
  slot, the per-dataset unit mounted its dataset, the account consumer saw the checkout and the slot,
  the second account consumer saw the dataset, and sessions started last:
  gate `1795522041.170076` to `1795522041.207146`, per-dataset unit `1795522041.216007` to
  `1795522041.247040`, its consumer `1795522041.249253` to `1795522041.261391`, sessions
  `1795522041.585504`.
  The unit systemd read is in `state-as-written-stopsec.log`, and `TimeoutStopSec=5` is one of its
  lines.
- **This host's shape: no key-loading unit for the checkout, its key unavailable, no key lines in the
  unit: the gate refuses and the boot fails closed.** Kept logs `state-host-shape.log`,
  `evidence-host-shape.log`, boot `773296e0-6fea-4ee0-94c1-1e486e063ce4`.
  The unit systemd read has no `Requires=` and no `After=` for a key unit, the checkout's key unit is
  `not-found`, and the gate refused `checkout-not-the-dataset` at
  `SWEEP refused=checkout-not-the-dataset 2026-11-24T12:09:24.209Z`, with sessions at
  `1795522164.523793`.
- **The hazard with the covered entry's flag on, `canmount=on`, and the checkout's mount failing: the
  provider cannot create the mountpoint and fails the boot's mount.** Kept log
  `evidence-hazard-flag-on.log`, boot `fcfae077-91df-49bf-bced-ff085a27b7ca`, one contiguous run:

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

  The home underlay's inventory in that boot is the three directories it started with: no `package`
  folder was created, and no dataset was mounted under the checkout.
- **A hung start with both bounds: a 20 s start bound and a 3 s stop bound hold logins 23.54 s.**
  Kept log `evidence-bound-stop.log`, boot `ad2d3287-25f8-4086-819a-3ac4320671ea`, quoted under
  "What a hung start costs".

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
(round three's three failure logs, and this round's `evidence-host-shape.log`,
`evidence-hazard-flag-on.log`).
That is why the design puts `Requires=` on the services of the account that need the checkout, which
is choice V6.

**SELinux was enforcing in every arm, and the audit log is attributed rather than counted.**
This round's evidence step reads three things: `ausearch --message AVC,USER_AVC --start boot`, every
record the audit file holds, and the count that falls inside each boot's journal window.
The result in the green arm (kept log `evidence-as-written-stopsec.log`, "the boots this log holds"):

```text
boot 0d127f12a43d4a1dbea9f0d39f396d62 from 11/24/2026 08:02:38 to 11/24/2026 08:02:44 avc-records=0
boot 355682a743e04408ac175bc984997c9e from 11/24/2026 12:00:42 to 11/24/2026 12:07:14 avc-records=14
boot 5b8c1894472948baa307ac0982ba40da from 11/24/2026 12:07:16 to 11/24/2026 12:09:01 avc-records=0
```

So the arm's own boot has no denial, and the 14 records the file holds all belong to the earlier
boot that built the packages and the pool.
Every one of them is the same denial, and none names a unit of this design:

```text
type=AVC msg=audit(1795521769.491:196): avc:  denied  { dac_override } for  pid=1176 comm="systemd-getty-g" capability=1  scontext=system_u:system_r:systemd_getty_generator_t:s0 tcontext=system_u:system_r:systemd_getty_generator_t:s0 tclass=capability permissive=0
```

The previous rounds' "avc count in the audit log this boot" was this same cumulative file count under
a label that said "this boot", and that label is why it read as a contradiction: the count rose with
each boot of the same guest while `--start boot` stayed empty, because the records belong to the
boots before the arm's.
Nothing in them names this design's programs, and the guest's stand-ins ran in
`unconfined_service_t` (kept log `evidence-as-written-stopsec.log`, the `selinux` block), which is
`mvm`'s own drop-in for a confining policy
(`doc/troubleshooting/mvm-fedora-guest-confined-by-selinux.md`).

## What a hung start costs

The costs are these numbers, each measured: the phases of one start on systemd 262.2 (the host's own
version) in round three's `systemd-262-arms.log`, the stop bound on the host's own user manager in
round three's `keep/host-user-manager-stopsec.log`, and two boots, round three's and this round's.

The default: a oneshot has no start timeout.
`systemctl show` for a unit with no `TimeoutStartSec=` reports `TimeoutStartUSec=infinity`
(round three's `systemd-262-arms.log` arm A, and round two's `start-bound.log`), so a hung reconcile
ordered before the sessions unit holds every login with no limit.

The bound is **per phase**, not for the start as a whole:

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

The **stop pass extends the hold**, because it runs inside the start job's failure path and the
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
The stop pass is itself bounded by `TimeoutStopSec=` (default 90 s): on the host's own user manager a
two second `TimeoutStopSec=` killed a five second `ExecStopPost=` after 2.07 s, with
`State 'stop-post' timed out. Terminating.` and `Control process exited, code=killed, status=15/TERM`
(round three's `keep/host-user-manager-stopsec.log`).

The bound at work in a boot, read after it fired (round three's `evidence-bound.log`,
`evidence-bound-late.log`, `TimeoutStartUSec=20s`, `Result=timeout`).
The wall clock and uptime in that boot disagree, and systemd's timer is monotonic, so the uptimes are
the ones that say when the bound fired:

```text
1795507863.398217 zfs-step-19-third-2026-11-24 systemd[1]: Starting reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts...
1795507884.711975 zfs-step-19-third-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: start operation timed out. Terminating.
1795507884.713096 zfs-step-19-third-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Main process exited, code=killed, status=15/TERM
1795507884.728790 zfs-step-19-third-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Failed with result 'timeout'.
1795507884.732255 zfs-step-19-third-2026-11-24 systemd[1]: Starting systemd-user-sessions.service - Permit User Sessions...
1795507884.751115 zfs-step-19-third-2026-11-24 systemd[1]: Finished systemd-user-sessions.service - Permit User Sessions.
```

with, from the same boot's evidence file, `SWEEP hung … uptime=6.39` and
`SWEEP stopped … uptime=26.57`.
So: 20.18 s of monotonic time from the pass's start to the stop pass's own line, against 21.31 s of
wall clock, because the clock stepped by about a second during that boot; and 17 ms
(`1795507884.711975` to `1795507884.728790`) from the timeout line to the unit's failure, with the
stop pass finished inside it.
The login hold in that boot is therefore 20.2 s of monotonic time and 21.33 s of wall clock, and the
second is the clock step, not the stop pass.

This round measured the pair in one boot, with a stop pass that would have slept ten minutes
(kept log `evidence-bound-stop.log`, boot `ad2d3287-25f8-4086-819a-3ac4320671ea`):

```text
1795522327.244840 zfs-step-19-fourth-2026-11-24 systemd[1]: Starting reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts...
1795522347.454452 zfs-step-19-fourth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: start operation timed out. Terminating.
1795522347.455101 zfs-step-19-fourth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Main process exited, code=killed, status=15/TERM
1795522350.705097 zfs-step-19-fourth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: State 'stop-post' timed out. Aborting.
1795522350.779011 zfs-step-19-fourth-2026-11-24 systemd[1]: reconcile-git-worktree-clone-mounts.service: Failed with result 'timeout'.
1795522350.788178 zfs-step-19-fourth-2026-11-24 systemd[1]: Starting systemd-user-sessions.service - Permit User Sessions...
```

20.21 s to the timeout, 3.25 s more to the stop bound, 23.54 s to the sessions unit; and the evidence
file's own two lines agree (`SWEEP hung … uptime=6.24`, `SWEEP stop-hung … uptime=26.44`).
`systemctl show` on that boot reports `TimeoutStartUSec=20s`, `TimeoutStopUSec=3s` and, from Fedora's
own drop-in, `TimeoutStopFailureMode=abort` (kept log `timeouts-read-back.log`), which is why the stop
bound ends in SIGABRT and two coredumps rather than a clean kill.

So the worst case a reader should carry is:

```text
the probe's own deadline (0.35 s) + TimeoutStartSec + TimeoutStopSec
```

and, on a unit whose `ExecStartPre=` is unbounded rather than the probe:

```text
TimeoutStartSec (the pre phase) + TimeoutStartSec (the main phase) + TimeoutStopSec
```

For a shipped 20 s start bound with the default 90 s stop bound, that is about 110 s of held login
with the probe in place, and about 130 s if an unbounded program ever stands where the probe stands.
That arithmetic is why the unit text carries `TimeoutStopSec=` as well as `TimeoutStartSec=`, and why
choice V8 ranks the options by the hold they leave.

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

- `prepare.log`: the guest's versions, the package install, the pinned tarball's sha256 and the build's
  stream. The stream is truncated by the guest agent, so the same script's read-back after the build is
  kept separately.
- `prepare-readback.log`: the same script, second run: Fedora Linux 43, systemd `258.11-1.fc43`,
  kernel `6.17.1-300.fc43.x86_64`, SELinux enforcing, module `2.4.4-1`, userspace `zfs-2.4.4-1`, and
  the source install's unit list with its `zfs-load-key.service -> /dev/null`.
- `fixture.log`: the pool, the datasets, the cache file the import unit reads, the immutable entry, and
  `zfs-mount-a-exit=0` with the checkout and the dataset under it mounted. The script then stops at a
  busy unmount, which `03-fix-fixture.sh` completes; the fixture's state is read back in
  `state-as-written-stopsec.log`.
- `units-install.log`: every unit and drop-in as systemd reads them, and the two escaped names.
- `state-as-written-stopsec.log`: the unit as systemd read it, with both bounds, and the dependencies it
  derived.
- `evidence-as-written-stopsec.log`: the green boot of the unit text as written, with the order gate,
  per-dataset unit, its consumer, the account consumer, sessions; and the audit attribution.
- `state-host-shape.log`, `evidence-host-shape.log`: this host's shape, no key unit and no key lines,
  the gate's refusal with the provider green.
- `state-hazard-flag-on.log`, `evidence-hazard-flag-on.log`: the hazard with the flag on, whose
  result is the provider's failure.
- `state-bound-stop.log`, `evidence-bound-stop.log`: the start bound and the stop bound in one boot.
- `timeouts-read-back.log`: `TimeoutStartUSec=20s`, `TimeoutStopUSec=3s`, `TimeoutStopFailureMode=abort`.
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
- `mvm-create-guest.log`, `mvm-destroy-guest.log`, `mvm-list-after-destroy.log`: this round's guest's
  create and destroy records.

Some logs named in this revision are not this round's, and each is named with its round:
`start-bound.log`, `dependencies-read.log`, `propagation-cycle.log` and `propagation.log` are round
two's; `restart-provider.log`, `systemd-262-arms.log`, `upholds-arm.log`, `escaping-arm.log`,
`units-install.log` and every `evidence-*` log of that round are round three's.

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
  all, and round three's were written by hand.
- Not ZED. No arm ran `zfs-zed.service`, and the hook is a text this design proposes.
- Not AppArmor. The guest runs Fedora's SELinux, not this host's AppArmor, so the hook's own profile
  question is a reading here, not a run.
- Not this host's key path. The guest's checkout is an encryption root of its own with a file key;
  this host's inherits the pool's passphrase, which the host-shape arm models but does not reproduce
  in its prompt.
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
- Not either choice V9 option: neither the mode nor the pass's step exists.

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

The command and what it printed, with the checkout absent (kept log `evidence-host-shape.log`, boot
`773296e0-6fea-4ee0-94c1-1e486e063ce4`):

```text
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
It also carries a failure the previous revision did not know: where a dataset under the checkout path
cannot have its mountpoint created, the provider fails and this unit's requirement turns that into a
`dependency` failure before its own check runs (kept log `evidence-hazard-flag-on.log`).

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

What it costs: a `canmount=noauto` on the checkout dataset, which is an owner's property change; a
missing property makes the provider mount the checkout and this unit fail with
`filesystem already mounted` (source reading, `zfs_main.c`, the `OP_MOUNT` branch; measured on the
same message, round two's `propagation.log:99`), a red boot for a state `doctor` could have reported;
its `ExecStop=` unmounts the checkout independently of the provider's stop, which the design's stop
ordering does not want; and with the key requirement in place its failure is the same dependency
failure as option A's (round two's `evidence-route-owned-key-removed.log`).

#### Option C: keep the conditional

A unit whose `Requires=` names whichever provider the machine has.

```ini
[Unit]
Requires=<whichever provider this machine has>
After=<the same>
```

What it delivers: nothing to settle here.

What it costs: a conditional in a unit file is not a thing systemd does; a reader cannot audit which
prerequisite a given boot had; and step 19's own open item asks this design to settle it.

Not run, because there is nothing to run: the text above is not a unit systemd accepts.

#### Ranking of V1

Ranking: A > B > C.

A over B: B's whole measured advantage, a status that names the checkout dataset, appears only in the
variant that drops the key unit's requirement, and it costs a property change on the checkout dataset
plus a second mount actor whose own stop unmounts the checkout; A reaches the same fail-closed boot
through the key unit where one exists and through the gate's refusal where none does, both measured.

B over C: B gives one auditable prerequisite with a status of its own, while C leaves the reader to
guess which unit a machine had.

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
failure before the gate's own named refusal can run (kept log `evidence-hazard-flag-on.log`): a boot
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

What it costs: nothing pulls the provider in, nothing re-runs the gate after a provider restart, and
the pull-in caveat in "Why each line" applies: with the module not loaded, the provider is a condition
skip.

#### Ranking of V2

Ranking: B > A > C. **This is a close call, and it is the owner's.**

B over A: both deliver the re-run after a provider restart, one through propagation and one through
the installed drop-in; B delivers it without unmounting every attached clone or restarting every
consumer and descendant, which is measured to be A's price, and B leaves the gate's own named refusal
in place where A replaces it with `dependency`.

A over C: A delivers a re-run and a pull-in, and C delivers neither.

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

The command and what it printed, on this round's guest (kept log
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
elsewhere. `RequiresMountsFor=`'s own expansion on this round's guest names the home dataset's unit,
and it is not called `home.mount` (kept log `state-as-written-stopsec.log`):

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

Three arms measure what the property decides, and this round added the third.

```text
# the flag off, canmount=on (round three's evidence-hazard-nested-auto.log)
d /srv/zfs-step-19-fixture/home/account/Monochromatic/package/vm-builder/cli/output
zfsstep19/nosnap/vm-builder-output  .../Monochromatic/package/vm-builder/cli/output  zfs  shared

# the flag off, canmount=noauto (round three's evidence-hazard-v4.log)
d /srv/zfs-step-19-fixture/home
d /srv/zfs-step-19-fixture/home/account
d /srv/zfs-step-19-fixture/home/account/Monochromatic
zfsstep19/nosnap/vm-builder-output  .../Monochromatic/package/vm-builder/cli/output  noauto  no  available

# the flag ON, canmount=on (this round's evidence-hazard-flag-on.log)
1795522243.192568 ... zfs[818]: cannot mount '/srv/zfs-step-19-fixture/home/account/Monochromatic/package/vm-builder/cli/output': failed to create mountpoint: Operation not permitted
1795522243.193878 ... systemd[1]: zfs-mount.service: Failed with result 'exit-code'.
```

The first two blocks are the logs' own `findmnt` and `zfs list` lines with the paths shortened; the
third is the boot's journal, quoted whole under "The failure controls".

What it delivers: the provider never attempts a descendant, so its own status stays out of the
checkout's story and the gate's named refusal is what a reader sees; the gated unit's explicit
`zfs mount` refuses with a named message rather than skipping; and it is the only way the design's
"gate every path descendant" sentence can hold.

What it costs: a property change on existing datasets, which is the owner's act; a dataset under the
path that setup misses is mounted by the provider again, so `doctor` must read and report it; and a
dataset under the path is no longer mounted by `zfs mount -a` alone, so anything that expected that
must name it.

Step 20's snapshot coverage and the setup runbook are the two other places this property has to be
written down, and neither is changed by this design.

#### Option B: let the provider mount them and have the gate report them

The command that leaves it as it is, and what round three's two hazard logs read back:

```text
$ zfs get -H -o property,value canmount zpcachyos/nosnap/vm-builder-output
on
```

(That line is the shape of the read; round three's `evidence-hazard-nested-auto.log` holds the
dataset's `on` in its `zfs list` line, quoted in this choice's first block.)

What it delivers: no property and no extra units.

What it costs, measured three ways: with the flag off, an account-visible mount of the descendant
created by the provider under the exposed home underlay with no checkout mounted over it; with the
flag on, the same dataset's mountpoint cannot be created, so the provider fails and — because the boot
unit requires it — the gate's job dies of `dependency` instead of refusing with its own reason, and
nothing else the provider would have mounted in that boot is mounted either (this round's
`evidence-hazard-flag-on.log`, the provider's `status=1/FAILURE`).
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

Measured in every green boot: `SWEEP propagation=private`
(kept log `evidence-as-written-stopsec.log`).

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

The journal of this round's green boot, where the consumer that requires the per-dataset unit ran
after it and saw the mount (kept log `evidence-as-written-stopsec.log`):

```text
1795522041.247040 zfs-step-19-fourth-2026-11-24 systemd[1]: Finished git-worktree-nested-output-mount.service - Mount a dataset whose mountpoint lies inside the verified checkout.
1795522041.249253 zfs-step-19-fourth-2026-11-24 systemd[1]: Starting git-vm-builder-standin.service - A vm-builder service of the account that needs the dataset under the checkout...
1795522041.261391 zfs-step-19-fourth-2026-11-24 systemd[1]: Finished git-vm-builder-standin.service - A vm-builder service of the account that needs the dataset under the checkout.
SESSION-DESCENDANT mounted=zfsstep19/nosnap/vm-builder-output
```

What it delivers: a real prerequisite, visible in the journal per service, and independent of login.

What it costs: it protects nothing about login; `boot-reconcile` exits zero when clones fail, so a
clone failure does not reach a consumer through the requirement; the service list is still to be
named; and with V2's option A every consumer restarts when the provider restarts, while with V2's
option B it does not.
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
Measured in this round's host-shape boot: the gate's job failed with its own refusal and sessions
started anyway (kept log `evidence-host-shape.log`, `1795522164.523793`).

What it delivers: nothing to name.

What it costs: measured, sessions start after a failed gate
(this round's `evidence-host-shape.log`, `evidence-hazard-flag-on.log`; round three's
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
- the boot unit's start bound, stop bound, stop-failure mode and their source;
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

### V8. What bounds a start, and how long a login can be held

#### Option A: a sourced `TimeoutStartSec=` and `TimeoutStopSec=`, with the ordering before sessions kept

```ini
[Service]
Type=oneshot
RemainAfterExit=yes
TimeoutStartSec=<the measured ceiling of one reconcile>
TimeoutStopFailureMode=terminate
TimeoutStopSec=<the measured ceiling of one stop pass>
```

What it delivers: the gate's protection is kept and its cost is a stated arithmetic; a hung pass is a
failed unit rather than a stuck boot; and a hung stop pass is cut at the second bound instead of the
default ninety seconds. Measured with provisional values, in one boot: 23.54 s of held login for a 20 s
start bound and a 3 s stop bound, against a stop pass that would have slept ten minutes
(kept log `evidence-bound-stop.log`).

What it costs: no measurement of one reconcile or one stop pass exists yet, so both values are step
22's or the owner's to supply; a bound that is too tight kills a reconcile mid-attach and then runs the
stop pass; and on a machine whose packaging sets `TimeoutStopFailureMode=abort` a timed-out stop pass is
killed with SIGABRT and leaves a coredump, which is why the option writes `terminate` beside it.

#### Option B: provisional fixed bounds by the owner's decision

```ini
TimeoutStartSec=5min
TimeoutStopSec=1min
```

What it delivers: a bounded hold now, with two numbers to revisit, and the journal records why they
are the values they are.

What it costs: unsourced numbers; the project's rule for shipped limits allows them only with an owner
decision beside them, which is what this option records; and the hold it permits is the largest of any
bounded option, up to the probe's deadline plus five minutes plus one minute.
Not run.

#### Option C: no bound and the ordering line withheld until bounds exist

```ini
# no TimeoutStartSec=
# no Before=systemd-user-sessions.service
```

What it delivers: no unsourced number and no login hold at all.

What it costs: while the bound is missing, nothing makes the checkout ready before a session, which is
the unit's purpose.

Not run: the two lines it removes are what every green arm has, and the failing arms measure what
their absence costs a session.

#### Option D: no bounds and the ordering kept

The unit with no `TimeoutStartSec=`, whose manager default is `infinity`
(round three's `systemd-262-arms.log` arm A, and round two's `start-bound.log`):

```text
TimeoutStartUSec=infinity
```

What it delivers: no number to defend, and no reconcile killed mid-attach.

What it costs: a hung reconcile holds every login with no limit, and the stop pass's own default is
90 s on top; a boot nobody can log in to, cleared only by a person.

#### Option E: the ordering line dropped, and the consumers carry the readiness

No `Before=systemd-user-sessions.service` on the boot unit, **and none on the per-dataset unit or on
either account consumer**, and every consumer that needs the checkout requires the unit that provides
it (V6's option A).

```ini
# the boot unit, the per-dataset unit and every consumer
# Before=systemd-user-sessions.service   <- removed from all of them
```

What it delivers: no login hold at all, whatever the pass does; the readiness reaches exactly the
services that name it.

What it costs: a session can start with the checkout absent, measured as the shape of every failing
arm (this round's `evidence-host-shape.log`, `evidence-hazard-flag-on.log`), so an interactive account
sees no checkout until the services that need it fail; and dropping the line from the consumers is
what makes E work at all, so E is **not separable from V6 and V9**: an option that drops one line and
leaves the others is a hold through the units that kept it.
The boot unit's own `Before=umount.target` ordering is the only thing left stating the intent.

Not run: E is A's line removed from four units, and the failing arms show sessions starting with the
checkout absent, which is the state E ships.

#### Ranking of V8

Ranking: A > B > E > C > D. **A is the target; B over E is the owner's call.**

A over B: the same protection with evidence instead of a promise, which is what the project's rule
asks for beside a shipped limit.

B over E: both bound what can go wrong, and B keeps the login protection the unit exists for while E
gives it up; B's cost is that its numbers are decisions rather than measurements.
The pair is a value judgment about a boot nobody can log in to against a session without a checkout,
and it is the owner's.

E over C: E has no unbounded hold and keeps per-consumer readiness; C has neither.

C over D: D ships an unbounded login hold together with the ordering line, the one combination a boot
unit ordered before the sessions must not have.

What ships while it is open: B's numbers at step 22's provisional values, with A's lines written and
the values left to be measured, exactly as this round booted them (`TimeoutStartSec=20`,
`TimeoutStopSec=5`).

Separable: A, B, C and D are separable from every other choice; E is not.

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

The command and what it printed, in this round's green boot (kept log
`evidence-as-written-stopsec.log`):

```text
$ systemctl --no-ask-password show --property=Id --property=ActiveState --property=CanReload --property=Result git-worktree-nested-output-mount.service
Id=git-worktree-nested-output-mount.service
ActiveState=active
CanReload=no
Result=success

$ ... the same boot's journal and evidence file
1795522041.216007 ... Starting git-worktree-nested-output-mount.service - Mount a dataset whose mountpoint lies inside the verified checkout...
1795522041.247040 ... Finished git-worktree-nested-output-mount.service - Mount a dataset whose mountpoint lies inside the verified checkout.
SESSION-DESCENDANT mounted=zfsstep19/nosnap/vm-builder-output
```

What it delivers: the mount after the verified checkout and before sessions, its own journal entry,
a per-dataset failure, and a consumer that requires exactly it and sees it.

What it costs: the mode `mount-path-descendant` does not exist, and its `--dataset` conflicts with the
grammar principle this design quotes under "The units for the datasets under the checkout path", so
this option ships only once the root-modes design admits both; one
unit per dataset, generated by setup and reported by `doctor`; a dataset under the path whose unit
setup forgets is silently unmounted; and one more unit for every new dataset.

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

#### Option C: one unit per dataset running `zfs mount <dataset>` directly

```ini
[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/usr/bin/zfs mount <dataset>
```

What it delivers: the shortest unit text.

What it costs: no verification handshake, so a unit that starts early mounts a descendant under an
unverified checkout, and the design has two mounters instead of one.

Not run: no arm mounts a descendant without the gate's verification, and the explicit call itself is
the one the stand-in makes in the green boot.

#### Ranking of V9

Ranking: A > B > C. **A over B is a close call, and neither ships today.**

A over B: A is the only option where a consumer can require the exact mount it reads, with a
per-dataset entry and failure, which is what a `vm-builder` service needs; B's measured shape leaves
such a consumer with nothing at all, silently, and A's cost is a mode the grammar must admit.

B over C: B keeps the verification that authorizes the mount; C does not.

What the ranking does not settle, and the owner should weigh: A's per-dataset failure and granularity
against B's single actor, which has no per-dataset unit to drift and no new mode; and A's own silent
case, a dataset whose unit setup forgets, is the same silence B visits on every consumer that names a
unit.

What ships while it is open: **neither**. Until the mode lands or the pass's step lands, nothing mounts
a dataset under the checkout path, and V4's property is what keeps the provider from doing it:
with `canmount=noauto` the datasets stay unmounted and `doctor` reports them.

Separable: not from V4, whose property both options need, and A is not separable from the root-modes
design's answer about `--dataset`.

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
  section reads the profile and ranks the two shapes. Whether ZED runs at all on this host is a host
  fact, not a waiting item: `zfs-zed.service` is disabled and inactive.
- `TimeoutStartSec=` and `TimeoutStopSec=` values: a measurement of one reconcile and one stop pass, or
  the owner's ceilings, which is choice V8.
- Whether consumers restart on a provider restart: choice V2.
- Whether the per-dataset unit or the pass mounts a descendant: choice V9.
- V4's property change on existing datasets: the owner's act, and step 20's snapshot coverage and the
  setup runbook must state it too.

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
unit", presuming V1's option A, V3's option A, V5's option A and either answer to V2, V8 and V9.

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
it. If it may not, V9's option A has no call to make and option B's step is the only mounter.

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
- F10. This host runs systemd `262.2-1-arch`, and has no
  `/usr/lib/systemd/system/service.d/10-timeout-abort.conf`. Basis: host read.
- F11. The guest runs Fedora Linux 43, systemd `258.11-1.fc43`, kernel `6.17.1-300.fc43.x86_64`,
  OpenZFS `2.4.4-1` from the pinned tarball, SELinux enforcing, agent in `unconfined_service_t`.
  Basis: kept logs `prepare-readback.log`, `evidence-as-written-stopsec.log`.
- F12. The hazard with the covered entry's flag on and `canmount=on` makes the provider fail:
  `failed to create mountpoint: Operation not permitted`, `Result=exit-code`, `ExecMainStatus=1`, and
  the gate's job fails with `dependency` before its own check. Basis: kept log
  `evidence-hazard-flag-on.log`.
- F13. The unit text as this design writes it, both bounds included, boots green with the order gate,
  per-dataset unit, its consumer, the account consumer and sessions. Basis: kept log
  `evidence-as-written-stopsec.log`.
- F14. The checkout's key unit failing fails the gate, both consumers and the per-dataset unit with
  `dependency`, and sessions start after. Basis: round three's `evidence-key-removed.log`, journal
  lines `1795507758.973672` to `1795507760.676330`.
- F15. With the checkout unmountable and the dataset under the path at `canmount=noauto`, the provider
  finishes `Result=success`, `ExecMainStatus=0`, both with a failing key unit and with no key unit at
  all. Basis: round three's `evidence-key-removed.log`, `evidence-no-key-unit.log`; this round's
  `evidence-host-shape.log`.
- F16. With no key-loading unit, the gate refuses `checkout-not-the-dataset` with status two.
  Basis: kept log `evidence-host-shape.log`.
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
  in a boot. Basis: round three's `keep/host-user-manager-stopsec.log`; kept log
  `evidence-bound-stop.log`.
- F26. `RequiresMountsFor=` puts `After=` on every prefix mount unit and `Requires=` only on the
  fragmented one, on 258 and on 262.2. Basis: kept log `state-as-written-stopsec.log`; round three's
  `systemd-262-arms.log` arm G.
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
- F33. A consumer requiring the per-dataset unit runs after it and sees the mount. Basis: kept log
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
  temp files, one sysfs parameter and `/dev/zfs`; it has no mount rule, no rule under the account's
  paths, and no exec rule matching this host's zedlet folder. Basis: profile reading,
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
  `dac_override` denial, all inside an earlier boot's window, and the arm's own boot had none.
  Basis: kept log `evidence-as-written-stopsec.log`, "the boots this log holds".

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

This round's folder: `~/temp/agent/zfs-step-19-fourth-revision-2026-11-24/`.

- `keep/guest-scripts/`, every guest step as its own script, so each arm is reproducible:
  `01-prepare.sh`, `02-fixture.sh`, `10-units.sh`, `11-state.sh`, `12-evidence.sh`, `13-reboot.sh`,
  and the four one-off steps `03-fix-fixture.sh`, `97-timeouts.sh`, `98-audit-debug.sh` and
  `99-ready.sh`.
  `01-prepare.sh` and `02-fixture.sh` are round three's, unchanged and re-run; `10-units.sh`,
  `11-state.sh` and `12-evidence.sh` are round three's with this round's arms added, and the
  differences are the `stopsec`/`stopbody` arguments of the gate's text, four state names, a stop
  stand-in that never returns, and the audit attribution.
- `keep/logs/`, every step's whole output, named for its arm.
- `keep/host-reads.log`, the host reads with their exact output.
- `keep/apparmor-and-unit-reads.log`, the AppArmor and systemd-profile reading, with its limits.
- `keep/state-so-far.md`, the round's running state.
  `98-audit-debug.sh` there is the read that found `ausearch`'s two-argument form, which the evidence
  step needs and which its first version got wrong.

The guest was created and destroyed through `mvm --backend libvirt` at the times in
`mvm-create-guest.log` (2026-11-24T12:00:39Z) and `mvm-destroy-guest.log` (12:13:25Z), by the name its
own create printed. `mvm-list-after-destroy.log` reports no guest.

One log is a partial stream: `prepare.log` is the build's output as the guest agent delivered it, cut
short before the script's own read-back. `prepare-readback.log` is the same script's second run, which
repeats every read-back in full because the build was already done.

## Revisions

The first revision was written after the first gate review; the second after a same-vendor re-read of
that revision; the third after a second gate review; and this one after a same-vendor read of the third.
The third revision's own narrative of the second review's folds is folded into the text it changed and
is not repeated here.

This revision replaces the finding-by-finding record with the read's nine medium findings and twelve
lows, boots the unit text with both bounds, prints and boots this host's unit without the key lines,
measures the hazard with the covered entry's flag on, measures a hung start and a hung stop in one
boot, attributes every SELinux record to its boot, states the mode's conflict with the grammar and
gives its refusal a name nothing else uses, rewrites the AppArmor section from the profile text and
ranks its two shapes, and re-poses every choice with the unit lines, a command, what it printed and a
reason for each adjacent pair.

No cross-vendor review of this revision has been received, so none is folded in, and the document
stays unapproved and non-final.
