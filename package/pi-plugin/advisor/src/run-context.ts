/**
 Pi host adapter assembling the shared Advisor run context.

 The shared core reads the host through three ports; this module builds
 them from pi's `ExtensionContext` (model registry, session manager, and
 pi-coding-agent serialization) exactly once per execution.

 @module
 */

import {
  convertToLlm,
  serializeConversation,
  type ExtensionContext,
} from '@earendil-works/pi-coding-agent';
import { estimateAdvisorInputTokens, } from '@monochromatic-dev/agent-harness-shared-model-selection/ts';
import type {
  AdvisorRunContext,
  AdvisorTranscriptSerialization,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import type {
  ForeignHostCapability,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Pi context surface the adapter reads: the typed extension context plus
 the live scope getter pi exposes at runtime beyond the interface.
 */
export type PiAdvisorRunContextSource = ForeignHostCapability<ExtensionContext> & {
  /**
   Live scoped-model getter, present on the runtime context object.
   */
  readonly getScopedModels?: () => unknown;
};

/**
 Transcript serialization seam backed by pi-coding-agent conversion and
 the shared model-selection token estimator.

 Host functions satisfy the port properties directly because the port
 parameters mirror the host's mutable message arrays.
 */
export const piAdvisorTranscriptSerialization: AdvisorTranscriptSerialization = {
  convertToLlm,
  serializeConversation,
  estimateInputTokens: estimateAdvisorInputTokens,
};

/**
 Assemble the shared Advisor run context from pi capabilities.

 @param ctx - pi extension context

 @returns run context wiring the three Advisor ports to pi

 @example
 ```typescript
 const result = await runAdvisorCore({ ctx: toAdvisorRunContext(ctx), config });
 ```
 */
export function toAdvisorRunContext(
  ctx: PiAdvisorRunContextSource,
): ForeignHostCapability<AdvisorRunContext> {
  return {
    modelRegistry: ctx
      .modelRegistry,
    transcript: {
      ...piAdvisorTranscriptSerialization,
      buildContextEntries(): ReturnType<typeof ctx.sessionManager.buildContextEntries> {
        return ctx.sessionManager
          .buildContextEntries();
      },
    },
    cwd: ctx.cwd,
    ...(ctx.model
      === undefined ? {} : { model: ctx.model, }),
    ...(ctx.getScopedModels
      === undefined ? {} : { getScopedModels: ctx.getScopedModels, }),
    ...(ctx.scopedModels
      === undefined ? {} : { scopedModels: ctx.scopedModels, }),
  };
}
