/**
 Tests for the loader of the rotation sweep (ledger B701), run the way the sweep runs it: a node process started with the hook
 imported, loading a stand-in for the built stage bundle from a scratch directory.

 The hook is a loader, so its cases start child processes through `runSalted`, which hands them no key and no setting.

 Fixtures are cat-themed invention.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

import { runSalted, } from './rotation-sweep-run.test-fixture.ts';

/**
 A stand-in for the bundle: the function as the minified build spells it, answering the length of the prompt.
 */
const BUNDLE = 'function benchRotation({messages,size}){if(size<=0)return 0;return messages.length%size}export{benchRotation};\n';

/**
 Program that loads the stand-in and prints its answer for two messages and a bench of five.
 */
const ASK = 'const { benchRotation } = await import(process.argv[1]); process.stdout.write(String(benchRotation({ messages: [1, 2], size: 5 })));';

/**
 What a salted process printed and how it ended.

 @param salt - seats the rotation start is moved by

 @returns Exit code, standard output and standard error of the process

 @example
 ```ts
 const run = await askUnder({ salt: 3, },);
 ```
 */
async function askUnder({ salt, }: { readonly salt: number; },): Promise<{ readonly code: number; readonly stdout: string; readonly stderr: string; }> {
  await using scratch = await scratchDir({ prefix: 'rotation-hook-', },);
  /**
   Path of the stand-in bundle.
   */
  const bundle = join(
    scratch.path,
    'bundle.mjs',
  );
  await writeFile(
    bundle,
    BUNDLE,
  );
  return await runSalted({
    salt,
    cwd: scratch.path,
    args: [
      '--input-type=module',
      '--eval',
      ASK,
      bundle,
    ],
  },);
}

await describe({
  name: runSalted.name,
  concurrency: 1,
  children: [
    it({
      name: 'moves the rotation of a loaded bundle by the salt and prints nothing of its own',
      fn: async () => {
        // The stand-in answers 2 % 5 = 2; salt 4 moves it to (2 + 4) % 5 = 1. The whole output is the program's answer.
        expect(await askUnder({ salt: 0, },),).toStrictEqual({ code: 0, stdout: '2', stderr: '', },);
        expect(await askUnder({ salt: 4, },),).toStrictEqual({ code: 0, stdout: '1', stderr: '', },);
      },
    },),
  ],
},);
