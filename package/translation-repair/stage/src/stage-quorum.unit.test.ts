/**
 Tests for stage voice gathering: retries stop at quorum, a straggler is
 abandoned a bounded grace after quorum rather than waited out, and roster
 shortfalls surface as findings.

 The grace cases are the user's standing rule of 2026-08-15 made testable:
 the failure of any one model for the day must not delay the pipeline. Both
 directions are covered, because only the pair distinguishes a grace from a
 cut: a voice arriving inside the window is still heard, and one that never
 arrives costs the window rather than its whole deadline.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import {
  createSeatTally,
  CUT_SHORT_RECOVERY_NUDGE,
  gatherStageVoices,
  OFF_SHAPE_RECOVERY_NUDGE,
  StageRosterRepeatError,
} from '../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_OPENROUTER_ONLY,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  isMeowReply,
  MEOW_FORMAT,
  untilAborted,
} from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Grace the stalling case gives a re-ask before abandoning it.
 */
const RECOVERY_GRACE_MS = 60;

/**
 Logger for the gathers under test.
 */
const l = tagged({ tag: 'stage-quorum-test', },);

/**
 Client whose named model throws rather than answering, and records calls.

 SEPARATE FROM {@link flakyClient} BECAUSE THE TWO LOSSES ARE DIFFERENT. That
 one returns `schema-mismatch`, which is a model that finished and wrote
 something nothing could read. This one never delivers, which is what a
 transport failure looks like from here, and it is the loss the recovery round
 must leave alone.

 @param silentModel - model that throws on every call

 @param calls - shared call log the test asserts on

 @returns Client honouring that script

 @example
 ```ts
 const client = silentClient({ silentModel: SEAT_SYNTHETIC_VISION_WITHHELD, calls, },);
 ```
 */
function silentClient(
  {
    silentModel,
    calls,
  }: {
    readonly silentModel: string;
    readonly calls: Record<string, number>;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      calls[request.modelId] = (calls[request.modelId] ?? 0) + 1;

      if (request.modelId === silentModel)
        throw new Error('scripted transport failure',);

      /**
       Scripted payload for the answering call.
       */
      const scripted: unknown = { meow: request.modelId, };

      if (!request.validate(scripted,))
        throw new Error('scripted payload failed the guard',);

      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

/**
 Client whose named seat is refused by the router for want of a wet provider
 and whose other named seat fails in transport, so a gather can tell the two
 losses apart.

 @param drySeats - seats no provider serves

 @param failingSeat - seat whose transport fails on a wet provider

 @param calls - call count per model, written when given

 @returns Client honouring that script

 @example
 ```ts
 const client = dryBenchClient({ drySeats: [SEAT_HYPER_ONLY,], failingSeat: SEAT_SYNTHETIC_VISION_WITHHELD, },);
 ```
 */
function dryBenchClient(
  {
    drySeats,
    failingSeat,
    calls,
  }: {
    readonly drySeats: readonly string[];
    readonly failingSeat: string;
    readonly calls?: Record<string, number>;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      if (calls !== undefined)
        calls[request.modelId] = (calls[request.modelId] ?? 0) + 1;
      if (drySeats.includes(request.modelId,)) {
        throw new NoProviderForModelError({
          modelId: request.modelId,
          reason: 'every provider serving this model is out of budget',
        },);
      }
      if (request.modelId === failingSeat)
        throw new Error('scripted transport failure',);

      /**
       Scripted payload for the answering call.
       */
      const scripted: unknown = { meow: request.modelId, };

      if (!request.validate(scripted,))
        throw new Error('scripted payload failed the guard',);

      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

/**
 Client that answers only the seats named, and whose every other seat fails in
 transport, recording how often each seat was asked.

 @param answeringSeats - seats that answer

 @param calls - call count per model, written as the gather asks

 @returns Client honouring that script

 @example
 ```ts
 const client = answeringBenchClient({ answeringSeats: [SEAT_HYPER_VISION,], calls, },);
 ```
 */
function answeringBenchClient(
  {
    answeringSeats,
    calls,
  }: {
    readonly answeringSeats: readonly string[];
    readonly calls: Record<string, number>;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      calls[request.modelId] = (calls[request.modelId] ?? 0) + 1;
      if (!answeringSeats.includes(request.modelId,))
        throw new Error('scripted transport failure',);

      /**
       Scripted payload for the answering call.
       */
      const scripted: unknown = { meow: request.modelId, };

      if (!request.validate(scripted,))
        throw new Error('scripted payload failed the guard',);

      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

/**
 The eleven-seat roster of 2026-09-09, in roster order.
 */
const ELEVEN_SEATS: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_VISION,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_ONLY,
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_OPENROUTER_ONLY,
];

/**
 The seven of those seats a Bedrock-alone day cannot serve.
 */
const DRY_SEVEN: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_HYPER_VISION,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_ONLY,
];

/**
 Client whose named model writes one unusable answer and then hangs forever.

 THE SHAPE THE RECOVERY ROUND'S BOUND IS FOR. A model that finished once is
 re-asked, and nothing says the second call comes back: this scripts the worst
 case so the round's time can be read rather than reasoned about.

 @param stallingModel - model that answers unusably once, then never returns

 @param calls - shared call log the test asserts on

 @param cuts - what aborted each hung call, as text, so a case can say the
 round's own cut ended it without timing anything

 @returns Client honouring that script

 @example
 ```ts
 const client = stallingClient({ stallingModel: SEAT_SYNTHETIC_VISION_WITHHELD, calls, cuts, },);
 ```
 */
function stallingClient(
  {
    stallingModel,
    calls,
    cuts,
  }: {
    readonly stallingModel: string;
    readonly calls: Record<string, number>;
    readonly cuts: string[];
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      calls[request.modelId] = (calls[request.modelId] ?? 0) + 1;

      if (request.modelId === stallingModel) {
        if ((calls[request.modelId] ?? 0) === 1) {
          return {
            kind: 'schema-mismatch',
            rawText: '',
            detail: 'scripted flake',
          };
        }

        // NEVER ANSWERS ON ITS OWN: the call ends only when it is aborted, and
        // this scripted client arms no exchange deadline (a provider client
        // arms that), so only the round's own cut can end it, on any machine.
        // A recovery round that waited for it would never end, and the case
        // would fail rather than pass. A 30 s sleep stood here, which let such
        // a round end after 30 s.
        await untilAborted({ signal: request.signal, },);
        cuts.push(String(request.signal.reason,),);
        throw request.signal.reason;
      }

      /**
       Scripted payload for the answering call.
       */
      const scripted: unknown = { meow: request.modelId, };

      if (!request.validate(scripted,))
        throw new Error('scripted payload failed the guard',);

      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

/**
 Client scripted per model: fails until the model's remaining failure
 budget is spent, then answers; records every call.

 A FAILURE IS AN UNREADABLE ANSWER unless `transport` says otherwise. In a
 run the prompt-uniqueness client replays an unreadable answer for the same
 prompt, so only the recovery round's nudged prompt can recover one (ledger
 P2); a transport failure is the weather the retry rounds exist for.

 @param failuresByModel - failures each model serves before answering

 @param calls - shared call log the test asserts on

 @param transport - whether each failure is a transport failure rather than
 an unreadable answer
 */
function flakyClient(
  {
    failuresByModel,
    calls,
    transport = false,
  }: {
    readonly failuresByModel: Readonly<Record<string, number>>;
    readonly calls: Record<string, number>;
    readonly transport?: boolean;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      calls[request.modelId] = (calls[request.modelId] ?? 0) + 1;

      /**
       Failures this model still owes.
       */
      const owed = failuresByModel[request.modelId] ?? 0;
      if (transport && ((calls[request.modelId] ?? 0) <= owed))
        throw new Error('scripted transport failure',);
      if ((calls[request.modelId] ?? 0) <= owed) {
        return {
          kind: 'schema-mismatch',
          rawText: '',
          detail: 'scripted flake',
        };
      }

      /**
       Scripted payload for the answering call.
       */
      const scripted: unknown = { meow: request.modelId, };
      if (!request.validate(scripted,))
        throw new Error('scripted payload failed the guard',);
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

/**
 Client where every model answers at once except one, which either answers
 late or not at all.

 @param hangingModelId - model that does not answer with the others

 @param cut - flag the hung call sets when its abort arrives, with the abort's
 reason as text, which says the round's own cut ended it

 @param lateMs - delay after which it answers anyway; omitted means it never
 answers on its own and waits to be abandoned

 @returns Client honoring that script

 @example
 ```ts
 const client = hangingClient({ hangingModelId, cut, },);
 ```
 */
function hangingClient(
  {
    hangingModelId,
    cut,
    lateMs,
  }: {
    readonly hangingModelId: RosterModelId;
    readonly cut: {
      aborted: boolean;
      reason: string;
    };
    readonly lateMs?: number;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Scripted payload every answering model returns.
       */
      const scripted: unknown = { meow: request.modelId, };
      if (!request.validate(scripted,))
        throw new Error('scripted payload failed the guard',);

      /**
       Answer shared by every model that speaks.
       */
      const answer: ChatJsonOutcome<ValueT> = {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
      if (request.modelId !== hangingModelId)
        return answer;
      if (lateMs !== undefined) {
        await wait(lateMs,);
        return answer;
      }
      await untilAborted({ signal: request.signal, },);
      cut.aborted = true;
      cut.reason = String(request.signal.reason,);
      return {
        kind: 'schema-mismatch',
        rawText: '',
        detail: 'abandoned before answering',
      };
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

await describe({
  name: gatherStageVoices.name,
  children: [
    it({
      name: 'asks everyone once when the first round meets quorum',
      fn: async () => {
        /** Call log shared with the scripted client. */
        const calls: Record<string, number> = {};
        /** Gather over a fully healthy roster. */
        const gather = await gatherStageVoices({
          client: flakyClient({ failuresByModel: {}, calls, },),
          modelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'critic',
          l,
          graceMs: HANG_STOP_MS,
        },);
        expect(gather.voices,).toHaveLength(3,);
        expect(gather.quorumMet,).toBe(true,);
        expect(gather.findings,).toHaveLength(0,);
        expect(calls[SEAT_HYPER_OPENROUTER_VISION_EDITOR],).toBe(1,);
      },
    },),

    it({
      name: 'REFUSES A ROSTER THAT SEATS ONE MODEL TWICE, naming the stage and the repeated id, before any '
        + 'seat is asked',
      fn: async () => {
        /** Call log shared with the scripted client. */
        const calls: Record<string, number> = {};
        /** What the gather rejected with. */
        const refusal = await rejectionOf(async function gatherOverRepeatedSeat(): Promise<unknown> {
          return gatherStageVoices({
            client: flakyClient({ failuresByModel: {}, calls, },),
            modelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
            messages: [{ role: 'user', content: 'meow', },],
            signal: new AbortController().signal,
            exchangeTimeoutMs: HANG_STOP_MS,
            responseFormat: MEOW_FORMAT,
            validate: isMeowReply,
            stage: 'panel',
            l,
            graceMs: HANG_STOP_MS,
          },);
        },);
        expect(refusal,).toBeInstanceOf(StageRosterRepeatError,);
        expect(String(refusal,),).toBe(
          `StageRosterRepeatError: the panel roster seats these models more than once: [${
            SEAT_HYPER_OPENROUTER_VISION_EDITOR
          }]; a repeated id is one model counted as several voices, which meets quorum on fewer `
            + 'independent voices than the roster size promises, and the ballots keyed by model id would '
            + 'keep only one of its replies',
        );
        expect(calls,).toEqual({},);
      },
    },),

    it({
      name: 'NAMES EVERY REPEATED ID ONCE, in the order each first repeats, however often it is seated',
      fn: async () => {
        /** What the gather rejected with. */
        const refusal = await rejectionOf(async function gatherOverRepeatedSeats(): Promise<unknown> {
          return gatherStageVoices({
            client: flakyClient({ failuresByModel: {}, calls: {}, },),
            modelIds: [
              SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            ],
            messages: [{ role: 'user', content: 'meow', },],
            signal: new AbortController().signal,
            exchangeTimeoutMs: HANG_STOP_MS,
            responseFormat: MEOW_FORMAT,
            validate: isMeowReply,
            stage: 'page-title',
            l,
            graceMs: HANG_STOP_MS,
          },);
        },);
        expect(refusal,).toBeInstanceOf(StageRosterRepeatError,);
        // The sentence is pinned by the case that seats one id twice; this
        // case pins which ids it lists and in what order.
        /** Refusal built from the ids the roster repeats, in that order. */
        const expected = new StageRosterRepeatError({
          stage: 'page-title',
          duplicated: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
        },);
        expect(String(refusal,),).toBe(String(expected,),);
      },
    },),

    it({
      name: 'NAMES THE SEATS NO PROVIDER SERVED apart from the voices lost in transport, so a '
        + 'stage sizing its minimum by the reachable bench (owner, 2026-09-09) counts the bench '
        + 'that could answer',
      fn: async () => {
        /** Gather where one seat is refused by the router and one fails on a wet provider. */
        const gather = await gatherStageVoices({
          client: dryBenchClient({
            drySeats: [SEAT_HYPER_ONLY,],
            failingSeat: SEAT_SYNTHETIC_VISION_WITHHELD,
          },),
          modelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD, SEAT_HYPER_ONLY,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'select',
          l,
          fanOut: 'whole-bench',
          graceMs: HANG_STOP_MS,
        },);
        expect(gather.voices,).toHaveLength(2,);
        expect([...gather.unreachable,],).toEqual([SEAT_HYPER_ONLY,],);
        expect(gather.findings,).toContain('stage-voice-lost (select glm-5.3)',);
        expect(gather.findings,).toContain('stage-voice-lost (select hf:moonshotai/Kimi-K3)',);
      },
    },),

    it({
      name: 'SIZES THE QUORUM ON THE REACHABLE BENCH with a two-voice floor and never re-asks a seat the '
        + 'router refused: with seven of eleven seats served by no wet provider the four that are get heard, '
        + 'the gather closes short-bench, and each refused seat was asked exactly once (hulicaijia, '
        + '2026-09-09: the archive block review heard 5 of 11 with three seats refused, spent four retry '
        + 'rounds re-asking them and interrupted the entry provider-unavailable)',
      fn: async () => {
        /** Call count per model. */
        const calls: Record<string, number> = {};
        /** Gather on a Bedrock-alone bench: seven seats refused, four answering. */
        const gather = await gatherStageVoices({
          client: dryBenchClient({
            drySeats: [...DRY_SEVEN,],
            failingSeat: 'nobody',
            calls,
          },),
          modelIds: [...ELEVEN_SEATS,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'archive-block-review',
          l,
          graceMs: HANG_STOP_MS,
        },);
        expect(gather.quorumMet,).toBe(true,);
        expect(gather.voices,).toHaveLength(4,);
        expect([...gather.unreachable,].toSorted(),).toEqual([...DRY_SEVEN,].toSorted(),);
        expect(gather.findings,).toContain(
          'stage-short-bench (archive-block-review reachable 4 of 11, quorum 2)',
        );
        for (const seat of DRY_SEVEN)
          expect(calls[seat],).toBe(1,);
      },
    },),

    it({
      name: 'SIZES THE QUORUM ON THE SEATS LEFT once the run has withheld a seat for throwing on every ask: with '
        + 'two of ten seats withheld the bench is eight, the quorum four, and neither withheld seat is asked again',
      fn: async () => {
        /** Tally holding two seats withheld, each recorded as having thrown on four asks and answered none. */
        const seatTally = createSeatTally();
        const withheldSeats: readonly RosterModelId[] = [SEAT_BEDROCK_ONLY_TEXT, SEAT_OPENROUTER_ONLY,];
        for (const seat of withheldSeats) {
          seatTally.record({ modelId: seat, outcome: 'threw', },);
          seatTally.record({ modelId: seat, outcome: 'threw', },);
          seatTally.record({ modelId: seat, outcome: 'threw', },);
          seatTally.record({ modelId: seat, outcome: 'threw', },);
        }
        /** Call count per model. */
        const calls: Record<string, number> = {};
        /** Gather over ten seats, of which four answer and the rest fail in transport. */
        const gather = await gatherStageVoices({
          client: answeringBenchClient({
            answeringSeats: [
              SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              SEAT_SYNTHETIC_VISION_WITHHELD,
              SEAT_HYPER_VISION,
            ],
            calls,
          },),
          modelIds: [
            SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            SEAT_SYNTHETIC_VISION_WITHHELD,
            SEAT_HYPER_VISION,
            SEAT_SYNTHETIC_TEXT_EVERYWHERE,
            SEAT_HYPER_OPENROUTER_UNMEASURED,
            SEAT_BEDROCK_ONLY_VISION_UNSEATED,
            SEAT_HYPER_ONLY,
            ...withheldSeats,
          ],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'archive-block-review',
          l,
          graceMs: HANG_STOP_MS,
          seatTally,
        },);
        expect(gather.quorum,).toBe(4,);
        expect(gather.quorumMet,).toBe(true,);
        expect(gather.voices,).toHaveLength(4,);
        for (const seat of withheldSeats)
          expect(calls[seat],).toBeUndefined();
      },
    },),

    it({
      name: 'HANDS BACK THE QUORUM IT CLOSED ON: six for a bench of eleven every seat of which answers, and two '
        + 'for the same bench with seven seats no provider serves, the number its short-bench finding names',
      fn: async () => {
        /**
         Gathers over the eleven seats with the named ones refused by the router.
         */
        async function gatherWithDry(
          { drySeats, }: { readonly drySeats: readonly string[]; },
        ): Promise<{ readonly quorum: number; readonly findings: readonly string[]; }> {
          /** The round over the bench. */
          const gather = await gatherStageVoices({
            client: dryBenchClient({
              drySeats: [...drySeats,],
              failingSeat: 'nobody',
            },),
            modelIds: [...ELEVEN_SEATS,],
            messages: [{ role: 'user', content: 'meow', },],
            signal: new AbortController().signal,
            exchangeTimeoutMs: HANG_STOP_MS,
            responseFormat: MEOW_FORMAT,
            validate: isMeowReply,
            stage: 'archive-block-review',
            l,
            fanOut: 'whole-bench',
            graceMs: HANG_STOP_MS,
          },);
          return {
            quorum: gather.quorum,
            findings: gather.findings,
          };
        }
        expect(await gatherWithDry({ drySeats: [], },),).toEqual({
          quorum: 6,
          findings: [],
        },);
        expect(await gatherWithDry({ drySeats: DRY_SEVEN, },),).toEqual({
          quorum: 2,
          findings: [
            'stage-short-bench (archive-block-review reachable 4 of 11, quorum 2)',
            ...ELEVEN_SEATS
              .filter(seat => DRY_SEVEN.includes(seat,))
              .map(seat => `stage-voice-lost (archive-block-review ${seat})`),
            'stage-roster-incomplete (archive-block-review 4/11)',
          ],
        },);
      },
    },),

    it({
      name: 'STILL READS A BENCH WITH NO REACHABLE SEAT AS AN OUTAGE, since the floor is two voices and none '
        + 'can answer',
      fn: async () => {
        /** Gather where every seat is refused. */
        const gather = await gatherStageVoices({
          client: dryBenchClient({
            drySeats: [SEAT_HYPER_ONLY, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
            failingSeat: 'nobody',
          },),
          modelIds: [SEAT_HYPER_ONLY, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'archive-block-review',
          l,
          fanOut: 'whole-bench',
          graceMs: HANG_STOP_MS,
        },);
        expect(gather.quorumMet,).toBe(false,);
        expect(gather.voices,).toHaveLength(0,);
        expect(gather.findings,).toContain('stage-quorum-unmet (archive-block-review 0/3)',);
      },
    },),

    it({
      name: 'STOPS RETRYING FOR QUORUM once it stands, and reads an unusable '
        + 'answer exactly once more, which is two calls and never a third',
      fn: async () => {
        /** Call log shared with the scripted client. */
        const calls: Record<string, number> = {};
        /** Gather where two of three answer immediately and one never does. */
        const gather = await gatherStageVoices({
          client: flakyClient({
            failuresByModel: { [SEAT_SYNTHETIC_VISION_WITHHELD]: 99, },
            calls,
          },),
          modelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'panel',
          l,
          graceMs: HANG_STOP_MS,
        },);
        expect(gather.voices,).toHaveLength(2,);
        expect(gather.quorumMet,).toBe(true,);
        // Quorum (2 of 3) was met after the first round, so the quorum loop
        // never runs a retry round. The one extra call is the recovery round:
        // this fixture fails with `schema-mismatch`, which means the model
        // finished and only the shape was wrong, and that is the one loss a
        // fresh call can fix. Two, never three: the recovery round runs once
        // whatever it finds, so a model broken all day costs one extra call per
        // gather rather than a ladder.
        expect(calls[SEAT_SYNTHETIC_VISION_WITHHELD],).toBe(2,);
      },
    },),

    it({
      name: 'RE-ASKS AN UNREADABLE ANSWER WITH A DIFFERENT PROMPT, the original plus a complaint '
        + 'naming what happened, so the prompt-uniqueness cache cannot serve the same unreadable '
        + 'bytes back: five recovery rounds over two passes on 2026-09-02 recovered nothing in 0 '
        + 'to 1 ms each',
      fn: async () => {
        /** Text of every message each call to the flaky model carried, in call order. */
        const seen: (readonly string[])[] = [];
        /** Client whose named model reads badly until the prompt tells it what went wrong. */
        const client: ModelClient = {
          chatText: async () => {
            throw new Error('chatText unused',);
          },
          chatJson: async <ValueT,>(
            request: ChatJsonRequest<ValueT>,
          ): Promise<ChatJsonOutcome<ValueT>> => {
            /**
             Scripted payload for an answering call.
             */
            const scripted: unknown = { meow: request.modelId, };
            if (request.modelId === SEAT_SYNTHETIC_VISION_WITHHELD) {
              /**
               This call's prompt as plain text, a vision part reading as
               nothing since none is sent here.
               */
              const texts = request.messages
                .map(function textOf(message,): string {
                  return ((typeof message.content) === 'string') ? message.content : '';
                },);
              seen.push(texts,);
              /**
               Whether this prompt carries the recovery complaint.
               */
              const nudged = texts.at(-1,)
                ?.includes('could not be read',) ?? false;
              if (!nudged) {
                return {
                  kind: 'schema-mismatch',
                  rawText: '',
                  detail: 'scripted unreadable answer',
                };
              }
            }
            if (!request.validate(scripted,))
              throw new Error('scripted payload failed the guard',);
            return {
              kind: 'ok',
              value: scripted,
              rawText: JSON.stringify(scripted,),
            };
          },
          quotas: async () => {
            throw new Error('quotas unused',);
          },
        };

        const gather = await gatherStageVoices({
          client,
          modelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'panel',
          l,
          graceMs: HANG_STOP_MS,
        },);
        // The recovery round heard the voice the first round could not read.
        expect(gather.voices,).toHaveLength(3,);
        expect(gather.findings,).toHaveLength(0,);
        expect([...gather.unreadable,],).toEqual([],);
        // Two calls: the first with the stage's prompt, the second with the
        // same prompt plus the complaint, and nothing else about it changed.
        expect(seen,).toHaveLength(2,);
        expect(seen[0],).toEqual(['meow',],);
        expect(seen[1]?.length,).toBe(2,);
        expect(seen[1]?.[0],).toBe('meow',);
        expect(seen[1]?.[1],).toContain('could not be read',);
      },
    },),

    it({
      name: 'NAMES the seats whose answers stayed unreadable after the recovery round, so a stage can tell '
        + 'a bench that answered from one that fell silent (class thirty-one: on 2026-09-16 Mio13 read '
        + 'seven cap-cut replies as an outage)',
      fn: async () => {
        /** Client whose named model answers, but in a shape nothing can read, every time. */
        const client: ModelClient = {
          chatText: async () => {
            throw new Error('chatText unused',);
          },
          chatJson: async <ValueT,>(
            request: ChatJsonRequest<ValueT>,
          ): Promise<ChatJsonOutcome<ValueT>> => {
            if (request.modelId === SEAT_SYNTHETIC_VISION_WITHHELD) {
              return {
                kind: 'schema-mismatch',
                rawText: '',
                reason: 'truncated-completion',
                detail: 'provider reported a truncating completion (model stopped with finish_reason=length)',
              };
            }
            /**
             Scripted payload for an answering call.
             */
            const scripted: unknown = { meow: request.modelId, };
            if (!request.validate(scripted,))
              throw new Error('scripted payload failed the guard',);
            return {
              kind: 'ok',
              value: scripted,
              rawText: JSON.stringify(scripted,),
            };
          },
          quotas: async () => {
            throw new Error('quotas unused',);
          },
        };
        const gather = await gatherStageVoices({
          client,
          modelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'panel',
          l,
          graceMs: HANG_STOP_MS,
        },);
        expect(gather.voices,).toHaveLength(2,);
        expect(gather.quorumMet,).toBe(true,);
        expect([...gather.unreadable,],).toEqual([SEAT_SYNTHETIC_VISION_WITHHELD,],);
        expect([...gather.unreachable,],).toEqual([],);
      },
    },),
    it({
      name: 'counts exactly half of an even roster as quorum',
      fn: async () => {
        /** Call log shared with the scripted client. */
        const calls: Record<string, number> = {};
        /** Four-model roster where exactly two never answer. */
        const gather = await gatherStageVoices({
          client: flakyClient({
            failuresByModel: {
              [SEAT_SYNTHETIC_VISION_WITHHELD]: 99,
              [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: 99,
            },
            calls,
          },),
          modelIds: [
            SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            SEAT_SYNTHETIC_VISION_WITHHELD,
            SEAT_SYNTHETIC_TEXT_EVERYWHERE,
          ],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'panel',
          l,
          graceMs: HANG_STOP_MS,
          // The case names the two seats that never answer, so every seat is asked: a window of three, rotated by the prompt, leaves
          // one of them unasked when it holds both healthy seats, and an unasked seat is not a lost one.
          fanOut: 'whole-bench',
        },);
        // 2 of 4 is exactly half. The superseded "strictly more than half"
        // rule would have called this short and burned every retry round;
        // ceil(4 / 2) = 2 makes it a quorum.
        expect(gather.voices,).toHaveLength(2,);
        expect(gather.quorumMet,).toBe(true,);
        // Quorum held, and two models still never answered. This assertion
        // read `toHaveLength(0)` until 2026-08-13, which is precisely the case
        // the findings dropped: a healthy-looking stage silently short two
        // voices, recorded nowhere the artifact could carry.
        expect(gather.findings,).toContain('stage-voice-lost (panel hf:moonshotai/Kimi-K3)',);
        expect(gather.findings,).toContain('stage-voice-lost (panel hf:openai/gpt-oss-120b)',);
        // TWO findings, not one naming two models, so counting them counts
        // voices lost rather than gathers that lost at least one.
        expect(
          gather.findings.filter(function isLoss(finding,) {
            return finding.startsWith('stage-voice-lost',);
          },),
        ).toHaveLength(2,);
      },
    },),

    it({
      name: 'NAMES the models that went quiet even when quorum was met, since '
        + 'voice loss reached only a log line before this and every question '
        + 'about it, which model and which stage, was answerable solely from a '
        + 'captured run log. On 2026-08-13 a run wrote its log into a pipe '
        + 'whose reader had exited, so the losses happened and nothing kept '
        + 'them; findings travel into the durable per-entry artifact instead',
      fn: async () => {
        /** Call log shared with the scripted client. */
        const calls: Record<string, number> = {};
        /** Gather where one model of three never answers. */
        const gather = await gatherStageVoices({
          client: flakyClient({
            failuresByModel: { [SEAT_SYNTHETIC_VISION_WITHHELD]: 99, },
            calls,
          },),
          modelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'critic',
          l,
          graceMs: HANG_STOP_MS,
        },);
        expect(gather.quorumMet,).toBe(true,);
        expect(gather.findings,).toContain('stage-voice-lost (critic hf:moonshotai/Kimi-K3)',);
      },
    },),

    it({
      name: 'records NOTHING when the whole roster answered, so a clean stage '
        + 'stays distinguishable from a degraded one rather than every entry '
        + 'carrying a finding nobody can act on',
      fn: async () => {
        /** Call log shared with the scripted client. */
        const calls: Record<string, number> = {};
        /** Gather over a fully healthy roster. */
        const gather = await gatherStageVoices({
          client: flakyClient({ failuresByModel: {}, calls, },),
          modelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'critic',
          l,
          graceMs: HANG_STOP_MS,
        },);
        expect(gather.findings,).toHaveLength(0,);
      },
    },),

    it({
      name: 'still refuses quorum one voice below half an even roster',
      fn: async () => {
        /** Call log shared with the scripted client. */
        const calls: Record<string, number> = {};
        /** Four-model roster where only one ever answers. */
        const gather = await gatherStageVoices({
          client: flakyClient({
            failuresByModel: {
              [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 99,
              [SEAT_SYNTHETIC_VISION_WITHHELD]: 99,
              [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: 99,
            },
            calls,
          },),
          modelIds: [
            SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            SEAT_SYNTHETIC_VISION_WITHHELD,
            SEAT_SYNTHETIC_TEXT_EVERYWHERE,
          ],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'panel',
          l,
          graceMs: HANG_STOP_MS,
        },);
        expect(gather.voices,).toHaveLength(1,);
        expect(gather.quorumMet,).toBe(false,);
        expect(gather.findings,).not.toHaveLength(0,);
      },
    },),

    it({
      name: 'RETRIES lost voices to quorum on a 1-of-6 round from a window of quorum plus one, drawing the '
        + 'unasked seats before re-asking the lost, and never spends the eleven calls the whole-bench '
        + 'fan-out did',
      fn: async () => {
        /** Call log shared with the scripted client. */
        const calls: Record<string, number> = {};
        /** Full six-model roster where five fail once then answer. */
        const roster: readonly RosterModelId[] = [
          SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          SEAT_HYPER_VISION,
          SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          SEAT_SYNTHETIC_VISION_WITHHELD,
          SEAT_HYPER_OPENROUTER_UNMEASURED,
          SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        ];
        /** Gather recovering the milestone-two weather pattern. */
        const gather = await gatherStageVoices({
          client: flakyClient({
            failuresByModel: Object.fromEntries(
              roster.slice(1,).map(function toFailure(modelId,) {
                return [modelId, 1,];
              },),
            ),
            calls,
            transport: true,
          },),
          modelIds: roster,
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'critic',
          l,
          graceMs: HANG_STOP_MS,
        },);
        expect(gather.quorumMet,).toBe(true,);
        expect(gather.voices.length,).toBeGreaterThanOrEqual(3,);
        expect(gather.voices.length,).toBeLessThanOrEqual(6,);
        // The healthy seat answers on its first ask wherever the rotation put it.
        expect(calls[SEAT_HYPER_OPENROUTER_VISION_EDITOR],).toBe(1,);
        /**
         Every call the gather made, whichever seats the rotation asked first.
         */
        const total = Object.values(calls,).reduce(function add(
          sum,
          count,
        ): number {
          return sum + count;
        }, 0,);
        // Six seats asked at once and five re-asked was eleven calls; a window
        // of four with the two unasked seats drawn before the lost ones stops at
        // quorum inside nine.
        expect(total,).toBeLessThanOrEqual(9,);
      },
    },),

    it({
      name: 'ASKS quorum plus one seat on a healthy six-model roster and never the other two, so a round '
        + 'that reaches quorum on its window spends four calls where the whole bench spent six',
      fn: async () => {
        /** Call log shared with the scripted client. */
        const calls: Record<string, number> = {};
        /** Full six-model roster, everyone answering at once. */
        const roster: readonly RosterModelId[] = [
          SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          SEAT_HYPER_VISION,
          SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          SEAT_SYNTHETIC_VISION_WITHHELD,
          SEAT_HYPER_OPENROUTER_UNMEASURED,
          SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        ];
        /** Gather over the healthy roster. */
        const gather = await gatherStageVoices({
          client: flakyClient({ failuresByModel: {}, calls, },),
          modelIds: roster,
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'critic',
          l,
          graceMs: HANG_STOP_MS,
        },);
        expect(gather.quorumMet,).toBe(true,);
        expect(gather.voices,).toHaveLength(4,);
        expect(gather.findings,).toHaveLength(0,);
        expect(Object.keys(calls,),).toHaveLength(4,);
        for (const count of Object.values(calls,))
          expect(count,).toBe(1,);
      },
    },),

    it({
      name: 'ASKS THE WINDOW FROM THE SEAT A LAUNCH PINS, whatever the prompt says, so a fixture seats the same voices '
        + 'whatever the sheet text is (n166, Z3)',
      fn: async () => {
        /** Call log shared with the scripted client. */
        const calls: Record<string, number> = {};
        /** Full six-model roster, everyone answering at once. */
        const roster: readonly RosterModelId[] = [
          SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          SEAT_HYPER_VISION,
          SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          SEAT_SYNTHETIC_VISION_WITHHELD,
          SEAT_HYPER_OPENROUTER_UNMEASURED,
          SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        ];
        /** Gather over the healthy roster, the rotation pinned at the third seat. */
        const gather = await gatherStageVoices({
          client: flakyClient({ failuresByModel: {}, calls, },),
          modelIds: roster,
          messages: [{ role: 'user', content: 'a prompt whose own rotation is somewhere else', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'critic',
          l,
          graceMs: HANG_STOP_MS,
          benchStart: 2,
        },);
        expect(gather.voices,).toHaveLength(4,);
        expect(Object.keys(calls,).toSorted(),).toEqual(roster.slice(2,).toSorted(),);
      },
    },),

    it({
      name: 'records a finding and proceeds when the rounds cannot reach quorum',
      fn: async () => {
        /** Call log shared with the scripted client. */
        const calls: Record<string, number> = {};
        /** Roster where only one of three ever answers. */
        const gather = await gatherStageVoices({
          client: flakyClient({
            failuresByModel: {
              [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 99,
              [SEAT_SYNTHETIC_VISION_WITHHELD]: 99,
            },
            calls,
          },),
          modelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'checker',
          l,
          graceMs: HANG_STOP_MS,
        },);
        expect(gather.voices,).toHaveLength(1,);
        expect(gather.quorumMet,).toBe(false,);
        expect(gather.findings,).toContain('stage-quorum-unmet (checker 1/3)',);
        // The initial ask and the one recovery round (ledger P2): a seat that
        // answered in a shape nothing could read is not re-asked in the retry
        // rounds, where the prompt-uniqueness cache would serve it the same
        // bytes, only in the recovery round, whose nudge makes the prompt new.
        expect(calls[SEAT_SYNTHETIC_VISION_NO_OPENROUTER],).toBe(2,);
      },
    },),

    it({
      name: 'STOPS at quorum instead of chasing a voice that will not come, and '
        + 'still records the shortfall: waiting for the whole roster let one '
        + 'model degraded for a day stall every gather that seated it, which '
        + 'is why the user removed that target on 2026-08-15',
      fn: async () => {
        /** Call log shared with the scripted client. */
        const calls: Record<string, number> = {};
        /** Gather where one voice never answers at all. */
        const gather = await gatherStageVoices({
          client: flakyClient({
            failuresByModel: { [SEAT_SYNTHETIC_VISION_WITHHELD]: 99, },
            calls,
          },),
          modelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'critic',
          l,
          graceMs: HANG_STOP_MS,
        },);
        expect(gather.voices,).toHaveLength(2,);
        expect(gather.quorumMet,).toBe(true,);
        // The whole point: no retry ROUNDS, so a degraded model costs one
        // deadline per gather rather than four. The second call is the single
        // recovery read of an answer that came back unusable.
        expect(calls[SEAT_SYNTHETIC_VISION_WITHHELD],).toBe(2,);
        // Met quorum reads as healthy everywhere else, so the shortfall is
        // recorded anyway, both as a ratio and by name.
        expect(gather.findings,).toContain('stage-roster-incomplete (critic 2/3)',);
        expect(gather.findings,).toContain('stage-voice-lost (critic hf:moonshotai/Kimi-K3)',);
      },
    },),

    it({
      name: 'RECOVERS a voice quorum did not need, which is the whole reason the '
        + 'recovery round exists: the model finished and only its shape was wrong',
      fn: async () => {
        /** Call log shared with the scripted client. */
        const calls: Record<string, number> = {};
        /** Two answer at once, and the third writes one unusable answer first. */
        const gather = await gatherStageVoices({
          client: flakyClient({
            failuresByModel: { [SEAT_SYNTHETIC_VISION_WITHHELD]: 1, },
            calls,
          },),
          modelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'panel',
          l,
          graceMs: HANG_STOP_MS,
        },);

        // Quorum (2 of 3) stood after the first round, so the quorum loop was
        // finished and the old contract would have shipped two voices. The
        // third is here because the recovery round asked again.
        expect(gather.voices,).toHaveLength(3,);
        expect(calls[SEAT_SYNTHETIC_VISION_WITHHELD],).toBe(2,);

        // A full roster leaves nothing to report as missing.
        expect(gather.findings,).toHaveLength(0,);
      },
    },),

    it({
      name: 'RECOVERS A SEAT UNREADABLE IN ANY ROUND, not only the last (ledger P2): a seat that answered '
        + 'unreadably and was then silent on the same prompt fell off the recovery list, and TianqiChen66620 '
        + 'slice 15 settled 2 to 2 with such a voice lost',
      fn: async () => {
        /** Call log shared with the scripted client. */
        const calls: Record<string, number> = {};
        /** Seat that answers unreadably first, is silent on the same prompt, and answers the nudge. */
        const recovering = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;
        /** Seat that never delivers. */
        const silent = SEAT_SYNTHETIC_VISION_WITHHELD;
        /** Client scripting the three seats. */
        const client: ModelClient = {
          chatText: async () => {
            throw new Error('chatText unused',);
          },
          chatJson: async <ValueT,>(
            request: ChatJsonRequest<ValueT>,
          ): Promise<ChatJsonOutcome<ValueT>> => {
            calls[request.modelId] = (calls[request.modelId] ?? 0) + 1;
            /** Whether this ask carries the recovery nudge. */
            const nudged = request.messages.at(-1,)?.content === OFF_SHAPE_RECOVERY_NUDGE.content;
            if (request.modelId === silent)
              throw new Error('scripted transport failure',);
            if ((request.modelId === recovering) && (!nudged)) {
              if (calls[request.modelId] === 1) {
                return {
                  kind: 'schema-mismatch',
                  rawText: '',
                  detail: 'scripted unreadable answer',
                };
              }
              throw new Error('scripted transport failure',);
            }
            /** Scripted payload for the answering call. */
            const scripted: unknown = { meow: request.modelId, };
            if (!request.validate(scripted,))
              throw new Error('scripted payload failed the guard',);
            return {
              kind: 'ok',
              value: scripted,
              rawText: JSON.stringify(scripted,),
            };
          },
          quotas: async () => {
            throw new Error('quotas unused',);
          },
        };
        /** Gather over the three seats. */
        const gather = await gatherStageVoices({
          client,
          modelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, recovering, silent,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'gate',
          l,
          graceMs: HANG_STOP_MS,
        },);
        expect(gather.voices.map(function seatOf(voice,): string {
          return voice.modelId;
        },),).toContain(recovering,);
        expect(gather.quorumMet,).toBe(true,);
      },
    },),

    it({
      name: 'NUDGES A REPLY CUT AT THE LENGTH LIMIT APART FROM ONE OFF THE SHAPE (ledger P10): the recovery round '
        + 'told a model its reply "did not match the required response shape" when the cap had cut it before its '
        + 'answer, which was 74 of the 75 cap-cut deepseek replies on Hyper',
      fn: async () => {
        /** Last message of each seat's nudged ask, by seat. */
        const nudges: Record<string, string> = {};
        /** Seat whose first reply the length limit cut. */
        const cutShort = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;
        /** Seat whose first reply parsed to nothing the guard reads. */
        const offShape = SEAT_SYNTHETIC_VISION_WITHHELD;
        /** Client scripting the three seats. */
        const client: ModelClient = {
          chatText: async () => {
            throw new Error('chatText unused',);
          },
          chatJson: async <ValueT,>(
            request: ChatJsonRequest<ValueT>,
          ): Promise<ChatJsonOutcome<ValueT>> => {
            /** Whether this ask carries anything past the stage's own prompt. */
            const nudged = request.messages.length > 1;
            if (nudged)
              nudges[request.modelId] = JSON.stringify(request.messages.at(-1,)?.content ?? '',);
            if ((request.modelId === cutShort) && (!nudged)) {
              return {
                kind: 'schema-mismatch',
                rawText: '',
                reason: 'truncated-completion',
                detail: 'provider reported a truncating completion',
              };
            }
            if ((request.modelId === offShape) && (!nudged)) {
              return {
                kind: 'schema-mismatch',
                rawText: 'purr',
                reason: 'unparseable-json',
                detail: 'scripted unparseable answer',
              };
            }
            /** Scripted payload for the answering call. */
            const scripted: unknown = { meow: request.modelId, };
            if (!request.validate(scripted,))
              throw new Error('scripted payload failed the guard',);
            return {
              kind: 'ok',
              value: scripted,
              rawText: JSON.stringify(scripted,),
            };
          },
          quotas: async () => {
            throw new Error('quotas unused',);
          },
        };
        /** Gather over the three seats. */
        const gather = await gatherStageVoices({
          client,
          modelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, cutShort, offShape,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'gate',
          l,
          graceMs: HANG_STOP_MS,
        },);
        expect({
          heard: gather.voices.length,
          cutShortNudged: cutShort in nudges,
          offShapeNudged: offShape in nudges,
          apart: nudges[cutShort] !== nudges[offShape],
          cutShortWording: nudges[cutShort] === JSON.stringify(CUT_SHORT_RECOVERY_NUDGE.content,),
          offShapeWording: nudges[offShape] === JSON.stringify(OFF_SHAPE_RECOVERY_NUDGE.content,),
        },).toEqual({
          heard: 3,
          cutShortNudged: true,
          offShapeNudged: true,
          apart: true,
          cutShortWording: true,
          offShapeWording: true,
        },);
      },
    },),

    it({
      name: 'REFUSES to re-ask a voice that never answered, since a model still '
        + 'thinking is bought a second deadline and not a second answer',
      fn: async () => {
        /** Call log shared with the scripted client. */
        const calls: Record<string, number> = {};
        /** Two answer at once, and the third never delivers anything. */
        const gather = await gatherStageVoices({
          client: silentClient({
            silentModel: SEAT_SYNTHETIC_VISION_WITHHELD,
            calls,
          },),
          modelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'panel',
          l,
          graceMs: HANG_STOP_MS,
        },);

        expect(gather.voices,).toHaveLength(2,);
        expect(gather.quorumMet,).toBe(true,);

        // ASKED ONCE. Quorum stood, so no retry round ran, and the recovery
        // round skipped this model because it never answered at all.
        expect(calls[SEAT_SYNTHETIC_VISION_WITHHELD],).toBe(1,);
        expect(gather.findings,).toContain('stage-voice-lost (panel hf:moonshotai/Kimi-K3)',);
      },
    },),

    it({
      name: 'BOUNDS the recovery round at one grace window, so a re-ask that '
        + 'hangs costs the window rather than a whole exchange deadline',
      fn: async () => {
        /** Call log shared with the scripted client. */
        const calls: Record<string, number> = {};

        /** What ended the hung re-ask, the only figure this case reads. */
        const cuts: string[] = [];

        /** Two answer at once; the third answers unusably, then hangs. */
        const gather = await gatherStageVoices({
          client: stallingClient({
            stallingModel: SEAT_SYNTHETIC_VISION_WITHHELD,
            calls,
            cuts,
          },),
          modelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'panel',
          l,
          graceMs: RECOVERY_GRACE_MS,
        },);

        // The re-ask happened and never came back, so the roster is still two.
        expect(calls[SEAT_SYNTHETIC_VISION_WITHHELD],).toBe(2,);
        expect(gather.voices,).toHaveLength(2,);

        // THE FIGURE THIS CASE EXISTS FOR. The hung re-ask ends only when
        // something aborts it, and this client arms no exchange deadline, so a
        // recovery round that waited for it would never end; the case ending,
        // with the re-ask aborted once by the round's own cut, is the bound. A
        // ceiling on the elapsed time stood here, which a loaded machine could
        // outlast (`mistake-prevention.md`, "Tests on the real clock").
        expect(cuts,).toEqual(['StragglerCutError: the round closed its grace window on this call after quorum stood',],);
      },
    },),

    it({
      name: 'retries a one-model roster until its voice is heard',
      fn: async () => {
        /** Call log shared with the scripted client. */
        const calls: Record<string, number> = {};
        /** Editor-style gather over a roster of one. */
        const gather = await gatherStageVoices({
          client: flakyClient({
            failuresByModel: { [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 2, },
            calls,
            transport: true,
          },),
          modelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'editor',
          l,
          graceMs: HANG_STOP_MS,
        },);
        expect(gather.voices,).toHaveLength(1,);
        expect(gather.quorumMet,).toBe(true,);
        expect(calls[SEAT_HYPER_OPENROUTER_VISION_EDITOR],).toBe(3,);
      },
    },),

    it({
      name: 'ABANDONS a voice that has not answered a grace period after '
        + 'quorum, so one model degraded for the day costs a stage that '
        + 'window rather than its whole per-call deadline. The stub answers '
        + 'only when aborted, so this passes only if the cut actually reached '
        + 'the call',
      fn: async () => {
        /** Whether the hung call saw its abort, and what the abort said. */
        const cut = {
          aborted: false,
          reason: '',
        };

        /** Gather where one model never answers on its own. */
        const gather = await gatherStageVoices({
          client: hangingClient({
            hangingModelId: SEAT_SYNTHETIC_VISION_WITHHELD,
            cut,
          },),
          modelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          // Never armed by this scripted client (a provider client arms its
          // deadline), so the window is the only bound that can end the call.
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'critic',
          l,
          graceMs: 50,
        },);
        expect(gather.voices,).toHaveLength(2,);
        expect(gather.quorumMet,).toBe(true,);
        // THE ROUND'S OWN CUT ended the hung call: a gather that waited for it
        // rather than for the window would never end. Read off the abort rather
        // than the elapsed time, which a loaded machine could stretch past any
        // ceiling a case sets.
        expect(cut,).toEqual({
          aborted: true,
          reason: 'StragglerCutError: the round closed its grace window on this call after quorum stood',
        },);
        expect(gather.findings,).toContain('stage-voice-lost (critic hf:moonshotai/Kimi-K3)',);
      },
    },),

    it({
      name: 'still hears a voice that arrives INSIDE the grace, which is what '
        + 'makes this a grace rather than a cut: quorum on a roster of three '
        + 'is two, so cutting there would discard a healthy third voice on '
        + 'nearly every gather',
      fn: async () => {
        /** Unused here; the late model answers on its own. */
        const cut = {
          aborted: false,
          reason: '',
        };

        /** Gather where the third voice is late but well inside the window. */
        const gather = await gatherStageVoices({
          client: hangingClient({
            hangingModelId: SEAT_SYNTHETIC_VISION_WITHHELD,
            cut,
            lateMs: 20,
          },),
          modelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'critic',
          l,
          // FAR LONGER THAN THE 20 ms DELAY, since the gather ends when every
          // voice has answered and so never waits this long; a 2,000 ms window
          // stood here, which a loaded machine could outrun. At `graceMs: 0` the
          // late voice is cut and this case fails, which shows the window is
          // what keeps it.
          graceMs: HANG_STOP_MS,
        },);
        expect(gather.voices,).toHaveLength(3,);
        expect(gather.findings,).toHaveLength(0,);
        expect(cut.aborted,).toBe(false,);
      },
    },),
  ],
},);
