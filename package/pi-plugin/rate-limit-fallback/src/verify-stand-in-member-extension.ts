/**
 Stand-in writer members for the real Pi sessions the extension check starts. No transport here reaches
 the network.
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
  type TranscriptContext,
} from '@earendil-works/pi-ai';

//region Behaviour the session chooses

/**
 Environment variable naming the behaviour every stand-in member plays.
 */
const BEHAVIOUR_VARIABLE = 'PI_STAND_IN_BEHAVIOUR';

/**
 Behaviour that answers every request.
 */
const ANSWERING = 'answer';

/**
 Behaviour whose head member refuses with NeuralWatt's recorded overage 429.
 */
const REFUSING_429 = 'refuse-429';

/**
 Behaviour whose head member refuses with a 503, the status a provider uses for fleet capacity.

 Made up: no capture of a provider's 503 exists on this machine. The boundary is the status a refusal
 opens with, in the installed Completions adapter's `<status>: <body>` shape
 (`@earendil-works/pi-ai` dist/utils/error-body.js, `formatProviderError`).
 */
const REFUSING_503 = 'refuse-503';

/**
 Behaviour whose head member refuses with Cloudflare's 523 page heading.
 */
const REFUSING_523 = 'refuse-523';

/**
 Behaviour whose head member refuses once a request outgrows the window it enforces.
 */
const OVERFLOWING = 'overflow';

/**
 Every behaviour a session may select, so a typo fails the run instead of answering.
 */
const BEHAVIOURS: ReadonlySet<string> = new Set([
  ANSWERING,
  REFUSING_429,
  REFUSING_503,
  REFUSING_523,
  OVERFLOWING,
],);

//endregion Behaviour the session chooses

//region Refusal texts

/**
 NeuralWatt's overage refusal, the 429 a writer member returns when its concurrent budget is full.

 Capture: `pi-runs/worktree-remove-does-not-wait-on-a-read-only-git-2/summary.json`, field
 `modelError`, 2026-11-10T21:11:23-05:00, 795 characters, recorded in `doc/captures.md` under
 "NeuralWatt's overage refusal".
 */
const OVERAGE_REFUSAL: string = '429: {"type":"rate_limit_error",'
  + '"code":"concurrent_budget_exceeded","message":"Concurrent limit reached for '
  + 'deepseek-ai/DeepSeek-V4.1-Flash: 6/6 slots in use. Wait for a request to complete and retry. '
  + 'Your subscription is in overage with auto-renew off, so pay-as-you-go limits apply until it '
  + 'renews. Turn auto-renew back on to restore your account level\'s limits: '
  + 'https://docs.neuralwatt.com/billing/account-levels#overage-with-auto-renew-off",'
  + '"retry_strategy":{"type":"concurrent_drain","suggested_initial_delay_s":1,"max_delay_s":30,'
  + '"backoff":"exponential","backoff_base":2,"jitter":true},"retryable":true,"context":{"budget":6,'
  + '"in_flight":6,"model":"deepseek-ai/DeepSeek-V4.1-Flash","limit_type":"concurrent",'
  + '"tpm_window_tokens":0,"tpm_limit":5242880,"floor":5,"burst":6,"burst_shed":false}}';

/**
 Provider status used for capacity it cannot serve now.
 */
const UNAVAILABLE_STATUS = 503;

/**
 A 503 refusal, in the Completions adapter's text form.
 */
const UNAVAILABLE_REFUSAL = `${UNAVAILABLE_STATUS}: {"error":{"message":"Service temporarily unavailable","type":"server_error"}}`;

/**
 Cloudflare's status for an origin it cannot reach.
 */
const UNREACHABLE_STATUS = 523;

/**
 Cloudflare 523, as that error page heads it.

 Source, not a capture: "Error 523: origin is unreachable"
 (https://developers.cloudflare.com/support/troubleshooting/http-status-codes/cloudflare-5xx-errors/error-523/,
 read 2026-11-19).
 */
const UNREACHABLE_REFUSAL = `${UNREACHABLE_STATUS}: Error ${UNREACHABLE_STATUS}: origin is unreachable`;

//endregion Refusal texts

//region Stand-in members

/**
 Provider id the pool asks, whose real extension this stand-in replaces for the check.
 */
const MEMBER_PROVIDER = 'neuralwatt';

/**
 Head member of the writer pool, the one that refuses or overflows.
 */
const HEAD_MEMBER = 'deepseek-v4.1-flash';

/**
 Second writer member, which answers whatever the head does.
 */
const SECOND_MEMBER = 'deepseek-v4.1-flash-flex';

/**
 Window both members declare: large enough that Pi's threshold compaction leaves the padded turns
 alone, so only the member's own refusal can trigger a recovery.
 */
const DECLARED_CONTEXT_WINDOW = 1_000_000;

/**
 Output limit both members declare.
 */
const DECLARED_MAX_TOKENS = 16_384;

/**
 Characters the head member accepts before it refuses: smaller than what it declares, the way a real
 member reserves room for its output.
 */
export const ENFORCED_CHARS = 140_000;

/**
 Stand-in origin, which is not a provider's quota endpoint, so a refusal's quota read stays home.
 */
const STAND_IN_ORIGIN = 'http://127.0.0.1:0/v1';

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
 Model fields a stand-in message reports, project-owned so a catalog record is assignable.
 */
type MemberIdentity = {
  readonly provider: string;
  readonly id: string;
  readonly api: string
};

/**
 Transcript size source: the member counts characters and reads nothing else.
 */
type TranscriptView = { readonly messages: unknown; };

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

 @param errorMessage - refusal text

 @returns stream whose only content is the refusal

 @example
 ```ts
 refusalStream({ model, errorMessage: UNREACHABLE_REFUSAL });
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
function requestChars({ context }: { readonly context: TranscriptView; },): number {
  return JSON.stringify(context.messages,)
    .length;
}

//endregion Stand-in members

//region Provider

/**
 Read the behaviour the session selected, refusing a name this module does not play.

 @returns behaviour every member plays

 @throws Error when the session named no behaviour or an unknown one

 @example
 ```ts
 const behaviour = selectedBehaviour();
 ```
 */
function selectedBehaviour(): string {
  /**
   Behaviour the session's environment names.
   */
  const named = process.env[BEHAVIOUR_VARIABLE];
  if ((named === undefined) || (!BEHAVIOURS.has(named,)))
    throw new Error(`${BEHAVIOUR_VARIABLE} must name one of: ${[...BEHAVIOURS,].join(', ',)}`,);
  return named;
}

/**
 Catalog entry for one stand-in member.

 @param id - member id the pool asks for

 @returns chat model the pool resolves through Pi's registry

 @example
 ```ts
 standInModel(SECOND_MEMBER);
 ```
 */
function standInModel(id: string,): Model<Api> {
  return {
    id,
    name: `Stand-in member: ${id}`,
    api: 'openai-completions',
    provider: MEMBER_PROVIDER,
    baseUrl: STAND_IN_ORIGIN,
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
 Refusal the selected behaviour gives the head member, or none when it answers.

 @param behaviour - behaviour every member plays

 @returns refusal text, or empty text when the head member answers

 @example
 ```ts
 headRefusal(REFUSING_523);
 ```
 */
function headRefusal(behaviour: string,): string {
  if (behaviour === REFUSING_429)
    return OVERAGE_REFUSAL;
  if (behaviour === REFUSING_503)
    return UNAVAILABLE_REFUSAL;
  if (behaviour === REFUSING_523)
    return UNREACHABLE_REFUSAL;
  return '';
}

/**
 The body an OpenAI-compatible member returns when the input exceeds its window.

 Pi's own overflow detector lists this shape with example values
 (`@earendil-works/pi-ai` dist/utils/overflow.js, its overflow-pattern documentation), and the counts
 here are the request's own, in the installed Completions adapter's `<status>: <body>` shape.

 @param inputChars - characters the member received

 @param limitChars - characters it accepts

 @returns refusal text Pi classifies as a context overflow
 */
function overflowRefusal({
  inputChars,
  limitChars,
}: {
  readonly inputChars: number;
  readonly limitChars: number;
},): string {
  return `400: {"error":{"message":"Input length (${inputChars}) exceeds model's maximum context length (${limitChars}).","type":"invalid_request_error","code":"context_length_exceeded"}}`;
}

/**
 Decide what one stand-in member does with a request.

 @param model - member the pool resolved

 @param context - transcript the pool forwarded

 @param behaviour - behaviour every member plays

 @returns the member's stream

 @example
 ```ts
 play({ model: standInModel(HEAD_MEMBER), context: { messages: [] }, behaviour: OVERFLOWING });
 ```
 */
function play({
  model,
  context,
  behaviour,
}: {
  readonly model: MemberIdentity;
  readonly context: TranscriptView;
  readonly behaviour: string;
},): AssistantMessageEventStream {
  /**
   Characters this request carries, the stand-in's only measure of size.
   */
  const chars = requestChars({ context, },);
  /**
   Whether this member is the pool's head, which refuses under a refusal behaviour.
   */
  const isHead = model.id === HEAD_MEMBER;
  /**
   Refusal the selected behaviour gives this member, or empty text when it answers.
   */
  const refusal = isHead ? headRefusal(behaviour,) : '';
  if (refusal !== '') {
    return refusalStream({
      model,
      errorMessage: refusal,
    },);
  }
  if ((behaviour === OVERFLOWING)
    && isHead
    && (chars > ENFORCED_CHARS)) {
    /**
     Body the member returns instead of an answer.
     */
    const overgrown = overflowRefusal({
      inputChars: chars,
      limitChars: ENFORCED_CHARS,
    },);
    return refusalStream({
      model,
      errorMessage: overgrown,
    },);
  }
  return answerStream({
    model,
    text: `Stand-in ${model.id} answered a ${chars}-character request.`,
  },);
}

/**
 Refuse the API-specific entry point, which a pool never uses.

 @throws Error always, because the pool dispatches through `streamSimple`
 */
function streamUnsupported(): never {
  throw new Error('The stand-in members answer through streamSimple only.',);
}

/**
 Capabilities this stand-in asks Pi for, and nothing else.
 */
type StandInHost = {
  registerProvider: (provider: Provider) => void;
  on: (
    event: 'session_start',
    handler: (
      event: unknown,
      context: StandInContext,
    ) => void,
  ) => unknown;
};

/**
 Selection Pi holds when a session starts, read-only.
 */
type StandInContext = {
  readonly model?: {
    readonly provider: string;
    readonly id: string
  }
};

/**
 Extension that registers the writer pool's two stand-in members.

 @param pi - Pi's extension API for this session

 @example
 ```ts
 pi -e src/verify-stand-in-member-extension.ts --model rate-limit-fallback/writer 'ready'
 ```
 */
export default function standInMemberExtension(pi: StandInHost,): void {
  /**
   Behaviour this session plays, read once so every member agrees.
   */
  const behaviour = selectedBehaviour();
  // The selection is not part of the JSON event stream, so the check reads it from here: whatever
  // Pi holds at session start is what `--model` resolved to.
  pi.on(
    'session_start',
    function noteSelection(
      _event: unknown,
      context: StandInContext,
    ): void {
      /**
       Selected model Pi holds, or nothing before one is chosen.
       */
      const selected = context.model;
      /**
       Provider Pi reports for the selection, empty before one exists.
       */
      const provider = selected?.provider ?? '';
      /**
       Model id Pi reports for the selection, empty before one exists.
       */
      const id = selected?.id ?? '';
      process.stderr
        .write(`[stand-in-selection] ${provider}/${id}\n`,);
    },
  );
  /**
   Members the pool asks, in its order.
   */
  const members = [
    HEAD_MEMBER,
    SECOND_MEMBER,
  ].map(function model(id,) {
    return standInModel(id,);
  },);
  /**
   Provider object registered with the host.
   */
  const provider: Provider = {
    id: MEMBER_PROVIDER,
    name: 'Stand-in writer members',
    auth: {
      apiKey: {
        name: 'Stand-in members',
        resolve() {
          return Promise.resolve({
            auth: {},
            source: MEMBER_PROVIDER,
          },);
        },
      },
    },
    getModels() {
      return members;
    },
    stream: streamUnsupported,
    streamSimple(
      model: MemberIdentity,
      context: TranscriptContext,
      _options?: SimpleStreamOptions,
    ): AssistantMessageEventStream {
      return play({
        model,
        context,
        behaviour,
      },);
    },
  };
  pi.registerProvider(provider,);
}
