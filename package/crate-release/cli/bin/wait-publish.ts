#!/usr/bin/env node
/**
 Wait for the publish workflow run of a commit, then check crates.io and the GitHub release. Run without arguments for
 the options, or see the README.
 */
import { existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';

import { runCli } from '../src/cli-main.ts';
import { findRepositoryRoot } from '../src/repository-root.ts';
import {
  type ProgramCall,
  type ProgramResult,
  spawnProgram,
} from '../src/run-program.ts';
import { waitUsage } from '../src/usage.ts';
import { parseWaitArguments } from '../src/wait-options.ts';
import { waitForPublish } from '../src/wait-run.ts';

/**
 Find the repository root from the location of this package.

 @returns the nearest ancestor directory that holds the repository marker
 */
function findRoot(): string {
  return findRepositoryRoot({
    startDirectory: resolve(
      import.meta.dirname,
      '..',
    ),
    exists: existsSync,
  });
}

/**
 Run one program call and capture its output.

 @param call - program, arguments and working directory
 
 @returns the exit status and the captured text
 */
async function run(call: ProgramCall): Promise<ProgramResult> {
  return await spawnProgram({
    ...call,
    capture: true,
  });
}

/**
 Wait the given time.

 @param milliseconds - how long to wait
 */
async function sleepFor(milliseconds: number): Promise<void> {
  await sleep(milliseconds);
}

/**
 Run the wait command on its arguments.

 @param argv - arguments after the script path
 
 @returns the exit status
 */
async function execute(argv: readonly string[]): Promise<number> {
  return await waitForPublish({
    options: parseWaitArguments({
      argv,
      root: findRoot,
    }),
    run,
    sleep: sleepFor,
    print: console.log,
  });
}

process.exitCode = await runCli({
  argv: process.argv
    .slice(2),
  usage: waitUsage,
  execute,
  print: console.log,
  printError: console.error,
});
