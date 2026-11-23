# Measurements owed before accepting the ZFS worktree design

**Non-final draft by an agent, unapproved.** It records guest observations and design implications, not design
acceptance. A section heading's stamp is the commit time of the commit that landed that section. This file's own
revision stamp is what `git log --format=%cI -1` prints for the file, which before this revision's commit reported
2026-11-19T01:34:47-05:00 at `6a678f1b5b`. The account ownership decision in the
[planning record](cli-git-zfs-worktree.md#owners-answers-of-2026-11-18) makes the refusal repeat a pre-acceptance
condition, not something to postpone until implementation.

## Fixture and evidence

The 2026-11-18 gate review of proposal `71fa25ece8` and the dated corrections in the
[clones research](cli-git-zfs-clones-mounted-under-the-checkout.md) supplied the open measurements. The guest probe
reused the descriptor calls from that research's `guest-extra-commands.txt`. Full commands, output and assertions
are in `~/temp/agent/zfs-measurements-owed-before-acceptance-2026-11-18/keep/`, in `guest-probe.py` and the
per-item logs. Each item uses a separate encrypted file-backed pool in `zfs-acceptance-mvm`, an Ubuntu 24.04 `mvm`
libvirt guest.
The checkout and worktree clone parent are sibling datasets under the guest home dataset. The Git fixtures use
plain `git` in the guest, with a disposable `HOME` and `XDG_RUNTIME_DIR`. The guest Git is `2.43.0`; the host wrapper
reports Git `2.56.1`. The probe is not an execution of the proposed Rust binding or product handler.

Host, read-only: `uname --kernel-release` printed `7.3.5-1-cachyos` and `/sys/module/zfs/version` read `2.4.4-1`.
`findmnt --output TARGET,SOURCE,PROPAGATION --target` returned these sources for `/`, `/home` and `~/.cache`:

```text
/                 zpcachyos/ROOT/cos/root     shared
/home             zpcachyos/ROOT/cos/home     shared
/home/user/.cache zpcachyos/nosnap/user-cache shared
```

`findmnt --target /home/user/Monochromatic` resolves to `/home`, not a dedicated checkout mount yet.
`zfs get` on the host home dataset returned `aes-256-gcm`, encryption root `zpcachyos`, key `available`,
`recordsize=32K` and `compression=zstd-3`. No host pool, filesystem mount or propagation was changed.

## 1. Register, snapshot, attach and fill

Guest: `6.8.0-147-generic`, OpenZFS userspace `2.4.4-1`, module `2.4.4-1` (`item-1-exact.log`). The kernel differs
from the host; the ZFS module release matches. A separate setup run used the guest package module `2.2.2-0ubuntu9.5`
with 2.4.4-1 userspace and also passed (`item-1-packaged-module.log`); the results here are from the matched run.

The account made an empty slot under its own `.claude/worktrees`, ran
`git worktree add --no-checkout --detach <slot> HEAD`, then root ran `zfs snapshot` and `zfs clone` in that order.
The root fixture opened the checkout and walked the target components with `openat2` and
`RESOLVE_NO_SYMLINKS | RESOLVE_BENEATH | RESOLVE_NO_XDEV | RESOLVE_NO_MAGICLINKS`, comparing the recorded device,
inode, owner, mode and ACL of the parent and slot. It used `fsopen`, `fsconfig` with the clone as source, `fsmount`
with `nosuid,nodev`, and `move_mount` onto the checked descriptor. A fresh walk from the checkout crossed the
final mount only and matched mount ID, device, inode, filesystem and source to the mounted descriptor.

```text
Preparing worktree (detached HEAD 1ae6bf0)
recorded parent/slot: (47, 3, 1000, 493, ()) (47, 22, 1000, 493, ())
361    360 zfsowed1/home/worktree-clone-datasets/registered private     rw,nosuid,nodev,relatime
post-attach identity: ['361', 48, 34, 'zfs', 'zfsowed1/home/worktree-clone-datasets/registered']
```

The clone's root had a `.git` directory; the nested copy of the slot had the exact underlay pointer, proving the
snapshot included the registration. The account moved the directory aside, installed the pointer and ran
`git read-tree HEAD`. `rev-parse --show-toplevel`, `rev-parse --git-common-dir` and the byte count of
`status --porcelain --untracked-files=no` returned:

```text
/var/lib/zfs-acceptance-fixtures/item-1/home/account/Monochromatic/.claude/worktrees/registered
/var/lib/zfs-acceptance-fixtures/item-1/home/account/Monochromatic/.git
tracked status stdout bytes: 0
underlay pointer unchanged after unmount
```

The specified order works in this fixture. Keep both pointer copies, recheck the canonical mount before fill, and
never run Git against the copied administration. Moving that directory aside did not measure the deletion cost or
prove recovery after interruption.

## 2. Refusals with account-owned parent and slot

Guest: `6.8.0-147-generic`, OpenZFS userspace/module `2.4.4-1` (`item-2-exact.log`). The kernel differs from the host;
the module release matches. Both recorded owners were UID 1000, both modes 0755. The account created the links,
replaced the slot and changed its mode and ACL. Root supplied the unexpected owner and the bind mount controls.
The attach control reused the item 1 descriptor checks.

```text
account link at .claude: openat2: ELOOP
account link at .claude/worktrees: openat2: ELOOP
account link at .claude/worktrees/checked: openat2: ELOOP
account-created link in owned parent (creation succeeded): openat2: ELOOP
parent escape ..: openat2: EXDEV
procfs magic link: openat2: ELOOP
bind mount with unchanged dev/inode/owner/mode/ACL: openat2: EXDEV
new inode with same owner and mode: recorded slot differs: inode
wrong slot owner: recorded slot differs: owner
widened mode: recorded slot differs: mode
ACL u:nobody:r-x with unchanged permission bits: recorded slot differs: ACL
ACL d:u:nobody:r-x with unchanged permission bits: recorded slot differs: ACL
already attached ZFS clone: openat2: EXDEV
post-attach identity: ['361', 48, 34, 'zfs', 'zfsowed2/home/worktree-clone-datasets/checked']
```

The account-created link is a changed result: creation now **succeeds** and the descriptor walk refuses it. The
magic-link arm used a real procfs descriptor link; it did not make procfs reachable below the checkout. The ACL
arms changed the access and default ACL separately while preserving mode bits. Mode alone missed both changes.

In a bounded rename loop, each trial synchronized one account rename of `.claude` between opening and attaching.
`move_mount` succeeded onto the held inode, not the canonical path. Each fresh walk from the checkout refused,
and the probe confirmed the clone at the renamed path, moved that mount to staging and unmounted it. It asserted
no mount remained at the canonical slot or staging; the final undisturbed descriptor attach worked.

```text
rename trial 1: canonical post-check: openat2: ENOENT
rename trial 6: canonical post-check: openat2: ENOENT
6 synchronized rename interleavings: misplaced attaches detected and removed; final control passed
```

Refuse identity changes without adopting them. A held parent descriptor can move with its ancestor, so only a fresh
canonical walk catches this case. The post-check and rollback are necessary; this is not a proof of atomic placement
or coverage of every free-running interleaving. The rollback was in a private namespace; item 3 measures the
propagation precondition.

## 3. Shared propagation and a confined reader

Guest: `6.8.0-147-generic`, OpenZFS userspace/module `2.4.4-1` (`item-3-fresh-signals.log`). The kernel differs from
the host; the module release matches. The mount topology and units ran in the guest boot namespace so the guest
PID 1 could start confined readers from the mount tree being tested. The guest's `/home` was a bind in an independent
shared group, not a separate ZFS home as on the host; the ZFS fixture home under it was also shared.

```text
29      1 /dev/vda1 shared      rw,relatime
60     29 /dev/vda1[/home] shared      rw,relatime
120     60 zfsowed3/home shared      rw,relatime
123    120 zfsowed3/home/checkout shared      rw,relatime
```

In both arms the probe created a clone with the item 1 descriptor route. Before attachment a guest systemd unit
used `PrivateMounts=yes` and `ReadOnlyPaths=` on an unrelated directory, outside the checkout. A readiness file
confirmed its own `findmnt` and marker looked absent before attach. After attach, the mounting process wrote a
marker and signaled the reader. Once the reader finished, a fresh process tried the same descriptor move to the
same private staging bind in both arms. The source parent and staging propagation were re-read at each move.

With the **checkout shared**, the unit saw the later mount and marker, but the move refused and the clone stayed
at its slot. Excerpts from the reader, move and subsequent `findmnt`:

```text
reader before: {"mount": "", "marker": null}
268    401 zfsowed3/home/worktree-clone-datasets/shared shared,slave rw,nosuid,nodev,relatime
move_mount: EINVAL
267    123 zfsowed3/home/worktree-clone-datasets/shared shared      rw,nosuid,nodev,relatime
```

With **only the checkout private**, the mounting namespace saw the clone but the same kind of reader did not.
The fresh process moved mount ID 267 from the checkout to staging mount ID 210 and unmounted it with exit 0;
the staging bind remained, not the clone.

```text
reader before: {"mount": "", "marker": null}
reader after: {"mount": "", "marker": null}
123    120 zfsowed3/home/checkout private     rw,relatime
120     60 zfsowed3/home shared      rw,relatime
267    210 zfsowed3/home/worktree-clone-datasets/private              private     rw,nosuid,nodev,relatime
clone mount ID 267 absent from mountinfo after unmount
```

The shared-parent refusal predicted by gate review GM1 held. The proposal's stronger claim that the checkout's
ancestors must also be private did not: only the checkout changed. That narrower option still loses delivery of
later mounts to already-confined consumers. The design needs to state which property it requires, and must not
assume both with the same move-to-staging route. Check visibility from the actual caller namespace. The probe did
not choose a propagation policy or test a different unmount mechanism.

Two fixture attempts are not the comparison: the first bind accidentally left `/home` a peer of `/`, duplicating
mounts, and the staging assertion mistook the staging bind for a surviving clone (`item-3-exact.log`). The next
attempt reused a readiness marker (`item-3-corrected.log`). The recorded run separated the peer groups, checked the
clone's mount ID after unmount, and deleted each arm's readiness, completion and result files before launching it.

## 4. Unmount from a fresh process

Guest: `6.8.0-147-generic`, OpenZFS userspace/module `2.4.4-1` (`item-4-exact.log`). The kernel differs from the
host; the module release matches. The attach process recorded the mounted root and closed its descriptors.
A new process opened the slot relative to the checkout, keeping the no-cross-device flag on the parent walk
but removing it for the final component. The attach flags on that final component had the expected `EXDEV`.
The new process matched mount ID from its own descriptor record, device and inode from `fstat`, and filesystem
and dataset source from its own mountinfo. With those values equal it called `move_mount` with the two
descriptors, closed them, then ran `umount` at the private staging path.

```text
attacher process 57766 has closed its mount descriptors
attach-side flags on mounted clone: openat2: EXDEV
276     62 zfsowed4/home/worktree-clone-datasets/mount-to-remove private     rw,nosuid,nodev,relatime
clone mount ID 276 absent from mountinfo after unmount
```

In the synchronized substitution arm the account renamed the ancestor after the fresh process opened the clone,
then recreated a plain slot. A second canonical walk saw the checkout dataset instead of the clone. The fixture
refused to move anything and verified the clone remained at the renamed path with its held identity unchanged.
A separate different-clone control put another ZFS mount at the canonical slot. The old record missed only the
dataset source: **mount ID 276, device 48 and inode 34 were reused**. The fixture refused the stale record
and left that clone mounted. With its own record, the replacement then unmounted normally.

```text
REFUSED before move: mounted identity differs
276    274 zfsowed4/home/worktree-clone-datasets/different-clone private     rw,nosuid,nodev,relatime
```

A separate busy-holder arm moved the verified clone to staging while a process kept its working directory there.
Staging `umount` refused, the clone remained visible at staging, and a retry after the holder exited succeeded.

```text
umount: /var/lib/zfs-acceptance-fixtures/item-4/staging: target is busy.
276     62 zfsowed4/home/worktree-clone-datasets/mount-to-remove private     rw,nosuid,nodev,relatime
```

Open and verify the actual mounted object in each handler process; do not treat an old mount ID, even together
with device and inode, as durable identity. Check the source and the live dataset GUID from the binding before
moving or destroying; the probe did not substitute a new dataset under the same dataset name. Recheck the canonical
relationship without using it as the action operand. Keep explicit staged state after a busy refusal; absence
at the slot is not absence from the namespace. The probe did not exercise reattach or staged-state crash recovery.

## 5. Encrypted fixture and an unloaded key

Guest: `6.8.0-147-generic`, OpenZFS userspace/module `2.4.4-1` (`item-5-exact.log`). The kernel differs from the
host; the module release matches. Every numbered item used a separate 1 GiB file-backed pool with an encryption root
at the pool root, `encryption=aes-256-gcm`, `keyformat=raw` with a guest-local key file, `recordsize=32K` and
`compression=zstd-3`. Each log records the inherited checkout properties. This matches the host values requested,
not the host's storage topology, pool feature flags or physical I/O.

For the unloaded-key arm the probe took the origin snapshot, unmounted checkout and fixture home, and ran
`zfs unload-key zfsowed5`. With stdin closed, root's `zfs clone` still exited 0. The clone's `zfs get` output was:

```text
zfsowed5/home/worktree-clone-datasets/key-unavailable	encryption	aes-256-gcm
zfsowed5/home/worktree-clone-datasets/key-unavailable	encryptionroot	zfsowed5
zfsowed5/home/worktree-clone-datasets/key-unavailable	keystatus	unavailable
zfsowed5/home/worktree-clone-datasets/key-unavailable	keylocation	none
zfsowed5/home/worktree-clone-datasets/key-unavailable	recordsize	32K
zfsowed5/home/worktree-clone-datasets/key-unavailable	compression	zstd-3
detached mount while key unavailable: fsconfig(create): EACCES
```

The detached mount failed before `fsmount` or any target attach. The test set a user property on the clone;
a guest ZED daemon delivered the history event to a root-owned test zedlet, which queried `keystatus` and
recorded the following. The product handler does not yet exist.

```text
ZED test zedlet: uid=0 keystatus=unavailable; action=defer without mount
```

`zfs load-key zfsowed5` followed by mounting the fixture home and checkout changed clone `keystatus` to
`available`; a descriptor attach then passed. A successful clone is not mount or fill readiness. Identify the
encryption root and report an unavailable key separately from mount identity or permission failures, before
attaching. Reconcile after key load and verified checkout mount rather than relying on another add request.
The root CLI probe did not exercise delegated clone permission with the key unloaded or automatic post-load retry.

## 6. Deleted underlay pointer, prune and lock

Guest: `6.8.0-147-generic`, OpenZFS userspace/module `2.4.4-1`, Git `2.43.0` (`item-6-exact.log`). The kernel
and Git releases differ from the host; the ZFS module release matches. No `gc.worktreePruneExpire` override
was set. In each arm the account registered and filled a clone-backed worktree, root unmounted the clone
and checked the slot was not a mountpoint, then the account unlinked its underlay `.git`. The locked arm ran
`git worktree lock --reason 'ZFS test underlay' <slot>` before unlink. The probe ran real
`git worktree prune --verbose`, not a dry run, then `--expire now`, and checked the administrative directory.

```text
Removing worktrees/unlocked: gitdir file points to non-existent location
registration after default prune: False
registration after --expire now: False
locked ZFS test underlay
registration after default prune: True
registration after --expire now: True
```

With the locked arm's pointer restored and the clone still unmounted, Git refused both normal and
single-`--force` move and remove. With `--force --force` each operation exited 0, and the dataset still existed
after Git removed the registration.

```text
fatal: cannot move a locked working tree, lock reason: ZFS test underlay
fatal: cannot remove a locked working tree, lock reason: ZFS test underlay
zfsowed6/home/worktree-clone-datasets/locked
```

The lock guards registration against normal pruning even when the account deletes the pointer. It is not an
authority boundary and is not a substitute for the clone binding or the wrapper's separate move refusal.
If the design uses a managed lock, distinguish it from a user lock and release it only as part of a verified
removal, not by forwarding force. Preserve both pointer copies; the lock does not repair either.

## 7. Setuid execution on the clone

Guest: `6.8.0-147-generic`, OpenZFS userspace/module `2.4.4-1` (`item-7-exact.log`). The kernel differs from the
host; the module release matches. The account installed a compiled executable that only prints its real and
effective UIDs. Root supplied ownership and mode `4755` for the positive control: a setuid program owned by
the executing account would not elevate even with no mount restriction. The probe ran the same executable as the
account on the same clone mounted first with no added VFS restrictions, then with `nosuid,nodev` via `fsmount`.

```text
276    274 zfsowed7/home/worktree-clone-datasets/privilege-probe private     rw,relatime
real=1000 effective=0
276    274 zfsowed7/home/worktree-clone-datasets/privilege-probe private     rw,nosuid,nodev,relatime
real=1000 effective=1000
```

The second run did not gain privilege despite unchanged root ownership and setuid mode. Enforce and verify the
mount flag independently of dataset properties; the control deliberately let setuid work at the dataset level.
This measures the setuid half of gate 5 for the descriptor primitive, not the installed handler.

## 8. Immutable underlay, boot and parent rollback

Guest before and after reboot: `6.8.0-147-generic`, OpenZFS userspace/module `2.4.4-1` (`item-8-pre-exact.log`,
`guest-reboot.log`, `item-8-post-exact.log`). The kernel differs from the host; the module release matches.

With the checkout **unmounted**, the account could rename its underlay folder and create a replacement at
the original path. Root snapshotted the parent dataset and ran `chattr +i` on the underlay directory.
With the same ownership and now an immutable directory, the account's `mv --no-target-directory` and
`rmdir` both exited 1, ending with `Operation not permitted`. `zfs mount -a` still mounted the checkout there;
after unmount, `lsattr -d` still reported `----i-----------------`.

The probe exported the file-backed pool, rebooted the VM, reloaded the exact module, imported with `-N`,
loaded the key and mounted only the parent. The boot ID changed from
`00ee24e6-a78a-4f9c-aeef-cd896fa0763b` to `df936c8d-fd5b-4b35-b763-ecbd06b8349e`. The flag remained and
the account's rename and remove still refused. Next, root cleared the flag, rolled back to the snapshot
that contained it, then rolled back again to the pre-flag snapshot:

```text
flag persisted through mount -a
rollback to flag-bearing snapshot restored immutable
rollback to pre-flag snapshot removed immutable
```

The account again renamed and replaced the unprotected folder after the second rollback. An immutable
entry is a viable protection for this directory under that ancestor layout, but not an invariant after a
parent rollback. Verify the underlay protection and intended mount identity before an automated checkout
attach, and after reimport or rollback. This was persistence across a normal reboot with manual import and
key load, not a test of the production boot service or of replacing an ancestor of the protected entry.

## 9. Automatic mounts below a legacy checkout

Guest: `6.8.0-147-generic`, OpenZFS userspace/module `2.4.4-1` (`item-9-exact.log`). The kernel differs from the
host; the module release matches. Host `zfs list` reported `zpcachyos/nosnap/vm-builder-output` mounted at
`/home/user/Monochromatic/package/vm-builder/cli/output`. The guest output dataset was likewise separate
in dataset ancestry but nested by path. The checkout had `mountpoint=legacy`. The probe created the
output dataset without mounting it (`zfs create -u`), then ran `zfs mount -a`.

The checkout was still not mounted, but the output dataset was attached to the home underlay. The probe
wrote a marker into that dataset, then attached the checkout by descriptor. The two mounts had the same
parent ID, 361. The checkout hid the output marker, even though the output mount still appeared in
`findmnt --mountpoint` and mountinfo:

```text
362    361 zfsowed9/nosnap/vm-builder-output private     rw,relatime
363    361 zfsowed9/home/checkout private     rw,relatime
362    361 zfsowed9/nosnap/vm-builder-output
```

`stat` on the marker exited 1 with `No such file or directory`. After unmounting the checkout, the marker
and output mount ID 362 were visible again. Hidden did not mean unmounted.

GM8's prediction held for an ordinary mount under a legacy checkout. A legacy checkout boot scheme must
exclude all path descendants from automatic early mounting and order their attachment after the verified
checkout mount, or refuse if that ordering cannot be proved. Include datasets outside the checkout's ZFS
ancestry and verify path reachability, not only mount-table membership. This did not exercise a failed
ordinary-checkout mount or production boot units.

## 10. Explicit ZFS mount of a legacy clone

Guest: `6.8.0-147-generic`, OpenZFS userspace/module `2.4.4-1` (`item-10-exact.log`). The kernel differs from the
host; the module release matches. `zfs get` showed inherited `mountpoint=legacy`, default `canmount=on` and
`mounted=no`. An explicit `zfs mount <clone>` exited 1 and left the slot unmounted:

```text
cannot mount 'zfsowed10/home/worktree-clone-datasets/legacy': legacy mountpoint
use mount(8) to mount this filesystem
```

The descriptor attach of that same dataset succeeded, with `rw,nosuid,nodev,relatime`. This confirms the
explicit-mount branch of `zfs-2.4.4`'s `cmd/zfs/zfs_main.c:7240` to `:7248`. The proposal's citation at the time
(its lines 118 to 119) pointed at the unmount branch instead; the current proposal cites this branch. Treat the
refusal as expected: an inherited legacy property does not name a target slot, and the design must not fall back
to a path-based mount.

## 11. Promote and the checkout's snapshot history

Guest: `6.8.0-147-generic`, OpenZFS userspace/module `2.4.4-1` (`item-11-exact.log`). The kernel differs from the
host; the module release matches. The probe made checkout snapshots `@earliest`, `@origin` and
`@checkout-later`, cloned `@origin`, and made `@worktree-own` on the clone. It recorded the origin and
encryption root of both datasets and snapshot names and GUIDs before and after `zfs promote <worktree-clone>`.

The checkout started independent and the worktree depended on checkout's `@origin`. Promotion reversed
that dependency: checkout now depended on the worktree's `@origin`. Checkout's origin snapshot and its
older snapshot moved to the worktree without changing GUIDs; the later checkout and worktree-owned snapshots
stayed where they were. This matches `zfs-promote(8)`. After promotion:

```text
zfsowed11/home/checkout	origin	zfsowed11/home/worktree-clone-datasets/promotion@origin
zfsowed11/home/worktree-clone-datasets/promotion	origin	-
zfsowed11/home/checkout@checkout-later	16110764806619961322
zfsowed11/home/worktree-clone-datasets/promotion@earliest	12367540667152475382
zfsowed11/home/worktree-clone-datasets/promotion@origin	14322725115875610900
zfsowed11/home/worktree-clone-datasets/promotion@worktree-own	15127986852297743065
```

`zfs destroy -r <promoted-clone>` exited 1 with the snapshot list unchanged. A **dry run** with `-n -v -R`
included the checkout itself and its later snapshot:

```text
cannot destroy 'zfsowed11/home/worktree-clone-datasets/promotion': filesystem has dependent clones
would destroy zfsowed11/home/checkout@checkout-later
would destroy zfsowed11/home/checkout
```

Promoting the checkout back restored the full pre-promotion snapshot map and origins. The unmounted
worktree and its own snapshot then destroyed together with `zfs destroy -r` exit 0, and the checkout
retained all its snapshots. This control is not a requirement to issue a recursive destroy in the wrapper.

A second promotion showed why the ordinary recursive refusal is not protection for the proposal's
individual snapshot deletes. `zfs destroy <promoted-clone>@earliest` exited 0. Destroying its `@origin`
then refused with `snapshot has dependent clones`. Promoting checkout back did not restore the older snapshot:

```text
cannot open 'zfsowed11/home/checkout@earliest': dataset does not exist
explicit removal lost checkout @earliest before the protected origin stopped it
```

Include the binding primitive as decided, but do not let ordinary add or remove promote from the protected
checkout into a removable worktree. For any explicit lineage change, prove who owns the affected snapshot
GUIDs, what will depend on the result, and where protected history will remain. Removal must reject a
promoted or otherwise mismatched lineage **before deleting the first snapshot**. A successful reverse
promotion in this fixture is a recovery control, not a guarantee for conflicting snapshot names, changed
history, other encryption roots or other clone relationships.

## Differences from the proposal and gate review

- The account-owned-parent refusal repeat belongs before design acceptance, as GH1 says. The permission denial
  GH1 predicted for account-created links is not part of this layout: link creation worked; the descriptor walk
  refused it.
- GM1's shared-checkout prediction held and its narrower private-checkout control worked. The proposal and the
  clones research then required private ancestors too; this item disproved that requirement for the move, and the
  current proposal requires only a verified private immediate parent. The cost is the loss of propagation into an
  already-confined reader.
- GM4 and GM5 reviewed a version before the project selected inclusion of `promote` and immediate destruction
  of the removed worktree's own snapshots. Those decisions stand; the measurements show why inherited checkout
  history must not be mistaken for the worktree's own history.
- The rename loop used synchronized, bounded interleavings, not a tight stress loop. Item 3 used the disposable
  VM's boot namespace so its real systemd units inherited the tested mount topology. The host was not used for
  either mount or rename probes.

## What this did not show

- Host-kernel execution or Git `2.56.1` runtime behavior: the agreed probe boundary was the disposable guest.
  A file-backed encrypted pool measures semantics, not performance on the host's storage.
- Product wrapper, Rust binding, ZED reconciliation, confinement variants or boot orchestration: the fixture used
  direct kernel calls, guest Git, a limited systemd reader and a test zedlet. Item 8 explicitly imported and loaded
  the key after reboot; it was not an unattended startup or shutdown test of the design.
- Free-running rename and replacement interleavings, recreation under the same dataset name, failed rollback or
  reattach, and process interruption at lifecycle steps. The controls synchronized selected windows and verified
  state, rather than claiming exhaustive scheduling coverage.
- Promotion reversal with name collisions, other encrypted lineage shapes, or dependents beyond the fixture.
  The dependent-destroy variant ran as a dry run only, because the deletion set, not destroying the checkout,
  was the relevant evidence.

## Delta gate measurements, 2026-11-18T23:04:53-05:00

This non-final, unapproved repeat addresses the delta gate review of proposal `652b51d74d`. Its first item commit
is `cc87958f37` at 2026-11-18T23:04:53-05:00 and its last is `666d6e2617` at 2026-11-18T23:16:35-05:00. The guest was
`zfs-delta-measurements`, an Ubuntu 24.04 `mvm` libvirt VM. Each probe reused the encrypted file-backed fixture:
`aes-256-gcm`, raw key, `recordsize=32K`, `compression=zstd-3`, checkout and clone parent under the same home dataset.
Scripts and logs are in `~/temp/agent/zfs-measurements-owed-before-acceptance-2026-11-18/delta-review-items/keep/`.
The project decisions to destroy the worktree and its own snapshots at once and to include `promote` still stand.

### 12. Same-name replacement with delegated grants

Guest: `6.8.0-147-generic`, OpenZFS userspace and module `2.4.4-1` (`item-12-exact.log`). Root enabled pool
`delegation`, set `zfs allow -l -u ubuntu create,mount` on the clone parent, `-d -u ubuntu clone` on the checkout,
and `-d -u ubuntu mount,destroy,userprop` on the clone parent. No other ZFS grants were set. The proposal also
lists local checkout `snapshot`; the DH1 repeat deliberately omitted it as requested, and root made the origin.
`id` under `runuser` returned `uid=1000(ubuntu) gid=1000(ubuntu) groups=1000(ubuntu)`.

The account cloned `zfsowed12/home/checkout@replacement` to `zfsowed12/home/worktree-clone-datasets/replacement`.
It ran `zfs destroy` (exit 0), then `zfs create` at the same name (exit 0). A `zfs get` before and after saw:

```text
# item-12-exact.log, selected property rows
zfsowed12/home/worktree-clone-datasets/replacement	mounted	no
zfsowed12/home/worktree-clone-datasets/replacement	guid	15139342543771036697
zfsowed12/home/worktree-clone-datasets/replacement	createtxg	18
zfsowed12/home/worktree-clone-datasets/replacement	mounted	no
zfsowed12/home/worktree-clone-datasets/replacement	guid	7645651087564023451
zfsowed12/home/worktree-clone-datasets/replacement	createtxg	21
```

`identity-destroy.lua` reads `guid`, `origin` and `createtxg` using `zfs.get_prop`, and calls `zfs.sync.destroy`
only if all match. `zfs program -- <pool> <script> <dataset> <guid> <origin> <createtxg>` as `ubuntu` with the
original clone's matching tuple exited 1, with the clone unchanged:

```text
# item-12-exact.log
Channel program execution failed:
Permission denied. Channel programs must be run as root.
```

As root, the stale original tuple against the plain replacement exited 1 with an `identity mismatch` diagnostic
naming the replacement's GUID, `origin=-` and `createtxg` (`item-12-exact.log:93`):

```text
# item-12-exact.log:93, the plain replacement's refusal
[string "channel program"]:36: identity mismatch: guid=7645651087564023451 origin=- createtxg=21
```

The account then replaced that filesystem
with a new clone of the **same** origin. The root program again exited 1 on the original tuple, and the new clone
was unchanged. With its actual tuple it exited 0; `zfs list` then exited 1 with `dataset does not exist`:

```text
# item-12-exact.log, selected output
[string "channel program"]:36: identity mismatch: guid=5278023353963548225 origin=zfsowed12/home/checkout@replacement createtxg=24
Channel program fully executed and produced output:
    return:
        createtxg: '24'
        destroyed: 'zfsowed12/home/worktree-clone-datasets/replacement'
        guid: '5278023353963548225'
        origin: 'zfsowed12/home/checkout@replacement'
```

The probe passed GUIDs as exact signed 64-bit decimal strings to match Lua's representation of unsigned ZFS
GUID bits. The `--` prevents a negative GUID from being parsed as a CLI option. Two setup attempts caught
that argument issue and that `delegation` is a `zpool set` property, not a `zfs set` property.

This confirms DH1's same-name replacement with the three grants and the root-only conditional destroy on this
release. A GUID read identifies the object at the time of the read; even matching the old origin name does not
identify a replacement. The test does not demonstrate the timing of a concurrent replacement, verify a product
removal path, or authorize a new privileged destroy service.

### 13. Creation transaction groups and a bookmark after promote

Guest: `6.8.0-147-generic`, OpenZFS userspace and module `2.4.4-1` (`item-13-exact.log`). The probe repeated
item 11's `zfs snapshot`, `zfs clone` and `zfs promote` sequence. Between `@earliest` and `@origin`, it ran
`zfs bookmark zfsowed13/home/checkout@earliest 'zfsowed13/home/checkout#from-earliest'`.
It read the clone with `zfs get -H -p -o value createtxg,guid` and listed snapshots and bookmarks under both
heads with `zfs list -H -p -o name,createtxg,guid -r`, selecting each object type in turn.

```text
# item-13-exact.log, selected rows before promote
zfsowed13/home/checkout@origin	15	11050373805378911967
zfsowed13/home/checkout#from-earliest	13	11477418097495754031
# Both reads of the clone's createtxg and guid, in that order
17
7828944690444299989
# All snapshots and the bookmark under the promoted clone
zfsowed13/home/worktree-clone-datasets/promotion@earliest	13	11477418097495754031
zfsowed13/home/worktree-clone-datasets/promotion@origin	15	11050373805378911967
zfsowed13/home/worktree-clone-datasets/promotion@worktree-own	18	17157080670377245572
zfsowed13/home/worktree-clone-datasets/promotion#from-earliest	13	11477418097495754031
transferred @earliest: 13 <= origin 15: True
transferred @origin: 15 <= origin 15: True
own @worktree-own: 18 > clone 17: True
bookmark tuple unchanged: True
```

Both transferred snapshots retained their `createtxg` and GUID. The checkout's later snapshot stayed on the
checkout; the clone's own snapshot stayed on the clone. The checkout acquired the promoted clone's `@origin`
as its direct origin, and the promoted clone's own origin became `-`. The bookmark moved to the promoted
clone with the same tuple and was no longer listed on the checkout. Promoting the checkout back restored the
snapshot map and printed `bookmark restored by reverse promote: True`.

The transaction-group comparisons separated transferred and own snapshots in this topology, and a snapshot-only
inventory missed the transferred bookmark. This is not proof of provenance based on `createtxg` alone for other
lineage changes, different encryption roots, or creation of multiple objects in one transaction group.

### 14. One snapshot destroy request after promote

Guest: `6.8.0-147-generic`, OpenZFS userspace and module `2.4.4-1` (`item-14-exact.log`). This item ran as root:
the log runs `zfs destroy` and `/usr/bin/python3 /root/guest-delta-probe.py` from the root shell, and drops to the
account with `runuser` only to create its own fixture directory. The probe repeated item 11's promote with an
extra worktree-owned snapshot for the positive control. It traced each destroy with
`strace -X raw -e trace=ioctl` and counted `0x5a3b`, `ZFS_IOC_DESTROY_SNAPS` in OpenZFS `include/sys/fs/zfs.h`.
`zfs destroy zfsowed14/home/worktree-clone-datasets/promotion@earliest,origin` exited 1:

```text
# item-14-exact.log, selected mixed CLI output
cannot destroy 'zfsowed14/home/worktree-clone-datasets/promotion@origin': snapshot has dependent clones
ZFS_IOC_DESTROY_SNAPS (0x5a3b) count: 0
```

The CLI refused before submitting the batch, so this command did not measure kernel atomicity. In the `zfs-2.4.4`
source, `snapshot_to_nvl_cb` in `cmd/zfs/zfs_main.c` checks dependents while gathering the snapshot names.
To exercise the kernel, the guest helper submitted exactly those two names in one `lzc_destroy_snaps` call
with `defer=false` (`argtypes` and call at `guest-delta-items.py:156` to `:165`). That the counted ioctl carried no
`defer` key is reasoning, not something the trace read: it ran with `-X raw` and shows the request's address, so the
inference rests on the helper's `defer=false` call, on `libzfs_core` adding the key only when it defers
(`lib/libzfs_core/libzfs_core.c:475` to `:476`, `zfs-2.4.4`), and on the guest userspace being that release.
It returned `EEXIST`, with only `@origin` in the per-name errors:

```text
# item-14-exact.log, selected direct-call output
zfsowed14/home/worktree-clone-datasets/promotion@origin: 17 (File exists)
ioctl(3, 0x5a3b, 0x7ffe98762ca0)        = -1 EEXIST (File exists)
ZFS_IOC_DESTROY_SNAPS (0x5a3b) count: 1
```

`zfs list -H -p -t snapshot -o name,createtxg,guid -r` showed the full set unchanged after both refusals,
including the same `@earliest` and `@origin`:

```text
# item-14-exact.log, selected rows after the direct call
zfsowed14/home/worktree-clone-datasets/promotion@earliest	13	1397524674383436074
zfsowed14/home/worktree-clone-datasets/promotion@origin	14	7714848456831525633
```

Positive control: `zfs destroy <promoted-clone>@worktree-own,worktree-own-next` exited 0 with
`ZFS_IOC_DESTROY_SNAPS (0x5a3b) count: 1`. Both own snapshots disappeared and no others did. This supports
DM2's all-or-nothing result for this kernel request, but not its premise that the refused CLI command issues it.
It does not prove ownership, make the subsequent dataset destroy atomic with the batch, test batch limits,
or establish safety if an unprotected foreign snapshot appears in a submitted set. Because every call ran as root,
it does not show that the delegated account may submit the same batch under the proposed grants; the only
delegated destroy measured is item 12's head destroy.

### 15. Mount namespaces created after attach

Guest: `6.8.0-147-generic`, OpenZFS userspace and module `2.4.4-1` (`item-15-exact.log`). Both arms ran in the
VM boot namespace so that PID 1 could start the reader unit after attach. `/`, `/home` and the fixture home
were shared; staging was private. After attaching with the descriptor calls from item 3, the probe started
`unshare --mount --propagation unchanged -- sleep 65` and a bounded `systemd-run` unit with
`--property=PrivateMounts=yes`. Both readers inherited the clone; neither worked in the clone directory.
`unchanged` preserves peer propagation for the shared control instead of making that reader recursively private.

With a private checkout, a fresh process moved the clone mount to staging and unmounted it. The clone vanished
from the first namespace, but **both** readers' `/proc/<pid>/mountinfo` still had a ZFS row for the clone at the
slot. As guest root, the probe grouped PIDs by `/proc/<pid>/ns/mnt`, read a representative `mountinfo` for each,
and matched the filesystem and source. After unmount it found both remaining holders:

```text
# item-15-exact.log, selected rows after private-checkout move and unmount
clone mount ID 366 absent from mountinfo after unmount
private checkout descriptor move/unmount result: 0
first namespace after removal: mnt:[4026531841] /proc/127917/mountinfo; hits=0
unshare after removal: mnt:[4026532396] /proc/127996/mountinfo; hits=1
service after removal: mnt:[4026532397] /proc/127998/mountinfo; hits=1
root scan: inspected 11 distinct namespaces; 0 unreadable or vanished PID entries
root scan: mnt:[4026532396] representative /proc/127996/mountinfo; hits=1
root scan: mnt:[4026532397] representative /proc/127998/mountinfo; hits=1
```

With the readers alive, the probe created and destroyed a disposable own snapshot (both exit 0), and confirmed
it was gone. `zfs destroy` of the clone then exited 1. After stopping both readers, no scanned namespace still
listed it and retrying the same clone destroy exited 0:

```text
# item-15-exact.log, selected destroy output while readers were alive
cannot open 'zfsowed15private/home/worktree-clone-datasets/post-attach@worktree-own': dataset does not exist
cannot destroy 'zfsowed15private/home/worktree-clone-datasets/post-attach': dataset is busy
private checkout destroy with readers alive: 1
```

With a shared checkout, the `unshare` reader was a peer and the unit's copy had a `master` pointing to the
first namespace's group. The descriptor move refused with `EINVAL`, as in item 3. A plain path `umount` in
the first namespace was used only as the shared propagation control. While both readers were still alive:

```text
# item-15-exact.log, selected shared-checkout control output
move_mount: EINVAL
shared checkout descriptor move/unmount result: 22
shared control: path umount after refused move
unshare after removal: mnt:[4026532396] /proc/128086/mountinfo; hits=0
service after removal: mnt:[4026532397] /proc/128088/mountinfo; hits=0
root scan: inspected 11 distinct namespaces; 0 unreadable or vanished PID entries
shared checkout destroy with readers alive: 0
```

All scanned namespaces had zero hits after the shared unmount. DM3's distinction holds: a private move and
unmount can succeed locally while another namespace keeps the clone busy, and snapshot destruction can succeed
before the clone destroy refuses. The scan detected the live, process-visible holders in this VM. It was not
a race-free proof of absence and did not test namespace references without a visible member PID, processes
with different root directories, the default `unshare --mount` propagation, or the installed Pi driver.

### 16. Availability of a guest with the host kernel

The guest used for items 12 to 15 booted `6.8.0-147-generic` and printed `zfs-2.4.4-1` and `zfs-kmod-2.4.4-1`.
The host, read-only, reported `7.3.5-1-cachyos` from `uname --kernel-release` and `2.4.4-1` from
`/sys/module/zfs/version`. The package ownership and tool queries returned:

```text
# pacman --query --owns on the host kernel image and ZFS module
/usr/lib/modules/7.3.5-1-cachyos/vmlinuz is owned by linux-cachyos 7.3.5-1
/usr/lib/modules/7.3.5-1-cachyos/extramodules/zfs.ko.zst is owned by linux-cachyos-zfs 7.3.5-1
# pacman --query linux-cachyos linux-cachyos-headers zfs-utils
linux-cachyos 7.3.5-1
linux-cachyos-headers 7.3.5-1
zfs-utils 2.4.4-3
```

`mvm create --help` said `Image to use: ubuntu (default), fedora, alpine, windows, or a custom template name`.
The [image documentation](../../package/mvm/cli/README.md#available-images) and `IMAGES` in
`package/mvm/cli/src/registry.ts` resolve those to Ubuntu 24.04 LTS, Fedora 43 Cloud Base, Alpine 3.23 and
Windows Server 2025. There is no `linux-cachyos` image in that libvirt registry. `ls --format=single-column`
of `~/.local/share/mvm/images/`, the `resolveImage` custom-template directory, returned only:

```text
# ls --format=single-column -- ~/.local/share/mvm/images
noble-server-cloudimg-amd64.img
template-ubuntu.qcow2
```

No offered local image boots the target package combination. The Hetzner backend defines no CachyOS shorthand;
its remote catalog and arbitrary image IDs were not queried or provisioned. The `mvm` custom-template route
would need a bootable x86-64 CachyOS qcow2 with `linux-cachyos` `7.3.5-1`, its matching `linux-cachyos-zfs`
module (or the same OpenZFS release built against the exact guest kernel headers), OpenZFS `2.4.4` userspace,
and an enabled `qemu-guest-agent`. It must boot that kernel, not just have the package on disk. Verify
`uname --kernel-release` and `zfs version` in that guest before repeating item 1 with a disposable encrypted
file-backed pool. No exact-kernel guest was available, so item 1's calls were not rerun. Matching the module
release in the Ubuntu guest does not establish that the host kernel passes the calls.

### What this repeat did not show

- No run on `7.3.5-1-cachyos`, because no built-in or locally supplied image booted it. No probe ran against
  the host's `zpcachyos` pool.
- No product-level removal, handler readiness gate, or concurrency guarantee. The same-name test sequenced
  the substitution; the privileged program was a fixture, not a delegated primitive or a production service.
- No universal transaction-group provenance rule, destroy batch limit or proof against a change between
  inspection and use. Only the selected promote, batch and hold cases were exercised.
- No snapshot batch submitted by the delegated account under the proposed grants. Item 14 ran its batch as
  root, and the only delegated destroy measured is item 12's head destroy; the account's permission on a clone's
  snapshots under the `-d` grants is unmeasured.
- No complete namespace or mount-reference exclusion. The guest scan had readable, live representatives
  for the observed namespaces; the result does not generalize to pinned namespaces or racing task changes.

## 17. Delegated snapshot batch, 2026-11-19T00:44:01-05:00

This non-final, unapproved section answers finding L6 of the second delta review of `7fe84c7be8`: item 14's batch
ran as root, and the delegated account had been measured destroying only a head (item 12). It also answers the
delta section's "What this repeat did not show" bullet about a delegated batch. The guest was
`zfs-item-17-measurements`, an Ubuntu 24.04 `mvm` libvirt VM, destroyed after the run
(`item-17/keep/mvm-destroy.log`). Scripts are in
`~/temp/agent/zfs-measurements-owed-before-acceptance-2026-11-18/item-17/`: `guest-probe.py` and
`guest-delta-items.py` are unchanged copies of the kept harness, `guest-item-17-items.py` adds the arms and
`run-item-17.ts` runs them through `mvm exec` in a private mount namespace, and `keep/item-17-exact-3.log` holds
every command and output quoted here. The host, read-only, reported `7.3.5-1-cachyos` from
`uname --kernel-release` and `2.4.4-1` from `/sys/module/zfs/version` in this run.

The run printed its own versions before any pool existed:

```text
# item-17-exact-3.log
$ uname -r
6.8.0-147-generic
$ cat /sys/module/zfs/version
2.4.4-1
$ zfs version
zfs-2.4.4-1
zfs-kmod-2.4.4-1
```

The guest kernel differs from the host's. OpenZFS userspace and module came from the `zfs-2.4.4` release
tarball built against that guest kernel, and their release matches the host module's.

### Item 1: one batch by the delegated account

The pool was `zfsowed17` in the fixture layout above: `aes-256-gcm` with a raw key file, `recordsize=32K`,
`compression=zstd-3`, a `legacy`, `canmount=off` clone parent, and the checkout dataset mounted under the guest
home. Guest: `6.8.0-147-generic`, OpenZFS userspace and module `2.4.4-1` (`item-17-exact-3.log`). Root set item
12's three `zfs allow` grants and no others, then printed them:

```text
# item-17-exact-3.log
$ zfs allow zfsowed17/home/worktree-clone-datasets
---- Permissions on zfsowed17/home/worktree-clone-datasets -----------
Local permissions:
	user ubuntu create
Descendent permissions:
	user ubuntu destroy,userprop
Local+Descendent permissions:
	user ubuntu mount
$ runuser --preserve-environment -u ubuntu -- id
uid=1000(ubuntu) gid=1000(ubuntu) groups=1000(ubuntu)
```

The proposal's delegation section also lists local `snapshot` on the checkout dataset. It was not set, as in item
12, because root made every snapshot. That grant could not reach a clone's snapshots either: the checkout dataset
is not an ancestor of the clone, and a snapshot's permission is checked as a descendant grant from its own dataset
upward (`module/zfs/dsl_deleg.c:576`, `zfs-2.4.4`).

Root then made the clone as items 11 and 14 do, with `@earliest`, `@origin` and `@checkout-later` on the checkout,
`zfs clone <checkout>@origin <clone>`, `@worktree-own` and `@worktree-own-next` on the clone, and
`zfs promote <clone>`. The clone stayed unmounted, and the promotion left the checkout as a clone of it:

```text
# item-17-exact-3.log, before the batch
zfsowed17/home/checkout	origin	zfsowed17/home/worktree-clone-datasets/removal@origin
zfsowed17/home/worktree-clone-datasets/removal	mounted	no
zfsowed17/home/worktree-clone-datasets/removal@earliest	17	5119344664520718075
zfsowed17/home/worktree-clone-datasets/removal@origin	18	11828178329495998103
zfsowed17/home/worktree-clone-datasets/removal@worktree-own	21	8296695595300551637
zfsowed17/home/worktree-clone-datasets/removal@worktree-own-next	22	6366918254970259933
```

The account then ran one `python3` process under `strace -X raw -e trace=ioctl`. It submitted both
`@worktree-own` and `@worktree-own-next` through `lzc_destroy_snaps` with `defer=false`, the same direct path item
14 used (`guest-delta-items.py:164`); `libzfs_core` adds no `defer` key for that value
(`lib/libzfs_core/libzfs_core.c:475` to `:476`, `zfs-2.4.4`). Its output, with the names elided in the middle:

```text
# item-17-exact-3.log
batch process uid=1000
lzc_destroy_snaps([... '@worktree-own', ... '@worktree-own-next'], defer=false): 0 (Success)
ioctl(3, 0x5a3b, 0x7ffddc554040)        = 0
ZFS_IOC_DESTROY_SNAPS (0x5a3b) count: 1
```

No per-name error entries were printed: the helper prints them only when the call filled an error list
(`guest-delta-items.py:166` to `:172`). The snapshot list afterwards, in the log's order and with the command line
elided:

```text
# item-17-exact-3.log, excerpt
zfsowed17/home/worktree-clone-datasets/removal@earliest	17	5119344664520718075
zfsowed17/home/worktree-clone-datasets/removal@origin	18	11828178329495998103
item 1 process exit: 0
item 1 named snapshots still present: []
```

It shows that the account holding exactly these grants destroyed both snapshots made on the clone in one kernel
request, with return value 0 and one `ZFS_IOC_DESTROY_SNAPS` call, and that the transferred origin history stayed.
It does not show a held snapshot, a `defer` key, names from more than one dataset, a set larger than two, an
unpromoted clone, another OpenZFS release, or the host kernel. Why the grants allow it is a source reading, not
this run: the request's policy requires `mount` and `destroy` for each name (`module/zfs/zfs_ioctl.c:824` to
`:834` and `:843` to `:877`), and a snapshot is checked for descendant grants only
(`module/zfs/dsl_deleg.c:576`), which the clone parent's `-d mount,destroy` supplies.

### Item 2: one named snapshot has a dependent clone

Root created `@with-dependent` and `@without-dependent` on the clone and cloned `@with-dependent` as
`zfsowed17/home/worktree-clone-datasets/dependent`; the log quotes that clone's `origin` and `mounted no`. Guest:
`6.8.0-147-generic`, OpenZFS userspace and module `2.4.4-1` (`item-17-exact-3.log`). The account then sent the
same direct batch naming both snapshots:

```text
# item-17-exact-3.log, names elided
batch process uid=1000
lzc_destroy_snaps([...], defer=false): 17 (File exists)
zfsowed17/home/worktree-clone-datasets/removal@with-dependent: 17 (File exists)
ioctl(3, 0x5a3b, 0x7ffc9c27ddf0)        = -1 EEXIST (File exists)
ZFS_IOC_DESTROY_SNAPS (0x5a3b) count: 1
item 2 process exit: 17
item 2 named snapshots still present: ['...@with-dependent', '...@without-dependent']
```

It shows that the account's batch kept the all-or-nothing result item 14 measured as root: one dependent clone made
the request return `EEXIST`, the per-name error named only the dependent snapshot, one request was sent, and the
snapshot without a dependent was not deleted either. It does not show a user hold instead of a clone, several
dependent names in one request, or a name outside the account's grants.

### Item 3: the same request as root

After item 1, root created `@root-control-one` and `@root-control-two` and ran the same helper and trace as
`uid=0`. Guest: `6.8.0-147-generic`, OpenZFS userspace and module `2.4.4-1` (`item-17-exact-3.log`):

```text
# item-17-exact-3.log, names elided
batch process uid=0
lzc_destroy_snaps([...], defer=false): 0 (Success)
ioctl(3, 0x5a3b, 0x7ffc3a904b90)        = 0
ZFS_IOC_DESTROY_SNAPS (0x5a3b) count: 1
item 3 process exit: 0
item 3 named snapshots still present: []
```

It is the control that the request the account sent is well formed on this guest: the same call shape returned 0 and
deleted both names as root too. It does not show a root-only refusal for a name outside a grant.

### Item 4: not needed

Item 1 was accepted, so no refusal had a permission to name and no grant was added. The log records
`item 4: not run, item 1 was accepted`. Which permission such a refusal would name, and whether adding only a
descendant `destroy` grant on each clone would change it, remain unmeasured: no run in this record produced that
refusal.

### What item 17 did not show

- No user hold, `defer` key, larger set, split or batch limit. Item 14 left request limits unmeasured, and they
  remain unmeasured.
- No route through the proposed binding, the root handler or a conditional channel program; the helper is the
  fixture item 14 used, called as the account.
- No unpromoted clone, other clone topology or encryption root, other OpenZFS release, or `7.3.5-1-cachyos`.
  [Item 18](#18-delegated-batch-unpromoted-clone-2026-11-19t012534-0500) measured an unpromoted clone in a guest of
  its own.
- No head destroy after the batch, no interleaving with a snapshot created between inspection and request, and
  no interleaving with a mount or unmount. The limits of items 14 and 15 stand.
- No request naming a transferred snapshot, a foreign snapshot or one outside the clone parent. The design names
  only the clone's own snapshots; what the same account could do to a transferred one under these grants was not
  measured.

## 18. Delegated batch, unpromoted clone, 2026-11-19T01:25:34-05:00

This non-final, unapproved section answers the third bullet of item 17's "What item 17 did not show" and the
unpromoted-clone limitation item 17 states under its item 1: removal destroys verified unpromoted clones, and no
unpromoted clone had been measured. The fixture, account, grants and direct `lzc_destroy_snaps` path are item
17's; the removal arms differ only in that the clone is never promoted, and `lists` calls were added to print both
datasets' snapshot names after each request (`guest-item-18-items.py`, against unchanged copies of `guest-probe.py`
and `guest-delta-items.py`). The guest was `zfs-item-18-measurements`, an Ubuntu 24.04 `mvm` libvirt VM,
destroyed after the run (`keep/mvm-destroy.log`). Scripts are in
`~/temp/agent/zfs-measurements-owed-before-acceptance-2026-11-18/item-18/`: `prepare-item-18.ts` prepared the guest
and built OpenZFS, `run-item-18.ts` pushed the probe and ran the arm, and `keep/item-18-exact.log` holds every
command and output quoted here. The host, read-only, reported `7.3.5-1-cachyos` from `uname --kernel-release` and
`2.4.4-1` from `/sys/module/zfs/version` in this run.

The run printed its own versions before any pool existed:

```text
# item-18-exact.log
$ uname -r
rc=0
6.8.0-147-generic
$ cat /sys/module/zfs/version
rc=0
2.4.4-1
$ zfs version
rc=0
zfs-2.4.4-1
zfs-kmod-2.4.4-1
```

The guest kernel differs from the host's. `prepare-item-18.ts` built the `zfs-2.4.4` release tarball in that
guest, configuring `--with-linux` for the guest's own kernel headers, and both userspace and module report release
`2.4.4-1`, the host module's release (`keep/guest-build-driver.log`).

### Grants and layout before the batch

The pool was `zfsowed18`, in the layout item 17 used: `aes-256-gcm` with a raw key file, `recordsize=32K`,
`compression=zstd-3`, a `legacy`, `canmount=off` clone parent, and the checkout dataset mounted under the guest
home. Guest: `6.8.0-147-generic`, OpenZFS userspace and module `2.4.4-1` (`item-18-exact.log`). Root set item 12's
three `zfs allow` grants and no others, then printed them:

```text
# item-18-exact.log, selected grant rows
$ zfs allow zfsowed18/home/checkout
---- Permissions on zfsowed18/home/checkout --------------------------
Descendent permissions:
	user ubuntu clone
$ zfs allow zfsowed18/home/worktree-clone-datasets
---- Permissions on zfsowed18/home/worktree-clone-datasets -----------
Local permissions:
	user ubuntu create
Descendent permissions:
	user ubuntu destroy,userprop
Local+Descendent permissions:
	user ubuntu mount
$ runuser --preserve-environment -u ubuntu -- id
rc=0
uid=1000(ubuntu) gid=1000(ubuntu) groups=1000(ubuntu)
```

As in items 12 and 17, the checkout's local `snapshot` grant was not set, because root made every snapshot, and it
could not reach a clone's snapshots either: the checkout dataset is not an ancestor of the clone, and a
snapshot's permission is checked as a descendant grant from its own dataset upward (`module/zfs/dsl_deleg.c:576`,
`zfs-2.4.4`).

Root made the clone as items 11 and 14 do, with `@earliest`, `@origin` and `@checkout-later` on the checkout,
`zfs clone <checkout>@origin <clone>`, then `@worktree-own` and `@worktree-own-next` on the clone, and did not
promote it. The clone stayed unmounted, and the checkout kept all three snapshots and its own origin role:

```text
# item-18-exact.log, before the batch
zfsowed18/home/checkout	origin	-
zfsowed18/home/checkout	mounted	yes
zfsowed18/home/worktree-clone-datasets/removal	origin	zfsowed18/home/checkout@origin
zfsowed18/home/worktree-clone-datasets/removal	mounted	no
zfsowed18/home/worktree-clone-datasets/removal@worktree-own	21	18084574447304504731
zfsowed18/home/worktree-clone-datasets/removal@worktree-own-next	22	3555362453517264833
zfsowed18/home/checkout@earliest	17	14504240285413109318
zfsowed18/home/checkout@origin	18	16453277525178033700
zfsowed18/home/checkout@checkout-later	19	9491771657209599626
```

The clone's snapshot list held only the two names made on it, so the set a removal batch builds from that list is
exactly the clone's own snapshots. `@earliest` and `@origin` are listed under the checkout and not under the
clone, which is the difference promotion made in item 17: there both transferred names were listed under the
promoted clone and stayed after its own snapshots were destroyed.

### Item 1: one batch by the delegated account

Guest: `6.8.0-147-generic`, OpenZFS userspace and module `2.4.4-1` (`item-18-exact.log`). The account then ran one
`python3` process under `strace -X raw -e trace=ioctl`. It submitted both `@worktree-own` and
`@worktree-own-next` through `lzc_destroy_snaps` with `defer=false`, the same direct path item 14 used
(`guest-delta-items.py:164`); `libzfs_core` adds no `defer` key for that value
(`lib/libzfs_core/libzfs_core.c:475` to `:476`, `zfs-2.4.4`). Its output, with the names elided in the middle:

```text
# item-18-exact.log
batch process uid=1000
lzc_destroy_snaps([... '@worktree-own', ... '@worktree-own-next'], defer=false): 0 (Success)
ioctl(3, 0x5a3b, 0x7ffe2fd7b320)        = 0
ZFS_IOC_DESTROY_SNAPS (0x5a3b) count: 1
item 1 process exit: 0
item 1 named snapshots still present: []
```

No per-name error entries were printed: the helper prints them only when the call filled an error list
(`guest-delta-items.py:166` to `:172`). The snapshot lists afterwards, in the log's order:

```text
# item-18-exact.log, excerpt
clone snapshots: []
zfsowed18/home/checkout@earliest	17	14504240285413109318
zfsowed18/home/checkout@origin	18	16453277525178033700
zfsowed18/home/checkout@checkout-later	19	9491771657209599626
```

It shows that the account holding exactly these grants destroyed both snapshots made on the unpromoted clone in
one kernel request, with return value 0 and one `ZFS_IOC_DESTROY_SNAPS` call, that the per-name error list was
empty, and that the checkout's three snapshots kept their transaction groups and GUIDs. It does not show a held
snapshot, a `defer` key, names from more than one dataset, a set larger than two, another OpenZFS release, or the
host kernel. Why the grants allow it is a source reading, not this run: the request's policy requires `mount` and
`destroy` for each name (`module/zfs/zfs_ioctl.c:824` to `:834` and `:843` to `:877`), and a snapshot is checked
for descendant grants only (`module/zfs/dsl_deleg.c:576`), which the clone parent's `-d mount,destroy` supplies.

### Item 2: one named snapshot has a dependent clone

Root created `@with-dependent` and `@without-dependent` on the clone and cloned `@with-dependent` as
`zfsowed18/home/worktree-clone-datasets/dependent`; the log quotes that clone's `origin` and `mounted no`. Guest:
`6.8.0-147-generic`, OpenZFS userspace and module `2.4.4-1` (`item-18-exact.log`). The account then sent the same
direct batch naming both snapshots:

```text
# item-18-exact.log, names elided
batch process uid=1000
lzc_destroy_snaps([...], defer=false): 17 (File exists)
zfsowed18/home/worktree-clone-datasets/removal@with-dependent: 17 (File exists)
ioctl(3, 0x5a3b, 0x7ffcf58cd730)        = -1 EEXIST (File exists)
ZFS_IOC_DESTROY_SNAPS (0x5a3b) count: 1
item 2 process exit: 17
item 2 named snapshots still present: ['...@with-dependent', '...@without-dependent']
```

It shows that on an unpromoted clone the account's batch kept the all-or-nothing result items 14 and 17
measured: one dependent clone made the request return `EEXIST`, the per-name error named only the dependent
snapshot, one request was sent, and the snapshot without a dependent was not deleted either. The checkout's three
snapshots kept their transaction groups and GUIDs. It does not show a user hold instead of a clone, several
dependent names in one request, or a name outside the account's grants.

### Item 3: the same request as root

After item 1, root created `@root-control-one` and `@root-control-two` on the same unpromoted clone and ran the
same helper and trace as `uid=0`. Guest: `6.8.0-147-generic`, OpenZFS userspace and module `2.4.4-1`
(`item-18-exact.log`):

```text
# item-18-exact.log, names elided
batch process uid=0
lzc_destroy_snaps([...], defer=false): 0 (Success)
ioctl(3, 0x5a3b, 0x7fffcecc5780)        = 0
ZFS_IOC_DESTROY_SNAPS (0x5a3b) count: 1
item 3 process exit: 0
item 3 named snapshots still present: []
```

It is the control that the request the account sent is well formed on this guest: the same call shape returned 0
and deleted both names as root too, and the checkout's snapshots were unchanged. It does not show a root-only
refusal for a name outside a grant.

### Item 4: not needed

Item 1 was accepted, so no refusal had a permission to name and no grant was added. The log records
`item 4: not run, item 1 was accepted`. Which permission such a refusal would name on an unpromoted clone, and
whether adding only a descendant `destroy` grant on each clone would change it, remain unmeasured.

### What item 18 did not show

- No user hold, `defer` key, larger set, split or batch limit. Item 14 left request limits unmeasured, and they
  remain unmeasured.
- No route through the proposed binding, the root handler or a conditional channel program; the helper is the
  fixture item 14 used, called as the account.
- No promoted clone in this guest. Item 17 promoted its clone in a separate guest on the same releases, so the
  two results agree across guests, not within one.
- No other clone topology or encryption root, other OpenZFS release, or `7.3.5-1-cachyos`.
- No head destroy after the batch, no interleaving with a snapshot created between inspection and request, and
  no interleaving with a mount or unmount. The limits of items 14 and 15 stand.
- No request naming a transferred snapshot or a foreign snapshot. On an unpromoted clone the transferred names
  are not in the clone's snapshot list, so a batch built from that list cannot name them; what the same account
  could do to a checkout snapshot under these grants was not measured.
## Tier 0 mounts and namespaces in a guest, 2026-11-23T05:24:20-05:00

**Measured by an agent.**
Unapproved and non-final.
This section records the first group of the design's Tier 0 arms,
T0-1 to T0-12,
measured in a disposable guest.
It decides nothing the design decides and nothing the design leaves open.
It records what the kernel did,
where that differs from the rows the design expected,
and what had to be decided to script each arm.

The arms are those of `doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md` in
the section it calls Tier 0,
for mounts and namespaces only,
read at commit `b62601213f` on the branch that carries it,
which is not this branch.

### The guest, the program and the common start

The guest was `zfs-tier0-mount-probe-20261123`,
made with `mvm --backend libvirt create` from the cached template `template-ubuntu.qcow2` and nothing
downloaded.
It reports kernel `6.8.0-147-generic`,
`Ubuntu 24.04.5 LTS`,
`mount`,
`unmount` and `findmnt` from util-linux `2.39.3`,
and the root filesystem is ext4 on `/dev/vda1` (`dev=253:1` in every row quoted here).
The host this record was written on runs `7.3.5-1-cachyos`;
every row in this section is a row on `6.8.0-147-generic`,
and a result on `6.8` is a result on `6.8`,
not on the production kernel.

A second guest was made for a newer kernel,
`zfs-tier0-mount-probe-fedora-20261123`,
also through `mvm --backend libvirt` with no download from the cached `template-fedora.qcow2`.
It reports `6.17.1-300.fc43.x86_64`,
`Fedora Linux 43 (Cloud Edition)`,
btrfs on `/dev/vda4[/root]`.
The group is unrun there,
because the guest agent runs in the SELinux domain `system_u:system_r:virt_qemu_ga_t:s0` and that domain is
refused every part of the start and of the fixture:
`/usr/bin/mount --version` answers `rc=126` and `Permission denied`,
`unshare --mount --propagation slave -- true` answers
`unshare: cannot change root filesystem propagation: Permission denied`,
`mkdir --parents /var/lib/zfs-tier0-probe` answers `Permission denied`,
`cp /mnt/shared/guest-tier0.py` answers `Permission denied` even though the host pushed the file there,
and `runcon -t unconfined_t` answers `Permission denied`.
Writing under `/tmp` is allowed,
so the refusal is the mount family and the paths,
not the shell.
The refusal log is `fedora-guest-refusals.log`.

The program is `guest-tier0.py`,
kept with this group's logs in `~/temp/agent/zfs-forced-removal-first-probe-arms-in-a-guest-2026-11-23/keep/`,
and it extends the fixture's `guest-probe.py` conventions:
`ctypes` for `mount`,
`umount2`,
`move_mount`,
`name_to_handle_at`,
`open_by_handle_at`,
`statx`,
`unshare` and the netlink `sock_diag` request,
and subprocesses for `unshare`,
`mount`,
`umount` and `findmnt`.
`run-arm.ts` pushes and runs one arm per invocation through `mvm --backend libvirt exec`,
and `run-all-arms.ts` runs the group.
One arm's invocation runs its phases twice;
every phase runs in a fresh private mount namespace H,
made by `unshare(CLONE_NEWNS)` and then `mount(NULL, "/", NULL, MS_REC|MS_PRIVATE, NULL)`,
so nothing an arm mounts reaches the guest's own namespaces.
A namespace that must be a slave is made with `unshare --mount --propagation slave` from its parent,
one that must be a peer with `--propagation unchanged`.
Each phase prints the tables of H and of every namespace it made before its action,
checks the relation its rows assume (which mount carries a shared group,
which is a slave of which),
and runs the action only when the check passes;
the checks and tables are in the logs.

The common start,
in H:
`mount --bind $B/src $B/parent`,
`mount --make-private $B/parent`,
`mount --make-shared $B/parent`,
and the check that the parent carries a `shared:` group no other row of H carries.
In this section P is that mount,
"the copy" is a bind of `$B/copy` onto `$B/parent/slot`,
"the slot" is that path,
and U,
S and V are the namespaces an arm makes.
In the quoted rows,
`$B` stands for the per-run fixture base `/var/lib/zfs-tier0-probe/<arm>/run-<n>-<phase>`,
and the mount ids,
parent ids and group numbers are the kernel's own for that run,
so they differ between runs and between arms while the relation does not.

Each arm ran twice,
in two runs from a fresh H.
`summarize_runs.py` compared the observed lines of the two runs of every arm,
after replacing `$B` and the kernel-chosen numbers,
and all 15 logs report `identical = True`.
The logs are `arm-<arm>-final-zfs-tier0-mount-probe-20261123.log`,
and the first full pass with fewer controls is kept in `superseded-measure-pass-1/`.
The group ran on a second lifetime of the same guest name and the same cached template after the first
lifetime was destroyed;
the kernel,
the distribution and the tools were identical.

### T0-1. A plain unmount reaches the parent's receivers

Expected:
S's copy vanishes,
and the host bind and P stay in H.
The positive control ran first,
in its own phase on the same constructed start:
`umount2` of the host bind at a second path under P in H removed its row,
and the check `the host bind vanished from H's table` passed.

Observed,
U and S before the action and both after it in U:

```text
# arm-T0-1-final-zfs-tier0-mount-probe-20261123.log, run 1, phase plain-unmount-in-U
    id=436 parent=412 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[shared:347 master:345]
    id=438 parent=436 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[shared:348]
    id=464 parent=440 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:347]
    id=466 parent=464 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[master:348]
# action: umount2(the copy) in U, plain
    id=436 parent=412 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[shared:347 master:345]
    id=464 parent=440 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:347]
    id=409 parent=97 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[shared:345]
# observed: U slot gone = True ; S slot gone = True
# observed: H still holds the parent = True ; H still holds the host bind = True
```

The expectation holds:
a plain unmount in U removed the copy in U and in S,
S being a slave of a slave,
and H's rows stayed.
The two runs agree.

Decisions needed to script it:
the control unmounts the very host bind whose survival the expectation names,
so the control is its own phase on the same constructed start and the action phase mounts the bind again;
the copy is a bind of a directory on the guest's root filesystem;
the row at the slot is recognised as the copy by its `root=` ending in `/copy`,
and among rows at one mountpoint the topmost is the greatest mount id.

### T0-2. A private parent before the unmount stops it

Expected:
S's copy stays mounted.
The control is T0-1's plain unmount on the same start,
which removed the copy in U and in S (`U slot gone = True ; S slot gone = True`).

Observed,
the change and the action in U:

```text
# arm-T0-2-final-zfs-tier0-mount-probe-20261123.log, run 1, phase action-private-parent
    U parent row after the change: id=436 parent=412 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[]
    id=464 parent=440 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:345]
    id=466 parent=464 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[master:348]
    id=464 parent=440 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:345]
    id=466 parent=464 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[]
# observed: U slot gone = True ; S slot gone = False
```

The expectation holds,
and the two runs agree.
Also observed:
the private change re-points S's parent row from `master:347` to `master:345`,
so U's slaves are handed to the master U's parent had.
Decisions needed to script it:
the change is ctypes `mount(NULL, P, NULL, MS_PRIVATE, NULL)` exactly as the text names it,
and the control is its own phase because it destroys the state the action needs.

### T0-3. A slave parent before the unmount stops it

Expected:
S's copy stays in both variants,
variant (a)'s P row reads `master:<P's group in H>`,
and variant (b)'s reads as private with no `master:`.
The control named is T0-1's plain unmount on each variant's start.

Variant (a):
the start is row for row T0-1's,
`tags=[shared:346 master:345]` in U and `tags=[master:346]` in S,
and the plain unmount control removes S's copy.
The action then leaves the same tables,
and S's copy is gone:

```text
# arm-T0-3-final-zfs-tier0-mount-probe-20261123.log, run 1
    id=435 parent=411 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[shared:346 master:345]
# observed: U slot gone = True ; S slot gone = True
```

This differs from the arm's expectation,
which says the copy stays.
It is also a conflict in the text:
variant (a)'s start and action are T0-1's,
whose expectation is that S's copy vanishes.
The row the arm names for variant (a) is observed exactly.

Variant (b):
the parent is made private and then slave in U before the copy is mounted,
and both U's and S's parent rows carry no tag at all.
The action leaves S's copy mounted:

```text
# arm-T0-3-final-zfs-tier0-mount-probe-20261123.log, run 1, phase variant-b
    variant (b) U parent row: id=435 parent=411 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[]
    variant (b) S parent row: id=462 parent=438 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[]
# observed: U slot gone = True ; S slot gone = False
```

Variant (b) has a control of its own,
because the control the arm names runs on a different start:
on variant (b)'s start a plain unmount in U also leaves S's copy mounted
(`# control observed: U slot gone =  True ; S slot gone =  False`).
So variant (b)'s "the copy stays" is the state's,
not the slave change's:
the parent already has no master and no group before any unmount-order question.
Both runs agree in both variants and in both controls.
Decisions needed to script it:
the two changes in variant (b) are made before the copy is mounted,
as the start reads,
and variant (b) gets its own control phase.

### T0-4. The peer case's rows

Expected,
derived in the design from `fs/pnode.c` and not previously run:
P's row becomes `master:<the former group>` without `shared:`,
P' keeps its `shared:` and its own `master:`,
S's row still prints `master:<the former group>`,
after the change a mount under P' no longer reaches P or S,
and one under P reaches neither.
The control recorded delivery under P' first.

```text
# arm-T0-4-final-zfs-tier0-mount-probe-20261123.log, run 1
    P row after the change:  id=409 parent=97 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:345]
    P' row after the change: id=410 parent=97 dev=253:1 root=$B/src mp=$B/peer fstype=ext4 source=/dev/vda1 tags=[shared:345]
    S row after the change:  id=436 parent=412 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:345]
# control delivery under P': {"H/P": true, "H/P'": true, "S/P": true}
# delivery under P' after the change: {"H/P": true, "H/P'": true, "S/P": true}
# delivery under P after the change: {"H/P": true, "H/P'": false, "S/P": false}
```

The rows the arm derives are all observed exactly.
The delivery claim differs:
the design expects a mount under P' to stop reaching P and S after the change,
and the guest delivers it to both.
P's row carries `master:347` for it and S's row carries `master:347`,
so P is a slave of P' and still a receiver of P',
and S keeps receiving through its own master.
The delivery under P matches the expectation.
Both runs agree.
Decisions needed to script it:
the copy is mounted in H under P,
because the checks are about delivery into P,
P' and S and the text names no namespace for it;
S is made from H with `unshare --mount --propagation slave` before the copy is mounted,
so S's copy arrives by propagation;
and the control's probe is a bind at `$B/peer/probe`,
withdrawn before the change.

### T0-5. The receiver closure and the handle unmount

The oracle is a closure over the tables:
the seed is the parent's own `shared:` group when the parent is shared,
or the group its slaves print as their `master:` when it is not;
a namespace is a member when its parent row's `shared:` or `master:` is in the set,
a member adds its own `shared:` group,
and only members whose topmost mount at the slot is the copy are kept.
`summarize_runs.py` and the program's `receiver_set` implement that rule.

#### T0-5a. The nested-in-nested receiver

Expected:
the closure finds V,
and the set of namespaces whose copy vanished equals it.

```text
# arm-T0-5a-final-zfs-tier0-mount-probe-20261123.log, run 1, phase nested-in-nested
# name_to_handle_at($B/parent/slot): handle_bytes=8 handle_type=1 mount_id=409 handle=e8010400fa74f364
# relation: U parent row = id=435 parent=411 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[shared:346 master:345]
# relation: V parent row = id=462 parent=438 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:346]
# control delivery to V: true
# control withdrawal from V: true
# oracle: the parent in U is shared, seed=[346]
# oracle: the closure is ['U', 'V'], members with a mount at the slot are ['U', 'V'], the kept receivers are ['U', 'V']
    open_by_handle_at on an O_PATH directory descriptor: EBADF
    open_by_handle_at on an O_RDONLY directory descriptor: opened
    open_by_handle_at gave $B/src/slot, 0 row(s) there
    umount2(/proc/self/fd/7): umount2(/proc/self/fd/7, 0): EINVAL
# observed after attempt 1: U slot still mounted = true
    open_by_handle_at gave $B/parent/slot, 1 row(s) there
    the topmost mount at its dentry by greatest mount id: id=436 parent=435 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[shared:347]
# handle result on the parent mount: {"opened": true, "resolved": "$B/parent/slot", "row_count": 1, "top": "id=436 parent=435 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[shared:347]", "top_id": 436, "top_fstype": "ext4", "top_source": "/dev/vda1", "umount": {"ok": true}}
# observed: namespaces whose slot mount vanished = ["U", "V"]
# T0-5a: closure ['U', 'V'], kept ['U', 'V'], vanished ['U', 'V'], equal = True
```

The expectation holds on this start,
and the two runs agree.
Three things the design does not say were measured here.
The handle is opened on the root of the mount that carries the slot,
which is what way 8's step 1 names;
the same handle opened on the filesystem root `/`,
which is what the arm's words say,
resolves to the covered directory's path on that mount,
finds no mount there,
and `umount2` answers `EINVAL`.
`open_by_handle_at` on an `O_PATH` directory descriptor is refused `EBADF` on this kernel,
and the same call on an `O_RDONLY` descriptor opens the handle.
And a handle taken before the cover still names the covered directory once the copy is mounted over it.

Decisions needed to script it:
U's parent is made shared in U,
as T0-1's start does,
because the closure cannot contain V otherwise;
the arm's literal start is measured separately,
as T0-5a-literal;
the handle is tried on the filesystem root first and on the parent mount second,
and both attempts are logged.

#### T0-5a-literal. The same arm on the start the text spells out

The text's start makes U a slave of H and does not share U's parent.
On that start a mount under U's parent does not reach V:

```text
# arm-T0-5a-literal-final-zfs-tier0-mount-probe-20261123.log, run 1
# control delivery to V: false
# the control did not deliver on this start, so this run shows nothing by seeing nothing
# oracle: the parent in U is not shared, seed=[345] from the slaves
# oracle: the closure is ['H', 'U', 'V'], members with a mount at the slot are ['U', 'V'], the kept receivers are ['U', 'V']
# observed: namespaces whose slot mount vanished = ["U"]
# T0-5a: closure ['H', 'U', 'V'], kept ['U', 'V'], vanished ['U'], equal = False
```

Both runs agree.
The arm's expectation holds only on a start whose acting namespace carries a shared group of its own,
and the design's text for T0-5a does not say to make one.

#### T0-5b. A receiver with no mount at the slot

Expected:
V is not a kept receiver,
is not refused,
and is not in the vanished set.

```text
# arm-T0-5b-final-zfs-tier0-mount-probe-20261123.log, run 1
# control delivery to V: true
CHECK ok: V no longer has a mount at the slot
# oracle: the closure is ['U', 'V'], members with a mount at the slot are ['U'], the kept receivers are ['U']
# observed: namespaces whose slot mount vanished = ["U"]
# T0-5b: closure ['U', 'V'], kept ['U'], vanished ['U'], V kept = False
```

The expectation holds,
the handle unmount was not refused,
and both runs agree.
The control the arm names,
T0-5a's V keeping its copy and appearing in the vanished set,
is supplied by T0-5a in this group;
this phase's own control shows a mount under U's parent reaching V and being withdrawn.
Decisions needed to script it:
V is made from U after the copy exists,
so it holds one,
and its copy is removed in V by a local `umount2` as the text says.

#### T0-5c. A stacked mount at the slot

Expected:
the closure script's predicate says way 8 is not chosen,
because V's topmost mount at the slot is not the copy,
and the stacked mount is untouched.

```text
# arm-T0-5c-final-zfs-tier0-mount-probe-20261123.log, run 1
# observed: V topmost at the slot = id=464 parent=463 dev=0:47 root=/ mp=$B/parent/slot fstype=tmpfs source=tmpfs tags=[]
# observed: U topmost at the slot = id=436 parent=435 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[shared:347]
# predicate: way 8 is not chosen for a namespace whose topmost mount at the slot is not the copy: {"U": true, "V": false, "H": false}
# oracle: the closure is ['U', 'V'], members with a mount at the slot are ['U', 'V'], the kept receivers are ['U']
# observed: namespaces whose slot mount vanished = ["U"]
    id=462 parent=438 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:346]
    id=464 parent=462 dev=0:47 root=/ mp=$B/parent/slot fstype=tmpfs source=tmpfs tags=[]
```

The predicate holds,
and the stacked tmpfs row stayed mounted in V.
Beyond the expectation:
the handle unmount in U removed V's copy underneath the stacked tmpfs too,
which the last two rows show,
since V's table after the action holds its parent row and the tmpfs and no copy row.
Both runs agree.
Decisions needed to script it:
the predicate is computed per namespace with a mount at the slot,
and the arm's one action is performed anyway,
so the log records the predicate and what the kernel did.

### T0-6. The shared-state difference

Expected:
the way 7 U keeps receiving H's mount and unmount of the second copy,
and the way 3 U loses both.
The control,
a mount and an unmount in H under P,
ran first in each phase and reached U and S.

```text
# arm-T0-6-final-zfs-tier0-mount-probe-20261123.log, run 1
# control delivery to U and S: {"U": true, "S": true}
# control withdrawal in U and S: {"U": true, "S": true}
    U parent row after way 7: id=435 parent=411 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:345]
# after way 7: H's mount reaches U and S = {"U": true, "S": true}
# after way 7: H's unmount of the second copy is absent in U and S = {"U": false, "S": false}
    U parent row after way 3: id=435 parent=411 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[]
# after way 3: H's mount reaches U and S = {"U": false, "S": true}
# after way 3: H's unmount of the second copy is absent in U and S = {"U": false, "S": false}
```

The expectation holds for U in both ways,
and both runs agree.
Also observed:
way 3's S keeps receiving H's mount,
because S's parent is a slave of H's group itself and not of U's parent.
Decisions needed to script it:
U is made with `unshare --mount --propagation unchanged`;
the second copy is a second bind at `$B/parent/second-copy`,
mounted and unmounted in H;
and way 7 and way 3 run as two phases from the same start.

### T0-7. The move sits after the signals

Expected:
with the move after the signals the write lands in the mounted copy and the slot's folder is empty when the
move is due;
with the move before the signals it lands in the slot's underlying folder,
which H sees.

```text
# arm-T0-7-final-zfs-tier0-mount-probe-20261123.log, run 1
# the control reads the write 10 s after the signal (the write appeared after 0.05 s)
# control observed after 0.05 s: the write is in the slot's underlying folder = True, in the copy = False
# control: H sees the write = true
# the move is due 10 s after the signal; the write appeared after 0.05 s
# observed after 0.05 s: the write is in the mounted copy = True, in the slot's underlying folder = False
# observed: the slot's folder is empty of the write when the move is due = true
# observed: H sees the new directory on the parent mount as a directory = true, mode 0o700
# observed: H sees a mount at that path = false (the move happened in U only)
```

The expectation holds,
and both runs agree.
The destination row is the copy,
at `$B/parent/.git-worktree-clone-removal-probe`,
and the slot row is gone.
Decisions needed to script it:
the blocker traps `SIGTERM`,
writes by absolute path at the slot and exits,
and its write appears about 0.05 s after the signal;
the program still waits the whole `T_wait` of 10 s before the move,
so the order is measured with the design's own wait;
the destination directory and the move follow way 7's steps 2 and 3,
with `MS_SLAVE` by descriptor,
a directory mode `0700` under the reserved prefix on the parent mount's root,
and `move_mount` with `MOVE_MOUNT_F_EMPTY_PATH|MOVE_MOUNT_T_EMPTY_PATH`;
and the phase's bound is 30 s rather than `T_wait`,
because it waits `T_wait` and then does the rest.

### T0-8. A socket in another network namespace

Expected:
the bound file's device and inode name the copy's filesystem,
the owning task is found by the socket's inode in `/proc/<pid>/fd`,
and the control's socket,
bound outside the copy,
returns a device that does not match.

```text
# arm-T0-8-final-zfs-tier0-mount-probe-20261123.log, run 1
    id=363 parent=322 dev=0:43 root=/ mp=$B/parent/slot fstype=tmpfs source=tmpfs tags=[shared:292]
# the copy is tmpfs on tmpfs, dev 0:43
# the bound file inside the copy: dev 43 ino 3
# the bound file outside the copy: dev 64769 ino 262939
# inside: socket ino 10527, UDIAG_VFS ino 3 dev 43 against the file's ino 3, dev 43 (as the kernel encodes it 43): match = True
# outside: socket ino 10528, UDIAG_VFS ino 262939 dev 265289729 against the file's ino 262939, dev 64769 (as the kernel encodes it 265289729): match = True
# the socket inode 10527 is held by tasks found in /proc/<pid>/fd: [1715]
```

The expectation holds,
and both runs agree.
Decisions needed to script it:
the copy's source is a tmpfs,
so its device (`0:43`) differs from the root filesystem's (`253:1`) and the control has something to test;
the socket owner is a child of H that unshares only the network namespace and binds one socket inside the copy
and one outside;
the request is an unfiltered `SOCK_DIAG_BY_FAMILY` dump with `udiag_ino` 0 and `udiag_states` `0xffffffff`,
matched locally against the file's inode;
`setns` runs in a forked child;
and the two 32-bit words of the `UNIX_DIAG_VFS` attribute were read as the inode first and the device second,
in the kernel's own device encoding,
because that is what this kernel wrote;
the header was not read.

### T0-9. Blind and gone reads

Expected:
the root task answers `EACCES` and is blind,
the reaped pid answers `ENOENT` and is gone,
and the reader succeeds on a live same-account task.

```text
# arm-T0-9-final-zfs-tier0-mount-probe-20261123.log, run 1
# root-owned task 1739: Uid:	0	0	0	0
# same-account task 1740: Uid:	65534	65534	65534	65534
# reaped pid 1741: gone
# the reader runs as uid 65534, the account nobody
# the reader is nobody; statx on /proc/<pid>/exe answers: {"control: live same-account task": {"ok": true}, "action: root-owned task": {"ok": false, "errno": "EACCES"}, "action: reaped pid": {"ok": false, "errno": "ENOENT"}}
# control: the same reader's statx on a live same-account task's exe succeeded = true
```

The expectation holds,
and both runs agree.
Decisions needed to script it:
the same-account task is made by fork,
`setuid` to `nobody`,
then `execv("/bin/sleep")`,
because a `runuser` parent stays root and a development run that read it got `EACCES` for the control;
`statx` is called through libc with flags 0,
which follows the link,
and mask `0x7ff`,
because a no-follow stat is not the read the classifier is about.

### T0-10. The taskless namespace

Unrun on this guest.
The kernel is `6.8.0-147-generic`,
whose syscall table has no `listns`,
and syscalls 470,
471,
472 and 473 each answered `ENOSYS` in both runs.
The arm's control is the same call,
so nothing here is passed.

```text
# arm-T0-10-final-zfs-tier0-mount-probe-20261123.log, run 1
# the pinned descriptor 5 names mnt:[4026532319]; the task that made the namespace has exited
# syscall 470: ENOSYS
# syscall 471: ENOSYS
# syscall 472: ENOSYS
# syscall 473: ENOSYS
# setns on the pinned descriptor entered it; the table has 25 rows
# the mark in the pinned namespace: ["id=386 parent=323 dev=0:43 root=/ mp=/mnt/pinned-probe fstype=tmpfs source=tmpfs tags=[]"]
```

What the rest of the group shows without it:
the taskless namespace can be entered and read when a descriptor for it is kept,
so the missing piece is only the call that would find one without a descriptor.
Decisions needed to script it:
the start keeps a descriptor for a namespace whose task has exited,
handed to the arm with `SCM_RIGHTS` from the child that unshares,
and the call is attempted for numbers 470 to 473 because the design names only 470.

### T0-11. The chain: a unit started shared, then repaired

Expected:
after the repair U's copy of P has no master,
and after the way 7 change it is a slave of its next peer or of no master,
and in the no-master case the change equals way 3's.
The control,
an H mount under P reaching U before the repair,
ran first.

```text
# arm-T0-11-final-zfs-tier0-mount-probe-20261123.log, run 1
    U parent row before the repair: id=387 parent=363 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[shared:292 master:270]
# control delivery to U before the repair: true
    H parent row after the repair: id=322 parent=60 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[]
    U parent row after the repair: id=387 parent=363 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[shared:292]
    U parent row after the change: id=387 parent=363 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[]
# after the repair and the change: the mount in H under P reaches U = false
# arm-T0-11-final-zfs-tier0-mount-probe-20261123.log, run 1, phase way-3-comparison
    U parent row after the repair: id=387 parent=363 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[shared:292]
    U parent row after the change: id=387 parent=363 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[]
```

The expectation holds:
the repair clears U's `master:` tag and leaves U's own `shared:` group,
and the way 7 change then reads exactly as way 3's change does,
in the second phase from the same start.
Both runs agree.
Decisions needed to script it:
the repair is `mount --make-private` on H's P,
which has no peers here;
a second phase runs the way 3 change on the same start so the equality is measured,
not asserted;
and the phase after the repair also records that H's own mounts under P no longer reach U,
which is the repair's effect and not the slave change's.

### T0-12. A sandbox that never held the clone, after way 7

Expected:
after the act V does not receive the later mount of the service's own under P,
and the control is that the same later mount made in U before the act is received by V.

The start the text spells out cannot both keep V without a copy of the slot and deliver into V.
With U's parent carrying no group and V made before the copy,
the control does not deliver at all:

```text
# arm-T0-12-final-zfs-tier0-mount-probe-20261123.log, run 1, phase literal-start
# relation: U parent row = id=387 parent=363 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:270]
# relation: V parent row = id=413 parent=389 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:270]
# observed: V holds a mount at the slot before the act = false
# control delivery to V: false
    U parent row after the change: id=387 parent=363 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:270]
# observed: after the act V receives the later mount under P = false
```

With U's parent shared in U the control delivers,
and the copy itself propagates into V:

```text
# arm-T0-12-final-zfs-tier0-mount-probe-20261123.log, run 1, phase shared-parent-start
# relation: U parent row = id=387 parent=363 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[shared:292 master:270]
# relation: V parent row = id=413 parent=389 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:292]
# observed: V holds a mount at the slot before the act = true
# control delivery to V: true
    U parent row after the change: id=387 parent=363 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:270]
# observed: the copy is gone in U = true, gone in V = false
# observed: after the act V receives the later mount under P = false
```

The state the arm's statement is about is a namespace that never held the copy,
and it is made here as T0-5b makes it,
with V made after the copy and its copy removed in V:

```text
# arm-T0-12-final-zfs-tier0-mount-probe-20261123.log, run 1, phase copy-removed-start
# observed: V holds a mount at the slot before the act = false
# control delivery to V: true
# observed: the copy is gone in U = true, gone in V = true
# observed: after the act V receives the later mount under P = false
```

The expectation holds on that state:
the control delivered before the act,
and after way 7 in U the namespace that never held the copy receives nothing.
Both runs agree in all three phases.
Decisions needed to script it:
the three starts are run because the arm's own start cannot satisfy its control and its "no copy" at once;
the third start,
a copy removed in V,
is the one that measures the statement.

### What the group shows

A mount change under a shared mount reaches the mount's peers and its slaves,
and slaves of slaves (T0-1).
A parent made private stops that reach (T0-2),
and so does a parent that is a slave with no master,
where the reach was already stopped before any unmount-order question (T0-3 variant (b)).
A parent made a slave while it has peers keeps receiving from the peer that becomes its master,
and its own slaves keep receiving through that peer (T0-4,
T0-6).
Way 7 therefore keeps a service receiving the host's mounts and unmounts under a shared checkout,
and way 3 cuts a service off from them (T0-6).
The repair of a shared checkout clears the master of a unit's parent mount and leaves that mount's own group,
and way 7 on that mount then reads exactly as way 3 does (T0-11).

For the direction the project gave and had not confirmed,
that a forced removal unmounts the clone inside the holder's own mount namespace,
the group is positive on both recommended ways.
A plain unmount in the holder's namespace removes the copy there and in every namespace that receives from its
parent (T0-1),
and a namespace without a mount at the slot is not refused and loses nothing (T0-5b).
The handle unmount of way 8 removes the copy and its receivers' copies without changing any propagation
(T0-5a),
and it does that even where a receiver has a stacked mount at the slot,
which is where the design's predicate already refuses way 8 (T0-5c).
Way 7's move passes when the parent is a slave (T0-7),
and after it a namespace that never held the copy stops receiving the service's later mounts under the parent
(T0-12).
The host's own mount and the parent's peers are untouched by an unmount in a slave (T0-1).

Two observations differ from the design's expected rows.
T0-4's delivery under a peer after the parent becomes a slave still reaches the parent and its slave,
where the design expects it to reach neither;
the rows the arm derives are observed exactly.
T0-3 variant (a) is T0-1's start and T0-1's action,
and S's copy vanishes there,
where T0-3's text says it stays.
The design's text for T0-5a also needs U's parent shared for its own oracle to be exercisable,
and its text for T0-12 needs a receiver without a copy that its own start cannot make.

### What the group does not show

- The production kernel.
  Every row here is a row on `6.8.0-147-generic`.
  The Fedora `6.17.1` guest was
  made without a download and refused every mount-family operation to the guest agent's SELinux domain,
  so it shows nothing.
- The `listns` route,
  and therefore any taskless namespace without a kept descriptor (T0-10 is unrun).
- The handler,
  the wrapper,
  the signal stage,
  the journal and the report.
  The arms call the kernel and
  the tools directly,
  and no ZFS pool was made for this group.
- What systemd leaves a unit's mounts as.
  The shared and slave starts here are made with `mount` and
  `unshare` by hand,
  not by a unit's `MountFlags` or `PrivateMounts`.
- A read-only parent,
  a hardened unit,
  or way 8 on a ZFS dataset with a real recorded handle.
  The handle
  here was taken on the guest's ext4 root filesystem before a bind covered the directory.
- Any timing,
  liveness or deadline claim.
  The phases were bounded,
  and no bound was reached.
- Two arms' expected rows conflict with the observed ones (T0-4's delivery,
  T0-3 variant (a)),
  and the
  measurement does not decide which the design should keep.
