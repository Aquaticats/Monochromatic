/**
 Tests the NeuralWatt catalog: the two standard-lane models the owner's free
 daily energy covers, read off the cards, and the three addresses the client
 calls (`https://docs.neuralwatt.com`, `GET https://api.neuralwatt.com/v1/models`,
 fetched 2026-10-20).

 Fixtures are the roster itself.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  cardsServing,
  NEURALWATT_SERVED_IDS,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  NEURALWATT_AUTH_HEADER,
  NEURALWATT_BASE_URL,
  neuralWattChatUrlFor,
  neuralWattIdFor,
  NEURALWATT_MODELS,
  neuralWattQuotaUrlFor,
} from '../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

await describe({
  name: 'neuralwatt catalog',
  children: [
    it({
      name: 'KEYS ONE ROW PER SERVED SPELLING with the card\'s fields, each shared with the roster seat it reaches',
      fn: async () => {
        expect(Object.keys(NEURALWATT_MODELS,).toSorted(),).toEqual([...NEURALWATT_SERVED_IDS,].toSorted(),);
        for (const card of cardsServing({ provider: 'neuralwatt', },)) {
          expect(NEURALWATT_MODELS[card.neuralwatt.id].sharedWith,).toBe(card.id,);
          expect(NEURALWATT_MODELS[card.neuralwatt.id].contextLength,).toBe(card.neuralwatt.contextLength,);
          expect(NEURALWATT_MODELS[card.neuralwatt.id].maxOutputLength,).toBe(card.neuralwatt.maxOutputLength,);
        }
      },
    },),

    it({
      name: 'SPELLS A ROSTER SEAT THE WAY THE LISTING DOES, and says plainly that every other seat is not served',
      fn: async () => {
        expect(neuralWattIdFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, },),).toEqual({
          served: true,
          id: 'deepseek-v4.1-flash',
        },);
        expect(neuralWattIdFor({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, },),).toEqual({
          served: true,
          id: 'glm-5.3-flash',
        },);
        expect(neuralWattIdFor({ modelId: SEAT_BEDROCK_ONLY_TEXT, },),).toEqual({ served: false, },);
      },
    },),

    it({
      name: 'ADDRESSES THE OPENAI-COMPATIBLE BASE AT /v1 WITH A BEARER KEY IN THE AUTHORIZATION HEADER',
      fn: async () => {
        expect(NEURALWATT_BASE_URL,).toBe('https://api.neuralwatt.com/v1',);
        expect(NEURALWATT_AUTH_HEADER,).toBe('Authorization',);
        expect(neuralWattChatUrlFor({ baseUrl: NEURALWATT_BASE_URL, },),).toBe(
          'https://api.neuralwatt.com/v1/chat/completions',
        );
        expect(neuralWattQuotaUrlFor({ baseUrl: NEURALWATT_BASE_URL, },),).toBe('https://api.neuralwatt.com/v1/quota',);
      },
    },),
  ],
},);
