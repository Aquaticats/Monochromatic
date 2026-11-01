/**
 Tests for the reader of the mode a re-ask of a reasoning runaway runs in (ledger B1141).

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  measuresModelOf,
  type ReaskSeat,
} from '../dist/final/node/index.mjs';

//region Re-ask mode tests

/**
 Seat of a question put to a bench of two models.
 */
const SEAT: ReaskSeat = { role: 'roster', reask: 'other-model', bench: ['glm-5.3', 'minimax-m3',], };

await describe({
  name: measuresModelOf.name,
  concurrency: 1,
  children: [
    it({
      name: 'READS A SEAT THAT NAMES THE MEASURED-MODEL MODE AS A MEASUREMENT',
      fn: async () => {
        expect(measuresModelOf({ seat: { ...SEAT, reask: 'measured-model', }, },),).toBe(true,);
      },
    },),

    it({
      name: 'READS THE OTHER-MODEL MODE AND A SEAT THAT NAMES NONE AS THE RUN\'S, which asks another model',
      fn: async () => {
        expect(measuresModelOf({ seat: { ...SEAT, reask: 'other-model', }, },),).toBe(false,);
        expect(measuresModelOf({ seat: SEAT, },),).toBe(false,);
      },
    },),
  ],
},);

//endregion Re-ask mode tests
