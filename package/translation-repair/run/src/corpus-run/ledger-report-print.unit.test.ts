/**
 Tests for what the ledger report prints, one printer at a time.

 EACH PRINTER IS HANDED THE READING IT PRINTS, so a case states the whole
 reading and asserts the whole text it produced. A count noun is shown at one
 and at several, since the report once printed a count before a fixed plural.

 THE EXCERPT IS CHECKED AT ITS CUT. A candidate longer than the excerpt is
 shown to a whole character only, and the report says it was cut, because
 candidate text is the evidence a reader weighs and a reader who cannot tell a
 whole candidate from an opening reads the opening as the whole.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type CandidateReading,
  type LedgerReading,
  printReading,
  printRefusals,
  printSeat,
  printSummary,
  type ReadRound,
} from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  NAP_CONTEST,
  PURR_CONTEST,
  SHORTFALL_SENTENCE,
  SUMMARY_POINTER,
} from './ledger-report.test-fixture.ts';

/**
 A candidate a panel chose, with one reason a judge gave.
 */
const CHOSEN_READING: CandidateReading = {
  task: 'render the nap passage',
  rendered: 'The cat naps in the sun.',
  won: true,
  remarks: ['siamese-3: keeps the sun and the nap',],
};

/**
 A candidate nobody named.
 */
const UNNAMED_READING: CandidateReading = {
  task: 'render the purr passage',
  rendered: 'The purr rolls on.',
  won: false,
  remarks: [],
};

/**
 A reading with contests and nothing refused.
 */
const WHOLE_READING: LedgerReading = {
  rounds: [
    NAP_CONTEST,
    PURR_CONTEST,
  ],
  refused: [],
};

/**
 Two contests no seat was judged over: nobody cast a ballot.
 */
const UNJUDGED_ROUND: ReadRound = {
  task: 'render the stretch passage',
  at: '2026-08-25T01:10:00.000Z',
  candidates: [
    {
      index: 1,
      producers: ['tabby-1',],
      rendered: 'The cat stretches.',
    },
  ],
  ballots: [],
  selectedIndex: 'declined',
};

await describe({
  name: 'ledger report printers',
  concurrency: 1,
  children: [
    describe({
      name: printReading.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS a chosen candidate with its position, its task, its text and the reasons judges gave',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printReading({
              reading: CHOSEN_READING,
              at: 0,
            },);
            expect(printed.lines,).toEqual([
              '\n--- 1 --- CHOSEN --- render the nap passage',
              'The cat naps in the sun.',
              '  siamese-3: keeps the sun and the nap',
            ],);
          },
        },),

        it({
          name: 'PRINTS a candidate nobody chose as not chosen and says no disinterested judge named it',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printReading({
              reading: UNNAMED_READING,
              at: 4,
            },);
            expect(printed.lines,).toEqual([
              '\n--- 5 --- not chosen --- render the purr passage',
              'The purr rolls on.',
              '  (no disinterested judge named this candidate)',
            ],);
          },
        },),

        it({
          name: 'PRINTS every reason a candidate drew, one line each, in the order the judges gave them',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printReading({
              reading: {
                ...CHOSEN_READING,
                remarks: [
                  'siamese-3: keeps the sun',
                  'bengal-4: keeps the nap',
                ],
              },
              at: 0,
            },);
            expect(printed.lines,).toEqual([
              '\n--- 1 --- CHOSEN --- render the nap passage',
              'The cat naps in the sun.',
              '  siamese-3: keeps the sun',
              '  bengal-4: keeps the nap',
            ],);
          },
        },),

        it({
          name: 'PRINTS a candidate of exactly the excerpt length whole, with no note of a cut',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Text of exactly the most the excerpt shows.
             */
            const rendered = 'm'.repeat(400,);
            printReading({
              reading: {
                ...UNNAMED_READING,
                rendered,
              },
              at: 0,
            },);
            expect(printed.lines,).toEqual([
              '\n--- 1 --- not chosen --- render the purr passage',
              rendered,
              '  (no disinterested judge named this candidate)',
            ],);
          },
        },),

        it({
          name: 'CUTS a longer candidate to the excerpt and says how much of it is shown',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printReading({
              reading: {
                ...UNNAMED_READING,
                rendered: 'm'.repeat(450,),
              },
              at: 0,
            },);
            expect(printed.lines,).toEqual([
              '\n--- 1 --- not chosen --- render the purr passage',
              'm'.repeat(400,),
              '  (cut: 400 of 450 UTF-16 units shown)',
              '  (no disinterested judge named this candidate)',
            ],);
          },
        },),

        it({
          name: 'CUTS before a surrogate pair the limit would split and counts the units shown, not the limit',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Text whose last unit before the limit is the first half of a pair.
             */
            const rendered = `${'m'.repeat(399,)}\u{1F431}${'m'.repeat(10,)}`;
            printReading({
              reading: {
                ...UNNAMED_READING,
                rendered,
              },
              at: 0,
            },);
            expect(printed.lines,).toEqual([
              '\n--- 1 --- not chosen --- render the purr passage',
              'm'.repeat(399,),
              '  (cut: 399 of 411 UTF-16 units shown)',
              '  (no disinterested judge named this candidate)',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: printRefusals.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS nothing when every ledger file read',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printRefusals({ reading: WHOLE_READING, },);
            expect(printed.lines,).toEqual([],);
          },
        },),

        it({
          name: 'NAMES the one file that would not read and counts it against one file read, in the singular',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printRefusals({
              reading: {
                rounds: [],
                refused: [
                  {
                    file: 'a-000001.json',
                    says: 'ledger file a-000001.json has no usable task',
                  },
                ],
              },
            },);
            expect(printed.lines,).toEqual([
              '  UNREADABLE a-000001.json: ledger file a-000001.json has no usable task',
              `  1 of 1 ledger file could not be read. ${SHORTFALL_SENTENCE}`,
            ],);
          },
        },),

        it({
          name: 'NAMES every file that would not read and counts them against every file, in the plural',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printRefusals({
              reading: {
                rounds: [NAP_CONTEST,],
                refused: [
                  {
                    file: 'a-000002.json',
                    says: 'ledger file a-000002.json has no usable at',
                  },
                  {
                    file: 'a-000003.json',
                    says: 'refused by SyntaxError',
                  },
                ],
              },
            },);
            expect(printed.lines,).toEqual([
              '  UNREADABLE a-000002.json: ledger file a-000002.json has no usable at',
              '  UNREADABLE a-000003.json: refused by SyntaxError',
              `  2 of 3 ledger files could not be read. ${SHORTFALL_SENTENCE}`,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: printSummary.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS both ballot faults, each seat\'s share of the disinterested ballots, and the pointer to a seat',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printSummary({ reading: WHOLE_READING, },);
            expect(printed.lines,).toEqual([
              '1 ballot named nothing, 1 named a candidate the slate did not have',
              '  tabby-1: 2 candidates, 1 chosen, 33.3% of 3 disinterested ballots, 1 self-vote',
              '  calico-2: 2 candidates, 0 chosen, 0.0% of 4 disinterested ballots, 0 self-votes',
              `\n${SUMMARY_POINTER}`,
            ],);
          },
        },),

        it({
          name: 'PRINTS the plural of a ballot count of zero and a seat with no disinterested ballot as UNJUDGED',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printSummary({
              reading: {
                rounds: [UNJUDGED_ROUND,],
                refused: [],
              },
            },);
            expect(printed.lines,).toEqual([
              '0 ballots named nothing, 0 named a candidate the slate did not have',
              '  tabby-1: 1 candidate, 0 chosen, UNJUDGED of 0 disinterested ballots, 0 self-votes',
              `\n${SUMMARY_POINTER}`,
            ],);
          },
        },),

        it({
          name: 'PRINTS the singular of a disinterested ballot and of a self-vote at a count of one',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printSummary({
              reading: {
                rounds: [
                  {
                    ...UNJUDGED_ROUND,
                    candidates: [
                      {
                        index: 1,
                        producers: ['tabby-1',],
                        rendered: 'The cat stretches.',
                      },
                      {
                        index: 2,
                        producers: ['calico-2',],
                        rendered: 'The cat yawns.',
                      },
                    ],
                    ballots: [
                      {
                        modelId: 'tabby-1',
                        best: 1,
                        reason: 'mine',
                      },
                    ],
                    selectedIndex: 1,
                  },
                ],
                refused: [],
              },
            },);
            expect(printed.lines,).toEqual([
              '0 ballots named nothing, 0 named a candidate the slate did not have',
              '  tabby-1: 1 candidate, 1 chosen, UNJUDGED of 0 disinterested ballots, 1 self-vote',
              '  calico-2: 1 candidate, 0 chosen, 0.0% of 1 disinterested ballot, 0 self-votes',
              `\n${SUMMARY_POINTER}`,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: printSeat.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS what a seat wrote, in the plural, then each candidate with the reasons judges gave',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printSeat({
              reading: WHOLE_READING,
              wanted: 'tabby-1',
            },);
            expect(printed.lines,).toEqual([
              'tabby-1 wrote 2 candidates, 1 chosen',
              '\n--- 1 --- CHOSEN --- render the nap passage',
              'The cat naps in the sun.',
              '  siamese-3: keeps the sun and the nap',
              '\n--- 2 --- not chosen --- render the purr passage',
              'The purr rolls on.',
              '  (no disinterested judge named this candidate)',
            ],);
          },
        },),

        it({
          name: 'PRINTS the singular of a candidate for a seat that wrote one',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printSeat({
              reading: {
                rounds: [UNJUDGED_ROUND,],
                refused: [],
              },
              wanted: 'tabby-1',
            },);
            expect(printed.lines,).toEqual([
              'tabby-1 wrote 1 candidate, 0 chosen',
              '\n--- 1 --- not chosen --- render the stretch passage',
              'The cat stretches.',
              '  (no disinterested judge named this candidate)',
            ],);
          },
        },),

        it({
          name: 'SAYS a seat the ledger never names wrote no candidates, rather than printing nothing',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printSeat({
              reading: WHOLE_READING,
              wanted: 'sphynx-9',
            },);
            expect(printed.lines,).toEqual(['sphynx-9 wrote 0 candidates, 0 chosen',],);
          },
        },),
      ],
    },),
  ],
},);
