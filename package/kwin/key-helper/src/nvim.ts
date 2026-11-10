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
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { encode } from '@msgpack/msgpack';

import {
  NoNvimSocketError,
  NoRuntimeDirectoryError,
} from './errors.ts';

//region Logging

/**
 Logger for this module, re-tagged with the enclosing function name at each
 site.
 */
const nvimLogger = tagged({ tag: 'kwin-key-helper-nvim', },);

//endregion Logging

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
 Name of the environment variable Neovim reads the runtime directory from.
 */
const RUNTIME_DIRECTORY_VARIABLE = 'XDG_RUNTIME_DIR';

/**
 Directory a per-user runtime directory sits in, which systemd-logind manages.
 */
const PER_USER_RUNTIME_ROOT = '/run/user';

/**
 Directory Neovim writes its default listen socket into.
 
 Neovim uses `$XDG_RUNTIME_DIR` for that (neovim/neovim#3517), so a session that
 points the variable somewhere else, as a non-systemd login or a container does,
 keeps its sockets outside `/run/user/<uid>`. The per-user runtime directory is
 the fallback only when the variable is absent.

 Both candidates arrive as arguments rather than being read here, so the
 precedence between them is testable without changing the test process's
 environment or stubbing its user id.
 
 @param xdgRuntimeDirectory - Value of `$XDG_RUNTIME_DIR`, or the empty string when unset
 
 @param perUserRuntimeDirectory - Per-user runtime directory, or the empty string when the runtime names none
 
 @returns Absolute path of the directory to look for listen sockets in
 
 @throws {@link NoRuntimeDirectoryError} when neither candidate names one
 
 @example
 ```ts
 nvimSocketDirectory({ xdgRuntimeDirectory: '/run/user/1000', perUserRuntimeDirectory: '' });
 ```
 */
export function nvimSocketDirectory({
  xdgRuntimeDirectory,
  perUserRuntimeDirectory,
}: {
  readonly xdgRuntimeDirectory: string;
  readonly perUserRuntimeDirectory: string;
}): string {
  if (xdgRuntimeDirectory !== '') {
    return xdgRuntimeDirectory;
  }
  if (perUserRuntimeDirectory !== '') {
    return perUserRuntimeDirectory;
  }
  throw new NoRuntimeDirectoryError();
}

/**
 The runtime directory this session names, read per call.
 
 Reading it per call rather than at import keeps a daemon that started before
 the variable was set able to find the session's sockets.
 
 @returns Absolute path of the directory to look for listen sockets in
 
 @throws {@link NoRuntimeDirectoryError} when neither source names one
 
 @example
 ```ts
 sessionSocketDirectory();
 ```
 */
function sessionSocketDirectory(): string {
  /**
   User id the runtime reports, or `undefined` when it reports none.
   */
  const uid = process.getuid?.();
  return nvimSocketDirectory({
    xdgRuntimeDirectory: process.env[RUNTIME_DIRECTORY_VARIABLE] ?? '',
    perUserRuntimeDirectory: uid === undefined ? '' : `${PER_USER_RUNTIME_ROOT}/${String(uid)}`,
  });
}

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
 
 @param directory - Runtime directory to scan, resolved from the session by default
 
 @returns Absolute path of the newest Neovim listen socket
 
 @throws {@link NoNvimSocketError} when no Neovim socket is present
 
 @throws {@link NoRuntimeDirectoryError} when no runtime directory can be named
 
 @example
 ```ts
 const sock = await findNewestNvimSocket();
 ```
 */
export async function findNewestNvimSocket({
  directory = sessionSocketDirectory(),
}: {
  readonly directory?: string;
} = {}): Promise<string> {
  /**
   All entries in the runtime directory.
   */
  const entries = await readdir(directory);
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
      const full = `${directory}/${entry}`;
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
        tagged({
          tag: findNewestNvimSocket.name,
          l: nvimLogger,
        },)
          .debug(`skipping nvim socket ${full}: ${message}`,);
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
    tagged({
      tag: sendNvimInput.name,
      l: nvimLogger,
    },)
      .error(`nvim send failed: ${message}`,);
    return false;
  }
}
