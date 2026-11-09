# Subvolume and conversion-route findings on `main`

Status:
audit,
2026-11-09T01:07:14-05:00.
Audited revision:
`main` at `9d4e5a9970`,
in the worktree
`/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/pi-subvolume-and-conversion-route-findings-on-main`,
on branch `docs/cli-git-subvolume-and-conversion-route-findings`.
No production code was changed and no test was committed.
The scratch controls are in
`package/git-policy/cli/src/native/subvolume/audit_scratch_tests.rs`,
registered in that module for the runs only,
and their fixtures are under
`${HOME}/temp/agent/pi-subvolume-audit-fixtures`.
Linux only:
the owner's pause covers Windows and macOS.

Scope:

- the btrfs subvolume layer,
  `package/git-policy/cli/src/native/subvolume/`
  (62 files,
  14,854 lines,
  test sidecars included),
  and its section of `package/git-policy/cli/SPEC.md`;
- the command that converts an existing linked worktree into a subvolume,
  `package/git-policy/cli/src/native/worktree/worktree_convert*.rs`
  (16 files,
  4,614 lines),
  reached through
  `git monochromatic-git-policy-cli convert-worktree-to-subvolume`;
- the command that converts the main checkout,
  `git monochromatic-git-policy-cli migrate-to-subvolume`
  and its `--rollback` and `--remove-old` modes,
  because it is the same layer and the same person's uncommitted work.

Out of scope:
`git worktree add` and the btrfs ladder it decides with (the add path,
which another audit covers),
and everything the earlier pages below already
record.

Every `path:line` on this page is relative to
`package/git-policy/cli/src/native/`.

## What the earlier pages already record

`doc/planning/cli-git-worktree-removal-and-conversion-route-findings-on-main.md`
(2026-11-08T22:25:01-05:00,
`main` at `20acf6f904`)
audits this route's sibling subject,
what removes,
renames or replaces whole directories.
No file under `src/native/subvolume/` or `src/native/worktree/` changed between
that revision and the one audited here
(`git diff --stat 20acf6f904..HEAD -- package/git-policy/cli/src/native/subvolume
package/git-policy/cli/src/native/worktree` is empty,
and the only `SPEC.md` change is a reftable passage far from this route),
so every `path:line` on that page still holds.
Four of its seven findings touch this page's route and are not repeated here:

- its finding 1,
  a file the carry leaves out before `--remove-old` deletes the old tree;
- its finding 3,
  a conversion interrupted past its swap whose old tree no command can clean
  once the worktree is removed;
- its finding 6,
  `--remove-old` never re-verifying which directory the checkout is;
- its finding 7,
  `--abandon` destroying a stage a process may be working in;
- its kill-point walk,
  its locks section and its waits section,
  all of which this page extends rather than restates.

`doc/planning/cli-git-copy-route-findings-on-main.md`
is the copy route and shares no module with this one.

`doc/handover/cli-git-rust-implementation.md`,
"Open review findings",
records nothing about this route:
its open items are the copy route's findings,
the push route's findings,
the audit-rotation generations,
one accepted downgrade limit and the four Critical findings of the
commits-never-fail review.

## The owner's rules this page judges by

Quoted verbatim from the brief:

- "never discard user data".
- "nothing should ever wait forever or indefinitely".
- "Use as few locks as possible."
- `AGENTS.md`,
  `NHP`:
  what is normal on this machine includes
  "a process killed at any instruction,
  a closed terminal or broken pipe,
  slow fsync,
  unusual filesystems,
  DNS or mirror trouble."

From `doc/decision/`:

- `doc/decision/cli-git-concurrent-commits.md:166`:
  "cli-git never deletes a lock."
  Both holders here delete only a lock whose bytes are their own token
  (`subvolume_lock.rs:128`,
  `is_ours`;
  `worktree_convert_phases.rs:187`,
  `release_index_lock`),
  so the decision holds.
  The same record,
  at `:158-160`,
  gives a proven-alive `index.lock` holder an unbounded wait inside a forwarded
  Git command.
  This route refuses instead,
  naming the file
  (`subvolume_lock.rs:146-160`),
  which is what `SPEC.md` states for it;
  the two are different commands,
  and nothing in the route waits on a lock.
- `doc/decision/cli-git-rust-rewrite.md`:
  "Preserve the existing required policy,
  transaction,
  concurrency,
  recovery,
  worktree,
  and push behavior."
  Nothing in this route was carried over from the TypeScript cli-git,
  so it has no parity question to answer;
  the rules above bind it.

## The route as it runs

The earlier page describes both commands step by step
("The route as it runs on this host");
what matters for the findings below is the shape of each one.

A conversion,
`convert_worktree` (`worktree_convert.rs:147`):

1. `inspect` reads the worktree's `.git` file,
   its administrative directory and
   the back-pointer (`worktree_convert_inspection.rs:83`).
2. A numbered claim file gives one command the conversion
   (`single_owner_claim.rs:348`).
3. The journal is created exclusively in the administrative directory
   (`worktree_convert.rs:255`).
4. `copy_phase` (`worktree_convert_phases.rs:215`)
   creates `<path>.cli-git-converting` as a subvolume,
   records its identity,
   copies with cloned data,
   takes Git's `index.lock` with its token (`:227`),
   and copies a second time with deletions to prove the copy equals the source
   (`:229`).
5. `swap_phase` exchanges the two directories (`:311`).
6. `verify_swap` checks the installed identity and the `.git` bytes (`:353`).
7. `index_phase` rewrites the index stat fields under the held lock and
   releases it (`:375`).
8. `removal_phase` reads the process table,
   compares the old tree with the installed one
   (`worktree_convert_remove.rs:99`),
   and deletes the old tree only when nothing differs (`:104-120`).
9. `finish` drops the journal (`:453`).

A migration,
`migrate` (`subvolume_migrate.rs:349`):

1. A claim file gives one command the checkout
   (`subvolume_command.rs:171`).
2. `Run::open` loads the journal and the unreadable-file list
   (`subvolume_run.rs:73`).
3. Preflight refuses a non-btrfs root,
   a mount point,
   too little free space,
   an unfinished Git operation,
   a live conversion,
   and leftovers no journal records
   (`subvolume_preflight.rs:228`).
4. `<root>.subvol-new` is created and filled by two passes plus a delta,
   the last of them under `index.lock` of the checkout and of every registered
   worktree (`subvolume_migrate.rs:189-233`).
5. `RENAME_EXCHANGE` swaps the two directories,
   the original tree is renamed `<root>.before-subvolume`,
   newer files are carried back,
   the indexes are rewritten,
   the locks are released
   (`:275-298`).
6. The original tree is kept until `--remove-old`
   (`subvolume_rollback.rs:299`),
   and `--rollback` swaps back and destroys the new subvolume
   (`:210-252`).

## Findings ranked by what they risk losing

### 1. A permission, owner or extended-attribute change made in a conversion's last window is discarded, and the copy that held it is deleted

Reproduced at the compare-and-delete step;
the sequence that puts a person's change into that window is read from the
code.

`first_difference` (`worktree_convert_compare.rs:40`)
decides whether the old tree is safe to delete.
For a regular file it compares the type,
the size and the modification time to
the nanosecond (`:66`);
for a symbolic link the target (`:60`);
for every entry the device (`:48`).
It does not compare the permission bits,
the owner,
the group or the extended attributes,
and nothing else does before the deletion
(`worktree_convert_remove.rs:99-120`).

Sequence:

1. `convert-worktree-to-subvolume W` copies W,
   takes `index.lock`,
   and its proving pass finds nothing left to copy
   (`worktree_convert_phases.rs:229-234`),
   so the journal records `synced` (`:245`).
   The copy of the tree is now the only copy that will survive.
2. Before the removal phase,
   a person or a program changes a file in W:
   `chmod +x` on a script,
   `chgrp`,
   `setfacl` to withdraw a colleague's access,
   or a `user.*` attribute.
3. `swap_phase` exchanges the directories (`:319`),
   so the copy is at W and the source,
   with the change,
   is at
   `W.cli-git-converting`.
4. `removal_phase` (`:421`)
   compares and gets `Ok(None)`,
   because size and modification time did not change,
   and deletes the old tree (`worktree_convert_remove.rs:104-120`).
5. `finish` removes the journal (`worktree_convert_phases.rs:453`),
   and the command exits 0.

Wrong result:
the change is gone,
the one tree that held it is deleted,
and nothing names it.
For a mode change on a tracked file the silence is complete:
the index rewrite replaces exactly six fields,
none of them the entry's mode
(`subvolume_index.rs:554`),
so the index still records the mode the file had before the change,
the installed file has that same mode,
and `git status` reports the worktree clean.
A person who made that change and then ran the command sees a clean worktree
and has no way to learn that their change was thrown away.

The window is not long for one worktree,
but it is not a single instruction either:
it spans the `index.lock` release,
the stat rewrite of the whole index
(`worktree_convert_phases.rs:375`),
the process-table scan and the comparison walk itself.
This audit's own twelve-entry conversion finished in under a second
(the runs under "Reproductions"),
and the largest index this repository has is 2.8 MB
(`subvolume_index_refresh_of_every_worktree.rs:21-40`),
whose rewrite is the part that follows the swap.

Suggestion,
not a decision:
add the permission bits,
the owner,
the group and the carried extended
attributes to the comparison.
`carried_attributes_equal` (`subvolume_sync_attrs.rs`)
already answers the last one,
and `file_is_equal`
(`subvolume_sync_pass_state_and_checks.rs:191`)
already compares the mode for the copy,
so the refusal shape the command has (keep the old tree,
name the entry) works
unchanged.
The alternative is to copy the difference into the installed tree rather than
keep the old one,
which would make the removal phase a merge;
that is more machinery than the refusal needs.

### 2. The conversion reports none of what its copy could not carry, and the old tree goes anyway

Read from the code;
the silent-skip half is reproduced.

`copy_phase` prints one line about the copy it made
(`worktree_convert_phases.rs:224`),
and `index_phase` one about the index (`:383`).
`SyncStats` also counts the attributes the copy could not set
(`subvolume_sync_pass_state_and_checks.rs:108-116`,
`losses`),
the sockets and device nodes it did not recreate (`:108-116`,
`special_skipped`),
and the entries it kept in the destination (`conflicts`).
The migration prints all three for every pass
(`subvolume_run.rs:196-212`);
a conversion prints none of them,
in either pass.

Sequence:

1. A worktree holds a file owned by another user,
   for example one a root shell or a colleague's process created,
   or one carrying a POSIX ACL.
   It is readable,
   so the refusal that exists for unreadable files does not
   fire (`worktree_convert_phases.rs:166`,
   `require_complete`).
2. The copy sets the owner and the ACLs and the kernel refuses them
   (`subvolume_sync_attrs.rs:314`,
   `settle`,
   which counts `EPERM` and `EACCES` instead of failing).
3. `copy_phase` prints only the entry count,
   so the counts of what was lost are never said.
4. `swap_phase`,
   `index_phase` and `removal_phase` run as usual;
   the comparison does not look at the owner (`worktree_convert_compare.rs:66`),
   so the old tree is deleted
   (`worktree_convert_remove.rs:120`).
5. The worktree is now a subvolume whose files have the caller's owner and
   none of the ACLs,
   and nothing has said so.

Wrong result:
an ownership or ACL change the person never asked for,
applied silently,
with the only tree that had the original attributes deleted.
The copy path's own message names the shape:
"not running as the owner or as root"
(`subvolume_run.rs:201`).

Reproduced for the skipped-special-file half:
the socket run below printed exactly two lines about its copy,
"copied 12 entries (7 files,
0 cloned)"
and "index:
6 of 7 entries refreshed",
and no line about the socket it left out of the stage.
The attribute-loss half is read from the code:
provoking it needs a file owned by another user,
which needs root.

Suggestion,
not a decision:
print the three counts in `copy_phase` with the same wording the migration uses,
which is one call to a shared line builder,
and let `require_complete` refuse a stage with a non-empty `losses` before the
swap,
where `--abandon` is still available.

### 3. A Unix socket or device node in a worktree makes a conversion unable to finish, and every rerun repeats the failure

Reproduced;
the block under "Reproductions" quotes both rounds and the abandon.

Sequence:

1. A registered linked worktree W holds a Unix socket,
   for example a development server's or an editor's.
2. `convert-worktree-to-subvolume W` passes its refusals:
   the socket is untracked and holds no process,
   so neither the cleanliness proof nor the process scan objects
   (`worktree_convert_inspection.rs:263-306`).
3. The copy skips the socket
   (`subvolume_sync.rs:182`,
   `special_skipped`),
   so the proving pass finds nothing to change
   (`worktree_convert_phases.rs:160`,
   `pass_changed`),
   and the swap happens.
4. `removal_phase` compares,
   finds the socket missing from the installed tree
   (`worktree_convert_compare.rs:54`),
   and refuses with `Differs`
   (`worktree_convert_remove.rs:101`):
   the old tree is kept,
   the journal stays,
   the command exits 2.
5. Every later run reads a journal at or past `exchanged`,
   skips the copy and the swap (`worktree_convert.rs:257-270`),
   and reaches the same refusal at the same step.
6. `--abandon` refuses as well,
   because it acts only before the swap (`worktree_convert.rs:112-114`).

Wrong result:
a conversion that can never finish on that worktree.
The worktree path holds a subvolume that lacks the socket,
a complete second copy sits at `W.cli-git-converting` with a journal nothing can
close,
and each rerun prints the same failure.
The only way out is to delete an entry by hand from a directory whose name only
the wrapper uses,
which is what the message asks for in the abstract ("settle which version is
wanted by hand").

The same shape reaches any entry the copy deliberately does not recreate:
the two types are sockets and device nodes
(`subvolume_sync.rs:182`).
Named pipes are recreated (`subvolume_sync_copy_one_entry.rs:57`),
so they are not affected.

Suggestion,
not a decision:
have `require_complete`
(`worktree_convert_phases.rs:166`)
refuse a first pass with a non-empty `special_skipped`,
naming the entries,
which stops the conversion before the swap,
where `--abandon` still works.
The alternative is to let the comparison pass over an entry the copy never
promised to recreate,
which keeps the socket only in the deleted old tree.

### 4. A named pipe where the migration reads blocks it forever, with the claim held

Reproduced for the read;
the consequence for a second command is read from the code.

Sequence:

1. A named pipe sits at `<parent>/.cli-git-migrate-<name>.journal.json`,
   the path the command reads its journal from
   (`subvolume_preflight.rs:174-190`).
   The file sits beside the checkout,
   so any process that can write that directory can create it,
   and nothing in the command checks its type before reading it.
2. `migrate-to-subvolume` takes the claim
   (`subvolume_command.rs:171`),
   then `migrate` calls `Run::open` (`subvolume_migrate.rs:359`),
   which reads the journal with `std::fs::read_to_string`
   (`subvolume_journal.rs:181`).
3. `open` on a named pipe for reading waits for a writer,
   and no writer comes.

Wrong result:
the command waits with no end and no output,
holding the claim file that names its live process.
Every later command on that checkout is refused as busy with that process
(`subvolume_command.rs:173-180`),
and every conversion in the same repository is refused too,
because a live
migration claim is one of its refusals
(`worktree_convert_inspection.rs:221`).
Nothing says why,
and the person has to find and kill the waiting process.
The conversion's own reader refuses the same file at once
(`worktree_convert_journal.rs:487-489`,
which opens with `O_NONBLOCK | O_NOFOLLOW` and then refuses anything that is not
a plain file),
which is the behavior `SPEC.md:1445-1447` promises:
"A journal,
a registration file or a `.git` that is a named pipe or any other
file that is not plain ends the run with a reason instead of blocking while the
lock is held."

The same shape is on every other file the migration reads with a plain call:
the unreadable list (`subvolume_run.rs:80`),
the index (`subvolume_index_files.rs:289`),
the repository's configuration (`subvolume_index_files.rs:483`),
and every registration's `gitdir`
(`subvolume_registered_worktrees.rs:140`).

Suggestion,
not a decision:
read all five through `read_plain_file`,
which already exists and is what the conversion uses,
and refuse rather than wait.

### 5. A preview promises a deletion the real run refuses

Reproduced.

`run_mode` opens the run and returns on `--dry-run`
before either mode's refusals are looked at
(`subvolume_command.rs:190-197`),
so the preview answers a question nobody asked it to check:

```text
dry-run --remove-old with no journal: exit 0:
["dry run: the command would delete the original tree of .../checkout;
nothing was changed."]
```

while the same invocation without `--dry-run` refuses with exit status 1,
because `remove_old_refusals` needs a journal that records `verify`
(`subvolume_preflight.rs:414-429`).

The preview also takes no claim (`subvolume_command.rs:168`),
so it can read a checkout another command is in the middle of exchanging.

Wrong result:
a preview that says a step is available when the real run refuses it.
Nothing is changed and no data is at risk,
which is why this ranks last.

Suggestion,
not a decision:
run both refusals in the preview and print them,
which is the same call the real run makes.

## Every state change and what a kill leaves

The earlier page walks both routes at their journaled steps.
What follows keeps that walk short and adds the states between those steps,
the ones a kill inside one pass leaves,
since that is where a person's work sits.
Every line here was read from the code.

The conversion's states,
in order:

1. After `ensure_stage` created the subvolume and before the journal records its
   identity:
   an empty directory the journal cannot prove,
   which every later run refuses with a message asking for `--abandon`
   (`worktree_convert_phases.rs:130-145`).
   Nothing is lost:
   the stage is empty.
2. Inside the first copy:
   a file may be half written under `.cli-git-migrate-partial` in one
   destination directory
   (`subvolume_sync_pass_state_and_checks.rs:34`),
   and the next pass in any directory removes that name before it writes
   (`subvolume_sync.rs:112`).
   A rerun copies from the start.
3. Between the two passes:
   `index.lock` is taken with the token
   (`worktree_convert_phases.rs:227`),
   so a Git writer to the index is held out from here to the end of the rewrite.
   A kill here leaves the lock and a journal before `synced`,
   and the rerun accepts its own lock (`worktree_convert.rs:219-223`)
   and proves the copy again.
4. Inside the proving pass:
   the journal still says `creating`,
   so a kill is repaired by repeating both passes,
   never by trusting a partial proof.
5. Between the end of the proving pass and the swap:
   finding 1's window.
   A content change here is caught by the removal comparison and keeps the old
   tree with a message naming the entry;
   a mode,
   owner or attribute change is not.
   A process that appears here is caught by `refusals_after_lock`
   (`worktree_convert_phases.rs:315`),
   which re-reads the process table just before the exchange.
6. Between the exchange and its journal line:
   the journal is one phase behind,
   and the recorded identity of the old root says where the old tree is
   (`worktree_convert.rs:190-206`),
   so a rerun neither exchanges back nor loses the tree.
7. Between the exchange and `verify_swap`:
   the checkout holds a subvolume that may not be the one this command created,
   and `verify_swap` refuses with the old tree kept
   (`worktree_convert_phases.rs:353-372`).
8. Inside the index rewrite:
   the new index is written into the held lock file and the lock is renamed over
   the index,
   so the index is either the old or the new one.
   A kill leaves the lock held and the journal before `refreshed`,
   and the rerun takes the lock again (`worktree_convert.rs:262-266`).
9. Between the lock release and the removal:
   another command may remove the worktree from here,
   which is the earlier
   page's finding 3.
10. Inside the removal walk:
    entries are unlinked one at a time
    (`worktree_convert_remove.rs:110-113`).
    A kill leaves a partly deleted old tree,
    the name still held by the recorded directory,
    and the journal before `removed`,
    so a rerun opens the same directory and continues.
    The comparison runs again first,
    and it asks only that the installed tree holds everything the old one does,
    so the entries a kill already removed from the old tree do not stop it.
11. After the last entry is gone and before the journal records `removed`:
    the rerun opens the name,
    finds nothing,
    and counts it absent
    (`worktree_convert_remove.rs:87`).
12. After `removed` and before `finish`:
    the journal is the only leftover,
    and the rerun removes it.

The migration's states,
adding what the earlier page's walk does not name:

1. Between `acquire_all` of the checkout and `mark("lock", "started")`
   (`subvolume_migrate.rs:205-211`):
   `index.lock` files holding the journal's token exist while the journal has no
   `lock` step.
   They are this command's own,
   because the token is the journal's start time
   (`subvolume_lock.rs:74`),
   which the `create` step persisted long before
   (`subvolume_migrate.rs:142`,
   `subvolume_journal.rs:297`),
   so the rerun's `is_ours` exempts them
   (`subvolume_preflight.rs:334-388`)
   and `acquire` accepts them (`subvolume_lock.rs:146-160`).
   Until the rerun or the rollback,
   a person's Git commands in the checkout fail
   on that file,
   and the failure message names the journal.
2. Between `mark("exchange", "started")` and the exchange call
   (`subvolume_migrate.rs:211-214`):
   the journal claims the exchange began while both directories are still in
   place.
   The rerun finds the checkout is not a subvolume,
   so it repeats the delta,
   the lock acquisition and the exchange,
   which is `RENAME_EXCHANGE` on the same two names and idempotent.
3. Inside `create_subvolumes` for the nested worktrees
   (`subvolume_migrate.rs:140-165`):
   each nested subvolume is created and none is journaled.
   A kill leaves some of them made and the journal without `nested`,
   and a rerun skips the ones that exist.
4. Inside a copy pass:
   a partially written entry is at `.cli-git-migrate-partial` in one destination
   directory and is removed by the next pass
   (`subvolume_sync_pass_state_and_checks.rs:34`);
   a pass interrupted anywhere is repeated whole.
5. Between the exchange and the rename of `<root>.subvol-new` to
   `<root>.before-subvolume`:
   the checkout is the new subvolume and the old tree still sits at
   `<root>.subvol-new`.
   The rerun renames it (`subvolume_migrate.rs:196-205`),
   and `--rollback` finishes the same bookkeeping before it swaps back
   (`subvolume_rollback.rs:220-227`).
6. Inside `--remove-old`'s `remove_tree`:
   the old tree is partly deleted and the journal still records `verify`.
   A rerun carries the late writes and deletes the rest.
   A `--rollback` from that state refills the hole from the checkout before it
   swaps back,
   which is the first block under "Reproductions".
   I expected the opposite here and measured it instead;
   the refill is the `update_only` pass copying every entry the destination
   lacks (`subvolume_rollback.rs:168`).
7. Inside the rollback's own exchange:
   the journal records `rollback-swap` started,
   and the checkout path tells the rerun which side it is on
   (`subvolume_rollback.rs:160`):
   a subvolume there means the exchange did not happen,
   anything else means only the rename and the record are left.

## Locks, waits and their ceilings

Every lock in the route,
and why it exists:

- The conversion's `index.lock` of the worktree
  (`worktree_convert_phases.rs:227`),
  from the proving pass to the end of the index rewrite (`:394`).
  Git's own protocol:
  only its presence keeps a Git writer from replacing the
  index across the exchange.
  One file,
  one index,
  created with `O_EXCL`,
  never waited for,
  and removed only while it holds this command's token
  (`worktree_convert_phases.rs:187`).
- The migration's `index.lock` of the checkout and of every registered
  worktree's index (`subvolume_migrate.rs:205-211`),
  from the last delta to the end of the index rewrite (`:298`).
  Same reason,
  same unit,
  same refusal of a foreign lock
  (`subvolume_lock.rs:146-160`).
- The stat rewrite's own `index.lock` when it runs by itself
  (`subvolume_index_files.rs:243`,
  `IndexLock::Acquire`,
  used by the rollback's refresh at `subvolume_rollback.rs:239`):
  `O_EXCL`,
  never waited for,
  removed only while the name still holds the file it created,
  and the index is skipped with a line when it is taken.
  The earlier page's locks section does not list this one.
- The kernel `flock` probe of the file release 0.5.20 left
  (`worktree_convert_journal.rs:543`):
  `LOCK_EX | LOCK_NB`,
  given straight back,
  and the file removed only while the probe still holds it.

Everything else is lock-free by design:
the claim is a numbered file published with one exclusive `link`
(`single_owner_claim.rs:273`),
the journals are replaced by an exclusively created sibling and a rename,
and the stage and the old tree are told apart by recorded identities.

Every wait,
and what ends it:

- No sleep,
  poll,
  retry loop or timer exists in the route;
  `rg 'sleep\(|park\(|recv\(|Instant \+'` over the two folders matches test code
  only,
  except the runner below.
- The claim takes at most 64 rounds and never waits
  (`single_owner_claim.rs:70`,
  `:348`).
  A live owner is a refusal with its process id.
- The conversion's parallel runner waits on its own workers'
  channel (`foundation/parallel_runner.rs:300`),
  which closes when every lane has finished:
  a worker blocked in the kernel holds the command,
  and nothing else can end it.
- `sync_all` on every journal write,
  claim publication and index install
  (`subvolume_file_write_steps.rs`):
  unbounded,
  and deliberate:
  the owner's rule that the final disk save is not cut short.
- The btrfs ioctls and `renameat2` do not wait on anything user space can see;
  the kernel's own transaction is inside them.
- Finding 4 is the one unbounded wait this page found that is not deliberate.
- Two reads can wait on a state another process creates,
  which the
  accepted-risk notice covers only as a lost change,
  not as a hang:
  `copy_file` opens a listed file with a plain `File::open`
  (`subvolume_sync_copy_one_entry.rs:160`),
  so an entry replaced by a named pipe between the listing and the open blocks
  the pass;
  `open_fifo` (`subvolume_sync_attrs.rs`)
  uses `O_NONBLOCK` for the pipes it does expect.

  Suggestion,
  not a decision:
  add `O_NONBLOCK` to the copy's source open,
  which turns that hang into an error naming the entry.

## Reproductions

The throwaway file is
`package/git-policy/cli/src/native/subvolume/audit_scratch_tests.rs`,
registered in that module for these runs and not committed.
It drives the real command on the stand-in platform of the migration controls
(`subvolume_test_support.rs:55`,
`PlainPlatform`,
a plain directory tree whose "subvolume" is a registered device and inode)
and the real conversion on real worktree fixtures with real Git.
The fixture root is `${HOME}/temp/agent/pi-subvolume-audit-fixtures`,
chosen short because a Unix socket path is limited to 107 bytes.

Command,
inside a `systemd-run --user --scope` with `CPUQuota=600%`,
`MemoryMax=24G`,
`TasksMax=512`,
`IOWeight=10`,
with `CLI_GIT_TEST_ROOT` naming that fixture root:

```console
mise run //package/git-policy/cli:native:test:host -- --no-capture audit_scratch_tests
```

### A change the removal comparison cannot see

Two trees with one file each,
written separately and then given the same
modification time,
so each arm differs in nothing but its own field.
The last arm is the positive control:
the probe can show a difference.

```text
--- arm mode: first_difference -> None
    removal outcome: Removed
    the old tree is gone: true
--- arm xattr: first_difference -> None
    removal outcome: Removed
    the old tree is gone: true
    setxattr accepted: true
--- arm size: first_difference -> Some("sub/x.txt differs from the installed worktree (size or modification time)")
    removal outcome: Differs("sub/x.txt differs from the installed worktree (size or modification time)")
    the old tree is gone: false
```

The mode arm made the old tree's file 0755 and the installed one 0644;
the xattr arm gave the old tree's file a `user.*` attribute the installed copy
does not have.
Both old trees were deleted.

### The silence that follows finding 1's deletion

The state after the swap is a tree with the copy's bytes and a copy of the
index.
Made directly,
by copying a repository with its index and changing the mode in only one of the
two:

```text
--- the tree the person chmod'ed (the old tree):
 M s.sh
--- the copied tree with the copied index (the installed tree):
```

The old tree reports a modified file;
the copied tree,
which is what the worktree holds after the swap,
reports nothing.
The index rewrite only makes the stat fields agree with the installed files
(`subvolume_index.rs:554`),
and the mode field is not among them,
so the clean report is what a person sees after the command has discarded the
change.

### A socket in a worktree, two reruns, and the abandon

```text
--- round 1 verdict: Err(Journal("the old tree at .../checkout/wt.cli-git-converting is kept, and nothing was copied from it: agent.sock is missing from the installed worktree; settle which version is wanted by hand, then run the command again to remove it"))
      copied 12 entries (7 files, 0 cloned)
      index: 6 of 7 entries refreshed
    worktree is a subvolume: Ok(true)
    old tree still beside it: true
    journal still there: true
    socket still in the worktree: false
--- round 2 verdict: Err(Journal("the old tree at .../checkout/wt.cli-git-converting is kept, and nothing was copied from it: agent.sock is missing from the installed worktree; settle which version is wanted by hand, then run the command again to remove it"))
    worktree is a subvolume: Ok(true)
    old tree still beside it: true
    journal still there: true
    socket still in the worktree: false
--- abandon verdict: Ok(Skipped(["past the swap"])): ["  the swap already happened; run the command again to finish it (there is nothing to undo)"]
```

Finding 3,
and finding 2's silent-skip half:
a copy that left an entry out printed no line about it.

The same socket inside a main checkout,
through the migration,
for the contrast:
there the pass names it every time.

```text
copy: not recreated (a socket or device node): agent.sock
first delta: not recreated (a socket or device node): agent.sock
last delta (under the lock): not recreated (a socket or device node): agent.sock
catch-up (files newer in the old tree): not recreated (a socket or device node): agent.sock
...
late writes (files written to the old tree after the catch-up): 1 entr(ies) absent from the destination were left out because they predate the catch-up
the socket is gone with the old tree: true
```

Two things to read there.
The migration names the socket during every pass and then,
at `--remove-old`,
the socket is the entry the left-out count is about and it is deleted with no
second word.
That count cannot tell a person's restored file from a special file the copy
never promised,
which is the earlier page's finding 1 seen from the other side.

### A named pipe where the migration's journal belongs

The read the migration makes,
and the read the conversion makes,
on one named
pipe,
each in a thread with a five-second ceiling.
The same two calls on a regular journal file are the control.

```text
mkfifo exit: Some(0)
regular file: Journal::load returned (persisted true)
regular file: read_plain_file returned 37 bytes
named pipe: Journal::load did not return within 5 s
named pipe: read_plain_file refused it: it is not a plain file (a named pipe, a device or a directory)
```

Finding 4.

### A rollback after a removal that stopped part way

The earlier page's finding 1 is about `--remove-old` losing a file it left out.
The question here was the other order:
the removal is interrupted,
and the person then does what the failure message
advises ("add --rollback to restore the original directory",
`subvolume_command.rs:154`).

```text
migrate exit 0
before: the checkout holds README.md: true
before: the old tree holds README.md: true
after the stopped removal: the old tree holds README.md: false
rollback exit 0
    carry back (files newer in the new root): 78 entries, 3 files copied (0 cloned), 0 hard-linked, 0 symlinks, 43 unchanged, 0 deleted, 0 vanished, 0.01 s
    index .../checkout/.git/index: 6 of 7 entries refreshed
    deleting the new subvolume .../checkout.subvol-new (nested subvolumes first)
    rolled back: .../checkout is the original directory again, and the migration journal is deleted.
after: the checkout holds README.md: true
after: the checkout holds src/deep/er/leaf.txt: true
after: the new subvolume is gone: true
after: the checkout is a subvolume: Ok(false)
after: git status --porcelain: ""
```

The state a kill inside `remove_tree` leaves was made by unlinking two files
from the old tree by hand;
the rest is the real command.
The rollback copied the deleted entries back from the checkout before it
swapped,
so the person's tree came back whole and Git saw a clean worktree.
This is a state I expected to be a finding and it is sound.

The last line of that run is finding 5:

```text
dry-run --remove-old with no journal: exit 0:
["dry run: the command would delete the original tree of .../checkout;
nothing was changed."]
```

## What I checked and found sound

So that nobody repeats it:

- A rollback after a partly deleted old tree:
  the `update_only` carry-back refills every entry the old tree lacks before the
  swap (`subvolume_rollback.rs:168`,
  `subvolume_sync.rs:149`),
  and the rollback run shows a clean checkout afterwards.
  The only refusal on that path is for a missing old tree
  (`subvolume_rollback.rs:160-165`),
  which is the right test for the case it can see.
- Non-btrfs and a mount point are refused before anything is created,
  with the remedy in the same message
  (`subvolume_preflight.rs:239-286`),
  and a root that is already a subvolume with no journal is refused rather than
  converted again (`:287-300`).
- Nearly full filesystems are refused before the copy starts,
  from the entry count and the measured metadata cost per entry
  (`subvolume_preflight.rs:51-87`,
  `:254-266`);
  if the filesystem fills anyway the pass fails with the entry named,
  the journal keeps the state,
  and a rerun after freeing space continues.
  The provision is metadata only because the copy shares data blocks;
  the fallback for a filesystem that refuses `FICLONE`
  (`subvolume_sync_copy_one_entry.rs:185`)
  copies data and is not provisioned for,
  which is a limit,
  not a loss.
- The migration carries a late permission change of an existing file:
  its update-only passes compare the mode (`subvolume_sync_pass_state_and_checks.rs:191`)
  and copy a file that differs,
  so the mode change is in the checkout before `--remove-old` deletes the old
  tree.
  This is why finding 1 is a conversion finding and not a migration one.
- Modes,
  owners,
  groups,
  times to the nanosecond and the carried extended
  attributes (user attributes and POSIX ACLs) are all applied through the
  descriptors of the copies
  (`subvolume_sync_attrs.rs:402`),
  so a path swapped for a link cannot redirect them,
  and the security label is deliberately left to the policy.
- Hard links are recreated as links inside one subvolume and as separate copies
  across a subvolume boundary,
  which the copy records so a later pass does not take two copies for one file
  (`subvolume_sync_copy_one_entry.rs:87-152`).
- A file this user cannot read stops a conversion before the swap and is listed
  for a migration (`worktree_convert_phases.rs:166`,
  `subvolume_run.rs:124`),
  and the list is what refuses `--remove-old` without an explicit flag
  (`subvolume_rollback.rs:303-320`).
- The recursive deletions never follow a link and never cross a device,
  subvolume or mount
  (`subvolume_remove_walk.rs:169`,
  `subvolume_remove.rs:73`).
- The migration's exchange is recovered from the directory's own state rather
  than the journal's last line
  (`subvolume_migrate.rs:196-205`),
  and the conversion's the same way from the recorded identity of the old root
  (`worktree_convert.rs:190-206`).
- The index rewrite is conservative in the direction that matters:
  an entry whose modification time is not older than the index file keeps its
  fields,
  so Git hashes it (`subvolume_index.rs:530-539`);
  an entry whose file does not agree with the tree the copy came from is not
  refreshed (`subvolume_index_files.rs:366-378`);
  declined shapes (split and sparse indexes) are reported instead of guessed at;
  and an index whose lock another command holds is skipped with a line
  (`subvolume_index_files.rs:246`).
- The two identities that make a deletion safe are recorded and compared:
  the stage the conversion created (`worktree_convert_phases.rs:130-145`)
  and the old root it is about to delete
  (`worktree_convert_remove.rs:84-93`),
  both with birth time,
  so a reused inode is not mistaken for the recorded one.
- The conversion refuses a worktree that is locked,
  has an `index.lock`,
  is not
  on btrfs,
  has a worktree registered inside it,
  is not quiet,
  or has a process
  inside it,
  and reports every reason at once
  (`worktree_convert_inspection.rs:263-306`).
- `--abandon` destroys only the subvolume whose identity the journal recorded
  and only before the swap (`worktree_convert.rs:110-130`).
- One claim file per item,
  published by one exclusive link,
  with a live owner
  refused in under a millisecond rather than queued
  (the earlier page measured this;
  the code is `single_owner_claim.rs:273-390`).

## What I could not determine

- Whether finding 1's window is hit in practice.
  There is no seam between the end of the proving pass and the removal phase,
  so what was reproduced is the comparison and the deletion,
  not a real `chmod` made through a whole conversion.
- How often a worktree holds a Unix socket.
  No census was taken;
  the earlier page's census of this host's 356 registrations counted subvolume
  roots and ordinary directories,
  not sockets.
- Whether any worktree on this host holds a file owned by another user or an
  ACL the person cannot set.
  Making one needs root,
  which this audit did not use,
  so finding 2's attribute half rests on the code.
- Whether a named pipe ever appears at one of the five paths of finding 4.
  The read is reproduced;
  how such a file gets there was not measured.
- The installed release's behavior:
  every run used the test binary built from this revision,
  not the installed wrapper.
- Windows and macOS,
  which the owner's pause covers.

## What to inspect and how to respond

Inspect,
in this order:

1. finding 1 against `worktree_convert_compare.rs:40-72`,
   `worktree_convert_remove.rs:99-120` and `subvolume_index.rs:554`,
   with the first block under "Reproductions";
2. finding 2 against `worktree_convert_phases.rs:222-224` and
   `subvolume_run.rs:196-212`,
   and the socket block that shows the silence;
3. finding 3 against the socket block's two rounds and the abandon line;
4. finding 4 against the named-pipe block and `subvolume_journal.rs:181`;
5. finding 5 against the last line of the rollback block.

Respond with:

- whether finding 1 should be fixed by refusing (keep the old tree,
  name the
  entry) or by copying the difference into the installed tree;
- whether a conversion should refuse a worktree holding a socket and a device
  node before the swap (finding 3) or pass over them;
- whether the attribute losses the copy counts should refuse a conversion
  (finding 2) or only be printed;
- whether every file the migration reads should go through `read_plain_file`
  (finding 4),
  which is a small change with one new refusal message per file;
- and whether a preview should run the refusals of the mode it previews
  (finding 5).

No production code was changed,
so nothing else is pending from this audit.

## Fixed after this audit

Branch
`fix/git-policy-cli-conversion-compares-modes-and-can-finish`,
cut from `main` at `2e267409a8`
and not merged to it in this record.
The page above stands as the record of the audited revision.
Each finding got a control written first and shown failing,
then the fix that made it pass,
in one commit for the control and one for the fix;
`SPEC.md` states the behaviour the route now has.
Every path below is relative to `package/git-policy/cli/src/native/`.

- Finding 1,
  control `c11a436b49`,
  fix `f129364b01`:
  `first_difference`
  (`worktree_convert_compare.rs`)
  now also compares the permission bits,
  the owner,
  the group and the carried extended attributes of every entry of both trees,
  beside the type,
  size,
  modification time and link target it compared already,
  and keeps the old tree naming the entry and the attribute,
  which is the page's first suggestion.
  Nothing is copied from the old tree into the installed one.
  The attributes are judged by `carried_attributes_equal`,
  the call the copy itself uses,
  so the comparison and the copy agree on what "carried" means;
  a security label the policy sets on the copies is not one of them.
  Cost,
  measured on a 24,345-entry tree extracted from this repository's `HEAD` into a disposable
  directory,
  five rounds alternating the arms:
  12.60 µs per entry for the walk as it was,
  12.70 with the permission bits,
  owner and group,
  and 38.02 with the attributes as well,
  which is 0.62 s more on that tree,
  paid once per conversion.
  Controls:
  `a_change_of_the_mode_an_owner_or_an_attribute_is_a_difference`,
  `an_owner_change_is_a_difference`,
  which runs only as root and says so where it cannot,
  and `a_permission_change_after_the_proof_keeps_the_old_tree`.
- Finding 2,
  control `7f4a092050`,
  fix `f16d6b9bf1`:
  both commands now say what each pass could not carry,
  through one line builder (`subvolume_pass_report.rs`),
  and the conversion says it for both of its passes.
  What is said:
  every socket and device node not recreated,
  the owner,
  permission bits,
  times and extended attributes the kernel refused,
  the destination entries kept,
  the entries left out because they predate the catch-up,
  and the files that changed while they were copied.
  The attribute losses are printed and do not refuse,
  which is the second of the page's two options for them;
  a socket or a device node does refuse,
  which is finding 3.
  Controls:
  `a_copy_that_leaves_a_socket_out_says_so`,
  `each_kind_of_what_a_pass_could_not_carry_is_said_with_its_label`,
  and `a_pass_that_lost_nothing_says_nothing`.
- Finding 3,
  control `ff868602f7`,
  fix `1f38cd7ca2`:
  `require_complete`
  (`worktree_convert_phases.rs`)
  now also fails a pass that did not recreate a socket or a device node,
  naming up to twenty entries and the count and saying that nothing was swapped,
  which is the state `--abandon` ends;
  a socket that reaches the old tree after the swap is named by the comparison with the file
  to delete rather than a version to choose,
  so a conversion already past its swap is finished the same way.
  Controls:
  `a_socket_stops_a_conversion_before_the_swap_and_deleting_it_finishes`,
  `a_socket_left_in_the_old_tree_names_the_entry_to_delete`,
  and `a_conversion_refused_over_a_socket_can_still_be_abandoned`.
- Finding 4,
  control `488a045e96`,
  fix `36c8f1d9c5`:
  `read_plain_file` and `read_plain_text` moved from the conversion's journal to
  `foundation/plain_file.rs`,
  and the five reads the page names go through them:
  the migration's journal (`subvolume_journal.rs`),
  its list of unreadable files (`subvolume_run.rs`),
  an index and the repository's configuration (`subvolume_index_files.rs`),
  and every registration's `gitdir` (`subvolume_registered_worktrees.rs`).
  Each opens with `O_NONBLOCK`,
  `O_NOFOLLOW` and `O_CLOEXEC`,
  takes the type from the open handle,
  so a name swapped for a link or a pipe cannot redirect the read,
  and reads no more than its limit.
  An index keeps its own bound of a gigabyte;
  the record files share one mebibyte,
  and a longer one is refused with its path named rather than read whole.
  Controls:
  `a_named_pipe_where_the_journal_belongs_is_refused_at_once`,
  `a_named_pipe_where_the_unreadable_list_belongs_ends_the_run_at_once`,
  `a_named_pipe_where_the_index_belongs_is_skipped_at_once`,
  `a_named_pipe_where_the_configuration_belongs_reads_as_none_at_once`,
  `a_named_pipe_where_a_registration_gitdir_belongs_is_left_out_at_once`,
  `a_plain_file_is_read_whole_and_a_larger_one_is_refused`,
  `anything_that_is_not_a_plain_file_is_refused_at_once`,
  and `bytes_that_are_not_text_are_refused_as_text`.
- Finding 5,
  control `6dad15c375`,
  fix `dc005c3c78`:
  a preview now runs the refusals of the mode it previews,
  prints them in the words the real run uses and exits with status 1 when any stands;
  only a mode whose refusals are clear says what it would do.
  The refusals of a rollback and of a deletion are built by
  `refusals_of_the_mode` and its helpers
  (`subvolume_rollback.rs`),
  which the preview and the real run both call,
  so their words cannot drift from the preview's.
  Control:
  `a_preview_refuses_what_the_real_run_would_refuse`,
  which also previews a finished migration as the deletion the real run makes,
  so a preview that refused everything would not pass.

Still open after this work:

- Finding 1's window in practice.
  No seam was added between the proving pass and the removal phase,
  so the control kills the run after the exchange,
  narrows a file in the old tree and resumes,
  and asserts the old tree is kept;
  a `chmod` made inside one run's window remains unmeasured.
- A device node's refusal has no control of its own:
  making one needs root,
  which this work did not use;
  the socket controls exercise the same branch.
- Whether any worktree holds a socket,
  and whether a named pipe ever appears at one of the five read paths,
  remain uncensused,
  as the page's "What I could not determine" says.
- The mebibyte bound on the migration's journal and on its list of unreadable files has no
  measurement:
  no such file existed on this host to measure.
  The two record files that do exist are measured on this repository:
  `.git/config` at 74,878 bytes and a registration's `gitdir` at 122.
- The installed release,
  Windows and macOS,
  unchanged:
  every run above used the test binary built from these revisions.
