/**
 Resolution of the `sops` executable on a curated `PATH`.

 @module
 */

import {
  constants as fsConstants,
} from 'node:fs';
import {
  access,
  stat,
} from 'node:fs/promises';
import {
  delimiter,
  join,
} from 'node:path';

import {
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { isNonBlank, } from './environment.ts';
import { EnvLocalJsonSopsMissingError, } from './errors.ts';
import type { EnvLocalJsonEnvironment, } from './types.ts';

//region Constants

/**
 Logger for executable lookup decisions; names candidate paths only.
 */
const executableLogger = tagged({ tag: 'envLocalJsonSopsExecutable', },);

/**
 Executable name looked up on the curated `PATH`.
 */
const SOPS_EXECUTABLE_NAME = 'sops';

/**
 Windows executable extension fallback when `PATHEXT` is absent.
 */
const DEFAULT_WINDOWS_EXTENSIONS: readonly string[] = [
  '.EXE',
  '.CMD',
  '.BAT',
  '.COM',
];

//endregion Constants

//region Lookup

/**
 Whether a candidate path is an executable regular file.

 @param path - candidate executable path

 @returns `true` when the file exists and is executable

 @example
 ```ts
 await isExecutableFile('/usr/bin/sops');
 ```
 */
async function isExecutableFile({ path, }: { readonly path: string; },): Promise<boolean> {
  try {
    /**
     Metadata of the candidate executable.
     */
    const metadata = await stat(path,);
    if (!metadata.isFile())
      return false;
    await access(
      path,
      fsConstants.X_OK,
    );
    return true;
  }
  catch (error) {
    executableLogger.trace(
      `candidate ${path} is not executable: ${caughtValueText(error,)}`,
    );
    return false;
  }
}

/**
 Resolves the `sops` executable on the curated `PATH` only.

 @param env - curated or parent environment whose `PATH` is searched

 @returns absolute or PATH-relative executable path

 @throws EnvLocalJsonSopsMissingError when no executable candidate exists

 @example
 ```ts
 const sopsPath = await resolveSopsExecutable({ env: process.env });
 ```
 */
export async function resolveSopsExecutable({
  env,
}: {
  readonly env: EnvLocalJsonEnvironment;
},): Promise<string> {
  /**
   Parent `PATH` value used for the lookup.
   */
  const pathValue = env.PATH ?? env.Path;
  if (!isNonBlank(pathValue,))
    throw new EnvLocalJsonSopsMissingError();

  /**
   Extensions tried after the executable name, empty on POSIX.
   */
  const extensions = process.platform === 'win32'
    ? (env.PATHEXT ?? '').split(';',)
      .filter(function hasExtension(value,): boolean {
        return value !== '';
      },)
    : ['',];
  /**
   Final extension list; Windows always includes the conventional defaults.
   */
  const suffixes = (process.platform === 'win32') && (extensions.length === 0)
    ? [...DEFAULT_WINDOWS_EXTENSIONS,]
    : extensions;

  /**
   Every candidate executable path, in `PATH` order.
   */
  const candidates = pathValue.split(delimiter,)
    .flatMap(function candidatesInDirectory(directory: string,): readonly string[] {
      /**
       Directory part of this `PATH` entry; an empty entry means the process
       working directory.
       */
      const base = directory === '' ? '.' : directory;
      return suffixes.map(function candidateWithSuffix(suffix: string,): string {
        return join(
          base,
          `${SOPS_EXECUTABLE_NAME}${suffix}`,
        );
      },);
    },);
  /**
   Executability of every candidate, in the same order.
   */
  const executable = await Promise.all(candidates.map(function checkCandidate(
    candidate: string,
  ): Promise<boolean> {
    return isExecutableFile({ path: candidate, },);
  },),);
  /**
   Position of the first executable candidate, or -1.
   */
  const position = executable.indexOf(true,);
  /**
   First executable candidate.
   */
  const found = candidates[position];
  if (found === undefined)
    throw new EnvLocalJsonSopsMissingError();
  return found;
}

//endregion Lookup
