/**
 Verifies the built model-retirement extension registers and filters as designed.

 Runs against `dist/final/node/index.mjs`, the artifact pi loads, so a build that drops the
 handler, the wrapper path, or the configuration path fails here rather than in a live
 session.

 @module
 */

import type {
  ExtensionAPI,
  ProviderConfig,
  ProviderModelConfig,
} from '@earendil-works/pi-coding-agent';
import type {
  Api,
  Model,
  Provider,
} from '@earendil-works/pi-ai';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Constants

/**
 Built extension path pi consumes.
 */
const BUILT_EXTENSION_PATH = '../dist/final/node/index.mjs';

/**
 Provider the fixture catalog uses.
 */
const FIXTURE_PROVIDER = 'hyper';

/**
 Id the rule must retire in the fixture catalog.
 */
const RETIRED_MODEL_ID = 'glm-5.2';

/**
 Id the rule must keep in the fixture catalog.
 */
const KEPT_MODEL_ID = 'glm-5.3';

/**
 Image model the fixture catalog carries, which must survive a configuration plan.
 */
const IMAGE_MODEL_ID = 'flux';

/**
 Classifier model the fixture catalog carries, which must survive a configuration plan.
 */
const CLASSIFIER_MODEL_ID = 'judge';

//endregion Constants

//region Types

/**
 Built model-retirement extension module shape.
 */
type ModelRetirementModule = {
  /**
   Pi extension factory.
   */
  readonly default: (pi: ExtensionAPI,) => Promise<void>;
};

/**
 Registration recorded by the fake host.
 */
type RecordedRegistration = {
  /**
   Provider name for a configuration registration, absent for a wrapped provider.
   */
  readonly name?: string;
  /**
   Configuration for a configuration registration.
   */
  readonly config?: ProviderConfig;
  /**
   Wrapped provider for a native registration.
   */
  readonly wrapped?: Provider;
};

/**
 State the fake host accumulates.
 */
type HarnessState = {
  readonly events: string[];
  readonly registrations: RecordedRegistration[];
  handler?: (
    payload: unknown,
    ctx: unknown,
  ) => Promise<void>;
};

//endregion Types

//region Fixtures

/**
 Build one chat model fixture for the fake registry.

 @param id - model id

 @returns chat model shaped like a registry read
 */
function chatModel(id: string,): Model<Api> {
  return {
    id,
    name: id,
    api: 'openai-completions',
    provider: FIXTURE_PROVIDER,
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
 Chat models the fixture registry reports, one retired and one kept.

 @returns chat models in registry order
 */
function fixtureChatModels(): Model<Api>[] {
  return [
    chatModel(RETIRED_MODEL_ID,),
    chatModel(KEPT_MODEL_ID,),
  ];
}

/**
 Image model the fixture registry reports, which must survive a configuration plan.

 @returns image model fixture
 */
function fixtureImageModel(): unknown {
  return {
    id: IMAGE_MODEL_ID,
    name: IMAGE_MODEL_ID,
    api: 'openrouter-images',
    provider: FIXTURE_PROVIDER,
    baseUrl: 'https://example.invalid',
    input: [
      'text',
      'image'
    ],
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
 Classifier model the fixture registry reports, which must survive a configuration plan.

 @returns classifier model fixture
 */
function fixtureClassifierModel(): unknown {
  return {
    id: CLASSIFIER_MODEL_ID,
    name: CLASSIFIER_MODEL_ID,
    api: 'typesafe-system-one',
    provider: FIXTURE_PROVIDER,
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
 Read the non-chat models the fixture registry reports for one type.

 @param type - model type the registry was asked for

 @returns image models for `image`, classifier models for anything else
 */
function fixtureModelsOfType(type: string,): readonly unknown[] {
  if (type === 'image')
    return [fixtureImageModel(),];
  return [fixtureClassifierModel(),];
}

/**
 Refresh stub standing in for pi's asynchronous `models.json` reload.

 @returns resolved refresh result
 */
function fixtureRefresh(): Promise<unknown> {
  return Promise.resolve({
    aborted: false,
    errors: new Map(),
  },);
}

/**
 Build the composed provider the fixture registry returns.

 @returns provider object with a stream member the wrapper must preserve
 */
/**
 Stand-in stream member the wrapper must preserve by reference.

 @returns marker string, never called by this verifier
 */
function fixtureStream(): string {
  return 'stream';
}

/**
 Build the composed provider the fixture registry returns.

 @returns provider object with a stream member the wrapper must preserve
 */
function fixtureComposedProvider(): unknown {
  return {
    id: FIXTURE_PROVIDER,
    name: FIXTURE_PROVIDER,
    auth: { name: 'fixture-auth', },
    stream: fixtureStream,
    getModels: fixtureChatModels,
  };
}

/**
 Build the session-start context the handler receives.

 @param composed - whether the registry should return a composed provider

 @param config - whether the registry should return an incumbent configuration

 @returns context carrying a fixture registry and no live model
 */
function fixtureContext(
  {
    composed,
    config,
  }: {
    readonly composed: boolean;
    readonly config: boolean;
  },
): unknown {
  return {
    modelRegistry: {
      getAll: fixtureChatModels,
      getModelsOfType: fixtureModelsOfType,
      refresh: fixtureRefresh,
      getRegisteredProviderConfig: function getRegisteredProviderConfig() {
        return config ? { api: 'openai-completions', } : undefined;
      },
      getProvider: function getProvider() {
        return composed ? fixtureComposedProvider() : undefined;
      },
    },
    model: undefined,
  };
}

/**
 Build the fake host and its recorded state.

 @param state - state the fake writes to

 @returns fake extension API implementing only what this extension touches
 */
function fakeHost(state: HarnessState,): ExtensionAPI {
  /**
   Object implementing `on` and both `registerProvider` overloads.
   */
  const fake = {
    on(
      event: string,
      handler: (
        payload: unknown,
        ctx: unknown,
      ) => Promise<void>,
    ): void {
      state.events
        .push(event,);
      state.handler = handler;
    },
    registerProvider(
      nameOrProvider: string | Provider,
      config?: ProviderConfig,
    ): void {
      if ((typeof nameOrProvider) === 'string') {
        state.registrations
          .push({
            name: nameOrProvider,
            ...(config === undefined ? {} : { config, }),
          },);
        return;
      }
      state.registrations
        .push({ wrapped: nameOrProvider, },);
    },
  };
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the verifier implements only the members the built extension touches
  return fake as unknown as ExtensionAPI;
}

/**
 Read the ids one configuration registration carries.

 @param config - recorded provider configuration

 @returns model ids in registration order
 */
function registeredIds(config: ForeignBorrowed<ProviderConfig>,): string[] {
  return (config.models ?? []).map(function toId(
    model: ForeignBorrowed<ProviderModelConfig>,
  ) {
    return model.id;
  },);
}

//endregion Fixtures

//region Verification

/**
 Test whether an imported module is this extension's built shape.

 @param mod - value the dynamic import returned

 @returns whether the module exports a default factory
 */
function isModelRetirementModule(mod: unknown,): mod is ModelRetirementModule {
  return ((typeof mod) === 'object') && (mod !== null)
    && ((typeof (mod as { readonly default?: unknown; }).default) === 'function');
}

/**
 Run the built factory against one fake host and return its recorded state.

 @param mod - built extension module

 @param context - session-start context the handler receives

 @returns state the fake host recorded

 @throws when the factory registers anything other than one session-start handler
 */
async function runFactory(
  {
    mod,
    context,
  }: {
    readonly mod: ModelRetirementModule;
    readonly context: unknown;
  },
): Promise<HarnessState> {
  /**
   State the fake host records into.
   */
  const state: HarnessState = {
    events: [],
    registrations: [],
  };
  await mod.default(fakeHost(state,),);
  if ((state.events
    .length
    !== 1) || (state.events[0] !== 'session_start'))
    throw new Error(`built extension must register only session_start, saw ${state.events
      .join(', ')}`,);
  if (state.registrations
    .length
    > 0)
    throw new Error('factory must not register providers before a session starts',);
  if (state.handler === undefined)
    throw new Error('built extension did not capture its session_start handler',);
  await state.handler(
    { type: 'session_start', },
    context,
  );
  return state;
}

/**
 Verify the built artifact end to end against a fake host.

 @returns verification result text

 @throws when the artifact exports no factory, registers unexpected events, or filters a
 provider incorrectly through either mechanism

 @example
 ```typescript
 console.log(await verifyBuiltExtension());
 */
async function verifyBuiltExtension(): Promise<string> {
  /**
   Built extension module imported through package output.
   */
  const mod: unknown = await import(BUILT_EXTENSION_PATH);
  if (!isModelRetirementModule(mod,))
    throw new Error('built model-retirement extension does not export a default factory',);
  /**
   State after a session start where pi exposes only a composed provider.
   */
  const wrappedState = await runFactory({
    mod,
    context: fixtureContext({
      composed: true,
      config: false,
    },),
  },);
  if (wrappedState.registrations
    .length
    !== 1)
    throw new Error(`wrapper path must register one provider, saw ${String(wrappedState.registrations
      .length,)}`,);
  /**
   Provider the wrapper path registered.
   */
  const {wrapped} = nonNullishOrThrow(wrappedState.registrations[0],);
  if (wrapped === undefined)
    throw new Error('wrapper path registered a configuration instead of a provider object',);
  /**
   Ids the wrapped provider lists.
   */
  const wrappedIds = wrapped.getModels()
    .map(function toId(model: ForeignBorrowed<Model<Api>>,) {
      return model.id;
    },);
  if (wrappedIds.includes(RETIRED_MODEL_ID,))
    throw new Error(`wrapped provider still lists retired ${RETIRED_MODEL_ID}`,);
  if (!wrappedIds.includes(KEPT_MODEL_ID,))
    throw new Error(`wrapped provider lost ${KEPT_MODEL_ID}`,);
  if ((wrapped.id !== FIXTURE_PROVIDER) || ((typeof wrapped.stream) !== 'function'))
    throw new Error('wrapped provider did not preserve the original id and stream member',);
  /**
   State after a session start where pi exposes an incumbent configuration.
   */
  const configState = await runFactory({
    mod,
    context: fixtureContext({
      composed: true,
      config: true,
    },),
  },);
  if (configState.registrations
    .length
    !== 1)
    throw new Error(`configuration path must register one provider, saw ${String(configState.registrations
      .length,)}`,);
  /**
   Configuration the configuration path registered.
   */
  const {config} = nonNullishOrThrow(configState.registrations[0],);
  if (config === undefined)
    throw new Error('configuration path registered a provider object instead of a configuration',);
  /**
   Ids the configuration carries.
   */
  const ids = registeredIds(config,);
  if (ids.length === 0)
    throw new Error('a registration must never carry an empty model list',);
  if (ids.includes(RETIRED_MODEL_ID,))
    throw new Error(`registration still carries retired ${RETIRED_MODEL_ID}`,);
  for (const required of [
    KEPT_MODEL_ID,
    IMAGE_MODEL_ID,
    CLASSIFIER_MODEL_ID,
  ]) {
    if (!ids.includes(required,))
      throw new Error(`registration lost ${required}; image and classifier models must pass through`,);
  }
  return `model-retirement extension verified: session_start only, wrapper path kept ${wrappedIds.join(', ')}, configuration path kept ${ids.join(', ')}`;
}

console.log(await verifyBuiltExtension(),);

//endregion Verification
