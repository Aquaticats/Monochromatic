/**
 Cordis plugin entry for the DSH Advisor variant: registers the `advisor`
 tool, the `/advisor` command, and the Advisor tool-guidance prompt section
 over the shared Advisor core. @module
 */

import type {
  Context,
} from '@deepseek-ai/cordis';
import z from '@deepseek-ai/schemastery';
import type {
  AdvisorConfig,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  createDshAdvisorSessionState,
  loadDshAdvisorConfig,
} from './config-state.ts';
import {
  createDshAdvisorCommand,
} from './command.ts';
import { dshAdvisorLogger, } from './logging.ts';
import {
  ADVISOR_TOOL_PROMPT_SECTION,
  ADVISOR_TOOL_PROMPT_SECTION_TEXT,
  createDshAdvisorTool,
} from './tool.ts';

export * from './agent-surface.ts';
export * from './catalog.ts';
export * from './command.ts';
export * from './config-state.ts';
export * from './constants.ts';
export * from './conversation-text.ts';
export * from './logging.ts';
export * from './message-map.ts';
export * from './provider-stream-chunk.ts';
export * from './provider-stream-finish.ts';
export * from './provider-stream.ts';
export * from './provider.ts';
export * from './ranking-policy.ts';
export * from './reasoning-effort.ts';
export * from './request-map.ts';
export * from './run-context.ts';
export * from './run-review.ts';
export * from './session-event.ts';
export * from './status.ts';
export * from './stream-driver.ts';
export * from './theme.ts';
export * from './tool-value.ts';
export * from './tool.ts';
export * from './transcript.ts';
export type * from './types.ts';
export * from './usage-map.ts';
export * from './project-context.ts';

//region Plugin contract

/**
 Merged advisor config resolver re-read per operation so the session's
 project file participates.

 @param cwd - session working directory selecting the project config file

 @returns merged advisor config for the operation
 */
function getConfig(cwd: string): Promise<AdvisorConfig> {
  return loadDshAdvisorConfig({ cwd, });
}

/**
 Plugin anchor name matching the bundle patch entry.
 */
export const name = 'advisor';

/**
 Host services the plugin registers against and reads at execution time.
 */
export const inject: readonly string[] = [
  'tools',
  'commands',
  'llm',
  'systemPrompt',
  'workingDirectory',
];

/**
 Loader-validated plugin config row.

 Structural fields only: tunables stay in the unified advisor JSON files the
 shared loader reads, and `defaultReviewerModel` is never a row value or a
 schema default. The row is empty, so `apply` reads no config argument.
 */
export const Config: ReturnType<typeof z.object> = z.object({},);

/**
 Activate the Advisor plugin: register the tool, the command, and the
 tool-guidance prompt section.

 Registration runs as one async effect body because the enablement seed
 comes from the merged advisor config file read at activation.

 @param ctx - agent-scoped registration context

 @example
 ```ts
 apply(ctx);
 ```
 */
export function apply(
  ctx: Context,
): void {
  /**
   Logger tagged with the plugin entry name.
   */
  const l = tagged({
    tag: apply.name,
    l: dshAdvisorLogger,
  },);
  ctx
    .effect(async function registerAdvisorPlugin() {
      /**
       Enablement seed read from the merged advisor config at activation.
       */
      const initialConfig = await loadDshAdvisorConfig({
        cwd: process.cwd(),
      },);
      /**
       Session-scoped toggle, transcript degradation record, and tool-mask
       disposer shared by the tool and the command.
       */
      const state = createDshAdvisorSessionState({
        initialEnabled: initialConfig.enabled,
      },);
      /**
       Disposers of every registration this effect owns.
       */
      const disposers = [
        ctx
          .tools
          .register(createDshAdvisorTool({
            getConfig,
            getSessionEnabled: state.getEnabled,
            recordTranscriptDegradation: state.recordTranscriptDegradation,
            llm: ctx.llm,
            systemPrompt: ctx.systemPrompt,
            workingDirectory: ctx.workingDirectory,
          },),),
        ctx
          .commands
          .register(createDshAdvisorCommand({
            getConfig,
            state,
            llm: ctx.llm,
            systemPrompt: ctx.systemPrompt,
            workingDirectory: ctx.workingDirectory,
            tools: ctx.tools,
          },),),
        ctx
          .systemPrompt
          .section({
            name: ADVISOR_TOOL_PROMPT_SECTION.name,
            order: ADVISOR_TOOL_PROMPT_SECTION.order,
            text: ADVISOR_TOOL_PROMPT_SECTION_TEXT,
          },),
      ];
      l.debug(`advisor plugin applied; enabled=${String(state.getEnabled(),)}`);
      return function disposeAdvisorPlugin(): void {
        for (const dispose of disposers
          .toReversed()) {
          dispose();
        }
      };
    },);
}

//endregion Plugin contract
