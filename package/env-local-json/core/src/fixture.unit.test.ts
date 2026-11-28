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

 The deadline's own guards are here too: the runner destroys both pipes,
 signals the tool's whole process group so a helper it left is stopped with it,
 and reports what it could not stop. A stand-in child below drives the failure
 paths no real tool produces on demand, such as a failed pipe read.

 @module
 */

import { Buffer, } from 'node:buffer';
import type { ChildProcess, } from 'node:child_process';
import { existsSync, } from 'node:fs';
import {
  chmod,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { PassThrough, } from 'node:stream';
import { setTimeout as delay, } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createTempRoot,
  FIXTURE_COMMAND_TIMEOUT_MS,
  FixtureCommandError,
  MAX_FIXTURE_COMMAND_STDERR_BYTES,
  runCommand,
  type SignalProcessGroup,
} from './fixture.ts';

//region Constants

/**
 Name of the stand-in key, the one key a stand-in store declares.
 */
const STAND_IN_KEY_NAME = 'STAND_IN_NAME';

/**
 Start of the stand-in value, up to the point where the stand-in cuts the
 document: the head ends inside this value, so a join that stops there holds
 text no reader can parse.
 */
const STAND_IN_VALUE_HEAD = 'stand-in-value-';

/**
 End of that value, which the stand-in prints after its direct child has ended.
 */
const STAND_IN_VALUE_TAIL = 'and-its-own-tail';

/**
 Values the stand-in store declares, which is what a cut store cannot parse to.
 */
const STAND_IN_VALUES: Readonly<Record<string, string>> = {
  [STAND_IN_KEY_NAME]: `${STAND_IN_VALUE_HEAD}${STAND_IN_VALUE_TAIL}`,
};

/**
 Whole document the stand-in tool prints, written by the same JSON encoder a
 store is written with so no part of it is written twice.
 */
const WHOLE_STORE = JSON.stringify(STAND_IN_VALUES,);

/**
 First half of that document: everything before the end of its one value. A
 join that stops here holds text no reader can parse.
 */
const STORE_HEAD = WHOLE_STORE.slice(
  0,
  WHOLE_STORE.indexOf(STAND_IN_VALUE_TAIL,),
);

/**
 Second half: the rest of that value and the document's close, which the stand-in
 prints after its direct child has ended.
 */
const STORE_TAIL = WHOLE_STORE.slice(STORE_HEAD.length,);

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

/**
 Deadline this issue replaced: under it, `sops --encrypt` calls of the driver's
 unit suite at `CPUQuota=400%` failed falsely (issue 6410).
 */
const PREVIOUS_DEFAULT_TIMEOUT_MS = 2_000;

/**
 How far past {@link PREVIOUS_DEFAULT_TIMEOUT_MS} the slow stand-in waits, so
 its call measurably outlasts that old default and still sits far inside the
 shipped one.
 */
const SLOW_CALL_MARGIN_MS = 250;
/** Finite so a missing deadline gives a failed assertion instead of stranding a tool. */
const HOLD_SECONDS = (4 * TEST_DEADLINE_MS) / MILLISECONDS_PER_SECOND;

/**
 Seconds the tool and its helper wait before each writes a survival marker,
 well after {@link TEST_DEADLINE_MS} has passed.
 */
const SURVIVAL_DELAY_SECONDS = (2 * TEST_DEADLINE_MS) / MILLISECONDS_PER_SECOND;

/**
 Milliseconds the survival test waits past the markers' own delay before
 reading them, so a process the runner failed to stop has written its marker.
 */
const SURVIVAL_CHECK_DELAY_MS = (3 * MILLISECONDS_PER_SECOND) / 2;

/**
 Group id the stand-in child reports, chosen so the group signal names a number
 no process of this machine owns.
 */
const STAND_IN_GROUP_ID = 4_242;

/**
 Signal the deadline must send: a stop a tool can ignore would leave the wait it
 already abandoned alive.
 */
const EXPECTED_KILL_SIGNAL = 'SIGKILL';

/**
 Failure a stand-in pipe read reports.
 */
const STAND_IN_PIPE_FAILURE = 'stand-in pipe failure';

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

/**
 A stand-in child and the two pipes the test can fail.
 */
type StandInTool = {
  /**
   Child the runner is made to start instead of a tool.
   */
  readonly child: ChildProcess;

  /**
   Standard output pipe the test can fail.
   */
  readonly stdout: PassThrough;

  /**
   Standard error pipe the test can fail.
   */
  readonly stderr: PassThrough;
};

/**
 Builds a child no real tool produces on demand: its pipes never close by
 themselves, so a failure the runner cleans up after can be raised here.

 The events the runner waits for come from an `EventTarget`, which is why the
 runner's child surface is only what `once` and the two pipes need.

 @param pid - group id the child reports, absent when the runner must not try to stop it

 @returns the stand-in child and its pipes
 */
function standInChild({ pid }: { readonly pid?: number; } = {},): StandInTool {
  /**
   Events the runner waits for on this child.
   */
  const events = new EventTarget();
  /**
   Standard output pipe the test can fail.
   */
  const stdout = new PassThrough();
  /**
   Standard error pipe the test can fail.
   */
  const stderr = new PassThrough();
  /**
   Child surface: the pipes, the exit state a stand-in never reaches, and the
   event methods a real child inherits from `EventEmitter` or `EventTarget`.
   */
  const child = Object.freeze({
    addEventListener: events.addEventListener.bind(events,),
    dispatchEvent: events.dispatchEvent.bind(events,),
    exitCode: null,
    pid,
    removeEventListener: events.removeEventListener.bind(events,),
    signalCode: null,
    stderr,
    stdout,
    unref: function unref(): void {
      // A stand-in has no operating-system handle to release.
    },
  },) as unknown as ChildProcess;
  return {
    child,
    stderr,
    stdout,
  };
}

/**
 Runs the fixture runner against a stand-in child whose named pipe fails.

 The runner attaches its pipe listeners before it awaits the child, so the
 failure raised here reaches them instead of the stream's unhandled-error path.

 @param pipe - pipe whose read fails
 @param signalGroup - signaler the runner is given, the operating system's kill when absent

 @returns the rejection message and the child the runner used
 */
async function standInChildFailure({
  pipe,
  signalGroup,
}: {
  readonly pipe: 'standard output' | 'standard error';
  readonly signalGroup?: SignalProcessGroup;
},): Promise<{ readonly message: string; readonly standIn: StandInTool; }> {
  /**
   The stand-in child and its pipes.
   */
  const standIn = standInChild({ pid: STAND_IN_GROUP_ID, },);
  /**
   The runner's pending call, which the pipe failure below rejects.
   */
  const pending = runCommand({
    args: [],
    command: STAND_IN_TOOL_NAME,
    spawnTool: function startStandInChild(): ChildProcess {
      return standIn.child;
    },
    ...(signalGroup === undefined ? {} : { signalGroup, }),
  },);
  /**
   The pipe the test fails, named so the runner reports which read broke.
   */
  const stream = pipe === 'standard output' ? standIn.stdout : standIn.stderr;
  stream.emit('error', new Error(STAND_IN_PIPE_FAILURE,),);
  try {
    await pending;
  }
  catch (error) {
    if (!(error instanceof FixtureCommandError))
      throw error;
    return {
      message: error.message,
      standIn,
    };
  }
  throw new Error('expected the fixture runner to reject a failed pipe read',);
}

/**
 Delay one call asked a replaced timer for, and that call.
 */
type CallUnderImmediateDeadline = {
  /**
   Delay the runner asked its deadline timer for, in milliseconds, and how many
   timers it asked for, so the test reads the one timer a call arms.
   */
  readonly armed: { calls: number; delayMilliseconds: number; };

  /**
   Pending fixture call.
   */
  readonly pending: Promise<unknown>;
};

/**
 Runs one fixture command with the deadline timer replaced for the call's own
 synchronous part, and reports the delay the runner armed.

The runner arms its deadline before its first `await`, so the replacement is
installed, used and put back before anything else in this process runs: no
other test's timer and no harness timer ever sees it. The replacement hands the
runner's own callback to `setImmediate` instead of arming a timer, so a shipped
default of seconds costs this test none of them.

@param options - invocation whose deadline is measured
@returns failure message and the delay the runner asked for
*/
async function failureUnderImmediateDeadline(
  options: Parameters<typeof runCommand>[0],
): Promise<{ readonly delayMilliseconds: number; readonly message: string; }> {
  const {
    armed,
    pending,
  } = startUnderImmediateDeadline(options,);
  try {
    await pending;
  }
  catch (error) {
    if (!(error instanceof FixtureCommandError))
      throw error;
    // One call arms one deadline. A second timer would make the delay below the
    // wrong one, so the count is part of what the control proves.
    expect(armed.calls,).toBe(1,);
    return {
      delayMilliseconds: armed.delayMilliseconds,
      message: error.message,
    };
  }
  throw new Error(`Expected fixture command ${options.command} to reject, but it returned normally.`,);
}

/**
 Starts one fixture command with the deadline timer replaced, and puts the
 process timer back before the call is awaited.

@param options - invocation to start

@returns the delay the runner asks for and the pending call
*/
function startUnderImmediateDeadline(
  options: Parameters<typeof runCommand>[0],
): CallUnderImmediateDeadline {
  /**
   The process timer the replacement puts back, in the same synchronous part it
   was taken in.
   */
  const realSetTimeout = globalThis.setTimeout;
  const armed = {
    calls: 0,
    delayMilliseconds: Number.NaN,
  };
  using _putTheTimerBack = {
    [Symbol.dispose]: function putTheTimerBack(): void {
      globalThis.setTimeout = realSetTimeout;
    },
  };
  globalThis.setTimeout = function immediateDeadline(
    callback: () => void,
    milliseconds?: number,
  ): ReturnType<typeof setImmediate> {
    armed.calls += 1;
    armed.delayMilliseconds = milliseconds ?? Number.NaN;
    return setImmediate(callback,);
  } as unknown as typeof globalThis.setTimeout;
  const pending = runCommand(options,);
  return {
    armed,
    pending,
  };
}

//endregion Helpers

await describe({
  children: [
    it({
      fn: async () => {
        // The split below is what makes the first test able to fail: a head that
        // were already a whole document would parse, and a join stopping there
        // would look complete.
        expect(JSON.parse(WHOLE_STORE,),).toEqual(STAND_IN_VALUES,);
        expect(`${STORE_HEAD}${STORE_TAIL}`,).toBe(WHOLE_STORE,);
        expect(function parseHead(): void {
          JSON.parse(STORE_HEAD,);
        },).toThrow();
      },
      name: 'splits its stand-in store inside the value, not between tokens',
    },),
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
        await using temp = await createTempRoot({ prefix: 'env-local-json-run-command-stop-', },);
        /**
         Marker the direct child writes when it outlives the deadline.
         */
        const toolMarker = join(temp.root, 'tool-survived',);
        /**
         Marker the helper writes when it outlives the deadline.
         */
        const helperMarker = join(temp.root, 'helper-survived',);
        const command = await writeStandInTool({
          root: temp.root,
          lines: [
            `( sleep ${String(SURVIVAL_DELAY_SECONDS,)}; printf '%s' 'helper' > '${helperMarker}' ) &`,
            `sleep ${String(SURVIVAL_DELAY_SECONDS,)}`,
            `printf '%s' 'tool' > '${toolMarker}'`,
            'exit 0',
          ],
        },);
        const message = await commandFailureMessage({ args: [], command, timeoutMilliseconds: TEST_DEADLINE_MS, },);
        expect(message,).toContain(`did not complete within ${String(TEST_DEADLINE_MS,)} ms`,);
        // The deadline passes long before either writer's own delay ends, so a
        // process the runner left alive writes its marker in the window below.
        await delay(SURVIVAL_CHECK_DELAY_MS,);
        expect(existsSync(toolMarker,),).toBe(false,);
        expect(existsSync(helperMarker,),).toBe(false,);
      },
      name: 'stops the tool and the helper it left when the deadline passes',
      timeout: (4 * TEST_DEADLINE_MS) + SURVIVAL_CHECK_DELAY_MS,
    },),
    it({
      fn: async () => {
        const { message, standIn, } = await standInChildFailure({ pipe: 'standard output', },);
        expect(message,).toContain('could not read standard output',);
        // The wait is abandoned, so neither pipe may stay open to a tool whose
        // output no caller will ever see.
        expect(standIn.stdout.destroyed,).toBe(true,);
        expect(standIn.stderr.destroyed,).toBe(true,);
      },
      name: 'destroys both output pipes when it abandons a tool',
    },),
    it({
      fn: async () => {
        /**
         Signals the runner sent and the group each named.
         */
        const signals: { readonly groupId: number; readonly signal: NodeJS.Signals; }[] = [];
        await standInChildFailure({
          pipe: 'standard error',
          signalGroup: function recordSignal({ processGroupId, signal, },) {
            signals.push({
              groupId: processGroupId,
              signal,
            },);
          },
        },);
        expect(signals,).toEqual([{
          groupId: -STAND_IN_GROUP_ID,
          signal: EXPECTED_KILL_SIGNAL,
        },],);
      },
      name: 'signals the tool\'s whole process group when it abandons a tool',
    },),
    it({
      fn: async () => {
        const { message, } = await standInChildFailure({
          pipe: 'standard output',
          signalGroup: function refuseSignal() {
            throw new Error('EPERM from the stand-in signaler',);
          },
        },);
        expect(message,).toContain(`Could not stop the tool's process group ${String(-STAND_IN_GROUP_ID,)}`,);
        expect(message,).toContain('EPERM from the stand-in signaler',);
      },
      name: 'reports a process group it could not stop',
    },),
    ...(['standard output', 'standard error',] as const).map(function failedPipe(pipe,) {
      return it({
        fn: async () => {
          const { message, } = await standInChildFailure({ pipe, },);
          expect(message,).toContain(`could not read ${pipe}`,);
          expect(message,).toContain(STAND_IN_PIPE_FAILURE,);
          expect(message,).not.toContain(INSTALL_COMMAND,);
        },
        name: `reports a failed read of ${pipe}`,
      },);
    },),
    it({
      fn: async () => {
        const {
          delayMilliseconds,
          message,
        } = await failureUnderImmediateDeadline({
          args: [],
          command: STAND_IN_TOOL_NAME,
          spawnTool: function startStandInChild(): ChildProcess {
            return standInChild().child;
          },
        },);
        // A caller that names no deadline gets the shipped constant, and a tool
        // whose pipes never close is failed by it. The replacement timer says
        // which delay the runner armed, so the test proves the shipped default
        // rather than waiting it out.
        expect(delayMilliseconds,).toBe(FIXTURE_COMMAND_TIMEOUT_MS,);
        expect(message,).toContain(`did not complete within ${String(FIXTURE_COMMAND_TIMEOUT_MS,)} ms`,);
        expect(message,).toContain('Check why the tool did not finish.',);
      },
      name: 'fails a tool whose output never closes at the shipped default, without waiting it',
    },),
    it({
      fn: async () => {
        // The bound is a hang guard, not a speed limit: a call the previous
        // two-second default failed must pass under the shipped one.
        expect(FIXTURE_COMMAND_TIMEOUT_MS,).toBeGreaterThan(PREVIOUS_DEFAULT_TIMEOUT_MS,);
        await using temp = await createTempRoot();
        const command = await writeStandInTool({
          root: temp.root,
          lines: [
            `sleep ${String((PREVIOUS_DEFAULT_TIMEOUT_MS + SLOW_CALL_MARGIN_MS) / MILLISECONDS_PER_SECOND,)}`,
            `printf '%s' '${STDOUT_MARKER}'`,
          ],
        },);
        const startedAt = performance.now();
        const result = await runCommand({ args: [], command, },);
        const elapsed = performance.now() - startedAt;
        expect(result.stdout,).toBe(STDOUT_MARKER,);
        // The positive control: this call really did outlast the old default, so
        // its success measures the shipped default's room and not a fast host.
        expect(elapsed,).toBeGreaterThan(PREVIOUS_DEFAULT_TIMEOUT_MS,);
      },
      name: 'allows a call slower than the two-second default this issue replaced',
      timeout: 4 * (PREVIOUS_DEFAULT_TIMEOUT_MS + SLOW_CALL_MARGIN_MS),
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
