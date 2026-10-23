import type { Logger, } from '@monochromatic-dev/module-logger/ts';
import { once, } from 'node:events';
import { settleWithin, } from '@monochromatic-dev/module-async-time/ts';
import type { ChatMessage, } from '@monochromatic-dev/module-llm-type/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  EveryProviderDryError,
  JsonSchemaResponseFormat,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';
import type { StageDecision, } from './stage-decision-call.ts';
import {
  attemptStageCall,
  isRequestFault,
  type StageVoice,
} from './stage-call.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  describeAbandon,
  exchangeFailureLogText,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { monotonicMs, } from '@monochromatic-dev/translation-repair-markup/ts';
import { roundLine, } from './stage-round-line.ts';
import { isDryRefusal, } from './stage-run-end.ts';
import { StragglerCutError, } from './straggler-cut.ts';

//region Stage round
// ONE fan-out round, and the rule that a stage never finishes later than its
// slowest voice by more than a bounded grace.
//
// A round used to await every call together, so a model that hung until its
// deadline delayed every stage that seated it by the whole of that deadline,
// 360 seconds on the corpus configuration, however early quorum arrived. The
// user's standing rule of 2026-08-15 is that the failure of any one model for
// the day must not delay the pipeline, and waiting six minutes per gather for a
// voice that never comes is exactly that delay.
//
// Cutting AT quorum was rejected for a measured reason rather than a cautious
// one: quorum on a roster of three is two, so cutting there would discard the
// third voice on nearly every gather, healthy or not, and shrink every ensemble
// to its minimum permanently. The grace window separates the two cases, because
// a working third model answers within seconds of the second and a hung one
// never answers at all.

/**
 Time a voice still in flight is given once quorum stands.

 SIXTY SECONDS ORIGINALLY, a user figure of 2026-08-15 chosen between this and
 cutting at quorum outright, and recorded then as not derived from the latency
 distribution and due a revisit against one. That revisit happened on
 2026-08-17 and the window moved; see
 `doc/decision/translation-repair-straggler-grace.md`.

 THE WINDOW WAS BELOW THE TWO SLOWEST MODELS' ORDINARY RANGE, which is why it
 cut them and nothing else. Whole-call latency over 602 bench exchanges:
 `hf:openai/gpt-oss-120b` p50 4.4 s, `hf:moonshotai/Kimi-K3` p50 9.5 s,
 `hf:zai-org/GLM-5.2` p50 24.0 s with p95 74.0 s and max 85.5 s, and
 `hf:zai-org/GLM-4.7-Flash` p50 30.5 s with p95 72.9 s and max 88.6 s. Sixty
 seconds sits between the GLM medians and their 95th percentiles, so it cut
 working voices by construction.

 IT HAS NEVER CAUGHT WHAT IT EXISTS TO CATCH. Across those 602 exchanges the
 only non-ok outcomes were 8 straggler cuts, all on the two GLM models, and no
 timeout of any other kind. Not one hung call was recorded, so every voice the
 window has taken was a slow-but-working one.

 THREE MINUTES FROM 2026-08-17, rather than the observed maximum, deliberately.
 A maximum over a few hundred samples is not a bound, which `STREAM_IDLE_MS`
 in `stream-idle-guard.ts` records this codebase learning the hard way, so
 that figure sat above the 88.6 s maximum by more than a factor of two.

 TWO MINUTES SINCE 2026-09-03, THE OWNER'S DECISION ON A MEASURED PAIR. The
 four-entry pass of 2026-09-02 had been launched at the original 60 s by
 dial, and two keyword233 passes on the all-OpenRouter bench, identical but
 for the dial, put 60 s against 120 s: cut streams 14 against 7, voices never
 heard 19 against 8, tally 1,247 against 1,321 seconds (inside the day's
 957 to 1,321 s run-to-run band), 0.76 against 0.45 USD. Every cut at 60 s
 was a reasoning stream still working. Offered 60, 120, 180 and withholding
 the slow seats, the owner chose 120; record in
 `doc/decision/translation-repair-straggler-grace.md`. It remains well under
 `RUN_PER_CALL_TIMEOUT_MS` of 360_000, so the window still cuts a genuinely
 hung voice long before its own deadline would, which is the whole purpose
 the user's rule of 2026-08-15 gave it.

 THE EDITOR CALIBRATION RUNS UNDER 300000 MS INSTEAD
 (`CALIBRATION_STRAGGLER_GRACE_MS` in `grace-override.ts`, the round fallback
 `editor-calibrate.ts` reads both windows with, the owner's decision of
 2026-08-26 on arm D), because under four slices in flight the longer window
 costs nothing measurable and cuts the fewest voices. Writer rounds wait `WRITER_GRACE_MS`
 (`writer-grace-override.ts`, the owner's decision of 2026-09-06) or this
 value when it is the longer one, unless `TRANSLATION_REPAIR_WRITER_GRACE_MS`
 says otherwise.
 */
export const STRAGGLER_GRACE_MS = 120_000;

/**
 One model's answer, or its silence, from one round.

 @example
 ```ts
 const outcome: RoundOutcome<Wire> = { modelId, voice: { heard: false, answered: false, }, };
 ```
 */
export type RoundOutcome<ValueT,> = {
  /**
   Model that was asked.
   */
  readonly modelId: RosterModelId;

  /**
   What came back.
   */
  readonly voice: StageVoice<ValueT>;
};

/**
 Waits for the next of the pending asks to settle, reporting an ask's
 rejection as the caller's own abort.

 AN ASK REJECTS ONLY ONCE THE CALLER'S SIGNAL HAS ABORTED, or the route refused
 its request as malformed: `askOnce` turns every other failure into a lost
 voice, bar the router's every-provider-dry refusal, which it rethrows as it
 is. What it rejects with is whatever ITS call caught, which is the caller's
 forwarded reason unless the call's own
 deadline or transport failure landed as the caller aborted. So two asks
 could reject with different values, and the race handed the round whichever
 ended first. The asks share no rejection value; they share the caller's
 signal, so this throws that signal's reason in place of whatever the first
 rejecting ask ended in, and the round's failure is the same whichever ask
 ended first.

 THE FAILURE IT REPLACES IS NOT ITS TO LOG. Each ask writes its own failure
 as it ends (`askOnce` in `runGatherRound`), so a failure is on a line
 whichever ask ended first, the asks that end after this throws included,
 and a provider's refusal keeps its words there.

 @param pending - promises of the asks still to settle, each settling to its ask's position

 @param signal - the caller's abort, whose reason replaces an ask's rejection
 so the round's failure does not depend on which ask ended first

 @returns Position of the ask that settled first

 @throws The caller's abort reason, once an ask rejected after the caller aborted

 @throws Whatever the ask rejected with, unchanged, where the caller had not
 aborted, which only a failure inside `askOnce`'s own catch produces, such as
 a logger that cannot write, or a request the route refused as malformed

 @example
 ```ts
 const settled = await nextSettled({ pending: pending.values(), signal, },);
 ```
 */
async function nextSettled(
  {
    pending,
    signal,
  }: {
    readonly pending: Iterable<Promise<number>>;
    readonly signal: AbortSignal;
  },
): Promise<number> {
  try {
    return await Promise.race(pending,);
  }
  catch (error) {
    signal.throwIfAborted();
    throw error;
  }
}

/**
 What the next router refusal settles to in the race `awaitHeard` runs, in
 place of an ask's position: no position is negative, so it names no ask.
 */
const REFUSAL_NOTED = -1;

/**
 One stretch of a round between router refusals: aborting `controller` marks
 the next refusal and settles `noted`.

 @example
 ```ts
 const tick: RefusalTick = freshRefusalTick();
 ```
 */
type RefusalTick = {
  /**
   Aborted at the refusal that ends this stretch.
   */
  readonly controller: AbortController;

  /**
   Settles to {@link REFUSAL_NOTED} once `controller` aborts.
   */
  readonly noted: Promise<number>;
};

/**
 Router refusals a round has seen so far, and the stretch the next one ends.

 @example
 ```ts
 const refusals: RefusalTally = { count: 0, tick: freshRefusalTick(), };
 ```
 */
type RefusalTally = {
  /**
   Seats the router refused this round, reserve seats included.
   */
  count: number;

  /**
   Stretch the next refusal ends.
   */
  tick: RefusalTick;
};

/**
 Opens a stretch the next router refusal ends.

 ONE PROMISE PER STRETCH, not one per pass of `awaitHeard`'s loop: every
 pass races the same `noted`, so the signal never gathers a listener per pass.

 @returns Stretch whose `noted` settles once its controller aborts

 @example
 ```ts
 refusals.tick = freshRefusalTick();
 ```
 */
function freshRefusalTick(): RefusalTick {
  /**
   Aborted at the refusal that ends this stretch.
   */
  const controller = new AbortController();
  return {
    controller,
    noted: (async function untilRefusal(): Promise<number> {
      await once(
        controller.signal,
        'abort',
      );
      return REFUSAL_NOTED;
    })(),
  };
}

/**
 Waits until enough voices are heard, or until every ask has settled.

 Counts HEARD voices rather than settled calls, because a call that came back
 unusable moves the round no closer to a quorum and waiting on it is the same
 waiting this module exists to bound.

 THE NEED IS READ AGAIN AT EVERY SETTLED ASK AND EVERY ROUTER REFUSAL, since
 the owner's answer of 2026-10-22 ("Let the grace open"): a refusal shrinks
 the bench the quorum is sized on, and a round sized before it would wait
 on every live seat where the bench, read again, closes one voice short of
 them. A refused seat that hands its place to a reserve seat settles no ask,
 so the refusal wakes the wait itself.

 @param asks - in-flight asks, one per model

 @param neededNow - voices this round must hear before the grace starts, on
 the refusals seen so far

 @param refusals - router refusals this round has seen

 @param signal - the caller's abort, whose reason is what the round rejects
 with when an ask rejects, whichever ask ended first

 @returns Voices the round needed when the wait ended

 @throws The caller's abort reason, once an ask rejected after the caller aborted

 @throws Whatever an ask rejected with, unchanged, where the caller had not
 aborted, which only a failure inside `askOnce`'s own catch produces, such as
 a logger that cannot write

 @example
 ```ts
 const needed = await awaitHeard({ asks, neededNow, refusals, signal, },);
 ```
 */
async function awaitHeard<ValueT,>(
  {
    asks,
    neededNow,
    refusals,
    signal,
  }: {
    readonly asks: readonly Promise<RoundOutcome<ValueT>>[];
    readonly neededNow: () => number;
    readonly refusals: Readonly<RefusalTally>;
    readonly signal: AbortSignal;
  },
): Promise<number> {
  /**
   Asks still to settle, keyed by position so the winner can be removed
   without identity comparisons on promises.
   */
  const pending = new Map(
    asks.map(function toRace(
      ask,
      index,
    ): readonly [
      number,
      Promise<number>,
    ] {
      return [
        index,
        (async function settleAt(): Promise<number> {
          await ask;
          return index;
        })(),
      ];
    },),
  );

  /**
   Voices heard so far this round.
   */
  const counters = { heard: 0, };
  while ((counters.heard < neededNow()) && (pending.size > 0)) {
    /* oxlint-disable no-await-in-loop -- the loop IS the wait: each pass consumes exactly one settled ask or one refusal and re-races the rest */
    /**
     Position of the ask that settled first among those still pending, or
     {@link REFUSAL_NOTED} where a router refusal came first.
     */
    const settled = await nextSettled({
      pending: [
        ...pending.values(),
        refusals.tick
          .noted,
      ],
      signal,
    },);
    /* oxlint-enable no-await-in-loop */
    if (settled === REFUSAL_NOTED)
      continue;
    pending.delete(settled,);

    /* oxlint-disable no-await-in-loop -- reading an already-settled promise, which suspends for one microtask rather than for a call */
    /**
     That ask's outcome, already settled and therefore free to await.
     */
    const outcome = await asks[settled];
    /* oxlint-enable no-await-in-loop */
    if (outcome?.voice
      .heard
      === true)
      counters.heard += 1;
  }
  return neededNow();
}

/**
 Whether the router refused a seat for want of a wet provider, the one loss
 that frees the seat's place in its round.

 @param outcome - one seat's outcome

 @returns True for a refusal, false for a voice or any other loss

 @example
 ```ts
 refusedByRouter({ outcome, },);
 ```
 */
function refusedByRouter<ValueT,>(
  { outcome, }: { readonly outcome: RoundOutcome<ValueT>; },
): boolean {
  /**
   What came back for the seat.
   */
  const { voice, } = outcome;
  return (!voice.heard) && voice.unreachable;
}

/**
 Runs one fan-out round and abandons whatever is still in flight once quorum
 has stood for the window it is handed.

 @param client - injected model client

 @param modelIds - models this round asks

 @param messages - prompt shared by every voice

 @param signal - caller abort, which always wins over the straggler cut

 @param exchangeTimeoutMs - deadline per exchange

 @param responseFormat - structured-output constraint

 @param validate - client-side schema guard

 @param stage - stage label for logging

 @param l - logger of the calling stage

 @param heardNeeded - voices still needed for quorum, which starts the grace

 @param heardNeededAfterRefusals - voices still needed once the router has
 refused the given number of this round's seats, reserve seats included;
 the round waits for the lesser of this and `heardNeeded`, read again at
 every refusal, so a bench the refusals leave short opens its grace on the
 quorum it now has (owner, 2026-10-22, "Let the grace open"). Absent, the
 round waits for `heardNeeded` alone.

 @param reserve - pending seats past this round's window, in the order the
 gather would ask them; a seat the router refuses for want of a wet provider
 hands its place to the next of these at once, so the refusal costs the
 round no voice. Empty where the round asks the whole bench.

 @param graceMs - window granted after quorum before stragglers are abandoned:
 the launch's round or writer window from the `StageSettings` its entry file
 read (`stage-settings.ts`), {@link STRAGGLER_GRACE_MS} unless a dial moved
 it. REQUIRED, where it once defaulted to a read of
 `TRANSLATION_REPAIR_STRAGGLER_GRACE_MS` at every round

 @returns One outcome per model asked: the window's in roster order, then
 each reserve seat taken, in the order the round took it

 @example
 ```ts
 const outcomes = await runGatherRound({ ..., heardNeeded: 2, },);
 ```
 */
export async function runGatherRound<ValueT,>(
  {
    client,
    modelIds,
    messages,
    signal,
    exchangeTimeoutMs,
    maxAnswerChars,
    responseFormat,
    validate,
    stage,
    l,
    heardNeeded,
    heardNeededAfterRefusals,
    reserve = [],
    graceMs,
    decision,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly messages: readonly ChatMessage[];
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly maxAnswerChars?: number;
    readonly responseFormat: JsonSchemaResponseFormat;
    readonly validate: (value: unknown,) => value is ValueT;
    readonly stage: string;
    readonly l: Logger;
    readonly heardNeeded: number;
    readonly heardNeededAfterRefusals?: (refused: number,) => number;
    readonly reserve?: readonly RosterModelId[];
    readonly graceMs: number;
    readonly decision?: StageDecision;
  }>,
): Promise<readonly RoundOutcome<ValueT>[]> {
  /**
   Cut for this round's stragglers, owned here so it can never outlive the
   round or reach the caller's own abort.
   */
  const abandon = new AbortController();

  /**
   Signal every call in this round honors: the caller's steering, or this
   round's cut, whichever comes first.
   */
  const roundSignal = AbortSignal.any([
    signal,
    abandon.signal,
  ],);

  /**
   Outcomes as they arrive, keyed by roster position.

   Written from inside each ask so the round can be assembled from whatever
   HAS answered rather than by awaiting calls it has already abandoned. That
   matters more than it looks: awaiting an abandoned call would make this
   round's completion depend on the client honouring an abort, and a client
   that ignored one would hang the stage forever rather than lose a voice.
   */
  const arrived = new Map<number, RoundOutcome<ValueT>>();

  /**
   When this round dispatched, so its own line can say how long it took.

   THE RUN COULD NOT SAY WHERE ITS HOURS WENT. It was found that the only
   per-call line fires at completion, so a log carries no round boundary at
   all: nothing separates the time a round spent gathering from the time it
   spent waiting on a straggler after quorum already stood. That second
   number is exactly what
   `doc/audit/every-volume-guard-is-blind-to-one-model.md` could bound only
   from above, at the grace window times the number of cut events, and said
   confirming it "needs the dispatch timestamps the run does not currently
   record". This is that timestamp, read on `monotonicMs` like the round's
   other two marks, so setting the system clock mid-round cannot move the
   durations between them (ledger B78).
   */
  const startedAt = monotonicMs();

  /**
   Reserve seats this round took in place of refused ones, in the order it
   took them; their outcomes sit after the window's.
   */
  const taken: RosterModelId[] = [];

  /**
   Requests the route refused as malformed that the asks met, in the order
   they ended, which the round throws once its asks have settled.
   */
  const requestFaults: unknown[] = [];

  /**
   Refusals that end the run, as the asks that met them raised them, kept so the grace window's wait, which swallows a
   rejected ask, can raise the first.
   */
  const dryRefusals: EveryProviderDryError[] = [];

  /**
   Router refusals this round has seen, which the wait for quorum reads.
   */
  const refusals: RefusalTally = {
    count: 0,
    tick: freshRefusalTick(),
  };

  /**
   Counts a seat the router refused and wakes the wait for quorum, which
   reads the need again; any other outcome leaves both alone.

   @param outcome - outcome of the seat a slot just asked
   */
  function noteRefusal({ outcome, }: { readonly outcome: RoundOutcome<ValueT>; },): void {
    if (!refusedByRouter({ outcome, },))
      return;
    refusals.count += 1;
    /**
     Stretch this refusal ends.
     */
    const ended = refusals.tick;
    refusals.tick = freshRefusalTick();
    ended.controller
      .abort();
  }

  /**
   Voices this round must hear before the grace opens, on the refusals seen
   so far: never more than `heardNeeded`.

   @returns Heard voices the wait for quorum stops at
   */
  function neededNow(): number {
    return (heardNeededAfterRefusals === undefined)
      ? heardNeeded
      : Math.min(
        heardNeeded,
        heardNeededAfterRefusals(refusals.count,),
      );
  }

  /**
   Asks one seat and records its outcome at its position.

   @param modelId - seat asked

   @param position - window index, or the window's length plus the order
   the reserve seat was taken in

   @returns Seat's answer, its recorded silence, or its refusal

   @throws Whatever the seat's call raised, once the caller aborted, having
   written it on the seat's line unless it is the caller's reason itself

   @throws The request fault the seat's call raised, where the route refused
   the request as malformed (`isRequestFault`), having cut the round's other
   seats with it

   @throws `EveryProviderDryError`, unchanged, when the router found no
   provider with budget left, which ends the round and the run instead of
   costing one seat its voice
   */
  async function askOnce(
    {
      modelId,
      position,
    }: {
      readonly modelId: RosterModelId;
      readonly position: number;
    },
  ): Promise<RoundOutcome<ValueT>> {
    try {
      /**
       This model's answer, or its recorded silence.
       */
      const outcome: RoundOutcome<ValueT> = {
        modelId,
        voice: await attemptStageCall({
          client,
          modelId,
          messages,
          signal: roundSignal,
          exchangeTimeoutMs,
          ...((maxAnswerChars === undefined) ? {} : { maxAnswerChars, }),
          responseFormat,
          validate,
          stage,
          l,
          // Conditional spread keeps the question absent instead of undefined.
          ...((decision === undefined) ? {} : { decision, }),
        },),
      };
      arrived.set(
        position,
        outcome,
      );
      return outcome;
    }
    catch (error) {
      // The CALLER's abort still propagates, so user steering stops a fan-out
      // exactly as it did. Only this round's own cut is turned into a lost
      // voice, because that is what it is: a model that had its chance.
      //
      // A FAILURE OF THE CALL'S OWN IS WRITTEN HERE, as the seat ends, since
      // the round fails with the caller's reason and no catch past it sees
      // this one: a deadline or a provider's refusal that landed as the stop
      // arrived, before quorum or in the grace window, whichever seat ended
      // first. Rendered as the seat's lost voice is (`attemptStageCall`), so
      // a refusal keeps its status and the provider's words, the key masked
      // where the transport built it.
      // EVERY PROVIDER DRY ENDS THE RUN, so the round fails with it rather than turning it into an abandoned
      // straggler: no other seat can be served either.
      if (isDryRefusal(error,)) {
        dryRefusals.push(error,);
        throw error;
      }
      if (signal.aborted) {
        if (error !== signal.reason) {
          l.warn(
            `${stage} ${modelId}: ${exchangeFailureLogText({ error, },)}, as the caller stopped the round`,
          );
        }
        throw error;
      }

      // A REQUEST THE ROUTE REFUSED AS MALFORMED ENDS THE ROUND, before
      // quorum and after it: it is no straggler the window cut, and the
      // quiet settling of an ask in the grace window would otherwise take it
      // for one. The other seats are cut with the fault as the reason, which
      // the seat tally reads as no seat's loss, and it is kept to be thrown
      // once the asks have settled.
      if (isRequestFault({ error, },)) {
        requestFaults.push(error,);
        abandon.abort(error,);
        throw error;
      }
      l.warn(
        `${stage} ${modelId}: abandoned ${
          String(graceMs,)
        }ms after quorum (${describeAbandon({ error, },)}), voice lost`,
      );

      /**
       Silence recorded for a model this round stopped waiting on.
       */
      const abandoned: RoundOutcome<ValueT> = {
        modelId,
        // NOT ANSWERED: this is the straggler window closing on a model that
        // was still working, which is the one loss a fresh call cannot shorten.
        voice: {
          heard: false,
          answered: false,
          unreachable: false,
        },
      };
      arrived.set(
        position,
        abandoned,
      );
      return abandoned;
    }
  }

  /**
   Every window slot, in flight together. A slot asks its window seat and,
   while the router refuses the seat it asked for want of a wet provider,
   the next reserve seat in its place.

   THE REFUSAL COMES BACK IN THE SAME MILLISECOND, so without this a window
   holding a seat the reading seated but no provider serves asked one voice
   fewer than it meant to: the spare was spent on nothing, and the round
   waited for its slowest remaining voice or fell through to a retry round.
   TianqiChen66620 (2026-09-27) closed 269 of 421 rounds a seat short with
   no grace, 2,852 s of the 4,575 s its rounds took. The refused seat stays
   on the bench, as the owner's rule of 2026-09-09 has it
   (`doc/decision/translation-repair-short-bench-share.md`); only its place
   in this round passes on. Since 2026-10-22 each refusal also wakes the
   wait, which sizes the need again (`heardNeededAfterRefusals`), so a bench
   a refusal leaves with exactly its quorum closes one voice below it.
   */
  const asks = modelIds.map(async function askSlot(
    modelId,
    index,
  ): Promise<RoundOutcome<ValueT>> {
    /**
     Outcome of the seat this slot asked most recently.
     */
    let outcome = await askOnce({
      modelId,
      position: index,
    },);
    noteRefusal({ outcome, },);
    /**
     Reserve seat next in line, if any is left.
     */
    let next = reserve[taken.length];
    while (refusedByRouter({ outcome, },)
      && (next !== undefined)
      && (!roundSignal.aborted)) {
      taken.push(next,);
      /* oxlint-disable-next-line no-await-in-loop -- each reserve seat is asked only after the seat before it in this slot was refused */
      outcome = await askOnce({
        modelId: next,
        position: (modelIds.length + taken.length) - 1,
      },);
      noteRefusal({ outcome, },);
      next = reserve[taken.length];
    }
    return outcome;
  },);

  /**
   Voices the round needed when the wait for quorum ended, after every
   refusal it saw.
   */
  const neededAtQuorum = await awaitHeard({
    asks,
    neededNow,
    refusals,
    signal,
  },);

  /**
   When quorum stood, which is the instant the grace window opens.

   SPLITTING THE ROUND HERE IS THE POINT. Time before this is the round doing
   its work; time after it is the round waiting on voices it may never hear.
   Only the second half is straggler cost, and a single round duration cannot
   tell them apart.
   */
  const quorumAt = monotonicMs();

  // Whichever comes first: everyone answers, or the grace expires. A round
  // that never reaches quorum has already waited for every one of `asks`, since
  // `awaitHeard` also stops when nothing is pending, so this resolves at once
  // rather than adding a window to a round that had nothing left to wait for.
  //
  // `allSettled` rather than `all`: the caller's own abort rejects every ask,
  // and a rejected `all` inside a race nobody reads becomes an unhandled
  // rejection on a teardown path.
  //
  // `settleWithin` RATHER THAN `Promise.race([..., wait(graceMs,),],)`, which
  // is what this was. `Promise.race` does nothing to the loser, and `wait` is a
  // bare `setTimeout` with no handle, so every round where the roster answered
  // in time left a live timer holding the event loop for the rest of the grace
  // window. The work was unaffected and the process simply would not exit:
  // measured at three minutes past a completed 40-subject run, long enough to
  // read as a hang and to stall anything chained behind it.
  await settleWithin({
    promise: Promise.allSettled(asks,),
    ms: graceMs,
  },);
  // A DRY REFUSAL THAT LANDED IN THE WINDOW ENDS THE RUN TOO. `allSettled` swallows the ask that raised it, and the
  // position would come out of the assembly as a silent seat in an ordinary thin roster, so the refusal `askOnce`
  // recorded as it ended is raised here. Before quorum it already surfaced through `awaitHeard`.
  /**
   The first such refusal, absent when none landed.
   */
  const [dryRefusal,] = dryRefusals;
  if (dryRefusal !== undefined)
    throw dryRefusal;
  // ABORTED WITH A REASON THE SEAT TALLY READS: a call this cuts is the straggler window closing, which counts against
  // its seat when the call had delivered nothing (`seat-tally.ts`), where a caller leaving never does.
  abandon.abort(new StragglerCutError(),);

  // A seat that met a refused request in the grace window settled quietly in
  // the wait for the window, so the fault is thrown here and not left as a thin
  // roster.
  if (requestFaults.length > 0)
    throw requestFaults[0];

  // The caller's abort has to leave this round as a FAILURE, not as a thin
  // roster. `allSettled` swallows every ask the abort tore down, so a
  // stop that arrives after quorum would otherwise return the voices that beat
  // it and read exactly like an ordinary degraded round: the stage would decide
  // on that, the slice would settle, and the driver would cache a decision the
  // run was told to stop making. Before quorum an ask's rejection already
  // surfaces through `awaitHeard` as this same reason; this covers the window
  // after it. Either way a seat's own failure is already on its line, written
  // by `askOnce` as the seat ended.
  signal.throwIfAborted();

  /**
   Every model's outcome, the window's in roster order and then each reserve
   seat taken, with a recorded silence at any position this round never
   filled.
   */
  const outcomes = [
    ...modelIds,
    ...taken,
  ].map(function toOutcome(
    modelId,
    index,
  ): RoundOutcome<ValueT> {
    return arrived.get(index,) ?? {
      modelId,
      // NOT ANSWERED: nothing ever arrived for this position at all.
      voice: {
        heard: false,
        answered: false,
        unreachable: false,
      },
    };
  },);

  /**
   Voices this round actually heard, counted off the outcomes rather than off
   `heardNeeded`: the grace window can add voices after quorum stood, so the
   target is a floor and never the figure.
   */
  const heard = outcomes
    .filter(function wasHeard({ voice, },): boolean {
      return voice.heard;
    },)
    .length;

  /**
   When this round finished, once the grace window closed.
   */
  const finishedAt = monotonicMs();

  // IDS, COUNTS AND DURATIONS ONLY, like every other line a corpus run emits:
  // the stage label, the roster size and the clock. A run directory holds
  // unlicensed corpus wording, and this line is written on every gather.
  //
  // BUILT THROUGH `roundLine` (`stage-round-line.ts`) rather than assembled
  // here, so a reader's own test can build this exact text by calling the
  // same function this call does, instead of retyping its template by hand.
  l.info(
    roundLine({
      stage,
      heard,
      asked: outcomes.length,
      totalMs: finishedAt - startedAt,
      // A round that never reaches quorum stops waiting once nothing is
      // pending, so its "quorum" mark is only when the last ask settled
      // (ledger P12).
      quorum: (heard >= neededAtQuorum)
        ? {
          kind: 'stood',
          toQuorumMs: quorumAt - startedAt,
          inGraceMs: finishedAt - quorumAt,
        }
        : {
          kind: 'never',
          needed: neededAtQuorum,
        },
    },),
  );

  return outcomes;
}

//endregion Stage round
