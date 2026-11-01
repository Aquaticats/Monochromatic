/**
 MCP tool entries over the shared search and fetch core.

 The shared specs in \@monochromatic-dev/search-fetch-core own names,
 descriptions, and valibot parameter schemas; those schemas go to hosts unchanged.
 Truncated results additionally record their full response in the response store and
 carry both a resource URI line and a `resource_link` content block.

 @module
 */

import {
  executeWebFetchTool,
  executeWebSearchTool,
  webFetchToolSpec,
  webSearchToolSpec,
  type LinkupConfig,
  type SearchFetchToolClient,
  type SearchFetchToolResult,
} from '@monochromatic-dev/search-fetch-core/ts';
import {
  defineTool,
  type ToolCallResult,
  type ToolContent,
  type ToolEntry,
} from '@monochromatic-dev/mcp-stdio/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type {
  ResponseStore,
} from './response-store.ts';

//region Logger

/**
 Logger root for the search-fetch MCP server.
 */
const searchFetchMcpLogger = tagged({ tag: 'search-fetch-mcp', },);

/**
 Module logger.
 */
const l = tagged({
  tag: 'tools',
  l: searchFetchMcpLogger,
},);

//endregion Logger

//region Types

/**
 Options for creating the MCP tools over the shared core.
 */
export type CreateSearchFetchToolsOptions = {
  /**
   Provider-routing client used by shared execution.
   */
  readonly client: SearchFetchToolClient;
  /**
   Loaded config carrying the blocklist.
   */
  readonly config: LinkupConfig;
  /**
   Store receiving full responses that truncation hid from the model.
   */
  readonly store: ResponseStore;
};

//endregion Types

//region Public API

/**
 Create the two MCP tool entries over shared execution.

 @param options - client, config, and response store dependencies

 @returns search and fetch tools in registration order

 @example
 ```ts
 createSearchFetchTools({ client, config, store });
 ```
 */
export function createSearchFetchTools(
  options: CreateSearchFetchToolsOptions,
): readonly ToolEntry[] {
  return [
    defineTool({
      name: webSearchToolSpec.name,
      entry: {
        description: webSearchToolSpec.description,
        schema: webSearchToolSpec.parameters,
        handler: async function runSearch(args,) {
          return toToolCallResult({
            result: await executeWebSearchTool({
              client: options.client,
              config: options.config,
              rawParams: args,
            },),
            store: options.store,
          },);
        },
      },
    },),
    defineTool({
      name: webFetchToolSpec.name,
      entry: {
        description: webFetchToolSpec.description,
        schema: webFetchToolSpec.parameters,
        handler: async function runFetch(args,) {
          return toToolCallResult({
            result: await executeWebFetchTool({
              client: options.client,
              config: options.config,
              rawParams: args,
            },),
            store: options.store,
          },);
        },
      },
    },),
  ];
}

//endregion Public API

//region Helpers

/**
 Map a shared tool result onto the MCP result shape.

 Text content passes through untouched, preserving parity with the pi surface.
 When truncation hid part of the response, the full text is recorded in the store and
 appended as a resource URI line plus a `resource_link` block.

 @param result - shared tool result to map

 @param store - store receiving full responses that truncation hid from the model

 @returns MCP tool call result

 @example
 ```ts
 toToolCallResult({ result, store });
 ```
 */
function toToolCallResult(
  {
    result,
    store,
  }: {
    /**
     Shared tool result to map.
     */
    readonly result: SearchFetchToolResult;
    /**
     Store receiving full responses that truncation hid from the model.
     */
    readonly store: ResponseStore;
  },
): ToolCallResult {
  /**
   Model-visible content items copied from the shared result.
   */
  const content: ToolContent[] = [...result.content,];
  /**
   Full response temp path, present only when truncation hid part of the response.
   */
  const { fullJsonPath, } = result.details;
  if (fullJsonPath === undefined)
    return { content, };

  /**
   Recorded full response hosts may fetch by URI.
   */
  const recorded = store.record({
    filePath: fullJsonPath,
    name: 'Full response text',
    description: 'Full response text behind the truncated tool output.',
  },);
  content.push(
    {
      type: 'text',
      text: `[Full response also available as MCP resource: ${recorded.uri}]`,
    },
    {
      type: 'resource_link',
      uri: recorded.uri,
      name: recorded.name,
      description: recorded.description,
    },
  );
  return { content, };
}

//endregion Helpers
