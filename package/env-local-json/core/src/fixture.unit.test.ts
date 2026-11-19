/**
 The fixture's command runner returns the whole standard output, or rejects
 with a bounded diagnostic, including when an inherited pipe stays open.

 The regression this guards: `runCommand` joined its captured chunks at the
 tool's `exit` event, which fires while the tool's standard output pipe can
 still hold unread data. A tool whose output reaches this process after that
 event leaves the join short, and `encryptJson` then writes the cut text as a
 store that every reader refuses as "not the JSON a sops store is".

 The stand-in tool below is a real process, not a synthetic emitter: it prints
 the head of a store, leaves a background subshell holding its standard output
 open, and exits. The tail therefore reaches this process strictly after the
 direct child's `exit` event, whichever way the scheduler runs the two.

 @module
 */

import { Buffer, } from 'node:buffer';
import {
  chmod,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createTempRoot,
  FixtureCommandError,
  MAX_FIXTURE_COMMAND_STDERR_BYTES,
  runCommand,
} from './fixture.ts';

//region Constants

/**
 First half of the document the stand-in tool prints: a store whose one value
 it has not finished. A join that stops here holds text no reader can parse.
 */
const STORE_HEAD = '{"STAND_IN_NAME":"stand-in-value-';

/**
 Second half of that value and the document's close, which the stand-in prints
 after its direct child has ended.
 */
const STORE_TAIL = '0123456789"}';

/**
 Whole document the stand-in tool prints, and the only text a reader can parse.
 */
const WHOLE_STORE = `${STORE_HEAD}${STORE_TAIL}`;

/**
 Values {@link WHOLE_STORE} declares, which is what a cut store cannot parse to.
 */
const STAND_IN_VALUES: Readonly<Record<string, string>> = { STAND_IN_NAME: 'stand-in-value-0123456789', };

/**
 Seconds the stand-in's background subshell waits before printing the tail:
 long enough that the direct child has ended in every observed run, so the
 tail cannot arrive before `exit`.
 */
const TAIL_DELAY_SECONDS = 0.3;

/**
 File name of the stand-in executable inside its disposable root.
 */
const STAND_IN_TOOL_NAME = 'stand-in-tool';

/** Arguments that must be identified as distinct strings, not reconstructed as a shell command. */
const STAND_IN_ARGS = ['store with spaces.json', 'quote" and\nnewline',];

/** Status reported by the failing stand-in tool. */
const STAND_IN_FAILURE_STATUS = 3;

/** Diagnostic written only by the stand-in, including a multibyte character to distinguish bytes from characters. */
const STDOUT_MARKER = 'plaintext kept out of failures';
const STDERR_MARKER = 'stand-in diagnostic: café';
const STDERR_AFTER_CAP = 'diagnostic tail kept out of failures';
const INSTALL_COMMAND = 'mise install sops age';

/** Large enough to fill the operating system's pipe if the reader stops draining after retaining its prefix. */
const FLOOD_PADDING_BYTES = (2 ** 10) * (2 ** 10);

/** Test deadline long after the stand-in can start, but before its deliberately open pipes close. */
const TEST_DEADLINE_MS = 500;
const MILLISECONDS_PER_SECOND = 1_000;
/** Finite so a missing deadline gives a failed assertion instead of stranding a tool. */
const HOLD_SECONDS = (4 * TEST_DEADLINE_MS) / MILLISECONDS_PER_SECOND;

//endregion Constants

//region Helpers

/**
 Writes a shell stand-in for one invocation of the fixture runner.

 @param root - disposable directory that holds the executable
 @param lines - stand-in's shell statements after the shebang

 @returns path of the executable

 @example
 ```ts
 const toolPath = await writeStandInTool({ root: temp.root, lines: ['exit 0'] });
 ```
 */
async function writeStandInTool({
  root,
  lines,
}: {
  readonly root: string;
  readonly lines: readonly string[];
},): Promise<string> {
  /**
   Path of the stand-in executable.
   */
  const path = join(
    root,
    STAND_IN_TOOL_NAME,
  );
  /**
   Executable text with a final newline so the last statement is complete.
   */
  const script = ['#!/bin/sh', ...lines, '',].join('\n',);
  await writeFile(
    path,
    script,
    { mode: 0o700, },
  );
  await chmod(
    path,
    0o700,
  );
  return path;
}

/**
 Reads the failure surfaced to the fixture's caller, rethrowing errors outside the runner's contract.

 @param options - invocation expected to fail
 @returns diagnostic emitted by the fixture runner
 */
async function commandFailureMessage(options: Parameters<typeof runCommand>[0],): Promise<string> {
  try {
    await runCommand(options,);
  }
  catch (error) {
    if (!(error instanceof FixtureCommandError))
      throw error;
    expect(error.message,).toContain(JSON.stringify([options.command, ...options.args,],),);
    return error.message;
  }
  throw new Error(`Expected fixture command ${options.command} to reject, but it returned normally.`,);
}

//endregion Helpers

await describe({
  children: [
    it({
      fn: async () => {
        await using temp = await createTempRoot({ prefix: 'env-local-json-run-command-', },);
        /**
         Path of the tool whose tail arrives after its exit event.
         */
        const toolPath = await writeStandInTool({
          root: temp.root,
          lines: [
            `printf '%s' '${STORE_HEAD}'`,
            `( sleep ${String(TAIL_DELAY_SECONDS,)}; printf '%s' '${STORE_TAIL}' ) &`,
            'exit 0',
          ],
        },);
        /**
         What the fixture's runner captured from that tool.
         */
        const result = await runCommand({
          args: [],
          command: toolPath,
        },);
        expect(result.stdout,)
          .toBe(WHOLE_STORE,);
        // The consequence, not just the text: text cut inside a value is a store no reader can parse, which is how the cut
        // text `encryptJson` wrote reached the driver suite as a SyntaxError.
        expect(JSON.parse(result.stdout,),)
          .toEqual(STAND_IN_VALUES,);
      },
      name: 'returns the standard output a tool writes after its exit event, once its child closes',
    },),
    it({
      fn: async () => {
        await using temp = await createTempRoot();
        const message = await commandFailureMessage({
          args: STAND_IN_ARGS,
          command: 'sops',
          extraEnv: { PATH: temp.root, },
        },);
        expect(message,).toContain('could not start:',);
        expect(message,).toContain('ENOENT',);
        expect(message,).toContain(INSTALL_COMMAND,);
      },
      name: 'gives install advice when a fixture tool is missing from PATH',
    },),
    it({
      fn: async () => {
        await using temp = await createTempRoot();
        const message = await commandFailureMessage({ args: STAND_IN_ARGS, command: join(temp.root, STAND_IN_TOOL_NAME,), },);
        expect(message,).toContain('ENOENT',);
        expect(message,).toContain('interpreter',);
        expect(message,).not.toContain(INSTALL_COMMAND,);
      },
      name: 'does not prescribe fixture tool installation for a missing stand-in',
    },),
    it({
      fn: async () => {
        await using temp = await createTempRoot();
        const command = join(temp.root, 'sops',);
        await writeFile(command, '#!/bin/sh\n', { mode: 0o600, },);
        const message = await commandFailureMessage({
          args: STAND_IN_ARGS,
          command: 'sops',
          extraEnv: { PATH: temp.root, },
        },);
        expect(message,).toContain('EACCES',);
        expect(message,).toContain('Check that the tool is executable',);
        expect(message,).not.toContain(INSTALL_COMMAND,);
      },
      name: 'reports a fixture tool that exists but is not executable without install advice',
    },),
    it({
      fn: async () => {
        const message = await commandFailureMessage({ args: ['\0',], command: 'sops', },);
        expect(message,).toContain('could not start:',);
        expect(message,).not.toContain(INSTALL_COMMAND,);
      },
      name: 'reports a launch rejected before starting a tool',
    },),
    it({
      fn: async () => {
        await using temp = await createTempRoot();
        const command = await writeStandInTool({
          root: temp.root,
          lines: [
            `printf '%s' '${STDOUT_MARKER}'`,
            `( sleep ${String(TAIL_DELAY_SECONDS,)}; printf '%s' '${STDERR_MARKER}' >&2 ) >/dev/null &`,
            `exit ${String(STAND_IN_FAILURE_STATUS,)}`,
          ],
        },);
        const message = await commandFailureMessage({ args: STAND_IN_ARGS, command, },);
        expect(message,).toContain(`exited with status ${String(STAND_IN_FAILURE_STATUS,)}`,);
        expect(message,).toContain(`Standard error:\n${STDERR_MARKER}`,);
        expect(message,).not.toContain('truncated',);
        expect(message,).not.toContain(STDOUT_MARKER,);
        expect(message,).not.toContain(INSTALL_COMMAND,);
      },
      name: 'reports non-zero status and stderr after exit without stdout or install advice',
    },),
    it({
      fn: async () => {
        await using temp = await createTempRoot();
        const command = await writeStandInTool({
          root: temp.root,
          lines: [
            `printf '%s%*s%s' '${STDERR_MARKER}' ${String(FLOOD_PADDING_BYTES,)} '' '${STDERR_AFTER_CAP}' >&2`,
            `exit ${String(STAND_IN_FAILURE_STATUS,)}`,
          ],
        },);
        const message = await commandFailureMessage({ args: [], command, },);
        const padding = ' '.repeat(MAX_FIXTURE_COMMAND_STDERR_BYTES - Buffer.byteLength(STDERR_MARKER,),);
        expect(message,).toContain(`exited with status ${String(STAND_IN_FAILURE_STATUS,)}`,);
        expect(message,).toContain(
          `Standard error:\n${STDERR_MARKER}${padding}\n`
            + `[Standard error truncated after ${String(MAX_FIXTURE_COMMAND_STDERR_BYTES,)} bytes]`,
        );
        expect(message,).not.toContain(STDERR_AFTER_CAP,);
      },
      name: 'drains noisy stderr but retains only the bounded byte prefix',
    },),
    it({
      fn: async () => {
        await using temp = await createTempRoot();
        const command = await writeStandInTool({ root: temp.root, lines: ['kill -TERM "$$"',], },);
        const message = await commandFailureMessage({ args: [], command, },);
        expect(message,).toContain('was terminated by SIGTERM',);
        expect(message,).not.toContain('exited with status',);
        expect(message,).not.toContain('Standard error:',);
        expect(message,).not.toContain(INSTALL_COMMAND,);
      },
      name: 'reports the signal that terminated the tool',
    },),
    ...(['standard output', 'standard error',] as const).map(function inheritedPipe(pipe,) {
      return it({
        fn: async () => {
          await using temp = await createTempRoot();
          const command = await writeStandInTool({
            root: temp.root,
            lines: [
              `printf '%s' '${STDERR_MARKER}' >&2`,
              `sleep ${String(HOLD_SECONDS,)} ${pipe === 'standard output' ? '2>/dev/null' : '>/dev/null'} &`,
              'exit 0',
            ],
          },);
          const message = await commandFailureMessage({ args: STAND_IN_ARGS, command, timeoutMilliseconds: TEST_DEADLINE_MS, },);
          expect(message,).toContain(`did not complete within ${String(TEST_DEADLINE_MS,)} ms`,);
          expect(message,).toContain('The tool exited, but its output did not close.',);
          expect(message,).toContain(STDERR_MARKER,);
          expect(message,).not.toContain(INSTALL_COMMAND,);
        },
        name: `reports the deadline after a helper keeps ${pipe} open`,
      },);
    },),
    it({
      fn: async () => {
        await using temp = await createTempRoot();
        const command = await writeStandInTool({
          root: temp.root,
          lines: [
            `printf '%s' '${STDERR_MARKER}' >&2`,
            `exec sleep ${String(HOLD_SECONDS,)}`,
          ],
        },);
        const message = await commandFailureMessage({ args: STAND_IN_ARGS, command, timeoutMilliseconds: TEST_DEADLINE_MS, },);
        expect(message,).toContain(`did not complete within ${String(TEST_DEADLINE_MS,)} ms`,);
        expect(message,).toContain('Check why the tool did not finish.',);
        expect(message,).toContain(STDERR_MARKER,);
        expect(message,).not.toContain(INSTALL_COMMAND,);
      },
      name: 'reports the deadline while the tool is still running',
    },),
    it({
      fn: async () => {
        const message = await commandFailureMessage({ args: [], command: STAND_IN_TOOL_NAME, timeoutMilliseconds: 0, },);
        expect(message,).toContain('has an invalid deadline (0 ms)',);
      },
      name: 'rejects an invalid deadline before starting a tool',
    },),
  ],
  name: 'the fixture command runner',
},);
