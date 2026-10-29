/**
 Tests that the energy a NeuralWatt call holds while in flight counts a
 picture as a picture and not as base64 text.

 WHY: XingZ6016 (2026-10-21) read `image0.webp` (1271156 bytes, 2400 by 2616)
 on `glm-5.3-flash`. The body's bytes stood for 423985 prompt tokens, the top
 bucket, which holds 0.9089 Wh; the provider reported 8188 prompt tokens and
 the call measured 0.2311 Wh. The same seat read `photo1.webp` (576 by 1280)
 and `photo2.webp` (576 by 1280) for a reported 1156 prompt tokens each
 against 26957 to 32466 estimated. NeuralWatt documents no image accounting
 (its chat completions page, models page and OpenAPI file, fetched 2026-10-21,
 say only that `image_url` parts are read by models with `vision`), so the
 count here is fitted to what the provider reported: the pictures cost
 round(width / 28) times round(height / 28) tokens, and the text around them
 190 (7998 plus 190 is 8188; 966 plus 190 is 1156).

 WHAT THESE PIN: a picture request's prompt tokens are its text bytes over
 four plus the picture's patch count; a picture whose size cannot be read is
 held at the fallback ceiling rather than at its base64 length; and the
 margin of one is untouched.

 Fixtures are cat-themed invention: the pictures are a lossy WebP header and
 padding, no pixels.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  estimateNeuralWattWh,
  NEURALWATT_ENERGY_MARGIN,
} from '../dist/final/node/index.mjs';
import { jsonRecordOf, } from './neuralwatt-ledger.test-fixture.ts';
import {
  chatExchangesOf,
  FREE_COST_COMMENT,
  scriptedClient,
  SIGNAL,
  streamOf,
} from './neuralwatt-scripted-client.test-fixture.ts';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { readFile, } from 'node:fs/promises';

/**
 Instruction text sent beside the picture.
 */
const INSTRUCTION = 'Transcribe every word visible in this image.';

/**
 A lossy WebP file of the stated size: the RIFF and VP8 headers a decoder
 reads the size from, then padding standing in for the coded pixels.

 @param width - pixels across

 @param height - pixels down

 @param padding - bytes of stand-in pixel data

 @returns File's bytes

 @example
 ```ts
 const bytes = lossyWebp({ width: 576, height: 1280, padding: 1_000, },);
 ```
 */
function lossyWebp(
  {
    width,
    height,
    padding,
  }: {
    readonly width: number;
    readonly height: number;
    readonly padding: number;
  },
): Buffer {
  /**
   The VP8 frame: tag, start code, size, then the stand-in data.
   */
  const frame = Buffer.alloc(10 + padding,);
  frame[3] = 0x9D;
  frame[4] = 0x01;
  frame[5] = 0x2A;
  frame.writeUInt16LE(width, 6,);
  frame.writeUInt16LE(height, 8,);
  /**
   The RIFF container around it.
   */
  const riff = Buffer.alloc(20,);
  riff.write('RIFF', 0,);
  riff.writeUInt32LE(12 + frame.length, 4,);
  riff.write('WEBP', 8,);
  riff.write('VP8 ', 12,);
  riff.writeUInt32LE(frame.length, 16,);
  return Buffer.concat([riff, frame,],);
}

/**
 The data URI a picture travels as.

 @param bytes - file's bytes

 @returns URI as the reading stage builds it

 @example
 ```ts
 const uri = dataUriOf({ bytes, },);
 ```
 */
function dataUriOf({ bytes, }: { readonly bytes: Buffer; },): string {
  return `data:image/webp;base64,${bytes.toString('base64',)}`;
}

/**
 Sends one picture to the GLM seat with no energy comment in the reply, so the
 estimate stands in the ledger, and reads back what was held.

 @param uri - picture's data URI

 @returns Prompt tokens the ledger line holds, the energy held, and the bytes
 of text around the picture

 @example
 ```ts
 const held = await heldForPicture({ uri, },);
 ```
 */
async function heldForPicture(
  { uri, }: { readonly uri: string; },
): Promise<{ readonly promptTokens: number; readonly wh: number; readonly textBytes: number; }> {
  await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-picture-', },);
  const { client, ledger, exchanges, } = scriptedClient({
    dir: scratch.path,
    chat: [{ status: 200, bodyText: streamOf({ comments: FREE_COST_COMMENT, }), },],
  },);
  await client.chatText({
    modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    messages: [{
      role: 'user',
      content: [
        {
          type: 'text',
          text: INSTRUCTION,
        },
        {
          type: 'image_url',
          image_url: { url: uri, },
        },
      ],
    },],
    signal: SIGNAL,
  },);
  /**
   Line the call wrote.
   */
  const line = jsonRecordOf({ text: (await readFile(ledger.path, 'utf8',)).trim(), },);
  return {
    promptTokens: line.promptTokens as number,
    wh: line.wh as number,
    textBytes: Buffer.byteLength(chatExchangesOf({ exchanges, },)[0]?.bodyJson ?? '',) - uri.length,
  };
}

await describe({
  name: 'NeuralWatt picture energy estimate',
  concurrency: 1,
  children: [
    it({
      name: 'COUNTS A 2400 BY 2616 PICTURE AS 7998 PATCH TOKENS PLUS ITS TEXT, not as base64 text (XingZ6016: '
        + '8188 prompt tokens reported, 423985 estimated)',
      fn: async () => {
        const held = await heldForPicture({
          uri: dataUriOf({ bytes: lossyWebp({ width: 2_400, height: 2_616, padding: 200_000, }), },),
        },);
        expect(held.promptTokens,).toBe(Math.ceil(held.textBytes / 4,) + 7_998,);
        expect(held.wh,).toBeCloseTo(
          estimateNeuralWattWh({ servedId: 'glm-5.3-flash', promptTokens: held.promptTokens, },).bucketMeanWh
            * NEURALWATT_ENERGY_MARGIN,
          12,
        );
        // The 4,096 to 16,384 bucket, 0.0714 Wh for this model (the measured mean of the settled text calls of
        // `XingZ6020`, see the estimate's module note), not the top bucket's matrix figure of 0.9089. The 8188 of the
        // title is the count XingZ6016's SPEND line reported, the figure the fit rests on; `XingZ6020` read the same
        // picture too, reported 8188 on its SPEND line and held it for 8264 on its energy line, both in this bucket.
        expect(held.wh,).toBeCloseTo(0.0714, 12,);
      },
    },),

    it({
      name: 'COUNTS A 576 BY 1280 PICTURE AS 966 PATCH TOKENS PLUS ITS TEXT (zheermao101: 1156 reported, '
        + '26957 to 32466 estimated)',
      fn: async () => {
        const held = await heldForPicture({
          uri: dataUriOf({ bytes: lossyWebp({ width: 576, height: 1_280, padding: 90_000, }), },),
        },);
        expect(held.promptTokens,).toBe(Math.ceil(held.textBytes / 4,) + 966,);
      },
    },),

    it({
      name: 'HOLDS A PICTURE WHOSE SIZE CANNOT BE READ AT THE 8192-TOKEN FALLBACK, which is above the largest '
        + 'reported count, rather than at its base64 length',
      fn: async () => {
        const held = await heldForPicture({
          uri: `data:image/webp;base64,${Buffer.alloc(300_000, 7,).toString('base64',)}`,
        },);
        expect(held.promptTokens,).toBe(Math.ceil(held.textBytes / 4,) + 8_192,);
      },
    },),
  ],
},);
