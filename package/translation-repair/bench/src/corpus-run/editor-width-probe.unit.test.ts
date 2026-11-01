/**
 Tests for the editor width probe runner: the built command, run in a child
 process with every provider key withheld, so it can only refuse before any
 model is asked.

 THE CHILD'S ENVIRONMENT. `runBuiltCommand` drops every variable whose
 name ends in `_API_KEY` and every variable whose name starts
 `TRANSLATION_REPAIR_` before a case adds the locations it needs
 (`environmentWithoutKeys` in `child-environment.test-fixture.ts`), and every
 case hands it a world `benchWorld` made: a throwaway runs directory, lookup
 cache and corpus clone, so the draw the probe makes before it builds its
 client never reads the operator's own clone.

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
  RUN_MODELS,
  RUN_ROSTER,
} from '@monochromatic-dev/translation-repair-run';
import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { BENCH_ENTRY, } from './built-entry.test-fixture.ts';
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
 One invented entry, one section against one, which the draw slices once.
 */
const MITTENS: BenchPages = {
  source: '## 窗台\n\n小猫在窗台上打盹。它的尾巴垂在地板上。\n',
  english: '## The windowsill\n\nThe kitten dozes on the windowsill. Its tail hangs to the floor.\n',
};

/**
 Header the probe prints before it reads the draw, counting the run
 configuration's editors against the whole roster.
 */
const HEADER = `WIDTH narrow ${String(RUN_MODELS.editorModelIds.length,)} against wide ${
  String(RUN_ROSTER.length,)
}, panel of ${String(RUN_ROSTER.length,)} held fixed\n`;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'editor-width-probe as built',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES as stated and exits 6 for want of a provider key once the sample is drawn, after printing '
            + 'the header and the draw, and before any call',
          fn: async () => {
            await using world = await benchWorld({ entries: { mittens: MITTENS, }, },);

            /**
             What the command wrote over a one-entry clone, every key withheld
             by the shared fixture's removal of each `_API_KEY` variable.
             */
            const run = await runBuiltCommand({
              command: 'editor-width-probe',
              packageEntry: BENCH_ENTRY,
              args: [],
              env: world.env,
            },);

            expect({
              code: run.code,
              stdout: run.stdout,
              stderr: run.stderr,
            },).toEqual({
              code: REFUSED_AS_STATED,
              stdout: `${HEADER}WIDTH sample 1, draw A 1, other half held back\n`,
              stderr: noKeyRefusal({ command: 'editor-width-probe', },),
            },);
          },
        },),

        it({
          name: 'REFUSES DRAW B OF A ONE-SLICE SAMPLE as stated and exits 6 before any client is built, so the '
            + 'refusal that needs no provider is the one a keyless operator reads',
          fn: async () => {
            await using world = await benchWorld({ entries: { mittens: MITTENS, }, },);

            /**
             What the command wrote asked for draw B of one slice, keys
             withheld as in every case of this suite.
             */
            const run = await runBuiltCommand({
              command: 'editor-width-probe',
              packageEntry: BENCH_ENTRY,
              args: [
                '1',
                'b',
              ],
              env: world.env,
            },);

            expect({
              code: run.code,
              stdout: run.stdout,
              stderr: run.stderr,
            },).toEqual({
              code: REFUSED_AS_STATED,
              stdout: `${HEADER}WIDTH sample 1, draw B 0, other half held back\n`,
              stderr: 'editor-width-probe: editor width probe refused: draw B holds no slice of the 1 slice drawn, '
                + 'so spending it would buy the positive control and report nothing; ask for at least 2 slices\n',
            },);
          },
        },),

        it({
          name: 'REFUSES a flag it does not read as stated and exits 6, naming the flag and the usage',
          fn: async () => {
            await using world = await benchWorld({ entries: { mittens: MITTENS, }, },);

            /**
             What the command wrote given one flag it does not declare, keys
             withheld as in every case of this suite.
             */
            const run = await runBuiltCommand({
              command: 'editor-width-probe',
              packageEntry: BENCH_ENTRY,
              args: ['--bogus',],
              env: world.env,
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'editor-width-probe: --bogus is not a flag this command reads. Usage: editor-width-probe [<slices>] '
                + '[<draw>]\n',
            );
          },
        },),
      ],
    },),
  ],
},);
