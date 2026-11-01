# Audit ledger: failures under a caller's stop

Fan-outs that dropped,
mislabelled or doubled a member's own failure once the caller had stopped the work.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B352: a gather round dropped an ask's own failure under the caller's stop

Red in `d1e6af41c`,
fixed in `ce2654b2c`.

Found on 2026-10-06 (UTC) by the review of the input-order change's second pass.
`nextSettled` (`stage-round.ts`) throws the caller's reason in place of whichever ask rejected first,
and the ask's own failure,
a deadline or a transport failure landing as the stop arrived,
reached no line.
The case "LOGS THE FAILURE AN ASK ENDED IN AFTER THE CALLER'S ABORT" asserts the line whole,
and on the agent's tree before the fix it failed with `expected [] to deeply equal [ Array(1) ]`.

The fix:
where the caught value is not the caller's reason itself,
`nextSettled` writes it through `refusalText` on a logger tagged `nextSettled`,
then throws through `throwIfAborted()`;
`awaitHeard` hands it the stage and the logger,
and gains `@throws` for the logger's failure and for an ask's own failure when no abort is in play.
The list note and the TSDoc say the asks share the signal and the catch puts its reason in place of what it caught.
The case holding that nothing is logged when the asks end in the caller's own reason is coverage only;
by the agent's control it fails with the identity check removed.
The client failing two seats after the stop is extracted as `clientFailingAfterTheStop`,
which the existing case on the rejection now uses.

Open:
by the agent's run at its base `f3c2bae13`,
none of 21 test files that reach the round saw the new line;
on the merged tree the gate on `ce2654b2c` failed four cases the credential work added since that base,
each pinning the provider's words in the line a stop writes,
in `window-trial-pick`,
`repair-refine-step`,
`repair-slice-buy` and `translate-slice-attempt`.
By the lead's reading,
the round now throws the abort reason and logs the failure it replaces through `refusalText`,
so those words reach no line (M140).
The same four fail at `4cca8c301`,
and the change that fixes them is not merged there.
Closed since by B365 in `fba9c5b60`.

Open to the owner's veto:
The line is written only when the caught value is not the caller's reason,
where the review's fix logs whenever the caller has aborted.
The line is tagged with `nextSettled`,
while `askOnce` writes on its logger untagged.
Extracting `clientFailingAfterTheStop` changed the body of an existing case.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B365: a stopped gather round logged a refusal by its class alone, or not at all

Red in `154da3c68`,
fixed in `fba9c5b60`.

Found on 2026-10-06 (UTC) by the gate on `ce2654b2c`,
which failed four cases the credential work had added,
each pinning the provider's words in the line a stop writes (ledger B352).
`nextSettled` (`stage-round.ts`) threw the caller's reason in place of the ask's failure,
so the catch sites printed the reason alone,
and it logged the failure it replaced through `refusalText`,
which names the class and drops the status and the provider's words.
By the agent's probe on the build of `4cca8c301`,
whose production source is that of `ce2654b2c`,
the refine,
slice-buy,
translate and pick paths each wrote `refused by SyntheticHttpError` (renamed `ProviderHttpError` on 2026-10-19)
and nothing of the reply.
The cause went beyond the lead's reading:
the round logged only the rejection that ended first,
so a refusal ending after an ask that ended in the caller's own reason reached no line,
which lost the words before `ce2654b2c` too;
and a stop after quorum stood let the wait on the grace window swallow each failure unlogged,
in `settleWithin(Promise.allSettled(asks))`.
The red commit fails eight cases against the stage round of `ce2654b2c`,
run on `4cca8c301` with the production code unchanged:
the four gate cases,
renamed so each name states what it checks,
each asserting every line its call writes;
and four cases of `runGatherRound`,
each failing on the line it asserts whole:
a refusal ending before an ask that ended in the caller's reason,
a refusal ending after such an ask,
a refusal ending after quorum stood,
and the updated pin on the failure that ends after the round rejected.
The reason assertion of every case passes.

The fix:
`askOnce` writes the call's own failure when the caller has aborted and the failure is not the caller's reason,
through `exchangeFailureLogText` with the seat's name,
then rethrows as before;
`nextSettled` keeps only the substitution of the caller's reason,
and loses its stage and logger parameters,
as `awaitHeard` does.
The five catch sites are unchanged.
Each failure is written once,
by the ask that owns it,
as it ends,
whichever order the asks end in.
The new cases bound their rounds with the hang stop and wait one turn of the event loop.
Since the line moved to `askOnce`,
B352's choice of a line tagged `nextSettled` no longer describes the code.
Run on the merged tree before the fix commit:
lint clean,
49 scans passing,
and the suite at 2,028 passing with no failure.

Open:
`repairInvalidCandidates` (`translate-repair.ts`) reads its re-asks through `allInInputOrder`,
so when two invalid candidates' re-asks both fail as the caller stops,
only the first failure in input order reaches the catch site,
and the other reaches no line;
this is the agent's reading,
not run.
It waits for a GitHub issue or a later change.
Closed in B403,
which ran it.
`runPick` under a stop still prints its slice as refused by `AbortError` and returns `refused`,
unchanged from the base,
which tells a log reader the stop ended the arms.

Open to the owner's veto:
The translate case shows three lines of the provider's words,
one per seat,
because its scripted client throws the same refusal from every call,
where a real client forwards the already aborted reason after the first
(the agent's inference from `call-deadline.ts`,
not measured);
the alternative is a client in that case that throws `request.signal.reason` once its signal has aborted,
a change of one line.
Deduplicating the lines by the thrown object's identity was rejected,
since the seat a line names would then depend on arrival order.
The stage-round test file runs its top-level describe with `concurrency: 1`,
since its timing case failed in 2 of 4 runs beside the new cases and passed in 5 of 5 after,
a cause the agent infers,
with the base's spread from run to run unmeasured;
the report names no alternative.
Lines appear in the order the asks end,
so in production a seat's line can land after the catch site's line;
the report names no alternative.
The four gate cases were renamed so each name states what it checks,
and each asserts the whole list of lines;
the report names no alternative.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B403: a repair re-ask's own failure under the caller's stop reached no line

Red in `4cafeda7ee47f30bc19a1f110085dbe91d9bf3ac`,
fixed in `689064b25cf82602a3a1d2ed17a2e2c063cedb82`.

Found on 2026-10-10 (UTC) as the open item B365 recorded:
`repairInvalidCandidates` (`translate-repair.ts`) starts one `repairOneCandidate` per invalid candidate
and reads them through `allInInputOrder`,
and each calls `attemptStageCall`,
which rethrows a failure without writing it once the caller's signal has aborted.
So when two invalid candidates' re-asks both fail as the caller stops,
the call throws only the first failure in input order,
and a deadline or a provider's refusal the other seat's re-ask ended in reached no line.
B365 gave that as the agent's reading,
not run;
the red case runs it.
By the agent's census of production calls to `attemptStageCall`,
there are two:
the seat of a gather round in `stage-round.ts`,
which B365 covers by the line `askOnce` writes,
and this one,
which had none.

The red case,
in `translate-repair.unit.test.ts`,
sends back three candidates whose re-asks end as the caller stops:
the later-listed seat's first,
with a provider refusal carrying its words,
the first-listed seat's after it,
with another,
and a middle seat's in the caller's own reason.
It asserts one line per failing seat with that seat's own words,
no line for the seat that ended in the caller's reason,
and the first failure in input order as the thrown error.
Against the unchanged code it fails with the log holding three lines where five are expected.
By the agent's control,
removing the check that the failure is not the caller's reason fails the case with an extra line.

The fix:
`repairOneCandidate` catches the failure of its stage call,
and when the caller has aborted and the failure is not the caller's reason,
writes it on a `translate-repair` line naming the seat,
through `exchangeFailureLogText`,
as the stage round does,
then rethrows it unchanged.
The thrown error is the same as before.
Lines land in the order the re-asks end,
which the TSDoc `@throws` of both functions now says.

Open:
`readImagePair` (`image-reading-pair.ts`) reads its readers under `Promise.allSettled`
and calls `signal.throwIfAborted()` before `contained` logs a rejected settlement,
so a reader's own failure under a stop reaches no line;
the agent read it in the source,
and no case runs it.
Two lesser sites hold the same shape,
`fidelity-reference-read.ts` and `overlapped-map.ts`,
and are unread beyond that.
`gateProvider` writes the caller's own reason as a meter that could not be read,
and `meterRecordOf` names the failure's class alone,
dropping the provider's words.
Closed in B404 to B407,
which ran each;
`overlapped-map.ts` is the one site that stays open,
recorded in B405.

Open to the owner's veto:
the line reads "as the caller stopped the repair",
where the stage round's reads "as the caller stopped the round";
the stage label is the constant `REPAIR_STAGE`,
shared with the exchange's own lines;
the line carries the logger's `repairOneCandidate` tag,
where the round's `askOnce` line carries none;
the case waits one turn of the event loop for the later lines;
and a first failure on the translate path may be written twice,
once here and once by `translate-slice-attempt.ts`,
which is the agent's inference from reading both,
not run.

The double on the translate path was run on 2026-10-11 (UTC) and held;
B412 fixed it.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B404: a vision reader's own failure under the caller's stop reached no line

Red in `2a41fd28808237ce5a6441f41d3688ce9a2c8ded`,
fixed in `93850c5d3eb1bae2f7a0d023a731f24fa5b5650a`.

Found on 2026-10-11 (UTC) as the open item B403 recorded.
`readImagePair` (`image-reading-pair.ts`) reads its readers under `Promise.allSettled`
and called `signal.throwIfAborted()` before `contained`,
the one place a rejected settlement was written,
so a deadline or a provider's refusal a reader ended in as the caller stopped reached no line.
`readImageAsset` holds no `try`,
and `readPastRefusal` throws the caller's reason between asks.
Two red cases in `image-reading-pair.unit.test.ts` run it.
In the first,
three readers end as the caller stops:
the later-listed one first,
with a refusal and its words;
the first-listed one after it,
with another;
and a middle one in the caller's own reason.
In the second,
one reader is refused before the stop lands.
Against the unchanged code each fails with no line where the case expects one per failing reader.
By the agent's control,
removing the check that the failure is not the caller's reason fails both with an extra line.

The fix:
`contained` writes a failed reader's line through `exchangeFailureLogText`,
"failed outright (the failure),
and the caller stopped the reading",
when the caller has aborted and the failure is not the caller's reason,
and the abort guard now stands after `contained`,
so the thrown error is unchanged.
The lines follow roster order whichever reader ended first,
and a failure that ended before the stop is written too,
which a catch inside each reader would have missed.

Open to the owner's veto:
roster order,
where the round and the repair write in completion order at the member;
the guard moved after `contained`,
where a separate loop over the settlements needed a file split under the line limit;
and the line's wording.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B405: a pinned reference page's own failure under the caller's stop reached no line

Red and fixed in the same commits as B404.

Found on 2026-10-11 (UTC) as the open item B403 recorded.
`readReviewedFidelityReferences` (`fidelity-reference-read.ts`) reads a reference's two pages under `Promise.allSettled`
and threw the caller's reason in place of both,
though `readCorpusFile` takes no signal,
so either read's failure is always its own,
and the module held no logger.
Only the exported API reaches it:
`judge-fidelity-probe-run.ts` passes no signal.
The red case,
in `fidelity-reference-read.unit.test.ts`,
stops the read while both pages of a disposable clone fail as missing,
and fails with no line where two are expected.

The fix:
a root logger tagged `readReference` writes one line per failed page,
the source page's before the archive page's,
through `exchangeFailureLogText`,
when the caller has aborted and the failure is not the caller's reason,
then throws the reason as before.

`mapOverlapped` (`overlapped-map.ts`) was read too.
A job's exchange failure under a stop is written by the code the job calls,
by `askOnce`,
`repairOneCandidate`,
`readImagePair`,
and the abort catches of `translate-slice-attempt.ts`,
`repair-slice-buy.ts` and `repair-refine-step.ts`.

Open:
a job's failure of another kind,
such as `ConsolidationLedgerGapError`,
a hook,
a roster refusal or a cache write,
ending as the stop lands while a lower job throws the reason,
is dropped at the helper's throw with no line,
in the consolidation,
contest,
translate,
refine,
repair and insertion-admission lanes when the overlap exceeds one.
This is the agent's reading of the source,
not run.
The fix is the caller's signal threaded into `mapOverlapped`,
writing each failure it does not throw unless it is that signal's reason,
and nine caller modules outside the change would pass it.

Closed on 2026-10-11 (UTC) by B408,
which ran the reading red in six of the nine callers and found the other three unable to drop a failure.

Open to the owner's veto:
the module root logger,
where the alternative is a logger the probe passes;
the check against the caller's reason,
which no input reaches while the reads take no signal,
and which stays so the line remains right if a read starts honouring the signal;
and the line's wording.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B406: the census of fan-outs that read a caller's stop

Run on 2026-10-11 (UTC) as B403 asked of a fix of this shape.
Of 56 production files calling `Promise.all`,
`Promise.allSettled`,
`Promise.race`,
`allInInputOrder` or `mapOverlapped`,
36 hold no signal.
Of the 20 sites that do,
each member's own failure under a stop is written:
by `askOnce` for the gather round,
its race and the recovery round;
by `repairOneCandidate` for the repair;
by `meterRecordOf` and `gateProvider` for the meters;
and by the cited reference and work title lookups
(their catches then,
their returned outcomes since B414).
Two sites wrote none:
`readImagePair` (B404)
and the reference read (B405).
The settled buy passes a signal that never aborts.
`attemptStageCall` has two production callers,
`stage-round.ts` and `translate-repair.ts`,
as B403 counted,
and `attemptDecisionCall` has one,
`attemptStageCall` itself.
Every model exchange outside the clients and their decorators reaches one of those lines.

Open:
the `mapOverlapped` residual B405 records.
The lookups for cited references and work titles write the caller's own reason as a fetch failure
and answer it as data,
which is the shape B407 fixed in the gate;
this is the agent's inference from reading,
not run.

Closed on 2026-10-11 (UTC):
the `mapOverlapped` residual by B408,
and the lookups by B409,
which ran the inference and found it true.
The sentence on this entry's census that the lookups' catches write a member's own failure holds only for a failure that is not the caller's reason.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B407: the required-provider gate read the caller's stop as a meter failure, and the meter line dropped the provider's words

Red and fixed in the same commits as B404.

Found on 2026-10-11 (UTC) as the open item B403 recorded.
`gateProvider` (`corpus-run/required-providers.ts`) caught the caller's own reason,
wrote it as a meter that could not be read,
and refused the arm as `meter unavailable`.
By the red case,
with the stop landing while two meter reads ran,
it wrote "synthetic meter could not be read:
refused by Error"
and threw `RequiredProviderError` naming synthetic.
Only the exported API reaches it:
the pass's own call passes a signal that never aborts.
`meterRecordOf` (`provider-meters.ts`) wrote the failure's class alone,
"budget unreadable,
treating as spendable (SyntheticHttpError)" (renamed `ProviderHttpError` on 2026-10-19),
dropping the status and the provider's words,
on every meter read a run makes,
and wrote a stopped read the same way.

The fix:
`gateProvider` takes the signal
and rethrows the caller's reason unchanged and unwritten,
still writing a read's own failure with the provider's words;
`meterRecordOf` takes the signal,
writes a stopped read as "budget not read,
since the caller stopped the reading",
and renders any other failure through `exchangeFailureLogText`.
A stopped read still records `unreadable`,
since the reading is shared and its first caller's stop resolves it for every sharer (`takeReading`).

Open to the owner's veto:
the gate still refusing as `meter unavailable` when a meter fails of its own under a stop;
a stopped read recorded `unreadable` in the `METERS` line,
which rests on the TSDoc of `takeReading` and on no ledger ruling;
and the stop line's wording.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B408: an overlapped job's own failure under the caller's stop reached no line

Red `7781cbaaa7`,
fixed in `63a4a35f0a`.

Found on 2026-10-11 (UTC) as the open item B405 recorded.
`mapOverlapped` (`overlapped-map.ts`) kept each job's failure by position
and threw the lowest position's,
so once the caller stopped,
a higher job's own failure reached no line when a lower job ended in the caller's reason.
By the red cases on the build of `e241a7be6`,
the consolidation,
contest,
translate,
refine,
repair and insertion-admission lanes each threw the reason at overlap two
and wrote nothing for a later slice refused by its hook,
by a re-seated roster with no translator,
or by an outcome naming no prepared slice.
The panel stage,
the judge fidelity probe and the reference read run one lane,
so they cannot drop one.
A buy's failure was already written by the twin memo,
and the abort catches of `translate-slice-attempt.ts`,
`repair-slice-buy.ts` and `repair-refine-step.ts` throw the reason.
B405 named a cache write among the dropped kinds;
that holds for neither the consolidation nor the contest,
where the persist runs inside the buy and the twin memo writes its failure.

The fix:
`mapOverlapped` takes the caller's signal,
a logger and an item namer,
all required,
and once every lane has ended and the signal has aborted,
writes each failure it does not throw,
in position order,
through `exchangeFailureLogText`,
except the signal's reason and a failure marked written through `markFailureWritten`,
which the twin memo calls after its own line.
The thrown error is unchanged.
Twelve cases hold it:
four in `overlapped-map.unit.test.ts`,
one per lane driver,
and the two lookup cases of B409.
A new fixture,
`stop-landing-hook.test-fixture.ts`,
lets a case land the stop inside a job's hook.

Open to the owner's veto:
the required item namer;
the caller's logger rather than the root;
position order once every lane has ended;
the module `WeakSet` marker,
set by the twin memo and not by `repairOneCandidate`;
a never-aborting signal in the reference read when its caller gave none;
the refine case's `UnpreparedSliceError`;
and the line's wording.

Changed on 2026-10-11 (UTC) by B412 and B413:
`repairOneCandidate` now marks its failure too,
and the map writes its unthrown failures whether or not the caller has stopped.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B409: the work-title and cited-reference lookups wrote the caller's stop as a failed fetch

Red `7781cbaaa7`,
fixed in `63a4a35f0a`.

Found on 2026-10-11 (UTC) as the open item B406 recorded.
`workTitleLookupLines` (`work-title-lookup.ts`) and `citedReferenceBlock` (`cited-reference-lookup.ts`)
caught the caller's own reason in each member,
wrote it as a lookup that failed or a page that could not be fetched,
and answered it as data:
by the agent's probe on the build of `e241a7be6`,
"refused by Error" for the member ending in the reason,
and the call resolved.
No ruling or caller holds a data answer under a stop.

The fix:
each member rethrows a failure that is the signal's reason while the signal has aborted,
unwritten,
so the call refuses with that one reason whichever member ended first;
every other failure keeps its line and its data answer.
The two scan entries now read `shared-abort`,
resting on `aborted` and `reason`.

Open to the owner's veto:
preparation now fails with the reason at the lookups,
where it went on with unfetched lines.

Changed on 2026-10-11 (UTC) by B414:
the members no longer reject,
so the two scan entries read `catches-own` and rest on the outcome readers.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B410: the census of other fan-outs found none open, and four defects beside them

Run on 2026-10-11 (UTC) beside B408.
Every caller of `mapOverlapped`,
the twin memo
and both lookups were read for a member's own failure under a stop that reaches no line,
on top of B406's census;
none remains.

Open,
found by that reading:
the twin memo writes the caller's reason as an abandoned buy on the translate and repair paths,
since its catch holds no signal (`twin-memo.ts`,
read);
a repair seat's failure under a stop is written by both `repairOneCandidate` and the twin memo in the consolidation,
the agent's reading,
not run;
two failures without a stop still drop the higher one,
which the owner's rule on a stop does not cover;
and the cited-reference block writes its per-page lines in completion order,
which its case hides by sorting.

Closed on 2026-10-11 (UTC) by B411 to B414,
which ran each reading red on the build of `06ca2a85ca`.
All four held,
and the run of the second found the translate path doubled as well.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B411: the twin memo wrote the caller's stop as an abandoned buy

Red `142ed0e987`,
fixed in `6c39c45556`.

Found on 2026-10-11 (UTC) as the open item B410 recorded.
`reuseTwinOrBuy` (`twin-memo.ts`) caught a buy's failure with no signal in reach,
and wrote the caller's own reason as "the buy for a shared key was abandoned (refused by Error)",
on every translate and repair buy the stop ended,
since their abort catches throw the reason.
By the red case on the base build,
a buy and the twin waiting on it each wrote that line.

The fix:
`reuseTwinOrBuy` takes the caller's signal,
required,
and `acquireConsolidation` with it;
the four production callers pass the signal their buy honours
(`consolidate-driver.ts`,
`lane-contest-driver.ts`,
`translate-slice-settle.ts` and `repair-slice-settle.ts`),
and no `AbortSignal.any` sits on those paths.
A failure that is the stopped caller's reason withdraws the key and writes nothing,
and the waiting twin still asks for itself,
its own buy meeting the same stop,
which the case shows by two buys started.
Every other failure keeps its line and the provider's words.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B412: a repair re-ask's failure under a stop was written twice

Red `142ed0e987`,
fixed in `6c39c45556`.

Found on 2026-10-11 (UTC) as the open item B410 recorded,
and on the translate path as the inference B403 left open.
In the consolidation,
`repairOneCandidate` wrote a seat's own failure as the caller stopped,
and the twin memo wrote the same failure again as an abandoned buy;
on the translate path,
`attemptTranslateSlice` wrote it again as the slice's abandoned line,
the memo then seeing only the caller's reason.
Both fail red on the base build.

The fix:
`repairOneCandidate` marks the failure it writes through `markFailureWritten`;
the twin memo writes no line for a failure that `failureWritten` reports;
the translate attempt writes its abandoned line without the failure's words when the failure is marked;
`mapOverlapped` already skipped marked failures.
The seat's line was kept because it names the seat and the stage,
is the only writer for every seat after the first in input order
and for callers with no memo,
and is written first.
Dropping the memo's clause "so any twin waiting on it asks for itself" loses no fact of the event:
it holds for every withdrawn key and is the documented behaviour of `reuseTwinOrBuy`.
By the agent's control,
restoring the memo's line for a marked failure fails the composed case with a third line.

Open to the owner's veto:
the memo writes nothing for a marked failure,
where the alternative is a short line with no failure text saying only that the key was withdrawn;
the translate attempt's line for a marked failure is bare,
where the alternative keeps the words and accepts the double;
and the marker stays in `overlapped-map.ts`,
now imported by the twin memo,
`translate-repair.ts`,
`translate-slice-attempt.ts` and `corpus-run/run-seats-read.ts`,
where the alternative is a module of its own.

Reversed by this entry:
B408's choice that the twin memo sets the marker and `repairOneCandidate` does not.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B413: an overlapped job's own failure with no stop reached no line, and the seat reading's was written twice

Red `142ed0e987`,
fixed in `6c39c45556`.

Found on 2026-10-11 (UTC) as the open item B410 recorded.
`mapOverlapped` (`overlapped-map.ts`) wrote nothing while the caller had not stopped,
so of two jobs failing of their own the higher one's failure reached no line;
and `readDrynessPastShortBench` (`corpus-run/run-seats-read.ts`) wrote its writing-bench stop
and threw it unmarked from the per-slice hook,
so under a stop the map wrote it a second time.
Both fail red on the base build.

The fix:
the map writes each failure it does not throw,
in position order,
stopped or not,
skipping the thrown one,
a failure marked written
and the aborted signal's reason;
a line with no stop names the item whose failure is thrown;
the seat reading marks its refusal.
The check that the signal has aborted stays,
since an unstopped signal's reason is `undefined`,
which a job can throw;
by the agent's control,
dropping it fails the case of a failure that is no error at all.
The case that pinned the old behaviour,
"WRITES NOTHING WHILE THE CALLER HAS NOT STOPPED",
is rewritten into the case that writes each unthrown failure with no stop.
From reading the source,
not running each caller,
the census of the nine callers' jobs found the hook refusals marked,
the buy failures written by the twin memo,
and a few kinds nothing wrote that the map now writes
(a consolidation ledger gap,
a roster refusal,
an unprepared slice,
and the stage and cache failures of the refine phase);
the panel stage,
the judge fidelity probe and the reference read run one lane and cannot leave a failure unthrown.
Of the catch blocks that both write and throw,
the writers reaching a job's end were the twin memo,
the seat reading and `repairOneCandidate`,
all now marked;
the scan's reach is limited to writes within eight lines of the throw.

Open to the owner's veto:
the map writes without a stop,
where the alternative is writing only under a stop,
which drops the higher failure;
and the no-stop line's wording,
"and the failure thrown is X's,
which comes before it".

Open:
under a stop,
`repair-slice-buy.ts` and `repair-refine-step.ts` would write a marked seat-reading refusal from the hook
a second time in their "abandoned by the caller's abort" line;
reaching it needs the dryness read to resolve as the stop lands,
and no honest red case was built,
so nothing was changed there.
Closed on 2026-10-11 (UTC) by B415,
which built the red cases.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B415: a seat reading's refusal under a stop was written again by the repair buy and the refine step

Red `0f8847e91f`,
fixed in `5b0b8fef14`.

Found on 2026-10-11 (UTC) as the open item B413 recorded.
`buyRepairSlice` (`repair-slice-buy.ts`) and `refineSettledSlices` (`repair-refine-step.ts`) wrote their
"abandoned by the caller's abort" line with the failure's words,
even when the failure was the writing-bench refusal that `readDrynessPastShortBench` (`corpus-run/run-seats-read.ts`)
had written and marked,
so the refusal reached two lines when the stop landed as the checker stage or the recheck re-seated.
By the agent's run on the base build,
both modules,
handed the pass's own `reseatHookFor` over a client whose dryness read stops the caller
and then reports Bedrock alone wet,
wrote the refusal's clauses inside the abandoned line.
It can happen in production:
the budget read answers from its cached reading without consulting the signal,
and holds decay with the clock,
so the gate can see a hold the reading no longer sees.

The fix:
both catches write the abandoned line without the failure's words when `failureWritten` reports it,
as `translate-slice-attempt.ts` has since B412;
an unmarked failure keeps its words.
The three abort catches now agree on a marked failure.
`repair-benchmark.ts` writes the one other such line;
by reading its path no marked failure reaches it,
since its repair gets no hook and runs at overlap one.
The shared fixture `stop-landing-hook.test-fixture.ts` gained `stopLandingSeatReading`.

Open to the owner's veto:
the line for a marked failure is bare,
where the alternative keeps the words and accepts the double;
and the fixture sits in `stop-landing-hook.test-fixture.ts`,
where the alternative is a fixture of its own.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".
