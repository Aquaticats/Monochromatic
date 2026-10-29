/**
 Priority entry points delegate transport and accounting to native OpenAI streaming. @module
 */
import {
  hasApi,
  type Api,
  type AssistantMessageEventStream,
  type Model,
  type SimpleStreamOptions,
  type StreamFunction,
  type StreamOptions,
  type TranscriptContext,
} from '@earendil-works/pi-ai';
import {
  type OpenAICodexResponsesOptions,
  stream as streamCodex,
} from '@earendil-works/pi-ai/api/openai-codex-responses';
import {
  type OpenAIResponsesOptions,
  stream as streamOpenAI,
} from '@earendil-works/pi-ai/api/openai-responses';
import { buildBaseOptions, } from '@earendil-works/pi-ai/api/simple-options';
import { clampThinkingLevel, } from '@earendil-works/pi-ai/models';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  CODEX_API,
  OPENAI_API,
  type PriorityApi,
} from './constants.ts';
import { PriorityRequestError, } from './priority-error.ts';
import { forcePriorityPayload, } from './priority-payload.ts';

//region Full stream: preserve native options and live model identity.

/**
 Module logger never receives payloads, options, or credentials.
 */
const l = tagged({ tag: 'openai-fast/priority-stream', },);

/**
 Delegate directly to the original model's native API when no registry dispatch is injected.

 @param model - original native model whose API determines transport

 @param context - normalized transcript retained by native streaming

 @param options - original options including provider-specific runtime fields

 @returns native stream without model or transport translation
 */
function nativeStream({
  model,
  context,
  options,
}: {
  readonly model: ForeignHostCapability<Model<PriorityApi>>;
  readonly context: ForeignHostCapability<TranscriptContext>;
  readonly options?: ForeignHostCapability<StreamOptions>;
},): AssistantMessageEventStream {
  /**
   Direct transport logger records only the original API identity.
   */
  const innerL = tagged({
    tag: nativeStream.name,
    l,
  },);
  innerL.debug(`delegating direct request to ${model.api}`,);
  if (hasApi(
    model,
    CODEX_API,
  ))
    return streamCodex(
      model,
      context,
      options,
    );
  if (!hasApi(
    model,
    OPENAI_API,
  ))
    throw new PriorityRequestError({ message: `Unsupported native priority API: ${model.api}`, },);
  return streamOpenAI(
    model,
    context,
    options,
  );
}

/**
 Delegate full native OpenAI options with priority in both the options and final request body.
 Model must be the live base OpenAI model, not a virtual routing model.

 @param model - original base model used by native request building and accounting

 @param context - normalized transcript supplied by native provider dispatch

 @param options - complete native options; only tier and payload composition are overridden

 @param stream - full native-compatible dispatch, optionally through the original model registry

 @returns native OpenAI event stream without an alternate transport or model fallback

 @mutates model - native stream and caller callbacks may inspect or retain live model data

 @mutates context - native stream consumes transcript data and its reachable foreign values

 @mutates options - native stream consumes transport, cancellation, and caller callback capabilities

 @mutates stream - dispatch may invoke or retain native transport and runtime authentication capabilities

 @example
 ```ts
 const events = streamPriority({ model: baseModel, context, options });
 ```
 */
export function streamPriority<const TApi extends PriorityApi>({
  model,
  context,
  options,
  stream,
}: {
  readonly model: ForeignHostCapability<Model<TApi>>;
  readonly context: ForeignHostCapability<TranscriptContext>;
  readonly options?: ForeignHostCapability<StreamOptions | OpenAICodexResponsesOptions | OpenAIResponsesOptions>;
  readonly stream?: ForeignHostCapability<StreamFunction<TApi>>;
},): AssistantMessageEventStream {
  /**
   Function logger records delegation without sensitive request data.
   */
  const innerL = tagged({
    tag: streamPriority.name,
    l,
  },);
  innerL.debug(`delegating priority request to native ${model.api} stream`,);
  /**
   Capture caller customization when options are composed, before registry dispatch can defer it.
   */
  const onPayload = options?.onPayload;
  /**
   Composed native options retain all caller fields while enforcing request intent.
   */
  const priorityOptions = {
    ...options,
    serviceTier: 'priority' as const,
    /**
     Native callback signature retains native model and payload identity for caller customization.

     @param payload - native request being customized

     @param requestModel - exact live model supplied by native streaming

     @returns customized record with priority enforced

     @mutates payload - caller payload callback may mutate or retain request data

     @mutates requestModel - caller payload callback may inspect or retain model data
     */
    onPayload: async function priorityPayload(
      payload: unknown,
      requestModel: ForeignHostCapability<Model<Api>>,
    ): Promise<Readonly<Record<string, unknown>> & { readonly service_tier: 'priority'; }> {
      /**
       Callback logger extends the stream function boundary.
       */
      const callbackL = tagged({
        tag: priorityPayload.name,
        l: innerL,
      },);
      callbackL.debug('composing priority payload callback',);
      return await forcePriorityPayload({
        payload,
        model: requestModel,
        onPayload,
      },);
    },
  };
  if (stream !== undefined)
    return stream(
      model,
      context,
      priorityOptions,
    );
  return nativeStream({
    model,
    context,
    options: priorityOptions,
  },);
}

//endregion Full stream

//region Simple stream: mirror native reasoning conversion before full delegation.

/**
 Use native simple-option preparation and reasoning clamping, then delegate to the full priority stream.
 Converting through native streamSimple would discard the priority accounting option.

 @param model - original base OpenAI model carrying current reasoning capabilities

 @param context - normalized native transcript

 @param options - provider-neutral options preserved through native buildBaseOptions

 @param stream - full native-compatible dispatch, optionally through the original model registry

 @returns native OpenAI event stream with priority request and accounting intent

 @mutates model - native option preparation and streaming may inspect or retain model data

 @mutates context - native option preparation and streaming consume transcript data

 @mutates options - native preparation and streaming consume transport and callback capabilities

 @mutates stream - full dispatch may invoke or retain transport and runtime authentication capabilities

 @throws {@link PriorityRequestError} when direct native Codex dispatch has no resolved credential

 @example
 ```ts
 const events = streamSimplePriority({ model: baseModel, context, options });
 ```
 */
export function streamSimplePriority<const TApi extends PriorityApi>({
  model,
  context,
  options,
  stream,
}: {
  readonly model: ForeignHostCapability<Model<TApi>>;
  readonly context: ForeignHostCapability<TranscriptContext>;
  readonly options?: ForeignHostCapability<SimpleStreamOptions>;
  readonly stream?: ForeignHostCapability<StreamFunction<TApi>>;
},): AssistantMessageEventStream {
  /**
   Function logger records conversion without exposing authentication data.
   */
  const innerL = tagged({
    tag: streamSimplePriority.name,
    l,
  },);
  innerL.debug(`preparing native simple ${model.api} options`,);
  /**
   Direct native dispatch requires a token; injected registry dispatch resolves it later.
   */
  const apiKey = options?.apiKey;
  if ((stream === undefined) && (model.api === CODEX_API)
    && ((apiKey === undefined) || (apiKey === ''))) {
    innerL.error('resolved native Codex credential is missing',);
    throw new PriorityRequestError({ message: `No API key for provider: ${model.provider}`, },);
  }
  /**
   Match native clamping, including conversion of unsupported reasoning to off.
   */
  const clampedReasoning = options?.reasoning ? clampThinkingLevel(
    model,
    options.reasoning,
  ) : undefined;
  /**
   Native off semantics omit reasoningEffort rather than sending the string off.
   */
  const reasoningEffort = clampedReasoning === 'off' ? undefined : clampedReasoning;
  innerL.debug(reasoningEffort === undefined ? 'delegating without explicit reasoning effort' : 'delegating with native clamped reasoning effort',);
  return streamPriority({
    model,
    context,
    ...(stream === undefined ? {} : { stream, }),
    options: {
      ...buildBaseOptions(
        model,
        context,
        options,
        apiKey,
      ),
      ...(options?.toolChoice === undefined ? {} : { toolChoice: options.toolChoice, }),
      ...(reasoningEffort === undefined ? {} : { reasoningEffort, }),
      serviceTier: 'priority',
    },
  },);
}

//endregion Simple stream
