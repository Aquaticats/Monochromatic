#!/usr/bin/env node
/**
 Wait for a release workflow run, then for the publish run it started, then check crates.io and the GitHub release. Run
 without arguments for the options, or see the README.
 */
import {
  existsSync,
  readFileSync,
} from 'node:fs';
import { resolve } from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';

import { parseAwaitArguments } from '../src/await-options.ts';
import { awaitRelease } from '../src/await-run.ts';
import { runCli } from '../src/cli-main.ts';
import { loadCrateRegistry } from '../src/crate-registry.ts';
import { findRepositoryRoot } from '../src/repository-root.ts';
import {
  type ProgramCall,
  type ProgramResult,
  spawnProgram,
} from '../src/run-program.ts';
import { awaitUsage } from '../src/usage.ts';

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
 Read the registry of releasable crates that sits in this package.

 @returns the validated registry
 */
function readRegistry() {
  return loadCrateRegistry({
    packageDirectory: resolve(
      import.meta.dirname,
      '..'
    ),
    readText: function readRegistryText(path) {
      return readFileSync(
        path,
        'utf8'
      );
    },
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
 Run the await command on its arguments.

 @param argv - arguments after the script path

 @returns the exit status
 */
async function execute(argv: readonly string[]): Promise<number> {
  return await awaitRelease({
    options: parseAwaitArguments({
      argv,
      root: findRoot,
      registry: readRegistry,
    }),
    run,
    sleep: sleepFor,
    print: console.log,
  });
}

process.exitCode = await runCli({
  argv: process.argv
    .slice(2),
  usage: awaitUsage,
  execute,
  print: console.log,
  printError: console.error,
});
