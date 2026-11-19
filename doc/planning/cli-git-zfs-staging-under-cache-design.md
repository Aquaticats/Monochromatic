# Staging under `~/.cache` and the wrapper's cleanup

Non-final, unapproved design proposal by an agent, written 2026-11-19T18:02:41-05:00. It designs the open item
"Staging under `~/.cache` and the wrapper's cleanup" of the accepted ZFS worktree design, and it makes no decision
that design did not make. It ran no ZFS command that changes anything, started no VM and no container, touched no
pool, and edited no other file.

The design is [the decision record][decision], the whole design is [the proposal][proposal], the evidence is
[the measurement record][measurements], and the build order is [the implementation plan][plan]. The plan lists the
item as step 9's blocker, "blocking for every unmount", and as writer design work
([plan][plan], "Open items and the step that needs each"), and step 9 records that it "also owes the interface that
unmounts a staged mount from the held descriptor". Steps 14, 16 and 17 wait on it as well
([plan][plan], step 9, "Open items").

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
- Staging reuse across sweeps (same section): "Reuse only a verified staging mount on subsequent sweeps; validate
  its identity and private propagation before each move."
- No unmount exists yet (same section): "The interface that unmounts a staged mount from the held descriptor, after
  the move, has no design yet, and no unmount runs from staging until it exists."
- Staged state is not absence (same section): "Track and report staged identity and location across interruption:
  absence at the slot does not mean the mount is gone. Treat absence from the handler's namespace as local detach,
  not completed unmount."
- Private staging and private parent ([proposal][proposal], "Mounts held by other namespaces"): "This descriptor
  route requires a verified private immediate parent and verified private staging, not private ancestors of the
  checkout", and "Private staging does not eliminate the all-namespace check."
- Staging counts as a live mount (same section): the design defines "Unmounted" as absence "in every mount
  namespace holding the clone's ZFS source, including binds and staging", and a "missing or mismatched unmount blocks
  removal outright".
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
- The boot unit does not establish staging ([proposal][proposal], "Boot and shutdown unit"): "It does not yet
  establish private staging: the folder's creation, its private bind and its cleanup have no design yet." Its stop
  pass "Unmount the verified managed mounts only; do not change durable intent", and it must "never force or detach"
  a busy clone.
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
- Placements this proposal leaves alone ([proposal][proposal], "One-time move and checkout boot protection"):
  "Hardening those existing mount targets is outside this proposal", `~/.cache` among them.
- The forced arm can change the staging rule ([proposal][proposal], "Forced removal and deferred origins"): "A
  force-unmount result would change two statements in this file: the busy-teardown rule that reports which clone
  holds the mount and never forces or detaches it ... and the staging step's unmount without force or lazy detach."
- The handler's record of slot identity is root's ([proposal][proposal], "Descriptor mount and unmount"): records
  live in `/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities`, and the handler never makes
  "privileged writes into the account's tree."

## Facts

Each fact names its source. Nothing here is a measurement taken on this host beyond the read-only reads named.

- F1. The account's `~/.cache` is the mount `zpcachyos/nosnap/user-cache`, file system `zfs`, propagation `shared`,
  mode 0700 owned by the account. Read on this host 2026-11-19 with
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
  (`guest-probe.py`, item 3, in `~/temp/agent/zfs-measurements-owed-before-acceptance-2026-11-18/keep/`), and item
  3's log reads the staging row as `private` ([measurements][measurements], item 3).
- F5. `move_mount` refuses `EINVAL` when the moved mount's current parent is shared (Linux `v6.8`,
  `fs/namespace.c:3163` to `:3166`, "Don't move a mount residing in a shared parent"). The
  same refusal was measured with a shared checkout, and the move succeeded with only the checkout private
  ([measurements][measurements], item 3).
- F6. Attaching a mount into a shared destination propagates it to that peer group and marks the moved tree shared
  (Linux `v6.8`, `fs/namespace.c:2335` to `:2349`). A move into a plain directory under the `shared` `~/.cache` mount
  (F1) is therefore not the measured route and is not private.
- F7. `umount2` resolves its argument with `LOOKUP_MOUNTPOINT | LOOKUP_FOLLOW` (Linux `v6.8`, `fs/namespace.c:1904`
  to `:1919`); `LOOKUP_MOUNTPOINT` means "follow mounts in the end" (`include/linux/namei.h:26`), and the walk
  steps into the mount at the final component (`fs/namei.c:2465` to `:2471`, `:2488` to `:2491`). A lookup that ends
  on a directory covered by a mount therefore resolves to that mount's root.
- F8. The unmount then requires that the resolved path is the root of its own mount in the caller's namespace:
  `can_umount` returns `EINVAL` otherwise (Linux `v6.8`, `fs/namespace.c:1871` to `:1886`, with `path_mounted` at
  `:1878` and `check_mnt` at `:1880`), and `EPERM` without `CAP_SYS_ADMIN` (`:1875`).
- F9. A non-lazy unmount refuses `EBUSY` when the mount has child mounts or more than two references (Linux `v6.8`,
  `fs/namespace.c:1791` to `:1793`; `propagate_mount_busy`, `fs/pnode.c:407` to `:421`; `do_refcount_check`,
  `fs/pnode.c:352` to `:355`). A working directory or an open file raises that count; item 4 measured the refusal
  with a cwd holder in the staged clone and success after the holder exited
  ([measurements][measurements], item 4).
- F10. An open descriptor holds a reference on the mount it was opened through: `vfs_open` stores the looked-up
  `path` in the file (Linux `v6.8`, `fs/open.c:1084` to `:1087`, and `:1175`), and a `path` holds its mount. An
  unmount performed while a descriptor on that same mount is open therefore reaches the reference count F9 refuses.
  This is read from the kernel's data model, and M2 measures it; the design's "release descriptors that would hold
  it busy" names the same effect.
- F11. Deleting or renaming a name that is a mountpoint fails `EBUSY` only when the mountpoint is in the caller's own
  mount namespace: `__is_local_mountpoint` walks `current->nsproxy->mnt_ns` (Linux `v6.8`, `fs/namespace.c:753` to
  `:770`), and the `rmdir`, `unlink` and `rename` paths test it (`fs/namei.c:4200` to `:4202`, `:4330` to `:4331`,
  `:4861` to `:4863`). The handler and the account share the host namespace
  ([proposal][proposal], "ZED reconciliation and mount safety"), so after the move the account can neither rename nor
  remove the bound staging root or the folder that holds the staged clone. The window the decided default gives up
  lies before the attach: an empty folder can be replaced, and an ordinary ancestor directory can be renamed at any
  time, which carries the bind and any staged mount with it while a fresh lookup of the old path fails.
- F12. Mounting at a path hides whatever mount already holds that path. VM item 9 measured `zfs mount -a` attaching a
  path descendant to the home underlay and the later checkout attach hiding it while its mount ID stayed
  ([measurements][measurements], item 9; [proposal][proposal], "One-time move and checkout boot protection"). That is
  what a bind made before the `user-cache` mount would suffer.
- F13. A mounted ZFS dataset holds a long hold on it for the life of its superblock: `zpl_get_tree` takes it
  (OpenZFS `zfs-2.4.4`, `module/os/linux/zfs/zpl_super.c:868`), `zfs_umount` releases it through `dmu_objset_disown`
  (`module/os/linux/zfs/zfs_vfsops.c:1526` to `:1553`, `module/zfs/dsl_dataset.c:1035` to `:1045`), and head destroy
  refuses `EBUSY` while the hold stands (`module/zfs/dsl_destroy.c:766` to `:777`). Item 15 measured the refusal as
  `dataset is busy` while reader namespaces held the clone, and success after they stopped
  ([measurements][measurements], item 15).
- F14. `umount2` has no form that takes a descriptor. The descriptor-taking mount calls are `open_tree` (with
  `OPEN_TREE_CLONE` and `AT_EMPTY_PATH`), `move_mount` (with `MOVE_MOUNT_F_EMPTY_PATH` and
  `MOVE_MOUNT_T_EMPTY_PATH`) and `fsmount` (Linux `v6.8`, `fs/namespace.c:2697` to `:2715`, `:4097` to `:4106`;
  `include/uapi/linux/mount.h:64`, `:72`, `:75`). Propagation is changed by `mount` with `MS_PRIVATE`, which
  requires the path to be the root of its own mount (`do_change_type`, `fs/namespace.c:2549` to `:2558`).
- F15. A receipt names the operation, the matching nonce, the outcome and the mount identity, and the request
  grammar carries no path, dataset name or mount flag ([proposal][proposal], "Requests, results and ordering").
- F16. The wrapper already owns a claim-before-create staging pattern: a container named with a prefix and a random
  identifier beside its destination, claimed in a journal root under the Git common directory before it exists, keyed
  to an owner, and reaped only when the owner is dead
  (`package/git-policy/cli/src/native/worktree_copying/worktree_copy_stage_claim.rs:1` to `:26`, `:448` to `:472`;
  `worktree_copy_selection.rs:46` to `:48`; `worktree_copy_stage.rs:205` to `:213`).
- F17. The wrapper has the descriptor and identity helpers this design needs: `open_root`, `open_directory_at`
  (which refuses a mount crossing with `EXDEV`), `open_directory_beneath`, `handle_path`, `identity_of` (device,
  inode and birth time), `make_directory_at` and `remove_at`
  (`package/git-policy/cli/src/native/subvolume/fs_beneath.rs`, `:184`, `:203`, `:230`, `:307`, `:384`, `:267`,
  `:335`).
- F18. The wrapper already maps a descriptor to its mount and its source: `opened_file_of_our_own` reads the
  descriptor's record for the mount ID and the mount table for the type and source
  (`package/git-policy/cli/src/native/process/open_file_of_a_descriptor.rs:48` to `:56`, `:91` to `:175`), and
  `mount_of` and `processes_using_until` give the mount of a path and the holders this design's verification repeats
  (`process_use.rs:313`, `:552`).
- F19. The measured staging in items 3 and 4 was one path on the guest's root file system, covered by the moved
  clone mount and unmounted by path, not this location, not a wrapper-created folder, and not a folder per removal
  ([measurements][measurements], items 3 and 4; `guest-probe.py`, items 3 and 4).
- F20. The wrapper's `REPORT_SCAN_CEILING` is the health report's own scan bound, measured on this host on
  2026-11-09, and not a shipped mount or unmount limit
  (`package/git-policy/cli/src/native/service/wait_ceilings.rs:1260` to `:1281`). Step 9 introduces the unmount
  deadline and step 22 sizes it ([plan][plan], "Wrapper code the design reuses", step 9, step 22).

## Assumptions

- A1. The wrapper and the root handler share the host mount namespace, as the design states, so F11 applies to both
  and a staged mount is visible to the account at its staging path.
- A2. The account home the staging target lives in is the passwd home of the pinned UID, not `$HOME` or
  `$XDG_CACHE_HOME`, because a root handler has neither and the answer names `~/.cache`.
- A3. `~/.cache` stays a mount of `zpcachyos/nosnap/user-cache` at boot and is not replaced by the account between
  setup and a removal. If it is replaced, the identity checks refuse rather than adopt it.
- A4. The account does not deliberately sabotage its own staging, which is the same-account residual the delegated
  destroy decision already accepts ([proposal][proposal], "Clone removal identity protection").
- A5. The removal flow obtains a receipt for its `unmount` nonce before the wrapper removes anything, and a removal
  whose receipt never arrives keeps its staged state reported.
- A6. The staging folder is only ever a mountpoint carrier: nothing but the moved mount is put inside it, so a
  successful unmount leaves it empty.
- A7. A staged mount's presence in the host namespace is acceptable to the account, since the folder sits in the
  account's own `~/.cache`, at the mode F1 records.

## Limits

- L1. No run exercised this design. Every VM statement in [the measurement record][measurements] comes from guest
  `6.8.0-147-generic` with OpenZFS `2.4.4-1`; nothing ran on `7.3.5-1-cachyos`, and the wrapper, handler and
  binding have not been built.
- L2. Kernel citations are to upstream Linux `v6.8`, fetched by URL at that tag. The guest kernel is Ubuntu's
  `6.8.0-147-generic`, whose patch set is not upstream `v6.8`, and the host kernel is `7.3.5-1-cachyos`, which no
  citation here covers.
- L3. No measurement exists for an unmount through a name anchored at a held descriptor, for a bind of this location,
  for a wrapper-created folder, for two removals at once, or for this location's boot ordering.
- L4. Item 2 measured an account renaming `.claude` between the open and the attach; item 4 measured a rename after
  the open. Neither measured a substitution between the staging identity check and the move at this location.
- L5. Free-running interleavings, concurrent same-name replacement, a failed reattach and crash recovery were not
  measured ([measurements][measurements], "What this repeat did not show").
- L6. The forced arm is still owed, and if a force-unmount works it changes the staging unmount rule this proposal
  keeps ([proposal][proposal], "Forced removal and deferred origins").
- L7. Nothing here sizes a shipped deadline, and no timing claim is made. Step 22 sizes the waits.
- L8. The staging route's cost in account-visible churn is unmeasured: the proposal states no number, and this
  proposal adds none.

## The parts that need a design

The item names the staging target's creation, the private bind, the wrapper's cleanup, the unmount interface, and the
ordering against the `user-cache` mount. Each is stated here as the shape any way must fill, not as a decision.

### The staging target and who creates it

The target is a directory under the account's `~/.cache` that the root handler can bind and into which it can move a
clone mount. It must be found by a no-crossing walk from the pinned home (F17), its identity recorded before the
first bind and rechecked before every move, and it must not be created by a request, since no request carries a path
(F15). The design gives neither the name nor the creator, and it forbids a root handler writing into the account's
tree (its slot-identity rule), which is the constraint any creator must meet.

### The private bind

The bind is what makes the staging target a mount instance whose propagation the handler sets, because attaching a
mount under a shared parent propagates it (F6) and `~/.cache` is shared (F1). F4 records the measured shape. The bind
is a mount in the host namespace at a path inside the account's tree, which is the exposure the decided default
accepts ([proposal][proposal], "Decided defaults").

### The unmount call from staging

The unmount must not force or lazily detach, must not act on a path the account can change, must release descriptors
that would hold the mount busy, and must end with the mount absent from every namespace (F9, F13, and the design's
sentences quoted under "Decisions of the accepted design that bound this proposal"). F7, F8 and F14 say what the
kernel offers: a path whose final component is stepped into the mount, no descriptor-only unmount, and a reference
count that an open descriptor on the same mount raises.

### What the wrapper removes and when

The answer's words are "make the wrapper clean after itself". The cleanup must not recurse over a mount, must not
remove a name it cannot prove it made, must cover an interrupted or failed removal, and must leave a retained staged
mount tracked and reported (the acceptance item quoted under the decisions). The wrapper already owns a claim, owner
and reap pattern for staging containers (F16), which is the nearest existing owner of this responsibility.

### Ordering against the mount of `~/.cache`

A bind made before `zpcachyos/nosnap/user-cache` mounts lands on the home underlay and is hidden when the dataset
mounts later (F12). Nothing orders the two today, and the design's open item says so. Any way must either refuse to
bind until the recorded mount identity is present or be created after that mount exists.

## Ways to meet it

Each way covers the parts named in "The parts that need a design". Costs are enumerated work, not sizes or times;
none is measured.

### Way A: a claimed folder per removal inside one wrapper-made staging root, bound for that removal

- The wrapper creates `<home>/.cache/monochromatic-git-policy-cli/stage` once, at the mode F1 records, and claims
  each removal's folder `<stage>/<unmount nonce>` in the existing stage-claim journal before the folder exists.
- The handler, on the first `unmount` intent for a clone, opens the parent chain from the pinned home, checks the
  `stage` identity against root state, binds the removal's folder and makes it private, verifies its row reads
  `private`, and moves the clone onto it.
- The unmount resolves the folder's name under the held staging descriptor, requires the lookup to land on the root
  of the mount whose identity matches the staged record (F7, F8), closes every descriptor on the clone first (F10),
  and refuses on any mismatch.
- The handler removes the bind once that removal's clone is unmounted, then the wrapper removes the claimed folder by
  `rmdir`, never recursively, and reports anything it cannot prove.

Pros: no privileged write into the account's tree; one identity check and one claim to keep; concurrent removals use
their own folders and their own binds, so no new lock and no stacking; every decided sentence stays true; the
existing claim and reap pattern is extended rather than replaced; the retained staged state the busy decision allows
is exactly a folder whose claim is unproven and stays reported.

Cons: the account can replace the empty folder before the attach and rename an ordinary ancestor at any time; the
identity check catches a replacement made before it, and the post-attach canonical check catches the rest; the
unmount reads a name inside the account's tree, so the design's "never on a path inside the folder the account can
change" is met for every component but the final one (Q1); a wrapper crash between claim and folder removal leaves a
folder and a stale claim that a later pass must settle.

Build cost: one wrapper module for the claim, creation, identity check, `rmdir` and notices; one handler step for
bind, verify, move, unmount and unbind, with its receipt and state fields; a new deadline owned by step 9 and sized
by step 22 (F20); `doctor` text for a staged mount, an unprovable claim and a bind whose path no longer resolves.

Test cost: device-free tests for the claim, the folder grammar, `rmdir`-only removal, each refusal's text and the
identity mismatch; the VM arms M1, M2, M3, M5, M6 and M7.

### Way B: a root-made target per removal, bound for that removal

- Root creates `<home>/.cache/monochromatic-git-policy-cli/handler/<nonce>` mode 0700 owned by root, records its
  identity under `/var/lib/monochromatic-git-policy-cli/`, binds it, moves the clone onto it, unmounts, removes the
  bind and removes the target.
- The wrapper creates nothing; it records the removal and waits for receipts.

Pros: the account cannot plant content in the target, and the identity it is checked against comes from root's own
record rather than from an account-written claim; the account's rename of the target's name is still refused by the
descriptor identity check before the move, as in Way A.

Cons: root writes a directory into the account's tree, which is the write the design's slot-identity rule avoids;
root owns an object under an account-owned parent, which the account can still remove or rename, so the protection
buys less than it appears to; the wrapper must report a state it cannot create, and `doctor` must explain a
root-owned entry in the account's cache; a wrapper removal and a root target creation must be ordered by a protocol
that does not exist today.

Build cost: Way A's handler work plus target creation, ownership and removal, a root-state record per target, a
refusal path for an account-removed parent, and the wrapper's side reduced to waiting and reporting.

Test cost: Way A's arms plus M8, and the same device-free tests for the wrapper's reduced side.

### Way C: one root-made target bound once per boot, reused by every removal

- Root creates one target under `~/.cache`, binds it and makes it private at boot reconcile, records its identity,
  and keeps it until the stop pass. Each removal moves its clone into a folder under that target.
- The unmount is Way A's; the bind's removal happens only at shutdown.

Pros: the bind exists before any removal, so a removal never waits for a mount transition; the boot unit's start
reconcile gains one setup step instead of an on-demand step; the closest match to the measured single-path shape.

Cons: a mount in the host namespace for the whole boot; one account rename of an ordinary ancestor makes the bind
unreachable by name for the rest of the boot, and under the strict rule it can then be released only by a reboot
(Q2); every concurrent removal shares one target, so either a global staging claim or serialisation of all unmounts
is added, and without one the second move stacks over the first; the boot unit must order the bind after the
`user-cache` mount, which nothing does today (F12); the stop pass must unmount a staged clone before the bind, and
must never force it.

Build cost: Way B's creation work moved to the boot unit, plus a shared-path rule (to be decided), plus stale-bind
recovery and reporting, plus boot ordering in the boot unit and its `doctor` text.

Test cost: Way A's arms plus M7's shared-path control, M5 and the stale-bind arm of M4.

### Way D: no staging at all, a private namespace teardown instead of an unmount

- The handler creates a child process in a new private mount namespace, moves the clone mount into it by descriptor,
  and lets the child exit; the mount goes with the namespace and no `umount2` runs.
- Nothing is created under `~/.cache`; there is no staging folder, no second path, and no bind.

Pros: recovers the protection the decided default gives up, since no mount ever lands in an account-controlled tree;
no staging path, no claim, no cleanup; the smallest state to track.

Cons: it reverses the two decided answers it would replace, the location and the staging default, so it is the
owner's to accept, not a writer's; the detach is lazy in effect, which the design's unmount rule excludes, and it
removes the busy refusal the removal workflow uses to keep history behind a holder, because the dataset stays busy
through references while no namespace lists the mount (F9, F13); nothing measured it.

Build cost: handler-only code for the namespace, the move and the exit, plus a new statement of what "unmounted"
means, plus the deletion of the staging machinery steps 9 and 18 plan.

Test cost: M9, and the negative control that a holder's reference still blocks the destroy after the teardown.

## Ranking

Ranking: A > B > C > D.

- A > B, because A reaches the same refusal guarantees without a privileged write into the account's tree and without
  a root-owned object under an account-owned parent that the account can still remove. B's extra protection covers
  content, and the target holds no content; the exposure both ways is a replaced or renamed name, which the
  descriptor identity check already refuses.
- B > C, because C's bind lives for a whole boot and one account rename of an ordinary ancestor leaves it
  unremovable by name until the machine restarts, while B's exposure lasts one removal. C also puts every removal on
  one path, which needs a global claim or serialised unmounts that B does not, and C moves a mount transition into
  boot reconcile, where nothing currently orders it after the `user-cache` mount.
- C > D, because D reverses two decided answers and removes the busy refusal the removal order depends on, while C
  keeps every decided sentence true and pays with a persistent mount and a boot-time setup that `doctor` can report.

## Owed measurements in a throwaway VM

Each runs in a disposable guest on a file-backed pool, as the [measurement record][measurements] fixture does, with
the built wrapper and the built handler, as account and as root, never against this host's pool and never with its
`/dev/zfs`. A run records `uname --kernel-release`, `zfs version` and `/sys/module/zfs/version` first, and keeps its
commands and output. Each arm names its control, because a null result is worth nothing without one.

### M1. The name-anchored unmount, staged and refused

Set up a verified clone at a slot, a private bind on its staging folder, and the move. Arms:

- Positive: no holder. Resolve `<held staging descriptor>/<folder>` and unmount without force or lazy detach; assert
  exit 0, the clone's mount ID absent from every namespace's `mountinfo`, the folder an empty directory, and
  `zfs destroy` of the clone then exits 0. A destroy attempt before the unmount is the control that the destroy
  refusal F13 is observable in this fixture.
- Busy: a cwd holder inside the staged clone. Assert `EBUSY`, the mount still staged, and `zfs destroy` refusing
  `dataset is busy`.
- Replaced before the bind: the account removes the empty folder after the claim and before the handler binds it.
  Assert the identity check refuses and no mount lands; the positive arm in the same run is the control.
- Replaced between the bind and the move: the account replaces the folder after the handler holds its descriptor.
  Record where the mount lands (the dentry the descriptor holds is the expected place), and assert the post-attach
  canonical check refuses. The measured analogue is item 2's renamed `.claude` arm. What the handler does next is a
  writer's design point: move the verified mount to the folder's current verified dentry, or retain and report it.
- Ancestor renamed after the move: the account renames an ordinary ancestor of the staging root. Assert the anchored
  unmount through the held descriptor still reaches the staged mount, and record what a fresh lookup and `doctor`
  report. The undisturbed positive arm is the control.
- Other folder: a second clone staged in a second folder. Assert unmounting one leaves the other mounted.

### M2. The descriptor-only unmount

Repeat the positive arm with `umount2("/proc/self/fd/<clone root descriptor>", 0)` and record the errno, then close
the descriptor and record that no call takes a descriptor any more (F14). The control is the M1 positive arm in the
same run; its outcome decides whether Q1's second option can ever be used.

### M3. The private bind is required at this location

- Positive: bind the staging folder, make it private, move the clone, and check the folder's mount row reads
  `private` in `/proc/self/mountinfo`; assert a reader namespace started before the move sees no clone at the staging
  path, as item 3's private arm did.
- Negative: the same move, with the checkout private, into a plain directory under the shared `~/.cache` mount
  (F1, F6). Record whether the moved mount appears at the staging path in a peer namespace and whether its row reads
  `shared`. This is the measurement that shows the bind is what makes the target private at this location, which item
  3 measured only for the checkout side.
- Second control: the same positive arm with the checkout shared, to reproduce F5's `EINVAL`.

### M4. Removing the bind, and what the account can still do

- Positive: after M1's unmount, remove the bind by resolving its name under a held descriptor on the directory that
  holds it; assert exit 0, the bind's mount ID gone, and the wrapper's `rmdir` then succeeding.
- Account arm: while the bind exists, the account's `rmdir` of the staging root returns `EBUSY` (F11), recorded as
  the kernel's own protection in the shared namespace.
- Ancestor arm: the account renames an ordinary ancestor while no descriptor on the staging root is held, as
  between removals with a bind still in place. Record whether the bind can still be reached or removed, and what
  `doctor` reports.
- Unreachable arm: make the bind unreachable by name (rename the ancestor before any descriptor is held) and record
  what the handler can and cannot do, which is Q2's input.

### M5. Ordering against the `user-cache` mount

- Negative: mount the pool's home without the `user-cache` dataset, or delay that dataset's mount, then run the
  handler's staging step. Assert it refuses, creates no bind and writes nothing on the home underlay, and that
  `doctor` names the missing dataset mount.
- Positive: with the dataset mounted, assert the bind lands on the dataset's directory: the recorded identity's
  device is the dataset's, and the mount table names `zpcachyos/nosnap/user-cache`.
- Control: the ordinary boot order, which must reach the positive arm on the first attempt.

### M6. Interruption and the cleanup pass

- Kill the wrapper after the move and before the unmount. Assert the staged mount is reported, the claim survives,
  no later pass removes the folder, and no reattach happens (the no-implicit-remount default).
- Kill after the unmount and before the folder removal. Assert the next pass removes the claimed folder only after
  proving it holds no mount in the host namespace, and prints a notice.
- Controls: a folder holding a foreign file is kept and reported; a folder with no claim is not removed; a folder
  whose recorded identity no longer matches is kept and reported.

### M7. Two removals at once

- Stage two clones in two folders of Way A's shape, unmount both, remove both folders, and assert each receipt names
  its own nonce and folder and that neither unmount touched the other mount. The control is the serial case.
- For Way C's shared target, record the second move covering the first at one path, which is the measurement that
  decides whether a global staging claim is needed.

### M8. A root-made target

- Root creates the target, the account then renames it, removes it and replaces it with an empty directory, and the
  handler is asked to proceed. Assert the identity check refuses each time, that no mount is made on the replacement,
  and that `doctor` names the state.
- Control: the undisturbed target proceeds, and the account's inability to write inside a 0700 root-owned target is
  recorded with a positive control that the account can write beside it.

### M9. The namespace teardown

- Move a clone into a private child namespace and let the child exit. Assert the mount is gone from every
  `mountinfo`, then record whether `zfs destroy` succeeds, and repeat with a cwd holder in the clone to record
  whether the destroy still refuses `dataset is busy`. The control is M1's positive and busy arms in the same run.

Each way needs M1, M2, M3, M5 and M6. Way A additionally needs M4's positive, account and ancestor arms and M7's
serial arm. Way B needs M4's unreachable arm and M8. Way C needs M5's delayed-mount arm, M7's shared-path arm and
M4's unreachable arm. Way D needs only M9 and the control it names.

## Questions for the owner

The plan calls the item writer design work, so these are the points where a writer would otherwise have to add a
decision the accepted design did not make. Each is worded so its options can go into the question tool as they
stand.

### Q1. Which unmount may the handler perform from the staging folder?

The design forbids an unmount "on a path inside the folder the account can change", and F7, F10 and F14 say what the
kernel allows. This decides the blocking part of the item, and every way but D depends on it.

1.  "Name under a held descriptor (Recommended)": resolve the staged folder's name under the held staging descriptor,
    require the lookup to land on the root of the mount whose recorded identity matches, and refuse anything else.
    Pros: no force or lazy detach; the account cannot mount in the host namespace, so it cannot steer the call to
    another mount (the scoped-delegation decision grants none), and after the attach it cannot replace the name
    either (F11), so the only window is the empty-folder one the descriptor checks already cover.
    Cons: the last component is a name inside the account's tree, so the decided sentence is met for every component
    but the final one, and M1 and M2 must pass.
2.  "Descriptor only": unmount `/proc/self/fd/<clone root descriptor>`.
    Pros: the literal reading, with no name at all.
    Cons: the descriptor's own reference makes a non-lazy unmount `EBUSY` (F10), and with it closed no call takes a
    descriptor (F14), so M2 is expected to refuse and the removal then blocks.
3.  "Namespace teardown": no unmount; move the clone into a private namespace and let it exit.
    Pros: no staging path and no account-controlled target at all.
    Cons: reverses the decided staging location and default, and detaches lazily in effect, which removes the busy
    refusal the removal order uses (F9, F13); nothing measured it.
4.  "Refuse every unmount from staging": keep the sentence literal and never unmount there.
    Pros: no new rule.
    Cons: step 9 builds a step that can never work, and an authorized removal can never reach the destroy order
    "B: Destroy at once" chose.

Ranking: 1 > 2 > 3 > 4. 1 beats 2 because 2 is expected to refuse and 1 reaches the same guarantee with one refused
outcome no worse than 2's. 2 beats 3 because 2 leaves every decided sentence in place while 3 replaces two of them
for an unmeasured mechanism. 3 beats 4 because 3 at least completes a removal, while 4 leaves the removal route
unfinishable and the staging step pointless.

### Q2. What may remove a staging bind the wrapper can no longer reach by name?

After an ancestor rename the bind has no path, F7 and F14 leave no form that names it, F11 explains why the account
can move it out of reach, and only a detach or a restart remains.

1.  "Report and reboot (Recommended)": nothing detaches a bind; the handler refuses the removal, keeps the staged
    state, and `doctor` prints the bind, its mount ID and the ancestor that moved.
    Pros: no force and no lazy detach anywhere, which is what the design says; the next boot starts clean because a
    mount namespace is not persistent.
    Cons: a reboot is the only remedy, and the account can cause that state by renaming an ordinary directory.
2.  "Detach the bind only": allow `MNT_DETACH` for a staging bind, never for a clone mount.
    Pros: no reboot, and the bind carries no worktree data.
    Cons: adds a second unmount rule beside the one the design states, and needs its own proof that a detach cannot
    hide a clone mount's busy state.
3.  "Boot-time force clean": the boot unit detaches any staging bind before it reconciles.
    Pros: bounded to boot, where nothing is running in the account's session yet.
    Cons: needs the same detach proof, and it silently repairs a state `doctor` would otherwise report.

Ranking: 1 > 3 > 2. 1 beats 3 because 3 adds a privileged detach for a state that only a reboot can be sure to
clear. 3 beats 2 because 3 confines the detach to boot while 2 makes it available during a live removal.

### Q3. Who creates the staging target?

The design forbids a root handler's privileged writes into the account's tree, and the answer names a location inside
it.

1.  "The wrapper, at the start of a removal (Recommended)": the wrapper creates and claims the folder, the handler
    binds it.
    Pros: no privileged write; the creator is the process that later removes the folder; the existing claim pattern
    fits.
    Cons: a removal waits for its folder to be made, and a wrapper crash between claim and bind leaves a claimed
    folder for the next pass.
2.  "The root handler, per removal": root creates its own target and removes it.
    Pros: the target's identity comes from root's record and the account cannot plant content in it.
    Cons: a privileged write into the account's tree, a root-owned object the account can still remove, and a
    protocol between wrapper and handler that the design does not have.
3.  "The boot unit, once": root creates one target at boot.
    Pros: no per-removal creation at all.
    Cons: the same privileged write, plus a target that must outlive the `user-cache` mount ordering (F12) and cannot
    be removed before the boot ends.

Ranking: 1 > 2 > 3. 1 beats 2 because 2 buys content protection the target does not need and pays with the write the
design avoids. 2 beats 3 because 2's object lives one removal while 3's lives a boot and carries the ordering
problem into boot reconcile.

### Q4. How long may the staging bind live?

1.  "For one removal (Recommended)": bound when that removal's unmount begins, removed when it ends.
    Pros: the shortest exposure to an ancestor rename; no shared path between concurrent removals; nothing to order
    at boot.
    Cons: a bind transition at each removal, and a removal that fails leaves the bind for the next pass to report or
    remove.
2.  "Until no staged mount remains": kept while any removal has a staged mount, removed by the last one.
    Pros: fewer transitions under back-to-back removals.
    Cons: needs the "is any staged mount using this bind" scan and the concurrency rule that scan implies.
3.  "For the boot": bound at boot, removed at shutdown.
    Pros: no transition on the removal path.
    Cons: the stale-bind problem of Q2 for a whole boot, and the boot ordering of F12.

Ranking: 1 > 2 > 3. 1 beats 2 because 2 adds a scan and a rule for a saving that is unmeasured. 2 beats 3 because 3
puts the longest exposure on the object the account can rename out of reach.

## What a writer can settle without asking

- The staging root's name under `~/.cache`, as long as it is one readable component that says what it holds, the
  folder is account-owned at the mode F1 records, and no request carries it.
- That the account home is read from the pinned UID's passwd entry, not from `$HOME` or `$XDG_CACHE_HOME` (A2).
- The folder grammar per removal: the unmount nonce, bounded ASCII, never a user-supplied component, claimed before
  creation through the existing journal root (F16).
- The identity fields checked before each move and before each removal: device, inode, birth time, owner, mode,
  access and default ACLs from the held descriptor, and the mount row's propagation and mount ID (F17, F18).
- The handler's state record for the bind: its mount ID, its source dataset GUID from the binding, and the identity
  of the folder it covers, written under `/var/lib/monochromatic-git-policy-cli/` beside the slot identities.
- The receipt and notice wording for a staged mount, a mounted folder that cannot be unmounted, a bind whose path
  no longer resolves, and a folder the wrapper cannot prove it made.
- That cleanup is `rmdir`-only and never recursive, that it stops at any mount, and that it runs from the existing
  reclaim pass keyed to a dead owner rather than from a timer.
- That the wrapper never removes the staging root while a bind covers it, and that the handler never removes a
  folder it did not bind.
- That the unmount, bind and unbind waits share the removal deadline step 9 introduces and step 22 sizes (F20).
- That `doctor` prints the staging folder, its live bind's propagation, any staged mount with its mount ID and
  dataset, and the next safe step, and that it changes nothing, as the `doctor` decision quoted under "Decisions of
  the accepted design that bound this proposal" states.
- The order inside a removal: bind, move, verify, unmount the clone, remove the bind, remove the folder, then the
  receipt that names the outcome.
- What the handler does with a staged mount the post-attach canonical check cannot find at the folder: move it to the
  folder's current verified dentry when that holds no mount, or retain and report it. This is the staging analogue of
  the Misplaced attach decision, whose rollback rule was written for a slot.

## What this proposal does not do

- It adds no decision the accepted design did not make. The ways, the ranking and the questions are proposals, and
  none of them is adopted here.
- It runs no VM measurement and no host probe beyond read-only reads of `findmnt`, `stat`, `zfs list` and `zfs get`.
- It does not design the removal workflow, the mount handler's modes, the boot unit's ordering, the ABI gate, the
  all-namespace scan or the deferred-origin checks.
- It does not change the measurements record, the proposal, the plan or any wrapper code.

## Sources read

- OpenZFS, tag `zfs-2.4.4`, commit `71a9f9578616a90c3c14bb59629fb4d31bfd68d1`: `module/os/linux/zfs/zpl_super.c`,
  `module/os/linux/zfs/zfs_vfsops.c`, `module/zfs/dsl_destroy.c`, `module/zfs/dsl_dataset.c`,
  `module/zfs/zfs_ioctl.c`.
- Linux, upstream tag `v6.8`, files fetched by URL at that tag: `fs/namespace.c`, `fs/namei.c`, `fs/pnode.c`,
  `fs/open.c`, `fs/proc/fd.c`, `fs/proc/base.c`, `include/linux/namei.h`, `include/uapi/linux/mount.h`.
- This repository, at `73eaad0846`: [decision][decision], [proposal][proposal], [measurements][measurements],
  [plan][plan], and `AGENTS.md`.
- Wrapper code: `package/git-policy/cli/src/native/subvolume/fs_beneath.rs`,
  `package/git-policy/cli/src/native/subvolume/subvolume_remove_walk.rs`,
  `package/git-policy/cli/src/native/process/open_file_of_a_descriptor.rs`,
  `package/git-policy/cli/src/native/process/process_use.rs`,
  `package/git-policy/cli/src/native/worktree_copying/worktree_copy_stage_claim.rs`,
  `package/git-policy/cli/src/native/worktree_copying/worktree_copy_stage.rs`,
  `package/git-policy/cli/src/native/worktree_copying/worktree_copy_selection.rs`,
  `package/git-policy/cli/src/native/service/wait_ceilings.rs`,
  `package/git-policy/cli/src/native/health/doctor_filesystem.rs`.
- The measurement harness item 3 and item 4 ran: `guest-probe.py` in
  `~/temp/agent/zfs-measurements-owed-before-acceptance-2026-11-18/keep/`, read for the staging bind's creation and
  the unmount calls.
- This host, read-only, 2026-11-19: `findmnt --output TARGET,SOURCE,FSTYPE,PROPAGATION --target /home/user/.cache`,
  `stat --format '%n %i %U %a' /home/user/.cache`,
  `zfs list -H -o name,mounted,mountpoint,canmount,com.sun:auto-snapshot zpcachyos/nosnap/user-cache`,
  `zfs get -H -o property,value,source mountpoint,canmount,mounted zpcachyos/nosnap`.

## Links

[decision]: ../decision/cli-git-zfs-worktree-design.md
[proposal]: cli-git-zfs-worktree-design-proposal.md
[measurements]: cli-git-zfs-measurements-owed-before-acceptance.md
[plan]: cli-git-zfs-worktree-implementation-plan.md
[decided-defaults]: cli-git-zfs-worktree-design-proposal.md#decided-defaults
