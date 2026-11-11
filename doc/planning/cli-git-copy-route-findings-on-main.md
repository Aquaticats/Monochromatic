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

## Triage of the branch on `main` (2026-11-10)

Status:
triage,
2026-11-10T06:33:26-05:00.
Revision triaged:
`main` at `3fb25eeeb1`,
in the worktree
`/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/pi-triage-unmerged-copy-agent-branch`,
on branch `fix/git-policy-cli-triage-unmerged-copy-agent-branch`.
Branch triaged:
`worktree-agent-ab3dd6294f9d91284` at `fe2081f2a1`,
nineteen commits not in `main`,
still unmerged and **not** deleted by this pass,
because the coordinator deletes it once this lands.
Owner's decision (question tool, 2026-11-10),
verbatim:
"Triage, then delete (Recommended)",
whose option text is
"An agent checks each finding against today's main, fixes what still holds, records the rest, and the branch goes once that is done."
What this pass did:
read the three reviews and the reconciliation in full,
checked every finding and every fix against today's code,
reproduced what a probe could reach,
fixed one finding that still held,
and wrote the rest of the verdicts here so nothing on the branch lives only there.
The scratch scripts are in `${HOME}/temp/agent/pi-copy-triage-20261110`,
and every result quoted below names the script that produced it.
Linux only,
copy route only,
as the audit above.

### What `main` gained after the audit above

The audit ranked three options;
the owner's later work built the middle one,
without the branch,
and most of what the branch's reviews asked for now exists in `main`'s own design:

- A creating command records its destination and stage name **before** real Git runs
  (`claim_stage_for_creation`,
  `worktree_copying/worktree_copy.rs:341`;
  `d30064ca28`),
  and every later command names such an add until the worktree is whole or gone
  (`worktree_copying/worktree_copy_interrupted_add_settlement.rs:361`;
  `20656913bf`,
  `84de1c4bcf`).
- A destination is reserved by one exclusive directory creation before Git,
  so one command owns it and a destination that stood there already is another command's
  (`reserve_destination`,
  `worktree/worktree_subvolume.rs:197`;
  `d403a70a5e`).
- The registration Git made is bound when Git returns, carried beside the staged payload,
  and checked before staging and again before the first entry is written
  (`find_created_worktree`,
  `worktree_copying/worktree_copy_registry.rs:322`;
  `worktree_copy_stage_binding.rs:120`;
  `14a35d25bd`).
- A copy stages nothing from a source that is itself being filled
  (`worktree_copying/worktree_copy_source_still_being_filled.rs:96`;
  `56d56d554f`).
- A worktree a copy could not finish keeps a durable record that every later command names
  (`worktree_copying/worktree_copy_unfilled_record.rs:271`;
  `84de1c4bcf`).
- A temporary carries the token of the process writing it, and only a dead writer's is removed
  (`worktree_copy_journal.rs:226`,
  `worktree_copy_stage_claim_reaping.rs:316`).
- A journal that vanished between the listing and the read is skipped, not reported
  (`worktree_copy_journal.rs:479`).
- No lock stands on the copy path at all:
  `rg 'flock|lockf|fcntl|RwLock|Condvar'` over `worktree_copying/` finds the notice stream's
  in-process `Mutex` and nothing else.

### Verdicts, finding by finding

Each verdict is about today's `main`.
"Closed" means the reviewed defect cannot occur and a named control pins it,
"absent" means the reviewed mechanism does not exist in `main` at all,
and "open" means the defect is reachable there.

#### `review-wtc`, the per-destination claim design

Finding 1,
expired claims bypass holder detection:
**absent.**
No item is adopted by age;
an item is taken only when its owner is proven dead
(`worktree_copy_owner.rs:275`),
and an item of another PID namespace past the patience stops the command instead
(`process/pid_namespace.rs:251`;
owner decision of 2026-11-04,
"Refuse and name it").

Finding 2,
`/proc` absence is not reliable evidence of death across hosts or restricted visibility:
**open for a repository two hosts share.**
An item written on another host names a process id that does not exist here,
and its namespace id can equal this host's,
so it reads as a dead owner's and can be taken.
A record of another namespace that this host can see differently is refused by the patience rule,
so the same-host and other-namespace cases are handled;
only the cross-host case is not.
Whether a repository shared by two hosts is supported is an owner question,
and no copy module mentions a shared or network filesystem.
This pass did not fix it: the fix is a host identity in every owner token,
which changes the record format,
and it is meaningless until the owner says such a deployment is supported.

Finding 3,
a copied marker keyed by a reusable administrative name,
with the staging-time ABA it named:
**closed.**
There is no marker
(`rg 'cli-git-copied|mark_copied'` over `src/` finds nothing).
The job the marker did is done by three proofs instead:
the reserved object's identity,
the registration that Git made
(`find_created_worktree`,
`worktree_copy_registry.rs:322`,
which refuses a registration that is not the reserved object at `:340`),
and the stage's binding,
re-checked before the first entry
(`worktree_copy_transaction.rs:274-288`).
Two windows stay open inside the microseconds between a check and a write,
and `package/git-policy/cli/SPEC.md:713` writes both down.
Controls:
`a_registration_that_is_not_the_reserved_object_is_not_claimed`
(`worktree_copy_registry_tests.rs:329`),
`a_binding_holds_until_the_registration_behind_the_path_is_another`
(`worktree_copy_stage_binding_tests.rs:64`),
`a_destination_made_again_before_the_lookup_is_left_alone`
(`worktree_copy_lifecycle_tests.rs:724`).

Finding 4,
a destination being copied becomes another creator's incomplete source:
**closed for a copy that runs or was interrupted;
the same consequence survived for a copy that ended unfilled, and is fixed in this pass.**
`source_is_still_being_filled` reads the claims and then the journals,
answers for a live owner and for a dead one,
and the copy then stages nothing and says why
(`worktree_copy_source_still_being_filled.rs:96`,
`worktree_copy_synchronize.rs:361`;
`56d56d554f`).
It did not read the record of a copy that **ended** before it was whole,
so a worktree the wrapper itself had named as holding less than its source ignores
could still be another add's source.
The next section records the reproduction and the fix.

Finding 5,
checkout readiness from `--lock`, translated text, or a sentinel reason:
**absent.**
No copy code reads a lock file or its text;
`rg 'initializing'` over `src/` finds nothing.
Readiness is decided by the reservation and the registration binding,
so a lock reason and a translation cannot mislead it.
A worktree created with `--lock` is copied like any other,
because the lock no longer gates anything.

Finding 6,
completion forgotten if the process dies after journal cleanup and before the marker:
**cannot occur.**
There is no marker and no path copies a destination twice.
The order is: install,
forget the unfilled record,
write `Complete`,
then remove the journal and stage
(`worktree_copy_transaction.rs:294-315`),
and a journal a process left is resumed idempotently from its install log
(`interrupted_installation_resumes_from_its_log`,
`worktree_copy_transaction_tests.rs:153`).

Finding 7,
on macOS an abandoned claim is never adopted:
**absent.**
There is no platform-specific adoption branch;
owner liveness on macOS is `ps`
(`process/process_identity.rs:364`),
so a dead owner's item is settled like one on Linux.
That path is untested by any gate,
and the owner's pause covers Win and mac.

Finding 8,
temporary cleanup can unlink a live publisher's temporary:
**closed.**
A temporary's name carries its writer's token
(`temporary_path`,
`worktree_copy_journal.rs:207`;
`temporary_writer_token`,
`:226`),
and the sweep removes only a dead writer's
(`worktree_copy_stage_claim_reaping.rs:316`;
control:
`a_temporary_left_by_a_killed_run_is_removed_and_never_published`,
`worktree_copy_publish_file_tests.rs:158`).

Finding 9,
a legitimate `journal-*` administrative id loses its marker,
and an adoption key can collide with one:
**absent.**
There is no marker and no adoption-key namespace;
journals, claims and records are named by a random identifier and the owner token.

Finding 10,
unsupported hard links fail the feature,
and a failed or pre-journal copy has no ordinary retry:
**partly closed.**
Publication no longer depends on hard links:
the copy publishes through an unnamed temporary file where the filesystem offers one,
`renameat2` with `RENAME_NOREPLACE` where it does,
and a link only as a fallback
(`worktree_copy_publish_file.rs:99-130`).
A copy that fails leaves a durable record that names the worktree and the two commands that settle it,
and nothing retries it automatically:
the line says so
(`worktree_copy_unfilled_record.rs:418`),
which is a deliberate choice, not a gap.

A remaining journal-listing race the review asked to verify:
**closed.**
A journal that vanishes between validation and read is skipped
(`read_listed_journal`,
`worktree_copy_journal.rs:479`).

#### `review-wtc/v2`, the copy in the resident service

Finding 1,
a killed wrapper leaves a completed worktree with no recoverable request:
**mostly closed;
a window of one fsync stays open and is reported below as the one open defect.**
The pre-Git claim and its destination record are the durable obligation
(`claim_stage_for_creation`,
`worktree_copy.rs:341`),
and the reaper reports an interrupted add until it is whole or gone
(`worktree_copy_interrupted_add_settlement.rs:361`;
controls
`a_creating_command_records_its_destination_before_real_git_runs`,
`worktree_copy_lifecycle_tests.rs:966`,
and
`an_add_killed_before_its_binding_is_recorded_is_named_until_it_is_finished`,
`worktree_copy_stage_claim_tests.rs:517`).
`probe-kill-points.ts` shows the window that is left:
the claim is published first and its destination record second,
so a kill between the two leaves a claim that names no destination,
and `settle_text` treats a record that is gone as nothing to say
(`worktree_copy_interrupted_add_settlement.rs:411`).
See "The one open defect this pass found".

Finding 2,
two services can delete each other's live stage:
**absent.**
There is no copy service;
the copy runs in the creating command's own process,
and another command's stage is touched only when its owner is proven dead
(`judge_stage_owners`,
`worktree_copy_stage_claim_reaping.rs:285`).

Finding 3,
the token can identify the replacement registration:
**closed.**
The registration's destination identity is compared with the object the command reserved
(`worktree_copy_registry.rs:340`),
and the copy is bound to the registration before the first byte is staged.

Finding 4,
ordering a prerequisite first does not require it to finish:
**absent as stated.**
There is no queue and no dependency ordering to get wrong;
every creating command copies only into its own destination,
and the source-readiness check is the fence.
"A prerequisite published during a drain" cannot arise without a drain.

Finding 5,
a terminal failure becomes success after a restart:
**absent.**
The failure is the creating command's own exit status, 2
(`worktree_copy.rs:249`),
and a worktree it could not fill keeps a record that every later command names.

Findings 6 to 9,
progress coupled to a drain,
a temporary deleted by age,
the added-path parser skipping a clustered `-fb`,
and a pending-only wake that does not upgrade an older service:
**absent or closed.**
There is no service, and therefore no drain, no wake and no version skew.
Temporaries carry their writer's token (finding 8 of the first review above).
The destination is read by one option table compared against `git worktree add -h`
(`the_option_table_matches_what_git_prints_for_dash_h`,
`worktree/worktree_add_plan_tests.rs:376`),
clustered short options are read as Git reads them
(`values_are_taken_as_gits_parser_takes_them`,
`:332`),
and a form the table does not read is reported rather than skipped
(`a_copy_destination_that_is_not_read_stays_unknown`,
`:224`).

Point A,
`Phase::Complete` can bypass marker publication:
**cannot occur.**
There is no marker to publish.
`Complete` is written only after the installation finished and the unfilled record was forgotten,
so a journal in that phase is a finished copy whose cleanup remains.

Point B,
registration-token publication must itself be atomic:
**partly closed.**
The registration's owner file is created exclusively and then written and synced,
so two commands cannot both win it
(`claim_registration`,
`worktree/worktree_add_binding.rs:496`).
A reader can see that file empty in the instant between its creation and its write,
and reads it as another command's proof.
Reaching it needs two commands on one registration,
which the reservation prevents except inside the window the SPEC already writes down
(a destination whose parent directory did not exist).
Nothing is written into the worktree by the command that reads it.

#### `review-wtc/v3`, the service plus intents

Finding 1,
the service key no longer matches the state it serializes:
**absent.**
No copy service, no service key, no per-worktree copy state.

Finding 2,
an intent lets the copy run before Git returned:
**absent.**
There is no intent.
A destination that existed before the command is another command's and is never copied into
(`ownership_of`,
`DestinationOwnership::Anothers`),
and the copy is bound to the registration it was made for.

Finding 3,
intents expire after an hour,
or a returned intent is removed after skipping its own locked destination:
**absent.**
No copy record is deleted by age.
A dead owner's item is settled, a live owner's is left whole,
and an item of another PID namespace past the patience refuses the command.

Finding 4,
giving up is reset by an unreturned intent:
**absent.**
There is no try counter, no given-up record and nothing that republishes a request.

Finding 5,
background failures and terminal-failure crash windows are silent:
**closed.**
There is no background delivery to lose:
the creating command reports the failure on its own standard error and exits 2,
and a durable record names the worktree on every later command.
Even the case where that record cannot be written is not silent:
it is a `[warn]` line on standard error
(`worktree_copy.rs:344`).

Finding 6,
reading intents before requests leaves the missing-prerequisite window:
**closed as source readiness,
and widened in this pass** by the record of a copy that ended unfilled.

Finding 7,
given-up history poisons a later source registration at the same pathname:
**absent.**
There is no given-up history.

Finding 8,
root observation and directory identity are sampled from different registrations:
**closed.**
The reservation reads the destination's identity before Git runs,
and `find_created_worktree` refuses a registration whose destination identity differs
(`worktree_copy_registry.rs:340`;
`a_destination_made_again_before_the_lookup_is_left_alone`,
`worktree_copy_lifecycle_tests.rs:724`).

Finding 9,
adopting a temporary can make its successful publisher report failure:
**absent.**
No request file is adopted;
a temporary is removed only when its writer is dead.

Finding 10,
a concurrent legacy-empty-token repair can unlink a valid winner:
**absent.**
There is no repair path: the registration owner and the stage binding are each one exclusive publication.

The retry issue the review added,
where a cached outcome stops a later message from retrying:
**absent.**
There is no cached outcome.

The unsynchronised intent and returned mark,
for process death and for power loss:
**closed.**
The claim, the destination record and the journal are each an atomic write flushed to disk
(`write_atomic`,
`worktree_copy_journal.rs:236`).
The one unsynchronised write is the completion marker,
and losing it leaves a record that the next command settles by proof rather than a copy that is forgotten.

#### The eight fixes that exist only on the branch

Each of them now has an answer in `main`,
so the branch carries no mechanism `main` lacks:

- `review-wtc` finding 1, expired claims bypass holder detection:
  `main` has no age rule, so there is nothing to port.
- `review-wtc` finding 2, `/proc` absence as evidence of death:
  open for a repository two hosts share,
  and it needs the owner's answer on whether that is a deployment to support
  (first review, finding 2 above).
- `review-wtc` finding 5, readiness from lock text:
  `main` reads no lock text.
- `review-wtc/v2` finding 1, a durable record before Git:
  `main` writes the destination record before real Git runs,
  which is the same job by a different mechanism.
- `review-wtc/v2` finding 2, one copy writer:
  `main`'s copy runs in the creating command, so there is no second writer to exclude.
- `review-wtc/v2` finding 3, a registration token:
  `main` binds the registration by the destination's directory identity.
- `review-wtc/v2` finding 4, dependency ordering:
  `main` copies only into the destination its own command made,
  so no order between worktrees is needed,
  and the source-readiness check covers the case that order was for.
- `review-wtc/v2` finding 5, durable terminal records:
  `main` reports a failed copy through the command's exit status and keeps a record that every later
  command names.

The branch's own summary of why it was written,
a service that serializes copies repository-wide,
is the design the owner declined on 2026-11-01
("do not fix these on the current repository-wide layout"),
and `main` did not need a service to close the findings that design was aiming at.

### The one open defect this pass found, and what it needs

A wrapper killed in the window between its stage claim and its destination record
leaves a claim that names no destination,
and a later command removes it without a word while the worktree stays unfilled.
This is the reviewed finding `review-wtc/v2` 1,
narrowed from the whole span between Git returning and the journal
(audit above,
first new finding)
to one directory flush.

The order is `claim_stage_for` and then the record
(`worktree_copy_stage_claim.rs:452-456`),
and a record that is gone is nothing to say
(`worktree_copy_interrupted_add_settlement.rs:411`).
Script:
`probe-kill-points.ts`.
Two runs out of two,
because a directory flush on this host is easily wider than the probe's poll:

```text
records while the copy runs: ["v1/c3d2c25b-93ba-4756-948d-b5841368f4be.stage"]
killed wrapper exit: {"code":null,"signal":"SIGKILL"}
left after the kill: ["v1/c3d2c25b-93ba-4756-948d-b5841368f4be.stage"]
later command exit: 0
later command stderr: ""
left after the later command: []
destination has its ignored payload: false
a later add exit: 0
a later add summary: ["monochromatic-git-policy-cli: copied 61 ignored filesystem entries from \"...kill-source\" into 1 new worktree."]
the first destination after the later add: false
records after the later add: []
```

The fix this pass did **not** make,
with its shape,
so it can be reviewed before anyone builds it:
make the claim itself name the destination,
so one atomic write carries both facts and no window exists between them;
then teach the reaper to report a claim that names a worktree whose record is missing,
by proving wholeness from the registration the destination binds to,
and remove the claim only once that proof holds.
The branch's answer to the same finding was a separate durable intent written before Git,
which is one more file and therefore one more window unless the intent is the only record.
Until this lands the wrapper loses a worktree's ignored-state copy without a word
when it is killed inside that window,
which the owner's requirement "failures never silent" does not allow.

### The finding that did still hold, and its fix

A worktree whose ignored-state copy **ended** before it was whole
was still another add's copy source,
although the wrapper itself had written a record saying it holds less than its source ignores.
`source_is_still_being_filled` read the claims and the journals and stopped there
(`worktree_copy_source_still_being_filled.rs:96`),
and a finished copy leaves neither a claim nor a journal, only its `.unfilled` record
(`worktree_copy_unfilled_record.rs:271`).

The consequence is the reviewed one:
a new worktree receives part of the partial source's ignored files,
the copy reports success,
and nothing says the new worktree is missing what the original source holds.
Script:
`probe-unfilled-source.ts`.
The first add fills nothing, because the source's ignored `zzz/` cannot be installed over the
checkout's tracked file of that name:

```text
first add exit: 2
first add stderr: "... ignored-state copy would overwrite differing destination entry \"zzz\" in \".../dest\"."
dest has aaa/data.txt: false
dest has zzz/data.txt: false
record files after the first add: ["v1/3bdcc03f-16fd-4fe5-a3b0-5eea6bf1b459.unfilled"]
```

The second add, made from that unfilled worktree, then copied from it and said it succeeded:

```text
second add exit: 0
second add stderr: "... \".../dest\" holds less than \".../main\" ignores: the ignored-state copy ended before it finished (the copy did not finish). ...
Preparing worktree (new branch 'topic2')
monochromatic-git-policy-cli: copied 0 ignored filesystem entries from \".../dest\" into 1 new worktree."
```

So the person read a line about the old worktree and an unrelated success line about the new one,
and the new worktree is missing everything the first add failed to install.

The same script against the binary built with the fix below,
which names the new worktree and its unfilled source,
and keeps the new add's own record so every later command names it again:

```text
second add exit: 0
second add stderr: "... \".../dest2\" was made, but it received no ignored-state copy, because the ignored-state copy of \".../dest\" ended before it was whole, and \".../dest\" holds less than its own source ignores until the record of that copy is settled. Once that worktree is whole, `git worktree remove \".../dest2\"` and run this `git worktree add` line again to have the ignored state copied; `git worktree remove` leaves a worktree that holds changes alone, so save them first."
second destination has aaa/data.txt: false
record files after the second add: ["v1/4d2bce7d-...add", "v1/4d2bce7d-...stage", "v1/f730aad8-...unfilled"]
```

What changed:

- `worktree_copying/worktree_copy_unfilled_record.rs`
gains `StandingRecord` and `standing_record_of` (`:455`),
which read the records whose worktree is still the object they name,
through the same identity proof the reporting pass already makes.
- `worktree_copying/worktree_copy_source_still_being_filled.rs`
reads that record after the claims and the journals,
answers `UnfinishedCopy::LeftUnfilled` when its owner is dead and `Running` when it still runs,
and its line names the route that settles such a source:
the person removes that worktree and makes it again,
not a command that would finish it.

Controls,
written before the change and shown red without it:

- `a_source_a_copy_ended_unfilled_is_not_staged_from`
  (`worktree_copy_source_still_being_filled_tests.rs`),
  which failed with
  "left: None, right: Some(LeftUnfilled)"
  when the record read was removed again,
  and passes with it;
- `the_notice_of_a_source_a_copy_ended_in_names_the_route_that_settles_it`,
  which pins the sentence;
- `a_record_of_a_replaced_worktree_does_not_make_the_source_busy`,
  which pins that a record of a worktree that was removed or replaced holds nothing back.

### Do any lock-based copy modules on `main` need a review of their own

No.
The copy path holds no lock of any kind:
`rg 'flock|lockf|fcntl|RwLock|Condvar'` over `worktree_copying/` finds the notice stream's
in-process `Mutex` and nothing else,
and the module named `worktree_copy_after_lock*.rs` is named for the settlement lock that was
removed on 2026-11-03,
not for one it takes.
The only kernel lock left anywhere on a worktree path is the probe that retires the legacy
conversion lock an older release left
(`legacy_lock_is_held`,
`worktree/worktree_convert_journal.rs:524`),
and that belongs to the subvolume conversion, not to the copy:
it is taken to prove the old lock is free before removing its name.
The conversion route has its own reviews and its own audits
(`review-subvol/core`,
`review-convert-2`,
`review-main-snapshot`,
`subvolume-conversion-round-3`,
and `doc/planning/cli-git-subvolume-and-conversion-route-findings-on-main.md`),
so nothing in it is reviewed by nobody.
No lock-based copy module is left that a review has not covered.

### What this triage could not determine

- Whether a repository on a filesystem shared by two hosts is supported.
  If it is, the owner evidence is not sound across hosts
  (first review, finding 2 above),
  and that needs an owner decision before any code.
- Whether the creating path, under live concurrency, can still fill a foreign registration.
  The recovery path and the replacement path are controlled;
  a live two-command race was not attempted,
  and the SPEC's two microsecond windows are unexercised.
- The behaviour of the macOS and Windows branches of owner liveness;
  no gate compiles or runs them,
  and the owner's pause covers both platforms.
- A partially installed destination end to end.
  The preflight refuses the whole installation before the first entry,
  so the probe's failed destination holds none of the source's ignored files;
  a failure in the middle of an installation (a full disk) was not produced.
- Whether the branch still compiles against this `main`.
  It was read, not built: its nineteen commits replace the in-process copy with the service,
  and the verdicts above make a build pointless.

### What to do next

The branch can be deleted once this lands:
nothing in it is now the only record of a finding or of a fix,
and the two items that survive have homes here.
The work that remains is the two items under "What this triage could not determine",
of which only the cross-host question needs the owner first.
