/**
 Tests for two readers of one picture whose replies were both cut by the completion limit (ledger B1076, B1080).

 ONE MODEL GIVES AT MOST ONE VOICE PER QUESTION, and the plain question to a model is the digest of that model's own
 first ask, so two cut readers re-asked of one model would record one claimed reading as two. The readers of one
 picture share one record of the models asked, each takes a model from it before it asks, and a reader that finds none
 left asks itself again on another provider.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  type ChatTextReply,
  type ChatTextRequest,
  createReaskClaims,
  type ModelClient,
  type ReaskMode,
} from '@monochromatic-dev/translation-repair-provider/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { readPastRefusal, } from '../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

//region Cut readers sharing a record tests

/**
 The two readers of the picture, both cut on their first ask.
 */
const READERS: readonly RosterModelId[] = [SEAT_SYNTHETIC_VISION_WITHHELD, SEAT_HYPER_VISION,];

/**
 Orders two asks, each a model and a hint, by the model and then the hint.

 @param left - first ask

 @param right - second ask

 @returns Negative where the first sorts first

 @example
 ```ts
 const sorted = asks.toSorted(byAsk);
 ```
 */
function byAsk(
  left: readonly string[],
  right: readonly string[],
): number {
  /**
   Both asks as one text each, compared by code unit.
   */
  const [first, second,] = [left.join('\u0000',), right.join('\u0000',),];
  if (first === second)
    return 0;
  return (first < second) ? -1 : 1;
}

/**
 Reply the completion cap ended while the model was still reasoning.
 */
const CUT: ChatTextReply = { text: '', finishReason: 'length', servedBy: 'neuralwatt', };

/**
 What a reader that reads returns.
 */
const READING = '走失猫咪 Mittens，虎斑，请电 555 0134。';

/**
 Reads the picture with both readers at once through one record, the way a picture's readers are asked.

 @param bench - every reader of the picture, left out of the models a cut reader is re-asked of

 @param reask - whose answer the re-ask may give, the pass's by default

 @returns Asks the client saw, as the model and the provider hint of each, in the order made

 @example
 ```ts
 const asks = await readWithBothReaders({ bench: READERS, },);
 ```
 */
async function readWithBothReaders(
  {
    bench,
    reask = 'other-model',
  }: {
    readonly bench: readonly RosterModelId[];
    readonly reask?: ReaskMode;
  },
): Promise<readonly (readonly [RosterModelId, string])[]> {
  /**
   Asks the client saw.
   */
  const asks: (readonly [RosterModelId, string])[] = [];

  /**
   Client whose readers are cut on their first ask and whose every other ask reads.
   */
  const client: ModelClient = {
    chatText: async (request,) => {
      /**
       Request as this package's own contract describes it.
       */
      const asked = request as ChatTextRequest;

      /**
       Asks this model had before this one.
       */
      const before = asks.filter(function sameModel(ask,): boolean {
        return ask[0] === asked.modelId;
      },).length;
      asks.push([asked.modelId, asked.otherThan?.join(' and ',) ?? 'no hint',],);
      await Promise.resolve();
      return (READERS.includes(asked.modelId,) && (before === 0)) ? CUT : { text: READING, servedBy: 'hyper', };
    },
    chatJson: async () => {
      throw new Error('chatJson unused by the reading stage',);
    },
    quotas: async () => {
      throw new Error('quotas unused by the reading stage',);
    },
  };

  /**
   Record the readers of the picture share.
   */
  const claims = createReaskClaims();
  await Promise.all(READERS.map(async function read(modelId,) {
    return await readPastRefusal({
      client,
      modelId,
      bytes: new Uint8Array(64,),
      assetName: 'noticeboard.webp',
      benchModelIds: bench,
      claims,
      reask,
      signal: AbortSignal.timeout(HANG_STOP_MS,),
      perCallTimeoutMs: HANG_STOP_MS,
      l: tagged({ tag: 'cut-claims-test', },),
    },);
  },),);
  return asks;
}

await describe({
  name: `${readPastRefusal.name} (readers cut together)`,
  concurrency: 1,
  children: [
    it({
      name: 'RE-ASKS TWO CUT READERS OF TWO DIFFERENT MODELS: the second takes the next reader of the order that the '
        + 'first did not take',
      fn: async () => {
        const asks = await readWithBothReaders({ bench: READERS, },);
        expect(asks.filter(function isRetake(ask,): boolean {
          return !READERS.includes(ask[0],);
        },).map(function modelOf(ask,) {
          return ask[0];
        },),).toEqual([SEAT_HYPER_TEXT_BEDROCK, SEAT_BEDROCK_ONLY_VISION_UNSEATED,],);
      },
    },),

    it({
      name: 'RE-ASKS EACH CUT READER OF ITSELF WHERE THE CALLER IS A MEASUREMENT, bench and record notwithstanding: a '
        + 'rival\'s reading scored as the reader\'s would measure the rival',
      fn: async () => {
        const asks = await readWithBothReaders({ bench: READERS, reask: 'measured-model', },);
        expect(asks.filter(function isRetake(ask,): boolean {
          return !READERS.includes(ask[0],);
        },),).toEqual([],);
        expect(asks.toSorted(byAsk,),).toEqual([
          [SEAT_HYPER_VISION, 'neuralwatt',],
          [SEAT_HYPER_VISION, 'no hint',],
          [SEAT_SYNTHETIC_VISION_WITHHELD, 'neuralwatt',],
          [SEAT_SYNTHETIC_VISION_WITHHELD, 'no hint',],
        ].toSorted(byAsk,),);
      },
    },),

    it({
      name: 'ASKS THE SECOND CUT READER AGAIN ON ANOTHER PROVIDER when the first took the one model that was left: one '
        + 'model is never asked for two readers of a picture',
      fn: async () => {
        const asks = await readWithBothReaders({
          bench: [
            ...READERS,
            SEAT_BEDROCK_ONLY_VISION_UNSEATED,
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          ],
        },);
        expect(asks.filter(function isRetake(ask,): boolean {
          return !READERS.includes(ask[0],);
        },).map(function modelOf(ask,) {
          return ask[0];
        },),).toEqual([SEAT_HYPER_TEXT_BEDROCK,],);
        expect(asks.filter(function isSecondReader(ask,): boolean {
          return ask[0] === SEAT_HYPER_VISION;
        },),).toEqual([[SEAT_HYPER_VISION, 'no hint',], [SEAT_HYPER_VISION, 'neuralwatt',],],);
      },
    },),
  ],
},);

//endregion Cut readers sharing a record tests
