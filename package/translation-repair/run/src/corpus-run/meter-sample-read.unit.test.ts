/**
 Tests for reading availability records back out of a log.

 THE FIXTURE LINES ARE REAL SHAPES, copied from what the console sink writes:
 `[level] [iso] [tag] [tag] message`. A test built on an invented prefix would
 pass while the reader failed on every line the pipeline actually produces.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  readMeterLine,
  readMeterLog,
} from '../../dist/final/node/index.mjs';
import { STAMPS_NOT_WRITTEN, } from '@monochromatic-dev/translation-repair-markup/ts/iso-stamp-text.test-fixture.ts';

/**
 A record exactly as `takeReading` writes one.
 */
const REAL_LINE =
  '[info] [2026-08-24T18:17:35.383Z] [translation-repair] [takeReading] '
    + 'METERS synthetic=wet hyper=dry';

/**
 The sampler's own summary, which mentions the marker in prose.

 THIS LINE IS WHY THE GATE EXISTS. Read back off a real sample, it counted
 as a record that would not parse and reported a hole in an intact log.
 */
const PROSE_LINE =
  '[info] [2026-08-24T18:17:35.383Z] [translation-repair] [sampleBudgets] '
    + 'SAMPLED: routing would use synthetic. The METERS line above is the record';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readMeterLine.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads a record the pipeline actually wrote',
          fn: async () => {
            expect(readMeterLine({ line: REAL_LINE, },),).toEqual({
              at: Date.parse('2026-08-24T18:17:35.383Z',),
              synthetic: 'wet',
              hyper: 'dry',
              neuralwatt: 'absent',
              bedrock: 'absent',
              deepseek: 'absent',
              xiaomi: 'absent',
              anthropic: 'absent',
              openrouter: 'absent',
              levels: [],
            },);
          },
        },),

        it({
          name: 'reads the third state a record carries since 2026-09-03, and keeps older two-state records '
            + 'readable with that column named absent rather than skipping every historical reading',
          fn: async () => {
            /**
             A record as `takeReading` writes one with every provider configured.
             */
            const line = '[info] [2026-09-03T17:00:00.000Z] [t] [takeReading] METERS '
              + 'synthetic=dry hyper=wet openrouter=wet syntheticWeekly=0% syntheticFiveHour=0/2750 '
              + 'syntheticThrottled=no hyperBalance=2497 openrouterUsd=57.62';

            expect(readMeterLine({ line, },),).toEqual({
              at: Date.parse('2026-09-03T17:00:00.000Z',),
              synthetic: 'dry',
              hyper: 'wet',
              neuralwatt: 'absent',
              bedrock: 'absent',
              deepseek: 'absent',
              xiaomi: 'absent',
              anthropic: 'absent',
              openrouter: 'wet',
              levels: [
                'syntheticWeekly=0%',
                'syntheticFiveHour=0/2750',
                'syntheticThrottled=no',
                'hyperBalance=2497',
                'openrouterUsd=57.62',
              ],
            },);
          },
        },),

        it({
          name: 'reads the NeuralWatt state and its watt-hour fields a record carries since 2026-10-20, first in the '
            + 'order, and names the column absent on every older record',
          fn: async () => {
            /**
             A record as `takeReading` writes one with NeuralWatt configured.
             */
            const line = '[info] [2026-10-20T17:00:00.000Z] [t] [takeReading] METERS '
              + 'neuralwatt=wet synthetic=dry bedrock=dry hyper=wet openrouter=wet neuralwattWh=549.986 '
              + 'neuralwattReckonedWh=0.500 syntheticWeekly=0% hyperBalance=2497';

            expect(readMeterLine({ line, },),).toEqual({
              at: Date.parse('2026-10-20T17:00:00.000Z',),
              neuralwatt: 'wet',
              synthetic: 'dry',
              hyper: 'wet',
              bedrock: 'dry',
              deepseek: 'absent',
              xiaomi: 'absent',
              anthropic: 'absent',
              openrouter: 'wet',
              levels: [
                'neuralwattWh=549.986',
                'neuralwattReckonedWh=0.500',
                'syntheticWeekly=0%',
                'hyperBalance=2497',
              ],
            },);
          },
        },),

        it({
          name: 'carries the account figure, the day start and the key allowance fields of the anchored NeuralWatt '
            + 'day through as levels, their values holding colons and dots, and the state from the neuralwatt field alone',
          fn: async () => {
            /**
             A record as `takeReading` writes one since the NeuralWatt day moved to the quota's anchor.
             */
            const line = '[info] [2026-10-20T23:15:00.000Z] [t] [takeReading] METERS '
              + 'neuralwatt=wet synthetic=dry bedrock=dry hyper=wet openrouter=wet neuralwattWh=545.905 '
              + 'neuralwattReckonedWh=0.000 neuralwattAccountWh=4.095 neuralwattDayStart=2026-10-20T19:15:08.000Z '
              + 'neuralwattKeyRemainingUsd=3.25 neuralwattKeyBlocked=no syntheticWeekly=0%';

            expect(readMeterLine({ line, },),).toEqual({
              at: Date.parse('2026-10-20T23:15:00.000Z',),
              neuralwatt: 'wet',
              synthetic: 'dry',
              hyper: 'wet',
              bedrock: 'dry',
              deepseek: 'absent',
              xiaomi: 'absent',
              anthropic: 'absent',
              openrouter: 'wet',
              levels: [
                'neuralwattWh=545.905',
                'neuralwattReckonedWh=0.000',
                'neuralwattAccountWh=4.095',
                'neuralwattDayStart=2026-10-20T19:15:08.000Z',
                'neuralwattKeyRemainingUsd=3.25',
                'neuralwattKeyBlocked=no',
                'syntheticWeekly=0%',
              ],
            },);
          },
        },),

        it({
          name: 'reads a record from a build that printed neuralwattBilled as one more '
            + 'level token and takes its state from the neuralwatt field alone',
          fn: async () => {
            /**
             A record as the builds that printed the billed field wrote it, one marking the day billed.
             */
            const line = '[info] [2026-10-20T17:00:00.000Z] [t] [takeReading] METERS '
              + 'neuralwatt=dry synthetic=dry bedrock=dry hyper=wet openrouter=wet neuralwattWh=549.986 '
              + 'neuralwattReckonedWh=0.500 neuralwattBilled=yes syntheticWeekly=0% hyperBalance=2497';

            expect(readMeterLine({ line, },),).toEqual({
              at: Date.parse('2026-10-20T17:00:00.000Z',),
              neuralwatt: 'dry',
              synthetic: 'dry',
              hyper: 'wet',
              bedrock: 'dry',
              deepseek: 'absent',
              xiaomi: 'absent',
              anthropic: 'absent',
              openrouter: 'wet',
              levels: [
                'neuralwattWh=549.986',
                'neuralwattReckonedWh=0.500',
                'neuralwattBilled=yes',
                'syntheticWeekly=0%',
                'hyperBalance=2497',
              ],
            },);
          },
        },),

        it({
          name: 'reads an unreadable meter as its own state rather than as budget left',
          fn: async () => {
            /**
             A record taken while one provider's meter endpoint was down.
             */
            const line = '[info] [2026-08-24T10:00:00.000Z] [t] [takeReading] '
              + 'METERS synthetic=unreadable hyper=wet';

            expect(readMeterLine({ line, },),).toEqual({
              at: Date.parse('2026-08-24T10:00:00.000Z',),
              synthetic: 'unreadable',
              hyper: 'wet',
              neuralwatt: 'absent',
              bedrock: 'absent',
              deepseek: 'absent',
              xiaomi: 'absent',
              anthropic: 'absent',
              openrouter: 'absent',
              levels: [],
            },);
          },
        },),

        it({
          name: 'reads the numbers a record carries beside the two states',
          fn: async () => {
            /**
             A record as `takeReading` writes one now, numbers included.
             */
            const line = '[info] [2026-08-24T19:00:00.000Z] [t] [takeReading] METERS '
              + 'synthetic=wet hyper=dry syntheticWeekly=97% syntheticFiveHour=48/50 '
              + 'syntheticThrottled=no hyperBalance=0';

            expect(readMeterLine({ line, },),).toEqual({
              at: Date.parse('2026-08-24T19:00:00.000Z',),
              synthetic: 'wet',
              hyper: 'dry',
              neuralwatt: 'absent',
              bedrock: 'absent',
              deepseek: 'absent',
              xiaomi: 'absent',
              anthropic: 'absent',
              openrouter: 'absent',
              levels: [
                'syntheticWeekly=97%',
                'syntheticFiveHour=48/50',
                'syntheticThrottled=no',
                'hyperBalance=0',
              ],
            },);
          },
        },),

        it({
          name: 'READS A RECORD WITH NO FIVE-HOUR WINDOW, whose fields say `none` where the older form said a count, as levels '
            + 'beside the states and not as a state',
          fn: async () => {
            /**
             A record as `takeReading` writes one for a Synthetic reply with no rolling window.
             */
            const line = '[info] [2026-10-19T15:00:00.000Z] [t] [takeReading] METERS '
              + 'synthetic=wet hyper=dry syntheticWeekly=64.5% syntheticFiveHour=none '
              + 'syntheticThrottled=none hyperBalance=0';

            expect(readMeterLine({ line, },),).toEqual({
              at: Date.parse('2026-10-19T15:00:00.000Z',),
              synthetic: 'wet',
              hyper: 'dry',
              neuralwatt: 'absent',
              bedrock: 'absent',
              deepseek: 'absent',
              xiaomi: 'absent',
              anthropic: 'absent',
              openrouter: 'absent',
              levels: [
                'syntheticWeekly=64.5%',
                'syntheticFiveHour=none',
                'syntheticThrottled=none',
                'hyperBalance=0',
              ],
            },);
          },
        },),

        it({
          name: 'ACCEPTS a level field it has never been taught the name of',
          fn: async () => {
            /**
             A record from a build that writes a field this reader predates,
             which must reach a human rather than being dropped for being new.
             */
            const line = '[info] [2026-08-24T19:00:00.000Z] [t] [takeReading] METERS '
              + 'synthetic=wet hyper=wet hyperResetsAt=03:00';

            expect(readMeterLine({ line, },),).toEqual({
              at: Date.parse('2026-08-24T19:00:00.000Z',),
              synthetic: 'wet',
              hyper: 'wet',
              neuralwatt: 'absent',
              bedrock: 'absent',
              deepseek: 'absent',
              xiaomi: 'absent',
              anthropic: 'absent',
              openrouter: 'absent',
              levels: ['hyperResetsAt=03:00',],
            },);
          },
        },),

        it({
          name: 'REFUSES to treat prose mentioning the marker as a record',
          fn: async () => {
            expect(readMeterLine({ line: PROSE_LINE, },),).toBe('not-a-record',);
          },
        },),

        it({
          name: 'ignores an ordinary log line',
          fn: async () => {
            /**
             A line from elsewhere in the pipeline entirely.
             */
            const line = '[info] [2026-08-24T10:00:00.000Z] [t] [runEntry] settled XYZ';

            expect(readMeterLine({ line, },),).toBe('not-a-record',);
          },
        },),

        it({
          name: 'ACCEPTS a record with no readable stamp as a skip, not as a reading',
          fn: async () => {
            /**
             A record whose prefix was mangled, which a truncated write produces.
             */
            const line = '[info] [not-a-date] [t] [takeReading] METERS synthetic=wet hyper=wet';

            expect(readMeterLine({ line, },),).toBe('skipped',);
          },
        },),

        it({
          name: 'SKIPS A RECORD WHOSE STAMP THE LOGGER DID NOT WRITE, rather than dating a reading by text no writer here '
            + 'makes: a stamp without its zone reads as local time and a date alone as midnight (ledger B73)',
          fn: async () => {
            expect(STAMPS_NOT_WRITTEN.map(function readingOf(stamp,) {
              return readMeterLine({
                line: REAL_LINE.replace(
                  '2026-08-24T18:17:35.383Z',
                  stamp,
                ),
              },);
            },),).toEqual(STAMPS_NOT_WRITTEN.map(function skipped(): string {
              return 'skipped';
            },),);
          },
        },),

        it({
          name: 'ACCEPTS a record cut off part way as a skip, so the hole stays visible',
          fn: async () => {
            /**
             A record whose second field never made it to disk. Its first field
             still parses, which is what keeps it counted rather than ignored.
             */
            const line = '[info] [2026-08-24T10:00:00.000Z] [t] [takeReading] METERS synthetic=wet hyp';

            expect(readMeterLine({ line, },),).toBe('skipped',);
          },
        },),
        it({
          name: 'SKIPS a record whose field carries no meter state even when a later field repeats the '
            + 'name, since the first reading is absent',
          fn: async () => {
            expect(readMeterLine({
              line: '[info] [2026-08-24T18:17:35.383Z] [translation-repair] [takeReading] '
                + 'METERS hyper=dry synthetic=bogus synthetic=wet',
            },),).toBe('skipped',);
          },
        },),
        it({
          name: 'SKIPS A RECORD THAT WRITES ONE FIELD TWICE, a state or a level, as two records run together leave '
            + 'it, since the writer writes each once and the first state would pass for the reading of both',
          fn: async () => {
            expect([
              'METERS synthetic=wet hyper=dry synthetic=dry',
              'METERS synthetic=wet hyper=dry hyperBalance=0 hyperBalance=2497',
            ].map(function readingOf(tail,) {
              return readMeterLine({
                line: `[info] [2026-08-24T18:17:35.383Z] [translation-repair] [takeReading] ${tail}`,
              },);
            },),).toEqual([
              'skipped',
              'skipped',
            ],);
          },
        },),
        it({
          name: 'counts no level out of a field carrying no separator, since it names no reading',
          fn: async () => {
            expect(readMeterLine({
              line: '[info] [2026-08-24T18:17:35.383Z] [translation-repair] [takeReading] '
                + 'METERS synthetic=wet hyper=dry hyperBalance',
            },),).toEqual({
              at: Date.parse('2026-08-24T18:17:35.383Z',),
              synthetic: 'wet',
              hyper: 'dry',
              neuralwatt: 'absent',
              bedrock: 'absent',
              deepseek: 'absent',
              xiaomi: 'absent',
              anthropic: 'absent',
              openrouter: 'absent',
              levels: [],
            },);
            expect(readMeterLine({
              line: '[info] [2026-08-24T18:17:35.383Z] [translation-repair] [takeReading] '
                + 'METERS synthetic=wet hyper=dry hyperBalance=0',
            },),).toEqual({
              at: Date.parse('2026-08-24T18:17:35.383Z',),
              synthetic: 'wet',
              hyper: 'dry',
              neuralwatt: 'absent',
              bedrock: 'absent',
              deepseek: 'absent',
              xiaomi: 'absent',
              anthropic: 'absent',
              openrouter: 'absent',
              levels: ['hyperBalance=0',],
            },);
          },
        },),
        it({
          name: 'skips a record carrying both states whose prefix holds no bracketed timestamp, in each shape '
            + 'a broken prefix takes: no bracket at all, a level bracket with none after it, and a second bracket '
            + 'never closed',
          fn: async () => {
            expect([
              'nostamp METERS synthetic=wet hyper=dry',
              'x] y METERS synthetic=wet hyper=dry',
              'x] [y METERS synthetic=wet hyper=dry',
            ].map(function readingOf(line,) {
              return readMeterLine({ line, },);
            },),).toEqual([
              'skipped',
              'skipped',
              'skipped',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: readMeterLog.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'separates readings from holes across a whole log',
          fn: async () => {
            /**
             A log carrying one good record, one prose mention, one ordinary
             line, and one truncated record.
             */
            const text = [
              REAL_LINE,
              PROSE_LINE,
              '[info] [2026-08-24T10:00:00.000Z] [t] [runEntry] settled XYZ',
              '[info] [2026-08-24T10:00:00.000Z] [t] [takeReading] METERS synthetic=dry hyp',
            ].join('\n',);

            /**
             What the log yielded.
             */
            const {
              samples,
              skippedLines,
            } = readMeterLog({ text, },);

            expect(samples.length,).toBe(1,);
            expect(skippedLines,).toBe(1,);
            expect(samples[0]?.hyper,).toBe('dry',);
          },
        },),

        it({
          name: 'ACCEPTS a log with no records at all, reporting neither readings nor holes',
          fn: async () => {
            expect(readMeterLog({ text: 'nothing here\nnor here\n', },),).toEqual({
              samples: [],
              skippedLines: 0,
            },);
          },
        },),
      ],
    },),
  ],
},);
