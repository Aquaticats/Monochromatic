# Persistent disk compression

## Completed policy at 2026-11-03T01:36:09-05:00

The user requested:
"Set all my disks to use zstd level 3 and make it permanent."
They included all future disks,
volumes,
subvolumes,
and future subvolumes.
They then instructed:
"For fs s that lack this setting,
ignore them."

The current NVMe and encrypted SATA Btrfs filesystems use `compress=zstd:3`.
The setting is saved in administrator configuration and the persistent boot arguments.
Future host mounts are covered for Btrfs and compression-capable F2FS.
All work in this record is complete.

## Installed configuration

- `/etc/fstab`:
  the Btrfs entries for `/`,
  `/home`,
  and `/var` changed from level 1 to level 3.
  The commented SATA entry also names level 3.
- `/etc/systemd/system/mount-860.service`:
  the encrypted SATA mount now explicitly requests `compress=zstd:3`.
  The live filesystem already used the implicit default level of 3.
  The unlock service was not restarted.
- `/etc/udisks2/mount_options.conf`:
  `btrfs_defaults=compress=zstd:3`
  and `f2fs_defaults=compress_algorithm=zstd:3,compress_extension=*`.
- `/etc/default/steamos-btrfs`:
  Btrfs home and SD-card options use `compress-force=zstd:3`;
  F2FS media options use `compress_algorithm=zstd:3,compress_extension=*`.
  Other mount options were preserved.
- `btrfs-zstd-policy.service` and `f2fs-zstd-policy.service`:
  installed in `/etc/systemd/system/`,
  enabled for `multi-user.target`,
  and running.
- Persistent `rootflags`:
  `rootflags=subvol=root,user_subvol_rm_allowed,compress=zstd:3`.
  Other kernel arguments were preserved.

The configuration files are root-owned with mode `0644`.
Original copies of the changed pre-existing files are retained in
`/home/user/temp/agent/zstd-level3.5DYHCrEG/`.

## Future mounts and subvolumes

UDisks and SteamOS defaults request compression at mount time.
The policy services reconcile other writable host mounts,
then wait for kernel mount/remount notifications using `findmnt --poll`.
A 60-second deadline covers the gap between reconciliation and listener startup.
Only mounts needing a change are remounted,
using live mount options to preserve existing access and security flags.

The F2FS filters require `compress_algorithm` to be reported by the mount,
which excludes images without the on-disk compression feature.
The F2FS policy also enables compression for newly created files with `compress_extension=*`.
Btrfs subvolumes share the parent filesystem's mount-level compression policy.

Each service is limited to a quarter of a CPU core,
128 MiB memory,
16 tasks,
and I/O weight 10.
The services share the host mount namespace.

## Verification

Disposable sparse disk images exercised these boundaries:

- The initial UDisks Btrfs default omitted compression.
  With the new configuration,
  UDisks mounted a new image with `compress=zstd:3`.
  This test ran before the Btrfs reconciliation service was enabled.
- Direct Btrfs reconciliation changed a fixture from level 1 to level 3.
  Selection excluded a read-only bind mount,
  which remained read-only.
- The running Btrfs service corrected a new manual mount and a later remount.
  A newly created and separately mounted subvolume inherited level 3.
- Direct F2FS reconciliation changed a compression-capable image from its default lz4 algorithm
  to zstd level 3.
  A newly created file received the compression attribute.
- The running F2FS service corrected a new manual mount.
- UDisks F2FS defaults passed with the F2FS reconciliation service stopped,
  proving the defaults independently.
  The service was restarted afterward.
- An F2FS image without compression support mounted successfully and remained uncompressed.
- The fixture checks preserved `rw`,
  `nodev`,
  `nosuid`,
  and `noexec`.
  Correctly configured F2FS mounts were excluded from repeated extension changes.

All fixture mounts and loop devices were removed.
The private directory retains `verify-policy.ts`,
`verify-udisks.ts`,
and `verify-f2fs.ts`.

`systemd-analyze verify` passed for both policy services and `mount-860.service`.
`findmnt --types btrfs --output TARGET,OPTIONS` reported level 3 on every real Btrfs mount,
including `/etc`,
`/var/home`,
`/sysroot`,
and `/var/mnt/encrypted`.
`/sysroot` remains read-only.

## Boot arguments and persistence

The user chose "Wait for the check to finish"
for their existing `rpm-ostree update check`,
then requested "Please query the boot-arg update automatically."

The bounded `boot-args.ts` job waited for that process,
confirmed the daemon was available,
changed the persistent arguments,
and re-read them.
It completed successfully at `2026-11-03T06:36:09.936Z`.
The daemon journal records
`Txn KernelArgs on /org/projectatomic/rpmostree1/default successful`.
The result is retained as `boot-args-result.json`.
The temporary job has exited.

`ostree admin config-diff` recognizes the local `fstab`,
UDisks defaults,
SteamOS defaults,
both policy units,
both enablement symlinks,
and the SATA mount service.
The installed `ostree-finalize-staged.service` invokes
`ostree admin finalize-staged` through `ExecStop` at shutdown.

The repository's [OSTree configuration persistence record](../troubleshooting/rpm-ostree-inotify-persistence.md)
documents the administrator `/etc` merge and the distinction between upgrades and rollback deployments.
The packaged build's reported Git identifier is unavailable from upstream;
that existing source-version limitation remains.
The staged tree before finalization is not the completed next-boot configuration.
No direct edits were made to the staged deployment.
Persistence was verified through saved configuration and the supported merge mechanism,
without rebooting the machine.

## Diagnostics and limits

`findmnt --verify --verbose` reported no parse errors,
but reported `wrong order: /var/home specified before /var`,
a noncanonical `/home` warning,
and a swapfile warning.
The same command against the original backup reproduced these diagnostics.
They predate this change.

The first F2FS UDisks harness used `udevadm settle --timeout=30`,
which emitted `Timed out while waiting for udev queue to empty.`
The installed `udevadm(8)` manual confirms that the command waits for the entire udev queue.
The events keeping it nonempty were not identified.
Using `udisksctl loop-setup` in the harness avoided that global prerequisite,
and the F2FS checks passed.

The policy covers Btrfs and compression-capable F2FS mounts visible in the host mount namespace.
A manual mount can receive writes before notification-driven reconciliation.
Read-only mounts are not made writable.
Existing extents and explicit per-inode exceptions are not rewritten.
Swapfiles and unsupported filesystems are left alone.
A filesystem installed later with a different compression API is outside this verified coverage.
No disk-wide defragmentation,
reformatting,
or reboot was performed.

The handover was rendered with the installed CommonMark renderer.
The scoped Markdown lint and `git diff --check` passed.
Implementation progress was recorded in commits
`9f5acfb801`,
`99b09defa9`,
`21d29590f0`,
and `f34bfe56ee`.
