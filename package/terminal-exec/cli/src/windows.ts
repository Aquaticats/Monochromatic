/**
 Resolves the default terminal emulator on Windows.
 Checks for Windows Terminal (`wt.exe`) first, then falls back to `cmd.exe`.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { ResolvedTerminal, } from './resolve.ts';
import {
  EXECUTABLE_NOT_ON_PATH,
  which,
} from './which.ts';

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
 Resolves the terminal emulator on Windows.
 
 Resolution order, each checked with the shared {@link which}:
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
  /**
   `access` imported lazily keeps the Windows-only probe cold on other platforms.
   */
  const { access, } = await import('node:fs/promises');
  if (await which({
    name: 'wt.exe',
    access,
  },) !== EXECUTABLE_NOT_ON_PATH) {
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
