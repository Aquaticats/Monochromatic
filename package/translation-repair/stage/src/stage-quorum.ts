import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';
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
import {
  reachableQuorum,
  shortBenchStageFinding,
} from './stage-reachable-quorum.ts';
import type { UnreadableCause, } from './recovery-nudge.ts';
import type { StageDecision, } from './stage-decision-call.ts';
import type {
  HeardVoice,
  StageGather,
} from './stage-gather.ts';
import {
  runRecoveryRound,
  type SharedRoundRequest,
} from './stage-recovery-round.ts';
import { runGatherRound, } from './stage-round.ts';
import {
  repeatedRosterIds,
  StageRosterRepeatError,
} from './stage-roster-repeat.ts';
import {
  lostVoiceFindings,
  rosterShortfallFindings,
} from './stage-voice-lost.ts';
import {
  RUN_SEATS,
  type SeatTally,
} from './seat-tally.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Stage quorum
// A stage that loses voices retries exactly the lost ones on fresh
// deadlines (user directive: quota regenerates faster than runs spend, so
// forfeiting voices cheaply leaves capacity unused). Retries stop at
// QUORUM, at least half the roster rounded up, for every stage. A roster
// still short after every round proceeds with what it has and records the
// degradation as findings.
//
// WAITING FOR THE WHOLE ROSTER IS NOT AN OPTION HERE, and used to be. The
// editor and refiner stages passed `retryTarget: 'full-roster'` from
// 2026-08-12 until the user removed the option outright on 2026-08-15:
// waiting on every voice makes one provider-side model degrading for a day
// block every stage that seats it, spending four deadlines per gather on a
// voice that will not come. The property full-roster was chosen to protect,
// no stage decided by a single model, is what the quorum aims at: at least
// two voices (`MIN_STAGE_VOICES`), and editors, refiners and checkers sit at
// three. It is a retry target, not a floor: a gather still short after every
// round proceeds on what it heard and records the shortfall, so one voice can
// still decide a stage then (XingZ6014 slice 87 resolved an issue on one
// usable ballot before ledger L8 stopped an empty report counting as heard).
//
// Critics never used it either, by a separate user decision on 2026-07-23,
// for the same reason stated locally: waiting on a complete roster stalls a
// run when a voice wedges.

/**
 Retry rounds after the initial fan-out;
 the milestone-one benchmark showed one fresh attempt recovers most
 forfeits, and three rounds bound the worst stage wall time at four
 deadlines.
 */
export const STAGE_RETRY_ROUNDS = 3;

/**
 Asks a seat must have been given, every one of them failed, before the run
 stops asking it: the most asks one gather gives a lost seat, since each of its
 rounds asks every seat not yet heard once (`STAGE_RETRY_ROUNDS` rounds after
 the first). A seat that fails every one of those asks has failed a whole
 gather's worth of asks, which is enough to tell it from a seat that merely
 lost a round. An ask fails by throwing or by a stream the round's grace window
 cut with nothing delivered (ledger B761).

 @example
 ```ts
 const withheld = RUN_SEATS.withheld({ minAsks: SEAT_WITHHOLD_ASKS, },);
 ```
 */
export const SEAT_WITHHOLD_ASKS: number = STAGE_RETRY_ROUNDS + 1;

/**
 What the rounds produced: the voices heard, which seats were asked at
 all, so the findings can tell a seat the window spared from one that was
 asked and stayed quiet, which seats no provider served, and which
 answered in a shape nothing could read.
 */
type RoundsOutcome<ValueT,> = {
  /**
   Voices heard across every round.
   */
  readonly collected: readonly HeardVoice<ValueT>[];

  /**
   Seats some round asked.
   */
  readonly asked: ReadonlySet<RosterModelId>;

  /**
   Seats the router refused for want of a wet provider.
   */
  readonly unreachable: ReadonlySet<RosterModelId>;

  /**
   Seats whose answers stayed unreadable after the recovery round.
   */
  readonly unreadable: ReadonlySet<RosterModelId>;
};

/**
 Fans one prompt out to a roster and retries lost voices to quorum.
 Since 2026-09-09 the first round asks a window of quorum plus one spare
 from the bench rotated by the prompt (`stage-fanout-window.ts`), and
 each later round asks the seats not yet asked before the ones it lost,
 on fresh deadlines; the loop stops as soon as half the roster, rounded
 up, is heard, and otherwise ends when the retry rounds are spent.

 @param client - injected model client

 @param modelIds - stage roster

 @param messages - prompt shared by every voice

 @param signal - caller abort honored by every exchange

 @param exchangeTimeoutMs - deadline per exchange

 @param maxAnswerChars - bound on one answer, when the caller knows how
 large its own input was

 @param responseFormat - structured-output constraint

 @param validate - client-side schema guard

 @param stage - stage label for logging and findings

 @param l - logger of the calling stage

 @param maxRetryRounds - rounds after the initial fan-out;
 defaults to {@link STAGE_RETRY_ROUNDS}

 @param graceMs - window a straggler gets after quorum before the round
 abandons it: the launch's round or writer window, from the `StageSettings`
 its entry file read

 @param fanOut - whether a round asks the window of quorum plus one, the
 production default, or the whole bench a fixture scripting every seat asks for

 @param benchStart - seat the bench's rotation starts at in place of the
 prompt's own, which a launch pins in its `StageSettings` so that the seats
 asked move with no sheet text (`rotatedBench`); absent in production

 @returns Heard voices plus quorum verdict and degradation findings

 @throws {@link StageRosterRepeatError} when the roster seats one model more
 than once, before any seat is asked

 @throws {@link StageBenchStartError} when `benchStart` is not a whole number from zero to below the size of the bench, before
 any seat is asked

 @example
 ```ts
 const gather = await gatherStageVoices({ ..., stage: 'critic', l, graceMs: stageSettings.grace.roundMs, },);
 ```
 */
export async function gatherStageVoices<ValueT,>(
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
    maxRetryRounds = STAGE_RETRY_ROUNDS,
    graceMs,
    fanOut = 'window',
    benchStart,
    decision,
    seatTally = RUN_SEATS,
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
    readonly maxRetryRounds?: number;
    readonly graceMs: number;
    readonly fanOut?: FanOutMode;
    readonly benchStart?: number;
    readonly decision?: StageDecision;
    readonly seatTally?: SeatTally;
  }>,
): Promise<StageGather<ValueT>> {
  /**
   Ids the roster seats more than once, which every role's readers collapse
   into one model while the quorum counts them as several voices.
   */
  const duplicated = repeatedRosterIds({ modelIds, },);
  if (duplicated.length > 0)
    throw new StageRosterRepeatError({
      stage,
      duplicated,
    },);

  /**
   Counts of the seats the tally has withheld for failing every ask they
   were given, by a throw or by a stream the round cut with nothing delivered.
   Those seats are neither asked nor counted in the bench this gather sizes its
   quorum on, so the quorum is taken over the seats left.
   */
  const withheldCounts = seatTally.withheld({ minAsks: SEAT_WITHHOLD_ASKS, },);
  /**
   The withheld seats, as a set the gather checks each stage roster against.
   */
  const withheldSeats = new Set<RosterModelId>(withheldCounts.map(
    function seatOf(count,): RosterModelId {
      return count.modelId;
    },
  ),);

  /**
   The roster this gather seats: every model the stage named less the withheld.
   */
  const seated = modelIds.filter(function notWithheld(modelId,): boolean {
    return !withheldSeats.has(modelId,);
  },);
  /**
   Withheld seats the stage named, which the gather logs by name.
   */
  const withheldHere = modelIds.filter(function isWithheld(modelId,): boolean {
    return withheldSeats.has(modelId,);
  },);
  if (withheldHere.length > 0) {
    l.warn(
      `${stage}: not asking ${withheldHere.join(', ',)}, withheld for failing every ask; `
        + 'the quorum is sized on the seats left',
    );
  }

  /**
   Voices a quorum needs: at least half the roster, rounded up, sized on
   the seats a wet provider serves once the router has named the rest.

   Was "strictly more than half", which differs only on EVEN rosters and was
   costing a round there. At six models the old rule demanded 4 while this
   demands 3; at seven both demand 4, so odd rosters are unaffected. User
   decision 2026-08-06, taken when the roster shrank to six: exactly half of
   an even panel is a quorum.

   SIZED ON THE REACHABLE BENCH SINCE 2026-09-09 (`reachableQuorum`): a
   seat the router refuses for want of a wet provider is not a voice the
   gather can wait for, and counting it cost `hulicaijia` its entry that
   evening. Since 2026-10-22 a bench left with exactly its quorum of live
   seats closes one voice below it (owner, "Let the grace open"), and a
   refusal inside a round sizes the need again through
   `heardNeededAfterRefusals`.

   @param unreachable - seats the router has refused so far

   @returns Heard voices the gather needs to close
   */
  function quorumNeededWith(unreachable: number,): number {
    /**
     Quorum on the bench as read so far.
     */
    const quorumNow = reachableQuorum({
      benchSize: seated.length,
      unreachable,
    },);
    return quorumNow.needed;
  }

  /**
   Heard voices accumulated across rounds;
   the round cursor lives inside the named IIFE so its mutation never
   leaks into the surrounding scope.
   */
  const rounds: RoundsOutcome<ValueT> = await (async function collectRounds(): Promise<RoundsOutcome<ValueT>> {
    /**
     Voices collected so far.
     */
    const collected: HeardVoice<ValueT>[] = [];

    /**
     Seats some round asked, so a seat the window spared is never reported
     as lost.
     */
    const asked = new Set<RosterModelId>();

    /**
     Seats the router refused because no wet provider serves them.
     */
    const unreachableSeats = new Set<RosterModelId>();

    /**
     Seats whose latest answer nothing could read, across every round, each
     with why, which picks its recovery wording (ledger P10): a seat leaves the
     moment some round hears it.
     */
    const unreadableSeats = new Map<RosterModelId, UnreadableCause>();

    /**
     Everything a round needs except who to ask, how many to wait for and its
     reserve.

     Hoisted so the recovery round cannot drift from the quorum rounds: it
     sends this unchanged apart from its nudge and its own logger tag, and
     writing these fields twice is how the two would eventually disagree
     about a deadline or a guard.
     */
    const roundRequest: SharedRoundRequest<ValueT> = {
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
      // Conditional spread keeps the question absent instead of undefined.
      ...((decision === undefined) ? {} : { decision, }),
    };

    /**
     Models still owing a reply: not yet asked first, then lost, so a fresh
     seat is tried before a seat that just failed.
     */
    let pending: readonly RosterModelId[] = rotatedBench({
      modelIds: seated,
      messages,
      // Conditional spread keeps the pinned seat absent instead of undefined.
      ...((benchStart === undefined) ? {} : { start: benchStart, }),
    },);

    for (let round = 0; round <= maxRetryRounds; round += 1) {
      if (pending.length === 0)
        break;
      /**
       Voices this round still needs to close, on the bench as the router
       has read it so far.
       */
      const quorumNeeded = quorumNeededWith(unreachableSeats.size,);
      if ((round > 0) && (collected.length >= quorumNeeded))
        break;
      /**
       Seats this round asks: what quorum still needs plus the spare, from
       the front of the pending order.
       */
      const asking = (fanOut === 'whole-bench')
        ? pending
        : askingWindow({
          pending,
          needed: quorumNeeded - collected.length,
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
       This round's outcomes, one per asked model, with anything still in
       flight a grace period after quorum abandoned rather than waited on.
       */
      const outcomes = await runGatherRound<ValueT>({
        ...roundRequest,
        modelIds: asking,
        // A REFUSED SEAT HANDS ITS PLACE TO THE NEXT PENDING ONE in this
        // round (`stage-round.ts`), so the seats past the window are its
        // reserve and the ones it took leave `pending` through
        // `pending.slice(outcomes.length)`.
        reserve: pending.slice(asking.length,),
        heardNeeded: quorumNeeded - collected.length,
        // SIZED AGAIN AT EVERY REFUSAL INSIDE THE ROUND, not only between
        // rounds: round 0 is sized before the router has named a seat, and a
        // bench its refusals leave with exactly its quorum closes one voice
        // below it (owner, 2026-10-22, "Let the grace open").
        heardNeededAfterRefusals: function neededAfter(refused,): number {
          return quorumNeededWith(unreachableSeats.size + refused,) - collected.length;
        },
      },);
      /* oxlint-enable no-await-in-loop */
      for (const { modelId, } of outcomes)
        asked.add(modelId,);

      /**
       Models this round still lost.
       */
      const stillLost: RosterModelId[] = [];

      /**
       Models this round lost to an answer nothing could read.
       */
      const answeredBadly: RosterModelId[] = [];
      for (const outcome of outcomes) {
        if (outcome.voice
          .heard) {
          collected.push({
            modelId: outcome.modelId,
            value: outcome.voice
              .value,
          },);
          unreadableSeats.delete(outcome.modelId,);
          continue;
        }
        stillLost.push(outcome.modelId,);
        if (outcome.voice
          .unreachable)
          unreachableSeats.add(outcome.modelId,);
        if (outcome.voice
          .answered) {
          answeredBadly.push(outcome.modelId,);
          unreadableSeats.set(
            outcome.modelId,
            outcome.voice
              .unreadable,
          );
        }
      }
      // A SEAT THE ROUTER REFUSED IS NOT RE-ASKED. Nothing changes between
      // rounds for a seat no wet provider serves, and re-queuing it spent
      // `hulicaijia`'s retry rounds on Qwen3.8-27B, Kimi-K3 and glm-5.3
      // (2026-09-09) while the seats that could answer waited.
      //
      // NOR IS A SEAT THAT ANSWERED UNREADABLY, until the recovery round
      // (ledger P2). A retry round sends the same prompt, which the
      // prompt-uniqueness cache answers with the same bytes, and the loop
      // once overwrote its list of such seats every round, so a seat that
      // answered badly and was then silent on the re-ask fell off it
      // (TianqiChen66620 slice 15 settled 2 to 2 with one lost).
      pending = [
        ...pending.slice(outcomes.length,),
        ...stillLost.filter(function stillOwed(modelId,): boolean {
          return (!unreachableSeats.has(modelId,)) && (!answeredBadly.includes(modelId,));
        },),
      ];
    }

    /**
     Seats still unreadable after every quorum round, in roster order: every
     seat whose latest answer nothing could read, whichever round it came in.
     SEPARATE FROM `pending`, because the two are re-asked for opposite
     reasons. A pending model is one quorum still NEEDS. One of these is a
     model whose voice is recoverable, since an answer arrived: whole with
     only the shape defeating the guard, or cut at the length limit.
     */
    const unreadable = seated.filter(function stillUnreadable(modelId,): boolean {
      return unreadableSeats.has(modelId,);
    },);

    // ONE RECOVERY ROUND, OUTSIDE THE QUORUM LOOP AND AFTER IT, for the seats
    // that answered and could not be read; `stage-recovery-round.ts` says why
    // only those, and why once.
    if (unreadable.length > 0) {
      /**
       Outcomes of re-asking each unreadable seat once, under its nudge.
       */
      const recovered = await runRecoveryRound<ValueT>({
        roundRequest,
        unreadable,
        causeOf: unreadableSeats,
      },);
      for (const outcome of recovered) {
        if (outcome.voice
          .heard) {
          collected.push({
            modelId: outcome.modelId,
            value: outcome.voice
              .value,
          },);
          unreadableSeats.delete(outcome.modelId,);
        }
      }
    }
    return {
      collected,
      asked,
      unreachable: unreachableSeats,
      unreadable: new Set(unreadableSeats.keys(),),
    };
  })();

  /**
   Voices heard across every round.
   */
  const {
    collected: voices,
    asked,
    unreachable,
    unreadable,
  } = rounds;

  /**
   Quorum the gather closes on, sized on the seats the router could serve.
   */
  const quorum = reachableQuorum({
    benchSize: seated.length,
    unreachable: unreachable.size,
  },);

  /**
   Whether at least half the roster, rounded up, ended up heard, or half
   the reachable bench where that is fewer.
   */
  const quorumMet = voices.length >= quorum.needed;

  /**
   Finding a short bench carries whatever the verdict, so a page decided on
   one is told apart in its artifact.
   */
  const shortFindings: readonly string[] = quorum.short
    ? [
      shortBenchStageFinding({
        stage,
        quorum,
        benchSize: seated.length,
      },),
    ]
    : [];
  if (quorum.short) {
    l.warn(
      `${stage}: bench short of quorum, reachable ${String(quorum.reachable,)} of ${
        String(seated.length,)
      }; closing on ${String(quorum.needed,)} ${
        wordForCount({
          count: quorum.needed,
          one: 'voice',
          many: 'voices',
        },)
      }`,
    );
  }

  /**
   One finding per model that went quiet, emitted even when quorum was met (`stage-voice-lost.ts` says why).
   */
  const lostFindings: readonly string[] = lostVoiceFindings({
    stage,
    seated,
    asked,
    voices,
  },);

  return {
    voices,
    quorumMet,
    quorum: quorum.needed,
    findings: [
      ...shortFindings,
      ...lostFindings,
      ...rosterShortfallFindings({
        stage,
        quorumMet,
        voices,
        asked,
      },),
    ],
    asked,
    unreachable,
    unreadable,
  };
}

//endregion Stage quorum
