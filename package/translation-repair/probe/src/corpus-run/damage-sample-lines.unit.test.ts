/**
 Tests for the sentences the damage command says about its pool and its sheet.

 EACH COUNT SHOWS ITS NOUN IN THE FORM IT ASKS FOR, and the verb of the clause
 about rows left out follows the count: one row is not drawn from where two
 rows are not. The counts are handed in directly, since a pool holding a row
 with no incumbent wording is one a fixture artifact cannot cheaply be built
 to produce.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  emptyPoolSays,
  poolLines,
  wroteLine,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'damage-sample-lines',
  children: [
    describe({
      name: poolLines.name,
      children: [
        it({
          name: 'SAYS one region and one row left out in the singular, with the verb that agrees',
          fn: async () => {
            expect(poolLines({
              regionCount: 1,
              filledWithoutIncumbent: 1,
              seed: 'damage-round-one',
            },),).toEqual([
              'DAMAGE pool 1 shipped region across both lanes, seed damage-round-one',
              'DAMAGE 1 shipped row had no incumbent wording and is not drawn from',
            ],);
          },
        },),
        it({
          name: 'SAYS several regions and several rows left out in the plural',
          fn: async () => {
            expect(poolLines({
              regionCount: 3,
              filledWithoutIncumbent: 2,
              seed: 'tabby-round',
            },),).toEqual([
              'DAMAGE pool 3 shipped regions across both lanes, seed tabby-round',
              'DAMAGE 2 shipped rows had no incumbent wording and are not drawn from',
            ],);
          },
        },),
        it({
          name: 'SAYS no rows left out in the plural',
          fn: async () => {
            expect(poolLines({
              regionCount: 2,
              filledWithoutIncumbent: 0,
              seed: 'damage-round-one',
            },)[1],).toBe('DAMAGE 0 shipped rows had no incumbent wording and are not drawn from',);
          },
        },),
      ],
    },),
    describe({
      name: emptyPoolSays.name,
      children: [
        it({
          name: 'REFUSES in words that count the rows left out, one in the singular',
          fn: async () => {
            expect(emptyPoolSays({ filledWithoutIncumbent: 1, },),).toBe(
              'the settled entries ship no replacement over an archive wording to draw from (1 shipped row had no '
                + 'incumbent wording and is not drawn from), so no sheet is written; a sheet with no item would be '
                + 'kept and refuse the next run',
            );
          },
        },),
        it({
          name: 'REFUSES in words that count the rows left out, several in the plural',
          fn: async () => {
            expect(emptyPoolSays({ filledWithoutIncumbent: 4, },),).toBe(
              'the settled entries ship no replacement over an archive wording to draw from (4 shipped rows had no '
                + 'incumbent wording and are not drawn from), so no sheet is written; a sheet with no item would be '
                + 'kept and refuse the next run',
            );
          },
        },),
      ],
    },),
    describe({
      name: wroteLine.name,
      children: [
        it({
          name: 'SAYS one item in the singular and names the sheet path',
          fn: async () => {
            expect(wroteLine({
              items: 1,
              runsDir: '/cats/runs',
            },),).toBe('DAMAGE wrote 1 item to /cats/runs/damage-sheet.md',);
          },
        },),
        it({
          name: 'SAYS several items in the plural',
          fn: async () => {
            expect(wroteLine({
              items: 20,
              runsDir: '/cats/runs',
            },),).toBe('DAMAGE wrote 20 items to /cats/runs/damage-sheet.md',);
          },
        },),
      ],
    },),
  ],
},);
