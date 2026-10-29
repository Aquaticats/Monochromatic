/**
 Example tests for realm-independent classification (`./realm.ts`) and the
 helpers built on it: `kindOf` buckets, `snapshot` prototypes, and
 `shapeMismatch` over Sets and Maps from another realm.

 Subclass instances are built inside the second realm, so this file declares
 no class of its own.

 @module
 */

import {
  createContext,
  runInContext,
} from 'node:vm';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  kindOf,
  type ValueKind,
} from './model.ts';
import { isObjectPrototype, } from './realm.ts';
import {
  NO_MISMATCH,
  shapeMismatch,
  snapshot,
} from './shape.ts';

/**
 Second realm shared by every foreign value, so they share prototypes.
 */
const FOREIGN_REALM = createContext();

/**
 Evaluate `code` in {@link FOREIGN_REALM}.

 @param code - Script whose completion value is the foreign value.

 @returns Value whose built-in prototypes belong to the other realm.

 @example
 ```ts
 const foreignSet = foreign('new Set([1])');
 ```
 */
function foreign(code: string,): unknown {
  return runInContext(
    code,
    FOREIGN_REALM,
  );
}

/**
 Foreign values and the bucket the documented semantics assign them.
 */
const KIND_CASES: readonly {
  readonly code: string;
  readonly kind: ValueKind;
}[] = [
  { code: '({ a: 1 })', kind: 'record', },
  { code: 'Object.create(null)', kind: 'record', },
  { code: 'new Set([1])', kind: 'set', },
  { code: 'new Map([[1, 2]])', kind: 'map', },
  { code: '[1]', kind: 'array', },
  { code: 'new Date(0)', kind: 'other', },
  { code: 'Object.create(Object.create(null))', kind: 'other', },
];

/**
 Candidate prototypes and whether each is a realm's `Object.prototype`.
 */
const PROTOTYPE_CASES: readonly {
  readonly code: string;
  readonly isRoot: boolean;
}[] = [
  { code: 'Object.prototype', isRoot: true, },
  { code: 'Date.prototype', isRoot: false, },
  { code: 'Object.create(null)', isRoot: false, },
  { code: 'null', isRoot: false, },
];

/**
 Collections whose snapshot must keep the original's prototype.
 */
const PROTOTYPE_KEEPING_CASES: readonly string[] = [
  'class TaggedArray extends Array {}; TaggedArray.from([1])',
  'class TaggedSet extends Set {}; new TaggedSet([1])',
  'class TaggedMap extends Map {}; new TaggedMap([[1, { a: 1 }]])',
  'new Set([1])',
  'new Map([[1, { a: 1 }]])',
];

await describe({
  name: 'realm-independent classification',
  children: [
    it({
      name: 'kindOf buckets foreign records, Sets, and Maps like local ones',
      fn: async () => {
        /**
         Buckets in case order.
         */
        const kinds = KIND_CASES.map(function classify({ code, },) {
          return kindOf(foreign(code,),);
        },);
        expect(kinds,).toEqual(KIND_CASES.map(function expectedKind({ kind, },) {
          return kind;
        },),);
      },
    },),
    it({
      name: 'isObjectPrototype accepts only a realm root with its own isPrototypeOf',
      fn: async () => {
        /**
         Local and foreign verdicts in case order.
         */
        const verdicts = PROTOTYPE_CASES.flatMap(function judge({ code, },) {
          return [
            isObjectPrototype(foreign(code,),),
            isObjectPrototype(runInContext(
              code,
              createContext(),
            ),),
          ];
        },);
        expect(verdicts,).toEqual(PROTOTYPE_CASES.flatMap(function expectedVerdict({ isRoot, },) {
          return [
            isRoot,
            isRoot,
          ];
        },),);
      },
    },),
    it({
      name: 'snapshot keeps subclass and foreign prototypes of Sets, Maps, and arrays',
      fn: async () => {
        for (const code of PROTOTYPE_KEEPING_CASES) {
          /**
           Original collection from the second realm.
           */
          const original = foreign(code,);
          /**
           Copy under test.
           */
          const copy = snapshot(original,);
          expect(copy,).not.toBe(original,);
          expect(Object.getPrototypeOf(copy,),).toBe(Object.getPrototypeOf(original,),);
          expect(shapeMismatch({ actual: copy, expected: original, },),).toBe(NO_MISMATCH,);
        }
      },
    },),
    it({
      name: 'shapeMismatch compares foreign Set elements and Map values',
      fn: async () => {
        expect(shapeMismatch({ actual: foreign('new Set([1])',), expected: foreign('new Set([2])',), },),)
          .toBe('$: Set elements or order differ',);
        expect(
          shapeMismatch({ actual: foreign('new Map([[1, 1]])',), expected: foreign('new Map([[1, 2]])',), },),
        )
          .not.toBe(NO_MISMATCH,);
      },
    },),
  ],
},);
