#!/usr/bin/env node
/**
 Start the release workflow for a Rust crate on GitHub and print the URL of its run. Run without arguments for the
 options, or see the README. Nothing is compiled or committed on this machine.
 */
import { existsSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';

import { runCli } from '../src/cli-main.ts';
import {
  type CrateRegistry,
  findCrate,
  loadCrateRegistry,
} from '../src/crate-registry.ts';
import { assertDependenciesReachCratesIo } from '../src/crates-io-dependency-check.ts';
import { parseDispatchArguments } from '../src/dispatch-options.ts';
import { dispatchRelease } from '../src/dispatch-run.ts';
import { findRepositoryRoot } from '../src/repository-root.ts';
import {
  type ProgramCall,
  type ProgramResult,
  spawnProgram,
} from '../src/run-program.ts';
import { dispatchUsage } from '../src/usage.ts';

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
 Read a file as text.

 @param path - absolute path

 @returns the file's text
 */
async function readUtf8(path: string): Promise<string> {
  return await readFile(
    path,
    'utf8'
  );
}

/**
 Read the registry of releasable crates that sits in this package.

 @returns the validated registry
 */
async function readRegistry(): Promise<CrateRegistry> {
  return await loadCrateRegistry({
    packageDirectory: resolve(
      import.meta.dirname,
      '..'
    ),
    readText: readUtf8,
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
 Run one program call with this command's own streams, so a step that takes an hour stays visible while it runs.

 @param call - program, arguments and working directory

 @returns the exit status, with no captured output because the streams went to the terminal
 */
async function runStreaming(call: ProgramCall): Promise<ProgramResult> {
  return await spawnProgram({
    ...call,
    capture: false,
  });
}

/**
 Run the dispatch command on its arguments.

 @param argv - arguments after the script path

 @returns the exit status
 */
async function execute(argv: readonly string[]): Promise<number> {
  /**
   The registry, read before the arguments so the parse can stay synchronous.
   */
  const registry = await readRegistry();
  /**
   The validated options, parsed before anything is run.
   */
  const options = parseDispatchArguments({
    argv,
    root: findRoot,
    registry: function knownRegistry() {
      return registry;
    },
  });
  // The release job refuses this too, on a runner; refusing it here costs no workflow run.
  await assertDependenciesReachCratesIo({
    crateName: options.crate,
    crateDirectory: findCrate({
      registry,
      name: options.crate
    })
      .crateDirectory,
    to: options.to,
    registry,
    root: options.root,
    readText: readUtf8,
  });
  return await dispatchRelease({
    options,
    run,
    runGate: runStreaming,
    print: console.log,
  });
}

process.exitCode = await runCli({
  argv: process.argv
    .slice(2),
  usage: dispatchUsage,
  execute,
  print: console.log,
  printError: console.error,
});
