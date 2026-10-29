/**
 Reads KDE's `kdeglobals` to extract the `TerminalService` desktop entry ID.
 Used as a fallback when no `xdg-terminals.list` config specifies a terminal.
 
 @module
 */

import { readFile, } from 'node:fs/promises';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

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
  tag: 'kde',
  l: parentLogger,
},);

/**
 Sentinel returned by {@link readFileOrAbsent} when a file is missing or
 unreadable. A `unique symbol`; {@link kdeTerminalService} narrows with
 `=== KDEGLOBALS_UNREADABLE`.
 */
const KDEGLOBALS_UNREADABLE: unique symbol = Symbol('terminal-exec/kdeglobals file cannot be read',);

/**
 Sentinel returned by {@link kdeTerminalService} when no `TerminalService` is
 configured (file missing, or key absent). A `unique symbol`; the resolver
 narrows with `=== NO_KDE_TERMINAL`.
 */
export const NO_KDE_TERMINAL: unique symbol = Symbol('terminal-exec/no-kde-terminal',);

/**
 Reads a UTF-8 text file, returning {@link KDEGLOBALS_UNREADABLE} when the read fails.
 Used to fold the kdeglobals-not-found branch into a single sentinel check at the call site.
 
 @param path - Absolute filesystem path to read.
 
 @returns File contents as a UTF-8 string, or {@link KDEGLOBALS_UNREADABLE} when the file is missing or unreadable.
 
 @example
 ```ts
 const text = await readFileOrAbsent({ path: '/home/alice/.config/kdeglobals' })
 // text === '[General]\nTerminalService=...\n' when present, KDEGLOBALS_UNREADABLE otherwise
 ```
 */
async function readFileOrAbsent(
  { path, }: { readonly path: string; },
): Promise<string | typeof KDEGLOBALS_UNREADABLE> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    return KDEGLOBALS_UNREADABLE;
  }
}

/**
 Reads `TerminalService` from `~/.config/kdeglobals`.
 This is the desktop entry filename that KDE System Settings writes
 when the user selects a default terminal emulator.
 
 @returns Desktop entry ID (e.g. `com.mitchellh.ghostty.desktop`), or {@link NO_KDE_TERMINAL} if not configured.
 
 @example
 ```ts
 const id = await kdeTerminalService()
 // 'com.mitchellh.ghostty.desktop'
 ```
 */
export async function kdeTerminalService(): Promise<string | typeof NO_KDE_TERMINAL> {
  /**
   HOME envar fallback keeps path construction deterministic on systems where HOME is unset.
   */
  const home = process.env
    .HOME
    ?? '/tmp';
  /**
   XDG config base; defaults under HOME per the spec.
   */
  const configHome = process.env
    .XDG_CONFIG_HOME
    ?? `${home}/config`;
  /**
   KDE's global settings file; source of the TerminalService key.
   */
  const path = `${configHome}/kdeglobals`;

  /**
   KDEGLOBALS_UNREADABLE when kdeglobals is missing or unreadable; the catch path is the only failure mode.
   */
  const text = await readFileOrAbsent({ path, },);
  if (text === KDEGLOBALS_UNREADABLE) {
    l.debug('kdeglobals not found',);
    return NO_KDE_TERMINAL;
  }

  for (const rawLine of text.split('\n',)) {
    /**
     Whitespace tolerance before the prefix check.
     */
    const line = rawLine.trim();
    if (line.startsWith('TerminalService=',)) {
      /**
       Payload after the key, trimmed of stray whitespace before the empty check.
       */
      const value = line.slice('TerminalService='.length,)
        .trim();
      if (value.length
        > 0) {
        l.debug(`kdeglobals TerminalService='${value}'`,);
        return value;
      }
    }
  }

  l.debug('no TerminalService in kdeglobals',);
  return NO_KDE_TERMINAL;
}
