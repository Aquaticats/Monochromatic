/**
 Bash-poke settings loading.

 @module
 */

import { readFile, } from 'node:fs/promises';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { configPathForHome, } from './config-paths.ts';
import { BashPokeConfigError, } from './config-error.ts';
import {
  DEFAULT_SETTINGS,
  parseSettings,
} from './config-schema.ts';
import type { BashPokeSettings, } from './config-types.ts';
import { FILE_NOT_FOUND_CODE, } from './constants.ts';
import { bashPokeLogger, } from './logger.ts';
import { isMissingFileCode, } from './system-error.ts';

//region Types

/**
 Result of reading the settings file.
 
 Absence is a normal state that means "use defaults", while an unreadable file
 is a fault the user must hear about, so the two are distinct outcomes rather
 than one optional value.
 */
type SettingsFileRead =
  | {
    readonly state: 'absent';
  }
    | {
      readonly state: 'present';
      readonly text: string;
    }
    | {
      readonly state: 'unreadable';
      readonly failure: string;
    };

/**
 Result of decoding settings text.
 */
type SettingsJsonParse =
  | {
    readonly state: 'parsed';
    readonly value: unknown;
  }
    | {
      readonly state: 'invalid';
      readonly failure: string;
    };

//endregion Types

//region Reading

/**
 Reads the settings file, distinguishing absence from an unreadable file.
 
 @param path - absolute settings path
 
 @returns which of the three read outcomes applies
 
 @example
 ```ts
 await readSettingsFile({ path: '/home/user/.pi/agent/extensions/pi-bash-poke.json', },);
 ```
 */
async function readSettingsFile(
  { path, }: { readonly path: string; },
): Promise<SettingsFileRead> {
  /**
   Function-scoped logger naming this boundary in every record.
   */
  const l = tagged({
    tag: readSettingsFile.name,
    l: bashPokeLogger,
  }, );
  try {
    /**
     Raw settings text.
     */
    const text = await readFile(
      path,
      'utf8',
    );
    return {
      state: 'present',
      text,
    };
  }
  catch (error: unknown) {
    if (isMissingFileCode({
      error,
      code: FILE_NOT_FOUND_CODE,
    }, )) {
      l.debug(`no settings file at ${path}; using defaults`, );
      return { state: 'absent', };
    }

    /**
     Failure text naming the underlying filesystem fault.
     */
    const failure = caughtValueText(error, );
    l.warn(`settings file at ${path} is unreadable: ${failure}`, );
    return {
      state: 'unreadable',
      failure,
    };
  }
}

/**
 Decodes settings text without throwing at the caller.
 
 @param text - raw settings text
 
 @returns decoded value, or the parse failure text
 
 @example
 ```ts
 parseSettingsJson({ text: '{}', },);
 ```
 */
function parseSettingsJson({ text, }: { readonly text: string; }, ): SettingsJsonParse {
  try {
    return {
      state: 'parsed',
      value: JSON.parse(text, ) as unknown,
    };
  }
  catch (error: unknown) {
    return {
      state: 'invalid',
      failure: caughtValueText(error, ),
    };
  }
}

//endregion Reading

//region Loading

/**
 Loads bash-poke settings, falling back to defaults when no file was ever created.
 
 A present but unreadable or invalid file fails loudly, because silently
 ignoring it would leave the user believing their knobs took effect.
 
 @param home - home directory the `.pi` tree lives below
 
 @returns settings with every key resolved
 
 @throws BashPokeConfigError when the file is unreadable, is not valid JSON, or fails validation
 
 @example
 ```ts
 const settings = await loadSettings({ home: '/home/user', },);
 ```
 */
async function loadSettings(
  { home, }: { readonly home: string; },
): Promise<BashPokeSettings> {
  /**
   Function-scoped logger naming this boundary in every record.
   */
  const l = tagged({
    tag: loadSettings.name,
    l: bashPokeLogger,
  }, );

  /**
   Absolute settings path derived from the injected home.
   */
  const path = configPathForHome({ home, }, );

  /**
   Outcome of reading that path.
   */
  const read = await readSettingsFile({ path, }, );
  if (read.state === 'absent')
    return DEFAULT_SETTINGS;
  if (read.state === 'unreadable')
    throw new BashPokeConfigError(`${path}: unreadable (${read.failure})`, );

  /**
   Outcome of decoding the text that was read.
   */
  const parsed = parseSettingsJson({ text: read.text, }, );
  if (parsed.state === 'invalid')
    throw new BashPokeConfigError(`${path}: invalid JSON (${parsed.failure})`, );

  /**
   Validated settings with every key resolved.
   */
  const settings = parseSettings({
    decoded: parsed.value,
    fileName: path,
  }, );
  l.debug(`loaded settings from ${path}`, );
  return settings;
}

//endregion Loading

export {
  loadSettings,
  parseSettingsJson,
  readSettingsFile,
};

export type {
  SettingsFileRead,
  SettingsJsonParse,
};
