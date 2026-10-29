/**
 Example tests for the reference model, the JSON corpus, and the build under
 test agreeing on it.

 The properties compare deepmerge-ts against the model, so a model bug could
 hide an upstream bug; these fixed cases pin the model to the documented
 examples independently of deepmerge-ts.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { JSON_CASES, } from './json-case.ts';
import {
  kindOf,
  modelMerge,
} from './model.ts';
import {
  NO_MISMATCH,
  shapeMismatch,
} from './shape.ts';
import { target, } from './target.ts';

await describe({
  name: 'reference model',
  children: [
    it({
      name: 'every JSON corpus case survives a JSON round trip unchanged',
      fn: async () => {
        for (const jsonCase of JSON_CASES) {
          /**
           Serialized case; JSON.stringify drops or rejects non-JSON values.
           */
          const text = JSON.stringify(jsonCase,);
          /**
           Case parsed back from its JSON text.
           */
          const reparsed: unknown = JSON.parse(text,);
          expect(
            shapeMismatch({
              actual: reparsed,
              expected: jsonCase,
              path: jsonCase.name,
            },),
          )
            .toBe(NO_MISMATCH,);
        }
      },
    },),
    it({
      name: 'model matches every JSON corpus case',
      fn: async () => {
        for (const jsonCase of JSON_CASES) {
          expect(
            shapeMismatch({ actual: modelMerge({ values: jsonCase.inputs, },), expected: jsonCase.expected, path: jsonCase.name, },),
          )
            .toBe(NO_MISMATCH,);
        }
      },
    },),
    it({
      name: 'build under test matches every JSON corpus case',
      fn: async () => {
        for (const jsonCase of JSON_CASES) {
          expect(
            shapeMismatch({ actual: target.deepmerge(...jsonCase.inputs,), expected: jsonCase.expected, path: jsonCase.name, },),
          )
            .toBe(NO_MISMATCH,);
        }
      },
    },),
    it({
      name: 'undefined is filtered, and an all-undefined position stays undefined',
      fn: async () => {
        expect(modelMerge({ values: [{ a: 1, }, { a: undefined, b: undefined, },], },),).toEqual({ a: 1, b: undefined, },);
        expect(modelMerge({ values: [undefined, undefined,], },),).toBeUndefined();
      },
    },),
    it({
      name: 'Sets union and Maps merge per key',
      fn: async () => {
        expect(modelMerge({ values: [new Set([1, 2,],), new Set([2, 3,],),], },),).toEqual(new Set([1, 2, 3,],),);
        expect(
          modelMerge({ values: [new Map([['k', { a: 1, },],],), new Map([['k', { b: 2, },],],),], },),
        )
          .toEqual(new Map([['k', { a: 1, b: 2, },],],),);
      },
    },),
    it({
      name: 'maxDepth stops merging and the last value wins',
      fn: async () => {
        /**
         Later nested record that must win outright below the limit.
         */
        const later = { y: 2, };
        /**
         Merge stopped at depth 1, so `r` is not merged.
         */
        const merged = modelMerge({ values: [{ r: { x: 1, }, }, { r: later, },], maxDepth: 1, },);
        expect(Reflect.get(merged as object, 'r',),).toBe(later,);
      },
    },),
    it({
      name: 'a single value and a mismatched last value keep their identity',
      fn: async () => {
        /**
         Record passed through untouched.
         */
        const only = { a: 1, };
        expect(modelMerge({ values: [only,], },),).toBe(only,);
        expect(modelMerge({ values: [[1,], only,], },),).toBe(only,);
      },
    },),
    it({
      name: '__proto__ becomes an own key, never the prototype',
      fn: async () => {
        /**
         Input whose own `__proto__` key must survive as data.
         */
        const input: unknown = JSON.parse('{"__proto__":{"polluted":true}}',);
        /**
         Merged result carrying the own key.
         */
        const merged = modelMerge({ values: [input, { a: 1, },], },) as object;
        expect(Object.getPrototypeOf(merged,),).toBe(Object.prototype,);
        expect(Object.hasOwn(merged, '__proto__',),).toBe(true,);
      },
    },),
    it({
      name: 'kindOf classifies by prototype',
      fn: async () => {
        expect(
          kindOf(Object.create(null,),),
        ).toBe('record',);
        expect(kindOf({},),).toBe('record',);
        expect(kindOf([],),).toBe('array',);
        expect(
          kindOf(new Set(),),
        ).toBe('set',);
        expect(
          kindOf(new Map(),),
        ).toBe('map',);
        expect(
          kindOf(new Date(),),
        ).toBe('other',);
        expect(kindOf(null,),).toBe('other',);
      },
    },),
  ],
},);
