/**
 The package's unit test task declares the build whose copy its test files import, so `mise run ...:test:unit` never runs a
 test against a built copy older than the sources it was built from.

 74 of the package's 77 unit test files import `../dist/final/node/index.mjs`, which is the copy the exports map names for a
 user of the package, and nothing else builds it before the suite runs. In a worktree whose copy was older than its sources
 the suite ran against that copy: on 2026-11-27 two files failed to load with `SyntaxError: The requested module
 '../dist/final/node/index.mjs' does not provide an export named 'AGENT_SCRATCH_FOLDER_PARTS'` while the rest said nothing
 (issue 6635).

 This test asks mise for the task's dependency list, so a declaration that stops building fails here instead of surfacing as
 a suite run against whatever copy a worktree happens to hold.

 @module
 */

import { execFile, } from 'node:child_process';
import { dirname, } from 'node:path';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 What mise answers about one task, of the fields this test reads.

 @param depends - the tasks mise runs before this one
 */
type TaskInfo = {
  readonly depends: readonly string[];
};

/**
 The package folder, which holds the `mise.toml` whose task this test asks mise about.
 */
const PACKAGE_FOLDER = dirname(new URL(import.meta.url,).pathname,);

/**
 The task a coordinator runs for this package's unit tests, named as this package's `mise.toml` names it.
 */
const TEST_UNIT_TASK = 'test:unit';

/**
 Seconds mise may take to answer. It answers from the monorepo's task graph in well under a second on a warm machine, so the
 bound only keeps a mise that hangs from holding the test.
 */
const ANSWER_SECONDS = 120;

/**
 `execFile` with a promise result, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);

/**
 Reads the dependencies mise declares for a task of this package.

 @param task - the task, named as this package's `mise.toml` names it

 @returns the tasks mise runs before it, as mise's own task graph names them

 @throws Error when mise could not be run at all, which names what to put on `PATH`

 @example
 ```ts
 const depends = await declaredDependenciesOf(TEST_UNIT_TASK);
 ```
 */
async function declaredDependenciesOf(task: string,): Promise<readonly string[]> {
  try {
    const ended = await execFileAsync('mise', [
      'tasks',
      'info',
      '--json',
      task,
    ], {
      cwd: PACKAGE_FOLDER,
      encoding: 'utf8',
      timeout: ANSWER_SECONDS * 1_000,
      killSignal: 'SIGKILL',
    },);
    /**
     What mise said about the task, of the fields this test reads.
     */
    const answered: TaskInfo = JSON.parse(ended.stdout,) as TaskInfo;
    return answered.depends;
  }
  catch (caught) {
    throw new Error(
      `mise could not be asked about ${task}: ${(caught as Error).message}`,
      { cause: caught, },
    );
  }
}

await describe({
  name: 'a unit test task of this package',
  children: [
    it({
      name: `${TEST_UNIT_TASK} builds before it runs a test file, so no test reads a copy older than the sources`,
      fn: async () => {
        /**
         The tasks mise runs before the unit test task.
         */
        const depends = await declaredDependenciesOf(TEST_UNIT_TASK,);
        // The test files import the exports map's `dist/final/node/index.mjs`, and only the `build` task writes it.
        expect(depends,).toContain('build',);
      },
    },),
  ],
},);
