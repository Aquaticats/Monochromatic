/**
 Tests for the printed pair of standings of the editor calibration: the
 editor table and the refiner table, each fed from the rounds its seat was
 judged in and the models that shipped or were heard.

 CAPTURING `console.log` IS PROCESS-WIDE, which is why this file runs at
 `concurrency: 1`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { SelectionRound, } from '@monochromatic-dev/translation-repair-translate-floor';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { printEditorCalibrateStandings, } from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

/**
 Model that wrote the candidate in the voted round.
 */
const WRITER = SEAT_HYPER_VISION;

/**
 Model that cast the ballot.
 */
const JUDGE = SEAT_HYPER_TEXT_BEDROCK;

/**
 Model that did nothing at either seat.
 */
const IDLE = SEAT_HYPER_ONLY;

/**
 Every seat the run filled.
 */
const ROSTER: readonly RosterModelId[] = [WRITER, JUDGE, IDLE,];

/**
 One judged round: a single candidate by the writer, voted for by the judge.
 */
const VOTED_ROUND: SelectionRound = {
  producers: [
    {
      kind: 'model',
      modelId: WRITER,
    },
  ],
  ballots: [
    {
      modelId: JUDGE,
      best: 1,
      reason: 'scripted',
      weight: 1,
      selfVote: false,
    },
  ],
};

/**
 Note a seat prints when it judged no round, the same for both seats.
 */
const NO_ROUNDS_NOTE = '  NO ROUNDS. This seat judged nothing across the sample, so it has no standing. For the editor '
  + 'seat that means no slice carried an ACCEPTED issue: critics can raise claims and the panel can adjudicate '
  + 'them and the lane still report "nothing to edit", which is what one live slice did. For the refiner seat it '
  + 'means the naturalness lane proposed nothing. Draw more slices.';

/**
 Row a seat prints for the writer after one disinterested ballot over one candidate.
 */
const WRITER_ROW = `  ${WRITER}: 100.0% (1 of 1 disinterested ballot, over 1 candidate)`;

/**
 What every seat's table says about a model nobody asked, from the seat line at the end of the command.
 */
const SEAT_LINES_POINTER = 'the SEAT lines at the end of this command say how often each seat was asked and how many '
  + 'answers were usable, and the run log names the failure. Re-run these seats before reading the table as a '
  + 'comparison of the roster.';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: printEditorCalibrateStandings.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS both headings with the no-rounds note for a sample that judged nothing',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printEditorCalibrateStandings({
              roster: ROSTER,
              perSlice: [],
            },);

            expect(printed.lines,).toEqual([
              '\nEDITOR standing over 0 judged rounds, from 0 of 0 slices',
              NO_ROUNDS_NOTE,
              '\nREFINER standing over 0 judged rounds, from 0 of 0 slices',
              NO_ROUNDS_NOTE,
            ],);
          },
        },),

        it({
          name: 'PRINTS the editor table before the refiner table, each credited from its own seat only, '
            + 'with shipping counted for the editor seat and heard for the refiner seat',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printEditorCalibrateStandings({
              roster: ROSTER,
              perSlice: [
                {
                  editor: [VOTED_ROUND,],
                  refiner: [],
                  refineAsked: true,
                  editorShipped: [WRITER,],
                  refinerShipped: [],
                  refinerHeard: [JUDGE,],
                },
                {
                  editor: [],
                  refiner: [VOTED_ROUND,],
                  refineAsked: true,
                  editorShipped: [JUDGE,],
                  refinerShipped: [WRITER,],
                  refinerHeard: [],
                },
              ],
            },);

            expect(printed.lines,).toEqual([
              '\nEDITOR standing over 1 judged round, from 1 of 2 slices',
              WRITER_ROW,
              `  WROTE AND WAS NEVER VOTED ON: ${JUDGE}. Their text reached a slate and no disinterested ballot was `
                + 'cast over it, which is what a slice where every producer proposed the same wording does: it ships '
                + 'unjudged. The table says nothing about them either way, and more slices are what would.',
              `  NO CANDIDATE OF THEIRS REACHED ANY SLATE: ${IDLE}, so the table covers 1 of 3 seats. This seat does `
                + `not record who answered, so a model that answered and was dropped before judging and a provider `
                + `that failed every call look alike from here; ${SEAT_LINES_POINTER}`,
              `  slice 1: 1 judged round; ${WRITER} 1/1 over 1`,
              '\nREFINER standing over 1 judged round, from 1 of 2 slices',
              WRITER_ROW,
              `  ANSWERED AND WAS NEVER SLATED: ${JUDGE}. At least one usable answer of theirs was heard and none `
                + 'became a candidate a judge saw: a rewriter that leaves a paragraph as it stands, or whose rewrite '
                + 'is dropped before judging, looks like this. The table says nothing about them, their SEAT lines '
                + 'carry the answers, and re-running them buys the same again; slices with something to rewrite are '
                + 'what would seat them.',
              `  ANSWERED NOTHING USABLE: ${IDLE}. No answer of theirs was heard at this seat, so the table covers `
                + `1 of 3 seats. A provider out of budget, a refused sheet and a call that timed out all look like this `
                + `from here; ${SEAT_LINES_POINTER}`,
              `  slice 2: 1 judged round; ${WRITER} 1/1 over 1`,
            ],);
          },
        },),
        it({
          name: 'ACCOUNTS THE REFINER TABLE FOR THE REFINER ROSTER WHEN ONE IS GIVEN (C4): a model seated as an editor and '
            + 'not as a refiner is not named as a refiner that answered nothing',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printEditorCalibrateStandings({
              roster: ROSTER,
              refinerRoster: [WRITER, JUDGE,],
              perSlice: [
                {
                  editor: [],
                  refiner: [VOTED_ROUND,],
                  refineAsked: true,
                  editorShipped: [],
                  refinerShipped: [WRITER,],
                  refinerHeard: [JUDGE,],
                },
              ],
            },);

            /**
             The lines the refiner table printed.
             */
            const refinerLines = printed.lines.slice(
              printed.lines.findIndex(function isRefinerHeading(line,): boolean {
                return line.startsWith('\nREFINER standing',);
              },),
            );
            expect(refinerLines.some(function namesIdle(line,): boolean {
              return line.includes(IDLE,);
            },),).toBe(false,);
            expect(refinerLines.length,).toBeGreaterThan(1,);
          },
        },),
      ],
    },),
  ],
},);
