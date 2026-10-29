// Stdio transport: reads JSON-RPC from stdin, dispatches through server handle, writes responses to stdout.

import { once, } from 'node:events';
import type { Writable, } from 'node:stream';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import {
  isJsonRpcMessage,
  JSON_RPC_INTERNAL_ERROR,
  JSON_RPC_INVALID_REQUEST,
  JSON_RPC_PARSE_ERROR,
  type JsonRpcId,
  type JsonRpcInbound,
  type JsonRpcOutbound,
} from './json-rpc.ts';
import { readLines, } from './line-reader.ts';

import {
  type McpServerHandle,
  NO_RESPONSE,
} from './server-types.ts';
import { CANCELLED_NOTIFICATION, } from './server-response.ts';
import {
  createSerialRequestQueue,
  NO_FRAME,
  type SerialRequestQueue,
  UNCANCELLABLE,
} from './transport-queue.ts';

//region Output writer abstraction: supports both Bun FileSink and standard WritableStream

/**
 Minimal writer interface for stdout output.
 Accepts any object whose `write(Uint8Array)` reports a byte count, which covers
 `Bun.stdout.writer()` and the {@link processStdoutWriter} helper.
 
 A `WritableStreamDefaultWriter` does not satisfy this: its `write` resolves to `void`,
 so wrap one in an adapter that returns the byte count rather than passing it directly.
 
 @example
 ```ts
 const writer: StdoutWriter = processStdoutWriter();
 await writer.write(new TextEncoder().encode('hello\n'));
 ```
 */
export type StdoutWriter = {
  readonly write: (data: Uint8Array,) => number | Promise<number>;
};

/**
 Creates a {@link StdoutWriter} backed by `process.stdout.write`.
 Cross-runtime alternative to `Bun.stdout.writer()` that works in Node, Bun, and Deno.
 
 Honors backpressure by waiting for `drain` whenever the stream stops accepting writes.
 Discarding that signal would let a server outrunning its client accumulate every
 unflushed response in memory; a tool returning many megabytes of command output is
 exactly that case.
 
 @param stream - Destination stream, defaulting to process stdout; injectable so backpressure is testable.
 
 @returns Writer that delegates to the stream and waits when it backs up.
 
 @example
 ```ts
 const writer = processStdoutWriter();
 await writer.write(new TextEncoder().encode('hello\n'));
 ```
 */
export function processStdoutWriter(
  { stream = process.stdout, }: { readonly stream?: Writable; } = {},
): StdoutWriter {
  return {
    /**
     Writes one byte chunk to the stream and waits when the stream asks it to pause.
     
     @param data - Bytes passed to Node stream.
     
     @returns Number of bytes handed to the stream.
     
     @mutates data - `write` may retain byte storage until output consumption completes.
     
     @mutates stream - Appends to stream buffer and advances its drain state.
     */
    async write(data: Uint8Array,): Promise<number> {
      /**
       Whether the chunk fit in the stream buffer; `false` means the buffer is over its
       high-water mark and the stream is asking the writer to pause until it drains.
       */
      const accepted = stream.write(data,);
      // `once` rejects if the stream errors first, so a broken pipe surfaces here rather
      // than hanging this await forever.
      if (!accepted)
        await once(
          stream,
          'drain',
        );
      return data.byteLength;
    },
  };
}

//endregion

//region Stdio message loop: reads stdin lines, validates, dispatches, writes responses

/**
 Connects an MCP server handle to stdin/stdout using newline-delimited JSON-RPC.
 Reads lines from stdin via {@link readLines}, parses and validates each as a JSON-RPC
 message via {@link isJsonRpcMessage}, dispatches to the server, and writes responses
 as newline-terminated JSON to stdout via {@link writeSerializedMessage}.
 
 Runs until stdin closes (the client terminates the subprocess).
 
 @param server - Immutable server handle created by {@link createMcpServer}.
 
 @param input - Async iterable of byte chunks for incoming messages. Defaults to `process.stdin`.
 
 @param output - Writer for outgoing messages. Defaults to a `process.stdout.write` wrapper.
 
 @mutates output - writeSerializedMessage delegates each response to output.write, which changes output stream state.
 
 @example
 ```ts
 import { createMcpServer, defineTool, serve } from '\@monochromatic-dev/mcp-stdio';
 
 const server = createMcpServer({ config: { name: 'demo', version: '0.1.0' }, tools: [] });
 await serve({ server });
 ```
 */
export async function serve(
  {
    server,
    input = process.stdin,
    output = processStdoutWriter(),
  }: {
    readonly server: McpServerHandle;
    readonly input?: AsyncIterable<Uint8Array>;
    readonly output?: StdoutWriter;
  },
): Promise<void> {
  /**
   Reused across every outbound message so each call avoids allocating a fresh encoder.
   */
  const encoder = new TextEncoder();

  /**
   Owns execution order, cancellation, and write order for this connection.
   
   Every outbound frame goes through it, error frames included: once dispatch no longer
   blocks this loop, a frame written directly from here could interleave with one a
   running request is writing.
   */
  const queue = createSerialRequestQueue({
    async write(frame: string,): Promise<void> {
      console.error(`[mcp-stdio] -> ${frame}`,);
      await writeSerializedMessage({
        writer: output,
        encoder,
        serialized: frame,
      },);
    },
  },);

  /**
   Drains accepted work on every exit from this function, not only a clean one.
   
   Stdin failing mid-session would otherwise abandon requests already accepted, losing
   their replies exactly as returning early once did.
   */
  await using _drainOnExit = {
    async [Symbol.asyncDispose](): Promise<void> {
      await queue.idle();
    },
  };

  for await (const line of readLines(input,)) {
    if (line.trim()
      .length
      === 0)
      continue;

    /**
     Holds the parsed JSON value, or stays `undefined` if `JSON.parse` threw.
     
     Declared with `let` because the assignment happens inside the try block; the catch
     branch needs a binding visible at this scope to write the parse-error response.
     */
    let parsed: unknown = undefined;
    try {
      parsed = JSON.parse(line,);
    }
    catch (error: unknown) {
      console.error(
        '[mcp-stdio] failed to parse JSON from stdin:',
        error,
      );
      /**
       Parse-error response returned with `id: null` because the original id cannot be recovered.
       */
      const errorResponse: JsonRpcOutbound = {
        jsonrpc: '2.0',
        id: null,
        error: {
          code: JSON_RPC_PARSE_ERROR,
          message: 'Failed to parse JSON',
        },
      };
      queueFrame({
        queue,
        serialized: JSON.stringify(errorResponse,),
      },);
      continue;
    }

    // Validate step: ensure the parsed value is a valid JSON-RPC 2.0 message
    // before dispatching. Catches non-object values, missing jsonrpc field, etc.
    if (!isJsonRpcMessage(parsed,)) {
      console.error(
        '[mcp-stdio] received invalid JSON-RPC message (missing jsonrpc or method):',
        parsed,
      );
      /**
       Shape-error response when the message parsed as JSON but is not a JSON-RPC message.
       Uses invalid-request rather than parse-error: the text was valid JSON, so `JSON.parse`
       never failed and reporting a parse failure would misdirect the client.
       */
      const errorResponse: JsonRpcOutbound = {
        jsonrpc: '2.0',
        id: null,
        error: {
          code: JSON_RPC_INVALID_REQUEST,
          message:
            'Invalid JSON-RPC message: requires jsonrpc "2.0", a string method, '
              + 'a number or string id when present, and object params when present',
        },
      };
      queueFrame({
        queue,
        serialized: JSON.stringify(errorResponse,),
      },);
      continue;
    }

    console.error(`[mcp-stdio] <- ${line}`,);

    /**
     Request this message cancels, when it is a well-formed cancellation.
     */
    const cancelling = cancelledRequestId(parsed,);
    // Applied here rather than through the queue: waiting its turn would defeat the point,
    // since the request it names is what would be holding the queue.
    if (cancelling !== NOT_A_CANCELLATION) {
      if (!queue.cancel({ id: cancelling, },))
        console.error(`[mcp-stdio] cancellation names no live request: ${String(cancelling,)}`,);
      continue;
    }

    queue.enqueue({
      id: ('id' in parsed) ? parsed.id : UNCANCELLABLE,
      async produce(): Promise<string | typeof NO_FRAME> {
        /**
         Id this message expects echoed back, or `null` for a notification.
         */
        const replyId = ('id' in parsed) ? parsed.id : null;
        // Deliberate catch-and-return: dispatch failing without a frame would leave the
        // client waiting on a reply that can never arrive, which is worse than reporting
        // an internal error it can act on.
        try {
          /**
           Dispatch result; `NO_RESPONSE` indicates a notification (no reply expected).
           */
          const response = await server.handleMessage(parsed,);
          if (response === NO_RESPONSE)
            return NO_FRAME;
          return serializeResponse({
            response,
            id: replyId,
          },);
        }
        catch (error: unknown) {
          console.error(
            '[mcp-stdio] dispatch failed:',
            error,
          );
          if (replyId === null)
            return NO_FRAME;
          return serializeResponse({
            response: {
              jsonrpc: '2.0',
              id: replyId,
              error: {
                code: JSON_RPC_INTERNAL_ERROR,
                message: `Dispatch failed: ${caughtValueText(error,)}`,
              },
            },
            id: replyId,
          },);
        }
      },
    },);
  }

}

//endregion

//region Cancellation: recognizing which request a notification cancels

/**
 Returned when a message is not a well-formed cancellation notification.
 */
const NOT_A_CANCELLATION: unique symbol = Symbol('mcp-stdio message is not a cancellation',);

/**
 Reads the request id a cancellation notification names.
 
 Revision 2026-07-28 declares `requestId` as a `RequestId`, so a cancellation carrying a
 missing or wrongly typed one names nothing and is treated as an ordinary notification
 rather than silently cancelling some other request.
 
 @param message - Validated inbound message
 
 @returns Request id to cancel, or {@link NOT_A_CANCELLATION}
 
 @example
 ```ts
 cancelledRequestId({
   jsonrpc: '2.0',
   method: 'notifications/cancelled',
   params: { requestId: 7 },
 });
 // 7
 ```
 */
function cancelledRequestId(message: JsonRpcInbound,): JsonRpcId | typeof NOT_A_CANCELLATION {
  if (message.method !== CANCELLED_NOTIFICATION)
    return NOT_A_CANCELLATION;
  // A cancellation is a notification; one carrying an id is malformed, and answering it
  // as a request would be a likelier reading of the client's intent than cancelling.
  if ('id' in message)
    return NOT_A_CANCELLATION;
  /**
   Request id this notification claims to cancel, before type validation.
   */
  const requestId = message.params
    ?.requestId;
  if ((typeof requestId) === 'number')
    return requestId;
  if ((typeof requestId) === 'string')
    return requestId;
  return NOT_A_CANCELLATION;
}

/**
 Queues one already-serialized frame that no cancellation can match.
 
 @param queue - Queue owning write order
 
 @param serialized - JSON text ready for framing
 
 @mutates queue - Appends one entry.
 
 @example
 ```ts
 queueFrame({ queue, serialized: '{"jsonrpc":"2.0","id":null,"error":{}}' });
 ```
 */
function queueFrame(
  {
    queue,
    serialized,
  }: {
    readonly queue: SerialRequestQueue;
    readonly serialized: string;
  },
): void {
  queue.enqueue({
    id: UNCANCELLABLE,
    produce(): Promise<string> {
      return Promise.resolve(serialized,);
    },
  },);
}

//endregion

//region Response serialization: keeps an unserializable payload from killing the process

/**
 Serializes an outbound response, falling back to an internal-error frame when the
 payload cannot become JSON. A tool returning a cyclic object or a `bigint` would
 otherwise throw inside the read loop and close the connection mid-session, which a
 client sees as an unexplained disconnect instead of a failed call.
 
 @param response - Dispatch outcome awaiting transmission.
 
 @param id - Request id echoed by the fallback frame so the client can settle its call.
 
 @returns JSON text ready for framing.
 
 @example
 ```ts
 serializeResponse({ response: { jsonrpc: '2.0', id: 1, result: {} }, id: 1 });
 // '{"jsonrpc":"2.0","id":1,"result":{}}'
 ```
 */
function serializeResponse(
  {
    response,
    id,
  }: {
    readonly response: JsonRpcOutbound;
    // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- JSON-RPC 2.0 section 5 mandates the literal wire value `null` for `id` when the request id cannot be determined; this `null` is the external protocol's required output, not an internal absence sentinel.
    readonly id: JsonRpcId | null;
  },
): string {
  // Deliberate catch-and-return: serialization failure must reach the client as a frame,
  // not as a closed connection.
  try {
    return JSON.stringify(response,);
  }
  catch (error: unknown) {
    console.error(
      '[mcp-stdio] failed to serialize response:',
      error,
    );
    /**
     Replacement frame reporting that a well-formed result could not be encoded.
     */
    const errorResponse: JsonRpcOutbound = {
      jsonrpc: '2.0',
      id,
      error: {
        code: JSON_RPC_INTERNAL_ERROR,
        message: `Failed to serialize response: ${caughtValueText(error,)}`,
      },
    };
    return JSON.stringify(errorResponse,);
  }
}

//endregion

//region Message serialization: writes JSON-RPC responses to stdout

/**
 Writes serialized JSON-RPC text as newline-terminated UTF-8 to output stream.
 
 @param writer - Writer for stdout output.
 
 @param encoder - Reusable TextEncoder instance.
 
 @param serialized - JSON-RPC response already serialized where value ownership is known.
 
 
 @example
 ```ts
 await writeSerializedMessage({
   writer: processStdoutWriter(),
   encoder: new TextEncoder(),
   serialized: '{"jsonrpc":"2.0","id":1,"result":{}}',
 });
 ```
 */
async function writeSerializedMessage(
  {
    writer,
    encoder,
    serialized,
  }: {
    readonly writer: StdoutWriter;
    readonly encoder: Pick<TextEncoder, 'encode'>;
    readonly serialized: string;
  },
): Promise<void> {
  /**
   Newline-terminated JSON; MCP stdio framing requires one message per line.
   */
  const framed = `${serialized}\n`;
  await writer.write(encoder.encode(framed,),);
}

//endregion
