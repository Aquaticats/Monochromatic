/**
 Tests for the producer calibration run over scripted slices and clients, in
 which no model is called and no corpus clone is read.

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

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { RunConfigError, } from '@monochromatic-dev/translation-repair-run';
import { runProducerCalibrate, } from '../../dist/final/node/index.mjs';
import { successiveClients, } from '@monochromatic-dev/translation-repair-run/ts/introduced-defect-scripted-client.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  SCRIPTED_HEAD,
  scriptedDrawing,
} from './scripted-bench-seams.test-fixture.ts';
import { translateLaneClient, } from '@monochromatic-dev/translation-repair-provider/ts/corpus-run/translate-lane-script.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Models the calibration runs, named after `--candidates` and run alone so the
 seated roster stays out of the case.
 */
const CANDIDATES = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_HYPER_VISION,
] as const;

/**
 Flags that run `CANDIDATES` alone, as typed.
 */
const ALONE = [
  '--candidates',
  CANDIDATES.join(',',),
  '--candidates-alone',
] as const;

/**
 Rendering the judges back in every case.
 */
const DOZING = 'The cat dozes on the windowsill, tail draped beside the radiator.';

/**
 What every writer answers.
 */
const RENDERINGS = {
  [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: DOZING,
  [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 'A cat naps on the sill, its tail hanging near the heater.',
  [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
} as const;

/**
 Runs the calibration over the candidates alone with no client to build,
 so every slice is lost.

 @param seams - scripted drawing and head reading

 @example
 ```ts
 await runWhereEveryClientFails({ seams, },);
 ```
 */
async function runWhereEveryClientFails(
  { seams, }: { readonly seams: ReturnType<typeof scriptedDrawing>['seams']; },
): Promise<void> {
  await runProducerCalibrate({
    line: lineOf({
      command: 'producer-calibrate',
      typed: [...ALONE,],
    },),
    newClient: successiveClients({ clients: [], },).newClient,
    ...seams,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runProducerCalibrate.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS the opening, a line per finished slice, the standing best first and the closing note, '
            + 'building one client per slice',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            const { asked, seams, } = scriptedDrawing({ slices: 2, },);
            const { newClient, built, } = successiveClients({
              clients: [
                translateLaneClient({
                  renderings: RENDERINGS,
                  needle: 'dozes',
                },),
                translateLaneClient({
                  renderings: RENDERINGS,
                  needle: 'dozes',
                },),
              ],
            },);

            await runProducerCalibrate({
              line: lineOf({
                command: 'producer-calibrate',
                typed: [
                  ...ALONE,
                  '2',
                ],
              },),
              newClient,
              ...seams,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(asked.draws,).toEqual([{ count: 2, },],);
            expect(built(),).toBe(2,);
            expect(printed.lines,).toEqual([
              `CALIBRATE 2 slices, all 3 writing and all 3 judging (${CANDIDATES.join(', ',)}), at ${SCRIPTED_HEAD}`,
              '  mittens#0: round 1 of 2',
              '  mittens#1: round 2 of 2',
              '\nSTANDING over 2 rounds, best first:',
              `  ${SEAT_HYPER_OPENROUTER_VISION_EDITOR}: 100.0% (4 of 4 disinterested ballots, over 2 candidates)`,
              `  ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER}: 0.0% (0 of 4 disinterested ballots, over 2 candidates)`,
              `  ${SEAT_HYPER_VISION}: 0.0% (0 of 4 disinterested ballots, over 2 candidates)`,
              '\nA lead smaller than its own denominator supports is not a lead.'
                + ' Read the counts before seating anyone.',
            ],);
          },
        },),

        it({
          name: 'SAYS one slice and one round in the singular and names the writer no candidate of reached a slate',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            const { seams, } = scriptedDrawing({ slices: 1, },);
            const { newClient, } = successiveClients({
              clients: [
                translateLaneClient({
                  renderings: {
                    [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: DOZING,
                    [SEAT_HYPER_VISION]: RENDERINGS[SEAT_HYPER_VISION],
                  },
                  needle: 'dozes',
                },),
              ],
            },);

            await runProducerCalibrate({
              line: lineOf({
                command: 'producer-calibrate',
                typed: [
                  ...ALONE,
                  '1',
                ],
              },),
              newClient,
              ...seams,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(printed.lines,).toEqual([
              `CALIBRATE 1 slice, all 3 writing and all 3 judging (${CANDIDATES.join(', ',)}), at ${SCRIPTED_HEAD}`,
              '  mittens#0: round 1 of 1',
              '\nSTANDING over 1 round, best first:',
              `  ${SEAT_HYPER_OPENROUTER_VISION_EDITOR}: 100.0% (2 of 2 disinterested ballots, over 1 candidate)`,
              `  ${SEAT_HYPER_VISION}: 0.0% (0 of 2 disinterested ballots, over 1 candidate)`,
              `  NO CANDIDATE OF THEIRS REACHED ANY SLATE: ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER}, so the table covers 2 of 3 seats. `
                + 'This seat does not record who answered, so a model that answered and was dropped before judging and a '
                + 'provider that failed every call look alike from here; the SEAT lines at the end of this command say how '
                + 'often each seat was asked and how many answers were usable, and the run log names the failure. Re-run '
                + 'these seats before reading the table as a comparison of the roster.',
              '\nA lead smaller than its own denominator supports is not a lead.'
                + ' Read the counts before seating anyone.',
            ],);
          },
        },),

        it({
          name: 'DRAWS ten slices when the command line names no count',
          fn: async (ctx,) => {
            using _printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            const { asked, seams, } = scriptedDrawing({ slices: 1, },);
            const { newClient, } = successiveClients({
              clients: [
                translateLaneClient({
                  renderings: RENDERINGS,
                  needle: 'dozes',
                },),
              ],
            },);

            await runProducerCalibrate({
              line: lineOf({
                command: 'producer-calibrate',
                typed: [...ALONE,],
              },),
              newClient,
              ...seams,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(asked.draws,).toEqual([{ count: 10, },],);
          },
        },),

        it({
          name: 'PRINTS LOST naming the error class for a slice whose round failed, and reports the rounds that finished',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            const { seams, } = scriptedDrawing({ slices: 2, },);
            const { newClient, } = successiveClients({
              clients: [
                translateLaneClient({
                  renderings: RENDERINGS,
                  needle: 'dozes',
                },),
              ],
            },);

            await runProducerCalibrate({
              line: lineOf({
                command: 'producer-calibrate',
                typed: [
                  ...ALONE,
                  '2',
                ],
              },),
              newClient,
              ...seams,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(printed.lines
              .slice(
              0,
              4,
              ),).toEqual([
                `CALIBRATE 2 slices, all 3 writing and all 3 judging (${CANDIDATES.join(', ',)}), at ${SCRIPTED_HEAD}`,
                '  mittens#0: round 1 of 2',
                '  mittens#1: LOST (Error)',
                '\nSTANDING over 1 round, best first:',
              ],);
          },
        },),

        it({
          name: 'REFUSES a count below one as stated, before the corpus is drawn or the head is read',
          fn: async () => {
            const { asked, seams, } = scriptedDrawing({ slices: 1, },);

            const refusal = await rejectionOf(async function runZero(): Promise<void> {
              await runProducerCalibrate({
                line: lineOf({
                  command: 'producer-calibrate',
                  typed: ['0',],
                },),
                newClient: successiveClients({ clients: [], },).newClient,
                ...seams,
                stageSettings: SCRIPTED_STAGE_SETTINGS,
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe('StatedRefusalError: slices must be at least 1, and 0 is not',);
            expect(asked,).toEqual({
              draws: [],
              heads: 0,
            },);
          },
        },),

        it({
          name: 'REFUSES --candidates-alone with no candidate named, before the corpus is drawn',
          fn: async () => {
            const { asked, seams, } = scriptedDrawing({ slices: 1, },);

            const refusal = await rejectionOf(async function runAloneOfNobody(): Promise<void> {
              await runProducerCalibrate({
                line: lineOf({
                  command: 'producer-calibrate',
                  typed: ['--candidates-alone',],
                },),
                newClient: successiveClients({ clients: [], },).newClient,
                ...seams,
                stageSettings: SCRIPTED_STAGE_SETTINGS,
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: --candidates-alone runs the candidates without the seated roster, and --candidates named none',
            );
            expect(asked.draws,).toEqual([],);
          },
        },),

        it({
          name: 'REFUSES a roster that cannot select anything as stated, before the corpus is drawn',
          fn: async () => {
            const { asked, seams, } = scriptedDrawing({ slices: 1, },);

            const refusal = await rejectionOf(async function runOneSeat(): Promise<void> {
              await runProducerCalibrate({
                line: lineOf({
                  command: 'producer-calibrate',
                  typed: [
                    '--candidates',
                    SEAT_HYPER_VISION,
                    '--candidates-alone',
                  ],
                },),
                newClient: successiveClients({ clients: [], },).newClient,
                ...seams,
                stageSettings: SCRIPTED_STAGE_SETTINGS,
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: this translator roster cannot select anything: judges seated: 1; ballots a winner '
                + 'needs: 2; so nothing it proposes could ever be selected. '
                + `translators [${SEAT_HYPER_VISION}], judges [${SEAT_HYPER_VISION}]`,
            );
            expect(asked,).toEqual({
              draws: [],
              heads: 0,
            },);
          },
        },),

        it({
          name: 'REFUSES as stated instead of printing LOST when a slice\'s client is refused for a missing key',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            const { seams, } = scriptedDrawing({ slices: 2, },);

            const refusal = await rejectionOf(async function runWithoutKey(): Promise<void> {
              await runProducerCalibrate({
                line: lineOf({
                  command: 'producer-calibrate',
                  typed: [...ALONE,],
                },),
                newClient: function refuseKey(): never {
                  throw new RunConfigError({ variable: 'TRANSLATION_REPAIR_SYNTHETIC_API_KEY', },);
                },
                ...seams,
                stageSettings: SCRIPTED_STAGE_SETTINGS,
              },);
            },);

            expect(refusal,).toBeInstanceOf(RunConfigError,);
            expect(String(refusal,),).toBe(
              'RunConfigError: TRANSLATION_REPAIR_SYNTHETIC_API_KEY is not set; run under mise so sops injects it',
            );
            expect(printed.lines,).toEqual([
              `CALIBRATE 2 slices, all 3 writing and all 3 judging (${CANDIDATES.join(', ',)}), at ${SCRIPTED_HEAD}`,
            ],);
          },
        },),

        it({
          name: 'REFUSES as stated when no slice finished a round, after naming each slice that was lost',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            const { seams, } = scriptedDrawing({ slices: 2, },);

            const refusal = await rejectionOf(async function runAllLost(): Promise<void> {
              await runWhereEveryClientFails({ seams, },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: 2 slices were drawn and no round finished, so the calibration measured nothing '
                + 'and prints no standing; the LOST lines name the error each slice was lost to',
            );
            expect(printed.lines,).toEqual([
              `CALIBRATE 2 slices, all 3 writing and all 3 judging (${CANDIDATES.join(', ',)}), at ${SCRIPTED_HEAD}`,
              '  mittens#0: LOST (Error)',
              '  mittens#1: LOST (Error)',
            ],);
          },
        },),

        it({
          name: 'SAYS one slice was drawn in the singular when the only slice was lost',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            const { seams, } = scriptedDrawing({ slices: 1, },);

            const refusal = await rejectionOf(async function runOneLost(): Promise<void> {
              await runWhereEveryClientFails({ seams, },);
            },);

            expect(String(refusal,),).toBe(
              'StatedRefusalError: 1 slice was drawn and no round finished, so the calibration measured nothing '
                + 'and prints no standing; the LOST lines name the error each slice was lost to',
            );
            expect(printed.lines,).toEqual([
              `CALIBRATE 1 slice, all 3 writing and all 3 judging (${CANDIDATES.join(', ',)}), at ${SCRIPTED_HEAD}`,
              '  mittens#0: LOST (Error)',
            ],);
          },
        },),
      ],
    },),
  ],
},);
