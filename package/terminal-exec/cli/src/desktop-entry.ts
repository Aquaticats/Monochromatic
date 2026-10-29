/**
 Parses `.desktop` files following the freedesktop Desktop Entry Specification.
 Extracts terminal-relevant keys: Exec, Categories, TryExec, Hidden,
 OnlyShowIn, NotShowIn, and X-TerminalArg* fields.
 
 @module
 */

import { readFile, } from 'node:fs/promises';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { applyKey, } from './desktop-entry-apply.ts';
import {
  createEmptyEntry,
  type DesktopEntry,
} from './desktop-entry-types.ts';

/**
 Logger root for terminal-exec after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'terminal-exec', },);

export type { DesktopEntry, } from './desktop-entry-types.ts';
export { expandEscapes, } from './desktop-entry-types.ts';

/**
 Tagged logger for this module.
 */
const l = tagged({
  tag: 'desktop-entry',
  l: parentLogger,
},);

/**
 Sentinel returned by {@link parseDesktopEntry} when the `.desktop` file
 cannot be read. A `unique symbol`; the resolver narrows with
 `=== DESKTOP_ENTRY_UNREADABLE` and skips the entry.
 */
export const DESKTOP_ENTRY_UNREADABLE: unique symbol = Symbol('terminal-exec/desktop entry file cannot be read',);

/**
 Parses a `.desktop` file and extracts terminal-relevant keys.
 Only reads keys from the `[Desktop Entry]` section (not actions).
 
 @param path - Absolute path to the `.desktop` file.
 
 @returns Parsed desktop entry, or {@link DESKTOP_ENTRY_UNREADABLE} if the file cannot be read.
 
 @example
 ```ts
 const entry = await parseDesktopEntry({ path: '/usr/share/applications/com.mitchellh.ghostty.desktop' })
 // entry.exec === '/usr/bin/ghostty --gtk-single-instance=true'
 // entry.isTerminal === true
 ```
 */
export async function parseDesktopEntry(
  { path, }: { readonly path: string; },
): Promise<DesktopEntry | typeof DESKTOP_ENTRY_UNREADABLE> {
  /**
   Empty default lets the catch return DESKTOP_ENTRY_UNREADABLE without restructuring the read path.
   */
  let text = '';
  try {
    text = await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    return DESKTOP_ENTRY_UNREADABLE;
  }

  /**
   Parse accumulator; each applyKey call returns an updated copy. Annotated readonly so the immutable updates type-check.
   */
  let result: DesktopEntry = createEmptyEntry();

  /**
   Section gate; toggled on each `[Section]` line so non-Desktop Entry sections are skipped.
   */
  let inDesktopEntry = false;

  for (const rawLine of text.split('\n',)) {
    /**
     Whitespace tolerance before prefix checks.
     */
    const line = rawLine.trim();

    if (line.startsWith('[',)) {
      inDesktopEntry = line === '[Desktop Entry]';
      if ((!inDesktopEntry) && (result.exec
        .length
        > 0))
        break;
      continue;
    }

    if (!inDesktopEntry)
      continue;

    /**
     Separator index; -1 means a non-key line that must be skipped.
     */
    const eqIdx = line.indexOf('=',);
    if (eqIdx === (-1))
      continue;

    /**
     Normalized key name passed to applyKey for dispatch.
     */
    const key = line
      .slice(
      0,
      eqIdx,
      )
      .trim();
    /**
     Payload after `=`, trimmed before applyKey stores it.
     */
    const value = line.slice(eqIdx + 1,)
      .trim();

    result = applyKey({
      entry: result,
      key,
      value,
    },);
  }

  l.debug(
    `parsed '${path}': exec='${result.exec}', isTerminal=${String(result.isTerminal,)}`,
  );
  return result;
}
