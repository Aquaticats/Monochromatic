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
  ],
});
