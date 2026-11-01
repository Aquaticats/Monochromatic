/**
 Desktop entry key application logic.
 
 Applies parsed key-value pairs from a `.desktop` file to a mutable result object.
 
 @module
 */

import {
  type DesktopEntry,
  expandEscapes,
} from './desktop-entry-types.ts';

/**
 Returns a copy of `entry` with the field(s) the parsed key sets updated.
 Unrecognized keys yield `entry` unchanged. Returning a fully-shaped
 {@link DesktopEntry} (rather than a `Partial` to merge) keeps the slot exactly
 entry-shaped, avoiding the `Partial` that would reopen
 exactOptionalPropertyTypes holes; the immutable update keeps the param
 deeply readonly. Escape-bearing keys (TryExec and the X-TerminalArg* family)
 run their value through {@link expandEscapes} before storing it.
 
 @param entry - Current parse accumulator; never mutated.
 
 @param key - Desktop entry key name.
 
 @param value - Raw value string.
 
 @returns Updated entry for a recognized key; the same `entry` otherwise.
 
 @example
 ```ts
 const next = applyKey({ entry: createEmptyEntry(), key: 'Exec', value: '/usr/bin/xterm' });
 // next.exec === '/usr/bin/xterm'
 ```
 */
export function applyKey({
  entry,
  key,
  value,
}: {
  readonly entry: DesktopEntry;
  readonly key: string;
  readonly value: string;
},): DesktopEntry {
  if (key === 'Exec')
    return {
      ...entry,
      exec: value,
    };
  if (key === 'Categories') {
    return {
      ...entry,
      isTerminal: value.split(';',)
        .some(function matchTerminal(cat,) {
          return cat === 'TerminalEmulator';
        },),
    };
  }
  if (key === 'Hidden')
    return {
      ...entry,
      hidden: value.toLowerCase()
        === 'true',
    };
  if (key === 'TryExec')
    return {
      ...entry,
      tryExec: expandEscapes({ s: value, },),
    };
  if (key === 'OnlyShowIn') {
    return {
      ...entry,
      onlyShowIn: value.split(';',)
        .filter(function nonEmpty(s,) {
          return s.length
            > 0;
        },),
    };
  }
  if (key === 'NotShowIn') {
    return {
      ...entry,
      notShowIn: value.split(';',)
        .filter(function nonEmpty(s,) {
          return s.length
            > 0;
        },),
    };
  }
  if ((key === 'X-TerminalArgExec') || (key === 'TerminalArgExec'))
    return {
      ...entry,
      execArg: expandEscapes({ s: value, },),
    };
  if ((key === 'X-TerminalArgAppId') || (key === 'TerminalArgAppId'))
    return {
      ...entry,
      appIdArg: expandEscapes({ s: value, },),
    };
  if ((key === 'X-TerminalArgTitle') || (key === 'TerminalArgTitle'))
    return {
      ...entry,
      titleArg: expandEscapes({ s: value, },),
    };
  if ((key === 'X-TerminalArgDir') || (key === 'TerminalArgDir'))
    return {
      ...entry,
      dirArg: expandEscapes({ s: value, },),
    };
  if ((key === 'X-TerminalArgHold') || (key === 'TerminalArgHold'))
    return {
      ...entry,
      holdArg: expandEscapes({ s: value, },),
    };
  return entry;
}
