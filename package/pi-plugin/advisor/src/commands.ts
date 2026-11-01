/**
 Slash command registration for Advisor.
 
 @module
 */

import type {
  ExtensionAPI,
  ExtensionCommandContext,
} from '@earendil-works/pi-coding-agent';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type { ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  ADVISOR_OPERATION_TYPE,
  ADVISOR_TOOL_NAME,
  AdvisorOperationError,
  containsToolName,
  serializeAdvisorProjectContext,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import { sendAdvisorMessage, } from './command-message.ts';
import { buildAdvisorStatus, } from './status.ts';
import { runAdvisor, } from './tool.ts';
import type { AdvisorConfig, } from './types.ts';

//region Types

/**
 Session-scoped Advisor toggle controlled by `/advisor on` and `/advisor off`.
 
 Exposes a readonly handle pair (`getEnabled` / `setEnabled`) backed by a
 closure-private boolean so parameter types stay readonly while the toggle
 mutation remains intentional and centralised in {@link createAdvisorSessionState}.
 */
export type AdvisorSessionState = {
  /**
   Read current session-enable flag.
   */
  readonly getEnabled: () => boolean;
  /**
   Update session-enable flag.
   */
  readonly setEnabled: (enabled: boolean) => void;
};

/**
 Build an {@link AdvisorSessionState} handle around a private mutable flag.
 
 @param initialEnabled - starting toggle value
 
 @returns session-state handle with readonly getter and setter
 
 @example
 ```typescript
 const state = createAdvisorSessionState(true);
 ```
 */
export function createAdvisorSessionState(
  initialEnabled: boolean,
): AdvisorSessionState {
  /**
   Closure-private session-enable flag.
   */
  let enabled = initialEnabled;
  /**
   Handle exposing getter and setter over the closure-private flag.
   */
  const state: AdvisorSessionState = {
    getEnabled: function getEnabled() {
      return enabled;
    },
    setEnabled: function setEnabled(value: boolean) {
      enabled = value;
    },
  };
  return state;
}

/**
 Options for command registration.
 */
export type RegisterAdvisorCommandsOptions = {
  /**
   Pi extension API.
   */
  readonly pi: ForeignHostCapability<ExtensionAPI>;
  /**
   Runtime config accessor.
   */
  readonly getConfig: () => AdvisorConfig;
  /**
   Session enablement state.
   */
  readonly state: AdvisorSessionState;
};

//endregion Types

//region Public API

/**
 Register `/advisor` command and subcommands.
 
 @param options - pi API and runtime state
 
 @mutates options - `options.pi.registerCommand` stores command registration in Pi host state
 
 @example
 ```typescript
 registerAdvisorCommands({ pi, getConfig, state });
 ```
 */
export function registerAdvisorCommands(
  options: ForeignHostCapability<RegisterAdvisorCommandsOptions>,
): void {
  options.pi
    .registerCommand(
    ADVISOR_TOOL_NAME,
    {
      description:
        'Run Advisor, inspect scoped models, or toggle Advisor for this session',
      /**
       Execute Advisor slash command through Pi host context.
       
       @mutates ctx - command handling can update Pi notifications and session state
       */
      async handler(
        args,
        ctx: ForeignHostCapability<ExtensionCommandContext>,
      ) {
        await handleAdvisorCommand({
          args,
          ctx,
          pi: options.pi,
          getConfig: options.getConfig,
          state: options.state,
        },);
      },
    },
    );
}

/**
 Synchronize active tools with Advisor session state.
 
 @param pi - pi extension API
 
 @param enabled - whether Advisor should be active
 
 @mutates pi - `pi.setActiveTools` changes active Pi host tools when Advisor state differs
 
 @example
 ```typescript
 syncAdvisorActiveTool({ pi, enabled: false });
 ```
 */
export function syncAdvisorActiveTool(
  {
    pi,
    enabled,
  }: {
    readonly pi: ForeignHostCapability<ExtensionAPI>;
    readonly enabled: boolean;
  },
): void {
  /**
   Current active tool names.
   */
  const activeTools = pi.getActiveTools();
  /**
   Whether Advisor is active.
   */
  const alreadyActive = containsToolName({
    toolNames: activeTools,
    targetName: ADVISOR_TOOL_NAME,
  },);
  if (enabled && (!alreadyActive)) {
    pi.setActiveTools([
      ...activeTools,
      ADVISOR_TOOL_NAME,
    ],);
    return;
  }
  if ((!enabled) && alreadyActive) {
    /**
     Active tools excluding Advisor.
     */
    const retainedTools: string[] = [];
    for (const toolName of activeTools) {
      if (toolName !== ADVISOR_TOOL_NAME)
        retainedTools.push(toolName,);
    }
    pi.setActiveTools(retainedTools,);
  }
}

//endregion Public API

//region Handler

/**
 Options for the command handler.
 */
type HandleAdvisorCommandOptions = {
  /**
   Raw command args.
   */
  readonly args: string;
  /**
   Command context.
   */
  readonly ctx: ForeignHostCapability<ExtensionCommandContext>;
  /**
   Pi extension API.
   */
  readonly pi: ForeignHostCapability<ExtensionAPI>;
  /**
   Runtime config accessor.
   */
  readonly getConfig: () => AdvisorConfig;
  /**
   Mutable session state.
   */
  readonly state: AdvisorSessionState;
};

/**
 Handle one `/advisor` invocation.
 
 @param options - command handler inputs
 
 @mutates options - changes session toggle and invokes Pi active-tool and notification capabilities
 */
async function handleAdvisorCommand(
  options: ForeignHostCapability<HandleAdvisorCommandOptions>,
): Promise<void> {
  /**
   Trimmed command args.
   */
  const trimmed = options.args
    .trim();
  if (trimmed === 'status') {
    options
      .ctx
      .ui
      .notify(await buildAdvisorStatus({
        ctx: options.ctx,
        config: options.getConfig(),
        enabled: options.state
          .getEnabled(),
      },),);
    return;
  }

  if (trimmed === 'off') {
    options.state
      .setEnabled(false,);
    syncAdvisorActiveTool({
      pi: options.pi,
      enabled: false,
    },);
    options
      .ctx
      .ui
      .notify('Advisor disabled for this session.',);
    return;
  }

  if (trimmed === 'on') {
    options.state
      .setEnabled(true,);
    syncAdvisorActiveTool({
      pi: options.pi,
      enabled: true,
    },);
    options
      .ctx
      .ui
      .notify('Advisor enabled for this session.',);
    return;
  }

  if (!options.state
    .getEnabled()) {
    options
      .ctx
      .ui
      .notify(
      'Advisor is disabled for this session. Run /advisor on to re-enable.',
      'error',
      );
    return;
  }

  await runImmediateAdvisor({
    ctx: options.ctx,
    pi: options.pi,
    config: options.getConfig(),
    ...(trimmed === '' ? {} : { requestedSlug: trimmed, }),
  },);
}

/**
 Run immediate manual Advisor review and append a custom message.
 
 @mutates ctx - prompt-option access and `ctx.ui.notify` can update Pi host state
 
 @mutates pi - `sendAdvisorMessage` calls `pi.sendMessage` to append Advisor output
 */
async function runImmediateAdvisor(
  {
    ctx,
    pi,
    config,
    requestedSlug,
  }: {
    readonly ctx: ForeignHostCapability<ExtensionCommandContext>;
    readonly pi: ForeignHostCapability<ExtensionAPI>;
    readonly config: AdvisorConfig;
    readonly requestedSlug?: string;
  },
): Promise<void> {
  await ctx.waitForIdle();
  try {
    /**
     Current prompt options read after pending agent work settles.
     */
    const promptOptions = ctx.getSystemPromptOptions();
    /**
     Current Pi-loaded project context from authoritative prompt options.
     */
    const projectContext = serializeAdvisorProjectContext(
      promptOptions.contextFiles ?? [],
    );
    /**
     Manual Advisor review result.
     */
    const result = await runAdvisor({
      ctx,
      config,
      projectContext,
      ...(requestedSlug === undefined ? {} : { requestedSlug, }),
      ...(ctx.signal
        === undefined ? {} : { signal: ctx.signal, }),
    },);
    sendAdvisorMessage({
      pi,
      result,
    },);
  }
  catch (error) {
    if (error instanceof AdvisorOperationError)
      pi.appendEntry(
        ADVISOR_OPERATION_TYPE,
        error.operation,
      );
    if (ctx.signal
      ?.aborted
      === true) {
      ctx.ui
        .notify(
        'Advisor review cancelled.',
        'warning',
        );
      return;
    }
    ctx.ui
      .notify(
      `Advisor review failed: ${caughtValueText(error,)}`,
      'error',
      );
  }
}

//endregion Handler
