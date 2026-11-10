/**
 Disposable stand-in executables for the tests that run a subprocess, and the
 read that waits for a failure the daemon logs about one.

 A test that drove the real `ydotool` or `kstart` would inject keys into the
 owner's session or start applications, so the tests put a shell script of that
 name first on `PATH` instead. The stand-in records the argument vector it was
 given and reports the exit status and stderr the test asked for.

 @module
 */

import {
  chmod,
  mkdtemp,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  delimiter,
  join,
} from 'node:path';

import {
  wait,
  withTimeout,
} from '@monochromatic-dev/async-time-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import { flushLogs, } from '../dist/final/node/testing.mjs';

/**
 Pause between reads of the daemon's log, in milliseconds.
 */
const POLL_MILLISECONDS = 20;

/**
 Mode a stand-in needs to be run as a program.
 */
const EXECUTABLE_FILE_MODE = 0o755;

/**
 Environment variable carrying the file the stand-in records its arguments in.
 */
const ARGUMENTS_VARIABLE = 'KEY_HELPER_FAKE_ARGUMENTS';

/**
 Environment variable carrying the text the stand-in writes to stderr.
 */
const STANDARD_ERROR_VARIABLE = 'KEY_HELPER_FAKE_STANDARD_ERROR';

/**
 Script body the stand-in runs.

 The parts that vary arrive through the environment, so no shell quoting is
 needed for a scratch path or a message: shell variables are read as data.
 */
const STAND_IN_SCRIPT = `#!/bin/sh
for argument in "$@"; do printf '%s\\n' "$argument"; done > "$KEY_HELPER_FAKE_ARGUMENTS"
if [ -n "$KEY_HELPER_FAKE_STANDARD_ERROR" ]; then printf '%s\\n' "$KEY_HELPER_FAKE_STANDARD_ERROR" >&2; fi
`;

/**
 How one stand-in reports itself.
 
 @example
 ```ts
 const shape: FakeCommandShape = { name: 'ydotool', exitCode: 7, standardError: 'down' };
 ```
 */
export type FakeCommandShape = {
  /**
   Executable name the stand-in answers to on `PATH`.
   */
  readonly name: string;
  /**
   Exit status the stand-in reports.
   */
  readonly exitCode: number;
  /**
   Text the stand-in writes to stderr, or the empty string to stay silent.
   */
  readonly standardError: string;
  /**
   Shell lines the stand-in runs after recording its arguments and before it
   exits, for cases that need it to produce output or keep running.
   */
  readonly extraLines?: readonly string[];
};

/**
 A stand-in installed on `PATH`, plus the way to put `PATH` back.
 
 @example
 ```ts
 const fake = await installFakeCommand({ name: 'ydotool', exitCode: 7, standardError: '' });
 const arguments = await fake.readArguments();
 fake.remove();
 ```
 */
export type FakeCommand = {
  /**
   Arguments the stand-in was given, in order.
   */
  readonly readArguments: () => Promise<readonly string[]>;
  /**
   Put `PATH` and the fixture's environment variables back as they were.
   */
  readonly remove: () => void;
  /**
   Dispose the stand-in, which restores `PATH` for the next case.
   */
  readonly [Symbol.dispose]: () => void;
};

/**
 File error code naming a recording file the stand-in never created.
 */
const MISSING_FILE_CODE = 'ENOENT';

/**
 Whether a read failed because the file does not exist.
 
 @param error - Value a read rejected with
 
 @returns True for a missing file, false for every other failure
 
 @example
 ```ts
 isMissingFile(Object.assign(new Error('absent'), { code: 'ENOENT' }));
 ```
 */
function isMissingFile(error: unknown): boolean {
  return Error.isError(error)
    && ('code' in error)
    && (error.code === MISSING_FILE_CODE);
}

/**
 Read the argument vector the stand-in recorded.
 
 @param argumentsPath - File the stand-in wrote into
 
 @returns One argument per recorded line, empty when the stand-in never ran
 
 @example
 ```ts
 await readRecordedArguments({ argumentsPath: '/scratch/arguments.txt' });
 ```
 */
async function readRecordedArguments({ argumentsPath }: {
  readonly argumentsPath: string;
}): Promise<readonly string[]> {
  try {
    /**
     Recorded arguments, one per line.
     */
    const recorded = (await readFile(
      argumentsPath,
      'utf8',
    )).split('\n');
    // A trailing newline leaves one empty field at the end.
    return recorded.slice(
      0,
      -1,
    );
  } catch (error) {
    // A stand-in that was never reached wrote nothing: that is an empty
    // argument vector, not a failure to report. Anything else is real.
    if (isMissingFile(error)) {
      return [];
    }
    throw error;
  }
}

/**
 Install one stand-in executable first on `PATH`.
 
 @param name - Executable name to answer to
 
 @param exitCode - Exit status the stand-in reports
 
 @param standardError - Text it writes to stderr, or the empty string
 
 @param extraLines - Shell lines it runs before exiting, or nothing
 
 @returns The installed stand-in and the way to remove it
 
 @example
 ```ts
 const fake = await installFakeCommand({ name: 'kstart', exitCode: 3, standardError: '' });
 ```
 */
export async function installFakeCommand({
  name,
  exitCode,
  standardError,
  extraLines = [],
}: FakeCommandShape): Promise<FakeCommand> {
  /**
   Disposable folder holding the stand-in and its recording file.
   */
  const directory = await mkdtemp(join(
    scratchRoot(),
    'key-helper-fake-command-',
  ));
  /**
   File the stand-in writes its argument vector into.
   */
  const argumentsPath = join(
    directory,
    'arguments.txt',
  );
  /**
   Path the stand-in is installed at.
   */
  const executablePath = join(
    directory,
    name,
  );
  await writeFile(
    executablePath,
    `${STAND_IN_SCRIPT}${extraLines.join('\n')}\nexit ${String(exitCode)}\n`,
    'utf8',
  );
  await chmod(
    executablePath,
    EXECUTABLE_FILE_MODE,
  );
  /**
   `PATH` as it was, restored when the stand-in is disposed.
   */
  const originalPath = process.env
    .PATH
    ?? '';
  process.env[ARGUMENTS_VARIABLE] = argumentsPath;
  process.env[STANDARD_ERROR_VARIABLE] = standardError;
  process.env
    .PATH = `${directory}${delimiter}${originalPath}`;
  /**
   Restoration shared by `remove` and the disposer, so a case that calls one
   cannot leave `PATH` changed for the next.
   */
  function remove(): void {
    process.env
      .PATH = originalPath;
    delete process.env
      .KEY_HELPER_FAKE_ARGUMENTS;
    delete process.env
      .KEY_HELPER_FAKE_STANDARD_ERROR;
  }
  return {
    readArguments: async function recorded(): Promise<readonly string[]> {
      return await readRecordedArguments({ argumentsPath, });
    },
    remove,
    [Symbol.dispose]: remove,
  };
}

/**
 Standard-error text one spy recorded, in call order.
 
 @param spy - Recorder over `process.stderr.write`
 
 @returns Each written chunk as text
 
 @example
 ```ts
 capturedStandardError(ctx.sinon.spy(process.stderr, 'write'));
 ```
 */
export function capturedStandardError(spy: {
  readonly args: readonly (readonly unknown[])[];
}): readonly string[] {
  return spy.args
    .map(function chunkText(call: readonly unknown[]): string {
      return String(call[0]);
    });
}

/**
 Read the lines a logged failure produced, until one names it, or the deadline
 passes.
 
 A program the daemon starts reports its own ending after the call that started
 it has returned, so a case has to wait for the log line rather than read the
 recorder once.
 
 @param spy - Recorder over the daemon's log destination
 
 @param matches - Whether a line names the failure the case is waiting for
 
 @param deadlineMilliseconds - Longest the case waits for it
 
 @returns The lines that matched, empty when none arrived in time
 
 @example
 ```ts
 await reportedUntil({ spy: standardError, matches: namesFailure, deadlineMilliseconds: 5000 });
 ```
 */
export async function reportedUntil({
  spy,
  matches,
  deadlineMilliseconds,
}: {
  readonly spy: { readonly args: readonly (readonly unknown[])[]; };
  readonly matches: (line: string) => boolean;
  readonly deadlineMilliseconds: number;
}): Promise<readonly string[]> {
  return await withTimeout({
    promise: (async function pollUntilReported(): Promise<readonly string[]> {
      /**
       Longest the poll keeps reading before the deadline above ends it.
       */
      const attempts = Math.ceil(deadlineMilliseconds / POLL_MILLISECONDS);
      for (let attempt = 0; attempt < attempts; attempt += 1) {
        // oxlint-disable-next-line eslint/no-await-in-loop -- each read is of the log after the writes before it, so a later read supersedes an earlier one; see doc/lint/sequential-awaits.md
        await flushLogs();
        /**
         Lines the daemon has reported so far.
         */
        const reported = capturedStandardError(spy)
          .filter(matches);
        if (reported.length > 0) {
          return reported;
        }
        // oxlint-disable-next-line eslint/no-await-in-loop -- the interval is the design: the line is awaited after it, never concurrently; see doc/lint/sequential-awaits.md
        await wait(POLL_MILLISECONDS,);
      }
      return [];
    })(),
    ms: deadlineMilliseconds,
    label: 'the failure log line',
  },);
}
