/**
 Tests the reports a run prints when it ends, which today is one: what the
 account's plan energy did between the run's first NeuralWatt call and its end
 (`neuralwatt-quota.ts` in the provider package says what that figure can and
 cannot see).

 The registry is process-wide, because a run client is built far from the
 command that ends the run. Every case drains it first, so no case reads
 another's report.

 Fixtures are invented; no case calls a provider.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { createSeatTally, } from '@monochromatic-dev/translation-repair-stage';
import {
  registerRunEndReport,
  reportingRefusals,
  runEndReports,
} from '../../dist/final/node/index.mjs';
import {
  BARE_ARGV,
  collectingErrors,
  holdingExitCode,
  NO_KEYS,
} from './cli-refusal-cases.test-fixture.ts';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: registerRunEndReport.name,
      concurrency: 1,
      children: [
        it({
          name: 'RUNS EACH REGISTERED REPORT ONCE, in registration order, and EMPTIES THE REGISTRY so a second run '
            + 'in the same process does not repeat the first one\'s lines',
          fn: async () => {
            await runEndReports();
            registerRunEndReport({ key: 'first', report: async () => 'first line', },);
            registerRunEndReport({ key: 'second', report: async () => 'second line', },);
            expect(await runEndReports(),).toEqual(['first line', 'second line',],);
            expect(await runEndReports(),).toEqual([],);
          },
        },),

        it({
          name: 'KEEPS ONE REPORT PER KEY, the later replacing the earlier, since one client registers itself once '
            + 'however many times a run builds it',
          fn: async () => {
            await runEndReports();
            registerRunEndReport({ key: 'same', report: async () => 'old line', },);
            registerRunEndReport({ key: 'same', report: async () => 'new line', },);
            expect(await runEndReports(),).toEqual(['new line',],);
          },
        },),

        it({
          name: 'SAYS A REPORT THAT FAILED, by its key and the name of the failure and never the failure\'s words, '
            + 'and goes on to the reports after it',
          fn: async () => {
            await runEndReports();
            registerRunEndReport({
              key: 'broken',
              report: async () => {
                throw new TypeError('the account number is 12345',);
              },
            },);
            registerRunEndReport({ key: 'fine', report: async () => 'fine line', },);
            /**
             Lines the two reports gave.
             */
            const lines = await runEndReports();
            expect(lines.length,).toBe(2,);
            expect(lines[0],).toContain('broken',);
            expect(lines[0],).toContain('TypeError',);
            expect(lines[0],).not.toContain('12345',);
            expect(lines[1],).toBe('fine line',);
          },
        },),
      ],
    },),

    describe({
      name: `${reportingRefusals.name} and the end reports`,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS THE END REPORTS BEFORE THE SEAT REPORT, when the command finishes and when it refuses',
          fn: async () => {
            using held = holdingExitCode();
            using printed = collectingErrors({ lines: [], },);
            await runEndReports();
            registerRunEndReport({ key: 'plan', report: async () => 'plan energy line', },);

            await reportingRefusals({
              what: 'score-verify',
              argv: BARE_ARGV,
              env: NO_KEYS,
              run: async () => undefined,
              seats: createSeatTally(),
            },);
            expect(printed.lines.includes('plan energy line',),).toBe(true,);
            expect(await runEndReports(),).toEqual([],);

            registerRunEndReport({ key: 'plan', report: async () => 'plan energy line again', },);
            await reportingRefusals({
              what: 'score-verify',
              argv: BARE_ARGV,
              env: NO_KEYS,
              run: async () => {
                throw new Error('a tabby walked across the keyboard',);
              },
              seats: createSeatTally(),
            },);
            expect(printed.lines.includes('plan energy line again',),).toBe(true,);
            void held;
          },
        },),
      ],
    },),
  ],
},);
