/**
 Pi Advisor extension entry point.
 
 Registers an `advisor` tool and `/advisor` command that consult a scoped
 secondary reviewer model using serialized conversation context.
 
 @module
 */

import type {
  BeforeAgentStartEvent,
  ExtensionAPI,
  ExtensionContext,
  MessageRenderOptions,
  Theme,
} from '@earendil-works/pi-coding-agent';
import type { ReadonlyDeep, } from 'type-fest';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type {
  ForeignBorrowed,
  ForeignHostCapability,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  ADVISOR_MESSAGE_TYPE,
  buildAdvisorSystemPrompt,
  createAdvisorProjectContextState,
  loadMergedConfig,
  MAIN_MODEL_GUIDANCE_PREFIX,
  selectAdvisorModel,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import { resolveEffectiveScope, } from '@monochromatic-dev/agent-harness-shared-model-selection/ts';
import {
  createAdvisorSessionState,
  registerAdvisorCommands,
  syncAdvisorActiveTool,
} from './commands.ts';
import { registerAdvisorFailureAccounting, } from './operation-failure-accounting.ts';
import { renderAdvisorMessage, } from './rendering.ts';
import { createAdvisorTool, } from './tool.ts';

/**
 Logger root for pi-advisor after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'pi-advisor', },);

/**
 Tagged logger for the Advisor entry point.
 */
const l = tagged({
  tag: 'index',
  l: parentLogger,
},);

//region Extension entry point

/**
 Advisor pi extension.
 
 @param pi - pi extension API
 
 @mutates pi - `pi.registerTool`, `pi.registerMessageRenderer`, `pi.on`, and delegated `pi.registerCommand` change Pi host registrations
 
 @example
 ```typescript
 // In ~/.pi/agent/settings.json:
 { "packages": ["./packages/pi-plugin/advisor"] }
 ```
 */
export default async function advisor(
  pi: ForeignHostCapability<ExtensionAPI>,
): Promise<void> {
  /**
   Logger tagged with the extension factory name.
   */
  const innerL = tagged({
    tag: advisor.name,
    l,
  },);
  /**
   Runtime config loaded at extension startup.
   */
  const config = await loadMergedConfig({ cwd: process.cwd(), },);
  /**
   Mutable session state controlled by `/advisor on` and `/advisor off`.
   */
  const state = createAdvisorSessionState(config.enabled,);
  /**
   Pi-loaded project context retained across compact-and-continue runs.
   */
  const projectContextState = createAdvisorProjectContextState();

  innerL.debug(`advisor extension loaded; enabled=${String(state.getEnabled(),)}`,);

  /**
   Restore failed nested usage when Pi finalizes the thrown tool error.
   */
  const onFailure = registerAdvisorFailureAccounting(pi,);
  pi.registerTool(createAdvisorTool({
    onFailure,
    getConfig: function getConfig() {
      return config;
    },
    getSessionEnabled: function getSessionEnabled() {
      return state.getEnabled();
    },
    getProjectContext: projectContextState.get,
  },),);

  registerAdvisorCommands({
    pi,
    getConfig: function getConfig() {
      return config;
    },
    state,
  },);

  pi.registerMessageRenderer(
    ADVISOR_MESSAGE_TYPE,
    /**
     Render manual Advisor message through Pi theme capability.
     
     @param message - custom Advisor message payload
     
     @param options - Pi transcript expansion state
     
     @param theme - Pi theme used to style message
     
     @returns styled Advisor message component
     
     @mutates theme - theme methods can update Pi host styling caches
     */
    function renderMessage(
      message: ReadonlyDeep<{
        content: unknown;
        details?: unknown;
      }>,
      options: Readonly<MessageRenderOptions>,
      theme: ForeignHostCapability<Theme>,
    ) {
      return renderAdvisorMessage({
        message,
        expanded: options.expanded,
        theme,
      },);
    },
  );

  pi.on(
    'session_start',
    function handleSessionStart() {
      projectContextState.clear();
      syncAdvisorActiveTool({
        pi,
        enabled: state.getEnabled(),
      },);
    },
  );

  pi.on(
    'before_agent_start',
    /**
     Adds Advisor guidance before each enabled agent turn.
     
     @param event - Pi prompt event.
     
     @param ctx - Pi extension context.
     
     @returns Prompt replacement when Advisor is enabled.
     
     @mutates ctx - `buildMainModelGuidance` invokes context scope and model-registry callbacks
     */
    async function handleBeforeAgentStart(
      event: ForeignBorrowed<BeforeAgentStartEvent>,
      ctx: ForeignHostCapability<ExtensionContext>,
    ) {
      /**
       Authoritative loaded context files for current agent run.
       */
      const { contextFiles, } = event.systemPromptOptions;
      projectContextState.replace(contextFiles,);
      if (!state.getEnabled())
        return undefined;

      /**
       Advisor guidance appended to the main model system prompt.
       */
      const guidance = await buildMainModelGuidance({
        ctx,
        config,
      },);
      return {
        systemPrompt: `${event.systemPrompt}\n\n${guidance}`,
      };
    },
  );
}

//endregion Extension entry point

//region Prompt guidance

/**
 Build dynamic main-model guidance for Advisor.
 
 @param ctx - pi extension context
 
 @param config - runtime Advisor config
 
 @returns prompt text appended to main model system prompt
 
 @mutates ctx - `resolveEffectiveScope` invokes context live-scope and model-registry callbacks
 
 @example
 ```typescript
 buildMainModelGuidance({ ctx, config });
 ```
 */
async function buildMainModelGuidance(
  {
    ctx,
    config,
  }: {
    readonly ctx: ForeignHostCapability<ExtensionContext>;
    readonly config: Awaited<ReturnType<typeof loadMergedConfig>>;
  },
): Promise<string> {
  /**
   Effective scoped model set.
   */
  const scope = await resolveEffectiveScope({
    ctx,
    errorPrefix: 'advisor',
  },);
  /**
   Default model for empty Advisor params.
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
   Canonical slugs available to Advisor.
   */
  const scopedSlugs = scope.entries
    .map(function mapEntry(
      entry: ReadonlyDeep<(typeof scope.entries)[number]>,
    ) {
      return entry.canonicalSlug;
    },);

  return [
    MAIN_MODEL_GUIDANCE_PREFIX,
    `Allowed Advisor model slugs: ${
      scopedSlugs.length
        === 0 ? 'none' : scopedSlugs.join(', ',)
    }`,
    `advisor({}) default model: ${defaultSelection?.selected
      .canonicalSlug
      ?? 'none'}`,
    `Advisor prompt: ${buildAdvisorSystemPrompt(config,)
      .split('\n',)[0]}`,
  ]
    .join('\n',);
}

//endregion Prompt guidance

/**
 Operation internals exported for built-artifact verification. @internal
 */
export { createAdvisorOperationLedger, } from '@monochromatic-dev/agent-harness-shared-advisor/ts';
export { runAdvisorOperation, } from '@monochromatic-dev/agent-harness-shared-advisor/ts';
export { formatAdvisorProgress, } from '@monochromatic-dev/agent-harness-shared-advisor/ts';
export {
  renderAdvisorOperationSummary,
  NO_ADVISOR_OPERATION_SUMMARY,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
export { createAdvisorTool, } from './tool.ts';
export { registerAdvisorFailureAccounting, } from './operation-failure-accounting.ts';
export { AdvisorOperationError, } from '@monochromatic-dev/agent-harness-shared-advisor/ts';
export { AdvisorCompletionError, } from '@monochromatic-dev/agent-harness-shared-advisor/ts';
export type { AdvisorOperationOptions, } from '@monochromatic-dev/agent-harness-shared-advisor/ts';
export type {
  AdvisorDispatch,
  AdvisorOperationCandidate,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
export {
  NO_ADVISOR_CANDIDATE,
  nextAdvisorCandidate,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
export {
  assertAdvisorProviderAvailable,
  isAdvisorCreditExhaustion,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
export {
  ADVISOR_CLOCK_BOUNDARY,
  createAdvisorCancellation,
  waitForAdvisorEvent,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
export {
  aggregateAdvisorUsage,
  copyAdvisorUsage,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
export type {
  AdvisorAttemptRecord,
  AdvisorOperationSnapshot,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';

export { buildAdvisorStatus, } from './status.ts';
export { buildMainModelGuidance, };

/**
 Internal project-context helpers exported for built-artifact verification.
 
 @internal
 */
export {
  createAdvisorProjectContextState,
  serializeAdvisorProjectContext,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';

/**
 Internal provider-call behavior exported for built-artifact verification.
 
 @internal
 */
export {
  buildAdvisorSystemPromptForProject,
  type CompleteAdvisorModel,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';

/**
 Internal request formatter exported for built-artifact verification.
 
 @internal
 */
export { buildAdvisorUserMessageText, } from '@monochromatic-dev/agent-harness-shared-advisor/ts';
export { completeAdvisor, } from '@monochromatic-dev/agent-harness-shared-advisor/ts';

/**
 Internal config helpers exported for built-artifact verification.
 
 @internal
 */
export {
  DEFAULT_CONFIG,
  loadMergedConfig,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';

/**
 Internal extension constants exported for built-artifact verification.
 
 @internal
 */
export {
  ADVISOR_MESSAGE_TYPE,
  ADVISOR_TOOL_NAME,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';

/**
 Internal context helpers exported for built-artifact verification.
 
 @internal
 */
export {
  buildAdvisorContext,
  maxContextCharsForAdvisorModel,
  truncateContext,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';

/**
 Internal rendering helper exported for built-artifact verification.
 
 @internal
 */
export { renderAdvisorResult, } from './rendering.ts';
export { firstAdvisoryLine, } from '@monochromatic-dev/agent-harness-shared-advisor/ts';

/**
 Internal selection helpers exported for built-artifact verification.
 
 @internal
 */
export { selectAdvisorModel, } from '@monochromatic-dev/agent-harness-shared-advisor/ts';
export { selectAdvisorRunContext, } from '@monochromatic-dev/agent-harness-shared-advisor/ts';

/**
 Internal tool argument adapter exported for built-artifact verification.
 
 @internal
 */
export { prepareAdvisorArguments, } from '@monochromatic-dev/agent-harness-shared-advisor/ts';

/**
 Internal Advisor execution exported for built-artifact verification.
 
 @internal
 */
export { runAdvisor, } from './tool.ts';

/**
 Internal Advisor data types exported for built-artifact verification.
 
 @internal
 */
export type {
  AdvisorConfig,
  AdvisorContext,
  AdvisorDetails,
  EffectiveModelScope,
} from './types.ts';
