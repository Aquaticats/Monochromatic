/**
 Tests for the counted range that replaces a classic counting loop.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { numbersBelow, } from './numbers-below.ts';

await describe({
  name: numbersBelow.name,
  children: [
    it({
      name: 'gives the numbers from zero to the bound, exclusive',
      fn: async ({ expect, }) => {
        expect([...numbersBelow(4,),],).toEqual([0, 1, 2, 3,],);
        expect([...numbersBelow(1,),],).toEqual([0,],);
      },
    },),
    it({
      name: 'gives no number for a bound of zero or less',
      fn: async ({ expect, }) => {
        expect([...numbersBelow(0,),],).toEqual([],);
        expect([...numbersBelow(-3,),],).toEqual([],);
      },
    },),
    it({
      name: 'keeps every step a counter below a fractional bound took',
      fn: async ({ expect, }) => {
        // A counter that steps up from zero while it stays below 2.5 stops after these three steps.
        expect([...numbersBelow(2.5,),],).toEqual([0, 1, 2,],);
      },
    },),
    it({
      name: 'gives the first number of a bound too large for an array to hold',
      fn: async ({ expect, }) => {
        expect(numbersBelow(Number.MAX_SAFE_INTEGER,).next(),).toEqual({ value: 0, done: false, },);
      },
    },),
    it({
      name: 'keeps the position of a walk that is half read to itself',
      fn: async ({ expect, }) => {
        const halfRead = numbersBelow(3,);
        halfRead.next();
        expect([...numbersBelow(3,),],).toEqual([0, 1, 2,],);
        expect([...halfRead,],).toEqual([1, 2,],);
      },
    },),
  ],
},);
