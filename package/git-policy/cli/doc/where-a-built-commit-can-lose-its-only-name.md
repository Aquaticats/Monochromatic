# Where a built commit can lose its only name

A commit Git built for a batch exists as an object before any branch names it,
and a commit no reference names is one `git gc --prune=now` takes.
This record walks that path and says,
for each point where the commit is named by exactly one reference,
what proves another name exists before that reference is removed.

The rule the fixes share
(the owner's rule "never discard user data"):

- a reference that is the only name of a built commit is deleted only after another name for that
  commit is known to be written;
- the delete is a compare-and-delete,
  `git update-ref -d <ref> <old>` with the value that was read,
  so a reference another command wrote between the read and the removal is never removed;
- when no other name can be written the reference stays,
  and the person is told the commit's name
  and which reference holds it.

`keep_an_unlanded_commit` (`src/native/landing/service_land_side_reference_write.rs`) carries that
rule for every ending that answers a batch as not made.

## The path

### 1. The objects exist, nothing names them yet

`import_stream` names the batch's reference inside the import
(`commit refs/worktree/cli-git-service/<batch>`),
so `git fast-import` writes the commit objects and
updates that reference at its end.
A commit Git built elsewhere is named by the client's shadow until the landing writes the batch's
reference (`src/native/landing/service_land_objects_of_batch.rs`,
the `keep_on_the_scratch_reference`
call).

**Nothing proves a name here.**
This is the one window left open.
The wrapper's automatic maintenance uses `gc --auto` with the default grace,
so only a concurrent `gc --prune=now` in that instant takes the commit,
and closing it means naming the reference in the same import transaction for every case,
which is a change to how batches are built and has not been decided.

### 2. One name: `refs/worktree/cli-git-service/<batch>`

Every way this reference is removed,
and the proof it now has:

- A step ended at its ceiling (`keep_what_an_ended_step_built`):
  the kept reference is proven written,
  then the reference goes compare-and-delete;
  a keep that cannot be written leaves it and the person is told the commit's name and that
  reference.
- The index lock refused,
  for any code and not only another program's lock
  (`src/native/landing/service_land_take_index_lock.rs`):
  the same ending.
- The branch write Git never started,
  for a reason that is not a ceiling
  (`end_a_batch_the_write_never_started`):
  the same ending.
- The branch write refused and the wait ended with nothing standing,
  so the refusal is Git's own
  (`give_up`):
  the same ending.
- The hold record cannot be written (`hold_the_batch`,
  its `Err` arm):
  the same ending.
- The worktree switched after the lock,
  the branch moved under the batch,
  or a refused write may be
  tried again (`src/native/landing/service_land_batch_body.rs`):
  compare-and-delete only,
  because the batch is built again from the same inputs and the object is
  not the only copy of the person's work.
- A client that went away after the lock (`drop_the_batch`):
  compare-and-delete only,
  because nothing reads that request's answer.
- A landing that fails after its objects and a restart that drops an open batch
  (`clear_after_dropped_batch` in `src/native/landing/service_scratch_ref_of_a_dropped_batch.rs`):
  the kept reference is written and proven,
  then the reference goes compare-and-delete.
- A service killed between the import and the intent:
  the `<batch>.scratch` record names the worktree,
  and the restart's sweep does what the drop does
  (`src/native/landing/service_scratch_ref_marker_of_a_batch.rs`).
- A service killed after the intent:
  the intent lets the restart decide the batch from the branch,
  and the reference stays until then.
- A worked tree removal (`src/native/worktree/worktree_removal_held_commits.rs`):
  the held commit is kept at `refs/cli-git/kept/<batch>` or,
  when that name is taken or locked,
  at
  `refs/cli-git/kept/<batch>-rescue`,
  before Git removes the worktree;
  when neither can be written the command stops before Git runs.

### 3. The branch names it

The branch move and the delete of the batch's reference are one `update-ref --stdin` transaction
(`src/native/landing/service_land_branch_move_transaction.rs`,
`src/native/landing/held_commit_settle_move.rs`),
so the branch is a name for the commit before the reference goes.

### 4. The branch names it and a kept reference may remain

A kept reference that names a commit the branch also has costs space once that commit leaves the
branch,
never the person's data.
The sweep that drops a batch removes it compare-and-delete only when it proves the commit safe.

## What a person is told

Every ending that answers a batch as not made says the commit's name and where it is kept:
`refs/cli-git/kept/<batch>` when the keep was written,
or the batch's own reference when it was not,
with the words Git gave for the refused keep.
`SPEC.md`,
"Commit protocol",
states the same in the place a reader checks the guarantee.
