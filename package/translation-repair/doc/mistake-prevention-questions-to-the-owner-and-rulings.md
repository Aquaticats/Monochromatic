# Preventing mistakes: questions to the owner and rulings

Rules for what is put to the owner and how an answer is read:
questions that are measured before they are asked,
choices decided for quality,
rulings kept in the design commitments,
the work a ruling reaches,
and the guards that stop an entry where a ruling says to ship.

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

## Quoting the owner

What happened:
a sentence reached the briefs,
the ledgers,
the design commitments and five comments in the source
as the owner's own words,
the lead's "No valid wording:
keep the archive at that slice and ship" (2026-09-27).
The owner's recorded answer of that day to the question of a slice with no valid wording is the option "Keep archive,
ship" (the design commitments file records it).
The longer sentence is the lead's paraphrase,
first in git in `1f2bb6aef6` (2026-10-29T15:59Z),
a comment of the code commit that added the interim page,
where the sentence wraps across two lines;
`227d4118b6` (2026-10-29T16:11Z) is the first document to carry it
(this place said `227d4118b6` alone until 2026-11-01 UTC;
ledger B1302).
The lead measured its own session transcript on 2026-11-01 (UTC):
all 52 user-role occurrences of "No valid wording" are in compaction summaries,
and none is in a message the owner typed.
Summaries of 2026-10-20 carried the rule as prose
("No valid wording means the archive stays at that slice and the page ships (2026-09-27)"),
and from 2026-10-22T20:17Z the summary carried it in quotation marks as the owner's words,
which briefs and ledgers copied.
So a paraphrase became a quotation through successive session summaries (ledger B1253,
M324).

The rule:
the owner is quoted only from a message the owner typed or from a recorded answer,
such as the option label the owner chose,
never from a summary,
a brief or a list of rules.
A rule found only in those is written as prose with its date and no quotation marks.
When a brief quotes the owner,
the quotation is checked against the record in the design commitments file before it is copied,
and a quotation not found there is written as prose.

What enforces it:
habit,
and a search for the quoted text with `rg --fixed-strings` over the package and `doc/`,
which on 2026-11-01 (UTC) found the paraphrase quoted as the owner's words nowhere
(the hits that remain name the mistake,
here and in the ledger).

## Guards that stop an entry where a ruling says to ship

What happened:
the entry `XingZ60` settled every slice on the XingZ6023 pass and shipped no page and no artifact (ledger B1181).
A page-level guard had found a definition of note 3 that no paragraph referred to,
and threw an interruption,
which the pass files as a stop;
only a stop with every provider dry ships an interim page.
The check was written on 2026-09-09,
before the owner ruled on 2026-09-27 that a run always ships
and that a check of the page ships with the defect reported,
and it was never read against that ruling;
a second guard,
for the evidence of a carried passage,
threw the same way.
The two footnote readers also judged two pages:
the guard read the page with no original in it,
the assembly proof counted a withdrawn anchor as carrying its original (ledger B1182).

The rule:
a check of the page that ships reports and does not throw.
It is a step of `publishFixedPage`,
whose refusal `publishDefects` collects and the DEFECTS line names.
A throw is for a fault of the code,
or for a page the site cannot build.
Before a check that can stop an entry is added,
the owner's rulings on shipping are read,
and what the check finds is put in the list of defects.
Two checks of one fact read one page,
the one that ships,
unless a test says why one must leave part of it out
(the carried-evidence check leaves the originals out,
because a region found there is no evidence the lanes wrote).

What enforces it:
the cases of `page-check/src/corpus-run/publish-page-guards-report.unit.test.ts`
and the `settleEntry` case of `corpus-pass/src/corpus-run/pass-entry.unit.test.ts`,
which fail if either guard stops the entry again;
the reading of the rulings is habit.
