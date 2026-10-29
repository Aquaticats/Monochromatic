/**
 Example tests for `./recall-edit.ts`: edits land exactly where the ledger
 says or throw, failing test names come out of real `module-test` line
 shapes, and attribution subtracts the fixed build's failures.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  applyEdits,
  attributable,
  failingTests,
  RecallEditError,
} from './recall-edit.ts';

await describe({
  name: 'recall edits and attribution',
  children: [
    it({
      name: 'an edit replaces every occurrence when the count matches, in ledger order',
      fn: async () => {
        expect(applyEdits({
          edits: [
            {
              count: 2,
              file: 'x.ts',
              find: 'a',
              replace: 'b',
            },
            {
              count: 1,
              file: 'x.ts',
              find: 'b-b',
              replace: 'c',
            },
          ],
          source: '[a-a]',
        },),).toBe('[c]',);
        expect(applyEdits({
          edits: [{
            count: 1,
            file: 'x.ts',
            find: 'b-',
            replace: '',
          },],
          source: 'b-b',
        },),).toBe('b',);
      },
    },),
    it({
      name: 'an edit whose text is missing, repeated more than stated, or empty throws',
      fn: async () => {
        /**
         Edits that must each be refused.
         */
        const refused = [
          {
            count: 1,
            file: 'x.ts',
            find: 'z',
            replace: '',
          },
          {
            count: 1,
            file: 'x.ts',
            find: 'a',
            replace: '',
          },
          {
            count: 0,
            file: 'x.ts',
            find: '',
            replace: 'q',
          },
        ];
        expect(refused.filter(function accepted(edit,) {
          try {
            applyEdits({
              edits: [edit,],
              source: 'aa',
            },);
            return true;
          } catch (error) {
            if (error instanceof RecallEditError)
              return false;
            throw error;
          }
        },),).toEqual([],);
      },
    },),
    it({
      name: 'failing test names keep the suite path and drop level, timestamp, and passing lines',
      fn: async () => {
        /**
         Output with a failing child, its suite rollup, a repeat, and a pass.
         */
        const output = [
          '[error] [2026-09-24T00:00:00.000Z] [suite] [child case] [FAIL] (3ms)',
          'Error: expected 1 to be 2',
          '[error] [2026-09-24T00:00:00.000Z] [suite] [FAIL] (4ms)',
          '[error] [2026-09-24T00:00:00.000Z] [suite] [child case] [FAIL] (3ms)',
          '[info] [2026-09-24T00:00:00.000Z] [other] [PASS] a, b (1ms)',
        ].join('\n',);
        expect(failingTests(output,),).toEqual([
          '[suite] [child case]',
          '[suite]',
        ],);
        expect(failingTests('[info] [t] [s] [PASS] x',),).toEqual([],);
      },
    },),
    it({
      name: 'attribution keeps buggy-only failures in order',
      fn: async () => {
        expect(attributable({
          buggy: [
            'c',
            'a',
            'b',
          ],
          fixed: [
            'b',
            'z',
          ],
        },),).toEqual([
          'c',
          'a',
        ],);
        expect(attributable({
          buggy: ['a',],
          fixed: ['a',],
        },),).toEqual([],);
      },
    },),
  ],
},);
