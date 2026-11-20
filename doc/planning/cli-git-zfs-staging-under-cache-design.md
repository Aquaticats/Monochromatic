# Staging under `~/.cache` and the wrapper's cleanup

Non-final, unapproved design proposal by an agent. It is not approved by the project owner and nothing in it is
final. It designs the open item "Staging under `~/.cache` and the wrapper's cleanup" of the accepted ZFS worktree
design, and it makes no decision that design or the project's answers of 2026-11-19 did not make. It ran no ZFS
command that changes anything, started no VM and no container, touched no pool, and edited no other file.

Every kernel fact here is read at upstream Linux `v6.8`, in the files listed under "Sources read", and every
OpenZFS fact is read at tag `zfs-2.4.4`. Nothing in this design was measured, on any host or any guest: every
measurement it owes stays owed, and no sentence here reports a run.

This revision answers three independent reviews, all outside this repository, and then applies the project's answers
of 2026-11-19. The reviews are the gate review of tip `2258de74d8`, its first re-check of tip `3972d4bf90`, and the
second re-check of tip `a879cd9be8`, under
`~/temp/agent/zfs-staging-under-cache-design-gate-review-2026-11-19/keep/`,
`~/temp/agent/zfs-staging-under-cache-design-revision-gate-recheck-2026-11-19/keep/` and
`~/temp/agent/zfs-staging-under-cache-design-second-revision-gate-recheck-2026-11-19/keep/`. No review's finding
numbers are used here: every reference is to a fact, an assumption, a limit or a measurement arm of this design.
The reviews' points stay where they apply: the folder's name derived on both sides from the clone's dataset GUID in
"The staging target and who creates it", the staged-folder record in A1 and "The staged-folder record", the recovery
capability in "The unmount call from staging" and M6, the retained states split by cause in F11 and L11, and the
corrected descriptor facts in F14 and F30. The project's answers of 2026-11-19, and the threat model they state, are
listed under "Answers of 2026-11-19" and followed throughout. The revision history, with each commit's committer
timestamp from `git log --format=%cI`, is under "Revisions".

The design is [the decision record][decision], the whole design is [the proposal][proposal], the evidence is
[the measurement record][measurements], and the build order is [the implementation plan][plan]. The plan lists the
item as step 9's blocker, "blocking for every unmount", and as writer design work
([plan][plan], "Open items and the step that needs each"), and step 9 records that it "also owes the interface that
unmounts a staged mount from the held descriptor". Steps 14, 16 and 17 wait on it as well
([plan][plan], step 9, "Open items"), and the plan's "What can start now" puts "step 9 and steps 14 to 23" behind
step 9's design.

## Answers of 2026-11-19

The design put three questions and a list of veto-open proposals to the project, in the question tool between
`2026-11-19T20:35` and `2026-11-19T20:43:51-05:00`. The record is `doc/handover/cli-git-rust-implementation.md` in
the main checkout, the paragraph beginning `THE ROUND WAS ASKED`; that paragraph is uncommitted there, so this
branch's copy of the file does not carry it. The answers, in the project's words:

- Where a clone's mount waits, and who creates the folder: "Private bind per removal (Recommended)". Way A is
  therefore the design: the wrapper creates one folder per removal, the handler binds it, makes the bind private and
  moves the clone onto it, and the unbind ends the bind ("The staging target and who creates it", "The private bind",
  "Ways to meet it").
- How a later sweep finishes an unmount the handler left behind: "Record the file handle (Recommended)". The handler
  writes the folder's file handle before the first mount, and a sweep opens the folder from that record and unmounts
  ("The staged-folder record", "The unmount call from staging", M6).
- What the handler does when the move lands on a mount the account stacked: "Move it back at once (Recommended)",
  answered after "What?" to the same question and a second asking in plain words. The handler rolls the exact mount
  back to its slot by its own descriptor, and keeps it staged only if the rollback fails ("The private bind", M1).
- The proposals left open to veto: "Veto none (Recommended)", for the `~/.cache` lookup from the pinned UID's passwd
  home, for the `user-cache` pin, and for the two readings of a `/proc`-path argument, that a `umount2` on it acts on
  the held descriptor and that a `mount(2)` on it is not path-based. Each is now the design ("Proposals the owner
  settled").
- Where the handle record lives: "In ~/.cache". The design had placed it in
  `/var/lib/monochromatic-git-policy-cli/`; the project asked why a durable root record was needed when it need not
  survive a reboot, was told that a reboot drops every mount and was offered a per-boot root file in `/run`, asked
  why not `~/.cache`, was told that any program running as the account could swap a handle there and steer a root
  unmount, and answered with the sentence that follows.

The threat model that answer states, and that this design now designs to, is the project's own sentence for this
machine: "We have to assume my machine isn't compromised. sudo is passwordless here anyway." The account is
trusted, and root and the account are not a security boundary here. Where the design argues about what the account
could do, it says whether the argument survives that sentence, and "Hardening the stated threat model may make
unnecessary" lists every check that exists only because a draft of this design assumed otherwise. Those entries are
suggestions for the project to decide: none is dropped on the strength of the sentence alone.

The design follows every answer, and each place says so: the record's place and name in "The staged-folder
record", the binding route in "The private bind" and "The unmount call from staging", the rollback in "The private
bind", the arms in "Owed measurements in a throwaway VM", and the settled proposals in "Proposals the owner
settled". Where an answer departs from a sentence of the accepted design, that sentence is quoted beside the
departure: the record as a cache that a sweep could rebuild is the one departure, in "The staged-folder record".

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

The round of 2026-11-19 answered what the item left undesigned. The wrapper creates one folder per removal, the
handler binds it and makes the bind private for that removal, a later sweep finishes an interrupted removal by
opening the folder from the recorded file handle, and a clone the move lands on a stacked mount goes back to its slot
at once. "Answers of 2026-11-19" lists the questions with the project's own words, and "Questions the owner answered"
records what each answer left untaken.

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
- No unmount exists yet ([proposal][proposal], "Descriptor mount and unmount"): "The interface that unmount from
  staging needs, after the move and from that descriptor, has no design yet, and no unmount runs from staging until
  it exists."
- Staging reuse across sweeps ([proposal][proposal], "Descriptor mount and unmount"): "Reuse only a verified staging
  mount on subsequent sweeps; validate its identity and private propagation before each move."
- Staged state is not absence ([proposal][proposal], "Descriptor mount and unmount"): "Track and report staged
  identity and location across interruption: absence at the slot does not mean the mount is gone. Treat absence from
  the handler's namespace as local detach, not completed unmount."
- Private staging and private parent ([proposal][proposal], "Mounts held by other namespaces"): "This descriptor
  route requires a verified private immediate parent and verified private staging, not private ancestors of the
  checkout", and "Private staging does not eliminate the all-namespace check" ([proposal][proposal], "Project
  decisions", the checkout propagation bullet, lines 1245 to 1246 of that file).
- Staging counts as a live mount ([proposal][proposal], "Mounts held by other namespaces"): the design defines
  "Unmounted" as absence "in every mount namespace holding the clone's ZFS source, including binds and staging", and
  "a missing or mismatched unmount still blocks removal outright".
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
  live in `/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities`. The staged-folder record is
  not one of those, and by the project's answer it lives in `~/.cache` instead ("The staged-folder record").

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
  a namespace that does not hold the mount removes the name, and every namespace loses the mounts on it: no mount
  row is left at that name anywhere, and what a holder keeps alive is a detached mount with no mountpoint (L11).
  The handler runs in the host namespace ([proposal][proposal], "ZED reconciliation and mount safety"), and the
  account can create a mount namespace of its own (F24).
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
  `AT_EMPTY_PATH`) (Linux `v6.8`, `fs/namespace.c:2692` to `:2741`, `:4071` to `:4115`, `:3943`, `:4647` to `:4693`;
  `include/uapi/linux/mount.h`). `AT_EMPTY_PATH` adds `LOOKUP_EMPTY` and nothing else (`fs/namei.c:192` to `:200`,
  `:208` to `:214`), and an empty name keeps the descriptor's own mount and dentry (`:2362` to `:2363`, `:2418` to
  `:2441`), so no lookup crosses a mount. Which mount each call then acts on differs by call, and only two of them
  keep the covered mount. `open_tree` reads the descriptor as its source and `mount_setattr` changes the mount its
  path names, so a descriptor opened before a bind keeps the covered mount, and those two act on it rather than on
  the mount stacked over it. The destination of `move_mount` and the target of `mount(2)` are the other way round:
  `do_lock_mount` follows the mount stack from the given path until the top mount is found (`fs/namespace.c:2409` to
  `:2490`, called by `do_move_mount` at `:3127` and by `path_mount` through `lock_mount` at `:2500` to `:2502`), so a
  mount the account stacked on the destination receives the moved mount on top of itself. `fsmount` takes a
  filesystem context descriptor and no path (`:3943`), so it names no mount at all. `mount_setattr` and `mount` with
  `MS_PRIVATE` change propagation only for a path that is the root of its own mount: `do_mount_setattr` refuses
  `EINVAL` otherwise (`:4433` to `:4439`, and `change_mnt_propagation` at `:4426` is what its commit calls), and
  `do_change_type` likewise (`:2549` to `:2573`). The folder is not the root of the `user-cache` mount, so the bind's
  own descriptor is the only thing that can carry the propagation change, and F28 is how the design obtains it.
- F15. A receipt names the operation, the matching nonce, the outcome and the mount identity, and the request
  grammar carries no path, dataset name or mount flag ([proposal][proposal], "Requests, results and ordering"). The
  mount identity is the mount's own, so no receipt carries a path or a folder name, and the staging claim a removal
  writes is what ties one of its nonces to one folder. The folder's name is not in the receipt either: it derives
  from the clone's dataset GUID, which both sides hold (F33).
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
  refuses `EOPNOTSUPP` when the file system cannot encode a handle (`:30` to `:31`, under the comment from `:26`), it
  reports the size to ask for by `EOVERFLOW` (`:57` to `:66`), and `AT_EMPTY_PATH` takes the handle from a descriptor
  (`:97` to `:113`).
  `open_by_handle_at` opens it again under a directory descriptor (`:258` to `:266`), needs `CAP_DAC_READ_SEARCH`
  (`:182` to `:186`), keeps the directory descriptor's own mount (`get_vfsmount_from_fd`, `:118` to `:135`) and the
  decoded dentry (`:142` to `:165`), and opens the empty path under it (`file_open_root`, `:235`), so no mount is
  crossed. Every error the decode reports other than `ENOMEM` reaches the caller as `ESTALE`
  (`fs/exportfs/expfs.c:577` to `:591`). ZFS implements both directions (`zpl_export_operations`,
  `module/os/linux/zfs/zpl_export.c:251` to `:257`; `zpl_encode_fh` at `:36`, `zpl_fh_to_dentry` at `:100`), and it
  resolves a handle by object number and generation: an object that is unlinked, or whose generation differs, gives
  `ENOENT` (`module/os/linux/zfs/zfs_vfsops.c:1584` to `:1695`), which the export code turns into `ESTALE`
  (`zpl_export.c:119` to `:127`). A renamed folder therefore still resolves and a folder deleted and re-made under
  the same name does not. The generation is the creation transaction group, so a folder removed and re-made in one
  transaction group that happens to reuse the object number would resolve; the identity check the design runs after
  each open, birth time included, is the backstop, and the handle alone is not the proof.
- F30. The unbind is a non-lazy unmount, so its busy check covers the copies as well, with one exception that leaves
  a copy mounted. `do_umount` calls `propagate_mount_busy(mnt, 2)` (`fs/namespace.c:1792`), which refuses when the
  mount itself holds more references or has a mount under it, and refuses for a copy under a peer or slave of its
  parent at the same mountpoint only when that copy holds more references or has exactly one covering mount on it
  (`fs/pnode.c:407` to `:441`, with `do_refcount_check` at `:352` to `:355`). A copy with any other mount under it is
  skipped by that check (`:433` to `:437`) and left mounted without refusing, because `__propagate_umount` finds a
  mounted child and moves the copy to the restore list instead of unmounting it (`:478` to `:517`). So a process
  working inside a copy refuses the host's unbind with `EBUSY`, and the host's bind stays until that copy is
  released, its namespace exits, or the mount is lazily detached, which this design forbids; a copy with a mount
  under it survives a successful unbind, so the receipt's "bind gone" is true of the host namespace and not of every
  copy. The next `rmdir` of the folder detaches what survives (F11), so the exposure is a mount that stands a little
  longer, not one that stays.
- F31. A new mount namespace gets the peer groups of the one it was copied from: `copy_mnt_ns` copies the tree with
  `CL_COPY_UNBINDABLE | CL_EXPIRE` and adds `CL_SHARED_TO_SLAVE` only when the new namespace belongs to another user
  namespace (Linux `v6.8`, `fs/namespace.c:3760` to `:3788`). A namespace created in the same user namespace therefore
  holds peers of the host's shared mounts, and one created in a new user namespace holds slaves of them; both receive
  propagation. A namespace whose own tree was already made private, which is what a session that runs
  `mount --make-rprivate` leaves behind, holds a private copy and receives nothing, so any arm about copies must say
  which kind of reader it uses.
- F32. A boot id names one boot: the kernel's `/proc/sys/kernel/random/boot_id` holds one random identifier, mode
  `0444` owned by root, and a new boot gives a new value. Read on this host 2026-11-19 with `cat` and `stat`; the
  value is not recorded here. It is what tells a reader of a durable record whether a recorded mount ID or mount
  namespace inode still means anything (L10).
- F33. Both sides of a staging removal hold the clone's canonical numeric dataset GUID, and an accepted sentence
  names it in a filename. The handler verifies the clone's GUID when it enumerates the pinned parent's children and
  reads the desired-state property ([proposal][proposal], "Requests, results and ordering"), the wrapper's durable
  binding record carries the clone's GUID ([proposal][proposal], "Add and Git registration"), and the receipt's
  filename is that GUID ([proposal][proposal], "Requests, results and ordering": "using only the canonical numeric
  dataset GUID as the receipt filename"). A folder component derived from it therefore needs no request to carry
  it and no claim for the other side to read.

## Assumptions

- A1. The handler acts in the host mount namespace, as the design states, and the wrapper normally does too. The
  `EBUSY` protection of F11 holds only inside the namespace that holds the mount, so the handler records the device
  and inode of its `/proc/self/ns/mnt` beside the boot id that gives them meaning (F32) in one record per
  clone GUID, and the wrapper reads that record, compares the pair with its own, and refuses on a mismatch or on a
  different boot id before it removes any folder. The record also carries the folder's file handle and identity,
  which is what a later sweep opens the folder from (F29, L12). It lives at
  `<home>/.cache/monochromatic-git-policy-cli-staging-<guid>.json`, beside the folder of the same component, by the
  project's answer "In ~/.cache" and under the threat model that answer states ("Answers of 2026-11-19"). It is
  therefore data the account can read and replace, which the project's sentence accepts, and it is a cache of the
  folder's handle rather than root's authority: a record that does not match the folder's identity is a refusal, and
  never proof ("The staged-folder record").
- A2. The account home the staging target lives in is the passwd home of the pinned UID, not `$HOME` or
  `$XDG_CACHE_HOME`, because a root handler has neither and the answer names `~/.cache`. The project accepted that
  trust source without veto ("Proposals the owner settled").
- A3. `~/.cache` stays the mount of `zpcachyos/nosnap/user-cache` at boot and is not replaced by the account between
  setup and a removal. If it is replaced, the source-and-parent identity check refuses rather than adopts it.
- A4. The account does not deliberately sabotage its own staging, which is what the project's threat model for this
  machine states directly: the account is trusted, and root and the account are not a security boundary here
  ("Answers of 2026-11-19"). It is also the same-account residual the delegated destroy decision already accepts
  ([proposal][proposal], "Clone removal identity protection"). Routine cleaning of `~/.cache` is not sabotage and is
  covered by "Limits".
- A5. The removal flow obtains a receipt for its `unmount` nonce, showing the clone absent in every namespace and the
  bind gone from the host namespace, before the wrapper removes anything; a removal whose receipt never arrives, or
  arrives unclear, keeps its staged state reported and removes no folder. A refused unbind is a clear receipt of the
  other kind: it names the failed step, the errno and the copy that refused it, and the wrapper then removes nothing.
- A6. The staging folder is only ever a mountpoint carrier: nothing but the bind and the moved mount is put inside
  it, so a successful unbind leaves it empty.
- A7. A staged mount's presence in the host namespace is acceptable to the account, since the folder sits in the
  account's own `~/.cache`, at the mode F1 records, and the project treats the account as trusted ("Answers of
  2026-11-19").

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
- L10. A mount ID is unique only while its mount structure lives (F21), and a reboot gives new structures; a mount
  namespace's inode number is boot-scoped in the same way. A record may therefore carry a mount ID or a namespace
  pair only beside the boot id that gives it meaning (F32), and a reader voids both when the boot id
  differs. This design's staged-folder record carries the namespace pair and no mount ID at all; the folder's file
  handle is the one field that outlives a boot, which is what the recovery route and M6 rest on. The record sits in
  `~/.cache`, so the account can replace it; the project's threat model accepts that, and the sweep's folder identity
  check is what refuses a record that describes some other folder (F29, "The staged-folder record").
- L11. A folder removed or renamed over from a namespace that does not hold its mounts detaches every mount at it in
  every namespace (F11), so a mount at a folder that is gone has no row anywhere. What stays is a detached mount held
  by whatever holds it, the refusal a removal then sees is the destroy's `dataset is busy` (F13) rather than a mount
  row, and `doctor` reports the busy refusal and the retained state. Nothing in this design releases such a mount,
  and no later move can reattach it, because `do_move_mount` refuses a source that is not mounted
  (`fs/namespace.c:3145` to `:3146`).
- L12. The recorded-handle route of F29 is designed from source and measured nowhere: not for this file system's
  handle encoding, not for the capability the handler holds, not for a folder renamed or replaced between the record
  and the recovery, and not for a mount the account stacks on the folder. It is also the route a sweep needs and the
  only one it has; M6 carries its arms and every one of them is owed.

## The parts that need a design

The item names the staging target's creation, the private bind, the wrapper's cleanup, the unmount interface, and the
ordering against the `user-cache` mount. Each is stated here as the shape every way fills.

### The staging target and who creates it

The target is a directory under the account's `~/.cache` that the root handler can bind and into which it can move a
clone mount. Its identity is recorded before the first bind and rechecked before every move, and no request carries
its name, since no request carries a path (F15). Its name is therefore derived on both sides from data both sides
already hold: `monochromatic-git-policy-cli-staging-<guid>`, where `<guid>` is the clone's canonical numeric dataset
GUID spelled as the accepted receipt filename spells it (F33). The handler derives the same component for the clone
whose intent it serves, so it needs no path from the request and no claim to read, and one folder belongs to one
clone, so two removals of one clone cannot share a folder and two clones cannot collide. The nonce stays out of every
filename: it is data in the receipt and in the claim ("The staging claim").

The path from the pinned home to the folder crosses one mount, `~/.cache`, and that crossing is the deliberate one:
the descriptor on the home reaches the dataset's mount, whose source and mount ID are checked against the pin, and
every walk beneath that descriptor crosses nothing else, which is why `open_directory_at`'s `EXDEV` (F17) is the
right refusal for a walk that must not cross and the wrong call for the one crossing that is wanted (F1, F28, and the
accepted hop of [proposal][proposal], "Descriptor mount and unmount", lines 651 to 655 of that file).

The handler's pre-bind check is then about the folder and its own record, not about the claim: a directory owned by
the pinned UID, at the mode F1 records, empty, named exactly the derived component, reached under the `~/.cache`
descriptor, on the mount whose ID `/proc/self/fdinfo` prints for that descriptor and which equals the `user-cache`
row's (F17, F21, F28). A record for this GUID that already stands must match the folder's identity or the handler
refuses; where there is no record yet, the handler writes one before it binds (A1, F29, "The staged-folder record").
No channel from the claim to the handler exists, so this design adds none and states that the handler never reads a
claim. One consequence is stated rather than hidden: before the first bind there is no identity to compare with, so
shape is all the handler can check, and a folder the account substituted before that first bind is bound and recorded
as it stands; the wrapper's own claim identity is what then refuses to remove it, and the receipt and `doctor` report
it (M1).

The owner's words name the wrapper as the cleaner, and the first question's answer made that the design: the
wrapper creates the folder. The proposal's rule stops the handler from creating or removing "account slots" and
calls its slot descriptors descriptors "for mount management, not privileged writes into the account's tree", and
whether a staging folder is such a slot was a writer's reading of those lines; the answer settles it for the folder.

### The private bind

The bind is what makes the staging target a mount instance whose propagation the handler sets, because attaching a
mount under a shared parent propagates it (F6) and `~/.cache` is shared (F1). F4 records the measured shape. The
order is load-bearing: the bind is attached first, which also copies it into every peer and slave namespace of
`~/.cache` (F22), and made private second, after which the clone's move onto it propagates nowhere. The propagation
change needs the bind's own descriptor, and the folder descriptor cannot give it: `mount_setattr` requires the path
to be the root of its own mount (F14), and the folder is not the root of `user-cache`. The handler therefore takes
the bind's descriptor with the design's only by-name lookup after a mount covers the folder: a name under the
`~/.cache` descriptor opened with `O_PATH | O_DIRECTORY` and with `RESOLVE_BENEATH`, `RESOLVE_NO_SYMLINKS` and
`RESOLVE_NO_MAGICLINKS`, omitting `RESOLVE_NO_XDEV` for that one hop so the walk crosses into the bind, which is the
flag set the accepted hop already prescribes ([proposal][proposal], "Descriptor mount and unmount", lines 651 to 655
of that file), and confirmed by the mount ID `/proc/self/fdinfo` prints (F28, F21). That lookup is not a path-based
mount or unmount: it is a read that names the mount, the propagation change runs on the descriptor, and the only
calls that carry a path string are `mount(2)` and `umount2`, whose argument is a descriptor's `/proc` path, a reading
the project accepted without veto ("Proposals the owner settled"). The bind is a mount in the host namespace at a
path inside the account's tree, which is the exposure the decided default accepts ([proposal][proposal], "Decided
defaults"); its copies in other namespaces are a further exposure this design states rather than denies, and the
unbind's rule over them is F22 and F30. The project's sentence treats the account as trusted and root and the
account as no security boundary ("Answers of 2026-11-19"), so these exposures are documented rather than defended
against here; "Hardening the stated threat model may make unnecessary" lists the checks that guard them anyway.

The move is the one step whose destination does not keep the covered mount: `move_mount`'s destination follows the
stack to its top mount (F14), so a mount the account stacks on the folder after the bind receives the clone on top
of itself, and no descriptor call closes that window. The project's answer to what the handler then does is "Move it
back at once (Recommended)" ("Answers of 2026-11-19"), which is the accepted rule for a clone that lands somewhere
other than its slot ("Misplaced attach" under [proposal][proposal], "Project decisions"): the post-move check reads
the topmost mount at the folder and asserts it is the clone, by its dataset source and root, and that the clone's own
parent row is the bind, which is how F21 says a bind is told apart from the mount it binds. Where the topmost mount is
not the clone, or the clone's parent is not the bind, the handler moves the clone's own mount back to its slot by
that mount's descriptor and the slot's, with `MOVE_MOUNT_F_EMPTY_PATH` and `MOVE_MOUNT_T_EMPTY_PATH`. Where the
rollback fails, the clone stays staged, the receipt carries the misplaced state and `doctor` reports it, and the
removal refuses; keeping it staged is the state the busy-unmount decision allows. M1 arms the rollback and its failed
arm.

### The unmount call from staging

The unmount must not force or lazily detach, must not act on a path the account can change, must release
descriptors that would hold the mount busy, and must end with the mount absent from every namespace (F9, F13, and
the decided sentences quoted under "Decisions of the accepted design that bound this proposal"). F7 says how: a
`umount2` call whose argument is `/proc/self/fd/<n>`, with the descriptor taken on the staging folder before the
bind. The lookup lands on the topmost mount at that folder, so the first call unmounts the clone and the second
unmounts the bind, and both calls meet the decided sentence because the string they receive names a descriptor in
`/proc`, not a path inside the account's tree. The descriptor keeps a pathname substitution out of the call, but it
cannot keep a mount the account stacks on the folder out of the stack the kernel selects (F24): the call removes the
topmost mount whatever it is. The handler therefore checks the topmost mount's identity before each call and refuses
without calling when that mount is not the recorded clone or the recorded bind, then checks the transition after each
call, that the recorded clone is gone and the recorded bind is now topmost, then that the bind is gone and no mount
is left. The pre-call check narrows the window and cannot close it: a mount stacked between the check and the call is
the one the call removes, and the post-call transition check is what then catches it, because the mount it finds is
not the one it expected. A mount stacked after the move is a different scene from the one the third question
answered, because no clone landed on it: when the pre-call check sees it, the refusal leaves the clone on the bind,
reports the stacked mount, and unbinds nothing ("Answers of 2026-11-19"). F7 also means the call needs no name under
any ancestor, so a rename of an ancestor cannot take the staged mount out of reach. The reviewed draft tested only a
descriptor opened through the clone, which F10 says holds the clone and must refuse (F9); M2 keeps that reading as
the control.

That descriptor lives one handler invocation. A later sweep has no descriptor of its own and cannot make one by name:
a by-name open crosses the bind or the clone (F28) and the descriptor it returns pins the very mount the sweep would
unmount, so the call then refuses `EBUSY` (F9, F10). The sweep therefore opens the folder from the handle the handler
recorded before the bind (F29). Recording it costs no capability; the recovery needs `CAP_DAC_READ_SEARCH` in the
initial user namespace (`fs/fhandle.c:182` to `:186`) and every mount call needs `CAP_SYS_ADMIN` (F8), so both entry
points, ZED's child and the boot unit's service, have to keep both capabilities: a unit hardening that drops either
one stops every recovery, and M6 arms exactly that. The descriptor the handle returns is on the `user-cache` mount,
names the folder whatever its name is now, and does not cross the bind, so `umount2("/proc/self/fd/<n>", 0)` then
selects the topmost mount at that dentry exactly as the held descriptor does. A handler that cannot record the handle
before the bind does not bind: it refuses, reports the failed step, and the clone stays at its slot, which is a state
the accepted design already blocks a removal on.

What a sweep then finds is not one state but three, and they need different answers. A sweep whose process lacks the
capability gets `EPERM` from its first `open_by_handle_at` and unmounts nothing. A sweep whose handle no longer
resolves gets `ESTALE` (F29), and the folder that handle named is gone from every namespace with every mount at it,
so there is nothing left to unmount: what remains is a detached mount that its holders keep alive, and only those
holders ending frees the dataset (F11, L11). A copy of the bind in another namespace that a process works inside
refuses the unbind with `EBUSY` (F30), and the host's bind stays. In each of the three the sweep unmounts nothing,
removes nothing and reports the retained state, which the second question's answer leaves to the administrator and to
a later removal; none of them is a case where the design guesses ("Answers of 2026-11-19"). None of this is measured
(L12, M6).

The accepted design asks for "the interface that unmounts a staged mount from the held descriptor" and for an act
"on the held descriptor and never on a path inside the folder the account can change". This design reads the held
descriptor as the folder descriptor the handler holds from before the bind, and, where no descriptor outlives the
invocation, the recorded handle as the way a later sweep holds that same folder again: a handle names the inode and
generation the descriptor named, and it is not a path. All three readings are the project's, because the sentence's
words are: that `umount2` on a descriptor's `/proc` path acts on the held descriptor, that the same call is not a path
unmount, and that a descriptor reopened from the record is the held descriptor too. All three were put to the project
and accepted without veto ("Proposals the owner settled").

### What the wrapper removes and when

The answer's words are "make the wrapper clean after itself". The cleanup must not recurse over a mount, must not
remove a name it cannot prove it made, must cover an interrupted or failed removal, and must leave a retained staged
mount tracked and reported (the acceptance item quoted under the decisions). The wrapper already owns a claim, owner
and reap pattern for staging containers (F16), which is the nearest existing owner of this responsibility, and F16
also says why a staging claim cannot be one of those containers: the existing reaper deletes its containers
recursively and aborts its whole pass on one that is a mount. The claim also has a place that no existing pass counts
as pending work, because a folder inside the journal root would make every forwarded command in that repository run
the recovery pass (F27). The folder itself is removed by `rmdir` of a name the wrapper can prove: the component the
claim records, or the one an identity scan of `~/.cache` finds for the recorded device and inode after a rename (the
handle of F29 survives a rename; a name does not). Because the component is the clone's dataset GUID (F33), the
wrapper can match an entry to a clone, and to that clone's staged-folder record, without the claim. Two proofs stay
apart: the wrapper removes a folder only where its own claim and that claim's recorded identity agree with it, and
`doctor` prints as an orphan every entry that matches no live clone's GUID or no clone's record, and removes none.
The record is a file beside the folder and is removed by `unlinkat` of that same component and suffix after the
folder's `rmdir`, so the settlement removes one folder and one file and never a tree. `doctor` lists `~/.cache` by
descriptor, keeps folders and records apart by kind and suffix, prints each entry it cannot match and the state of
each it can, and only a removal's own sweep settles a folder ("The staged-folder record").

The claim's proof is what defends against a name the wrapper did not make, and the project's sentence treats the
account as trusted ("Answers of 2026-11-19"). The design keeps the proof anyway, as one entry of "Hardening the
stated threat model may make unnecessary" rather than as something the sentence removed.

### Ordering against the mount of `~/.cache`

A bind made before `zpcachyos/nosnap/user-cache` mounts lands on the home underlay and is hidden when the dataset
mounts later (F12). Nothing orders the two today, and the design's open item says so. Any way must either refuse to
bind until the recorded mount source is present or be created after that mount exists. The pin is the dataset's
source (`zpcachyos/nosnap/user-cache`) and the identity of the mount root under it, and never a mount ID across a
boot (L10).

## Ways to meet it

Each way covers the parts named in "The parts that need a design". Way A is the design, by the answer "Private bind
per removal (Recommended)" ("Answers of 2026-11-19"). The other ways are kept as ways not taken, because each one's
cost is a fact the design must not walk back into, and because a later revision that needs one of them has to carry
that cost openly. Costs are enumerated work, not sizes or times; none is measured.

### Way A: a claimed, depth-one folder per removal, bound and made private for that removal

- The wrapper checks `~/.cache`'s mount source against the pin of "Proposals the owner settled" before it creates
  anything, so a missing or replaced `user-cache` mount writes no entry on the home underlay (F12), then reads the
  clone's dataset GUID, creates `<home>/.cache/monochromatic-git-policy-cli-staging-<guid>` once per removal, at the
  mode F1 records, and claims it in a staging claim of its own kind ("The staging claim") before the folder exists.
  The component is the clone's GUID, never the intent's nonce, and the handler derives the same one (F15, F33).
- The handler opens `~/.cache` by descriptor from the pinned home, checks the mount's source against the pin, opens
  the folder by `openat2` under that descriptor with `O_PATH | O_DIRECTORY` and with `RESOLVE_BENEATH`,
  `RESOLVE_NO_SYMLINKS` and `RESOLVE_NO_MAGICLINKS`, and keeps that descriptor, which is on the `user-cache` mount
  (F10, F17, F28). It then checks shape and its own record rather than the claim: directory, owner, mode, empty, the
  derived component, and the descriptor's mount ID against the `user-cache` row (F21, F28, "The staging target and
  who creates it"). It records the folder's file handle, its identity and its own mount-namespace identity and the
  boot id in the staged-folder record (A1, F29), attaches the bind with the descriptor's path as both source and
  target (`/proc/self/fd/<n>`), opens the bind by name under the `~/.cache` descriptor with the accepted hop's flags
  and no `RESOLVE_NO_XDEV` so the lookup crosses into the bind, checks that descriptor's `mnt_id` in
  `/proc/self/fdinfo` against the bind's row (F28, F21), and makes it private with
  `mount_setattr(fd, "", AT_EMPTY_PATH, ...)` (F14), then closes that descriptor and moves the clone onto the bind
  by `move_mount` from the clone's own descriptor (F14, F5). The descriptor lifetimes stay apart (F10): the folder
  descriptor, opened before the bind and kept until the bind is gone; the bind descriptor, which exists only for the
  propagation change and is closed before the move; and every clone descriptor, closed before the clone's unmount.
  Because the move's destination follows whatever is stacked on the folder (F14), the post-move check reads the
  topmost mount, asserts it is the clone by its dataset source and root, and asserts the clone's parent row is the
  bind; where either assertion fails, the clone is rolled back to its slot by its own descriptor and kept staged only
  if that rollback fails ("The private bind", M1).
- The unmount is two `umount2("/proc/self/fd/<n>", 0)` calls, clone then bind (F7, F8), each preceded by a check of
  the topmost mount's ID in the mount table: the clone is confirmed by its dataset source and the bind by its root,
  parent and mountpoint, which F21 says is the only way to tell a bind from the mount it binds. Both run after every
  descriptor on the clone is closed (F10) and after the holder scan is clear. A later sweep runs the same two calls
  from the handle of F29, in the same order and with the same checks (M6).
- The handler unbinds, scans every namespace for the clone's source, writes the receipt, and the wrapper removes the
  folder by `rmdir` only after that clear receipt, after its own mount-namespace identity matches the handler's
  recorded one, and after re-checking the folder's identity against the claim the wrapper wrote (A5, F11). The unbind
  unmounts the copies in peer and slave namespaces too; a copy a process works inside refuses the whole unbind with
  `EBUSY`, and the handler then reports the copy, its namespace and the processes holding it instead of claiming the
  bind is gone (F22, F30). A copy that carries a mount under it is left mounted by a successful unbind, so the
  receipt says the bind is gone from the host namespace, which is what it can prove (F30, L11, M4).
- The folder's only ancestor inside `~/.cache` is the mount root of the dataset, which the account cannot rename in
  the host namespace (F11), so no ordinary ancestor can carry the bind away.

Pros: no privileged write of content into the account's tree, since the handler writes one record file and creates no
directory of root's there; the creator of the folder is also the cleaner; one claim per removal; the bind lives one
removal, so concurrent removals use their own folders and their own binds and need no lock; the unmount and unbind
act on a descriptor's own path and never on a name inside the account's tree (F7); the depth-one folder removes the
renamable ancestor the reviewed draft carried, and the recorded handle survives a rename of the folder from a foreign
namespace (F29), so only a recovery the handle cannot serve is left to the administrator; the retained staged state
the busy decision allows is a folder whose name and record this design can prove and report.

Cons: the account can remove the empty folder before the first bind and replace it, and the first bind has no
identity to compare, so the replacement is bound and recorded as it stands while the wrapper then refuses to remove
it (M1's replacement arm); the bind is copied into peer and slave namespaces of `~/.cache` at attach time (F22, M3),
and a successful unbind takes those idle copies with it while a copy a process works inside refuses the unbind (F30,
M4); the by-name open of the bind is the one lookup that runs after a mount covers the folder, and its mount ID
check is what keeps it honest (F28); a recovery needs the handle, which the wrapper cannot record on the handler's
behalf, so a lost handle is a retained mount (L12); a wrapper crash between claim and folder removal leaves a folder
and a stale claim the staging sweep must settle; a third-party recursive delete of `~/.cache` reaches a staged clone
(F26, L9).

Build cost: one wrapper module for the staging claim, folder creation, identity check, `rmdir`, the orphan scan and
notices; one handler step for the handle record, bind, propagation change, move, two unmounts, unbind and the
receipt, with its state fields; a new deadline owned by step 9 and sized by step 22 (F20); `doctor` text for a
retained staged mount, an orphaned folder, an unprovable claim, a handle that no longer resolves, and a detached
mount that still holds the dataset.

Test cost: device-free tests for the staging claim kind, the folder grammar, `rmdir`-only removal, each refusal's
text, the identity mismatches and the namespace identity check; the VM arms M1, M2, M3, M4's positive,
host-namespace, foreign-namespace, renamed and copied arms, M5, every arm of M6, and M7's two-folder arm.

### Way B: a root-made target per removal, bound for that removal

- Root creates `<home>/.cache/monochromatic-git-policy-cli-staging-<guid>` mode `0700` owned by root, says so in the
  staged-folder record, binds it, makes it private, moves the clone onto it, unmounts, unbinds, and the wrapper
  removes the target after the clear receipt. Root draws the name here,
  which is the one way the handler does not have to derive it (F33, A1).
- The wrapper creates no folder; it records the removal and waits for receipts.

Pros: the account cannot plant content in the target, and the identity it is checked against comes from root's own
record rather than from the wrapper's claim, though that record is itself account-writable now ("The staged-folder
record"); the account's rename of the target's name is refused by the descriptor identity check before the move, as in
Way A.

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
  keeps it until the stop pass. Each removal creates its folder inside the bind, named from the clone's GUID as
  everywhere else (F33), and moves its clone there.
- Each removal's unmount is one call, on the clone only: after that the folder is a plain directory inside the bind,
  not the root of a mount, so a second `umount2` through a folder descriptor would fail `path_mounted` (F8). The
  bind is removed by the stop pass, which needs its own anchor: either a descriptor the boot unit kept alive or a
  path unmount under `~/.cache`, which the decided sentence forbids; the second question's answer leaves a retained
  bind's handle to a later sweep.

Pros: the bind exists before any removal, so a removal never waits for a mount transition; the boot unit's start
reconcile gains one setup step instead of an on-demand step; the folders inside one private bind do not interact, so
no global claim and no serialisation of unmounts is needed (this corrects the reviewed draft, which read the shared
target as forcing one).

Cons: a mount in the host namespace for the whole boot; every removal's folder sits inside it, so an account
`rmdir` from a foreign namespace detaches the bind and every staged mount with it (F11); the stop pass must unmount
a staged clone before the bind, and must never force it; the stop pass also needs an anchor for the bind itself,
which a boot-once process does not keep, and the recorded-handle route is the design's answer only for the folder it
was recorded for; the boot unit must order the bind after the `user-cache` mount, which
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

Test cost: M1, M2, M3's propagation arms on `~/.cache` itself, M5, every arm of M6, and M7, plus M3's
late-mount arm, which is the control that an already-confined reader loses a late mount under `~/.cache` once it is
private.

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

Ways B, C, D, E and F are not refused by any decision: each is a design a writer could build, and none of them is
the design, which the answer of 2026-11-19 fixed as A. They are recorded with the cost that kept each out: C moves
the bind's removal to the boot unit and to an anchor no long-lived process keeps, E protects a folder that holds no
content and repeats a shape the project vetoed, and F buys one printed path with a renamable ancestor and an adoption
rule. The first question's other two options were D and B, and what each leaves untaken is one sentence under
"Questions the owner answered".

## Ranking

The answer of 2026-11-19 makes A the design; this ranking is what its costs decide, and it is kept so that a later
reader can see why the others were not taken. Order: A > D > B > C > E > F.

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
- Replaced before the open: the account removes the folder and makes a new one of the same name before the handler
  opens it. Assert the first bind cannot tell the difference, so the bind proceeds on the substituted folder and the
  record carries its identity, and then that the wrapper's removal refuses on the claim's identity mismatch and
  reports it (F17, F29). Control: the undisturbed folder in the same run, whose removal proceeds.
- Replaced between the open and the bind: the account removes the folder after the handler holds the descriptor and
  makes a new one. A removed dentry cannot become a mountpoint: `vfs_rmdir` marks the inode dead and calls
  `dont_mount` (`fs/namei.c:4213` to `:4216`), and `d_set_mounted` refuses an unlinked dentry with `ENOENT`
  (`fs/dcache.c:1369` to `:1394`). So assert the bind is refused, the replacement name holds no mount, and the
  state is reported; record the errno. Control: M1's positive arm in the same run, where the undisturbed folder
  carries the bind and the clone, so the refusal in that arm is the removal's and not the fixture's.
- Stacked before the move: the account stacks a FUSE mount on the folder with `fusermount3` after the bind and its
  propagation change and before the move. Assert the move succeeds and its destination is the account's mount, so the
  topmost mount is the clone and the clone's parent row is the account's mount and not the bind, then that the handler
  rolls the clone's own mount back to its slot by its descriptor (F14, F24). Assert the clone's source row is at its
  slot again and the account's mount still covers the folder. Control: M1's positive arm in the same run, where the
  topmost mount is the clone and its parent is the bind, and the check passes. No kernel call closes this window; the
  arm records what it costs.
- Stacked before the move, rollback refused: the same stacking, and the slot's directory removed from a foreign
  namespace before the rollback, so the rollback's destination is gone and the move refuses. Assert the clone stays
  staged, the receipt carries the misplaced state, `doctor` prints it, nothing is unbound, and the clone's source row
  is not at its slot. Control: M1's stacking arm in the same run, where the slot stands and the rollback succeeds.
- The bind's descriptor: after the bind and before the propagation change, open the folder by name under the
  `~/.cache` descriptor with `O_PATH | O_DIRECTORY` and with `RESOLVE_BENEATH`, `RESOLVE_NO_SYMLINKS` and
  `RESOLVE_NO_MAGICLINKS`, and no `RESOLVE_NO_XDEV`; assert `/proc/self/fdinfo/<n>` prints the mount ID of the bind's
  row (F28, F21), then assert `mount_setattr(fd, "", AT_EMPTY_PATH, ...)` succeeds and the row loses its `shared:`
  tag (F14, F23). Control: the same two calls on the folder descriptor taken before the bind, where the `fdinfo`
  mount ID is `user-cache`'s and `do_mount_setattr` refuses `EINVAL` (F14), and no row changes; and an
  `open_tree` on the folder descriptor with `""` and `AT_EMPTY_PATH`, which must also report `user-cache`'s mount ID,
  so a reader that prints the bind's ID for both has not read what it claims (F14, F28).
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
- Substituted top mount, seen by the check: the account stacks a FUSE mount on the staged folder with `fusermount3`
  after the handler's identity check and before its first pre-call check. Assert the pre-call check refuses without
  calling, so the FUSE mount is still there, the recorded clone is still under it, no `umount2` ran, and the handler
  reports and unbinds nothing. Control: the covered-folder arm in the same run, where the topmost mount before the
  call is the recorded clone and the call proceeds.
- Substituted top mount between the check and the call: the account stacks the FUSE mount after the second call's
  pre-call check and before its `umount2`. Assert the call removes the FUSE mount, that the post-call transition check
  finds the bind where it expected no mount, and that the handler therefore refuses, reports and unbinds nothing, so
  the bind stays. Control: the covered-folder arm in the same run, where the second call finds the bind gone. This
  arm records what the race costs: a mount the account made can be the one the call removes, and no kernel call closes
  that window (F24).

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
- Late mount lost: with `~/.cache` made private before the reader namespace is created, mount a new file system under
  `~/.cache` after the reader exists. Assert the reader holds no row for that mount (F6, F31). Control: the same
  mount under a shared `~/.cache` in the same run, which the reader must see. This is Way D's cost, and it is the arm
  that shows the delivery a private `~/.cache` gives up.

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
- Copied bind, mounted under: the reader from the idle arm mounts a file system on a folder inside its own copy, so
  the copy has a mount under it. Assert the host's unbind succeeds, the host's bind row is gone, and the reader's copy
  is still mounted at the folder path with the mount under it (F30). Then assert the next `rmdir` of the folder from
  a namespace that holds no mount at it detaches both (F11, L11). Control: the idle arm in the same run, where the
  copy goes with the unbind.
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
  the staging claim is present, the staged-folder record carries the folder's handle, its identity, the handler's
  namespace pair and the boot id, and no later pass removes the folder or the claim. Then assert the next `unmount`
  intent's sweep opens the folder from that handle (F29), unmounts the empty bind, and leaves the clone at its slot,
  so the removal starts over at the slot and may refuse there again, which is the accepted busy refusal. Control:
  M1's positive arm in the same run, where an undisturbed handler moves the clone.
- Kill after the move, before the unmount. Assert the staged clone's source row is present, the staging claim is
  present, the staged-folder record carries the folder's handle, its identity, the handler's namespace pair and the
  boot id, no later pass removes the folder or the claim, and no reattach happens (the no-implicit-remount default).
  Then assert the next `unmount` intent's sweep opens the folder from that handle (F29), checks the descriptor's
  `fdinfo` mount ID against `user-cache`'s row and the folder's identity against the record, completes the unmount
  and the unbind, and only then allows the folder's removal. Control: an undisturbed removal reaches M1's positive
  arm.
- The handle against the by-name route: the same kill, then two sweeps from fresh processes, the by-name sweep first,
  because the handle sweep removes what the by-name sweep would test. Assert the by-name sweep unmounts nothing and
  records `EBUSY` from its first call, with its descriptor's `fdinfo` mount ID equal to the clone's row (F10, F28),
  and then that the handle sweep unmounts the clone and then the bind. Control: the handle sweep's success in the same
  run, so the arm shows the handle route does what the name route cannot.
- No capability: run the sweep of the kill-after-the-move state from a unit that drops `CAP_DAC_READ_SEARCH`. Assert
  the first `open_by_handle_at` refuses `EPERM` (`fs/fhandle.c:182` to `:186`), nothing is unmounted, nothing is
  removed, the retained state is reported, and `doctor` prints it. Control: the same state swept by a unit that keeps
  both capabilities in the same run, which unmounts and reports as the kill-after-the-move arm says.
- A folder substituted from a foreign namespace: an account process in a private mount namespace created before the
  bind removes the folder and makes a new one of the same name, then the sweep runs. Assert `open_by_handle_at`
  refuses `ESTALE` (F29), that no namespace holds a row for the bind or the clone because the removal detached every
  mount at that dentry (F11), that a holder left inside the staged clone keeps `zfs destroy` refusing
  `dataset is busy` (F13, L11), that the sweep unmounts nothing and removes nothing, and that the claim and `doctor`
  name the state. Control: the undisturbed handle resolves in the same run, where the bind and the clone still have
  rows.
- Kill after the unmount, before the folder removal. Assert the next pass removes the folder only after a clear
  receipt for the last scan and after its mount-namespace identity and boot id match the handler's recorded ones,
  then unlinks the record beside it, and prints a notice. Control: the same pass with the receipt withheld keeps the
  folder and the record and reports both.
- The record's place and atomic write: assert the record is at
  `<home>/.cache/monochromatic-git-policy-cli-staging-<guid>.json`, on the `user-cache` mount and not the home
  underlay, mode `0644`, and that it appears in one rename. Kill the handler between the temporary write and the
  rename, then assert the recorded name is absent or holds the previous whole record, the temporary name is the only
  stray, the sweep reports the folder as unrecorded and unmounts nothing, and the next removal rewrites the record.
  Control: the undisturbed handler writes it once, which the sweep reads.
- A record from another boot: write a record carrying a boot id the machine does not hold now, and run the sweep.
  Assert it voids the handle and the namespace pair on the boot id alone, unmounts nothing, leaves the folder and the
  record standing, and reports the state. Control: the record of the current boot in the same run, whose handle the
  sweep opens.
- Namespace created while staged: a reader namespace enters while the clone is staged, then the wrapper's cleanup
  runs. Assert first that the scan names the reader's copy and the wrapper removes nothing (the reader's rows are the
  assertion), then that after the reader exits and a clear receipt arrives the wrapper removes the folder and the
  reader's namespace holds no row for it. Controls: a scan with no reader is clear and the folder is removed at once;
  a wrapper run in a namespace whose `/proc/self/ns/mnt` identity differs from the handler's recorded one refuses
  and removes nothing.
- Kept folders: a folder holding a foreign file is kept and reported; a folder whose component matches no live
  clone's dataset GUID is kept and reported as an orphan; a folder whose recorded identity no longer matches is kept
  and reported. Control: the same three folders with nothing wrong in them, which must each be removed by `rmdir`,
  so a sweep that keeps every folder fails the arm.
- A claim lost with the common directory: remove the repository's staging claim and leave the folder, its record and
  its handle standing. Assert the sweep keeps the folder, reports it, and that `doctor` prints it as an orphan,
  because without that claim nothing proves the wrapper made the folder. Control: the same folder with its claim
  standing and its identity matching, which the sweep removes by `rmdir` after the clear receipt.

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

Each live way needs M1, M2, M3, M5 and every arm of M6. Way A, the design, additionally needs M4's positive,
host-namespace, copied-idle, copied-held, copied-mounted-under, foreign-namespace and renamed arms, M7's two-folder
arm, M1's failed-rollback arm, and M6's record-place, atomic-write and other-boot arms. Way B needs Way A's handler
arms plus M7 and M8. Way C needs M1, M2, M3, M5, M6, M4's renamed arm and M7's Way C arm. Way D
needs M1, M2, M3's propagation arms and its late-mount arm on `~/.cache`, M5, M6 and M7. Way E needs Way A's arms and
M8's source arm. Way F needs Way A's arms and M4's renamed arm. No live way needs M9, which exists only to record the
refused way's failure.

## Questions the owner answered

No question stands open. The plan calls the item writer design work, so these were the points where a writer would
otherwise have had to add a decision the accepted design did not make, or reverse one; the three questions and the
veto list were answered on 2026-11-19, and "Answers of 2026-11-19" lists them with the answers in the project's
words. What each answer left untaken, in the project's terms:

- The first question's other two ways. Making `~/.cache` private once at boot would change an existing mount, which
  the accepted proposal puts outside this work, and letting root create the folder writes into the account's tree,
  against the proposal's rule on slots. Both stay described under "Ways to meet it" as ways not taken.
- The second question's other two routes. A boot-long supervisor holding every staged folder open adds a privileged
  process whose crash returns the removal to the administrator, and waiting for the administrator makes every handler
  crash leave a worktree unremovable until a person acts.
- The third question's other two answers. Leaving the clone on the mount the account stacked keeps the dataset busy on
  a mount the handler did not make, and moving it onto the folder's bind again repeats the step that was interfered
  with.
- The one proposal the project did not take as written: the record under
  `/var/lib/monochromatic-git-policy-cli/`, which now lives in `~/.cache` ("The staged-folder record").

Three ways are described and ranked under "Ways to meet it" without having been offered as a question: the boot-made
bind around a folder per removal leaves the bind's removal to the boot unit and to an anchor no long-lived process
keeps, and the project's sentence names the wrapper as the cleaner; the root-owned empty source protects a folder
that holds no content and repeats the root-owned shape the project vetoed at `/run/git-worktree-clone-mount-staging`;
the staging root two levels deep buys one printed path with a renamable ancestor and an adoption rule. A fourth way,
the namespace teardown, cannot be built at all, because the kernel refuses to move a mount out of another namespace.

### Not asked, and why

- How long a retained staged mount may stand. The accepted acceptance item already says a retained staged mount is
  tracked and reported, and `doctor` prints the folder's recorded age, so a refusal or a time limit is an addition
  nobody asked for. It becomes a question only if the project wants one, with the number it would use.
- Whether a lazy detach or a path unmount may clear a retained mount. The decided sentences exclude both, so
  reversing them is a separate request, not an option inside this design.
- Whether the namespace teardown may stage a clone. The kernel refuses to move a mount out of the namespace that
  holds it, so this is recorded as a refused design rather than asked.

## The staging claim

The existing claim pattern admits only its own container shape, and its settlement deletes recursively through
whatever is mounted (F16). A staging claim is therefore a kind of its own, with every one of its parts:

- Its own place: `<common>/monochromatic-git-policy-cli-worktree-copy/staging/`, a private directory beside `v1`
  under the same private parent. It cannot live inside `v1`: the forwarding check counts every entry there except the
  unreadable folder as pending work, so a subdirectory would make every forwarded command in that repository run the
  recovery pass, and a directory beside `v1` is listed by no pass (F27).
- Its own suffix: `<guid>.staged`, where the component is the clone's canonical numeric dataset GUID, so no reader
  of `.stage` or `.json` ever sees it (F27, F33).
- Its own reader: a staging sweep that understands only this shape, and that keeps and reports a claim it cannot
  read instead of moving it aside, because an unreadable claim may name a folder whose state is unproven. It runs
  from a removal, for that repository's own claims, and no forwarded command pays for it. `doctor` reads the claims
  and prints what it finds; it settles none, because it changes nothing ("What a writer can settle without asking").
- Its own settlement: `rmdir` of the folder, then `unlinkat` of the record beside it, and only after the wrapper's
  mount-namespace identity and boot id match the handler's recorded ones, after the clear receipt for the nonce the
  claim records arrived, and after the recorded identity of the folder still matches. It never changes a mode, never
  deletes recursively, and never calls the stage removal helper.

The claim carries the nonce of the `unmount` intent it is waiting for, the owner token and the folder's recorded
identity (device, inode, birth time, owner, mode). Its filename is the folder's component, which the wrapper derives
from the clone's dataset GUID and the handler derives the same way, so no record has to hand a name to the other side
(F33). A retry that writes a new nonce for the same removal keeps the standing folder and updates the claim's nonce
before it writes the intent, so the receipt's nonce always matches the claim's own and a receipt never has to name a
folder (F15). A new folder is made only after the standing one is removed and its claim released, which is also what
keeps two removals of one clone apart, since both would derive one component. An intent whose folder was never made
leaves a claim whose folder is absent, and the sweep releases that claim with a notice.

The accepted text says three things this section has to honour. "No request supplies a path, dataset name or mount
flags" ([proposal][proposal], "Requests, results and ordering"); "Nonces appear only as validated data in the
receipt, never in a filename" ([proposal][proposal], "Requests, results and ordering"); and the receipt directory
uses "only the canonical numeric dataset GUID as the receipt filename" ([proposal][proposal], "Requests, results
and ordering"). The folder's component is the same GUID the accepted text already puts in that directory's names, the
intent's nonce stays data in the receipt and the claim, and no request carries anything new. The handler checks the
component against the clone GUID it derived before it opens the folder, so a name it did not derive is never used.

One limit stays, and it is per repository: a claim lives in a repository's Git common directory, so a repository
whose common directory is gone takes its claims with it, and a folder it left under `~/.cache` has no readable claim.
Such a folder is reported rather than removed: `doctor` lists `~/.cache` by descriptor, matches each entry's component
against the live clones' dataset GUIDs and their staged-folder records, and prints as an orphan every entry it cannot
match to a live clone, or cannot match to that clone's record. It removes nothing, and it claims no index of
repositories it does not have; only a removal's own sweep settles a folder, because only that removal holds the claim
that proves it made the folder ("The staged-folder record").

The device-free test this owes: in a private user and mount namespace (no ZFS device, no pool), bind a directory
holding a sentinel file onto a claimed folder, write a staging claim whose owner is dead, then run the existing
recovery pass and the staging sweep. Assert the recovery pass returns success, the sentinel's directory keeps its
mode, the sentinel survives, and the staging sweep keeps and reports the claim. A pass that aborts, changes a mode or
deletes a byte fails the test. The control is the same fixture with the mount replaced by a plain directory, where
the staging sweep must remove the folder by `rmdir` and the recovery pass must still return success.

## The staged-folder record

A later sweep needs three things a receipt cannot give it: the folder's file handle, because a name may have moved
or gone (F29), the folder's identity, because the handle alone is not the proof, and the boot id with the
mount-namespace pair that say whether a recorded mount ID or inode still means anything (F32, L10). They live in one
record per clone GUID, at `<home>/.cache/monochromatic-git-policy-cli-staging-<guid>.json`: the folder's own
component with a `.json` suffix, beside the folder of that component, so one name pair under `~/.cache` says which
folder a clone's mount waits in and which record describes it (F33). The suffix is one no staging claim keeps (F27),
and no journal pass lists `~/.cache` at all. The record is a file and not a folder, so the staging folder stays the
empty mountpoint carrier A6 describes, and the unlink that removes the record is not a delete under any mount.

The handler writes it before the first bind, through the `~/.cache` descriptor and so on the `user-cache` dataset, at
mode `0644`, and the write is atomic: the handler unlinks any stale `<component>.json.tmp` left by a killed handler
under its `~/.cache` descriptor, creates the temporary name beside the record with `O_CREAT | O_EXCL`, writes and
`fsync`s it, renames it over `<component>.json` with `renameat`, and `fsync`s the directory. A reader therefore sees
the whole record of one write or the one before it and never half of one, and a temporary name needs no settlement of
its own because the next write replaces it. The wrapper, `doctor` and a later sweep read it; the handler is the only
writer, and the wrapper's settlement unlinks it after the folder's `rmdir` succeeds.

The record is written by root into a directory the account owns, so it is data the account can read, replace or
delete. The project's answer to where it lives accepts that and states why: "We have to assume my machine isn't
compromised. sudo is passwordless here anyway." ("Answers of 2026-11-19"). It is therefore not authority against the
account. It is the only cache the design has of a handle no live state can rebuild once the handler's invocation has
ended, and both the sweep and the wrapper check it against the folder's own identity before acting on it (F29, A1).
Writing it is also a privileged write into the account's tree, which the accepted sentence about the handler's
descriptors does not allow: they serve "mount management, not privileged writes into the account's tree"
([proposal][proposal], "Descriptor mount and unmount"). The project's answer puts the record there, so this design
takes that departure for this one file, at this one name, and keeps every other handler write inside its descriptors'
own mount management.

The accepted text says two things about root's files, and the record's place keeps both. "Publish in
`/run/git-worktree-clone-mount-results`, using only the canonical numeric dataset GUID as the receipt filename"
([proposal][proposal], "Requests, results and ordering") holds because the design writes nothing else into that
directory, and nothing into `/var/lib/monochromatic-git-policy-cli/` for staging: the record is not a receipt and
does not live beside the slot identities. "The runtime files are a cache, not authority to act; rebuild them from
live state and durable intent" ([proposal][proposal], "Requests, results and ordering") is the sentence this
section departs from, and the departure is exactly the record's handle. In place and ownership the record is a cache:
the account's file, under the account's directory, that any later run can replace. In every field but one it is
rebuildable: the identity is re-read from the folder, the namespace pair and boot id are re-read from the running
handler, and the nonce comes from the intent. The handle alone cannot be rebuilt from live state once the invocation
that opened the folder has ended. So the record is a cache that holds one field of authority, the right to reopen one
folder, and no authority to unmount: it names no mount, carries no mount ID, and every act after it is checked against
the folder's identity and the live mount table (F29, F21). The project's answer "Record the file handle
(Recommended)" accepted that field ("Answers of 2026-11-19").

The record is one JSON object. It holds the folder's component, the clone's dataset name and GUID, the folder's
identity, its file handle, the nonce of the intent it last served, the handler's mount-namespace device and inode,
and the boot id those belong to. It carries no bind mount ID: the bind's row is read live, from its root field, its
parent and its mountpoint (F21), which is what lets the whole record be written before the bind (F29). The namespace
pair and the boot id are the boot-scoped fields, and a reader voids them when the boot id is not the one it reads now
(L10). A later invocation for the same GUID reads the record before it binds: a record whose identity does not match
the folder refuses, and a record with no handle refuses too, because the handler that wrote it did not bind. The
wrapper reads the record before it removes a folder, which is where its mount-namespace check comes from (A1), and
`doctor` reads it to print a folder's age and to keep a record apart from a folder when it lists `~/.cache`. No
request and no receipt carries any field of it (F15).

Its lifetime is the folder's, with one exception. It is written before the first bind, replaced atomically by every
later invocation for that GUID, and unlinked after the folder's `rmdir`. A record whose folder is gone is an orphan
`doctor` prints and no pass removes, because only a removal's settlement unlinks a record. A reboot drops every
mount, so a record carrying another boot id is void: a sweep reads it, voids the handle and the namespace pair
because the boot id differs, unmounts nothing on that record and reports it, and the next removal for that clone
rewrites it (F32, L10).
Nothing else ages it out, and no timer touches it.

## What a writer can settle without asking

- The settled answers are the design's own: one folder per removal with its private bind, the recorded handle as a
  sweep's route, and the rollback of a clone that lands on a mount the account stacked ("Answers of 2026-11-19").
- The folder grammar under `~/.cache`: one readable component that says what it holds, at the mode F1 records, and
  that component is the clone's canonical numeric dataset GUID and no other variable part (F33).
- The unmount and unbind route: the covered-folder descriptor and the two `umount2` calls (F7, F8, F10, F14), the
  topmost-mount check before each call and the transition check after it, the by-name open that reaches the bind for
  the propagation change (F28), and the recorded handle a later sweep opens the folder with (F29).
- The order inside a removal: write the staged-folder record with the handle, the identity, the namespace pair and
  the boot id, bind, take the bind descriptor and make it private, move, verify that the topmost mount is the clone
  and that its parent is the bind, roll the clone back to its slot where either fails, unmount the clone, unbind,
  scan every namespace, write the receipt, then the wrapper's `rmdir` and the record's `unlinkat` after the clear
  receipt.
- The wrapper's mount-namespace identity check before any folder removal, comparing `/proc/self/ns/mnt`'s device and
  inode and the boot id with the handler's recorded ones in the staged-folder record, and refusing on a mismatch
  (F11, A1).
- That a later `unmount` intent's staging sweep completes an interrupted removal's unmount and unbind from the
  recorded handle (F29), re-verifying the folder's identity and the mount table's rows before it acts, and that
  anything the handle cannot serve stays staged and reported.
- The identity fields checked before each move and before each removal: device, inode, birth time, owner, mode,
  access and default ACLs from the held descriptor, plus the mount's source and parent (F17, F18, F21).
- The record's place, shape and atomic write: one record per clone GUID beside the folder of the same component
  under `~/.cache`, mode `0644`, written by the handler before the first bind through a temporary name renamed over
  the final one, holding what "The staged-folder record" names, and unlinked after the folder's `rmdir` (A1, L10).
- The receipt and notice wording for a retained staged mount, a mounted folder that cannot be unmounted, a handle
  that no longer resolves, a copy that refused the unbind, a detached mount that still holds the dataset, an orphan
  folder, and a folder the wrapper cannot prove it made.
- That cleanup is `rmdir`-only and never recursive, that it stops at any mount, and that it runs from the staging
  sweep keyed to a dead owner rather than from a timer.
- That the wrapper never removes a folder while a mount covers it in the host namespace, and that the handler never
  removes a folder it did not bind.
- That the unmount, bind and unbind waits share the removal deadline step 9 introduces and step 22 sizes (F20).
- That both entry points keep `CAP_DAC_READ_SEARCH` and `CAP_SYS_ADMIN` (F8, F29), and that a unit hardening which
  drops either one is a defect for every recovery, which M6 arms.
- That `doctor` prints the staging folder, its live bind's propagation, any staged mount with its dataset and age and
  the next safe step, that it reads the claims and the staged-folder records and prints what it finds, that it
  settles no claim and removes no folder, and that it changes nothing, as the `doctor` decision quoted under
  "Decisions of the accepted design that bound this proposal" states.
- That the staging claims live in the place and the shape "The staging claim" names, and the records in the place and
  the shape "The staged-folder record" names.

## Proposals the owner settled

The project answered the whole list with "Veto none (Recommended)" ("Answers of 2026-11-19"), so every entry is
now the design. The list stays because each entry names what the design decides beyond the accepted text.

- Pinning the `user-cache` mount: the handler records the dataset's source and the mount root's device and inode,
  and refuses to create a bind when the mount at `~/.cache` is not that source. The pin is one key of the
  root-controlled `/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc` the proposal keeps root's
  ([proposal][proposal], "ZED reconciliation and mount safety"); the handler reads it before every bind, the boot
  reconcile reads it before it verifies staging, and the wrapper reads it before it creates a folder. This is new
  root configuration the accepted design does not pin, and it is what makes F12's ordering checkable rather than
  assumed. The proposal owes the boot-time shape of the same check ("Test staging setup at boot and refuse shared or
  replaced staging and checkout mounts until a private setup is verified"), and this design meets it by verifying the
  pin at boot reconcile and by verifying each removal's bind and the checkout's propagation before each move.
  Accepted, veto none.
- Reading the staging location from the Pinned UID's passwd home (A2), rather than from an environment variable a
  root handler does not have. Accepted, veto none.
- Reading `umount2` on a descriptor's `/proc` path as an act on the held descriptor rather than on a path. The
  argument is a string, and what makes it safe is that the kernel never resolves a name of the account's tree: it
  jumps to the descriptor's own location (F7). `umount2` has no form that takes a descriptor, so this reading is the
  only way to act from the held descriptor at all. Accepted, veto none, and the design unmounts this way.
- Reading a `mount(2)` call whose source and target are the same descriptor's `/proc` path as not being a path-based
  mount. The call does receive a string, and the sentence it tests is "No path-based mount or unmount on the
  account's tree" ([plan][plan], "What the design forbids"); the design reads the forbidden thing as a name inside
  that tree. Accepted, veto none, and the design binds this way.
- Reading a descriptor re-obtained from the recorded file handle as "the held descriptor" the accepted sentence
  names, rather than only a descriptor a running process still holds. Accepted with the file-handle route the second
  question's answer took (F29, L12).
- The folder's file handle. The design first put it in a durable record under
  `/var/lib/monochromatic-git-policy-cli/`, and the project's answer moved it to `~/.cache` with the threat model
  that answer states ("Answers of 2026-11-19"). The record now lives beside the folder, at
  `<home>/.cache/monochromatic-git-policy-cli-staging-<guid>.json`, as one cache file rather than root's state
  ("The staged-folder record").

## Hardening the stated threat model may make unnecessary

The project's sentence treats the account as trusted and root and the account as no security boundary on this machine
("Answers of 2026-11-19"). Several checks in this design exist only because a draft assumed the opposite. None is
removed here on the strength of that sentence: each is listed with what it costs today and what dropping it would
simplify, as a suggestion for the project to decide. Taking one up is a revision of this design, not a reading of the
sentence, because the design would lose a refusal that its arms currently assert.

- The account-owned folder, and the root-owned alternatives of Ways B and E. The design reasons about who owns the
  staging folder, checks its owner and mode before binding, keeps M1's replacement arms and M8, and the project
  vetoed a root-owned `0700` bind at `/run`, giving up the protection that placement outside account-controlled trees
  carried. Under a trusted account that whole argument loses its premise: the folder's ownership protects against a
  substitution the project's sentence says will not happen. What it costs: a shape check on every bind, the
  replacement arms, and Way B's and Way E's extra root objects and their lifecycles. What removing it would simplify:
  the folder could be made by either side, the owner and mode checks would go, and the two root-owned ways would need
  no defence for an account that is not hostile. F11 still bounds this: root ownership does not stop a rename or
  `rmdir` from another mount namespace, so dropping the argument buys less than it first appears.
- The claim's identity proof, and the orphan scan that replaces it. The wrapper removes a folder only where its own
  claim and that claim's recorded identity agree with the folder; a folder whose claim is gone is kept and reported;
  `doctor` lists every entry it cannot match to a live clone. What it costs: the staging claim kind with its own
  place, suffix, reader and settlement ("The staging claim"), the orphan scan, the "kept folders" and "claim lost"
  arms of M6, and the notice wording for each. What removing it would simplify: settlement becomes `rmdir` of any
  entry whose component is a live clone's dataset GUID after a clear receipt, the claim kind leaves the build, and
  `doctor` prints the folder without proving who made it.
- The stacking defences. The post-move check and the rollback it triggers, the topmost-mount check before each
  `umount2`, M1's two stacking arms and M2's substituted-top-mount arm all answer a mount the account stacked with
  `fusermount3`. What it costs: a second privileged move per misplaced attach, an unmeasured failed-rollback route,
  two checks per removal, and `doctor` text for a misplaced state. What removing it would simplify: the removal keeps
  the accepted misplaced-attach rule as the only rollback path and needs no staging-specific check, and the mount
  table is read once per call instead of twice. The design keeps both checks, and M1 and M2 keep their controls.
- The `user-cache` pin. A root configuration key, a boot-reconcile check and a per-removal check against the dataset's
  source and the mount root's identity exist because a bind made before that dataset mounts lands on the home underlay
  (F12). What it costs: one new root key, one boot step, two checks per removal and M5's three arms. What removing it
  would simplify: no root configuration for staging, no boot ordering step, and no per-removal source comparison.
  This one is not purely a defence against the account: it also catches a mount ordering accident, which the trusted
  account sentence does not make impossible, so dropping it needs a different answer to the ordering question.
- The wrapper's mount-namespace pair and boot-id check. The wrapper compares `/proc/self/ns/mnt`'s device and inode
  and the boot id with the handler's recorded ones before it removes a folder. What it costs: two record fields, one
  comparison, one refusal and M6's namespace arms. What removing it would simplify: the record carries no namespace
  pair, and the wrapper's removal path is one check shorter. It also catches an accidentally unshared wrapper, which
  is a tooling accident rather than an attack, so the sentence alone does not retire it.
- The folder's shape checks. The handler refuses a folder that is not owned by the pinned UID, not at the mode F1
  records, or not empty. What it costs: a `stat` of a few fields on every bind and the replacement arms. What
  removing it would simplify: the handler binds the folder as it finds it, which A6 says holds nothing but mounts,
  so no content can be at stake.
- The record's identity check before a sweep acts. The sweep re-reads the folder's identity after opening it from the
  handle and refuses on a mismatch. What it costs: one check per recovery and M6's foreign-namespace substitution
  arm. What removing it would simplify: the sweep acts on whatever the handle resolves to. It is also the backstop
  F29 names for a folder re-made in one transaction group with a reused object number, so it is a correctness check
  and not only a defence.

The rest of the design's refusals are not on this list because they answer something other than a hostile account: a
busy mount (F9), a shared checkout (F5), a missing capability (F29), a receipt that never arrives (A5), and the
destroy refusal a mounted dataset gives (F13) all protect the worktree's own data or the caller's own state.

## How the accepted sentences are met

- "Reuse only a verified staging mount on subsequent sweeps" has no work to do for a completed removal under Way A,
  which binds, verifies and unbinds within one invocation. It does bite for an interrupted one: a bind, and possibly
  a staged clone, outlives that invocation (M6), so the staging sweep opens the folder from the recorded handle
  (F29), re-verifies the folder's identity and the mount table's rows for the bind's source and parent before it
  acts, and reports instead when the handle no longer resolves. Way C, which keeps a bind across sweeps by
  design, verifies it before each move.
- "Test staging setup at boot and refuse shared or replaced staging and checkout mounts until a private setup is
  verified" is met twice: at boot reconcile, the pin of "Proposals the owner settled" is checked; at each removal,
  the handler refuses when the checkout's propagation is not private (the move would fail anyway, F5) or when the
  bind it made does not read private in the mount table (F23). The spec's wording asks for a boot test; this design
  adds the per-removal test beside it and says which sentence it strengthens rather than replaces.
- "Unmounted" as absence in every namespace holding the source, including binds and staging, is met by the receipt's
  all-namespace scan and by the wrapper's `rmdir` only after that receipt shows absence (A5).
- The acceptance item's two halves are met by M1 and M6: after a completed removal no mount and no entry remain;
  after an interrupted one, nothing under a mounted tree is deleted and the retained mount is tracked and reported.
- "The runtime files are a cache, not authority to act; rebuild them from live state and durable intent" is met
  for every field of the staged-folder record but one, and that field is the departure quoted in "The staged-folder
  record": no live state rebuilds the folder's handle once the handler's invocation has ended, and the project's
  answer "Record the file handle (Recommended)" accepted that one field ("Answers of 2026-11-19").

## What this proposal does not do

- It adds no decision that the accepted design or the project's answers of 2026-11-19 did not make. The answers
  settled the ways and the questions ("Answers of 2026-11-19"), and the proposals they left open are now the design
  ("Proposals the owner settled"). The ways not taken stay described as alternatives, and every measurement arm stays
  owed.
- It runs no VM measurement and no host probe beyond read-only reads of `findmnt`, `stat`, `zfs list`, `zfs get`,
  `rm --help` and `/proc/sys/kernel/random/boot_id`.
- It does not design the removal workflow, the mount handler's modes, the boot unit's ordering, the ABI gate, the
  all-namespace scan or the deferred-origin checks.
- It does not change the measurements record, the proposal, the plan or any wrapper code.

## Revisions

Each revision is named by the first commit that carries it, with that commit's committer timestamp
(`git log --format=%cI`). Later commits of one revision amend its text in place, and the entry says what they add, so
the name stays the revision's first commit rather than its last.

- `a709cd7f58` at `2026-11-19T18:02:34-05:00`: the first complete design, with its facts, ways, questions and
  measurement arms.
- `2258de74d8` at `2026-11-19T18:03:18-05:00`: names the plan's statement that the steps after step 9 wait on this
  design.
- `ff0a4019d9` at `2026-11-19T18:53:06-05:00`: the first revision, answering the gate review with a
  descriptor-anchored unmount and the questions split by creator and lifetime.
- `3972d4bf90` at `2026-11-19T18:53:29-05:00`: records the host's user-namespace knobs the design read.
- `0f0f61c15b` at `2026-11-19T19:38:48-05:00`: the previous revision, answering the first re-check: the recorded
  handle a later sweep recovers the folder with, the bind's own descriptor, the unbind's rule over the copies, the
  claim's place beside `v1`, the corrected mount-source mechanism, and the questions rebuilt as one choice among the
  ways plus three follow-ups.
- `ca9ca049fd` at `2026-11-19T20:25:12-05:00`: this revision, answering the second re-check: the folder's component
  derived on both sides from the clone's dataset GUID, the capability the recovery needs and the units that must keep
  it, the retained states split by cause, the corrected descriptor facts for a move's destination and for a copy
  with a mount under it, the durable staged-folder record that replaces the `/run` bind record, the questions rebuilt
  as three that stand alone in a dialog, and the readings of decided sentences moved into the veto list. Its later
  commits in the same revision add the arms those facts earn: a mount the account stacks between the bind and the
  move, a copy that carries a mount under it and survives the unbind, a sweep run without the recovery capability, a
  sweep of a folder a foreign namespace removed, the late mount a private `~/.cache` stops delivering, and a folder
  whose claim is gone; they also name which axis separates each pair of options in the first question, and keep a
  folder whose claim is gone instead of removing it.

## Sources read

- Linux, upstream tag `v6.8`, files fetched by URL at that tag: `fs/namespace.c`, `fs/namei.c`, `fs/pnode.c`,
  `fs/open.c`, `fs/dcache.c`, `fs/proc/fd.c`, `fs/proc/base.c`, `fs/proc_namespace.c`, `fs/exportfs/expfs.c`,
  `include/linux/exportfs.h`, `include/linux/namei.h`, `include/uapi/linux/mount.h`. `fs/fhandle.c` was fetched at
  the same tag into the re-checks' notes folders and into this revision's scratch, and is read there.
- OpenZFS, tag `zfs-2.4.4`, commit `71a9f9578616a90c3c14bb59629fb4d31bfd68d1`:
  `module/os/linux/zfs/zpl_super.c`, `module/os/linux/zfs/zfs_vfsops.c`, `module/os/linux/zfs/zpl_export.c`,
  `module/zfs/dsl_dataset.c`, `module/zfs/dsl_destroy.c`, `cmd/zfs/zfs_main.c`.
- This repository at `73eaad0846`: [decision][decision], [proposal][proposal], [measurements][measurements],
  [plan][plan], and `AGENTS.md`. The round's answers were read on 2026-11-19 from
  `doc/handover/cli-git-rust-implementation.md` in the main checkout, the paragraph beginning
  `THE ROUND WAS ASKED`.
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
  `zfs get -H -o property,value,source mountpoint,canmount,mounted zpcachyos/nosnap`, and `cat` and `stat` of
  `/proc/sys/kernel/random/boot_id`, whose value is not recorded here.

## Links

[decision]: ../decision/cli-git-zfs-worktree-design.md
[proposal]: cli-git-zfs-worktree-design-proposal.md
[measurements]: cli-git-zfs-measurements-owed-before-acceptance.md
[plan]: cli-git-zfs-worktree-implementation-plan.md
[decided-defaults]: cli-git-zfs-worktree-design-proposal.md#decided-defaults
