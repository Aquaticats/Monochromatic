import { PassThrough, } from 'node:stream';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import {
  createSerialRequestQueue,
  type DispatchResult,
  JSON_RPC_INTERNAL_ERROR,
  JSON_RPC_INVALID_REQUEST,
  JSON_RPC_PARSE_ERROR,
  type JsonRpcId,
  type JsonRpcOutbound,
  type McpServerHandle,
  NO_RESPONSE,
  NO_FRAME,
  processStdoutWriter,
  serve,
  UNCANCELLABLE,
  type StdoutWriter,
} from '@monochromatic-dev/mcp-stdio';

//region helpers: test doubles for stdin/stdout and server handle

/**
 Creates a ReadableStream from newline-delimited messages.
 
 @param messages - Raw strings to send as stdin lines (newline appended automatically).
 @returns ReadableStream simulating stdin.
 */
function stdinFromMessages(messages: readonly string[],): ReadableStream<Uint8Array> {
  const encoder = new TextEncoder();
  const content = messages.map(message => `${message}\n`).join('',);
  return new ReadableStream({
    start(controller: ReadableStreamDefaultController<Uint8Array>,) {
      controller.enqueue(encoder.encode(content,),);
      controller.close();
    },
  },);
}

/**
 Creates a StdoutWriter that collects all written output into a string array.
 
 @returns Object with writer and collected output lines.
 */
function collectingWriter(): { writer: StdoutWriter; lines: string[]; } {
  const decoder = new TextDecoder();
  const lines: string[] = [];
  const writer: StdoutWriter = {
    write(data: Uint8Array,): number {
      const text = decoder.decode(data,);
      // Split on newlines and filter trailing empty string from final newline.
      const parts = text.split('\n',).filter((part, index, array,) =>
        (index < (array.length - 1)) || (part.length > 0)
      );
      lines.push(...parts,);
      return data.length;
    },
  };
  return { writer, lines, };
}

/**
 Milliseconds the backpressure probe waits before concluding a write is still parked.
 Nothing consumes the stream during that window, so a writer honoring backpressure cannot
 resolve within it no matter how long the window is; the value only bounds the test.
 */
const BACKPRESSURE_PROBE_MS = 25;

/**
 Creates a mock MCP server handle that returns a fixed response for any request.
 
 @param response - Dispatch result to return for all requests; `NO_RESPONSE` for notifications.
 @returns McpServerHandle that always returns the given response.
 */
function mockServer(response: DispatchResult,): McpServerHandle {
  return {
    handleMessage: () => response,
  };
}

/**
 Milliseconds a deliberately slow tool occupies the queue.
 
 Long enough that the read loop consumes every remaining line while it runs, which is what
 makes a cancellation arrive mid-flight rather than after the queue has already emptied.
 */
const SLOW_TOOL_MS = 30;

/**
 Creates a server handle recording which request ids reached dispatch, delaying chosen ones.
 
 `dispatched` is what separates "cancelled before it ran" from "ran and was silenced": a
 suppressed reply looks identical from the wire, so the assertion needs the handler's own view.
 
 @param slowIds - Request ids whose handler waits before answering
 
 @returns Handle plus the mutable record of dispatched ids
 */
function recordingServer(
  { slowIds = [], }: { readonly slowIds?: readonly JsonRpcId[]; } = {},
): {
  readonly server: McpServerHandle;
  readonly dispatched: JsonRpcId[];
} {
  /**
   Request ids that reached the handler, in dispatch order.
   */
  const dispatched: JsonRpcId[] = [];
  return {
    dispatched,
    server: {
      async handleMessage(message,): Promise<DispatchResult> {
        if (!('id' in message))
          return NO_RESPONSE;
        dispatched.push(message.id,);
        if (slowIds.includes(message.id,))
          await wait(SLOW_TOOL_MS,);
        return {
          jsonrpc: '2.0' as const,
          id: message.id,
          result: {},
        };
      },
    },
  };
}

/**
 Reads the `id` of every response line, in the order they were written.
 
 @param lines - Collected output lines
 
 @returns Ids in write order
 */
function writtenIds(lines: readonly string[],): readonly unknown[] {
  return lines.map(function idOf(line,): unknown {
    return (JSON.parse(line,) as { id: unknown; }).id;
  },);
}

/**
 Builds a cancellation notification naming one request.
 
 @param requestId - Request to cancel
 
 @returns Serialized notification line
 */
function cancelLine(requestId: JsonRpcId,): string {
  return JSON.stringify({
    jsonrpc: '2.0',
    method: 'notifications/cancelled',
    params: { requestId, },
  },);
}

/**
 Builds a request line for a method that needs no params.
 
 @param id - Request id
 
 @returns Serialized request line
 */
function requestLine(id: JsonRpcId,): string {
  return JSON.stringify({
    jsonrpc: '2.0',
    id,
    method: 'server/discover',
  },);
}

//endregion helpers

//region serve; stdio transport connecting stdin/stdout to server handle

await describe({
  name: serve.name,
  children: [
    it({
      name: 'parses valid JSON-RPC message and writes response',
      fn: async () => {
        const serverResponse: JsonRpcOutbound = { jsonrpc: '2.0', id: 1,
          result: { tools: [], }, };
        const server = mockServer(serverResponse,);
        const input = stdinFromMessages([
          '{"jsonrpc":"2.0","id":1,"method":"tools/list"}',
        ],);
        const { writer, lines, } = collectingWriter();

        await serve({ server, input, output: writer, },);

        expect(lines,).toEqual([JSON.stringify(serverResponse,),],);
      },
    },),
    it({
      name: 'skips blank lines without sending them to server',
      fn: async () => {
        const serverResponse: JsonRpcOutbound = { jsonrpc: '2.0', id: 1, result: {}, };
        const server = mockServer(serverResponse,);
        const input = stdinFromMessages(['', '  ',
          '{"jsonrpc":"2.0","id":1,"method":"server/discover"}',],);
        const { writer, lines, } = collectingWriter();

        await serve({ server, input, output: writer, },);

        expect(lines,).toHaveLength(1,);
      },
    },),
    it({
      name: 'returns parse error for invalid JSON',
      fn: async () => {
        const server = mockServer(NO_RESPONSE,);
        const input = stdinFromMessages(['not-json',],);
        const { writer, lines, } = collectingWriter();

        await serve({ server, input, output: writer, },);

        expect(lines,).toHaveLength(1,);
        const parsed = JSON.parse(lines[0] ?? '{}',) as JsonRpcOutbound;
        expect((parsed as { error: { code: number; }; }).error.code,).toBe(
          JSON_RPC_PARSE_ERROR,
        );
      },
    },),
    it({
      name: 'returns invalid request for valid JSON that is not a JSON-RPC message',
      fn: async () => {
        const server = mockServer(NO_RESPONSE,);
        const input = stdinFromMessages(['{"not":"jsonrpc"}',],);
        const { writer, lines, } = collectingWriter();

        await serve({ server, input, output: writer, },);

        expect(lines,).toHaveLength(1,);
        const parsed = JSON.parse(lines[0] ?? '{}',) as JsonRpcOutbound;
        expect((parsed as { error: { code: number; }; }).error.code,).toBe(
          JSON_RPC_INVALID_REQUEST,
        );
      },
    },),
    it({
      name: 'returns invalid request for a message whose id is null',
      fn: async () => {
        const server = mockServer(NO_RESPONSE,);
        const input = stdinFromMessages([
          '{"jsonrpc":"2.0","id":null,"method":"tools/list"}',
        ],);
        const { writer, lines, } = collectingWriter();

        await serve({ server, input, output: writer, },);

        expect(lines,).toHaveLength(1,);
        const parsed = JSON.parse(lines[0] ?? '{}',) as JsonRpcOutbound;
        expect((parsed as { error: { code: number; }; }).error.code,).toBe(
          JSON_RPC_INVALID_REQUEST,
        );
      },
    },),
    it({
      name: 'parks a write on a backed-up stream until it drains',
      fn: async () => {
        // A one-byte high-water mark means any real payload overshoots it, so the stream
        // refuses the write and asks the writer to wait. Nothing consumes the stream until
        // `resume()` below, so a writer honoring that request cannot resolve before then.
        const sink = new PassThrough({ highWaterMark: 1, },);
        const writer = processStdoutWriter({ stream: sink, },);

        const pending = writer.write(
          new TextEncoder().encode(`${'x'.repeat(1_024,)}\n`,),
        );

        /**
         Resolves once the write completes, labelling that outcome for the race below.
         */
        const flushOutcome = async (): Promise<string> => {
          await pending;
          return 'resolved';
        };
        /**
         Resolves after the probe window, labelling a write still parked on backpressure.
         */
        const probeOutcome = async (): Promise<string> => {
          await wait(BACKPRESSURE_PROBE_MS,);
          return 'parked';
        };

        // Dropping the drain wait makes this 'resolved': the writer hands the stream a
        // chunk it refused and reports success, so `serve` runs on and can return with
        // replies still unflushed, which a process exiting on stdin close then loses.
        expect(await Promise.race([flushOutcome(), probeOutcome(),],),).toBe('parked',);

        // Consuming the stream fires `drain`, which releases the parked write.
        sink.resume();
        expect(await pending,).toBe(1_025,);
      },
    },),
    it({
      name: 'returns an internal error frame when a result cannot be serialized',
      fn: async () => {
        /** Result carrying a `bigint`, which `JSON.stringify` refuses to encode. */
        const server: McpServerHandle = {
          handleMessage: () => ({
            jsonrpc: '2.0' as const,
            id: 1,
            result: { size: 1n, } as unknown,
          }),
        };
        const input = stdinFromMessages([
          '{"jsonrpc":"2.0","id":1,"method":"server/discover"}',
        ],);
        const { writer, lines, } = collectingWriter();

        await serve({ server, input, output: writer, },);

        expect(lines,).toHaveLength(1,);
        const parsed = JSON.parse(lines[0] ?? '{}',) as JsonRpcOutbound;
        expect((parsed as { error: { code: number; }; }).error.code,).toBe(
          JSON_RPC_INTERNAL_ERROR,
        );
        expect((parsed as { id: unknown; }).id,).toBe(1,);
      },
    },),
    it({
      name: 'does not write response for notifications',
      fn: async () => {
        const server = mockServer(NO_RESPONSE,);
        const input = stdinFromMessages([
          '{"jsonrpc":"2.0","method":"notifications/initialized"}',
        ],);
        const { writer, lines, } = collectingWriter();

        await serve({ server, input, output: writer, },);

        expect(lines,).toHaveLength(0,);
      },
    },),
    it({
      name: 'handles multiple messages in sequence',
      fn: async () => {
        /** Counter to give each response a unique id. */
        let callCount = 0;
        const server: McpServerHandle = {
          handleMessage: () => {
            callCount += 1;
            return { jsonrpc: '2.0' as const, id: callCount, result: {}, };
          },
        };
        const input = stdinFromMessages([
          '{"jsonrpc":"2.0","id":1,"method":"server/discover"}',
          '{"jsonrpc":"2.0","id":2,"method":"server/discover"}',
          '{"jsonrpc":"2.0","id":3,"method":"server/discover"}',
        ],);
        const { writer, lines, } = collectingWriter();

        await serve({ server, input, output: writer, },);

        expect(lines,).toHaveLength(3,);
      },
    },),
    it({
      name: 'never dispatches a request cancelled while it waited in the queue',
      fn: async () => {
        const {
          server,
          dispatched,
        } = recordingServer({ slowIds: [1,], },);
        // Request 1 holds the queue. Because the read loop no longer waits on dispatch, the
        // cancellation for request 2 is read while 2 is still queued behind 1.
        const input = stdinFromMessages([
          requestLine(1,),
          requestLine(2,),
          cancelLine(2,),
        ],);
        const { writer, lines, } = collectingWriter();

        await serve({ server, input, output: writer, },);

        expect(dispatched,).toEqual([1,],);
        expect(writtenIds(lines,),).toEqual([1,],);
      },
    },),
    it({
      name: 'lets a running request finish but withholds its reply once cancelled',
      fn: async () => {
        const {
          server,
          dispatched,
        } = recordingServer({ slowIds: [1,], },);
        const input = stdinFromMessages([
          requestLine(1,),
          cancelLine(1,),
        ],);
        const { writer, lines, } = collectingWriter();

        await serve({ server, input, output: writer, },);

        // Dispatched, so a half-created VM is never abandoned midway; unanswered, because the
        // schema says a cancelled request's result will be unused.
        expect(dispatched,).toEqual([1,],);
        expect(lines,).toEqual([],);
      },
    },),
    it({
      name: 'ignores a cancellation naming a request it never saw',
      fn: async () => {
        const { server, } = recordingServer();
        const input = stdinFromMessages([
          cancelLine(99,),
          requestLine(1,),
        ],);
        const { writer, lines, } = collectingWriter();

        await serve({ server, input, output: writer, },);

        // The schema allows a cancellation to arrive after its request finished, so an
        // unmatched one is ordinary traffic rather than an error worth answering.
        expect(writtenIds(lines,),).toEqual([1,],);
      },
    },),
    it({
      name: 'writes replies in request order even when the first one is slow',
      fn: async () => {
        const { server, } = recordingServer({ slowIds: [1,], },);
        const input = stdinFromMessages([
          requestLine(1,),
          requestLine(2,),
          requestLine(3,),
        ],);
        const { writer, lines, } = collectingWriter();

        await serve({ server, input, output: writer, },);

        expect(writtenIds(lines,),).toEqual([1, 2, 3,],);
      },
    },),
    it({
      name: 'keeps an error frame from overtaking a request already running',
      fn: async () => {
        const { server, } = recordingServer({ slowIds: [1,], },);
        // Every outbound frame shares one path, so the parse error queues behind the running
        // request instead of racing its reply onto the same stream.
        const input = stdinFromMessages([
          requestLine(1,),
          'not-json',
        ],);
        const { writer, lines, } = collectingWriter();

        await serve({ server, input, output: writer, },);

        expect(writtenIds(lines,),).toEqual([1, null,],);
      },
    },),
    it({
      name: 'drains queued work before returning when stdin closes',
      fn: async () => {
        const { server, } = recordingServer({ slowIds: [1,], },);
        const input = stdinFromMessages([
          requestLine(1,),
          requestLine(2,),
        ],);
        const { writer, lines, } = collectingWriter();

        // Returning as soon as stdin ends would strand request 2 and lose its reply, the same
        // class of loss as ignoring backpressure.
        await serve({ server, input, output: writer, },);

        expect(lines,).toHaveLength(2,);
      },
    },),
    it({
      name: 'answers a request whose dispatch threw instead of leaving it unanswered',
      fn: async () => {
        /** Server whose dispatch rejects rather than returning a response. */
        const server: McpServerHandle = {
          handleMessage: () => {
            throw new Error('dispatch exploded',);
          },
        };
        const input = stdinFromMessages([requestLine(1,),],);
        const { writer, lines, } = collectingWriter();

        await serve({ server, input, output: writer, },);

        // Silence here is the worst outcome available: the client waits on a reply that
        // can never arrive, with no way to tell a slow tool from a dead one.
        expect(lines,).toHaveLength(1,);
        const parsed = JSON.parse(lines[0] ?? '{}',) as {
          id: unknown;
          error: { code: number; };
        };
        expect(parsed.id,).toBe(1,);
        expect(parsed.error.code,).toBe(JSON_RPC_INTERNAL_ERROR,);
      },
    },),
    it({
      name: 'drains accepted work even when stdin fails partway',
      fn: async () => {
        const { server, } = recordingServer({ slowIds: [1,], },);
        /** Stdin that yields one request, then fails as a broken pipe would. */
        const input = {
          async *[Symbol.asyncIterator](): AsyncGenerator<Uint8Array> {
            yield new TextEncoder().encode(`${requestLine(1,)}\n`,);
            throw new Error('stdin failed',);
          },
        };
        const { writer, lines, } = collectingWriter();

        // Deliberate catch: the failure must still propagate, but not before the reply
        // for work already accepted has been written.
        try {
          await serve({ server, input, output: writer, },);
        }
        catch (error: unknown) {
          expect(caughtValueText(error,),).toContain('stdin failed',);
        }

        expect(writtenIds(lines,),).toEqual([1,],);
      },
    },),
    it({
      name: 'surfaces a failing stdout rather than reporting a clean shutdown',
      fn: async () => {
        const { server, } = recordingServer();
        const input = stdinFromMessages([requestLine(1,),],);
        /** Writer standing in for a stdout whose pipe has closed. */
        const output: StdoutWriter = {
          write: () => Promise.reject(new Error('EPIPE broken pipe',),),
        };

        // A reply that never reached the client is not a successful run. Resolving here
        // would tell the caller the session ended cleanly while its answer was lost.
        try {
          await serve({ server, input, output, },);
          expect('serve resolved',).toBe('serve should have thrown',);
        }
        catch (error: unknown) {
          expect(caughtValueText(error,),).toContain('EPIPE',);
        }
      },
    },),
    it({
      name: 'continues processing after encountering invalid JSON',
      fn: async () => {
        const serverResponse: JsonRpcOutbound = { jsonrpc: '2.0', id: 1, result: {}, };
        const server = mockServer(serverResponse,);
        const input = stdinFromMessages([
          'bad-json',
          '{"jsonrpc":"2.0","id":1,"method":"server/discover"}',
        ],);
        const { writer, lines, } = collectingWriter();

        await serve({ server, input, output: writer, },);

        // One parse error response + one valid response.
        expect(lines,).toHaveLength(2,);
      },
    },),
  ],
},);

//endregion serve

//region createSerialRequestQueue; ordering, reentrancy, and cancellation bookkeeping

await describe({
  name: createSerialRequestQueue.name,
  children: [
    it({
      name: 'runs one drain when a producer enqueues during the first producer call',
      fn: async () => {
        /** Frames written, in order. */
        const written: string[] = [];
        const queue = createSerialRequestQueue({
          write: async (frame: string,) => {
            written.push(frame,);
            await wait(0,);
          },
        },);

        queue.enqueue({
          id: 1,
          produce: () => {
            // Reentrant enqueue before the first suspension. The active-drain marker is
            // published before any drain code runs, so this must join the running drain
            // rather than start a second one racing it.
            queue.enqueue({
              id: 2,
              produce: () => Promise.resolve('second',),
            },);
            return Promise.resolve('first',);
          },
        },);

        await queue.idle();

        expect(written,).toEqual(['first', 'second',],);
      },
    },),
    it({
      name: 'keeps two requests sharing one id independently cancellable',
      fn: async () => {
        /** Frames written, in order. */
        const written: string[] = [];
        const queue = createSerialRequestQueue({
          write: (frame: string,) => {
            written.push(frame,);
            return Promise.resolve();
          },
        },);

        // A client should never leave two requests outstanding under one id, but if it
        // does, settling the first must not erase the second's cancellation state.
        queue.enqueue({ id: 7, produce: () => Promise.resolve('a',), },);
        queue.enqueue({ id: 7, produce: () => Promise.resolve('b',), },);
        queue.cancel({ id: 7, },);

        await queue.idle();

        expect(written,).toEqual([],);
      },
    },),
    it({
      name: 'surfaces a write failure through idle rather than resolving quietly',
      fn: async () => {
        const queue = createSerialRequestQueue({
          write: () => Promise.reject(new Error('pipe closed',),),
        },);
        queue.enqueue({ id: 1, produce: () => Promise.resolve('frame',), },);

        // Deliberate catch: a lost frame cannot be reported over the stream that lost it,
        // so the only honest signal left is refusing to resolve as though it succeeded.
        try {
          await queue.idle();
          expect('idle resolved',).toBe('idle should have thrown',);
        }
        catch (error: unknown) {
          expect(caughtValueText(error,),).toContain('pipe closed',);
        }
      },
    },),
    it({
      name: 'reports no match for a cancellation naming an unknown id',
      fn: async () => {
        const queue = createSerialRequestQueue({
          write: () => Promise.resolve(),
        },);
        expect(queue.cancel({ id: 'never-seen', },),).toBe(false,);
        expect(queue.cancel({ id: UNCANCELLABLE as unknown as number, },),).toBe(false,);
      },
    },),
    it({
      name: 'reports no match for a cancellation arriving after the request settled',
      fn: async () => {
        const queue = createSerialRequestQueue({
          write: () => Promise.resolve(),
        },);
        queue.enqueue({ id: 3, produce: () => Promise.resolve('done',), },);
        await queue.idle();

        // Bookkeeping is released on settle, so a late cancellation finds nothing. That is
        // also the limit of this design: were a client to reuse id 3 for a new request, a
        // cancellation meant for the old one would name the new one instead.
        expect(queue.cancel({ id: 3, },),).toBe(false,);
      },
    },),
    it({
      name: 'releases a cancelled entry immediately rather than at its turn',
      fn: async () => {
        /** Frames written, in order. */
        const written: string[] = [];
        const queue = createSerialRequestQueue({
          write: async (frame: string,) => {
            written.push(frame,);
            await wait(SLOW_TOOL_MS,);
          },
        },);

        queue.enqueue({ id: 1, produce: () => Promise.resolve('first',), },);
        queue.enqueue({ id: 2, produce: () => Promise.resolve('second',), },);

        // Cancelling while entry 1 is still writing must drop entry 2 from the waiting list
        // there and then, not leave it parked until it reaches the front. A second
        // cancellation naming it therefore matches nothing.
        expect(queue.cancel({ id: 2, },),).toBe(true,);
        expect(queue.cancel({ id: 2, },),).toBe(false,);

        await queue.idle();

        expect(written,).toEqual(['first',],);
      },
    },),
    it({
      name: 'stops the queue when a producer throws instead of leaving it unanswered',
      fn: async () => {
        /** Frames written, in order. */
        const written: string[] = [];
        const queue = createSerialRequestQueue({
          write: (frame: string,) => {
            written.push(frame,);
            return Promise.resolve();
          },
        },);
        queue.enqueue({
          id: 1,
          produce: () => Promise.reject(new Error('producer exploded',),),
        },);
        queue.enqueue({ id: 2, produce: () => Promise.resolve('second',), },);

        // Producers are expected to answer their own failures; one that throws anyway has
        // left a request permanently unanswered, which is a broken connection rather than a
        // skippable entry. Stopping surfaces it instead of quietly continuing.
        try {
          await queue.idle();
          expect('idle resolved',).toBe('idle should have thrown',);
        }
        catch (error: unknown) {
          expect(caughtValueText(error,),).toContain('producer exploded',);
        }
        expect(written,).toEqual([],);
      },
    },),
    it({
      name: 'writes nothing for a producer yielding the no-frame sentinel',
      fn: async () => {
        /** Frames written, in order. */
        const written: string[] = [];
        const queue = createSerialRequestQueue({
          write: (frame: string,) => {
            written.push(frame,);
            return Promise.resolve();
          },
        },);
        queue.enqueue({ id: 1, produce: () => Promise.resolve(NO_FRAME,), },);

        await queue.idle();

        expect(written,).toEqual([],);
      },
    },),
  ],
},);

//endregion createSerialRequestQueue
