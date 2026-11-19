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
`recordsize=32K` and `compression=zstd-3`. No host filesystem state or propagation was changed.

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
