/**
 Tests for Neovim listen-socket discovery and the input sender.

 Neovim writes its default listen socket into `$XDG_RUNTIME_DIR`, so a session
 that overrides that variable must not send the daemon looking under
 `/run/user/<uid>`. A Neovim that dies without cleaning up leaves its socket
 file behind, and that file refuses connections while remaining newer than the
 live Neovim's socket, so the sender must fall through to the candidates that
 still accept.

 Every socket in these cases is real: a listening stand-in speaks just enough
 msgpack-rpc to answer `nvim_input`, and a leftover socket is made by renaming
 a bound socket out from under its server before closing it. All of it lives in
 a disposable folder, and the environment a case changes is restored through a
 disposer.

 @module
 */

import { once } from 'node:events';
import {
  mkdtemp,
  rename,
  rm,
  utimes,
  writeFile,
} from 'node:fs/promises';
import {
  connect,
  createServer,
} from 'node:net';
import { join, } from 'node:path';

import { encode, decodeMulti, } from '@msgpack/msgpack';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  findNewestNvimSocket,
  findNvimSockets,
  NoNvimSocketError,
  NoRuntimeDirectoryError,
  nvimSocketDirectory,
  sendNvimInput,
} from '../dist/final/node/testing.mjs';

/**
 Prefix Neovim gives its listen-socket filenames.
 */
const SOCKET_PREFIX = 'nvim.';

/**
 Socket filename used by the cases, in Neovim's `nvim.<pid>.<seq>` shape.
 */
const OLDER_SOCKET_NAME = `${SOCKET_PREFIX}111.0`;

/**
 A second socket filename, newer than the first.
 */
const NEWER_SOCKET_NAME = `${SOCKET_PREFIX}222.0`;

/**
 A third socket filename, newest of the three.
 */
const NEWEST_SOCKET_NAME = `${SOCKET_PREFIX}333.0`;

/**
 Name of a file that is not a Neovim socket, newer than both sockets.
 */
const DECOY_NAME = 'not-a-nvim-socket';

/**
 Modification time, in seconds, of a socket every other socket is newer than.
 */
const OLDER_SOCKET_SECONDS = 1_000;

/**
 Modification time, in seconds, of the middle socket.
 */
const MIDDLE_SOCKET_SECONDS = 2_000;

/**
 Modification time, in seconds, of the socket discovery must prefer last.
 */
const NEWEST_SOCKET_SECONDS = 3_000;

/**
 Per-user runtime root the fallback builds a path in.
 */
const PER_USER_RUNTIME_ROOT = '/run/user';

/**
 User id the fallback cases ask for, chosen so no real session owns it.
 */
const TEST_UID = 4_242;

/**
 msgpack-rpc message type for a request, which is what the daemon sends.
 */
const RPC_REQUEST = 0;

/**
 msgpack-rpc message type for a response, which the stand-in sends.
 */
const RPC_RESPONSE = 1;

/**
 The method the double-shift and Ctrl+F4 paths call.
 */
const NVIM_INPUT_METHOD = 'nvim_input';

/**
 Message id the stand-in answers when a frame does not decode to one.
 */
const UNREADABLE_MESSAGE_ID = 0;

/**
 An empty disposable folder that removes itself when disposed.
 
 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
type ScratchFolder = {
  /**
   Absolute path of the folder.
   */
  readonly path: string;
  /**
   Remove the folder and everything inside it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 A listening stand-in Neovim that records the requests it received.
 
 @example
 ```ts
 await using fake = await startFakeNvim({ directory, name: OLDER_SOCKET_NAME });
 ```
 */
type FakeNvim = {
  /**
   Absolute path of the socket it listens on.
   */
  readonly socketPath: string;
  /**
   Decoded RPC frames it received, in arrival order.
   */
  readonly requests: readonly (readonly unknown[])[];
  /**
   Stop listening and remove the socket file.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Create an empty disposable folder.
 
 @returns The folder and its own removal
 
 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<ScratchFolder> {
  /**
   Absolute path of the new folder.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    'key-helper-nvim-',
  ));
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        path,
        { recursive: true, force: true, },
      );
    },
  };
}

/**
 Create one file with a chosen modification time.
 
 @param path - Absolute path of the file
 @param content - Text to write, stands in for a file that is not a socket
 @param secondsSinceEpoch - Modification time
 
 @example
 ```ts
 await makeAgedFile({ path, content: '', secondsSinceEpoch: 1000 });
 ```
 */
async function makeAgedFile({
  path,
  content,
  secondsSinceEpoch,
}: {
  readonly path: string;
  readonly content: string;
  readonly secondsSinceEpoch: number;
}): Promise<void> {
  await writeFile(
    path,
    content,
    'utf8',
  );
  await utimes(
    path,
    secondsSinceEpoch,
    secondsSinceEpoch,
  );
}

/**
 Leave one bound-but-dead Neovim socket file behind at a chosen path.

 The server binds a sibling path, the file is renamed into place, and only then
 is the server closed, so the socket file survives the listener that used to
 answer on it. That is the state a Neovim killed without cleanup leaves, and
 connecting to it fails with ECONNREFUSED.
 
 @param path - Absolute path the stale socket must have
 @param secondsSinceEpoch - Modification time
 
 @example
 ```ts
 await makeStaleSocket({ path, secondsSinceEpoch: 2000 });
 ```
 */
async function makeStaleSocket({
  path,
  secondsSinceEpoch,
}: {
  readonly path: string;
  readonly secondsSinceEpoch: number;
}): Promise<void> {
  /**
   Path the server binds before the file is moved into place.
   */
  const bindingPath = `${path}.binding`;
  /**
   Server whose socket file becomes the leftover.
   */
  const server = createServer();
  server.listen(bindingPath);
  await once(
    server,
    'listening',
  );
  await rename(
    bindingPath,
    path,
  );
  server.close();
  await once(
    server,
    'close',
  );
  await utimes(
    path,
    secondsSinceEpoch,
    secondsSinceEpoch,
  );
}

/**
 Read the message id out of a decoded RPC frame.
 
 @param message - Decoded frame of unknown shape
 @returns Its message id, or {@link UNREADABLE_MESSAGE_ID} when it has none
 
 @example
 ```ts
 messageIdOf({ message: [0, 7, 'nvim_input', ['<F20>']] }); // 7
 ```
 */
function messageIdOf({ message }: { readonly message: unknown; }): number {
  if (!Array.isArray(message)) {
    return UNREADABLE_MESSAGE_ID;
  }
  /**
   The frame, one element per RPC field.
   */
  const frame: readonly unknown[] = message;
  /**
   Second element of an RPC frame, the message id.
   */
  const [, msgId,] = frame;
  return (typeof msgId) === 'number' ? msgId : UNREADABLE_MESSAGE_ID;
}

/**
 Start a stand-in Neovim that answers every `nvim_input` request.
 
 @param directory - Folder the socket is created in
 @param name - Socket filename, in Neovim's `nvim.<pid>.<seq>` shape
 
 @returns The listening stand-in and its own shutdown
 
 @example
 ```ts
 await using fake = await startFakeNvim({ directory, name: OLDER_SOCKET_NAME });
 ```
 */
async function startFakeNvim({
  directory,
  name,
}: {
  readonly directory: string;
  readonly name: string;
}): Promise<FakeNvim> {
  /**
   Absolute path of the socket the stand-in listens on.
   */
  const socketPath = join(
    directory,
    name,
  );
  /**
   Frames the stand-in received, filled as connections arrive.
   */
  const requests: (readonly unknown[])[] = [];
  /**
   The stand-in itself.
   */
  const server = createServer();
  server.on(
    'connection',
    function onConnection(connection): void {
      connection.on(
        'data',
        function onData(chunk: Buffer): void {
          for (const message of decodeMulti(chunk)) {
            requests.push(Array.isArray(message) ? message : [message,]);
            connection.write(
              Buffer.from(encode([
                RPC_RESPONSE,
                messageIdOf({ message, }),
                null,
                null,
              ],)),
            );
          }
        },
      );
    },
  );
  server.listen(socketPath);
  await once(
    server,
    'listening',
  );
  return {
    socketPath,
    requests,
    async [Symbol.asyncDispose](): Promise<void> {
      server.close();
      await once(
        server,
        'close',
      );
    },
  };
}

/**
 `XDG_RUNTIME_DIR` pointed at a folder for one case, restored when disposed.
 
 @param value - Directory the variable names for this case
 
 @returns A disposer that puts the variable back
 
 @example
 ```ts
 using _runtimeDirectory = overrideRuntimeDirectory({ value: folder.path });
 ```
 */
function overrideRuntimeDirectory({ value }: { readonly value: string; }): {
  readonly [Symbol.dispose]: () => void;
} {
  /**
   The variable's value before this case, or `undefined` when it was unset.
   */
  const original = process.env.XDG_RUNTIME_DIR;
  process.env.XDG_RUNTIME_DIR = value;
  return {
    [Symbol.dispose](): void {
      if (original === undefined) {
        delete process.env.XDG_RUNTIME_DIR;
        return;
      }
      process.env.XDG_RUNTIME_DIR = original;
    },
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: nvimSocketDirectory.name,
      children: [
        it({
          name: 'uses XDG_RUNTIME_DIR when the session sets it',
          fn: async () => {
            expect(nvimSocketDirectory({
              xdgRuntimeDirectory: `${PER_USER_RUNTIME_ROOT}/1000/overridden`,
              perUserRuntimeDirectory: `${PER_USER_RUNTIME_ROOT}/${String(TEST_UID)}`,
            })).toBe(`${PER_USER_RUNTIME_ROOT}/1000/overridden`);
          },
        }),
        it({
          name: 'falls back to the per-user runtime directory without XDG_RUNTIME_DIR',
          fn: async () => {
            expect(nvimSocketDirectory({
              xdgRuntimeDirectory: '',
              perUserRuntimeDirectory: `${PER_USER_RUNTIME_ROOT}/${String(TEST_UID)}`,
            })).toBe(`${PER_USER_RUNTIME_ROOT}/${String(TEST_UID)}`);
          },
        }),
        it({
          name: 'throws NoRuntimeDirectoryError when neither source names one',
          fn: async () => {
            let caught: unknown;
            try {
              nvimSocketDirectory({ xdgRuntimeDirectory: '', perUserRuntimeDirectory: '', });
            } catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(NoRuntimeDirectoryError);
          },
        }),
      ],
    }),
    describe({
      name: findNvimSockets.name,
      children: [
        it({
          name: 'returns every socket newest first',
          fn: async () => {
            await using folder = await makeFolder();
            /** Path of the oldest socket. */
            const olderSocketPath = join(
              folder.path,
              OLDER_SOCKET_NAME,
            );
            /** Path of the middle socket. */
            const middleSocketPath = join(
              folder.path,
              NEWER_SOCKET_NAME,
            );
            /** Path of the newest socket. */
            const newestSocketPath = join(
              folder.path,
              NEWEST_SOCKET_NAME,
            );
            await makeStaleSocket({
              path: middleSocketPath,
              secondsSinceEpoch: MIDDLE_SOCKET_SECONDS,
            });
            await makeStaleSocket({
              path: olderSocketPath,
              secondsSinceEpoch: OLDER_SOCKET_SECONDS,
            });
            await makeStaleSocket({
              path: newestSocketPath,
              secondsSinceEpoch: NEWEST_SOCKET_SECONDS,
            });
            expect(await findNvimSockets({ directory: folder.path, })).toEqual([
              newestSocketPath,
              middleSocketPath,
              olderSocketPath,
            ]);
          },
        }),
        it({
          name: 'omits a file that is not a socket',
          fn: async () => {
            await using folder = await makeFolder();
            /** Path of the only socket. */
            const socketPath = join(
              folder.path,
              OLDER_SOCKET_NAME,
            );
            await makeStaleSocket({
              path: socketPath,
              secondsSinceEpoch: OLDER_SOCKET_SECONDS,
            });
            await makeAgedFile({
              path: join(
                folder.path,
                DECOY_NAME,
              ),
              content: '',
              secondsSinceEpoch: NEWEST_SOCKET_SECONDS,
            });
            expect(await findNvimSockets({ directory: folder.path, })).toEqual([socketPath]);
          },
        }),
        it({
          name: 'returns nothing when the directory holds no socket',
          fn: async () => {
            await using folder = await makeFolder();
            expect(await findNvimSockets({ directory: folder.path, })).toEqual([]);
          },
        }),
      ],
    }),
    describe({
      name: findNewestNvimSocket.name,
      children: [
        it({
          name: 'reads the socket directory XDG_RUNTIME_DIR names',
          fn: async () => {
            await using folder = await makeFolder();
            /** Socket inside the disposable runtime directory. */
            const socketPath = join(
              folder.path,
              OLDER_SOCKET_NAME,
            );
            await makeStaleSocket({
              path: socketPath,
              secondsSinceEpoch: OLDER_SOCKET_SECONDS,
            });
            using _runtimeDirectory = overrideRuntimeDirectory({ value: folder.path, });
            expect(await findNewestNvimSocket()).toBe(socketPath);
          },
        }),
        it({
          name: 'picks the newest socket in the directory',
          fn: async () => {
            await using folder = await makeFolder();
            await makeStaleSocket({
              path: join(
                folder.path,
                OLDER_SOCKET_NAME,
              ),
              secondsSinceEpoch: OLDER_SOCKET_SECONDS,
            });
            /** Path of the socket discovery must prefer. */
            const newerSocketPath = join(
              folder.path,
              NEWER_SOCKET_NAME,
            );
            await makeStaleSocket({
              path: newerSocketPath,
              secondsSinceEpoch: MIDDLE_SOCKET_SECONDS,
            });
            expect(await findNewestNvimSocket({ directory: folder.path, }))
              .toBe(newerSocketPath);
          },
        }),
        it({
          name: 'ignores a newer file that is not a Neovim socket',
          fn: async () => {
            await using folder = await makeFolder();
            /** Path of the only real socket. */
            const socketPath = join(
              folder.path,
              OLDER_SOCKET_NAME,
            );
            await makeStaleSocket({
              path: socketPath,
              secondsSinceEpoch: OLDER_SOCKET_SECONDS,
            });
            await makeAgedFile({
              path: join(
                folder.path,
                DECOY_NAME,
              ),
              content: '',
              secondsSinceEpoch: NEWEST_SOCKET_SECONDS,
            });
            expect(await findNewestNvimSocket({ directory: folder.path, })).toBe(socketPath);
          },
        }),
        it({
          name: 'throws NoNvimSocketError when the directory holds none',
          fn: async () => {
            await using folder = await makeFolder();
            let caught: unknown;
            try {
              await findNewestNvimSocket({ directory: folder.path, });
            } catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(NoNvimSocketError);
          },
        }),
      ],
    }),
    describe({
      name: sendNvimInput.name,
      children: [
        it({
          name: 'delivers input to the live socket when a newer socket is stale',
          fn: async () => {
            await using folder = await makeFolder();
            await using fake = await startFakeNvim({
              directory: folder.path,
              name: OLDER_SOCKET_NAME,
            });
            // The live socket must be the older one, or the stale socket could
            // not shadow it and the case would pass whichever candidate the
            // sender picked.
            await utimes(
              fake.socketPath,
              OLDER_SOCKET_SECONDS,
              OLDER_SOCKET_SECONDS,
            );
            await makeStaleSocket({
              path: join(
                folder.path,
                NEWER_SOCKET_NAME,
              ),
              secondsSinceEpoch: NEWEST_SOCKET_SECONDS,
            });
            using _runtimeDirectory = overrideRuntimeDirectory({ value: folder.path, });
            expect(await sendNvimInput('<F20>')).toBe(true);
            expect(fake.requests).toHaveLength(1);
            /**
             The one frame the stand-in received.
             */
            const [request] = fake.requests;
            expect(request?.[0]).toBe(RPC_REQUEST);
            expect(request?.[2]).toBe(NVIM_INPUT_METHOD);
            expect(request?.[3]).toEqual(['<F20>']);
          },
        }),
        it({
          name: 'returns false when every candidate refuses the connection',
          fn: async () => {
            await using folder = await makeFolder();
            /** Path of the older leftover socket. */
            const olderSocketPath = join(
              folder.path,
              OLDER_SOCKET_NAME,
            );
            /** Path of the newer leftover socket. */
            const newerSocketPath = join(
              folder.path,
              NEWER_SOCKET_NAME,
            );
            await makeStaleSocket({
              path: olderSocketPath,
              secondsSinceEpoch: OLDER_SOCKET_SECONDS,
            });
            await makeStaleSocket({
              path: newerSocketPath,
              secondsSinceEpoch: MIDDLE_SOCKET_SECONDS,
            });
            using _runtimeDirectory = overrideRuntimeDirectory({ value: folder.path, });
            expect(await sendNvimInput('<F20>')).toBe(false);
          },
        }),
        it({
          name: 'returns false when the directory holds no socket',
          fn: async () => {
            await using folder = await makeFolder();
            using _runtimeDirectory = overrideRuntimeDirectory({ value: folder.path, });
            expect(await sendNvimInput('<F20>')).toBe(false);
          },
        }),
      ],
    }),
  ],
});
