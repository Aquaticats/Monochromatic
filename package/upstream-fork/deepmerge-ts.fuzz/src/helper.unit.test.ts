/**
 Unit tests for the sidecar's own helpers: structural comparison and
 snapshots, the V8 line projection, the literal emitter, and the run plan.

 The properties trust these helpers as oracles, so each branch that decides
 a verdict is pinned here with a case that must pass and one that must not.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  codeLineCount,
  coveredLines,
  isCoverageFile,
  uncalledFunctions,
} from './coverage-v8.ts';
import {
  firstNonBlankColumn,
  ranges,
} from './coverage-source.ts';
import {
  ForkBuildError,
  replaceOnce,
} from './fork-build.ts';
import {
  FuzzPlanError,
  fuzzRunPlan,
} from './fuzz-budget.ts';
import { recordKeys, } from './model.ts';
import {
  NO_MISMATCH,
  shapeMismatch,
  snapshot,
  snapshotObject,
} from './shape.ts';
import {
  emitConstArgument,
  emitLiteral,
} from './type-case-generate.ts';

/**
 Set campaign variables for one scope and restore them on dispose.

 @param seed - Value for `DEEPMERGE_FUZZ_SEED`.
 @param numRuns - Value for `DEEPMERGE_FUZZ_NUM_RUNS`.

 @returns Disposable restoring the previous environment.

 @example
 ```ts
 using _env = campaignEnv({ seed: '1', numRuns: '2', });
 ```
 */
function campaignEnv({ seed, numRuns, }: { readonly seed: string; readonly numRuns: string; },): Disposable {
  /**
   Previous values, restored on dispose.
   */
  const previous = { numRuns: process.env.DEEPMERGE_FUZZ_NUM_RUNS, seed: process.env.DEEPMERGE_FUZZ_SEED, };
  process.env.DEEPMERGE_FUZZ_SEED = seed;
  process.env.DEEPMERGE_FUZZ_NUM_RUNS = numRuns;
  return {
    [Symbol.dispose]: function restoreEnv() {
      for (const [name, value,] of [['DEEPMERGE_FUZZ_SEED', previous.seed,], ['DEEPMERGE_FUZZ_NUM_RUNS', previous.numRuns,],] as const) {
        if (value === undefined)
          Reflect.deleteProperty(process.env, name,);
        else
          process.env[name] = value;
      }
    },
  };
}

await describe({
  name: 'sidecar helpers',
  children: [
    it({
      name: 'shapeMismatch accepts equal trees and reports each observable difference',
      fn: async () => {
        expect(shapeMismatch({ actual: { a: [1, new Set([2,],),], }, expected: { a: [1, new Set([2,],),], }, },),).toBe(NO_MISMATCH,);
        expect(shapeMismatch({ actual: { a: 1, b: 2, }, expected: { b: 2, a: 1, }, },),).toContain('own keys differ',);
        expect(shapeMismatch({ actual: Object.create(null,), expected: {}, },),).toContain('prototype differs',);
        // oxlint-disable-next-line no-sparse-arrays -- the hole is the difference under test.
        expect(shapeMismatch({ actual: [, 1,], expected: [undefined, 1,], },),).toContain('own keys differ',);
        expect(shapeMismatch({ actual: new Set([1, 2,],), expected: new Set([2, 1,],), },),).toContain('Set',);
        expect(shapeMismatch({ actual: new Map([['k', 1,],],), expected: new Map([['k', 2,],],), },),).toBe('$.k: expected 2, got 1',);
        expect(shapeMismatch({ actual: new Date(0,), expected: new Date(0,), },),).not.toBe(NO_MISMATCH,);
        expect(shapeMismatch({ actual: Number.NaN, expected: Number.NaN, },),).toBe(NO_MISMATCH,);
      },
    },),
    it({
      name: 'snapshot copies containers but keeps leaves, descriptors, and frozen state',
      fn: async () => {
        /**
         Leaf shared by the original and the copy.
         */
        const leaf = new Date(0,);
        /**
         Original with a hidden key, a getter, and a frozen nested record.
         */
        const original = Object.defineProperties({ nested: Object.freeze({ leaf, },), list: [1,], }, {
          hidden: { configurable: true, enumerable: false, value: 1, writable: true, },
          read: { configurable: true, enumerable: true, get: function read() {
            return 2;
          }, },
        },);
        /**
         Structural copy under test.
         */
        const copy = snapshotObject(original,);
        expect(copy,).not.toBe(original,);
        expect(shapeMismatch({ actual: copy, expected: original, },),).toBe(NO_MISMATCH,);
        expect(Reflect.get(Reflect.get(copy, 'nested',) as object, 'leaf',),).toBe(leaf,);
        expect(
          Object.isFrozen(Reflect.get(copy, 'nested',),),
        ).toBe(true,);
        expect(typeof Reflect.getOwnPropertyDescriptor(copy, 'read',)?.get,).toBe('function',);
        expect(snapshot(leaf,),).toBe(leaf,);
      },
    },),
    it({
      name: 'recordKeys lists string keys then enumerable symbols only',
      fn: async () => {
        /**
         Enumerable symbol key.
         */
        const shown = Symbol('enumerable symbol key under test',);
        /**
         Non-enumerable symbol key.
         */
        const hidden = Symbol('hidden symbol key under test',);
        /**
         Record mixing both.
         */
        const record = Object.defineProperty({ b: 1, [shown]: 2, a: 3, }, hidden, { enumerable: false, value: 4, },);
        expect(recordKeys(record,),).toEqual(['b', 'a', shown,],);
      },
    },),
    it({
      name: 'coveredLines lets the innermost range decide and ignores blank lines',
      fn: async () => {
        /**
         Three code lines and one blank line.
         */
        const source = 'one();\nif (x) {\n  never();\n\n}';
        /**
         Script whose inner block (line 2) never ran.
         */
        const script = {
          url: 'file:///x.mjs',
          functions: [{
            functionName: '',
            ranges: [
              { startOffset: 0, endOffset: source.length, count: 1, },
              { startOffset: source.indexOf('never',), endOffset: source.indexOf('\n\n',), count: 0, },
            ],
          },],
        };
        expect([...coveredLines({ script, source, },),],).toEqual([0, 1, 4,],);
        expect(codeLineCount(source,),).toBe(4,);
        expect(isCoverageFile({ result: [script,], },),).toBe(true,);
        expect(isCoverageFile({ result: 'no', },),).toBe(false,);
        expect(uncalledFunctions([{ url: 'u', functions: [{ functionName: 'idle', ranges: [{ startOffset: 0, endOffset: 1, count: 0, },], },], },],),)
          .toEqual(['idle',],);
      },
    },),
    it({
      name: 'source report helpers collapse ranges and find the first code column',
      fn: async () => {
        expect(ranges([7, 1, 3, 2, 9, 10,],),).toBe('1-3, 7, 9-10',);
        expect(ranges([],),).toBe('',);
        expect(firstNonBlankColumn('   x = 1;',),).toBe(3,);
        expect(firstNonBlankColumn('  \t ',),).toBe(-1,);
      },
    },),
    it({
      name: 'replaceOnce edits the fork config or fails loudly when it drifted',
      fn: async () => {
        expect(replaceOnce({ from: 'sourcemap: false', text: 'a sourcemap: false b', to: 'sourcemap: true', },),).toBe('a sourcemap: true b',);
        expect(() => replaceOnce({ from: 'absent', text: 'config', to: 'x', },),).toThrow(ForkBuildError,);
      },
    },),
    it({
      name: 'emitLiteral writes typeable literals and emitConstArgument adds const only where legal',
      fn: async () => {
        expect(
          emitLiteral(Object.defineProperty({}, '__proto__', { enumerable: true, value: 1, },),),
        ).toBe('{ ["__proto__"]: 1 }',);
        expect(emitLiteral([],),).toBe('([] as never[])',);
        expect(
          emitLiteral(new Set(),),
        ).toBe('new Set<never>()',);
        expect(
          emitLiteral(new Map([['k', 1,],],),),
        ).toBe('mapOf([["k", 1]])',);
        expect(
          emitLiteral(new Date(3,),),
        ).toBe('new Date(3)',);
        expect(emitConstArgument([],),).toBe('[] as const',);
        expect(emitConstArgument({ a: 1, },),).toBe('{ "a": 1 } as const',);
        expect(emitConstArgument(null,),).toBe('null',);
        expect(
          emitConstArgument(new Set([1,],),),
        ).toBe('new Set([1])',);
        expect(() => emitLiteral(Symbol('symbol with no literal form',),),).toThrow(TypeError,);
      },
    },),
    it({
      name: 'fuzzRunPlan is bounded by default, follows campaign variables, and rejects malformed ones',
      fn: async () => {
        expect(fuzzRunPlan().params.seed,).toBe(20_260_923,);
        {
          using _campaign = campaignEnv({ numRuns: '7', seed: '11', },);
          expect(fuzzRunPlan().params,).toEqual({ numRuns: 7, seed: 11, },);
        }
        {
          using _malformed = campaignEnv({ numRuns: '7', seed: 'eleven', },);
          expect(() => fuzzRunPlan(),).toThrow(FuzzPlanError,);
        }
        expect(fuzzRunPlan().params.seed,).toBe(20_260_923,);
      },
    },),
  ],
},);
