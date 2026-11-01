/**
 Tests for `functionOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { functionOrThrow, } from '@monochromatic-dev/or-throw-core';

function double(n: number,): number {
  return n * 2;
}

async function asyncOne(): Promise<number> {
  return 1;
}

function* yieldOne(): Generator<number> {
  yield 1;
}

function noop(): void {}

await describe({
  name: functionOrThrow.name,
  children: [
    it({
      name: 'returns named function declarations unchanged',
      fn: async () => {
        expect(functionOrThrow(double,),).toBe(double,);
      },
    },),

    it({
      name: 'returns async, generator, and built-in functions unchanged',
      fn: async () => {
        expect(functionOrThrow(asyncOne,),).toBe(asyncOne,);
        expect(functionOrThrow(yieldOne,),).toBe(yieldOne,);
        expect(functionOrThrow(Array.from,),).toBe(Array.from,);
      },
    },),

    it({
      name: 'throws on non-callable values',
      fn: async () => {
        expect(() => functionOrThrow({},)).toThrow('function',);
        expect(() => functionOrThrow([],)).toThrow('function',);
        expect(() => functionOrThrow('fn',)).toThrow('function',);
        expect(() => functionOrThrow(null,)).toThrow('function',);
        expect(() => functionOrThrow(undefined,)).toThrow('function',);
      },
    },),

    it({
      name: 'narrows unknown to a callable type',
      fn: async () => {
        const input: unknown = noop;
        const output = functionOrThrow(input,);
        expectTypeOf(output,).toBeFunction();
      },
    },),
  ],
},);
