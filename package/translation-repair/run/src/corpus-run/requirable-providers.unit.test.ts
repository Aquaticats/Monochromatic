/**
 Tests for the providers a validation arm may require wet: every provider but the two optional direct ones, in the order
 the pass names them.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { PROVIDER_ORDER, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  isRequirable,
  REQUIRABLE_PROVIDERS,
} from '../../dist/final/node/index.mjs';

await describe({
  name: isRequirable.name,
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'ACCEPTS every provider an arm may require, and REFUSES the three optional direct ones and anything that is '
        + 'not a provider name',
      fn: async () => {
        for (const name of ['neuralwatt', 'synthetic', 'bedrock', 'hyper', 'openrouter',])
          expect(isRequirable(name,),).toBe(true,);
        for (const name of ['deepseek', 'xiaomi', 'anthropic', '', 'Hyper', 'openrouter ', 'nobody',])
          expect(isRequirable(name,),).toBe(false,);
      },
    },),
    it({
      name: 'LISTS the whole provider order minus the three direct providers, in that order, so a provider added to the '
        + 'order is required unless it is left out here on purpose',
      fn: async () => {
        expect(REQUIRABLE_PROVIDERS,)
          .toEqual(PROVIDER_ORDER.filter((name,) => (name !== 'deepseek') && (name !== 'xiaomi') && (name !== 'anthropic')),);
      },
    },),
  ],
},);
