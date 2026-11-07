/**
 Runs or prints the commands of a pass, with a deadline on each and the total deadline over all of them.

 @module
 */

import {
  FAILURE_MESSAGE_LIMIT,
  FAILURE_STDERR_LIMIT,
  KILL_GRACE_SECONDS,
  MAX_OUTPUT_BYTES,
  MILLISECONDS_PER_SECOND,
  TERMINATED_BY_TIMEOUT,
  TRANSFER_SCOPE_BOUNDS,
} from './constants.ts';
import {
  boundedSeconds,
  type DeadlineClock,
} from './deadline-clock.ts';
import { MoveStopError, } from './errors.ts';
import type { MoveOptions, } from './move-options.ts';
import { isSafeWord, } from './text-checks.ts';
import { childFailureOf, } from './json-text-fields.ts';
import { execFileAsync, } from './exec-file-async.ts';

/**
 One command to show and, in run mode, run.
 */
export type ExecRequest = {
  readonly command: string;
  readonly args: readonly string[];

  /**
   Run inside a bounded systemd scope with `timeout`, the way a transfer runs.
   */
  readonly transfer?: boolean;

  /**
   Per-step deadline in seconds; the default is the transfer or quick-command deadline.
   */
  readonly deadlineSeconds?: number;
};

/**
 The runner one invocation shares across its passes.
 */
export type Runner = {
  /**
   Shows the command and, in run mode, runs it. Returns its output; print mode returns an empty string.
   */
  readonly exec: (request: ExecRequest,) => Promise<string>;

  /**
   Writes a timestamped status line to standard error.
   */
  readonly log: (message: string,) => void;
};

/**
 Quotes a word for display so a shown command can be pasted into a shell.

 @param word - the word
 
 @returns the word, single-quoted when it has unusual characters
 */
function quote(word: string,): string {
  return isSafeWord(word,) ? word : `'${word.replaceAll(
    "'",
    `'\\''`,
  )}'`;
}

/**
 Writes one status line to standard error.

 @param message - the line
 */
function log(message: string,): void {
  console.error(`${new Date().toISOString()} ${message}`,);
}

/**
 Makes the runner of one invocation.

 @param options - the options; `run` decides between printing and running
 
 @param clock - the total-deadline clock
 
 @returns the runner

 @example
 ```ts
 import { createRunner, } from './command-runner.ts';
 
 const result = createRunner({ options, clock, },);
 ```
 */
export function createRunner(
  {
    options,
    clock,
  }: {
    readonly options: MoveOptions;
    readonly clock: DeadlineClock;
  },
): Runner {
  /**
   Runs one command through the program, argument list and bounds the request implies.

   @param request - the command
   
   @returns the command's standard output, or an empty string in print mode
   
   @throws MoveStopError when the total deadline passed, or the command failed or timed out
   */
  async function exec(request: ExecRequest,): Promise<string> {
    if (options.run && (clock.remainingSeconds() <= 0)) {
      throw new MoveStopError(
        `the total deadline of ${String(clock.totalSeconds,)} s was reached before ${request.command} `
          + `${request.args
            .slice(
              0,
              2,
            )
            .join(' ',)}; every earlier step is recorded, and a rerun continues`,
      );
    }
    /**
     The is transfer of this step.
     */
    const isTransfer = request.transfer === true;
    /**
     The step seconds of this step.
     */
    const stepSeconds = request.deadlineSeconds
      ?? (isTransfer ? options.transferDeadlineSeconds : options.commandDeadlineSeconds);
    /**
     The deadline in seconds.
     */
    const seconds = boundedSeconds({
      stepSeconds,
      clock,
    },);
    /**
     The parts of the value this function uses, taken out of it.
     */
    const [program = '', ...argv] = [
      ...(isTransfer && (!options.noScope) ? [
        'systemd-run',
        ...TRANSFER_SCOPE_BOUNDS,
      ] : []),
      ...(isTransfer ? [
        'timeout',
        `--kill-after=${String(KILL_GRACE_SECONDS,)}`,
        String(seconds,),
      ] : []),
      request.command,
      ...request.args,
    ];
    console.log([
      program,
      ...argv,
    ].map(quote,)
      .join(' ',),);
    if (!options.run) {
      return '';
    }
    try {
      /**
       What the call returned.
       */
      const result = await execFileAsync(
        program,
        argv,
        {
          encoding: 'utf8',
          maxBuffer: MAX_OUTPUT_BYTES,
          ...(isTransfer ? {} : { timeout: seconds * MILLISECONDS_PER_SECOND, }),
        },
      );
      return result.stdout;
    }
    catch (error) {
      /**
       The caught failure, in the shape the caller reads.
       */
      const failure = childFailureOf(error,);
      /**
       The limited by total of this step.
       */
      const limitedByTotal = seconds < stepSeconds;
      /**
       The ended of this step.
       */
      const ended = limitedByTotal && ((failure.code === TERMINATED_BY_TIMEOUT) || ((typeof failure.code) !== 'number'))
        ? `the total deadline of ${String(clock.totalSeconds,)} s ended this step: `
        : '';
      throw new MoveStopError(
        `${ended}${[
          request.command,
          ...request.args,
        ].join(' ',)} exited ${String(failure.code,)} `
          + `(${failure.message
            .slice(
              0,
              FAILURE_MESSAGE_LIMIT,
            )}): ${failure.stderr
              .trim()
              .slice(
                0,
                FAILURE_STDERR_LIMIT,
              )}`,
        { cause: error, },
      );
    }
  }

  return {
    exec,
    log,
  };
}
