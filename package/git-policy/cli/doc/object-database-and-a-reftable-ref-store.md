# The object database reader and a reftable ref store

**Unapproved and non-final.**
This note records what the in-process object database answered on a reftable repository,
which shipped callers reached it,
what each asks Git when it declines,
the refusal that now stands in `repo_read_odb.rs`,
and what the refusal costs.
The first round was measured on this host on 2026-11-21 and the second on 2026-11-22.
Both rounds' sources and findings are on `main` from merge `96d965f9f8`,
in `package/git-policy/cli/src/native/repo_read/repo_read_odb.rs`,
`package/git-policy/cli/src/native/landing/service_object_questions_left_to_git.rs`
and this note.
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

`objects_are_replaced` returns `true` when `GIT_REFERENCE_BACKEND` names any store but exactly `files`,
when `reftable` exists in the common directory,
and otherwise reads the repository's own `config` and declines
`extensions.refStorage` with any value but exactly `files`
(`repo_read_odb.rs`,
`ref_store_is_reftable`).
The environment is read because it overrides the configuration on every Git start,
so the configuration alone cannot say where the refs live (see the section below).
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
A repository whose reader now declines where `main` opened it keeps its answer,
because the caller asks Git;
that is the safe direction,
at the price of Git starts.
The declines this round added are:

- a stray `reftable` folder in a files repository,
and a linked worktree whose common directory holds one;
- a configuration that cannot be parsed,
or one larger than one mebibyte (`CONFIG_TEXT_LIMIT`):
  the read refuses rather than guesses;
- `extensions.refStorage` spelled `Files`,
which Git refuses as an unknown format;
- `extensions.refStorage` set to `reftable` with no `reftable` folder behind it;
- a `reftable://` or `files://` payload that puts the store outside the common directory,
  including a reftable folder moved away and a linked worktree of that repository;
- `GIT_REFERENCE_BACKEND` naming anything but exactly `files`.

A person who wonders why one of these repositories now starts Git where it did not before
finds the rule in this section and in the package's `README.md`,
"Reading the repository in process",
where the list of questions left to Git names the same cases.
The cost is one Git start per declined question,
never a wrong answer.

The controls in the default build:

- `repo_read_odb::tests::a_reftable_repositorys_replace_ref_declines_the_database`:
  a reftable repository with a replace ref declines the database.
- `repo_read_odb::tests::a_ref_store_the_configuration_moves_away_declines_the_database`:
  the same repository with its tables moved out and `extensions.refstorage` naming them declines,
  while Git still reads the replacement there,
  and a files repository whose configuration names `files` still opens.
- `repo_read_odb::tests::a_files_store_the_configuration_moves_away_declines_the_database`:
  a files store named as a `files://` payload declines,
  while Git resolves the history and the replacement there.
- `repo_read_odb::tests::an_environment_that_moves_the_ref_store_declines_the_database`:
  a files repository whose replace ref sits in the store `GIT_REFERENCE_BACKEND` names declines the reader,
  and Git reads the replacement there in the control.
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
- The four functions of `service_object_questions_left_to_git.rs`
  (`service_object_questions_left_to_git::tests::`):
  `commit_bytes` answers with the bytes Git reads and with nothing for a name Git has no commit of;
  `objects_missing` sees a blob a pruned subtree names;
  `object_missing` sees a commit Git built whose object is gone;
  `built_commits_missing` asks every commit of a request.
  Each runs on a reftable repository and on a files repository holding a replace ref.
- `service_object_questions_left_to_git::tests::a_landing_reads_a_commit_once`:
  a landing that reads one commit twice starts Git once.
- `service_object_questions_left_to_git::tests::a_request_too_large_for_one_command_line_reaches_git`:
  the names reach Git on standard input at a count derived from the host's own `ARG_MAX`,
  whose bytes could not fit one command line.
- `repo_read_layout::tests::a_ref_store_the_configuration_moves_away_is_left_to_git`:
  a moved reftable store,
a linked worktree of a reftable repository and a files payload are left to Git,
  each with Git's own answer as the control.
- `repo_read_layout::tests::a_shallow_or_grafted_history_keeps_the_layout_and_declines_the_reader`:
  a shallow file and a graft file keep the layout the files backend has and decline the object reader.

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
  with the service's general words in place of the content-pruned ones;
- the four question controls,
with all four questions made to answer nothing,
  failed as the six tests of `service_object_questions_left_to_git::tests::`:
  `commit_bytes_answers_with_the_commit_git_reads`,
  `objects_missing_sees_a_blob_a_subtree_names`,
  `object_missing_sees_a_gone_commit_git_built`,
  `built_commits_missing_asks_every_commit`,
  and the two controls that read them;
- with the names back on the command line,
  `a_request_too_large_for_one_command_line_reaches_git` failed with
  `a request this large still reaches Git`;
- with the landing's memory of a commit it read removed,
  `a_landing_reads_a_commit_once` failed with
  `["cat-file commit <name>", "cat-file commit <name>"]` where one start was expected;
- with the environment check removed,
  `an_environment_that_moves_the_ref_store_declines_the_database` failed with
  `a ref store the environment names outside the files backend is outside what this reader scans`.

## The environment that overrides the configuration

`GIT_REFERENCE_BACKEND` is Git's per-command override of the ref storage format.
Git 2.56.1's `git(1)` says,
under "Git Commits":
"Specify which reference backend to be used along with its URI.
See `extensions.refStorage` option in linkgit:git-config[1] for more details.
Overrides the config variable when used."
It first shipped in Git 2.54.0 (commit `53592d68e868`,
"refs:
add GIT_REFERENCE_BACKEND to specify reference backend",
merged as `1d0a2acb78f1`;
the release notes say "Allow the directory in which reference backends store their data to be specified").
Git 2.56.1 reads it in two places in `setup.c`:
discovery,
lines 2087 to 2100,
and initialization,
lines 2824 to 2836,
each through `parse_reference_uri` (lines 635 to 648),
and its name is defined in `environment.h` line 46.
The later rename to `GIT_REF_STORAGE_FORMAT`,
with the old name kept as an alias,
is commit `768ec528646c` and is not in 2.56.1.

Measured on the host with Git 2.56.1,
on a files repository whose two commits were made before the variable was set:

- `GIT_REFERENCE_BACKEND=files://<store>` with a replacement ref written under `<store>/refs/replace`:
  `git cat-file commit <older>` prints the replacement (the head commit's bytes),
  and the same command without the variable prints the older commit's own bytes;
  `git rev-parse --show-ref-format` answers `files` either way.
- `GIT_REFERENCE_BACKEND=reftable` on a files repository with no reftable store:
  `git rev-parse --show-ref-format` answers `reftable`,
  `git for-each-ref` prints nothing,
  and `git rev-parse HEAD` fails with
  `fatal: ambiguous argument 'HEAD': unknown revision or path not in the working tree`.

The first arm is the wrong answer of issue 6340 in another spelling:
the reader would scan the files under the common directory and answer the older commit,
while Git reads the replacement from the store the environment names.
`repo_read_odb.rs` therefore declines the database when the variable is set to anything but exactly `files`
(`environment_names_another_ref_store`),
and its control is
`repo_read_odb::tests::an_environment_that_moves_the_ref_store_declines_the_database`,
which runs its body in a child process because the variable belongs to one process.
The second arm needs no separate rule:
the same decline covers it,
and Git answers from the store it was told to read.
The landing's `HISTORY_VARIABLES` (`service_built_checks.rs`) does not list the variable:
the reader's decline already sends every commit question to Git,
and `adds_only_present_objects` takes the decline as "not proven" and runs the `rev-list` it always ran,
so naming it there would change no answer.

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
  a landing remembers the bytes of a commit it has already asked Git for
  (`service_object_questions_left_to_git.rs`,
`commit_bytes`),
  so a one-commit batch that reads the same commit for its objects and again for its row starts Git once;
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

## What remembering a commit saves

A landing reads one commit for the objects a batch lands and again for its reflog row.
On a reftable repository,
and on a files repository holding a replace ref,
those were two
`git cat-file commit` starts for the same commit;
the landing now keeps the bytes of the first read and the second asks nothing.

Measured on this host on 2026-11-22,
with three release binaries in their own worktrees:
`main` at `e3c9b7e62d` (which already carries merge `96d965f9f8`),
this branch,
and a byte copy of `main`'s binary as the identical-binary control.
Fixtures of thirty files come from the package's own latency harness (`bin/native-perf-*.ts`),
each round runs the three arms once with the order rotated,
and every run sits in its own capped scope (`CPUQuota=300%`).
Ratios are per round,
then their median and range;
the host carried other sessions' load throughout (rule `NQH`).

- Reftable repository:
  the service's own record gives a median of 19 Git steps per landing on `main` and on the control,
  18 on this branch,
  and the last landing's steps show two `git cat-file` starts on `main` and one here;
  `reads_ms` median 7 on `main`,
7 on the control,
6 here.
  Whole command (about 63 ms):
this branch over `main` median 0.956 (0.199 to 1.945),
  control over `main` 0.974 (0.225 to 1.669),
  this branch over control 0.962 (0.589 to 2.093).
- Files repository holding a replace ref:
  9 steps per landing on `main` and the control,
8 here,
  with the same two-to-one `git cat-file` pair;
  `reads_ms` median 3,
2 and 1.
  Whole command (about 37 ms):
this branch over `main` median 0.970 (0.502 to 1.579),
  control over `main` 0.954 (0.463 to 1.360).
- Files repository with a `.git/shallow` naming the tip:
  9 steps on all three arms and `reads_ms` 3 on all three.
  The one `git cat-file` stays:
  a rewritten history skips `read_commit`,
so the reflog row is the only question and there is no second one to remember.
  Whole command (about 40 ms):
this branch over `main` median 0.982 (0.248 to 2.463),
  control over `main` 0.977 (0.284 to 2.960).

The identical pair's per-round range is wider than the one-step effect in any single round,
so the whole-command ratios are read as medians;
the service's step counts are the measured saving,
one Git start per declined landing where two commits were asked once each.

## Rows the recheck left open

The recheck's "What I could not do or establish" named rows it had read but not run.
Each was run on this host with Git 2.56.1,
and each has a control in the default build:

- The `files://<path>` payload form.
  A files repository initialized under `GIT_REFERENCE_BACKEND=files://<store>`,
  or one whose `.git/refs` is moved under `<store>/refs` and whose configuration names
  `extensions.refStorage = files://<store>`,
  is read by Git:
  `git rev-parse HEAD~1` names the older commit and `git cat-file commit <older>` prints the replacement.
  The layout reader declines it (`repo_read_layout::tests::a_ref_store_the_configuration_moves_away_is_left_to_git`)
  and so does the object reader
  (`repo_read_odb::tests::a_files_store_the_configuration_moves_away_declines_the_database`).
  The first pass's failure to resolve `HEAD~1` came from setting the configuration without putting the
  refs under the payload;
  with the refs there,
Git resolves them.
- A reftable repository whose `reftable` folder was moved away and whose configuration names it:
  `git rev-parse --show-ref-format` answers `reftable`,
  and the layout reader declines it (same control).
- A linked worktree of a reftable repository:
  the linked worktree reads (`git rev-parse HEAD`),
  and the layout reader declines it (same control).
- A shallow row and a graft row:
  a files repository with `.git/shallow` naming the tip,
and one with an empty `.git/info/grafts`,
  keeps the layout the files backend has (the reader's five lines equal Git's),
  and the object reader declines it
  (`repo_read_layout::tests::a_shallow_or_grafted_history_keeps_the_layout_and_declines_the_reader`).

## What was not measured

No guest ran:
every number is this host's ZFS,
under whatever load it carried,
and the arms alternated rather than standing alone.
The gate's own test binaries were not timed.
The per-open microbenchmark of the configuration read was not repeated;
the first round's 7.5 microseconds is used with this round's open counts.
The reftable cost was timed on one fixture of thirty files;
a larger tree adds blobs to the standard input of `objects_missing`,
which is no longer on a command line.
