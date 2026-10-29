import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type {
  ClaudeAttemptAsk,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';
import type { ModelTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  CLAUDE_KEY_VARIABLE,
  PROBE_MAX_HOLD_MS,
  runClientFrom,
} from '@monochromatic-dev/translation-repair-run/ts';
import {
  ANTHROPIC_SERVED_IDS,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Editor fidelity client
// WHAT KEEPS THE PROBE'S SPEND ON THE CARDS IT WAS STARTED FOR, in two layers.
//
// THE ENVIRONMENT THE CLIENT IS BUILT OVER holds the Claude key and no other provider's. The run client configures a
// provider for every key it finds, so a launch under mise, which injects them all, would otherwise build a client
// that could route a call to any of them. Built over one key, it has one route, and a launch with that key unset
// reports that one route unconfigured and nothing else.
//
// THE GUARD AROUND THE CLIENT refuses any model the probe was not started for, before the exchange reaches the
// client. The probe asks nothing but the named cards, so a critic, a panel seat, a judge or a checker asked by mistake
// would be a fault, and this turns it into a thrown error that spends nothing, even on the same key.

/**
 Whether the Claude route serves a card.

 @param cardId - card named

 @returns Whether the card is one of the Claude spellings

 @example
 ```ts
 // yields true
 const served = isClaudeCard({ cardId: 'claude-haiku-5-5', },);
 ```
 */
function isClaudeCard({ cardId, }: { readonly cardId: RosterModelId; },): boolean {
  return ANTHROPIC_SERVED_IDS.some(function is(servedId,): boolean {
    return servedId === cardId;
  },);
}

/**
 The environment a client for the named cards is built over.

 @param env - process's environment, which may hold every provider's key

 @param cardIds - cards the probe seats

 @returns Environment holding the Claude key alone

 @throws StatedRefusalError when a card is not served by the Claude route, or the Claude key is unset or empty, in
 which case only the Claude route is named

 @example
 ```ts
 const env = editorProbeEnv({ env: process.env, cardIds, },);
 ```
 */
export function editorProbeEnv(
  {
    env,
    cardIds,
  }: {
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly cardIds: readonly RosterModelId[];
  },
): NodeJS.ProcessEnv {
  /**
   Card no Claude key reaches, if the list holds one.
   */
  const stranger = cardIds.find(function isStranger(cardId,): boolean {
    return !isClaudeCard({ cardId, },);
  },);
  if (stranger !== undefined) {
    throw new StatedRefusalError({
      says: `${stranger} is not served by the Claude route, the only one this probe asks`,
    },);
  }
  /**
   The Claude key, empty when unset.
   */
  const key = env[CLAUDE_KEY_VARIABLE] ?? '';
  if (key === '') {
    throw new StatedRefusalError({
      says: `${CLAUDE_KEY_VARIABLE} is not set, so the Claude route, the only one this probe asks (for ${
        cardIds.join(', ',)
      }), is unconfigured; run under mise so sops injects it`,
    },);
  }
  return { [CLAUDE_KEY_VARIABLE]: key, };
}

/**
 Wraps a client so it asks the named cards and no other model.

 @param inner - client to wrap

 @param cardIds - only models an exchange may name

 @returns Client with every member of the inner one, at run time, its exchanges refused for any other model

 @throws {@link Error} from an exchange naming a model that is not a named card, before the inner client is reached

 @example
 ```ts
 const client = cardsOnlyClient({ inner, cardIds, },);
 ```
 */
export function cardsOnlyClient(
  {
    inner,
    cardIds,
  }: {
    readonly inner: ModelClient;
    readonly cardIds: readonly RosterModelId[];
  },
): ModelClient {
  /**
   Refuses a model the probe was not started for.

   @param modelId - model an exchange names

   @throws {@link Error} when it is not a named card

   @example
   ```ts
   assertNamed({ modelId, },);
   ```
   */
  function assertNamed({ modelId, }: { readonly modelId: string; },): void {
    if (!cardIds.some(function is(cardId,): boolean {
      return cardId === modelId;
    },)) {
      throw new Error(
        `the editor fidelity probe asked ${modelId}, which is not one of the cards it was started for (${
          cardIds.join(', ',)
        }); nothing was sent`,
      );
    }
  }

  return {
    ...inner,
    chatText: function guardedText(request,) {
      assertNamed({ modelId: request.modelId, },);
      return inner.chatText(request,);
    },
    chatJson: function guardedJson(request,) {
      assertNamed({ modelId: request.modelId, },);
      return inner.chatJson(request,);
    },
  };
}

/**
 Builds the client the cards are asked through: the run client over the Claude key alone, guarded to the cards named,
 and told the admit of the caller for each attempt the Claude client makes.

 @param env - process's environment, which may hold every provider's key

 @param transport - HTTP the client calls over: `fetchTransport` in a run, a script in a test

 @param promptPayloadDir - where the client keeps request payloads for the run

 @param cardIds - cards the run seats

 @param admitClaudeAttempt - told by the Claude client before each attempt it makes, which ends the call where it throws

 @returns Client that refuses any model that is not one of the cards and tells the caller of each attempt

 @throws StatedRefusalError when a card is not served by the Claude route, or the Claude key is unset or empty

 @example
 ```ts
 const client = cardsClientFrom({ env: process.env, transport: fetchTransport, promptPayloadDir, cardIds, admitClaudeAttempt, },);
 ```
 */
export function cardsClientFrom(
  {
    env,
    transport,
    promptPayloadDir,
    cardIds,
    admitClaudeAttempt,
  }: {
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly transport: ModelTransport;
    readonly promptPayloadDir: string;
    readonly cardIds: readonly RosterModelId[];
    readonly admitClaudeAttempt?: (attempt: ClaudeAttemptAsk,) => void;
  },
): ModelClient {
  return cardsOnlyClient({
    inner: runClientFrom({
      env: editorProbeEnv({
        env,
        cardIds,
      },),
      transport,
      promptPayloadDir,
      maxHoldMs: PROBE_MAX_HOLD_MS,
      // Conditional spread keeps the hook absent instead of undefined.
      ...((admitClaudeAttempt === undefined) ? {} : { admitClaudeAttempt, }),
    },),
    cardIds,
  },);
}

//endregion Editor fidelity client
