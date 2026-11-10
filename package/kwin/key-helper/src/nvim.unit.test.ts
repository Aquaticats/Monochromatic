/**
 Tests for Neovim listen-socket discovery.

 Neovim writes its default listen socket into `$XDG_RUNTIME_DIR`, so a session
 that overrides that variable must not send the daemon looking under
 `/run/user/<uid>`. Every case works in a disposable folder and restores the
 environment it changed through a disposer.

 @module
 */

import {
  mkdtemp,
  rm,
  utimes,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  findNewestNvimSocket,
  NoNvimSocketError,
  NoRuntimeDirectoryError,
  nvimSocketDirectory,
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
 Name of a file that is not a Neovim socket, newer than both sockets.
 */
const DECOY_NAME = 'not-a-nvim-socket';

/**
 Modification time, in seconds, of a socket every other socket is newer than.
 */
const OLDER_SOCKET_SECONDS = 1_000;

/**
 Modification time, in seconds, of the socket discovery must prefer.
 */
const NEWER_SOCKET_SECONDS = 2_000;

/**
 Per-user runtime root the fallback builds a path in.
 */
const PER_USER_RUNTIME_ROOT = '/run/user';

/**
 User id the fallback cases ask for, chosen so no real session owns it.
 */
const TEST_UID = 4_242;

/**
 An empty disposable folder that removes itself when disposed.
 
 @example
 ```ts
 await using folder = await makeFolder();
 await writeFile(join(folder.path, 'nvim.1.0'), '');
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
 @param content - Text to write, stands in for a socket Neovim left behind
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
      name: findNewestNvimSocket.name,
      children: [
        it({
          name: 'reads the socket directory XDG_RUNTIME_DIR names',
          fn: async () => {
            await using folder = await makeFolder();
            /** Socket file inside the disposable runtime directory. */
            const socketPath = join(
              folder.path,
              OLDER_SOCKET_NAME,
            );
            await makeAgedFile({
              path: socketPath,
              content: '',
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
            await makeAgedFile({
              path: join(
                folder.path,
                OLDER_SOCKET_NAME,
              ),
              content: '',
              secondsSinceEpoch: OLDER_SOCKET_SECONDS,
            });
            /** Path of the socket discovery must prefer. */
            const newerSocketPath = join(
              folder.path,
              NEWER_SOCKET_NAME,
            );
            await makeAgedFile({
              path: newerSocketPath,
              content: '',
              secondsSinceEpoch: NEWER_SOCKET_SECONDS,
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
            await makeAgedFile({
              path: socketPath,
              content: '',
              secondsSinceEpoch: OLDER_SOCKET_SECONDS,
            });
            await makeAgedFile({
              path: join(
                folder.path,
                DECOY_NAME,
              ),
              content: '',
              secondsSinceEpoch: NEWER_SOCKET_SECONDS,
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
  ],
});
