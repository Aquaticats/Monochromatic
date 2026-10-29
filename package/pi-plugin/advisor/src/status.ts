/**
 Advisor status rendering for slash-command inspection.
 
 @module
 */

import type { ExtensionCommandContext, } from '@earendil-works/pi-coding-agent';
import type { ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  buildAdvisorSystemPrompt,
  maxContextCharsForAdvisorModel,
  selectAdvisorModel,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import { resolveEffectiveScope, } from '@monochromatic-dev/agent-harness-shared-model-selection/ts';
import { piAdvisorTranscriptSerialization, } from './run-context.ts';
import type { AdvisorConfig, } from './types.ts';

/**
 Build `/advisor status` text.
 
 @param ctx - command-capable extension context
 
 @param config - runtime Advisor config
 
 @param enabled - session enablement state
 
 @returns status text
 
 @mutates ctx - `resolveEffectiveScope` invokes context live-scope and model-registry callbacks
 
 @example
 ```typescript
 const text = buildAdvisorStatus({ ctx, config, enabled: true });
 ```
 */
export async function buildAdvisorStatus(
  {
    ctx,
    config,
    enabled,
  }: {
    readonly ctx: ForeignHostCapability<ExtensionCommandContext>;
    readonly config: AdvisorConfig;
    readonly enabled: boolean;
  },
): Promise<string> {
  /**
   Effective model scope for status.
   */
  const scope = await resolveEffectiveScope({
    ctx,
    errorPrefix: 'advisor',
  },);
  /**
   Empty-context default ranking for status display.
   */
  const defaultSelection = scope.entries
    .length
    === 0
    ? undefined
    : selectAdvisorModel({
      scope,
      config,
      estimatedInputTokens: 0,
      modelRegistry: ctx.modelRegistry,
      ...(ctx.model
        === undefined ? {} : { currentMainModel: ctx.model, }),
    },)
      .defaultSelection;
  /**
   Advisor model system prompt used for budget reserve estimate.
   */
  const advisorSystemPrompt = buildAdvisorSystemPrompt(config,);
  /**
   Effective context budget for status default model.
   */
  const defaultContextBudget = defaultSelection === undefined
    ? undefined
    : maxContextCharsForAdvisorModel({
      transcript: piAdvisorTranscriptSerialization,
      config,
      model: defaultSelection.selected
        .model,
      advisorSystemPrompt,
    },);
  /**
   Effective context budget shown when present.
   */
  const defaultContextBudgetText = defaultContextBudget === undefined
    ? 'none'
    : `${defaultContextBudget} chars`;
  /**
   Configured context cap shown when present.
   */
  const configuredContextCap = config.maxContextChars
    === undefined
    ? 'none'
    : `${config.maxContextChars} chars`;

  return [
    `Advisor: ${enabled ? 'on' : 'off'}`,
    `Scope source: ${scope.source}`,
    'Default failures: serial fallback through untried scoped models; explicit requests remain exact',
    `Overlapping default reviews: ${config.hedgingEnabled ? `enabled after ${String(config.hedgeDelayMs,)}ms` : 'disabled'}`,
    `Collection grace: ${String(config.collectionGraceMs,)}ms after first usable review, capped by original deadline`,
    'Overlap can bill both requests; the set of completed reviewers is nondeterministic',
    'Credit exhaustion excludes that provider for the current call only; cancellation does not guarantee complete billing data',
    `Scoped models: ${
      scope.entries
        .length
        === 0 ? 'none' : String(scope.entries
          .length,)
    }`,
    `Advisor request budget: ${
      String(config.maxAdvisorOutputTokens,)
    } output tokens, clamped to each route's advertised maximum`,
    `Default model: ${defaultSelection?.selected
      .canonicalSlug
      ?? 'none'}`,
    `Default ranking: ${defaultSelection?.reason
      ?? 'none'}`,
    `Config: global=${
      config.source
        .globalLoaded ? config.source
          .globalPath : 'absent'
    } project=${config.source
      .projectLoaded ? config.source
        .projectPath : 'absent'}`,
    [
      `Context budget: ${defaultContextBudgetText} effective for default model,`,
      `cap=${configuredContextCap},`,
      `${config.maxAdvisorOutputTokens} output tokens`,
    ]
      .join(' ',),
    `Prior Advisor results: ${
      config.includePriorAdvisorResults ? 'included' : 'omitted'
    }`,
  ]
    .join('\n',);
}
