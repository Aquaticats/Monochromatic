/**
 gh child process boundary for Pi Search Fetch.
 
 @module
 */

import { execFile, } from 'node:child_process';
import { promisify, } from 'node:util';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  GhCommandNotRan,
  GhCommandOutcome,
  GhCommandRan,
  GhCommandRequest,
  GhCommandRunner,
  GhCommandRunnerOptions,
} from './github-fetch-types.ts';

import { scratchParent, } from './scratch-parent.ts';
/**
 Logger root for pi-search-fetch after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: ghProcessLogger, },);
 ```
 */
const ghProcessLogger = tagged({ tag: 'pi-search-fetch', },);

/**
 Module logger.
 */
const l = tagged({
  tag: 'gh-process',
  l: ghProcessLogger,
},);

//region Constants

/**
 Default GitHub CLI executable resolved through PATH.
 */
const GH_EXECUTABLE = 'gh';

/**
 Default child deadline in milliseconds, 30 s.

 Chosen so one gh child plus the 30 s DeepSeek request bound leave a real 60 s
 tail of the 120 s whole-call deadline for the Linkup and Exa fallback hops.
 */
const GH_DEADLINE_MILLISECONDS = 30_000;

/**
 Default captured output ceiling in mebibytes.
 */
const GH_OUTPUT_MAX_MEBIBYTES = 32;

/**
 Bytes in one mebibyte, used to express the output ceiling.
 */
const BYTES_PER_MEBIBYTE = 1_048_576;

/**
 Default captured output ceiling in bytes.
 */
const GH_OUTPUT_MAX_BYTES: number = GH_OUTPUT_MAX_MEBIBYTES * BYTES_PER_MEBIBYTE;

/**
 Node diagnostic code reported when the executable cannot be spawned.
 */
const MISSING_EXECUTABLE_CODE = 'ENOENT';

/**
 Node diagnostic code reported when captured output exceeds the ceiling.
 */
const OUTPUT_CEILING_CODE = 'ERR_CHILD_PROCESS_STDIO_MAXBUFFER';

/**
 Node diagnostic code reported when a caller signal cancels the child.
 */
const CANCELLATION_CODE = 'ABORT_ERR';

/**
 Environment variable naming the host gh targets when a command carries no host.
 */
const GH_HOST_VARIABLE = 'GH_HOST';

/**
 Host every mapped GitHub URL names, pinned so an ambient GH_HOST cannot redirect a fetch.
 */
const PINNED_GITHUB_HOST = 'github.com';

/**
 Environment variable that forces terminal-shaped output even when output is redirected.
 */
const GH_FORCE_TTY_VARIABLE = 'GH_FORCE_TTY';

/**
 Environment variable keeping ANSI color in piped output.
 */
const CLICOLOR_FORCE_VARIABLE = 'CLICOLOR_FORCE';

/**
 Environment variable suppressing ANSI color output.
 */
const NO_COLOR_VARIABLE = 'NO_COLOR';

/**
 Value disabling forced color while leaving the variable defined.
 */
const COLOR_DISABLED_VALUE = '0';

/**
 Value gh treats as unset for its forced-terminal variable.
 */
const UNSET_VARIABLE_VALUE = '';

/**
 Character marking a payload as binary even when it decodes as valid UTF-8.
 */
const NUL_CHARACTER = '\u0000';

//endregion Constants

//region Types

/**
 Numeric child exit code read from one rejection.
 */
type ExitCodeResult = {
  /**
   Whether the child completed and reported a numeric exit code.
   */
  readonly found: true;
  /**
   Numeric child exit code.
   */
  readonly exitCode: number;
} | {
  /**
   Whether the child completed and reported a numeric exit code.
   */
  readonly found: false;
};

//endregion Types

/* oxlint-disable typescript/strict-void-return -- node:util.promisify intentionally accepts execFile even though execFile also returns a ChildProcess handle; this wrapper only consumes the promise result. */
/**
 Promise-returning `execFile` used for one child invocation.
 */
const execFileAsync = promisify(execFile,);
/* oxlint-enable typescript/strict-void-return */

//region Public API

/**
 Create one gh command runner with fixed executable, deadline, output ceiling, and working directory.
 
 @param runnerOptions - runner overrides, primarily for tests
 
 @returns runner classifying every child outcome without throwing for child failures
 
 @example
 ```ts
 const runner = createGhCommandRunner({});
 ```
 */
function createGhCommandRunner(
  runnerOptions: GhCommandRunnerOptions,
): GhCommandRunner {
  /**
   Executable resolved through PATH.
   */
  const executable = runnerOptions.executable ?? GH_EXECUTABLE;
  /**
   Child deadline in milliseconds.
   */
  const deadlineMs = runnerOptions.deadlineMs ?? GH_DEADLINE_MILLISECONDS;
  /**
   Captured output ceiling in bytes.
   */
  const maxOutputBytes = runnerOptions.maxOutputBytes ?? GH_OUTPUT_MAX_BYTES;

  return async function runConfiguredGhCommand(
    request: ForeignBorrowed<GhCommandRequest>,
  ): Promise<GhCommandOutcome> {
    /**
     Working directory keeping gh away from ambient repository context.
     */
    const workingDirectory = runnerOptions.cwd ?? await scratchParent();
    /**
     Logger tagged for this child invocation.
     */
    const innerL = tagged({
      tag: createGhCommandRunner.name,
      l,
    },);
    /**
     Argument vector and caller-owned cancellation signal.
     */
    const {
      args,
      signal,
    } = request;
    innerL.debug(`running ${executable} ${args.join(' ',)} in ${workingDirectory}`,);
    try {
      /**
       Completed child result captured as buffers.
       */
      const result = await execFileAsync(
        executable,
        [
          ...args,
        ],
        {
          cwd: workingDirectory,
          encoding: 'buffer',
          env: childEnvironment(),
          maxBuffer: maxOutputBytes,
          timeout: deadlineMs,
          ...(signal === undefined ? {} : { signal, }),
        },
      );
      return completedRun({
        exitCode: 0,
        stdout: toBuffer(result.stdout,),
        stderr: toBuffer(result.stderr,),
      },);
    }
    catch (error: unknown) {
      if (isCancellation({
        error,
        ...(signal === undefined ? {} : { signal, }),
      },))
        throw error;

      /**
       Classified failure outcome for routing and diagnostics.
       */
      const outcome = classifyChildFailure({
        error,
        deadlineMs,
        maxOutputBytes,
        executable,
        workingDirectory,
      },);
      innerL.warn(
        outcome.ran
          ? `${executable} exited with code ${String(outcome.exitCode,)} for args: ${args.join(' ',)}`
          : `${executable} produced no result for args: ${args.join(' ',)}: ${outcome.reason}`,
      );
      return outcome;
    }
  };
}

/**
 Runner using the default gh executable, deadline, output ceiling, and a neutral working directory.
 */
const runGhCommand: GhCommandRunner = createGhCommandRunner({},);

//endregion Public API

//region Outcome construction

/**
 Build the child environment, pinning the variables that change gh routing or output shape.
 
 Measured on gh 2.101.0:
 an ambient `GH_HOST` redirects `gh api` to that host,
 an ambient `GH_FORCE_TTY` swaps raw output for ANSI-colored terminal output and grew one issue
 comment read from 477 bytes to 13990 bytes,
 an empty `GH_FORCE_TTY` value behaves as unset,
 and `CLICOLOR_FORCE=0` keeps `gh pr diff` output free of ANSI escapes.
 
 @returns inherited environment with routing and output variables pinned
 
 @example
 ```ts
 childEnvironment();
 ```
 */
function childEnvironment(): NodeJS.ProcessEnv {
  return {
    ...process.env,
    [GH_HOST_VARIABLE]: PINNED_GITHUB_HOST,
    [GH_FORCE_TTY_VARIABLE]: UNSET_VARIABLE_VALUE,
    [CLICOLOR_FORCE_VARIABLE]: COLOR_DISABLED_VALUE,
    [NO_COLOR_VARIABLE]: '1',
  };
}

/**
 Classify one rejected child invocation.
 
 @param error - rejection from the promisified child boundary
 
 @param deadlineMs - child deadline used for the invocation
 
 @param maxOutputBytes - captured output ceiling used for the invocation
 
 @param executable - executable name used only in safe diagnostics
 
 @returns completed run output for a non-zero exit, otherwise a safe no-run reason
 
 @example
 ```ts
 classifyChildFailure({ error: new Error('spawn gh ENOENT'), deadlineMs: 60000, maxOutputBytes: 1024, executable: 'gh' });
 ```
 */
function classifyChildFailure(
  {
    error,
    deadlineMs,
    maxOutputBytes,
    executable,
    workingDirectory,
  }: {
    readonly error: unknown;
    readonly deadlineMs: number;
    readonly maxOutputBytes: number;
    readonly executable: string;
    readonly workingDirectory: string;
  },
): GhCommandOutcome {
  /**
   Exit code reported by Node for a completed non-zero child.
   */
  const exitCode = failureExitCode(error,);
  if (exitCode.found)
    return completedRun({
      exitCode: exitCode.exitCode,
      stdout: failureBuffer({
        error,
        stream: 'stdout',
      },),
      stderr: failureBuffer({
        error,
        stream: 'stderr',
      },),
    },);

  /**
   Node diagnostic code naming the failure class.
   */
  const diagnosticCode = failureDiagnosticCode(error,);
  if (diagnosticCode === MISSING_EXECUTABLE_CODE)
    return missingRun(
      `gh child did not spawn, so either ${executable} is missing from PATH or working directory ${workingDirectory} is missing: ${caughtValueText(error,)}`,
    );
  if (diagnosticCode === OUTPUT_CEILING_CODE)
    return missingRun(`gh output exceeded the ${String(maxOutputBytes,)} byte capture ceiling`,);
  if (isKilledFailure(error,))
    return missingRun(`gh was terminated after exceeding the ${String(deadlineMs,)}ms deadline`,);
  return missingRun(`gh invocation failed: ${caughtValueText(error,)}`,);
}

/**
 Build one completed run outcome with UTF-8 validation.
 
 @param exitCode - numeric child exit code
 
 @param stdout - captured standard output bytes
 
 @param stderr - captured standard error bytes
 
 @returns completed run outcome
 
 @example
 ```ts
 completedRun({ exitCode: 0, stdout: Buffer.from('# Title'), stderr: Buffer.alloc(0) });
 ```
 */
function completedRun(
  {
    exitCode,
    stdout,
    stderr,
  }: {
    readonly exitCode: number;
    readonly stdout: Buffer;
    readonly stderr: Buffer;
  },
): GhCommandRan {
  /**
   Standard output decoded as UTF-8, lossy for binary payloads.
   */
  const stdoutText = stdout.toString('utf8',);
  return {
    ran: true,
    exitCode,
    stdout: stdoutText,
    stdoutIsText: isRenderableText({
      text: stdoutText,
      bytes: stdout,
    },),
    stdoutByteLength: stdout.length,
    stderr: stderr.toString('utf8',),
  };
}

/**
 Build one no-run outcome.
 
 @param reason - safe explanation
 
 @returns no-run outcome
 
 @example
 ```ts
 missingRun('gh executable gh was not found on PATH');
 ```
 */
function missingRun(reason: string,): GhCommandNotRan {
  return {
    ran: false,
    reason,
  };
}

/**
 Return whether captured output is safe to render as text.
 
 @param text - UTF-8 decoded output
 
 @param bytes - captured output bytes
 
 @returns whether decoding lost no information and the text carries no NUL character
 
 @example
 ```ts
 isRenderableText({ text: 'plain', bytes: Buffer.from('plain') });
 ```
 */
function isRenderableText(
  {
    text,
    bytes,
  }: {
    readonly text: string;
    readonly bytes: Buffer;
  },
): boolean {
  if (text.includes(NUL_CHARACTER,))
    return false;
  return Buffer.from(
    text,
    'utf8',
  )
    .equals(bytes,);
}

//endregion Outcome construction

//region Failure inspection

/**
 Return whether one rejection is a caller cancellation rather than a child failure.
 
 @param error - rejection from the promisified child boundary
 
 @param signal - caller-owned cancellation signal, when supplied
 
 @returns whether the caller cancelled this invocation
 
 @example
 ```ts
 isCancellation({ error: Object.assign(new Error('aborted'), { code: 'ABORT_ERR' }) });
 ```
 */
function isCancellation(
  {
    error,
    signal,
  }: {
    readonly error: unknown;
    readonly signal?: AbortSignal;
  },
): boolean {
  if (signal?.aborted === true)
    return true;
  return failureDiagnosticCode(error,) === CANCELLATION_CODE;
}

/**
 Read one numeric child exit code from a rejection.
 
 @param error - rejection from the promisified child boundary
 
 @returns numeric exit code result, absent when the child never completed
 
 @example
 ```ts
 failureExitCode(Object.assign(new Error('failed'), { code: 1 }));
 ```
 */
function failureExitCode(error: unknown,): ExitCodeResult {
  /**
   Node diagnostic code carried by the rejection.
   */
  const code = failureDiagnosticCode(error,);
  return (typeof code) === 'number'
    ? {
      found: true,
      exitCode: code,
    }
    : { found: false, };
}

/**
 Read one Node diagnostic code from a rejection.
 
 @param error - rejection from the promisified child boundary
 
 @returns diagnostic code, absent when the rejection carries none
 
 @example
 ```ts
 failureDiagnosticCode(Object.assign(new Error('failed'), { code: 'ENOENT' }));
 ```
 */
function failureDiagnosticCode(error: unknown,): unknown {
  if (!isRecord(error,))
    return undefined;
  return error.code;
}

/**
 Return whether Node reported a forced child termination.
 
 @param error - rejection from the promisified child boundary
 
 @returns whether the child was killed
 
 @example
 ```ts
 isKilledFailure(Object.assign(new Error('failed'), { killed: true }));
 ```
 */
function isKilledFailure(error: unknown,): boolean {
  return isRecord(error,)
    && (error.killed === true);
}

/**
 Read one captured output stream from a rejection.
 
 @param error - rejection from the promisified child boundary
 
 @param stream - captured stream name
 
 @returns captured bytes, empty when the rejection carries none
 
 @example
 ```ts
 failureBuffer({ error: Object.assign(new Error('failed'), { stderr: Buffer.from('boom') }), stream: 'stderr' });
 ```
 */
function failureBuffer(
  {
    error,
    stream,
  }: {
    readonly error: unknown;
    readonly stream: 'stdout' | 'stderr';
  },
): Buffer {
  if (!isRecord(error,))
    return Buffer.alloc(0,);
  return toBuffer(error[stream],);
}

/**
 Normalize one captured stream value into bytes.
 
 @param value - stream value reported by Node
 
 @returns captured bytes
 
 @example
 ```ts
 toBuffer('plain text');
 ```
 */
function toBuffer(value: unknown,): Buffer {
  if (Buffer.isBuffer(value,))
    return value;
  if ((typeof value) === 'string')
    return Buffer.from(
      value,
      'utf8',
    );
  return Buffer.alloc(0,);
}

/**
 Return whether one value can be read by string keys.
 
 @param value - unknown rejection payload
 
 @returns whether value is a non-null, non-array object
 
 @example
 ```ts
 isRecord(new Error('failed'));
 ```
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return (value !== null)
    && ((typeof value) === 'object')
    && (!Array.isArray(value,));
}

//endregion Failure inspection

export {
  createGhCommandRunner,
  GH_DEADLINE_MILLISECONDS,
  runGhCommand,
};
