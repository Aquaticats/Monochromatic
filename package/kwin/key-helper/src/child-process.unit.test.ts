/**
 Tests for the daemon's two child-process shapes. `runCommand` runs a command to
 completion: a child that exits zero is success, and a nonzero exit, a signal,
 or a command that could not be started throws `CommandFailedError` naming the
 command, how it ended, and its stderr. `startCommand` starts a program that
 keeps running: it resolves once the program runs, throws only when it could not
 be started, and logs an ending that comes later.

 The children are this same Node executable run with `-e` one-liners, so no
 external tool is needed and nothing in the owner's session is touched.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  CommandFailedError,
  flushLogs,
  runCommand,
  startCommand,
} from '../dist/final/node/testing.mjs';

import {
  capturedStandardError,
  reportedUntil,
} from './fake-command-fixture.ts';

/**
 Interpreter that runs each child, so a stand-in tool is never needed.
 */
const NODE_EXECUTABLE = process.execPath;

/**
 Exit status a child reports when it is meant to fail.
 */
const FAILURE_EXIT_CODE = 7;

/**
 Text a failing child writes to stderr.
 */
const FAILURE_TEXT = 'ydotoold is not running';

/**
 How long the held-open child runs before it exits, in milliseconds.
 */
const HELD_OPEN_MILLISECONDS = 2_000;

/**
 Longest a case waits for the log line a started child's failure produces.
 */
const REPORT_DEADLINE_MILLISECONDS = 5_000;

/**
 Wording a command that was never started reports.
 */
const UNSTARTED_WORDING = 'could not be started';

/**
 Characters a buffered one-shot runner holds before it ends the child, which is
 Node's documented default `maxBuffer` for `execFile`.
 */
const BUFFER_LIMIT_CHARACTERS = 1_024 * 1_024;

/**
 Output the chatty child writes, comfortably over that bound.
 */
const CHATTY_OUTPUT_CHARACTERS = 2 * BUFFER_LIMIT_CHARACTERS;

/**
 Characters one write of the chatty child carries.
 */
const CHATTY_CHUNK_CHARACTERS = 64 * 1_024;

/**
 Pause between those writes in milliseconds, so they span several event-loop
 turns and a bound the parent enforces on output gets the chance to act on a
 child that is still writing.
 */
const CHATTY_CHUNK_PAUSE_MILLISECONDS = 10;

/**
 Program the chatty child runs: write more stderr than a buffered runner holds,
 then report the exit status it chose, which it never reaches when the parent
 ends it first.
 */
const CHATTY_CHILD_SCRIPT = `
  const chunk = 'x'.repeat(${String(CHATTY_CHUNK_CHARACTERS)});
  let written = 0;
  while (written < ${String(CHATTY_OUTPUT_CHARACTERS)}) {
    process.stderr.write(chunk);
    written += chunk.length;
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ${String(CHATTY_CHUNK_PAUSE_MILLISECONDS)});
  }
  process.exit(${String(FAILURE_EXIT_CODE)});
`;

/**
 Run one command and return the error it threw, or `undefined` when it resolved.
 
 @param request - Command the test wants run
 @returns Whatever the command threw
 
 @example
 ```ts
 await failureOf({ command: NODE_EXECUTABLE, args: ['-e', 'process.exit(7)'] });
 ```
 */
async function failureOf(request: {
  readonly command: string;
  readonly args: readonly string[];
}): Promise<unknown> {
  try {
    await runCommand(request);
  } catch (error) {
    return error;
  }
  return undefined;
}

/**
 Program the sleep-held child runs: report that it started, then keep running
 for longer than a caller that waited for it would allow.
 */
const HELD_OPEN_CHILD_SCRIPT = `
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ${String(HELD_OPEN_MILLISECONDS)});
`;

/**
 Program the failing started child runs: say why on stderr, then exit
 nonzero after the call that started it has returned.
 */
const FAILING_STARTED_CHILD_SCRIPT = `
  process.stderr.write(${JSON.stringify(FAILURE_TEXT)});
  process.exit(${String(FAILURE_EXIT_CODE)});
`;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runCommand.name,
      children: [
        it({
          name: 'resolves when the child exits zero',
          fn: async () => {
            await runCommand({
              command: NODE_EXECUTABLE,
              args: ['-e', 'process.exit(0)',],
            });
          },
        }),
        it({
          name: 'resolves when a child that exits zero still writes to stderr',
          fn: async () => {
            await runCommand({
              command: NODE_EXECUTABLE,
              args: ['-e', 'console.error("noise"); process.exit(0)',],
            });
          },
        }),
        it({
          name: 'throws CommandFailedError naming the command, exit code and stderr',
          fn: async () => {
        /** Error the failing child produced. */
            const caught = await failureOf({
              command: NODE_EXECUTABLE,
              args: [
                '-e',
                `console.error("ydotoold is not running"); process.exit(${String(FAILURE_EXIT_CODE)})`,
              ],
            });
            expect(caught).toBeInstanceOf(CommandFailedError);
            expect((caught as Error).message).toContain(NODE_EXECUTABLE);
            expect((caught as Error).message).toContain(
              `exited with code ${String(FAILURE_EXIT_CODE)}`,
            );
            expect((caught as Error).message).toContain('ydotoold is not running');
          },
        }),
        it({
          name: 'throws CommandFailedError naming the signal that ended the child',
          fn: async () => {
        /** Error the signal-ended child produced. */
            const caught = await failureOf({
              command: NODE_EXECUTABLE,
              args: ['-e', 'process.kill(process.pid, "SIGKILL")',],
            });
            expect(caught).toBeInstanceOf(CommandFailedError);
            expect((caught as Error).message).toContain('was ended by SIGKILL');
          },
        }),
        it({
          name: 'throws CommandFailedError when the command cannot be started',
          fn: async () => {
        /** Error the missing executable produced. */
            const caught = await failureOf({
              command: 'monochromatic-no-such-executable-for-the-key-helper-tests',
              args: [],
            });
            expect(caught).toBeInstanceOf(CommandFailedError);
            expect((caught as Error).message).toContain(UNSTARTED_WORDING);
          },
        }),
        it({
          name: 'reports the exit status of a child that writes more than the diagnostic limit',
          fn: async () => {
        /** Error the chatty child produced. */
            const caught = await failureOf({
              command: NODE_EXECUTABLE,
              args: ['-e', CHATTY_CHILD_SCRIPT,],
            });
            expect(caught).toBeInstanceOf(CommandFailedError);
            expect((caught as Error).message).toContain(
              `exited with code ${String(FAILURE_EXIT_CODE)}`,
            );
          },
        }),
      ],
    }),
    describe({
      name: startCommand.name,
      children: [
        it({
          name: 'resolves while the child keeps running',
          fn: async () => {
            /**
             When the start was asked for, so the case can tell a running child
             from one whose whole lifetime was waited for.
             */
            const startedAt = performance.now();
            await startCommand({
              command: NODE_EXECUTABLE,
              args: ['-e', HELD_OPEN_CHILD_SCRIPT,],
            });
            expect(performance.now() - startedAt).toBeLessThan(HELD_OPEN_MILLISECONDS / 2);
          },
        }),
        it({
          name: 'throws CommandFailedError when the command cannot be started',
          fn: async () => {
            let caught: unknown;
            try {
              await startCommand({
                command: 'monochromatic-no-such-executable-for-the-key-helper-tests',
                args: [],
              });
            } catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(CommandFailedError);
            expect((caught as Error).message).toContain(UNSTARTED_WORDING);
          },
        }),
        it({
          name: 'logs a child that exits nonzero after it was started',
          fn: async ctx => {
            /** Recorder over the daemon's log destination. */
            const standardError = ctx.sinon.spy(process.stderr, 'write');
            await startCommand({
              command: NODE_EXECUTABLE,
              args: ['-e', FAILING_STARTED_CHILD_SCRIPT,],
            });
            /** Log text the started child's own failure produced. */
            const reported = await reportedUntil({
              spy: standardError,
              matches: function namesStartedFailure(line: string): boolean {
                return line.includes(startCommand.name)
                  && line.includes(`exited with code ${String(FAILURE_EXIT_CODE)}`);
              },
              deadlineMilliseconds: REPORT_DEADLINE_MILLISECONDS,
            });
            expect(reported).toHaveLength(1);
            expect(reported[0]).toContain(NODE_EXECUTABLE);
            expect(reported[0]).toContain(FAILURE_TEXT);
          },
        }),
        it({
          name: 'logs nothing when the child exits zero',
          fn: async ctx => {
            /** Recorder over the daemon's log destination. */
            const standardError = ctx.sinon.spy(process.stderr, 'write');
            await startCommand({
              command: NODE_EXECUTABLE,
              args: ['-e', 'process.exit(0)',],
            });
            // Waiting the deadline the failing case waits, so silence is a fact
            // about the start rather than about how fast the case read.
            await wait(REPORT_DEADLINE_MILLISECONDS,);
            await flushLogs();
            /** Log lines naming a failure of a command this case started. */
            const reported = capturedStandardError(standardError)
              .filter(function namesStartedFailure(line: string): boolean {
                return line.includes(startCommand.name);
              });
            expect(reported).toEqual([]);
          },
        }),
      ],
    }),
  ],
});
