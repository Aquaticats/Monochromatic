/**
 Runs every `src/*.unit.test.ts` file of this package under another
 JavaScript runtime command, two at a time, and prints pass or fail per file
 with the tail of each failure.

 Runs inside the runtime's official image, capped, with this repo read-only;
 the runtime executing this file only needs `node:child_process`:

 ```sh
 bun src/surface-runtime.ts bun
 deno run --allow-all --no-lock --node-modules-dir=manual src/surface-runtime.ts deno run --allow-all --no-lock --node-modules-dir=manual
 ```

 `DEEPMERGE_FUZZ_TARGET` passes through to each file, so the same command
 checks the CJS build (`node_modules/deepmerge-ts/dist/index.cjs`) or a
 control build. Findings: `doc/audit/deepmerge-ts-surface-2026-09-24.md`.

 @module
 */

import { readdir, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  runChild,
  type ChildOutcome,
} from './surface-toolchain.ts';

/**
 This package's directory.
 */
const PACKAGE = join(
  import.meta.dirname,
  '..',
);

/**
 Per-file limit; the slowest file measured under Node takes under a minute.
 */
const FILE_TIMEOUT_MS = 300_000;

/**
 Output lines kept from a failing file.
 */
const TAIL_LINES = 6;

/**
 Characters kept per failing output line.
 */
const LINE_WIDTH = 240;

/**
 Files run at once, matching the container's two CPUs.
 */
const WORKERS = 2;

/**
 Run every unit test file with `command` and print one line per file.

 @param command - Runtime executable.

 @param commandArgs - Arguments before the file path.

 @returns Whether every file passed.

 @example
 ```ts
 await runAll({ command: 'bun', commandArgs: [], },);
 ```
 */
async function runAll(
  {
    command,
    commandArgs,
  }: {
    readonly command: string;
    readonly commandArgs: readonly string[];
  },
): Promise<boolean> {
  /**
   Test files in name order.
   */
  const files = (await readdir(join(
    PACKAGE,
    'src',
  ),)).filter(function isUnitTest(name,) {
    return name.endsWith('.unit.test.ts',);
  },)
    .toSorted();
  /**
   Outcome per file, filled by the workers.
   */
  const outcomes = new Map<string, ChildOutcome>();
  /**
   Files not yet started.
   */
  const queue = [...files,];
  await Promise.all(Array.from(
    { length: WORKERS, },
    async function worker() {
    /* oxlint-disable eslint/no-await-in-loop -- each worker runs its files one after another. */
      for (let file = queue.shift(); file !== undefined; file = queue.shift()) {
        outcomes.set(
          file,
          await runChild({
            args: [
              ...commandArgs,
              join(
                'src',
                file,
              ),
            ],
            command,
            cwd: PACKAGE,
            timeoutMs: FILE_TIMEOUT_MS,
          },),
        );
      }
    /* oxlint-enable eslint/no-await-in-loop */
    },
  ),);
  for (const file of files) {
    /**
     This file's outcome.
     */
    const outcome = outcomes.get(file,);
    if (outcome === undefined)
      throw new Error(`no outcome recorded for ${file}`,);
    console.log(`${outcome.passed ? 'pass' : 'FAIL'} ${file}${outcome.passed ? '' : ` (exit ${outcome.exit})`}`,);
    if (!outcome.passed) {
      for (const line of outcome.output
        .split('\n',)
        .filter(function nonEmpty(text,) {
          return text.trim() !== '';
        },)
        .slice(-TAIL_LINES,))
        console.log(`    ${line.slice(
          0,
          LINE_WIDTH,
        )}`,);
    }
  }
  /**
   Files that passed.
   */
  const passed = [...outcomes.values(),].filter(function didPass(outcome,) {
    return outcome.passed;
  },)
    .length;
  console.log(`${String(passed,)} of ${String(files.length,)} files pass`,);
  return passed === files.length;
}

if (import.meta.main) {
  /**
   Runtime command and its arguments before each file.
   */
  const [command, ...commandArgs] = process.argv
    .slice(2,);
  if (command === undefined)
    throw new Error('usage: surface-runtime.ts <runtime command> [arguments before the file...]',);
  if (!(await runAll({
    command,
    commandArgs,
  },)))
    throw new Error(`some test files failed under ${command}`,);
}
