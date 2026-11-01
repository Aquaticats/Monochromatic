/**
 Tests for the checker sensitivity run over scripted clients, in which no
 model is ever called.

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

import { RUN_MODELS, } from '@monochromatic-dev/translation-repair-run';
import {
  checkAllFixedSheet,
  CHECKER_NOTE,
  checkMixedSheet,
  checkOne,
  runCheckerSensitivity,
} from '../../dist/final/node/index.mjs';
import { checkerScriptedClient, } from '@monochromatic-dev/translation-repair-provider/ts/checker-scripted-client.test-fixture.ts';
import { successiveClients, } from '@monochromatic-dev/translation-repair-run/ts/introduced-defect-scripted-client.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Checkers the roster seats, which every scripted client answers for.
 */
const SEATS = String(RUN_MODELS.checkerModelIds.length,);

/**
 Builds a client in which every checker casts the same verdicts.

 @param verdicts - verdicts every checker casts, one per issue of the sheet

 @returns Scripted client

 @example
 ```ts
 const client = everyCheckerCasts({ verdicts: ['fixed',], },);
 ```
 */
function everyCheckerCasts(
  { verdicts, }: { readonly verdicts: readonly string[]; },
): ReturnType<typeof checkerScriptedClient> {
  return checkerScriptedClient({
    verdictsFor: function same() {
      return verdicts;
    },
    asked: [],
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: checkOne.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS one line with the checkers heard and the tally of the one issue when every checker says fixed',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            await checkOne({
              label: 'genuinely-fixed',
              patchedText: 'The cat sleeps.',
              expectation: 'fixed',
              client: everyCheckerCasts({ verdicts: ['fixed',], },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(printed.lines,).toEqual([
              `CHECKER genuinely-fixed expected=fixed heard=${SEATS} fixed=${SEATS} notFixed=0 worse=0 `
                + 'resolved=true regressed=false',
            ],);
          },
        },),

        it({
          name: 'PRINTS resolved false when every checker says not fixed and regressed true when every one says worse',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            await checkOne({
              label: 'untouched',
              patchedText: 'The cat is doing the sleeping.',
              expectation: 'not-fixed',
              client: everyCheckerCasts({ verdicts: ['not-fixed',], },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            await checkOne({
              label: 'fixed-but-damaged',
              patchedText: 'The cat sleeps.',
              expectation: 'fixed-or-worse',
              client: everyCheckerCasts({ verdicts: ['worse',], },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(printed.lines,).toEqual([
              `CHECKER untouched expected=not-fixed heard=${SEATS} fixed=0 notFixed=${SEATS} worse=0 `
                + 'resolved=false regressed=false',
              `CHECKER fixed-but-damaged expected=fixed-or-worse heard=${SEATS} fixed=0 notFixed=0 worse=${SEATS} `
                + 'resolved=false regressed=true',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: checkMixedSheet.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS one line per issue of the sheet with its own expectation and the verdict each issue drew',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            await checkMixedSheet({
              client: everyCheckerCasts({
                verdicts: [
                  'fixed',
                  'not-fixed',
                  'worse',
                ],
              },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(printed.lines,).toEqual([
              `CHECKER mixed-sheet/adjudicated/tense expected=fixed fixed=${SEATS} notFixed=0 worse=0 resolved=true`,
              `CHECKER mixed-sheet/adjudicated/meaning expected=not-fixed fixed=0 notFixed=${SEATS} worse=0 `
                + 'resolved=false',
              `CHECKER mixed-sheet/adjudicated/absent expected=not-fixed-defect-was-never-there fixed=0 notFixed=0 `
                + `worse=${SEATS} resolved=false`,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: checkAllFixedSheet.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS one line per issue expecting fixed for each, with the tallies the checkers cast',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            await checkAllFixedSheet({
              client: everyCheckerCasts({
                verdicts: [
                  'fixed',
                  'fixed',
                  'not-fixed',
                ],
              },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(printed.lines,).toEqual([
              `CHECKER all-fixed/adjudicated/tense expected=fixed fixed=${SEATS} notFixed=0 worse=0 resolved=true`,
              `CHECKER all-fixed/adjudicated/meaning expected=fixed fixed=${SEATS} notFixed=0 worse=0 resolved=true`,
              `CHECKER all-fixed/adjudicated/absent expected=fixed fixed=0 notFixed=${SEATS} worse=0 resolved=false`,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: runCheckerSensitivity.name,
      concurrency: 1,
      children: [
        it({
          name: 'ASKS the three single cases, the mixed sheet and the all-fixed sheet, one fresh client each, then prints the note',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Builder over one client per call and the count it handed out.
             */
            const { newClient, built, } = successiveClients({
              clients: [
                everyCheckerCasts({ verdicts: ['fixed',], },),
                everyCheckerCasts({ verdicts: ['not-fixed',], },),
                everyCheckerCasts({ verdicts: ['worse',], },),
                everyCheckerCasts({
                  verdicts: [
                    'fixed',
                    'not-fixed',
                    'not-fixed',
                  ],
                },),
                everyCheckerCasts({
                  verdicts: [
                    'fixed',
                    'fixed',
                    'fixed',
                  ],
                },),
              ],
            },);

            await runCheckerSensitivity({ newClient, stageSettings: SCRIPTED_STAGE_SETTINGS, },);

            expect(built(),).toBe(5,);
            expect(printed.lines,).toEqual([
              `CHECKER genuinely-fixed expected=fixed heard=${SEATS} fixed=${SEATS} notFixed=0 worse=0 `
                + 'resolved=true regressed=false',
              `CHECKER untouched expected=not-fixed heard=${SEATS} fixed=0 notFixed=${SEATS} worse=0 `
                + 'resolved=false regressed=false',
              `CHECKER fixed-but-damaged expected=fixed-or-worse heard=${SEATS} fixed=0 notFixed=0 worse=${SEATS} `
                + 'resolved=false regressed=true',
              `CHECKER mixed-sheet/adjudicated/tense expected=fixed fixed=${SEATS} notFixed=0 worse=0 resolved=true`,
              `CHECKER mixed-sheet/adjudicated/meaning expected=not-fixed fixed=0 notFixed=${SEATS} worse=0 `
                + 'resolved=false',
              `CHECKER mixed-sheet/adjudicated/absent expected=not-fixed-defect-was-never-there fixed=0 `
                + `notFixed=${SEATS} worse=0 resolved=false`,
              `CHECKER all-fixed/adjudicated/tense expected=fixed fixed=${SEATS} notFixed=0 worse=0 resolved=true`,
              `CHECKER all-fixed/adjudicated/meaning expected=fixed fixed=${SEATS} notFixed=0 worse=0 resolved=true`,
              `CHECKER all-fixed/adjudicated/absent expected=fixed fixed=${SEATS} notFixed=0 worse=0 resolved=true`,
              CHECKER_NOTE,
            ],);
          },
        },),
      ],
    },),
  ],
},);
