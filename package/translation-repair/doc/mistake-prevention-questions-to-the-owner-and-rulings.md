# Preventing mistakes: questions to the owner and rulings

Rules for what is put to the owner and how an answer is read:
questions that are measured before they are asked,
choices decided for quality,
rulings kept in the design commitments,
and the work a ruling reaches.

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.
It held the section of the same name in
[Preventing mistakes: shell commands, edits, commits and agents](mistake-prevention-shell-edits-commits-and-agents.md)
until 2026-10-29,
when that file neared the 1,000-line limit.

## Questions to the owner

What happened:
an adopted reading parked in docs across a compaction (M7);
a measurable question asked instead of measured (M8);
a fix started against a ruling (M15);
quality calls put to the owner as design questions (M30).
Later rounds took the alternatives of recorded vetoes,
and the page of veto choices for the owner,
which collects paragraphs by a marker phrase,
did not read their closing notes,
so the old veto paragraphs stayed on that page as live cards,
asking the owner to keep or veto behaviour that no longer existed (ledger M146).
A question described an idiom's sense that three of its four occurrences in the corpus contradict,
and the owner chose an option on that description (ledger M148).

The rule:
measure what can be measured before asking.
A sense,
count or reading an option rests on is measured over the pinned corpus before the question is asked,
every occurrence read.
A choice that differs only in the quality of the result is decided for quality,
built,
recorded,
and open to veto.
A ruling goes into [the design commitments](design-commitments.md) the turn it is given,
and a finding is checked against them before any fix starts.
When a later round takes the alternative of a recorded veto,
the lead writes the closing note,
lists the item ids the page of veto choices holds for the old paragraph,
and turns each reversed choice into a pointer on the page,
or removes it,
before the page is rebuilt,
then drives the page again.
The owner ruled on 2026-10-10 (UTC) that a choice left unmarked on the page of veto choices counts as kept.
A mise task named in a question to the owner is quoted with its `description` line,
read from its `mise.toml` in the same turn,
so the approval names the command that runs (ledger M277).

What enforces it:
habit,
and the commitments page.

## Rulings read for the work they reach

What happened:
on 2026-10-29 (UTC) the owner answered where finished rounds should land with "Branch only",
and added that merges between the branch and main are made only when the owner says.
The lead read the answer as reaching rounds that had already merged main,
measured the main commits each carried,
planned to redo each round,
and stopped a running gate and a running sweep.
The owner said the rounds were not unmergeable,
that the answer applied going forward,
and that no work was to be abandoned.
The plan was dropped and the gate and the sweep were started again (ledger M320).

The rule:
a ruling applies from the turn it is given.
Work already done and work already running stay as they are
until the owner says that the ruling reaches them.
A reading that would stop a run,
discard finished work or redo it is put to the owner in the same turn,
with the work it would stop or discard named,
and nothing is stopped before the answer.
When a ruling can be read both ways,
the narrower reading is carried out and the wider one is asked.

What enforces it:
habit.
