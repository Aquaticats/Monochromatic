# The deferred-destroy checks of a removal

Non-final,
unapproved design proposal by an agent.
It is not approved by the project owner and nothing in it is final.
It designs the open item "How the deferred-destroy checks are designed" of the accepted ZFS worktree design
(`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
"Open items the decisions leave"),
the item the implementation plan lists as "step 15,
blocking,
writer design work"
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
"Open items and the step that needs each").
It makes no decision that the accepted design,
the proposal or the project's answers of 2026-11-19 did not make,
beyond the writer's choices "Choices this design marks for the project's veto" lists,
and the two questions "Questions only the project can answer" puts.

It read,
whole or in the passages named:
`doc/decision/cli-git-zfs-worktree-design.md`,
the proposal's "Binding crate and compatibility",
"Requests and input ownership",
"Removal and recovery",
"Cancelling a blocked removal",
"Project decisions" ("Origin cleanup authority",
"Blocked-removal cancellation",
"Prune protection while unmounted"),
"Clone removal identity protection",
"Bookmarks on a clone being removed",
"Open items the decisions leave",
"Forced removal and deferred origins",
"What doctor checks and prints" and "Acceptance tests the build still owes";
the implementation plan's steps 13 to 18,
"Open items and the step that needs each" and "Gaps the design's open-item list does not name";
`doc/decision/cli-git-zfs-staging-under-cache.md`,
`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
`doc/decision/cli-git-zfs-step-8-sweep-choices.md`;
`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
its opening and
"The open item and the answer it carries";
`package/git-policy/cli/SPEC.md`,
"What never discarding a person's data means",
"The descriptor mount of a ZFS worktree clone",
"The clone's mount intent,
receipts and readiness",
"### Doctor",
"Records of a worktree that is removed" and the pre-response credential passage;
the crate `package/zfs/request-to-openzfs-kernel-device`'s README,
`src/requests/`,
`src/kernel_device/` and `src/layout/`;
the code under `package/git-policy/cli/src/native/worktree_clone_mount_handler/`,
`worktree_removal_interrupted_destroy.rs` and `worktree_removal_finishes_an_interrupted_destroy.rs`.

It read OpenZFS at tag `zfs-2.4.4` (commit `71a9f95`),
cloned into this round's scratch folder with
`--depth 1 --branch zfs-2.4.4`,
in the files each fact names.
It read no Git and no Linux source:
nothing here claims a Git,
Linux or systemd behaviour.
It ran no ZFS or pool command that changes anything,
opened no `/dev/zfs`,
started no guest and no container,
took no measurement,
and edited no other file.
Every kernel sentence here is source reading.
Every measurement it owes stays owed,
and "Measurements owed" writes the probe that would run each one.

The one command it ran against the machine was read-only:
`ls --format=long --directory /dev/zfs`,
which answered
`crw-rw-rw- 1 root root 10, 249`,
so an account may open the device for the reads this design uses.

## The item and the sentences that bind it

The open item,
quoted whole from the proposal (lines 1725 to 1737):

> How the deferred-destroy checks are designed.
> The answers' words:
> the pair's "Keep both,
> check first
> (Recommended)",
> and "Also check first (Recommended)",
> which puts the same check before the first snapshot delete
> as well,
> so such a removal refuses before this run deletes anything,
> while the check before the head destroy
> stays.
> Where the checks read the planned clone and origin identities,
> how they read the origin's deferred-destroy
> mark,
> what each prints on refusal,
> and which refusals keep the removal record are undesigned.
> The check before the
> head destroy runs after the snapshot batch,
> so a refusal there leaves the clone without its own snapshots,
> and
> nothing says whether such a refusal keeps the removal record or lets the removal cancel itself.
> The window between
> the check before the head destroy and the destroy,
> which a same-account process can use to mark the origin,
> is the
> residual the delegated answer accepts rather than a gap this section asks anyone to close;
> the answer leaves a
> mark set between the two checks inside that same residual
> ([Clone removal identity protection](#clone-removal-identity-protection)).
> The refusal before the head destroy
> is one of the VM measurements the build owes
> ([Forced removal and deferred origins](#forced-removal-and-deferred-origins)).

The sentences that decide a point,
each quoted from the file and line named:

- `doc/decision/cli-git-zfs-worktree-design.md:45` to `:48`:
  "The check that the origin is not marked for deferred destroy also runs before the first snapshot delete,
  so such a removal refuses before this run deletes anything.
  The check before the head destroy stays,
  and a mark set between the two remains the accepted residual."
- `doc/decision/cli-git-zfs-worktree-design.md:57` to `:60`:
  "The two owed VM measurements still gate shipping,
  and they decide only the sentences already conditional on them:
  forcing the removal of a clone a holder keeps,
  and the refusal of a removal whose origin is marked for deferred
  destroy."
- The proposal,
  `:1416` to `:1420`:
  "Delegated origin `destroy` also lets the account mark a checkout snapshot for deferred destroy,
  so this decision
  is the one that pairs with clone removal identity protection;
  the project answered that pair
  "Keep both,
  check first (Recommended)",
  and on 2026-11-19 placed the check "Also check first (Recommended)":
  it
  runs before the first snapshot delete as well as before the head destroy,
  so a marked origin refuses before this
  run deletes anything,
  and a mark set between the two checks remains the accepted residual."
- The proposal,
  `:929` to `:932`:
  "Run the check the paired answer requires,
  "Keep both,
  check first (Recommended)",
  before the first snapshot
  delete and again before the head destroy:
  that the clone and its origin are the planned ones and that the origin
  is not marked for deferred destroy,
  refusing otherwise,
  because destroying the last clone can reap an origin
  already marked for deferred destroy outside the separate origin-cleanup step."
- The proposal,
  `:1517` to `:1522`:
  "Removal checks,
  before the first snapshot delete and again just before the head destroy,
  that the clone and its
  origin are the planned ones and that the origin is not marked for deferred destroy,
  refusing otherwise".
- The proposal,
  `:1527` to `:1528`:
  "that exposure is reasoning from the grant model (`lib/libzfs_core/libzfs_core.c:442` to `:451`,
  OpenZFS
  `zfs-2.4.4`),
  not a measured account-run deferred destroy".
- The proposal,
  `:933` to `:940`:
  "so a removal whose origin is marked refuses before this run deletes anything,
  and the clone,
  its remaining
  snapshots and the registration stay.
  A removal that resumes may already have lost the snapshots an earlier run
  destroyed,
  and no later refusal brings them back.
  ... The check before the head destroy runs after the snapshot
  batch,
  so a refusal there leaves the clone without its own snapshots:
  the head and the registration survive at
  that moment,
  the history does not come back,
  and whether the refusal then keeps the removal record or lets the
  removal cancel itself is under Open items the decisions leave."
- The plan,
  step 15,
  "What it builds":
  "The check before the first snapshot delete and again before the head destroy:
  the clone and its origin are the
  planned ones,
  and the origin is not marked for deferred destroy.
  What each refusal prints,
  and whether a refusal
  keeps the removal record."

Two further records bind every design of this route and are followed here:

- The threat model,
  `doc/decision/cli-git-zfs-staging-under-cache.md:221` to `:237`:
  "the account is trusted,
  and root and the account are not a security boundary here."
  "Where the design argues about what the account could do,
  it says whether the argument survives that sentence."
  The design says,
  for each argument it makes,
  whether that argument survives the sentence:
  the causes in "The refusals" and the keep-or-drop rule need no hostile account to matter,
  "The accepted residual" is a window inside one account,
  which the project's sentence treats as no boundary,
  and question Q2 asks the project about authority rather than about an attack.
- No configuration,
  `doc/decision/cli-git-zfs-step-8-sweep-choices.md:17` to `:24`:
  "The project first answered that everything must work by default with no configuration",
  and its answer of 2026-11-22,
  "1:
  no file needed (Recommended)".
  This design adds no file,
  no key and no setting:
  the check runs on every removal of a managed clone.

## What this design reuses

Nothing here is new machinery.
Each piece has an owner that already exists:

- The read:
  `ZFS_IOC_OBJSET_STATS`,
  its property list and the origin field,
  through
  `package/zfs/request-to-openzfs-kernel-device`,
  the crate the project chose,
  in the section that says
  "Use `package/zfs/request-to-openzfs-kernel-device`,
  with Cargo package
  `monochromatic-zfs-request-to-openzfs-kernel-device`"
  (proposal `:143` to `:145`,
  F22).
  That section also forbids the binding,
  the wrapper and the handler from invoking the `zfs` program
  (proposal `:151` to `:152`),
  so the mark is read by a request and never by a command.
  The crate's `ZfsDevice::objset_stats` already returns the identity,
  the origin and every property with its
  source (`package/zfs/request-to-openzfs-kernel-device/src/kernel_device/read_requests.rs:45` to `:59`).
- The device seam:
  `DatasetReader`,
  the trait whose docstring says
  "The trait is the seam a control drives with a scripted pool,
  because no control on a person's machine may open
  the device"
  (`package/git-policy/cli/src/native/worktree_clone_mount_handler/sweep_dataset_reader.rs:22` to `:28`).
- The property reader:
  `PropertyValue` and `value_of`,
  which already tell an absent property from an unreadable one
  and word both for a refusal
  (`package/git-policy/cli/src/native/worktree_clone_mount_handler/sweep_dataset_facts.rs:110` to `:152`,
  `:210` to `:225`).
- The refusal idiom:
  a typed enum with the fields of the finding,
  a `step()` word and a `message()` that ends
  "Nothing was mounted or recorded on its behalf" plus "Ways forward",
  with the commands a person runs
  (`.../sweep_clone_check.rs:206` to `:280`),
  and the checklist of what every refusal names in
  `package/git-policy/cli/SPEC.md:5834` to `:5840`.
- The durable record of a destroy a removal started:
  the Btrfs route writes one before the kernel call,
  in
  `monochromatic-git-policy-cli/recorded-destroys` under the common Git directory,
  named by the registration's id,
  and drops it when the object it named is still there
  (`package/git-policy/cli/src/native/worktree/worktree_removal_interrupted_destroy.rs:37` to `:52`).
- Lock-free publication when something is written:
  the slot identity store's `O_EXCL` staging file and one
  `linkat`,
  with no lock and no listing on a reader's path
  (`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
  "The design accepted").
  This design writes nothing,
  so it reuses the shape only as the rule it must not break.
- `doctor`:
  the read-only filesystem report,
  which already reports unmounted and staged clones,
  standing intent
  and the last verified loss (proposal `:1229` to `:1239`).

## Facts

Each fact is one claim a reviewer can check alone.
Nothing here was measured in this round.

F1.
The mark is a property named `defer_destroy`,
an index property with the values `yes | no` and default 0,
registered read-only for snapshots only
(OpenZFS `zfs-2.4.4`,
`module/zcommon/zfs_prop.c:544` to `:546`;
its enum member at
`include/sys/fs/zfs.h:151`).

F2.
Its value is computed,
not stored:
`dsl_get_defer_destroy` returns 1 when the dataset's `ds_flags` hold
`DS_FLAG_DEFER_DESTROY` and 0 otherwise
(OpenZFS `zfs-2.4.4`,
`module/zfs/dsl_dataset.c:2622` to `:2626`;
the macro at
`include/sys/dsl_dataset.h:76` to `:78`).

F3.
Every object-stats reply carries it,
added unconditionally for any dataset or snapshot
(OpenZFS `zfs-2.4.4`,
`module/zfs/dsl_dataset.c:2856` to `:2857`).

F4.
The entry is an nvlist holding a `value` and,
for a property whose default the kernel can report,
a `source`.
A read-only property that is not set-once has no default to report,
so `dodefault` answers `ENOENT` and the entry
carries `value` alone
(OpenZFS `zfs-2.4.4`,
`module/zfs/dsl_prop.c:44` to `:46` and `:1328` to `:1343`;
the key names at
`include/sys/fs/zfs.h:282` to `:283`).

F5.
`ZFS_IOC_OBJSET_STATS` fills that property list only when `zc_simple` is zero and a reply buffer is named
(OpenZFS `zfs-2.4.4`,
`module/zfs/zfs_ioctl.c:2244` to `:2252` and `:2265` to `:2270`),
and the crate leaves
`zc_simple` zero in a fresh command buffer
(`package/zfs/request-to-openzfs-kernel-device/src/layout/command_buffer.rs:109`,
`src/layout/zfs_cmd_t.rs:299`,
`src/requests/request.rs:60` to `:75`).

F6.
Reading it needs no privilege and no delegation:
the request is registered as a dataset read
(OpenZFS `zfs-2.4.4`,
`module/zfs/zfs_ioctl.c:7827` to `:7828`),
whose policy returns 0 in the global zone
(`:386` to `:394`).

F7.
A snapshot with clones cannot be destroyed without the defer mark:
the check refuses `EEXIST` when
`ds_num_children` is above one and `EBUSY` when user holds exist
(OpenZFS `zfs-2.4.4`,
`module/zfs/dsl_destroy.c:79` to `:86`),
while a deferred request returns 0 before either
test (`:68` to `:73`).

F8.
The mark is set in the destroy's sync step,
and only when the deferred request met a user hold or more than one
child,
which is what a snapshot with clones has
(OpenZFS `zfs-2.4.4`,
`module/zfs/dsl_destroy.c:322` to `:334`).
No code at `zfs-2.4.4` clears the flag:
the assignment at `:327` is the only one in the tree.

F9.
`ds_num_children` counts the snapshot itself plus its clones,
so a snapshot with exactly one clone reports two,
and the same number appears in the reply as `dds_num_clones`
(OpenZFS `zfs-2.4.4`,
`module/zfs/dsl_dataset.c:2635` to `:2639` and `:2896`).
The crate carries it as `DmuObjsetStats::dds_num_clones`
(`package/zfs/request-to-openzfs-kernel-device/src/layout/zfs_cmd_t.rs:75`).

F10.
Destroying the last clone's head destroys a marked origin in the same transaction:
the sync step computes
`rmorigin` from a clone directory,
the flag on the previous snapshot,
`ds_num_children == 2` and no user refs
(OpenZFS `zfs-2.4.4`,
`module/zfs/dsl_destroy.c:1028` to `:1031`),
and then destroys that snapshot
(`:1169` to `:1173`).
The head's own check does not refuse for the mark:
it refuses only when that origin is long-held
(`:799` to `:806`).

F11.
The crate can already read the property list of any dataset or snapshot in one request,
and already asserts
the mark on an unmarked dataset in a guest-only control
(`package/zfs/request-to-openzfs-kernel-device/src/kernel_device/live_read_tests.rs:120` to `:154`,
`live_account_tests.rs:87`).
It has no typed accessor for the mark:
nothing outside those tests names `defer_destroy` for reading,
and the
mark it writes is the command field `zc_defer_destroy`
(`src/requests/dataset_requests.rs:48` to `:57`,
`src/layout/zfs_cmd_t.rs:196` to `:197`,
`:334` to `:345`).

F12.
The crate's `DESTROY_SNAPS` request omits the `defer` key by construction,
and a control asserts that neither
the request nor its bytes name the key
(`package/zfs/request-to-openzfs-kernel-device/src/requests/snapshot_requests_tests.rs:58` to `:78`).
So the snapshot batch of a removal cannot set the mark on the clone's or the origin's snapshots.

F13.
The wrapper already turns one property of a reply into a value or a described absence,
and refuses rather than
treating an unreadable property as a default
(`package/git-policy/cli/src/native/worktree_clone_mount_handler/sweep_dataset_facts.rs:210` to `:225`,
`.../sweep_clone_check.rs:247` to `:251`).

F14.
The Btrfs route already keeps a durable record of a destroy it started,
and drops it when the destroy did not
happen
(`package/git-policy/cli/src/native/worktree/worktree_removal_interrupted_destroy.rs:37` to `:52`).
Its record is read at a bound and its presence proves an attempt,
never a loss.

F15.
The ZFS removal record and the persisted destruction plan have no home,
no writer and no format yet;
the plan
names both as gaps,
and sibling designs own them
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
"Gaps the design's open-item list does not name").
The proposal states the fields the binding record must carry,
which the plan's tuple is drawn from
(proposal `:414` to `:421`).

F16.
`zfs` and `libzfs` refuse to set a read-only property before they send any request
(OpenZFS `zfs-2.4.4`,
`cmd/zfs/zfs_main.c:517`,
`lib/libzfs/libzfs_dataset.c:1969`).
The kernel's own refusal for a raw `SET_PROP` naming a read-only property was not traced,
so nothing here claims
that the flag has exactly one setter.

F17.
The mark's read is a read of the origin snapshot by the name the plan records;
a rename of that snapshot,
or a
promotion that moves it to another dataset,
changes the name the read must use,
so a live name that differs from
the planned one is a disagreement this design refuses.

F18.
Two `OBJSET_STATS` reads are the whole kernel traffic of one check:
one for the clone,
one for the origin
snapshot,
and the mark rides in the second reply (F3,
F5).

F19.
The wrapper's `doctor` already reports standing intent,
staged and unmounted clones,
and the last verified
loss,
and it reports the deferred-origin check "as not yet designed"
(proposal `:1229` to `:1239`;
plan,
step 18,
"The doctor clauses of the removal bullets").

F20.
A removal of a managed clone refuses rather than falling back to a route that skips a proof:
the design's gate
"does not authorize an unrelated mount or cleanup;
keep registrations and durable recovery state intact"
(proposal,
"ABI gate and tests").

F21.
The same reply carries the origin snapshot's `createtxg` and its `userrefs`
(OpenZFS `zfs-2.4.4`,
`module/zfs/dsl_dataset.c:2842` to `:2843` and `:2854` to `:2855`),
so the numbers the reap condition turns on arrive with the mark,
and the refusal can say whether this removal is the transaction that would reap it (F10).

F22.
The binding the project chose is the one the proposal's own words name:
"Use `package/zfs/request-to-openzfs-kernel-device`,
with Cargo package
`monochromatic-zfs-request-to-openzfs-kernel-device`"
(proposal `:143` to `:145`),
and "Do not add a C build step,
link or load OpenZFS's CDDL libraries,
or invoke the `zfs` program from the binding,
wrapper or handler"
(proposal `:151` to `:152`).
The phrases "our own small binding over /dev/zfs" and "no crate that runs zfs" are headings of the handover record,
not text of that section
(`doc/handover/cli-git-rust-implementation.md:43949` and `:43955`),
so nothing here quotes them as the proposal's.

## Assumptions

A1.
The persisted destruction plan carries the clone's name and GUID,
and the origin snapshot's name,
GUID and
`createtxg`,
at the moment the pre-batch check runs,
because step 14 persists the plan after proving them
(proposal `:414` to `:421`,
plan step 14).
The plan's home and format are a sibling's;
if its format cannot carry a field this design needs,
the check reads
that field from the binding record the add wrote,
and "Dependencies on sibling designs" says so again.

A2.
The removal record is written before the unmount request the plan's step 2 makes,
so a check can decide whether
a refusal keeps or drops it (proposal `:896` to `:902`,
plan step 14).
The record's home and format are a sibling's.

A3.
The removal's kernel calls sit behind a seam a control can drive without the device,
as the sweep's reads do
(F11,
and the trait at `sweep_dataset_reader.rs:22` to `:28`).
Step 14 and step 16 own that seam;
this design states what it needs of it and nothing more.

A4.
A check runs before the snapshot request only when this run sends a snapshot request,
and always before a head
request.
The reasoning is under "One check,
called at two places".

A5.
Reading an origin snapshot's properties needs no loaded key,
because the sweep reads `keystatus` from the same
reply for a clone whose key may be unloaded and refuses on it
(`package/git-policy/cli/SPEC.md:5959` to `:5961`).
The origin snapshot of an encrypted checkout is the case this assumes for.

A6.
A same-account process can mark the origin between any two requests,
and no read or name-based request binds a
later delete,
which [VM item 12][vm-12] measured for a clone destroy
(proposal `:1540` to `:1543`;
OpenZFS `zfs-2.4.4`,
`module/zfs/zfs_ioctl.c:2244` to `:2300`,
`:4105` to `:4150`).

## Limits

L1.
Nothing in this design was run.
Every sentence about the kernel,
the crate's live answers or a removal's
behaviour in a guest is source reading,
and the measurements under "Measurements owed" are unpaid.

L2.
The check refuses on the mark alone,
which is stricter than the kernel's reap test (F9,
F10):
a clone with a
second clone pinning the origin is refused too,
although destroying it would not reap the marked snapshot.
That strictness is the decided sentence's ("the origin is not marked for deferred destroy"),
and it is a cost,
not
a gap this design can close on its own.

L3.
The mark has no undo at `zfs-2.4.4` (F8),
so a refusal's way forward is a person's decision about that
snapshot,
never a wrapper command.

L4.
The check's answer is not bound to the request it precedes;
a same-account process can substitute objects or set
the mark in that window,
and that is the accepted residual,
stated under "The accepted residual".

L5.
The reap sentence a refusal prints comes from `dds_num_clones` and `userrefs` in the same reply (F9,
F21).
It explains the kernel's reap condition;
it does not change the decision.

L6.
Whether a raw `SET_PROP` naming a read-only property is refused by the kernel was not traced (F16).
The check
reads `value` and ignores `source`,
so a stored local property of that name could not change what it reads (F4),
but this design claims no more than that.

L7.
The check adds two ioctls per call.
It ships no limit,
no timeout and no cap,
so nothing here needs a workload
measurement under rule `LQM`;
the cost has to fit the removal's own deadline,
which is sized elsewhere and
unmeasured.

L8.
The exit status and the exact wording of a refusal that a person sees are proposed here and unverified against
the built binary,
and the wrapper's refusal status for a removal is step 14's.

## The design

### One check, called at two places

One function,
`check_planned_origin`,
with one input and one verdict:

- the input is the planned tuple A1 requires,
  the stage,
  and two identity reads (F18);
- the verdict is `Proceed`,
  or `Refused` carrying one typed cause from "The refusals".

It is called at the removal's end of step 2,
at the place the plan calls "before the first snapshot delete",
and
again in step 3 immediately before the head destroy request
(proposal `:929` to `:932`;
plan step 15,
step 16).

The first call site runs when this run is about to send the snapshot request.
When the remaining planned snapshot
set is empty the request is skipped (proposal `:918` to `:919`),
and with it the call,
because no snapshot of this
run is deleted:
the guarantee the project's answer buys,
"refuses before this run deletes anything",
is kept by the
head check that still runs.
This is a writer's reading of the decided sentence's scope,
marked for veto as V1.
It is also what makes the refusal after the batch reachable at all:
a resumed removal whose batch is done is the
state the open item describes,
and it is the state "Measurements owed" probes.

Neither call site can be skipped by a flag,
an environment variable or a setting.
`--force` waives cleanliness and,
once the owed measurement shows it works,
ends holders;
it waives no identity or mark check
(proposal `:806` to `:809`,
`:855` to `:858`).

### Where the planned identities come from

The planned identities are read from the persisted destruction plan,
never from the live pool first:
the plan is
what step 14 proved,
and a check that re-derived its own expectation from the live objects would agree with any
substitution.
The tuple the check consumes is:

- `clone_name`:
  the clone's full dataset name,
  one plain child of the pinned clone parent;
- `clone_guid`:
  the clone's dataset GUID,
  as step 1 proved it (proposal `:869` to `:871`);
- `origin_snapshot_name`:
  `<origin dataset>@<snapshot>`,
  the snapshot step 1 resolved from the clone's live origin
  and compared with the recorded origin;
- `origin_snapshot_guid`,
  `origin_snapshot_createtxg`:
  the tuple step 1 recorded for it.

The live side is read by the planned names and compared field by field.
It is never read by whatever name happens
to be live,
because a name a caller chose is not a proof (proposal `:400` to `:404`,
"never use names,
properties or
a matching result alone to prove ownership").

A plan that lacks one of these fields is a refusal,
not a check with a missing expectation:
`PlanIncomplete` names
the field and the plan's home,
and nothing is deleted.

### What the check reads, in order

Each check makes two `ZFS_IOC_OBJSET_STATS` reads through the crate,
in this order,
and stops at the first
disagreement:

1. The clone,
   at `clone_name`.
   The reply must call it a filesystem (`dds_type` is `DMU_OST_ZFS`,
   `dds_is_snapshot`
   is zero),
   report `clone_guid`,
   and report an origin equal to `origin_snapshot_name`.
2. The origin snapshot,
   at `origin_snapshot_name`.
   The reply must call it a snapshot,
   report
   `origin_snapshot_guid`,
   and report `createtxg` equal to `origin_snapshot_createtxg`.
3. The mark,
   and the two numbers the refusal's reap sentence needs,
   from that same second reply:
   the `defer_destroy` property's `value`,
   which decides,
   and `userrefs`,
   which does not decide and only says whether a user hold pins the snapshot (F21).

The order is the design's,
and it is the part a control pins:
the mark is only ever read from an object whose
identity the same reply proved,
because a mark read from a replacement proves nothing about the planned snapshot.
Reading the origin at the planned name also proves the name still belongs to the recorded dataset (F17).

The clone's reply carries its live origin in `dds_origin`,
which the crate already exposes as
`DmuObjsetStats::origin()`
(`package/zfs/request-to-openzfs-kernel-device/src/layout/zfs_cmd_t.rs:91` to `:105`).

No other request is made.
The check does not list snapshots,
does not read the origin dataset,
and does not read
the origin-side inventory:
step 1 did that,
and repeating it here would not bind the later delete either (L4).

### Reading the mark

One request,
already in the crate:
`ZFS_IOC_OBJSET_STATS` (`0x5a12`,
OpenZFS `zfs-2.4.4`,
`include/sys/fs/zfs.h:1502`),
with `zc_simple` zero and a reply buffer,
which the crate's `objset_stats` sends
(F5,
F11).

The field of its answer is the property list's `defer_destroy` entry,
and the value is that entry's `value`
(`include/sys/fs/zfs.h:282`):
an unsigned integer,
0 for a snapshot not marked and 1 for one marked (F2,
F3).
The entry carries no `source` for this property at `zfs-2.4.4` (F4),
so the reader reads `value` and never requires
or judges `source`;
a test pins that,
because a reader that required `source` would refuse every unmarked origin and
every marked one.

What the source says the field means for a snapshot that still has clones:

- a destroy request carrying `defer` returns success for such a snapshot instead of `EEXIST` (F7),
  and the sync step
  then marks it rather than destroying it (F8);
- the marked snapshot stays visible in the namespace,
  and its clones may still be created from it
  (OpenZFS `zfs-2.4.4`,
  `man/man8/zfs-destroy.8:107` to `:112`);
- it is destroyed when its last child or hold goes.
  For the case this check guards,
  the last child is the clone a
  removal is about to destroy:
  the head destroy's own sync step destroys the marked origin in the same transaction
  (F10).

So `defer_destroy` reading 1 on the planned origin snapshot means:
some program asked the kernel to destroy a
snapshot that still has clones,
and the next clone destroy can be the transaction that does it.
The two programs on
this host that prune snapshots pass no deferred destroy,
and that exposure is reasoning from the grant model rather
than a measured account-run deferred destroy (proposal `:1523` to `:1528`),
which is what makes the check the
compensation for the delegated grant.

The numbers the refusal prints are `dds_num_clones` and `userrefs` from the same reply (F9,
F21):
the reap needs exactly one clone and no user refs,
so one clone and `userrefs` 0 mean this removal would be the transaction that
reaps the marked snapshot,
and anything else means the snapshot stands after this removal.
Both refuse all the same:
the decided sentence reads the mark,
not the reap (L2).

### The refusals

Every refusal names the object,
the field that disagreed with the recorded and the live value,
what this run has
already deleted,
and every way forward,
in the idiom "The refusals" of the descriptor mount records
(`package/git-policy/cli/SPEC.md:5834` to `:5840`;
`sweep_clone_check.rs:206` to `:280`).
Each message carries the stage sentence,
which is the only part that differs between the two call sites:
`Nothing was deleted by this run` for the first,
and for the second
`This run already destroyed the clone's own snapshots, and they do not come back`
followed by their names.

The mark,
at the first call site,
reads:

```text
refusing to remove the ZFS worktree clone `monochromatic-git-worktree-clone-datasets/fix-flaky-test`:
its origin snapshot `monochromatic-git-checkout@git-worktree-clone-origin-fix-flaky-test` is marked for deferred
destroy (`defer_destroy` reads 1), and the kernel reports 1 clone of it and 0 user holds, so destroying this clone
destroys that snapshot in the same transaction. Nothing was deleted by this run: the clone, its own snapshots, the
origin snapshot and the Git registration all stand. Ways forward: `zfs get -H -o value defer_destroy
monochromatic-git-checkout@git-worktree-clone-origin-fix-flaky-test` and `zfs holds
monochromatic-git-checkout@git-worktree-clone-origin-fix-flaky-test` show the mark and the holds, and the mark
cannot be cleared in place. Whether that snapshot may go is a person's decision: keep another clone of it, or
restore it elsewhere, before the slot is removed. This command will not take that decision for you.
```

At the second call site the same cause reads the same first sentences,
then:

```text
This run already destroyed the clone's own snapshots (`…@manual-2026-11-21`, `…@sanoid-hourly-2026-11-22`), and
they do not come back. The clone, the origin snapshot and the Git registration stand.
Ways forward: `doctor` names this slot and the mark, `cancel-worktree-removal fix-flaky-test` is the manual way
back and reports the snapshots already destroyed, and the origin snapshot's mark needs the same decision the first
refusal describes.
```

The other causes,
each with the stage sentence and the ways forward of the same shape:

- `PlanIncomplete`:
  the plan does not carry the field named,
  so no object is proven and nothing is judged.
  Ways forward:
  the plan's home,
  and that a removal which finds an incomplete plan deletes nothing.
- `CloneIsNotThePlannedOne`:
  the clone `…/fix-flaky-test` reports GUID `…` and the plan recorded `…`.
  Ways forward:
  an administrator's lineage check;
  a record is never refreshed to match a change.
- `CloneOriginDiffers`:
  the clone's live origin is `…@…` and the plan recorded `…@…`.
- `CloneHasNoOrigin`:
  the clone is not a clone,
  so nothing ties it to the planned origin.
- `CloneGone` and `OriginSnapshotGone`:
  the planned object is not there (the kernel answered `ENOENT`).
  The first says the settle rules of step 16 decide,
  not this check;
  the second says the origin cannot be judged
  and nothing was deleted.
- `OriginSnapshotIsNotThePlannedOne` and `OriginSnapshotCreatetxgDiffers`:
  the origin snapshot reports a GUID or a
  `createtxg` the plan did not record.
- `MarkUnreadable`:
  the reply has no `defer_destroy`,
  or its value is not 0 or 1,
  or the property list is missing,
  or the gate refused raw requests,
  or the device could not be opened.
  This one never falls through to "not marked":
  an unreadable mark is not an off mark (F13's rule),
  and the property
  read's own words say which state it found (`sweep_dataset_facts.rs:146` to `:152`).

A refusal prints no destructive command.
It prints the reads that show the state and names the decision a person
takes;
whether a wrapper command should ever destroy a marked origin is question Q2.

The check writes nothing,
so no refusal can leave half a record,
and there is no lock to take (rule `NLK`).

### Which refusals keep the removal record

One rule,
with two arms,
and the arm the open item asks about is question Q1:

- A refusal **drops** the removal record only when the plan names no object this removal destroyed,
  so the slot is
  exactly as it was:
  the clone,
  its snapshots,
  the origin,
  the registration and no record,
  and a later retry starts
  from the top.
  This is the Btrfs precedent's rule (F14) and the removal cancels itself as a blocked removal does,
  except that nothing is unmounted to reattach,
  because this refusal happens before the unmount's request in a
  first run.
- A refusal **keeps** the removal record whenever the plan names an object this removal destroyed.
  That covers the
  second call site always,
  and the first call site of a resumed removal.
  On the machine this leaves:
  the clone head present and unmounted (staged under `~/.cache` where the staging route
  was used),
  its own snapshots gone,
  the origin snapshot and its mark untouched,
  the Git registration intact,
  the
  record present with each tuple's outcome,
  `doctor` naming the slot and the mark,
  and `cancel-worktree-removal`
  as the way back (a sibling's design).

The design adopts the keep arm for the second call site,
because the alternative erases the only statement that
history was destroyed,
and because the machine it leaves is the one the proposal already describes for a removal
that ran past its deadline:
neither cancelled nor finished,
with a person's way back.
The project's answer to Q1 is what decides it;
until then the design is whole but provisional here.

### The accepted residual

Stated exactly,
and left open as the decision leaves it:

- Between either check and the delete it precedes,
  a process of the same account can replace the clone or the origin
  snapshot,
  or mark the origin.
  The check's answer is a read,
  and the delete is a later request that names objects
  rather than GUIDs (A6).
- Between the two checks,
  a mark set on the origin is caught by the second check in a single invocation,
  because
  the second check re-reads the mark.
  A mark set after the second check and before the head request is not caught,
  and the head request is the transaction that reaps the marked origin (F10).
- The delegated answer accepts both windows rather than asking for a root-run conditional destroy,
  which is the
  alternative the project did not select (proposal,
  "Clone removal identity protection").
  Nothing here closes them,
  and no test claims to.

### Where the code lives and what it needs from the removal

A new folder,
`package/git-policy/cli/src/native/worktree_removal_deferred_origin_check/`,
with a top doc listing its files in dependency order (rule `LYF`):

- `planned_origin.rs`:
  the tuple A1 requires and its reading from the plan,
  and `PlanIncomplete`;
- `live_origin.rs`:
  the two reads and the mark's extraction,
  over the shared reader;
- `origin_verdict.rs`:
  the ordered comparison of "What the check reads,
  in order" and the stage;
- `origin_refusals.rs`:
  the causes and their words;
- `mod.rs` and `uses/`:
  the folder's links to the folders its code uses,
  including the removal's plan module.

Two small moves keep the folder graph acyclic (rule `LYL`),
both marked for veto as V6:

- `PropertyValue` and `value_of` move from
  `worktree_clone_mount_handler/sweep_dataset_facts.rs` into a folder of their own,
  `package/git-policy/cli/src/native/zfs_property_value/`,
  and the handler re-exports them so its callers do not
  change;
- `DatasetReader`,
  `ReadFailure` and `ListedChild` move from
  `worktree_clone_mount_handler/sweep_dataset_reader.rs` into
  `package/git-policy/cli/src/native/zfs_dataset_reader/`,
  with the same re-export.

What the check needs from the removal,
and this design does not design:

- the plan's reader,
  with the fields A1 names and an absent field it can name;
- the record's writer and its home,
  from A2;
- a seam at the removal's kernel calls,
  so a control can count that no write was sent after a refusal and can set a
  mark at the moment between two calls (A3);
- the removal's deadline,
  which bounds the two extra reads without a limit of its own (L7).

### What each build step builds

- Step 15 builds:
  the four files,
  the two moves,
  the plan's field requirements as a refusal,
  the refusal words,
  the
  first call site,
  and the controls that need no device.
- Step 16 builds:
  the second call site;
  the record arms of "Which refusals keep the removal record";
  the per-name
  outcomes the second refusal prints;
  and the run of the owed measurement once the route is whole.
- Step 18 builds:
  `doctor`'s lines from the kept record,
  which replace the sentence that reports the check "as not
  yet designed" (proposal `:1233` to `:1236`),
  and its reporting of an unreadable state as unknown
  (plan,
  step 18).
- No step ships the check behind a setting,
  and step 15 lands ahead of step 16,
  so no build reaches a snapshot
  delete of this route without the first check.

## The tests a build must have

Every control below except the two guest ones runs on the host against the trait seam (A3),
no device and no pool,
following `sweep_one_clone_tests.rs:158`,
which already builds a `DatasetIdentity` by hand.

### Controls that need no device

1. `a_marked_origin_refuses_before_the_snapshot_batch`:
   the mark reads 1,
   the verdict is `Refused`,
   and the fixture
   that stands for the batch records no call.
2. `an_unmarked_origin_proceeds_and_reads_exactly_two_identities`:
   the fixture counts two `identity` calls,
   in the
   order clone then origin snapshot,
   and no other call.
3. `the_mark_is_read_from_the_value_and_not_the_source`:
   a pair with `value` alone,
   a pair with `value` and a
   `source`,
   and a pair whose `value` is text;
   only the integers 0 and 1 decide.
4. `an_unreadable_mark_refuses_rather_than_proceeding`:
   absent key,
   missing property list,
   text value,
   a value of
   2,
   and a gate refusal each produce `MarkUnreadable` with its own state named.
5. `a_replaced_clone_refuses_before_the_mark_is_read`:
   `clone_guid` differs,
   and the fixture records that the second
   read never ran.
6. `a_clone_whose_live_origin_differs_refuses_before_the_origin_is_read`,
   and
   `a_clone_with_no_origin_refuses`.
7. `a_replaced_origin_snapshot_refuses`,
   for a GUID and for a `createtxg` that differs,
   and
   `an_origin_that_kernel_calls_a_filesystem_refuses`.
8. `an_incomplete_plan_names_the_field_and_deletes_nothing`.
9. `the_first_refusal_prints_nothing_was_deleted_and_the_second_names_what_went`:
   the exact message per cause and
   stage,
   compared as the crate compares its README line
   (`package/zfs/request-to-openzfs-kernel-device/README.md`,
   "What a failed request tells you").
10. `a_marked_origin_with_a_second_clone_still_refuses_and_says_the_origin_is_pinned`:
    `dds_num_clones` is 2,
    the verdict is `Refused`,
    and the message says the origin is pinned by another clone.
11. `a_marked_origin_under_a_user_hold_still_refuses_and_says_the_reap_will_not_happen`:
    `userrefs` is 1 and `dds_num_clones` is 1,
    the verdict is `Refused`,
    and the message says the snapshot stands after this removal.
12. `a_refused_first_check_with_an_empty_plan_drops_the_record_and_a_refused_resume_keeps_it`.
13. `a_refused_second_check_keeps_the_record_and_the_outcomes_it_names`.
14. `the_doctor_line_names_the_slot_the_marked_origin_and_the_destroyed_snapshots` (step 18's control over the
    record).

### The races, each forced without timing luck

R1.
A mark set between the first check and the batch.
Forced by the batch fixture,
which sets the mark before it
returns;
the second check then refuses and the head fixture records no call.
This is the single-invocation arm,
and
it proves the second check protects what the first could not see.

R2.
A mark set between the second check and the head request.
Forced by the head fixture,
which sets the mark and
then receives the request.
The control asserts the request is sent,
which documents the residual rather than
closing it (L4).
The guest arm measures what the kernel then does.

R3.
The origin snapshot destroyed or replaced between the two checks.
Forced by the origin fixture,
which answers
`ENOENT` on the second call;
the verdict is `OriginSnapshotGone` and the head fixture records no call.

R4.
The clone replaced between the two checks.
Forced the same way with another GUID;
the verdict is
`CloneIsNotThePlannedOne` and the head fixture records no call.

R5.
A same-account substitution between a check and the request it precedes.
Not forced and not claimed:
it is L4,
and the measured arms of [VM item 12][vm-12] are its record.

### Controls that need a guest

15. `a_marked_snapshot_reports_the_mark_and_an_unmarked_one_does_not`,
    in
    `package/zfs/request-to-openzfs-kernel-device/src/kernel_device/live_read_tests.rs`:
    the identity read on a
    snapshot before a deferred destroy reads 0,
    after it reads 1,
    the entry carries no `source`,
    and
    `dds_num_clones` reads 1.
    It runs through the package's e2e recipe like the other `#[ignore]` live controls
    (`live_read_tests.rs:120` to `:121`).
16. The removal arms of M3 below,
    in a new guest recipe under `package/git-policy/cli/e2e/`,
    driven the way
    `mount-git-worktree-clone-datasets-guest.ts` drives the step 8 arms.

## Measurements owed

None of these ran in this round.
Each is written to be run by a later round as it stands.

M1,
the positive control:
the reap this check exists to prevent.
Owed before the refusal arms,
because a harness
that cannot see the loss cannot show that the refusal avoids it (rule `QPC`).
It needs no built wrapper.

```bash
# in the disposable guest, as root, on a file-backed pool of its own
mkdir --parents /var/lib/zfs-fixtures/deferred-origin
truncate --size 4G /var/lib/zfs-fixtures/deferred-origin/pool.img
zpool create zfsdeferred /var/lib/zfs-fixtures/deferred-origin/pool.img
zfs create -o mountpoint=/var/lib/zfs-fixtures/deferred-origin/checkout zfsdeferred/checkout
zfs create -o mountpoint=legacy zfsdeferred/monochromatic-git-worktree-clone-datasets
printf 'history\n' > /var/lib/zfs-fixtures/deferred-origin/checkout/history
zfs snapshot zfsdeferred/checkout@git-worktree-clone-origin-fix-flaky-test
zfs clone zfsdeferred/checkout@git-worktree-clone-origin-fix-flaky-test \
    zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test
# the read before the mark, the way the check reads it
zfs get -H -o value defer_destroy zfsdeferred/checkout@git-worktree-clone-origin-fix-flaky-test
# the mark, as the account that holds item 17's descendant destroy grant on the checkout
sudo --user account zfs destroy -d zfsdeferred/checkout@git-worktree-clone-origin-fix-flaky-test
zfs get -H -o value defer_destroy zfsdeferred/checkout@git-worktree-clone-origin-fix-flaky-test
# the control: no check stands between this and the reap
zfs destroy zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test
zfs list -H -o name zfsdeferred/checkout@git-worktree-clone-origin-fix-flaky-test
```

Recorded,
in `keep/`,
as one log:
the first read answers `off`;
`zfs destroy -d` exits 0 and prints nothing;
the
second read answers `on`;
the clone destroy exits 0;
and the last listing exits non-zero naming no such dataset,
so
the marked snapshot went with the clone in one transaction.
Pass is that sequence.
Fail,
and the whole item returns to the owner,
is the marked snapshot still present after the clone destroy:
the
check would then guard a reap this release does not perform.

M2,
the read through the binding:
control 15 above,
run in the same guest,
with its own log.
It shows that the field
the check reads is the field the kernel fills,
for both values,
through the crate rather than through `zfs`.

M3,
the two refusal arms and what a person sees.
It needs steps 15 and 16 built.

1. Fresh fixture as in M1,
   with two own snapshots of the clone,
   and a user hold on the clone head so the head
   destroy refuses deterministically (`zfs hold deferred-origin-probe <clone>`,
   which the head check answers `EBUSY` for:
   OpenZFS `zfs-2.4.4`,
   `module/zfs/dsl_destroy.c:776` to `:777`).
2. Run the built removal.
   Record:
   the snapshot batch destroyed both own snapshots,
   the head request answered busy,
   the removal stopped,
   the record's state,
   and the exit status.
3. As the account:
   `zfs destroy -d <origin snapshot>`,
   then `zfs release deferred-origin-probe <clone>`.
4. Run the built removal again.
   This is the state the open item names:
   the batch is done,
   so only the head check
   runs.
   Record:
   the message,
   the exit status,
   `doctor`'s lines,
   the origin snapshot and the mark still present,
   the
   clone head present without its snapshots,
   the record present,
   and `git worktree list` unchanged.
   Pass is the refusal before any head request plus those objects standing.
   Which record state passes depends on the
   project's answer to Q1;
   the log records both what the run left and what the answer requires.
5. The residual's reach,
   as counts and not as a guarantee:
   repeat the run with the origin unmarked and a
   same-account watcher that marks it the moment the clone's snapshot list empties,
   in one interleaved round with an
   unchanged-input control,
   and report how many runs had the second check see the mark and how many ended with the
   origin reaped.
   Rule `QNB` applies:
   the spread of the control arm is reported beside it.

M4,
the cost:
the wall time of one removal with the check against one without,
in the same guest and the same
round,
both arms alternating,
reported as a ratio with its range (rule `TVC`).
It sizes no shipped number (L7).

## Choices this design marks for the project's veto

Each entry names the option this design adopts and the alternative it ranked over,
with a concrete example of each,
the pros and cons a person would meet,
and the ranking with the reason for the adjacent pair.
A veto reopens the entry;
none of them is a decision the records made.

V1.
When the first check runs.
- Adopted:
  it runs when this run is about to send the snapshot request,
  and not when the remaining planned set is empty.
  Example:
  a resumed removal whose plan says both planned snapshots are gone sends only the head request,
  so only the head check runs,
  and a marked origin refuses there.
  Pro:
  the refusal after the batch is reachable,
  which is the state the owed measurement and question Q1 are about.
  Con:
  a removal without own snapshots is guarded by one check instead of two.
- Alternative:
  always run both checks.
  Example:
  the same resumed removal refuses at a check that guards no request,
  so the refusal before the head is reachable only through the accepted residual.
  Pro:
  two checks on every removal.
  Con:
  question Q1 would be about a window nobody can observe.
Ranking:
  adopted over the alternative,
because the alternative leaves the answer Q1 asks for about a state no run can produce.

V2.
Where the check reads the planned identities.
- Adopted:
  from the persisted destruction plan.
  Example:
  the plan's origin GUID disagrees with the binding record's,
  and the check refuses against the plan's.
  Pro:
  one record on the decision's path,
  the one step 14 proved in this run.
  Con:
  a corrupt plan is a refusal where the binding record might have proved the tuple.
- Alternative:
  from the binding record.
  Example:
  the same disagreement,
  and the check judges it against a record the add wrote months earlier,
  which a lineage change may have invalidated since.
  Pro:
  it exists from the add onward.
  Con:
  it is not what this run verified.
Ranking:
  adopted over the alternative,
because the check's job is to re-prove the objects this removal planned,
and only the plan holds that plan.
It depends on A1.

V3.
How the mark's absence is read.
- Adopted:
  read `value` alone,
  and refuse when the entry,
  its value or the reply is not readable.
  Example:
  a reply with no `defer_destroy` key refuses,
  and one whose value is text refuses.
  Pro:
  it fails closed.
  Con:
  a release that renames or drops the property stops every removal until the reader learns the name,
  and the layout gate compares no property name (F5,
  F22's gate).
- Alternative:
  treat an absent entry as an unmarked origin.
  Example:
  the same reply passes,
  and a marked origin whose entry the kernel stopped sending is destroyed with its mark unread.
  Pro:
  no removal stops on a changed reply.
  Con:
  the check would fail open on exactly the state it exists to catch.
Ranking:
  adopted over the alternative,
because a check that cannot read its one field has not checked.

V4.
Which refusals keep the removal record.
- Adopted:
  the bucket rule,
  drop only when the plan names no object this removal destroyed.
  Example:
  a first-run refusal drops the record and leaves the slot as it was;
  a resumed or post-batch refusal keeps it and `doctor` reports the loss.
  Pro:
  the record survives exactly when it is the only statement of a loss.
  Con:
  one refusal kind has two outcomes,
  so a person reads the message to learn which they have.
- Alternative:
  always drop.
  Example:
  a post-batch refusal drops the record,
  and nothing on the machine states that the clone's snapshots went.
  Pro:
  one outcome.
  Con:
  it discards the loss statement.
Ranking:
  adopted over the alternative,
because the loss is what the record is for.
Which arm applies after the batch is question Q1.

V5.
Whether the check writes anything.
- Adopted:
  nothing,
  no record of its own and no lock.
  Example:
  a killed check leaves no staging file and no state a later `doctor` reconciles.
  Pro:
  rule `NLK` holds trivially and a refusal leaves the machine as it found it.
  Con:
  nothing records that a check ran,
  so `doctor` reads only what the removal's own record says.
- Alternative:
  a check record naming the reads and their answers.
  Example:
  `doctor` prints the last check's claim,
  and cannot say when it was true,
  because the objects have changed since.
  Pro:
  a visible claim.
  Con:
  a second record to publish,
  reconcile and expire,
  for a statement the next kernel write invalidates.
Ranking:
  adopted over the alternative,
because a record of a read that the next write invalidates misleads.

V6.
Where the shared property reader and the reader trait live.
- Adopted:
  two small folders of their own,
  with the mount handler re-exporting them.
  Example:
  the check's folder links to `zfs_property_value` and so does the handler's,
  and each link points one way.
  Pro:
  rules `LYL` and `LYF` hold without a cycle.
  Con:
  step 8 files move in step 15 and their controls follow.
- Alternative:
  the check imports from the handler's folder.
  Example:
  the handler later needs a ruling the check holds,
  and the two folders then use each other,
  which rule `LYL` forbids.
  Pro:
  no move.
  Con:
  the cycle the rule exists to prevent.
Ranking:
  adopted over the alternative,
because a one-way move now is cheaper than a ruled cycle later.

V7.
What a refusal prints.
- Adopted:
  the reads and the decision,
  never a destructive command.
  Example:
  the marked-origin refusal prints `zfs get` and `zfs holds` and names the decision,
  and prints no `zfs destroy`.
  Pro:
  the wrapper never prints a command that destroys history another program marked.
  Con:
  a person must know how to act,
  or ask the administrator.
- Alternative:
  print the exact `zfs destroy` for the slot.
  Example:
  a hurried reader copies it,
  and the marked snapshot goes with the clone.
  Pro:
  one copy.
  Con:
  printing it reads as authority,
  and the authority question Q2 is open.
Ranking:
  adopted over the alternative,
because a printed destructive command is read as permission.

V8.
What the refusal says about the reap.
- Adopted:
  it names `dds_num_clones` and `userrefs` from the same reply.
  Example:
  one clone and 0 holds reads "so destroying this clone destroys that snapshot in the same transaction",
  and two clones reads "another clone pins it".
  Pro:
  a person learns whether their decision is urgent.
  Con:
  two more numbers in the message.
- Alternative:
  say only that the origin is marked.
  Example:
  the same message without either number,
  and a person cannot tell a snapshot about to be destroyed from one that will stand.
  Pro:
  less to read.
  Con:
  it leaves out the fact that separates a loss from a no-op.
Ranking:
  adopted over the alternative,
because the message's job is the effect on the person's history.

## Questions only the project can answer

Q1.
After a refusal at the check before the head destroy,
does the removal keep its record or cancel itself?
The records do not decide it (proposal `:938` to `:942`;
the open item says so).

- Option A,
  adopted by this design:
  keep the record.
  Example:
  a resumed removal,
  its batch done,
  the origin marked
  after the interruption.
  A person sees the refusal,
  the clone without its snapshots,
  `doctor` naming the slot,
  the mark and the destroyed snapshots (by name),
  and `cancel-worktree-removal fix-flaky-test` as the way back.
  Pro:
  the only statement that history was destroyed survives,
  and the slot waits for a person who must decide
  about the mark anyway.
  Con:
  a worktree stays out of use and out of the namespace until someone acts.
- Option B:
  let the removal cancel itself.
  Example:
  the same run reattaches the clone as it stands and drops the
  record,
  so the slot returns with its head and no snapshots and nothing but the printed message says they went.
  Pro:
  the slot returns to use without a person,
  and the removal ends in one state,
  the cancelled one.
  Con:
  the per
  name loss is no longer written anywhere,
  and a later removal re-proves from scratch against a clone history that
  silently shrank;
  the cancellation rule the project decided is about holders,
  not about a refused check.
- Option C:
  cancel itself and write a notice that outlives the record.
  Example:
  the reattach happens and a notice
  in the checkout's notices names the snapshots lost.
  Pro:
  both the return to use and the loss report.
  Con:
  it adds
  a notice kind and a reader for one refusal cause.

Ranking:
A > C > B.
A over C,
because A needs no new record kind and the record a person must consult anyway is
the one that already names the tuples;
C over B,
because B is the one option that loses the loss.

Q2.
May the wrapper ever destroy a snapshot another program marked for deferred destroy?
Example:
a person decides the marked sanoid snapshot may go,
and asks the wrapper to finish the removal.
The records decide that the removal refuses and nothing authorizes the destruction (F8,
L3).

- Option A,
  adopted by this design:
  no. Example:
  the refusal names the decision and prints no destructive command;
  the person either keeps a clone pinning the snapshot or destroys the slot outside the wrapper.
  Pro:
  the wrapper never destroys history a program other than itself marked,
  and the delegated grant it holds is
  not widened.
  Con:
  the work is manual,
  and a person who does not read the message may destroy more than they
  meant.
- Option B:
  a guarded command that takes the slot and names the marked snapshot it will destroy,
  once.
  Example:
  `monochromatic-git-policy-cli remove-worktree --accept-deferred-origin <slot>` prints the snapshot's name
  and tuple and needs a second confirmation word.
  Pro:
  the removal finishes inside the proofs the route already
  has.
  Con:
  a new authority and a new refusal surface,
  and a person can now destroy exactly the history the check
  exists to protect.
- Option C:
  a `doctor` command printed for the administrator,
  never run by the wrapper.
  Pro:
  nothing new is authorized in the wrapper.
  Con:
  it is the same manual act with the wrapper's blessing
  printed,
  which reads as authority it does not have.

Ranking:
A > C > B.
A over C,
because printing a command is indistinguishable to a hurried reader from
authorizing it;
C over B,
because B widens what the account may destroy through the wrapper for a case the project
has not asked to solve.

## Dependencies on sibling designs

Each of these is an assumption this design made so it could be whole;
a later pass reconciles the two.

- "Where a removal's persisted destruction plan lives,
  and its format"
  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`):
  must carry the five tuple fields of A1.
  If it cannot,
  the check reads the origin tuple from the binding record instead and V2 changes.
- "Where the add's durable binding record lives,
  and its format"
  (`doc/planning/cli-git-zfs-add-binding-record-design.md`):
  supplies the origin tuple A1 falls back to.
- "The record that carries origin cleanup after a registration is settled"
  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`):
  a kept removal record and the origin-cleanup record
  can both name the origin snapshot;
  this design requires only that the two cannot contradict each other about its
  identity,
  and that the origin-cleanup record is not read as the plan.
- "What a cancelled,
  timed-out or killed removal leaves,
  and what the cancel command proves and does"
  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`):
  owns the state a kept
  record leaves and what the cancel command prints.
  This design assumes the cancel command is the way back for a
  kept record and that it reports the destroyed snapshots (proposal `:977` to `:1000`).
- "The forced arm of a removal"
  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`):
  owns what `--force` ends.
  This design assumes `--force` skips no check here.
- "What re-applies private propagation after a replacement mount of the checkout"
  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`):
  unrelated to the check,
  and
  named only because a checkout whose mount was replaced is one way the origin's name could change under a resumed
  removal (F17).

## Limitations

- Nothing was measured (L1).
  The design's whole kernel case rests on F1 to F10 read at `zfs-2.4.4`,
  and the host's
  module is `2.4.4-1`;
  a different release is the gate's question,
  not this design's.
- The check refuses on the mark alone (L2),
  so it refuses removals the kernel would have performed without reaping
  the marked snapshot.
- A refusal has no in-ZFS undo (L3):
  the design can report the mark,
  not clear it.
- The identity proof is by name,
  GUID and `createtxg` in practice,
  not against a 64-bit GUID collision,
  which is the
  same standard the proposal uses for bookmarks (proposal `:1620` to `:1625`).
- The check's two reads are not bound to the request after them (L4).
- The check adds two ioctls to a removal and ships no number that bounds them (L7).
- The refusal's exit status and the removal's status for a refusal are step 14's to settle (L8).

## What this design does not cover

- The destruction plan's home,
  format,
  writer and resume rules.
- The removal record's home,
  format and writer,
  beyond the keep-or-drop rule.
- The origin-cleanup record.
- The blocked-unmount cancellation,
  `--force`'s holder arm,
  the cancel command and what a timed-out removal leaves.
- What `doctor` prints beyond the fields a kept record must give it.
- The start-time gate,
  the runtime layout checker and the binding itself.
- Any Git,
  Linux,
  mount or systemd behaviour:
  none is claimed here.
- Ending a holder,
  signalling a process or a unit's task,
  and what an unprivileged signal request answers.
  This design ends nothing and signals nothing,
  so it needs no such fact and owes no such measurement.
  That fact belongs to "The forced arm of a removal",
  where it has to come from systemd's and polkit's source at the version this host runs,
  and from `kill(2)`,
  and never from a request made on this machine:
  a request to the system service manager that needs authorization can put a password dialog on the desktop of
  the person using the machine and wait on that person,
  and asking a person for a secret is what this project forbids
  (`package/git-policy/cli/SPEC.md:7345` to `:7350`).
  This round was told to make no such request,
  and it made none.
- The rejected root alternative's channel program (proposal,
  "Clone removal identity protection").

## Sources read

OpenZFS `zfs-2.4.4`,
commit `71a9f95`,
cloned at
`/home/user/temp/agent/zfs-deferred-destroy-checks-design-2026-11-22/scratch/openzfs-zfs-2.4.4`:
`include/sys/fs/zfs.h`,
`include/sys/dsl_dataset.h`,
`module/zcommon/zfs_prop.c`,
`module/zfs/dsl_dataset.c`,
`module/zfs/dsl_prop.c`,
`module/zfs/dsl_destroy.c`,
`module/zfs/dmu_objset.c`,
`module/zfs/zfs_ioctl.c`,
`module/zfs/zcp_get.c`,
`lib/libzfs_core/libzfs_core.c`,
`lib/libzfs/libzfs_dataset.c`,
`cmd/zfs/zfs_main.c`,
`man/man7/zfsprops.7`,
`man/man8/zfs-destroy.8`.

This repository at `5dd8883a60`:
the decision,
proposal,
plan and decision records named in the opening,
`package/git-policy/cli/SPEC.md`,
the crate under `package/zfs/request-to-openzfs-kernel-device`,
and the native code named in "What this design reuses".

No Git and no Linux source was read:
nothing here claims their behaviour.

## Revisions

One revision so far,
the commit that adds this file,
whose commit timestamp is what `git log --format=%cI` prints for it and whose subject names the open item.

[vm-12]: cli-git-zfs-measurements-owed-before-acceptance.md
