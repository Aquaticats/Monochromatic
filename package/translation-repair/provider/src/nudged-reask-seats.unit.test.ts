/**
 Tests for the re-ask of a reasoning runaway when the request names no seat, and when several seats of one question
 run away together (ledger B1079, B1080).

 A RE-ASK NEEDS THE ROLE OF ITS SEAT ("Honour holds (Recommended)", 2026-10-30), so a request that names no seat is not
 asked of another model on a guess. SEATS OF ONE QUESTION SHARE ONE RECORD OF THE MODELS ASKED, so two of them never
 take one model (one model gives at most one voice per question).

 FIXTURES ARE CAT-THEMED.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  alternateModelsFor,
  type ChatJsonOutcome,
  type ChatJsonRequest,
  type ChatTextReply,
  type ChatTextRequest,
  createReaskClaims,
  reaskElsewhereNudged,
  type ReaskClaims,
} from '../dist/final/node/index.mjs';
import {
  infoLinesDuring,
  warnLinesDuring,
} from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_TEXT_BEDROCK,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region Nudged reask seat tests

/**
 What the cat questions answer with.
 */
type Cat = { readonly spot: string; };

/**
 Request asking the model that ran away, with no seat named.
 */
const UNSEATED_REQUEST: ChatJsonRequest<Cat> = {
  modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
  messages: [{ role: 'user', content: 'the sheet', },],
  signal: new AbortController().signal,
  responseFormat: {
    type: 'json_schema',
    json_schema: { name: 'cat', schema: { type: 'object', }, },
  },
  validate: function isSpotted(value: unknown,): value is Cat {
    return ((typeof value) === 'object') && (value !== null) && Object.hasOwn(
      value,
      'spot',
    );
  },
};

/**
 Reply of a reasoning runaway: the length stop with no content.
 */
const RUNAWAY_REPLY: ChatTextReply = { text: '', finishReason: 'length', servedBy: 'neuralwatt', };

/**
 What reading that reply produced.
 */
const RUNAWAY: ChatJsonOutcome<Cat> = {
  kind: 'schema-mismatch',
  rawText: '',
  reason: 'truncated-completion',
  detail: 'provider reported a truncating completion (model stopped with finish_reason=length)',
};

/**
 The nudge a re-ask of the same model adds, in its words.
 */
const NUDGE_CONTENT = 'Your previous reply could not be used: it was not the JSON object the question asks for. '
  + 'Answer the same question again, replying with ONLY that JSON object, nothing before or after it.';

/**
 The two seats of the question that run away together.
 */
const FIRST_SEAT: RosterModelId = SEAT_HYPER_OPENROUTER_UNMEASURED;

/**
 The second seat of the question.
 */
const SECOND_SEAT: RosterModelId = SEAT_HYPER_TEXT_BEDROCK;

/**
 Asks both seats' runaways at once through one record, the way the seats of a gather are asked.

 @param bench - models seated on the question

 @param claims - record the seats share

 @returns Models asked, in the order the asks were made, with the nudge each carried, and the lines logged

 @example
 ```ts
 const { asked, } = await runBothSeats({ bench: [FIRST_SEAT, SECOND_SEAT,], claims: createReaskClaims(), },);
 ```
 */
async function runBothSeats(
  {
    bench,
    claims,
  }: {
    readonly bench: readonly RosterModelId[];
    readonly claims: ReaskClaims;
  },
) {
  /**
   Asks the re-ask received.
   */
  const asks: ChatTextRequest[] = [];

  /**
   Answers every ask usably.
   */
  async function answer(nudged: ChatTextRequest,): Promise<ChatTextReply> {
    asks.push(nudged,);
    return { text: '{"spot":"radiator"}', servedBy: 'hyper', };
  }
  const { logged, } = await infoLinesDuring({
    run: async () =>
      Promise.all([FIRST_SEAT, SECOND_SEAT,].map(async function runaway(modelId,) {
        return await reaskElsewhereNudged({
          request: { ...UNSEATED_REQUEST, modelId, reaskSeat: { role: 'roster', bench, claims, }, },
          first: { reply: RUNAWAY_REPLY, outcome: RUNAWAY, },
          ask: answer,
        },);
      },),),
  },);
  return { asks, logged, };
}

await describe({
  name: `${reaskElsewhereNudged.name} (seats)`,
  concurrency: 1,
  children: [
    it({
      name: 'ASKS NO OTHER MODEL WHERE THE REQUEST NAMES NO SEAT: with no role to read the holds of, the re-ask is the '
        + 'runaway\'s own, nudged, on another provider, and the warning says that no seat was named',
      fn: async () => {
        /**
         Calls the re-ask received.
         */
        const asks: ChatTextRequest[] = [];
        const { result, warned, } = await warnLinesDuring({
          run: async () =>
            reaskElsewhereNudged({
              request: UNSEATED_REQUEST,
              first: { reply: RUNAWAY_REPLY, outcome: RUNAWAY, },
              ask: async (nudged,) => {
                asks.push(nudged,);
                return { text: '{"spot":"radiator"}', servedBy: 'hyper', };
              },
            },),
        },);
        expect(result.kind,).toBe('ok',);
        expect(asks.map(function askOf(request,) {
          return [request.modelId, request.otherThan,];
        },),).toEqual([[SEAT_HYPER_OPENROUTER_UNMEASURED, 'neuralwatt',],],);
        expect(asks[0]?.messages.at(-1,)?.content,).toBe(NUDGE_CONTENT,);
        expect(warned,).toEqual([
          `[translation-repair] [reaskElsewhereNudged] [askOtherModels] ${SEAT_HYPER_OPENROUTER_UNMEASURED}: the request `
            + 'names no seat, so no role says which model may take the re-ask of a reasoning runaway; re-asking '
            + `${SEAT_HYPER_OPENROUTER_UNMEASURED} on another provider, as before`,
        ],);
      },
    },),

    it({
      name: 'ASKS TWO DIFFERENT MODELS FOR TWO SEATS OF ONE QUESTION THAT RAN AWAY TOGETHER: the second takes the next '
        + 'model of its order that the first did not take, so one claimed reply is never recorded as two voices',
      fn: async () => {
        /**
         Models seated on the question.
         */
        const bench = [FIRST_SEAT, SECOND_SEAT,];
        const { asks, } = await runBothSeats({ bench, claims: createReaskClaims(), },);

        /**
         Model the first seat takes.
         */
        const [firstTakes,] = alternateModelsFor({ modelId: FIRST_SEAT, role: 'roster', bench, },).alternates;
        if (firstTakes === undefined)
          throw new Error('the first seat has a model left to take',);

        /**
         Model the second seat takes once the first has taken its own.
         */
        const [secondTakes,] = alternateModelsFor({
          modelId: SECOND_SEAT,
          role: 'roster',
          bench,
          askedForOtherSeats: [firstTakes,],
        },).alternates;
        expect(asks.map(function modelOf(request,) {
          return request.modelId;
        },),).toEqual([firstTakes, secondTakes,],);
        expect(firstTakes === secondTakes,).toBe(false,);
        for (const plain of asks) {
          expect(plain.messages,).toEqual([{ role: 'user', content: 'the sheet', },],);
          expect(plain.otherThan,).toBeUndefined();
        }
      },
    },),

    it({
      name: 'NAMES THE MODEL THE FIRST SEAT TOOK IN THE LOG OF THE SECOND, left out because it was already asked for '
        + 'another seat of the question',
      fn: async () => {
        /**
         Models seated on the question.
         */
        const bench = [FIRST_SEAT, SECOND_SEAT,];
        const { asks, logged, } = await runBothSeats({ bench, claims: createReaskClaims(), },);
        expect(logged.some(function namesIt(line,): boolean {
          return line.includes(`${asks[0]?.modelId ?? ''} (already asked for another seat of this question)`,);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'FALLS BACK TO ITS OWN MODEL WITH THE WARNING FOR THE SECOND SEAT when the first took the one model that was '
        + 'left: the second is the only model left to ask, nudged, on another provider',
      fn: async () => {
        /**
         Models the first seat could be re-asked of, with the bench holding the two seats.
         */
        const { alternates, } = alternateModelsFor({
          modelId: FIRST_SEAT,
          role: 'roster',
          bench: [FIRST_SEAT, SECOND_SEAT,],
        },);
        const [onlyOne, ...others] = alternates;

        /**
         Asks the re-ask received.
         */
        const asks: ChatTextRequest[] = [];
        const claims = createReaskClaims();
        const { warned, } = await warnLinesDuring({
          run: async () =>
            Promise.all([FIRST_SEAT, SECOND_SEAT,].map(async function runaway(modelId,) {
              return await reaskElsewhereNudged({
                request: {
                  ...UNSEATED_REQUEST,
                  modelId,
                  reaskSeat: { role: 'roster', bench: [FIRST_SEAT, SECOND_SEAT, ...others,], claims, },
                },
                first: { reply: RUNAWAY_REPLY, outcome: RUNAWAY, },
                ask: async (nudged,) => {
                  asks.push(nudged,);
                  return { text: '{"spot":"radiator"}', servedBy: 'hyper', };
                },
              },);
            },),),
        },);
        expect(asks.map(function askOf(request,) {
          return [request.modelId, request.otherThan,];
        },),).toEqual([[onlyOne, undefined,], [SECOND_SEAT, 'neuralwatt',],],);
        expect(asks[1]?.messages.at(-1,)?.content,).toBe(NUDGE_CONTENT,);
        expect(warned,).toEqual([
          `[translation-repair] [reaskElsewhereNudged] [askOtherModels] ${SECOND_SEAT}: no other model could take the `
            + `re-ask of a reasoning runaway (0 tried); re-asking ${SECOND_SEAT} on another provider, as before`,
        ],);
      },
    },),
  ],
},);

//endregion Nudged reask seat tests
