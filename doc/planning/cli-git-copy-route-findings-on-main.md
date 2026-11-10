# Copy-route findings on `main`

Status:
audit,
2026-11-09.
Audited revision:
`main` at `64596b62f4`,
in the worktree
`/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/pi-copy-route-findings-on-main`.
No production code was changed and no test was committed.
The scratch scripts and fixtures are in `${HOME}/temp/agent/pi-copy-route-audit`
and are named where each result is quoted.
Scope:
`package/git-policy/cli/src/native/worktree_copying/` (43 files,
15,061 lines,
test sidecars included) and the modules that decide whether it runs at all.
Linux only,
and the copy route only:
this page says nothing about Windows or macOS,
which the owner's pause covers.

Purpose:
the handover's "Open review findings" records nine open findings and eight fixes
that exist only on the unmerged branch `worktree-agent-ab3dd6294f9d91284`
(head `fe2081f2a1`,
nineteen commits not in `main`),
and says no review covers
what `main` runs instead.
This page answers,
finding by finding,
whether the defect is in `main`,
measures when `main` still takes the copy route,
records what no review covered (locks,
waits,
kill points),
and puts the branch's options in front of the owner.
The branch's fate is the owner's decision;
this page does not take it.

## Where the reports are

All three reviews exist,
plus the reconciliation that numbered the findings:

- `review-wtc/sol-review.txt`,
  the first review,
  of the claim design
  (`/var/home/user/temp/agent/review-wtc/sol-review.txt`).
  Its design document is `review-wtc/worktree-copy-claim.md`.
- `review-wtc/v2/sol-review.txt`,
  the second review,
  of the move into the resident service.
  Its design document is `review-wtc/v2/worktree-copy-service.md`.
  The handover cites only the first and third reviews;
  this one exists and
  `review-wtc/v3/previous-review.txt` is a copy of it.
- `review-wtc/v3/sol-review.txt`,
  the third review,
  of the service plus intents.
  Its design document is `review-wtc/v3/worktree-copy-service.md`.
- `/var/home/user/temp/agent/reviews/p1-reconciliation.md`,
  which assigns the
  nine open findings and the eight off-`main` fixes their identifiers.

No report is missing.
The handover's own summary cites
`/var/home/user/temp/agent/reviews/p1-reconciliation.md` and the three `review-wtc` reviews,
and
all of them were read in full.

## The code the reviews examined against what `main` runs

The three reviews examined three successive shapes of one design:

1.  a copy held by a per-destination claim,
    with `/proc` evidence of the holder and
    an age rule that adopted expired claims;
2.  the same copy moved into the resident service,
    with request files and a
    per-registration token;
3.  the service plus a durable intent written before Git runs,
    a terminal outcome
    record,
    and dependency ordering of requests.

`main` has none of those three:
it runs the copy **inside the creating command's
own process**,
with no service,
no request,
no intent,
and no per-destination
completion marker.
What it has instead:

- `worktree_copy.rs` is the whole lifecycle around one forwarded Git run;
  the copy
  runs after real Git returns,
  in the same process.
- A stage claim (`worktree_copy_stage_claim.rs`) records a stage name and the
  process that owns it before any staged byte exists;
  a journal
  (`worktree_copy_journal.rs`) records the destination,
  source and stage after
  staging,
  and its file name carries the owning process.
- Every exclusive decision is a name that one process owns plus an atomic rename
  (`worktree_copy_journal.rs:389`,
  `adopt_journal`),
  or an exclusive create.
- There is no `mark_copied`,
  no `cli-git-copied` marker,
  and no directory named
  after a worktree's administrative id anywhere in the copy path.
  `rg 'cli-git-copied|mark_copied'` over `src/` finds nothing.
- Nothing in `worktree_copying/` reads a worktree's lock file or its text:
  `rg 'initializing'` over `src/` finds nothing.
- The directory a copy decides to fill is the destination the command line names
  (`worktree_copy.rs:319`,
  `worktree_copy.rs:325`),
  and the only proof that Git made
  it is "it was not registered before real Git ran and it is registered after"
  (`worktree_copy_registry.rs:311-322`).

So no verdict in this page says "the reviewed code is here".
It is either "the reviewed code is absent and so is its defect",
or
"the reviewed code is absent but its defect survives in this other shape",
and that other shape is named by `path:line`.

## Verdicts on the nine open findings

### `review-wtc` finding 3, a copied marker keyed by a reusable administrative name

The reviewed defect:
`mark_copied(admin_root, admin_id)` wrote a marker inside `worktrees/<admin-id>`,
so removing and recreating a worktree under the same administrative name carried
the marker to the wrong registration.

Verdict in `main`:
**the marker is absent;
the defect is present in another shape.**

- No marker exists (`rg 'cli-git-copied|mark_copied'` over `src/` finds nothing),
  so nothing can be transferred by name.
- The identity that a copy acts on is still the destination **pathname**,
  and it is
  still not tied to the registration that was observed:
  - `worktree_copy_registry.rs:311-322` returns the destination as created when
    `destination_is_registered` is false before Git and true after it;
    the
    administrative directory and the pointer are only a boolean check
    (`worktree_copy_registry.rs:206-208`),
    and the binding
    (`worktree_add_binding.rs`,
    `bind_registration`) that the ladder does carry is
    not passed to the copy.
  - The install is handed the pathname alone
    (`worktree_copy.rs:358-368` calls
    `synchronize_created_worktree_noting` with `created.root`),
    and
    `worktree_copy_transaction.rs:419-451` never re-checks registration identity.
  - Recovery decides the same way,
    by pathname:
    `worktree_copy_recovery.rs:100-125` accepts any registration whose pointer sits
    under the repository's `worktrees` directory and names the destination back.
- Reproduced:
  a pending copy's destination was removed and another registration was
  made at the same path without a checkout;
  the next command's recovery installed
  the old snapshot into the new registration and said
  "recovered ignored-state copies for 1 worktree transaction."
  The output is quoted in "Reproductions".

### `review-wtc` finding 4, a destination being copied becomes another creator's source

The reviewed defect:
`source_is_being_created` checked only Git's `initializing` lock text,
not whether
the source's own ignored-state copy was still running,
so a copy could stage from a
source that was half filled.

Verdict in `main`:
**present.**

- No module checks whether the source has pending copy state.
  `worktree_copy.rs:526-543` runs recovery first,
  and recovery deliberately skips
  every journal and claim whose owner still runs
  (`worktree_copy_recovery.rs:258-282`,
  `worktree_copy_owner.rs:246-262`),
  then
  `settle_creation` copies from `observation.source_root` without asking whether
  that root is the destination of a live claim or journal.
  `worktree_copy_registry.rs:145-172` reads the source identity and nothing about
  its copy state.
- So a second command run from a worktree whose ignored state is still being
  installed stages whatever is there at that moment,
  and its own copy then
  succeeds and is never revisited.

### `review-wtc` finding 6, a kill between journal cleanup and the marker

The reviewed defect:
`complete_journal` removed the journal before the copied marker was written,
so a
kill in between left a completed worktree with no record of completion,
and the
next reconciliation copied it again.

Verdict in `main`:
**the state cannot occur;
a worse-shaped gap replaces it.**

- There is no per-destination completion marker,
  and no path copies a destination
  twice:
  the copy is attempted only by the command that created the destination,
  and later commands have it in their before-set
  (`worktree_copy_registry.rs:311-322`).
- The ordering is therefore not the reviewed one:
  `worktree_copy_transaction.rs:307-315` removes the stage and journal only after
  the installation ended,
  and `worktree_copy_journal_removal.rs:98-110` writes the
  `Complete` phase before removing them,
  so a kill there is finished by recovery
  rather than repeated (`worktree_copy_recovery.rs:215-218`).
- The gap that replaces it is the window **before** a journal exists:
  `worktree_copy_transaction.rs:419` claims a stage and only `:436` writes the
  journal,
  so a process killed in between leaves a worktree that is never filled
  and never revisited.
  Reproduced and quoted in "Reproductions";
  ranked as the
  first new finding.

### `review-wtc/v3` finding 1, the service key no longer matches the state it serializes

The reviewed defect:
copy state is repository-wide,
so a service keyed by one worktree's Git directory
could not serialize the requests and journals of the whole repository.

Verdict in `main`:
**absent.**

- There is no copy service (`rg 'service_copy'` over `src/` finds nothing),
  no
  request file,
  and no copy key of any kind.
- State lives under the repository's common directory
  (`worktree_copy_journal.rs:132-139`,
  `journal_root`),
  and only its owning process
  acts on it:
  a journal is taken over by one atomic rename
  (`worktree_copy_journal.rs:389-407`),
  and a claim of a live owner is left alone
  (`worktree_copy_stage_claim_reaping.rs:273-283`).
- There is nothing to key,
  so nothing can mismatch.

### `review-wtc/v3` finding 2, an intent lets the copy run before Git returned

The reviewed defect:
`reconcile_intent` published a copy request for a destination whose creating Git
had not returned,
because `allow_locked` tested the pathname only.

Verdict in `main`:
**the intent is absent;
the defect is present in another shape.**

- There is no intent file (`rg 'worktree_copy_intent'` over `src/` finds nothing).
- The same confusion exists for a destination that **another** command's Git
  registered while this command's Git ran:
  `worktree_copy.rs:404-406` samples "registered before" before Git,
  and
  `worktree_copy_registry.rs:311-322` accepts any registration at the path after
  Git,
  with no binding to the registration this command made and no readiness
  check.
  A destination whose own checkout has not run is therefore filled.
- Reproduced by the recovery path:
  the second registration was made with
  `--no-checkout`,
  its checkout had not run,
  and the old snapshot was installed
  into it (output in "Reproductions").

### `review-wtc/v3` finding 3, intents expire after an hour while a suspended wrapper needs them

The reviewed defect:
an intent older than an hour was deleted,
so a wrapper suspended before Git could
resume,
create the worktree and be killed with no record left.

Verdict in `main`:
**absent.**

- There is no intent file and no age-based deletion of any copy record.
  Claims,
  journals and temporaries are removed only when their **owner** is proven
  dead (`worktree_copy_owner.rs:246-262`,
  `worktree_copy_stage_claim_reaping.rs:302-306`),
  or when the owner is of another PID namespace and the item has outlived the
  patience,
  which **refuses the command instead of deleting anything**
  (`process/pid_namespace.rs:83`,
  `worktree_copy_owner.rs:246-262`;
  owner decision of 2026-11-04,
  "Refuse and name it").
- A record of an owner this process cannot judge is kept,
  not taken
  (`worktree_copy_owner.rs:256-259`).

### `review-wtc/v3` finding 4, giving up is reset by a live intent

The reviewed defect:
a given-up request was not terminal while an unreturned intent still named its
destination,
so a new request reset the three-try limit.

Verdict in `main`:
**absent.**

- There are no request files,
  no try counter and no given-up record in the copy:
  `rg 'worktree_copy_request'` over `src/` finds nothing,
  and the words "given-up" and "tries" appear only in unrelated landing
  work-item fixtures.
- A failed installation ends its transaction:
  rollback,
  then removal of the stage
  and journal (`worktree_copy_transaction.rs:307-315`),
  and the failure is reported
  by the creating command with exit status 2 (`worktree_copy.rs:236-249`,
  `exit_code`).
  Nothing retries it,
  which is the opposite defect and is ranked as a
  new finding.

### `review-wtc/v3` finding 5, background failures and terminal-failure crash windows are silent

The reviewed defect:
a wake-only drain dropped failure notices,
and a kill between the terminal record
and the notice left a failure nobody ever saw.

Verdict in `main`:
**present in another shape.**

- Recovery notices go to the standard error of the command that ran recovery
  (`worktree_copy_recovery.rs:302-320`),
  and a copy failure is the creating
  command's own exit status 2,
  so there is no background delivery to lose.
- The silent window that remains is the one before a journal exists:
  a process
  killed between real Git returning and the journal write leaves **no notice at
  all**.
  Reproduced:
  after the kill,
  a later command removed the claim with an
  empty standard error,
  and the destination worktree kept missing its ignored
  state.
- A discard with a notice does name the destination
  (`worktree_copy_recovery.rs:206-229`),
  but it also ends the obligation:
  nothing retries it.

### `review-wtc/v3` finding 6, reading intents before requests leaves the missing-prerequisite window

The reviewed defect:
reconciliation and request selection were separate snapshots,
so a dependant
could copy from a source whose own copy had not been picked up yet.

Verdict in `main`:
**the queue is absent;
the defect is present as finding 4's
source-readiness gap.**

- There is no queue and no dependency ordering.
- Nothing asks whether the source has pending copy state before staging from it
  (`worktree_copy.rs:526-543`),
  which is exactly the window the reviewed finding
  describes,
  without the intent machinery that made it narrower.

## Verdicts on the eight fixes that exist only on the branch

The branch is reported to fix these;
`main` does not have the fixes.
For each,
whether the defect is in `main`:

- `review-wtc` finding 1,
  expired claims bypass holder detection:
  **absent in
  `main`.**
  No claim is ever adopted by age;
  a claim whose owner is alive is left
  alone and one whose owner is dead is reaped
  (`worktree_copy_stage_claim_reaping.rs:273-306`).
- `review-wtc` finding 2,
  `/proc` absence is not evidence of death:
  **present for a
  repository shared by two hosts.**
  An item whose owner is another PID namespace
  and whose record is younger than the patience counts as alive
  (`process/pid_namespace.rs:248-259`),
  but an item written on another host carries
  a process id that does not exist here,
  so `process_birth_identity` returns nothing,
  and the owner reads as dead (`worktree_copy_owner.rs:202-215`,
  `process/process_identity.rs:370`),
  which
  lets a second host take over a live item.
  No copy module mentions a shared or
  network filesystem,
  so whether that deployment is supported is an owner question,
  not a settled limit.
- `review-wtc` finding 5,
  readiness from `--lock` or translated text:
  **absent in
  `main`.**
  No copy code reads a lock file or its text;
  readiness is registration
  presence (`worktree_copy_registry.rs:311-322`),
  which is
  finding `v3` 2's problem,
  not this one.
- `review-wtc/v2` finding 1,
  a killed wrapper leaves a completed worktree with no
  recoverable request:
  **present.**
  This is the first finding under "New findings";
  the
  branch's fix (a durable intent before Git) is the shape `main` lacks.
- `review-wtc/v2` finding 2,
  two services delete each other's live stage:
  **absent.**
  There is no service,
  and only a dead owner's stage is removed
  (`worktree_copy_stage_claim_reaping.rs:273-306`).
- `review-wtc/v2` finding 3,
  the token tags a replacement registration:
  **present in
  the pathname shape** described under `review-wtc` finding 3.
  `main` has no token
  (`rg 'worktree_copy_identity'` over `src/` finds nothing).
- `review-wtc/v2` finding 4,
  dependants run after a failed prerequisite:
  **absent as
  stated,
  present as finding `review-wtc` 4.**
  There is no dependency ordering to
  get wrong;
  the source-readiness gap survives.
- `review-wtc/v2` finding 5,
  a terminal failure becomes success after restart:
  **absent.**
  The creating command reports its own failure synchronously;
  a killed
  command leaves a journal that recovery finishes,
  and recovery's failure notices
  name the destination (`worktree_copy_recovery.rs:206-229`).

## Locks, waits and kill points in `main`'s copy modules

### Locks

There is **no lock** in `worktree_copying/`.
`rg 'flock|lockf|fcntl|Mutex|RwLock|Condvar'` over the folder,
test sidecars
excluded,
finds nothing;
the only synchronisation is:

- an atomic rename that publishes a journal,
  a claim,
  or a temporary
  (`worktree_copy_journal.rs:448-456`,
  `adopt_journal` at `:389`),
  which is how one
  command takes an abandoned journal and every other command loses;
- the owning process written into each item's name
  (`worktree_copy_owner.rs:143-147`,
  `token`),
  so liveness is a property of the
  name,
  not of a lock;
- exclusive creation of every installed path
  (`worktree_copy_install_entry.rs`,
  `copy_file_exclusive`),
  so two writers cannot
  both create one entry;
- an atomic cursor,
  not a mutex,
  for the removal and copy worker pools
  (`worktree_copy_journal_removal.rs:163-170`,
  `:209-219`).

This is the owner's "use as few locks as possible" carried to zero in the copy
path.
The price is that exclusion rests on two proofs:
a file name whose owner can be
judged,
and a rename that only one process can win.

### Waits and their bounds

- The copy's own Git queries (`ls-files`,
  `worktree list`) run through the bounded
  step runner:
  a plumbing step is bounded at two minutes
  (`service/wait_ceilings.rs:330`,
  `transaction/transaction_git_step_bounds.rs:52`),
  so no copy waits on Git forever.
- No sleep,
  retry loop or poll loop exists in the copy modules;
  recovery of a live
  owner's item is skipped,
  not waited for
  (`worktree_copy_recovery.rs:258-282`).
- After either route the package-manager repair runs with an eighteen-minute ceiling
  and a quiet window (`service/wait_ceilings.rs:406`,
  `repair/package_repair_command.rs`),
  and a snapshot's trash disposal runs on a
  thread the command joins before exit (`worktree_copy_after_lock.rs:202`).
  A copy route therefore has one bounded wait,
  in the repair,
  and no thread join of
  its own;
  a snapshot route also joins the disposal thread,
  whose own work is bounded.
- A record of another PID namespace older than the patience does not wait:
  the
  command refuses and names the record
  (`worktree_copy_owner.rs:246-262`).
  The patience is twice `LOCK_CEILING`
  (`process/pid_namespace.rs:83`).

### What a process killed at each step leaves behind

Reading `worktree_copy_transaction.rs:411-462`,
`worktree_copy_journal_removal.rs:98-110`
and `worktree_copy_recovery.rs:196-240`:

1.  Before real Git returns:
    nothing.
    Git owns the worktree and its registration.
2.  After Git returns,
    before `claim_stage` (`worktree_copy_transaction.rs:419`):
    **no record of any kind**,
    so the new worktree is never filled,
    no notice is
    possible,
    and no later command revisits it.
    Not measured directly;
    the next
    window was,
    and shows the same outcome.
3.  Claim written,
    stage container not yet made:
    recovery judges the owner dead,
    finds no container and no destination record,
    and removes the claim **without a
    notice** (`worktree_copy_stage_claim_reaping.rs:374-399`,
    `settle_interrupted_add` at `:64-77`).
    Measured:
    the destination kept no ignored state and the later command printed
    nothing.
4.  Stage container made,
    staging,
    before the journal:
    recovery removes the stage
    and prints one notice naming the **stage**,
    not the worktree,
    and the worktree
    stays unfilled (`worktree_copy_stage_claim_reaping.rs:389-394`).
5.  Journal written,
    installing:
    recovery resumes the installation from the install
    log (`worktree_copy_recovery.rs:230-240`,
    `worktree_copy_install_log.rs`).
    Measured:
    the destination was completed and
    recovery said "recovered ignored-state copies for 1 worktree transaction."
6.  Installed,
    before the `Complete` phase:
    the same resumption path,
    idempotent
    against the log's claimed and created entries.
7.  `Complete` written,
    cleanup interrupted:
    recovery removes the rest
    (`worktree_copy_recovery.rs:215-218`).
8.  After cleanup:
    nothing left;
    the destination stays filled.

The one state recovery **discards** is a destination that is no longer registered,
or a journal whose stage is gone;
both print a notice that names the destination
and says the ignored files may be incomplete
(`worktree_copy_recovery.rs:206-229`).
Neither retries.

## Reachability measured on this host

Host:
`/var/mnt/encrypted` on `/dev/mapper/crypt_sda`,
btrfs,
`subvolid=5`,
`compress=zstd:3`;
`/home` and the repository are subvolumes;
four CPUs.
The binary was `target/debug/monochromatic-git-policy-cli` built from the audited
revision (`mise run //package/git-policy/cli:native:build`,
2 m 38 s).
The route choice does not depend on the build profile,
but every figure in this
section is the debug build's.
Each fixture was a repository under `${HOME}/temp/agent/pi-copy-route-audit/fixtures`,
with one tracked file and one ignored file.
Script:
`measure-btrfs.ts`.
The copy route is the one that prints
"copied N ignored filesystem entries";
the snapshot and clone rungs print no such
line.

- Main checkout of a repository that is a btrfs subvolume,
  clean:
  **snapshot**.
  "made the worktree as a snapshot of the main checkout;
  nested worktrees,
  caches
  and run records were left out."
- Main checkout of such a repository,
  one tracked file modified:
  **snapshot with a
  restore**.
  "the uncommitted changes of the main checkout (1 path) were left out of
  the new worktree,
  which holds what Git's own checkout holds."
- Linked worktree made by the wrapper (so a subvolume) and clean:
  **snapshot**,
  with the same line the clean main checkout printed.
- Linked worktree made by plain `git worktree add` (not a subvolume),
  clean:
  **copy route.**
  The clean check refused it:
  "tracked.txt is as new as the index,
  so its stat fields prove nothing;
  snapshot:
  None;
  clone:
  None",
  so the ladder
  fell to Git's checkout and the copy ran.
  This is the state of every worktree
  created by plain Git until `git status` refreshes its index.
- Linked worktree,
  one tracked file modified:
  **copy route.**
  "this worktree was not made as a snapshot,
  so it took longer,
  because this
  worktree has uncommitted changes in 1 path (for example tracked.txt)."
- A form the plan does not read,
  `git worktree add -fb topic6 ../case6-dest`:
  **copy route.**
  The destination was still read by the command-line reader,
  so the
  copy ran rather than the no-destination notice.
- Repository not on a subvolume (a plain directory on btrfs):
  **copy route** with
  "the main checkout is not a btrfs subvolume.
  Run `git monochromatic-git-policy-cli
  migrate-to-subvolume`".
- The repository itself:
  352 registered worktrees,
  of which 308 are clean,
  40
  linked worktrees and the main checkout are dirty (41),
  and 3 could not be read.
  Script:
  `census.ts`.
  An add run from any of those 40 dirty linked worktrees takes the copy route;
  an
  add from the dirty main checkout takes the snapshot-with-restore route.

So on a btrfs host the copy route is not the rare fallback the handover's "may be
moot" implies:
it is what every add from a plain-Git worktree takes until its index
is refreshed,
what every add from a dirty linked worktree takes,
and what every add
takes outside btrfs.

## Reachability measured off btrfs

Container:
`localhost/git-policy-native-test-base:9c2c03d16a57-97221c29742b`,
rootless podman,
`--cpus 2 --memory 2g --pids-limit 512 --blkio-weight 10`,
`/work` on tmpfs and the container's own `/tmp` on overlayfs.
Script:
`measure-container.ts`.
The same debug binary was bind-mounted read-only with SELinux labels disabled.

- "filesystem of /work:
  tmpfs",
  "filesystem of /tmp:
  overlayfs".
- Main checkout on tmpfs,
  add on tmpfs:
  **copy route.**
  "copied 2 ignored filesystem entries from \"/work/src\" into 1 new worktree.";
  the destination received `ignored/data.bin`.
- Main checkout on tmpfs,
  destination on the container's overlay root (a different
  filesystem):
  **copy route**,
  same line,
  destination filled.
- No journal root was left after either add.

A loop-mounted ext4 image was not needed:
tmpfs and overlayfs are both filesystems
that never snapshot or reflink,
which is the question.
Mounting a loop image inside a rootless container would have tested nothing
different.

## Reproductions

All three ran against the same debug binary,
on fixtures under
`${HOME}/temp/agent/pi-copy-route-audit`,
which is not the repository and is not
committed.
The source in each was a dirty linked worktree,
because that is the copy route a
btrfs host reaches every day.

### A kill before the journal

Script:
`kill-points.ts`.

The wrapper was killed with `SIGKILL` while its claim existed and no journal did:

```text
claim files while the copy runs: ["0247e85a-dcd0-4d5b-ba0b-325efba164b9.stage"]
journal files while the copy runs: []
killed wrapper exit: {"code":null,"signal":"SIGKILL"}
left after the kill: ["0247e85a-dcd0-4d5b-ba0b-325efba164b9.stage"]
later command exit: 0
later command stderr: ""
left after the later command: []
destination has its ignored payload: false
destination file count in ignored/: absent
a later add exit: 0; summary line:
["...copied 61 ignored filesystem entries from \"...kill-source\" into 1 new worktree."]
the first destination after the later add: false
```

The destination is a registered worktree of the repository on branch `killed`,
it
holds `.git`,
`.gitignore` and `tracked.txt`,
and it never receives `ignored/`.
The later command removed the claim with an **empty** standard error,
and the add
that created a second destination copied 61 entries while the first destination
stayed unfilled.

### A kill after the journal

Same script.

```text
journal while installing: ["30e43d4f-40ea-45e5-92dc-0d34ffd83318.3496201~linux:29456741~pid:[4026531836].json"]
left after the second kill: ["30e43d4f-...json","b7cffd24-6d7a-434b-9ca1-25454edd0495.stage"]
recovery command exit: 0
recovery command stderr: "monochromatic-git-policy-cli: recovered ignored-state copies for 1 worktree transaction.\n"
second destination was completed: true
left after recovery: []
```

A kill after the journal is recorded is recovered:
this part of the design works.

### A replacement registration at the destination

Script:
`replacement-registration.ts`.
A copy was killed after its journal was written;
the destination's registration was
then removed and another one created at the same path **without a checkout**,
as a
concurrent command's add would have it while its own Git still runs.

```text
journal before the kill: ["6f69b40a-...json"]
journal after the kill: ["40dadf65-6772-4e2c-b31b-ec2450a0fc40.stage","6f69b40a-...json"]
destination registered: true
destination has the payload: false
remove the first registration: exit 0
second registration at the same path: exit 0 Preparing worktree (detached HEAD bd06f90)
second registration's checkout exists: false
payload before recovery: false
recovery command exit: 0
recovery command stderr: "monochromatic-git-policy-cli: recovered ignored-state copies for 1 worktree transaction.\n"
payload after recovery in the second registration: true
journal left: []
```

The second registration's checkout had not run (`tracked.txt` absent),
and the
recovered snapshot wrote `ignored/secret.env` into it.
No user data was deleted,
but a destination this copy never observed was filled.

### A failed real Git that created the destination

Not reproducible this way:
on Linux the real-Git search promotes `/usr/bin/git`
and `/usr/local/bin/git` ahead of the `PATH` it searched
(`invocation/real_git.rs:311-316`),
so a `git` script placed first on `PATH` is
never used.
The attempt's own output was
"shadowing git at /usr/bin/git".
The creating-path shape of the replacement race is therefore established by reading
`worktree_copy_registry.rs:311-322` and by the recovery-path reproduction,
not by a
live two-command race.

## New findings ranked by what they risk losing

No finding in this section deletes or overwrites user data:
installation is
exclusive-create
(`worktree_copy_install_entry.rs`) and rollback removes only entries whose identity
and content prove it made them (`worktree_copy_rollback.rs`).
What these findings risk is a worktree that silently differs from its source,
and writes into a worktree another command owns.

### 1. A kill between real Git returning and the journal leaves no record and no notice

`worktree_copy_transaction.rs:419-436`:
the claim comes first,
the journal later,
and nothing is written before the claim.
A process killed in that window,
or a few instructions earlier,
leaves either a
claim with no destination record or nothing at all.
Recovery then removes the claim without a word
(`worktree_copy_stage_claim_reaping.rs:374-399`,
`:64-77`),
and no later command
retries the destination,
because it is in every later before-set
(`worktree_copy_registry.rs:311-322`).
Measured in "Reproductions":
the worktree stayed without `ignored/` and the later
command printed an empty standard error.

What it risks:
a person works in a worktree whose ignored state (an environment file,
a key,
a
local cache,
build state) is missing,
and nothing ever says so.
This is the highest risk because it is both silent and permanent,
and because it
sits exactly where the branch's fix (a durable intent before Git) was aimed.
The owner's "never discard user data" is not what is broken;
"failures never
silent" is.

### 2. The copy fills a registration this command did not create

`worktree_copy_registry.rs:311-322` and `worktree_copy.rs:358-368`:
the only proof
is a pathname's registration state before and after real Git,
and the install gets
the pathname alone.
The same is true of recovery (`worktree_copy_recovery.rs:100-125`).
Measured in "Reproductions":
the recovered snapshot was installed into a
replacement registration whose checkout had not run.

What it risks:
writes into another command's worktree,
including one still checking out,
which can
disturb that checkout;
and a person's new worktree receiving ignored state that
belongs to a registration that was removed.
The ladder already has the machinery that closes this for snapshots and clones
(`worktree_add_binding.rs`,
`verify_association`);
the copy path does not use it.

### 3. Recovery discards the obligation when the stage is missing

`worktree_copy_recovery.rs:219-229`:
a journal whose stage is gone is removed with a
notice that names the destination and says the ignored files may be incomplete,
and
nothing retries.
Also `worktree_copy_stage_claim_reaping.rs:389-394`,
where the notice names only the
stage,
not the worktree that will stay incomplete.

What it risks:
a permanently incomplete worktree.
Better than the first new finding because a notice exists,
worse than the fifth because no record of the outstanding work survives,
so a person who missed the line
has no second chance.

### 4. A source whose own copy is running is used as a source

`worktree_copy.rs:526-543`:
recovery skips live owners,
and the copy then stages from
the source without asking whether that source is itself the destination of a live
journal or claim.

What it risks:
a worktree whose ignored state is a partial copy of its source,
with no notice
anywhere,
because the copy that produced it succeeded.
This is the reviewed finding `review-wtc` 4 and `v3` 6,
still open in `main`.

### 5. A failed installation is retried by nobody

`worktree_copy_transaction.rs:411-418` and `:307-315`:
a failed installation rolls
back what it created and its journal and stage are removed,
so the creating command's
exit status 2 is the only record.
No later command revisits the destination.

What it risks:
the same incomplete worktree as finding 1,
but the person was told at the time,
so
it ranks last.

## Options for the branch

The branch's nineteen commits move the copy from the command's process into the
resident service,
and add intents,
requests,
registration tokens,
terminal outcome
records and dependency ordering.
Its nine open findings are in that architecture.
The eight fixes it carries are real:
a durable pre-Git intent (the first new
finding),
a directory-identity registration token (the second),
dependency ordering (the second review's finding 4),
and terminal records (the third review's finding 5,
in a different shape).

### Option A, merge the branch

Pros:
every off-`main` fix arrives at once,
with its tests,
including the two that map
onto the first two new findings.

Cons:
it replaces the in-process copy with a service-keyed design whose own review found
the service key does not match the state it serializes (v3 finding 1,
owner decision
of 2026-11-01:
"do not fix these on the current repository-wide layout");
the shipped wrapper would route every copy through the service,
which no review
in this folder covers;
it was written against an older `main` (it deletes `worktree_copy_lock.rs`,
which
`main` no longer has) and needs a nineteen-commit reconciliation that nobody has
attempted;
and it would reintroduce an architectural choice the btrfs ladder already replaced.

### Option B, port the narrow pieces that still fit

Three pieces,
in the order their risk ranks:

1.  a durable destination record written **before** real Git runs,
    on the rungs that
    end in Git's own checkout and on the copy route:
    `main` already does this for a
    snapshot or clone (`claim_stage_for_add`,
    `worktree_ladder_run.rs:213-240`) but
    not for `Rung::Git` (`:193`,
    `:207`,
    `:244`),
    and recovery already knows how to
    report a recorded interrupted add
    (`worktree_copy_stage_claim_reaping.rs:64-120`);
2.  carry the registration binding the ladder already builds
    (`worktree_add_binding.rs`) into the copy and the recovery decision,
    so the
    install writes into the registration that was bound and stops when it is gone;
3.  a source-readiness check that refuses to stage from a root with a live claim or
    journal.

Pros:
each piece is small and lands in `main`'s own design,
with `main`'s tests;
they close the first,
second and fourth new findings without any service;
they need no reconciliation of nineteen commits.

Cons:
three separate changes with their own tests and reviews;
nothing arrives from the branch's eight fixes automatically,
so the branch's tests
cannot simply be reused;
whether these are worth doing is the owner's call,
since the audit was told not to
change production code.

### Option C, drop the branch

Pros:
`main` already replaced the copied marker,
the `/proc` adoption,
the expiry and the
token with a design the reviews did not need to cover,
and the btrfs ladder removed
the slow path for the common btrfs cases;
nothing touches the service key problem,
which the owner already decided not to fix.

Cons:
the two findings that survive in `main` (a durable pre-Git record and a registration
binding) are exactly what two of the branch's fixes implement,
so dropping the
branch loses ready-made answers;
the branch also carries its own tests for those fixes.

### Ranking

Ranking:
B > C > A,
because:

- B over C:
  findings 1 and 2 are reachable every day on this host (a dirty linked
  worktree,
  a plain-Git worktree,
  any non-btrfs filesystem),
  and both are silent or
  write into someone else's registration;
  the branch already contains working
  answers to both,
  so porting them costs less than rediscovering them.
- C over A:
  the branch's architecture is the one the owner declined to repair,
  and
  merging it would reintroduce a service-keyed copy over state that is
  repository-wide;
  dropping it keeps `main`'s simpler design,
  at the price of
  leaving findings 1 and 2 open until someone ports them.

## What I could not determine

- Whether the branch still merges and passes its tests against this `main`.
  The audit was read-only for production code,
  and no build of the branch was
  started.
- Whether the creating path,
  as opposed to the recovery path,
  fills a foreign
  registration under live concurrency.
  The missing guard was read and the recovery
  path was reproduced;
  a live two-command race was not attempted.
- What a kill during the rollback of a failed installation leaves,
  and whether
  recovery's choice to resume an installation rather than a rollback can produce a
  second,
  different result after the person was told the first one failed.
  The rollback and install are exclusive and identity-checked,
  so no user data is at
  risk by reading,
  but the state machine there was not exercised.
- Whether a repository on a filesystem shared by two hosts is supported.
  If it is,
  the released `/proc` evidence is not sound across hosts (see the
  `review-wtc` finding 2 verdict) and that needs an owner decision of its own.
- The installed wrapper's route choice:
  every measurement used a debug binary
  built from the audited revision,
  by path,
  with `PATH` set to `/usr/bin:/bin`.
  The route decision is code in that binary,
  but the installed release was not
  exercised,
  and the owner's pause covers Windows and macOS.
- The reachability census read three worktrees as unreadable and did not say why;
  they are counted separately rather than guessed clean.

## What to inspect and how to respond

Inspect,
in this order:

1.  the verdicts on the nine findings,
    against
    `worktree_copy.rs`,
    `worktree_copy_registry.rs`,
    `worktree_copy_transaction.rs`
    and `worktree_copy_recovery.rs`;
2.  the reproductions under "Reproductions",
    whose scripts are in
    `${HOME}/temp/agent/pi-copy-route-audit`;
3.  the ranking and the three options.

Respond with a decision on the branch (A,
B or C),
and,
if B,
with which of the
three pieces to build first.
The audit itself did not change production code,
so no other action is pending.
