/**
 Tests for the compiled `editor-fidelity-probe` command's zero-call paths: what it does at a cap of zero over a clone that
 holds no corpus, and which requests it refuses before any corpus or provider access, with which exit status and stderr.

 Each run is the built command in an owned empty directory with no provider key set, so no case can spend. Fixtures are
 cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { PROBE_ENTRY, } from './built-entry.test-fixture.ts';
import { withoutStamp, } from '../fidelity-reference.test-fixture.ts';
import { LOW_CARD, } from '../editor-fidelity.test-fixture.ts';

/**
 Exit status the command gives a refusal it states in its own words.
 */
const REFUSED_AS_STATED = 6;

/**
 What a run left behind.
 */
type BuiltRun = {
  /**
   Exit status, zero for an accepted request.
   */
  readonly code: number;

  /**
   Standard output, one entry per line, each log line without its timestamp.
   */
  readonly stdout: readonly string[];

  /**
   Standard error, one entry per line.
   */
  readonly stderr: readonly string[];
};

/**
 Runs the built command with the arguments given, in an owned empty directory that is also the corpus clone.

 @param args - every argument after the script path

 @returns Exit status and both output streams, one entry per line

 @example
 ```ts
 const run = await runAsBuilt({ args: ['--cap', '0',], },);
 ```
 */
async function runAsBuilt({ args, }: { readonly args: readonly string[]; },): Promise<BuiltRun> {
  await using owned = await scratchDir({ prefix: 'editor-fidelity-as-built-', },);
  /**
   What the built command did, whatever its exit code; the fixture removes every provider key from its environment.
   */
  const run = await runBuiltCommand({
    command: 'editor-fidelity-probe',
    packageEntry: PROBE_ENTRY,
    args,
    cwd: owned.path,
    env: {
      TRANSLATION_REPAIR_RUNS_DIR: owned.path,
      TRANSLATION_REPAIR_CORPUS_CLONE_DIR: owned.path,
    },
  },);
  return {
    code: run.code,
    stdout: run.stdout.split('\n',)
      .filter((line,) => line !== '',)
      .map((line,) => withoutStamp({ line, },),),
    stderr: run.stderr.split('\n',)
      .filter((line,) => line !== '',),
  };
}

await describe({
  name: 'editor-fidelity-probe as built',
  children: [
    it({
      name: 'READS THE CORPUS AT A CAP OF ZERO, so over a clone that holds none it stops there with a nonzero status and '
        + 'no preflight line, having called nothing and needing no key',
      fn: async () => {
        const run = await runAsBuilt({
          args: ['--cap', '0', '--bench-spent-usd', '0', '--candidates', LOW_CARD, '--candidates-alone',],
        },);
        expect(run.code,).not.toBe(0,);
        expect(run.stdout.map(function plain(line,): string {
          return line.replace('[editor-fidelity-probe] ', '',);
        },),).toEqual([`[info] editors: ${LOW_CARD}`,],);
        expect(run.stderr.length,).toBeGreaterThan(0,);
        expect(run.stderr.some(function names(line,): boolean {
          return line.includes('preflight only',);
        },),).toBe(false,);
      },
    },),

    it({
      name: 'REFUSES A LINE WITH NO --bench-spent-usd at exit six with one line, since the arm is not told what the '
        + 'earlier arms spent',
      fn: async () => {
        const run = await runAsBuilt({
          args: ['--cap', '0', '--candidates', LOW_CARD, '--candidates-alone',],
        },);
        expect(run.code,).toBe(REFUSED_AS_STATED,);
        expect(run.stdout,).toStrictEqual([],);
        expect(run.stderr.length,).toBe(1,);
        expect(run.stderr[0],).toContain('editor-fidelity-probe: --bench-spent-usd is required: ',);
      },
    },),

    it({
      name: 'REFUSES A CARD THE ROSTER DOES NOT KNOW at exit six with one line, in the judge probe\'s wording',
      fn: async () => {
        const run = await runAsBuilt({
          args: ['--cap', '0', '--bench-spent-usd', '0', '--candidates', 'nobody/such-model', '--candidates-alone',],
        },);
        expect(run.code,).toBe(REFUSED_AS_STATED,);
        expect(run.stderr.length,).toBe(1,);
        expect(run.stderr[0],).toContain(
          'editor-fidelity-probe: --candidates names "nobody/such-model", which is not seatable; the roster knows ',
        );
      },
    },),

    it({
      name: 'REFUSES A RUN THAT DOES NOT SAY ALONE, and a flag the judge probe reads and this one does not',
      fn: async () => {
        const unaccompanied = await runAsBuilt({
          args: ['--cap', '0', '--bench-spent-usd', '0', '--candidates', LOW_CARD,],
        },);
        expect(unaccompanied.code,).toBe(REFUSED_AS_STATED,);
        expect(unaccompanied.stderr,).toStrictEqual([
          'editor-fidelity-probe: this probe seats only the cards --candidates names; write '
            + '--candidates-alone to say so',
        ],);
        const withContext = await runAsBuilt({
          args: ['--cap', '0', '--bench-spent-usd', '0', '--candidates', LOW_CARD, '--candidates-alone', '--context',],
        },);
        expect(withContext.code,).toBe(REFUSED_AS_STATED,);
        expect(withContext.stderr[0],).toContain('editor-fidelity-probe: ',);
      },
    },),
  ],
},);
