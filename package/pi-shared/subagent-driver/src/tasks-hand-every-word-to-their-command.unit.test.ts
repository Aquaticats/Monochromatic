/**
 Every task of this package hands every word it was given to its command, and a command answers `--help` with its usage line and a
 zero status.

 On 2026-11-11 `mise run //package/pi-shared/subagent-driver:start-reviews-of-finished-branches -- --help` printed a syntax error
 instead of help: mise appends a word its task's `usage` spec does not claim to the task's script text, `--help` is mise's own flag,
 and the task was a `node -e` script, so the word landed in the program. The same append, on a task that runs its command as a
 plain command line, hands the word to the command as an argument.

 Each test here runs a real `mise task`, which is the layer that was broken, so the words, the task body and the command are all
 the ones a coordinator runs.

 @module
 */

import { execFile, } from 'node:child_process';
import { promisify, } from 'node:util';
import { dirname, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 Every task of this package that runs one of its commands, named as `mise run` names it.
 */
const TASKS_RUNNING_A_COMMAND: readonly string[] = [
  'start-pi-run-in-a-worktree',
  'start-pi-run-when-a-pool-has-room',
  'retry-pi-run-start-until-a-pool-has-room',
  'start-pi-runs-from-a-batch-list',
  'save-the-final-message-of-a-review-that-wrote-no-review-file',
  'list-processes-whose-command-line-holds-text',
  'list-running-units-whose-worktree-is-gone',
  'print-the-final-report-of-the-newest-run-of-each-agent',
  'list-launchers-whose-unit-has-ended',
  'remove-finished-worktrees',
  'start-a-batch-in-its-own-unit',
  'write-a-brief-for-a-review',
  'start-reviews-of-finished-branches',
  'write-a-fix-brief-for-each-review',
];

/**
 The package folder, which holds the `mise.toml` whose tasks these are and is where `mise run` is started.
 */
const PACKAGE_FOLDER = dirname(new URL(import.meta.url,).pathname,);

/**
 Seconds one task may take. Each of these commands answers help before it reads an option, so what it costs is mise reading its
 configuration and one Node start; the bound only keeps a task that hangs from holding the test.
 */
const TASK_SECONDS = 120;

/**
 `execFile` with a promise result, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);

/**
 What asking a task for help answered.

 @param task - the task's name

 @returns its status and both streams

 @throws Error when `mise` could not be started at all, which names what to put on `PATH`

 @example
 ```ts
 const ended = await helpOfTask('start-pi-run-in-a-worktree');
 ```
 */
async function helpOfTask(task: string,): Promise<{
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string
}> {
  try {
    /**
     What the task answered.
     */
    const ended = await execFileAsync('mise', [
      'run',
      task,
      '--',
      '--help',
    ], {
      cwd: PACKAGE_FOLDER,
      encoding: 'utf8',
      timeout: TASK_SECONDS * 1_000,
      killSignal: 'SIGKILL',
    },);
    return {
      status: 0,
      stdout: ended.stdout,
      stderr: ended.stderr,
    };
  }
  catch (caught) {
    /**
     What a task that failed reported, read field by field because a failure carries the status it exited with.
     */
    const failure = caught as {
      readonly code?: unknown;
      readonly message?: unknown;
      readonly stdout?: unknown;
      readonly stderr?: unknown;
    };
    const status = ((typeof failure.code) === 'number') ? failure.code : (-1);
    // A task whose program was never found is the one failure the message explains, and it is worth saying so rather than
    // reporting a status with nothing behind it.
    if (status === (-1)) {
      throw new Error(
        `mise could not be run for ${task}: ${String(failure.message,)}`,
        { cause: caught, },
      );
    }
    return {
      status,
      stdout: ((typeof failure.stdout) === 'string') ? failure.stdout : '',
      stderr: ((typeof failure.stderr) === 'string') ? failure.stderr : '',
    };
  }
}

await describe({
  name: 'a task of this package',
  children: TASKS_RUNNING_A_COMMAND.map(function testOf(task,) {
    return it({
      name: `${task} hands --help to its command, which prints its usage line and exits zero`,
      fn: async () => {
        /**
         What the task answered.
         */
        const ended = await helpOfTask(task,);
        expect(ended.status,).toBe(0,);
        expect(ended.stdout,).toContain('Usage: ',);
        // A word that never reached the command would end as a syntax error in the task's own text, which is what this test
        // exists to catch; the message names the failure rather than leaving a bare status.
        expect(ended.stderr,).not.toContain('ERR_INVALID_TYPESCRIPT_SYNTAX',);
      },
    },);
  },),
},);
