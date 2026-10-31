/**
 Tests for the seat a heard voice answered for (ledger B1142).

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { seatOfVoice, } from '../dist/final/node/index.mjs';

//region Stage gather tests

await describe({
  name: seatOfVoice.name,
  concurrency: 1,
  children: [
    it({
      name: 'READS THE MODEL ITSELF AS THE SEAT where no other model answered for it',
      fn: async () => {
        expect(seatOfVoice({ voice: { modelId: 'glm-5.3', }, },),).toBe('glm-5.3',);
      },
    },),

    it({
      name: 'READS THE SEAT A VOICE ANSWERED FOR where another model gave the reply',
      fn: async () => {
        expect(seatOfVoice({ voice: { modelId: 'minimax-m3', seat: 'glm-5.3', }, },),).toBe('glm-5.3',);
      },
    },),
  ],
},);

//endregion Stage gather tests
