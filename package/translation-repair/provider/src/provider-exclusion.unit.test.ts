/**
 Tests the providers a re-ask must stay off (ledger B1410): the request's own, then the provider that just answered.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ChatTextRequest,
  excludingAlso,
} from '../dist/final/node/index.mjs';

//region Provider exclusion tests

/**
 A request that carries no hint.
 */
const PLAIN: ChatTextRequest = {
  modelId: 'hf:zai-org/GLM-5.3-Flash',
  messages: [{ role: 'user', content: '喵？', },],
  signal: AbortSignal.timeout(1,),
};

await describe({
  name: 'the providers a re-ask must stay off',
  concurrency: 1,
  children: [
    it({
      name: 'NAMES ONLY THE PROVIDER THAT ANSWERED where the request carries no hint',
      fn: async () => {
        expect(excludingAlso({ request: PLAIN, provider: 'hyper', },),).toEqual(['hyper',],);
      },
    },),

    it({
      name: 'KEEPS EVERY PROVIDER THE REQUEST CARRIED, in its order, and puts the one that answered after them',
      fn: async () => {
        expect(excludingAlso({ request: { ...PLAIN, otherThan: ['synthetic', 'bedrock',], }, provider: 'hyper', },),)
          .toEqual(['synthetic', 'bedrock', 'hyper',],);
      },
    },),

    it({
      name: 'NAMES THE PROVIDER THAT ANSWERED ONCE where the request already excluded it, and hands the list back whole',
      fn: async () => {
        /**
         Hint the request carries.
         */
        const hint = ['synthetic', 'hyper',] as const;
        expect(excludingAlso({ request: { ...PLAIN, otherThan: hint, }, provider: 'hyper', },),).toEqual(hint,);
      },
    },),
  ],
},);

//endregion Provider exclusion tests
