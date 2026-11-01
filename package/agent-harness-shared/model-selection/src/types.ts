/**
 Structural model-selection types shared by pi plugins.
 
 @module
 */

import type { ReadonlyDeep, } from 'type-fest';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Core model types

/**
 Minimum model identity needed for slug and exact-match helpers.
 */
export type ModelIdentity = {
  /**
   Provider identifier, e.g. `anthropic`.
   */
  readonly provider: string;
  /**
   Model identifier, e.g. `claude-sonnet-4-5-20251001`.
   */
  readonly id: string;
  /**
   Human-readable model display name.
   */
  readonly name: string;
};

/**
 Per-token price metadata used by cost and budget helpers.
 */
export type ModelTokenPricing = {
  /**
   Input-token price per million tokens.
   */
  readonly input: number;
  /**
   Output-token price per million tokens.
   */
  readonly output: number;
  /**
   Cache-read price per million tokens.
   */
  readonly cacheRead: number;
  /**
   Cache-write price per million tokens.
   */
  readonly cacheWrite: number;
};

/**
 Model identity plus pricing and token limits.
 */
export type ModelPricing = ModelIdentity & {
  /**
   Per-token pricing metadata.
   */
  readonly cost: ModelTokenPricing;
  /**
   Maximum input-token context budget.
   */
  readonly contextWindow: number;
  /**
   Maximum output tokens for one response.
   */
  readonly maxTokens: number;
};

/**
 Full pi-like readonly model shape for callers that need complete records.
 */
export type ReadonlyModel = ModelPricing & {
  /**
   API protocol identifier.
   */
  readonly api: string;
  /**
   Provider base URL.
   */
  readonly baseUrl: string;
  /**
   Whether the model can emit reasoning content.
   */
  readonly reasoning: boolean;
  /**
   Optional thinking-level mapping from pi to provider values.
   */
  readonly thinkingLevelMap?: ReadonlyDeep<Record<string, unknown>>;
  /**
   Supported input modalities.
   */
  readonly input: readonly ('text' | 'image')[];
  /**
   Optional provider request headers.
   */
  readonly headers?: ReadonlyDeep<Record<string, string>>;
  /**
   Optional provider-specific compatibility metadata.
   */
  readonly compat?: unknown;
};

//endregion Core model types

//region Scope types

/**
 Where an effective scoped model set came from.
 */
export type ScopeSource = 'live' | 'argv' | 'settings' | 'available';

/**
 Thinking level suffix carried by pi model scope patterns.
 */
export type ScopedThinkingLevel = 'off' | 'minimal' | 'low' | 'medium' | 'high' | 'xhigh';

/**
 Model entry selected by model scope resolution.
 */
export type ScopedModel<TModel extends ModelIdentity = ReadonlyModel,> = {
  /**
   Original model object supplied by the caller.
   */
  readonly model: TModel;
  /**
   Canonical `provider/modelId` slug.
   */
  readonly canonicalSlug: string;
  /**
   Thinking level suffix from a scope pattern, when present.
   */
  readonly thinkingLevel?: ScopedThinkingLevel;
};

/**
 Effective scoped model set for a tool call, command, or status query.
 */
export type EffectiveModelScope<TModel extends ModelIdentity = ReadonlyModel,> = {
  /**
   Source that produced this scoped model set.
   */
  readonly source: ScopeSource;
  /**
   Models available to the consumer.
   */
  readonly entries: readonly ScopedModel<TModel>[];
  /**
   Patterns used for argv or settings scope reconstruction.
   */
  readonly patterns?: readonly string[];
};

//endregion Scope types

//region Selection and cost types

/**
 Cost inputs computed for one model during default selection.
 */
export type ModelCostScore = {
  /**
   Canonical model slug.
   */
  readonly slug: string;
  /**
   Input tokens estimated for the request.
   */
  readonly inputTokens: number;
  /**
   Output-token budget used for ranking.
   */
  readonly maxOutputTokens: number;
  /**
   Expected cost score used for ordering.
   */
  readonly expectedCost: number;
  /**
   Input-token price from model metadata.
   */
  readonly inputCost: number;
  /**
   Output-token price from model metadata.
   */
  readonly outputCost: number;
  /**
   Model context window used as tie-breaker.
   */
  readonly contextWindow: number;
};

/**
 Default model selection result.
 */
export type DefaultModelSelection<TModel extends ModelPricing = ReadonlyModel,> = {
  /**
   Selected scoped model.
   */
  readonly selected: ScopedModel<TModel>;
  /**
   Sorted scores, best first.
   */
  readonly ranking: readonly ModelCostScore[];
  /**
   Human-readable reason for the top rank.
   */
  readonly reason: string;
};

/**
 Explicit or default model selection result.
 */
export type ModelSelection<TModel extends ModelIdentity = ReadonlyModel,> = {
  /**
   Selected scoped model.
   */
  readonly selected: ScopedModel<TModel>;
  /**
   Requested slug, when user supplied one.
   */
  readonly requestedSlug?: string;
  /**
   Default-selection metadata, present for empty params.
   */
  readonly defaultSelection?: DefaultModelSelection<TModel & ModelPricing>;
};

//endregion Selection and cost types

//region Budget types

/**
 Authentication details for a budget model.
 */
export type BudgetModelAuth = {
  /**
   API key for the model provider.
   */
  readonly apiKey?: string;
  /* oxlint-disable no-restricted-syntax/no-nullish-union -- mirrors Pi AI 0.87 `ProviderHeaders = Record<string, string | null>` returned by ModelRegistry auth resolution: `null` is not absence but an instruction to suppress the provider's default header of that name, so the value must reach the provider unchanged. */
  /**
   Custom headers for the request.

   Mirrors Pi AI `ProviderHeaders`:
   `null` suppresses the provider's default header of that name,
   so it must reach the provider unchanged rather than be dropped.
   */
  readonly headers?: Readonly<Record<string, string | null>>;
  /* oxlint-enable no-restricted-syntax/no-nullish-union */
};

/**
 A selected budget model with auth credentials.
 */
export type BudgetModel<TModel extends ModelIdentity = ReadonlyModel,> = {
  /**
   Selected model.
   */
  readonly model: TModel;
  /**
   Authentication details for the model.
   */
  readonly auth: BudgetModelAuth;
};

/**
 Strategy for finding a budget model.
 */
export type BudgetModelStrategy = 'same-provider' | 'any-provider';

/**
 Pinned-model override for budget-model selection.
 */
export type BudgetModelOverride =
  | string
    | {
    /**
     Pinned provider/model slug.
     */
      readonly model: string;
    /**
     Inline authentication, bypassing registry lookup.
     */
      readonly auth: BudgetModelAuth;
    };

/**
 Candidate metadata surfaced in budget-model errors.
 */
export type BudgetModelCandidate = {
  /**
   Provider slug.
   */
  readonly provider: string;
  /**
   Model id.
   */
  readonly modelId: string;
  /**
   Name-heuristic speed score.
   */
  readonly speedScore: number;
  /**
   Input-token price per million tokens.
   */
  readonly costInput: number;
  /**
   Output-token price per million tokens.
   */
  readonly costOutput: number;
  /**
   Whether host registry reports usable auth.
   */
  readonly hasApiKey: boolean;
};

/**
 Sentinel returned by a {@link ResolveBudgetAuth} implementation (and by
 {@link ResolveBudgetOverrideAuth}) when no usable auth exists for a model.
 A `unique symbol`; budget selection narrows with `=== NO_AUTH`. Shared
 across the package boundary so host auth resolvers return the same identity
 the selectors check. Lives here, the only internal-import-free module after
 `maybe.ts` was removed, so {@link ResolveBudgetAuth} can reference both
 `typeof NO_AUTH` and {@link BudgetModelAuth} without a module cycle.
 */
export const NO_AUTH: unique symbol = Symbol('model selection budget auth credentials absent',);

/**
 Auth callback used by budget selection.
 */
export type ResolveBudgetAuth<TModel extends ModelIdentity = ReadonlyModel,> = (
  options: { readonly model: TModel; },
) => Promise<BudgetModelAuth | typeof NO_AUTH>;

/**
 Options for shared budget-model strategy selection.
 */
export type BudgetModelSelectionOptions<TModel extends ModelPricing = ReadonlyModel,> = ForeignBorrowed<{
  /**
   Active model whose provider anchors same-provider selection.
   */
  readonly activeModel: TModel;
  /**
   Registry model list.
   */
  readonly allModels: readonly TModel[];
  /**
   Search strategy.
   */
  readonly strategy: BudgetModelStrategy;
  /**
   Number of major-version families to search, or zero for all.
   */
  readonly majorVersions: number;
  /**
   Host auth resolver.
   */
  readonly resolveAuth: ResolveBudgetAuth<TModel>;
  /**
   Host auth availability predicate for error reports.
   */
  readonly hasConfiguredAuth: (options: { readonly model: TModel; }) => boolean;
}>;

//endregion Budget types
