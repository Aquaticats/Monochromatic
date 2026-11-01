/**
 Tests for the two closing paragraphs of the editor calibration's report: how
 many slices reached a rewriter, and what shipped with no vote behind it.

 CAPTURING `console.log` IS PROCESS-WIDE, which is why this file runs at
 `concurrency: 1`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  printEditorCalibrateRefineReach,
  printEditorCalibrateShipped,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { unjudgedRounds, } from './editor-calibrate-rounds.test-fixture.ts';

/**
 Closing clause of the reach line when one slice carried nothing to rewrite.
 */
const ONE_UNREACHED = '; 1 slice carried no paragraph over the eligibility floor, so no refiner was asked '
  + 'there and its silence is not evidence about any model';

/**
 Closing sentence the shipped paragraph ends on whenever something shipped.
 */
const NOT_A_PREFERENCE = '  THIS IS NOT A PREFERENCE. A model ships here by writing text that survived, including '
  + 'text every other editor proposed identically, which no judge was ever asked about.';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: printEditorCalibrateRefineReach.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS how many of the slices reached a rewriter, with the slice noun in the plural',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printEditorCalibrateRefineReach({
              perSlice: [
                unjudgedRounds({ shippers: [], refineAsked: true, },),
                unjudgedRounds({ shippers: [], refineAsked: false, },),
                unjudgedRounds({ shippers: [], refineAsked: true, },),
              ],
            },);

            expect(printed.lines,).toEqual([`  reached a rewriter on 2 of 3 slices${ONE_UNREACHED}`,],);
          },
        },),

        it({
          name: 'PRINTS NO CLAUSE ABOUT THE REST when every slice reached a rewriter, since there is no rest',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printEditorCalibrateRefineReach({
              perSlice: [
                unjudgedRounds({ shippers: [], refineAsked: true, },),
                unjudgedRounds({ shippers: [], refineAsked: true, },),
              ],
            },);

            expect(printed.lines,).toEqual(['  reached a rewriter on 2 of 2 slices',],);
          },
        },),

        it({
          name: 'NAMES BOTH SLICES, in the plural, as carrying nothing to rewrite when neither of two reached a '
            + 'rewriter',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printEditorCalibrateRefineReach({
              perSlice: [
                unjudgedRounds({ shippers: [], refineAsked: false, },),
                unjudgedRounds({ shippers: [], refineAsked: false, },),
              ],
            },);

            expect(printed.lines,).toEqual([
              '  reached a rewriter on 0 of 2 slices; 2 slices carried no paragraph over the eligibility floor, so '
                + 'no refiner was asked there and their silence is not evidence about any model',
            ],);
          },
        },),

        it({
          name: 'NAMES THE ONE SLICE, in the singular, that carried nothing to rewrite when one of two reached a '
            + 'rewriter',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printEditorCalibrateRefineReach({
              perSlice: [
                unjudgedRounds({ shippers: [], refineAsked: false, },),
                unjudgedRounds({ shippers: [], refineAsked: true, },),
              ],
            },);

            expect(printed.lines,).toEqual([`  reached a rewriter on 1 of 2 slices${ONE_UNREACHED}`,],);
          },
        },),

        it({
          name: 'PRINTS the slice noun in the singular when the run drew one slice, and zero when it reached no rewriter',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printEditorCalibrateRefineReach({
              perSlice: [unjudgedRounds({ shippers: [], refineAsked: false, },),],
            },);

            expect(printed.lines,).toEqual([`  reached a rewriter on 0 of 1 slice${ONE_UNREACHED}`,],);
          },
        },),
      ],
    },),

    describe({
      name: printEditorCalibrateShipped.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS that nothing shipped, and stops, when no slice carried an editor credited with a repair',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printEditorCalibrateShipped({
              perSlice: [
                unjudgedRounds({ shippers: [], refineAsked: true, },),
                unjudgedRounds({ shippers: [], refineAsked: false, },),
              ],
            },);

            expect(printed.lines,).toEqual([
              '\nEDITORS SHIPPED on 0 of 2 slices, 0 of them with no editor round judged at all',
              '  NOTHING SHIPPED. No slice in this sample carried an accepted issue.',
            ],);
          },
        },),

        it({
          name: 'PRINTS each model with the slices it wrote shipping text on, most slices first, then the preference caveat',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printEditorCalibrateShipped({
              perSlice: [
                unjudgedRounds({ shippers: [SEAT_HYPER_VISION,], refineAsked: false, },),
                unjudgedRounds({
                  shippers: [SEAT_HYPER_VISION, SEAT_HYPER_TEXT_BEDROCK,],
                  refineAsked: false,
                },),
                unjudgedRounds({ shippers: [], refineAsked: false, },),
                unjudgedRounds({ shippers: [SEAT_SYNTHETIC_TEXT_EVERYWHERE,], refineAsked: false, },),
              ],
            },);

            expect(printed.lines,).toEqual([
              '\nEDITORS SHIPPED on 3 of 4 slices, 3 of them with no editor round judged at all',
              `  ${SEAT_HYPER_VISION}: wrote shipping text on 2 of 3 slices`,
              `  ${SEAT_HYPER_TEXT_BEDROCK}: wrote shipping text on 1 of 3 slices`,
              `  ${SEAT_SYNTHETIC_TEXT_EVERYWHERE}: wrote shipping text on 1 of 3 slices`,
              NOT_A_PREFERENCE,
            ],);
          },
        },),

        it({
          name: 'COUNTS A SLICE WHOSE EDITOR ROUND DREW NO BALLOT among those that shipped with no editor round '
            + 'judged, as a lone composite candidate leaves it, and prints the slice noun in the singular',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printEditorCalibrateShipped({
              perSlice: [
                {
                  ...unjudgedRounds({ shippers: [SEAT_HYPER_VISION,], refineAsked: false, },),
                  editor: [
                    {
                      producers: [],
                      ballots: [],
                    },
                  ],
                },
              ],
            },);

            expect(printed.lines,).toEqual([
              '\nEDITORS SHIPPED on 1 of 1 slice, 1 of them with no editor round judged at all',
              `  ${SEAT_HYPER_VISION}: wrote shipping text on 1 of 1 slice`,
              NOT_A_PREFERENCE,
            ],);
          },
        },),

        it({
          name: 'LEAVES A SLICE WHOSE EDITOR ROUND DREW A BALLOT out of those that shipped with no editor round '
            + 'judged',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printEditorCalibrateShipped({
              perSlice: [
                {
                  ...unjudgedRounds({ shippers: [SEAT_HYPER_VISION,], refineAsked: false, },),
                  editor: [
                    {
                      producers: [
                        {
                          kind: 'model',
                          modelId: SEAT_HYPER_VISION,
                        },
                      ],
                      ballots: [
                        {
                          modelId: SEAT_HYPER_TEXT_BEDROCK,
                          best: 1,
                          reason: 'scripted',
                          weight: 1,
                          selfVote: false,
                        },
                      ],
                    },
                  ],
                },
              ],
            },);

            expect(printed.lines,).toEqual([
              '\nEDITORS SHIPPED on 1 of 1 slice, 0 of them with no editor round judged at all',
              `  ${SEAT_HYPER_VISION}: wrote shipping text on 1 of 1 slice`,
              NOT_A_PREFERENCE,
            ],);
          },
        },),
      ],
    },),
  ],
},);
