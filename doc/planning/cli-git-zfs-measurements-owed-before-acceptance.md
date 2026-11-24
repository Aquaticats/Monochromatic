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

## Tier 0 on two kernels and Tier 1 on ZFS, 2026-11-23T18:44:55-05:00

**Measured by an agent.**
Unapproved and non-final.
The heading is the commit time of `5cbdaa6714`,
the commit that wrote this section;
the corrections this section carries later are the commits named with them.
It replaces the Tier 0 section at 2026-11-23T05:24:20-05:00.
Every arm of that group was run again,
the arms the design's review asked for were added,
the whole group ran a second time on this machine's kernel,
and a third pass ran the whole group again on both kernels from one program whose hash every one of its logs
prints.
It decides nothing the design decides and nothing the design leaves open.

The arms are those of `doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md` in
the section it calls Tier 0,
for mounts and namespaces only,
and of its Tier 1,
for the two ZFS arms;
the design was read at commit `b62601213f` on the branch that carries it,
which is not this branch.

### What ran, and where

Two kernels,
one program.

- **The guests**,
  two of them,
  one per pass that ran a guest:
  `zfs-tier1-handle-and-68-reruns` for the second pass and
  `zfs-probe-third-round-handle-and-quota` for the third,
  each made through `mvm --backend libvirt create` from the cached
  template `template-ubuntu.qcow2`,
  nothing downloaded,
  and each destroyed through `mvm` when its pass ended.
  Both report kernel `6.8.0-147-generic` and `Ubuntu 24.04.5 LTS`,
  with `mount`,
  `unshare` and `findmnt` from util-linux `2.39.3`,
  and the root filesystem is ext4 on `/dev/vda1` (`dev=253:1`).
  Each holds a file-backed pool of its own for Tier 1:
  a 2 GiB sparse file,
  ZFS `2.2.2-0ubuntu9.5`,
  module and userspace the same,
  from the distribution's packages.
  Tier 0's arms ran inside fresh private mount namespaces in each,
  as before.
- **The container**,
  a rootless `podman` container from the repository's CachyOS base image
  `docker.io/cachyos/cachyos-v4:latest`,
  which was already in the local store.
  It runs on this machine's own kernel,
  `7.3.5-2-cachyos`,
  with `CachyOS` rolling,
  `mount`,
  `unshare` and `findmnt` from util-linux `2.42.4`,
  and Python `3.14`.
  Its `/probe/work` is the round folder's scratch directory,
  bind-mounted in,
  and that directory sits on this machine's ZFS dataset `zpcachyos/ROOT/cos/home` (`dev=0:57`),
  with the host's module `2.4.4-1` behind it;
  the container's own `/` is a `fuse-overlayfs` mount,
  which is rootless podman's overlay.
  So the Tier 0 rows in the container are rows over ZFS on 7.3,
  and the handle rows there are ZFS handles.
- `mvm`'s registry,
  read before choosing the container,
  holds four images:
  alpine 3.23,
  fedora 43,
  ubuntu noble and windows server 2025.
  The newest kernel among them is fedora 43's `6.17.1-300.fc43.x86_64`,
  which the first round tried,
  and the local image store held `template-alpine.qcow2`,
  `template-fedora.qcow2`,
  `template-ubuntu.qcow2` and their two base images.
  No image with a kernel of 6.19 or newer is cached or in the registry,
  so no guest could run `listns` without a download this round did not make,
  and the container was used for this machine's kernel instead.
- The exact container invocation,
  one per arm:

```text
podman run --rm --name tier0-<arm>-kernel73 --network none --cpus 4 --memory 4g --pids-limit 512 \
  --cap-add sys_admin --cap-add dac_read_search --security-opt seccomp=unconfined \
  --hostname tier0-kernel73 --env TIER0_BASE=/probe/work \
  --volume <round>/scratch/tier0-container/work:/probe/work:Z \
  --volume <round>/keep:/probe/keep:ro,Z \
  docker.io/cachyos/cachyos-v4:latest python3 -u /probe/keep/guest-probe-arms.py tier0-kernel73 <arm> 2
```

No home directory,
no other volume,
no network.
`sys_admin`:
the arms call `unshare`,
`mount`,
`umount2`,
`move_mount` and `setns`,
which inside the container's own user namespace need it;
without it `unshare(CLONE_NEWNS)` answered `EPERM`.
`dac_read_search`:
way 8's `open_by_handle_at` takes that capability in the caller's user namespace on 7.3.
`seccomp=unconfined`:
see the next subsection;
the default policy would have hidden a syscall the kernel has.
Every mount,
bind,
move and unmount happened inside the container's own user and mount namespaces,
and nothing mounted on the host.

Three program files,
each named for the logs it produced,
kept with every log in `~/temp/agent/zfs-forced-removal-first-probe-arms-in-a-guest-2026-11-23/keep/`:

- `probe-guest-round2.py`,
sha256 `e390d490a7eb…15d1e0`:
  it produced the second pass's guest logs (`arm-*-round2-zfs-tier1-handle-and-68-reruns.log`),
  which are the rows quoted below.
- `probe-container-round2.py`,
sha256 `58f8264e1016…1c285`:
  it produced the second pass's container logs (`arm-*-kernel73-tier0-kernel73.log`).
  It was an intermediate copy of the same program without the two Tier 1 arms and without T0-10's guard
  around `NS_GET_ID`;
  the arms that ran in it are otherwise the same code.
- `probe-round3.py`,
sha256 `4291233419d8…81de4`:
  it produced the third pass's logs on both kernels (`arm-*-round3-zfs-probe-third-round-handle-and-quota.log`,
  `arm-*-kernel73-round3-tier0-kernel73.log`).
  Its first printed line in every log names the file and its hash,
  so each of those logs names its own producer:

```text
# arm-T0-1-round3-zfs-probe-third-round-handle-and-quota.log, and every other third-pass log
# program probe-round3.py sha256 4291233419d80649a69ad678d6b0b2cec32c3655865988e8efba7df508a81de4
```

The first pass's program,
which produced `arm-*-final-*.log`,
is gone:
an edit in the second pass overwrote it,
and no copy of it was kept,
so those logs' producer cannot be shown,
only its logs.
That is the one provenance gap this section cannot close;
every other log names a file that is kept.
`run-arm-guest.ts` and `run-all-arms-guest.ts` drive the guest through `mvm --backend libvirt exec`;
`run-arm-container.ts` and `run-all-arms-container.ts` drive the container through `podman`.
One arm per invocation,
every arm's phases twice,
every phase in a fresh private mount namespace H (`unshare(CLONE_NEWNS)`,
then `mount(NULL, "/", NULL, MS_REC|MS_PRIVATE, NULL)`).

The second pass's logs are `arm-<arm>-round2-zfs-tier1-handle-and-68-reruns.log` (the guest,
6.8) and
`arm-<arm>-kernel73-tier0-kernel73.log` (the container,
7.3);
the third pass's are `arm-<arm>-round3-zfs-probe-third-round-handle-and-quota.log` and
`arm-<arm>-kernel73-round3-tier0-kernel73.log`.
The drivers' own lines are in `guest-tier0-round2-driver.log`,
`container-driver-kernel73.log`,
`guest-round3-driver.log` and `container-driver-round3.log`,
and `container-confinement-round3.log` records what the container's confinement is,
read from the files that
hold it (`uid_map`,
`/proc/self/status`,
the root's mount row and `/sys/module/zfs/version`),
by the kept
`confinement-round3.py`;
the second pass's log of the same name was made by a script that is not kept and
mis-parsed two values,
so this section cites only the third pass's.

Two kept checkers produce the agreement this section reports,
and both were run on the third pass's logs:

- `check-runs-agree.py` compares run 1 with run 2 of every arm,
at two strengths.
  Under its weak mask,
  which hides only the run numbers,
  pids,
  durations,
  handle bytes,
  descriptor numbers
  and socket inodes,
  23 of 34 logs are identical;
  the eleven that differ do so in mount ids the kernel
  reused,
  socket inodes of files made in each run,
  and ZFS's own `used` figures.
  Under its strong mask,
  which also hides mount ids,
  parent ids,
  group numbers,
  devices and inodes while
  keeping every tag's shape,
  33 of 34 are identical;
  the one that differs is `Z1-2`,
  in a `zpool list`
  line's `used` figure,
  which is a report of the pool's state and not an arm's row.
- `check-kernels.py` compares each arm's guest log with its container log over every line,
table rows
  included,
  as a set.
  Ten of the sixteen Tier 0 arms are identical on both kernels:
  T0-1,
  T0-2,
  T0-3,
  T0-4,
  T0-6,
  T0-7,
  T0-9,
  T0-11,
  T0-12 and T0-13.
  The six that differ differ in one of three things and nothing else:
  the attempt on the file system's own root (T0-5a,
  T0-5a-literal,
  T0-5b,
  T0-5c open it and get `EINVAL`
  from the unmount on 6.8,
  and get `ESTALE` from the container's `fuse-overlayfs` root on 7.3),
  the copy's device number (T0-8,
  `0:51` against `0:102`),
  and the presence of `listns` and `NS_GET_ID`
  (T0-10).
  T0-4's rows are the same lines in a different order on the two kernels,
  a difference in the order `/proc/self/mountinfo` lists two sibling mounts and not in any tag or relation.
  Z1-1 and Z1-2 ran only in the guest,
  so they are not in that comparison.

### How the container's confinement could change a row

Each of these was checked,
and the last two changed a result before they were dealt with.

- **User namespace.**
  `uid_map` inside reads `0 1000 1` and `1 100000 65536`,
  so the container's root is this
  account and its capabilities are the ones the user namespace grants.
  Every mount-family call ran inside it;
  none reached the host,
  and none needed to.
- **Capabilities.**
  `CapEff` is `00000000802405ff`:
  the podman default set plus `sys_admin` and
  `dac_read_search`.
  Without `sys_admin`,
  `unshare(CLONE_NEWNS)` and `mount` answered `EPERM`.
- **`open_by_handle_at` and its flags.**
  With `O_PATH` alone it answers `EPERM` on 7.3;
  with
  `O_PATH|O_DIRECTORY` it opens the same handle.
  `fs/fhandle.c` at `v7.3` says why:
  `may_decode_fh` lets a caller without `CAP_DAC_READ_SEARCH` in the initial user namespace through only when
  the flags carry `O_DIRECTORY`,
  then asks for that capability in the caller's own user namespace,
  which the container has.
  Version 6.8's older model asks for the initial namespace's capability outright,
  which root in the guest has.
  The same call,
  with `O_PATH|O_DIRECTORY`,
  runs on both kernels;
  the flags mattered only on 7.3.
  That is a property of the container's confinement and not of the handler:
  `may_decode_fh` returns success before it tests the flag for a caller with `CAP_DAC_READ_SEARCH` in the
  initial user namespace,
  which the handler is and the container is not.
  Passing `O_DIRECTORY` on a directory handle is harmless,
  so the design may keep it,
  but the handler does not depend on it and no measurement here shows that it does.
- **seccomp.**
  Podman's default profile answers `ENOSYS` for every syscall it does not list.
  In the first container run the arm read `listns` as absent for that reason;
  with `--security-opt seccomp=unconfined` the same call answers `EFAULT`,
  which is the kernel's own refusal for a null request.
  A run with the default profile would have recorded a present syscall as missing.
- **AppArmor.**
  `/proc/self/attr/current` inside the container reads `unconfined`,
  so the host's profiles do not
  mediate the arms,
  and the rows are the kernel's,
  not a confinement's.
- **Locked mounts.**
  `mount(NULL, "/", NULL, MS_REC|MS_PRIVATE, NULL)` and every later bind and tmpfs mount
  succeeded inside the container,
  so no locked mount refused a step of the common start.
- **Mount ids are reused.**
  The kernel hands out freed mount ids again,
  so a mount made later can carry a lower
  id than its parent;
  the first container pass read a tmpfs over a copy as `id=1110 parent=1235`.
  The program now takes the topmost mount at a path as the one no other row at that path has as its parent,
  a chain rule rather than the greatest id.
  The guest's ids never reused within a phase,
  so its rows are unchanged by that fix;
  the container's are,
  and the arms were re-run after it.

### The common start and its checks

H is one fresh private mount namespace per phase.
The common start makes P in H:
`mount --bind <dir> P`,
`mount --make-private P`,
`mount --make-shared P`,
and it checks that P carries a `shared:` group no other row of H carries (56 checked phases per kernel).
"P" is that mount,
"the copy" a bind of `$B/copy` onto `$B/parent/slot`,
"the slot" that path,
and U,
S and V the namespaces an arm makes.
In the quoted rows `$B` stands for the per-run fixture base (`/var/lib/zfs-tier0-probe/<arm>/run-<n>-<phase>`
in the guest,
`/probe/work/<arm>/run-<n>-<phase>` in the container) and `$W` for the container's host-side path of that
base;
the mount ids,
parent ids,
group numbers and devices are the kernel's own for that run.

Which arms assert the relation their rows assume,
and which only print it:
T0-1,
T0-2,
T0-3,
T0-4,
T0-6,
T0-7,
T0-11 and T0-13 assert it with a failing check or with a control that must deliver;
T0-5a to T0-5c assert it through their control's delivery;
T0-12 and T0-5a-literal print the relation and the control and do not assert either,
and their actions ran with the control false,
which their own paragraphs say.
Only those two phases run an action after a control that did not fire.

Every phase has its arm's bound as a wall-clock deadline and the parent kills the phase's process group when
it passes.
No phase reached its bound;
the longest were T0-7's,
at 10.11 s in the guest and 10.27 s in the container,
against the 30 s the program gives that arm because the design's own `Bound: T_wait 10 s` is shorter than the
phase (it waits `T_wait` and then does the rest).

### Agreement

Two runs of every arm on both kernels,
and the third pass's whole group again on both kernels.
The kept checkers' summary lines,
run on the third pass's logs:

```text
$ python3 check-runs-agree.py arm-*-round3-*.log arm-T0-*-kernel73-round3-*.log
weak mask: 23 of 34 logs identical between their two runs
strong mask: 33 of 34 logs identical between their two runs
  differ: arm-Z1-2-round3-zfs-probe-third-round-handle-and-quota.log

$ python3 check-kernels.py
identical on both kernels, every line including table rows, compared as a set: 10 of 16 arms
  identical: T0-1, T0-11, T0-12, T0-13, T0-2, T0-3, T0-4, T0-6, T0-7, T0-9
  differing: T0-10, T0-5a, T0-5a-literal, T0-5b, T0-5c, T0-8
```

The weak mask hides only run numbers,
pids,
durations,
handle bytes,
descriptor numbers and socket inodes,
so eleven logs differ there in mount ids the kernel reused,
in the inode of a socket file each run makes,
and in
ZFS's own `used` figures.
The strong mask also hides mount ids,
parent ids,
group numbers,
devices and inodes and keeps every tag's shape;
the one log that still differs is `Z1-2`'s `zpool list` line,
whose `used` figure is the pool's own report.
So every arm's two runs agree on every row's shape and relation,
and 33 of 34 agree even on the kernel's own
numbers.
Between the kernels,
six arms differ and only in three things:
the attempt on the file system's own root (T0-5a and its literal start,
T0-5b,
T0-5c open it and answer
`EINVAL` on the unmount on 6.8,
and get `ESTALE` from the container's `fuse-overlayfs` root on 7.3,
a fuse mount being unable to resolve another file system's handle),
the copy's device number (T0-8,
`0:51` against `0:102`),
and the presence of `listns` and `NS_GET_ID` (T0-10).
T0-4's rows are the same lines in a different order on the two kernels,
which is `/proc/self/mountinfo`'s order for two sibling mounts and not a tag or a relation.
No mount row,
no sharing or master relation,
no delivery and no unmount result differs between the kernels.

The third pass against the second,
from the kept `check-rounds.py`:
ten of the eighteen guest arms reproduce line for line,
table rows included
(T0-1,
T0-2,
T0-3,
T0-4,
T0-6,
T0-7,
T0-9,
T0-11,
T0-12 and Z1-1),
and the eight that differ do so only in what this round changed on purpose or in a number each run makes:
the handle route's printed label for the top row (T0-5a,
T0-5a-literal,
T0-5b and T0-5c),
T0-10's new lines and its id-to-descriptor step,
the shapes T0-13 gained,
T0-8's tmpfs device number,
and Z1-2's `zpool` used figures.

### T0-1. A plain unmount reaches the parent's receivers

Expected:
S's copy vanishes,
and the host bind and P stay in H.
Control:
`umount2` of a host bind at a second path under P removed its row and the check passed.

```text
# arm-T0-1-round2-zfs-tier1-handle-and-68-reruns.log, run 1, phase plain-unmount-in-U
    id=428 parent=400 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[shared:326 master:324]
    id=430 parent=428 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[shared:327]
    id=460 parent=432 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:326]
    id=462 parent=460 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[master:327]
# action: umount2(the copy) in U, plain
# observed: U slot gone = True ; S slot gone = True
# observed: H still holds the parent = True ; H still holds the host bind = True
```

```text
# arm-T0-1-kernel73-tier0-kernel73.log, run 1, phase plain-unmount-in-U
    id=1058 parent=1057 dev=0:57 root=$W/T0-1/run-1-plain-unmount-in-U/src mp=$B/parent fstype=zfs source=zpcachyos/ROOT/cos/home tags=[shared:748 master:746]
    id=1089 parent=1058 dev=0:57 root=$W/T0-1/run-1-plain-unmount-in-U/copy mp=$B/parent/slot fstype=zfs source=zpcachyos/ROOT/cos/home tags=[shared:749]
# observed: U slot gone = True ; S slot gone = True
# observed: H still holds the parent = True ; H still holds the host bind = True
```

The expectation holds on both kernels,
and every run agrees.
Decisions needed to script it:
the control unmounts the very host bind whose survival the expectation names,
so it is its own phase on the same constructed start and the action phase mounts the bind again;
the row at the slot is recognised as the copy by its `root=` ending in `/copy`.

### T0-2. A private parent before the unmount stops it

Expected:
S's copy stays mounted.
Control:
T0-1's plain unmount on the same start removed the copy in U and in S.

```text
# arm-T0-2-round2-zfs-tier1-handle-and-68-reruns.log, run 1, phase action-private-parent
    U parent row after the change: id=428 parent=400 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[]
    id=460 parent=432 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:324]
    id=462 parent=460 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[master:327]
# observed: U slot gone = True ; S slot gone = False
```

```text
# arm-T0-2-kernel73-tier0-kernel73.log, run 1, phase action-private-parent
    U parent row after the change: id=1058 parent=1057 dev=0:57 root=$W/T0-2/run-1-action-private-parent/src mp=$B/parent fstype=zfs source=zpcachyos/ROOT/cos/home tags=[]
# observed: U slot gone = True ; S slot gone = False
```

The expectation holds on both kernels,
and every run agrees.
Also observed on both:
the private change re-points S's parent row from `master:<U's former group>` to `master:<H's group>`,
so U's slaves are handed to the master U's parent had,
and S's copy row loses its tag.
Decisions needed to script it:
the change is `mount(NULL, P, NULL, MS_PRIVATE, NULL)`,
and the control is its own phase because it destroys the action's state.

### T0-3. A slave parent before the unmount stops it

The review's G2:
variant (a) as first scripted made no change in U,
so its action was T0-1's and the row the arm names was never produced.
This round's variant (a) makes the arm's own change,
`mount(NULL, P, NULL, MS_SLAVE, NULL)` in U,
before the unmount.

Expected:
S's copy stays in both variants;
variant (a)'s P row reads `master:<P's group in H>` with no `shared:`;
variant (b)'s reads as private with no `master:`.

```text
# arm-T0-3-round2-zfs-tier1-handle-and-68-reruns.log, run 1, phase variant-a
    variant (a) U parent row after the slave change: id=427 parent=399 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:324]
    variant (a) S parent row after the slave change: id=458 parent=430 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:324]
# observed: the slave change made U's parent share the former group's master = {"master_before": 324, "shared_before": [325], "master_after": 324, "shared_after": []}
# observed: U slot gone = True ; S slot gone = False
# observed: S slot row after the action = id=459 parent=458 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[]
```

```text
# arm-T0-3-kernel73-tier0-kernel73.log, run 1, phase variant-a
    variant (a) U parent row after the slave change: id=1057 parent=1056 dev=0:57 root=$W/T0-3/run-1-variant-a/src mp=$B/parent fstype=zfs source=zpcachyos/ROOT/cos/home tags=[master:746]
# observed: the slave change made U's parent share the former group's master = {"master_before": 746, "shared_before": [747], "master_after": 746, "shared_after": []}
# observed: U slot gone = True ; S slot gone = False
```

Variant (a) now shows what it names:
the slave change takes U's parent out of its peer group,
its `master:` becomes the former group,
and the unmount in U leaves S's copy mounted,
which T0-1's plain unmount on the same start does not.
Variant (b) is unchanged:
the parent made private and then slave carries no tag,
S's copy stays,
and the plain unmount on that same start also leaves it (`U slot gone = True ; S slot gone = False` in its
control phase),
so variant (b)'s "stays" belongs to the state rather than to the change.
Both kernels agree,
and every run agrees.
Decisions needed to script it:
variant (b) keeps its own control phase because the control the arm names runs on a different start;
the copy in variant (b) is mounted after the two changes,
as its start reads.

### T0-4. The peer case's rows

Expected,
derived in the design from `fs/pnode.c` and not run when it was written:
P's row becomes `master:<the former group>` without `shared:`;
P' keeps its `shared:` and its own `master:`;
S's row still reads `master:<the former group>`;
after the change a mount made under P' no longer reaches P or S,
and one under P reaches neither.
Control:
a mount under P' reached P and S before the change.

```text
# arm-T0-4-round2-zfs-tier1-handle-and-68-reruns.log, run 1
    P row after the change:  id=397 parent=354 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:324]
    P' row after the change: id=398 parent=354 dev=253:1 root=$B/src mp=$B/peer fstype=ext4 source=/dev/vda1 tags=[shared:324]
    S row after the change:  id=428 parent=400 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:324]
# control delivery under P': {"H/P": true, "H/P'": true, "S/P": true}
# delivery under P' after the change: {"H/P": true, "H/P'": true, "S/P": true}
# delivery under P after the change: {"H/P": true, "H/P'": false, "S/P": false}
```

The rows the arm derives are observed exactly,
on both kernels,
and the delivery under P matches.
The delivery claim differs,
as it did in the first round and as the review said:
the design's expected sentence says a mount under P' no longer reaches P or S,
and it reaches both,
because P has become a slave of P' and S receives through P.
The design's own text for way 7 says the service keeps receiving through the peer,
so it is that sentence of the arm,
not the way,
that is wrong;
the rows show the mechanism (`P ... tags=[master:324]` and P' carrying `shared:324`).
Both kernels agree,
and every run agrees.
Decisions needed to script it:
the copy is mounted in H under P,
because the checks are about delivery into P,
P' and S and the text names no namespace for it;
S is made from H with `unshare --mount --propagation slave` before the copy is mounted.

### T0-5. The receiver closure and the handle unmount

The oracle is a closure over the tables:
the seed is the acting parent's own `shared:` group when it is shared,
or the group its slaves print as their `master:`;
a namespace is a member when its parent row's `shared:` or `master:` is in the set;
a member adds its own `shared:` group;
and only members whose topmost mount at the slot is the copy are kept.

#### T0-5a. The nested-in-nested receiver

Expected:
the closure finds V,
and the set of namespaces whose copy vanished equals it.
Control:
a mount under P in U reached V and was withdrawn.

```text
# arm-T0-5a-round2-zfs-tier1-handle-and-68-reruns.log, run 1
# name_to_handle_at($B/parent/slot): handle_bytes=8 handle_type=1 mount_id=397 handle=2d050400c4be0d75
# relation: U parent row = id=427 parent=399 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[shared:325 master:324]
# relation: V parent row = id=458 parent=430 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:325]
# control delivery to V: true
# oracle: the closure is ['U', 'V'], members with a mount at the slot are ['U', 'V'], the kept receivers are ['U', 'V']
# handle result on the parent mount: {"opened": true, "resolved": "$B/parent/slot", "row_count": 1, "top": "id=428 parent=427 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[shared:326]", "top_id": 428, "top_fstype": "ext4", "top_source": "/dev/vda1", "umount": {"ok": true}}
# observed: namespaces whose slot mount vanished = ["U", "V"]
# T0-5a: closure ['U', 'V'], kept ['U', 'V'], vanished ['U', 'V'], equal = True
```

```text
# arm-T0-5a-kernel73-tier0-kernel73.log, run 1
# name_to_handle_at($B/parent/slot): handle_bytes=12 handle_type=1 mount_id=1013 handle=0a00232ec9000000e9350200
# observed: namespaces whose slot mount vanished = ["U", "V"]
# T0-5a: closure ['U', 'V'], kept ['U', 'V'], vanished ['U', 'V'], equal = True
```

The expectation holds on both kernels and every run agrees,
so way 8's unmount is shown on 7.3 as well as on 6.8,
over ZFS in the container and over ext4 in the guest.
Three things the design's text does not say are measured:
the handle must be opened on the mount that carries the slot (on 6.8 the attempt on the filesystem root opens
but names the covered directory's path there and no mount,
and `umount2` answers `EINVAL`;
on 7.3 the `fuse-overlayfs` root answers `ESTALE` for a ZFS handle);
the open call must carry `O_DIRECTORY` on 7.3 for a caller that is not root in the initial user namespace,
as the container is and the handler is not (see the confinement's flags bullet);
`open_by_handle_at` refuses an `O_PATH` directory descriptor with `EBADF` on both kernels,
a descriptor's path-only flag and not the open flags.
The top row under the descriptor is chosen by the chain rule,
as it is everywhere else in the program:
the row at the resolved path that no other row there has as its parent.
The second pass's handle route still took the greatest mount id there,
which the kernel's reuse of ids makes wrong in principle;
every handle result in every pass had zero or one row at the dentry,
so nothing was decided by it,
and the third pass's program takes the chain rule.
Decisions needed to script it:
U's parent is made shared in U,
as T0-1's start does,
because the closure cannot contain V otherwise;
the arm's literal start is T0-5a-literal;
both descriptor attempts are logged.

#### T0-5a-literal. The start the text spells out

```text
# arm-T0-5a-literal-round2-zfs-tier1-handle-and-68-reruns.log, run 1
# control delivery to V: false
# the control did not deliver on this start, so this run shows nothing by seeing nothing
# oracle: the closure is ['H', 'U', 'V'], members with a mount at the slot are ['U', 'V'], the kept receivers are ['U', 'V']
# observed: namespaces whose slot mount vanished = ["U"]
# T0-5a: closure ['H', 'U', 'V'], kept ['U', 'V'], vanished ['U'], equal = False
```

Both kernels and every run agree.
The arm's expectation holds only on a start whose acting namespace carries a shared group of its own,
and the design's text for T0-5a does not ask for one.

#### T0-5b. A receiver with no mount at the slot

Expected:
V is not a kept receiver,
is not refused,
and is not in the vanished set.

```text
# arm-T0-5b-round2-zfs-tier1-handle-and-68-reruns.log, run 1
# control delivery to V: true
# oracle: the closure is ['U', 'V'], members with a mount at the slot are ['U'], the kept receivers are ['U']
# T0-5b: closure ['U', 'V'], kept ['U'], vanished ['U'], V kept = False
```

The expectation holds on both kernels and every run agrees.
The control the arm names,
T0-5a's V keeping its copy and appearing in the vanished set,
is T0-5a's phase in this group;
this phase's own control shows a mount under U's parent reaching V and being withdrawn.
Decisions needed to script it:
V is made from U after the copy exists,
and its copy is removed in V by a local `umount2` as the text says.

#### T0-5c. A stacked mount at the slot

Expected:
the closure's predicate says way 8 is not chosen for V,
because V's topmost mount at the slot is not the copy,
and the stacked mount stays.

```text
# arm-T0-5c-round2-zfs-tier1-handle-and-68-reruns.log, run 1
# observed: V topmost at the slot = id=460 parent=459 dev=0:50 root=/ mp=$B/parent/slot fstype=tmpfs source=tmpfs tags=[]
# observed: U topmost at the slot = id=428 parent=427 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[shared:326]
# predicate: way 8 is not chosen for a namespace whose topmost mount at the slot is not the copy: {"U": true, "V": false, "H": false}
# oracle: the closure is ['U', 'V'], members with a mount at the slot are ['U', 'V'], the kept receivers are ['U']
# observed: namespaces whose slot mount vanished = ["U"]
    id=458 parent=430 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:325]
    id=460 parent=458 dev=0:50 root=/ mp=$B/parent/slot fstype=tmpfs source=tmpfs tags=[]
```

```text
# arm-T0-5c-kernel73-tier0-kernel73.log, run 1
# observed: V topmost at the slot = id=1126 parent=1121 dev=0:113 root=/ mp=$B/parent/slot fstype=tmpfs source=tmpfs tags=[]
# predicate: way 8 is not chosen for a namespace whose topmost mount at the slot is not the copy: {"U": true, "V": false, "H": false}
# oracle: the closure is ['U', 'V'], members with a mount at the slot are ['U', 'V'], the kept receivers are ['U']
```

The predicate holds on both kernels and the stacked tmpfs stayed mounted in V.
The kernel's own behaviour goes further than the predicate:
the handle unmount in U removed V's copy underneath the stack,
on both kernels,
leaving V's parent and the tmpfs and no copy row.
The arm's log also carries the answer the design's bullet leaves open,
and this round states both rather than one:
the oracle's kept set drops V (`kept ['U']`) while the kernel still removed V's copy.
Which of the two the preflight's policy follows is the design's to say;
the measurement gives both.
Both kernels agree and every run agrees.
Decisions needed to script it:
the predicate is computed per namespace with a mount at the slot,
and the arm's one action is performed anyway,
so the log records the predicate and what the kernel did.

### T0-6. The shared-state difference

The review's G1:
the first round's way 3 phase mounted the second copy after the change,
so way 3's U never held it and the unmount half showed nothing.
This round mounts the second copy in H before the change and reads U's and S's tables after H's unmount,
for both ways.
Control:
before the change,
a mount and an unmount in H under P were seen by U and S.

```text
# arm-T0-6-round2-zfs-tier1-handle-and-68-reruns.log, run 1
# control delivery to U and S: {"U": true, "S": true}
# control withdrawal in U and S: {"U": true, "S": true}
# before way 7: H's mount reaches U and S = {"U": true, "S": true}
    U parent row after way 7: id=427 parent=399 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:324]
# after way 7: H's unmount of the second copy reached U and S = {"U": true, "S": true}
# after way 7: H's later mount of a copy at a new path reaches U and S = {"U": true, "S": true}
# before way 3: H's mount reaches U and S = {"U": true, "S": true}
    U parent row after way 3: id=427 parent=399 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[]
# after way 3: H's unmount of the second copy reached U and S = {"U": false, "S": true}
# after way 3: H's later mount of a copy at a new path reaches U and S = {"U": false, "S": true}
# summary for way 3: before the change the mount reached U and S {"U": true, "S": true}; after it the unmount reached {"U": false, "S": true} and a later mount at a new path reached {"U": false, "S": true}
```

The container's rows are the same line for line after normalising,
in both ways and both runs.
So both halves are now shown:
under way 7 the service keeps receiving H's mounts and H's unmounts under P;
under way 3 it keeps neither,
which is the kernel rule the design's text states and the first round could only assume.
Also observed on both kernels:
way 3's S still received both,
because S's parent is a slave of H's group itself and not of U's parent.
Decisions needed to script it:
the second copy is a bind at `$B/parent/second-copy` mounted before the change;
the later mount uses a third path,
because a mount left in U from before the change would answer for a new mount at the same path;
way 7 and way 3 are two phases from the same start.

### T0-7. The move sits after the signals

Expected:
with the move after the signals the write lands in the mounted copy and the slot's folder is empty when the
move is due;
with the move before the signals it lands in the slot's underlying folder,
which H sees.
Control:
the move before the signals.

```text
# arm-T0-7-round2-zfs-tier1-handle-and-68-reruns.log, run 1
# the control reads the write 10 s after the signal (the write appeared after 0.05 s)
# control observed after 0.05 s: the write is in the slot's underlying folder = True, in the copy = False
# the move is due 10 s after the signal; the write appeared after 0.05 s
# observed: the slot's folder is empty of the write when the move is due = true
# move: {"parent_row": "id=427 parent=399 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:324]", "destination": "$B/parent/.git-worktree-clone-removal-probe", "destination_row": "id=428 parent=427 dev=253:1 root=$B/copy mp=$B/parent/.git-worktree-clone-removal-probe fstype=ext4 source=/dev/vda1 tags=[shared:326]", "slot_row_after": null}
# observed: H sees the new directory on the parent mount as a directory = true, mode 0o700
# observed: H sees a mount at that path = false (the move happened in U only)
```

The expectation holds on both kernels and every run agrees.
Decisions needed to script it:
the blocker traps `SIGTERM`,
writes by absolute path at the slot and exits,
and its write appears about 0.05 s after the signal,
while the program still waits the whole `T_wait` before the move;
the destination and the move follow way 7's steps 2 and 3;
the phase's bound is 30 s,
for the reason given under the common start.

### T0-8. A socket in another network namespace

Expected:
the bound file's device and inode name the copy's filesystem,
the owning task is found by the socket's inode,
and the control's socket outside the copy returns a device that does not match.
Control:
the same request for a socket bound outside the copy.

```text
# arm-T0-8-round2-zfs-tier1-handle-and-68-reruns.log, run 1
# the copy is tmpfs on tmpfs, dev 0:50
# inside: socket ino 21597, UDIAG_VFS ino 3 dev 50 against the file's ino 3, dev 50 (as the kernel encodes it 50): match = True
# outside: socket ino 21598, UDIAG_VFS ino 263733 dev 265289729 against the file's ino 263733, dev 64769 (as the kernel encodes it 265289729): match = True
# the socket inode 21597 is held by tasks found in /proc/<pid>/fd: [4243]
```

The expectation holds on both kernels,
and the only difference between them is the copy's device number.
Decisions needed to script it:
the copy's source is a tmpfs so its device differs from the root filesystem's;
the owner is a child of H that unshares only the network namespace and binds one socket inside the copy and
one outside;
the request is an unfiltered `SOCK_DIAG_BY_FAMILY` dump matched locally by inode;
`setns` runs in a forked child;
the diag device is the raw `dev_t` in the kernel's own encoding,
so a build comparing it with `st_dev` must re-encode it.

### T0-9. Blind and gone reads

```text
# arm-T0-9-round2-zfs-tier1-handle-and-68-reruns.log, run 1
# the reader is nobody; statx on /proc/<pid>/exe answers: {"control: live same-account task": {"ok": true}, "action: root-owned task": {"ok": false, "errno": "EACCES"}, "action: reaped pid": {"ok": false, "errno": "ENOENT"}}
# control: the same reader's statx on a live same-account task's exe succeeded = true
```

Expected and observed on both kernels;
every run agrees.
Decisions needed to script it:
the same-account task is made by fork,
`setuid` to `nobody`,
then `execv("/bin/sleep")`,
because a `runuser` parent stays root;
`statx` runs through libc with flags 0 and mask `0x7ff`.

### T0-10. The taskless namespace

On the guest's 6.8 the arm is unrun and nothing is passed:
syscall 470 answers `ENOSYS` and the `NS_GET_ID` ioctl answers `ENOTTY`.
On the container's 7.3 the kernel has both,
the program tries 470 only,
and the arm ran:

```text
# arm-T0-10-kernel73-round3-tier0-kernel73.log, run 1
# program probe-round3.py sha256 4291233419d80649a69ad678d6b0b2cec32c3655865988e8efba7df508a81de4
# struct ns_id_req is 32 bytes: size, spare, ns_id, ns_type, spare2, user_ns_id
# /proc/self/ns/pid equals /proc/1/ns/pid, the container's own init, which any container satisfies = true
# /proc/self/uid_map reads "0 1000 1 1 100000 65536"; the initial user namespace reads "0 0 4294967295", so the caller is in the initial user namespace = false
# /proc/self/status CapEff = 00000000802405ff
# NS_GET_ID on the pinned descriptor: 50559
# NS_GET_ID on this process's own mount namespace: 50558
# listns(470) for CLONE_NEWNS over 2 page(s): 1 id(s), failure null
# the ids listns returned: [50558]
# the pinned namespace's id is in the listed set = false
# this process's own mount namespace id is in the listed set = true
# a one-id page returned 50558; the continuation request carries ns_id=50558
# the walk over every page returned 1 id(s), failure null, the same set as the one-id first page starts = true
# the id 50558 (this process's own id, which listns listed) opened by open_by_handle_at(FD_NSFS_ROOT): mnt:[4026533637]
# setns on the descriptor the listed id gave: entered, 33 rows, the mark present = false
# the id 50559 (the pinned id, which listns did not list) to a descriptor by open_by_handle_at(FD_NSFS_ROOT): ESTALE
# setns on the pinned descriptor entered it; the table has 34 rows
# the mark in the pinned namespace: ["id=1086 parent=1018 dev=0:102 root=/ mp=/mnt/pinned-probe fstype=tmpfs source=tmpfs tags=[]"]
```

```text
# arm-T0-10-round3-zfs-probe-third-round-handle-and-quota.log, run 1
# /proc/self/uid_map reads "0 0 4294967295"; the initial user namespace reads "0 0 4294967295", so the caller is in the initial user namespace = true
# listns(470) for CLONE_NEWNS over 1 page(s): 0 id(s), failure "ENOSYS"
# NS_GET_ID on the pinned descriptor refused: [Errno 25] NS_GET_ID: ENOTTY
# setns on the pinned descriptor entered it; the table has 29 rows
# the mark in the pinned namespace: ["id=483 parent=403 dev=0:51 root=/ mp=/mnt/pinned-probe fstype=tmpfs source=tmpfs tags=[]"]
```

What the `uid_map` line answers,
and what it does not:
`/proc/self/ns/pid` equals `/proc/1/ns/pid` in any container,
because `/proc/1` there is the container's own
init,
so that comparison distinguishes nothing and the log no longer claims it does;
the map does,
reading `0 0 4294967295` only in the initial user namespace.
What 7.3 shows:
the call exists,
its request structure is the 32 bytes the design names,
pagination works from the last id of a page,
`NS_GET_ID` reads a descriptor's id,
and a listed id becomes a namespace descriptor and enters it
(the design's second half,
which the second pass left unscripted):
a handle of `{ns_id, 0, 0}` of type `FILEID_NSFS` opened with `open_by_handle_at(FD_NSFS_ROOT, …)` gave
`mnt:[4026533759]` for the id the caller's own listing held,
and `setns` on it entered that namespace.
The same construction on the pinned id,
which no listing gave this caller,
answers `ESTALE`;
`nsfs`'s `fh_to_dentry` refuses a namespace the caller may not see with `EPERM`,
and `do_handle_to_path` reports every failure of the decode as `ESTALE`
(`fs/fhandle.c` at `v7.3`),
so the errno the call answers names the decode and not the refusal's cause.
What it does not show is the arm's own statement,
that the pinned taskless namespace is found:
the listed set holds exactly one id,
this process's own namespace,
and not the pinned one.
`kernel/nscommon.c` at `v7.3` says why,
and this is a reading of source and not a measurement:
`may_list_ns` lets a caller list only its own namespaces unless `may_see_all_namespaces()`,
which needs the caller to be in the initial pid namespace and to hold `CAP_SYS_ADMIN` in the initial user
namespace;
a rootless container has neither.
The handler the design describes runs as root in the host's init namespaces,
so the arm's route is open to it and closed to this container;
nothing here measures the handler.
The arm as scripted prints the listed ids,
the pinned id and their membership,
and enters the namespace by the pinned descriptor,
which needs no `listns`.
Decisions needed to script it:
the descriptor is handed over with `SCM_RIGHTS` from the child that unshares,
because a descriptor opened after `fork` does not exist in the parent;
syscalls 471 to 473 are gone from the program,
since 471 is `rseq_slice_yield` and 472 is `fchroot` on 7.3 and a stray call would have meant something else.

### T0-11. The chain: a unit started shared, then repaired

The review's G11:
the first round's single delivery probe ran after the repair and the change together,
so the attribution rested on the row alone.
This round probes between them,
and adds a start where H's parent has a peer.
Control:
a mount in H under P reached U before the repair.

```text
# arm-T0-11-round2-zfs-tier1-handle-and-68-reruns.log, run 1, phase chain-and-way-7
    U parent row after the repair: id=427 parent=399 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[shared:325]
    U parent row after the change: id=427 parent=399 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[]
# attribution: delivery before the repair, between the repair and the change, and after both = {"before_repair": true, "between": false, "after_both": false}
```

```text
# arm-T0-11-round2-zfs-tier1-handle-and-68-reruns.log, run 1, phase peer-repair
    H peer row after the repair: id=398 parent=354 dev=253:1 root=$B/src mp=$B/peer fstype=ext4 source=/dev/vda1 tags=[shared:324]
    U parent row after the repair: id=428 parent=400 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[shared:325 master:324]
# delivery to U between the repair and the change = false
    U parent row after the change: id=428 parent=400 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:324]
# attribution on the peer start = {"before_repair": true, "between": false, "after_both": false}
```

The expectation holds on both kernels and every run agrees,
and the attribution is now measured:
the repair alone stops H's mounts under P reaching U,
before any slave change.
With a peer in H,
the repair does not clear U's master:
U's parent keeps its own `shared:` group and reads `master:` as the peer's group,
which is the design's "or keeps its next peer's master (it had peers)" branch;
the way 7 change then reads `tags=[master:<the peer's group>]`.
Both kernels agree;
the container's rows are the same after normalising.

### T0-12. A receiver that holds no copy, after way 7

Expected:
after the act,
the namespace that holds no copy receives nothing of the service's later mounts under P.

```text
# arm-T0-12-round2-zfs-tier1-handle-and-68-reruns.log, run 1
# observed: V holds a mount at the slot before the act = false
# control delivery to V: false
# observed: after the act V receives the later mount under P = false
# observed: V holds a mount at the slot before the act = true
# control delivery to V: true
# observed: the copy is gone in U = true, gone in V = false
# observed: after the act V receives the later mount under P = false
# observed: V holds a mount at the slot before the act = false
# control delivery to V: true
# observed: the copy is gone in U = true, gone in V = true
# observed: after the act V receives the later mount under P = false
```

The three starts are unchanged from the first round and agree with it.
The start that measures the statement is the third,
a receiver that holds no copy;
the first two cannot both keep V without a copy and let the control deliver,
and the second shows the copy arriving in V by propagation.
Both kernels agree,
and every run agrees.
The review's correction to the name is taken:
the case is a sandbox that holds no copy,
not one that never held the clone.

### T0-13. The host's own copy when the holder unmounts

The review's G3:
no arm unmounted a holder's copy while H itself held a mount at the slot,
which is the production shape.
This arm adds it,
and the review's Q9 added the shapes:
four of them,
because the shape a service's mount has decides both what the unmount reaches and what the
preflight's closure computes.

The start is the common one,
with H mounting the copy at the slot so that the holder receives it by propagation.
The four shapes of the holder's parent mount:

- `slave-group`:
a slave with a group of its own (`mount --make-shared` in U after the slave change),
  which is the shape systemd leaves a unit started while the checkout was shared.
- `slave`:
a pure slave,
no group of its own.
- `peer`:
a peer of H's mount,
both ways.
- `unrelated`:
the same directory bound at another path in the holder's own namespace,
  with no propagation relation to H's mount at all,
  the shape a unit started after the repair has.
  It receives nothing from H,
  so it takes a copy of its own,
  and that copy is what it unmounts.

Each shape runs twice:
once unmounting by the recorded handle,
once by the path.
The controls are the copy's arrival in the holder (or,
for `unrelated`,
the holder's own copy being present) and
a second mount in H under P reaching the holder.

```text
# arm-T0-13-round3-zfs-probe-third-round-handle-and-quota.log, run 1, one phase per shape
# program probe-round3.py sha256 4291233419d80649a69ad678d6b0b2cec32c3655865988e8efba7df508a81de4
# relation: U parent row = id=484 parent=404 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[shared:330 master:329]
# positive control: the copy mounted in H reached U = true
# oracle: the closure over H and U = ['U'], kept = ['U']
# the host's own namespace is a member with a mount at the slot = false
# observed: the slave-group shape: U's copy gone = true; H's copy at the slot still mounted = true; H's copy topmost row = id=485 parent=402 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[shared:331]
# relation: U parent row = id=484 parent=404 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[master:329]
# positive control: the copy mounted in H reached U = true
# oracle: the closure over H and U = ['H', 'U'], kept = ['H', 'U']
# the host's own namespace is a member with a mount at the slot = true
# observed: the slave shape: U's copy gone = true; H's copy at the slot still mounted = true; H's copy topmost row = id=485 parent=402 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[shared:330]
# relation: U parent row = id=484 parent=404 dev=253:1 root=$B/src mp=$B/parent fstype=ext4 source=/dev/vda1 tags=[shared:329]
# positive control: the copy mounted in H reached U = true
# oracle: the closure over H and U = ['H', 'U'], kept = ['H', 'U']
# the host's own namespace is a member with a mount at the slot = true
# observed: the peer shape: U's copy gone = true; H's copy at the slot still mounted = false; H's copy topmost row = none
# relation: U parent row = id=485 parent=404 dev=253:1 root=$B/src mp=$B/own-parent fstype=ext4 source=/dev/vda1 tags=[]
# the unrelated holder received nothing by propagation = true
# oracle: the holder's parent is not H's mount, so the closure over H's group does not reach it
# observed: the unrelated shape: U's copy gone = true; H's copy at the slot still mounted = true; H's copy topmost row = id=486 parent=402 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[shared:330]
```

```text
# arm-T0-13-kernel73-round3-tier0-kernel73.log, run 1
# observed: the slave-group shape: U's copy gone = true; H's copy at the slot still mounted = true; H's copy topmost row = id=1089 parent=1016 dev=0:57 root=$W/T0-13/run-1-slave-group-marker/copy mp=$B/parent/slot fstype=zfs source=zpcachyos/ROOT/cos/home tags=[shared:748]
# observed: the slave shape: U's copy gone = true; H's copy at the slot still mounted = true; H's copy topmost row = id=1089 parent=1016 dev=0:57 root=$W/T0-13/run-1-slave-marker/copy mp=$B/parent/slot fstype=zfs source=zpcachyos/ROOT/cos/home tags=[shared:747]
# observed: the peer shape: U's copy gone = true; H's copy at the slot still mounted = false; H's copy topmost row = none
# observed: the unrelated shape: U's copy gone = true; H's copy at the slot still mounted = true; H's copy topmost row = id=1090 parent=1016 dev=0:57 root=$W/T0-13/run-1-unrelated-marker/copy mp=$B/parent/slot fstype=zfs source=zpcachyos/ROOT/cos/home tags=[shared:747]
```

All four shapes answer the design's open question,
on both kernels,
and every run agrees:
where the holder only receives from the host's mount,
whether as a pure slave or as a slave with a group of its
own or with no relation at all,
the holder's unmount removes its own copy and the host's copy stays;
where the holder shares with the host both ways,
the same unmount removes the host's copy as well.
That is the kernel rule the design states for way 4 ("the host's mount stays only where it is a master or no
receiver of the parent"),
now measured rather than expected,
and it is the same for the handle and for the path.

The oracle's line differs by shape,
and that is the measurement the review asked for:
in the `slave-group` shape the closure seeded from the holder's own group is `['U']` and the host's namespace is
not in it,
because the holder's group is its own and the host's group appears only as the holder's `master:`;
in the pure `slave` and the `peer` shapes the closure is `['H', 'U']` and the host is a kept member.
So a preflight that seeds from the acting namespace's own group would not name the host in systemd's own shape,
even though the host's copy is safe there;
and in the shapes where the host is named,
the design's policy already falls from way 8 to way 7.
Which of those two the preflight should seed from is the design's to say,
and this arm measures both.

Decisions needed to script it:
the copy is mounted in H,
not in U,
and the handle is recorded in H before that mount covers the directory;
the plain phases exist because the handle route is a different action and the propagation question is the same
for both;
and the `unrelated` shape binds the same source directory at `$B/own-parent` inside the holder,
so its copy is its own mount and its unmount cannot reach H.

### Tier 1: the handle on ZFS and a dataset at its quota

Pool:
a 2 GiB sparse file inside the throwaway guest,
`zpool create tier1pool /var/lib/zfs-tier1/pool.img`,
with datasets `tier1pool/work`,
`tier1pool/quota` and `tier1pool/origin`;
ZFS `2.2.2-0ubuntu9.5`,
module and userspace the same;
the preparation's own output is `guest-zfs-prepare.log`.
The container's Tier 0 handle rows are the other ZFS data point,
over the host's module `2.4.4-1` on 7.3.

#### Z1-1. The recorded handle on a ZFS dataset

State:
`tier1pool/work/ancestor/slot`,
a handle taken with `name_to_handle_at` before a bind covers it,
then the cover,
then a rename of the ancestor.
Control:
the same handle opened before the cover names the plain directory and no mount sits at its dentry.

```text
# arm-Z1-1-round2-zfs-tier1-handle-and-68-reruns.log, run 1, phase under-a-cover
# name_to_handle_at(/tier1pool/work/ancestor/slot): handle_bytes=12 handle_type=1 mount_id=394 handle=0a000900000000005c010000
# control before the cover: the descriptor names /tier1pool/work/ancestor/slot
# control: no mount sits at its dentry = true
# observed: the topmost mount at the slot by path = id=397 parent=394 dev=0:47 root=/cover-source mp=/tier1pool/work/ancestor/slot fstype=zfs source=tier1pool/work tags=[]
# action: open the same handle on an O_RDONLY descriptor of the dataset mount
# observed: the descriptor names the covered directory = true (/tier1pool/work/ancestor/slot)
# observed: the topmost mount at its dentry is the cover = true
# observed: the cover's marker reads through the descriptor = false
# action: umount2("/proc/self/fd/<n>", 0)
# observed: the umount was accepted
# observed: after the umount the topmost mount at the slot = none
```

```text
# arm-Z1-1-round2-zfs-tier1-handle-and-68-reruns.log, run 1, phase after-an-ancestor-rename
# the rename of the ancestor: /tier1pool/work/ancestor to /tier1pool/work/ancestor-renamed
# after the cover and the rename: the descriptor names /tier1pool/work/ancestor-renamed/slot
# observed: the descriptor names the covered directory = true (/tier1pool/work/ancestor-renamed/slot)
# observed: the topmost mount at its dentry is the cover = true
# observed: the umount was accepted
```

The expectation holds on ZFS 2.2.2,
in both states,
and every run agrees:
the recorded handle names the covered directory under a cover and after an ancestor's rename,
the topmost mount at its dentry is the cover,
and `umount2` by the descriptor removes that cover.
One further row is worth the design's attention:
the cover's marker is not readable through the descriptor,
so the descriptor names a place in the underlying filesystem and traversal stays there while `umount2` takes
the topmost mount at the dentry;
the handle is not a way to read what covers it.
Decisions needed to script it:
the descriptor is taken on the dataset's mount (`/tier1pool/work`) and not on `/`,
because a handle resolves only against the filesystem it was taken from;
the directory is recreated per run so a previous run's rows cannot answer.

#### Z1-2. A dataset at its quota

State:
a dataset at its quota,
and a dataset holding a clone.
Control:
a directory made on the same pool's `work` dataset,
which has room.

```text
# arm-Z1-2-round2-zfs-tier1-handle-and-68-reruns.log, run 1
# control: the directory was made = true
# the fill stopped at 4194304 bytes with errno 122 (EDQUOT): Disk quota exceeded
# statvfs on the quota dataset: blocks 33, free 0, available 0, fragment size 131072
# observed: the mkdir was refused errno 122 (EDQUOT): Disk quota exceeded
# observed: the directory exists after the attempt = false
# observed: the preflight reads of the dataset = {"quota": "4M", "free_bytes": 0, "dir_exists": false}
# observed: the clone's mount row = id=397 parent=393 dev=0:50 root=/ mp=/tier1pool/clone fstype=zfs source=tier1pool/clone tags=[]
# observed: a directory can be made in the clone = true
```

The expectation holds:
a dataset at its quota refuses the directory with `EDQUOT`,
and the reads the preflight would take name it (`available` 0 from `statvfs`,
`quota=4M`,
`used` at the quota).
The clone-holding half is recorded rather than decided:
a clone of a snapshot mounts as its own dataset and takes a directory.
Decisions needed to script it:
the fill is incompressible bytes,
because a compressible filler does not fill a quota on a dataset with compression;
the quota is 4 MiB so the fill is short;
the leftover file and directory are removed at the start of every run.

### What the group shows, arm by arm and kernel by kernel

T0-1:
a plain unmount of the copy inside a holder's namespace removes it in that namespace and in every namespace
that receives from the same parent,
a slave of a slave included,
and the host's own rows stay.
Shown on 6.8 and on 7.3,
over ext4 and over ZFS.
T0-2:
a parent made private before the unmount stops that reach;
shown on both kernels.
T0-3:
a parent made a slave while it has a master stops it too,
and its row then reads `master:<the former group>` with no group of its own;
shown on both kernels,
and variant (b)'s "stays" belongs to a parent that already had no master and no group,
which its own control shows.
T0-4:
the rows a slave change gives a parent with a peer are as the design derived;
a mount under the peer still reaches the parent and its slaves,
which the arm's expected sentence denies and the way's own text allows.
Shown on both kernels.
T0-5a to T0-5c:
the receiver closure,
the kept set and the vanished set agree;
the handle unmount takes the copy and its receivers' copies with no sharing change,
and removes a receiver's copy from under a stacked mount as well;
a receiver with no copy is not refused.
Shown on both kernels,
with a ZFS handle on 7.3 and an ext4 handle on 6.8.
T0-6:
in a shared checkout,
way 7 keeps the service receiving the host's later mounts and unmounts under P,
and way 3 cuts it off from both;
measured on both kernels,
both halves.
T0-7:
the move after the signals keeps a dying job's write inside the clone and leaves the slot's folder empty;
the host sees the short-lived directory and no mount at it.
Shown on both kernels.
T0-8:
a bound unix socket in another network namespace is found with its file's inode and device,
and the owner by `/proc/<pid>/fd`.
Shown on both kernels.
T0-9:
the classifier's two answers are `EACCES` for blind and `ENOENT` for gone,
and a reader that may read one succeeds.
Shown on both kernels.
T0-10:
on 6.8 the call and the id ioctl are absent;
on 7.3 the call exists,
answers with the caller's own namespace only in a rootless container,
paginates,
converts a listed id into a descriptor that `setns` enters,
answers `ESTALE` for an id it may not see,
and the pinned namespace is entered by its descriptor.
The arm's own statement,
that a taskless namespace is found without a task,
is shown only to a caller that holds `CAP_SYS_ADMIN` in the initial user namespace,
which this container is not.
T0-11:
the repair alone stops the flow to a unit started while the checkout was shared,
and way 7 then reads as way 3 does;
a parent that kept a peer keeps that peer as its master.
Shown on both kernels,
with the delivery probe between the repair and the change.
T0-12:
a namespace that holds no copy stops receiving the service's later mounts under P after way 7.
Shown on both
kernels.
T0-13:
where the holder only receives from the host's mount
(a slave with a group of its own,
a pure slave,
or a holder with no relation),
its unmount of the copy leaves the host's copy mounted;
where it shares with the host both ways,
the host's copy goes too.
The preflight's closure names the host in the pure slave and peer shapes and not in systemd's own.
Shown on both kernels,
by the handle and by the path,
in four shapes.
Z1-1:
the recorded handle names the covered directory on ZFS under a cover and after an ancestor's rename,
and `umount2` by the descriptor removes the cover.
Shown on ZFS 2.2.2 in the guest and,
through Tier 0's arms,
on ZFS 2.4.4 in the container.
Z1-2:
a dataset at its quota refuses the directory with `EDQUOT` and the preflight's reads name it.
Shown on ZFS 2.2.2.

What is a reading of source only:
the reasons behind three measured effects,
namely that the preflight's visibility rule for `listns` is `may_see_all_namespaces()` in `kernel/nscommon.c`
at `v7.3`,
that `O_DIRECTORY` is required by `may_decode_fh` in `fs/fhandle.c` at `v7.3`,
and that the propagation rules that produce the 7.3 rows live in `gather_candidates`,
`trim_one` and `reparent` in `fs/pnode.c` at `v7.3`.
The effects are measured on 7.3;
the mechanisms are read,
and no line of that kernel's source was compiled or traced here.

### What the group does not show

- A service the manager started,
  `PrivateMounts=yes`,
  or a unit's own mount flags:
  every namespace here is made by
  `unshare` and `mount` by hand.
- Live blockers,
  `EBUSY`,
  kills,
  or the wait between `SIGTERM` and `SIGKILL` beyond the one blocker T0-7 writes
  through.
- Way 7 end to end:
  no arm unmounts at the destination or removes the short-lived directory.
- A read-only parent,
  a hardened unit,
  or a socket held by a task the caller cannot read.
- A holder in its own user namespace,
  which is what bubblewrap,
  a rootless container or systemd's `PrivateUsers=yes` make.
  Every holder here was in the caller's user namespace,
  so no copied mount was locked.
  This is a reading of `v7.3` and not a run:
  `copy_mnt_ns` calls `lock_mnt_tree` when the new namespace has a user namespace of its own
  (`fs/namespace.c` 4440 to 4441);
  `can_umount` answers `EINVAL` for a locked mount (2088);
  `do_move_mount` refuses a locked source (3786);
  and `propagate_umount`'s `handle_locked` unmounts a locked receiver's copy only when that receiver's parent
  goes too (`fs/pnode.c` 669 to 688).
  If that reading holds on the machine's kernel,
  way 8 would be refused or would leave the copy in such a holder,
  and way 7's move would be refused there too,
  so a locked holder would need its own act or a refusal.
  No arm built one.
- The taskless route as the handler would run it,
  from the initial namespaces with `CAP_SYS_ADMIN` there:
  the container lists only its own namespace and converts only an id it may see (T0-10).
- The handler's own confinement:
  the host's AppArmor enforces since 2026-11-23,
  and inside the container the label read `unconfined`,
  so no profile mediated a row.
  What a profile would do to the handler's `setns`,
  `umount2`,
  `move_mount`,
  `open_by_handle_at` and its reads
  of other tasks' `/proc/<pid>/exe` was not measured,
  and no denial from the host's kernel log was read.
- The design's T0-3 delivery control as the merged text now names it
  ("a mount made in H under P reaches S before the change"):
  the logs have none,
  and they were run from `b62601213f`'s text.
  The plain-unmount control on each variant's start is what they carry.
- The marker's ancestor-rename half on ZFS `2.4.4` and on 7.3:
  Z1-1 ran both halves on ZFS `2.2.2` in the 6.8 guest,
  and T0-5a's handle ran under a cover over ZFS `2.4.4` in the 7.3 container without the rename.
  No cover was a ZFS clone dataset;
  every cover was a bind of another folder.
- A unit's reception from a host peer after the repair:
  T0-11's peer start probes a mount under P,
  which the repair made private,
  so the unit's continued reception from the peer rests on its row's `master:` and not on a probe.
- ZFS `2.4.4` in a guest:
  the guest's pool is `2.2.2`,
  and the `2.4.4` rows are the container's,
  over the host's
  module.
- Any timing claim:
  every phase was bounded and none reached its bound.

### Evidence behind each statement, as the review listed them

- One unmount reaches every receiver,
  with no sharing change:
  T0-1 and T0-5a,
  on 6.8 and on 7.3.
- The handle must be opened on the checkout's mount,
  with an `O_RDONLY` directory descriptor;
  `O_DIRECTORY` is needed on 7.3 only by a caller that is not root in the initial user namespace,
  which the container is and the handler is not:
  T0-5a's two attempts,
  `container-confinement-round3.log`,
  and the flags bullet above.
- A receiver without a copy is not refused:
  T0-5b,
  on both kernels.
- A receiver's own mount on top of its copy:
  T0-5c,
  on both kernels;
  the copy goes and the cover stays.
- Stopping the sharing first does not reach the receivers:
  T0-2,
  T0-3 variant (a) and T0-12's second start.
- A receiver that holds no copy loses the service's later mounts:
  T0-12's third start.
- A shared checkout,
  way 7 against way 3,
  the mount half and the unmount half:
  T0-6,
  both halves now measured.
- A private checkout,
  the two ways are one change:
  T0-11.
- The repair alone stops the flow,
  and a peer keeps its master:
  T0-11's probes and its peer start.
- The short-lived folder and a dying job's late write:
  T0-7.
- The host's own copy stays where the holder only receives from it and goes where the holder shares with it:
  T0-13,
  in four shapes (a slave with its own group,
  a pure slave,
  a peer,
  and a holder with no relation),
  by the handle and by the path,
  on both kernels.
- The closure's seed:
from the holder's own group the host is not a member in systemd's own shape,
and from the
  slaves' master it is:
  T0-13's oracle lines.
- A listed namespace id becomes a descriptor and is entered,
and an id the caller may not see answers `ESTALE`:
  T0-10 on 7.3.
- The handle on ZFS,
  under a cover and after an ancestor's rename:
  Z1-1.
- A dataset at its quota refuses the directory and the reads name it:
  Z1-2.
The statements this round fills no slot for are left visibly empty,
as the review left them:

- `<the unmount at the destination and the folder's removal: way 7 has never been run end to end>`.
- `<a service the manager started with PrivateMounts=yes, and its live blockers>`.
- `<the taskless route as a handler with the init namespaces' CAP_SYS_ADMIN would run it>`.
- `<a holder in its own user namespace, whose copied mounts the kernel locks: a run, not the reading above>`.
- `<the kill cases: a blocker that will not die, EBUSY after the structural stage, and what a kill leaves>`.
- `<a read-only parent, and way 8 where way 7 is refused>`. 

## The three arms that need no step 17 code, 2026-11-23T21:16:46-05:00

**Measured by an agent.**
Unapproved and non-final.
The heading is the commit time of `c3084fbc96`,
the commit that wrote this section.
This section records the three Tier 0 arms the third read listed as unrun or open and that need no step 17
code:
T0-14 in the shape that read's P1 names,
T0-10 as the handler would run it,
and T0-3's delivery control.
It decides nothing the design or the record decides.

### The guest, its kernel and the program

One guest,
`zfs-probe-fourth-round-user-units-and-listns-20261124`,
made through `mvm --backend libvirt create` from the cached template `template-ubuntu.qcow2`,
nothing downloaded to make it,
and destroyed through `mvm` when the round ended.
It is `Ubuntu 24.04.5 LTS`.
It was created with the template's `6.8.0-147-generic`,
and its own Ubuntu archive then offered `linux-image-generic-hwe-24.04` at `7.0.0-43`,
which was installed inside the guest and booted,
so every log in this section is a log on **`7.0.0-43-generic`**,
a kernel of 6.19 or newer whose syscall table has `listns`.
The guest's own pool was made under the older kernel (`tier1pool`,
a 2 GiB sparse file,
ZFS `2.2.2-0ubuntu9.5`) and imported on the new one with the distribution's `zfs-kmod-2.4.1-1ubuntu5.1`,
so the fixture kept its file-backed pool across the change;
none of these three arms needs it.

The program is `probe-round4.py`,
sha256 `02e0a82c6e33d3077d9a627493cdecd90a51ac48a470e7af1c5c5fdfa4108879`,
kept with the logs in `~/temp/agent/zfs-forced-removal-first-probe-arms-in-a-guest-2026-11-23/keep/`,
and every log in this section prints that name and hash as its sixth line,
after the driver's push lines.
Each arm ran twice,
and every log ends `0 failing phase(s)`:

- `arm-T0-14-round4-zfs-probe-fourth-round-user-units-and-listns-20261124.log`
- `arm-T0-10-root-round4-zfs-probe-fourth-round-user-units-and-listns-20261124.log`
- `arm-T0-3-round4-zfs-probe-fourth-round-user-units-and-listns-20261124.log`

### T0-14. A holder that is a systemd unit, and what a user namespace does to its copy

The third read's P1 reads that a sandboxed unit of a person's own user manager is in the class the design
refuses,
because systemd gives such a unit a user namespace of its own,
and that the design's lock read cannot be made from `mountinfo`.
This arm measures the class instead of reading it:
six shapes,
each run twice,
each measuring the same five things over a copy the holder holds.

This arm's H is not a fresh private namespace.
A unit started by systemd is forked from the guest's init,
so a mount made in a private namespace of the probe never reaches it;
the arm therefore mounts in the guest's own root namespace,
before each unit starts,
and unmounts everything before it returns (each phase prints the count of its own rows left behind,
which is zero).
That deviation from the common start is the arm's,
not the program's,
and it is what makes the holder's namespace *copy* the copy rather than receive it by propagation.

Each shape prints,
in this order:
the unit's own `uid_map` and `CapEff`;
whether its user namespace differs from the initial one;
`NS_GET_USERNS` on its mount namespace descriptor and whether that owner is the initial user namespace;
the copy's row in the holder's own table;
what the holder's namespace answers to a move and to an unmount,
entered as root of the initial user namespace in a child per action;
and,
as the control,
the same copy unmounted in this arm's H.

The system unit with the same option,
which the brief names as the control that must differ:

```text
# arm-T0-14-round4-zfs-probe-fourth-round-user-units-and-listns-20261124.log, run 1, phase system-private-mounts
# the unit: system manager, properties ['PrivateMounts=yes']
# /proc/2874/uid_map = 0          0 4294967295
# /proc/2874/status CapEff = 000001ffffffffff
# the initial user namespace's inode = 4026531837; the holder's = 4026531837; they differ = false
# NS_GET_USERNS on the holder's mount namespace = ["user:[4026531837]", 4026531837], failure null
# its mount namespace is owned by the initial user namespace = true
# the holder holds the copy = true: id=464 parent=463 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[shared:382 master:1]
# the move probe = {"action": "move", "the copy in that namespace": "id=464 parent=463 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[shared:382 master:1]", "make-slave of the parent": "accepted", "move_mount of the copy": "accepted", "the destination row": "id=464 parent=463 dev=253:1 root=$B/copy mp=$B/parent/.move-probe fstype=ext4 source=/dev/vda1 tags=[shared:382 master:1]", "the move back": "EINVAL", "the copy at the slot after the action": "none"}
# the umount probe = {"action": "umount", "the copy in that namespace": "none", "the copy the umount probe met at": "$B/parent/.move-probe", "umount2 of the copy": "accepted", "the copy at the slot after the action": "none"}
# control: the same copy unmounted in the guest root namespace was accepted
```

A user-manager unit with `PrivateMounts=yes`,
which is the shape the design's own example has:

```text
# arm-T0-14-round4-zfs-probe-fourth-round-user-units-and-listns-20261124.log, run 1, phase user-private-mounts
# the unit: user manager, properties ['PrivateMounts=yes']
# /proc/2898/uid_map = 1000       1000          1
# the initial user namespace's inode = 4026531837; the holder's = 4026532395; they differ = true
# NS_GET_USERNS on the holder's mount namespace = ["user:[4026531837]", 4026531837], failure null
# its mount namespace is owned by the initial user namespace = true
# the holder holds the copy = true: id=113 parent=105 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[shared:1]
# the move probe = {"action": "move", "the copy in that namespace": "id=113 parent=105 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[shared:1]", "make-slave of the parent": "accepted", "move_mount of the copy": "accepted", "the destination row": "id=113 parent=105 dev=253:1 root=$B/copy mp=$B/parent/.move-probe fstype=ext4 source=/dev/vda1 tags=[shared:1]", "the move back": "EINVAL", "the copy at the slot after the action": "none"}
# the umount probe = {"action": "umount", "the copy in that namespace": "none", "the copy the umount probe met at": "$B/parent/.move-probe", "umount2 of the copy": "accepted", "the copy at the slot after the action": "none"}
# control: the same copy unmounted in the guest root namespace was accepted
```

The three other user-manager shapes behave the same,
in both runs:
`PrivateMounts=yes` with `PrivateUsers=yes`,
`ProtectHome=read-only`,
and `PrivateTmp=yes` each give the process its own user namespace (`uid_map` reads `1000 1000 1`,
the holder's user namespace inode differs from the initial one) and leave the *mount* namespace owned by the
initial one (`NS_GET_USERNS` reads `user:[4026531837]`),
so the move and the unmount are both accepted.

The kernel's locked class is reached by a holder whose mount and user namespaces are made in one clone,
which is what bubblewrap and a rootless container do:

```text
# arm-T0-14-round4-zfs-probe-fourth-round-user-units-and-listns-20261124.log, run 1, phase handmade-user-namespace
# the holder: unshare --user --map-root-user --mount, then sleep
# the holder's main process: 3056; waited 0.05 s for its own user namespace
# /proc/3056/uid_map = 0          0          1
# the initial user namespace's inode = 4026531837; the holder's = 4026532392; they differ = true
# NS_GET_USERNS on the holder's mount namespace = ["user:[4026532392]", 4026532392], failure null
# its mount namespace is owned by the initial user namespace = false
# the holder holds the copy = true: id=433 parent=432 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[]
# the move probe = {"action": "move", "the copy in that namespace": "id=433 parent=432 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[]", "make-slave of the parent": "accepted", "move_mount of the copy": "EINVAL", "the copy at the slot after the action": "id=433 parent=432 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[]"}
# the umount probe = {"action": "umount", "the copy in that namespace": "id=433 parent=432 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[]", "the copy the umount probe met at": "$B/parent/slot", "umount2 of the copy": "EINVAL", "the copy at the slot after the action": "id=433 parent=432 dev=253:1 root=$B/copy mp=$B/parent/slot fstype=ext4 source=/dev/vda1 tags=[]"}
# control: the same copy unmounted in the guest root namespace was accepted
```

What the six shapes show,
in both runs:

- The class the design refuses is not "a unit with `PrivateMounts=yes`":
  it is **a mount namespace owned by a user namespace other than the initial one**,
  and only the hand-made holder has one.
  `NS_GET_USERNS` on a namespace descriptor reads exactly that,
  and the reading does not need `mountinfo`.
- systemd 255,
  which this guest has,
  creates the unit's *mount* namespace before the user namespace,
  so the mount namespace stays owned by the initial user namespace even though the unit's processes run in one
  of their own;
  the kernel then locks nothing,
  and both the move and the unmount are accepted from the handler's side.
  The third read's reading is of systemd `v262`,
  and this guest does not have it;
  the host does,
  and no unit was run on the host.
- When the two namespaces are made at once,
  as bubblewrap and rootless containers do,
  the copy is locked:
  the move answers `EINVAL`,
  the unmount answers `EINVAL`,
  and the copy stays at the slot,
  while the same copy unmounted in H is accepted,
  so the probe can see a difference.
  That is the reviewer's P1 confirmed by a run,
  and the read that distinguishes the two shapes is `NS_GET_USERNS`,
  not a `MNT_LOCKED` tag.
- In the systemd shapes the copy is a *peer* of the copy in H (`tags=[shared:1]` on both),
  so an unmount inside the holder would reach H's copy too;
  in the hand-made shape the copy is private (`tags=[]`) and H's copy is untouched.

Decisions needed to script it:
the arm mounts in the guest's own root namespace rather than a fresh one,
for the reason above;
the unit is started with `--machine=ubuntu@.host --user` from root,
because a process merely set to that uid has no session and the manager refuses it with "Interactive
authentication required";
the copy is mounted before the unit starts,
so that the unit's namespace copies it;
the move probe makes the parent a slave first,
as way 7's step 1 does,
because a shared parent refuses a move by itself;
the hand-made holder is waited for until its user namespace differs from the initial one,
because a read taken at once reports the mapping the process inherited.

### T0-10 with the handler's privileges

The third read's P2 says the namespace no task holds is unmeasured on a kernel that has `listns`.
This arm runs as root in the guest's initial user and pid namespaces,
which is where the design's handler runs,
and keeps a namespace alive only by a **bind of its `nsfs` file**:
the child that made it unshares,
mounts a tmpfs mark and exits,
so no task holds it and no descriptor but the bind does.

```text
# arm-T0-10-root-round4-zfs-probe-fourth-round-user-units-and-listns-20261124.log, run 1
# /proc/self/uid_map = 0 0 4294967295; the initial user namespace reads "0 0 4294967295" = true
# /proc/self/status CapEff = 000001ffffffffff
# /proc/self/ns/pid equals /proc/1/ns/pid = true
# the child 2847 has exited; its mount namespace is kept alive by the bind at $B/kept-namespace only
    id=439 parent=107 dev=0:5 root=mnt:[4026532397] mp=$B/kept-namespace fstype=nsfs source=nsfs tags=[]
# no task holds that namespace: /proc lists no process whose ns/mnt is it = true
# NS_GET_ID on the kept bind: 166
# listns(470) for CLONE_NEWNS over 2 page(s): 11 id(s), failure "end of list (ENOENT)"
# the ids listns returned: [8, 10, 18, 23, 25, 27, 31, 33, 36, 165, 166]
# the kept namespace's id is in the listed set = true
# the kept id 166 opened by open_by_handle_at(FD_NSFS_ROOT): mnt:[4026532397]
# entered the kept namespace by its listed id: {"rows": 29, "mark": ["id=438 parent=410 dev=0:53 root=/ mp=/mnt/kept-probe fstype=tmpfs source=tmpfs tags=[]"]
```

Both runs are the same and both are complete:
`listns` lists eleven mount namespaces over two pages,
the kept namespace's id is among them,
`open_by_handle_at` on the nsfs root turns that id into a descriptor naming `mnt:[4026532397]`,
and `setns` on it enters the namespace,
whose table holds the tmpfs mark the child left.
`ENOENT` on the walk is `listns`'s way of saying the page cursor is past the last id,
which the program now prints as the end of the list rather than as a failure.
So for a caller in the initial namespaces the taskless route works:
a namespace no task holds is found by id and entered,
without any descriptor kept by the scan.
The container's answer in the third round was the other half of this:
a caller without the initial user namespace's `CAP_SYS_ADMIN` is not given such an id,
and the kernel answers `ESTALE` when it tries to enter one.

Decisions needed to script it:
the bind target is a file,
because a bind of an `nsfs` file onto a directory is refused `ENOTDIR`;
the arm reads the kept namespace's id with `NS_GET_ID` on the bind it made;
and the walk from a listed id to an entered namespace is a handle of `{ns_id, 0, 0}` of type `FILEID_NSFS`
opened with `open_by_handle_at(FD_NSFS_ROOT, …)`,
which is the design's own route.

### T0-3's delivery control

The section's previous passes recorded this control as owed:
the design's T0-3 names "a mount made in H under P reaches S before the change,
recorded first",
and no log had it.
It is now its own phase,
on variant (a)'s start,
before the plain-unmount control and the action:

```text
# arm-T0-3-round4-zfs-probe-fourth-round-user-units-and-listns-20261124.log, run 1, phase variant-a-delivery-control
# the control: a mount made in H under P reaches S before the change
# control delivery to U and S = {"U": true, "S": true}
# control withdrawal in U and S = {"U": true, "S": true}
# the action the control precedes: umount2 the copy in U, plain
# observed: S slot gone = true
```

Both runs agree:
the mount made in H under P reaches U and S,
its withdrawal reaches both,
and the plain unmount that follows removes S's copy,
so the control and the action it precedes are shown on the same start.
The arm's other four phases are unchanged and their rows are in the same log.

### What these three arms show and do not show

Shown,
on `7.0.0-43-generic`:

- A holder whose mount namespace is owned by a user namespace of its own has its copy locked:
  the move and the unmount both answer `EINVAL`,
  and the copy stays.
  T0-14's hand-made shape,
  both runs,
  with the control that the same copy unmounted in H is accepted.
- A systemd user-manager unit with `PrivateMounts=yes`,
  `PrivateUsers=yes`,
  `ProtectHome=read-only` or `PrivateTmp=yes` has its processes in a user namespace but its mount namespace
  owned by the initial one,
  so nothing is locked and both actions are accepted.
  T0-14's five systemd shapes,
  both runs,
  with the system unit as the control.
- `NS_GET_USERNS` on a holder's mount namespace distinguishes the two shapes,
  which is a read a preflight can make before ending anything.
- For root in the initial namespaces,
  `listns` lists a mount namespace that only a bind of its `nsfs` file keeps alive,
  and `open_by_handle_at` on the nsfs root enters it by its id.
  T0-10-root,
  both runs.
- T0-3's delivery control fires,
  on the start the arm names.
  Both runs.

Not shown:

- A systemd that creates the unit's mount and user namespaces at once,
  which the third read reads in `v262` and this guest,
  systemd 255,
  does not have.
  The host has `v262` and no unit was run there,
  so which of the two shapes a `v262` user unit takes is unmeasured.
- A bubblewrap or rootless-container holder:
  the hand-made `unshare` holder is the same kernel class and not those tools.
- `ProtectSystem=`,
  `ReadOnlyPaths=` and the other options the third read lists,
  and any option on a unit of a user manager other than the four measured.
- Whether the design's preflight should refuse such a holder,
  and with which word:
  that is the design's,
  and the third read's P1 and P6 are its to settle.
- The taskless route for a caller that also holds a snapshot batch that must not be lost:
  the arm measures the scan's reach,
  not the removal's behaviour when the scan misses.
