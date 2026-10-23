/**
 Tests for the one-wire-model-per-bench rule (C6 of the review of the Claude route). The eight Claude cards share
 two wire models, so two cards of one model on one bench are one model heard twice: they would count as two voices
 toward a quorum and overstate agreement. The rule lived in comments only (`claude-card-sides.ts`, `roster-id.ts`);
 these cases hold the code that enforces it.

 Fixtures are the roster itself; no corpus content appears.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  assertOneWireModelPerBench,
  ROSTER_MODEL_IDS,
  wireModelOf,
  WireModelRepeatedError,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'bench wire model',
  children: [
    describe({
      name: wireModelOf.name,
      children: [
        it({
          name: 'NAMES THE WIRE MODEL A CLAUDE CARD SENDS, the same for the card of a model and its effort cards, and a '
            + 'roster model of another provider by its own id',
          fn: async () => {
            expect(wireModelOf({ modelId: 'claude-sonnet-5-5', },),).toBe('claude-sonnet-5-5',);
            expect(wireModelOf({ modelId: 'claude-sonnet-5-5-effort-high', },),).toBe('claude-sonnet-5-5',);
            expect(wireModelOf({ modelId: 'claude-haiku-5-5-effort-low', },),).toBe('claude-haiku-5-5',);
            expect(wireModelOf({ modelId: 'glm-5.3', },),).toBe('glm-5.3',);
          },
        },),
      ],
    },),

    describe({
      name: assertOneWireModelPerBench.name,
      children: [
        it({
          name: 'REFUSES TWO CARDS OF ONE WIRE MODEL ON ONE BENCH, naming the bench, the wire model and both cards: Sonnet '
            + 'beside its own effort card, and two effort cards of Haiku',
          fn: async () => {
            for (const modelIds of [
              ['glm-5.3', 'claude-sonnet-5-5', 'claude-sonnet-5-5-effort-high',],
              ['claude-haiku-5-5-effort-low', 'claude-haiku-5-5-effort-medium',],
            ] as const) {
              expect(() => {
                assertOneWireModelPerBench({ bench: 'editors', modelIds, },);
              },).toThrow(WireModelRepeatedError,);
            }
            expect(() => {
              assertOneWireModelPerBench({
                bench: 'editors',
                modelIds: ['claude-sonnet-5-5', 'claude-sonnet-5-5-effort-high',],
              },);
            },).toThrow('The editors bench seats claude-sonnet-5-5 more than once (claude-sonnet-5-5, claude-sonnet-5-5-effort-high)',);
          },
        },),

        it({
          name: 'REFUSES THE SAME ROSTER ID TWICE, which is one voice counted twice by any model',
          fn: async () => {
            expect(() => {
              assertOneWireModelPerBench({ bench: 'checkers', modelIds: ['glm-5.3', 'glm-5.3',], },);
            },).toThrow(WireModelRepeatedError,);
          },
        },),

        it({
          name: 'ACCEPTS ONE CARD OF EACH WIRE MODEL, Sonnet beside Haiku, and an empty bench, and the whole roster less '
            + 'nothing it repeats',
          fn: async () => {
            assertOneWireModelPerBench({ bench: 'editors', modelIds: ['claude-sonnet-5-5', 'claude-haiku-5-5',], },);
            assertOneWireModelPerBench({ bench: 'empty', modelIds: [], },);
            assertOneWireModelPerBench({
              bench: 'roster',
              modelIds: ROSTER_MODEL_IDS.filter(function notEffortCard(modelId,): boolean {
                return !modelId.includes('-effort-',);
              },),
            },);
            expect(true,).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
