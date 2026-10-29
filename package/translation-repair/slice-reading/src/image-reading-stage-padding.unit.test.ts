/**
 Guards the padded reading of Xu_Yushu1 (2026-09-26): a reader answered a
 handwritten picture with 27 solid characters followed by about 2,700
 ideographic spaces. The sense rule judged the trimmed text, but the stage
 logged "2753 characters" and handed the padded text on, so the log counted
 padding as a transcript and every later comparison carried it. A usable
 reading leaves the stage trimmed, the same as a short one.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider';
import { readImageAsset, } from '../dist/final/node/index.mjs';
import { SEAT_SYNTHETIC_VISION_WITHHELD, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Logger the stage writes its progress to.
 */
const l = tagged({ tag: 'image-reading-stage-padding-test', },);

/**
 Transcription a reader returns for the picture under test.
 */
const A_READING = '虎斑猫 Mittens，2019 年领养，联系方式 @mittenspaw。';

/**
 How many ideographic spaces pad the reply, near the real reply's count.
 */
const PADDING_WIDTH = 2_700;

/**
 Reply carrying the reading, a line break, and a run of ideographic spaces.
 */
const PADDED_REPLY = `${A_READING}\n${'\u3000'.repeat(PADDING_WIDTH,)}`;

/**
 Client answering every reading with the padded reply.
 */
const client: ModelClient = {
  chatText: async () => ({ text: PADDED_REPLY, }),
  chatJson: async () => {
    throw new Error('chatJson unused by the reading stage',);
  },
  quotas: async () => {
    throw new Error('quotas unused by the reading stage',);
  },
};

await describe({
  name: 'a usable reading padded with whitespace (Xu_Yushu1)',
  children: [
    it({
      name: 'RETURNS THE READING TRIMMED, so the padding never reaches the corroboration or the log',
      fn: async () => {
        /**
         One reading of the padded reply.
         */
        const reading = await readImageAsset({
          client,
          modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
          bytes: new Uint8Array(64,).fill(7,),
          assetName: 'mittens.webp',
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
        },);

        expect(reading.kind,).toBe('read',);
        if (reading.kind !== 'read')
          throw new Error('read by construction',);
        expect(reading.text,).toBe(A_READING,);
      },
    },),
  ],
},);
