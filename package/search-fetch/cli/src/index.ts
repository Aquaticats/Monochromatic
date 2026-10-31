#!/usr/bin/env node
/**
 Entry point of the `search-fetch` command.

 Loads the same global config the pi extension and the MCP server use, wires
 the shared core to one command line, and writes the shared tools' rendered text
 to stdout. Behavior has exactly one implementation in
 \@monochromatic-dev/search-fetch-core; this package adds only the terminal
 grammar and the exit-code mapping recorded in
 `doc/decision/cli-output-and-exit-codes.md`.

 @module
 */

import { realpath, } from 'node:fs/promises';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  createSearchFetchClient,
} from '@monochromatic-dev/search-fetch-core/ts/search-fetch-client.ts';
import {
  loadLinkupConfig,
} from '@monochromatic-dev/search-fetch-core/ts/config.ts';

import {
  programName,
  usageExitCode,
} from './constants.ts';
import { SearchFetchCliConfigurationError, } from './errors.ts';
import {
  runSearchFetchCli,
  type SearchFetchCliRunDependencies,
} from './run.ts';

//region Logger

/**
 Logger root for the search-fetch CLI.
 */
const searchFetchCliLogger = tagged({ tag: programName, },);

/**
 Module logger.
 */
const l = tagged({
  tag: 'index',
  l: searchFetchCliLogger,
},);

//endregion Logger

//region Public re-exports

export {
  parseSearchFetchInvocation,
} from './args.ts';
export type {
  SearchFetchCliInvocation,
} from './args.ts';
export {
  fetchVerb,
  programName,
  searchVerb,
  successExitCode,
  usageExitCode,
} from './constants.ts';
export type {
  SearchFetchCliSettledStatus,
  SearchFetchVerb,
} from './constants.ts';
export {
  SearchFetchCliConfigurationError,
  SearchFetchCliUsageError,
} from './errors.ts';
export {
  validateFetchParams,
  validateSearchParams,
} from './params.ts';
export {
  runSearchFetchCli,
} from './run.ts';
export type {
  SearchFetchCliRunDependencies,
  SearchFetchCliRunOptions,
  SearchFetchCliTextWriter,
} from './run.ts';
export {
  globalUsageText,
  verbUsageText,
} from './usage.ts';
export {
  packageVersion,
} from './version.ts';

//endregion Public re-exports

//region Bin entry: runs only when executed directly

/**
 Whether this module was launched as the executable rather than imported for its exports.

 Compares real paths because bin shims reach the built file through symlinks.
 */
const isBinLaunch = await isDirectlyExecuted();

if (isBinLaunch) {
  /**
   Settled status of this command line; only the non-standard usage status is
   set on `process.exitCode`, and a failed run never returns here at all.
   */
  const status = await runSearchFetchCli({
    argv: process.argv
      .slice(2,),
    stdout: process.stdout,
    stderr: process.stderr,
    loadRunDependencies: loadRealRunDependencies,
  },);
  if (status === usageExitCode)
    process.exitCode = usageExitCode;
}

//endregion Bin entry

//region Helpers

/**
 Load the shared core's config and provider client for one data run.

 @returns config and provider-routing client from the global configuration

 @throws SearchFetchCliConfigurationError when the global configuration cannot be loaded

 @example
 ```ts
 await loadRealRunDependencies();
 ```
 */
async function loadRealRunDependencies(): Promise<SearchFetchCliRunDependencies> {
  try {
    /**
     Runtime config loaded from the same global file the pi extension uses.
     */
    const config = await loadLinkupConfig();
    /**
     Value-free store diagnostic forwarded to missing-key errors.
     */
    const { storeError, } = config.source;
    return {
      config,
      client: createSearchFetchClient({
        ...(config.deepseekApiKey === undefined
          ? {}
          : { deepseekApiKey: config.deepseekApiKey, }),
        ...(config.exaApiKey === undefined
          ? {}
          : { exaApiKey: config.exaApiKey, }),
        ...(config.linkupApiKey === undefined
          ? {}
          : { linkupApiKey: config.linkupApiKey, }),
        ...(storeError === undefined ? {} : { credentialHint: storeError, }),
        blocklist: config.blocklist,
      },),
    };
  }
  catch (error: unknown) {
    throw new SearchFetchCliConfigurationError({
      message: `The ${programName} configuration could not be loaded: ${
        caughtValueText(error,)
      }. Fix the configuration file, then run the command again.`,
      cause: error,
    },);
  }
}

/**
 Report whether this module is the launched executable.

 @returns true when the process was started as the built bin

 @example
 ```ts
 await isDirectlyExecuted();
 ```
 */
async function isDirectlyExecuted(): Promise<boolean> {
  /**
   Path the process was launched through, when launched at all.
   */
  const [, launchedPath,] = process.argv;
  if (launchedPath === undefined)
    return false;

  /**
   Real path of this module in the built output.
   */
  const modulePath = await realpath(import.meta.filename,);
  // Deliberate catch-and-return: a launch path that cannot be resolved means an import,
  // not a bin launch, so resolution failure must never run the command by accident.
  try {
    return (await realpath(launchedPath,)) === modulePath;
  }
  catch (error: unknown) {
    l.debug(`could not resolve launch path ${launchedPath}: ${caughtValueText(error,)}`,);
    return false;
  }
}

//endregion Helpers
