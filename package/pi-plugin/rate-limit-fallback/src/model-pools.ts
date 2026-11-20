/**
 Ordered routing decisions from issue 6314.
 @module
 */

import type {
  Api,
  Model,
} from '@earendil-works/pi-ai';

//region Public catalog and ordered targets

/**
 Provider selected in Pi.
 */
export const PROVIDER_ID = 'rate-limit-fallback';

/**
 Dedicated API identifier; no external protocol speaks it.
 */
const ROUTING_API = 'rate-limit-fallback';

/**
 Reviewer membership in preference order.
 */
const REVIEWER = [
  {
    provider: 'qwen-token-plan-individual',
    id: 'deepseek-v4.1-flash-1017',
  },
  {
    provider: 'synthetic',
    id: 'hf:deepseek-ai/DeepSeek-V4.1-Flash-1017',
  },
  {
    provider: 'opencode-go',
    id: 'deepseek-v4.1-flash',
  },
  {
    provider: 'neuralwatt',
    id: 'deepseek-v4.1-flash-flex',
  },
] as const;

/**
 Writer membership in preference order, decided 2026-11-19
 (`main:.agents/skills/pi-as-subagent/SKILL.md`, section "Who writes and who reviews"):
 DeepSeek V4.1 Flash on NeuralWatt's standard lane, then its flex lane, then OpenCode Go.
 */
const WRITER = [
  {
    provider: 'neuralwatt',
    id: 'deepseek-v4.1-flash',
  },
  {
    provider: 'neuralwatt',
    id: 'deepseek-v4.1-flash-flex',
  },
  {
    provider: 'opencode-go',
    id: 'deepseek-v4.1-flash',
  },
] as const;

/**
 Physical model being considered for a request.
 */
export type Member = Readonly<{
  provider: string;
  id: string
}>;

/**
 One pool: the public name Pi's selections carry, the physical model the router dispatches through,
 and the members it asks in order.

 The public name is the virtual model's id, so `rate-limit-fallback/writer` stays the selection a
 session makes; the physical model id carries `-pool` because Pi refuses a virtual model whose id a
 physical model of the same provider already holds
 (`@earendil-works/pi-coding-agent` dist/core/model-runtime.js, `registerVirtualModel`).
 */
export type PoolDefinition = Readonly<{
  name: 'writer' | 'reviewer';
  displayName: string;
  modelId: string;
  members: readonly Member[];
}>;

/**
 The two pools the package offers, in the order Pi lists them.
 */
export const POOL_DEFINITIONS: readonly PoolDefinition[] = [
  {
    name: 'writer',
    displayName: 'Writer',
    modelId: 'writer-pool',
    members: WRITER,
  },
  {
    name: 'reviewer',
    displayName: 'Reviewer',
    modelId: 'reviewer-pool',
    members: REVIEWER,
  },
];

/**
 Members one physical pool model dispatches to.

 Every pool has at least one member, so an empty list means no pool carries that id.

 @param id - physical model id the request selected

 @returns ordered members, or an empty list when no pool carries that id

 @example
 ```ts
 poolMembers('writer-pool');
 ```
 */
export function poolMembers(id: string,): readonly Member[] {
  return POOL_DEFINITIONS.find(function samePool(pool,) {
    return pool.modelId === id;
  },)
    ?.members
    ?? [];
}

/**
 Pool-wide limits advertised to Pi before any session registry binds.

 Pi compacts on a threshold taken from the selected model's window, so a zero window makes it compact
 on any non-empty context. These values are Pi's own fallback for a model that declares no window
 (`@earendil-works/pi-coding-agent` dist/core/provider-composer.js, `modelFromJson`), which keeps the
 pool no larger than a member built the same way; `poolModels` then advertises the smallest
 registered member's limits instead.
 */
const PROVISIONAL_LIMITS: PoolLimits = {
  contextWindow: 128_000,
  maxTokens: 16_384,
};

/**
 Limits one member reports, or zeroes for a member that is not registered.

 A zero is passed over rather than pulling the pool to zero: the pool must advertise a usable window,
 and a member that then rejects a context has its error returned like any other.
 */
export type PoolLimits = Readonly<{
  contextWindow: number;
  maxTokens: number
}>;

/**
 Zeroes a reader returns for a member it cannot resolve.
 */
export const NO_LIMITS: PoolLimits = {
  contextWindow: 0,
  maxTokens: 0,
};

/**
 Reads the limits one physical member reports from the session registry.
 */
export type MemberLimitsReader = (
  provider: string,
  id: string,
) => PoolLimits;

/**
 Smaller of two limits, treating zero as "not reported".

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
 Smallest limits every registered member of one pool can honour.

 @param members - ordered members of the pool

 @param readMemberLimits - registry reader, or undefined before a session binds

 @returns limits Pi may use for that pool, with Pi's own fallback when no member reports any

 @example
 ```ts
 advertisedLimits({ members: POOL_DEFINITIONS[0].members });
 ```
 */
export function advertisedLimits({
  members,
  readMemberLimits,
}: {
  readonly members: readonly Member[];
  readonly readMemberLimits?: MemberLimitsReader;
},): PoolLimits {
  /**
   Smallest limits accepted so far, zero meaning none reported yet.
   */
  const smallest = members.reduce(
    function foldLimits(
      accepted: PoolLimits,
      member: Member,
    ): PoolLimits {
      /**
       Limits that member reported, zeroed when it is not registered.
       */
      const reported = readMemberLimits === undefined
        ? NO_LIMITS
        : readMemberLimits(
          member.provider,
          member.id,
        );
      return {
        contextWindow: smallerPositive({
          current: accepted.contextWindow,
          reported: reported.contextWindow,
        },),
        maxTokens: smallerPositive({
          current: accepted.maxTokens,
          reported: reported.maxTokens,
        },),
      };
    },
    NO_LIMITS,
  );
  return {
    contextWindow: smallest.contextWindow === 0 ? PROVISIONAL_LIMITS.contextWindow : smallest.contextWindow,
    maxTokens: smallest.maxTokens === 0 ? PROVISIONAL_LIMITS.maxTokens : smallest.maxTokens,
  };
}

/**
 One pool's physical catalog entry, the model a virtual model routes through.

 @param pool - pool the entry belongs to

 @param readMemberLimits - registry reader, or undefined before a session binds

 @returns catalog entry Pi dispatches through

 @example
 ```ts
 poolEntryFor({ pool: POOL_DEFINITIONS[0] });
 ```
 */
export function poolEntryFor({
  pool,
  readMemberLimits,
}: {
  readonly pool: PoolDefinition;
  readonly readMemberLimits?: MemberLimitsReader;
},): Model<typeof ROUTING_API> {
  /**
   Limits every member of this pool can honour.
   */
  const limits = advertisedLimits({
    members: pool.members,
    ...(readMemberLimits === undefined ? {} : { readMemberLimits, }),
  },);
  return {
    id: pool.modelId,
    name: `${pool.displayName} pool`,
    api: ROUTING_API,
    provider: PROVIDER_ID,
    baseUrl: '',
    input: ['text',],
    reasoning: true,
    cost: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
    },
    contextWindow: limits.contextWindow,
    maxTokens: limits.maxTokens,
  };
}

/**
 Catalog Pi sees for the pool provider.

 Text is the shared modality. The limits are the smallest of the pool's registered members', so a
 compaction Pi decides here fits every member, and a re-registration after the session binds reads
 them afresh.

 @param readMemberLimits - registry reader, or undefined before a session binds

 @returns writer and reviewer pool entries with usable limits

 @example
 ```ts
 poolModels();
 ```
 */
export function poolModels({
  readMemberLimits,
}: {
  readonly readMemberLimits?: MemberLimitsReader;
} = {},): readonly Model<typeof ROUTING_API>[] {
  return POOL_DEFINITIONS.map(function entry(pool,) {
    return poolEntryFor({
      pool,
      ...(readMemberLimits === undefined ? {} : { readMemberLimits, }),
    },);
  },);
}

//endregion Public catalog and ordered targets
