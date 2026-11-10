/**
 Child program for `transport-frame-echo.unit.test.ts`.

 Serves one JSON-RPC request and one unterminated trailing frame through the built
 `serve()`, collects every frame the transport writes, then reports the collected
 frames together with the exact input text on stdout. A caller can therefore compare
 what reached stderr against what actually crossed the wire, without repeating any
 literal in two places.

 The test runs this program directly, never imports it:
 `node transport-frame-echo-child-program.ts`.

 @module
 */

import process from 'node:process';

import {
  type DispatchResult,
  type JsonRpcInbound,
  NO_RESPONSE,
  serve,
} from '@monochromatic-dev/mcp-stdio';

//region Input the caller compares stderr against

/**
 Valid JSON-RPC request frame, so the transport reads it and writes a response.
 */
const REQUEST_FRAME = '{"jsonrpc":"2.0","id":1,"method":"tools/list"}';

/**
 Trailing bytes with no newline terminator, cut mid-token so the line reader has to
 flush its buffer at end of stream and the transport has to reject the text.

 The text never forms a valid frame, so it reaches stderr only by being echoed.
 */
const UNTERMINATED_FRAME = '{"jsonrpc":"2.0","id":2,"method":"tools/lis';

/**
 Text carried only by the dispatch result, so a stderr hit proves a response payload
 was echoed rather than a frame name mentioned.
 */
const RESULT_PAYLOAD_MARKER = 'tool-result-payload-marker';

//endregion

/**
 Bytes the transport reads: both frames in one chunk, the second unterminated.
 */
const input = new ReadableStream<Uint8Array>({
  /**
   Hands the transport its bytes and ends its input.

   @param controller - Stream controller `serve` reads through.
   */
  start: function enqueueFrames(controller: ReadableStreamDefaultController<Uint8Array>,): void {
    controller.enqueue(
      new TextEncoder()
        .encode(`${REQUEST_FRAME}\n${UNTERMINATED_FRAME}`,),
    );
    controller.close();
  },
});

/**
 Replies to a request with the payload marker and sends nothing for a notification.

 @param message - Inbound message the transport dispatched.

 @returns Response frame, or `NO_RESPONSE` when the message expects no reply.
 */
function replyToRequest(message: JsonRpcInbound,): DispatchResult {
  if (!('id' in message))
    return NO_RESPONSE;
  return {
    jsonrpc: '2.0',
    id: message.id,
    result: { marker: RESULT_PAYLOAD_MARKER, },
  };
}

/**
 Complete outbound frames, in write order.
 */
const written: string[] = [];

/**
 Decodes each written byte chunk back to the frame text the transport handed over.
 */
const decoder = new TextDecoder();

/**
 Writer the transport sends its frames to, standing in for stdout so the report below
 is the only thing this program prints.
 */
const report = {
  requestFrame: REQUEST_FRAME,
  resultPayloadMarker: RESULT_PAYLOAD_MARKER,
  unterminatedFrame: UNTERMINATED_FRAME,
  written,
};

await serve({
  server: { handleMessage: replyToRequest, },
  input,
  output: {
    /**
     Records one outbound frame and reports the bytes accepted.

     @param data - Frame bytes the transport wrote.

     @returns Number of bytes accepted, as the stream writer contract requires.
     */
    write(data: Uint8Array,): number {
      written.push(decoder.decode(data,),);
      return data.byteLength;
    },
  },
},);

/**
 Serialized report, built before the write so no call nests inside another.
 */
const serializedReport = JSON.stringify(report,);

// stdout is this program's report channel: the transport wrote to the collecting writer
// above, so this line is the only thing the program prints.
process.stdout
  .write(`${serializedReport}\n`,);
