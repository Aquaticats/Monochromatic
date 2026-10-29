/**
 Tests for the current-vs-selected diff.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { computeChanges, } from './changes.ts';

await describe({
  name: 'changes',
  children: [
    describe({
      name: computeChanges.name,
      children: [
        it({
          name: 'no-op when selection matches current state',
          fn: async () => {
            expect(computeChanges({ all: ['a', 'b',], currentlyExempted: ['a',], selected: ['a',], },),).toEqual({
              toExempt: [],
              toRevert: [],
            },);
          },
        },),
        it({
          name: 'exempts newly checked apps only',
          fn: async () => {
            expect(
              computeChanges({ all: ['a', 'b', 'c',], currentlyExempted: ['b',], selected: ['a', 'b',], },),
            ).toEqual({ toExempt: ['a',], toRevert: [], },);
          },
        },),
        it({
          name: 'reverts unchecked previously-exempted apps only',
          fn: async () => {
            expect(
              computeChanges({ all: ['a', 'b',], currentlyExempted: ['a', 'b',], selected: ['a',], },),
            ).toEqual({ toExempt: [], toRevert: ['b',], },);
          },
        },),
        it({
          name: 'handles a mixed exempt-and-revert selection',
          fn: async () => {
            expect(
              computeChanges({ all: ['a', 'b', 'c',], currentlyExempted: ['b',], selected: ['a', 'c',], },),
            ).toEqual({ toExempt: ['a', 'c',], toRevert: ['b',], },);
          },
        },),
        it({
          name: 'ignores selected and exempted ids absent from the full list',
          fn: async () => {
            expect(
              computeChanges({ all: ['a', 'b',], currentlyExempted: ['a', 'ghost',], selected: ['a', 'zzz',], },),
            ).toEqual({ toExempt: [], toRevert: [], },);
          },
        },),
        it({
          name: 'emits results in full-list order',
          fn: async () => {
            expect(
              computeChanges({ all: ['c', 'a', 'b',], currentlyExempted: [], selected: ['a', 'b', 'c',], },),
            ).toEqual({ toExempt: ['c', 'a', 'b',], toRevert: [], },);
          },
        },),
      ],
    },),
  ],
},);
