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
 Physical routes by public model id.
 */
export const POOLS: Readonly<Record<'writer' | 'reviewer', readonly Member[]>> = {
  writer: WRITER,
  reviewer: REVIEWER,
};


/**
 Pool-wide limits advertised to Pi before any session registry binds.

 Pi compacts on a threshold taken from the selected model's window, so a zero window makes it compact
 on any non-empty context. These provisional values stand in until the session registry says which
 members are registered; `poolModels` then advertises the smallest registered member's limits.
 */
const PROVISIONAL_LIMITS: PoolLimits = {
  contextWindow: 1_048_576,
  maxTokens: 65_536,
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

 @param index - member position this step inspects

 @param smallest - smallest limits accepted so far

 @returns limits Pi may use for that pool
 */
function smallestMemberLimits({
  members,
  readMemberLimits,
  index,
  smallest,
}: {
  readonly members: readonly Member[];
  readonly readMemberLimits?: MemberLimitsReader;
  readonly index: number;
  readonly smallest: PoolLimits;
},): PoolLimits {
  /**
   Member this step inspects, absent once every member has been passed.
   */
  const member = members[index];
  if (member === undefined)
    return smallest;
  /**
   Limits that member reported, zeroed when it is not registered.
   */
  const reported = readMemberLimits === undefined
    ? NO_LIMITS
    : readMemberLimits(
      member.provider,
      member.id,
    );
  return smallestMemberLimits({
    members,
    ...(readMemberLimits === undefined ? {} : { readMemberLimits, }),
    index: index + 1,
    smallest: {
      contextWindow: smallerPositive({
        current: smallest.contextWindow,
        reported: reported.contextWindow,
      },),
      maxTokens: smallerPositive({
        current: smallest.maxTokens,
        reported: reported.maxTokens,
      },),
    },
  },);
}

/**
 One pool's catalog entry with the limits its members allow.

 @param id - public pool id

 @param name - display name

 @param members - ordered members of the pool

 @param readMemberLimits - registry reader, or undefined before a session binds

 @returns catalog entry Pi selects
 */
function poolEntry({
  id,
  name,
  members,
  readMemberLimits,
}: {
  readonly id: 'writer' | 'reviewer';
  readonly name: string;
  readonly members: readonly Member[];
  readonly readMemberLimits?: MemberLimitsReader;
},): Model<typeof ROUTING_API> {
  /**
   Limits every member of this pool can honour.
   */
  const reported = smallestMemberLimits({
    members,
    ...(readMemberLimits === undefined ? {} : { readMemberLimits, }),
    index: 0,
    smallest: NO_LIMITS,
  },);
  return {
    id,
    name,
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
    contextWindow: reported.contextWindow === 0 ? PROVISIONAL_LIMITS.contextWindow : reported.contextWindow,
    maxTokens: reported.maxTokens === 0 ? PROVISIONAL_LIMITS.maxTokens : reported.maxTokens,
  };
}

/**
 Catalog Pi sees for the pool provider.

 Text is the shared modality. The limits are the smallest of the pool's registered members', so a
 compaction Pi decides here fits every member, and a re-registration after the session binds reads
 them afresh.

 @param readMemberLimits - registry reader, or undefined before a session binds

 @returns writer and reviewer entries with usable limits

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
  return [
    poolEntry({
      id: 'writer',
      name: 'Writer',
      members: POOLS.writer,
      ...(readMemberLimits === undefined ? {} : { readMemberLimits, }),
    },),
    poolEntry({
      id: 'reviewer',
      name: 'Reviewer',
      members: POOLS.reviewer,
      ...(readMemberLimits === undefined ? {} : { readMemberLimits, }),
    },),
  ];
}

//endregion Public catalog and ordered targets
