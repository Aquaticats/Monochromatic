/**
 Boundary tests for the recall benchmark command: the built command, run in a
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

import { readHeadSha, } from '@monochromatic-dev/translation-repair-run';
import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { PROBE_ENTRY, } from './built-entry.test-fixture.ts';
import {
  type BenchPages,
  benchWorld,
} from '@monochromatic-dev/translation-repair-adjudication/ts/corpus-run/bench-world.test-fixture.ts';
import { noKeyRefusal, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/keyless-built-command.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 One invented entry whose English page holds one sentence worth deleting.
 */
const MITTENS: BenchPages = {
  source: '## 窗台\n\n小猫在窗台上打盹。它的尾巴垂在地板上。\n',
  english: '## The windowsill\n\nThe kitten dozes on the windowsill. Its tail hangs to the floor.\n',
};

/**
 The whole opening line the command prints before it needs a client, for a
 corpus holding the one invented entry.

 @returns Line with its newline

 @example
 ```ts
 expect(run.stdout,).toBe(await startOfOneEntry(),);
 ```
 */
async function startOfOneEntry(): Promise<string> {
  return `START tip=${await readHeadSha()} entries=1 seeds=1 `
    + 'perBand={"small":1,"medium":0,"large":0} budget=43200000ms\n';
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'recall-benchmark as built',
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
              command: 'recall-benchmark',
              packageEntry: PROBE_ENTRY,
              args: ['--bogus',],
              env: world.env,
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe('recall-benchmark: --bogus is not a flag this command reads. Usage: recall-benchmark [--plan]\n',);
          },
        },),

        it({
          name: 'REFUSES as stated and exits 6 when launched without any provider key, after planning one entry',
          fn: async () => {
            await using world = await benchWorld({ entries: { mittens: MITTENS, }, },);

            /**
             What the command wrote over a one-entry clone, keys withheld as
             in every case of this suite.
             */
            const run = await runBuiltCommand({
              command: 'recall-benchmark',
              packageEntry: PROBE_ENTRY,
              env: world.env,
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe(await startOfOneEntry(),);
            expect(run.stderr,).toBe(noKeyRefusal({ command: 'recall-benchmark', },),);
          },
        },),

        it({
          name: 'REFUSES --plan for the missing key too, since the plan constructs the client it reports on',
          fn: async () => {
            await using world = await benchWorld({ entries: { mittens: MITTENS, }, },);

            /**
             What the command wrote given the plan switch, keys withheld as in
             every case of this suite.
             */
            const run = await runBuiltCommand({
              command: 'recall-benchmark',
              packageEntry: PROBE_ENTRY,
              args: ['--plan',],
              env: world.env,
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe(await startOfOneEntry(),);
            expect(run.stderr,).toBe(noKeyRefusal({ command: 'recall-benchmark', },),);
          },
        },),

        it({
          name: 'REFUSES A RUN over a corpus with nobody in it as stated, where --plan refuses and before it '
            + 'needs a key, rather than building a client that refuses for the missing key',
          fn: async () => {
            await using world = await benchWorld({ entries: {}, },);

            /**
             What the command wrote over an empty clone, keys withheld as in
             every case of this suite.
             */
            const run = await runBuiltCommand({
              command: 'recall-benchmark',
              packageEntry: PROBE_ENTRY,
              env: world.env,
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe(
              `START tip=${await readHeadSha()} entries=0 seeds=0 `
                + 'perBand={"small":0,"medium":0,"large":0} budget=43200000ms\n',
            );
            expect(run.stderr,).toBe(
              'recall-benchmark: no entry at the pin can be seeded, so the benchmark would plant no seed and '
                + 'measure nothing; check that the corpus clone and commit it reads hold entries with both pages and '
                + 'a sentence worth deleting\n',
            );
          },
        },),

        it({
          name: 'REFUSES --plan over a corpus with nobody in it as stated, before it needs a key',
          fn: async () => {
            await using world = await benchWorld({ entries: {}, },);

            /**
             What the command wrote given the plan switch over an empty
             clone, keys withheld as in every case of this suite.
             */
            const run = await runBuiltCommand({
              command: 'recall-benchmark',
              packageEntry: PROBE_ENTRY,
              args: ['--plan',],
              env: world.env,
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe(
              `START tip=${await readHeadSha()} entries=0 seeds=0 `
                + 'perBand={"small":0,"medium":0,"large":0} budget=43200000ms\n',
            );
            expect(run.stderr,).toBe(
              'recall-benchmark: no entry at the pin can be seeded, so the benchmark would plant no seed and '
                + 'measure nothing; check that the corpus clone and commit it reads hold entries with both pages and '
                + 'a sentence worth deleting\n',
            );
          },
        },),
      ],
    },),
  ],
},);
