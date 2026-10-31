#!/usr/bin/env node
/**
 MCP server entry point exposing the shared search and fetch tools.

 Loads the same global config the pi extension uses, wires the shared core to
 \@monochromatic-dev/mcp-stdio, and serves stdio transport when executed as a bin.

 @module
 */

import { realpath, } from 'node:fs/promises';
import {
  createSearchFetchClient,
  loadLinkupConfig,
  type LinkupConfig,
  type SearchFetchClient,
} from '@monochromatic-dev/search-fetch-core/ts';
import {
  createMcpServer,
  serve,
  type McpServerHandle,
} from '@monochromatic-dev/mcp-stdio/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  createResponseStore,
  type ResponseStore,
} from './response-store.ts';
import {
  createSearchFetchTools,
} from './tools.ts';

//region Logger

/**
 Logger root for the search-fetch MCP server.
 */
const searchFetchMcpLogger = tagged({ tag: 'search-fetch-mcp', },);

/**
 Module logger.
 */
const l = tagged({
  tag: 'index',
  l: searchFetchMcpLogger,
},);

//endregion Logger

//region Constants

/**
 Maximum number of full responses kept in the resource index.
 */
const RESPONSE_STORE_CAPACITY = 32;

/**
 Server identity reported through `server/discover`.
 */
const SERVER_IDENTITY = {
  name: 'search-fetch-mcp',
  version: '0.1.0',
  title: 'Search and fetch MCP server',
} as const;

/**
 Model-facing guidance for this server's tools.
 */
const SERVER_INSTRUCTIONS = 'Provides web_search and web_fetch with the same behavior as the pi search-fetch extension: DeepSeek-first search (maximum results with page content) with Exa and Linkup fallback, gh CLI routing for mapped GitHub URLs, DeepSeek fetch before Linkup and Exa, and a global host blocklist enforced before providers are called. Truncated results carry their full text as a search-fetch://response resource.';

//endregion Constants

//region Types

/**
 Dependencies injected into server assembly.
 */
export type CreateSearchFetchServerOptions = {
  /**
   Loaded config carrying the blocklist.
   */
  readonly config: LinkupConfig;
  /**
   Provider-routing client used by the tools.
   */
  readonly client: SearchFetchClient;
  /**
   Store receiving full responses that truncation hid from the model.
   */
  readonly store: ResponseStore;
};

//endregion Types

//region Public API

/**
 Assemble the MCP server over the shared core.

 @param options - config, client, and response store dependencies

 @returns server handle dispatching JSON-RPC over stdio

 @example
 ```ts
 const server = createSearchFetchServer({ config, client, store });
 ```
 */
export function createSearchFetchServer(options: CreateSearchFetchServerOptions,): McpServerHandle {
  return createMcpServer({
    config: {
      ...SERVER_IDENTITY,
      instructions: SERVER_INSTRUCTIONS,
    },
    tools: createSearchFetchTools({
      client: options.client,
      config: options.config,
      store: options.store,
    },),
    resources: options.store,
  },);
}

//endregion Public API

//region Public re-exports

export {
  createResponseStore,
  ResourceReadError,
} from './response-store.ts';
export {
  createSearchFetchTools,
} from './tools.ts';
export type {
  RecordedResponse,
  RecordResponseInput,
  ResponseStore,
} from './response-store.ts';
export type {
  CreateSearchFetchToolsOptions,
} from './tools.ts';
export type {
  LinkupConfig,
  SearchFetchClient,
} from '@monochromatic-dev/search-fetch-core/ts';

//endregion Public re-exports

//region Bin entry: runs only when executed directly

/**
 Whether this module was launched as the executable rather than imported for its exports.

 Compares real paths because bin shims reach the built file through symlinks.
 */
const isBinLaunch = await isDirectlyExecuted();

if (isBinLaunch) {
  /**
   Runtime config loaded from the same global file the pi extension uses.
   */
  const config = await loadLinkupConfig();
  /**
   Value-free store diagnostic forwarded to missing-key errors.
   */
  const { storeError, } = config.source;
  /**
   Provider-routing client shared by the registered tools.
   */
  const client = createSearchFetchClient({
    ...(config.deepseekApiKey === undefined ? {} : { deepseekApiKey: config.deepseekApiKey, }),
    ...(config.exaApiKey === undefined ? {} : { exaApiKey: config.exaApiKey, }),
    ...(config.linkupApiKey === undefined ? {} : { linkupApiKey: config.linkupApiKey, }),
    ...(storeError === undefined ? {} : { credentialHint: storeError, }),
    blocklist: config.blocklist,
  },);
  await serve({
    server: createSearchFetchServer({
      config,
      client,
      store: createResponseStore({ capacity: RESPONSE_STORE_CAPACITY, }),
    },),
  },);
  l.debug('search-fetch-mcp server exited',);
}

//endregion Bin entry

//region Helpers

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
  // not a bin launch, so resolution failure must never start the server by accident.
  try {
    return (await realpath(launchedPath,)) === modulePath;
  }
  catch (error: unknown) {
    l.debug(`could not resolve launch path ${launchedPath}: ${String(error,)}`,);
    return false;
  }
}

//endregion Helpers
