# The wrapper's kept references grew without end, and the pruning that bounds them

What this record answers:
which commands wrote the references the wrapper keeps built commits on,
what one reading of this repository on 2026-11-12 found,
the rules and the values the wrapper prunes them by,
where the record of a pruning is,
and how a person reads the state themselves.
The code is in `package/git-policy/cli/src/native/kept_references/`.

## What the references are, and who wrote them

A commit the wrapper builds and cannot put on a branch is kept on
`refs/cli-git/kept/<batch>` of the common Git directory,
with `refs/cli-git/kept/<batch>-rescue` beside it when that name could not be written.
The writers are:

- `src/native/worktree/worktree_removal_held_commits.rs`,
  before a worktree is removed while its commit is still held:
  the removal deletes the worktree's own Git directory and the per-worktree reference
  `refs/worktree/cli-git-service/<batch>` with it,
  so the commit is kept where a removal cannot reach it.
- `src/native/landing/service_land_kept_commit.rs`,
  for a batch the branch does not show.
- `src/native/landing/service_land_git_steps_and_import_stream.rs`
  (`keep_unlanded`,
  `keep_on_a_side_ref`,
  `keep_on_a_side_ref_only_if_free`),
  for a commit a landing could not land,
  and for the name beside a batch's own.
- `src/native/landing/service_scratch_ref_of_a_dropped_batch.rs`,
  which writes `refs/cli-git/kept/<batch>` directly when a batch will not land.
- `src/native/merge/merge_isolation_apply.rs`,
  for the merge commit an isolated merge saves.

Nothing removed any of them,
so the objects only they reach were pinned for the life of the repository.
The owner answered on 2026-11-12
("Keep the reference,
auto prune by size and age",
`doc/handover/cli-git-rust-implementation.md`,
the answer of 2026-11-12T11:58:45-05:00).

What else lives under `refs/cli-git/` is not touched by the pruning:

- `refs/cli-git/carried/<n>`:
  the carried work of an isolated merge
  (`src/native/merge/merge_isolation_recovery_tests.rs`,
  `keep_on_a_side_ref`).
- `refs/cli-git/auto-push/fetch-<pid>`:
  the private fetch of an automatic push
  (`src/native/push/auto_push_reconcile_basics_and_limits.rs`,
  `PRIVATE_FETCH_PREFIX`).
- `refs/cli-git/trash`:
  only the reference a trash's work record carries;
  no reference of that name is created.

`refs/cli-git/kept/` is the one folder the pruning removes a reference from,
and only a name that is a number,
or a number and `-rescue`,
is a kept reference.

## The reading of this repository on 2026-11-12

Read-only,
in the repository at `/var/mnt/encrypted/repository/Monochromatic`:

```console
$ git for-each-ref refs/cli-git/ --format='%(refname) %(objectname) %(creatordate:iso-strict)'
refs/cli-git/kept/1794071191963902 deb10b333be2e7c350d4d5cce35b8d474161fbbc 2026-11-07T13:01:46-05:00
refs/cli-git/kept/1794071191963903 4abc7d6a39b8f76b741705ae772910419412a60e 2026-11-07T13:06:41-05:00
refs/cli-git/kept/1794071191963904 3889918ba5620877443cd2f815a215b95fb8be8d 2026-11-07T13:15:07-05:00
refs/cli-git/kept/1794071191963905 833f0338f0b92a52187ff7e9ea69349e4eb768a6 2026-11-07T13:36:36-05:00
refs/cli-git/kept/1794071191963906 901d9a6a8e8ef5e9da461d6211e3b91c150b2215 2026-11-07T13:48:16-05:00
```

Five kept references,
all of them the batch's own name and none a `-rescue` one,
written over 46 minutes of 2026-11-07,
so the oldest was 6.5 days old when this was read and the newest 5.7.

```console
$ for t in $(git for-each-ref --format='%(objectname)' refs/cli-git/kept/); do git rev-list --count "$t" --not --branches --remotes --tags; done
1
1
1
1
1
```

Each commit is one commit no branch,
remote-tracking reference or tag reaches:
none of the five had landed.

```console
$ git for-each-ref --format='%(objectname)' refs/cli-git/kept/ \
  | git rev-list --objects --stdin --not --exclude='refs/cli-git/kept/*' --all | wc -l
27
```

Twenty-seven objects are reachable only through kept references:
5 commits,
13 trees and 9 blobs.

```console
$ git for-each-ref --format='%(objectname)' refs/cli-git/kept/ \
  | git rev-list --objects --stdin --not --exclude='refs/cli-git/kept/*' --all \
  | awk '{print $1}' | git cat-file --batch-check='%(objecttype) %(objectsize) %(objectsize:disk)'
```

summed:
7,573 bytes on disk and 258,055 bytes before compression,
so one kept reference cost about 1,515 bytes on disk here,
and the largest single object was a blob of 2,023 bytes on disk.

The rate is five references in 5.7 days,
about one every 1.2 days,
at about 1.3 KiB of objects a day.

## The rules the pruning follows

In order,
for each reference under `refs/cli-git/kept/` that is a number or a number and `-rescue`:

1. A local branch,
   a remote-tracking branch or a tag reaches the commit:
   the reference goes at once,
   whatever its age.
2. The reference is older than the age limit and no branch reaches its commit:
   it goes.
3. The objects reachable only through kept references weigh more than the size limit:
   the oldest references go first,
   one at a time,
   each removal followed by a fresh reading of what the survivors reach,
   and no reference younger than the floor goes for its size.

Only a branch,
a remote-tracking branch and a tag settle a commit.
Every other reference is a holder in flight or a holder of its own:
the landing's `refs/worktree/cli-git-service/<batch>`,
the carried work of an isolated merge,
a private fetch of a push,
a person's stash.
Each of those goes away by itself,
so a kept reference removed because one of them reached the commit would lose the commit when that
holder went,
and the pass leaves it to the age rule instead.
The question is asked again immediately before a reference goes for that reason,
because a branch can move off the commit between the listing and the removal while the reference's
own value stays the same.

A pass plans at most sixteen removals and stops after twenty seconds,
leaving the rest to the next pass,
which runs in the commit service's after-commit upkeep after the next landed commit.
`git monochromatic-git-policy-cli kept-references --prune` runs one pass at once,
and `git monochromatic-git-policy-cli kept-references` lists what is kept and what the next pass
takes,
because it is the same judgement with the removals left out.

The size is measured over the objects reachable from a kept reference and from no reference outside
the kept folder,
in bytes on disk (`%(objectsize:disk)`),
because that is what the repository gives back when the reference goes.

## The values, and what they rest on

- `KEPT_REFERENCE_MAX_AGE`:
  thirty days.
  The oldest reference this repository ever reached was 6.5 days old and still unreached,
  so thirty days is about five times that,
  and a person has a month to recover a commit.
  At this repository's rate it bounds the folder at about 25 references;
  ten times the activity bounds it at about 250 and changes no age.
- `KEPT_REFERENCE_MAX_BYTES`:
  64 MiB.
  One reference cost 1,515 bytes on disk here,
  so 64 MiB is about 43,000 of them,
  and the age limit binds long before that at this weight.
  The size rule answers a kept commit that carries large content instead:
  this repository's five carry no object over 2 KiB,
  while a kept commit of a worktree holding an artifact can carry megabytes,
  and ten of 6 MiB each bring the limit to its bound.
- `KEPT_REFERENCE_FLOOR`:
  one hour.
  A kept reference is written at the moment a worktree goes,
  so a burst of removals writes a burst of references.
  The cleanup of 68 unused worktrees on 2026-11-12 ran from 11:38 to before 11:49,
  so a burst takes tens of minutes and an hour covers several of them,
  which is what keeps a commit kept at the start of one out of the hands of the end of it.

A run changes any of them with
`CLI_GIT_KEPT_REFERENCE_MAX_AGE_SECONDS`,
`CLI_GIT_KEPT_REFERENCE_MAX_BYTES` and
`CLI_GIT_KEPT_REFERENCE_FLOOR_SECONDS`;
a value that is not a positive whole number keeps the shipped one,
so a typo never removes a bound.
Nothing in `doc/decision/` names these values:
they are put to the owner before a release carries them.

## Where the record of a pruning is

`<common Git directory>/monochromatic-git-policy-cli-kept-references.jsonl`,
one line per pruned reference,
written through the appender every log of the wrapper uses
(`log_lines_appended_without_a_lock.rs`),
so two wrappers pruning at once lose no line,
and rotated by the bound every log shares (`LOG_HISTORY_BYTES`).
A line is the time,
the reference,
the commit,
the batch,
whether it was the name beside the batch's own,
the age,
the worktree the commit was built in,
and the reason:

```json
{"outcome":"pruned","time_ms":1794513038147,"time":"2026-11-12T18:30:38.147Z","reference":"refs/cli-git/kept/1794071191963902","commit":"deb10b333be2e7c350d4d5cce35b8d474161fbbc","batch":1794071191963902,"rescued":false,"age_seconds":5434600,"kept_at":"2026-11-07T18:26:31.963Z","worktree":"/home/a person/worktrees/task","why":"too-old","why_words":"it reached no branch and was older than the age limit of 30 days"}
```

The line goes before the reference does,
because the reference was the only handle on the commit:
a crash between the two must leave the account of what went.
A removal Git then refuses,
because another pass took the reference first or another command moved it,
adds a second line about the same reference with `"outcome":"refused"`,
so a person reading both knows the removal did not happen.

The time a reference was written is kept beside it while it exists,
one small file per reference in
`<common Git directory>/monochromatic-git-policy-cli/kept-references/`,
because a reference carries no time of its own:
the wrapper writes no reflog for one.
A reference a release before this one wrote has no such record,
and the first pass that lists it writes one dated at that moment,
so it is never pruned before it has been listed for the whole age limit.
A reference a worktree removal wrote carries the worktree's path,
which is the only moment the two are known together.

## Reading the state yourself

```sh
git for-each-ref refs/cli-git/kept/ --format='%(refname) %(objectname)'
git monochromatic-git-policy-cli kept-references
ls .git/monochromatic-git-policy-cli/kept-references/
tail -n 5 .git/monochromatic-git-policy-cli-kept-references.jsonl
```

A reference removed by hand is one the pass stops answering for;
its record goes with it at the next pass,
and nothing else needs clearing.

## What a merge with the landing branch needs

The branch
`fix/git-policy-cli-landing-keeps-the-built-commit-when-a-keep-or-a-lock-is-refused`
moves the landing's side-reference write out of
`service_land_git_steps_and_import_stream.rs` into
`service_land_side_reference_write.rs`,
where `keep_on_a_side_ref(git, repository, name, tip)` replaces
`keep_on_a_side_ref_only_if_free(git, repository, name, tip)`.
The pruning names the two forms of a kept reference itself
(`kept_reference_name.rs`),
so it answers for both a merge produces.

`git merge-tree --write-tree --name-only` between this branch and that one reports two conflicts,
both of them the one name that moved:

- `src/native/worktree/worktree_removal_held_commits.rs`,
  where this branch adds `remember_the_worktree` and its two call sites beside the import that branch
  renames.
  The merge keeps that branch's import
  (`crate::service_land_side_reference_write::keep_on_a_side_ref`)
  and this branch's helper and calls.
- `src/native/worktree/worktree_removal_held_commits_tests.rs`,
  where the same rename reaches one call.

Everything else merges cleanly.

One line belongs after the merge rather than before it:
`service_land_side_reference_write.rs` is where a landing's side-reference write lives after that
branch,
and the same `kept_reference_record::record_a_removals_kept_reference` call belongs after a
successful write there,
so a commit a landing kept also names the worktree it was built in.
This branch records the worktree only where a removal keeps a held commit
(`worktree_removal_held_commits.rs`).
Without that call,
a reference a landing kept is dated by the first pass that lists it and its line says the worktree
was not recorded,
which loses no commit.
