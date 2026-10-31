/**
 Run orchestration for the search-fetch CLI.

 One settled-status contract: help, version, and successful data runs return
 the standard success status; usage, configuration, and blocklist refusals
 return the non-standard usage status with usage text on stderr; every provider
 or deadline failure is rethrown, so a failed run exits 1 through a thrown error
 and leaves stdout empty.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  BlockedUrlError,
  executeWebFetchTool,
  executeWebSearchTool,
  type SearchFetchToolClient,
} from '@monochromatic-dev/search-fetch-core/ts/tool-exec.ts';
import type {
  LinkupConfig,
} from '@monochromatic-dev/search-fetch-core/ts/config.ts';

import {
  parseSearchFetchInvocation,
} from './args.ts';
import {
  programName,
  searchVerb,
  successExitCode,
  usageExitCode,
  type SearchFetchCliSettledStatus,
} from './constants.ts';
import {
  SearchFetchCliConfigurationError,
  SearchFetchCliUsageError,
} from './errors.ts';
import {
  validateFetchParams,
  validateSearchParams,
} from './params.ts';
import {
  globalUsageText,
  verbUsageText,
} from './usage.ts';
import { packageVersion, } from './version.ts';

//region Logger

/**
 Logger root for the search-fetch CLI.
 */
const searchFetchCliLogger = tagged({ tag: programName, },);

//endregion Logger

//region Types

/**
 Destination for terminal output: result text and usage text.
 */
export type SearchFetchCliTextWriter = {
  /**
   Write one chunk of terminal text.
   */
  readonly write: (text: string,) => void;
};

/**
 Runtime dependencies one data run needs from the shared core.
 */
export type SearchFetchCliRunDependencies = {
  /**
   Provider-routing client used by shared execution.
   */
  readonly client: SearchFetchToolClient;
  /**
   Loaded config carrying the blocklist.
   */
  readonly config: LinkupConfig;
};

/**
 Options driving one CLI run against injected streams and dependencies.
 */
export type SearchFetchCliRunOptions = {
  /**
   Arguments after the script path.
   */
  readonly argv: readonly string[];
  /**
   Stream receiving result text and explicit `--help` or `--version` output.
   */
  readonly stdout: SearchFetchCliTextWriter;
  /**
   Stream receiving usage text and diagnostics.
   */
  readonly stderr: SearchFetchCliTextWriter;
  /**
   Loads config and provider client only when a data run needs them.
   */
  readonly loadRunDependencies: () => Promise<SearchFetchCliRunDependencies>;
};

/**
 Model-visible tool result projection this CLI renders: its content items, deeply readonly.
 */
type RenderableToolResult = {
  /**
   Model-visible content items in result order.
   */
  readonly content: readonly {
    /**
     Content item text.
     */
    readonly text: string;
  }[];
};

//endregion Types

//region Public API

/**
 Run one search-fetch command line to its settled status.

 @param options - arguments, streams, and dependency loading

 @returns success status, or the usage status every caller or configuration refusal maps to

 @throws whatever the shared core throws on a provider failure or deadline expiry, because a failed run exits 1 through a thrown error

 @example
 ```ts
 await runSearchFetchCli({ argv: ['search', 'docs'], stdout, stderr, loadRunDependencies });
 ```
 */
export async function runSearchFetchCli(
  options: SearchFetchCliRunOptions,
): Promise<SearchFetchCliSettledStatus> {
  /**
   Logger tagged for this run.
   */
  const innerL = tagged({
    tag: runSearchFetchCli.name,
    l: searchFetchCliLogger,
  },);
  try {
    /**
     Parsed command line.
     */
    const invocation = parseSearchFetchInvocation({ argv: options.argv, },);
    if (invocation.kind === 'help') {
      options
        .stdout
        .write(`${
          invocation.verb === undefined
            ? globalUsageText()
            : verbUsageText(invocation.verb,)
        }\n`,);
      return successExitCode;
    }
    if (invocation.kind === 'version') {
      options
        .stdout
        .write(`${await packageVersion()}\n`,);
      return successExitCode;
    }

    innerL.debug(`running ${programName} ${invocation.verb}`,);
    /**
     Runtime dependencies for the data run.
     */
    const {
      client,
      config,
    } = await options.loadRunDependencies();
    if (invocation.verb === searchVerb) {
      validateSearchParams({ rawParams: invocation.rawParams, },);
      options
        .stdout
        .write(`${
          renderedToolText(await executeWebSearchTool({
            client,
            config,
            rawParams: invocation.rawParams,
          },))
        }\n`,);
      return successExitCode;
    }
    validateFetchParams({ rawParams: invocation.rawParams, },);
    options
      .stdout
      .write(`${
        renderedToolText(await executeWebFetchTool({
          client,
          config,
          rawParams: invocation.rawParams,
        },))
      }\n`,);
    return successExitCode;
  }
  catch (error: unknown) {
    if (error instanceof SearchFetchCliUsageError) {
      innerL.error(caughtValueText(error,),);
      options
        .stderr
        .write(`${globalUsageText()}\n`,);
      return usageExitCode;
    }
    if ((error instanceof SearchFetchCliConfigurationError)
      || (error instanceof BlockedUrlError)) {
      innerL.error(caughtValueText(error,),);
      return usageExitCode;
    }
    throw error;
  }
}

//endregion Public API

//region Helpers

/**
 Render the shared tool result into the text a data run writes to stdout.

 @param result - shared tool result carrying model-visible content items

 @returns rendered tool text: response text, preceded by any tool warning item

 @example
 ```ts
 renderedToolText({ content: [{ text: 'page' }] });
 ```
 */
function renderedToolText(result: RenderableToolResult,): string {
  return result
    .content
    .map(function contentText(item,) {
      return item.text;
    },)
    .join('\n',);
}

//endregion Helpers
