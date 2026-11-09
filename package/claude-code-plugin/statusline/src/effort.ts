/**
 Claude effort-level indicator formatting.
 
 @module
 */

import { readFile, } from 'node:fs/promises';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts/tagged.ts';

import {
  diagnosticLogger,
  reportUnreadable,
} from './diagnostic-logger.ts';

//region Constants

/**
 Effort level symbols matching Claude Code's built-in indicators.
 */
const EFFORT_SYMBOLS: Record<string, string> = {
  low: '\u25CB',
  medium: '\u25D0',
  max: '\u25C9',
};

/**
 Default effort level, rendered as an empty indicator.
 */
const DEFAULT_EFFORT_LEVEL = 'high';

//endregion Constants

//region Settings parser

/**
 Reads `effortLevel` from settings JSON text.
 
 @param raw - raw settings JSON text
 
 @returns effort level string, or default level when unreadable
 
 @example
 ```ts
 effortLevelFromSettings('{"effortLevel":"low"}');
 ```
 */
function effortLevelFromSettings(raw: string,): string {
  try {
    /**
     Parsed settings JSON value.
     */
    const parsed: unknown = JSON.parse(raw,);
    /**
     Whether JSON-owned settings can carry an effort-level property.
     */
    const isReadableObject = ((typeof parsed) === 'object')
      && (parsed !== null)
      && (!Array.isArray(parsed,));
    if (!isReadableObject)
      return DEFAULT_EFFORT_LEVEL;

    /**
     Optional effort level read from JSON-owned data with no caller-owned hooks.
     */
    const level: unknown = Reflect.get(
      parsed,
      'effortLevel',
    );
    return ((typeof level) === 'string') && (level.length > 0)
      ? level
      : DEFAULT_EFFORT_LEVEL;
  }
  catch (error: unknown) {
    // The settings file was read, so unparsable text is durable corruption of a
    // user file rather than an absent one, and it silently reverts the effort
    // indicator to the default on every render until it is repaired.
    tagged({
      tag: effortLevelFromSettings.name,
      l: diagnosticLogger,
    },)
      .warn(`settings JSON is unparsable, so the effort level stays unknown: ${caughtValueText(error,)}`,);
    return DEFAULT_EFFORT_LEVEL;
  }
}

//endregion Settings parser

//region Public reader

/**
 Reads `effortLevel` from `~/.claude/settings.json`.
 
 @returns effort indicator symbol, or empty string for high/default/unreadable settings
 
 @example
 ```ts
 await readEffortIndicator();
 ```
 */
async function readEffortIndicator(): Promise<string> {
  /**
   User home directory; empty path simply fails to resolve settings.
   */
  const { HOME: home = '', } = process.env;
  /**
   Path to the global Claude Code settings file storing `effortLevel`.
   */
  const settingsPath = `${home}/.claude/settings.json`;
  try {
    /**
     Raw JSON read from disk.
     */
    const raw = await readFile(
      settingsPath,
      'utf8',
    );
    /**
     Resolved effort level.
     */
    const level = effortLevelFromSettings(raw,);
    return EFFORT_SYMBOLS[level] ?? '';
  }
  catch (error: unknown) {
    // A home without a settings file is ordinary, and so is this line: the
    // reason still names the file the level was read from.
    reportUnreadable({
      error,
      l: tagged({
        tag: readEffortIndicator.name,
        l: diagnosticLogger,
      },),
      subject: `effort level unreadable from ${settingsPath}`,
    },);
    return '';
  }
}

//endregion Public reader

export {
  EFFORT_SYMBOLS,
  effortLevelFromSettings,
  readEffortIndicator,
};
