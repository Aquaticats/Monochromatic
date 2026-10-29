/**
 Freeze evidence once while retaining each candidate's own context budget. @module
 */
import type { ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { resolveEffectiveScope, } from '@monochromatic-dev/agent-harness-shared-model-selection/ts';
import { buildAdvisorSystemPromptForProject, } from './advisor-client.ts';
import {
  selectAdvisorRunContext,
  type AdvisorSelectionContext,
  type AdvisorContextCandidate,
} from './tool-context-selection.ts';
import type {
  AdvisorRunOptions,
  EffectiveModelScope,
} from './types.ts';

/**
 Prepared evidence and original selection provenance.
 */
export type PreparedAdvisorRun = {
  /**
   Original effective scope and source.
   */
  readonly scope: EffectiveModelScope;
  /**
   Initial selection and evidence, before any failure fallback.
   */
  readonly initial: AdvisorSelectionContext;
  /**
   Ranked candidates sharing a single session snapshot.
   */
  readonly candidates: readonly AdvisorContextCandidate[];
};

/**
 Resolve the original scope and serialize candidate evidence before any provider dispatch.
 
 @param options - Pi capabilities and captured project context
 
 @returns candidate evidence in existing default ranking order
 
 @mutates options - scope and session capabilities are read through the host
 
 @throws when scope or explicit model selection is unavailable
 
 @example
 ```ts
 const prepared = await prepareAdvisorRun(options);
 ```
 */
export async function prepareAdvisorRun(options: ForeignHostCapability<AdvisorRunOptions>,): Promise<PreparedAdvisorRun> {
  /**
   Effective scope resolved for this operation.
   */
  const scope = await resolveEffectiveScope({
    ctx: options.ctx,
    errorPrefix: 'advisor',
  },);
  if (scope.entries
    .length
    === 0)
    throw new Error('advisor: no scoped models with configured auth. Check --models, enabledModels, /scoped-models, or provider login.',);
  /**
   Same project instructions and question are used for every candidate.
   */
  const advisorSystemPrompt = buildAdvisorSystemPromptForProject({
    config: options.config,
    projectContext: options.projectContext ?? '',
  },);
  /**
   One compaction-aware session snapshot.
   */
  const initial = selectAdvisorRunContext({
    branch: options.ctx
      .transcript
      .buildContextEntries(),
    transcript: options.ctx
      .transcript,
    config: options.config,
    advisorSystemPrompt,
    scope,
    modelRegistry: options.ctx
      .modelRegistry,
    ...(options.ctx
      .model
      === undefined ? {} : { currentMainModel: options.ctx
        .model, }),
    ...(options.requestedSlug === undefined ? {} : { requestedSlug: options.requestedSlug, }),
    ...(options.question === undefined ? {} : { question: options.question, }),
    ...(options.toolCallId === undefined ? {} : { toolCallId: options.toolCallId, }),
    ...(options.rankDefaultCandidates
      === undefined ? {} : { rankDefaultCandidates: options.rankDefaultCandidates, }),
  },);
  return {
    scope,
    initial,
    candidates: initial.candidates ?? [{
      scopedModel: initial.selection
        .selected,
      advisorContext: initial.advisorContext,
    },],
  };
}
