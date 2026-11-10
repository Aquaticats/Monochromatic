/**
 TCP localhost IPC for morph-compact.
 
 Creates a one-shot TCP server on 127.0.0.1 that writes compressed
 context to the first client connection, then closes. Used as the
 final fallback when both argv and file tiers fail (e.g. read-only
 `/tmp`).
 
 @module
 */

import {
  type AddressInfo,
  createConnection as createTcpConnection,
  createServer as createTcpServer,
  type Server,
} from 'node:net';

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Constants

/**
 Milliseconds before the server auto-closes if no client connects.
 */
const SERVER_IDLE_TIMEOUT_MS = 30_000;

/**
 Milliseconds before a client read attempt times out.
 */
const CLIENT_READ_TIMEOUT_MS = 10_000;

/**
 Loopback address (no remote access).
 */
const LOCALHOST = '127.0.0.1';

//endregion

//region Address guard

/**
 Decide whether the value a listening server reported is a bound TCP address.

 Node types `Server.address()` as `AddressInfo | string | null`: a server bound
 to a Unix socket path reports the path as a string, and a server that stopped
 listening reports null. Only an `AddressInfo` carries the port to hand the
 child session.

 @param value - Value returned by `Server.address()`

 @returns Whether value is a bound TCP address
 */
function isTcpAddress(value: unknown,): value is AddressInfo {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('address' in value)
    && ((typeof value.address) === 'string')
    && ('port' in value)
    && ((typeof value.port) === 'number');
}

/**
 Describe an unexpected listening address so a failure message names it.

 @param value - Value returned by `Server.address()`

 @returns Text naming the reported value and what kind of value it is
 */
function reportedAddressText(value: unknown,): string {
  if ((value === undefined) || (value === null))
    return 'no address';
  if ((typeof value) === 'string')
    return `the Unix socket path ${value}`;
  return `the value ${JSON.stringify(value,)}`;
}

//endregion

//region Types

/**
 Result of creating a one-shot TCP server.
 */
export type OneShotTcpServerResult = {
  /**
   Host:port string (e.g. "127.0.0.1:43210").
   */
  address: string;
  /**
   Closes the server.
   */
  cleanup: () => void;
};

//endregion

//region Server

/**
 Create a one-shot TCP server on localhost.
 
 Binds to `127.0.0.1:0` (OS picks a free port), waits for the
 server to start listening, then returns the address. When a
 client connects, the entire `text` is written to that single
 connection, and the server closes.
 
 A 30-second idle timeout auto-cleans if no client connects.
 
 @param text - the compressed context string to serve
 
 @returns the host:port address and a cleanup function
 
 @throws when the TCP server cannot be created or bound
 
 @example
 ```typescript
 const { address, cleanup } = await createOneShotTcpServer(compressedText);
 // pass address to new pi session via --morph-compact-tcp
 ```
 */
export async function createOneShotTcpServer(
  text: string,
): Promise<OneShotTcpServerResult> {
  /**
   Lazily assigned server and idle-timer handles, held in a mutable record so
   the net.Server event handlers (handleConnection, handleError, close, idle
   timer) can share and clear them without a function-root `let` or a nullish
   union. Absent property means "not created / already cleared".
   */
  const handles: {
    server?: Server;
    idleTimer?: ReturnType<typeof setTimeout>;
  } = {};
  /**
   Guards against re-entry when multiple clients race to connect.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-function-root-let -- single-shot latch for racing client connections
  let served = false;

  /**
   Close the server and clear idle timer.
   */
  function close(): void {
    if (handles.idleTimer
      !== undefined) {
      clearTimeout(handles.idleTimer,);
      delete handles.idleTimer;
    }
    if (handles.server
      !== undefined) {
      handles.server
        .close();
      delete handles.server;
    }
  }

  // Wait for the server to start listening before returning
  /* oxlint-disable eslint-plugin-promise/avoid-new -- wrapping callback-based server.listen requires manual Promise construction */
  /**
   Host:port string resolved once the listening callback observes a valid AddressInfo.
   */
  const address = await new Promise<string>(
    function awaitListening(
      resolve,
      reject,
    ): void {
      handles.server = createTcpServer(
        /**
         Serves text through accepted host socket.
         
         @param socket - Accepted TCP socket.
         
         @mutates socket - `socket.write` and `socket.end` advance and close stream state.
         */
        function handleConnection(
          socket,
        ): void {
          if (served)
            return;
          served = true;

          if (handles.idleTimer
            !== undefined) {
            clearTimeout(handles.idleTimer,);
            delete handles.idleTimer;
          }

          socket.write(
            text,
            'utf8',
          );
          socket.end();

          // Close server after writing (one-shot)
          queueMicrotask(close,);
        },
      );

      handles.server
        .on(
        'error',
        /**
         Rejects listening operation with host error.
         
         @param err - TCP server failure retained as rejection reason.
         
         @mutates err - Promise rejection retains error as its rejection reason.
         */
        function handleError(
          err: Error,
        ): void {
          close();
          reject(err,);
        },
        );

      // Bind to localhost with OS-assigned port
      handles.server
        .listen(
        0,
        LOCALHOST,
        function onListening(): void {
          /**
           AddressInfo from the bound socket; resolved into host:port for callers.
           */
          const addrInfo = handles.server
            ?.address();
          if (isTcpAddress(addrInfo,)) {
            resolve(`${addrInfo.address}:${addrInfo.port}`,);
            return;
          }
          // An address the child cannot dial is a failed launch, not a pending one.
          close();
          reject(
            new Error(
              'createOneShotTcpServer: the listening server reported '
                + `${reportedAddressText(addrInfo,)} where a TCP address was expected`,
            ),
          );
        },
        );

      // Auto-cleanup after idle timeout
      handles.idleTimer = setTimeout(
        close,
        SERVER_IDLE_TIMEOUT_MS,
      );
    },
  );
  /* oxlint-enable eslint-plugin-promise/avoid-new */

  return {
    address,
    cleanup: close,
  };
}

//endregion

//region Client

/* oxlint-disable eslint-plugin-promise/avoid-new -- wrapping callback-based net.createConnection requires manual Promise construction */
/**
 Read compressed context from a TCP localhost server.
 
 Connects to the one-shot server, reads all data until the
 server closes the connection, and returns the text.
 
 A 10-second timeout prevents hanging if the server never responds.
 
 @param address - host:port string (e.g. "127.0.0.1:43210")
 
 @returns the compressed context string
 
 @throws when the TCP connection fails or reading times out
 
 @example
 ```typescript
 const text = await readFromTcpSocket('127.0.0.1:43210');
 // text contains the Morph-compressed conversation context
 ```
 */
export function readFromTcpSocket(
  address: string,
): Promise<string> {
  return new Promise(
    function promiseRead(
      resolve,
      reject,
    ): void {
      /**
       Host and stringified port split out of the address.
       */
      const [host, portStr,] = address.split(':',);
      /**
       Numeric port forwarded to createTcpConnection.
       */
      const port = Number(portStr,);
      /**
       Captured data buffers concatenated when the server ends the stream.
       */
      const chunks: Buffer[] = [];
      /**
       Latch ensuring resolve/reject is called exactly once.
       */
      // oxlint-disable-next-line no-restricted-syntax/no-function-root-let -- single-shot latch shared between data/end/error handlers and the timeout
      let settled = false;

      /**
       Handles the read owns, held in a mutable record so a settled read can
       disarm its timeout: a scheduled timer keeps the event loop alive.
       */
      const handles: {
        readTimeout?: ReturnType<typeof setTimeout>;
      } = {};

      /**
       Settle the promise with either an error or a result.
       
       @param settlement - Error or result selected by one socket lifecycle event.
       
       @mutates settlement - Promise rejection retains `settlement.error` as its rejection reason.
       */
      function finish(settlement: {
        readonly error?: Error;
        readonly result?: string;
      },): void {
        /**
         Settlement fields read after naming effect boundary.
         */
        const {
          error,
          result,
        } = settlement;
        if (settled)
          return;
        settled = true;

        if (handles.readTimeout
          !== undefined) {
          clearTimeout(handles.readTimeout,);
          delete handles.readTimeout;
        }

        if (error !== undefined)
          reject(error,);
        else
          resolve(result ?? '',);
      }

      /**
       Outbound TCP connection whose event handlers feed the promise lifecycle.
       */
      const socket = createTcpConnection(
        {
          host,
          port,
        },
        function onConnect(): void {
          socket.on(
            'data',
            function onData(
              chunk: Buffer,
            ): void {
              chunks.push(chunk,);
            },
          );
          socket.on(
            'end',
            function onEnd(): void {
              finish({
                result: Buffer.concat(chunks,)
                  .toString('utf8',),
              },);
            },
          );
          socket.on(
            'error',
            /**
             Settles read with host socket failure.
             
             @param err - Host-owned socket error.
             
             @mutates err - Promise rejection retains error as its rejection reason.
             */
            function onError(
              err: ForeignBorrowed<Error>,
            ): void {
              finish({ error: err, },);
            },
          );
        },
      );

      socket.on(
        'error',
        /**
         Settles connection with host socket failure.
         
         @param err - Host-owned connection error.
         
         @mutates err - Promise rejection retains error as its rejection reason.
         */
        function onError(
          err: ForeignBorrowed<Error>,
        ): void {
          finish({ error: err, },);
        },
      );

      // Read timeout, disarmed by finish() once the read settles
      handles.readTimeout = setTimeout(
        function onTimeout(): void {
          socket.destroy();
          finish({
            error: new Error(
              `readFromTcpSocket: timed out after ${CLIENT_READ_TIMEOUT_MS}ms`,
            ),
          },);
        },
        CLIENT_READ_TIMEOUT_MS,
      );
    },
  );
}
// oxlint-enable eslint-plugin-promise/avoid-new

//endregion
