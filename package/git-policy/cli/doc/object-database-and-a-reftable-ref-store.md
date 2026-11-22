# The object database reader and a reftable ref store

**Unapproved and non-final.**
This note records what the in-process object database answered on a reftable repository,
which shipped callers reached it,
the refusal that now stands in `repo_read_odb.rs`,
and what the refusal costs.
Every number was measured on this host on 2026-11-21,
in the worktree `.claude/worktrees/object-database-reftable-replace-refs`,
branch `fix/git-policy-cli-object-database-reftable-replace-refs`.
The owner has not approved it,
and only the owner declares it final.

## The defect on `main`

`ObjectDatabase::open` refuses a repository whose object names mean more than the files say:
a replace ref under `refs/replace`,
a `packed-refs` file naming `refs/replace/`,
a `shallow` file,
an `info/grafts` file
(`repo_read_odb.rs`,
`objects_are_replaced`).
A reftable repository keeps its replace refs in the `reftable` tables,
and has neither `refs/replace` nor `packed-refs`,
so the database opened and answered objects as the loose files and packs hold them,
while Git answers the replacement.

The fixture,
built in the scratch folder with real Git:

- `git init --ref-format=reftable`,
  two commits made with `hash-object`,
  `mktree` and `commit-tree`,
  the branch moved with `update-ref`;
- `git replace <first> <second>`.
- `for-each-ref` lists `refs/replace/<first>`;
  the directory `.git/refs/replace` and the file `.git/packed-refs` do not exist,
  and `.git/reftable` does.
- Git answers 221 bytes for `cat-file commit <first>`,
  the replacement,
  and 172 bytes with `--no-replace-objects`,
  the commit the files hold.

What `main`'s reader answered there,
from the probe in `scratch/odb-probe` linked against this branch's base
(`5632932...` is the released wrapper of that base,
`9f95002...` the probe):

- `odb-probe open .git` printed `Some`.
- `odb-probe read .git <first>` printed `kind=commit length=172 digest=94eb0a286f1a5e9d`,
  the commit the replace ref stands for,
  while Git answers the replacement.

## Which shipped callers reach it

`ObjectDatabase::open` has two shipped entry points,
and one group of callers gives it a value no layout read vouched for:

- The repository layout,
  `read_layout`,
  refuses a reftable repository twice:
  `extensions.refstorage` with any value but `files` returns `None`
  (`repo_read_layout.rs`,
  `format_of`),
  and a `reftable` directory in the Git directory or the common directory returns `None`
  (`repo_read_layout.rs`,
  `finish`).
  The clean check (`worktree_clean.rs`),
  the ladder target (`worktree_ladder_target.rs`),
  the index reset (`index_reset_file.rs`),
  the LFS plan (`lfs_object_plan.rs`) and the push trigger (`auto_push_trigger.rs`) open the database
  from that layout,
  so on a reftable repository they never reach the reader and ask Git.
- The landing reads objects from the `common_dir` the client's commit request carries
  (`service_built_checks.rs`,
  `service_pruned.rs`).
  The client captures that path before it commits;
  for a reftable repository the in-process layout is declined,
  and the path comes from Git's own `rev-parse` answer,
  so the service opens the database on a reftable store.
  Four functions read it:
  `commit_content`,
  `read_commit`,
  `adds_only_present_objects` and `batch_content_missing`.
  `read_commit` answers the parents an amend of a replaced commit stands on,
  and the import stream builds the new commit with them.

The shipped divergence,
in the binary control added to `binary_service_commit_tests.rs`:
an amend of the commit a replace ref stands for landed a commit whose one parent is the commit the
files hold,
while Git gives the amended commit the replacement's parents,
which are none.
The control's files-backend arm,
where the reader declines the database,
answered as Git does.

## The refusal and its controls

`objects_are_replaced` now returns `true` when `reftable` exists in the common directory,
before the replace-dir and packed-refs scans
(`repo_read_odb.rs`,
`ref_store_is_reftable`).
Nothing reads the tables,
so a reftable repository declines the whole database and the caller asks Git,
which is the case the package's `README.md` already lists for every reader.

Two controls live in the default build:
`repo_read_odb::tests::a_reftable_repositorys_replace_ref_declines_the_database` and
`service_commit::an_amend_of_a_replaced_commit_lands_with_gits_parents`.
Both were run on this tree before the refusal was added,
one per run,
and both failed:

- the reader control failed with
  `assertion left == right failed: the reader answers the replacement Git answers`,
  left the 172-byte commit and right the 221-byte one;
- the commit control failed with
  `reftable: the amended commit stands on the parents Git reads, which are none`,
  left naming the amended commit and one parent.

## What the refusal costs a files-backend repository

The change adds one `exists` call per `ObjectDatabase::open`,
on `common_dir/reftable`.
Method:
the probe linked against the reader of `main` and against the reader with the refusal,
50,000 opens per round,
16 rounds,
the three arms rotated per round,
each run in its own bounded scope,
on one files-backend fixture with two commits and loose objects.
Every figure below is nanoseconds per open:

- `main`:
  median 19,574,
  range 17,412 to 23,992.
- identical-binary control,
  the same `main` probe named a second time:
  median 19,219,
  range 17,741 to 22,356.
- with the refusal:
  median 20,997,
  range 18,960 to 22,831.
- per-round ratios,
  refusal over `main`:
  median 1.093,
  range 0.897 to 1.253,
  at or above 1.0 in 13 of 16 rounds.
- per-round ratios,
  control over `main`:
  median 0.985,
  range 0.871 to 1.266,
  at or above 1.0 in 7 of 16 rounds.
- the stat the refusal adds,
  timed by the harness's positive control on the same 50,000 iterations:
  766 ns on a path that is absent and 1,016 ns on one that is present.

So the per-open cost is one stat,
about 0.8 microseconds,
and it is what the medians show;
the single arm's round-to-round spread is wide enough that the per-round range overlaps the
identical pair's,
and the medians,
not the range,
carry the claim.

## What the refusal costs a reftable repository

The database is declined whole,
because a reader that cannot list a ref store cannot tell a store with replace refs from one without.
On a reftable repository the landing's object reads therefore ask Git where they used to answer in
process:
the amend control's service record reports `reads_parents_ms=0` before the refusal and `3` after it,
one `git rev-list --parents` start,
while the files-backend arm of the same control reports `5`.
The clean check,
the ladder target,
the index reset,
the LFS plan and the push trigger pay nothing new:
the layout already refused the repository.
A narrower refusal,
one that reads the tables for `refs/replace/`,
was not built:
the tables are compressed and prefix-coded,
and a scan that misses a name would answer wrongly instead of slowly.

## What was not measured

No guest ran:
every number is this host's ZFS,
under whatever load it carried,
and the arms alternated rather than standing alone.
The gate's own test binaries were not timed.
