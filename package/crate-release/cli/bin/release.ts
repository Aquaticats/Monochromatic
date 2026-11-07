#!/usr/bin/env node
/**
 Move a Rust crate to a new version, check it, dry-run the publish and commit the bump. Run without arguments for the
 options, or see the README. Nothing is pushed or published. This is what the release workflow runs on a GitHub runner;
 start a release with the dispatch command, not by running this on a developer machine.
 */
import { existsSync } from 'node:fs';
import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { resolve } from 'node:path';

import { runCli } from '../src/cli-main.ts';
import {
  type CrateRegistry,
  loadCrateRegistry,
} from '../src/crate-registry.ts';
import { assertDependenciesReachCratesIo } from '../src/crates-io-dependency-check.ts';
import { parseReleaseArguments } from '../src/release-options.ts';
import { runRelease } from '../src/release-run.ts';
import type { ReleaseStep } from '../src/release-steps.ts';
import { findRepositoryRoot } from '../src/repository-root.ts';
import { spawnProgram } from '../src/run-program.ts';
import { releaseUsage } from '../src/usage.ts';

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
async function readText(path: string): Promise<string> {
  return await readFile(
    path,
    'utf8',
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
    readText,
  });
}

/**
 Write a file's text.

 @param path - absolute path
 
 @param text - new contents
 */
async function writeText({
  path,
  text,
}: {
  readonly path: string;
  readonly text: string;
}): Promise<void> {
  await writeFile(
    path,
    text,
  );
}

/**
 Run one release step with the caller's streams, so the step's own output shows.

 @param step - program call to run
 
 @returns the step's exit status
 */
async function runStep(step: ReleaseStep): Promise<{ readonly status: number }> {
  return await spawnProgram({
    program: step.program,
    args: step.args,
    cwd: step.cwd,
    capture: false,
  });
}

/**
 Run the release command on its arguments.

 @param argv - arguments after the script path
 
 @returns the exit status
 */
async function execute(argv: readonly string[]): Promise<number> {
  /**
   The registry, read before the arguments so the parse can stay synchronous.
   */
  const registry = await readRegistry();
  /**
   The validated options, parsed before anything is read or written.
   */
  const options = parseReleaseArguments({
    argv,
    root: findRoot,
    registry: function knownRegistry() {
      return registry;
    },
  });
  // A release that goes to crates.io needs every dependency in the repository to be there too, so it is refused here,
  // before a file is edited.
  await assertDependenciesReachCratesIo({
    crateName: options.crateName,
    crateDirectory: options.crateDirectory,
    to: options.to,
    registry,
    root: options.root,
    readText,
  });
  return await runRelease({
    options,
    readText,
    writeText,
    runStep,
    print: console.log,
  });
}

process.exitCode = await runCli({
  argv: process.argv
    .slice(2),
  usage: releaseUsage,
  execute,
  print: console.log,
  printError: console.error,
});
