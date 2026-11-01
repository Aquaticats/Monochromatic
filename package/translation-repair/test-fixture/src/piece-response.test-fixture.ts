//region Piece response
// A RESPONSE WHOSE BODY ARRIVES IN PIECES the case chooses, so a text can be cut
// at a place the case names and the stream can end clean or fail after the
// last piece. Cat-themed invention; nothing here is a real reply.

/**
 Builds a response whose body hands over one piece per pull.

 @param pieces - texts delivered in order, each as one chunk

 @param ending - how the body ends after the last piece: `close` for a clean end,
 or the failure the stream errors with

 @param status - HTTP status of the response

 @returns The response

 @example
 ```ts
 const response = pieceResponse({ pieces: ['bad key whisker-', 'key-7421',], ending: 'close', status: 200, },);
 ```
 */
export function pieceResponse(
  {
    pieces,
    ending,
    status,
  }: {
    readonly pieces: readonly string[];
    readonly ending: 'close' | Error;
    readonly status: number;
  },
): Response {
  /**
   Pieces not yet handed over.
   */
  const waiting = [...pieces,];

  /**
   Encoder, since a body carries bytes.
   */
  const encoder = new TextEncoder();
  return new Response(
    new ReadableStream<Uint8Array>({
      pull(controller,): void {
        /**
         Next piece, or nothing once all are out.
         */
        const next = waiting.shift();
        if (next !== undefined) {
          controller.enqueue(encoder.encode(next,),);
          return;
        }
        if (ending === 'close') {
          controller.close();
          return;
        }
        controller.error(ending,);
      },
    },),
    { status, },
  );
}

//endregion Piece response
