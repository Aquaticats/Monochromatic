/**
 Unit tests for the DSH default-ranking policy.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  dshRankDefaultCandidates,
} from '../dist/final/node/index.mjs';
import type {
  AdvisorRankingCandidate,
  ScopedAdvisorModel,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';

/**
 Build one scoped model candidate with the given ranking-relevant facts.

 @param options - slug, display name, and context window for the model

 @returns scoped model usable as a ranking candidate

 @example
 ```ts
 const scoped = scopedModelFixture({ slug: 'p/big', name: 'Big', contextWindow: 200_000 });
 ```
 */
function scopedModelFixture(
  options: {
    readonly slug: string;
    readonly name: string;
    readonly contextWindow: number;
  },
): ScopedAdvisorModel {
  return {
    canonicalSlug: options.slug,
    model: {
      provider: 'provider',
      id: options.slug,
      name: options.name,
      api: 'dsh',
      baseUrl: '',
      reasoning: true,
      input: ['text',],
      cost: {
        input: 0,
        output: 0,
        cacheRead: 0,
        cacheWrite: 0,
      },
      contextWindow: options.contextWindow,
      maxTokens: 8_192,
    },
  };
}

/**
 Wrap one scoped model as an eligible ranking candidate.

 @param scopedModel - scoped model under consideration

 @returns ranking candidate carrying a fixed input estimate

 @example
 ```ts
 const candidate = candidateFixture(scopedModelFixture({ slug: 'p/m', name: 'M', contextWindow: 100 }));
 ```
 */
function candidateFixture(
  scopedModel: ScopedAdvisorModel,
): AdvisorRankingCandidate {
  return {
    scopedModel,
    estimatedInputTokens: 1_000,
  };
}

await describe({
  name: dshRankDefaultCandidates.name,
  children: [
    it({
      name: 'orders largest context window first',
      fn: async () => {
        const small = scopedModelFixture({
          slug: 'p/small',
          name: 'Small',
          contextWindow: 32_000,
        },);
        const big = scopedModelFixture({
          slug: 'p/big',
          name: 'Big',
          contextWindow: 200_000,
        },);
        const decision = dshRankDefaultCandidates({
          candidates: [
            candidateFixture(small,),
            candidateFixture(big,),
          ],
          maxOutputTokens: 8_192,
        },);
        expect(decision.ranked.map((entry,) => entry.canonicalSlug,),).toEqual([
          'p/big',
          'p/small',
        ],);
        expect(decision.reason.includes('context window',),).toBe(true,);
      },
    },),
    it({
      name: 'breaks window ties by name then slug',
      fn: async () => {
        const zeta = scopedModelFixture({
          slug: 'p/zeta',
          name: 'Zeta',
          contextWindow: 100_000,
        },);
        const alphaSecond = scopedModelFixture({
          slug: 'p/b',
          name: 'Alpha',
          contextWindow: 100_000,
        },);
        const alphaFirst = scopedModelFixture({
          slug: 'p/a',
          name: 'Alpha',
          contextWindow: 100_000,
        },);
        const decision = dshRankDefaultCandidates({
          candidates: [
            candidateFixture(zeta,),
            candidateFixture(alphaSecond,),
            candidateFixture(alphaFirst,),
          ],
          maxOutputTokens: 8_192,
        },);
        expect(decision.ranked.map((entry,) => entry.canonicalSlug,),).toEqual([
          'p/a',
          'p/b',
          'p/zeta',
        ],);
      },
    },),
    it({
      name: 'reports the winning context window in the reason',
      fn: async () => {
        const one = scopedModelFixture({
          slug: 'p/one',
          name: 'One',
          contextWindow: 200_000,
        },);
        const two = scopedModelFixture({
          slug: 'p/two',
          name: 'Two',
          contextWindow: 32_000,
        },);
        const decision = dshRankDefaultCandidates({
          candidates: [
            candidateFixture(one,),
            candidateFixture(two,),
          ],
          maxOutputTokens: 8_192,
        },);
        expect(decision.reason.includes('200000',),).toBe(true,);
        expect(decision.ranked[0]?.canonicalSlug,).toBe('p/one',);
      },
    },),
    it({
      name: 'keeps losing candidates out of the reason',
      fn: async () => {
        const one = scopedModelFixture({
          slug: 'p/one',
          name: 'One',
          contextWindow: 200_000,
        },);
        const two = scopedModelFixture({
          slug: 'p/two',
          name: 'Two',
          contextWindow: 32_000,
        },);
        const three = scopedModelFixture({
          slug: 'p/three',
          name: 'Three',
          contextWindow: 16_000,
        },);
        const decision = dshRankDefaultCandidates({
          candidates: [
            candidateFixture(one,),
            candidateFixture(two,),
            candidateFixture(three,),
          ],
          maxOutputTokens: 8_192,
        },);
        expect(decision.reason.includes('p/one',),).toBe(true,);
        expect(decision.reason.includes('p/two',),).toBe(false,);
        expect(decision.reason.includes('p/three',),).toBe(false,);
        expect(decision.reason.includes('32000',),).toBe(false,);
        expect(decision.reason.includes('16000',),).toBe(false,);
      },
    },),
    it({
      name: 'states the empty-candidate fallback in the reason',
      fn: async () => {
        const decision = dshRankDefaultCandidates({
          candidates: [],
          maxOutputTokens: 8_192,
        },);
        expect(decision.ranked,).toEqual([],);
        expect(decision.reason.includes('no eligible candidates',),).toBe(true,);
      },
    },),
  ],
},);
