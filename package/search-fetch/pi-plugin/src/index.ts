/**
 Pi Search Fetch extension entry point.

 Registers the shared provider-neutral search and fetch tools with pi,
 keeping global host blocklist enforcement and no account-management surfaces.
 Behavior lives in \@monochromatic-dev/search-fetch-core;
 this module only wires pi registration.

 @module
 */

import type { ExtensionAPI, } from '@earendil-works/pi-coding-agent';
import {
  createSearchFetchClient,
  loadLinkupConfig,
  type LinkupConfig,
  type SearchFetchToolClient,
} from '@monochromatic-dev/search-fetch-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  createLinkupTools,
} from './tools.ts';

/**
 Logger root for pi-search-fetch after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: searchFetchLogger, },);
 ```
 */
const searchFetchLogger = tagged({ tag: 'pi-search-fetch', },);

/**
 Module logger.
 */
const l = tagged({
  tag: 'index',
  l: searchFetchLogger,
},);

//region Types

/**
 Options for registering Pi Search Fetch with injected dependencies.
 */
export type RegisterPiLinkupOptions = {
  /**
   Pi extension API.
   */
  readonly pi: ExtensionAPI;
  /**
   Loaded Search Fetch config.
   */
  readonly config: LinkupConfig;
  /**
   Provider-routing client used by tools.
   */
  readonly client: SearchFetchToolClient;
};

//endregion Types

//region Extension entry point

/**
 Pi Search Fetch extension factory.
 
 @param pi - Pi extension API
 
 @example
 ```ts
 // In ~/.pi/agent/settings.json:
 { "packages": ["@monochromatic-dev/search-fetch-pi-plugin"] }
 ```
 */
export default async function piLinkup(pi: ExtensionAPI,): Promise<void> {
  /**
   Logger tagged for extension startup.
   */
  const innerL = tagged({
    tag: piLinkup.name,
    l,
  },);
  /**
   Runtime config loaded from the global Pi extension config file.
   */
  const config = await loadLinkupConfig();
  /**
   Value-free store diagnostic forwarded to missing-key errors.
   */
  const { storeError, } = config.source;
  /**
   Provider-routing client shared by registered tools.
   */
  const client = createSearchFetchClient({
    ...(config.deepseekApiKey === undefined ? {} : { deepseekApiKey: config.deepseekApiKey, }),
    ...(config.exaApiKey === undefined ? {} : { exaApiKey: config.exaApiKey, }),
    ...(config.linkupApiKey === undefined ? {} : { linkupApiKey: config.linkupApiKey, }),
    ...(storeError === undefined ? {} : { credentialHint: storeError, }),
    blocklist: config.blocklist,
  },);

  registerPiLinkup({
    pi,
    config,
    client,
  },);
  innerL.debug(`pi-search-fetch extension loaded; blocklist entries=${String(
    config
      .blocklist
      .length,
  )}`,);
}

/**
 Register Pi Search Fetch tools using already-created dependencies.
 
 @param options - Pi API, config, and client
 
 @example
 ```ts
 registerPiLinkup({ pi, config, client });
 ```
 */
export function registerPiLinkup(options: RegisterPiLinkupOptions,): void {
  /**
   Logger tagged for registration.
   */
  const innerL = tagged({
    tag: registerPiLinkup.name,
    l,
  },);
  /**
   Public Search Fetch tools.
   */
  const tools = createLinkupTools({
    config: options.config,
    client: options.client,
  },);

  tools.forEach(function registerTool(tool,) {
    options.pi
      .registerTool(tool,);
    innerL.debug(`registered Search Fetch tool: ${tool.name}`,);
  },);
}

//endregion Extension entry point

//region Public re-exports

export {
  LINKUP_WEB_FETCH_TOOL_NAME,
  LINKUP_WEB_SEARCH_TOOL_NAME,
} from '@monochromatic-dev/search-fetch-core/ts';
export {
  createLinkupTools,
} from './tools.ts';
export type {
  LinkupConfig,
  LinkupWebFetchInput,
  LinkupWebSearchInput,
  SearchFetchToolClient,
} from '@monochromatic-dev/search-fetch-core/ts';
export type {
  CreateLinkupToolsOptions,
  LinkupToolDefinition,
} from './tools.ts';

//endregion Public re-exports
