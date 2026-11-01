/**
 Shared Advisor review pipeline for tool calls and `/advisor` reviews. @module
 */

import {
  runAdvisor as runAdvisorCore,
  formatAdvisorProgress,
  type AdvisorConfig,
  type AdvisorOperationSnapshot,
  type AdvisorRunResult,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import { buildDshAdvisorCatalog, } from './catalog.ts';
import {
  buildDshAdvisorProjectContext,
  type DshAdvisorSystemPromptSurface,
} from './project-context.ts';
import { dshRankDefaultCandidates, } from './ranking-policy.ts';
import { createDshAdvisorRunContext, } from './run-context.ts';
import { createDshAdvisorTranscript, } from './transcript.ts';
import type {
  DshAdvisorAgentSurface,
  DshAdvisorLlmSurface,
} from './types.ts';

//region Types

/**
 Inputs for one immediate Advisor review.
 */
export type RunDshAdvisorReviewOptions = {
  /**
   Runtime config snapshot.
   */
  readonly config: AdvisorConfig;
  /**
   DSH LLM surface serving route catalog and streaming.
   */
  readonly llm: DshAdvisorLlmSurface;
  /**
   System-prompt registry surface for project context.
   */
  readonly systemPrompt: DshAdvisorSystemPromptSurface;
  /**
   Agent surface carrying the session, working directory, and route identity.
   */
  readonly agent: DshAdvisorAgentSurface;
  /**
   Degradation recorder shared with `/advisor status`.
   */
  readonly recordTranscriptDegradation: (reason: string) => void;
  /**
   Optional exact reviewer request.
   */
  readonly requestedSlug?: string;
  /**
   Optional focused review question.
   */
  readonly question?: string;
  /**
   Optional tool call id used to omit the placeholder context entry.
   */
  readonly toolCallId?: string;
  /**
   Caller cancellation.
   */
  readonly signal?: AbortSignal;
  /**
   Progress observer invoked after each bounded status line.
   */
  readonly onUpdate?: (operation: AdvisorOperationSnapshot) => void;
};

/**
 Review result with the folded progress line for tool rendering.
 */
export type DshAdvisorReviewResult = {
  /**
   Collected reviews and operation metadata.
   */
  readonly result: AdvisorRunResult;
  /**
   Latest bounded progress line, empty when no progress update arrived.
   */
  readonly progress: string;
};

//endregion Types

//region Public API

/**
 Run one Advisor review over the captured conversation and project context.

 Builds the route catalog, project context, and transcript snapshot for the
 operation, then delegates to the shared core with the DSH ranking policy.

 @param options - config, host surfaces, and review parameters

 @returns review text, details, and folded progress line

 @throws when the shared core rejects the request or every candidate fails

 @example
 ```ts
 const { result, progress } = await runDshAdvisorReview({ config, llm, systemPrompt, agent, recordTranscriptDegradation });
 ```
 */
export async function runDshAdvisorReview(
  options: RunDshAdvisorReviewOptions,
): Promise<DshAdvisorReviewResult> {
  /**
   Mutable per-operation state carrying the folded progress line.
   */
  const progressState = {
    latest: '',
  };
  // Clear the degradation record so only this operation's acquisition state
  // remains for `/advisor status`.
  options
    .recordTranscriptDegradation('');
  /**
   Transcript port over one captured derived-history snapshot.
   */
  const transcript = createDshAdvisorTranscript({
    source: options.agent
      .session,
    onDegraded: options.recordTranscriptDegradation,
  },);
  /**
   Catalog and project context resolved in parallel before the run.
   */
  const [
    catalog,
    projectContext,
  ] = await Promise.all([
    buildDshAdvisorCatalog({
      llm: options.llm,
      sessionId: options.agent
        .sessionId,
      maxAdvisorOutputTokens: options.config
        .maxAdvisorOutputTokens,
      ...(options.signal === undefined
        ? {}
        : { signal: options.signal, }),
    },),
    buildDshAdvisorProjectContext({
      systemPrompt: options.systemPrompt,
      agent: options.agent,
      ...(options.signal === undefined
        ? {}
        : { signal: options.signal, }),
    },),
  ],);
  /**
   Canonical slug of the agent's conversation model route.
   */
  const currentModelSlug = (options.agent
    .options
    .provider
    === undefined)
    || (options.agent
      .options
      .model
      === undefined)
    ? undefined
    : `${options.agent
      .options
      .provider}/${options.agent
        .options
        .model}`;
  /**
   Display name of the conversation model when the catalog resolved that route,
   letting default avoidance recognize the same model on another route.
   */
  const currentModelName = currentModelSlug === undefined
    ? undefined
    : catalog.factsBySlug
      .get(currentModelSlug,)
      ?.model
      .name;
  /**
   Reviews and operation metadata collected by one Advisor operation.
   */
  const result = await runAdvisorCore({
    ctx: createDshAdvisorRunContext({
      catalog,
      transcript,
      cwd: options.agent
        .cwd,
      ...(currentModelSlug === undefined
        ? {}
        : {
          mainProvider: options.agent
            .options
            .provider,
          mainModel: options.agent
            .options
            .model,
          ...(currentModelName === undefined
            ? {}
            : { mainModelName: currentModelName, }),
        }),
    },),
    config: options.config,
    projectContext,
    rankDefaultCandidates: dshRankDefaultCandidates,
    ...(options.requestedSlug === undefined
      ? {}
      : { requestedSlug: options.requestedSlug, }),
    ...(options.question === undefined
      ? {}
      : { question: options.question, }),
    ...(options.toolCallId === undefined
      ? {}
      : { toolCallId: options.toolCallId, }),
    ...(options.signal === undefined
      ? {}
      : { signal: options.signal, }),
    onUpdate: function onUpdate(operation: AdvisorOperationSnapshot): void {
      progressState.latest = formatAdvisorProgress({
        operation,
        now: Date.now(),
      },);
      options
        .onUpdate
        ?.(operation,);
    },
  },);
  /**
   Operation ledger persisted for every completed operation.

   The session record stores rendered tool text only, so attempt diagnostics
   and usage survive in the durable `plugin:advisor/operation` record.
   */
  const {
    operation,
  } = result.details;
  if (operation !== undefined)
    options
      .agent
      .appendOperationRecord(operation,);
  return {
    result,
    progress: progressState.latest,
  };
}

//endregion Public API
