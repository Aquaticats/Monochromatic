/**
 Pi extension entry for rate-limit fallback between the declared writer and reviewer members.
 @module
 */

import {
  lazyStream,
  type Api,
  type ApiStreamOptions,
  type AssistantMessageEvent,
  type AssistantMessageEventStream,
  type Model,
  type Provider,
  type SimpleStreamOptions,
  type TranscriptContext,
} from '@earendil-works/pi-ai';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  dispatchRequest,
  PoolDispatchError,
  type RoutingSession,
} from './dispatch-request.ts';
import {
  type MemberLimitsReader,
  NO_LIMITS,
  poolModels,
  PROVIDER_ID,
} from './model-pools.ts';

import type { RoutingRegistry, } from './quota-endpoints.ts';

export { PROVIDER_ID, } from './model-pools.ts';

/**
 Response timing parsed into a reset time, exported so the built-artifact tests reach it.

 @internal
 */
export { headerReset, } from './rate-limit-evidence.ts';

/**
 Minimum interval between two quota reads of one member, exported so the suites use the shipped
 value.

 @internal
 */
export { QUOTA_READ_INTERVAL_MS, } from './member-capacity.ts';

/**
 Reads one member's limits from the session registry.

 @param registry - the session's live model registry

 @returns reader that zeroes a member the registry does not resolve
 */
function memberLimitsReader({ registry, }: { readonly registry: RoutingRegistry; },): MemberLimitsReader {
  /**
   Registry object the lookup belongs to.
   */
  const owner = registry;
  /**
   Registry lookup bound to its own object, so the reader can be called freely.
   */
  const resolve = owner
    .find
    .bind(owner,);
  return function memberLimits(
    provider: string,
    id: string,
  ): typeof NO_LIMITS {
    /**
     Catalog record Pi resolves for that member, or undefined when it has none.
     */
    const found = resolve(
      provider,
      id,
    );
    if (found === undefined)
      return NO_LIMITS;
    return {
      contextWindow: found.contextWindow,
      maxTokens: found.maxTokens,
    };
  };
}

/**
 Request-time catalog supplied by each started session.
 */
type SessionContext = {
  /**
   Registry operations needed by the pool.
   */
  readonly modelRegistry: RoutingRegistry;
};

/**
 Host capabilities used by this extension, without unrelated command or UI operations.
 */
type FallbackHost = {
  /**
   Make the pool available for model selection.
   */
  readonly registerProvider: (provider: Provider) => void;
  /**
   Bind the registry supplied by the active session.
   */
  readonly on: (
    event: 'session_start',
    handler: (
      event: unknown,
      context: SessionContext
    ) => void
  ) => () => void;
};

//region Register the provider before startup model selection

/**
 Logger for session binding and registration, never credential or transcript data.
 */
const l = tagged({ tag: 'rate-limit-fallback', },);

/**
 Register the keyless pool provider and bind it to the current Pi session.
 Both stream entry points use provider-neutral options because the picker cannot know a member's protocol.

 @param pi - host extension API

 @example
 ```ts
 rateLimitFallbackExtension(pi); // Called by Pi's extension loader.
 ```
 */
export default function rateLimitFallbackExtension(pi: FallbackHost,): void {
  /**
   Tagged logger that names the current operation without exposing credentials.
   */
  const rl = tagged({
    tag: rateLimitFallbackExtension.name,
    l,
  },);
  /**
   Registry and quota cache belonging to the current Pi session.
   */
  const binding: { session?: RoutingSession; } = {};

  /**
   Bridge Pi's synchronous stream entry to asynchronous member dispatch.

   @param model - selected pool model

   @param context - normalized transcript

   @param options - request options

   @returns stream of the eventual member, with no events from discarded attempts
   */
  function route({
    model,
    context,
    options,
  }: {
    readonly model: Model<Api>;
    readonly context: TranscriptContext;
    readonly options?: SimpleStreamOptions;
  },): AssistantMessageEventStream {
    return lazyStream(
      model,
      function prepare(): Promise<AsyncIterable<AssistantMessageEvent>> {
        if (binding.session === undefined)
          return Promise.reject(new PoolDispatchError('The rate-limit-fallback provider requires a started Pi session.',),);
        return Promise.resolve(dispatchRequest({
          session: binding.session,
          model,
          context,
          ...(options === undefined ? {} : { options, }),
        },),);
      },
      undefined,
      options?.signal,
    );
  }

  /**
   Provider object registered with the host.
   */
  const provider: Provider = {
    id: PROVIDER_ID,
    name: 'Rate limit fallback',
    auth: {
      apiKey: {
        name: 'Keyless pool',
        resolve() {
          return Promise.resolve({
            auth: {},
            source: PROVIDER_ID,
          },);
        },
      },
    },
    getModels() {
      // The smallest registered member's limits, so Pi's compaction threshold fits every
      // member; model-pools' provisional limits stand in before a session binds.
      /**
       Session bound by the last session start, or undefined before one.
       */
      const { session, } = binding;
      return session === undefined
        ? poolModels()
        : poolModels({ readMemberLimits: memberLimitsReader({ registry: session.registry, },), },);
    },
    stream: function streamPool<T extends Api>(
      model: Model<T>,
      context: TranscriptContext,
      options?: ApiStreamOptions<T>,
    ): AssistantMessageEventStream {
      // API-specific controls have no meaning on a pool. Keep only shared request options.
      /**
       Neutral subset of API-specific options understood by every pool member.
       */
      const {
        signal,
        onPayload,
        onResponse,
        onProviderStreamEvent,
        maxTokens,
        fetch,
        timeoutMs,
        maxRetries,
        maxRetryDelayMs,
        sessionId,
      } = options ?? {};
      return route({
        model,
        context,
        options: {
          ...(signal === undefined ? {} : { signal, }),
          ...(onPayload === undefined ? {} : { onPayload, }),
          ...(onResponse === undefined ? {} : { onResponse, }),
          ...(onProviderStreamEvent === undefined ? {} : { onProviderStreamEvent, }),
          ...(maxTokens === undefined ? {} : { maxTokens, }),
          ...(fetch === undefined ? {} : { fetch, }),
          ...(timeoutMs === undefined ? {} : { timeoutMs, }),
          ...(maxRetries === undefined ? {} : { maxRetries, }),
          ...(maxRetryDelayMs === undefined ? {} : { maxRetryDelayMs, }),
          ...(sessionId === undefined ? {} : { sessionId, }),
        },
      },);
    },
    streamSimple(
      model,
      context,
      options,
    ) {
      return route({
        model,
        context,
        ...(options === undefined ? {} : { options, }),
      },);
    },
  };
  pi.registerProvider(provider,);
  rl.debug(`Registered ${PROVIDER_ID}`,);
  pi.on(
    'session_start',
    function bindRegistry(
      _event: unknown,
      context,
    ): void {
      binding.session = {
        registry: context.modelRegistry,
        members: new Map(),
      };
      // Pi composes its model snapshot from getModels() when a provider is registered, so the pool
      // registers again now that the registry is known and the catalog can carry the smallest
      // registered member's limits instead of the provisional ones.
      pi.registerProvider(provider,);
      rl.debug('Bound session model registry',);
    },
  );
}

//endregion Register the provider before startup model selection
