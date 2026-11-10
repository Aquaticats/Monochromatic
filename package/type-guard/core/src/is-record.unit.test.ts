/**
 `isRecord` accepts every object that is neither null nor an array.
 
 The copies it replaces were spelled `value !== null && typeof value === 'object'
 && !Array.isArray(value)`, so the suite pins all three halves: the accepted set
 (plain objects, class instances, `Map`, `Date`, errors) and the rejected set
 (`null`, arrays, and every primitive, functions included, because a function is
 `typeof 'function'` and the old bodies refused it).
 */
import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isRecord, } from '../dist/final/neutral/index.mjs';

//region Fixtures

/**
 Object whose prototype is null, as `JSON.parse` never builds but parsers can.
 */
const nullPrototypeRecord: unknown = Object.create(null,);

/**
 Accepted values, each paired with the name the assertion reports.
 */
const acceptedValues: readonly { readonly name: string; readonly value: unknown }[] = [
  {
    name: 'a plain object',
    value: { field: 'value', },
  },
  {
    name: 'an empty plain object',
    value: {},
  },
  {
    name: 'an object with a null prototype',
    value: nullPrototypeRecord,
  },
  {
    name: 'a Map',
    value: new Map(),
  },
  {
    name: 'a Date',
    value: new Date(0,),
  },
  {
    name: 'an Error',
    value: new Error('failure',),
  },
];

/**
 Rejected values, each paired with the name the assertion reports.
 */
const rejectedValues: readonly { readonly name: string; readonly value: unknown }[] = [
  {
    name: 'null',
    value: null,
  },
  {
    name: 'undefined',
    value: undefined,
  },
  {
    name: 'an empty array',
    value: [],
  },
  {
    name: 'an array holding a value',
    value: [1,],
  },
  {
    name: 'a function declaration',
    value: function declaredFunction(): void {},
  },
  {
    name: 'an arrow function',
    value: () => undefined,
  },
  {
    name: 'a string',
    value: 'field',
  },
  {
    name: 'a number',
    value: 0,
  },
  {
    name: 'a bigint',
    value: 0n,
  },
  {
    name: 'a boolean',
    value: false,
  },
  {
    name: 'a symbol',
    value: Symbol('symbol value used by the rejection fixture',),
  },
];

//endregion Fixtures

await describe({
  name: isRecord.name,
  children: [
    //region Accepted values

    ...acceptedValues.map(function acceptedCase(fixture,) {
      return it({
        name: `accepts ${fixture.name}`,
        fn: async () => {
          expect(isRecord(fixture.value,),).toBe(true,);
        },
      },);
    },),

    //endregion Accepted values

    //region Rejected values

    ...rejectedValues.map(function rejectedCase(fixture,) {
      return it({
        name: `rejects ${fixture.name}`,
        fn: async () => {
          expect(isRecord(fixture.value,),).toBe(false,);
        },
      },);
    },),

    //endregion Rejected values

    //region Narrowing

    it({
      name: 'narrows an unknown value so its fields are readable',
      fn: async () => {
        /**
         Value as a caller holds it before the guard runs.
         */
        const value: unknown = { field: 'value', };
        if (!isRecord(value,)) {
          throw new Error('the fixture is a record',);
        }
        expect(value.field,).toBe('value',);
        expectTypeOf(value,).toEqualTypeOf<Readonly<Record<string, unknown>>>();
      },
    },),

    //endregion Narrowing
  ],
},);
