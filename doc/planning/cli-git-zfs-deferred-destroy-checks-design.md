# The deferred-destroy checks of a removal

Design by an agent,
accepted by the project as written on 2026-11-22
(`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`,
"The design is accepted",
which lists the ten choices accepted with it).
Nothing in it has been run:
every probe and arm is a specification the build step writes and runs,
and its statements about designs that are still unapproved bind nothing until those are accepted.
It designs the open item "How the deferred-destroy checks are designed" of the accepted ZFS worktree design
(`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
"Open items the decisions leave"),
the item the implementation plan lists as "step 15,
blocking,
writer design work"
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
"Open items and the step that needs each").
It makes no decision that the accepted design,
the proposal,
the project's answers of 2026-11-19 or the three answers of 2026-11-22 did not make,
beyond the choices "Choices this design marks for the project's veto" lists.
The three answers are the project's,
recorded in
`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`,
and "What the project decided" says what each one changes here.

This revision answers six reviews,
all outside this repository:
the gate review of `124ea5b573`
(`~/temp/agent/gate-review-zfs-deferred-destroy-checks-design-2026-11-22/keep/review.md`),
the second pass over `3e34817184`
(`~/temp/agent/gate-review-zfs-deferred-destroy-checks-design-2026-11-22/keep/review-second-pass.md`),
the diff check of `f82f20ecab`
(`~/temp/agent/gate-review-zfs-deferred-destroy-checks-design-2026-11-22/keep/review-diff-check.md`),
the review after the project's answers of `1f7a0800fa`
(`~/temp/agent/gate-review-zfs-deferred-destroy-checks-design-2026-11-22/keep/review-after-the-owners-answers.md`),
the check before acceptance of `946578a8e5`
(`~/temp/agent/gate-review-zfs-deferred-destroy-checks-design-2026-11-22/keep/review-check-before-acceptance.md`)
and the last diff read of `d0ff7604b7`
(`~/temp/agent/gate-review-zfs-deferred-destroy-checks-design-2026-11-22/keep/review-last-diff-read.md`).
"Revisions" lists what each commit changed,
and "What this revision rewrote" says which sections each review's findings turned over
and which stand as the first version wrote them.

It read,
whole or in the passages named:
`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`,
`doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
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
Every measurement it owes stays owed.
Of "Measurements owed",
three probes can run as written once a guest exists,
and the rest are specifications of the state,
the action and the expectation,
whose scripts the build step that builds the route they need writes and first runs in a guest.

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
  The project answered it on 2026-11-22:
  a refusal at the first check and a refusal at the second check each keep the removal pending,
  leave the clone unmounted and make the cancel command the way back
  (`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`,
  "What a refusal at the first check leaves" and "What a refusal at the second check leaves"),
  which is the reading the owed measurement's text already had.
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
  and "What the record does not decide" is an authority note rather than an attack.
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

### The order the three refusal points sit in

A removal of a managed clone does this,
in the accepted order (proposal `:886` to `:900`):

1. Prove the lineage and the candidate identities (proposal step 1),
   and,
   on a first run only,
   that is,
   when the clone carries no removal intent and no plan exists for it,
   look for the mark on the origin snapshot the proof resolved,
   before the intent is written
   (`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`,
   "A look before the unmount").
2. Record the removal intent,
   print the slot,
   the dataset and the exact snapshot names to be lost,
   release local handles and request `unmount` (proposal `:886` to `:888`).
3. After the verified unmount,
   persist the destruction plan (proposal `:894` to `:896`).
4. Require fresh root all-namespace verification and reprove lineage and candidate identities
   immediately before each snapshot request (proposal `:896` to `:899`),
   and that reproof does not look:
   a mark there is refused by the first check or,
   with no snapshot left to send,
   the head check,
   with the plan's history in its message.
5. Send the snapshot request,
   or skip it for an empty set (proposal `:908`),
   and record its per-name outcomes in the plan.
6. Enumerate the clone's snapshots again,
   then send the head destroy (proposal `:926`).

The three refusal points therefore sit in three states.
The look runs once per removal,
in step 1's proof of a first run,
when the clone carries no removal intent and no plan exists for it:
a removal intent is the clone's desired-state property reading `unmount`,
which a clone does not carry when the property reads `mount` or is absent,
so a first run's clone carries none,
and a plan is the destruction plan step 3 writes.
The accepted order proves lineage again in three later places,
the reproof of step 4 (proposal `:897` to `:898`),
"before every chunk" (proposal `:918` to `:919`)
and a resume (proposal `:963`,
with the amended-plan rule rerunning "all lineage and unmount checks",
`:922` to `:923`),
and none of those proofs looks,
because in each of them the intent and the plan exist and the plan may already name destroyed snapshots.
Both checks run with the clone already unmounted,
the intent already written and the plan already persisted,
and they are what refuses a mark in every later proof,
with the plan's history in the message.
The first check runs after step 4's verification and reproof and before the plan rewrite that announces a request;
the second after step 6's enumeration and before the plan rewrite that announces the head request
(`head=requested` in the sibling destruction-plan design,
"For each snapshot request" and "Before the head destroy").
A check that ran before a `requested` rewrite is what keeps a refusal from leaving a plan that claims a request
this removal never sent.

Nothing here moves the unmount or the plan,
and this design does not change the accepted order.

### The look before the unmount

The project answered yes to looking before the unmount:
a marked origin is refused there "with the worktree still mounted and nothing written",
and the first decided check's state then arises only when the mark appears between that look and the check
(`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`,
"A look before the unmount").

Where it runs,
and where it does not:
it runs once per removal,
inside step 1's lineage proof of a first run,
immediately after the proof resolves the clone's live `origin` and compares that snapshot's name,
GUID and `createtxg` with the binding record's,
and before the proof returns,
so before step 2 writes the intent and before the unmount is requested
(proposal `:854` to `:857`,
`:886` to `:888`).
It runs only when the clone carries no removal intent and no plan exists for it.
A removal intent is the clone's desired-state property reading `unmount`;
a clone whose property reads `mount` or is absent carries none,
and a cancel returns it to `mount`;
a plan is the destruction plan step 3 writes.
Every later proof leaves a mark to the two decided checks:
the reproof of step 4 (proposal `:897` to `:898`),
the proof before each chunk (`:918` to `:919`)
and a resume (`:963`,
`:922` to `:923`).
The reason is the sentences the look's text makes:
in each later proof the clone is unmounted,
the intent stands and the plan exists,
so a look there would print a state sentence that is false,
a cause name that is false,
and a history sentence that hides the snapshots the plan says were destroyed,
which is the loss the project chose to keep visible at the second check.
So on a rerun over a standing mark the look does not run,
and the first check refuses with the plan's history in its message,
or,
with no snapshot left to send,
the head check does.
A mark that appears after the first run's look is therefore refused by the first check on that run,
by the next chunk's check,
by the head check,
or by the first check or,
with no snapshot left to send,
the head check of the next run,
according to when it appears.
A retry after a cancel looks only when no plan stands after the cancellation:
a holder at the removal's step 3 leaves no plan at all,
and a cancel that destroyed nothing and completed removes the all-`planned` plan it left,
because a plan that witnesses nothing is removed by the cancellation
(plan record,
read at `3cd520b6a0`,
"A cancellation,
and the plan it leaves");
so that retry is a first run and the look runs with no change.
A cancellation that kept a plan,
because it destroyed something or was killed before it removed an all-`planned` plan,
gives a retry that does not look,
and "How a person finishes" says what such a retry does.
An identity the proof refuses never reaches the look:
the look judges a mark only on the snapshot whose own reply proved it is the recorded origin.

Which request:
`ZFS_IOC_OBJSET_STATS` on that origin snapshot,
the request the proof already needs for the snapshot's GUID and `createtxg`
(the crate's `objset_stats`,
same request as "Reading the mark").
No listing and no new request kind.

What it compares:
the one number the mark's entry holds (F3,
F4),
read exactly as "Reading the mark" describes.
It has no identity causes of its own:
the proof owns the identity comparisons,
and a mark read from an object the proof did not identify is never judged.

What it prints:
`OriginMarkedBeforeTheUnmount`,
or the shared `MarkUnreadable`,
with form A of "The refusals".
The owner's answer describes that state as "with the worktree still mounted and nothing written"
(record,
"A look before the unmount").
The look reads no mount,
so its message says the removal stopped before it took anything down and makes no claim about the mount;
and the plan-record and origin-cleanup designs take the removal's per-slot claim at the removal's first step,
before the proof,
so the claim is the one write a refusal there comes after and it is the removal's,
not a record of destruction
("What a refusal leaves,
and what the record keeps").

How it is told apart from the first check:
by the stage and by the cause's name.
The look's message says the removal stopped before it took anything down and nothing is pending;
the first check's says the clone is unmounted and the removal stays pending,
with the cancel command as the way back for a removal that is not forced.
Because the mark's two causes are named apart,
a receipt,
a control and `doctor` can say which of the three refused by the mark;
an unreadable mark shares one cause name,
and only the state sentence says which point met it.

What it costs,
and what step 14 must offer:
"that the look costs no extra read is an inference until the step that makes the lineage proof is built"
(record,
same section).
It holds when step 14's proof reads the origin snapshot with the crate's `objset_stats`,
which returns the whole property list (F5,
F11),
and hands that reply,
or the mark's value,
to the look.
It fails when the proof reads the snapshot with `zc_simple` set,
which leaves the property list empty (F5),
or resolves the snapshot by name alone without keeping a reply.
So step 14 must offer one of two things:
the reply it reads for the snapshot's GUID and `createtxg`,
or a call of the look made while that read's request is still in hand.
If it offers neither,
the look adds one `ZFS_IOC_OBJSET_STATS` per removal,
and the cost is that ioctl.

### One check, called at every snapshot request and before the head destroy

One function,
`check_planned_origin`,
called at the two places the removal is about to send a request.
It is not the look:
the look takes no plan,
has no identity causes and runs inside the proof ("The look before the unmount").
The function has one input and one verdict:

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

Every refusal names the object it judged and every way forward,
in the idiom "The refusals" of the descriptor mount records
(`package/git-policy/cli/SPEC.md:5834` to `:5840`;
`sweep_clone_check.rs:206` to `:280`).
The two checks also name the field that disagreed with the recorded and the live value,
and what the removal has already destroyed;
the look names no field of a plan,
because no plan exists when it runs.
A refusal's text is read by a person:
it names no document and no field of a record,
and it uses no word a person has not met in the message itself.

The cause decides which of three forms the text takes,
because the sentences that are true differ by cause.

Form A,
the look's,
for `OriginMarkedBeforeTheUnmount` and for an unreadable mark at the look:

```text
refusing to remove the ZFS worktree clone `<clone name>`: <the cause, with the object and the field>.
This removal has not destroyed anything yet. This check changed nothing and the removal stopped before it took
anything down: nothing is pending and there is nothing to cancel. Ways forward: <the reads and the acts a person
has>.
```

Form B,
a check's,
for `OriginMarkedForDeferredDestroy` and for an unreadable mark at a check:

```text
refusing to remove the ZFS worktree clone `<clone name>`: <the cause, with the object and the field>.
<the history sentence>. This check changed nothing, and the clone is unmounted.
The removal stays pending; `cancel-worktree-removal <slot>` is the way back for a removal that is not forced,
and it reports any snapshot already destroyed. Ways forward: <the reads and the acts a person has>.
```

Form C,
a check's,
for a planned object that is not there or is not the planned one,
and for a check that could not judge:

```text
refusing to remove the ZFS worktree clone `<clone name>`: <the cause, with the object and the field>.
<the history sentence>. This check changed nothing, and it sent no request.
<what this check could not compare, in the cause's own words>. The removal stays pending; putting the worktree
back needs a person to reconcile that first, and `cancel-worktree-removal <slot>` reports the same mismatch
rather than attaching anything. Ways forward: <the reads and the acts a person has>.
```

Form C's "reports the same mismatch rather than attaching anything" is the accepted reversal rule:
whatever reverses a removal must prove the clone,
its original lineage,
the Git registration,
the mount state and the exact destruction outcomes,
and "If a destroy outcome is unknown or the head is gone,
do not issue `mount`;
report the matching resume step or administrator recovery required"
(proposal `:1018` to `:1021`).

Each form claims only what its refusal point read,
plus the two states the removal's own order produced:
at the look,
before the removal has taken anything down;
at the checks,
after the unmount (proposal `:886` to `:888`).
The look makes no claim about the mount,
because it reads no mount:
a slot whose clone somebody else unmounted,
with no removal under way,
would be a first run the look misdescribes.
Neither form claims that anything else stands,
because a check reads three objects and a cause such as `CloneIsNotThePlannedOne` arises exactly where a planned
object is not what its name says any more.
The Git registration is the removal's own step 1 read and is not repeated here.
The look always prints the first form of the history sentence,
because no plan exists when it runs.

The history sentence comes from the plan's recorded outcomes,
never from which call site reached the check,
so it is true on a resumed removal and at a later chunk's check:
- a plan that records no destroyed tuple says
  `This removal has not destroyed anything yet.`
- a plan that records destroyed tuples names every one and says
  `This clone has already lost <the names>, in this removal or an earlier one, and they do not come back.`

The same history sentence covers both check points.
What differs between a first refusal and a later one is only what the plan records,
so no sentence claims "this run" did or did not delete something,
and the look says outright that this removal has destroyed nothing,
because it cannot have:
it ran before the removal wrote anything of its own beyond the per-slot claim it takes at its first step.
`cancel-worktree-removal` is step 17's command (implementation plan `:1958`),
so a control compares the name as a fixed string,
and a refusal may name a command the build does not yet carry.
Whether a forced removal may be cancelled at all is open in the forced-removal record,
which is why form B scopes the sentence to a removal that is not forced.

The causes,
each with the object and field the message names:
- `OriginMarkedBeforeTheUnmount`:
  the origin snapshot is marked,
  judged by the look inside the proof;
  form A.
- `OriginMarkedForDeferredDestroy`:
  the origin snapshot is marked,
  judged by one of the two checks;
  form B.
  Both name `dds_num_clones` and `userrefs` and what they mean (F9,
  F10,
  F17).
  The two causes are one state at two points;
  they are named apart so a receipt,
  a control and `doctor` can say which point refused.
- `PlanIncomplete`:
  the plan does not carry the field named,
  so no object is proven and nothing is judged;
  form C.
  Whether a plan missing a field is the seven answers' unreadable record,
  which is refused by naming the file,
  is the plan record's reader to say
  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
  "A record that cannot be read");
  this design's text names the field and the plan's home,
  and if that reader calls it unreadable,
  the message names the plan file too,
  a change this design accepts without designing it.
- `CloneGone`:
  the planned clone is not there (the kernel answered `ENOENT`);
  form C.
- `CloneIsNotAFilesystem`:
  the object at the clone's name is not a filesystem or is a snapshot;
  form C.
- `CloneIsNotThePlannedOne`:
  `clone_guid` differs from the plan's;
  form C.
- `CloneHasNoOrigin`:
  the clone reports no origin;
  form C.
- `CloneOriginDiffers`:
  the clone's live origin is not `origin_snapshot_name`;
  form C.
- `OriginDatasetIsNotThePlannedOne`:
  the origin dataset's GUID differs from the plan's;
  form C.
- `OriginDatasetGone`:
  the origin dataset is not there;
  form C.
- `OriginDatasetIsNotAFilesystem`:
  the object at the origin dataset's name is not a filesystem,
  so the plan's dataset identity cannot be compared;
  form C.
- `OriginIsNotASnapshot`:
  the object at the origin snapshot's name is not a snapshot;
  form C.
- `OriginSnapshotIsNotThePlannedOne`:
  `origin_snapshot_guid` differs from the plan's;
  form C.
- `OriginSnapshotCreatetxgDiffers`:
  the origin snapshot's `createtxg` differs from the plan's;
  form C.
- `OriginSnapshotGone`:
  the origin snapshot is not there,
  so its mark cannot be judged;
  form C.
- `MarkUnreadable`:
  the reply has no `defer_destroy`,
  or its value is not 0 or 1,
  or the property list is missing,
  or the start-time gate refused raw requests,
  or the device could not be opened.
  It takes the form of the point that met it:
  form A at the look,
  form B at a check.
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
all three refuse,
and the message names the plan's tuple and says the settle rules of the resume own the next attempt
(proposal `:963` to `:967`).
The check never proceeds over a gone object,
because the request that follows would act on a name nobody proved.

Four full messages.
The look's:

```text
refusing to remove the ZFS worktree clone `monochromatic-git-worktree-clone-datasets/fix-flaky-test`: its origin
snapshot `monochromatic-git-checkout@git-worktree-clone-origin-7f3a9c2e` is marked for deferred destroy
(`defer_destroy` reads 1), and the kernel reports 1 clone of it and 0 user holds, so destroying this clone destroys
that snapshot in the same transaction. This removal has not destroyed anything yet. This check changed nothing and
the removal stopped before it took anything down: nothing is pending and there is nothing to cancel. Ways forward:
`zfs get -H -o value defer_destroy monochromatic-git-checkout@git-worktree-clone-origin-7f3a9c2e` and `zfs holds
monochromatic-git-checkout@git-worktree-clone-origin-7f3a9c2e` show the mark and the holds. The mark cannot be
cleared in place, so this slot's removal finishes only outside the wrapper: either you accept that snapshot's
destruction and destroy the clone yourself, which takes the worktree's files with it and needs the worktree taken
down first, where a program holding it may refuse; or you clone that snapshot elsewhere first, which pins it, and
then destroy the clone. Either way, run the removal again once the clone is gone: that run refuses and names the
command that finishes what is left, or the reason when none can.
```

The first check's,
which is the cause this item exists for:

```text
refusing to remove the ZFS worktree clone `monochromatic-git-worktree-clone-datasets/fix-flaky-test`: its origin
snapshot `monochromatic-git-checkout@git-worktree-clone-origin-7f3a9c2e` is marked for deferred destroy
(`defer_destroy` reads 1), and the kernel reports 1 clone of it and 0 user holds, so destroying this clone destroys
that snapshot in the same transaction. This removal has not destroyed anything yet. This check changed nothing, and
the clone is unmounted. The removal stays pending; `cancel-worktree-removal fix-flaky-test` is the way back for a
removal that is not forced, and it reports any snapshot already destroyed. Ways forward: `zfs get -H -o value
defer_destroy monochromatic-git-checkout@git-worktree-clone-origin-7f3a9c2e` and `zfs holds
monochromatic-git-checkout@git-worktree-clone-origin-7f3a9c2e` show the mark and the holds. The mark cannot be
cleared in place, so this slot's removal finishes only outside the wrapper: either you accept that snapshot's
destruction and destroy the clone yourself, which takes the worktree's files with it and needs the worktree taken
down first, where a program holding it may refuse; or you clone that snapshot elsewhere first, which pins it, and
then destroy the clone. Either way, run the removal again once the clone is gone: that run refuses and names the
command that finishes what is left, or the reason when none can.
```

The second check's,
with a plan that recorded two destroyed snapshots:

```text
refusing to remove the ZFS worktree clone `monochromatic-git-worktree-clone-datasets/fix-flaky-test`: its origin
snapshot `monochromatic-git-checkout@git-worktree-clone-origin-7f3a9c2e` is marked for deferred destroy
(`defer_destroy` reads 1), and the kernel reports 1 clone of it and 0 user holds, so destroying this clone destroys
that snapshot in the same transaction. This clone has already lost `fix-flaky-test@manual-2026-11-21` and
`fix-flaky-test@sanoid-hourly-2026-11-22`, in this removal or an earlier one, and they do not come back. This check
changed nothing, and the clone is unmounted. The removal stays pending; `cancel-worktree-removal fix-flaky-test`
is the way back for a removal that is not forced, and it reports any snapshot already destroyed. Ways forward: as
for the refusal above, and the loss named here is the cost already paid.
```

And form C,
for a planned clone that is not there:

```text
refusing to remove the ZFS worktree clone `monochromatic-git-worktree-clone-datasets/fix-flaky-test`: the planned
clone is not there, and the kernel answered that no such dataset exists. This clone has already lost
`fix-flaky-test@manual-2026-11-21` and `fix-flaky-test@sanoid-hourly-2026-11-22`, in this removal or an earlier one,
and they do not come back. This check changed nothing, and it sent no request. Nothing this removal recorded about
that clone can be compared any more. The removal stays pending; putting the worktree back needs a person to
reconcile that object first, and `cancel-worktree-removal fix-flaky-test` reports the same mismatch rather than
attaching anything. Ways forward: a later run settles only after it has proved that clone is gone, and `doctor`
names the slot and what this removal recorded.
```

A refusal prints no destructive command.
Every attempt that meets one prints it,
because the refusal is the answer to that attempt and the printed report is the trace the project chose
(`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
"What trace of a destruction outlives the removal").
It prints the reads that show the state and names the decisions a person takes;
whether a wrapper command should ever destroy a marked origin is left alone by the record
("What the record does not decide").
The `doctor` wording that repeats a refusal is step 18's,
not this design's (F19).

The check writes nothing of its own,
so no refusal can leave half a record,
and there is no lock to take (rule `NLK`).

### What a refusal leaves, and what the record keeps

The project decided all three on 2026-11-22
(`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`).

The look's refusal takes nothing down and writes no intent and no plan.
It runs before the intent is written and before any unmount is requested,
so nothing is pending,
no plan exists,
and the removal has not taken anything down.
The one thing the removal has written by then is its per-slot claim,
which the plan-record design places at the removal's first step,
before the proof:
its step 1 is "the claim and the proof:
no plan is written"
(`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
"The order",
`:1055` to `:1084`),
and a claim is the removal's,
not a record of what it destroyed.
The look's message therefore says that nothing is pending and nothing is to cancel,
and it makes no claim about the mount,
which it does not read.
No record of destruction is kept,
because the look runs before any record of destruction exists,
and the cancel command has nothing to cancel.

A refusal at either check keeps the removal pending.
The clone stays unmounted,
the intent and the plan stand,
and the cancel command is the way back.
What the plan holds differs by point:
at the first check it holds the verified tuples and no outcome;
at the second it holds the batch's outcomes,
so it is the statement of which snapshots were destroyed.
The record's own words:

- "the removal is kept pending,
  the clone stays unmounted,
  and the worktree is out of use until a person deals with the mark and runs the removal again,
  or runs the cancel command to get the worktree back"
  ("What a refusal at the first check leaves").
- "the removal is kept pending with its record of which snapshots were destroyed,
  the clone stays unmounted,
  and the cancel command is the way back"
  ("What a refusal at the second check leaves").

Nothing about any of the three rewrites the intent,
reattaches the clone,
unlinks the plan or removes a registration.
The one family of cases in which a removal remounts the clone by itself is a holder's,
seen by the removal's scan
or blocking its unmount:
a scan's holder makes the removal "stop before any snapshot is deleted,
attach the clone at its slot again,
and report the holder"
(`doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
"What a removal without `--force` does when its scan sees a holder").
None of these three refusals is that case,
and the deferred-destroy record says why:
"a holder goes away by itself and a mark does not"
("What a refusal at the first check leaves").
The busy-unmount cancellation (proposal `:1357` to `:1359`,
`:1492` to `:1498`) is the other holder case,
and no refusal here is it either.

The destruction-plan design still removes a plan only after a completed removal's settle
(`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
"When it is removed"),
and the cancelled design assumes the intent and the plan are kept
(`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
"This design assumes it keeps the plan").
Both agree with what the project decided.

### An add's own destroy does not pass through these checks

The project decided on 2026-11-22 that an add which fails at any step destroys only what this add made and
recorded,
"the clone,
then its snapshot",
and finishes with Git's own checkout at the same slot
(`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
"Whether a failed add may finish with Git's own checkout by itself").

Those destroys are not a removal,
so these checks do not run for them:
the look belongs to the removal's step 1 lineage proof,
and both checks read a destruction plan,
a removal intent and a stage,
none of which an add writes.
The add's own path,
order and reporting are that design's,
not this one's.

What the checks answer for a clone the add itself just made,
if an add ever called them with the tuple the add recorded:
for a clone whose origin is the unmarked snapshot the add made,
they proceed,
because every identity the plan would hold is what the add recorded;
for one whose origin a same-account process marked,
they refuse,
because the accepted residual covers an add as much as a removal (A6).
Nothing here asks an add to call them,
and nothing here forbids it.

One consequence belongs in this design.
The add destroys the clone and then its snapshot.
A mark set on the snapshot the add made reaps it in the same transaction as the clone destroy,
when that clone is the snapshot's last child and no hold pins it (F10),
so the add's next destroy of that snapshot answers that it is not there.
Nothing of the person's is lost beyond what the add made:
the snapshot is the add's own,
and the add was going to destroy it anyway.
The add's fallback has to treat that one `ENOENT` as done,
and how it reports that is that design's to say.
A marked snapshot that another clone or a hold pins does not go with the clone:
the add's own destroy of it answers `EEXIST` or `EBUSY` (F18),
the snapshot stands,
and the add reports what it could not destroy rather than claiming a clean fallback.
Either way the add destroys nothing of the person's,
and the checks judge none of it.
An add that destroys another slot's leftover origin snapshot,
one of the five `git worktree` commands the project's answer names,
reads that snapshot's mark through `origin_mark`,
not through these checks,
because no plan of that slot's stands for it and the check needs a plan.

### How a person finishes a refused removal

The mark cannot be cleared (F8),
and a marked snapshot cannot be destroyed while any clone of it exists (F18).
So while the mark stands the wrapper refuses every attempt at that slot,
and only these things can then happen:

- The person leaves it as it is.
  After a refusal at the look the removal has taken nothing down and written no record of destruction,
  so the slot keeps whatever state it had and the next removal starts with its first proof again.
  After a refusal at either check the worktree is unmounted and out of use until someone takes one of the ways
  below.
- The person accepts the destruction.
  Outside the wrapper they destroy the clone (`zfs destroy` of its remaining snapshots,
  then of the head,
  or `zfs destroy -r` of the clone),
  taking the worktree down first where it is mounted;
  a program holding it may refuse that.
  The destroy reaps the marked origin in the same transaction (F10) and takes the worktree's files with it.
  What a later run does with what is left is not settled for a clone destroyed outside the wrapper:
  - the nearest accepted decision is the project's "Settle + own record":
    the Git registration is settled "after clone destruction is proven",
    and "a separate durable record carries the origin cleanup"
    (proposal `:954` to `:957`,
    `:1460` to `:1463`);
    that covers an origin that stands,
    because its cleanup goes to its own record;
  - it has two limits here.
    It needs the clone's destruction proven,
    and the plan design's absence rule restates the accepted proof;
    what no record states is which path a removal takes from a plan that says the head is present,
    which is the request below;
    and the origin-cleanup record is published at that design's own step 6,
    after the first check (its `:1147` and `:1154`),
    so at the look and at a first-check refusal it does not exist yet;
  - the one accepted sentence about a gone clone and origin is narrower than this state:
    "A removal whose recorded clone and origin are already gone can settle a missing registration only after proving
    there is no replacement"
    (proposal `:460` to `:461`);
    it is about a registration Git no longer holds,
    and a person's own destroy does not make the registration missing.
  So this design settles neither completion and promises nothing for them.
  Two things it does know.
  The proof of a gone clone is the accepted one,
  "never from a missing name alone",
  where a head is settled "only when its name is absent and no child of the pinned parent carries the recorded GUID"
  (proposal `:963` to `:967`).
  And the project decided what happens where that proof cannot be made,
  under "A slot whose recorded clone is gone and cannot be proved gone":
  "Print an explicit command that removes the record after you confirm"
  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
  that heading),
  whose guard and proof are open.
  The binding design gives one name for it,
  `git monochromatic-git-policy-cli settle-worktree-clone-record <slot> --confirm-removal-of <slot>
  --confirm-clone-guid <the last four decimal digits of cloneGuid=>`,
  in its own veto entry V21 (read at `07858070bb`,
  `:3216` to `:3220` and `:4805` to `:4809`),
  and that name is adopted,
  so a refusal's text may print it.
  The plan design (unapproved;
  read at `cf51d140af`,
  "Which resume path a removal takes") reconciles the two.
  A plan whose head word is `requested` or `unknown` beside a gone clone
  takes the head-gone path with the accepted proof (proposal `:963` to `:967`);
  a plan whose head word is `present` or `refused` is an outside destroy,
  and the removal refuses and names the settling command;
  a name carrying another GUID is refused with the reuse named and no command;
  and a pool that cannot prove the absence is refused,
  naming the settling command for every head word but `gone`.
  The messages say that the removal can be run again once the clone is gone,
  and that the run refuses and names the command that finishes what is left,
  or the reason when none can.
- The person keeps the history.
  Outside the wrapper they first clone the marked snapshot under another name,
  which pins it (F21),
  and then destroy the slot's clone the same way.
  The marked snapshot survives under the new clone,
  and the slot is the standing-origin state the bullet above covers:
  the registration can be settled once the destruction is proven,
  and the origin cleanup goes to its own record;
  what proves the destruction is the one gap that bullet names.
- The person cancels the removal with `cancel-worktree-removal <slot>`,
  which is the sibling's design,
  and leaves the slot unmounted or reattached as that design's checked reattach decides.
  The mark still stands,
  so a retry cannot finish while it does:
  a retry over a kept plan reads it,
  proves it and adopts it,
  carrying its loss forward,
  so the worktree is taken down again before the first check refuses,
  and the look does not run on it;
  unless its identities no longer match live,
  where the plan record has the retry refuse before anything is deleted,
  or an `unknown` tuple of it cannot be settled by the plan record's head-present arm,
  where it refuses naming the tuple and the read that failed
  (read at `cf51d140af`,
  "The removal intent");
  it names no wrapper command in either case,
  and the settling of an `unknown` tuple comes before the first deferred check,
  so no check refuses first;
  a cancel that destroyed nothing and completed leaves no plan,
  and that retry is a first run (the paragraph after this list).

A retry after a cancel is the gate's other edge case,
and the project has decided the part that matters here.
The cancel returns the clone's desired-state property to `mount`.
A cancel that destroyed nothing removes its plan,
because a plan that witnesses nothing is removed by the cancellation
(plan record,
read at `3cd520b6a0`,
"A cancellation,
and the plan it leaves"),
so a retry after it is a first run and the look runs with no change.
A cancellation killed before it removed such an all-`planned` plan leaves that plan to the next removal,
which adopts it (the same section),
so that retry has a plan and does not look,
and V11's alternative names it.
A cancel that destroyed part of the clone's snapshots keeps its plan,
and the seven answers decide how long it stays:
"until that clone's next removal",
where "The next removal of that clone reads the record,
proves it,
carries the loss forward and ends it"
(`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
"How long the plan a cancellation keeps stays").
The plan record states that retry in the same shape:
"A retry after a cancel therefore always presents the kept plan:
it adopts it under its own nonce,
keeps every tuple and outcome,
and proceeds to the first deferred check;
the look does not run on it"
(read at `3cd520b6a0`,
`:2166` to `:2170`).
That quoted "always" holds only where a plan is kept,
and its adoption has one exception:
the plan record's nonce rule adopts a plan beside a `mount` intent
"if its identities still match live,
once any `unknown` tuple has been settled by the head-present arm",
and refuses only where that arm cannot settle one,
naming the tuple and the read that failed
(read at `cf51d140af`,
"The removal intent").
So a retry over a kept plan with an `unknown` outcome settles it first and is then adopted,
and no check runs before the settling.
In every state where a plan stands a retry does not look,
whether that plan records nothing destroyed,
some destroyed,
or an `unknown` outcome:
the gate needs no removal intent and no plan together,
and a cancellation that destroyed something always leaves the kept plan.
Where in the next removal's order the kept plan ends is the cancelled design's to say.
Its own words still have "A later removal of the same clone rewrites the plan under the same key,
and that is the natural end of the loss record's life"
(read at `eb7a78eccc`,
`:1648` to `:1649`),
which reads as replacing the old record where the seven answers and the plan record carry its loss forward;
that difference is that design's to reconcile,
and this design needs only what both give:
the retry's first refusal names the carried loss,
which the history sentence does.

The four messages say these two completions,
the cancel command at the two checks,
and at the look the fact that nothing is pending and nothing was taken down;
the second check's message repeats that the loss is already paid.
The wrapper itself never finishes a removal over a marked origin:
the record does not decide that the wrapper may ("What the record does not decide").

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
  The project decided that only `git worktree add`,
  `remove`,
  `list`,
  `prune` and `repair` destroy such a leftover snapshot,
  printing one line
  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
  "Which commands may destroy a leftover origin snapshot"),
  so `origin_mark` is read by the attempt in all five of those commands,
  and never by these checks.

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

- Step 14 builds the proof and leaves one named hook at the place the proof has the origin snapshot's reply in
  hand,
  immediately after it resolves that snapshot's identity and before the intent is written
  ("The look before the unmount").
  The proof is handed what the gate needs:
  whether this is a first run,
  which is its read of the clone's desired-state property and its lookup of the plan,
  so the hook is told "first run" or "not" and nothing else.
  The hook proceeds when nothing judges it,
  because step 15 depends on step 14 in the implementation plan (`:1858` to `:1860`),
  so step 14 cannot call code step 15 builds.
  Step 14's controls cover the hook's place in the order:
  it runs before the intent write and before the unmount request,
  a hook with no judge refuses nothing and hands the reply on,
  and the proof refuses an identity before the hook is reached.
- Step 15 builds the look and places it inside step 14's hook,
  and step 15's controls cover the judgement:
  a marked origin refuses there (control 19),
  an unreadable mark refuses there (20),
  an identity the proof refused never reaches it (21),
  and the three forms print what each cause's form requires (25).
  Step 15 also builds the function,
  the three reads,
  the exported `origin_mark`,
  the plan's field requirements as a refusal,
  the causes and the refusal words,
  what each refusal leaves.
  Step 15 owns what each refusal prints and leaves for all three points,
  because the implementation plan's step 15 builds "What each refusal prints,
  and whether a refusal keeps the removal record"
  (`:1818` to `:1822`).
- Step 16 writes the controls that need its call sites and loops:
  22 (step 4's reproof does not look),
  23 (a chunk's reproof does not look),
  24 (a rerun over a standing mark),
  26 (an intent and no plan does not look) and
  27 (a retry after a cancel does not look).
  None of them can be written earlier,
  because each drives the reproof call,
  the chunk loop or the call sites step 16 places.
- Step 16 places both calls:
  one before each snapshot request,
  each chunk included
  (implementation plan `:1868`,
  `:1872` to `:1875`),
  and one after the fresh enumeration and before the head request (`:1876` to `:1877`).
  It also adds the write seam the refusals' controls drive,
  and the pause route M3b and M3c-2 drive:
  a file named by `MONOCHROMATIC_ZFS_REMOVAL_PAUSE`,
  live only under the `measurement-routes` feature (F24),
  that writes `<path>.ready` and waits for `<path>.go` at the phase `<path>.at` names
  (`before-first-check` or `before-head-check`),
  bounded by the removal's own deadline,
  which the plan sizes later and no record sizes today,
  and recording a wait that expired rather than honouring it.
  Building it widens that feature's own comment in `package/git-policy/cli/Cargo.toml:71` to `:75`,
  which today names the clone fence alone.
  It writes and first runs the scripts of M0b,
  M3 and M4 once the route is whole,
  the managed slot's install being placed by hand from step 6's and step 19's files until step 19 ships it.
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

The implementation plan's lists for steps 14 and 15 (`:1738` to `:1860`) name neither the hook nor the look,
so this design asks that plan to add them:
the hook to step 14's "What it builds",
and the look,
its refusal and its controls to step 15's.

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
10. `every_cause_prints_its_form_with_the_object_the_field_and_the_ways_forward`:
    one message per cause,
    compared against the form its cause takes and the fields it names,
    at the point that judges it.
11. `the_history_sentence_follows_the_plans_outcomes`:
    a plan with no outcome prints "has not destroyed anything yet",
    one with two destroyed tuples names both,
    and the two messages are otherwise identical;
    a resume whose earlier run,
    or whose clone's earlier removal,
    destroyed the tuples prints the names,
    never says "this run",
    and says "in this removal or an earlier one".
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
19. `a_marked_origin_refuses_at_the_look_before_anything_is_taken_down`:
    the mark reads 1 on the origin snapshot the proof resolved,
    the verdict is `Refused` with `OriginMarkedBeforeTheUnmount`,
    and the fixture records that the intent write,
    the plan's first write,
    the unmount request and every snapshot request were never called.
20. `an_unreadable_mark_at_the_look_refuses_before_anything_is_taken_down`:
    an absent key,
    a text value,
    a missing property list and a gate refusal each produce `MarkUnreadable` at the look,
    and the same four calls are never recorded.
21. `an_identity_the_proof_refuses_never_reaches_the_look`:
    the proof refuses because the clone's GUID differs from the binding record's,
    and the fixture records that the look was never called.
22. `the_step_four_reproof_does_not_look`:
    a plan,
    a mark reading 1 and step 4's reproof;
    the look is never called,
    and the first check refuses with the plan's history sentence.
23. `a_chunk_reproof_does_not_look`:
    two chunks,
    a mark set before the second chunk's reproof;
    the look is never called,
    and the second chunk's check refuses.
24. `a_rerun_over_a_standing_mark_refuses_where_this_run_deletes`:
    a plan recording two destroyed tuples and no remaining tuple,
    and a mark reading 1;
    the look is never called,
    the first call site is skipped because no snapshot request is sent,
    the head check refuses,
    and the message names both tuples and never says "has not destroyed anything yet".
    Its second variant leaves one tuple to send:
    there the first check refuses with the same names.
25. `each_cause_prints_the_form_that_is_true_for_it`:
    per form,
    a list of phrases the text must contain and phrases it must not.
    Form A must say that nothing is pending and that there is nothing to cancel,
    and must not say "unmounted" or "the way back".
    Form B must say "the clone is unmounted" and "the way back for a removal that is not forced",
    and must not say "nothing is pending".
    Form C for `CloneGone` must say that the planned clone is not there,
    that the check sent no request,
    and that the cancel command "reports the same mismatch",
    and must not say "the way back" or "nothing is pending".

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
A mark set between the look and the first check.
Forced by the pause at `before-first-check` (M3b),
which sets the mark after the proof returned and before the check runs;
the first check refuses,
and the look's control shows the same mark refused earlier when it is already there.
This is the window the record names:
"the state of a refusal at the first decided check then arises only when the
mark appears between that look and the check".

R7.
A same-account substitution between a check and the request it precedes.
Not forced and not claimed:
it is L4,
and the measured arms of [VM item 12][vm-12] are its record.

26. `a_clone_with_a_removal_intent_and_no_plan_does_not_look`:
    the clone's desired-state property reads `unmount` and no plan exists,
    which is what a kill between the intent write and the plan write leaves;
    step 1's proof runs,
    the look is never called,
    and the check that follows the verified unmount refuses.
27. `a_retry_after_a_cancel_does_not_look`:
    the property reads `mount` and the kept plan records two destroyed tuples;
    the look is never called,
    the run adopts the kept plan,
    keeps every tuple and outcome,
    and the first check,
    or the head check when no snapshot is left to send,
    refuses with the carried loss named.
    Its second variant is the retry after a cancel that destroyed nothing and completed:
    the plan is gone,
    so the look runs,
    which is the gate working as defined (control 26 covers the intent half).
    Its third is the retry after a cancellation killed before it removed an all-`planned` plan:
    the plan stands and records nothing destroyed,
    the look is never called,
    the run adopts the plan,
    and a check refuses saying that nothing has been destroyed yet.
    Its fourth is the retry over a plan with an `unknown` outcome:
    the look is never called,
    and the run settles the tuple by the plan record's head-present arm and adopts the plan,
    or refuses,
    naming the tuple and the read,
    where the arm cannot settle it;
    that one is a control against the plan record's text,
    not against this design's.

### Controls that need a guest

28. `a_marked_snapshot_reports_the_mark_and_an_unmarked_one_does_not`,
    in `package/zfs/request-to-openzfs-kernel-device/src/kernel_device/live_read_tests.rs`:
    the identity read on a snapshot before a deferred destroy reads 0,
    after it reads 1,
    the entry carries no `source`,
    and `dds_num_clones` follows F9's meaning.
    It runs through the package's e2e recipe like the other `#[ignore]` live controls
    (`live_read_tests.rs:120` to `:121`).
29. The removal arms of M3 below,
    in a new guest recipe under `package/git-policy/cli/e2e/`,
    written and first run by step 16,
    and driven the way `mount-git-worktree-clone-datasets-guest.ts` drives the step 8 arms.

## Measurements owed

None of these ran in this round.

Three probes can run as written once a guest exists:
M0a,
M1 and M2.
The rest are specifications,
because the route they drive does not exist yet.
Each specification gives the state it needs,
with the record or source line that says so,
the one action,
the refusal or result it expects with the step of the accepted order that produces it,
and the bound on its wait,
and each names the build step that writes its script and first runs it in a guest.
A specification stands where a script would pretend to be evidence:
three reviews found three different reasons a hand-written script could not start,
and none of these arms can run until the steps that build the add route,
the mount handler and the pause route exist.

Where OpenZFS's programs live in the guest:
the step 8 guest recipe keeps the 2.4.4 userspace at `/opt/openzfs-2.4.4`
and runs every pool command with
`/opt/openzfs-2.4.4/sbin:/opt/openzfs-2.4.4/bin:/sbin:/usr/sbin:/bin:/usr/bin` on its `PATH`
(`package/git-policy/cli/e2e/mount-git-worktree-clone-datasets-guest.ts:105` to `:106`,
`:291`),
so a probe that runs `zfs` through `sudo` passes that `PATH` in the `env` list rather than relying on what the
guest's sudoers keeps.
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
# (proposal `:858` to `:860`)
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

A specification.
Its script is written and first run by step 16,
the first step whose measurement needs a managed slot;
until step 19 ships the installer,
the install it needs is placed by hand in the guest from the same files.

M3 removes a slot the wrapper manages,
so M0b runs the built add at the canonical destination,
as the account,
on M0a's pool.
A hand-made registration is neither the binding record nor the intent a removal reads,
so M3 never makes its slot by hand.

The state it needs,
each part with what says so:

- M0a's pool,
  datasets,
  account and grants,
  unchanged.
- The checkout is a Git repository with one commit,
  and the checkout,
  its `.git`,
  `.claude/worktrees` and the account's run folder belong to `probe`,
  because the add writes the claim as that account
  (proposal `:373` to `:375`)
  and the account owns the directory and each slot (proposal `:132` to `:134`).
  M0a's root-made checkout is therefore given to `probe` before the add runs,
  and `/home/probe/.run` is made by root with `--owner probe --group probe --mode 0700`,
  because it is the `XDG_RUNTIME_DIR` the add writes into.
- The installed root-owned handler and its configuration,
  a live ZED,
  usable root-owned status and results folders,
  and a current handler readiness report for this pool,
  checkout and boot,
  because the add's preflight verifies each of them (proposal `:365` to `:374`),
  together with the checkout mount and the caller's visibility,
  `/dev/zfs` access,
  the ABI gate,
  effective grants,
  the slot parent's identity and owner,
  and mount propagation,
  which the same list names (`:365` to `:371`).
  The handler's install paths are
  `/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets`
  and the unit that starts it (`package/git-policy/cli/SPEC.md:266`,
  `:274`),
  and the installer is step 19's (implementation plan `:195`,
  `:2580`).
  The step 8 guest recipe does not install the handler:
  it drives the handler's modes directly
  (`package/git-policy/cli/e2e/mount-git-worktree-clone-datasets-guest.ts:3`,
  `:341`),
  so this install is M0b's own precondition,
  placed from step 6's and step 19's files.
- The wrapper as `git`:
  a copy of the built executable and a `bin/git` symbolic link to it,
  which is how every binary control runs it
  (`package/git-policy/cli/src/native/binary/binary_support_fixture_directories.rs:130` to `:131`,
  `:210` to `:213`).
- The origin is the snapshot the add makes,
  never one made by hand (proposal `:383` to `:385`),
  so M0b makes no snapshot.
- The clone is mounted at the slot,
  which the handler does on a qualifying sweep after the add
  (proposal `:380` to `:382`),
  and the assertion is on the mount point itself,
  `findmnt --mountpoint <slot>`:
  `findmnt --target <path>` prints the mount that contains the path,
  so it would pass on the checkout's mount while the clone is detached
  (`findmnt(8)`,
  its `--target` entry).

The one action:
the built add,
as `probe`,
at the path the route accepts and nowhere else
(the clone mounts at the slot,
proposal `:275` to `:277`;
the canonical path only,
`:1375` to `:1376`),
with its own `HOME` and `XDG_RUNTIME_DIR`.

The result it expects:
the clone attached at the slot,
proved by `findmnt --mountpoint`;
the add's claim and binding record written under the common Git directory;
the clone's `origin` property naming the snapshot the add made;
and then two own snapshots on the clone,
one manual and one sanoid-shaped,
which are the set a removal inventories (proposal `:858` to `:860`).
Bound:
300 s for the add,
and 120 s for the mount assertion.
A mount that is not there by either bound fails the arm and names the handler's last readiness report.

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
# The guest PATH goes through env, as the header says a probe that runs zfs through sudo must pass it
# (the recipe's constant, mount-git-worktree-clone-datasets-guest.ts:106).
sudo --user probe env \
    PATH=/opt/openzfs-2.4.4/sbin:/opt/openzfs-2.4.4/bin:/sbin:/usr/sbin:/bin:/usr/bin \
    zfs destroy -d "$origin"
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

Control 28,
in the crate's own pool rather than M0a's,
because the crate's live test makes the pool it needs (`live_read_tests.rs:127`).
It shows that the field the check reads is the field the kernel fills,
for both values,
through the crate rather than through `zfs`.

### M3: the wrapper's refusals

Specifications.
Their scripts are written and first run by step 16,
which places the two calls and builds the pause route they drive,
in a guest where M0b's install is placed by hand.
They cover three refusal points now:
the look inside step 1's proof,
the first check and the second check.

M3 needs M0b's managed slot.
Each arm starts from a fresh M0a and M0b,
because each arm destroys objects that the next one needs,
and the round tears its fixture down at the end:
`zpool destroy zfsdeferred`,
then the pool image removed and `userdel --remove probe`,
because every arm uses the same pool name and the same slot name,
so a second round would otherwise collide.
Each arm states whether it is a control (an assertion that must hold) or a reach report (counts).
One arm reruns a removal that an earlier arm left pending,
because the owner's answers make that the normal state after a refusal over a standing mark.

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
A pause lands after the *look* by construction:
the look runs in step 1's proof,
before the intent and the unmount,
and the phase names a point the check reaches,
so an arm that sets the mark at the pause reaches the first check with the look already passed.
Every arm puts that path under `/home/probe/.run/`,
which `probe` owns and root can write in,
because the removal runs as `probe` and writes `.ready` there while root writes `.at` and `.go`.
Every line below sets the variable after `sudo`,
so nothing depends on what the guest's sudoers keeps
(`sudoers(5)`'s `env_reset` is on by default).

#### M3a: the look before the unmount refuses (control)

The state it needs:
M0b's slot,
mounted,
asserted with `findmnt --mountpoint`;
the origin's name read live from the clone
(`man/man7/zfsprops.7:252` to `:255`);
and the mark made before the run by the account,
which marks because the origin still has this one clone
(`module/zfs/dsl_destroy.c:322` to `:333`);
the account's reach is the checkout's descendant `mount,destroy` grant (proposal `:341` to `:343`).

The one action:
`git worktree remove <slot>` as `probe`,
through the wrapper as `git`,
with its own `HOME` and `XDG_RUNTIME_DIR`.

The refusal it expects,
and the step that produces it:
the look inside step 1's lineage proof refuses with `OriginMarkedBeforeTheUnmount`,
after the proof resolves the clone's live origin and compares that snapshot's identity,
and before the proof returns,
so before step 2 writes the intent and requests the unmount (proposal `:886` to `:888`),
before step 3 writes the plan and before any snapshot request.

This is a first run only:
the clone has no intent and no plan when the proof runs,
which is the one state the look runs in
("The look before the unmount").

Pass:
the message in its look form
(the removal has taken nothing down,
nothing is pending and nothing is to cancel,
and no claim about the mount),
the exit status,
`findmnt --mountpoint <slot>` still naming the slot
(the arm's own read,
not the message's),
the desired-state property still reading what the add left,
no plan in the plan folder,
the removal's per-slot claim file present,
which the arm reads and does not judge for release,
the registration listed,
the clone and both its snapshots still listed,
the origin and the marked snapshot still listed,
and no snapshot request issued.
Bound:
300 s for the removal.

#### M3b: the first check refuses (control)

The state it needs:
M0b's slot,
mounted;
the origin's name read live from the clone
(`man/man7/zfsprops.7:252` to `:255`);
and the pause at `before-first-check`,
its path under `/home/probe/.run/pause`.

The one action:
start the removal with the pause,
wait for it to pause,
have the account mark the origin,
release the pause,
and wait for the removal to end.

The refusal it expects,
and the step that produces it:
the look passed,
because nothing was marked when the proof ran (proposal `:854` to `:857`);
step 2 wrote the intent,
printed the names and requested the unmount (`:886` to `:888`);
step 3 wrote the plan;
step 4 ran the fresh all-namespace verification and reproofed the identities (`:896` to `:899`);
and the first check then refuses with `OriginMarkedForDeferredDestroy`,
before the plan's `requested` rewrite and before any snapshot request.
This is the record's state:
the first check's refusal "arises only when the mark appears between that look and the check"
(`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`,
"A look before the unmount").

Pass:
the message in its checks form
(the clone unmounted,
the removal pending,
`cancel-worktree-removal <slot>` named as the way back),
the exit status,
the clone and both its snapshots still listed,
the origin and the marked snapshot still listed,
the intent reading `unmount`,
the plan present with no outcome,
the registration listed,
and no snapshot destroyed.
Bounds:
120 s for the pause's `.ready`,
and 300 s for the removal.
A pause that does not arrive within its bound fails the arm and names what it last saw,
because the removal was then not paused.

#### M3c: a replacement clone whose different origin is marked (control)

Two arms,
because the replacement can be in place before the run or made after step 4's reproof,
and only the second reaches the check's own identity comparison.

M3c-1,
the replacement already in place.
The plan lists this as step 15's acceptance test
(`:1843` to `:1844`,
and the proposal's own words at `:2095` to `:2098`).

The state it needs:
M0b's slot;
then root destroys the planned clone outside the wrapper,
after its own snapshots,
in the removal's own order (proposal `:894` to `:900`),
makes another checkout snapshot `@other`,
clones it at the slot's name
(a clone needs its snapshot first,
`man/man8/zfs-clone.8:76` to `:77`),
and the account marks `@other`,
which now has its one clone (`module/zfs/dsl_destroy.c:322` to `:333`).
The replacement is not mounted at the slot.

The one action:
the removal,
as in M3b.

The refusal it expects:
the first refusal the built route reaches.
The proposal's boundary and registration checks open the removal
(proposal `:796` to `:812`),
and the live clone's identity comparison is step 1's
(`:854` to `:857`),
so the expectation is a refusal naming the binding's clone GUID and the live one:
only the binding exists,
because no plan was written,
and the route may refuse earlier still for the missing mount.
That is a prediction about the built order,
recorded as one.
A refusal at the look is not expected here:
the look judges a mark,
and this arm's origin snapshot is `@other`,
which is marked,
so the proof refuses on the clone's identity before any mark is judged.

Pass:
the first refusal reached names the binding's clone GUID and the live one,
nothing was destroyed,
the replacement clone and `@other` stand,
the registration is unchanged,
and no plan exists.
This shows the proposal's sentence about a replacement being refused is met by step 1,
and that the check's identity reads are defence in depth for the window after step 1.
Bound:
300 s.

M3c-2,
the replacement made at the pause before the first check,
so the check's own comparison decides.

The state it needs:
M0b's slot;
another checkout snapshot `@other`,
held by root and then marked by the account,
because a deferred destroy marks only a held or cloned snapshot
(`module/zfs/dsl_destroy.c:322` to `:333`;
the grant,
proposal `:341` to `:343`);
and the pause at `before-first-check`,
its path under `/home/probe/.run/pause`,
the account's own folder.

The one action:
start the removal with the pause,
wait for it to pause,
have root destroy the planned clone's own snapshots and then the head and clone `@other` at the slot's name,
release the pause,
and wait for the removal to end.

The refusal it expects,
and the step that produces it:
at the pause the plan is written and the clone unmounted,
so the first check's first read is the clone's GUID,
which now answers a different object,
and the check refuses with `CloneIsNotThePlannedOne`
before the origin reads and before the plan's `requested` rewrite.

Pass:
that refusal,
naming the plan's GUID and the live one,
with the replacement and `@other` standing,
and the plan still naming the destroyed snapshots this arm took outside the wrapper.
Bounds:
120 s for the pause's `.ready`,
and 300 s for the removal.

#### M3d: the check before the head destroy refuses (control)

The state it needs:
M0b's slot;
the origin's name read live before the run
(`man/man7/zfsprops.7:252` to `:255`);
and the pause at `before-head-check`,
its path under `/home/probe/.run/pause`.

The one action:
start the removal with the pause,
wait for it to pause,
have the account mark the origin,
release the pause,
and wait for the removal to end.

The refusal it expects,
and the step that produces it:
the look and the first check passed,
because nothing was marked when each read the origin;
the batch destroyed the clone's own snapshots and the plan recorded each outcome;
the fresh enumeration ran (proposal `:926`);
and the head check refuses with `OriginMarkedForDeferredDestroy`
before the plan's `head=requested` rewrite and before the head request,
with the history sentence naming the two destroyed snapshots.

Pass:
that message,
the exit status,
the origin and the marked snapshot still listed,
the clone head still listed with its own snapshots gone,
the plan present with the batch's outcomes,
and no head request issued.
`doctor`'s lines are not a pass condition here:
they are step 18's (implementation plan `:2116` to `:2119`).
For the proposal's "otherwise" clause (`:2095` to `:2098`),
the same pause with a replacement instead of a mark refuses with `CloneIsNotThePlannedOne` at the head check.
Bounds:
120 s for the pause,
and 300 s for the removal.

#### M3e: the residual (reach report, not an owed arm)

The state it needs:
M0b's slot,
and the origin's name read live.

The one action:
run the removal while a same-account watcher marks the origin the moment the clone's snapshot list empties.
The watcher is bounded at 300 s,
and a wait that runs out prints what it last saw and fails.

The result it expects:
counts,
not a guarantee:
how many runs had the head check see the mark and refuse,
and how many ended with the origin reaped.
The run is repeated in one interleaved round with an unchanged-input control,
and rule `QNB` applies:
the control arm's spread is reported beside the counts.
This is now the only arm that depends on timing,
because the record's other windows are reached through the pause.
Bound:
300 s for the removal and 300 s for the watcher.

#### M3f: the rerun over a standing mark (control)

The state it needs:
the state M3d leaves,
on the same fixture,
before its teardown:
the clone's own snapshots destroyed,
the plan recording their outcomes,
the head still present,
the intent standing,
and the origin marked.

The one action:
run the removal again,
as in M3b.

The refusal it expects,
and the step that produces it:
step 1's proof runs and does not look,
because the clone carries a removal intent and a plan (the rule of "The look before the unmount");
step 4's reproof does not look either;
every tuple the plan records is settled,
so this run sends no snapshot request,
and the first call site is skipped (V1,
proposal `:908`);
step 6's enumeration runs,
and the head check refuses with `OriginMarkedForDeferredDestroy`
before the plan's `head=requested` rewrite and before the head request,
with the message naming the two destroyed snapshots the plan records,
and never saying that the removal has destroyed nothing.

Pass:
that message,
the exit status,
every tuple and outcome the plan recorded still there,
with the check itself having written nothing,
the head still present with its own snapshots gone,
the origin and the marked snapshot still listed,
the intent still standing,
and no head request issued.
It shows that the loss stays visible on a rerun,
and that the head check is the point that keeps it visible when no snapshot is left to send,
which is what the owner's second answer chose.
Bounds:
300 s for the removal,
and no pause,
because no look runs.
The same rerun after M3b is this arm's other variant:
there the plan records no destroyed tuple and a snapshot request is still to send,
so the first check refuses,
with the message saying that this removal has not destroyed anything yet.

#### M3g: the retry after a cancel (control)

The state it needs:
the state M3d leaves,
then the cancel run as the cancelled design says,
which attaches the clone and keeps the plan,
whose tuples are all settled and not `unknown`;
the origin stays marked.
The cancel command is step 17's,
so this arm's script is written with M3's by step 16 and first run when step 17 lands.

The one action:
run the removal again,
as in M3b.

The refusal it expects,
and the step that produces it:
the plan is kept with its two destroyed tuples,
the property reads `mount`,
so the retry carries no removal intent and has a plan,
and the look does not run (V11);
step 1's proof runs,
the run adopts the plan under its own nonce and keeps every tuple and outcome,
which the nonce rule allows because no tuple is `unknown` and the identities match live,
step 4's reproof does not look,
and the first check,
or the head check when no snapshot is left to send,
refuses with `OriginMarkedForDeferredDestroy`,
naming the carried loss.

Pass:
that message,
the exit status,
the plan's tuples and outcomes still there,
the head still present,
the origin and the marked snapshot still listed,
the worktree taken down again before the refusal,
and no snapshot request.
It shows that the retry's first refusal carries the kept plan's loss,
which the seven answers require.
Bounds:
300 s for the cancel and 300 s for the retry.

### M4: the cost (specification)

Written and first run by step 16,
in the same round as M3.

The state it needs:
M0b's slot,
once per arm of the round.

The one action:
one removal with the look and both checks,
and one with none of them,
alternating in the same round on the same build tree.
The arm without them is a `measurement-routes` build that serves the look's one read and each check's three reads
from recorded replies,
built by step 16 with the pause route and shipped nowhere.
If the project will not sanction that arm,
the fallback is to time those reads alone against the same pool,
and the report then covers the reads' cost rather than the removal's.

The result it expects:
the ratio of the two arms' wall times with its range,
never an absolute time (rule `TVC`).
It sizes no shipped number (L7).
Bound:
300 s per removal,
and the interleaving order,
which the runner records in the log beside the two arms' times.

## Choices this design marks for the project's veto

Each entry names the option this design adopts and the alternative it ranked over,
with a concrete example of each,
the pros and cons a person would meet,
and the ranking with the reason for the adjacent pair.
A veto reopens the entry;
none of them is a decision the records made,
and none is one of the three the project answered on 2026-11-22,
whose options are listed once in "What was not chosen".

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
M3d reaches it by the pause route whichever way this entry is decided.

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

V4 was the record rule,
which the project decided on 2026-11-22;
its alternatives are in "What was not chosen",
and the numbering below is left as it was so old references still find their entry.

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
  The rule separates a command that removes a record from one that destroys a snapshot or a clone:
  only the second is what a refusal never prints,
  and the project's settling command,
  which removes a record after a person confirms,
  is the first kind
  ("A slot whose recorded clone is gone and cannot be proved gone",
  `doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`).
  Whether these refusals print it is for the binding and plan designs first,
  and then for this entry.
- Alternative:
  print the exact `zfs destroy` for the slot.
  Example:
  a hurried reader copies it,
  and the marked snapshot goes with the clone.
  Pro:
  one copy.
  Con:
  printing it reads as authority,
  and the record leaves the finishing of a marked removal to a person rather than to a printed command.
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
  which refuses with `CloneGone` and leaves the settle to the removal's own resume rules,
  which prove the clone gone first (proposal `:963` to `:967`);
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

V11.
Whether the look also runs when a plan exists but records no destroyed tuple.
- Adopted:
  the look runs only when the clone carries no removal intent and no plan exists,
  which is the state the project's answer describes.
  Example:
  a retry after a cancellation that destroyed part of the clone's snapshots,
  the property back at `mount` and the kept plan naming what went;
  the look does not run,
  the retry adopts the kept plan and carries its loss forward,
  and the first check,
  or the head check when no snapshot is left to send,
  refuses with those names.
  Pro:
  form A's sentences are true wherever the look runs,
  no kept plan's history has to be carried into a text that says nothing was destroyed,
  and the project's decided lifetime of a kept plan is kept.
  Con:
  a mark present at such a retry is reported after the retry has unmounted the worktree again,
  not before it takes anything down,
  and where the plan record's head-present arm cannot settle a kept plan's `unknown` tuple
  the plan record has the retry refuse instead (read at `cf51d140af`,
  "The removal intent"),
  which the gate does not change.
- Alternative:
  also look when the property does not read `unmount` and the plan records no destroyed tuple,
  which is the state a cancellation killed before it removed an all-`planned` plan leaves.
  Example:
  a retry over such an all-`planned` plan,
  the property at `mount`;
  the look runs,
  and form A says that this removal has not destroyed anything yet,
  which is true there.
  Pro:
  that retry gets the early refusal too.
  Con:
  it makes two gate rules where the project's answer names one state,
  it needs the plan's outcomes read before step 1's proof,
  and under the plan record's adoption rule the retry rewrites the plan with its own nonce,
  so the look would judge a plan the run is about to rewrite.
Ranking:
  adopted over the alternative,
because the alternative buys an early refusal only for a state a killed cancellation leaves,
where the first check refuses at the same place,
at the price of a gate that reads the plan's outcomes first.

## What the project decided

The three questions this design carried were put to the project and answered on 2026-11-22,
recorded in
`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`.
Each answer is built into the section named beside it,
and the options the project did not take are listed once,
after the three.

### What a refusal at the first check leaves

The project chose "A:
stay pending (Recommended)":
"the removal is kept pending,
the clone stays unmounted,
and the worktree is out of use until a person deals with the mark and runs the removal again,
or runs the cancel command to get the worktree back"
(record,
"What a refusal at the first check leaves").
Built into "What a refusal leaves,
and what the record keeps" for the pending state and the way back,
and into "The look before the unmount" for the state the first check now sees,
which is only the mark that appeared after that look.
The removal does not remount the clone by itself;
the record's own contrast is "a holder goes away by itself and a mark does not",
and the one case a removal remounts is a scan's seen holder cancelling,
which the forced-removal record decides.

### A look before the unmount

The project chose "Yes,
look before unmounting (Recommended)":
"a marked origin is refused there with the worktree still mounted and nothing written.
The state of a refusal at the first decided check then arises only when the mark appears between that look
and the check",
and "This adds a third refusal point to the two the project decided earlier"
(record,
"A look before the unmount").
Built into "The look before the unmount" (where it runs,
its request,
what it compares,
its words,
its cause,
its cost and what step 14 must offer),
"The refusals" (its message),
"What each build step builds" and the controls and the guest arm.
The one thing the design adds to the record is what it costs:
"that the look costs no extra read is an inference until the step that makes the lineage proof is built",
as the record says,
and the section names what step 14 must offer for the inference to hold.

### What a refusal at the second check leaves

The project chose "A:
stay pending,
keep records (Recommended)":
"the removal is kept pending with its record of which snapshots were destroyed,
the clone stays unmounted,
and the cancel command is the way back"
(record,
"What a refusal at the second check leaves").
Built into "What a refusal leaves,
and what the record keeps",
the second check's message and the history sentence,
which names every destroyed tuple the plan records.

### What was not chosen

The record's own list,
each with the reason this design does not carry it:

- Cancelling and remounting at the first check.
  It would spend the decided remount exception on a refusal,
  and the same removal refuses again while the mark stands.
- Keeping exactly the two decided checks.
  The project added the look,
  so a mark present when the person runs the removal is now refused before the removal unmounts anything,
  which is the record's "with the worktree still mounted and nothing written".
- Cancelling and remounting with the record kept,
  and cancelling with the record dropped,
  at the second check.
  The first spends the same exception and adds the remount race;
  the second erases the only statement that history was destroyed.

### What the record does not decide

"Whether the wrapper may ever finish a removal over a marked origin was not asked:
the decided refusal stands"
(record,
"What this record does not decide").
So the design keeps the refusal,
names the person's two completions and,
at the two checks,
the cancel command,
and V7 carries whether a refusal may print a destructive command.

## Dependencies on sibling designs

Each of these is an assumption this design made so it could be whole;
a later pass reconciles the two.
A sibling's document is unreviewed,
so its words are quoted and never adopted as decided.
Each was read with `git show <branch>:<doc>` at the tip named here,
and a tip can move after this revision:
the quoted text is the text at that commit.
The tips read this round are the ones the brief named,
except the propagation design,
whose branch moved after that tip and is read where the brief put it.
One ordering difference is worth naming once:
the plan record's step 5 and the origin-cleanup list's step 5 put the deferred-origin check before the fresh
all-namespace verification and the reproof,
while this design and its control 17 put the check after them,
following the proposal's "Reprove lineage and candidate identities immediately before submitting" (`:896` to
`:899`).
The two sibling lists may mean a set rather than a sequence;
nothing here reads them otherwise,
and this design's order is the proposal's.
A second difference is the same kind:
the plan record's head-present path puts the second deferred-origin check at its step 7 and the re-enumeration at
step 8,
while this design and control 18 put the check after the enumeration;
this design follows the proposal's "Enumerate the clone's snapshots again,
then send the head destroy" (`:926`),
and the two may again be a set rather than a sequence.

- "Where a removal's persisted destruction plan lives,
  and its format"
  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
  read at `3cd520b6a0`):
  it places every field A1 needs,
  including `originDatasetGuid`,
  and says those are "the identities the deferred-destroy check compares with live";
  that agrees with this design's third read (V9).
  It writes the plan's first record "after the unmount and the proof" (`:1246` to `:1248`),
  and removes it only after a completed removal's settle;
  that agrees with A2,
  and with the pending state the project decided a refusal leaves
  ("What a refusal leaves,
  and what the record keeps").
  It writes `requested` before each request and `head=requested` before the head destroy,
  which is why the checks run before those rewrites.
  It places the look where this design does:
  its step 1 is "the claim and the proof:
  no plan is written",
  "and the early look for the origin's deferred-destroy mark is made inside that proof,
  so a mark found there refuses with the worktree still mounted and nothing written" (`:1151` to `:1158`).
  Its head-gone path has "no lineage proof ... and no `unmount` intent ... because the intent is a property of the
  clone and went with the head" (`:1646` to `:1652`),
  and it says "A `head=gone` plan has no live intent by construction" and is never refused for that reason
  (`:1664` to `:1667`),
  while "A plan with no live intent while the head still stands is refused by the removal" (`:2158`).
  An intent reading `mount` beside a standing head is "the reattach of a removal that cancelled itself,
  so a plan beside it is adopted on the head-present path under the nonce rule above,
  and every tuple and outcome it holds is kept" (`:2160` to `:2165`),
  and "A retry after a cancel therefore always presents the kept plan:
  it adopts it under its own nonce,
  keeps every tuple and outcome,
  and proceeds to the first deferred check;
  the look does not run on it" (`:2166` to `:2170`).
  That answers the requirement this design put to it last round,
  and it agrees with the seven answers' lifetime of a kept plan
  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
  "How long the plan a cancellation keeps stays").
  Its finish pass acts only on a plan whose head is `gone`
  (`:1907` to `:1909`,
  in the section at `:1882`),
  and its worked example of the head-gone path is a plan whose head is `requested` (`:1692` to `:1696`),
  while its head-present path refuses a record that disagrees with the live clone (`:1598` to `:1608`).
  The plan design (unapproved;
  read at `cf51d140af`,
  "Which resume path a removal takes") reconciles the two.
  A plan whose head word is `requested` or `unknown` beside a gone clone
  takes the head-gone path with the accepted proof (proposal `:963` to `:967`);
  a plan whose head word is `present` or `refused` is an outside destroy,
  and the removal refuses and names the settling command;
  a name carrying another GUID is refused with the reuse named and no command;
  and a pool that cannot prove the absence is refused,
  naming the settling command for every head word but `gone`.
  Where the accepted proof cannot be made the project answered with
  "Print an explicit command that removes the record after you confirm",
  and that command's name,
  guard and proof are open.
  Nothing here contradicts an accepted sentence:
  the nearest,
  proposal `:460` to `:461`,
  is about a registration Git no longer holds.
- "What a cancelled,
  timed-out or killed removal leaves,
  and what the cancel command proves and does"
  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
  read at `eb7a78eccc`):
  its A9 says "The deferred-destroy design keeps the intent and the plan on every refusal,
  and rewrites neither" (`:763` to `:765`),
  and its points 13 and 16 are the two refusal states with that assumption.
  It also says to "Run `cancel-worktree-removal <slot>` after a deferred-destroy refusal",
  that the refusal "keeps the intent and the plan",
  and that "the command's proof accepts it:
  it rewrites the intent,
  attaches,
  and reports that nothing was destroyed or which tuples were" (`:1951` to `:1958`).
  That is what form B's cancel sentence promises for a removal that is not forced.
  That agrees with what the project decided on 2026-11-22:
  the intent and the plan are kept at both checks
  (`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`,
  "What a refusal at the first check leaves" and "What a refusal at the second check leaves").
  Its own end-state list "has no cancelled value,
  and this design needs none" (`:3443` to `:3447`);
  it answers this design's third option by name (`:1670` to `:1676`):
  the state that option asked for "is the intent plus the kept plan,
  and the rule is the cancellation this design writes".
  It also says "A cancellation writes nothing to [the plan]" (`:1643` to `:1645`),
  and it "does not decide whether such a refusal should cancel instead" (`:3455` to `:3459`).
  Its own note that this design's retry requirement "is answered there in the words it asked for" (`:3445` to
  `:3447`) points at the earlier request,
  "a retry after a cancel always presents a plan",
  which this design replaced with the conditional paragraph of "How a person finishes",
  because a retry after a cancel that destroyed nothing presents no plan;
  the answer it refers to is the plan record's,
  and this design quotes it there.
  So the option the project did not take,
  cancelling and keeping the plan,
  would need no new plan state;
  its costs are the remount it spends,
  the remount race and the surviving verified binding
  ("What was not chosen").
  One difference between it and the seven answers is that design's to reconcile:
  it says "A later removal of the same clone rewrites the plan under the same key,
  and that is the natural end of the loss record's life" (`:1648` to `:1649`),
  which reads as replacing the old record,
  where the answers and the plan record say the next removal "reads it,
  proves it,
  carries the loss forward and ends it".
  This design needs only what both give:
  the retry's first refusal names the carried loss,
  which the history sentence does,
  and the look does not run on a retry over a kept plan.
- "Where the add's durable binding record lives,
  and its format"
  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
  read at `07858070bb`):
  at this tip it names the checks only in the order list it cites from the origin-cleanup design,
  and says nothing about them beyond that.
  It designs the settling command the seven answers decided,
  and it gives its name in its own veto entry V21:
  `git monochromatic-git-policy-cli settle-worktree-clone-record <slot> --confirm-removal-of <slot>
  --confirm-clone-guid <the last four decimal digits of cloneGuid=>`
  ("The settling command",
  `:3216` to `:3220`,
  and V21 at `:4804` to `:4809`).
  That name is adopted,
  so a refusal's text here may print it,
  and this design cites it only where "How a person finishes" says the command exists.
  The one gap this design shares with the plan record is the path selection that entry narrows,
  and the gap's decided last resort is that command.
  The earlier version this design quoted
  ("reads the origin identity this design records,
  and gives it no new field")
  is no longer in its text,
  so the fallback A1 names rests on the binding's own fields as the proposal lists them (proposal `:414` to `:421`),
  not on a sibling statement.
- "The record that carries origin cleanup after a registration is settled"
  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
  read at `8cc31f59a4`):
  it assumes "The removal's step 15 check of the origin's deferred-destroy mark is available to this design's
  attempt as a read" (`:892` to `:894`).
  This design exports `origin_mark` for exactly that,
  and the sibling may use it.
  Its own order now agrees with this design's three points:
  its step 5 makes the first check,
  its step 6 publishes the origin cleanup record "before it submits the first destroy request of this invocation"
  (`:1154` to `:1159`),
  and its step 8 is the second check (`:1166` to `:1172`).
  That sibling also says the mark can be cleared "when an administrator removes the mark" (`:1872`),
  which F8 contradicts at `zfs-2.4.4`;
  the source decides,
  and the correction is that sibling's.
- "The forced arm of a removal:
  which holders are ended,
  by whom,
  and how the caller's session is spared"
  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
  read at `aa1233b331`):
  its D3 says "The deferred-destroy checks own step 16's checks and call sites.
  This design claims no ownership of them;
  it needs the scan file's reader and V2's rule,
  and the build place is the first consumer's choice" (`:2778` to `:2782`),
  which is the whole of what it says about the checks.
  Both entries agree with this design:
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
  read at `4dadc5fd27`):
  its fallback records a dependency instead of an assumption:
  "no sibling design gives an add a callable unmount-and-destroy;
  ... the four removal designs are written for removals",
  and "the settlement is marked as depending on a callable unmount-and-destroy no sibling offers".
  This design's part of that answer is interface,
  not permission:
  `check_planned_origin` takes the plan's tuple,
  the three reads and the stage,
  so any caller that holds a plan gets the same check.
  Whether an add may call the removal's unmount and destruction is the removal's and the add's design to settle,
  which this design neither grants nor withholds.
  What it says about an add's own destroys,
  in words that design can quote:
  the checks do not run for them,
  because the look belongs to the removal's proof and both checks read a removal's plan,
  intent and stage,
  none of which an add writes;
  a same-account mark on the snapshot an add made is reaped with the add's clone when that clone is its last child
  and no hold pins it,
  so the add meets one `ENOENT` and treats it as done;
  a marked add snapshot that another clone or a hold pins answers `EEXIST` or `EBUSY` and stands;
  and a refusal is what a caller meets only when it gives the check a plan.
  At this tip that design agrees:
  it says the add "writes no destruction plan,
  no removal intent and no stage,
  and it calls none of that design's checks" (`:1648` to `:1652`),
  that it "treats `ENOENT` as done" (`:1645`),
  and that "Any other answer to either destroy is a failed settle" (`:1654`).
  The earlier reading that the check "might refuse the origin snapshot an add made" is gone from its text.
- "The add-time Git lock of a ZFS clone worktree"
  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
  read at `7d0dcdc875`):
  of the destruction steps it assumes "A removal's destruction proof can say "the clone is gone" before the Git
  settle runs,
  as [plan] step 16 states it" (its AS3 at `:1307` to `:1309`;
  the inner words are the sibling's own quoting),
  and it places the lock's release in plan step 16's settle.
  It names the checks in two lists and says nothing about them beyond that.
  A refused check never reaches that settle,
  so the lock it protects stands,
  which follows from the sibling's own release rule.
- "Which triggers a lost last event waits for,
  and how nudges are merged"
  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
  read at `b6855dce15`):
  it lists the deferred-destroy checks as "no mention" (`:4244`),
  and this design depends on none of it.
- "What re-applies private propagation after a replacement mount of the checkout"
  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
  read at `c7708953c0`):
  it lists the deferred-destroy checks as "no dependency" (`:3087`),
  and that agrees with this design:
  the checks read dataset names and properties,
  never a mount fact,
  so a replacement mount changes nothing they read.
  Its branch had moved past that tip when this round ended.

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
- The look's "no extra read" is an inference until step 14 is built
  (`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`,
  "A look before the unmount").
  It holds only if that proof reads the origin snapshot with a reply that keeps the property list
  ("The look before the unmount"),
  and the cost otherwise is one `ZFS_IOC_OBJSET_STATS` per removal.

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
- The add's own fallback:
  its order,
  its reports and how it treats the one `ENOENT` a mark can cause.
  This design says only that its destroys do not pass through these checks,
  and what the checks answer for a clone the add made ("An add's own destroy does not pass through these
  checks").
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
the opening (the second pass named beside the gate review),
"What this design reuses" (the wrapper-as-git fixture the probes run),
F19 and F24 (their citations and the package named),
"Reading the mark" (the pruning sentence's own citation),
"The item and the sentences that bind it" (the man page named as the source of the marked-snapshot sentence),
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
Q1's Option C cost and the residual's two bullets,
which name the head check where chunks make "the second check" ambiguous.
The Option C cost then read the cancelled design's "no cancelled value" as if that sibling left the state undesigned;
the diff check showed that sibling answers the objection by name,
and this revision restates the cost as the remount it spends,
the remount race and the surviving verified binding.

Rewritten after the diff check,
and nothing else:
"Measurements owed" (M0b,
M3a to M3d and M4 restated as specifications of the state,
the one action,
the expectation with the step that produces it,
and the bound,
each naming the step that writes and first runs its script;
the checkout,
its `.git`,
`.claude/worktrees` and the account's run folder given to the account that runs the add;
the installed handler,
ZED,
folders and readiness named as the add's preconditions;
the pause path under the account's run folder;
the mount assertion changed to `findmnt --mountpoint`;
OpenZFS's guest `PATH` carried into the header;
M0a's snapshot citation corrected;
the round's teardown added),
"The refusals" (the template and both messages name the unmounted clone,
claim only what the check read and the plan records,
and promise only that a later run is meant to finish the removal;
the gone-object choice says all three),
"How a person finishes" (the pointer and the finish pass dropped,
and completion stated as a requirement on the plan record),
V10's lead-in,
Q1's Option C cost and the reason A is ranked over it,
QA-2 marked as held rather than adopted,
Q2 reduced to a note,
control 11's review label,
control 29's owner,
"What each build step builds" (which step writes which probe's script),
"Sources read" and "Revisions".
Two statements the diff check found sound changed because a fix forced it:
the messages' unmounted sentence came back from the first pass,
and "How a person finishes" lost the finish-pass claim that rested on the pointer file.

Rewritten after the project's three answers of 2026-11-22,
and the two records that came with them:
the opening (the answers cited and the three decisions added to the read list),
"The item and the sentences that bind it" (the records' disagreement closed by the record),
"The order the three refusal points sit in" (the look inserted and the three states named),
"The look before the unmount" (new:
where it runs,
which request,
what it compares,
its words,
its cause,
its cost and what step 14 must offer),
"One check,
called at every snapshot request and before the head destroy" (told apart from the look),
"The refusals" (two template forms,
the look's cause,
three full messages),
"What a refusal leaves,
and what the record keeps" (decided,
with the record's words and the forced record's one remount case),
"An add's own destroy does not pass through these checks" (new),
"How a person finishes" (the look's extra way out and the record's undecided note),
"What each build step builds" (the look's placement in step 14 and the arms renumbered),
the controls (two for the look and the guest ones renumbered),
race R6,
"Measurements owed" (M3a is the look's arm,
M3b the first check's,
M3c to M3e the others,
and M4's two arms),
"Choices this design marks for the project's veto" (V4 removed,
V1's arm name and V7's note),
"Questions only the project can answer" replaced by "What the project decided" (three decided sections,
the record's not-chosen list and what the record leaves undecided),
"Dependencies on sibling designs" (the cancelled entry's use of V4 and the option the project did not take),
"Limitations" and "What this design does not cover",
"Sources read" and "Revisions".
The look's design,
its refusal,
its control and its guest arm are this document's own,
beyond the record's sentence.

Rewritten after the review of the answers round:
the opening (the fourth review),
"The order the three refusal points sit in" (the look runs once,
in step 1's proof of a first run,
and step 4's reproof does not look),
"The look before the unmount" (the proof citation,
where it does not look and why,
the mount claim withdrawn from its message,
and the per-slot claim named),
"The refusals" (three forms and the cause each takes,
four full messages,
the cancel sentence scoped to an unforced removal,
no design words in person text,
and the worktree's files named),
"What a refusal leaves,
and what the record keeps" (what the look's refusal writes,
the holder family and the busy-unmount case),
"An add's own destroy does not pass through these checks" (a pinned marked snapshot),
"How a person finishes a refused removal" (the accepted sentence for a gone clone and origin,
the standing-origin case put to the plan and binding designs,
and the look's state),
"What each build step builds" (step 14 leaves the hook and step 15 places the look in it,
and the pause's unsized bound named),
the controls (the look's two moved into the host list,
four for the later proofs and the texts,
and the guest ones renumbered),
"Measurements owed" (the guest `PATH` citation and M1's `env`,
M0b's other preconditions,
M3a as a first run only,
M3f the rerun,
and M4's bound),
"Choices this design marks for the project's veto" (the V4 note),
"Dependencies on sibling designs" (four tips read again,
the plan record's current lines,
the add's destroys in words the dirty-main design can quote,
and the request for a standing-origin settle),
"Sources read" and "Revisions".

Rewritten after the diff check of the answers round:
the opening (the fifth review),
"The order the three refusal points sit in" (the gate defined,
and the two checks named where no snapshot is left),
"The look before the unmount" (the gate's wording,
the retry after a cancel,
and the head check where no snapshot is left),
"The refusals" (the person's text promises less at both completions,
names the unmount and a holder,
and its `CloneGone` example names the recorded tuple and the settle rule),
"How a person finishes a refused removal" (the accepted "Settle + own record" decision with its two limits,
`:460` to `:461` narrowed to what it covers,
the one gap put to the plan and binding designs,
and the retry after a cancel),
"What each build step builds" (the hook's input,
the controls each step writes,
and the request to the implementation plan),
the controls (21,
24 and 25 corrected,
26 and 27 added,
and the guest ones renumbered 28 and 29),
"Measurements owed" (M3a's claim line and M3f's expectation and pass line),
"Choices this design marks for the project's veto" (V11),
"Dependencies on sibling designs" (nine tips re-read,
the plan record's own look placement,
the dirty-main agreement,
the one collapsed gap,
and the retry requirement),
"Sources read" and "Revisions".

Rewritten after the check before acceptance,
and for the seven answers and the two later records:
the opening (the sixth review,
and the seven answers in the read list),
"How a person finishes a refused removal" (the duplicated block removed,
the cancel bullet and the retry paragraph rebuilt on the seven answers and the plan record's answer,
and the gap narrowed to path selection with the accepted absence proof and the settling command as its last
resort),
"The look before the unmount" (a retry looks only when the cancellation removed its plan),
the history sentence,
the four messages and control 11 (this clone's earlier removals named),
control 27 (the plan record's reading,
with the dropped-plan variant),
"Where the code lives" and the add section (`origin_mark` read by the five worktree commands),
"The refusals" (`PlanIncomplete` and the unreadable file,
and every attempt printing its refusal),
"Measurements owed" (M3g,
the retry arm),
"Choices this design marks for the project's veto" (V7's record-removing clause and V11 rebuilt),
"Dependencies on sibling designs" (the plan record's retry answer and settling section,
the cancelled design's difference,
the binding design's settling command as a proposal,
the lock design's AS3 label,
and the second ordering difference),
"Sources read" and "Revisions".

Rewritten after the last diff read:
the opening (the fifth and sixth reviews named,
and the count corrected),
"The order the three refusal points sit in" (the gate's definition no longer says the opposite),
"The look before the unmount" (the no-plan states a retry can leave),
"How a person finishes a refused removal" (the retry's `unknown` exception,
the no-plan states named,
and the plan design's text called unapproved and reconciled against the accepted rule),
V11 (its adopted Con and the exception),
M3g (the `unknown`-free state and the step that first runs it),
control 27 (the killed-cancellation and `unknown` variants),
"Dependencies on sibling designs" (the plan design's disagreement,
the cancelled design's dangling pointer,
and the second limit reworded),
"Sources read" and "Revisions".

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

This repository at `36ad1de937`,
the merge that brought the seven answers and the dirty-main record's three further answers;
the sibling designs were read at the tips "Dependencies on sibling designs" names,
and this document's own earlier tip was `946578a8e5`
(its first version was read at `5dd8883a60`;
the only change to a file cited here between `3e34817184` and the earlier merge is one line of `SPEC.md` at
`:7789`,
which is after every line cited from it):
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
  with the changes the first part of "What this revision rewrote" lists.
- `3e34817184`,
  committed `2026-11-22T18:11:19-05:00`:
  the metadata-only commit that recorded the two above.
- `8228a9efdb`,
  committed `2026-11-22T18:39:07-05:00`:
  the version that answers the second pass,
  with the changes the second part of "What this revision rewrote" lists.
- `2c59ed3329`,
  committed `2026-11-22T18:39:13-05:00`:
  the metadata-only commit that recorded `8228a9efdb`'s time.
- `139061f746`,
  committed `2026-11-22T18:40:22-05:00`:
  the three sibling tips re-read and their statements corrected.
- `415022b47c`,
  committed `2026-11-22T18:40:33-05:00`:
  the metadata-only commit that restated this list with every older commit's time.
- `0f394ae4c0`,
  committed `2026-11-22T18:40:45-05:00`:
  the list completed with every section the second pass turned over.
- `f82f20ecab`,
  committed `2026-11-22T18:40:52-05:00`:
  the commit whose subject begins "docs(planning):
  close the revision list",
  which wrote the entry above it as "the commit that writes this line".
- `6a479f16a7`,
  committed `2026-11-22T19:02:32-05:00`:
  the version that answers the diff check,
  with the changes the third part of "What this revision rewrote" lists.
- `9ee117c04d`,
  committed `2026-11-22T19:02:49-05:00`:
  the metadata-only commit that listed the two above with their times.
- `ff946dcae2`,
  committed `2026-11-22T19:02:57-05:00`:
  the metadata-only commit that recorded `9ee117c04d`'s time,
  and the entry above it was "the commit that writes this line" until this round.
- `fda838910a`,
  committed `2026-11-22T19:16:37-05:00`:
  the merge of `main` that brought in the project's three answers and the two sibling records.
- `7bf0bbb76e`,
  committed `2026-11-22T19:21:28-05:00`:
  the version that writes the design to the three answers,
  with the changes the fourth part of "What this revision rewrote" lists.
- `0e816c603b`,
  committed `2026-11-22T19:21:43-05:00`:
  the metadata-only commit that listed the three above with their times.
- `1f7a0800fa`,
  committed `2026-11-22T19:21:49-05:00`:
  the metadata-only commit that recorded `0e816c603b`'s time,
  and the entry above it was "the commit that writes this line" until this round.
- `128da19117`,
  committed `2026-11-22T19:59:45-05:00`:
  the version that answers the review of the answers round,
  with the changes the fifth part of "What this revision rewrote" lists.
- `dfc6e64739`,
  committed `2026-11-22T19:59:58-05:00`:
  the metadata-only commit that listed the two above with their times.
- `31e65839dd`,
  committed `2026-11-22T20:00:07-05:00`:
  the metadata-only commit that recorded `dfc6e64739`'s time,
  and the entry above it was "the commit that writes this line" until this commit.
- `1800f0cb51`,
  committed `2026-11-22T20:00:54-05:00`:
  the commit that cited the reversal rule the refused causes rest on,
  and the entry above it was "the commit that writes this line" until this commit.
- `fd00d54dd2`,
  committed `2026-11-22T20:49:03-05:00`:
  the version that answers the diff check of the answers round,
  with the changes the sixth part of "What this revision rewrote" lists.
- `57a9b59e07`,
  committed `2026-11-22T20:49:09-05:00`:
  the metadata-only commit that listed the two above with their times.
- `946578a8e5`,
  committed `2026-11-22T20:49:15-05:00`:
  the metadata-only commit that recorded `57a9b59e07`'s time,
  and the entry above it was "the commit that writes this line" until this commit.
- `36ad1de937`,
  committed `2026-11-22T21:39:44-05:00`:
  the merge of `main` that brought in the seven answers and the dirty-main record's three further answers.
- `1d38fefc51`,
  committed `2026-11-22T21:44:08-05:00`:
  the version that answers the check before acceptance and the seven answers,
  with the changes the seventh part of "What this revision rewrote" lists.
- `8f38000259`,
  committed `2026-11-22T21:44:14-05:00`:
  the metadata-only commit that listed the three above with their times.
- `d0ff7604b7`,
  committed `2026-11-22T21:44:21-05:00`:
  the metadata-only commit that recorded `8f38000259`'s time,
  and the entry above it was "the commit that writes this line" until this commit.
- `bbcd945c81`,
  committed `2026-11-22T21:59:46-05:00`:
  the wording fixes the last diff read listed,
  with the changes the eighth part of "What this revision rewrote" lists.
- `2ebfa3f513`,
  committed `2026-11-22T21:59:52-05:00`:
  the metadata-only commit that listed the two above with their times.
- The commit that writes this line,
  subject beginning "docs(planning):
  record the wording round's metadata commit":
  its own committer time can only be read from `git log --format=%cI` for it,
  because a document cannot name the time of the commit that writes it.

[vm-12]: cli-git-zfs-measurements-owed-before-acceptance.md
[vm-17]: cli-git-zfs-measurements-owed-before-acceptance.md
