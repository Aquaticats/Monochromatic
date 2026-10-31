/**
 Pi tool adapters for the shared search and fetch core.

 The shared specs in \@monochromatic-dev/search-fetch-core own names,
 descriptions, and valibot parameter schemas; this module converts those schemas to the
 TypeBox form pi compiles and forwards execution to the shared executors.

 @module
 */

import {
  defineTool,
  type AgentToolUpdateCallback,
  type ExtensionContext,
  type ToolDefinition,
} from '@earendil-works/pi-coding-agent';
import {
  LINKUP_WEB_FETCH_TOOL_NAME,
  LINKUP_WEB_SEARCH_TOOL_NAME,
  executeWebFetchTool,
  executeWebSearchTool,
  valibotToTypeBox,
  webFetchToolSpec,
  webSearchToolSpec,
  type LinkupConfig,
  type LinkupToolDetails,
  type SearchFetchToolClient,
} from '@monochromatic-dev/search-fetch-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { TSchema, } from 'typebox';

//region Logger

/**
 Logger root for pi-search-fetch after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: linkupLogger, },);
 ```
 */
const linkupLogger = tagged({ tag: 'pi-search-fetch', },);

/**
 Module logger.
 */
const l = tagged({
  tag: 'tools',
  l: linkupLogger,
},);

//endregion Logger

//region Types

/**
 Options for creating Pi tools over the shared core.
 */
export type CreateLinkupToolsOptions = {
  /**
   Loaded extension config carrying the blocklist.
   */
  readonly config: LinkupConfig;
  /**
   Provider-routing client used by shared execution.
   */
  readonly client: SearchFetchToolClient;
};

/**
 Pi tool definition type for the shared search and fetch contracts.
 */
export type LinkupToolDefinition = ToolDefinition<TSchema, LinkupToolDetails>;

//endregion Types

//region Public API

/**
 Create the two Pi tool definitions over shared execution.

 @param options - config and client dependencies

 @returns search and fetch tools in registration order

 @example
 ```ts
 createLinkupTools({ config, client });
 ```
 */
export function createLinkupTools(options: CreateLinkupToolsOptions,): readonly LinkupToolDefinition[] {
  return [
    createLinkupWebSearchTool(options,),
    createLinkupWebFetchTool(options,),
  ];
}

//endregion Public API

//region Helpers

/**
 Create the Pi search tool over the shared search contract.

 @param options - config and client dependencies

 @returns Pi tool definition

 @example
 ```ts
 createLinkupWebSearchTool({ config, client });
 ```
 */
function createLinkupWebSearchTool(options: CreateLinkupToolsOptions,): LinkupToolDefinition {
  return defineTool({
    name: LINKUP_WEB_SEARCH_TOOL_NAME,
    label: webSearchToolSpec.label,
    description: webSearchToolSpec.description,
    promptSnippet: webSearchToolSpec.promptSnippet,
    promptGuidelines: [...webSearchToolSpec.promptGuidelines,],
    parameters: valibotToTypeBox(webSearchToolSpec.parameters,),
    async execute(
      _toolCallId: string,
      rawParams: unknown,
      signal?: AbortSignal,
      onUpdate?: AgentToolUpdateCallback<LinkupToolDetails>,
      _ctx?: ExtensionContext,
    ) {
      /**
       Logger tagged for this tool registration.
       */
      const innerL = tagged({
        tag: LINKUP_WEB_SEARCH_TOOL_NAME,
        l,
      },);
      innerL.debug('delegating search execution to shared core',);
      return await executeWebSearchTool({
        client: options.client,
        config: options.config,
        rawParams,
        ...(signal === undefined ? {} : { signal, }),
        ...(onUpdate === undefined ? {} : { onUpdate, }),
      },);
    },
  },);
}

/**
 Create the Pi fetch tool over the shared fetch contract.

 @param options - config and client dependencies

 @returns Pi tool definition

 @example
 ```ts
 createLinkupWebFetchTool({ config, client });
 ```
 */
function createLinkupWebFetchTool(options: CreateLinkupToolsOptions,): LinkupToolDefinition {
  return defineTool({
    name: LINKUP_WEB_FETCH_TOOL_NAME,
    label: webFetchToolSpec.label,
    description: webFetchToolSpec.description,
    promptSnippet: webFetchToolSpec.promptSnippet,
    promptGuidelines: [...webFetchToolSpec.promptGuidelines,],
    parameters: valibotToTypeBox(webFetchToolSpec.parameters,),
    async execute(
      _toolCallId: string,
      rawParams: unknown,
      signal?: AbortSignal,
      onUpdate?: AgentToolUpdateCallback<LinkupToolDetails>,
      _ctx?: ExtensionContext,
    ) {
      /**
       Logger tagged for this tool registration.
       */
      const innerL = tagged({
        tag: LINKUP_WEB_FETCH_TOOL_NAME,
        l,
      },);
      innerL.debug('delegating fetch execution to shared core',);
      return await executeWebFetchTool({
        client: options.client,
        config: options.config,
        rawParams,
        ...(signal === undefined ? {} : { signal, }),
        ...(onUpdate === undefined ? {} : { onUpdate, }),
      },);
    },
  },);
}

//endregion Helpers
