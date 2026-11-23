# The add-time Git lock of a ZFS clone worktree

Non-final,
unapproved design proposal by an agent.
It is not approved by the project owner and nothing in it is final.
It designs the open item "The add-time Git lock" of the accepted ZFS worktree design
([decision][decision],
"Decision"),
and it makes no decision that design,
the proposal,
or the project's answers made,
beyond the writer's choices that "Choices this design marks for the project's veto" lists
and the questions that "Questions only the project can answer" puts.
It ran no ZFS command that changes anything,
started no virtual machine and no guest,
touched no pool,
mounted nothing,
opened no session transcript,
and edited no file but this document.
The one container it ran was an already-present gate image of this repository,
to run Git alone on throwaway repositories inside it,
with no folder of the home mounted;
no image was built or removed.

This revision answers the gate review of 2026-11-22T18:05:06-05:00
(`~/temp/agent/gate-review-zfs-add-time-git-lock-and-binding-record-designs-2026-11-22/keep/review.md`),
whose findings are answered one by one in its terms:
the lock's proof now derives its bytes from the binding record's `nonce=`,
this design keeps no record of its own,
the failure path and the states are rewritten from the records that exist in each state,
and every citation was rechecked against its file.
It read no guest.
Every kernel,
ZFS or mount fact it rests on is one the repository already measured,
except where a sentence says it is owed or unmeasured;
no mount fact is used as measured,
because none was.
Every Git fact is read at upstream `v2.56.1`,
the version this host runs (`git version 2.56.1`),
in the files listed under "Sources read",
or measured in throwaway repositories,
as "What was run" records.
No OpenZFS source was needed:
the design touches Git's administrative directory and the ZFS removal order,
not the ZFS request interface.

Plain `git` on this machine is this repository's wrapper,
`monochromatic-git-policy-cli 0.6.1-alpha.44`,
and the wrapper adds its own steps to `git worktree add` and `git worktree remove`
("What the wrapper adds to `git worktree add` and `git worktree remove`").
So the probes ran twice:
once on this host through the wrapper,
and once inside an already-present gate image,
where `git` is Git itself at `2.56.1`.
Where Git's own source at `v2.56.1` and a run disagree,
the design says so;
where they agree,
the run is recorded as the check.
The wrapper build this session ran was built on 2026-11-17,
before any ZFS step merged,
so its `worktree add` is the route before the ZFS one,
and no probe exercises the ZFS add this design specifies.

## What was read

The records this design answers to,
read whole where the brief says so:

- [`doc/decision/cli-git-zfs-worktree-design.md`][decision] (69 lines),
  the acceptance,
  with its open items still open.
- [`doc/planning/cli-git-zfs-worktree-design-proposal.md`][proposal],
  its "Add and Git registration",
  "Unmounted,
  pruned or moved worktrees",
  "Removal and recovery",
  "What doctor checks and prints",
  "Project decisions" and "Open items the decisions leave".
- [`doc/planning/cli-git-zfs-worktree-implementation-plan.md`][plan],
  its steps 10,
  12,
  13 and 16,
  "Open items and the step that needs each",
  "Gaps the design's open-item list does not name",
  "Where the design is silent:
  this plan's suggestions" and "Where the tests run".
- [`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`][measurements],
  item 6 whole and the sections it refers to.
- [`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`][fallback],
  113 lines,
  recorded 2026-11-22T19:11:12-05:00,
  whole:
  the fallback,
  the proof's window and what the slot holds while it is filled.
- [`doc/decision/cli-git-zfs-add-time-git-lock-put-back-when-lost.md`][lostlock],
  58 lines,
  recorded 2026-11-22T20:21:19-05:00,
  whole:
  the project's answer on a lost lock.
- [`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`][sevenanswers],
  138 lines,
  recorded 2026-11-22T21:16:30-05:00,
  whole,
  for the two answers this design leans on:
  a record that cannot be read is refused and named,
  and a slot whose recorded clone is gone and cannot be proved gone gets a settling command.
- [`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`][dirtymain],
  accepted and on `main`,
  its "What happens when the proof fails or the restore cannot finish" and
  "Which build step builds which part",
  for the settle marker and the destroys the fallback runs.
- `doc/planning/cli-git-zfs-add-binding-record-design.md` on branch
  `docs/planning-cli-git-zfs-add-binding-record-design`,
  read by `git show` at `3b4b073f0e` (2026-11-23T03:55:16-05:00):
  "The add's writes in one order",
  "A resume after a kill between Git's add and the registered write",
  "The account's claim and root's claim",
  "The binding",
  "What the writer confirms before each write" and
  "Which step of the plan builds which part".
  Its order is cited by its step labels at that commit;
  nothing here restates it.
- [`doc/decision/cli-git-zfs-staging-under-cache.md`][staging] (526 lines),
  for the threat model it records and the way it argues under it.
- [`doc/decision/cli-git-zfs-step-8-sweep-choices.md`][sweepchoices] (100 lines),
  for the no-configuration answer,
  and its per-slot claim answer.
- [`doc/decision/cli-git-zfs-slot-identity-store-format.md`][slotstore] (179 lines),
  as the precedent for a small record that takes no lock.
- [`package/git-policy/cli/SPEC.md`][spec],
  "Doctor",
  "The per-slot claim",
  the removal route and its accepted risk,
  the add's own records,
  the descriptor mount of a ZFS worktree clone,
  the shadow section that names Git's automatic maintenance,
  and the credential rule.
- The wrapper's own code under `package/git-policy/cli/src/native/`:
  `lock/single_owner_claim.rs`,
  `worktree/worktree_add_binding.rs`,
  `worktree/worktree_added_by_the_wrapper.rs`,
  `worktree/worktree_registration_proof.rs`,
  `worktree/worktree_removal_second_look.rs`,
  `worktree_clone_mount/clone_slot_git_pointer.rs`,
  `foundation/private_storage.rs`,
  `invocation/real_git_candidate.rs` and
  `health/doctor.rs`.
- The sibling design this one depends on,
  [`doc/planning/cli-git-zfs-add-binding-record-design.md`][binding] on its branch,
  read by `git show`,
  for its claim,
  its binding's fields,
  its phases and its order;
  and,
  for what each of the other siblings places and expects,
  the destruction plan,
  the origin cleanup,
  the cancelled or killed removal,
  the forced arm and the deferred-destroy checks,
  each read by `git show` of its branch.
  A sibling's document is unreviewed here:
  nothing in it is adopted as decided,
  and "Where a sibling and this design disagree" says where they differ.

## What was run

### Through the wrapper, on this host

Plain `git` (the installed wrapper,
`0.6.1-alpha.44`,
shadowing `/usr/bin/git`,
Git `2.56.1`,
built 2026-11-17),
in throwaway repositories under
`/home/user/temp/agent/zfs-add-time-git-lock-design-2026-11-22/scratch/probe2` to `probe8`,
each with its own `HOME` and `XDG_RUNTIME_DIR` under that scratch folder,
each script in its own capped `systemd-run --user --scope`,
and the fixtures' one commit made through plumbing
(`hash-object`,
`mktree`,
`commit-tree`,
`update-ref`)
because the wrapper's commit route needs this session's commit service,
which a private `XDG_RUNTIME_DIR` cannot reach.
Every `git worktree add` in these runs therefore shows the wrapper on top of Git:
it copied ignored entries (there were none),
wrote `monochromatic-git-policy-cli-copy-owner` and `monochromatic-git-policy-cli-worktree-added`
beside the registration (F13),
and took its own add path with its own checks.
The wrapper's cheap ladder,
whose accepted forms include `--lock` and `--reason` ([spec][spec],
line 1162),
did not run in these fixtures:
no stage claim or destination record was written under the common directory.
`git worktree move` is forwarded by the wrapper unchanged ([spec][spec],
line 880),
and `git worktree prune`,
`git gc` and `git maintenance` are forwarded too,
so those runs show Git's own answers.
`git worktree remove` is served by the wrapper only for a btrfs subvolume worktree,
and is Git's own for every other case ([spec][spec],
lines 3119 to 3381),
so a plain directory's removal shows Git's answer here as well.

### As Git alone, inside a gate image

`localhost/git-policy-native-test:zfs-step-9-unmount-2026-11-22` was present for the first round
(`podman images` listed it);
by the review's reading its tag is gone now,
and the evidence of that round is the scripts and printed lines kept in
`~/temp/agent/zfs-add-time-git-lock-design-2026-11-22/keep/git-2.56.1-lock-probes/`.
This round ran its own Git-alone arms in a tag that is present now,
`localhost/git-policy-native-test:gpc-6346-commit-timing-window-20261122T1809-0500`,
whose image ID is `d681456f37c4fd97a294d00945fed2225ea9c8afe320407f1267e0ab96ba9661`
and whose repo digest is
`sha256:8df3466d3b9f382596cc04c3ec74ee02545970ad5fbe1d63962724384a57176b`;
that tag may be gone in a later round too,
so the digest is recorded here.
The arms ran with `podman run --rm --network none --cpus=2 --memory=2g --pids-limit=256`,
no folder of the home mounted,
on a throwaway repository in a named volume that the probe created and removed,
with the fixture's commit made by `git commit` inside the image.
`git --version` there printed one line,
`git version 2.56.1`,
where this host's prints the wrapper's version first,
so `git` in the image is Git itself.
No image was built and none was removed.
Its arms are G1 to G9 below,
the first revision round's own arms are G10 to G14,
and the second revision round's arms for the leftovers of a kill inside Git's add are G15 to G17,
run in the same image by the ID above.
The second revision round ran no probe:
every Git statement it added or rechecked is source reading at `v2.56.1`.
The third revision round ran none either:
it read the third check's probe output
(`~/temp/agent/gate-review-zfs-add-time-git-lock-and-binding-record-designs-2026-11-22/keep/probe-registration-shapes-output.txt`),
which measured a finished add's entries and the sanitiser's names in the gate image,
and Git's source at `v2.56.1`,
including `refs.c:191` to `:321` and `:376`,
`worktree.c:1105` to `:1134` and `builtin/worktree.c:495` to `:604`.

### The arms, and what each showed

The first round's probes on this host,
and what each showed:

- **P1**,
  the lock an add writes and leaves.
  A plain `git worktree add --no-checkout` leaves no `locked` file.
  `--lock --reason <text>` leaves `<text>` plus one newline,
  mode `0644`,
  and `git worktree list --porcelain` reports `locked <text>`.
- **P2**,
  sampling the administrative directory every millisecond through an add of a 4,000 file checkout.
  A plain add held `locked` with `initializing` for the whole run and ended with no lock;
  a `--lock` add held the given text from the first sample and kept it.
  No sample showed the administrative directory present without its lock.
  This is a regression observation,
  not a test of a window's absence (L3).
- **P3**,
  explicit `git worktree prune` on 2.56.1.
  A locked registration with the slot's `.git` deleted survived;
  unlocked,
  the same registration was pruned at once with
  `gitdir file points to non-existent location`.
  An administrative directory holding only a `locked` file survived and was not listed by
  `git worktree list`;
  one holding nothing was pruned at once.
- **P4**,
  `git gc` on 2.56.1.
  Unlocked,
  pointer missing,
  no index:
  pruned.
  Unlocked,
  pointer missing,
  an index younger than three months:
  kept.
  The same index older than three months:
  pruned.
  Locked,
  pointer missing:
  kept in every arm.
- **P5**,
  `git maintenance run --auto`,
  the task Git runs after each commit it makes.
  Unlocked,
  pointer missing,
  no index:
  pruned.
  Locked:
  kept.
  A healthy unlocked worktree:
  kept.
- **P6**,
  a lock file the probe writes itself,
  with no Git command involved.
  It prevented both prunes and `git gc`,
  `git worktree lock` refused over it and repeated its words,
  and a torn lock and an empty lock both still protected.
- **P7**,
  `git worktree move` and `git worktree remove` on a locked worktree with its pointer present.
  Both refused plain and with one `--force`,
  both succeeded with two forces,
  and the double-force removal deleted the slot directory and the registration.
- **P8**,
  double force against a locked registration whose slot existed but held no `.git`.
  Even two forces failed:
  `fatal: validation failed, cannot remove working tree: '<slot>/.git' does not exist`.
- **P9**,
  `git worktree repair` with the pointer missing and the administrative directory present.
  It rewrote the pointer from the `gitdir` file and left the lock as it was.
  With the administrative directory pruned it restored nothing,
  and `git status` in a repaired but empty slot reported every tracked file deleted.
- **P10**,
  a post-checkout hook written into a fixture repository.
  It did not run in that probe.
  Why it did not is not established here:
  the wrapper runs its own hooks path for the Git it starts,
  and whether that path is what kept this hook from running is unmeasured,
  so the design does not rest on it.

The first round's Git-alone arms,
and what each showed:

- **G1**:
  `--lock --reason` writes the words plus one newline,
  mode `0644`,
  into a `--no-checkout` registration
  with no index,
  and `worktree list --porcelain` reports `locked <words>`;
  a plain add leaves no lock.
- **G2**:
  explicit prune keeps the locked registration with the pointer deleted,
  and removes it at once after
  `git worktree unlock`.
- **G3**:
  `git gc` keeps it locked,
  and removes it unlocked,
  with no index.
- **G4**:
  `git maintenance run --auto` does the same.
- **G5**:
  `git worktree remove` refuses plain and with one force,
  and two forces delete the registration and the
  slot directory.
- **G6**:
  with the slot present and its pointer missing,
  two forces fail validation.
- **G7**:
  `git worktree repair` rewrites the pointer from the `gitdir` file and leaves the lock.
- **G8**:
  `git worktree lock` with no reason writes a zero-byte file that still protects.
- **G9**:
  a lock file written by `sh`,
  with no Git command involved,
  protects exactly as Git's does,
  `git worktree lock` refuses over it and repeats its words,
  and `git gc` keeps it.

This round's Git-alone arms,
run in the image named above:

- **G10**:
  a locked registration whose slot directory is gone entirely.
  `git worktree remove -f -f <slot>` exited 0 and deleted the registration,
  because a missing worktree path passes validation (`WT_VALIDATE_WORKTREE_MISSING_OK`,
  `builtin/worktree.c:1458`).
- **G11**:
  the same state with one force.
  It refused:
  `cannot remove a locked working tree, lock reason: held`.
- **G12**:
  the slot present and empty,
  the registration locked.
  Two forces failed validation on the missing `.git`,
  exactly as G6.
- **G13**:
  `--lock --reason` into a pre-existing empty slot exits 0 and leaves the lock.
- **G14**:
  a control `--lock` add leaves `locked` holding the words (repeated in this round's image).

This round's arms for the leftovers of a kill inside Git's add,
run in the same image by ID:

- **G15**:
  an administrative directory holding only a `locked` file,
  with no `gitdir`,
  is kept by `git worktree prune --verbose` and not shown by `git worktree list --porcelain`.
- **G16**:
  the same leftover at a slot's own name,
  then
  `git worktree add --no-checkout --detach --lock --reason <sentence> <slot>`:
  the add exits 0,
  makes `worktrees/<name>1`,
  and leaves the locked leftover where it was.
- **G17**:
  a registration whose `gitdir` names `<slot>/.git` and whose `locked` holds the sentence,
  with no pointer:
  `git worktree list --porcelain` shows it with a zero `HEAD`,
  and `git worktree add` at that slot dies
  `fatal: '<slot>' is a missing but locked worktree; use 'add -f -f' to override, or 'unlock' and 'prune' or 'remove' to clear`,
  with and without one force.

Every host arm agreed with its Git-alone twin,
with three differences,
all the wrapper's own:
its version line,
its ignored-entry copy message,
and the records it writes beside a registration.
The btrfs removal route's own lock refusal was not exercised.

Nothing was measured in a guest,
on a pool,
or through a mount.
The scripts and their exact printed lines are kept in
`~/temp/agent/zfs-add-time-git-lock-design-2026-11-22/keep/git-2.56.1-lock-probes/`.

## The open item and the answer it carries

The open item,
quoted whole ([proposal][proposal],
"Open items the decisions leave"):

> The add-time Git lock.
> The answer's words:
> "Git lock + pointers (Recommended)":
> a lock is taken at registration
> and kept through unmounts and restarts.
> How the wrapper writes that lock,
> what reason it records,
> and how
> `doctor` and removal tell it from an operator lock without reading the reason text
> ([Unmounted, pruned or moved worktrees](#unmounted-pruned-or-moved-worktrees)) are undesigned,
> and the add
> sequence takes the lock at registration without saying how
> ([Add and Git registration](#add-and-git-registration),
> step 2).

The two cross references inside that quotation name sections of the proposal,
not of this document,
and `[vm-prune]` below names the measurement record's item 6.

The answer it carries,
quoted ([proposal][proposal],
"Prune protection while unmounted"):

> Decided:
> "Git lock + pointers (Recommended)",
> a managed Git lock in addition to preserving the pointer copies.
> The unselected root-set immutable flag would need removal and parent-rename proof and does not protect Git admin
> state.
> [VM item 6][vm-prune] measured the lock with an absent pointer.
> Explicit prune removed the unlocked entry immediately;
> the `git gc` grace period is not a safeguard against explicit prune (Git `v2.43.0`,
> `builtin/worktree.c:244`;
> `git-config(1)`,
> `gc.worktreePruneExpire`).
> Item 6 did not measure `git gc`,
> lock recovery after pointer loss,
> or Git `2.56.1`.

The one sentence about the lock that binds every later step,
quoted ([proposal][proposal],
"Unmounted,
pruned or moved worktrees"):

> A lock protects registration,
> not the pointer itself or an account deliberately deleting Git admin state.

And the sentences the build must satisfy,
quoted from the same section:

> Never treat the reason text alone as ownership or use `--force` to bypass a lock.
> An operator or unexplained lock still refuses removal.
> A verified wrapper lock is released when the registration is settled,
> in the authorized registration-cleanup step,
> and only after clone destruction is proven.

## Records that bind this design

Each of these decides a point this design does not reopen.
Where a record's line numbers moved between the release it measured and Git 2.56.1,
the design gives the 2.56.1 position in "What Git 2.56.1 does".

### Where the lock is taken

Take the lock with the registration,
in the add's own Git invocation
([proposal][proposal],
"Add and Git registration",
step 2,
lines 373 to 382):

> Claim the destination as the account with an exclusive per-add record in the Git common directory.
> Write the proposed clone name and unique `git-worktree-clone-origin-<nonce>` snapshot name before any mutation
> beyond the claim.
> Create or verify an empty account-owned slot,
> then run Git's supported `worktree add --no-checkout` form.
> Record the reciprocal registration and exact `.git` pointer bytes;
> leave it in the slot on the checkout dataset.
> Take the decided managed Git lock here,
> with the registration,
> and retain it through unmounts and restarts

The words "in the add's own Git invocation" are this design's reading of "with the registration",
and the route that reading takes is V1,
a writer's choice for veto.

### What the lock may and may not do

The measured facts,
quoted ([proposal][proposal],
"Unmounted,
pruned or moved worktrees",
lines 439 to 450):

> The account can delete the underlay pointer while the clone is unmounted.
> In [VM item 6][vm-prune],
> Git `2.43.0` pruned the unlocked registration with a missing pointer on explicit
> `git worktree prune`,
> both without `--expire` and with `--expire now`.
> A lock set before pointer loss preserved the registration in both arms.
> Explicit prune without the flag does not wait for a grace period (Git `v2.43.0`,
> `builtin/worktree.c:236` to `:248`,
> `worktree.c:732` to `:733` and `:772` to `:782`).
> By contrast,
> `git gc` defaults to `--expire 3.months.ago` (`git-config(1)`,
> `gc.worktreePruneExpire`),
> so a recent index can keep an unlocked entry with a missing pointer.
> Item 6 did not measure the `git gc` path.
> After pointer restoration,
> the unmounted locked worktree refused ordinary and single-`--force` move and remove;
> double force bypassed both,
> and Git removal left the dataset.
> This supports a lock as a prune guard,
> not as an authority boundary or a replacement for the wrapper's managed
> move refusal.
> Installing or recovering the lock after pointer loss and Git `2.56.1` behavior remain unmeasured.

The measurement record is [`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`][measurements],
"6.
Deleted underlay pointer,
prune and lock",
from line 248:
guest `6.8.0-147-generic`,
OpenZFS `2.4.4-1`,
Git `2.43.0`,
`registration after default prune: False` in the unlocked arm and `True` in the locked one,
and both `move`/`remove` refusals naming the reason.

### What the lock does not replace

The proposal keeps the pointer copies beside the lock,
and the pointer half is plan step 12's
([plan][plan],
step 12):

> The underlay pointer kept for prune,
> independent of the clone's own pointer;
> the read-only distinction between a known unmounted clone and a pending add or removal;
> the report of slot,
> dataset and next safe action;
> the refusal of `git worktree move` for a bound managed clone,
> mounted or not;
> and the repair report that says what a damaged link can and cannot restore.

### When the lock goes

The release belongs to the settle,
and the settle does not wait for the origin
([proposal][proposal],
"Project decisions",
lines 1460 to 1466):

> Decided:
> "Settle + own record",
> against the ranking the question round carried,
> which put keeping the
> registration first:
> the registration is settled once the clone is destroyed,
> and a separate durable record
> carries the origin cleanup.

And "Removal and recovery" step 5,
quoted ([proposal][proposal]):

> Settle only the matching Git registration and pointer after clone destruction is proven.

Plan step 16 is where that settle is built ([plan][plan],
step 16):

> the Git registration and pointer settle after proven clone destruction,
> with the separate origin-cleanup record left standing.

### The fallback, and what the slot holds while it is filled

The decision of 2026-11-22 ([fallback][fallback]),
quoted from "Whether a failed add may finish with Git's own checkout by itself":

> at every step,
> the add destroys only what this add made and recorded
> (the clone,
> then its snapshot),
> runs Git's own checkout at the same slot,
> and says which route made the worktree and why the first was abandoned.

and from "What the slot holds while the add fills it":

> the copied `.git` stays until the last step,
> and the pointer and the copy then change places in one exchanging rename,
> so that no moment and no kill leaves the folder without a `.git`.

The first amends the accepted sentence for this route and makes the fallback the add's own finish,
not a person's recovery.
The second removes the fill window this design's first revision rested the lock on:
the slot holds a `.git` at every moment of the fill,
so prune keeps the registration there whether the lock stands or not.
The lock's reasons are the moments where no `.git` stands (D1 to D2a and the failure path's rows)
and the deleted underlay pointer that the decision of 2026-11-19 names.

### What a failed add leaves

The accepted text,
quoted ([proposal][proposal],
"Add and Git registration",
lines 427 to 431):

> If snapshot or clone returns `ENOSPC`,
> report the operation,
> object and pool,
> keep the claim,
> and clean up only
> objects whose creation and identity are proven and whose destruction is allowed.
> A clone pins its origin even before it is mounted.
> If mutation succeeded but its identity was not recorded before interruption,
> keep and report it
> unless identity can be independently recovered.

and the resume state it leaves reachable ([proposal][proposal],
lines 424 to 425):

> On resume distinguish a claimed empty slot,
> registration without a dataset (including a possible origin only),
> an unmounted clone,
> a mounted but unfinished clone and a verified checkout.

The accepted text does not remove the registration,
and the decision of 2026-11-22 keeps it while the add destroys only the clone and the snapshot it made and finishes with
Git's own checkout in the same slot
(`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
"Whether a failed add may finish with Git's own checkout by itself").
So the registration stands and the lock goes:
"A failed add,
and the fallback when the clone route fails" says who removes
it and by which proof.

### The record whose nonce the lock's proof rests on

The add's durable binding record is the sibling design's,
`doc/planning/cli-git-zfs-add-binding-record-design.md`
on branch `docs/planning-cli-git-zfs-add-binding-record-design`,
read by `git show` at `3b4b073f0e` (2026-11-23T03:55:16-05:00),
its "The record family",
"The binding" and "The add's writes in one order".
That document is the sibling's design and nothing in it is adopted here as decided.

Its first write is the binding's step A2,
made before Git runs,
and it carries the request nonce,
32 lowercase hexadecimal digits,
before anything has mutated (binding design,
"The add's writes in one order",
A2,
read at that commit).
Its grammar is one `key=value` line per field (binding design,
"The binding",
lines 1480 to 1484 at that commit):

> claim=<generation>
> nonce=<32 lowercase hexadecimal digits>
> request=<branch|commit|detach> <lowercase hexadecimal of the argument,
> or ->
> phase=<claimed|slot|registered|snapshotted|cloned|mounted|filled|settling>
> settleReason=<proof|enospc|mount|clone|snapshot|destroy|checkout>

and the binding's A6 writes `registration=`,
`registrationIdentity=`,
`pointer=` and `phase=registered`,
after Git's add returns (binding design,
"The add's writes in one order",
A6,
read at that commit).
Its fields are `registration=<lowercase hexadecimal of the registration's path bytes>`,
`registrationIdentity=<inode>:<seconds>:<nanoseconds>` and `pointer=<the slot's .git bytes, lowercase
hexadecimal>`.

That is the whole of what this design takes from the binding:
the nonce written before Git runs,
and the registration's path,
identity and pointer bytes written after.
The claim carries nothing this design uses:
`take_claim` takes no payload,
and the claim's record holds only its header and its owner's generation,
process,
birth and namespace
(`lock/single_owner_claim.rs:167` to `:199`,
`:351`).

The binding is unavoidably a dependency,
not a record this design owns:
the review's H1 is right that this design may not ask the claim for a field it cannot carry,
and the review's first-ranked answer is that the lock's bytes are a pure function of the binding's nonce.
This design is written that way,
and V8 keeps the other two answers reachable.

### The threat model this design argues under

The project's own sentence,
quoted ([staging][staging],
"The handle record's place"):

> The threat model that answer states is the project's own for this machine:
> the account is trusted,
> and root and
> the account are not a security boundary here.

This design says where an argument survives that sentence and where a check exists only for a hostile account,
in "Hardening the stated threat model may make unnecessary".

### No configuration

The project answered that everything works with no configuration file
([sweepchoices][sweepchoices],
"What was decided",
lines 19 to 27):

> The project first answered that everything must work by default with no configuration

and then,
for the case with no file at all,
"1:
no file needed (Recommended)",
recorded at 2026-11-22T14:27:54-05:00.
This design adds no setting and no file a person must write.

### No prompt for authority

The wrapper never asks a person for a credential ([spec][spec],
line 7348):

> Avoid ever asking for someone to type a username or password.

with "Passphrases count too (Recommended)" at line 7350.
Where this design would need authority it does not have,
it refuses and says what to do.

### Doctor's place in the report

The proposal fixes where a setup finding goes ([proposal][proposal],
"What doctor checks and prints",
lines 1190 to 1193):

> Setup findings go to standard error and do not change `doctor`'s exit
> status.
> `doctor` does not start a service,
> create a test dataset,
> repair registrations or change configuration.

So this design's doctor lines are standard error,
and doctor writes no lock and removes no registration.

### Lock-free first, and no unmeasured limit

`AGENTS.md` rule `NLK`:

> Concurrency is lock-free first (atomic rename,
> `O_EXCL`,
> one owner per item);
> an unavoidable lock covers the
> smallest unit,
> never a whole repository,
> with its reason written beside it.

`AGENTS.md` rule `LQM`:

> Shipped resource limits (unit caps,
> timeouts,
> size bounds) cite a workload measurement or owner decision beside
> the value

So this design ships no wait and no cap that it has not measured,
and the one byte bound it ships is derived from the bytes it composes rather than chosen.

### The record precedent

The slot identity store was accepted as one file per slot,
each made with `O_EXCL` and published by one link,
with no lock ([slotstore][slotstore],
"What was decided"):

> becomes a folder with one file per slot.
> Each slot's file is made with `O_EXCL`,
> and the store takes no lock.

with "A:
O_EXCL staging,
then link (Recommended)" and "A:
no listing per sweep (Recommended)" in "The design
accepted".
It is a precedent to weigh for the one write this design still makes (the lock's install in recovery),
not a decision about it,
and this design keeps no record folder of its own,
so the slot store's listing rule bears on it only through that install.

## What Git 2.56.1 does

Every statement here is read at `v2.56.1` in the named file and line,
or measured in P1 to P10 through the wrapper and again in G1 to G14 as Git alone.

### What the wrapper adds to `git worktree add` and `git worktree remove`

The wrapper forwards a `git worktree add` whose form is outside its cheap ladder unchanged,
and takes the ladder's forms itself ([spec][spec],
lines 1152 to 1176):

> The ladder applies to a `git worktree add` that names one path and at most one commit-ish and uses only `-b <branch>`,
> `--detach`,
> `-d`,
> `--quiet`,
> `-q`,
> `--lock`,
> `--reason` and `--checkout`,
> run through the wrapper from a linked worktree,
> a bare repository or a main checkout,
> without `--no-worktree-copy`,
> without a global option other than `-C`,
> `--no-pager`,
> `-P`,
> `-p`,
> `--paginate`,
> `--no-advice` and `--no-lazy-fetch`,
> and with an existing parent directory of the destination.
> Any other form is Git's own,
> unchanged.

Three things the wrapper adds matter to a probe read as evidence about Git:

- It writes records of its own beside the registration.
  An add through the wrapper leaves `monochromatic-git-policy-cli-copy-owner` and `monochromatic-git-policy-cli-worktree-added` in the registration (F13,
  observed in P1),
  and the cheap ladder's stage claim and destination record are written before real Git runs and rewritten when it
  returns ([spec][spec],
  lines 4401 to 4405).
- It copies the source's ignored entries into the new worktree and says so on standard error ([spec][spec],
  from line 867,
  "Linked-worktree ignored-state synchronization"),
  which P1 printed as `copied 0 ignored filesystem entries`.
- Its `git worktree remove` serves a btrfs subvolume worktree itself,
  refuses a worktree whose registration holds `locked`,
  and hands every other case to Git ([spec][spec],
  lines 3119 to 3381),
  with the lock lookup at line 3252.
  `git worktree move` is forwarded like any other command ([spec][spec],
  line 880).

So a host probe's answer about `git worktree add` is the wrapper's answer plus Git's,
and only the gate image's runs are Git's alone.
Every claim below is one that both runs agreed on,
or one read from Git's source that no wrapper step can change:
the presence and bytes of `worktrees/<id>/locked`,
and what `prune`,
`gc`,
the automatic maintenance,
`move`,
`remove`,
`repair`,
`lock` and `unlock` make of it.
The wrapper's own removal route refuses a lock by design (F16),
which is the one removal sentence in this design that is the wrapper's and not Git's.

### Where the lock lives, and what Git reads it for

`worktrees/<id>/locked` is the whole mechanism ([git-layout][git-layout],
lines 292 to 297):

> If this file exists,
> the linked working tree may be on a portable device and not available.
> The presence of this file prevents `worktrees/<id>` from being pruned either automatically or manually by `git worktree prune`.
> The file may contain a string explaining why the repository is locked.

Git reads it with `file_exists`,
which is `lstat` (`dir.c:3191` to `:3195`),
so any entry at that name counts,
a directory and a dangling symbolic link included (P3,
the link arm of probe3).
It reads the bytes only to show them:
`worktree_lock_reason` reads the file,
trims it,
and returns it (`worktree.c:305` to `:326`),
which is what `git worktree list` and Git's refusals print (`builtin/worktree.c:1025` to `:1030` and `:1072` to `:1076`).
That trim is why this design compares the file's bytes on disk,
never the words Git prints (L5).
It writes the bytes with `write_file`,
which truncates and writes,
completes the line with one `\n`,
and creates the file `0666` less the umask (`wrapper.c:760` to `:783`;
P1 measured `0644`).
Nothing in Git records who wrote the file.

The lock marks;
it does not serialize.
`git worktree lock` reads the file and then writes it,
with no exclusive create (`builtin/worktree.c:1203` to `:1212`),
so two callers can both pass the read and both write,
and neither is refused (F19).
Nothing in this design uses the lock to keep two operations apart:
two cooperating wrapper commands are kept apart by the per-slot claim,
which is the accepted design's own sentence ([proposal][proposal],
lines 909 to 910):

> A per-slot claim only serializes cooperating
> wrapper operations.

The add takes that claim as its first step (the binding design's order),
the removal holds it (the binding design's V9),
and this design's install of a lost lock and its settle act only under it.
A command outside the wrapper is kept apart by nothing,
and the lock does not pretend to be one.

### Git writes the lock before it writes the registration

`git worktree add` creates `worktrees/<id>` (`builtin/worktree.c:502` to `:512`),
then writes `locked` (`:526` to `:530`),
and only then writes the `gitdir` file and the slot's `.git` (`:540` to `:546`),
the `commondir` (`:548` to `:550`),
and the worktree's ref store,
`HEAD` and logs (`:555` to `:567`).
The value it writes is `initializing` unless `--lock` was given,
in which case it is the `--reason` text (`:527` to `:530`,
`:879` to `:884`).
Upstream states why `--lock` exists ([git-worktree][git-worktree],
lines 246 to 248):

> Keep the worktree locked after creation.
> This is the equivalent of `git worktree lock` after `git worktree add`,
> but without a race condition.

and when `--reason` arrived ([git-relnotes][git-relnotes],
lines 20 to 21):

> "git worktree add --lock" learned to record why the worktree is locked with a custom message.

### What Git's own failure path leaves

On a failure after `locked` is written,
`done:` unlinks `locked` first (`builtin/worktree.c:599` to `:604`),
and the process's exit handler `remove_junk` then deletes the administrative directory and the destination directory
(`:273` to `:288`,
with `junk_work_tree` set to the destination at `:536`).
So a failing `--lock` add has a short unlocked moment before both directories go,
and a kill inside that moment leaves an unlocked partial registration.
The destination `remove_junk` deletes is the slot as it stands,
a pre-existing empty directory included.
A signal Git catches runs the same handler (`:515` to `:520`,
`:290` to `:295`).
The review's signal probe saw the slot removed in three of eight attempts through the wrapper,
which cannot separate Git's removal from the wrapper's own cleanup;
Git's source is what says the destination goes.

### What prune removes, and what stops it

`git worktree prune` lists `worktrees`,
and for each entry `should_prune_worktree` decides (`builtin/worktree.c:212` to `:247`,
`worktree.c:937` to `:1025`):

1.  An entry that is not a directory is pruned,
    with the reason `not a valid directory`.
2.  An entry holding `locked` is kept,
    whatever the file says,
    and it is not even offered to the duplicate check (`worktree.c:966` to `:969`,
    and the caller's `else if (path)` at `builtin/worktree.c:232` to `:233`,
    which skips it because a locked entry answers no path).
3.  An entry with no `gitdir` file is pruned at once,
    with `gitdir file does not exist`.
4.  An entry whose `gitdir` names a path that does not exist is pruned when the entry's `index` is missing or no newer than the expiry (`worktree.c:1013` to `:1018`).

The default expiry of an explicit `git worktree prune` is `TIME_MAX` (`builtin/worktree.c:259`),
which is why an unlocked registration with a missing pointer and a recent index still goes,
as item 6 measured on 2.43.0 and P3 measured on 2.56.1.

### What `git gc` and automatic maintenance do

`git gc` runs `git worktree prune --expire <gc.worktreePruneExpire>` (`builtin/gc.c:338` to `:347`,
`:718` to `:720`),
whose default is `3.months.ago` (`builtin/gc.c:144`).
Git also runs `maintenance run --auto` after each commit it makes (`run-command.c:1956` to `:1985`,
`--auto` at `:1980`),
and that run's `worktree-prune` task uses the same expiry and the same condition (`builtin/gc.c:350` to `:382`),
which the repository's specification already states ([spec][spec],
lines 17451 to 17462):

> Git 2.56.1 starts its automatic maintenance after each commit it makes (`maintenance run --auto`,
> detached by default,
> `prepare_auto_maintenance` in Git's `run-command.c`),
> and that run's `worktree-prune` task removes every `worktrees/<id>` that is not locked and holds no `gitdir` file

That quoted sentence is the specification's wording and is narrower than Git:
`worktree.c:1013` to `:1018` also prunes an entry whose `gitdir` file stands but whose target does not,
when the entry's `index` is absent or older than the expiry,
which is the case this design cares about (F5,
F6).

P4 and P5 measured the two consequences that matter here:

- an unlocked registration with a missing pointer and **no index** is pruned at once by `git gc` and by
  `git maintenance run --auto`;
- an unlocked registration with a missing pointer and an index younger than three months survives both.

A `--no-checkout` registration has no index (P1),
so the grace period does not protect the window this design cares about,
and the lock is the only thing that does.

### What a person's own Git commands do

Measured through the wrapper (P3 to P9) and as Git alone (G2 to G7,
G10 to G13):

- `git worktree list` shows `locked`,
  with the reason on its own line in the porcelain form.
  An administrative directory holding only a lock is not listed at all.
- `git worktree move` refuses with `cannot move a locked working tree, lock reason: <text>`,
  refuses one `--force`,
  and succeeds with two.
- `git worktree remove` refuses the same way,
  refuses one `--force`,
  and succeeds with two when the slot is a valid worktree:
  it deletes the slot directory and the registration (P7,
  G5).
  With the slot directory gone entirely it also succeeds,
  because a missing worktree path passes validation (G10).
  With the slot present and its `.git` missing even two forces fail validation (P8,
  G6,
  G12).
- `git worktree prune` and `git gc` keep a locked registration in every arm,
  and prune an unlocked one as described above.
- `git worktree repair`,
  with the administrative directory present and the pointer missing,
  rewrites the pointer from the `gitdir` file and leaves the lock untouched.
  With the administrative directory pruned it restores nothing.
- `git worktree lock` refuses over an existing lock and prints its words;
  that refusal is a read followed by a write,
  so two lockers at once can both succeed (F19);
  `git worktree unlock` succeeds without any check of who wrote the lock,
  and removes the protection.
- `git worktree lock` with no reason writes an empty file,
  which still protects.
- `git worktree add` at a path already registered refuses with `already exists`,
  and `--lock --reason` into a pre-existing empty directory succeeds and leaves the lock (G13).

## Facts

F1.
`worktrees/<id>/locked` is Git's only lock state:
its presence protects,
its bytes are a message,
and Git trims those bytes before it prints them.
Source:
`Documentation/gitrepository-layout.adoc:292` to `:297`,
`worktree.c:305` to `:326`,
`dir.c:3191` to `:3195`;
P1,
P3,
P6,
and G1,
G9,
G15 as Git alone.

F2.
`git worktree add --lock --reason <text>` writes `<text>` plus one newline as the registration's first lock,
before the `gitdir` file and the slot's `.git`,
and keeps it.
Source:
`builtin/worktree.c:502` to `:546`,
`:599` to `:604`;
P1,
P2,
G1 and G13 as Git alone.

F3.
A plain `git worktree add` also writes a lock during the add,
the word `initializing`,
and removes it before it returns unless `--lock` is given.
Source:
`builtin/worktree.c:526` to `:530`,
`:599` to `:604`;
P2.
G1 confirms the endpoint as Git alone,
that a plain add ends with no lock;
the interim word is Git's source's.

F4.
Prune keeps any entry that holds `locked`,
skips it in the duplicate check,
and never reads who wrote it.
Source:
`worktree.c:966` to `:969`,
`builtin/worktree.c:232` to `:233`;
P3,
P6,
P7,
and G2,
G5,
G9,
G15.

F5.
Prune removes an entry with no `gitdir` file at once,
and an entry whose `gitdir` names a missing path at once when the entry has no index,
or after the expiry when it has one.
Source:
`worktree.c:966` to `:1018`,
`builtin/worktree.c:259`;
P3,
P4,
P5,
and G2,
G3,
G4.

F6.
`git gc` and the automatic maintenance after a commit run the same prune with `3.months.ago` (`gc.worktreePruneExpire`),
so they remove an unlocked registration with no index at once.
Source:
`builtin/gc.c:144`,
`:338` to `:347`,
`:350` to `:382`,
`:718` to `:720`,
`run-command.c:1956` to `:1985`;
P4,
P5,
G3,
G4;
and [spec][spec],
lines 17451 to 17462.

F7.
A `--no-checkout` registration has no index,
so the three-month grace does not protect it.
Source:
P1 (no `index` in the administrative directory after a `--no-checkout` add),
P4,
P5,
G1,
G3,
G4.

F8.
Git's lock file is written by truncating and writing,
with one newline completed,
at mode `0666` less the umask,
and a kill inside that write leaves a shorter file that still protects.
Source:
`wrapper.c:760` to `:783`;
P6,
G8,
G9 (a torn lock and an empty lock both protect;
the torn shape is P6's).

F9.
`git worktree move` and `git worktree remove` refuse a lock,
refuse one force,
and succeed with two when the slot is a valid worktree;
with the slot present and its pointer missing even two forces fail validation,
and with the slot directory gone two forces also succeed.
Source:
`builtin/worktree.c:1339` to `:1344`,
`:1450` to `:1456`,
`:1461` to `:1477`;
P7,
P8,
G5,
G6,
G10,
G11,
G12.

F10.
`git worktree repair` restores a missing pointer from the surviving `gitdir` file and leaves the lock as it was,
and restores nothing when the administrative directory is gone.
Source:
P9,
G7;
`builtin/worktree.c:1491` to `:1540` and `worktree.c:644` to `:760`.

F11.
On a failure after the lock is written,
Git unlinks `locked` first and its exit handler then deletes the administrative directory and the destination
directory,
so a kill between those two steps leaves an unlocked partial registration,
and a pre-existing empty destination goes with the rest.
Source:
`builtin/worktree.c:273` to `:295`,
`:515` to `:520`,
`:536`,
`:599` to `:604`.

F12.
`git worktree unlock` removes any lock without checking its words,
and `git worktree lock` with no reason writes an empty file that still protects.
Source:
`builtin/worktree.c:1180` to `:1248`;
P6,
G8,
G9.

F13.
The wrapper already keeps small records inside a worktree's administrative directory:
`monochromatic-git-policy-cli-worktree-added` holds the worktree's path,
and `monochromatic-git-policy-cli-copy-owner` holds the identity of a copied destination.
Source:
`worktree/worktree_added_by_the_wrapper.rs:44` and its opening comment,
`:59` to `:91`;
`worktree/worktree_add_binding.rs:597`.

F14.
The wrapper's registration proof starts from the worktree's pointer bytes and ends with a `same_file` of the
registration's `gitdir` against the pointer's path,
so it cannot succeed without the pointer,
and its third fact needs the registration's `commondir`.
Source:
`worktree/worktree_registration_proof.rs:1` to `:20`,
`:229` to `:262`,
`:275`;
`invocation/real_git_candidate.rs:271` to `:276`.

F15.
The wrapper already removes a registration by emptying it through a held descriptor and removing the empty directory by name from the held `worktrees` directory.
Source:
[spec][spec],
line 3377.

F16.
The wrapper already refuses a removal when a `locked` entry stands in the registration it would remove (`worktree/worktree_removal_second_look.rs:127` to `:129`),
which is right for the btrfs route and wrong for a registration the ZFS add locked itself.

F17.
The add claim carries nothing this design can use:
`take_claim` takes no payload,
and its record is the header and the owner's generation,
process,
birth and namespace.
Source:
`lock/single_owner_claim.rs:167` to `:199`,
`:351`;
read from the source;
no probe.

F18.
The add binding's first write carries `nonce=`,
32 lowercase hexadecimal digits,
before Git runs,
and its `registered` write carries `registration=`,
`registrationIdentity=` and `pointer=`.
Source:
the binding design (`doc/planning/cli-git-zfs-add-binding-record-design.md` on its branch),
read at `3b4b073f0e` (2026-11-23T03:55:16-05:00),
its "The binding" and "What the writer confirms before each write";
read as the sibling's design,
not as a decision of this one.

F19.
The lock file serializes nothing:
`git worktree lock` reads the existing words and then writes,
with no exclusive create,
so two callers can both succeed,
and the wrapper's per-slot claim is what keeps cooperating wrapper commands apart.
Source:
`builtin/worktree.c:1203` to `:1212`;
[proposal][proposal],
lines 909 to 910;
`lock/single_owner_claim.rs:351`.

## Assumptions

These are this design's assumptions,
AS1 to AS7.
The binding design's add order uses A for its steps,
so a citation of one of its steps says "the binding's" beside the label,
and this design's assumptions carry the two letters AS so that the two lists cannot be read for each other.

AS1.
The binding's `nonce=` is on disk before the add's Git invocation runs,
survives until the registration is settled,
and is flushed with the write that made it.
This is the binding design's stated order and durability;
this design depends on it and would break if a write of the binding were lost.

AS2.
The registration's `gitdir` file and the slot's `.git` are written by one Git call,
`gitdir` first and the slot's `.git` second (`worktree.c:1132` to `:1133`),
so a kill between those two writes leaves a registration whose `gitdir` names the slot and whose slot holds no
pointer,
which is D2a,
and no later write of that call repairs it.

AS3.
A removal's destruction proof can say "the clone is gone" before the Git settle runs,
as [plan][plan] step 16 states it.

AS4.
The slot's directory on the checkout dataset is empty once the clone is destroyed,
as [proposal][proposal] step 5 requires,
so removing it changes nothing of a person's.

AS5.
The request nonce is unique per add,
so a lock whose words carry it names one add and no other.

AS6.
A scan of `worktrees/*` that finds the composed sentence in exactly one `locked` file has found the add that
wrote it,
because only that add could know the nonce,
and two registrations carrying the same sentence would mean a copied word,
which the design reports rather than choosing between.

AS7.
The binding's `claim=` reader rule does not refuse a record whose generation is older than the newest one present
(the review's H2,
the binding's to fix);
this design reads `nonce=` and `registration=` and never `claim=`.

## Limits

L1.
The lock protects the registration only.
A person with two forces,
with `unlock`,
or with a deliberate `rm` in the administrative directory,
ends it,
and this design does not try to stop that ([proposal][proposal],
lines 451 to 453;
F9,
F12).

L2.
The lock says nothing about the clone,
the dataset or the pointer.
The pointer copies and the binding record carry that,
and the lock is not a substitute for either.

L3.
Nothing proves who wrote a lock file from the file.
This design's ownership answer rests on the binding's nonce and a byte comparison,
and under the project's threat model that is not a security boundary,
only a durable fact.
The second host probe's millisecond samples do not prove that no shorter window exists;
they are a regression observation.

L4.
The window inside `git worktree add` between its `mkdir` and its write of `locked` is Git's own and the wrapper cannot close it (F2,
F3).
It is the same window every caller of `git worktree add` has,
and it is a few statements long.

L5.
The lock's bytes are compared as they stand on disk.
Git trims them before printing,
so the words a person sees and the bytes compared can differ by whitespace (F1),
and nothing here compares the printed words.

L6.
If the oldest Git the wrapper accepts does not support `--lock --reason`,
the clone rung has no lock it can trust,
and this design refuses that rung rather than shipping a registration prune can remove mid-fill.
`--lock --reason` arrived upstream by Git 2.33 (`Documentation/RelNotes/2.33.0.adoc:20` to `:21`),
and the wrapper's own floor is not stated anywhere this design read (M4).

L7.
The one byte bound this design ships is the length of the reason it composes,
computed in code from the constant parts and the nonce (32 bytes at most).
It is not a measured limit and is not a wait.

L8.
Nothing here measures a mounted clone.
Every probe ran on a plain directory,
mounted nothing,
and the mounted arms are owed (M1 to M3),
so no sentence of this design says "mounted or not".

L9.
An add killed between Git's write of `locked` and its write of the `gitdir` file leaves a locked registration
directory holding nothing else.
No prune removes it (F4),
`git worktree list` does not show it (F1,
G15),
and only the words identify it (S8).
A re-run of the add at that slot does not find it either:
Git makes `worktrees/<name>1` and leaves the leftover locked where it is (G16).
The leftover stands until the add's own resume removes it in the settle's order,
as the binding design's "A resume after a kill between Git's add and the registered write" decides,
or until a person removes the directory.
The finder that names the state,
and the removal rule,
are the binding's;
this design cites them and states none of their rules.

L10.
Git's own failure path unlinks `locked` before it deletes the registration and the destination (F11),
so a kill inside that moment leaves an unprotected partial state that neither the lock nor this design covers,
and the binding's A6 is what removes it.

L11.
A failing add can delete a pre-existing empty slot itself (F11),
so a slot the wrapper made may not be there after Git fails,
and the binding's recorded slot identity may no longer answer (the binding design's M6).

L12.
The lock's proof needs the binding's `nonce=`:
a record that cannot be read makes every lock foreign,
and a removal refuses on an unreadable record anyway.
A record the binding's reader refuses as foreign or behind the live facts is not unreadable:
its `nonce=` is still read,
as the check order says (N13 of the second gate review).

L13.
The lock is a marker,
not a mutex (F19).
Two cooperating wrapper commands are kept apart only by the per-slot claim,
and two commands outside the wrapper are kept apart by nothing,
so nothing here promises that a person's `git worktree lock` cannot stand beside the wrapper's own.

## How the wrapper writes the lock, and when

### The invocation

The add's Git invocation gains two arguments beside `--no-checkout`:

```text
git worktree add --no-checkout --lock --reason <text> <slot> <commit-ish>
```

`--reason` without `--lock` is refused by Git (`builtin/worktree.c:879` to `:881`),
so the two are always passed together,
from one list,
never through a shell.
No second `git worktree lock` runs,
and the wrapper does not create `locked` itself at this point:
Git's own write is the add's only write of the lock.

Why this route and not the other two,
with the alternatives ranked in "Choices this design marks for the project's veto",
V1:

- It is the only route whose lock carries this add's sentence from the moment Git makes the registration.
  A separate `git worktree lock` leaves whatever Git wrote (`initializing`) or nothing until that command runs,
  so a kill inside the add leaves a lock whose words are not this add's nonce,
  and the finder has to reach it by `gitdir` or by name instead of by the sentence.
- The wrapper writing the file itself would need Git's private id derivation (the sanitized basename and the counter
  suffix,
  `builtin/worktree.c:496` to `:501`),
  which is Git's to change,
  and would write after the same return.
- Upstream calls `--lock` the equivalent "without a race condition" (F2,
  `Documentation/git-worktree.adoc:246` to `:248`).

### The order of the add's writes

The order is not this design's.
It is the binding design's "The add's writes in one order",
read at `3b4b073f0e`
(2026-11-22T19:58:00-05:00),
and this design adds no step to it and reorders none.
Each moment below names that section's step labels as they read at that commit,
and says only what the lock is at that point and what proves it.
A moment is a window inside one of the binding's steps,
never a step of its own.

### The moments of an add

The route is `--no-checkout` (the binding's A5),
so the only long part of the add is Git's own preparation.

D0,
the binding's A0 to A4
(the preflight,
the claim,
the binding's first write,
the empty slot and the slot's identity).
No lock exists,
and nothing in `worktrees` names this slot,
so there is nothing for a lock to protect yet and nothing for this design to prove.
The binding's A2 has already written the nonce,
so the sentence this add will lock with is recomputable from the first write on.

D1,
inside the binding's A5,
after Git has made `worktrees/<id>` (`builtin/worktree.c:505` to `:512`) and before it writes `locked`
(`:526` to `:530`).
A bare administrative directory with no lock.
Any prune removes it at once,
because it holds no `gitdir` (F5),
and so do `git gc` and the automatic maintenance.
A SIGKILL or a power loss leaves it and nothing else runs:
there is no lock,
so nothing here is this design's to prove or protect,
and the binding's A6 scan reaches it only by its name rule.
A prune that removes the directory under a live add is the other case,
not a kill:
the add's next `write_file` fails and Git exits through its failure path (F11).
A signal Git catches runs the same handler (F11).

D2,
inside A5,
after `locked` holds the composed sentence and before `gitdir` (`:539` to `:541`).
A locked administrative directory with nothing else.
No prune removes it (F4),
and `git worktree list` does not show it (F1,
G15),
so a plain `git worktree add` at that slot makes `worktrees/<name>1` and leaves this one locked where it
is (G16),
which is why the binding's resume clears the leftover before it re-runs A5.
The lock's bytes equal the sentence recomposed from the binding's `nonce=`,
and that equality is the whole of what proves it:
there is no `registration=` and no `gitdir`,
so the wrapper's three-fact proof (F14) cannot run.
The finder of "The finder,
and the lock's part in it" returns it as a half-made registration,
naming its state.

D2a,
inside A5,
after the `gitdir` file and before the slot's `.git`,
which the same call writes second (`worktree.c:1132` to `:1133`).
The directory's `gitdir` names `<slot>/.git`,
which does not exist.
The sentence and the `gitdir` name prove the attribution together,
and the registration is proved by the two-fact variant (S1's note),
because the wrapper's own proof is still short its pointer (F14).
`git worktree list` shows this registration with a zero `HEAD`,
and `git worktree add` at the slot dies `is a missing but locked worktree` even with one force (G17),
so a re-run is no way out either.

D2b,
inside A5,
after the slot's `.git` and before `commondir` (`:543` to `:545`).
The pointer stands and its bytes are the ones the binding's A6 will record;
the wrapper's proof is still short `commondir` (F14).

D2c,
inside A5,
after `commondir` and before the ref store and `HEAD` (`:548` to `:557`).
The two-fact variant passes,
so the registration is proved,
and it is one the wrapper's own three-fact proof will recognise once `HEAD` exists and the binding's A6 records
the pointer.

D3,
the end of A5:
after the ref store,
`HEAD` and logs,
Git returns (`:555` to `:567`).
A complete registration whose pointer stands in the empty slot.
Every prune keeps it (F4),
and so would they unlocked,
because the pointer exists (F5).
This is the state the binding's A6 adopts and writes.

D4,
the binding's A6 write:
`registration=`,
`registrationIdentity=` and `pointer=`,
with phase `registered`.
Until it lands,
the binding stands at phase `slot`,
or at `claimed` when A4's write did not land,
and the lock is still attributed by the nonce (D2).
From this write on,
every later reader has the two facts without a scan.

D5,
the binding's A8 and A9
(the checkout snapshot and its origin inventory,
then the clone and its inventory).
The lock stands on a verified registration the binding names,
and every prune keeps it.

D6,
the binding's A10,
A10a,
A10b and A11
(the mount and its verification,
the frozen-content comparison,
the restore with the copied administration left in place,
and the exchange).
The binding states this window's boundary itself,
in its A11:
D6 begins at A10's write and ends at the exchange,
"that is the one statement of D6's boundary between the two documents"
(read at `3b4b073f0e`).
The project decided on 2026-11-22 what holds at the slot while the add fills it:
the copied `.git` stays until the exchange,
"so that no moment and no kill leaves the folder without a `.git`"
(`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
"What the slot holds while the add fills it").
So the slot's path holds a `.git` entry at every moment:
the copy until the exchange and the pointer after it.
Prune's `file_exists` counts any entry (F1),
so even unlocked the registration survives here,
and the lock is not what stops prune at this window.
What the lock covers is every window in which no `.git` stands at the slot:
D1,
D2 and D2a before Git writes one,
the rows Git's failure path leaves,
and an account that deletes the underlay pointer while the clone is unmounted
(the decision of 2026-11-19,
"Git lock + pointers (Recommended)",
and item 6).
A kill inside the window leaves a mounted clone whose slot shows the copy or the pointer,
a locked registration,
and a prune that would keep it either way;
the binding's resume reads the mount live and continues at the step its phase names.

D7,
the binding's A11's verification and A12
(the verified worktree and the dependency repairs).
The lock stays,
and it stays through every unmount and restart (the decision's own words).

Two consequences worth stating plainly:

- The exchange is the one moment the slot's `.git` changes kind,
  and it is one `renameat2` call,
  so no kill leaves it absent.
  Its guest run is the dirty-main design's owed measurement (the decision's "Still owed"),
  and this design does not rest a lock claim on it.
- The lock's protection at the fill is belt to the pointer's braces,
  and the design keeps the lock through it because the decision retains it through unmounts and restarts.
  The windows it really covers are the ones named above.

Git's own failure path is outside A5's moments.
A `ret` at one of the add's `goto done` sites
(all of them after `gitdir`,
the slot's `.git` and `commondir` were written)
unlinks `locked` first,
and the process's exit handler then removes the administrative and destination directories (F11).
A kill between those two steps therefore leaves the binding's row 7:
`gitdir`,
the slot's pointer,
`commondir`,
perhaps `HEAD`,
and no lock.
A `die` or a caught signal runs the same handler without the unlink,
so it leaves nothing (F11).
The binding's table and its recipe own what is done with each shape;
this design's finder names the row for them.

### A failed add, and the fallback when the clone route fails

The accepted text keeps the claim and cleans up only what is proven,
and the project decided on 2026-11-22 what becomes of the clone route:
"at every step,
the add destroys only what this add made and recorded (the clone,
then its snapshot),
runs Git's own checkout at the same slot,
and says which route made the worktree and why the first was abandoned"
(`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
"Whether a failed add may finish with Git's own checkout by itself").
That amends the accepted sentence for this route:
the fallback is not silent,
since the add reports it,
and the add settles the mount and its own intents itself.

The fallback's steps are the binding design's A20 to A27,
read at `3b4b073f0e`,
and the mechanics of the destroys and the checkout are the dirty-main design's
("What happens when the proof fails or the restore cannot finish",
in `doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`).
The order is theirs,
and this design states none of its own:

- A20:
  the binding's phase `settling` and `settleReason=` are written before the first destroy,
  and the dirty-main design's settle marker `<slot>.settle` is published beside its capture record,
  holding the nonce,
  the kind of refusal and the instant.
- A21:
  the unmount is requested and its receipt verified,
  when the entry phase has a mounted clone.
- A22:
  the clone is destroyed from the identities A9 recorded.
- A23:
  the origin cleanup record is published as `owed` before A22,
  or before A23 when A22 is skipped because the add made no clone,
  and the origin snapshot is destroyed from the identities A8 recorded,
  with `ENOENT` counted as done,
  which is the deferred-destroy design's rule for an add ("An add's own destroy does not pass through these
  checks",
  accepted and on `main`).
  A refused destroy leaves the record `owed` and the add goes on.
- A24:
  the managed lock is released and the registration kept.
- A25:
  Git's own checkout runs at the slot.
- A26 and A27:
  the binding is closed and the claim released.
  A27 prints nothing:
  the add's own sentence is the restoring design's,
  and it is printed after A25 and before A26,
  as the binding's A27 says.

The lock's part is A24,
and this design owns three things in it,
with the dirty-main design's note asking for the third:

- Who releases it:
  the add's fallback,
  in the same invocation,
  under the slot's claim.
- By which proof:
  that A22 destroyed the clone,
  and that A23 either destroyed the snapshot or left it to the origin cleanup record as `owed`,
  that the registration is the one the binding's A6 adopted (its five facts),
  and that the lock is this add's own (S1:
  its bytes equal the sentence recomposed from the binding's `nonce=`).
- The mechanism:
  one `unlinkat` of the `locked` name through the held registration descriptor,
  then one sync of the registration directory,
  and nothing else.
  The registration,
  its pointer and the slot stay,
  because A25 needs them.
  This is the removal's release step alone,
  not its recipe.

A kill on either side of A24 leaves:

- Before A20's write:
  nothing has been destroyed yet,
  so the ordinary resume applies,
  and the add continues the fill or the fallback by the live facts it reads.
  No `settling` phase and no settle marker stand,
  and the lock stands throughout,
  because nothing has released it.
- After A20 and before A24:
  the binding says `settling` and the marker stands,
  so the next add,
  holding the claim,
  continues at A20 to A23 and reaches A24;
  the lock stands,
  so prune keeps the registration and the finder names it.
- After A24 and before A25:
  the registration is unlocked and the checkout has not run.
  The next add continues at A25,
  and nothing of the lock's is left to finish.
  A command that proves the registration before the next add runs writes the lock again,
  by the decision of 2026-11-22,
  prints the one line,
  and the fallback's A24 releases it again.
- After A25:
  the checkout may be partial,
  and the next add repeats it at A25,
  as the binding says.
  The lock is already released,
  and the add's own Git child is not a forwarded wrapped command,
  so nothing on that path writes it again.

A kill between the failure and A20's write has destroyed nothing,
so the ordinary resume applies and no state is left that only a person can settle;
A20's write before the first destroy keeps that true for every kill after it.

For a mutation this add made and cannot prove,
the accepted text still decides:
keep it and report it
([proposal][proposal],
lines 429 to 431).
The fallback's destroy is only for what the add made and recorded.

## What the reason records, and what proves ownership

### The reason text

The reason is a message for a person,
and it is never authority.
It is the wrapper's sentence with the binding's nonce in it:

```text
monochromatic-git-policy-cli holds this Git registration for a ZFS clone slot (add nonce 9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f); remove the worktree with git worktree remove
```

It is ASCII,
one line,
and it ends without a newline:
Git completes the line (F8),
so `locked` holds exactly these bytes plus `\n`.
It names the program,
says what the registration belongs to,
carries the add's nonce,
and names the command a person runs.
It does not say "the wrapper",
because on this machine `git worktree remove` is the wrapper,
and a sentence that named a second program would send a person to a command that does not exist here (L5).

Its bytes are recomputed,
never stored:
the prefix,
the binding's 32 hexadecimal digits and the suffix are the whole of it,
and the byte bound this design reads a lock with is that length plus one,
computed in code from those parts (L7).

That is what plan step 10's "with its recorded reason" becomes ([plan][plan],
step 10):
the binding's `nonce=` and the sentence recomputed from it stand for the words,
and no second copy of them is written anywhere.

The nonce earns its place:
it makes two registrations' locks distinguishable by their words,
so a lock copied from another worktree of the same repository does not read as this one's,
and it attributes a lock whose `registration=` was never written (D2,
D4).

The shape of the sentence is a writer's choice for veto,
V2.

### What proves ownership

Ownership is a fact the wrapper recorded,
and the check is a byte comparison:

```text
verified = the slot's binding has a nonce=
        and `locked` is a regular file whose bytes on disk equal
            the sentence recomposed from that nonce
        and the registration is the one the binding names, or the one the sentence places
```

Each part is a read the wrapper already knows how to make:

- the binding's `nonce=`,
  read by the binding design's reader (AS1);
- the lock,
  read as a bounded,
  no-follow regular file,
  the way the slot pointer is read
  (`worktree_clone_mount/clone_slot_git_pointer.rs:51` and the reader below it),
  and compared as bytes on disk (L5);
- the registration,
  by the binding's `registration=` and `registrationIdentity=` when the phase is `registered` or later,
  or by the two-fact variant below when it is not.

The two-fact variant of the registration proof is new,
and this design asks the build for it:
the wrapper's existing proof starts from the pointer and ends with `same_file` against its path (F14),
so it cannot answer for a slot with no pointer (S5) or a binding whose `registration=` is not written yet (D2).
The variant proves the registration from:
the binding's recorded slot path and its identity,
the registration's `gitdir` file naming exactly `<slot>/.git`,
and,
when the `commondir` file stands,
its `commondir` naming this repository.

Which identity the variant compares is not one answer for every state,
and the second gate review's N5 is right that one answer is missing:
while the clone is mounted,
`<slot>` names the clone's root,
not the underlay folder the binding's A4 recorded,
so `slotIdentity=` cannot be compared at that path.
The rule is:

- unmounted,
  through the pinned parent descriptor,
  the variant compares the binding's `slot=` path and `slotIdentity=`;
  a mount at the slot prevents that comparison,
  which is why the mounted case is the next line and not this one;
- mounted,
  it compares the binding's `slot=` path and the clone's own recorded identity (`cloneGuid=` with the mount's
  source as the route's attach checks read it),
  and it does not compare `slotIdentity=` at the covered path.

M2's guest arm measures the mounted case,
because the covering is a mount fact and nothing on this host measured it.
With no pointer the variant is weaker than the three-fact proof,
and the design says so where it is used:
S5,
S6 and D2 to D2b name it,
and a test proves each.

Nothing in the check parses the reason for authority.
The nonce is read from the binding,
the words are compared,
and a person's lock that happens to contain the wrapper's sentence is treated as verified,
which is the deliberate cost of a proof that compares bytes rather than identity (V5).

### What the binding must hold

This design requires of the sibling's record,
and of nothing else:

- `nonce=` present in the first write,
  before Git runs,
  and unique per add (AS5).
- `nonce=` unchanged for the slot's life,
  so the sentence recomputes the same at every read.
- `registration=` and `registrationIdentity=` at phase `registered`,
  and `pointer=` with them.
- The record readable while the registration is alive:
  a reader that refuses the record after two claim
  generations (the review's H2) would make every lock foreign,
  so the binding's `claim=` rule is a dependency.
- Durability at least as strong as the lock's own:
  the binding flushes its writes,
  so a power loss that loses Git's unsynced `locked` keeps the nonce (AS1,
  L12's converse).

The binding's identity fields are an inode with a birth time
(`<inode>:<seconds>:<nanoseconds>`),
not a device number:
its "The identity fields",
read at `3b4b073f0e`,
records that choice,
and the device number the repository's own specification records as anonymous and per mount ([spec][spec],
lines 5589 to 5601) is not one of its fields.
Whether an inode and its birth survive a reboot is unmeasured,
and the binding says so in its own limits and reserves a test for it.
This design uses the recorded registration identity in the settle only,
where a refused identity stops the settle instead of guessing,
and the lock's own proof uses no identity at all,
so a reboot that changes the identity makes the settle refuse and leaves the lock unaffected.

None of these is a change to what the binding holds today,
by its own text:
they are the reason this design can keep no record of its own.

## How doctor and removal tell the states apart

The check order is the same in both,
and the words differ.

1.  Read the slot's binding,
    and take its `nonce=` from any well-formed record of the slot,
    whatever its phase,
    and even when the binding's own reader refuses the record as foreign or ahead of the live facts (N13 of the
    second gate review):
    those refusals decide what the add or the removal may do with the record,
    not whether the sentence can be recomposed.
    If the record is absent or unreadable,
    no lock can be verified,
    and every lock found is S3 or S7.
2.  Read the registration,
    by `registration=` or by the finder of "The finder,
    and the lock's part in it".
3.  Read `locked`:
    absent,
    or a regular file with bytes,
    or an entry the read refuses (a link,
    a named pipe,
    a
    directory,
    an unreadable file).
4.  Recompose the sentence from `nonce=` and classify.

### The finder, and the lock's part in it

How a registration is found and matched after a kill inside Git's add is the binding design's rule,
not this
design's.
It is its "A resume after a kill between Git's add and the registered write",
its "The states Git's source can
leave" and its "The recipe that clears rows 1 to 5,
7 and 8",
read at `3b4b073f0e` (2026-11-23T03:55:16-05:00).
That section lists one row per state Git's source can leave,
reads each entry twice through the pinned `<common>/worktrees` parent,
compares `locked` with the sentence recomposed from this add's `nonce=` including the newline Git's
`write_file` appends (`wrapper.c:760` to `:783`),
counts a proper prefix only when it covers the sentence's fixed part and the whole nonce,
trims a `gitdir`'s newline and resolves a relative one against the registration's directory
(`worktree.c:1105` to `:1134`),
and derives the candidate names from the slot's name through Git's own sanitiser
(`refs.c:191` to `:321` and `sanitize_refname_component` at `:376`)
plus Git's decimal counter (`builtin/worktree.c:495` to `:512`),
excluding a name another slot's binding owns.
This design does not restate those rules,
and a reader takes them from that section.

What stays in this document is the lock's own part,
and it is these three things:

- The sentence's composition from the binding's `nonce=`,
  which is the bytes the finder compares ("The reason text").
- What a lock's bytes prove about ownership,
  and that the proof reads the nonce from any well-formed record of the slot,
  whatever else that record is refused for ("What proves ownership").
- Who may install,
  complete or release a lock:
  only a command that holds the slot's claim,
  and the decision of 2026-11-22 says which commands write a lost one again ("A lost lock is written again,
  and which commands do it").

This design adds one rule for its own readers:
`doctor` and the removal read the finder's rows and never run its recipe,
and the removal's settle runs that recipe under the destruction proof ("How the wrapper's own removal releases the
lock").
The states S1 to S9 name the finder's rows where a lock meets them.

Why the finder is not `git worktree prune`:
a repository-wide prune would remove another slot's half-made registration,
whose lock may already be missing (the binding's rows 1 to 8),
so the finder is per slot by construction and prunes nothing.

### The states, the checks and the words

S1.
**The wrapper's verified lock.**
The binding has a `nonce=`,
and `locked`'s bytes equal the recomposed sentence,
and the registration is proved by `registration=` or by the two-fact variant.
Doctor prints,
on standard error,
with no change to its exit status:

```text
the clone slot <slot> holds the managed Git lock its add wrote for add nonce <nonce>, so `git worktree prune` keeps its registration; the settle releases it after the clone is destroyed
```

S2.
**The wrapper's own lock,
cut short.**
`locked`'s bytes are a proper prefix of the recomposed sentence **that covers the sentence's fixed part and
the whole nonce**,
which is the only shape a killed single write leaves (F8),
and which is how the binding's finder counts a prefix.
A shorter prefix,
including the empty file a person's `git worktree lock` with no reason leaves,
attributes nothing and is S3.
Doctor prints:

```text
the clone slot <slot> holds the managed Git lock its add wrote for add nonce <nonce>, cut short at <n> of <m> bytes, and it still keeps the registration; the next holder of the slot's claim completes it
```

S3.
**A lock the wrapper cannot recompose.**
The bytes are neither equal to the sentence nor a prefix that covers its fixed part and the whole nonce,
or the binding is absent or unreadable so that nothing can be recomposed.
An empty file is S3 here.
The binding's resume reads an empty `locked` as no lock for attribution (its "The states Git's source can leave" and its T20b),
and the two readers differ on purpose:
an empty file is also what a person's `git worktree lock` with no reason leaves,
and a removal must not release a lock it cannot attribute to this add.
The wrapper does not decide which person or program wrote it,
and it does not parse the words.
Doctor prints,
with the first line of the words as they are,
bounded by the same read:

```text
the clone slot <slot> is locked by words its add did not write (<words>), so the wrapper will not release it, and a removal refuses until a person unlocks it with `git worktree unlock <slot>`
```

S4.
**No lock.**
The binding has a `nonce=` and `locked` is absent.
Doctor prints:

```text
the clone slot <slot> holds no managed Git lock for add nonce <nonce>, so `git worktree prune` can remove its registration while the slot shows no .git and the registration has no index; the next command that holds the slot's claim writes the lock again and prints one line saying it did
```

S5.
**No pointer.**
The slot shows no `.git`,
so `file_exists` fails and prune falls through to the index and the expiry (F5).
This is a state beside the lock's,
not instead of it,
and doctor prints it with the lock's own state:

```text
the clone slot <slot> shows no .git, so `git worktree prune` can remove its registration at once while it has no index; the managed Git lock is what stops it
```

The registration is proved here by the two-fact variant,
never by the pointer proof (F14).

S6.
**No registration.**
`worktrees/<id>` is gone,
and nothing in the registration can be read.
Doctor prints what the binding names and what cannot be rebuilt,
with the measured words (F10):

```text
the clone slot <slot> has no Git registration, and `git worktree repair` cannot rebuild one once it is gone; the binding names <registration>, and the ways on are an explicit re-registration at the commit a person names, or the settle against the binding
```

S7.
**A registration no binding explains.**
A registration stands,
and either it holds no lock at all or its lock's words name a nonce the slot's binding does not hold.
A ZFS add always leaves a lock (V1),
so such a registration is not this design's add,
and doctor prints:

```text
the clone slot <slot> holds a registration no binding of this slot explains (<what was found>), so it is not this wrapper's worktree, and nothing here changes it
```

S8.
**A half-made registration the finder names.**
A `worktrees/<id>` answers one of the binding's rows 1 to 8 (its "The states Git's source can leave",
read at `3b4b073f0e`),
with this add's sentence in `locked` where a lock stands.
Doctor prints:

```text
the worktrees directory holds a half-made registration, <id>, for add nonce <nonce>, in row <row>; while a lock stands no prune removes it, and `git worktree list` does not show the ones whose only entry is `locked`; the add's own resume clears it with the binding's recipe and runs its Git step again
```

The finder is the binding's,
as specified above;
doctor reads its rows and never runs its recipe.

S9.
**A lock on somebody else's worktree.**
The registration is not this slot's at all (another path,
another `commondir`,
another repository),
and its lock is whatever it is.
Doctor prints it under the slot it found it for and says nothing is changed,
and a removal never touches it.

### What removal does with each state

The removal's preflight and its second look both classify,
and the second look classifies again immediately before the settle.
The removal holds the slot's claim,
so nothing else can install a lock between its steps (T8),
and a lock it finds missing it writes again by the decision of 2026-11-22.

- S1 verified:
  not a refusal.
  The settle releases it (below).
  `--force` is not involved.
- S2 cut short:
  not a refusal.
  The removal completes it in place before the destruction proof is used,
  so the release later has the full bytes to compare.
- S3 not the wrapper's:
  refusal,
  with the existing sentence and the person's words (`worktree/worktree_removal_second_look.rs:127` to `:129`),
  plus one sentence saying the way out is the person's `git worktree unlock <slot>`.
  No `--force` reaches it,
  and a single force keeps its existing meaning,
  waiving cleanliness.
- S4 no lock:
  the removal writes the lock again from the nonce,
  prints the one line,
  and then releases it after the destruction proof,
  as the decision of 2026-11-22 says;
  so the settle releases a lock that was there.
- S5 no pointer:
  not a refusal by itself.
  The pointer is the underlay's and the settle removes the slot;
  the registration is proved by the two-fact variant,
  and the removal never falls through to Git's own recursive removal for a managed slot ([proposal][proposal],
  lines 811 to 813):

  > If an inspection cannot prove its result,
  > refuse instead of falling through to Git's recursive removal for a managed
  > slot.

- S6 no registration:
  the removal may settle only after proving there is no replacement registration at that slot and no other
  registration naming that path ([proposal][proposal],
  lines 460 to 461):

  > A removal whose recorded clone and origin are already gone can settle a missing registration only after proving
  > there is no replacement.

  With no registration there is nothing to release,
  and the settle's Git half records that it found none.
- S7 or S9:
  refusal,
  before anything is destroyed.
- S8:
  not a removal's state.
  A dead add's half-made registration has no dataset to settle:
  doctor reports it,
  and the add's own resume removes it in the settle's order and runs its Git step again,
  as the binding design's "A resume after a kill between Git's add and the registered write" decides;
  a person may remove the directory,
  and no worktree is there.

A lock whose read is refused (a link,
a pipe,
a directory at `locked`) is S3 with its own cause named,
because `file_exists` still protects it (F1) and the wrapper cannot compare bytes it refuses to read.

## How the wrapper's own removal releases the lock

The removal's Git settle is plan step 16's code,
not step 13's:
[plan][plan] step 14 says (step 14,
lines 1760 to 1762):

> Steps 4 and 5,
> origin cleanup and the Git settle,
> need a destroyed clone and are built in step 16.

and step 16 lists "the Git registration and pointer settle after proven clone destruction".
It runs after the clone's destruction is proven (AS3),
and it is the authorized registration-cleanup step the decision names.
It never runs `git worktree remove`,
so no `--force` and no Git-side validation is involved,
and the slot is not a valid worktree at that point anyway (S5,
F9).

The order of removals is the binding design's,
not this design's:
its "The recipe that clears rows 1 to 5,
7 and 8",
in "A resume after a kill between Git's add and the registered
write",
read at `3b4b073f0e`.
This design adds nothing to it except the proof that comes first,
because the registration's subject is gone.
A builder reads its steps 1 to 8 there;
in its terms the slot's `.git` goes first,
the registration's files follow,
`locked` goes last,
and the registration's directory goes by `rmdir` from the held `worktrees` parent.
This design states none of its own,
and the earlier revision's order,
which unlinked `locked` first and then asked for an empty directory,
is withdrawn:
the binding's recipe removes the files first and the lock last,
so every kill before the lock's removal leaves a locked row the finder recognises,
and every kill after it leaves a bare directory it recognises by name.

Before that order,
the settle proves:

1.  **Destruction.**
    The persisted plan and the live reads say the recorded clone is gone (AS3).
2.  **The registration.**
    Hold `worktrees`,
    the registration and the slot's parent open by descriptor,
    and prove it is the recorded one:
    the binding's `registration=` path and identity,
    the `gitdir` back-link naming the slot's `.git`,
    and the pointer bytes against `pointer=` when the pointer exists.
3.  **The lock.**
    S1 or S2,
    from the recomposed sentence.
    A missing lock is S4,
    and the decision of 2026-11-22 has it written again here,
    with the one line,
    so this step then sees S1.

After the binding's recipe,
the settle removes the empty slot on the checkout dataset,
only with the recorded identity and only when it is empty (AS4),
and records the settle in the removal's own records.

The slot's `.git` is the binding's first step,
and this design's earlier revision lacked it:
its last step removed the slot only "when it is empty",
while the slot's underlay pointer stood in it,
so a removal of a destroyed clone would have left the pointer and a non-empty slot.
The review's R1 is right that the pointer's removal belongs to this order,
and the binding's recipe is where it now lives.


### A lost lock is written again, and which commands do it

The project decided on 2026-11-22,
answering "A:
put it back,
say so (Recommended)":
"any wrapped command that already holds that worktree's slot and has proved its registration writes the lock again
and prints one line saying it did"
([lostlock][lostlock],
"The wrapper writes a lost lock again and says so").
The removal is such a command:
it writes the lock again,
says so,
and then releases it as every removal does.
The design's first two questions,
who may install a lock and whether a removal stops when the lock is gone,
are answered by that record and are no longer asked;
"What the project decided about a lost lock" lists the options it did not take.

Every command that writes it again holds the slot's claim:
`<slot>.claim.<generation>`,
taken with `take_claim`
(the binding design's "The account's claim and root's claim",
read at `3b4b073f0e`).
It is not root's claim under `/run` that [spec][spec] describes at lines 6228 to 6272.

The commands,
the moment in each,
and what is proved first:

- The add's own resume,
  inside the binding's A6:
  from step 13 on,
  after the five facts prove the registration and before `registration=` is written,
  it writes the lock again if S4 stands,
  and prints the line.
  Until step 13 lands,
  step 10 refuses an unlocked complete entry,
  names it and prints the sentence the lock must hold,
  which is the binding's A6 and its V22 (the same split this design's V7 records).
  A fresh add needs none of this:
  its Git invocation wrote the lock with `--lock` at A5.
- The removal,
  at its settle's third proof:
  after the destruction proof and the registration proof,
  a missing lock (S4) is written again and the line printed,
  and the binding's recipe later releases it at its `locked` step.
  The write protects nothing between the two,
  because the clone is already destroyed;
  it is the record's rule applied literally,
  and it keeps the recipe's invariant that a settle releases a lock that was there.
- The kept cancel command,
  when it proves the registration under the slot's claim.
  That is a requirement on its design,
  in the decision's own words,
  because its design at `3035d29766` says the registration and its lock stand unchanged inside a removal
  and states no put-back;
  that design's round carries it.
- The binding design's settling command,
  after its third read proves the registration and before its own removal step,
  which then releases it with the rest.
- No other command:
  `doctor` never writes (it holds no claim and is read-only),
  a person's plain `git` is not the wrapper,
  and the root handler holds no account claim and writes no Git state.
  The add's own Git child at A25 is a Git process the add starts,
  not the wrapper and not a forwarded wrapped command,
  so it never writes the lock again;
  a resumed fallback add at A25 does not either,
  though it holds the slot and has proved the registration,
  because what it runs at A25 is that child.

The line,
printed once by the command that wrote it:

```text
the clone slot <slot> had lost its managed Git lock, so this command wrote it again for add nonce <nonce>
```

A kill between the write and the line leaves the lock written and the line unprinted.
The next command that proves the registration finds the lock standing,
writes nothing,
and says nothing about it;
nothing is lost but the report.
A kill inside the write leaves a prefix of the sentence (F8):
one that covers the fixed part and the whole nonce is S2,
which the next claim holder completes in place (V4),
and a shorter one is S3,
a lock the wrapper cannot recompose,
which a removal refuses until a person unlocks it.
A lock file stands either way,
so prune is stopped in every case.

### What a kill between the settle's steps leaves

Because the order is the binding's,
its invariant is this design's too:
every kill before `locked` is removed leaves a locked row the finder recognises,
and every kill after it leaves a bare directory the finder recognises by name.

- Before the destruction proof:
  nothing is removed and the clone may stand;
  doctor and the removal report the state,
  and the resumed removal runs again.
- After the proof and before the recipe's first step:
  a verified lock on a destroyed clone.
  Doctor reports S1 with the clone named as gone,
  and the resumed removal,
  or the kept cancel command,
  runs the recipe.
- Inside the recipe before `locked`'s step:
  a locked row.
  Whatever files the recipe already removed,
  the lock stands,
  no prune removes it (F4),
  and the finder names the row;
  the resumed removal repeats the recipe from the row it finds.
- After `locked`'s step and before the `rmdir`:
  a bare directory with no lock and an empty slot,
  which is the finder's row 1;
  the resumed removal removes it.
  No sync is needed between the two:
  the `rmdir` can succeed only when the lock's removal is in place,
  and a power loss that loses the unlink leaves the locked row it was.
- After the `rmdir` and before the slot's removal:
  an empty slot with no registration.
  The pointer went in the recipe's first step.
  Doctor reports S6,
  and the removal's resume removes the folder.
- After the slot's removal and before the settle's record:
  everything gone,
  the records unsettled.
  The resume proves absence and settles (the proposal's own rule,
  "Settle recorded destroys only after proving the expected objects are gone").

## Recovery

### The pointer is lost, the registration and the lock are there

The lock still protects,
and nothing about it changes.
Git's own repair rewrites the pointer from the surviving `gitdir` file and leaves the lock (F10,
P9),
which is the route the proposal already names ([proposal][proposal],
lines 457 to 459):

> A damaged link with surviving admin state can use `git worktree repair <verified-slot>` from the main checkout.

The wrapper's add resume can do the same itself,
by writing the binding's `pointer=` bytes into the slot after
proving the registration and the slot's identity;
the design prefers that inside an add,
and names `git worktree repair` to a person in every report where the wrapper is not the one acting.
The lock's recovery is nothing:
a lock is not lost with the pointer.

### The lock is lost, the registration is there

The binding's `nonce=` is still there,
so the sentence is still recomputable.
The install is one exclusive write of the recomposed bytes to `locked`:

```text
open(<registration>/locked, O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW, 0644), write the recomposed bytes
```

- `O_EXCL` decides the winner if two commands install at once,
  and no lock is held (NLK).
- A kill inside the write leaves a prefix,
  which is still a lock and still protects (F8),
  which is why this write does not stage and rename:
  a staging file would leave the registration unprotected.
- `EEXIST` means someone wrote a lock first;
  the classification runs again and never overwrites.
- The command holds the slot's claim ("A lost lock is written again,
  and which commands do it"),
  so the install cannot race the settle;
  the lock itself orders nothing (F19).

Any command that holds the slot's claim and proves the registration writes a missing lock again and prints the one
line,
which is the decision of 2026-11-22,
and `doctor` never writes.

### The registration is pruned

Nothing restores it.
Git's repair restores nothing once the administrative directory is gone (F10,
P9),
and the proposal says so ([proposal][proposal],
lines 457 to 461):

> If Git already pruned the admin entry,
> repair does not reconstruct HEAD,
> index or lock state.
> Restore verified registration data from backup before repairing links;
> if there is none,
> report what is lost and require an explicit recovery or discard,
> never guessing from a branch or path.

So the wrapper reports S6 and names what the binding records.
It offers exactly two ways on:
an explicit re-registration at the commit a person names,
which no record holds,
and a discard through the removal's settle against the binding.
It never guesses a registration from a branch or a path,
and it never re-adds at a commit it read from nowhere.

### Across restarts

Everything this design uses is on disk:
Git's `locked` file in the registration,
and the binding's `nonce=` in the wrapper's record folder.
Nothing is held in memory,
no daemon is needed,
and no start-time step writes either.
Git's `locked` write is unsynced (F8),
while the binding flushes its writes,
so a power loss can keep the nonce and lose the lock:
that is S4,
installable by the next claim holder,
and it is the safe direction because a lost lock only loses prune protection.

### What cannot be recovered

- The lock's words,
  if a person overwrote them (S3).
- The binding,
  if it was deleted or cannot be read:
  no sentence can be recomposed,
  every lock is S3 or S7,
  and a person settles the slot by hand.
- Any state of a pruned registration (S6).
- The lock's protection during the D1 window (L4) and during Git's failure path (L10).
- A dead add's locked directory with no `gitdir`,
  whose nonce the slot's binding no longer holds:
  nothing then attributes it,
  and a person removes the directory (S8).

## What the lock changes for a person using Git directly

Measured on 2.56.1 through the wrapper (P3 to P9) and as Git alone (G2 to G7,
G10 to G13),
for a worktree this design's add made,
on a plain directory.
The mounted arms are owed (M1 to M3,
L8).

- `git worktree list` shows `locked`,
  and its porcelain form shows the words.
  Nothing else about the listing changes.
- `git worktree move` refuses,
  and says `use 'move -f -f' to override or unlock first`.
  The wrapper refuses the move of a bound managed clone anyway ([proposal][proposal],
  lines 474 to 476),
  so this only makes Git agree.
- `git worktree remove` refuses plain and with one force,
  and succeeds with two while the slot is a valid worktree,
  deleting the slot directory and the registration (P7,
  G5).
  With the slot directory gone entirely,
  two forces also succeed and delete the registration (G10).
  With the slot present and its `.git` missing,
  two forces fail validation (P8,
  G12).
- `git worktree prune` and `git gc` keep the registration while the lock stands,
  with or without the pointer,
  on a plain directory.
- Automatic maintenance after a commit keeps it too (P5),
  where without the lock it would remove a no-index registration with a missing pointer (F6,
  F7).
- `git worktree repair` still works and does not disturb the lock (F10).
- `git worktree unlock` removes the protection with no question and no record;
  `git worktree lock` afterwards leaves the person's words,
  and the wrapper reports S3 from then on.
- `git worktree add` at the slot refuses with `already exists`,
  which is Git's answer for any existing directory.
- `git status` inside a repaired slot whose clone is gone reports every tracked file deleted (P9),
  which is the confusion the proposal already asks `doctor` to report instead of ([proposal][proposal],
  lines 436 to 438).

## Where the code goes

Named so a build can place it,
not to prescribe internals:

- The reason's composition and the recompose-and-compare:
  one module exporting the prefix,
  the suffix,
  the bytes for a nonce,
  the byte bound and the comparison,
  placed in plan step 10,
  because the binding's A5 and A6 call it there,
  so the add,
  the recovery,
  the removal and `doctor` all use one spelling.
  The finder itself is the binding design's and is built with its own part of step 10.
- The invocation's two arguments and the add's use of the binding's nonce:
  in the ZFS add route step 10 builds,
  beside the binding's first write it already depends on.
- The classification:
  a function with the states of "How doctor and removal tell the states apart" as its answer,
  built in step 13 and used by `doctor`,
  by the removal's preflight,
  by its second look and by the settle,
  so no caller invents its own order.
- The two-fact registration variant:
  beside `worktree/worktree_registration_proof.rs`,
  as a second entry point that takes the recorded slot,
  its path and identity,
  and the `gitdir` back-link,
  and never needs the pointer (step 13).
- The finder is not this module's:
  this design calls the binding's,
  whose rows `doctor`,
  the removal and the add's resume all read,
  and whose recipe the binding's resume and this removal's settle both run.
- The release and the clearing:
  the binding design's recipe (its "The recipe that clears rows 1 to 5,
  7 and 8"),
  cited and not restated,
  run in plan step 16's settle after the destruction proof.
- `doctor`'s lines:
  the filesystem report the proposal extends ([proposal][proposal],
  "What doctor checks and prints",
  lines 1188 to
  1197),
  on standard error,
  with the exit status unchanged.
- The wrapper specification ([spec][spec]):
  the add section gains the two arguments and the reason's composition,
  the removal section gains the release ordering,
  and `### Doctor` gains the states' lines.
  This design keeps no record of its own,
  so no records section gains one.

## The split between plan steps 10, 12, 13 and 16

The plan's own split is a suggestion,
not a decision
([plan][plan],
"Where the design is silent",
lines 2913 to 2929,
and "Open items and the step that needs each",
lines 2828 to 2835):

> Step 10 takes the lock's write path with the registration,
> which is the plan's choice,
> and step 13 takes the distinction,
> recovery and release.

and the split is V7's choice for veto.

What this design builds,
step by step:

- Step 10:
  the two arguments the add hands Git,
  and the reason's module that composes them from the binding's nonce and recomposes them for every reader.
  The finder itself is the binding design's and is built with its own part of step 10;
  this design calls it and states none of its rules ("The finder,
  and the lock's part in it").
  The binding's A5 and A6 call the arguments,
  the sentence and the finder,
  and all are step 10's,
  so step 10 cannot wait for step 13 to supply them (N4 of the second gate review).
  Until step 13 lands,
  step 10 refuses an unlocked complete entry,
  names it and prints the sentence the lock must hold,
  which is the binding's A6 and its V22.
  Step 10 writes no record of its own.
- Step 12:
  the presence of `locked` in the slot report,
  unclassified.
- Step 13:
  the states' classification,
  the two-fact proof variant,
  the install of a lost lock,
  and `doctor`'s lines.
- Step 16:
  the settle's release ordering and emptying,
  which this design's "How the wrapper's own removal releases the lock" specifies.

The plan's allocation is the quote above:
step 10 takes the write path and step 13 the distinction,
recovery and release.
This design keeps the distinction,
the recovery and the release where the plan puts them,
and adds that the composer and the comparison land in step 10,
because step 10's own steps call them,
while the finder is the binding's and is built with its part of step 10;
the second gate review's N4 is that sequencing fault.
V7 records the choice with the plan's allocation beside the adopted one.

## Tests a build must have

Each race is named with how a test forces it without timing luck,
and each test sits under the step that builds what it tests.
A `hook` below means a test-only seam of the kind the wrapper already uses
(`#[cfg(test)] AFTER_THE_STAGING_FILE_IS_WRITTEN` in `foundation/private_storage.rs:432`).

Step 10,
which builds the invocation's arguments,
the sentence,
the comparison and the finder:

T1.
The add's invocation carries `--lock` and `--reason` with the bytes recomposed from the binding's `nonce=`,
and no separate `git worktree lock` runs.
Forced by a seam that records the argument list the add hands Git,
with a real-Git arm that then reads `locked` and compares it to the recomposition (F2).
T2 was dropped in the first revision round and its number is not reused,
so a citation after it does not move.
T5.
A `git worktree prune`,
`git gc` and `git maintenance run --auto` before the exchange does not remove the registration:
the slot holds the copied `.git`,
which `file_exists` counts (the decision of 2026-11-22,
"What the slot holds while the add fills it"),
so the locked and the unlocked arm both keep it.
Forced by the seam before the exchange,
with a hand-built control that removes the copy first:
the unlocked control loses the registration and the locked one keeps it,
which proves the seam can see the difference.
T6.
The same three commands on either side of the exchange keep the registration in both arms,
because the slot holds the copy or the pointer at every moment (the same decision).
Forced by a seam before and after the single exchanging rename,
on a host fixture whose `.git` is a directory exchanged with a file.
The mounted-clone arm is M1's.

T16.
The finder returns each of the binding's rows,
"The states Git's source can leave" at `3b4b073f0e`,
for a fixture planted from the source line that leaves it:
row 1 (bare directory),
row 2 (lock only),
row 3 (`gitdir` holding the slot's path and a newline),
row 4 (the slot's pointer,
alone or empty),
row 5 (`commondir` holding `../..` and a newline),
row 6 (complete,
with `logs/HEAD`,
`refs/` and `HEAD`),
row 7 (the failure path killed after the unlink),
row 8 (Git's own removal killed inside it),
row 9 (nothing,
empty slot) and
row 10 (nothing,
the slot holds a `.git`).
Forced by one fixture per row,
and the test also covers the finder's own matching rules with a second slot named `topic1` beside `topic`,
a relative `gitdir`,
an empty `locked`,
a torn one that stops before the nonce,
and the two-match case (AS6).


Step 13,
which builds the states,
the two-fact variant,
the install and `doctor`'s lines:

T3.
Each kill state is recognised by `doctor` and by the removal,
not only by the add's resume.
Forced by building the state by hand for D1,
D2,
D2a,
D2b,
D2c and D4,
from the measured shapes (P1,
P3,
P6,
G15,
G16,
G17):
a D4 state must read S1,
never S3 or S7.
T4.
A registration at D3,
with the binding at phase `slot` and the lock present,
is adopted as S1 by the next command that holds the claim.
Forced by the fixture of T3 with the binding's first write present.
T9.
`doctor` reports S1 to S9 with the exact words and an unchanged exit status,
each from a fixture,
with a healthy slot as the control.
T10.
Removal refuses S3,
S7 and S9,
and proceeds for S1,
S2,
S4,
S5 and S6 as specified.
T12.
A person's `git worktree unlock` followed by `git worktree lock --reason <the wrapper's own sentence>`
is treated as S1.
This is a deliberate consequence,
and the test fixes it rather than leaving it to chance (V5).
T13.
A lock installed by the recovery protects exactly as Git's does:
install,
then `git worktree prune`,
`git gc` and two forces,
each expected as P6,
G8 and G9 measured.
T14.
`git worktree repair` after pointer loss on a host fixture leaves the lock and returns the pointer (F10),
so `doctor`'s S5 words stay true.
T15.
The two-fact registration proof answers for S5 (no pointer) and D2 (no `registration=`),
and refuses a registration whose `gitdir` names another slot or whose `commondir` is another repository.

Step 16,
which builds the settle:

T8.
An install cannot race the settle.
Forced by a seam after the settle's proof and before the binding's recipe's first step,
where a second command tries to install:
the second must refuse for want of the slot claim,
and the settle must finish the recipe.
T11.
The order is the binding's recipe,
not an order of this design's:
a seam after each of its steps
(the slot's `.git`,
`gitdir`,
`commondir`,
`HEAD` with `refs/` and `logs/`,
the other entries,
`locked`,
the `rmdir`,
the empty slot),
each of which leaves a row "What a kill between the settle's steps leaves" names,
with `git worktree prune`,
`git gc` and `git maintenance run --auto` run against it.

The fallback's step,
which the dirty-main design allocates to step 11
(its "Which build step builds which part",
in `doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`),
and whose order is the binding's A20 to A27:

T17.
The fallback's release:
with A20's `settling` phase and settleReason written and the dirty-main design's `<slot>.settle` marker
published,
and with A22's clone destroyed and A23 either having destroyed the snapshot or left it `owed` to the origin
cleanup record,
and their proof in hand,
the add releases the lock at A24,
before A25's Git checkout.
Forced by a seam on either side of that release:
a kill before it leaves S1 with the binding at `settling`,
and the next add at the slot finishes A21 to A25,
while a kill after it leaves an unlocked registration and an ordinary worktree,
with nothing of the lock's left to finish,
and a command that proves the registration in between writes the lock again and prints the one line,
which A24 then releases.
The fixture is the binding's `settling` phase with its `settleReason=`,
and the dirty-main design's settle marker.
A second arm plants a held snapshot:
A23's destroy is refused,
the origin cleanup record stands `owed`,
the lock is still released at A24,
the binding and the claim are closed at A26 and A27,
and the next `git worktree` command carries the snapshot.

Which need the guest:

- M1 and M2 (below) are the only arms whose subject is a mounted clone,
  and they are guest tests.
- T5 and T6 have a host arm and a guest arm;
  the host arm fixes the seam and the exchange,
  the guest arm the mount.
- T15's no-pointer arm is a host fixture;
  the mounted arm is M2's.
- T17's fixture is the binding's `settling` phase and the dirty-main design's settle marker;
  its host arm forces the seam,
  and a guest arm would need the mount that M1 to M3 already owe.
- Every other test above runs on the host,
  in a throwaway repository,
  against the host's Git and the built wrapper.

## Measurements owed

M1.
**A deleted underlay pointer while the clone is unmounted,
in the guest,
on the releases this design targets.**
Item 6 measured it on Git `2.43.0` and OpenZFS `2.4.4-1`:
with the checkout mounted from the file-backed pool and a clone mounted at the slot,
unmount the clone,
delete the underlay pointer,
then run `git worktree prune`,
`git gc` and `git maintenance run --auto` in the main checkout,
once with the managed lock and once without,
and record which registration survives each of the six runs.
The contract:
the unlocked arms lose the registration,
the locked arms keep it.
Host fixtures show the same on a plain directory,
through the wrapper (P3 to P5) and as Git alone (G2 to G4).
This is the measurement that decides whether the lock's reason holds on the releases the route ships.
The fill no longer needs an arm:
the decision of 2026-11-22 keeps a `.git` at the slot at every moment,
and the exchange's guest run is that decision's own owed measurement.


M2.
**`git worktree repair` over a mounted and an unmounted clone slot,
in the guest,
and the identity the two-fact variant compares while the clone is mounted.**
Where the rewritten pointer lands in each case,
whether the lock file is touched,
what `git worktree list --porcelain` and `git status` say afterwards,
and,
while mounted,
whether the clone's own recorded identity (`cloneGuid=`) answers at the covered path while `slotIdentity=` does not.
That last arm is the second gate review's N5,
and it decides the rule "What proves ownership" states.
Host P9 and G7 measured the plain-directory case only.

M3.
**A person's double force on a clone slot,
in the guest,
mounted and unmounted.**
Unmounted,
the three shapes this round measured on a plain directory are the contract:
the slot directory gone and the registration locked,
two forces exit 0 and delete the registration (G10);
the slot present and its `.git` missing,
two forces fail validation (G12);
the slot a valid worktree,
two forces delete the slot and the registration (P7,
G5).
The mounted arm is not measured and is owed:
Git's `delete_git_work_tree` recurses through the checked-out files before the mount point refuses the removal
(`builtin/worktree.c:1463` to `:1477`),
so the question is what a person's two forces delete inside a mounted clone before that refusal.
What survives of the slot directory,
the pointer and the dataset,
and what the wrapper's next command sees.

M4.
**The wrapper's Git floor,
on the host.**
`--lock --reason` arrived upstream by Git 2.33 (`Documentation/RelNotes/2.33.0.adoc:20` to `:21`),
so what is owed is only the wrapper's own floor:
the oldest Git it accepts,
stated once,
and the check that it carries `--lock --reason`,
with a positive control on 2.56.1.
If the floor is below 2.33,
L6 refuses the clone rung on such a machine.

Nothing else about Git is owed.
The gate image's runs answered every Git-alone arm on `2.56.1`,
and no image was built or removed for them.
The guest arms above need a guest,
and this round ran none.

## Choices this design marks for the project's veto

Each is a writer's choice,
adopted so the design is whole.
A veto takes the named alternative and amends the sections that depend on it.

One cost comes from a decided answer rather than from a writer's choice,
and it is recorded here so the project reads it with the veto entries:
a kill inside the write that puts a lost lock back leaves an empty or short `locked`,
and a removal cannot tell an empty file from a person's `git worktree lock` with no reason;
it refuses as S3 until a person runs `git worktree unlock`.
The decision of 2026-11-22 carries that cost (W19 of the fourth check).

V1.
**The write route**:
`--lock --reason` inside the add's own Git invocation (adopted),
or a separate `git worktree lock` after the add,
or the wrapper writing `locked` itself.
Example:
an add at `.claude/worktrees/one` that is killed just after Git accepts it.
Under the adopted route the registration is locked with this add's sentence from Git's own creation of it,
and the finder reads the sentence from the lock.
Under a separate `git worktree lock` the registration holds Git's `initializing` or no lock at all until that
command runs,
so the finder reaches it only by `gitdir` or by name,
and the add has a second command to run and another moment to be killed in.
The wrapper writing the file itself needs Git's private id derivation and writes after the same return.
Pros of the adopted route:
the lock carries the sentence from the first moment,
no second command,
no window the wrapper adds,
and upstream's own answer to the race (F2).
Cons:
the reason must be known before the add,
so it must be recomputable from the binding's nonce (AS1).
Ranking:
`--lock --reason` > a separate `git worktree lock` > the wrapper writing the file,
because the first puts the sentence in the lock from the start,
the second leaves the lock's words to Git or to nobody until it runs,
and the third adds a name derivation Git owns.

V2.
**The reason's shape**:
one ASCII line with no trailing newline,
ending in the command a person runs (adopted);
or a short token with no sentence;
or free text that may hold newlines and arbitrary bytes.
Example:
the words appear in `git worktree list` and in Git's `cannot remove a locked working tree, lock reason: <words>`.
The adopted line tells a person what holds the worktree and what to do next with one command;
the token tells them nothing;
free text lets a newline split a `git worktree list` row and makes the word a person reads differ from the bytes
compared.
Pros:
actionable,
one row,
bounded,
and its bytes are the bytes compared.
Cons:
it is long enough that a person retyping it will not match it exactly (V5).
Ranking:
the one line > the token > free text,
because a person's way out is the point of the words,
and free text costs both the row and the comparison.

V3.
**The settle's order and mechanism**:
the binding design's recipe,
which removes the slot's `.git`,
`gitdir`,
`commondir`,
`HEAD` with `refs/` and `logs/`,
the other entries,
then `locked` last,
and the directory by `rmdir` (adopted);
or the earlier revision's order,
which unlinked `locked` first and then emptied;
or a rename of the whole registration out of `worktrees` and a delete under a wrapper name;
or unlocking and running `git worktree remove -f -f`.
Example:
a removal killed after the first step of its Git settle.
The adopted order leaves a locked row the finder recognises at every kill before the lock's removal,
and a bare directory it recognises by name after it;
the release-first order leaves an unlocked,
partly emptied registration at every kill,
which only the name rule reaches;
the rename leaves nothing Git can see from the first step,
at the cost of a new name and a new leftover for `doctor`;
Git's removal deletes the slot directory and needs two forces,
which this design never forwards to a lock.
Pros of the adopted order:
one order for the add's resume and the removal,
no new name,
no force,
and every kill lands on a state the table names.
Cons:
the lock stands while the registration's files go,
so a kill there leaves a locked entry only the wrapper can find,
and the wrapper must keep the recipe's names right.
Ranking:
the binding's recipe > release first > rename out > Git's removal with two forces,
because one order beats two,
the release-first order trades an attributable leftover for one only the name rule reaches,
the rename buys a better failure mode with a new name,
and Git's removal forwards the force this design forbids.
This is the second gate review's N9 and the diff check's R1:
the proposal's refusal to fall through to Git's recursive removal (lines 811 to 813) is a refusal rule,
not a decision of this mechanism,
and the order is the binding's to state and this design's to cite.


V4.
**The torn lock**:
the wrapper completes its own cut-short lock in place (adopted),
or leaves it and reports,
or refuses the slot until a person acts.
Example:
a kill inside Git's write of the reason,
leaving the first 140 bytes,
which covers the sentence's fixed part and the whole nonce.
The adopted completion rewrites the same bytes with one truncating write,
so the lock is never absent (F8).
Leaving it means the registration stays protected but every removal refuses until a person unlocks it.
Pros of completion:
one write,
no unlocked instant,
no person needed.
Cons:
the wrapper writes inside Git's directory for a state it did not create.
Ranking:
complete > leave and report > refuse,
because the bytes are provably the wrapper's own (a prefix that covers the nonce),
and a person cannot fix it
faster than one write.

V5.
**A person's identical words**:
treated as the wrapper's verified lock (adopted),
or treated as S3 by recording something the words cannot carry.
Example:
a person copies the wrapper's sentence into `git worktree lock --reason` after an unlock.
The adopted rule makes the lock S1 for the next removal,
which then releases it after the destruction proof.
Under the alternative the record would have to carry something else (a timestamp,
an inode),
and each of those is a
guess.
Pros of the adopted rule:
one comparison,
no guesses,
and the consequence is a person undoing and redoing a
protection on the worktree they are about to remove.
Cons:
the wrapper cannot tell the person's copy from its own.
Ranking:
adopted > recording an identity,
because under the project's threat model the identity proves nothing the bytes do not.

V6.
**The mode of an installed lock**:
`0644`,
matching Git's own files (adopted),
or `0600`,
matching the wrapper's records.
Example:
a lock the recovery installs after a person's unlock.
`0644` looks exactly like Git's file to any other reader,
and `0600` is what every other record in the registration
is.
The words are not secret;
the difference is who can read them.
Pros of `0644`:
one shape for Git's file whoever wrote it.
Cons:
it is the one file in the registration the wrapper writes with a non-private mode.
Ranking:
`0644` > `0600`,
because the file is Git's,
not the wrapper's.

V7.
**The split across steps 10,
12,
13 and 16**,
and where the composer and the comparison live:
the plan's allocation with those two in step 10,
and step 10 refusing an unlocked complete entry until step 13's installer (adopted);
or the plan's allocation read literally,
with them in step 13 while step 10 inlines its own copies;
or keeping the add route refusing until step 13.
The plan's own allocation,
quoted beside the adopted one:

> Step 10 takes the lock's write path with the registration,
> which is the plan's choice,
> and step 13 takes the distinction,
> recovery and release.

Example:
the first build order merges the add's Git step and the module it calls together;
the second merges step 10 calling a module that does not exist yet,
so step 10 carries a copy the build must later delete or move;
the third holds the whole ZFS add route back until step 13 lands.
Pros of the adopted split:
the lock lands with the registration it protects,
step 10 calls only code that exists at that step,
and the pointer half stays free of the lock's classification.
Cons:
step 10 carries a module whose classification and `doctor` lines land two steps later.
Ranking:
adopted > the plan read literally > the add refusing until step 13,
because the first is the only order in which every call has its callee,
the second makes step 10 carry a copy it must then move,
and the third holds the route back for nothing it protects.

V8.
**Where the lock's proof lives**:
the sentence recomposed from the binding's `nonce=` (adopted);
or a second record inside the registration holding the exact lock bytes;
or fields in the claim file.
Example:
a person runs `git worktree lock --reason held` on a managed slot.
Under the adopted proof the words do not match the recomposed sentence,
so the lock is S3;
under a second record the record says otherwise and the same answer follows;
under claim fields the evidence would sit in a file `take_claim` writes and rewrites.
Pros of the adopted proof:
one record family,
one durability policy,
attribution at D2 and D4 for free,
and no write of this design's own.
Cons:
verification depends on the binding being readable,
and the lock design leans on a sibling's field.
Ranking:
B (the nonce) > A (a second record) > C (the claim),
because B removes a file and a durability mismatch while keeping a byte comparison,
A leaves the shared claim alone but adds a home and a second flush rule,
and C asks a primitive that carries no payload for one (F17).
The second gate review puts its options as B > A > C for the same reasons.

V9.
**A mutation this add made and cannot prove**:
keep it and report it (adopted);
or stop before acting and name the state for a person;
or destroy it anyway.
Example:
the clone request returned an error and the GUID read that would identify what it made never answered.
The adopted answer keeps the object and reports it,
as the accepted text says ([proposal][proposal],
lines 429 to 431);
stopping before acting leaves the same object and stops the add with a report;
destroying it would guess at an identity.
Pros of the adopted answer:
it destroys nothing it cannot name,
and it is the accepted text's own answer.
Cons:
the object stands until a person or a later proof resolves it.
Ranking:
keep and report > stop and name > destroy,
because the first is the accepted text and destroys nothing,
the second is the same state with the add stopped,
and the third cannot be undone.
The project decided on 2026-11-22 what a failed add does when it can prove what it made (the fallback,
"A failed add,
and the fallback when the clone route fails");
this entry covers only the mutation it cannot prove,
which that decision leaves to the accepted text.


V10.
**When the fallback releases the lock**:
at the binding's A24,
after A20's `settling` record and the settle marker and after the destroys A22 and A23,
and before A25's Git checkout (adopted);
or after A25's checkout;
or at the failure,
before A20's record.
Example:
a clone route that failed,
its clone destroyed and its snapshot either destroyed or left `owed` to the origin cleanup record,
and the fallback about to run Git's checkout at the slot.
The adopted point is the binding's own A24,
so the two documents state one order,
and a kill during the checkout leaves an unlocked registration and an ordinary worktree.
Releasing after the checkout leaves a locked ordinary worktree at every kill during it,
which a person's `git worktree remove` refuses and which the wrapper must classify as S1.
Releasing at the failure,
before A20's record,
leaves an unlocked managed registration whose clone may still stand,
which is the one state where prune can remove a registration whose clone exists.
Pros of the adopted point:
it is the binding's label,
no second order,
and no kill leaves a locked ordinary worktree.
Cons:
the registration is unlocked for the length of the checkout,
which costs nothing while the slot holds a `.git`.
Ranking:
A24 > after the checkout > at the failure,
because the first is the binding's place and keeps the invariant that an ordinary worktree is never locked,
the second breaks that invariant at every kill during the checkout,
and the third can lose the registration in the one state this design's D6 names,
where the account deleted the underlay pointer while the clone is unmounted.

## What the project decided about a lost lock

The project answered the design's first question on 2026-11-22,
choosing "A:
put it back,
say so (Recommended)":
"any wrapped command that already holds that worktree's slot and has proved its registration writes the lock again
and prints one line saying it did"
([lostlock][lostlock],
"The wrapper writes a lost lock again and says so").
The record decides what plan step 13 builds,
and step 10 does not wait for it.

Its second question,
whether a removal stops when it finds the lock gone,
was to be asked only under another answer.
Under this one the removal is such a command:
it writes the lock again,
says so,
and then releases it as every removal does.
The question does not arise,
and the design no longer asks either.
"A lost lock is written again,
and which commands do it" says which commands those are,
the moment in each,
what each proves first,
the one line,
and what a kill between the write and the line leaves.

The options the project did not take,
listed once,
as the record lists them:

- writing the lock again only through a new repair command,
  with `doctor` naming it and a removal refusing until it is run;
- never writing it again,
  reporting only.

The cost the record states with the chosen answer is the one this design carries:
it undoes a deliberate `git worktree unlock` without asking.
The record leaves nothing else to ask;
what a reader does with a record a later release cannot read,
and what happens to a slot whose recorded clone is gone with its absence unprovable,
are the binding record design's questions,
and their answers are in
[sevenanswers][sevenanswers],
"A record that cannot be read" and
"A slot whose recorded clone is gone and cannot be proved gone".
This design leans on the first where it refuses on an unreadable record (S3 and its note on an unreadable
binding),
and on the second where the settle's absence cannot be proved (S6).

## Hardening the stated threat model may make unnecessary

The project's sentence is that the account is trusted and root and the account are not a security boundary
([staging][staging],
"The handle record's place").
This design follows it without dropping a check whose absence would hurt even a trusted account:

- The comparison of bytes rather than words **survives** the threat model.
  It exists because a person's accidental `git worktree lock` must not read as the wrapper's own,
  which is an accident,
  not an attack.
- The binding's nonce as the only source of the sentence **survives** it.
  It exists because a record outlives the worktree it was written for,
  which happens without any attacker.
- The bounded,
  no-follow reads and the `O_EXCL`,
  `O_NOFOLLOW` write **survive** it.
  A named pipe at `locked` would hold a wrapper command inside `open(2)` for as long as the pipe's writer likes,
  and a symbolic link at the binding's name would put the wrapper's bytes wherever it points;
  both are mistakes a person and a program can make with no hostility.
- A claim that **does not survive** it:
  that only the wrapper can write the lock.
  A hostile account can write it,
  byte for byte,
  and this design would then call it its own.
  That is acceptable only because the account is trusted;
  the design makes no ownership claim beyond that.
- Checks that exist **only** for a hostile account,
  kept anyway because they cost a read:
  refusing a `locked` entry that is not a regular file,
  refusing a binding longer than its limit,
  and comparing `commondir`.
  Under the threat model none of them buys security;
  each prevents a mistake.

## Where a sibling and this design disagree

Each line names the two positions and,
where a record decides it,
which one.

1. **What the claim carries.**
   This design uses nothing from the claim (F17).
   The binding design adds no field to it.
   They now agree;
   the first draft of this design had asked the claim for the nonce and the bytes (H1),
   which is
   withdrawn.
2. **The nonce's form.**
   Both say 32 lowercase hexadecimal digits;
   this design's earlier example had eight,
   taken from a probe placeholder,
   and is corrected.
3. **Where the lock's ownership is proved.**
   This design:
   from the binding's `nonce=`,
   with no record of its own.
   The binding design's dependency note said "the add-time Git lock of a ZFS clone worktree decides where the
   lock's record lives";
   the answer is nowhere,
   and this design says so.
   Both documents now say the same,
   and V8 records the choice against a second record and against claim fields.
4. **The key.**
   This design keys the lock on the slot through the binding and keeps no key of its own.
   The binding keys on the slot.
   The destruction plan keys on the clone GUID,
   and its design once kept a pointer file inside the registration
   (`<registration>/monochromatic-git-policy-cli-clone-destruction-plan`);
   if it still does,
   the binding's recipe removes it with the other entries,
   and if it does not,
   the recipe has nothing to remove there.
5. **The moment of writing.**
   This design writes nothing;
   the binding's first write precedes Git and its `registered` write follows it.
6. **Durability.**
   Git's `locked` write is unsynced (F8);
   the binding flushes.
   A power loss can therefore keep the nonce and lose the lock,
   which is S4 and installable,
   and a power loss cannot produce a verified lock without a nonce.
7. **How the registration is settled.**
   The mechanism is one:
   the binding design's recipe,
   with the destruction proof before it,
   and never `git worktree remove`.
   Two other sentences in the siblings name a mechanism of their own,
   and both should cite the recipe instead:
   the binding's "The record's end" cites the origin cleanup design's removal order,
   whose step 10 "settles the registration and releases the managed lock",
   and the binding's settling command describes the same step as "the lock released,
   the registration emptied through the held descriptor,
   the directory removed by name",
   which is the earlier revision's release-first order.
   This design's settle is the recipe,
   and reconciling the three statements is the binding round's and the removal-side round's,
   not a second order here.
   The proposal's sentence (lines 811 to 813) is a refusal rule for a failed inspection,
   not a decision of the settle's mechanism (the second gate review's N9);
   the mechanism is V3's writer's choice.
8. **What a failed add leaves,
   and the fallback's end.**
   The project decided on 2026-11-22 that the clone route's failure finishes by itself with Git's own checkout,
   after destroying only what the add made,
   and what the slot holds while the add fills it (the copied `.git`,
   exchanged at the end).
   This design follows both,
   and the binding now designs the same fallback in its A20 to A27 and the exchange in its A10b and A11,
   so the two agree:
   the registration stays and the lock goes at A24,
   the fill window no longer carries the lock's reason,
   and what a failed add leaves when it cannot prove what it made is V9's entry,
   which the accepted text decides.
9. **How a resume finds the registration after a kill before `registered`.**
   The finder is the binding design's,
   and this design cites it by its heading and states none of its rules ("The finder,
   and the lock's part in it").
   The binding still calls it "the lock design's scan" in three places (its A6,
   its "A resume after a kill between Git's add and the registered write" and its siblings list),
   which is a label for the binding round to correct;
   nothing in this design depends on the label,
   only on the section being the one place the rule lives.
10. **A re-add "at the recorded commit".**
    The first draft offered it;
    no record holds the commit,
    the branch or the command line,
    and the recovery section now says so.
11. **The registration proof.**
    This design asks for a two-fact variant that works without the pointer (F14,
    T15);
    the binding design already records the facts it needs.
12. **The fill window.**
    The binding states the window's boundary in its A11,
    D6 begins at A10's write and ends at the exchange,
    and this design cites that statement and states none of its own.
    The project decided on 2026-11-22 that the copied `.git` stays until the exchange,
    so the slot holds a `.git` at every moment of the fill and the lock is not that window's guard;
    the lock covers the moments where no `.git` stands.
    The two documents now agree.
13. **Who runs `doctor`.**
    This design's doctor reads the binding and the lock as the account,
    which is what the proposal's doctor
    section assumes;
    the destruction plan design's step 18 has root reading its record.
    Root can read these too,
    so no conflict,
    and nothing here needs root.
14. **The three record folders.**
    This design adds no folder.
    The binding,
    the destruction plan and the origin cleanup choose three homes;
    that choice is theirs and the binding design's question about the three record folders,
    not this design's,
    and this design only needs the binding's name resolved through whatever reader it publishes.

## What this design does not cover

- The pointer half of prune protection:
  its underlay pointer's writing,
  keeping and removal are step 12's.
- The binding's home,
  format,
  phases,
  durability and reader rules,
  which are the binding design's,
  and the separate question of where the route's record folders live.
- The removal's own steps before the settle:
  lineage proof,
  unmount,
  snapshot batch,
  head destroy.
- A lock for any worktree the wrapper did not add,
  and any lock on an ordinary (non-ZFS) worktree.
  The btrfs route refuses any lock and keeps doing so (F16).
- The wrapper's Git floor beyond naming the measurement (M4,
  L6).
- Any ZFS property,
  mount,
  unmount or dataset behaviour,
  which the accepted design and the measurements already
  own.
- `git worktree repair`'s internals:
  the design uses its measured behaviour and never re-implements it.
- The mounted arms (M1 to M3),
  which no sentence here treats as measured.

## Sources read

Git,
cloned at `v2.56.1` into
`~/temp/agent/zfs-add-time-git-lock-design-2026-11-22/scratch/git-v2.56.1`
(`gh repo clone git/git ... --depth 1 --branch v2.56.1`,
with the push URL disabled before anything else):

- `builtin/worktree.c`:
  `:200` to `:247` (prune and its duplicate pass),
  `:259` (the expiry of an explicit prune),
  `:273` to `:295` (the junk removal and its signal handler),
  `:496` to `:520` (the id,
  the administrative directory and the first lock),
  `:526` to `:567` (the lock,
  then the linking files,
  `commondir` and the ref store),
  `:599` to `:604` (the lock's removal),
  `:845` to `:884` (`--lock` and `--reason`),
  `:1025` to `:1076` (list),
  `:1180` to `:1248` (lock and unlock),
  `:1339` to `:1344` (move),
  `:1450` to `:1477` (remove,
  its validation and its deletion).
- `worktree.c`:
  `:305` to `:326` (the reason reader),
  `:644` to `:760` (repair,
  and the pointer rewrite inside it),
  `:937` to `:1025` (`should_prune_worktree`).
- `dir.c`:
  `:3191` to `:3195` (`file_exists`).
- `wrapper.c`:
  `:760` to `:783` (`write_file` and its line completion).
- `builtin/gc.c`:
  `:144` (the default expiry),
  `:338` to `:347` (the prune task),
  `:350` to `:382` (its auto condition),
  `:718` to `:720` (gc runs it).
- `run-command.c`:
  `:1956` to `:1985` (`prepare_auto_maintenance` and `maintenance run --auto`).
- `Documentation/git-worktree.adoc`:
  `:246` to `:248` (`--lock`),
  `:400` to `:410` (the `locked` file).
- `Documentation/gitrepository-layout.adoc`:
  `:292` to `:297` (`worktrees/<id>/locked`).
- `Documentation/RelNotes/2.33.0.adoc`:
  `:20` to `:21` (`--lock` records why).

This repository,
at `main` `5dd8883a60`,
and the sibling branches by `git show`:

- [`doc/decision/cli-git-zfs-worktree-design.md`][decision],
  whole.
- [`doc/planning/cli-git-zfs-worktree-design-proposal.md`][proposal],
  the sections quoted above.
- [`doc/planning/cli-git-zfs-worktree-implementation-plan.md`][plan],
  steps 10,
  12,
  13,
  14 and 16,
  and the sections
  named above.
- [`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`][measurements],
  item 6.
- [`doc/decision/cli-git-zfs-staging-under-cache.md`][staging],
  "The handle record's place".
- [`doc/decision/cli-git-zfs-step-8-sweep-choices.md`][sweepchoices],
  "What was decided".
- [`doc/decision/cli-git-zfs-slot-identity-store-format.md`][slotstore],
  whole.
- [`package/git-policy/cli/SPEC.md`][spec]:
  lines 683 to 760 (`doctor`),
  2024 (the convert command's lock refusal),
  3252 to 3273 (the removal's lock lookup and its force reading),
  3377 (the registration's emptying),
  4401 to 4405 (the stage claim),
  5589 to 5601 (the anonymous device number),
  6228 to 6270 (the per-slot claim),
  7348 to 7350 (the credential rule),
  17451 to 17462 (Git's automatic maintenance and `worktrees`).
- `package/git-policy/cli/src/native/lock/single_owner_claim.rs`,
  lines 167 to 199,
  220 to 233,
  351 and 392 to 396.
- `package/git-policy/cli/src/native/worktree/worktree_add_binding.rs`,
  lines 70 to 87 and 182 to 196.
- `package/git-policy/cli/src/native/worktree/worktree_added_by_the_wrapper.rs`,
  lines 1 to 91.
- `package/git-policy/cli/src/native/worktree/worktree_registration_proof.rs`,
  lines 1 to 30,
  125,
  229 to 262 and 275.
- `package/git-policy/cli/src/native/worktree/worktree_removal_second_look.rs`,
  lines 95 to 135.
- `package/git-policy/cli/src/native/worktree_clone_mount/clone_slot_git_pointer.rs`,
  lines 40 to 60 and the reader
  below them.
- `package/git-policy/cli/src/native/foundation/private_storage.rs`,
  lines 424 to 442 and 479 to 481.
- `package/git-policy/cli/src/native/invocation/real_git_candidate.rs`,
  lines 271 to 280.
- `package/git-policy/cli/src/native/health/doctor.rs`,
  lines 340 to 560.
- `AGENTS.md`,
  rules `NLK`,
  `LQM`,
  `WTU`,
  `HOM` and `RXI`.

The sibling designs.
Three are accepted and on `main`,
and this design cites them there by heading:
the dirty-main restore design ([dirtymain][dirtymain]),
the deferred-destroy checks and
the private propagation after a replacement mount.
The others are read by `git show` of their branches and treated as unreviewed:

- [`doc/planning/cli-git-zfs-add-binding-record-design.md`][binding],
  "The claim",
  "The binding",
  "What the writer confirms before each write" and "The record's end".
- `doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
  its home,
  its key and its pointer file inside the registration.
- `doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
  its home.
- `doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
  its stop points.
- `doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`.

## Revisions

The commits of this document,
with each committer time from `git log --format=%cI`:

- `e053ad7807`,
  2026-11-22T17:31:11-05:00:
  the design,
  whole,
  including the first round's gate-image runs.
- `db99dc05eb`,
  2026-11-22T17:31:40-05:00:
  the first commit list and the Markdown line breaks normalized to one clause per line.
- `d0c43ad410`,
  2026-11-22T17:33:30-05:00:
  corrected Git and specification line citations.
- `36f317a73b`,
  2026-11-22T17:33:46-05:00:
  the document's commit list.
- `05a0eadf9d`,
  2026-11-22T17:35:46-05:00:
  the add's moments renamed D0 to D7,
  apart from the measurements.
- `71de819090`,
  2026-11-22T18:30:09-05:00:
  the answer to the gate review:
  the proof derives from the binding's nonce and no lock record is kept,
  the moments,
  the failure path and the states are rewritten,
  the split names step 16,
  the questions are the review's merged ones,
  and every citation was rechecked.
- `a7902e293b`,
  2026-11-22T18:31:12-05:00:
  the answer round's commit list.
- `cf555b1a9f`,
  2026-11-22T19:22:28-05:00:
  the answer to the second gate review:
  the moments named by the binding's step labels read at `c144499eb9eb`,
  the scan specified,
  the composer and the comparison moved into step 10,
  the account's claim cited instead of root's,
  the settle's mechanism recorded as a veto entry,
  one Git-alone arm restored and two added,
  the stale citations corrected,
  and the questions reduced to the review's two.
- `5b628ea8f4`,
  2026-11-22T19:22:42-05:00:
  the second review round's commit list.
- `ec7055ba25`,
  2026-11-22T19:23:38-05:00:
  the binding design's `registered` write quoted at its current tip,
  with no blockquote of an earlier revision left.
- `144622019a`,
  2026-11-22T20:00:09-05:00:
  main merged with `--no-ff`,
  bringing the decision of 2026-11-22 on the fallback,
  the proof's window and the slot's copied `.git`.
- `6f0c2bdb23`,
  2026-11-22T20:08:42-05:00:
  the answer to the diff check:
  the settle names the binding's recipe and states no order of its own,
  the slot's pointer is in that order,
  the finder is cited instead of restated,
  the failure path's row corrected,
  the fallback's lock release stated,
  the fill window no longer rests the lock,
  the tests filed by step,
  the assumptions relabelled AS,
  and V3,
  V9 and V10 rewritten.

- `567e3c02f8`,
  2026-11-22T21:21:14-05:00:
  main merged with `--no-ff`,
  bringing the project's answers on a lost lock and on the seven record questions.
- The commit after `567e3c02f8` that carries this list,
  whose hash `git log` reports:
  the third check's fixes:
  the lost lock written again by the commands that hold the slot,
  the decided section that replaces the two questions,
  the fallback's one order in the binding's A20 to A27,
  D6 and the exchange named from the binding,
  S2 and S3's prefix rule,
  T16 and T17 refiled,
  and the corrected citations.

- `ba39ffa043`,
  2026-11-22T22:07:27-05:00:
  main merged with `--no-ff`,
  bringing the owner's further answer on who records what when the add undoes what it made.
- `9e3ceff079`,
  2026-11-22T22:09:48-05:00:
  the fourth check's fixes:
  A24's proof and T17's refused-destroy arm,
  the A23 bullet,
  the add's own Git child in the command list,
  the kill-before-A20 bullet,
  V10's example,
  the cancel command as a requirement,
  the empty lock's two readers,
  the recipe's eight steps,
  the grammar quote,
  the binding at `7d041d773b`,
  and W19's cost beside the veto entries.

- `a7b9f02924`,
  2026-11-23T03:57:11-05:00:
  main merged with `--no-ff`,
  bringing the accepted dirty-main restore design and the two other accepted designs.
- The commit after `a7b9f02924` that carries this list,
  whose hash `git log` reports:
  the fifth diff read's fixes:
  V4's example and V7's clause,
  the binding at `3b4b073f0e`,
  A27's printed nothing,
  the empty lock's cite,
  and the accepted designs cited from `main`.

Any commit after those carries this list;
the round's report names every commit with its committer time.

## Links

[decision]: ../decision/cli-git-zfs-worktree-design.md
[proposal]: cli-git-zfs-worktree-design-proposal.md
[plan]: cli-git-zfs-worktree-implementation-plan.md
[measurements]: cli-git-zfs-measurements-owed-before-acceptance.md
[fallback]: ../decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md
[dirtymain]: cli-git-zfs-dirty-main-restore-on-rung-1-design.md
[lostlock]: ../decision/cli-git-zfs-add-time-git-lock-put-back-when-lost.md
[sevenanswers]: ../decision/cli-git-zfs-clone-worktree-records-seven-answers.md
[staging]: ../decision/cli-git-zfs-staging-under-cache.md
[sweepchoices]: ../decision/cli-git-zfs-step-8-sweep-choices.md
[slotstore]: ../decision/cli-git-zfs-slot-identity-store-format.md
[spec]: ../../package/git-policy/cli/SPEC.md
[git-worktree]: https://github.com/git/git/blob/v2.56.1/Documentation/git-worktree.adoc
[git-layout]: https://github.com/git/git/blob/v2.56.1/Documentation/gitrepository-layout.adoc
[git-relnotes]: https://github.com/git/git/blob/v2.56.1/Documentation/RelNotes/2.33.0.adoc
[vm-prune]: cli-git-zfs-measurements-owed-before-acceptance.md#6-deleted-underlay-pointer-prune-and-lock
