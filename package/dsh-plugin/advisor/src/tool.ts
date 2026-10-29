/**
 Advisor tool definition: sequential reviewer consultation over the shared
 Advisor core. @module
 */

import {
  defineTool,
  type ParameterSchemaSpec,
  type ToolDefinition,
} from '@deepseek-ai/dsh-tools';
import type {
  ContentBlock,
} from '@deepseek-ai/dsh-llm';
import type {
  JsonValue,
} from '@deepseek-ai/dsh-util-values';
import type {
  WorkingDirectoryService,
} from '@deepseek-ai/dsh-working-directory';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  ADVISOR_TOOL_NAME,
  AdvisorOperationError,
  prepareAdvisorArguments,
  type AdvisorConfig,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import { toDshAdvisorAgentSurface, } from './agent-surface.ts';
import { dshAdvisorLogger, } from './logging.ts';
import type {
  DshAdvisorSystemPromptSurface,
} from './project-context.ts';
import { runDshAdvisorReview, } from './run-review.ts';
import type {
  DshAdvisorAgentSurface,
  DshAdvisorLlmSurface,
} from './types.ts';
import {
  advisorDetailsAsJsonValue,
  renderAdvisorToolContent,
  type DshAdvisorToolValue,
} from './tool-value.ts';

//region Tool prompts

/**
 Model-facing tool description.

 The default-reviewer clause names the DSH ranking rule (largest context
 window) instead of pi's expected-cost wording, because the DSH variant
 ranks through the shared ranking seam.
 */
export const ADVISOR_TOOL_DESCRIPTION = 'Consult independent scoped reviewer models using the current conversation context. Empty params start with the largest-context-window eligible non-current model and recover through other scoped models on failure. Optional configured overlap collects completed reviews together after bounded straggler grace. Explicit model requests never switch models.';

/**
 System-prompt section text carrying pi's `promptSnippet` and four
 `promptGuidelines` into the DSH prompt registry.
 */
export const ADVISOR_TOOL_PROMPT_SECTION_TEXT = `Consult Advisor with empty params for default recovery, or specify an exact scoped model and optional focus question.
Advisor receives the current serialized conversation and loaded project context automatically.
Call Advisor to check flawed assumptions, missing verification, and overlooked files.
Advisor may return multiple separately labelled reviews; none carries a quality guarantee.
Explicit Advisor model requests remain exact and never fall back to another model.`;

/**
 Placement of the Advisor tool-guidance prompt section.
 */
export const ADVISOR_TOOL_PROMPT_SECTION = {
  /**
   Unique prompt-section name in the host registry.
   */
  name: 'tool:advisor',
  /**
   Sort order between the goal and workflow tool sections.
   */
  order: 2_500,
} as const;

//endregion Tool prompts

//region Types

/**
 Host accessors the Advisor tool reads at execution time.
 */
export type CreateDshAdvisorToolOptions = {
  /**
   Merged config resolver taking the session's working directory.
   */
  readonly getConfig: (cwd: string) => Promise<AdvisorConfig>;
  /**
   Session enablement accessor.
   */
  readonly getSessionEnabled: () => boolean;
  /**
   Degradation recorder shared with `/advisor status`.
   */
  readonly recordTranscriptDegradation: (reason: string) => void;
  /**
   DSH LLM surface serving route catalog and streaming.
   */
  readonly llm: DshAdvisorLlmSurface;
  /**
   System-prompt registry surface for project context.
   */
  readonly systemPrompt: DshAdvisorSystemPromptSurface;
  /**
   Working-directory service resolving the session's project directory.
   */
  readonly workingDirectory: Pick<WorkingDirectoryService, 'get'>;
};

/**
 One tool call projected for the execution core.
 */
export type ExecuteDshAdvisorToolCallOptions = CreateDshAdvisorToolOptions & {
  /**
   Validated tool arguments, however malformed beyond the schema.
   */
  readonly args: unknown;
  /**
   Tool call id recorded on the operation and used to omit placeholder
   context.
   */
  readonly callId: string;
  /**
   Caller cancellation.
   */
  readonly signal: AbortSignal;
  /**
   Agent surface of the calling session; absent fails loud.
   */
  readonly agent?: DshAdvisorAgentSurface;
};

//endregion Types

//region Public API

/**
 Execute one Advisor tool call and return its canonical JSON value.

 @param options - host accessors and the projected tool call

 @returns review text, `AdvisorDetails`, and the folded progress line

 @throws when the session toggle is off or the call has no agent-backed
 session

 @example
 ```ts
 const value = await executeDshAdvisorToolCall({ getConfig, getSessionEnabled, recordTranscriptDegradation, llm, systemPrompt, workingDirectory, args: {}, callId: 'call-1', signal, agent });
 ```
 */
export async function executeDshAdvisorToolCall(
  options: ExecuteDshAdvisorToolCallOptions,
): Promise<DshAdvisorToolValue> {
  /**
   Logger tagged with the execution core name.
   */
  const l = tagged({
    tag: executeDshAdvisorToolCall.name,
    l: dshAdvisorLogger,
  },);
  if (!options
    .getSessionEnabled())
    throw new Error(
      'advisor: disabled for this session. Run /advisor on to re-enable.',
    );
  if (options.agent
    === undefined)
    throw new Error(
      'advisor: this tool call has no agent-backed session; the review needs the derived conversation history from the agent session',
    );
  /**
   Normalized tool parameters after shared semantic validation.
   */
  const params = prepareAdvisorArguments(options.args,);
  l.debug(
    `advisor tool call ${options.callId} starting; model=${params.model
      ?? 'default'} question=${params.question === undefined
      ? 'none'
      : 'set'}`,
  );
  try {
    /**
     Reviews, metadata, and the folded progress line for this call.
     */
    const {
      result,
      progress,
    } = await runDshAdvisorReview({
      config: await options
        .getConfig(options.agent
          .cwd,),
      llm: options.llm,
      systemPrompt: options.systemPrompt,
      agent: options.agent,
      recordTranscriptDegradation: options.recordTranscriptDegradation,
      toolCallId: options.callId,
      signal: options.signal,
      ...(params.model === undefined
        ? {}
        : { requestedSlug: params.model, }),
      ...(params.question === undefined
        ? {}
        : { question: params.question, }),
    },);
    return {
      text: result.text,
      details: advisorDetailsAsJsonValue(result.details,),
      progress,
    };
  }
  catch (error) {
    if (error instanceof AdvisorOperationError) {
      options.agent
        .appendOperationRecord(error.operation,);
    }
    l.warn(
      `advisor tool call ${options.callId} failed: ${caughtValueText(error,)}`,
    );
    throw error;
  }
}

//region Tool surface callbacks

/**
 Callbacks the tool definition passes to the tool runtime.
 */
type AdvisorToolCallbacks = {
  /**
   Whether the runtime may run this tool concurrently with itself.
   */
  readonly isConcurrencySafe: () => boolean;
  /**
   Result renderer producing model-facing text blocks.
   */
  readonly render: (
    args: unknown,
    value: JsonValue,
  ) => ContentBlock[];
};

/**
 Tool-runtime callbacks the definition passes through unchanged.

 The callbacks live at module scope inside one annotated object literal so
 their runtime-supplied positional signatures stay intact, no nested scope is
 introduced, and the property-style function types keep every reference
 bound.
 */
const advisorToolCallbacks: AdvisorToolCallbacks = {
  /**
   Whether advisor tool calls may run concurrently. Advisor reviews share one
   session's sequential review slot, so the tool never reports
   concurrency-safe execution (parity with pi's sequential consultation).
   */
  isConcurrencySafe: function isConcurrencySafe(): boolean {
    return false;
  },
  /**
   Render one canonical advisor tool value as text content blocks.

   @param args - tool arguments, unused by the renderer

   @param value - canonical JSON value, however stale or malformed on replay

   @returns text content blocks for the model-facing tool result
   */
  render: function renderAdvisorToolResult(
    args: unknown,
    value: JsonValue,
  ): ContentBlock[] {
    void args;
    return renderAdvisorToolContent(value,);
  },
};

//endregion Tool surface callbacks

/**
 Build the Advisor tool definition.

 Calls run sequentially (sibling overlap disabled) matching pi's
 `executionMode: 'sequential'`, because default recovery mutates shared
 operation state.

 @param toolOptions - config, session state, and host service surfaces

 @returns registry-ready tool definition

 @example
 ```ts
 ctx.tools.register(createDshAdvisorTool({ getConfig, getSessionEnabled, recordTranscriptDegradation, llm, systemPrompt, workingDirectory }));
 ```
 */
export function createDshAdvisorTool(
  toolOptions: CreateDshAdvisorToolOptions,
): ToolDefinition {
  return defineTool({
    name: ADVISOR_TOOL_NAME,
    description: ADVISOR_TOOL_DESCRIPTION,
    parameters: {
      model: {
        type: 'string',
        description: 'Exact scoped reviewer model as canonical `provider/model` slug, bare model id, or unique model name. Omit to let default recovery pick the initial reviewer and fall back across scoped models; an explicit value never switches models.',
      },
      question: {
        type: 'string',
        description: 'Focused question for the reviewers to answer against the conversation context. Omit to request a general review of the conversation.',
      },
    } satisfies ParameterSchemaSpec,
    isConcurrencySafe: advisorToolCallbacks.isConcurrencySafe,
    output: {
      schema: {
        type: 'json',
      },
      render: advisorToolCallbacks.render,
    },
    /**
     Execute one Advisor review for this tool call and return its canonical
     value.

     @param args - validated tool arguments

     @param exec - runtime execution context for this call

     @returns canonical JSON value carrying text, details, and progress

     @throws when Advisor is disabled for the session or the call has no
     agent-backed session
     */
    async execute(
      args,
      exec,
    ) {
      return await executeDshAdvisorToolCall({
        ...toolOptions,
        args,
        callId: String(exec.callId,),
        signal: exec.signal,
        ...(exec.agent === undefined
          ? {}
          : {
            agent: toDshAdvisorAgentSurface({
              agent: exec.agent,
              workingDirectory: toolOptions.workingDirectory,
            },),
          }),
      },);
    },
  },);
}

//endregion Public API
