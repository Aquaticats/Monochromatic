/**
 Tests for what a stage call carries of a reasoning runaway (ledger B1074): the seat's role and bench go out on the
 request, and an answer that is a length stop with no content comes back as a runaway that names the model an earlier
 re-ask already went to.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
  ReaskSeat,
} from '@monochromatic-dev/translation-repair-provider';
import { attemptStageCall, } from '../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_TEXT_BEDROCK,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  isMeowReply,
  MEOW_FORMAT,
} from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

//region Stage call runaway tests

/**
 Client that records the request it was handed and answers with the outcome it was given.

 @param seen - requests, filled as they arrive

 @param outcome - what every ask comes back as

 @returns Scripted client

 @example
 ```ts
 const client = answeringWith({ seen, outcome, },);
 ```
 */
function answeringWith(
  {
    seen,
    outcome,
  }: {
    readonly seen: ChatJsonRequest<unknown>[];
    readonly outcome: ChatJsonOutcome<unknown>;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      seen.push(request as ChatJsonRequest<unknown>,);
      return outcome as ChatJsonOutcome<ValueT>;
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

/**
 Asks the seat that ran away once.

 @param client - scripted client

 @param reaskSeat - role and bench the call carries, when it carries any

 @returns Voice as data

 @example
 ```ts
 const voice = await askRunawaySeat({ client, },);
 ```
 */
async function askRunawaySeat(
  {
    client,
    reaskSeat,
  }: {
    readonly client: ModelClient;
    readonly reaskSeat?: ReaskSeat;
  },
) {
  return await attemptStageCall({
    client,
    modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
    messages: [{ role: 'user', content: 'Does the cat meow?', },],
    signal: new AbortController().signal,
    exchangeTimeoutMs: HANG_STOP_MS,
    responseFormat: MEOW_FORMAT,
    validate: isMeowReply,
    stage: 'meow-check',
    l: levelCapturingLogger({ lines: [], },),
    ...((reaskSeat === undefined) ? {} : { reaskSeat, }),
  },);
}

/**
 A reply that ended in a length stop with no content.
 */
const RUNAWAY: ChatJsonOutcome<unknown> = {
  kind: 'schema-mismatch',
  rawText: '',
  reason: 'truncated-completion',
  detail: 'provider reported a truncating completion',
};

await describe({
  name: `${attemptStageCall.name} (a reasoning runaway)`,
  concurrency: 1,
  children: [
    it({
      name: 'READS A LENGTH STOP WITH NO CONTENT AS AN ANSWER THAT RAN AWAY, unreadable for the recovery round to ask '
        + 'another model, and a cut that holds content as one cut short',
      fn: async () => {
        const ranAway = await askRunawaySeat({ client: answeringWith({ seen: [], outcome: RUNAWAY, },), },);
        expect(ranAway,).toEqual({ heard: false, answered: true, unreachable: false, unreadable: 'runaway', },);

        const cutShort = await askRunawaySeat({
          client: answeringWith({ seen: [], outcome: { ...RUNAWAY, rawText: '{"meow":', }, },),
        },);
        expect(cutShort,).toEqual({ heard: false, answered: true, unreachable: false, unreadable: 'cut-short', },);
      },
    },),

    it({
      name: 'NAMES THE MODEL AN EARLIER RE-ASK WENT TO on the voice, so the recovery round asks the next model and does '
        + 'not replay the first model\'s claimed reply',
      fn: async () => {
        const voice = await askRunawaySeat({
          client: answeringWith({
            seen: [],
            outcome: {
              ...RUNAWAY,
              reaskFailure: {
                servedBy: 'hyper',
                kind: 'schema-mismatch',
                rawText: 'purr',
                cause: `unparseable; asked of ${SEAT_HYPER_TEXT_BEDROCK}`,
                askedModelId: SEAT_HYPER_TEXT_BEDROCK,
              },
            },
          },),
        },);
        expect(voice,).toEqual({
          heard: false,
          answered: true,
          unreachable: false,
          unreadable: 'runaway',
          askedInstead: SEAT_HYPER_TEXT_BEDROCK,
        },);
      },
    },),

    it({
      name: 'SENDS THE SEAT\'S ROLE AND BENCH ON THE REQUEST when the caller names them, and sends no member when it '
        + 'does not',
      fn: async () => {
        /**
         Requests the client saw.
         */
        const seen: ChatJsonRequest<unknown>[] = [];
        /**
         Role and bench the stage names.
         */
        const reaskSeat: ReaskSeat = { role: 'translator', bench: [SEAT_HYPER_OPENROUTER_UNMEASURED,], };
        await askRunawaySeat({ client: answeringWith({ seen, outcome: RUNAWAY, },), reaskSeat, },);
        await askRunawaySeat({ client: answeringWith({ seen, outcome: RUNAWAY, },), },);
        expect(seen[0]?.reaskSeat,).toEqual(reaskSeat,);
        expect('reaskSeat' in (seen[1] ?? {}),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: `${attemptStageCall.name} (an answer of another model)`,
  children: [
    it({
      name: 'CARRIES THE MODEL THAT ANSWERED on a heard voice when the re-ask of a runaway went to another model, and '
        + 'none when the seat\'s own model answered',
      fn: async () => {
        const other = await askRunawaySeat({
          client: answeringWith({
            seen: [],
            outcome: {
              kind: 'ok',
              value: { meow: true, },
              rawText: '{"meow":true}',
              answeredBy: SEAT_HYPER_TEXT_BEDROCK,
            },
          },),
        },);
        expect(other,).toEqual({ heard: true, value: { meow: true, }, answeredBy: SEAT_HYPER_TEXT_BEDROCK, },);

        const own = await askRunawaySeat({
          client: answeringWith({
            seen: [],
            outcome: { kind: 'ok', value: { meow: true, }, rawText: '{"meow":true}', },
          },),
        },);
        expect(own,).toEqual({ heard: true, value: { meow: true, }, },);
      },
    },),
  ],
},);

//endregion Stage call runaway tests
