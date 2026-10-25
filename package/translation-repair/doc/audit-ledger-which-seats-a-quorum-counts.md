# Audit ledger: which seats a quorum counts

Quorums that counted seats which never answered,
could not answer,
or were withheld.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B27: a quorum counting seats the router refused (B21, family five)

Status:
fixed,
2026-09-29.
The short-bench rule
(`doc/decision/translation-repair-short-bench-share.md`)
sizes a gather's quorum on the seats a wet provider serves.
Family five asked where a threshold derived from a gather still counted the whole bench.

### The census

Every quorum-sizing line in the package's source was read
(`f5-sites.txt` in `~/temp/agent/audit-glossary-fix/`,
157 lines,
18 of them calls or mentions of `rosterQuorumSize` or `reachableQuorum`),
and each threshold was asked which seats its denominator counts.
Outside its own file,
`rosterQuorumSize` had 8 calls when the census read them,
and has 7 since the fix removed the archive block review's.
The gather itself (`stage-quorum.ts`),
the windowed rounds,
the select minimum
and the naturalness verdict rebuilt from its recorded basis
all take `reachableQuorum`.
The first round's window (`firstRoundWindow`) sizes a round before any seat is lost,
and `shortBenches` uses the whole bench's quorum to say what short means;
both count the whole bench by design.
Reference attestation takes its majority over the voices heard,
the vote tally over the ballots cast,
and insertion admission over the reachable seats coverage asked.
The checker bench width is a roster floor,
and the non-translation finding compares characters,
not seats.

### What was wrong

`runArchiveBlockReviewStage` held its anchored voices to `rosterQuorumSize` over the whole bench,
while the same function's outage test had taken `reachableQuorum` since the short-bench rule.
On a bench whose reachable seats fell short of the whole bench's quorum,
a block every reachable seat anchored in the original was left unresolved:
it skipped its naturalness read
and carried a finding naming the whole bench's quorum as the number required.
The archive's own text ships either way,
so the defect cost evidence rather than wording.

Two sentences claimed what a short bench cannot do.
The seat reader's wait line said it waits "rather than seating a bench that cannot settle",
and its function's TSDoc said the same;
the run client contract said "a phase that settles on nobody"
and named `run-seats.ts`,
from which the reading had moved to `run-seats-read.ts`.
Since the short-bench rule,
a short bench decides on a share of the seats a wet provider serves,
or is an outage below two of them.

### Exposure

Under `~/temp/agent/`,
1,404 stored logs and JSON files name the stage,
which is the control that the store is live,
and 10 carry the review's uncorroborated-claim finding,
which is the control that its findings reach stored files.
5 carry the unresolved finding:
2 are test output
(`audit-tests/it-blocks.json`,
`gfp-20260902/anchor-test.log`),
and 3 are replays or a probe from 2026-09-10
whose benches heard 7 of 11 seats,
so at least 7 were reachable and the quorum stays 6 under the fix.
No stored file outside the test output carries the review's short-bench finding.
The defect was latent.

### The fix

One `reachableQuorum` is computed from `gather.unreachable` before the outage test,
and both the outage test and the participation threshold read it.
The wait line now says "rather than seating a bench short of its quorum",
the TSDoc says when a bench the phase leans on would sit short of its quorum,
and the contract names `run-seats-read.ts` and `pass-reseat-hook.ts`
and says what a short bench decides.
Red `dd6d22874`,
fix `a3c83b333`,
boundary case `b968cbdc6`.

### Mutants and caches

The red case seats eight reviewers with five refused,
asserts from the exported helpers that the reachable seats fall below the whole bench's quorum
and meet the reachable share,
and asserts the review gather recorded the short bench.
The mutation batch (`b27-mutants.json`) caught ignoring the refusals,
but a boundary mutant turning the threshold's `<` into `<=` survived:
no case held anchored voices exactly at the quorum.
The boundary case has exactly the first quorum's worth of reviewers anchor,
and the rerun caught both mutants,
with the comment control surviving both times.
The archive block review keeps no cache (B26),
and the seat reader's line is a log.
The full suite passed on `b968cbdc6` with no failing case.

## B46: ballot guards that disagreed on whether a finding's shape costs a voice

Status:
fixed 2026-09-30 (UTC),
found reading the ballot-reply guards in T8's eighth batch.
Three guards decide whether a judge's reply is a ballot:
the lane contest's (`lane-contest-wire.ts`),
the consolidation gate's (`consolidate-gate-wire.ts`)
and the polish gate's (`consolidation-polish-gate-wire.ts`).
The lane contest refused a reply whose `unsupported` or `dropped` was missing or not a list of strings,
choice and reason readable or not.
The two gates kept a reply whose findings were anything at all,
reading a non-list as no findings,
but refused one with a findings key left out.
So a reply whose findings were `null` was a ballot at the gates and a lost voice at the contest,
and a reply with `dropped` left out was a lost voice at all three.

How they came apart:
`57e0ad7fa` (2026-08-20) stopped the lane contest's guard refusing findings over their wording,
after calibration lost two of its first sixty voices that way,
on the rule that the choice is what a contest counts and no wording of a finding may cost a voice;
it kept the shape check,
which its test pinned so that the reader would never receive a value it could not read.
consolidate-10
(`f83e9b449`,
2026-08-26,
`doc/audit/translation-repair-package-audit.md`)
carried the rule to type at the consolidation gate,
narrowing in the reader instead,
but left the gate's key-presence checks,
and neither change read the other guard.
The shape check was deliberate,
but for the reader's sake,
and a reader that narrows removes that reason.

Decided,
under the owner's standing quality directive and open to veto:
one rule for all three.
A reply is a ballot when its choice names a candidate and its reason is text;
findings left out,
`null`,
a bare phrase or a list carrying anything but strings read as no findings,
raw findings included.
`findingsOrNone` (`contest-ballot-wire.ts`) is the one reading,
and each reader takes both fields through it;
the lane contest's wire type now carries the findings,
and the archive answer its guard never checked,
as `unknown`.
Cases:
each guard's case in `consolidate-gate-wire.unit.test.ts`,
`lane-contest-wire.unit.test.ts` and `consolidation-polish-gate.unit.test.ts`
accepts a reply with a findings key left out;
the lane contest's case that pinned the refusal of a non-list now pins its acceptance;
a reader case each for the lane contest and the polish gate reads non-list findings as none,
choice kept;
`contest-ballot-wire.unit.test.ts` pins `findingsOrNone`.
Those cases were committed red first (`395567499`):
the three guard cases and the lane contest's non-list case failed against the build before the fix.
The change turns a lost voice into a kept one and never the reverse,
which changes whose voices a round hears,
so the accounts ride inside `LANE_CONTEST_CACHE_VERSION` 6 and `CONSOLIDATE_CACHE_VERSION` 20:
no slice-cache file was written after 04:27 UTC on 2026-09-27
under the agent runs or either worktree's runs directory,
copy payloads left out,
where a control from midnight that day finds 494.
The replies the runs stored under `prompt-payloads/` were not counted for such ballots:
they sit in many run roots,
and no count over them could show it covered every reply behind a cached ballot.

## B141: a refinement recheck nobody answered read as passed

Red in `9c2eb03d9`,
fixed in `e0151b92c`.

Found by the independent review of the trial's cases (report 4,
F1),
behind a case asserting `refine-recheck-passed` for a round in which no checker was heard.
`retainsResolvedIssues` rolled a rewrite back over an open issue only where a tally said `worse`,
and a round with no ballot has no tally saying anything,
so it returned `refine-recheck-passed` and the rewrite shipped.
It read neither `heardCheckers` nor the checker stage's findings,
so a round every checker answered `not-fixed` and a round nobody answered left one findings list.
On a bench of three,
the floor `assertCheckerQuorumReachable` holds a run to,
one checker heard of three passed the recheck by itself.
The stage's `stage-quorum-unmet (checker 0/3)` was dropped with the rest of its findings,
so the settlement passed `everyStageHeard` and was written to the refine cache.
(`everyStageHeard` is kept as written here and in the two accounts that name it.
It was replaced on 2026-10-22 by `refinementIsResumable` and `silentStagesOfSettlement` in `stage/src/stage-silence.ts`,
one predicate for the refine cache's write gate and its read gate,
and removed in `b423e15be8` (B820,
D36).)
A confirmed issue in such a round did roll back,
under `refine-rolled-back`,
which names it as an issue the rewrite broke,
and that was cached too.
Ledger L11 rules that a rewrite ships only after a checker round;
a round short of its quorum is none.

The fix:
the recheck reads the checker stage's findings through `silentStagesOf` (`stage-silence.ts`),
the reader the caches use,
and a round short of quorum rolls the slice back whatever its ballots say,
under `refine-recheck-unheard (<heard> of <configured> checkers heard)`,
decided before any issue is read,
so an outage is never recorded as an issue the rewrite broke.
The checker stage's findings now ride on all three verdicts,
as they do on the accuracy lane's outcome (`repair-chunk.ts`),
so a lost voice or a missing check shows on a pass and on a rollback,
and a round short of quorum keeps its settlement out of the cache.
Rolling back follows the package's gates between a new text and a standing one:
the consolidation gate and the polish gate each ship the standing text unless two ballots name the new one.
Decided for quality under the owner's standing directive and open to veto;
the alternative ships the rewrite under the unheard finding.
The gate moved to `refine-recheck.ts` for the file-length cap,
with its cases in `refine-recheck.unit.test.ts`.

Measured on the built package with a scripted bench of three:
with the unfixed gate,
none heard and one heard each gave `refine-recheck-passed (1 issue)`,
shipped the rewrite and passed `everyStageHeard`;
with the fix both roll back under `refine-recheck-unheard` and fail `everyStageHeard`.
(Replaced on 2026-10-22 by `refinementIsResumable`,
as noted in B141.)

Left as it was,
each shown by a probe:
a damage probe that hears no prober still ships the rewrite,
under `stage-quorum-unmet (introduced-defect-probe ...)` and uncached;
and a round that met its quorum counts an open issue no heard checker ruled on as passed,
now beside one `missing-check` finding a checker.

Cache:
no version moved.
A record written at refine version 5 by the unfixed gate could hold a rewrite shipped on a round short of quorum,
and would resume as it is;
none exists.
`cache-account-audit --runs-under` over the agent runs on 2026-10-05 counted 13,714 slice-cache records
under 391 runs directories,
the newest written at 04:26 UTC on 2026-09-27,
before every cache version's current value was set;
the account is in `REFINE_CACHE_VERSION`'s TSDoc (`refine-slice-key.ts`).

Recurrence:
`mistake-prevention.md`,
"Which seats a quorum counts":
a gate that passes on "nobody objected" first reads that enough voices answered,
through the stage's own account,
and carries that stage's findings with its verdict.

## B160: a damage probe that heard too few probers shipped the rewrite

Red in `779c90b3f`,
fixed in `86f7a2efa`.

Left open by the recheck fix of `e0151b92c`,
shown by its agent's probe.
`settleRefinedSlice` (`refine-slice-settle.ts`) rolled a rewrite back on the damage probe's report
only where the screen admitted a claim,
and a probe that heard no prober admits none,
so the rewrite shipped as probed.
Measured on the built package with a scripted bench of three probers:
none heard,
and one heard raising nothing,
each returned `refined: true` and shipped the rewrite,
with `stage-quorum-unmet (introduced-defect-probe 0/3)` or `(introduced-defect-probe 1/3)` among the findings
and no rollback finding.
The settlement failed `everyStageHeard`,
so it was never cached.
(Replaced on 2026-10-22 by `refinementIsResumable`,
as noted in B141.)

The fix:
`refineProbeVerdict` (`refine-probe-verdict.ts`) decides what the probe's report does to the rewrite.
An admitted claim rolls it back under `refine-rolled-back-by-probe`,
worded as before.
A round short of quorum,
read through `silentStagesOf`,
the reader the recheck and the caches use,
rolls it back under `refine-probe-unheard (<heard> of <configured> probers heard)`.
Claims are read first,
since the settler attaches no report to a rollback
and a short round named unheard would drop a claim the screen admitted from every finding;
the short round still shows by the probe stage's own `stage-quorum-unmet` beside it.
A probe outage now rolls back sound rewrites,
uncached,
so a later run buys the slice again;
the package's gates between a new text and a standing one fail closed.
Decided for quality under the owner's standing directive and open to veto;
the alternative reads the quorum first,
as the recheck does.
Cases in `refine-slice-settle.unit.test.ts`:
none heard,
one heard raising nothing,
one heard with an admitted claim,
and two heard,
which ships.

Cache:
no version moved;
a short probe round never reached the cache;
`cache-account-audit` found no slice-cache record under refine version 5.

Recurrence:
`mistake-prevention.md`,
"Which seats a quorum counts":
a gate that ships on "no objection" first reads that enough voices were heard,
through `stage-silence.ts`,
and reads positive evidence before silence wherever a rollback carries no report.

## B161: a recheck that met its quorum passed an issue it cast too few ballots on

Red in `779c90b3f`,
fixed in `86f7a2efa`.

Left open by the same fix.
`retainsResolvedIssues` (`refine-recheck.ts`) read the round's quorum off the checker stage's findings,
but read each issue's verdict from its tally whatever the number of ballots behind it.
A checker heard on the round may skip an issue,
leaving `missing-check (<n>)`.
B215 has since made that finding name the issue by its id,
the checker and the cause.
On a bench of three that met its quorum of two,
two checkers skipping the second of two open issues left one ballot on it,
and the round returned `refine-recheck-passed (2 issues)` and kept the rewrite.
A confirmed issue with one `fixed` ballot read as unresolved under `MIN_RESOLUTION_BALLOTS`
and was named under `refine-rolled-back` as one the rewrite broke.
On a five-seat bench the router left with two reachable seats,
closing on two,
one ballot on an issue passed too.

The fix:
an issue's count is its reading's ballots,
one per heard checker that ruled on it,
compared with the quorum the checker stage closed on:
the bench quorum,
or the quorum in the gather's `stage-short-bench` finding,
matched by rebuilding that finding with `shortBenchStageFinding` and `reachableQuorum` (`checkerQuorumClosedOn`).
B214 replaced that rebuild:
the gather returns the quorum it closed on,
and the recheck reads it.
An issue short of it is named `refine-recheck-unheard (<issue>: <n> ballots, quorum <q>)`,
whatever its ballots say,
and is never read as regressed or worsened;
a heard issue that broke is named beside it under `refine-rolled-back`.
Decided for quality under the owner's standing directive and open to veto.
Such a rollback carries no `stage-quorum-unmet`,
so it is cached as a rollback.
Cases in `refine-recheck.unit.test.ts` hold the count at one and at exactly the quorum,
on the bench of three and on the short bench of five.

Cache:
no version moved;
`cache-account-audit` found no slice-cache record under refine version 5,
the newest written at 04:26 UTC on 2026-09-27.

Recurrence:
`mistake-prevention.md`,
"Which seats a quorum counts":
a per-issue threshold on a gather's ballots takes the quorum the gather closed on.

## B214: the recheck rebuilt the checker round's quorum from the wording of a finding

Fixed in `2b0bfad75`,
with its cases,
which read a field the fix adds;
the fixing agent's red run read `undefined` where 2 and 3 were expected.

Found on 2026-10-05 (UTC)
by the agent closing what the refine-lane review left of B161.
`retainsResolvedIssues` (`refine-recheck.ts`) needs the quorum the checker round closed on,
and nothing carried that number but the wording of the gather's `stage-short-bench` finding.
B161 had read it there:
`checkerQuorumClosedOn` rebuilt the finding for every count of refused seats and looked for a match.

The fix:
`CheckerStageResult` gains `quorum`,
the recheck reads it,
and `checkerQuorumClosedOn` is deleted.
The agent's patch sized the number in `runCheckerStage` (`repair-edit-stages.ts`),
from the bench size and the seats the gather found unreachable,
since the gather's file was outside the files it was given.
The lead changed that at the merge:
`gatherStageVoices` (`stage-quorum.ts`) returns `quorum`,
the number its own `quorumMet` compared the heard voices with,
and `runCheckerStage` hands it on,
so the quorum is sized in one place.
The same commit corrected the TSDoc of the gather's `findings`,
which said the list is empty when quorum was met,
though the gather names each lost voice on a met quorum too.

Cases in `repair-edit-stages.unit.test.ts` hold the quorum of two for a bench of three,
of three for a bench of five,
of two for a bench of five whose last three seats no provider serves,
beside the short-bench finding naming the same number,
and of two for a bench of three with one seat refused.
The lead's case in `stage-quorum.unit.test.ts` holds six for a bench of eleven every seat of which answers
and two for the same bench with seven seats refused,
with the nine findings that bench leaves.

Cache:
no version moved;
a result gains a field,
and no cached record holds it.

Not checked by the agent:
every writer of an artifact,
for one that stores a checker stage's result whole;
its search found no shape guard that would meet the new field,
and the whole suite passed with it.

Recurrence:
`mistake-prevention.md`,
"Which seats a quorum counts":
a reader of a gather's round takes the quorum the gather returns,
never one read back out of a finding's wording or sized a second time.

## B298: a region no prober answered for was recorded as silent

Fixed in `097b6ac40`,
with its cases,
which the agent ran red in its worktree.

Found by the move B295 records.
`damage-sample` recorded a region no prober answered for as silent,
which is the human grade's negative reference.
A region nobody heard and a region every prober passed read the same on the sheet.

The fix:
such a region is printed as unheard and left off the sheet,
and a run in which no region is heard refuses.
A tally missing after a region was probed is an `unreachable:` throw.
The whole suite's run on `097b6ac40` is recorded in B295.

Open to the owner's veto:
an unheard region left off the sheet and counted in a printed line,
where refusing the whole run at the first one is the alternative;
one client opened for each region of the damage sample,
as before.

Recurrence:
`mistake-prevention.md`,
"Which seats a quorum counts":
no answer is never recorded as an answer.

## B420: a seat that threw on every call kept its place on every bench for the whole run

Red `2a049370cd`,
fixed in `06442b4689`.

Found on 2026-10-11 (UTC) in the read of TianqiChen66622,
where seats that threw on every call they were given were still asked by every later gather
and still counted in every quorum denominator,
so each gather waited on them and sized its quorum on a bench they could not fill.

The fix:
`seatTallyClient` (`seat-tally.ts`) takes a `minAsks` threshold
and refuses a seat the tally has withheld with `NoProviderForModelError` before the inner client is asked,
which every stage already reads as an unreachable seat;
a refusal is not an ask,
so it leaves the counts as they were.
The threshold is `SEAT_WITHHOLD_ASKS` in `stage-quorum.ts`,
`STAGE_RETRY_ROUNDS` plus one:
the most asks one gather gives a lost seat,
so a withheld seat has failed a whole gather's worth of asks rather than one round.
The quorum gather leaves withheld seats out of its bench and its denominator,
and each gather whose stage named a withheld seat warns which seats it did not ask and why.
The run's seat line names each withheld seat with how many of its asks threw.

Open to the owner's veto:
the threshold of four asks;
`runWindowedRounds`,
the judge benches and the floor checks do not filter withheld seats themselves,
and reach them only through the refusal.

## B421: a call the caller abandoned would have counted as the seat's failure

Red `2a049370cd`,
fixed in `06442b4689`.

Found on 2026-10-11 (UTC) in the lead's review of the B420 patch,
before it merged.
Every catch in `seatTallyClient` recorded `threw`,
so a call the caller aborted
(a straggler cut off after quorum,
a stage deadline,
an entry stop)
counted as the seat's own failure.
A slow but healthy seat cut off four times would then have been withheld for the whole run;
Kimi-K3 took 44 to 74 seconds to its first byte in TianqiChen66622,
which is that shape.

The fix:
a throw after the request's signal has aborted records nothing,
the treatment the tally already gave a call cut off when the process ends.
One case records nothing for four calls a caller aborted and that then threw,
and another withholds a seat whose four calls threw with the caller still waiting on each.

Revised on 2026-10-19 (UTC):
"records nothing" in the fix of this entry is replaced by "records abandoned and nothing else".
The call is counted in a column of its own,
`abandoned`,
which is part of neither `asked` nor `threw`,
so the protection stands:
a seat that its callers cut off is not withheld for it.
The `SEAT` line gains `abandoned=N`,
because the census of the pass `TianqiChen66626` read `asked=299 usable=299` and hid 58 cut replies (B687).

## B685: a rejected credential was named on no line, and `wet` did not mean a seat could be asked

Found in the read of the pass `TianqiChen66626`,
which launched at 20:58 UTC on 2026-10-18 (B671),
where the meter line read `bedrockUsd=5.02` while an expired Bedrock token failed five asks of each Gemma seat.
It is the first of three pipeline defects of that pass that the round `w191` fixed (B686 and B687 are the others).

Outcome written on 2026-10-19 (UTC).
Red `4e180b0761` (stage) and `f56acdc9b8` (run),
fixed in `05403f5176`.

What `wet` means:
`wet` is a `MeterState` (`provider/src/provider-meters.ts`),
and it says the meter answered and is not dry.
The Bedrock meter reads a local ledger (`credits` in `provider/src/bedrock-client.ts`),
so `bedrockUsd=5.02` is the money left,
and an expired token cannot move it.
The meter line is honest about money and says nothing about whether the seat can be reached.

Is a 401 a dead seat:
there is no rule for a 401.
The generic tally withholds a seat that threw on every one of `SEAT_WITHHOLD_ASKS` asks (4,
`stage/src/stage-quorum.ts`,
`stage/src/seat-tally.ts`,
B420,
open to the owner's veto).
By the notes' reading of the pass log (which this filing did not open),
five asks each were counted,
because the four `repair slice 0..3` critic rounds (lines 284 to 298 of the log) asked concurrently
after two or three sequential block-pairing asks per seat,
before the tally reached four.
The TSDoc of `seatTallyClient` agrees on the five asks.
After that the router refuses the seat before any provider is asked,
which every stage reads as an unreachable seat.

The quorum over 8 follows the recorded rule,
and no change was made.
`runWindowedRounds` (`stage/src/stage-windowed-rounds.ts`) calls `reachableQuorum`,
which keeps the quorum of the bench while the reachable seats can meet it.
The notes measured the built `stage`:
a bench of 8 needs 4 for 0 to 4 unreachable seats,
and needs 2 (short) at 5.
The log has 2 and 3 unreachable,
so 6 and 5 reachable,
both at least 4.
The rule is recorded in `doc/decision/translation-repair-short-bench-share.md` (the second addendum of 2026-09-27),
in `audit-ledger-defect-class-history.md` (E3) and in B27.
The asymmetry that the gather drops withheld seats from its bench,
and the windowed stages keep them and count them unreachable,
is the last paragraph of B420,
and stays open to the owner's veto.

Fixed:
`seatTallyClient` (`stage/src/seat-tally.ts`) writes one line the first time a provider answers a seat with HTTP 401:
`SEAT CREDENTIAL REFUSED <model>: <the failure and the provider's words>, after N usable answers this run.`,
then that the provider does not accept the credential the seat is asked with,
that a credential does not renew during a run,
that the run withholds the seat after four failed asks,
and "Renew the credential for this seat's provider before the next run."
The set of seats already named lives in the closure of `seatTallyClient`,
while the counts are process-wide (`RUN_SEATS`),
so the line is said once per seat and per client.
`createRunClient` builds one client.
`doc/configuration.md` describes the line.

First-contact check:
before this change no line named a rejected credential or said what to do.
By the notes' reading of the pass log,
what existed was the warn of each call (line 212 of the log,
one of 283 lines naming the failure,
the count that the TSDoc of `seatTallyClient` also gives)
and,
after the withholding,
`threw-withheld=` on the `JUDGE SEATS` line (line 9220),
which names the seat and its counts and not the error.
Only 401 is matched,
since 403 carries too many other meanings to name a credential.

Recurrence:
`mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"Which seats a quorum counts".

## B687: a reply the caller abandoned was counted in no column of the seat census

Found in the same read of `TianqiChen66626`,
and the third defect of the round `w191`.
It revises B421.

Outcome written on 2026-10-19 (UTC).
Red `4e180b0761` (stage) and `f56acdc9b8` (run),
fixed in `05403f5176`.

What the census did:
the report that began the round said abandoned Qwen replies were counted as usable.
They were not counted at all.
`seatTallyClient` returned early when the signal of the call had aborted (B421,
to stop a slow seat being withheld),
and `runGatherRound` cuts a straggler by aborting its signal (`stage/src/stage-round.ts`).
The round's check of the pass log (not repeated in this filing) found 298 Qwen `SPEND` lines,
298 `stream ... completed` lines,
58 abandoned replies,
and a census of `asked=299 usable=299`,
which said nothing of the 58.

Fixed:
`abandoned` is counted beside the other outcomes and in none of them,
so it adds to neither `asked` nor `threw`,
and the protection of B421 holds.
The `SEAT` line gains `abandoned=N`.

Did the cut replies spend their budget on reasoning:
the report said they had no answer text and 30,000 to 63,000 characters of reasoning.
They were cut by the straggler window of 120 seconds,
and not by `finish_reason=length`.
The notes measured the 58 lines:
55 have no answer characters and 3 have some,
and the reasoning runs from 18,470 to 62,639 characters (median 37,121).
By the same reading,
the 298 completed Qwen replies reach 16,788 completion tokens at most,
against the cap of 20,894 on the card (`shared-contract/src/model-cards.ts`),
and 45 of them carry 30,000 characters of reasoning or more,
so a reasoning length of that size completes normally.
The cut replies are slow reasoning runs caught mid-thought,
and no reply that reached the cap.
The shape the seat is known for is the reasoning runaway recorded in `shared-contract/src/completion-cap.ts`
(Qwen3.8-27B on Synthetic:
185 of 197 calls cut at the cap had no content)
and in `stage/src/recovery-nudge.ts`.
No seat setting changed,
since the owner's instruction of 2026-08-25 keeps thinking parameters off the wire.

The two corrections of the report's premise are M270.

Recurrence:
`mistake-prevention-messages-labels-and-counts.md`,
"Log lines that report nothing done".

## B711: a refusal the router gave before any request was counted as the seat's failed ask

Red `4508efa798`,
fixed in `965a5c9c67`.
It revises B420 and B421.

Found on 2026-10-21 (UTC) in the read of `zheermao20`,
and fixed in `w254`.

What the run showed:
`glm-5.3` is served by Charm Hyper alone,
and Hyper was out of budget from the start.
The end report read `SEAT glm-5.3 asked=4 usable=0 unusable=0 threw=4 abandoned=0`
though no request was ever sent.
Each of the four asks was the provider router refusing with `NoProviderForModelError`
("no provider can take glm-5.3:
every provider serving this model is out of budget,
seat unreachable").
The seat tally then withheld the seat for the run:
"the run withheld it after it threw on all 4 asks".
`hf:Qwen/Qwen3.8-27B` (Synthetic and Hyper) read `asked=158 usable=21 threw=137 abandoned=3`,
and nearly all of the 137 were router refusals after Synthetic ran dry at about 02:37 UTC.
Every `JUDGE SEATS phase=lanes` line from then on named a seat withheld as `threw-withheld=glm-5.3 (threw 4 of 4 asks)`,
and was followed by `chunk re-seated under a hold: checkers=xiaomi/mimo-v2.6-flash,inception/mercury-2.5,hf:Qwen/Qwen3.8-27B`.
Qwen was padding on that bench with no wet provider,
and neither line said so.

The rule the code broke:
the `seatTallyClient` TSDoc said a refusal "is not an ask,
so it leaves the counts as they were",
and `doc/decision/translation-repair-short-bench-share.md` says a seat no wet provider serves stays on the seated bench
and is lost at call time as unreachable.
That held for the tally's own withholding refusal,
which is thrown before the inner client is asked,
and not for the router's,
which comes out of the inner client.
`recordThrow` counted every throw of the inner client that was not an abort as `threw`,
the router's refusal included,
so the census,
the `SEATS DARK` line
and the `JUDGE SEATS` withholding all treated four refusals as four failed asks
(B420 withholds after `SEAT_WITHHOLD_ASKS` of four).
B421 had split the abort out of that count and left the refusal in it.

The fix:
a `NoProviderForModelError` thrown by the inner client is counted as the new outcome `unreachable`,
in neither `asked` nor `threw` and in no withholding threshold,
so `withheld({ minAsks })` stays "asked at least `minAsks` times and every ask threw" over real asks only.
The refusal is tested before the abort:
a refusal is a fact about the seat,
no request was sent for it,
and it counts as `unreachable` whether or not the caller had left.
The owner's cull (`culled-seat-guard.ts`) refuses a culled seat inside the tally with the same error,
so a culled seat is counted `unreachable` too and no longer reads as a seat that threw.
The `SEAT` line gains `unreachable=N` after `abandoned=N`,
and a seat that was only refused still gets its line (`asked=0 ... unreachable=N`).

Decided from the documents:
`dark()` keeps its meaning,
"asked at least once and never once produced a usable answer",
so a seat only the router refused is not dark.
Its `SEATS DARK` entry would say it was asked and produced nothing,
which is the claim B420 and the decision document reject.
Its evidence is the `unreachable` column of its `SEAT` line and the `unserved=` field of the `JUDGE SEATS` lines,
which name every seated seat no wet provider serves at the moment of the reading.
The `SEATS DARK` line states "of N seats asked" over the seats asked,
since a seat with only abandoned or refused calls now has a counter and no ask.
Its explanation no longer lists a provider that cannot serve the seat or a key never injected among the causes of a dark seat,
since the router refuses both before any request and they are counted `unreachable`
(the header comment of `seat-tally.ts` and `configuration.md` say the same;
red `5c6ec6aa49`,
fixed in `1c737df61b`).

Effect on a run:
a seat that only the router refuses is never withheld by the tally.
The router refuses it cheaply at each call,
the stages read the refusal as an unreachable seat,
and the seat comes back by itself when a provider is wet again.
The gathers therefore keep such a seat on the bench and size their quorum with `reachableQuorum`,
the way the decision document says,
where the tally had taken it off the bench after four refusals.
A seat that threw for another reason,
a rejected credential with another provider left or a model that errors,
is counted and withheld exactly as before.

The two logging gaps:
`judgeSeatsFor` returns `unserved`,
the seats on any bench of the reading that no wet provider serves,
and every `JUDGE SEATS` reading line prints `unserved=<ids or none>` after `withheld=`.
The repair lane's `chunk re-seated under a hold` line prints `unserved-checkers=<ids or none>`,
the checkers that are padding.
`lanesHooksFor` takes the optional logger the other hooks take,
so a case can read that line.

Guards:
`seat-tally.unit.test.ts` pins `unreachable` on the text,
JSON and typed surfaces with the seat refused seven times and never withheld,
a seat that throws for another reason withheld at four asks with refusals between them,
a refusal given after the caller left counting `unreachable` and any other throw after the abort staying `abandoned`,
the tally's own refusal counting nothing,
and the `SEAT` and `SEATS DARK` lines.
`run-seats.unit.test.ts` and `run-seats-read.unit.test.ts` pin `unserved` in the benches and the line,
and `pass-reseat.unit.test.ts` pins `unserved-checkers`.
The cases were red on `4508efa798` and green on `965a5c9c67`.

Cache versions:
no cache key or stored record reads a seat count or the benches' `unserved` list,
so no version moves.
The search is in the body of `965a5c9c67`.

Not done:
the readers of the `SEAT` line are the tests and the people reading the log,
and no parser reads it.
`audit-ledger-entries-by-id.md` stops at B701,
so B702 to B709 are not indexed there either.

Recurrence:
`mistake-prevention-messages-labels-and-counts.md`,
"Counts that include calls never made".

## B721: Gemma was listed as unable to read a picture on OpenRouter before anyone had tried

Red `7885284331`,
fixed in `a767785dd4`.

Found on 2026-10-21 (UTC) in the read of `XingZ6016`,
and fixed in `w258`.
The owner's ruling of the same day on the picture bench:
"Yes try gemma".

What the run showed:
the bench of picture readers had two readers that could be reached,
because Bedrock's key is never renewed and Gemma pictures reach it through Bedrock alone.
The OpenRouter side of `gemma-4-26b-a4b-it` said `readsImages: false`,
which `doc/decision/translation-repair-openrouter-fallback.md` gave as "gemma's row reports no pictures until a transcription is measured".
The public listing (fetched 2026-10-21) reports `input_modalities` image,
text and video for `google/gemma-4-26b-a4b-it`,
and its zero data retention endpoint listing names thirteen endpoints.
The only measurement of this seat's pictures,
nine pictures with a mean overlap of 0.668 on 2026-09-08,
was made through Bedrock.

The fix:
the card's OpenRouter side reports the listing's truth,
and the card carries the new hold `openrouter-pictures-unmeasured`.
`OPENROUTER_PICTURES_UNMEASURED` (`openrouter-catalog.ts`) is built from that hold,
and `openRouterShowsPictures` returns false for a seat in it,
so the vision reach stays Bedrock alone until the hold is lifted.
The model-wide `reader-unmeasured` was not used,
because it would also unseat the reader measured on Bedrock.
Every OpenRouter request keeps `provider.zdr: true`,
which `openrouter-picture-body.unit.test.ts` pins on a picture body to this seat,
beside no reasoning key and a numeric `max_tokens`.
The step that lifts the hold is B723.

Cache versions:
while the hold stands,
no roster that a picture cache key names changes.
Lifting it adds Gemma to the readers a wet OpenRouter reaches,
which changes the roster `imageReadingKey` hashes for every entry whose Bedrock is dry,
and the version bump belongs to the commit that lifts it.

Update of 2026-10-21 (UTC),
at the lift:
the claim of the cache-versions paragraph that the lift changes the roster `imageReadingKey` hashes,
and that a version bump belongs to the lifting commit,
was wrong,
and no version moved.
No cache version constant names the picture readers.
The roster the key hashes is the readers bench (`readers` of `judgeSeatsFor`),
which holds a reader whenever a wet provider serves it text,
so it already held `gemma-4-26b-a4b-it` while Bedrock was dry and OpenRouter wet,
and the lift does not change a key.
What it changes is whether that reader's ask can be sent to a provider that shows it a picture.
A stored reading says what the readers asked saw and stays valid.
The reading store opens under the digest of the built pipeline,
and the card is bundled into that build:
the `corpus-pass` build at the lifting commit digests to `sha256-tree-v1:b4eacc1f...`,
and the same tree with the hold put back digests to `sha256-tree-v1:96cc43b6...`.
The first pass on a new build therefore discards the store before it reads it
(`slice-cache-pairing.unit.test.ts` holds the mechanism for the pairing caches,
`openNamespacedCache` is the one opener of the picture store too).
`cache-account-audit`,
run against an empty scratch runs directory,
read eight constants,
none of them a picture reader's,
and lists the lifting commit among the commits that ride inside the standing accounts.

Open to the owner's veto:
a hold of its own,
not a card flag flipped back and forth,
and `google/gemma-4-26b-a4b-it` as the size tried.

Unchecked:
whether each of the thirteen endpoints accepts an image.
The measuring task reports the endpoint that served each call.

## B722: the readers bench counted a seat by the providers that serve it text

Red `4a93952e3e`,
fixed in `fce4252472`.

Found on 2026-10-21 (UTC) in the read of `XingZ6016`,
and fixed in `w258`.

What the run showed:
the `JUDGE SEATS` line printed `readers=5`,
and the readers quorum of two read as met,
though the seats were counted by `reachOf`,
the providers that serve the model text.
Gemma is served text by OpenRouter and shown pictures by Bedrock alone,
and Bedrock's token had expired.
Two readers could see.

The fix:
`reachableSeats` takes a reach (`text`,
the default,
or `vision`),
and `shortBenches` reads the `readers` bench with the vision reach (`visionReachOf`).
`judgeSeatsFor` returns `readersSeeing`,
the readers a wet provider can show a picture,
and the `JUDGE SEATS` line prints `seeing=N` beside `readers=N`.
The bench itself is unchanged,
so a reader with no wet picture route stays seated and is lost at call time,
as every unserved seat is.

Cache versions:
the bench and the roster a cache key names do not change.
Only the phase's wait and the quorum shortfall clause read the new count.

Recurrence:
`mistake-prevention-messages-labels-and-counts.md`,
"Counts that include calls never made".

## B723: the picture reader measurement was a scratch script outside the repository

Red `1f839c0fd4`,
fixed in `4a1fe70d3a` and `b7cf2b578f`.

Found on 2026-10-21 (UTC) in the work of B721.

What the run showed:
the rule that seated a picture reader on 2026-09-08 (each reading corroborated by every seated reader at least as often as the seated reader corroborated least often,
and uncorroborated by all no more often than the seated reader uncorroborated most often,
with empty,
cut and thrown calls counting for neither clause)
was applied by `bedrock-reader-measure.mjs`,
a scratch script that is not in the repository and ran through Bedrock.
Seating Gemma on OpenRouter needs the same measurement on that route.

The fix:
the keyed task `reader-measure` in the bench package.
It reads the corroborated and the disagreed picture readings the pipeline stored under the runs directories it is given,
matches each corpus picture to a stored record by the overlap of its deterministic reading,
asks the candidate through `readPastRefusal` (so a refusal is re-asked and a cut reply asked again as in a pass),
and scores the candidate against the seated band.
It builds the OpenRouter client itself,
because the router keeps pictures off the route while the hold stands.
It reports the verdict on each clause,
or "not measured" when fewer than `MINIMUM_PRODUCED_READINGS` (six) readings were produced,
and the one edit a pass calls for.
The command is
`mise run //package/translation-repair/bench:reader-measure -- --trusted <runs dir> --pictures <people dir>`,
with `--model` (default `gemma-4-26b-a4b-it`) and `--sample` (default nine) optional.
It needs `TRANSLATION_REPAIR_OPENROUTER_API_KEY` and was never run by the agent that wrote it.

Limits:
the stored records name no picture,
so the match rests on the overlap of the deterministic reading at the corroboration share,
the way the 2026-09-08 instrument matched,
and a short deterministic text can match a wrong record.
The seated band comes from the records of the matched pictures only.
Stored records are of corroborated pictures and of stable disagreements,
so the band's floor is high when most records are two agreeing readings.
`reader-measure-files.ts` walks directories through `namesOfKind`,
which takes neither a link nor a special file.

Open to the owner's veto:
a floor of six produced readings out of nine asked.
