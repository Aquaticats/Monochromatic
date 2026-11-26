/**
 Tests for the one asynchronous command runner: what a command that finished produced,
 and the reason a command never finished.

 @module
 */

import { mkdtempDisposable, readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { runCommand, } from './run-command.ts';

/**
 Longest time a test command may run before the runner stops it, in milliseconds.
 */
const TEST_TIMEOUT_MS = 500;

/**
 How long the child that must overrun the timeout would run for, in milliseconds.

 Longer than the timeout, so the runner always stops it and never waits for it to end
 on its own.
 */
const OUTLIVING_LIFETIME_MS = 30_000;

/**
 Name of a command no machine has, so starting it fails the same way everywhere.
 */
const MISSING_COMMAND = 'rclone-bisync-garage-no-such-command';

/**
 Stops a process a stopped command left running, so a regression cannot hold the suite
 open.

 @param pid - process id the stopped command reported

 @returns whether the process was still running
 */
function stopIfRunning(pid: number,): boolean {
  try {
    process.kill(pid, 'SIGKILL',);
    return true;
  } catch (error) {
    if (Error.isError(error,) && ('code' in error) && (error.code === 'ESRCH')) return false;
    throw error;
  }
}

await describe({
  name: runCommand.name,
  children: [
    it({
      name: 'reports stdout, stderr and exit status 0 for a command that exits 0 with output',
      fn: async function exitsZero() {
        /**
         What the finished command reported.
         */
        const result = await runCommand({
          command: process.execPath,
          args: [
            '--eval',
            String.raw`process.stdout.write("out\n"); process.stderr.write("err\n")`,
          ],
        },);
        expect(result,).toEqual({
          stdout: 'out\n',
          stderr: 'err\n',
          exitCode: 0,
        },);
      },
    },),
    it({
      name: 'reports a non-zero exit status and the printed text, instead of throwing',
      fn: async function exitsNonZero() {
        /**
         What the finished command reported.
         */
        const result = await runCommand({
          command: process.execPath,
          args: [
            '--eval',
            'process.stdout.write("before exit"); process.exitCode = 3',
          ],
        },);
        expect(result,).toEqual({
          stdout: 'before exit',
          stderr: '',
          exitCode: 3,
        },);
      },
    },),
    it({
      name: 'reports no exit status for a command a signal ended',
      fn: async function endedBySignal() {
        /**
         What the finished command reported.
         */
        const result = await runCommand({
          command: process.execPath,
          args: [
            '--eval',
            'process.kill(process.pid, "SIGKILL")',
          ],
          timeoutMs: TEST_TIMEOUT_MS,
        },);
        expect(result,).toEqual({
          stdout: '',
          stderr: '',
          exitCode: -1,
        },);
      },
    },),
    it({
      name: 'reports a problem naming the command when it cannot start',
      fn: async function cannotStart() {
        /**
         What the command that could not start reported.
         */
        const result = await runCommand({
          command: MISSING_COMMAND,
          args: [],
        },);
        expect(result.exitCode,).toBeUndefined();
        expect(result.stdout,).toBe('');
        expect(result.stderr,).toBe('');
        expect(result.problem,).toContain(MISSING_COMMAND,);
        expect(result.problem,).toContain('could not be run',);
      },
    },),
    it({
      name: 'stops a command that outlives its timeout and reports it, instead of waiting for it',
      fn: async function outlivesTimeout() {
        /**
         What the command the timeout stopped reported.
         */
        const result = await runCommand({
          command: process.execPath,
          args: [
            '--eval',
            `setTimeout(function later() {}, ${String(OUTLIVING_LIFETIME_MS,)})`,
          ],
          timeoutMs: TEST_TIMEOUT_MS,
        },);
        expect(result.exitCode,).toBeUndefined();
        expect(result.problem,).toContain(`${String(TEST_TIMEOUT_MS,)} ms`,);
      },
    },),
    it({
      name: 'kills a stopped command that ignores SIGTERM, so it cannot hold the runner open',
      fn: async function stopsAnIgnoringCommand() {
        await using home = await mkdtempDisposable(join(scratchRoot(), 'garage-run-command-',),);
        /**
         File the child writes its own process id to before it stops listening to SIGTERM.
         */
        const pidFile = join(home.path, 'pid.txt',);
        /**
         A command that reports its id, ignores SIGTERM and keeps running.
         */
        const childScript = `require('node:fs').writeFileSync(${JSON.stringify(pidFile)}, String(process.pid)); process.on('SIGTERM', function ignored() {}); setInterval(function tick() {}, 1000);`;
        /**
         What the command the timeout stopped reported.
         */
        const result = await runCommand({
          command: process.execPath,
          args: [
            '--eval',
            childScript,
          ],
          timeoutMs: TEST_TIMEOUT_MS,
        },);
        expect(result.problem,).toContain(`${String(TEST_TIMEOUT_MS,)} ms`,);
        /**
         The process id the child wrote, as text.
         */
        const pidText = await readFile(pidFile, 'utf8',);
        /**
         Process id the child reported, which a missing file reports as a child that never
         started.
         */
        const pid = Math.trunc(Number(pidText,),);
        expect(Number.isNaN(pid,),).toBe(false,);
        expect(stopIfRunning(pid,),).toBe(false,);
      },
      timeout: 30_000,
    },),
  ],
},);
