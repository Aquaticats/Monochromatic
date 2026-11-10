# Lock, claim and transaction route findings on `main`

Status:
audit,
2026-11-09.
Audited revision:
`main` at `9d4e5a9970`,
in the worktree
`/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/pi-locks-and-transactions-route-findings-on-main`.
No production code was changed and no test was committed.
The scratch fixtures,
the two reproduction scripts and the probe are in
`${HOME}/temp/agent/pi-locks-audit`,
and each result quotes the script that produced it.
Scope:
the shared machinery every route stands on,
in `package/git-policy/cli/src/native/`:
`lock/`,
`transaction/`,
the index-lock and claim modules under
`landing/`,
`merge/merge_isolation_journal.rs` and `merge/merge_isolation_recovery.rs`,
`process/pid_namespace.rs` and `process/process_identity.rs`,
and the callers in
`repo_read/git_directory_roots.rs`,
`invocation/entry_push_stacking_and_lfs_upload.rs`,
`worktree/` and
`subvolume/`.
Linux only:
the owner's pause covers Windows and macOS,
and this page says nothing about them.

Purpose:
single-route audits each listed that route's own waits and locks,
and none covered the machinery they share:
who a holder is,
how its liveness is proven,
how a stale holder is judged and cleared,
how Git's `index.lock` and reference locks of other programs are treated,
and how a transaction is recorded and recovered.
This page is the inventory of that machinery,
and the defects in it ranked by what they risk losing.

## What this page does not repeat

`doc/handover/cli-git-rust-implementation.md`,
section "Open review findings" (line 4795),
already records,
for this machinery:

- the index-lock claim work of 2026-11-04 (`69650ced17`,
  merged),
  whose open items are a previous release's `flock` reclaimer racing a claim reclaimer on one lock,
  a permanently unjudgeable claim reading as a transient `Busy`,
  and a lost placement answer stranding this process's own claim.
  All three are in the service's claim chain (`landing/service_index_claim.rs`),
  which is not what this page reports.
- "the 20-minute foreign-owner patience can authorize reclaiming a still-live owner now that writes may run past any ceiling",
  with an owner question to follow.
  That owner answer has since been given ("Refuse and name it",
  2026-11-04),
  so expiry now refuses instead of reclaiming;
  the finding about the patience on this page is about the **record's age** as evidence,
  not about what expiry authorizes.
- the copy route's journals,
  stage claims and owner records,
  audited in
  `doc/planning/cli-git-copy-route-findings-on-main.md`.
  This page cites that machinery only where a shared rule decides its behaviour.

Nothing below is one of those findings.

## The inventory

Each item is one file or one directory entry,
not a topic.
"Ceiling" is the longest anyone waits;
"no wait" means the answer is immediate,
whatever it is.

### The foreign `index.lock` wait

- What it is:
  another program's `index.lock`,
  waited for before a command changes the checkout.
- Where:
  `lock/index_holder_wait.rs:167-250`,
  evidence in `lock/index_lock_evidence.rs`,
  the descriptor scan in `lock/index_lock_holders.rs`.
- Holder record:
  none in the lock.
  Evidence is a process with the file open (device,
  inode)
  or Git's `index~pid.lock` (`core.lockfilePid`,
  which the wrapper injects),
  naming a PID whose start time is compared with the lock's change time
  (`lock/index_lock_evidence.rs:280-300`).
- Who waits:
  a forwarded `merge` or `pull`'s merge
  (`lock/index_writer_forward.rs:221`,
  `:237`),
  the index reconciliation of a landing
  (`landing/service_land_follow.rs:267`).
- Ceiling:
  `MERGE_LOCK_CEILING`,
  one minute
  (`service/wait_ceilings.rs`),
  and `RECONCILE_LOCK_CEILING` for the reconciliation.
- Dead holder:
  a PID file naming a process that no longer runs ends the wait at once with a diagnostic
  (`lock/index_holder_wait.rs:215-219`).
  Nothing is removed on any verdict.
- Removable:
  no.
  Removing takes a live Git's index update away from it,
  which is the reason the decision record gives ("cli-git never deletes a lock").
  The wait itself could be dropped for the merge path,
  at the price of a merge that fails at once behind a person's editor-held commit.

### The service's own `index.lock`

- What it is:
  the lock the landing writes the new index through.
- Where:
  placement in `landing/service_index_lock.rs:280-350`,
  the marker in `landing/service_index_marker.rs`,
  the holder judgement in `landing/service_index_identity.rs`,
  the reclaim in `landing/service_index_reclaim.rs:164-230`,
  the claim chain in `landing/service_index_claim.rs:413`.
- Holder record:
  `cli-git-service v2 <batch> <pid> <birth>#<boot>#<pid namespace>#<machine>`,
  written into the lock itself before it is placed.
- Who waits:
  the landing (one per worktree),
  and a forwarded index writer for the service's hold
  (`lock/index_writer_forward.rs:213`).
- Ceiling:
  `LANDING_LOCK_CEILING` (one minute) at the lock,
  `SERVICE_HOLD_CEILING` (ten minutes) for a forwarded writer's whole series.
- Dead holder:
  a marker whose holder is gone (another boot of this machine,
  no such PID,
  or a PID born at another tick) is reclaimed through the claim chain.
  A marker that cannot be compared is left and named,
  with the `rm` (`landing/service_index_reclaim.rs`,
  `SPEC.md` "One reclaimer per stale lock").
- Removable:
  no.
  It is the mechanism.
  One owner per file is already what it is,
  and no lock primitive is involved
  (`landing/service_index_claim.rs:414` records the reason).

### An empty, unmarked `index.lock`

- What it is:
  the owner's answer of 2026-11-07,
  "Service removes it":
  an empty lock that no readable process holds and that has no PID file at all.
- Where:
  `landing/service_index_unheld_removal.rs:146-215`.
- Holder record:
  none.
  The proof is the file's own identity (device,
  inode,
  size),
  a complete scan of every readable process,
  the absence of any PID file,
  and exactly one descriptor of the service's own on it.
- Who waits:
  nobody.
  The service removes it and lands the commit.
- Ceiling:
  not a wait.
- Dead holder:
  this **is** the dead-holder path,
  and it acts only on the proof above.
- Removable:
  it removes,
  by decision,
  and states in the journal and to the person what it could not see.
  The scan it trusts is where finding 4 lands.

### A reference lock of another Git

- What it is:
  `packed-refs.lock`,
  `HEAD.lock`,
  any `*.lock` under `refs/`,
  and an orphan `packed-refs.new`.
- Where:
  found by `lock/stale_ref_lock_walk.rs:70-120`,
  judged by `lock/stale_ref_lock_judge.rs:477-543`,
  removed by `lock/stale_ref_lock_retire.rs:65-175`.
- Holder record:
  Git writes none.
  The proofs are a PID file born at or after the lock naming a process that is gone or whose number was reused,
  a descriptor held open (a veto,
  never a proof),
  a staging file with no lock beside it,
  and,
  after a stop,
  the wrapper's own observation that a member of the stopped tree ended
  (`lock/stopped_git_locks.rs`).
- Who waits:
  nobody.
  A forwarded `fetch` or `push` judges and clears before Git starts
  (`lock/forwarded_lock_notice.rs:50-70`),
  the wrapper's own pushes judge and clear before every attempt
  (`lock/push_lock_clearing.rs`),
  and a stopped child's tree is swept after it ends.
- Ceiling:
  no wait;
  a lock younger than the seven-second margin is not even looked at
  (`lock/stale_ref_lock_judge.rs:126-140`).
- Dead holder:
  removed through a `RENAME_NOREPLACE` to a private name,
  an identity check of what was moved,
  and a removal,
  or a hard link back when the file that was taken is not the one judged
  (`lock/stale_ref_lock_retire.rs:84-115`).
  Anything unproven is left and named with the `rm`.
- Removable:
  the walk is the part that is incomplete;
  see finding 1.

### A worktree conversion or a checkout migration claim

- What it is:
  one command's right to convert one worktree
  (`worktree/worktree_convert.rs:157`) or to migrate one checkout
  (`subvolume/subvolume_command.rs:171`).
- Where:
  `lock/single_owner_claim.rs`.
- Holder record:
  a numbered file `<prefix>.<generation>`,
  holding `monochromatic-git-policy-cli owner-claim v1`,
  `pid=`,
  `birth=` (`linux:<start tick>`) and `namespace=`.
- Who waits:
  nobody.
  A live holder is an immediate `Busy`;
  a released or dead one means the next generation is taken.
- Ceiling:
  no wait.
  The only repeated path is `MOST_ROUNDS` (64) rounds against other claimants,
  then an error
  (`lock/single_owner_claim.rs:70`,
  `:361-400`).
- Dead holder:
  the next command takes the next generation.
  The newest generation file is never removed;
  older ones are removed by whoever takes a newer one.
- Removable:
  no.
  It is one owner per item with no lock and no wait,
  which is what the owner asked for.
  Its liveness rule is where finding 2 lands.

### A merge's intent record and a legacy transaction owner record

- What it is:
  the durable record that lets the next command finish or discard a merge whose process died.
- Where:
  `merge/merge_isolation_journal.rs:191` writes it,
  `merge/merge_isolation_recovery.rs:78-190` judges and settles it,
  the owner liveness rule in `transaction/transaction_owner.rs:233-275`.
- Holder record:
  JSON naming `ownerPid`,
  `ownerIdentity`,
  `createdAt` and,
  since 2026-11-04,
  `ownerPidNamespace`.
- Who waits:
  nobody.
  A record whose owner is alive is skipped.
- Ceiling:
  no wait.
- Dead holder:
  recovery discards a merge that never touched the worktree or the index,
  and completes one that did.
  A record that cannot be read stops the command with its path.
  A record of another PID namespace past the patience is refused and named.
- Removable:
  no.
  Its liveness rule shares finding 2.

### The service's work-item claims

- What it is:
  the claim on one after-commit work item (a push,
  upkeep,
  trash,
  a review).
- Where:
  `landing/work_item.rs`,
  the lease in `landing/claim_time.rs`.
- Holder record:
  the claim file's name carries the holder's clock reading:
  boot ID,
  monotonic milliseconds,
  wall seconds and machine ID.
- Who waits:
  nobody;
  a queued item waits its turn in `pending` and is taken by rename.
- Ceiling:
  none.
  A lease is how a later instance learns a claim was abandoned.
- Dead holder:
  the reading decides:
  the same boot is compared by the monotonic clock,
  a different boot of the same machine is dead at once,
  everything else is compared by the wall clock against the lease.
- Removable:
  no.
  This is the only claim whose liveness survives a reboot and a clock step,
  because the boot ID and the monotonic clock are in the record
  (`SPEC.md`,
  "Leases of claimed work").
  It is the model the other records do not follow.

### The private copy of the landing's index

- What it is:
  the directory a landing builds the new index in,
  beside the live index.
- Where:
  `landing/service_land_copy.rs:150-290`.
- Holder record:
  a marker naming the owning process,
  plus an exclusive `flock` on the directory.
- Who waits:
  the sweep of abandoned copies in a later landing.
- Ceiling:
  none;
  the `flock` is released by the kernel when the owner dies.
- Removable:
  a candidate.
  The marker already names the owner and the sweep already judges it,
  so the `flock` is a second mechanism for the same question;
  it is kept for the kernel's release on death,
  and the code records that reason beside it.

### The kept lock primitives

- `service/journald_native.rs`:
  one mutex around one non-blocking write of one journal line,
  the only lock primitive in the service and its support modules.
  `source_guard/service_lock_inventory_tests.rs:61` fails any other.
- `worktree/worktree_convert_journal.rs:540-575`:
  a `flock` probe of the lock file releases up to 0.5.20 left,
  taken and given back only to learn whether an old command still holds it.
  It removes the file when nothing holds it.
  It can go when no 0.5.20 or older command is supported.
- `foundation/earlier_log_lines_*.rs`:
  a `flock` around folding one log generation,
  outside this route.

## Findings, ranked by what they risk losing

### 1. A killed Git's reftable lock blocks every command and is never named

**Reproduced.**

Where:
`lock/stale_ref_lock_walk.rs:89-110` builds the candidate list from
`packed-refs.lock`,
`HEAD.lock`,
an orphan `packed-refs.new` and the walk of `refs/`.
A reftable repository keeps none of those:
its refs and its lock live in the `reftable` directory of the Git directory
(`reftable/tables.list`,
`reftable/tables.list.lock`).

The sequence:

1.  A reftable repository is in use.
    It is a supported configuration:
    `package/git-policy/cli/SPEC.md:7015` says the container end-to-end suite covers reftable.
2.  A Git process is killed at any instruction while it holds the reftable lock
    (the closed terminal,
    the out-of-memory killer,
    a reboot;
    `AGENTS.md` rule `NHP`).
    It leaves `.git/reftable/tables.list.lock`.
3.  Any later command that writes a ref fails:
    `git update-ref` exits 128 with `fatal: update_ref failed for ref 'refs/heads/probe': cannot lock references`,
    and `git commit` exits 128 with `fatal: cannot lock references`
    (quoted in "Reproductions").
4.  The wrapper judges and clears before a forwarded `fetch` or `push`
    (`lock/forwarded_lock_notice.rs:50-70`),
    and it finds nothing:
    the lock is not a candidate,
    so it is neither removed nor named.
    No notice,
    no path,
    no `rm` line reaches the person.

The wrong result:
a repository that cannot take a commit at all,
with no statement of which file stands in the way,
while a files-backend repository in exactly the same state is repaired and told about
(quoted side by side in "Reproductions").
This is the defect the machinery was built for
(`package/git-policy/cli/SPEC.md:4018-4030`:
locks left by a killed Git "blocked every later `git fetch` and `git push` ... until someone deleted the file by hand"),
still open for one ref backend,
and it breaks the rule the section states:
"otherwise the wrapper says what it found and prints the command that clears it".

Suggestion,
not a change made here:
add `reftable/tables.list.lock` (with its `tables.list~pid.lock` companion)
to `ref_lock_candidates`,
and the reftable directory's staging files to the orphan rule,
with the same proof,
the same set-aside step and the same notice.
The judgement needs no new proof:
Git writes `tables.list~pid.lock` beside it exactly as it writes the other companion files.

Not claimed:
whether any other reftable lock (`reftable/tables.list.lock` only,
or per-table temporary files) also blocks a writer.
The measured one is `tables.list.lock`.

### 2. A record whose file time cannot be measured is a live owner for ever, and one whose time is old is refused while it lives

**Reproduced,
both directions.**

Where:
`process/pid_namespace.rs:151-168` (`foreign_record_age`) and
`:263-270` (`foreign_owner_standing`):

- the age of a record is `SystemTime::now() - the file's modification time`;
- an age that cannot be measured,
  which is every file whose modification time is later than this reader's clock,
  answers `None`,
  and `foreign_owner_standing` maps `None` to `Ok(())`,
  that is,
  **alive**;
- an age greater than `FOREIGN_OWNER_PATIENCE` (twenty minutes) is refused and named,
  with the `rm`.

The rule is used by every record of another PID namespace:
the conversion and migration claim (`lock/single_owner_claim.rs:306-320`),
the merge intent (`transaction/transaction_owner.rs:261-275`),
and the worktree-copy journals and stage claims.

The sequence,
first direction:

1.  A command runs in a container,
    or on another host that shares the repository,
    and writes a claim or an intent record into it.
    Its record is written once and never rewritten,
    so the file's time is the whole hold.
2.  The host's clock is stepped back,
    or the record was written by a machine whose clock is ahead,
    or a copy preserved a future time.
    The record's modification time is now later than the reader's clock.
3.  The command dies.
4.  Every later command that reads the record gets `Ok(())` from `foreign_owner_standing`,
    which means "a live owner",
    and never reaches the age comparison that would refuse it.
    A conversion or migration of that checkout is refused with
    "process N is migrating the main checkout ... wait for it to finish"
    (`worktree/worktree_convert_inspection.rs:229`);
    an interrupted merge is never settled,
    because `merge_intents_to_recover` (`merge/merge_isolation_recovery.rs:78`) skips it.

The wrong result:
the person is told a process is working right now,
about a process that no longer exists,
with no age and no `rm`,
for as long as the file's time stays ahead of the clock.
The third answer the rule was built to give (neither alive nor dead,
refused with the file,
the age and the `rm`;
`SPEC.md` "Foreign-owner expiry" at line 3213) is unreachable in this state.

The sequence,
second direction:

1.  A record of another namespace is older than the patience
    because its owner is a long command
    (the claim is taken at the start of the whole conversion or migration),
    or because this reader's clock is ahead of the record's.
2.  The owner is alive.
3.  The reader refuses,
    says the record is past the patience,
    and gives `rm -- '<record>'` for a person who finds "no monochromatic-git-policy-cli command running".

The wrong result:
a live holder told its record is expired,
and an `rm` that would let two commands work on one checkout
if it is taken while the holder lives.
The module's own text says the premise is "the patience must outlast the longest such hold"
(`process/pid_namespace.rs:29-95`),
and the basis it cites is other commands' durations,
not the two commands whose record covers their whole run.

Suggestion,
not a change made here:
judge an age that cannot be measured as the third answer,
not as alive.
Reading a file time later than the clock as "alive for ever" is the one outcome the rule's own text forbids:
it is neither a proof of life nor of death.
A future time is a state the record can enter without any command's fault (a clock step,
another host),
so it deserves the same refusal as an expired one,
which names the file and the `rm`.
That change alone closes the first direction;
the second needs the patience's basis widened to a conversion's whole run,
or a record that is renewed.

### 3. A clock step flips the index lock's PID-file proof both ways

**Reproduced at the seam** the module offers for it
(`lock/index_lock_evidence.rs:385`,
`gather_with`).

Where:
`process/process_start.rs:187` derives a process's start as
`now - uptime + ticks`,
so every live process's derived start moves with the wall clock,
while the lock file's stored change time does not;
`lock/index_lock_evidence.rs:280-300` then compares them:

- a start later than the lock's change time is read as "the number was reused",
  so the holder is not proven;
- a start no later than the change time is read as "the holder is running".

The sequence,
first direction:

1.  Another program's Git holds `index.lock` with `core.lockfilePid` on,
    so `index~pid.lock` names it.
2.  The wall clock steps forward (an NTP correction,
    a person's `date`,
    a dual-boot machine).
3.  The holder's derived start is now an hour later than the lock's change time,
    so the evidence classifies as `EvidenceFree`.
4.  A commit's landing waits the whole `LANDING_LOCK_CEILING`,
    then withdraws the commit and keeps it on a side ref
    (`SPEC.md` "Every wait has a bound",
    line 2730);
    a merge waits `MERGE_LOCK_CEILING` and then meets Git's own refusal.

The wrong result:
a commit that would have landed fails,
and nothing in the message says that a clock step is why.

The sequence,
second direction:

1.  A Git was killed holding `index.lock`,
    and its `index~pid.lock` names a PID that has since been reused
    by a process that is not a Git.
2.  The wall clock steps back,
    so the young process's derived start lands before the lock's change time.
3.  The evidence proves a live holder that does not exist.
4.  Every attempt waits the whole ceiling,
    and the ceiling line names that process
    ("PID N (proven by its Git lock PID file)") and says to run the command again once it is done,
    which never happens.

The wrong result:
a minute of waiting on every attempt,
and a statement about the holder that is false.

Suggestion,
not a change made here:
the refusal of the clock is already the chosen rule for reference locks,
whose judgement uses no time as a proof
(`lock/stale_ref_lock_judge.rs:1-20`);
the index-lock PID-file proof could take the same shape,
using the process's start tick and the boot ID,
which the service's own marker already carries,
instead of a wall-clock difference.
The shorter change is to stop treating "started after the lock" as proof of reuse:
the only proof of a dead owner that does not rest on the clock is the process being gone.

Not claimed:
how often a clock step happens here.
What is claimed is the direction:
the same evidence answers "reused" or "running" by the clock alone.

### 4. A descriptor is matched by mount and inode, which is neither unique nor complete

**Reproduced** on btrfs,
this repository's own filesystem.

Where:
`lock/index_lock_holders.rs:144-200` matches a descriptor by the `mnt_id` and `ino` lines of
`/proc/<pid>/fdinfo/<n>`,
and `:400-430` passes the lock's own mount from `process/process_use.rs:161`;
`process/process_use.rs:178-190` uses the same pair for the veto in
`lock/stale_ref_lock_judge.rs:250-260`.

Two facts make the pair not an identity:

- On btrfs,
  inode numbers are unique only inside one subvolume,
  so two files in two subvolumes of one filesystem share an inode number,
  and they are reached through the same mount when the subvolumes are directories of one mount.
  Measured on this host:
  `ino=257 sv-a/index.lock` and `ino=257 sv-b/held`,
  both with `mnt_id: 297`.
  Worktree subvolumes are exactly what the conversion feature creates,
  so this is the layout of an owner's repository after it uses the feature it ships.
- A bind mount of the same file has a different mount number,
  so a holder that reached the file through a bind mount is not found at all.

The sequence,
first direction:

1.  A landing,
    merge or reconciliation is about to take `index.lock`
    whose inode number is shared with a file in another subvolume.
2.  Any process on the host holds that other file open.
3.  The scan reports it as a holder of the lock,
    and the verdict is `ProvenAlive { by_descriptor: true }`
    with the lock named,
    the process named,
    and its command.
4.  The wait runs its whole ceiling and fails,
    with a statement that the person's lock is held by a process that has nothing to do with it.

The wrong result:
a minute of waiting per attempt and a false accusation,
repeated while that descriptor is open.
Quoted output in "Reproductions":
the scan answered `Holder { pid: 3625949, command: Some("sleep") }` for a lock that no process held,
because a `sleep` held a file with the same inode number in another subvolume.

The sequence,
second direction:

1.  A live Git has created `index.lock` and not yet written the index into it.
2.  It reached the lock through a bind mount,
    so the descriptor's mount number differs from the lock's.
3.  The scan finds no holder.
4.  The service's unheld removal (`landing/service_index_unheld_removal.rs:146-215`) removes the empty lock,
    because its proof is precisely "no PID file,
    no holder the scan sees".
5.  The live Git's later rename of its lock into place either fails,
    or,
    when another writer has put a lock at the path meanwhile,
    installs that writer's file as the index.

The wrong result:
staged content replaced by another writer's half-written index.
That is the hazard the specification spells out for reference locks
(`package/git-policy/cli/SPEC.md`,
"Rejected rule:
nothing has the lock open",
line 4108:
"the first one installs the second one's half-written file as the reference"),
reached here for the index.
The window is narrow (Git writes the index into the lock before it runs hooks or an editor),
and the preconditions are a bind mount plus a concurrent placement,
which is why this ranks below findings 1 to 3.
The module's own text claims the opposite of what the pair can do:
"bind mounts and renamed paths cannot fake or hide a holder"
(`lock/index_lock_holders.rs:1-20`).

Suggestion,
not a change made here:
the pair is used for two opposite decisions,
and each needs the other repair.
For "a holder exists",
a hit from another subvolume must not count,
so the comparison needs a number that is unique across subvolumes,
which a file's generation (`statx`) or its descriptor's own link target gives,
at the cost of the stalled-disk wait the code avoided
(`lock/index_lock_holders.rs:130-142`).
For "no holder exists",
which is what the unheld removal stands on,
the scan must be able to see a holder that reached the file by another mount,
or the removal must rest on the lock's own marker and PID file alone,
which a plain Git does not write.

Not claimed:
how often inode numbers collide in the gate's overlay filesystem.
Measured on btrfs only,
where it is the normal layout of two subvolumes.

### 5. One decision record says the wrapper never deletes a lock, and three modules delete locks

Read from the records and the code;
nothing to reproduce.

- `doc/decision/cli-git-concurrent-commits.md:166` says
  "cli-git never deletes a lock.",
  under "## Locks",
  with no amendment in that record.
- `lock/stale_ref_lock_retire.rs:84-115` deletes a reference lock on a proof about its holder,
  and says so in
  `package/git-policy/cli/SPEC.md` "The rule:
  a lock is removed only on a proof about its holder" (line 4129).
- `landing/service_index_reclaim.rs:164-230` deletes a stale `index.lock` of the service
  on a proof about the marker,
  by the owner's answer of 2026-11-04 ("Claim files always").
- `landing/service_index_unheld_removal.rs:146-215` deletes an empty unmarked `index.lock`
  on the owner's answer of 2026-11-07 ("Service removes it").

Only the first is a disagreement a reader can be misled by,
and the brief asked for it explicitly:
the sentence is true of **another program's** `index.lock` on the forwarded path
(nothing on `lock/index_holder_wait.rs` removes anything),
and false of reference locks and of the service's own index lock.
The record's own "Implemented 2026-11-03" paragraph amends its two neighbouring bullets
(the unbounded wait and the backoff budget)
and leaves this one standing.

Suggestion,
not a change made here (the audit was told not to edit `doc/decision/`):
replace the line with what the code does:
the wrapper never deletes another program's `index.lock`;
it deletes a reference lock only on a proof about its holder;
and the service deletes the `index.lock` it placed,
or an empty one whose proof the record states.

## What a kill at each state change leaves

Read from the code;
the ordering claims are `path:line` above.

### Taking a claim (`lock/single_owner_claim.rs:359-400`)

- Killed before `publish_new`:
  nothing on disk;
  the next command takes the next generation.
- Killed after the exclusive `hard_link` and before the newer-generation check:
  a generation naming a dead PID and birth;
  the next command judges it dead and takes the next.
- Killed after removing the older generations:
  the same as above;
  the newest generation is the claim.
- Killed during `release` (a rename of a released record over the claim):
  either the old record or the released one,
  never a mixture;
  a rename is atomic.
  A released record is taken over without judging a process.
- Killed between `release` and the command's end:
  a released claim,
  which the next command takes over.
- What notices a leftover:
  the next command's `live_owner` (PID and birth),
  and nothing else.
  No work is lost,
  because the claim holds no state;
  the state is the journal,
  written exclusively and never replaced
  (`worktree/worktree_convert_journal.rs`).

### A merge's intent (`merge/merge_isolation_journal.rs:191`)

- Killed before the record is written:
  nothing was changed;
  Git had not started.
- Killed after the record,
  before Git starts:
  recovery finds no merged content and no index entry of the merge and discards the record.
- Killed while Git updates the worktree and the index:
  recovery completes the merge from the record,
  for exactly the merge's paths,
  and moves the branch by compare-and-swap.
  A path holding neither its old nor its new content stops recovery and keeps the record.
- Killed after the branch moved,
  before the record is removed:
  recovery finds the branch at the new commit and removes the record.
- What changes this:
  only finding 2.
  When the record's owner is read as alive (a future file time),
  recovery never looks at it,
  and the half-updated worktree stays with no notice.

### The service's index lock (`landing/service_index_lock.rs:280-430`)

- Killed after the staged marker is flushed,
  before the placement:
  a staging file naming this process;
  the next successful acquisition removes it when the process is gone
  (`SPEC.md`,
  "The marker format and earlier releases").
- Killed after the placement,
  before the branch moves:
  a whole marker of a dead process;
  the next acquisition reclaims it through the claim chain.
- Killed on a filesystem that cannot place a claim (no `RENAME_NOREPLACE`,
  no `link`):
  the lock stays,
  the notice names it with the `rm`,
  and the commit is withdrawn and kept.
- Killed after the branch moves but before the index is installed:
  the batch stays open and the restart replays the index update,
  with the recorded entries of the index paths so a path staged meanwhile keeps its staging
  (`SPEC.md`,
  "Removing the lock and what is made durable").
- Killed between the index rename and the `<batch>.indexed` record:
  the completion record is missing,
  so the restart replays the index update.
  The window is stated in the specification as the case the record exists for.

### A reference lock's removal (`lock/stale_ref_lock_retire.rs:84-115`)

- Killed after the rename to the private name,
  before the identity check:
  the judged file is left at `cli-git-stale-lock-*`,
  not removed,
  and the lock's name is free.
  The next writer places a new lock;
  the leftover is inert and no notice names it.
  This is a file left behind,
  not data lost.
- Killed after the identity check,
  before the removal:
  the same leftover.
- Killed after the removal of the lock,
  before its companions:
  the PID file and `packed-refs.new` stay,
  and the next judgement of a lock that is no longer there does not look at them.
  A `packed-refs.new` with no lock beside it is judged as an orphan on the next pass and removed.
- What notices it:
  the next pass over the same directory;
  nothing else remeasures these files.

### A work-item claim (`landing/work_item.rs`, `landing/claim_time.rs`)

- Killed between the take and the publication into `claimed`:
  an entry in `recovering`,
  which runs out its lease and is recovered like a claim
  (`SPEC.md`,
  "Leases of claimed work").
- Killed after the publication:
  the lease in the file name decides.
  This is the only claim whose liveness survives a reboot and a clock step.

## What I checked and found sound

- The generation claim has no two-owner outcome I could construct.
  The exclusive `hard_link` (`lock/single_owner_claim.rs:273`) lets one claimant create a generation;
  a claimant that publishes below a newer generation withdraws
  (`:379-390`);
  older generations are removed only by the winner;
  the newest generation is never removed,
  so no claimant can re-create a number another already read.
  A live owner is judged by PID **and** birth,
  so a reused PID is not the owner.
- The birth identity's only gap is across a reboot:
  it is `linux:<start tick>` (`process/process_identity.rs:195`) with no boot ID,
  so a record from an earlier boot would need the same PID and the same start tick
  (the same centisecond of the new boot) to be read as alive.
  The service's own marker carries the boot ID,
  the PID namespace and the machine ID
  (`landing/service_index_identity.rs`),
  and the work-item lease carries boot and monotonic clock,
  so the omission is only in the claim and the transaction owner record.
  I found no reachable sequence that meets the coincidence.
- The service's marker judgement is careful in every branch I traced:
  another boot of the same machine is gone,
  another machine or an invalid machine ID is unknown,
  another PID namespace is unknown,
  a `/proc` that does not agree with its own PID is unknown,
  and a marker with no holder is never gone
  (`landing/service_index_identity.rs:311-390`).
- The claim chain has one holder per stale file:
  a claim is placed whole,
  a dead claimer's claim is passed over and never removed while the file stands,
  the slots below are read again after placing,
  and the name is re-checked and re-unlinked as one step
  (`landing/service_index_claim.rs:413-490`).
  I found no interleaving in which two reclaimers both unlink the stale file,
  which is what it is for.
- The ref-lock removal re-checks the file's identity immediately before the move and puts back a file
  that is not the one judged,
  with a hard link that never replaces
  (`lock/stale_ref_lock_retire.rs:84-115`).
  The residue it leaves (three writers and another cleaner inside the window) is stated in the
  specification and in the handover,
  so it is not reported again here.
- A dead index-lock owner ends the wait at once and nothing is removed
  (`lock/index_lock_evidence.rs:370`,
  `lock/index_holder_wait.rs:215-219`).
  The forwarded `index.lock` path never removes anything.
- The waits on this route all end:
  `MERGE_LOCK_CEILING`,
  `RECONCILE_LOCK_CEILING` and `LANDING_LOCK_CEILING` are one minute,
  `SERVICE_HOLD_CEILING` ten,
  `ANSWER_CEILING` twenty,
  each with the measurement beside it in `service/wait_ceilings.rs`.
  The one write with no ceiling is the branch write,
  which is the owner's decision of 2026-11-04 ("Never cut a branch write").
- No path holds a lock of the wrapper's own across a wait on another program,
  a network call or an editor.
  The landing holds `index.lock` for the branch move and the index build only
  (`SPEC.md`,
  "Locks and waits of the service,
  by item"),
  the conversion holds it only across the in-process index refresh and releases it before the long removal
  (`worktree/worktree_convert_phases.rs:375-395`),
  and a forwarded index writer waits before Git starts,
  holding nothing
  (`lock/index_writer_forward.rs:213-240`).
- The retired configuration keys are inert and told so:
  `indexLock.unprovenOwnerTimeoutMs`,
  `hooks.concurrentCommits` and `landing.reserveAfterLostRaces`
  are parsed,
  checked and ignored,
  and `doctor` names a file that sets one
  (`native/config/config_schema.rs:215-260`,
  `package/git-policy/cli/SPEC.md` "Concurrency configuration",
  line 275).
- `repo_read` skips any name ending in `.lock` when it lists refs
  (`repo_read/repo_read_refs.rs:112`,
  `:312`),
  so a lock is never read as a reference.
- The service's unheld removal asks for more than the scan:
  an empty file,
  no PID file at all,
  a complete scan,
  exactly one descriptor of the service's own,
  and the name still referring to the opened file
  (`landing/service_index_unheld_removal.rs:146-215`).
  Finding 4 is the one input it trusts that can be wrong.

## What I could not determine

- Whether the reftable omission is deliberate.
  No record in `doc/decision/` and nothing in
  `package/git-policy/cli/SPEC.md`
  says reftable locks are excluded;
  `SPEC.md:4131` lists the candidates as if the list were complete.
  If it is deliberate,
  the notice's promise needs the exception written down.
- Whether a push's exit 0 beside its tracking-ref refusal
  (`error: update_ref failed for ref 'refs/remotes/origin/second': cannot lock references`
  in "Reproductions")
  is Git's behaviour or the wrapper's.
  Plain Git 2.56.0 prints the same line and exits 0 with no wrapper on `PATH`,
  so it is Git's,
  and it is recorded here only because it decides what a reader of the wrapper's exit code sees.
- How long a real `convert-worktree-to-subvolume` or `migrate-to-subvolume` holds its claim.
  No log records those commands,
  so the second direction of finding 2 has a mechanism and no measurement.
  The patience's own basis cites other commands' durations
  (`process/pid_namespace.rs:29-95`).
- How often two btrfs subvolumes share an inode number in practice.
  The mechanism and one measured pair are in "Reproductions";
  a census over an owner's repository was not made.
- Whether the gate's overlay filesystem produces the same inode collisions.
  Only btrfs was measured.
- Whether the service's landing,
  in a reftable repository,
  surfaces the reftable lock's path in its log
  or the client's message.
  That needs a commit through a service,
  and no service was started by this audit.
- Whether the out-of-date precondition in finding 4's second direction can be met in practice
  (a bind-mounted path into a repository whose index a service writes).
  The code path is read;
  the live race was not attempted.

## Reproductions

Every result below ran against a debug build of the audited revision,
started by path,
with `PATH` set to a fixture directory first,
`HOME` set to a fixture home,
and `GIT_CONFIG_NOSYSTEM` and `GIT_CONFIG_GLOBAL` pointing at nothing.
No service was started,
and nothing was run against the installed wrapper or this repository.

### The reftable lock and the files-backend lock, side by side

Script:
`${HOME}/temp/agent/pi-locks-audit/reproduce-ref-lock.sh`.

```text
=== files backend: planted .../files/.git/refs/remotes/origin/other.lock, aging 9 s ===
  exit=0
  monochromatic-git-policy-cli: removed the reference lock .../files/.git/refs/remotes/origin/other.lock, left by a Git that is gone: process 999999, named by the pid file beside it, has ended.
  From .../ref-lock/files
   * [new branch]      other      -> origin/other
  files in refs/remotes/origin: main other
=== reftable backend: planted .../reftable/.git/reftable/tables.list.lock, aging 9 s ===
  exit=251
  From .../ref-lock/reftable
   * [new branch]      other      -> origin/other
  error: cannot lock references
  files in .git/reftable: 0x000000000001-0x000000000003-8284f85b.ref tables.list tables.list.lock tables.list~pid.lock
=== reftable backend: a push, whose tracking-ref update meets the same lock ===
  exit=0
  To .../ref-lock/reftable.git
   * [new branch]      HEAD -> second
  error: update_ref failed for ref 'refs/remotes/origin/second': cannot lock references
```

What plain Git does in the same state,
with no wrapper (Git 2.56.0,
throwaway repository):

```text
$ git update-ref refs/heads/probe HEAD
fatal: update_ref failed for ref 'refs/heads/probe': cannot lock references
exit=128
$ git commit --quiet -m two
fatal: cannot lock references
exit=128
```

Both repositories had the same kind of leftover,
planted the same way and aged the same way.
The files-backend one was removed with a line naming the file,
the process and the end of that process,
and the fetch then updated its tracking ref and exited 0.
The reftable one was not named at all:
the fetch exited 251 with Git's own words,
the push exited 0 with its tracking-ref refusal,
and the lock is still there.
The first block is the wrapper's promise;
the second and third are its silence where the same promise applies.

### A record whose file time cannot be measured

Script:
`${HOME}/temp/agent/pi-locks-audit/reproduce-claim-age.sh`,
which plants a claim of another PID namespace (naming a PID that does not exist)
beside a checkout and runs the conversion's dry run,
which reads it:

```text
--- mtime now: a young record of another namespace (documented: counts as alive)
    skipped: process 999999 is migrating the main checkout .../space/main-repo to a subvolume (it owns .../space/.cli-git-migrate-main-repo.owner.1); wait for it to finish, then run this command again
--- mtime 21 minutes ago: the documented expiry, which names the file, the age and the rm
    skipped: whether the main checkout .../space/main-repo is being migrated cannot be decided (monochromatic-git-policy-cli: the record .../space/.cli-git-migrate-main-repo.owner.1 was written by process 999999 in the PID namespace pid:[1], which is not this one, and it is 21 minutes old, past the 20 minutes after which monochromatic-git-policy-cli no longer takes such a record for a live owner's. Nothing was changed. ... when none runs, remove the record with `rm -- .../space/.cli-git-migrate-main-repo.owner.1` and run the command again.)
--- mtime 100 years ahead: the age cannot be measured
    skipped: process 999999 is migrating the main checkout .../space/main-repo to a subvolume (it owns .../space/.cli-git-migrate-main-repo.owner.1); wait for it to finish, then run this command again
  stat: 2126-11-09 01:02:05.821048556 -0500
--- again two seconds later: the same live-owner answer, and no expiry
    skipped: process 999999 is migrating the main checkout ... the same line
```

Each block is elided where the output repeats the rule's own text;
the file name,
the age,
the `rm` line and the answers are as printed.
The third and fourth blocks are the finding:
the file's time is a hundred years ahead,
the named process does not exist,
and the answer is "wait for it to finish",
with no age and no `rm`,
for as long as the file's time stays ahead of the clock.

### The clock step, through the module's own seam

Probe:
`${HOME}/temp/agent/pi-locks-audit/probe`,
`probe a <dir>`,
which builds the evidence with `gather_with` and the start times the host would read
before and after a one-hour wall-clock step.

```text
      start read now: ProvenAlive pid=4242 by_descriptor=false command=None
                      pid file: Owner(4242, StartedBeforeLock)
    one hour forward: EvidenceFree
                      pid file: Owner(4242, StartedAfterLock)
   one hour backward: ProvenAlive pid=4242 by_descriptor=false command=None
                      pid file: Owner(4242, StartedBeforeLock)
```

The lock and the PID file are unchanged between the three lines;
only the start time the host derives moves,
exactly as a clock step moves it.

### A descriptor that is not the lock

Probe:
`probe b <lock>`,
with two btrfs subvolumes of one filesystem,
`sv-a/index.lock` and a `sleep` holding `sv-b/held` open:

```text
ino=257 sv-a/index.lock
ino=257 sv-b/held
  scan: method=proc-fd holders=[Holder { pid: 3625949, command: Some("sleep") }, Holder { pid: 3596786, command: Some("monochromatic-g") }] partial=[...]
  verdict: ProvenAlive pid=3625949 by_descriptor=true command=Some("sleep")
  the lock itself: inode 257 mount 297
```

No process held `sv-a/index.lock`.
The scan named two processes that held other files with the same inode number on the same mount,
and the verdict is a live holder of the lock.

## What to inspect and how to respond

Inspect,
in this order:

1.  the reproduction of finding 1,
    against `lock/stale_ref_lock_walk.rs:89-110`
    and `lock/forwarded_lock_notice.rs:50-70`,
    and the plain-Git block that shows the commit failing;
2.  the two directions of finding 2,
    against `process/pid_namespace.rs:151-168` and `:263-270`;
3.  findings 3 and 4,
    against `process/process_start.rs:187`,
    `lock/index_lock_evidence.rs:280-300`
    and `lock/index_lock_holders.rs:144-200`,
    with the two probes under "Reproductions";
4.  finding 5,
    against `doc/decision/cli-git-concurrent-commits.md:166`;
5.  the inventory and "What I checked and found sound",
    which say what a redesign would have to keep.

Respond with a decision per finding,
and,
if finding 1 or 2 is to be fixed,
whether the fix is the suggestion on this page or another.
The audit changed no production code,
so no other action is pending.
