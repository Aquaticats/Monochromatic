/**
 Tests for `drained`, the loop the walks of the schema check take their next item with: it takes the last item first,
 takes the items pushed while it runs, and leaves the stack empty.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { drained, } from './claude-schema-drain.test-fixture.ts';

await describe({
  name: 'the work stack of the schema check',
  concurrency: 1,
  children: [
    it({
      name: 'TAKES THE ITEMS OF A STACK FROM THE END, last pushed first, and leaves the stack empty',
      fn: async () => {
        /**
         Items still to follow.
         */
        const stack = [
          'a',
          'b',
          'c',
        ];
        expect([...drained({ stack, },),],).toEqual([
          'c',
          'b',
          'a',
        ],);
        expect(stack,).toEqual([],);
      },
    },),
    it({
      name: 'TAKES AN ITEM PUSHED WHILE THE LOOP RUNS BEFORE THE ITEMS PUSHED BEFORE IT, as a stack gives them',
      fn: async () => {
        /**
         Items still to follow.
         */
        const stack = [
          1,
          2,
        ];
        /**
         Items in the order they were taken.
         */
        const taken: number[] = [];
        for (const item of drained({ stack, },)) {
          taken.push(item,);
          if (item === 2) {
            stack.push(
              10,
              20,
            );
          }
        }
        expect(taken,).toEqual([
          2,
          20,
          10,
          1,
        ],);
      },
    },),
    it({
      name: 'TAKES NOTHING FROM AN EMPTY STACK',
      fn: async () => {
        expect([...drained({ stack: [], },),],).toEqual([],);
      },
    },),
  ],
},);
