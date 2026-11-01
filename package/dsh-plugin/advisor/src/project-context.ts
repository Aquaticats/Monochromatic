/**
 Project-context capture from the DSH system-prompt registry. @module
 */

import {
  renderPrompt,
  type SystemPrompt,
} from '@deepseek-ai/dsh-system-prompt';
import type {
  DshAdvisorAgentSurface,
} from './types.ts';

//region Types

/**
 System-prompt surface the plugin reads for project context.

 `SystemPrompt` is a nominal registry class, so tests substitute this
 structural pick while the live service satisfies it unchanged.

 @example
 ```ts
 const projectContext = await buildDshAdvisorProjectContext({ systemPrompt: ctx.systemPrompt });
 ```
 */
export type DshAdvisorSystemPromptSurface = Pick<SystemPrompt, 'assemble'>;

//endregion Types

//region Public API

/**
 Render the assembled system prompt as Advisor project context.

 The rendered text carries `AGENTS.md`, repo instructions, and other context
 sections into the Advisor system prompt so reviewers see them even after
 conversation compaction.

 @param systemPrompt - system-prompt registry surface whose assembly carries
 the project context

 @param agent - agent surface whose assembly context resolves the `model` and
 `provider` prompt variables referenced by persona sections

 @param signal - abort signal forwarded into the assembly context

 @returns rendered project-context text

 @example
 ```ts
 const projectContext = await buildDshAdvisorProjectContext({ systemPrompt, agent });
 ```
 */
export async function buildDshAdvisorProjectContext(
  {
    systemPrompt,
    agent,
    signal,
  }: {
    readonly systemPrompt: DshAdvisorSystemPromptSurface;
    readonly agent: DshAdvisorAgentSurface;
    readonly signal?: AbortSignal;
  },
): Promise<string> {
  return renderPrompt(await systemPrompt
    .assemble(agent
      .assembleContext(signal),),);
}

//endregion Public API
