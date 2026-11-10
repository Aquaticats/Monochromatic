/**
 One `$PATH` lookup, shared by the Linux and the Windows terminal resolver.

 The two resolvers differed only in how they import `access`: the Linux path
 imports it beside `stat`, and the Windows path imports it lazily so that
 Windows-only branch stays cold elsewhere. The rest is one rule, the delimiter
 split of the search path, the `resolve` of each candidate, the sequential
 `access` walk, the `caughtValueText` log of a rejected candidate and the
 {@link EXECUTABLE_NOT_ON_PATH} sentinel, and it lives here once so that a
 change to the walk reaches both platforms.

 @module
 */

import {
  delimiter,
  resolve,
} from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Logger root for terminal-exec after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'terminal-exec', },);

/**
 Tagged logger for this module.
 */
const l = tagged({
  tag: 'which',
  l: parentLogger,
},);

/**
 Sentinel returned by {@link which} when no directory of the search path holds
 a usable executable. A `unique symbol`; callers narrow with
 `!== EXECUTABLE_NOT_ON_PATH`.

 @example
 ```ts
 const found = await which({ name: 'bash', access, });
 if (found !== EXECUTABLE_NOT_ON_PATH) launchTerminal({ execTokens: [found] });
 ```
 */
export const EXECUTABLE_NOT_ON_PATH: unique symbol = Symbol('terminal-exec/executable-not-on-path',);

/**
 Probes one candidate path for a usable executable.

 Resolving means answering whether the path is executable, so a rejection
 carrying an `Error` reports that this candidate is not, and the caller walks on.

 @param candidate - Absolute path built from one search-path directory and the
 executable name.

 @returns nothing when the candidate is usable

 @throws Error when the candidate cannot be executed

 @example
 ```ts
 const access = async function access(candidate: string): Promise<void> {
   await accessSync(candidate, constants.X_OK);
 };
 ```
 */
export type ExecutableAccess = (candidate: string,) => Promise<void>;

/**
 Resolves an executable name by walking the directories of a search path.

 The first candidate the probe accepts answers the walk, in search-path order,
 so a later directory never shadows an earlier one.

 @param name - Executable name to find.

 @param access - Probe awaited once per candidate, in search-path order.

 @param pathEnv - Search path to walk, split on the platform delimiter; reads
 this process's `PATH` when the caller injects none.

 @returns Absolute path of the first usable candidate, or
 {@link EXECUTABLE_NOT_ON_PATH} when none is.

 @throws The rejected value of `access`, unchanged, when it is not an `Error`,
 because only an `Error` reports a candidate that is missing or not executable.

 @example
 ```ts
 const found = await which({ name: 'wt.exe', access, });
 ```
 */
export async function which(
  {
    name,
    access,
    pathEnv = process.env
      .PATH
      ?? '',
  }: {
    readonly name: string;
    readonly access: ExecutableAccess;
    readonly pathEnv?: string;
  },
): Promise<string | typeof EXECUTABLE_NOT_ON_PATH> {
  /**
   Directories the search path names, in the order that decides the answer.
   */
  const dirs = pathEnv.split(delimiter,);
  for (const dir of dirs) {
    /**
     Candidate this directory contributes.
     */
    const candidate = resolve(
      dir,
      name,
    );
    try {
      /* oxlint-disable-next-line no-await-in-loop -- the walk must check one directory at a time to keep the search-path order */
      await access(candidate,);
      return candidate;
    }
    catch (error) {
      if (!(Error.isError(error,)))
        throw error;

      l.debug(`PATH candidate '${candidate}' cannot be used: ${caughtValueText(error,)}`,);
      continue;
    }
  }
  return EXECUTABLE_NOT_ON_PATH;
}
