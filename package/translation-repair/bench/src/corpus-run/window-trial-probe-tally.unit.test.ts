/**
 Tests for what the window trial's walk has bought and refused, and the run of
 refusals that ends it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  NOTHING_BOUGHT,
  tallyRefusal,
  tallyRows,
} from '../../dist/final/node/index.mjs';
import { armRow, } from './window-trial-probe-rows.test-fixture.ts';

await describe({
  name: 'window trial tally',
  concurrency: 1,
  children: [
    describe({
      name: tallyRefusal.name,
      children: [
        it({
          name: 'COUNTS a refusal and extends the streak by one',
          fn: async () => {
            expect(tallyRefusal({ tally: {
              count: 2,
              refused: 1,
              refusedInARow: 1,
            }, },),).toEqual({
              count: 2,
              refused: 2,
              refusedInARow: 2,
            },);
          },
        },),
        it({
          name: 'STOPS the run with a stated refusal when the fifth refusal in a row arrives, naming the five',
          fn: async () => {
            /**
             What the fifth refusal threw.
             */
            const refusal = caught(function act(): unknown {
              return tallyRefusal({ tally: {
                count: 0,
                refused: 4,
                refusedInARow: 4,
              }, },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: 5 slices refused in a row, which is a fault in the run rather than in the slices; '
                + 'stopping before the rest of the draw is spent producing slates nobody judges',
            );
          },
        },),
        it({
          name: 'DOES NOT STOP at the fourth refusal in a row',
          fn: async () => {
            expect(tallyRefusal({ tally: {
              count: 0,
              refused: 3,
              refusedInARow: 3,
            }, },).refusedInARow,).toBe(4,);
          },
        },),
      ],
    },),
    describe({
      name: tallyRows.name,
      children: [
        it({
          name: 'COUNTS a slice that bought arms and ends the streak',
          fn: async () => {
            expect(tallyRows({
              tally: {
                count: 1,
                refused: 3,
                refusedInARow: 3,
              },
              rows: [armRow({ arm: 'wide', shipped: true, },),],
            },),).toEqual({
              count: 2,
              refused: 3,
              refusedInARow: 0,
            },);
          },
        },),
        it({
          name: 'LEAVES the streak and the count where they were for a slice the ledger already held',
          fn: async () => {
            expect(tallyRows({
              tally: {
                count: 1,
                refused: 3,
                refusedInARow: 3,
              },
              rows: [],
            },),).toEqual({
              count: 1,
              refused: 3,
              refusedInARow: 3,
            },);
          },
        },),
        it({
          name: 'STARTS from nothing bought, nothing refused and no streak',
          fn: async () => {
            expect(NOTHING_BOUGHT,).toEqual({
              count: 0,
              refused: 0,
              refusedInARow: 0,
            },);
          },
        },),
      ],
    },),
  ],
},);
