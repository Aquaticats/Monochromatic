/**
 Local routing identities preserve native capabilities without authentication headers. @module
 */
import {
  hasApi,
  type Api,
  type AnyModel,
  type Model,
} from '@earendil-works/pi-ai';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  CODEX_API,
  OPENAI_API,
  OPENAI_PROVIDER,
  PRIORITY_TARGET_PREFIX,
  type PriorityApi,
} from './constants.ts';
import { FastModelError, } from './fast-model-error.ts';
import type { OriginalModelLookup, } from './original-dispatch-types.ts';

//region Routing identity helpers

/**
 Module logger excludes payloads, authentication, and configured headers.
 */
const moduleLogger = tagged({ tag: 'pi-plugin-openai-fast.priority-target', },);

/**
 Detect extension-owned routing identities, never model priority compatibility.

 @param model - candidate identity

 @returns whether the reserved local prefix matches

 @example
 ```ts
 isPriorityTarget({ id: '__pi_openai_fast__/gpt-model' });
 ```
 */
export function isPriorityTarget(model: Readonly<Pick<AnyModel, 'id'>>, ): boolean {
  /**
   Identity logger records only the routing-boundary operation.
   */
  const l = tagged({
    tag: isPriorityTarget.name,
    l: moduleLogger,
  },);
  l.trace('checking internal routing identity',);
  return model.id
    .startsWith(PRIORITY_TARGET_PREFIX,);
}

/**
 Clone capabilities for a local target without inheriting request-auth headers.
 
 @param model - original catalog model
 
 @returns local physical target, never an upstream model ID

 @example
 ```ts
 const target = priorityTarget(baseModel);
 ```
 */
export function priorityTarget(model: ForeignBorrowed<Model<Api>>, ): Model<Api> {
  /**
   Target logger excludes all request-auth metadata.
   */
  const l = tagged({
    tag: priorityTarget.name,
    l: moduleLogger,
  },);
  l.trace(`creating physical routing target for ${model.id}`,);
  /**
   Authentication headers are deliberately left at the original request boundary.
   */
  const {
    headers: _headers,
    ...metadata
  } = model;
  return {
    ...metadata,
    provider: `${model.provider}-fast`,
    id: `${PRIORITY_TARGET_PREFIX}${model.id}`,
  };
}

/**
 Resolve a target to the live original rather than persisting a stale clone.
 
 @param model - requested internal target
 
 @param lookup - live original-model lookup
 
 @returns original native OpenAI model
 
 @throws FastModelError when input is not a target, the original disappeared, or its API is unsupported
 
 @mutates lookup - invokes supplied catalog lookup capability

 @example
 ```ts
 const base = resolvePriorityBase({ model: target, lookup });
 ```
 */
export function resolvePriorityBase({
  model,
  lookup,
}: {
  readonly model: ForeignBorrowed<Model<Api>>;
  readonly lookup: OriginalModelLookup;
},): Model<PriorityApi> {
  /**
   Original-model logger keeps translation visible without request data.
   */
  const l = tagged({
    tag: resolvePriorityBase.name,
    l: moduleLogger,
  },);
  if (!isPriorityTarget(model,))
    throw new FastModelError(`Model "${model.id}" is not an internal OpenAI priority target. Select its fast virtual entry instead.`,);
  /**
   Fixed prefix removal recovers the original catalog identity.
   */
  const id = model.id
    .slice(PRIORITY_TARGET_PREFIX.length,);
  /**
   Current original is resolved after configuration and catalog changes.
   */
  const base = lookup(id,);
  if (base === undefined)
    throw new FastModelError(`OpenAI fast model "${id}" no longer exists. Refresh models or select an available model.`,);
  /**
   Each original provider retains its own native wire API.
   */
  const expectedApi = base.provider === OPENAI_PROVIDER ? OPENAI_API : CODEX_API;
  if (!hasApi(
    base,
    expectedApi,
  )) {
    throw new FastModelError(`OpenAI fast model "${id}" uses "${base.api}" instead of its native "${expectedApi}" transport. Correct its model configuration or select its ordinary entry.`,);
  }
  l.debug(`resolved priority target to ${base.id}`,);
  return base;
}

//endregion
