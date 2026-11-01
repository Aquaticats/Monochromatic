/**
 Upstream documentation examples, run against the build under test.

 Each case copies a README.md or docs/deepmergeCustom.md example's inputs and
 options and asserts the result the documentation prints, so a behaviour
 change that breaks a documented example turns this file red. The examples'
 type annotations are adapted where the verbatim snippet does not type-check
 (`./surface-docs-type.unit.test.ts` pins those failures); the runtime calls
 are the documented ones.

 Checked against deepmerge-ts 8.0.2 (docs at upstream `17fc99cb`), 2026-09-24.

 @module
 */

import type { DeepMergeLeafURI, } from 'deepmerge-ts';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { target, } from './target.ts';

/**
 Whether every value is a number, as the docs' `areAllNumbers` helper.

 @param values - Values being merged.

 @returns True when each value is a number.

 @example
 ```ts
 areAllNumbers([1, 2,],); // true
 ```
 */
function areAllNumbers(values: readonly unknown[],): values is readonly number[] {
  return values.every(function isNumber(value,) {
    return (typeof value) === 'number';
  },);
}

await describe({
  name: 'deepmerge-ts documentation examples',
  children: [
    it({
      name: 'README "Example using default config" returns the printed result',
      fn: async () => {
        /**
         Result of the README's three-input call.
         */
        const merged = target.deepmerge(
          {
            array: [1, 2, 3,],
            map: new Map([['key1', 'value1',], ['key2', 'value2',],],),
            record: { prop1: 'value1', prop2: 'value2', },
            set: new Set([1, 2, 3,],),
          },
          {
            array: [2, 3, 4,],
            map: new Map([['key2', 'changed',], ['key3', 'value3',],],),
            record: { prop1: 'changed', prop3: 'value3', },
            set: new Set([2, 3, 4,],),
          },
          {
            array: undefined,
            map: undefined,
            record: { prop1: undefined, prop2: undefined, prop3: undefined, prop4: undefined, },
            set: undefined,
          },
        );
        expect(merged.record,).toEqual({ prop1: 'changed', prop2: 'value2', prop3: 'value3', prop4: undefined, },);
        expect(Object.keys(merged.record,),).toEqual(['prop1', 'prop2', 'prop3', 'prop4',],);
        expect(merged.array,).toEqual([1, 2, 3, 2, 3, 4,],);
        expect([...merged.set,],).toEqual([1, 2, 3, 4,],);
        expect([...merged.map,],).toEqual([['key1', 'value1',], ['key2', 'changed',], ['key3', 'value3',],],);
      },
    },),
    it({
      name: 'deepmergeCustom.md option examples return their documented results',
      fn: async () => {
        expect(target.deepmergeCustom({ mergeArrays: false, },)({ bar: [3, 4,], foo: [1, 2,], }, { foo: [5, 6,], },),)
          .toEqual({ bar: [3, 4,], foo: [5, 6,], },);
        expect(target.deepmergeCustom({ filterValues: false, },)({ key1: { subkey1: 'one', }, }, { key1: undefined, },),)
          .toEqual({ key1: undefined, },);
        /**
         Filter that drops `null` instead of `undefined`.
         */
        const dropNull = target.deepmergeCustom({
          filterValues: function withoutNull(values,) {
            return values.filter(function kept(value,) {
              return value !== null;
            },);
          },
        },);
        expect(dropNull({ key1: { subkey1: 'one', }, }, { key1: null, }, { key1: { subkey2: 'two', }, },),)
          .toEqual({ key1: { subkey1: 'one', subkey2: 'two', }, },);
        /**
         Dates amalgamated into the values array.
         */
        const dates = [new Date('2020-01-01',), new Date('2021-02-02',), new Date('2022-03-03',),] as const;
        /**
         mergeOthers returning the values when all are dates.
         */
        const amalgamate = target.deepmergeCustom({
          mergeOthers: function datesToArray(values, utils,) {
            if (values.every(function isDate(value,) {
              return value instanceof Date;
            },))
              return values;
            return utils.defaultMergeFunctions.mergeOthers(values,);
          },
        },);
        expect(amalgamate({ foo: dates[0], }, { foo: dates[1], }, { foo: dates[2], },),).toEqual({ foo: [...dates,], },);
      },
    },),
    it({
      name: 'deepmergeCustom.md meta data examples return their documented results',
      fn: async () => {
        /**
         Key-based numeric merge from "Meta Data".
         */
        const byKey = target.deepmergeCustom({
          mergeOthers: function numbersByKey(values, utils, meta,) {
            if ((meta !== undefined) && areAllNumbers(values,)) {
              /**
               The narrowed values, re-typed as the docs do.
               */
              const numbers: readonly number[] = values;
              if (meta.key === 'sum')
                return numbers.reduce(function add(sum, value,) { return sum + value; },);
              if (meta.key === 'product')
                return numbers.reduce(function multiply(product, value,) { return product * value; },);
              if (meta.key === 'mean')
                return numbers.reduce(function add(sum, value,) { return sum + value; },) / numbers.length;
            }
            return utils.defaultMergeFunctions.mergeOthers(values,);
          },
        },);
        expect(byKey(
          { mean: 3, product: 2, sum: 1, },
          { mean: 6, product: 5, sum: 4, },
          { mean: 9, product: 8, sum: 7, },
          { mean: 12, product: 11, sum: 10, },
        ),).toEqual({ mean: 7.5, product: 880, sum: 22, },);
        /**
         Key-path metadata from "Customizing the Meta Data", with the key typed `unknown`.
         */
        const byPath = target.deepmergeCustom<unknown, { DeepMergeOthersURI: DeepMergeLeafURI; }, { readonly keyPath: readonly unknown[]; }>({
          mergeOthers: function special(values, utils, meta,) {
            if ((meta !== undefined) && (meta.keyPath.at(-2,) === 'bar') && (meta.keyPath.at(-1,) === 'baz'))
              return 'special merge';
            return utils.defaultMergeFunctions.mergeOthers(values,);
          },
          metaDataUpdater: function keyPath(previousMeta, mergeInfo,) {
            if (previousMeta === undefined)
              return { keyPath: (mergeInfo.key === undefined) ? [] : [mergeInfo.key,], };
            if (mergeInfo.key === undefined)
              return previousMeta;
            return { ...previousMeta, keyPath: [...previousMeta.keyPath, mergeInfo.key,], };
          },
        },);
        expect(byPath(
          { bar: { baz: 3, qux: 4, }, foo: { bar: { baz: 1, qux: 2, }, }, },
          { bar: { baz: 8, qux: 9, }, foo: { bar: { bar: { baz: 6, qux: 7, }, baz: 5, }, }, },
        ),).toEqual({
          bar: { baz: 'special merge', qux: 9, },
          foo: { bar: { bar: { baz: 6, qux: 7, }, baz: 'special merge', qux: 2, }, },
        },);
      },
    },),
    it({
      name: 'deepmergeCustom.md "Skipping a Property" drops all-Date skipme keys and keeps the non-Date values',
      fn: async () => {
        /**
         The documented skipme merge function.
         */
        const skipDates = target.deepmergeCustom({
          mergeOthers: function skipme(values, utils, meta,) {
            if (meta?.key === 'skipme') {
              /**
               Values that are not dates.
               */
              const nonDateValues = values.filter(function notDate(value,) {
                return !(value instanceof Date);
              },);
              if (nonDateValues.length === 0)
                return utils.actions.skip;
              return utils.defaultMergeFunctions.mergeOthers(nonDateValues,);
            }
            return utils.actions.defaultMerge;
          },
        },);
        expect(skipDates({ k: 1, skipme: new Date(1,), }, { k: 2, skipme: new Date(2,), },),).toEqual({ k: 2, },);
        expect(skipDates({ skipme: new Date(1,), }, { skipme: 5, }, { skipme: new Date(2,), },),).toEqual({ skipme: 5, },);
      },
    },),
  ],
},);
