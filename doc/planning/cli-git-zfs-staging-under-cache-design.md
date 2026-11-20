# Staging under `~/.cache` and the wrapper's cleanup

Non-final, unapproved design proposal by an agent. It is not approved by the project owner and nothing in it is
final. It designs the open item "Staging under `~/.cache` and the wrapper's cleanup" of the accepted ZFS worktree
design, and it makes no decision that design did not make. It ran no ZFS command that changes anything, started no
VM and no container, touched no pool, and edited no other file.

Every kernel fact here is read at upstream Linux `v6.8`, in the files listed under "Sources read", and every
OpenZFS fact is read at tag `zfs-2.4.4`. Nothing in this design was measured, on any host or any guest: every
measurement it owes stays owed, and no sentence here reports a run.

This revision answers two independent reviews, the gate review of tip `2258de74d8` and its re-check of tip
`3972d4bf90`. Both live outside this repository, under
`~/temp/agent/zfs-staging-under-cache-design-gate-review-2026-11-19/keep/` and
`~/temp/agent/zfs-staging-under-cache-design-revision-gate-recheck-2026-11-19/keep/`. Neither review's finding
numbers are used here: every reference is to a fact, an assumption, a limit or a measurement arm of this design.
Each finding is answered where it applies: the recovered folder descriptor and the bind's own descriptor in "The
unmount call from staging" and "The private bind", the unbind's copies in F22 and F30, the claim's place in "The
staging claim", the corrected mechanism in F21, and the questions in "Questions for the owner". The revision
history, with each commit's committer timestamp from `git log --format=%cI`, is under "Revisions".

The design is [the decision record][decision], the whole design is [the proposal][proposal], the evidence is
[the measurement record][measurements], and the build order is [the implementation plan][plan]. The plan lists the
item as step 9's blocker, "blocking for every unmount", and as writer design work
([plan][plan], "Open items and the step that needs each"), and step 9 records that it "also owes the interface that
unmounts a staged mount from the held descriptor". Steps 14, 16 and 17 wait on it as well
([plan][plan], step 9, "Open items"), and the plan's "What can start now" puts "step 9 and steps 14 to 23" behind
step 9's design.

## The open item and the answer it carries

The open item, quoted whole ([proposal][proposal], "Open items the decisions leave"):

> Staging under `~/.cache` and the wrapper's cleanup. The answer's words: "Under ~/.cache, and make the wrapper
> clean after itself." The folder's creation, the private bind that makes a staged mount safe to unmount there, and
> what the wrapper removes and when are undesigned, including after an interrupted or failed removal. This answer
> gives up the protection the draft proposed, a root-owned `0700` bind outside account-controlled trees, so the
> account can substitute or remove the staging target, and nothing replaces that protection
> ([Decided defaults][decided-defaults]). The interface that
> unmounts a staged mount from the held descriptor, after the move, is not designed either, and no unmount runs from
> staging until it is. Nothing orders the `zpcachyos/nosnap/user-cache` dataset's mount before the private bind into
> `~/.cache`: a bind made before that dataset mounts lands on the home underlay, and the dataset's later mount hides
> it. The cleanup also has to stop at any mount, because during an interrupted removal a clone is mounted inside the
> staging folder and a recursive delete there would delete the worktree's files.

The answer the item carries, in the owner's words: "Under ~/.cache, and make the wrapper clean after itself."

## Decisions of the accepted design that bound this proposal

- Staging location. Decided as "Account-owned place", then placed by the answer quoted under "The open item and the
  answer it carries", and recorded as a decided default
  ([proposal][proposal], "Decided defaults"): "Staging in an account-owned place, decided as "Account-owned place"
  against the ranking the question round carried, and then placed by a further answer: "Under ~/.cache, and make
  the wrapper clean after itself."" That entry also states the trade: "a folder under `~/.cache` is one the account
  can rename, replace or delete, so the account can substitute or remove the staging target", and "the descriptor
  identity checks still run before each move, and they catch a substitution made before them, not one made after."
- The staging step's unmount rule ([proposal][proposal], "Descriptor mount and unmount"): "Recheck the canonical
  lookup before moving." The move then goes to "the decided staging folder, verify the mount there, release
  descriptors that would hold it busy, then unmount without force or lazy detach, acting on the held descriptor and
  never on a path inside the folder the account can change".
- The plan forbids the same thing in one line ([plan][plan], "What the design forbids"): "No path-based mount or
  unmount on the account's tree", attributed there to "Descriptor mount and unmount".
- No unmount exists yet (same section): "The interface that unmount from staging needs, after the move and from
  that descriptor, has no design yet, and no unmount runs from staging until it exists."
- Staging reuse across sweeps (same section): "Reuse only a verified staging mount on subsequent sweeps;
  validate its identity and private propagation before each move."
- Staged state is not absence (same section): "Track and report staged identity and location across interruption:
  absence at the slot does not mean the mount is gone. Treat absence from the handler's namespace as local detach,
  not completed unmount."
- Private staging and private parent ([proposal][proposal], "Mounts held by other namespaces"): "This descriptor
  route requires a verified private immediate parent and verified private staging, not private ancestors of the
  checkout", and "Private staging does not eliminate the all-namespace check" ([proposal][proposal], "Project
  decisions", the checkout propagation bullet, lines 1245 to 1246 of that file).
- Staging counts as a live mount (same section): the design defines "Unmounted" as absence "in every mount
  namespace holding the clone's ZFS source, including binds and staging", and "a missing or mismatched unmount still
  blocks removal outright".
- Reattach when safe ([proposal][proposal], "Project decisions", "Busy unmount after a staging move"): the decision
  "moves the same mount back to the original slot only after revalidation, and retains it staged otherwise. Reattach
  restores the worktree's visibility but adds a second attach and a race"; "Both routes require staged-state
  tracking", and neither reattach nor crash recovery was measured.
- One exception to no implicit remount ([proposal][proposal], "Decided defaults"): "No implicit remount after a
  failure or restart: kept, with one exception ... The exception is the checked reattach of a removal that cancels
  itself without `--force`".
- Receipts and requests ([proposal][proposal], "Requests, results and ordering"): the wrapper asks for a fresh scan
  "by writing a new `v1:<nonce>:unmount` intent on the clone it removes, and the handler answers under that same
  nonce, so a receipt always names the scan that produced it and no receipt is reused for a later one." "No request
  supplies a path, dataset name or mount flags." Receipts are root-owned mode `0644` under
  `/run/git-worktree-clone-mount-results`, read-only to the wrapper. The handler "cannot take conflicting actions on
  the same mount", and a per-slot claim "only serializes cooperating wrapper operations".
- The boot unit does not establish staging ([proposal][proposal], "Boot and shutdown unit", "Open items the
  decisions leave"): the proposal lists the folder's creation, its private bind and its cleanup as undesigned, and
  its `ExecStart` step says so where it tells the unit what it does not do. Its stop pass "Unmount the verified
  managed mounts only; do not change durable intent", and it must "never force or detach" a busy clone.
- The boot and test controls the build owes ([proposal][proposal], "Descriptor safety and mount authority"): "Test
  staging setup at boot and refuse shared or replaced staging and checkout mounts until a private setup is
  verified".
- Removal's report for a staging failure ([proposal][proposal], "Waiting and refusal"): "For staging or rollback
  failure, give the staged location and administrator recovery instructions."
- `doctor` reports staging ([proposal][proposal], "What doctor checks and prints"): "Check the staging folder under
  `~/.cache` and the propagation of its live bind separately from the checkout", and `doctor` "does not start a
  service, create a test dataset, repair registrations or change configuration."
- The acceptance item the build owes ([proposal][proposal], "Decided behaviours with no test yet"): "After a
  completed removal, assert the staging folder holds no mount and no entry a recursive delete could reach a worktree
  through. After an interrupted one, assert cleanup deletes nothing under a mounted tree and that any retained
  staged mount is tracked and reported, since retaining it is the state the busy-unmount decision allows."
- Mount authority stays root's ([proposal][proposal], "Scoped delegation"): the account's `mount` grant "is an ioctl
  prerequisite, not Linux mount-namespace authority", and the design grants no `mountpoint` or `canmount`. Only the
  root handler makes mounts.
- What the handler may not do ([proposal][proposal], "ZED reconciliation and mount safety"): it "does not load code
  from the checkout, run Git, create or remove account slots, or widen sandbox access to `/dev/zfs`", and the handler
  uses verified slot descriptors "for mount management, not privileged writes into the account's tree"
  ([proposal][proposal], "Descriptor mount and unmount").
- Placements this proposal leaves alone ([proposal][proposal], "One-time move and checkout boot protection"):
  "Hardening those existing mount targets is outside this proposal", `~/.cache` among them.
- The forced arm can change the staging rule ([proposal][proposal], "Forced removal and deferred origins"): "A
  force-unmount result would change two statements in this file: the busy-teardown rule that reports which clone
  holds the mount and never forces or detaches it ... and the staging step's unmount without force or lazy detach."
- The handler's record of slot identity is root's ([proposal][proposal], "Descriptor mount and unmount"): records
  live in `/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities`.

## Facts

Each fact names its source and the lines read. Nothing here is a measurement taken on this host beyond the
read-only reads named.

- F1. The account's `~/.cache` is the mount `zpcachyos/nosnap/user-cache`, file system `zfs`, propagation `shared`,
  mode `0700` owned by the account. Read on this host 2026-11-19 with
  `findmnt --output TARGET,SOURCE,FSTYPE,PROPAGATION --target /home/user/.cache` and
  `stat --format '%n %i %U %a'`; the same mount row, with `shared`, is the fixture's host inventory in
  [the measurement record][measurements], "Fixture and evidence".
- F2. `zpcachyos/nosnap` is an unmounted container (`mountpoint` `none`, `canmount` `off`), and its child
  `user-cache` carries `com.sun:auto-snapshot=false`; the proposal's sanoid block excludes it
  ([proposal][proposal], "Snapshots and setup reporting"). Read on this host with `zfs list -H` and `zfs get -H`.
- F3. `~/.cache` is a recorded substitution exposure for a dataset mountpoint inside an account-owned home path, and
  hardening it is outside the proposal ([proposal][proposal], "One-time move and checkout boot protection").
- F4. The measured staging route was a bind of the staging directory made private: item 3's harness ran
  `mount --bind <stage> <stage>` then `mount --make-private <stage>` in the guest's boot namespace
  (`guest-probe.py`, item 3, in `~/temp/agent/zfs-measurements-owed-before-acceptance-2026-11-18/keep/`), and the
  item's log reads the staging row as `private` ([measurements][measurements], item 3). The measured order is the
  bind first and the propagation change second, because the bind is attached under a shared parent (F22).
- F5. `move_mount` refuses `EINVAL` when the moved mount's current parent is shared (Linux `v6.8`,
  `fs/namespace.c:3163` to `:3166`, "Don't move a mount residing in a shared parent"). The
  same refusal was measured with a shared checkout, and the move succeeded with only the checkout private
  ([measurements][measurements], item 3).
- F6. Attaching a mount into a shared destination propagates it to that peer group and marks the moved tree shared
  (Linux `v6.8`, `fs/namespace.c:2335` to `:2349`). A move into a plain directory under the `shared` `~/.cache` mount
  (F1) is therefore not the measured route and is not private.
- F7. `umount2` resolves its argument with `LOOKUP_MOUNTPOINT | LOOKUP_FOLLOW` (Linux `v6.8`,
  `fs/namespace.c:1904` to `:1919`), and `LOOKUP_MOUNTPOINT` means "follow mounts in the end"
  (`include/linux/namei.h:26`). After the walk the kernel steps into the mount at the final dentry
  (`fs/namei.c:2465` to `:2471`, called from `:2486` to `:2489`), and that step follows mount after mount to the
  topmost one in the caller's namespace (`fs/namei.c:1838` to `:1843`, `:1431` to `:1446`, `:1378` to `:1410`;
  `fs/namespace.c:722` to `:736`). A `/proc/self/fd/<n>` argument reaches the same step with the descriptor's own
  path, because that path is a magic link whose resolution jumps to the descriptor's `f_path`
  (`fs/proc/fd.c:171` to `:190`; `fs/proc/base.c:1741` to `:1757`; `fs/namei.c:995` to `:1013`). So
  `umount2("/proc/self/fd/<n>", 0)` unmounts the topmost mount at the location the descriptor names, and no path
  inside that location is resolved. The step follows the descriptor's own mount object (`fs/namespace.c:695` to
  `:736`), so the descriptor must be one the handler opened inside its own mount namespace; a descriptor another
  namespace's process opened resolves in that namespace and then fails `check_mnt` (F8).
- F8. The unmount then requires the resolved path to be the root of its own mount in the caller's namespace:
  `may_mount` gives `EPERM` without `CAP_SYS_ADMIN` (`fs/namespace.c:1875`, returning at `:1876`), `path_mounted`
  gives `EINVAL` (`:1877`, returning at `:1878`), and `check_mnt` gives `EINVAL` when the mount is not in the
  caller's namespace (`:1879`, returning at `:1880`), with `check_mnt` itself at `:858` to `:860`. A mount carrying
  the locked flag, `MNT_LOCKED`, gives `EINVAL` at `:1881` to `:1882`; that flag marks a copy a less privileged
  namespace was given, and no mount this design unmounts carries it.
- F9. A non-lazy unmount refuses `EBUSY` when the mount has child mounts or more than the reference count the call
  expects (Linux `v6.8`, `fs/namespace.c:1791` to `:1793`; `propagate_mount_busy`, `fs/pnode.c:407` to `:421`;
  `do_refcount_check`, `fs/pnode.c:352` to `:355`). A working directory or an open file raises that count; item 4
  measured the refusal with a cwd holder in the staged clone and success after the holder exited
  ([measurements][measurements], item 4).
- F10. An open descriptor holds the mount of the path it was opened through, because the file keeps the looked-up
  path (Linux `v6.8`, `fs/open.c:1084` to `:1087`, `:1175`), and what that mount is depends on when the descriptor
  was opened relative to the mounts that cover the path. Opened before the bind, the folder's descriptor names the
  `user-cache` mount, which pins neither the bind nor the clone, so both stay at the count F9 accepts. Opened after
  the bind, a descriptor resolved through the bind names the bind, and one resolved through the moved clone names the
  clone; each adds the reference F9 counts, so each must be closed before the mount it names is unmounted.
- F11. Deleting or renaming a name that is a mountpoint fails `EBUSY` only when the mountpoint is in the caller's own
  mount namespace: `__is_local_mountpoint` walks `current->nsproxy->mnt_ns` (Linux `v6.8`, `fs/namespace.c:753` to
  `:770`), and the `rmdir`, `unlink` and `rename` paths test it (`fs/namei.c:4201` to `:4202`, `:4330` to `:4331`,
  `:4862` to `:4863`). After a successful `rmdir` or a rename over the name, `vfs_rmdir` calls
  `detach_mounts(dentry)` (`fs/namei.c:4216`; rename-over target `:4894`), and `__detach_mounts` runs
  `umount_tree` on every mount on that dentry in every namespace (`fs/namespace.c:1815` to `:1842`). So a caller in
  a namespace that does not hold the mount removes the name, and every namespace loses the mounts on it. The handler
  runs in the host namespace ([proposal][proposal], "ZED reconciliation and mount safety"), and the account can
  create a mount namespace of its own (F24).
- F12. Mounting at a path hides whatever mount already holds that path. VM item 9 measured `zfs mount -a` attaching a
  path descendant to the home underlay and the later checkout attach hiding it while its mount ID stayed
  ([measurements][measurements], item 9; [proposal][proposal], "One-time move and checkout boot protection"). That is
  what a bind made before the `user-cache` mount would suffer.
- F13. A mounted ZFS dataset holds a long hold on it for the life of its superblock: `zfs_domount` calls
  `zfsvfs_create` (`module/os/linux/zfs/zfs_vfsops.c:1375`, `:651`), which owns the objset with
  `dmu_objset_own` (`:660`), and owning sets the long hold in `dsl_dataset_tryown`
  (`module/zfs/dsl_dataset.c:1049` to `:1062`). `zfs_umount` releases it through `dmu_objset_disown`
  (`zfs_vfsops.c:1526` to `:1553`) and `dsl_dataset_disown` (`dsl_dataset.c:1036` to `:1045`). Head destroy refuses
  `EBUSY` while the long hold stands (`module/zfs/dsl_destroy.c:766` to `:777`). The hold `zpl_get_tree` takes at
  `module/os/linux/zfs/zpl_super.c:868` is released in the same function at `:894` and is not the persistent one.
  The same item measured the refusal as `dataset is busy` while reader namespaces held the clone, and success after
  they stopped ([measurements][measurements], item 15).
- F14. `umount2` has no form that takes a descriptor; its argument is a path, and F7 is what a descriptor can be used
  for. The descriptor-taking mount calls are `open_tree` (with `OPEN_TREE_CLONE` and `AT_EMPTY_PATH`), `move_mount`
  (with `MOVE_MOUNT_F_EMPTY_PATH` and `MOVE_MOUNT_T_EMPTY_PATH`), `fsmount` and `mount_setattr` (with
  `AT_EMPTY_PATH`) (Linux `v6.8`, `fs/namespace.c:2692` to `:2741`, `:4072` to `:4115`, `:4647` to `:4693`,
  `:4426`; `include/uapi/linux/mount.h`). Every one of them resolves the descriptor's own path and none passes
  `LOOKUP_MOUNTPOINT`: `AT_EMPTY_PATH` adds `LOOKUP_EMPTY` and nothing else (`fs/namei.c:192` to `:200`, `:208` to
  `:214`), and an empty name keeps the descriptor's own mount and dentry (`:2362` to `:2363`, `:2418` to `:2441`), so
  a descriptor opened before a bind keeps the covered mount and these calls act on that mount, not on the mount
  stacked over it. `mount_setattr` and `mount` with `MS_PRIVATE` change propagation only for a path that is the root
  of its own mount: `do_mount_setattr` refuses `EINVAL` otherwise (`fs/namespace.c:4433` to `:4439`), and
  `do_change_type` likewise (`:2549` to `:2573`). The folder is not the root of the `user-cache` mount, so the
  bind's own descriptor is the only thing that can carry the propagation change, and F28 is how the design obtains
  it.
- F15. A receipt names the operation, the matching nonce, the outcome and the mount identity, and the request
  grammar carries no path, dataset name or mount flag ([proposal][proposal], "Requests, results and ordering"). The
  mount identity is the mount's own, so no receipt carries a path or a folder name, and the staging claim a removal
  writes is what ties one of its nonces to one folder.
- F16. The wrapper already owns a claim, owner and reap pattern for staging containers: a container named
  `.cli-git-worktree-copy-<uuid>` beside its destination, claimed in a journal root under the Git common directory
  before it exists, keyed to an owner, and reaped only when the owner is dead
  (`package/git-policy/cli/src/native/worktree_copying/worktree_copy_stage_claim.rs:501` to `:530`;
  `worktree_copy_stage.rs:205` to `:213`; `worktree_copy_selection.rs:46` to `:48`). That pattern admits only its own
  shape: a claim is read only when the container it names starts with `STAGE_PREFIX`
  (`worktree_copy_stage_claim_reaping.rs:215` to `:224`), a claim naming anything else reads as unreadable and is
  moved aside (`worktree_copy_unreadable_records.rs:178` to `:191`), and every settlement of a named container ends
  in `remove_stage`, which changes every directory's mode to `0700` and deletes the tree recursively
  (`worktree_copy_journal_removal.rs:34` to `:52`, `:91` to `:98`, `:440` to `:446`;
  `worktree_copy_stage_claim_reaping.rs:520` to `:531`). The existing reaper deletes its containers recursively and
  aborts its whole pass on one that is a mount, so a staging folder must never be one of those containers, and a
  staging claim is its own kind.
- F17. The wrapper has the descriptor and identity helpers this design needs: `open_root`, `open_directory_at`
  (which refuses a mount crossing with `EXDEV`), `open_directory_beneath`, `handle_path`, `identity_of` (device,
  inode and birth time), `make_directory_at` and `remove_at`
  (`package/git-policy/cli/src/native/subvolume/fs_beneath.rs`, `:184`, `:203`, `:230`, `:307`, `:384`, `:267`,
  `:335`).
- F18. The wrapper maps a descriptor to its device and inode (`opened_file_of_our_own`, a `statx` with
  `AT_EMPTY_PATH`, `process/open_file_of_a_descriptor.rs:235` to `:255`), to the mount ID of a path
  (`mount_of`, `process/process_use.rs:313`), to the file system type of a mount named in a process's mount table
  (`file_system_of_mount`, `open_file_of_a_descriptor.rs:104` to `:137`), and to the holders of a path
  (`processes_using_until`, `process_use.rs:552`). No helper returns a mount's source, so mapping an identity to a
  dataset's source and checking it against the binding's recorded GUID is new code this design adds, not reuse.
- F19. The measured staging in items 3 and 4 was one path on the guest's root file system, covered by the moved
  clone mount and unmounted by path, not this location, not a wrapper-created folder, and not a folder per removal
  ([measurements][measurements], items 3 and 4; `guest-probe.py`, items 3 and 4).
- F20. The wrapper's `REPORT_SCAN_CEILING` is the health report's own scan bound, measured on this host on
  2026-11-09, and not a shipped mount or unmount limit
  (`package/git-policy/cli/src/native/service/wait_ceilings.rs:1260` to `:1281`). Step 9 introduces the unmount
  deadline and step 22 sizes it ([plan][plan], "Wrapper code the design reuses", step 9, step 22).
- F21. Every mount structure gets its own ID (`mnt_alloc_id`, Linux `v6.8`, `fs/namespace.c:131` to `:139`, called
  from `alloc_vfsmnt` at `:202` to `:208`), and a propagation copy is a new structure (`clone_mnt`, `:1179` to
  `:1236`). A copy therefore carries a different ID from the mount it copies, so no scan can match a clone or a bind
  across namespaces by ID. A clone mount is matched by its dataset source, and a bind cannot be: `/proc/self/mountinfo`
  prints the source through the file system's `show_devname` when it has one (`fs/proc_namespace.c:175` to `:181`),
  ZFS has one (`zpl_show_devname`, `module/os/linux/zfs/zpl_super.c:301` to `:304`, registered in the superblock
  operations at `:1058`), and it prints the objset name the superblock was opened on (`__zpl_show_devname`, `:271`
  to `:299`, through `dmu_objset_name`), not the `mnt_devname` that `clone_mnt` copies. A bind therefore prints the
  same source and the same file system as the mount it binds, so it is matched by its mountpoint path, its root
  field, its parent mount and its ID within the current boot (`fs/proc_namespace.c:135` to `:151`).
- F22. `mount --bind` clones the covering mount with the same peer group, so the bind is a peer of the shared
  `user-cache` mount (`__do_loopback`, `fs/namespace.c:2581` to `:2603`, which calls `clone_mnt` with no
  `CL_PRIVATE`; `clone_mnt` copies the group ID at `:1193`), and the attach then runs `propagate_mnt` because the
  destination is shared (`fs/namespace.c:2335` to `:2347`; `fs/pnode.c:287` to `:326`), which copies the bind into
  every eligible peer mount and slave group of the destination, that is, every one whose root contains the target
  dentry (`propagate_one`, `fs/pnode.c:224` to `:233`). The measured bind is made private afterwards (F4), which
  changes the host's instance only. Those copies do not stay behind when the host's bind goes: an unmount follows
  the parent's propagation, not the unmounted mount's own group (`umount_tree` with `UMOUNT_PROPAGATE`,
  `fs/namespace.c:1795`; `propagate_umount`, `fs/pnode.c:573` to `:631`, which looks the copy up under every peer
  and slave of the parent at the same mountpoint), and an idle unlocked copy with no mounted child is unmounted
  with it (`__propagate_umount`, `fs/pnode.c:478` to `:517`). A copy a process is working inside raises the busy
  check instead, and the whole unbind then refuses `EBUSY` (F9, F30), so the host's bind and every copy of it stay.
  Propagation only reaches namespaces that already hold such a peer or slave; a namespace created later receives a
  full copy of whatever is mounted at the time it is created, which is a separate exposure (M6).
- F23. `/proc/self/mountinfo` prints each mount's current path and tags a mount `shared:`, `master:` or `unbindable`
  only when it has that property, so a private mount is the one that carries none of those tags
  (Linux `v6.8`, `fs/proc_namespace.c:135` to `:170`). `statmount` returns a mount's current mountpoint by mount ID
  (`fs/namespace.c:4828` to `:4837`, `:4906` to `:4941`; `struct statmount` in `include/uapi/linux/mount.h`), and
  `listmount` enumerates them. Both interfaces skip a mountpoint outside the observer's root: `seq_path_root`
  returns `SEQ_SKIP` (`fs/proc_namespace.c:150` to `:170`) and `statmount_mnt_point` turns that skip into success
  with an empty path (`fs/namespace.c:4828` to `:4837`). For an observer whose root contains the mount, which the
  handler's `/` does, a rename of an ancestor changes the printed path but does not leave the mount unnamed, so no
  design may claim a mount cannot be named after a rename.
- F24. The account can hold a mount namespace of its own, and it can mount in the host namespace through a setuid
  helper: `/usr/bin/fusermount3` and `/usr/bin/fusermount` are mode `-rwsr-xr-x` owned by `root` on this host (read
  2026-11-19 with `stat --format '%n %A %U'`). This host allows unprivileged user namespaces:
  `/proc/sys/user/max_user_namespaces` reads `252369` and `/proc/sys/kernel/unprivileged_userns_clone` reads `1`
  (read 2026-11-19). The premise "the account cannot mount in the host namespace", which the reviewed draft relied
  on, is therefore unsupported, and FUSE stacking in the host namespace is possible.
- F25. A mount cannot be moved out of another namespace: `do_move_mount` refuses `EINVAL` unless the destination's
  mount is in the caller's namespace ("The mountpoint must be in our namespace",
  Linux `v6.8`, `fs/namespace.c:3141` to `:3142`) and unless an attached source mount is the caller's own
  ("either ours or the root of anon namespace", `:3149` to `:3150`). A child that unshares a mount namespace
  receives copies, so a host mount stays in the host namespace.
- F26. GNU `rm` crosses mounts by default and offers `--one-file-system` only as an option: `rm --help` on this host
  reads "skip any directory that is on a file system different from that of the corresponding command line argument"
  under `--one-file-system` (coreutils, `rm --help`, read 2026-11-19). A third-party recursive delete of `~/.cache`
  therefore reaches a clone mounted inside it, unless that tool passes the option.
- F27. No existing pass can see a staging claim, and none may: every pass lists the journal root and keeps only its
  own suffix, `claim_names` keeps `.stage` (`worktree_copy_stage_claim.rs:662` to `:681`), `read_listed_journals`
  keeps `.json` (`worktree_copy_journal.rs:577` to `:607`), the temporary sweep keeps `.tmp` and the scratch suffix
  (`worktree_copy_stage_claim_reaping.rs:383` to `:400`), and the made-object sweep keeps `MADE_SUFFIX`
  (`worktree_copy_stage_claim_swept_files.rs:70` to `:90`). All of them list `journal_root`, which is
  `<common>/monochromatic-git-policy-cli-worktree-copy/v1` (`worktree_copy_journal.rs:132` to `:136`), and the
  forwarding check counts every entry of that directory except the unreadable folder as pending work
  (`journal_items_exist`, `:400` to `:415`, called from `worktree_copy_recovery.rs:596`), so a staging subdirectory
  under `v1` would make every forwarded command in that repository run the recovery pass. A claim directory beside
  `v1`, under the same private parent, is listed by no pass and counts as no pending work.
- F28. A by-name lookup under a directory descriptor crosses every mount on the way and lands on the topmost one:
  `step_into` runs the mount traversal (`fs/namei.c:1838` to `:1843`, `:1527` to `:1560`), which swaps the path for
  the covering mount (`__traverse_mounts`, `:1378` to `:1420`), and `RESOLVE_NO_XDEV` turns that crossing into
  `EXDEV` (`:1547` to `:1551`, `:1890`, `:1933`) while `RESOLVE_BENEATH` leaves it alone and guards only escapes
  (`:1912`, `:1945`). `/proc/<pid>/fdinfo/<n>` prints the mount ID of the descriptor's own mount (`fs/proc/fd.c:59`
  to `:61`, from `real_mount(file->f_path.mnt)->mnt_id`), so a by-name open under the `~/.cache` descriptor without
  `RESOLVE_NO_XDEV` returns a descriptor on the bind, and the number it prints is the identity the design checks
  against the bind's row (F21).
- F29. A file handle names an inode without a path: `name_to_handle_at` encodes one (`fs/fhandle.c:15` to `:75`), it
  refuses `EOPNOTSUPP` when the file system cannot encode a handle (`:25` to `:26`), it reports the size to ask for
  by `EOVERFLOW` (`:57` to `:66`), and `AT_EMPTY_PATH` takes the handle from a descriptor (`:97` to `:113`).
  `open_by_handle_at` opens it again under a directory descriptor (`:258` to `:266`), needs `CAP_DAC_READ_SEARCH`
  (`:182` to `:186`), keeps the directory descriptor's own mount (`get_vfsmount_from_fd`, `:118` to `:135`) and the
  decoded dentry (`:142` to `:165`), and opens the empty path under it (`file_open_root`, `:235`), so no mount is
  crossed. ZFS implements both directions (`zpl_export_operations`, `module/os/linux/zfs/zpl_export.c:251` to
  `:257`; `zpl_encode_fh` at `:36`, `zpl_fh_to_dentry` at `:100`), and it resolves a handle by object number and
  generation: an object that is unlinked, or whose generation differs, gives `ENOENT`
  (`module/os/linux/zfs/zfs_vfsops.c:1584` to `:1695`), which the export code turns into `ESTALE` (`zpl_export.c:119`
  to `:127`). A renamed folder therefore still resolves and a folder deleted and re-made under the same name does
  not.
- F30. The unbind is a non-lazy unmount, so its busy check covers the copies as well: `do_umount` calls
  `propagate_mount_busy(mnt, 2)` (`fs/namespace.c:1792`), which refuses when the mount itself, or a copy under a peer
  or slave of its parent at the same mountpoint, holds more references or has a mount under it (`fs/pnode.c:407` to
  `:441`, with `do_refcount_check` at `:352` to `:355`; F9). A process working inside a copy therefore refuses the
  host's unbind with `EBUSY`, and the host's bind stays until that copy is released, its namespace exits, or the
  mount is lazily detached, which this design forbids.
- F31. A new mount namespace gets the peer groups of the one it was copied from: `copy_mnt_ns` copies the tree with
  `CL_COPY_UNBINDABLE | CL_EXPIRE` and adds `CL_SHARED_TO_SLAVE` only when the new namespace belongs to another user
  namespace (Linux `v6.8`, `fs/namespace.c:3760` to `:3788`). A namespace created in the same user namespace therefore
  holds peers of the host's shared mounts, and one created in a new user namespace holds slaves of them; both receive
  propagation. A namespace whose own tree was already made private, which is what a session that runs
  `mount --make-rprivate` leaves behind, holds a private copy and receives nothing, so any arm about copies must say
  which kind of reader it uses.

## Assumptions

- A1. The handler acts in the host mount namespace, as the design states, and the wrapper normally does too. The
  `EBUSY` protection of F11 holds only inside the namespace that holds the mount, so the handler records the device
  and inode of its `/proc/self/ns/mnt` in the current boot's bind record, one file per removal named
  `/run/git-worktree-clone-mount-results/staging/<hex>.binding` (root-owned, mode `0644`, beside the receipts and
  outside their GUID-only grammar), and the wrapper reads the record of the nonce it is settling, compares the pair
  with its own, and refuses on a mismatch before it removes any folder. A boot clears the file with the mounts it
  describes (L10).
- A2. The account home the staging target lives in is the passwd home of the pinned UID, not `$HOME` or
  `$XDG_CACHE_HOME`, because a root handler has neither and the answer names `~/.cache`. That trust source is a
  proposal of this design ("Proposals the owner may veto").
- A3. `~/.cache` stays the mount of `zpcachyos/nosnap/user-cache` at boot and is not replaced by the account between
  setup and a removal. If it is replaced, the source-and-parent identity check refuses rather than adopts it.
- A4. The account does not deliberately sabotage its own staging, which is the same-account residual the delegated
  destroy decision already accepts ([proposal][proposal], "Clone removal identity protection"). Routine cleaning of
  `~/.cache` is not sabotage and is covered by "Limits".
- A5. The removal flow obtains a receipt for its `unmount` nonce, showing the clone absent in every namespace and the
  bind gone from the host namespace, before the wrapper removes anything; a removal whose receipt never arrives, or
  arrives unclear, keeps its staged state reported and removes no folder. A refused unbind is a clear receipt of the
  other kind: it names the failed step, the errno and the copy that refused it, and the wrapper then removes nothing.
- A6. The staging folder is only ever a mountpoint carrier: nothing but the bind and the moved mount is put inside
  it, so a successful unbind leaves it empty.
- A7. A staged mount's presence in the host namespace is acceptable to the account, since the folder sits in the
  account's own `~/.cache`, at the mode F1 records.

## Limits

- L1. No run exercised this design. Every VM statement in [the measurement record][measurements] comes from guest
  `6.8.0-147-generic` with OpenZFS `2.4.4-1`; nothing ran on `7.3.5-1-cachyos`, and the wrapper, handler and
  binding have not been built.
- L2. Kernel citations are to upstream Linux `v6.8`, fetched by URL at that tag. The guest kernel is Ubuntu's
  `6.8.0-147-generic`, whose patch set is not upstream `v6.8`, and the host kernel is `7.3.5-1-cachyos`, which no
  citation here covers.
- L3. No measurement exists for an unmount or an unbind through a descriptor on the covered folder, for the mount
  recovered from a recorded file handle, for the by-name open that reaches the bind, for a bind of this location, for
  a wrapper-created folder, for two removals at once, or for this location's boot ordering.
- L4. Item 2 measured an account renaming `.claude` between the open and the attach; item 4 measured a rename after
  the open. Neither measured a substitution between the staging identity check and the move at this location.
- L5. Free-running interleavings, concurrent same-name replacement, a failed reattach and crash recovery were not
  measured ([measurements][measurements], "What this repeat did not show").
- L6. The forced arm is still owed, and if a force-unmount works it changes the staging unmount rule this proposal
  keeps ([proposal][proposal], "Forced removal and deferred origins").
- L7. Nothing here sizes a shipped deadline, and no timing claim is made. Step 22 sizes the waits.
- L8. The staging route's cost in account-visible churn is unmeasured: the proposal states no number, and this
  proposal adds none.
- L9. A third-party recursive delete can cross into a staged clone (F26), and the account can clean `~/.cache` at
  any time. The design does not prevent that. It reports a staged mount (the `doctor` line of "What a writer can
  settle without asking") and leaves the clone's worktree files inside the mount, so a reattach after such a delete
  restores what the delete did not remove.
- L10. A mount ID is unique only while its mount structure lives (F21), and a reboot gives new structures. No
  durable root record may key anything on a mount ID: a boot voids every staged record it holds, and only the
  current boot's `/run` state and receipts carry IDs. The handler's mount-namespace identity is boot-scoped the same
  way, because a new boot gives the namespace a new inode number.
- L11. A clone mount that another namespace detached lazily leaves no row anywhere while it still holds the dataset
  (F13), so the refusal a removal sees is the destroy's `dataset is busy`, not a mount row. `doctor` reports the
  busy refusal and the retained staged state; nothing in this design releases such a mount.
- L12. The recorded-handle route of F29 is designed from source and measured nowhere: not for this file system's
  handle encoding, not for the capability the handler holds, and not for a folder renamed or replaced between the
  record and the recovery. It is also the route a sweep needs and the only one it has; M6 carries its arms and every
  one of them is owed.

## The parts that need a design

The item names the staging target's creation, the private bind, the wrapper's cleanup, the unmount interface, and the
ordering against the `user-cache` mount. Each is stated here as the shape every way fills.

### The staging target and who creates it

The target is a directory under the account's `~/.cache` that the root handler can bind and into which it can move a
clone mount. It must be found by a no-crossing walk from the pinned home (F17), its identity recorded before the
first bind and rechecked before every move, and it must not be created by a request, since no request carries a path
(F15). Its name is the wrapper's own: `32` lowercase hex digits written into the staging claim before the folder
exists, never the request's nonce, which the accepted text keeps out of every filename ("The staging claim"). The
owner's words name the wrapper as the cleaner; who creates the folder is part of Q1, because the proposal's rule
stops the handler from creating or removing "account slots" and calls its slot descriptors descriptors "for mount
management, not privileged writes into the account's tree", and whether a staging folder is such a slot is a
writer's reading of those lines.

### The private bind

The bind is what makes the staging target a mount instance whose propagation the handler sets, because attaching a
mount under a shared parent propagates it (F6) and `~/.cache` is shared (F1). F4 records the measured shape. The
order is load-bearing: the bind is attached first, which also copies it into every peer and slave namespace of
`~/.cache` (F22), and made private second, after which the clone's move onto it propagates nowhere. The propagation
change needs the bind's own descriptor, and the folder descriptor cannot give it: `mount_setattr` requires the path
to be the root of its own mount (F14), and the folder is not the root of `user-cache`. The handler therefore takes
the bind's descriptor with the design's only by-name lookup after a mount covers the folder, a name under the
`~/.cache` descriptor without `RESOLVE_NO_XDEV`, and confirms it by the mount ID `/proc/self/fdinfo` prints (F28,
F21). That lookup is not a path-based mount or unmount: it is a read that names the mount, the propagation change
runs on the descriptor, and no mount call receives a path inside the account's tree (the decided sentence and the
plan's forbidden line). The bind is a mount in the host namespace at a path inside the account's tree, which is the
exposure the decided default accepts ([proposal][proposal], "Decided defaults"); its copies in other namespaces are
a further exposure this design states rather than denies, and the unbind's rule over them is F22 and F30.

### The unmount call from staging

The unmount must not force or lazily detach, must not act on a path the account can change, must release
descriptors that would hold the mount busy, and must end with the mount absent from every namespace (F9, F13, and
the decided sentences quoted under "Decisions of the accepted design that bound this proposal"). F7 says how: a
`umount2` call whose argument is `/proc/self/fd/<n>`, with the descriptor taken on the staging folder before the
bind. The lookup lands on the topmost mount at that folder, so the first call unmounts the clone and the second
unmounts the bind, and both calls meet the decided sentence because the string they receive names a descriptor in
`/proc`, not a path inside the account's tree. The descriptor keeps a pathname substitution out of the call, but it
cannot keep a mount the account stacks on the folder out of the stack the kernel selects (F24): the call removes the
topmost mount whatever it is, so after each call the handler asserts the transition it expected, that the recorded
clone is gone and the recorded bind is now topmost, then that the bind is gone and no mount is left. A topmost mount
that is not the recorded one means the call removed something else, and the handler refuses, reports, and unbinds
nothing. F7 also means the call needs no name under any ancestor, so a rename of an ancestor cannot take the staged
mount out of reach. The reviewed draft tested only a descriptor opened through the clone, which F10 says holds the
clone and must refuse (F9); M2 keeps that reading as the control.

That descriptor lives one handler invocation. A later sweep has no descriptor of its own and cannot make one by name:
a by-name open crosses the bind or the clone (F28) and the descriptor it returns pins the very mount the sweep would
unmount, so the call then refuses `EBUSY` (F9, F10). The sweep therefore opens the folder from the handle the handler
recorded before the bind (F29). Recording it costs no capability; the recovery needs `CAP_DAC_READ_SEARCH`; the
descriptor it returns is on the `user-cache` mount, names the folder whatever its name is now, and does not cross the
bind, so `umount2("/proc/self/fd/<n>", 0)` then selects the topmost mount at that dentry exactly as the held
descriptor does. A handler that cannot record the handle before the bind does not bind: it refuses, reports the
failed step, and the clone stays at its slot, which is a state the accepted design already blocks a removal on. A
sweep whose handle no longer resolves, or whose process lacks the capability, unmounts nothing and removes nothing,
and reports the retained staged mount, which Q2 answers for. None of this is measured (L12, M6).

The accepted design asks for "the interface that unmounts a staged mount from the held descriptor" and for an act
"on the held descriptor and never on a path inside the folder the account can change". This design reads the held
descriptor as the folder descriptor the handler holds from before the bind, which is what its own calls use, and it
reads the recorded handle as the way a later sweep holds that same folder again: a handle names the inode and
generation the descriptor named, and it is not a path.

### What the wrapper removes and when

The answer's words are "make the wrapper clean after itself". The cleanup must not recurse over a mount, must not
remove a name it cannot prove it made, must cover an interrupted or failed removal, and must leave a retained staged
mount tracked and reported (the acceptance item quoted under the decisions). The wrapper already owns a claim, owner
and reap pattern for staging containers (F16), which is the nearest existing owner of this responsibility, and F16
also says why a staging claim cannot be one of those containers: the existing reaper deletes its containers
recursively and aborts its whole pass on one that is a mount. The claim also has a place that no existing pass counts
as pending work, because a folder inside the journal root would make every forwarded command in that repository run
the recovery pass (F27). The folder itself is removed by `rmdir` of a name the wrapper can prove: the one the claim
records, or the one an identity scan of `~/.cache` finds for the recorded device and inode after a rename (the
handle of F29 survives a rename; a name does not). A folder that matches no claim of any registered repository is an
orphan, which `doctor` finds and prints, and which no automatic pass removes.

### Ordering against the mount of `~/.cache`

A bind made before `zpcachyos/nosnap/user-cache` mounts lands on the home underlay and is hidden when the dataset
mounts later (F12). Nothing orders the two today, and the design's open item says so. Any way must either refuse to
bind until the recorded mount source is present or be created after that mount exists. The record is the dataset's
source (`zpcachyos/nosnap/user-cache`) and the identity of the folder under it, never a mount ID across a boot
(L10).

## Ways to meet it

Each way covers the parts named in "The parts that need a design". Costs are enumerated work, not sizes or times;
none is measured.

### Way A: a claimed, depth-one folder per removal, bound and made private for that removal

- The wrapper checks `~/.cache`'s mount source against the pin of "Proposals the owner may veto" before it creates
  anything, so a missing or replaced `user-cache` mount writes no entry on the home underlay (F12), then draws `32`
  lowercase hex digits of its own, creates `<home>/.cache/monochromatic-git-policy-cli-staging-<hex>` once per
  removal, at the mode F1 records, and claims it in a staging claim of its own kind ("The staging claim") before the
  folder exists. The hex value is the wrapper's, never the intent's nonce (F15).
- The handler opens `~/.cache` by descriptor from the pinned home, checks the mount's source against the pin, opens
  the claimed folder by `openat2` under that descriptor with `RESOLVE_BENEATH` and keeps that descriptor, which is on
  the `user-cache` mount (F10, F17, F28), checks the folder's identity against the claim and the descriptor's mount
  ID against the `user-cache` row (F21, F28), records the folder's file handle and its own mount-namespace identity in
  the boot's bind record (A1, F29), attaches the bind with the descriptor's path as both source and target
  (`/proc/self/fd/<n>`), opens the bind by name under the `~/.cache` descriptor without `RESOLVE_NO_XDEV` so the
  lookup crosses into the bind, checks that descriptor's `mnt_id` in `/proc/self/fdinfo` against the bind's row
  (F28, F21), and makes it private with `mount_setattr(fd, "", AT_EMPTY_PATH, ...)` (F14), then closes that
  descriptor and moves the clone onto the bind by `move_mount` from the clone's own descriptor (F14, F5). The
  descriptor lifetimes stay apart (F10): the folder descriptor, opened before the bind and kept until the bind is
  gone; the bind descriptor, which exists only for the propagation change and is closed before the move; and every
  clone descriptor, closed before the clone's unmount.
- The unmount is two `umount2("/proc/self/fd/<n>", 0)` calls, clone then bind (F7, F8), each preceded by a check of
  the topmost mount's ID in the mount table: the clone is confirmed by its dataset source and the bind by its root,
  parent and mountpoint, which F21 says is the only way to tell a bind from the mount it binds. Both run after every
  descriptor on the clone is closed (F10) and after the holder scan is clear. A later sweep runs the same two calls
  from the handle of F29, in the same order and with the same checks (M6).
- The handler unbinds, scans every namespace for the clone's source, writes the receipt, and the wrapper removes the
  folder by `rmdir` only after that clear receipt, after its own mount-namespace identity matches the handler's
  recorded one, and after re-checking the folder's identity against the claim (A5, F11). The unbind unmounts the
  copies in peer and slave namespaces too; a copy a process works inside refuses the whole unbind with `EBUSY`, and
  the handler then reports the copy, its namespace and the processes holding it instead of claiming the bind is gone
  (F22, F30).
- The folder's only ancestor inside `~/.cache` is the mount root of the dataset, which the account cannot rename in
  the host namespace (F11), so no ordinary ancestor can carry the bind away.

Pros: no privileged write into the account's tree; the creator is also the cleaner; one claim per removal; the bind
lives one removal, so concurrent removals use their own folders and their own binds and need no lock; the unmount
and unbind meet the decided sentence literally (F7); the depth-one folder removes the renamable ancestor the reviewed
draft carried, and the recorded handle survives a rename of the folder from a foreign namespace (F29), so only a
recovery the handle cannot serve waits on Q2; the retained staged state the busy decision allows is a claimed folder
whose claim this design can prove and report.

Cons: the account can remove the empty folder before the bind and replace it, which the identity check refuses, and
the window between the check and the bind is a race the kernel does not close (M1's replacement arms); the bind is
copied into peer and slave namespaces of `~/.cache` at attach time (F22, M3), and a successful unbind takes those
idle copies with it while a copy a process works inside refuses the unbind (F30, M4); the by-name open of the bind is
the one lookup that runs after a mount covers the folder, and its mount ID check is what keeps it honest (F28); a
recovery needs the handle, which the wrapper cannot record on the handler's behalf, so a lost handle is a retained
mount (L12); a wrapper crash between claim and folder removal leaves a folder and a stale claim the staging sweep
must settle; a third-party recursive delete of `~/.cache` reaches a staged clone (F26, L9).

Build cost: one wrapper module for the staging claim, folder creation, identity check, `rmdir`, the orphan scan and
notices; one handler step for the handle record, bind, propagation change, move, two unmounts, unbind and the
receipt, with its state fields; a new deadline owned by step 9 and sized by step 22 (F20); `doctor` text for a
retained staged mount, an orphaned folder, an unprovable claim, a handle that no longer resolves, and a detached
mount that still holds the dataset.

Test cost: device-free tests for the staging claim kind, the folder grammar, `rmdir`-only removal, each refusal's
text, the identity mismatches and the namespace identity check; the VM arms M1, M2, M3, M4's positive,
host-namespace, foreign-namespace, renamed and copied arms, M5, every arm of M6, and M7's two-folder arm.

### Way B: a root-made target per removal, bound for that removal

- Root creates `<home>/.cache/monochromatic-git-policy-cli-staging-<hex>` mode `0700` owned by root, records its
  identity under `/var/lib/monochromatic-git-policy-cli/`, binds it, makes it private, moves the clone onto it,
  unmounts, unbinds, and the wrapper removes the target after the clear receipt.
- The wrapper creates no folder; it records the removal and waits for receipts.

Pros: the account cannot plant content in the target, and the identity it is checked against comes from root's own
record rather than from an account-writable claim; the account's rename of the target's name is refused by the
descriptor identity check before the move, as in Way A.

Cons: root writes a directory into the account's tree, which the writer's reading of the proposal's slot rule
avoids; root owns an object under an account-owned parent, which the account can still remove or rename from a
foreign namespace (F11), so the protection buys less than it appears to; the wrapper must report a state it cannot
create, and `doctor` must explain a root-owned entry in the account's cache; the wrapper's cleanup of a root-made
object needs an ownership rule the design does not have.

Build cost: Way A's handler work plus target creation, ownership and removal, a root-state record per target, a
refusal path for an account-removed parent, and the wrapper's side reduced to waiting, reporting and `rmdir`.

Test cost: Way A's arms plus M8, and the same device-free tests for the wrapper's reduced side.

### Way C: one boot-made target, bound private once per boot, with a folder per removal inside it

- The boot unit creates `<home>/.cache/monochromatic-git-policy-cli-staging`, binds it, makes it private (the bind's
  own descriptor, taken and checked as in Way A, is what carries the change: F14, F28), records its identity, and
  keeps it until the stop pass. Each removal creates its folder inside the bind and moves its clone
  there.
- Each removal's unmount is one call, on the clone only: after that the folder is a plain directory inside the bind,
  not the root of a mount, so a second `umount2` through a folder descriptor would fail `path_mounted` (F8). The
  bind is removed by the stop pass, which needs its own anchor: either a descriptor the boot unit kept alive or a
  path unmount under `~/.cache`, which the decided sentence forbids; Q2 answers for a retained bind whose anchor is
  gone.

Pros: the bind exists before any removal, so a removal never waits for a mount transition; the boot unit's start
reconcile gains one setup step instead of an on-demand step; the folders inside one private bind do not interact, so
no global claim and no serialisation of unmounts is needed (this corrects the reviewed draft, which read the shared
target as forcing one).

Cons: a mount in the host namespace for the whole boot; every removal's folder sits inside it, so an account
`rmdir` from a foreign namespace detaches the bind and every staged mount with it (F11); the stop pass must unmount
a staged clone before the bind, and must never force it; the stop pass also needs an anchor for the bind itself,
which a boot-once process does not keep (Q2); the boot unit must order the bind after the `user-cache` mount, which
nothing does today (F12); the bind's copies in peer and slave namespaces live and go with it, with the same busy rule
as Way A's (F22, F30); the bind's removal falls to the boot unit rather than to the wrapper, so the owner's "make
the wrapper clean after itself" does not cover it.

Build cost: Way B's creation work moved to the boot unit, plus stale-bind recovery and reporting, plus boot ordering
in the boot unit and its `doctor` text, plus the stop pass's extra step.

Test cost: Way A's arms plus M4's renamed-folder arm, M5's delayed mount and M7's Way C arm.

### Way D: make `~/.cache` private, and stage in a plain folder with no bind

- The boot unit makes `~/.cache` private (`mount --make-private` or `mount_setattr` on the dataset's mount; a
  descriptor opened on `~/.cache` is that mount's root, so the descriptor call works on it, F14), records the mount's
  source and identity, and each removal creates a plain folder inside it.
- Nothing is bound: the clone is moved onto a folder under a private parent, so the attach propagates nowhere (F6),
  and the unmount is one `umount2("/proc/self/fd/<n>", 0)` on the folder descriptor.

Pros: no bind at all, so no copy in a peer or slave namespace (F22) and no second mount to track; the unmount and
the move both act on the descriptor F7 provides; a propagation event never carries the staged clone into another
namespace, though a namespace created while it is staged still receives a copy (F22, M6); a retained staged mount has
no bind to unbind and no copy that can refuse that unbind (F30).

Cons: it changes the propagation of an existing mount, which the proposal leaves outside itself ("Hardening those
existing mount targets is outside this proposal"), so the owner must accept it; the same change stops late mounts
under `~/.cache` from reaching already-confined readers, which is the cost the proposal recorded for the checkout's
private propagation, though the proposal names no consumer of that delivery under `~/.cache`; the boot unit must
re-apply it after a replacement mount, which the proposal lists as an open item, and a replacement mount in the
middle of a boot arrives with the propagation its attach gives it, which for this mount is shared again (F1, F6).
`zfs mount -a` is not that trigger: against an already-mounted dataset it returns success without mounting
anything (`cmd/zfs/zfs_main.c:7380` to `:7388`), so only an unmount and a fresh mount, or a reboot, replaces the
mount.

Build cost: a boot-unit step, a recorded source and identity check, the same claim and folder management as Way A
without the bind, and re-application reporting.

Test cost: M1, M2, M3's propagation arms on `~/.cache` itself, M5, every arm of M6, and M7, plus the control that an
already-confined reader loses a late mount under `~/.cache`.

### Way E: a root-owned empty source bound onto an account-made folder

- Root owns an empty directory under `/var/lib/monochromatic-git-policy-cli/`, binds it onto the account-made
  folder, makes the bind private, and moves the clone onto the bind.
- The unmount and unbind are Way A's; the source directory stays root's for the boot.

Pros: the mount's source is root's, so the folder's content side is not account-controlled, and the account cannot
plant a file that a later reattach would restore.

Cons: the target holds no content at all (A6), so the protection covers nothing this design stores; the mountpoint is
still account-made, so the account can remove or rename it from a foreign namespace (F11) and the exposure is
unchanged; root's empty directory is a root-owned place outside the account's tree, which is the shape the project
vetoed when it refused the root-owned `0700` bind at `/run/git-worktree-clone-mount-staging`; it adds a second root
object and its lifecycle for no measured gain.

Build cost: Way A's work plus a root-owned source directory, its boot-long lifecycle, and a check that the source is
still root's empty directory.

Test cost: Way A's arms plus M8's replacement arms against the source rather than the target.

### Way F: one claimed staging root two levels deep, with a folder per removal

- This was the reviewed draft's leading way: the wrapper creates
  `<home>/.cache/monochromatic-git-policy-cli/stage` once and each removal's folder under it.

Pros: one recognizable staging root for `doctor` to print, and one identity to check instead of one per removal.

Cons: the root's parent `monochromatic-git-policy-cli` is an ordinary directory the account can rename in the host
namespace, which carries the bind and the staged clone to a new path and makes the recorded path fail; a durable
record of the root's identity then refuses every unmount until an administrator re-records it after a legitimate
recreation of `~/.cache`; it buys nothing Way A does not have.

Build cost: Way A's work plus a durable root identity record and its adoption rule.

Test cost: Way A's arms plus M4's renamed-folder arm.

### Refused ways

The namespace teardown is refused, not ranked. It would have the handler create a child in a private mount namespace,
move the clone into it, and let the child exit, so no mount ever lands in the account's tree. F25 refuses it at
`v6.8`: `do_move_mount` requires the moved mount to be the caller's own, so a child that unshares receives copies and
the host's clone stays where it is. The reviewed draft offered it as an option and called it unmeasured; it is worse
than unmeasured, it cannot be built, and M9 records the refusal.

Ways B, C, D, E and F are not refused by any decision: each is a design a writer could build, and Q1 asks which one
to take where the records leave the choice open.

## Ranking

Ranking: A > D > B > C > E > F.

- A > D, because A changes no existing mount, while D removes the bind and every copy it creates (F22) and pays by
  changing `~/.cache`'s propagation, which the proposal leaves outside itself, and by cutting late-mount delivery
  under `~/.cache` to confined readers. A's price is the recovery anchor and the copied bind's unbind rule: a sweep
  without the capability cannot clear a retained bind, and a copy a process works inside refuses the unbind (F29,
  F30), while D has no bind to recover and no copy to be busy.
- D > B, because D has no privileged write into the account's tree and no root-owned object under an
  account-owned parent, and B's content protection covers a folder that holds no content (A6).
- B > C, because B's object and bind live one removal, while C keeps a host-namespace bind for a whole boot, puts
  every staged mount inside it, and moves a mount transition into boot reconcile where nothing orders it after the
  `user-cache` mount (F12).
- C > E, because C's target is account-made and needs no root-owned source, while E adds a root-owned directory and
  its lifecycle to protect content the folder never holds, and repeats the shape the project vetoed at `/run`.
- E > F, because E at least changes what the mount's source is, while F adds a renamable ancestor, a durable root
  identity and an adoption rule in exchange for a single printed path.

## Owed measurements in a throwaway VM

Each runs in a disposable guest on a file-backed fixture pool, as the [measurement record][measurements] fixture
does, with the built wrapper and the built handler, as account and as root, never against this host's pool and never
with its `/dev/zfs`. The fixture's own datasets are named, never this host's: the arms create
`<fixture pool>/home/cache` as the guest's `user-cache` shape and mount it at the guest home's `.cache`, and the pin
the handler checks is configured to that dataset. A run records `uname --kernel-release`, `zfs version` and
`/sys/module/zfs/version` first, and keeps its commands and output. Every arm names an assertion that can fail and a
control, because a null result is worth nothing without one. Mounts are matched by source (the guest's dataset name)
and by the guest's own mount namespaces (each distinct `/proc/<pid>/ns/mnt`), never by mount ID across namespaces
(F21).

### M1. The staged move and the descriptor-armed unmount

Set up a clone at a slot, a private bind on a depth-one staging folder, and the move. Arms:

- Positive: no holder. Unmount the clone by `umount2("/proc/self/fd/<folder descriptor>", 0)`; assert the call
  returns success, no row in the host namespace whose source is the clone's dataset, an empty folder, and the
  clone's `zfs destroy` then succeeding. Control: a destroy attempt before the unmount refuses `dataset is busy`
  (F13), which proves the refusal is observable in this fixture.
- Busy: a cwd holder inside the staged clone. Assert `EBUSY`, the clone's source row still in the host namespace, and
  the destroy refusing. Control: the same arm after the holder exits reaches success.
- Replaced before the open: the account removes the claimed folder and makes a new one of the same name before the
  handler opens it. Assert the identity check refuses, no row for the clone's dataset appears, and the claim is
  reported. Control: the undisturbed folder in the same run proceeds.
- Replaced between the open and the bind: the account removes the folder after the handler holds the descriptor and
  makes a new one. A removed dentry cannot become a mountpoint: `vfs_rmdir` marks the inode dead and calls
  `dont_mount` (`fs/namei.c:4213` to `:4216`), and `d_set_mounted` refuses an unlinked dentry with `ENOENT`
  (`fs/dcache.c:1369` to `:1394`). So assert the bind is refused, the replacement name holds no mount, and the claim
  is reported; record the errno. Control: M1's positive arm in the same run, where the undisturbed folder carries the
  bind and the clone, so the refusal in that arm is the removal's and not the fixture's.
- The bind's descriptor: after the bind and before the propagation change, open the folder by name under the
  `~/.cache` descriptor with `RESOLVE_BENEATH` and no `RESOLVE_NO_XDEV`; assert `/proc/self/fdinfo/<n>` prints the
  mount ID of the bind's row (F28, F21), then assert `mount_setattr(fd, "", AT_EMPTY_PATH, ...)` succeeds and the
  row loses its `shared:` tag (F14, F23). Control: the same two calls on the folder descriptor taken before the
  bind, where the `fdinfo` mount ID is `user-cache`'s and `do_mount_setattr` refuses `EINVAL` (F14), and no row
  changes; and `open_tree(folder descriptor, "", AT_EMPTY_PATH)`, which must also report `user-cache`'s mount ID, so
  a reader that prints the bind's ID for both has not read what it claims (F14, F28).
- Account attempts on the folder's names: the account renames and `rmdir`s the folder and `~/.cache` from the host
  namespace while the bind stands; assert `EBUSY` for each. Control: M4's foreign-namespace arm, which must succeed,
  so the refusals here are the namespace's and not the kernel's everywhere.
- Two folders: two clones staged in two folders; assert both source rows are present before either unmount, that
  unmounting one leaves the other's source row in the host namespace, and that unmounting the second then succeeds.
  Control: the two-row scan before the unmounts, which proves the scan finds mounts when they exist.

### M2. The unmount argument: covered folder, clone root, and a name under the descriptor

- Covered folder: M1's positive arm, then a second `umount2("/proc/self/fd/<folder descriptor>", 0)` for the bind.
  Assert success for both calls, no row in the host namespace whose mountpoint is the folder path, and the clone's
  dataset gone from `zfs list`. Control: M2's clone-root arm.
- Clone root: after the move, open `O_PATH` on the folder through the clone and unmount by that descriptor. Assert
  `EBUSY` and the clone's source row still present (F10). Control: the covered-folder arm in the same run.
- Name under the descriptor: unmount the path `/proc/self/fd/<cache descriptor>/<folder name>`, built from a
  descriptor on `~/.cache` and the folder's name. Assert success and no row for the clone's dataset. This records
  that the reviewed draft's route works at the kernel level, and that the design does not use it because it names a
  component inside the account's tree (F7, and the plan's forbidden line). Control: the covered-folder arm, which
  must reach the same outcome without that name.
- Substituted top mount: the account stacks a FUSE mount on the staged folder with `fusermount3` between the
  handler's identity check and its first `umount2`. The call removes the topmost mount, which is the FUSE mount, so
  assert the post-call transition check fails: the topmost mount is the recorded clone again, not the recorded bind,
  and the handler therefore refuses, reports and unbinds nothing. Control: the covered-folder arm in the same run,
  where the topmost mount becomes the recorded bind and the second call proceeds. The arm records what the race
  costs: a mount the account made can be the one the call removes, and no kernel call closes that window (F24).

### M3. The private bind is required, and what it copies

- Positive: bind the folder, make it private, move the clone, and assert the folder's row in the host
  `/proc/self/mountinfo` carries no `shared:`, `master:` or `unbindable` tag (F23), and that a reader namespace
  started before the move has no row whose source is the clone's dataset at any path. The reader must hold a peer or
  a slave copy of the `user-cache` mount, so the arm asserts its `user-cache` row carries `shared:` or `master:`
  before the bind: a namespace copies peer groups as they are and turns them into slaves only when it belongs to
  another user namespace (F31), and a session that already made its own tree private gives a private copy, which
  would receive nothing whatever the arm did. Control: that same reader namespace shows a row whose mountpoint is the
  folder path, with the bind's root field and the dataset's source (F21), after the bind and before the unbind, which
  proves its scan can see the folder's mounts (F22); a second control is the same reader made private before the
  bind, which must hold no such row.
- Negative: the same move into a plain folder under the shared `~/.cache` without a private bind. Assert a peer
  namespace that holds a copy of the `user-cache` mount shows a row for the clone's dataset at the folder path, and
  that its row carries `shared:` (F6). Control: the positive arm's reader, which must show none.
- Checkout shared: the same positive arm with the checkout shared. Assert the move refuses `EINVAL` (F5). Control:
  the positive arm's checkout, private, which must move.

### M4. Removing the bind, and what the account can still do from where

- Positive: unbind by the covered-folder descriptor after M1's unmount. Assert success, no row in the host namespace
  whose mountpoint is the folder path, the wrapper's `rmdir` succeeds, and the folder is gone. Control: an `rmdir`
  before the unbind gives `EBUSY`.
- Host-namespace account arm: while the bind stands, the account runs `rmdir` on the folder and on `~/.cache` in the
  host namespace; assert `EBUSY` each time and the host rows unchanged. Control: M4's foreign-namespace arm, which
  must succeed.
- Copied bind, idle: a reader namespace holding a peer or a slave copy of the `user-cache` mount (F31), opened
  before the bind. Assert the reader holds the bind's copy at the folder path, then that the host's unbind succeeds
  and the reader's copy is gone in the same scan (F22). Control: the same reader before the unbind, whose copy is
  the assertion, so a scan that never found the copy could not claim it went.
- Copied bind, held: the same reader with a process whose working directory is the folder inside its own copy.
  Assert the host's unbind refuses `EBUSY`, the host's bind row stays, and the receipt and `doctor` name the copy,
  its mount namespace and the holding process (F30, A5). Control: the idle arm in the same run, which succeeds, so
  the refusal is the copy's and not the fixture's.
- Foreign-namespace account arm: an account process in a private mount namespace created before the bind runs
  `rmdir` on the folder while a cwd holder sits inside the staged clone. Assert success, the host namespace loses
  the bind and the clone (F11), the holder's reference keeps the dataset busy under a detached mount, so a
  `zfs destroy` refuses `dataset is busy` (L11), and `doctor` names it. Control: the same arm with no holder, where
  the detach leaves no row for the bind in any namespace the scan reads (the assertion), and the destroy's output is
  recorded rather than predicted, because a dataset whose last reference went may either destroy or refuse; the
  host-namespace arm in the same run refuses with `EBUSY`.
- Renamed folder arm: from a foreign namespace, rename the folder while the bind stands. Assert the recorded bind's
  mount ID is still in the host namespace, printed at a different mountpoint than the recorded folder path, and that
  the staged clone is still on it: a rename moves the source dentry and its mounts, and only an overwritten target is
  detached (F11, F23). Control: the `rmdir` arm detaches, so a scan that finds the bind after a rename can also
  distinguish a detach.

### M5. Ordering against the guest's `user-cache` dataset

- Negative: mount the fixture home without `<fixture pool>/home/cache`, or delay that dataset's mount, then run the
  wrapper's claim and creation step and the handler's staging step. Assert both refuse, no row for the bind appears,
  no entry is created on the home underlay
  (the underlay's listing is unchanged and `findmnt --target <guest home>/.cache` names the fixture home), and
  `doctor` names the pin's dataset. Control: the positive arm in the same run.
- Positive: with `<fixture pool>/home/cache` mounted, assert the folder's row has the cache dataset's
  `sb_dev_major:minor` and the same source as the dataset's own row, and that its root field and mountpoint make it
  the bind rather than the dataset's own mount, which F21 says a source comparison alone cannot do. Assert the pin's
  recorded source matches. Control: the negative arm, whose refusal must not be reachable here.
- Boot order: assert the ordinary boot order reaches the positive arm on the first attempt. Control: the negative
  arm's delayed mount, which must be the only way to the refusal.

### M6. Interruption, namespace identity, and the cleanup pass

- Kill between the bind and the move. Assert the bind's row is present, the clone's source row is still at its slot,
  the staging claim is present, the boot's bind record carries the folder's handle and the handler's namespace pair,
  and no later pass removes the folder or the claim. Then assert the next `unmount` intent's sweep opens the folder
  from that handle (F29), unmounts the empty bind, and leaves the clone at its slot, so the removal starts over at the
  slot and may refuse there again, which is the accepted busy refusal. Control: M1's positive arm in the same run,
  where an undisturbed handler moves the clone.
- Kill after the move, before the unmount. Assert the staged clone's source row is present, the staging claim is
  present, the boot's bind record carries the folder's handle and the handler's namespace pair, no later pass
  removes the folder or the claim, and no reattach happens (the no-implicit-remount default). Then assert the next
  `unmount` intent's sweep opens the folder from that handle (F29), checks the descriptor's `fdinfo` mount ID against
  `user-cache`'s row and the folder's identity against the record, completes the unmount and the unbind, and only
  then allows the folder's removal. Control: an undisturbed removal reaches M1's positive arm.
- The handle against the by-name route: the same kill, then two sweeps from fresh processes. Assert the handle sweep
  unmounts the clone and then the bind, and that the by-name sweep does neither and records `EBUSY` from its first
  call, with its descriptor's `fdinfo` mount ID equal to the clone's row (F10, F28). Control: the handle sweep's
  success in the same run, so the arm shows the handle route does what the name route cannot.
- A substituted folder: the account removes the folder and makes a new one of the same name before the sweep. Assert
  `open_by_handle_at` refuses `ESTALE` (F29), the sweep unmounts nothing, the retained staged mount and the claim are
  reported, and `doctor` prints them. Control: the undisturbed handle resolves in the same run.
- Kill after the unmount, before the folder removal. Assert the next pass removes the claimed folder only after a
  clear receipt for the last scan and after its mount-namespace identity matches the handler's recorded one, and
  prints a notice. Control: the same pass with the receipt withheld keeps the folder and reports it.
- Namespace created while staged: a reader namespace enters while the clone is staged, then the wrapper's cleanup
  runs. Assert first that the scan names the reader's copy and the wrapper removes nothing (the reader's rows are the
  assertion), then that after the reader exits and a clear receipt arrives the wrapper removes the folder and the
  reader's namespace holds no row for it. Controls: a scan with no reader is clear and the folder is removed at once;
  a wrapper run in a namespace whose `/proc/self/ns/mnt` identity differs from the handler's recorded one refuses
  and removes nothing.
- Kept folders: a folder holding a foreign file is kept and reported; a folder with no staging claim is not removed;
  a folder whose recorded identity no longer matches is kept and reported. Control: the same three folders with
  nothing wrong in them, which must each be removed by `rmdir`, so a sweep that keeps every folder fails the arm.

### M7. Two removals at once

- Two folders, each with its own bind: assert each receipt carries its own nonce and matches the claim that records
  it, each unmount removes only its own clone (the other's source row remains in the host namespace), and both
  folders are gone. Control: the serial case, one removal after the other.
- Way C's shape: two folders inside one boot-long private bind; assert both clones are visible at their own folders
  at the same time, each unmount leaves the other, and no second move covers the first. Control: the same run with
  one removal.

### M8. A root-made target

- Root creates the target, and the account then renames it, removes it and replaces it (from a foreign namespace
  where the host refuses with `EBUSY`), and the handler is asked to proceed. Assert the identity check refuses each
  time, no row for the clone's dataset appears, and `doctor` names the state. Control: the undisturbed target
  proceeds, and the account's write inside a `0700` root-owned target is refused with a positive control that the
  account can write beside it.
- Source arm for Way E: assert the bind's row's source is root's empty directory and its `sb_dev_major:minor` is the
  directory's file system. Control: Way A's arm, whose source is the account's folder.

### M9. The refused namespace teardown

- Run in the guest's boot namespace, where the unit that attaches runs, so the clone the child reaches for is the
  one the host actually holds. A child unshares a mount namespace and then tries `move_mount` of the host's clone
  into it. Assert the move fails `EINVAL` (F25), the clone's row is still in the boot namespace, and record the
  errno. Control: the same move inside one namespace (M2's covered-folder route) succeeds.

Each live way needs M1, M2, M3, M5 and every arm of M6. Way A additionally needs M4's positive, host-namespace,
copied-idle, copied-held, foreign-namespace and renamed arms, and M7's two-folder arm. Way B needs Way A's handler
arms plus M7 and M8. Way C needs M1, M2, M3, M5, M6, M4's renamed arm and M7's Way C arm. Way D needs M1, M2, M3's
propagation arms on `~/.cache`, M5, M6 and M7. Way E needs Way A's arms and M8's source arm. Way F needs Way A's
arms and M4's renamed arm. No live way needs M9, which exists only to record the refused way's failure.

## Questions for the owner

The plan calls the item writer design work, so these are the points where a writer would otherwise have to add a
decision the accepted design did not make, or would have to reverse one. Each question stands alone: it defines the
words it uses, gives one concrete scene, lists exclusive options with their pros and cons, and ends with one ranking
statement that gives the reason for every adjacent pair. No answer to one question is needed to answer another, so
they may be answered in any order.

What this design does not ask, because the records or the kernel settle it: the unmount route is the covered-folder
descriptor (F7, F8, F10) plus, for a later sweep, the handle the handler recorded before the bind (F29), and both
meet the decided sentence and the plan's forbidden line without a name inside the account's tree; the propagation
change runs on the bind's own descriptor, taken by one by-name lookup whose mount ID is checked (F14, F28); the
namespace teardown is refused (F25); and refusing every unmount contradicts the "B: Destroy at once" answer the
project gave.

### Q1. Which shape should the staging target take?

The scene, in plain words. A person runs the removal of a worktree whose clone cannot be unmounted at the path Git
records, because a process still works inside it. Under the first option the wrapper creates
`~/.cache/monochromatic-git-policy-cli-staging-4f3c1a8e5b2d47c9a06e1f3b8d5c2a74`, the root handler attaches a private
mount on that folder and moves the clone onto it, and once the process has gone the handler unmounts the clone and
the private mount, and the wrapper removes the folder. While the state is not clean, `doctor` prints one line per
removal:

```text
staged: user-cache/example-repo/example-clone at monochromatic-git-policy-cli-staging-4f3c1a8e..., mount private,
clone mounted, held by pid 4211
```

The words this question uses. A staging folder is a directory under `~/.cache` that holds a mount for one removal. A
claimed folder is one the wrapper wrote a claim record for before creating it, so a later pass can prove the folder
is the one it made. A bind is a second mount of a directory onto itself, which gives the handler a mount of its own.
A private mount is one the kernel copies into no other mount namespace, which the mount table shows by printing no
`shared:`, `master:` or `unbindable` tag. A foreign namespace is a mount namespace the account created for itself, in
which a mount the host holds can be renamed or detached without the host's permission. A copy is the same mount as
another mount namespace holds it, made by propagation. The stop pass is the boot unit's shutdown step that unmounts
the managed mounts. A receipt is the root-owned file the handler writes for one nonce, naming the outcome and the
mount.

Each option is a complete design, so this question covers who creates the folder and how long the mount lives as
well:

1.  "A claimed folder per removal, with a bind that lives one removal (Recommended)": Way A. The wrapper creates and
    claims the folder, the handler binds it and makes the bind private, and the unbind ends the bind.
    Pros: nothing existing changes; the folder has no renamable ancestor in the host namespace; one removal's failure
    cannot touch another's folder; the recorded handle recovers an interrupted removal (F29).
    Cons: a mount transition at each removal; the bind is copied into peer and slave namespaces (F22); one by-name
    lookup is needed to reach the bind (F28); a copy a process works inside refuses the unbind (F30); a third-party
    recursive delete of `~/.cache` reaches a staged clone (F26).
2.  "A plain folder under `~/.cache` made private once at boot, and no bind": Way D. The boot unit makes `~/.cache`
    private and records the mount; each removal creates a plain folder and moves its clone there; no bind exists.
    Pros: no bind, so no copy in any namespace, no unbind to refuse and one unmount call on a folder descriptor.
    Cons: it changes an existing mount the proposal leaves outside itself; late mounts under `~/.cache` stop reaching
    confined readers; a replacement mount in the middle of a boot is shared again and nothing re-applies the setting
    (F1, F6, `cmd/zfs/zfs_main.c:7380` to `:7388`).
3.  "A root-made folder per removal, bound for that removal": Way B. Root creates the folder mode `0700` owned by
    root, records its identity under `/var/lib/monochromatic-git-policy-cli/`, binds it and makes it private, and the
    wrapper removes the folder after the clear receipt.
    Pros: the identity comes from root's own record, and the account cannot plant content in the folder.
    Cons: root writes a directory into the account's tree, which the writer's reading of the proposal's slot rule
    avoids; the folder is still removable or renamable from a foreign namespace (F11); the wrapper removes an object
    it did not create.
4.  "One folder made and bound at boot, with each removal's folder inside it": Way C. The boot unit creates one
    folder, binds it private and keeps it until the stop pass; each removal creates its folder inside the bind.
    Pros: no mount transition on a removal's path.
    Cons: a host-namespace bind for a whole boot, with a copy in every peer and slave namespace; a foreign-namespace
    `rmdir` of any removal folder detaches the bind and every staged mount (F11); the stop pass needs an anchor for
    the bind that it does not keep (Q2); the bind's cleanup leaves the wrapper the owner named.
5.  "A root-owned empty directory bound onto an account-made folder": Way E. Root owns an empty directory, binds it
    onto the wrapper's folder, and the unbind and cleanup are the first option's.
    Pros: the mount's source is root's, so the folder's content side is not the account's.
    Cons: the folder holds no content (A6), so that protection covers nothing; the mountpoint is still account-made
    and removable from a foreign namespace (F11); it adds a root-owned object of the shape the project vetoed at
    `/run`.
6.  "One claimed staging root two levels deep, with a folder per removal": Way F. The wrapper creates
    `~/.cache/monochromatic-git-policy-cli/stage` once, and each removal's folder under it.
    Pros: one printed path for `doctor`, and one identity to record.
    Cons: the root's parent is an ordinary directory the account can rename in the host namespace, which carries the
    bind and the staged clone to a new path; a durable record of the root's identity then refuses every unmount
    until an administrator re-records it after a legitimate recreation of `~/.cache`.

Ranking: option one > option two > option three > option four > option five > option six. Option one beats option two
because option two pays for the removed bind with the propagation of an existing mount the proposal leaves outside
itself and with late-mount delivery under `~/.cache`, while option one's costs are one transition per removal and the
recovery anchor. Option two beats option three because option two has no privileged write into the account's tree and
no root-owned object under an account-owned parent, and option three's content protection covers a folder that holds
no content (A6). Option three beats option four because option three's object lives one removal, while option four
keeps a host-namespace bind for a whole boot and hands its cleanup to the boot unit. Option four beats option five
because option four's folder is account-made and needs no root-owned source, while option five adds a root-owned
directory to protect content the folder never holds. Option five beats option six because option five at least changes
what the mount's source is, while option six adds a renamable ancestor and an adoption rule for one printed path.

### Q2. What may clear a retained staged mount?

The scene, in plain words. A handler attached the bind and moved the clone, then died. The next removal's sweep
could not finish: the handle it recorded no longer resolves, or its process had no `CAP_DAC_READ_SEARCH`, or a copy
of the bind in another mount namespace is held. The mount, and possibly the clone, is still there, the dataset stays
busy, that removal cannot finish, and `doctor` prints on every run:

```text
staged, retained: source user-cache/example-repo/example-clone, mount 41, path unknown, bind private,
copy in mnt:[4026534] held by pid 4211, next safe step: an administrator unmounts it
```

The words this question uses. A staged mount is a clone's mount that was moved onto a staging folder under
`~/.cache`. A retained staged mount is one that outlived the handler invocation that made it, so no process holds a
descriptor for its folder. A sweep is a later attempt, run for a removal's `unmount` nonce, to finish what an earlier
invocation left. A copy is the same mount as another mount namespace holds it, made by propagation. A foreign
namespace is a mount namespace the account made for itself, in which a mount the host holds can be detached without
the host's permission. The administrator is the person with root on the machine.

1.  "Report it and wait for the administrator (Recommended)": `doctor` names the mount, its dataset and the holders,
    nothing acts on it, and that removal refuses until it is gone.
    Pros: the decided sentences stay whole, with no lazy detach and no path unmount anywhere; the next boot clears the
    state, because a mount does not survive one.
    Cons: the account can keep a removal blocked until a restart by holding a copy, and the dataset stays busy
    meanwhile.
2.  "Keep one root process holding each staged folder's descriptor": a boot-long supervisor keeps one descriptor per
    staged folder, so an unmount still runs on a descriptor after any rename (F7).
    Pros: a sweep never needs the handle, and the unmount finishes without an administrator.
    Cons: a new boot-long privileged process with per-folder state; its own crash leaves option one, and the state it
    serves is one a restart clears anyway.
3.  "Detach a stranded bind once at boot": the boot unit lazily detaches a bind whose folder no longer resolves, and
    never a clone's mount.
    Pros: no reboot is needed for the bind, and no path is unmounted.
    Cons: a lazy detach is what the decided sentence excludes, so it needs the owner's word and a proof it cannot hide
    a clone's references; it also repairs silently what `doctor` would report.
4.  "Unmount the path the mount table prints, after re-verifying identity": read the path `mountinfo` or `statmount`
    prints for the recorded mount, verify its source, parent and root, then unmount that path.
    Pros: nothing new to keep, and the removal finishes.
    Cons: a path-based unmount on the account's tree, which the decided sentence and the plan's forbidden line forbid;
    only the owner can accept it.

Ranking: option one > option two > option three > option four. Option one beats option two because option two adds a
boot-long privileged process and per-folder descriptor state for a state a restart already clears, and the crash that
creates the state kills that process too. Option two beats option three because option two keeps the decided "never
lazily detach" sentence and needs no proof, while option three adds a privileged lazy detach and asks for one. Option
three beats option four because option three touches only the bind, while option four unmounts whatever the path
names and breaks the decided letter for every mount it reaches.

### Q3. What may the handler do with a staged mount the post-attach check cannot find at the folder?

The scene, in plain words. The clone's mount was moved onto the staging folder, and the handler's check then does not
find it at the folder's verified dentry, because the account removed and re-made the folder name in the window
between the move and the check. The clone is mounted somewhere the handler cannot prove, the dataset stays busy, and
`doctor` prints:

```text
staged, unconfirmed: source user-cache/example-repo/example-clone, folder identity changed, next safe step: an
administrator checks the mount
```

The words this question uses. A staged mount is a clone's mount that was moved onto a staging folder under `~/.cache`.
The post-attach check is the handler's read of the mount table right after the move, which must find the clone's
mount on the folder it verified. A verified dentry is the directory object the handler opened and recorded before the
bind. The Misplaced attach rollback is the accepted design's rule for a clone found mounted somewhere other than its
slot. Reattach is the busy-unmount decision's rule that moves a mount back to its slot after revalidation. The
administrator is the person with root on the machine.

1.  "Keep it where it is and report it (Recommended)": the handler records its identity and reports it with the next
    safe step, and changes nothing.
    Pros: no privileged move to a dentry the account may have substituted; the state stays visible to `doctor`;
    nothing new is attached.
    Cons: the mount stays somewhere unproven, the dataset stays busy, and the removal cannot finish until an
    administrator acts.
2.  "Move it to the folder's current verified dentry": the staging analogue of the Misplaced attach rollback.
    Pros: the removal can continue, and the rollback at once is the closest recorded precedent.
    Cons: that rule was written for a slot, this dentry is one the account may have substituted, and a second
    privileged move adds another window.
3.  "Reattach it to the slot": the busy-unmount decision's reattach, applied to a mount the check cannot find.
    Pros: it restores the worktree's visibility at the path Git records.
    Cons: that decision requires revalidating the slot and a second attach; nothing measured either one, and this is
    a case the decision did not consider.

Ranking: option one > option two > option three. Option one beats option two because option two moves a privileged
mount to a dentry the account may control, while option one keeps the proven state and asks an administrator. Option
two beats option three because option two stays inside the staging design, while option three adds a second attach at
the slot, on an unmeasured path, to a case the busy decision did not consider.

### Q4. What bound applies to a staged mount that outlives its removal?

The scene, in plain words. A clone's mount stayed on its staging folder after its removal gave up, because a process
works inside it. `doctor` prints the retained line of Q2 on every run, the dataset stays busy, and nothing in this
design makes that state's age visible beyond that line.

The words this question uses. A staged mount is a clone's mount that was moved onto a staging folder under `~/.cache`.
A retained staged mount is one that outlived its removal and is reported rather than cleared. `doctor` is the
read-only health report the wrapper runs, and it changes nothing.

1.  "Report it and refuse nothing (Recommended)": `doctor` names the mount, its dataset, the age of the boot's record
    for it and the next safe step, and no command acts on it.
    Pros: nothing new is refused and nothing is repaired behind the wrapper's back; the state is visible.
    Cons: it can sit for a long time, and a third-party delete of `~/.cache` can reach it (F26, L9).
2.  "Stage nothing new in that repository while one is retained": a removal of another clone in the same repository
    refuses to stage, leaves its clone at its slot and reports why, while the retained removal's own retries
    continue.
    Pros: retained staged mounts cannot accumulate in one repository, and the recovery of the retained one is not
    blocked.
    Cons: an unrelated removal is refused, and its remedy is the same administrator action.
3.  "A time bound, after which `doctor` turns it into a failure and removals refuse": an interval recorded in root
    configuration.
    Pros: the age is bounded and visible to `doctor`.
    Cons: nothing measured sizes the interval (L7), and the design may not ship a constant nobody measured.

Ranking: option one > option two > option three. Option one beats option two because option two refuses an unrelated
removal without releasing anything, so the administrator's action is the same either way. Option two beats option
three because option two needs no unmeasured constant, and option three would ship one.

## The staging claim

The existing claim pattern admits only its own container shape, and its settlement deletes recursively through
whatever is mounted (F16). A staging claim is therefore a kind of its own, with every one of its parts:

- Its own place: `<common>/monochromatic-git-policy-cli-worktree-copy/staging/`, a private directory beside `v1`
  under the same private parent. It cannot live inside `v1`: the forwarding check counts every entry there except the
  unreadable folder as pending work, so a subdirectory would make every forwarded command in that repository run the
  recovery pass, and a directory beside `v1` is listed by no pass (F27).
- Its own suffix: `<hex>.staged`, where the hex part is `32` lowercase digits, so no reader of `.stage` or
  `.json` ever sees it (F27).
- Its own reader: a staging sweep that understands only this shape, and that keeps and reports a claim it cannot
  read instead of moving it aside, because an unreadable claim may name a folder whose state is unproven. It runs
  from a removal, for that repository's own claims, and from `doctor`'s read of them; no forwarded command pays for
  it.
- Its own settlement: `rmdir` of the folder, and only after the wrapper's mount-namespace identity matches the
  handler's recorded one, after the clear receipt for the nonce the claim records arrived, and after the recorded
  identity of the folder still matches. It never changes a mode, never deletes recursively, and never calls the stage
  removal helper.

The claim carries the folder's component, the nonce of the `unmount` intent it is waiting for, the owner token and the
folder's recorded identity (device, inode, birth time, owner, mode). A retry that writes a new nonce for the same
removal keeps the standing folder and updates the claim's nonce before it writes the intent, so the receipt's nonce
always matches the claim's own and a receipt never has to name a folder (F15). A new folder is made only after the
standing one is removed and its claim released. An intent whose folder was never made leaves a claim whose folder is
absent, and the sweep releases that claim with a notice.

The accepted text says two things about names. "No request supplies a path, dataset name or mount flags"
([proposal][proposal], "Requests, results and ordering"), and "Nonces appear only as validated data in the receipt,
never in a filename" (same section). The folder's component is therefore the wrapper's own draw: `32` lowercase hex
digits from the wrapper's randomness, written into the claim before the folder exists, with the intent's nonce in the
claim as data and in no name at all. Root checks the component against that grammar, and the claim's nonce against
the intent's grammar, before it uses either.

One limit stays, and it is per repository: a claim lives in a repository's Git common directory, so a repository
whose common directory is gone takes its claims with it, and a folder it left under `~/.cache` has no readable claim.
`doctor` finds such an orphan by listing `~/.cache` by descriptor and matching every entry of the folder grammar
against the claims of the repositories the wrapper holds durable binding records for (the records step 10 writes);
an entry that matches none is printed as an orphan, and nothing removes it automatically, because the wrapper cannot
prove it made it.

The device-free test this owes: in a private user and mount namespace (no ZFS device, no pool), bind a directory
holding a sentinel file onto a claimed folder, write a staging claim whose owner is dead, then run the existing
recovery pass and the staging sweep. Assert the recovery pass returns success, the sentinel's directory keeps its
mode, the sentinel survives, and the staging sweep keeps and reports the claim. A pass that aborts, changes a mode or
deletes a byte fails the test. The control is the same fixture with the mount replaced by a plain directory, where
the staging sweep must remove the folder by `rmdir` and the recovery pass must still return success.

## What a writer can settle without asking

- The folder grammar under `~/.cache`: one readable component that says what it holds, at the mode F1 records, with
  the `32` hex digits the claim records and no other variable part.
- The unmount and unbind route: the covered-folder descriptor and the two `umount2` calls (F7, F8, F10, F14), the
  by-name open that reaches the bind for the propagation change (F28), and the recorded handle a later sweep opens
  the folder with (F29).
- The order inside a removal: record the handle and the namespace pair, bind, take the bind descriptor and make it
  private, move, verify by mount ID with its source and parent, unmount the clone, unbind, scan every namespace,
  write the receipt, then the wrapper's `rmdir` after the clear receipt.
- The wrapper's mount-namespace identity check before any folder removal, comparing `/proc/self/ns/mnt`'s device and
  inode with the handler's recorded pair in the boot's bind record, and refusing on a mismatch (F11, A1).
- That a later `unmount` intent's staging sweep completes an interrupted removal's unmount and unbind from the
  recorded handle (F29), re-verifying the folder's identity and the mount table's rows before it acts, and that
  anything the handle cannot serve stays staged and reported (Q2).
- The identity fields checked before each move and before each removal: device, inode, birth time, owner, mode,
  access and default ACLs from the held descriptor, plus the mount's source and parent (F17, F18, F21).
- The handler's state records: its source dataset GUID and the identity of the folder it covers in a durable record
  under `/var/lib/monochromatic-git-policy-cli/` beside the slot identities, and the bind's mount ID, parent, the
  folder's handle and its own mount-namespace pair in the current boot's bind record under
  `/run/git-worktree-clone-mount-results/staging/`, because no durable record may key anything on a mount ID (A1,
  L10).
- The receipt and notice wording for a retained staged mount, a mounted folder that cannot be unmounted, a handle
  that no longer resolves, a copy that refused the unbind, a detached mount that still holds the dataset, an orphan
  folder, and a folder the wrapper cannot prove it made.
- That cleanup is `rmdir`-only and never recursive, that it stops at any mount, and that it runs from the staging
  sweep keyed to a dead owner rather than from a timer.
- That the wrapper never removes a folder while a mount covers it in the host namespace, and that the handler never
  removes a folder it did not bind.
- That the unmount, bind and unbind waits share the removal deadline step 9 introduces and step 22 sizes (F20).
- That `doctor` prints the staging folder, its live bind's propagation, any staged mount with its dataset and age and
  the next safe step, and that it changes nothing, as the `doctor` decision quoted under "Decisions of the accepted
  design that bound this proposal" states.
- That the staging claims live in the place and the shape "The staging claim" names.

## Proposals the owner may veto

- Pinning the `user-cache` mount: the handler records the dataset's source and the mount root's device and inode,
  and refuses to create a bind when the mount at `~/.cache` is not that source. The pin is one key of the
  root-controlled `/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc` the proposal keeps root's
  ([proposal][proposal], "ZED reconciliation and mount safety"); the handler reads it before every bind, the boot
  reconcile reads it before it verifies staging, and the wrapper reads it before it creates a folder. This is new
  root configuration the accepted design does not pin, and it is what makes F12's ordering checkable rather than
  assumed. The proposal owes the boot-time shape of the same check ("Test staging setup at boot and refuse shared or
  replaced staging and checkout mounts until a private setup is verified"), and this design meets it by verifying the
  pin at boot reconcile and by verifying each removal's bind and the checkout's propagation before each move.
- Reading the staging location from the Pinned UID's passwd home (A2), rather than from an environment variable a root
  handler does not have.

## How the accepted sentences are met

- "Reuse only a verified staging mount on subsequent sweeps" has no work to do for a completed removal under Way A,
  which binds, verifies and unbinds within one invocation. It does bite for an interrupted one: a bind, and possibly
  a staged clone, outlives that invocation (M6), so the staging sweep opens the folder from the recorded handle
  (F29), re-verifies the folder's identity and the mount table's rows for the bind's source and parent before it
  acts, and reports instead when the handle no longer resolves (Q2). Way C, which keeps a bind across sweeps by
  design, verifies it before each move.
- "Test staging setup at boot and refuse shared or replaced staging and checkout mounts until a private setup is
  verified" is met twice: at boot reconcile, the pin of "Proposals the owner may veto" is checked; at each removal,
  the handler refuses when the checkout's propagation is not private (the move would fail anyway, F5) or when the
  bind it made does not read private in the mount table (F23). The spec's wording asks for a boot test; this design
  adds the per-removal test beside it and says which sentence it strengthens rather than replaces.
- "Unmounted" as absence in every namespace holding the source, including binds and staging, is met by the receipt's
  all-namespace scan and by the wrapper's `rmdir` only after that receipt shows absence (A5).
- The acceptance item's two halves are met by M1 and M6: after a completed removal no mount and no entry remain;
  after an interrupted one, nothing under a mounted tree is deleted and the retained mount is tracked and reported.

## What this proposal does not do

- It adds no decision the accepted design did not make. The ways, the ranking and the questions are proposals, and
  none of them is adopted here. It proposes root configuration (the pin) and a location rule (the passwd home), and
  marks both for veto.
- It runs no VM measurement and no host probe beyond read-only reads of `findmnt`, `stat`, `zfs list`, `zfs get`
  and `rm --help`.
- It does not design the removal workflow, the mount handler's modes, the boot unit's ordering, the ABI gate, the
  all-namespace scan or the deferred-origin checks.
- It does not change the measurements record, the proposal, the plan or any wrapper code.

## Revisions

Each revision is named by the commit that carries it, with that commit's committer timestamp
(`git log --format=%cI`). The last entry's values were filled in by the commit that follows it, because a commit's
own hash and timestamp cannot appear in the commit that creates them.

- `a709cd7f58` at `2026-11-19T18:02:34-05:00`: the first complete design, with its facts, ways, questions and
  measurement arms.
- `2258de74d8` at `2026-11-19T18:03:18-05:00`: names the plan's statement that the steps after step 9 wait on this
  design.
- `ff0a4019d9` at `2026-11-19T18:53:06-05:00`: the first revision, answering the gate review with a
  descriptor-anchored unmount and the questions split by creator and lifetime.
- `3972d4bf90` at `2026-11-19T18:53:29-05:00`: records the host's user-namespace knobs the design read.
- `HASH` at `WHEN`: this revision, answering the re-check: the recorded handle a later sweep recovers the folder
  with,
  the bind's own descriptor, the unbind's rule over the copies, the claim's place beside `v1`, the corrected
  mount-source mechanism, and the questions rebuilt as one choice among the ways plus three follow-ups.

## Sources read

- Linux, upstream tag `v6.8`, files fetched by URL at that tag: `fs/namespace.c`, `fs/namei.c`, `fs/pnode.c`,
  `fs/open.c`, `fs/dcache.c`, `fs/proc/fd.c`, `fs/proc/base.c`, `fs/proc_namespace.c`, `include/linux/namei.h`,
  `include/uapi/linux/mount.h`. `fs/fhandle.c` and `fs/dcache.c` were fetched at the same tag into the re-check's
  notes folder and read there.
- OpenZFS, tag `zfs-2.4.4`, commit `71a9f9578616a90c3c14bb59629fb4d31bfd68d1`:
  `module/os/linux/zfs/zpl_super.c`, `module/os/linux/zfs/zfs_vfsops.c`, `module/os/linux/zfs/zpl_export.c`,
  `module/zfs/dsl_dataset.c`, `module/zfs/dsl_destroy.c`, `cmd/zfs/zfs_main.c`.
- This repository at `73eaad0846`: [decision][decision], [proposal][proposal], [measurements][measurements],
  [plan][plan], and `AGENTS.md`.
- Wrapper code: `package/git-policy/cli/src/native/subvolume/fs_beneath.rs`,
  `package/git-policy/cli/src/native/process/open_file_of_a_descriptor.rs`,
  `package/git-policy/cli/src/native/process/process_use.rs`,
  `package/git-policy/cli/src/native/worktree_copying/worktree_copy_stage_claim.rs`,
  `package/git-policy/cli/src/native/worktree_copying/worktree_copy_stage_claim_reaping.rs`,
  `package/git-policy/cli/src/native/worktree_copying/worktree_copy_stage_claim_swept_files.rs`,
  `package/git-policy/cli/src/native/worktree_copying/worktree_copy_stage.rs`,
  `package/git-policy/cli/src/native/worktree_copying/worktree_copy_selection.rs`,
  `package/git-policy/cli/src/native/worktree_copying/worktree_copy_journal.rs`,
  `package/git-policy/cli/src/native/worktree_copying/worktree_copy_journal_removal.rs`,
  `package/git-policy/cli/src/native/worktree_copying/worktree_copy_recovery.rs`,
  `package/git-policy/cli/src/native/worktree_copying/worktree_copy_unreadable_records.rs`,
  `package/git-policy/cli/src/native/service/wait_ceilings.rs`.
- The measurement harness items 3 and 4 ran: `guest-probe.py` in
  `~/temp/agent/zfs-measurements-owed-before-acceptance-2026-11-18/keep/`, read for the staging bind's creation and
  the unmount calls.
- This host, read-only, 2026-11-19: `findmnt --output TARGET,SOURCE,FSTYPE,PROPAGATION --target /home/user/.cache`,
  `stat --format '%n %i %U %a' /home/user/.cache`, `stat --format '%n %A %U' /usr/bin/fusermount3 /usr/bin/fusermount`,
  `rm --help`,
  `zfs list -H -o name,mounted,mountpoint,canmount,com.sun:auto-snapshot zpcachyos/nosnap/user-cache`,
  `zfs get -H -o property,value,source mountpoint,canmount,mounted zpcachyos/nosnap`.

## Links

[decision]: ../decision/cli-git-zfs-worktree-design.md
[proposal]: cli-git-zfs-worktree-design-proposal.md
[measurements]: cli-git-zfs-measurements-owed-before-acceptance.md
[plan]: cli-git-zfs-worktree-implementation-plan.md
[decided-defaults]: cli-git-zfs-worktree-design-proposal.md#decided-defaults
