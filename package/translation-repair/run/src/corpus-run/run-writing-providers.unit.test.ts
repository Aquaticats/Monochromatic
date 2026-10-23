/**
 Tests for the providers whose wet reading keeps a run going: those that serve a model seated in a writing bench.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { reachOf, } from '@monochromatic-dev/translation-repair-provider';
import { writingProvidersOf, } from '../../dist/final/node/index.mjs';

await describe({
  name: writingProvidersOf.name,
  concurrency: 1,
  children: [
    it({
      name: 'MARKS NO PROVIDER FOR NO MODEL, and exactly the providers that serve a model for the models handed in',
      fn: async () => {
        /**
         A model Xiaomi serves, and no provider but Xiaomi.
         */
        const xiaomiModel = 'xiaomi/mimo-v2.6-flash';
        const none = writingProvidersOf({ modelIds: [], },);
        const xiaomi = writingProvidersOf({ modelIds: [xiaomiModel,], },);
        expect(Object.values(none,).some(Boolean,),).toBe(false,);
        expect(xiaomi,).toEqual(reachOf({ modelId: xiaomiModel, },),);
        expect(xiaomi.xiaomi,).toBe(true,);
      },
    },),
  ],
},);
