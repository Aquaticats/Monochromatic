/**
 Tests for reading the definitions the roster paired and the paired slices
 as one body of evidence (class ninety-five). Cat-themed invention; no
 corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { widenFootnoteRelabel, } from '../dist/final/node/index.mjs';

await describe({
  name: widenFootnoteRelabel.name,
  children: [
    it({
      name: 'completes the definitions\' relations with the slices\' through one consistency check, keeping every '
        + 'skipped slice',
      fn: async () => {
        expect(widenFootnoteRelabel({
          definitions: {
            kind: 'relabel',
            map: [ {
              from: '3',
              to: '2',
            }, ],
            correspondences: [ {
              from: '3',
              to: '2',
            }, ],
            skipped: [],
          },
          slices: {
            kind: 'relabel',
            map: [ {
              from: '2',
              to: '1',
            }, ],
            correspondences: [
              {
                from: '3',
                to: '2',
              },
              {
                from: '2',
                to: '1',
              },
            ],
            skipped: [ 'slice 2 references 1 distinct note in the original and 2 in the archive', ],
          },
        },),).toStrictEqual({
          kind: 'relabel',
          map: [
            {
              from: '3',
              to: '2',
            },
            {
              from: '2',
              to: '1',
            },
          ],
          correspondences: [
            {
              from: '3',
              to: '2',
            },
            {
              from: '2',
              to: '1',
            },
          ],
          skipped: [ 'slice 2 references 1 distinct note in the original and 2 in the archive', ],
        },);
      },
    },),

    it({
      name: 'leaves the archive standing, naming the slice, where a slice contradicts a definition the roster paired',
      fn: async () => {
        /**
         The widened reading over a contradiction.
         */
        const widened = widenFootnoteRelabel({
          definitions: {
            kind: 'relabel',
            map: [ {
              from: '3',
              to: '2',
            }, ],
            correspondences: [ {
              from: '3',
              to: '2',
            }, ],
            skipped: [],
          },
          slices: {
            kind: 'relabel',
            map: [ {
              from: '3',
              to: '1',
            }, ],
            correspondences: [ {
              from: '3',
              to: '1',
            }, ],
            skipped: [],
          },
        },);
        expect(widened.kind,).toBe('ambiguous',);
        if (widened.kind !== 'ambiguous')
          throw new Error('unreachable',);
        // The earlier claim is the definition's, which this detail once called
        // "an earlier slice".
        expect(widened.detail,).toBe(
          'a paired slice maps archive [^3] to original [^1], where a definition the roster paired mapped that '
            + 'archive label to original [^2]',
        );
      },
    },),

    it({
      name: 'passes an ambiguous input reading through as it is, the definitions\' first',
      fn: async () => {
        /**
         An ambiguous reading.
         */
        const ambiguous = {
          kind: 'ambiguous',
          detail: 'invented',
        } as const;
        /**
         A reading with nothing to say.
         */
        const silent = {
          kind: 'unchanged',
          correspondences: [],
          skipped: [],
        } as const;
        expect(widenFootnoteRelabel({
          definitions: ambiguous,
          slices: silent,
        },),).toStrictEqual(ambiguous,);
        expect(widenFootnoteRelabel({
          definitions: silent,
          slices: ambiguous,
        },),).toStrictEqual(ambiguous,);
      },
    },),
  ],
},);
