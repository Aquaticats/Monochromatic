import { randomUUID, } from 'node:crypto';
import {
  addAbortListener,
  once,
} from 'node:events';
import {
  createServer,
  type Server,
  type Socket,
} from 'node:net';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  acceptAuthenticatedSocket,
  type AuthenticatedSocket,
  MAX_PROTOCOL_BYTES,
} from './answer-channel-auth.ts';
import { HELPER_READY, } from './helper-connection.ts';
import {
  type HelperCompletion,
  HelperProtocolError,
  parseHelperCompletion,
} from './helper-protocol.ts';

//region Constants

/**
 IPv4 loopback keeps helper channel unreachable from network peers.
 */
const LOOPBACK_HOST = '127.0.0.1';

/**
 Non-aborting fallback used with {@link addAbortListener}.
 */
const NEVER_ABORT_SIGNAL = new AbortController().signal;

//endregion Constants

//region Logger

/**
 Tagged logger for answer-channel lifecycle.
 */
const l = tagged({ tag: 'ask-user-question:answer-channel', },);

//endregion Logger

//region Types

/**
 Authenticated one-shot channel awaited by Pi tool execution.
 */
export type AnswerChannel = AsyncDisposable & {
  /**
   Loopback host written to helper request file.
   */
  readonly host: string;
  /**
   Ephemeral listening port written to helper request file.
   */
  readonly port: number;
  /**
   Random authentication token written to private helper request file.
   */
  readonly token: string;
  /**
   Waits until helper submits,
   cancels,
   fails,
   or disconnects.
   */
  readonly wait: (options: { readonly signal?: AbortSignal; },) => Promise<HelperCompletion>;
};

//endregion Types

//region Public lifecycle

/**
 Opens a private one-shot loopback answer channel.
 
 @returns disposable endpoint and authenticated completion waiter
 
 @example
 ```ts
 await using channel = await createAnswerChannel();
 ```
 */
export async function createAnswerChannel(): Promise<AnswerChannel> {
  /**
   Listener accepting detached helper connection.
   */
  const server = createServer();
  /**
   Mutable accepted-socket slot used by disposal path.
   */
  const sockets = new Set<Socket>();
  server.on(
    'connection',
    function retainConnection(socket: Socket,): void {
      sockets.add(socket,);
      socket.on(
        'error',
        function logSocketError(error: Error,): void {
          l.debug(`answer connection failed: ${String(error,)}`,);
        },
      );
      socket.once(
        'close',
        function releaseConnection(): void {
          sockets.delete(socket,);
        },
      );
    },
  );
  server.listen({
    host: LOOPBACK_HOST,
    port: 0,
    exclusive: true,
  },);
  await once(
    server,
    'listening',
  );
  // Keep startup referenced until helper connects or caller cancels.
  // Detached terminal launch must not let an otherwise idle requester exit.
  /**
   Bound endpoint assigned by operating system.
   */
  const address = server.address();
  if ((address === null) || ((typeof address) === 'string'))
    throw new Error('Answer channel did not receive a TCP endpoint.',);
  /**
   Per-request token unavailable through process arguments.
   */
  const token = randomUUID();
  l.debug(`listening on ${LOOPBACK_HOST}:${String(address.port,)}`,);
  return {
    host: LOOPBACK_HOST,
    port: address.port,
    token,
    wait: async function waitForCompletion(
      { signal, }: { readonly signal?: AbortSignal; },
    ): Promise<HelperCompletion> {
      /**
       Authenticated helper socket and buffered completion prefix.
       */
      const authenticated = await acceptAuthenticatedSocket({
        server,
        token,
        ...(signal === undefined ? {} : { signal, }),
      },);
      if (signal !== undefined)
        signal.throwIfAborted();
      authenticated.socket
        .write(HELPER_READY,);
      for (const socket of sockets) {
        if (socket !== authenticated.socket)
          socket.destroy();
      }
      server.close();
      return readCompletion({
        authenticated,
        ...(signal === undefined ? {} : { signal, }),
      },);
    },
    async [Symbol.asyncDispose](): Promise<void> {
      // Includes unauthenticated and queued candidates, not just the accepted helper.
      for (const socket of sockets)
        socket.destroy();
      await closeListeningServer({ server, },);
      l.debug('closed answer channel',);
    },
  };
}

//endregion Public lifecycle

//region Completion

/**
 Reads one helper completion frame until authenticated socket closes.
 
 @param authenticated - helper socket positioned after authentication
 
 @param signal - optional tool cancellation signal
 
 @returns validated helper completion;
 empty frame means cancellation
 */
async function readCompletion(
  {
    authenticated,
    signal,
  }: {
    readonly authenticated: AuthenticatedSocket;
    readonly signal?: AbortSignal;
  },
): Promise<HelperCompletion> {
  /**
   Completion accumulation state seeded by authentication chunk remainder.
   */
  const state = {
    text: authenticated.remainder,
    bytes: Buffer.byteLength(
      authenticated.remainder,
      'utf8',
    ),
  };
  /**
   Subscription closing helper socket when tool aborts.
   */
  using abortSubscription = addAbortListener(
    signal ?? NEVER_ABORT_SIGNAL,
    function abortCompletion(): void {
      authenticated.socket
        .destroy();
    },
  );
  for await (const chunk of {
    [Symbol.asyncIterator](): AsyncIterator<string> {
      return authenticated.iterator;
    },
  }) {
    state.text += chunk;
    state.bytes += Buffer.byteLength(
      chunk,
      'utf8',
    );
    if (state.bytes > MAX_PROTOCOL_BYTES)
      throw new HelperProtocolError('Answer helper completion exceeded protocol byte limit.',);
  }
  if (signal !== undefined)
    signal.throwIfAborted();
  return parseHelperCompletion({ payload: state.text, },);
}

/**
 Closes listening server when still active.
 
 @param server - server owned by channel
 */
async function closeListeningServer(
  { server, }: { readonly server: Server; },
): Promise<void> {
  if (!server.listening)
    return;
  /**
   Close event registered before close call to avoid event race.
   */
  const closed = once(
    server,
    'close',
  );
  server.close();
  await closed;
}

//endregion Completion
