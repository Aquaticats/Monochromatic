# An unreachable Git directory keeps its push plans until the filesystem proves the repository is gone

Date:
 2026-11-10.
Subject:
 an unreachable Git directory keeps its push plans until the filesystem proves the repository is gone.
Owner's rule this serves:
 "nothing should ever wait forever or indefinitely".

## What was reported

A commit service retried the pushes of `refs/heads/main` for two repositories that no longer existed,
every 600 s for hours,
reaching try 48 and climbing:
`/tmp/lfs-login-proof-ikmLJW/repository-a/.git` and
`/tmp/lfs-login-proof-ikmLJW/repository-b/.git`.
Both were throwaway repositories another agent ran with the installed wrapper and then deleted.

The journal line was:

```text
push will be tried again branch="refs/heads/main" oid=... next_try=48 in_seconds=600 cause: the repository at /tmp/lfs-login-proof-ikmLJW/repository-a/.git is not reachable now (is its drive mounted?)
```

## Why it happened

A drain whose Git directory cannot be resolved keeps every plan it has,
because that state covered two cases the service could not tell apart
(`landing/service_work_handlers_replanning_and_acceptance.rs`,
`keep_plans_while_unreachable`):

- a drive that is not mounted yet,
  whose repository returns when it is mounted,
  which the owner wants kept:
  the plan is what tries the push once the drive is back;
- a repository that was deleted,
  whose plans can never run and whose work records went with the directory.

The second case produced a journal line every ten minutes,
forever,
for a path nothing could reach again.

## The rule now

The proof is asked only after the directory failed to resolve,
so a reachable repository pays nothing for it
(`landing/unreachable_git_dir_proven_gone.rs`).
It walks up from the Git directory to the nearest directory that does exist,
and reads that directory three ways:

- it is the root of a mounted filesystem,
  so every name below it would live on a filesystem that is mounted and answering,
  and only someone writing the repository again brings it back:
  the repository is gone;
- it is a directory that holds other entries,
  so the repository was one entry among them and was deleted:
  the repository is gone;
- it holds nothing at all,
  which is what a mount point looks like while its drive is not mounted:
  something may still return,
  so the plan is kept,
  exactly as before.

The mount table the first reading uses is read when the question is asked,
never kept for the life of the process:
a drive mounted between two checks of a serving service changes what its root proves,
and a table kept from the first check kept the service of a deleted repository serving.

A repository that is gone ends its plans with one journal line naming the path,
the directory the proof looked at,
what proved it,
and the branches and commits that lost their tries.
The person's own notice cannot be written:
it lives inside the Git directory,
which is gone with the repository.

## The limit of the proof

Four states can still be read as something else,
and each was chosen so that the mistake loses nothing:

- a repository deleted under a directory that is left empty reads as a directory waiting for a drive,
  so its plans are kept;
- a mount point whose directory does not exist reads as gone,
  so its plans end,
  and a commit made there again is pushed as any other;
- a path the filesystem will not answer about proves nothing,
  so a repository behind a directory that refuses the search,
  or an empty directory whose entries cannot be listed,
  keeps its plans until a question can be answered,
  which costs nothing but a retry an unmounted drive also spends;
- a path under a name that is not a directory cannot be reached,
  and is read as one that may return,
  so a plan of a journal that nothing could have written is kept.

## What was verified

`landing/unreachable_git_dir_proven_gone_tests.rs` has a control for each side:

- `a_deleted_repository_ends_its_plans` fails on the release before this rule,
  with the plan the incident kept,
  and passes after it;
- `a_path_behind_an_unmounted_mount_point_keeps_its_plans` keeps its plan on both releases,
  which is the behaviour the owner asked to preserve.

The controls of the proof itself are
`a_deleted_repository_is_proven_gone`,
`a_path_behind_an_unmounted_mount_point_may_return`,
`the_root_of_a_mounted_filesystem_proves_the_repository_gone` and
`the_line_names_the_path_the_proof_and_the_branches`.

## The same proof decides the units of the gone worktree

The units of the same deleted repositories kept a socket listening and a service retrying,
because the proof was asked about the plans alone.
Since 2026-11-11 the units and the sweep ask it too
(`doc/troubleshooting/git-policy-cli-service-units-of-deleted-worktrees.md`),
so a worktree whose repository this proof says is gone loses its socket,
and the plans of that repository end with one journal line instead of another try.

## A repository that is back before the change lands keeps its plans

The proof is a point in time,
and the drain does not apply what it asks:
the change a gone repository produces is applied by the service loop after the drain returns
(`push_retry.rs`,
`Retrying::apply`).
A repository renamed away and back in that gap,
or one whose drive comes back then,
reads as gone when it was looked at and is reachable when the plan would be dropped.

The forget a gone repository produces therefore carries `only_while_unreachable`,
and the applier asks the filesystem once more before it drops the plan:
a Git directory that names a directory now keeps its plan,
and one that is still missing ends it as before.
Every other forget (a push that landed,
one that needs the person,
one whose record is no longer acceptable)
is a decision about the work and not a reading of the filesystem,
so it is applied whatever the path shows.

Control:
`a_repository_renamed_back_before_the_change_lands_keeps_its_plans`
in `landing/unreachable_git_dir_proven_gone_tests.rs` fails before this change
with the plan removed after the repository returned,
and passes after it.

What this does not repair:
a repository renamed away and back long after the change was applied is still read as gone when the change lands,
and its plan ends with the journal line.
Nothing on the filesystem tells a deleted repository from a renamed one,
so that case is one of the proof's limits rather than a state the applier can see.
The commit stays local in that case,
and the journal line names the path whose plan ended.

## The review of this reading

A review of the change that made this proof decide the units of a gone worktree too
(2026-11-11,
`doc/troubleshooting/git-policy-cli-service-units-of-deleted-worktrees.md`)
reported two defects in the proof,
and both held:

- every metadata error was read as absence,
  so a live repository behind a directory that refused the search lost its retry plan.
  Only `NotFound` says nothing is there now;
  every other answer proves nothing,
  so the plan stays.
  Controls:
  `a_plan_of_a_repository_behind_a_directory_that_cannot_be_searched_stays`,
  `a_path_behind_a_directory_that_cannot_be_searched_may_return` and
  `a_path_behind_a_directory_whose_entries_cannot_be_listed_may_return`.
- the mount table was kept for the life of the process,
  so a drive mounted while a service ran was never seen.
  Every question reads the table now.
  Control:
  `the_mount_table_is_read_again_after_a_drive_is_mounted`.

The review's third finding,
that the cleanup of a gone repository deleted a valid plan it could not read,
does not hold on this branch:
`load` has kept an unreadable plan since `7d0e2f51bd`,
and `a_valid_plan_that_cannot_be_read_stays_when_the_gone_plans_end`
holds that at the cleanup.
