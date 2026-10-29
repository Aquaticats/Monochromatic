/**
 Tests that a seat the router refuses for want of a wet provider gives its
 place in the round to the next pending seat at once, rather than leaving the
 round to wait on its slowest remaining voice or to fall through to a retry
 round, and that a windowed stage never re-asks a refused seat.

 TianqiChen66620 (2026-09-27): 269 of 421 rounds lost a seat and closed with
 no grace window, 2,852 s of the 4,575 s its rounds took, while the seat
 reader had seated two judges no provider served; the ledger's P3 and X7.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  gatherStageVoices,
  rotatedBench,
  runWindowedRounds,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  isPurrReply,
  PURR_FORMAT,
  untilAborted,
} from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Straggler window after quorum, short so the test's wall clock stays small.
 */
const GRACE_MS = 20;

/**
 Logger for the rounds under test.
 */
const l = tagged({ tag: 'stage-round-refill-test', },);

/**
 Prompt every seat is asked, which also fixes the rotation.
 */
const MESSAGES = [{ role: 'user', content: 'which cushion is warmest', },] as const;

/**
 Six seats, whose roles are assigned by their place in the prompt's rotation.
 */
const SIX_SEATS: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_VISION,
  SEAT_HYPER_TEXT_BEDROCK,
];

/**
 How a scripted seat behaves when asked.
 */
type SeatRole = 'refused' | 'slow' | 'stuck' | 'fast' | 'failing';

/**
 How long a stuck seat holds its call before failing in transport: long
 against the grace, short against the test's wall clock, so a round that
 waited for it ends in a failed assertion within seconds rather than a hang.
 */
const STUCK_MS = 1_000;

/**
 Client scripted per seat: refused by the router, answering only when the
 round cuts it, answering at once, or failing in transport on a wet
 provider; records every call.

 @param roles - behaviour per seat

 @param calls - call count per seat, written as calls arrive

 @returns Client honouring that script

 @example
 ```ts
 const client = scriptedClient({ roles, calls, },);
 ```
 */
function scriptedClient(
  {
    roles,
    calls,
  }: {
    readonly roles: ReadonlyMap<RosterModelId, SeatRole>;
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
      /**
       What this seat was scripted to do.
       */
      const role = roles.get(request.modelId,);
      if (role === 'refused') {
        throw new NoProviderForModelError({
          modelId: request.modelId,
          reason: 'every provider serving this model is out of budget',
        },);
      }
      if (role === 'failing')
        throw new Error('scripted transport failure',);
      // THE SLOW SEAT NEVER ANSWERS ON ITS OWN: it waits until the round cuts
      // it, and this scripted client arms no exchange deadline, so a round that
      // waited for it rather than closing on the others would never end, and
      // no stall can bring its answer inside the grace window. A 1,500 ms
      // sleep stood here, which a stall of that length between asking it and
      // quorum would have let in.
      if (role === 'slow') {
        await untilAborted({ signal: request.signal, },);
        throw request.signal.reason;
      }
      // THE STUCK SEAT answers nothing either, but gives up in transport after
      // STUCK_MS if no round cut it first, so the old rule's wait on it ends.
      // A plain timer, cleared on the cut: `AbortSignal.timeout` arms an
      // unreferenced one, and with nothing else pending the process exited
      // with the test's top-level await unsettled rather than failing it.
      if (role === 'stuck') {
        /** Fires when the seat gives up. */
        const givingUp = new AbortController();
        /** The seat's own clock. */
        const timer = setTimeout(function giveUp() {
          givingUp.abort();
        }, STUCK_MS,);
        await untilAborted({
          signal: AbortSignal.any([
            request.signal,
            givingUp.signal,
          ],),
        },);
        clearTimeout(timer,);
        if (request.signal.aborted)
          throw request.signal.reason;
        throw new Error('scripted stuck seat gave up',);
      }

      /**
       Scripted payload for the answering call.
       */
      const scripted: unknown = { purr: request.modelId, };
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
 The six seats in the prompt's rotation, with the first refused by the
 router, the second slow, and the rest answering at once. The round-0
 window is the first four (quorum three of six, plus one spare), so the
 fifth is the seat a refusal should hand its place to.

 @returns Rotated seats and the role of each
 */
function refusedFirstBench(): {
  readonly rotated: readonly RosterModelId[];
  readonly roles: ReadonlyMap<RosterModelId, SeatRole>;
} {
  /**
   Seats in the order the gather will ask them.
   */
  const rotated = rotatedBench({
    modelIds: SIX_SEATS,
    messages: MESSAGES,
  },);
  /**
   Behaviour per rotated position.
   */
  const roles = new Map<RosterModelId, SeatRole>(rotated.map(function roleAt(
    modelId,
    index,
  ): readonly [
    RosterModelId,
    SeatRole,
  ] {
    if (index === 0)
      return [modelId, 'refused',];
    if (index === 1)
      return [modelId, 'slow',];
    return [modelId, 'fast',];
  },),);
  return {
    rotated,
    roles,
  };
}

/**
 The six seats in the prompt's rotation, with the first three refused by the
 router, the fourth stuck and the last two answering at once. The round-0
 window is the first four (quorum three of six, plus one spare); the first
 two refusals hand their places to the last two seats and the third has no
 seat left to hand on to, so three seats are out of reach and the three that
 could answer are exactly the bench quorum.

 @returns Rotated seats and the role of each
 */
function quorumOfLiveSeatsBench(): {
  readonly rotated: readonly RosterModelId[];
  readonly roles: ReadonlyMap<RosterModelId, SeatRole>;
} {
  /**
   Seats in the order the gather will ask them.
   */
  const rotated = rotatedBench({
    modelIds: SIX_SEATS,
    messages: MESSAGES,
  },);
  /**
   Behaviour per rotated position.
   */
  const roles = new Map<RosterModelId, SeatRole>(rotated.map(function roleAt(
    modelId,
    index,
  ): readonly [
    RosterModelId,
    SeatRole,
  ] {
    if (index < 3)
      return [modelId, 'refused',];
    if (index === 3)
      return [modelId, 'stuck',];
    return [modelId, 'fast',];
  },),);
  return {
    rotated,
    roles,
  };
}

/**
 Seat at one rotated position, which the fixture always has.

 @param rotated - seats in rotation order

 @param index - position wanted

 @returns Seat at that position
 */
function seatAt(
  {
    rotated,
    index,
  }: {
    readonly rotated: readonly RosterModelId[];
    readonly index: number;
  },
): RosterModelId {
  /**
   Seat at the position, if the fixture is as sized.
   */
  const seat = rotated[index];
  if (seat === undefined)
    throw new Error(`fixture has no seat at ${String(index,)}`,);
  return seat;
}

await describe({
  name: 'refused seat refill',
  children: [
    it({
      name: 'THE GATHER HANDS A REFUSED SEAT\'S PLACE TO THE NEXT PENDING SEAT in the same round, so quorum '
        + 'closes on three prompt voices and the slow one is left to the grace window',
      fn: async () => {
        /** Call count per seat. */
        const calls: Record<string, number> = {};
        /** Rotation and roles. */
        const {
          rotated,
          roles,
        } = refusedFirstBench();
        /** Gather under test. */
        const gather = await gatherStageVoices({
          client: scriptedClient({
            roles,
            calls,
          },),
          modelIds: SIX_SEATS,
          messages: MESSAGES,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: PURR_FORMAT,
          validate: isPurrReply,
          stage: 'critic',
          l,
          graceMs: GRACE_MS,
        },);
        /** Seats heard. */
        const heard = gather.voices.map(function seatOf({ modelId, },): RosterModelId {
          return modelId;
        },);
        expect(gather.quorumMet,).toBe(true,);
        expect(heard,).toContain(seatAt({
          rotated,
          index: 4,
        },),);
        expect(heard,).not.toContain(seatAt({
          rotated,
          index: 1,
        },),);
        expect([...gather.unreachable,],).toEqual([seatAt({
          rotated,
          index: 0,
        },),],);
        expect(calls[seatAt({
          rotated,
          index: 5,
        },)],).toBeUndefined();
      },
    },),

    it({
      name: 'A WINDOWED STAGE HANDS A REFUSED SEAT\'S PLACE ON the same way',
      fn: async () => {
        /** Call count per seat. */
        const calls: Record<string, number> = {};
        /** Rotation and roles. */
        const {
          rotated,
          roles,
        } = refusedFirstBench();
        /** Outcomes under test. */
        const { outcomes, } = await runWindowedRounds({
          client: scriptedClient({
            roles,
            calls,
          },),
          modelIds: SIX_SEATS,
          messages: MESSAGES,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: PURR_FORMAT,
          validate: isPurrReply,
          stage: 'lane-contest',
          l,
          graceMs: GRACE_MS,
        },);
        /** Seats heard. */
        const heard = outcomes
          .filter(function wasHeard({ voice, },): boolean {
            return voice.heard;
          },)
          .map(function seatOf({ modelId, },): RosterModelId {
            return modelId;
          },);
        expect(heard,).toContain(seatAt({
          rotated,
          index: 4,
        },),);
        expect(heard,).not.toContain(seatAt({
          rotated,
          index: 1,
        },),);
      },
    },),

    it({
      name: 'A WINDOWED STAGE NEVER RE-ASKS A SEAT THE ROUTER REFUSED, while a seat lost in transport is '
        + 'asked again',
      fn: async () => {
        /** Call count per seat. */
        const calls: Record<string, number> = {};
        /**
         Four seats: one refused, two failing, one answering. TWO FAILING
         SINCE 2026-10-22: the refusal leaves three of the six-seat bench in
         reach, exactly its quorum, so the bench closes on two voices ("Let
         the grace open"), and two answering seats would close it in the
         first round with no seat lost in transport left to re-ask.
         */
        const roles = new Map<RosterModelId, SeatRole>([
          [SEAT_HYPER_OPENROUTER_VISION_EDITOR, 'refused',],
          [SEAT_SYNTHETIC_VISION_NO_OPENROUTER, 'failing',],
          [SEAT_SYNTHETIC_VISION_WITHHELD, 'failing',],
          [SEAT_SYNTHETIC_TEXT_EVERYWHERE, 'fast',],
        ],);
        await runWindowedRounds({
          client: scriptedClient({
            roles,
            calls,
          },),
          modelIds: [...roles.keys(),],
          messages: MESSAGES,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: PURR_FORMAT,
          validate: isPurrReply,
          stage: 'consolidate-gate',
          l,
          quorumOver: 6,
          graceMs: GRACE_MS,
        },);
        expect(calls[SEAT_HYPER_OPENROUTER_VISION_EDITOR],).toBe(1,);
        expect(calls[SEAT_SYNTHETIC_VISION_NO_OPENROUTER],).toBeGreaterThan(1,);
      },
    },),

    it({
      name: 'THE GATHER LETS THE GRACE OPEN IN THE FIRST ROUND where the refusals leave exactly the bench quorum '
        + 'within reach (owner, 2026-10-22, "Let the grace open"): it closes on two of the three live seats, cuts the '
        + 'stuck one at the grace and never asks it again',
      fn: async () => {
        /** Call count per seat. */
        const calls: Record<string, number> = {};
        /** Rotation and roles. */
        const {
          rotated,
          roles,
        } = quorumOfLiveSeatsBench();
        /** Gather under test. */
        const gather = await gatherStageVoices({
          client: scriptedClient({
            roles,
            calls,
          },),
          modelIds: SIX_SEATS,
          messages: MESSAGES,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: PURR_FORMAT,
          validate: isPurrReply,
          stage: 'critic',
          l,
          graceMs: GRACE_MS,
        },);
        expect(gather.quorumMet,).toBe(true,);
        expect(gather.quorum,).toBe(2,);
        expect(gather.voices.length,).toBe(2,);
        expect(gather.unreachable.size,).toBe(3,);
        expect(calls[seatAt({
          rotated,
          index: 3,
        },)],).toBe(1,);
        expect(gather.findings,).toContain('stage-short-bench (critic reachable 3 of 6, quorum 2)',);
      },
    },),

    it({
      name: 'A WINDOWED STAGE LETS THE GRACE OPEN IN THE FIRST ROUND the same way, closing on two of the three '
        + 'live seats without asking the stuck one again',
      fn: async () => {
        /** Call count per seat. */
        const calls: Record<string, number> = {};
        /** Rotation and roles. */
        const {
          rotated,
          roles,
        } = quorumOfLiveSeatsBench();
        /** Rounds under test. */
        const rounds = await runWindowedRounds({
          client: scriptedClient({
            roles,
            calls,
          },),
          modelIds: SIX_SEATS,
          messages: MESSAGES,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: PURR_FORMAT,
          validate: isPurrReply,
          stage: 'lane-contest',
          l,
          graceMs: GRACE_MS,
        },);
        /** Seats heard. */
        const heard = rounds.outcomes.filter(function wasHeard({ voice, },): boolean {
          return voice.heard;
        },);
        expect(heard.length,).toBe(2,);
        expect(rounds.quorum.needed,).toBe(2,);
        expect(rounds.unreachable,).toBe(3,);
        expect(calls[seatAt({
          rotated,
          index: 3,
        },)],).toBe(1,);
      },
    },),

    it({
      name: 'A WHOLE-BENCH ROUND LETS THE GRACE OPEN the same way, sizing its need again as the refusals land',
      fn: async () => {
        /** Call count per seat. */
        const calls: Record<string, number> = {};
        /** Roles: three refused, one slow, two answering. */
        const { roles, } = quorumOfLiveSeatsBench();
        /** Rounds under test. */
        const rounds = await runWindowedRounds({
          client: scriptedClient({
            roles,
            calls,
          },),
          modelIds: SIX_SEATS,
          messages: MESSAGES,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: PURR_FORMAT,
          validate: isPurrReply,
          stage: 'lane-contest',
          l,
          graceMs: GRACE_MS,
          fanOut: 'whole-bench',
        },);
        /** Seats heard. */
        const heard = rounds.outcomes.filter(function wasHeard({ voice, },): boolean {
          return voice.heard;
        },);
        expect(heard.length,).toBe(2,);
        expect(rounds.quorum.needed,).toBe(2,);
        /** The stuck seat's outcome, which the grace cut before it gave up. */
        const stuck = rounds.outcomes.find(function isStuck({ modelId, },): boolean {
          return roles.get(modelId,) === 'stuck';
        },);
        expect(stuck?.voice.heard,).toBe(false,);
        expect(calls[stuck?.modelId ?? ''],).toBe(1,);
      },
    },),
  ],
},);
