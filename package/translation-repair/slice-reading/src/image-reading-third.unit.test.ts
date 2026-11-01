/**
 Guards class fifty-two (XingZ601, 2026-09-17): a picture read by three
 readers is corroborated when ANY two of them agree, not only when the first
 two do. The corroboration compared only the first two readings, a rule
 written when the vision sub-roster was exactly two models; on a bench of
 five, a score screenshot read by three models stopped the entry at overlap
 0.297 between the two it compared while the third reading agreed with both
 of them (0.586 and 0.326). Cat-themed invention throughout; no corpus
 content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChatTextRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { readImagePair, } from '../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Logger the stage writes its progress to.
 */
const l = tagged({ tag: 'image-reading-third-test', },);

/**
 Third seat on the vision bench.
 */
const THIRD_READER = 'gemma-4-26b-a4b-it' as RosterModelId;

/**
 Vision sub-roster of three.
 */
const READERS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  THIRD_READER,
];

/**
 What one reader transcribed from a picture of a noticeboard.
 */
const READING = '走失猫咪 Mittens，虎斑，2019 年出生，联系 @mittenspaw，请电 555 0134。';

/**
 What another reader transcribed from the same picture, worded differently
 where a transcription can differ and identical where it cannot.
 */
const AGREEING_READING = '走失猫咪 Mittens，虎斑，2019 年出生，联系 @mittenspaw，电话 555 0134。';

/**
 What a reader transcribed from some other picture entirely.
 */
const OTHER_PICTURE = '兽医诊所营业时间：周一至周五上午九点到下午六点，周六休息。';

/**
 Deterministic reader that finds text, which is what lets the models be asked.

 @returns Reading with enough text to ask the models

 @example
 ```ts
 const reading = await found();
 ```
 */
async function found(): Promise<{
  readonly kind: 'read';
  readonly text: string;
}> {
  return {
    kind: 'read',
    text: '走失猫咪 Mittens',
  };
}

/**
 Client answering each model with whatever that model is scripted to say.

 @param byModel - reply per model

 @returns Client the pair reader can be driven with

 @example
 ```ts
 const client = scriptedClient({ byModel: { [THIRD_READER]: READING, }, },);
 ```
 */
function scriptedClient(
  { byModel, }: { readonly byModel: Readonly<Record<string, string>>; },
): ModelClient {
  return {
    chatText: async (request,) => {
      /**
       Request as this package's own contract describes it.
       */
      const { modelId, } = request as ChatTextRequest;
      return { text: byModel[modelId] ?? '', };
    },
    chatJson: async () => {
      throw new Error('chatJson unused by the reading stage',);
    },
    quotas: async () => {
      throw new Error('quotas unused by the reading stage',);
    },
  };
}

await describe({
  name: 'a picture three readers read, two of them agreeing (class fifty-two)',
  children: [
    it({
      name: 'CORROBORATES on the second and third readers when the first read another picture, '
        + 'and forwards the two readings that vouch for each other',
      fn: async () => {
        const paired = await readImagePair({
          client: scriptedClient({
            byModel: {
              [SEAT_SYNTHETIC_VISION_WITHHELD]: OTHER_PICTURE,
              [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: READING,
              [THIRD_READER]: AGREEING_READING,
            },
          },),
          readOcr: found,
          readerModelIds: READERS,
          bytes: new Uint8Array(64,).fill(7,),
          assetName: 'noticeboard.webp',
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
        },);
        expect(paired.kind,).toBe('corroborated',);
        if (paired.kind !== 'corroborated')
          throw new Error('corroborated by construction',);
        expect(paired.readings
          .map(function toModel(reading,): RosterModelId {
            return reading.modelId;
          },),).toEqual([
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            THIRD_READER,
          ],);
      },
    },),
    it({
      name: 'STILL REFUSES three readings of three different pictures, reporting the closest pair',
      fn: async () => {
        const paired = await readImagePair({
          client: scriptedClient({
            byModel: {
              [SEAT_SYNTHETIC_VISION_WITHHELD]: OTHER_PICTURE,
              [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: READING,
              [THIRD_READER]: '今天的天气很好，猫在花园里晒太阳，邻居家的狗在叫。',
            },
          },),
          readOcr: found,
          readerModelIds: READERS,
          bytes: new Uint8Array(64,).fill(7,),
          assetName: 'noticeboard.webp',
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
        },);
        expect(paired.kind,).toBe('unavailable',);
        if (paired.kind !== 'unavailable')
          throw new Error('unavailable by construction',);
        expect(paired.reason,).toBe('readers-disagree',);
        expect(paired.readings?.length,).toBe(3,);
      },
    },),
  ],
},);
