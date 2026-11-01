/**
 Tests for where each direct provider reaches the roster: one model each, spelled one way, over every roster entry.
 Nothing reaches a provider.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ROSTER_MODEL_IDS, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  DEEPSEEK_MODELS,
  deepSeekIdFor,
  deepSeekShowsPictures,
  XIAOMI_MODELS,
  xiaomiIdFor,
  xiaomiShowsPictures,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_OPENROUTER_ONLY_CHECKER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

await describe({
  name: 'direct reach',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'SPELLS only the one model each provider is granted for, and shows pictures for neither of the others',
      fn: async () => {
        expect(deepSeekIdFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, },),)
          .toEqual({ served: true, id: 'deepseek-flash', },);
        expect(deepSeekIdFor({ modelId: SEAT_OPENROUTER_ONLY_CHECKER, },),).toEqual({ served: false, },);
        expect(xiaomiIdFor({ modelId: SEAT_OPENROUTER_ONLY_CHECKER, },),)
          .toEqual({ served: true, id: 'mimo-v2.6-flash', },);
        expect(xiaomiIdFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, },),).toEqual({ served: false, },);
        expect(deepSeekShowsPictures({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, },),).toBe(false,);
        expect(xiaomiShowsPictures({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, },),).toBe(false,);
        expect(deepSeekShowsPictures({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, },),)
          .toBe(DEEPSEEK_MODELS['deepseek-flash'].readsImages,);
        expect(xiaomiShowsPictures({ modelId: SEAT_OPENROUTER_ONLY_CHECKER, },),)
          .toBe(XIAOMI_MODELS['mimo-v2.6-flash'].readsImages,);
      },
    },),
    it({
      name: 'SERVES exactly one roster model on each provider, found by walking every roster entry, and shows a '
        + 'picture on no model it does not serve',
      fn: async () => {
        const deepSeekServed = ROSTER_MODEL_IDS.filter((modelId,) => deepSeekIdFor({ modelId, },).served);
        const xiaomiServed = ROSTER_MODEL_IDS.filter((modelId,) => xiaomiIdFor({ modelId, },).served);
        expect(deepSeekServed,).toEqual([SEAT_HYPER_OPENROUTER_UNMEASURED,],);
        expect(xiaomiServed,).toEqual([SEAT_OPENROUTER_ONLY_CHECKER,],);
        for (const modelId of ROSTER_MODEL_IDS) {
          if (modelId !== SEAT_HYPER_OPENROUTER_UNMEASURED)
            expect(deepSeekShowsPictures({ modelId, },),).toBe(false,);
          if (modelId !== SEAT_OPENROUTER_ONLY_CHECKER)
            expect(xiaomiShowsPictures({ modelId, },),).toBe(false,);
        }
      },
    },),
  ],
},);
