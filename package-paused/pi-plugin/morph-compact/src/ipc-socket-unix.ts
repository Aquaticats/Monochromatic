/**
 Unix domain socket IPC for morph-compact.
 
 Creates a one-shot server that writes compressed context to the
 first client connection, then closes. Used as a fallback when
 the temp-file tier fails (e.g. the tool's scratch directory is read-only).
 
 @module
 */

import { randomUUID, } from 'node:crypto';
import { unlink, } from 'node:fs/promises';
import {
  createConnection,
  createServer,
  type Server,
} from 'node:net';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Module logger

/**
 Module logger tagged for morph-compact unix-socket IPC.
 */
const l = tagged({ tag: 'morph-compact:ipc-socket-unix', },);

//endregion

//region Constants

/**
 Milliseconds before the server auto-closes if no client connects.
 */
const SERVER_IDLE_TIMEOUT_MS = 30_000;

/**
 Milliseconds before a client read attempt times out.
 */
const CLIENT_READ_TIMEOUT_MS = 10_000;

//endregion

//region Types

/**
 Result of creating a one-shot socket server.
 */
export type OneShotSocketServerResult = {
  /**
   Filesystem path of the Unix domain socket.
   */
  socketPath: string;
  /**
   Unlinks the socket file and closes the server.
   */
  cleanup: () => void;
};

//endregion

//region Server

/**
 Create a one-shot Unix domain socket server.
 
 Listens on a unique socket path under `XDG_RUNTIME_DIR`, whose path stays short enough for a Unix socket.
 When a client connects, the entire `text` is written to that
 single connection, the server closes, and the socket file is unlinked.
 
 A 30-second idle timeout auto-cleans if no client connects.
 
 @param text - the compressed context string to serve
 
 @returns the socket path and a cleanup function
 
 @throws when the per-user runtime directory is unset, or when the socket cannot be created or bound; a bind failure
 arrives on the server's asynchronous `error` event, so it is reported by rejecting the returned promise and carries
 the socket error
 
 @example
 ```typescript
 const { socketPath, cleanup } = await createOneShotSocketServer(compressedText);
 // pass socketPath to new pi session via --morph-compact-socket
 ```
 */
export async function createOneShotSocketServer(
  text: string,
): Promise<OneShotSocketServerResult> {
  /**
   Per-user runtime directory, the only place a short socket path is guaranteed; the system temporary directory is
   never used because its backing device is unknowable.
   */
  const runtimeDirectory = process.env['XDG_RUNTIME_DIR'];
  if (runtimeDirectory === undefined || runtimeDirectory === '') {
    throw new Error('XDG_RUNTIME_DIR is not set; the one-shot socket needs the per-user runtime directory',);
  }
  /**
   Unique socket path for this one-shot run.
   */
  const socketPath = join(
    runtimeDirectory,
    `morph-compact-${randomUUID()}.sock`,
  );

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
   Close the server and unlink the socket file.
   
   Stays synchronous so it satisfies its `queueMicrotask`, `setTimeout`,
   `EventEmitter.on`, and disposable-cleanup call sites (all void-return
   slots). The socket-file removal is async (`unlink`), so it runs as a
   detached, self-contained task via the `void (async () => {})()` idiom
   the workspace uses for async work in synchronous callback positions.
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
    void (async function unlinkSocketFile(): Promise<void> {
      try {
        await unlink(socketPath,);
      }
      catch (error: unknown) {
        // Socket file may already be removed or never created.
        tagged({
          tag: unlinkSocketFile.name,
          l,
        },)
          .debug(`Socket file unlink failed: ${String(error,)}`,);
      }
    })();
  }

  /**
   Server serving the one-shot payload to its first accepted client.
   */
  const server = createServer(
    /**
     Serves text through accepted host socket.
     
     @param socket - Accepted Unix socket.
     
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
  handles.server = server;

  /* oxlint-disable eslint-plugin-promise/avoid-new -- wrapping the server's asynchronous `listening` and `error` events requires manual Promise construction */
  /**
   Settles once the socket is bound and listening, and rejects with the bind
   failure `listen` reports on the server's `error` event. Without the
   rejection a bind failure would leave the caller launching a session against
   a socket path nobody serves.
   */
  const listening = new Promise<void>(
    function awaitListening(
      resolve,
      reject,
    ): void {
      server.once(
        'listening',
        function onListening(): void {
          resolve();
        },
      );
      server.once(
        'error',
        /**
         Rejects the bind with the host error that stopped it.
         
         @param err - Host-owned socket server failure retained as rejection reason.
         
         @mutates err - Promise rejection retains error as its rejection reason.
         */
        function onBindError(
          err: ForeignBorrowed<Error>,
        ): void {
          reject(err,);
        },
      );
    },
  );
  /* oxlint-enable eslint-plugin-promise/avoid-new */

  server.listen(socketPath,);
  try {
    await listening;
  }
  catch (error: unknown) {
    // The bind failed: release the handle and the socket file, then report it.
    close();
    throw error;
  }

  server.on(
    'error',
    /**
     Records a failure that arrives after the socket started listening, and
     releases the server; the bind failures that matter to callers already
     rejected the promise above.
     
     @param err - Host-owned socket server failure.
     */
    function handleServerError(
      err: ForeignBorrowed<Error>,
    ): void {
      tagged({
        tag: handleServerError.name,
        l,
      },)
        .debug(`Socket server failed after listening: ${String(err,)}`,);
      close();
    },
  );

  // Auto-cleanup after idle timeout
  handles.idleTimer = setTimeout(
    close,
    SERVER_IDLE_TIMEOUT_MS,
  );

  return {
    socketPath,
    cleanup: close,
  };
}

//endregion

//region Client

/* oxlint-disable eslint-plugin-promise/avoid-new -- wrapping callback-based net.createConnection requires manual Promise construction */
/**
 Read compressed context from a Unix domain socket.
 
 Connects to the one-shot server, reads all data until the
 server closes the connection, and returns the text.
 
 A 10-second timeout prevents hanging if the server never responds.
 
 @param socketPath - filesystem path of the Unix domain socket
 
 @returns the compressed context string
 
 @throws when the socket cannot be connected to or reading times out
 
 @example
 ```typescript
 const text = await readFromUnixSocket(socketPath);
 // text contains the Morph-compressed conversation context
 ```
 */
export function readFromUnixSocket(
  socketPath: string,
): Promise<string> {
  return new Promise(
    function promiseRead(
      resolve,
      reject,
    ): void {
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
       Outbound connection whose event handlers feed the promise lifecycle.
       */
      const socket = createConnection(
        { path: socketPath, },
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
              `readFromUnixSocket: timed out after ${CLIENT_READ_TIMEOUT_MS}ms`,
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
