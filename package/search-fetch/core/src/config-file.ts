/**
 Pi Search Fetch config file parsing.
 
 @module
 */

import {
  access,
  readFile,
} from 'node:fs/promises';
import { basename, } from 'node:path';

import { FILE_NOT_FOUND_CODE, } from './config-constants.ts';
import type {
  ConfigJsonReadResult,
  ErrorWithCode,
} from './config-types.ts';

/**
 Read and parse optional config JSON.
 
 @param configPath - absolute config path
 
 @returns parsed JSON result, or absent result when file is absent
 
 @throws when reading fails for reason other than missing file or JSON parsing fails
 
 @example
 ```ts
 await readOptionalConfigJson({ configPath: '/tmp/pi-search-fetch.json' });
 ```
 */
async function readOptionalConfigJson(
  { configPath, }: { readonly configPath: string; },
): Promise<ConfigJsonReadResult> {
  try {
    /**
     Local value for content.
     */
    const content = await readFile(
      configPath,
      'utf8',
    );
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
 Report whether a path exists without exposing read errors as absence.
 
 @param filePath - absolute or relative path to probe
 
 @returns whether the path exists
 
 @throws when access fails for a reason other than missing file
 
 @example
 ```ts
 await fileExists({ filePath: '/tmp/pi-search-fetch.json' });
 ```
 */
async function fileExists({ filePath, }: { readonly filePath: string; }): Promise<boolean> {
  try {
    await access(filePath,);
    return true;
  }
  catch (error: unknown) {
    if (isMissingFileError(error,))
      return false;
    throw error;
  }
}

/**
 Parse config JSON content.
 
 The parse failure message deliberately omits the parser detail: a
 pre-migration Pi config can contain plaintext keys, and Node's JSON parser
 includes an input snippet in its message.
 
 @param content - raw file content
 
 @param configPath - config path used in diagnostics
 
 @returns parsed JSON value
 
 @throws when content is not valid JSON
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
    void error;
    throw new Error(
      `${basename(configPath,)} parsing failed at ${configPath}`,
      { cause: error },
    );
  }
}

/**
 Return whether error is a missing-file filesystem error.
 
 @param error - unknown read error
 
 @returns whether error has ENOENT code
 */
function isMissingFileError(error: unknown,): boolean {
  return isErrorWithCode(error,)
    && (error.code === FILE_NOT_FOUND_CODE);
}

/**
 Return whether error is an Error with a system code.
 
 @param error - unknown error
 
 @returns whether error has a code property
 */
function isErrorWithCode(error: unknown,): error is ErrorWithCode {
  return (Error.isError(error,))
    && ('code' in error);
}

export {
  fileExists,
  readOptionalConfigJson,
};
