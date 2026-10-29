/**
 Tests for the band lines a draw prints about its pool.

 THE LINES EXIST BECAUSE BAND TOTALS HIDE HOW LOPSIDED A BAND IS: the draw
 round-robins across entries, so a band's spread comes from how many entries
 contribute, not from how many issues they brought. Each case therefore reads
 the three counts and the per-entry shape together.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { poolBandLines, } from '../../dist/final/node/index.mjs';
import { bandedEntryOf, } from './banded-entry.test-fixture.ts';

await describe({
  name: poolBandLines.name,
  children: [
    it({
      name: 'PRINTS one line per band, in band order, with no entry anywhere for an empty pool',
      fn: async () => {
        expect(poolBandLines({ entries: [], },),).toEqual([
          'POOL band=small entries=0 contributing=0 accepted=0 perEntry=',
          'POOL band=medium entries=0 contributing=0 accepted=0 perEntry=',
          'POOL band=large entries=0 contributing=0 accepted=0 perEntry=',
        ],);
      },
    },),
    it({
      name: 'COUNTS an entry that accepts nothing as an entry of its band but not as a contributing one',
      fn: async () => {
        expect(poolBandLines({
          entries: [
            bandedEntryOf({
              id: 'mittens',
              band: 'small',
              count: 1,
            },),
            bandedEntryOf({
              id: 'tabby',
              band: 'small',
              count: 0,
            },),
          ],
        },),).toEqual([
          'POOL band=small entries=2 contributing=1 accepted=1 perEntry=mittens:1',
          'POOL band=medium entries=0 contributing=0 accepted=0 perEntry=',
          'POOL band=large entries=0 contributing=0 accepted=0 perEntry=',
        ],);
      },
    },),
    it({
      name: 'SUMS the accepted issues of a band and lists its contributing entries heaviest first, '
        + 'ties kept in the order the entries came',
      fn: async () => {
        expect(poolBandLines({
          entries: [
            bandedEntryOf({
              id: 'whiskers',
              band: 'medium',
              count: 1,
            },),
            bandedEntryOf({
              id: 'mittens',
              band: 'medium',
              count: 3,
            },),
            bandedEntryOf({
              id: 'tabby',
              band: 'medium',
              count: 1,
            },),
            bandedEntryOf({
              id: 'biscuit',
              band: 'large',
              count: 2,
            },),
          ],
        },),).toEqual([
          'POOL band=small entries=0 contributing=0 accepted=0 perEntry=',
          'POOL band=medium entries=3 contributing=3 accepted=5 perEntry=mittens:3,whiskers:1,tabby:1',
          'POOL band=large entries=1 contributing=1 accepted=2 perEntry=biscuit:2',
        ],);
      },
    },),
  ],
},);
