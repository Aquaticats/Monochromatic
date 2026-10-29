# Audit ledger: process mistakes, questions and handoffs to the owner

What was put to the owner wrongly:
measurable questions,
options the evidence contradicts,
and requests the owner had ruled out.

Part of [Audit ledger: process mistakes](audit-ledger-process-mistakes.md),
which lists every file of this record.

## M7: an adopted reading parked in docs across a compaction

Status:
fixed (owner confirmed "English letters" on 2026-09-27).
The "kept in English letters" reading waited in the planning doc for a veto instead of being asked.
Prevention:
a reading adopted without the owner's answer is asked in the same turn.

## M8: asked the owner a measurable question

Status:
corrected;
measured,
and the bench reseated in L1.
The agent asked which model should fill the third checker seat,
offering gemma-4-31b,
Mercury 2.5,
a stop,
or no change,
when checker quality on the role's own sheet is measurable.
Owner,
2026-09-27:
"Why don't you measure?
Also Mercury 2.5 is really cheap so it's fine."
Measurement:
`~/temp/agent/audit-repair/checker-cases.mjs` builds 85 pairs from twelve runs
(a unanimous-fixed patch,
and the same issue against the unchanged archive,
where not-fixed is certain);
`checker-bench.mjs` and `checker-bench-raw.mjs` score each candidate on the resolution sheet.
The owner approved Mimo v2.6 Flash and Mimo v2.6 Pro on OpenRouter the same day,
then Solar Mini 4 (`upstage/solar-mini4`);
all three are among the candidates.
Solar Mini 4 is out of the checker seat on its first reading:
78 of 85 positives called fixed,
but only 7 of 85 unchanged archive texts called not-fixed,
so it calls almost any text fixed.
Result:
the ranked order in `RUN_CHECKER_ORDER` (L1).
The Mimo candidates were first scored by raw fetch without `zdr`;
through the run client Mimo v2.6 Flash reaches only DeepInfra,
its one zero-retention endpoint,
and scored 81 and 80 at 3.9 s median against 82 and 79 at 0.35 s.
A measurement for a seat is taken on the route the run uses.
Prevention:
a model-for-role choice is measured on that role's task before anything is asked;
only what a measurement cannot settle goes to the owner.

## M15: a finding carried and a fix started against an owner ruling

Status:
corrected;
the ruling is now in `doc/design-commitments.md` (moved on 2026-10-18 to
`package/translation-repair/doc/design-commitments.md`).
A16 listed a triple newline as a defect,
and a seam-spacing guard was written for it,
though the owner had said extra newlines need no fixing,
since Markdown renders them alike;
the ruling was in no package doc,
so neither the audit that filed A16 nor the fix read it.
Owner,
2026-09-27:
"There is no need to eliminate extra newlines,
because markdown doesn't care.
I believe I said this before."
Prevention:
an owner ruling goes into `doc/design-commitments.md` (moved on 2026-10-18 to
`package/translation-repair/doc/design-commitments.md`) (or its decision record) the turn it is given,
and a finding is checked against those commitments before any fix starts.

## M18: a count put to the owner that measured something narrower than the option it backed

Status:
corrected by re-asking,
2026-09-27 (A16c).
A question said a page had never disagreed with its artifact,
"0 of 214 pages across 372 run directories";
the script behind the number had compared which files exist and never judged a page against its artifact.
The owner chose on that premise;
judged,
77 of 214 disagree.
The ledger already said so:
F-2 recorded 77 of 209 pages that no longer reproduce by splice,
so a search of this file before asking would have caught the claim.
Prevention:
every number in a question option names the check that produced it,
and a claim about a state is backed by a run of the instrument that decides that state,
with its positive control,
before the question is asked;
and the ledger is searched for the state (`rg` over `doc/audit-ledger.md` (moved on 2026-10-18 to
`package/translation-repair/doc/audit-ledger.md`) and `doc/status.md` (moved on 2026-10-18 to
`package/translation-repair/doc/status.md`))
before any claim that it has never occurred.

## M30: quality calls put to the owner as design questions

Status:
caught by the owner,
2026-09-28;
the directive is recorded in `design-commitments.md`.
After the L4 replay,
the fix-shaped refusals and the L11 probe's role went to the owner as two option sets
with a recommended option that saved effort (keep the gate as built;
keep the probe in shadow).
Each set differed only in what ships,
so the answer was fixed by the goal:
"Do not try to save effort and just do the option that would result in best quality of the end result",
now a standing directive.
An earlier instance of the same shape:
the P9 question recommended deleting the re-ask,
and the owner chose to enable it.
Prevention:
before asking,
name what the options differ in;
if it is only the quality of the end result,
build the best option unasked and record why;
ask only when a quality choice conflicts with an earlier ruling or the options differ in something else.
Effort,
code size and wall clock never rank an option first.

## M146: closing notes the review page does not collect left reversed choices on it

Status:
happened on 2026-10-10 (UTC),
by the lead's notes for docs batch thirteen;
caught by the advisor before the page was handed to the owner.
The closing notes docs batch twelve added
(each beginning "Closed on 2026-10-10 (UTC)")
were not read by the page of veto choices for the owner,
which collects paragraphs by a marker phrase,
so the old veto paragraphs of B379,
B381 and the artifact pool case in the T8 passage "The runner and git cluster of the final census"
stayed on the page as live cards,
though that round had reversed what they state:
the owner would have been asked to keep or veto behaviour that no longer exists.
One batch of the page's choices lost 8 to pointers
(the page's items T8-5 choice c,
B379-1,
and B381-1 choices b to g),
and the page then held 397 choices and 13 pointers.

Prevention:
when a later round takes the alternative of a recorded veto,
the lead writes the closing note,
lists the item ids the page holds for the old paragraph,
and turns each reversed choice into a pointer on the page,
or removes it,
before the page is rebuilt,
then drives the page again.
`mistake-prevention.md`,
"Questions to the owner".

## M148: an option put to the owner described a sense the corpus contradicts

Status:
happened on 2026-10-11 (UTC),
in the lead's question about 扑面而来;
caught before any change shipped.
The question said the idiom means coming straight at the onlooker,
not at the subject of the sentence,
and the owner chose a dictionary entry on that reading.
The pinned corpus carries the idiom on four pages,
and on three of them (MocaKawai,
lin10104,
republic_o85611) pressure or loneliness washes over the person herself,
so an entry refusing "toward her" would have refused three correct renderings.
The entry (`6bc235960`) says the target is whoever faces it and refuses no form.

Prevention:
a glossary sense put to the owner is measured over the pinned corpus first,
every occurrence read,
before the question is asked.
`mistake-prevention.md`,
"Questions to the owner".

## M159: the lead's own decision made a writer bench out of reach stop the consistency pass

Status:
found on 2026-10-12 (UTC) by the independent review of the rebuild,
and changed in the work that fixes that review's findings.
The lead decided that the page consistency phase should lean on the writer bench too,
without reading what a bench below its floor does in that phase:
the seat reading throws,
the pass contains the throw,
and the page ships with no first round and no code splice,
under a log line saying the entry stops when it does not.
The lead now decides that a writer bench out of reach leaves the carry without writers,
and the judges' bench alone decides whether the pass runs.

Prevention:
a decision that changes which benches a phase leans on is traced through the seat reading's refusal and the
caller's catch before it is briefed,
with the log lines each path prints.

## M160: the lead handed the owner a cleanup command whose list stopped short of the worktrees named

Status:
happened on 2026-10-12 (UTC).
The lead asked the owner to run `remove-worktrees.ts force` for the worktrees `w109` to `w127`;
the script's list ended at `w104`,
so the owner's run removed 104 older worktrees and none of those named.
The lead widened the list to `w127`,
leaving out the worktree an agent still held,
and removed the clean ones itself.

Prevention:
before handing the owner a command,
the lead reads what it acts on and runs its dry mode.

## M161: the lead asked the owner to renew a credential the owner had ruled out paying for

Status:
happened on 2026-10-12 (UTC) in the read of TianqiChen66623.
The lead listed renewing the Bedrock Gemma seats' expired credential as the owner's action before the next launch,
though the owner does not top up Bedrock (2026-09-07)
and the run withholds both seats after their first failed calls,
so nothing waited on them.
The pass log,
the handover and `status.md` were corrected the same day.

Prevention:
before naming an owner action,
the lead checks the owner's rulings on that provider and whether any launch waits on it.

## M320: a ruling that began "going forward" was read as covering work already done, and running work was stopped

Status:
happened on 2026-10-29 (UTC),
recorded from the lead's notes.
The owner answered the question of where finished rounds should land with "Branch only",
and added:
"Branch only,
growing merges are acceptable.
Only merge from rebased to main or main to rebased whenever I tell you."

The lead measured which main commits each round that had merged main carried beyond the branch
(`w363` 3,
`w366` 23,
`w367` 29,
`w370` 17),
concluded that none could land as it was,
wrote a plan to redo each round on a fresh tree at the branch,
and stopped gate g233 (on `1084f7adb3`) and sweep g232 (on `b47f95a5a8`).
The owner then said:
"They aren't unmergeable.
I only said that applies going forward.
Do not abandon any work."
The plan was dropped,
the gate restarted as g233b and the sweep as g232b.
The notes name the cause:
the lead read "going forward" as covering merges already made.

Prevention:
`mistake-prevention-questions-to-the-owner-and-rulings.md`,
"Rulings read for the work they reach".

Recurrence:
M15,
a fix started against a ruling,
and M161,
an owner action asked for against a ruling the owner had given.
