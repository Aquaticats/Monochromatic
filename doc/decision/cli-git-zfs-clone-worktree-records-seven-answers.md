# cli-git ZFS clone worktree records: seven answers on leftovers, traces, unreadable records and lost clones

## Status

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T21:16:30-05:00.
It answers the one set of questions the gate review drew from three designs,
each on its own branch and still unapproved:
the add's binding record (`doc/planning/cli-git-zfs-add-binding-record-design.md`),
the record that carries origin cleanup (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`),
and the removal's destruction plan (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`).
The questions are in the review's section "Check 7",
`~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review-third-check.md`.
The sixth is also the cancelled-removal design's open question,
and the seventh the forced-removal design's.

Each answer is given as the option's label and its own description,
as the project was shown them.

## What was decided

### Which commands may destroy a leftover origin snapshot

Asked:
when a clone worktree is removed,
the snapshot of the main checkout it was cloned from must be destroyed too;
if something blocks that,
a record keeps the cleanup pending;
once the block is gone,
which commands may destroy the leftover snapshot.
The project chose "B:
worktree commands only (Recommended)":
"Only `git worktree add`,
`remove`,
`list`,
`prune` and `repair` destroy it,
printing one line;
`doctor` stays read-only and prints the `zfs destroy` command."
The cost stated with it:
a repository that never runs one of those keeps the snapshot and its space.

### What trace of a destruction outlives the removal

The project chose "A:
the printed report only (Recommended)":
"Nothing beyond what the removal printed,
and its records while they stand."
The cost stated with it:
a person who missed the line has,
a week later,
nothing that answers what went.

### How often a pending origin cleanup speaks

Asked with the note that the same rule would decide how often a refusal by the route's compatibility check prints.
The project chose "A:
on a change only (Recommended)":
"One line when it destroys,
or when the reason it is blocked changes;
silence while nothing changed."

### A record that cannot be read

Asked:
a record cut short by a power cut,
or holding a line a later release wrote,
may be the only evidence of a destroyed snapshot or of a snapshot still pinned.
The project chose "A:
refuse and name the file (Recommended)":
"Change nothing,
name the file,
and wait for a person."
The cost stated with it:
the add or removal at that worktree waits,
even when the record merely came from a newer release.

### A slot whose recorded clone is gone and cannot be proved gone

The project chose "B:
offer a settling command":
"Print an explicit command that removes the record after you confirm."
The recommended option was to keep the slot blocked and report;
the project did not take it.
The cost stated with the chosen option:
a command that removes evidence is one typo from a clone nobody then cleans up.
The question's example named the command
`git-policy-cli settle-worktree-clone-record add-cache-layer`;
the name and the confirmation it asks for are the design's to settle.

### How long the plan a cancellation keeps stays

Asked:
a removal that cancels itself after it destroyed some of the clone's snapshots keeps its plan,
the only record of what was lost.
The project chose "A:
until that clone's next removal (Recommended)":
"The next removal of that clone reads the record,
proves it,
carries the loss forward and ends it."
The cost stated with it:
a clone that is never removed keeps its record for ever.

### Whether a forced removal records who it ended, beyond the journal

The project chose "A:
the journal alone (Recommended)":
"The handler's journal lines are the record."
The cost stated with it:
the journal may rotate away,
and the removal's own record does not show it.

## What was not chosen

- Any wrapped Git command,
  only by hand,
  and a user timer,
  for the leftover snapshot.
- An audit-log line,
  a notice on later commands,
  and a history `doctor` prints,
  for the trace.
- A line on every attempt,
  and nothing outside `doctor`,
  for the pending cleanup.
- Moving an unreadable record aside,
  and dropping it after one line.
- Keeping a slot blocked with no way to settle it.
- Keeping the plan until a person acknowledges it,
  and for a fixed time.
- The handler's receipt naming the processes,
  a count in the plan record,
  and a record of its own.

## A further answer: the settling command and a plan a cancellation kept

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T22:03:25-05:00,
on the one question the fourth check of the three record designs left for the project
(`~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review-fourth-check.md`,
"Question:
may the settling command also remove a plan the clone's next removal would have ended?").

Asked:
two answers of this record meet in a case neither covers.
A plan a cancellation keeps stays until that clone's next removal;
a slot whose clone is gone gets the settling command;
and when the clone is gone no next removal can come.
The example put:
a removal of `fix-flaky-test` destroyed one snapshot,
was cancelled and kept its plan;
later someone destroys the clone by hand with `zfs destroy -r`,
and the settling command is run on that slot.

The project chose "A:
remove it,
printing what it held (Recommended)":
"After you confirm,
the command prints the snapshot names the plan recorded as destroyed,
then removes the plan with the other records and frees the slot."
The cost stated with it:
the only durable record of what the cancelled removal destroyed is deleted by one confirmed command.

Not chosen:
freeing the slot and leaving the plan,
named by `doctor`;
and refusing while a plan stands.

## What this record does not decide

The settling command's name,
its guard and what it proves before it removes a record.
Every choice the three designs list for the project's veto.
