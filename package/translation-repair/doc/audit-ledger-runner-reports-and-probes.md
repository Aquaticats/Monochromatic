# Audit ledger: runner reports and probes

Report and probe commands that printed a claim their own figures deny,
or bought work nothing could use,
the examination of coverage rounds that were claimed to buy nothing (B811),
and the pass's per-entry lines that named neither the time nor the attempt they belong to (B960).

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B279: `ledger-report --model` cut a candidate at 400 units with no word that it was cut

Fixed in `387af02b7`,
with its cases,
which the agent ran red in its worktree.

Found by the move B278 records.
`printReading` printed each candidate through `wholeOpening` at 400 UTF-16 units and nothing else.
A candidate of 450 units showed its opening with no sign it was longer,
so a reader weighing a seat's wording took the opening for the whole.

The fix:
a cut opening is followed by a line saying how many of the whole's units are shown.
The cases in `ledger-report-print.unit.test.ts` hold an excerpt of exactly 400 units,
a cut,
and a cut that would split a surrogate pair.

Open to the owner's veto:
the line after a cut candidate.

Recurrence:
`mistake-prevention.md`,
"Checks that vouch for a whole block":
an excerpt of evidence shown to a reader states its cut.

## B280: `spend-report` said four things its own figures deny

Fixed in `387af02b7`,
with its cases,
which the agent ran red in its worktree.

Found by the move B278 records,
reading each printer whole.

- `printCost` said no call went to the metered provider wherever no seat was priced.
  A run whose every metered call sat on a seat the price table has no row for said so,
  then listed those seats under the heading for unpriced ones.
  It says none was priced in that state,
  and keeps the old line where no metered call exists.
- The USD group holds OpenRouter and Bedrock seats,
  and its heading and total said OpenRouter,
  so a Bedrock bill read as OpenRouter's.
  Heading and total name the providers present,
  byte for byte unchanged for a run on OpenRouter alone;
  a seat of neither provider throws `unreachable:`.
- The floor note wrote `1 call reported no usage block, so their tokens`.
  The pronoun follows the count through `wordForCount`.
- A log with no spend line printed NOTHING RECORDED,
  whose own line says silence is not a zero,
  and then a zero credit total.
  The report stops after its notes where no seat was read.

Open to the owner's veto:
nothing printed after NOTHING RECORDED.

Recurrence:
`mistake-prevention.md`,
"Claims without their evidence":
a claim of absence is printed only in the state that proves it.

## B281: `meter-report` merged two readings that shared a stamp and the first two providers' states

Fixed in `387af02b7`,
with its cases,
which the agent ran red in its worktree.

Found by the move B278 records.
The merge key was the stamp,
the Synthetic and Hyper states and the numbers joined with a comma.
It left out the OpenRouter and Bedrock states,
and the join let one field holding a comma read as two fields.
Two readings that differed only there collapsed into one.

The fix:
the key is every field of the reading,
so a reading differing in the third or fourth provider's state,
or in a number,
is a reading of its own;
exact repeats still collapse.

Open:
`reportLedger` and `reportMeters` set `process.exitCode` themselves,
so their cases hold and restore it.
The lead's rule for the clusters after this one is that a moved procedure returns its code and the entry sets it;
these two are left to change with that follow-up.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B282: `probe-verify` asked the whole roster before it refused a sheet already taken

Fixed in `abb506b7b`,
with its cases,
which the agent ran red in its worktree.

Found on 2026-10-06 (UTC) while moving five probe and sensitivity runners into modules (T8).
`probe-verify` asked the whole roster about every damaged and control region,
and only then found the sheet or its manifest already in the runs directory,
refused,
and wrote nothing.
A rerun over a graded sheet paid for the whole run.

The fix:
the check is `assertSheetPairFree`,
taken out of `writeSheetPair` (`sheet-write.ts`),
which still uses it,
and it runs before anything is gathered or asked.

The as-built suites of the five commands are in `139a31f4a` and pass against the unmoved commands and the moved ones,
so the move is shown to change nothing a keyless run can see.
Measured by the lead for B282 and B283:
the 22 test files of the cluster and 10 guard and inventory files pass;
lint ends with no warning and no error,
its type check included;
of the 40 source scans 39 pass,
and `dead-test-declarations` fails on four fixture names of `a13f71ba8`,
which only the test files of `1088e72d9` read.
The agent's measurement,
not rerun by the lead:
a census over its 23 test files lists none of the five bundles among those no test loaded,
and no cold stretch in the entry files or new modules beyond one throw.
The whole suite's run over this change is the one B278 records.

Open to the owner's veto:
a builder of clients in place of a client,
since a client answers a prompt it was already asked from its stored reply;
the gatherers passed in by the entry;
`resolveRunsDir()` read before the audits in `audit-sensitivity`,
where it was read after.

Recurrence:
`mistake-prevention.md`,
"Work no floor can check":
state known at the start is checked before anything is bought.

## B286: score reports said nothing, or one thing for three states

Fixed in `1088e72d9`,
with their cases,
which the agent ran red in its worktree.

Found by the move B284 records.

- `score-verify` over a sheet and a manifest with no item printed only its closing note,
  which reads as a report over something.
  It prints a line saying no set was reported.
- `score-crosscheck` printed its status breakdown label and nothing more
  where every judgeable claim was accepted.
  It prints `none`.
- `score-crosscheck` said no entry carried attribution in three different states.
  Where every claim was proposed by the whole roster,
  it said so and returned before its warning about unjudgeable claims;
  where entries carried attribution and no issue joined to it,
  it said the same.
  Each state has its own note,
  and the whole-roster note is followed by the warning.

Three branches no input reaches are `unreachable:` throws:
the rate of a critic heard on no chunk,
which rendered `n/a` or `INCONSISTENT`,
and the joins of positions behind a length check in `score-verify` and `score-probe`.

Open to the owner's veto:
a line for an empty verify sheet,
where a refusal is the alternative;
the unreachable rate throwing where it rendered a word.

Recurrence:
`mistake-prevention.md`,
"Claims without their evidence":
an early return names which state reached it.

## B288: `cap-census` counted a log once for every named path that reached it, and one unreadable log ended the census

Fixed in `53e25b458`,
with its cases,
which the agent ran red in its worktree.

Found on 2026-10-06 (UTC) while moving three report runners into modules (T8).
`capCensusLogsUnder` listed a log once for every named path that reached it,
so a directory named beside a log inside it counted each of that log's calls twice
and moved the percentile the cap rule reads.
The walk counted a directory it could not list and went on,
but a log it could not open ended the whole census as a fault,
exit 5,
discarding the census of every readable log.

The fix:
each log is taken once,
by its resolved path;
and a log that cannot be opened is counted and named like the directory,
through the one printer `reportCapCensusUnreadable`.
The cases are "TAKES A LOG ONCE where the paths named reach it twice"
and the one that begins "COUNTS A LOG IT CANNOT READ".

The as-built suites of the three commands arrive with the move and not in a red commit of their own,
since they share their files with the cases of the moved functions.
For B288 to B290,
measured by the lead:
44 test files pass,
the 18 of these three commands among them;
after one local was renamed,
the 40 source scans pass and lint ends with no warning and no error.
Before the two edits named here as the lead's,
two of the 40 scans failed,
on those two lines alone.
The held entry of `caught-value-text` names `reportCapCensusUnreadable`,
where the agent's report named `visit`;
and a local of `slice-cost-bands.ts` whose name was a position word became `smallerBound`,
since the position-reference scan reads the word as a pointer by position,
a finding the agent's report did not list (M131).
The whole suite ran on `53e25b458` in a side checkout:
1,821 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 40 source scans passing.

Recurrence:
`mistake-prevention.md`,
"Searches and censuses that miss":
a census over named paths collapses its inputs before it counts,
and the two reads of one walk share one policy.

## B289: `slice-cost-report` printed no spread block and no word about it, and a missing log as a fault

Fixed in `53e25b458`,
with its cases,
which the agent ran red in its worktree.

Found by the move B288 records.
`printSpread` returned without a word for a log with fewer than two priced slices,
or one whose cheapest priced slice took no measurable time,
so a reader could not tell an omitted block from a clean one.
A log that was not there was a fault with a stack,
exit 5.

The fix:
both arms say why no spread is shown;
and a missing log is a stated refusal naming the path,
exit 6,
while other read errors still leave as the fault they are.
A fallback of zero for a band with no bound before it (`slice-cost-bands.ts`) is an `unreachable:` throw.

Recurrence:
`mistake-prevention.md`,
"Claims without their evidence":
a branch that skips a block of a report says so.

## B294: `translate-probe` without a key reported three failed slices and exited 0

Red in `7d2ca2604`,
fixed in `da8a963cc`.

Found on 2026-10-06 (UTC) while moving five probe runners into modules a test can load (T8).
`translate-probe` built its client inside the `try` that reports one slice as failed.
With no provider key,
the stated refusal printed as `SLICE FAILED: RunConfigError: ...` once for each of three slices,
and the command ended at exit 0 having asked nobody.

The fix:
the client is built before the `try`.
A refusal to build one stops the command,
exit 6,
and a failure inside the gather is still reported for its slice.

Two more in the same commit,
each cased red first by the agent in its worktree:
`translate-probe` printed `probing the first 3` whatever the count of slices was,
and prints the smaller of the count and three;
and B308 records the count of `coverage-probe`.
With them,
no defect:
`displacement-probe` answered `false` without a word
where a relocation candidate named a slice its slicing did not hold,
in the two filters that count transcription suspects and markup donors.
No input builds that state,
so it is an `unreachable:` throw naming the slice and the slice count,
and no case drives it.

Measured by the lead on the red tree:
four of the five as-built test files pass against the unmoved commands,
and `translate-probe` fails in that one case by assertion,
exit 0 where 6 is expected.
The agent's measurement,
not rerun by the lead:
a census over its 20 test files lists none of the five bundles among those no test loaded,
and no cold stretch in the entry files or new modules beyond the one throw.
The whole suite ran on `da8a963cc` in a side checkout:
1,858 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 41 source scans passing.

Open,
to rule on:
`coverage-control-probe` refuses the whole run where an entry lacks a page,
and `coverage-probe` skips such an entry as an ordinary state of the corpus.

Open to the owner's veto:
the client built before the `try` for one slice,
where rethrowing a stated refusal from the catch is the alternative,
which would also stop a slice whose gather refused;
`probing the first 1` and `probing the first 2` in the existing form of the line;
the stamps of the log lines of `displacement-probe` read as `<time>` in its as-built cases,
where pinning them wants a clock handed to the logger.

Recurrence:
`mistake-prevention.md`,
"What a catch charges":
a catch that charges one unit of work does not also hold the setup every unit shares.

## B299: `editor-width-probe` over a draw holding no slice bought the control and wrote a report with no row

Tests in `327ea1de0`,
fixed in `77a213478`,
with its case,
which the agent ran red in its worktree.

Found on 2026-10-06 (UTC) while moving `editor-calibrate` and `editor-width-probe` into modules (T8).
`editor-width-probe 1 b` drew one slice,
of which draw B takes none,
then bought the positive control and wrote a report with no row:
a clean finish over no work,
paid for.

The fix:
a draw that holds no slice is a stated refusal before the control,
naming how many slices that draw needs.

The as-built cases are in `327ea1de0` and pass against the unmoved commands and the moved ones:
six for `editor-calibrate` and two for `editor-width-probe`,
every child through the keyless fixture,
over a throwaway repository holding one invented entry.
The whole suite ran on `77a213478` in a side checkout:
1,898 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 41 source scans passing.

Open,
seen by the agent and left to rule on:
`editor-calibrate-standing.ts` counts a round whose lone composite candidate has no ballot as judged,
while the shipped table counts it as not unvoted;
and the wording of the reach line,
"the rest carried no paragraph",
which the agent questions.
B307 closes the third thing it saw,
the bare `Error` of `bench-sample.ts`.

Open to the owner's veto:
the stage seams of the width probe (`controlHolds`,
`gather`,
`runSlice`,
`writeReport`) and the draw passed in as functions;
the wording of the empty-draw refusal;
`readOverlap`,
`adoptGrace`,
`readWriterGrace` and `newClient` passed as functions.

Recurrence:
`mistake-prevention.md`,
"Work no floor can check":
state known at the start is checked before anything is bought.

## B304: the pass order left medium and large bands tied, and retried a failing entry first on every run

Fixed in `9a68e3c78`,
with its cases,
which the agent ran red in its worktree.

Found by the move B302 records,
reading the comparator against its own comment.
The comparator's comment and the bands' own doc say the larger band leads within one rank,
and that the fewest attempts go first so a flaky entry is deprioritized.
The code let only the small band trail,
so medium and large tied.
Ranks are unique within a band,
so the attempts tier could only ever break that one tie:
an entry that kept failing kept its place in the walk and was retried first on every run.

The fix:
large leads medium leads small within a rank,
and a band's ranks follow fewest attempts first,
so a tried entry waits behind the untried ones of its band.
Which entry a pass starts first can differ from before.
`smallBandIds` (`band-order.ts`),
which the new order leaves with no caller,
is removed with its cases and its barrel line;
and `selectPendingEntries` computed the order twice and dropped the first result,
and computes it once.
The whole suite's run on `9a68e3c78` is recorded in B302.

Not measured:
no pass ran under the new order,
and no count was taken of how often a tried entry led a past run.

Open to the owner's veto:
the order as described,
and reverting to the comparator as it was.

Recurrence:
`mistake-prevention.md`,
"Claims without their evidence":
a comment that states an order is a claim,
and a case holds each tier it names.

## B305: `producer-calibrate` printed a standing over rounds that never ran

Red in `cd78288d7`,
fixed in `f8e99b84d`.

Found on 2026-10-06 (UTC) while moving `producer-calibrate`,
`recall-benchmark` and `roster-bench` into modules (T8).

- The run client was built inside the catch that counts a slice as lost.
  A launch with no provider key printed `LOST (RunConfigError)` for every slice,
  then a standing over no round,
  and exited 0.
  The loop rethrows a stated refusal.
- `producer-calibrate --candidates <one id> --candidates-alone` drew the corpus
  and asked the translate stage for every slice,
  which refused the same roster of one seat each time;
  each refusal was read as a lost slice.
  The roster is checked before the draw,
  and the stage's own refusal is stated.
- A calibration in which no round finished printed `STANDING over 0 rounds` and exited 0.
  It refuses,
  after the lost lines,
  naming how many slices were drawn.

The first is cased in `cd78288d7`;
the other two were cased red first by the agent in its worktree.

Measured by the lead on the red tree for B305 to B307:
six cases fail,
each on the assertion its commit message names,
and no other case of the four files fails.
Measured by the lead on the fix tree:
the cluster's thirteen test files,
the three as-built suites among them,
print no fail line against a fresh build;
the source scans print 41 pass lines and no fail line;
and lint reports no warning and no error.
The agent's measurements,
not rerun by the lead:
a census over its eleven test files shows every one of the eleven sources loaded and no cold stretch,
with two invariant throws (`producer-calibrate-run.ts`,
`roster-bench-run.ts`).
Not reachable without a key,
by the same agent:
a round that reaches a provider,
a recall run with real quorums,
a report written from a live row.

The whole suite ran on `f8e99b84d` in a side checkout and is not green:
1,938 `[PASS]` lines and 2 `[FAIL]` lines,
the lint finding nothing,
and the 41 source scans passing.
Both fail lines are for one case this change does not touch,
the case and its suite:
`displacement-probe as built`,
"REFUSES as stated and exits 6 when an artifact names a commit the clone lacks".
B300 records that case,
the lead's reading of why it fails under load,
and that its fix is in no commit through `f8e99b84d`.

Open,
seen by the agent and left:
`BenchReportError` is not a stated class,
so any other caller of `benchWidths` prints its refusal as a fault;
and `drawPinned` is written twice,
once in each of two entry files.

Open to the owner's veto:
the missing-key refusal comes at the first slice,
after the draw and the opening line,
since each slice builds a fresh client;
the wording of each new refusal.

Recurrence:
`mistake-prevention.md`,
"What a catch charges":
a catch that counts one unit as lost rethrows a refusal that would be every unit's.

## B306: `recall-benchmark --plan` printed `PLAN ok` over a corpus that yielded no entry

Red in `cd78288d7`,
fixed in `f8e99b84d`.

Found by the move B305 records.
`--plan` is the setup check before a run of hours.
Over a clone that holds no entry the plan passed,
and the key refusal followed.

The fix:
it refuses before a client is built.
The whole suite's run on `f8e99b84d` is recorded in B305.

Open:
`recall-benchmark` without `--plan` over a corpus that yields no entry still builds a client
and keeps an empty scorecard before its own refusal,
where the plan refuses before the client.

Recurrence:
`mistake-prevention.md`,
"Work no floor can check":
a setup check that found nothing to run over does not print that the plan is in order.

## B312: the settled rendering audit named its runs directory only after buying every subject

Fixed in `8206fa5c6`,
with its case run red by the agent in its worktree.

Found on 2026-10-06 (UTC) by the agent moving `rendering-audit-settled`.
The runs directory was resolved when the run was kept,
after the last roster call,
so a directory that could not be named cost the whole roster's calls.

The fix:
`runSettledAudit` (`corpus-run/rendering-audit-settled-drive.ts`) resolves the runs directory
right after the buying line,
before the client is built;
a case with a resolver that refuses asserts that no model,
client or page read happened.

Recurrence:
`mistake-prevention.md`,
"Work no floor can check":
state known at the start is checked before anything is bought.

## B372: the standing reader counted no digest whose rounds drew no ballot

Red in `bb2a8b9ad1e164974e54d2750432a25b267ef0de`,
fixed in `cfac5b981`.

Found on 2026-10-06 (UTC) by the review of the runner observations change,
its sixth finding.
The summary printed `digestsWithRounds=`,
so beside a judged digest,
a digest whose rounds drew no ballot was counted nowhere.
By the agent's run on the port build,
a run of two digests,
one judged and one with no ballot,
printed `digestsWithRounds=1` and no trace of the second.

The fix:
the summary prints `digestsJudged=`,
`digestsUnjudged=` and `unjudgedDigestRounds=` in its place,
and the case compares all four runs in one object;
`child-environment.unit.test.ts`,
outside the change,
quotes the summary line and moved with it.

Open to the owner's veto:
`editor-standing-read` still exits 1 for rounds with no ballot,
with the summary fields `digestsJudged`,
`digestsUnjudged` and `unjudgedDigestRounds`,
and an off-roster note that follows its count;
the report names no alternative.

Recurrence:
`mistake-prevention.md`,
"Claims without their evidence".

## B800: the picture reader measurement told a run short of trusted records to ask again with a larger sample

Red `b1a88def13`,
fixed in `43589efacb`,
`3a4ac9e2ea` and `2d38261279`.

Found on 2026-10-21 (UTC) in the three keyed runs of `reader-measure` for `gemma-4-26b-a4b-it`.

What the runs showed:
the first two runs (08:44 UTC and 08:48 UTC) named the 20 recent `corpus-pass-*` runs directories,
the second adding the full-corpus pass of 2026-08-19.
They read 3 trusted records,
matched 3 of the 291 pictures,
produced 3 readings,
and printed `RESULT: not measured: 3 of the 6 readings a verdict needs were produced.`
with `NEXT: keep the hold 'openrouter-pictures-unmeasured'; ask again with a larger --sample, or read the per-picture reasons the run printed.`
The recent runs directories hold 3 `picture.<key>.json` files in all,
every one for one entry,
so a larger sample could not have added a picture:
the loop had already tried all 291.
The older single-entry runs directories hold 450 such files,
and the third run (09:02 UTC),
naming 107 of them,
read 192 distinct trusted records across 10 entries and was measured.
Two other gaps showed in the same printouts.
The candidate's rates were printed without the floor and ceiling they were held to,
so a verdict could not be checked from the printout,
and nothing said which entries the matched pictures came from,
so a measurement resting on two entries read like one resting on the corpus.

The cause:
`verdictLines` in `reader-measure-report.ts` printed one fixed `NEXT` line for every candidate that produced too few readings,
whichever input had limited the count,
and `renderMeasureReport` was handed neither the sample asked,
the trusted records read,
nor the pictures found,
so it could not tell the three limits apart.
The sample limits the count only when pictures were left untried.
The trusted records limit it when every picture was tried and fewer than six matched.
The candidate's own answers limit it when six or more matched and few readings came back.

The fix:
the report prints a `CANDIDATE:` line with the candidate's reading count and both of its shares beside the seated floor and ceiling
(or says no reading was produced),
an `OVERLAP:` line,
and an `ENTRIES:` line counting the matched pictures by the first directory of their path,
most pictures first.
The `NEXT` line for too few readings asks for more trusted records when every picture was tried and fewer than six matched,
naming `--trusted` directories whose runs directories hold `slice-cache/<entry>/picture.<key>.json` files of other entries,
and `--pictures`;
asks for a larger `--sample` only when pictures were left untried;
and points at the per-picture reasons when six or more pictures matched.
A run is read as having tried every picture when it matched fewer pictures than its sample,
since the loop stops short of its sample only when the pictures run out.
Two follow-up commits brought the lines within the lint rules and the source scans:
no position references,
an example calling every required key,
and every count noun counted by `wordForCount`.

Limits:
the trusted records name no picture,
so the report counts records and cannot say which entries they came from;
only the matched pictures are grouped by entry.
A picture directly in the pictures directory is counted under `(no entry directory)`.
The `ENTRIES:` line shows how narrow a sample is and does not set a floor on it.

Cache versions:
none.
The report is printed text.

Open to the owner's veto:
a floor on the number of entries a measurement rests on.
The measurement of 2026-10-21 rested on two (Mio 7,
Toka_ls 2),
and the rule of 2026-09-08 sets none.

Recurrence:
`mistake-prevention-messages-labels-and-counts.md`,
"Log lines that report nothing done".

## B811: the coverage rounds over the untranslated tail were examined and kept, and no skip or reordering is safe

Status:
examined on 2026-10-22 (UTC) in `w272`;
nothing changed in code,
and no model-facing text,
cache key or cached-step input changed.
One owner decision is open:
ending a round early once its outcome is fixed (see "Open for the owner").

The claim examined:
the corpus pass `XingZ6017` (build `2a336603bf`) spent 712 s (11 min 52 s),
from 23:18:40 UTC to 23:30:32 UTC on 2026-10-21,
in `decidePassInsertionAdmission` (`package/translation-repair/corpus-pass/src/corpus-run/pass-insertion-admission.ts`).
It ran 30 per-slice `coverage` rounds over slices 88 to 117
and then logged (line 1201 of `~/temp/agent/corpus-pass-XingZ6017.log`):
`untranslated tail: 30 slice(s) after the last agreed pair, 5131 source code points at expansion 2.94, expected 15074, last pair renders 147: admitted on that bound`.
The claim is that the rounds bought nothing because that bound decided anyway.

What the code does,
read at the head `24c736ea80`.
`git log --oneline 2a336603bf..HEAD --` over `pass-insertion-admission.ts`,
`insertion-coverage-model.ts`,
`coverage-tail.ts`,
`coverage-stage.ts`,
`coverage-verdict.ts`,
`stage-quorum.ts` and `stage-round.ts` printed nothing:
none of the seven files changed since the run's build.

- The tail and its bound do not depend on any round.
  `readUntranslatedTail` (`archive/src/coverage-tail.ts`,
  line 169) reads only `prepared.slices`,
  and the log line is printed after the rounds
  (`pass-insertion-admission.ts`,
  lines 322 to 344).
  So running the rounds after the bound check changes nothing about the work;
  the bound could be computed first,
  and the rounds are still the input to the next step.
- The rounds are the first of the two signatures `doc/decision/translation-repair-absence-verdict.md` requires:
  a roster that read both whole texts leaves the passage unplaced,
  and a model-free signal corroborates.
  The tail bound is the second signature and replaced the whole-page shortfall.
  The addendum of 2026-09-19 says so:
  "The second signature stays model-free."
  Dropping the rounds would admit on one signature,
  which the decision forbids.
- Inside `classifyInsertionCoverage` (`insertion/src/corpus-run/insertion-coverage-model.ts`,
  lines 200 to 217)
  the bound admits a tail row only when `row.verdictKind` is `absent` or `split`.
  A row the majority found `carried` goes to `carried`
  (not inserted;
  folded into a neighbour by `pass-carried-fold.ts`,
  or guarded at publish),
  and a row found `partly-carried` or `inconclusive` stays unresolved and ships as a recorded gap.
  In those cases the verdict changes the outcome even though the bound admits.
  `insertion/src/corpus-run/insertion-tail-split.unit.test.ts`
  ("STILL LEAVES a tail slice a majority found partly carried out of the admission") pins that.
  The bound says the tail cannot have been merged into the last agreed pair;
  it does not say the passage is not rendered somewhere else on the page.
  The rounds search the whole target independent of pairing,
  which is their point.
- An unheard roster for any row (`row.heard === 0`) throws `TranslationRepairInterruptedError`
  (`pass-insertion-admission.ts`,
  lines 310 to 317).
  That outage check would go with the rounds.

What the logs show:
all 60 corpus-pass and `XingZ6*.log` files in `~/temp/agent` were scanned
(`~/temp/agent/tr-fan/tr-w272-notes/tally.ts`).
36 files hold coverage rounds,
1249 rounds in all,
every one on the `XingZ60` page:
955 `absent`,
294 `split`,
and none `carried`,
`partly-carried` or `inconclusive`.
`zheermao22` and `CuspariaKLSY22` ran no coverage round (no source-only slice),
so they say nothing about the tail.
`XingZ6015` and `XingZ6017`,
the two runs with the current rule,
ran 31 and 30 rounds,
`absent` and `split` 27 and 4 in the first and 24 and 6 in the second,
all admitted on the bound.

So on this page the verdict has never changed the tail outcome,
which supports the claim for this page and this archive.
It does not support it as a rule.
The `XingZ6017` artifact (`corpus-pass-XingZ6017/artifacts/XingZ60.json`)
shows one voice anchoring a `full` claim on slices 88,
97,
111 and 116,
and a majority needs 4 of 6.
A lone dissenting voice does occur (4 of 30 rows),
and was three votes short of a carried verdict on those rows,
a thin margin that this page happens to have and the rule cannot assume.
`XingZ60` is the only page among the 36 scanned logs that ran a coverage round,
so there is no sample of a page where a tail slice is rendered elsewhere.

Time:
the per-round totals sum to 2518 s in `XingZ6017`,
of which 970 s is time to quorum and 1548 s is the straggler grace after quorum.
2518 s over 712 s is a concurrency of about 3.5,
consistent with the overlap of 4 the JSDoc example shows;
the live overlap value is not logged,
so that is inferred.
`XingZ6015` ran 1596 s in all,
443 s to quorum and 1153 s of grace.
Seven rows in `XingZ6017` ended with one asked voice never heard (5 of 6 on slices 89,
91,
96,
99,
100,
105 and 110),
which most likely spent the grace to its end;
the log does not name the missing voice.

The decision:
no skip,
no reordering.
The rounds protect the carried-elsewhere case
(duplicated content or a lost carried-evidence guard),
the partly-carried and inconclusive cases
(a recorded gap rather than an insertion on one signature),
and the unheard-roster outage stop.

Open for the owner:
ending a round when its outcome is fixed.
Most of the 712 s is straggler grace
(`STRAGGLER_GRACE_MS` and `grace.roundMs`,
`doc/decision/translation-repair-straggler-grace.md`)
spent after the tail outcome was already certain.
For a tail row the bound admits,
the outcome is `admitted` unless `anchoredAny >= floor(asked/2)+1` (carried or partly-carried) or the quorum fails.
With 6 asked (a majority of 4) the outcome is fixed as soon as `anchoredAny + unheard < 4`:
three non-covering voices at quorum are enough.
Stopping there would keep the position set and the carried list identical,
and would change only the recorded tallies (`heard 3 of 6`),
the `absent` or `split` label and the findings text.
The upper bound on the saving is the grace share of the rounds,
about 61 percent of seat time in `XingZ6017`,
about 7 minutes of 11.9.
It was not built,
for three reasons.

- It overrides a standing owner decision on grace:
  "No grace change is authorized before active runs finish and matched evidence exists"
  (`doc/decision/translation-repair-straggler-grace.md`).
- It needs an early-stop predicate in the shared `stage-round.ts`,
  and `judgeCoverage` anchors each voice's quote against the target only after `gatherStageVoices` returns,
  so the anchoring would have to move into the round per voice.
  That is why this is not a small change.
- It cannot be measured from the logs,
  because the order voices arrived in is not logged.
  A measurement would add the per-voice arrival order to the round line first.

Commits:
none.

## B960: the pass's per-entry lines carried no clock time and no attempt, and START did not say the cap is per attempt

Fixed on 2026-10-25 (UTC) in `4b85db7c20` (slice-cache),
`7126ed31e4` (page-check) and `9780419baf` (corpus-pass),
with the stamp writer in `89c495264a` (markup).
Their cases were committed red first,
in `541a576781`,
`09d99aea15` and `7b711c6047`,
and each failed on its parent for the missing field.

Found on 2026-10-22 reading the `XingZ60` pass.
Its log carried a `TALLY XingZ60` line with `status=ERROR`,
`ms=25200743`,
`aborted=true` and `error=Timeout: XingZ60 exceeded its 25200000ms deadline`,
then a `REATTEMPT XingZ60` line for 310 more cache records,
and the second attempt began under a fresh 420-minute cap.
That is the design (`doc/configuration.md`,
"The cap ends an attempt rather than an entry"),
but nothing on those lines said when each was printed or which attempt it closed,
and the `START` line said `hard=25200000ms` and no more.
A reader looking for what happened inside a time window could not place a `TALLY`,
a `REATTEMPT` or an `OVERLAP` line,
and the cap read as one that had fired without ending the entry.

What the census found
(a read of every `console` call in the non-test source of `corpus-pass`,
`slice-cache`,
`page-check` and `artifact-pool`,
of the other `translation-repair` packages,
and of the tagged-logger calls of the pass packages):

- No `console.log` line of the pass carried the time it was printed at.
  Only the tagged logger's lines do (`[level] [iso] ...`,
  the spelling `toISOString` writes),
  which `isIsoStampText` reads back (B73).
  `returnsAt=` on a `PARKED` tally is a time of day in the future.
- Ten kinds of line are printed for an attempt of an entry:
  `OVERLAP`,
  `TALLY` for settled,
  error,
  incomplete,
  parked and declined entries,
  `INTERRUPTED`,
  `DESTINATIONS`,
  `DEFECTS`,
  `CLEANUP`,
  `REATTEMPT`,
  `STALLED`,
  `INCOMPLETE` and `NOT RESUMED`.
- No code reads them:
  a search of every non-test source for the keywords found only emitters and comments.
  The readers are the tests that pin their text and the docs that quote them.

The fix:

- Each of those lines carries `at=<ISO 8601 UTC with milliseconds>` and `attempt=<n>` straight after its opening words
  (the keyword and the entry,
  and the status of a `TALLY` or the `queued` of a `REATTEMPT`),
  before the other fields and before any capped text,
  so a search by prefix still finds them.
  `lifecycleStamp` in `markup/src/lifecycle-stamp.ts` writes the two fields in one place.
- The clock is a parameter (`wallClock`,
  the wall clock by default) all the way down,
  and no case waits on the real one.
  Durations stay on the monotonic clock.
- The attempt is the number `attempts.json` holds once the pass has counted the attempt,
  handed to the settlement by `runPassQueue` and to `runAttemptQueue` as `attemptOf`.
  A `TALLY` and the `REATTEMPT` after it print the same number,
  and a case with four earlier attempts counted prints 5 and 6.
  A number that is not a whole number from 1 is refused with `LifecycleAttemptError`.
- `START` says `hard=<ms>ms hard-scope=per-attempt`.
- Handover,
  planning and audit files that quote the lines as they were printed are records and are not rewritten.

Cache versions:
none moved.
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' 5c40a5fcb9..HEAD` printed nothing,
and its control over `c265e8df95~1..c265e8df95` printed `c265e8df95`.
No prompt,
model call or cached step input changed.

Left open by this round and answered in round 2
(`audit-ledger-lifecycle-line-stamps-round-two.md`,
B961 to B964):

- `SOFT budget reached`,
  `SPEND CEILING reached` and `STOP every provider ...` carried no clock time.
  `the pass resumes` is a tagged-logger line and always carried the logger's.
- The logger's own per-entry lines carried the logger's stamp and no attempt number.
- `CAP OVERRIDDEN` read "entries run under N minutes" without saying per attempt.

Open to the owner's veto:
the attempt counts every attempt of every invocation,
because that is what `attempts.json` holds and the ordering already reads it,
so a first attempt of a later invocation prints `attempt=5` for an entry already tried four times.
A count within the invocation would restart at 1 and could not be found in any file.

Recurrence:
`mistake-prevention-messages-labels-and-counts.md`,
"A lifecycle line a reader windows by time".
