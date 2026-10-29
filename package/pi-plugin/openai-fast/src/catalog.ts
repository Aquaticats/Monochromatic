/**
 Read-only bootstrap of the host's configured OpenAI catalogs and native providers. @module
 */

import type { Provider, } from '@earendil-works/pi-ai';
import { ModelRuntime, } from '@earendil-works/pi-coding-agent';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { CODEX_PROVIDER, } from './constants.ts';
import { FastModelError, } from './fast-model-error.ts';
import { createCatalogCredentials, } from './catalog-credentials.ts';

export { createCatalogCredentials, } from './catalog-credentials.ts';

//region Credential-free catalog bootstrap

/**
 Module logger reports lifecycle without credential or prompt data.
 */
const moduleLogger = tagged({ tag: 'pi-plugin-openai-fast.catalog', },);

/**
 Load original provider metadata and persisted catalogs without real credentials or network refresh.
 
 @param modelsPath - optional disposable model configuration for tests

 @param providerId - original provider whose native metadata remains authoritative
 
 @returns effective native provider whose auth resolves later in the real host
 
 @throws FastModelError when the model configuration or native provider is unavailable

 @example
 ```ts
 const provider = await loadOriginalProvider();
 ```
 */
export async function loadOriginalProvider({
  modelsPath,
  providerId = CODEX_PROVIDER,
}: {
  readonly modelsPath?: string;
  readonly providerId?: string;
} = {},): Promise<Provider> {
  /**
   Catalog logger excludes credential and header values.
   */
  const l = tagged({
    tag: loadOriginalProvider.name,
    l: moduleLogger,
  },);
  l.debug(`loading configured ${providerId} model metadata without credential access`,);
  /**
   Disposable metadata runtime has no access to stored real credentials.
   */
  const catalog = await ModelRuntime.create({
    credentials: createCatalogCredentials(),
    refreshOnCreate: false,
    allowModelNetwork: false,
    ...(modelsPath === undefined ? {} : { modelsPath, }),
  },);
  /**
   Native cache restoration runs without network or OAuth refresh.
   */
  const refresh = await catalog.refresh({
    allowNetwork: false,
    providers: [providerId,],
  },);
  /**
   Cache restoration errors remain explicit initialization failures.
   */
  const refreshError = refresh.errors
    .get(providerId,);
  if (refreshError !== undefined)
    throw new FastModelError(`Cannot restore the ${providerId} model catalog: ${String(refreshError,)}. Correct the cached catalog or model configuration.`,);
  /**
   Configuration diagnostics prevent registering misleading companion entries.
   */
  const problem = catalog.getError();
  if (problem !== undefined)
    throw new FastModelError(`Cannot initialize ${providerId} fast models: ${problem}. Correct the model configuration before loading the fast extension.`,);
  /**
   Effective source includes configured metadata and restored native cache data.
   */
  const provider = catalog.getProvider(providerId,);
  if (provider === undefined)
    throw new FastModelError(`The native ${providerId} provider is unavailable. Restore the native provider before loading the fast extension.`,);
  l.debug(`loaded ${provider.getModels()
    .length} base ${providerId} models`,);
  return provider;
}

/**
 {@inheritDoc loadOriginalProvider}

 @param options - optional disposable model configuration for tests

 @returns original legacy provider whose authentication remains host-owned
 */
export function loadCodexProvider(options: { readonly modelsPath?: string; } = {},): ReturnType<typeof loadOriginalProvider> {
  return loadOriginalProvider(options,);
}

//endregion
