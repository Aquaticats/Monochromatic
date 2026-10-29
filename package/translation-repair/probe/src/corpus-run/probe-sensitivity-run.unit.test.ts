/**
 Tests for the probe sensitivity run over a scripted client, in which no
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

import {
  NO_DEFECT_CHECK,
  probeScriptedClient,
} from '@monochromatic-dev/translation-repair-run/ts/introduced-defect-scripted-client.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { RUN_MODELS, } from '@monochromatic-dev/translation-repair-run';
import {
  runSensitivity,
  type SensitivityArm,
  sensitivityNotes,
} from '../../dist/final/node/index.mjs';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Replaced wording of the region both arms ask about.
 */
const BEFORE = 'The cat is doing the sleeping, and she wakes at dusk.';

/**
 Arm asking about the nap edit under the withheld list.
 */
const WITHHELD_ARM: SensitivityArm = {
  region: {
    envelopeId: 'envelope/nap',
    issueIds: [],
    before: BEFORE,
    editorAfter: 'The cat sleeps.',
  },
  expectation: 'damage-omission',
  list: 'withheld',
  issue: 'prior',
  issues: [],
  disclosure: 'withheld',
  editKind: 'accuracy-repair',
  baselineText: BEFORE,
};

/**
 The same edit asked about under no list at all.
 */
const NONE_ARM: SensitivityArm = {
  ...WITHHELD_ARM,
  expectation: 'no-damage',
  list: 'none',
  issue: 'none',
  disclosure: 'withheld',
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runSensitivity.name,
      concurrency: 1,
      children: [
        it({
          name: 'ASKS one arm with every prober finding nothing, printing the opening, one tally line and the notes',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Model ids the client was asked.
             */
            const asked: string[] = [];

            await runSensitivity({
              client: probeScriptedClient({
                checkFor: function none() {
                  return NO_DEFECT_CHECK;
                },
                asked,
              },),
              arms: [WITHHELD_ARM,],
              production: 'withheld',
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(printed.lines,).toEqual([
              'SENSITIVITY production sends list=withheld; 1 arm follows',
              `SENSITIVITY envelope/nap list=withheld issue=prior expected=damage-omission heard=${
                String(asked.length,)
              }/${String(RUN_MODELS.checkerModelIds.length,)} corroborated=0 removal=0 contradicted=0 `
                + `unanchored=0 preExisting=0 noneFound=${String(asked.length,)} uncertain=0`,
              ...sensitivityNotes({ production: 'withheld', },),
            ],);
          },
        },),

        it({
          name: 'ASKS every arm in order with a claim line for each prober that claimed, and the arms in the plural',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Model ids the client was asked, over both arms.
             */
            const asked: string[] = [];

            await runSensitivity({
              client: probeScriptedClient({
                checkFor: function dropsClause() {
                  return {
                    verdict: 'introduced-defect',
                    category: 'accuracy/omission',
                    severity: 'major',
                    evidence: '',
                    omittedText: 'she wakes at dusk',
                    verdictReason: 'the clause is gone',
                  };
                },
                asked,
              },),
              arms: [
                WITHHELD_ARM,
                NONE_ARM,
              ],
              production: 'none',
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            /**
             Probers each arm heard, which is every prober it asked.
             */
            const heard = asked.length / 2;

            /**
             Claim lines one arm prints: one per prober.
             */
            const claims = Array.from(
              { length: heard, },
              function claimLine(): string {
                return '  claim removal-corroborated (accuracy/omission/major)';
              },
            );

            expect(printed.lines,).toEqual([
              'SENSITIVITY production sends list=none; 2 arms follow',
              `SENSITIVITY envelope/nap list=withheld issue=prior expected=damage-omission heard=${
                String(heard,)
              }/${String(RUN_MODELS.checkerModelIds.length,)} corroborated=0 removal=${
                String(heard,)
              } contradicted=0 unanchored=0 preExisting=0 noneFound=0 uncertain=0`,
              ...claims,
              `SENSITIVITY envelope/nap list=none issue=none expected=no-damage heard=${
                String(heard,)
              }/${String(RUN_MODELS.checkerModelIds.length,)} corroborated=0 removal=${
                String(heard,)
              } contradicted=0 unanchored=0 preExisting=0 noneFound=0 uncertain=0`,
              ...claims,
              ...sensitivityNotes({ production: 'none', },),
            ],);
          },
        },),
      ],
    },),
  ],
},);
