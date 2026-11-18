# A fast worktree route on ZFS for the git wrapper: measurements, precedent and open choices (2026-11-17)

Not final and not approved: this is a draft that gathers what was measured and read
before the owner picks the route.
The Btrfs route it sits beside is designed in `doc/planning/cli-git-btrfs-worktree.md`.

## Why

The owner, 2026-11-16: "B: Full parity before the wipe" (the wrapper supports ZFS before either old drive is wiped);
2026-11-17: "this implies we really should be doing proper fast route first" and "A: ZFS route first (Recommended)".
On ZFS the wrapper's ladder does nothing today, so `git worktree add` takes Git's own checkout
and the wrapper's dependency install.

## What was measured on this machine

All on OpenZFS 2.4.4 (`zfs version`), pool `zpcachyos`, home dataset `zpcachyos/ROOT/cos/home`,
block cloning active and `zfs_bclone_enabled` 1, one sample each while about twenty agents ran,
scripts and outputs under `~/temp/agent/zfs-fast-route-design-facts-2026-11-17/keep/`.

- The wrapper's `git worktree add --detach` of `main` took 48.84 s and left 56,478 files;
  the repository tracks 24,129 (`git ls-files`), so most of that time is the dependency install, not Git's checkout.
- A plain `git checkout` of the 24,129 tracked files into a fresh dataset took 2.44 s.
- A reflink copy (`cp -a --reflink=always`) of the whole 56,478-file worktree took 5.63 s;
  of the main checkout's `node_modules` (4,821 files) 1.04 s; deleting those copies 1.56 s and 0.14 s.
- Block cloning works for an unprivileged user inside one dataset:
  a 64 MiB reflink copy grew the pool's `bclonesaved` by exactly 67,108,864 bytes, a plain copy by 0.
- No reflink crosses datasets here: FICLONE answered "Invalid cross-device link",
  and `cp --reflink=auto` (copy_file_range) copied the data with `bclonesaved` unchanged.
- With `create,clone,snapshot,destroy,mount,promote` delegated on a throwaway dataset (destroyed afterwards),
  the user could snapshot (0.118 s for a dataset holding the 24,129-file checkout) and clone (0.017 s),
  but `zfs clone` printed "filesystem successfully created, but it may only be mounted by root",
  `zfs mount` answered "Insufficient privileges", `mount -t zfs` answered "zfs_mount_at() failed: Insufficient privileges",
  and destroying a mounted clone answered "cannot unmount … permission denied".
  Root's `zfs mount` took 0.028 s; the clone then showed all 24,129 files and took writes.
- A snapshot's files are readable by the user through `<mountpoint>/.zfs/snapshot/<name>`,
  which the kernel mounts on first access.

## What others say and do

- `zfs-allow(8)` (OpenZFS master): "Delegations are supported under Linux with the exception of mount, unmount,
  mountpoint, canmount, rename, and share", because the Linux mount command restricts the global namespace to root.
  The commit that added Linux delegation says "Under Linux only the root user is allowed to modify the namespace
  (unless it's a private namespace)".
- A Server Fault answer on the same symptom (OpenZFS 2.2.2) lists the workarounds:
  `mountpoint=legacy` with an fstab `user` entry, a narrow `sudo zfs mount/umount` rule,
  or a systemd unit the user may trigger.
- mitosis-db, a Rust tool that clones Postgres data directories on ZFS (`clone_backend/zfs.rs` on docs.rs),
  runs only mount, unmount, `set mountpoint`, `set canmount` and rename as `sudo -n zfs …`
  under a sudoers rule its `zfs setup` command prints for an admin to install once,
  refuses any mountpoint outside its parent dataset before calling sudo,
  and sets `setuid=off devices=off` on the parent because the rule lets the user mount as root.
- Genesis (agent worktrees) makes copy-on-write worktrees with `cp --reflink=auto` on ZFS 2.2 and later:
  it clones the source worktree's unchanged files and build folders (`node_modules`, `target`)
  and lets Git check out only the files that differ, falling back to `git worktree add`.
- OpenZFS PR #18167 (merged, milestone 2.5, not in 2.4.4) adds `zoned_uid` for rootless Podman:
  the caller must be in a user namespace owned by that UID with `CAP_SYS_ADMIN` inside it,
  so its mounts live in that namespace and are not seen by the person's own shells.
- A Full Disclosure post of 2026-08 reports OpenZFS flaws reachable from unprivileged user namespaces
  with `/dev/zfs` ("pool-level paths omit the zone/delegation gate"), unfixed upstream when posted.

## Open choices for the owner

- The route: dataset clones with a reflink fallback, dataset clones alone, or reflinks alone.
- For dataset clones, the root mechanism that mounts and unmounts:
  a narrow sudo rule, a polkit rule with systemd mount units, a root service of our own, or fstab `user` slots.
