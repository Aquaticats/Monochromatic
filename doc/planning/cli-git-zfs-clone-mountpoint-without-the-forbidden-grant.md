# The add's clone gets `mountpoint=legacy` without the forbidden grant

## Status

This is a proposal for the owner's decision.
It is not a decision,
and it is unapproved and non-final:
only the project declares a design complete.

It was written in the worktree
`.claude/worktrees/zfs-clone-mountpoint-design-second-revision`,
on branch `docs/planning-zfs-clone-mountpoint-design-second-revision`,
made from `main`.
It revises the proposal of 2026-11-25
after that proposal's gate review,
which is
`/home/user/temp/agent/gate-review-zfs-clone-mountpoint-without-the-forbidden-grant-2026-11-25/keep/review.md`.
Every High,
Medium and Low that review raised is answered here,
by a change in this document or by evidence that the review was wrong;
`## The gate review's findings, and where each is answered` names each one.

No code changed.
Nothing under `doc/decision/` or `doc/handover/` was touched.
One throwaway `mvm` guest ran the arms,
and `mvm` destroyed it after its logs were saved.

The question is
[issue 6462](https://github.com/Aquaticats/Monochromatic/issues/6462):
the ZFS add makes its clone with the property `mountpoint=legacy`,
that property is one the design forbids the managed account,
so the kernel refuses the account's clone request.

## The answer in four lines

The add,
today,
sends one property with its clone request:
`mountpoint=legacy`
(`package/git-policy/cli/src/native/worktree_clone_add_route/add_zfs.rs:103` to `:112`).
That property is one the design forbids the managed account,
so as the account the kernel refuses the whole clone request,
and the add cannot make a clone at all.

The accepted design says something else:
clones "must inherit `mountpoint=legacy`"
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:283`),
and root must not grant the account `mountpoint` (`:349` to `:350`),
which `doc/decision/cli-git-zfs-worktree-design.md` accepts.

The three answers produce this.
Under C a clone is `legacy` from the instant it exists,
the account holds no new permission,
and no new root write exists.
Under B a clone is `legacy` from creation whenever the container is,
and root also rewrites the property at attach,
a write the handler does not have today.
Under A the clone's property is a value the account itself sets,
bought by granting the account the one permission the design forbids,
which also lets the account choose the path root's own boot mount uses.

C is the accepted text.
The add is that text's one departure,
and C brings the add back to it.
A and B would each amend the accepted design,
so the question is whether to keep the design or to change it.

## The question for the owner

This section stands alone:
nothing in it needs the rest of this document.

### What the add does, what the design says, and what each answer gives

The ZFS add works as the managed account.
Today it asks the kernel to create its clone with the property `mountpoint=legacy`,
and `mountpoint` is a permission the design forbids that account,
so the kernel refuses the request and no clone is made.

The accepted design says a clone inherits `mountpoint=legacy` from the container,
and that the account is never granted `mountpoint`.

Three answers exist.
C deletes the property from the request and relies on that inheritance.
B deletes it too,
and has root write the property at attach.
A grants the account `mountpoint`,
so the request goes through.

Ranking:
C > B > A,
because C is what the accepted design already says and the add is
its only departure;
B keeps the forbidden list intact but makes the root handler write a
property,
after the clone exists;
A buys a local property by handing the account the
permission that lets it aim root's own mount at any path it chooses.

### Choice 1: which answer

#### C: the add sends no property, and the clone inherits the container's

Recommended.
What it gives:
a clone that is `legacy` from the instant the kernel creates it,
no permission added to the account,
no write added to the root handler,
and a pool state that already matches what the mount handler's sweep requires.
Worked example:
the container is `mountpoint=legacy`,
the account holds the nine decided grants and no other,
so `zfs clone <checkout>@<origin> <container>/<slot>` succeeds with no property,
and the clone reads `mountpoint legacy inherited from <container>`.

What it costs:
the container must be verified rather than assumed,
because the setup that creates it is plan step 19's and is not built yet;
today an operator or a fixture makes it.
Two shipped sentences are also wrong and get corrected:
`package/git-policy/cli/SPEC.md:5036` to `:5037` and the comment at `add_zfs.rs:103` to `:109`
both say a clone inherits the origin's mountpoint.
It also leaves a reader to know that an inherited value is a real value,
where A writes one locally.

Why it ranks first:
it is the accepted design's own sentence,
so choosing it changes no accepted record,
and each of the next two answers pays something for a property that inheritance already gives.

#### B: the add sends no property, and root sets `mountpoint=legacy` at attach

Ranked second.
What it gives:
the same ending value as C without granting the account anything,
and it keeps working if the container's own `mountpoint` is not `legacy`.

What it costs:
root's handler gains a property write that the accepted design does not give it
(`doc/decision/cli-git-zfs-worktree-design.md:40` gives it "own clone mount and unmount",
not property writes),
and a clone whose property is wrong becomes something the handler repairs
rather than something the sweep refuses,
which changes what the sweep's check means.
The repair also arrives after the clone exists:
an interrupted add leaves a clone whose property nothing has fixed,
until the handler's next attach.

Why it ranks second:
it keeps the forbidden list intact,
which A does not,
and everything it delivers C delivers,
so its whole cost is paid for a value C has from creation.

#### A: grant the account `mountpoint`, and keep sending the property

Ranked last,
and it has two shapes.
The first grants `mountpoint` on the container's descendants (`zfs allow -d -u <account> mountpoint <container>`);
the second grants it at creation (`zfs allow -c mountpoint <container>`),
a command that names no account,
so the kernel copies it onto every dataset created below the container,
for its creator,
and the account holds `mountpoint` on its own clone afterwards as well.

What it gives:
a property value written locally on the clone,
which survives a later change of the container's own `mountpoint`,
and a clone request that tolerates a container that is not `legacy`.

What it costs:
it deletes one entry from the list whose absence is the removal proof's threat boundary,
and it gives the account the ability to choose the path root's own `zfs mount -a` uses.
Both shapes are measured.
With the descendant grant the account set its clone's `mountpoint` inside the checkout
and at an arbitrary path,
and root's ordinary boot scan mounted it at each
(`### A with the descendant grant: works, and hands the account an automatic mount (log one)`).
With the creation-time grant the same account set the mountpoint at a path it chose
and root's scan mounted it there over a root-owned,
non-empty directory
(`## The guest probe`,
`### A's second shape: the creation-time grant, measured this round`).

Why it ranks last:
it is the only answer that gives up a permission the design forbids,
and it delivers nothing C lacks except a local value,
which C's own check covers by refusing a container that is not `legacy`.

### Choice 2: where C's check runs

C needs one check,
because the container's property is setup state that no shipped code
reads at add time today.
Two placements exist,
and a third option is both.
In all three the check reads a reply the add already asks for:
the container's properties arrive in the same kernel reply as its GUID
(`package/zfs/request-to-openzfs-kernel-device/src/kernel_device/listed_object.rs:36` to `:47`),
and `check_dataset`,
the check `doctor` already runs on exactly those properties,
is public
(`package/git-policy/cli/src/native/health/doctor_zfs_route/zfs_checks.rs:469`,
re-exported at `mod.rs:163`).
So no new request type is needed by any placement.

#### Both: refuse in the preflight, then read the clone after creation

Recommended.
What it gives:
a container that is not `legacy`,
has `canmount` on,
or has `setuid` or `devices` on
is refused before the add writes a claim,
a slot,
a registration,
a snapshot or a clone,
and a container changed while the add runs is caught after the clone,
where the refusal names the clone and the container rather than leaving the mount wait to time out.
Worked example:
a container repointed at `mountpoint=/srv/x` after the last `doctor` run.
The preflight refuses and names the container and the command that fixes it.
If the container is repointed between the preflight and the clone,
the read after creation reports the clone's value and its source,
and the add says what it left.

What it costs:
two reads of replies the add already gets,
and a second refusal sentence to keep true.

#### Preflight only

What it gives:
the stronger of the two refusals,
and the smaller change:
one reader call on the container's reply before anything is written.

What it costs:
a container changed during the add is not seen by it,
so the add records a clone the sweep will refuse,
and the person learns it from the sweep rather than from the add.

Why it ranks second:
it covers the case that exists today (a hand-made container,
no shipped setup)
and leaves only the wide window open,
but it leaves that window with no answer at all.

#### Read the clone after creation only

What it gives:
one place to look,
after the clone exists,
and a report that names the property,
its value and its source.

What it costs:
it can only detect.
The add issues no destroy request
(`add_grants.rs:52` to `:67`,
and `source_guard/add_route_issues_no_destroy_tests.rs` holds that),
so a wrong clone stays until plan step 14's settle,
and the person pays a claim,
a slot,
a registration,
a snapshot and a clone
before learning the container was wrong.

Why it ranks last:
it refuses nothing,
and the case it fails on is the only case the check exists for.

### What each choice changes in the records

C changes no decision record:
it corrects `SPEC.md:5036` to `:5037` and the comment at `add_zfs.rs:103` to `:109`,
and adding the check changes the add's preflight and nothing else.

B amends the accepted handler record
(`doc/decision/cli-git-zfs-worktree-design.md:40`),
which today gives the handler mount and unmount and no property write,
and it changes what `SPEC.md`'s sweep check is for
("`mountpoint` is `legacy`,
so neither `zfs mount -a` nor the systemd mount generator mounts it first",
`:7429`).

A amends the design's "Scoped delegation" list
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:349` to `:350`)
and the reason "Dataset and path layout" gives for it
("An ordinary clone mountpoint would reopen the early-mount hazard even if only root could set it",
`:288` to `:289`),
and it amends `FORBIDDEN_PERMISSIONS`
(`package/git-policy/cli/src/native/health/doctor_zfs_route/delegation_checks.rs:90`).

## What the issue found, and what it left open

The issue's probe ran in a guest on OpenZFS 2.4.4 as the account,
with the nine decided grants.
Its transcript records this:

```text
$ zfs clone <checkout>@probe <container>/bare
filesystem successfully created, but it may only be mounted by root
$ zfs clone -o mountpoint=legacy <checkout>@probe <container>/with-property
cannot create '<container>/with-property': permission denied
```

The transcript puts one line under the wrong command.
The "filesystem successfully created" line belongs to the bare clone,
which succeeded.
The refused request prints only the denial.
The earlier round's logs show both:
`mountpoint-arm-two-2026-11-25.log` line 35 holds the line under the bare clone,
and line 42 holds the denial alone under the refused one.
That line prints from `zfs`'s own delegation check,
with no mount attempt
(`cmd/zfs/zfs_main.c:800` to `:804`).

Two claims follow from the issue,
and they are not the same claim.
The account cannot create a clone that carries `mountpoint` at creation.
That is measured,
and this round's guest re-measures it.
A clone without the property "inherits the origin's `mountpoint` (the checkout path) and `canmount=on`",
which is the issue's stated reason the property is needed at creation.
That sentence is false,
and the next two sections say why.

## How the kernel refuses the property

`zfs clone -o <prop>=<value>` sends one `ZFS_IOC_CLONE` request whose input carries a `props` nvlist.
The kernel creates the dataset first and applies the properties after
(`module/zfs/zfs_ioctl.c:3860` to `:3872`,
OpenZFS `zfs-2.4.4`):

```c
	error = dsl_dataset_clone(fsname, origin_name);

	/*
	 * It would be nice to do this atomically.
	 */
	if (error == 0) {
		error = zfs_set_prop_nvlist(fsname, ZPROP_SRC_LOCAL,
		    nvprops, outnvl);
		if (error != 0)
			(void) dsl_destroy_head(fsname);
	}
	return (error);
```

`zfs_set_prop_nvlist` validates every property through `zfs_check_settable`,
whose default arm calls `zfs_secpolicy_setprop` (`:5183`),
and that function asks for the property's own name as a delegation permission
(`:605` to `:659`,
the ask at `:658`):

```c
	return (zfs_secpolicy_write_perms(dsname, zfs_prop_to_name(prop), cr));
```

So `-o mountpoint=legacy` needs the `mountpoint` delegation on the dataset being created,
which is the clone.
Two grants reach a not-yet-existing clone.
A descendant grant on the container does,
because `dsl_deleg_access_impl` checks the dataset's own local grants
and then every ancestor's descendant grants (`module/zfs/dsl_deleg.c:574` to `:589`).
A creation-time grant does too,
because `dsl_dataset_create_sync` copies the parent's creation-time permissions
onto the new dataset,
for its creator,
as a local grant
(`module/zfs/dsl_dataset.c:1340`,
`dsl_deleg_set_create_perms` at `module/zfs/dsl_deleg.c:728`;
`man/man8/zfs-allow.8:300` to `:305`:
"These permissions are granted locally to the creator of any newly-created descendent file system").
Those are A's two shapes,
and they fail for the same reason.

The clone request's own policy is cheaper.
`zfs_secpolicy_create_clone` asks for `clone` on the origin,
`create` on the parent,
and `mount` on the parent (`module/zfs/zfs_ioctl.c:1128` to `:1153`).
All three are among the nine decided grants,
so a clone without properties passes.

The refusal's cleanup is not a guarantee.
The kernel discards `dsl_destroy_head`'s result (`:3868` to `:3869`),
so "the refused request left no clone" is what one release was measured to do,
never something the code promises.
The design already says failed cleanup is not proof of absence
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:395`,
"Add and Git registration" item 3).

## Where a clone's properties actually come from

Three independent readings say the clone inherits from its new parent,
the clone container,
and not from its origin.

- A clone's dataset record points at the target's directory,
  not the origin's (`module/zfs/dsl_dataset.c:1200`,
  `dsphys->ds_dir_obj = dd->dd_object;`).
  That line sits inside `dsl_dataset_create_sync_dd` at `:1177` to `:1291`,
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

Two shipped sentences disagree with all of it.
`package/git-policy/cli/SPEC.md:5036` to `:5037` says "a clone inherits the origin's mountpoint",
and the comment in `clone_the_origin` at `add_zfs.rs:103` to `:109` says the same.
Both are corrected by every answer here except A,
whose local property makes the origin's value irrelevant but still not inherited.

## What the design forbids, and why, in the record's words

The design names eight permissions the account must not hold:
"Do not grant `mountpoint`,
`canmount`,
`setuid`,
`devices`,
`rename`,
`promote`,
`receive` (including `receive:append`) or permissions over the whole home tree"
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:349` to `:350`).
The build's list has nine entries,
and the ninth,
`allow`,
is the build's own addition:
`FORBIDDEN_PERMISSIONS` in
`package/git-policy/cli/src/native/health/doctor_zfs_route/delegation_checks.rs:90`,
whose module doc gives the reason (`:76` to `:84`):
they are "permissions whose presence would let the account change the lineage the removal proof rests on,
or change what a mount exposes".

The design's sentences on the two that matter here follow.

- `mountpoint`:
  "An ordinary clone mountpoint would reopen the early-mount hazard even if only root could set it"
  (`doc/planning/cli-git-zfs-worktree-design-proposal.md:288` to `:289`).
  The mount study keeps it:
  "any future change that gives a clone a real mountpoint re-opens the early-mount hazard"
  (`cli-git-zfs-clones-mounted-under-the-checkout.md`,
  "What the design needs",
  item 4).
  The hazard is measured there:
  a clone was mounted at a path inside the checkout before the checkout's own mount,
  the checkout's later mount covered it,
  the clone became unreachable by path,
  and it blocked the pool's export.
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
  The earlier round's logs show both properties inheriting into the clone's mount flags.
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

One measured fact about this list matters to the options.
`zfs-allow(8)` says the Linux delegation support excludes "`mount`,
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

## The options

Each option is designed until it fails.
Each is judged on what it delivers for the add's safety and correctness.
The effort to build is a cost beside that,
never a ranking reason.

### A: grant `mountpoint`, on the container's descendants or at creation

A has two shapes,
and both keep the add sending `-o mountpoint=legacy`.

The first shape grants it on the container's descendants:
`zfs allow -d -u <account> mountpoint <container>`.
The second grants it at creation:
`zfs allow -c mountpoint <container>`.
The kernel copies a creation-time grant from every ancestor onto the new dataset,
for the account that created it,
as a local grant (`module/zfs/dsl_dataset.c:1340`),
so the second shape leaves the account holding `mountpoint` on every clone it makes,
for as long as the clone exists,
not only while it is being created.

What either shape delivers:
the clone's property is a local value the account wrote,
so no reader depends on inheritance,
and the request goes through whatever the container's own `mountpoint` is.

Where it fails:

- it deletes one entry of a list whose stated purpose is the removal proof and the mount exposure
  (`delegation_checks.rs:76` to `:85`),
  and the design's threat boundary is the absence of exactly these grants;
- the same grant lets the account aim its own clone's mountpoint anywhere,
  including inside the checkout,
  and a root-run `zfs mount -a` at boot then mounts it there.
  The earlier round's first shape is measured at
  `mountpoint-arm-2026-11-25.log:129` to `:147`:
  the account set `/home/cloneprobe-checkout/.claude/worktrees/auto`,
  root's ordinary scan mounted the clone on the checkout's own mount (mount 70 on mount 52),
  and a second aim at an arbitrary path outside the checkout was mounted the same way.
  This round's guest measures the second shape,
  and the same root mount with `overlay` on
  over a directory root had already filled (see `## The guest probe`);
- it cannot be narrowed.
  Delegation has no per-value or per-name scope,
  so a descendant grant on the container reaches every clone of that container,
  and a creation-time grant stays on each clone its creator made;
- nothing checks it.
  `doctor`'s forbidden-permission finding skips creation-time grants
  (`delegation_checks.rs:359`,
  the `Who::Create` arm),
  so the second shape is invisible to `doctor` and to the add's preflight,
  which runs the same check (`add_grants.rs:199` to `:270`).
  That gap is its own issue,
  named in `## The `doctor` gap this decision leans on`.

The exposure is a root mount of account-controlled content at an account-chosen path.
The exact case the mount study measured is worse than a nuisance:
the clone lands inside the checkout,
the checkout's later mount covers it,
and the pool cannot be exported.
The case this round measured is the plainer one:
a root-owned,
non-empty directory at the path the account chose,
with `overlay=on` on the dataset,
and root's `zfs mount -a` covering that directory's contents.
`nosuid,nodev` on the mount do not prevent an executable file inside it.

`doctor`'s `permission` finding would also have to be taught an exception,
which is the design change the issue names as crossing the boundary.

### B: clone bare, and have the root handler set `mountpoint=legacy` when it attaches

The add sends no property.
The handler runs `zfs set mountpoint=legacy <clone>` before it attaches,
as root,
which bypasses delegation.

What it delivers:
the same ending value as A without the forbidden grant,
and it works even if the container's own `mountpoint` is not `legacy`.

Where it fails:

- the handler's sweep already refuses a child whose `mountpoint` is not `legacy` before it attaches
  (`package/git-policy/cli/SPEC.md:7429`,
  and the refusal at
  `src/native/worktree_clone_mount_handler/sweep_clone_check.rs:253`).
  B therefore gives the sweep a write it does not have,
  before the check that exists to keep a clone out of the automatic mounts.
  That check's meaning becomes "the handler makes this true" instead of "the pool already proves this".
  A clone whose property is wrong but whose identity is otherwise right would be repaired rather than refused,
  which is a wider change than the clone request;
- it gives the mount handler a property write,
  and the accepted record gives that handler "own clone mount and unmount"
  (`doc/decision/cli-git-zfs-worktree-design.md:40`);
- it does not close the window.
  The set happens at attach,
  and the next boot's `zfs mount -a` runs before the handler.
  Under the issue's premise the clone would carry the checkout's own mountpoint through that window,
  which is worse than A,
  not better.
  Under the measured inheritance the clone is already `legacy` in that window,
  so B's set changes the value to the same value and buys nothing there;
- its cost is one root-side property write on a path that today writes nothing but the mount.
  That is a cost to state,
  not a reason to rank it lower.

### C: rely on the container's `mountpoint=legacy`, and set nothing at creation

The design's setup creates the container with `mountpoint=legacy`
(design,
"Dataset and path layout";
`SPEC.md`,
"The sweep's checks").
That setup is plan step 19's and is not built
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md:1363`,
"step 19's setup creates it";
the shipped hint for a neighbouring dataset says of this release that it
"carries no setup that creates it",
`src/native/start_probe/probe_mode_words.rs:93`).
So today the container exists because an operator or a fixture made it,
and `doctor` is the shipped check that reads it.
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

What it costs:

- the container's properties must be verified rather than assumed.
  `doctor` already checks exactly this:
  `check_dataset` requires the container's `mountpoint` to be `legacy` and its `canmount` to be `off`,
  and `check_clone_properties` requires every clone's `mountpoint` to be `legacy`
  with the message "a clone must inherit `legacy`"
  (`src/native/health/doctor_zfs_route/zfs_checks.rs:427` to `:449` and `:534` to `:561`).
  The add's own preflight does not run those checks:
  it reads identities and the delegation only (`src/native/worktree_clone_add_route/add_grants.rs`).
  So the guard exists and needs no new design,
  but the add does not consult it,
  and a container changed after the last `doctor` run would give every clone an ordinary mountpoint
  until someone ran it again.
  `## Choice 2: where C's check runs` asks which placement to give it;
- the check itself is cheap.
  The kernel's reply for one dataset carries every property with its value and its source,
  in the same object as its GUID
  (`package/zfs/request-to-openzfs-kernel-device/src/kernel_device/listed_object.rs:36` to `:47`,
  `DatasetIdentity`).
  The add already asks for that reply three times:
  the preflight reads the container's (`add_preflight.rs:142`),
  the clone's read follows the clone request (`add_zfs.rs:205` to `:208`),
  and the sweep reads `mountpoint` out of the same shape
  (`sweep_dataset_facts.rs:236` to `:252`,
  `value_of` at `:210` to `:230`).
  `read_identity` throws the properties away today;
  keeping them is the whole change,
  and `check_dataset` is public and already takes exactly those facts
  (`zfs_checks.rs:469`,
  re-exported at `health/doctor_zfs_route/mod.rs:163`).
  There is no new request type and no new check logic;
- without any check,
  the wrong container shows up as the mount wait timing out rather than as a refusal naming the container;
- two shipped sentences become false and must be corrected:
  the SPEC's "a clone inherits the origin's mountpoint"
  (`:5036` to `:5037`),
  and the code comment beside the request in `clone_the_origin`
  (`add_zfs.rs:103` to `:109`);
- a clone's property source reads `inherited from <container>` rather than `local`,
  so a reader must understand inheritance to see why it is safe.
  The guest prints both sources.

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

The window exists only in one situation,
and it is worth naming before the readings.
A clone sits between creation and attach when the add is interrupted or fails after the clone
and before the mount,
or when the wrapper or the machine stops between them.
Unmounted clones are a state the accepted design keeps,
including across reboots ("Unmounted,
pruned or moved worktrees").
In that state the actor is the next boot's `zfs mount -a`,
which runs before any handler does.

- `zfs mount -a` skips a dataset whose mountpoint is `legacy` or `none`,
  silently,
  in the automatic case
  (`cmd/zfs/zfs_main.c:7210` to `:7248`).
- The systemd mount generator ends unit generation for `legacy` and `none`
  (`etc/systemd/system-generators/zfs-mount-generator.c:430` to `:432`),
  and `zfs_is_mountable` returns false for both (`lib/libzfs/libzfs_mount.c:282` to `:297`).
- `zfs mount <clone>` refuses a legacy dataset with "cannot mount '<clone>':
  legacy mountpoint",
  which is the CLI's own check for an explicit mount
  (`cmd/zfs/zfs_main.c:7240` to `:7248`),
  before any helper runs.
  The helper's own `zfsutil` check (`cmd/mount_zfs.c:309`) is never reached.
- `mount -t zfs <clone> <dir>`,
  run by root,
  succeeds for a legacy dataset and fails for any other,
  by the helper's own check (`cmd/mount_zfs.c:320` to `:330`).
  The handler does not use this path.
  Its descriptor attach holds the objset by name and never reads `mountpoint`
  (`module/os/linux/zfs/zpl_super.c:849` to `:866`).

So under A,
between creation and attach,
a clone carries whatever mountpoint the account aimed it at,
and a boot's `zfs mount -a` mounts it,
with `overlay` on by default (`module/zcommon/zfs_prop.c:528`,
`lib/libzfs/libzfs_mount.c:521` to `:528`),
so it mounts over a non-empty directory rather than refusing.
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
which the handler does not use.

## The guest probe

The measurements come from two rounds' guests,
both on the same release and the same recipe.

- The earlier round's guest,
  `zfs-clone-mountpoint-probe`,
  on OpenZFS 2.4.4-1 (`zfs-2.4.4-1`,
  `zfs-kmod-2.4.4-1`) and kernel 6.8.0-147-generic,
  with the account `ubuntu`,
  a file-backed pool `cloneprobe` and its second run's `cloneprobe2`,
  the design's two siblings,
  the nine decided grants,
  and the container at `mountpoint=legacy`,
  `canmount=off`,
  `setuid=off`,
  `devices=off`.
  Its logs are `mountpoint-arm-2026-11-25.log` (cited here as log one)
  and `mountpoint-arm-two-2026-11-25.log` (log two),
  both in
  `/home/user/temp/agent/zfs-add-clone-mountpoint-without-the-forbidden-grant-design-2026-11-25/keep/guest-logs/`.
  That pool's size was not recorded in either log,
  so this document claims none.
- This round's guest,
  `zfs-clone-mountpoint-controls-2026-11-25`,
  created and prepared through the same recipe
  (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`),
  ran the arms the review asked for,
  the creation-time grant arm,
  and the control that removes that grant.
  Its commands are kept in that round folder's
  `/home/user/temp/agent/zfs-clone-mountpoint-design-second-revision-2026-11-25/keep/guest-logs/`
  as `probe-commands-2026-11-25.txt`,
  and its output in three logs in the same folder.
  They ran through `mvm exec <guest> -- <command>` inside the guest,
  with `runuser -u ubuntu --` for the account's arms,
  against a 1 GiB file-backed pool `cloneprobe` at `/var/lib/cloneprobe.img`,
  on the same release and kernel the earlier round used.

### The setup, before any clone (log one)

```text
cloneprobe/ROOT/clones  mountpoint  legacy      local
cloneprobe/ROOT/clones  canmount    off         local
cloneprobe/ROOT/clones  setuid      off         local
cloneprobe/ROOT/clones  devices     off         local
```

### C: a bare clone as the account, and what it inherits (log two)

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

### C's premise: the account cannot re-aim the clone, measured this round

The gate review named the arm the earlier round did not run:
the account,
holding the nine decided grants and no `mountpoint`,
tries to move its own clone's mountpoint and to inherit it away.
This round's guest ran it,
and it refused both.

Its clone was created by the same bare request the add will send:

```text
filesystem successfully created, but it may only be mounted by root
bare-clone exit=0
```

Then the account tried to move the clone's mountpoint,
and to inherit it away:

```text
cannot set property for 'cloneprobe/ROOT/clones/negctl': permission denied
set-mountpoint exit=255
cannot set property for 'cloneprobe/ROOT/clones/negctl': permission denied
set-legacy exit=255
cannot inherit mountpoint for 'cloneprobe/ROOT/clones/negctl': permission denied
inherit-mountpoint exit=1
```

The three refused commands were `zfs set mountpoint=/home/negctl-aimed <clone>`,
`zfs set mountpoint=legacy <clone>` and `zfs inherit mountpoint <clone>`,
each run as `ubuntu` through `runuser`.
The clone still read:

```text
mountpoint	legacy	inherited from cloneprobe/ROOT/clones
canmount	on	default
setuid	off	inherited from cloneprobe/ROOT/clones
devices	off	inherited from cloneprobe/ROOT/clones
```

The positive control on that same clone succeeded:
`zfs set org.monochromatic:desired-state=v1` exited 0 and read back `v1`,

```text
userprop exit=0
org.monochromatic:desired-state	v1
```

so the refusals are about `mountpoint`,
not about the clone being unreadable or the account being wrong.
Root's `zfs mount -a` then exited 0 and mounted nothing:
the container and the clone each read `mounted no`,

```text
cloneprobe/ROOT/clones	legacy	no
cloneprobe/ROOT/clones/negctl	legacy	no
```

The transcript is in that round folder's
`/home/user/temp/agent/zfs-clone-mountpoint-design-second-revision-2026-11-25/keep/guest-logs/`
as `setup-and-negative-control-2026-11-25.log`.

### The separating control: a container with an ordinary mountpoint (log one)

A second container was created at `mountpoint=/home/cloneprobe-other`,
and a bare clone made in it as the account:

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
The control's own `setuid` and `devices` read `on default`,
which is the same under either reading of inheritance,
so it is the legacy container's clone that shows those two following the parent:
`setuid off inherited` and `devices off inherited` in
`### C: a bare clone as the account, and what it inherits (log two)`.

### A with the descendant grant: works, and hands the account an automatic mount (log one)

After `zfs allow -d -u ubuntu mountpoint <container>`,
the account's `zfs clone -o mountpoint=legacy` succeeded:

```text
arm-a-granted exit=0
cloneprobe/ROOT/clones/granted  mountpoint  legacy  local
```

Then,
as the account:

```text
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
A second aim at an arbitrary path outside the checkout was mounted the same way at `/home/cloneprobe-aimed`
(mount 70,
parent 29).
The same scan mounted `cloneprobe/ROOT/other/plain`,
the positive control that a clone with an ordinary mountpoint is exactly what `zfs mount -a` picks up.

### A's second shape: the creation-time grant, measured this round

`zfs allow -c mountpoint <container>` names no account:
the guest's `zfs` refused `zfs allow -c -u ubuntu mountpoint <container>`
with "invalid options combined with `-c`" and exit 2,
so a creation-time permission applies to whoever creates the dataset.
The kernel copies it from every ancestor onto the new dataset,
for that creator,
as a local grant
(`module/zfs/dsl_dataset.c:1340`,
`dsl_deleg_set_create_perms` at `module/zfs/dsl_deleg.c:728`;
`man/man8/zfs-allow.8:300` to `:305`).
This round's guest granted it,
then asked the question the add asks:

```text
=== create-time grant of mountpoint on the container ===
allow-create exit=0
---- Permissions on cloneprobe/ROOT/clones ---------------------------
Create time permissions:
	mountpoint
Local permissions:
	user ubuntu create
```

and the same clone request the add sends today,
as the account:

```text
=== the same clone request the add sends today, as the account ===
filesystem successfully created, but it may only be mounted by root
clone-with-property exit=0
mountpoint	legacy	local
canmount	on	default
setuid	off	inherited from cloneprobe/ROOT/clones
devices	off	inherited from cloneprobe/ROOT/clones
```

The clone the account made holds the permission on itself afterwards:

```text
---- Permissions on cloneprobe/ROOT/clones/creategrant ---------------
Local permissions:
	user ubuntu mountpoint
```

The control is the same request either side of the grant.
`zfs unallow -c mountpoint <container>` put it back to
`cannot create 'cloneprobe/ROOT/clones/creategrant2': permission denied`,
exit 1,
and `zfs allow -c mountpoint <container>` again put it back to exit 0.
So the acceptance is the creation-time grant,
and nothing else.

With that local permission in hand the account aimed the clone at a path it chose,
a directory root already owned and had filled:

```text
=== a root-owned, non-empty directory at a path the account picks ===
mkdir exit=0
touch exit=0
root:root:755:/home/creategrant-aimed
total 8
drwxr-xr-x 2 root root 4096 Nov 25 22:26 .
drwxr-xr-x 6 root root 4096 Nov 25 22:26 ..
-rw-r--r-- 1 root root    0 Nov 25 22:26 root-owned-file
```

```text
=== the account sets its own clone mountpoint there ===
cannot mount 'cloneprobe/ROOT/clones/creategrant': Insufficient privileges
property may be set but unable to remount filesystem
aim exit=0
mountpoint	/home/creategrant-aimed	local
on
```

The two lines before `aim exit=0` are the CLI's own remount attempt inside the same command:
the property is set,
and the mount is root's,
so the command still exits 0.
The last value is `overlay`,
which reads `on`.
Root's ordinary boot scan then mounted it there:

```text
=== roots ordinary boot scan, with overlay on ===
zfs-mount-a exit=0
71 29 0:48 / /home/creategrant-aimed rw,nosuid,nodev,relatime shared:309 - zfs cloneprobe/ROOT/clones/creategrant rw,xattr,noacl,casesensitive
root:root:755:/home/creategrant-aimed
total 5
drwxr-xr-x 2 root root    2 Nov 25 22:24 .
drwxr-xr-x 6 root root 4096 Nov 25 22:26 ..
```

The directory's `root-owned-file` is not in the mount's listing,
so `overlay=on` let root's mount cover a non-empty directory rather than refuse it
(`lib/libzfs/libzfs_mount.c:521` to `:528` skips the emptiness check when overlay is set),
and the mount's `nosuid,nodev` are the container's inherited properties.
The transcript is in that round folder's
`/home/user/temp/agent/zfs-clone-mountpoint-design-second-revision-2026-11-25/keep/guest-logs/`
as `creation-time-grant-2026-11-25.log`.

### Between creation and attach (log two)

From log two:

```text
zfs-mount-a exit=0
cloneprobe2/ROOT/clones/bare   legacy  on  no
cloneprobe2/ROOT/clones/bareu  legacy  on  no
```

`zfs mount <clone>` refused,
in log one:
`cannot mount 'cloneprobe/ROOT/clones/bare': legacy mountpoint`,
exit 1
(log one lines 209 to 210).
Root's `mount -t zfs <clone> /home/cloneprobe2-attach` succeeded,
and the mount table showed `rw,nosuid,nodev`,
the container's `setuid=off` and `devices=off` carried into the mount flags.

### B: root sets the property at attach (log one)

Root's `zfs set mountpoint=legacy <bare>` exited 0.
It changed the source from `inherited from cloneprobe/ROOT/clones` to `local`,
with the value `legacy` before and after.
On the measured inheritance the write is a no-op in value.

## Ranking

**C > B > A.**

C ranks first for four reasons.
The sweep's check exists to refuse a clone whose property is wrong
(`SPEC.md:7400` to `:7431`),
and B turns that refusal into a repair.
B gives the mount handler a write the accepted record does not
(`doc/decision/cli-git-zfs-worktree-design.md:40`).
C's inherited value covers all three properties the sweep checks,
while A's local `mountpoint` covers one of them and leaves `setuid` and `devices` inherited anyway
(`SPEC.md:7430` to `:7431`).
And B's repair arrives after the clone exists,
so the clone stands wrong through the window the next boot's `zfs mount -a` runs in,
where C's preflight check refuses before the clone exists at all.

B ranks second,
above A and below C,
because it keeps the forbidden list intact and ends at the same value C has from creation.
Its costs are real and stated:
a root-side property write on the attach path,
and a sweep check whose refusal becomes a repair.
It also delivers one thing C does not:
it keeps working when the container's own `mountpoint` is not `legacy`.
That is not a reason to rank it above C,
because a container that is not `legacy` is the state C's preflight refuses,
and a container the setup builds that way is the design's premise rather than a case to tolerate.

A ranks last because it is the only answer that deletes a line of the forbidden list,
and it delivers what C lacks only as a local value:
a per-clone pin that survives a later change of the container.
That pin is bought with a permission that also lets the account aim root's own boot mount
at any path it chooses,
measured twice,
and no check would notice A's second shape today.
C answers the same worry by refusing a container that is not `legacy`,
so the surviving-change case becomes a refused add rather than a wrong clone that still attaches.

## What changes if the owner picks C

- `package/git-policy/cli/SPEC.md:5036` to `:5037`:
  the clone is made with no `mountpoint` property,
  because it inherits the container's,
  and the sentence that says it inherits the origin's is corrected.
- `add_zfs.rs`,
  `clone_the_origin`:
  no property list,
  and the comment says what the pool actually does.
- The design proposal,
  "Add and Git registration" item 3,
  needs no edit:
  it already says to request `mount` "via the local desired-state user property ... not via `mountpoint`"
  (`:390` to `:392`),
  and its "Send it in clone `props` only if the delegated creation path is verified" is about that user property,
  not about `mountpoint`.
  No sentence there changes.
- The design proposal's "Dataset and path layout" needs no edit either:
  `:283` already says "Clones must inherit `mountpoint=legacy`".
- Tests,
  in the guest:
  the negative control this round ran (the account's `zfs set mountpoint` and `zfs inherit mountpoint` refused,
  `userprop` accepted);
  a container with an ordinary mountpoint,
  which the preflight's new check refuses;
  and,
  if the owner picks a check,
  the container changed between the preflight and the clone,
  which the read after creation reports.

## The repository's own tests of C, and the runs on record

The earlier proposal said the add was not run as the account,
so the probe used the `zfs` CLI.
That is true of the add's binary,
and it is not true of the route's tests,
which already run C's path in a guest.

- `package/git-policy/cli/src/native/binary/binary_zfs_sweep_guest_tests.rs:730` to `:774`,
  inside the test at `:501`:
  it creates a legacy container,
  has the account clone with no `-o`
  (`runuser -u ubuntu zfs clone ...`),
  writes the mount request,
  and asserts the shipped handler attached the clone at its slot.
  A recorded run exists,
  twice,
  on 2026-11-25:
  `/home/user/temp/agent/sweep-guest-arms-find-a-clone-by-its-whole-name-2026-11-25/keep/guest-sweep-arm-first-run.log`
  and `guest-sweep-arm-second-run-after-the-cleanup.log`,
  with `STOPPAGE account-clone` at line 1383 of the driver log
  and `test result: ok. 1 passed; ... finished in 194.76s` at line 1406,
  both after the test file's last change (`c3c9830585`,
  2026-11-25T05:30:59-05:00).
- `package/zfs/request-to-openzfs-kernel-device/src/kernel_device/live_account_tests.rs:356` to `:392`:
  the account clones through the device itself with a creation-time user property and reads it back.
  `:423` to `:449` clone the same name again with `None`.
  A recorded run exists:
  `/home/user/temp/agent/zfs-step-6-recheck-findings-fixed-2026-11-20/keep/guest/account-suite.log:107`
  and `:140`,
  dated 2026-11-20 20:41,
  after that file's last change (2026-11-20T01:18:41-05:00).
- `live_write_tests.rs:149` to `:174`:
  a refused creation request creates nothing (as root,
  `E2BIG`).
  A recorded run exists:
  `/home/user/temp/agent/gate-recheck-zfs-step-6-after-its-findings-2026-11-20/keep/guest/root-suite.log:790`.

Together they close the CLI-versus-ioctl gap the earlier proposal left open,
and they fill the "handler's real attach" item with a measured run.
What they still do not cover is the container that is not `legacy`,
which is the preflight check's case,
and the ungranted re-aim arm,
which this round's guest ran (see `## The guest probe`).

## The `doctor` gap this decision leans on

C is safe only while the account holds no `mountpoint` and no creation-time `mountpoint`.
`doctor` refuses a forbidden grant that the account or everyone holds,
but its filter skips `Who::Create` and `Who::NamedSet` (`delegation_checks.rs:359`).
A named set carrying a forbidden permission still lands in the check's "unproved" answer,
because `is_route_permission` counts every forbidden name (`:537` to `:543`),
so it refuses rather than passes,
though without naming the permission (`:583` to `:589`).
A creation-time grant is read by nothing:
the check sees the entry and drops it,
and no other check reads that who type.
So `zfs allow -c mountpoint <container>` passes `doctor` today,
which is how A's second shape would go unnoticed,
and which also means C's premise has one unread corner.

That is outside this decision,
and it has its own issue:
[#6472](https://github.com/Aquaticats/Monochromatic/issues/6472),
"git-policy-cli:
doctor reads no create-time ZFS grant,
and its forbidden check skips named sets".
This round's guest measured that the creation-time grant is effective
(the section `### A's second shape: the creation-time grant, measured this round`),
so the issue is not hypothetical.

## The gate review's findings, and where each is answered

The review is
`/home/user/temp/agent/gate-review-zfs-clone-mountpoint-without-the-forbidden-grant-2026-11-25/keep/review.md`.
Each finding is named by its own words,
and each answer is in this document.

### High

- "The 'or' inside C,
  and its cost stated wrongly":
  the new-`ZFS_IOC_GET_PROP` claim is deleted.
  C's cost is now one reader call on replies the add already gets
  (`## Choice 2: where C's check runs` and `### C`).
  The fork inside C is its own flagged choice,
  with three placements and a recommendation.
- "The accepted design already says C,
  and the edit list misreads it":
  `## The answer in four lines` states it plainly,
  and
  `## What changes if the owner picks C` keeps only `SPEC.md:5036` to `:5037` and the comment at
  `add_zfs.rs:103` to `:109`.
  The item 3 edit is dropped,
  with the reason the review gave.
  The layout sentence needs no edit either,
  because `:283` already holds it.

### Medium

- "Two ranking reasons contradict the proposal's own text":
  the two untrue reasons are gone.
  `## Ranking` names B's and A's real extra,
  gives four reasons C still ranks first,
  and reads "C > B > A" with a reason for each adjacent pair.
- "The window's scenario is never stated,
  and its key negative control was not run":
  the section "What a person or `zfs mount -a` does with a clone before the handler attaches" opens
  with the interrupted-add-then-boot scenario,
  and `## The guest probe` records the account's refused `zfs set mountpoint` and `zfs inherit mountpoint`
  with its `userprop` positive control.
- "'So this is not an escalation' is unsupported":
  the sentence is gone.
  `### A` calls the exposure a root mount of account-controlled content at an account-chosen path,
  and this round's guest measures root's `zfs mount -a` with `overlay` on over a non-empty directory.
- "'The root setup already creates' the container is not true yet":
  `### C` now says the design's setup is plan step 19's and is not built,
  that an operator or a fixture makes the container today,
  that `doctor` is the shipped check reading it,
  and that this is why the check inside C exists.
- "The proposal misses the repository's own tests of C":
  `## The repository's own tests of C, and the runs on record` names the three,
  says which runs are recorded and when,
  and trims the list to what they do not cover.
- "C's safety rests on a grant that doctor checks incompletely":
  `### A` now gives A's second shape,
  the section "The `doctor` gap this decision leans on" names the gap and its issue,
  and the review's two "cannot be narrowed" claims are corrected in the same two places.

### Low

- Line mis-citations:
  `zfs_ioctl.c:5183` for the default arm's call,
  `:605` to `:659` for `zfs_secpolicy_setprop` with its return at `:658`,
  `dsl_dataset.c:1291` for the end of `dsl_dataset_create_sync_dd`,
  and `mount_zfs.c:320` to `:330` for the helper's check,
  all corrected in `## How the kernel refuses the property` and in the section
  "What a person or `zfs mount -a` does with a clone before the handler attaches".
- "`zfs mount <clone>` refuses because `zfs mount` passes `zfsutil`":
  corrected to the CLI's own check at `zfs_main.c:7240` to `:7248`,
  with the helper's `zfsutil` check at `mount_zfs.c:309` named as the one the CLI never reaches.
- The `zfs mount <clone>` refusal attributed to log two:
  now cited from log one,
  lines 209 to 210,
  which is where it is.
- The control's `setuid` and `devices` as evidence they follow the parent:
  `### The separating control` now says the control reads `on default` either way,
  and that the legacy clone's `inherited` lines are the evidence.
- "1 GiB" in neither log,
  and the probe script not kept:
  no pool size is claimed for the earlier round,
  and this round's commands and pool size are kept in `keep/guest-logs/probe-commands-2026-11-25.txt`.
  The `ubuntu$ zfs set ...` line the earlier proposal printed inside a block of log text is not repeated;
  the measured lines quoted here are copied from the logs.
- "nine" permissions the design forbids:
  `## What the design forbids, and why` now says the design names eight,
  and that `allow` is the build's own ninth,
  citing `delegation_checks.rs:90` and its module doc.
- The "filesystem successfully created" line under the refused `-o`:
  `## What the issue found, and what it left open` gives the line its own command
  and says the line comes from the delegation check with no mount attempt.
- "leaves no clone behind":
  `## How the kernel refuses the property` now says the kernel discards the cleanup's result,
  that one release left none in the measured case,
  and cites the design's own "failed cleanup is not proof of absence".
- "which the design keeps to mount placement and folder identity only":
  the uncited sentence is gone,
  replaced by the accepted record's own words
  (`doc/decision/cli-git-zfs-worktree-design.md:40`).
- Logs named by file name only:
  `## The guest probe` gives both logs' full paths and says where this round's logs sit.

## Open items

- The add's own binary was not run as the account,
  and the last open item names why:
  the wrapper's account companion refuses a bare `ubuntu` at `/proc/self/setgroups`,
  as the round that found the issue recorded.
  The route's guest tests and this round's arms narrow that gap.
- `zfs clone -o mountpoint=legacy` as the account was refused under the nine grants
  and succeeded once `mountpoint` was granted,
  descendant or at creation,
  which is the whole of A's delivery.
- The probe's pool is not encrypted,
  and the real checkout is.
  The property walk does not depend on encryption;
  the clone's key state does,
  and the design's item 5 covers that.
- Only OpenZFS 2.4.4 was read and measured.
  An older release's inheritance is not claimed here.
- Whether `zpool import` skips legacy datasets through the same `zfs_is_mountable` path as `zfs mount -a`
  was not read.
