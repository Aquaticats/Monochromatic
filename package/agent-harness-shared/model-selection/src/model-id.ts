/**
 Model id, slug, and explicit-selection helpers.
 
 @module
 */

import type {
  EffectiveModelScope,
  ModelIdentity,
  ModelSelection,
  ScopedModel,
} from './types.ts';

/**
 Sentinel returned by {@link parseProviderModelSlug} when a slug is not a
 `provider/model` pair (no slash, or empty provider or model id). A
 `unique symbol`; callers narrow with `=== MALFORMED_SLUG`. Exported because
 `budget-override` consumes it across the module seam.
 */
export const MALFORMED_SLUG: unique symbol = Symbol('model selection canonical slug malformed',);

//region Types

/**
 Parsed `provider/model` slug.
 */
export type ProviderModelSlug = {
  /**
   Provider segment before the first slash.
   */
  readonly provider: string;
  /**
   Model id segment after the first slash.
   */
  readonly modelId: string;
};

/**
 Global model registry shape needed by explicit selection.
 */
export type ModelRegistryLookup<TModel extends ModelIdentity = ModelIdentity,> = {
  /**
   Return every registry model.
   */
  getAll: () => readonly TModel[];
};

/**
 Options for resolving an explicit model slug.
 */
export type ResolveRequestedModelOptions<TModel extends ModelIdentity = ModelIdentity,> = {
  /**
   Effective scoped model set.
   */
  readonly scope: EffectiveModelScope<TModel>;
  /**
   User supplied slug.
   */
  readonly requestedSlug: string;
  /**
   Global model registry used to distinguish out-of-scope slugs.
   */
  readonly modelRegistry: ModelRegistryLookup<TModel>;
  /**
   Error prefix used by the consuming extension.
   */
  readonly errorPrefix?: string;
};

//endregion Types

//region Public API

/**
 Return canonical slug for a model.
 
 @param model - model identity
 
 @returns canonical `provider/modelId` slug
 
 @example
 ```typescript
 canonicalSlug({ provider: 'openai', id: 'gpt-5.5', name: 'GPT' });
 ```
 */
export function canonicalSlug(
  model: Pick<ModelIdentity, 'provider' | 'id'>,
): string {
  return `${model.provider}/${model.id}`;
}

/**
 Parse a canonical `provider/model` slug.
 
 @param slug - candidate provider/model slug
 
 @returns parsed slug, or {@link MALFORMED_SLUG} when the slug is malformed
 
 @example
 ```typescript
 parseProviderModelSlug('openai/gpt-5.5');
 ```
 */
export function parseProviderModelSlug(
  slug: string,
): ProviderModelSlug | typeof MALFORMED_SLUG {
  /**
   Slash index between provider and model id.
   */
  const slashIndex = slug.indexOf('/',);
  if (slashIndex === (-1))
    return MALFORMED_SLUG;

  /**
   Provider segment before slash.
   */
  const provider = slug.slice(
    0,
    slashIndex,
  )
    .trim();
  /**
   Model id segment after slash.
   */
  const modelId = slug.slice(slashIndex + 1,)
    .trim();
  if ((provider === '') || (modelId === ''))
    return MALFORMED_SLUG;

  return {
    provider,
    modelId,
  };
}

/**
 Return the final slash-delimited segment from a model id.
 
 @param modelId - model id to inspect
 
 @returns final slash-delimited model id segment
 
 @example
 ```typescript
 getModelIdLeaf({ modelId: 'openai/gpt-5.5' });
 ```
 */
export function getModelIdLeaf(
  {
    modelId,
  }: {
    readonly modelId: string;
  },
): string {
  /**
   Index after final slash, or zero when no slash exists.
   */
  const leafStartIndex = modelId.lastIndexOf('/',)
    + 1;
  return modelId.slice(leafStartIndex,);
}

/**
 Format allowed canonical slugs for errors and status output.
 
 @param scope - effective model scope
 
 @returns comma-separated canonical slugs or `none`
 
 @example
 ```typescript
 allowedSlugs(scope);
 ```
 */
export function allowedSlugs<TModel extends ModelIdentity,>(
  scope: EffectiveModelScope<TModel>,
): string {
  return scope.entries
    .length
    === 0
    ? 'none'
    : scope
      .entries
      .map(function mapEntry(entry,) {
        return entry.canonicalSlug;
      },)
      .join(', ',);
}

/**
 Resolve an explicit model slug inside an effective scoped model set.
 
 @param scope - effective scoped model set
 
 @param requestedSlug - user supplied slug
 
 @param modelRegistry - global model registry used for out-of-scope detection
 
 @param errorPrefix - optional error prefix used by the consuming extension
 
 @returns selected scoped model
 
 @throws when slug is absent, ambiguous, unknown, or outside scope
 
 @example
 ```typescript
 resolveRequestedModel({ scope, requestedSlug: 'openai/gpt-5.5', modelRegistry });
 ```
 */
export function resolveRequestedModel<TModel extends ModelIdentity,>(
  {
    scope,
    requestedSlug: rawRequestedSlug,
    modelRegistry,
    errorPrefix: rawErrorPrefix,
  }: ResolveRequestedModelOptions<TModel>,
): ModelSelection<TModel> {
  /**
   Error message prefix for the consuming extension.
   */
  const errorPrefix = rawErrorPrefix
    ?? 'model selection';
  /**
   Trimmed requested model slug.
   */
  const requestedSlug = rawRequestedSlug
    .trim();
  if (requestedSlug === '')
    throw new Error(`${errorPrefix}: model slug must not be empty`,);

  /**
   Matching scoped model candidates.
   */
  const scopedMatches = findScopedSlugMatches({
    scope,
    requestedSlug,
  },);
  if (scopedMatches.length
    === 1) {
    /**
     Only scoped match after length guard.
     */
    const [selected,] = scopedMatches;
    if (selected === undefined)
      throw new Error(`${errorPrefix}: internal slug resolution failed`,);
    return {
      selected,
      requestedSlug,
    };
  }
  if (scopedMatches.length
    > 1) {
    throw new Error(
      `${errorPrefix}: model slug "${requestedSlug}" is ambiguous in scoped models. Matching scoped slugs: ${
        scopedMatches
          .map(function mapMatch(match,) {
            return match.canonicalSlug;
          },)
          .join(', ',)
      }`,
    );
  }

  if (slugExistsGlobally({
    requestedSlug,
    models: modelRegistry
      .getAll(),
  },)) {
    throw new Error(
      `${errorPrefix}: model slug "${requestedSlug}" is not in scoped models. Allowed scoped slugs: ${
        allowedSlugs(
          scope,
        )
      }`,
    );
  }

  throw new Error(
    `${errorPrefix}: model slug "${requestedSlug}" was not found in scoped models. Allowed scoped slugs: ${
      allowedSlugs(
        scope,
      )
    }`,
  );
}

//endregion Public API

//region Matching

/**
 Find scoped matches for canonical, bare id, or unique model name slug forms.
 
 @param scope - effective model scope
 
 @param requestedSlug - requested model slug
 
 @returns matching scoped models
 */
function findScopedSlugMatches<TModel extends ModelIdentity,>(
  {
    scope,
    requestedSlug,
  }: {
    readonly scope: EffectiveModelScope<TModel>;
    readonly requestedSlug: string;
  },
): ScopedModel<TModel>[] {
  if (requestedSlug.includes('/',)) {
    return scope.entries
      .filter(function matchCanonical(entry,) {
        return entry.canonicalSlug
          === requestedSlug;
      },);
  }

  /**
   Bare id matches inside scope.
   */
  const idMatches = scope.entries
    .filter(function matchId(entry,) {
      return entry.model
        .id
        === requestedSlug;
    },);
  /**
   Model display-name matches inside scope.
   */
  const nameMatches = scope.entries
    .filter(function matchName(entry,) {
      return entry.model
        .name
        === requestedSlug;
    },);
  /**
   Unique matches across both bare forms.
   */
  const uniqueMatches = uniqueScopedModels([
    ...idMatches,
    ...nameMatches,
  ],);
  return uniqueMatches;
}

/**
 Check whether a slug exists anywhere in the global registry.
 
 @param requestedSlug - requested model slug
 
 @param models - global model registry entries
 
 @returns whether slug exists globally
 */
function slugExistsGlobally(
  {
    requestedSlug,
    models,
  }: {
    readonly requestedSlug: string;
    readonly models: readonly ModelIdentity[];
  },
): boolean {
  if (requestedSlug.includes('/',)) {
    return models.some(function matchesCanonical(model,) {
      return canonicalSlug(model,)
        === requestedSlug;
    },);
  }

  return models.some(function matchesBareModel(model,) {
    return (model.id
      === requestedSlug) || (model.name
        === requestedSlug);
  },);
}

/**
 Deduplicate scoped model matches by canonical slug.
 
 @param models - scoped models to deduplicate
 
 @returns unique scoped models
 */
function uniqueScopedModels<TModel extends ModelIdentity,>(
  models: readonly ScopedModel<TModel>[],
): ScopedModel<TModel>[] {
  /**
   Locally-owned accumulator built without mutating input.
   */
  const result: ScopedModel<TModel>[] = [];
  for (const model of models) {
    if (result.some(function alreadyAdded(entry,) {
      return entry.canonicalSlug
        === model
          .canonicalSlug;
    },))
      continue;
    result.push(model,);
  }
  return result;
}

//endregion Matching
