/**
 Runtime replay of the committed declared-type corpus
 (`./declared-type-soundness.generated.ts`).

 `lint:types` proves each case's literal is assignable to the declared-input
 result type. This test proves the literal still equals what deepmerge
 returns, so the static proof keeps describing the real result: a behavior
 change in deepmerge-ts that the literal no longer matches fails here, and the
 corpus must be rebuilt (`node src/declared-type-campaign.ts corpus`) and
 type-checked again.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { DECLARED_TYPE_CASES, } from './declared-type-soundness.generated.ts';

await describe({
  name: 'deepmerge-ts declared-type corpus replay',
  children: DECLARED_TYPE_CASES.map(function replay(declaredCase, id,) {
    return it({
      name: `case ${String(id,)} result equals its type-checked literal`,
      fn: async () => {
        /**
         Merge result and the literal `lint:types` checked against its type.
         */
        const { merged, value, } = declaredCase();
        expect(merged,).toEqual(value,);
      },
    },);
  },),
},);
