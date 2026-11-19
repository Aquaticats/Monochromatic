/**
 Stand-in pool members the prototype's probe sessions route to, with no network behind them.
 @module
 */

import {
  createAssistantMessageEventStream,
  type Api,
  type AssistantMessage,
  type AssistantMessageEventStream,
  type Model,
  type Provider,
  type SimpleStreamOptions,
  type StreamOptions,
  type TranscriptContext,
} from '@earendil-works/pi-ai';
import {
  CLOUDFLARE_523_REFUSAL,
  NEURALWATT_OVERAGE_REFUSAL,
  overflowRefusal,
  SYNTHETIC_QUOTA_REFUSAL,
} from './captured-refusal-text.ts';
import {
  type CatalogLimitsView,
  limitsOf,
  note,
} from './prototype-evidence.ts';

//region Identity and limits

/**
 Provider id of the stand-in members, distinct from every real member provider.
 */
export const STAND_IN_PROVIDER_ID = 'hybrid-probe-member';

/**
 Window every stand-in member declares, matching the real members' catalog window so Pi's
 pre-flight compaction leaves the probe's transcript alone.
 */
const DECLARED_CONTEXT_WINDOW = 1_000_000;

/**
 Output limit every stand-in member declares.
 */
const DECLARED_MAX_TOKENS = 16_384;

/**
 Characters the small-window member accepts before it refuses: smaller than what it declares, the
 way a real member reserves room for its output. The probe pads a turn past this and lets Pi compact.
 */
const SMALL_WINDOW_ACCEPTS_CHARS = 140_000;

/**
 Stand-in member that always answers.
 */
export const ANSWERING_MEMBER = 'answers';

/**
 Stand-in member whose window is smaller than it declares.
 */
export const SMALL_WINDOW_MEMBER = 'small-window';

/**
 Stand-in member that refuses like Synthetic at its subscription limit.
 */
export const QUOTA_REFUSING_MEMBER = 'refuses-quota';

/**
 Stand-in member that refuses like NeuralWatt at its concurrent budget.
 */
export const OVERAGE_REFUSING_MEMBER = 'refuses-neuralwatt-429';

/**
 Stand-in member that refuses like Cloudflare's unreachable origin.
 */
export const TIMEOUT_REFUSING_MEMBER = 'refuses-cloudflare-523';

/**
 Behavior each stand-in member id selects.
 */
const BEHAVIORS = [
  ANSWERING_MEMBER,
  SMALL_WINDOW_MEMBER,
  QUOTA_REFUSING_MEMBER,
  OVERAGE_REFUSING_MEMBER,
  TIMEOUT_REFUSING_MEMBER,
] as const;

//endregion Identity and limits

//region Views the prototype reads

/**
 Which member a stream belongs to, read-only.

 Project-owned so a catalog model is assignable to it without the prototype claiming its fields.
 */
type MemberIdentity = {
  readonly provider: string;
  readonly id: string;
  readonly api: string
};

/**
 Catalog record a stand-in member is dispatched as, read-only.

 Project-owned so Pi's model type is assignable to it and this file never claims its fields.
 */
type StandInModelView = CatalogLimitsView & {
  readonly api: string
};

/**
 Transcript size source: the prototype counts characters and reads nothing else.

 Project-owned, so Pi's transcript type is assignable to it.
 */
type TranscriptView = {
  readonly messages: unknown
};

/**
 Request options the prototype reads from Pi's own request.

 Project-owned, so Pi's options type is assignable to it.
 */
type RequestOptionsView = {
  readonly maxTokens?: number
};

//endregion Views the prototype reads

//region Streams from stand-in members

/**
 Empty usage, since a stand-in member bills nothing.
 */
const NO_USAGE = {
  input: 0,
  output: 0,
  cacheRead: 0,
  cacheWrite: 0,
  totalTokens: 0,
  cost: {
    input: 0,
    output: 0,
    cacheRead: 0,
    cacheWrite: 0,
    total: 0,
  },
} as const;

/**
 Build the assistant message a stand-in member reports.

 @param model - member its own stream names

 @param text - answer text, or empty for a refusal

 @param errorMessage - refusal text when the member refuses

 @returns terminal assistant message

 @example
 ```ts
 messageOf({ model, text: 'ok' });
 ```
 */
function messageOf({
  model,
  text,
  errorMessage,
}: {
  readonly model: MemberIdentity;
  readonly text: string;
  readonly errorMessage?: string;
},): AssistantMessage {
  return {
    role: 'assistant',
    provider: model.provider,
    model: model.id,
    api: model.api,
    content: text === '' ? [] : [
      {
        type: 'text',
        text,
      },
    ],
    usage: NO_USAGE,
    stopReason: errorMessage === undefined ? 'stop' : 'error',
    ...(errorMessage === undefined ? {} : { errorMessage, }),
    timestamp: Date.now(),
  };
}

/**
 Report one refusal as the stream event Pi's error path reads.

 @param model - member that refused

 @param errorMessage - refusal text, captured or from its producer's own source

 @returns stream whose only content is the refusal

 @example
 ```ts
 refusalStream({ model, errorMessage: SYNTHETIC_QUOTA_REFUSAL });
 ```
 */
function refusalStream({
  model,
  errorMessage,
}: {
  readonly model: MemberIdentity;
  readonly errorMessage: string;
},): AssistantMessageEventStream {
  /**
   Stream the pool reads, with no output before the refusal.
   */
  const stream = createAssistantMessageEventStream();
  /**
   Member-attributed refusal.
   */
  const message = messageOf({
    model,
    text: '',
    errorMessage,
  },);
  stream.push({
    type: 'start',
    partial: message,
  },);
  stream.push({
    type: 'error',
    reason: 'error',
    error: message,
  },);
  stream.end(message,);
  return stream;
}

/**
 Report one answer as the stream events Pi's done path reads.

 @param model - member that answered

 @param text - answer text

 @returns stream that completes normally

 @example
 ```ts
 answerStream({ model, text: 'Stand-in answer.' });
 ```
 */
function answerStream({
  model,
  text,
}: {
  readonly model: MemberIdentity;
  readonly text: string;
},): AssistantMessageEventStream {
  /**
   Stream the pool reads to its terminal event.
   */
  const stream = createAssistantMessageEventStream();
  /**
   Partial message the answer is built into.
   */
  const message = messageOf({
    model,
    text,
  },);
  stream.push({
    type: 'start',
    partial: message,
  },);
  stream.push({
    type: 'text_start',
    contentIndex: 0,
    partial: message,
  },);
  stream.push({
    type: 'text_delta',
    contentIndex: 0,
    delta: text,
    partial: message,
  },);
  stream.push({
    type: 'text_end',
    contentIndex: 0,
    content: text,
    partial: message,
  },);
  stream.push({
    type: 'done',
    reason: 'stop',
    message,
  },);
  stream.end(message,);
  return stream;
}

/**
 Size of everything a member would receive, counted in characters.

 @param context - transcript the pool forwards

 @returns characters of the serialized messages

 @example
 ```ts
 requestChars({ messages: [] });
 ```
 */
function requestChars({ context }: { readonly context: TranscriptView },): number {
  return JSON.stringify(context.messages,)
    .length;
}

//endregion Streams from stand-in members

//region Provider

/**
 Catalog entry for one stand-in member.

 @param id - member id selecting its behavior

 @returns chat model the pool resolves through Pi's registry

 @example
 ```ts
 standInModel(SMALL_WINDOW_MEMBER);
 ```
 */
function standInModel(id: typeof BEHAVIORS[number],): Model<Api> {
  return {
    id,
    name: `Stand-in member: ${id}`,
    api: 'hybrid-probe-member',
    provider: STAND_IN_PROVIDER_ID,
    baseUrl: '',
    input: ['text',],
    reasoning: false,
    cost: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
    },
    contextWindow: DECLARED_CONTEXT_WINDOW,
    maxTokens: DECLARED_MAX_TOKENS,
  };
}

/**
 Decide what one stand-in member does with a request, and record it.

 @param model - member the pool resolved

 @param context - transcript the pool forwarded

 @param options - request options the pool forwarded after its own clamp

 @returns the member's stream

 @example
 ```ts
 playStandInMember({ model: standInModel(SMALL_WINDOW_MEMBER), context: { messages: [] } });
 ```
 */
function playStandInMember({
  model,
  context,
  options,
}: {
  readonly model: MemberIdentity & {
    readonly contextWindow: number;
    readonly maxTokens: number
  };
  readonly context: TranscriptView;
  readonly options?: RequestOptionsView;
},): AssistantMessageEventStream {
  /**
   Characters this request carries, the stand-in's only measure of size.
   */
  const chars = requestChars({ context, },);
  note({
    event: 'stand-in-request',
    member: `${model.provider}/${model.id}`,
    requestChars: chars,
    smallWindowAcceptsChars: SMALL_WINDOW_ACCEPTS_CHARS,
    maxTokens: options?.maxTokens,
    ...limitsOf({ model, }),
  },);
  if (model.id === QUOTA_REFUSING_MEMBER) {
    return refusalStream({
      model,
      errorMessage: SYNTHETIC_QUOTA_REFUSAL,
    },);
  }
  if (model.id === OVERAGE_REFUSING_MEMBER) {
    return refusalStream({
      model,
      errorMessage: NEURALWATT_OVERAGE_REFUSAL,
    },);
  }
  if (model.id === TIMEOUT_REFUSING_MEMBER) {
    return refusalStream({
      model,
      errorMessage: CLOUDFLARE_523_REFUSAL,
    },);
  }
  if ((model.id === SMALL_WINDOW_MEMBER) && (chars > SMALL_WINDOW_ACCEPTS_CHARS)) {
    return refusalStream({
      model,
      errorMessage: overflowRefusal({
        inputChars: chars,
        limitChars: SMALL_WINDOW_ACCEPTS_CHARS,
      },),
    },);
  }
  return answerStream({
    model,
    text: `Stand-in ${model.id} answered a ${chars}-character request.`,
  },);
}

/**
 Provider whose members answer, refuse or overflow without a network.

 @returns provider Pi registers beside the real ones

 @example
 ```ts
 registerProvider(standInProvider());
 ```
 */
export function standInProvider(): Provider {
  return {
    id: STAND_IN_PROVIDER_ID,
    name: 'Hybrid prototype stand-in members',
    auth: {
      apiKey: {
        name: 'Stand-in members',
        resolve() {
          return Promise.resolve({
            auth: {},
            source: STAND_IN_PROVIDER_ID,
          },);
        },
      },
    },
    getModels() {
      return BEHAVIORS.map(standInModel,);
    },
    stream(
      model: StandInModelView,
      context: TranscriptContext,
      options?: StreamOptions,
    ): AssistantMessageEventStream {
      return playStandInMember({
        model,
        context,
        ...(options === undefined ? {} : { options, }),
      },);
    },
    streamSimple(
      model: StandInModelView,
      context: TranscriptContext,
      options?: SimpleStreamOptions,
    ): AssistantMessageEventStream {
      return playStandInMember({
        model,
        context,
        ...(options === undefined ? {} : { options, }),
      },);
    },
  };
}

//endregion Provider
