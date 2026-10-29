/**
 Tests for the direct providers' catalogs and address builders: one row each, read off the cards, and the three URLs
 built from a base address a test can override. Nothing reaches a provider.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DEEPSEEK_BASE_URL,
  DEEPSEEK_MODELS,
  deepSeekBalanceUrlFor,
  deepSeekChatUrlFor,
  DIRECT_AUTH_HEADER,
  XIAOMI_BASE_URL,
  XIAOMI_MODELS,
  xiaomiChatUrlFor,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_OPENROUTER_ONLY_CHECKER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 A base address no provider owns, so a case proves the builder uses what it is given.
 */
const TEST_BASE_URL = 'http://127.0.0.1:9/cat';

await describe({
  name: 'direct catalogs and addresses',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'BUILDS the three addresses from the base it is given, and from the providers\' own bases by default',
      fn: async () => {
        expect(deepSeekChatUrlFor({ baseUrl: TEST_BASE_URL, },),).toBe(`${TEST_BASE_URL}/chat/completions`,);
        expect(deepSeekBalanceUrlFor({ baseUrl: TEST_BASE_URL, },),).toBe(`${TEST_BASE_URL}/user/balance`,);
        expect(xiaomiChatUrlFor({ baseUrl: TEST_BASE_URL, },),).toBe(`${TEST_BASE_URL}/chat/completions`,);
        expect(deepSeekChatUrlFor({ baseUrl: DEEPSEEK_BASE_URL, },),)
          .toBe('https://api.deepseek.com/chat/completions',);
        expect(deepSeekBalanceUrlFor({ baseUrl: DEEPSEEK_BASE_URL, },),).toBe('https://api.deepseek.com/user/balance',);
        expect(xiaomiChatUrlFor({ baseUrl: XIAOMI_BASE_URL, },),)
          .toBe('https://api.xiaomimimo.com/v1/chat/completions',);
      },
    },),
    it({
      name: 'CARRIES both keys in the Authorization header',
      fn: async () => {
        expect(DIRECT_AUTH_HEADER,).toBe('Authorization',);
      },
    },),
    it({
      name: 'HOLDS exactly one row per provider: deepseek-flash for deepseek-v4.1-flash and mimo-v2.6-flash for '
        + 'xiaomi/mimo-v2.6-flash, each naming the seat it reaches',
      fn: async () => {
        expect(Object.keys(DEEPSEEK_MODELS,),).toEqual(['deepseek-flash',],);
        expect(Object.keys(XIAOMI_MODELS,),).toEqual(['mimo-v2.6-flash',],);
        expect(DEEPSEEK_MODELS['deepseek-flash'].id,).toBe('deepseek-flash',);
        expect(DEEPSEEK_MODELS['deepseek-flash'].sharedWith,).toBe(SEAT_HYPER_OPENROUTER_UNMEASURED,);
        expect(XIAOMI_MODELS['mimo-v2.6-flash'].id,).toBe('mimo-v2.6-flash',);
        expect(XIAOMI_MODELS['mimo-v2.6-flash'].sharedWith,).toBe(SEAT_OPENROUTER_ONLY_CHECKER,);
      },
    },),
  ],
},);
