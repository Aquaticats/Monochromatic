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
 Members both pools ask, in the order the reviewer has asked them.

 The writer asks these members from 2026-11-27: the owner typed "Change writer's list to match the reviewer's list."
 (`main:doc/decision/writer-rounds-take-qwen-first-through-the-virtual-writer.md`, "The writer's members become the
 reviewer's"), which supersedes for the writer the order of 2026-11-26, "Qwen, opencode-go, flex (Recommended)".

 One list, named by both pools, rather than two equal ones, so that a match with the reviewer's list cannot drift
 apart from it.

 NeuralWatt's standard DeepSeek lane is no member: the owner closed it to writers on 2026-11-11, and it left this
 list on 2026-11-26. The provider named `deepseek` is never a member of either pool (the same record).
 */
const BOTH_POOLS_MEMBERS = [
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
 The two pools the package offers, in the order Pi lists them. Both ask {@link BOTH_POOLS_MEMBERS}.
 */
export const POOL_DEFINITIONS: readonly PoolDefinition[] = [
  {
    name: 'writer',
    displayName: 'Writer',
    modelId: 'writer-pool',
    members: BOTH_POOLS_MEMBERS,
  },
  {
    name: 'reviewer',
    displayName: 'Reviewer',
    modelId: 'reviewer-pool',
    members: BOTH_POOLS_MEMBERS,
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
 pool no larger than a member built the same way; `poolModels` then advertises the head member's
 limits instead.
 */
const PROVISIONAL_LIMITS: PoolLimits = {
  contextWindow: 128_000,
  maxTokens: 16_384,
};

/**
 Limits one member reports, or zeroes for a member that is not registered.

 A member reporting no usable pair is passed over for the next member in the order, so the pool
 always advertises a window Pi can compact against; a request a member's own window cannot hold is
 passed over at that member alone (`src/member-admission.ts`).
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
 Limits one pool advertises: those of the head member a session registers.

 The head is the first member of the pool's order that the session registry resolves with a usable
 pair of limits, and that is the first member a request is asked of: dispatch skips an unregistered
 member the same way (`src/dispatch-request.ts`), so a member that reports no limits cannot stand for
 the pool either.

 A member further along that reports a smaller window no longer pulls the pool down, which is the
 owner's decision of 2026-11-27 (`doc/decision/writer-rounds-take-qwen-first-through-the-virtual-writer.md`,
 "Skip a member too small for the request, and advertise the head's window"): the router passes over
 that member only for a request its own window cannot hold, and Pi compacts on the head's window
 instead of on the smallest member's.

 @param members - ordered members of the pool

 @param readMemberLimits - registry reader, or undefined before a session binds

 @returns head member's limits, or Pi's own fallback when no member reports a usable pair

 @example
 ```ts
 headMemberLimits({ members: POOL_DEFINITIONS[0].members });
 ```
 */
export function headMemberLimits({
  members,
  readMemberLimits,
}: {
  readonly members: readonly Member[];
  readonly readMemberLimits?: MemberLimitsReader;
},): PoolLimits {
  for (const member of members) {
    /**
     Limits that member reported, zeroed when it is not registered.
     */
    const reported = readMemberLimits === undefined
      ? NO_LIMITS
      : readMemberLimits(
        member.provider,
        member.id,
      );
    if ((reported.contextWindow > 0) && (reported.maxTokens > 0))
      return reported;
  }
  return PROVISIONAL_LIMITS;
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
   Limits the pool advertises: its head registered member's.
   */
  const limits = headMemberLimits({
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

 Text is the shared modality. The limits are the head registered member's, so Pi compacts on the
 window of the member every request is asked of first; a member further along whose own window is
 smaller is passed over for a request it cannot hold rather than lowering what the pool advertises.
 A re-registration after the session binds reads them afresh.

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
