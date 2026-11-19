# Measurements owed before accepting the ZFS worktree design

**Non-final draft by an agent, unapproved.** This records guest observations and design implications, not design
acceptance. The account ownership decision in the [planning record](cli-git-zfs-worktree.md#owners-answers-of-2026-11-18)
makes the refusal repeat a pre-acceptance condition, not something to postpone until implementation.

## Fixture and evidence

The 2026-11-18 gate review of proposal `71fa25ece8` and the dated corrections in the
[clones research](cli-git-zfs-clones-mounted-under-the-checkout.md) supplied the open measurements. The guest probe
reused the descriptor calls from that research's `guest-extra-commands.txt`. Full commands, output and assertions
are in `~/temp/agent/zfs-measurements-owed-before-acceptance-2026-11-18/keep/`, in `guest-probe.py` and the per-item logs.
Each item uses a separate encrypted file-backed pool in `zfs-acceptance-mvm`, an Ubuntu 24.04 `mvm` libvirt guest.
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
explicit-mount branch of `zfs-2.4.4`'s `cmd/zfs/zfs_main.c:7240` to `:7248`. The proposal's citation at
lines 118 to 119 points at the unmount branch instead. Treat the refusal as expected: an inherited legacy
property does not name a target slot, and the design must not fall back to a path-based mount.
