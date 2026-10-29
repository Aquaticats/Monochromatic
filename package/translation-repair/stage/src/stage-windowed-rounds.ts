import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';
import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  JsonSchemaResponseFormat,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  askingWindow,
  type FanOutMode,
  rotatedBench,
} from './stage-fanout-window.ts';
import { STAGE_RETRY_ROUNDS, } from './stage-quorum.ts';
import {
  type ReachableQuorum,
  reachableQuorum,
} from './stage-reachable-quorum.ts';
import {
  type RoundOutcome,
  runGatherRound,
} from './stage-round.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Stage windowed rounds
// THE WINDOW FOR THE STAGES THAT READ THEIR OWN ROUND. Six stages ask their
// bench through `runGatherRound` directly and read every seat's outcome
// themselves, lost seats included, because what they record is a seat per
// outcome: the lane contest, section pairing, block pairing, the consolidation
// gate, the naturalness review and the polish gate. They never retried a lost
// voice; they asked everyone once, and on the first `noname` pass of
// 2026-09-09 that was 356 seats (lane contest 96, naturalness review 104,
// gate 72, polish 48, pairing 36) for quorums of about half. This is
// `stage-fanout-window.ts` applied to them: quorum plus one seat first, in
// the prompt's rotation, the rest only when a voice is lost, up to
// `STAGE_RETRY_ROUNDS` retry rounds these stages did not have before. A seat
// that answered unreadably is not re-asked, as before, nor is a seat the
// router refused, whose place in its round passes to the next pending seat.
//
// ONE OUTCOME PER SEAT ASKED, IN ROSTER ORDER. A seat lost in one round and
// heard in the next appears once, heard; a seat the window spared does not
// appear at all, so a caller recording a seat per outcome never counts the
// same seat twice and never records a silence nobody was asked to break.

/**
 What windowed rounds came to.

 @example
 ```ts
 const { outcomes, quorum, unreachable, } = await runWindowedRounds({ ... },);
 ```
 */
export type WindowedRounds<ValueT,> = Readonly<{
  /**
   One outcome per seat asked, in roster order.
   */
  outcomes: readonly RoundOutcome<ValueT>[];

  /**
   Quorum the rounds closed on.
   */
  quorum: ReachableQuorum;

  /**
   Seats of the bench the quorum counted out of reach.
   */
  unreachable: number;
}>;

/**
 Whether the router refused this seat for want of a wet provider.

 @param outcome - one seat's outcome

 @returns Whether the seat was out of reach rather than lost

 @example
 ```ts
 const refused = outcomes.filter(refusedByRouter,).length;
 ```
 */
function refusedByRouter<ValueT,>(outcome: RoundOutcome<ValueT>,): boolean {
  /**
   What came back for the seat.
   */
  const { voice, } = outcome;
  return (!voice.heard) && voice.unreachable;
}

/**
 Names the quorum windowed rounds closed on, and says so in the log when the
 bench was short of its quorum.

 @param stage - stage the rounds served

 @param quorumOver - bench the quorum was taken over

 @param closing - quorum as the rounds closed, and the seats out of reach

 @param l - stage logger

 @returns That quorum and count, unchanged

 @example
 ```ts
 return { outcomes, ...closedOn({ stage, quorumOver, closing, l, },), };
 ```
 */
function closedOn(
  {
    stage,
    quorumOver,
    closing,
    l,
  }: {
    readonly stage: string;
    readonly quorumOver: number;
    readonly closing: Readonly<{
      quorum: ReachableQuorum;
      unreachable: number;
    }>;
    readonly l: Logger;
  },
): Readonly<{
  quorum: ReachableQuorum;
  unreachable: number;
}> {
  /**
   Quorum the rounds closed on.
   */
  const { quorum, } = closing;
  if (quorum.short) {
    l.warn(
      `${stage}: closed on a short bench, ${String(quorum.reachable,)} of ${String(quorumOver,)} ${
        wordForCount({
          count: quorumOver,
          one: 'seat',
          many: 'seats',
        },)
      } `
        + `within reach, quorum ${String(quorum.needed,)} of ${String(quorum.benchQuorum,)}`,
    );
  }
  return closing;
}

/**
 Asks a bench through windowed rounds and returns one outcome per seat
 asked.

 @param client - provider client every ask goes through

 @param modelIds - bench in roster order

 @param messages - prompt every seat is asked, which also fixes the rotation

 @param signal - caller cancellation every ask honors

 @param exchangeTimeoutMs - deadline per exchange

 @param maxAnswerChars - answer volume bound, when the stage sets one

 @param responseFormat - schema every reply must fit

 @param validate - guard a reply must pass to count as heard

 @param stage - stage name for log lines

 @param l - stage logger

 @param quorumOver - bench the quorum is taken over, when the seats asked are
 part of a wider one; the seats asked by default

 @param graceMs - straggler window after quorum: the launch's round window,
 from the `StageSettings` its entry file read

 @param fanOut - window by default; whole bench once for a fixture scripting
 every seat

 @returns Outcomes for the seats asked, in roster order, heard where a
 round heard them and lost otherwise, beside the quorum the rounds closed on

 @example
 ```ts
 const { outcomes, } = await runWindowedRounds({ client, modelIds, messages, signal, exchangeTimeoutMs, responseFormat, validate, stage: 'gate', l, graceMs: stageSettings.grace.roundMs, },);
 ```
 */
export async function runWindowedRounds<ValueT,>(
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
    quorumOver = modelIds.length,
    graceMs,
    fanOut = 'window',
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
    readonly quorumOver?: number;
    readonly graceMs: number;
    readonly fanOut?: FanOutMode;
  }>,
): Promise<WindowedRounds<ValueT>> {
  /**
   Bench seats this stage may not ask at all, which are out of its reach as
   surely as a refused seat is: a confirmation asks only the seats its
   discovery asked, at the discovery's quorum.
   */
  const unaskable = Math.max(
    0,
    quorumOver - modelIds.length,
  );
  /**
   Quorum over the bench once the seats the router has refused are known.

   @param refused - seats the router refused so far

   @returns Quorum and every seat it counted out of reach

   @example
   ```ts
   const closing = quorumWith(refusedSeats.size,);
   ```
   */
  function quorumWith(refused: number,): Pick<WindowedRounds<ValueT>, 'quorum' | 'unreachable'> {
    /**
     Seats out of reach, refused or never askable.
     */
    const unreachable = refused + unaskable;
    return {
      quorum: reachableQuorum({
        benchSize: quorumOver,
        unreachable,
      },),
      unreachable,
    };
  }
  /**
   Voices the quorum needs once the seats the router has refused are known.

   @param refused - seats the router refused so far

   @returns Heard voices the rounds wait for

   @example
   ```ts
   const heardNeeded = neededWith(refusedSeats.size,);
   ```
   */
  function neededWith(refused: number,): number {
    /**
     Quorum as the bench now reads.
     */
    const { quorum, } = quorumWith(refused,);
    return quorum.needed;
  }
  /**
   Everything a round needs except who to ask and how many to wait for.
   */
  const roundRequest = {
    client,
    messages,
    signal,
    exchangeTimeoutMs,
    ...((maxAnswerChars === undefined) ? {} : { maxAnswerChars, }),
    responseFormat,
    validate,
    stage,
    l,
    graceMs,
  };
  if (fanOut === 'whole-bench') {
    /**
     The one round's outcomes, one per seat.
     */
    const outcomes = await runGatherRound<ValueT>({
      ...roundRequest,
      modelIds,
      heardNeeded: neededWith(0,),
      // Sized again at every refusal inside the round (owner, 2026-10-22).
      heardNeededAfterRefusals: neededWith,
    },);
    /**
     Seats the router refused in that round.
     */
    const refused = outcomes.filter(refusedByRouter,);
    return {
      outcomes,
      ...closedOn({
        stage,
        quorumOver,
        closing: quorumWith(refused.length,),
        l,
      },),
    };
  }

  /**
   Seats still owed an answer: unasked first, in the prompt's rotation,
   then lost, so a fresh seat is tried before a seat that just failed.
   */
  const pending: RosterModelId[] = [...rotatedBench({
    modelIds,
    messages,
  },),];
  /**
   Latest outcome per seat asked.
   */
  const latest = new Map<RosterModelId, RoundOutcome<ValueT>>();
  /**
   Seats heard so far.
   */
  const heardSeats = new Set<RosterModelId>();
  /**
   Seats the router refused, which no retry round asks again.
   */
  const refusedSeats = new Set<RosterModelId>();
  for (let round = 0; round <= STAGE_RETRY_ROUNDS; round += 1) {
    // SIZED BEFORE EVERY ROUND on the seats that could still answer (ledger
    // X8): a seat the router refused is no voice to wait for, so a bench it
    // left short closes on its reachable share instead of chasing a quorum
    // those seats cannot reach through every retry round.
    /**
     Voices the quorum needs as the bench now reads.
     */
    const heardNeeded = neededWith(refusedSeats.size,);
    if ((heardSeats.size >= heardNeeded) || (pending.length === 0))
      break;
    /**
     Seats this round asks: what quorum still needs plus the spare.
     */
    const asking = askingWindow({
      pending,
      needed: heardNeeded - heardSeats.size,
    },);
    if (round > 0) {
      l.warn(
        `${stage}: retry round ${String(round,)} asking ${String(asking.length,)} of ${
          String(pending.length,)
        } pending ${
          wordForCount({
            count: pending.length,
            one: 'voice',
            many: 'voices',
          },)
        }`,
      );
    }
    /* oxlint-disable no-await-in-loop -- rounds are sequential by design: each round asks the seats the previous round left unasked or lost */
    /**
     This round's outcomes, one per seat asked.
     */
    const outcomes = await runGatherRound<ValueT>({
      ...roundRequest,
      modelIds: asking,
      // A refused seat hands its place to the next pending one in this round.
      reserve: pending.slice(asking.length,),
      heardNeeded: heardNeeded - heardSeats.size,
      // Sized again at every refusal inside the round (owner, 2026-10-22).
      heardNeededAfterRefusals: function neededAfter(refused,): number {
        return neededWith(refusedSeats.size + refused,) - heardSeats.size;
      },
    },);
    /* oxlint-enable no-await-in-loop */
    // The window, then every reserve seat the round took, leave `pending`.
    pending.splice(
      0,
      outcomes.length,
    );
    for (const outcome of outcomes) {
      /**
       Seat and voice of this outcome.
       */
      const {
        modelId,
        voice,
      } = outcome;
      latest.set(
        modelId,
        outcome,
      );
      if (voice.heard)
        heardSeats.add(modelId,);
      else if (voice.unreachable)
        refusedSeats.add(modelId,);
      // A seat that answered unreadably had its chance, and a seat the router
      // refused has no wet provider until the next seat reading; only a seat
      // lost in transport or to the grace is owed another ask. The refused
      // seat was re-asked every retry round until 2026-09-27, the
      // `hulicaijia` defect `stage-quorum.ts` fixed on 2026-09-09.
      else if (!voice.answered)
        pending.push(modelId,);
    }
  }
  return {
    outcomes: modelIds.flatMap(function inRosterOrder(modelId,): readonly RoundOutcome<ValueT>[] {
      /**
       What this seat's last ask came to, absent when the window spared it.
       */
      const outcome = latest.get(modelId,);
      return (outcome === undefined) ? [] : [outcome,];
    },),
    ...closedOn({
      stage,
      quorumOver,
      closing: quorumWith(refusedSeats.size,),
      l,
    },),
  };
}

//endregion Stage windowed rounds
