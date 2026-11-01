import type {
  ChatTextRequest,
  createRoutingClient,
} from '../dist/final/node/index.mjs';

//region Provider router text call
// ROUTES ONE TEXT CALL AND REPORTS WHAT HAPPENED, catching what the router
// throws rather than letting a case fail on it, so a case can assert on the
// outcome either way.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The provider-router, provider-router-model-hold
// and provider-router-stream-bound tests kept their own copy of this call;
// all now import it from here.

/**
 Routes one text call and reports what happened.

 @param client - router under test

 @param messages - conversation the call carries

 @param signal - abort signal the call carries

 @param modelId - model to ask

 @returns Reply text, or the error thrown

 @example
 ```ts
 const outcome = await textCallOutcome({ client, messages: MESSAGES, signal: SIGNAL, modelId: SEAT, },);
 ```
 */
export async function textCallOutcome(
  {
    client,
    messages,
    signal,
    modelId,
  }: {
    readonly client: ReturnType<typeof createRoutingClient>;
    readonly messages: ChatTextRequest['messages'];
    readonly signal: ChatTextRequest['signal'];
    readonly modelId: Parameters<ReturnType<typeof createRoutingClient>['chatText']>[0]['modelId'];
  },
): Promise<{ readonly text: string; } | { readonly thrown: unknown; }> {
  try {
    /**
     Reply the router's client gave back.
     */
    const reply = await client.chatText({
      modelId,
      messages,
      signal,
    },);
    return {
      text: reply.text,
    };
  } catch (error) {
    return { thrown: error, };
  }
}

//endregion Provider router text call
