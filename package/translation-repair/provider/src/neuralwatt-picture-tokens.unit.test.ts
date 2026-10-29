/**
 Tests the prompt tokens a NeuralWatt request stands for once its pictures are
 counted by their patches: the patch count fitted to the two reported counts
 (8188 and 1156, XingZ6016 and zheermao101), the ceiling for a picture whose
 size cannot be read, and the request total that takes inline pictures out of
 the text bytes.

 The headers are built by hand and are cat-themed invention: headers and
 padding, no pixels.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  NEURALWATT_UNSIZED_PICTURE_TOKENS,
  pictureTokensOfSize,
  pictureTokensOfUrl,
  promptTokensOfRequest,
} from '../dist/final/node/index.mjs';
import {
  lossy,
  uriOf,
} from './picture-headers.test-fixture.ts';

await describe({
  name: 'NeuralWatt picture tokens',
  children: [
    describe({
      name: 'picture tokens',
      children: [
        it({
          name: 'FITS THE TWO REPORTED COUNTS: 2400 by 2616 is 7998 and 576 by 1280 is 966, each 190 under what the '
            + 'provider reported with its text (8188 and 1156)',
          fn: async () => {
            await Promise.resolve();
            expect(pictureTokensOfSize({ width: 2_400, height: 2_616, },),).toBe(7_998,);
            expect(pictureTokensOfSize({ width: 576, height: 1_280, },),).toBe(966,);
            expect(7_998 + 190,).toBe(8_188,);
            expect(966 + 190,).toBe(1_156,);
          },
        },),

        it({
          name: 'COUNTS AT LEAST ONE PATCH ON A SIDE, so a sliver is never free',
          fn: async () => {
            await Promise.resolve();
            expect(pictureTokensOfSize({ width: 3, height: 3, },),).toBe(1,);
          },
        },),

        it({
          name: 'HOLDS AN INLINE PICTURE OF READABLE SIZE AT ITS PATCHES, and anything else at the ceiling: unreadable '
            + 'bytes, a data URI that is not base64, and a remote address',
          fn: async () => {
            await Promise.resolve();
            expect(pictureTokensOfUrl({ url: uriOf({ bytes: lossy({ width: 576, height: 1_280, }), }), },),).toBe(966,);
            expect(pictureTokensOfUrl({ url: uriOf({ bytes: Buffer.alloc(100, 7,), }), },),).toBe(
              NEURALWATT_UNSIZED_PICTURE_TOKENS,
            );
            expect(pictureTokensOfUrl({ url: 'data:text/plain,whiskers', },),).toBe(NEURALWATT_UNSIZED_PICTURE_TOKENS,);
            expect(pictureTokensOfUrl({ url: 'https://example.invalid/cat.webp', },),).toBe(NEURALWATT_UNSIZED_PICTURE_TOKENS,);
          },
        },),
      ],
    },),

    describe({
      name: promptTokensOfRequest.name,
      children: [
        it({
          name: 'COUNTS A TEXT-ONLY REQUEST AS ITS BODY BYTES OVER FOUR, rounded up, as the body-only count did',
          fn: async () => {
            await Promise.resolve();
            expect(promptTokensOfRequest({
              messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
              bodyBytes: 4_001,
            },),).toBe(1_001,);
          },
        },),

        it({
          name: 'TAKES AN INLINE PICTURE OUT OF THE TEXT BYTES AND ADDS ITS PATCHES, one count per picture part',
          fn: async () => {
            await Promise.resolve();
            /**
             Two copies of one picture's URI.
             */
            const url = uriOf({ bytes: lossy({ width: 576, height: 1_280, }), });
            expect(promptTokensOfRequest({
              messages: [{
                role: 'user',
                content: [
                  { type: 'text', text: 'Read these.', },
                  { type: 'image_url', image_url: { url, }, },
                  { type: 'image_url', image_url: { url, }, },
                ],
              },],
              bodyBytes: 400 + (2 * url.length),
            },),).toBe(100 + (2 * 966),);
          },
        },),
      ],
    },),
  ],
},);
