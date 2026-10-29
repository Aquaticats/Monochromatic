/**
 Advisor model selection helper.
 
 @module
 */

import type { ReadonlyDeep, } from 'type-fest';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { caughtValueText, } from '@monochromatic-dev/module-caught-value/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';
import { resolveAdvisorRequestedModel, } from './advisor-requested-model.ts';
import { rankDefaultAdvisorCandidates, } from './default-ranking.ts';
import type { AdvisorModelResolution, } from './ports.ts';
import type {
  AdvisorConfig,
  AdvisorDefaultSelection,
  AdvisorModelSelection,
  CurrentMainModelIdentity,
  EffectiveModelScope,
  RankAdvisorDefaultCandidates,
  ScopedAdvisorModel,
} from './types.ts';

/**
 Selection-module logger root.
 */
const l = tagged({ tag: 'advisor/advisor-selection', },);

//region Configured initial reviewer

/**
 Outcome of looking up the configured default reviewer.

 `entry` is absent when the key is blank, absent, or unresolvable;
 `keyText` is absent when no key is configured at all.
 */
export type ConfiguredReviewerResolution = {
  /**
   Matching scoped model, present only when the configured key resolves.
   */
  readonly entry?: ScopedAdvisorModel;
  /**
   Trimmed configured key text, present whenever a key is configured.
   */
  readonly keyText?: string;
};

/**
 Resolve the configured default reviewer inside the supplied scope.

 Uses the accepted explicit-model forms (`provider/modelId`, unique bare
 `model.id`, unique `model.name`), but never marks the result as an exact
 request: `defaultReviewerModel` only picks the initial reviewer of a default
 call, so serial fallback and overlap recovery still cover it.

 @param config - runtime Advisor config

 @param scope - scope the default selection may use

 @param modelRegistry - model registry for explicit-slug validation

 @returns resolution outcome naming the configured key and its scoped match

 @example
 ```typescript
 resolveConfiguredDefaultReviewer({ config, scope, modelRegistry });
 ```
 */
export function resolveConfiguredDefaultReviewer(
  {
    config,
    scope,
    modelRegistry,
  }: ForeignBorrowed<Readonly<{
    readonly config: AdvisorConfig;
    readonly scope: EffectiveModelScope;
    readonly modelRegistry: AdvisorModelResolution;
  }>>,
): ConfiguredReviewerResolution {
  /**
   Trimmed configured reviewer key; blank counts as explicitly cleared.
   */
  const keyText = config.defaultReviewerModel
    ?.trim();
  if ((keyText === undefined) || (keyText
    === ''))
    return {};
  /**
   Function-boundary logger for fall-through evidence.
   */
  const innerL = tagged({
    tag: resolveConfiguredDefaultReviewer.name,
    l,
  },);
  try {
    /**
     Scope match for the configured key under explicit-model accepted forms.
     */
    const resolution = resolveAdvisorRequestedModel({
      scope,
      requestedSlug: keyText,
      modelRegistry,
    },);
    innerL.debug(`defaultReviewerModel "${keyText}" resolves as ${resolution.selected
      .canonicalSlug}`,);
    return {
      keyText,
      entry: resolution.selected,
    };
  }
  catch (error) {
    innerL.debug(`defaultReviewerModel "${keyText}" falls through to the default ranking: ${
      caughtValueText(error,)
    }`,);
    return { keyText, };
  }
}

/**
 Return scope entries with the configured initial reviewer appended when missing.

 The configured reviewer can sit outside current-main avoidance
 (`scopeAvoidingCurrentMainModel`), but it must still rank and dispatch as the
 initial reviewer.

 @param scope - default-selection scope after current-main avoidance

 @param configuredReviewer - resolved configured initial reviewer

 @returns original scope, or the same scope with the configured reviewer appended

 @example
 ```typescript
 scopeIncludingConfiguredReviewer({ scope, configuredReviewer });
 ```
 */
export function scopeIncludingConfiguredReviewer(
  {
    scope,
    configuredReviewer,
  }: ForeignBorrowed<Readonly<{
    readonly scope: EffectiveModelScope;
    readonly configuredReviewer?: ScopedAdvisorModel;
  }>>,
): EffectiveModelScope {
  if (configuredReviewer
    === undefined)
    return scope;
  for (const entry of scope.entries) {
    if (entry.canonicalSlug
      === configuredReviewer
        .canonicalSlug)
      return scope;
  }
  return {
    ...scope,
    entries: [
      ...scope.entries,
      configuredReviewer,
    ],
  };
}

/**
 Compose the default-selection result with the configured initial reviewer.

 The configured reviewer leads the ranking (initial dispatch plus fallback
 order); remaining candidates keep their ranking order, so serial fallback and
 overlap recovery are unchanged. An unresolvable configured key is reported in
 the reason and falls through to the ranking without error.

 @param defaultSelection - ranking result before configured-reviewer composition

 @param resolution - configured default reviewer lookup outcome

 @returns default selection led by the configured initial reviewer

 @throws when the configured reviewer's ranking entry disappeared

 @example
 ```typescript
 applyConfiguredInitialReviewer({ defaultSelection, resolution });
 ```
 */
export function applyConfiguredInitialReviewer(
  {
    defaultSelection,
    resolution,
  }: ForeignBorrowed<Readonly<{
    readonly defaultSelection: AdvisorDefaultSelection;
    readonly resolution: ConfiguredReviewerResolution;
  }>>,
): AdvisorDefaultSelection {
  /**
   Configured reviewer match and configured key text from the lookup outcome.
   */
  const {
    entry,
    keyText,
  } = resolution;
  if (entry
    === undefined) {
    if (keyText
      === undefined)
      return defaultSelection;
    return {
      ...defaultSelection,
      reason: `${defaultSelection.reason}; defaultReviewerModel "${keyText}" did not resolve among scoped models, so the default ranking applies`,
    };
  }
  /**
   Configured key label named in the composed reason.
   */
  const keyLabel = keyText ?? entry
    .canonicalSlug;
  /**
   Canonical slug of the configured initial reviewer.
   */
  const configuredSlug = entry
    .canonicalSlug;
  /**
   Ranking entry computed for the configured initial reviewer.
   */
  const headScore = defaultSelection
    .ranking
    .find(function matchesConfiguredSlug(score,): boolean {
      return score.slug
        === configuredSlug;
    },);
  if (headScore
    === undefined)
    throw new Error(
      `advisor: configured defaultReviewerModel "${keyLabel}" selection ${configuredSlug} disappeared from the default ranking`,
    );
  return {
    selected: entry,
    ranking: [
      headScore,
      ...defaultSelection
        .ranking
        .filter(function keepsOtherScores(score,): boolean {
          return score.slug
            !== configuredSlug;
        },),
    ],
    reason: `configured defaultReviewerModel "${keyLabel}" selects ${configuredSlug} as initial reviewer; untried candidates keep the default ranking (${defaultSelection.reason})`,
  };
}

//endregion Configured initial reviewer

//region Public API

/**
 Select explicit model, configured initial reviewer, or default highest expected-cost non-current model.
 
 Selection order: an explicit slug is exact; otherwise `defaultReviewerModel`
 picks the initial reviewer when it resolves in scope;
 otherwise the ranking policy (or the built-in expected-cost ranking when no
 policy is supplied) picks one. Only explicit requests disable fallback and
 overlap.
 
 @param scope - effective scoped model set
 
 @param requestedSlug - optional requested model slug
 
 @param config - runtime Advisor config
 
 @param estimatedInputTokens - estimated Advisor input tokens
 
 @param modelRegistry - global model registry for explicit slug validation
 
 @param currentMainModel - active primary model to avoid for default selection when possible
 
 @param rankDefaultCandidates - optional external default-ranking policy
 
 @returns selected Advisor model
 
 
 @example
 ```typescript
 selectAdvisorModel({ scope, config, estimatedInputTokens, modelRegistry, currentMainModel });
 ```
 */
export function selectAdvisorModel(
  {
    scope,
    requestedSlug,
    config,
    estimatedInputTokens,
    modelRegistry,
    currentMainModel,
    rankDefaultCandidates,
  }: ForeignBorrowed<Readonly<{
    scope: ReadonlyDeep<EffectiveModelScope>;
    requestedSlug?: string;
    config: AdvisorConfig;
    estimatedInputTokens: number;
    modelRegistry: AdvisorModelResolution;
    currentMainModel?: CurrentMainModelIdentity;
    rankDefaultCandidates?: RankAdvisorDefaultCandidates;
  }>>,
): AdvisorModelSelection {
  if ((requestedSlug !== undefined) && (requestedSlug.trim()
    !== '')) {
    /**
     Explicit model resolved against authenticated scope; the request budget
     clamps to the endpoint's advertised capacity at dispatch.
     */
    return resolveAdvisorRequestedModel({
      scope,
      requestedSlug,
      modelRegistry,
    },);
  }

  /**
   Configured initial reviewer lookup, resolved before the variant ranking runs.
   */
  const configuredReviewerResolution = resolveConfiguredDefaultReviewer({
    config,
    scope,
    modelRegistry,
  },);
  /**
   Default-selection scope with current main model removed when alternatives exist.
   */
  const defaultScope = scopeAvoidingCurrentMainModel({
    scope,
    ...(currentMainModel
      === undefined ? {} : { currentMainModel, }),
  },);
  /**
   Ranking scope keeping the configured initial reviewer even when current-main avoidance dropped it.
   */
  const rankingScope = scopeIncludingConfiguredReviewer({
    scope: defaultScope,
    ...(configuredReviewerResolution
      .entry
      === undefined ? {} : { configuredReviewer: configuredReviewerResolution
        .entry, }),
  },);
  /**
   Input token estimates keyed by canonical scoped model slug.
   */
  const estimatedInputTokensBySlug = new Map<string, number>();
  for (const entry of rankingScope.entries) {
    estimatedInputTokensBySlug.set(
      entry.canonicalSlug,
      estimatedInputTokens,
    );
  }
  /**
   Default model selection for empty params.
   */
  const defaultSelection = rankDefaultAdvisorCandidates({
    scope: rankingScope,
    estimatedInputTokensBySlug,
    maxOutputTokens: config.maxAdvisorOutputTokens,
    ...(rankDefaultCandidates
      === undefined ? {} : { rankDefaultCandidates, }),
  },);
  /**
   Default selection led by the configured initial reviewer when one resolves.
   */
  const composedSelection = applyConfiguredInitialReviewer({
    defaultSelection,
    resolution: configuredReviewerResolution,
  },);
  return {
    selected: composedSelection.selected,
    defaultSelection: composedSelection,
  };
}

/**
 Return scope entries excluding current main model when at least one alternative remains.

 Exclusion matches the active primary model by route slug, and by provider
 independent identity (model id plus display name) so the same model served on
 another route does not masquerade as an independent reviewer.
 
 @param scope - effective scoped model set
 
 @param currentMainModel - active primary model to avoid for default Advisor selection
 
 @returns original scope or same metadata with current main model omitted
 
 @example
 ```typescript
 scopeAvoidingCurrentMainModel({ scope, currentMainModel });
 ```
 */
export function scopeAvoidingCurrentMainModel(
  {
    scope,
    currentMainModel,
  }: ReadonlyDeep<{
    readonly scope: EffectiveModelScope;
    readonly currentMainModel?: CurrentMainModelIdentity;
  }>,
): EffectiveModelScope {
  if (currentMainModel === undefined)
    return scope;

  /**
   Canonical slug for active primary model.
   */
  const currentMainModelSlug = `${currentMainModel.provider}/${currentMainModel.id}`;
  /**
   Scoped entries that differ from active primary model, by route slug or by
   provider independent identity (model id plus display name).
   */
  const alternativeEntries: EffectiveModelScope['entries'][number][] = [];
  for (const entry of scope.entries) {
    /**
     Scoped model record compared against the active primary identity.
     */
    const { model: entryModel, } = entry;
    /**
     Whether the entry is the active primary model served on another route.
     */
    const sameModelDifferentRoute = (entryModel.id === currentMainModel.id)
      && (currentMainModel.name !== undefined)
      && (entryModel.name === currentMainModel.name);
    if ((entry.canonicalSlug
      !== currentMainModelSlug)
      && (!sameModelDifferentRoute))
      alternativeEntries[alternativeEntries.length] = entry;
  }

  /**
   Count of non-current default candidates.
   */
  const alternativeEntryCount = alternativeEntries.length;
  /**
   Whether no non-current default candidate exists.
   */
  const noAlternativeEntries = alternativeEntryCount === 0;
  /**
   Original scoped entries before default-candidate filtering.
   */
  const { entries: scopedEntries, } = scope;
  /**
   Count of original scoped entries.
   */
  const scopedEntryCount = scopedEntries.length;
  /**
   Whether active primary model was absent from scope.
   */
  const currentMainModelAbsentFromScope = alternativeEntryCount === scopedEntryCount;
  if (noAlternativeEntries || currentMainModelAbsentFromScope)
    return scope;

  return {
    ...scope,
    entries: alternativeEntries,
  };
}

//endregion Public API
