# The holder scan across mount namespaces, measured 2026-11-22

**Note,
2026-11-25:**
the program this record measured as
`mount-git-worktree-clone-datasets holder-scan` is a mode of
`monochromatic-git-policy-cli` now,
reached as `monochromatic-git-policy-cli zfs-root holder-scan --dataset <name>`
or through one root-owned file under the installed ZED history event's name
(`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`).
The text below keeps the spelling of the program it measured.

Status:
written by an agent in one disposable `mvm` `libvirt` guest,
unapproved and non-final.
Only the owner settles a value.

This record holds what one scan of a ZFS worktree clone's holders across mount
namespaces costs and what the guest arms of plan step 9 showed,
so the bounds the scan ships are read against something measured
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
"Step 9:
Unmount from a fresh process,
the staging folder and reattach";
`package/git-policy/cli/SPEC.md`,
"The all-namespace holder scan of a ZFS worktree clone").

## Where and how

The guest was Ubuntu 24.04 through
`package/git-policy/cli/e2e/mount-git-worktree-clone-datasets-guest.ts` with
`--name zfs-step9-holder-scan-12`,
OpenZFS 2.4.4,
and the crate built at `-release` inside it.
The arms are
`src/native/binary/binary_zfs_holder_scan_guest_tests.rs`,
run through the driver's holder-scan step,
and the log the driver pulled back is
`holder-scan-arm.log`.

The fixture is the design's layout:
a checkout dataset mounted at `/root/holder-scan-checkout`,
its sibling container,
and one clone of the checkout dataset's snapshot carrying a mount request.
The clone is attached by the built sweep,
and every count below is read from the scan's own result,
parsed by the same `parse_holder_scan` a later step reads it with.

## The arms and the lines they printed

    SCAN baseline own=1 holders=0 incomplete=0 guid=12937390079315674186
    SCAN HOLDER other-namespace=1 namespace=mnt:[4026532403] mount=/root/holder-scan-checkout/.claude/worktrees/one
    SCAN confined-consumer=held holders=2 namespace=mnt:[4026532405]
    SCAN churn scans=20 holder-kept=20 task-ended-named=0
    MEASUREMENT holder-scan-ms=small=2 large=4 tasks-small=212 tasks-large=412
    SCAN pinned-namespace invisible=true holders=0 incomplete=0 note=the-scan-cannot-see-a-namespace-no-task-holds
    SCAN divergent-root=unavailable reason=this-guest-refused-a-task-under-another-process-root
    SCAN done

- The baseline line is the scan of the handler's own namespace:
  one mount of the clone,
  at the slot,
  no foreign holder,
  and no incomplete read.
- The holder line is one `unshare --mount` task that mounted the clone in a
  mount namespace of its own:
  the scan names that namespace and the mount point it holds.
- The confined line is an unprivileged user namespace,
  whose own mount namespace bound the clone's root at
  `/mnt/holder-scan-confined`:
  the scan names it as a second holder.
  Pi is not installed in this guest,
  so the plan's confined Pi consumer is stood in for by that namespace,
  and this record does not claim the Pi driver.
- The churn line is twenty scans while short-lived processes came and went:
  every one of the twenty still named the holder the scan had already found,
  and none of them named a task that ended as an incomplete read in this guest.
- The pinned line is a namespace held open by a handle after its last task
  ended:
  the scan names no holder for it and says nothing about it,
  which is the limit this record keeps.
  The run's own line says the scan cannot see a namespace no task holds.
- The divergent-root arm reported itself unavailable in this guest and the
  rounds before the last one measured what it was looking for:
  a task chrooted after its mount was made answered `EINVAL` for its mount
  table when that table was read from the handler's namespace,
  and the scan reported the namespace as an unreadable table rather than as a
  holder or as empty.
  That is what this round measured,
  and this guest would not make the arm's task under a process root of its own
  in the last two rounds.
  (Corrected 2026-11-23:
  the integration pass showed that a task chrooted
  after its mount was made answers a table that is short rather than
  unreadable,
  and that `EINVAL` is what the kernel answers for a task that is going.
  The fix round's chrooted-lowest arm shows the short table,
  and its
  leader-ended arm shows `EINVAL`'s case as an unreadable link,
  which is what
  `SPEC.md` states now.)
- The measurement line is below.

## What one scan costs

Two arms,
five rounds each,
averaged:

- 215 live tasks and 10 mount namespaces:
3 milliseconds.
- 414 live tasks and 10 mount namespaces:
4 milliseconds.

That is one millisecond for one hundred and ninety-nine more live tasks,
which is the resolution this guest's clock reports for an average of five
rounds,
and no run-to-run spread was recorded for either arm,
so the pair is context and not a slope,
and the round that read 2 against 4 ran while no other job of this session did.

The slope this record does own is measured on fixture `/proc` roots on the host,
which read no process of it:

- 100 tasks in 8 namespaces:
639 microseconds a scan.
- 10,000 tasks in 8 namespaces:
67,444 microseconds a scan.

That is about 6.7 microseconds for one more live task,
and the two arms' namespace count was the same,
so the part that grows with tasks is the per-task read of `/proc/<pid>/ns/mnt`
and its place in the grouping.
A scan costs,
in this order:
one directory listing of the `/proc` root,
one `ns/mnt` read per live task,
one sort of the tasks by pid inside each namespace,
one `mountinfo` read and parse per distinct namespace,
and one source comparison per mount row of those tables.
Nothing in it grows with the number of mounts the machine has outside the
namespaces it read,
and one more namespace costs one more table read whatever it holds.

## The bounds the scan ships, against these numbers

- 1024 mount namespaces one scan reads.
  The guest ran 10;
  a scan that stops at the bound is one incomplete read and never a clear
  result.
- 1024 findings one result lists.
  The guest's results held at most four.
- 8 mounts of the clone listed for one namespace.
  The guest's namespaces held two each:
  the slot's own mount,
  which the holder's namespace inherits,
  and the mount the arm made.

These numbers are context for the three bounds and not their derivation:
the bounds are adopted with the project's veto open,
and the owner may change them.

## What the rounds found, and what they do not show

Three rounds failed before the arms passed,
and each failure was a defect in the round's own work rather than in a bound:

- the reader held the result's namespace count to the number of lines below it
  and so refused the first guest result,
  which read ten namespaces and listed one mount;
- a holder's shell ran `mount` and then `sleep` as a child,
  so the arms that released a holder left the task that kept its namespace
  alive;
- a diverging-root task put the clone's mount above its own root.

Each was fixed and the arms then ran.

What these arms do not show:

- a confined Pi consumer,
  which this guest cannot run;
- an unrestricted caller that attaches the clone after a scan;
- a forced removal and what a holder sees in it;
- the start probe's own guest arms,
  whose unit controls pass but which were not re-run in this guest.

## The fix round of 2026-11-23, after the integration pass

The integration pass found the scan answering clear while four namespaces held
the clone,
and the project answered its three questions on 2026-11-23
(`doc/decision/cli-git-zfs-holder-scan-names-every-holder-it-can-read.md`).
This round took the set-aside out of the scan,
made it read every thread and every root,
gave every scan its own nonce,
and added an arm for each holder the pass found missed.
One whole round of the driver passed every marker
(`keep/holder-scan-arm-round-fix-4.log` in that round's folder),
and its lines
are:

    SCAN baseline own=1 holders=0 incomplete=0 guid=13245473912317187654
    SCAN HOLDER other-namespace=1 namespace=mnt:[4026532422] mount=/root/holder-scan-checkout/.claude/worktrees/one
    SCAN H1 intent=mount holders=2 at-the-slot=2 own=1 clear=false
    SCAN H1 intent=unmount holders=2 at-the-slot=2 own=1 clear=false
    SCAN nonces differ=true first=69b876eb00372619977e957583429afd second=854ba58b8c8f03a05733c04c056f4820 named=fedcba9876543210fedcba9876543210
    SCAN unreadable-link incomplete=235 holders=0 clear=false
    SCAN thread-namespace found=true namespace=mnt:[4026532424] thread=/proc/221153/task/221154
    SCAN leader-ended found=true namespace=mnt:[4026532423] leader-link=unreadable leader-state=Z
    SCAN chrooted-lowest found=true namespace=mnt:[4026532424] lowest-pid=221166 lowest-omits-mount=true
    SCAN confined-consumer=held holders=2 namespace=mnt:[4026532424]
    SCAN churn scans=20 holder-kept=20 task-ended-named=0
    MEASUREMENT holder-scan-ms=small=4 large=8 tasks-small=212 tasks-large=411
    SCAN detached-mount invisible=true holders=0 destroy-refused=true refused-while-pinned=true
    SCAN detached-mount after-release=destroyed
    SCAN pinned-namespace invisible=true holders=0 incomplete=0 note=the-scan-cannot-see-a-namespace-no-task-holds
    SCAN done

- The two `H1` lines are the same clone under the two intents with a mount
  namespace made after the attach:
  the inherited copy at the slot is a holder under both,
  and the number of holder namespaces is equal under the two,
  which is the finding the pass raised.
- `unreadable-link incomplete=235 holders=0` is the same scan through
  `setpriv --bounding-set=-sys_ptrace`:
  every link it may not read is named,
  and the scan is not clear.
  Its zero holders is the point:
  without `CAP_SYS_PTRACE` the scan could not name the two namespaces that
  held the clone,
  and before this round it said nothing about them.
- `leader-ended ... leader-link=unreadable leader-state=Z` is the pass's own
  case as this guest's kernel produces it:
  the leader's link answers nothing and the scan still names the namespace
  through the thread that lives.
- `chrooted-lowest` is the namespace whose lowest task chroots away from the
  clone:
  the lowest task's table omits the mount,
  the child's holds it,
  and the scan names the namespace by reading the task at its root.
- `detached-mount ... destroy-refused=true` is the limit:
  a mount detached with `umount --lazy` while a working directory pins it is in
  no table,
  so the scan names no holder,
  `zfs destroy -f` answers "dataset is busy",
  and the destroy succeeds once the task ends.
  This is stated as a limit in `SPEC.md` rather than fixed.
- The cost pair reads 4 milliseconds at 212 live tasks and 8 at 411,
  which is inside the spread the rounds of 2026-11-22 showed,
  and is context and not a slope.

The host's own fixture measurement of the same round,
which reads no process of the machine,
gave 100 tasks in 8 namespaces at 639 microseconds a scan and 10,000 tasks at
67,444 microseconds before this round's thread read was added.
The thread read adds one `ns/mnt` and one `root` read per thread rather than per
process,
so the per-thread cost is that same read and no new measurement of it
was taken.
