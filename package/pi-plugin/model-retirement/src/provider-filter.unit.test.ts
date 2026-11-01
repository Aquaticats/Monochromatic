/**
 Tests for turning registry reads into per-provider filtering plans.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { ProviderConfig, } from '@earendil-works/pi-coding-agent';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type {
  Api,
  ClassifierApi,
  ClassifierModel,
  ImageApi,
  ImageModel,
  Model,
  Provider,
} from '@earendil-works/pi-ai';
import {
  buildRetiredIndex,
  decideRetirements,
  planProviderFilters,
  toModelConfig,
  wrapProvider,
  type CatalogRead,
  type ComposedProvider,
  type IncumbentConfig,
  type ProviderFilterPlan,
} from '../dist/final/node/index.mjs';

//region Fixtures

/** API type used by chat fixtures. */
const CHAT_API: Api = 'openai-completions';

/**
 Build one chat model fixture.

 @param provider - provider id owning the model

 @param id - model id

 @param api - API type the model is served under

 @param baseUrl - endpoint the model reports, empty to simulate a missing one

 @param samplingParams - sampling overrides to prove optional metadata survives

 @returns chat model shaped like a registry read
 */
function chatModel(
  {
    provider,
    id,
    api = CHAT_API,
    baseUrl = 'https://example.invalid',
    samplingParams,
  }: {
    readonly provider: string;
    readonly id: string;
    readonly api?: Api;
    readonly baseUrl?: string;
    readonly samplingParams?: Record<string, unknown>;
  },
): Model<Api> {
  return {
    id,
    name: id,
    api,
    provider,
    baseUrl,
    input: ['text'],
    cost: {
      input: 1,
      output: 2,
      cacheRead: 0,
      cacheWrite: 0,
    },
    reasoning: true,
    contextWindow: 128_000,
    maxTokens: 4_096,
    ...(samplingParams === undefined ? {} : { samplingParams, }),
  };
}

/**
 Build one image model fixture.

 @param provider - provider id owning the model

 @param id - model id

 @returns image model shaped like a registry read
 */
function imageModel(
  {
    provider,
    id,
  }: {
    readonly provider: string;
    readonly id: string;
  },
): ImageModel<ImageApi> {
  return {
    id,
    name: id,
    api: 'openrouter-images',
    provider,
    baseUrl: 'https://example.invalid',
    input: ['text', 'image'],
    cost: {
      input: 1,
      output: 2,
      cacheRead: 0,
      cacheWrite: 0,
    },
    type: 'image',
    output: ['image'],
  };
}

/**
 Build one classifier model fixture.

 @param provider - provider id owning the model

 @param id - model id

 @returns classifier model shaped like a registry read
 */
function classifierModel(
  {
    provider,
    id,
  }: {
    readonly provider: string;
    readonly id: string;
  },
): ClassifierModel<ClassifierApi> {
  return {
    id,
    name: id,
    api: 'typesafe-system-one',
    provider,
    baseUrl: 'https://example.invalid',
    input: ['text'],
    cost: {
      input: 1,
      output: 2,
      cacheRead: 0,
      cacheWrite: 0,
    },
    type: 'classifier',
    contextWindow: 8_000,
  };
}

/**
 Build one composed provider fixture.

 @param provider - provider id

 @param models - chat models its listing returns, read live on every call

 @returns composed provider shaped like a registry read
 */
function composedProvider(
  {
    provider,
    models,
  }: {
    readonly provider: string;
    readonly models: () => readonly Model<Api>[];
  },
): Provider {
  /**
   Object implementing the members wrapping reads and spreads.
   */
  const fixture = {
    id: provider,
    name: provider,
    auth: { name: 'fixture-auth', },
    stream: function fixtureStream(): string {
      return 'stream';
    },
    getModels: models,
  };
  return fixture as unknown as Provider;
}

/**
 Build the injected catalog read from fixture lists.

 @param chat - chat models the registry reports

 @param images - image models the registry reports

 @param classifiers - classifier models the registry reports

 @param configs - incumbent configurations by provider

 @param composed - composed providers by provider

 @returns reads the planner consumes
 */
function readOf(
  {
    chat,
    images = [],
    classifiers = [],
    configs = {},
    composed = {},
  }: {
    readonly chat: readonly Model<Api>[];
    readonly images?: readonly ImageModel<ImageApi>[];
    readonly classifiers?: readonly ClassifierModel<ClassifierApi>[];
    readonly configs?: Readonly<Record<string, ProviderConfig>>;
    readonly composed?: Readonly<Record<string, Provider>>;
  },
): CatalogRead {
  return {
    readChatModels: function readChatModels() {
      return chat;
    },
    readImageModels: function readImageModels() {
      return images;
    },
    readClassifierModels: function readClassifierModels() {
      return classifiers;
    },
    readProviderConfig: function readProviderConfig(provider,): IncumbentConfig {
      /**
       Incumbent configuration for this fixture provider, when one was supplied.
       */
      const config = configs[provider];
      if (config === undefined)
        return { kind: 'absent', };
      return {
        kind: 'present',
        config,
      };
    },
    readComposedProvider: function readComposedProvider(provider,): ComposedProvider {
      /**
       Composed provider for this fixture provider, when one was supplied.
       */
      const found = composed[provider];
      if (found === undefined)
        return { kind: 'absent', };
      return {
        kind: 'present',
        provider: found,
      };
    },
  };
}

/**
 Read the ids a plan carries, in order.

 @param models - configurations one plan would register

 @returns ids in registration order
 */
function planIds(models: readonly { readonly id: string; }[],): readonly string[] {
  return models.map(function toId(model,) {
    return model.id;
  },);
}

/**
 Read the ids a configuration plan carries, in order.

 @param plan - plan to inspect

 @returns ids in registration order, empty for a wrapper plan or no plan
 */
function configIds(plan: ProviderFilterPlan,): readonly string[] {
  if (plan.kind !== 'config')
    return [];
  return planIds(plan.config.models ?? [],);
}

/**
 Require a configuration plan so its incumbent fields can be asserted.

 @param plan - plan to inspect

 @returns configuration the plan would register

 @throws when the plan is absent or is a wrapper plan
 */
function requireConfig(plan: ProviderFilterPlan,): ProviderConfig {
  if (plan.kind !== 'config')
    throw new Error('expected a configuration plan',);
  return plan.config;
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: planProviderFilters.name,
      children: [
        it({
          name: 'plans a configuration filter for a provider with a readable configuration',
          fn: async function runConfigPlan() {
            /**
             Planning over one superseded pair on a configuration-registered provider.
             */
            const planning = planProviderFilters({
              read: readOf({
                chat: [
                  chatModel({ provider: 'synthetic', id: 'glm-5.2', },),
                  chatModel({ provider: 'synthetic', id: 'glm-5.3', },),
                  chatModel({ provider: 'openai', id: 'gpt-5.5', },),
                ],
                configs: { synthetic: { api: 'openai-completions', }, },
              },),
            },);
            expect(planning.plans.length,).toBe(1,);
            expect(planning.plans[0]?.kind,).toBe('config',);
            expect(planning.plans[0]?.provider,).toBe('synthetic',);
            expect(
              configIds(nonNullishOrThrow(planning.plans[0],),),
            ).toEqual(['glm-5.3'],);
            expect(planning.retirements.length,).toBe(1,);
          },
        },),
        it({
          name: 'preserves the incumbent configuration it merges over',
          fn: async function runIncumbentPreserved() {
            /**
             Planning over a provider whose incumbent configuration carries an endpoint,
             an api, and request headers.
             */
            const planning = planProviderFilters({
              read: readOf({
                chat: [
                  chatModel({ provider: 'radius', id: 'glm-5.2', },),
                  chatModel({ provider: 'radius', id: 'glm-5.3', },),
                ],
                configs: {
                  radius: {
                    api: 'pi-messages',
                    baseUrl: 'https://radius.invalid/v1',
                    headers: { 'x-source': 'pi', },
                  },
                },
              },),
            },);
            /**
             Configuration the plan would register.
             */
            const config = requireConfig(nonNullishOrThrow(planning.plans[0],),);
            expect(
              configIds(nonNullishOrThrow(planning.plans[0],),),
            ).toEqual(['glm-5.3'],);
            expect(config.api,).toBe('pi-messages',);
            expect(config.baseUrl,).toBe('https://radius.invalid/v1',);
            expect(config.headers,).toEqual({ 'x-source': 'pi', },);
          },
        },),
        it({
          name: 'carries image and classifier models through a configuration plan',
          fn: async function runOtherModelTypes() {
            /**
             Planning over a provider serving all three model types.
             */
            const planning = planProviderFilters({
              read: readOf({
                chat: [
                  chatModel({ provider: 'openrouter', id: 'vendor/gpt-5', },),
                  chatModel({ provider: 'openrouter', id: 'vendor/gpt-5.5', },),
                ],
                images: [imageModel({ provider: 'openrouter', id: 'vendor/flux', },)],
                classifiers: [classifierModel({ provider: 'openrouter', id: 'vendor/judge', },)],
                configs: { openrouter: {}, },
              },),
            },);
            expect(
              configIds(nonNullishOrThrow(planning.plans[0],),),
            ).toEqual([
              'vendor/gpt-5.5',
              'vendor/flux',
              'vendor/judge',
            ],);
          },
        },),
        it({
          name: 'plans a wrapper for a provider pi exposes only as a composed object',
          fn: async function runWrapperPlan() {
            /**
             Chat models the composed provider lists.
             */
            const models = [
              chatModel({ provider: 'hyper', id: 'glm-5.2', },),
              chatModel({ provider: 'hyper', id: 'glm-5.3', },),
            ];
            /**
             Planning over a native registration with no readable configuration.
             */
            const planning = planProviderFilters({
              read: readOf({
                chat: models,
                composed: {
                  hyper: composedProvider({
                    provider: 'hyper',
                    models: function listModels() {
                      return models;
                    },
                  },),
                },
              },),
            },);
            expect(planning.plans.length,).toBe(1,);
            expect(planning.plans[0]?.kind,).toBe('wrapper',);
            expect(planning.skippedProviders,).toEqual([],);
          },
        },),
        it({
          name: 'falls back to a wrapper when a model cannot be re-declared',
          fn: async function runWrapperFallback() {
            /**
             Chat models where the winner carries no endpoint.
             */
            const models = [
              chatModel({ provider: 'azure-openai-responses', id: 'gpt-4.1', },),
              chatModel({
                provider: 'azure-openai-responses',
                id: 'gpt-5.5',
                baseUrl: '',
              },),
            ];
            /**
             Planning over a provider that has a configuration but no usable endpoint.
             */
            const planning = planProviderFilters({
              read: readOf({
                chat: models,
                configs: { 'azure-openai-responses': {}, },
                composed: {
                  'azure-openai-responses': composedProvider({
                    provider: 'azure-openai-responses',
                    models: function listModels() {
                      return models;
                    },
                  },),
                },
              },),
            },);
            expect(planning.plans.length,).toBe(1,);
            expect(planning.plans[0]?.kind,).toBe('wrapper',);
            expect(planning.skippedProviders,).toEqual([],);
          },
        },),
        it({
          name: 'skips a provider whose models cannot be re-declared and that has no composed object',
          fn: async function runEndpointSkip() {
            /**
             Planning over a provider with a configuration but no endpoint anywhere.
             */
            const planning = planProviderFilters({
              read: readOf({
                chat: [
                  chatModel({ provider: 'azure-openai-responses', id: 'gpt-4.1', },),
                  chatModel({
                    provider: 'azure-openai-responses',
                    id: 'gpt-5.5',
                    baseUrl: '',
                  },),
                ],
                configs: { 'azure-openai-responses': {}, },
              },),
            },);
            expect(planning.plans,).toEqual([],);
            expect(planning.retirements.length,).toBe(1,);
            expect(planning.skippedProviders.length,).toBe(1,);
            expect(planning.skippedProviders[0]?.reason.includes('baseUrl',),).toBe(true,);
          },
        },),
        it({
          name: 'skips a provider pi exposes nothing filterable for',
          fn: async function runNothingToFilterSkip() {
            /**
             Planning over a provider with neither a configuration nor a composed object.
             */
            const planning = planProviderFilters({
              read: readOf({
                chat: [
                  chatModel({ provider: 'hyper', id: 'glm-5.2', },),
                  chatModel({ provider: 'hyper', id: 'glm-5.3', },),
                ],
              },),
            },);
            expect(planning.plans,).toEqual([],);
            expect(planning.skippedProviders.length,).toBe(1,);
            expect(
              planning.skippedProviders[0]?.reason.includes('neither a provider configuration',),
            ).toBe(true,);
          },
        },),
        it({
          name: 'reports the abstention it took on an unordered pair',
          fn: async function runAbstentionReporting() {
            /**
             Planning over the measured month-day against eight-digit pair.
             */
            const planning = planProviderFilters({
              read: readOf({
                chat: [
                  chatModel({ provider: 'openrouter', id: 'qwen/qwen3.5-plus-02-15', },),
                  chatModel({ provider: 'openrouter', id: 'qwen/qwen3.5-plus-20260420', },),
                ],
                configs: { openrouter: {}, },
              },),
            },);
            expect(planning.plans,).toEqual([],);
            expect(planning.abstentions.keeperAmbiguity > 0,).toBe(true,);
          },
        },),
        it({
          name: 'plans nothing for a catalog with no superseded pair',
          fn: async function runEmptyPlanning() {
            /**
             Planning over two models in different families.
             */
            const planning = planProviderFilters({
              read: readOf({
                chat: [
                  chatModel({ provider: 'hyper', id: 'glm-5.3', },),
                  chatModel({ provider: 'hyper', id: 'glm-5.3-flash', },),
                ],
                configs: { hyper: {}, },
              },),
            },);
            expect(planning.plans,).toEqual([],);
            expect(planning.retirements,).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: wrapProvider.name,
      children: [
        it({
          name: 'omits retired chat models and keeps everything else',
          fn: async function runWrapperFiltering() {
            /**
             Chat models the composed provider lists.
             */
            const models = [
              chatModel({ provider: 'hyper', id: 'glm-5.2', },),
              chatModel({ provider: 'hyper', id: 'glm-5.3', },),
              chatModel({ provider: 'hyper', id: 'glm-5.3-flash', },),
            ];
            /**
             Composed provider under test.
             */
            const original = composedProvider({
              provider: 'hyper',
              models: function listModels() {
                return models;
              },
            },);
            /**
             Index built from one retirement of the oldest model.
             */
            const index = buildRetiredIndex({
              retirements: decideRetirements({
                entries: models.map(function toEntry(model,) {
                  return {
                    provider: model.provider,
                    api: model.api,
                    modelId: model.id,
                  };
                },),
              },).retirements,
            },);
            /**
             Wrapped provider.
             */
            const wrapped = wrapProvider({ provider: original, index, },);
            expect(wrapped.getModels().map(function toId(model,) {
              return model.id;
            },),).toEqual(['glm-5.3', 'glm-5.3-flash'],);
            expect(wrapped.id,).toBe('hyper',);
            expect(wrapped.name,).toBe('hyper',);
            expect(typeof wrapped.stream,).toBe('function',);
            expect(wrapped.auth,).toBe(original.auth,);
          },
        },),
        it({
          name: 'filters live, so a refreshed catalog is filtered on the next read',
          fn: async function runLiveFiltering() {
            /**
             Mutable list standing in for a catalog a refresh can change.
             */
            const models: Model<Api>[] = [chatModel({ provider: 'hyper', id: 'glm-5.3', },)];
            /**
             Composed provider reading that mutable list.
             */
            const original = composedProvider({
              provider: 'hyper',
              models: function listModels() {
                return models;
              },
            },);
            /**
             Index retiring a model that is not listed yet.
             */
            const index = buildRetiredIndex({
              retirements: [{
                provider: 'hyper',
                api: CHAT_API,
                retiredId: 'glm-5.2',
                keeperId: 'glm-5.3',
              }],
            },);
            /**
             Wrapped provider.
             */
            const wrapped = wrapProvider({ provider: original, index, },);
            expect(wrapped.getModels().length,).toBe(1,);
            models.push(chatModel({ provider: 'hyper', id: 'glm-5.2', },),);
            expect(wrapped.getModels().map(function toId(model,) {
              return model.id;
            },),).toEqual(['glm-5.3'],);
          },
        },),
      ],
    },),
    describe({
      name: toModelConfig.name,
      children: [
        it({
          name: 'marks a chat model and keeps its sampling params',
          fn: async function runChatConfig() {
            /**
             Configuration for a chat model carrying sampling overrides.
             */
            const config = toModelConfig(chatModel({
              provider: 'hyper',
              id: 'glm-5.3',
              samplingParams: { temperature: 0.5, },
            },),);
            expect(config.type ?? 'chat',).toBe('chat',);
            expect('samplingParams' in config,).toBe(true,);
          },
        },),
        it({
          name: 'marks an image model and keeps its output modalities',
          fn: async function runImageConfig() {
            /**
             Configuration for an image model.
             */
            const config = toModelConfig(imageModel({ provider: 'openrouter', id: 'vendor/flux', },),);
            expect(config.type,).toBe('image',);
            expect('output' in config,).toBe(true,);
          },
        },),
        it({
          name: 'marks a classifier model and keeps its context window',
          fn: async function runClassifierConfig() {
            /**
             Configuration for a classifier model.
             */
            const config = toModelConfig(classifierModel({
              provider: 'openrouter',
              id: 'vendor/judge',
            },),);
            expect(config.type,).toBe('classifier',);
            expect('contextWindow' in config,).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
