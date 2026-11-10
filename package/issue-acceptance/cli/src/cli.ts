#!/usr/bin/env node
/**
 Executable entry point for the issue-acceptance command.

 Prints the failure's own words on standard error
 and exits with the status the failure class means,
 so a session reads the reason without a stack trace.

 @module
 */

import process from 'node:process';

import {
  EXIT_FAILED,
  EXIT_MISUSE,
  runCli,
} from './cli-entry.ts';
import {
  CliInvocationError,
  PlanError,
} from './errors.ts';

/**
 Writes one line to standard output.
 
 @param line - Text the command printed.
 
 @example
 ```ts
 write('#1 closed\n');
 ```
 */
function write(line: string,): void {
  process.stdout
    .write(line,);
}

/**
 Writes one line to standard error.
 
 @param line - Text the command refused with.
 
 @example
 ```ts
 errorWrite('settle needs --plan FILE\n');
 ```
 */
function errorWrite(line: string,): void {
  process.stderr
    .write(line,);
}

/**
 Maps a thrown value to the status the process exits with, printing its words.
 
 @param error - Value caught from the command.
 
 @returns The exit status for that failure.
 
 @throws Re-throws a value that is not an error.
 
 @example
 ```ts
 const status = statusOf(new PlanError('x'));
 ```
 */
function statusOf(error: unknown,): number {
  if ((error instanceof CliInvocationError) || (error instanceof PlanError)) {
    errorWrite(`${error.message}\n`,);
    return EXIT_MISUSE;
  }
  if (Error.isError(error,)) {
    errorWrite(`${error.message}\n`,);
    return EXIT_FAILED;
  }
  throw error;
}

/**
 Runs the command and reports its status.
 
 @returns The status the process should exit with.
 
 @example
 ```ts
 process.exitCode = await main();
 ```
 */
async function main(): Promise<number> {
  try {
    return await runCli({
      arguments: process.argv
        .slice(2,),
      streams: {
        write,
        errorWrite,
      },
    },);
  }
  catch (error) {
    return statusOf(error,);
  }
}

process.exitCode = await main();
