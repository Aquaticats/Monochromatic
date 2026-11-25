# The add's clone gets `mountpoint=legacy` without the forbidden grant

## Status

This is a proposal for the owner's decision.
It is not a decision,
and it is unapproved and non-final:
only the project declares a design complete.

It was written in the worktree
`.claude/worktrees/zfs-add-clone-mountpoint-without-the-forbidden-grant`,
on branch `docs/planning-zfs-add-clone-mountpoint-without-the-forbidden-grant`,
made from `main` at `1e82cffdfa`.

No code changed.
Nothing under `doc/decision/` or `doc/handover/` was touched.
One throwaway `mvm` guest ran the probe,
and `mvm` destroyed it after its log was saved.

The question is
[issue 6462](https://github.com/Aquaticats/Monochromatic/issues/6462):
the ZFS add makes its clone with the property `mountpoint=legacy`;
that property is one of the nine the design forbids the managed account;
so the kernel refuses the account's clone request.

The answer this document asks for is C:
the add sends no property,
and the clone inherits `mountpoint=legacy` from the container the root setup already creates that way.
The issue's two supporting sentences were tested.
A clone inherits from its new parent,
not from its origin.
A refused `-o` leaves no clone behind.

## What the issue found, and what it left open

The issue's probe ran in a guest on OpenZFS 2.4.4 as the account,
with the nine decided grants.
It recorded this:

```text
$ zfs clone <checkout>@probe <container>/bare
$ zfs clone -o mountpoint=legacy <checkout>@probe <container>/with-property
filesystem successfully created, but it may only be mounted by root
cannot create '<container>/with-property': permission denied
```

Two things follow from it,
and they are not the same claim.
The delegated account cannot create a clone that carries `mountpoint` at creation.
That is measured,
and it is re-measured in this round's guest.
A clone without the property "inherits the origin's `mountpoint` (the checkout path) and `canmount=on`",
which is the issue's stated reason the property is needed at creation.
That sentence is a claim to test,
and this document tests it.

The refusal itself is expected from the kernel source.
The section "How the kernel refuses the property" says where.

## How the kernel refuses the property

`zfs clone -o <prop>=<value>` sends one `ZFS_IOC_CLONE` request whose input carries a `props` nvlist.
The kernel creates the dataset first and applies the properties after (`module/zfs/zfs_ioctl.c:3844` to `:3871`,
OpenZFS `zfs-2.4.4`):

```c
	error = dsl_dataset_clone(fsname, origin_name);
	/*
	 * It would be nice to do this atomically.
	 */
	if (error == 0) {
		error = zfs_set_prop_nvlist(fsname, ZPROP_SRC_LOCAL, nvprops, outnvl);
		if (error != 0)
			(void) dsl_destroy_head(fsname);
	}
```

`zfs_set_prop_nvlist` validates every property through `zfs_check_settable` (`:2914`).
Its default arm calls `zfs_secpolicy_setprop` (`:5218` to `:5221`),
and that function asks for the property's own name as a delegation permission (`:606` to `:633`):

```c
	return (zfs_secpolicy_write_perms(dsname, zfs_prop_to_name(prop), cr));
```

So `-o mountpoint=legacy` needs the `mountpoint` delegation on the dataset being created,
which is the clone.
The grant that reaches a not-yet-existing clone is a descendant grant on the container,
because `dsl_deleg_access_impl` checks the dataset's own local grants
and then every ancestor's descendant grants (`module/zfs/dsl_deleg.c:575` to `:589`).
A local grant on the container does not reach the clone.
That is why option A has exactly one shape.

The clone request's own policy is cheaper.
`zfs_secpolicy_create_clone` asks for `clone` on the origin,
`create` on the parent,
and `mount` on the parent (`module/zfs/zfs_ioctl.c:1128` to `:1153`).
All three are among the nine decided grants,
so a clone **without** properties passes.
The guest probe confirms that.

The userland's extra line in the issue's transcript comes after the clone succeeded.
`zfs_do_clone` calls `zfs_mount_and_share`,
which for a non-root caller with `canmount=on` prints "filesystem successfully created,
but it may only be mounted by root",
because `zfs_mount_delegation_check` is `geteuid() != 0`
(`cmd/zfs/zfs_main.c:772` to `:820`,
`lib/libzfs/os/linux/libzfs_mount_os.c:564` to `:567`).
That call's result is discarded (`cmd/zfs/zfs_main.c:926` to `:929`,
`(void) zfs_mount_and_share(...)`).
So the CLI prints the line and still exits 0.
`-u` skips the call and prints nothing.
The add sends the ioctl itself and never takes that userland path,
so it sees neither the line nor the mount attempt.
The probe measures both.

## Where a clone's properties actually come from

Three independent readings say the clone inherits from its **new parent**,
the clone container,
and not from its origin.

- A clone's dataset record points at the target's directory,
  not the origin's (`module/zfs/dsl_dataset.c:1200`,
  `dsphys->ds_dir_obj = dd->dd_object;`).
  That line sits inside `dsl_dataset_create_sync_dd` at `:1177` to `:1295`,
  which copies no property from the origin.
- Inheritance resolves by walking the dataset's own directory upward
  (`module/zfs/dsl_prop.c:115`,
  `for (; dd != NULL; dd = dd->dd_parent)`).
  That walk is reached from `dsl_prop_get_ds` at `:192`,
  and the origin is not on it.
- `mountpoint` is registered `PROP_INHERIT` (`module/zcommon/zfs_prop.c:583` to `:587`),
  while `canmount` is `PROP_DEFAULT` (`:536` to `:538`).
  So `zfs_prop_inheritable(canmount)` is false (`:996` to `:1002`),
  and a clone gets the default `canmount=on` whatever its container says.
  `setuid` and `devices`,
  which the design also relies on,
  are `PROP_INHERIT` (`:503` to `:511`).

The repository had already measured the first half of this,
in `doc/planning/cli-git-zfs-clones-mounted-under-the-checkout.md`
("Where the clones' parent dataset can sit",
citing `guest-exact-zfs.log:18` to `:25`):

```text
.../git-worktree-clone-datasets          mountpoint  legacy  local
.../git-worktree-clone-datasets/registered  mountpoint  legacy  inherited from .../git-worktree-clone-datasets
.../git-worktree-clone-datasets/registered  canmount    on      default
```

So the design's layout section is right,
and two shipped sentences are wrong.
`package/git-policy/cli/SPEC.md` says "a clone inherits the origin's mountpoint".
`src/native/worktree_clone_add_route/add_zfs.rs`,
in `clone_the_origin`,
says the same in a comment.
This round's guest probe re-measures it,
with a control that separates the two readings.

## What the design forbids, and why, in the record's words

The list is `FORBIDDEN_PERMISSIONS` in
`package/git-policy/cli/src/native/health/doctor_zfs_route/delegation_checks.rs:90`.
Its module doc gives the reason the route cannot carry these (`:76` to `:85`):
they are "permissions whose presence would let the account change the lineage the removal proof rests on,
or change what a mount exposes".
`allow` is the build's own addition,
because it "would let the account grant itself any of the others".

The design's own sentences follow,
by name.

- `mountpoint`:
  "Do not grant `mountpoint`,
  `canmount`,
  `setuid`,
  `devices`,
  `rename`,
  `promote`,
  `receive` (including `receive:append`) or permissions over the whole home tree"
  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
  "Scoped delegation").
  The same proposal gives its reason in "Dataset and path layout":
  "An ordinary clone mountpoint would reopen the early-mount hazard even if only root could set it".
  The mount study records that "any future change that gives a clone a real mountpoint re-opens the early-mount hazard"
  (`cli-git-zfs-clones-mounted-under-the-checkout.md`,
  "What the design needs",
  item 4).
  The hazard it names is measured there.
  A clone was mounted at a path inside the checkout before the checkout's own mount,
  so the checkout's later mount covered it.
  The clone was then unreachable by path, and it blocked the pool's export.
- `canmount`:
  the record gives no separate reason.
  It falls under the module's shared sentence about what a mount exposes,
  and under the layout rule that no clone gets an automatic mount,
  whose next sentence accounts for the container's own `canmount=off` being local.
- `setuid`,
  `devices`:
  the same shared sentence.
  The design sets both off on the container,
  and enforces `nosuid,nodev` at attach independently of the properties
  ("Descriptor mount and unmount",
  VM item 7).
  The probe shows both properties inheriting into the clone's mount flags.
- `promote`:
  "Promoting a clone of the checkout makes the checkout a clone of the promoted dataset
  and transfers the origin snapshot and the checkout's older snapshots into the promoted dataset".
  So the record keeps the promotion to root alone
  ("Eligible clone origins" and "Clone removal identity protection").
- `rename`,
  `receive`,
  `receive:append`:
  "Without `promote`,
  `rename` or `receive` authority,
  the delegated account cannot move foreign history into the removal set"
  ("Clone removal identity protection").
  The design marks that reasoning as taken from VM items 11 and 12,
  and "to be checked in the product".
- `allow`:
  the build's own addition,
  as the module doc says.

Two measured facts about this list matter to the options.

- `zfs-allow(8)` says the Linux delegation support excludes "`mount`,
  `unmount`,
  `mountpoint`,
  `canmount`,
  `rename`,
  and `share`",
  because "the Linux `mount` command restricts modifications of the global namespace to the root user"
  (`man/man8/zfs-allow.8:110` to `:120`).
  That is about who may mount,
  not about the property check.
  The kernel still gates the property,
  and a root-granted `mountpoint` permission is accepted by `zfs allow`
  (`module/zcommon/zfs_deleg.c:46` to `:95`,
  `zfs_prop_delegatable` at `module/zcommon/zfs_prop.c:807` to `:818`).
- `canmount` being non-inheritable is why the design's container `canmount=off` does not reach the clones,
  as the design's layout section already says.

## The options

Each option is designed until it fails.
Each is judged on what it delivers for the add's safety and correctness.
The effort to build is a cost beside that,
never a ranking reason.

### A: grant `mountpoint` on the container's descendants

The setup grants `zfs allow -d -u <account> mountpoint <container>`,
and the add keeps sending `-o mountpoint=legacy`.

What it delivers:
the clone's property is local and explicit,
so no reader depends on inheritance.

Where it fails:

- it deletes one entry of a list whose stated purpose is the removal proof and the mount exposure
  (`delegation_checks.rs:76` to `:85`),
  and the design's threat boundary is the absence of exactly these grants;
- the same grant lets the account aim its own clone's mountpoint anywhere,
  including inside the checkout,
  and a root-run `zfs mount -a` at boot then mounts it there.
  The section "A with the grant" records a probe that mounts an account-aimed clone
  inside the checkout through `zfs mount -a`.
  The account still cannot mount by itself (the man page's Linux exception),
  so this is not an escalation.
  It is exactly the early-mount hazard the design's sentence names,
  reached without root cooperation beyond the boot's own `zfs mount -a`;
- it cannot be narrowed.
  Delegation has no per-value or per-name scope,
  so the only shape that reaches a clone at creation is a descendant grant on the container,
  which every clone of that container receives.

`doctor`'s `permission` finding would also have to be taught an exception,
which is the design change the issue names as crossing the boundary.

### B: clone bare, and have the root handler set `mountpoint=legacy` when it attaches

The add sends no property.
The handler runs `zfs set mountpoint=legacy <clone>` before it attaches,
as root,
which bypasses delegation.

What it delivers:
the same ending property as A without the forbidden grant,
and it works even if the container's own `mountpoint` is not `legacy`.

Where it fails:

- the handler's sweep already refuses a child whose `mountpoint` is not `legacy` before it attaches
  (`package/git-policy/cli/SPEC.md`,
  "The sweep's checks":
  "`mountpoint` is `legacy`,
  so neither `zfs mount -a` nor the systemd mount generator mounts it first").
  B therefore gives the sweep a write it does not have,
  before the check that exists to keep a clone out of the automatic mounts.
  That check's meaning becomes "the handler makes this true" instead of "the pool already proves this".
  A clone whose property is wrong but whose identity is otherwise right would be repaired rather than refused,
  which is a wider change than the clone request;
- it adds a root-side property write to the attach path,
  which the design keeps to mount placement and folder identity only;
- it does not close the window.
  The set happens at attach,
  and a boot's `zfs mount -a` runs before the handler.
  Under the issue's premise the clone would carry the checkout's own mountpoint through that window,
  which is worse than A,
  not better.
  Under the measured inheritance the clone is already `legacy` in that window,
  so B's set changes the value to the same value and buys nothing.

### C: rely on the container's `mountpoint=legacy`, and set nothing at creation

The root setup already creates the container with `mountpoint=legacy`
(design,
"Dataset and path layout";
`SPEC.md`,
"The sweep's checks").
The add drops the property from the clone request,
and the clone inherits `legacy` from its parent at creation.

What it delivers:
a clone that is `legacy` from the instant it exists,
with no window,
no forbidden grant,
no new root write,
and no change to the sweep's check,
which still sees the property it already requires.
It relies on the same inheritance mechanism the design already relies on for `setuid=off` and `devices=off`,
which the sweep also checks.

Where it fails,
and what it costs:

- it depends on root-owned setup state,
  so the container's `mountpoint` must be verified rather than assumed.
  `doctor` already checks exactly this.
  `check_dataset` requires the container's `mountpoint` to be `legacy` and its `canmount` to be `off`,
  and `check_clone_properties` requires every clone's `mountpoint` to be `legacy`
  with the message "a clone must inherit `legacy`"
  (`src/native/health/doctor_zfs_route/zfs_checks.rs:425` to `:445` and `:525` to `:560`).
  The add's own preflight does not run those checks:
  it uses only the delegation half of the same module (`src/native/worktree_clone_add_route/add_grants.rs`).
  So the guard exists and needs no new design,
  but the add does not consult it today,
  and a container changed after the last `doctor` run would give every clone an ordinary mountpoint
  until someone ran it again;
- the add's own reading after the clone does not read properties.
  The device crate has identity,
  listing and delegation reads (`src/requests/read_requests.rs`) but no `ZFS_IOC_GET_PROP` request.
  So "verify the clone's mountpoint is legacy" at add time is a new request type
  in `package/zfs/request-to-openzfs-kernel-device` plus one read in `add_zfs.rs`.
  That is the effort C adds beside removing one line.
  Without it,
  the wrong container shows up as the mount wait timing out rather than as a refusal naming the container;
- two shipped sentences become false and must be corrected:
  the SPEC's "a clone inherits the origin's mountpoint",
  and the code comment beside the request in `clone_the_origin`;
- a clone's property source reads `inherited from <container>` rather than `local`,
  so a reader must understand inheritance to see why it is safe.
  The probe prints both sources.

### Considered and dropped

- **Root performs the clone.**
  It would remove the delegated clone step the design's whole route is built on,
  and it needs a privileged request channel the design does not have.
- **A channel program sets the property.**
  `zfs.sync.set_prop` exists in `module/zfs/zcp_synctask.c:505` to `:540`,
  but `ZFS_IOC_CHANNEL_PROGRAM` is registered with `zfs_secpolicy_config` (`module/zfs/zfs_ioctl.c:7664` to `:7665`),
  so the account cannot issue one.
  A root-run fixed program would be a new privileged surface,
  for one property whose value inheritance already gives.
- **Set the property after creation with `ZFS_IOC_SET_PROP`.**
  The same `mountpoint` delegation is checked
  (`zfs_check_settable`,
  `zfs_secpolicy_setprop`),
  so it fails for the same reason as A.
  The design's existing fallback sentence
  ("If the path refuses and inspection proves no clone exists,
  create it without `props`,
  then set the local property with `ZFS_IOC_SET_PROP`",
  "Add and Git registration" item 3)
  covers the local desired-state user property,
  whose permission `userprop` is granted,
  not `mountpoint`.

## What a person or `zfs mount -a` does with a clone before the handler attaches

- `zfs mount -a` skips a dataset whose mountpoint is `legacy` or `none`,
  silently,
  in the automatic case
  (`cmd/zfs/zfs_main.c:7210` to `:7248`).
- The systemd mount generator ends unit generation for `legacy` and `none`
  (`etc/systemd/system-generators/zfs-mount-generator.c:430` to `:432`),
  and `zfs_is_mountable` returns false for both (`lib/libzfs/libzfs_mount.c:282` to `:297`).
- `zfs mount <clone>` refuses with "legacy mountpoint" (`cmd/zfs/zfs_main.c:7242` to `:7248`),
  because `zfs mount` passes `zfsutil`.
- `mount -t zfs <clone> <dir>`,
  run by root,
  succeeds for a legacy dataset and fails for any other,
  by the helper's own check (`cmd/mount_zfs.c:290` to `:327`).
  The handler does not use this path.
  Its descriptor attach holds the objset by name and never reads `mountpoint`
  (`module/os/linux/zfs/zpl_super.c:849` to `:866`).

So under A,
between creation and attach,
a clone carries whatever mountpoint the account aimed it at,
and a boot's `zfs mount -a` mounts it.
Under B,
the clone carries inherited properties until the handler's write.
Under the measured inheritance that is `legacy`,
so nothing mounts it,
and the write changes nothing.
Under C,
the clone is `legacy` from creation.
So `zfs mount -a` skips it,
the generator makes no unit for it,
and only an explicit root `mount -t zfs` can place it,
which is what the handler does not need.

## The guest probe

One `mvm` guest,
`zfs-clone-mountpoint-probe`,
prepared by `package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`
(`create`,
`prepare`),
ran OpenZFS 2.4.4-1 (`zfs-2.4.4-1`,
`zfs-kmod-2.4.4-1`) on kernel 6.8.0-147-generic,
with the account `ubuntu`.
The probe made a 1 GiB file-backed pool with the design's two siblings,
the nine decided grants to the account,
and the container at `mountpoint=legacy`,
`canmount=off`,
`setuid=off`,
`devices=off`.
Its logs are `mountpoint-arm-2026-11-25.log` and `mountpoint-arm-two-2026-11-25.log` in this round's `keep/` folder,
beside the fixture's own `create` and `prepare` logs.
The lines the rest of this document rests on follow.

### The setup, before any clone

```text
cloneprobe/ROOT/clones  mountpoint  legacy      local
cloneprobe/ROOT/clones  canmount    off         local
cloneprobe/ROOT/clones  setuid      off         local
cloneprobe/ROOT/clones  devices     off         local
```

### C: a bare clone as the account, and what it inherits

From `mountpoint-arm-two-2026-11-25.log`:

```text
filesystem successfully created, but it may only be mounted by root
bare-clone exit=0
cloneprobe2/ROOT/clones/bare  mountpoint  legacy      inherited from cloneprobe2/ROOT/clones
cloneprobe2/ROOT/clones/bare  canmount    on          default
cloneprobe2/ROOT/clones/bare  setuid      off         inherited from cloneprobe2/ROOT/clones
cloneprobe2/ROOT/clones/bare  devices     off         inherited from cloneprobe2/ROOT/clones
```

The same request with `-u` printed nothing and exited 0,
and read the same four values.
The message is the userland's failed mount attempt,
not the clone.

### The separating control: a container with an ordinary mountpoint

From `mountpoint-arm-2026-11-25.log`:
a second container created at `mountpoint=/home/cloneprobe-other`,
and a bare clone in it as the account:

```text
other-container exit=0
cloneprobe/ROOT/other/plain  mountpoint  /home/cloneprobe-other/plain  inherited from cloneprobe/ROOT/other
cloneprobe/ROOT/other/plain  canmount    on                            default
cloneprobe/ROOT/other/plain  setuid      on                            default
cloneprobe/ROOT/other/plain  devices     on                            default
```

The clone followed its parent,
`/home/cloneprobe-other`,
and not the checkout at `/home/cloneprobe-checkout`,
which is the dataset it was cloned from.
So the issue's sentence is wrong about the direction of inheritance.
A clone inherits its new parent's properties,
as the source reading in "Where a clone's properties actually come from" says.
The control also shows `setuid` and `devices` following the same route,
which is the mechanism the design already relies on.

### A without the grant: refused, and no clone left behind

```text
cannot create 'cloneprobe2/ROOT/clones/withprop': permission denied
with-property exit=1
```

The listing right after it holds only the two clones the account made without the property (`bare` and `bareu`).
So the kernel's cleanup (`dsl_destroy_head` at `module/zfs/zfs_ioctl.c:3865` to `:3870`) removed the partial clone.
The issue's sentence that the refusal "leaves a clone the add does not go on to record"
is wrong in this guest and on this release.

Root's control with the same `-o` succeeded and read `mountpoint legacy local`
(`mountpoint-arm-2026-11-25.log`,
`control-with-property-as-root`).

### A with the grant: works, and hands the account an automatic mount

After `zfs allow -d -u ubuntu mountpoint <container>`,
the account's `zfs clone -o mountpoint=legacy` succeeded:

```text
arm-a-granted exit=0
cloneprobe/ROOT/clones/granted  mountpoint  legacy  local
```

Then,
as the account:

```text
ubuntu$ zfs set mountpoint=/home/cloneprobe-checkout/.claude/worktrees/auto <container>/granted
aim exit=0
cloneprobe/ROOT/clones/granted  mountpoint  /home/cloneprobe-checkout/.claude/worktrees/auto  local
```

And root's ordinary boot scan mounted it there:

```text
zfs-mount-a exit=0
52 29 0:45 / /home/cloneprobe-checkout ... - zfs cloneprobe/ROOT/checkout ...
62 29 0:48 / /home/cloneprobe-other/plain ... - zfs cloneprobe/ROOT/other/plain ...
70 52 0:47 / /home/cloneprobe-checkout/.claude/worktrees/auto ... - zfs cloneprobe/ROOT/clones/granted ...
```

Mount 70 sits on mount 52,
which is the checkout's own mount.
So the account's property choice became a mount inside the checkout,
made by root's own `zfs mount -a`.
A second aim at an arbitrary path outside the checkout was mounted the same way at `/home/cloneprobe-aimed`
(mount 70,
parent 29).
The same scan mounted `cloneprobe/ROOT/other/plain`,
the positive control that a clone with an ordinary mountpoint is exactly what `zfs mount -a` picks up.

### Between creation and attach

From `mountpoint-arm-two-2026-11-25.log`:

```text
zfs-mount-a exit=0
cloneprobe2/ROOT/clones/bare   legacy  on  no
cloneprobe2/ROOT/clones/bareu  legacy  on  no
```

`zfs mount <clone>` refused with `cannot mount '<clone>': legacy mountpoint` and exit 1.
Root's `mount -t zfs <clone> /home/cloneprobe2-attach` succeeded,
and the mount table showed `rw,nosuid,nodev`,
the container's `setuid=off` and `devices=off` carried into the mount flags.

### B: root sets the property at attach

From `mountpoint-arm-2026-11-25.log`:
root's `zfs set mountpoint=legacy <bare>` exited 0.
It changed the source from `inherited from cloneprobe/ROOT/clones` to `local`,
with the value `legacy` before and after.
On the measured inheritance the write is a no-op in value.

## Ranking

**C > B > A.**

C ranks first because it delivers the property the handler and the sweep require,
from creation,
with nothing added to the threat boundary and nothing added to the root handler.
It also uses the mechanism the design already relies on for `setuid` and `devices`.
Its costs are the add's own read of the same property check,
which `doctor` already carries but the add does not,
and the correction of two shipped sentences.

B ranks second,
above A and below C,
because it keeps the forbidden list intact but buys nothing C lacks.
Both end with a `legacy` clone.
B pays a root-side property write and changes what the sweep's check means,
and its window is no safer than C's.
Where C fails closed only once the add's preflight gains the property check,
B would repair the clone only after the boot window that matters has passed,
so B does not cover that failure either.

A ranks last because it delivers the property by deleting the boundary the removal proof rests on,
and it hands the account the ability to aim an automatic root mount into the checkout,
which the probe measures.
Everything A delivers,
C delivers without either cost.

## What the proposal asks to change, if the owner picks C

- The design proposal,
  "Dataset and path layout":
  say that a clone inherits the container's `mountpoint=legacy` and sets no property of its own,
  and keep the sentence about `canmount` being non-inheritable.
- The design proposal,
  "Add and Git registration" item 3:
  replace "Request `mount` via the local desired-state user property ... Send it in clone `props` only if the
  delegated creation path is verified"
  with the same,
  minus `mountpoint`,
  and state that the clone carries no creation-time property.
- `SPEC.md`,
  "The add that registers a ZFS clone before it fills the slot":
  the clone is made with no `mountpoint` property,
  because it inherits the container's,
  and the sentence that says it inherits the origin's is corrected.
- `add_zfs.rs`,
  `clone_the_origin`:
  no property list,
  and the comment says what the pool actually does.
- `doctor` and the add's preflight:
  the container and clone property checks already exist in `doctor`.
  The proposal adds them to the add's preflight,
  or names the add's own post-clone read of the clone's `mountpoint` through a new `ZFS_IOC_GET_PROP` request,
  so a container changed between preflight and clone is caught rather than waited on.
- Tests,
  in the guest:
  the clone request as the account with no property succeeds and the clone reads `legacy` inherited from the container;
  the same request with the property is refused and leaves no clone;
  a control container with an ordinary mountpoint gives a clone an ordinary mountpoint,
  which the sweep refuses;
  and the preflight's new check refuses that control container.

## Open items

- The handler's real attach for a legacy clone was measured in item 10 of the design's VM measurements,
  not here.
  This probe shows the property and the mount paths around it.
- `zfs clone -o mountpoint=legacy` as the account was refused under the nine grants
  and succeeded once `mountpoint` was added,
  which is the whole of A's delivery.
- The probe's pool is not encrypted,
  and the real checkout is.
  The property walk does not depend on encryption;
  the clone's key state does,
  and item 5 covers that.
- The add's own binary was not run as the account,
  and the last open item names why.
- The add cannot yet run as the account at all:
  the wrapper's account companion refuses a bare `ubuntu` at `/proc/self/setgroups`,
  as the round that found this issue recorded.
  So the probe uses the `zfs` CLI and the ioctl behavior it implies.
- Only OpenZFS 2.4.4 was read and measured.
  An older release's inheritance is not claimed here.
