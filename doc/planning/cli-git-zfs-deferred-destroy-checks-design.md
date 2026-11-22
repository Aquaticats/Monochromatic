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
beyond the choices "Choices this design marks for the project's veto" lists
and the questions "Questions only the project can answer" puts.

This revision answers the gate review of the version at `124ea5b573`
(`~/temp/agent/gate-review-zfs-deferred-destroy-checks-design-2026-11-22/keep/review.md`,
which is outside this repository).
"Revisions" lists what each commit changed,
and "What this revision rewrote" says which sections the review's findings turned over
and which stand as the first version wrote them.

It read,
whole or in the passages named:
`doc/decision/cli-git-zfs-worktree-design.md`,
the proposal's "Binding crate and compatibility",
"Requests and input ownership",
"Removal and recovery",
"Cancelling a blocked removal",
"Project decisions" ("Origin cleanup authority",
"Blocked-removal cancellation",
"No implicit remount after a failure or restart",
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
`worktree_removal_interrupted_destroy.rs`,
`worktree_clone_conversion_fence_measurement_routes.rs` and `worktree_clone_conversion_beside_the_clone.rs`.

It read each sibling design this item leans on,
and each that leans on this item,
with `git show <branch>:<doc>`,
and "Dependencies on sibling designs" quotes the words that bear on the checks.
A sibling's document is unreviewed:
none of its answers is adopted here as decided,
and none is edited.

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
and "Measurements owed" writes the probe that would run each one,
with the account and grants it needs.

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
> check first (Recommended)",
> and "Also check first (Recommended)",
> which puts the same check before the first snapshot delete as well,
> so such a removal refuses before this run deletes anything,
> while the check before the head destroy stays.
> Where the checks read the planned clone and origin identities,
> how they read the origin's deferred-destroy mark,
> what each prints on refusal,
> and which refusals keep the removal record are undesigned.
> The check before the head destroy runs after the snapshot batch,
> so a refusal there leaves the clone without its own snapshots,
> and nothing says whether such a refusal keeps the removal record or lets the removal cancel itself.
> The window between the check before the head destroy and the destroy,
> which a same-account process can use to mark the origin,
> is the residual the delegated answer accepts rather than a gap this section asks anyone to close;
> the answer leaves a mark set between the two checks inside that same residual
> ([Clone removal identity protection](#clone-removal-identity-protection)).
> The refusal before the head destroy is one of the VM measurements the build owes
> ([Forced removal and deferred origins](#forced-removal-and-deferred-origins)).

The order of a removal the checks sit in,
from "Removal and recovery" step 2 (proposal `:886` to `:888`):

> For a proven set,
> record removal intent and print the slot,
> dataset and exact snapshot names to be lost.
> Release local handles and request `unmount`.

and step 3 (proposal `:894` to `:897`):

> After unmount,
> persist the verified inventory and expected clone identity as the destruction plan.
> ...
> Require fresh root all-namespace verification before the first snapshot delete.
> Reprove lineage and candidate identities immediately before submitting the full verified snapshot set

The removal intent is the clone's own mount property,
which the wrapper specification names at `package/git-policy/cli/SPEC.md:6093`:
"the removal intent is step 14's",
and describes at `:6097` to `:6108`:
the clone's local `org.monochromatic:git-worktree-clone-mount-desired-state` property carries
`v1:<nonce>:mount` or `v1:<nonce>:unmount`,
and "the value is the whole request".

The decisions and the answers,
each quoted from the file and line named:

- `doc/decision/cli-git-zfs-worktree-design.md:45` to `:48`:
  "The check that the origin is not marked for deferred destroy also runs before the first snapshot delete,
  so such a removal refuses before this run deletes anything.
  The check before the head destroy stays,
  and a mark set between the two remains the accepted residual."
- `doc/decision/cli-git-zfs-worktree-design.md:59` to `:62`:
  "The two owed VM measurements still gate shipping,
  and they decide only the sentences already conditional on them:
  forcing the removal of a clone a holder keeps,
  and the refusal of a removal whose origin is marked for deferred destroy."
- The proposal,
  `:1416` to `:1420`:
  "Delegated origin `destroy` also lets the account mark a checkout snapshot for deferred destroy,
  so this decision is the one that pairs with clone removal identity protection;
  the project answered that pair "Keep both,
  check first (Recommended)",
  and on 2026-11-19 placed the check "Also check first (Recommended)":
  it runs before the first snapshot delete as well as before the head destroy,
  so a marked origin refuses before this run deletes anything,
  and a mark set between the two checks remains the accepted residual."
- The proposal,
  `:929` to `:932`:
  "Run the check the paired answer requires,
  "Keep both,
  check first (Recommended)",
  before the first snapshot delete and again before the head destroy:
  that the clone and its origin are the planned ones and that the origin is not marked for deferred destroy,
  refusing otherwise,
  because destroying the last clone can reap an origin already marked for deferred destroy outside the separate
  origin-cleanup step."
- The proposal,
  `:1517` to `:1522`:
  "Removal checks,
  before the first snapshot delete and again just before the head destroy,
  that the clone and its origin are the planned ones and that the origin is not marked for deferred destroy,
  refusing otherwise".
- The proposal,
  `:1526` to `:1528`:
  "That exposure is reasoning from the grant model and from what the `defer` key does
  (`lib/libzfs_core/libzfs_core.c:442` to `:451`,
  OpenZFS `zfs-2.4.4`),
  not a measured account-run deferred destroy".
- The proposal,
  `:894` to `:900`:
  "After unmount,
  persist the verified inventory and expected clone identity as the destruction plan.
  ...
  Require fresh root all-namespace verification before the first snapshot delete.
  Reprove lineage and candidate identities immediately before submitting the full verified snapshot set in one
  `ZFS_IOC_DESTROY_SNAPS` request that omits the `defer` key;
  retain the per-name error list".
- The proposal,
  `:934` to `:940`:
  "so a removal whose origin is marked refuses before this run deletes anything,
  and the clone,
  its remaining snapshots and the registration stay.
  A removal that resumes may already have lost the snapshots an earlier run destroyed,
  and no later refusal brings them back.
  ...
  The check before the head destroy runs after the snapshot batch,
  so a refusal there leaves the clone without its own snapshots:
  the head and the registration survive at that moment,
  the history does not come back,
  and whether the refusal then keeps the removal record or lets the removal cancel itself is under Open items
  the decisions leave."
- The proposal,
  `:1961` to `:1970`,
  the owed measurement's own expectation of what a refusal leaves:
  "record that the check before the first snapshot delete refuses,
  with the clone,
  its snapshots,
  the origin and the marked snapshot still there,
  and what the caller and `doctor` see.
  Then set the mark between that check and the head destroy and record that the check before the head destroy
  refuses before the head request,
  with the origin and the marked snapshot still there and the removal record and the Git registration surviving."
- The proposal,
  `:2095` to `:2098`,
  the replacement-clone arm the build owes:
  "Include a replacement clone whose different origin is marked for deferred destroy:
  the deferred-destroy check before the first snapshot delete must refuse it when the replacement is already in
  place,
  the check before the head destroy must refuse it otherwise".
- The proposal's one decided remount exception,
  `:1492` to `:1498`:
  "No implicit remount after a failure or restart:
  kept,
  with one exception,
  answering "Keep,
  one exception (Recommended)".
  The exception is the checked reattach of a removal that cancels itself without `--force`".
- The proposal's acceptance test for a post-unmount refusal,
  `:2103` to `:2104`:
  "After a post-unmount refusal,
  test that restart honors the standing intent,
  `doctor` shows it,
  and a removal that cancels itself without `--force` reattaches only for a surviving,
  verified binding with known outcomes".
- The proposal's `doctor` lines for a standing removal,
  `:1241` to `:1245`:
  "For a blocked removal,
  show standing `unmount` intent,
  the last verified loss and remaining objects,
  ...,
  and name the kept `cancel-worktree-removal <slot>` command as the way back,
  reporting any snapshots already destroyed".
- The proposal,
  `:810` to `:812`:
  "If an inspection cannot prove its result,
  refuse instead of falling through to Git's recursive removal for a managed slot."
- The proposal on `--force`,
  `:806` to `:808`:
  "It does not waive an operator or unexplained lock,
  mount boundary or identity check."

Where two records disagree,
this design says so and does not pick silently:

- The open item calls "whether such a refusal keeps the removal record or lets the removal cancel itself" open
  (proposal `:937` to `:942`),
  while the owed measurement's text already expects the record to survive
  (proposal `:1966` to `:1968`),
  and the acceptance test expects the intent to stand after a post-unmount refusal (proposal `:2103` to `:2104`).
  This design therefore adopts "keep the intent and the destruction plan" and says where the records lean;
  question Q1 and question QA put the choice to the project.
- The proposal says a snapshot marked for deferred destroy "remains visible,
  and the user may create new clones from it and place new holds on it"
  (OpenZFS `zfs-2.4.4`,
  `man/man8/zfs-destroy.8:110` to `:115`),
  and nothing in `zfs-2.4.4` clears the flag (F8).
  One sibling design says an administrator can remove the mark
  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
  "`deferred` ... or when an administrator removes the mark").
  The source decides against that sibling:
  no code clears it,
  so a reliance on clearing is a defect in that sibling,
  which "Dependencies on sibling designs" names.

Two further records bind every design of this route and are followed here:

- The threat model,
  `doc/decision/cli-git-zfs-staging-under-cache.md:231` to `:232`:
  "the account is trusted,
  and root and the account are not a security boundary here.",
  and the staging design's rule for arguing under it,
  `doc/planning/cli-git-zfs-staging-under-cache-design.md:96` to `:97`:
  "Where the design argues about what the account could do,
  it says whether the argument survives that sentence".
  The design says,
  for each argument it makes,
  whether that argument survives the sentence:
  the causes in "The refusals" and the keep rule need no hostile account to matter,
  "The accepted residual" is a window inside one account,
  which the project's sentence treats as no boundary,
  and question Q2 asks the project about authority rather than about an attack.
- No configuration,
  `doc/decision/cli-git-zfs-step-8-sweep-choices.md:19` to `:22`:
  "The project first answered that everything must work by default with no configuration",
  and its answer of 2026-11-22,
  "1:
  no file needed (Recommended)" (`:44` to `:45`).
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
  the origin and every property with its source
  (`package/zfs/request-to-openzfs-kernel-device/src/kernel_device/read_requests.rs:45` to `:53`,
  `src/kernel_device/listed_object.rs:39` to `:47`).
- The device seam:
  `DatasetReader`,
  whose module doc says
  "The trait is the seam a control drives with a scripted pool,
  because no control on a person's machine may open the device"
  (`package/git-policy/cli/src/native/worktree_clone_mount_handler/sweep_dataset_reader.rs:13` to `:15`).
- The property reader:
  `PropertyValue` and `value_of`,
  which already tell an absent property from an unreadable one and word both for a refusal
  (`.../sweep_dataset_facts.rs:110` to `:152`,
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
  in `monochromatic-git-policy-cli/recorded-destroys` under the common Git directory,
  named by the registration's id
  (`package/git-policy/cli/src/native/worktree/worktree_removal_interrupted_destroy.rs:57` to `:59`),
  marks a returned call as `destroyed=1` (`:79` to `:84`),
  and drops the record when the object it named is still there (`:22` to `:24`).
- Lock-free publication when something is written:
  the slot identity store's `O_EXCL` staging file and one `linkat`,
  with no lock and no listing on a reader's path
  (`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
  "The design accepted").
  This design writes nothing,
  so it reuses the shape only as the rule it must not break.
- A measurement route that a shipped build cannot take:
  the `measurement-routes` feature,
  off by default,
  "so no setting can reach a route the shipped code does not take"
  (`package/git-policy/cli/Cargo.toml:71` to `:75`),
  and its env-var switch as the clone fence uses one
  (`.../worktree_clone_conversion_beside_the_clone.rs:123` to `:126`).
  "Measurements owed" uses this for the one arm no shipped run can pause.
- `doctor`:
  the read-only filesystem report,
  which already reports unmounted and staged clones,
  standing intent,
  the last verified loss and the kept cancel command (proposal `:1241` to `:1245`).

## Facts

Each fact is one claim a reviewer can check alone.
Nothing here was measured in this round.

F1.
The mark is a property named `defer_destroy`,
an index property with the values `yes | no` and default 0,
registered read-only for snapshots only
(OpenZFS `zfs-2.4.4`,
`module/zcommon/zfs_prop.c:544` to `:546`;
its enum member at `include/sys/fs/zfs.h:151`;
its boolean table at `module/zcommon/zfs_prop.c:318` to `:322`).

F2.
Its value is computed,
not stored:
`dsl_get_defer_destroy` returns 1 when the dataset's `ds_flags` hold `DS_FLAG_DEFER_DESTROY` and 0 otherwise
(OpenZFS `zfs-2.4.4`,
`module/zfs/dsl_dataset.c:2622` to `:2626`;
the macro at `include/sys/dsl_dataset.h:76` to `:78`).

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
so `dodefault` answers `ENOENT` and the entry carries `value` alone
(OpenZFS `zfs-2.4.4`,
`module/zfs/dsl_prop.c:48` to `:58`,
and `:1328` to `:1348`;
the key names at `include/sys/fs/zfs.h:282` to `:283`).

F5.
`ZFS_IOC_OBJSET_STATS` fills that property list only when `zc_simple` is zero and a reply buffer is named
(OpenZFS `zfs-2.4.4`,
`module/zfs/zfs_ioctl.c:2251` to `:2252`,
and `:2270` to `:2271`),
and the crate leaves `zc_simple` zero in a fresh command buffer
(`package/zfs/request-to-openzfs-kernel-device/src/layout/command_buffer.rs:109`,
`src/layout/zfs_cmd_t.rs:299`,
`src/requests/request.rs:60` to `:75`).

F6.
Reading it needs no privilege and no delegation:
the request is registered as a dataset read
(OpenZFS `zfs-2.4.4`,
`module/zfs/zfs_ioctl.c:7827` to `:7828`),
whose policy returns 0 in the global zone (`:386` to `:394`).

F7.
A snapshot with clones cannot be destroyed without the defer mark:
a deferred request returns 0 before either test that follows
(OpenZFS `zfs-2.4.4`,
`module/zfs/dsl_destroy.c:68` to `:73`),
while a plain request refuses `EBUSY` for a user hold and `EEXIST` when `ds_num_children` is above one
(`:79` to `:86`).

F8.
The mark is set in the destroy's sync step,
and only when the deferred request met a user hold or more than one child,
which is what a snapshot with clones has
(OpenZFS `zfs-2.4.4`,
`module/zfs/dsl_destroy.c:322` to `:334`).
No code at `zfs-2.4.4` clears the flag:
the assignment at `:327` is the only one in the tree.

F9.
`dds_num_clones` is `ds_num_children - 1`,
so a snapshot with exactly one clone reports 1
(OpenZFS `zfs-2.4.4`,
`module/zfs/dsl_dataset.c:2635` to `:2639`,
filled into the reply at `:2896`).
The crate carries the field as `DmuObjsetStats::dds_num_clones`
(`package/zfs/request-to-openzfs-kernel-device/src/layout/zfs_cmd_t.rs:75`).

F10.
Destroying the last clone's head destroys a marked origin in the same transaction:
the sync step computes `rmorigin` from a clone directory,
the flag on the previous snapshot,
`ds_num_children == 2` and no user refs
(OpenZFS `zfs-2.4.4`,
`module/zfs/dsl_destroy.c:1028` to `:1031`),
and then destroys that snapshot (`:1169` to `:1173`).
The head's own check does not refuse for the mark:
it refuses a long hold (`:776` to `:777`),
snapshots of the clone (`:784` to `:790`),
a child dataset (`:792` to `:800`),
and a long-held origin it would reap (`:802` to `:807`).

F11.
The crate can already read the property list of any dataset or snapshot in one request,
and already asserts the mark on an unmarked dataset in guest-only controls
(`package/zfs/request-to-openzfs-kernel-device/src/kernel_device/live_read_tests.rs:120` to `:153`,
`live_write_tests.rs:144`,
`live_account_tests.rs:87`).
It has no typed accessor for the mark:
nothing outside those tests names `defer_destroy` for reading,
and the mark it writes is the command field `zc_defer_destroy`
(`src/requests/dataset_requests.rs:48` to `:57`,
`src/layout/zfs_cmd_t.rs:196` to `:197`,
`:334` to `:345`).
Whether those guest controls have been run at this tip is not known here.

F12.
The crate's `DESTROY_SNAPS` request omits the `defer` key by construction,
and a control asserts that neither the request nor its bytes name the key
(`package/zfs/request-to-openzfs-kernel-device/src/requests/snapshot_requests_tests.rs:58` to `:78`).
So the snapshot batch of a removal cannot set the mark on any snapshot.

F13.
The wrapper already turns one property of a reply into a value or a described absence,
and refuses rather than treating an unreadable property as a default
(`package/git-policy/cli/src/native/worktree_clone_mount_handler/sweep_dataset_facts.rs:210` to `:225`,
`.../sweep_clone_check.rs:250` to `:252`).

F14.
The Btrfs route already keeps a durable record of a destroy it started and drops it when the destroy did not
happen
(`package/git-policy/cli/src/native/worktree/worktree_removal_interrupted_destroy.rs:22` to `:24`).
Its record is read at a bound,
and the `destroyed=1` line is the only proof its kernel call returned (`:79` to `:84`).

F15.
The deferred-destroy checks' two records have no home here:
the destruction plan is a sibling's,
and the removal intent is the clone's mount desired-state property (SPEC `:6093`,
`:6097` to `:6108`).
The proposal states the fields the add-time binding record must carry,
which the destruction plan's tuple is drawn from (proposal `:414` to `:421`).

F16.
`defer_destroy` cannot be set as a property:
`libzfs` refuses a read-only property before it sends any request
(OpenZFS `zfs-2.4.4`,
`lib/libzfs/libzfs_dataset.c:1202` to `:1210`),
and the kernel's `zfs_check_settable` returns `EINVAL` for any non-user property named on a snapshot
(`module/zfs/zfs_ioctl.c:5016` to `:5017`,
reached from `zfs_set_prop_nvlist` at `:2914`).
`zfs_ioc_set_prop` was not traced end to end,
so this claims the two refusal points read,
not that every route into the kernel refuses.

F17.
The origin snapshot's reply carries its `createtxg`,
its `userrefs` and its mark,
so one read supplies every number the check and its message use
(OpenZFS `zfs-2.4.4`,
`module/zfs/dsl_dataset.c:2842` to `:2843`,
`:2854` to `:2855`,
`:2856` to `:2857`).

F18.
A marked snapshot with clones still cannot be destroyed by name:
the mark makes a *deferred* request return success,
and a plain request still refuses `EEXIST` while a clone exists (F7).
So the mark goes only when its last clone goes,
which the check exists to keep out of this removal (F10,
F8).

F19.
`doctor` already has the sentences a kept removal needs,
and the sentence that reports the check "as not yet designed" is the implementation plan's,
not the proposal's
(proposal `:1241` to `:1245`;
implementation plan `:2065`,
`:2112` to `:2113`).

F20.
A managed slot's removal refuses rather than falling through to a route that skips a proof
(proposal `:810` to `:812`),
and `--force` waives no identity check (proposal `:806` to `:808`).

F21.
The man page states what a mark means for a snapshot that still has clones:
it "remains visible,
and the user may create new clones from it and place new holds on it",
and the read-only properties `defer_destroy` and `userrefs` decide eligibility and marked status
(OpenZFS `zfs-2.4.4`,
`man/man8/zfs-destroy.8:110` to `:123`,
`man/man7/zfsprops.7:153` to `:157`).

F22.
The binding the project chose is the one the proposal's own words name:
"Use `package/zfs/request-to-openzfs-kernel-device`,
with Cargo package `monochromatic-zfs-request-to-openzfs-kernel-device`"
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

F23.
A user hold exists only on a snapshot:
`zfs hold` refuses a name with no `@`
(OpenZFS `zfs-2.4.4`,
`cmd/zfs/zfs_main.c:6792` to `:6799`;
the kernel refuses it with `EINVAL` at `module/zfs/dsl_userhold.c:121` to `:124`).
A user hold on a clone's own snapshot therefore blocks that snapshot's destroy with `EBUSY` (F7),
and never the clone's head.
What refuses the head after a snapshot batch is a long hold
(`module/zfs/dsl_destroy.c:776` to `:777`),
a snapshot the clone still has (`:784` to `:790`),
a child dataset (`:792` to `:800`),
or a long-held origin it would reap (`:802` to `:807`).
"Measurements owed" uses this to pick a forcing that needs no timing.

F24.
A measurement route exists that a shipped build cannot take:
the crate's package has the `measurement-routes` feature off by default,
described as "Off by default and never enabled by a shipped build",
"so no setting can reach a route the shipped code does not take"
(`package/git-policy/cli/Cargo.toml:71` to `:75`),
and the clone fence is switched by an env var inside such a build
(`.../worktree_clone_conversion_beside_the_clone.rs:123` to `:126`).

## Assumptions

A1.
The destruction plan carries the clone's name and GUID,
and the origin's dataset name and GUID,
snapshot name,
snapshot GUID and `createtxg`,
at the moment the check runs,
because the sibling design places exactly those fields
(`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
"the identities the deferred-destroy check compares with live",
its field list under "What the record holds").
If that design changes those fields,
the check reads the missing ones from the add-time binding record instead,
and V2 in "Choices this design marks for the project's veto" changes with it.
The destruction plan is called the plan from here on;
the implementation plan is always named in full.

A2.
The check runs after the verified unmount and after the destruction plan's first write,
because that is the accepted order (proposal `:886` to `:897`)
and the plan is written "after the unmount and not before it"
(sibling destruction-plan design,
"The first write is after the unmount and not before it").
The removal intent is the clone's mount desired-state property
(`package/git-policy/cli/SPEC.md:6093`,
`:6097` to `:6108`),
not a separate begun record:
the plan's sibling says this route writes none
(its "The removal intent" section).

A3.
The removal's kernel calls sit behind a seam a control can drive without the device,
as the sweep's reads do.
The existing `DatasetReader` is read-only
(`package/git-policy/cli/src/native/worktree_clone_mount_handler/sweep_dataset_reader.rs:137` to `:143`),
so the write side is a seam steps 14 and 16 must add,
and this design states only what it needs of it.

A4.
The first call site runs before every snapshot request,
including each chunk of a split
(implementation plan `:1872` to `:1875`),
and the second before the head request.

A5.
Reading an origin snapshot's properties needs no loaded key,
because the sweep reads `keystatus` from the same reply for a clone whose key may be unloaded and refuses on it
(`package/git-policy/cli/SPEC.md:5959` to `:5961`).
The origin snapshot of an encrypted checkout is the case this assumes for.

A6.
A same-account process can mark the origin between any two requests,
and a name-based request is not bound to an earlier read.
The only measurement behind that second half is [VM item 12][vm-12],
which measured *sequenced* replacements,
with another clone of the same origin,
and never a concurrent substitution (proposal `:1534` to `:1538`).

## Limits

L1.
Nothing in this design was run.
Every sentence about the kernel,
the crate's live answers or a removal's behaviour in a guest is source reading,
and the measurements under "Measurements owed" are unpaid.

L2.
The check refuses on the mark alone,
which is stricter than the kernel's reap test (F9,
F10):
a clone with a second clone pinning the origin is refused too,
and so is one whose origin carries a user hold,
although destroying the clone would reap nothing.
That strictness is the decided sentence's ("the origin is not marked for deferred destroy"),
and it is a cost,
not a gap this design can close on its own.

L3.
The mark has no undo at `zfs-2.4.4` (F8),
so a refusal's way forward is a person's decision about that snapshot,
never a wrapper command.

L4.
The check's answer is not bound to the request it precedes;
a same-account process can substitute objects or set the mark in that window,
and that is the accepted residual,
stated under "The accepted residual".

L5.
The two numbers a refusal prints explain the kernel's reap condition (F9,
F10,
F17);
they do not change the decision,
and what the kernel actually does when this design's removal goes ahead is unmeasured until M1 runs.
The message says what the kernel's code says,
not what a run has shown.

L6.
`zfs_ioc_set_prop` was not traced end to end (F16),
so the claim that a property request cannot set the mark rests on the two refusal points read.

L7.
The check adds three ioctls per call.
It ships no limit,
no timeout and no cap,
so nothing here needs a workload measurement under rule `LQM`;
the cost has to fit the removal's own deadline,
which is sized elsewhere and unmeasured.

L8.
The exit status and the exact wording a person sees are proposed here and unverified against the built binary,
and the wrapper's refusal status for a removal is step 14's.

L9.
V3's claim that the start-time layout gate compares no property name is not verified in this round:
the gate compares the command struct's layout and the request numbers (F5,
F22),
and nothing read here shows it compares a property name.

## The design

### The order the checks sit in

A removal of a managed clone does this,
in the accepted order (proposal `:886` to `:900`):

1. Prove the lineage and the candidate identities (proposal step 1).
2. Record the removal intent,
   print the slot,
   the dataset and the exact snapshot names to be lost,
   release local handles and request `unmount` (proposal `:886` to `:888`).
3. After the verified unmount,
   persist the destruction plan (proposal `:894` to `:896`).
4. Require fresh root all-namespace verification and reprove lineage and candidate identities
   immediately before each snapshot request (proposal `:896` to `:899`).
5. Send the snapshot request,
   or skip it for an empty set (proposal `:908`),
   and record its per-name outcomes in the plan.
6. Enumerate the clone's snapshots again,
   then send the head destroy (proposal `:926`).

Both checks therefore run with the clone already unmounted,
the intent already written and the plan already persisted.
The first runs after step 4's verification and reproof and before the plan rewrite that announces a request;
the second after step 6's enumeration and before the plan rewrite that announces the head request
(`head=requested` in the sibling destruction-plan design,
"For each snapshot request" and "Before the head destroy").
A check that ran before a `requested` rewrite is what keeps a refusal from leaving a plan that claims a request
this removal never sent.

Nothing here moves the unmount or the plan,
and this design does not change the accepted order.

### One check, called at every snapshot request and before the head destroy

One function,
`check_planned_origin`,
with one input and one verdict:

- the input is the planned tuple A1 requires,
  the stage,
  and the identity reads of "What the check reads,
  in order";
- the verdict is `Proceed`,
  or `Refused` carrying one typed cause from "The refusals".

It is called at each place the removal is about to send a request:
before every snapshot request,
each chunk of a split included (A4),
and before the head request.
The call sites are step 16's,
because step 16 is what sends the batch and the head destroy
(implementation plan `:1868`,
`:1872` to `:1877`);
step 15 builds the function,
the reads and the record rule (the review's finding on the step split,
"What each build step builds").

The first call site is skipped only when this run sends no snapshot request at all,
which is the empty-set skip (proposal `:908`).
The guarantee the project's answer buys,
"refuses before this run deletes anything",
is then kept by the head check that still runs,
because the only delete left in that run is the head destroy.
This is a writer's reading of the decided sentence's scope,
marked for veto as V1,
and its reason is measurable:
a resumed removal whose batch is done is the only deterministic guest route to a head-check-first refusal.

Neither call site can be skipped by a flag,
an environment variable or a setting.
`--force` waives cleanliness and,
once the owed measurement shows it works,
ends holders;
it waives no identity or mark check (proposal `:806` to `:808`,
F20).

### Where the planned identities come from

The planned identities are read from the destruction plan,
never from the live pool first:
the plan is what step 1 proved,
and a check that re-derived its own expectation from the live objects would agree with any substitution.
The tuple the check consumes is:

- `clone_name`:
  the clone's full dataset name,
  one plain child of the pinned clone parent;
- `clone_guid`:
  the clone's dataset GUID,
  as step 1 proved it (proposal `:854` to `:857`);
- `origin_dataset_name`,
  `origin_dataset_guid`:
  the origin dataset's name and GUID;
- `origin_snapshot_name`:
  `<origin dataset>@<snapshot>`,
  the snapshot step 1 resolved from the clone's live origin and compared with the recorded origin;
- `origin_snapshot_guid`,
  `origin_snapshot_createtxg`:
  the tuple step 1 recorded for that snapshot.

The live side is read by the planned names and compared field by field.
It is never read by whatever name happens to be live,
because "never use names,
properties or a matching result alone to prove ownership" (proposal `:426`).

A plan that lacks one of these fields is a refusal,
not a check with a missing expectation:
`PlanIncomplete` names the field and the plan's home,
and nothing is deleted.

### What the check reads, in order

Each check makes three `ZFS_IOC_OBJSET_STATS` reads through the crate,
in this order,
and stops at the first disagreement:

1. The clone,
   at `clone_name`.
   The reply must call it a filesystem (`dds_type` is `DMU_OST_ZFS`,
   `dds_is_snapshot` is zero),
   report `clone_guid`,
   and report an origin equal to `origin_snapshot_name`.
2. The origin dataset,
   at `origin_dataset_name`.
   The reply must call it a filesystem and report `origin_dataset_guid`.
   This read is what catches an origin dataset destroyed and received again under the same name,
   whose snapshot can keep its GUID.
   It is a writer's choice,
   marked for veto as V9,
   because the decided sentence does not say how much of "its origin" is compared.
3. The origin snapshot,
   at `origin_snapshot_name`.
   The reply must call it a snapshot,
   report `origin_snapshot_guid`,
   report `createtxg` equal to `origin_snapshot_createtxg`,
   and carry the mark and the two numbers the refusal needs (F17).

The order is the design's,
and it is the part a control pins:
the mark is only ever read from an object whose identity the same reply proved,
because a mark read from a replacement proves nothing about the planned snapshot.
Reading the origin at the planned name also proves the name still belongs to the recorded dataset,
and a rename or a promotion that moved the snapshot makes the live origin name differ,
which step 1's comparison already refuses.

The clone's reply carries its live origin in `dds_origin`,
which the crate already exposes as `DmuObjsetStats::origin()`
(`package/zfs/request-to-openzfs-kernel-device/src/layout/zfs_cmd_t.rs:91` to `:105`).

No other request is made.
The check does not list snapshots and does not read the origin-side inventory:
step 1 did that,
and repeating it here would not bind the later delete either (L4).

### Reading the mark

One request,
already in the crate:
`ZFS_IOC_OBJSET_STATS` (`0x5a12`,
OpenZFS `zfs-2.4.4`,
`include/sys/fs/zfs.h:1502`),
with `zc_simple` zero and a reply buffer,
which the crate's `objset_stats` sends (F5,
F11).

The field of its answer is the property list's `defer_destroy` entry,
and the value is that entry's `value` (`include/sys/fs/zfs.h:282`):
an unsigned integer,
0 for a snapshot not marked and 1 for one marked (F1,
F2,
F3).
The entry carries no `source` for this property at `zfs-2.4.4` (F4),
so the reader reads `value` and never requires or judges `source`;
a test pins that,
because a reader that required `source` would refuse every origin,
marked or not.

What the source says the field means for a snapshot that still has clones:

- a destroy request carrying `defer` returns success for such a snapshot,
  before the `EBUSY` and `EEXIST` tests that a plain request meets (F7);
- the sync step then marks it rather than destroying it (F8),
  and the snapshot stays visible,
  clonable and holdable (F21);
- it is destroyed when its last child or hold goes,
  and for the case this check guards the last child is the clone a removal is about to destroy:
  the head destroy's own sync step destroys the marked origin in the same transaction (F10).

So `defer_destroy` reading 1 on the planned origin snapshot means:
some program asked the kernel to destroy a snapshot that still has clones,
and the next clone destroy can be the transaction that does it.
The two programs on this host that prune snapshots pass no deferred destroy,
and that exposure is reasoning from the grant model rather than a measured account-run deferred destroy
(proposal `:1526` to `:1528`),
which is what makes the check the compensation for the delegated grant.

The numbers the refusal prints are `dds_num_clones` and `userrefs` from the same reply (F9,
F17):
the reap needs exactly one clone and no user refs (F10),
so one clone and `userrefs` 0 mean this removal would be the transaction that reaps the marked snapshot,
and anything else means the snapshot stands after this removal.
Both refuse all the same:
the decided sentence reads the mark,
not the reap (L2).
What the kernel then does is unmeasured until M1 runs,
and the message is worded as the kernel's condition,
not as a measured result (L5).

### The refusals

Every refusal names the object,
the field that disagreed with the recorded and the live value,
what this run has already deleted,
and every way forward,
in the idiom "The refusals" of the descriptor mount records
(`package/git-policy/cli/SPEC.md:5834` to `:5840`;
`sweep_clone_check.rs:206` to `:280`).

One template,
with the fields a control compares,
covers every cause:

```text
refusing to remove the ZFS worktree clone `<clone name>`: <the cause, with the object and the field>.
<what this run deleted, or that it deleted nothing>. Ways forward: <the reads and the acts a person has>.
```

The stage sentence is the only part that differs between the two call sites.
At the first call site it reads:
`Nothing was deleted by this run: the clone, its own snapshots, the origin snapshot and the Git registration all
stand. The removal is pending and the clone is unmounted; cancel-worktree-removal <slot> puts it back.`
At the second call site it reads:
`This run already destroyed the clone's own snapshots (<the names, comma separated>), and they do not come back.
The clone, the origin snapshot and the Git registration stand.
The removal is pending and the clone is unmounted; cancel-worktree-removal <slot> is the way back and reports the
snapshots already destroyed.`

The causes,
each with the object and field the message names:

- `OriginMarkedForDeferredDestroy`:
  the origin snapshot is marked,
  and the message names `dds_num_clones` and `userrefs` and what they mean (F9,
  F10,
  F17).
- `PlanIncomplete`:
  the destruction plan does not carry the field named,
  so no object is proven and nothing is judged.
- `CloneGone`:
  the planned clone is not there (the kernel answered `ENOENT`).
- `CloneIsNotAFilesystem`:
  the object at the clone's name is not a filesystem or is a snapshot.
- `CloneIsNotThePlannedOne`:
  `clone_guid` differs from the plan's.
- `CloneHasNoOrigin`:
  the clone reports no origin.
- `CloneOriginDiffers`:
  the clone's live origin is not `origin_snapshot_name`.
- `OriginDatasetIsNotThePlannedOne`:
  the origin dataset's GUID differs from the plan's.
- `OriginDatasetGone`:
  the origin dataset is not there.
- `OriginIsNotASnapshot`:
  the object at the origin snapshot's name is not a snapshot.
- `OriginSnapshotIsNotThePlannedOne`:
  `origin_snapshot_guid` differs from the plan's.
- `OriginSnapshotCreatetxgDiffers`:
  the origin snapshot's `createtxg` differs from the plan's.
- `OriginSnapshotGone`:
  the origin snapshot is not there,
  so its mark cannot be judged.
- `MarkUnreadable`:
  the reply has no `defer_destroy`,
  or its value is not 0 or 1,
  or the property list is missing,
  or the start-time gate refused raw requests,
  or the device could not be opened.
  This one never falls through to "not marked":
  an unreadable mark is not an off mark (F13),
  and the property read's own words say which state it found
  (`sweep_dataset_facts.rs:146` to `:152`).

`CloneGone` and `OriginSnapshotGone` are the two causes whose verdict needs a word,
because a removal can meet them legitimately after an interruption.
This design's choice,
marked for veto as V10:
both refuse,
and the message names the plan's tuple and says the settle rules of the resume own the next attempt
(proposal `:963` to `:967`).
The check never proceeds over a gone object,
because the request that follows would act on a name nobody proved.

Two full messages,
the first for the cause this item exists for:

```text
refusing to remove the ZFS worktree clone `monochromatic-git-worktree-clone-datasets/fix-flaky-test`: its origin
snapshot `monochromatic-git-checkout@git-worktree-clone-origin-fix-flaky-test` is marked for deferred destroy
(`defer_destroy` reads 1), and the kernel reports 1 clone of it and 0 user holds, so destroying this clone destroys
that snapshot in the same transaction. Nothing was deleted by this run: the clone, its own snapshots, the origin
snapshot and the Git registration all stand. The removal is pending and the clone is unmounted;
`cancel-worktree-removal fix-flaky-test` puts it back. Ways forward: `zfs get -H -o value defer_destroy
monochromatic-git-checkout@git-worktree-clone-origin-fix-flaky-test` and `zfs holds
monochromatic-git-checkout@git-worktree-clone-origin-fix-flaky-test` show the mark and the holds. The mark cannot
be cleared in place, so this slot's removal finishes only outside the wrapper: either you accept that snapshot's
destruction and destroy the clone yourself, or you clone that snapshot elsewhere first, which pins it, and then
destroy the clone; the next run of the removal finds the head gone and settles it. If neither is right for you,
leave the slot and ask the administrator.
```

and the same cause at the second call site:

```text
refusing to remove the ZFS worktree clone `monochromatic-git-worktree-clone-datasets/fix-flaky-test`: its origin
snapshot `monochromatic-git-checkout@git-worktree-clone-origin-fix-flaky-test` is marked for deferred destroy
(`defer_destroy` reads 1), and the kernel reports 1 clone of it and 0 user holds, so destroying this clone destroys
that snapshot in the same transaction. This run already destroyed the clone's own snapshots
(`fix-flaky-test@manual-2026-11-21`, `fix-flaky-test@sanoid-hourly-2026-11-22`), and they do not come back. The
clone, the origin snapshot and the Git registration stand. The removal is pending and the clone is unmounted;
`cancel-worktree-removal fix-flaky-test` is the way back and reports the snapshots already destroyed. Ways
forward: as for the first refusal, and the loss above is the cost already paid.
```

A refusal prints no destructive command.
It prints the reads that show the state and names the decisions a person takes;
whether a wrapper command should ever destroy a marked origin is question Q2.
The `doctor` wording that repeats a refusal is step 18's,
not this design's (F19).

The check writes nothing,
so no refusal can leave half a record,
and there is no lock to take (rule `NLK`).

### What a refusal leaves, and which record it keeps

At either call site the clone is already unmounted,
the intent already stands and the destruction plan already exists.
So a refusal cannot leave the slot as it was,
and dropping the record would mean rewriting the intent to `mount` and reattaching,
which is the one remount the project allows and only for a removal that cancels itself because a holder blocks
the unmount (proposal `:1492` to `:1498`,
`:2103` to `:2104`).

This design therefore adopts one rule,
marked for veto as V4:

- A refusal keeps the removal intent and the destruction plan.
  On the machine that leaves:
  the clone unmounted and,
  where the staging route unmounted it,
  staged under `~/.cache`;
  the intent property reading `v1:<nonce>:unmount`;
  the plan with each verified tuple and each outcome this removal recorded;
  the origin snapshot and its mark untouched;
  the Git registration and the pointer intact.
  A later run resumes from that state (proposal `:963` to `:967`),
  `doctor` reports the standing intent and the last verified loss when step 18 lands (F19),
  and `cancel-worktree-removal <slot>` is the way back,
  reporting any snapshot already destroyed.
- Nothing about a refusal rewrites the intent,
  reattaches the clone,
  unlinks the plan or removes a registration.
  That is the property the records' lean needs (proposal `:2103` to `:2104`),
  and it is the rule both sibling designs assume
  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
  "This design assumes it keeps the plan";
  the destruction-plan design removes a plan only after a completed removal's settle,
  "When it is removed").

What a kept plan then needs from a person is question Q1's subject for the second call site,
and question QA's for the first.

### How a person finishes a refused removal

The mark cannot be cleared (F8),
and a marked snapshot cannot be destroyed while any clone of it exists (F18).
So while the mark stands the wrapper refuses every attempt at that slot,
and only three things can then happen:

- The person accepts the destruction.
  Outside the wrapper they destroy the clone (`zfs destroy` of its remaining snapshots,
  then of the head,
  or `zfs destroy -r` of the clone),
  which reaps the marked origin in the same transaction (F10).
  The next run of the removal finds the head gone and settles it by the absence rule
  (proposal `:963` to `:967`),
  then settles the registration and the pointer,
  leaves the origin-cleanup record where an origin remains (a sibling's design),
  and the plan is unlinked by the removal that completed (the sibling's "When it is removed").
- The person keeps the history.
  Outside the wrapper they first clone the marked snapshot under another name,
  which pins it (F21),
  and then destroy the slot's clone the same way.
  The marked snapshot survives under the new clone,
  and this removal settles exactly as the one above.
- The person cancels the removal with `cancel-worktree-removal <slot>`,
  which is the sibling's design,
  and leaves the slot unmounted or reattached as that design's checked reattach decides.
  The mark still stands,
  so a later removal attempt refuses again until one of the first two happens.

The first message says these two completions and the cancel command,
and the second repeats that the loss is already paid.
The wrapper itself never finishes a removal over a marked origin,
which is the decided refusal and question Q2's subject.

### The accepted residual

Stated exactly,
and left open as the decision leaves it:

- Between either check and the delete it precedes,
  a process of the same account can replace the clone or the origin,
  or mark the origin.
  The check's answer is a read,
  and the delete is a later request that names objects rather than GUIDs (A6,
  L4).
- Between two snapshot requests of one run,
  a mark set after the first chunk's check is caught by the next chunk's check,
  which refuses before that chunk.
  The snapshots the first chunk destroyed are gone by then,
  and no later refusal brings them back (proposal `:934` to `:936`).
- Between the second check and the head request,
  a mark set on the origin is not caught,
  and the head request is the transaction that reaps the marked origin (F10).
- A mark set between the two checks is caught by the second check,
  and the price is the snapshots the batch destroyed before it (proposal `:938` to `:940`).
  The second check therefore does not close the between-checks window:
  it stops the reap after the clone's own history has already gone.
- The delegated answer accepts all of these windows rather than asking for a root-run conditional destroy,
  which is the alternative the project did not select (proposal,
  "Clone removal identity protection").
  Nothing here closes them,
  and no test claims to.

### Where the code lives, and what it exports

A new folder,
`package/git-policy/cli/src/native/worktree_removal_deferred_origin_check/`,
with a top doc listing its files in dependency order (rule `LYF`):

- `planned_origin.rs`:
  the tuple A1 requires and its reading from the destruction plan,
  and `PlanIncomplete`;
- `live_origin.rs`:
  the three reads and the mark's extraction,
  over the shared reader;
- `origin_verdict.rs`:
  the ordered comparison of "What the check reads,
  in order" and the stage;
- `origin_refusals.rs`:
  the causes,
  the template and their words;
- `mod.rs` and `uses/`:
  the folder's links to the folders its code uses,
  including the destruction plan's module.

The folder exports two things,
because a sibling needs the second:

- `check_planned_origin`,
  the verdict function;
- `origin_mark`,
  one read of one dataset's or snapshot's `defer_destroy` value and the states it can be in,
  which `doc/planning/cli-git-zfs-origin-cleanup-record-design.md` assumes is "available to this design's attempt
  as a read".
  Its own attempt reads the mark for the cleanup,
  and it must not have to make a second implementation.

Two moves are a precaution,
both marked for veto as V6:
`PropertyValue` and `value_of` move from
`worktree_clone_mount_handler/sweep_dataset_facts.rs` into
`package/git-policy/cli/src/native/zfs_property_value/`,
and `DatasetReader`,
`ReadFailure` and `ListedChild` move from
`worktree_clone_mount_handler/sweep_dataset_reader.rs` into
`package/git-policy/cli/src/native/zfs_dataset_reader/`,
with the handler re-exporting both so its callers do not change.
The rules `LYL` and `LYF` do not require them today:
a folder using another folder one way is no cycle.
They keep the folder graph a tree,
and they move files merged with step 8,
so the build checks what the step 9 branches in flight touch before it moves anything.

What the check needs from the removal,
and this design does not design:

- the destruction plan's reader,
  with the fields A1 names and an absent field it can name;
- the removal intent's property and the rule that a refusal never rewrites it (A2);
- a seam at the removal's kernel calls,
  so a control can count that no write was sent after a refusal and can set a mark at the moment between two
  calls (A3);
- the removal's deadline,
  which bounds the three extra reads without a limit of its own (L7).

### What each build step builds

- Step 15 builds the function,
  the three reads,
  the exported `origin_mark`,
  the plan's field requirements as a refusal,
  the causes and the refusal words,
  the keep rule,
  and the controls and measurements that need no batch.
  Step 15 owns the record rule for both call sites,
  because the implementation plan's step 15 builds "What each refusal prints,
  and whether a refusal keeps the removal record"
  (`:1818` to `:1822`).
- Step 16 places both calls:
  one before each snapshot request,
  each chunk included
  (implementation plan `:1868`,
  `:1872` to `:1875`),
  and one after the fresh enumeration and before the head request (`:1876` to `:1877`).
  It also adds the write seam the refusals' controls drive,
  and it runs M3 once the route is whole.
- Step 18 builds `doctor`'s lines from the kept intent and plan,
  and replaces the sentence that reports the check "as not yet designed"
  (implementation plan `:2065`,
  `:2112` to `:2113`).
  It depends on steps 7,
  9 and 14 (`:2116` to `:2119`),
  not on this one,
  so nothing here may assume its lines exist when M3 runs.
- No step ships the check behind a setting,
  and step 15 lands ahead of step 16,
  so no build reaches a snapshot delete of this route without the check at its call site.

## The tests a build must have

Every control below except the two guest ones runs on the host against the seams (A3),
no device and no pool,
following `sweep_one_clone_tests.rs:158`,
which already builds a `DatasetIdentity` by hand.

### Controls that need no device

1. `a_marked_origin_refuses_before_the_snapshot_request`:
   the mark reads 1,
   the verdict is `Refused`,
   and the fixture that stands for the snapshot request records no call.
2. `an_unmarked_origin_proceeds_after_exactly_three_identity_reads`:
   the fixture counts three `identity` calls,
   in the order clone,
   origin dataset,
   origin snapshot,
   and no other call.
3. `the_mark_is_read_from_the_value_and_not_the_source`:
   a pair with `value` alone,
   a pair with `value` and a `source`,
   and a pair whose `value` is text;
   only the integers 0 and 1 decide.
4. `an_unreadable_mark_refuses_rather_than_proceeding`:
   absent key,
   missing property list,
   text value,
   a value of 2,
   and a gate refusal each produce `MarkUnreadable` with its own state named.
5. `a_replaced_clone_refuses_before_the_origin_is_read`:
   `clone_guid` differs,
   and the fixture records that neither origin read ran.
6. `a_clone_whose_live_origin_differs_refuses_before_the_origin_is_read`,
   and `a_clone_with_no_origin_refuses`.
7. `a_replaced_origin_dataset_refuses`,
   `a_replaced_origin_snapshot_refuses`,
   and `an_origin_whose_createtxg_differs_refuses`,
   each naming its own cause.
8. `a_name_that_is_not_the_planned_kind_refuses`:
   the clone's name answers as a snapshot,
   and the origin snapshot's name answers as a filesystem,
   each with its own cause.
9. `an_incomplete_plan_names_the_field_and_deletes_nothing`.
10. `every_cause_prints_the_template_with_its_object_the_field_and_the_ways_forward`:
    one message per cause,
    compared against the template and the fields,
    at both stages (the review's finding on the message texts).
11. `a_gone_clone_or_origin_snapshot_refuses_and_names_the_settle`:
    `ENOENT` on either,
    with the plan's tuple named and the settle rule pointed at (V10).
12. `a_marked_origin_with_a_second_clone_still_refuses_and_says_the_origin_is_pinned`:
    `dds_num_clones` is 2,
    the verdict is `Refused`,
    and the message says the origin is pinned by another clone.
13. `a_marked_origin_under_a_user_hold_still_refuses_and_says_the_reap_will_not_happen`:
    `userrefs` is 1 and `dds_num_clones` is 1,
    the verdict is `Refused`,
    and the message says the snapshot stands after this removal.
14. `a_refusal_keeps_the_intent_and_the_plan`:
    neither the intent rewrite nor the plan unlink is called,
    at either stage.
15. `a_refusal_before_a_second_chunk_stops_the_chunks_after_it`:
    the fixture counts one snapshot request,
    the second chunk's check refuses,
    and no second request is recorded.
16. `the_check_runs_after_the_enumeration_and_before_the_requested_rewrite`:
    the fixture records the call order of one head step,
    the enumeration before the check,
    and the plan rewrite announcing `head=requested` after it.

### The races, each forced without timing luck

R1.
A mark set between the first check and its snapshot request.
Forced by the batch fixture,
which sets the mark before it returns;
the next check refuses and the fixture records no second request.
This is the single-invocation arm,
and what it proves is bounded:
the second check stops the head destroy,
after the first batch's snapshots have gone (the review's finding on the residual,
"The accepted residual").

R2.
A mark set between the chunk two check and its request.
Forced the same way with two chunks;
the third check refuses and the third request is never recorded.

R3.
A mark set between the head check and the head request.
Forced by the head fixture,
which sets the mark and then receives the request.
The control asserts the request is sent,
which documents the residual rather than closing it (L4).
The guest arm measures what the kernel then does.

R4.
The origin snapshot destroyed or replaced between two checks.
Forced by the origin fixture,
which answers `ENOENT` or another GUID on the later call;
the verdict is `OriginSnapshotGone` or `OriginSnapshotIsNotThePlannedOne`,
and the fixture records no request.
R5.
The clone replaced between two checks,
forced with another GUID;
the verdict is `CloneIsNotThePlannedOne` and no request follows.

R6.
A same-account substitution between a check and the request it precedes.
Not forced and not claimed:
it is L4,
and the measured arms of [VM item 12][vm-12] are its record.

### Controls that need a guest

17. `a_marked_snapshot_reports_the_mark_and_an_unmarked_one_does_not`,
    in `package/zfs/request-to-openzfs-kernel-device/src/kernel_device/live_read_tests.rs`:
    the identity read on a snapshot before a deferred destroy reads 0,
    after it reads 1,
    the entry carries no `source`,
    and `dds_num_clones` follows F9's meaning.
    It runs through the package's e2e recipe like the other `#[ignore]` live controls
    (`live_read_tests.rs:120` to `:121`).
18. The removal arms of M3 below,
    in a new guest recipe under `package/git-policy/cli/e2e/`,
    driven the way `mount-git-worktree-clone-datasets-guest.ts` drives the step 8 arms.

## Measurements owed

None of these ran in this round.
Each is written to be run by a later round as it stands,
and each names every account and grant it needs.

### M0a: the kernel fixture, for M1 and M2

One file-backed pool in a disposable guest,
as root,
with the account the design's delegated route runs as.
The dataset names are the design's,
so the same pool can carry M0b.
The account's grant is the set [VM item 17][vm-17] used,
less the checkout's local `snapshot`,
so the mark's `zfs destroy -d` is the delegated act the exposure is about.

```bash
mkdir --parents /var/lib/zfs-fixtures/deferred
truncate --size 4G /var/lib/zfs-fixtures/deferred/pool.img
zpool create zfsdeferred /var/lib/zfs-fixtures/deferred/pool.img
zfs create -o mountpoint=/var/lib/zfs-fixtures/deferred/checkout zfsdeferred/monochromatic-git-checkout
zfs create -o mountpoint=legacy zfsdeferred/monochromatic-git-worktree-clone-datasets
printf 'history\n' > /var/lib/zfs-fixtures/deferred/checkout/history
zfs snapshot zfsdeferred/monochromatic-git-checkout@git-worktree-clone-origin-fix-flaky-test
zfs clone zfsdeferred/monochromatic-git-checkout@git-worktree-clone-origin-fix-flaky-test \
    zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test
zfs snapshot zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test@manual-2026-11-21
zfs snapshot zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test@sanoid-hourly-2026-11-22
useradd --create-home --uid 1000 probe
zfs allow -l -u probe mount zfsdeferred/monochromatic-git-worktree-clone-datasets
zfs allow -d -u probe mount,destroy zfsdeferred/monochromatic-git-checkout
zfs allow -u probe destroy zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test
```

The grant commands are the fixture's,
not the design's:
the design's preflight checks effective grants as [VM item 17][vm-17] recorded them.
Each arm below records the grant set it ran under.

### M0b: the managed slot, for M3

M3 removes a slot the wrapper manages,
so its fixture is made by the built add route,
never by hand:
a hand-made registration is neither the binding record nor the intent the removal reads.

It needs the built add (step 10),
the built removal (steps 14,
15 and 16),
and the mount handler of steps 6 to 9 installed as the step 8 guest recipe installs it.
Until those land,
M3 waits,
and M1 and M2 do not.

```bash
# the checkout is a Git repository on the checkout dataset
git init --initial-branch=main /var/lib/zfs-fixtures/deferred/checkout
git -C /var/lib/zfs-fixtures/deferred/checkout config user.name probe
git -C /var/lib/zfs-fixtures/deferred/checkout config user.email probe@example.invalid
printf 'history\n' > /var/lib/zfs-fixtures/deferred/checkout/history
git -C /var/lib/zfs-fixtures/deferred/checkout add history
git -C /var/lib/zfs-fixtures/deferred/checkout commit --message 'the fixture'
# the built wrapper's add, as the account, which writes the binding, the origin snapshot and the clone
sudo --user probe env PATH=<the built wrapper's bin>:$PATH \
    git worktree add /var/lib/zfs-fixtures/deferred/worktrees/fix-flaky-test
```

Each M3 arm then needs the slot's own snapshots;
M0b adds them on the clone the add made,
by name,
before the arm:

```bash
zfs snapshot zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test@manual-2026-11-21
zfs snapshot zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test@sanoid-hourly-2026-11-22
```

### M1: the reap, the positive control

Owed before the refusal arms,
because a harness that cannot see the loss cannot show that the refusal avoids it (rule `QPC`).
It needs no built wrapper,
and it runs on M0a's hand-made clone.

```bash
# the read before the mark, the way the check reads it
zfs get -H -o value defer_destroy zfsdeferred/monochromatic-git-checkout@git-worktree-clone-origin-fix-flaky-test
# the mark, as the account that holds the descendant destroy grant
sudo --user probe zfs destroy -d zfsdeferred/monochromatic-git-checkout@git-worktree-clone-origin-fix-flaky-test
zfs get -H -o value defer_destroy zfsdeferred/monochromatic-git-checkout@git-worktree-clone-origin-fix-flaky-test
# the removal's order, by hand: the clone's own snapshots, then the head
zfs destroy zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test@manual-2026-11-21
zfs destroy zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test@sanoid-hourly-2026-11-22
zfs destroy zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test
zfs list -H -o name zfsdeferred/monochromatic-git-checkout@git-worktree-clone-origin-fix-flaky-test
```

Recorded,
in `keep/`,
as one log:
the first read answers `off`;
`zfs destroy -d` exits 0 and prints nothing;
the second read answers `on`;
each clone destroy exits 0;
and the last listing exits non-zero naming no such dataset,
so the marked snapshot went with the head in one transaction.
Pass is that sequence.
Fail,
and the item returns to the owner,
is the marked snapshot still present after the head destroy:
the check would then guard a reap this release does not perform,
and L5's sentence about the message would need rewording.

### M2: the mark read through the binding

Control 17 above,
run on M0a's pool in the same guest,
with its own log.
It shows that the field the check reads is the field the kernel fills,
for both values,
through the crate rather than through `zfs`.

### M3: the wrapper's refusals

It needs M0b's managed slot.
Each arm states whether it is a control (an assertion that must hold) or a reach report (counts).

#### M3a: the first check refuses (control)
The origin is marked before the run,
by M0a's account.
The removal runs as the account,
through the built wrapper.

```bash
sudo --user probe zfs destroy -d \
    zfsdeferred/monochromatic-git-checkout@git-worktree-clone-origin-fix-flaky-test
sudo --user probe env PATH=<the built wrapper's bin>:$PATH \
    git worktree remove /var/lib/zfs-fixtures/deferred/worktrees/fix-flaky-test
```

Recorded:
the message (the first template),
the exit status,
and after the run `zfs list -H -o name,guid` for the clone and both its snapshots,
`zfs get -H -o value,source defer_destroy` for the origin snapshot,
`zfs get -H -o value org.monochromatic:git-worktree-clone-mount-desired-state` for the clone,
the destruction plan's bytes,
and `git worktree list`.
Pass is the first-stage refusal,
the clone and both its snapshots present,
the origin and the marked snapshot present,
the intent reading `unmount`,
the plan present with no outcome,
the registration listed,
and no snapshot destroyed.

#### M3b: a replacement clone whose different origin is marked (control)
The planned clone is destroyed outside the wrapper after its own snapshots,
a second checkout snapshot is made and marked,
and a same-name clone is made from it:

```bash
zfs destroy zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test@manual-2026-11-21
zfs destroy zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test@sanoid-hourly-2026-11-22
zfs destroy zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test
zfs snapshot zfsdeferred/monochromatic-git-checkout@other
sudo --user probe zfs destroy -d zfsdeferred/monochromatic-git-checkout@other
zfs clone zfsdeferred/monochromatic-git-checkout@other \
    zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test
```

The removal must refuse on `CloneIsNotThePlannedOne`,
naming the plan's GUID and the live one,
before any delete.
This is the arm the implementation plan lists as step 15's acceptance test
(`:1843` to `:1844`).

#### M3c: the check before the head destroy refuses (control)
Forced with the measurement route of F24,
never with a watcher:
the guest build is made with `--features measurement-routes`,
and the removal reads `MONOCHROMATIC_ZFS_REMOVAL_PAUSE` as a file path.
After the batch's per-name outcomes are recorded and before the second check,
that build writes `<path>.ready` and waits for `<path>.go`,
bounded by the removal's own deadline;
a pause that outlives the deadline is not honoured and the arm records that.

```bash
MONOCHROMATIC_ZFS_REMOVAL_PAUSE=/var/lib/zfs-fixtures/deferred/pause \
    sudo --user probe env PATH=<the built wrapper's bin>:$PATH \
    git worktree remove /var/lib/zfs-fixtures/deferred/worktrees/fix-flaky-test &
while [ ! -e /var/lib/zfs-fixtures/deferred/pause.ready ]; do sleep 1; done
sudo --user probe zfs destroy -d \
    zfsdeferred/monochromatic-git-checkout@git-worktree-clone-origin-fix-flaky-test
touch /var/lib/zfs-fixtures/deferred/pause.go
wait
```

Recorded:
the same facts as M3a,
plus the plan's bytes,
which must show the batch's outcomes recorded,
and the clone head present with its own snapshots gone.
Pass is the second-stage refusal,
the origin and the marked snapshot present,
the clone head present,
the plan present with the batch's outcomes,
and no head request issued (the clone still exists).
`doctor`'s lines are not a pass condition here:
they are step 18's (implementation plan `:2116` to `:2119`).

#### M3d: the residual (reach report, not an owed arm)
The proposal says a same-account mark set between the second check and the destroy is the accepted residual,
"not a case this item has to close" (proposal `:1968` to `:1970`).
This design records it as reach only:
repeat M3c with the mark made by a script set to fire while the removal runs,
in one interleaved round with an unchanged-input control,
and report how many runs had the mark seen by the second check and how many ended with the origin reaped.
Rule `QNB` applies:
the control arm's spread is reported beside the counts.

### M4: the cost

The wall time of one removal with the checks against one without,
in the same guest and the same round,
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
  before every snapshot request,
  skipped only when this run sends none.
  Example:
  a resumed removal whose plan says both planned snapshots are gone sends only the head request,
  so only the head check runs,
  and a marked origin refuses there.
  Pro:
  a resume with an empty set is the only deterministic guest route to a head-check-first refusal,
  and the guarantee "before this run deletes anything" holds because the only delete left is the head.
  Con:
  on that resume the decided "also check first" is met by the head check alone.
- Alternative:
  run the check at the first call site even when the snapshot request is skipped.
  Example:
  the same resume refuses at a check that guards no request,
  and no guest run can reach the head-check refusal.
  Pro:
  two checks on every removal.
  Con:
  question Q1 and M3c would be about a state no run can produce.
Ranking:
  adopted over the alternative,
because it is what makes the owed measurement of the head-check refusal runnable.

V2.
Where the check reads the planned identities.
- Adopted:
  from the destruction plan.
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
because the check's job is to re-prove the objects this removal planned.
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
  a release that renames or drops the property stops every removal until the reader learns the name.
  The start-time gate would not catch that,
  though whether it compares property names at all is unverified here (L9).
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
Which record a refusal keeps.
- Adopted:
  both refusals keep the intent and the destruction plan (see "What a refusal leaves").
  Example:
  a first-stage refusal prints the refusal and leaves the clone unmounted with the intent standing,
  the plan present and nothing destroyed;
  a second-stage refusal leaves the same records plus the batch's outcomes.
  Pro:
  it follows the records' lean (proposal `:2103` to `:2104`) and what both siblings assume;
  no remount;
  the plan is evidence.
  Con:
  the worktree is out of use until a person acts.
- Alternative A:
  cancel on a refusal that destroyed nothing,
  rewriting the intent to `mount` and reattaching after the checked proof.
  Example:
  the first-stage refusal prints that the worktree is mounted again and nothing was lost.
  Pro:
  back in use at once.
  Con:
  it uses the one decided remount exception (proposal `:1492` to `:1498`) for a case it does not name,
  needs a surviving verified binding,
  adds the remount race the default exists to avoid,
  and the same removal refuses again while the mark stands.
- Alternative B:
  cancel on the second-stage refusal too,
  rewriting the intent and dropping the plan.
  Example:
  the second-stage refusal reattaches a clone whose snapshots are gone and writes nothing about them.
  Pro:
  one outcome for both stages.
  Con:
  it erases the only statement that history was destroyed.
Ranking:
  adopted > Alternative A > Alternative B.
Adopted over A,
because A's remount is the exception the records gave to a blocked holder,
and it inherits the refusal on the next run.
A over B,
because B loses the loss as well as adding the remount.

V5.
Whether the check writes anything.
- Adopted:
  nothing,
  no record of its own and no lock.
  Example:
  a killed check leaves no staging file and no state a later `doctor` reconciles.
  Pro:
  rule `NLK` holds trivially and a refusal leaves the records as it found them.
  Con:
  nothing records that a check ran,
  so `doctor` reads only the intent and the plan.
- Alternative:
  a check record naming the reads and their answers.
  Example:
  `doctor` prints the last check's claim and cannot say when it was true,
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
  the folder graph stays a tree,
  and a later two-way need has a home.
  Con:
  merged step 8 files move in step 15,
  their controls follow,
  and a step 9 branch in flight may touch the same files.
- Alternative:
  leave them where they are and let the check's folder use the handler's.
  Example:
  no move,
  and the check reads `PropertyValue` from `worktree_clone_mount_handler`.
  Pro:
  no churn in merged code.
  Con:
  the check's folder then depends on a sibling folder's internals,
  and a later ruling they share has no place to live.
Ranking:
  adopted over the alternative,
to be confirmed against the step 9 branches before the build moves anything.

V7.
What a refusal prints.
- Adopted:
  the reads and the decisions,
  never a destructive command.
  Example:
  the marked-origin refusal prints `zfs get` and `zfs holds` and names the two completions and the
  cancel command,
  and prints no `zfs destroy`.
  Pro:
  the wrapper never prints a command that destroys history another program marked.
  Con:
  a person must act outside the wrapper or ask the administrator.
- Alternative:
  print the exact `zfs destroy` for the slot.
  Example:
  a hurried reader copies it,
  and the marked snapshot goes with the clone.
  Pro:
  one copy.
  Con:
  printing it reads as authority,
  and question Q2 is open.
Ranking:
  adopted over the alternative,
because a printed destructive command is read as permission.

V8.
What the refusal says about the reap.
- Adopted:
  it names `dds_num_clones` and `userrefs` from the same reply.
  Example:
  one clone and 0 holds reads "so destroying this clone destroys that snapshot in the same transaction",
  and one clone with 1 hold reads that the snapshot stands after this removal.
  Pro:
  a person learns whether their decision is urgent.
  Con:
  two more numbers in the message,
  and their meaning is the kernel's code until M1 runs (L5).
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

V9.
Whether the origin dataset's own identity is compared.
- Adopted:
  yes,
  a third read compares `origin_dataset_guid`,
  because the destruction plan carries it and the decided sentence names "its origin".
  Example:
  the checkout dataset is destroyed and received again under the same name with a snapshot whose GUID
  survived,
  and the check refuses on the dataset's GUID where two reads would have passed.
  Pro:
  the origin is proven at both levels the plan recorded.
  Con:
  one more ioctl per check,
  and one more read to keep in the ordered comparison.
- Alternative:
  two reads,
  the origin snapshot's name and GUID standing for the dataset.
  Example:
  the same re-received lineage passes on a snapshot GUID that survived the round trip.
  Pro:
  two ioctls instead of three.
  Con:
  the plan's `originDatasetGuid` is never compared,
  and the decided sentence's "origin" narrows to a snapshot.
Ranking:
  adopted over the alternative,
because a field the plan records and the sentence names should be compared,
  and the read is cheap.
The sibling plan design already assumes the check compares it.

V10.
What the check does when a planned object is gone.
- Adopted:
  refuse,
  naming the plan's tuple and the settle rules the next attempt uses (proposal `:963` to `:967`).
  Example:
  an interrupted removal whose clone was destroyed by hand reaches the check,
  which refuses with `CloneGone` and leaves the settle to the resume.
  Pro:
  the check never proceeds over a name nobody proved,
  and a resume settles by the absence rule.
  Con:
  a person who expected the run to settle it sees a refusal that says the rules own it.
- Alternative:
  pass the gone object to the settle step inside the same run.
  Example:
  the same run settles the registration and the plan.
  Pro:
  one run finishes.
  Con:
  the check would then decide the settle rules,
  which another design owns,
  and a partial state it cannot prove would pass.
Ranking:
  adopted over the alternative,
because the check's verdict stays one word and the settle rules stay their owner's.

## Questions only the project can answer

### QA. When the first check refuses, what happens to the clone that is already unmounted?

At that moment the clone is unmounted (possibly staged under `~/.cache`),
the removal intent stands,
the destruction plan is written,
and nothing is destroyed.
This design adopts Option A;
the records lean to it (proposal `:2103` to `:2104`,
`:1241` to `:1245`) and both sibling designs assume it,
but the open item does not decide it.

- Option A,
  keep the intent and the plan;
  the clone stays unmounted until a person acts.
  Example:
  `git worktree remove` of `fix-flaky-test` with
  `monochromatic-git-checkout@git-worktree-clone-origin-fix-flaky-test` marked prints that the origin snapshot is
  marked,
  that nothing was deleted,
  that the clone is unmounted and the removal is pending,
  and that `cancel-worktree-removal fix-flaky-test` puts it back;
  `doctor`,
  when step 18 lands,
  lists the standing intent.
  Pro:
  it follows the records' lean and what both siblings assume;
  no remount,
  no new race;
  the plan is evidence.
  Con:
  the worktree is out of use until a person acts,
  and a person who does not read the message may not know.
- Option B,
  cancel itself:
  rewrite the intent to `mount`,
  reattach after the checked proof,
  drop the plan.
  Example:
  the same command prints the same refusal and then that the worktree is mounted again and nothing was
  lost.
  Pro:
  back in use at once;
  nothing is lost at this point.
  Con:
  it widens the one decided remount exception (proposal `:1492` to `:1498`) from a blocked holder to a
  refused check,
  adds a remount race,
  needs a surviving verified binding (proposal `:2103` to `:2104`),
  and the same removal refuses again while the mark stands.
- Option C,
  an extra check before the unmount,
  reading the binding record,
  so a marked origin refuses while the clone is still mounted,
  with the decided two checks kept.
  Example:
  the same command refuses before any unmount;
  the worktree is untouched;
  a mark set after that check is caught by the decided first check,
  which then needs Option A.
  Pro:
  the common case disturbs nothing.
  Con:
  a third call site and a binding-record read (V2's ranked-over alternative),
  more code,
  and it does not remove the need for A.

Ranking:
A > C > B.
A over C,
because A is what the records lean to and the siblings assume,
and adds nothing to build,
while C adds a
call site that still needs A.
C over B,
because C avoids the remount exception altogether in the common case,
while B uses a decided exception
for a case it does not name.

### Q1. After a refusal before the head destroy, does the removal keep its records or cancel itself?

This is the open item's own question (proposal `:937` to `:942`),
and the records disagree with the item's "undesigned" wording:
the owed measurement expects the record to survive (proposal `:1966` to `:1968`).
This design adopts Option A,
for the same reason as QA.

- Option A,
  keep:
  the intent stays and the plan keeps the batch's outcomes and the last verified loss.
  Example:
  a resumed removal of `fix-flaky-test`,
  its two own snapshots (`fix-flaky-test@manual-2026-11-21`,
  `fix-flaky-test@sanoid-hourly-2026-11-22`) already destroyed,
  the origin marked after the interruption,
  refuses,
  leaves the head and the registration,
  and `doctor` (when step 18 lands) names the slot,
  the mark and the destroyed snapshots;
  `cancel-worktree-removal fix-flaky-test` is the way back and reports them.
  Pro:
  the only statement that history was destroyed survives,
  and the plan already names each tuple.
  Con:
  the worktree stays out of use until a person acts.
- Option B,
  cancel and drop:
  rewrite the intent to `mount`,
  reattach,
  unlink the plan.
  Example:
  the same run reattaches the clone as it stands and writes nothing about the snapshots that went.
  Pro:
  the slot returns to use without a person.
  Con:
  it erases the loss statement,
  it uses the remount exception for a case it does not name,
  and a later removal re-proves against a history that silently shrank.
- Option C,
  cancel and keep the plan:
  rewrite the intent,
  reattach,
  leave the plan as the record of what the
  removal did.
  Example:
  the same run reattaches the clone and leaves the plan,
  which `doctor` reports as a cancelled removal's outcomes.
  Pro:
  both the return to use and the loss statement.
  Con:
  the destruction-plan design has no cancelled state and removes the plan at the end of a completed removal,
  so this needs a state and a removal rule that sibling does not have.

Ranking:
A > C > B.
A over C,
because A needs no new plan state and no remount;
C over B,
because B is the one option that loses the loss.

### Q2. May the wrapper ever finish a removal over a marked origin?

This is an amendment question,
not an open one:
the records decide that such a removal refuses
(`doc/decision/cli-git-zfs-worktree-design.md:46` to `:48`;
proposal `:930` to `:932`;
proposal `:1517` to `:1521`).
Options B and C each reverse a decided sentence;
this design adopts A.

- Option A,
  no:
  the refusal stands,
  and the person finishes outside the wrapper (see "How a person finishes").
  Example:
  the refusal message names the two completions and prints no destructive command.
  Pro:
  the wrapper never destroys history a program other than itself marked,
  and the delegated grant is not
  widened.
  Con:
  the work is manual.
- Option B,
  a guarded command that takes the slot and names the marked snapshot it will destroy,
  once.
  Example:
  `monochromatic-git-policy-cli remove-worktree --accept-deferred-origin <slot>` prints the snapshot's
  name and tuple and needs a second confirmation word.
  Pro:
  the removal finishes inside the proofs the route already has.
  Con:
  a new authority and refusal surface,
  and a person can destroy exactly the history the check exists to protect.
- Option C,
  a `doctor` line the administrator runs,
  never the wrapper.
  Example:
  after the refusal `doctor` prints `zfs destroy monochromatic-git-worktree-clone-datasets/fix-flaky-test`
  as an administrator step,
  and the wrapper never runs it.
  Pro:
  nothing new is authorized in the wrapper.
  Con:
  it is the same manual act with the wrapper's blessing printed,
  which reads as authority it does not have,
  and it reverses the decided refusal.

Ranking:
A > C > B.
A over C,
because C reverses the decided refusal while still letting a person destroy the history,
and printing
it is indistinguishable to a hurried reader from authorizing it.
C over B,
because B also adds the authority the delegated grant was chosen not to carry.

## Dependencies on sibling designs

Each of these is an assumption this design made so it could be whole;
a later pass reconciles the two.
A sibling's document is unreviewed,
so its words are quoted and never adopted as decided.

- "Where a removal's persisted destruction plan lives,
  and its format"
  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`):
  it places every field A1 needs,
  including `originDatasetGuid`,
  and says those are "the identities the deferred-destroy check compares with live";
  that agrees with this design's third read (V9).
  It writes the plan "after the unmount and not before it" and removes it only after a completed removal's settle;
  that agrees with A2 and V4,
  and it means Q1's Option C would need a cancelled state it does not have.
  It writes `requested` before each request and `head=requested` before the head destroy,
  which is why the checks run before those rewrites.
- "What a cancelled,
  timed-out or killed removal leaves,
  and what the cancel command proves and does"
  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`):
  it lists the first-check refusal as a state where "the clone is mounted nowhere ... the intent stands",
  and says "This design assumes it keeps the plan".
  That agrees with V4 and QA's Option A;
  it leaves the record's fate to this design,
  which V4 and Q1 decide.
- "Where the add's durable binding record lives,
  and its format"
  (`doc/planning/cli-git-zfs-add-binding-record-design.md`):
  it says this design "reads the origin identity this design records,
  and gives it no new field".
  That agrees with A1's fallback and V2;
  the plan copies the tuple,
  so the fallback is unexercised unless A1's fields move.
- "The record that carries origin cleanup after a registration is settled"
  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`):
  it assumes "the plan's step 15 check of the origin's deferred-destroy mark is available to this design's attempt
  as a read".
  This design exports `origin_mark` for exactly that,
  and the sibling may use it.
  That sibling also says the mark can be cleared "when an administrator removes the mark",
  which F8 contradicts at `zfs-2.4.4`;
  the source decides,
  and the correction is that sibling's.
- "The forced arm of a removal:
  which holders are ended,
  by whom,
  and how the caller's session is spared"
  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`):
  it says this design "owns step 16's checks",
  which agrees with "What each build step builds";
  it assumes "a refusal rewrites the intent to `mount`",
  which disagrees with V4 and QA's Option A.
  The records decide against that assumption for a refused check:
  the remount exception is the blocked holder's (proposal `:1492` to `:1498`),
  so the sibling's assumption is the one to reconcile,
  and nothing here adopts it.
- "The dirty-main restore on rung 1 and its frozen proof",
  "The add-time Git lock of a ZFS clone worktree",
  "Which triggers a lost last event waits for,
  and how nudges are merged":
  none states anything about the deferred-destroy checks that this design reads,
  and this design depends on none of them.
- "What re-applies private propagation after a replacement mount of the checkout"
  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`):
  it lists the deferred-destroy checks as "no dependency",
  and that agrees with this design:
  the checks read dataset names and properties,
  never a mount fact,
  so a replacement mount changes nothing they read.

## Limitations

- Nothing was measured (L1).
  The design's whole kernel case rests on F1 to F23 read at `zfs-2.4.4`,
  and the host's module is `2.4.4-1`;
  a different release is the gate's question,
  not this design's.
- The check refuses on the mark alone (L2),
  so it refuses removals the kernel would have performed without reaping the marked snapshot.
- A refusal has no in-wrapper completion (L3,
  "How a person finishes").
- The identity proof is by name,
  GUID and `createtxg` in practice,
  not against a 64-bit GUID collision,
  which is the same standard the proposal uses for bookmarks (proposal `:1621` to `:1626`).
- The checks' reads are not bound to the request after them (L4).
- The check adds three ioctls per call and ships no number that bounds them (L7).
- The refusal's exit status and the removal's status for a refusal are step 14's to settle (L8).
- The reap sentence a message prints is the kernel's condition,
  not a measured result (L5).

## What this design does not cover

- The destruction plan's home,
  format,
  writer and resume rules.
- The removal intent's property format and writer.
- The origin-cleanup record.
- The blocked-unmount cancellation,
  `--force`'s holder arm,
  the cancel command and what a timed-out removal
  leaves.
- What `doctor` prints beyond the fields a kept intent and plan must give it.
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
  asking a person for a secret is what this project forbids
  (`package/git-policy/cli/SPEC.md:7345` to `:7350`).
- The rejected root alternative's channel program (proposal,
  "Clone removal identity protection").

## What this revision rewrote

Rewritten after the gate review:
"The item and the sentences that bind it" (the order,
the owed measurement's quotes and the records' disagreement added),
"What this design reuses" (the measurement route and the corrected citations),
F9 and F16,
F17 to F24 (eight facts added or renumbered,
F17 and F18 of the first version moved into the design text),
A2,
A6,
L5,
L6,
L9 and "The design" (the order section added,
the call sites made step 16's and per chunk,
the third read added,
the causes named with a template and full messages,
the keep rule replacing the bucket rule,
"How a person finishes" added,
the residual restated with the losses),
"Where the code lives" (the exported mark reader added),
"What each build step builds" (the split corrected),
the controls and races (two chunk-order controls,
the gone-object cause,
the corrected messages),
"Measurements owed" (a runnable fixture with the account and grants,
the first arm,
the replacement arm,
the seam-forced head arm,
the `doctor` condition dropped),
"Choices this design marks for the project's veto" (V1's reason and V4's alternatives rewritten,
V9 and V10 added,
V6 restated as a precaution),
"Questions only the project can answer" (QA added,
Q1 reframed with the records' lean,
Q2 made an amendment question with Option C's example),
"Dependencies on sibling designs" (the siblings' words,
three disagreements,
two dropped dependencies),
"Limitations",
"What this design does not cover",
"Sources read" and "Revisions".

Left as the first version wrote them:
F1 to F8,
F10 to F15,
F21 to F24 (with citations checked),
"Reading the mark",
"Where the planned identities come from" (but for the origin dataset),
"The accepted residual"'s kernel sentences,
A1,
A3 to A5,
L1 to L4,
L7 and L8.

## Sources read

OpenZFS `zfs-2.4.4`,
commit `71a9f95`,
cloned at
`/home/user/temp/agent/zfs-deferred-destroy-checks-design-2026-11-22/scratch/openzfs-zfs-2.4.4`:
`include/sys/fs/zfs.h`,
`include/sys/dsl_dataset.h`,
`module/zcommon/zfs_prop.c`,
`module/zfs/dsl_dataset.c`,
`module/zfs/dsl_destroy.c`,
`module/zfs/dsl_prop.c`,
`module/zfs/dsl_userhold.c`,
`module/zfs/dmu_objset.c`,
`module/zfs/zfs_ioctl.c`,
`lib/libzfs/libzfs_dataset.c`,
`lib/libzfs_core/libzfs_core.c`,
`cmd/zfs/zfs_main.c`,
`man/man7/zfsprops.7`,
`man/man8/zfs-destroy.8`.

This repository at `5dd8883a60`:
the decision,
proposal,
implementation plan and decision records named in the opening,
`package/git-policy/cli/SPEC.md`,
the crate under `package/zfs/request-to-openzfs-kernel-device`,
and the native code named in "What this design reuses".

The seven sibling designs read with `git show <branch>:<doc>`:
the destruction plan,
the cancelled or interrupted removal,
the add binding record,
the origin-cleanup record,
the forced arm,
the private propagation after a replacement mount,
and the lost last event.

No Git and no Linux source was read:
nothing here claims their behaviour.

## Revisions

- `124ea5b573`,
  committed `2026-11-22T17:27:58-05:00`:
  the first version,
  which designed the open item for the first time.
- The commit that answers the gate review,
  whose subject begins "docs(planning):
  answer the deferred-destroy checks gate review",
  and whose committer time is what `git log --format=%cI` prints for it.
  It makes the changes "What this revision rewrote" lists.
- The commit that fills this section in,
  a metadata-only change whose subject begins "docs(planning):
  record the revisions",
  whose committer time is what `git log --format=%cI` prints for it.

[vm-12]: cli-git-zfs-measurements-owed-before-acceptance.md
[vm-17]: cli-git-zfs-measurements-owed-before-acceptance.md
