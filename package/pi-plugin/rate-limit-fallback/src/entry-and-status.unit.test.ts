/**
 Legacy status prefixes and dispatch entry failures at the built provider boundary.
 @module
 */

import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import extension from '../dist/final/node/index.mjs';
import {
  collect,
  fixture,
  immediate,
  PHYSICAL_MODELS,
  poolModel,
  RATE_LIMIT_STATUS,
  TARGETS,
  TRANSCRIPT,
} from './stand-in-pi-fixture.ts';

/** States that cannot legally reach inference. */
const UNDISPATCHABLE = {
  'before starting a session': { startSession: false, message: 'requires a started Pi session', },
  'with an unknown virtual model': { id: 'unknown-model', message: 'Unknown rate-limit-fallback model', },
  'with a mismatched registry model': { findModel: () => PHYSICAL_MODELS[1], message: 'Pi registry returned a different model', },
  'without any registered member': { models: [], message: 'No writer member is registered in Pi', },
} as const;

/** The first refusal advances to the next writer member. */
const [OPENAI, QWEN,] = TARGETS;

await describe({
  name: 'entry points and status prefixes',
  children: [
    ...Object.entries(UNDISPATCHABLE,).map(function invalidEntry([label, entry,],) {
      return it({
        name: `returns a terminal setup error ${label}`,
        fn: async function rejectEntry() {
          const { provider, calls, options, } = await fixture({
            extension,
            ...entry,
            play({ model, }) { return immediate({ model, },); },
          },);
          const writer = poolModel({ provider, id: 'writer', },);
          const model = 'id' in entry ? { ...writer, id: entry.id, } : writer;
          const { result, } = await collect(provider.stream(model, TRANSCRIPT, options,),);
          expect(result.stopReason,).toBe('error',);
          expect(result.errorMessage,).toContain(entry.message,);
          expect(calls,).toEqual([],);
        },
      },);
    },),
    ...['Status:', 'Status Code:',].map(function legacyPrefix(prefix,) {
      return it({
        name: `hands off on the legacy ${prefix} status prefix via the API-specific stream entry`,
        fn: async function apiSpecificStream() {
          const { provider, calls, options, } = await fixture({
            extension,
            play({ model, }) {
              return model.provider === OPENAI.provider
                ? immediate({ model, errorMessage: `${prefix}\t${RATE_LIMIT_STATUS}: Capacity reached`, },)
                : immediate({ model, },);
            },
          },);
          const model = poolModel({ provider, id: 'writer', },);
          const { result, } = await collect(provider.stream(model, TRANSCRIPT, {
            ...options,
            maxTokens: Number.MAX_SAFE_INTEGER,
          },),);
          expect(result.provider,).toBe(QWEN.provider,);
          expect(calls.map(call => call.model.provider,),).toEqual([OPENAI.provider, QWEN.provider,],);
          expect(calls.every(call => call.options?.maxTokens === call.model.maxTokens,),).toBe(true,);
        },
      },);
    },),
  ],
},);
