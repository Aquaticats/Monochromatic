/**
 Model catalog over `ctx.llm`: route facts for selection and status, plus the
 shared model-resolution port. @module
 */

import type {
  Api,
  Model,
  Provider,
} from '@earendil-works/pi-ai';
import {
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import type {
  AdvisorRequestAuth,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import type {
  LlmModelInfo,
  LlmProviderInfo,
  LlmResolvedModelInfo,
  ReasoningEffortId,
} from '@deepseek-ai/dsh-llm';
import type {
  SessionId,
} from '@deepseek-ai/dsh-session/types';
import type {
  ForeignBorrowed,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  DSH_MODEL_API,
  DSH_MODEL_BASE_URL,
  DSH_UNKNOWN_CONTEXT_WINDOW_TOKENS,
} from './constants.ts';
import { dshAdvisorLogger, } from './logging.ts';
import { createDshAdvisorProvider, } from './provider.ts';
import { selectDshAdvisorReasoningEfforts, } from './reasoning-effort.ts';
import type {
  DshAdvisorCatalog,
  DshAdvisorLlmSurface,
  DshAdvisorModelFacts,
} from './types.ts';

//region Types

/**
 Inputs for building one catalog snapshot per operation.
 */
export type BuildDshAdvisorCatalogOptions = {
  /**
   DSH LLM surface listing and resolving provider routes.
   */
  readonly llm: DshAdvisorLlmSurface;
  /**
   Session identity stamped onto every catalog route's requests.
   */
  readonly sessionId: SessionId;
  /**
   Configured output token budget used when a route declares no capacity.
   */
  readonly maxAdvisorOutputTokens: number;
  /**
   Cancellation for adapter-owned route resolution.
   */
  readonly signal?: AbortSignal;
};

//endregion Types

//region Route facts

/**
 Project one resolved route onto the pi-ai model record plus DSH facts.

 Unknown context windows substitute `DSH_UNKNOWN_CONTEXT_WINDOW_TOKENS`
 (128_000) and stay eligible; unknown output capacity substitutes
 `maxAdvisorOutputTokens` and also stays eligible, mirroring the shared
 capacity rule.

 @param resolved - route metadata from `resolveModelInfo`

 @param maxAdvisorOutputTokens - configured output budget substituting unknown
 output capacity

 @returns model record with capacity and effort facts
 */
function buildDshAdvisorModelFacts(
  {
    resolved,
    maxAdvisorOutputTokens,
  }: ForeignBorrowed<{
    readonly resolved: LlmResolvedModelInfo;
    readonly maxAdvisorOutputTokens: number;
  }>,
): DshAdvisorModelFacts {
  /**
   Reasoning efforts eligible for Advisor requests, least to greatest.
   */
  const eligibleEfforts = selectDshAdvisorReasoningEfforts(resolved.reasoning,);
  /**
   Greatest eligible effort, absent on routes advertising none.
   */
  const effort = eligibleEfforts.at(-1);
  return {
    model: {
      id: resolved.id,
      name: resolved.name,
      api: DSH_MODEL_API,
      provider: resolved.provider,
      baseUrl: DSH_MODEL_BASE_URL,
      input: resolved.inputModalities === undefined
        ? ['text',]
        : [...resolved.inputModalities,],
      cost: {
        input: 0,
        output: 0,
        cacheRead: 0,
        cacheWrite: 0,
      },
      reasoning: (resolved.reasoning
        ?.efforts
        .length
        ?? 0)
        > 0,
      contextWindow: resolved.context
        ?.contextWindow
        ?? DSH_UNKNOWN_CONTEXT_WINDOW_TOKENS,
      maxTokens: resolved.defaultMaxTokens
        ?? maxAdvisorOutputTokens,
    },
    outputCapacityUnknown: resolved.defaultMaxTokens === undefined,
    contextWindowUnknown: resolved.context === undefined,
    ...(effort === undefined
      ? {}
      : { reasoningEffort: effort, }),
    slug: `${resolved.provider}/${resolved.id}`,
  };
}

/**
 Advisor request auth for one DSH route.

 The DSH runtime owns route authentication, so the model-resolution port
 reports a successful credential read with no credential payload.

 @param model - route model requesting credentials

 @returns credential read reporting success without a payload
 */
function resolveAdvisorRequestAuth(
  model: ForeignBorrowed<Model<Api>>,
): Promise<AdvisorRequestAuth> {
  void model;
  return Promise.resolve({
    ok: true,
  },);
}

//endregion Route facts

//region Public API

/**
 Build one catalog snapshot over every provider route `ctx.llm` exposes.

 Each route is resolved once per operation; a route whose resolution fails is
 skipped with a warning so one broken adapter entry cannot veto the catalog.

 @param options - LLM surface and configured output budget

 @returns catalog snapshot satisfying the shared model-resolution port

 @throws when `listModels` rejects for a registered provider route

 @example
 ```ts
 const catalog = await buildDshAdvisorCatalog({ llm: ctx.llm, sessionId, maxAdvisorOutputTokens: 4096 });
 ```
 */
export async function buildDshAdvisorCatalog(
  options: BuildDshAdvisorCatalogOptions,
): Promise<DshAdvisorCatalog> {
  /**
   Logger tagged with the catalog builder name.
   */
  const l = tagged({
    tag: buildDshAdvisorCatalog.name,
    l: dshAdvisorLogger,
  },);
  /**
   Provider routes in registration order.
   */
  const providers = options
    .llm
    .listProviders();
  /**
   Resolved facts collected per provider route in registration order.
   */
  const factsByProvider = await Promise.all(
    providers.map(async function collectProviderFacts(
      providerInfo: ForeignBorrowed<LlmProviderInfo>,
    ) {
      /**
       Catalog models advertised for this provider route.
       */
      const modelInfos = await options
        .llm
        .listModels(providerInfo.id,);
      /**
       Resolved route lookups kept individually so one failure drops one route.
       */
      const settled = await Promise.allSettled(
        modelInfos.map(async function resolveRoute(
          info: ForeignBorrowed<LlmModelInfo>,
        ) {
          return await options
            .llm
            .resolveModelInfo(
            info.provider,
            info.id,
            options.signal,
            );
        },),
      );
      /**
       Facts built from the resolved routes of this provider route.
       */
      const facts: DshAdvisorModelFacts[] = [];
      for (const outcome of settled) {
        if (outcome.status
          === 'rejected') {
          l.warn(
            `advisor catalog skipped one route of provider ${providerInfo.id}: ${String(outcome.reason,)}`,
          );
          continue;
        }
        facts.push(
          buildDshAdvisorModelFacts({
            resolved: outcome.value,
            maxAdvisorOutputTokens: options.maxAdvisorOutputTokens,
          },),
        );
      }
      return facts;
    },),
  );
  /**
   Resolved facts in provider and catalog order.
   */
  const entries: DshAdvisorModelFacts[] = factsByProvider.flat();
  /**
   Facts indexed by canonical `provider/model` slug.
   */
  const factsBySlug = new Map<string, DshAdvisorModelFacts>();
  for (const facts of entries)
    factsBySlug.set(
      facts.slug,
      facts,
    );
  /**
   Catalog models in discovery order, one per resolved route.
   */
  const models = entries.map(function readModel(
    facts: ForeignBorrowed<DshAdvisorModelFacts>,
  ) {
    return facts.model;
  },);
  /**
   Provider shims indexed by provider route id.
   */
  const providersById = new Map<string, Provider>();
  /**
   Selected reasoning efforts indexed by canonical slug, shared by every
   provider shim.
   */
  const reasoningEffortByModelId = new Map<string, ReasoningEffortId>();
  for (const facts of entries) {
    if (facts.reasoningEffort !== undefined)
      reasoningEffortByModelId.set(
        `${facts.model
          .provider}/${facts.model
            .id}`,
        facts.reasoningEffort,
      );
  }
  for (const providerInfo of providers) {
    providersById.set(
      providerInfo.id,
      createDshAdvisorProvider({
        llm: options.llm,
        providerInfo,
        sessionId: options.sessionId,
        getModels: function getModels() {
          return models.filter(function belongsToProvider(
            model: ForeignBorrowed<Model<Api>>,
          ) {
            return model.provider
              === providerInfo.id;
          },);
        },
        reasoningEffortByModelId,
      },),
    );
  }
  return {
    entries,
    factsBySlug,
    getAvailable: function getAvailable(): readonly Model<Api>[] {
      return models;
    },
    getAll: function getAll(): readonly Model<Api>[] {
      return models;
    },
    getProvider: function getProvider(providerId: string) {
      return providersById.get(providerId,);
    },
    getApiKeyAndHeaders: resolveAdvisorRequestAuth,
  };
}

//endregion Public API
