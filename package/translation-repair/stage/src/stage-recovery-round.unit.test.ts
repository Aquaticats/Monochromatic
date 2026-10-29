/**
 Tests for the recovery round: each seat whose answer arrived unreadable is
 re-asked once, under the nudge naming what happened to it, and the round's
 two log lines count what it asked and what it heard.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

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
import {
  CUT_SHORT_RECOVERY_NUDGE,
  OFF_SHAPE_RECOVERY_NUDGE,
  runRecoveryRound,
  type UnreadableCause,
} from '../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  isMeowReply,
  MEOW_FORMAT,
  type MeowReply,
} from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Text the stage's own prompt carries.
 */
const STAGE_PROMPT = 'meow';

/**
 What the scripted client saw of one seat's asks.
 */
type SeatAsks = {
  /**
   Asks the seat received.
   */
  readonly count: number;

  /**
   Contents of the latest ask's messages, in order.
   */
  readonly contents: readonly string[];
};

/**
 Client that answers every seat readably except the one named to stay
 unreadable, and records what each seat was asked.

 @param asks - record of each seat's asks, filled as they arrive

 @param unreadableSeat - seat whose every answer is off the shape, or none

 @returns Client honouring that script

 @example
 ```ts
 const client = scriptedClient({ asks, unreadableSeat: 'none', },);
 ```
 */
function scriptedClient(
  {
    asks,
    unreadableSeat,
  }: {
    readonly asks: Map<string, SeatAsks>;
    readonly unreadableSeat: RosterModelId | 'none';
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      asks.set(
        request.modelId,
        {
          count: (asks.get(request.modelId,)?.count ?? 0) + 1,
          contents: request.messages.map(function contentOf(message,): string {
            /**
             What the message says, which every scripted ask writes as text.
             */
            const { content, } = message;
            if ((typeof content) !== 'string')
              throw new Error('scripted asks carry text content only',);
            return content;
          },),
        },
      );
      if (request.modelId === unreadableSeat) {
        return {
          kind: 'schema-mismatch',
          rawText: 'purr',
          reason: 'unparseable-json',
          detail: 'scripted unparseable answer',
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
 Runs one recovery round over scripted seats and reports what it asked,
 returned and logged.

 @param unreadable - seats to re-ask, in roster order

 @param causeOf - why each seat's answer could not be read

 @param unreadableSeat - seat whose re-ask stays unreadable, or none

 @returns Round's outcomes, each seat's asks, and the recovery lines it
 logged

 @example
 ```ts
 const run = await recoverOver({ unreadable, causeOf, unreadableSeat: 'none', },);
 ```
 */
async function recoverOver(
  {
    unreadable,
    causeOf,
    unreadableSeat,
  }: {
    readonly unreadable: readonly RosterModelId[];
    readonly causeOf: ReadonlyMap<RosterModelId, UnreadableCause>;
    readonly unreadableSeat: RosterModelId | 'none';
  },
): Promise<{
  readonly outcomes: readonly {
    readonly modelId: RosterModelId;
    readonly heard: boolean;
  }[];
  readonly asks: ReadonlyMap<string, SeatAsks>;
  readonly roundLines: readonly string[];
}> {
  /**
   Each seat's asks, as the client saw them.
   */
  const asks = new Map<string, SeatAsks>();

  /**
   Every line the round logged, behind its level.
   */
  const lines: string[] = [];

  /**
   Outcomes of the round under test.
   */
  const recovered = await runRecoveryRound<MeowReply>({
    roundRequest: {
      client: scriptedClient({
        asks,
        unreadableSeat,
      },),
      messages: [{ role: 'user', content: STAGE_PROMPT, },],
      signal: new AbortController().signal,
      exchangeTimeoutMs: HANG_STOP_MS,
      responseFormat: MEOW_FORMAT,
      validate: isMeowReply,
      stage: 'gate',
      l: levelCapturingLogger({ lines, },),
      graceMs: HANG_STOP_MS,
    },
    unreadable,
    causeOf,
  },);
  return {
    outcomes: recovered.map(function summarized(outcome,): {
      readonly modelId: RosterModelId;
      readonly heard: boolean;
    } {
      return {
        modelId: outcome.modelId,
        heard: outcome.voice
          .heard,
      };
    },),
    asks,
    roundLines: lines.filter(function isRoundLine(line,): boolean {
      return line.includes(': recovery round ',);
    },),
  };
}

await describe({
  name: runRecoveryRound.name,
  children: [
    it({
      name: 'RE-ASKS each seat once under the nudge naming its cause, after the stage\'s own prompt, and returns '
        + 'every re-ask grouped by cause in UNREADABLE_CAUSES order rather than in roster order',
      fn: async () => {
        /**
         Seat whose answer the length limit cut.
         */
        const cutShort = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

        /**
         Seat whose answer arrived off the shape.
         */
        const offShape = SEAT_SYNTHETIC_VISION_WITHHELD;

        /**
         The round over both, the off-shape seat first in roster order.
         */
        const run = await recoverOver({
          unreadable: [offShape, cutShort,],
          causeOf: new Map<RosterModelId, UnreadableCause>([
            [offShape, 'off-shape',],
            [cutShort, 'cut-short',],
          ],),
          unreadableSeat: 'none',
        },);
        expect({
          outcomes: run.outcomes,
          cutShortAsks: run.asks.get(cutShort,),
          offShapeAsks: run.asks.get(offShape,),
        },).toEqual({
          outcomes: [
            {
              modelId: cutShort,
              heard: true,
            },
            {
              modelId: offShape,
              heard: true,
            },
          ],
          cutShortAsks: {
            count: 1,
            contents: [STAGE_PROMPT, CUT_SHORT_RECOVERY_NUDGE.content,],
          },
          offShapeAsks: {
            count: 1,
            contents: [STAGE_PROMPT, OFF_SHAPE_RECOVERY_NUDGE.content,],
          },
        },);
      },
    },),

    it({
      name: 'ASKS only the seats it was handed, and names no group for a cause none of them has',
      fn: async () => {
        /**
         The one seat re-asked.
         */
        const offShape = SEAT_SYNTHETIC_VISION_WITHHELD;

        /**
         The round over that seat alone.
         */
        const run = await recoverOver({
          unreadable: [offShape,],
          causeOf: new Map<RosterModelId, UnreadableCause>([[offShape, 'off-shape',],],),
          unreadableSeat: 'none',
        },);
        expect({
          asked: [...run.asks.keys(),],
          roundLines: run.roundLines,
        },).toEqual({
          asked: [offShape,],
          roundLines: [
            'warn [runRecoveryRound] gate: recovery round for 1 unreadable answer (1 off-shape)',
            'info [runRecoveryRound] gate: recovery round heard 1 of 1 re-asked voice',
          ],
        },);
      },
    },),

    it({
      name: 'COUNTS what it asked and what came back readable, in the plural past one seat (ledger B109), '
        + 'and returns a re-ask that stayed unreadable as unheard',
      fn: async () => {
        /**
         Seat whose re-ask comes back readable.
         */
        const recovering = SEAT_HYPER_OPENROUTER_VISION_EDITOR;

        /**
         Seat whose re-ask is off the shape again.
         */
        const stubborn = SEAT_SYNTHETIC_VISION_WITHHELD;

        /**
         The round over both.
         */
        const run = await recoverOver({
          unreadable: [recovering, stubborn,],
          causeOf: new Map<RosterModelId, UnreadableCause>([
            [recovering, 'cut-short',],
            [stubborn, 'off-shape',],
          ],),
          unreadableSeat: stubborn,
        },);
        expect({
          outcomes: run.outcomes,
          roundLines: run.roundLines,
        },).toEqual({
          outcomes: [
            {
              modelId: recovering,
              heard: true,
            },
            {
              modelId: stubborn,
              heard: false,
            },
          ],
          roundLines: [
            'warn [runRecoveryRound] gate: recovery round for 2 unreadable answers (1 cut-short, 1 off-shape)',
            'info [runRecoveryRound] gate: recovery round heard 1 of 2 re-asked voices',
          ],
        },);
      },
    },),
  ],
},);
