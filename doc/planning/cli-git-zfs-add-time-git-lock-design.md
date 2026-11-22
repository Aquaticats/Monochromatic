# The add-time Git lock of a ZFS clone worktree

Non-final,
unapproved design proposal by an agent.
It is not approved by the project owner and nothing in it is final.
It designs the open item "The add-time Git lock" of the accepted ZFS worktree design ([decision][decision],
"Decision"),
and it makes no decision that design,
the proposal,
or the project's answers made,
beyond the writer's choices that "Choices this design marks for the project's veto" lists and the two questions that "Questions only the project can answer" puts.
It ran no ZFS command that changes anything,
started no virtual machine and no guest,
touched no pool,
mounted nothing,
opened no session transcript,
and edited no file but this document.
The one container it ran was this repository's gate image,
already present,
to run Git alone on throwaway repositories inside it,
with no folder of the home mounted;
no image was built or removed.

It read no guest.
Every kernel,
ZFS or mount fact it rests on is one the repository already measured,
cited to the measurement record,
and every claim that needs the guest is written as unmeasured under "Measurements owed".
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
and the wrapper adds its own steps to `git worktree add` and `git worktree remove` ("What the wrapper adds to `git worktree add` and `git worktree remove`).
So the probes ran twice:
once on this host through the wrapper,
and once inside the repository's gate image,
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
  for the no-configuration answer.
- [`doc/decision/cli-git-zfs-slot-identity-store-format.md`][slotstore] (179 lines),
  as the precedent for a small record that takes no lock.
- [`package/git-policy/cli/SPEC.md`][spec],
  "Doctor",
  the removal route and its accepted risk,
  the descriptor mount of a ZFS worktree clone,
  the shadow section that names Git's automatic maintenance,
  and the credential rule.
- The wrapper's own code under `package/git-policy/cli/src/native/`:
  `worktree/worktree_added_by_the_wrapper.rs`,
  `worktree/worktree_registration_proof.rs`,
  `worktree/worktree_removal_second_look.rs`,
  `worktree_clone_mount/clone_slot_git_pointer.rs`,
  `foundation/private_storage.rs`,
  `lock/single_owner_claim.rs` and `health/doctor.rs`.

## What was run

### Through the wrapper, on this host

Plain `git` (the installed wrapper,
`0.6.1-alpha.44`,
shadowing `/usr/bin/git`,
Git `2.56.1`,
built 2026-11-17),
in throwaway repositories under `/home/user/temp/agent/zfs-add-time-git-lock-design-2026-11-22/scratch/probe2` to `probe8`,
each with its own `HOME` and `XDG_RUNTIME_DIR` under that scratch folder,
each script in its own capped `systemd-run --user --scope`,
and the fixtures' one commit made through plumbing (`hash-object`,
`mktree`,
`commit-tree`,
`update-ref`) because the wrapper's commit route needs this session's commit service,
which a private `XDG_RUNTIME_DIR` cannot reach.
Every `git worktree add` in these runs therefore shows the wrapper on top of Git:
it copied ignored entries (there were none),
wrote `monochromatic-git-policy-cli-copy-owner` and `monochromatic-git-policy-cli-worktree-added` beside the registration (F13),
and took its own add path with its own checks.
The wrapper's cheap ladder,
whose accepted forms include `--lock` and `--reason` ([spec][spec],
line 1162),
did not run in these fixtures:
no stage claim or destination record was written under the common directory.
`git worktree move` is forwarded by the wrapper unchanged ([spec][spec],
lines 873 to 874),
and `git worktree prune` and `git gc` are forwarded too,
so those runs show Git's own answers.

### As Git alone, inside the repository's gate image

`localhost/git-policy-native-test:zfs-step-9-unmount-2026-11-22` was already present (`podman images` listed it),
so the same arms ran there with `podman run --rm --network none --cpus=2 --memory=2g --pids-limit=256`,
no folder of the home mounted,
on a throwaway repository made inside a named volume that the probe created and removed,
with the fixtures' commit made by `git commit` inside the image.
`git --version` there printed one line,
`git version 2.56.1`,
where this host's prints the wrapper's version first,
so `git` in the image is Git itself.
No image was built and none was removed.
Its arms are G1 to G11 below.

### The arms, and what each showed

The probe runs on this host,
and what each showed:

- **P1**,
  the lock an add writes and leaves.
  A plain `git worktree add --no-checkout` leaves no `locked` file.
  `--lock --reason <text>` leaves `<text>` plus one newline,
  mode `0644`,
  and `git worktree list --porcelain` reports `locked <text>`.
- **P2**,
  sampling the administrative directory every millisecond through an add of a 4,000 file checkout.
  A plain add holds `locked` with `initializing` for the whole run and ends with no lock;
  a `--lock` add holds the given text from the first sample and keeps it.
  No sample showed the administrative directory present without its lock.
- **P3**,
  explicit `git worktree prune` on 2.56.1.
  A locked registration with the slot's `.git` deleted survives;
  unlocked,
  the same registration is pruned at once with `gitdir file points to non-existent location`.
  An administrative directory holding only a `locked` file survives and is not listed by `git worktree list`;
  one holding nothing is pruned at once.
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
  It prevents both prunes and `git gc`,
  `git worktree lock` refuses over it and repeats its words,
  and every word of the design's rule about presence holds for it as for Git's own file.
  A torn lock,
  a prefix of the intended text,
  and an empty lock both still protect,
  and `git worktree remove` refuses with the torn words or with no words.
- **P7**,
  `git worktree move` and `git worktree remove` on a locked worktree with its pointer present.
  Both refuse plain and with one `--force`,
  both succeed with two forces,
  and the double-force removal deletes the slot directory and the registration.
- **P8**,
  double force against a locked registration whose pointer is missing.
  Even two forces fail:
  `fatal: validation failed, cannot remove working tree: '<slot>/.git' does not exist`.
- **P9**,
  `git worktree repair` with the pointer missing and the administrative directory present.
  It rewrites the pointer from the `gitdir` file and leaves the lock as it was.
  With the administrative directory pruned it restores nothing,
  with or without the slot named,
  and `git status` in a repaired but empty slot reports every tracked file deleted.
- **P10**,
  a post-checkout hook written into a fixture repository.
  It never ran:
  the wrapper does not run a repository's own hooks.
  P2 replaced it.

The same arms as Git alone,
in the gate image,
and what each showed:

- **G1**:
  `--lock --reason` writes the words plus one newline,
  mode `0644`,
  into a `--no-checkout` registration with no index,
  and `worktree list --porcelain` reports `locked <words>`;
  a plain add leaves no lock.
  This is P1 and P2's endpoint without the wrapper.
- **G3**:
  explicit prune keeps the locked registration with the pointer deleted,
  and removes it at once after `git worktree unlock`.
- **G4**:
  `git gc` keeps it locked,
  and removes it unlocked,
  with no index.
- **G5**:
  `git maintenance run --auto` does the same.
- **G6**:
  `git worktree remove` refuses plain and with one force,
  and two forces delete the registration and the slot directory.
- **G7**:
  with the pointer missing,
  two forces still fail validation.
- **G8**:
  `git worktree repair` rewrites the pointer from the `gitdir` file and leaves the lock.
- **G9**:
  `git worktree lock` with no reason writes a zero-byte file that still protects.
- **G10**:
  an administrative directory holding only a lock is kept by prune and is not listed by `git worktree list`.
- **G11**:
  a lock file written by `sh`,
  with no Git command involved,
  protects exactly as Git's does,
  `git worktree lock` refuses over it and repeats its words,
  and `git gc` keeps it.

Every arm P1 to P9 records agreed with its Git-alone twin on this host,
with three differences,
all the wrapper's own:
the wrapper's version line,
its ignored-entry copy message,
and the two records it writes beside a registration.
The wrapper's refusal of a locked worktree in its own btrfs removal route was not exercised,
because these fixtures are on ZFS and were never btrfs subvolumes.

Nothing was measured in a guest,
on a pool,
or through a mount.
The scripts and their exact printed lines are kept in `~/temp/agent/zfs-add-time-git-lock-design-2026-11-22/keep/git-2.56.1-lock-probes/`.

## The open item and the answer it carries

The open item,
quoted whole ([proposal][proposal],
"Open items the decisions leave"):

> The add-time Git lock.
> The answer's words:
> "Git lock + pointers (Recommended)":
> a lock is taken at registration and kept through unmounts and restarts.
> How the wrapper writes that lock,
> what reason it records,
> and how `doctor` and removal tell it from an operator lock without reading the reason text ([Unmounted, pruned or moved worktrees](#unmounted-pruned-or-moved-worktrees)) are undesigned,
> and the add sequence takes the lock at registration without saying how ([Add and Git registration](#add-and-git-registration),
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
> The unselected root-set immutable flag would need removal and parent-rename proof and does not protect Git admin state.
> [VM item 6][vm-prune] measured the lock with an absent pointer.
> Explicit prune removed the unlocked entry immediately;
> the `git gc` grace period is not a safeguard against explicit prune.
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
in the add's own Git invocation ([proposal][proposal],
"Add and Git registration",
step 2,
lines 373 to 382):

> Claim the destination as the account with an exclusive per-add record in the Git common directory.
> Write the proposed clone name and unique `git-worktree-clone-origin-<nonce>` snapshot name before any mutation beyond the claim.
> Create or verify an empty account-owned slot,
> then run Git's supported `worktree add --no-checkout` form.
> Record the reciprocal registration and exact `.git` pointer bytes;
> leave it in the slot on the checkout dataset.
> Take the decided managed Git lock here,
> with the registration,
> and retain it through unmounts and restarts

The claim record's home is named,
the Git common directory,
and that is the only record of an add whose home the proposal names ([proposal][proposal],
"Add and Git registration",
lines 414 to 421):

> This design does not name where that binding and lineage record lives:
> only the per-add claim record has a named home,
> the account-owned Git common directory,
> and the account writes the binding record wherever it is placed

### What the lock may and may not do

The measured facts,
quoted ([proposal][proposal],
"Unmounted,
pruned or moved worktrees",
lines 439 to 449):

> The account can delete the underlay pointer while the clone is unmounted.
> In [VM item 6][vm-prune],
> Git `2.43.0` pruned the unlocked registration with a missing pointer on explicit `git worktree prune`,
> both without `--expire` and with `--expire now`.
> A lock set before pointer loss preserved the registration in both arms.
> Item 6 did not measure the `git gc` path.
> After pointer restoration,
> the unmounted locked worktree refused ordinary and single-`--force` move and remove;
> double force bypassed both,
> and Git removal left the dataset.
> This supports a lock as a prune guard,
> not as an authority boundary or a replacement for the wrapper's managed move refusal.
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
and the pointer half is plan step 12's ([plan][plan],
step 12,
lines 1637 to 1648):

> the underlay pointer kept for prune,
> independent of the clone's own pointer;
> the read-only distinction between a known unmounted clone and a pending add or removal;
> the report of slot,
> dataset and next safe action;
> the refusal of `git worktree move` for a bound managed clone,
> mounted or not

### When the lock goes

The release belongs to the settle,
and the settle does not wait for the origin ([proposal][proposal],
"Project decisions",
lines 1460 to 1466):

> Decided:
> "Settle + own record",
> against the ranking the question round carried,
> which put keeping the registration first:
> the registration is settled once the clone is destroyed,
> and a separate durable record carries the origin cleanup.

And "Removal and recovery" step 5,
quoted ([proposal][proposal]):

> Settle only the matching Git registration and pointer after clone destruction is proven.

### The threat model this design argues under

The project's own sentence,
quoted ([staging][staging],
"The handle record's place"):

> The threat model that answer states is the project's own for this machine:
> the account is trusted,
> and root and the account are not a security boundary here.

This design says where an argument survives that sentence and where a check exists only for a hostile account,
in "Hardening the stated threat model may make unnecessary".

### No configuration

The project answered that everything works with no configuration file ([sweepchoices][sweepchoices],
"What was decided",
lines 17 to 27):

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

### Lock-free first, and no unmeasured limit

`AGENTS.md` rule `NLK`:

> Concurrency is lock-free first (atomic rename,
> `O_EXCL`,
> one owner per item);
> an unavoidable lock covers the smallest unit,
> never a whole repository,
> with its reason written beside it.

`AGENTS.md` rule `LQM`:

> Shipped resource limits (unit caps,
> timeouts,
> size bounds) cite a workload measurement or owner decision beside the value

So this design ships no wait and no cap that it has not measured,
and the one byte bound it ships is derived from the bytes it writes rather than chosen.

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
no listing per sweep (Recommended)" in "The design accepted".
It is a precedent to weigh for this design's record,
not a decision about it.

## What Git 2.56.1 does

Every statement here is read at `v2.56.1` in the named file and line,
or measured in P1 to P10 through the wrapper and again in G1 to G11 as Git alone.

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
  and the cheap ladder's stage claim and destination record are written before real Git runs and rewritten when it returns ([spec][spec],
  lines 4410 to 4415).
- It copies the source's ignored entries into the new worktree and says so on standard error ([spec][spec],
  from line 867,
  "Linked-worktree ignored-state synchronization"),
  which P1 printed as `copied 0 ignored filesystem entries`.
- Its `git worktree remove` serves a btrfs subvolume worktree itself,
  refuses a worktree whose registration holds `locked`,
  and hands every other case to Git ([spec][spec],
  lines 3100 to 3290),
  with the lock lookup at line 3252.
  `git worktree move` is forwarded like any other command ([spec][spec],
  lines 873 to 874).

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
It writes the bytes with `write_file`,
which truncates and writes,
completes the line with one `\n`,
and creates the file `0666` less the umask (`wrapper.c:760` to `:783`;
P1 measured `0644`).
Nothing in Git records who wrote the file.

### Git writes the lock before it writes the registration

`git worktree add` creates `worktrees/<id>` (`builtin/worktree.c:496` to `:505`),
then writes `locked` (`:526` to `:530`),
and only then writes the `gitdir` file and the slot's `.git` (`:540` to `:546`).
The value it writes is `initializing` unless `--lock` was given,
in which case it is the `--reason` text (`:527` to `:530`,
`:879` to `:884`).
It removes that lock when the add fails or when `--lock` was not given (`:600` to `:605`).
Upstream states why `--lock` exists ([git-worktree][git-worktree],
lines 246 to 248):

> Keep the worktree locked after creation.
> This is the equivalent of `git worktree lock` after `git worktree add`,
> but without a race condition.

P2 measured both halves:
`initializing` for the whole of a plain add,
and the given text for a `--lock` add,
with no sample showing a registration without a lock.

A signal Git catches during the add removes the administrative directory and the worktree directory it made (`builtin/worktree.c:273` to `:288`,
`:515` to `:520`).
A SIGKILL and a power loss leave whatever was written,
and Git's own failure path removes the directory it made.

### What prune removes, and what stops it

`git worktree prune` lists `worktrees`,
and for each entry `should_prune_worktree` decides (`builtin/worktree.c:212` to `:247`,
`worktree.c:937` to `:1000`):

1.  An entry that is not a directory is pruned,
    with the reason `not a valid directory`.
2.  An entry holding `locked` is kept,
    whatever the file says,
    and it is not even offered to the duplicate check (`worktree.c:956` to `:958`,
    and the caller's `else if (path)` at `builtin/worktree.c:236` to `:238`,
    which skips it because a locked entry answers no path).
3.  An entry with no `gitdir` file is pruned at once,
    with `gitdir file does not exist`.
4.  An entry whose `gitdir` names a path that does not exist is pruned when the entry's `index` is missing or no newer than the expiry (`worktree.c:990` to `:999`).

The default expiry of an explicit `git worktree prune` is `TIME_MAX` (`builtin/worktree.c:255`),
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

P4 and P5 measured the two consequences that matter here:

- an unlocked registration with a missing pointer and **no index** is pruned at once by `git gc` and by `git maintenance run --auto`;
- an unlocked registration with a missing pointer and an index younger than three months survives both.

A `--no-checkout` registration has no index (P1),
so the grace period does not protect the window this design cares about,
and the lock is the only thing that does.

### What a person's own Git commands do

Measured through the wrapper (P3 to P9) and as Git alone (G3 to G8):

- `git worktree list` shows `locked`,
  with the reason on its own line in the porcelain form.
  An administrative directory holding only a lock is not listed at all.
- `git worktree move` refuses with `cannot move a locked working tree, lock reason: <text>`,
  refuses one `--force`,
  and succeeds with two.
- `git worktree remove` refuses the same way,
  refuses one `--force`,
  and succeeds with two only while the slot is still a valid worktree;
  with the pointer missing even two forces fail validation.
- `git worktree prune` and `git gc` keep a locked registration in every arm,
  and prune an unlocked one as described above.
- `git worktree repair`,
  with the administrative directory present and the pointer missing,
  rewrites the pointer from the `gitdir` file and leaves the lock untouched.
  With the administrative directory pruned it restores nothing.
- `git worktree lock` refuses over any existing lock and prints its words;
  `git worktree unlock` succeeds without any check of who wrote the lock,
  and removes the protection.
- `git worktree lock` with no reason writes an empty file,
  which still protects.
- `git worktree add` at a path already registered refuses with `already exists`.

## Facts

F1.
`worktrees/<id>/locked` is Git's only lock state:
its presence protects,
its bytes are a message.
Source:
`Documentation/gitrepository-layout.adoc:292` to `:297`,
`worktree.c:305` to `:326`,
`dir.c:3191` to `:3195`;
P1,
P3,
P6,
and G1,
G10,
G11 as Git alone.

F2.
`git worktree add --lock --reason <text>` writes `<text>` plus one newline as the registration's first lock,
before the `gitdir` file and the slot's `.git`,
and keeps it.
Source:
`builtin/worktree.c:496` to `:546`,
`:600` to `:605`;
P1,
P2,
and G1 as Git alone.

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
`worktree.c:956` to `:958`,
`builtin/worktree.c:236` to `:238`;
P3,
P6,
P7,
and G3,
G6,
G10,
G11.

F5.
Prune removes an entry with no `gitdir` file at once,
and an entry whose `gitdir` names a missing path at once when the entry has no index,
or after the expiry when it has one.
Source:
`worktree.c:960` to `:999`,
`builtin/worktree.c:255`;
P3,
P4,
P5,
and G3,
G4,
G5.

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
G4,
G5;
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
G4,
G5.

F8.
Git's lock file is written by truncating and writing,
with one newline completed,
at mode `0666` less the umask,
and a kill inside that write leaves a shorter file that still protects.
Source:
`wrapper.c:760` to `:783`;
P6,
G9,
G11 (a torn lock and an empty lock both protect).

F9.
`git worktree move` and `git worktree remove` refuse a lock,
refuse one force,
and succeed with two while the slot is a valid worktree;
with the pointer missing even two forces fail validation.
Source:
`builtin/worktree.c:1339` to `:1344`,
`:1451` to `:1456`;
P7,
P8,
G6,
G7.

F10.
`git worktree repair` restores a missing pointer from the surviving `gitdir` file and leaves the lock as it was,
and restores nothing when the administrative directory is gone.
Source:
P9,
G8;
`builtin/worktree.c` `repair` and `worktree.c` `repair_worktree`.

F11.
A signal Git catches during an add removes the administrative directory and the worktree directory Git made.
Source:
`builtin/worktree.c:273` to `:288`,
`:515` to `:520`.

F12.
`git worktree unlock` removes any lock without checking its words,
and `git worktree lock` with no reason writes an empty file that still protects.
Source:
`builtin/worktree.c:1195` to `:1248`;
P6,
G9,
G11.

F13.
The wrapper already keeps small records inside a worktree's administrative directory:
`monochromatic-git-policy-cli-worktree-added` holds the worktree's path and is written after Git accepts the add,
and `monochromatic-git-policy-cli-copy-owner` holds the identity of a copied destination.
Source:
`worktree/worktree_added_by_the_wrapper.rs:44` and its opening comment,
`:64`;
`worktree/worktree_add_binding.rs:597`.

F14.
The wrapper already proves that a directory is one of this repository's registrations from three facts:
the pointer names a directory directly below this repository's `worktrees`,
that registration's `gitdir` names the pointer back,
and its `commondir` is this repository.
Source:
`worktree/worktree_registration_proof.rs:1` to `:30`,
`:125`.

F15.
The wrapper already removes a registration by emptying it through a held descriptor and removing the empty directory by name from the held `worktrees` directory.
Source:
[spec][spec],
line 3377.

F16.
The wrapper already refuses a removal when a `locked` entry stands in the registration it would remove (`worktree/worktree_removal_second_look.rs:127` to `:129`),
which is right for the btrfs route and wrong for a registration the ZFS add locked itself.

## Assumptions

A1.
The per-add claim record that step 10 writes before the add can carry two more fields,
the claim's nonce and the exact lock bytes the add will write,
without changing its home or its purpose.
Its home is named ([proposal][proposal],
lines 373 to 375) and its format is step 10's writer work.

A2.
The registration the add records is the one the removal later holds open,
and Git's `worktree add` answers it before the wrapper's next step,
so the record of the lock can be written beside it.

A3.
A removal's destruction proof can say "the clone is gone" before the Git settle runs,
as [plan][plan] step 16 states it.

A4.
The slot's directory on the checkout dataset is empty once the clone is destroyed,
as [proposal][proposal] step 5 requires,
so removing it changes nothing of a person's.

A5.
A binding record (the sibling design's) may also carry the lock's bytes and nonce for reporting,
and nothing in this design decides ownership from it.

A6.
The claim's nonce is unique per add and short enough to sit inside one reason line without reaching any bound.

## Limits

L1.
The lock protects the registration only.
A person with two forces,
or with `unlock`,
or with a deliberate `rm` in the administrative directory,
ends it,
and this design does not try to stop that ([proposal][proposal],
line 451 to 453;
F9,
F12).

L2.
The lock says nothing about the clone,
the dataset or the pointer.
The pointer copies and the binding record carry that,
and the lock is not a substitute for either.

L3.
Nothing proves who wrote a lock file from the file.
This design's ownership answer rests on a record the wrapper wrote,
and under the project's threat model that record is not a security boundary,
only a durable fact.

L4.
The window inside `git worktree add` between its `mkdir` and its write of `locked` (F2,
F3) is Git's own and the wrapper cannot close it.
It is the same window every caller of `git worktree add` has,
and it is a few statements long.

L5.
A lock the wrapper installs after the fact has the same effect as Git's only because Git reads presence and never authorship (F1).
Nothing else about Git's behaviour changes.

L6.
If the oldest Git the wrapper accepts does not support `--lock --reason`,
the clone rung has no lock it can trust,
and this design refuses that rung rather than shipping a registration prune can remove mid-fill.
The floor is not established here;
"Measurements owed" names the check.

L7.
The one byte bound this design ships is the length of the reason it composes,
computed from the constant parts and the nonce.
It is not a measured limit and is not a wait.

L8.
Nothing here measures a mounted clone.
Every ZFS or mount sentence is the repository's existing measurement or is owed.

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
and the wrapper does not create `locked` itself at this point.

Why this route and not the other two,
with the alternatives ranked in "Choices this design marks for the project's veto":

- It is the only route whose lock exists for the whole life of the registration.
  A separate `git worktree lock` leaves the registration unlocked from the moment the add returns until that command runs,
  and a kill in that gap leaves exactly the state item 6 measured as prunable.
- The wrapper writing the file itself would need Git's private id derivation (the sanitized basename and the counter suffix,
  `builtin/worktree.c:496` to `:505`),
  which is Git's to change,
  and would write after the same return.
- Upstream calls `--lock` the equivalent "without a race condition" (F2,
  `Documentation/git-worktree.adoc:246` to `:248`).

### The moments of an add

The route is `--no-checkout` (step 2),
so the only long part of the add is Git's own preparation.
Each moment names what stands on disk and what each prune can do there.

M0.
Before the invocation:
the claim record stands in the Git common directory,
with the nonce and the reason bytes it is about to give Git.
Nothing in `worktrees` names this slot.

M1.
Inside Git,
after `worktrees/<id>` exists and before `locked` is written:
a bare administrative directory.
Any prune removes it at once,
because it holds no `gitdir` (F5),
and so does `git gc` and the automatic maintenance.
The wrapper cannot act here;
the moment is Git's own (L4).
A SIGKILL or power loss leaves the bare directory;
a signal Git catches removes it (F11).

M2.
After `locked` holds the wrapper's text,
before the `gitdir` file and the slot's `.git`:
a locked administrative directory with nothing else.
No prune removes it (F4);
`git worktree list` does not show it (F1,
P3);
a kill leaves it and the wrapper's claim attributes it by the nonce in the words (F2,
A1).

M3.
After `gitdir` and the slot's `.git`,
before the add returns:
a locked registration whose pointer stands in the empty slot.
Every prune keeps it (F4),
and so they would even unlocked,
because the pointer exists (F5).
A kill leaves it;
a signal Git catches removes it (F11);
the wrapper's resume reads it as a registration with no clone.

M4.
After the add returns,
before the wrapper writes the lock record beside the registration:
a locked registration whose evidence is the claim only.
A kill leaves it,
and the resume writes the record after checking the bytes against the claim (A1,
A2).

M5.
After the record,
before the clone exists:
a verified locked registration with no dataset.
Every prune keeps it.

M6.
After the clone is mounted and the copied Git administration is discarded,
before the pointer is installed in the clone:
the slot shows no `.git`,
because the mount hides the underlay pointer.
This is the window the lock exists for:
an explicit prune,
and `git gc` or the automatic maintenance,
remove an unlocked registration here at once,
because the entry has no index (F5,
F6,
F7).
With the lock,
all three keep it (P4,
P5).
A kill here leaves a mounted clone with no pointer,
a locked registration and the underlay pointer hidden;
the resume re-attaches,
verifies and installs the pointer.

M7.
After the pointer is installed and the fill is verified:
the lock stays,
and it stays through every unmount and restart (the decision's own words).

Two consequences worth stating plainly:

- The window between M5 and M6,
  the clone's own fill,
  is not exposed:
  the clone root still holds the copied `.git` directory,
  and `file_exists` counts any entry (F1).
- The window M6 opens is reopened by every unmount that leaves the underlay pointer in place only when that pointer is deleted.
  Without the deletion the unmounted slot shows the pointer again,
  and the lock is belt to the pointer's braces.

### What the wrapper writes beside the registration

Immediately after Git accepts the add,
the wrapper writes one record beside the registration,
following `worktree_added_by_the_wrapper.rs` (F13):

```text
<common>/worktrees/<id>/monochromatic-git-policy-cli-managed-lock
```

Its content is exactly the bytes the lock file holds,
the composed reason plus its one newline,
and nothing else:
no version line,
no path,
no identity.
The record lives inside the registration,
so it dies with the registration (F13's reason for the added record),
and no removal of its own is needed.

The write reuses `write_private_file_unsynced` (`foundation/private_storage.rs:479` to `:481`),
which stages the bytes beside the target and publishes them whole with one exclusive step
(`private_storage.rs:424` to `:442`),
mode `0600` as the wrapper's other records.

A failure to write this record fails the add:
unlike the added-by-the-wrapper record,
which only enables an automatic cleanup and is logged and dropped,
this record decides whether the wrapper may later remove the worktree,
and an add whose lock the wrapper cannot vouch for leaves a worktree only two forces can remove (F9,
L1).
The failed add takes the rollback in "How the wrapper's own removal releases the lock".

## What the reason records, and the record that proves ownership

### The reason text

The reason is a message for a person,
and it is never authority.
It names the program,
says what the lock holds,
carries the claim's nonce,
and names the way out:

```text
monochromatic-git-policy-cli holds this registration for a ZFS clone slot (add claim 1f3c9a2b); remove the worktree with the wrapper
```

It is ASCII,
one line,
and it ends without a newline:
Git completes the line (F8),
so `locked` holds exactly these bytes plus `\n`,
and the record holds the same bytes.
Its length is the sum of a fixed prefix,
the claim nonce and a fixed suffix,
and the byte bound the wrapper reads it with is that sum plus its one newline,
computed in code from those parts (L7).

The nonce earns its place:
it makes two registrations' locks distinguishable by their words,
so a lock copied from another worktree of the same repository does not read as this one's,
and it lets the resume at M2 and M4 attribute a lock whose record was never written (A1,
A6).

Alternatives and their ranking are in "Choices this design marks for the project's veto",
V2.

### The record that proves ownership

Ownership is a fact the wrapper recorded,
and the check is a comparison of bytes:

```text
verified = the registration is proved to be this repository's (F14)
        and the record `monochromatic-git-policy-cli-managed-lock` exists in it
        and `locked` is a regular file whose bytes equal the record's bytes
```

Every part is a read the wrapper already knows how to make:
the registration proof (F14),
the record read with `read_record_here` (`worktree_registration_proof.rs:125`,
under the 4,096-byte record limit at `:41`),
and the lock read as a bounded,
no-follow regular file,
the way `read_the_slot_git_pointer` reads the slot's pointer (`worktree_clone_mount/clone_slot_git_pointer.rs:51` and the reader below it).

Nothing in the check parses the reason.
The words are compared,
never interpreted,
and a person's lock that happens to contain the wrapper's sentence is treated as verified,
which is the deliberate cost of a record that compares bytes rather than identity (V6).

### What is not recorded, and why

- No device and inode of the lock file.
  A prefix relation already explains a torn write of ours (F8),
  inode numbers are reused,
  and under the project's threat model the identity proves nothing a byte comparison does not.
- No timestamp.
  It would date a write nobody acts on.
- No path.
  The record's own directory is the registration,
  which is stronger than any spelling.
- No lock of its own.
  One writer per registration is the add's claim's job while the add runs,
  and afterwards the record is written once and never rewritten except by a recovery that proves the bytes first (NLK).

## How doctor and removal tell the states apart

The check order is the same in both,
and the words differ.

1.  Prove the registration (F14).
    If it is not proved,
    nothing below applies and the state is whatever the proof answered,
    which the route already reports.
2.  Read `locked`:
    absent,
    or a regular file with bytes,
    or an entry the read refuses (a link,
    a named pipe,
    a directory,
    an unreadable file).
3.  Read the record:
    absent,
    or bytes.
4.  Classify.

### The states, the checks and the words

S1.
**The wrapper's verified lock.**
The record exists and `locked` holds its bytes.
Doctor prints,
on standard error,
with no change to its exit status:

```text
the clone slot <slot> holds the managed Git lock its add wrote, so `git worktree prune` keeps its registration; the wrapper releases it after the clone is destroyed
```

S2.
**The wrapper's own lock,
cut short.**
The record exists and `locked`'s bytes are a proper prefix of the record's,
which is the only shape a killed single write leaves (F8).
Doctor prints:

```text
the clone slot <slot> holds the managed Git lock its add wrote, cut short at <n> of <m> bytes, and it still keeps the registration; the wrapper completes it before the slot is used again
```

S3.
**A lock the wrapper did not write.**
The record is absent,
or the bytes are neither equal nor a prefix of the record's.
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
The record exists and `locked` is absent.
Doctor prints:

```text
the clone slot <slot> holds no managed Git lock, so `git worktree prune` can remove its registration while the slot shows no .git and the registration has no index, and the next add or removal that proves this registration writes the lock again
```

S5.
**No pointer.**
The slot shows no `.git`,
so `file_exists` fails and prune falls through to the index and the expiry (F5).
This is a state beside the lock's,
not instead of it,
and doctor prints it with the lock's own state:

```text
the clone slot <slot> shows no .git, so `git worktree prune` can remove its registration at once while it has no index, and the managed Git lock is what stops it
```

S6.
**No registration.**
`worktrees/<id>` is gone,
and the record with it.
Doctor prints what is left and what cannot be rebuilt,
with the measured words (F10):

```text
the clone slot <slot> has no Git registration, and `git worktree repair` cannot rebuild one once it is gone; the binding record names the clone, and recovery is the removal's own settle against that record
```

S7.
**A registration the wrapper did not make.**
The registration proves (F14) but no added-by-the-wrapper record stands,
so this add did not make it.
Doctor and removal treat it as somebody else's worktree,
whatever lock it holds.

### What removal does with each state

The removal's preflight and its second look both classify,
and the second look classifies again immediately before the settle.

- S1 verified:
  not a refusal.
  The settle releases it (below).
  `--force` is not involved.
- S2 cut short:
  not a refusal,
  once the wrapper completes it.
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
  the removal proceeds.
  The lock protects a registration that is about to be settled,
  its absence costs nothing the removal needs,
  and refusing would ask a person to restore a protection only to remove it.
  The removal says so once,
  and re-proves the registration immediately before the settle,
  because a registration with no lock can be pruned between any two steps.
- S5 no pointer:
  not a refusal by itself.
  The pointer is the underlay's and the settle removes the slot;
  the removal re-proves the registration by the `gitdir` back-link,
  which does not need the pointer's existence,
  and it never falls through to Git's own recursive removal for a managed slot.
- S6 no registration:
  the removal may settle only after proving there is no replacement registration at that slot and no other registration naming that path ([proposal][proposal],
  lines 460 to 461):

  > A removal whose recorded clone and origin are already gone can settle a missing registration only after proving there is no replacement.

  With no registration there is nothing to release,
  and the settle's Git half records that it found none.
- S7 somebody else's worktree:
  refusal,
  before anything is destroyed.

A lock whose read is refused (a link,
a pipe,
a directory at `locked`) is S3 with its own cause named,
because `file_exists` still protects it (F1) and the wrapper cannot compare bytes it refuses to read.

## How the wrapper's own removal releases the lock

The removal's Git settle runs after the clone's destruction is proven (A3),
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
    the pointer bytes,
    the `gitdir` back-link,
    the `commondir`,
    and the added-by-the-wrapper record naming this slot (F14,
    F13).
3.  **Prove the lock.**
    S1 or S2,
    from the record;
    never from the words.
    A missing lock is S4 and skips step 4.
4.  **Release.**
    `unlinkat` the name `locked` through the held registration descriptor,
    and sync the registration directory.
    The unlink is one step:
    the lock is there or it is not,
    and no reader sees a half-released lock.
    The sync is what keeps a power loss from bringing the lock back beside a registration whose files this settle already removed.
5.  **Empty the registration** through the held descriptor:
    the wrapper's own records and pool,
    then Git's files (F15).
    The `locked` name is already gone,
    which is the ordering this design adds to that step.
6.  **Remove the registration directory** by name from the held `worktrees` directory,
    which succeeds only when it is empty (F15).
7.  **Remove the empty slot** on the checkout dataset,
    only with the recorded identity and only when it is empty (A4).
8.  **Record the settle** in the removal's own records.

Why release before emptying:

- A kill after the emptying and before the release would leave a locked administrative directory with no `gitdir` and no `HEAD`.
  Prune,
  `git gc` and the automatic maintenance would never remove it (F4),
  and `git worktree list` would not even show it (F1,
  P3),
  so only the wrapper's own scan could find it.
- A kill after the release and before the emptying leaves an unlocked,
  partly emptied registration of a destroyed clone.
  Prune removes it once the pointer is gone or the index is old,
  `git gc` and the automatic maintenance do the same,
  and doctor reports it as S4 on a destroyed clone.
  Nothing is lost by that removal,
  because the clone is already gone.

The same helper serves the add's own rollback (below),
which is why the ordering lives in one place.

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

### The add's own rollback

An add that fails after Git accepted the registration takes the same order with its own proof:

- no clone was created:
  prove that,
  then release and empty the registration with steps 4 to 6,
  drop the claim,
  and leave the empty slot as it was found;
- a clone was created:
  prove its destruction with the removal's own proof before the release,
  because the decision's "only after clone destruction is proven" governs every release this design makes.

Without this,
a failed add would leave a locked registration that Git cannot remove and only two forces could (F9,
F12),
which is the one state a person cannot fix with the words Git prints.

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
by writing the recorded pointer bytes into the slot after proving the registration and the slot's identity;
the design prefers that inside an add,
and names `git worktree repair` to a person in every report where the wrapper is not the one acting.
The lock's recovery is nothing:
a lock is not lost with the pointer.

### The lock is lost, the registration is there

The record is still there,
because it lives in the registration.
The install is one exclusive write of the record's bytes to `locked`:

```text
open(<registration>/locked, O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW, 0644), write the record's bytes
```

- `O_EXCL` decides the winner if two wrapper processes install at once,
  and no lock is held (NLK).
- A kill inside the write leaves a prefix,
  which is still a lock and still protects (F8),
  which is why this write does not stage and rename:
  a staging file would leave the registration unprotected.
- `EEXIST` means someone wrote a lock first;
  the classification runs again and never overwrites.

Who may install it is a question for the project (Q1),
because an automatic install answers a person's own `git worktree unlock` by undoing it.
Until that answer comes,
the design is written for A:
any wrapper command that proves the registration installs it and says so once,
and `doctor` never writes.
A different answer changes the triggers that install it,
not the write or the record.

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

So the wrapper reports S6,
names the clone from the binding record,
and offers exactly two ways on:
an explicit recovery that re-registers the slot with Git (`git worktree add` again at the recorded commit) with every identity re-proved,
or a discard through the removal's settle against the binding record.
It never guesses a registration from a branch or a path.

### Across restarts

Everything this design uses is on disk in Git's administrative directory:
the lock file and the record beside it.
Nothing is held in memory,
no daemon is needed,
and no start-time step writes either.
After a reboot,
`doctor` and every command read the same facts,
and the ZFS boot unit's reconcile does not touch either file.

### What cannot be recovered

- The lock's words,
  if a person overwrote them.
  The wrapper will not replace a lock it cannot match,
  and the way back is the person's `git worktree unlock`.
- The record,
  if it was deleted while the lock stayed.
  The lock is then S3,
  and removal refuses until a person unlocks it and a wrapper command,
  no longer able to prove the lock,
  treats the slot as one it may not settle without an explicit recovery.
- Any state of a pruned registration (above).
- The lock's protection during the M1 window (L4).

## What the lock changes for a person using Git directly

Measured on 2.56.1 through the wrapper (P3 to P9) and as Git alone (G3 to G11),
for a worktree this design's add made,
mounted or not:

- `git worktree list` shows `locked`,
  and its porcelain form shows the words.
  Nothing else about the listing changes.
- `git worktree move` refuses,
  and says `use 'move -f -f' to override or unlock first`.
  The wrapper refuses the move of a bound managed clone anyway ([proposal][proposal],
  lines 474 to 476),
  so this only makes Git agree.
- `git worktree remove` refuses the same way,
  and succeeds with two forces while the slot is a valid worktree.
  With the clone unmounted and the slot a plain empty directory,
  those two forces delete the slot directory and the registration and leave the dataset,
  as item 6 saw on 2.43.0.
- `git worktree prune` and `git gc` keep the registration in every state,
  mounted or unmounted,
  with or without the pointer.
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

- The add's invocation and the record's write:
  beside the wrapper's existing add bookkeeping,
  in the module that writes `monochromatic-git-policy-cli-worktree-added` (F13),
  reusing its registration lookup and `write_private_file_unsynced`.
- The reason's composition and the record's name:
  one module of its own,
  exporting the name,
  the composed bytes for a nonce,
  and the byte bound,
  so the add,
  the recovery and `doctor` all use one spelling.
- The classification:
  a function with the states of "How doctor and removal tell the states apart" as its answer,
  used by `doctor`,
  by the removal's preflight,
  by its second look and by the settle,
  so no caller invents its own order.
- The release and the emptying:
  the existing registration emptying (F15),
  with the `locked` unlink and the directory sync added as its first step,
  called by both the settle and the add's rollback.
- `doctor`'s lines:
  the filesystem report the proposal extends ([proposal][proposal],
  "What doctor checks and prints",
  lines 1188 to 1197),
  on standard error,
  with the exit status unchanged.

## The split between plan steps 10, 12 and 13

What each part of this design is built by:

- The two arguments the add hands Git,
  the claim's two extra fields,
  and the record beside the registration:
  step 10.
- The failed add's rollback release:
  step 10.
- The presence of `locked` in the slot report,
  unclassified:
  step 12.
- The record's format and reader,
  the states,
  the install,
  the settle's release and `doctor`'s lines:
  step 13.

Kept,
with three refinements,
each with its reason.

Step 10 keeps the write:
the invocation gains `--lock --reason`,
and the record is written beside the registration ([plan][plan],
step 10;
[plan][plan],
"Where the design is silent",
lines 2913 to 2921).
Refinement one:
step 10 also owns the failed add's rollback release,
because a failed add must not leave a locked registration,
and because the helper the rollback needs is the release,
which is otherwise step 13's.
Without it,
step 10's own kill and `ENOSPC` tests would leave states only two forces could remove (F9).

Step 12 keeps the pointer half ([plan][plan],
step 12,
lines 1637 to 1648).
Refinement two:
step 12 reads the presence of `locked` and prints it as part of the slot's state,
and it does not classify ownership.
A slot report that cannot say "a lock stands" misreports an unmounted clone the moment the lock exists,
and a classification in step 12 would be a second copy of step 13's predicate.

Step 13 keeps the distinction,
the recovery and the release ([plan][plan],
step 13,
lines 1690 to 1736).
Refinement three:
step 13 owns the record's format and its reader,
the `locked`-first ordering of the emptying,
the `locked` install of the recovery,
and `doctor`'s lines.
The plan gives step 13 the distinction and the release already;
the record is named here because a distinction with no record is the reason-text trap the decision forbids.

The plan's suggestion that step 12 proceeds without step 13's distinction is sound,
then,
because the pointer half alone is what the unmounted states and the move refusal need ([plan][plan],
lines 2922 to 2929),
as long as step 12 shows the lock without judging it.

## Tests a build must have

Each race is named with how a test forces it without timing luck.
A `hook` below means a test-only seam of the kind the wrapper already uses (`#[cfg(test)] AFTER_THE_STAGING_FILE_IS_WRITTEN` in `foundation/private_storage.rs:432`).

Step 10:

T1.
The add's invocation carries `--lock` and `--reason` with the composed bytes,
and no separate `git worktree lock` runs.
Forced by a seam that records the argument list the add hands Git,
with a real-Git arm that then reads `locked` and compares it to the record (F2).
T2.
A `--lock` add leaves the registration locked at every observable moment.
Forced by sampling the administrative directory at one millisecond through an add of a fixture large enough to last (P2's method),
and by the real-Git arm of T1.
The M1 window inside Git is out of scope and named in L4.
T3.
Each kill state is recognised by the resume.
Forced by building the state by hand for M1,
M2,
M3 and M4,
from the measured shapes (P1,
P3,
P6),
not by killing a real add at a lucky instant.
T4.
A registration at M4,
with the claim and no record,
is adopted:
the resume writes the record and classifies S1.
Forced by the fixture of T3 with the claim present.
T5.
A concurrent `git worktree prune` during the fill does not remove the registration.
Forced by a seam between "discard the copied Git administration" and "install the pointer in the clone",
which runs `git worktree prune` in the main checkout;
a locked arm must keep the registration and an unlocked control arm must lose it,
which proves the seam can see the difference (host arm;
the mounted-clone arm needs the guest,
M1).
T6.
`git gc` and `git maintenance run --auto` during the fill behave as T5,
with the same seam and the same control arm.
T7.
A rollback after a failed add leaves no registration and no lock,
and a rollback after a made clone proves the destroy before the release.
Forced by failing the clone step through the add's own injection point,
and by a seam that fails the record write.
T8.
A record write that fails fails the add and runs the rollback.

Step 13:

T9.
`doctor` reports S1 to S7 with the exact words and an unchanged exit status.
Each state is a fixture;
the control is a healthy slot with no lock.
T10.
Removal refuses S3 and S7,
and proceeds for S1,
S2,
S4,
S5 and S6 with the right notice.
S2's completion runs before the refusal decision is made.
T11.
The release order:
a seam after the `unlinkat` and before the emptying,
and one after the emptying and before the directory removal,
each of which leaves the state "What a kill between the settle's steps leaves" names,
with `git worktree prune`,
`git gc` and `git maintenance run --auto` run against it in the host fixture.
T12.
A person's `git worktree unlock` followed by `git worktree lock --reason <the wrapper's own words>` is treated as S1.
This is a deliberate consequence,
and the test fixes it rather than leaving it to chance (V6).
T13.
A lock written by the wrapper after the fact protects exactly as Git's does:
install through the recovery route,
then `git worktree prune`,
`git gc` and a double force,
each expected as P6 measured.
T14.
`git worktree repair` after pointer loss on a host fixture leaves the lock and returns the pointer (F10),
so `doctor`'s S5 words stay true.

Which need the guest:

- M1 and M2 (below) are the only arms whose subject is a mounted clone,
  and they are guest tests.
- T5 and T6 have a host arm and a guest arm;
  the host arm fixes the seam,
  the guest arm the mount.
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
through the wrapper (P3 to P5) and as Git alone (G3 to G5);
the guest adds the mount covering the underlay pointer,
which is the state M6 describes.
This is the one measurement that decides whether the design's single reason for the lock holds on a real clone.

M2.
**`git worktree repair` over a mounted and an unmounted clone slot,
in the guest.**
Where the rewritten pointer lands in each case,
whether the lock file is touched,
and what `git worktree list --porcelain` and `git status` say afterwards.
Host P9 measured the plain-directory case only.

M3.
**A person's double force on an unmounted clone slot,
in the guest.**
What survives of the slot directory,
the pointer and the dataset,
and what the wrapper's next command sees.
Item 6 measured Git `2.43.0` leaving the dataset;
P7 measured 2.56.1 deleting the slot directory on a plain one.

M4.
**The oldest Git the wrapper accepts,
on the host.**
That `git worktree add --lock --reason` exists and writes the bytes,
with a positive control on 2.56.1.
If it does not exist,
L6 refuses the clone rung,
and this is a build gate rather than a design question.

Nothing else about Git is owed.
The gate image's runs (G1 to G11) answered every Git-alone arm on `2.56.1`,
and no image was built or removed for them.

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
the loss shows after the clone mounts over the pointer (M6).
The wrapper writing the file itself is the same window plus Git's private id derivation.
Pros of the adopted route:
no window the wrapper adds,
upstream's own answer to the race,
no private name derivation (F2).
Cons:
the reason must be known before the add,
so the claim must carry it (A1).
Ranking:
`--lock --reason` > a separate `git worktree lock` > the wrapper writing the file,
because the first removes the window,
the second removes only part of it,
and the third adds a name derivation Git owns.

V2.
**The ownership record**:
one file beside the registration holding the exact lock bytes (adopted),
or the binding record holding them,
or no record with the words parsed.
Example:
a person runs `git worktree lock --reason "held"` on a managed slot.
Under the adopted record the bytes do not match and the lock is S3.
Under the binding record the same result needs the sibling design to carry the field,
which is not settled (A5).
With the words parsed,
any person can write the wrapper's sentence and be believed,
which the decision forbids.
Pros of the adopted record:
it dies with the registration,
needs no sibling,
and needs no new home (F13).
Cons:
one more file in Git's directory,
and a second place to keep in step with the claim.
Ranking:
beside the registration > the binding record > parsing the words,
because the first is self-contained,
the second depends on an unfinished design,
and the third is what the decision forbids.

V3.
**The words the reason carries**:
the sentence with the claim nonce (adopted),
a terse token,
or an explanatory paragraph.
Example:
the words appear in `git worktree list` and in Git's `cannot remove a locked working tree, lock reason:
<words>`.
The adopted sentence tells a person what holds the worktree and what to do next;
the token tells them nothing;
the paragraph repeats mechanics where nobody can act on them.
Pros:
actionable,
unique per add,
bounded.
Cons:
it widens the `git worktree list` column,
and it is long enough that a person who retypes it will not match it exactly (V6).
Ranking:
the sentence > the token > the paragraph,
because a person's way out is the point of the words.

V4.
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
at the cost of a new name and a new leftover for `doctor`.
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

V5.
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
because the bytes are provably the wrapper's own (a prefix of its record),
and a person cannot fix it faster than one write.

V6.
**A person's identical words**:
treated as the wrapper's verified lock (adopted),
or treated as S3 by recording something the words cannot carry.
Example:
a person copies the wrapper's sentence into `git worktree lock --reason` after an unlock.
The adopted rule makes the lock S1 for the next removal,
which then releases it after the destruction proof.
Under the alternative the record would have to carry something else (a timestamp,
an inode),
and each of those is a guess (see "What is not recorded,
and why").
Pros of the adopted rule:
one comparison,
no guesses,
and the consequence is a person undoing and redoing a protection on the worktree they are about to remove.
Cons:
the wrapper cannot tell the person's copy from its own.
Ranking:
adopted > recording an identity,
because under the project's threat model the identity proves nothing the bytes do not.

V7.
**The mode of an installed lock**:
`0644`,
matching Git's own files (adopted),
or `0600`,
matching the wrapper's records.
Example:
a lock the recovery installs after a person's unlock.
`0644` looks exactly like Git's file to any other reader,
and `0600` is what every other record in the registration is.
The words are not secret;
the difference is who can read them.
Pros of `0644`:
one shape for Git's file whoever wrote it.
Cons:
it is the one file in the registration the wrapper writes with a non-private mode.
Ranking:
`0644` > `0600`,
because the file is Git's,
not the wrapper's,
and the wrapper's own record beside it stays `0600`.

## Questions only the project can answer

Q1.
**May the wrapper install a lost managed lock without being asked?**
The record still says the wrapper locked this registration,
and the install is one exclusive write.

- A,
  install in any wrapper command that proves the registration,
  and say so once (recommended).
  Example:
  a person ran `git worktree unlock <slot>` yesterday;
  today `git worktree remove <slot>` by the wrapper first writes the lock again,
  reports `the clone slot <slot> held no managed Git lock, and this command wrote it again before removing the worktree`,
  then continues.
  Pro:
  the protection is back before the window that needs it.
  Con:
  it undoes a person's explicit unlock without being asked,
  and a person using Git on the slot sees a lock they did not set.
- B,
  only when a person runs a new explicit recovery command.
  Example:
  the same slot needs `git monochromatic-git-policy-cli repair-worktree-lock <slot>` first,
  and the removal refuses until then.
  Pro:
  no surprise.
  Con:
  a new command,
  and a person who does not know it is stuck with a refusal.
- C,
  never,
  and only report.
  Example:
  `doctor` prints S4 and the removal refuses until the person unlocks and relocks by hand.
  Pro:
  the wrapper never writes a lock.
  Con:
  the whole point of the lock,
  prune protection,
  is lost until a person acts,
  and the window M6 is exactly when nobody is watching.
  Ranking:
  A > B > C,
  because A restores the protection at the moment the wrapper already proves everything the install needs,
  B adds a command for a state a person rarely recognizes,
  and C leaves the registration unguarded.

Q2.
**Does a removal proceed when the managed lock is missing?**
The record says the wrapper locked this registration,
and `locked` is gone.

- A,
  proceed,
  with one notice and a fresh registration proof immediately before the settle (recommended).
  Example:
  `git worktree remove .claude/worktrees/one` prints `the clone slot one held no managed Git lock when this removal settled it, so nothing was released` and destroys the clone as asked.
  Pro:
  a person's removal is not blocked by a protection that has nothing left to protect and that would be restored and released in the same breath.
  Con:
  if the person unlocked to do something else with the registration,
  the removal does not notice in time to stop.
- B,
  refuse until the lock is back.
  Example:
  the same command refuses with `the clone slot one holds no managed Git lock; run the removal again and this wrapper writes it first` (which presumes Q1's A).
  Pro:
  every settle releases a lock that was there.
  Con:
  two runs for one removal,
  and the first refusal reaches every person with an interrupted write.
  Ranking:
  A > B,
  because the lock protects a registration the removal is destroying,
  so its absence cannot cost the removal anything,
  while a refusal costs every removal a second run.

## Hardening the stated threat model may make unnecessary

The project's sentence is that the account is trusted and root and the account are not a security boundary ([staging][staging],
"The handle record's place").
This design follows it without dropping a check whose absence would hurt even a trusted account:

- The comparison of bytes rather than words **survives** the threat model.
  It exists because a person's accidental `git worktree lock` must not read as the wrapper's own,
  which is an accident,
  not an attack.
- The registration proof's three facts **survive** it.
  They exist because a record outlives the worktree it was written for (F14),
  which happens without any attacker.
- The bounded,
  no-follow reads and the `O_EXCL`,
  `O_NOFOLLOW` write **survive** it.
  A named pipe at `locked` would hold a wrapper command inside `open(2)` for as long as the pipe's writer likes,
  and a symbolic link at the record's name would put the wrapper's bytes wherever it points;
  both are mistakes a person and a program can make with no hostility.
- A claim that **does not survive** it:
  that only the wrapper can write the record and the lock.
  A hostile account can write both,
  byte for byte,
  and this design would then call the lock its own.
  That is acceptable only because the account is trusted;
  the design makes no ownership claim beyond that.
- Checks that exist **only** for a hostile account,
  kept anyway because they cost a read:
  refusing a `locked` entry that is not a regular file,
  refusing a record longer than the pointer limit,
  and comparing `commondir`.
  Under the threat model none of them buys security;
  each prevents a mistake.

## Dependencies on sibling designs

- **Where the add's durable binding record lives,
  and its format.**
  This design needs nothing from it to verify a lock:
  the record beside the registration is enough (V2).
  It assumes the binding record may copy the lock's bytes and the claim's nonce for its own report (A5),
  and that if it does,
  nothing decides ownership from that copy.
  If the sibling puts the binding record's home inside the registration,
  the two records sit beside each other and both die with it,
  which changes nothing here.
- **Where a removal's persisted destruction plan lives,
  and its format.**
  The release is ordered after "clone destruction is proven" as that record states it (A3).
  The design assumes only that the plan can say "the head is gone" before the Git settle,
  which the plan's own step 16 already requires.
- **What a cancelled,
  timed-out or killed removal leaves,
  and what the cancel command proves and does.**
  A removal killed inside the Git settle leaves the states "What a kill between the settle's steps leaves" names,
  and this design assumes the kept cancel command may finish the release and the emptying as a wrapper command acting on the slot.
  If the cancel command is designed to reverse a removal instead,
  it must not reverse a released lock:
  the release is the point after which the registration is disposable.
- **The forced arm of a removal.**
  This design assumes `--force` never reaches a lock:
  the forced arm ends holders,
  and a lock is not a holder.
  The refusal sentences in "What removal does with each state" keep the single force's existing meaning,
  waiving cleanliness.
- **The deferred-destroy checks of a removal.**
  No interaction.
  The release runs after the head destroy,
  which is after both checks,
  so a refusal there leaves the lock standing and the registration kept,
  as it should.
- **The record that carries origin cleanup.**
  No interaction beyond the order:
  the settle does not wait for the origin,
  and the lock goes with the registration at the settle whatever the origin's state.
- **The dirty-main restore on rung 1.**
  No interaction.
  The fill window M6 is bounded by that design's discard and pointer-install steps,
  and this design says only that the lock must stand across them.
- **What re-applies private propagation after a replacement mount.**
  No interaction.
- **Which triggers a lost last event waits for,
  and how nudges are merged.**
  If Q1's answer is A,
  the install runs inside commands the wrapper already runs,
  and it adds no trigger of its own.
  If the sweep is to carry it,
  the trigger design gains one action,
  not one trigger.

## What this design does not cover

- The pointer half of prune protection:
  its underlay pointer's writing,
  keeping and removal are step 12's.
- The binding record's home and format,
  and the destruction plan's,
  which are sibling designs.
- The removal's own steps before the settle:
  lineage proof,
  unmount,
  snapshot batch,
  head destroy.
- A lock for any worktree the wrapper did not add,
  and any lock on an ordinary (non-ZFS) worktree.
  The btrfs route refuses any lock and keeps doing so (F16).
- The clone rung's compatibility floor for `--lock --reason` beyond naming the check (M4,
  L6).
- Any ZFS property,
  mount,
  unmount or dataset behaviour,
  which the accepted design and the measurements already own.
- `git worktree repair`'s internals:
  the design uses its measured behaviour and never re-implements it.

## Sources read

Git,
cloned at `v2.56.1` into `~/temp/agent/zfs-add-time-git-lock-design-2026-11-22/scratch/git-v2.56.1` (`gh repo clone git/git ... --depth 1 --branch v2.56.1`,
with the push URL disabled before anything else):

- `builtin/worktree.c`:
  `:200` to `:247` (prune and its duplicate pass),
  `:255` (the expiry of an explicit prune),
  `:273` to `:288` (the junk removal),
  `:496` to `:520` (the administrative directory and the first lock),
  `:526` to `:546` (the lock,
  then the linking files),
  `:554` to `:605` (the ref store,
  the checkout,
  and the lock's removal),
  `:845` to `:884` (`--lock` and `--reason`),
  `:1025` to `:1076` (list),
  `:1195` to `:1248` (lock and unlock),
  `:1339` to `:1344` (move),
  `:1451` to `:1456` (remove).
- `worktree.c`:
  `:305` to `:326` (the reason reader),
  `:937` to `:1000` (`should_prune_worktree`).
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

This repository,
at `main` `5dd8883a60`:

- [`doc/decision/cli-git-zfs-worktree-design.md`][decision],
  whole.
- [`doc/planning/cli-git-zfs-worktree-design-proposal.md`][proposal],
  the sections quoted above.
- [`doc/planning/cli-git-zfs-worktree-implementation-plan.md`][plan],
  steps 10,
  12,
  13 and 16,
  and the sections named above.
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
  7348 to 7350 (the credential rule),
  17451 to 17462 (Git's automatic maintenance and `worktrees`).
- `package/git-policy/cli/src/native/worktree/worktree_added_by_the_wrapper.rs`,
  lines 1 to 70.
- `package/git-policy/cli/src/native/worktree/worktree_registration_proof.rs`,
  lines 1 to 130.
- `package/git-policy/cli/src/native/worktree/worktree_removal_second_look.rs`,
  lines 95 to 135.
- `package/git-policy/cli/src/native/worktree_clone_mount/clone_slot_git_pointer.rs`,
  lines 40 to 60 and the reader below them.
- `package/git-policy/cli/src/native/foundation/private_storage.rs`,
  lines 424 to 442 and 479 to 481.
- `package/git-policy/cli/src/native/lock/single_owner_claim.rs`,
  lines 1 to 40.
- `package/git-policy/cli/src/native/health/doctor.rs`,
  lines 340 to 560.
- `AGENTS.md`,
  rules `NLK`,
  `LQM`,
  `WTU`,
  `HOM` and `RXI`.

## Revisions

This is the first revision of this document.
Its commits,
with each committer time from `git log --format=%cI`:

- `e053ad7807`,
  2026-11-22T17:31:11-05:00:
  the design,
  whole,
  including the gate image's Git-alone runs.
- The commit that follows it on this branch,
  whose hash `git log` reports:
  this list,
  and the Markdown line breaks normalized to the repository's one clause per line.

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
[vm-prune]: cli-git-zfs-measurements-owed-before-acceptance.md#6-deleted-underlay-pointer-prune-and-lock
