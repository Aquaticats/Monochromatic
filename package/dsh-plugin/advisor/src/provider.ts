/**
 pi-ai `Provider` shim over the DSH LLM runtime for one provider route. @module
 */

import type {
  Api,
  ApiStreamOptions,
  AssistantMessageEventStream,
  Model,
  Provider,
  SimpleStreamOptions,
  TranscriptContext,
} from '@earendil-works/pi-ai';
import type {
  LlmProviderInfo,
  ReasoningEffortId,
} from '@deepseek-ai/dsh-llm';
import type {
  SessionId,
} from '@deepseek-ai/dsh-session/types';
import type {
  DshAdvisorLlmSurface,
} from './types.ts';
import { buildDshAdvisorGenerateOptions, } from './request-map.ts';
import {
  startDshAdvisorStream,
} from './provider-stream.ts';
import type {
  DshAdvisorStreamSurface,
} from './stream-driver.ts';

//region Types

/**
 Inputs for building one route-backed provider shim.
 */
export type CreateDshAdvisorProviderOptions = {
  /**
   DSH LLM surface owning every request this provider issues.
   */
  readonly llm: DshAdvisorLlmSurface;
  /**
   Provider route identity from `listProviders`.
   */
  readonly providerInfo: LlmProviderInfo;
  /**
   Catalog models belonging to this provider route.
   */
  readonly getModels: () => readonly Model<Api>[];
  /**
   Selected reasoning effort per canonical `provider/model` slug, from the
   DSH greatest-below-max rule.
   */
  readonly reasoningEffortByModelId: ReadonlyMap<string, ReasoningEffortId>;
  /**
   Session identity stamped onto every request this provider issues.
   */
  readonly sessionId: SessionId;
};

//endregion Types

//region Public API

/**
 DSH adapter credentials resolver for the pi-ai provider record.

 Authentication is owned by the DSH runtime behind the route, so this
 resolver reports no configured credentials to pi-ai credential readers
 (documented gap).

 @returns resolved credentials are always absent
 */
function resolveDshAdapterCredentials(): Promise<undefined> {
  return Promise.resolve(undefined,);
}

/**
 Build one pi-ai `Provider` view of one DSH provider route.

 Only the members the shared Advisor seam consumes carry real behavior:
 `getModels`, `stream`, and `streamSimple`. Authentication is owned by the DSH
 adapter behind the route, so `auth.resolve` reports the provider as
 unconfigured to pi-ai credential readers and `getApiKeyAndHeaders` on the
 catalog always succeeds without credentials.

 @param options - runtime, route identity, and catalog slices

 @returns provider shim dispatching through `ctx.llm.stream`

 @example
 ```ts
 const provider = createDshAdvisorProvider({ llm, providerInfo, sessionId, getModels, reasoningEffortByModelId });
 ```
 */
export function createDshAdvisorProvider(
  options: CreateDshAdvisorProviderOptions,
): Provider {
  return {
    id: options.providerInfo
      .id,
    name: options.providerInfo
      .name,
    auth: {
      apiKey: {
        name: 'DSH adapter credentials',
        resolve: resolveDshAdapterCredentials,
      },
    },
    getModels: options.getModels,
    stream: function stream<T extends Api>(
      model: Model<T>,
      context: TranscriptContext,
      streamOptions?: ApiStreamOptions<T>,
    ): AssistantMessageEventStream {
      return toAssistantMessageEventStream(
        startAdvisorRouteStream({
          options,
          model,
          context,
          streamOptions,
        },),
      );
    },
    streamSimple: function streamSimple(
      model: Model<Api>,
      context: TranscriptContext,
      streamOptions?: SimpleStreamOptions,
    ): AssistantMessageEventStream {
      return toAssistantMessageEventStream(
        startAdvisorRouteStream({
          options,
          model,
          context,
          streamOptions,
        },),
      );
    },
  };
}

//endregion Public API

//region Request dispatch

/**
 Forwarded request fields read from opaque pi-ai stream options.
 */
type AdvisorForwardedRequestOptions = {
  /**
   Output token cap requested by the caller, when present.
   */
  readonly maxTokens?: number;
  /**
   Sampling temperature requested by the caller, when present.
   */
  readonly temperature?: number;
  /**
   Cancellation signal requested by the caller, when present.
   */
  readonly signal?: AbortSignal;
};

/**
 One optional field read from an object source.
 */
type FieldRead<T> = {
  /**
   The field value, absent when the source lacks it or fails the guard.
   */
  readonly value?: T;
};

/**
 Whether one unknown field value is a number.

 @param value - raw field value

 @returns whether the value is a number
 */
function isNumberField(value: unknown): value is number {
  return (typeof value)
    === 'number';
}

/**
 Whether one unknown field value is an `AbortSignal`.

 @param value - raw field value

 @returns whether the value is an abort signal
 */
function isAbortSignalField(value: unknown): value is AbortSignal {
  return value instanceof AbortSignal;
}

/**
 Read one guarded field from an object source.

 @param source - object source known to be present

 @param key - field name to read

 @param guard - type guard validating the raw field value

 @returns optional field value after the guard
 */
function readField<T>(
  {
    source,
    key,
    guard,
  }: {
    readonly source: object;
    readonly key: string;
    readonly guard: (value: unknown) => value is T;
  },
): FieldRead<T> {
  if (!(key in source))
    return {};
  /**
   Field value after the presence check.
   */
  const value: unknown = Reflect.get(
    source,
    key,
  );
  return guard(value,)
    ? { value, }
    : {};
}

/**
 Read the forwarded request fields from opaque request options.

 The pi-ai `ApiStreamOptions` shape is api-parameterized and stays opaque
 inside the generic `stream` signature, so the forwarded fields are read with
 runtime narrowing instead of assertions.

 @param source - raw options object, possibly absent

 @returns forwarded fields present on the options
 */
function readAdvisorForwardedRequestOptions(
  source: unknown,
): AdvisorForwardedRequestOptions {
  if (((typeof source)
    !== 'object')
    || (source === null))
    return {};
  /**
   Present numeric output cap.
   */
  const maxTokens = readField({
    source,
    key: 'maxTokens',
    guard: isNumberField,
  },)
    .value;
  /**
   Present sampling temperature.
   */
  const temperature = readField({
    source,
    key: 'temperature',
    guard: isNumberField,
  },)
    .value;
  /**
   Present cancellation signal.
   */
  const signal = readField({
    source,
    key: 'signal',
    guard: isAbortSignalField,
  },)
    .value;
  return {
    ...(maxTokens === undefined
      ? {}
      : { maxTokens, }),
    ...(temperature === undefined
      ? {}
      : { temperature, }),
    ...(signal === undefined
      ? {}
      : { signal, }),
  };
}

/**
 Start one request through the route runtime with the DSH effort rule.

 The pi reasoning level on the stream options is advisory only: effort
 selection follows `LlmResolvedModelInfo.reasoning.efforts` (greatest effort
 not named `max`), omitting `reasoningEffort` on routes advertising none.

 @param options - provider construction state for the route

 @param model - pi-ai model record naming the DSH route

 @param context - normalized pi-ai transcript context

 @param streamOptions - opaque per-request options from the shared seam

 @returns stream surface for this request
 */
function startAdvisorRouteStream(
  {
    options,
    model,
    context,
    streamOptions,
  }: {
    readonly options: CreateDshAdvisorProviderOptions;
    readonly model: Model<Api>;
    readonly context: TranscriptContext;
    readonly streamOptions: unknown;
  },
): DshAdvisorStreamSurface {
  /**
   Selected reasoning effort for this route slug, absent when the route
   advertises none.
   */
  const effort = options
    .reasoningEffortByModelId
    .get(`${model.provider}/${model.id}`,);
  /**
   Forwarded request fields read through runtime narrowing.
   */
  const forwarded = readAdvisorForwardedRequestOptions(streamOptions,);
  return startDshAdvisorStream({
    llm: options.llm,
    generate: buildDshAdvisorGenerateOptions({
      context,
      model,
      sessionId: options.sessionId,
      ...(effort === undefined
        ? {}
        : { reasoningEffort: effort, }),
      ...forwarded,
    },),
  });
}

/**
 Present one adapter stream surface as the nominal pi-ai event stream type.

 `AssistantMessageEventStream` carries private class state, so a structural
 implementation cannot typecheck against it; the shared completion seam
 consumes only event iteration and `result()`, which the surface implements.
 This is the single documented assertion at the shim boundary (README
 capability gaps).

 @param surface - structural stream surface

 @returns surface typed as the pi-ai event stream
 */
function toAssistantMessageEventStream(
  surface: DshAdvisorStreamSurface,
): AssistantMessageEventStream {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- pi-ai AssistantMessageEventStream is a nominal class with private fields; the shared seam consumes only AsyncIterable and result(), which DshAdvisorStreamSurface implements, so the assertion restates the consumed contract at the one shim boundary. */
  return surface as AssistantMessageEventStream;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
}

//endregion Request dispatch
