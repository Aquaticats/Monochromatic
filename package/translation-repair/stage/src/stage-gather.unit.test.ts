/**
 Tests for the seat a heard voice answered for (ledger B1142).

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  seatOfVoice,
  seatOrderOf,
} from '../dist/final/node/index.mjs';

//region Stage gather tests

await describe({
  name: 'stage gather',
  concurrency: 1,
  children: [
    describe({
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
    },),

    describe({
      name: seatOrderOf.name,
      concurrency: 1,
      children: [
        it({
          name: 'ORDERS VOICES BY THE SEAT THEY ANSWERED FOR (ledger B1200): a stand-in for the last seat sorts last, '
            + 'where its own position, which is on no bench, would put it first',
          fn: async () => {
            /**
             Bench in roster order.
             */
            const modelIds = ['glm-5.3', 'minimax-m3', 'inception/mercury-2.5',] as const;
            /**
             Voices out of order, the last seat answered by a model on no bench.
             */
            const voices = [
              { modelId: 'hf:moonshotai/Kimi-K3', seat: 'inception/mercury-2.5', value: 'late', },
              { modelId: 'minimax-m3', value: 'middle', },
              { modelId: 'glm-5.3', value: 'early', },
            ] as const;
            expect(voices.toSorted(seatOrderOf({ modelIds, },),).map(function valueOf({ value, },): string {
              return value;
            },),).toEqual(['early', 'middle', 'late',],);
          },
        },),
        it({
          name: 'THROWS FOR A VOICE WHOSE SEAT IS ON NO BENCH (B1235) instead of placing it before every seat, since a '
            + 'position of -1 sorted a voice nobody seated first with no sign that anything was wrong',
          fn: async () => {
            /**
             Bench in roster order.
             */
            const modelIds = ['glm-5.3', 'minimax-m3',] as const;
            /**
             Voices, the second answered by a model on no bench and standing for no seat of it.
             */
            const voices = [
              { modelId: 'glm-5.3', value: 'early', },
              { modelId: 'hf:moonshotai/Kimi-K3', value: 'stranger', },
            ] as const;
            expect(() => voices.toSorted(seatOrderOf({ modelIds, },),),).toThrow('on no seat of the bench',);
          },
        },),
      ],
    },),
  ],
},);

//endregion Stage gather tests
