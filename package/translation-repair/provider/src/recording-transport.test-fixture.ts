/**
 A transport that records each request body and refuses it, for tests that
 read what a client would send and never need a reply.

 @module
 */

import type { TransportExchange, } from '@monochromatic-dev/translation-repair-provider-stream';

/**
 One provider's client over a transport that records the body and fails.

 @param bodies - where the recorded bodies go

 @returns Transport recording each chat body

 @example
 ```ts
 const transport = recordingTransport({ bodies, },);
 ```
 */
export function recordingTransport(
  { bodies, }: { readonly bodies: string[]; },
): (exchange: TransportExchange,) => Promise<never> {
  return async function record(exchange,): Promise<never> {
    bodies.push(exchange.bodyJson ?? '',);
    await Promise.resolve();
    throw new Error('recorded, not sent',);
  };
}

/**
 A transport that keeps each whole exchange it is handed and refuses it, for
 tests that read what a client puts on the exchange besides the body.

 @param seen - where the exchanges go

 @returns Transport recording each exchange

 @example
 ```ts
 const transport = keepingTransport({ seen, },);
 ```
 */
export function keepingTransport(
  { seen, }: { readonly seen: TransportExchange[]; },
): (exchange: TransportExchange,) => Promise<never> {
  return async function keep(exchange,): Promise<never> {
    seen.push(exchange,);
    await Promise.resolve();
    throw new Error('recorded, not sent',);
  };
}

/**
 Awaits a call whose failure is expected, since the transport records and
 refuses every body.

 @param call - the call to settle

 @example
 ```ts
 await settled(client.chatText(request,),);
 ```
 */
export async function settled(call: Promise<unknown>,): Promise<void> {
  try {
    await call;
  }
  catch (error) {
    // The transport refuses by design; the recorded body is what is read.
    void error;
  }
}
