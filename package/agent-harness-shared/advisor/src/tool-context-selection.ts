/**
 Advisor model selection paired with model-budgeted serialized context.
 
 @module
 */

import type { ReadonlyDeep, } from 'type-fest';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  buildAdvisorContext,
  maxContextCharsForAdvisorModel,
} from './context.ts';
import {
  applyConfiguredInitialReviewer,
  resolveConfiguredDefaultReviewer,
  scopeAvoidingCurrentMainModel,
  scopeIncludingConfiguredReviewer,
  selectAdvisorModel,
} from './advisor-selection.ts';
import { rankDefaultAdvisorCandidates, } from './default-ranking.ts';
import type {
  AdvisorModelResolution,
  AdvisorSessionEntry,
  AdvisorTranscriptSerialization,
} from './ports.ts';
import type {
  AdvisorConfig,
  AdvisorContext,
  AdvisorModelSelection,
  CurrentMainModelIdentity,
  EffectiveModelScope,
  RankAdvisorDefaultCandidates,
  ScopedAdvisorModel,
} from './types.ts';

//region Types

/**
 Selected model paired with serialized context built for that model.
 */
export type AdvisorSelectionContext = {
  /**
   Advisor model selection.
   */
  readonly selection: AdvisorModelSelection;
  /**
   Serialized context using selected model budget.
   */
  readonly advisorContext: AdvisorContext;
  /**
   Candidate evidence snapshots used by default fallback without re-reading the session.
   */
  readonly candidates?: readonly AdvisorContextCandidate[];
};

/**
 Options for selecting model and model-budgeted context together.
 */
export type SelectAdvisorRunContextOptions = ForeignBorrowed<{
  /**
   Session branch entries from the host transcript port.
   */
  readonly branch: readonly ForeignBorrowed<AdvisorSessionEntry>[];
  /**
   Transcript serialization seam used for LLM conversion and token estimates.
   */
  readonly transcript: AdvisorTranscriptSerialization;
  /**
   Runtime Advisor configuration.
   */
  readonly config: AdvisorConfig;
  /**
   Advisor model system prompt.
   */
  readonly advisorSystemPrompt: string;
  /**
   Effective scoped model set.
   */
  readonly scope: ReadonlyDeep<EffectiveModelScope>;
  /**
   Global model registry for explicit slug validation.
   */
  readonly modelRegistry: AdvisorModelResolution;
  /**
   Active primary model to avoid for default selection when possible.
   */
  readonly currentMainModel?: CurrentMainModelIdentity;
  /**
   Optional user-requested model slug.
   */
  readonly requestedSlug?: string;
  /**
   Focused question supplied by the primary agent.
   */
  readonly question?: string;
  /**
   Current Advisor tool call id to omit.
   */
  readonly toolCallId?: string;
  /**
   External default-ranking policy; absent keeps expected-cost ranking (pi behavior).
   */
  readonly rankDefaultCandidates?: RankAdvisorDefaultCandidates;
}>;

/**
 Options for building context for one scoped Advisor model.
 */
type BuildContextForScopedModelOptions = ForeignBorrowed<Readonly<{
  /**
   Session branch entries from the host transcript port.
   */
  readonly branch: readonly AdvisorSessionEntry[];
  /**
   Transcript serialization seam used for LLM conversion and token estimates.
   */
  readonly transcript: AdvisorTranscriptSerialization;
  /**
   Runtime Advisor configuration.
   */
  readonly config: AdvisorConfig;
  /**
   Advisor model system prompt.
   */
  readonly advisorSystemPrompt: string;
  /**
   Scoped Advisor model.
   */
  readonly scopedModel: ScopedAdvisorModel;
  /**
   Focused question supplied by the primary agent.
   */
  readonly question?: string;
  /**
   Current Advisor tool call id to omit.
   */
  readonly toolCallId?: string;
}>>;

/**
 Context candidate for a scoped Advisor model.
 */
export type AdvisorContextCandidate = {
  /**
   Scoped Advisor model.
   */
  readonly scopedModel: ScopedAdvisorModel;
  /**
   Serialized context using scoped model budget.
   */
  readonly advisorContext: AdvisorContext;
};

//endregion Types

//region Candidate lookup

/**
 Return context candidate for selected canonical model slug.
 
 @param candidates - model-budgeted context candidates
 
 @param selectedSlug - selected canonical model slug
 
 @returns matching context candidate
 
 @throws when selected candidate disappeared
 
 @example
 ```typescript
 selectedContextCandidate({ candidates, selectedSlug: 'provider/model' });
 ```
 */
function selectedContextCandidate(
  {
    candidates,
    selectedSlug,
  }: {
    readonly candidates: readonly AdvisorContextCandidate[];
    readonly selectedSlug: string;
  },
): AdvisorContextCandidate {
  for (const candidate of candidates) {
    if (candidate.scopedModel
      .canonicalSlug
      === selectedSlug)
      return candidate;
  }
  throw new Error(
    `advisor: selected model ${selectedSlug} context disappeared`,
  );
}

//endregion Candidate lookup

//region Public API

/**
 Select Advisor model and build context with that model's context budget.
 
 @param options - branch, config, scope, and model-selection inputs
 
 @returns selected model and serialized context
 
 
 @example
 ```typescript
 const selectionContext = selectAdvisorRunContext({ branch, config, advisorSystemPrompt, scope, modelRegistry });
 ```
 */
export function selectAdvisorRunContext(
  options: SelectAdvisorRunContextOptions,
): AdvisorSelectionContext {
  if (options.requestedSlug
    !== undefined) {
    /**
     Explicit Advisor model selection.
     */
    const selection = selectAdvisorModel({
      scope: options.scope,
      requestedSlug: options.requestedSlug,
      config: options.config,
      estimatedInputTokens: 0,
      modelRegistry: options.modelRegistry,
      ...(options.currentMainModel
        === undefined ? {} : { currentMainModel: options.currentMainModel, }),
    },);
    return {
      selection,
      advisorContext: buildContextForScopedModel({
        branch: options.branch,
        transcript: options.transcript,
        config: options.config,
        advisorSystemPrompt: options.advisorSystemPrompt,
        scopedModel: selection.selected,
        ...(options.question
          === undefined ? {} : { question: options.question, }),
        ...(options.toolCallId
          === undefined ? {} : { toolCallId: options.toolCallId, }),
      },),
    };
  }

  /**
   Configured initial reviewer lookup, resolved before the variant ranking runs.
   */
  const configuredReviewerResolution = resolveConfiguredDefaultReviewer({
    config: options.config,
    scope: options.scope,
    modelRegistry: options.modelRegistry,
  },);
  /**
   Default-selection scope with current main model removed when alternatives exist.
   */
  const defaultScope = scopeAvoidingCurrentMainModel({
    scope: options.scope,
    ...(options.currentMainModel
      === undefined ? {} : { currentMainModel: options.currentMainModel, }),
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
   Context candidates using each scoped model's effective context budget.
   */
  const candidates: AdvisorContextCandidate[] = [];
  for (const scopedModel of rankingScope.entries) {
    candidates.push({
      scopedModel,
      advisorContext: buildContextForScopedModel({
        branch: options.branch,
        transcript: options.transcript,
        config: options.config,
        advisorSystemPrompt: options.advisorSystemPrompt,
        scopedModel,
        ...(options.question
          === undefined ? {} : { question: options.question, }),
        ...(options.toolCallId
          === undefined ? {} : { toolCallId: options.toolCallId, }),
      },),
    },);
  }
  /**
   Input token estimates keyed by canonical scoped model slug.
   */
  const estimatedInputTokensBySlug = new Map<string, number>();
  for (const candidate of candidates) {
    estimatedInputTokensBySlug.set(
      candidate.scopedModel
        .canonicalSlug,
      candidate.advisorContext
        .estimatedInputTokens,
    );
  }
  /**
   Default Advisor model selection using each candidate's own estimate.
   */
  const defaultSelection = rankDefaultAdvisorCandidates({
    scope: rankingScope,
    estimatedInputTokensBySlug,
    maxOutputTokens: options.config
      .maxAdvisorOutputTokens,
    ...(options.rankDefaultCandidates
      === undefined ? {} : { rankDefaultCandidates: options.rankDefaultCandidates, }),
  },);
  /**
   Default selection led by the configured initial reviewer when one resolves.
   */
  const composedSelection = applyConfiguredInitialReviewer({
    defaultSelection,
    resolution: configuredReviewerResolution,
  },);
  /**
   Context candidate matching the initial default model.
   */
  const selectedCandidate = selectedContextCandidate({
    candidates,
    selectedSlug: composedSelection.selected
      .canonicalSlug,
  },);
  return {
    selection: {
      selected: composedSelection.selected,
      defaultSelection: composedSelection,
    },
    advisorContext: selectedCandidate.advisorContext,
    candidates: composedSelection.ranking
      .map(function rankedCandidate(score): AdvisorContextCandidate {
        return selectedContextCandidate({
          candidates,
          selectedSlug: score.slug,
        },);
      },),
  };
}

//endregion Public API

//region Internal helpers

/**
 Build serialized Advisor context for one scoped model.
 
 @param options - branch, config, prompt, scoped model, and current tool call
 
 @returns serialized context truncated for scoped model
 */
function buildContextForScopedModel(
  options: BuildContextForScopedModelOptions,
): AdvisorContext {
  /**
   Effective serialized-context character budget for selected model.
   */
  const maxContextChars = maxContextCharsForAdvisorModel({
    transcript: options.transcript,
    config: options.config,
    model: options.scopedModel
      .model,
    advisorSystemPrompt: options.advisorSystemPrompt,
    ...(options.question === undefined ? {} : { question: options.question, }),
  },);
  return buildAdvisorContext({
    branch: options.branch,
    transcript: options.transcript,
    config: options.config,
    advisorSystemPrompt: options.advisorSystemPrompt,
    maxContextChars,
    ...(options.question === undefined ? {} : { question: options.question, }),
    ...(options.toolCallId
      === undefined ? {} : { toolCallId: options.toolCallId, }),
  },);
}

//endregion Internal helpers
