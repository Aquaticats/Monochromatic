# Private propagation after a replacement mount of the checkout

Non-final,
unapproved design proposal by an agent.
It is not approved by the project,
and nothing in it is
final.
It designs the open item "What re-applies private propagation after a replacement mount" of the accepted ZFS
worktree design,
and it makes no decision that design or the project's answers did not make beyond the writer's
choices,
which "Choices this design marks for the project's veto" lists.
The project's four answers to this design's questions are decided,
quoted under "The project's four answers,
and what this design does with each",
and the design asks nothing further.
It ran no ZFS command that changes anything,
made no mount,
unmount or propagation change anywhere,
started no service manager,
no guest,
no container and no virtual machine,
and it edited no other file.
Applying anything to this machine is outside a round,
and "What must be applied to the machine,
and where that is recorded" names each act,
its command and the record that carries it.

This revision applies the project's four answers and the diff check's edits.
The project answered this design's four questions on 2026-11-22 ([record][repair-record],
recorded 2026-11-22T19:23:30-05:00),
and a second record answered the lost-event design's questions and accepted the boot unit's `ExecReload=` line
([record][lost-record],
recorded 2026-11-22T19:20:28-05:00).
The questions therefore leave the question list and become decided sections that quote the record
("The project's four answers,
and what this design does with each"),
the trigger section keeps the one trigger the project chose,
with its kill criterion
("The trigger the route adds,
and what it must show"),
and the options the project did not choose are listed once and ranked nowhere.

The diff check of the previous round's revisions is answered where it applies:
the states the unit's request can meet are corrected and completed ("The propagation service and its drop-in",
F39),
with the case it cannot repair named and the next run of the handler as what repairs it,
a refusal is reported by the path it came by (F35,
"What a failure reports",
L10,
M4,
M7),
the three cross-references that still named the rename measurement and the last review label are corrected,
the filter option and its ranking are gone and one sentence says what that costs,
and the unit is installed only beside the boot unit's `ExecReload=` line,
with the check that keeps that order
("The propagation service and its drop-in",
L11).
The commits are listed under "Revisions".

An earlier revision answered the second gate review,
from another vendor than the writer's,
which read the document whole at `5e6a2ff9dc` and returned "fit after the listed fixes,
not as it stands":
the trigger unit is rewritten with `Type=oneshot` and `RemainAfterExit=yes` and asks the boot unit to act instead of
running the handler,
what systemd does after each way that unit can end is stated from source,
the job-driven examples name only units systemd can start,
the headline answer and A4 say what the design does,
the stale-instance comparison uses the number the kernel never reuses,
and every cross-reference names the thing it means.

An earlier revision still answered the first gate review,
also from another vendor than the writer's,
which read the document whole at `fecdc987b1`:
the mount-unit event path is replaced by a trigger and its kill criterion,
the mount numbers are split into the unique one and the table's one,
the OpenZFS statements about rollback,
rename and a second mount are corrected from source,
the printed command becomes `reload-or-restart`,
the no-file resolution is stated,
the threat model is applied check by check,
and the acceptance arm is named.
Every citation either review questioned was opened and corrected,
as were the ones they did not list.

What it read,
on this branch of `main` at `5dd8883a60`:
the decision record
[`doc/decision/cli-git-zfs-worktree-design.md`](../decision/cli-git-zfs-worktree-design.md) whole,
the proposal
[`doc/planning/cli-git-zfs-worktree-design-proposal.md`](cli-git-zfs-worktree-design-proposal.md) in the sections
"Boot and shutdown unit",
"One-time move and checkout boot protection",
"Checkout propagation policy" and "Open items
the decisions leave",
the implementation plan
[`doc/planning/cli-git-zfs-worktree-implementation-plan.md`](cli-git-zfs-worktree-implementation-plan.md) in the
sections named under "Links",
the step 8 choices
([`doc/decision/cli-git-zfs-step-8-sweep-choices.md`](../decision/cli-git-zfs-step-8-sweep-choices.md)),
the slot
identity store format ([`doc/decision/cli-git-zfs-slot-identity-store-format.md`](../decision/cli-git-zfs-slot-identity-store-format.md)),
the staging decision's threat model
([`doc/decision/cli-git-zfs-staging-under-cache.md`](../decision/cli-git-zfs-staging-under-cache.md)) and the
accepted staging design's opening,
`package/git-policy/cli/SPEC.md` for the handler,
the readiness record,
the sweep
and the credential rule,
and the handler's code under
`package/git-policy/cli/src/native/worktree_clone_mount_handler/` and
`package/git-policy/cli/src/native/worktree_clone_mount_protocol/`.

What it read outside the repository,
each at a named tag,
in a shallow clone of its own under the round's scratch
folder:
Linux `v7.3`,
OpenZFS `zfs-2.4.4` and systemd `v262`,
whose version this host prints as `systemd 262
(262.2-1-arch)`.
Every kernel,
ZFS and systemd fact here is read from those trees and cited by file and line at the
tag;
nothing in this document was measured on a kernel.

What it ran,
all of it read-only on this host:
`findmnt --output TARGET,SOURCE,FSTYPE,PROPAGATION` for `/`,
`/home`,
`/home/user`,
`/home/user/Monochromatic` and `/home/user/.cache`;
`systemctl --version`;
`ls /run/systemd/generator`;
`ls /etc/zfs/zfs-list.cache`;
`systemctl --no-ask-password list-units --type=mount --all`;
`git --version`;
and text searches over the repository.
The sibling documents were read with `git show` at the tips "Dependencies on sibling designs" gives,
without checking
a branch out and without editing one.

What it did not run:
any guest,
container or virtual machine;
any ZFS or pool command that changes anything;
any mount,
unmount or propagation change;
any request to the service manager that changes or signals anything;
and any test suite,
because it changed no code.
No guest ran,
so every claim
that rests on kernel,
ZFS or mount behaviour this repository has not already measured is written as unmeasured,
and
"Measurements owed" states the exact probe that would measure it.

The item is the plan's step 19 blocker ([plan][plan],
"Open items and the step that needs each"):
the entry named "What re-applies private propagation after a replacement mount" says
"step 19",
"blocking for its acceptance item",
and "writer design work".

## The open item and the answer it carries

The open item,
quoted whole ([proposal][proposal],
"Open items the decisions leave"):

> What re-applies private propagation after a replacement mount.
> The answer's words:
> "Boot unit sets it
> (Recommended)",
> whose option text has the unit repeat the setting after a replacement mount.
> The installed unit is
> `Type=oneshot` with `RemainAfterExit=yes`,
> so nothing fires on a later checkout mount,
> and what re-runs the setting,
> which event or unit does it,
> and how a failure is reported are undesigned
> ([Checkout propagation policy](#project-decisions)).

The decision it carries ([proposal][proposal],
"Project decisions",
lines 1282 to 1305):

> Checkout propagation policy.
> Decided:
> "Boot unit sets it (Recommended)".
> The installed boot unit applies private
> propagation to the verified checkout at boot,
> and the answer's option text says it repeats that after a replacement
> mount.
> Nothing re-applies it today:
> the unit is `Type=oneshot` with `RemainAfterExit=yes`,
> which does not fire on a
> later mount,
> and what re-runs the setting after a replacement mount is an open item

and,
in the same bullet,
the two sentences this design is built around:

> The alternative the project did not select,
> an
> administrator setting it once,
> does not persist across reboot or replacement of the checkout mount:
> propagation
> belongs to a mount instance (`mount_namespaces(7)`,
> "Shared subtrees").
> Recheck it regardless of who sets it.

and,
later in the same bullet,

> Do not silently change propagation.

The item asks four things and this design answers each:
what a replacement mount is,
which event exists for each way
it happens,
what re-runs the setting and what it verifies,
and how a failure is reported.
The answer this design
gives is that the setting is re-applied by **the same root-run handler,
as the first step of every entry point it has,
before that entry point enumerates,
attaches or unmounts anything**,
and that the boot unit remains the setter it was
decided to be,
being one of those entry points.
No event starts the setting directly:
every trigger starts an entry point,
and the step is what runs inside it,
which is why the same step serves a boot,
a ZED event and a root command.
Which events reach an entry point,
and which of them the route installs,
is the project's answer,
quoted with the other three under "The project's four answers,
and what this design does with each",
and the trigger it installs is stated once under "The trigger the route adds,
and what it must show".

## The project's four answers, and what this design does with each

The project answered this design's four questions on 2026-11-22,
and the answers are decided ([record][repair-record],
"Status",
recorded 2026-11-22T19:23:30-05:00).
A second record answers the lost-event design's questions and accepts the boot unit's `ExecReload=` line this design
shares with it ([record][lost-record],
recorded 2026-11-22T19:20:28-05:00).
This design follows all five answers,
and this section quotes each and names its consequence.

- Who may set the propagation when no person is present.
  The record:
  "The project chose "A:
  every root run (Recommended)":
  whenever the root handler runs,
  for any event,
  it first checks the checkout and sets it private when it finds it shared,
  and records that it did"
  ([record][repair-record],
  "Who may set the propagation when no person is present").
  The design does exactly this:
  the step is the first act of both entry points,
  after the compatibility gate ("Where it runs,
  as whom,
  and in what order"),
  and every trigger starts one of those entry points
  ("The trigger the route adds,
  and what it must show").
  The record's costs stand as it states them,
  and this host's reading of one of them is F40:
  nothing is repaired while `zfs-zed.service` is stopped,
  and it is stopped here.
- Which triggers the route adds.
  The record:
  "The project chose one:
  "Unit on the checkout's mount (Recommended)",
  a drop-in on the checkout's mount unit and one one-shot unit,
  which asks the boot unit to reconcile when the mount comes back,
  with the same request the readiness hook uses.
  It is taken provisionally:
  it is dropped if the guest measurement shows that it misses a replacement typed by hand or that it loops"
  ([record][repair-record],
  "Which triggers the route adds").
  "The trigger the route adds,
  and what it must show" states it with its reach and its kill criterion,
  and "The propagation service and its drop-in" gives both files.
- What the repair does with a state that is neither private nor shared.
  The record:
  "The repair sets the mount private only when it is shared;
  a mount that is slave,
  unbindable or a combination is left as it is,
  the route goes on,
  and the state is reported"
  ([record][repair-record],
  "What the repair does with a state that is neither private nor shared").
  "The call that sets it" acts on `MS_SHARED` alone and records the set it found.
- How far the setting reaches.
  The record:
  "The repair changes the checkout's own mount and no mount beneath it,
  and reports a shared mount it finds beneath"
  ([record][repair-record],
  "How far the setting reaches").
  "The call that sets it" sends no `AT_RECURSIVE`,
  and "What it verifies after" checks the one instance.
- The boot unit's `ExecReload=` line.
  The lost-event record chose
  "a drop-in on the provider whose `ExecStartPost=` is prefixed with a dash and runs
  `systemctl --no-block --no-ask-password reload-or-restart reconcile-git-worktree-clone-mounts.service`,
  with the boot unit gaining an `ExecReload=` that runs the handler's `boot-reconcile`"
  ([record][lost-record],
  "Which trigger runs after the checkout becomes ready").
  The line is therefore accepted,
  and this design no longer offers it as a choice:
  "What becomes of `Type=oneshot` with `RemainAfterExit=yes`" states it as decided,
  and L11 keeps one short note for a machine that lacks it.

What the project did not choose,
stated once here and ranked nowhere in this document
([record][repair-record],
"What was not chosen"):
only the boot unit as setter;
the wider event filter,
a watcher on the kernel's mount events,
and no trigger beyond the root command;
refusing every state that is not private;
and setting the propagation recursively.
The design describes none of them further:
each is closed by the record,
and where one of them touches a sentence the project kept,
that sentence cites the record instead of re-arguing the choice.

One consequence of the trigger answer belongs here rather than in a question:
the event filter does not gain the checkout's dataset for this purpose
([record][repair-record],
"Which triggers the route adds",
and the lost-event document's V2).
Events of the checkout's own dataset therefore do not reach the step,
which is why the mount-unit trigger exists and why the handler's own live check stands behind it (F18).

The wrapper's own half was decided before this design and is not reopened:
it refuses before writing a claim,
intent or nudge on a shared checkout
([proposal][proposal],
"Add and Git registration",
step 1,
lines 365 to 372),
and "What the wrapper and the caller see between the replacement and the re-run" states that refusal.

## Decisions and records that bound this proposal

- The propagation policy itself,
  quoted under "The open item and the answer it carries":
  the boot unit sets it,
  its setting does not
  survive a replacement mount,
  an administrator's one-off setting does not either,
  it is rechecked regardless of who
  sets it,
  and it is never changed silently ([proposal][proposal],
  "Checkout propagation policy",
  lines 1282 to 1305).
- The unit as designed and as built:
  `reconcile-git-worktree-clone-mounts.service` with `Type=oneshot`,
  `RemainAfterExit=yes`,
  `Requires=` and `After=` the checkout provider,
  `ExecStart` running
  `mount-git-worktree-clone-datasets boot-reconcile` and `ExecStopPost` running `shutdown-unmount`
  ([proposal][proposal],
  "Boot and shutdown unit",
  lines 499 to 540).
  Step 8 built the two modes' checks and the sweep
  (`package/git-policy/cli/SPEC.md`,
  "The sweep,
  and the boot it reconciles",
  lines 6272 to 6300).
- The checkout's boot mount protection,
  decided as "Immutable entry (Recommended)":
  the underlay is verified before
  automated attach,
  including after reimport or rollback ([proposal][proposal],
  "Checkout boot mount protection",
  lines 1275 to 1281;
  "One-time move and checkout boot protection",
  lines 290 to 333).
- The triggers the project controls ([proposal][proposal],
  lines 1393 to 1400):
  "the boot unit's start reconcile,
  the
  verified checkout provider's post-key-load readiness hook,
  and the clone-local sweep nudge the wrapper writes on a
  caller's behalf",
  with ZED's delivery described in the open items as "installed but not controlled by the design"
  ([proposal][proposal],
  "Open items the decisions leave",
  line 1784).
- No configuration,
  wherever it can be avoided:
  the project "first answered that everything must work by default with
  no configuration",
  and for the case with no file at all it chose "1:
  no file needed (Recommended)"
  ([step 8 choices][sweep-choices],
  "With no configuration file").
  This design adds no key to that file and no
  setting the project has to keep.
- The threat model ([staging decision][staging-decision],
  "The handle record's place",
  lines 221 to 237),
  applied check by check under "What each check owes the threat model":
  "We have
  to assume my machine isn't compromised.
  sudo is passwordless here anyway.",
  read there as "the account is trusted,
  and root and the account are not a security boundary here."
  Every check this design adds is measured against that
  sentence where it is stated.
- Concurrency ([`AGENTS.md`](../../AGENTS.md),
  rule `NLK`):
  "Concurrency is lock-free first (atomic rename,
  `O_EXCL`,
  one owner per item);
  an unavoidable lock covers the smallest unit,
  never a whole repository,
  with its reason
  written beside it."
- Shipped limits (rule `LQM`):
  "Shipped resource limits (unit caps,
  timeouts,
  size bounds) cite a workload
  measurement or owner decision beside the value".
  This design introduces no timeout and no cap;
  the one size it
  states,
  the record's file bound,
  is derived from kernel limits and names them.
- The record precedent this design weighs rather than follows
  ([slot identity store][slot-store],
  whole):
  one file per item,
  `O_EXCL` staging then one `link`,
  no listing on the
  waiting path,
  `doctor` reporting what is left over.
  This design takes the publication habit and the `doctor` duty,
  and takes the single-file replace that the store's parent record uses,
  because there is one checkout and one
  current answer.
- The account's limits on asking for authority ([`SPEC.md`][spec],
  lines 7345 to 7352):
  "No command the wrapper runs
  or forwards asks a person for a credential",
  with "Avoid ever asking for someone to type a username or password."
  This design makes the wrapper refuse and print root's command;
  it never prompts.
- Worktrees live under `.claude/worktrees/` in the main checkout (rule `WTU`),
  and every path here comes from the
  injected home (rule `HOM`),
  never from a hard-coded user name.

## Facts

Each fact names its source and the lines read.
F1 is read on this host;
the rest are read in the sources at the tags
named,
and none is a measurement of behaviour.

- F1.
  On this host `/` is `zpcachyos/ROOT/cos/root`,
  `/home` and `/home/user` are `zpcachyos/ROOT/cos/home`,
  and
  `~/.cache` is `zpcachyos/nosnap/user-cache`;
  all three carry `PROPAGATION shared`.
  The checkout path
  `/home/user/Monochromatic` is on no mount of its own:
  `findmnt --target` answers the `/home` row for it.
  Read
  2026-11-22 with `findmnt --output TARGET,SOURCE,FSTYPE,PROPAGATION --target <path>`.
  The measurement record's host
  inventory agrees for the same three mounts
  ([measurements][measurements],
  "Fixture and evidence").
- F2.
  systemd makes `/` and every mount under it shared when it is started by the kernel:
  `mount(NULL, "/", NULL, MS_REC|MS_SHARED, NULL)`,
  skipped in a container and skipped when a launcher asks for it
  (systemd `v262`,
  `src/shared/mount-setup.c:513` to `:521`).
  The comment beside it says why:
  "The kernel defaults to
  'private',
  but we think it makes more sense to have a default of 'shared'".
  So a checkout whose parent is `/home`
  has a shared immediate parent on an ordinary boot (F1 measured the same shape).
- F3.
  A mount created under a shared parent is made shared itself and its mount event is propagated to the parent's
  peers:
  `attach_recursive_mnt` calls `invent_group_ids(source_mnt, true)` and `propagate_mnt` when
  `IS_MNT_SHARED(dest_mnt)`,
  then runs `set_mnt_shared(p)` over the new subtree (Linux `v7.3`,
  `fs/namespace.c:2687` to `:2703`),
  and `invent_group_ids` allocates a fresh group id rather than reusing the parent's
  (`fs/namespace.c:2524` to `:2539`).
  The new instance therefore joins a fresh peer group of its own,
  and it comes
  back shared after every replacement,
  however often it was made private before.
- F4.
  The same code leaves the new mount private when the destination parent is not shared:
  `invent_group_ids`,
  `propagate_mnt` and `set_mnt_shared` all sit inside the `IS_MNT_SHARED(dest_mnt)` branch (Linux `v7.3`,
  `fs/namespace.c:2687` to `:2703`).
  That is the mechanism the decided route relies on for clones:
  a clone attached
  under a private checkout stays private,
  and a checkout replaced by a shared instance hands its later children a
  shared mount.
- F5.
  Propagation is changed by `mount_setattr`,
  not only by `mount(2)`:
  `struct mount_attr` carries a `propagation`
  field (Linux `v7.3`,
  `include/uapi/linux/mount.h:133` to `:138`),
  `mount_setattr_commit` calls
  `change_mnt_propagation(m, kattr->propagation)` for the mount and,
  with `MOUNT_KATTR_RECURSE`,
  for each mount under
  it (`fs/namespace.c:5073` to `:5091`).
- F6.
  `mount_setattr` accepts `AT_EMPTY_PATH`,
  so it can act on a descriptor's own mount:
  it admits `AT_EMPTY_PATH`,
  `AT_RECURSIVE`,
  `AT_SYMLINK_NOFOLLOW` and `AT_NO_AUTOMOUNT` and resolves with the descriptor's path
  (Linux `v7.3`,
  `fs/namespace.c:5317` to `:5356`),
  and `do_mount_setattr` then requires the path to be a mount root
  (`path_mounted`,
  `fs/namespace.c:5100` to `:5101`) and the mount to be in the caller's namespace
  (`check_mnt`,
  `fs/namespace.c:5119` to `:5120`).
  A mount detached by a lazy unmount is in no namespace,
  so the call
  answers `EINVAL` there rather than changing a mount nobody can see.
- F7.
  Two gates guard a propagation change,
  and neither is the account's:
  `mount(2)` with a propagation flag goes through `do_change_type` and `may_change_propagation`,
  which answers `EINVAL` for a mount in no namespace and `EPERM` without `CAP_SYS_ADMIN` in that namespace's user
  namespace (Linux `v7.3`,
  `fs/namespace.c:2943` to `:2954`,
  called at `:2993`),
  while `mount_setattr` asks `may_mount()` for the same authority and `check_mnt` for membership of the caller's
  namespace (`fs/namespace.c:5298`,
  `:5119` to `:5122`).
  Root in the host namespace satisfies both;
  the account does not,
  which is why the wrapper cannot do this work.
- F8.
  `statmount` reports a mount's propagation and identity by descriptor:
  `struct statmount` carries `mnt_id`,
  `mnt_parent_id`,
  `mnt_propagation`,
  `mnt_peer_group`,
  `mnt_master`,
  `mnt_ns_id` and the mount point
  (Linux `v7.3`,
  `include/uapi/linux/mount.h:157` to `:182`),
  `STATMOUNT_BY_FD` asks for "mountinfo for given fd"
  (`:243` to `:245`),
  and `do_statmount` reads `mnt_file->f_path.mnt`,
  drops `STATMOUNT_MNT_POINT` and
  `STATMOUNT_MNT_NS_ID` when the mount has no namespace,
  (`fs/namespace.c:5908` to `:5928`).
  The `ENOENT` for a mount id it cannot find belongs to the branch that looks a number up in a namespace
  (`:5930`,
  `:5934`),
  not to the descriptor read this step uses.
  It reports two different mount numbers,
  and they are not interchangeable:
  `mnt_id` is filled from the mount's `mnt_id_unique`,
  "mount ID unique until reboot",
  and `mnt_id_old` from `mnt_id`,
  which `/proc/self/mountinfo` prints and the kernel may reuse
  (Linux `v7.3`,
  `fs/namespace.c:5494` to `:5501`;
  `fs/mount.h:102` to `:103`;
  `fs/proc_namespace.c:143`).
  The repository's parser reads the mountinfo number,
  the first field,
  and its checkout record keeps no number at all
  (`src/native/subvolume/fs_mount_options.rs:53` to `:54`;
  `src/native/worktree_clone_mount_handler/sweep_checkout_mount.rs:62` to `:73`).
- F9.
  An ordinary unmount refuses while the mount has children or holders:
  `do_umount` answers `EBUSY` unless
  `propagate_mount_busy(mnt, 2)` is clear (Linux `v7.3`,
  `fs/namespace.c:2004` to `:2012`;
  `fs/pnode.c:501` to `:531`,
  whose `:512` is the child-mount test).
  A checkout with any child mount,
  the clone mounts and the vm-builder output dataset among them,
  is such a mount (F35).
- F10.
  A lazy unmount detaches the whole subtree:
  `do_umount` with `MNT_DETACH`
  calls `umount_tree(mnt, UMOUNT_PROPAGATE)` without the busy check (`fs/namespace.c:2004` to `:2008`),
  `umount_tree` walks every mount under the one named,
  moves it out of its namespace and clears the namespace pointer
  (`:1831` to `:1873`),
  and `bulk_make_private` clears each detaching mount's master and moves any slaves to what was its master
  (`fs/pnode.c:162` to `:175`).
  The shared flag of the detached mounts is not what those lines show;
  the subtree is simply out of every namespace and out of any peer group.
- F11.
  A second mount of a dataset that is already mounted succeeds as a second mount of the same superblock:
  `zpl_get_tree` looks the superblock up by the dataset and mounts a fresh one only when `sb->s_root == NULL`,
  so an existing superblock is reused and the new mount shares it
  (OpenZFS `zfs-2.4.4`,
  `module/os/linux/zfs/zpl_super.c:849` to `:951`,
  with the comment "Matching would imply a multimount of a dataset" at `:345` to `:357`).
  Its `EBUSY` answers are a differing objset (`:881` to `:887`) and a read-only flag mismatch (`:937` to `:943`),
  not a dataset held by another mount.
  The long hold that refuses a fresh superblock,
  through `zfsvfs_create` and `dsl_dataset_tryown`,
  belongs to the `sb->s_root == NULL` branch (`module/os/linux/zfs/zfs_vfsops.c:651` to `:669`;
  `module/zfs/dsl_dataset.c:896` to `:918`,
  `:1049` to `:1062`).
  So a clone whose old mount was lazily detached can plausibly be attached a second time while the detached mount
  stays invisible,
  which is unmeasured (M5).
- F12.
  systemd mirrors mounts made outside it:
  "Mount points created at runtime (independently of unit files or
  `/etc/fstab`) will be monitored by systemd and appear like any other mount unit" (systemd `v262`,
  `man/systemd.mount.xml:74` to `:79`).
  A mount that disappears puts its unit into the dead state,
  and a mount that
  appears puts it into the mounted state with a fresh invocation id,
  both without a job
  (`src/core/mount.c:2242` to `:2271`,
  through `mount_set_state` at `:707`).
- F13.
  A stop or restart job on a unit reaches its dependents that carry `UNIT_ATOM_PROPAGATE_STOP`:
  the job adds a stop,
  or a try-restart for a restart,
  to every such dependent (`src/core/transaction.c:1199` to `:1215`),
  and `RequiredBy` carries that atom (`src/core/unit-dependency-atom.c:43` to `:46`),
  as does `ConsistsOf`,
  which is what a `PartOf=` on the other unit shows as (`:85` to `:86`).
  So a restart of a unit with a `Requires=` dependent try-restarts that dependent,
  and a restart of a mount unit propagates only through the jobs systemd is running.
  Separately,
  the retroactive start that runs on an unexpected activation covers the activating unit's own `Requires=`,
  `BindsTo=` and `Upholds=` targets,
  and its `Wants=` targets (`src/core/unit.c:2313` to `:2331`,
  called at `:2805` to `:2815`),
  skipping each one that has an `After=` from the activating unit;
  it does not start the activating unit's dependents.
- F14.
  A path unit cannot see a mount appear at a path it watches,
  and it is rate-limited:
  path units watch with
  inotify (`man/systemd.path.xml:49` to `:52`),
  `PathExists=` fires on existence and `PathChanged=` on a change to
  the watched file (`:115` to `:130`),
  a path that already satisfies `PathExists=` fires at once and "Something
  similar does not apply to `PathChanged=`" (`:148` to `:156`),
  and a trigger limit of 2s and 200 activations,
  after
  which the unit is failed and stops watching,
  applies (`:193` to `:206`).
  A mountpoint is a directory whether or not
  a mount covers it.
- F15.
  The checkout dataset gets a mount unit wherever the ZFS generator runs,
  and the generator writes one
  `<escaped mountpoint>.mount` per dataset with `canmount=on` and a path mountpoint,
  with `Before=zfs-mount.service`,
  `After=zfs-import.target` and the key-load unit,
  and a `[Mount]` section of `Where=`,
  `What=`,
  `Type=zfs`
  (OpenZFS `zfs-2.4.4`,
  `etc/systemd/system-generators/zfs-mount-generator.c:629` to `:673`),
  reading its dataset list from the cache file.
  On this host `/etc/zfs/zfs-list.cache` does not exist and
  `/run/systemd/generator` holds one mount unit,
  `boot.mount`,
  so the generator is idle here;
  systemd nevertheless tracks the host's dataset mounts as mount units it learned from the mount table
  (F33),
  and the checkout will have such a unit once it is a mount.
  Read 2026-11-22.
- F16.
  `zfs-mount.service` mounts the datasets itself:
  `Type=oneshot`,
  `RemainAfterExit=yes`,
  `Conflicts=umount.target`,
  `Before=umount.target`,
  `ExecStart=<sbindir>/zfs mount -a` (OpenZFS `zfs-2.4.4`,
  `etc/systemd/system/zfs-mount.service.in:1` to `:25`).
  A dataset it mounts is mounted outside any mount unit,
  which
  is F12's extrinsic case.
- F17.
  ZFS has no event for a mount or an unmount:
  the sysevent classes are pool,
  vdev,
  scrub,
  trim,
  errorscrub,
  config,
  history and bootfs ones (OpenZFS `zfs-2.4.4`,
  `include/sys/sysevent/eventdefs.h:96` to `:131`),
  and the
  mount path posts no ereport (`module/os/linux/zfs/zfs_vfsops.c`,
  searched for `zfs_ereport_post`).
  The proposal
  already relies on the same list for its boot and shutdown argument ([proposal][proposal],
  "Boot and shutdown unit",
  lines 489 to 497),
  which cites the same block at `:97` to `:137`;
  the two ranges name the same definitions with a line of slack.
- F18.
  The built sweep already refuses a checkout whose mount is shared,
  and reads it live on every run:
  the checkout
  is `ready` only when "the checkout's own mount carries no `shared:` tag",
  `not-ready` "when the folder is missing,
  the path is not canonical or the mount is shared",
  and "`unknown` admits nothing"
  ([`SPEC.md`][spec],
  lines 6337 to 6346).
  The code reads the tag through the repository's own mountinfo parser
  (`src/native/worktree_clone_mount_handler/sweep_checkout_mount.rs:1` to `:60`,
  `:160` to `:175`;
  `sweep_preconditions.rs`,
  `checkout_state`).
  Nothing in that path sets anything.
- F19.
  The handler already publishes a root-owned record atomically and proves it before believing it:
  a root-owned
  `0755` directory under `/run`,
  a `0644` file staged under a name carrying the writer's pid and attempt and moved
  into place by one `rename`,
  a write refused when the writer could not read the record back
  (`src/native/worktree_clone_mount_protocol/clone_mount_readiness_file.rs:1` to `:60`;
  [`SPEC.md`][spec],
  "The readiness record",
  lines 6184 to 6218).
  The same file's reader proves the directory,
  the
  file's owner and its mode before use.
- F20.
  `move_mount` refuses a mount whose current parent is shared:
  "`if (IS_MNT_SHARED(old->mnt_parent)) return
  -EINVAL;`" (Linux `v7.3`,
  `fs/namespace.c:3789` to `:3790`).
  That refusal was measured with a shared checkout and
  disappeared once only the checkout was private ([measurements][measurements],
  item 3),
  in a guest on kernel `6.8.0-147-generic`,
  which is not this host's kernel (F1;
  [measurements][measurements],
  "3.
  Shared propagation and a confined reader").
- F21.
  The wrapper already reads mount facts itself,
  without privilege,
  through the repository's mountinfo parser:
  `fs_mount_options::read_mount_facts` is used by the domain checks,
  the removal,
  the lock probes and `doctor`
  (`src/native/worktree/worktree_clone_domain.rs:104`,
  `src/native/worktree/worktree_remove.rs:198`,
  `src/native/health/doctor_filesystem.rs:230`,
  `:235`).
  What the caller side did not have before this design is the descriptor read:
  `doctor` now opens the checkout path and calls `statmount` with `STATMOUNT_BY_FD` for the unique number (F37),
  which needs no capability in this namespace,
  and falls back to "same number,
  not a proven instance" where the kernel refuses it.
  A caller-side check of the checkout's propagation needs no
  new reader,
  no device and no root.
- F22.
  `systemctl` raises an authorization dialog rather than failing quietly,
  unless its caller forbids it:
  `systemctl` sets the bus message's interactive-authorization flag from its own `--ask-password` setting
  (systemd `v262`,
  `src/systemctl/systemctl-util.c:68`),
  and the manager asks polkit for the action `org.freedesktop.systemd1.manage-units`,
  adding `POLKIT_ALLOW_INTERACTIVE` exactly when the message carries that flag
  (`src/core/dbus-util.c:161` to `:199`;
  `src/shared/bus-polkit.c:652` to `:660`).
  So a request to change or signal a system unit is answered by whatever authorization agent the caller's session
  has,
  and the command waits while that agent asks;
  `--no-ask-password` is what turns the wait into a plain refusal.
- F23.
  `Upholds=` keeps a unit started while the upholding unit is up,
  with no delay and the usual rate limit:
  "as long as this unit is up,
  all units listed in `Upholds=` are started whenever found to be inactive or failed,
  and no job is queued for them" (systemd `v262`,
  `man/systemd.unit.xml:781` to `:789`),
  and the manager starts such a unit from its upheld queue
  (`src/core/manager.c:1565` to `:1601`),
  which throttles rather than stops:
  the queue allows 16 starts in 10 seconds per unit and re-queues the unit at the end of the window when that is
  passed (`src/core/unit.c:124` to `:129`;
  `src/core/manager.c:1485` to `:1497`,
  `:1499` to `:1523`),
  and the service's own start limit,
  5 in 10 seconds,
  refuses the sixth request with "Start request repeated too quickly" and fails the unit
  (`src/core/unit.c:1873` to `:1897`;
  `src/basic/constants.h:20` to `:21`;
  `src/core/service.c:6392` to `:6395`),
  after which a new fixed window opens once 10 seconds have passed since the first request of the refused one
  (`src/basic/ratelimit.c:9` to `:29`).
  Every transition to active submits the unit's `Upholds=` targets to that queue
  (`src/core/unit.c:2293` to `:2301`,
  called at `:2882`),
  and every transition to inactive submits the unit itself,
  so a failed upheld unit is started again while its upholder stands,
  throttled by those limits (`src/core/unit.c:2866`).
- F24.
  A drop-in is read for a mount unit systemd learned from the mount table:
  `mount_load` calls `unit_load_fragment_and_dropin(u, !from_kernel)`,
  so the fragment is optional but the drop-in directory is read either way (systemd `v262`,
  `src/core/mount.c:666` to `:668`).
- F25.
  A reload is refused on an inactive unit and a start does not imply a reload:
  "Unit cannot be reloaded because it is inactive" (systemd `v262`,
  `src/core/unit.c:2155` to `:2156`),
  while `reload-or-restart` starts a unit that is not running (systemd `v262`,
  `src/core/job.c:505` to `:510`;
  `man/systemctl.xml:530` to `:537`).
- F26.
  A plain `zfs rollback` of a mounted file system does not replace its mount:
  `zfs_ioc_rollback` calls `zfs_suspend_fs`,
  then `dsl_dataset_rollback`,
  then `zfs_resume_fs` on the mounted dataset (OpenZFS `zfs-2.4.4`,
  `module/zfs/zfs_ioctl.c:4800` to `:4841`).
  The instance and its propagation are the ones it had before.
- F27.
  A plain `zfs rename` of a file system with a non-legacy mountpoint unmounts and remounts it:
  the rename path keeps the mount up only for `legacy` and `none` mountpoints or with `-u`
  (OpenZFS `zfs-2.4.4`,
  `lib/libzfs/libzfs_dataset.c:4571` to `:4582`),
  gathers the dependents as a mountpoint change (`lib/libzfs/libzfs_changelist.c:713` to `:715`),
  unmounts them (`changelist_prefix`,
  `libzfs_changelist.c:97`),
  and remounts what was mounted (`changelist_postfix`,
  `:179`,
  with the remount at `:260` to `:270`).
  So a rename is an unmount and a mount by the `zfs` program,
  outside any systemd job.
  Whether it also unmounts the legacy-mounted clones,
  which are dependents of the checkout's snapshots,
  is unmeasured (M9).
- F28.
  `zfs mount -a` does not mount a dataset that is already mounted,
  and an explicit `zfs mount` of one refuses:
  "filesystem already mounted" (OpenZFS `zfs-2.4.4`,
  `cmd/zfs/zfs_main.c:7380` to `:7391`).
  The accepted staging design recorded the same reading
  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
  lines 1006 to 1008).
- F29.
  A child mount keeps its parent busy:
  `propagate_mount_busy` returns busy when `mnt_mounts` is not empty (Linux `v7.3`,
  `fs/pnode.c:501` to `:531`,
  the child test at `:512`).
  This host mounts the vm-builder output dataset under the checkout,
  with its own unit (F33).
- F30.
  `mnt_propagation` is a bit set,
  not one word:
  `mnt_to_propagation_flags` sets `MS_SHARED`,
  `MS_SLAVE` and `MS_UNBINDABLE` independently and returns `MS_PRIVATE` only when none is set
  (Linux `v7.3`,
  `fs/namespace.c:5455` to `:5469`).
  A mount can therefore be shared and slave at once,
  and a measured arm prints exactly that for one mount ("shared,slave",
  [measurements][measurements],
  line 141).
- F31.
  ZFS posts a `history_event` for every logged command,
  and the installed zedlet runs the handler's default invocation for each event it receives:
  `spa_history_log_notify` posts `ESC_ZFS_HISTORY_EVENT` after writing the record (OpenZFS `zfs-2.4.4`,
  `module/zfs/spa_history.c:200`,
  the post at `:245`),
  and the default invocation takes no event variable today
  ([`SPEC.md`][spec],
  lines 6277 to 6285).
  The sibling design proposes a filter that runs a pass only for the parent subtree
  ([`cli-git-zfs-lost-last-event-triggers-design.md`](cli-git-zfs-lost-last-event-triggers-design.md),
  "The event filter"),
  which the checkout dataset,
  a sibling of the container,
  is outside.
- F32.
  The kernel notifies mount attach and detach to a mount-namespace mark,
  so a watcher can subscribe without polling:
  `fsnotify_mnt_attach` and `fsnotify_mnt_detach` send `FS_MNT_ATTACH` and `FS_MNT_DETACH` to the namespace's marks
  (Linux `v7.3`,
  `include/linux/fsnotify.h:502` to `:515`),
  and fanotify exposes them as `FAN_MNT_ATTACH`,
  `FAN_MNT_DETACH` and `FAN_MARK_MNTNS`
  (`include/uapi/linux/fanotify.h:31` to `:32`,
  `:100`).
- F33.
  This host's systemd holds mount units for mounts no generator wrote,
  read 2026-11-22 with `systemctl --no-ask-password list-units --type=mount --all`:
  `home.mount`,
  `home-user-.cache.mount`,
  `home-user-.cargo-build.mount`,
  `home-user-.npm.mount` and
  `home-user-Monochromatic-package-vm\x2dbuilder-cli-output.mount`.
  `/run/systemd/generator` holds only `boot.mount`.
- F34.
  With no configuration file the project decided how the checkout is found:
  "the root handler finds the checkout as the one mounted ZFS dataset whose last name component is
  `monochromatic-git-checkout`,
  takes the container beside it",
  "Only a dataset that is mounted counts",
  and "When the name matches none or several,
  the handler refuses and lists them"
  ([sweep-choices][sweep-choices],
  "With no configuration file",
  lines 45 to 55),
  with the setup still writing the file and "a file that is present wins".
  The built handler requires the key today and the fallback is being built on a branch of its own;
  the record decides.
  The design name for the checkout dataset is one constant
  (`src/native/worktree_clone_mount_handler/sweep_checkout_mount.rs:36` to `:37`),
  and the container's another (`:39` to `:40`).
- F35.
  A refused mode exits `2`,
  the dispatch's status for every refusal
  (`src/native/worktree_clone_mount_handler/handler_mode_dispatch.rs:12` to `:14`,
  `:105` to `:114`),
  and what that status does to the boot unit depends on which of its two commands ran it:
  a failed `ExecStart` fails the unit,
  because "a boot unit's `ExecStart` that could not
  reconcile must fail",
  and a failed start runs the unit's `ExecStopPost` commands,
  which for this unit are `shutdown-unmount` ([proposal][proposal],
  "Boot and shutdown unit",
  lines 523 to 527);
  a failed `ExecReload` does neither,
  because `service_reload_finish` records the failure in the reload result and returns the unit to running,
  which for a `RemainAfterExit=yes` unit settles at active,
  with no `ExecStopPost`
  (systemd `v262`,
  `src/core/service.c:3685` to `:3697`,
  the control process at `:5083` to `:5086`,
  and the exit path at `:2798` to `:2809`,
  `:2863` to `:2864`),
  while the reload job itself is failed when the unit is active and its reload did not succeed
  (`src/core/unit.c:2666` to `:2670`).
  So after the first boot,
  where every repair reaches the unit as a reload,
  a refusal leaves it active with its reload result set,
  and only a start-path refusal fails it and runs the stop pass.
- F36.
  A mount unit that systemd learned only from the mount table cannot be started,
  and a start job on it fails:
  `mount_start` enters the mounting state (`src/core/mount.c:1336` to `:1350`),
  which reads its parameters with `get_mount_parameters_fragment`,
  a function that answers `NULL` unless the unit came from a fragment (`:84` to `:91`),
  and fails with "No mount parameters to operate on" when it does (`:1183` to `:1187`).
  Such a unit can be stopped,
  which unmounts it,
  and not started again;
  this host's dataset mount units are all of that kind (F33),
  and a unit the ZFS generator writes is not (`src/core/mount.c:666` to `:668`;
  `etc/systemd/system-generators/zfs-mount-generator.c:616` to `:676`,
  OpenZFS `zfs-2.4.4`).
- F37.
  The reused mount number is the lowest free one and returns to the pool,
  while the unique number never does,
  and a descriptor read needs no capability in this namespace:
  `mnt_alloc_id` takes the lowest free number from an xarray and sets `mnt_id_unique` from a counter that only
  rises (Linux `v7.3`,
  `fs/namespace.c:227` to `:237`),
  `cleanup_mnt` returns the reused one with `mnt_free_id` after the last reference goes (`:1382` to `:1399`,
  the call at `:1397`),
  and `do_statmount` runs the reachability and `CAP_SYS_ADMIN` test only when it was given a mount number rather
  than a descriptor (`if (!mnt_file)`,
  `:5942` to `:5952`),
  leaving `security_sb_statfs` (`:5955`),
  with the syscall's own entry check asking `CAP_SYS_ADMIN` only for another mount namespace (`:6173` to `:6175`).
- F38.
  systemd compares no mount number when it decides that a mount changed:
  it marks an entry "just mounted" when it has not seen the mount point in its table before or is waiting for it,
  and "just changed" when the source,
  options or type differ (systemd `v262`,
  `src/core/mount.c:1827` to `:1841`),
  then follows the state change those flags call for (`:2258` to `:2283`).
  A mount that leaves and returns between two reads of the table with the same source,
  options and type therefore compares equal,
  and `mount.c` holds no mount number to compare (`rg 'mnt_id|mount_id' src/core/mount.c`
  finds no match).
  systemd also rate-limits its own reading of the table,
  5 events in 1 second by default (`src/core/mount.c:2082` to `:2083`,
  set at `:2138`),
  coalescing changes inside a window into one rescan and holding mount start jobs while it is limited
  (`src/core/unit.c:1948` to `:1955`),
  so a burst such as `zfs mount -a` can delay and merge the transitions a mount-unit trigger depends on.
- F39.
  `reload-or-restart` is one job request whose type systemd decides on arrival,
  and what it does depends on the state the boot unit is in:
  it becomes a reload when the unit is active or reloading and a start otherwise
  (systemd `v262`,
  `src/core/job.c:505` to `:510`),
  which needs an `ExecReload=` to be applicable (`service_can_reload`,
  `src/core/service.c:3778` to `:3786`).
  A start job merges into one already installed on the unit (`job_install`,
  `src/core/job.c:240` to `:279`),
  and a start is refused `-EALREADY` when the unit is already active or reloading and `-EAGAIN` when it is
  deactivating or under maintenance (`src/core/unit.c:1957` to `:1965`),
  while a failed unit is started normally.
  An activating unit is not started a second time:
  `service_test_startable` answers no for a unit that is already starting (`src/core/service.c:6388` to
  `:6389`),
  `unit_start` returns without running it again (`src/core/unit.c:2012` to `:2016`),
  and a start job installed beside a running one merges and re-runs a no-op (`src/core/job.c:258` to `:277`).
  A reload that arrives while the unit is reloading is refused `-EAGAIN` (`src/core/unit.c:2148` to `:2153`),
  and the job waits rather than failing (`src/core/job.c:967` to `:969`),
  as does a start whose run-time check answers `-EAGAIN`.
  A start that conflicts with an installed stop job cancels it under the default job mode `replace`
  (`src/systemctl/systemctl.h:102` to `:103`;
  `src/core/transaction.c:618` to `:632`),
  and against an irreversible stop the enqueue is refused as destructive,
  which is the one refusal `systemctl` answers with a non-zero status.
- F40.
  This host's ZED is not running while its mount service is:
  read 2026-11-22 with `systemctl --no-ask-password is-enabled zfs-zed.service`,
  `systemctl --no-ask-password is-active zfs-zed.service`,
  and the same two for `zfs-mount.service`,
  they answer `disabled`,
  `inactive`,
  `enabled` and `active`.
  A trigger that travels through ZED therefore repairs nothing here today,
  whatever its source says about delivery.

## Assumptions

- A1.
  The checkout's immediate parent mount is shared on an ordinary boot,
  as F1 and F2 say for this host,
  unless the
  project changed it.
  Where it is private,
  every checkout mount comes back private (F4) and the propagation step does nothing
  but record what it found.
- A2.
  The handler,
  and not the wrapper,
  is the only actor that sets propagation:
  only root in the host namespace may
  (F7),
  and the accepted design puts every mount under the checkout in the handler's hands.
- A3.
  The lost-event sibling owns the triggers,
  and at its tip `88f6a15194` the provider hook is two files:
  a `zfs-mount.service` drop-in whose `ExecStartPost=` is prefixed `-` and asks the boot unit to act with
  `systemctl --no-block --no-ask-password reload-or-restart reconcile-git-worktree-clone-mounts.service`,
  and the boot unit's `ExecReload=`,
  which is the line the lost-event record accepts (sibling,
  "The hook,
  in one form a sibling can cite";
  [record][lost-record],
  "Which trigger runs after the checkout becomes ready").
  Its event filter,
  its V2 option A,
  passes only events of the clone parent's subtree,
  so a checkout dataset's own history events are dropped (F31).
  This design assumes that hook and that filter,
  and it assumes the sibling keeps citing this design's order inside an entry point instead of stating one,
  which it does at that tip in "Order inside a trigger".
  Both are unapproved proposals,
  and "Dependencies on sibling designs" states every remaining difference.
- A4.
  The build may add a mode or a step to the installed handler binary and a field list to a root-owned record,
  and
  the release still ships the two executables `SPEC.md` names ([`SPEC.md`][spec],
  lines 249 to 272).
  Nothing here
  needs a third executable,
  and no new unit beyond the one the project's trigger answer installs.
- A5.
  `mount_setattr` and `statmount` are available to the release's targets.
  `mount_setattr` is already called by the
  built start probe with `MS_PRIVATE` and a `mount(2)` fallback
  (`src/native/start_probe/probe_mount_calls.rs:30` to `:60`),
  and `statmount` is not called anywhere in the
  repository today,
  so its use here is new code on a kernel interface the ABI gate does not yet cover.
  The callers are two:
  the handler's step and the sweep's admission,
  and the wrapper's `doctor`,
  which opens the checkout path and reads the unique mount number (F21,
  F37).
  Whether the release targets' `libc` exposes `SYS_statmount` and `struct statmount` is unverified here.
- A6.
  The checkout's path and dataset reach the step through whatever the handler resolves them with:
  the configuration's `cloneMounts.checkout` and `cloneMounts.checkoutDataset` when the file names them,
  and the finder the project decided for a file that names neither (F34;
  M5).

## Limitations

- L1.
  Nothing here was measured.
  Every kernel,
  ZFS and systemd sentence is read from source at the tags named under
  "Sources read".
  The guest arms this design needs are owed under "Measurements owed",
  and no guest ran this round.
- L2.
  The event question is answered from source,
  not from a run:
  which event fires for which way a mount is replaced,
  and whether systemd notices a quick unmount and mount at all,
  has not been observed on this machine or any guest (M4).
- L3.
  A checkout made private in the host namespace is not thereby private everywhere.
  Copies of the checkout's mount
  that propagation already delivered to other namespaces stay where they are,
  and F3 invents a fresh peer group for
  each new instance rather than reconnecting to an old one;
  the measured namespace limits stand
  ([measurements][measurements],
  item 15;
  [proposal][proposal],
  "Mounts held by other namespaces").
- L4.
  Between a replacement and the next root handler run,
  mounts made by anyone else under the checkout are created
  under a shared parent and are propagated to its peers (F3),
  and on this host that window is routine rather than exceptional:
  `zfs mount -a` mounts the vm-builder output dataset under the new shared checkout before any handler run (F29,
  F33).
  The propagation step cannot prevent that;
  it can only shorten the window to the next run of the handler,
  which the project's first answer makes the repair for everything the trigger misses.
- L5.
  Whether systemd notices a quick unmount and mount,
  and how the three limits behave under a flap of the mount unit,
  are unmeasured (F23,
  F38;
  M4):
  the upheld queue's,
  the service's own start limit,
  and systemd's mount-table event limit.
  A drop-in on the checkout's mount unit therefore relies on systemd's own reading of the mount table,
  which is a source reading here and not an observation.
- L6.
  The step changes the checked mount's peer group and nothing else.
  Whether that is enough for the staging move,
  the rollback and the all-namespace scan is measured elsewhere (F20,
  and the staging design's arms),
  not here.
- L7.
  A lazily detached clone plausibly stays mounted in a second mount of the same dataset (F11),
  and this design neither releases it nor finds it.
  What it does about that state is unmeasured (M5);
  the removal's holder rules and step 9's unmount are the designs that look for holders
  ([proposal][proposal],
  "Project decisions",
  "Blocked-removal cancellation").
- L8.
  The record is a cache written after the act.
  A reader that arrives between the set and the write sees the old
  record;
  every reader therefore compares it with live facts (F8,
  F18) and none acts on it alone.
- L9.
  The wrapper's refusal depends on the preflight that step 10 builds.
  Until that step lands the wrapper says
  nothing about propagation,
  and a caller learns of it only at the sweep's refusal and its deadline.
- L10.
  A refused step fails its unit only on the start path,
  and there that also runs the unit's `ExecStopPost`,
  which is `shutdown-unmount`,
  leaving the managed clones unmounted until the next attach admits (F35,
  M7).
  A refused reload leaves the unit active with its reload result set and unmounts nothing (F35).
  Whether the start path's order is right for a refused prerequisite is a property of step 9's unmount and not
  designed here.
- L11.
  The design prints the lost-event design's root command unchanged,
  and that command's one condition is the boot unit's `ExecReload=`,
  which its record accepts (sibling,
  "T5:
  a trigger the project fires";
  [record][lost-record]).
  On a machine whose boot unit lacks that line,
  which is an older install and not one this design provisions,
  the same command on an active unit is a restart whose stop pass unmounts every clone,
  so the printed command there is `start` for a stopped or failed unit and nothing for an active one,
  and the propagation service is not installed at all ("The propagation service and its drop-in").
- L12.
  The mount-unit trigger is provisional ([record][repair-record],
  "Which triggers the route adds"),
  and it is dropped if M4 shows that it misses a replacement typed by hand or that it loops.
  Until M4 runs,
  the reach stated under "The trigger the route adds,
  and what it must show" is a source reading (L2),
  and only the first answer's rule,
  the next run of the handler,
  covers a replacement systemd does not show.

## What a replacement mount is

### The ways a checkout mount instance is replaced

A mount instance is the kernel's mount object:
it carries the propagation,
the peer group and the namespace link,
and a replacement is any act that leaves a different
object at the checkout path.
Each of the seven ways says what the new instance's propagation is and what becomes of the children attached under the
old one.

1.  An unmount and a mount,
    by `zfs unmount` then `zfs mount`,
    by `umount` then `mount`,
    or by a job on the mount unit where that unit has a fragment:
    `systemctl restart home-user-Monochromatic.mount` acts only on a unit a generator or an administrator wrote,
    because a mount unit systemd learned from its table cannot be started at all (F36),
    and on this host every dataset mount unit is of that kind (F33).
    A job on such a unit stops it,
    which unmounts the checkout,
    and its start then fails,
    so the way it is meant to serve is reached by `zfs mount` afterwards or not until the next boot.
    An ordinary unmount refuses while any child mount stands,
    a clone or the vm-builder output dataset (F9,
    F29),
    so
    the route's own stop pass runs first,
    and an unmount with those attached is refused `EBUSY` by the kernel,
    not by
    this route.
    Once the old instance is gone and a new one is mounted,
    F3 makes the new one shared and its later children
    shared with it.
    A child mount cannot survive it,
    because the unmount refused while it stood.
2.  A lazy unmount and a mount,
    `umount -l` then a mount.
    The lazy unmount does not refuse the children:
    it detaches the whole subtree from the namespace (F10).
    Their datasets stay mounted,
    and a later mount of the same dataset reuses the live superblock instead of refusing
    (F11),
    so a clone can plausibly come back as a second mount while the detached one stays invisible;
    that is unmeasured (M5).
    The new instance at the checkout path has no children and is shared by F3,
    and F18's check refuses every clone
    until the detached mounts are gone.
    The handler can see that an old instance has no namespace only from a descriptor it opened before the detach
    (F8),
    which is the race test 6 forces,
    not a way to find a detached mount afterwards.
3.  A mount after a late key load.
    The checkout's dataset was not mounted at boot,
    its key is loaded later,
    and the key-load unit or the mount unit
    then mounts it,
    or `zfs-mount.service` mounts it with `zfs mount -a` (F15,
    F16).
    The boot unit's run either happened before the mount and refused the checkout as not ready,
    or has not happened
    yet;
    either way the new instance is shared by F3,
    and what reaches the step is "The trigger the route adds,
    and what it must show".
4.  A rollback.
    A plain `zfs rollback` of a mounted checkout is not a replacement:
    `zfs_ioc_rollback` suspends the mounted file
    system,
    rolls back and resumes it,
    so the instance and its propagation stay as they were (F26).
    What a rollback can change is the underlay's immutable entry,
    when it goes back past the snapshot that set it,
    and
    that is the protection the design already requires verifying before automated attach
    ([proposal][proposal],
    "One-time move and checkout boot protection",
    lines 309 to 315;
    [measurements][measurements],
    item 8).
    So a rollback is a protection event and not a propagation event,
    and a rollback of a mounted checkout needs no
    re-application.
5.  A rename.
    A plain `zfs rename` of a file system whose mountpoint is a path unmounts it and remounts it through the
    changelist (F27),
    which makes it way 1 with the `zfs` program as the actor;
    `-u`,
    and the `legacy` and `none` mountpoints,
    keep the mount (F27).
    Renaming the directory at the checkout path needs the dataset unmounted,
    so it is way 1 too.
    `mount -o remount` is not a replacement:
    it acts on the existing mount object and leaves its propagation alone,
    and
    `doctor` prints the live tag so the expectation is checked rather than assumed.
6.  `zfs mount -a`,
    `zfs mount <dataset>`,
    and a reimport followed by either.
    `zfs mount -a` skips a dataset that is already mounted,
    and an explicit `zfs mount` of one refuses (F28),
    so this
    way is the second half of a replacement whose first half was way 1,
    way 2 or way 5.
    The accepted staging design recorded the same reading
    (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
    lines 1006 to 1008).
    systemd notices the mount (F12);
    whether anything starts is "The trigger the route adds,
    and what it must show";
    ZFS posts no
    mount event (F17);
    the new instance is shared by F3,
    and on this host `zfs mount -a` also mounts the vm-builder
    output dataset under that shared instance (F29,
    F33) before any handler run.
7.  A mount that completes between two of systemd's reads of the mount table,
    or inside a namespace systemd does not
    read.
    systemd compares no mount number and marks a mount point mounted again only from its own memory of the table,
    so a pair of commands that finishes between two reads leaves the entry equal and starts nothing (F38).
    That is the case no trigger bound to a mount unit can see,
    and it is unmeasured (M4);
    what only M4 can settle is said there and not assumed here.

### The trigger the route adds, and what it must show

The project chose one trigger beyond the root command,
provisionally ([record][repair-record],
"Which triggers the route adds"):
a drop-in on the checkout's mount unit and one one-shot unit that asks the boot unit to reconcile when the mount comes
back,
with the same request the lost-event design's readiness hook uses.
"The propagation service and its drop-in" gives both files.
The root command stays whatever is chosen,
and it is the sibling design's:
`systemctl --no-ask-password reload-or-restart reconcile-git-worktree-clone-mounts.service`,
printed by the wrapper's refusal and by `doctor` (sibling `88f6a15194`,
"T5:
a trigger the project fires").

What the trigger reaches,
read from source and not observed (L2):

- The mount unit's transitions that systemd sees,
  which covers ways 1,
  2,
  3,
  5 and 6 wherever the entry leaves the table and returns between two reads (F12,
  F23,
  F24).
- Not a replacement that finishes between two reads,
  because systemd compares no mount number and marks the entry equal (F38).
- Not a replacement inside a mount namespace systemd does not read,
  which is way 7.

A restart of `zfs-mount.service` is not a replacement either:
it re-runs the boot unit through `Requires=`,
but `zfs mount -a` replaces nothing when the checkout is already mounted (F28),
so the re-run finds a private checkout and records that it did.
A replacement is what the two misses above are about.

What covers the two it misses is the project's first answer:
the next run of the handler,
started by any event the route has,
checks the checkout and sets it private before it attaches anything (F18,
"The step that re-applies the setting").

Two things it costs,
in the record's words:
a `systemctl` request from a unit's start is untested until a guest runs it (the sibling's A9,
and this design's M4),
and it adds one unit and one drop-in to install and to report.

The guest measurement that decides whether it stays is M4,
and it must show three things:
that the trigger sees a replacement typed by hand,
that it does not loop,
and how the three limits behave,
which are the upheld queue's 16 starts in 10 seconds,
the service's own 5 in 10 seconds,
and systemd's own mount-table limit of 5 events in 1 second (F38,
F23).
If it misses a typed replacement or loops,
the record drops it,
and the root command is what remains (L12).

The triggers the project did not choose are listed once under "The project's four answers,
and what this design does
with each",
and none of them is ranked here.

### What the clones under the old instance become

Four outcomes,
and each needs saying because the sweep's answer differs:

- Released:
  a stop pass unmounted them before the checkout was replaced (F9,
  F29),
  so they are gone and their datasets
  are free.
  The sweep attaches them again on intent.
- Detached with the checkout:
  a lazy unmount took them out of the namespace (F10).
  Their datasets are still mounted,
  so a later attach of one plausibly becomes a second mount of the same superblock
  while the detached one stays invisible (F11),
  which is unmeasured (M5).
  Nothing in the handler's namespace reaches the detached one,
  and `doctor` reports the clone with its intent
  standing.
- Blocked by a child:
  while the vm-builder output dataset is mounted under the checkout,
  an ordinary unmount of the
  checkout is refused (F29),
  and a lazy one detaches it too.
- Never attached:
  nothing to do.

Which outcome each of the seven ways gives,
in the same order as "The ways a checkout mount instance is replaced":

1.  An unmount and a mount:
    released,
    because the unmount refused until the children were gone (F9,
    F29).
2.  A lazy unmount and a mount:
    detached,
    and the new instance is the one a later attach has to work from (F11,
    M9).
3.  A mount after a late key load:
    never attached,
    and the boot unit's earlier run had refused the checkout as not
    ready.
4.  A rollback of a mounted checkout:
    unchanged,
    because the rollback does not replace the instance (F26);
    a rollback
    that needed an unmount was way 1.
5.  A rename:
    released where the changelist unmounted them (F27),
    and untouched where `-u` kept the mount.
    Whether the legacy-mounted clones are among the dependents it unmounts is unmeasured (M9).
6.  `zfs mount -a` and the others:
    the second half of way 1,
    2 or 5,
    so that way's outcome.
7.  A mount no witness saw:
    the second or the fourth,
    and nothing here can tell which.

## The step that re-applies the setting

### Where it runs, as whom, and in what order

The step is a new module of the installed handler,
`checkout_propagation`,
called as the first act of the two entry
points that decide about attaches,
once the dispatch's compatibility gate has passed,
and before either one enumerates anything.
The gate runs before the mode,
so a gate refusal skips the step as well
(`src/native/worktree_clone_mount_handler/handler_mode_dispatch.rs:118` to `:153`).
Both run as root in the host mount namespace ([proposal][proposal],
"Boot and shutdown unit",
lines 499 to 520).

- `boot-reconcile`,
  from `ExecStart` of `reconcile-git-worktree-clone-mounts.service`,
  in this order:
  read the
  configuration and resolve the checkout;
  **the propagation step**;
  the start probe's record and the boot's facts;
  the
  pool's state;
  publish readiness `running`;
  the sweep;
  publish readiness `finished` or `failed`.
  That is the order the
  built mode has today,
  with the step inserted after the resolution and before `prepare`
  (`src/native/worktree_clone_mount_handler/boot_reconcile_mode.rs:1` to `:40`;
  [`SPEC.md`][spec],
  lines 6283 to 6289).
- the default invocation,
  which is what the installed ZED event starts and what a caller's nudge starts,
  in this
  order:
  read the configuration and resolve the checkout;
  **the propagation step**;
  the start probe's record;
  the
  pool and the checkout checks;
  one sweep.
  It publishes no readiness,
  and it must not start doing so,
  because the
  readiness record is the boot mode's (`attaching_sweep_mode.rs` calls no readiness writer;
  [`SPEC.md`][spec],
  lines 6277 to 6282).
  A step that published readiness from a ZED event would tell a caller a
  boot's checks had passed when only a sweep had run.

Neither `shutdown-unmount`,
`start-probe` nor `adopt-worktree-parent` calls it:
the first unmounts and changes no
durable state,
the second already runs in a private namespace of its own and changes propagation there
(`src/native/start_probe/probe_mount_calls.rs:30` to `:60`),
and the third records identities and mounts nothing.

A refusal in the step enumerates nothing,
attaches nothing,
and publishes no readiness,
which is the design's existing
sentence for a failed prerequisite:
"A failed prerequisite authorizes no attachment" ([proposal][proposal],
"Boot and shutdown unit",
line 523).
Its exit status is `2`,
the status every refusal of this handler gives,
and what that does to the boot unit depends on the path it came by:
on the start path the unit fails and runs its stop pass,
while on a reload it stays active with its reload result set (F35,
"What a failure reports").

Where the step sits among the checks is a writer's choice,
since no record orders it against the start probe's record;
"Choices this design marks for the project's veto" lists it (V5).

### What it verifies before

The step is about one instance,
and it proves which instance before it touches it:

1.  Take the checkout as the handler already resolved it:
    the path and the dataset together,
    from the configuration's
    `cloneMounts.checkout` and `cloneMounts.checkoutDataset`
    (`src/native/worktree_clone_mount_handler/sweep_configuration.rs:63` to `:67`),
    or,
    when the file names neither,
    from the finder the project decided.
    With no file,
    the project answered that "the root handler finds the checkout as the one mounted ZFS dataset whose
    last name component is `monochromatic-git-checkout`",
    that "Only a dataset that is mounted counts",
    and that "When
    the name matches none or several,
    the handler refuses and lists them" ([sweep-choices][sweep-choices],
    "With no configuration file",
    lines 44 to 51;
    F34).
    The step takes the same answer from the same place and never reads the key itself,
    so a file that names neither
    dataset leaves the step working,
    and the same two refusals,
    none and several,
    reach it unchanged,
    and the tests for each under "The tests a build must have" state them (F34).
2.  Open that path `O_PATH | O_DIRECTORY | O_CLOEXEC`.
3.  Read the descriptor's mount with `statmount(STATMOUNT_BY_FD)` (F8):
    `mnt_id`,
    `mnt_id_old`,
    `mnt_ns_id`,
    `mnt_propagation`,
    `mnt_peer_group`,
    `mnt_master`,
    `mnt_point`,
    `fs_type`,
    `sb_source`.
4.  Refuse unless all of these hold,
    and name the one that failed:
    the point is the checkout path;
    the type is `zfs`;
    the source is the resolved checkout dataset;
    the mount has a namespace id,
    so it is not a detached instance
    (F8,
    F10);
    and the propagation set is one this step knows.
5.  Read the live table through the repository's own parser as the sweep does (F18),
    find the row whose first field is this mount's `mnt_id_old`,
    and refuse when the row is absent or describes another mount.
    The instance is named by the unique `mnt_id` the descriptor read gave,
    because that is the number the kernel never reuses (F8,
    F37);
    `mnt_id_old` is only what finds the table's row (F31).
    A row that is absent,
    or whose source,
    type and point are not the ones the descriptor read gave,
    is a replacement between the two reads,
    and nothing is set on either instance.

Everything here is a read.
The refusal that follows the reads leaves the mount as it was,
and the account never has the authority the change
needs (F7).

### The call that sets it

`mount_setattr(fd, "", AT_EMPTY_PATH, &mount_attr{ propagation: MS_PRIVATE }, size_of::<mount_attr>())`,
with the
descriptor from the verify read,
never with a path (F5,
F6).
This is the repository's own idiom for the same change:
the start probe already sends `mount_setattr` with
`propagation: MS_PRIVATE` and falls back to `mount(NULL, path, NULL, MS_REC|MS_PRIVATE, NULL)`,
both inside a namespace
of its own (`src/native/start_probe/probe_mount_calls.rs:30` to `:60`).
The step has no path fallback:
a path fallback is the substitution window the staging design refuses everywhere else,
and a descriptor that is refused `EINVAL` for having no namespace (F6) is a fact the record should carry,
not
something to route around.
Having no fallback is a writer's choice,
and V4 lists it.
Two arguments carry it,
and "What each check owes the threat model" states both once:
the set changes only an instance the step verified,
which a trusted machine still needs because a replacement
between the read and the set is an administrator's or another run's act;
and a path fallback would act on whatever the path names at that moment,
which is the substitution window only a hostile account needs,
and the staging design refuses that window everywhere else.

The step acts when the propagation set carries `MS_SHARED`,
because that is the measured disqualifier (F20,
and
[measurements][measurements],
item 3),
and it acts on the set rather than on one word:
a mount that is shared and
slave at once is ordinary (F30),
the repository's own measurement printed exactly that for one mount,
and the built
check reads the `shared:` field,
which such a mount carries (`src/native/worktree_clone_mount_handler/sweep_checkout_mount.rs:172`).
A set with `MS_SLAVE` or `MS_UNBINDABLE` and no `MS_SHARED` carries no `shared:` tag and is admitted by the built check
today (F18),
which is what the project decided:
"the repair sets the mount private only when it is shared",
and "a mount that is slave,
unbindable or a combination is left as it is,
the route goes on,
and the state is reported" ([record][repair-record],
"What the repair does with a state that is neither private nor shared").
The record carries the set it found.
Where the set carries `MS_SHARED` and `MS_SLAVE` together,
setting `MS_PRIVATE` also detaches the mount from its master,
so it stops receiving the master's mounts (`fs/pnode.c:93` to `:124`).

No `AT_RECURSIVE`,
and the reason is written beside the call:
recursion would make every mount under the checkout
private too,
including the vm-builder output dataset,
which the route did not make,
and the design's sentence is
"Do not silently change propagation" ([proposal][proposal],
line 1298).
Recursion would also change a mount the route did not make,
and that is what the project did not choose:
the record decides that "the repair changes the checkout's own mount and no mount beneath it,
and reports a shared mount it finds beneath" ([record][repair-record],
"How far the setting reaches").
So a shared mount found under the checkout is reported and not changed,
which is the gap the design's earlier question about recursion put to the project and the record closed.

### What it verifies after

The same descriptor,
read again with `statmount(STATMOUNT_BY_FD)`,
and all three must hold:
`mnt_id` is the unique
number the before read gave,
so the instance is the same;
the propagation set is `MS_PRIVATE`,
with `mnt_peer_group`
and `mnt_master` clear;
and `mnt_ns_id` is still present,
so the instance was not detached under the call.
Anything else is a refusal,
and the record says which one.
Test 5 forces a set that reports success and leaves `MS_SHARED` standing,
and test 6 the instance that is detached
between the two reads.

Then,
and separately,
the sweep's own live check re-reads the checkout from the mount table (F18) before the first clone
is touched.
That second check is the decision's "Recheck it regardless of who sets it" made mechanical:
the step proves what it
set,
and the sweep proves what it is about to attach under,
and a replacement in between is caught by the second read
and refused (test 8).

### What it writes

One record,
over the previous one,
published the way the readiness record is published (F19):
root-owned `0755`
directory `/run/git-worktree-clone-mount-propagation`,
root-owned `0644` file `result`,
staged under
`result.staging.<pid>.<attempt>` and moved into place with one `rename`,
refused when the writer cannot read its own
record back.
No lock:
two runs write the same facts about the same instance,
the last `rename` wins whole,
and a reader sees one
complete record (rule `NLK`).
"The propagation record" states its fields and their bounds,
including the two mount numbers and the propagation as a
set.

## What each check owes the threat model

The project's sentence is "We have to assume my machine isn't compromised.
sudo is passwordless here anyway.",
read in
the decision as "the account is trusted,
and root and the account are not a security boundary here"
([staging decision][staging-decision],
"The handle record's place",
lines 221 to 237),
and the accepted staging design
followed it "without dropping one on the strength of the sentence alone".
This design follows both,
and says for each check whether it survives that sentence or exists only for a hostile
account.

- The descriptor-only set,
  with no path fallback (V4):
  survives.
  The race it closes is an administrator's replacement,
  or another handler run,
  between the read and the set,
  and both
  happen on a trusted machine.
  A hostile account needs no part in it,
  and the refusal is about knowing which instance was changed and not about
  who could substitute one.
- The source,
  type and point checks (step 3 of "What it verifies before"):
  survive,
  for the same reason.
  They prove the step is changing the checkout's dataset mount and not a bind,
  a tmpfs or another dataset at the same
  path,
  which is what a boot whose provider mounted the wrong thing produces.
- The namespace-id check:
  survives.
  A detached instance is what a lazy unmount leaves,
  and no account action is needed to produce one.
- The instance comparison by the unique mount number:
  survives.
  A disagreement is a replacement between two reads,
  which any root or systemd act can cause,
  and the reused number would miss a replacement that took the old number back (F37).
- The record's owner and mode checks and its read-back refusal:
  survives in part.
  A trusted machine still has to tell a record root wrote from one a stale writer half-wrote,
  so the read-back and the
  proof of the directory stay.
  What only a hostile account motivates is refusing a record whose owner or mode is wrong:
  the account could replace
  the record to steer `doctor`,
  and `doctor` acts on nothing.
  That check stays,
  as the staging decision keeps its own,
  because it costs one `stat` and makes the two records
  behave alike.
- The fact that the wrapper refuses rather than setting the propagation itself:
  not a check but the authority
  boundary,
  and it survives:
  the account has no `CAP_SYS_ADMIN` (F7),
  so this is not a trust decision at all.

No check in this list is dropped on the strength of that sentence,
and none is added because of it alone.

## What the wrapper and the caller see between the replacement and the re-run

The wrapper has no authority here (F7),
so its part is to say the truth early and to say what root must do.
Step 10's preflight already lists "mount propagation" among the checks it shares with `doctor`
([proposal][proposal],
"Add and Git registration",
step 1,
lines 365 to 375),
and this design fills that check in with
the repository's own reader (F21),
which the account can call.
The refusal is printed before any claim,
snapshot,
clone or registration is written,
so nothing managed is left
behind:

> `/home/user/Monochromatic` is mounted with the `shared` propagation tag,
> so a clone mounted under it would be delivered to another mount namespace,
> and this wrapper does not change propagation because it has no authority over mounts.
> Nothing was claimed,
> snapshotted,
> cloned or registered.
> Ways forward:
> as root,
> `systemctl --no-ask-password reload-or-restart reconcile-git-worktree-clone-mounts.service` re-runs the handler's
> checks and its sweep,
> and then this command can be run again;
> `monochromatic-git-policy-cli doctor` prints the live tag and the handler's last propagation record.

`reload-or-restart` is named because both other words fail in one of the two states a replacement leaves the unit in:
a reload is refused on an inactive unit,
which is what a provider restart that stopped it leaves behind (F25,
F13),
and a plain `start` on an active unit with `RemainAfterExit=yes` runs nothing.
`reload-or-restart` reloads it when it is active and starts it when it is not (F25,
F39).
The command is the sibling design's,
printed here unchanged,
with `--no-ask-password` as the sibling prints it (sibling,
"T5:
a trigger the project fires"),
and its one condition is the unit's `ExecReload=`,
which both documents state and the record accepts ([record][lost-record];
sibling,
"T5:
a trigger the project fires").
Without that line the same command on an active unit is a restart that runs the stop pass and unmounts every clone.

The command the refusal prints is one this wrapper never runs and never waits on.
A request to change or signal a system unit is answered by the caller session's authorization agent,
and the command
waits while that agent asks (F22),
which is why this route prints the act instead of performing it:
nothing here asks
for a credential,
and nothing here starts a dialog.
A script that must not wait adds `--no-ask-password`,
which turns the wait into a plain refusal (F22).

A caller already waiting when a replacement happens has written its intent and its nudge;
the sweep that answers it
runs the step first,
and either attaches or refuses.
A refusal files no receipt,
which is step 8's built rule ([`SPEC.md`][spec],
lines 6364 to 6369),
and the caller's own
deadline report already says that a deadline is not cancellation and names the journal ([`SPEC.md`][spec],
lines 6409 to 6416).
This design adds no receipt for a propagation refusal and does not change that rule.

`doctor` is the account's read-only view:
it prints the checkout's live propagation set,
the record's boot,
both mount numbers,
outcome and failing step,
and whether the record still describes the live instance.
That last comparison opens the checkout path and reads the descriptor's unique `mnt_id` the way the step does,
which needs no capability in this namespace (F37),
and compares it with the record's `mount-id`;
the table row is printed only to show what the reused number names today (F21).
Where the unique number differs it says the record names an instance that is no longer the checkout's;
where the descriptor read is refused it says the record names the same number and not a proven instance,
and prints the same root command.
The reused number alone is never reported as freshness,
because a replacement can take it back (F37).
The comparison is a host-namespace one and says so:
a `doctor` run inside a mount namespace that holds a propagated copy of the checkout reads that copy's unique number,
which differs from the host's,
so the line names the namespace it read in and does not call the record stale on that difference alone.

## What a failure reports, and where it is read

Five places,
each with a different reader,
and none of them a prompt:

- The step's exit status,
  which is what the unit sees first:
  a refusal exits `2`,
  the handler's status for every refusal,
  and what that status does depends on the path (F35):
  on the start path it fails the boot unit,
  and after the first boot the repairs this design names arrive as reloads,
  where it sets the unit's reload result and leaves the unit active.
- The handler's summary line,
  which an operator and a guest arm read:
  `applied`,
  `already-private` or `refused` for the
  checkout,
  with both mount numbers,
  the propagation set before and after,
  the failing step and the errno on a
  refusal.
- The handler's journal,
  under the unit's name,
  which the design already names in its refusals ([`SPEC.md`][spec],
  lines 6211 to 6212,
  and the refusal texts in `clone_mount_readiness_admission.rs:100`).
  The step logs the same fields the record carries,
  tagged `checkoutPropagation`.
- The record,
  `/run/git-worktree-clone-mount-propagation/result`,
  which `doctor` reads and which any account can read
  without root.
  It is a cache:
  F18's live check is what admits an attach,
  not the record.
- The boot unit's reload result,
  `systemctl show --property=ReloadResult`,
  which is where a refusal that came by a reload is read (F35):
  the unit stays active,
  `systemctl --failed` shows nothing,
  and the job is failed only for a blocking caller.
- The readiness record and the sweep's refusal,
  which are unchanged in shape:
  a shared checkout is `not-ready` (F18),
  readiness is not published for that run,
  and any attach is refused with the words the build already has,
  extended by
  the step to name the record and the root command.

A refused start is not only a report,
and a refused reload is a quieter thing.
A unit whose start fails runs its `ExecStopPost` commands ([proposal][proposal],
"Boot and shutdown unit",
lines 523
to 527),
and this unit's are `shutdown-unmount`,
so a refused start in `boot-reconcile` also runs the stop pass and
leaves the managed clones unmounted until the next attach admits.
That consequence is L10,
it belongs to the start path only,
and the design states it and does not design around it,
because the stop pass and its refusal rules are step 9's.
A refused reload runs nothing else:
the unit stays active,
its reload result names the failure,
and the record and the journal carry the step's fields (F35).
A refusal inside the sibling's provider hook cannot fail `zfs-mount.service`:
the hook is prefixed `-`,
and what it runs is a job request rather than a reconcile (sibling,
"The hook,
in one form a sibling can cite").
A refusal there is the boot unit's,
and it shows in that unit's journal and in the propagation record,
which is the consequence the sibling states as Q-A's cons.

Nothing is reported anywhere but those five places,
and nothing asks for a credential or a password:
the wrapper's
refusal names the command root runs and prompts for nothing ([`SPEC.md`][spec],
lines 7345 to 7352).

## What becomes of `Type=oneshot` with `RemainAfterExit=yes`

It stays.
Four reasons,
and one addition:

- The unit's remaining duty is the shutdown half,
  and that needs a unit that was started and stays active so that its
  `ExecStopPost` runs at a stop:
  a failed start already runs it,
  and a failed reload does not,
  "execution continues with commands specified in
  `ExecStopPost=`" (systemd `v262`,
  `man/systemd.service.xml:456` to `:459`),
  and `RemainAfterExit=` is what keeps a
  oneshot service active after its process exits (`:346` to `:353`).
  The design's `ExecStopPost` passes are unaffected.
- The installed unit's `Requires=` on the provider already makes a provider restart try-restart it (F13),
  so a
  `systemctl restart zfs-mount.service` re-runs `boot-reconcile` without anything new being installed.
  That is the one job-driven case the unit covers today,
  and it reaches ways 1 and 3 when systemd runs that restart;
  a restart of the checkout's own mount unit does not reach them on this host,
  because a mount unit systemd learned from its table cannot be started (F36).
- `RemainAfterExit=yes` keeps one boot's readiness publication at one start,
  which is what the record's `running` and
  `finished` states mean,
  and a ZED-driven run of the default invocation publishes no readiness ([`SPEC.md`][spec],
  lines 6277 to 6282,
  and "Where it runs,
  as whom,
  and in what order").
- The addition is `ExecReload=<the same handler> boot-reconcile`,
  paired with `reload-or-restart` as the printed
  command,
  so that a root command re-runs the checks,
  the propagation step and the sweep whether the unit is active
  or not (F25).
  It is one line of the unit step 19 installs,
  and the lost-event record accepts it ([record][lost-record],
  "Which trigger runs after the checkout becomes ready").

What the unit still cannot do is notice a replacement systemd did not show it,
which is way 7 and F38;
the trigger the project chose is the drop-in and the service on the mount unit (F23,
F24),
and what it misses the next run of the handler repairs.

## The propagation record

One file,
replaced whole,
for the reason the readiness record gives for its own shape (F19):
a reader sees one
complete answer,
and no reader takes a half-written one.
It is written by whichever root entry point ran the step,
after the step,
and it names the instance so that a reader
can compare it with the live one (F8).

Its text is `key value` lines,
values running to the end of the line,
in this order:
`v1`;
`kind checkout-propagation`;
`boot`;
`mount-id`;
`mount-id-old`;
`mount-ns-id`;
`checkout`;
`source`;
`before`;
`after`;
`outcome`;
`step`;
`errno`;
`time`.

- `boot` is the kernel's boot identifier,
  compared the way the readiness record compares it.
- `mount-id` is `statmount`'s `mnt_id`,
  the number the kernel keeps unique until reboot,
  and the one the verify-after read and `doctor` compare (F8,
  F37).
- `mount-id-old` is `statmount`'s `mnt_id_old`,
  the number `/proc/self/mountinfo` prints (F31).
  It is kept to find the table's row and to print what an operator's own tools show,
  and it is never the proof of an instance:
  the kernel hands it out again from the lowest free number once the mount is freed (F37).
- `before` and `after` are a propagation set,
  in the combinations the kernel can produce:
  `private` for an empty set,
  `shared`,
  `slave`,
  `unbindable`,
  and `shared+slave`,
  which is how the kernel reports the bits (F30).
  `unbindable` never joins another word,
  because setting it clears the peer group and the master (`fs/pnode.c:93` to `:124`).
- `outcome` is one of `applied`,
  `already-private` or `refused`.
- `step` is `none` on a success,
  and otherwise the step that refused:
  `resolve`,
  `open`,
  `statmount`,
  `point`,
  `type`,
  `source`,
  `namespace`,
  `before-table`,
  `set` or `verify-after`.
  `resolve` is the no-file finder's answer when it matched none or several (F34).
- `errno` is the kernel's error number in decimal where a kernel call answered,
  and `none` otherwise.
- `time` is the time with its offset,
  as the readiness record writes it.

The field bounds are the kernel's and the existing records':
36 bytes for the boot id,
20 decimal digits for each of the
two mount numbers and for the namespace id,
4095 for the path (`MAXIMUM_CHECKOUT_PATH_BYTES`,
the readiness record's
own),
255 for a ZFS dataset name (`MAXIMUM_POOL_NAME_BYTES`),
13 for the propagation set,
which is `shared+slave` and its terminator,
the longest set the kernel can produce:
`MS_UNBINDABLE` clears the peer group and the master,
so it stands alone (`fs/pnode.c:93` to `:124`),
15 for the longest outcome,
32 for a step word,
3 for
an error number,
and 35 for a time.
The file bound is derived from those and the key names the same way the readiness record's bound is derived and stated
beside its constant ([`SPEC.md`][spec],
"The readiness record",
lines 6192 to 6193);
the build computes the number and
writes it beside `MAXIMUM_PROPAGATION_BYTES`,
and this design ships no invented value (rule `LQM`).

The writer's side reuses `clone_mount_readiness_file.rs`'s publication and refusal machinery (F19):
the directory made
and proven root-owned `0755`,
the staging name carrying pid and attempt,
one `rename`,
a sync of the directory,
and a
read back before the write is called done.
The reader's side proves the directory's and the file's owner and mode before believing either,
exactly as that
module's reader does.
`doctor` prints the record,
compares the unique `mount-id` with the descriptor's live number,
and never needs it to act.

The precedent the project recorded for a root-owned store is weighed and not followed in one respect
([slot store][slot-store]):
that store holds one file per item because items accumulate,
and this record holds one
file because one checkout has one current answer.
Where a slot's file needed `O_EXCL` so that exactly one writer creates it,
this record needs the single `rename` so
that exactly one whole record stands,
which is the operation that store's parent record uses for the same purpose.

## The tests a build must have

Each test names the race it forces and the way it forces it without timing luck:
every kernel-facing piece is behind
a trait or a fixture,
and no test waits.

1.  A private checkout is left alone.
    Read says `private`;
    the setter is a double that fails the test if called;
    the
    record says `already-private`.
    No race.
2.  A shared checkout is made private.
    Read says `shared`;
    the setter records the mount id it was given;
    the second
    read says `private` with the same id;
    the record says `applied`,
    with `before` `shared` and `after` `private`.
    A second case reads `shared+slave`,
    which is the set the repository's own measurement printed (F30):
    the same
    result,
    `before` `shared+slave`.
    No race.
3.  A refusal at each verified field,
    one test each:
    an open that fails (`open`),
    a `statmount` that fails (`statmount`),
    a point that is another path (`point`),
    a type that is not `zfs`
    (`type`),
    a source that is another dataset (`source`),
    a missing namespace id (`namespace`),
    an unreadable table
    (`before-table`).
    Each asserts nothing was called on the setter and that the refusal names the field.
4.  The set refuses.
    The setter answers `EPERM`;
    the outcome is `refused` with step `set` and errno `1`;
    nothing is
    admitted.
    No race.
5.  The set appears to work and does not.
    The setter answers success and the second read still says `shared`;
    the
    outcome is `refused` with step `verify-after`.
6.  The instance is replaced between the before read and the set (race 1).
    The fixture's setter,
    when called,
    swaps the
    descriptor's mount to a detached one,
    which is the state the kernel refuses with `EINVAL` (F6),
    so the setter answers that;
    the outcome is `refused` with step `set` and errno `22`,
    and the test asserts the unique mount number in the record is the before one.
    Forced by the fixture,
    not by timing.
7.  The table read and the descriptor read disagree (race 2).
    The descriptor's `mnt_id` is 9001 and its `mnt_id_old` is 41;
    the table fixture has no row whose first field is 41;
    the outcome is `refused` with step `before-table`,
    the setter is never called,
    and the test asserts the record keeps both numbers and that the instance was named by the unique one (F31,
    F37).
8.  The instance is replaced between the step and the sweep (race 3).
    The step succeeds;
    the mount fixture then reports
    `shared` for the walk's read;
    the run refuses with the existing shared-checkout refusal,
    enumerates no clone,
    attaches nothing,
    and publishes no readiness.
    This is the decision's "Recheck it regardless of who sets it" as an
    assertion.
9.  Two runs at once (race 4).
    Two steps run in two threads against one fixture that both read before either sets;
    both
    set the same instance to `private`;
    both records are whole and name the same facts;
    no lock file is created
    anywhere,
    asserted by listing the record's directory.
    The interleaving is forced by the fixture's barrier,
    not by
    wall clock.
10. The record is published whole and refused when unreadable.
    A writer whose read-back fails leaves the previous
    record standing and reports the refusal;
    a reader refuses a record whose directory mode,
    file owner or file mode
    is wrong;
    a record naming another boot is stale and reported as such.
11. The record's parser refuses each malformed shape:
    a missing key,
    a wrong version,
    a wrong kind,
    an over-long field,
    an unknown word in `before`,
    `after`,
    `outcome` or `step`.
12. The boot mode publishes readiness only after the step admits.
    A refused step leaves no readiness record for the
    boot;
    an admitted step publishes `running` before the sweep and `finished` or `failed` after it.
13. The refusal wording.
    The sweep's shared-checkout refusal names the record,
    the unit,
    the journal query and the
    root command;
    asserted on the text,
    as the existing refusals are
    (`clone_mount_receipt_parse_tests.rs:243`).
14. The wrapper's refusal.
    The preflight fixture reports `shared` for the checkout;
    the add writes no claim,
    no intent,
    no snapshot request and no registration;
    the message names the command and prompts for nothing.
15. `doctor`.
    It prints the live word and the record's fields;
    a record naming another mount id is reported as a stale
    instance;
    a missing record is reported as never written,
    not as a failure.
16. Guest only:
    the set changes the kernel's answer,
    the pinned instance is the one changed,
    and a detached instance is
    refused.
    These are arms M1 to M4 under "Measurements owed";
    none of them can run as a host test,
    because every one
    of them mounts.
17. The no-file resolution.
    With no configuration file and a finder fixture that matches none,
    the step refuses with step `resolve` and names
    that nothing matched;
    with a fixture that matches two,
    it refuses and lists both.
    With one match it proceeds,
    and the configuration key is never read by the step.
18. The default invocation publishes no readiness.
    A run through the default entry point takes the readiness writer's double,
    which fails the test if it is called;
    a run through `boot-reconcile` publishes `running` and then `finished`,
    in that order.
19. The exit status.
    An admitted step gives `0` and a refused one gives `2`,
    and the refusal line is written before the status
    (`src/native/worktree_clone_mount_handler/handler_mode_dispatch.rs:105` to `:114`).
20. `doctor`'s comparison.
    It reads the live unique number from a descriptor and compares it with the record's `mount-id`;
    a record whose reused number matches a live row while its unique number differs is called a stale instance,
    which is the case the reused number cannot see (F37),
    and a record whose unique number matches is current.
    Where the descriptor read is refused,
    the line says the record names the same reused number and not a proven instance.

## Measurements owed

No guest ran this round (L1).
Each arm is written to be run as it stands,
in a throwaway guest with a pool of its own on kernel
`6.8.0-147-generic` unless it needs another,
and each says what it would show and what it would change.
They are additions to the record the build already owes ([measurements][measurements]) and to
[`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`](cli-git-zfs-measurements-owed-before-acceptance.md).

- M1.
  What a replacement mount's propagation is,
  and what a private parent changes.
  Create `tank/chk` with `zfs create -o mountpoint=/home/guest/chk tank/chk`;
  read
  `findmnt --output TARGET,SOURCE,FSTYPE,PROPAGATION --target /home/guest/chk`;
  then
  `mount --make-private /home/guest/chk` and read again;
  then `zfs unmount tank/chk` and `zfs mount tank/chk` and read
  again;
  then `mount --make-private /home/guest` and repeat.
  Would show:
  a replacement under a shared parent comes back `shared` (F3),
  and under a private parent `private` (F4).
  Would change:
  A1's scope,
  and whether the step can ever find a private instance it did not make.
- M2.
  That `mount_setattr` on a descriptor changes the instance that descriptor names,
  and refuses a detached one.
  With the checkout shared,
  `exec 9</home/guest/chk`;
  call `mount_setattr(9, "", AT_EMPTY_PATH, {propagation:
  MS_PRIVATE}, sizeof)`;
  read `statmount(9, STATMOUNT_BY_FD)` before and after,
  comparing `mnt_id` and
  `mnt_propagation`;
  then `umount --lazy /home/guest/chk` and call it again on descriptor 9,
  recording the errno;
  then
  `zfs mount tank/chk` and repeat on a fresh descriptor.
  A plain unmount would answer `EBUSY` while descriptor 9 holds the mount (F9),
  which is why the arm detaches instead.
  Would show:
  the call succeeds on the live instance with the same unique number and answers `EINVAL` on the detached
  one (F6),
  and what the new instance reports afterwards (F3).
  Would change:
  F6's reading,
  and the step's `namespace` and `set` refusals.
- M3.
  That the staging move and the clone attach behave as the decision's measurement said once the step has run.
  Repeat [measurements][measurements] item 3 with the built step in place:
  a shared checkout,
  then the step,
  then the
  `move_mount` of a staged mount (F20).
  Would show:
  `EINVAL` before the step and success after,
  with the clone private afterwards.
  Would change:
  nothing in this design,
  and everything in the build's acceptance arm.
- M4.
  What a mount made outside systemd does,
  way by way,
  and whether a drop-in's `Upholds=` service starts.
  In one guest,
  with a mount unit `home-guest-chk.mount` that systemd learned from the mount table (F12,
  F33):
  `zfs unmount tank/chk && zfs mount tank/chk`;
  `zfs rename tank/chk tank/chk2` and back;
  `zfs mount -a` after an
  unmount;
  and a two-command script that unmounts and mounts as fast as the shell can.
  After each,
  read `systemctl --no-ask-password show --property=ActiveState,InvocationID home-guest-chk.mount` and
  `journalctl --unit reconcile-git-worktree-clone-mounts.service --since '-2 min'`.
  Then install a drop-in `/etc/systemd/system/home-guest-chk.mount.d/10-upholds-propagation.conf` with
  `Upholds=checkout-propagation.service` and a unit `checkout-propagation.service` whose `ExecStart=` appends its
  start to a log,
  then the same with the design's own
  `systemctl --no-block --no-ask-password reload-or-restart reconcile-git-worktree-clone-mounts.service`,
  and repeat every act.
  Read the log,
  the propagation unit's `ActiveState` and `Result`,
  the boot unit's `InvocationID`,
  its `Result` and its `ReloadResult`,
  and the `Result` of the boot unit's last job where a blocking `systemctl` reports it.
  Then hold the boot unit busy:
  with a sweep long enough to observe,
  replace the checkout again during it,
  and read whether the request starts a second run or is absorbed (F39).
  Would show:
  which acts leave the mount unit dead and then mounted with a new invocation id (F12);
  whether anything starts without the drop-in (F13);
  whether the upheld unit starts for an extrinsic mount and for the quick act;
  what each exit does to it and to the boot unit,
  by path:
  a clean run leaves the propagation unit active and nothing restarts it,
  a start-path refusal fails the boot unit and runs its stop pass,
  a reload-path refusal leaves it active with `ReloadResult` set and unmounts nothing (F35),
  and a failed request is retried,
  throttled by the two limits and not stopped by them (F23);
  where the three limits sit under a flap of the mount unit and under `zfs mount -a`'s burst (F38);
  whether a replacement during a running reconcile is repaired at all (F39,
  and the first answer's rule);
  and whether a `daemon-reload` alone starts the propagation unit,
  which the source reading does not show and this design does not assume.
  Would change:
  L2,
  L5,
  L12,
  and the sentence on the activating state under "The propagation service and its drop-in".
- M5.
  What a lazy unmount does to a clone,
  and whether a second mount of its dataset succeeds.
  With a clone attached under the checkout,
  `umount /home/guest/chk` (expect `EBUSY`,
  F9);
  `umount --lazy
  /home/guest/chk`;
  then `zfs mount <the clone's dataset> /home/guest/chk/clone` and record whether it mounts,
  what
  `findmnt --output TARGET,PROPAGATION` says for it,
  and whether the detached mount is still there in `mountinfo`;
  then have the handler attach the same clone and compare.
  Would show:
  whether a second mount of a live dataset succeeds (F11),
  which decides what the sweep does after a lazy
  detach.
  Would change:
  F11,
  L7,
  and `doctor`'s words for that state.
- M6.
  What the step costs.
  Time `statmount` on a descriptor,
  the `mount_setattr` call and one whole sweep,
  each interleaved with a control of
  the same shape (a `stat` of the same path),
  in one guest,
  and report ratios with their ranges (rules `TVC` and
  `QNB`).
  Would show:
  whether the step is measurable beside the sweep at all.
  Would change:
  nothing shipped,
  because the design introduces no timeout and no cap,
  and it would give step 22 a
  number if one is ever wanted.
- M7.
  What must be run after a replacement,
  what admits an attach,
  and what a refused start leaves.
  `zfs unmount tank/chk` then `zfs mount tank/chk`,
  which is the pair a mount unit learned from the table needs (F36);
  then
  `monochromatic-git-policy-cli doctor` and one add;
  then a `systemctl stop` and `systemctl start` of a hand-written `home-guest-chk.mount` that carries its own
  `[Mount]` section,
  for the job-driven case;
  then `systemctl reload-or-restart
  reconcile-git-worktree-clone-mounts.service` and one add again;
  then a `boot-reconcile` whose configuration names
  another checkout dataset,
  so the step refuses at `source`,
  run twice:
  once as a start,
  from a stopped unit,
  and once as a reload,
  through `systemctl reload` on the active unit;
  then read the unit's `ActiveState`,
  `Result` and `ReloadResult`,
  the clones' mounts and the journal.
  Would show:
  whether readiness survives the stop;
  what the wrapper prints;
  that `reload-or-restart` admits the attach
  where `reload` alone is refused (F25);
  that a refused start runs `shutdown-unmount` and leaves the clones unmounted,
  while a refused reload leaves the unit active and unmounts nothing (F35);
  and where a person reads each of the two (F35,
  "What a failure reports").
  Would change:
  the printed command,
  the wrapper's refusal wording,
  and L10.
- M8.
  What an unprivileged request to the system manager answers.
  In a guest with an authorization agent on the session,
  and again with none,
  as an unprivileged account,
  run
  `systemctl reload reconcile-git-worktree-clone-mounts.service`,
  `systemctl reload-or-restart
  reconcile-git-worktree-clone-mounts.service`,
  and each with `--no-ask-password`;
  record each exit status and message.
  Would show:
  whether the wait and the refusal take the shapes F22 reads from source,
  and what each message says.
  Would change:
  the wrapper's refusal wording,
  and whether it must say plainly that the command asks for authorization.
- M9.
  Whether a rename unmounts the managed clones,
  and whether the nested output dataset comes back shared.
  With a clone attached and `zpcachyos/nosnap/vm-builder-output` mounted under the checkout,
  `zfs rename tank/chk
  tank/chk2`;
  read `findmnt` for the clone's slot,
  for the nested dataset and for the checkout's propagation;
  then
  `zfs mount -a` and read again.
  Would show:
  whether the legacy clones are among the changelist's dependents (F27),
  and whether the nested dataset is
  mounted under a shared checkout before any handler run (F28,
  F29).
  Would change:
  way 5's and way 6's clone outcomes,
  and L4.

### The acceptance arm step 19 owes

The proposal's arm for this item is "Re-applying checkout propagation.
Replace the checkout mount without a reboot and
assert the checkout is private again" ([proposal][proposal],
"Decided behaviours with no test yet",
lines 1926 to
1928),
and the plan repeats it ([plan][plan],
step 19,
"Acceptance tests").
It cannot assert "private again" without
naming a replacement and a trigger,
so step 19's arm is these,
each run in the guest on the built step:

1.  Restart `zfs-mount.service` through systemd,
    which is a job on a unit that has a fragment:
    assert the checkout reads `shared` before the trigger
    and `private` after the unit's re-run (F12,
    F13).
    The same job on the checkout's mount unit belongs here only where that unit was generated or written by hand,
    because a mount unit systemd learned from its table cannot be started at all (F36).
2.  `zfs unmount` and `zfs mount` of the checkout dataset with nothing attached:
    assert `shared` in the window,
    the
    wrapper's refusal for an add in that window,
    and `private` after the trigger the project chose ("The trigger the route adds,
    and what it must show").
3.  `zfs rename` of the checkout dataset:
    the same three assertions (F27).
4.  `zfs mount -a` after an unmount:
    the same,
    with the nested dataset's propagation read as well (F28,
    F29).
5.  For each of the four,
    the same sequence repeated with the sibling's proposed filter in place,
    which decides whether
    a ZED event reaches the step at all (F31,
    A3).

## What must be applied to the machine, and where that is recorded

Nothing this design adds is applied by a round.
Applying anything to this machine is the project's act outside a round,
and the design names the act,
its command and
the record that carries it:

- The unit file.
  `reconcile-git-worktree-clone-mounts.service` is written to
  `/etc/systemd/system/reconcile-git-worktree-clone-mounts.service` with the `[Unit]`,
  `[Service]` and `[Install]`
  contents the design states,
  including the `ExecReload=` this design adds,
  and applied with
  `systemctl daemon-reload` followed by `systemctl enable --now reconcile-git-worktree-clone-mounts.service`,
  both run
  by the project with its own authorization,
  never by a round and never by the wrapper.
  The content is step 19's,
  printed by `doctor` and written in the setup runbook step 19 delivers under
  `doc/runbook/`,
  and `doctor` reports the installed unit,
  its ordering edges and its `ExecReload=` as findings of its
  own.
- The propagation service and its drop-in,
  which the project's trigger answer installs.
  Both files are written by the project outside a round,
  applied with `systemctl daemon-reload`,
  and nothing else is run:
  the mount unit upholds the service (F23,
  F24),
  so no `systemctl start` is needed and none is printed.
  "The propagation service and its drop-in" gives the whole text of both and what systemd does after each way the
  service can end.
  A drop-in on a mount unit is read even where systemd learned the mount from its table (F24),
  which is this host's
  case for every dataset mount (F33).
  `doctor` reports the drop-in and the service,
  and whether the service's `BindsTo=` names the mount unit the checkout
  path escapes to.
- The record's directory.
  Nothing is installed for it:
  the handler makes `/run/git-worktree-clone-mount-propagation` and proves it root-owned
  on its first run,
  the way the readiness record's directory is made (F19),
  and `/run` empties at every boot.
- The propagation setting.
  It is not an installed fact and needs no `zfs set`,
  no mount option and no unit field:
  the step applies it at run
  time on the verified instance,
  and the record names that instance (F8).
- The checkout dataset and its container.
  This design creates none and sets nothing on any dataset;
  step 19's migration does,
  and that step's runbook carries
  its commands.

Where each act is recorded:
`doctor` prints the commands and reports what is installed,
the setup runbook holds what
is applied,
the unit's journal holds what each run did,
and the propagation record holds the last answer for the
instance it names.

### The propagation service and its drop-in

Two files,
and the whole text of each.
The unit's mount-unit name is derived from the resolved checkout path,
as `systemd-escape --suffix=mount` of that path (rule `HOM`);
`home-user-Monochromatic.mount` below is that name for this host's path,
written to show the shape,
not a name any machine copies.

```ini
# /etc/systemd/system/checkout-propagation.service
[Unit]
Description=Ask the boot unit to re-verify the checkout's private propagation
BindsTo=home-user-Monochromatic.mount
After=home-user-Monochromatic.mount

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/usr/bin/systemctl --no-block --no-ask-password reload-or-restart reconcile-git-worktree-clone-mounts.service
```

```ini
# /etc/systemd/system/home-user-Monochromatic.mount.d/10-upholds-propagation.conf
[Unit]
Upholds=checkout-propagation.service
```

There is no `[Install]` section and the unit is not enabled:
what starts it is the mount unit's `Upholds=`,
which submits it whenever the mount unit is up and the service is not (F23),
and an enabled unit would run at a boot whose checkout may not be mounted at all.

The pair is installed only on a boot unit that carries the `ExecReload=` line,
which the lost-event record accepts ([record][lost-record],
"Which trigger runs after the checkout becomes ready"),
because without that line the same request is a plain restart (F39),
and a restart of the boot unit runs `ExecStopPost=shutdown-unmount` and unmounts every attached clone.
Two checks keep that order:
the setup runbook writes the line with the boot unit it belongs to,
before the pair,
and `doctor` reads the installed boot unit's `ExecReload=` and reports the trigger as not installed
when it is absent.

`Type=oneshot` with `RemainAfterExit=yes` is what keeps a clean run from repeating:
the service exits after its request,
stays active because of `RemainAfterExit=`,
and the upheld queue starts nothing while the unit is active (F23);
`BindsTo=` stops it when the mount unit goes inactive,
so the next activation of the mount unit arms it again (F23,
F12).
Without that pair it would be started again after every clean exit,
which is what `Upholds=` means (F23).

What systemd does after each way the service can end,
read from source and not measured (L2):

- A clean run.
  `systemctl --no-block` queues the job and exits `0`,
  the unit stays active,
  and nothing runs it again while the mount unit stays up (F23).
  The request is not the step's outcome:
  `--no-block` returns once the job is queued,
  and the record and the journal are what say whether the step ran (F39).
- A refused step.
  This service does not see it,
  and what the refusal does depends on the path the boot unit was run by (F35):
  on the start path the unit fails and runs its `ExecStopPost=`;
  on the reload path it stays active with its reload result set and runs no stop pass.
  Nothing retries this service on that account,
  because it never failed.
- A failed request.
  Only a non-zero `systemctl` fails the service:
  no bus,
  a unit that is not loaded,
  or an enqueue refused as destructive against an irreversible job.
  A job that later waits on `-EAGAIN` does not.
  The failed unit is submitted to the upheld queue by its own transition (F23),
  throttled and not stopped by the two limits:
  the queue's 16 starts in 10 seconds,
  and the service's own start limit of 5 in 10 seconds (F23).
  Each retry is one `systemctl` call and no handler run,
  and a fixed window opens new attempts once 10 seconds have passed (F23).
- A job already queued on the boot unit and still waiting.
  The request merges into it,
  so it adds no second run while that job waits (F39).
- A boot unit that is not active.
  The request is a start,
  and a failed unit starts normally (F39).
  A deactivating unit's installed stop job is cancelled under the default mode,
  and the start waits for the unit to go inactive and then runs it;
  against an irreversible stop the enqueue is refused and `systemctl` answers non-zero (F39).
- A boot unit that is activating,
  which is the probe or `boot-reconcile` running.
  The request is absorbed:
  no second run starts (F39),
  so a replacement that lands after that run's step has passed is not repaired by this request.
  What repairs it is the project's first answer:
  the next run of the handler,
  started by any event,
  checks the checkout and sets it private before it attaches anything (F18).
  The window is the length of one boot reconcile or one long reload,
  and a sweep makes it as long as the number of clones reaches.
- A boot unit that is reloading.
  The reload is refused `-EAGAIN` and its job waits (F39),
  so a second reload follows the first rather than being lost.
- A reload while the boot unit is active.
  It runs the unit's `ExecReload=` (F39),
  which is the line this design and the lost-event record share.

Two things this recipe does not show and this design does not assume:
whether a `daemon-reload` alone starts it without a mount-unit activation (M4),
and whether an activated mount unit with this drop-in reaches the service within the same transaction as its own
activation (M4).

## Which build step builds which part

- Step 19,
  which this item blocks,
  builds:
  the `checkout_propagation` module (the descriptor reads,
  the verified set,
  the after read,
  the record's writer and reader);
  its call at the head of `boot-reconcile` and of the default sweep;
  the unit's `ExecReload=`;
  the extended refusal wording of `checkout_state`;
  the acceptance arm under "Measurements
  owed";
  and the SPEC section that states all of it,
  plus the propagation service and its drop-in,
  which the project's trigger answer installs.
  Step 8's code is where the call in the sweep lands,
  and the plan already gives that split:
  "The checkout's private
  propagation is checked here,
  and the unit that sets it is step 19's" ([plan][plan],
  step 8,
  "Open items").
- Step 10 builds the wrapper's half:
  the propagation check in the shared preflight,
  and the refusal in "What the
  wrapper and the caller see between the replacement and the re-run".
  Until it lands,
  L9 stands.
- Step 19 also builds the trigger the project chose:
  the drop-in and the propagation service.
  The content is step 19's,
  and the installation is the project's act.
  The record's provisional term means the same step keeps the root command working if the trigger is dropped
  (L12).
- Step 18 builds `doctor`'s reading of the record and its comparison with the live instance,
  including the descriptor read it takes to get the unique mount number and the fallback line for a refused read
  (F21,
  F37);
  where step 18 lands first,
  the line is its own,
  and step 19 adds the same line to "Doctor gains the migration and boot findings"
  ([plan][plan],
  step 19,
  "What it builds").
- Step 22 builds nothing here.
  This design ships no timeout and no cap,
  so no value waits on its measurements;
  M6 exists so that a later bound can
  be sized if one is ever proposed.
- Steps 9,
  14 to 17 and 20 to 23 are unaffected:
  this design changes no unmount,
  no removal,
  no sanitisation and no
  measurement they own.

## Choices this design marks for the project's veto

Each entry is a writer's choice,
adopted so that the design is whole,
and each gives the option taken,
the option it
replaced with a concrete example for both,
and the ranking with its reason.
A veto changes the sentence named and nothing else.

1.  Where the record lives.
    Taken:
    a new root-owned directory `/run/git-worktree-clone-mount-propagation` with one file `result`.
    Replaced:
    four more fields in the readiness record.
    Example for the taken option:
    after a replacement at 14:00,
    `/run/git-worktree-clone-mount-propagation/result`
    reads `outcome refused` and `step source`,
    and `doctor` prints both beside the live tag.
    Example for the replaced one:
    the same facts appear as `propagation-outcome` and `propagation-step` lines in
    `/run/git-worktree-clone-mount-readiness/result`,
    which `doctor` already reads.
    Advantage of the taken option:
    the taken option keeps step 8's record byte for byte,
    so a readiness parser and its tests are untouched;
    a refused propagation is recorded even for a boot whose readiness was never published.
    Advantage of the replaced one:
    one file to read and one place to look,
    at the price of a version bump,
    more fields and
    one more thing a readiness record means.
    Ranking:
    taken > replaced,
    because a record that a refusal must outlive the readiness gate cannot be the readiness
    record.
2.  The record's words.
    Taken:
    `kind checkout-propagation`,
    outcomes `applied`,
    `already-private`,
    `refused`,
    and the step words in "The
    propagation record".
    Replaced:
    reuse the readiness record's vocabulary and word the outcome as a state (`ready`,
    `not-ready`,
    `unknown`).
    Example for the taken option:
    `outcome refused` and `step namespace` tell a reader that the descriptor's mount had
    no namespace,
    which a readiness state could not say.
    Example for the replaced one:
    the same event reads `checkout not-ready`,
    which is true but says nothing about
    which check refused.
    Advantage of the taken option:
    the taken words name the act and the failing step;
    Advantage of the replaced one:
    one vocabulary across two
    records,
    and fewer words to learn.
    Ranking:
    taken > replaced,
    because the step words are what a build and a person act on.
3.  A receipt for a propagation refusal.
    Taken:
    none,
    keeping step 8's rule that a refusal files no receipt but an unavailable key's.
    Replaced:
    one more exception,
    so a propagation refusal files a `refused` receipt naming the step.
    Example for the taken option:
    a caller waiting through a replacement reaches its deadline report,
    which names the
    journal and the record.
    Example for the replaced one:
    the same caller reads a `refused` receipt naming `propagation-not-private` at once,
    and its command fails with that answer instead of a timeout.
    Advantage of the taken option:
    the taken option touches no built rule;
    Advantage of the replaced one:
    the caller learns at once,
    at the price
    of a rule change in `SPEC.md`,
    in the sweep and in its tests.
    Ranking:
    taken > replaced,
    because the caller already has a way to learn and the rule is step 8's.
4.  No path fallback for the set.
    Taken:
    the descriptor only,
    with a refusal when the descriptor's mount has no namespace.
    Replaced:
    the descriptor,
    and `mount(NULL, path, NULL, MS_PRIVATE, NULL)` when it is refused.
    Example for the taken option:
    the checkout is replaced between the read and the set;
    the set answers `EINVAL`,
    the
    record says `step set` and errno `22`,
    and nothing changes.
    Example for the replaced one:
    the same race;
    the fallback sets the new instance private by path and the record
    says `applied`,
    with nothing proving which instance it changed.
    Advantage of the taken option:
    the taken option never acts on an instance it did not verify;
    Advantage of the replaced one:
    a repair that
    succeeds in one more case.
    Ranking:
    taken > replaced,
    because the extra case is one where the fallback cannot say what it changed,
    and the
    staging design refuses the same window everywhere else.
    A replacement between the read and the set needs no hostile account,
    while the path a fallback would act on is the substitution window a hostile account needs;
    "What each check owes the threat model" states both once.
5.  Where the step sits.
    Taken:
    after the configuration and the checkout are resolved,
    before the start probe's record and the walk.
    Replaced:
    after `prepare`,
    before the sweep.
    Example for the taken option:
    a boot whose start probe record is missing;
    the step still sets the checkout private
    and records it,
    and the run then refuses on the record,
    so the checkout is private for whatever comes next.
    Example for the replaced one:
    the same boot;
    the run refuses on the record before the step,
    and the checkout waits
    for a later trigger.
    Advantage of the taken option:
    the taken option repairs what it can before refusing on something else;
    Advantage of the replaced one:
    the
    step runs only where a run is otherwise admissible,
    which is fewer root changes.
    Ranking:
    taken > replaced,
    because a repair that costs nothing should not wait on a record that may never appear.
6.  Both mount numbers in the record.
    Taken:
    `mount-id` and `mount-id-old` together,
    with the unique one deciding whether an instance is the same and the reused one only finding the table's row.
    Replaced:
    the unique number only.
    Example for the taken option:
    `doctor` prints the unique number it read from a descriptor beside the record's,
    and the reused number for what an operator's own tools show.
    Example for the replaced one:
    the same `doctor` line cannot say which row of `/proc/self/mountinfo` is the instance the record names.
    Advantage of the taken option:
    the record speaks to both readers,
    the one with a descriptor and the one with the table;
    Advantage of the replaced one:
    one field
    fewer and no reused number to explain.
    Ranking:
    taken > replaced,
    because a reader that has only the table can still find the row,
    while the proof of the instance stays with the number the kernel never reuses (F37).

## Dependencies on sibling designs

The sibling documents were read with `git show <branch>:<path>` at the tips below.
Each is an unapproved proposal,
and none of its answers is treated as decided here.

- "Which triggers a lost last event waits for"
  (`docs/planning-cli-git-zfs-lost-last-event-triggers-design`,
  tip `88f6a15194`,
  2026-11-22T19:34:54-05:00).
  One mechanism,
  and each document owns half of it:

  - The triggers are its subject,
    and this design cites it for all four:
    the provider hook,
    the way a trigger asks the boot unit to act,
    the event filter and its scope,
    and the root command the project runs.
    Its hook is two files:
    a `zfs-mount.service` drop-in whose `ExecStartPost=` is prefixed `-` and asks the boot unit to act,
    and the boot unit's `ExecReload=` ("The hook,
    in one form a sibling can cite"),
    so a refusal inside it never fails the provider and shows in the boot unit's journal.
    One verb therefore has three senders at its tip:
    this design's propagation service,
    that hook,
    and the project's printed lever (its "The shape the project chose").
    Its root command is `systemctl --no-ask-password reload-or-restart
    reconcile-git-worktree-clone-mounts.service` ("T5:
    a trigger the project fires"),
    which this design prints unchanged in the wrapper's refusal and in `doctor`,
    with its one condition,
    the boot unit's `ExecReload=`,
    which the lost-event record accepts.
  - The step and its order are this design's.
    The sibling cites "Where it runs,
    as whom,
    and in what order" and states no order of its own:
    configuration and checkout resolution,
    the propagation step,
    the probe record,
    the pool and the checkout checks,
    the sweep ("Order inside a trigger").
    A pass its rate rule holds has therefore already run the step,
    which the sibling says in as many words,
    so this design needs no exception for a held pass.
  - The propagation service and its drop-in is the trigger the project chose,
    and its `ExecStart=` is the same job request the sibling's hook uses,
    so both documents reach `boot-reconcile` one way and no second reconcile path exists.
    The sibling's Q-A option C rejects a companion unit on the checkout's behalf only in the shape that needs a
    generated `.mount` and exclusion from `zfs mount -a`;
    this design's drop-in needs neither (F24,
    F33),
    so the two are about different shapes and M4 measures which one this host gives.
  - The event filter's scope is one question,
    asked in the sibling's terms as its V2.
    This design states only that the filter does not gain the checkout's dataset (F31,
    and the repair record's "Which triggers the route adds").
  - Its A9 calls a `systemctl` request from a unit's start unprobed,
    which is this design's M4 too,
    and its D1 lists four sentences of this design that assumed its earlier direct hook;
    this round rewrites all four (A3,
    this section's hook bullet,
    the printed-command paragraph and L11).
  - The sibling's citations of this design are pointers by commit and accurate at the commits they name:
    its D1 reads this document at `30274435f3` (its lines 3728 to 3760),
    which this round moves.
    Two of the things it reads there have changed kind,
    not just place:
    the `ExecReload=` line is no longer this design's veto choice 3 but a decided fact the lost-event record accepts,
    and this design's question 2 is gone as a question,
    so what its D1 calls "its question 2 option 1" is now the
    trigger the project chose ("The trigger the route adds,
    and what it must show").
    A later agreement check should read this document at this round's tip.
- The eight other siblings
  (`docs/planning-cli-git-zfs-add-time-git-lock-design`,
  `docs/planning-cli-git-zfs-add-binding-record-design`,
  `docs/planning-cli-git-zfs-removal-destruction-plan-record-design`,
  `docs/planning-cli-git-zfs-origin-cleanup-record-design`,
  `docs/planning-cli-git-zfs-dirty-main-restore-on-rung-1-design`,
  `docs/planning-cli-git-zfs-deferred-destroy-checks-design`,
  `docs/planning-cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design`,
  `docs/planning-cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design`):
  no dependency.
  Each mentions this item only as out of its own scope or not interacting with it,
  and two of them cite this
  document as the owner of "What re-applies private propagation after a replacement mount"
  (`cli-git-zfs-origin-cleanup-record-design` line 2667 and
  `cli-git-zfs-deferred-destroy-checks-design` line 2187).
  This design reads no record they place and writes none they read.
- The add-time lock's design says it has no interaction with this item (its line 2155);
  this design adds one check to
  the add's shared preflight and nothing else,
  so the two can be reconciled by whoever writes the later one.

## What this design does not cover

- It does not measure anything.
  Every kernel,
  ZFS and systemd claim is a source reading,
  and "Measurements owed" says
  what would change each reading.
- It does not design any trigger:
  the provider hook,
  the way a trigger asks the boot unit to act,
  the event filter and its scope,
  and the root command the project runs are the sibling design's,
  and this design cites it for each.
- It does not design the checkout's boot mount protection,
  the immutable entry,
  or the migration that creates the
  checkout dataset and its container;
  step 19 builds those and the proposal's sections decide them.
- It does not design the unmount of a staged mount,
  the removal's holder scan or the forced arm.
  It states what a
  detached clone is (F10,
  F11) and leaves its repair to those designs.
- It does not change the readiness record,
  the receipts or the sweep's receipt rule.
- It proposes no timer,
  no daemon,
  no executable and no configuration key,
  and no new unit except the propagation
  service the project's trigger answer installs;
  it proposes no timeout or cap.

## Sources read

- Linux `v7.3`:
  `fs/namespace.c`,
  `fs/pnode.c`,
  `fs/mount.h`,
  `fs/proc_namespace.c`,
  `include/uapi/linux/mount.h`,
  `include/uapi/linux/fanotify.h`,
  `include/linux/fsnotify.h`.
- OpenZFS `zfs-2.4.4`:
  `etc/systemd/system-generators/zfs-mount-generator.c`,
  `etc/systemd/system/zfs-mount.service.in`,
  `include/sys/sysevent/eventdefs.h`,
  `module/os/linux/zfs/zfs_vfsops.c`,
  `module/os/linux/zfs/zpl_super.c`,
  `module/zfs/dsl_dataset.c`,
  `module/zfs/zfs_ioctl.c`,
  `module/zfs/spa_history.c`,
  `lib/libzfs/libzfs_dataset.c`,
  `lib/libzfs/libzfs_changelist.c`,
  `cmd/zfs/zfs_main.c`.
- systemd `v262`:
  `src/shared/mount-setup.c`,
  `src/shared/bus-polkit.c`,
  `src/core/mount.c`,
  `src/core/unit.c`,
  `src/core/service.c`,
  `src/core/transaction.c`,
  `src/core/unit-dependency-atom.c`,
  `src/core/manager.c`,
  `src/core/job.c`,
  `src/core/dbus-util.c`,
  `src/systemctl/systemctl-util.c`,
  `man/systemd.mount.xml`,
  `man/systemd.unit.xml`,
  `man/systemd.service.xml`,
  `man/systemd.path.xml`,
  `man/systemctl.xml`.
- This repository:
  `doc/decision/cli-git-zfs-worktree-design.md`,
  `doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
  `doc/decision/cli-git-zfs-slot-identity-store-format.md`,
  `doc/decision/cli-git-zfs-staging-under-cache.md`,
  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
  `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
  `doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
  `doc/planning/cli-git-zfs-staging-under-cache-design.md`,
  `doc/planning/cli-git-zfs-start-probe-design.md`,
  `package/git-policy/cli/SPEC.md`,
  the handler and protocol modules named in the facts,
  and the start probe's mount
  calls.
- The siblings named under "Dependencies on sibling designs",
  read with `git show` at the tips given there.

## Revisions

Each commit of this document,
with its committer time from `git log --format=%cI`:

- `d5c95e3a6a`,
  2026-11-22T17:28:38-05:00:
  the first complete design.
- `c59e84088d`,
  2026-11-22T17:29:28-05:00:
  what must be applied to the machine,
  and what an unprivileged request to
  the system manager answers.
- `fecdc987b1`,
  2026-11-22T17:30:29-05:00:
  the record may carry an errno from any kernel call.
- `8609098715`,
  2026-11-22T18:17:49-05:00:
  the gate review's findings,
  the corrected
  citations throughout,
  the triggers survey,
  both mount numbers,
  the propagation set,
  the no-file resolution,
  the threat-model list,
  the acceptance arm,
  the expanded veto and question lists,
  the sibling reconciliation,
  and this section.
- `9a5ca2e6cf`,
  2026-11-22T18:18:06-05:00:
  this list gained its own commit's hash and time.
- `5e6a2ff9dc`,
  2026-11-22T18:18:14-05:00:
  the list gained the commit that recorded it.
- `43aba4b52c`,
  2026-11-22T18:49:30-05:00:
  the second gate review's findings:
  the propagation service gains `Type=oneshot` and `RemainAfterExit=yes`,
  asks the boot unit to act with the sibling's job request,
  and has its whole text and each of its exits stated from source;
  the job-driven examples name only units systemd can start,
  with the start refusal of a table-learned mount unit as a new fact;
  the headline answer and A4 are rewritten;
  the stale-instance comparison uses the unique mount number;
  the trigger question becomes an order of adding them;
  the cross-references,
  the record's set bound and the review labels are corrected;
  and the sibling section is rewritten against its tip `2c860b3930`.
- `6ce5636f51`,
  2026-11-22T18:49:43-05:00:
  this list gained the commit that recorded this round.
- `ba096f5efd`,
  2026-11-22T18:49:51-05:00:
  the list gained the commit that recorded itself.
- `12e3760710`,
  2026-11-22T18:50:13-05:00:
  the propagation set's real combinations,
  and the boot unit's inactive state.
- `01c89c326f`,
  2026-11-22T19:55:28-05:00,
  on the merge of `main` at `9dfb749815`:
  the project's four answers are written in,
  with the two decision records quoted;
  the questions section becomes "The project's four answers,
  and what this design does with each";
  the trigger section keeps the one provisional trigger and the root command,
  with the kill criterion and the three limits;
  the filter option and its ranking are removed and the record's refusal quoted;
  the boot unit's `ExecReload=` line becomes decided,
  with L11 kept as a short note for an older install;
  the unit's `ExecReload=`-less case is guarded by the install order and the `doctor` check;
  the states the request can meet are corrected and completed,
  with the activating and reloading rows and what repairs the missed case;
  a refusal is reported by the path it came by,
  with `ReloadResult` added to where it is read;
  the three wrong measurement references and the last review label are corrected;
  the third rate limit,
  the man-page line,
  the `doctor` namespace note and the `statmount` callers are fixed;
  and every sibling citation is re-pointed at `88f6a15194`.

## Links

[proposal]: cli-git-zfs-worktree-design-proposal.md
[plan]: cli-git-zfs-worktree-implementation-plan.md
[measurements]: cli-git-zfs-measurements-owed-before-acceptance.md
[spec]: ../../package/git-policy/cli/SPEC.md
[sweep-choices]: ../decision/cli-git-zfs-step-8-sweep-choices.md
[slot-store]: ../decision/cli-git-zfs-slot-identity-store-format.md
[staging-decision]: ../decision/cli-git-zfs-staging-under-cache.md
[repair-record]: ../decision/cli-git-zfs-checkout-propagation-repair.md
[lost-record]: ../decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md
