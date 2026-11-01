/**
 Host-neutral Advisor core type definitions.

 Host-specific surface lives behind the ports in `ports.ts`; harness
 adapters translate their own tool and run shapes onto these types.

 @module
 */

import type {
  StopReason,
  Usage,
} from '@earendil-works/pi-ai';
import type { ReadonlyDeep, } from 'type-fest';
import type { AdvisorOperationSnapshot, } from './operation-types.ts';
import type {
  AdvisorModelResolution,
  AdvisorTranscript,
} from './ports.ts';
import type {
  ForeignHostCapability,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Static,
  TSchema,
} from 'typebox';
import type {
  DefaultModelSelection as SharedDefaultModelSelection,
  EffectiveModelScope as SharedEffectiveModelScope,
  ModelCostScore as SharedModelCostScore,
  ReadonlyModel,
  ScopedModel,
  ScopedThinkingLevel as SharedScopedThinkingLevel,
  ScopeSource as SharedScopeSource,
} from '@monochromatic-dev/agent-harness-shared-model-selection/ts';

//region Configuration

/**
 Source metadata for merged Advisor config.
 */
export type AdvisorConfigSource = {
  /**
   Absolute path checked for global config.
   */
  readonly globalPath: string;
  /**
   Absolute path checked for project config.
   */
  readonly projectPath: string;
  /**
   Whether global config file existed and parsed.
   */
  readonly globalLoaded: boolean;
  /**
   Whether project config file existed and parsed.
   */
  readonly projectLoaded: boolean;
};

/**
 Runtime Advisor extension configuration.
 */
export type AdvisorConfig = {
  /**
   Whether Advisor starts enabled for each session.
   */
  readonly enabled: boolean;
  /**
   Total Advisor operation deadline in milliseconds.
   */
  readonly timeoutMs: number;
  /**
   Explicit opt-in to overlapping default reviews.
   */
  readonly hedgingEnabled: boolean;
  /**
   Required launch delay when overlapping reviews are enabled.
   */
  readonly hedgeDelayMs?: number;
  /**
   Post-success collection grace, still bounded by timeoutMs.
   */
  readonly collectionGraceMs: number;
  /**
   Optional hard cap for serialized context characters sent to Advisor.
   */
  readonly maxContextChars?: number;
  /**
   Maximum output tokens requested from the advisor model.
   */
  readonly maxAdvisorOutputTokens: number;
  /**
   Whether prior Advisor tool results stay in serialized context.
   */
  readonly includePriorAdvisorResults: boolean;
  /**
   Project-specific prompt suffix for the advisor model.
   */
  readonly systemPrompt?: string;
  /**
   Initial reviewer for default calls, resolved inside the effective scope.

   User config only (`~/.pi/agent/extensions/pi-advisor.json` plus project
   overrides); never a schema default and never a host plugin config-row
   value. Empty or blank clears the key.
   */
  readonly defaultReviewerModel?: string;
  /**
   Config source metadata surfaced by `/advisor status`.
   */
  readonly source: AdvisorConfigSource;
};

//endregion Configuration

//region Scope

/**
 Where the effective scoped model set came from.
 */
export type ScopeSource = SharedScopeSource;

/**
 Thinking level suffix carried by pi model scope patterns.
 */
export type ScopedThinkingLevel = SharedScopedThinkingLevel;

/**
 Deep-readonly view of pi-ai's `Model<Api>` with `api` widened to plain `string`.
 
 Pi-ai's `Api` is `KnownApi | (string & {})`; the `(string & {})` brand
 trick is treated as non-readonly by the lint rule. Hand-authoring the
 mirror with primitive types cleans the chain reliably while remaining
 structurally assignable from the live `Model<Api>` value.
 */
export type AdvisorReadonlyModel = ReadonlyModel;

/**
 Model entry available to Advisor after scope resolution.
 */
export type ScopedAdvisorModel = ScopedModel;

/**
 Effective scoped model set for a tool call or status query.
 */
export type EffectiveModelScope = SharedEffectiveModelScope;

//endregion Scope

//region Selection

/**
 Cost inputs computed for one model during default selection.
 */
export type ModelCostScore = SharedModelCostScore;

/**
 Default model selection result.
 */
export type DefaultModelSelection = SharedDefaultModelSelection;

/**
 Ranking evidence recorded for one default-selection candidate.

 Entries carry at least the canonical slug; the built-in expected-cost ranking
 stores full cost scores here, and external ranking policies store their order.
 */
export type AdvisorRankingScore = {
  /**
   Canonical slug of the ranked model.
   */
  readonly slug: string;
};

/**
 Eligible candidate handed to a default-ranking policy.
 */
export type AdvisorRankingCandidate = {
  /**
   Scoped model under consideration.
   */
  readonly scopedModel: ScopedAdvisorModel;
  /**
   Estimated Advisor input tokens for this candidate's serialized context.
   */
  readonly estimatedInputTokens: number;
};

/**
 Ranking decision produced by an external default-ranking policy.
 */
export type AdvisorDefaultRanking = {
  /**
   Input scoped candidates reordered best-first; a permutation of the inputs.
   */
  readonly ranked: readonly ScopedAdvisorModel[];
  /**
   Human-readable ranking reason surfaced as the default-selection reason.
   */
  readonly reason: string;
};

/**
 External ranking policy for default (non-explicit) selection.

 Harness adapters compute their own ordering outside the core (the core never
 reads host model metadata) and hand the result back pre-ranked. When no policy
 is supplied, selection ranks by expected call cost (pi behavior).
 */
export type RankAdvisorDefaultCandidates = (
  options: {
    /**
     Eligible candidates after output-capacity filtering and current-main avoidance.
     */
    readonly candidates: readonly AdvisorRankingCandidate[];
    /**
     Requested output token budget for the default request.
     */
    readonly maxOutputTokens: number;
  },
) => AdvisorDefaultRanking;

/**
 Default-selection decision for empty params.
 */
export type AdvisorDefaultSelection = {
  /**
   Scoped model chosen as the initial reviewer.
   */
  readonly selected: ScopedAdvisorModel;
  /**
   Ranked candidates, best first; the initial reviewer leads after composition.
   */
  readonly ranking: readonly AdvisorRankingScore[];
  /**
   Human-readable reason for the initial pick and the fallback order.
   */
  readonly reason: string;
};

/**
 Explicit or default selection result for a tool call.
 */
export type AdvisorModelSelection = {
  /**
   Selected scoped model.
   */
  readonly selected: ScopedAdvisorModel;
  /**
   Requested slug, when user supplied one.
   */
  readonly requestedSlug?: string;
  /**
   Default-selection metadata, present for empty params.
   */
  readonly defaultSelection?: AdvisorDefaultSelection;
};

/**
 Identity of the primary model, used to avoid self-review in default selection.

 The display name, when known, extends the identity across provider routes:
 the same model id under two routes with one name is still the primary model.
 Ids repeat across unrelated models, so identity matching without a name stays
 route-exact.
 */
export type CurrentMainModelIdentity = Pick<AdvisorModelSelection['selected']['model'], 'provider' | 'id'> & {
  /**
   Display name of the active primary model, used to recognize the same model
   served on another provider route.
   */
  readonly name?: string;
};

//endregion Selection

//region Context

/**
 Serialized conversation context plus metadata.
 */
export type AdvisorContext = {
  /**
   Serialized conversation after deterministic truncation.
   */
  readonly text: string;
  /**
   Effective serialized-context character budget used for truncation.
   */
  readonly maxContextChars: number;
  /**
   Serialized conversation length before truncation.
   */
  readonly originalChars: number;
  /**
   Serialized conversation length after truncation.
   */
  readonly finalChars: number;
  /**
   Whether middle content was omitted.
   */
  readonly truncated: boolean;
  /**
   Number of session messages included before LLM conversion.
   */
  readonly includedMessageCount: number;
  /**
   Estimated tokens for the advisor request.
   */
  readonly estimatedInputTokens: number;
  /**
   Latest user prompt excerpt, when one exists.
   */
  readonly latestUserPromptExcerpt?: string;
};

//endregion Context

//region Tool and command results

/**
 Tool parameters exposed to the primary model.
 */
export type AdvisorToolParams = {
  /**
   Optional scoped model slug. Empty params select default non-current scoped model when possible.
   */
  readonly model?: string;
  /**
   Optional focused question for Advisor to answer using conversation context.
   */
  readonly question?: string;
};

/**
 Structured details stored with Advisor tool and command results.
 */
export type AdvisorDetails = {
  /**
   Full accounting for all attempts rather than only the first returned review.
   */
  readonly operation?: AdvisorOperationSnapshot;
  /**
   User-requested slug, if supplied.
   */
  readonly requestedSlug?: string;
  /**
   Canonical selected model slug.
   */
  readonly selectedSlug: string;
  /**
   Selected model provider.
   */
  readonly provider: string;
  /**
   Source of scoped model set.
   */
  readonly scopeSource: ScopeSource;
  /**
   Canonical slugs allowed for this call.
   */
  readonly scopedSlugs: readonly string[];
  /**
   Default-selection reason, when empty params selected the model.
   */
  readonly defaultSelectionReason?: string;
  /**
   Duration in milliseconds for provider call and local preparation.
   */
  readonly durationMs: number;
  /**
   Effective serialized-context character budget for selected model.
   */
  readonly contextBudgetChars: number;
  /**
   Final serialized context character count.
   */
  readonly contextChars: number;
  /**
   Estimated input tokens for the advisor request.
   */
  readonly estimatedInputTokens: number;
  /**
   Whether serialized context was truncated.
   */
  readonly truncated: boolean;
  /**
   Advisor model stop reason.
   */
  readonly stopReason: StopReason;
  /**
   Usage metadata returned by the provider, when present.
   */
  readonly usage?: ReadonlyDeep<Usage>;
  /**
   Default ranking evidence, best first, for status and diagnostics.
   */
  readonly costRanking?: readonly AdvisorRankingScore[];
};

/**
 Advisor text plus details.
 */
export type AdvisorRunResult = {
  /**
   Advisor text response.
   */
  readonly text: string;
  /**
   Tool-compatible details object.
   */
  readonly details: AdvisorDetails;
};

/**
 Host run context: the three Advisor ports plus host facts selection reads.

 Harness adapters assemble this from their own runtime objects; the core
 never reaches past these members. The model-resolution port keeps the
 `modelRegistry` member name so it also satisfies the shared
 model-selection scope resolver directly.

 @example
 ```typescript
 const ctx: AdvisorRunContext = {
   modelRegistry,
   transcript,
   cwd,
 };
 ```
 */
export type AdvisorRunContext = {
  /**
   Model-resolution port backed by the host model registry.
   */
  readonly modelRegistry: AdvisorModelResolution;
  /**
   Transcript port backed by the host session manager and serializer.
   */
  readonly transcript: AdvisorTranscript;
  /**
   Host working directory for project-scope resolution.
   */
  readonly cwd: string;
  /**
   Active primary model to avoid for default selection, when the host has one.
   */
  readonly model?: CurrentMainModelIdentity;
  /**
   Live scoped-model getter from the host runtime, when exposed.
   */
  readonly getScopedModels?: () => unknown;
  /**
   Live scoped-model snapshot from the host runtime, when exposed.
   */
  readonly scopedModels?: unknown;
};

/**
 Arguments for executing Advisor.
 */
export type AdvisorRunOptions = ForeignHostCapability<{
  /**
   Host run context carrying the three Advisor ports.
   */
  readonly ctx: ForeignHostCapability<AdvisorRunContext>;
  /**
   Runtime config.
   */
  readonly config: AdvisorConfig;
  /**
   Loaded project-context files serialized for Advisor system prompt.
   */
  readonly projectContext?: string;
  /**
   Optional user-requested model slug.
   */
  readonly requestedSlug?: string;
  /**
   Optional focused question for Advisor to answer using conversation context.
   */
  readonly question?: string;
  /**
   Current advisor tool call id, used to omit placeholder context.
   */
  readonly toolCallId?: string;
  /**
   External default-ranking policy; absent keeps expected-cost ranking (pi behavior).
   */
  readonly rankDefaultCandidates?: RankAdvisorDefaultCandidates;
  /**
   Bounded ledger-derived progress callback.
   */
  readonly onUpdate?: (operation: AdvisorOperationSnapshot) => void;
  /**
   Abort signal from tool or command execution.
   */
  readonly signal?: ForeignHostCapability<AbortSignal>;
}>;

/**
 Metadata-only progress discriminator; provider diagnostics and review text never cross this update boundary.
 */
export type AdvisorProgressDetails = { readonly kind: 'advisor-progress'; };

/**
 Extracted static parameter type for typebox schemas.
 */
export type ToolParamsFor<TParameters extends TSchema,> = Static<TParameters>;

//endregion Tool and command results
