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

This revision answers two reviews,
both outside this repository:
the gate review of `124ea5b573`
(`~/temp/agent/gate-review-zfs-deferred-destroy-checks-design-2026-11-22/keep/review.md`)
and the second pass over `3e34817184`
(`~/temp/agent/gate-review-zfs-deferred-destroy-checks-design-2026-11-22/keep/review-second-pass.md`).
"Revisions" lists what each commit changed,
and "What this revision rewrote" says which sections each review's findings turned over
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
  question Q1 and question QA-1 put the choice to the project.
- The man page says a snapshot marked for deferred destroy "remains visible,
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
- The way a control runs the built wrapper as `git`:
  a hard link of the executable at `<root>/wrapper/monochromatic-git-policy-cli` and a `<root>/bin/git`
  symbolic link to it,
  on a fixture `PATH` with no real home or global Git configuration
  (`package/git-policy/cli/src/native/binary/binary_support_fixture_directories.rs:130` to `:131`,
  `:210` to `:213`;
  `.../binary_support.rs:1` to `:8`).
  "Measurements owed" runs the wrapper this way in the guest.
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
implementation plan `:2065` and `:2097`,
which carry the phrase,
and `:2112` to `:2113`,
which says a check the report cannot read or prove is reported "as unknown").

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
the wrapper's package,
`package/git-policy/cli/Cargo.toml`,
has the `measurement-routes` feature off by default,
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
the reads and the record rule
("What each build step builds").

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
Neither of the two programs on this host that prune snapshots passes a deferred destroy
(proposal `:1422` to `:1424`),
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
what the destruction plan records as already destroyed,
and every way forward,
in the idiom "The refusals" of the descriptor mount records
(`package/git-policy/cli/SPEC.md:5834` to `:5840`;
`sweep_clone_check.rs:206` to `:280`).

One template,
with the fields a control compares,
covers every cause:

```text
refusing to remove the ZFS worktree clone `<clone name>`: <the cause, with the object and the field>.
<the history sentence>. Everything the history sentence and the cause do not name as gone is still there; the Git
registration stands.
The removal is pending; `cancel-worktree-removal <slot>` is the way back and reports any snapshot already
destroyed. Ways forward: <the reads and the acts a person has>.
```

The history sentence comes from the destruction plan's recorded outcomes,
never from which call site reached the check,
so it is true on a resumed removal and at a later chunk's check:
- a plan that records no destroyed tuple says
  `This removal has not destroyed anything yet.`
- a plan that records destroyed tuples names every one and says
  `This removal has already destroyed <the names, comma separated> (in this run or an earlier one), and they do not
  come back.`

The same template covers both check points.
What differs between a first refusal and a later one is only what the plan records,
so no sentence claims "this run" did or did not delete something.
`cancel-worktree-removal` is step 17's command (implementation plan `:1958`),
so a control compares the name as a fixed string,
and a refusal may name a command the build does not yet carry.

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
- `OriginDatasetIsNotAFilesystem`:
  the object at the origin dataset's name is not a filesystem,
  so the plan's dataset identity cannot be compared.
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

`CloneGone`,
`OriginDatasetGone` and `OriginSnapshotGone` are the three causes whose verdict needs a word,
because a removal can meet one by a substitution after step 1's proof and before the check,
which is exactly the window a read cannot close (L4).
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
snapshot `monochromatic-git-checkout@git-worktree-clone-origin-7f3a9c2e` is marked for deferred destroy
(`defer_destroy` reads 1), and the kernel reports 1 clone of it and 0 user holds, so destroying this clone destroys
that snapshot in the same transaction. This removal has not destroyed anything yet. Everything the history sentence
and the cause do not name as gone is still there; the Git registration stands. The removal is pending;
`cancel-worktree-removal fix-flaky-test` is the way back and reports any snapshot already destroyed. Ways forward:
`zfs get -H -o value defer_destroy monochromatic-git-checkout@git-worktree-clone-origin-7f3a9c2e` and `zfs holds
monochromatic-git-checkout@git-worktree-clone-origin-7f3a9c2e` show the mark and the holds. The mark cannot be
cleared in place, so this slot's removal finishes only outside the wrapper: either you accept that snapshot's
destruction and destroy the clone yourself, or you clone that snapshot elsewhere first, which pins it, and then
destroy the clone. Either way the plan beside the registration still names what this removal owed, and once the
head is gone it is a `head=gone` plan, which that design never refuses for want of a live intent
(doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md,
"the pointer beside the registration" and "The head-gone path"), so its finish pass settles it. If neither is
right for you, leave the slot and ask the administrator.
```

and the same cause with a plan that recorded two destroyed snapshots:

```text
refusing to remove the ZFS worktree clone `monochromatic-git-worktree-clone-datasets/fix-flaky-test`: its origin
snapshot `monochromatic-git-checkout@git-worktree-clone-origin-7f3a9c2e` is marked for deferred destroy
(`defer_destroy` reads 1), and the kernel reports 1 clone of it and 0 user holds, so destroying this clone destroys
that snapshot in the same transaction. This removal has already destroyed `fix-flaky-test@manual-2026-11-21` and
`fix-flaky-test@sanoid-hourly-2026-11-22` (in this run or an earlier one), and they do not come back. Everything
the history sentence and the cause do not name as gone is still there; the Git registration stands. The removal is
pending; `cancel-worktree-removal fix-flaky-test` is the way back and reports any snapshot already destroyed. Ways
forward: as for the refusal above, and the loss named here is the cost already paid.
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
and question QA-1's for the first.

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
  The clone's own intent property went with the clone,
  so the plan's head-gone rule is what completes it:
  "A `head=gone` plan has no live intent by construction",
  and "A plan whose head is gone is never refused for that reason"
  (doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md,
  "The head-gone path" and "The removal intent's rule"),
  while the finish pass reaches the plan through the pointer beside the registration
  (the same design's "The pointer beside the registration" and its section 5).
  The finish pass then settles the registration and the pointer,
  leaves the origin-cleanup record where an origin remains (a sibling's design),
  and unlinks the plan (the same design's "When it is removed").
  That division is the plan sibling's,
  not this design's.
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
- Between the head check and the head request,
  a mark set on the origin is not caught,
  and the head request is the transaction that reaps the marked origin (F10).
- A mark set between the check before the last chunk and the head check is caught by the head check,
  and the price is the snapshots the batch destroyed before it (proposal `:938` to `:940`).
  The head check therefore does not close that window:
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
  and the pause route M3c and M3b-2 drive:
  a file named by `MONOCHROMATIC_ZFS_REMOVAL_PAUSE`,
  live only under the `measurement-routes` feature (F24),
  that writes `<path>.ready` and waits for `<path>.go` at the phase `<path>.at` names
  (`before-first-check` or `before-head-check`),
  bounded by the removal's own deadline,
  and recording a wait that expired rather than honouring it.
  Building it widens that feature's own comment in `package/git-policy/cli/Cargo.toml:71` to `:75`,
  which today names the clone fence alone.
  It runs M3 once the route is whole.
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
  The pause route is a shipped build's dead code:
  the feature is off in every release,
  and nothing but a measurement build turns it on (F24).

The implementation plan puts the owed refusal measurement in step 15's "VM measurements"
(`:1846` to `:1852`).
This design has step 16 run M3,
because the call sites a refusal comes from are step 16's,
so until step 16 lands there is no refusal to measure;
the two steps ship in the plan's order,
and the measurement waits for both.

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
   the origin dataset's name answers as a snapshot,
   and the origin snapshot's name answers as a filesystem,
   each with its own cause (`CloneIsNotAFilesystem`,
   `OriginDatasetIsNotAFilesystem`,
   `OriginIsNotASnapshot`).
9. `an_incomplete_plan_names_the_field_and_deletes_nothing`.
10. `every_cause_prints_the_template_with_its_object_the_field_and_the_ways_forward`:
    one message per cause,
    compared against the template and the fields,
    at either call site.
11. `the_history_sentence_follows_the_plans_outcomes`:
    a plan with no outcome prints "has not destroyed anything yet",
    one with two destroyed tuples names both,
    and the two messages are otherwise identical;
    a resume whose earlier run destroyed the tuples prints the names and never says "this run" (M-A's fix).
12. `a_gone_clone_or_origin_dataset_or_origin_snapshot_refuses_and_names_the_settle`:
    `ENOENT` on any of the three,
    with the plan's tuple named and the settle rule pointed at (V10).
13. `a_marked_origin_with_a_second_clone_still_refuses_and_says_the_origin_is_pinned`:
    `dds_num_clones` is 2,
    the verdict is `Refused`,
    and the message says the origin is pinned by another clone.
14. `a_marked_origin_under_a_user_hold_still_refuses_and_says_the_reap_will_not_happen`:
    `userrefs` is 1 and `dds_num_clones` is 1,
    the verdict is `Refused`,
    and the message says the snapshot stands after this removal.
15. `a_refusal_keeps_the_intent_and_the_plan`:
    neither the intent rewrite nor the plan unlink is called,
    at either call site.
16. `a_refusal_before_a_second_chunk_stops_the_chunks_after_it`:
    the fixture counts one snapshot request,
    the second chunk's check refuses,
    and no second request is recorded.
17. `the_first_check_runs_after_the_reproof_and_before_the_requested_rewrite`:
    the fixture records the call order of one snapshot step,
    the fresh verification and reproof before the check,
    and the plan rewrite announcing the request after it.
18. `the_head_check_runs_after_the_enumeration_and_before_the_requested_rewrite`:
    the fixture records the call order of the head step,
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
the next chunk's check,
or the head check when that was the last chunk,
stops the next delete after the earlier chunk's snapshots have gone (see "The accepted residual").

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

19. `a_marked_snapshot_reports_the_mark_and_an_unmarked_one_does_not`,
    in `package/zfs/request-to-openzfs-kernel-device/src/kernel_device/live_read_tests.rs`:
    the identity read on a snapshot before a deferred destroy reads 0,
    after it reads 1,
    the entry carries no `source`,
    and `dds_num_clones` follows F9's meaning.
    It runs through the package's e2e recipe like the other `#[ignore]` live controls
    (`live_read_tests.rs:120` to `:121`).
20. The removal arms of M3 below,
    in a new guest recipe under `package/git-policy/cli/e2e/`,
    driven the way `mount-git-worktree-clone-datasets-guest.ts` drives the step 8 arms.

## Measurements owed

None of these ran in this round.
Each is written to be run by a later round as it stands,
and each names every account and grant it needs.

Every line below carries the record or source line that says what it does on the state the lines before it left.
That is what proves a probe can run when nobody runs it.
Two of the things they use are code a later step must build and that does not exist now:
the pause route (step 16,
"What each build step builds") and the managed slot's add (step 10,
with the mount
handler of steps 6 to 9).
M0a,
M1 and M2 need neither.
`zfs`'s own flags have no long spelling for the ones used here
(`man/man8/zfs-destroy.8:147` for `-d`,
`man/man8/zfs-get.8:139` and `:149` for `-H` and `-o`),
so rule `LFF` leaves them short.

### M0a: the kernel fixture, for M1 and M2

Run as root in a disposable guest.
It makes the design's own datasets,
one hand-made clone for M1,
and the account's grants.

```bash
# a file-backed pool, the way the step 8 guest recipe makes one
# (package/git-policy/cli/e2e/mount-git-worktree-clone-datasets-guest.ts:293)
mkdir --parents /var/lib/zfs-fixtures/deferred
truncate --size 4G /var/lib/zfs-fixtures/deferred/pool.img
zpool create zfsdeferred /var/lib/zfs-fixtures/deferred/pool.img
# the checkout dataset, mounted at a fixture path (proposal :272 to :273)
zfs create -o mountpoint=/var/lib/zfs-fixtures/deferred/checkout zfsdeferred/monochromatic-git-checkout
# the container with the design's four properties, unmounted
# (mountpoint=legacy, canmount=off, setuid=off and devices=off, proposal :272 to :274)
zfs create -o mountpoint=legacy -o canmount=off -o setuid=off -o devices=off \
    zfsdeferred/monochromatic-git-worktree-clone-datasets
printf 'history\n' > /var/lib/zfs-fixtures/deferred/checkout/history
# an account name no image ships, and no pinned uid: a pinned 1000 collides with ubuntu,
# which is the account the measurements record ran as
# (doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md:765 to :767)
useradd --create-home probe
# the origin snapshot in the add route's own name form, then a clone of it,
# in the order a clone requires (the name's form, proposal :373 to :375;
# a clone is created from a snapshot, man/man8/zfs-clone.8:76 to :77)
nonce=$(od --address-radix=n --format=x1 --read-bytes=4 /dev/urandom | tr --delete ' \n')
zfs snapshot zfsdeferred/monochromatic-git-checkout@git-worktree-clone-origin-$nonce
zfs clone zfsdeferred/monochromatic-git-checkout@git-worktree-clone-origin-$nonce \
    zfsdeferred/monochromatic-git-worktree-clone-datasets/m1-clone
# the clone's own snapshots, one manual and one sanoid-shaped, which are the set a removal inventories
# (proposal :876 to :880)
zfs snapshot zfsdeferred/monochromatic-git-worktree-clone-datasets/m1-clone@manual-2026-11-21
zfs snapshot zfsdeferred/monochromatic-git-worktree-clone-datasets/m1-clone@sanoid-hourly-2026-11-22
# the grants a built add and a removal need, each from the proposal's delegation section:
# snapshot local on the checkout (:337 to :338),
# clone on its descendants (:338),
# the decided checkout-origin grant, mount,destroy on its descendants (:341 to :343),
# create,mount local on the container (:338 to :339),
# and mount,userprop,destroy on its descendants (:339 to :340)
zfs allow -l -u probe snapshot zfsdeferred/monochromatic-git-checkout
zfs allow -d -u probe clone,mount,destroy zfsdeferred/monochromatic-git-checkout
zfs allow -l -u probe create,mount zfsdeferred/monochromatic-git-worktree-clone-datasets
zfs allow -d -u probe mount,userprop,destroy zfsdeferred/monochromatic-git-worktree-clone-datasets
```

The grants are the proposal's set,
not a measurement's,
because the add's preflight reads effective grants before it writes anything (proposal `:365` to `:367`):
a fixture with fewer of them refuses at the add's own step 1 and never reaches a removal.
The earlier version of this section labelled a different set as [VM item 17][vm-17]'s.
That set is the three grants the measurements record prints
(`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md:753` to `:767`,
`:927` to `:938`),
and the checkout-origin grant is the proposal's addition to them (`:341` to `:343`).

The hand-made clone is called `m1-clone`,
which is no slot name a caller can ask the add for,
so M0b's add of `fix-flaky-test` cannot collide with it whatever order the probes run in.

### M0b: the managed slot, for M3

M3 removes a slot the wrapper manages,
so M0b runs the built add at the canonical destination,
as the account,
on M0a's pool.
A hand-made registration is neither the binding record nor the intent a removal reads,
so M3 never makes its slot by hand.

```bash
# the checkout is the add's origin, so it is a Git repository with one commit
# (the add's preflight requires a supported Git form and an eligible source, proposal :365 to :370)
git init --initial-branch=main /var/lib/zfs-fixtures/deferred/checkout
git -C /var/lib/zfs-fixtures/deferred/checkout config user.name probe
git -C /var/lib/zfs-fixtures/deferred/checkout config user.email probe@example.invalid
git -C /var/lib/zfs-fixtures/deferred/checkout add history
git -C /var/lib/zfs-fixtures/deferred/checkout commit --message 'the fixture'
# the built wrapper as `git`: a hard link of the executable and a bin/git symbolic link to it,
# which is how every binary control runs it
# (package/git-policy/cli/src/native/binary/binary_support_fixture_directories.rs:130 to :131, :210 to :213)
install --directory /opt/deferred-removal/wrapper
install --mode 0755 <the built monochromatic-git-policy-cli> \
    /opt/deferred-removal/wrapper/monochromatic-git-policy-cli
install --directory /opt/deferred-removal/bin
ln --symbolic /opt/deferred-removal/wrapper/monochromatic-git-policy-cli /opt/deferred-removal/bin/git
install --directory --mode 0700 /home/probe/.run
# the add as the account, at the canonical destination and nowhere else
# (the clone mounts at the slot, proposal :275 to :277;
# the route accepts only the canonical path, :1375 to :1376), with its own home and runtime directory
sudo --user probe --set-home env HOME=/home/probe XDG_RUNTIME_DIR=/home/probe/.run \
    PATH=/opt/deferred-removal/bin:/usr/bin:/bin \
    git worktree add /var/lib/zfs-fixtures/deferred/checkout/.claude/worktrees/fix-flaky-test
# the origin's name comes from the live clone, never typed
# (the origin property is the snapshot the clone was created from, man/man7/zfsprops.7:252 to :255)
origin=$(zfs get -H -o value origin zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test)
printf 'origin=%s\n' "$origin"
# the handler attached the clone to its slot on a qualifying sweep (proposal :379 to :380);
# the removal's step 2 unmounts it, so assert it is mounted before the arm
findmnt --target /var/lib/zfs-fixtures/deferred/checkout/.claude/worktrees/fix-flaky-test
# the clone's own snapshots, where M0a puts them, on the clone the add made
zfs snapshot zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test@manual-2026-11-21
zfs snapshot zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test@sanoid-hourly-2026-11-22
```

The add's own sequence is the one it ships:
the claim writes the clone's name and the origin snapshot's name before mutation,
the account creates the snapshot and the clone,
and the handler attaches the clone on its next qualifying sweep (proposal `:365` to `:380`).
Until step 10 and the handler of steps 6 to 9 are built,
this block cannot run and M3 waits;
M0a,
M1 and M2 do not.

### M1: the reap, the positive control

Owed before the refusal arms,
because a harness that cannot see the loss cannot show that the refusal avoids it (rule `QPC`).
It needs no built wrapper and runs on M0a.

```bash
# the mark before it is set: the property's value is off (F1)
origin=$(zfs get -H -o value origin zfsdeferred/monochromatic-git-worktree-clone-datasets/m1-clone)
zfs get -H -o value defer_destroy "$origin"
# the mark, made by the account under the checkout's descendant mount,destroy grant
# (the grant, proposal :341 to :343; a snapshot's permission is checked upward from its dataset,
# module/zfs/dsl_deleg.c:574 to :582).
# It marks instead of destroying because the origin still has one clone, so two children
# (module/zfs/dsl_destroy.c:322 to :333); a snapshot with none would fall to the destroy below that branch.
sudo --user probe zfs destroy -d "$origin"
zfs get -H -o value defer_destroy "$origin"
# the removal's own order by hand: the clone's own snapshots, then the head (proposal :894 to :900)
zfs destroy zfsdeferred/monochromatic-git-worktree-clone-datasets/m1-clone@manual-2026-11-21
zfs destroy zfsdeferred/monochromatic-git-worktree-clone-datasets/m1-clone@sanoid-hourly-2026-11-22
# the head: the last clone, so the same transaction reaps the marked origin
# (module/zfs/dsl_destroy.c:1028 to :1031, :1169 to :1173)
zfs destroy zfsdeferred/monochromatic-git-worktree-clone-datasets/m1-clone
# the origin is gone: the listing exits non-zero naming no such dataset
zfs list -H -o name "$origin"
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

Control 19,
in the crate's own pool rather than M0a's,
because the crate's live test makes the pool it needs (`live_read_tests.rs:127`).
It shows that the field the check reads is the field the kernel fills,
for both values,
through the crate rather than through `zfs`.

### M3: the wrapper's refusals

M3 needs M0b's managed slot.
Each arm starts from a fresh M0a and M0b,
because each arm destroys objects that the next one needs.
Each arm states whether it is a control (an assertion that must hold) or a reach report (counts).

#### The pause route

The pause route does not exist in this tree,
and the removal code it belongs to is steps 14 to 16.
Step 16 builds it,
under the `measurement-routes` feature (F24),
in a module of the removal's own,
and widens that feature's comment (`package/git-policy/cli/Cargo.toml:71` to `:75`).
It reads `MONOCHROMATIC_ZFS_REMOVAL_PAUSE` as a file path,
and the phase from that path with `.at` appended,
one line holding `before-first-check` or `before-head-check`.
At the phase it names,
after the fresh verification and reproof and before the check,
it writes `<path>.ready` and waits for `<path>.go`,
bounded by the removal's own deadline;
a wait that runs out is not honoured,
the removal carries on,
and the route records that the pause expired.
Every line below sets the variable after `sudo`,
so nothing depends on what the guest's sudoers keeps
(`sudoers(5)`'s `env_reset` is on by default).

#### M3a: the first check refuses (control)

The origin is marked before the run,
by the account,
on the name the live clone reports.

```bash
# the origin's name, read live (man/man7/zfsprops.7:252 to :255)
origin=$(zfs get -H -o value origin zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test)
# the mark, as in M1: the origin has one clone, so the deferred destroy marks it
# (module/zfs/dsl_destroy.c:322 to :333)
sudo --user probe zfs destroy -d "$origin"
# the removal as the account, through the wrapper as `git`
sudo --user probe --set-home env HOME=/home/probe XDG_RUNTIME_DIR=/home/probe/.run \
    PATH=/opt/deferred-removal/bin:/usr/bin:/bin \
    git worktree remove /var/lib/zfs-fixtures/deferred/checkout/.claude/worktrees/fix-flaky-test
```

The state it runs on,
and the step that refuses:
step 1 proves the lineage and the clone's identity and reads no mark (proposal `:854` to `:857`);
step 2 writes the intent,
prints the names and requests the unmount (proposal `:886` to `:888`);
step 3 writes the plan;
step 4 runs the fresh all-namespace verification and reproofs the identities (proposal `:896` to `:899`);
and the first check then refuses,
before the plan's `requested` rewrite and before any snapshot request.
If the project answers QA-2 yes,
a refusal would come earlier,
in step 1's proof;
this design adds no such refusal,
so this arm expects the first check's.

Recorded:
the message (the template with a plan that recorded nothing),
the exit status,
and after the run `zfs list -H -o name,guid` for the clone and both its snapshots,
`zfs get -H -o value,source defer_destroy` for the origin,
`zfs get -H -o value org.monochromatic:git-worktree-clone-mount-desired-state` for the clone,
the plan's bytes,
and `git worktree list`.
Pass is the first check's refusal,
the clone and both its snapshots present,
the origin and the marked snapshot present,
the intent reading `unmount`,
the plan present with no outcome,
the registration listed,
no snapshot destroyed,
and the history sentence saying this removal has not destroyed anything yet.

#### M3b: a replacement clone whose different origin is marked (control)

Two arms,
because the replacement can be in place before the run or made after step 4's reproof,
and only the second reaches the check's own identity comparison.

M3b-1,
the replacement already in place.
The plan lists this as step 15's acceptance test
(`:1843` to `:1844`,
and the proposal's own words at `:2095` to `:2098`),
and on this state it is step 1's proof that refuses,
not the check:

```bash
# destroy the planned clone outside the wrapper, after its own snapshots
# (the removal's own order, proposal :894 to :900)
zfs destroy zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test@manual-2026-11-21
zfs destroy zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test@sanoid-hourly-2026-11-22
zfs destroy zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test
# another checkout snapshot, a clone of it at the planned name, and the mark on that snapshot,
# which now has its one clone (a clone needs its snapshot first, man/man8/zfs-clone.8:76 to :77;
# the mark's condition, module/zfs/dsl_destroy.c:322 to :333)
zfs snapshot zfsdeferred/monochromatic-git-checkout@other
zfs clone zfsdeferred/monochromatic-git-checkout@other \
    zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test
sudo --user probe zfs destroy -d zfsdeferred/monochromatic-git-checkout@other
# the removal: step 1 compares the live clone's GUID with the binding's and refuses (proposal :854 to :857);
# nothing was unmounted, no intent was rewritten and no plan was written
sudo --user probe --set-home env HOME=/home/probe XDG_RUNTIME_DIR=/home/probe/.run \
    PATH=/opt/deferred-removal/bin:/usr/bin:/bin \
    git worktree remove /var/lib/zfs-fixtures/deferred/checkout/.claude/worktrees/fix-flaky-test
```

Pass:
the refusal names the plan's or binding's clone GUID and the live one,
nothing was destroyed,
the replacement clone and `@other` stand,
the registration is unchanged,
and no plan exists.
This shows the plan's sentence about a replacement being refused is met by step 1,
and that the check's identity reads are defence in depth for the window after step 1.

M3b-2,
the replacement made at the pause before the first check,
so the check's own comparison decides:

```bash
# the replacement's source: another checkout snapshot, held and then marked,
# because a deferred destroy marks only a held or cloned snapshot
# (module/zfs/dsl_destroy.c:322 to :333; the hold is root's,
# and the mark is the account's under the checkout's descendant destroy grant, proposal :341 to :343)
zfs snapshot zfsdeferred/monochromatic-git-checkout@other
zfs hold deferred-replacement zfsdeferred/monochromatic-git-checkout@other
sudo --user probe zfs destroy -d zfsdeferred/monochromatic-git-checkout@other
zfs get -H -o value defer_destroy zfsdeferred/monochromatic-git-checkout@other
# the pause at the first check: the phase file names the point, and the variable is set after sudo
mkdir --parents /var/lib/zfs-fixtures/deferred
printf 'before-first-check\n' > /var/lib/zfs-fixtures/deferred/pause.at
sudo --user probe --set-home env HOME=/home/probe XDG_RUNTIME_DIR=/home/probe/.run \
    MONOCHROMATIC_ZFS_REMOVAL_PAUSE=/var/lib/zfs-fixtures/deferred/pause \
    PATH=/opt/deferred-removal/bin:/usr/bin:/bin \
    git worktree remove /var/lib/zfs-fixtures/deferred/checkout/.claude/worktrees/fix-flaky-test &
removal=$!
# a bounded wait for the pause: 120 s, after which the arm prints what it saw and fails
if ! timeout 120 sh -c 'until [ -e /var/lib/zfs-fixtures/deferred/pause.ready ]; do sleep 1; done'; then
    printf '%s\n' 'FAIL: the pause ready file did not appear within 120 s;
        the removal was not paused, and the plan and the log follow' >&2
    exit 1
fi
# at the pause the clone is unmounted, the intent stands and the plan is written,
# so the planned clone can be replaced by one from a marked snapshot
zfs destroy zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test@manual-2026-11-21
zfs destroy zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test@sanoid-hourly-2026-11-22
zfs destroy zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test
zfs clone zfsdeferred/monochromatic-git-checkout@other \
    zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test
# let it go on, and wait for it with a deadline of its own
touch /var/lib/zfs-fixtures/deferred/pause.go
timeout 300 tail --pid="$removal" --follow /dev/null
wait "$removal"
printf 'removal exit=%s\n' "$?"
```

Pass:
the check refuses with `CloneIsNotThePlannedOne`,
naming the plan's GUID and the live one,
before the plan's `requested` rewrite and before any snapshot request;
the replacement and `@other` stand;
the plan still names the destroyed snapshots this arm took outside the wrapper,
and the removal destroyed nothing else.

#### M3c: the check before the head destroy refuses (control)

The pause moves to the head check,
and the mark is made at the pause,
so the batch has already run and the clone's own snapshots are gone.

```bash
# the origin's name, read live before the run (man/man7/zfsprops.7:252 to :255)
origin=$(zfs get -H -o value origin zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test)
printf 'before-head-check\n' > /var/lib/zfs-fixtures/deferred/pause.at
sudo --user probe --set-home env HOME=/home/probe XDG_RUNTIME_DIR=/home/probe/.run \
    MONOCHROMATIC_ZFS_REMOVAL_PAUSE=/var/lib/zfs-fixtures/deferred/pause \
    PATH=/opt/deferred-removal/bin:/usr/bin:/bin \
    git worktree remove /var/lib/zfs-fixtures/deferred/checkout/.claude/worktrees/fix-flaky-test &
removal=$!
if ! timeout 120 sh -c 'until [ -e /var/lib/zfs-fixtures/deferred/pause.ready ]; do sleep 1; done'; then
    printf '%s\n' 'FAIL: the pause ready file did not appear within 120 s;
        the removal was not paused, and the plan and the log follow' >&2
    exit 1
fi
# the batch ran: the clone's own snapshots are gone and the plan holds their outcomes
zfs list -H -o name -t snapshot -r zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test
# the mark, now that the head is the origin's last child (module/zfs/dsl_destroy.c:322 to :333)
sudo --user probe zfs destroy -d "$origin"
touch /var/lib/zfs-fixtures/deferred/pause.go
timeout 300 tail --pid="$removal" --follow /dev/null
wait "$removal"
printf 'removal exit=%s\n' "$?"
```

The state it runs on,
and the step that refuses:
the check before the snapshot request passed (nothing was marked yet),
the batch destroyed the clone's own snapshots and the plan recorded each outcome,
the fresh enumeration ran (proposal `:926`),
and the head check refuses before the plan's `head=requested` rewrite and before the head request.

Recorded:
the message (the history sentence naming the two destroyed snapshots),
the exit status,
the origin and the marked snapshot present,
the clone head present with its own snapshots gone,
the plan present with the batch's outcomes,
and no head request issued (the clone still exists).
`doctor`'s lines are not a pass condition here:
they are step 18's (implementation plan `:2116` to `:2119`).
For the proposal's "otherwise" clause (`:2095` to `:2098`),
the same pause with a replacement instead of a mark refuses with `CloneIsNotThePlannedOne` at the head check.

#### M3d: the residual (reach report, not an owed arm)

The proposal calls a same-account mark set between the head check and the destroy the accepted residual,
"not a case this item has to close" (proposal `:1969` to `:1970`).
This design records it as reach only.
The watcher marks the origin the moment the clone's snapshot list empties,
and gives up with a printed line if that has not happened within its own deadline:

```bash
origin=$(zfs get -H -o value origin zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test)
# the watcher runs as the account, and a wait that runs out prints what it last saw and fails
sudo --user probe timeout 300 sh -c '
    while zfs list -H -o name -t snapshot \
        -r zfsdeferred/monochromatic-git-worktree-clone-datasets/fix-flaky-test 2>/dev/null \
        | grep --quiet .; do
        sleep 1
    done
    zfs destroy -d "$1" || { printf "%s\n" "FAIL: the mark could not be set after the batch" >&2; exit 1; }
' watcher "$origin" &
watcher=$!
sudo --user probe --set-home env HOME=/home/probe XDG_RUNTIME_DIR=/home/probe/.run \
    PATH=/opt/deferred-removal/bin:/usr/bin:/bin \
    git worktree remove /var/lib/zfs-fixtures/deferred/checkout/.claude/worktrees/fix-flaky-test
timeout 300 tail --pid="$watcher" --follow /dev/null
wait "$watcher"
printf 'watcher exit=%s\n' "$?"
```

Repeat it in one interleaved round with an unchanged-input control,
and report how many runs had the head check see the mark and how many ended with the origin reaped.
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
  the guarantee "before this run deletes anything" holds,
  because the only delete left in that run is the head destroy,
  and a resume with an empty set gets a second refusal point without needing the measurement route.
  Con:
  a reader who takes "before the first snapshot delete" as "always at a first call site" sees one check fewer
  on that resume,
  and the decided sentence then reads as met by the head check.
- Alternative:
  run the check before the snapshot request even when none is sent.
  Example:
  the same resume refuses at a check that guards no request,
  and the head check never sees the mark on that route.
  Pro:
  two checks on every removal,
  whatever the plan says.
  Con:
  a check that guards no request proves nothing about a delete,
  and it stops a run that would otherwise reach the head check.
Ranking:
  adopted over the alternative,
because the skip costs nothing (the check that is skipped guards no request) and it leaves the second refusal
point reachable without the measurement route.
It does not make the head-check refusal measurable:
M3c reaches it by the pause route whichever way this entry is decided.

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
  one outcome for both call sites.
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
  an administrator renames the origin dataset and back,
  or a promotion moves the snapshot and moves it back,
  and the origin snapshot's name and GUID are what the first and third reads compare;
  the dataset read is the one that would catch a future lineage change neither name reveals.
  Pro:
  the origin is proven at both levels the plan recorded,
  and the two reads either side of it already refuse every substitution this release can make.
  Con:
  one more ioctl per check,
  and one more read to keep in the ordered comparison.
  It is defence in depth:
  no arm of any probe here reaches it,
  because a dataset with a clone cannot be destroyed (F7's `EEXIST` for the snapshot,
  and the dataset itself cannot go while the clone exists),
  and a promotion or a rename changes the clone's reported origin,
  which the first read refuses,
  because the kernel names the origin from the object's current name
  (OpenZFS `zfs-2.4.4`,
  `module/zfs/dsl_dir.c:1066` to `:1076`).
- Alternative:
  two reads,
  the origin snapshot's name and GUID standing for the dataset.
  Example:
  the same rename or promotion,
  refused by the first read on the reported origin.
  Pro:
  two ioctls instead of three.
  Con:
  the plan's `originDatasetGuid` is never compared,
  and the decided sentence's "origin" narrows to a snapshot.
Ranking:
  adopted over the alternative,
because the field is cheap to compare and the plan records it,
  and the read is stated as defence in depth rather than as one a probe reaches.
The sibling plan design already assumes the check compares it.

V10.
What the check does when a planned object is gone.
- Adopted:
  refuse,
  naming the plan's tuple and the settle rules the next attempt uses (proposal `:963` to `:967`).
  Example:
  a same-account process destroys the planned clone after step 1's proof and before the check,
  which refuses with `CloneGone` and leaves the settle to the plan's finish pass;
  a resume that finds the head gone never reaches the check,
  because the settle sends it to the absence rule (proposal `:963` to `:967`).
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

### QA-1. When the first check refuses, the clone is already unmounted. Keep that, or cancel and remount?

At that moment the clone is unmounted (possibly staged under `~/.cache`),
the removal intent stands,
the destruction plan is written,
and nothing is destroyed.
This design adopts Option A;
the records lean to it (proposal `:2103` to `:2104`,
`:1241` to `:1245`) and both siblings that reach this state assume it
(the cancelled design's A9,
and the plan sibling's kept plan and pointer),
but the open item does not decide it.

- Option A,
  keep the intent and the plan;
  the clone stays unmounted until a person acts.
  Example:
  `git worktree remove` of `fix-flaky-test` with
  `monochromatic-git-checkout@git-worktree-clone-origin-7f3a9c2e` marked prints that the origin snapshot is
  marked,
  that this removal has not destroyed anything yet,
  that the removal is pending,
  and that `cancel-worktree-removal fix-flaky-test` is the way back;
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

Ranking:
A > B,
because A is what the records lean to and the siblings assume,
and adds nothing to build,
while B spends a decided exception on a case the records do not name and gains only convenience for a refusal
that will recur.

### QA-2. Should a marked origin also be refused inside the lineage proof, before the unmount?

Reading the mark there needs no extra ioctl if step 14 shares the reply step 1 already makes for the origin
snapshot (proposal `:854` to `:857`);
step 14 is unbuilt,
so that is an inference about how it will read the origin,
not a fact.
The decided sentence names two checks,
and this design adds none,
so it adopts Option no until the project answers.

- Option yes:
  a refusal in step 1's proof,
  before the intent and the unmount.
  Example:
  the mark is already on when the person runs the removal,
  and the command refuses with the clone still mounted at its slot,
  no intent written and no plan;
  a mark set later is still caught by the decided first check,
  which then needs QA-1's answer.
  Pro:
  the usual case changes nothing on the machine,
  and the read is one step 1 already makes.
  Con:
  a third refusal point and a third message state,
  where the decided text names two checks.
- Option no,
  adopted for now:
  exactly the two decided checks.
  Example:
  the same command unmounts the clone first and then refuses (QA-1's state).
  Pro:
  exactly what the project decided,
  and no refusal point this design invents.
  Con:
  every marked-origin removal pays an unmount and a pending removal before it is refused.

Ranking:
yes > no on cost,
because the field is in a reply step 1 already makes and the common case would disturb nothing;
the design keeps no until the project answers,
because a further refusal point is the project's to authorize.
QA-2 yes is reachable with QA-1 either way,
so the two questions are independent.

### Q1. After a refusal before the head destroy, does the removal keep its records or cancel itself?

This is the open item's own question (proposal `:937` to `:942`),
and the records disagree with the item's "undesigned" wording:
the owed measurement expects the record to survive (proposal `:1966` to `:1968`).
This design adopts Option A,
for the same reason as QA-1.

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
  the destruction-plan design removes the plan at the end of a completed removal,
  and the cancelled design says its own list of end states "has no cancelled value,
  and this design needs none" (its "Where a sibling and this design disagree",
  the deferred-destroy entry),
  so this needs a plan state and a removal rule neither sibling has.

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
Option B would amend that decided refusal;
a printed administrator command leaves the refusal in place,
which is V7's subject (whether a refusal may print a destructive command),
so it is not a second option here.
This design adopts A,
and the question is put only if the owner wants a guarded finish.

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
Ranking:
A > B,
because B amends the decided refusal and adds an authority the delegated grant was chosen not to carry,
while A keeps the refusal and the manual completion the message already names.
The printed-command option is not ranked here:
it is V7's,
and V7 ranks the refusal's reads over a printed destructive command.

## Dependencies on sibling designs

Each of these is an assumption this design made so it could be whole;
a later pass reconciles the two.
A sibling's document is unreviewed,
so its words are quoted and never adopted as decided.
Each was read with `git show <branch>:<doc>` at the tip named here,
and a tip can move after this revision:
the quoted text is the text at that commit.

- "Where a removal's persisted destruction plan lives,
  and its format"
  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
  read at `76849b8d9b`):
  it places every field A1 needs,
  including `originDatasetGuid`,
  and says those are "the identities the deferred-destroy check compares with live";
  that agrees with this design's third read (V9).
  It writes the plan "after the unmount and not before it" and removes it only after a completed removal's settle;
  that agrees with A2 and V4.
  It writes `requested` before each request and `head=requested` before the head destroy,
  which is why the checks run before those rewrites.
  Its head-gone path has "no lineage proof ... and no `unmount` intent ... because the intent is a property of the
  clone and went with the head",
  and it says "A `head=gone` plan has no live intent by construction" and is never refused for that reason,
  while a plan with no live intent whose head still stands is refused.
  Its finish pass reaches such a plan through the pointer beside the registration,
  which is what "How a person finishes" relies on after an out-of-wrapper destroy.
- "What a cancelled,
  timed-out or killed removal leaves,
  and what the cancel command proves and does"
  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
  read at `42f01440f3`):
  its A9 says "The deferred-destroy design keeps the intent and the plan on every refusal,
  and rewrites neither",
  and its points 12 and 15 are the two refusal states with that assumption.
  That agrees with V4,
  QA-1's Option A and Q1's Option A.
  Its own end-state list "has no cancelled value,
  and this design needs none",
  which is why Q1's Option C would need a plan state and a removal rule that sibling does not have.
- "Where the add's durable binding record lives,
  and its format"
  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
  read at `428040d45d`):
  at this tip it says nothing about the checks.
  The earlier version this design quoted
  ("reads the origin identity this design records,
  and gives it no new field")
  is no longer in its text,
  so the fallback A1 names rests on the binding's own fields as the proposal lists them (proposal `:414` to `:421`),
  not on a sibling statement.
- "The record that carries origin cleanup after a registration is settled"
  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
  read at `987a3ba886`):
  it assumes "The removal's step 15 check of the origin's deferred-destroy mark is available to this design's
  attempt as a read".
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
  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
  read at `095b807355`):
  it says "The deferred-destroy checks own step 16's call sites ...
  this design claims no ownership of those checks and only needs the scan file's reader",
  and its D3 says an earlier version of it assumed "a refusal rewrites the intent to `mount`"
  and that "this revision removes that assumption and defers the state to D2".
  Both agree with this design:
  step 16 owns the call sites,
  this design owns the checks,
  and no refused check rewrites an intent.
  Its own D2 reversal is its caller-session refusal,
  not one of these checks;
  whether that reversal is the decided remount exception is not settled by any record read here
  (proposal `:1492` to `:1498` names "a removal that cancels itself without `--force`"),
  so it belongs to that sibling and the cancelled-removal design,
  and nothing here adopts it.
- "The dirty-main restore on rung 1 and its frozen proof"
  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
  read at `c342dd1a06`):
  its fallback now records a dependency instead of an assumption:
  "no sibling design gives an add a callable unmount-and-destroy;
  ... the four removal designs are written for removals",
  and "the settlement is marked as depending on a callable unmount-and-destroy no sibling offers".
  Its list names this design at `3e34817184`.
  This design's part of that answer is interface,
  not permission:
  `check_planned_origin` takes the plan's tuple,
  the three reads and the stage,
  so any caller that holds a plan gets the same check.
  Whether an add may call the removal's unmount and destruction is the removal's and the add's design to settle,
  which this design neither grants nor withholds.
  What it does say is that its check gives an add's fresh clone no exception:
  a same-account mark on the origin snapshot an add made is the accepted residual,
  and the check refuses it as it refuses any other.
- "The add-time Git lock of a ZFS clone worktree"
  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
  read at `a7902e293b`):
  of the destruction steps it assumes "A removal's destruction proof can say "the clone is gone" before the Git
  settle runs,
  as [plan] step 16 states it" (its A3;
  the inner words are the sibling's own quoting),
  and it places the lock's release in plan step 16's settle.
  At this tip it says nothing about the checks;
  the earlier "No interaction" paragraph this design first read is gone from its text.
  A refused check never reaches that settle,
  so the lock it protects stands,
  which follows from the sibling's own release rule.
- "Which triggers a lost last event waits for,
  and how nudges are merged"
  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
  read at `80a874b773`):
  it lists the deferred-destroy checks as "no mention",
  and this design depends on none of it.
- "What re-applies private propagation after a replacement mount of the checkout"
  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
  read at `5e6a2ff9dc`):
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

Rewritten after the second pass,
and nothing else:
"The refusals" (one template,
the history sentence chosen by the plan's recorded outcomes rather than the call site,
the cancel sentence,
and the nonce-form snapshot name in both messages;
the missing cause `OriginDatasetIsNotAFilesystem` added),
"How a person finishes" (which rule completes a removal whose clone was destroyed outside the wrapper),
"What each build step builds" (the pause route,
the feature comment it widens,
and why M3 runs at step 16),
the controls (the history-sentence control,
the origin dataset in the gone-object and kind controls,
the first call site's order control,
and the renumbering),
"Measurements owed" (every line traced,
the fixture made where the route makes one with the proposal's grants,
the origin's name read live,
the mark made on a state that marks,
each arm's refusing step named,
the pause route described as unbuilt,
every wait bounded with what it prints when the bound passes),
V1 (its reason restated,
without the reason M3c contradicted),
V9 and V10 (their examples,
and V9 stated as defence in depth),
QA split into QA-1 and QA-2,
Q2 (Option C dropped to V7,
its framing corrected),
"Dependencies on sibling designs" (every sibling re-read at its current tip,
including the forced arm's current text,
the plan record's head-gone rule,
the dirty-main design's dependency on a callable unmount-and-destroy,
the add-time lock's statement and the answer to the dirty-main design,
and the binding's silence at its tip),
"Sources read" and "Revisions".

Two statements the second pass found sound changed only where one of these fixes forced it:
Q1's Option C cost now quotes the cancelled design's own "no cancelled value",
and the residual's two bullets name the head check where chunks make "the second check" ambiguous.

## Sources read

OpenZFS `zfs-2.4.4`,
commit `71a9f95`,
cloned at
`/home/user/temp/agent/zfs-deferred-destroy-checks-design-2026-11-22/scratch/openzfs-zfs-2.4.4`:
`include/sys/fs/zfs.h`,
`include/sys/dsl_dataset.h`,
`module/zcommon/zfs_prop.c`,
`module/zfs/dsl_dataset.c`,
`module/zfs/dsl_deleg.c`,
`module/zfs/dsl_destroy.c`,
`module/zfs/dsl_dir.c`,
`module/zfs/dsl_prop.c`,
`module/zfs/dsl_userhold.c`,
`module/zfs/dmu_objset.c`,
`module/zfs/zfs_ioctl.c`,
`lib/libzfs/libzfs_dataset.c`,
`lib/libzfs_core/libzfs_core.c`,
`cmd/zfs/zfs_main.c`,
`man/man7/zfsprops.7`,
`man/man8/zfs-clone.8`,
`man/man8/zfs-destroy.8`,
`man/man8/zfs-get.8`.

This repository at `3e34817184`
(its first version was read at `5dd8883a60`,
and the code files cited here did not change between the two):
the decision,
proposal,
implementation plan and decision records named in the opening,
`package/git-policy/cli/SPEC.md`,
`package/git-policy/cli/Cargo.toml`,
the guest recipe `package/git-policy/cli/e2e/mount-git-worktree-clone-datasets-guest.ts`,
the binary controls' fixture support
(`package/git-policy/cli/src/native/binary/binary_support.rs`,
`.../binary_support_fixture_directories.rs`),
the crate under `package/zfs/request-to-openzfs-kernel-device`,
the measurements record `doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
and the native code named in "What this design reuses".

The nine sibling designs read with `git show <branch>:<doc>`,
each at the tip "Dependencies on sibling designs" names:
the destruction plan,
the cancelled or interrupted removal,
the add binding record,
the origin-cleanup record,
the forced arm,
the dirty-main restore on rung 1,
the add-time Git lock,
the private propagation after a replacement mount,
and the lost last event.

No Git and no Linux source was read:
nothing here claims their behaviour.

## Revisions

- `124ea5b573`,
  committed `2026-11-22T17:27:58-05:00`:
  the first version,
  which designed the open item for the first time.
- `6ea002aa7d`,
  committed `2026-11-22T18:11:08-05:00`:
  the version that answers the gate review,
  with the changes "What this revision rewrote" lists.
- `3e34817184`,
  committed `2026-11-22T18:11:19-05:00`:
  the metadata-only commit that recorded the two above.
- `8228a9efdb`,
  committed `2026-11-22T18:39:07-05:00`:
  the version that answers the second pass,
  with the changes the second part of "What this revision rewrote" lists.
- The commit that re-read the three siblings whose tips moved while this revision was written,
  subject beginning "docs(planning):
  follow the sibling tips that moved",
  which corrects the plan record's head-gone rule,
  the dirty-main design's dependency,
  and the origin-cleanup read.
- The metadata-only commit after it,
  subject beginning "docs(planning):
  record the second pass's revisions",
  whose only change is the times this list is missing,
  read from `git log --format=%cI` before it is made.
  Its own committer time can only be read from `git log --format=%cI` for it,
  because a document cannot name the time of the commit that writes it.

[vm-12]: cli-git-zfs-measurements-owed-before-acceptance.md
[vm-17]: cli-git-zfs-measurements-owed-before-acceptance.md
