/**
 Tests for the Claude catalog and its address builder: eight rows read off the cards, each naming its wire model, and
 the Messages URL built from a base address a test can override. Nothing reaches a provider.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { ANTHROPIC_SERVED_IDS, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  CLAUDE_API_VERSION,
  CLAUDE_AUTH_HEADER,
  CLAUDE_BASE_URL,
  CLAUDE_MODELS,
  CLAUDE_VERSION_HEADER,
  claudeMessagesUrlFor,
} from '../dist/final/node/index.mjs';

/**
 A base address no provider owns, so a case proves the builder uses what it is given.
 */
const TEST_BASE_URL = 'http://127.0.0.1:9/cat';

await describe({
  name: 'claude catalog and address',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'BUILDS the Messages address from the base it is given, and from Anthropic\'s own base by default',
      fn: async () => {
        expect(claudeMessagesUrlFor({ baseUrl: TEST_BASE_URL, },),).toBe(`${TEST_BASE_URL}/messages`,);
        expect(claudeMessagesUrlFor({ baseUrl: CLAUDE_BASE_URL, },),).toBe('https://api.anthropic.com/v1/messages',);
      },
    },),
    it({
      name: 'NAMES the key header x-api-key and pins the API version header to 2023-06-01',
      fn: async () => {
        expect(CLAUDE_AUTH_HEADER,).toBe('x-api-key',);
        expect(CLAUDE_VERSION_HEADER,).toBe('anthropic-version',);
        expect(CLAUDE_API_VERSION,).toBe('2023-06-01',);
      },
    },),
    it({
      name: 'HOLDS one row per Claude card, each reaching the roster seat of its own spelling and a wire model '
        + 'from the two the key is granted for',
      fn: async () => {
        expect(Object.keys(CLAUDE_MODELS,),).toEqual([...ANTHROPIC_SERVED_IDS,],);
        for (const id of ANTHROPIC_SERVED_IDS) {
          expect(CLAUDE_MODELS[id].id,).toBe(id,);
          expect(CLAUDE_MODELS[id].sharedWith,).toBe(id,);
          expect(['claude-sonnet-5-5', 'claude-haiku-5-5',],).toContain(CLAUDE_MODELS[id].wireModel,);
        }
      },
    },),
  ],
},);
