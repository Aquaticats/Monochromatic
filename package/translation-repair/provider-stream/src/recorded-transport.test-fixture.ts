import type {
  ModelTransport,
  TransportExchange,
  TransportReply,
} from '../dist/final/node/index.mjs';

//region Recorded transport
// A TRANSPORT REPLAYING RECORDED REPLIES IN CALL ORDER WHILE RECORDING EVERY
// EXCHANGE, the last reply repeating once the script runs short, for cases
// that need no live provider.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The hyper-client and synthetic-client
// tests kept their own copy of this transport; both now import it from here.

/**
 Builds a transport replaying recorded replies in order while recording every
 exchange for assertions.

 @param replies - replies replayed in call order, last one repeating

 @returns Transport plus its recorded exchanges

 @example
 ```ts
 const { transport, exchanges, } = recordedTransport({ replies: [reply,], },);
 ```
 */
export function recordedTransport(
  { replies, }: { readonly replies: readonly TransportReply[]; },
): {
  readonly transport: ModelTransport;
  readonly exchanges: TransportExchange[];
} {
  /**
   Every exchange the client performed, in order.
   */
  const exchanges: TransportExchange[] = [];

  return {
    transport: function replay(exchange,): Promise<TransportReply> {
      exchanges.push(exchange,);

      /**
       Reply for this exchange; the last recorded reply repeats.
       */
      const reply = replies[Math.min(
        exchanges.length - 1,
        replies.length - 1,
      )];
      if (reply === undefined)
        return Promise.reject(new Error('recordedTransport needs at least one reply',),);
      return Promise.resolve(reply,);
    },
    exchanges,
  };
}

//endregion Recorded transport
