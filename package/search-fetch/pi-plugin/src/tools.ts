/**
 Pi tool adapters for the shared search and fetch core.

 The shared specs in \@monochromatic-dev/search-fetch-core own names,
 descriptions, and valibot parameter schemas; this module converts those schemas to the
 TypeBox form pi compiles and forwards execution to the shared executors.
 Both tools register through one factory, so the host contract has a single owner.

 @module
 */

import {
  defineTool,
  type AgentToolUpdateCallback,
  type ExtensionContext,
  type ToolDefinition,
} from '@earendil-works/pi-coding-agent';
import {
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

/**
 Executor contract shared execution offers and both Pi tools delegate to.
 */
type LinkupToolExecutor = typeof executeWebSearchTool;

/**
 Host contract a Pi tool renders from.
 */
type LinkupToolSpec = typeof webSearchToolSpec | typeof webFetchToolSpec;

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
    createLinkupTool({
      options,
      spec: webSearchToolSpec,
      executeTool: executeWebSearchTool,
    },),
    createLinkupTool({
      options,
      spec: webFetchToolSpec,
      executeTool: executeWebFetchTool,
    },),
  ];
}

//endregion Public API

//region Helpers

/**
 Render one shared spec as the Pi tool contract and delegate execution to the shared executor.

 @param options - config and client dependencies

 @param spec - shared tool contract owning the name, labels, descriptions, and parameters

 @param executeTool - shared executor running one call of that tool

 @returns Pi tool definition

 @example
 ```ts
 createLinkupTool({ options, spec: webSearchToolSpec, executeTool: executeWebSearchTool });
 ```
 */
function createLinkupTool(
  {
    options,
    spec,
    executeTool,
  }: {
    /**
     Config and client dependencies shared by both tools.
     */
    readonly options: CreateLinkupToolsOptions;
    /**
     Shared tool contract this registration renders.
     */
    readonly spec: LinkupToolSpec;
    /**
     Shared executor this registration delegates to.
     */
    readonly executeTool: LinkupToolExecutor;
  },
): LinkupToolDefinition {
  return defineTool({
    name: spec.name,
    label: spec.label,
    description: spec.description,
    promptSnippet: spec.promptSnippet,
    promptGuidelines: [...spec.promptGuidelines,],
    parameters: valibotToTypeBox(spec.parameters,),
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
        tag: spec.name,
        l,
      },);
      innerL.debug(`delegating ${spec.name} execution to shared core`,);
      return await executeTool({
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
