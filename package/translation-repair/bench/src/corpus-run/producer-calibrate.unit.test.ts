/**
 Boundary tests for the producer calibration command: the built command, run
 in a child whose environment carries no provider key, so it can never reach a
 model, and whose runs directory, lookup cache and corpus clone are throwaway
 places the case made.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  readHeadSha,
  RUN_ROSTER,
} from '@monochromatic-dev/translation-repair-run';
import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { BENCH_ENTRY, } from './built-entry.test-fixture.ts';
import {
  type BenchPages,
  benchWorld,
  NO_ENTRY_NO_SLICE_REFUSAL,
} from '@monochromatic-dev/translation-repair-adjudication/ts/corpus-run/bench-world.test-fixture.ts';
import { noKeyRefusal, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/keyless-built-command.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 One invented entry, one section against one.
 */
const MITTENS: BenchPages = {
  source: '## 窗台\n\n小猫在窗台上打盹。它的尾巴垂在地板上。\n',
  english: '## The windowsill\n\nThe kitten dozes on the windowsill. Its tail hangs to the floor.\n',
};

/**
 What the command's usage line says, which every refused argument ends with.
 */
const USAGE = 'Usage: producer-calibrate [--candidates <seatable ids>] [--candidates-alone] [<slices>]';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'producer-calibrate as built',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES a flag it does not read as stated and exits 6, naming the flag and the usage',
          fn: async () => {
            await using world = await benchWorld({ entries: { mittens: MITTENS, }, },);

            /**
             What the command wrote given one unknown flag, with the child's
             keys withheld by the shared fixture's removal of every
             `_API_KEY` variable.
             */
            const run = await runBuiltCommand({
              command: 'producer-calibrate',
              packageEntry: BENCH_ENTRY,
              args: ['--bogus',],
              env: world.env,
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(`producer-calibrate: --bogus is not a flag this command reads. ${USAGE}\n`,);
          },
        },),

        it({
          name: 'REFUSES a slice count below one as stated and exits 6, before the corpus is read',
          fn: async () => {
            await using world = await benchWorld({ entries: { mittens: MITTENS, }, },);

            /**
             What the command wrote given a count of zero, keys withheld as in
             every case of this suite.
             */
            const run = await runBuiltCommand({
              command: 'producer-calibrate',
              packageEntry: BENCH_ENTRY,
              args: ['0',],
              env: world.env,
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe('producer-calibrate: slices must be at least 1, and 0 is not\n',);
          },
        },),

        it({
          name: 'REFUSES --candidates-alone with no candidate named as stated and exits 6',
          fn: async () => {
            await using world = await benchWorld({ entries: { mittens: MITTENS, }, },);

            /**
             What the command wrote given the switch alone, keys withheld as
             in every case of this suite.
             */
            const run = await runBuiltCommand({
              command: 'producer-calibrate',
              packageEntry: BENCH_ENTRY,
              args: ['--candidates-alone',],
              env: world.env,
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'producer-calibrate: --candidates-alone runs the candidates without the seated roster, '
                + 'and --candidates named none\n',
            );
          },
        },),

        it({
          name: 'REFUSES as stated and exits 6 when launched without any provider key, before any call',
          fn: async () => {
            await using world = await benchWorld({ entries: { mittens: MITTENS, }, },);

            /**
             What the command wrote with every provider key withheld by the
             shared fixture, over a one-entry clone.
             */
            const run = await runBuiltCommand({
              command: 'producer-calibrate',
              packageEntry: BENCH_ENTRY,
              env: world.env,
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe(
              `CALIBRATE 1 slice, all ${String(RUN_ROSTER.length,)} writing and all ${String(RUN_ROSTER.length,)} judging `
                + `(${RUN_ROSTER.join(', ',)}), at ${await readHeadSha()}\n`,
            );
            expect(run.stderr,).toBe(noKeyRefusal({ command: 'producer-calibrate', },),);
          },
        },),

        it({
          name: 'REFUSES A CORPUS THAT YIELDS NO SLICE as stated and exits 6, with the draw\'s own sentence and '
            + 'nothing on stdout, where it once printed the refusal as a fault in the command',
          fn: async () => {
            await using world = await benchWorld({ entries: {}, },);

            /**
             What the command wrote over a clone holding no entry, keys
             withheld as in every case of this suite.
             */
            const run = await runBuiltCommand({
              command: 'producer-calibrate',
              packageEntry: BENCH_ENTRY,
              env: world.env,
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(`producer-calibrate: ${NO_ENTRY_NO_SLICE_REFUSAL}\n`,);
          },
        },),
      ],
    },),
  ],
},);
