/**
 Neovim RPC bridge: locate the newest Neovim listen socket and send input to
 it via msgpack-rpc `nvim_input`.
 
 Used by the double-shift handler (F20) and by the KWin Ctrl+F4 remap for
 Neovide (F16). The newest socket is chosen because the user's active Neovim is
 almost always the most recently started one.
 
 @module
 */

import { once } from 'node:events';
import {
  readdir,
  stat,
} from 'node:fs/promises';
import { connect } from 'node:net';

import { wait } from '@monochromatic-dev/async-time-core';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { encode } from '@msgpack/msgpack';

import { NoNvimSocketError } from './errors.ts';

/**
 msgpack-rpc message type for a request.
 */
const RPC_REQUEST = 0;

/**
 Milliseconds to wait for an RPC reply before giving up on the send.
 */
const RPC_TIMEOUT_MS = 500;

/**
 Prefix of Neovim listen-socket filenames under the runtime dir.
 */
const NVIM_SOCK_PREFIX = 'nvim.';

/**
 UID fallback when `process.getuid` is unavailable and `$UID` is unset.
 */
const DEFAULT_UID = 1_000;

/**
 Current UID, used to locate the per-user runtime directory.
 */
const UID = process.getuid?.() ?? Number(process.env
  .UID
  ?? DEFAULT_UID);

/**
 Runtime directory Neovim writes its listen sockets into.
 */
const NVIM_SOCK_DIR = `/run/user/${UID}`;

/**
 Mutable holder for the monotonically increasing msgpack-rpc message id, a
 `const` object rather than a module-root `let` binding.
 */
const rpc: { msgId: number } = { msgId: 0 };

/**
 One discovered Neovim socket with its modification time.
 */
type SocketEntry = {
  /**
   Absolute socket path.
   */
  readonly path: string;
  /**
   Modification time in milliseconds, used to pick the newest.
   */
  readonly mtime: number;
};

/**
 Find the most recently modified Neovim listen socket.
 
 @returns Absolute path of the newest Neovim listen socket
 
 @throws {@link NoNvimSocketError} when no Neovim socket is present
 
 @example
 ```ts
 const sock = await findNewestNvimSocket();
 ```
 */
export async function findNewestNvimSocket(): Promise<string> {
  /**
   All entries in the per-user runtime directory.
   */
  const entries = await readdir(NVIM_SOCK_DIR);
  /**
   Entry names matching the Neovim socket prefix.
   */
  const candidates = entries.filter(function isNvimSocket(entry: string): boolean {
    return entry.startsWith(NVIM_SOCK_PREFIX);
  });
  /**
   Stat result per candidate, each a zero- or one-element array so a socket
   removed mid-scan simply drops out on flatten.
   */
  const stated = await Promise.all(
    candidates.map(async function statSocket(entry: string): Promise<readonly SocketEntry[]> {
      /**
       Absolute path of this candidate socket.
       */
      const full = `${NVIM_SOCK_DIR}/${entry}`;
      try {
        /**
         File info used to read the modification time.
         */
        const info = await stat(full);
        return [{
          path: full,
          mtime: info.mtimeMs
        }];
      } catch (error) {
        /**
         Message for a socket that vanished between readdir and stat.
         */
        const message = caughtValueText(error,);
        console.error(`[key-helper] skipping nvim socket ${full}: ${message}`);
        return [];
      }
    }),
  );
  /**
   Live sockets flattened from the per-candidate results.
   */
  const sockets = stated.flat();
  if (sockets.length === 0) {
    throw new NoNvimSocketError();
  }
  return sockets.reduce(function newer(
    best: SocketEntry,
    candidate: SocketEntry
  ): SocketEntry {
    return candidate.mtime > best.mtime ? candidate : best;
  })
    .path;
}

/**
 Send a key sequence to the newest Neovim over msgpack-rpc `nvim_input`.
 
 @param keys - Neovim key notation, e.g. `<F20>`, `<F16>`, `<Esc>`
 
 @returns Whether the input was written to a socket
 
 @example
 ```ts
 await sendNvimInput('<F20>');
 ```
 */
export async function sendNvimInput(keys: string): Promise<boolean> {
  try {
    /**
     Newest Neovim socket; throws when Neovim is not running.
     */
    const sockPath = await findNewestNvimSocket();
    /**
     Connection to the Neovim RPC socket.
     */
    const sock = connect(sockPath);
    /**
     Closes the socket when this scope exits, on success or failure.
     */
    using sockCleanup = {
      [Symbol.dispose](): void {
        sock.end();
      },
    };
    await once(
      sock,
      'connect'
    );
    rpc.msgId += 1;
    /**
     Encoded `nvim_input` request frame.
     */
    const message = encode([
      RPC_REQUEST,
      rpc.msgId,
      'nvim_input',
      [keys]
    ]);
    sock.write(Buffer.from(message));
    // Wait for a reply, but cap the wait so a silent Neovim cannot hang us.
    await Promise.race([
      once(
        sock,
        'data'
      ),
      wait(RPC_TIMEOUT_MS)
    ]);
    return true;
  } catch (error) {
    /**
     Message for a no-socket, connect, or write failure.
     */
    const message = caughtValueText(error,);
    console.error(`[key-helper] nvim send failed: ${message}`);
    return false;
  }
}
