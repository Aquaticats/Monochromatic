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
