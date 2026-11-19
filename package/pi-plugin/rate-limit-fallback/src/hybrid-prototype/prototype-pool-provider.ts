/**
 A pool provider over a member list this prototype chooses, using the package's own dispatch.
 @module
 */

import {
  lazyStream,
  type Api,
  type AssistantMessageEvent,
  type AssistantMessageEventStream,
  type Model,
  type Provider,
  type SimpleStreamOptions,
  type StreamOptions,
  type TranscriptContext,
} from '@earendil-works/pi-ai';
import {
  dispatchRequest,
  type RoutingSession,
} from '../dispatch-request.ts';
import type { Member } from '../model-pools.ts';
import type { RoutingRegistry } from '../quota-endpoints.ts';
import {
  limitsOf,
  note,
} from './prototype-evidence.ts';

//region Pool catalog

/**
 Provider id of the prototype pool, kept apart from the package's own provider id.
 */
export const PROTOTYPE_POOL_PROVIDER_ID = 'rate-limit-fallback-probe';

/**
 The prototype pool's only pool id.
 */
export const PROTOTYPE_POOL_ID = 'probe';

/**
 Limits advertised before a session binds, matching the package's own provisional pair so the pool
 only carries a window the smallest member can honour.
 */
const PROVISIONAL_LIMITS = {
  contextWindow: 128_000,
  maxTokens: 16_384,
} as const;

/**
 Limits one member reported, or zeroes when Pi has no catalog record for it.
 */
type MemberLimits = Readonly<{
  contextWindow: number;
  maxTokens: number
}>;

/**
 Smaller of two reported limits, treating zero as unreported.

 @param current - smallest value accepted so far

 @param reported - value one member reported

 @returns the smaller positive value, or zero when neither reported one
 */
function smallerPositive({
  current,
  reported,
}: {
  readonly current: number;
  readonly reported: number;
},): number {
  if (reported <= 0)
    return current;
  return current <= 0 ? reported : Math.min(
    current,
    reported,
  );
}

/**
 Smallest limits every member of the prototype pool can honour.

 @param members - ordered members of the pool

 @param registry - session registry, or undefined before a session binds

 @returns limits the pool model advertises

 @example
 ```ts
 poolLimits({ members, registry });
 ```
 */
function poolLimits({
  members,
  registry,
}: {
  readonly members: readonly Member[];
  readonly registry?: RoutingRegistry;
},): MemberLimits {
  /**
   Smallest limits accepted so far, zero meaning none reported yet.
   */
  let smallest: MemberLimits = {
    contextWindow: 0,
    maxTokens: 0,
  };
  if (registry === undefined)
    return smallest;
  /**
   Catalog lookup bound to its own registry: `ModelRegistry.find` takes two strings, and a bare
   `find(` call with two arguments reads like an array callback reference to the repository's
   array-method rules. `src/index.ts` binds the same lookup for the same reason.
   */
  const findModel = registry
    .find
    .bind(registry,);
  for (const member of members) {
    /**
     Catalog record Pi holds for that member.
     */
    const found = findModel(
      member.provider,
      member.id,
    );
    if (found === undefined)
      continue;
    smallest = {
      contextWindow: smallerPositive({
        current: smallest.contextWindow,
        reported: found.contextWindow,
      },),
      maxTokens: smallerPositive({
        current: smallest.maxTokens,
        reported: found.maxTokens,
      },),
    };
  }
  return smallest;
}

/**
 Catalog entry the prototype pool advertises.

 @param members - ordered members of the pool

 @param registry - session registry, or undefined before a session binds

 @returns pool model Pi selects

 @example
 ```ts
 prototypePoolModel({ members });
 ```
 */
export function prototypePoolModel({
  members,
  registry,
}: {
  readonly members: readonly Member[];
  readonly registry?: RoutingRegistry;
},): Model<Api> {
  /**
   Smallest limits the members allow.
   */
  const limits = poolLimits({
    members,
    ...(registry === undefined ? {} : { registry, }),
  },);
  return {
    id: PROTOTYPE_POOL_ID,
    name: 'Hybrid prototype probe pool',
    api: PROTOTYPE_POOL_PROVIDER_ID,
    provider: PROTOTYPE_POOL_PROVIDER_ID,
    baseUrl: '',
    input: ['text',],
    reasoning: false,
    cost: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
    },
    contextWindow: limits.contextWindow === 0
      ? PROVISIONAL_LIMITS.contextWindow
      : limits.contextWindow,
    maxTokens: limits.maxTokens === 0 ? PROVISIONAL_LIMITS.maxTokens : limits.maxTokens,
  };
}

//endregion Pool catalog

//region Provider and session binding

/**
 Capabilities the prototype pool needs from Pi's extension API.
 */
export type PoolHost = {
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
      context: { readonly modelRegistry: RoutingRegistry }
    ) => void
  ) => unknown;
};

/**
 Register the prototype pool provider and bind it to each started session.

 @param host - Pi extension API

 @param members - ordered members the probe pool dispatches to, in the pool's preference order

 @example
 ```ts
 registerPrototypePool({ host: pi, members: [{ provider: 'hybrid-probe-member', id: 'answers' }] });
 ```
 */
export function registerPrototypePool({
  host,
  members,
}: {
  readonly host: PoolHost;
  readonly members: readonly Member[];
},): void {
  /**
   Registry and quota evidence belonging to the current session.
   */
  /**
   Live session binding, or undefined before Pi starts one.
   */
  const binding: { session?: RoutingSession; } = {};
  note({
    event: 'prototype-pool-registered',
    members: members.map(function label(member: Member,): string {
      return `${member.provider}/${member.id}`;
    },),
  },);
  /**
   Provider object registered with the host.
   */
  const provider: Provider = {
    id: PROTOTYPE_POOL_PROVIDER_ID,
    name: 'Hybrid prototype probe pool',
    auth: {
      apiKey: {
        name: 'Probe pool',
        resolve() {
          return Promise.resolve({
            auth: {},
            source: PROTOTYPE_POOL_PROVIDER_ID,
          },);
        },
      },
    },
    getModels() {
      return [
        prototypePoolModel({
          members,
          ...(binding.session === undefined ? {} : { registry: binding.session
            .registry, }),
        },),
      ];
    },
    stream(
      model: Model<Api>,
      context: TranscriptContext,
      options?: StreamOptions,
    ): AssistantMessageEventStream {
      note({
        event: 'pool-request',
        reason: 'stream',
        maxTokens: options?.maxTokens,
        ...limitsOf({ model, }),
      },);
      return lazyStream(
        model,
        function prepare(): Promise<AsyncIterable<AssistantMessageEvent>> {
          if (binding.session === undefined)
            return Promise.reject(new Error('The prototype pool requires a started Pi session.',),);
          return Promise.resolve(dispatchRequest({
            session: binding.session,
            model,
            context,
            pools: { [PROTOTYPE_POOL_ID]: members, },
            ...(options === undefined ? {} : { options, }),
          },),);
        },
        undefined,
        options?.signal,
      );
    },
    streamSimple(
      model: Model<Api>,
      context: TranscriptContext,
      options?: SimpleStreamOptions,
    ): AssistantMessageEventStream {
      return provider.stream(
        model,
        context,
        options,
      );
    },
  };
  host.registerProvider(provider,);
  host.on(
    'session_start',
    function bindRegistry(
      _event: unknown,
      context: { readonly modelRegistry: RoutingRegistry },
    ): void {
      binding.session = {
        registry: context.modelRegistry,
        members: new Map(),
      };
      // Pi composes its model snapshot from getModels() when a provider is registered, so the pool
      // registers again now that the registry is known and advertises the members' own limits.
      host.registerProvider(provider,);
    },
  );
}

//endregion Provider and session binding
