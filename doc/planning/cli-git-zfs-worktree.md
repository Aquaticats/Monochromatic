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
- The whole life of a clone worktree, three rounds on a throwaway dataset holding the 24,130 tracked files of `HEAD`
  (`keep/populated-clone-life.txt`): snapshot 0.036 s; clone 0.015 to 0.017 s; root's mount 0.026 to 0.029 s,
  after which the clone showed all 24,130 files; writing 10,000 small files in it 0.296 to 0.301 s;
  root's unmount 0.099 to 0.104 s; the user's destroy of the unmounted clone 0.017 to 0.031 s,
  with the pool's `freeing` at 0 right after.
  The origin snapshot refused destruction while a clone depended on it ("snapshot has dependent clones"),
  and the user's destroy of a mounted clone failed on the unmount ("permission denied").

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

## The root-once mechanisms, each tried on this machine

The owner asked to dig deeper and allowed root ("You have passwordless sudo.").
Each mechanism was tried on a throwaway dataset under `zpcachyos/nosnap` with `setuid=off devices=off`,
and every system file the probe changed was put back and checked afterwards
(scripts `probe-the-root-once-mount-routes-sudo-polkit-fstab-and-zed.ts`,
`probe-the-polkit-and-fstab-mount-routes-again.ts`, `probe-what-each-root-once-route-lets-through-options-regex-legacy.ts`;
full output `keep/root-once-route-probe.txt`).

- **A sudoers rule with anchored regular expressions works and refused every variant tried.**
  sudo here is 1.9.17p2, and sudo 1.9.10 and later read arguments that start with `^` and end with `$` as a regex.
  For `nobody ALL=(root) NOPASSWD: /usr/bin/zfs ^mount <parent>/[a-z0-9-]+$, /usr/bin/zfs ^unmount <parent>/[a-z0-9-]+$`
  (`visudo -cf` parsed it), `sudo -l -U nobody` allowed `zfs mount <parent>/a-clone` and `zfs unmount <parent>/a-clone`
  and refused `… -o suid`, a second operand, `unmount … -f`, `unmount … -a`, a nested `<parent>/a/b`
  and a dataset outside the parent. It needs no root code of ours.
- **A sudoers rule with wildcards lets the user pick mount options.**
  For `nobody ALL=(root) NOPASSWD: /usr/bin/zfs mount <parent>/*, /usr/bin/zfs unmount <parent>/*`,
  `sudo -l -U nobody` allowed a second operand, `unmount … -a` and `unmount … -f` after the operand.
  `zfs` itself rejects the second operand and `-a` with an operand ("too many arguments",
  `share_mount` and `unshare_unmount` in `cmd/zfs/zfs_main.c` at tag `zfs-2.4.4`, lines 7696 and 7954),
  but it reads options placed after the operand (`getopt_long` permutes; `zfs get name zpcachyos -H` printed no header),
  and root's `zfs mount <clone> -o suid,dev` mounted the clone without `nosuid,nodev` although the parent has
  `setuid=off devices=off`. So that rule lets the user turn setuid and device files back on for any dataset under the parent.
  mitosis-db prints a sudoers rule from `mitosis zfs setup`, but its text was not read
  (the docs.rs source view dropped the function bodies), so whether its rule has this shape is unknown.
- **A polkit rule with systemd mount units mounts, and lets the user mount anything at the allowed path.**
  Without a rule, `StartTransientUnit` from the user answered "Access denied as the requested operation requires
  interactive authentication"; with a rule allowing one unit name, the user's call over D-Bus
  (`What=<dataset>`, `Type=zfs`, `Options=zfsutil`) mounted the clone in 0.011 s, `StopUnit` unmounted it in 0.011 s,
  and another unit name was refused.
  But under the same rule the user's `Options=zfsutil,suid,dev` mounted the clone without `nosuid,nodev`,
  and `What=tmpfs Type=tmpfs` mounted a tmpfs at that path:
  systemd hands polkit only the unit name and the verb (`bus_verify_manage_units_async_impl(m, name, "start", …)`
  in `src/core/dbus-manager.c` on systemd main), so no rule can check what is mounted or how.
  A user allowed this could mount a block device such as the EFI partition there with options of their choosing,
  which is as good as root; this route is unfit to ship.
  Unit names hold `\x2d` escapes, which a rule must double inside a JavaScript string (the first try failed on that).
- **An fstab `user` entry does not work for ZFS**, with `zfsutil` on an ordinary dataset
  or without it on a dataset whose `mountpoint` is `legacy` (the Server Fault answer's form):
  util-linux 2.42 ran `mount.zfs` for the user, and both answered "zfs_mount_at() failed: Insufficient privileges".
- **A ZED handler works.** With ZED started (it is installed but disabled here) and a handler under `/etc/zfs/zed.d/`
  that mounts a clone made under the parent and unmounts one whose user property `org.monochromatic:unmount`
  says `requested`, the clone was mounted 0.006 s after the user's `zfs clone` returned
  and unmounted 0.030 s after the user set the property.
  On start ZED replayed older events still in the kernel's queue, so a handler must recheck before acting.
  ZED was stopped and left disabled.

## Without root at all

- **fuse-overlayfs over a read-only snapshot** (the user reads snapshots through `.zfs/snapshot`, and `fusermount3`
  is setuid): mount 0.010 to 0.013 s, unmount 0.07 to 0.12 s, writes and copy-up work.
  Against the native dataset in the same rounds: reading every tracked file 3.7× cold, then 1.0× to 1.3×;
  walking `package/` 11.5× cold, then 1.7× to 2.1×; a warm `rg` through `package/` 2.2×;
  writing and deleting 10,000 small files 3.4× every round. That cost lasts for the worktree's life.
  Its lower layer must outlive the worktree, so it needs snapshots of the wrapper's own (sanoid prunes its own after 25 hours)
  and so `snapshot` delegated on the dataset that holds the source.
  fuse-overlayfs splits `lowerdir` on `:`, and sanoid's snapshot names hold colons.
- **Reflink copies** within one dataset: 5.63 s for the 56,478-file worktree.
  Genesis's route copies the unchanged files and build folders this way and then has Git check out the files that differ;
  only the full copy was timed, not Genesis's split.
- **`zfs zone`** (in 2.4.4; `zfs-zone(8)`) attaches a dataset whose `zoned` property is on to a user namespace,
  and is run by root. Inside that namespace its administrator mounts it,
  and "a zoned file system cannot be mounted outside the user namespace", so the person's own shells would not see it,
  the same limit as 2.5's `zoned_uid`.

## What container tools do

- Docker's `zfs` storage driver is configured with root (`sudo zpool create … -m /var/lib/docker`, `/etc/docker/daemon.json`),
  and its daemon makes a snapshot and a clone per layer (docs.docker.com, "ZFS storage driver").
  Whether rootless Docker can use it was not settled: the rootless page read lists no storage drivers.
- Incus runs its `zfs` driver from its root daemon; its `zfs.delegate` option (ZFS 2.2 or later) hands a dataset to a
  container so the `zfs` command works inside it, which is the namespace route of `zfs zone`
  (linuxcontainers.org, Incus "ZFS - zfs").
- Each keeps the mounts in a root daemon or inside a namespace; none gives an ordinary user mounts in their own namespace.

## Dependencies to pick

- **How the wrapper drives ZFS.** The `zfs` program, as mitosis-db and Genesis do, or a Rust crate.
  A package listing in a search on 2026-11-17 (crawl date not shown, so ages are relative to an unknown day) gave:
  `libzetta` 0.5.0 "almost 2 years ago", `razor-libzfscore` 0.13.1 "almost 2 years ago",
  `razor-safe-lzc` 0.12.0 "almost 3 years ago", `libzfs` 0.6.16 "over 4 years ago",
  `libzfs-sys` 0.5.11 "over 5 years ago".
  Each binds the C libraries of whatever OpenZFS it was written against; none was checked against 2.4.4,
  and none lifts the root requirement for mounts.
- **What mounts and unmounts**: the ranked list in "Recommendation".
- **What the reflink fallback calls**: `FICLONE` per file (the call Genesis's `cp --reflink` reaches).

## Layout facts the design must meet

- A clone's source must be a dataset. The main checkout is a folder inside `zpcachyos/ROOT/cos/home` today,
  so the route needs the checkout moved into a dataset of its own once, with worktrees as clones beneath it.
- sanoid's entry names `zpcachyos/ROOT/cos/home` with no `recursive` line (`/etc/sanoid/sanoid.conf`),
  so a child dataset holding the checkout would drop out of the hourly snapshots unless sanoid names it too.
- `zfs-space-guard` destroys only `<dataset>@autosnap_*` of its own dataset (`/usr/local/sbin/zfs-space-guard`, line 44),
  so it leaves the wrapper's own snapshots alone.
- Reflinks fail across datasets, so the fallback works only for a worktree in the source's own dataset.

## Recommendation (this session's, not decided)

Route: dataset clones with a reflink fallback (C), because a clone plus its mount takes 0.04 to 0.05 s
(plus a 0.04 to 0.12 s snapshot) and carries ignored state the way the Btrfs route does,
while the fallback keeps a 5.6 s route on machines without the root setup.
Driver: the `zfs` program, because no crate in the listing is shown current for OpenZFS 2.4.
Mount mechanism, ranked: an anchored-regex sudoers rule > a ZED handler > a root-owned helper of ours > fuse-overlayfs.
Out: a polkit rule (lets the user mount anything at the path), a wildcard sudoers rule (lets the user turn setuid back on),
fstab `user` entries (do not work).
The regex rule ranks above ZED because it needs no root code of ours and no daemon turned on,
and it refused every variant tried; ZED ranks above our own helper because ZED was probed working
and the helper was never built (it ranked first in this page's earlier draft, before the regex rule was tried);
the helper ranks above fuse-overlayfs because its clones run at native speed,
where the overlay costs 2.2× to 3.4× for as long as the worktree lives.

## Not researched yet

- containerd's ZFS snapshotter.
- Genesis's split (reflink the unchanged files, Git checks out the rest), timed as a whole.
- How the wrapper's controls get a real pool (a file-backed pool made once by root, a delegated dataset on the
  person's pool as these probes used, or a VM with ZFS).
- The text of mitosis-db's printed sudoers rule.

## Open choices for the owner

- The route: dataset clones with a reflink fallback, dataset clones alone, or reflinks alone.
- For dataset clones, the root mechanism that mounts and unmounts, from the ranked list in "Recommendation".
- Driving ZFS through the `zfs` program or a crate.
- Moving the main checkout into a dataset of its own, and naming that dataset in sanoid's configuration.
