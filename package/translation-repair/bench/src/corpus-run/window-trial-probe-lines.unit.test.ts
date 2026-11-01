/**
 Tests for the sentences the window trial's walk tells the log.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  boughtLine,
  ledgerReadLine,
  openingLine,
  walkEndLine,
} from '../../dist/final/node/index.mjs';
import { armRow, } from './window-trial-probe-rows.test-fixture.ts';

await describe({
  name: 'window trial lines',
  concurrency: 1,
  children: [
    describe({
      name: openingLine.name,
      children: [
        it({
          name: 'NAMES the first twelve characters of the protocol and one arm in the singular',
          fn: async () => {
            expect(openingLine({
              protocol: 'abcdef0123456789',
              armsBought: 1,
            },),).toBe('protocol abcdef012345; 1 arm already bought',);
          },
        },),
        it({
          name: 'NAMES arms in the plural, zero included',
          fn: async () => {
            expect(openingLine({
              protocol: 'abcdef0123456789',
              armsBought: 0,
            },),).toBe('protocol abcdef012345; 0 arms already bought',);
            expect(openingLine({
              protocol: 'abcdef0123456789',
              armsBought: 3,
            },),).toBe('protocol abcdef012345; 3 arms already bought',);
          },
        },),
      ],
    },),
    describe({
      name: boughtLine.name,
      children: [
        it({
          name: 'SAYS for each arm bought whether it replaced the archive or kept it, in the order bought',
          fn: async () => {
            expect(boughtLine({
              entryId: 'Mittens',
              pick: {
                entryId: 'Mittens',
                sliceIndex: 1,
                sliceClass: 'untranslated',
              },
              rows: [
                armRow({ arm: 'narrow-a', shipped: false, },),
                armRow({ arm: 'wide', shipped: true, },),
                armRow({ arm: 'narrow-b', shipped: false, },),
              ],
            },),).toBe('Mittens/1 (untranslated): narrow-a=kept wide=replaced narrow-b=kept',);
          },
        },),
      ],
    },),
    describe({
      name: walkEndLine.name,
      children: [
        it({
          name: 'COUNTS one slice in the singular and any other count in the plural',
          fn: async () => {
            expect(walkEndLine({
              count: 1,
              refused: 0,
            },),).toBe('bought 1 slice this run; 0 refused',);
            expect(walkEndLine({
              count: 0,
              refused: 2,
            },),).toBe('bought 0 slices this run; 2 refused',);
            expect(walkEndLine({
              count: 2,
              refused: 1,
            },),).toBe('bought 2 slices this run; 1 refused',);
          },
        },),
      ],
    },),
    describe({
      name: ledgerReadLine.name,
      children: [
        it({
          name: 'COUNTS one row and one left-out line in the singular, and says the last line was whole',
          fn: async () => {
            expect(ledgerReadLine({
              rows: 1,
              leftOut: 1,
              tornTail: false,
            },),).toBe(
              'ledger read for this report: 1 row under every protocol; 1 whole line left out as no trial row of this '
                + 'build; last line whole',
            );
          },
        },),
        it({
          name: 'COUNTS rows and left-out lines in the plural, and says the last line was cut short by a kill',
          fn: async () => {
            expect(ledgerReadLine({
              rows: 6,
              leftOut: 0,
              tornTail: true,
            },),).toBe(
              'ledger read for this report: 6 rows under every protocol; 0 whole lines left out as no trial row of this '
                + 'build; last line cut short by a kill and not counted',
            );
          },
        },),
      ],
    },),
  ],
},);
