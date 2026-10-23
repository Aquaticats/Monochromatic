/**
 Tests the JSON exchange and the per-model limiter every provider client
 shares (audit area six): the exchange carries every field of the request but
 its validator, and each model gets one limiter of its own.

 Fixtures are cat-themed invention.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { perModelLimiter, } from '@monochromatic-dev/translation-repair-shared-contract';
import { chatJsonThrough, } from '../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Whether a parsed answer names a cat.

 @param value - parsed answer

 @returns True for an object with a string `cat`

 @example
 ```ts
 namesCat({ cat: 'Mittens', },); // true
 ```
 */
function namesCat(value: unknown,): value is { readonly cat: string; } {
  return ((typeof value) === 'object') && (value !== null) && ('cat' in value) && ((typeof value.cat) === 'string');
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: chatJsonThrough.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FORWARDS EVERY FIELD OF THE REQUEST BUT ITS VALIDATOR to the text exchange, the ones no copy '
            + 'named included, and reads the reply against the validator',
          fn: async () => {
            /**
             Requests the text exchange received.
             */
            const seen: Record<string, unknown>[] = [];
            const chatJson = chatJsonThrough({
              chatText: async function chatText(request,) {
                seen.push({ ...request, },);
                return { text: '{"cat":"Mittens"}', };
              },
            },);
            const outcome = await chatJson({
              modelId: 'minimax-m3',
              messages: [{ role: 'user', content: 'Name the cat.', },],
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              maxTokens: 64,
              otherThan: 'synthetic',
              validate: namesCat,
            },);
            expect(outcome.kind,).toBe('ok',);
            expect(Object.keys(seen[0] ?? {},).toSorted(),).toEqual([
              'maxTokens',
              'messages',
              'modelId',
              'otherThan',
              'signal',
            ],);
          },
        },),

        it({
          name: 'READS THE REPLY AGAINST THE RESPONSE FORMAT IT SENT, taking the schema\'s spelling of a listed value '
            + 'that came back differing only in its capital',
          fn: async () => {
            const chatJson = chatJsonThrough({
              chatText: async function chatText() {
                return { text: '{"cat":"Tabby"}', };
              },
            },);
            const outcome = await chatJson({
              modelId: 'minimax-m3',
              messages: [{ role: 'user', content: 'Name the cat.', },],
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              responseFormat: {
                type: 'json_schema',
                json_schema: {
                  name: 'cat_name',
                  schema: {
                    type: 'object',
                    properties: { cat: { type: 'string', enum: ['mittens', 'tabby',], }, },
                  },
                },
              },
              validate: function namesListedCat(value: unknown,): value is { readonly cat: string; } {
                return namesCat(value,) && ((value.cat === 'mittens') || (value.cat === 'tabby'));
              },
            },);
            expect(outcome.kind === 'ok' ? outcome.value : undefined,).toEqual({ cat: 'tabby', },);
          },
        },),
      ],
    },),

    describe({
      name: perModelLimiter.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'HANDS EACH MODEL ONE LIMITER OF ITS OWN, kept across calls, at the concurrency asked for',
          fn: async () => {
            const limiterFor = perModelLimiter({ perModelConcurrency: 2, },);
            expect(limiterFor('minimax-m3',),).toBe(limiterFor('minimax-m3',),);
            expect(limiterFor('minimax-m3',),).not.toBe(limiterFor('google.gemma-4-31b',),);
            expect(limiterFor('minimax-m3',).concurrency,).toBe(2,);
          },
        },),
      ],
    },),
  ],
},);
