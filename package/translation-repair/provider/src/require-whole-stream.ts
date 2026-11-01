import { isSuccessStatus, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { TransportReply, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { requireStreamTerminator, } from './stream-completion.ts';

//region Require whole stream
// THE RETRY LADDER'S CHECK FOR AN OPENAI-COMPATIBLE STREAM, shared by NeuralWatt's client and the direct clients.

/**
 Refuses a success reply whose stream never reached its terminator, so the
 retry ladder retries the attempt as the transport failure it is.

 @param reply - one attempt's reply, read before the ladder returns it

 @throws {@link MalformedCompletionError}
 when the stream stops before `[DONE]`

 @example
 ```ts
 requireWholeStream(reply,);
 ```
 */
export function requireWholeStream(reply: TransportReply,): void {
  if (isSuccessStatus({ status: reply.status, },))
    requireStreamTerminator({ bodyText: reply.bodyText, },);
}

//endregion Require whole stream
