/**
 Tests for what the meter report says one provider's meter was reading.

 THE RECORD IS READ AT BOTH ENDS. The last reading answers what the budget is
 now, and the first says which way it moved to get there, so two readings print
 two lines and one reading prints one.

 A RECORD WITH NO NUMBERS SAYS SO. A run written before the levels were added
 carries states and no numbers, and silence there would read as a provider
 whose meter never said anything.

 EACH FIELD IS ATTRIBUTED BY THE NAME IT STARTS WITH, so a reading naming
 another provider's numbers adds nothing to this one's.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { levelLines, } from '../../dist/final/node/index.mjs';
import {
  BASE_AT,
  sampleOf,
} from './meter-report.test-fixture.ts';

/**
 What the report says when no reading carries a number for the provider.
 */
const NOT_RECORDED = '  level: NOT RECORDED. These readings predate the meter numbers being written down, so a dry one '
  + 'here cannot be told from a threshold that was wrong about a budget that was fine';

await describe({
  name: levelLines.name,
  concurrency: 1,
  children: [
    it({
      name: 'SAYS NOT RECORDED for no reading at all',
      fn: async () => {
        expect(levelLines({
          samples: [],
          provider: 'hyper',
        },),).toEqual([NOT_RECORDED,],);
      },
    },),

    it({
      name: 'SAYS NOT RECORDED where readings carry no number for this provider, even if they carry another\'s',
      fn: async () => {
        expect(levelLines({
          samples: [
            sampleOf({ overrides: {}, },),
            sampleOf({
              overrides: {
                at: BASE_AT + 60_000,
                levels: ['syntheticWeekly=12%',],
              },
            },),
          ],
          provider: 'hyper',
        },),).toEqual([NOT_RECORDED,],);
      },
    },),

    it({
      name: 'WRITES one line for one reading that named a number, with every field of that reading in order',
      fn: async () => {
        expect(levelLines({
          samples: [
            sampleOf({
              overrides: {
                levels: [
                  'syntheticWeekly=12%',
                  'hyperBalance=2497',
                  'hyperTier=basic',
                ],
              },
            },),
          ],
          provider: 'hyper',
        },),).toEqual([
          '  level 2026-08-25T10:00:00.000Z: hyperBalance=2497 hyperTier=basic',
        ],);
      },
    },),

    it({
      name: 'WRITES BOTH FORMS OF SYNTHETIC\'S FIVE-HOUR FIELDS, a run that began on the window and ended on a reply with none, verbatim',
      fn: async () => {
        expect(levelLines({
          samples: [
            sampleOf({
              overrides: {
                at: BASE_AT,
                levels: ['syntheticWeekly=97%', 'syntheticFiveHour=48/50', 'syntheticThrottled=no',],
              },
            },),
            sampleOf({
              overrides: {
                at: BASE_AT + 60_000,
                levels: ['syntheticWeekly=64.5%', 'syntheticFiveHour=none', 'syntheticThrottled=none',],
              },
            },),
          ],
          provider: 'synthetic',
        },),).toEqual([
          '  level first 2026-08-25T10:00:00.000Z: syntheticWeekly=97% syntheticFiveHour=48/50 syntheticThrottled=no',
          '  level last 2026-08-25T10:01:00.000Z: syntheticWeekly=64.5% syntheticFiveHour=none syntheticThrottled=none',
        ],);
      },
    },),

    it({
      name: 'WRITES the first and the last reading that named a number, leaving out what lies between',
      fn: async () => {
        expect(levelLines({
          samples: [
            sampleOf({
              overrides: {
                at: BASE_AT,
                levels: ['hyperBalance=2497',],
              },
            },),
            sampleOf({
              overrides: {
                at: BASE_AT + 60_000,
                levels: ['hyperBalance=2000',],
              },
            },),
            sampleOf({
              overrides: {
                at: BASE_AT + 120_000,
                levels: ['syntheticWeekly=12%',],
              },
            },),
            sampleOf({
              overrides: {
                at: BASE_AT + 180_000,
                levels: ['hyperBalance=500',],
              },
            },),
          ],
          provider: 'hyper',
        },),).toEqual([
          '  level first 2026-08-25T10:00:00.000Z: hyperBalance=2497',
          '  level last 2026-08-25T10:03:00.000Z: hyperBalance=500',
        ],);
      },
    },),

    it({
      name: 'WRITES two readings as first and last, with no reading between them',
      fn: async () => {
        expect(levelLines({
          samples: [
            sampleOf({
              overrides: {
                at: BASE_AT,
                levels: ['openrouterUsd=57.62',],
              },
            },),
            sampleOf({
              overrides: {
                at: BASE_AT + 60_000,
                levels: ['openrouterUsd=57.10',],
              },
            },),
          ],
          provider: 'openrouter',
        },),).toEqual([
          '  level first 2026-08-25T10:00:00.000Z: openrouterUsd=57.62',
          '  level last 2026-08-25T10:01:00.000Z: openrouterUsd=57.10',
        ],);
      },
    },),
  ],
},);
