import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

// TODO: import from package.
import { $ as isBigint, } from './index.ts';

await describe({
  name: isBigint.name,
  children: [
    it({
      name: 'returns true for bigint values',
      fn: async () => {
        expect(isBigint(0n,),).toBe(true,);
        expect(isBigint(1n,),).toBe(true,);
        expect(isBigint(-1n,),).toBe(true,);
        expect(isBigint(BigInt(Number.MAX_SAFE_INTEGER,) + 1n,),).toBe(true,);
        expect(isBigint(BigInt(Number.MIN_SAFE_INTEGER,) - 1n,),).toBe(true,);
        // oxlint-disable-next-line unicorn/prefer-bigint-literals -- testing
        expect(
          isBigint(123_456_789_012_345_678_901_234_567_890n,),
        ).toBe(true,);
      },
    },),
    it({
      name: 'returns false for non-bigint values',
      fn: async () => {
        expect(isBigint(0,),).toBe(false,);
        expect(isBigint(1,),).toBe(false,);
        expect(isBigint(-1,),).toBe(false,);
        expect(isBigint(Number.NaN,),).toBe(false,);
        expect(isBigint(Infinity,),).toBe(false,);
        expect(isBigint(-Infinity,),).toBe(false,);
        expect(isBigint(null,),).toBe(false,);
        expect(isBigint(undefined,),).toBe(false,);
        expect(isBigint('',),).toBe(false,);
        expect(isBigint('0',),).toBe(false,);
        expect(isBigint('123n',),).toBe(false,);
        expect(isBigint({},),).toBe(false,);
        expect(isBigint([],),).toBe(false,);
        expect(isBigint(() => {/* intentionally empty */},),).toBe(false,);
        expect(
          isBigint(Symbol('test',),),
        ).toBe(false,);
        expect(isBigint(true,),).toBe(false,);
        expect(isBigint(false,),).toBe(false,);
      },
    },),
  ],
},);
