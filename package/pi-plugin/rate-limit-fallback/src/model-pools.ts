/**
 Ordered routing decisions from issue 6314.
 @module
 */

import type { Model, } from '@earendil-works/pi-ai';

//region Public catalog and ordered targets

/** Provider selected in Pi. */
export const PROVIDER_ID = 'rate-limit-fallback';

/** Dedicated API identifier; no external protocol speaks it. */
const ROUTING_API = 'rate-limit-fallback';

/** Reviewer membership in preference order. */
const REVIEWER = [
  { provider: 'qwen-token-plan-individual', id: 'deepseek-v4.1-flash-1017', },
  { provider: 'synthetic', id: 'hf:deepseek-ai/DeepSeek-V4.1-Flash-1017', },
  { provider: 'opencode-go', id: 'deepseek-v4.1-flash', },
  { provider: 'neuralwatt', id: 'deepseek-v4.1-flash-flex', },
] as const;

/** Physical model being considered for a request. */
type Member = Readonly<{ provider: string; id: string; }>;

/** Physical routes by public model id. */
export const POOLS: Readonly<Record<'writer' | 'reviewer', readonly Member[]>> = {
  writer: [{ provider: 'openai', id: 'gpt-6.2-sol', }, ...REVIEWER,],
  reviewer: REVIEWER,
};

/**
 Catalog available before an extension receives its session registry.
 Text is the shared modality; zero limits describe unknown pool-wide values.
 Dispatch applies the selected member's real output limit and pricing.
 */
export const MODELS: readonly Model<typeof ROUTING_API>[] = [
  {
    id: 'writer',
    name: 'Writer',
    api: ROUTING_API,
    provider: PROVIDER_ID,
    baseUrl: '',
    input: ['text',],
    reasoning: true,
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, },
    contextWindow: 0,
    maxTokens: 0,
  },
  {
    id: 'reviewer',
    name: 'Reviewer',
    api: ROUTING_API,
    provider: PROVIDER_ID,
    baseUrl: '',
    input: ['text',],
    reasoning: true,
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, },
    contextWindow: 0,
    maxTokens: 0,
  },
];

//endregion Public catalog and ordered targets
