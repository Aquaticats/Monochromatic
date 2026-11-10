#!/usr/bin/env node
/**
 Executable entry point for the owner-answer command.

 @module
 */

import process from 'node:process';

import {
  EXIT_MISUSE,
  runCli,
} from './cli-entry.ts';
import {
  AnswersError,
  CliInvocationError,
} from './errors.ts';

/**
 Writes one line to standard output.
 
 @param line - Text the command printed.
 
 @example
 ```ts
 write('recorded in doc/handover/questions-for-the-owner.md\n');
 ```
 */
function write(line: string,): void {
  process.stdout
    .write(line,);
}

/**
 Maps a thrown value to the status the process exits with, printing its words.
 
 @param error - Value caught from the command.
 
 @returns The exit status for that failure.
 
 @throws Re-throws a value that is not an error.
 
 @example
 ```ts
 const status = statusOf(new AnswersError('no such heading'));
 ```
 */
function statusOf(error: unknown,): number {
  if ((error instanceof CliInvocationError) || (error instanceof AnswersError)) {
    process.stderr
      .write(`${error.message}\n`,);
    return EXIT_MISUSE;
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
      streams: { write, },
    },);
  }
  catch (error) {
    return statusOf(error,);
  }
}

process.exitCode = await main();
