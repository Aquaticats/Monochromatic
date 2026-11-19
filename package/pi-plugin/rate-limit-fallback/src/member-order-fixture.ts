/**
 Pool orders the project decided, spelled out for tests that must not read the package's own table.
 @module
 */

import type {
  Api,
  Model,
} from '@earendil-works/pi-ai';
import { PHYSICAL_MODELS, } from './stand-in-pi-fixture.ts';

/**
 Provider and physical model id of one member, the pair that names it exactly.
 */
type MemberLabel = Readonly<{
  provider: string;
  id: string
}>;

/**
 Catalog record for one physical member named by its exact provider and model.

 @param provider - provider id as Pi reports it

 @param id - physical model id

 @returns catalog model the stand-in registry would return

 @throws Error when the fixture has no such target
 */
function targetOf({
  provider,
  id,
}: MemberLabel,): Model<Api> {
  for (const model of PHYSICAL_MODELS) {
    if ((model.provider === provider) && (model.id === id))
      return model;
  }
  throw new Error(`No fixture model for ${provider}/${id}`,);
}

/**
 Writer members in the order the project chose on 2026-11-19
 (`.agents/skills/pi-as-subagent/SKILL.md`, "Who writes and who reviews").
 Spelled out here rather than read from the package's pool table, so a pool change fails a test.
 */
export const WRITER_TARGETS: readonly [
  Model<Api>,
  Model<Api>,
  Model<Api>,
] = [
  targetOf({
    provider: 'neuralwatt',
    id: 'deepseek-v4.1-flash',
  },),
  targetOf({
    provider: 'neuralwatt',
    id: 'deepseek-v4.1-flash-flex',
  },),
  targetOf({
    provider: 'opencode-go',
    id: 'deepseek-v4.1-flash',
  },),
];

/**
 Reviewer members in preference order, unchanged by the 2026-11-19 writer decision.
 */
export const REVIEWER_TARGETS: readonly [
  Model<Api>,
  Model<Api>,
  Model<Api>,
  Model<Api>,
] = [
  targetOf({
    provider: 'qwen-token-plan-individual',
    id: 'deepseek-v4.1-flash-1017',
  },),
  targetOf({
    provider: 'synthetic',
    id: 'hf:deepseek-ai/DeepSeek-V4.1-Flash-1017',
  },),
  targetOf({
    provider: 'opencode-go',
    id: 'deepseek-v4.1-flash',
  },),
  targetOf({
    provider: 'neuralwatt',
    id: 'deepseek-v4.1-flash-flex',
  },),
];

/**
 Exact member label, as the pool logs it and the terminal record names it.

 @param model - physical member or catalog target

 @returns `provider/id`, which is unique even when two members share a provider

 @example
 ```ts
 labelOf({ provider: 'neuralwatt', id: 'deepseek-v4.1-flash' }); // 'neuralwatt/deepseek-v4.1-flash'
 ```
 */
export function labelOf(model: MemberLabel,): string {
  return `${model.provider}/${model.id}`;
}
