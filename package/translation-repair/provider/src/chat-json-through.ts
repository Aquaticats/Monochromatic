import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelCaller,
} from './chat-contract.ts';
import {
  readJsonOutcome,
  responseFormatSpreadOf,
} from './chat-json-outcome.ts';

//region Chat json through
// The `chatJson` every provider client exposes, built over that client's own
// `chatText`. Each client (Bedrock, Hyper, OpenRouter, Synthetic) kept its own
// copy (audit area six, 2026-09-28), and each copy forwarded four named
// request fields by hand, so a field added to the text request (`otherThan`
// was one) reached no client's JSON exchange. This forwards the whole request
// but its validator.

/**
 Builds a client's JSON exchange: one text exchange carrying every field of
 the request but its validator, read as JSON against that validator.

 @param chatText - the client's own text exchange

 @returns The client's `chatJson`

 @example
 ```ts
 const chatJson = chatJsonThrough({ chatText, },);
 const outcome = await chatJson({ modelId, messages, signal, validate: isVerdict, },);
 ```
 */
export function chatJsonThrough(
  { chatText, }: { readonly chatText: ModelCaller['chatText']; },
): ModelCaller['chatJson'] {
  /**
   One JSON exchange: the text exchange, then the reply read as JSON.

   @param request - text request plus the guard admitting parsed content

   @returns Outcome as data: ok, refusal-shaped, or schema-mismatch

   @example
   ```ts
   const outcome = await chatJson({ modelId, messages, signal, validate: isVerdict, },);
   ```
   */
  return async function chatJson<ValueT,>(
    request: ForeignBorrowed<ChatJsonRequest<ValueT>>,
  ): Promise<ChatJsonOutcome<ValueT>> {
    /**
     Guard for the parsed answer, and the text request it rides on.
     */
    const {
      validate,
      ...textRequest
    } = request;

    /**
     Raw text reply of the underlying exchange.
     */
    const reply = await chatText(textRequest,);
    return readJsonOutcome({
      modelId: request.modelId,
      reply,
      validate,
      ...responseFormatSpreadOf({ request, },),
    },);
  };
}

//endregion Chat json through
