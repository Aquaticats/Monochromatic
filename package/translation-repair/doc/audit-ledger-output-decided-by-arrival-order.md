# Audit ledger: output decided by arrival order

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

Findings,
refusals and log lines whose content or order depended on which concurrent work ended first,
the scan that looks for it,
and the scripted cases that read a seat the rotated window did not ask (B701,
B734,
B812).

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B253: one checker round read twice ordered its findings by which seat lost its first answer

Red in `3d4558d9c`,
fixed in `df807a8b5`.

Found on 2026-10-06 (UTC).
`runCheckerStage` took the gather's order,
the retry round and then the prompt rotation of the bench.
One input gave its findings and readings in two orders,
depending on which seat lost its first answer,
and two cases had sorted the findings,
which hid it.

The fix:
heard voices are ordered by roster through `inRosterOrder`,
so ballots,
findings,
readings and log lines follow the roster,
and the two sorts are removed from the cases,
whose assertions are now exact.
The case is "ORDERS THE READINGS AND THE FINDINGS BY ROSTER whichever seats lost their first answer,
since the gather hands voices back by the round that heard them and one input must read the same twice".

With the same change,
and with no change of behaviour,
the sentence for a pairing set aside is shared by the two commands that print it
(`pairingSetAsideSentence`,
the item B196 left open).

Open:
`refine-recheck.unit.test.ts` sorts its missing-check lines,
which may now be redundant (the agent's note).

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B287: the author table of `score-crosscheck` ordered ties by the directory listing

Fixed in `1088e72d9`,
with its case,
which the agent ran red in its worktree.

Found by the move B284 records.
Authors with equal accepted counts came out in whichever order the artifacts were read,
and the listing is not sorted.

The fix:
`tallyAuthors` breaks a tie by model id in code-point order.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B310: `verify-published` printed its entry lines in the order the disk answered

Red in `a9d447691`,
fixed in `8206fa5c6`.

Found on 2026-10-06 (UTC) by the agent moving the command.
It read its entries together,
and each read printed its lines as it finished,
so a run's entry lines came in the order the reads ended,
not in the order the run lists its entries,
and two runs over one directory could print two orders.
By the agent's measurement,
six of six runs of the unmoved command over sixty settled entries printed them out of order.
The red commit's case over sixty entries expects listing order.

The fix:
`judgePublishedEntry` (`corpus-run/verify-published-entry.ts`) returns its lines,
the reads still run together,
and `verifyPublishedRun` (`corpus-run/verify-published-run.ts`) prints each entry's lines in listing order.

Open to the owner's veto:
the listing order;
the bytes of a report over two or more entries can differ from before in order only.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order",
which named this defect from the lead's notes before its merge.

## B316: when several concurrent reads failed, the refusal named whichever ended first

Red in `75085f17f`,
fixed in `f3c2bae13`.

Found on 2026-10-06 (UTC),
the case B300 left open:
`displacement-probe as built` failed in whole-suite runs on its expected stderr for a commit the clone lacks,
and passed alone.
`carveSettled` (`corpus-run/settled-carve.ts`) read an entry's `page.md` and `page.en.md` with `Promise.all`,
both reads refuse for such a commit,
and `Promise.all` rejects with whichever member fails first by the clock.
The agent's real-git probe of 100 such pairs named `page.md` 98 times and `page.en.md` twice,
which measures the reading B300 recorded as the lead's inference.
The diff of the suite failure was cut,
so tying that failure to this cause stays an inference.
Measured by the lead on the red tree:
nine site cases fail,
each printing the later member's refusal,
in `corpus-name-index`,
`displacement-probe-run`,
`draw-sample-pool`,
`editor-calibrate-drive`,
`entry-pictures`,
`required-providers` and `translate-repair`.

The fix:
`allInInputOrder` (`all-in-input-order.ts`) takes promises the caller has already started,
so the work still runs together;
where members fail,
it throws what the first failing member in input order rejected with,
unchanged,
once every member before it has settled.
It cancels nothing and gives a caller's abort no precedence.
Eight sites moved onto it with a case that scripts the later member failing first,
`carveSettled` taking its corpus reader as a required parameter so its case can;
B317 records the sites moved without one.
By the agent's measurement,
with the helper degraded to `Promise.all` seven of its cases and every site case fail.
The whole suite on `f3c2bae13`,
run in a side checkout:
1,986 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and 42 source scans passing.

Open,
and given to a follow-up agent whose work is not merged at `3ce947dce`:
the sites other agents were moving then
(`pass-republish`,
the settled audit's input and buy,
the coverage census's reads,
`verify-published-run`);
a scan failing a bare `Promise.all` outside the helper;
the bench sample's skip line,
left on `Promise.all` because its cut of 80 UTF-16 units ends inside the commit hash,
so both pages print one line;
and lines several members print as they end,
which still come out in completion order.

Open to the owner's veto:
first in input order as the rule,
at the cost that a later failure waits for the earlier members to settle;
one helper and no variant;
in `corpus-name-index`,
an entry whose original is absent is stepped over even when its archive page was refused for another reason;
`translate-repair` moved though its voices' refusals differ only in a detail field.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B317: fifteen more concurrent read sets reported the failure that came first, two under docs saying otherwise

Fixed in `f3c2bae13`,
with no case at the sites;
what holds them is the helper's own cases.

Found on 2026-10-06 (UTC) by the agent of B316,
in its census of every `Promise.all`,
`allSettled`,
`race` and `any` in the package's source:
by its count,
53 lines in 40 files called `Promise.all` in production code at its base,
and 28 in 20 after the fix.
Fifteen sites read with no parameter a case could script
(a direct file read,
`stat`,
`rm`,
`readlink` or a git child),
among them the cache account's readers,
the pass schema census and guard,
the pipeline digest,
the published tree listing,
the report log read,
the score probe's gather,
the process identity and the cited reference lookup.
The TSDoc of `readLogTexts` said it reported "the first to fail",
which `Promise.all` did not deliver,
and the comment of `score-probe-gather` said its first report was not fixed.

The fix:
each site goes through `allInInputOrder`,
and the TSDoc and the comment say what the code does.

Open to the owner's veto:
the sites moved with no case of their own.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B328: a test helper sorted away the order the recheck guarantees

Changed in `75ea14bff`;
its red is the agent's measurement in its worktree,
since the cases pass on the stage as it stands.

Found on 2026-10-06 (UTC),
the helper a review had flagged without settling.
`missingChecksSorted` and `readingsInSeatOrder` put the findings and the ballots of every recheck verdict in order
before the cases of `refine-recheck.unit.test.ts` compared them,
for a gather order the stage no longer exposes:
`repair-edit-stages.ts` returns them in the roster's order.
By the agent's measurement,
with the roster ordering removed from the stage every case of the file still passed,
and with both helpers removed as well six failed.

The fix:
both helpers are gone from that file,
and its cases assert the roster order as the stage returns it.

Open:
`refine-slice-settle.unit.test.ts` wraps `readingsInSeatOrder` the same way.
Closed since by B343.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order":
a case asserts an order exactly,
never after a sort of its own.

## B343: a second test file sorted away the order the recheck guarantees

Changed in `de6ff17`;
its red is the agent's measurement,
since the cases pass on the stage as it stands.

Found on 2026-10-06 (UTC),
the item B328 left open.
`refine-slice-settle.unit.test.ts` compared each settled slice's recheck readings through `readingsInSeatOrder`,
which sorted the ballots into seat order.
By the agent's mutant,
with the checker stage's roster ordering removed from `repair-edit-stages.ts`,
every case of the file still passed while `refine-recheck.unit.test.ts` failed six;
with the helper also removed,
the two cases whose round heard three checkers failed.
The third case hears no checker,
so it has no ballots to order.

The fix:
the three cases assert the readings as the stage returns them,
and `readingsInSeatOrder` is gone from `sunbathing-recheck.test-fixture.ts`.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B367: the bench draw decided between a skip and a refusal by which read ended first

Red in `bb2a8b9ad1e164974e54d2750432a25b267ef0de`,
fixed in `cfac5b981`.

Found on 2026-10-06 (UTC) by the review of the runner observations change,
its first finding.
The change rethrew a read failure inside the catch of a `Promise.all`,
so an entry whose original was absent and whose English page git could not produce
was skipped or refused depending on which read ended first.
The port kept main's `textOf`,
which had no rethrow,
and two of main's cases named the skip of an entry unreadable for another reason.
By the agent's run on the port build,
five cases fail,
each because the draw returned where it should refuse:
two new cases scripting the later read failing first,
within one entry and across two;
main's two cases,
renamed to refuse;
and the change's own case on a lost object.

The fix:
`textOf`'s catch rethrows every read failure except `isMissingCorpusObject`,
failures that are no `CorpusReadError` among them,
decided at the read,
and `allInInputOrder` reports the first in input order,
so which read ended first no longer decides a skip or a refusal.
Parse and alignment failures stay skips,
since they happen in `stepOf`.
The TSDoc and `@throws` of `textOf`,
`sliceEntry`,
`sliceListedEntries` and `sampleBenchSlices` say so.

Open to the owner's veto:
The draw refuses every read failure except a page absent at a commit the clone holds,
faults that are no corpus read refusal included;
the alternative is the narrowing of the change's second report,
which rethrew a `CorpusReadError` alone and skipped any other failure.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B388: two runners logged per-entry lines in completion order

Red in `3fa45b1377073e74042d66fdb033f2533abbb7cc`,
fixed in `9d558da592162f9765fb03a2280af9477ada147d`.

Found on 2026-10-10 (UTC) by the agent working the runner leftovers,
in its census of combinator calls in `src/corpus-run` (moved on 2026-10-18 to
`package/translation-repair/*/src/corpus-run`):
48 sites,
five of them logging from inside a member.
The census reads only function bodies written inline in a combinator's arguments,
so a member passed by name is out of its sight,
and the order of log lines was checked only at the two sites fixed.
`probeDisplacement` (`corpus-run/displacement-probe-run.ts`) logged its per-entry lines
from inside its member of `allInInputOrder`,
and `withCitedReferences` (`corpus-run/rendering-audit-settled-buy.ts`) logged each page's CITED REFERENCES line
from inside its read,
so one input gave its lines in whichever order the members ended.
By the agent's probes against the base build,
with a print added to each test file and the file restored after,
the displacement probe logged Tabby before Mittens,
and the settled audit logged tabby's `characters=64` before mittens' `characters=77`;
the cases
"LOGS EACH ENTRY IT SKIPS IN THE ORDER THE ENTRIES ARE LISTED when the later entry's carve ends first,
so two walks over one archive log the same lines"
and
"LOGS WHAT EACH PAGE CITES IN BUYING ORDER when the later page's read ends first,
so two runs over one archive log the same lines"
fail there.

The fix:
the members return what they found,
and each runner logs after `allInInputOrder` has settled,
in the order of its input.

Left by the agent,
each with its reason:
`editor-calibrate-drive.ts` and `pass-insertion-admission.ts` print progress lines,
which follow completion by design;
and `entry-pictures.ts` reads through nothing a case can script,
so no case can order its reads and none can be red.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order" and "Searches and censuses that miss".

## B392: the editor standing read printed its stderr lines in whichever order its reads ended

Red in `cb553634188266b72019e60b6bd285d5746b0d32`,
fixed in `752be22cf8f4d875333bf1d54efdb90955d619a1`.

Found on 2026-10-10 (UTC) by the agent working the runner entry tails,
while moving the printer out of `corpus-run/editor-standing-read.ts`,
an open item of the T8 passage "The runner and git cluster of the final census" and of B379's veto.
The entry file held the whole printer and set its own exit code,
605 lines.
The listing of every named directory and the reading of every artifact ran side by side under `Promise.all`,
and each member printed its "no artifacts under",
off-roster or refused line from inside itself,
as it found it,
so one set of archives gave its stderr lines in completion order.
The case that holds it is
"SAYS EACH REFUSED ARTIFACT IN THE ORDER ITS ARCHIVE WAS NAMED when the later archive's read ends first,
so two runs over the same archives say the same lines",
which puts a 32 MB refused artifact in the archive named first and an empty one in the archive named second;
by the agent's measurement against the unmoved printer,
the second's line printed first.
At the base the case's file,
`corpus-run/editor-standing-read-run.unit.test.ts`,
cannot load,
since it imports a module the fix adds;
the order's red rests on that one measurement of a slow read,
and the fix does not depend on timing.
Before the move,
the agent tightened the as-built suite to pin whole outputs;
it passed against the unmoved printer and again after the move,
so the move changed no byte,
by the agent's account.

The fix:
the entry file is 43 lines of wiring,
and the printer is four role modules:
`corpus-run/editor-standing-read-listing.ts`,
`corpus-run/editor-standing-read-artifact.ts`,
`corpus-run/editor-standing-read-seat.ts` and `corpus-run/editor-standing-read-run.ts`,
whose `reportStandings` returns `NOTHING_RECORDED` (1) or `STANDINGS_READ` (0),
and the entry sets `process.exitCode` from it.
The listing and the reader return what they have to say (`said`),
and `reportStandings` prints every line once all reads settle,
in the order the directories were named and their artifacts listed.
The listing's "no artifacts under" lines have no red of their own,
since two failed listings end together;
the run case
"SAYS WHY EACH NAMED DIRECTORY COULD NOT BE LISTED,
in the order they were named,
and returns 1 over no artifact"
pins their order.
The editor runner barrel names the new modules;
the key in `caught-value-text.unit.test.ts` and two entries of `promise-combinators-in-order.unit.test.ts` follow them.

Open to the owner's veto:
`resolveRuns` is called only when the command line names no directory,
so git is never asked when directories are named,
as at the base;
the alternative is to resolve it at the start.
A kind-union outcome carrying `said` replaces the string sentinels and the `ARTIFACT_ABSENCES` record;
the report names no alternative.
The summary still prints `archives=0` when the runs directory is read by default,
byte for byte as before;
the alternative counts that directory as one archive.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B414: the cited-reference and work-title lookups wrote their per-member lines in completion order

Red `142ed0e987`,
fixed in `6c39c45556`.

Found on 2026-10-11 (UTC) as the open item B410 recorded for `citedReferenceBlock`,
and beside it for `workTitleLookupLines`.
Each lookup wrote its `REFERENCE` line,
its page that could not be fetched
or its failed lookup
from inside its member,
as the member ended,
and the cited-reference case hid it by sorting.
With a later page or title ending first,
both fail red on the base build.

The fix:
no member rejects;
each returns a named outcome (`PageOutcome`,
`TitleOutcome`),
the caller's reason as a stop;
once every lookup has ended,
the lines are written in input order from what each returned,
and the first stop's reason is thrown unchanged;
the cases assert the order exactly;
the scan entries now read `catches-own`,
resting on `failedPageOutcome` and `failedTitleOutcome`.
B409's stop case needs no wait for the line any more.

Open to the owner's veto:
a stopped call now waits for every lookup before refusing,
where the alternative refuses at the first lookup that ends in the reason
and gives up the input order of a stopped call's lines.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B631: tests made independent of the rotation lost a guard each had

Found on 2026-10-15 by the review of n162 (`s233/review.md`,
finding Z3),
by reading the diffs of the tests that n161 made independent of the rotation (B627).

Outcome written on 2026-10-18.
Red `555bb74aa0`,
fixed in `29d6064708`;
the fix is the round n166.

What the review found:
the new sheet text moved the bench rotation,
and n161 made ten tests independent of the order of seats.
Six of them lost more than the order:
- `src/corpus-run/judge-fidelity-probe-run.unit.test.ts` pinned the exact text of the row files,
  the kept file and the written file,
  including the indent of two and a trailing newline;
  its new helper parsed both sides and wrote them out again,
  so a writer that dropped the indent or the newline would pass.
- `src/corpus-run/probe-verify-run.unit.test.ts` did the same to `probe-verify-manifest.json`
  and the output of `formatVerifyManifest`.
- `src/corpus-run/audit-sensitivity-print.unit.test.ts` took the expected order of voices from the function under test,
  so the report could list them in a wrong order and the test pass.
- `src/corpus-run/editor-calibrate-lane.unit.test.ts` and `src/corpus-run/editor-calibrate-run.unit.test.ts`
  accepted any six distinct seats of the seven that can edit,
  so a lane that seated a worse subset would pass.
- `src/deep-reply-floors.unit.test.ts` looked up two named voices with `find`,
  so an extra or duplicate entry in the slate no longer failed.

The other rotation tests sort both sides by model id and stay attached to the voice,
so they still fail when a ballot's content,
weight or self-vote breaks.
That was read in the diffs and not mutated.

Fixed by n166.
- The two format tests pin the on-disk text again,
  through a helper `formatterOutputOf`:
  every row file and the kept run are the formatter's own text with an indent of two and no newline,
  and the rows document written to the output stream ends in one newline.
  The manifest and the formatter output re-serialize to themselves.
- The audit test derives the asked order by the documented rule,
  `rotatedBench` over the roster's checker models and the rendered messages,
  and a new test says the rows of the report equal it.
  The doc of `rows` in `src/rendering-audit.ts` now says the order is the rotated bench order.
- The deep reply test compares the whole slate,
  sorted by model id,
  and requires exactly two entries.
- The editor lane and run tests pin the six old seats again by injection,
  and not by the prompt:
  `StageSettings.editorBenchStart` (an optional number,
  read in `src/repair-editor-stage.ts`)
  goes through `gatherStageVoices` and `rotatedBench` as their `start`,
  and `runEditorCalibrate` forwards it.
  No production launch gives it,
  so production still rotates by prompt.
  The new fixture `src/corpus-run/editor-bench-start.test-fixture.ts` exports `PINNED_EDITOR_BENCH_START` (5),
  which seats the roster indices 0,
  1,
  5,
  6,
  7 and 8.
  The fixture constant `SEATS_THAT_CAN_EDIT` became unread and the dead-declarations scan found it,
  so it is removed.

Test:
the guards were shown to fail by removal (`s237/mut-d.log`,
`s237/mut-e.log` and `s237/mut-f.log`).
- Ignoring `editorBenchStart` in `src/repair-editor-stage.ts` turns the lane and run tests red.
- Ignoring `benchStart` in `src/stage-quorum.ts` turns the lane,
  the run and the quorum tests red.
- Ignoring `start` in `rotatedBench` turns the window and quorum tests red.
- Reversing the model ids of the audit,
  a manifest or row file written with an indent of four,
  and a duplicate entry in the deep-reply validity turn the matching tests red.

Which reds are real regression tests:
the seat pins of the lane and run tests and the tests of the seam (window and quorum)
were red against the build the round started from.
The audit order,
the two formats and the deep reply floors were green there and are red only by removal of the guard,
as is the empty-base envelope control of B635.
Seven further tests were red in the first whole run for a reason unrelated to any guard:
they pin cache keys,
a version number or the order of seats,
which the sheet text of B637 and its six cache versions moved.
The literals and versions were updated,
and the two tests that named three translator seats in a fixed order now compare the seat lines in text order.

Recurrence:
`mistake-prevention-tests-clocks-and-arrival-order.md`,
"Orders and seats that a prompt hash decides".

## B701: tests that name one seat asked broke whenever a sheet's text changed (the seat-window family)

Status:
fixed on 2026-10-20 (UTC),
and the census is closed.
Two members were fixed first:
the consolidate deep-reply case by `20a379f77a`,
and the corpus-pass contest re-seat case by `a169580740`.
The census of the other scripted cases that name a seat or an order is integrated:
nine members across `stage`,
`corpus-pass`,
`probe` and `translate-stage`,
fixed by `38b8c172b8` (five test-only pins) and `1305066161` (four seams that now take `fanOut`),
with an automatic merge repaired in `1001a94d91`.
Gate g120 on `1001a94d91` ran green,
with 2561 pass and 0 fail,
and the three commits landed in `dc4982c9cb` over `main` at `86fec6f872`.
The guard against a next member was decided and built on 2026-10-21 (UTC):
the rotation sweep,
`mise run //package/translation-repair/source-scan:test:rotation-sweep` (the paragraph "Guard" of this entry).
No production caller passes `fanOut`,
so production behaviour is unchanged,
and no cache version constant moved.

The first round of a stage asks a window of quorum plus one seats from the bench,
rotated by a hash of the prompt (`package/translation-repair/stage/src/stage-fanout-window.ts`).
Any edit to a sheet moves the hash,
and with it which seats a scripted case reaches.
A case that needs one named seat asked therefore passes only while the current sheet text rotates that seat
into the window.
The reader-protection scope of B697 lengthened the house rule that every sheet splices,
and two such cases failed for a reason that had nothing to do with what they test.

Consolidate,
`deep-reply-floors.unit.test.ts`:
the case withholds the deep reviewer's 300-marker revision and expects the floor to refuse it.
With the longer house rule the rotated window left the deep voice out,
no refusal was recorded,
and the case failed at its line 383 with nothing about the floor in the failure.
On `main` at `e262b18601` the same case passed only because the old house rule happened to rotate the deep voice
into the window.
`runArchiveBlockReviewStage` (archive package) now passes an optional `fanOut` to `gatherStageVoices`,
as `gateConsolidatedSlice` and the other stages already did,
and the case asks the whole bench.

Corpus-pass,
`pass-seams-reseat.unit.test.ts`:
the contest re-seat case asks whether a contest whose view never changes asks the seat a dry-out takes.
Gate g118 on `191a8bc38d` ran 2553 pass and 3 fail,
the failing package corpus-pass alone:
the control no longer asked the lost seat,
since the reader-protection scope had changed the house rule every contest sheet splices.
`runPassContest` now takes an optional `fanOut` it hands the contest as it stands,
and the case passes the whole bench the way the consolidate case does.

The census:
an agent ran it in a scratch worktree at `20a379f77a`,
to find every scripted case that expects a specific seat or order from a window the sheet text rotates.
Its method was a probe,
not a reading of the tests.
A scratch edit of `benchRotation` read a salt from the environment and shifted the rotation start by 0 to 9 seats,
and the unit suite of every family package ran under each of the ten salts.
Ten salts cover every start of the largest bench in the family,
which holds nine seats,
so they stand for every wording a sheet can hash to.
The edit was reverted before any commit,
and the later sweep did the same through a scratch loader that rewrites `benchRotation` as its module loads,
so no production source reads the environment.
Only `stage`,
`corpus-pass`,
`probe` and `translate-stage` held a failing case of their own.
The `bench` and `coverage-census` suites also failed in the first sweep,
on a line the first loader printed to the stderr of the built commands,
which their cases compare (the paragraph "Open" of this entry).
Salt 0 shifts nothing,
so a case red under it is red with no probe:
the contest case of `pass-seams-reseat.unit.test.ts` was already red on `20a379f77a`.

The five test-only pins (`38b8c172b8`,
no production code):
- `stage-quorum.unit.test.ts`,
  "counts exactly half of an even roster as quorum":
  red under salts 2,
  3,
  6 and 7;
  the window held both healthy seats and left a silent seat unasked.
  It now asks the whole bench.
- `stage-windowed-rounds.unit.test.ts`,
  the case that counts the bench seats it may not ask as out of reach:
  red under salt 5,
  since a refused seat outside the window is never seen refused.
  It now asks the whole bench.
- `pass-consistency-ask.unit.test.ts`,
  "ASKS EVERY SEAT AND HANDS BACK THE BALLOTS IN ROSTER ORDER":
  red under salts 2,
  5 and 8;
  the control's gather hands voices back rotated.
  The control's gather now starts at the second seat.
- `pass-consistency-carry-run-off.unit.test.ts`,
  "RUNS OFF A LONE LEADER WITH THE CANDIDATES NEXT BEHIND IT":
  red under salts 1,
  2,
  3,
  7,
  8 and 9;
  the scene hands out its split in the order the asks arrive,
  and the record lists ballots in roster order.
  The case is about the split,
  so it compares the choices sorted.
- `audit-sensitivity-print.unit.test.ts` in `probe`,
  "PRINTS a DEGRADED line for each finding when a voice is lost":
  red under salts 1,
  4 and 7;
  it named the heard voices in roster order,
  while the report lists them in the order the clean rendering's prompt rotates the bench to.
  The order is now derived from that prompt.

The four seams that now take `fanOut` (`1305066161`):
- `pass-seams-reseat.unit.test.ts`,
  the contest case:
  red under salts 0,
  6,
  7 and 8.
- The same file,
  the insertion case:
  red under salts 4,
  5 and 6.
- The same file,
  the preparation case:
  red under salts 1,
  2 and 3.
- `translate-deep-reply.unit.test.ts`,
  the case that finishes on the voices that wrote ordinary renderings:
  red under salts 3,
  4 and 9.
  The one self-vote the findings record belongs to a judge the window of four of six may leave out.

`runTranslateStage`,
`runTranslateRepairs`,
`produceTranslateSlate` and `judgeTranslateSlate` take an optional `fanOut`
for the translate stage;
`prepareDocumentPairWithRoster`,
`buySectionPairing` and `prepareBlockPairing` take it for the pairing preparation;
`runPassContest`,
`admitPassInsertions` and `runPassPreparation` take it for the corpus pass.
It is passed with the conditional spread the sibling stages use.
After the change all ten salts pass for every one of these cases and for every other test of the three packages.

The merge that duplicated `fanOut`:
the two census commits (`514d4f0038` and `f5cfe42cf1` in the scratch worktree) were written on `20a379f77a`,
before `a169580740` gave `runPassContest` an optional `fanOut` and made the re-seat contest case pass
`fanOut: 'whole-bench'`.
Picked onto `c3f3ea5532` as `38b8c172b8` and `1305066161`,
the automatic merge reported no conflict and kept both copies:
two `@param fanOut` lines,
two destructured bindings,
two `fanOut` fields in the parameter type
and two conditional spreads in `pass-contest.ts`,
and two `fanOut` keys in the options of the contest case.
Git merged two identical additions placed at different lines without asking.
`1001a94d91` keeps the copy already on `main` in both files,
so `pass-contest.ts` is as on `main` and the contest case passes the whole bench once.

Guard:
the rotation sweep,
decided by the main session and built on 2026-10-21 (UTC) as an explicitly invoked mise task.
The other shape proposed in this entry,
a test-only override seam in `stage` like `StageSettings.editorBenchStart`,
was not chosen:
it needs no loader and no bundle shape,
but it adds a setting to production code that only tests use.

Running it:
`mise run //package/translation-repair/source-scan:test:rotation-sweep`,
narrowed with `-- --package <directory>` and `-- --salt <n>`,
each repeatable.
The task builds every package that can reach `stage` through its manifest's dependencies,
then runs each of their unit test files once per salt from 0 to 9,
in the package's directory and with no key and no setting of the package.
At its first run that was 26 packages and 1152 test files;
`source-scan` itself is left out,
since its tests read source and docs and ask no seat.
It exits non-zero if any salt fails a case that salt 0 passes,
and lists each such case once,
with the test file,
the tags the harness logged for the case and the salts that fail it.
A file holding such a case runs again under that salt,
and a case that fails only once is listed as a flake and does not fail the sweep.
A case that is red under salt 0 is listed as red with no shift and fails the sweep as well,
since it cannot be read against the salts and would hide its own dependence on the rotation.

How the salt is applied:
a loader,
`source-scan/src/rotation-sweep/rotation-salt-hooks.test-fixture.ts`,
registered with `module.registerHooks` and imported with `node --import`
(synchronous in-thread hooks,
stability 1.2,
release candidate,
in the Node documentation read on 2026-10-21 (UTC)
for Node 26.12.0).
The salt travels in the loader's own URL (`...rotation-salt-hooks.test-fixture.ts?salt=3`),
so no production source reads the environment,
and the loader writes nothing to the standard output or standard error of the process under test.
The hook renames the declaration `function benchRotation(` to `benchRotationUnsalted`
and appends a wrapper under the old name that answers `(unsalted + salt) % size`,
and zero for an empty bench.
It does not read the body,
so it serves the minified bundle (`...,2166136261)%size}`) and the source (`return hash % size;`) alike,
and it refuses a module that declares the function more than once.
The built `stage` is one file (`dist/final/node/index.mjs`) with the family's siblings left as imports,
so every package's tests reach the same rewritten module by the package name;
the `coverage-census` bundle holds a copy of its own,
which the hook rewrites as well.
The sweep passed the `bench` and `coverage-census` suites under every salt,
which compare the standard error of built commands,
so the loader added nothing to it.

What keeps the sweep honest:
the pitfall the first loader showed,
a rewrite that matches a bundle shape and leaves the salt doing nothing,
is closed by a canary.
Before any suite runs,
the sweep asks the built `stage` for the rotation of three prompts over every bench size from one to nine,
with no shift and under each salt to be run,
and stops unless each answer moved by exactly the salt modulo its bench.
A bundle shape the loader no longer matches therefore stops the sweep and cannot pass it.
Positive controls were run on the finished task on 2026-10-21 (UTC),
each by taking one pin out of a test and putting it back:
- `stage-quorum.unit.test.ts`,
  "counts exactly half of an even roster as quorum",
  without the `fanOut: 'whole-bench'` of `38b8c172b8`:
  the sweep reported `FAIL under salts 2, 3, 6, 7`
  with the case's file and tags,
  the salts this entry names for the case,
  no other salt failed it,
  and the task exited 1.
- `pass-seams-reseat.unit.test.ts` in `corpus-pass`,
  "THE PREPARATION SEAM WIRES ITS HOOK",
  without the `fanOut: 'whole-bench'` of `1305066161`:
  `FAIL under salts 1, 2, 3`,
  the salts this entry names for the case,
  which shows the hook reaches `stage` through another package's tests.
  The two controls ran together as `-- --package stage --package corpus-pass`,
  over 132 test files.
- The contest case of the same file,
  without its `fanOut`,
  was red under salt 0 itself
  (the first sweep that ran it listed it as red with no shift and exited 0,
  which is why a case red with no shift now fails the sweep).
After both pins were put back,
the full sweep ran clean:
1152 test files of 26 packages under salts 0 to 9,
no case failing under a salt that salt 0 passes,
none red with no shift,
no flake,
and the line `rotation sweep: clean` with exit 0.

Cost:
one full sweep (ten salts,
the builds included) took 824 s on 2026-10-21 (UTC),
809 s of it in the task after the builds,
and each shifted salt took 77 to 86 s over the 1152 test files with 28 files at once.
It is one measurement on a host shared with other agents,
whose load average read above 12 throughout and up to 41 at the end,
so it is an order of magnitude and not a figure.
A run narrowed to one package is much cheaper:
`-- --package stage` took 75 s with the builds,
about 1 s for each salt after them.
The cost is why the sweep is a task run on request and not part of `test:unit`.

When it must run:
before a commit that changes the text of a sheet lands
(a prompt,
a house rule or floor wording that a sheet splices,
or a scripted case that names a seat),
and in the pre-launch checks of `mistake-prevention.md` ("Before a run launches",
step 13).
It is not run on every commit.
A case it lists is made independent of the rotation as "Orders and seats that a prompt hash decides" says
(`fanOut: 'whole-bench'`,
a pinned start,
or a comparison by seat),
and the sweep is run again.

Limits:
ten salts cover the starts of a bench of up to nine seats,
the largest in the family,
and a larger bench needs more salts and a larger bound in the canary
(`SALT_COUNT` in `rotation-sweep.test-fixture.ts` and `LARGEST_BENCH` in `rotation-sweep-canary.test-fixture.ts`).
The sweep reads only the packages that reach `stage` through their manifests,
and a case that depends on the rotation through anything other than `benchRotation`
(a seat chosen by another hash,
a bench of another stage) is not moved by it.
Nothing yet checks that the bound follows the roster;
that stays open.

Recurrence:
B627 and B631,
"Orders and seats that a prompt hash decides" in `mistake-prevention-tests-clocks-and-arrival-order.md`.
Those rules pinned a seat set by an injected start of the rotation
(`StageSettings.editorBenchStart`) for the editor seats;
the first-round windows of the other stages had no such seam,
and two seams did not take `fanOut`.
The prevention rule now names `fanOut: 'whole-bench'` as the second way.
The merge that duplicated `fanOut` is a second occurrence of the unconflicted merge that held a rule twice
(M238),
and "Work merged from parallel briefs" in `mistake-prevention-shell-edits-commits-and-agents.md`
now covers commits picked onto a tree that already carries an overlapping change.

## B734: the composite cases numbered their candidates by a rotation the sheet text seeds

Status:
fixed on 2026-10-21 (UTC),
with the fix of B732 (`e5bc27f9f0`),
source scans `2e5c2f82c1`.
A member of the family of B701,
found when the reply instruction of the archive block review changed.
No cache version moved on its account.

What the code did:
`archive-composite-revision.unit.test.ts` (B712) asks the whole bench with `fanOut: 'whole-bench'`
but not a pinned start of the rotation,
so the order in which the voices are heard,
and with it the number each revision has on the slate and which contributors a composite names,
followed a hash of the sheet text.
The reply instruction of the archive block review changed with B732,
the hash moved,
and five cases failed on candidate numbers and on the log line that names the winner.

The fix:
`runArchiveBlockReviewStage` takes `benchStart` and hands it to the gather,
which already took it (the stage did not pass it,
as it once did not pass `fanOut`,
see B701),
and the cases pin it to the first seat.

Why the census missed it:
the census of B701 ran on 2026-10-20 (UTC),
and the composite cases were added on 2026-10-21 (UTC).
The rotation sweep that would find a case like this one (`source-scan:test:rotation-sweep`) was not run in this change:
it runs the family's unit tests ten times,
which the ban on repeated full test runs forbids to a session without the lead's say.
Every package that renders a changed sheet ran its unit tests once,
and they pass on the new text,
which says nothing about the other nine rotations.

Open:
the rotation sweep before the change lands,
as `mistake-prevention-tests-clocks-and-arrival-order.md` says for a change to the text of a sheet.

## B812: the run-off line cases asked a rotated window of the bench and read a seat the window left out

Status:
fixed on 2026-10-22 (UTC),
fix `b8b91e2d7c`,
cache versions none.
A member of the family of B701,
found by the rotation sweep.

What the sweep showed:
`package/translation-repair/translate-stage/src/translate-runoff-line.unit.test.ts`
failed under rotation salts 2 and 6 of
`mise run //package/translation-repair/source-scan:test:rotation-sweep`,
with `expected 'judged' to equal 'declined-indecision'` at lines 328 and 349.
It passed under the other salts of the sweep.

The cause:
the two cases read the interested translator's own rendering and script each judge's ballot.
`produceTranslateSlate` was called with four translators and the default fan-out window,
so the first round asked quorum plus one seats (three of the four),
rotated by an FNV hash of the prompt (`stage/src/stage-fanout-window.ts`).
Under salts 2 and 6 the interested translator `CAT_A` was left out of the window,
its own word came back empty,
and the judges agreed where the cases expect a decline.
The scripted ballots named a seat the rotated window no longer asked,
which is the shape of B701.

The fix:
the slate and the judging of both cases pass `fanOut: 'whole-bench'`,
so every translator and every judge is asked and the hash decides nothing the case reads.
After the fix the translate-stage sweep was clean over salts 0 to 9.

Why the cases ran green until the sweep:
a plain unit run uses one start of the rotation,
and the cases pass under it.
Only the sweep,
which runs the family under ten shifts of the rotation,
reaches salts 2 and 6.
When the cases were added relative to the census of B701 was not traced.

Cache versions:
none moves.
The change is in a test file and adds no text a model is shown.

Recurrence:
`mistake-prevention-tests-clocks-and-arrival-order.md`,
"Orders and seats that a prompt hash decides":
a scripted case that reads a named seat's own output or ballot asks the whole bench.
