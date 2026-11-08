/**
 Reading fields whose shapes another program decides.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  fieldsOf,
  isRecord,
  recordsIn,
  textOf,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'reading fields of Pi records',
  children: [
    it({
      name: `${isRecord.name} accepts an object and refuses null, a list and a primitive`,
      fn: async () => {
        expect(isRecord({},),).toBe(true,);
        expect(isRecord(null,),).toBe(false,);
        expect(isRecord([],),).toBe(false,);
        expect(isRecord('text',),).toBe(false,);
      },
    },),
    it({
      name: `${fieldsOf.name} gives the object, or an empty one for anything else`,
      fn: async () => {
        expect(fieldsOf({ a: 1, },),).toEqual({ a: 1, },);
        expect(fieldsOf('text',),).toEqual({},);
      },
    },),
    it({
      name: `${recordsIn.name} keeps the objects of a list and gives nothing for anything else`,
      fn: async () => {
        expect(recordsIn([{ a: 1, }, 'text', null,],),).toEqual([{ a: 1, },],);
        expect(recordsIn('text',),).toEqual([],);
      },
    },),
    it({
      name: `${textOf.name} gives text, or nothing for anything else`,
      fn: async () => {
        expect(textOf('text',),).toBe('text',);
        expect(textOf({},),).toBe('',);
      },
    },),
  ],
},);
