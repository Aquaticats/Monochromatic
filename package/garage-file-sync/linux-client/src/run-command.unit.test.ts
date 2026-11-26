/**
 Tests for the one asynchronous command runner: what a command that finished produced,
 and the reason a command never finished.

 @module
 */

import { mkdtempDisposable, readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { setTimeout as delay, } from 'node:timers/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type CommandResult,
  runCommand,
} from './run-command.ts';

/**
 Longest time a test command may run before the runner stops it, in milliseconds.
 */
const TEST_TIMEOUT_MS = 500;

/**
 How far past its timeout the runner may return a problem, in milliseconds.

 Destroying the pipes makes the pending stream reads fail at once, so the runner has no
 work left beyond reporting what the timeout stopped.
 */
const TIMEOUT_RETURN_MARGIN_MS = 2_000;

/**
 How long the child that must overrun the timeout would run for, in milliseconds.

 Longer than the timeout, so the runner always stops it and never waits for it to end
 on its own.
 */
const OUTLIVING_LIFETIME_MS = 30_000;

/**
 How long the descendant that holds the child's pipes open would run for, in
 milliseconds.

 A timer, never a loop: the descendant only has to outlive the runner's timeout and the
 test's assertions.
 */
const DESCENDANT_LIFETIME_MS = 60_000;

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

/**
 Waits for a command, but not past a budget.

 @param run - the command run to wait for

 @param budgetMs - longest wait before reporting that the run did not finish

 @returns what the command reported, or nothing when it overran the budget
 */
async function runWithinBudget(run: Promise<CommandResult>, budgetMs: number,): Promise<CommandResult | undefined> {
  const outcome = await Promise.race([
    run.then(function finished(result,) {
      return { result, };
    },),
    delay(budgetMs,).then(function overran() {
      return {};
    },),
  ],);
  return outcome.result;
}

/**
 Builds a script that starts a descendant inheriting the script's stdout and stderr, and
 exits at once.

 The descendant only exists to hold the pipes the child leaves open, so it waits on a
 timer for a named lifetime.

 @param pidFile - file the script records the descendant's process id in

 @returns the script, for `node --eval`
 */
function descendantScript(pidFile: string,): string {
  return `const { spawn } = require('node:child_process');
const { writeFileSync } = require('node:fs');
const descendant = spawn(process.execPath, ['--eval', 'setTimeout(function waited() {}, ${String(DESCENDANT_LIFETIME_MS,)})'], { stdio: ['ignore', 'inherit', 'inherit'] });
writeFileSync(${JSON.stringify(pidFile,)}, String(descendant.pid,));
descendant.unref();`;
}

/**
 Reads the process id a script recorded.

 @param pidFile - file the script wrote

 @returns the process id, or nothing when the file holds no number
 */
async function readReportedPid(pidFile: string,): Promise<number | undefined> {
  const pid = Math.trunc(Number(await readFile(pidFile, 'utf8',),),);
  return Number.isNaN(pid,) ? undefined : pid;
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
      name: 'returns a problem within its timeout when a descendant keeps the pipes open',
      fn: async function descendantHoldsPipes() {
        await using home = await mkdtempDisposable(join(scratchRoot(), 'garage-run-command-',),);
        /**
         File the child records its descendant's process id in.
         */
        const pidFile = join(home.path, 'descendant-pid.txt',);
        /**
         When the runner was asked to start the command.
         */
        const startedAt = performance.now();
        /**
         What the command reported, or nothing when it overran the timeout and the
         margin.
         */
        const result = await runWithinBudget(
          runCommand({
            command: process.execPath,
            args: [
              '--eval',
              descendantScript(pidFile,),
            ],
            timeoutMs: TEST_TIMEOUT_MS,
          },),
          TEST_TIMEOUT_MS + TIMEOUT_RETURN_MARGIN_MS,
        );
        /**
         How long the runner took to report.
         */
        const elapsedMs = performance.now() - startedAt;
        /**
         Process id of the descendant holding the pipes, which the test must end.
         */
        const pid = await readReportedPid(pidFile,);
        if (pid !== undefined) stopIfRunning(pid,);
        expect(result,).toBeDefined();
        expect(result?.exitCode,).toBeUndefined();
        expect(result?.problem,).toContain(`${String(TEST_TIMEOUT_MS,)} ms`,);
        expect(elapsedMs,).toBeLessThanOrEqual(TEST_TIMEOUT_MS + TIMEOUT_RETURN_MARGIN_MS,);
      },
      timeout: 30_000,
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
