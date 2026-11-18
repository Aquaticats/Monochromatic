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

## ZED against the regex sudoers rule, probed again (2026-11-18)

The owner, 2026-11-18: "My impression is ZFS event daemon is the best idea. It's interesting we didn't agree there.
Am I missing something? Are you missing something?"
The earlier ranking put the regex rule first for its install cost alone, and neither mechanism had been run
from the places where worktrees get made, so both were probed again on throwaway datasets under `zpcachyos/nosnap`
(scripts and output in `~/temp/agent/zed-against-sudo-for-clone-mounts-2026-11-18/keep/`:
`probe-zed-against-sudo-from-each-caller-context.ts` with `zed-against-sudo-probe.txt`,
`probe-confined-unit-sight-and-the-regex-rule-run-for-real.ts` with `confined-unit-sight-and-regex-rule-probe.txt`,
`probe-confined-unit-control-and-reflink-fallback.ts` with `confined-unit-control-and-reflink-fallback-probe.txt`;
each output file marks the lines of a first run that a script fault voided, and the rerun beneath them).
Every dataset was destroyed afterwards; the sudoers rule, the throwaway account and the ZED handler were removed;
ZED was left stopped and disabled; `/etc/passwd` and `/etc/group` read byte for byte as before.

### What the delegated user cannot change on a clone

`zfs clone -o mountpoint=…`, `-o setuid=on`, `-o canmount=noauto` and `-o overlay=off`
each answered "permission denied", as did `zfs set mountpoint=…` and `zfs set setuid=on` on a clone and `zfs rename` of a clone out of the parent.
So under either mechanism a clone mounts where the parent's mountpoint puts it, with the parent's options.

### A planted symlink redirects both mechanisms

With the worktrees parent's folder owned by the user, the user made a symlink `<worktrees>/planted`
pointing at a scratch folder and a clone named `planted`;
root's `zfs mount` of that clone (what the regex rule allows and a plain handler runs)
mounted it on the scratch folder (`rw,nosuid,nodev`).
`zfs_mount_at` (`lib/libzfs/libzfs_mount.c` at tag `zfs-2.4.4`) only `lstat`s the mountpoint and, with `overlay` on
(the value here), skips the empty-folder check before `do_mount` (`lib/libzfs/os/linux/libzfs_mount_os.c`)
calls `mount(2)` with the path, and `mount(2)` follows links.
A link pointing at `/etc` would hand the user `/etc`, so this is root for whoever can plant it.
With the folder owned by root, mode 0755, the user's `ln -s` there was refused, the clone still mounted,
and the user wrote inside it (the clone's own top folder is the user's).
So the worktrees parent's folder must be root's under either mechanism.
The same code mounts the parent dataset itself at boot (`zfs mount -a`), so a parent mounted inside a folder the user owns
could be redirected the same way while it is unmounted; that was not probed, and it holds today for every dataset
mounted inside the home folder.
A handler could also check the path before mounting (not tried); a sudoers rule sees only the arguments.
Root's mount leaves its mountpoint folder behind after the clone is destroyed.
Who this guards against: on this machine the account has passwordless sudo, so a process running as the account gains
nothing from it; it matters for an account that holds only the root-once setup (the wrapper on other machines,
or this one if blanket sudo ever ends) and for a confined agent process that can still write that folder.

### From each place a worktree can be made

- **This shell, and a unit started the way the driver on main starts Pi runs**: no_new_privs reads 0,
  the host's namespaces; the delegated clone and `sudo -n zfs mount` both worked.
- **Under no_new_privs** (`setpriv --no-new-privs`): the delegated clone worked, and sudo refused with
  "The "no new privileges" flag is set, which prevents sudo from running as root."; a mount root made in the host's
  namespace (as ZED makes it) was seen and writable from there.
  The throwaway account holding only the regex rule got the same refusal under the flag.
  So under no_new_privs ZED works and the sudoers rule does not.
  No caller read on this machine sets the flag today: this Claude Code session's own shell and a unit with the
  driver's properties both read 0 (no Pi run was going, so a Pi tool process itself was not read);
  systemd's `NoNewPrivileges=`, containers and agent sandboxes can set it.
- **Inside a user namespace**, both `unshare --user --map-current-user` and a unit with `ReadOnlyPaths=`
  (as branch b75c20af41 starts Pi runs: systemd gave it a user namespace, uid map `1000 1000 1`,
  though `PrivateUsers` read `no`): `zfs list` of the dataset answered "dataset does not exist" and the clone failed.
  Neither mechanism helps there: the caller cannot make the clone, so ZED gets no event and sudo has nothing to mount.
- **What such a confined unit sees of a clone mounted outside it**: with the worktrees parent itself named in
  `ReadOnlyPaths=`, a clone root mounted while the unit ran never appeared inside it, and one mounted before the unit
  started appeared read-only, or writable when its own path was named in `ReadWritePaths=`.
  The control, with the parent outside every `Read*Paths=` (the unit made read-only an unrelated folder instead),
  saw a clone mounted while it ran, writable. So what hides a later mount is the read-only bind of the parent,
  not the unit's namespaces.
- **Who makes worktrees today**: the driver makes each run's worktree before it starts the run's unit
  (`makeWorktree` in `package/pi-shared/subagent-driver/src/start-pi-run-in-a-worktree.ts`), outside any confinement;
  an agent that runs `git worktree add` inside a confined unit cannot clone.
- **The reflink fallback inside a confined unit works**: a `cp --reflink=always` of a 64 MiB file within one writable
  folder, and from a folder named in `ReadOnlyPaths=` into one named in `ReadWritePaths=` of the same dataset,
  each grew the pool's `bclonesaved` by exactly 67,108,864 bytes, as the same copy outside any unit did
  (a first run that read `bclonesaved` before the pool synced read 0 for all three, the control included, and is void).

### The regex rule, run for real

A throwaway system account holding only
`/usr/bin/zfs ^mount <parent>/[a-z0-9-]+$, /usr/bin/zfs ^unmount <parent>/[a-z0-9-]+$` mounted a clone
(`nosuid,nodev`) and unmounted it, and was refused `-o suid,dev` after the operand, `-o suid` before it,
a second operand, a dataset outside the parent, a nested name, `unmount … -f` and `unmount -a`.
User nobody could not stand in: PAM refused its account ("Account expired or PAM config lacks an "account" section").

### A ZED handler that reconciles instead of reacting

The handler acted on any history event under the probe's tree by reading the whole parent: it mounted every clone
not flagged, unmounted every clone whose `org.monochromatic:unmount` said `requested`,
and wrote `org.monochromatic:unmount-result=failed` when an unmount failed.

- **Speed**, five rounds alternating the two arms and swapping their order each round:
  from the start of `zfs clone` to a mounted worktree, the ZED arm took 0.48 to 0.60 times as long as
  `zfs clone` followed by `sudo -n zfs mount`; the unmount (the user's `zfs set` plus the handler)
  took 0.69 to 0.83 times as long as `sudo -n zfs unmount`.
  (A ratio taken from when `zfs clone` returned would read about a quarter, but by then the handler had already begun.)
  Most of sudo's time is sudo: `sudo -n /usr/bin/true` took 29 times as long as `/usr/bin/true` in the same rounds.
  Both arms stay in tens of milliseconds, against the 48.84 s of today's `git worktree add`.
- **Events while ZED was stopped**: a clone made then was mounted shortly after ZED started,
  since ZED resumes from the event id it saved.
- **A lost event**: with the handler held (logging and doing nothing, standing in for an event ZED missed),
  the clone stayed unmounted, and stayed so after the hold was lifted until the user set a property on the parent;
  then it was mounted. A handler that reconciles the whole parent recovers a lost event at the next event,
  and the wrapper can make that event itself.
- **A busy unmount**: with a process sitting inside, the handler's unmount failed and the failure property appeared;
  after the process left, a nudge unmounted it. The handler's own property writes are events too,
  so it writes a value only when it changes, or it would call itself without end.
- **From source** (`keep/` copies at tag `zfs-2.4.4`): the kernel holds at most `zfs_zevent_len_max` events
  (512 here) and drops the oldest when full (`zfs_zevent_insert`, `module/zfs/fm.c`), which ZED logs as
  "Missed %d events" (`cmd/zed/zed_event.c`); ZED runs zedlets in parallel up to its job limit (`cmd/zed/zed_exec.c`),
  so two runs of the handler can race and each step must survive the other (a second mount just fails);
  ZED ignores a zedlet that root does not own or that group or others can write (`cmd/zed/zed_conf.c`).
- **What turning ZED on brings with it**: the package's other zedlets in `/etc/zfs/zed.d/` (syslog, notify,
  enclosure LED and slot, scrub after resilver, and the list cacher, which does nothing here because
  `/etc/zfs/zfs-list.cache/` does not exist); which act depends on `zed.rc`, which sets `ZED_EMAIL_ADDR`,
  `ZED_USE_ENCLOSURE_LEDS` and `ZED_SYSLOG_SUBCLASS_EXCLUDE` here.
- **Precedent**: upstream's own `history_event-zfs-list-cacher.sh`
  (OpenZFS commit "systemd mount generator and tracking ZEDLET") is a history-event zedlet that keeps mount state for the mount generator;
  one search found no project that mounts datasets for users from ZED.

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
  so the route needs the checkout moved into a dataset of its own once.
  Its clones cannot be children of that dataset: a child mounts inside the checkout's folder, which the user owns,
  and a planted symlink there redirects root's mount (section "A planted symlink redirects both mechanisms").
  They go under a separate parent dataset whose folder is root's.
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
Mount mechanism, ranked (changed 2026-11-18): a reconciling ZED handler > an anchored-regex sudoers rule
> a root-owned helper or service of ours > fuse-overlayfs.
Out: a polkit rule for transient units (lets the user mount anything at the path), a wildcard sudoers rule
(lets the user turn setuid back on), fstab `user` entries (do not work).
ZED ranks above the regex rule, a close call: the only place measured where one works and the other does not is a
caller under no_new_privs without a user namespace, and no caller read here sets that flag today;
the rest is that the caller never runs anything as root, its worktree was ready in 0.48 to 0.60 of the sudo arm's time,
and its handler can check the mount path (not tried), where a sudoers rule sees only arguments.
So the ranking is a bet on future callers and on a path check still to be built.
The regex rule's real advantages: it is synchronous, `zfs` gives the caller its own error text,
it needs no daemon and no root code of ours, and it was run for real and refused every variant.
ZED's costs: a daemon turned on (with the package's other zedlets), root code of ours,
and an asynchronous request whose waiting, deadline and error report the wrapper must design,
which the 2026-11-18 probes showed workable (replay after a stop, a lost event recovered, a busy unmount reported).
The earlier draft ranked the regex rule first for needing no root code and no daemon, which is install cost;
the 2026-11-18 probes measured where each works and what each can check, and that moved it.
The regex rule ranks above a helper of ours because it needs no root code
and was run for real and refused every variant, while the helper was never built,
and a setuid helper fails under no_new_privs as sudo does;
a root service with its own request channel is the only kind that could make clones
for a caller inside a user namespace, also unbuilt.
The helper ranks above fuse-overlayfs because its clones run at native speed,
where the overlay costs 2.2× to 3.4× for as long as the worktree lives.
Under every mechanism: the worktrees parent's folder is root's (a planted symlink otherwise redirects root's mount);
a caller inside a user namespace (a unit with `ReadOnlyPaths=` included) cannot clone, so its ZFS worktree is made
outside it, as the driver already makes run worktrees, or it takes the reflink fallback, which works there;
and a unit that should see clones mounted after it starts must not name the worktrees parent in `ReadOnlyPaths=`.

## Not researched yet

- containerd's ZFS snapshotter.
- Genesis's split (reflink the unchanged files, Git checks out the rest), timed as a whole.
- How the wrapper's controls get a real pool (a file-backed pool made once by root, a delegated dataset on the
  person's pool as these probes used, or a VM with ZFS).
- The text of mitosis-db's printed sudoers rule.
- A polkit rule for a root-installed template service with a fixed `ExecStart` (the user picks only the instance name);
  only transient units, where the caller supplies `What=` and `Options=`, were tried.
- A handler that checks every part of the mount path, or mounts through a file descriptor, before mounting.
- A handler that sets a clone's `mountpoint` to a path the person asked for, so `git worktree add <path>`
  could put a clone there; a sudoers rule cannot do this safely.
- What a reboot does to pending requests: the kernel's event queue starts empty, and `zfs mount -a` at boot
  mounts every clone whose `canmount` is on, flagged or not.

## Open choices for the owner

- The route: dataset clones with a reflink fallback, dataset clones alone, or reflinks alone.
- For dataset clones, the root mechanism that mounts and unmounts, from the ranked list in "Recommendation"
  (the owner leaned towards ZED on 2026-11-18, before these probes; not yet chosen).
- Driving ZFS through the `zfs` program or a crate.
- Moving the main checkout into a dataset of its own, and naming that dataset in sanoid's configuration.
