/**
 Pi guardrail config file reading and JSON parsing.
 
 @module
 */

import { readFile, } from 'node:fs/promises';
import { basename, } from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import { FILE_NOT_FOUND_CODE, } from './constants.ts';

//region Types

/**
 Config file read result.
 */
type ConfigReadResult =
  | {
    /**
     Whether config file existed.
     */
    readonly loaded: false;
  }
    | {
    /**
     Whether config file existed.
     */
      readonly loaded: true;
    /**
     Parsed JSON value.
     */
      readonly value: unknown;
    };

/**
 Error shape with optional Node filesystem code.
 */
type ErrorWithCode = Error & {
  /**
   Node filesystem error code.
   */
  readonly code?: string;
};

//endregion Types

//region File parsing

/**
 Default config file reader.
 
 @param path - absolute file path
 
 @returns UTF-8 file contents as a Promise
 
 @example
 ```typescript
 await defaultReadConfigFile('/home/user/.pi/agent/extensions/pi-guardrail.json');
 ```
 */
function defaultReadConfigFile(path: string,): Promise<string> {
  return readFile(
    path,
    'utf8',
  );
}

/**
 Reads optional JSON config from disk.
 
 @param configPath - absolute config path
 
 @param readConfigFile - file reader dependency
 
 @returns parsed config read result
 
 @example
 ```typescript
 await readOptionalConfigJson({ configPath: '/tmp/pi-guardrail.json', readConfigFile });
 ```
 */
async function readOptionalConfigJson(
  {
    configPath,
    readConfigFile,
  }: {
    readonly configPath: string;
    readonly readConfigFile: (path: string) => Promise<string>;
  },
): Promise<ConfigReadResult> {
  try {
    /**
     Raw config file contents.
     */
    const content = await readConfigFile(configPath,);
    return {
      loaded: true,
      value: parseConfigJson({
        content,
        configPath,
      },),
    };
  }
  catch (error: unknown) {
    if (isMissingFileError(error,))
      return { loaded: false, };
    throw error;
  }
}

/**
 Parses config JSON with file-path context in errors.
 
 @param content - raw JSON text
 
 @param configPath - config file path used in diagnostics
 
 @returns parsed JSON value
 
 @example
 ```typescript
 parseConfigJson({ content: '{}', configPath: '/tmp/pi-guardrail.json' });
 ```
 */
function parseConfigJson(
  {
    content,
    configPath,
  }: {
    readonly content: string;
    readonly configPath: string;
  },
): unknown {
  try {
    return JSON.parse(content,) as unknown;
  }
  catch (error: unknown) {
    /**
     Human-readable parse error detail.
     */
    const detail = caughtValueText(error,);
    throw new Error(
      `${basename(configPath,)} parsing failed at ${configPath}: ${detail}`,
      { cause: error, },
    );
  }
}

//endregion File parsing

//region Error helpers

/**
 Returns whether error is a missing-file error.
 
 @param error - caught read error
 
 @returns whether error code is ENOENT
 */
function isMissingFileError(error: unknown,): boolean {
  return isErrorWithCode(error,)
    && (error.code === FILE_NOT_FOUND_CODE);
}

/**
 Returns whether value is an Error with an optional code field.
 
 @param error - caught error
 
 @returns whether error shape can carry a Node code
 */
function isErrorWithCode(error: unknown,): error is ErrorWithCode {
  return Error.isError(error,)
    && ('code' in error);
}

//endregion Error helpers

export {
  defaultReadConfigFile,
  parseConfigJson,
  readOptionalConfigJson,
};
export type { ConfigReadResult, };
