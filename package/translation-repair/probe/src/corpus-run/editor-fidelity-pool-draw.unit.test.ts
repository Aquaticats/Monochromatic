/**
 Tests for the compiled `editor-fidelity-pool-draw` command's paths that touch no corpus: the requests it refuses before
 it reads anything, with which exit status and stderr.

 Each run is the built command in an owned empty directory with no provider key set, so no case can read a clone or spend.

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
   Standard error, one entry per line.
   */
  readonly stderr: readonly string[];
};

/**
 Runs the built command with the arguments given, in an owned empty directory that is also the corpus clone.

 @param args - every argument after the script path

 @returns Exit status and standard error, one entry per line

 @example
 ```ts
 const run = await runAsBuilt({ args: ['--check',], },);
 ```
 */
async function runAsBuilt({ args, }: { readonly args: readonly string[]; },): Promise<BuiltRun> {
  await using owned = await scratchDir({ prefix: 'editor-fidelity-pool-draw-as-built-', },);
  /**
   What the built command did, whatever its exit code.
   */
  const run = await runBuiltCommand({
    command: 'editor-fidelity-pool-draw',
    packageEntry: PROBE_ENTRY,
    args,
    cwd: owned.path,
    env: { TRANSLATION_REPAIR_CORPUS_CLONE_DIR: owned.path, },
  },);
  return {
    code: run.code,
    stderr: run.stderr.split('\n',)
      .filter(function present(line,): boolean {
        return line !== '';
      },),
  };
}

await describe({
  name: 'editor-fidelity-pool-draw as built',
  children: [
    it({
      name: 'REFUSES A CHECK WITH NO DIRECTORY at exit six with one line, before it reads the clone',
      fn: async () => {
        const run = await runAsBuilt({ args: ['--check',], },);
        expect(run.code,).toBe(REFUSED_AS_STATED,);
        expect(run.stderr,).toStrictEqual([
          'editor-fidelity-pool-draw: --check compares the draw with the files in --out, and --out was not written; '
            + 'write the directory the drawn files sit in',
        ],);
      },
    },),

    it({
      name: 'REFUSES A FLAG IT DOES NOT DECLARE at exit six, naming the command',
      fn: async () => {
        const run = await runAsBuilt({ args: ['--clone', 'elsewhere',], },);
        expect(run.code,).toBe(REFUSED_AS_STATED,);
        expect(run.stderr[0],).toContain('editor-fidelity-pool-draw: ',);
      },
    },),
  ],
},);
