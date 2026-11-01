/**
 Pi-style model scope pattern resolution.
 
 @module
 */

import zeptomatch from 'zeptomatch';
import { canonicalSlug, } from './model-id.ts';
import {
  parseModelPattern,
  patternHasGlob,
  scopedModelFromModel,
  splitThinkingSuffix,
} from './pattern-match.ts';
import type {
  ModelIdentity,
  ScopedModel,
} from './types.ts';

//region Public API

/**
 Resolve pi-style model scope patterns against available models.
 
 @param patterns - pi model scope patterns
 
 @param availableModels - models with configured auth
 
 @returns deduplicated scoped models in pattern order
 
 @example
 ```typescript
 resolveModelPatterns({ patterns: ['anthropic/*'], availableModels });
 ```
 */
export function resolveModelPatterns<TModel extends ModelIdentity,>(
  {
    patterns,
    availableModels,
  }: {
    readonly patterns: readonly string[];
    readonly availableModels: readonly TModel[];
  },
): ScopedModel<TModel>[] {
  /**
   Accumulated unique matches across patterns.
   */
  const accumulator: ScopedModel<TModel>[] = [];
  for (const pattern of patterns) {
    /**
     Models matched by current pattern.
     */
    const matches = patternHasGlob(pattern,)
      ? resolveGlobPattern({
        pattern,
        availableModels,
      },)
      : resolveLiteralPattern({
        pattern,
        availableModels,
      },);
    /**
     Updated accumulator with new unique matches appended.
     */
    const next = appendUniqueMatches({
      accumulator,
      matches,
    },);
    accumulator.length = 0;
    accumulator.push(...next,);
  }
  return accumulator;
}

//endregion Public API

//region Pattern resolution

/**
 Match model glob patterns with pi's case-insensitive scope semantics.
 
 Zeptomatch has no `nocase` option, so both sides are normalized to
 lowercase before matching. The scoped model surface is intentionally
 case-insensitive for provider/model slug matching.
 
 @param pattern - pi model scope glob pattern
 
 @param candidate - canonical or bare model id candidate
 
 @returns whether candidate matches pattern ignoring case
 
 @example
 ```typescript
 modelGlobMatches({ pattern: 'OpenAI/*', candidate: 'openai/gpt-5' });
 ```
 */
function modelGlobMatches(
  {
    pattern,
    candidate,
  }: {
    readonly pattern: string;
    readonly candidate: string;
  },
): boolean {
  return zeptomatch(
    pattern.toLowerCase(),
    candidate.toLowerCase(),
  );
}

/**
 Resolve a glob model pattern.
 
 @param pattern - pi glob pattern
 
 @param availableModels - models with configured auth
 
 @returns matching scoped models
 */
function resolveGlobPattern<TModel extends ModelIdentity,>(
  {
    pattern,
    availableModels,
  }: {
    readonly pattern: string;
    readonly availableModels: readonly TModel[];
  },
): ScopedModel<TModel>[] {
  /**
   Pattern split into glob body and optional thinking suffix.
   */
  const parsed = splitThinkingSuffix(pattern,);
  return availableModels
    .filter(function matchesModel(model,) {
      /**
       Canonical model reference.
       */
      const fullId = canonicalSlug(model,);
      return modelGlobMatches({
        pattern: parsed.pattern,
        candidate: fullId,
      },)
        || modelGlobMatches({
          pattern: parsed.pattern,
          candidate: model.id,
        },);
    },)
    .map(function mapMatch(model,) {
      return scopedModelFromModel({
        model,
        ...(parsed.thinkingLevel
          === undefined
          ? {}
          : { thinkingLevel: parsed.thinkingLevel, }),
      },);
    },);
}

/**
 Resolve a non-glob model pattern.
 
 @param pattern - pi literal or fuzzy pattern
 
 @param availableModels - models with configured auth
 
 @returns matching scoped model when found
 */
function resolveLiteralPattern<TModel extends ModelIdentity,>(
  {
    pattern,
    availableModels,
  }: {
    readonly pattern: string;
    readonly availableModels: readonly TModel[];
  },
): ScopedModel<TModel>[] {
  /**
   Parsed model pattern.
   */
  const resolution = parseModelPattern({
    pattern,
    availableModels,
  },);
  return resolution.model
    === undefined
    ? []
    : [scopedModelFromModel({
      model: resolution.model,
      ...(resolution.thinkingLevel
        === undefined
        ? {}
        : { thinkingLevel: resolution.thinkingLevel, }),
    },),];
}

/**
 Append matches that have not appeared earlier in scope order.
 
 `@earendil-works/pi-ai` implements `modelsAreEqual` as provider and id equality.
 Verified in `/tmp/pi-mono-pi-ai-audit/packages/ai/src/models.ts`, lines 86 to 93,
 for pi-ai source cloned from `earendil-works/pi-mono` on 2026-05-27.
 
 @param accumulator - scoped model accumulator
 
 @param matches - matches from current pattern
 
 @returns same accumulator with new unique matches appended
 */
function appendUniqueMatches<TModel extends ModelIdentity,>(
  {
    accumulator,
    matches,
  }: {
    readonly accumulator: readonly ScopedModel<TModel>[];
    readonly matches: readonly ScopedModel<TModel>[];
  },
): ScopedModel<TModel>[] {
  /**
   Fresh accumulator seeded from input to avoid mutating caller's array.
   */
  const result: ScopedModel<TModel>[] = [...accumulator,];
  for (const match of matches) {
    if (result.some(function alreadyAdded(entry,) {
      return modelsHaveSameCanonicalIdentity({
        left: entry.model,
        right: match.model,
      },);
    },))
      continue;
    result.push(match,);
  }
  return result;
}

/**
 Compare models using pi-ai's documented equality semantics.
 
 @param left - first model identity
 
 @param right - second model identity
 
 @returns whether provider and id match exactly
 */
function modelsHaveSameCanonicalIdentity<TModel extends ModelIdentity,>(
  {
    left,
    right,
  }: {
    readonly left: TModel;
    readonly right: TModel;
  },
): boolean {
  return (left.id === right.id) && (left.provider === right.provider);
}

//endregion Pattern resolution
