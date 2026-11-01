/**
 Width scaling of the two-input record path.

 Timing is too noisy to gate on, so this counts the own-property checks
 deepmerge-ts makes on its inputs, a deterministic proxy for the work the
 record merge does per key. With two inputs the count must grow exactly
 linearly with the number of keys; a superlinear change (for example a key
 lookup that rescans the other input) turns this red.

 Measured in a capped container on 8.0.2: 10^6 keys per input merge in
 about 1.5 s, roughly 14x the 10^5 time (hashing and GC overhead),
 for every entry point.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { target, } from './target.ts';

/**
 Keys per input at the smaller size; the larger size doubles it.
 */
const BASE_WIDTH = 1_000;

/**
 Count own-property checks for two records of `width` keys, half overlapping.

 @param width - Keys per input.

 @returns Number of `[[GetOwnProperty]]` calls observed.

 @example
 ```ts
 const checks = twoInputChecks(10);
 ```
 */
function twoInputChecks(width: number,): number {
  /**
   Mutable tally shared with the proxy traps.
   */
  const tally = { checks: 0, };
  /**
   Two records: keys `0..width` and `width/2..width*1.5`.
   */
  const records = [0, width / 2,].map(function countedRecord(offset,) {
    /**
     Plain record before wrapping.
     */
    const plain = Object.fromEntries(Array.from({ length: width, }, function entry(_unused, index,) {
      return [`k${String(index + offset,)}`, index,];
    },),);
    return new Proxy(plain, {
      getOwnPropertyDescriptor: function countCheck(record, key,) {
        tally.checks += 1;
        return Reflect.getOwnPropertyDescriptor(record, key,);
      },
    },);
  },);
  target.deepmerge(...records,);
  return tally.checks;
}

await describe({
  name: 'deepmerge-ts width scaling',
  children: [
    it({
      name: 'two-input record merges check each key a constant number of times',
      fn: async () => {
        /**
         Checks at the base width.
         */
        const base = twoInputChecks(BASE_WIDTH,);
        expect(base,).toBeGreaterThan(0,);
        expect(twoInputChecks(BASE_WIDTH * 2,),).toBe(base * 2,);
      },
    },),
  ],
},);
