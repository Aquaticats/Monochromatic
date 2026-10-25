import type {
  ChatMessage,
  CompletionUsage,
  ContentPart,
} from '@monochromatic-dev/module-llm-type/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { Decider, } from './decision-contract.ts';
import type {
  ProviderName,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { QuotaSnapshot, } from '@monochromatic-dev/translation-repair-provider-stream/ts';

//region Chat contract
// Request and outcome shapes of the model-facing client. Outcomes are data:
// refusals and schema mismatches are ordinary results of calling unreliable models,
// never exceptions; only provider protocol failures throw.
// Sampling knobs are absent by design: the serving stack (upstream GPU
// providers, inference pipelines, and the models themselves) does not honor
// temperature or reasoning effort reliably (errors, degraded or truncated
// output), so every call runs on defaults. Do not add a temperature or
// effort field.

/**
 OpenAI-style structured-output constraint;
 every catalog model advertises `structured_outputs`,
 but client-side validation stays because per-model strictness is unverified.

 @example
 ```ts
 const format: JsonSchemaResponseFormat = {
   type: 'json_schema',
   json_schema: { name: 'issue_claims', schema: { type: 'object', }, },
 };
 ```
 */
export type JsonSchemaResponseFormat = {
  /**
   Discriminator the OpenAI-compatible API expects.
   */
  readonly type: 'json_schema';

  /**
   Schema envelope: name, optional strictness, JSON schema body.
   */
  readonly json_schema: {
    /**
     Identifier the API requires per schema.
     */
    readonly name: string;

    /**
     Whether the server should enforce the schema strictly.
     */
    readonly strict?: boolean;

    /**
     JSON schema constraining the completion.
     */
    readonly schema: Record<string, unknown>;
  };
};

/**
 One message carrying parts rather than a plain string.

 FOR IMAGES, and only images so far. A picture is sent so a transcribed
 passage has a source that can be CHECKED rather than only preserved, and the
 provider takes that as an OpenAI-compatible content-part array. The part type
 is the shared one from `@monochromatic-dev/module-llm-type`, so nothing here
 invents a protocol.

 @example
 ```ts
 const message: VisionMessage = {
   role: 'user',
   content: [
     { type: 'text', text: 'Read this picture.', },
     { type: 'image_url', image_url: { url: dataUri, }, },
   ],
 };
 ```
 */
export type VisionMessage = {
  /**
   Author of the message, the same roles a text message uses.
   */
  readonly role: ChatMessage['role'];

  /**
   Parts this message is composed of, in the order the model reads them.
   */
  readonly content: readonly ContentPart[];
};

/**
 Text of a message, whichever shape it carries.

 ONE READER RATHER THAN A CAST AT EVERY SITE. Widening `content` to carry
 picture parts left several readers expecting a string, and a cast at each of
 them would be five places to get wrong rather than one. A picture is not
 recoverable as text and is named by its part type instead, which is what a
 witness or a test assertion wants: what the model was ASKED, not the bytes it
 was shown.

 @param message - message to read

 @returns Its text, with any non-text part named

 @example
 ```ts
 const asked = messageText({ message, },);
 ```
 */
export function messageText({ message, }: { readonly message: ChatMessage | VisionMessage; },): string {
  /**
   Content as the message carries it.
   */
  const { content, } = message;
  if ((typeof content) === 'string')
    return content;

  /**
   Text of each part, with a picture named rather than rendered.
   */
  const parts = content.map(function partText(part,): string {
    return (part.type === 'text') ? part.text : `[${part.type}]`;
  },);
  return parts.join('\n',);
}

/**
 Whether a conversation shows anything that only a vision stack can read.

 ASKED BECAUSE REACH IS NARROWER FOR PICTURES THAN FOR TEXT, and not by the
 same models on both providers. `roster-reach.ts` records the case: one model
 reads pictures on one provider and not on the other, same weights, different
 serving stack. A router that asked one question for both would either send a
 picture where it cannot be read or refuse one that can.

 ANY NON-TEXT PART COUNTS, rather than the image part alone. A part this
 pipeline has not met yet is still something a text-only stack cannot take,
 and guessing the other way is the mistake that costs a call.

 @param messages - conversation as the caller built it

 @returns Whether any message carries a part that is not text

 @example
 ```ts
 const needsVision = carriesPicture({ messages, },);
 ```
 */
export function carriesPicture(
  { messages, }: { readonly messages: readonly (ChatMessage | VisionMessage)[]; },
): boolean {
  return messages.some(function showsOne(message,): boolean {
    /**
     Content as this message carries it, string or parts.
     */
    const { content, } = message;

    if ((typeof content) === 'string')
      return false;
    return content.some(function isPicture(part,): boolean {
      return part.type !== 'text';
    },);
  },);
}

/**
 One chat exchange request.

 @example
 ```ts
 const request: ChatTextRequest = {
   modelId: 'hf:zai-org/GLM-5.3-Flash',
   messages: [{ role: 'user', content: '喵？', },],
   signal: AbortSignal.timeout(120_000,),
 };
 ```
 */
export type ChatTextRequest = {
  /**
   Catalog model receiving the exchange.
   */
  readonly modelId: RosterModelId;

  /**
   Conversation sent as-is.

   TEXT OR VISION, in one field rather than in two request shapes. The body is
   a pass-through, `messages: request.messages` inside a `JSON.stringify`, and
   the provider is OpenAI-compatible, so a message whose content is an array of
   parts serialises correctly with no other change. Widening the union leaves
   every existing caller valid, where a second request type and a second client
   method would have duplicated the limiter, the deadline, the retry ladder and
   the drain for one field's sake.
   */
  readonly messages: readonly (ChatMessage | VisionMessage)[];

  /**
   Abort signal honored for the whole exchange, wait included.
   */
  readonly signal: AbortSignal;

  /**
   Deadline bounding the exchange itself, armed only after the
   per-model slot is acquired, so local queue wait behind concurrent
   same-model calls never counts against it.
   Fan-outs need this: a dispatch-time deadline starves every queued
   call simultaneously.
   */
  readonly exchangeTimeoutMs?: number;

  /**
   Characters this call may produce on the answer channel before the
   stream watch ends it.
   Set by a caller that knows how large its own input was, so a bound
   relative to that input can end a runaway the absolute default is far
   too loose to see.
   */
  readonly maxAnswerChars?: number;

  /**
   Completion token cap when the caller bounds output.
   Thinking tokens count against it and dominate output on these models
   (expect 90%+), so set it generously or omit it;
   a tight cap truncates mid-thinking and destroys the answer.
   */
  readonly maxTokens?: number;

  /**
   Structured-output constraint when the caller expects JSON.
   */
  readonly responseFormat?: JsonSchemaResponseFormat;

  /**
   Provider this exchange must not be served by, set only on the nudged
   re-ask of a reply that could not be used (ledger P9). The router serves it
   on another provider or throws `NoProviderForModelError`; a client that
   routes nowhere never tags a reply, so it is never asked to honour one.
   */
  readonly otherThan?: ProviderName;

  /**
   Names the repeat this exchange belongs to when a caller asks one prompt again on purpose, as a probe that measures
   whether a model answers alike on a second pass does.
   The prompt reuse layer (`promptUniqueClient`) keeps the scope in the identity it claims a prompt under, so the same
   prompt under another scope is a new exchange, bought from the model, and under the same scope it is reused as ever.
   Absent for every caller that wants the reuse, so the identity of its prompts does not change.
   No client puts it on the wire.
   */
  readonly reuseScope?: string;
};

/**
 Raw text outcome of one chat exchange.

 @example
 ```ts
 const reply: ChatTextReply = { text: '喵。', };
 ```
 */
export type ChatTextReply = {
  /**
   Verbatim content of the first choice;
   empty when the API refused and returned no content.
   */
  readonly text: string;

  /**
   First-class refusal from the message `refusal` field.
   */
  readonly refusal?: string;

  /**
   Token usage when the server reported it;
   completion counts include thinking tokens.
   */
  readonly usage?: CompletionUsage;

  /**
   Why the model stopped, verbatim from the provider, when it said.

   CARRIED SO A CALLER CAN TELL A CUT-OFF REPLY FROM A MALFORMED ONE. Those
   arrive identically, as content that will not parse, and they need opposite
   remediation: one points at the token ceiling, the other at the prompt and
   the guard.
   */
  readonly finishReason?: string;

  /**
   Provider that served the reply, set by the router and replayed by the
   payload store, so a reply that could not be used can be re-asked
   elsewhere (ledger P9). Absent from a client that routes nowhere and from
   payloads stored before 2026-09-28, which are then not re-asked.
   */
  readonly servedBy?: ProviderName;
};

/**
 One schema-validated chat request:
 text request plus the guard that admits parsed content.

 @example
 ```ts
 const request: ChatJsonRequest<Verdict> = { ...textRequest, validate: isVerdict, };
 ```
 */
export type ChatJsonRequest<ValueT,> = ChatTextRequest & {
  /**
   Guard admitting parsed model JSON into the typed pipeline.
   */
  readonly validate: (value: unknown,) => value is ValueT;
};

/**
 Machine-readable cause of a schema mismatch, beside the prose detail.
 `truncated-completion` marks the provider's own token-ceiling stop,
 which invalidates even parseable content because a syntactically complete
 prefix does not prove generation completed.

 @example
 ```ts
 const reason: SchemaMismatchReason = 'truncated-completion';
 ```
 */
export type SchemaMismatchReason =
  | 'caller-guard-rejected'
    | 'unparseable-json'
    | 'truncated-thinking'
    | 'truncated-completion';

/**
 How the nudged re-ask on another provider failed too, kept beside the first answer's own failure (the lead's ruling of
 2026-10-15 on the run of TianqiChen66625, where the log of a lost voice named only the first answer's failure and a
 reader could not tell a label written in a form no guard accepted from a second refusal).

 @example
 ```ts
 const failure: ChatJsonReaskFailure = { servedBy: 'hyper', kind: 'schema-mismatch', rawText: '{}', cause: 'content is not valid JSON', };
 ```
 */
export type ChatJsonReaskFailure = {
  /**
   Provider that served the re-ask, or the words `another provider` where it did not say.
   */
  readonly servedBy: string;

  /**
   Which kind of failure the re-ask's own reply was.
   */
  readonly kind: 'refusal-shaped' | 'schema-mismatch';

  /**
   Verbatim model text of the re-ask's reply.
   */
  readonly rawText: string;

  /**
   What failed in it: the refusal marker, or the mismatch's detail.
   */
  readonly cause: string;
};

/**
 Outcome of one schema-validated chat exchange.
 Refusals and mismatches are data (reroute and scorecard), never exceptions.

 @example
 ```ts
 const outcome: ChatJsonOutcome<Verdict> = { kind: 'ok', value, rawText, };
 ```
 */
export type ChatJsonOutcome<ValueT,> =
  | {
    /**
     Content parsed and validated.
     */
    readonly kind: 'ok';

    /**
     Typed content admitted by the caller's guard.
     */
    readonly value: ValueT;

    /**
     Verbatim model text for audit trails.
     */
    readonly rawText: string;

    /**
     Token usage when reported; thinking tokens included.
     */
    readonly usage?: CompletionUsage;
  }
    | {
    /**
     The API refused outright or the answer reads as a refusal;
     reroute cross-family.
     */
      readonly kind: 'refusal-shaped';

    /**
     Verbatim model text for audit trails.
     */
      readonly rawText: string;

    /**
     Refusal marker that fired;
     `api-refusal-field` when the message `refusal` field was set.
     Feeds the scorecard.
     */
      readonly marker: string;

    /**
     Token usage when reported; thinking tokens included.
     */
      readonly usage?: CompletionUsage;

    /**
     How the nudged re-ask on another provider failed too, when one was made and failed.
     */
      readonly reaskFailure?: ChatJsonReaskFailure;
    }
    | {
    /**
     Content is not valid JSON, failed the caller's guard,
     or was truncated by the provider (completion or thinking).
     */
      readonly kind: 'schema-mismatch';

    /**
     Verbatim model text for audit trails.
     */
      readonly rawText: string;

    /**
     Machine-readable cause, for callers that branch on truncation.
     */
      readonly reason?: SchemaMismatchReason;

    /**
     What failed: parse step, guard, or completion truncation.
     */
      readonly detail: string;

    /**
     Token usage when reported; thinking tokens included.
     */
      readonly usage?: CompletionUsage;

    /**
     How the nudged re-ask on another provider failed too, when one was made and failed.
     */
      readonly reaskFailure?: ChatJsonReaskFailure;
    };

/**
 What every provider's client can do, whichever protocol it speaks.

 THE PART THAT IS NOT ABOUT BUDGET. Both providers take a conversation and
 return an answer; what they do NOT share is how the money is counted, which
 is a weekly quota (with a five-hour window where a reply still carries
 one) on one and a credit balance on the other.
 Splitting the surface here lets a router hold two clients as one type and
 lets a stage take either without knowing which it has.

 @example
 ```ts
 async function ask({ caller, }: { readonly caller: ModelCaller; },): Promise<void> {}
 ```
 */
export type ModelCaller = {
  /**
   Free-text chat exchange.
   */
  readonly chatText: (request: ForeignBorrowed<ChatTextRequest>,) => Promise<ChatTextReply>;

  /**
   Schema-validated chat exchange returning outcomes as data.
   */
  readonly chatJson: <ValueT,>(
    request: ForeignBorrowed<ChatJsonRequest<ValueT>>,
  ) => Promise<ChatJsonOutcome<ValueT>>;

  /**
   Typed-decision exchange for a decision seat, present on a client that
   carries the decisions transport (`decisions-client.ts`, 2026-09-18) and
   absent on one that does not, so a stage asking a decision seat through a
   client without it records a lost voice rather than a chat call.
   */
  readonly decide?: Decider['decide'];
};

/**
 The client surface every driver consumes, whichever provider or router serves it.

 A provider's own client (`createSyntheticClient` builds the Synthetic one)
 and the routed client that spans every provider both fit this type,
 so a driver takes `ModelClient` and never names the provider behind it.

 @example
 ```ts
 async function drive({ client, }: { readonly client: ModelClient; },): Promise<void> {}
 ```
 */
export type ModelClient = ModelCaller & {
  /**
   Current quota snapshot; free per provider docs.
   */
  readonly quotas: (args: { readonly signal: AbortSignal; },) => Promise<QuotaSnapshot>;
};

//endregion Chat contract
