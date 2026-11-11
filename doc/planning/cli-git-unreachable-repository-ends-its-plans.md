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

A repository that is gone ends its plans with one journal line naming the path,
the directory the proof looked at,
what proved it,
and the branches and commits that lost their tries.
The person's own notice cannot be written:
it lives inside the Git directory,
which is gone with the repository.

## The limit of the proof

Two states can still be confused,
and both were chosen so that the mistake loses nothing:

- a repository deleted under a directory that is left empty reads as a directory waiting for a drive,
  so its plans are kept;
- a mount point whose directory does not exist reads as gone,
  so its plans end,
  and a commit made there again is pushed as any other.

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

## A repository that is back before the change lands keeps its plans

The proof is a point in time,
and the drain does not apply what it asks:
the change a gone repository produces is applied by the service loop after the drain returns
(`push_retry.rs`,
`PushRetries::apply`).
A repository renamed away and back in that gap,
or one whose drive comes back then,
reads as gone when it was looked at and is reachable when the plan would be dropped.

The forget a gone repository produces therefore carries `only_while_unreachable`,
and the applier asks the filesystem once more before it drops the plan:
a Git directory that names a directory now is not dropped on the stale proof,
and one that is still missing ends its plan as before.
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

### The plan the round had already taken out

The first release of this rule skipped the forget and kept the branch's plan on disk,
and the running service still had no try for it.
The round of the service's work takes what is due out of the plan before it drains the directory and hands
it to the worker (`landing/service_work_run_round.rs`,
`plan_the_round`),
so the plan the gone repository's drain ends is one the schedule no longer holds:
skipping the forget kept a file and nothing else,
`next_wait` answered nothing,
and the promised push waited for a service restart to read the plan from disk again.

Cross-vendor review of 2026-11-11,
round 174,
finding 3.
The forget a gone repository produces now carries the plan its reading ended,
and the applier puts that plan back when the directory is there again:
the try the plan was for never ran,
so it is due at once,
and the running service drains the repository as soon as the plan says so.
The control above models that `take_due` before it drains,
which is what the first version of it did not do:
a control that schedules a plan of its own hides the loss,
because nothing removes the plan the service holds.

What this does not repair:
a repository renamed away and back long after the change was applied is still read as gone when the change lands,
and its plan ends with the journal line.
Nothing on the filesystem tells a deleted repository from a renamed one,
so that case is one of the proof's limits rather than a state the applier can see.
The commit stays local in that case,
and the journal line names the path whose plan ended.

### The file the worker removes and the plan the loop puts back

The file of the forgotten plan goes where every forget's file goes:
the worker that decided the branch needs no new try removes it,
and for this kind of forget it removes it only while the directory still fails a directory check
(`landing/service_work_handlers_replanning_and_acceptance.rs`,
`forget_when_unreachable`).
So a repository renamed back before the worker ran keeps its file as well as the plan the applier puts back,
and one that reads as gone on the worker's own reading loses the file there;
the applier's later reading is what decides about the list,
and a plan it puts back is written to the journal again by the next drain that decides the push needs a
new try (`Replanning::keep_the_plan`).
