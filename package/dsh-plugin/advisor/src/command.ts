/**
 `/advisor` command: manual reviews, status inspection, and session toggles. @module
 */

import type {
  CommandDefinition,
  CommandResult,
} from '@deepseek-ai/dsh-commands';
import type {
  ToolRestriction,
} from '@deepseek-ai/dsh-tools';
import type {
  LlmProviderInfo,
} from '@deepseek-ai/dsh-llm';
import type {
  WorkingDirectoryService,
} from '@deepseek-ai/dsh-working-directory';
import { caughtValueText, } from '@monochromatic-dev/module-caught-value/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  ADVISOR_TOOL_NAME,
  AdvisorOperationError,
  type AdvisorConfig,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import type {
  ForeignBorrowed,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { toDshAdvisorAgentSurface, } from './agent-surface.ts';
import { buildDshAdvisorCatalog, } from './catalog.ts';
import type {
  DshAdvisorSessionState,
} from './config-state.ts';
import { dshAdvisorLogger, } from './logging.ts';
import type {
  DshAdvisorSystemPromptSurface,
} from './project-context.ts';
import { runDshAdvisorReview, } from './run-review.ts';
import { buildDshAdvisorStatus, } from './status.ts';
import type {
  DshAdvisorAgentSurface,
  DshAdvisorLlmSurface,
} from './types.ts';

//region Types

/**
 Host accessors the `/advisor` command reads at invocation time.
 */
export type CreateDshAdvisorCommandOptions = {
  /**
   Merged config resolver taking the session's working directory.
   */
  readonly getConfig: (cwd: string) => Promise<AdvisorConfig>;
  /**
   Session state owning the toggle, degradation record, and tool restriction.
   */
  readonly state: DshAdvisorSessionState;
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
  /**
   Tool registry surface used to restrict the Advisor tool when toggled off.
   */
  readonly tools: {
    /**
     Apply a global tool mask; the returned disposer lifts it.
     */
    readonly restrict: (filter: ToolRestriction) => () => void;
  };
};

/**
 One command invocation projected for the handler core.
 */
export type RunDshAdvisorCommandOptions = CreateDshAdvisorCommandOptions & {
  /**
   Exact text following the command name, including separator whitespace.
   */
  readonly rawInput: string;
  /**
   Dispatching UI cancellation.
   */
  readonly signal: AbortSignal;
  /**
   Agent surface of the receiving session.
   */
  readonly agent: DshAdvisorAgentSurface;
};

//endregion Types

//region Review runner

/**
 Run one immediate `/advisor` review and normalize its outcome.

 Mirrors pi's `runImmediateAdvisor`: pending agent work settles first
 (`whenIdle`), cancellation reports as `Advisor review cancelled.`, and a
 failed operation persists its ledger before the error text is returned.

 @param options - command accessors and projected invocation

 @param requestedSlug - exact reviewer request, absent for the default form

 @returns command result carrying the review text or the failure line
 */
async function runAdvisorCommandReview(
  {
    options,
    requestedSlug,
  }: {
    readonly options: RunDshAdvisorCommandOptions;
    readonly requestedSlug?: string;
  },
): Promise<CommandResult> {
  /**
   Logger tagged with the review runner name.
   */
  const l = tagged({
    tag: runAdvisorCommandReview.name,
    l: dshAdvisorLogger,
  },);
  await options
    .agent
    .whenIdle();
  try {
    /**
     Reviews and operation metadata for this manual review.
     */
    const { result, } = await runDshAdvisorReview({
      config: await options
        .getConfig(options.agent
          .cwd,),
      llm: options.llm,
      systemPrompt: options.systemPrompt,
      agent: options.agent,
      recordTranscriptDegradation: options
        .state
        .recordTranscriptDegradation,
      signal: options.signal,
      ...(requestedSlug === undefined
        ? {}
        : { requestedSlug, }),
    },);
    return {
      kind: 'success',
      text: result.text,
    };
  }
  catch (error) {
    if (error instanceof AdvisorOperationError) {
      options
        .agent
        .appendOperationRecord(error.operation,);
    }
    l.warn(`advisor command review failed: ${caughtValueText(error,)}`,);
    if (options.signal
      .aborted)
      return {
        kind: 'error',
        text: 'Advisor review cancelled.',
      };
    return {
      kind: 'error',
      text: `Advisor review failed: ${caughtValueText(error,)}`,
    };
  }
}

//endregion Review runner

//region Public API

/**
 Run one `/advisor` form and normalize its outcome.

 Accepted forms after the command name: empty for a default review, a model
 slug for an exact review, `status`, `off`, and `on`. There is no `set` form:
 persistent model overrides would bypass scoped tool-parameter selection.
 Reviews settle as result text (no custom message channel exists), and
 cancellation reports as `Advisor review cancelled.`

 @param options - host accessors and the projected invocation

 @returns command result carrying the rendered text

 @throws never; review failures settle as `{ kind: 'error' }` text

 @example
 ```ts
 const result = await runDshAdvisorCommand({ getConfig, state, llm, systemPrompt, workingDirectory, tools, rawInput: 'status', signal, agent });
 ```
 */
export async function runDshAdvisorCommand(
  options: RunDshAdvisorCommandOptions,
): Promise<CommandResult> {
  /**
   Logger tagged with the command core name.
   */
  const l = tagged({
    tag: runDshAdvisorCommand.name,
    l: dshAdvisorLogger,
  },);
  /**
   Trimmed form selector following the command name.
   */
  const trimmed = options
    .rawInput
    .trim();
  l.debug(`advisor command form=${trimmed === ''
    ? 'default'
    : trimmed}`);
  if (trimmed
    === 'status') {
    /**
     Merged config for the session's project, read once for both status
     inputs.
     */
    const config = await options
      .getConfig(options.agent
        .cwd,);
    /**
     Catalog snapshot serving the status scope and capacity lines.
     */
    const catalog = await buildDshAdvisorCatalog({
      llm: options.llm,
      sessionId: options.agent
        .sessionId,
      maxAdvisorOutputTokens: config.maxAdvisorOutputTokens,
      signal: options.signal,
    },);
    return {
      kind: 'success',
      text: await buildDshAdvisorStatus({
        catalog,
        providerIds: options
          .llm
          .listProviders()
          .map(function readProviderId(
            provider: ForeignBorrowed<LlmProviderInfo>,
          ) {
            return provider.id;
          },),
        cwd: options.agent
          .cwd,
        config,
        enabled: options
          .state
          .getEnabled(),
        transcriptDegradationReason: options
          .state
          .getTranscriptDegradationReason(),
        ...((options.agent
          .options
          .provider
          === undefined)
          || (options.agent
            .options
            .model
            === undefined)
          ? {}
          : {
            mainProvider: options.agent
              .options
              .provider,
            mainModel: options.agent
              .options
              .model,
          }),
      },),
    };
  }
  if (trimmed
    === 'off') {
    options
      .state
      .setEnabled(false,);
    options
      .state
      .adoptToolRestriction(
      options
        .tools
        .restrict({
          deny: [ADVISOR_TOOL_NAME,],
        },),
      );
    return {
      kind: 'success',
      text: 'Advisor disabled for this session.',
    };
  }
  if (trimmed
    === 'on') {
    options
      .state
      .setEnabled(true,);
    options
      .state
      .clearToolRestriction();
    return {
      kind: 'success',
      text: 'Advisor enabled for this session.',
    };
  }
  if (!options
    .state
    .getEnabled())
    return {
      kind: 'error',
      text: 'Advisor is disabled for this session. Run /advisor on to re-enable.',
    };
  return await runAdvisorCommandReview({
    options,
    ...(trimmed === ''
      ? {}
      : { requestedSlug: trimmed, }),
  },);
}

/**
 Build the `/advisor` command definition.

 @param commandOptions - config, session state, and host service surfaces

 @returns registry-ready command definition

 @example
 ```ts
 ctx.commands.register(createDshAdvisorCommand({ getConfig, state, llm, systemPrompt, workingDirectory, tools }));
 ```
 */
export function createDshAdvisorCommand(
  commandOptions: CreateDshAdvisorCommandOptions,
): CommandDefinition {
  return {
    name: ADVISOR_TOOL_NAME,
    description: 'Run Advisor, inspect scoped models, or toggle Advisor for this session',
    input: {
      hint: '[status|off|on|<model slug>]',
    },
    async handler(invocation): Promise<CommandResult> {
      return await runDshAdvisorCommand({
        ...commandOptions,
        rawInput: invocation.rawInput,
        signal: invocation.signal,
        agent: toDshAdvisorAgentSurface({
          agent: invocation.agent,
          workingDirectory: commandOptions.workingDirectory,
        },),
      },);
    },
  };
}

//endregion Public API
