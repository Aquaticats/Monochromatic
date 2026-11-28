# Move the checkout onto its own dataset and install the boot route

Status:
written by plan step 19's build-ahead round on 2026-11-25,
run in a throwaway guest on 2026-11-27 by that step's own round
(`package/git-policy/cli/e2e/zfs-step-19-guest.ts`,
`migration`),
corrected where that run showed a step that could not work as written,
not approved,
not final.
The five corrections are marked **Corrected 2026-11-27** beside the step they change,
and each names what the guest did rather than what the page assumed.
Nothing in this page has been applied to this machine,
and no round applies it:
the migration,
the units and the property changes are the owner's acts,
on the owner's authorisation recorded in
`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`
("The root recording command choice 1's answer E",
and the migration paragraph after it).

The pieces this page installs are built in
`package/git-policy/cli/src/native/checkout_boot_units/`
(the unit and drop-in texts)
and `package/git-policy/cli/src/native/checkout_underlay_entry/`
(the immutable entry's verdict),
and the recording command is the wrapper's own
`zfs-root record-checkout` mode.
The design this page follows is
`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
"One-time move and checkout boot protection" and "Boot and shutdown unit",
with the owner's answers in
`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`
and `doc/decision/cli-git-zfs-checkout-propagation-repair.md`.

## What this page changes

- Two sibling datasets under the home dataset:
  `monochromatic-git-checkout`,
  mounted at the checkout path,
  and the unmounted container
  `monochromatic-git-worktree-clone-datasets`.
- The immutable flag on the checkout path's covered entry in the home underlay.
- `snapdir=disabled` on both datasets,
  so `.zfs/snapshot` refuses every name.
- The root-owned configuration and the checkout record the route's gate reads.
- The boot and shutdown units,
  their drop-ins,
  and the propagation pair.
- The ZED hook's root-owned file and its link,
  where ZED will run it.

What it does not do:
enable ZED,
write `sanoid.conf`,
set `zfs allow`,
change any other dataset,
or reboot.
Those stay the owner's acts and plan steps 20 and 21 name them.

## Before anything

Run every step below as root,
in one session,
with no other session working in the checkout.
Every Claude Code and Pi session works inside the checkout this page moves.

1.  Confirm no process holds the checkout:

    ```bash
    sudo lsof +D /home/user/Monochromatic > /dev/null && echo 'a process holds the checkout'
    ```

    Expect no output.
    A holder is a session to end first,
    not a file to force.

2.  Record what is there,
    before anything moves:

    ```bash
    git -C /home/user/Monochromatic worktree list --porcelain
    findmnt --list --output TARGET,SOURCE,FSTYPE --submounts /home/user/Monochromatic
    zfs list -H -o name,mountpoint,used,avail -r zpcachyos/ROOT/cos/home
    ```

    The `findmnt` output is the inventory of every mount under the checkout path,
    whatever its ZFS ancestry.
    This host has one,
    `zpcachyos/nosnap/vm-builder-output` at
    `package/vm-builder/cli/output`,
    and the round that read the host listed it.
    That dataset stays under `nosnap` in the ZFS tree and only its mount point is inside the checkout:
    never copy its contents into the checkout and never write through its mount point during the move.

3.  Measure the copy's own size,
    which is the source filesystem's logical bytes with the nested mounts excluded:

    ```bash
    du --one-file-system --summarize --block-size=1 /home/user/Monochromatic
    ```

    `--one-file-system` keeps the nested dataset out,
    so the number is the bytes the copy moves.
    The planning record's measurement is not this number:
    it includes compressed allocation and nested data.

4.  Take a recoverable backup and prove it:

    ```bash
    sudo zfs snapshot "zpcachyos/ROOT/cos/home@before-checkout-migration-$(date --iso-8601=date)"
    sudo zfs list -H -o name,used,refer -t snapshot -r zpcachyos/ROOT/cos/home
    ```

    **Corrected 2026-11-27:**
    the name carried the day the page was written,
    so a second run on another day failed on an existing snapshot or named one a
    reader could not tell from the first.
    The name is the day the operator runs the step,
    and the second command lists the snapshots the parent holds.

    Keep the snapshot until the checkout,
    every linked worktree and every nested mount pass the checks below.
    A rollback to a snapshot taken before the immutable flag also removes the flag,
    which is why the
    protection is verified after a rollback and not only at installation.

5.  Confirm the copies' free space covers the measured bytes plus a margin:

    ```bash
    zfs list -H -o name,avail zpcachyos/ROOT/cos/home
    ```

## Create the two datasets

The checkout dataset mounts at the checkout path itself.
The container is unmounted,
`mountpoint=legacy`,
`canmount=off`,
`setuid=off` and `devices=off`,
and every clone inherits `mountpoint=legacy` from it.

```bash
sudo zfs create -o mountpoint=/mnt/checkout-migration \
  -o snapdir=disabled zpcachyos/ROOT/cos/home/monochromatic-git-checkout
sudo zfs create -o mountpoint=legacy -o canmount=off \
  -o setuid=off -o devices=off \
  -o snapdir=disabled zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets
```

**Corrected 2026-11-27:**
the checkout dataset was made with
`-o mountpoint=<the checkout path>`,
which mounts it over the very tree the next section has to read,
and "Copy the checkout" then created the same dataset a second time at a
temporary mount point,
which answers `cannot create ...: dataset already exists` and stops the page.
The dataset is made once,
at the staging mount point the copy and verify steps use,
and the mountpoint moves in "Switch the mounts at the same path".
A person whose `/mnt/checkout-migration` is already a mount picks another
temporary folder there and uses that name throughout.

`snapdir=disabled` is the project's answer of 2026-11-23
(`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
"What hiding comes to,
answered 2026-11-23").
It exists from OpenZFS 2.3.0 on,
and where the loaded module refuses the value the two commands fail
with the module's own error.
Read that as the answer it is:
this module cannot hide the snapshot directories,
the mount table's bound is all that stands in their place,
and the next mount is when a value takes effect.
Report the refusal with `zfs version` and stop rather than reporting a set that was not made.

A value set on a live mount is only partly in force,
and a clone inherits the container's value,
so both datasets carry it before the account's next mount of either.

## Copy the checkout

The source is the home dataset's folder at the checkout path while the checkout dataset is unmounted,
so the copy is one dataset's tree into another.
It preserves ownership,
permissions,
times,
hard links,
ACLs and extended attributes:

The destination cannot be mounted at the checkout path yet:
the copy still reads that path,
so the dataset is made once at a temporary mount point,
filled there,
verified there,
and switched to the checkout path only after the checks pass.
The temporary mount point is the operator's choice and the only value this page leaves to them:

```bash
sudo rsync --archive --hard-links --acls --xattrs --numeric-ids --one-file-system --sparse \
  /home/user/Monochromatic/ /mnt/checkout-migration/
```

**Corrected 2026-11-27:**
the `zfs create` at the head of this block made the
dataset a second time,
and the previous section now makes it once.

`--one-file-system` keeps the nested dataset's contents out of the copy,
which is what the design asks.
A copy that crossed into it would either duplicate the dataset's bytes or write into its mount.

## Verify before the switch

Every check reads the destination and changes nothing.
A check that fails stops the migration here,
with the source still in place.

```bash
sudo git -C /mnt/checkout-migration fsck --no-progress --connectivity-only
sudo git -C /mnt/checkout-migration status --porcelain=v1 --untracked-files=all --ignored=matching
sudo git -C /mnt/checkout-migration worktree list --porcelain
```

Compare the `status` output with the same command run in the source before the copy,
and every linked worktree's pointer and back-link with the `worktree list` recorded above.
The design's sentence is that the Git object database,
tracked,
untracked and eligible ignored state,
and each linked pointer and back-link are verified;
these three commands are the reading of it,
and their output is the evidence a person keeps.

## Switch the mounts at the same path

Nothing but the two mounts changes,
because the path stays the same:
no consumer rewrite and no Git repair is needed solely for the move.

```bash
sudo zfs set mountpoint=/home/user/Monochromatic zpcachyos/ROOT/cos/home/monochromatic-git-checkout
sudo zfs mount zpcachyos/ROOT/cos/home/monochromatic-git-checkout
sudo mount --make-rprivate /home/user/Monochromatic
```

**Corrected 2026-11-27:**
the `sudo umount /home/user/Monochromatic` this block
began with answers `not mounted` and stops a run under `set -e`:
the tree being replaced is a folder of the home dataset,
and only the dataset the two `zfs` lines mount is a mount of its own.
Its folders stay where they are and the new dataset hides them,
which is what the covered entry below is set on.
The `mount --make-rprivate` line is here because a mount made by hand comes up
shared:
until the boot units exist nothing else makes the checkout private.

The checkout's own mount is `shared` on this host's home dataset until the boot unit makes it private,
and a mount made by hand comes up shared again.
That is what the propagation unit and the boot unit's first act exist for;
they are installed below,
and until then a clone attached under a shared checkout is copied into every
other mount namespace.

Reopen the watches and the sessions the move stopped only after the checks pass.

## Roll back, if a check fails

The rollback is the reverse of the switch,
and the snapshot from "Before anything" is the fallback:

```bash
sudo zfs unmount /home/user/Monochromatic
sudo zfs set mountpoint=/mnt/checkout-migration zpcachyos/ROOT/cos/home/monochromatic-git-checkout
sudo zfs destroy zpcachyos/ROOT/cos/home/monochromatic-git-checkout
sudo chattr -i /home/user/Monochromatic
```

**Corrected 2026-11-27:**
three things in this block could not do what they say.
The `umount` named a path that is not a mount,
the `zfs mount` of the home dataset answers `filesystem already mounted`
because that dataset was never unmounted,
and the covered entry keeps the immutable flag the section below sets,
so a person who rolled back found the folder they had to fill again was
unchanged only because it was still flagged.
`zfs unmount` names the dataset rather than the path,
and the `chattr -i` is the flag's own removal,
which a person runs before the path may be used as a folder again.

A rollback of the home dataset itself to the pre-migration snapshot is the last resort.
It removes the immutable flag the next section sets,
so the protection is verified again after it.

## Protect the covered entry

The account can replace an unmounted checkout path on its own dataset,
so the project decided an ordinary checkout mountpoint plus an immutable covered entry
(VM item 8 measured that the flag refused the account's rename and removal while unmounted,
still allowed ordinary mounting,
and survived a reboot with a manual import and key load).

Unmount the checkout,
set the flag on the entry the home underlay holds,
and verify:

```bash
sudo umount /home/user/Monochromatic
sudo chattr +i /home/user/Monochromatic
sudo lsattr -d /home/user/Monochromatic
```

Expect the flag in the output (`----i---------`).
Verify again after every reimport,
restore or rollback,
never only here,
and verify it while the checkout is unmounted:
a mounted checkout makes the path another filesystem,
and the flag read there is the dataset's,
not the underlay's.
The wrapper's `checkout_underlay_entry` verdict does this read in code and names the difference;
`doctor` reports it once plan step 18's wiring adds the line.

## The key provider the boot needs

**Added 2026-11-27:**
the checkout dataset is encrypted,
and `zfs mount -a` does not load a key from a file `keylocation`.
Measured in the guest of 2026-11-27:
with the key unloaded,
`zfs mount -a` exits zero and mounts nothing,
and the checkout's `keystatus` stays `unavailable`,
so a boot with no key provider comes up with no checkout,
and the gate then refuses for the checkout's own state rather than for its
ordering.

Name the unit the machine loads keys with,
because the gate's key lines exist for it and the boot needs it before
`zfs-mount.service`:

```bash
systemctl list-unit-files 'zfs-load-key*'
systemctl list-units --all 'zfs-load-key*' 'zfs-load-key@*'
```

A packaged OpenZFS on Debian and Ubuntu ships `zfs-load-key.service` and its
per-dataset `zfs-load-key@.service`;
the upstream tree this route's guests are built from ships neither,
and Ubuntu 24.04's image masks `zfs-load-key.service` in
`/usr/lib/systemd/system`.
Where the machine has one,
`zfs-root record-checkout`'s own line and the gate's two key lines name it and
this page's setup writes it into the record;
where it has none,
the checkout is unencrypted or the key is loaded by the person before the
gate runs,
and the setup says which.

## Write the route's root-owned files

Two files carry the route's names for a person and for root:

- `/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc`,
  naming `cloneMounts.checkout`,
  `cloneMounts.checkoutDataset` and `cloneMounts.parent`.
  Step 8's handler answers a file that names neither dataset through its own names,
  so naming them here is the preferred reading and never the condition of the route.
- The checkout record,
  written by the wrapper's own mode rather than by hand:

    ```bash
    sudo /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets \
      monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic
    ```

  The mode proves the checkout's path,
  dataset,
  GUID,
  container and pool against the kernel,
  refuses rather than writing a name the pool does not answer for,
  and prints the GUID a later `--replace` takes when a legitimate restore moves it.
  It writes `/var/lib/monochromatic-git-policy-cli/git-worktree-clone-checkout.json`.
  It refuses before the path's dataset exists,
  so it runs after the switch above.

The root-owned program every unit line starts is installed from this release's build and linked for ZED:

```bash
sudo install -D --mode=0755 --owner=root --group=root \
  package/git-policy/cli/target/release/monochromatic-git-policy-cli \
  /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets
sudo ln --symbolic --force \
  /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets \
  /etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets
```

The program is the wrapper itself,
because the owner answered "One program only,
ever"
(`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`).
Check the installed bytes with the words that carry no mode:

```bash
/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets \
  monochromatic-git-policy-cli zfs-root --version
```

## Write the units

The texts are built by the release code,
not retyped here.
`checkout_boot_units::install_plan` returns the whole list,
in the order a setup run writes it:

- `/etc/systemd/system/reconcile-git-worktree-clone-mounts.service`
- `/etc/systemd/system/reconcile-git-worktree-clone-mounts.service.d/zz-checkout-stop-failure-mode.conf`
- `/etc/systemd/system/zfs-mount.service.d/reconcile-git-worktree-clone-mounts-after-checkout.conf`
- `/etc/systemd/system/checkout-propagation.service`
- `/etc/systemd/system/<the checkout's mount unit>.d/10-upholds-propagation.conf`
- one `/etc/systemd/system/git-worktree-nested-<short word>-mount.service` per dataset whose mountpoint
  lies inside the checkout

The plan refuses a value a systemd line cannot carry,
and it refuses to write the propagation pair
beside a gate whose text lacks `ExecReload=`:
without that line the pair's request is a restart,
which runs the gate's stop pass and unmounts every
attached clone.

Each nested dataset also carries `canmount=noauto`,
which is the owner's act on an existing dataset
and what keeps `zfs mount -a` from mounting it before the gate proved the checkout:

```bash
sudo zfs set canmount=noauto zpcachyos/nosnap/vm-builder-output
```

Where no program writes the plan yet,
the texts are the ones the release code builds and the host
controls assert;
this build reaches the plan from no command and prints it nowhere,
which is the gap
the wiring step closes.

Load the units and enable the gate:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now reconcile-git-worktree-clone-mounts.service
```

Do not start the propagation service by hand:
the mount unit's `Upholds=` starts it,
and an enabled copy would run at a boot whose checkout may not be
mounted at all.
The provider drop-in asks the gate for a reload after `zfs mount -a`,
including a provider started by
something other than the boot,
which is the trigger a key-load event before the checkout cannot give.

Read what the manager loaded,
rather than trusting the files:

```bash
systemctl show --property=TimeoutStartUSec,TimeoutStopUSec,DropInPaths,CanReload \
  reconcile-git-worktree-clone-mounts.service
systemd-analyze verify /etc/systemd/system/reconcile-git-worktree-clone-mounts.service
```

The verify command reports one line for the propagation pair on a machine whose mount unit systemd
learned from the mount table (`Unit <escaped checkout path>.mount not found`);
that is an artifact of verify having no live manager,
and the pair resolves once the mount unit is
loaded.
Any other line is a finding.

## After the boot

A reboot is the owner's act,
never this page's.
After one,
read the order and the two records:

```bash
journalctl --boot --unit=reconcile-git-worktree-clone-mounts.service --unit=checkout-propagation.service
cat /run/git-worktree-clone-mount-readiness/result
cat /run/git-worktree-clone-mount-propagation/result
```

The gate's start bound is `5min` and its stop bound `1min`,
the owner's numbers,
which plan step 22
replaces with measured ceilings.
A start that failed its stop pass leaves the stop line prefixed with `-`,
so a shutdown reports the unmount refusal and still ends.

## What is still owed before this page is final

Run in a throwaway guest on 2026-11-27 by plan step 19's own round
(`package/git-policy/cli/e2e/zfs-step-19-guest.ts`):
the migration and its rollback on a disposable layout,
each step's command and
output kept in that run's logs;
the install plan written at the paths systemd reads,
each unit file parsed by the guest's own `systemd-analyze verify`,
and every drop-in read back from the guest's manager;
the immutable entry's verdict against a real folder carrying the kernel's flag
on a real dataset,
with the account's own refused rename and removal beside it;
the `snapdir=disabled` set on both datasets and the clone that inherits it;
and a reboot of the guest with the plan installed,
an idle managed clone attached again by the gate,
and three boots whose import,
key and checkout mount did not happen,
where nothing attached and no mountpoint folder appeared in the home underlay.

What the boot arms showed,
and what they still owe:

- The plan's files written where systemd reads them,
  the gate enabled and the
  guest rebooted,
  the manager starts the gate,
  its `ExecStartPre` runs the start probe,
  and a probe that cannot prove the route refuses the whole reconciliation:
  the gate ends `failed` with `Result=exit-code`,
  no readiness record is published for that boot,
  the nested dataset's unit stays `inactive`,
  no nested dataset is mounted,
  and no mountpoint folder appears under the checkout path in the home
  underlay.
  That is the plan's "fail a prerequisite,
  assert no nested dataset attaches or creates mountpoint folders in the home
  underlay" arm,
  run against a real manager (the step 19 guest of 2026-11-27,
  `STEP19-BOOT read expect=failed gate=failed ... nested-mounted=false
  underlay-folder=false`).
- The same boot also ran `ExecStopPost`'s `shutdown-unmount`,
  which refuses in words that name plan step 19,
  so the journal of one boot carries both an undone start and an undone stop.
- The arm that expects an idle clone attached again after a reboot did not
  run:
  the guest of the Ubuntu recipe boots with its own packaged ZFS module
  (2.2.2),
  because `modprobe` searches `kernel/` before the `extra/` folder the
  recipe installs the release it built into,
  so the route's compatibility gate refuses that boot and nothing is attached.
  What it waits for is a guest that boots the module the route's releases are
  built against:
  plan step 23's template carries one,
  and this page's own boot arm needs it.

Still owed:

- `zfs-root shutdown-unmount` still refuses in this build,
  so no shutdown pass is measured,
  and the plan's "reboot and shutdown with idle and busy legacy clones" arm is
  half run:
  the boot half passed,
  the shutdown half waits for that mode's body (plan step 19's own wiring,
  `doc/decision/cli-git-zfs-boot-units-start-at-boot-and-the-reload-guard.md`
  being the only record it has).
- The propagation step (`checkout_propagation`) and its call at the head of
  `boot-reconcile` and the default sweep are designed and accepted,
  and are not built:
  the pair's drop-in and `Upholds=` are read back from the manager,
  but the trigger is never fired,
  so whether a hand-typed replacement mount is missed is unmeasured.
- `doctor` does not yet report the migration,
  the boot findings or the immutable entry,
  which plan step 19's `### Doctor` passage and the owner's V7 answer require.
- The ZED half of this page:
  the zedlet's install,
  its AppArmor rules in `/etc/apparmor.d/local/zed`,
  and the ordering arms of a live ZED (replay,
  dropped events,
  concurrent zedlets,
  stale receipts and a pending unmount across a restart) belong to the
  lost-event rounds and run nowhere yet.
- This page has not been run on the owner's machine:
  the migration,
  the property changes,
  the flag and the units are the owner's
  acts,
  on the owner's authorisation,
  at a moment the owner chooses.
