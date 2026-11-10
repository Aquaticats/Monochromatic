import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { NativeGateError } from './native-gate-error.ts';
import { runGateCommand } from './native-gate-run.ts';
import { createGateScratchDirectory } from './stand-in-gate-programs-for-tests.ts';
import { gateExitStatusMarker } from './native-gate-unit.ts';
import {
  defaultVerdictPollMilliseconds,
  readGateLog,
  verdictLineLimit,
  verdictLinesOf,
  waitForGateVerdict,
} from './native-gate-verdict-reader.ts';

/**
 Path of the command under test, which runs as TypeScript.
 */
const verdictPath = join(
  import.meta.dirname,
  'native-gate-verdict.ts',
);

/**
 A gate log that ended, as the unit writes it: nextest's summary, one failed test and the exit-status line last.

 @returns the log's text
 */
function finishedLog(): string {
  return [
    '   Compiling monochromatic-git-policy-cli',
    '        PASS [   0.176s] (1/2) monochromatic-git-policy-cli a::b',
    '        FAIL [   1.204s] (2/2) monochromatic-git-policy-cli c::d',
    'error: test failed, to rerun pass `--lib`',
    '     Summary [ 1000.412s] 2 tests run: 1 passed, 1 failed',
    `${gateExitStatusMarker}1`,
    '',
  ].join('\n');
}

/**
 A gate whose start never worked: the systemd unit's first line, and nothing else.

 @returns the log's text
 */
function unfinishedLog(): string {
  return 'Running as unit: gpc-gate-a-20261110T180203-0500.service\n';
}

/**
 Run the command under test.

 @param args - arguments after the script path

 @returns how the command ended, with its output
 */
async function verdict({
  args,
}: {
  readonly args: readonly string[];
}) {
  return await runGateCommand({
    program: process.execPath,
    args: [
      verdictPath,
      ...args,
    ],
    deadlineMilliseconds: 60_000,
  });
}

await describe({
  name: 'reading a gate\'s verdict from its log',
  children: [
    it({
      name: 'keeps the summary, the failed tests and the exit-status line of a finished gate',
      fn: async () => {
        /**
         The lines the log answers.
         */
        const lines = verdictLinesOf({ text: finishedLog(), });
        expect(lines,).toContain('     Summary [ 1000.412s] 2 tests run: 1 passed, 1 failed',);
        expect(lines,).toContain('        FAIL [   1.204s] (2/2) monochromatic-git-policy-cli c::d',);
        expect(lines,).toContain('error: test failed, to rerun pass `--lib`',);
        expect(lines.at(-1),).toBe(`${gateExitStatusMarker}1`,);
      },
    }),
    it({
      name: 'drops the lines a person does not need, so a verdict is the answer and not the whole log',
      fn: async () => {
        expect(verdictLinesOf({ text: finishedLog(), }),).not.toContain('   Compiling monochromatic-git-policy-cli',);
      },
    }),
    it({
      name: 'answers a line once however often a run printed it, because nextest repeats a test\'s status line',
      fn: async () => {
        /**
         The same interesting line twice.
         */
        const lines = verdictLinesOf({ text: 'error: same\nmiddle\nerror: same\n', });
        expect(lines,).toEqual(['error: same',],);
      },
    }),
    it({
      name: 'keeps the last lines and no more, because the failures of a run are at its end',
      fn: async () => {
        /**
         More interesting lines than a verdict prints.
         */
        const text = Array.from(
          { length: verdictLineLimit + 5, },
          function failureLine(_unused, index) {
            return `error: failure ${String(index + 1)}`;
          },
        ).join('\n');
        /**
         What the log answers.
         */
        const lines = verdictLinesOf({ text, });
        expect(lines,).toHaveLength(verdictLineLimit,);
        expect(lines.at(-1),).toBe(`error: failure ${String(verdictLineLimit + 5)}`,);
        expect(lines.at(0),).toBe('error: failure 6',);
      },
    }),
    it({
      name: 'reads a log the unit has not written yet as empty text, so a wait is not a failure',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-gate-verdict-', });
        expect(
          await readGateLog(join(
            scratch.path,
            'absent.log',
          )),
        ).toBe('',);
      },
    }),
    it({
      name: 'refuses a log that is there and cannot be read, instead of waiting out a verdict it could not see',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-gate-verdict-', });
        await expect(readGateLog(scratch.path),)
          .rejects
          .toThrow(NativeGateError,);
      },
    }),
    it({
      name: 'answers a log that already holds a verdict without waiting at all',
      fn: async () => {
        /**
         The intervals the wait would have slept.
         */
        const slept: number[] = [];
        /**
         What the wait found.
         */
        const outcome = await waitForGateVerdict({
          logPath: '/l/gate.log',
          deadlineMilliseconds: 60_000,
          pollMilliseconds: defaultVerdictPollMilliseconds,
          readLog: async function finished() {
            return finishedLog();
          },
          wait: async function record(milliseconds) {
            slept.push(milliseconds);
            return undefined;
          },
        });
        expect(outcome.kind,).toBe('verdict',);
        expect(slept,).toEqual([],);
      },
    }),
    it({
      name: 'looks again after each interval and answers the verdict that arrives',
      fn: async () => {
        /**
         The clock the wait reads, which the stand-in sleep advances.
         */
        const clock = { now: 0, };
        /**
         How many looks have been taken.
         */
        let looks = 0;
        /**
         The intervals the wait slept.
         */
        const slept: number[] = [];
        /**
         What the wait found.
         */
        const outcome = await waitForGateVerdict({
          logPath: '/l/gate.log',
          deadlineMilliseconds: 60_000,
          pollMilliseconds: defaultVerdictPollMilliseconds,
          now: function currentTime() {
            return clock.now;
          },
          readLog: async function afterThreeLooks() {
            looks += 1;
            return looks < 3 ? unfinishedLog() : finishedLog();
          },
          wait: async function advance(milliseconds) {
            slept.push(milliseconds);
            clock.now += milliseconds;
            return undefined;
          },
        });
        expect(outcome.kind,).toBe('verdict',);
        expect(slept,).toEqual([
          defaultVerdictPollMilliseconds,
          defaultVerdictPollMilliseconds,
        ],);
      },
    }),
    it({
      name: 'never waits past its deadline, shortening the last interval to what is left',
      fn: async () => {
        /**
         The clock the wait reads.
         */
        const clock = { now: 0, };
        /**
         The intervals the wait slept.
         */
        const slept: number[] = [];
        /**
         What the wait found for a log that never ends.
         */
        const outcome = await waitForGateVerdict({
          logPath: '/l/gate.log',
          deadlineMilliseconds: 25,
          pollMilliseconds: defaultVerdictPollMilliseconds,
          now: function currentTime() {
            return clock.now;
          },
          readLog: async function unfinished() {
            return unfinishedLog();
          },
          wait: async function advance(milliseconds) {
            slept.push(milliseconds);
            clock.now += milliseconds;
            return undefined;
          },
        });
        expect(outcome.kind,).toBe('none',);
        expect(slept,).toEqual([25],);
      },
    }),
    it({
      name: 'takes one look and no sleep when the deadline has already passed',
      fn: async () => {
        /**
         How many looks were taken.
         */
        let looks = 0;
        /**
         What the wait found.
         */
        const outcome = await waitForGateVerdict({
          logPath: '/l/gate.log',
          deadlineMilliseconds: 0,
          pollMilliseconds: defaultVerdictPollMilliseconds,
          readLog: async function counted() {
            looks += 1;
            return unfinishedLog();
          },
          wait: async function noSleep() {
            throw new Error('a passed deadline never sleeps');
          },
        });
        expect(outcome.kind,).toBe('none',);
        expect(looks,).toBe(1,);
      },
    }),
    it({
      name: 'lets a log it cannot read end the wait at once, rather than waiting for nothing',
      fn: async () => {
        await expect(waitForGateVerdict({
          logPath: '/l/gate.log',
          deadlineMilliseconds: 60_000,
          pollMilliseconds: defaultVerdictPollMilliseconds,
          readLog: async function unreadable() {
            throw new NativeGateError('the gate log is not text');
          },
        }),)
          .rejects
          .toThrow('not text',);
      },
    }),
    it({
      name: 'prints the verdict of a finished gate and exits zero, whatever the gate\'s own status was',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-gate-verdict-', });
        /**
         The log's path.
         */
        const logPath = join(
          scratch.path,
          'gpc-gate-a-20261110T180203-0500.log',
        );
        await writeFile(
          logPath,
          finishedLog(),
        );
        /**
         How the command ended.
         */
        const printed = await verdict({ args: [logPath, '0',], });
        expect(printed.status,).toBe(0,);
        expect(printed.stdout,).toContain(`${gateExitStatusMarker}1`,);
        expect(printed.stdout,).toContain('Summary [ 1000.412s]',);
        expect(printed.stdout,).not.toContain('Compiling',);
      },
    }),
    it({
      name: 'says no verdict arrived when the deadline passes, and exits nonzero',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-gate-verdict-', });
        /**
         The log's path.
         */
        const logPath = join(
          scratch.path,
          'gpc-gate-a-20261110T180203-0500.log',
        );
        await writeFile(
          logPath,
          unfinishedLog(),
        );
        /**
         How the command ended.
         */
        const printed = await verdict({ args: [logPath, '0',], });
        expect(printed.status,).toBe(1,);
        expect(printed.stdout,).toContain(`no verdict in ${logPath} within 0 s`,);
      },
    }),
    it({
      name: 'says no verdict arrived for a log that was never written, without failing to read it',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-gate-verdict-', });
        /**
         The log's path.
         */
        const logPath = join(
          scratch.path,
          'never-written.log',
        );
        /**
         How the command ended.
         */
        const printed = await verdict({ args: [logPath, '0',], });
        expect(printed.status,).toBe(1,);
        expect(printed.stdout,).toContain(`no verdict in ${logPath} within 0 s`,);
      },
    }),
    it({
      name: 'refuses a command line with one positional, and says how the command is called',
      fn: async () => {
        /**
         How the refused command ended.
         */
        const printed = await verdict({ args: ['/l/gate.log',], });
        expect(printed.status,).toBe(1,);
        expect(printed.stderr,).toContain('usage: native:gate:verdict',);
      },
    }),
    it({
      name: 'refuses a deadline that is not a whole number of seconds',
      fn: async () => {
        /**
         How the refused command ended.
         */
        const printed = await verdict({ args: ['/l/gate.log', 'an hour',], });
        expect(printed.status,).toBe(1,);
        expect(printed.stderr,).toContain('is not a whole number from zero',);
      },
    }),
    it({
      name: 'refuses a third positional instead of ignoring it',
      fn: async () => {
        /**
         How the refused command ended.
         */
        const printed = await verdict({ args: ['/l/gate.log', '0', 'stray',], });
        expect(printed.status,).toBe(1,);
        expect(printed.stderr,).toContain('takes one log path and one deadline',);
      },
    }),
  ],
});
