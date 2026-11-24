# Owner instructions for the nights away of 2026 11 11 and 2026 11 12

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 29295 to 29298 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

#### Decided while the owner is away, each open to veto

The owner,
about 20:03:
"Make decisions I'd definitely approve (based on my apparent taste) yourself."

From lines 29683 to 29687 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### The owner is away for eight hours, and the first commits on 0.6.1-alpha.11 (2026-11-11T20:05:00-05:00)

The owner,
at about 20:03:
"I'll be away for 8h,
you keep working.
Make decisions I'd definitely approve (based on my apparent taste) yourself."
And,
on reading that the first commit on alpha.11 spent 17.6 s starting the new release's commit service:
"Also,
I believe stopping the old commit service can be made async."

From lines 30299 to 30308 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### Superseded work is culled every hour (first pass 2026-11-11T18:03:00-05:00)

The owner,
at about 18:00:
"Please take a look at in flight and queued work now and cull or reframe anything that's superseded.
In fact,
make this an hourly cron lasting for 12h."
Cron `4c1ff2ae` fires hourly until 2026-11-12T06:40-05:00 (`c921d8d2` deletes it then);
both live only in the coordinator's session,
so a restart loses them and they are made again.
Each pass looks at running `pi-*` units,
`launch-*` batch units,
retry loops waiting for a slot and briefs not started,
against owner decisions and landed work since those runs were briefed,
and records here what it stopped or reframed and why.

From lines 30317 to 30328 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- Superseded,
  nothing to stop:
  `fix/git-policy-cli-open-issues-next-batch` was ported as `fix/git-policy-cli-open-issues-fourth-pass` (merged),
  and the review starter refused it on its own ("nothing to review").
  The coordinator first reported it as "deletable when you allow it";
  the owner,
  at about 18:08:
  "uh,
  I said to auto delete."
  So a superseded branch is deleted by the coordinator without asking,
  once its content is shown to live elsewhere (the earlier note that branch deletion needs permission each time was wrong).
  Asked whether to write that into `AGENTS.md`,
  the owner answered "Add the rule,
  and make it cover not just branches.",
  and rule `XSU` now covers anything superseded (branches,
  worktrees,
  units,
  briefs,
  scratch).
  This one's four commits were all documentation,
  its planning file `doc/planning/git-wrapper-open-issues-round-4.md` is in the integration in a newer form (`7f84c90b1b`),
  no worktree held it,
  and it was deleted here and on the remote at about 18:10.
