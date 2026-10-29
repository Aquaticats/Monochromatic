/**
 Resolves the default terminal emulator on Windows.
 Checks for Windows Terminal (`wt.exe`) first, then falls back to `cmd.exe`.
 
 @module
 */

import {
  delimiter,
  resolve,
} from 'node:path';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import type { ResolvedTerminal, } from './resolve.ts';

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
  tag: 'windows',
  l: parentLogger,
},);

/**
 Sentinel returned by the local {@link which} when an executable is not found
 on `$PATH`. A `unique symbol`; callers narrow with `!== EXECUTABLE_NOT_ON_PATH`.
 */
const EXECUTABLE_NOT_ON_PATH: unique symbol = Symbol('terminal-exec/executable-not-on-path',);

/**
 Cross-runtime `which` for Windows.
 Resolves an executable by searching directories in `$PATH`.
 
 @param name - Executable name to find.
 
 @returns Absolute path if found, or {@link EXECUTABLE_NOT_ON_PATH}.
 */
async function which(name: string,): Promise<string | typeof EXECUTABLE_NOT_ON_PATH> {
  /**
   Dynamic import keeps the Windows-only path cold on other platforms.
   */
  const { access, } = await import('node:fs/promises');
  /**
   Empty PATH fallback yields an empty dirs list, which returns null cleanly.
   */
  const pathEnv = process.env
    .PATH
    ?? '';
  /**
   Per-platform PATH delimiter; semicolon on Windows.
   */
  const dirs = pathEnv.split(delimiter,);
  for (const dir of dirs) {
    /**
     Absolute path candidate to access-check inside the loop.
     */
    const candidate = resolve(
      dir,
      name,
    );
    try {
      /* oxlint-disable-next-line no-await-in-loop -- sequential PATH walk must check one dir at a time */
      await access(candidate,);
      return candidate;
    }
    catch (error) {
      if (!(Error.isError(error,)))
        throw error;

      continue;
    }
  }
  return EXECUTABLE_NOT_ON_PATH;
}

/**
 Resolves the terminal emulator on Windows.
 
 Resolution order, each checked with the local {@link which}:
 1. Windows Terminal (`wt.exe`): default on Windows 11+, widely installed on Windows 10
 2. `cmd.exe`: always available
 
 @returns Resolved terminal entry.
 
 @example
 ```ts
 const terminal = await resolveWindowsTerminal()
 // terminal.execTokens === ['wt.exe'] or ['cmd.exe']
 ```
 */
export async function resolveWindowsTerminal(): Promise<ResolvedTerminal> {
  if (await which('wt.exe',) !== EXECUTABLE_NOT_ON_PATH) {
    l.debug('found Windows Terminal (wt.exe)',);
    return {
      entryId: 'wt.exe',
      execTokens: ['wt.exe',],
      execArg: '--',
      appIdArg: '',
      titleArg: '--title',
      dirArg: '--startingDirectory',
      holdArg: '',
    };
  }

  l.debug('Windows Terminal not found, falling back to cmd.exe',);
  return {
    entryId: 'cmd.exe',
    execTokens: ['cmd.exe',],
    execArg: '/c',
    appIdArg: '',
    titleArg: '',
    dirArg: '',
    holdArg: '/k',
  };
}
