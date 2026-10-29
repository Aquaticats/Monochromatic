/**
 `/advisor status` text for the DSH plugin. @module
 */

import {
  buildAdvisorSystemPrompt,
  maxContextCharsForAdvisorModel,
  selectAdvisorModel,
  type AdvisorConfig,
  type ScopedAdvisorModel,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import {
  resolveEffectiveScope,
} from '@monochromatic-dev/agent-harness-shared-model-selection/ts';
import type {
  ForeignBorrowed,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { dshRankDefaultCandidates, } from './ranking-policy.ts';
import { dshAdvisorTranscriptSerialization, } from './transcript.ts';
import type {
  DshAdvisorCatalog,
} from './types.ts';

//region Types

/**
 Host facts `/advisor status` reads.
 */
export type BuildDshAdvisorStatusOptions = {
  /**
   Catalog snapshot serving the model-resolution port.
   */
  readonly catalog: DshAdvisorCatalog;
  /**
   Provider route ids from `ctx.llm.listProviders()`, reported in the scope
   source line.
   */
  readonly providerIds: readonly string[];
  /**
   Working directory for project-scope resolution.
   */
  readonly cwd: string;
  /**
   Runtime Advisor config.
   */
  readonly config: AdvisorConfig;
  /**
   Session enablement state.
   */
  readonly enabled: boolean;
  /**
   Recorded transcript degradation reason; empty when acquisition is healthy.
   */
  readonly transcriptDegradationReason: string;
  /**
   Agent route identity used as the current main model to avoid, when both
   provider and model are configured.
   */
  readonly mainProvider?: string;
  /**
   Agent model id paired with `mainProvider`.
   */
  readonly mainModel?: string;
};

//endregion Types

//region Public API

/**
 Build `/advisor status` text.

 Mirrors pi's `buildAdvisorStatus` line set with the DSH adaptations: the
 scope-source line reports the `llm.listProviders` catalog, one added line
 names `defaultReviewerModel` and its resolution state, one added line lists
 output-capacity-unknown candidates, and a final line records degraded
 transcript acquisition after a Q4 fallback.

 @param options - catalog, config, session state, and host facts

 @returns status text

 @throws when scope resolution rejects the effective model scope

 @example
 ```ts
 const text = await buildDshAdvisorStatus({ catalog, providerIds, cwd, config, enabled: true, transcriptDegradationReason: '' });
 ```
 */
export async function buildDshAdvisorStatus(
  options: BuildDshAdvisorStatusOptions,
): Promise<string> {
  /**
   Effective model scope for status.
   */
  const scope = await resolveEffectiveScope({
    ctx: {
      cwd: options.cwd,
      modelRegistry: options.catalog,
    },
    errorPrefix: 'advisor',
  },);
  /**
   Current main model to avoid for default selection, when configured.
   */
  const currentMainModel = (options.mainProvider === undefined)
    || (options.mainModel === undefined)
    ? undefined
    : {
      provider: options.mainProvider,
      id: options.mainModel,
    };
  /**
   Empty-context default ranking for status display.
   */
  const defaultSelection = scope.entries
    .length
    === 0
    ? undefined
    : selectAdvisorModel({
      scope,
      config: options.config,
      estimatedInputTokens: 0,
      modelRegistry: options.catalog,
      rankDefaultCandidates: dshRankDefaultCandidates,
      ...(currentMainModel === undefined
        ? {}
        : { currentMainModel, }),
    },)
      .defaultSelection;
  /**
   Advisor model system prompt used for budget reserve estimate.
   */
  const advisorSystemPrompt = buildAdvisorSystemPrompt(options.config,);
  /**
   Effective context budget for the status default model.
   */
  const defaultContextBudget = defaultSelection === undefined
    ? undefined
    : maxContextCharsForAdvisorModel({
      transcript: dshAdvisorTranscriptSerialization,
      config: options.config,
      model: defaultSelection.selected
        .model,
      advisorSystemPrompt,
    },);
  /**
   Effective context budget shown when present.
   */
  const defaultContextBudgetText = defaultContextBudget === undefined
    ? 'none'
    : `${String(defaultContextBudget,)} chars`;
  /**
   Configured context cap shown when present.
   */
  const configuredContextCap = options.config
    .maxContextChars
    === undefined
    ? 'none'
    : `${String(options.config
      .maxContextChars,)} chars`;
  /**
   Configured `defaultReviewerModel` value, or the unset sentinel.
   */
  const reviewerModel = options.config
    .defaultReviewerModel
    ?? 'unset';
  /**
   Whether the configured reviewer resolves inside the effective scope.
   */
  const reviewerResolved = (options.config
    .defaultReviewerModel
    !== undefined)
    && scope
      .entries
      .some(function matchesReviewer(
        entry: ForeignBorrowed<ScopedAdvisorModel>,
      ) {
        return (entry.canonicalSlug
          === options.config
            .defaultReviewerModel)
          || (entry.model
            .id
            === options.config
              .defaultReviewerModel)
          || (entry.model
            .name
            === options.config
              .defaultReviewerModel);
      },);
  /**
   Scoped slugs whose output capacity is unknown; requests to them carry the
   configured budget unclamped.
   */
  const capacityUnknownSlugs = scope
    .entries
    .filter(function hasUnknownCapacity(
      entry: ForeignBorrowed<ScopedAdvisorModel>,
    ) {
      return options
        .catalog
        .factsBySlug
        .get(entry.canonicalSlug,)
        ?.outputCapacityUnknown
        === true;
    },)
    .map(function readSlug(
      entry: ForeignBorrowed<ScopedAdvisorModel>,
    ) {
      return entry.canonicalSlug;
    },);

  return [
    `Advisor: ${options.enabled
      ? 'on'
      : 'off'}`,
    `Scope source: ${scope.source}; llm providers: ${options.providerIds
      .length
      === 0
      ? 'none'
      : options.providerIds
        .join(', ',)}`,
    'Default failures: serial fallback through untried scoped models; explicit requests remain exact',
    `Overlapping default reviews: ${options.config
      .hedgingEnabled
      ? `enabled after ${String(options.config
        .hedgeDelayMs,)}ms`
      : 'disabled'}`,
    `Collection grace: ${String(options.config
      .collectionGraceMs,)}ms after first usable review, capped by original deadline`,
    'Overlap can bill both requests; the set of completed reviewers is nondeterministic',
    'Credit exhaustion excludes that provider for the current call only; cancellation does not guarantee complete billing data',
    `Scoped models: ${scope.entries
      .length
      === 0
      ? 'none'
      : String(scope.entries
        .length,)}`,
    `Advisor request budget: ${
      String(options.config
        .maxAdvisorOutputTokens,)
    } output tokens, clamped to each route's advertised maximum`,
    `Output capacity unknown: ${capacityUnknownSlugs.length === 0
      ? 'none'
      : capacityUnknownSlugs.join(', ',)}`,
    `Default model: ${defaultSelection?.selected
      .canonicalSlug
      ?? 'none'}`,
    `Default ranking: ${defaultSelection?.reason
      ?? 'none'}`,
    `Default reviewer model: ${reviewerModel}${options.config
      .defaultReviewerModel
      === undefined
      ? ''
      : reviewerResolved
        ? ' (resolved in scope)'
        : ' (absent from scope)'}`,
    `Config: global=${
      options.config
        .source
        .globalLoaded ? options.config
          .source
          .globalPath : 'absent'
    } project=${options.config
      .source
      .projectLoaded ? options.config
        .source
        .projectPath : 'absent'}`,
    [
      `Context budget: ${defaultContextBudgetText} effective for default model,`,
      `cap=${configuredContextCap},`,
      `${String(options.config
        .maxAdvisorOutputTokens,)} output tokens`,
    ]
      .join(' ',),
    `Prior Advisor results: ${
      options.config
        .includePriorAdvisorResults
        ? 'included'
        : 'omitted'
    }`,
    ...(options.transcriptDegradationReason === ''
      ? []
      : [
        `Session transcript: derived history unavailable (${options.transcriptDegradationReason}); the question parameter carries the review subject`,
      ]),
  ]
    .join('\n',);
}

//endregion Public API
