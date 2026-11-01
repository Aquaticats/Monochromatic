/**
 Advisor explicit-model resolution boundary.
 
 @module
 */

import {
  type ModelIdentity,
  type ModelRegistryLookup,
  type ModelSelection,
  resolveRequestedModel,
  type ScopedModel,
} from '@monochromatic-dev/agent-harness-shared-model-selection/ts';
import type {
  ForeignBorrowed,
  ForeignHostCapability,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  AdvisorReadonlyModel,
  EffectiveModelScope,
} from './types.ts';

/**
 Resolve explicit Advisor model while isolating caller-owned scoped entries from shared matching internals.
 
 @param scope - effective Advisor model scope
 
 @param requestedSlug - explicit model slug
 
 @param modelRegistry - pi model registry used for global lookup
 
 @returns explicit Advisor model selection
 
 @throws when requested slug is empty, ambiguous, out of scope, unknown, or no longer in supplied scope
 
 @example
 ```typescript
 resolveAdvisorRequestedModel({ scope, requestedSlug: 'provider/model', modelRegistry });
 ```
 */
export function resolveAdvisorRequestedModel(
  {
    scope,
    requestedSlug,
    modelRegistry,
  }: ForeignBorrowed<{
    readonly scope: EffectiveModelScope;
    readonly requestedSlug: string;
    readonly modelRegistry: ForeignHostCapability<ModelRegistryLookup<AdvisorReadonlyModel>>;
  }>,
): ModelSelection {
  /**
   Locally-owned identity-only entries safe for shared matching internals.
   */
  const detachedEntries: ScopedModel<ModelIdentity>[] = [];
  for (const entry of scope.entries) {
    detachedEntries[detachedEntries.length] = {
      canonicalSlug: entry.canonicalSlug,
      model: {
        provider: entry.model
          .provider,
        id: entry.model
          .id,
        name: entry.model
          .name,
      },
      ...(entry.thinkingLevel === undefined
        ? {}
        : { thinkingLevel: entry.thinkingLevel, }),
    };
  }

  /**
   Locally-owned scope patterns safe for shared matching internals.
   */
  const detachedPatterns: string[] = [];
  if (scope.patterns !== undefined) {
    for (const pattern of scope.patterns)
      detachedPatterns[detachedPatterns.length] = pattern;
  }

  /**
   Explicit identity selection from detached scope.
   */
  const detachedSelection = resolveRequestedModel({
    scope: {
      source: scope.source,
      entries: detachedEntries,
      ...(scope.patterns === undefined
        ? {}
        : { patterns: detachedPatterns, }),
    },
    requestedSlug,
    modelRegistry,
    errorPrefix: 'advisor',
  },);
  /**
   Canonical slug selected from detached scope.
   */
  const selectedSlug = detachedSelection
    .selected
    .canonicalSlug;
  for (const entry of scope.entries) {
    if (entry.canonicalSlug
      === selectedSlug) {
      return {
        selected: entry,
        requestedSlug: requestedSlug.trim(),
      };
    }
  }

  throw new Error(
    `advisor: selected scoped model "${selectedSlug}" disappeared during explicit resolution`,
  );
}
