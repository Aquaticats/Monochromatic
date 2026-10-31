#!/usr/bin/env node
/**
 Move a Rust crate to a new version, check it, dry-run the publish and commit the bump. Run without arguments for the
 options, or see the README. Nothing is pushed or published.
 */
import { existsSync } from 'node:fs';
import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { resolve } from 'node:path';

import { runCli } from '../src/cli-main.ts';
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
  return await runRelease({
    options: parseReleaseArguments({
      argv,
      root: findRoot,
    }),
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
