import type { ChatMessage, } from '@monochromatic-dev/module-llm-type/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ChatTextReply,
  ChatTextRequest,
} from './chat-contract.ts';
import {
  readJsonOutcome,
  responseFormatSpreadOf,
} from './chat-json-outcome.ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Nudged re-ask elsewhere
// LEDGER P9, the owner's ruling of 2026-09-28 ("Enable with the nudge"). A
// reply that could not be used is asked again, once, of the SAME MODEL ON
// ANOTHER PROVIDER, since the providers extract structure by different
// mechanisms (a forced tool on one, a `response_format` on the others) and the
// same weights can conform on one stack and not another. The router's re-ask
// had never run in a pass: `promptUniqueClient` buys every JSON reply through
// `chatText` and reads it itself, so the router's `chatJson` was never called.
//
// NUDGED, SO THE RE-ASK IS A NEW PROMPT. The uniqueness wrapper samples one
// model and prompt once; the same bytes asked again would be the same sample
// by that rule, and would come back from its cache in no time.
//
// A WORDING OF ITS OWN, not either of the stage recovery round's
// (`recovery-nudge.ts`). That round later re-asks every seat still unreadable
// with its nudge on whatever provider the policy picks; sharing a wording
// would make its prompt this re-ask's digest, answered from the claims with
// the reply that already failed, and the round would go silent for exactly
// the seats it exists for.
//
// ONCE, AND NEVER FROM A REPLY NO ROUTER TAGGED: a payload stored before the
// tag existed, or a client that routes nowhere, cannot name a provider to
// avoid, and the recovery round is its second chance.

/**
 Logger root for the re-ask.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 What the re-ask adds to the prompt.

 NEUTRAL ON WHY the reply could not be used, since it serves a reply that did
 not parse or match its shape and one that read as a refusal alike, and it
 must not press a model that declined.
 */
export const CROSS_PROVIDER_NUDGE: ChatMessage = {
  role: 'user',
  content: 'Your previous reply could not be used: it was not the JSON object the question asks for. '
    + 'Answer the same question again, replying with ONLY that JSON object, nothing before or after it.',
};

/**
 The first reply and what it turned out to be.
 */
type FirstAnswer<ValueT,> = {
  /**
   Reply as it arrived, tagged with its provider when a router served it.
   */
  readonly reply: ChatTextReply;

  /**
   What reading it produced.
   */
  readonly outcome: ChatJsonOutcome<ValueT>;
};

/**
 Re-asks a reply that could not be used, nudged, of the same model on another
 provider, and returns the better of the two outcomes.

 @param request - exchange the first reply answered

 @param first - first reply and its outcome

 @param ask - performs the nudged exchange, through whatever claims and
 stores the caller keeps

 @returns The re-ask's outcome when it is usable, else the first outcome

 @throws the re-ask's error when the caller's own signal aborted it

 @example
 ```ts
 const outcome = await reaskElsewhereNudged({ request, first: { reply, outcome, }, ask: chatText, },);
 ```
 */
export async function reaskElsewhereNudged<ValueT,>(
  {
    request,
    first,
    ask,
  }: {
    readonly request: ForeignBorrowed<ChatJsonRequest<ValueT>>;
    readonly first: FirstAnswer<ValueT>;
    readonly ask: (nudged: ForeignBorrowed<ChatTextRequest>,) => Promise<ChatTextReply>;
  },
): Promise<ChatJsonOutcome<ValueT>> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: reaskElsewhereNudged.name,
    l,
  },);
  if (first.outcome
    .kind
    === 'ok')
    return first.outcome;

  /**
   Provider the first reply came from, which the re-ask must avoid.
   */
  const { servedBy, } = first.reply;
  if (servedBy === undefined) {
    rl.info(`${request.modelId}: ${first.outcome
      .kind} from an untagged reply; no re-ask elsewhere`,);
    return first.outcome;
  }
  rl.info(`${request.modelId}: ${first.outcome
    .kind} on ${servedBy}; re-asking another provider, nudged`,);
  try {
    /**
     The nudged exchange's reply.
     */
    const reply = await ask({
      ...request,
      messages: [
        ...request.messages,
        CROSS_PROVIDER_NUDGE,
      ],
      otherThan: servedBy,
    },);

    /**
     What the other stack's answer turned out to be.
     */
    const second = readJsonOutcome({
      modelId: request.modelId,
      reply,
      validate: request.validate,
      ...responseFormatSpreadOf({ request, },),
    },);
    if (second.kind === 'ok') {
      rl.info(`${request.modelId}: ${reply.servedBy ?? 'another provider'} answered usably on the re-ask`,);
      return second;
    }
    // THE FIRST ANSWER IS RETURNED WHEN BOTH FAIL: it came from the provider
    // the policy preferred, and the caller's own handling is written against
    // it. Both are logged, so a reader can see the re-ask happened, and the
    // second failure rides on the returned outcome, so the line that records
    // the lost voice names both failures and not only the first's (the lead's
    // ruling of 2026-10-15 on the run of TianqiChen66625).
    rl.info(`${request.modelId}: ${reply.servedBy ?? 'another provider'} answered ${second.kind} too`,);
    return {
      ...first.outcome,
      reaskFailure: {
        servedBy: reply.servedBy ?? 'another provider',
        kind: second.kind,
        rawText: second.rawText,
        cause: (second.kind === 'schema-mismatch') ? second.detail : second.marker,
      },
    };
  }
  catch (error) {
    if (request.signal
      .aborted)
      throw error;
    // EVERY PROVIDER DRY CANNOT REACH THIS CATCH, which is why it is not checked here. The re-ask is hinted away from
    // the provider that served the first answer (`otherThan`), so the router serves it through `routedTextElsewhere`,
    // which reads the budgets once (`secondOpinionsFrom`) and raises no dry error of its own, only
    // `NoProviderForModelError`; the one path that raises `EveryProviderDryError` is `routedText`, which a hinted ask
    // never takes. Under the pass guard the shared signal is aborted before any error reaches here, which the
    // `request.signal.aborted` check rethrows.
    // THE FIRST ANSWER STANDS, since it is an answer and the re-ask was only a
    // second chance: no other provider could take it, one refused it on
    // budget, or its exchange failed. Its prompt was released by the caller's
    // claims, so a later ask of the same question may still buy it.
    rl.warn(`${request.modelId}: no re-ask elsewhere (${exchangeFailureLogText({ error, },)}); keeping the first answer`,);
    return first.outcome;
  }
}

//endregion Nudged re-ask elsewhere
