/**
 Final dispatch uses the shared live-scope reader, including getter-backed hosts. @module
 */
import {
  NO_LIVE_SCOPE,
  readLiveScope,
  type ResolveEffectiveScopeContext,
} from '@monochromatic-dev/agent-harness-shared-model-selection/ts';
import type { ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  AdvisorReadonlyModel,
  ScopedAdvisorModel,
} from './types.ts';

/**
 Recheck the authoritative live scope after asynchronous request preparation.
 
 @param ctx - host scope property or getter, using the same precedence as initial selection
 
 @param model - prepared canonical endpoint
 
 @mutates ctx - invokes the optional runtime scope getter
 
 @throws when the current live scope excludes this endpoint
 
 @example
 ```ts
 assertAdvisorLiveScope({ ctx, model: 'provider/reviewer' });
 ```
 */
export function assertAdvisorLiveScope({
  ctx,
  model,
}: {
  readonly ctx: ForeignHostCapability<Pick<ResolveEffectiveScopeContext, 'scopedModels' | 'getScopedModels'>>;
  readonly model: string;
},): void {
  /**
   Synchronous normalized snapshot, not a selection-time cached property.
   */
  const scope = readLiveScope<AdvisorReadonlyModel>(ctx,);
  if (scope === NO_LIVE_SCOPE)
    return;
  if (!scope.some(function includesEndpoint(entry: ScopedAdvisorModel,): boolean {
    return entry.canonicalSlug === model;
  },))
    throw new Error(`advisor: ${model} left the live scope before dispatch`,);
}
