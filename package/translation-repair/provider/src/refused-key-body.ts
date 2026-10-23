import type {
  ModelTransport,
  TransportExchange,
  TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';

//region Refused key body
// THE GUARD EVERY ROUTE THAT SENDS A KEY PUTS ON ITS TRANSPORT, in a file of its own because the DeepSeek, Xiaomi and
// Claude routes all use it and none of them should reach into another route's client for it.

/**
 Status a provider answers when it rejects the credential its request carried.
 */
const HTTP_UNAUTHORIZED = 401;

/**
 Wraps a transport so a 401 reaches the caller without the provider's words.

 A PROVIDER'S REFUSAL OF A KEY MAY ECHO A FRAGMENT OF IT (DeepSeek's documented 401 shows the key's last four
 characters), and the transport's mask finds whole credentials, not tails: masking every four-unit piece of a key would
 rewrite ordinary words in the answers that carry no refusal. The status alone says everything the router and an
 operator act on (the provider is held out and the credential renewed), so the body is dropped.

 @param transport - transport the exchange would otherwise go over

 @returns Transport answering the same, with an empty body on a 401

 @example
 ```ts
 const guarded = withoutRefusedKeyBody({ transport: fetchTransport, },);
 ```
 */
export function withoutRefusedKeyBody({ transport, }: { readonly transport: ModelTransport; },): ModelTransport {
  return async function wordlessOnRefusedKey(exchange: TransportExchange,): Promise<TransportReply> {
    /**
     Reply as the wire answered it.
     */
    const reply = await transport(exchange,);
    if (reply.status !== HTTP_UNAUTHORIZED)
      return reply;
    return {
      ...reply,
      bodyText: '',
    };
  };
}

//endregion Refused key body
