# Merge and repair route findings on `main`

Status:
audit,
2026-11-08.
Audited revision:
the code of this worktree at `554f97334339693eadc5b486889babf3e72760c0`;
the only files where it differs from `main` (`6a340105a3`) are two landing test sidecars
(`landing/service_claim_renewal_tests.rs` and `landing/service_land_tests.rs`),
which this route does not use.
No production code was changed and no test was committed.
The scratch scripts and their output are in `${HOME}/temp/agent/pi-merge-repair-audit`,
and each reproduction names its script.
Scope:
`package/git-policy/cli/src/native/{merge,conflict,repair,health,lock}/`,
the shared owner-record layer it uses
(`transaction/transaction_owner.rs`,
`process/pid_namespace.rs`,
`process/process_identity.rs`,
`invocation/startup_gate.rs`),
and the sections of `package/git-policy/cli/SPEC.md` that bind them.
Linux only,
which is what the owner's pause leaves in scope.

Purpose:
the audit of the copy route
(`doc/planning/cli-git-copy-route-findings-on-main.md`)
showed that a route can pass its tests and still lose state at the instants a process is killed,
and that no review had looked at the locks,
waits and owner records it rests on.
This page does the same for the merge and repair route:
every state change with what a kill on either side of it leaves,
every wait with its ceiling,
every lock with its reason,
the races between two commands on one repository,
and what is sound,
so nobody re-audits it.

## Decisions this route answers to

Owner words that shape the findings here,
quoted from `package/git-policy/cli/SPEC.md`:

- "Auto merging should never fail."
- "Manual merge should never fail too."
- "Commits should never fail."
- the acceptance bar (`SPEC.md:2203`):
  "nothing the machine or other processes do ends them with a nonzero status.
  That covers a held `index.lock`,
  a branch that moved underneath the command
  (the auto-push's own reconcile merge included),
  a slow disk,
  a stopped child and a deadline."
- the owner's answer for a live foreign `index.lock` (`SPEC.md:2187`):
  "Same as the ref lock (Recommended)":
  "a commit lands on the branch and exits 0,
  the index catches up when the lock is free,
  and the command says so;
  a merge saves its result on a side ref,
  exits 0,
  and finishes the checkout when the lock frees,
  carrying edits made meanwhile forward".
- "nothing should ever wait forever or indefinitely",
  quoted again in `lock/index_holder_wait.rs:217` and in `service/wait_ceilings.rs:13`.

Findings 2 and 3 are measured against the last three of those.
Finding 1 has no owner quote behind it:
this route recovers a killed merge without a word where the copy route says what it recovered,
which is the inconsistency it records.

## Every state change, and what a kill on either side of it leaves

The route is the one that changes a checkout in place,
so the interesting instants are the ones between a durable record and the write it protects.

1.  The merge commit object.
    `merge_isolation_apply.rs:commit_merge` runs `git commit-tree` before anything is applied.
    Killed before it ends:
    nothing is on disk.
    Killed after:
    one unreachable commit object,
    which `git gc` drops;
    nothing notices and nothing is lost.
2.  The merge intent record.
    `merge_isolation_journal.rs:write_merge_intent` publishes it whole
    (a staging file,
    a sync,
    an exclusive publish) and syncs the directory.
    Killed before:
    the state of item 1.
    Killed after:
    a record naming `oldOid` and `newOid` and no checkout change;
    the next command in that worktree settles it through `startup_gate.rs:211`
    and discards it,
    because neither the index nor the worktree holds anything of the merge.
    Nothing is lost and nothing is reported,
    which is right:
    the merge never happened.
3.  Git's own fast-forward to the merge commit.
    `merge_isolation_apply.rs:250` runs `git merge --ff-only <newOid>`,
    which writes the merge's worktree files,
    then the index entries,
    then the branch.
    Killed inside:
    worktree and index hold the merge,
    the branch does not,
    and the record from item 2 is there,
    so the next command completes it
    (`merge_isolation_recovery.rs:settle_failed_merge`,
    the `index_started` and `worktree_started` checks).
    That completion is silent;
    see finding 1.
4.  The record's removal.
    `merge_isolation_apply.rs:281` and `merge_isolation_recovery_steps.rs:remove_record`.
    Killed before it:
    the record outlives the merge,
    and the next command sees the branch already at `newOid` and removes it
    (`MergeRecoveryAction::AlreadyLanded`).
    Killed after:
    nothing is left.
5.  Recovery's own install,
    for a merge that started and lost its process.
    `merge_isolation_recovery_steps.rs:251` writes the index entries,
    `:267` writes the files,
    `merge_isolation_recovery.rs:297` moves the branch by compare and swap against `oldOid`.
    Killed between the files and the branch write:
    the same state as item 3,
    and the same completion,
    which is idempotent against the files it already wrote.
6.  The settling of a branch that moved.
    `merge_isolation_moved.rs:143` installs the moved head's entries for the merge's paths,
    then removes the record.
    Killed inside:
    the next command settles again from the record,
    and a path that holds neither the old,
    the merge's nor the moved head's content stops it with the record kept.
7.  The word-level second pass over a conflict a forwarded command left.
    `merge_second_pass_worktree.rs:201` stages the merged blobs,
    `:202` writes the files,
    `:208` puts the conflict stages back when the write fails.
    Killed between the two:
    the index says resolved and the file still holds markers,
    which `git status` shows,
    so the person sees it and the merge commit would carry the markers if they commit.
    `SPEC.md:2200` accepts exactly that outcome for a plain `git commit` over unmerged files.
8.  The merge message.
    `merge_second_pass_finish.rs:439` rewrites `MERGE_MSG` with `std::fs::write`,
    which truncates and then writes.
    Killed inside the write:
    a short or empty merge message beside a `MERGE_HEAD` the person has not committed yet.
    Nothing notices until they commit,
    and `git commit` then refuses an empty message.
9.  A saved copy of a conflicted file.
    `conflict_sides.rs:save_copies` creates the private directory,
    copies each file (each copy is `sync_all`ed,
    `conflict_side_source.rs:copy_stream`),
    and syncs every directory it wrote.
    Killed before the copies:
    nothing changed and the command never ran.
    Killed after them:
    an orphan save directory,
    which the prune removes under its own limits.
    The audit line that names the copies comes after the copies and before Git runs
    (`policy_checks.rs:374`),
    so a kill in that window leaves copies that no line names;
    the copy itself is what "never discard user data" is about,
    and it is on disk.
10. The prune of saved copies.
    `discarded_prune.rs:remove_entry` empties one directory and then writes its audit line.
    Killed between:
    the copies are gone and no line says so;
    the next pass finds the emptied directory and writes a `pruned` line with no files in it.
    What is lost is the attribution of a copy that was already removed by the pass that judged it removable.
11. The audit log's rotation.
    `discard_audit.rs:move_aside_between` renames the full file to a private name,
    links it under the next generation number,
    and removes the private name.
    Killed between the rename and the link:
    the history sits at a private name,
    and `discard_audit_generations.rs:settle_names` gives it the next number on the next rotation.
    Killed between the link and the removal:
    two names for one file,
    which the same pass removes.
12. A repair tool's work.
    `package_repair_command.rs:run_tool` starts the tool in its own process group with a quiet window and a ceiling.
    Killed while it runs:
    the tool keeps running,
    because nothing outside the wrapper signals that group;
    the worktree it writes is the new one,
    and the source is only read.
    Nothing notices,
    and the next add is unaffected.
13. A workspace's `Cargo.lock` around the clean.
    `package_repair_cargo.rs` reads the lock,
    runs `cargo clean --package`,
    and `package_repair_cargo_lock.rs:66` writes the bytes back with `std::fs::write`.
    Killed before the write:
    the new worktree's `Cargo.lock` holds what Cargo rewrote,
    which `git status` shows as a modification.
    Killed inside it:
    a truncated tracked file.
    Nothing says so;
    the checked-out bytes come back with `git restore -- Cargo.lock`.
14. A stale reference lock's removal.
    `stale_ref_lock_retire.rs:take_and_check` renames the judged lock to a private name in the Git directory,
    compares it with what was judged,
    and removes it or links it back.
    Killed between the rename and one of those:
    the private name stays for good,
    and the lock is off its path.
    The judgement that allowed it is the proof the design rests on;
    if it was wrong,
    the Git that held the lock fails loudly at its rename,
    which `SPEC.md:4268` states.
    No wrapper and no pass removes a private `cli-git-stale-lock-*` name.
15. An owner claim.
    `single_owner_claim.rs:publish_new` writes a private file,
    links it under the next generation number,
    and removes the private file;
    `release` renames a released record over the newest generation.
    Killed inside either:
    the claim stands (item is not taken by a second command),
    and at worst a `.cli-git-claim.tmp.*` file stays,
    which nothing removes.
16. `doctor`'s settling of kept names.
    `health/doctor.rs:491` calls `kept_names_in_the_git_directories::settle_common_directory_names`,
    which renames folders and folds earlier log lines into the current log.
    See finding 6.

## Findings, ranked by what they risk losing

### 1. A read-only command lands a merge with no notice

`invocation/startup_gate.rs:211` settles the worktree's merge records before any wrapped or read-only command,
and `:221` reports the outcomes with `debug` alone.
`merge/merge_isolation.rs:328` drops the outcomes entirely for the merge command's own recovery.

Sequence:

1.  A merge makes a commit,
    installs the checkout,
    and is killed before it moves the branch
    (item 3 of the state changes above).
2.  The person runs any command in that worktree,
    `git status` included.
3.  The startup gate finds the record of a dead owner,
    sees the index and worktree holding the merge,
    installs the entries,
    and moves the branch by compare and swap
    (`merge_isolation_recovery.rs:297`).
4.  The command prints only what it was asked to print.

Wrong result:
the branch gains a merge commit that the person never saw land,
from a command they used to look at the worktree.
Nothing on standard output or standard error says so,
and the only record is a debug line.
The copy route reports its own recovery
(`worktree_copying/worktree_copy_recovery.rs:310`,
"recovered ignored-state copies for N worktree transactions");
this route does not.

Reproduced,
quoted in "Reproductions",
item R1:
a `git status --porcelain=v1` exited 0 with empty standard error and moved `main` from `dcd32667` to `74072837`,
a two-parent merge commit.

Suggestion,
not made:
print one line per outcome that changed something
(`Completed` and `BranchMoved`),
naming the ref and the commit,
in the shape the copy route uses.

### 2. Recovery stops instead of waiting for another program's index lock

`merge_isolation_recovery_steps.rs:251` writes the index with `git update-index --index-info`,
and `:267` writes the files with `git checkout-index --force`.
Neither waits for `index.lock`;
`git update-index` takes it,
and fails when another program holds it.
The merge's own apply does wait (`merge_isolation_apply.rs:223`),
so the wait exists in the route and
is missing exactly where a killed merge is finished.

Sequence:

1.  A merge is killed after Git installed the checkout and before the branch moved.
2.  Anything writes the index in that worktree before the next command:
    the commit service landing a commit,
    the person's own `git commit` with the editor open,
    an IDE,
    another `git add`.
3.  Any command runs,
    `git status` included.
4.  The startup gate calls recovery,
    recovery calls `git update-index`,
    Git refuses for the existing lock.
5.  `merge_isolation_recovery.rs:295` propagates the failure,
    `startup_gate.rs:297` turns it into `Gate::Stop`,
    and the command exits 2 as a `content-unavailable` engine failure.
6.  The record is kept,
    so every later command repeats this until the lock is free.

Wrong result:
every command in the worktree exits 2 while another program holds the index,
with Git's own `index.lock` sentence in place of a statement of what is unfinished.
The copy route's recovery skips an owner that still runs and lets the command go on;
this route makes the command fail.

Reproduced,
quoted in "Reproductions",
item R2:
recovery returned
`git update-index -z --index-info failed: fatal: Unable to create '.../index.lock': File exists.`,
and the same call after removing the lock returned `Completed`.

Suggestion,
not made:
wait for the index lock before `install_entries`,
bounded and naming the holder,
the way `merge_isolation_apply.rs:223` does,
and let the command go on when the wait ends;
the recovery is idempotent,
so leaving the record for the next command is also safe.

### 3. A foreign `index.lock` ends a merge with a non-zero status

Three pieces make this one finding.

- `lock/index_holder_wait.rs:165` waits at most `MERGE_LOCK_CEILING`,
  one minute (`service/wait_ceilings.rs:310`).
- The merge's own apply ignores the wait's result:
  `merge_isolation_apply.rs:223` calls it as a statement.
- When Git then refuses for the lock,
  `merge_isolation_apply.rs:261` reads "index.lock" in Git's words and returns `Bail::Again`,
  which `merge_isolation.rs:108` turns into another whole round,
  up to `MAX_ROUNDS` 20 (`merge_isolation_run_types.rs:114`),
  after which `merge_isolation.rs:117` refuses with exit 2.

Sequence,
the proven-dead holder:

1.  A Git killed under `core.lockfilePid` leaves `index.lock` and a pid file naming a process that has ended.
2.  The person runs `git merge` of a branch that makes a commit.
3.  Each round waits nothing (`wait_for_index_release` returns `OwnerGone` at once,
    `index_holder_wait.rs:210`) and runs `git merge --ff-only`,
    which fails on the lock,
    and the round repeats.
4.  After 20 rounds the merge exits 2 saying
    "the branch moved or the index lock was taken in 20 merges in a row,
    so the wrapper stopped;
    nothing was changed.
    Run the merge again."

Wrong result:
the merge fails,
the message names a cause that did not happen (no other program took the lock in those 20 rounds;
a dead Git left it),
and its advice,
"Run the merge again",
cannot work until a person deletes the file.
The commit service removes exactly such a lock and lands
(`SPEC.md:3170`,
"Service removes it"),
so a commit repairs what a merge refuses.

Sequence,
a live or unproven holder:

1.  Another program holds `index.lock` and is not the commit service,
    for instance the person's own `git commit` with its editor open.
2.  A `git merge` that fast-forwards is forwarded to real Git
    (`lock/index_writer_forward.rs:237`) after the same bounded wait.
3.  At the ceiling the wait names the blocker and ends,
    and Git refuses for the lock.

Wrong result:
the same `Unable to create ... index.lock` failure native Git gives in 4 ms,
delivered after 60 seconds and eight waiting lines.
Measured:
native Git exits 1 in 4 ms;
the wrapper exits 1 after 60 seconds (item R4).

The owner decided this case should not fail at all:
`SPEC.md:2187` has a merge save its result on a side ref,
exit 0 and finish the checkout when the lock frees,
and `SPEC.md:2203` puts a held `index.lock` outside anything that may end a merge with a non-zero status.
No side-ref save for a merge exists in this route;
`rg 'side ref|refs/cli-git' src/native/{merge,conflict,repair,lock}` finds only word-merge corpus fixtures.
`SPEC.md:3129` states the outcome the code has,
and contradicts those two passages.

Reproduced,
quoted in "Reproductions",
items R3 and R4.

Suggestion,
not made:
one decision is owed here,
not a patch.
The two passages must be reconciled,
and if the owner's answer stands,
the merge needs the side-ref save and the deferred finish that the commit route already has.

### 4. Recovery writes a worktree file whose content it checked moments earlier

`merge_isolation_recovery_steps.rs:standing` decides a worktree path is old,
new or someone else's,
and `install_entries` writes the new content with `git checkout-index --force` after it.
The check and the write are separate steps with `git diff-tree` and `git update-index` between them
(`merge_isolation_recovery.rs:240` to `:297`),
one `git hash-object` per changed path in `worktree_state`.

Sequence:

1.  A killed merge leaves a record and a checkout that holds the merge.
2.  A command starts,
    and recovery judges each changed path to be the merge's content.
3.  The person's editor saves one of those files between that judgement and `checkout-index --force`,
    or another process writes it.
4.  `checkout-index --force` replaces it with the merged blob.

Wrong result:
the edit is gone with no notice.
The design intends the opposite
(`merge_isolation_recovery.rs:1`:
"A path that holds neither its old nor its new content belongs to someone else,
so recovery stops"),
and the check exists;
what is missing is that the check and the write are one step.

Not reproduced:
the window is between two Git starts and closing it needs timing,
which this audit does not use.
Read from the code only.

Suggestion,
not made:
write each file by creating it exclusively or by replacing it only when its identity is still the one judged,
which is the step `stale_ref_lock_retire.rs:take_and_check` already uses for a lock.

### 5. The decision record still promises an unbounded lock wait

`doc/decision/cli-git-concurrent-commits.md:158` to `:161` says an `index.lock` owner proven alive
"gets an unbounded wait with one line naming the holder",
`:186` records that the code was changed to do so,
and `:194` repeats it for an owner nothing proves.

The code does the opposite:
every wait in this route has a ceiling,
the merge's is `MERGE_LOCK_CEILING` (`wait_ceilings.rs:310`),
the forwarded writer's is `SERVICE_HOLD_CEILING` (`wait_ceilings.rs:337`),
and the owner's later requirement
("nothing should ever wait forever or indefinitely",
`service/wait_ceilings.rs:13`)
is what the ceilings were built for.

Wrong result:
a reader who trusts the decision record restores an unbounded wait and
reintroduces exactly the stall the ceilings removed;
the two documents cannot both be followed.

Not reproduced:
this is a contradiction between documents and code,
not a runtime state.

Suggestion,
not made:
amend the decision record to name the ceilings,
keeping the requirement that a held lock is waited for while it is held,
and note that the wait ends by naming the blocker.

### 6. `doctor` writes to the repository it reports on

`SPEC.md:527` says of `doctor`:
"It does not load configuration,
recover transactions or take a lock,
and it changes nothing."
`health/doctor.rs:491` calls `settle_common_directory_names`,
which renames the route's kept folders and files in the common Git directory
and folds earlier log lines into the current log
(`kept_names_in_the_git_directories.rs:settle_common_directory_names`).
`doctor.rs:488` also reads configuration through `config_findings`.

Sequence:

1.  A repository still holds names an earlier release wrote,
    such as an audit log or a data folder under the earlier spelling.
2.  Someone runs `git monochromatic-git-policy-cli doctor`,
    to look,
    on a repository they may not own or do not mean to change.
3.  Doctor renames those names and appends to the audit log.

Wrong result:
the one command documented as changing nothing changes the repository,
and a person who ran it to diagnose a read-only problem has written to a Git directory.
Nothing is discarded;
the rename never replaces and the fold keeps the other spelling when both exist.

Read from the code only;
no reproduction was attempted because the state needs a repository written by an earlier release.

Suggestion,
not made:
either state in `SPEC.md` that doctor settles the route's kept names and reads configuration,
or move the settling to the command that owns the directory.

## Waits and their ceilings

Every wait in this route is bounded.
None waits for a lock without a ceiling,
and none sleeps in a loop without one.

- The merge's wait for a foreign `index.lock`:
  `MERGE_LOCK_CEILING`,
  one minute
  (`lock/index_holder_wait.rs:165`,
  `service/wait_ceilings.rs:310`).
  A dead pid-file owner ends it at once with a diagnostic (`index_holder_wait.rs:210`).
  The whole command can spend `MAX_ROUNDS` times that,
  20 minutes (`merge_isolation_run_types.rs:114`),
  which nothing states.
- The forwarded index writer's wait for the commit service's hold:
  `SERVICE_HOLD_CEILING`,
  ten minutes (`wait_ceilings.rs:337`),
  and its whole series of reruns is bounded by the same number
  with at most `ATTEMPTS` 50 starts (`landing/service_hold_retry.rs:40`).
  The same wait and ceiling apply to a pull's own merge (`merge/pull_isolation.rs:496`).
- The repair tools:
  a quiet window of 600 seconds and `REPAIR_TOOL_CEILING`,
  18 minutes,
  per tool start (`repair/package_repair_request.rs:53`,
  `wait_ceilings.rs:454`).
  The Cargo repair runs up to one metadata read,
  one clone and one clean per workspace,
  each bounded on its own.
- A step the wrapper never ends:
  the branch write is left running after its class ceiling plus
  `SHORT_EXTRA_WAIT` 7 seconds,
  and the command says the result is not known
  (`wait_ceilings.rs:412`,
  `transaction/transaction_git_step_bounds.rs`).
  The command itself does not wait longer.
- The route's own conflicts,
  claims and audit steps wait for nothing:
  `single_owner_claim.rs:take_claim` states that it never waits,
  the prune passes never wait (`discarded_prune.rs:1`),
  and `doctor` takes no lock.

What has no ceiling at all is a **tool** the wrapper left running:
`run_tool` stops the group and reaps the leader within `STOP_GRACE`,
then hands a leader it could not reap to a background task
(`package_repair_command.rs:reap_within`).
Nothing else in this route starts something it does not wait for.

## Locks

The wrapper holds no lock of its own in this route.
`source_guard/service_lock_inventory_tests.rs` fails a lock primitive in these sources,
and `rg 'flock|Mutex|RwLock|Condvar' src/native/{merge,conflict,repair,health,lock}`
finds only test fixtures and the PowerShell Restart Manager script.

What the route does use:

- Git's own `index.lock`,
  taken by the Git children the route starts
  (`git merge`,
  `git update-index`,
  `git checkout-index`,
  `git read-tree`).
  The route never creates or removes it.
  `lock/index_writer_forward.rs` waits for the commit service's hold on it,
  and `lock/index_holder_wait.rs` waits for any other holder,
  bounded.
- A claim file per item,
  published with a hard link under the next generation number
  (`lock/single_owner_claim.rs`).
  One command owns an item,
  no command waits,
  and the newest generation is never removed.
- The reference-lock judgement's conditional removal:
  one `renameat2` with `RENAME_NOREPLACE` takes the file off its name,
  the taken file is compared with what was judged,
  and it is put back with a hard link when it differs
  (`lock/stale_ref_lock_retire.rs:take_and_check`,
  `landing/service_index_placement.rs:rename_noreplace`).
- Exclusive creation for every record and every saved copy
  (`foundation/private_storage.rs:write_then_publish`,
  `conflict/conflict_side_source.rs:copy_entry`).

A lock that could be removed:
none.
The one candidate the design already considered and rejected is an `flock` beside the stale `index.lock`
(`SPEC.md:5689`,
"the earlier `flock` on the stale lock's own file is retired"),
and it is gone.

## Races between two commands on one repository

- Two commands settling one killed merge.
  Both read the same record,
  both judge the owner dead,
  both call `settle_failed_merge`.
  One wins the branch write (`update-ref` with the old value),
  the other's `update-ref` fails and its command exits 2;
  nothing is written twice,
  and the loser's `install_entries` wrote the same content the winner did.
  The record's removal is idempotent (`remove_merge_intent` ignores a missing file).
- A command settling a killed merge while the commit service lands a commit in the same worktree.
  The service holds `index.lock` for the branch move and the index build,
  so recovery cannot take it and exits 2 (finding 2).
- Two merges in one worktree.
  Both write records and both run `git merge --ff-only`;
  Git serializes them on `index.lock`,
  the loser sees a branch that is neither its old nor its new commit,
  settles its own paths back (`merge_isolation_moved.rs`) and takes the next round.
  The record of each is its own file,
  so neither recovers the other.
- A merge in one worktree and a command in another worktree of the same repository.
  The merge records live in the worktree's own Git directory
  (`merge_isolation_journal.rs:intent_directory`),
  so a command in another worktree never settles them and never touches that checkout.
  Saved copies and the audit log live in the common Git directory,
  and their writes are exclusive creates and single `write` calls,
  so two worktrees cannot interleave a line
  (`conflict/discard_audit.rs:write_once`).
- Two prune passes over the saved copies.
  No lock.
  The file belongs to the pass whose `unlinkat` succeeded,
  and its audit line is that pass's,
  so each copy is counted once (`conflict/discarded_store.rs:1`).
- Two `git checkout --ours` commands during one merge.
  Both read the index in process and both save a copy;
  the second reads an index that still shows the path conflicted only if the first has not run yet,
  and a copy is never written over another (the directory name is claimed by exclusive creation,
  `conflict_sides.rs:fresh_directory`).
- Two claim attempts.
  The kernel lets one `hard_link` create generation `n + 1`;
  the loser re-reads and reports the live owner (`lock/single_owner_claim.rs:take_claim`).
- A repair and a commit in the new worktree.
  The repair runs after the copy and before the command ends,
  and it writes only the new worktree;
  another command in the parent checkout is unaffected.
  A commit made in the new worktree while its repair runs is not defended against,
  which `SPEC.md:2067` states as the person's risk.

## Reproductions

Everything below ran against the debug binary built from this worktree
(`mise run //package/git-policy/cli:native:build`),
or against the crate's own test target,
in disposable directories under
`${HOME}/temp/agent/pi-merge-repair-audit`.
No repository of this host's was touched,
and no test was committed.

### R1. A `git status` lands a killed merge with no notice

Script:
`silent-recovery.ts`.
It builds a repository whose `main` and `feature` have diverged,
writes the intent record a killed merge leaves after Git moved the checkout,
and runs the wrapper as `git status --porcelain=v1`.

```text
$ node --experimental-strip-types silent-recovery.ts
{
 "command": "git status --porcelain=v1",
 "exit": 0,
 "stderr": "",
 "stdout": "",
 "branchMoved": true,
 "headBefore": "dcd32667",
 "headAfter": "74072837",
 "headAfterIsTheMerge": true,
 "recordLeft": false,
 "logSubject": "merge",
 "logParents": 2
}
```

The read-only command moved `main` onto the merge commit,
printed nothing at all,
and removed the record.
The finding is reproduced.

### R2. Recovery fails when another program holds the index

A test in `merge_isolation_recovery_tests.rs` (removed again,
as the brief asks),
taken from the sidecar's own `world()` fixture:
the merge had begun,
the owner was dead,
and `.git/index.lock` existed.

```text
AUDIT recovery error: git update-index -z --index-info failed: fatal: Unable to create
 '.../repo/.git/index.lock': File exists.

Another git process seems to be running in this repository, or the lock file may be stale
AUDIT second attempt: [MergeRecoveryOutcome { record: ".../aaaaaaaa-...-000000000001.json",
 action: Completed }]
```

The first call returned an error and kept the record;
the same call after the lock was removed completed the merge.
The finding is reproduced.

### R3. A dead Git's index lock ends a merge

A test in `merge_isolation_repository_tests.rs` (removed again),
on the fixture whose merge the same file's own test completes with status 0 when nothing holds the index:
a dead pid file beside an empty `index.lock`.

```text
AUDIT merge outcome: Finished { code: 2, stdout: "", stderr:
 "monochromatic-git-policy-cli: git merge: the branch moved or the index lock was taken in 20 merges in a row,
 so the wrapper stopped; nothing was changed. Run the merge again.\n" }
```

The same state seen end to end,
through the binary,
with the wrapper's own lines (`repro.ts dead-lock`,
8 seconds in all):

```text
$ node --experimental-strip-types repro.ts dead-lock
{"mode":"dead-lock","exit":2,"headMoved":false,"seconds":8,"stderrLines":42,
 "stderrFirstThree":[
  "monochromatic-git-policy-cli: .../.git/index.lock was left by PID 4294967, which no longer runs,
   so nothing will release it. git merge stops waiting and reports what Git says;
   nothing was changed and the lock was left in place. ...",
  "monochromatic-git-policy-cli: git merge: another program took the index lock first;
   merging again on the current branch.",
  "monochromatic-git-policy-cli: .../.git/index.lock was left by PID 4294967, ..."],
 "stderrLastTwo":[
  "monochromatic-git-policy-cli: git merge: another program took the index lock first;
   merging again on the current branch.",
  "monochromatic-git-policy-cli: git merge: the branch moved or the index lock was taken in 20 merges in a row,
   so the wrapper stopped; nothing was changed. Run the merge again."],
 "lockStillThere":true}
```

The lock was proven dead on every round,
the branch did not move,
and the merge still exited 2
with advice that cannot work.
The finding is reproduced.

### R4. A held index lock turns a 4 ms refusal into a 60 second one

Script:
`repro.ts ff-live-lock` for the wrapper,
`native-lock.ts` for native Git on the same fixture and the same lock.

```text
$ node --experimental-strip-types native-lock.ts
{"tool":"native git","exit":1,"millis":4,
 "stderr":["error: Unable to create '.../.git/index.lock': File exists.",
  "Another git process seems to be running in this repository, or the lock file may be stale"]}

$ node --experimental-strip-types repro.ts ff-live-lock
{"mode":"ff-live-lock","exit":1,"seconds":60,"headMoved":false,
 "stderr":["monochromatic-git-policy-cli: git merge has changed nothing yet and waits for the index lock of
   this repository; waiting for .../.git/index.lock, whose owner cannot be proven alive ...;
   this waits for as long as the lock stays there. ...",
  ... eight such lines, then ...,
  "monochromatic-git-policy-cli: .../.git/index.lock is still held after 60 seconds by an owner nothing proves
   alive ..., so git merge stops waiting and reports what Git says; nothing was changed and the lock was left
   in place. Run the command again once that program is done.",
  "error: Unable to create '.../.git/index.lock': File exists.",
  "Another git process seems to be running in this repository, or the lock file may be stale"]}
```

The same two closing lines native Git prints at once,
and the same exit status,
60 seconds later.
The finding is reproduced.

## What I checked and found sound

Each of these was read,
and where a sequence could have gone wrong it was followed through the code.
None needs a second audit.

- The intent record cannot be half written:
  `write_then_publish` writes a staging file,
  syncs it,
  and publishes it exclusively,
  so a kill leaves no record or a whole one
  (`foundation/private_storage.rs:377`).
  A malformed record is therefore only reachable from outside,
  and recovery refuses it by name and keeps it
  (`merge_isolation_recovery.rs:121`).
- Recovery never acts on a record whose owner may be running:
  `merge_isolation_recovery.rs:105` reads every record and judges every owner before it settles any,
  and a probe the account may not make counts as alive (`:155`).
  A record of another PID namespace past the patience stops the command with the record kept,
  by the owner's decision of 2026-11-04 ("Refuse and name it").
- The branch move in recovery is a compare and swap against the recorded old commit
  (`merge_isolation_recovery.rs:297`),
  so a branch that moved meanwhile is never overwritten.
  A moved branch is settled back to the moved head's own entries,
  including a path the moved head added or deleted
  (`merge_isolation_moved.rs:143` installs the end state,
  not the merge's).
- Recovery refuses to write a path that holds none of the three contents it knows,
  and names the paths and the record when it does (`merge_isolation_recovery.rs:258`).
- A merge killed after Git began is completed from the record;
  one killed before anything began is discarded with the checkout unchanged.
  Both were exercised by the sidecar's existing controls,
  and my R2 run completed a begun merge.
- A merge is never attempted over unsaved work that overlaps it
  (`merge_isolation.rs:262`),
  and a merge over unrelated unsaved work is completed without touching it.
- A saved copy of a conflicted file is durable before the command that replaces it runs:
  each copy is `sync_all`ed,
  every written directory is synced,
  and only then is the command allowed
  (`conflict_side_source.rs:copy_stream`,
  `conflict_sides.rs:save_copies`).
  A copy that cannot be made rejects the command instead of discarding the file
  (`conflict_side_guard.rs`,
  `SideVerdict::CopyFailed`).
  A copy that exists goes into a directory nothing else can name
  (`fresh_directory`) and is never overwritten.
- The prune cannot remove the copy just written,
  nor one younger than a day
  (`discarded_prune_plan.rs:is_reachable`),
  and the audit generation that names a copy still on disk is never removed
  (`discard_audit_generations.rs:names_a_copy_on_disk`).
- The audit file's rotation is lock-free and loses no line to a kill between its steps;
  `settle_names` is on `main` and gives a file a killed rotation left at its private name the next generation number.
- The saved-copy store never follows a link and addresses every removal through a descriptor,
  so a replaced path cannot redirect a removal (`conflict/discarded_store.rs:1`).
- The reference-lock judgement removes a lock only on a proof about its holder,
  never on a time stamp,
  never on an absent descriptor,
  and never on a file system that another machine may share
  (`lock/stale_ref_lock_judge.rs:examine`).
  The removal is one conditional rename with the identity checked before and after it.
- Every repair tool is bounded twice,
  by a quiet window and by a ceiling,
  is started in its own process group,
  and is stopped with the whole group
  (`repair/package_repair_command.rs`).
  A tool that works is never ended for being slow,
  which is the owner's decision of 2026-11-04.
- `doctor` takes no lock,
  recovers no transaction and changes no configuration file;
  what it does change is finding 6.

## What I could not determine

- Whether the side-ref save the owner chose for a merge that meets a live `index.lock`
  exists anywhere outside this route;
  `rg 'saves its result on a side ref'` finds the decision and no implementation.
  If it exists in the commit route under another name,
  finding 3's suggestion changes shape.
- What a kill inside `doctor`'s log fold leaves.
  The fold and its crash steps are `earlier_names_folded_into_current.rs` and
  `earlier_log_lines_added_to_current_log.rs`,
  which this audit did not read.
- Whether a repository on a file system shared by two hosts is supported.
  If it is,
  every owner record in this route reads a foreign host's process number as a dead one,
  because the record carries only a PID and a namespace name that is not unique across machines
  (`process/pid_namespace.rs:foreign_owner_standing`,
  `transaction/transaction_owner.rs:216`).
  The copy route's audit reached the same conclusion for its records
  (`doc/planning/cli-git-copy-route-findings-on-main.md`,
  its verdict on `review-wtc` finding 2),
  so this is one owner question,
  not two findings.
- How often a merge actually meets a foreign `index.lock` here.
  No log holds a merge's wait length:
  `MERGE_LOCK_CEILING`'s comment (`service/wait_ceilings.rs:294`) records
  that the commit timing log held no merge record in the window it was read.
  So finding 3's cost is unmeasured,
  and only its wrong result is established.
- Whether `MAX_ROUNDS` rounds of a one minute wait were ever reached.
  Nothing logs a round count.
