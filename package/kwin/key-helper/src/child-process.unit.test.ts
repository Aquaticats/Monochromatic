/**
 Tests for `runCommand`, the single owner of the daemon's one child-process
 shape: a child that exits zero is success, and a nonzero exit, a signal, or a
 command that could not be started throws `CommandFailedError` naming the
 command, how it ended, and its stderr.

 The children are this same Node executable run with `-e` one-liners, so no
 external tool is needed and nothing in the owner's session is touched.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CommandFailedError,
  runCommand,
} from '../dist/final/node/testing.mjs';

/**
 Interpreter that runs each child, so a stand-in tool is never needed.
 */
const NODE_EXECUTABLE = process.execPath;

/**
 Exit status a child reports when it is meant to fail.
 */
const FAILURE_EXIT_CODE = 7;

/**
 Wording a command that was never started reports.
 */
const UNSTARTED_WORDING = 'could not be started';

/**
 Characters a buffered one-shot runner holds before it ends the child, which is
 Node's documented default `maxBuffer` for `execFile`.
 */
const BUFFER_LIMIT_CHARACTERS = 1024 * 1024;

/**
 Output the chatty child writes, comfortably over that bound.
 */
const CHATTY_OUTPUT_CHARACTERS = 2 * BUFFER_LIMIT_CHARACTERS;

/**
 Characters one write of the chatty child carries.
 */
const CHATTY_CHUNK_CHARACTERS = 64 * 1024;

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

await describe({
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
});
