/**
 Router views and evidence lines the hybrid prototype records, shared by both of its files.
 @module
 */

import type {
  ExtensionAPI,
  ExtensionContext,
  ModelRoute,
} from '@earendil-works/pi-coding-agent';
import type {
  Api,
  Model,
  ModelThinkingLevel,
} from '@earendil-works/pi-ai';
import { PROVIDER_ID, } from '../index.ts';
import type { Member } from '../model-pools.ts';
import {
  excerpt,
  limitsOf,
  note,
} from './prototype-evidence.ts';

//region Views the router reads

/**
 Provider and model id of a physical member, read-only.

 Project-owned so a catalog model is assignable to it.
 */
type MemberIdentity = {
  readonly provider: string;
  readonly id: string;
  readonly api: string
};

/**
 Failed attempt a retry router is told about, read-only.

 Project-owned so Pi's routing request is assignable to it.
 */
type FailedAttemptView = {
  readonly model: MemberIdentity;
  readonly message: {
    readonly stopReason: string;
    readonly errorMessage?: string
  }
};

/**
 One routing request as the prototype reads it, read-only.

 Project-owned so Pi's routing request is assignable to it without this file claiming its fields.
 */
type RouteRequestView = {
  readonly reason: string;
  readonly thinkingLevel: ModelThinkingLevel;
  readonly failed?: FailedAttemptView;
  readonly previous?: {
    readonly model: MemberIdentity
  }
};

/**
 One finished turn as the prototype reads it, read-only.

 Project-owned so Pi's turn event is assignable to it without this file claiming its fields.
 */
export type TurnMessageView = {
  readonly turnIndex: number;
  readonly message: {
    readonly role: string;
    readonly provider?: string;
    readonly model?: string;
    readonly stopReason?: string;
    readonly errorMessage?: string;
    readonly usage?: {
      readonly input: number;
      readonly output: number
    }
  }
};

//endregion Views the router reads

//region Evidence hooks

/**
 Report the limits Pi would apply now, from Pi's own context-usage call.

 @param label - point in the session the reading belongs to

 @param ctx - session context Pi supplies to the handler

 @example
 ```ts
 noteContextUsage({ label: 'session_start', ctx });
 ```
 */
export function noteContextUsage({
  label,
  ctx,
}: {
  readonly label: string;
  readonly ctx: ExtensionContext;
},): void {
  note({
    event: 'context-usage',
    at: label,
    selection: ctx.model === undefined ? undefined : `${ctx.model
      .provider}/${ctx.model
        .id}`,
    declared: ctx.model === undefined ? undefined : limitsOf({ model: ctx.model, }),
    usage: ctx.getContextUsage(),
  },);
}

/**
 Resolve one catalog model, refusing with the pair it could not find.

 @param registry - session registry

 @param member - provider and id to resolve

 @returns the physical catalog model

 @throws Error when Pi's catalog holds no such model

 @example
 ```ts
 resolved(ctx.modelRegistry, { provider: PROVIDER_ID, id: 'writer' });
 ```
 */
export function resolved({
  registry,
  member,
}: {
  readonly registry: ExtensionContext['modelRegistry'];
  readonly member: Member;
},): Model<Api> {
  /**
   Catalog lookup bound to its own registry: `ModelRegistry.find` takes two strings, and a bare
   `find(` call with two arguments reads like an array callback reference to the repository's
   array-method rules. `src/index.ts` binds the same lookup for the same reason.
   */
  const findModel = registry
    .find
    .bind(registry,);
  /**
   Catalog record Pi holds for the pair.
   */
  const found = findModel(
    member.provider,
    member.id,
  );
  if (found === undefined)
    throw new Error(`The hybrid prototype could not resolve ${member.provider}/${member.id}`,);
  return found;
}

/**
 Report what one router was asked for and what it picked.

 @param virtualModelId - virtual model id the router belongs to

 @param request - routing request Pi supplied

 @param ctx - session context Pi supplies to the router

 @param chosen - model the router returns

 @example
 ```ts
 noteRoute({ virtualModelId: HYBRID_WRITER_ID, request, ctx, chosen: pool });
 ```
 */
export function noteRoute({
  virtualModelId,
  request,
  ctx,
  chosen,
}: {
  readonly virtualModelId: string;
  readonly request: RouteRequestView;
  readonly ctx: ExtensionContext;
  readonly chosen: Model<Api>;
},): void {
  note({
    event: 'route',
    virtualModel: `${PROVIDER_ID}/${virtualModelId}`,
    reason: request.reason,
    selectedThinkingLevel: request.thinkingLevel,
    failedModel: request.failed === undefined
      ? undefined
      : `${request.failed
        .model
        .provider}/${request.failed
          .model
          .id}`,
    failedStopReason: request.failed
      ?.message
      .stopReason,
    failedMessage: excerpt(request.failed
      ?.message
      .errorMessage
      ?? '',),
    previousModel: request.previous === undefined
      ? undefined
      : `${request.previous
        .model
        .provider}/${request.previous
          .model
          .id}`,
    chosen: limitsOf({ model: chosen, }),
    usageBeforeRoute: ctx.getContextUsage(),
  },);
}

/**
 Register one virtual model whose router always picks the same catalog model.

 @param pi - extension API

 @param id - virtual model id, listed under the package's provider

 @param name - selection name Pi shows

 @param pick - resolves the physical model for one request from the session context

 @example
 ```ts
 registerAlwaysRouter({ pi, id: 'hybrid-writer', name: 'Hybrid writer', pick: (ctx) => ... });
 ```
 */
export function registerAlwaysRouter({
  pi,
  id,
  name,
  declaredLimits,
  pick,
}: {
  readonly pi: ExtensionAPI;
  readonly id: string;
  readonly name: string;
  readonly declaredLimits: {
    readonly contextWindow: number;
    readonly maxTokens: number
  };
  readonly pick: (ctx: ExtensionContext) => Model<Api>;
},): void {
  pi.registerVirtualModel({
    provider: PROVIDER_ID,
    id,
    name,
    contextWindow: declaredLimits.contextWindow,
    maxTokens: declaredLimits.maxTokens,
    thinkingLevels: ['off',],
    route(
      request: RouteRequestView,
      ctx: ExtensionContext,
    ): ModelRoute {
      /**
       Physical model this request goes to.
       */
      const chosen = pick(ctx,);
      noteRoute({
        virtualModelId: id,
        request,
        ctx,
        chosen,
      },);
      return {
        model: chosen,
        thinkingLevel: request.thinkingLevel,
      };
    },
  },);
}

//endregion Evidence hooks
