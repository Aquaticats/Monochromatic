/**
 Budget-model override resolution with injected lookup and auth callbacks.
 
 @module
 */

import {
  MALFORMED_SLUG,
  parseProviderModelSlug,
} from './model-id.ts';
import { NoBudgetModelError, } from './budget-report.ts';
import {
  type BudgetModel,
  type BudgetModelAuth,
  type BudgetModelOverride,
  type ModelIdentity,
  NO_AUTH,
} from './types.ts';

/**
 Sentinel returned by a {@link FindBudgetOverrideModel} implementation when no
 registry model matches the override slug. A `unique symbol`; narrowed with
 `=== NO_OVERRIDE_MODEL`. Shared across the package boundary so host registry
 lookups return the same identity this resolver checks.
 */
export const NO_OVERRIDE_MODEL: unique symbol = Symbol('model-selection/no-override-model',);

//region Types

/**
 Model lookup callback used by override resolution.
 */
export type FindBudgetOverrideModel<TModel extends ModelIdentity = ModelIdentity,> = (
  options: {
    /**
     Provider segment from override slug.
     */
    readonly provider: string;
    /**
     Model id segment from override slug.
     */
    readonly modelId: string;
  },
) => TModel | typeof NO_OVERRIDE_MODEL;

/**
 Auth lookup callback used by override resolution.
 */
export type ResolveBudgetOverrideAuth<TModel extends ModelIdentity = ModelIdentity,> = (
  options: { readonly model: TModel; },
) => Promise<BudgetModelAuth | typeof NO_AUTH>;

/**
 Options for resolving budget-model overrides.
 */
export type ResolveBudgetModelOverrideOptions<TModel extends ModelIdentity = ModelIdentity,> = {
  /**
   Pinned model override.
   */
  readonly override: BudgetModelOverride;
  /**
   Host registry model lookup.
   */
  readonly findModel: FindBudgetOverrideModel<TModel>;
  /**
   Host auth lookup.
   */
  readonly resolveAuth: ResolveBudgetOverrideAuth<TModel>;
};

//endregion Types

//region Public API

/**
 Resolve a budget-model override and skip automatic selection.
 
 @param options - override plus injected registry lookup and auth callbacks
 
 @returns budget model with auth credentials
 
 @throws {@link NoBudgetModelError} when override is malformed, missing, or lacks auth
 
 @example
 ```typescript
 resolveBudgetModelOverride({ override: 'openai/gpt-4o-mini', findModel, resolveAuth });
 ```
 */
export async function resolveBudgetModelOverride<TModel extends ModelIdentity,>(
  options: ResolveBudgetModelOverrideOptions<TModel>,
): Promise<BudgetModel<TModel>> {
  /**
   Override value supplied by caller.
   */
  const { override, } = options;
  /**
   `provider/id` string form of override.
   */
  const modelSlug = budgetModelOverrideSlug(override,);
  /**
   Parsed provider/model slug.
   */
  const parsed = parseProviderModelSlug(modelSlug,);
  if (parsed === MALFORMED_SLUG) {
    throw new NoBudgetModelError(
      `model override "${modelSlug}" is not a provider/model slug`,
    );
  }

  /**
   Registry-resolved model record.
   */
  const model = options.findModel({
    provider: parsed.provider,
    modelId: parsed.modelId,
  },);
  if (model === NO_OVERRIDE_MODEL) {
    throw new NoBudgetModelError(
      `model override "${modelSlug}" not found in registry`,
    );
  }

  if ((typeof override) !== 'string') {
    /**
     Inline auth from structured override.
     */
    const { auth, } = override;
    return {
      model,
      auth,
    };
  }

  /**
   Registry-resolved auth for the override.
   */
  const auth = await options.resolveAuth({ model, },);
  if (auth === NO_AUTH) {
    throw new NoBudgetModelError(
      `no API key for model override "${modelSlug}"`,
    );
  }

  return {
    model,
    auth,
  };
}

//endregion Public API

//region Internal helpers

/**
 Return provider/model slug from an override value.
 
 @param override - budget-model override
 
 @returns provider/model slug
 */
function budgetModelOverrideSlug(
  override: BudgetModelOverride,
): string {
  if ((typeof override) === 'string')
    return override;
  /**
   Pinned model slug from structured override.
   */
  const { model, } = override;
  return model;
}

//endregion Internal helpers
