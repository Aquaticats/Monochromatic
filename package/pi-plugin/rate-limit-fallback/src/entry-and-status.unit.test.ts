/**
 Dispatch entry failures at the built provider boundary.
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
  TRANSCRIPT,
} from './stand-in-pi-fixture.ts';

/** States that cannot legally reach inference. */
const UNDISPATCHABLE = {
  'before starting a session': { startSession: false, message: 'requires a started Pi session', },
  'with an unknown virtual model': { id: 'unknown-model', message: 'Unknown rate-limit-fallback model', },
  'with a mismatched registry model': { findModel: () => PHYSICAL_MODELS[1], message: 'Pi registry returned a different model', },
  'without any registered member': { models: [], message: 'No writer member is registered in Pi', },
} as const;

/** One test case per entry the provider must refuse before it asks any member. */
const REJECTIONS = Object.entries(UNDISPATCHABLE,).map(function invalidEntry([label, entry,],) {
  return it({
    name: `returns a terminal setup error ${label}`,
    fn: async function rejectEntry() {
      const { provider, calls, options, } = await fixture({
        extension,
        ...entry,
        play({ model, }) { return immediate({ model, },); },
      },);
      const writer = poolModel({ provider, id: 'writer', },);
      const model = ('id' in entry) ? { ...writer, id: entry.id, } : writer;
      const { result, } = await collect(provider.stream(model, TRANSCRIPT, options,),);
      expect(result.stopReason,).toBe('error',);
      expect(result.errorMessage,).toContain(entry.message,);
      expect(calls,).toEqual([],);
    },
  },);
},);

await describe({
  name: 'entry points',
  children: REJECTIONS,
},);
