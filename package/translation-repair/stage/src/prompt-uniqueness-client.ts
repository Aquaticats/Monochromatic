import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  type ChatTextReply,
  type ChatTextRequest,
  readJsonOutcome,
  reaskElsewhereNudged,
  responseFormatSpreadOf,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { MalformedCompletionError, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  PROMPT_PAYLOAD_MISSING,
  type PromptPayloadStore,
  PromptPayloadStoreError,
} from './prompt-payload-store.ts';

//region Model-prompt uniqueness boundary

/**
 Logger root for privacy-safe payload reuse telemetry.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 What a prompt digest is computed over: the model id and the ordered messages,
 each a role and either a string or a list of content parts, every part a
 record of strings. No number, boolean, null or absent value reaches it, so
 the serializer has no arm for one.
 */
type PromptValue =
  | string
    | readonly PromptValue[]
    | { readonly [key: string]: PromptValue; };

/**
 Narrows a prompt value to a list, keeping its items typed as prompt values
 where `Array.isArray` alone would widen them.

 @param value - prompt value of either compound shape, or a string

 @returns Whether value is a list

 @example
 ```ts
 const isList = isPromptList(value,);
 ```
 */
function isPromptList(value: PromptValue,): value is readonly PromptValue[] {
  return Array.isArray(value,);
}

/**
 Serializes prompt value with stable object-key order.

 Arrays preserve semantic order while object construction order does not affect identity.

 KEYS SORT BY UTF-16 CODE UNIT, as canonical JSON (RFC 8785) sorts them, and as
 the default `toSorted` compares strings. `localeCompare` follows the host's
 locale, and this digest names a durable payload record that a run on another
 host may read. The one code-unit ordering the code-unit scan
 (`locale-orderings.unit.test.ts`, ledger B122) exempts, since code-point
 order would be another canonical form.

 @param value - prompt value composed from protocol strings

 @returns Stable structural serialization

 @example
 ```ts
 const serialized = canonicalPromptValue({ role: 'user', content: 'Hello', },);
 ```
 */
function canonicalPromptValue(value: PromptValue,): string {
  if ((typeof value) === 'string')
    return JSON.stringify(value,);
  if (isPromptList(value,)) {
    /**
     Canonically serialized array items in semantic order.
     */
    const items = value.map(function serializeItem(item,): string {
      return canonicalPromptValue(item,);
    },);
    return `[${items.join(',',)}]`;
  }
  /**
   Record entries serialized in code-unit order of their keys.
   */
  const entries = Object.keys(value,)
    .toSorted()
    .map(function serializeEntry(key,): string {
      /**
       Value under a key this record was just asked for.
       */
      const entryValue = nonNullishOrThrow(value[key],);
      return `${JSON.stringify(key,)}:${canonicalPromptValue(entryValue,)}`;
    },);
  return `{${entries.join(',',)}}`;
}

/**
 Canonical model and ordered-message identity shared by text and JSON calls.

 Request metadata is deliberately excluded.
 Changing response schema,
 timeout,
 or output cap does not turn same substantive conversation into independent evidence.

 @param request - model request whose exact message bytes form prompt

 @returns Privacy-safe digest used only for duplicate accounting

 @example
 ```ts
 const digest = modelPromptDigest({ request, });
 ```
 */
export function modelPromptDigest(
  { request, }: ForeignBorrowed<{ readonly request: ChatTextRequest; }>,
): string {
  /**
   Ordered messages reduced to destination-relevant role and content.
   */
  const messages = request.messages
    .map(function canonicalMessage(message,) {
      return {
        role: message.role,
        content: message.content,
      };
    },);
  return hashContent({
    content: canonicalPromptValue({
      modelId: request.modelId,
      messages,
    },),
  },);
}

/**
 Prevents one model and one completed prompt from being sampled twice.

 Concurrent and completed duplicates reuse first payload before second provider call.
 Provider-level delivery retries remain inside wrapped call;
 when wrapped call throws without outcome,
 identity is released so operational recovery may retry it.
 Any returned outcome claims identity permanently for client lifetime,
 including schema mismatch or refusal.

 @param inner - routed provider client performing first unique call

 @param store - optional durable raw-payload checkpoint across invocations

 @returns Client enforcing model-prompt uniqueness by reuse

 @example
 ```ts
 const client = promptUniqueClient({ inner, });
 ```
 */
export function promptUniqueClient(
  {
    inner,
    store,
  }: ForeignBorrowed<{
    readonly inner: ModelClient;
    readonly store?: PromptPayloadStore;
  }>,
): ModelClient {
  /**
   Prompt identities mapped to first in-flight or completed provider payload.
   */
  const claimed = new Map<string, Promise<ChatTextReply>>();

  /**
   One exchange under the uniqueness rule: a claimed payload reused, a stored
   one replayed, or the first provider call bought and stored.

   @param request - exchange to perform

   @returns Payload for this model and prompt

   @example
   ```ts
   const reply = await claimedReply(request,);
   ```
   */
  async function claimedReply(request: ChatTextRequest,): Promise<ChatTextReply> {
    /**
     Canonical model and prompt identity.
     */
    const promptDigest = modelPromptDigest({ request, },);
    /**
     Earlier in-flight or completed payload for same identity.
     */
    const existing = claimed.get(promptDigest,);
    if (existing !== undefined) {
      /**
       Reused payload after owner call completed successfully.
       */
      const reply = await existing;
      l.info(`PROMPT-REUSE source=memory model=${request.modelId} digest=${promptDigest}`,);
      return reply;
    }
    /**
     Durable payload replay or first provider exchange,
     claimed synchronously before any await.
     */
    const pending = (async function buyOrResume(): Promise<ChatTextReply> {
      /**
       Durable payload or explicit absence when store is configured.
       */
      const stored = await store?.read({ promptDigest, },)
        ?? PROMPT_PAYLOAD_MISSING;
      if ((typeof stored) !== 'symbol') {
        l.info(`PROMPT-REUSE source=disk model=${request.modelId} digest=${promptDigest}`,);
        return stored;
      }
      /**
       First provider payload for this prompt identity.
       */
      const reply = await inner.chatText(request,);
      await store?.write({
        promptDigest,
        reply,
      },);
      return reply;
    })();
    claimed.set(
      promptDigest,
      pending,
    );
    try {
      return await pending;
    }
    catch (error) {
      /**
       Whether provider completed payload or durable store failed after claim.
       */
      const retainsClaim = (error instanceof MalformedCompletionError)
        || (error instanceof PromptPayloadStoreError);
      if (!retainsClaim)
        claimed.delete(promptDigest,);
      throw error;
    }
  }

  return {
    chatText: claimedReply,
    chatJson: async function uniqueJson<ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> {
      /**
       Payload for this model and prompt, reused, replayed or bought.
       */
      const reply = await claimedReply(request,);

      // A REPLY THAT COULD NOT BE USED IS RE-ASKED ELSEWHERE, NUDGED, through
      // this same claim path (ledger P9), so the re-ask is claimed and stored
      // like the first ask and a resumed run replays both.
      return await reaskElsewhereNudged({
        request,
        first: {
          reply,
          outcome: readJsonOutcome({
            modelId: request.modelId,
            reply,
            validate: request.validate,
            ...responseFormatSpreadOf({ request, },),
          },),
        },
        ask: claimedReply,
      },);
    },
    quotas: inner.quotas,
    // A TYPED EXCHANGE IS NEITHER CLAIMED NOR STORED: its state is the
    // chat sheet's evidence restated, and the endpoint answers in under a
    // second for a fraction of a cent, so there is nothing to reuse.
    ...((inner.decide === undefined) ? {} : { decide: inner.decide, }),
  };
}

//endregion Model-prompt uniqueness boundary
