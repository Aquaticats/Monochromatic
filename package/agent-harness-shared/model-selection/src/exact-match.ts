/**
 Exact model reference matching for scope reconstruction.
 
 @module
 */

import { canonicalSlug, } from './model-id.ts';
import type { ModelIdentity, } from './types.ts';

/**
 Sentinel returned by {@link findExactModelReferenceMatch} (and internal
 {@link matchProviderModelReference}) when no unambiguous exact model match exists:
 empty reference, ambiguous matches, or no match. A `unique symbol`; callers
 narrow with `=== NO_EXACT_MATCH`. Exported because `pattern-match`'s
 `tryMatchModel` consumes it across the module seam.
 */
export const NO_EXACT_MATCH: unique symbol = Symbol('model-selection/no-exact-match',);

//region Public API

/**
 Find an exact canonical or bare model reference match.
 
 @param modelReference - user or config model reference
 
 @param availableModels - models with configured auth
 
 @returns matched model, if unambiguous
 
 @example
 ```typescript
 findExactModelReferenceMatch({ modelReference: 'anthropic/claude', availableModels });
 ```
 */
export function findExactModelReferenceMatch<TModel extends ModelIdentity,>(
  {
    modelReference,
    availableModels,
  }: {
    readonly modelReference: string;
    readonly availableModels: readonly TModel[];
  },
): TModel | typeof NO_EXACT_MATCH {
  /**
   Trimmed user reference.
   */
  const trimmedReference = modelReference.trim();
  if (trimmedReference === '')
    return NO_EXACT_MATCH;

  /**
   Lowercase reference for pi-compatible exact matching.
   */
  const normalizedReference = trimmedReference.toLowerCase();
  /**
   Canonical slug matches.
   */
  const canonicalMatches = availableModels.filter(function matchesCanonical(model,) {
    return canonicalSlug(model,)
      .toLowerCase()
      === normalizedReference;
  },);
  if (canonicalMatches.length
    === 1)
    return canonicalMatches[0] ?? NO_EXACT_MATCH;
  if (canonicalMatches.length
    > 1)
    return NO_EXACT_MATCH;

  /**
   Match provider/model form before bare id.
   */
  const providerMatch = matchProviderModelReference({
    trimmedReference,
    availableModels,
  },);
  if (providerMatch !== NO_EXACT_MATCH)
    return providerMatch;

  /**
   Bare id matches.
   */
  const idMatches = availableModels.filter(function matchesId(model,) {
    return model.id
      .toLowerCase()
      === normalizedReference;
  },);
  return idMatches.length
    === 1 ? (idMatches[0] ?? NO_EXACT_MATCH) : NO_EXACT_MATCH;
}

//endregion Public API

//region Internal helpers

/**
 Match provider/model exact references.
 
 @param trimmedReference - trimmed model reference
 
 @param availableModels - models with configured auth
 
 @returns matched model, if unambiguous
 */
function matchProviderModelReference<TModel extends ModelIdentity,>(
  {
    trimmedReference,
    availableModels,
  }: {
    readonly trimmedReference: string;
    readonly availableModels: readonly TModel[];
  },
): TModel | typeof NO_EXACT_MATCH {
  /**
   Slash index used to parse provider/model references.
   */
  const slashIndex = trimmedReference.indexOf('/',);
  if (slashIndex === (-1))
    return NO_EXACT_MATCH;

  /**
   Provider segment from a canonical reference.
   */
  const provider = trimmedReference
    .slice(
    0,
    slashIndex,
    )
    .trim();
  /**
   Model id segment from a canonical reference.
   */
  const modelId = trimmedReference
    .slice(slashIndex + 1,)
    .trim();
  if ((provider === '') || (modelId === ''))
    return NO_EXACT_MATCH;

  /**
   Exact provider and model id matches.
   */
  const providerMatches = availableModels.filter(function matchesProvider(model,) {
    return (model.provider
      .toLowerCase()
      === provider
        .toLowerCase())
      && (model.id
        .toLowerCase()
        === modelId
          .toLowerCase());
  },);
  if (providerMatches.length
    === 1)
    return providerMatches[0] ?? NO_EXACT_MATCH;
  return NO_EXACT_MATCH;
}

//endregion Internal helpers
