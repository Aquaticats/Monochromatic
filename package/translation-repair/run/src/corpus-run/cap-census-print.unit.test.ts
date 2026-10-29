/**
 Tests for how the cap census reads on a terminal: the line of what was read,
 each seat's row, each provider's two readings and each flag's words.

 What is printed is read off `console.log` through a diverting capture, so a
 case sees the whole text and the terminal sees none of it. The capture is
 process-wide, which is why the suite runs one case at a time.

 Rows are invented over a roster seat, with invented counts.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  COMPLETION_CAP,
  MIN_PROVIDER_CALLS,
  POOLED_P90,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type CapCensus,
  capCensusProviderLine,
  type CapCensusRow,
  type CapCensusTally,
  printCapCensus,
  type ProviderCapReading,
} from '../../dist/final/node/index.mjs';
import { SEAT_HYPER_OPENROUTER_UNMEASURED, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { CAP_CENSUS_CLOSING_NOTE, } from '@monochromatic-dev/translation-repair-provider-stream/ts/corpus-run/cap-census-log.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

/**
 Seat the rows belong to.
 */
const SEAT = SEAT_HYPER_OPENROUTER_UNMEASURED;

/**
 What was counted before the rule was applied, every count plural.
 */
const TALLY_PLURAL = {
  logs: 3,
  passRunLogs: 2,
  samples: 40,
  unreadable: 4,
  danglingLinks: 3,
  unstampedLines: 5,
};

/**
 One provider's reading, with every count given.

 @param calls - completed calls
 @param sinceCaps - of those, calls since the caps
 @param atCap - of those, calls at the cap

 @returns Reading, its cut calls all carrying content

 @example
 ```ts
 const reading = readingOf({ calls: 10, sinceCaps: 8, atCap: 2, },);
 ```
 */
function readingOf(
  {
    calls,
    sinceCaps,
    atCap,
  }: {
    readonly calls: number;
    readonly sinceCaps: number;
    readonly atCap: number;
  },
): ProviderCapReading {
  return {
    provider: 'hyper',
    calls,
    p99: 321,
    sinceCaps,
    atCap,
    atCapWithContent: atCap,
    atCapNoContent: 0,
    atCapUnpaired: 0,
  };
}

/**
 A seat's row over the given readings.

 @param placeholder - whether the card names the pooled 99th
 @param ruleCap - rule's reading
 @param providers - one reading per provider

 @returns Row

 @example
 ```ts
 const row = rowOf({ placeholder: false, ruleCap: 'too-few-calls', providers: [], },);
 ```
 */
function rowOf(
  {
    placeholder,
    ruleCap,
    providers,
  }: {
    readonly placeholder: boolean;
    readonly ruleCap: CapCensusRow['ruleCap'];
    readonly providers: readonly ProviderCapReading[];
  },
): CapCensusRow {
  return {
    modelId: SEAT,
    cardCap: COMPLETION_CAP[SEAT],
    placeholder,
    ruleCap,
    providers,
  };
}

/**
 What `printCapCensus` printed for a census and a tally.

 @param census - rule applied per seat
 @param tally - what was counted before it
 @param printed - capture of `console.log`

 @returns Lines, one per call

 @example
 ```ts
 const lines = printedFor({ census, tally, printed, },);
 ```
 */
function printedFor(
  {
    census,
    tally,
    printed,
  }: {
    readonly census: CapCensus;
    readonly tally: CapCensusTally;
    readonly printed: { readonly lines: readonly string[]; };
  },
): readonly string[] {
  printCapCensus({
    census,
    tally,
  },);
  return printed.lines;
}

await describe({
  name: 'cap-census print',
  concurrency: 1,
  children: [
    describe({
      name: capCensusProviderLine.name,
      children: [
        it({
          name: 'PRINTS A PROVIDER\'S PLURAL COUNTS with the share of its capped calls that ran to the cap, to two places',
          fn: async () => {
            expect(capCensusProviderLine({ reading: readingOf({ calls: 12, sinceCaps: 3, atCap: 1, },), },),).toBe(
              '  hyper: 12 calls, p99 321; since the caps 3 calls, 1 at the cap (33.33%): '
                + '1 with content, 0 with none, 0 unpaired',
            );
          },
        },),
        it({
          name: 'PRINTS A PROVIDER\'S SINGULAR COUNTS with "call" after a count of one',
          fn: async () => {
            expect(capCensusProviderLine({ reading: readingOf({ calls: 1, sinceCaps: 1, atCap: 0, },), },),).toBe(
              '  hyper: 1 call, p99 321; since the caps 1 call, 0 at the cap (0.00%): '
                + '0 with content, 0 with none, 0 unpaired',
            );
          },
        },),
        it({
          name: 'PRINTS N/A FOR THE SHARE where no call was made since the caps, rather than a share of nothing',
          fn: async () => {
            expect(capCensusProviderLine({ reading: readingOf({ calls: 2, sinceCaps: 0, atCap: 0, },), },),).toBe(
              '  hyper: 2 calls, p99 321; since the caps 0 calls, 0 at the cap (n/a): '
                + '0 with content, 0 with none, 0 unpaired',
            );
          },
        },),
      ],
    },),

    describe({
      name: printCapCensus.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS THE COUNTS IN THE PLURAL, no seat row, and the closing note, for a census of nothing',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            expect(printedFor({
              census: {
                rows: [],
                offRoster: 6,
              },
              tally: TALLY_PLURAL,
              printed,
            },),).toEqual([
              'cap-census: 3 logs, 2 pass-run logs, 40 completed calls, 6 on ids no card names, 4 paths unreadable, '
                + '3 dangling links with no log name passed over; '
                + 'lines left out for a stamp the logger did not write: 5',
              CAP_CENSUS_CLOSING_NOTE,
            ],);
          },
        },),
        it({
          name: 'PRINTS THE COUNTS IN THE SINGULAR, each noun after a count of one',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            expect(printedFor({
              census: {
                rows: [],
                offRoster: 1,
              },
              tally: {
                logs: 1,
                passRunLogs: 1,
                samples: 1,
                unreadable: 1,
                danglingLinks: 1,
                unstampedLines: 1,
              },
              printed,
            },),).toEqual([
              'cap-census: 1 log, 1 pass-run log, 1 completed call, 1 on ids no card names, 1 path unreadable, '
                + '1 dangling link with no log name passed over; '
                + 'lines left out for a stamp the logger did not write: 1',
              CAP_CENSUS_CLOSING_NOTE,
            ],);
          },
        },),
        it({
          name: 'PRINTS A SEAT WHOSE CARD AND CALLS AGREE as its card cap, what the rule reads and its provider, with no flag',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            expect(printedFor({
              census: {
                rows: [rowOf({
                  placeholder: false,
                  ruleCap: COMPLETION_CAP[SEAT],
                  providers: [readingOf({ calls: 4, sinceCaps: 4, atCap: 0, },),],
                },),],
                offRoster: 0,
              },
              tally: TALLY_PLURAL,
              printed,
            },).slice(1,),).toEqual([
              `${SEAT}: card cap ${String(COMPLETION_CAP[SEAT],)}, rule reads ${String(COMPLETION_CAP[SEAT],)}`,
              '  hyper: 4 calls, p99 321; since the caps 4 calls, 0 at the cap (0.00%): '
                + '0 with content, 0 with none, 0 unpaired',
              CAP_CENSUS_CLOSING_NOTE,
            ],);
          },
        },),
        it({
          name: 'PRINTS A POOLED PLACEHOLDER AND A RULE THAT READS NOTHING for a seat with too few calls, and no flag',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            expect(printedFor({
              census: {
                rows: [rowOf({
                  placeholder: true,
                  ruleCap: 'too-few-calls',
                  providers: [readingOf({ calls: 1, sinceCaps: 1, atCap: 0, },),],
                },),],
                offRoster: 0,
              },
              tally: TALLY_PLURAL,
              printed,
            },).slice(1,),).toEqual([
              `${SEAT}: card cap ${String(COMPLETION_CAP[SEAT],)} (pooled placeholder), rule reads nothing (too few calls)`,
              '  hyper: 1 call, p99 321; since the caps 1 call, 0 at the cap (0.00%): '
                + '0 with content, 0 with none, 0 unpaired',
              CAP_CENSUS_CLOSING_NOTE,
            ],);
          },
        },),
        it({
          name: 'PRINTS EACH FLAG IN ITS WORDS beneath the seat it names, in the order the flags come',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            expect(printedFor({
              census: {
                rows: [rowOf({
                  placeholder: true,
                  ruleCap: POOLED_P90,
                  providers: [readingOf({ calls: MIN_PROVIDER_CALLS, sinceCaps: MIN_PROVIDER_CALLS, atCap: 2, },),],
                },),],
                offRoster: 0,
              },
              tally: TALLY_PLURAL,
              printed,
            },).slice(
              3,
              -1,
            ),).toEqual([
              '  PLACEHOLDER WITH A DISTRIBUTION: the card names the pooled 99th and its calls now read by the rule',
              `  RULE READS ${String(POOLED_P90,)} AGAINST THE CARD'S ${String(COMPLETION_CAP[SEAT],)}`,
              '  CUTS OVER ONE PERCENT on at least one provider with enough calls to say',
            ],);
          },
        },),
        it({
          name: 'PRINTS EVERY SEAT\'S ROW AND EVERY PROVIDER\'S LINE where several are read, seat by seat',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            /**
             Two seats' rows, the first with two providers.
             */
            const rows = [
              rowOf({
                placeholder: false,
                ruleCap: COMPLETION_CAP[SEAT],
                providers: [readingOf({ calls: 2, sinceCaps: 2, atCap: 0, },), readingOf({ calls: 3, sinceCaps: 3, atCap: 0, },),],
              },),
              rowOf({
                placeholder: false,
                ruleCap: 'too-few-calls',
                providers: [readingOf({ calls: 1, sinceCaps: 1, atCap: 0, },),],
              },),
            ];

            expect(printedFor({
              census: {
                rows,
                offRoster: 0,
              },
              tally: TALLY_PLURAL,
              printed,
            },).slice(1,),).toEqual([
              `${SEAT}: card cap ${String(COMPLETION_CAP[SEAT],)}, rule reads ${String(COMPLETION_CAP[SEAT],)}`,
              '  hyper: 2 calls, p99 321; since the caps 2 calls, 0 at the cap (0.00%): '
                + '0 with content, 0 with none, 0 unpaired',
              '  hyper: 3 calls, p99 321; since the caps 3 calls, 0 at the cap (0.00%): '
                + '0 with content, 0 with none, 0 unpaired',
              `${SEAT}: card cap ${String(COMPLETION_CAP[SEAT],)}, rule reads nothing (too few calls)`,
              '  hyper: 1 call, p99 321; since the caps 1 call, 0 at the cap (0.00%): '
                + '0 with content, 0 with none, 0 unpaired',
              CAP_CENSUS_CLOSING_NOTE,
            ],);
          },
        },),
      ],
    },),
  ],
},);
