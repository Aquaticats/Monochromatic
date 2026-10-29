/**
 Tests for what the meter report says of one provider across the record.

 AN AVAILABILITY IS OVER THE READINGS THAT ANSWERED. A meter that could not be
 read says nothing about budget, so a provider whose every reading was
 unreadable has no measurable fraction, and the report says so instead of
 printing zero.

 A SAMPLE THAT PREDATES A PROVIDER CONTRIBUTES NO READING for it, so a record
 older than a provider reports it with nothing counted.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { reportProvider, } from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  BASE_AT,
  LEVEL_NOT_RECORDED,
  sampleOf,
} from './meter-report.test-fixture.ts';

await describe({
  name: reportProvider.name,
  concurrency: 1,
  children: [
    it({
      name: 'PRINTS the counts, the share that answered wet, the longest outage and the levels of a provider',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        reportProvider({
          samples: [
            sampleOf({
              overrides: {
                at: BASE_AT,
                hyper: 'wet',
                levels: ['hyperBalance=2497',],
              },
            },),
            sampleOf({
              overrides: {
                at: BASE_AT + 1_800_000,
                hyper: 'dry',
                levels: ['hyperBalance=0',],
              },
            },),
            sampleOf({
              overrides: {
                at: BASE_AT + 3_600_000,
                hyper: 'wet',
                levels: ['hyperBalance=500',],
              },
            },),
            sampleOf({
              overrides: {
                at: BASE_AT + 5_400_000,
                hyper: 'unreadable',
                levels: [],
              },
            },),
          ],
          provider: 'hyper',
        },);
        expect(printed.lines,).toEqual([
          '\nhyper: wet=2 dry=1 unreadable=1',
          '  spendable on 66.7% of readings that answered (2 of 3)',
          '  longest outage: at least 0s, at most 1h (2026-08-25T10:30:00.000Z .. 2026-08-25T10:30:00.000Z)',
          '  level first 2026-08-25T10:00:00.000Z: hyperBalance=2497',
          '  level last 2026-08-25T11:00:00.000Z: hyperBalance=500',
        ],);
      },
    },),

    it({
      name: 'SAYS NO MEASURABLE FRACTION and no outage for a provider whose every reading was unreadable',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        reportProvider({
          samples: [
            sampleOf({ overrides: { bedrock: 'unreadable', }, },),
          ],
          provider: 'bedrock',
        },);
        expect(printed.lines,).toEqual([
          '\nbedrock: wet=0 dry=0 unreadable=1',
          '  spendable on NO MEASURABLE FRACTION: no reading in this record answered',
          '  longest outage: none, no reading found this provider out',
          LEVEL_NOT_RECORDED,
        ],);
      },
    },),

    it({
      name: 'COUNTS nothing for a provider the samples predate, rather than a reading in a state it never had',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        reportProvider({
          samples: [
            sampleOf({ overrides: { openrouter: 'absent', }, },),
          ],
          provider: 'openrouter',
        },);
        expect(printed.lines,).toEqual([
          '\nopenrouter: wet=0 dry=0 unreadable=0',
          '  spendable on NO MEASURABLE FRACTION: no reading in this record answered',
          '  longest outage: none, no reading found this provider out',
          LEVEL_NOT_RECORDED,
        ],);
      },
    },),

    it({
      name: 'PRINTS a provider that was never out as spendable on every reading that answered',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        reportProvider({
          samples: [
            sampleOf({ overrides: {}, },),
          ],
          provider: 'synthetic',
        },);
        expect(printed.lines,).toEqual([
          '\nsynthetic: wet=1 dry=0 unreadable=0',
          '  spendable on 100.0% of readings that answered (1 of 1)',
          '  longest outage: none, no reading found this provider out',
          LEVEL_NOT_RECORDED,
        ],);
      },
    },),
  ],
},);
