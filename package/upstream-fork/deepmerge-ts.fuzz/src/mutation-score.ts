/**
 In-container StrykerJS run of upstream's own Vitest suite over a
 deepmerge-ts checkout, run by the `mutation` mise task.

 Stages the checkout into `/upstream`, copies
 `../container/stryker.config.json` beside it, and runs Stryker, which writes
 `/out/mutation.json` and prints the score. The mutant ids in that report are
 what `./mutation-sweep.ts` and the audit report refer to.

 @module
 */

import { copyFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  runStep,
  stageCheckout,
  UPSTREAM,
} from './mutation-container.ts';

/**
 Stage the checkout and run Stryker.

 @throws {@link MutationStepError} When Stryker exits unsuccessfully.

 @example
 ```ts
 await runScore();
 ```
 */
export async function runScore(): Promise<void> {
  await stageCheckout();
  await copyFile(
    join(
      import.meta.dirname,
      '..',
      'container',
      'stryker.config.json',
    ),
    join(
      UPSTREAM,
      'stryker.config.json',
    ),
  );
  await runStep({
    args: ['run',],
    command: join(
      UPSTREAM,
      'node_modules',
      '.bin',
      'stryker',
    ),
    cwd: UPSTREAM,
  },);
}

if (import.meta.main)
  await runScore();
