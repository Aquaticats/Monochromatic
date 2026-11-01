/**
 Default-ranking seam for Advisor model selection.

 Harness adapters may supply a ranking policy (their comparator or
 pre-ordered candidates computed outside the core); without one, selection
 ranks candidates by expected call cost exactly as pi does.

 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { selectDefaultModelFromContextEstimates, } from '@monochromatic-dev/agent-harness-shared-model-selection/ts';
import type {
  AdvisorDefaultRanking,
  AdvisorDefaultSelection,
  AdvisorRankingCandidate,
  EffectiveModelScope,
  RankAdvisorDefaultCandidates,
  ScopedAdvisorModel,
} from './types.ts';

//region Default ranking seam

/**
 Assert that an external ranking policy returned a permutation of its inputs.

 @param inputSlugs - canonical slugs of eligible candidates handed to the policy

 @param ranked - policy-ranked scoped models

 @throws when ranked candidates are missing, duplicated, unknown, or differently counted

 @example
 ```typescript
 assertCompleteDefaultRanking({ inputSlugs: ['a/b'], ranked: [entry] });
 ```
 */
function assertCompleteDefaultRanking(
  {
    inputSlugs,
    ranked,
  }: ForeignBorrowed<Readonly<{
    readonly inputSlugs: readonly string[];
    readonly ranked: readonly ScopedAdvisorModel[];
  }>>,
): void {
  /**
   Occurrence count for each ranked canonical slug.
   */
  const rankedCounts = new Map<string, number>();
  for (const entry of ranked) {
    /**
     Prior occurrence count for this ranked slug.
     */
    const priorCount = rankedCounts
      .get(entry
        .canonicalSlug) ?? 0;
    rankedCounts.set(
      entry
        .canonicalSlug,
      priorCount + 1,
    );
  }
  /**
   Count of policy-ranked candidates.
   */
  const rankedCount = ranked.length;
  /**
   Count of eligible candidates handed to the policy.
   */
  const inputCount = inputSlugs.length;
  if (rankedCount
    !== inputCount)
    throw new Error(
      `advisor: default ranking policy returned ${String(rankedCount,)} ranked candidates for ${String(inputCount,)} eligible candidates`,
    );
  for (const slug of inputSlugs) {
    if (rankedCounts
      .get(slug)
      !== 1)
      throw new Error(
        `advisor: default ranking policy must return each eligible candidate exactly once; missing or duplicated: ${slug}`,
      );
  }
}

/**
 Rank default-selection candidates through the supplied policy or the built-in cost ranking.

 The built-in path keeps pi's expected-cost ranking bit-identical and carries
 full cost scores as ranking evidence. External policies (a harness adapter's
 comparator or pre-ordered candidates) decide the order outside the core and
 return it pre-ranked with a human-readable reason.

 @param scope - eligible scoped models to rank

 @param estimatedInputTokensBySlug - input token estimate keyed by canonical slug

 @param maxOutputTokens - requested output token budget

 @param rankDefaultCandidates - optional external ranking policy

 @returns default selection with ranked evidence, best first

 @throws when the scope is empty, an estimate is missing, or the policy returns an incomplete ranking

 @example
 ```typescript
 rankDefaultAdvisorCandidates({ scope, estimatedInputTokensBySlug, maxOutputTokens });
 ```
 */
export function rankDefaultAdvisorCandidates(
  {
    scope,
    estimatedInputTokensBySlug,
    maxOutputTokens,
    rankDefaultCandidates,
  }: ForeignBorrowed<Readonly<{
    readonly scope: EffectiveModelScope;
    readonly estimatedInputTokensBySlug: ReadonlyMap<string, number>;
    readonly maxOutputTokens: number;
    readonly rankDefaultCandidates?: RankAdvisorDefaultCandidates;
  }>>,
): AdvisorDefaultSelection {
  if (rankDefaultCandidates
    === undefined) {
    /**
     Built-in expected-cost ranking (pi behavior), unchanged.
     */
    const costSelection = selectDefaultModelFromContextEstimates({
      scope,
      estimatedInputTokensBySlug,
      maxOutputTokens,
    },);
    return {
      selected: costSelection.selected,
      ranking: costSelection.ranking,
      reason: costSelection.reason,
    };
  }
  /**
   Eligible candidates with their input-token estimates, in scope order.
   */
  const candidates: AdvisorRankingCandidate[] = [];
  for (const entry of scope.entries) {
    /**
     Input token estimate recorded for this scoped model.
     */
    const estimatedInputTokens = estimatedInputTokensBySlug
      .get(entry
        .canonicalSlug);
    if (estimatedInputTokens
      === undefined)
      throw new Error(
        `advisor: missing input-token estimate for ${entry
          .canonicalSlug}`,
      );
    candidates.push({
      scopedModel: entry,
      estimatedInputTokens,
    },);
  }
  /**
   Externally ranked candidates plus the policy's ranking reason.
   */
  const ranking: AdvisorDefaultRanking = rankDefaultCandidates({
    candidates,
    maxOutputTokens,
  },);
  assertCompleteDefaultRanking({
    inputSlugs: candidates
      .map(function candidateSlug(candidate: AdvisorRankingCandidate,): string {
        return candidate.scopedModel
          .canonicalSlug;
      },),
    ranked: ranking.ranked,
  },);
  /**
   Policy's top-ranked scoped model.
   */
  const [firstRanked,] = ranking.ranked;
  if (firstRanked
    === undefined)
    throw new Error('advisor: default ranking policy returned no candidates',);
  return {
    selected: firstRanked,
    ranking: ranking.ranked
      .map(function rankedSlug(entry: ScopedAdvisorModel,): { readonly slug: string } {
        return { slug: entry.canonicalSlug, };
      },),
    reason: ranking.reason,
  };
}

//endregion Default ranking seam
