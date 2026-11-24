# Which unit mounts the checkout, and the ordering the boot gate needs

Design proposal by an agent, **unapproved and non-final**.
It answers the two parts plan step 19 leaves open
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, "Step 19", "Open items"):
the provider stays conditional in the worktree design's "One-time move and checkout boot protection",
and the same design's "Checkout boot mount protection" says to prove the failure control and the production
ordering before the gate may be enabled.
It designs no wrapper code, edits nothing under `doc/decision/` or `doc/handover/`, and applies nothing to any
machine.
The two sentences of the plan and the design that it amends are quoted under
"Edits this design suggests", as suggestions to those two records, not as changes made.

What it ran, and where the evidence is:
one throwaway `mvm` guest (`zfs-step-19-boot-order-2026-11-23`, Ubuntu 24.04, kernel `6.8.0-147-generic`,
systemd `255` (`255.4-1ubuntu8.19`), OpenZFS `2.4.4-1` module and userspace built from the release tarball),
a 1 GiB file-backed pool laid out like this host's, fifteen unattended boots, three of them ended by a reboot whose
stop pass the round read back, and three arms that run without rebooting.
Every run is logged whole under
`~/temp/agent/zfs-step-19-checkout-boot-mount-provider-and-ordering-2026-11-23/keep/guest-logs/`.
Host reads are read-only and kept in that folder's
`keep/host-reads-for-the-provider-2026-11-23.log`.
Nothing was measured on this host's pool: no host dataset was mounted, unmounted, imported, exported or changed,
and no service manager on this host was asked to change anything.

The stand-ins are not the product.
The gate's program in the guest is a shell program that performs the checks this design describes
(read the checkout's mount row, read the covered entry's immutable flag through a second view of the home dataset,
set the checkout private, then attach one slot by bind mount), because the root mode that will do it is not built.
So every result below is about the **units, their ordering edges and the failures they carry**, and about the
kernel and systemd behaviour they rest on; none of it is a result about the wrapper's own handler.

## The answer in brief

- **The provider is `zfs-mount.service` on a machine of this host's shape.**
  Its `ExecStart=/usr/bin/zfs mount -a` is what puts the checkout's dataset at the checkout path, because this host's
  ZFS mount generator writes no unit for any dataset here: it reads `/etc/zfs/zfs-list.cache/<pool>`, and that file
  does not exist.
  The lost-event record already names the same unit for the same machine
  (`doc/decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md`, "Which trigger runs after the checkout
  becomes ready": "the accepted design obliges the checkout's provider, on this machine `zfs-mount.service`").
  This design shows the premises of that sentence on the host and gives the rule that replaces the conditional.
- **What decides it on another machine** is whether the generator writes a mount unit for the checkout dataset.
  Where it does, the provider is that unit and it can be required; where it does not, the provider is the unit that
  runs `zfs mount -a` at boot. Where neither is enabled, nothing mounts the checkout at boot and the route refuses.
- **The gate is the handler's own verification, not the provider's success.**
  Measured: `zfs-mount.service` finishes with exit status zero while the checkout is not mounted at all, and it
  finishes with exit status one while the datasets are partly mounted.
  Neither status says anything about the checkout.
- **Never require a mount unit systemd learned from the mount table.**
  Measured: a `Requires=` on it made the dependent's start job fail with `No mount parameters to operate on`, and at
  an unattended boot the gate's job vanished without a journal line, so the sweep never ran and account sessions
  opened.
- **A `Requires=zfs-mount.service` carries a real failure and a pull-in, and neither is a mount.**
  Measured: with the checkout's key taken away the provider failed, and that failure failed the gate's job, so
  nothing ran; and on a boot where nothing wanted `zfs.target` the requirement did bring the provider into the
  transaction, where its own condition skipped it and the gate refused.
  Ordering alone the line does not buy on this systemd (measured).
- **The path descendants the design wants gated are mounted by the provider itself** unless they are excluded from
  the automatic mount.
  Measured: the nested output dataset was mounted by `zfs mount -a` under the checkout before any handler ran, and a
  gated unit that tried afterwards found `filesystem already mounted`.
  With `canmount=noauto` on that dataset the provider skipped it and the gated unit mounted it after the verified
  checkout. The design's gate therefore needs that exclusion to exist at all.
- **The immutable entry stops the entry's own replacement and nothing above it.**
  Measured: with the checkout unmounted, the account could not rename, remove or write into the entry; it could not
  rename a parent whose own parent is root-owned; it could rename a parent it owns, and a file it planted at the new
  path stayed there. The design needs that condition said out loud.

## The provider on this host, and how the reads decide it

Read 2026-11-23, read-only, whole output in `keep/host-reads-for-the-provider-2026-11-23.log`.

The checkout path has no mount of its own yet, which the design already says:

```text
$ findmnt --output TARGET,SOURCE,FSTYPE,PROPAGATION --target /home/user/Monochromatic
TARGET SOURCE                  FSTYPE PROPAGATION
/home  zpcachyos/ROOT/cos/home zfs    shared
```

The ZFS mount generator exists and writes nothing for this pool, because its input file does not exist:

```text
$ ls --directory /usr/lib/systemd/system-generators/zfs-mount-generator
/usr/lib/systemd/system-generators/zfs-mount-generator
$ ls --almost-all /etc/zfs/zfs-list.cache
ls: cannot access '/etc/zfs/zfs-list.cache': No such file or directory
$ ls --almost-all /run/systemd/generator
boot.mount  dev-zram0.swap  ... (no ZFS mount unit)
```

`zfs-mount-generator(8)` says why: "information on ZFS mountpoints must be stored separately. The output of
`zfs list -Ho name,<every property above in order>` for datasets that should be mounted by systemd should be kept at
`/etc/zfs/zfs-list.cache/poolname`", kept up to date by the `history_event-zfs-list-cacher.sh` zedlet.
The zedlet is installed here (`/etc/zfs/zed.d/history_event-zfs-list-cacher.sh` resolves), but its carrier is not
running, so nothing keeps such a file:

```text
$ systemctl --no-ask-password is-enabled zfs-zed.service; systemctl --no-ask-password is-active zfs-zed.service
disabled
inactive
```

What mounts this host's datasets instead is the pair of units the OpenZFS package installs:

```text
$ systemctl --no-ask-password cat zfs-import-cache.service
ExecStart=/usr/bin/zpool import -c /etc/zfs/zpool.cache -aN $ZPOOL_IMPORT_OPTS
$ systemctl --no-ask-password cat zfs-mount.service
Before=local-fs.target
After=zfs-import.target
Conflicts=umount.target
Before=umount.target
[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/usr/bin/zfs mount -a
[Install]
WantedBy=zfs.target
$ systemctl --no-ask-password is-enabled zfs.target zfs-import-cache.service zfs-mount.service zfs-load-key.service
enabled
enabled
enabled
masked
```

`-aN` is import without mounting, so the import unit brings the pool in and `zfs mount -a` is what mounts: every
dataset with `canmount=on` and a path `mountpoint`, in one command, before `local-fs.target`.
`zfs-load-key.service` is masked here because this host's pool key is entered when the pool is imported
(`zfs get -o name,property,value keystatus zpcachyos` answers `available`), so this host has no key-loading unit for
the checkout's provider to order after.

systemd does hold a mount unit for every dataset here, but it learned each of them from the mount table, not from a
fragment:

```text
$ systemctl --no-ask-password show -p Id -p FragmentPath -p SourcePath home.mount
Id=home.mount
FragmentPath=
SourcePath=/proc/self/mountinfo
```

and the unit the checkout will get, escaped as a path, does not exist yet:

```text
$ systemd-escape --suffix=mount --path /home/user/Monochromatic
home-user-Monochromatic.mount
$ systemctl --no-ask-password show -p LoadState home-user-Monochromatic.mount
LoadState=not-found
```

(`systemd-escape --suffix=mount /home/user/Monochromatic` without `--path` escapes the string literally and yields a
leading dash, `-home-user-Monochromatic.mount`, which names no unit. Every unit name in this design comes from the
`--path` form.)

So on this machine the checkout's provider, in the design's own words "the protected
`home-user-Monochromatic.mount` if using a generated mount unit, or `zfs-mount.service` where it mounts the
checkout", is `zfs-mount.service`.
The other branch is not available here at all: there is no generated unit to protect, and a unit learned from the
mount table cannot be started (measured below, and the sibling design's F36 states the source read).

## What decides the provider on another machine

The rule, in the order the facts decide it:

1.  The provider is the generated mount unit,
    `<escaped checkout path>.mount` from `systemd-escape --suffix=mount --path`,
    when the distribution installs the ZFS mount generator and
    `/etc/zfs/zfs-list.cache/<pool>` exists and lists the checkout dataset.
    It is a unit with a fragment, so `Requires=` and `After=` on it both work,
    and systemd mounts the checkout as that unit's own job.
    Its own ordering is the generator's,
    `After=zfs-import.target` and the key-loading unit,
    `Before=zfs-mount.service`
    (OpenZFS `zfs-2.4.4`, `etc/systemd/system-generators/zfs-mount-generator.c`, and the sibling design's F15).
2.  Otherwise the provider is the unit that runs `zfs mount -a` at boot,
    `zfs-mount.service` in every OpenZFS packaging this route's records name.
    It mounts the checkout as a side effect of one command over every dataset,
    so the route names the actor and not a mount unit,
    because the mount unit appears only after the actor ran.
3.  Where neither is enabled, nothing mounts the checkout at boot.
    The route must refuse at boot and name what is missing,
    rather than gate on a mount that no unit produces.
    `doctor` reports which of the three states the machine is in.

What a reader needs to check on a machine, then, is three facts and not two:
the generator's cache file for the pool,
the state of the unit that runs `zfs mount -a`,
and the key-loading unit where the checkout's own key is not loaded at import.
This design's units name the last two, and the first replaces them when it exists.

## The units' ordering lines

Two installed files, and the ZED hook that is the design's third trigger.
The program they start is a **placeholder**, because the owner answered "One program only, ever" on 2026-11-23
(`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`):
the root modes become modes of `monochromatic-git-policy-cli`, and "How root reaches the modes (an argument, the name
the program is started under, or another way), what the ZED hook and the units name" is left to a design of its own.
`<root-mode-entry>` below stands for exactly that, the invocation that reaches a root mode, and `<boot-reconcile>`,
`<shutdown-unmount>` and `<sweep>` are the mode names the design already uses.
`mount-git-worktree-clone-datasets` appears nowhere in a line this design installs; the name is step 5's and the
one-program decision retires it.

```ini
# /etc/systemd/system/reconcile-git-worktree-clone-mounts.service
[Unit]
Description=Reconcile the managed worktree clone mounts
Requires=zfs-mount.service
After=zfs-mount.service zfs-import.target
After=zfs-load-key.service
After=home.mount
After=home-user-Monochromatic.mount
Before=systemd-user-sessions.service
Before=umount.target
Conflicts=umount.target

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=<root-mode-entry> <boot-reconcile>
ExecReload=<root-mode-entry> <boot-reconcile>
ExecStopPost=<root-mode-entry> <shutdown-unmount>

[Install]
WantedBy=multi-user.target
```

Why each line, and what the runs said about it:

- `Requires=zfs-mount.service`.
  What it buys is the failure path and a pull-in, and neither of them is a mount.
  Measured, with the checkout's key taken away: the provider failed with exit status one, and systemd failed this
  unit's start job with `Job reconcile-git-worktree-clone-mounts.service/start failed with result 'dependency'`, so
  the handler never ran and no slot, no nested mount and no account consumer ran either.
  Measured on a boot where nothing wanted `zfs.target`: this line did bring `zfs-mount.service` into the
  transaction, and the provider was then **skipped** by its own
  `ConditionPathIsDirectory=/sys/module/zfs` because nothing had ordered or pulled the module load and the import
  before it, so the gate ran and refused.
  A requirement is therefore not a provisioner: the machine must also want the import and the module's load, which
  is what `zfs.target` does, and the refusal the gate printed is the correct outcome on a machine that does not.
  Replace this name with the generated mount unit where the generator writes one.
- `After=zfs-mount.service zfs-import.target`.
  Honest limit: on this systemd it added nothing to the startup order, and the negative control shows it.
  A unit with `DefaultDependencies=yes` is `After=basic.target` by default
  (`systemd.service(5)`, "Default Dependencies"), and `basic.target` is after `local-fs.target`, which the provider's
  `Before=local-fs.target` already precedes.
  Measured: the same gate unit with **no** edge to the provider at all still started after the provider and passed,
  in the same boot shape, with `Finished zfs-mount.service` at `.210` and `Starting reconcile-...` at `.214` of the
  same second.
  The line stays because it states the dependency a reader audits and because it is what `systemd-analyze verify`
  checks, and this design does not claim more for it.
- `After=zfs-load-key.service`.
  The key-loading provider, named where one exists.
  This host masks that unit and has nothing to order after, so the line is inert here and `doctor` should report
  whether the machine has a key-loading unit at all.
  The guest models the other shape: with a key-loading unit in the boot, the checkout mounted; with its key file
  missing, that unit failed and the provider failed with it.
- `After=home.mount` and `After=home-user-Monochromatic.mount`.
  Ordering only, never `Requires=`, and only the stop half can use them.
  At the first boot that mounts the checkout, neither unit exists when the transaction is built
  (`LoadState=not-found`, measured on the host), so an edge on them cannot delay the gate's start;
  the sibling design's F12 and F38 already say systemd learns these mounts after they appear.
  What the edges do work is the stop pass: at shutdown the gate stops before the provider and before `umount.target`.
- `Before=systemd-user-sessions.service`.
  Measured in every green boot: the pre-session witness ran, found the checkout mounted, private, and its slot
  attached, and `systemd-user-sessions.service` started only after the gate's job had ended.
  One boot, whose first entry is `02:10:35` (`keep/guest-logs/journals-per-boot.log`):
  `Finished reconcile-...` at `1795486241.197907`, the witness `Finished` at `.260876`,
  `Starting systemd-user-sessions.service` at `.298290` and `Finished` at `.344026`.
- `Before=umount.target` with `Conflicts=umount.target`.
  Measured in the same boot's shutdown: `Stopping reconcile-...` at `1795486310.937130`, deactivated at `.958381`,
  then `zfs-mount.service: Deactivated successfully` at `1795486311.645979`, then the checkout's own learned mount
  unit `Unmounting` at `.700170` and `Unmounted` at `.706510`.
  The stop pass therefore ran before the provider's stop and before the checkout's own unmount, and its own durable
  line landed in `/var/log/step19-sweep-stop.log` at uptime `67.40`, which is the same pass.
- `WantedBy=multi-user.target` with default dependencies.
  This is what makes the gate part of the boot at all; without it the unit runs only on a manual start.
  It also carries the implicit `After=basic.target` that the ordering paragraph above explains.

The ZED hook, which reaches the same entry point.
Every zedlet this host installs is a `.sh` file under `/etc/zfs/zed.d`, so the hook is one too, and its whole body is
the one line that matters:

```sh
#!/bin/sh
# /etc/zfs/zed.d/history_event-reconcile_git_worktree_clone_mounts.sh
exec <root-mode-entry> <sweep>
```

The earlier text installed the handler's executable itself in that directory, which the one-program decision
retires along with the program name.

and the account-side rule the design already states, now with a unit to name:
a service of the account that needs the checkout carries `Requires=` and `After=` on
`reconcile-git-worktree-clone-mounts.service`, never on the checkout's learned mount unit.
Measured: such a unit, so ordered, ran and saw the checkout, its propagation and the attached slot;
when the gate's job failed, its own job failed with `result 'dependency'` and its program never ran.

## The failure control

What the design must show, and what the run showed.

The control state, built by removing the checkout's key file so that its dataset cannot be mounted while every other
prerequisite is exactly the production boot's:

```text
$ zfs get -o name,property,value keystatus zfsstep19/home/monochromatic-git-checkout
... keystatus unavailable
```

What happened, from the guest's journal of one unattended boot:

```text
zfs-load-key.service: Main process exited, code=exited, status=1/FAILURE
zfs[669]: cannot mount '/srv/.../Monochromatic/package/vm-builder/cli/output':
          failed to create mountpoint: Operation not permitted
zfs-mount.service: Failed with result 'exit-code'
Dependency failed for reconcile-git-worktree-clone-mounts.service
Dependency failed for git-worktree-nested-output-mount.service
Dependency failed for git-account-session-standin.service
reconcile-...: Job .../start failed with result 'dependency'
```

The assertions the design asks for, each measured, and one more the arm showed:

- The gate's mounts did not run.
  No slot mount exists, the nested stand-in never ran its program, and its unit's job failed with `dependency`.
- The sweep did not run.
  The gate's job failed with `dependency` and its program never started; nothing wrote to `/run/step19-evidence.log`.
- Nothing wrote to the covered entry.
  The home dataset's inventory after that boot is exactly the three directories it had before it
  (`d .../home`, `d .../home/account`, `d .../home/account/Monochromatic`), the immutable flag is still set
  (`----i-----------------`), and the nested dataset's own mount failed rather than creating folders inside the
  entry: `failed to create mountpoint: Operation not permitted`.
  That last line is worth keeping: the protected entry is also what stops the provider from writing mountpoint
  folders under the exposed home when the checkout's own mount fails.
- Account sessions did open.
  `Finished systemd-user-sessions.service` follows the failed jobs, which is why the design gates the account's
  services on the gate unit rather than on login: a plain login has no dependency on the gate and cannot be given
  one.

Two more control states, because the failure control is only worth reading beside them:

- **The provider can succeed while nothing is mounted, and this is the case that defeats gating on its status.**
  A boot with no key-loading unit at all, keys unavailable, produced
  `Finished zfs-mount.service - Mount ZFS filesystems` with `ExecMainStatus=0`, `zfs list` showing the checkout
  `mounted no`, and the gate then running and refusing with `SWEEP refused=checkout-not-the-dataset`.
  The provider's unit was `active (exited)` and its exit status zero with the checkout absent.
- **The verification's own refusal path, as a positive control.**
  With the immutable flag removed from the covered entry and everything else green, the gate ran and refused:
  `SWEEP covered=---------------------- ...` then `SWEEP refused=covered-entry-not-immutable`.
  Its unit failed with exit status two, and every dependent job failed with `dependency`.
  The check therefore can fail, which is what makes its pass in the green arm mean something.

## The gated mounts the provider performs itself

The design gates "nested mounts on verified success where path ordering cannot fail closed".
On this machine the provider mounts the checkout's path descendants itself, in the same `zfs mount -a`, and no
ordering edge can come before that:

```text
$ zfs mount -a      # in the guest, with the fixture at rest
zfs-mount-a-exit=0
TARGET                                       SOURCE                                    PROPAGATION
/srv/.../home                                zfsstep19/home                            shared
/srv/.../home/account/Monochromatic          zfsstep19/home/monochromatic-git-checkout shared
/srv/.../Monochromatic/package/.../output    zfsstep19/nosnap/vm-builder-output        shared
```

The nested dataset is mounted by the provider, under a checkout that is `shared` at that moment, before any handler
runs; in the green arm, the gated unit that followed the verified checkout found
`cannot mount 'zfsstep19/nosnap/vm-builder-output': filesystem already mounted`.
So the gate cannot order those mounts, and an ordering claim about them is empty.

What makes the gate real, measured: with the nested dataset set to `canmount=noauto`, the provider skipped it, the
gated unit mounted it after the verified checkout, and the mount appeared:

```text
SWEEP attached=registered 2026-11-24T02:22:39.012Z
NESTED start 2026-11-24T02:22:39.093Z
NESTED mounted=1 2026-11-24T02:22:39.146Z
```

The design therefore needs one sentence it does not have:
every dataset whose mountpoint lies inside the checkout path carries `canmount=noauto`,
the automatic mount skips it, and the gated unit or the handler mounts it after the checkout is verified.
Setup sets it; `doctor` reads it and reports the ones that would be mounted by the scan.
Without that, the gate protects the checkout's own mount and reports the descendants, and cannot prevent them.

This also settles what the design's `zfs mount -a` sentence must say about the propagation window:
the provider mounts those descendants while the checkout is still shared, so the private propagation the boot unit
sets afterwards is what makes them private, and their mount instances are children of a shared parent until then.
That is the sibling design's L4 from the other side, now measured rather than read.

## The covered entry, and the ancestor the design leaves unrun

Measured, with the checkout unmounted and the entry immutable, as the account:

```text
mv ~/Monochromatic ~/Monochromatic.moved          -> Operation not permitted     (the flag)
rmdir ~/Monochromatic                              -> Operation not permitted     (the flag)
touch ~/Monochromatic/planted                       -> Operation not permitted     (the flag)
mv <home>/account <home>/account.moved              -> Permission denied           (the parent, root-owned)
mv ~/projects ~/projects.moved                      -> succeeds                    (the parent, account-owned)
   then ~/projects/Monochromatic/planted exists
zfs mount zfsstep19/home/monochromatic-git-checkout -> Insufficient privileges    (the account)
```

So the immutable entry refuses exactly the entry's own replacement, removal and writing.
An ancestor above the entry is a different matter: if the account can write the directory that holds an ancestor,
it renames that ancestor wholesale and plants its own directory at the entry's path, and the flag on the directory
that moved away protects nothing.
Whether that is available depends only on ownership between the entry and the home dataset's root, and at this
host's decided path it is not:

```text
/home                      root:root     0755   (the home dataset's root)
/home/user                 user:user     0700   (renaming it needs write on /home)
/home/user/Monochromatic   user:user     0755   (renaming it is what the flag refuses)
```

The design's sentence, "The account can replace `~/Monochromatic` while it is unmounted, regardless of the entry's
owner", is therefore too strong for the protection it chooses: with the flag it can replace neither the entry nor any
ancestor above it at that path, and at a deeper account-writable path such as `~/projects/Monochromatic` it can
replace `~/projects` and the flag does not help.
The design needs that condition beside the verification step: the covered entry is protected while every directory
between it and the home dataset's root is one the account cannot write.

## What the runs show

`keep/guest-logs/journals-per-boot.log` holds every boot's journal for the units this design touches, one section
per boot, and each section starts with the boot's first entry time in Unix seconds.
The arms below are cited by that first entry time, `02:07:01` meaning the boot whose first entry is
`2026-11-24 02:07:01 UTC`, because boot indexes move as the guest reboots.

- `02:07:01`: the pool's cache file sat where the source install does not look, the import unit was skipped by its
  condition, and the provider nevertheless finished with exit status zero while nothing mounted.
- `02:08:49`: the import ran and the keys were unavailable, so nothing mounted; the provider finished with exit
  status zero, and the gate ran and refused (`SWEEP refused=checkout-not-the-dataset`).
  This is the boot that shows a provider's success is not readiness.
- `02:10:35`: the first green boot, with a key-loading unit in the boot.
- `02:12:01`: green, production lines. The checkout mounted by the provider over the immutable entry, the gate read
  its row, read the covered entry's flag through a second view of the home dataset, set the checkout `private` and
  attached the slot; the pre-session witness then saw all three; the nested gated unit ran but found the dataset
  already mounted. Log `guest-evidence-production-on.log`.
- `02:12:40`: green, with no edge to the provider at all, which is the negative control for the ordering claim.
  Log `guest-evidence-mount-unit-edge-only-on.log`.
- `02:13:06`: `Requires=` on the learned mount unit. The gate never ran, systemd logged nothing for it, the unit was
  left inactive with `Result=success`, and both dependents failed with `dependency`. Later in the same boot, the
  manual start with the mount up succeeded, and with the mount gone the same start answered
  `No mount parameters to operate on` and `Job ... failed with result 'dependency'`.
  Logs `guest-evidence-require-mount-unit-on.log` and `guest-stale-unit.log`.
- `02:15:00`: the immutable flag removed, everything else green. The gate ran and refused
  (`SWEEP refused=covered-entry-not-immutable`), its unit failed with exit status two, and both dependents failed.
  Log `guest-evidence-production-on.log` of that boot.
- `02:15:35`: the failure control, with the checkout's key file removed. The key-loading unit failed, the provider
  failed with exit status one, every dependent job failed with `dependency`, and the covered entry kept exactly the
  three directories it had and its flag. Log `guest-evidence-production-on.log` of that boot.
- `02:16:38`, `02:18:53` and `02:21:19`: green again, with the learned unit's two jobs run in them (stop with a
  child mount refused as `target is busy`, start refused as `No mount parameters to operate on`) and the ancestor arm.
  Log `guest-ancestor.log`.
- `02:19:35` and `02:20:46`: green, and the poweroff's stop order: `Stopping reconcile-...` then
  `zfs-mount.service: Deactivated successfully` then `Reached target umount.target`, with the stop program's own
  line in `/var/log/step19-sweep-stop.log` at uptime `67.40`.
  Log `guest-evidence-production.log`, section "previous boot stop order".
- `02:22:33`: the nested dataset set to `canmount=noauto`. The provider skipped it and the gated unit mounted it after
  the verified checkout (`NESTED mounted=1`). Log `keep/guest-logs/final-state-of-boot-0.log`.
- `02:28:04`: nothing wanted `zfs.target`, and the gate's requirement had no effect beyond pulling
  `zfs-mount.service` into the transaction to be skipped by its condition; the gate ran and refused.
  Log `guest-evidence-production-off.log`.

## What the runs do not show

- **Not the product handler.**
  The gate is a shell stand-in that performs the reads this design names; the wrapper's root mode, its ABI and layout
  gates, its readiness record and its start probe never ran.
  A refusal's exit status, its journal line and its effect on dependents are measured through the stand-in only.
- **Not this host's kernel or ZFS module.**
  The guest runs `6.8.0-147-generic` and OpenZFS `2.4.4-1`; this host runs `7.3.5-2-cachyos`. No ZFS or systemd
  behaviour that the two could share was compared between them.
- **Not this host's units.**
  The guest's `zfs-mount.service` comes from the source install and names `/opt/openzfs-2.4.4/sbin/zfs`; this host's
  names `/usr/bin/zfs`. The unit's shape and dependencies are the same file's.
- **Not a generated mount unit.**
  No arm made the ZFS mount generator write a unit, because that needs a cache file written as root on a machine
  whose ZED tracks the pool. The branch of the rule that names `<escaped path>.mount` as the provider is a source
  reading (the sibling design's F15) plus the host reads, not a run.
- **Not the key path of this host.**
  This host's key is entered at import and `zfs-load-key.service` is masked; the guest needed a key-loading unit and
  got a stand-in for it. That the provider must be ordered after one is designed from the guest's shape.
- **Not ZED.**
  No arm ran `zfs-zed.service`, and the hook's line is a text this design proposes, not a run.
- **Not a mount namespace other than the host's.**
  Every gate run was in the guest's own namespace, as root, with no second namespace holding the checkout.
- **Not a failed import.**
  No arm made the pool's import fail; the failure control failed the key load, and the checkout's mount failed with
  it. Whether a refused import leaves a different trail is unmeasured, and the provider's behaviour when import
  fails is only what the first boot above shows for an import unit skipped by its own condition.
- **Not the migration.**
  The fixture's pool, datasets and covered entry were created directly; nothing here exercises the one-time move,
  the rollback, or a covered entry whose flag was set by that move.
- **Not a busy shutdown.**
  The stop pass ran with one slot attached and nothing holding it; a busy clone, a failed stop and an interrupted
  boot were not exercised, and those are step 9's arms.
- **Not a stale learned unit.**
  Whether a mount unit left loaded from a previous mount can mislead a later start job looked at by `doctor` was not
  measured beyond the two jobs above.

## Edits this design suggests

Suggestions only; this round edits neither record.

To the plan's step 19, "Open items", which reads:

> Two parts the design leaves open block its acceptance arms:
> the checkout provider stays conditional in "One-time move and checkout boot protection", which names "the
> protected `home-user-Monochromatic.mount` if using a generated mount unit, or `zfs-mount.service` where it mounts
> the checkout", and "Checkout boot mount protection" says to prove the failure control and production ordering
> before enabling the gate.

the suggestion is to replace that sentence with the fixed provider and a pointer to this design:

> The checkout provider is fixed on a machine of this host's shape as `zfs-mount.service`, whose `zfs mount -a`
> mounts the checkout because the ZFS mount generator writes no unit here;
> where the generator's cache file exists and names the checkout dataset, the provider is the generated
> `<escaped checkout path>.mount` instead.
> The failure control and the production ordering are measured in
> `doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`, which also fixes the units'
> ordering lines and the exclusion every dataset mounted under the checkout path needs.

To the plan's step 19, "What it builds", the sentence "The gate that orders every path descendant after a verified
checkout mount" needs the condition this design measured:

> The gate that orders every path descendant after a verified checkout mount, where each such dataset carries
> `canmount=noauto` so that the automatic mount cannot mount it first.

To the design's "Boot and shutdown unit", the two bullets that name the provider and the legacy alternative need the
same substitution, and the paragraph beginning "The decided provider must require the actual home and key-loading
providers" needs three additions:

- the learned mount unit is named in `After=` only, never in `Requires=`, with the measured reason;
- the provider is named by the unit that performs the mount, not by the mount unit systemd learns afterwards;
- a machine with neither provider refuses at boot and says so, and `doctor` reports which of the two it found.

To the design's "One-time move and checkout boot protection", the paragraph that begins "The account can replace
`~/Monochromatic` while it is unmounted, regardless of the entry's owner" needs the ancestor condition:

> The account can replace an ancestor of the checkout path while the checkout is unmounted, wherever it can write
> that ancestor's parent; the immutable covered entry refuses the entry's own rename, removal and inner writes, so
> the protection holds only while every directory between the entry and the home dataset's root is one the account
> cannot write. At `/home/user/Monochromatic` that holds; at a deeper account-owned path it does not, and the entry
> belongs at a path whose ancestors are not the account's.

and the paragraph ending "The probe did not replace an ancestor of the protected entry or exercise unattended boot."
can say what now has been run:

> Item 8's two gaps are measured in the guest of
> `doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`:
> an unattended boot with the checkout's mount made to fail wrote nothing to the covered entry and created no
> mountpoint folder there, and an account-owned ancestor was renamed with a file planted at the entry's path while a
> root-owned ancestor refused it.

To the design's "Checkout boot mount protection" decision, whose last sentence reads "Failed ordinary mount and
unattended boot remain unmeasured.", the measurement now exists and the sentence becomes a pointer to this design's
failure-control section.

To the design's `ExecStart`, `ExecStopPost` and hook text, the program name follows the one-program decision
(`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`);
this design writes them against `<root-mode-entry>` and says so.

## Choices this design marks for the project's veto

- V1. Fixing `zfs-mount.service` as the provider where no generated unit exists,
  rather than leaving the conditional or proposing a unit of the route's own for the checkout.
  The alternative, a route-owned mount unit for the checkout
  (`mount-monochromatic-checkout.service` with `ExecStart=zfs mount <dataset>`),
  would be a fragment the route owns and could therefore be `Requires=`d and protected,
  and it would put a second mount path beside the provider's scan that has to be kept from racing it.
  This design does not take it, because the project's decision names the tried units, and because a route-owned mount
  of an account-writable path is the exposure the immutable entry exists to bound.
- V2. `Requires=` on the provider rather than ordering only.
  The measured value is the failure path and the pull-in; the cost is that a provider failure also fails the gate's
  unit, which is the intended behaviour but means a boot can show a failed gate whose cause is the provider.
- V3. `After=home.mount` and `After=<learned checkout mount unit>` kept although they cannot order the first start.
  The alternative is to drop both lines and state in prose what systemd does; the lines stay because the stop half
  uses them and because a reader can verify them.
- V4. Requiring `canmount=noauto` on every dataset mounted inside the checkout path.
  This is the largest change this design proposes and it reaches past step 19 into step 20's snapshot coverage and
  the setup's runbook: a `noauto` dataset is still snapshotted recursively, but the automatic mount no longer gives
  it a unit at boot, so its mount must come from the gated unit or a unit that names it.
  The alternative, letting the provider mount them and having the gate report them, needs no exclusion and gives up
  the ordering claim the design makes.
- V5. The propagation window is closed by the gate's own `private` step, and not by ordering the provider.
  Measured: the descendants are mounted under a shared checkout and the gate's first act makes the checkout private
  afterwards, so a descendant attached by the provider is a child of a shared parent for that window.
  The alternative, ordering each descendant after the gate's propagation step, is V4 by another route.
- V6. The pre-session account consumer named in the arms is a stand-in unit.
  What the project must decide is whether any real account service gets a `Requires=` on the gate, and which;
  login itself cannot be given one, and this design does not claim otherwise.
- V7. `doctor` and setup report the three provider states (generated unit, `zfs mount -a` unit, neither),
  the key-loading unit's presence, and every dataset inside the checkout path whose `canmount` is not `noauto`.
  The alternative, reporting only the provider, leaves the exclusion of V4 unread.

## Sources read

On this host, read-only, 2026-11-23:
`findmnt --output TARGET,SOURCE,FSTYPE,PROPAGATION --target` for the checkout, `/home` and every ZFS mount;
`systemctl --no-ask-password list-units --type=mount --all`;
`systemctl --no-ask-password show -p Id -p FragmentPath -p SourcePath -p After -p Before` for `home.mount`,
`home-user-Monochromatic-package-vm\x2dbuilder-cli-output.mount`, `zfs-mount.service`, `zfs-import.target`,
`zfs.target` and `systemd-user-sessions.service`;
`systemctl --no-ask-password cat zfs-mount.service zfs-import-cache.service zfs-import-scan.service zfs-import.target`;
`systemctl --no-ask-password list-unit-files` for the ZFS units' states;
`ls --directory /usr/lib/systemd/system-generators/zfs-mount-generator`;
`ls --almost-all /etc/zfs/zfs-list.cache`, `/etc/zfs/`, `/etc/zfs/zed.d/`, `/run/systemd/generator`;
`zfs get -o name,property,value mountpoint,canmount,mounted,encryption,keystatus`;
`zfs list -r -o name,mountpoint,canmount,mounted zpcachyos/ROOT/cos/home`;
`lsattr -d /home/user/Monochromatic`; `stat` of `/home`, `/home/user` and the checkout path;
`systemd-escape --suffix=mount --path` and the same without `--path`; `/etc/default/zfs` read for its ZFS rows;
`man 8 zfs-mount-generator`; `man 5 systemd.service` ("Default Dependencies").

In the guest, OpenZFS `2.4.4` as built from the release tarball pinned at sha256
`2a3c70d55a37cc71618a95a60e81ad66530201eb118d37741dc92efcf848c8b1`:
`etc/systemd/system/zfs-mount.service.in`, `etc/systemd/system/zfs-load-key.service`,
`cmd/zfs/zfs_main.c` (`zfs mount -a`'s iteration and per-dataset refusal), and the units' `.wants` wiring the
package installs.
Kernel behaviour and systemd behaviour quoted in this design come from the sibling design's facts
(`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`, F2 to F12, F15, F16, F23, F24,
F33, F36, F38, F39) rather than from a fresh source read, and each is cited where it is used.

## Measurements and logs

The round's folder,
`~/temp/agent/zfs-step-19-checkout-boot-mount-provider-and-ordering-2026-11-23/`:

- `keep/host-reads-for-the-provider-2026-11-23.log`, the host reads above with their exact output.
- `keep/guest-versions-and-units.log`, the guest's systemd, kernel, ZFS releases and `zfs-mount.service` text.
- `keep/guest-logs/`, every guest step's log: the build, the fixture, each arm's unit text, each boot's evidence
  read-back, the ancestor arm and the learned unit's two jobs.
- `keep/step19-guest-driver.ts`, the driver that ran the guest steps through `mvm --backend libvirt exec` inside
  capped `systemd-run --user` scopes.

The guest was destroyed through `mvm` at the end of the round; `mvm list` no longer reports it.

## Revisions

The writing round corrected this document in four edits before any review, and each correction is in the text above:
the requirement bullet says what a `Requires=` buys and does not buy;
the ordering bullets cite one boot's journal lines;
the ZED hook is a zedlet rather than the installed executable;
and the limits name a failed import as an arm nobody ran.

No cross-vendor review has been received, so none is folded in, and the document stays unapproved.
Every path it cites is repo-relative or under the round folder, so it carries no link table.
