import { once, } from 'node:events';
import {
  createConnection,
  type Socket,
} from 'node:net';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { HelperProtocolError, } from './helper-protocol.ts';
import type { HelperRequest, } from './helper-request.ts';

//region Request lease

/**
 Parent must explicitly accept the request before a detached editor can open.
 */
export const HELPER_READY = 'ready\n';

/**
 Tagged logger for connection lifetime, never request tokens or answer contents.
 */
const l = tagged({ tag: 'ask-user-question:helper-connection', },);

/**
 Parent disappeared or revoked this request before the helper could finish.
 */
export class AnswerRequestEndedError extends Error {
  /**
   Preserves disconnect evidence without producing an unhandled socket error.

   @param cause - request channel failure or closure
   */
  constructor(cause: unknown,) {
    super(
      'This question is no longer active. Return to Pi for the current question.',
      { cause, },
    );
    this.name = 'AnswerRequestEndedError';
  }
}

/**
 Request lifetime shared by authentication, editor execution, and completion.
 */
export type HelperConnection = Disposable & {
  readonly socket: Socket;
  readonly signal: AbortSignal;
};

/**
 Opens a request channel and installs disconnect handling before any asynchronous work.

 @param request - private endpoint and authentication token

 @returns owned channel with cancellation tied to peer lifetime

 @example
 ```ts
 using connection = createHelperConnection(request);
 ```
 */
export function createHelperConnection(request: HelperRequest,): HelperConnection {
  /**
   Function-scoped lifecycle diagnostics.
   */
  const rl = tagged({
    tag: createHelperConnection.name,
    l,
  },);
  /**
   One cancellation source shared with the attached editor.
   */
  const controller = new AbortController();
  /**
   Socket retained until completion or explicit disposal.
   */
  const socket = createConnection({
    host: request.host,
    port: request.port,
  },);
  socket.setEncoding('utf8',);
  socket.setNoDelay(true,);
  socket.on(
    'error',
    function onConnectionError(error: Error,): void {
      rl.debug(`answer channel failed: ${String(error,)}`,);
      controller.abort(new AnswerRequestEndedError(error,),);
    },
  );
  socket.once(
    'close',
    function onConnectionClosed(): void {
      rl.debug('answer channel closed',);
      controller.abort(new AnswerRequestEndedError('request channel closed',),);
    },
  );
  return {
    socket,
    signal: controller.signal,
    [Symbol.dispose](): void {
      socket.destroy();
    },
  };
}

/**
 Waits for explicit acceptance rather than assuming a sent token was accepted.

 @param connection - owned helper channel

 @param token - private request credential

 @throws when the requester disconnects or sends an unexpected acknowledgement

 @example
 ```ts
 await authenticateHelper({ connection, token: request.token });
 ```
 */
export async function authenticateHelper({
  connection,
  token,
}: {
  readonly connection: HelperConnection;
  readonly token: string;
},): Promise<void> {
  /**
   Function-scoped authentication diagnostics.
   */
  const rl = tagged({
    tag: authenticateHelper.name,
    l,
  },);
  /**
   Request-owned channel capabilities.
   */
  const {
    socket,
    signal,
  } = connection;
  await once(
    socket,
    'connect',
    { signal, },
  );
  signal.throwIfAborted();
  socket.write(`${token}\n`,);
  /**
   Bounded acknowledgement accumulator cannot exceed the fixed protocol frame.
   */
  const state = { acknowledgement: '', };
  for await (const chunk of socket.iterator({ destroyOnReturn: false, },)) {
    state.acknowledgement += String(chunk,);
    if (!HELPER_READY.startsWith(state.acknowledgement,))
      throw new HelperProtocolError('Answer requester sent an invalid startup acknowledgement.',);
    if (state.acknowledgement === HELPER_READY) {
      signal.throwIfAborted();
      rl.debug('requester accepted answer helper',);
      socket.resume();
      return;
    }
  }
  throw new AnswerRequestEndedError('requester closed before acknowledging the helper',);
}

//endregion Request lease
