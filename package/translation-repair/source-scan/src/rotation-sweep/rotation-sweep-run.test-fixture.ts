import { availableParallelism, } from 'node:os';
import {
  join,
  relative,
} from 'node:path';
import { pathToFileURL, } from 'node:url';

import {
  type ChildRun,
  runKeyless,
} from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';

import { FAMILY_ROOT, } from './rotation-sweep-packages.test-fixture.ts';
import {
  type FailedCase,
  failedCaseNames,
} from './rotation-sweep-report.test-fixture.ts';

//region Rotation sweep run
// THE SWEEP'S MACHINERY (ledger B701). One test file is one child process, started
// with the salt hook imported first (`rotation-salt-hooks.test-fixture.ts`), in the
// directory of its package and with no credential and no setting of the package, the
// way `mise run test:unit` starts it. Every file of every package that can reach
// the stage package runs once per salt.

/**
 URL of the hook module, without its salt.
 */
const HOOKS_URL: string = pathToFileURL(join(
  import.meta.dirname,
  'rotation-salt-hooks.test-fixture.ts',
),)
  .href;

/**
 Runs a node command with the salt hook imported, collecting what it writes.

 The child is started through the shared keyless fixture, which removes every credential and every setting of the package
 from its environment.

 @param salt - seats the rotation start is moved by

 @param cwd - directory the command runs in

 @param args - arguments after `node --import <hook>`

 @returns Exit code, standard output and standard error, each whole

 @example
 ```ts
 const run = await runSalted({ salt: 3, cwd, args: ['a.unit.test.ts',], },);
 ```
 */
export async function runSalted(
  {
    salt,
    cwd,
    args,
  }: {
    readonly salt: number;
    readonly cwd: string;
    readonly args: readonly string[];
  },
): Promise<ChildRun> {
  return await runKeyless({
    file: process.execPath,
    args: [
      '--import',
      `${HOOKS_URL}?salt=${String(salt,)}`,
      ...args,
    ],
    cwd,
  },);
}

/**
 A test file's failed cases under one salt.

 @param file - absolute path of the test file

 @param salt - seats the rotation start is moved by

 @returns The failed cases, named by the file under the family directory

 @example
 ```ts
 const failed = await failedCasesOf({ file, salt: 2, },);
 ```
 */
export async function failedCasesOf(
  {
    file,
    salt,
  }: {
    readonly file: string;
    readonly salt: number;
  },
): Promise<readonly FailedCase[]> {
  /**
   Path of the file under the family directory.
   */
  const familyPath = relative(
    FAMILY_ROOT,
    file,
  );
  /**
   What the file did, run in its package's directory.
   */
  const run = await runSalted({
    salt,
    cwd: join(
      FAMILY_ROOT,
      familyPath.split('/',)[0] ?? '',
    ),
    args: [file,],
  },);
  return failedCaseNames({
    output: `${run.stdout}${run.stderr}`,
    exitCode: run.code,
  },)
    .map(function named(name,): FailedCase {
      return {
        file: familyPath,
        name,
      };
    },);
}

/**
 Runs async work over a list with a bound on how much runs at once, results in input order.

 @param items - what to run over

 @param limit - most that run at once

 @param work - run for each item

 @returns Each item's result, in the order of `items`

 @example
 ```ts
 const lengths = await mapBounded({ items: ['a',], limit: 2, work: async function lengthOf(item,) { return item.length; }, },);
 ```
 */
export async function mapBounded<Item, Result,>(
  {
    items,
    limit,
    work,
  }: {
    readonly items: readonly Item[];
    readonly limit: number;
    readonly work: (item: Item,) => Promise<Result>;
  },
): Promise<readonly Result[]> {
  /**
   Slots for the results, filled at each item's own position.
   */
  const results: Result[] = [];
  /**
   Items not yet taken, shared by every worker so each item is taken once.
   */
  const queue = items.entries();
  /**
   One worker: takes the next item, runs it, and takes another until none is left.
   */
  async function worker(): Promise<void> {
    for (const [at, item,] of queue) {
      /* oxlint-disable no-await-in-loop -- a worker runs one item after another; the bound on how many run at once is the number of workers */
      results[at] = await work(item,);
      /* oxlint-enable no-await-in-loop */
    }
  }
  await Promise.all(Array.from(
    { length: Math.max(
      1,
      Math.min(
        limit,
        items.length,
      ),
    ), },
    worker,
  ),);
  return results;
}

/**
 How many test files run at once, the number `mise run test:unit` uses.

 @returns Parallelism the machine reports

 @example
 ```ts
 const limit = filesAtOnce();
 ```
 */
export function filesAtOnce(): number {
  return availableParallelism();
}

//endregion Rotation sweep run
