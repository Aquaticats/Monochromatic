/**
 Dispatch entry failures and the catalog's advertised limits at the built provider boundary.
 @module
 */

import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import extension from '../dist/final/node/index.mjs';
import { WRITER_TARGETS, } from './member-order-fixture.ts';
import {
  collect,
  fixture,
  immediate,
  PHYSICAL_MODELS,
  poolModel,
  TRANSCRIPT,
} from './stand-in-pi-fixture.ts';

/**
 Window and output limit Pi itself falls back to for a model that declares none
 (`@earendil-works/pi-coding-agent` dist/core/provider-composer.js, `modelFromJson`), which the pool
 advertises until a session registry binds.
 */
const PROVISIONAL_CONTEXT_WINDOW = 128_000;
const PROVISIONAL_MAX_TOKENS = 16_384;

/** States that cannot legally reach inference. */
const UNDISPATCHABLE = {
  'before starting a session': { startSession: false, message: 'requires a started Pi session', },
  'with an unknown virtual model': { id: 'unknown-model', message: 'Unknown rate-limit-fallback model', },
  'with a mismatched registry model': { findModel: () => PHYSICAL_MODELS[1], message: 'Pi registry returned a different model', },
  'without any registered member': { models: [], message: 'No writer member is registered in Pi', },
} as const;

/**
 Writer members the catalog cases register with their own limits.

 @param limits - window and output limit one case reports

 @param index - which writer member to base the record on

 @returns catalog record carrying those limits
 */
function memberWithLimits({
  limits,
  index,
}: {
  readonly limits: { readonly contextWindow: number; readonly maxTokens: number };
  readonly index: 0 | 1;
},) {
  return {
    ...WRITER_TARGETS[index],
    ...limits,
  };
}

/**
 Window and output limit one catalog case advertises, in the order its members are registered.

 @param models - catalog records the fixture registers

 @returns writer entry Pi would read
 */
async function writerEntryFor(models: ReturnType<typeof memberWithLimits>[],) {
  const { provider, } = await fixture({
    extension,
    models,
    play({ model, }) { return immediate({ model, },); },
  },);
  const [writerEntry,] = provider.getModels();
  return writerEntry;
}

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

/** Catalog cases: the limits Pi reads before a member is dispatched. */
const CATALOG = [
  it({
    name: 'advertises the smallest limits its registered members report',
    fn: async function smallestLimits() {
      const writerEntry = await writerEntryFor([
        memberWithLimits({ limits: { contextWindow: 40_000, maxTokens: 8_000, }, index: 0, },),
        memberWithLimits({ limits: { contextWindow: 60_000, maxTokens: 4_000, }, index: 1, },),
      ],);
      expect(writerEntry?.contextWindow,).toBe(40_000,);
      expect(writerEntry?.maxTokens,).toBe(4_000,);
    },
  },),
  it({
    name: 'passes over a member that reports no limits',
    fn: async function zeroLimitMember() {
      const writerEntry = await writerEntryFor([
        memberWithLimits({ limits: { contextWindow: 0, maxTokens: 0, }, index: 0, },),
        memberWithLimits({ limits: { contextWindow: 50_000, maxTokens: 9_000, }, index: 1, },),
      ],);
      expect(writerEntry?.contextWindow,).toBe(50_000,);
      expect(writerEntry?.maxTokens,).toBe(9_000,);
    },
  },),
  it({
    name: 'falls back to Pi’s own window when no member is registered',
    fn: async function noMembers() {
      const writerEntry = await writerEntryFor([],);
      expect(writerEntry?.contextWindow,).toBe(PROVISIONAL_CONTEXT_WINDOW,);
      expect(writerEntry?.maxTokens,).toBe(PROVISIONAL_MAX_TOKENS,);
    },
  },),
];

await describe({
  name: 'entry points and advertised limits',
  children: [...REJECTIONS, ...CATALOG,],
},);
