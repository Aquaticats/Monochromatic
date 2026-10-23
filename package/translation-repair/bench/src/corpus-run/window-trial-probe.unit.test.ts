/**
 Boundary test for the window trial command.

 The command spends quota and is composition over modules with their own
 suites, so what is checked here is the one thing only the built command can
 show: launched without both provider keys it refuses as stated, exits 6, and
 never reaches a call. The environment handed to the child carries no key, so
 the case cannot spend anything whatever the runner's own environment holds.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  readHeadSha,
  readRunCorpusPinSetting,
} from '@monochromatic-dev/translation-repair-run';
import { protocolDigest, } from '../../dist/final/node/index.mjs';
import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { BENCH_ENTRY, } from './built-entry.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 What the built command wrote and how it exited.

 @example
 ```ts
 const run: CommandRun = { code: 6, stderr: 'window-trial-probe: ...', };
 ```
 */
type CommandRun = {
  /**
   Exit code, or -1 when the process was signalled.
   */
  readonly code: number;

  /**
   Everything written to stderr.
   */
  readonly stderr: string;
};

/**
 Runs the built command with every provider key withheld and a disposable
 runs directory.

 @returns Exit code and stderr

 @example
 ```ts
 const run = await runWithoutKeys();
 ```
 */
async function runWithoutKeys(): Promise<CommandRun> {
  /**
   Throwaway runs directory the child points at, removed once this function's
   `await using` scope ends (after the child's streams close).
   */
  await using scratch = await scratchDir({ prefix: 'window-trial-probe-', },);

  /**
   The command as it ended; the shared fixture removes every provider key
   from the child's environment, so the child can neither refuse for the
   wrong reason nor spend.
   */
  const run = await runBuiltCommand({
    command: 'window-trial-probe',
    packageEntry: BENCH_ENTRY,
    env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
  },);

  return run;
}

await describe({
  name: 'window-trial-probe',
  children: [
    it({
      name: 'REFUSES as stated and exits 6 when launched without both provider keys, before any call',
      fn: async () => {
        const run = await runWithoutKeys();

        expect(run.code,).toBe(REFUSED_AS_STATED,);
        expect(run.stderr,).toContain('window-trial-probe: ',);
        expect(run.stderr,).toContain('_API_KEY is not set',);
      },
    },),
    it({
      name: 'REFUSES as stated at exit 6, whole, after the ledger read and before any call, with no key in the child',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-probe-whole-', },);

        /**
         Digest the run buys under at this checkout's head and the built-in
         pin, which the child reads with no corpus setting in its environment,
         and which the opening line names the first twelve characters of.
         */
        const protocol = protocolDigest({
          headSha: await readHeadSha(),
          pin: readRunCorpusPinSetting({ env: {}, },).pin,
        },);

        // The shared fixture removes every variable ending in `_API_KEY`.
        const run = await runBuiltCommand({
          command: 'window-trial-probe',
          packageEntry: BENCH_ENTRY,
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run.code,).toBe(REFUSED_AS_STATED,);
        /**
         Words of the opening line, the second of which is its time.
         */
        const words = run.stdout.split(' ',);
        expect(words.with(
          1,
          '[TIME]',
        ).join(' ',),).toBe(`[info] [TIME] [window-trial] protocol ${ protocol.slice(
          0,
          12,
        ) }; 0 arms already bought\n`,);
        expect(run.stderr,).toBe(
          'window-trial-probe: TRANSLATION_REPAIR_NEURALWATT_API_KEY, TRANSLATION_REPAIR_SYNTHETIC_API_KEY, '
            + 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY, '
            + 'TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY, DEEPSEEK_API_KEY, TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY, TRANSLATION_REPAIR_CLAUDE_API_KEY '
            + 'or TRANSLATION_REPAIR_OPENROUTER_API_KEY is not set; '
            + 'run under mise so sops injects it\n',
        );
      },
    },),
    it({
      name: 'REFUSES a flag the command does not declare, as stated, at exit 6 with nothing on stdout',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-probe-flag-', },);

        // The shared fixture removes every variable ending in `_API_KEY`.
        const run = await runBuiltCommand({
          command: 'window-trial-probe',
          packageEntry: BENCH_ENTRY,
          args: ['--bogus',],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: 6,
          stdout: '',
          stderr: 'window-trial-probe: --bogus is not a flag this command reads. Usage: window-trial-probe\n',
        },);
      },
    },),
  ],
},);
