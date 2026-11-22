# The object database reader and a reftable ref store

**Unapproved and non-final.**
This note records what the in-process object database answered on a reftable repository,
which shipped callers reached it,
what each asks Git when it declines,
the refusal that now stands in `repo_read_odb.rs`,
and what the refusal costs.
The first round was measured on this host on 2026-11-21,
the second on 2026-11-22,
both in the worktree `.claude/worktrees/object-database-reftable-declined-reader`,
branch `fix/git-policy-cli-object-database-reftable-replace-refs`
(the first round's worktree was `.claude/worktrees/object-database-reftable-replace-refs`).
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
built in a scratch folder with real Git:

- `git init --ref-format=reftable`,
  two commits made with `hash-object`,
  `mktree` and `commit-tree`,
  the branch moved with `update-ref`;
- `git replace <first> <second>`;
- `for-each-ref` lists `refs/replace/<first>`;
  the directory `.git/refs/replace` and the file `.git/packed-refs` do not exist,
  and `.git/reftable` does;
- Git answers 221 bytes for `cat-file commit <first>`,
  the replacement,
  and 172 bytes with `--no-replace-objects`,
  the commit the files hold.

A probe built against this branch's base called `ObjectDatabase::open` and then `read` on that
fixture's Git directory,
in process,
with no Git started:

- `open` answered `Some`;
- `read <first>` answered `kind=commit length=172 digest=94eb0a286f1a5e9d`,
  the commit the replace ref stands for,
  while Git answers the replacement.

The probe was a scratch program of one call to each function,
and it was deleted with the round that ran it.

Git 2.56 reads `extensions.refStorage` as a format name or `<format>://<payload>`
(installed `git-config(1)`,
entry `refStorage`),
and the payload is where the tables live:
a store moved out of the common directory leaves no `reftable` folder behind,
so a refusal keyed on the folder alone declined nothing there.
Measured on the same fixture with the tables moved and
`extensions.refstorage = reftable://<absolute path>`:
`for-each-ref` lists the replace ref,
`cat-file -s` gives 221 for the replaced name and 172 with `--no-replace-objects`,
and `.git/reftable` does not exist.
An amend of a replaced tip through the service there landed one parent where Git reads none,
on `main` and on the refusal that keyed on the folder.
The configuration was set by hand,
because `git init --ref-format=reftable://...` refuses
(`fatal: unknown ref storage format`).

## Which shipped callers reach it, and what each does when it declines

`ObjectDatabase::open` answers `Some` or `None`,
and `None` means "ask Git",
never "there is nothing to say".
Every shipped caller was read for what it does with that `None`:

- `commit_content` (`service_built_checks.rs`,
  used by the landing in `service_land_record_the_batch.rs` for the reflog row):
  asks Git now,
  with `cat-file commit`.
  It answered an empty message and no parents before,
  so the row lost its subject and a merge conclusion its kind.
- `read_commit` (`service_built_checks.rs`,
  for the amend's base in `service_land_batch_body.rs`,
  and for the commit Git built in `service_land_objects_of_batch.rs`):
  answers through `commit_content`,
  so Git's parents now,
  where its callers used to run `rev-list --parents` after the decline.
  The callers' own fallback stays for a repository whose history is rewritten by a shallow or
  graft file.
- `adds_only_present_objects` (`service_built_checks.rs`,
  used in `service_land_objects_of_batch.rs`):
  answers `false`,
  and the landing runs the `rev-list --objects` it always ran.
  That is asking Git,
  and it answers as before.
- `batch_content_missing` (`service_pruned.rs`,
  used in `service_land_batch_body.rs`):
  asks Git now,
  with `rev-list --objects` over the objects and trees a request names,
  and `cat-file -e` for a commit Git built.
  It answered `false` before,
  so the import ran over the missing blob and left `fast_import_crash_<pid>`.
- `prepared_content_missing` (`service_pruned.rs`,
  used by the client in `service_commit_after_a_failure.rs`):
  asks Git the same way.
  It answered `false` before,
  so the person met the service's general words instead of the content-pruned ones.
- The clean check (`worktree_clean.rs`,
  `read_state`):
  does not reach the reader on a reftable repository.
  Its `head_commit` returns `None` first,
  because a reftable repository's `HEAD` holds `refs/heads/.invalid` and a ref name whose
  component starts with a dot is not listable,
  and the check answers `Err(HEAD is not a plain ref to a commit)` on `main` and on this branch.
  On a files repository holding a replace ref the database does decline,
  and the refusal is the handled `Unreadable`:
  the snapshot route is not taken and Git's own copy of the worktree answers.
- The ladder target (`worktree_ladder_target.rs`,
  `prepare_target`):
  does not reach the reader either,
  for the same `head_commit` refusal;
  the rung is reached only from a clean source and checks the new worktree's `HEAD` first.
  On a files repository holding a replace ref the rung refuses and the add falls back to Git's
  copy.
- The index reset (`index_reset_file.rs`),
  the LFS plan (`lfs_object_plan.rs`),
  the push trigger (`auto_push_trigger.rs`) and the query context
  (`repo_read_query_request_answer_and_context.rs`) open the database from `RepoLayout`,
  and `read_layout` refuses a reftable repository twice:
  `extensions.refstorage` with any value but `files` returns `None`
  (`repo_read_layout.rs`,
  `format_of`),
  and a `reftable` directory in the Git directory or the common directory returns `None`
  (`repo_read_layout.rs`,
  `finish`).
  None of them reaches the reader on a reftable or payload store,
  and each answers Git's way there.
  On a files repository holding a replace ref the database declined on `main` too,
  so this branch changes nothing for them:
  the two readers answer `None` and the work is left to Git,
  and the push trigger declines the automatic push,
  with its own recorded reason.
  That last one is the one caller that treats a decline as a decision rather than asking Git.
  Its reason,
  `object-database-unopened`,
  is one of the words `SPEC.md` lists for a decision the wrapper could not make,
  level `warn`,
  beside a walk that runs out of budget (`walk-undecided-growth`):
  the trigger says it could not decide and does not push,
  exactly as it does when the in-process walk gives up,
  and it is unchanged from `main`.
  Asking Git there would make the wrapper hand over a push it does not hand over today,
  on a files repository holding a replace ref,
  which is a change to when automatic pushes happen rather than a caller answering wrongly.

## The refusal and its controls

`objects_are_replaced` returns `true` when `reftable` exists in the common directory,
and otherwise reads the repository's own `config` and declines
`extensions.refStorage` with any value but exactly `files`
(`repo_read_odb.rs`,
`ref_store_is_reftable`).
Git reads an extension from that file alone and honors no include of it
(`read_repository_format` in `setup.c`),
and it refuses a repository that carries one under any version but 1
(`fatal: repo version is 0, but v1-only extension found`),
so a key that names another store is declined whatever the version states.
The version's own spelling (`1`,
`01`,
`+1`) therefore cannot decide whether a store is declined,
which reading it the way Git reads an integer would have to get right.
A configuration that cannot be parsed declines the database rather than being guessed at.

Nothing reads the tables,
so a reftable repository declines the whole database and the caller asks Git,
which is the case the package's `README.md` already lists for every reader.
The `exists` checks here answer `false` on a stat error,
which fails open;
the neighbours (`shallow`,
`info/grafts`,
the layout's own `reftable` check) are written the same way,
so this is consistent and not new.
A stray `reftable` folder in a files repository declines the reader,
and Git answers;
the safe direction,
at the price of Git starts.

Five controls live in the default build:

- `repo_read_odb::tests::a_reftable_repositorys_replace_ref_declines_the_database`:
  a reftable repository with a replace ref declines the database.
- `repo_read_odb::tests::a_ref_store_the_configuration_moves_away_declines_the_database`:
  the same repository with its tables moved out and `extensions.refstorage` naming them declines,
  while Git still reads the replacement there,
  and a files repository whose configuration names `files` still opens.
- `service_commit::an_amend_of_a_replaced_commit_lands_with_gits_parents`:
  an amend of a replaced commit lands with Git's parents,
  in a files repository,
  in a reftable one,
  and in a linked worktree of the reftable one,
  each with the `rev-list` answer as its control.
- `commit::a_commit_git_builds_writes_gits_reflog_row_where_the_reader_declines`:
  the reflog row of an editor commit,
  an editor amend and a merge conclusion,
  in a reftable repository and in a files repository holding a replace ref,
  each compared with native Git's row in a twin repository.
- `service_pruned::content_removed_before_landing_fails_plainly_in_a_reftable_repository` and
  `service_pruned::content_removed_before_landing_fails_plainly_beside_a_replace_ref`:
  content removed before the landing gives the plain content-pruned failure,
  changes nothing,
  and leaves no `fast_import_crash_<pid>`;
  the three pruned controls check that last claim,
  beside the plain arm that already existed.

Each was run failing before its fix was in place:

- the configuration control failed with
  `a ref store the configuration names outside the common directory is outside what this reader
  scans`,
  after its positive control showed Git reading the replacement from the moved store;
- the reflog control failed on the reftable arm with
  `left: "commit:"` against `right: "commit: editor subject"`,
  then on the files arm with the same pair,
  then with `left: "commit (amend):"` against `right: "commit (amend): editor subject"`,
  then with `left: "commit:"` against `right: "commit (merge): merged"`;
- the two pruned arms failed with
  `left: ["fast_import_crash_1865805"]` and `left: ["fast_import_crash_1865832"]` against
  `right: []`,
  and before that assertion was moved first,
  with the service's general words in place of the content-pruned ones.

## What each declined caller now asks Git

`service_object_questions_left_to_git.rs` holds the three questions:

- `commit_bytes`:
  `git cat-file commit <name>`,
  which prints the commit Git reads,
  the replacement a replace ref names included,
  and whose parents and message a reflog row repeats.
- `objects_missing`:
  `git rev-list --objects --no-object-names --missing=print <names>`,
  which prints `?<name>` for each object it cannot read and exits zero,
  for the blobs a request names and the trees it embeds.
  Absence is read from that line and never from how Git ended:
  a Git a signal ended prints no such line,
  and an object it fails for another reason is not called gone
  (a corrupt subtree makes Git exit 128 with no `?` line,
  measured on Git 2.56.1).
  A commit name is never passed here:
  walking a commit also walks its history,
  which no question asks.
- `object_missing`:
  `git cat-file -e <name>`,
  whose status 1 is an object Git cannot find and whose status 128 is a name it cannot read,
  for the commit object of a request Git built.

A Git that cannot be started answers neither way,
and the caller keeps its old answer rather than guessing.
A commit written with an `encoding` header is one `parse_commit` refuses,
as it does on a store the reader opens,
so its reflog row still falls back to the request's own message on `main` and here alike.

## What the refusal costs a files-backend repository

Two parts,
measured separately in paired runs on one files-backend fixture of two commits with loose
objects:

- the `reftable` folder check alone:
  ns per open,
  `main` median 17,814.1 (range 16,475.1 to 19,844.0),
  the folder check 18,273.0 (16,877.0 to 20,803.4),
  per-round ratio median 1.045 (0.986 to 1.139),
  in 10 rounds of 50,000 opens;
- the folder check and the configuration read:
  ns per open,
  `main` median 16,464.9 (15,847.8 to 17,716.3),
  this branch 24,034.2 (23,522.0 to 25,640.6),
  per-round ratio median 1.463 (1.393 to 1.530),
  in 12 rounds of 50,000 opens.

The identical-binary control is a byte-identical copy of this branch's probe:
its median is 24,553.2 (23,257.1 to 25,521.1),
1.482 of `main` per round (1.410 to 1.557),
and 0.999 of this branch (0.971 to 1.035).
So the run-to-run noise of the harness is about one part in a hundred,
and the two arms are measured against the same rounds.
The folder-only arm is a third build,
so part of the 1.463 is code layout rather than the read:
that arm bounds the layout effect at about 4.5 percent,
and the configuration read is the rest,
about 7.5 microseconds added to a 16.5 microsecond open on this host under load.
A landing opens the database a few times,
so the price of one commit is tens of microseconds against the milliseconds its Git starts cost.

The alternative was tried and did not help:
a case-insensitive byte scan for `refstorage` before the parse measured 1.438 of `main` against
1.463,
inside the noise of the identical pair,
because the file read and its system calls are the cost and not the parse.
It was left out.

## What the refusal costs a reftable repository

The database is declined whole,
because a reader that cannot list a ref store cannot tell a store with replace refs from one
without.
On a reftable repository the landing's object reads therefore ask Git where they used to answer
in process,
and so does the client's own content check:

- one `git cat-file commit` for the reflog row's subject and kind;
- one `git rev-list --objects` over the objects and trees a request names,
  and one `git cat-file -e` for a commit Git built,
  in the landing and again in the client's check when the service answered that it could not
  finish;
- one `git rev-list --parents` when an amend's base is read with the history left to Git
  (the first round's recorded amend spent 3 ms in `reads_parents_ms`).

The index reset,
the LFS plan,
the push trigger and the query context pay nothing new:
the layout already refused the repository.
The clean check and the ladder target stop reading the database on a reftable repository,
so every answer they took from it now comes from the refusal their caller handles,
and that refusal is the one `main` already gave there,
because `head_commit` refuses first.
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
The files form of `extensions.refStorage` (`files://<path>`) was set up by hand but Git could not
resolve `HEAD~1` in that fixture,
so no wrong read is established for it.
The cost of the Git starts a reftable repository now pays was not timed in this round;
only the first round's single `rev-list --parents` start was,
at 3 ms.
