/**
 Unit tests for closing containers: folding a comment found before the closing
 bracket onto the last element or entry, or onto the empty container node, and
 leaving elements and entries otherwise untouched.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type JsoncRecordEntry,
  type JsoncValue,
  closeArray,
  closeRecord,
} from '../dist/final/neutral/index.mjs';

const num = (value: number,): JsoncValue => ({ kind: 'number', value, });

const entry = (key: string, value: number,): JsoncRecordEntry => ({ key: { value: key, }, value: num(value,), });

await describe({
  name: 'parse-close',
  children: [
    describe({
      name: closeArray.name,
      children: [
        it({
          name: 'leaves a non-empty array unchanged when there is no dangling comment',
          fn: async () => {
            expect(closeArray({ elements: [num(1,), num(2,),], dangling: [], },),).toEqual({
              kind: 'array',
              elements: [num(1,), num(2,),],
            },);
          },
        },),
        it({
          name: 'folds a dangling comment onto the last element only, preserving count',
          fn: async () => {
            expect(closeArray({
              elements: [num(1,), num(2,), num(3,),],
              dangling: [{ type: 'block', text: ' d ', },],
            },),).toEqual({
              kind: 'array',
              elements: [
                num(1,),
                num(2,),
                { kind: 'number', value: 3, comment: { type: 'block', text: ' d ', }, },
              ],
            },);
          },
        },),
        it({
          name: 'attaches a dangling comment to the empty array node itself',
          fn: async () => {
            expect(closeArray({ elements: [], dangling: [{ type: 'inline', text: ' x', },], },),).toEqual({
              kind: 'array',
              elements: [],
              comment: { type: 'inline', text: ' x', },
            },);
          },
        },),
        it({
          name: 'leaves an empty array bare when there is no dangling comment',
          fn: async () => {
            expect(closeArray({ elements: [], dangling: [], },),).toEqual({ kind: 'array', elements: [], },);
          },
        },),
      ],
    },),
    describe({
      name: closeRecord.name,
      children: [
        it({
          name: 'leaves a non-empty record unchanged when there is no dangling comment',
          fn: async () => {
            expect(closeRecord({ entries: [entry('a', 1,), entry('b', 2,),], dangling: [], },),).toEqual({
              kind: 'record',
              entries: [entry('a', 1,), entry('b', 2,),],
            },);
          },
        },),
        it({
          name: 'folds a dangling comment onto the last entry value only, preserving count',
          fn: async () => {
            expect(closeRecord({
              entries: [entry('a', 1,), entry('b', 2,), entry('c', 3,),],
              dangling: [{ type: 'inline', text: ' d', },],
            },),).toEqual({
              kind: 'record',
              entries: [
                entry('a', 1,),
                entry('b', 2,),
                {
                  key: { value: 'c', },
                  value: { kind: 'number', value: 3, comment: { type: 'inline', text: ' d', }, },
                },
              ],
            },);
          },
        },),
        it({
          name: 'attaches a dangling comment to the empty record node itself',
          fn: async () => {
            expect(closeRecord({ entries: [], dangling: [{ type: 'block', text: ' only ', },], },),).toEqual({
              kind: 'record',
              entries: [],
              comment: { type: 'block', text: ' only ', },
            },);
          },
        },),
        it({
          name: 'leaves an empty record bare when there is no dangling comment',
          fn: async () => {
            expect(closeRecord({ entries: [], dangling: [], },),).toEqual({ kind: 'record', entries: [], },);
          },
        },),
      ],
    },),
  ],
},);
