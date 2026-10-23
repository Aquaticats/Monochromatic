/**
 Tests for the per-model slots each provider grants by default: the two real ceilings and no ceiling elsewhere, a row
 for every provider the roster names.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { PROVIDER_ORDER, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  DEEPSEEK_PER_MODEL_CONCURRENCY,
  DEFAULT_SLOT_LIMITS,
  NEURALWATT_PER_MODEL_CONCURRENCY,
  SYNTHETIC_PER_MODEL_CONCURRENCY,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'DEFAULT_SLOT_LIMITS',
  children: [
    it({
      name: 'READS each client\'s own default where it has one and NO CEILING for the providers that state a request '
        + 'rate (Xiaomi, the Claude key) or nothing at all, with a row for every provider',
      fn: async () => {
        expect(Object.keys(DEFAULT_SLOT_LIMITS,).toSorted(),).toEqual([...PROVIDER_ORDER,].toSorted(),);
        expect(DEFAULT_SLOT_LIMITS.neuralwatt,).toBe(NEURALWATT_PER_MODEL_CONCURRENCY,);
        expect(DEFAULT_SLOT_LIMITS.synthetic,).toBe(SYNTHETIC_PER_MODEL_CONCURRENCY,);
        expect(DEFAULT_SLOT_LIMITS.deepseek,).toBe(DEEPSEEK_PER_MODEL_CONCURRENCY,);
        for (const provider of ['hyper', 'bedrock', 'xiaomi', 'anthropic', 'openrouter',] as const)
          expect(DEFAULT_SLOT_LIMITS[provider],).toBe(Number.POSITIVE_INFINITY,);
      },
    },),
  ],
},);
