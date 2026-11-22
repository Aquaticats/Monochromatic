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
and this round's own arms are G10 to G14.

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

> the underlay pointer kept for prune,
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

Neither the decision nor the proposal says a failed add removes its registration,
so this design does not remove it (Q2).

### The record whose nonce the lock's proof rests on

The add's durable binding record is the sibling design's,
`doc/planning/cli-git-zfs-add-binding-record-design.md`
on branch `docs/planning-cli-git-zfs-add-binding-record-design`,
read by `git show` ("The record family",
"The binding").
That document is unreviewed here and nothing in it is adopted as decided.

Its first write is made before Git runs,
and it carries the request nonce,
32 lowercase hexadecimal digits,
before anything has mutated (binding design,
"What the writer confirms before each write"):

> The claim is taken,
> then the binding's first write carries the claim's generation,
> the nonce,
> the slot's path and identity,
> the proposed clone name and the proposed `git-worktree-clone-origin-<nonce>` snapshot name,
> with phase `claimed`.
> Nothing has mutated yet.

Its grammar is one `key=value` line per field (binding design,
"The binding"):

> claim=<generation>
> nonce=<32 lowercase hexadecimal digits>
> phase=<claimed|registered|snapshotted|cloned|mounted|filled>

and `registration=<lowercase hexadecimal of the registration's path bytes>` is written at phase `registered`,
after Git's add returns (binding design,
"What the writer confirms before each write"):

> Then phase `registered`,
> with `registration=`,
> `registrationIdentity=` and `pointer=`.

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
and "Questions only the project can answer",
Q1,
keeps the other two answers reachable.

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
`done:` unlinks `locked` first (`builtin/worktree.c:600` to `:605`),
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
G10 as Git alone.

F2.
`git worktree add --lock --reason <text>` writes `<text>` plus one newline as the registration's first lock,
before the `gitdir` file and the slot's `.git`,
and keeps it.
Source:
`builtin/worktree.c:502` to `:546`,
`:600` to `:605`;
P1,
P2,
G1 and G13 as Git alone.

F3.
A plain `git worktree add` also writes a lock during the add,
the word `initializing`,
and removes it before it returns unless `--lock` is given.
Source:
`builtin/worktree.c:526` to `:530`,
`:600` to `:605`;
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
G10.

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
G10 (a torn lock and an empty lock both protect).

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
`:600` to `:605`.

F12.
`git worktree unlock` removes any lock without checking its words,
and `git worktree lock` with no reason writes an empty file that still protects.
Source:
`builtin/worktree.c:1180` to `:1248`;
P6,
G8,
G10.

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
"The binding" and "What the writer confirms before each write";
read as the sibling's unreviewed design,
not as a decision.

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

A1.
The binding's `nonce=` is on disk before the add's Git invocation runs,
survives until the registration is settled,
and is flushed with the write that made it.
This is the binding design's stated order and durability;
this design depends on it and would break if a write of the binding were lost.

A2.
The registration's `gitdir` file is written by the same Git call that writes the slot's `.git` (`builtin/worktree.c:540` to `:546`),
so the two arrive together,
and a registration with a `gitdir` and no slot pointer is a state only the account can make.

A3.
A removal's destruction proof can say "the clone is gone" before the Git settle runs,
as [plan][plan] step 16 states it.

A4.
The slot's directory on the checkout dataset is empty once the clone is destroyed,
as [proposal][proposal] step 5 requires,
so removing it changes nothing of a person's.

A5.
The request nonce is unique per add,
so a lock whose words carry it names one add and no other.

A6.
A scan of `worktrees/*` that finds the composed sentence in exactly one `locked` file has found the add that
wrote it,
because only that add could know the nonce,
and two registrations carrying the same sentence would mean a copied word,
which the design reports rather than choosing between.

A7.
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
`git worktree list` does not show it (F1),
and only the words identify it (S8).
An abandoned add leaks it until its own resume,
the next holder of that slot's claim,
or a person removes it,
and the next add at that name gets `worktrees/<name>1` (Git's counter,
`builtin/worktree.c:505` to `:512`).

L10.
Git's own failure path unlinks `locked` before it deletes the registration and the destination (F11),
so a kill inside that moment leaves an unprotected partial state that neither the lock nor this design covers.

L11.
A failing add can delete a pre-existing empty slot itself (F11),
so a slot the wrapper made may not be there after Git fails,
and the binding's recorded slot identity may no longer answer (the binding design's M6).

L12.
The lock's proof needs the binding:
a binding that cannot be read makes every lock foreign,
and a removal refuses on an unreadable binding anyway.

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

- It is the only route whose lock exists for the whole life of the registration.
  A separate `git worktree lock` leaves the registration unlocked from the moment the add returns until that
  command runs,
  and a kill in that gap leaves the state item 6 measured as prunable.
- The wrapper writing the file itself would need Git's private id derivation (the sanitized basename and the counter
  suffix,
  `builtin/worktree.c:496` to `:501`),
  which is Git's to change,
  and would write after the same return.
- Upstream calls `--lock` the equivalent "without a race condition" (F2,
  `Documentation/git-worktree.adoc:246` to `:248`).

### The order of the add's writes

The order is stated once,
here,
and each step names the document that owns it:

1.  The slot claim is taken (`take_claim`,
    the binding design's "The claim").
2.  The binding's first write,
    with `nonce=` and phase `claimed`,
    is made before Git runs (the binding design's order).
3.  `git worktree add --no-checkout --lock --reason <composed>` runs,
    and Git writes the registration (this design,
    V1).
4.  Git returns;
    the pointer bytes and the `gitdir` back-link are read,
    and the binding's `registered` write records
    `registration=`,
    `registrationIdentity=` and `pointer=` (the binding design's order).
5.  This design writes nothing of its own at any point;
    there is no lock record to place,
    order or flush.
6.  The clone is made,
    mounted and filled under the binding's later phases,
    and the lock stands through all of them.

### The moments of an add

The route is `--no-checkout` (step 2),
so the only long part of the add is Git's own preparation.
Each moment is named D0 to D7 and says what stands on disk,
which record proves what,
and what each prune can do there.

D0.
Before the invocation:
the binding's first write stands in the wrapper's record folder with its `nonce=`,
and the slot claim is held.
Nothing in `worktrees` names this slot,
so there is nothing for a lock to protect yet.

D1.
Inside Git,
after `worktrees/<id>` exists and before `locked` is written (`builtin/worktree.c:502` to `:530`):
a bare administrative directory.
Any prune removes it at once,
because it holds no `gitdir` (F5),
and so do `git gc` and the automatic maintenance.
A SIGKILL or power loss leaves the bare directory;
the add's next `write_file` then fails and the add exits through its failure path,
which unlinks the lock it never wrote and removes the directory (F11).
A signal Git catches runs the same handler (F11).

D2.
After `locked` holds the composed sentence,
before the `gitdir` file:
a locked administrative directory with nothing else.
No prune removes it (F4);
`git worktree list` does not show it (F1).
The lock's bytes equal the sentence recomputed from the binding's `nonce=`,
which is what attributes this directory to this add;
`registration=` does not exist yet,
so the add is found by the scan of "How doctor and removal tell the states apart" (S8).
A kill here leaves a locked directory with no `gitdir`,
and L9 says who removes it.

D2a.
After the `gitdir` file,
before the slot's `.git`:
the directory's `gitdir` names `<slot>/.git`,
which does not exist yet.
The scan confirms the attribution by that name;
the wrapper's own registration proof still cannot run,
because its `same_file` needs the pointer (F14).

D2b.
After the slot's `.git`,
before the `commondir` file:
the pointer stands and its bytes are the ones the binding will record.
The wrapper's proof is still short one fact,
its `commondir` comparison (F14),
so this design's pointer-free variant (S1's proof) is what a reader uses until then.

D2c.
After the `commondir` file,
before the ref store:
the registration is one the wrapper's own proof recognises,
once the binding's `registered` write has recorded the pointer bytes.

D3.
After the ref store,
`HEAD` and logs,
and the add returns (`builtin/worktree.c:555` to `:567`):
a complete registration whose pointer stands in the empty slot.
Every prune keeps it (F4),
and so would they unlocked,
because the pointer exists (F5).
The wrapper reads the pointer and back-link,
and the binding's `registered` write follows (D4).

D4.
After the binding's `registered` write:
the binding names `registration=` and `pointer=`,
so every later reader has the two facts without a scan.
A kill before this write leaves the claim and the binding at phase `claimed`,
and the lock is still attributed by the nonce (D2).

D5.
After the clone exists and before the mount:
a verified locked registration with no dataset and then a clone the binding names.
Every prune keeps it.

D6.
After the clone is mounted and the copied Git administration is discarded,
before the pointer is installed in the clone:
the slot shows no `.git`,
because the mount covers the underlay pointer,
and the registration has no index.
An explicit prune,
`git gc` and the automatic maintenance all remove an unlocked registration here at once (F5,
F6,
F7).
The lock is the only thing that stops all three (P4,
P5).
A kill here leaves a mounted clone with no pointer,
a locked registration and the underlay pointer hidden;
the resume re-attaches,
verifies and installs the pointer.

D7.
After the pointer is installed and the fill is verified:
the lock stays,
and it stays through every unmount and restart (the decision's own words).

Two consequences worth stating plainly:

- The window between D5 and D6 is not exposed:
  the clone root still holds the copied `.git` directory,
  and `file_exists` counts any entry (F1,
  F5).
- The window D6 opens is reopened by every unmount that leaves the underlay pointer in place only when that pointer
  is deleted.
  Without the deletion the unmounted slot shows the pointer again,
  and the lock is belt to the pointer's braces.

Git's own failure path is the one state outside D0 to D7 that an add can leave:
on any failure after `locked` is written,
`locked` is unlinked and then the administrative and destination directories are removed (F11),
and a kill inside that moment leaves an unlocked partial registration (L10).
This design does nothing there,
because nothing it writes exists before that moment,
and the add's own resume treats the leftover by the state it finds.

### A failed add

This design does not remove a registration after a failed add.
The accepted text keeps the claim and cleans up only what is proven,
and a registration without a dataset is one of the resume states it names
("What a failed add leaves").
So for a failure after Git returned,
such as `ENOSPC` at the snapshot or the clone,
what stands is the claim,
the binding at its last phase,
the locked registration,
and the slot:
the add resumes at the same slot,
or the removal's settle against the binding discards it.
For a failure inside Git,
F11 leaves whatever Git left and nothing of this design's.
The alternative,
a rollback that releases the lock and empties the registration,
is Q2's B,
and this design does not take it.

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
  read by the binding design's reader (A1);
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
the binding's recorded slot path and identity,
the registration's `gitdir` file naming exactly `<slot>/.git`,
and,
when the `commondir` file stands,
its `commondir` naming this repository.
With no pointer it is weaker than the three-fact proof,
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
  and unique per add (A5).
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
  so a power loss that loses Git's unsynced `locked` keeps the nonce (A1,
  L12's converse).

The binding's identity fields carry a device number,
and the repository's own specification records that number as anonymous and per mount,
with no measurement that it survives a reboot or an export and import ([spec][spec],
lines 5589 to 5601).
This design uses the recorded registration identity in the settle only,
where a refused identity stops the settle instead of guessing,
and the lock's own proof uses no identity at all.
So the boot hazard is the binding design's to fix (its H3),
and this design's exposure is exact:
a reboot that changes the number makes the settle refuse until the binding's identity rule is fixed,
and leaves the lock unaffected.

None of these is a change to what the binding holds today,
by its own text:
they are the reason this design can keep no record of its own.

## How doctor and removal tell the states apart

The check order is the same in both,
and the words differ.

1.  Read the slot's binding.
    If it is absent or unreadable,
    no lock can be verified,
    and every lock found is S3 or S7.
2.  Read the registration,
    by `registration=` or by the scan (S8's paragraph).
3.  Read `locked`:
    absent,
    or a regular file with bytes,
    or an entry the read refuses (a link,
    a named pipe,
    a
    directory,
    an unreadable file).
4.  Recompose the sentence from `nonce=` and classify.

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
`locked`'s bytes are a proper prefix of the recomposed sentence,
which is the only shape a killed single write
leaves (F8).
Doctor prints:

```text
the clone slot <slot> holds the managed Git lock its add wrote for add nonce <nonce>, cut short at <n> of <m> bytes, and it still keeps the registration; the next holder of the slot's claim completes it
```

S3.
**A lock the wrapper cannot recompose.**
The bytes are neither equal to nor a prefix of the sentence,
or the binding is absent or unreadable so that nothing can be recomposed.
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
the clone slot <slot> holds no managed Git lock for add nonce <nonce>, so `git worktree prune` can remove its registration while the slot shows no .git and the registration has no index; the next command that holds the slot's claim writes the lock again
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
**A locked directory with no `gitdir`.**
A `worktrees/<id>` holds `locked` whose bytes equal the recomposed sentence for a nonce,
and holds no `gitdir` file,
which is what a kill at D2 leaves (L9).
Doctor prints:

```text
the worktrees directory holds a locked registration with no `gitdir`, <id>, whose words name add nonce <nonce>; no prune removes it and `git worktree list` does not show it; the add that wrote it, or the next holder of that slot's claim, removes it
```

The scan this state and D2 need is per slot:
for one slot whose binding has a nonce,
one pass over `worktrees/*/locked`,
reading each lock's first bytes with the same bound and comparing them with that one recomposed sentence.
It never needs the binding folder's listing,
it runs on a resume or a doctor pass and never on an add's waiting path,
and it is bounded by the number of registrations the repository has.

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
Under Q3's A,
the removal holds the slot's claim,
so nothing else can install a lock between its steps (T8).

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
  under Q3's A the removal installs the lock from the nonce and then releases it after the destruction proof,
  so the settle releases a lock that was there;
  under Q3's B or C the removal follows Q4.
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
  A dead add's locked directory has no registration to settle,
  so doctor reports it and the add's own resume or the next holder of that slot's claim removes it;
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
It runs after the clone's destruction is proven (A3),
and it is the authorized registration-cleanup step the decision names.
It never runs `git worktree remove`,
so no `--force` and no Git-side validation is involved,
and the slot is not a valid worktree at that point anyway (S5,
F9).

The order,
each step with what it proves:

1.  **Prove destruction.**
    The persisted plan and the live reads say the recorded clone is gone (A3).
2.  **Prove the registration.**
    Hold `worktrees`,
    the registration and the slot's parent open by descriptor,
    and prove the registration is the recorded one:
    the binding's `registration=` path and identity,
    the `gitdir` back-link naming the slot's `.git`,
    and the pointer bytes against `pointer=` when the pointer exists.
3.  **Prove the lock.**
    S1 or S2,
    from the recomposed sentence.
    A missing lock is S4,
    and under Q3's A it is installed here,
    so this step then sees S1.
4.  **Release.**
    `unlinkat` the name `locked` through the held registration descriptor,
    and sync the registration directory.
    The unlink is one step:
    the lock is there or it is not,
    and no reader sees a half-released lock.
    The sync is what keeps a power loss from bringing the lock back beside a registration whose files this settle
    already removed.
5.  **Empty the registration** through the held descriptor:
    the wrapper's own records and pool,
    and the destruction plan sibling's pointer file inside the registration,
    then Git's files (F15).
    The `locked` name is already gone,
    which is the ordering this design adds to that step.
6.  **Remove the registration directory** by name from the held `worktrees` directory,
    which succeeds only when it
    is empty (F15).
7.  **Remove the empty slot** on the checkout dataset,
    only with the recorded identity and only when it is empty (A4).
8.  **Record the settle** in the removal's own records.

Why release before emptying:

- A kill after the emptying and before the release would leave a locked administrative directory with no `gitdir` and no `HEAD`.
  Prune,
  `git gc` and the automatic maintenance would never remove it (F4),
  and `git worktree list` would not even show it (F1,
  P3),
  so only the scan could find it (S8).
- A kill after the release and before the emptying leaves an unlocked,
  partly emptied registration of a destroyed
  clone.
  Prune removes it once the pointer is gone or the index is old,
  `git gc` and the automatic maintenance do the same,
  and doctor reports it as S4 on a destroyed clone.
  Nothing is lost by that removal,
  because the clone is already gone.

### Who may install a lock

Only a command that holds the slot's claim installs a lock,
and the removal's settle holds it while it releases.
That is what keeps Q3's A from racing the settle:
without the claim,
a command could see S4 after the settle's step 4 and write `locked` again,
leaving the registration locked and non-empty for step 6 (T8).
The slot claim is the wrapper's own per-slot claim ([spec][spec],
"The per-slot claim"),
which the removal holds (the binding design's V9),
so this rule adds no new lock and no new record.

### What a kill between the settle's steps leaves

- Before step 4:
  a verified lock on a destroyed clone.
  Doctor reports S1 with the clone named as gone,
  and the resumed removal,
  or the kept cancel command,
  releases it.
- Inside step 4 (a kill before the unlink,
  or after it and before the sync):
  the lock is there or not;
  a reboot may show it again only if the sync did not run,
  and then step 5 had not run either,
  so the state is the first bullet.
- After step 4 and before step 6:
  an unlocked registration,
  whole or partly emptied,
  of a destroyed clone.
  Inert,
  prunable,
  and reported as S4.
- After step 6 and before step 7:
  a stale pointer in an empty slot,
  with no registration.
  Doctor reports S6,
  and the removal's resume removes the folder.
- After step 7 and before step 8:
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
- The command holds the slot's claim ("Who may install a lock"),
  so the install cannot race the settle;
  the lock itself orders nothing (F19).

Who may install it is Q3's question,
because an automatic install answers a person's own `git worktree unlock` by undoing it.
Until that answer comes,
the design is written for A:
any command that holds the slot's claim and proves the registration installs it and says so once,
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

- The reason's composition:
  one module exporting the prefix,
  the suffix,
  the bytes for a nonce and the byte bound,
  so the add,
  the recovery,
  the scan and `doctor` all use one spelling.
- The invocation's two arguments and the add's use of the binding's nonce:
  in the ZFS add route step 10 builds,
  beside the binding's first write it already depends on.
- The classification:
  a function with the states of "How doctor and removal tell the states apart" as its answer,
  used by `doctor`,
  by the removal's preflight,
  by its second look and by the settle,
  so no caller invents its own order.
- The two-fact registration variant:
  beside `worktree/worktree_registration_proof.rs`,
  as a second entry point that takes the recorded slot,
  its path and identity,
  and the `gitdir` back-link,
  and never needs the pointer.
- The scan:
  beside the classification,
  one pass over `worktrees/*/locked`,
  on a resume or a doctor pass only.
- The release and the emptying:
  the existing registration emptying (F15),
  with the `locked` unlink and the directory sync added as its first step,
  in plan step 16's settle.
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
  composed from the binding's nonce;
  no record of its own.
- Step 12:
  the presence of `locked` in the slot report,
  unclassified.
- Step 13:
  the reason's module,
  the classification,
  the two-fact proof variant,
  the scan,
  the install of a lost lock,
  and `doctor`'s lines.
- Step 16:
  the settle's release ordering and emptying,
  which this design's "How the wrapper's own removal releases
  the lock" specifies.

## Tests a build must have

Each race is named with how a test forces it without timing luck.
A `hook` below means a test-only seam of the kind the wrapper already uses
(`#[cfg(test)] AFTER_THE_STAGING_FILE_IS_WRITTEN` in `foundation/private_storage.rs:432`).

Step 10:

T1.
The add's invocation carries `--lock` and `--reason` with the bytes recomposed from the binding's `nonce=`,
and no separate `git worktree lock` runs.
Forced by a seam that records the argument list the add hands Git,
with a real-Git arm that then reads `locked` and compares it to the recomposition (F2).
T3.
Each kill state is recognised by doctor and by removal,
not only by the add's resume.
Forced by building the state by hand for D1,
D2,
D2a,
D2b and D4,
from the measured shapes (P1,
P3,
P6):
a D4 state must read S1,
never S3 or S7.
T4.
A registration at D4,
with the binding at phase `claimed` and the lock present,
is adopted as S1 by the next command that holds the claim.
Forced by the fixture of T3 with the binding's first write present.
T5.
A concurrent `git worktree prune` during the fill does not remove the registration.
Forced by a seam between "discard the copied Git administration" and "install the pointer in the clone",
which runs `git worktree prune` in the main checkout;
a locked arm must keep the registration and an unlocked control arm must lose it,
which proves the seam can see the difference (host arm;
the mounted-clone arm is M1).
T6.
`git gc` and `git maintenance run --auto` during the fill behave as T5,
with the same seam and control arm.
T7.
A failed add after Git returned keeps the claim,
the binding,
the locked registration and the slot,
and a later add at the same slot resumes (Q2's A).
Forced by failing the snapshot or clone step through the add's own injection point.
T8.
An install cannot race the settle.
Forced by a seam after the settle's step 4,
where a second command tries to install:
the second must refuse for want of the slot claim,
and the settle must finish its steps 5 to 8.

Step 13:

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
T11.
The release order:
a seam after the `unlinkat` and before the emptying,
and one after the emptying and before the directory removal,
each of which leaves the state "What a kill between the settle's steps leaves" names,
with `git worktree prune`,
`git gc` and `git maintenance run --auto` run against it.
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
each expected as P6 and G10 measured.
T14.
`git worktree repair` after pointer loss on a host fixture leaves the lock and returns the pointer (F10),
so `doctor`'s S5 words stay true.
T15.
The two-fact registration proof answers for S5 (no pointer) and D2 (no `registration=`),
and refuses a registration whose `gitdir` names another slot or whose `commondir` is another repository.
T16.
The scan finds exactly the one registration carrying the sentence,
and reports two when two stand (A6).

Which need the guest:

- M1 and M2 (below) are the only arms whose subject is a mounted clone,
  and they are guest tests.
- T5 and T6 have a host arm and a guest arm;
  the host arm fixes the seam,
  the guest arm the mount.
- T15's no-pointer arm is a host fixture;
  the mounted arm is M2's.
- Every other test above runs on the host,
  in a throwaway repository,
  against the host's Git and the built wrapper.

## Measurements owed

M1.
**The fill window on a mounted clone,
in the guest.**
With the checkout mounted from the file-backed pool and a clone mounted at the slot,
and the copied Git administration discarded so the slot shows no `.git`:
run `git worktree prune`,
`git gc` and `git maintenance run --auto` in the main checkout,
once with the managed lock and once without,
and record which registration survives each of the six runs.
The contract:
the unlocked arms lose the registration,
the locked arms keep it.
Host fixtures already show the same on a plain directory,
through the wrapper (P3 to P5) and as Git alone (G2 to
G4);
the guest adds the mount covering the underlay pointer,
which is the state D6 describes.
This is the one measurement that decides whether the design's single reason for the lock holds on a real clone.

M2.
**`git worktree repair` over a mounted and an unmounted clone slot,
in the guest.**
Where the rewritten pointer lands in each case,
whether the lock file is touched,
and what `git worktree list --porcelain` and `git status` say afterwards.
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

V1.
**The write route**:
`--lock --reason` inside the add's own Git invocation (adopted),
or a separate `git worktree lock` after the add,
or the wrapper writing `locked` itself.
Example:
an add at `.claude/worktrees/one` that is killed just after Git accepts it.
Under the adopted route the registration is locked from Git's own creation of it.
Under a separate `git worktree lock` the registration is unlocked in that window,
with the pointer present,
so prune keeps it and `git gc` keeps it while the index is young;
the loss shows after the clone mounts over the pointer (D6).
The wrapper writing the file itself is the same window plus Git's private id derivation.
Pros of the adopted route:
no window the wrapper adds,
upstream's own answer to the race,
no private name derivation (F2).
Cons:
the reason must be known before the add,
so it must be recomputable from the binding's nonce (A1).
Ranking:
`--lock --reason` > a separate `git worktree lock` > the wrapper writing the file,
because the first removes the window,
the second removes only part of it,
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
**The release order in the settle**:
unlink `locked` first and then empty (adopted),
or empty and release last,
or rename the whole registration out of `worktrees` in one step and delete it under a wrapper name.
Example:
a removal killed after the first step of its Git settle.
The adopted order leaves an unlocked,
partly emptied registration of a destroyed clone,
which every prune removes.
Emptying first can leave a locked directory with no `gitdir`,
which no prune removes and no listing shows (F4,
F1).
The rename leaves nothing Git can see from the first step,
at the cost of a new name and a new leftover for
`doctor`.
Pros of the adopted order:
no new name,
the existing emptying,
and a harmless leftover.
Cons:
a leftover that only `doctor` names until a prune takes it.
Ranking:
unlink first > rename out > empty first,
because a leftover prune can remove beats a leftover only the wrapper can remove,
and no new name beats a new one while the existing emptying serves.

V4.
**The torn lock**:
the wrapper completes its own cut-short lock in place (adopted),
or leaves it and reports,
or refuses the slot until a person acts.
Example:
a kill inside Git's write of the reason,
leaving the first 20 bytes.
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
because the bytes are provably the wrapper's own (a prefix of its recomposed sentence),
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
13 and 16**:
the plan's suggestion,
with step 16 owning the release (adopted);
or keeping the add route refusing until step 13;
or putting all lock work in step 13.
Example:
the first build order merges the add and its lock together and the settle later;
the second holds the whole ZFS add route back until step 13 lands;
the third ships an add whose registrations are unprotected until step 13.
Pros of the adopted split:
the lock lands with the registration it protects,
and the pointer half is free of the
lock's classification.
Cons:
step 10 carries a lock whose classification,
recovery and release land two steps later.
Ranking:
adopted > all in step 13 > the add refusing until step 13,
because the first builds the lock with the state it protects,
the second leaves an unprotected window that step 10's own kill tests would have to accept,
and the third holds the route back for nothing it protects.

## Questions only the project can answer

These are the review's merged questions where they concern this design,
renumbered.

Q1.
**Where is the proof that a Git lock is the wrapper's,
and what does the claim carry?**
Asked in plain words:
when a person runs `git worktree unlock` and `lock` again,
or a kill leaves a lock,
how does the wrapper know the lock is its own?

- A.
  A second record inside the registration holds the exact lock bytes;
  the claim carries nothing new.
  Example:
  a person runs `git worktree lock --reason held` on a managed slot;
  `locked` reads `held`,
  the record reads the sentence;
  doctor prints "locked by words its add did not write".
  Pros:
  needs no change to `take_claim`,
  and dies with the registration.
  Cons:
  a second home and a second durability rule (written unsynced,
  so a power loss can keep `locked` and drop
  the record,
  which reads as a foreign lock and refuses the removal);
  a third file in the registration beside the wrapper's two and the destruction plan sibling's pointer.
- B.
  No second record:
  the lock sentence is a pure function of the binding's `nonce=`,
  and the proof is "`locked` equals the sentence recomposed from the recorded nonce,
  in the recorded registration"
  (the design this document is written for).
  Example:
  the same `held` lock reads as foreign;
  a killed add at D4 is recognised by the binding's nonce with no extra write.
  Pros:
  one record family,
  one durability policy (flushed),
  D2 and D4 attribution for free,
  and a record that survives the registration's loss.
  Cons:
  lock verification depends on the binding being readable (an unreadable binding makes every lock foreign,
  but the removal refuses on an unreadable binding anyway);
  and the binding design must state the rule.
- C.
  The claim file carries the nonce and bytes.
  Example:
  `<slot>.claim.1` gains two lines.
  Pros:
  the evidence exists before Git runs by construction.
  Cons:
  changes the shared claim primitive and its fuzz decoder,
  and the claim is rewritten as `released` and loses the fields.

Ranking:
B > A > C.
B beats A because it removes a file,
a durability mismatch and the unprovable power-loss state while keeping the
proof a byte comparison.
A beats C because it leaves the shared primitive alone.
The writer's first draft ranked A first because it was self-contained;
it ranks second here because its self-containment buys a record the binding makes redundant,
and because the claim cannot carry the bytes at all (H1,
F17).

Q2.
**After a failed add that made a registration but no clone,
does the add remove its registration,
or keep it?**
Asked in plain words:
the snapshot step returns "no space";
what stands afterwards?

- A.
  Keep the claim,
  the locked registration and the slot,
  report,
  and resume or discard explicitly
  (the accepted text,
  proposal 427 to 431;
  the design this document is written for).
  Example:
  `git worktree add .claude/worktrees/one` prints the operation,
  object and pool,
  and a later add at `one` resumes;
  `doctor` shows "registration without a dataset".
  Pros:
  matches the accepted words,
  destroys nothing,
  reuses the registration's recorded identity.
  Cons:
  a locked registration that Git cannot remove without two forces stays until a person resumes or discards;
  the discard path (the removal's settle) must exist.
- B.
  Roll back:
  release,
  empty,
  drop the claim.
  Example:
  the same failure ends with no registration and an empty slot
  (Git itself deletes a slot it did not create,
  F11).
  Pros:
  leaves no half state.
  Cons:
  contradicts "keep the claim";
  loses the evidence that explains the failure;
  a kill in the rollback is a new state this design would then have to classify.
- C.
  Keep on `ENOSPC` (as named in the proposal),
  roll back on other failures.
  Example:
  `ENOSPC` keeps;
  a refused mount rolls back.
  Pros:
  follows the only failure the text names.
  Cons:
  needs a failure taxonomy no record gives.

Ranking:
A > C > B.
A is the accepted behaviour;
C is the same for the named case and adds an unwritten rule for the rest;
B overrides an accepted sentence with no record behind it.

Q3.
**May the wrapper install a lost managed lock without being asked?**
Asked in plain words:
a person ran `git worktree unlock` on a managed slot,
or a kill left the lock missing;
does the wrapper put it back by itself?

- A.
  Install in any command that holds the slot claim and proves the registration,
  and say so once
  (the design this document is written for).
  Example:
  a person ran `git worktree unlock .claude/worktrees/one` yesterday;
  today `git worktree remove .claude/worktrees/one` writes the lock again,
  prints that it did,
  and continues.
  Pros:
  the protection returns before the window that needs it (the fill window with no pointer).
  Cons:
  undoes a person's deliberate unlock without asking.
- B.
  Only through a new explicit command.
  Example:
  `doctor` prints "no managed lock;
  run `git monochromatic-git-policy-cli repair-worktree-lock one`"
  and a removal refuses until it is run.
  Pros:
  no surprise,
  and no write the person did not ask for.
  Cons:
  a new command to build and learn,
  and an ordinary removal refuses after an interrupted write.
- C.
  Never;
  report only.
  Example:
  `doctor` prints the missing lock and a removal refuses until the person runs `git worktree lock` by hand.
  Pros:
  the wrapper never writes a lock after the add.
  Cons:
  the lock's one purpose,
  prune protection,
  is lost until a person acts,
  and the fill window is when nobody watches.

Ranking:
A > B > C.
A beats B because A restores the protection at the moment the wrapper already proves everything the install needs,
while B makes a rare,
easily forgotten state need a command.
B beats C because B at least offers a one-line way back,
while C leaves the registration unguarded and refuses
removal until the person works it out.
Ask it after Q1,
because Q1 decides what "proves" means.

Q4.
**Does a removal proceed when the managed lock is missing?**
Asked in plain words:
the removal is about to destroy a registration and finds the wrapper's lock already gone;
does it stop?
Ask only if Q3 is not A;
under Q3-A the removal installs the lock and releases it,
and the question is moot.

- A.
  Proceed with one notice and a fresh registration proof before the settle.
  Example:
  `git worktree remove .claude/worktrees/one` prints
  "the clone slot one held no managed Git lock when this removal settled it,
  so nothing was released" and removes.
  Pros:
  no refusal for a protection that is about to be released anyway;
  one run.
  Cons:
  a person who unlocked on purpose for another reason is not stopped.
- B.
  Refuse until the lock is back.
  Example:
  the same command refuses with
  "the clone slot one holds no managed Git lock;
  run the removal again and this wrapper writes it first"
  (which presumes Q3-A).
  Pros:
  every settle releases a lock that was there.
  Cons:
  two runs for one removal,
  and the first refusal reaches everyone with an interrupted write.

Ranking:
A > B,
because the lock protects a registration the removal is destroying,
so its absence costs the removal nothing,
while a refusal costs every affected removal a second run.

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
4. **The key.**
   This design keys the lock on the slot through the binding and keeps no key of its own.
   The binding keys on the slot.
   The destruction plan keys on the clone GUID,
   and it also keeps a pointer file inside the registration
   (`<registration>/monochromatic-git-policy-cli-clone-destruction-plan`),
   which this design's settle must empty with the rest (step 5).
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
   This design:
   never `git worktree remove`;
   unlink,
   empty by descriptor,
   remove by name,
   because the proposal refuses falling through to Git's recursive removal for a managed slot (lines 811 to 813).
   The binding design's A4 and "The record's end" step 3 say "Git removes with its folder (F2)".
   The proposal's sentence is the accepted design's and decides it,
   so this design's settle stands,
   and the binding design's sentence is the one to change.
8. **What a failed add leaves.**
   This design now keeps the claim,
   the binding,
   the locked registration and the slot (Q2's A),
   which agrees with the binding design's T22 and with the proposal.
9. **How a resume finds the registration after a kill before `registered`.**
   This design:
   the scan over `worktrees/*/locked` for the recomposed sentence (S8's paragraph,
   A6).
   The binding design's M6 asks how a resume finds it;
   this is the answer for the lock's own use,
   and the binding design may use it too.
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
    This design's D6 falls inside the binding design's `mounted` to `filled` span,
    which has no phase of its own.
    The lock covers that window whatever the binding's phase says,
    and the binding design's resume reads the live facts there.
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
    that choice is theirs and the review's Q3,
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
  `:600` to `:605` (the lock's removal),
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

The sibling designs,
read by `git show` of their branches and treated as unreviewed:

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
- `doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`.

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
- The commit that carries this list is the one after `71de819090` on this branch,
  and `git log` reports its hash and time.

## Links

[decision]: ../decision/cli-git-zfs-worktree-design.md
[proposal]: cli-git-zfs-worktree-design-proposal.md
[plan]: cli-git-zfs-worktree-implementation-plan.md
[measurements]: cli-git-zfs-measurements-owed-before-acceptance.md
[staging]: ../decision/cli-git-zfs-staging-under-cache.md
[sweepchoices]: ../decision/cli-git-zfs-step-8-sweep-choices.md
[slotstore]: ../decision/cli-git-zfs-slot-identity-store-format.md
[spec]: ../../package/git-policy/cli/SPEC.md
[git-worktree]: https://github.com/git/git/blob/v2.56.1/Documentation/git-worktree.adoc
[git-layout]: https://github.com/git/git/blob/v2.56.1/Documentation/gitrepository-layout.adoc
[git-relnotes]: https://github.com/git/git/blob/v2.56.1/Documentation/RelNotes/2.33.0.adoc
[vm-prune]: cli-git-zfs-measurements-owed-before-acceptance.md#6-deleted-underlay-pointer-prune-and-lock
