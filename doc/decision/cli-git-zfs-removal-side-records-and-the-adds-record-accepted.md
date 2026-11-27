# ZFS route: the add's record, the origin cleanup record and the destruction plan are accepted

## Status

Accepted by the project in the question tool on 2026-11-23.
The designs are on `main`:
`doc/planning/cli-git-zfs-add-binding-record-design.md`,
`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`
and `doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`.
The question's full text,
with all 37 choices,
their examples and the 14 entries,
is the record reviewer's eleventh read,
`~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review-eleventh-read-and-the-records-question.md`
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here),
"The question".
That text put each of the 37 choices with what it decides,
its options best first by what each gives the person who uses the worktrees,
the cost of each,
and one example of what the person sees;
its 14 entries each name a sentence of the two designs
and the words that would replace it.
The add's record's step 10 parts were accepted earlier the same day
(`doc/decision/cli-git-zfs-step-10-add-time-lock-and-binding-record.md`).

## The three records

The project chose "Accept the three (Recommended)":
"All 37 choices stand as adopted,
except whatever you pick in the next two questions.
Cost:
unmeasured limits are accepted as provisional."
The question said that nothing is measured yet,
and that a measurement that shows a limit wrong reopens its choice;
and that the cancelled-removal and forced-removal designs the records point to are not accepted by this.
Not chosen:
"Accept,
with vetoes",
and "Not yet".

## Choice 2: one line format for all three records

The project chose "One key=value format (Recommended)":
"All three records use plain key=value lines.
Cost:
the origin record's slot path shows only as hex."
This replaces the origin cleanup record's adopted short lines and one JSON line.
Not chosen:
"Keep the JSON line (adopted)".

## Choice 32: how a plan records each snapshot's history

The project chose "One line per attempt (Recommended)":
"The whole history stays readable after a later success.
Cost:
the record grows with each retry,
and the build needs a merge rule and a bound counted in attempts."
This replaces the plan design's adopted one line per snapshot.
Not chosen:
"One line per snapshot (adopted)".

## Edits to two accepted designs

The project chose both options offered:
"Edit all 14 (Recommended)":
"Each sentence takes the reviewer's 'would read' words
(listed in the review file's 'Sentences of two accepted designs' section).
Cost:
14 edits to text you accepted."
and "Add the no-clone clause (Recommended)":
"One clause in the dirty-main design for a fallback that entered at `registered` and made no clone.
Cost:
wording not yet drafted;
a round writes it and you see it in the diff."
The two designs are `doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`
and `doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`.
The two entries that spell the clean-up command's name follow choice 6,
which stands as adopted.
