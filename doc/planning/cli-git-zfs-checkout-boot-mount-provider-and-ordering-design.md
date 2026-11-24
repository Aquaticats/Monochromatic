# Which unit mounts the checkout, and the ordering the boot gate needs

Design proposal by an agent,
revised after a same-vendor gate review,
**unapproved and non-final**.
It answers the two parts plan step 19 leaves open
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, "Step 19", "Open items")
and the findings of its own gate review,
kept under
`~/temp/agent/gate-review-zfs-step-19-checkout-boot-mount-provider-2026-11-24/keep/`.
It designs no wrapper code,
edits nothing under `doc/decision/` or `doc/handover/`,
and applies nothing to any machine.
The plan and design sentences it amends are quoted under "Edits this design suggests",
as suggestions to those records.

Every claim in this document carries a label for what it rests on:

- a kept log under this round's `keep/logs/`, named in backticks;
- a journal line, quoted with its Unix timestamp and the boot it belongs to;
- a source reading, with the file and the lines;
- `[nothing]`, where a sentence is an inference, a recommendation or an open question.

## What this revision changed

The gate review's findings,
each with what became of it.
Every finding held against `main` and the sources when this round checked it;
none is answered with a rebuttal.

- H1, the ZED hook is the declined script with a word the grammar has no.
  Folded in: the hook is the root-owned wrapper file under a ZED-legal name,
  started by ZED with no argument,
  and no argument is the attaching sweep
  (`package/git-policy/cli/src/native/worktree_clone_mount_handler/handler_mode_arguments.rs`,
  `parse`, the `arguments.is_empty()` arm).
  No script is installed,
  no `.sh` file is installed,
  and no sweep word is written anywhere.
  The exact installed name is the root-modes design's to settle,
  and "The ZED hook, and what it waits on" says which of its open choices decide each part of the line.
- H2, the failure control was not shown for the recommended configuration.
  Folded in and re-run with `canmount=noauto` applied:
  the provider now finishes with exit status zero while the checkout is absent,
  so its status carries nothing about the checkout
  (kept log `evidence-key-removed-v4.log`).
  The checkout-specific failure path is the key-loading unit's own,
  named in "The provider's status is not the checkout's status".
  The misattributed exit 1 is deleted.
- H3, three central arms had no retained log, and the driver held none of the state changes.
  Folded in: every arm this document reports has its own log,
  the driver writes one log per step,
  and the driver holds the key file, the immutable flag, each dataset's `canmount`,
  every unit variant and both drop-ins as explicit steps
  (`keep/guest-scripts/04-state.sh`, `keep/step19-revision-driver.ts`).
- M1, obligations of accepted records the unit omitted.
  Folded in: the provider drop-in, the `ExecStartPre=-` start probe,
  the propagation service and its drop-in all appear with their whole text,
  a start bound is proposed as choice V8,
  and "What a stop does today" states that the built `shutdown-unmount` refuses,
  so the unit's stop line is prefixed until the unmount half lands.
  "What a consumer's requirement carries" states that `boot-reconcile` exits zero when clones fail.
- M2, V4 had holes.
  Folded in: the pod that mounts each path descendant is named (choice V9),
  the call is the explicit `zfs mount <dataset>`,
  the descendant's unit is ordered before `systemd-user-sessions.service`,
  the hazard arm is run and shown (kept log `evidence-hazard-nested-auto.log`),
  the empty state is shown beside it (kept log `evidence-hazard-v4.log`),
  and setting the property on an existing dataset is named as the owner's act.
- M3, "the ordering edge adds nothing" held only for the boot transaction.
  Folded in and measured on systemd 258:
  a provider restart takes the gate through its stop pass and starts it again under `Requires=`
  (kept log `restart-provider.log`),
  and does nothing to the gate under `Wants=` (kept log `wants-control.log`).
  The ordering line is now stated as load-bearing at runtime, not merely as documentation.
- M4, `RequiresMountsFor=` was not considered.
  Folded in as choice V3 and run:
  one line derives the ordering edges to the root mount, the home dataset's mount unit and the checkout's learned unit,
  and adds `Requires=` only to the mount unit that has a fragment (kept log `requires-mounts-for-dependencies.log`).
- M5, `zfs-load-key.service` is the wrong name.
  Folded in: the key unit named is the generator's
  `zfs-load-key@<escaped encryption root>.service`,
  this host's `zfs-load-key.service` is a `/dev/null` symlink owned by `zfs-utils`,
  and the guest's source install writes the same `/dev/null` symlink and no `@` template
  (kept log `prepare.log` and `guest-load-key-unit.log`; host read in `keep/host-reads.log`).
  "The key-loading unit, and what happens where there is none" states what a machine with neither gets.
- M6, the ancestor condition was misworded.
  Folded in: the condition is that no directory above the entry's parent is one the account can write,
  the host path is shown to satisfy it while `/home/user` itself is the account's,
  and the planted file's writer is named as root.
- M7, quoted evidence did not match the kept logs.
  Folded in: no line is quoted from a log that does not hold it;
  every quoted line in this revision is in the kept file it is cited from,
  and the one silent-vanish result is reported both ways it was measured
  (kept logs `evidence-learned-unit-only.log` and `evidence-learned-unit-required.log`).
- M8, limits not stated.
  Folded in: the guest runs systemd 258 rather than the host's 262,
  and the claims that could move with a version are named,
  the stand-in stop pass no longer forces or detaches,
  the guest checkout is its own encryption root and that is stated as a difference from this host,
  and the generator branch and ZED remain source and text only.
- Low findings. L1's correction of V1's reason is folded into the choice;
  L2's relative pointers are replaced by names of sections, tags and paths;
  L3's long-option finding touches no line this design installs;
  L4's host timer is named in "What else on this host touches the checkout's boot".

## What it ran, and how to read this document

One throwaway guest, created and destroyed through `mvm --backend libvirt`
under this round's name `zfs-step-19-revision-2026-11-24`,
one guest at a time.
It ran Fedora 43 with systemd **258** (`258.11-1.fc43`) on kernel `6.17.1-300.fc43.x86_64`,
with OpenZFS **2.4.4** built from the release tarball pinned at sha256
`2a3c70d55a37cc71618a95a60e81ad66530201eb118d37741dc92efcf848c8b1`
(kept log `prepare.log`).
Systemd 258 is the newest a guest this host can create from `mvm` carries:
the registry's Ubuntu image is 24.04 with systemd 255,
its Fedora 43 image is the newer one,
and no newer cloud image is in the registry.
The host runs systemd 262.2-1-arch.

Three limits of the guest are stated here so no result in this document is read wider than it is:

- SELinux. Fedora's policy confines the guest agent
  (`system_u:system_r:virt_qemu_ga_t:s0`), and under it `dnf`, `mount`,
  `systemctl start` and every write outside `/run` were denied.
  The guest was therefore set to permissive for the round,
  from its serial console,
  with `sudo setenforce 0` and `SELINUX=permissive` in `/etc/selinux/config`,
  so the setting survived every boot arm.
  Nothing measured here is a result about an enforcing boot.
- The checkout is its own encryption root. The guest's checkout dataset carries its own raw key file,
  so one arm can make its mount fail by moving that file away while every other prerequisite is unchanged.
  On this host the checkout will inherit the home dataset's key, so the guest's failure arm exercises
  a checkout-specific key unit that this host will not have.
  "The key-loading unit, and what happens where there is none" says what differs.
- The programs are stand-ins. The gate is a shell program that performs the reads this design names
  (the checkout's mount row, the covered entry's flag through a second view of the home dataset,
  the private propagation, then one slot bind);
  the start probe, the stop pass and the gated descendant mount are stand-ins too,
  because the wrapper's root modes are not built.
  So every result is about the units, their ordering edges and the failures they carry,
  and about systemd and ZFS behaviour, never about the wrapper's own handler.

The driver and every guest step are kept:

- `keep/step19-revision-driver.ts` pushes each step and runs it in a capped `systemd-run --user` scope,
  one log per step;
- `keep/guest-scripts/` holds each step's script:
  `01-prepare.sh`, `02-fixture.sh`, `03-units.sh`, `04-state.sh`, `05-evidence.sh`,
  `06-reboot.sh`, `07-runtime-edges.sh`, `08-start-bound.sh`, `09-propagation.sh`;
- `keep/logs/` holds every step's whole output, named for its arm.

## The answer in brief

- The provider is `zfs-mount.service` on a machine of this host's shape,
  because this host's ZFS mount generator writes no unit for any dataset here and the unit that runs
  `zfs mount -a` is what mounts them
  (host reads; kept file `keep/host-reads.log`).
- The provider's exit status says nothing about the checkout.
  With `canmount=noauto` on the datasets under the checkout path and the checkout's key file away,
  the provider finishes with exit status zero, `active (exited)`, while the checkout is not mounted at all
  (kept log `evidence-key-removed-v4.log`).
- What fails closed on a missing checkout key is the key-loading unit, named as the generator names it,
  `zfs-load-key@<escaped encryption root>.service`, and required by the boot unit where it exists
  (kept log `evidence-key-removed-v4.log`, journal
  `zfs-load-key@…: Main process exited, code=exited, status=255/EXCEPTION`).
- Never require a mount unit systemd learned from the mount table.
  Measured on systemd 258, in the unit shape the earlier round measured on 255:
  the gate's start job vanished with no journal line at the default level and the sweep never ran
  (kept log `evidence-learned-unit-only.log`).
  A second shape, which also requires the provider, started green anyway,
  so the require is unreliable rather than uniformly fatal,
  which is the worse property for a prerequisite.
- Order with `RequiresMountsFor=<checkout path>` rather than by hand.
  One line produced `After=` on the root mount, the home dataset's mount unit and the checkout's learned unit,
  and `Requires=` only on the one mount unit that has a fragment
  (kept log `requires-mounts-for-dependencies.log`).
- The gate's ordering before `systemd-user-sessions.service` is real, and its cost is unbounded by default.
  In the green boot the sessions unit started only after the gate's job ended
  (kept log `evidence-green-closing.log`);
  a oneshot's start timeout is `infinity` by default
  (kept log `start-bound.log`, `TimeoutStartUSec=infinity`),
  so a hung reconcile holds every login with no limit until a bound is set.
  With `TimeoutStartSec=20` the hung start was killed at the bound, the stop pass ran,
  and sessions started 5 ms later
  (journal lines of the `bound` boot, quoted in "What a hung start costs").
- The datasets under the checkout path must carry `canmount=noauto`,
  and something after the verified checkout must mount them.
  Measured, with the flag absent and the checkout failing:
  the provider created the mountpoint folders inside the exposed home underlay and mounted the nested dataset there
  (kept log `evidence-hazard-nested-auto.log`).
  With `canmount=noauto`, nothing was mounted and no folder appeared
  (kept log `evidence-hazard-v4.log`);
  in the green boot the gated unit mounted it after the verified checkout and before sessions
  (kept log `evidence-green-closing.log`).
- The propagation record's service and drop-in work on a mount unit systemd learned.
  The drop-in's `Upholds=` is read for that unit,
  the service starts when the mount unit activates and stops with it,
  and its request produced a real reload of the gate
  (kept log `propagation-cycle.log`).
- The provider drop-in's request runs, and does not need restart propagation to be useful.
  With the gate on `Wants=` the provider's restart left the gate active and the request ran its `ExecReload=`
  (kept log `wants-control.log`).
  With the gate on `Requires=` the same restart already restarted the gate, and the boot's journal holds no separate
  reload, so the request added none (kept log `restart-provider.log`).
- A route-owned unit running `zfs mount <dataset>` gives a status that names the checkout and the reason,
  but only when it does not itself require the key unit:
  `cannot mount 'zfsstep19/home/monochromatic-git-checkout': encryption key not loaded`, exit status one
  (kept log `evidence-route-owned-bare.log`).
  With the key requirement in place, its failure is the same dependency failure the key unit's already produces
  (kept log `evidence-route-owned-key-removed.log`).
- The hook is the root-owned wrapper file under a ZED-legal name and nothing else.
  ZED passes no argument, and no argument is the attaching sweep.
  The installed name and the file's own name are the root-modes design's choices,
  named in "The ZED hook, and what it waits on".

## The provider on this host, and how the reads decide it

Read-only, on this host, 2026-11-24;
the whole output is in `keep/host-reads.log`.

- The checkout path has no mount of its own; `/home` is the pool's home dataset,
  mounted `shared` (`findmnt --output TARGET,SOURCE,FSTYPE,PROPAGATION --target /home/user/Monochromatic`).
- The ZFS mount generator exists at `/usr/lib/systemd/system-generators/zfs-mount-generator`,
  `/etc/zfs/zfs-list.cache` does not exist,
  and `/run/systemd/generator` holds no ZFS mount unit.
  `zfs-mount-generator(8)` says the generator reads only that cache file.
- `zfs-zed.service` is `disabled` and `inactive`, so nothing keeps a cache file either.
- What mounts the datasets is the pair the OpenZFS package installs:
  `zfs-import-cache.service` runs `zpool import -c /etc/zfs/zpool.cache -aN $ZPOOL_IMPORT_OPTS`,
  and `zfs-mount.service` runs `zfs mount -a` with `Before=local-fs.target` and `WantedBy=zfs.target`.
  `zfs.target`, `zfs-import.target`, `zfs-import-cache.service` and `zfs-mount.service` are all enabled.
  `/etc/default/zfs` leaves `ZPOOL_IMPORT_OPTS` empty,
  so the import mounts nothing and `zfs mount -a` is the only thing that mounts a dataset.
- `/usr/lib/systemd/system/zfs-load-key.service` is a symlink to `/dev/null` owned by `zfs-utils 2.4.4-3`,
  there is no `zfs-load-key@.service` file,
  and `systemctl is-enabled zfs-load-key.service` answers `masked`.
  That unit is not a key loader on this host and never was: it is the package's mask.
- The host's own key path is a person's:
  `zpcachyos` has `keyformat=passphrase` and `keylocation=prompt` with `keystatus available`,
  and the home dataset inherits it.
  So on this host the key is loaded when the pool is imported by a person at boot,
  and no unit loads it.
- `home.mount` is learned from the mount table (`FragmentPath=` empty, `SourcePath=/proc/self/mountinfo`),
  and the checkout's own unit name, `home-user-Monochromatic.mount`
  (`systemd-escape --suffix=mount --path /home/user/Monochromatic`), is `LoadState=not-found`.

The ownership and mode of the path the entry sits in, read the same way:

```text
/home                      root:root     0755   (the home dataset's root)
/home/user                 user:user     0700   (renaming it needs write on /home)
/home/user/Monochromatic   user:user     0755   (renaming it is what the flag refuses)
```

## What decides the provider on another machine

The rule, in the order the facts decide it:

1.  The provider is the generated mount unit
    `<escaped checkout path>.mount`
    when the distribution installs the ZFS mount generator and
    `/etc/zfs/zfs-list.cache/<pool>` exists and lists the checkout dataset.
    That unit has a fragment, so `Requires=` and `After=` both work on it,
    and systemd mounts the checkout as that unit's own job.
    Its ordering is the generator's:
    `After=zfs-import.target` and the key-loading unit,
    `Before=zfs-mount.service`
    (OpenZFS `zfs-2.4.4` `etc/systemd/system-generators/zfs-mount-generator.c`,
    and the private-propagation design's F15).
2.  Otherwise the provider is the unit that runs `zfs mount -a` at boot,
    `zfs-mount.service` in every OpenZFS packaging this route's records name.
    It mounts the checkout as a side effect of one command over every dataset,
    so the route names the actor and not a mount unit,
    because a mount unit appears only after the actor ran.
3.  Where neither is enabled, nothing mounts the checkout at boot,
    and nothing loads the checkout's key unless the pool's import does.
    The route refuses at boot and names what is missing;
    `doctor` reports which of the three states the machine is in,
    and whether a key-loading unit exists.

The three facts a reader must check on a machine are therefore the generator's cache file for the pool,
the state of the unit that runs `zfs mount -a`,
and the key-loading unit the checkout's dataset needs, if any.

## The provider's status is not the checkout's status

This section replaces the failure control of the earlier text,
which read a provider exit status one as a checkout failure.
It is not one, and the recommended configuration never produces it.

The arm: `canmount=noauto` on the dataset under the checkout path,
the checkout's own key file moved away,
the immutable flag in place,
every other prerequisite of the green boot unchanged.
Kept log `evidence-key-removed-v4.log`, boot `07f30872-2d36-4de1-872e-3f1dceb81e98`.

The provider's own result:

```text
Id=zfs-mount.service
ActiveState=active
Result=success
ExecMainStatus=0
```

What failed instead, in the same boot's journal:

```text
1795502972.946105 zfs-load-key@zfsstep19-home-monochromatic\x2dgit\x2dcheckout.service: Main process exited, code=exited, status=255/EXCEPTION
1795502972.950251 Dependency failed for reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts.
1795502972.956144 reconcile-git-worktree-clone-mounts.service: Job reconcile-git-worktree-clone-mounts.service/start failed with result 'dependency'.
1795502974.272641 Starting systemd-user-sessions.service - Permit User Sessions.
```

So three things are true at once, and the design must say all three:

- the provider can finish green with the checkout absent,
  because `zfs mount -a` treats an unavailable key as a dataset to skip when it is not asked explicitly
  (source: `cmd/zfs/zfs_main.c`, `share_mount_one`, the `explicit` branch that returns 0 for an unavailable key,
  reached through `share_mount_one_cb` with `B_FALSE`);
- the checkout-specific failure arrives through the **key** unit, which is named for the checkout's encryption root,
  and the boot unit's `Requires=` on it is what fails the boot unit closed;
- nothing was written into the covered entry:
  the home underlay kept exactly its three directories and the flag stayed set
  (same log, "the home underlay's inventory" and "the covered entry as it stands").

The earlier text's sentence that the provider "fails with exit status one while the datasets are partly mounted"
is withdrawn for this configuration:
with the exclusion in place there is no partly-mounted provider failure to read.
A provider exit one is still possible from another dataset's mountpoint, as the earlier round's journal showed,
and it is still not about the checkout.

## The key-loading unit, and what happens where there is none

The unit to name is the generator's own:
`zfs-load-key@<escaped encryption root>.service`,
written by `zfs-mount-generator` beside the mount unit it serves
(source: `zfs-2.4.4` `etc/systemd/system-generators/zfs-mount-generator.c`,
the block that builds `keyloadunit` with `systemd_escape(p_encroot, "zfs-load-key@", ".service")`).
It carries `DefaultDependencies=no`, `Before=zfs-mount.service`, a `keystatus` guard in its `ExecStart`,
and an `ExecStop` that unloads the key again.

Two consequences:

- Where such a unit exists, the boot unit requires and orders after it,
  as the worktree design's "Boot and shutdown unit" asks.
  The boot unit's line is
  `Requires=zfs-load-key@<escaped encryption root>.service` and
  `After=zfs-load-key@<escaped encryption root>.service`,
  and the name is the checkout dataset's own encryption root where it has one,
  and the home dataset's where the checkout inherits.
- Where none exists, nothing loads a key at boot except the pool's import.
  That is this host's state: the pool's key is entered by a person at import
  (`keylocation=prompt`, `keystatus available`), and the module ships only a mask under the name the earlier text used.
  A machine with file-based keys and no generator cache has neither,
  and on it the provider mounts nothing and the boot unit refuses with `checkout-not-the-dataset`.
  That refusal is correct and the design does not paper over it:
  `doctor` reports that no key-loading unit exists and that the checkout's key must be loaded before the provider runs.

The guest is the second shape, so it wrote the two units the generator would write,
one for the pool's encryption root and one for the checkout's,
and named them exactly as the generator names them
(`zfs-load-key@zfsstep19.service`, `zfs-load-key@zfsstep19-home-monochromatic\x2dgit\x2dcheckout.service`;
kept log `units-install.log`).
The failure arm moved only the checkout's key file,
so the failure is the checkout's own and not the pool's.

## The units, whole

The files this design installs are the boot unit, one unit per dataset whose mountpoint lies inside the checkout path,
the propagation service, the provider's drop-in, the propagation drop-in, and the ZED hook.
Two of the lines below wait on a value or a measurement this round does not have,
and each says so where it appears:
`TimeoutStartSec=`, whose value choice V8 settles, and `ExecStartPre=`, which the accepted start-probe design
installs only once the probe's deadline is measured.

The program they start is a **placeholder**:
the owner answered "One program only, ever" on 2026-11-23
(`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`),
and how root reaches the modes is the root-modes design's open choice
(`doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`).
Two placeholders stand in this design's text:

- `<root-file>` is the root-owned regular file the modes are installed as,
  which the root-modes design's choice 3 fixes:
  one file under the ZED hook's own name (its option A),
  one file named for the modes (option B),
  one file named for the wrapper (option C),
  or a copy or hard link per name (option D).
- `<group-word>` is the group word the units spell before the mode word,
  which that design's choice 2 fixes: `zfs-root` (its option A), `mount` (B) or `root` (C).

Every line in this document is written so that it holds under **any** of those answers:
the root-modes design drops the group words when the start is the hook's own name,
and reads them otherwise,
so one line works from a file named either way.
The only thing that changes with choices 2 and 3 is the path and the word in it,
and "The ZED hook, and what it waits on" lists precisely which line waits on which choice.

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
ExecStartPre=-<root-file> <group-word> start-probe
ExecStart=<root-file> <group-word> boot-reconcile
ExecReload=<root-file> <group-word> boot-reconcile
ExecStopPost=-<root-file> <group-word> shutdown-unmount

[Install]
WantedBy=multi-user.target
```

### Why each line

- `Requires=zfs-load-key@<escaped encryption root>.service` and its `After=`.
  The prerequisite whose failure is the checkout's.
  Installed only where such a unit exists; see "The key-loading unit, and what happens where there is none".
  Measured: with the key file away, the key unit failed and this unit's start job failed with `dependency`
  before its `ExecStart` ran (kept log `evidence-key-removed-v4.log`).
- `Requires=zfs-mount.service` and `After=zfs-mount.service zfs-import.target`.
  What the requirement buys is a pull-in, a dependency failure, and **restart propagation**, and the last one is
  load-bearing at runtime rather than at boot:
  measured, `systemctl restart zfs-mount.service` took this unit through its stop pass
  (`Stopping reconcile-…` then `Deactivated`) and started it again
  (journal lines `1795503513.546891` to `1795503513.686141`, kept log `restart-provider.log`),
  while the same pair written as `Wants=` left it untouched
  (kept log `wants-control.log`).
  At boot the ordering edge adds nothing:
  the same unit with no edge to the provider at all still started after it and passed
  (kept log `evidence-no-provider-edge.log`,
  `Finished zfs-mount.service` at `1795503075.003525` then `Starting reconcile-…` at `1795503075.501653`).
  The lost-event design's F17 and the propagation design's first acceptance arm both depend on the restart
  propagation, so the line is required for them and not for boot ordering.
  Replace the name with the generated mount unit where the generator writes one;
  keep the `zfs-import.target` edge either way.
- `RequiresMountsFor=<checkout path>`.
  This replaces the earlier text's two hand-written ordering lines
  (`After=home.mount` and `After=<escaped checkout path>.mount`).
  Measured, one line produced `After=` on `-.mount`, on the home dataset's mount unit and on the checkout's
  learned mount unit, and `Requires=` only on `-.mount`, the one of the three that has a fragment
  (kept log `requires-mounts-for-dependencies.log`).
  That is the property the hand-written lines cannot give:
  every prefix mount is covered, the names are derived from the path rather than copied per machine,
  and no `Requires=` lands on a learned unit, which is the trap "Never require a learned mount unit" measures.
  The line is what orders the stop pass: reverse ordering puts this unit before the checkout's mount unit.
- `Before=systemd-user-sessions.service`.
  What makes the checkout ready before an account session.
  Measured in every green boot: the gate finished before the sessions unit started
  (kept log `evidence-green-closing.log`,
  `Finished reconcile-…` at `1795503632.695051`, `Starting systemd-user-sessions.service` at `1795503633.152026`).
  Its cost is the subject of "What a hung start costs" and choice V8.
- `Before=umount.target` with `Conflicts=umount.target`.
  The stop half.
  Measured in the earlier round's shutdown on systemd 255:
  `Stopping reconcile-…`, then `zfs-mount.service: Deactivated successfully`, then the checkout's learned unit
  unmounted, with the stop program's durable line in the same pass.
  This round measured the same reverse order at runtime on 258 through the provider restart recorded in "Why each line".
- `TimeoutStartSec=<the measured ceiling of one reconcile>`.
  New in this revision, and the value is not yet known:
  no run has measured one reconcile over this host's storage.
  What the line buys is measured at a provisional value:
  `TimeoutStartSec=20` on a hung start produced
  `start operation timed out. Terminating.`, `Failed with result 'timeout'`,
  the stop pass's own durable line in that same stop,
  and `systemd-user-sessions.service` starting at the same instant
  (kept log `evidence-bound.log` and the follow-up read in `evidence-bound-late.log`).
  Choice V8 ranks the alternatives and says what ships while the value is missing.
- `ExecStartPre=-<root-file> <group-word> start-probe`.
  The start-probe design's placement, adopted here because plan step 19 installs the unit:
  one root process per boot, in the service manager's own child, before the reconcile.
  The dash prefix is load-bearing and measured:
  a failing `ExecStartPre=` under the prefix left the unit `active` and ran its `ExecStart`,
  the same line without the prefix failed the unit and skipped it,
  and a **timeout** inside `ExecStartPre=` sent the unit to its stop action whatever the prefix said
  (kept log `start-bound.log`, three units, each read back).
  The probe's own deadline is what bounds the probe;
  it is not the unit's `TimeoutStartSec=`.
  The accepted start-probe design defers this line until step 6's measurement exists;
  this proposal keeps that condition: the line is installed with the unit only once the probe's deadline is sourced,
  and until then the unit is installed without it, which the accepted design already allows.
- `ExecStart=<root-file> <group-word> boot-reconcile`.
  The pass itself.
  It verifies the checkout's mount identity, the covered entry's flag and the private propagation,
  then attaches what the intent asks for
  (the wrapper's modes today; the stand-in in this round's guest).
- `ExecReload=<root-file> <group-word> boot-reconcile`.
  The line the lost-event record accepts and the propagation design's printed lever needs.
  Measured: `systemctl reload-or-restart` on the active unit ran this command and no stop pass
  (kept log `reload-gate.log`, the `Reloading reconcile-…` line at `1795503559.285343` and
  `Reloaded` at `1795503559.315252`; the pass it started is the evidence file's own
  `SWEEP start … 06:59:19.301Z` with `attached=registered-already`, in `reload-gate-after.log`).
  A reload re-runs the reconcile on an active checkout, so the mode must be idempotent;
  the stand-in was made so after the first run showed a second bind of the same slot.
- `ExecStopPost=-<root-file> <group-word> shutdown-unmount`.
  The stop pass, and the dash prefix is this revision's answer to M1.
  The built `shutdown-unmount` always refuses
  (`package/git-policy/cli/src/native/worktree_clone_mount_handler/shutdown_unmount_mode.rs`,
  `run` returning `AWAITING`),
  so without the prefix every stop fails the unit for a mode whose step has not landed.
  With the prefix the refusal is logged and the stop stays clean,
  and the line loses the prefix when the unmount half lands,
  so a real unmount failure fails the unit as the design wants.
  The design's requirement that the stop pass run is unchanged either way.
- `WantedBy=multi-user.target` with default dependencies.
  This is what makes the unit part of the boot at all.
  It also carries the implicit `After=basic.target`, which is why the explicit provider ordering adds nothing at boot.

### What a stop does today

Two facts, both source readings of `main`:

- `shutdown-unmount` refuses, so the stop line in "The boot and shutdown unit" is prefixed with `-`
  until that mode lands.
- `boot-reconcile` exits **zero** when individual clones fail:
  `boot_reconcile_mode.rs` publishes a readiness record with `SweepState::Failed` and still returns `Ok`
  (`boot_reconcile_mode.rs:158` to `:164`), and only a refused prerequisite returns `Err`.
  So a consumer's `Requires=` on this unit carries a boot refusal and not a per-clone sweep failure.
  A clone's failure reaches a consumer through the readiness record, not through the unit's result.

### The units for the datasets under the checkout path

One unit per dataset whose mountpoint lies inside the checkout path,
generated by setup and reported by `doctor`:

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
ExecStart=<root-file> <group-word> mount-path-descendant --dataset <dataset>

[Install]
WantedBy=multi-user.target
```

For this host that is one unit, for `zpcachyos/nosnap/vm-builder-output`.
The measured stand-in ran the same call the mode would make, an explicit
`zfs mount <dataset>`, after checking that the checkout's own mount row is the checkout dataset
(kept log `evidence-green-closing.log`:

```text
SWEEP checkout-row=zfsstep19/home/monochromatic-git-checkout zfs
NESTED start 2026-11-24T07:04:10.887Z
NESTED mounted=zfsstep19/nosnap/vm-builder-output 2026-11-24T07:04:10.920Z
```

).
The explicit form matters:
`zfs mount -a` skips a `noauto` dataset and skips an unavailable key silently,
while the explicit form refuses with a named message
(source: `cmd/zfs/zfs_main.c`, `share_mount_one`, the `canmount` table and the key branch, both gated on `explicit`;
measured in "The route-owned alternative" for the key case).
`Before=systemd-user-sessions.service` is new in this revision and is what closes the window M2 named:
in the earlier round's `noauto` boot the nested unit finished **after** the sessions unit
(`Finished systemd-user-sessions.service` at `1795486959.129715`,
`Finished git-worktree-nested-output-mount.service` at `1795486959.180427`,
that round's `journals-per-boot.log`),
so an account session could observe the mount arriving late.
With the line, the closing boot's ordering is gate, then descendant, then sessions
(kept log `evidence-green-closing.log`).

Choice V9 says who mounts these datasets and why one unit per descendant is ranked first.

### The provider drop-in, from the lost-event record

```ini
# /etc/systemd/system/zfs-mount.service.d/reconcile-git-worktree-clone-mounts-after-checkout.conf
[Service]
ExecStartPost=-/usr/bin/systemctl --no-block --no-ask-password reload-or-restart reconcile-git-worktree-clone-mounts.service
```

Measured on systemd 258, with the gate's requirement written as `Wants=` so the restart would not move it:
after `systemctl restart zfs-mount.service` the gate stayed active and then
`Reloading reconcile-…` and `Reloaded` appeared
(journal lines `1795503693.546036` and `1795503693.566525`, kept log `wants-control.log`).
The `-` prefix kept the provider `active` with `Result=success`.
With the gate's requirement written as `Requires=`, the same restart already restarted the gate,
and that boot's journal holds no separate reload, so the drop-in's own effect is not visible in that shape.
The absence is the whole of that result: what the log shows is a restart with no `Reloading` line
(kept log `dropin.log`).
The install is the record's, and the record's reason stands: the request is what reconciles after a post-key-load
checkout mount, including a provider started by something other than the boot.

### The propagation service and its drop-in, from the propagation record

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

Three results this round adds, all on systemd 258 and all kept:

- The drop-in is read for a mount unit systemd learned.
  `systemctl show` for the learned unit reports
  `Upholds=checkout-propagation.service` and names the drop-in file in `DropInPaths`
  (kept log `dependencies-read.log`).
- `Upholds=` starts the service when the mount unit activates,
  and `BindsTo=` stops it when the mount unit deactivates.
  In the cycle, stopping the learned mount unit took the service with it
  (`checkout-propagation.service: Deactivated successfully` at `1795503644.232958`,
  then the mount unit's own `Unmounted` at `1795503644.249084`),
  and mounting the dataset again brought it back
  (`Starting checkout-propagation.service` at `1795503646.277962`,
  then `Reloading` and `Reloaded` of the gate at `1795503646.297339` and `1795503646.325515`),
  kept log `propagation-cycle.log`.
- The service starts before the gate on a boot where both are wanted.
  In the green boot the service started at `1795503632.608716` and the gate at `1795503632.624819`,
  because the learned unit's activation submits the upheld service immediately
  (kept log `evidence-green-closing.log`).
  Its request then starts or merges into the gate, which is what `reload-or-restart` does.

`systemd-analyze verify` reports one line for this pair, and it is expected and must be in `doctor`'s list of known
lines rather than treated as a defect:
`checkout-propagation.service: Failed to create checkout-propagation.service/start:`
`Unit <escaped checkout path>.mount not found.`
(kept log `units-install.log`), because the mount unit exists only after the mount appears.

Neither drop-in is installed without the boot unit's `ExecReload=`;
the propagation record's own condition, and the reason is that without the line the same request is a restart,
which runs the stop pass and unmounts every attached clone.

### The ZED hook, and what it waits on

The hook is **the root-owned regular file under a ZED-legal name**, and nothing else.

```text
/usr/libexec/monochromatic-git-policy-cli/<the installed file>   regular, 0755, root
/etc/zfs/zed.d/history_event-<hook-name>                          symlink to the file
```

- ZED starts a zedlet by its file name, with no arguments
  (`zed.8`, and the root-modes design's replay of `cmd/zed/zed_event.c:1007` to `:1022`).
- A zedlet's name must carry the zevent class as a prefix followed by a non-alphabetic character
  (`zed.8`, "ZEDLETs are named after the zevent class");
  `history_event-…` satisfies that and so does the name the handler's constant already carries.
- No argument is the attaching sweep:
  `handler_mode_arguments.rs`'s `parse` returns `HandlerMode::AttachingSweep` for an empty argument list.
  There is no `sweep` word in the grammar, and this design writes none.
- No script is installed. The earlier text's `exec <root-mode-entry> <sweep>` file is deleted with its justification:
  "every zedlet here is a `.sh` file" was a description of the packaged zedlets, not a requirement of ZED's,
  and the one-program record retires an installed program of the route's own.

What each part waits on in the root-modes design, by whose choice:

- the file's own name and whether the file is named for the hook or for the modes: choice 3 (its options A, B, C, D);
- the hook name's exact spelling: choice 1's constant, which the record fixes as
  `history_event-mount_git_worktree_clone_datasets` today
  (`package/git-policy/cli/src/native/worktree_clone_mount_handler/handler_install_paths.rs`,
  `ZEDLET_INSTALL_PATH`)
  unless that design renames it, which the start-name record allows;
- the group word in every unit line: choice 2;
- whether a root start under another name runs the wrapper: choice 4's placement, which decides whether a typo at
  that path is the modes' refusal or Git as root;
- the words `--version` and the refusals print: choice 5, which only `doctor`'s reading depends on;
- whether the account's `PATH` keeps a short name: choice 6, which this design does not use.

The unit lines in "The boot and shutdown unit" and "The units for the datasets under the checkout path"
are written so that they survive choices 2, 3 and 4 unchanged in shape:
the same `<root-file> <group-word> <mode>` reaches the mode whether the file is named for the hook or for the modes,
because the root-modes design drops the group words under the name rule and reads them otherwise.

## The covered entry, and the ancestors the earlier text got wrong

Measured as the account, with the checkout unmounted and the entry immutable
(the earlier round's `guest-ancestor.log`, unchanged by this round and re-read for this text):

```text
mv ~/Monochromatic ~/Monochromatic.moved          -> Operation not permitted     (the flag)
rmdir ~/Monochromatic                              -> Operation not permitted     (the flag)
touch ~/Monochromatic/planted                       -> Operation not permitted     (the flag)
mv <home>/account <home>/account.moved              -> Permission denied           (the parent, root-owned)
mv ~/projects ~/projects.moved                      -> succeeds                    (the parent, account-owned)
   then ~/projects/Monochromatic/planted exists     (planted by root in the arm, not by the account)
zfs mount zfsstep19/home/monochromatic-git-checkout -> Insufficient privileges    (the account)
```

The condition the design needs, which the earlier text stated wrongly:

> the covered entry is protected while every directory **above the entry's parent**, up to the home dataset's root,
> is one the account cannot write.

The earlier text's wording, "every directory between it and the home dataset's root is one the account cannot write",
fails at this host's own path, because `/home/user` is the account's directory (mode `0700`, owner `user:user`,
read from the host), and the earlier text then claimed the condition "holds" there.
What actually holds at `/home/user/Monochromatic` is the corrected condition:
`/home` is root's, the entry is the account's, and the account cannot rename `/home/user`
because that needs write on `/home`.
At a deeper path the account owns, such as `~/projects/Monochromatic`, the account renames `~/projects` and the flag
on the entry protects nothing, which the arm shows.

## The failure controls

Four arms, each with its own kept log, each measured on systemd 258.
The controls are what make the green boot's pass mean something.

- Green, production lines: passes.
  Kept log `evidence-green-closing.log`.
  The probe ran, the gate read the checkout row, the flag and the private propagation, attached the slot,
  the descendant's unit mounted its dataset, the consumer saw all of it, and sessions started after both.
- The covered entry's flag removed: refuses and fails closed.
  Kept log `evidence-flag-removed.log`:
  `SWEEP covered=----------------------`, `SWEEP refused=covered-entry-not-immutable`,
  `ExecMainStatus=2`, both dependents' jobs failed with `dependency`.
- The checkout's key file away: the key unit fails, the gate's job fails closed.
  Kept log `evidence-key-removed-v4.log`, quoted in "The provider's status is not the checkout's status".
- The provider cannot fail on the checkout's behalf: shown by the same arm.
  The provider's `ExecMainStatus=0`, and a boot with no key-loading unit at all produced the same green provider
  in the earlier round.
- The account's login is not gated by the gate's success, only delayed by its end.
  Kept logs `evidence-key-removed-v4.log`, `evidence-flag-removed.log` and `evidence-bound.log`:
  in each, `Starting systemd-user-sessions.service` follows the failed job.
  That is why the design puts `Requires=` on the services of the account that need the checkout
  rather than on login, and choice V6 says which services are still to be named.

## What a hung start costs

Three measurements, one boot and two units, all on systemd 258.

The default: a oneshot has no start timeout.
`systemctl show` for a unit with no `TimeoutStartSec=` reports `TimeoutStartUSec=infinity`
(kept log `start-bound.log`, its "five" arm).
Combined with `Before=systemd-user-sessions.service`, an unbounded reconcile holds every login with no limit,
which is what the green boot's ordering also shows from the other side
(the sessions unit started only after the gate finished).

The bound at work, from the `bound` boot's journal, a gate whose `ExecStart` sleeps and whose
`TimeoutStartSec=20`:

```text
1795503319.620544 Starting reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts (start bound, hanging start)...
1795503339.770182 reconcile-git-worktree-clone-mounts.service: start operation timed out. Terminating.
1795503339.771542 reconcile-git-worktree-clone-mounts.service: Main process exited, code=killed, status=15/TERM
1795503339.793068 reconcile-git-worktree-clone-mounts.service: Failed with result 'timeout'.
1795503339.798341 Starting systemd-user-sessions.service - Permit User Sessions.
```

and the stop pass ran inside that failure:
`SWEEP stopped 2026-11-24T06:55:39.785Z uptime=26.90` (same boot, `/run/step19-evidence.log`,
kept in `evidence-bound-late.log`).
So a bound converts "login held forever" into "login delayed by the bound, the unit failed, the stop pass ran".

The prefix's limit, measured separately:

- a failing `ExecStartPre=` under `-`: unit `active`, `ExecStart` ran;
- the same line without `-`: unit `failed`, `ExecStart` did not run;
- a **timeout** in `ExecStartPre=` under `-`: `start-pre operation timed out`, `Result=timeout`, and the unit's
  `ExecStopPost=` ran.

All three are in `start-bound.log`.
The consequence for the design is that the probe must enforce its own deadline,
as the accepted start-probe design has it, and must not rely on the prefix to survive a timeout.

## The route-owned alternative, measured

Choice V1's second option is a unit the route owns, which mounts the checkout explicitly,
with the checkout dataset set to `canmount=noauto` so the provider's scan skips it.
Its whole text, as it ran:

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
ExecStart=/usr/sbin/zfs mount <checkout dataset>
ExecStop=/usr/sbin/zfs unmount <checkout dataset>

[Install]
WantedBy=multi-user.target
```

- Green: it mounts the checkout.
  Kept log `evidence-route-owned.log`:
  the provider finished at `1795503146.401532` with the checkout skipped,
  the route unit started at `1795503146.404749` and finished at `1795503146.434234`,
  and the gate verified afterwards.
- With the key unit required and the key file away: a dependency failure, like the provider shape.
  Kept log `evidence-route-owned-key-removed.log`:
  the gate's job and the route unit's job both failed with `dependency`, and
  `ExecMainStatus=0` for the route unit because its `ExecStart` never ran.
- With no key requirement and the key file away: a status that names the dataset and the reason.
  Kept log `evidence-route-owned-bare.log`:

  ```text
  zfs[838]: cannot mount 'zfsstep19/home/monochromatic-git-checkout': encryption key not loaded
  mount-monochromatic-checkout.service: Main process exited, code=exited, status=1/FAILURE
  mount-monochromatic-checkout.service: Failed with result 'exit-code'
  ```

That third result is the option's only measured advantage,
and it exists exactly when the design does **not** put the key unit's requirement on the route unit.
Choice V1 weighs that.

## What the runs show, one arm per log

Each line names the arm, its kept log and the one result to read there.
Every log names the boot's id, uptime and versions at its head.

- `prepare.log`: Fedora 43, systemd `258.11-1.fc43`, kernel `6.17.1-300.fc43`, OpenZFS `2.4.4-1`;
  the source install's unit list and its `zfs-load-key.service -> /dev/null`.
- `fixture.log`: the pool, the datasets, the cache file the import unit reads, the immutable entry,
  and `zfs mount -a` mounting the checkout **and** the nested dataset under it.
- `units-install.log`: every unit and drop-in as systemd reads them, the escaped unit name,
  `systemd-analyze verify`'s one expected line.
- `evidence-production.log`: the boot with no key units at all, which is why the key units exist:
  the provider finished green, the checkout was absent,
  and the gate refused with `SWEEP refused=checkout-not-the-dataset`.
- `evidence-green-final.log` and `evidence-green-closing.log`: green boots with every line of this design:
  probe, verify, propagation, slot, descendant mount, consumer, sessions.
- `evidence-key-removed-v4.log`: the provider green while the checkout is absent, and the key unit's failure.
- `evidence-flag-removed.log`: the flag's removal refused with `covered-entry-not-immutable`.
- `evidence-hazard-nested-auto.log`: the provider mounting a `canmount=on` descendant into the exposed underlay while
  the checkout is absent, and creating the folders there.
- `evidence-hazard-v4.log`: the same with `canmount=noauto`, nothing mounted, nothing created.
- `evidence-route-owned.log`, `evidence-route-owned-key-removed.log`, `evidence-route-owned-bare.log`:
  the three route-owned results of "The route-owned alternative".
- `evidence-requires-mounts-for.log` and `requires-mounts-for-dependencies.log`:
  the V3 boot and the dependencies systemd derived from one `RequiresMountsFor=` line.
- `evidence-no-provider-edge.log`: the boot-ordering control, green with no edge to the provider.
- `restart-provider.log`, `wants-control.log`, `dropin.log`, `reload-gate.log` and `reload-gate-after.log`:
  the four runtime edge results, restart propagation, its absence under `Wants=`, the provider hook's request,
  and the reload path.
- `propagation-cycle.log` and `dependencies-read.log`: `Upholds=` and `BindsTo=` on the learned mount unit,
  and the drop-in paths systemd recorded.
- `start-bound.log`: the four prefix and bound units.
- `evidence-bound.log`, `evidence-bound-late.log`: the hung start, the bound firing, the stop pass and the sessions.
- `evidence-learned-unit-only.log`: the gate's job vanishing silently with `Requires=` on the learned unit alone,
  and `evidence-learned-unit-required.log` where the same require with the provider required ran green and the
  drop-in's request logged `Unit … not found`.

## What the runs do not show

- Not the product handler. Every program is a stand-in; the wrapper's root modes, the readiness record,
  the start probe's own deadline and the unmount half never ran.
- Not this host's kernel or pool. The guest runs kernel `6.17.1-300.fc43`, OpenZFS `2.4.4-1` and a file-backed pool.
  No host dataset was mounted, unmounted, imported, exported or changed, and no service manager on this host was asked
  to change anything.
- Not a generated mount unit. No arm made the ZFS mount generator write a unit, because that needs a root-written
  cache file on a machine whose ZED tracks the pool.
  The branch of the rule that names `<escaped checkout path>.mount` as the provider is a source reading
  plus the host reads, not a run.
- Not the generator's key units as generated. The guest's two key units were written by hand to the generator's
  own shape and names, not emitted by the generator.
- Not ZED. No arm ran `zfs-zed.service`, and the hook is a text this design proposes.
- Not this host's key path. The guest's checkout is an encryption root of its own with a file key;
  this host's is a passphrase prompted at import with no key unit at all.
- Not an enforcing SELinux boot. The guest ran permissive, as "What it ran" states.
- Not a failed import, a delayed import, or a delayed key load.
  The arms failed the key load and, separately, removed the flag; nothing made the pool's import slow or absent.
  Plan step 19's acceptance text asks for the delayed cases too, and they remain unmeasured.
- Not a busy shutdown. The stop pass ran with one slot attached and nothing holding it;
  a busy clone, a failed stop and an interrupted boot are step 9's arms.
  The stand-in's stop pass does not force or detach, and reports a busy target, which is the design's requirement.
- Not the same systemd as this host. Every arm ran on 258; this host runs 262.2-1-arch.
  The claims that could move with a version are the `Requires=`-on-a-learned-unit outcome
  (measured both ways here, and the source that logs it at debug level),
  the mount-unit learning the sibling design's F12 and F38 read from 262, and
  `RequiresMountsFor=`'s expansion, which the guest's own manager reported rather than the host's.

## Never require a mount unit systemd learned

Measured twice on systemd 258, in two shapes, and both shapes matter.

**The shape the earlier round used**: `Requires=<escaped checkout path>.mount` and no `Requires=` on the provider.
The gate's start job never appeared in the boot's journal at all,
the unit was left `inactive (dead)` with `Result=success`,
the sweep never ran (`/run/step19-evidence.log` did not exist),
and sessions opened.
Kept log `evidence-learned-unit-only.log`:
its journal holds `Starting zfs-mount.service` and then `Starting systemd-user-sessions.service`
with no line for the gate and no `Dependency failed` line.

**The shape with the provider also required**: `Requires=<escaped checkout path>.mount` beside
`Requires=zfs-mount.service` and its `After=`.
The gate's job ran and passed, because by the time its job was reached the provider had mounted the checkout
and the learned unit existed.
The same boot's journal records the cost of the require in another place:
the provider drop-in's request answered
`Failed to reload-or-restart reconcile-git-worktree-clone-mounts.service: Unit <escaped checkout path>.mount not found.`
(kept log `evidence-learned-unit-required.log`).

So the require is not uniformly fatal, which is worse than uniformly fatal:
whether the boot unit's whole pass survives depends on a unit that does not exist when the transaction is built
and on the order in which other requirements happen to pull the provider in.
The source says why the failure is quiet: a dependency job that cannot be added for `-ENOENT` is logged at debug level
(systemd v262 `src/core/transaction.c`, the block that logs
`Cannot add dependency job, ignoring` through `log_unit_full_errno` with `LOG_DEBUG`,
and `log_unit_debug_errno` for `Cannot add dependency job to transaction, deleting job`),
and a learned mount unit cannot be started anyway:
its own start answers `No mount parameters to operate on`
(the earlier round's `journals-per-boot.log`, `No mount parameters to operate on` at `1795486467.319399`).

The rule the design states, then, is:
**order with `After=` or with `RequiresMountsFor=`,
and never write `Requires=` on a unit systemd learned from the mount table.**
The corrected V1 reason follows from it:
the lost-event record rejected a mount unit **as a trigger**, not as a provider
(`doc/decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md`, "What was not chosen"),
and the account-writable-path exposure applies to `zfs mount -a` just as much as to a route-owned unit.

## What else on this host touches the checkout's boot

`zfs-space-guard.service` is `After=zfs-mount.service` and its timer runs every five minutes,
deleting old `/home` snapshots when the pool is over 85 percent full
(host read in `keep/host-reads.log`).
It has no edge to the boot unit and this design gives it none;
what matters for a reader is that a service ordered after the provider can run while the checkout is absent,
which is another reason a provider's own success is not a readiness signal.

## The choices this design puts to the owner

Each is separable, each shows a concrete example, and each is ranked by what it delivers for this route's boot,
not by effort and not by what an incumbent does.
After each ranking, "What the review changed" names the earlier ranking it replaces and why.

### V1. Which unit is the provider

#### Option A: the unit that runs `zfs mount -a`

This design's answer. `Requires=zfs-mount.service` and `After=zfs-mount.service zfs-import.target`,
with `Requires=zfs-load-key@<escaped encryption root>.service` carrying the checkout-specific failure.

What the boot prints when the checkout's key is missing, from kept log `evidence-key-removed-v4.log`:

```text
zfs-load-key@zfsstep19-home-monochromatic\x2dgit\x2dcheckout.service: Main process exited, code=exited, status=255/EXCEPTION
Dependency failed for reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts.
Job reconcile-git-worktree-clone-mounts.service/start failed with result 'dependency'.
```

What it delivers: one actor mounts each dataset once, so no second mount of the checkout path and no property beyond
V4's; the accepted lost-event and propagation designs already name this unit; and the checkout-specific failure is
carried by the key unit's requirement.

What it costs: the provider's own status says nothing about the checkout, so a reader must know which unit to read,
and on a machine with file-based keys and no key unit, nothing loads the key and the gate's refusal is the only signal.

#### Option B: a route-owned unit that mounts the checkout explicitly

The unit in "The route-owned alternative", with the checkout dataset at `canmount=noauto` so the provider skips it,
and `ExecStart=/usr/bin/zfs mount <checkout dataset>`.

What it delivers: a unit whose status is the checkout's, and,
where the design does not also require the key unit, a message that names the dataset and the reason
(`cannot mount '…': encryption key not loaded`, exit status one, kept log `evidence-route-owned-bare.log`);
its own journal entry; a unit a person can reload alone.

What it costs: a `canmount=noauto` on the checkout dataset itself, which is an owner's property change;
a missing property makes the provider mount the checkout and this unit fail with `filesystem already mounted`,
a red boot for a state `doctor` could have reported;
its `ExecStop=` unmounts the checkout independently of the provider's stop,
which the design's stop ordering does not want;
and with the key requirement in place its failure is the same dependency failure as option A's
(kept log `evidence-route-owned-key-removed.log`), so its measured advantage appears only in the variant without that
requirement.

#### Option C: keep the conditional

A unit whose `Requires=` names whichever provider the machine has.

What it delivers: nothing to settle here.

What it costs: a conditional in a unit file is not a thing systemd does;
a reader cannot audit which prerequisite a given boot had;
and plan step 19's own open item asks this design to settle it.

#### Ranking of V1

Ranking: A > B > C.

A over B: A needs no property change and no second actor,
and the checkout-specific failure B was proposed to buy is already bought by the key unit's requirement;
B's advantage survives only where that requirement is removed,
and then the design has two places to keep in step with the checkout's encryption root.

B over C: B gives one auditable prerequisite with a status of its own,
while C leaves the reader to guess which unit a machine had.

What the review changed: the ranking stands and every reason the earlier text gave is replaced.
The lost-event record rejected a mount unit as a **trigger**, not as a provider,
and the account-writable-path exposure applies to `zfs mount -a` equally,
so neither can be used against B. What decides it now is the property change and the second actor.

### V2. What the boot unit requires of the provider

#### Option A: `Requires=` plus `After=`

This design's answer. Measured, `systemctl restart zfs-mount.service` printed
`Stopping reconcile-…`, `Deactivated`, then `Starting reconcile-…` and `Finished`
(kept log `restart-provider.log`).

What it delivers: restart propagation, which the lost-event record's F17 and the propagation design's first acceptance
arm both use; a dependency failure when the provider fails;
and a pull-in on a machine where nothing wanted the provider.

What it costs: a provider failure also fails the boot unit,
so one boot can show a failed gate whose cause is the provider, and the stop pass runs on every provider restart.

#### Option B: `Wants=` plus `After=`

Measured: after the provider's restart the gate stayed active, and only the provider's own stop and start appear
(kept log `wants-control.log`).

What it delivers: the gate survives a provider restart without a stop pass,
and the provider hook still reconciles after it (the same log's `Reloading` and `Reloaded`).

What it costs: no restart propagation, which two accepted records depend on;
and a provider failure does not fail the boot unit.

#### Option C: `After=` only

The boot-ordering control, which started after the provider and passed with no edge to it at all
(kept log `evidence-no-provider-edge.log`).

What it delivers: the least coupling.

What it costs: nothing pulls the provider in, and no restart reaches the gate.

#### Ranking of V2

Ranking: A > B > C.

A over B: two accepted records rely on restart propagation, and the measured difference is exactly that propagation;
B's survival property is attractive, but the wrapper's own reload supplies it,
which needs `ExecReload=` and the provider hook, both of which A has too.

B over C: B pulls the provider in and reconciles after it; C does neither.

What the review changed: the ranking stands and its reason does not.
The earlier text justified `Requires=` by the failure path and the pull-in,
and the failure path it named was the provider's exit status, which this revision shows is not about the checkout.
The reason now is restart propagation, measured, with the failure path carried by the key unit.

### V3. How the mount units are named in the ordering

#### Option A: `RequiresMountsFor=<checkout path>`

This design's answer. One line produced, in the guest:

```text
Requires=-.mount "zfs-load-key@zfsstep19-home-monochromatic\x2dgit\x2dcheckout.service" zfs-mount.service system.slice sysinit.target
After=-.mount "srv-zfs\\x2dstep\\x2d19\\x2dfixture-home.mount" zfs-mount.service "srv-zfs\\x2dstep\\x2d19\\x2dfixture-home-account-Monochromatic.mount" … zfs-import.target
```

(kept log `requires-mounts-for-dependencies.log`), and the boot passed
(kept log `evidence-requires-mounts-for.log`).

What it delivers: every prefix mount unit derived from the path;
both provider states covered without a line per machine;
`Requires=` only on units that have a fragment, which is exactly the safe half of the rule in
"Never require a mount unit systemd learned"; one line instead of two.

What it costs: the derived dependency set is a manager report rather than a line a reader can check in the unit file,
and a machine whose path prefix differs has a different set, which is the point but also means the file alone
under-describes it.

#### Option B: the two hand-written ordering lines

`After=home.mount` and `After=<escaped checkout path>.mount`, as the earlier text had them.

What it delivers: every name visible in the file, and no expansion to reason about.

What it costs: it names the home dataset's mount unit even on a machine where the checkout lives under another prefix,
misses that prefix's other mounts, and needs the learned unit's name computed per machine.

#### Option C: no mount-unit edge and a prose statement

What it delivers: nothing to keep in step.

What it costs: the stop half loses its only statement, and `systemd-analyze verify` cannot check the intent.

#### Ranking of V3

Ranking: A > B > C.

A over B: A covers both provider states and every prefix mount with one line, and it cannot put a `Requires=` on a
learned unit; B needs a per-machine name and covers only what it names.

B over C: B states the stop half and C states nothing.

What the review changed: A is new.
The review named `RequiresMountsFor=` as an option this design had not considered, this round ran it,
and it takes first place over the two lines the earlier text kept.

### V4. `canmount=noauto` on every dataset under the checkout path

#### Option A: exclude them and mount them after the gate

This design's answer. With the flag absent and the checkout's mount failing,
`canmount=noauto` produced no mount and no folder (kept log `evidence-hazard-v4.log`),
against the hazard arm's `d …/Monochromatic/package/vm-builder/cli/output` and a mounted output dataset there
(kept log `evidence-hazard-nested-auto.log`).

What it delivers: the provider cannot mount a descendant before the checkout is verified;
the gated unit's explicit `zfs mount` refuses with a named message rather than skipping;
and it is the only way the design's "gate every path descendant" sentence can hold.

What it costs: a property change on existing datasets, which is the owner's act;
a dataset under the path that setup misses is mounted by the provider again, so `doctor` must read and report it;
and a dataset under the path is no longer mounted by `zfs mount -a` alone, so anything that expected that must name it.

#### Option B: let the provider mount them and have the gate report them

What it delivers: no property and no extra units.

What it costs: the hazard arm, which is an account-visible mount of `vm-builder-output`,
created by the provider under an exposed home underlay with no checkout above it,
when the checkout's mount fails and the flag is absent;
the design's gated-descendant claim becomes a report rather than a gate.

#### Ranking of V4

Ranking: A > B.

A over B: the hazard arm is the difference, and it is the failure the design exists to refuse.

What the review changed: the ranking stands and gains four conditions, all folded in:
who mounts the dataset (choice V9), the ordering before sessions, the hazard arm as a run rather than an inference,
and the property change as the owner's act.

### V5. How the propagation window is closed

#### Option A: the gate's own first act makes the checkout private

This design's answer: `mount --make-rprivate <checkout path>`, then the propagation read reports `private`
(kept log `evidence-green-closing.log`, `SWEEP propagation=private`).

What it delivers: one step, in the same pass that verifies, measured in every green boot.

What it costs: a descendant mounted by the provider before the step is a child of a shared parent for that window,
which V4 removes.

#### Option B: order each descendant after the gate's propagation step

What it delivers: the window closed by ordering rather than by a step.

What it costs: the provider cannot be ordered, so only its own scan can be prevented, which is V4.

#### Ranking of V5

Ranking: A > B.

A over B: B has no mechanism the provider obeys, and A is measured and already the design's.

What the review changed: nothing in the ranking.
With V4 adopted the window has no descendant in it at all, which strengthens A rather than replacing it.

### V6. What carries the checkout's readiness to the account

#### Option A: the account's services require the boot unit, named one by one

This design's answer. The stand-in consumer, ordered `Requires=` and `After=` and
`Before=systemd-user-sessions.service`, ran and saw the checkout, the private propagation and the attached slot
(kept log `evidence-green-closing.log`).

What it delivers: a real prerequisite, visible in the journal per service, and independent of login.

What it costs: it protects nothing about login;
`boot-reconcile` exits zero when clones fail, so a clone failure does not reach a consumer through the requirement;
and the service list is still to be named.

#### Option B: gate the account's user manager

A drop-in on `user@<uid>.service` with `Requires=` and `After=` on the boot unit.

What it delivers: everything the account starts is behind the gate.

What it costs: no design here for its ordering against `systemd-user-sessions.service`,
or for a refused boot that would then leave the account entirely without a session;
and a failed gate would remove far more than the checkout.

#### Option C: rely on login ordering alone

No consumer requirement; sessions start after the gate's job by the gate's own `Before=`.

What it delivers: nothing to name.

What it costs: measured, sessions start after a failed gate
(kept logs `evidence-flag-removed.log`, `evidence-key-removed-v4.log`),
so a session begins with no checkout and no signal.

#### Ranking of V6

Ranking: A > B > C.

A over B: A fails only what needs the checkout and leaves the session usable;
B's blast radius is the whole session and has no design for the ordering it would need.

B over C: C carries nothing, while B at least refuses something.

What the review changed: the ranking stands;
the review added the measured point that a failed gate does not hold sessions.

### V7. What `doctor` and setup report

#### Option A: the whole set

This design's answer, in plain words:
which of the three provider states the machine is in;
whether a key-loading unit exists and its name;
every dataset under the checkout path whose `canmount` is not `noauto`;
the boot unit's start bound and its source;
the provider drop-in and the boot unit's `ExecReload=`, with the note that a copy systemd has not reloaded is not seen;
the propagation service and its drop-in, installed only beside a boot unit that can reload;
the hook's name being one ZED's scan accepts;
and the installed file's owner, mode, ancestors, `--version` and digest.

What it delivers: every state a boot depends on is one command away,
and each finding prints the command that repairs it.

What it costs: a long report, and every line must be kept true as the design changes.

#### Option B: the provider state only

One line, `provider: zfs-mount.service`.

What it delivers: a short report.

What it costs: it reads green on a machine with the wrong `canmount`, no `ExecReload=`, or a hook name ZED ignores.

#### Ranking of V7

Ranking: A > B.

A over B: the states that make a boot fail are exactly the ones B leaves unread.

What the review changed: A stands and its list grew by the provider drop-in, `ExecReload=`, the start bound,
the real key unit name, the hook name's legality and the provider-state reading.

### V8. What bounds a start

#### Option A: a sourced `TimeoutStartSec=`, with the ordering before sessions kept

This design's answer. `TimeoutStartSec=20` on a hung start produced `Failed with result 'timeout'`,
then the stop pass's line, then `Starting systemd-user-sessions.service` in the same instant
(journal quotes in "What a hung start costs").

What it delivers: the gate's protection is kept and its cost is bounded, and a hung pass is a failed unit rather than a
stuck boot.

What it costs: no measurement exists yet, so the value is step 22's or the owner's to supply;
a bound that is too tight kills a reconcile mid-attach and then runs the stop pass;
and a timeout inside `ExecStartPre=` reaches the stop path whatever the dash prefix says,
so the probe needs its own shorter deadline.

#### Option B: a provisional fixed bound by the owner's decision

`TimeoutStartSec=5min`, recorded as the owner's ceiling rather than as a measurement.

What it delivers: a bounded hold now, with one number to revisit.

What it costs: an unsourced number, which the project's rule for shipped limits forbids unless an owner decision is its
source.

#### Option C: no bound, and the ordering line withheld until a bound exists

The accepted start-probe design's deferral, applied to the unit's ordering rather than to the probe.

What it delivers: no unsourced number and no unbounded hold.

What it costs: while the bound is missing, nothing makes the checkout ready before a session,
which is the unit's purpose.

#### Option D: no bound and the ordering kept

The unit with no `TimeoutStartSec=`, whose manager default is `infinity` (kept log `start-bound.log`).

What it delivers: no number to defend, and no reconcile killed mid-attach.

What it costs: a hung reconcile holds every login with no limit,
which is a boot nobody can log in to and only a person can clear.

#### Ranking of V8

Ranking: A > B > C > D.

A over B: A's ceiling is evidence and B's is a promise,
and the project's own rule asks for evidence beside a shipped limit.

B over C: both stay bounded or bounded-for-now, and B keeps the readiness protection the unit exists for.

C over D: D ships an unbounded login hold,
which is the one outcome a boot unit ordered before the sessions must not have.

What the review changed: D was this design's earlier position, inherited from the accepted start-probe design,
and the review's M1 named the unbounded hold. The ranking is new and D is now last.

### V9. Who mounts a dataset under the checkout path

#### Option A: one unit per dataset, after the gate and before sessions, running the handler's mode

This design's answer. The measured stand-in's unit and call, with the result
`NESTED mounted=zfsstep19/nosnap/vm-builder-output` after the verified checkout
(kept log `evidence-green-closing.log`).

What it delivers: a consumer can require exactly the mount it reads, which is what a `vm-builder` service needs;
each mount has its own journal entry and its own failure; and the ordering before sessions is per unit and measured.

What it costs: one unit per dataset, generated by setup and reported by `doctor`;
each new dataset under the path needs its unit.

#### Option B: the boot unit's own reconcile pass mounts them, one explicit `zfs mount` each

What it delivers: one unit, one ordering edge, and the mount in the same process as the verification that authorizes it.

What it costs: a consumer cannot require one descendant's readiness, only the whole pass's;
and the pass's start bound then has to cover every descendant's mount.

#### Option C: one unit per dataset running `zfs mount <dataset>` directly

What it delivers: the shortest unit text.

What it costs: no verification handshake, so a unit that starts early mounts a descendant under an unverified checkout,
and the design's rule that root's mounts come from the one program is broken.

#### Ranking of V9

Ranking: A > B > C.

A over B: A is the only option where a consumer can require the exact mount it reads,
and the plan's sentence already names a unit per path descendant.

B over C: B keeps the verification that authorizes the mount; C does not.

What the review changed: A is new as a requirement.
The earlier round measured a gated unit, but the earlier text did not say who mounts the dataset or by what call,
and the review's M2 named both gaps.

## Edits this design suggests

Suggestions only; this round edits neither record.

To plan step 19, "Open items", which reads:

> Two parts the design leaves open block its acceptance arms:
> the checkout provider stays conditional in "One-time move and checkout boot protection", which names "the
> protected `home-user-Monochromatic.mount` if using a generated mount unit, or `zfs-mount.service` where it mounts
> the checkout", and "Checkout boot mount protection" says to prove the failure control and production ordering
> before enabling the gate.

the suggestion is to replace that sentence with the fixed provider, the key unit and a pointer to this design:

> The checkout provider is fixed on a machine of this host's shape as `zfs-mount.service`, whose `zfs mount -a`
> mounts the checkout because the ZFS mount generator writes no unit here;
> where the generator's cache file exists and names the checkout dataset, the provider is the generated
> `<escaped checkout path>.mount` instead.
> The checkout's key-loading unit is the generator's `zfs-load-key@<escaped encryption root>.service` where one
> exists, and the provider's status is not a readiness signal for the checkout.
> The failure control, the production ordering, the hazard arm and the route-owned alternative are measured in
> `doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`, which also fixes the units'
> whole text, their start bound, the key and provider drop-ins and the exclusion every dataset mounted under the
> checkout path needs.

To plan step 19, "What it builds", the sentence "The gate that orders every path descendant after a verified
checkout mount" needs the condition this design measured:

> The gate that orders every path descendant after a verified checkout mount, where each such dataset carries
> `canmount=noauto` so that the automatic mount cannot mount it first, and where a unit per descendant is ordered
> after the gate and before `systemd-user-sessions.service`.

To the design's "Boot and shutdown unit", the two bullets that name the provider and the legacy alternative need the
same substitution, and the paragraph beginning "The decided provider must require the actual home and key-loading
providers" needs four additions:

- the learned mount unit is ordered with `RequiresMountsFor=` and never required, with the measured reason;
- the provider is named by the unit that performs the mount, not by the mount unit systemd learns afterwards;
- the key unit is the generator's instance unit, named for the encryption root the checkout's dataset needs;
- a machine with neither provider refuses at boot and says so, `doctor` reports which state it found, and a machine with
  no key-loading unit says that the key must be loaded before the provider runs.

To the design's "One-time move and checkout boot protection", the paragraph that begins "The account can replace
`~/Monochromatic` while it is unmounted, regardless of the entry's owner" needs the ancestor condition:

> The account can replace an ancestor of the checkout path while the checkout is unmounted, wherever it can write
> that ancestor's parent; the immutable covered entry refuses the entry's own rename, removal and inner writes, so
> the protection holds only while no directory above the entry's parent, up to the home dataset's root, is one the
> account can write. At `/home/user/Monochromatic` that holds: `/home` is root's, and `/home/user` is the account's
> but its own parent is not. At a deeper account-owned path such as `~/projects/Monochromatic` it does not hold, and
> the entry belongs at a path whose ancestors above its parent are not the account's.

and the paragraph ending "The probe did not replace an ancestor of the protected entry or exercise unattended boot."
can say what now has been run:

> Item 8's two gaps are measured in the guest of
> `doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`:
> an unattended boot with the checkout's mount made to fail wrote nothing to the covered entry and created no
> mountpoint folder there while the exclusion was in place, the same boot without the exclusion created the folders
> in the home underlay and mounted the output dataset into them, and an account-owned ancestor was renamed with a file
> planted at the entry's path while a root-owned ancestor refused it.

To the design's "Checkout boot mount protection" decision, whose last sentence reads "Failed ordinary mount and
unattended boot remain unmeasured.", the measurement now exists and the sentence becomes a pointer to this design's
"The failure controls".

To the design's `ExecStart`, `ExecStopPost` and hook text, the program name follows the one-program decision
(`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`);
this design writes them against `<root-file>` and `<group-word>` and says so.

To the lost-event design's "The hook, in one form a sibling can cite", no edit:
this design installs that drop-in and that `ExecReload=` unchanged, and the guest arm measured both.

To the propagation design's "The propagation service and its drop-in", no edit of the two files;
this design installs them, and its guest arm supplies the two measurements that design listed as unshown:
`Upholds=` on a learned mount unit, and the service's activation and binding to that unit.

## Facts and what each rests on

The facts this document relies on, each with its basis, so a reader can re-run the ones that matter.

- F1. The checkout path has no mount of its own on this host; `/home` is the home dataset, `shared`.
  Basis: host reads, `keep/host-reads.log`.
- F2. The ZFS mount generator is installed, `/etc/zfs/zfs-list.cache` is absent,
  and `/run/systemd/generator` holds no ZFS mount unit. Basis: host reads.
- F3. `zfs-zed.service` is disabled and inactive on this host. Basis: host reads.
- F4. `zfs-mount.service` runs `/usr/bin/zfs mount -a` with `Before=local-fs.target`; `zfs-import-cache.service` runs
  `zpool import -c /etc/zfs/zpool.cache -aN $ZPOOL_IMPORT_OPTS`;
  `ZPOOL_IMPORT_OPTS` is empty; all four ZFS units are enabled.
  Basis: host reads.
- F5. `/usr/lib/systemd/system/zfs-load-key.service` is a `/dev/null` symlink owned by `zfs-utils 2.4.4-3`, there is no
  `zfs-load-key@.service` file, and the unit is `masked`. Basis: host reads.
- F6. The host's pool key is a passphrase prompted at import, its keystatus is available,
  and the home dataset inherits it.
  Basis: host reads.
- F7. `home.mount` is learned from the mount table; the checkout's own unit name is `not-found` on this host.
  Basis: host reads.
- F8. `/home` is root's `0755`, `/home/user` is the account's `0700`, the checkout path is the account's `0755`.
  Basis: host reads.
- F9. `zfs-space-guard.service` is `After=zfs-mount.service` with a five-minute timer. Basis: host reads.
- F10. This host runs systemd `262.2-1-arch`. Basis: host reads.
- F11. The guest ran systemd `258.11-1.fc43` on kernel `6.17.1-300.fc43` with OpenZFS `2.4.4-1`,
  Fedora 43, SELinux permissive.
  Basis: kept log `prepare.log`.
- F12. The guest's source install placed `zfs-load-key.service -> /dev/null` and no `zfs-load-key@.service`
  template; the generator writes instance key units.
  Basis: kept logs `prepare.log` and `guest-load-key-unit.log`, and the source reading of
  `etc/systemd/system-generators/zfs-mount-generator.c` (the `keyloadunit` block).
- F13. The whole unit set boots green: key units, provider, gate, descendant unit, consumer, sessions in that order.
  Basis: kept logs `evidence-green-final.log`, `evidence-green-closing.log`.
- F14. `canmount=noauto` makes `zfs mount -a` skip a dataset and the explicit `zfs mount <dataset>` mounts it.
  Basis: kept logs `evidence-green-closing.log`, `evidence-hazard-v4.log`, and the source reading of
  `cmd/zfs/zfs_main.c` `share_mount_one`'s `canmount` table with `explicit` distinguishing `-a` from a plain mount.
- F15. With the checkout's key file away and the exclusion in place, the provider finishes `Result=success`,
  `ExecMainStatus=0`, with the checkout absent. Basis: kept log `evidence-key-removed-v4.log`.
- F16. The checkout's key unit failing fails the boot unit's start job with `dependency` and nothing of the pass runs.
  Basis: kept log `evidence-key-removed-v4.log`, journal lines at `1795502972.946105` to `.956144`.
- F17. The flag's removal is refused with `covered-entry-not-immutable`, exit status two, and both dependents fail.
  Basis: kept log `evidence-flag-removed.log`.
- F18. With the flag absent, the checkout's mount failing and the descendant `canmount=on`,
  the provider creates the mountpoint folders in the home underlay and mounts the descendant there.
  Basis: kept log `evidence-hazard-nested-auto.log`.
- F19. The same with the descendant `canmount=noauto` mounts nothing and creates nothing.
  Basis: kept log `evidence-hazard-v4.log`.
- F20. The route-owned unit mounts the checkout explicitly in a green boot. Basis: kept log `evidence-route-owned.log`.
- F21. The route-owned unit without a key requirement, with the key file away, prints
  `cannot mount '…': encryption key not loaded` and fails with exit status one.
  Basis: kept log `evidence-route-owned-bare.log`.
- F22. `RequiresMountsFor=` produces `After=` on every prefix mount unit and `Requires=` only on the fragmented one.
  Basis: kept log `requires-mounts-for-dependencies.log` and the source reading of
  `src/core/unit.c` `unit_add_mount_dependencies` (the `m->fragment_path` branch) and `src/core/mount.c`
  `mount_add_mount_dependencies`.
- F23. A boot unit with no edge to the provider still starts after it and passes.
  Basis: kept log `evidence-no-provider-edge.log`.
- F24. `Requires=` plus `After=` makes a provider restart take the boot unit through its stop pass and start it again.
  Basis: kept log `restart-provider.log`.
- F25. `Wants=` plus `After=` does not; the boot unit stays active. Basis: kept log `wants-control.log`.
- F26. `reload-or-restart` on the active boot unit runs `ExecReload=` and no stop pass.
  Basis: kept logs `reload-gate.log` and `reload-gate-after.log`.
- F27. The provider drop-in's request runs after a provider start and produces a reload of the boot unit.
  Basis: kept log `wants-control.log`.
- F28. `Upholds=` and `BindsTo=` work on a mount unit learned from the mount table:
  the service starts on the mount unit's activation and stops on its deactivation.
  Basis: kept log `propagation-cycle.log` and `dependencies-read.log`.
- F29. `systemd-analyze verify` reports only the propagation service's `BindsTo=` on the not-yet-existing mount unit.
  Basis: kept log `units-install.log`.
- F30. A `TimeoutStartSec=20` bound kills a hung start, records `timeout`, runs the stop pass and lets sessions start.
  Basis: kept log `evidence-bound.log`, journal lines `1795503339.770182` to `1795503339.812539`, and
  `evidence-bound-late.log`.
- F31. The `ExecStartPre=` dash prefix keeps `ExecStart` after a non-zero exit, does not after a timeout,
  and a timeout there sends the unit to its stop action. Basis: kept log `start-bound.log`.
- F32. A oneshot's default start timeout is `infinity`. Basis: kept log `start-bound.log`,
  `TimeoutStartUSec=infinity`, and the manual's `TimeoutStartSec=` paragraph.
- F33. `Requires=` on the learned mount unit alone made the boot unit's job vanish silently and the sweep never ran.
  Basis: kept log `evidence-learned-unit-only.log`.
- F34. The same require with the provider also required ran green, and the provider drop-in's request reported
  `Unit … not found`. Basis: kept log `evidence-learned-unit-required.log`.
- F35. Sessions start after a failed boot unit. Basis: kept logs `evidence-flag-removed.log`,
  `evidence-key-removed-v4.log`, `evidence-bound.log`.
- F36. The propagation service starts before the boot unit in a boot where both are wanted.
  Basis: kept log `evidence-green-closing.log`, journal lines `1795503632.608716` and `1795503632.624819`.
- F37. `boot-reconcile` exits zero when clones fail and only a refusal returns an error.
  Basis: source reading of `boot_reconcile_mode.rs:158` to `:164` and `:127` to `:136`.
- F38. The built `shutdown-unmount` always refuses. Basis: source reading of `shutdown_unmount_mode.rs`, `run`.
- F39. A no-argument start is the attaching sweep and no `sweep` word exists.
  Basis: source reading of `handler_mode_arguments.rs`, `parse`.
- F40. ZED starts a zedlet by file name with no arguments, and the name must carry the class prefix followed by a
  non-alphabetic character. Basis: `zed.8`, and the root-modes design's replay of `cmd/zed/zed_event.c`.

## Sources read

On this host, read-only, 2026-11-24.
`keep/host-reads.log` holds the whole output:
`systemctl --version`, `systemctl cat` for `zfs-import-cache.service`, `zfs-mount.service`, `zfs-space-guard.service`,
`zfs-load-key.service`'s symlink and owner through `pacman -Qo`,
`systemctl --no-ask-password is-enabled` and `list-unit-files` for the ZFS units,
`systemctl --no-ask-password show -p Id -p LoadState -p FragmentPath -p SourcePath` for `home.mount`,
`systemctl --no-ask-password show -p Wants` for `zfs.target` and `zfs-import.target`,
`grep` of `/etc/default/zfs` for `ZPOOL_IMPORT_OPTS`,
`zfs get -o name,property,value encryption,keyformat,keylocation,keystatus` for the pool and its home dataset,
`systemd-escape --suffix=mount --path` for the checkout path, and `stat --format` for `/home`, `/home/user` and the
checkout path.

In the guest, from the sources that shipped with the arms:
OpenZFS `zfs-2.4.4` `etc/systemd/system/zfs-mount.service.in`,
`etc/systemd/system-generators/zfs-mount-generator.c` (the mount unit's `Before=zfs-mount.service` and the key unit's
`keyloadunit` block),
`cmd/zfs/zfs_main.c` `share_mount_one` and `share_mount_one_cb` (the `canmount` table, the unavailable-key skip, and
`explicit=B_FALSE` for `-a`),
`man/man8/zed.8` on zedlet names,
and systemd 258's own manuals as `systemctl show` reported the defaults.
Systemd v262's `src/core/transaction.c`, `src/core/unit.c`, `src/core/mount.c` and `man/systemd.unit.xml`
quotes are the gate review's, kept in its `keep/systemd-v262-quotes.log`, and each is cited where it is used.

## Measurements and logs

This round's folder:
`~/temp/agent/zfs-step-19-design-after-its-gate-review-2026-11-24/`.

- `keep/step19-revision-driver.ts`, the driver that pushes and runs each guest step in a capped
  `systemd-run --user` scope and writes one log per step.
- `keep/guest-scripts/`, every guest step as its own script, listed under "What it ran".
- `keep/logs/`, every step's whole output, named for its arm.
- `keep/host-reads.log`, the host reads with their exact output.
- `keep/state-so-far.md`, the round's running state.

The guest was created through `mvm --backend libvirt` at 2026-11-24T06:36:36Z and destroyed through `mvm`
at 2026-11-24T07:05:42Z; `mvm list` no longer reports it.

## Revisions

The writing round of 2026-11-24 revised this document whole after its gate review.
What changed, in the order the review gave it, is "What this revision changed".
The earlier text's failure control, its hook, its quoted journal lines and its ancestor condition are all replaced;
the provider rule, the exclusion decision and the ranking of every choice are kept, with new reasons where the review
showed the old ones false.

No cross-vendor review of this revision has been received, so none is folded in, and the document stays unapproved and
non-final.

Facts F1 to F10 rest on host reads, F11 to F36 on kept guest logs and their journals, F37 to F40 on source readings;
nothing in this document rests on an unlabelled recall.
