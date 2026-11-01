# Audit ledger: the every-provider-dry refusal, what it stops and what it says

The router's refusal for a call made when every provider is out of budget:
whether it ends the run,
what the run ships when it does,
what its text states about NeuralWatt's day,
and the two rulings of 2026-10-22 that bound it.

The briefs of 2026-10-22 called the three defects of the XingZ6018 read D1 to D3,
and the review of their fixes called its findings S1 to S6 and N1 to N8.
Those labels are not ids of this ledger's D family,
and each entry names the label beside its number.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B818: the every-provider-dry refusal was lost as one seat's voice for 77 s, and four catches kept swallowing it where the pass guard was not

Status:
fixed on 2026-10-22 (UTC) in two rounds,
cache versions none.
The first round is the brief's D1:
the test commit `5971347842` (red),
the fix `bae1bb310e`,
and `67f5212ed4`,
which gave the queue case the one refusal body and the imported hang stop the source scans list.
The second round answers the review of the first (S2,
S3 and S6):
reds `b1f1196379` and `51e18a1831`,
fixes `4e5c7559ef`,
`803ad52aa8` and `3dcd82ed6d`,
with `068d523090`,
`555809589d`,
`9348f67ef3`,
`b423e15be8` and `462a2d8772`.
Found by the read of the XingZ6018 run (build `8864316eb2`) and confirmed by a second reader of its log.

What the run showed:
OpenRouter answered with HTTP 402 and was marked refused,
Hyper had been dry for a while,
and every call after that met the router's `EveryProviderDryError`.
Its TSDoc says it "ENDS THE RUN,
at the owner's instruction"
(`provider/src/budget-routing.ts`),
but `attemptStageCall` (`stage/src/stage-call.ts`) and `attemptDecisionCall`
rethrew an error only when the signal had aborted,
marked a seat `unreachable` only for `NoProviderForModelError`,
and logged `voice lost` for everything else.
992 lines of the log carry the refusal's text,
990 of them as `voice lost`,
and 903 of them stand before the stop.
The refine rounds logged 240 `refiner round: 0/3 heard` lines.
The run ended 77.655 s after OpenRouter was marked,
when the writing bench's floor failed in the middle of an entry,
with `status=ERROR`,
a STALLED line and no stop line naming a cause.
No chat stream started after either mark;
the meter reads continued.
Outside the provider package only tests named the class,
so no caller higher up caught it,
and the provider's own tests asserted only that the call throws.
Two owner rulings stood behind the claim:
"no matter what,
we must ship" (2026-09-27,
`design-commitments-shipping-and-the-archive.md`)
and that this refusal ends the run.

The first fix:
`attemptStageCall`,
`attemptDecisionCall` and `askOnce` rethrow it through `endsTheRun` (`stage/src/stage-run-end.ts`);
`entryErrorOutcome` files it as `INCOMPLETE` stopped work,
so no re-attempt is queued;
and `createPassDryStop` (`corpus-pass/src/corpus-run/pass-dry-stop.ts`) aborts the entries' base signal with the
refusal as its reason,
and the queue stops before the next entry with one stop line naming the cause,
the way the spend ceiling does.
The pages of every entry already settled stay shipped.
The bench-floor stop was not reused,
because it ends an entry from the inside as an error,
which the queue then re-attempts or stalls on.

The first fix was green in production and proved less than it said (M289).
`PassDryStop.guard` aborts the shared signal before the error reaches any catch,
and every layer that checks `signal.aborted` rethrows,
so the three new `endsTheRun` checks were the only protection for a caller without the guard,
and other swallowing paths had none.

Review S2:
four catches could swallow the refusal with the signal not aborted.
The picture readers' settle (`slice-reading/src/image-reading-pair.ts`) turned a rejected reader into
`reader-failed`,
so every picture of an entry read as "no reader available";
the grace window of a stage round waited on `Promise.allSettled(asks)`,
and a refusal arriving after quorum came out as `NOT ANSWERED` silence;
the overlapped map threw the failure at the earliest position,
so a non-dry failure on an earlier row hid a dry one on a later row;
and the re-ask elsewhere (`provider/src/nudged-reask.ts`) logged any failure as "no re-ask elsewhere" (B825).
The fix rethrows through `endsTheRun` in the first three
(`containFailedReaders` in `image-reading-contain.ts`,
the grace-window wait,
and a dry failure that wins over an earlier non-dry one in the map),
and keeps a typed refusal (`isDryRefusal`) in the round.
The reds were built without the guard,
and each asserts the error reaches the caller unchanged;
the grace-window case later waits a turn of the loop in place of a real timer,
as the real-clock scan lists.

Review S3:
the seat reader derives its benches from `client.providerDryness`,
and when every provider is dry it threw `WritingBenchUnreachableError` before any chat call.
No guarded call had met the router,
so the gate never tripped.
An entry that reaches its lanes with its preparation cached makes no chat call,
and so stopped as `resumable-failure` with `ERROR`:
no stop line,
and every pending entry the queue visited was charged an attempt,
which `orderPendingEntries` reads as a flaky entry and ranks last within its size band.
The fix:
`writtenRefusal` (`run/src/corpus-run/run-seats-refusal.ts`) raises the every-provider-dry refusal,
stating the reading and the holds,
when every provider reads dry,
and keeps the bench error for a bench below its floor with a provider still wet;
`tallyCaughtEntry` calls `endRun` when `endsTheRun` holds;
and the gate's `trip` takes a refusal that reached an entry without passing a guarded call.
The red of `51e18a1831` failed with `dryStop.trip is not a function`,
`dryStop.takeStopNote is not a function`,
`endRun is not a function`
and a settle case that told the run nothing.

Review S6:
the trip was silent,
and the one stop line came from `stopBeforeNext`,
which the queue asks only before an attempt.
A dry entry that was the last one pending ended the loop with no stop line at all,
so the cause stood only in the `error=` text of one TALLY line.
When a layer filed the failure as something other than `stopped`,
the REATTEMPT line printed first and the stop line right after it.
The fix:
the gate logs one line when it trips,
with the cause,
and hands out its stop notes once (`takeStopNotes`),
the queue announces the stop when it ends too (`corpus-pass-queue.ts`),
and `9348f67ef3` pins that the attempt queue queues no second attempt for an entry the run stopped during.
The red of `51e18a1831` failed on the log line (`expected +0 to equal 1`),
on the last-entry case and on the REATTEMPT case.
No document used the name `takeStopNote`;
the first spelling lived only in that red commit.

What is left:
the entry in flight ships nothing (B821),
the refusal is still raised per model (B822),
and the decision seat's branch is reached only by a test (B824).

Cache versions:
none moves.
The change is in run control and decides nothing a record may hold;
`9d6095ae7e` states this for the cache versions that stay.

Recurrence:
`mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"A refusal that ends the run".

## B819: the every-provider-dry refusal named 00:00 UTC for NeuralWatt's return, then opened with a sentence that contradicted its own clause

Status:
fixed on 2026-10-22 (UTC),
cache versions none.
The brief's D2:
red `0883519545`,
fix `38db033837`,
scans `bae059bdfe`.
The review's S4 and S5:
red `a4552e1722`,
fix `14b952c1d2`.
Found by the read of the XingZ6018 run.

What the run showed:
the refusal said "NeuralWatt's free energy returns at 00:00 UTC"
(`provider/src/budget-routing.ts`),
on every line that carried it.
The account's day starts at 19:15:08 UTC:
the log decided it at the start of the run from the quota's `subscription.current_period_start`,
and `neuralwattDayStart=2026-10-21T19:15:08.000Z` stands on all 63 `METERS` lines.
The owner ruled on 2026-10-20 that the day is "not 00:00 UTC,
which the first version of this package assumed"
(`provider/src/neuralwatt-day.ts`),
and `neuralwatt-energy.ts` records that the vendor's documentation says 00:00 UTC while the account does not.
The refusal's constructor took no day anchor,
so it repeated the documentation and could not state the account's time.
The comment in `provider-budget-refusal.ts` repeated it,
and `provider-router.unit.test.ts` pinned the string.

The fix:
the refusal takes what the run knows of NeuralWatt's day through the budgets (`neuralwattReturn`,
read without deciding) and states its time of day in UTC;
a run with no NeuralWatt client says NeuralWatt is off,
and one that never learned a day start says so.
The client builds its anchor keeper and hands it to the day session,
and the model hold refuses itself (`refuseWhileHeld`),
which keeps `provider-router.ts` and `neuralwatt-day-session.ts` under the line cap.
The scans follow the helper that spells the time of day (`neuralWattTimeOfDayUtc`):
the number-read record and the message inventory.

Review S4:
the message still opened "NeuralWatt has no free daily energy left" for every day state.
For a run with NeuralWatt off it then read "has no free daily energy" and "is off" together,
and for a day start never learned the keeper refuses NeuralWatt until it learns one,
so NeuralWatt may well have energy and the opening was false.
`openingClauseOf` now says nothing for off,
"refused until its day start is learned" for unanchored,
and the original sentence only for a decided day.
The red failed in two cases on `not.toContain('has no free daily energy')`.

Review S5:
the refusal after a hold wait (`provider/src/budget-hold-wait.ts`) reused the day captured before the wait.
The second budget read calls the NeuralWatt meter,
which is where the keeper learns a day start,
so the capture could be wrong by construction.
The refusal now reads the day again after the wait.
The red failed on `expected ... to include 'at 19:15:08 UTC'`.

A second claim of the same sentence stands unchanged:
the refusal says Bedrock "has no credit left short of the owner's card"
where the log of the run says Bedrock was off because its token had expired (HTTP 401).
The reader of the log noted it,
and no commit of this round touches that sentence.

Cache versions:
none moves.
The text is an error message,
stored in no cache record.

Recurrence:
`mistake-prevention-messages-labels-and-counts.md`,
"A message built from a state read once".

## B821: the entry in flight when the run stops ships nothing, and "a run always ships" was read to cover it

Status:
ruled on 2026-10-22 (UTC) by the lead,
open to the owner's veto,
and in force as built by `bae1bb310e`.
Recorded in `design-commitments-rulings-of-2026-10-22.md`.
No code changes.
Found by note N5 of the review of the XingZ6018 fixes,
and carried to a decision by the fix brief of that day.

The question:
the pass comments (`corpus-run/pass-entry.ts`) say "A run always ships",
and the owner said "no matter what,
we must ship".
A dry stop abandons the entry in flight:
no artifact is written,
the pages of earlier entries stay,
and the entry is left for the next launch,
with the stop note saying "left resumable".
Whether the rule covers the partial entry
(settled slices with their wording,
the archive's text for the rest)
was not decided.

The ruling:
the entry in flight stays `INCOMPLETE` and stopped,
resumable,
keeping its slice cache.
The pages of the entries settled earlier ship,
each written when its entry settled.
The run ships what it settled.

Why not the archive path of 2026-09-27:
that path (a slice with no valid wording keeps the archive's and the page ships) ends with the entry settled.
A settled entry discards its slice cache (`retireSettledEntryCache`),
and nothing revisits it.
An entry that a dry router interrupted has slices no repair has reached,
not slices with no valid wording,
so filing it there would freeze them unrepaired,
a worse page than resuming the entry on the next launch.

What any other design is missing,
from the hand-back of the worktree `tr-w291` (outside the repository):
a degraded finish across the stages that makes no provider call,
so the remaining stages can end an entry when nothing can be bought,
and an artifact status for a partial entry,
since a settled artifact has none for work settled in part.

Cache versions:
none.

Recurrence:
`mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"A refusal that ends the run":
a rule that says a run ships names what it covers when work is in flight.

## B822: the router raised the every-provider-dry refusal per model, so one model's dry providers ended the whole run

Status:
ruled on 2026-10-22 (UTC) by the lead,
open to the owner's veto,
and queued as D4,
implemented apart from this record in the worktree `tr-w293`.
At `495341919a` the code still raises the refusal per model.
Recorded in `design-commitments-rulings-of-2026-10-22.md`.
Found by note N5 of the review and by reading the owner's ruling against the router.

The mismatch:
the owner's ruling that the refusal ends the run rests on one reason,
"this one leaves nothing,
so no seat's voice can be heard" (`stage/src/stage-run-end.ts`).
The router raises it per model.
In `provider/src/budget-routing.ts` the branch that builds `meters read ... wet but not serving this model`
throws it when every provider that serves the model asked is dry,
even while other providers are wet,
and `dryForTheModelAsked` counts an always-wet Xiaomi meter as dry for a model Xiaomi does not serve.
So a call for `xiaomi/mimo-v2.6-flash` with Xiaomi and OpenRouter dry,
or for `glm-5.3` with Hyper dry,
ends the pass while NeuralWatt and DeepSeek can still serve every other seat.

The ruling:
only dryness of every provider the run holds ends the run.
A model whose own providers are all dry while others are wet is a lost seat:
the seat is dark and the bench reseats it as it reseats an unserved seat.
A call that finds every provider dry still ends the run.

Closes an open question:
the note of the first review fix asked whether a dry OpenRouter at the decisions route alone,
with other providers wet,
is "every provider dry".
Under this ruling it is not,
and `provider-router-decide.ts` already treats it so (B824).

What the ruling changes outside the code:
`doc/decision/translation-repair-direct-providers.md` states the per-model ending as designed
("a fleet that is dry except for Xiaomi still ends the run for every other model").
It describes the code as shipped,
so it stays as written until the change lands,
and the change updates it.

Cache versions:
none expected;
the change is in routing and run control.

Recurrence:
`mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"A refusal that ends the run".

## B823: the seat reader's strict every-provider-dry case depends on a roster edge a roster change can flip

Status:
open,
found while fixing B818 (S3),
and queued in the brief of D4 as the step that pins the roster in that case.
Not a fault of the shipped code.

The seat reader's strict every-provider-dry case relies on a model-relative edge:
a Xiaomi-only roster reads dry when only Xiaomi is out.
A change to the roster's providers can therefore flip the case from dry to wet
without any change to the code under test.

The fix owed:
the case pins the roster it reads,
so the reading it asserts does not depend on the shipped one.

Cache versions:
none.

Recurrence:
`mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"A refusal that ends the run":
a case that asserts a dryness reading pins the roster the reading is relative to.
