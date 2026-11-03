# Persistent disk compression

## Requested scope at 2026-11-03T01:11:03-05:00

The user requested:
"Set all my disks to use zstd level 3 and make it permanent."
They extended this to all future disks, volumes, subvolumes, and future subvolumes,
then instructed:
"For fs s that lack this setting, ignore them."

## Host changes at 2026-11-03T01:11:03-05:00

- `lsblk` and `btrfs filesystem show --mounted` identify the NVMe Btrfs filesystem,
  spanning encrypted partitions,
  and the encrypted SATA Btrfs filesystem.
- Changed the Btrfs entries in `/etc/fstab` from `compress=zstd:1` to `compress=zstd:3`.
- Changed `/etc/systemd/system/mount-860.service` from implicit `compress=zstd`
  to explicit `compress=zstd:3`.
  Reloaded systemd without restarting the mounted disk's unlock service.
- Remounted `/var` with `compress=zstd:3`.
  `findmnt --types btrfs --output TARGET,SOURCE,OPTIONS` subsequently reported level 3
  on every Btrfs mount,
  including `/etc`, `/var/home`, `/sysroot`, and `/var/mnt/encrypted`.
  `/sysroot` remains read-only.
- Original configuration copies are in `${HOME}/temp/agent/zstd-level3.5DYHCrEG/`.
- `findmnt --verify --verbose` reports no parse errors,
  but reports `wrong order: /var/home specified before /var`,
  a noncanonical `/home` warning,
  and a swapfile warning.
  Running the same command against the unchanged backup reproduced the same diagnostics.
  These are pre-existing and outside the compression change.

## Remaining work at 2026-11-03T01:11:03-05:00

- Configure UDisks and SteamOS defaults for future Btrfs media.
- Verify an event-driven systemd policy for Btrfs mounts outside those automounters,
  using disposable disk images and checking subvolume inheritance.
- Replace level 1 in the persistent `rootflags` boot argument.
  An existing `rpm-ostree update check`, PID `1403954`, holds an Upgrade transaction.
  A question asks whether the user authorizes cancelling that check.
  Do not cancel without their answer.
  The bounded wait is command session `48868`,
  scope `run-p1778395-i35317046.scope`,
  with a 600-second deadline.
- Verify final runtime settings, persisted configuration, and generated units.

## Scope and evidence limits

The installed `btrfs(5)` manual documents filesystem-wide compression mount options,
shared by subvolume mounts,
and the default zstd level of 3.
This changes the policy for eligible future writes.
Existing extents are not recompressed.
Swapfiles, inode exceptions, unsupported filesystem types,
and read-only data must not be rewritten or reformatted.
No reboot or disk-wide defragmentation is authorized by this work.

## Future mount coverage at 2026-11-03T01:19:52-05:00

Configured `/etc/udisks2/mount_options.conf` with `btrfs_defaults=compress=zstd:3`.
Updated the active Btrfs home and SD-card values in `/etc/default/steamos-btrfs` to level 3.

Installed and enabled `btrfs-zstd-policy.service`.
It reconciles writable Btrfs mounts using their existing live mount options,
then waits for kernel mount/remount notifications through `findmnt --poll`.
A 60-second wait deadline covers the gap between reconciliation and listener startup.
It uses a quarter of a CPU core at most,
128 MiB memory,
16 tasks,
and I/O weight 10.
It must share the host mount namespace.

Disposable sparse images verified:
the old UDisks default omitted compression;
the configured default supplies `compress=zstd:3`;
direct policy selection excludes read-only mounts;
new manual mounts and later remounts are corrected;
and newly created/mounted subvolumes inherit the filesystem setting.
The `rw`, `nodev`, `nosuid`, and `noexec` options were preserved.
All fixture mounts and loop devices were removed.
Test harnesses are retained in the private backup directory as `verify-policy.ts` and `verify-udisks.ts`.

The user answered "Wait for the check to finish"
and then requested "Please query the boot-arg update automatically."
The expired 600-second wait has been replaced by `boot-args.ts`,
running in `btrfs-zstd3-boot-args.scope`,
command session `47909`.
It waits for PID `1403954` with a 3600-second deadline,
checks that no other rpm-ostree transaction is active,
updates the current persistent rootflags while retaining unrelated options,
and verifies the result.
A successful run writes `boot-args-result.json`.
Do not cancel the user's update check.
The original handover commit is `9f5acfb801`.

Remaining work:
check F2FS compression support for future media,
complete final verification and rendering of this record,
and collect the automatic boot-argument result.

## F2FS and final mount verification at 2026-11-03T01:26:10-05:00

The running kernel configuration enables F2FS compression and zstd.
A disposable F2FS image demonstrated that this kernel accepts a live change
from its default lz4 algorithm to `compress_algorithm=zstd:3`.
Adding `compress_extension=*` made a newly created file receive the compression attribute.

Added F2FS defaults to UDisks and the SteamOS media configuration.
Installed and enabled `f2fs-zstd-policy.service`,
with the same limits and notification-driven lifecycle as the Btrfs policy.
Its mount filters require a writable F2FS mount that reports `compress_algorithm`,
so images without the on-disk compression feature are excluded.
The policy changes the algorithm and ensures the wildcard extension is present.

The `verify-f2fs.ts` disposable-image checks passed for direct reconciliation,
the running system service,
UDisks defaults with the reconciliation service stopped,
and an F2FS image without compression support.
The unsupported image mounted successfully and remained uncompressed.
The test preserved `rw`, `nodev`, `nosuid`, and `noexec`;
its new file had the compression attribute.
The service was restarted after the UDisks-only check.

The first UDisks F2FS harness used `udevadm settle --timeout=30`,
which emitted `Timed out while waiting for udev queue to empty.`
The installed `udevadm(8)` manual confirms that this waits for the entire udev queue.
The corrected harness uses `udisksctl loop-setup`,
as the already successful Btrfs harness does,
and all F2FS checks then passed.
No cause is attributed to the unrelated queued events.

`systemd-analyze verify` passed for both policy services and `mount-860.service`.
Both policy services are enabled and running.
`findmnt` still reports level 3 on every real Btrfs mount,
and `lsblk` confirms that fixture loop devices were removed.

The automatic boot-argument job observed the end of the update check
and started a `KernelArgs` transaction.
Its completion and result file remain to be collected.
The staged deployment's `/etc` has not yet received the local files;
do not mistake the pre-finalization tree for verified next-boot configuration.
Verify OSTree's configuration-merge contract before declaring persistence complete.

## Coverage limits

The policy covers Btrfs and compression-capable F2FS mounts visible in the host mount namespace.
UDisks and SteamOS defaults apply compression at mount time.
The reconciliation services correct other writable mounts after notification,
with a 60-second fallback for a listener startup race.
A manually mounted filesystem can receive writes before reconciliation.

Read-only mounts are not made writable.
Subvolumes share the Btrfs filesystem compression setting.
Existing extents and explicit per-inode compression exceptions are not rewritten.
No unsupported filesystem is reformatted or given a new on-disk feature.
Filesystems installed in the future with different compression APIs
are outside the verified Btrfs/F2FS coverage.
