/**
 Advisor tool registration and ledger-driven execution. @module
 */
import type {
  AgentToolResult,
  ExtensionContext,
  Theme,
  ToolRenderResultOptions,
} from '@earendil-works/pi-coding-agent';
import type { ReadonlyDeep, } from 'type-fest';
import type { ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  ADVISOR_TOOL_NAME,
  AdvisorOperationError,
  AdvisorToolParametersSchema,
  formatAdvisorProgress,
  prepareAdvisorArguments,
  runAdvisor as runAdvisorCore,
  type AdvisorOperationSnapshot,
  type AdvisorRunOptions,
  type AdvisorRunResult,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import {
  renderAdvisorCall,
  renderAdvisorResult,
} from './rendering.ts';
import {
  toAdvisorRunContext,
  type PiAdvisorRunContextSource,
} from './run-context.ts';
import type {
  AdvisorConfig,
  AdvisorDetails,
  AdvisorProgressDetails,
  AdvisorToolDefinition,
  AdvisorToolResult,
} from './types.ts';

//region Host run adapter

/**
 Pi-flavored Advisor run options: the extension context stands in for the
 shared run-context ports.
 */
export type PiAdvisorRunOptions = Omit<AdvisorRunOptions, 'ctx'> & {
  /**
   Pi extension context carrying registry, session, and scope facts.
   */
  readonly ctx: PiAdvisorRunContextSource;
};

/**
 Run Advisor from a pi extension context.
 
 @param options - pi-flavored run options
 
 @returns collected reviews and complete available accounting
 
 @example
 ```ts
 const result = await runAdvisor({ ctx, config });
 ```
 */
export async function runAdvisor(
  options: PiAdvisorRunOptions,
): Promise<AdvisorRunResult> {
  return await runAdvisorCore({
    ...options,
    ctx: toAdvisorRunContext(options.ctx,),
  },);
}

//endregion Host run adapter

/**
 Runtime state and persistence boundaries supplied by the extension.
 */
export type CreateAdvisorToolOptions = {
  /**
   Runtime config snapshot.
   */
  readonly getConfig: () => AdvisorConfig;
  /**
   Session enablement.
   */
  readonly getSessionEnabled: () => boolean;
  /**
   Current Pi-loaded project context.
   */
  readonly getProjectContext: () => string;
  /**
   Preserve accounting before throwing through the host's error boundary.
   */
  readonly onFailure?: (failure: {
    readonly toolCallId: string;
    readonly operation: AdvisorOperationSnapshot
  }) => void;
};

/**
 Host-defined positional progress callback, including absence outside streaming hosts.
 */
type AdvisorUpdateCallback = Parameters<AdvisorToolDefinition<typeof AdvisorToolParametersSchema>['execute']>[3];

/**
 Register a default fallback tool while preserving exact explicit-model requests.
 
 @param toolOptions - session accessors and accounting persistence
 
 @returns Pi tool definition
 
 @example
 ```ts
 pi.registerTool(createAdvisorTool({ getConfig, getSessionEnabled, getProjectContext }));
 ```
 */
export function createAdvisorTool(toolOptions: CreateAdvisorToolOptions,): AdvisorToolDefinition<typeof AdvisorToolParametersSchema> {
  return {
    name: ADVISOR_TOOL_NAME,
    label: 'Advisor',
    description: 'Consult independent scoped reviewer models using the current conversation context. Empty params start with the highest expected-cost eligible non-current model and recover through other scoped models on failure. Optional configured overlap collects completed reviews together after bounded straggler grace. Explicit model requests never switch models.',
    promptSnippet: 'Consult Advisor with empty params for default recovery, or specify an exact scoped model and optional focus question.',
    promptGuidelines: [
      'Advisor receives the current serialized conversation and loaded project context automatically.',
      'Call Advisor to check flawed assumptions, missing verification, and overlooked files.',
      'Advisor may return multiple separately labelled reviews; none carries a quality guarantee.',
      'Explicit Advisor model requests remain exact and never fall back to another model.',
    ],
    parameters: AdvisorToolParametersSchema,
    executionMode: 'sequential',
    prepareArguments: prepareAdvisorArguments,
    /**
     Run through one operation and expose bounded metadata-only progress.
     @param toolCallId - host identity for persisted failure accounting
     @param params - model and focus question
     @param signal - caller cancellation
     @param onUpdate - host progress callback
     @param ctx - scope, session, and provider capabilities
     @returns original collected reviews and aggregate available usage
     @mutates ctx - resolves scope and provider authentication through the host
     @mutates onUpdate - publishes progress to the host
     */
    async execute(
      toolCallId: string,
      params: {
        readonly model?: string;
        readonly question?: string
      },
      // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- Pi dictates this optional positional signal before required host context; intersecting the entire optional type with a marker would erase undefined.
      signal: ForeignHostCapability<AbortSignal> | undefined,
      onUpdate: AdvisorUpdateCallback,
      ctx: ForeignHostCapability<ExtensionContext>,
    ): Promise<AdvisorToolResult> {
      if (!toolOptions.getSessionEnabled())
        throw new Error('advisor: disabled for this session. Run /advisor on to re-enable.',);
      try {
        /**
         Final reviews and operation metadata.
         */
        const result = await runAdvisor({
          ctx,
          config: toolOptions.getConfig(),
          projectContext: toolOptions.getProjectContext(),
          toolCallId,
          ...(params.model === undefined ? {} : { requestedSlug: params.model, }),
          ...(params.question === undefined ? {} : { question: params.question, }),
          ...(signal === undefined ? {} : { signal, }),
          onUpdate(operation): void {
            onUpdate?.({
              content: [{
                type: 'text',
                text: formatAdvisorProgress({
                  operation,
                  now: Date.now(),
                },),
              },],
              details: { kind: 'advisor-progress', },
            },);
          },
        },);
        return {
          content: [{
            type: 'text',
            text: result.text,
          },],
          details: result.details,
          ...(result.details
            .usage
            === undefined ? {} : { usage: result.details
              .usage, }),
        };
      }
      catch (error) {
        if (error instanceof AdvisorOperationError)
          toolOptions.onFailure?.({
            toolCallId,
            operation: error.operation,
          },);
        throw error;
      }
    },
    /**
     Render the initial requested identity through the host theme.
     */
    renderCall(
      args: {
        readonly model?: string;
        readonly question?: string
      },
      theme: ForeignHostCapability<Theme>,
    ) {
      return renderAdvisorCall({
        args,
        theme,
      },);
    },
    /**
     Render partial metadata independently from final successful or failed results.
     */
    renderResult(
      result: ReadonlyDeep<AgentToolResult<AdvisorDetails | AdvisorProgressDetails>>,
      renderOptions: ReadonlyDeep<ToolRenderResultOptions>,
      theme: ForeignHostCapability<Theme>,
    ) {
      return renderAdvisorResult({
        result,
        expanded: renderOptions.expanded,
        isPartial: renderOptions.isPartial,
        theme,
      },);
    },
  };
}
