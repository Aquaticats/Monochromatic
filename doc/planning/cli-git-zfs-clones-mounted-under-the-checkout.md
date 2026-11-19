# ZFS clones mounted inside the checkout: what conflicts and what each conflict needs (2026-11-18)

**Non-final draft, written by an agent, unapproved.** Nothing here is a decision and nothing is complete. The place is
decided by the project: the checkout stays at `~/Monochromatic`, and every worktree, clones included, lives under
`.claude/worktrees/` inside it (`AGENTS.md` rule `WTU`). This page reports what that place costs on ZFS; it does not
argue for another place.

What it checks, in one line each: the seven items the project asked about, against OpenZFS at tag `zfs-2.4.4`, Linux
and Git source, and a throwaway guest where pools, clones and mounts were made.

The probes ran under one model session; this document was assembled and written under a later one from the probe's
logs, so any misreading of a log line is the writer's, not the probe's.

## What was checked and how

- OpenZFS at tag `zfs-2.4.4`, read at `~/temp/agent/rust-crate-for-openzfs-vet-2026-11-18/openzfs-at-zfs-2.4.4`
  (`git describe --tags --exact-match` answered `zfs-2.4.4`). The module running on this machine is `2.4.4-1`
  (`/sys/module/zfs/version`), so the tag is the right one to read.
- Linux v6.17 sources fetched whole and read: `fs/namespace.c`, `fs/namei.c`, `fs/open.c`,
  `include/uapi/linux/mount.h`, plus the installed host headers at `/usr/include/linux/{mount,openat2}.h`.
- Git v2.56.1 `builtin/worktree.c` and `builtin/clean.c` fetched and read; the guest ran Git 2.43.0, so every Git
  behaviour reported was measured on 2.43.0 and checked against 2.56.1 source.
- systemd v259 `systemd.mount(5)` read for the mount hierarchy's implicit dependencies.
- One throwaway guest, Ubuntu 24.04 (kernel `6.8.0-147-generic`), 3 vCPU, 3 GiB, a 1 GiB file-backed zpool, every
  probe inside `unshare --mount --propagation private`. OpenZFS 2.4.4 was **built from the tag inside the guest** and
  its `spl.ko` and `zfs.ko` loaded, because no mvm template has OpenZFS: `zfs version` then answered
  `zfs-2.4.4-1 / zfs-kmod-2.4.4-1`. The distribution's own 2.2.2 was used for one earlier control run.
- Tools in the guest: `zfsutils-linux`, `sanoid` 2.2.0, `ripgrep` 14.1.0, Git 2.43.0. The project's own measurements
  of sanoid were on 2.3.0, and the section order result reproduced here is the same, so the version difference is
  noted but not load bearing.
- On this machine, read-only only: `zfs list`, `zfs get`, `findmnt`, `stat`, `systemctl --user show`,
  `systemctl --user list-units`. No pool, dataset, snapshot, clone, mount, `zfs`, `zpool` or `sudo` was run here.
- Probe logs and the scripts, kept in `~/temp/agent/zfs-nested-checkout-mounts-2026-11-18-scratch/keep/`:
  `guest-exact-zfs.log` (the main run, cited by line throughout as `guest-exact-zfs.log:<line>`),
  `guest-exact-zfs-core.log` (the first, narrower run), `guest-packaged-zfs.log` (the 2.2.2 control),
  `exact-zfs-build.log`, `guest-console.log`.

The fixture the guest used is a stand-in for this repository's layout, not this repository: a zpool `checkoutprobe`
whose `home` dataset mounts at a fake `/home`, a checkout dataset mounted at `<fake>/account/Monochromatic` holding a
two-file Git repository with `/.claude/` ignored, and a sibling dataset
`home/git-worktree-clone-datasets` with `mountpoint=legacy` and `canmount=off` whose children are the clones. That
matches the layout facts on `doc/planning/cli-git-zfs-worktree.md` lines 270 to 281 and the dataset decisions in its
"Owner's answers of 2026-11-18".

## 1. A clone mounted inside the mounted tree of the dataset it was cloned from

**Conflict, in two of the three parts.** A snapshot and a new clone both behave exactly as the design needs; unmounting
and exporting do not fail or hang, but they refuse in a way the design must handle rather than inherit.

### What a snapshot of the checkout holds of `.claude/worktrees/`

Nothing of the mounted clone, and everything under the mountpoint. Measured: with a clone mounted at
`.claude/worktrees/registered` holding a file `clone-marker`, a snapshot of the checkout was mounted read-only and its
`.claude/worktrees` listed, and so was a clone made from that snapshot (`guest-exact-zfs.log:46` to `:50`):

```text
fallback/ordinary-marker
registered/.git
fallback/ordinary-marker
registered/.git
```

`clone-marker` was absent from both, by an assertion the run passed. So a mount below the mountpoint is invisible to
the snapshot, which is the property ZFS block sharing requires: the snapshot's block pointer for the mountpoint
directory is the underlying directory's block, not the clone's root.

The `registered/.git` line is the underlay, and it matters for item 7: it is Git's own pointer file, written by
`git worktree add` into the checkout dataset before the clone was mounted. The snapshot therefore contains whatever
the underlay folder holds, and for a registered slot that is a non-empty folder with a forward pointer into the main
repository.

Source: a snapshot is a copy of one dataset's block pointer at a transaction group, not a tree walk.
`dsl_dataset_snapshot_sync_impl` copies the dataset's block pointer into the snapshot
(`module/zfs/dsl_dataset.c:1725`, `:1783`) and hands the live dataset the new one (`:1882`); nothing consults mount
tables, so a nested mount is simply not part of the dataset. Linux's mount tree is separate from the VFS dentry tree
(`fs/namespace.c:820` to `:880`, `lookup_mnt` and `__lookup_mnt`), which is why the snapshot cannot see it.

### What a new clone of the checkout holds there

The same underlay bytes, and an empty directory where the mounted clone was. Measured in the clone-from-snapshot at
`guest-exact-zfs.log:49` to `:50`: `registered/.git` present, `clone-marker` absent. A wrapper cloning the checkout
into a new slot at `.claude/worktrees/<name>` therefore gets, at the clone's own root, the **checkout's** `.git`
administrative directory, because the clone is a copy of the checkout dataset root, not a copy of `.claude/worktrees`.

### Unmounting, exporting and shutting down while clones are mounted inside

No hang, and no partial unmount. Every refusal was immediate and named:

- `zfs unmount <checkout>` while a clone was mounted below it: `cannot unmount
  '<checkout>': pool or dataset is busy` (`guest-exact-zfs.log:106`), exit 1.
- `umount <checkout path>`: `target is busy`, `EBUSY` (`:108`).
- `zfs unmount -a`: failed on the checkout and on the `home` dataset above it (`:110` to `:112`).
- `zpool export` while a process's working directory sat inside a mounted clone: `cannot unmount '<clone>': pool or
  dataset is busy` (`:118`), exit 1.
- The same `zpool export` after that process left, with two nested mounts still present and idle: **succeeded**
  (`:120`), because export unmounts every ZFS entry in `/proc/self/mounts` deepest first
  (`lib/libzfs/libzfs_mount.c:1395` `zpool_disable_datasets`, sort at `:1466`, unmount loop at `:1478` to `:1483`,
  `mountpoint_compare` at `:1379` to `:1386`).

The kernel reason for the single-dataset refusal is that a mount with a mount below it cannot be detached by path
without a forced detach (`fs/namespace.c:1677`, `propagate_mount_busy`), and OpenZFS turns that into the message at
`lib/libzfs/libzfs_mount.c:617`.

Two consequences the design must own. First, an operator or a service that unmounts the checkout dataset alone will be
refused while any clone is mounted, and the refusal names the checkout, not the clone. Second, `zfs unmount -a` does
not unmount the legacy clones at all: it walks `/proc/self/mounts` and skips `mountpoint=legacy` and
`canmount=noauto` entries (`cmd/zfs/zfs_main.c:8013` to `:8025`), so nothing in OpenZFS's own shutdown path will
unmount them; the ZED handler is the only owner of those mounts.

### What the design needs

1. Treat "checkout unmount refused because a clone is mounted" as an expected state with its own report, naming the
   clone that holds it, not as an I/O failure.
2. Make the handler the owner of clone unmounts at shutdown, since neither `zfs unmount -a` nor a systemd mount unit
   knows a legacy clone. A shutdown that leaves clones mounted is not corrupting, but the next boot's
   `zfs mount -a` then sees them only through the handler, not through ZFS (item 2).
3. Keep `export`/`import` in the acceptance tests with an idle clone inside the checkout, because it succeeds and the
   design must not add a check that forbids it.
4. Record that a snapshot of the checkout never contains a live clone's bytes, so "worktrees are snapshotted" means
   the clone's **own** dataset is snapshotted, not that the checkout's snapshots hold the worktree.

## 2. Boot and mount order

**No conflict with the decided dataset ancestry; one real conflict with path-based mounting, and one with an early
mount.**

### Where the clones' parent dataset can sit

Anywhere in dataset terms, as long as it is not a child of the checkout dataset. The probe used a sibling of the
checkout, `checkoutprobe/home/git-worktree-clone-datasets`, and clones inherited `mountpoint=legacy` and
`canmount=off` from it (`guest-exact-zfs.log:18` to `:25`):

```text
.../git-worktree-clone-datasets          mountpoint  legacy  local
.../git-worktree-clone-datasets          canmount    off     local
.../git-worktree-clone-datasets/registered  mountpoint  legacy  inherited from .../git-worktree-clone-datasets
.../git-worktree-clone-datasets/registered  canmount    on      default
```

That is the property set the design assumed: the clone is invisible to every automatic mount because its mountpoint is
`legacy`, and it can still be mounted explicitly at a path.

Source for the three places `legacy` is skipped: `lib/libzfs/libzfs_dataset.c:2678` to `:2688` (inherited legacy keeps
a dataset unmountable in the automatic sense), `lib/libzfs/libzfs_mount.c:295` to `:297`, and
`etc/systemd/system-generators/zfs-mount-generator.c:430` to `:432`, which ends the unit's generation for `legacy` and
`none`. Measured in the guest: running the generator over a cache file listing the pool produced `.mount` units for
the checkout and for a clone with an ordinary mountpoint, and **none** for the legacy clones
(`guest-exact-zfs.log:141` to `:153`).

### How `zfs mount -a` and the generator order mounts

Both order by **mount path**, not by dataset ancestry.

- `zfs mount -a` sorts the datasets by mountpoint string (`lib/libzfs/libzfs_mount.c:1016` `mountpoint_cmp`,
  called at `:1277`) and dispatches one task per top-level chain (`non_descendant_idx` at `:1095`, used at `:1298`),
  so a dataset whose mountpoint is inside another's is mounted after it in the same call.
- Proof that the ordering key is the path and not the dataset tree: a clone of the checkout, dataset-wise a child of
  the clone-parent and not of the checkout, with `mountpoint=<checkout>/.claude/worktrees/auto`, was mounted by
  `zfs mount -a` at mount ID 354 with parent ID 353, and 353 is the checkout's own mount
  (`guest-exact-zfs.log:133` to `:139`). The checkout was mounted in the same call; the clone came after it.
- The systemd generator gives a mount unit `Before=zfs-mount.service` and the path hierarchy gives the implicit
  requirement and ordering edges (`etc/systemd/system-generators/zfs-mount-generator.c:611`; `systemd.mount(5)`
  "Implicit Dependencies"), so a clone with an ordinary mountpoint nested under the checkout's gets a unit that
  requires the checkout's unit. A legacy clone gets no unit at all, so systemd orders nothing for it.

### A clone mounted before the checkout is mounted

It ends up hidden and it blocks export. Measured: a clone was mounted at `<checkout>/.claude/worktrees/early`
**before** the checkout dataset was mounted; it sat at mount ID 353 with parent 352 (the `home` dataset). Mounting the
checkout then created mount ID 354, also a child of 352, whose path prefix covers 353's path. From then on
(`guest-exact-zfs.log:122` to `:131`):

- the clone's contents were no longer visible at its path, by an assertion the run passed;
- `findmnt --target` still reported the clone at that path, because the path resolves to the checkout's mount first;
- `zpool export` then failed with `cannot unmount '...': no such pool or dataset` (`:131`), because the clone's
  mountpoint path is no longer reachable as a path, so the export path cannot unmount it by name.

That is the failure mode the design's ordering rule exists to prevent, and it is worse than a spurious mount: the
hidden mount is unreachable by path and therefore not unmountable by the ordinary tools.

### How inherited `mountpoint=legacy` behaves for clones

Inheritance works and is the reason no clone gets an automatic mount, but it does not stop an explicit
`zfs mount <clone>` at the path the parent's mountpoint would imply, and it does not hand the clone a path at all:
`zfs mount <clone>` with a `legacy` mountpoint refuses (`cmd/zfs/zfs_main.c:7863` to `:7867`, "legacy mountpoint,
use umount(8)"), while `mount -t zfs` and a descriptor attach work. The clone also does not inherit a mount **path**,
which is why the handler must derive the path from the slot name and verify it, item 3.

### What the design needs

1. A stated order: the checkout's dataset must be mounted before the handler mounts any clone, and the handler must
   refuse to mount while the checkout's mount is absent or not the expected dataset. The probe shows the cost of
   getting this wrong is a hidden, unmountable clone, not a slow boot.
2. A boot-time check, because the handler starts independently of `zfs-mount.service`: after a pool import, a clone
   can be mounted in the window before the checkout's own unit runs. The handler's reconcile pass must therefore
   check the checkout mount's identity (`st_dev` from `statfs`, or the mount's source in `/proc/self/mountinfo`)
   before acting, and a startup pass after pool import must run after the checkout is mounted, not merely after the
   pool is imported.
3. No dependency on `zfs mount -a` for clones: with `legacy` they are outside it, which is the decided behaviour.
4. A record that `zfs mount -a` would mount a clone that ever gets an ordinary mountpoint, even one the account
   cannot set (the delegated user's `-o mountpoint=` was refused in the project's earlier probe), so any future
   change that gives a clone a real mountpoint re-opens the early-mount hazard.

## 3. The planted-link finding

**Conflict, and the largest one.** `~/Monochromatic/.claude` is the account's. Root owns `.claude/worktrees` in the
decided design, which stops a link planted **inside** it, but the account can rename `.claude` and put a link where
`.claude` or `.claude/worktrees` was. Measured on this machine: `/home/user/Monochromatic/.claude` and
`/home/user/Monochromatic/.claude/worktrees` are both owned by the account today, mode 0755, and `.claude` is ignored
by Git (`.gitignore:192`, `git check-ignore --verbose` names it), so the wrapper's clean check has nothing to say
about it.

The probe's positive control shows the danger is live on 2.4.4, not a reading of older code: after the account renamed
`.claude` and planted a link to a decoy folder, root's `mount -t zfs` mounted the clone **on the decoy**
(`guest-exact-zfs.log:210`), because `zfs_mount_at` only `lstat`s the mountpoint and, with `overlay` on, skips the
empty-folder check before `mount(2)`, which follows links (`lib/libzfs/libzfs_mount.c:506` to `:544`).

What follows is the requirement list: the properties a root mount at `<checkout>/.claude/worktrees/<name>` must
check, and what the kernel did in each case. No commands or code here on purpose; the system calls and flags are named.

### The mount must not resolve a path that the account can change

Requirement: root opens the checkout once, then opens each remaining part of the target relative to the previous part,
with `openat2` and `RESOLVE_NO_SYMLINKS | RESOLVE_BENEATH | RESOLVE_NO_XDEV | RESOLVE_NO_MAGICLINKS`. `RESOLVE_BENEATH`
rejects any component that leaves the opened parent; `RESOLVE_NO_XDEV` rejects a path that crosses any mount, which is
what removes a bind mount planted at the slot; `RESOLVE_NO_MAGICLINKS` removes `/proc`-style links. The flags exist in
`include/uapi/linux/openat2.h:26` to `:38` and are turned into `LOOKUP_BENEATH` and `LOOKUP_NO_SYMLINKS` in
`fs/open.c:1345` to `:1347`.

Measured refusals, each with no mount anywhere (`guest-exact-zfs.log:209` to `:221`):

- a link at the place of `.claude` fails the open with `ELOOP` (`:211`);
- a link at the place of `.claude/worktrees` fails the open with `ELOOP` (`:212`);
- an account-created link directly inside the root-owned `.claude/worktrees` is refused by the filesystem with
  `EPERM` (`:213` to `:214`); note added 2026-11-18 by the coordinator: the project has since decided the account
  keeps owning `.claude/worktrees` (`doc/planning/cli-git-zfs-worktree.md`, "Owner's answers of 2026-11-18"), so this
  layer is absent and the `ELOOP` refusal of a link at `<name>` is what holds; every refusal in this list has to be
  measured again with the account owning `.claude/worktrees` and its slots;
- a link at the place of `<name>` itself fails the open with `ELOOP` (`:215`);
- a `..` component fails with `EXDEV` (`:221`);
- a `/proc` magic link fails with `ELOOP` (`:220`);
- a bind mount placed at `<name>`, which has the same inode and `st_dev` as the recorded folder, fails with `EXDEV`
  from `RESOLVE_NO_XDEV` (`:219`). Without that flag this case would pass every identity check.

Requirement: refuse on the first such failure and report which component failed. A link at `.claude` is a state the
account can create at any time and does not mean the machine is compromised; it means the mount was refused.

### The mount must check the target folder's recorded identity, not just its kind

Requirement: at setup, root records for `.claude/worktrees` and for each slot the values from `fstat` of the opened
descriptor: `st_dev`, `st_ino`, `st_uid`, and the permission bits with ACLs and any default ACL taken into account.
At mount time, after opening with the flags above, it refuses unless every recorded value matches.

Measured refusals (`guest-exact-zfs.log:216` to `:218`):

- the account removes `<name>` and creates a new folder with the same owner and mode: refused by the inode identity,
  reported as an owner, mode or inode mismatch (`:216`);
- the slot's owner is changed to the account: refused (`:217`);
- the slot's mode is widened to 0777: refused (`:218`).

Requirement: the recorded identity is per slot and is rewritten only by root when it creates a slot; a
`doctor` finding names which value changed. Mode and owner alone are not enough, which the inode case shows.

### The mount must be attached to the opened folder, not to a path

Requirement: build the filesystem with `fsopen("zfs")`, `fsconfig(FSCONFIG_SET_STRING, "source", <dataset>)`,
`fsconfig(FSCONFIG_CMD_CREATE)` and `fsmount` with `MOUNT_ATTR_NOSUID | MOUNT_ATTR_NODEV`, then attach it with
`move_mount(fsfd, "", targetfd, "", MOVE_MOUNT_F_EMPTY_PATH | MOVE_MOUNT_T_EMPTY_PATH)`. Then re-check the slot with
`fstatat(parentfd, name, AT_SYMLINK_NOFOLLOW)` and compare the mount identity from the mount table, and unmount if the
two disagree. ZFS implements the new mount API at 2.4.4: `zpl_get_tree` holds `fc->source` as the dataset name through
`dmu_objset_hold` and never consults a `mountpoint` property (`module/os/linux/zfs/zpl_super.c:850` to `:870`).

Measured, in the guest (`guest-exact-zfs.log:222` to `:229`):

- the descriptor attach succeeded and the mount appeared with `rw,nosuid,nodev` (`:222`);
- `nosuid,nodev` are enforced, not decorative: a character device created inside the clone could not be opened even
  by root (`EACCES`, `:223`). This matters because the dataset in the probe carries `setuid=on devices=on`, and the
  earlier planning probe showed `zfs mount -o suid,dev` can turn the dataset's own flags back on
  (`doc/planning/cli-git-zfs-worktree.md:115` to `:117`), so the handler must pass the flags itself;
- one deliberately won race: with the slot descriptor open, the account renamed `.claude` and planted a link, and the
  attach still landed on the **old inode**, which the rename had moved to `.claude-moved/worktrees/security-slot`,
  never on the decoy (`:227` to `:229`). The residual state is a mount at a place the account chose by renaming a
  directory it owns; it cannot be `/etc` or any other path the account does not control, and the post-attach check
  must find it and unmount it.

Requirement: the design says what happens in that window. Either the post-attach identity check refuses and unmounts
by the same descriptor route, or the mount is left and reported with the two paths. The probe shows the check must
compare the **attached mount** identity, because path comparison at that moment is exactly what the rename defeats.

Requirement: mounting must be the only privileged step taken with these descriptors, and the handler must not accept
a target path, mount options or a dataset name from the account. The account's own delegated clone is the only input,
and the slot name's grammar must bound it.

### What unmount needs, and where the probe was incomplete

Requirement: unmount must go through the same opened descriptor, because `umount2` resolves a path
(`fs/namespace.c:2112`, `user_path_at`) and so re-opens the planted-link problem. Moving the mount back to a
root-owned staging folder by descriptor and unmounting there is the sound shape.

Measured limits (`guest-exact-zfs.log:224` to `:230`):

- an unmount attempted through the detached mount's own descriptor path was refused `EBUSY` and the mount stayed
  (`:225` to `:226`), so that exact form is not the answer;
- moving a mount to staging by descriptor works when the checkout's mount is private (`:227`, "moved verified mount
  to private staging");
- when the checkout's mount is made **shared**, the same move fails with `EINVAL` (`:230`), from the kernel's refusal
  to move a mount whose parent is shared (`fs/namespace.c:3611`, `IS_MNT_SHARED(parent)`).

Requirement: the design must state that the checkout mount and its ancestors stay private, and that the unmount path
is tested against a shared ancestor as a refusal rather than discovered in production. The project should also decide
what happens when some other tool makes `/` or `~` shared, which is a real state on machines with certain container
runtimes.

### What remains open after the probe

The probe covers links at every component, an inode replacement with matching owner and mode, owner and mode drift, a
bind mount, a magic link, `..`, and one rename window. It does **not** cover: an account process in a tight
rename/create loop during many mounts, ACL grants and default ACLs on the slot, the mount table check itself, a
planted link inside `.claude/worktrees` achieved through a route other than `ln -s` in the slot's parent, and an
unmount path that is both descriptor-based and busy. Those are the gates a builder must meet, and the design must name
them as gates, not as done.

## 4. Snapshots and sanoid

**A conflict that is already solved by configuration, plus one cost the design must accept.**

### Where the two datasets go

Both the checkout dataset and the clone-parent dataset must be children of the recursive snapshot root, and the
clone-parent must have `mountpoint=legacy`, so its dataset ancestry is snapped and its path ancestry is empty.
Measured: with the clone parent a sibling of the checkout under the same `home` dataset, one recursive snapshot of the
`home` dataset covered both and every clone (`guest-exact-zfs.log:61` to `:65`):

```text
checkoutprobe/home@home-recursive
checkoutprobe/home/git-checkout@home-recursive
checkoutprobe/home/git-worktree-clone-datasets@home-recursive
checkoutprobe/home/git-worktree-clone-datasets/from-with-mount@home-recursive
checkoutprobe/home/git-worktree-clone-datasets/registered@home-recursive
```

So the recursive home snapshot covers each dataset once, and no dataset is snapshotted twice. The clone parent having
`legacy` and `canmount=off` is what makes this safe: an ordinary mountpoint on it would put a mount in the middle of
the checkout's path tree, which item 1 and item 2 show is the dangerous shape.

### Sanoid's section order, reproduced

Measured in the guest with `--readonly --take-snapshots`, probe configs whose recursive parent is the `home` dataset
and whose excluded child is one clone (`guest-exact-zfs.log:155` to `:208`):

- child section written **after** its parent, `recursive = yes`: the excluded clone was never named, and every other
  dataset was (`:156` to `:177`);
- child section written **before** its parent: the excluded clone was snapshotted anyway (`:180` to `:185`), because
  sanoid reads sections in file order (`/usr/bin/sanoid:936`, `tie my %ini`) and copies a parent's settings onto
  children without checking for a child section (`/usr/bin/sanoid:1110`);
- `recursive = zfs`: one recursive snapshot named on the parent only, and a child cannot be excluded (`:205` to
  `:208`).

This reproduces what `doc/planning/cli-git-zfs-worktree.md` lines 362 to 385 records, on a second sanoid version, and
the same conclusion follows: a blocklist works only with `recursive = yes` and every excluded section written below
its parent's.

### What a living clone pins

Every clone pins its origin snapshot, so the checkout's snapshot for one worktree stays for as long as that worktree
lives, pinning every block the checkout changed since. Measured (`guest-exact-zfs.log:53` to `:60`, `:233` to `:243`):

- `zfs destroy <checkout>@origin` refused: `snapshot has dependent clones`, with the dependent datasets listed
  (`:53` to `:56`, `:238` to `:242`);
- the same refusal for a second checkout snapshot with exactly one clone (`:57` to `:60`);
- after the last dependent clone was destroyed, the destroy succeeded (`:243`);
- the check is not name-based but dependency-based: a fresh snapshot with no clone was destroyed with no message at
  all (`:60`).

Source: the clone records its origin's object (`module/zfs/dsl_dataset.c:1216`, `:1265` to `:1274`), and destruction
refuses while the origin's clone list is non-empty (`module/zfs/dsl_destroy.c:212` to `:222`). Blocks are shared, so a
pinned origin means every changed block of the checkout stays allocated, which is the cost of "worktrees are not
necessarily throwaway".

The clone's own snapshots die with it, and they are a real obstacle to removal: destroying a clone with a snapshot
taken by the recursive pass was refused with `filesystem has children` and a list naming
`.../registered@home-recursive` (`guest-exact-zfs.log:233` to `:236`).

### What the design needs

1. Both datasets below the recursive sanoid entry, and the clone parent `mountpoint=legacy`; `doctor` compares the
   measured dataset tree with that shape.
2. The blocklist sections below their recursive parent, and a `doctor` check for the order, because a wrongly ordered
   config silently snapshots the excluded dataset and the earlier probe shows the failure only appears in a run.
3. An accepted cost: a living clone pins one snapshot of the checkout. The design should make the origin snapshot's
   name and identity durable in the per-worktree record, so `doctor` can report which checkout snapshot is held by
   which worktree and how many blocks it pins.
4. Removal destroys the clone's snapshots explicitly before the clone, never with a blanket `zfs destroy -r` of a
   parent, and the dependent-clone refusal is the check that an origin is safe to delete, not a name test. That
   matches the withdrawal of the recursive destroy in the first design draft.

## 5. Git and walkers

**No conflict for the read-only walkers; a real conflict for `git clean`, and an unmeasured backup job.**

### Git's own view

The directory is ignored here: `.gitignore:192` is `/.claude/`, and `git check-ignore --verbose -- .claude/worktrees`
names that line. In the guest's fixture, with the same ignore, and a mounted clone holding a unique file
(`guest-exact-zfs.log:71` to `:74`):

- `git check-ignore -v` named the ignore and the file inside the clone (`:71`);
- `git status --short --untracked-files=all` printed nothing; `git status --short --ignored --untracked-files=all`
  printed three entries, one per ignored direct child (`:72` to `:74`).

So `status` does not descend into the clone, and the ignored listing names the directory, not its contents. That is
what the wrapper's clean check needs.

### `git clean`

This is the conflict. Measured in the guest (`guest-exact-zfs.log:79` to `:104`):

- `git clean -n -d -X` with one `--force` printed `Would skip repository .claude/worktrees/walker` and
  `Would skip repository .claude/worktrees/registered`, and `Would remove .claude/worktrees/fallback`: the nested
  `.git` protection stops it at a mounted clone;
- with `--force --force`, it printed `Would remove .claude/` (`:83`), which means it would remove the ignored tree that
  contains the mountpoints;
- `git clean -n -d -x` with one force printed nothing, and with two forces it printed
  `Would remove .claude/worktrees/walker/`, then `Would remove .claude/worktrees/walker` (`:88` to `:92`): it descends
  into the mounted clone and would remove its contents;
- the decisive case: `git clean -d --force -X -- .claude/worktrees/walker/only-on-clone/walk-marker`, a pathspec naming
  one file, removed `walker/only-on-clone`, `walker/.gitignore`, `walker/.claude`, `walker/.old-admin-for-probe`,
  `walker/tracked`, and `fallback`, and failed on the mountpoint with `Device or resource busy`
  (`:93` to `:104`). The pathspec inside an ignored ancestor did not limit the removal to the named file. (Note added
  2026-11-18 by the coordinator: this ran after the probe moved the clone's copied `.git` aside, `user mv "$B/.git"
  "$B/.old-admin-for-probe"` in `keep/probe.ts`, so the single-force nested-repository protection no longer applied
  to `walker`; it is the state of a mounted clone with no `.git` at its root, step 4 of item 7's sequence.)

Source: `git clean` walks with `DIR_SHOW_IGNORED` and `DIR_SHOW_IGNORED_TOO` (`builtin/clean.c:971` to `:997`) and
removes with `REMOVE_DIR_KEEP_NESTED_GIT` (`:927`), which is the single-force protection; the second force clears it
and lets the walk descend (`:176`). The pathspec is applied to what the walk yields, and because the ignored ancestor
is yielded as a unit, a pathspec naming a descendant does not narrow the removal.

What the design needs: the wrapper's clean proof must never hand `git clean` a path under `.claude/worktrees`, and
must treat "the ignored tree is a mountpoint or holds one" as a refusal rather than as something to force. The safest
shape is to exclude `.claude` from the clean proof entirely and to prove the checkout clean with a pathspec that
starts at the repository root and excludes the ignored tree, or to use the wrapper's own walker, which stops at mount
boundaries (the Btrfs route already walks boundaries; see `package/git-policy/cli/SPEC.md:3468`).

### The other walkers

Measured (`guest-exact-zfs.log:74` to `:78`):

- `rg --files <checkout>` did not list the clone's marker, because the tree is ignored;
- `rg --files --hidden --no-ignore <checkout>` **did** list it: a walker told to ignore nothing crosses into the
  clone;
- `rg --files --hidden --no-ignore --one-file-system` did not list it;
- `find <checkout> -name <marker>` found it, `find <checkout> -xdev -name <marker>` did not;
- `du -a` found it, `du -a -x` did not.

So the rule is the flag, not the tool: every walker crosses a mounted clone unless told not to cross a filesystem
boundary, and a ZFS mount is a different `st_dev` even inside one pool. The design should not depend on walkers being
polite; it should assume any walk without a boundary flag will descend, and it should state that `rg` with
`--no-ignore` and `find`/`du` without `-xdev` are the ones to expect in scripts around this repository.

### The account's backup job

Found on this machine, not run: the user units `kopia-snapshot-all.service` and `kopia-snapshot-all.timer`
(`~/.config/systemd/user/`), `ExecStart=.../kopia snapshot create --all`, hourly. Its `ExecStart` path
`/home/user/.local/share/monochromatic-dev/toolchain-global/exposure/kopia` does not exist here, and the service is
inactive, so the job cannot run on this machine as configured. The repository is excluded from kopia by a policy
recorded for another host: `kopia policy set user@bazzite:/home/user/Monochromatic --manual '--add-ignore=*'`
(`doc/handover/questions-for-the-owner.md:934` to `:936`), and this machine's host name is `cachyos-pc1`, so whether
this host carries the same policy is not shown by anything I read. The repository's `.kopiaignore` lists
`node_modules`, `dist` and file suffixes, not `.claude`.

What the design needs: a `doctor` line that says whether a backup job can see the checkout at all, because if one
does and it does not stop at filesystem boundaries, a mounted clone is backed up twice: once as the clone's own
snapshot target and once as a path inside the checkout. The probing of kopia's boundary behaviour is left undone, and
listing it here is not a finding that it is broken.

## 6. Removal

**A conflict in three places, all fixable in the removal sequence.**

### Busy mounts

Measured (`guest-exact-zfs.log:105` to `:121`): with a process whose working directory was inside a mounted clone,
`umount <clone>` failed `target is busy` (`:116` to `:117`) and `zpool export` failed on that clone (`:118`). Nothing
hung, and no partial unmount was left. A clone whose slots are held by an agent process cannot be unmounted, and
therefore cannot be destroyed, without either the holder leaving or a forced detach the design should not use.

### The mountpoint folder root leaves behind

Measured: after the clone was destroyed, `<checkout>/.claude/worktrees/registered` remained, an empty directory owned
by the account (`guest-exact-zfs.log:237`). Source: `remove_mountpoint` only removes a directory whose mountpoint is
the default or inherited, and it gives up early when the dataset is not mountable in the automatic sense
(`lib/libzfs/libzfs_mount.c:906` to `:925`); a `legacy` clone therefore never has its folder removed.

In the decided place that folder is inside the checkout dataset, so it is snapshotted, it accumulates, and it is
account-owned. The design must remove it deliberately, as the account, after proving it is empty and still the same
inode that was recorded, and it must do so after the Git registration is settled, not before: the folder without a
pointer file is exactly what `git worktree prune` deletes (`guest-exact-zfs.log:42` and `:44`).

### What the nested place adds beyond the withdrawn anchor

The first design draft put the clone under a root-owned anchor outside the checkout
(`doc/planning/cli-git-zfs-worktree-design-proposal.md`, branch
`docs/planning-cli-git-zfs-worktree-design-proposal`, withdrawn). The nested place adds four obligations the anchor
did not have:

1. the underlay folder is inside the checkout, so it is account-owned and snapshotted, and removal must clear it as
   the account rather than as root;
2. the Git registration lives in the checkout's `.git`, so the unmount and the registration's removal are two changes
   to two different datasets, and a crash between them leaves a registered path whose dataset is gone;
3. the checkout's snapshots hold the underlay folder, so a removed worktree's empty folder survives in the checkout's
   snapshot history for as long as sanoid keeps those snapshots, which is a fact `doctor` should be able to report;
4. the clone's own snapshots are removed with it, which the measured `filesystem has children` refusal shows must be
   explicit (`guest-exact-zfs.log:233` to `:236`), and the origin may only be destroyed once no clone depends on it,
   where the kernel's refusal is the check (`:237` to `:243`).

### What the design needs

1. An ordered removal: request the unmount, wait for the handler's matching result, verify the mount is gone and that
   the recorded dataset GUID and slot identity still match, destroy the clone's snapshots by name, destroy the clone,
   destroy the recorded origin only after the dependent-clone refusal no longer names it, then settle the Git
   registration, then remove the empty folder.
2. A named diagnostic for each blocker: a holder (`EBUSY`), a snapshot that would be lost, a nested dataset, a
   dependent clone, an identity mismatch.
3. A resume path for a killed removal that reads state rather than names: which of the four steps completed, from the
   mount table, the dataset properties and the registration.

## 7. Becoming a worktree

**A real conflict with the shape the first draft used, and a sequence that works, proven in the guest.**

### The Git refusal, restated correctly

The cross-vendor review's finding H1 (`review.md`, section H1) measured that Git refuses a non-empty destination. The
probe confirms it and sharpens it: Git **accepts an existing empty directory** and refuses only a non-empty one. In
the guest (`guest-exact-zfs.log:16` to `:28`):

- `git worktree add --no-checkout --detach <slot> HEAD` into an empty directory that already existed succeeded
  (`:16` to `:17`);
- the same command at the same path after the clone was mounted there failed with
  `fatal: '<path>' already exists` (`:27`).

Source: `builtin/worktree.c:322`, `if (file_exists(path) && !is_empty_dir(path)) die(_("'%s' already exists"), path)`.
So H1's conclusion holds and its sentence "Git refuses a non-empty folder" is the correct one; "Git refuses an
existing folder" is not.

A second fact the clone makes plain: a clone of the checkout carries the **checkout's** `.git` administrative
directory at its root, so the slot after mounting is not merely non-empty but holds a whole repository's metadata.
The guest's sequence moved that copied directory aside before putting Git's pointer in place, and the result reads
back as the intended worktree (`guest-exact-zfs.log:29` to `:31`); that move is mandatory, not cosmetic: leaving it
would make `git status` inside the worktree read the wrong repository.

### A sequence that ends with the worktree's root at exactly `.claude/worktrees/<name>`

Proven in the guest, in this order (`guest-exact-zfs.log:16` to `:50`):

1. `mkdir <slot>` as the account, inside `.claude/worktrees` (correction of 2026-11-18 by the coordinator: in this
   part of the probe `.claude/worktrees` was the account's, made by `user mkdir` after `chown` of the checkout,
   `keep/probe.ts:58` and `:59`; it became root-owned only for the refusal cases of item 3,
   `keep/guest-extra-commands.txt:25` and `:26`, so registration and the refusals were never shown together);
2. `git worktree add --no-checkout --detach <slot> HEAD`, which registers the worktree and writes Git's pointer file
   **into the checkout dataset**, so the registration is durable before any clone exists;
3. snapshot the checkout, then clone it to the clone parent's `<name>`;
4. mount the clone over `<slot>` with the descriptor route of item 3, after the identity checks;
5. inside the mounted clone, move the copied `.git` aside and install the known pointer file, then populate the index
   from `HEAD`;
6. verify from inside: `rev-parse --show-toplevel` answered the slot path, `--git-common-dir` answered the checkout's
   `.git`, and `status --porcelain --untracked-files=no` produced zero bytes (`:29` to `:31`).

The root is the slot itself, so each worktree keeps the short path the project decided. No exchange and no staging
subdirectory are needed, which removes finding H1's second and third problems with the first draft.

### What the registration does while the clone is unmounted

Measured (`guest-exact-zfs.log:42` and `:44`): with the underlay pointer file present after unmount,
`git worktree prune --dry-run --verbose` printed nothing; with that file moved away, it printed
`Removing worktrees/registered: gitdir file points to non-existent location`.

The precise claim this supports is the one that matters: the registration survives unmount **because Git wrote its
pointer into the checkout dataset**, not because anything in the clone does. The probe compared the underlay pointer
byte for byte after unmount and found it unchanged, which is a statement about the underlay only; it proves nothing
about the clone's own copy. A design that relies on the clone's copy for durability would be relying on unproven
behaviour.

Source: `builtin/worktree.c:229` `should_prune_worktree`, and the pointer is written by
`write_worktree_linking_files` (`:540`).

So the two-layer pointer is not an optimisation, it is what keeps the registration alive across an unmount, which
happens at every boot with `legacy` clones.

### What each step leaves behind if it is killed

From the states the guest reached, and from what each step writes. These are sequences, not kill tests: no probe
killed a process between steps, and that is listed as unshown.

- killed after step 1: an empty registered directory in the checkout dataset, no registration, no dataset. A later run
  can reuse or remove it, and `git worktree prune` does not know it.
- killed after step 2: a registration, a pointer file, an empty slot, no clone and no snapshot. `prune` keeps it, and
  the wrapper's record has no dataset, so the removal path must treat "registered without a dataset" as a state.
- killed after step 3, before mounting: a clone dataset with a `legacy` mountpoint, an origin snapshot pinning the
  checkout's blocks, and a registration. Nothing is visible at the path; `zfs list` and the record are the only places
  the clone appears. This is the state the first draft's step 4 covered, and the origin can be deleted by name because
  no clone depends on it yet.
- killed after step 4, before step 5: a mounted clone whose root holds the checkout's copied `.git`, and a pointer
  file in the underlay that nothing visible uses. `git status` inside the slot reads the copied repository, which is
  the worst state of the five: it looks like a worktree and is not one. A resume must detect it by comparing the
  visible `.git` with the recorded pointer bytes.
- killed after step 5: a usable worktree. What remains is bookkeeping: the record's dataset and origin identities.

### What the design needs

1. Register first, mount second, and write the pointer inside the clone last; the order is what makes every
   intermediate state recoverable.
2. A stated rule that the copied `.git` inside a clone is discarded before any Git command runs inside the mounted
   clone, and that the wrapper never runs Git against it.
3. The per-worktree record must carry the origin snapshot name, the clone dataset GUID, the slot's recorded identity
   and the pointer bytes, so a resume can tell the five states apart without guessing from names.
4. An acceptance test for a killed add at each of the five points, which this probe did not do.
5. A decision on `git worktree move`: it failed in the guest with `Device or resource busy` on a mounted clone
   (`guest-exact-zfs.log:40`), so a managed clone's worktree cannot be moved, and the design should say so and name the
   message rather than forward `move` and let it fail.

## What I could not show and why

- **A real shutdown and reboot with clones mounted.** The guest was not rebooted, so the claim that systemd's shutdown
  unmounts nested legacy mounts in a safe order is reasoning from `systemd.mount(5)` and the kernel's mount tree, not
  a measurement. Related: whether a `zfs mount -a` at boot can race the handler was reasoned from the units, not
  observed at boot.
- **Kill tests at each step of the add sequence.** Item 7's five states are the states the guest demonstrated, plus
  what each step writes; no process was killed between them.
- **A tight rename/create loop during many mounts**, ACL and default-ACL variants on the slot, and the post-attach
  mount-identity check itself. Item 3's residual risk is real and unmeasured beyond one deterministic rename.
- **An unmount that is both descriptor-based and busy.** The probe shows the path route fails `EBUSY` and the
  descriptor move works when private; the combination was not built.
- **kopia's boundary behaviour.** The job is configured for another host and its binary path is absent here, so it
  was read, not run, and the item 5 backup paragraph is a requirement on `doctor`, not a finding.
- **The exact sanoid version the project uses.** The guest had 2.2.0, the project measured 2.3.0; the section order
  and the `recursive = zfs` result matched, but a prune-path difference between the versions was not checked.
- **Git 2.56.1's runtime behaviour.** Every Git measurement is 2.43.0; 2.56.1 was read as source.
- **`zfs destroy` of a mounted clone, by root, on 2.4.4.** The probe's destroy cases were an unmounted clone with a
  snapshot, and origin snapshots. The planning page records the delegated user's refusal; root's behaviour on a
  mounted clone was not re-measured.
- **A second host's sanoid and ZED configuration**, and ZED itself: ZED was not started in the guest, so item 2's
  handler ordering is a requirement, not a measurement of the handler.
- **The mvm route.** `mvm create --image ubuntu zfs-nested-checkout-probe` failed its template build with
  `guest agent on template-setup did not respond within 120s`, and `mvm destroy` then found no domain to destroy. The
  guest was therefore made by hand with `qemu-system-x86_64` from the same Ubuntu cloud image, with a cloud-init seed
  carrying an SSH key, and it was disposed of by ending its process and deleting its disk. That is a deviation from
  the brief's stated route, and no mvm template was found that carries OpenZFS; the OpenZFS under test was built from
  the tag inside the guest.

## Where this record disagrees with the brief, or narrows it

- The brief asked for the conflict in item 1 as "whether unmounting, exporting or shutting down the checkout dataset
  fails or hangs". Measured: unmounting fails immediately, exporting fails only while something holds a mount, and
  both are correct behaviour rather than a defect. Recording them as "conflict" without that distinction would
  overstate the cost.
- Item 3 asked for "the properties the wrapper's mount must check". The probe showed one property list is not enough:
  the identity check must be re-run after the attach, because a rename wins a window the open-time check cannot close.
  This page states that as a requirement; it is not a claim that the first draft's check was sufficient.
- Item 7 asked to "prove one sequence" and "say what each leaves behind if killed at each step". One sequence is
  proven; the kill states are partly inferred, and the inference is labelled as such rather than presented as
  measured.
- The review file H1 was read whole. Its finding is accepted; its sentence about Git's refusal is corrected here to
  "refuses a non-empty folder", because the probe registered an existing empty one.
