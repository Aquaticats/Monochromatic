/**
 Native virtual OpenAI fast entries using the unchanged original provider. @module
 */

import type { Provider, } from '@earendil-works/pi-ai';
import {
  ModelRuntime,
  type ExtensionAPI,
  type SessionStartEvent,
} from '@earendil-works/pi-coding-agent';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type {
  ForeignBorrowed,
  ForeignHostCapability,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { loadOriginalProvider, } from './catalog.ts';
import {
  CODEX_PROVIDER,
  OPENAI_PROVIDER,
} from './constants.ts';
import { createOriginalDispatch, } from './original-dispatch.ts';
import { createPriorityProvider, } from './priority-provider.ts';
import { createFastModelRegistration, } from './virtual-registration.ts';

export {
  createCatalogCredentials,
  loadCodexProvider,
  loadOriginalProvider,
} from './catalog.ts';
export {
  CODEX_API,
  CODEX_PROVIDER,
  FAST_PROVIDER,
  OPENAI_API,
  OPENAI_PROVIDER,
  PRIORITY_TARGET_PREFIX,
  type PriorityApi,
} from './constants.ts';
export { FastModelError, } from './fast-model-error.ts';
export { createKeylessAuth, } from './keyless-auth.ts';
export {
  createPriorityProvider,
  isPriorityTarget,
  priorityTarget,
  resolvePriorityBase,
} from './priority-provider.ts';
export { createFastModelRegistration, } from './virtual-registration.ts';
export {
  streamPriority,
  streamSimplePriority,
} from './priority-stream.ts';
export { forcePriorityPayload, } from './priority-payload.ts';
export { PriorityRequestError, } from './priority-error.ts';

//region Native registration

/**
 Module logger records lifecycle without additional selection UI.
 */
const moduleLogger = tagged({ tag: 'pi-plugin-openai-fast.index', },);

/**
 Register virtual companions and keyless targets without replacing the original provider.
 
 @param pi - host registration capability
 
 @param provider - credential-free bootstrap metadata source

 @param checkInitialAvailability - native source readiness before session_start
 
 @mutates pi - registers adapter, virtual models, and session-start callback
 
 @mutates provider - invokes original metadata accessors

 @example
 ```ts
 registerOpenAIFast({ pi, provider });
 ```
 */
export function registerOpenAIFast({
  pi,
  provider,
  checkInitialAvailability,
}: {
  readonly pi: ForeignHostCapability<ExtensionAPI>;
  readonly provider: ForeignHostCapability<Provider>;
  readonly checkInitialAvailability?: (signal: ForeignBorrowed<AbortSignal>) => Promise<boolean>;
},): void {
  /**
   Registration logger carries the module boundary into helper calls.
   */
  const l = tagged({
    tag: registerOpenAIFast.name,
    l: moduleLogger,
  },);
  l.debug('registering virtual priority companions',);
  /**
   Session binding is private state, never a global request-tier switch.
   */
  const binding = createOriginalDispatch({
    provider,
    l,
    ...(checkInitialAvailability === undefined ? {} : { checkInitialAvailability, }),
  },);
  /**
   Structural registration guard prevents getter-triggered reentrant catalog recursion.
   */
  const synchronize = createFastModelRegistration({
    pi,
    fastProvider: `${provider.id}-fast`,
  },);
  pi.registerProvider(createPriorityProvider({
    provider,
    getProvider: binding.getProvider,
    lookup: binding.lookup,
    isConfigured: binding.isConfigured,
    dispatch: binding.stream,
    onCatalog: synchronize,
  },),);
  synchronize(provider.getModels(),);
  pi.on(
    'session_start',
    function sessionStart(
      _event: ForeignBorrowed<SessionStartEvent>,
      ctx,
    ) {
    /**
     Session logger exposes binding lifecycle without payloads or credentials.
     */
      const sl = tagged({
        tag: sessionStart.name,
        l,
      },);
      binding.bind(ctx.modelRegistry,);
      synchronize(binding.getProvider()
        .getModels(),);
      sl.debug('bound priority dispatch to the unchanged original pi provider',);
    },
  );
}

//endregion

//region Extension factory

/**
 Initialize both configured OpenAI companion families before startup model selection.
 
 @param pi - native registration capability
 
 @mutates pi - delegates adapter and virtual-model registration

 @example
 ```ts
 await openAIFast(pi);
 ```
 */
export default async function openAIFast(pi: ForeignHostCapability<ExtensionAPI>,): Promise<void> {
  /**
   Factory logger records initialization without another footer or notification.
   */
  const l = tagged({
    tag: openAIFast.name,
    l: moduleLogger,
  },);
  l.debug('initializing virtual OpenAI fast extension',);
  /**
   Bootstrap uses metadata only; real request auth stays in the active original provider.
   */
  const providers = await Promise.all([
    CODEX_PROVIDER,
    OPENAI_PROVIDER,
  ].map(async function loadProvider(providerId,) {
    return await loadOriginalProvider({ providerId, },);
  },),);
  /**
   Native source readiness uses the original pi auth path without OAuth refresh or network catalog updates.
   */
  const availability = await ModelRuntime.create({
    refreshOnCreate: false,
    allowModelNetwork: false,
  },);
  // Finish sink initialization before synchronous catalog registration can fill startup buffering.
  await l.flush();
  for (const provider of providers) {
    registerOpenAIFast({
      pi,
      provider,
      checkInitialAvailability: async function checkInitialAvailability(signal: ForeignBorrowed<AbortSignal>): Promise<boolean> {
        return (await availability.getAvailableOfType(
          'chat',
          provider.id,
          { signal, },
        )).length > 0;
      },
    },);
  }
  l.debug('virtual OpenAI fast extension initialized',);
}

//endregion
