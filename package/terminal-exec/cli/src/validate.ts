/**
 Validates desktop entry candidates for use as terminal emulators.
 Checks Categories, Hidden, TryExec, Exec executability, and OnlyShowIn/NotShowIn.
 
 @module
 */

import {
  access,
  stat,
} from 'node:fs/promises';
import {
  delimiter,
  resolve,
} from 'node:path';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import type { DesktopEntry, } from './desktop-entry.ts';
import {
  INVALID_EXEC,
  tokenizeExec,
} from './tokenize.ts';

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
  tag: 'validate',
  l: parentLogger,
},);

/**
 Sentinel returned by the local {@link which} when an executable is not found
 on `$PATH`. A `unique symbol`; callers narrow with `!== EXECUTABLE_NOT_ON_PATH`.
 */
const EXECUTABLE_NOT_ON_PATH: unique symbol = Symbol('terminal-exec/executable-not-on-path',);

/**
 Sentinel for "no usable terminal", shared across the resolution chain
 ({@link validateEntry} to `tryEntry` to `resolveTerminal`) and re-exported
 by `resolve.ts`. A `unique symbol`; consumers narrow with `=== NO_TERMINAL`.
 */
export const NO_TERMINAL: unique symbol = Symbol('terminal-exec/no usable terminal could be resolved',);

/**
 Checks if an executable exists in `$PATH` using platform-native resolution.
 
 @param name - Executable name or absolute path.
 
 @returns `true` if the executable is found.
 */
async function executableExists({ name, }: { readonly name: string; },): Promise<boolean> {
  if (name.startsWith('/',)) {
    try {
      await stat(name,);
      return true;
    }
    catch (error) {
      if (!(Error.isError(error,)))
        throw error;

      return false;
    }
  }
  return await which(name,) !== EXECUTABLE_NOT_ON_PATH;
}

/**
 Cross-runtime `which` implementation.
 Resolves an executable name by searching directories in `$PATH`.
 
 @param name - Executable name to find.
 
 @returns Absolute path if found, or {@link EXECUTABLE_NOT_ON_PATH}.
 */
async function which(name: string,): Promise<string | typeof EXECUTABLE_NOT_ON_PATH> {
  /**
   Empty PATH fallback yields no candidates, which falls through to null cleanly.
   */
  const pathEnv = process.env
    .PATH
    ?? '';
  /**
   Split on the platform PATH delimiter; colon on POSIX.
   */
  const dirs = pathEnv.split(delimiter,);
  for (const dir of dirs) {
    /**
     Absolute path candidate fed to access() inside the loop.
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
 Result of validating a desktop entry for terminal emulator use.
 */
export type ValidatedEntry = {
  /**
   Tokenized Exec command as an argument array.
   */
  readonly execTokens: readonly string[];
  /**
   Resolved TerminalArgExec value (from entry, default, or `-e`).
   */
  readonly execArg: string;
  /**
   TerminalArgAppId value.
   */
  readonly appIdArg: string;
  /**
   TerminalArgTitle value.
   */
  readonly titleArg: string;
  /**
   TerminalArgDir value.
   */
  readonly dirArg: string;
  /**
   TerminalArgHold value.
   */
  readonly holdArg: string;
};

/**
 Validates a parsed desktop entry as a usable terminal emulator. The Exec
 line is tokenized with {@link tokenizeExec}; {@link INVALID_EXEC} or an
 empty result disqualifies the entry.
 
 @param entry - Parsed desktop entry.
 
 @param entryId - Desktop entry ID for logging and default lookups.
 
 @param desktops - Current desktop names for OnlyShowIn/NotShowIn checks.
 
 @param isFallback - Whether this is a fallback entry (enables ShowIn checks).
 
 @param execArgDefault - Default TerminalArgExec from config, if any.
 
 @returns Validated entry with tokenized Exec, or {@link NO_TERMINAL} if the entry fails validation.
 
 @example
 ```ts
 const validated = validateEntry({
   entry, entryId: 'org.xterm', desktops: ['kde'],
   isFallback: false, execArgDefault: '-e',
 });
 ```
 */
export async function validateEntry({
  entry,
  entryId,
  desktops,
  isFallback,
  execArgDefault,
}: {
  readonly entry: DesktopEntry;
  readonly entryId: string;
  readonly desktops: readonly string[];
  readonly isFallback: boolean;
  readonly execArgDefault: string;
},): Promise<ValidatedEntry | typeof NO_TERMINAL> {
  if (!entry.isTerminal) {
    l.debug(`${entryId}: not a TerminalEmulator`,);
    return NO_TERMINAL;
  }

  if (entry.hidden) {
    l.debug(`${entryId}: hidden`,);
    return NO_TERMINAL;
  }

  //region OnlyShowIn / NotShowIn (fallback entries only)
  if (isFallback) {
    if (entry.onlyShowIn
      .length
      > 0) {
      /**
       Whether any OnlyShowIn entry matches the current desktops list.
       */
      const shown = entry.onlyShowIn
        .some(function matchDesktop(d,) {
          return desktops.includes(d.toLowerCase(),);
        },);
      if (!shown) {
        l.debug(`${entryId}: OnlyShowIn does not match current desktops`,);
        return NO_TERMINAL;
      }
    }
    if (entry.notShowIn
      .length
      > 0) {
      /**
       Whether any NotShowIn entry matches the current desktops list.
       */
      const hidden = entry.notShowIn
        .some(function matchDesktop(d,) {
          return desktops.includes(d.toLowerCase(),);
        },);
      if (hidden) {
        l.debug(`${entryId}: NotShowIn matches current desktop`,);
        return NO_TERMINAL;
      }
    }
  }
  //endregion

  if ((entry.tryExec
    .length
    > 0)
    && (!await executableExists({ name: entry.tryExec, },)))
    {
    l.debug(`${entryId}: TryExec '${entry.tryExec}' not found`,);
    return NO_TERMINAL;
  }

  /**
   Argv form of the Exec line; INVALID_EXEC or empty disqualifies the entry.
   */
  const execTokens = tokenizeExec({ exec: entry.exec, },);
  if ((execTokens === INVALID_EXEC) || (execTokens.length
    === 0)) {
    l.debug(`${entryId}: Exec tokenization failed or empty`,);
    return NO_TERMINAL;
  }

  /**
   Executable token, the only one we PATH-check before validating.
   */
  const [firstToken,] = execTokens;
  if (firstToken === undefined)
    throw new Error('unreachable: length checked above',);
  if (!await executableExists({ name: firstToken, },)) {
    l.debug(`${entryId}: Exec[0] '${firstToken}' not found in PATH`,);
    return NO_TERMINAL;
  }

  /**
   Resolve TerminalArgExec: entry value \> config default \> `-e`.
   */
  const resolvedExecArg = entry.execArg
    .length
    > 0
    ? entry.execArg
    : (execArgDefault.length
      > 0
      ? execArgDefault
      : '-e');

  l.debug(`${entryId}: validated, execArg='${resolvedExecArg}'`,);

  return {
    execTokens,
    execArg: resolvedExecArg,
    appIdArg: entry.appIdArg,
    titleArg: entry.titleArg,
    dirArg: entry.dirArg,
    holdArg: entry.holdArg,
  };
}
