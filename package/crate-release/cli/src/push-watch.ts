import { posix } from 'node:path';

import {
  cargoProgram,
  manifestFileName,
} from './constants.ts';
import { CrateReleaseError } from './error.ts';
import type { PushOptions } from './push-options.ts';
import type {
  ProgramCall,
  ProgramResult,
} from './run-program.ts';
import {
  buildCargoMetadataArguments,
  mergeWatchSets,
  parseCargoMetadata,
  type WatchSet,
} from './watch-paths.ts';

/**
 Runs one program call to completion and reports how it ended.
 */
export type WatchRunner = (call: ProgramCall) => Promise<ProgramResult>;

/**
 Read the closure of one manifest from `cargo metadata`.

 @param options - validated push options

 @param run - runs a program call and reports its status and output

 @param manifestPath - repository-relative path of the manifest

 @returns the paths its build depends on

 @throws CrateReleaseError when `cargo metadata` fails, or its answer cannot be read, because a release must not be
 pushed with a dependency closure it could not record

 @example
 ```ts
 await readClosure({ options, run, manifestPath: 'package/a/Cargo.toml' });
 ```
 */
async function readClosure({
  options,
  run,
  manifestPath,
}: {
  readonly options: PushOptions;
  readonly run: WatchRunner;
  readonly manifestPath: string;
}): Promise<WatchSet> {
  /**
   Arguments of the call.
   */
  const args = buildCargoMetadataArguments(manifestPath);
  /**
   How `cargo` ended.
   */
  const result = await run({
    program: cargoProgram,
    args,
    cwd: options.root
  });
  if (result.status !== 0)
    throw new CrateReleaseError(
      `cargo ${args.join(' ')} failed with status ${String(result.status)} in ${options.root}: ${result.stderr
        .trim()}`,
    );
  /**
   The paths the manifest's build depends on, or why they cannot be read.
   */
  const closure = parseCargoMetadata({
    body: result.stdout,
    root: options.root
  });
  if (!closure.found)
    throw new CrateReleaseError(`The cargo metadata of ${manifestPath} cannot be used: ${closure.reason}`);
  return closure.value;
}

/**
 Record every path the release depends on, before anything is sent.

 Those are the files the release commit changes, the directory of the crate, the directories whose lockfile the release
 refreshed, and the dependency closure of each of those crates as `cargo metadata` reports it at the checkout: every
 path dependency directory, and the manifest and lockfile of each workspace root. The closure is read from the
 checkout the release verified, so it is exactly what the check and the dry run built.

 @param options - validated push options

 @param run - runs a program call and reports its status and output

 @param commitPaths - the files the release commit changes

 @returns the watch set

 @throws CrateReleaseError when a closure cannot be read

 @example
 ```ts
 await collectWatchSet({ options, run, commitPaths });
 ```
 */
export async function collectWatchSet({
  options,
  run,
  commitPaths,
}: {
  readonly options: PushOptions;
  readonly run: WatchRunner;
  readonly commitPaths: readonly string[];
}): Promise<WatchSet> {
  /**
   Directories of the crate and of the lockfiles the release refreshed.
   */
  const directories = [options.crateDirectory, ...options.lockDirectories];
  /**
   The closure of each of those crates.
   */
  const closures = await Promise.all(directories.map(async function closureOf(directory) {
    return await readClosure({
      options,
      run,
      manifestPath: posix.join(
        directory,
        manifestFileName
      )
    });
  }));
  return mergeWatchSets([
    {
      files: commitPaths,
      directories
    },
    ...closures,
  ]);
}
