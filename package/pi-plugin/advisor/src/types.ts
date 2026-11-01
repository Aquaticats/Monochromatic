/**
 Pi-side Advisor type surface.

 Host-neutral core types re-export from the shared package; tool shapes
 that reference pi's `ToolDefinition` and `AgentToolResult` stay here.

 @module
 */

import type {
  AgentToolResult,
  ToolDefinition,
} from '@earendil-works/pi-coding-agent';
import type { TSchema, } from 'typebox';
import type {
  AdvisorDetails,
  AdvisorProgressDetails,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';

export type {
  AdvisorConfig,
  AdvisorContext,
  AdvisorDetails,
  AdvisorProgressDetails,
  AdvisorRunResult,
  AdvisorToolParams,
  EffectiveModelScope,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';

//region Tool shapes

/**
 Advisor tool definition type with concrete parameters and details.
 */
export type AdvisorToolDefinition<TParameters extends TSchema = TSchema,> =
  ToolDefinition<
    TParameters,
    AdvisorDetails | AdvisorProgressDetails
    >;

/**
 Tool result shape returned by Advisor execution.
 */
export type AdvisorToolResult = AgentToolResult<AdvisorDetails>;

//endregion Tool shapes
