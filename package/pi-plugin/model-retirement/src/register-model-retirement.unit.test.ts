/**
 Tests for the retirement pass and its session-start wiring.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type {
  ExtensionAPI,
  ProviderConfig,
} from '@earendil-works/pi-coding-agent';
import type {
  Api,
  ClassifierApi,
  ClassifierModel,
  ImageApi,
  ImageModel,
  Model,
  Provider,
} from '@earendil-works/pi-ai';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  applyRetirements,
  readsFromRegistry,
  registerModelRetirement,
  type CatalogRead,
  type ComposedProvider,
  type IncumbentConfig,
  type RetirementLog,
} from '../dist/final/node/index.mjs';

//region Fixtures

/** API type used by chat fixtures. */
const CHAT_API: Api = 'openai-completions';

/**
 Build one chat model fixture.

 @param provider - provider id owning the model

 @param id - model id

 @returns chat model shaped like a registry read
 */
function chatModel(
  {
    provider,
    id,
  }: {
    readonly provider: string;
    readonly id: string;
  },
): Model<Api> {
  return {
    id,
    name: id,
    api: CHAT_API,
    provider,
    baseUrl: 'https://example.invalid',
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

 @param models - chat models its listing returns

 @returns composed provider shaped like a registry read
 */
function composedProvider(
  {
    provider,
    models,
  }: {
    readonly provider: string;
    readonly models: readonly Model<Api>[];
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
    getModels: function listModels() {
      return models;
    },
  };
  return fixture as unknown as Provider;
}

/** Log records captured by a fake logger. */
type CapturedLog = {
  readonly info: string[];
  readonly debug: string[];
  readonly warn: string[];
};

/**
 Build an empty capture triple.

 @returns fresh capture arrays
 */
function emptyCapture(): CapturedLog {
  return {
    info: [],
    debug: [],
    warn: [],
  };
}

/**
 Build a fake logger that captures every message.

 @param captured - arrays the fake appends to

 @returns logger surface the pass accepts
 */
function fakeLog(captured: CapturedLog,): RetirementLog {
  return {
    info: function captureInfo(message,) {
      captured.info.push(message,);
    },
    debug: function captureDebug(message,) {
      captured.debug.push(message,);
    },
    warn: function captureWarn(message,) {
      captured.warn.push(message,);
    },
  };
}

/**
 Refresh stub that resolves immediately.

 @returns resolved promise, so a pass can await it
 */
function noopRefresh(): Promise<void> {
  return Promise.resolve();
}

/** One configuration registration recorded by a fake registrar. */
type RecordedConfig = {
  readonly name: string;
  readonly config: ProviderConfig;
};

/** One wrapper registration recorded by a fake registrar. */
type RecordedWrapper = {
  readonly provider: Provider;
};

/** Everything a fake registrar recorded. */
type RecordedRegistrations = {
  readonly configs: RecordedConfig[];
  readonly wrappers: RecordedWrapper[];
};

/**
 Build a fake registrar that records both registration forms.

 @param recorded - arrays the fake appends to

 @param rejectProvider - provider whose configuration registration should throw

 @returns registrar surface the pass accepts
 */
function fakeRegistrar(
  {
    recorded,
    rejectProvider,
  }: {
    readonly recorded: RecordedRegistrations;
    readonly rejectProvider?: string;
  },
) {
  return {
    registerConfig: function recordConfig(
      {
        name,
        config,
      }: {
        readonly name: string;
        readonly config: ProviderConfig;
      },
    ) {
      if (name === rejectProvider)
        throw new Error('"baseUrl" is required when defining custom models',);
      recorded.configs.push({ name, config, },);
    },
    registerProviderObject: function recordWrapper(
      {
        provider,
      }: {
        readonly provider: Provider;
      },
    ) {
      recorded.wrappers.push({ provider, },);
    },
  };
}

/**
 Build a catalog read over fixture lists.

 @param chat - chat models to report

 @param images - image models to report

 @param classifiers - classifier models to report

 @param configs - incumbent configurations by provider

 @param composed - composed providers by provider

 @returns reads the pass consumes
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
 Read the ids one configuration registration would install.

 @param config - configuration a fake registrar recorded

 @returns model ids in registration order
 */
function registeredModelIds(config: ProviderConfig,): readonly string[] {
  return (config.models ?? []).map(function toId(model,) {
    return model.id;
  },);
}

/** State a fake pi host accumulates. */
type FakeHostState = {
  readonly events: string[];
  readonly configs: RecordedConfig[];
  readonly wrappers: RecordedWrapper[];
  handler?: (payload: unknown, ctx: unknown,) => Promise<void>;
};

/**
 Build a fake pi host recording events and both registration forms.

 @param state - mutable state the fake writes to

 @returns fake extension API
 */
function fakeHost(state: FakeHostState,): ExtensionAPI {
  /**
   Object implementing only the members this extension touches.
   */
  const fake = {
    on(
      event: string,
      handler: (payload: unknown, ctx: unknown,) => Promise<void>,
    ): void {
      state.events.push(event,);
      state.handler = handler;
    },
    registerProvider(
      nameOrProvider: string | Provider,
      config?: ProviderConfig,
    ): void {
      if ((typeof nameOrProvider) === 'string') {
        state.configs.push({
          name: nameOrProvider,
          config: nonNullishOrThrow(config,),
        },);
        return;
      }
      state.wrappers.push({ provider: nameOrProvider, },);
    },
  };
  return fake as unknown as ExtensionAPI;
}

/**
 Build a session-start context over one fixture catalog.

 @param chat - chat models the registry reports

 @param composed - composed provider the registry returns, when any

 @param model - live model the session runs, when any

 @returns context shaped like the one pi passes
 */
function sessionContext(
  {
    chat,
    composed,
    model,
  }: {
    readonly chat: readonly Model<Api>[];
    readonly composed?: Provider;
    readonly model?: Model<Api>;
  },
): unknown {
  return {
    modelRegistry: {
      getAll: function getAll() {
        return chat;
      },
      getModelsOfType: function getModelsOfType() {
        return [];
      },
      refresh: function refresh() {
        return Promise.resolve({});
      },
      getRegisteredProviderConfig: function getRegisteredProviderConfig() {
        return undefined;
      },
      getProvider: function getProvider() {
        return composed;
      },
    },
    model,
  };
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: readsFromRegistry.name,
      children: [
        it({
          name: 'forwards every registry read',
          fn: async function runReadForwarding() {
            /**
             Chat model the fake registry reports.
             */
            const chat = chatModel({ provider: 'hyper', id: 'glm-5.3', },);
            /**
             Image model the fake registry reports.
             */
            const image = imageModel({ provider: 'hyper', id: 'flux', },);
            /**
             Classifier model the fake registry reports.
             */
            const classifier = classifierModel({ provider: 'hyper', id: 'judge', },);
            /**
             Composed provider the fake registry reports.
             */
            const composed = composedProvider({ provider: 'hyper', models: [chat], },);
            /**
             Reads adapted from single-method fakes.
             */
            const read = readsFromRegistry({
              chatReader: {
                getAll: function getAll() {
                  return [chat];
                },
              },
              imageReader: {
                getModelsOfType: function getImages() {
                  return [image];
                },
              },
              classifierReader: {
                getModelsOfType: function getClassifiers() {
                  return [classifier];
                },
              },
              configReader: {
                getRegisteredProviderConfig: function readConfig() {
                  return { api: 'pi-messages', };
                },
              },
              composedReader: {
                getProvider: function readComposed() {
                  return composed;
                },
              },
            },);
            expect(read.readChatModels(),).toEqual([chat],);
            expect(read.readImageModels('hyper',),).toEqual([image],);
            expect(read.readClassifierModels('hyper',),).toEqual([classifier],);
            expect(read.readProviderConfig('hyper',).kind,).toBe('present',);
            expect(read.readComposedProvider('hyper',).kind,).toBe('present',);
          },
        },),
        it({
          name: 'reports absence for a provider pi describes nowhere',
          fn: async function runReadAbsence() {
            /**
             Reads adapted from fakes that return nothing.
             */
            const read = readsFromRegistry({
              chatReader: {
                getAll: function getAll() {
                  return [];
                },
              },
              imageReader: {
                getModelsOfType: function getImages() {
                  return [];
                },
              },
              classifierReader: {
                getModelsOfType: function getClassifiers() {
                  return [];
                },
              },
              configReader: {
                getRegisteredProviderConfig: function readConfig() {
                  return undefined;
                },
              },
              composedReader: {
                getProvider: function readComposed() {
                  return undefined;
                },
              },
            },);
            expect(read.readProviderConfig('hyper',),).toEqual({ kind: 'absent', },);
            expect(read.readComposedProvider('hyper',),).toEqual({ kind: 'absent', },);
          },
        },),
      ],
    },),
    describe({
      name: applyRetirements.name,
      children: [
        it({
          name: 'registers the filtered provider and logs one line per retirement',
          fn: async function runRegistration() {
            /**
             Capture arrays for this pass.
             */
            const captured = emptyCapture();
            /**
             Registrations the pass made.
             */
            const recorded: RecordedRegistrations = { configs: [], wrappers: [], };
            /**
             Pass summary over one superseded pair.
             */
            const summary = await applyRetirements({
              refresh: noopRefresh,
              read: readOf({
                chat: [
                  chatModel({ provider: 'hyper', id: 'glm-5.2', },),
                  chatModel({ provider: 'hyper', id: 'glm-5.3', },),
                ],
                composed: {
                  hyper: composedProvider({
                    provider: 'hyper',
                    models: [
                      chatModel({ provider: 'hyper', id: 'glm-5.2', },),
                      chatModel({ provider: 'hyper', id: 'glm-5.3', },),
                    ],
                  },),
                },
              },),
              registrar: fakeRegistrar({ recorded, },),
              log: fakeLog(captured,),
            },);
            expect(recorded.wrappers.length,).toBe(1,);
            expect(summary.registeredProviders,).toEqual([{
              provider: 'hyper',
              kind: 'wrapper',
            }],);
            expect(summary.retirements.length,).toBe(1,);
            expect(captured.info.length,).toBe(1,);
            expect(captured.debug.length,).toBe(1,);
            expect(captured.debug[0]?.includes('glm-5.2'),).toBe(true,);
            expect(captured.warn,).toEqual([],);
            expect(summary.liveModelRetirement,).toBe(undefined,);
          },
        },),
        it({
          name: 'uses the configuration mechanism when pi exposes one',
          fn: async function runConfigRegistration() {
            /**
             Registrations the pass made.
             */
            const recorded: RecordedRegistrations = { configs: [], wrappers: [], };
            /**
             Pass summary over a configuration-registered provider.
             */
            const summary = await applyRetirements({
              refresh: noopRefresh,
              read: readOf({
                chat: [
                  chatModel({ provider: 'synthetic', id: 'glm-5.2', },),
                  chatModel({ provider: 'synthetic', id: 'glm-5.3', },),
                ],
                configs: { synthetic: { api: 'openai-completions', }, },
              },),
              registrar: fakeRegistrar({ recorded, },),
              log: fakeLog(emptyCapture(),),
            },);
            expect(recorded.configs.length,).toBe(1,);
            expect(recorded.wrappers,).toEqual([],);
            expect(registeredModelIds(nonNullishOrThrow(recorded.configs[0],).config,),).toEqual([
              'glm-5.3',
            ],);
            expect(summary.registeredProviders,).toEqual([{
              provider: 'synthetic',
              kind: 'config',
            }],);
          },
        },),
        it({
          name: 'warns when the session started on a retired model',
          fn: async function runLiveModelWarning() {
            /**
             Capture arrays for this pass.
             */
            const captured = emptyCapture();
            /**
             Registrations the pass made.
             */
            const recorded: RecordedRegistrations = { configs: [], wrappers: [], };
            /**
             Pass summary with a live model the rule retires.
             */
            const summary = await applyRetirements({
              refresh: noopRefresh,
              read: readOf({
                chat: [
                  chatModel({ provider: 'hyper', id: 'glm-5.2', },),
                  chatModel({ provider: 'hyper', id: 'glm-5.3', },),
                ],
                composed: {
                  hyper: composedProvider({
                    provider: 'hyper',
                    models: [chatModel({ provider: 'hyper', id: 'glm-5.3', },)],
                  },),
                },
              },),
              registrar: fakeRegistrar({ recorded, },),
              liveModel: { provider: 'hyper', id: 'glm-5.2', },
              log: fakeLog(captured,),
            },);
            expect(captured.warn.length,).toBe(1,);
            expect(captured.warn[0]?.includes('glm-5.3'),).toBe(true,);
            expect(summary.liveModelRetirement?.keeperId,).toBe('glm-5.3',);
          },
        },),
        it({
          name: 'stays quiet when the live model is the keeper',
          fn: async function runLiveModelKept() {
            /**
             Capture arrays for this pass.
             */
            const captured = emptyCapture();
            /**
             Registrations the pass made.
             */
            const recorded: RecordedRegistrations = { configs: [], wrappers: [], };
            /**
             Pass summary with a live model the rule keeps.
             */
            const summary = await applyRetirements({
              refresh: noopRefresh,
              read: readOf({
                chat: [
                  chatModel({ provider: 'hyper', id: 'glm-5.2', },),
                  chatModel({ provider: 'hyper', id: 'glm-5.3', },),
                ],
                composed: {
                  hyper: composedProvider({
                    provider: 'hyper',
                    models: [chatModel({ provider: 'hyper', id: 'glm-5.3', },)],
                  },),
                },
              },),
              registrar: fakeRegistrar({ recorded, },),
              liveModel: { provider: 'hyper', id: 'glm-5.3', },
              log: fakeLog(captured,),
            },);
            expect(captured.warn,).toEqual([],);
            expect(summary.liveModelRetirement,).toBe(undefined,);
          },
        },),
        it({
          name: 'awaits the catalog refresh before reading',
          fn: async function runRefreshOrder() {
            /**
             Order the pass touched its dependencies in.
             */
            const order: string[] = [];
            await applyRetirements({
              read: {
                readChatModels: function readChatModels() {
                  order.push('read',);
                  return [];
                },
                readImageModels: function readImageModels() {
                  return [];
                },
                readClassifierModels: function readClassifierModels() {
                  return [];
                },
                readProviderConfig: function readProviderConfig(): IncumbentConfig {
                  return { kind: 'absent', };
                },
                readComposedProvider: function readComposedProvider(): ComposedProvider {
                  return { kind: 'absent', };
                },
              },
              refresh: function recordingRefresh() {
                order.push('refresh',);
                return Promise.resolve();
              },
              registrar: fakeRegistrar({
                recorded: { configs: [], wrappers: [], },
              },),
              log: fakeLog(emptyCapture(),),
            },);
            expect(order,).toEqual(['refresh', 'read'],);
          },
        },),
        it({
          name: 'keeps filtering when the catalog refresh fails',
          fn: async function runRefreshFailure() {
            /**
             Capture arrays for this pass.
             */
            const captured = emptyCapture();
            /**
             Registrations the pass made.
             */
            const recorded: RecordedRegistrations = { configs: [], wrappers: [], };
            /**
             Pass summary over a refresh that rejects.
             */
            const summary = await applyRetirements({
              refresh: function failingRefresh() {
                return Promise.reject(new Error('models.json unreadable',),);
              },
              read: readOf({
                chat: [
                  chatModel({ provider: 'hyper', id: 'glm-5.2', },),
                  chatModel({ provider: 'hyper', id: 'glm-5.3', },),
                ],
                composed: {
                  hyper: composedProvider({
                    provider: 'hyper',
                    models: [chatModel({ provider: 'hyper', id: 'glm-5.3', },)],
                  },),
                },
              },),
              registrar: fakeRegistrar({ recorded, },),
              log: fakeLog(captured,),
            },);
            expect(summary.registeredProviders.length,).toBe(1,);
            expect(captured.warn.length,).toBe(1,);
            expect(captured.warn[0]?.includes('models.json unreadable',),).toBe(true,);
          },
        },),
        it({
          name: 'records a provider pi refused and still filters the rest',
          fn: async function runRegistrationFailure() {
            /**
             Capture arrays for this pass.
             */
            const captured = emptyCapture();
            /**
             Registrations the pass made.
             */
            const recorded: RecordedRegistrations = { configs: [], wrappers: [], };
            /**
             Pass summary where one provider throws on registration.
             */
            const summary = await applyRetirements({
              refresh: noopRefresh,
              read: readOf({
                chat: [
                  chatModel({ provider: 'azure-openai-responses', id: 'gpt-4.1', },),
                  chatModel({ provider: 'azure-openai-responses', id: 'gpt-5.5', },),
                  chatModel({ provider: 'synthetic', id: 'glm-5.2', },),
                  chatModel({ provider: 'synthetic', id: 'glm-5.3', },),
                ],
                configs: {
                  'azure-openai-responses': {},
                  synthetic: {},
                },
              },),
              registrar: fakeRegistrar({
                recorded,
                rejectProvider: 'azure-openai-responses',
              },),
              log: fakeLog(captured,),
            },);
            expect(summary.registeredProviders,).toEqual([{
              provider: 'synthetic',
              kind: 'config',
            }],);
            expect(summary.failedProviders.length,).toBe(1,);
            expect(summary.failedProviders[0]?.provider,).toBe('azure-openai-responses',);
            expect(summary.failedProviders[0]?.reason.includes('baseUrl',),).toBe(true,);
            expect(recorded.configs.length,).toBe(1,);
            expect(captured.warn.length,).toBe(1,);
          },
        },),
        it({
          name: 'registers nothing when no family is superseded',
          fn: async function runNoRetirement() {
            /**
             Capture arrays for this pass.
             */
            const captured = emptyCapture();
            /**
             Registrations the pass made.
             */
            const recorded: RecordedRegistrations = { configs: [], wrappers: [], };
            /**
             Pass summary over two models in different families.
             */
            const summary = await applyRetirements({
              refresh: noopRefresh,
              read: readOf({
                chat: [
                  chatModel({ provider: 'hyper', id: 'glm-5.3', },),
                  chatModel({ provider: 'hyper', id: 'glm-5.3-flash', },),
                ],
                configs: { hyper: {}, },
              },),
              registrar: fakeRegistrar({ recorded, },),
              log: fakeLog(captured,),
            },);
            expect(recorded.configs,).toEqual([],);
            expect(recorded.wrappers,).toEqual([],);
            expect(summary.retirements,).toEqual([],);
            expect(captured.info.length,).toBe(1,);
          },
        },),
      ],
    },),
    describe({
      name: registerModelRetirement.name,
      children: [
        it({
          name: 'registers only a session-start handler',
          fn: async function runHandlerRegistration() {
            /**
             State the fake host records into.
             */
            const state: FakeHostState = {
              events: [],
              configs: [],
              wrappers: [],
            };
            registerModelRetirement({ pi: fakeHost(state,), },);
            expect(state.events,).toEqual(['session_start'],);
            expect(state.configs,).toEqual([],);
            expect(state.wrappers,).toEqual([],);
          },
        },),
        it({
          name: 'filters the catalog when the session starts',
          fn: async function runHandlerInvocation() {
            /**
             State the fake host records into.
             */
            const state: FakeHostState = {
              events: [],
              configs: [],
              wrappers: [],
            };
            registerModelRetirement({ pi: fakeHost(state,), },);
            /**
             Chat models the session registry reports.
             */
            const chat = [
              chatModel({ provider: 'hyper', id: 'glm-5.2', },),
              chatModel({ provider: 'hyper', id: 'glm-5.3', },),
            ];
            await state.handler?.({ type: 'session_start', }, sessionContext({
              chat,
              composed: composedProvider({ provider: 'hyper', models: chat, },),
              model: chatModel({ provider: 'hyper', id: 'glm-5.3', },),
            },),);
            expect(state.wrappers.length,).toBe(1,);
            expect(state.configs,).toEqual([],);
            expect(nonNullishOrThrow(state.wrappers[0],).provider.getModels().map(
              function toId(model,) {
                return model.id;
              },
            ),).toEqual(['glm-5.3'],);
          },
        },),
        it({
          name: 'tolerates a session without a live model or a composed provider',
          fn: async function runHandlerWithoutModel() {
            /**
             State the fake host records into.
             */
            const state: FakeHostState = {
              events: [],
              configs: [],
              wrappers: [],
            };
            registerModelRetirement({ pi: fakeHost(state,), },);
            await state.handler?.({ type: 'session_start', }, sessionContext({
              chat: [chatModel({ provider: 'hyper', id: 'glm-5.3', },)],
            },),);
            expect(state.configs,).toEqual([],);
            expect(state.wrappers,).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
