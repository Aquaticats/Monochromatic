/**
 Tests the loop of `reader-measure`: pictures are screened by the
 deterministic reader, matched to a trusted record, and only matched ones are
 put to the candidate, up to the number asked for.

 THE CANDIDATE IS ASKED THROUGH THE PRODUCTION READER STAGE
 (`readPastRefusal`), so a refusal is re-asked as it is in a pass, and a
 candidate that produces nothing is a row that says so rather than a gap.

 Fixtures are cat-themed invention; the client and the deterministic reader
 are scripts. No provider is reached.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { ModelCaller, } from '@monochromatic-dev/translation-repair-provider';
import { SEAT_HYPER_TEXT_BEDROCK, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { measureReader, } from '../../dist/final/node/index.mjs';

/**
 A transcription of a notice about a lost cat.
 */
const NOTICE = 'Lost tabby cat called Mittens, last seen by the old well. Please call 555 0134.';

/**
 What the deterministic reader says about each picture, by name.
 */
const OCR: Readonly<Record<string, string>> = {
  'notice.webp': NOTICE,
  'stranger.webp': 'Zyx wvu qj kk zz yy xx.',
};

/**
 The one trusted record.
 */
const RECORDS = [[
  {
    modelId: 'seated-one',
    text: NOTICE,
  },
  {
    modelId: 'seated-two',
    text: NOTICE,
  },
],];

/**
 Client answering every ask with one reply, counting the asks.

 @param reply - text every ask gets

 @returns Client and its ask count

 @example
 ```ts
 const { client, asks, } = answering({ reply: NOTICE, },);
 ```
 */
function answering({ reply, }: { readonly reply: string; },): { readonly client: ModelCaller; readonly asks: string[]; } {
  /**
   Asset names asked, in order.
   */
  const asks: string[] = [];
  return {
    asks,
    client: {
      chatText: async (request,) => {
        asks.push(request.modelId,);
        await Promise.resolve();
        return { text: reply, };
      },
      chatJson: async () => {
        throw new Error('chatJson unused',);
      },
    },
  };
}

/**
 Pictures named, each a few stand-in bytes.

 @param names - asset names

 @returns Pictures

 @example
 ```ts
 const pictures = picturesNamed({ names: ['notice.webp',], },);
 ```
 */
function picturesNamed({ names, }: { readonly names: readonly string[]; },) {
  return names.map(function picture(assetName,) {
    return {
      assetName,
      bytes: new Uint8Array(8,),
    };
  },);
}

/**
 Deterministic reader that reads the OCR table named OCR and finds no text elsewhere.

 @param picture - picture asked about

 @returns Its reading

 @example
 ```ts
 const reading = await scriptedOcr({ assetName: 'notice.webp', },);
 ```
 */
async function scriptedOcr(
  { assetName, }: { readonly assetName: string; },
) {
  await Promise.resolve();
  /**
   Text the script holds for this picture.
   */
  const text = OCR[assetName];
  return (text === undefined)
    ? {
      kind: 'no-text' as const,
      characters: 0,
    }
    : {
      kind: 'read' as const,
      text,
    };
}

await describe({
  name: measureReader.name,
  children: [
    it({
      name: 'PUTS ONLY THE PICTURES THAT MATCH A TRUSTED RECORD TO THE CANDIDATE: a textless picture and one '
        + 'matching nothing cost no call',
      fn: async () => {
        const { client, asks, } = answering({ reply: NOTICE, },);
        const rows = await measureReader({
          client,
          readOcr: scriptedOcr,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
          records: RECORDS,
          pictures: picturesNamed({ names: ['blank.webp', 'stranger.webp', 'notice.webp',], },),
          limit: 9,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'reader-measure-test', },),
        },);
        expect(asks.length,).toBe(1,);
        expect(rows.length,).toBe(1,);
        expect(rows[0]?.assetName,).toBe('notice.webp',);
        expect(rows[0]?.candidate,).toEqual({ kind: 'read', text: NOTICE, },);
      },
    },),

    it({
      name: 'STOPS AT THE NUMBER OF MATCHED PICTURES ASKED FOR',
      fn: async () => {
        const { client, asks, } = answering({ reply: NOTICE, },);
        const rows = await measureReader({
          client,
          readOcr: scriptedOcr,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
          records: RECORDS,
          pictures: picturesNamed({ names: ['notice.webp', 'notice.webp', 'notice.webp',], },),
          limit: 2,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'reader-measure-test', },),
        },);
        expect(rows.length,).toBe(2,);
        expect(asks.length,).toBe(2,);
      },
    },),

    it({
      name: 'REPORTS A CANDIDATE THAT PRODUCED NOTHING AS A ROW THAT SAYS SO, with the reason',
      fn: async () => {
        const { client, } = answering({ reply: '', },);
        const rows = await measureReader({
          client,
          readOcr: scriptedOcr,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
          records: RECORDS,
          pictures: picturesNamed({ names: ['notice.webp',], },),
          limit: 9,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'reader-measure-test', },),
        },);
        expect(rows[0]?.candidate,).toEqual({ kind: 'nothing', reason: 'empty-reply', },);
      },
    },),
  ],
},);
