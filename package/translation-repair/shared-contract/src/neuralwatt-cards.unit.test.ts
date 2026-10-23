/**
 Tests the NeuralWatt side of the model cards: the two models the owner's key
 covers with free daily energy, spelled the way `GET /v1/models` spells them,
 added to cards that already exist, with every hold those cards carried
 before the provider joined still in place.

 Facts: `https://api.neuralwatt.com/v1/models` (public, fetched 2026-10-20)
 and `https://docs.neuralwatt.com/billing/daily-free-energy`.

 Fixtures are the roster itself; no corpus content appears.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  CARD_PROVIDERS,
  cardsServing,
  MODEL_CARDS,
  NEURALWATT_SERVED_IDS,
  ROSTER_MODEL_IDS,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'neuralwatt cards',
  children: [
    it({
      name: 'SERVES THE TWO STANDARD IDS THE FREE ENERGY COVERS and no other lane: not a flex id, billed at '
        + '0.65 and held under load, not a speed id, which the free energy does not cover, and not the alias',
      fn: async () => {
        expect([...NEURALWATT_SERVED_IDS,],).toEqual(['deepseek-v4.1-flash', 'glm-5.3-flash',],);
        for (const id of NEURALWATT_SERVED_IDS) {
          expect(id.endsWith('-flex',),).toBe(false,);
          expect(id.endsWith('-speed',),).toBe(false,);
          expect(id,).not.toBe('nw-flash',);
        }
      },
    },),

    it({
      name: 'ADDS NO ROSTER IDENTITY: the two cards are the existing DeepSeek V4.1 Flash and GLM-5.3-Flash, '
        + 'so no seat is created and the roster list is the one it was',
      fn: async () => {
        // The eight Claude cards of 2026-10-22 joined after NeuralWatt did and are not NeuralWatt's to add.
        expect(ROSTER_MODEL_IDS.filter(function notClaude(id,): boolean {
          return !id.startsWith('claude-',);
        },).length,).toBe(13,);
        expect(cardsServing({ provider: 'neuralwatt', },).map(function toId(card,): string {
          return card.id;
        },),).toEqual(['hf:zai-org/GLM-5.3-Flash', 'deepseek-v4.1-flash',],);
        expect(MODEL_CARDS['hf:zai-org/GLM-5.3-Flash'].neuralwatt?.id,).toBe('glm-5.3-flash',);
        expect(MODEL_CARDS['deepseek-v4.1-flash'].neuralwatt?.id,).toBe('deepseek-v4.1-flash',);
        expect(CARD_PROVIDERS.includes('neuralwatt',),).toBe(true,);
      },
    },),

    it({
      name: 'KEEPS EVERY HOLD THE CARDS CARRIED, none lifted and none added: the owner unseated GLM-5.3-Flash as a '
        + 'judge on 2026-09-02 and that stays, and DeepSeek V4.1 Flash still has no transcription measured',
      fn: async () => {
        expect(MODEL_CARDS['hf:zai-org/GLM-5.3-Flash'].holds,).toEqual([
          'wide-seat-dropped',
          'late-judge-dropped',
          'synthetic-withheld',
        ],);
        expect(MODEL_CARDS['deepseek-v4.1-flash'].holds,).toEqual(['reader-unmeasured',],);
      },
    },),

    it({
      name: 'RECORDS WHAT THE LISTING STATES: a million-token context for both, DeepSeek\'s 393,216-token output '
        + 'ceiling, the context window standing in for GLM\'s unlisted one, and image input reported for both',
      fn: async () => {
        for (const card of cardsServing({ provider: 'neuralwatt', },)) {
          expect(card.neuralwatt.contextLength,).toBe(1_048_560,);
          expect(card.neuralwatt.readsImages,).toBe(true,);
        }
        expect(MODEL_CARDS['deepseek-v4.1-flash'].neuralwatt?.maxOutputLength,).toBe(393_216,);
        expect(MODEL_CARDS['hf:zai-org/GLM-5.3-Flash'].neuralwatt?.maxOutputLength,).toBe(1_048_560,);
      },
    },),
  ],
},);
