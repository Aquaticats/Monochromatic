import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  armCallDeadline,
  type Decider,
  type DecisionReply,
  type DecisionReplyShapeError,
  type DecisionRequest,
  DEFAULT_RETRY_POLICY,
  exchangeWithRetry,
  OPENROUTER_AUTH_HEADER,
  readDecisionReplyBody,
  reportSpend,
  type RetryPolicy,
} from '@monochromatic-dev/translation-repair-provider/ts';
import {
  type ModelTransport,
  ProviderHttpError,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  type DecisionsCardMissingError,
  decisionsCardOf,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Decisions client
// THE FIFTH TRANSPORT, and the first that is not a chat completion:
// OpenRouter's decisions router, which fronts TypeSafe's System One models.
// One POST, one JSON body back, no stream, no completion cap (the answer
// is a typed value) and no thinking parameter (there is nothing to think
// in). Raw fetch through the same transport seam as every other client, on
// the owner's rule of 2026-09-07 (no vendor SDKs). Spend is reported on the
// OpenRouter meter, which is the credit balance this endpoint draws on.

/**
 Where OpenRouter answers a typed-decision request.
 */
export const OPENROUTER_DECISIONS_URL = 'https://openrouter.ai/api/alpha/decisions';

/**
 Lowest status the endpoint answers a decided request with.
 */
const HTTP_SUCCESS_FLOOR = 200;

/**
 First status past the success range.
 */
const HTTP_SUCCESS_CEILING = 300;

/**
 Logger root for this client.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Builds the client of the decisions endpoint.

 @param apiKey - OpenRouter key, the same one the chat client carries

 @param transport - HTTP the client calls over: `fetchTransport` in a run, a
 recorded one in a test. REQUIRED, since a test that left it out would reach
 the provider (ledger M43, X24)

 @param url - endpoint, overridable for a recorded test

 @param retryPolicy - transient retry pacing

 @returns Client a router or a stage asks a typed question through

 @example
 ```ts
 const client = createDecisionsClient({ apiKey: process.env['TRANSLATION_REPAIR_OPENROUTER_API_KEY'] ?? '', transport: fetchTransport, },);
 ```
 */
export function createDecisionsClient(
  {
    apiKey,
    transport,
    url = OPENROUTER_DECISIONS_URL,
    retryPolicy = DEFAULT_RETRY_POLICY,
  }: {
    readonly apiKey: string;
    readonly transport: ModelTransport;
    readonly url?: string;
    readonly retryPolicy?: RetryPolicy;
  },
): Decider {
  /**
   Headers shared by every exchange, auth included.
   */
  const headers: Readonly<Record<string, string>> = {
    [OPENROUTER_AUTH_HEADER]: `Bearer ${apiKey}`,
    'content-type': 'application/json',
  };

  /**
   Typed-decision exchange.

   @param request - state and questions to decide

   @mutates request - `JSON.stringify` may invoke toJSON methods or getters while serializing the state

   @returns Answers, model and usage as the endpoint reported them

   @throws {@link DecisionsCardMissingError} when the seat is not a decision seat

   @throws {@link ProviderHttpError} on a non-success status

   @throws {@link DecisionReplyShapeError} on a body that is not the documented shape

   @example
   ```ts
   const reply = await client.decide({ modelId, state, questions, signal, },);
   ```
   */
  async function decide(request: ForeignBorrowed<DecisionRequest>,): Promise<DecisionReply> {
    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: decide.name,
      l,
    },);

    /**
     Wire spelling, resolved first so a chat seat addressed here fails at
     once.
     */
    const servedId = decisionsCardOf({ modelId: request.modelId, },)
      .id;

    /**
     Per-exchange deadline, absent when the caller set none.
     */
    using deadline = (request.exchangeTimeoutMs === undefined)
      ? undefined
      : armCallDeadline({
        signal: request.signal,
        timeoutMs: request.exchangeTimeoutMs,
        label: servedId,
      },);

    /**
     Signal the exchange honors: deadline-joined when armed.
     */
    const exchangeSignal = (deadline === undefined)
      ? request.signal
      : deadline.callSignal;

    /**
     Exactly what goes on the wire.
     */
    const bodyJson = JSON.stringify({
      model: servedId,
      state: request.state,
      questions: request.questions,
    },);

    /**
     Questions asked, for the entry log line.
     */
    const questionCount = Object.keys(request.questions,)
      .length;
    rl.debug(`-> ${servedId}: ${String(questionCount,)} ${
      wordForCount({
        count: questionCount,
        one: 'question',
        many: 'questions',
      },)
    }`,);

    /**
     Raw reply, retried on transient statuses.
     */
    const reply = await exchangeWithRetry({
      provider: 'openrouter',
      transport,
      exchange: {
        url,
        label: servedId,
        method: 'POST',
        headers,
        bodyJson,
        signal: exchangeSignal,
      },
      policy: retryPolicy,
    },);
    if ((reply.status < HTTP_SUCCESS_FLOOR) || (reply.status >= HTTP_SUCCESS_CEILING)) {
      throw new ProviderHttpError({
        provider: 'openrouter',
        status: reply.status,
        bodyText: reply.bodyText,
      },);
    }

    /**
     Reply as the contract types it.
     */
    const read = readDecisionReplyBody({ bodyText: reply.bodyText, },);

    /**
     Answers read, for the exit log line.
     */
    const answerCount = Object.keys(read.answers,)
      .length;
    rl.debug(`<- ${servedId}: ${String(answerCount,)} ${
      wordForCount({
        count: answerCount,
        one: 'answer',
        many: 'answers',
      },)
    }`,);
    reportSpend({
      provider: 'openrouter',
      label: servedId,
      extracted: {
        text: '',
        // Conditional spread keeps the block absent where the wire sent none.
        ...((read.usage === undefined) ? {} : { usage: read.usage, }),
      },
      ...((read.costUsd === undefined) ? {} : { costUsd: read.costUsd, }),
    },);
    return read;
  }

  return { decide, };
}

//endregion Decisions client
