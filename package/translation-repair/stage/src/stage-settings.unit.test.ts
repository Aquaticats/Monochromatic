/**
 Tests for the settings a launch fixes for every stage round it runs: both
 straggler windows and the contest ledger, read once from the environment an
 entry file hands in.

 EVERY CASE HANDS ITS OWN ENVIRONMENT, an object, and none writes the
 process's: the readers take the environment as a value, which is the whole
 point of moving the reads to the entries.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { readContestLedger, } from '@monochromatic-dev/translation-repair-house-rule';
import {
  MAX_TIMER_DELAY_MS,
  StatedRefusalError,
  WRITER_GRACE_MS,
} from '@monochromatic-dev/translation-repair-glossary';
import {
  readStageSettings,
  type StageSettings,
  STRAGGLER_GRACE_MS,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: readStageSettings.name,
      children: [
        it({
          name: 'GIVES every round the built-in window, the writers their longer built-in, and keeps no contest, '
            + 'for an environment that sets none of the three',
          fn: async () => {
            expect(readStageSettings({ env: {}, },),).toStrictEqual({
              grace: {
                writerMs: WRITER_GRACE_MS,
                roundMs: STRAGGLER_GRACE_MS,
                source: 'built-in',
              },
              contestLedger: { kind: 'unkept', },
            },);
          },
        },),

        it({
          name: 'READS both window dials and the runs directory from the environment it is handed',
          fn: async () => {
            expect(readStageSettings({
              env: {
                TRANSLATION_REPAIR_STRAGGLER_GRACE_MS: '90000',
                TRANSLATION_REPAIR_WRITER_GRACE_MS: '240000',
                TRANSLATION_REPAIR_RUNS_DIR: '/cattery/runs',
              },
            },),).toStrictEqual({
              grace: {
                writerMs: 240_000,
                roundMs: 90_000,
                source: 'writer-dial',
              },
              contestLedger: {
                kind: 'kept',
                runsDir: '/cattery/runs',
              },
            },);
          },
        },),

        it({
          name: 'REFUSES a round dial nothing can read as a stated refusal naming that dial',
          fn: async () => {
            /**
             What the read threw on an unreadable round dial.
             */
            const refusal = caught(function readUnit(): StageSettings {
              return readStageSettings({ env: { TRANSLATION_REPAIR_STRAGGLER_GRACE_MS: '2m', }, },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: TRANSLATION_REPAIR_STRAGGLER_GRACE_MS must be a whole number of milliseconds '
                + `from 1 to ${String(MAX_TIMER_DELAY_MS,)}, and "2m" is not; leave it unset to run under the `
                + 'built-in window',
            );
          },
        },),
      ],
    },),

    describe({
      name: readContestLedger.name,
      children: [
        it({
          name: 'KEEPS NO CONTEST when the runs directory variable is unset, which every unit run and every probe '
            + 'launched without one is',
          fn: async () => {
            expect(readContestLedger({ env: {}, },),).toStrictEqual({ kind: 'unkept', },);
          },
        },),

        it({
          name: 'KEEPS NO CONTEST when the runs directory variable is set empty, as the recorder always read it',
          fn: async () => {
            expect(readContestLedger({ env: { TRANSLATION_REPAIR_RUNS_DIR: '', }, },),).toStrictEqual({
              kind: 'unkept',
            },);
          },
        },),

        it({
          name: 'KEEPS every contest under the runs directory the variable names, as written',
          fn: async () => {
            expect(readContestLedger({ env: { TRANSLATION_REPAIR_RUNS_DIR: '/cattery/runs', }, },),).toStrictEqual({
              kind: 'kept',
              runsDir: '/cattery/runs',
            },);
          },
        },),
      ],
    },),
  ],
},);
