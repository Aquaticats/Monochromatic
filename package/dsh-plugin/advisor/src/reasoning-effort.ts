/**
 DSH reasoning-effort eligibility rule: every advertised effort that is not
 the `max` effort, kept least to greatest. @module
 */

import type {
  LlmModelReasoningInfo,
  ReasoningEffortId,
} from '@deepseek-ai/dsh-llm';
import type {
  ForeignBorrowed,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  DSH_MAX_REASONING_EFFORT_LABEL,
} from './constants.ts';

//region Public API

/**
 Collect the reasoning efforts one Advisor request may carry.

 Efforts arrive least to greatest; the rule keeps every effort that is not
 the `max` effort (by effort name or id), matching pi's
 getSupportedThinkingLevels discipline of never sending `max`. The caller
 sends the last eligible effort, so a route advertising only `max` yields an
 empty list and the request omits `reasoningEffort` entirely.

 @param reasoning - route reasoning info with least-to-greatest efforts

 @returns eligible effort ids in least-to-greatest order

 @example
 ```ts
 const effort = selectDshAdvisorReasoningEfforts(resolved.reasoning).at(-1);
 ```
 */
export function selectDshAdvisorReasoningEfforts(
  reasoning?: ForeignBorrowed<LlmModelReasoningInfo>,
): readonly ReasoningEffortId[] {
  if (reasoning === undefined)
    return [];
  /**
   Eligible efforts in least-to-greatest order.
   */
  const eligible: ReasoningEffortId[] = [];
  for (const effort of reasoning
    .efforts) {
    /**
     Whether this effort is the `max` effort the rule never sends.
     */
    const isMaxEffort = (effort.name
      === DSH_MAX_REASONING_EFFORT_LABEL)
      || (String(effort.id,)
        === DSH_MAX_REASONING_EFFORT_LABEL);
    if (!isMaxEffort)
      eligible.push(effort.id,);
  }
  return eligible;
}

//endregion Public API
