/**
 Cost ranking for default model selection.
 
 @module
 */

import type {
  DefaultModelSelection,
  EffectiveModelScope,
  ModelCostScore,
  ModelPricing,
  ScopedModel,
} from './types.ts';

//region Public API

/**
 Options for default model selection.
 */
export type SelectDefaultModelOptions<TModel extends ModelPricing = ModelPricing,> = {
  /**
   Effective scoped model set.
   */
  readonly scope: EffectiveModelScope<TModel>;
  /**
   Estimated input tokens for this request.
   */
  readonly estimatedInputTokens: number;
  /**
   Maximum output tokens requested.
   */
  readonly maxOutputTokens: number;
};

/**
 Options for default model selection with per-model context estimates.
 */
export type SelectDefaultModelFromContextEstimatesOptions<
  TModel extends ModelPricing = ModelPricing,
  > = {
  /**
   Effective scoped model set.
   */
    readonly scope: EffectiveModelScope<TModel>;
  /**
   Estimated input tokens keyed by canonical scoped model slug.
   */
    readonly estimatedInputTokensBySlug: ReadonlyMap<string, number>;
  /**
   Maximum output tokens requested.
   */
    readonly maxOutputTokens: number;
  };

/**
 Options for scoring one scoped model.
 */
export type ScoreModelOptions<TModel extends ModelPricing = ModelPricing,> = {
  /**
   Scoped model entry.
   */
  readonly entry: ScopedModel<TModel>;
  /**
   Estimated request input tokens.
   */
  readonly estimatedInputTokens: number;
  /**
   Requested output token budget.
   */
  readonly maxOutputTokens: number;
};

/**
 Options for comparing cost scores.
 */
export type CompareCostScoresOptions = {
  /**
   First score.
   */
  readonly left: ModelCostScore;
  /**
   Second score.
   */
  readonly right: ModelCostScore;
};

/**
 Options for building a cost ranking.
 */
export type BuildCostRankingOptions<TModel extends ModelPricing = ModelPricing,> = {
  /**
   Effective scoped model set.
   */
  readonly scope: EffectiveModelScope<TModel>;
  /**
   Estimated input tokens keyed by canonical scoped model slug.
   */
  readonly estimatedInputTokensBySlug: ReadonlyMap<string, number>;
  /**
   Requested output token budget.
   */
  readonly maxOutputTokens: number;
  /**
   Error prefix used by thrown messages.
   */
  readonly errorPrefix?: string;
};

/**
 Select the most expensive scoped model by expected call cost.
 
 @param options - scope and request-size inputs
 
 @returns selected model and sorted cost ranking
 
 @throws when scope is empty
 
 @example
 ```typescript
 selectDefaultModel({ scope, estimatedInputTokens: 1000, maxOutputTokens: 4096 });
 ```
 */
export function selectDefaultModel<TModel extends ModelPricing,>(
  options: SelectDefaultModelOptions<TModel>,
): DefaultModelSelection<TModel> {
  /**
   Shared token estimate applied to every scoped model.
   */
  const estimatedInputTokensBySlug = new Map(
    options
      .scope
      .entries
      .map(function mapEntry(entry,) {
        return [
          entry.canonicalSlug,
          options.estimatedInputTokens,
        ] as const;
      },),
  );
  return selectDefaultModelFromContextEstimates({
    scope: options.scope,
    estimatedInputTokensBySlug,
    maxOutputTokens: options.maxOutputTokens,
  },);
}

/**
 Select most expensive scoped model using each candidate's context estimate.
 
 @param options - scope and per-candidate request-size inputs
 
 @returns selected model and sorted cost ranking
 
 @throws when scope is empty or a candidate has no estimate
 
 @example
 ```typescript
 selectDefaultModelFromContextEstimates({ scope, estimatedInputTokensBySlug, maxOutputTokens });
 ```
 */
export function selectDefaultModelFromContextEstimates<TModel extends ModelPricing,>(
  options: SelectDefaultModelFromContextEstimatesOptions<TModel>,
): DefaultModelSelection<TModel> {
  /**
   Sorted scores, highest expected cost first.
   */
  const ranking = buildCostRanking({
    scope: options.scope,
    estimatedInputTokensBySlug: options.estimatedInputTokensBySlug,
    maxOutputTokens: options.maxOutputTokens,
  },);

  /**
   Top-ranked score.
   */
  const [topScore,] = ranking;
  if (topScore === undefined)
    throw new Error('model selection: default selection failed for empty ranking',);

  /**
   Matching scoped entry for top score.
   */
  const selected = options
    .scope
    .entries
    .find(function matchesTopScore(entry,) {
      return entry.canonicalSlug
        === topScore
          .slug;
    },);
  if (selected === undefined)
    throw new Error(`model selection: selected model ${topScore.slug} disappeared from scope`,);

  return {
    selected,
    ranking,
    reason: buildSelectionReason({ score: topScore, },),
  };
}

/**
 Score a single scoped model for expected request cost.
 
 This is a new decomposition extracted during shared package migration.
 
 @param options - scoped entry and request-size inputs
 
 @returns cost score for ranking
 
 @example
 ```typescript
 scoreModel({ entry, estimatedInputTokens: 100, maxOutputTokens: 200 });
 ```
 */
export function scoreModel<TModel extends ModelPricing,>(
  options: ScoreModelOptions<TModel>,
): ModelCostScore {
  /**
   Scoped model entry and request-size inputs to score.
   */
  const {
    entry,
    estimatedInputTokens,
    maxOutputTokens,
  } = options;
  /**
   Model metadata to score.
   */
  const { model, } = entry;
  /**
   Pricing and context metadata from model.
   */
  const {
    cost,
    contextWindow,
  } = model;
  /**
   Input-token price from model metadata.
   */
  const { input: inputCost, } = cost;
  /**
   Output-token price from model metadata.
   */
  const { output: outputCost, } = cost;
  /**
   Expected request cost ignoring cache pricing.
   */
  const expectedCost = (estimatedInputTokens * inputCost)
    + (maxOutputTokens * outputCost);

  return {
    slug: entry.canonicalSlug,
    inputTokens: estimatedInputTokens,
    maxOutputTokens,
    expectedCost,
    inputCost,
    outputCost,
    contextWindow,
  };
}

/**
 Compare scores using deterministic tie-break order.
 
 This is a new decomposition extracted during shared package migration.
 
 @param options - score pair
 
 @returns negative when left ranks before right
 
 @example
 ```typescript
 compareCostScores({ left, right });
 ```
 */
export function compareCostScores(
  options: CompareCostScoresOptions,
): number {
  /**
   Scores in comparison order.
   */
  const {
    left,
    right,
  } = options;
  if (right.expectedCost !== left.expectedCost)
    return right.expectedCost - left.expectedCost;
  if (right.outputCost !== left.outputCost)
    return right.outputCost - left.outputCost;
  if (right.inputCost !== left.inputCost)
    return right.inputCost - left.inputCost;
  if (right.contextWindow !== left.contextWindow)
    return right.contextWindow - left.contextWindow;
  return left.slug
    .localeCompare(right.slug,);
}

/**
 Build sorted cost scores for a scoped model set.
 
 This is a new decomposition extracted during shared package migration.
 
 @param options - scope and per-candidate request-size inputs
 
 @returns sorted scores, highest expected cost first
 
 @throws when scope is empty or a candidate has no estimate
 
 @example
 ```typescript
 buildCostRanking({ scope, estimatedInputTokensBySlug, maxOutputTokens });
 ```
 */
export function buildCostRanking<TModel extends ModelPricing,>(
  options: BuildCostRankingOptions<TModel>,
): ModelCostScore[] {
  /**
   Error prefix for thrown messages.
   */
  const errorPrefix = options.errorPrefix
    ?? 'model selection';
  if (options
    .scope
    .entries
    .length
    === 0)
    throw new Error(`${errorPrefix}: no scoped models with configured auth`,);

  return options
    .scope
    .entries
    .map(function scoreEntry(entry,) {
      /**
       Input-token estimate for this scoped model.
       */
      const estimatedInputTokens = options.estimatedInputTokensBySlug
        .get(
        entry.canonicalSlug,
        );
      if (estimatedInputTokens === undefined) {
        throw new Error(
          `${errorPrefix}: missing input-token estimate for ${entry.canonicalSlug}`,
        );
      }
      return scoreModel({
        entry,
        estimatedInputTokens,
        maxOutputTokens: options.maxOutputTokens,
      },);
    },)
    .toSorted(function compareScoreCallback(
      left,
      right,
    ) {
      return compareCostScores({
        left,
        right,
      },);
    },);
}

//endregion Public API

//region Internal helpers

/**
 Build a concise human-readable default selection reason.
 
 @param score - winning cost score
 
 @returns human-readable selection reason
 */
function buildSelectionReason(
  {
    score,
  }: {
    readonly score: ModelCostScore;
  },
): string {
  return [
    `highest expected cost: ${score.slug} =`,
    `${score.inputTokens} input tokens * ${score.inputCost} +`,
    `${score.maxOutputTokens} output tokens * ${score.outputCost}`,
  ]
    .join(' ',);
}

//endregion Internal helpers
