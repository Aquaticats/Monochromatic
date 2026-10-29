/**
 Tests for the census lines about sections the aligner would not pair.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { sliceCensusUnpairedLines, } from '../../dist/final/node/index.mjs';
import { censusRowOf, } from './slice-census-row.test-fixture.ts';

await describe({
  name: sliceCensusUnpairedLines.name,
  children: [
    it({
      name: 'PRINTS ZERO TOTALS and no entry line where no row carries an unpaired section',
      fn: async () => {
        expect(sliceCensusUnpairedLines({
          rows: [
            censusRowOf({
              entryId: 'mochi',
              measures: {
                sliceSourceChars: [10,],
                sliceTargetChars: [30,],
              },
            },),
          ],
        },),).toEqual([
          'CENSUS unpaired sections reaching no slice: source 0, target 0; entries: 0; chars: source 0, target 0',
        ],);
      },
    },),
    it({
      name: 'PRINTS ZERO TOTALS for no rows',
      fn: async () => {
        expect(sliceCensusUnpairedLines({ rows: [], },),).toEqual([
          'CENSUS unpaired sections reaching no slice: source 0, target 0; entries: 0; chars: source 0, target 0',
        ],);
      },
    },),
    it({
      name: 'COUNTS AN ENTRY WITH ONLY SOURCE SECTIONS unpaired, and one with only target sections unpaired',
      fn: async () => {
        expect(sliceCensusUnpairedLines({
          rows: [
            censusRowOf({
              entryId: 'mochi',
              measures: {
                unpairedSourceSections: 2,
                unpairedSourceChars: 65,
              },
            },),
            censusRowOf({
              entryId: 'nori',
              measures: {
                unpairedTargetSections: 1,
                unpairedTargetChars: 47,
              },
            },),
            censusRowOf({ entryId: 'tama', },),
          ],
        },),).toEqual([
          'CENSUS unpaired sections reaching no slice: source 2, target 1; entries: 2; chars: source 65, target 47',
          'CENSUS   mochi: source sections 2 (chars: 65), target sections 0 (chars: 0)',
          'CENSUS   nori: source sections 0 (chars: 0), target sections 1 (chars: 47)',
        ],);
      },
    },),
    it({
      name: 'NAMES ONLY THE FIVE ENTRIES holding the most unpaired characters, in that order, while the totals '
        + 'count all six and a closing line says one entry was left out',
      fn: async () => {
        expect(sliceCensusUnpairedLines({
          rows: [
            censusRowOf({
              entryId: 'mochi',
              measures: {
                unpairedSourceSections: 1,
                unpairedSourceChars: 10,
              },
            },),
            censusRowOf({
              entryId: 'nori',
              measures: {
                unpairedSourceSections: 1,
                unpairedSourceChars: 60,
              },
            },),
            censusRowOf({
              entryId: 'tama',
              measures: {
                unpairedSourceSections: 1,
                unpairedSourceChars: 30,
              },
            },),
            censusRowOf({
              entryId: 'yuzu',
              measures: {
                unpairedSourceSections: 1,
                unpairedSourceChars: 50,
              },
            },),
            censusRowOf({
              entryId: 'kuro',
              measures: {
                unpairedSourceSections: 1,
                unpairedSourceChars: 20,
              },
            },),
            censusRowOf({
              entryId: 'sora',
              measures: {
                unpairedSourceSections: 1,
                unpairedSourceChars: 40,
              },
            },),
          ],
        },),).toEqual([
          'CENSUS unpaired sections reaching no slice: source 6, target 0; entries: 6; chars: source 210, target 0',
          'CENSUS   nori: source sections 1 (chars: 60), target sections 0 (chars: 0)',
          'CENSUS   yuzu: source sections 1 (chars: 50), target sections 0 (chars: 0)',
          'CENSUS   sora: source sections 1 (chars: 40), target sections 0 (chars: 0)',
          'CENSUS   tama: source sections 1 (chars: 30), target sections 0 (chars: 0)',
          'CENSUS   kuro: source sections 1 (chars: 20), target sections 0 (chars: 0)',
          'CENSUS   and 1 more entry with unpaired text, left out of this list',
        ],);
      },
    },),
    it({
      name: 'RANKS BY THE UNPAIRED TEXT OF BOTH SIDES TOGETHER, so an entry whose unpaired text is all on the '
        + 'target side is listed by its size, and says how many entries the list left out',
      fn: async () => {
        expect(sliceCensusUnpairedLines({
          rows: [
            censusRowOf({
              entryId: 'mochi',
              measures: {
                unpairedSourceSections: 1,
                unpairedSourceChars: 10,
              },
            },),
            censusRowOf({
              entryId: 'tama',
              measures: {
                unpairedSourceSections: 1,
                unpairedSourceChars: 30,
              },
            },),
            censusRowOf({
              entryId: 'yuzu',
              measures: {
                unpairedSourceSections: 1,
                unpairedSourceChars: 50,
              },
            },),
            censusRowOf({
              entryId: 'kuro',
              measures: {
                unpairedSourceSections: 1,
                unpairedSourceChars: 20,
              },
            },),
            censusRowOf({
              entryId: 'sora',
              measures: {
                unpairedSourceSections: 1,
                unpairedSourceChars: 15,
                unpairedTargetSections: 1,
                unpairedTargetChars: 25,
              },
            },),
            censusRowOf({
              entryId: 'nori',
              measures: {
                unpairedTargetSections: 2,
                unpairedTargetChars: 90,
              },
            },),
            censusRowOf({
              entryId: 'momo',
              measures: {
                unpairedSourceSections: 1,
                unpairedSourceChars: 5,
              },
            },),
          ],
        },),).toEqual([
          'CENSUS unpaired sections reaching no slice: source 6, target 3; entries: 7; chars: source 130, target 115',
          'CENSUS   nori: source sections 0 (chars: 0), target sections 2 (chars: 90)',
          'CENSUS   yuzu: source sections 1 (chars: 50), target sections 0 (chars: 0)',
          'CENSUS   sora: source sections 1 (chars: 15), target sections 1 (chars: 25)',
          'CENSUS   tama: source sections 1 (chars: 30), target sections 0 (chars: 0)',
          'CENSUS   kuro: source sections 1 (chars: 20), target sections 0 (chars: 0)',
          'CENSUS   and 2 more entries with unpaired text, left out of this list',
        ],);
      },
    },),
    it({
      name: 'BREAKS A TIE between entries holding as much unpaired text by entry id in code point order, an id '
        + 'beyond the first plane after one inside it that UTF-16 order would put after it, and prints no closing '
        + 'line when the list leaves nothing out',
      fn: async () => {
        expect(sliceCensusUnpairedLines({
          rows: [
            censusRowOf({
              entryId: '\u{1F63A}',
              measures: {
                unpairedSourceSections: 1,
                unpairedSourceChars: 10,
                unpairedTargetSections: 1,
                unpairedTargetChars: 20,
              },
            },),
            censusRowOf({
              entryId: 'mittens',
              measures: {
                unpairedSourceSections: 1,
                unpairedSourceChars: 30,
              },
            },),
            censusRowOf({
              entryId: '\u{FF4D}',
              measures: {
                unpairedTargetSections: 1,
                unpairedTargetChars: 30,
              },
            },),
            censusRowOf({
              entryId: 'Tabby',
              measures: {
                unpairedTargetSections: 1,
                unpairedTargetChars: 30,
              },
            },),
          ],
        },),).toEqual([
          'CENSUS unpaired sections reaching no slice: source 2, target 3; entries: 4; chars: source 40, target 80',
          'CENSUS   Tabby: source sections 0 (chars: 0), target sections 1 (chars: 30)',
          'CENSUS   mittens: source sections 1 (chars: 30), target sections 0 (chars: 0)',
          'CENSUS   \u{FF4D}: source sections 0 (chars: 0), target sections 1 (chars: 30)',
          'CENSUS   \u{1F63A}: source sections 1 (chars: 10), target sections 1 (chars: 20)',
        ],);
      },
    },),
  ],
},);
