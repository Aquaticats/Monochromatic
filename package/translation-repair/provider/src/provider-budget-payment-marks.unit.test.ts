/**
 Tests for the marks that keep a provider dry after a refusal for payment, until its meter moves.

 Fixtures are cat-themed invention; the levels are plain words and no meter is read.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  providerRecord,
  type ProviderRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { createPaymentMarks, } from '../dist/final/node/index.mjs';

/**
 Levels the meters read, one word for every provider.

 @param word - level every meter reads

 @returns Level of each provider

 @example
 ```ts
 const levels = levelsOf({ word: 'tabby', },);
 ```
 */
function levelsOf(
  { word, }: { readonly word: string; },
): ProviderRecord<string> {
  return providerRecord({ of: () => word, },);
}

await describe({
  name: createPaymentMarks.name,
  concurrency: 1,
  children: [
    it({
      name: 'HOLDS NO PROVIDER DRY BEFORE A REFUSAL marks one, whatever the meters read',
      fn: async () => {
        const { paidOut, } = createPaymentMarks();
        expect(paidOut({ provider: 'hyper', levels: levelsOf({ word: 'tabby', },), },),).toBe(false,);
      },
    },),

    it({
      name: 'HOLDS THE MARKED PROVIDER DRY WHILE ITS METER READS THE LEVEL IT READ AT THE REFUSAL, and no other',
      fn: async () => {
        const { paidOut, markPaid, } = createPaymentMarks();
        markPaid({ provider: 'hyper', level: 'tabby', },);
        expect(paidOut({ provider: 'hyper', levels: levelsOf({ word: 'tabby', },), },),).toBe(true,);
        expect(paidOut({ provider: 'bedrock', levels: levelsOf({ word: 'tabby', },), },),).toBe(false,);
      },
    },),

    it({
      name: 'CLEARS THE MARK WHEN THE METER MOVES, either way, and does not bring it back when the level returns',
      fn: async () => {
        const { paidOut, markPaid, } = createPaymentMarks();
        markPaid({ provider: 'hyper', level: 'tabby', },);
        expect(paidOut({ provider: 'hyper', levels: levelsOf({ word: 'calico', },), },),).toBe(false,);
        expect(paidOut({ provider: 'hyper', levels: levelsOf({ word: 'tabby', },), },),).toBe(false,);
      },
    },),

    it({
      name: 'MARKS A PROVIDER AGAIN AT THE LEVEL OF A LATER REFUSAL after the first mark was cleared',
      fn: async () => {
        const { paidOut, markPaid, } = createPaymentMarks();
        markPaid({ provider: 'hyper', level: 'tabby', },);
        expect(paidOut({ provider: 'hyper', levels: levelsOf({ word: 'calico', },), },),).toBe(false,);
        markPaid({ provider: 'hyper', level: 'calico', },);
        expect(paidOut({ provider: 'hyper', levels: levelsOf({ word: 'calico', },), },),).toBe(true,);
      },
    },),
  ],
},);
