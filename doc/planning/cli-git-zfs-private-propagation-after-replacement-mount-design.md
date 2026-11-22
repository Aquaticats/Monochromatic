# Private propagation after a replacement mount of the checkout

Non-final,
unapproved design proposal by an agent.
It is not approved by the project,
and nothing in it is
final.
It designs the open item "What re-applies private propagation after a replacement mount" of the accepted ZFS
worktree design,
and it makes no decision that design or the project's answers did not make beyond the writer's
choices and the questions "Choices this design marks for the project's veto" and "Questions only the project can
answer" list.
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
`git --version`;
and text searches over the repository.

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
"What re-applies
private propagation after a replacement mount:
step 19,
blocking for its acceptance item,
writer design work."

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
Nothing new fires on a mount:
the re-application does not wait for an
event,
because F12 to F14 show that the events available for a mount made outside systemd either do not exist or do
not start a unit.

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
  with ZED's delivery "installed but not controlled".
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
  lines 221 to 237):
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
  `include/uapi/linux/mount.h:161` to `:167`),
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
  Changing a mount's propagation needs the mount to be in some namespace and the caller to be administrative in
  that namespace's user namespace:
  `may_change_propagation` answers `EINVAL` when the mount has none and `EPERM`
  without `CAP_SYS_ADMIN` (Linux `v7.3`,
  `fs/namespace.c:2943` to `:2954`).
  Root in the host namespace satisfies it;
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
  `include/uapi/linux/mount.h:157` to `:186`),
  `STATMOUNT_BY_FD` asks for "mountinfo for given fd"
  (`:243` to `:245`),
  and `do_statmount` reads `mnt_file->f_path.mnt`,
  drops `STATMOUNT_MNT_POINT` and
  `STATMOUNT_MNT_NS_ID` when the mount has no namespace,
  and answers `ENOENT` for a mount id it cannot find
  (`fs/namespace.c:5908` to `:5928`).
- F9.
  An ordinary unmount refuses while the mount has children or holders:
  `do_umount` answers `EBUSY` unless
  `propagate_mount_busy(mnt, 2)` is clear (Linux `v7.3`,
  `fs/namespace.c:2004` to `:2012`;
  `fs/pnode.c:352` to
  `:355`,
  `:407` to `:421`).
  A checkout with a clone mounted under it is such a mount.
- F10.
  A lazy unmount detaches the whole subtree and makes it private on the way out:
  `do_umount` with `MNT_DETACH`
  calls `umount_tree(mnt, UMOUNT_PROPAGATE)` without the busy check (`fs/namespace.c:2004` to `:2008`),
  `umount_tree` walks every mount under the one named and clears each namespace link (`:1831` to `:1873`),
  and
  `bulk_make_private` clears the detaching mounts' master relationships (`fs/pnode.c:162` to `:175`).
- F11.
  A second mount of one ZFS dataset is refused `EBUSY` by the dataset's own long hold:
  `zfsvfs_create` owns the
  objset through `dmu_objset_own` (OpenZFS `zfs-2.4.4`,
  `module/os/linux/zfs/zfs_vfsops.c:651` to `:669`),
  the own
  path returns `SET_ERROR(EBUSY)` when `dsl_dataset_tryown` finds an owner (`module/zfs/dsl_dataset.c:896` to `:918`),
  and `dsl_dataset_tryown` takes the long hold only when no owner stands (`:1049` to `:1062`).
  A clone whose old mount
  was detached but not released is therefore still owned,
  and a new attach of it fails.
- F12.
  systemd mirrors mounts made outside it:
  "Mount points created at runtime (independently of unit files or
  `/etc/fstab`) will be monitored by systemd and appear like any other mount unit" (systemd `v262`,
  `man/systemd.mount.xml:76` to `:81`).
  A mount that disappears puts its unit into the dead state,
  and a mount that
  appears puts it into the mounted state with a fresh invocation id,
  both without a job
  (`src/core/mount.c:2242` to `:2271`).
- F13.
  No dependency brings a unit back when a mount unit becomes active without a job.
  Stop and restart propagate to
  `PartOf=` units only while systemd adds a stop or restart job (`man/systemd.unit.xml:761` to `:775`;
  `src/core/transaction.c:1197` to `:1206`),
  and the retroactive start that runs on an unexpected activation covers
  only the activating unit's own `Requires=` and `BindsTo=` dependencies,
  and only those without `After=`
  (`src/core/unit.c:2313` to `:2331`,
  called at `:2805` to `:2815`).
  An external mount or unmount starts nothing.
- F14.
  A path unit cannot see a mount appear at a path it watches,
  and it is rate-limited:
  path units watch with
  inotify (`man/systemd.path.xml:48` to `:52`),
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
  The checkout dataset gets a mount unit only where the ZFS generator runs:
  the generator writes one
  `<escaped mountpoint>.mount` per dataset with `canmount=on` and a path mountpoint,
  with `Before=zfs-mount.service`,
  `After=zfs-import.target` and the key-load unit,
  and a `[Mount]` section of `Where=`,
  `What=`,
  `Type=zfs`
  (OpenZFS `zfs-2.4.4`,
  `etc/systemd/system-generators/zfs-mount-generator.c:616` to `:676`),
  and it reads its
  dataset list from the cache file.
  On this host `/etc/zfs/zfs-list.cache` does not exist and
  `/run/systemd/generator` holds one mount unit,
  `boot.mount`,
  so no `home-user-Monochromatic.mount` exists here
  today.
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
  `include/sys/sysevent/eventdefs.h:95` to `:133`),
  and the
  mount path posts no ereport (`module/os/linux/zfs/zfs_vfsops.c`,
  searched for `zfs_ereport_post`).
  The proposal
  already relies on the same list for its boot and shutdown argument ([proposal][proposal],
  "Boot and shutdown unit",
  lines 488 to 498).
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
  lines 6185 to 6210).
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
  item 3).
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
  A signal to another account's process is refused `EPERM`:
  `check_kill_permission` answers `-EPERM` unless the caller and the target share credentials
  (`kill_ok_by_cred`) or the signal is `SIGCONT` within one session
  (Linux `v7.3`,
  `kernel/signal.c:805` to `:837`).
  Nothing in this design signals a process,
  and this is the answer a way forward that named a signal would have to give.

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
  The sibling design "Which triggers a lost last event waits for" keeps the three triggers the proposal names and
  does not move the provider hook off the reconciliation mode.
  This design's record and its set are written by either
  entry point,
  so the assumption only bears on readiness republication,
  which "Dependencies on sibling designs"
  states.
- A4.
  The build may add a mode or a step to the installed handler binary and a field list to a root-owned record,
  and
  the release still ships the two executables `SPEC.md` names ([`SPEC.md`][spec],
  lines 250 to 280).
  Nothing here
  needs a third executable or a new unit.
- A5.
  `mount_setattr` and `statmount` are available to the release's targets.
  `mount_setattr` is already called by the
  built start probe with `MS_PRIVATE` and a `mount(2)` fallback
  (`src/native/start_probe/probe_mount_calls.rs:30` to `:60`),
  and `statmount` is not called anywhere in the
  repository today,
  so its use here is new code on a kernel interface the ABI gate does not yet cover.

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
  which of F12 to F14 fires for which way a mount is
  replaced has not been observed on this machine or any guest.
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
  under a shared parent and are propagated to its peers (F3).
  The propagation step cannot prevent that;
  it can only shorten
  the window to the next trigger,
  and no trigger exists for a mount made outside systemd (F13,
  F17).
- L5.
  The mount-unit event path is unavailable on this host today:
  no per-dataset mount unit is generated here (F15),
  so `systemd` knows the checkout only from `mountinfo` (F12) and no unit of ours is attached to it.
  Any design that
  trusted that event would be wrong here,
  which is why none is built on it.
- L6.
  The step changes the checked mount's peer group and nothing else.
  Whether that is enough for the staging move,
  the rollback and the all-namespace scan is measured elsewhere (F20,
  and the staging design's arms),
  not here.
- L7.
  F11 makes a detached clone un-attachable until its old mount is released.
  This design does not release it,
  does
  not find its holders,
  and does not end them;
  the removal's holder rules and step 9's unmount do
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

## What a replacement mount is

### The ways a checkout mount instance is replaced

A mount instance is the kernel's mount object:
it carries the propagation,
the peer group and the namespace link,
and
a replacement is any act that leaves a different object at the checkout path.
Each of the six ways says what the new
instance's propagation is and what becomes of the clones attached under the old one.

1.  An unmount and a mount,
    by `systemctl restart home-user-Monochromatic.mount`,
    by `zfs unmount` then `zfs mount`,
    or
    by `umount` then `mount`.
    An ordinary unmount refuses while a clone is mounted under the checkout (F9),
    so the
    route's own stop pass runs first,
    and an unmount of the checkout with clones attached is refused `EBUSY` by
    the kernel,
    not by this route.
    Once the old instance is gone and a new one is mounted,
    F3 makes the new one shared
    and its children,
    mounted later,
    shared with it.
    Clones attached under the old instance do not survive it,
    because
    the unmount could not happen while they were attached.
2.  A lazy unmount and a mount,
    `umount -l` then a mount.
    `umount -l` does not refuse the clones:
    it detaches the
    whole subtree,
    clones included,
    from the namespace and makes the detached mounts private (F10).
    The clones' ZFS
    datasets stay owned (F11),
    so a later attach of one fails `EBUSY`;
    the new instance at the checkout path has no
    children,
    is shared by F3,
    and the F18 check refuses every clone until the detached mounts are released.
    The
    handler can still read its own descriptor's mount and see it has no namespace (F8),
    which is how it tells a
    detached instance from one nobody opened.
3.  A mount after a late key load.
    The checkout's dataset was not mounted at boot,
    its key is loaded later,
    and the
    key-load unit or the mount unit then mounts it (F15).
    The boot unit's run either happened before the mount and
    refused the checkout as not ready,
    or has not happened yet;
    either way the new instance is shared by F3,
    and the
    provider hook is the trigger the design names for this way.
4.  A rollback.
    `zfs rollback` needs the dataset unmounted,
    so this way is the first one plus one more fact:
    a rollback
    to a snapshot taken before the immutable flag removes the underlay's protection,
    which the design already says must
    be verified before automated attach ([proposal][proposal],
    "One-time move and checkout boot protection",
    lines
    285 to 300;
    [measurements][measurements],
    item 8).
    The propagation answer is the first way's.
5.  A rename.
    Renaming the dataset's mountpoint,
    or replacing the directory at the checkout path,
    needs the dataset
    unmounted,
    so the propagation answer is the first way's.
    A rename of the dataset that keeps the mountpoint is not a
    replacement:
    the mount object stays and its propagation with it.
    `mount -o remount` is not a replacement either:
    `do_mount` sends `MS_REMOUNT` to the existing mount and its propagation is untouched (Linux `v7.3`,
    `fs/namespace.c:4317` to `:4321`).
    A remount leaves the propagation alone,
    and `doctor`
    prints the live tag so the expectation is checked rather than assumed.
6.  An act outside a job that mounts the dataset:
    `zfs mount -a` from `zfs-mount.service`
    (F16),
    a manual `zfs mount <dataset>`,
    a reimport followed by `zfs mount -a`.
    systemd notices (F12) and starts
    nothing (F13),
    ZFS posts no event (F17),
    and the new instance is shared by F3.
    This is the way with no trigger at
    all,
    and the one this design is shaped by.

### Which events exist, and what each covers

Read from source,
not observed (L2).
The ways are the six in "The ways a checkout mount instance is replaced";
"covers" means "starts something that could re-apply
the setting".

- The mount unit's state change.
  systemd puts the unit dead on an external unmount and mounted,
  with a fresh invocation
  id,
  on an external mount (F12).
  It is a state change and not a job,
  so nothing is started by it:
  `PartOf=` needs a
  job (F13),
  the retroactive start covers the activating unit's own dependencies and not its dependents (F13),
  and a
  `WantedBy=`/`RequiredBy=` edge is only walked when the mount is started as a job.
  Covers ways 1 and 3 where
  systemd runs the mount,
  and way 6 only if some other unit asks systemd to start the mount unit,
  which
  `zfs mount -a` does not.
  Unavailable on this host in any case,
  because no such unit exists (F15).
- A unit bound to that mount.
  `Requires=` plus `After=` stops the bound unit when the mount unit is stopped by a job
  and orders it when the mount is started by a job;
  it does not restart it on a replacement (F13).
  `BindsTo=` adds the
  refusal to stay active without the mount and the retroactive start of the mount's dependencies,
  not of itself (F13).
  Covers ways 1 and 3 when systemd does the mount,
  and nothing else.
- A path unit.
  It watches with inotify (F14);
  a mountpoint keeps existing as a directory whether or not a mount covers
  it,
  so `PathExists=` and `PathChanged=` do not distinguish the two states,
  and a failed trigger limit stops it
  watching altogether (F14).
  It could be pointed at `/proc/self/mountinfo`,
  which is a procfs file whose changes inotify
  does not report,
  and it would fire on every mount in the host namespace rather than on the checkout's.
  Covers
  nothing reliably,
  and is refused.
- A ZED event.
  The event list has no mount or unmount class (F17),
  and the design already says ZED's delivery is
  "installed but not controlled" ([proposal][proposal],
  lines 1393 to 1400).
  Covers nothing for a mount.
  The nearest
  events,
  `pool_import` and `pool_export`,
  bracket a reimport,
  not the mount that follows it.
- The readiness hook.
  The checkout provider must trigger reconciliation after post-key-load checkout readiness
  ([proposal][proposal],
  "Boot and shutdown unit",
  lines 540 to 551).
  It covers ways 3 and 6 as far as they are
  "the provider just mounted the checkout",
  which is exactly when it fires,
  and it is the one trigger designed for a
  late mount.
  Its own design is a sibling item.
- The wrapper's next command and the sweep nudge it writes.
  Covers whatever the caller does next,
  after a refusal that
  this design adds (F18,
  and the section on the wrapper).

The conclusion the design draws:
there is no event to hang the re-application on that exists for every way,
so the
design does not use one.
It puts the step where every way must pass anyway,
which is the next run of the root handler.

### What the clones under the old instance become

Three outcomes,
and each needs saying because the sweep's answer differs:

- Attached and released:
  the unmount was refused (F9) until a stop pass unmounted them,
  so they are gone and their
  datasets are free.
  The sweep attaches them again on intent.
- Detached with the checkout:
  a lazy unmount took them with it (F10).
  Their datasets stay owned (F11),
  a new attach
  fails `EBUSY`,
  and nothing in the wrapper's namespace reaches them:
  the sweep reports the `EBUSY` per clone,
  `doctor` reports the clone as unmounted with its intent standing,
  and the removal's holder scan is where they are
  looked for ([proposal][proposal],
  "Mounts held by other namespaces").
  This design adds no new repair for that state.
- Never attached:
  nothing to do.

Which outcome each of the six ways gives,
in the same order as "The ways a checkout mount instance is replaced":

1.  An unmount and a mount:
    the clones were released first or the unmount was refused (F9),
    so they are the first outcome.
2.  A lazy unmount and a mount:
    the second outcome,
    and the new instance is the one a later attach has to work from once they are released.
3.  A mount after a late key load:
    there was no instance for a clone to be attached under,
    so the third outcome,
    and the boot unit's earlier run had already refused the checkout as not ready.
4.  A rollback:
    the first outcome,
    because the rollback's own unmount needed the clones gone,
    and the underlay's immutable entry is what a rollback to an older snapshot can remove
    ([measurements][measurements],
    item 8).
5.  A rename:
    the first outcome for a rename that needs an unmount;
    for a rename that keeps the mountpoint,
    the instance is the same one and the clones under it are untouched.
6.  An act outside a job:
    the clones were released or the mount refused (F9)
    or,
    where that act detaches lazily,
    the second outcome.

## The step that re-applies the setting

### Where it runs, as whom, and in what order

The step is a new module of the installed handler,
`checkout_propagation`,
called as the first act of the two entry
points that decide about attaches,
before either one enumerates anything:

- `boot-reconcile`,
  from `ExecStart` of `reconcile-git-worktree-clone-mounts.service`,
  as root,
  in the host mount
  namespace ([proposal][proposal],
  "Boot and shutdown unit",
  lines 499 to 520).
  Its position is after the
  configuration is read,
  because the checkout path and dataset come from it,
  and before `prepare`,
  which reads the
  start probe's record and the checkout's walk.
- the default invocation,
  which is what the installed ZED event starts and what a caller's nudge starts,
  as root.
  Its
  position is the same:
  after the configuration,
  before the probe record and the walk.

Neither `shutdown-unmount`,
`start-probe` nor `adopt-worktree-parent` calls it:
the first unmounts and changes no
durable state,
the second already runs in a private namespace of its own and changes propagation there
(`src/native/start_probe/probe_mount_calls.rs:30` to `:60`),
and the third records identities and mounts nothing.

The order in each of the two entry points becomes:
read configuration;
**the propagation step**;
the start probe's
record and the boot's facts;
the pool's state;
publish readiness `running`;
the sweep;
publish readiness `finished` or
`failed`.
A refusal in the step publishes no readiness and enumerates nothing,
which is the design's existing sentence
for a failed prerequisite:
"A failed prerequisite authorizes no attachment" ([proposal][proposal],
"Boot and shutdown
unit",
line 523).

### What it verifies before

The step is about one instance,
and it proves which instance before it touches it:

1.  Open the checkout path `O_PATH | O_DIRECTORY | O_CLOEXEC`.
    The path is the configured one
    (`cloneMounts.checkout`,
    `src/native/worktree_clone_mount_handler/sweep_configuration.rs:63`),
    which the sweep
    already verifies as an absolute path on the configured dataset.
2.  Read the descriptor's mount with `statmount(STATMOUNT_BY_FD)` (F8):
    `mnt_id`,
    `mnt_ns_id`,
    `mnt_propagation`,
    `mnt_peer_group`,
    `mnt_master`,
    `mnt_point`,
    `fs_type`,
    `sb_source`.
3.  Refuse unless all of these hold,
    and name the one that failed:
    the point is the checkout path;
    the type is `zfs`;
    the source is the configured checkout dataset (`cloneMounts.checkoutDataset`,
    `:67`,
    or the design name that step 8
    falls back to,
    `sweep_checkout_mount.rs:40` to `:44`);
    the mount has a namespace id,
    so it is not a detached
    instance (F8,
    F10);
    and the propagation is one of the four words.
4.  Read the live table through the repository's own parser as the sweep does (F18),
    and refuse when the two reads
    disagree about the mount id:
    that is a replacement between the two,
    and nothing is set on either instance.

Everything here is a read.
The refusal that follows the reads leaves the mount as it was,
and the account never has
the authority the change needs (F7).

### The call that sets it

`mount_setattr(fd, "", AT_EMPTY_PATH, &mount_attr{ propagation: MS_PRIVATE }, size_of::<mount_attr>())`,
with the
descriptor from the verify read,
never with a path (F5,
F6).
This is the repository's own idiom for the same change:
the start probe already sends `mount_setattr` with `propagation: MS_PRIVATE` and falls back to
`mount(NULL, path, NULL, MS_REC|MS_PRIVATE, NULL)`,
both inside a namespace of its own
(`src/native/start_probe/probe_mount_calls.rs:30` to `:60`).
The step has no path fallback:
a path fallback is the
substitution window the staging design refuses everywhere else,
and a descriptor that is refused `EINVAL` for having
no namespace (F6) is a fact the record should carry,
not something to route around.

No `AT_RECURSIVE`,
and the reason is written beside the call:
recursion would make every mount under the checkout
private too,
including one the route did not make,
and the design's sentence is "Do not silently change propagation"
([proposal][proposal],
line 1300).
Whether the project agrees is question 4.

The step acts only when the propagation is `shared`,
because that is the measured disqualifier (F20,
and
[measurements][measurements],
item 3).
A `slave` or `unbindable` mount carries no `shared:` tag and is therefore
admitted by the built check today (F18);
this design keeps that and reports the word.
Whether it should refuse is
question 3.

### What it verifies after

The same descriptor,
read again with `statmount(STATMOUNT_BY_FD)`,
and all three must hold:
the `mnt_id` is the one
the before read gave,
so the instance is the same;
`mnt_propagation` is `private`,
with `mnt_peer_group` and
`mnt_master` clear;
and `mnt_ns_id` is still present,
so the instance was not detached under the call.
Anything else
is a refusal,
and the record says which one.

Then,
and separately,
the sweep's own live check re-reads the checkout from the mount table (F18) before the first
clone is touched.
That second check is the decision's "Recheck it regardless of who sets it" made mechanical:
the step
proves what it set,
and the sweep proves what it is about to attach under,
and a replacement in between is caught by
the second read and refused.

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
and a reader
sees one complete record (rule `NLK`).
"The propagation record" states its fields and their bounds.

## What the wrapper and the caller see between the replacement and the re-run

The wrapper has no authority here (F7),
so its part is to say the truth early and to say what root must do.
Step 10's
preflight already lists "mount propagation" among the checks it shares with `doctor`
([proposal][proposal],
"Add and Git registration",
step 1,
lines 365 to 375),
and this design fills that check in with
the repository's own reader (F21),
which the account can call.
The refusal is printed before any claim,
snapshot,
clone
or registration is written,
so nothing managed is left behind:

> `/home/user/Monochromatic` is mounted with the `shared` propagation tag,
> so a clone mounted under it would be
> delivered to another mount namespace,
> and this wrapper does not change propagation because it has no authority over
> mounts.
> Nothing was claimed,
> snapshotted,
> cloned or registered.
> Ways forward:
> as root,
> `systemctl reload reconcile-git-worktree-clone-mounts.service` re-runs the handler's checks and its sweep,
> and then
> this command can be run again;
> `monochromatic-git-policy-cli doctor` prints the live tag and the handler's last
> propagation record;
> the next ZED event,
> the checkout provider's post-key-load hook or another caller's sweep nudge
> runs the same step.

The command the refusal prints is one this wrapper never runs and never waits on.
A request to change or signal a system unit is answered by the caller session's authorization agent,
and the command
waits while that agent asks (F22),
which is why this route prints the act instead of performing it:
nothing here asks for a credential,
and nothing here starts a dialog.
A script that must not wait adds `--no-ask-password`,
which turns the wait into a plain refusal (F22).

`systemctl reload` is named rather than `start` because the unit is `Type=oneshot` with `RemainAfterExit=yes`,
so it is
active after its first run and `start` on an active unit does nothing (systemd `v262`,
`man/systemd.service.xml:346` to
`:353`;
`src/core/service.c:3765` to `:3786` admits a reload in the exited-with-remainder state).
The design adds
`ExecReload=` for exactly this (see "What becomes of `Type=oneshot` with `RemainAfterExit=yes`").

A caller already waiting when a replacement happens has written its intent and its nudge;
the sweep that answers it
runs the step first,
and either attaches or refuses.
A refusal files no receipt,
which is step 8's built rule ([`SPEC.md`][spec],
lines 6364 to 6369),
and the
caller's own deadline report already says that a deadline is not cancellation and names the journal
([`SPEC.md`][spec],
lines 6409 to 6416).
This design adds no receipt for a propagation refusal and does not change
that rule.

`doctor` is the account's read-only view:
it prints the live `shared:`/`slave:`/`private`/`unbindable` word for the
checkout's mount,
the record's boot,
mount id,
outcome and failing step,
and whether the record's mount id is the live
one.
When they differ it says the record names an instance that is no longer the checkout's,
and prints the same
root command.

## What a failure reports, and where it is read

Four places,
each with a different reader,
and none of them a prompt:

- The handler's summary line,
  which an operator and a guest arm read:
  `applied`,
  `already-private` or
  `refused` for the checkout,
  with the mount id before and after,
  the failing step and the errno on a refusal.
- The handler's journal,
  under the unit's name,
  which the design already names in its refusals
  ([`SPEC.md`][spec],
  lines 6211 to 6212,
  and the refusal texts in `clone_mount_readiness_admission.rs:100`).
  The step logs the same
  fields the record carries,
  tagged `checkoutPropagation`.
- The record,
  `/run/git-worktree-clone-mount-propagation/result`,
  which `doctor` reads and which any account can read
  without root.
  It is a cache:
  F18's live check is what admits an attach,
  not the record.
- The readiness record and the sweep's refusal,
  which are unchanged in shape:
  a shared checkout is `not-ready` (F18),
  readiness is not published for that run,
  and any attach is refused with the words the build already has,
  extended by
  the step to name the record and the root command.

Nothing is reported anywhere but those four places,
and nothing asks for a credential or a password:
the wrapper's
refusal names the command root runs and prompts for nothing ([`SPEC.md`][spec],
lines 7345 to 7352).

## What becomes of `Type=oneshot` with `RemainAfterExit=yes`

It stays.
Four reasons,
and one addition:

- The unit's remaining duty is the shutdown half,
  and that needs a unit that was started and stays active so that its `ExecStopPost` runs at a stop:
  a failed start already runs it,
  "execution continues with commands specified in `ExecStopPost=`" (systemd `v262`,
  `man/systemd.service.xml:459` to `:462`),
  and `RemainAfterExit=` is what keeps a oneshot service active after its process exits
  (`:346` to `:353`).
  The design's `ExecStopPost` passes are unaffected.
- Nothing needs it to fire on a later mount,
  because the setting no longer depends on a unit firing (the section on the
  step).
  Making it fire would mean a unit bound to the mount,
  which F13 shows does not fire for a mount made outside
  systemd,
  or a path unit,
  which F14 shows cannot see the change.
- `RemainAfterExit=yes` keeps one boot's readiness publication at one start,
  which is what the record's `running` and
  `finished` states mean.
- The `Requires=` the design already wires to the checkout provider keeps its consequence:
  stopping the mount unit by
  a job stops this unit too and runs its `ExecStopPost`,
  which is the design's own stop pass,
  and is the right answer
  to a stop that precedes a replacement of the checkout.
- The addition is `ExecReload=<the same handler> boot-reconcile`,
  so that `systemctl reload` re-runs the checks,
  the
  propagation step and the sweep on demand.
  It is the only manual way back that does not depend on a clone event,
  and
  it is one line of the unit step 19 installs.
  It is a writer's choice,
  listed for veto.

What the unit still cannot do is notice a replacement by itself,
and the design says so rather than pretending
otherwise (L4).

## Ways to meet it

### Way A: the step at the head of every root entry point (adopted)

The handler re-applies the setting as its first act,
in both entry points,
on the descriptor-verified instance,
and
records the outcome.
The boot unit remains a setter,
the sweep becomes one too,
and no unit is bound to the mount.

- Covers:
  every way in "The ways a checkout mount instance is replaced",
  as soon as any trigger the project controls
  runs.
  For way 6,
  that is the next ZED event,
  the next nudge or the next provider hook.
- Costs:
  a run that nobody triggered never repairs (L4);
  the record is one more root-owned file;
  the wrapper's refusal
  needs step 10.
- Why it wins:
  it needs no event that does not exist,
  no new unit,
  no new executable and no configuration;
  it puts the
  change where the design already requires the check (F18);
  it acts on a described instance rather than a path
  (F6,
  F8);
  and it is the only way that covers way 6.

### Way B: a drop-in on the checkout's mount unit that pulls a propagation service

`/etc/systemd/system/home-user-Monochromatic.mount.d/10-propagation.conf` with `Wants=` and `After=` a new
`checkout-propagation.service`.
Every job that starts the mount unit also starts the service.

- Covers:
  ways 1 and 3 where systemd starts the mount,
  and boot.
- Costs:
  it does nothing for a mount made outside systemd,
  which is way 6 and is the common case (`zfs mount -a`,
  F16);
  it needs the mount unit to exist,
  which it does not on this host (F15);
  and it adds a unit whose only job is
  one call the handler can make itself.
- Example:
  `systemctl restart home-user-Monochromatic.mount` re-applies the setting;
  `zfs unmount <ds> && zfs mount <ds>`
  does not.

### Way C: a service bound with `PartOf=` and `BindsTo=`

The existing unit or a second one gains `PartOf=home-user-Monochromatic.mount` and `BindsTo=` the same,
so systemd
restarts it whenever it restarts the mount.

- Covers:
  `systemctl restart` of the mount unit,
  and nothing else.
- Costs:
  it never fires for an external mount or unmount (F13),
  and the restart it does get runs the whole boot mode,
  republishing readiness,
  which is more than the setting needs.
- Example:
  `systemctl restart home-user-Monochromatic.mount` re-runs it;
  `mount -t zfs <ds> /home/user/Monochromatic`
  does not.

### Way D: a path unit on the checkout path or on the mount table

A `checkout-propagation.path` with `PathChanged=`/`PathExists=` on the checkout or `/proc/self/mountinfo`.

- Covers:
  nothing for the checkout's own mount,
  and everything in the namespace for the mount table,
  at the price of a
  process start per mount anywhere on the machine (F14).
- Costs:
  the watch cannot distinguish a mounted from an unmounted mountpoint;
  the rate limit fails the unit under
  churn (F14);
  and the fire hose is a cost nothing here measures.
- Example:
  a container mount in the same namespace starts the service;
  the checkout being remounted does not.

### Way E: a ZED event

A zedlet on a ZFS sysevent.

- Covers:
  nothing.
  There is no mount class (F17),
  and the import and export classes bracket the wrong moment
  ([proposal][proposal],
  "Boot and shutdown unit",
  lines 488 to 498).
- Costs:
  it would need a new sysevent in OpenZFS,
  which is out of this route's hands.

### Way F: the wrapper sets it

- Refused:
  the account has no authority (F7),
  the design forbids a path-based mount or unmount on the account's tree
  ([proposal][proposal],
  "What the design forbids"),
  and the wrapper is not the actor the decision names.

### Ways refused

- Setting it once after each replacement,
  outside the route:
  the decision's rejected alternative,
  which does not persist
  across a replacement ([proposal][proposal],
  line 1289).
- A periodic timer that re-runs the sweep:
  the project decided "Triggers only,
  and we control the triggers"
  ([proposal][proposal],
  lines 1393 to 1400),
  and a timer would need the coalescing and rate rule the design does not
  have.
- A daemon watching the mount table through netlink or a bus subscription:
  a new long-running privileged process,
  a
  new code path in the release,
  and a new failure mode,
  for an answer the next handler run already gives.

## Ranking

Way A > Way B > Way C > Way D > Way E,
with Way F refused.

- A over B:
  A covers way 6 and B does not,
  and B needs a mount unit that this host does not generate (F15).
- B over C:
  B fires for the boot transaction and for `systemctl start`,
  where C fires only for a restart,
  and both
  miss way 6.
- C over D:
  C fires at most too rarely,
  while D fires far too often and cannot tell the state it is asked about.
- D over E:
  D at least fires,
  and E has no event to fire on.

## The propagation record

One file,
replaced whole,
for the reason the readiness record gives for its own shape (F19):
a reader sees one
complete answer,
and no reader takes a half-written one.
It is written by whichever root entry point ran the step,
after the step,
and it names the instance so that a reader can compare it with the live one (F8).

Its text is `key value` lines,
values running to the end of the line,
in this order:
`v1`;
`kind
checkout-propagation`;
`boot`;
`mount-id`;
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
- `mount-id` and `mount-ns-id` are the numbers `statmount` gave (F8),
  in decimal.
- `before` and `after` are one of `shared`,
  `slave`,
  `private`,
  `unbindable` or `unknown`.
- `outcome` is one of `applied`,
  `already-private` or `refused`.
- `step` is `none` on a success,
  and otherwise the step that refused:
  `open`,
  `statmount`,
  `point`,
  `type`,
  `source`,
  `namespace`,
  `before-table`,
  `set` or `verify-after`.
- `errno` is the kernel's error number in decimal on a `set` or `verify-after` refusal,
  and `none` otherwise.
- `time` is the time with its offset,
  as the readiness record writes it.

The field bounds are the kernel's and the existing records':
36 bytes for the boot id,
20 decimal digits for a 64-bit
mount or namespace id,
4095 for the path (`MAXIMUM_CHECKOUT_PATH_BYTES`,
the readiness record's own),
255 for a ZFS
dataset name (`MAXIMUM_POOL_NAME_BYTES`),
11 bytes for the longest propagation word,
15 for the longest outcome,
32 for
a step word,
3 for an error number,
and 35 for a time.
The file bound is derived from those and the key names the same
way the readiness record's is derived and stated beside its constant ([`SPEC.md`][spec],
"The readiness record",
lines
6195 to 6200);
the build computes the number and writes it beside `MAXIMUM_PROPAGATION_BYTES`,
and this design ships no
invented value (rule `LQM`).

The writer's side reuses `clone_mount_readiness_file.rs`'s publication and refusal machinery (F19):
the directory made
and proven root-owned `0755`,
the staging name carrying pid and attempt,
one `rename`,
a sync of the directory,
and a
read back before the write is called done.
The reader's side proves the directory's and the file's owner and mode
before believing either,
exactly as that module's reader does.
`doctor` prints the record and never needs it to act.

The precedent the project recorded for a root-owned store is weighed and not followed in one respect
([slot store][slot-store]):
that store holds one file per item because items accumulate,
and this record holds one file
because one checkout has one current answer.
Where a slot's file needed `O_EXCL` so that exactly one writer creates it,
this record needs the single `rename` so that exactly one whole record stands,
which is the operation that store's
parent record uses for the same purpose.

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
    the record says `applied`.
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
    descriptor's mount to a detached one;
    the second read reports no namespace id;
    the outcome is `refused` with step
    `verify-after`,
    and the test asserts the mount id in the record is the before one.
    Forced by the fixture,
    not by
    timing.
7.  The table read and the descriptor read disagree (race 2).
    The mount table fixture names mount id 41 and the
    descriptor fixture names 42;
    the outcome is `refused` with step `before-table`,
    and the setter is never called.
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

## Measurements owed

No guest ran this round (L1).
Each arm is written to be run as it stands,
in a throwaway guest with a pool of
its own,
and each says what it would show and what it would change.
They are additions to the record the build already
owes ([measurements][measurements]) and to
[`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`](cli-git-zfs-measurements-owed-before-acceptance.md).

- M1.
  What a replacement mount's propagation is,
  and what a private parent changes.
  Create `tank/chk` with
  `zfs create -o mountpoint=/home/guest/chk tank/chk`;
  read
  `findmnt --output TARGET,SOURCE,FSTYPE,PROPAGATION --target /home/guest/chk`;
  then `mount --make-private
  /home/guest/chk` and read again;
  then `zfs unmount tank/chk` and `zfs mount tank/chk` and read again;
  then
  `mount --make-private /home/guest` and repeat the unmount and mount.
  Would show:
  a replacement under a shared parent
  comes back `shared` (F3),
  and under a private parent comes back `private` (F4).
  Would change:
  A1's scope,
  and
  whether the step can ever find a private instance it did not make.
- M2.
  That `mount_setattr` on a descriptor changes the instance the descriptor names and refuses a detached one.
  With the checkout shared,
  `exec 9</home/guest/chk`;
  call `mount_setattr(9, "", AT_EMPTY_PATH, {propagation:
  MS_PRIVATE}, sizeof)`;
  read `statmount(9, STATMOUNT_BY_FD)` before and after,
  comparing `mnt_id` and
  `mnt_propagation`;
  then unmount the checkout and call it again,
  recording the errno.
  Would show:
  the call succeeds on
  the live instance with the same mount id,
  and answers `EINVAL` on the detached one (F6).
  Would change:
  F6's reading,
  and the step's `namespace` and `verify-after` refusals.
- M3.
  That the staging move and the clone attach behave as the decision's measurement said once the step has run.
  Repeat [measurements][measurements] item 3 with the built step in place:
  a shared checkout,
  then the step,
  then the
  `move_mount` of a staged mount (F20).
  Would show:
  `EINVAL` before the step and success after,
  with the clone private
  afterwards.
  Would change:
  nothing in this design,
  and everything in the build's acceptance arm.
- M4.
  What a mount made outside systemd does to the units,
  and that nothing starts.
  `zfs unmount tank/chk && zfs mount
  tank/chk` while the guest's generated mount unit for it is active;
  then `systemctl is-active
  home-user-chk.mount`,
  `systemctl show --property=InvocationID,NActiveEnterTimestamp`,
  `journalctl --unit
  reconcile-git-worktree-clone-mounts.service --since '-2 min'`.
  Would show:
  the unit goes dead and then mounted with a
  new invocation id (F12),
  and no unit of ours starts (F13).
  Would change:
  nothing,
  and it is the observation the whole
  design's shape rests on,
  so it must be recorded even though source already says it (L2).
- M5.
  What a lazy unmount does to clones and what releases them.
  With a clone attached under the checkout,
  `umount
  /home/guest/chk` (expect `EBUSY`,
  F9);
  `umount --lazy /home/guest/chk`;
  then have the handler attach the same clone
  and record the errno (expect `EBUSY`,
  F11);
  close every holder of the detached mount and retry.
  Would show:
  which
  process or descriptor holds the detached clone alive,
  and whether the handler's own retry succeeds once it is gone.
  Would change:
  L7,
  and `doctor`'s words for that state.
- M6.
  What the step costs.
  Time `statmount` on a descriptor,
  the `mount_setattr` call and one whole sweep,
  each
  interleaved with a control of the same shape (a `stat` of the same path),
  in one guest,
  and report ratios with their
  ranges (rules `TVC` and `QNB`).
  Would show:
  whether the step is measurable beside the sweep at all.
  Would change:
  nothing shipped,
  because the design introduces no timeout and no cap,
  and it would give step 22 a number if one is
  ever wanted.
- M7.
  What must be run after a replacement,
  and what admits an attach.
  `systemctl stop
  home-user-chk.mount`,
  `systemctl start home-user-chk.mount`,
  then `monochromatic-git-policy-cli doctor` and one add;
  then `systemctl reload reconcile-git-worktree-clone-mounts.service` and one add again.
  Would show:
  whether readiness
  survives the stop,
  what the wrapper prints,
  and that the reload admits the attach.
  Would change:
  the wrapper's
  refusal wording and "What becomes of `Type=oneshot` with `RemainAfterExit=yes`".
- M8.
  What an unprivileged request to the system manager answers,
  and what a signal to another account's process answers.
  In a guest with a polkit agent on the session and again with none,
  as an unprivileged account,
  run `systemctl reload reconcile-git-worktree-clone-mounts.service`,
  then the same command with `--no-ask-password`,
  and `systemctl kill --signal=TERM home-user-chk.mount`;
  record each exit status and message.
  Then,
  as that account,
  `kill -TERM <a process of another account>` and record the error.
  Would show:
  whether the wait and the refusal take the shapes F22 and F23 read from source,
  and what each message says.
  Would change:
  the wrapper's refusal wording,
  and whether it must say plainly that the command asks for authorization.

## What must be applied to the machine, and where that is recorded

Nothing this design adds is applied by a round.
Applying anything to this machine is the project's act outside a
round,
and the design names the act,
its command and the record that carries it:

- The unit file.
  `reconcile-git-worktree-clone-mounts.service` is written to
  `/etc/systemd/system/reconcile-git-worktree-clone-mounts.service` with the `[Unit]`,
  `[Service]` and `[Install]`
  contents the design states,
  including the `ExecReload=` this design adds,
  and applied with
  `systemctl daemon-reload` followed by `systemctl enable --now reconcile-git-worktree-clone-mounts.service`,
  both run by the project with its own authorization,
  never by a round and never by the wrapper.
  The
  content is step 19's,
  printed by `doctor` and written in the setup runbook step 19 delivers under
  `doc/runbook/`,
  and `doctor` reports the installed unit,
  its ordering edges and its `ExecReload=` as findings of
  its own.
- The record's directory.
  Nothing is installed for it:
  the handler makes `/run/git-worktree-clone-mount-propagation`
  and proves it root-owned on its first run,
  the way the readiness record's directory is made (F19),
  and `/run`
  empties at every boot.
- The propagation setting.
  It is not an installed fact and needs no `zfs set`,
  no mount option and no unit field:
  the step applies it at run time on the verified instance,
  and the record names that instance (F8).
- The checkout dataset and its container.
  This design creates none and sets nothing on any dataset;
  step 19's
  migration does,
  and that step's runbook carries its commands.

Where each act is recorded:
`doctor` prints the commands and reports what is installed,
the setup runbook holds
what is applied,
the unit's journal holds what each run did,
and the propagation record holds the last answer for
the instance it names.

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
  and the SPEC section that states it.
  Step 8's code is where the call in the sweep lands,
  and the plan already gives that split:
  "The checkout's private
  propagation is checked here,
  and the unit that sets it is step 19's"
  ([plan][plan],
  step 8,
  "Open items").
- Step 10 builds the wrapper's half:
  the propagation check in the shared preflight,
  and the refusal in "What the
  wrapper and the caller see between the replacement and the re-run".
  Until it lands,
  L9 stands.
- Step 18 builds `doctor`'s reading of the record and its comparison with the live instance;
  where step 18 lands first,
  the line is its own,
  and step 19 adds the same line to "Doctor gains the migration and boot findings"
  ([plan][plan],
  step 19,
  "What it builds").
- Step 22 builds nothing here.
  This design ships no timeout and no cap,
  so no value waits on its measurements;
  M6
  exists so that a later bound can be sized if one is ever proposed.
- Steps 9,
  14 to 17 and 20 to 23 are unaffected:
  this design changes no unmount,
  no removal,
  no sanitisation and no
  measurement they own.

## Choices this design marks for the project's veto

Each is a writer's choice,
adopted so that the design is whole.
A veto changes the sentence named and nothing else.

- V1.
  The step runs in both entry points rather than only in the boot unit.
  A veto restores "the boot unit sets it,
  and
  every other entry point refuses with the live check",
  which is question 1's second option,
  and changes "Where it
  runs,
  as whom,
  and in what order".
- V2.
  The record's home is a new root-owned directory `/run/git-worktree-clone-mount-propagation` with one file
  `result`,
  rather than fields added to the readiness record.
  A veto makes it four more fields in that record,
  a
  version bump and a parser change there,
  and removes a directory.
- V3.
  The record's words:
  `kind checkout-propagation`,
  outcomes `applied`,
  `already-private`,
  `refused`,
  and the step
  words in "The propagation record".
  A veto renames them.
- V4.
  The unit gains `ExecReload=` running `boot-reconcile`,
  so that `systemctl reload` is the manual re-run.
  A veto
  leaves `systemctl restart` as the only manual way.
- V5.
  The step acts only on `shared` and reports `slave` and `unbindable` without changing them.
  A veto is question 3.
- V6.
  The step's set is `MS_PRIVATE` on the checkout's mount alone,
  with no `AT_RECURSIVE`.
  A veto is question 4.
- V7.
  The sweep files no receipt for a propagation refusal and keeps step 8's rule.
  A veto adds one exception to that
  rule in `SPEC.md` and in `sweep_one_clone.rs`.

## Questions only the project can answer

Each is a matter of value,
authority or risk that no measurement settles.
Each gives the options,
one concrete
example per option,
and the ranking with its reason.

1.  Who may set the propagation with no person present?
    The decision names the boot unit as the setter,
    and this design
    lets every root entry point set it too,
    which means root changes a mount in response to an event nobody watched.

    - Option A,
      every root entry point sets it (this design).
      Example:
      at 14:00 the checkout is unmounted and mounted again outside any handler run;
      at 14:05 a caller runs `git worktree add`,
      the wrapper refuses (L9 aside),
      the
      unit is reloaded as root,
      the step sets the mount private and the sweep attaches.
    - Option B,
      only `boot-reconcile` sets it.
      Example:
      the same 14:00 replacement;
      the 14:05 add is refused,
      and the
      caller's only way forward is a reload of the unit as root,
      which runs `boot-reconcile` and sets it.
    - Pros of A:
      the setting is repaired by the next trigger the project already controls,
      and no window outlives the
      next event.
      Cons of A:
      root changes a mount on an event no person watched,
      and the record is the only trace.
    - Pros of B:
      the act the decision named stays the only act,
      and no event can change a mount.
      Cons of B:
      every way
      but boot needs a root run,
      which makes the route's own triggers unable to repair it,
      and a ZED event or a nudge
      can attach under a checkout that no run has made private,
      which the live check then refuses.
    - Ranking:
      A > B,
      because B's refusal is the same refusal A's live check already gives but without the repair A
      adds,
      so B costs a root run for every replacement and buys no safety the live check does not already give.

2.  Should the wrapper refuse an add on a shared checkout,
    or write its request and let the sweep repair?
    Example for
    each:
    after a 14:00 replacement,
    a caller runs `git worktree add`;
    the refusing wrapper stops before the claim,
    and
    the caller runs one root command and repeats the add;
    the writing wrapper registers,
    snapshots,
    clones,
    writes the
    mount intent,
    and the sweep sets the propagation and attaches,
    with no person involved.

    - Pros of refusing:
      no managed object is created while the parent is unverified,
      which is the accepted preflight's
      own promise ([proposal][proposal],
      "Add and Git registration",
      step 1);
      the caller learns before anything is
      written.
      Cons:
      the caller must run a root command,
      and the route's triggers are not enough on their own.
    - Pros of writing:
      one step for the caller and a self-repairing route.
      Cons:
      an unprivileged command acts while the
      checkout is shared,
      and the add's registration,
      snapshot and clone exist before anyone has verified the parent,
      which the accepted preflight deliberately forbids.
    - Ranking:
      refusing > writing,
      because the accepted preflight already refuses on the same fact,
      and because a
      wrapper that acts on a parent the design calls unverified would need that sentence changed,
      which is a design
      decision rather than a writer's choice.

3.  What should the route do with a checkout mount that is `slave` or `unbindable` rather than `shared`?
    Example:
    `/home/user` is made a slave of a container host's mount;
    a new checkout mount inherits nothing shared
    and reports `slave`;
    today's check admits it,
    and a clone mounted under it is private (F4).

    - Option A,
      admit and report (this design).
      Pros:
      it is not the measured disqualifier (F20),
      the check's own
      sentence is about `shared`,
      and a container or a backup namespace can legitimately leave a slave mount as the checkout's parent.
      Cons:
      a slave mount receives mounts made elsewhere,
      so what the checkout's parent does is not fully in the
      route's hands.
    - Option B,
      refuse.
      Pros:
      only a private parent is admitted,
      which is what the decision's sentence says the unit
      sets.
      Cons:
      it refuses a state that may be the correct one on a machine that runs containers,
      and it turns
      `doctor` into a refusal that must be undone.
    - Ranking:
      A > B,
      because the design's measured fact is `shared` and B would refuse deployments this design has no
      evidence against;
      the reporting A adds is what makes the difference visible.

4.  Should the set be `MS_PRIVATE` on the checkout's mount alone,
    or `AT_RECURSIVE` over everything under it?
    Example:
    the checkout is shared and the vm-builder output dataset is mounted at
    `package/vm-builder/cli/output` under it (the proposal's own inventory);
    recursive makes that mount private too.

    - Option A,
      the checkout alone (this design).
      Pros:
      it changes only the mount this route owns,
      which is what
      "Do not silently change propagation" asks;
      a stacked mount keeps the propagation its owner chose.
      Cons:
      a child
      mount that is itself shared keeps delivering its own mounts elsewhere,
      and nothing in this design reports it.
    - Option B,
      recursive.
      Pros:
      one call makes the whole subtree safe,
      including a descendant nobody inventoried.
      Cons:
      it changes a mount the route did not make,
      and a descendant whose propagation was set deliberately
      would no longer show its own value in the record.
    - Ranking:
      A > B,
      because B changes mounts the route does not own,
      and A's gap can be reported by `doctor` without
      changing anything;
      if the project prefers B,
      "The call that sets it" and V6 change and the record's `before`
      needs a per-mount list.

## Dependencies on sibling designs

- "Which triggers a lost last event waits for":
  this design assumes the provider hook runs `boot-reconcile`,
  so that a
  late mount both repairs the propagation and republicates readiness.
  If that design has the hook run only the sweep,
  the step and the record still happen,
  and a replacement that stopped the unit leaves no readiness record for the rest
  of the boot;
  then the sentence "a replacement outside systemd leaves no attach admitted until a run publishes
  readiness" needs that design's answer folded in.
- "The record that carries origin cleanup",
  "Where the add's durable binding record lives",
  "Where a removal's
  persisted destruction plan lives",
  "The add-time Git lock",
  "The dirty-main restore on rung 1",
  "The deferred-destroy
  checks",
  "The forced arm of a removal",
  "What a cancelled,
  timed-out or killed removal leaves":
  no dependency.
  This
  design reads no record they place and writes none they read.
- The wrapper's half (step 10) and the add-time lock's design both touch the add's preflight.
  This design adds one
  check to that list and nothing else,
  so the two can be reconciled by whoever writes the later one.

## What this design does not cover

- It does not measure anything.
  Every kernel,
  ZFS and systemd claim is a source reading,
  and "Measurements owed" says
  what would change each reading.
- It does not design the provider hook,
  the nudge's coalescing,
  or the triggers of the lost-last-event item.
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
- It does not propose a timer,
  a daemon,
  a new unit,
  a new executable or a configuration key,
  and it proposes no
  timeout or cap.

## Sources read

- Linux `v7.3`:
  `fs/namespace.c`,
  `fs/pnode.c`,
  `include/uapi/linux/mount.h`,
  `include/linux/namei.h`,
  `include/linux/fsnotify.h`.
- OpenZFS `zfs-2.4.4`:
  `etc/systemd/system-generators/zfs-mount-generator.c`,
  `etc/systemd/system/zfs-mount.service.in`,
  `include/sys/sysevent/eventdefs.h`,
  `module/os/linux/zfs/zfs_vfsops.c`,
  `module/zfs/dsl_dataset.c`.
- systemd `v262`:
  `src/shared/mount-setup.c`,
  `src/core/mount.c`,
  `src/core/unit.c`,
  `src/core/service.c`,
  `src/core/transaction.c`,
  `src/core/unit-dependency-atom.c`,
  `man/systemd.mount.xml`,
  `man/systemd.unit.xml`,
  `man/systemd.service.xml`,
  `man/systemd.path.xml`.
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

## Links

[proposal]: cli-git-zfs-worktree-design-proposal.md
[plan]: cli-git-zfs-worktree-implementation-plan.md
[measurements]: cli-git-zfs-measurements-owed-before-acceptance.md
[spec]: ../../package/git-policy/cli/SPEC.md
[sweep-choices]: ../decision/cli-git-zfs-step-8-sweep-choices.md
[slot-store]: ../decision/cli-git-zfs-slot-identity-store-format.md
[staging-decision]: ../decision/cli-git-zfs-staging-under-cache.md
