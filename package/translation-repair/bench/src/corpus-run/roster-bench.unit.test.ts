/**
 Boundary tests for the roster bench command: the built command, run in a
 child whose environment carries no provider key, so it can never reach a
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

import { RUN_ROSTER, } from '@monochromatic-dev/translation-repair-run';
import { benchWidths, } from '../../dist/final/node/index.mjs';
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

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'roster-bench as built',
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
              command: 'roster-bench',
              packageEntry: BENCH_ENTRY,
              args: ['--bogus',],
              env: world.env,
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe('roster-bench: --bogus is not a flag this command reads. Usage: roster-bench [<slices>]\n',);
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
              command: 'roster-bench',
              packageEntry: BENCH_ENTRY,
              args: ['0',],
              env: world.env,
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe('roster-bench: slices must be at least 1, and 0 is not\n',);
          },
        },),

        it({
          name: 'REFUSES as stated and exits 6 when launched without any provider key, after naming the widths it would run',
          fn: async () => {
            await using world = await benchWorld({ entries: { mittens: MITTENS, }, },);

            /**
             Widths and repeated width of the run roster, as the command
             reads them.
             */
            const { widths, repeated, } = benchWidths({ roster: RUN_ROSTER, },);

            /**
             What the command wrote over a one-entry clone, keys withheld as
             in every case of this suite.
             */
            const run = await runBuiltCommand({
              command: 'roster-bench',
              packageEntry: BENCH_ENTRY,
              env: world.env,
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe(
              `BENCH 1 slice, widths ${widths.join(', ',)}, width ${String(repeated,)} run twice, `
                + `roster of ${String(RUN_ROSTER.length,)}\n`,
            );
            expect(run.stderr,).toBe(noKeyRefusal({ command: 'roster-bench', },),);
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
              command: 'roster-bench',
              packageEntry: BENCH_ENTRY,
              env: world.env,
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(`roster-bench: ${NO_ENTRY_NO_SLICE_REFUSAL}\n`,);
          },
        },),
      ],
    },),
  ],
},);
