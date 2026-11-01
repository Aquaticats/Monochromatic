#!/usr/bin/env node
/**
 Append the release commit of a checkout to a remote branch through the GraphQL API, so GitHub signs it. Run without
 arguments for the options, or see the README. The release workflow runs this after the release command.
 */
import { existsSync } from 'node:fs';
import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  resolve,
} from 'node:path';

import { runCli } from '../src/cli-main.ts';
import { parsePushArguments } from '../src/push-options.ts';
import { pushRelease } from '../src/push-run.ts';
import { findRepositoryRoot } from '../src/repository-root.ts';
import {
  type ProgramCall,
  type ProgramResult,
  spawnProgram,
} from '../src/run-program.ts';
import { pushUsage } from '../src/usage.ts';

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
 Read a file's bytes.

 @param path - absolute path

 @returns the file's contents
 */
async function readBytes(path: string): Promise<Uint8Array> {
  return await readFile(path);
}

/**
 Write a file's text, creating its directory.

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
  await mkdir(
    dirname(path),
    { recursive: true }
  );
  await writeFile(
    path,
    text
  );
}

/**
 Run the push command on its arguments.

 @param argv - arguments after the script path

 @returns the exit status
 */
async function execute(argv: readonly string[]): Promise<number> {
  return await pushRelease({
    options: parsePushArguments({
      argv,
      root: findRoot,
    }),
    run,
    readBytes,
    writeText,
    print: console.log,
  });
}

process.exitCode = await runCli({
  argv: process.argv
    .slice(2),
  usage: pushUsage,
  execute,
  print: console.log,
  printError: console.error,
});
