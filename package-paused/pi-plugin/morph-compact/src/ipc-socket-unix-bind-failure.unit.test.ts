/**
 Tests that the one-shot Unix domain socket server reports a bind failure
 instead of discarding it ({@link createOneShotSocketServer}).

 A bind failure arrives on the server's asynchronous `error` event, so the
 failure can only leave the function through a rejected promise. While it was
 discarded, `launchWithLargeContext` could not fall through to its TCP tier and
 launched a child session pointed at a socket path nobody served.

 This suite runs in its own file because it points `XDG_RUNTIME_DIR` at an
 unwritable directory, and a suite in another file would otherwise bind its own
 socket into that directory while the variable is set.

 @module
 */

import {
  chmod,
  mkdir,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { createOneShotSocketServer, } from './ipc-socket-unix.ts';

/**
 Payload the guarded server would serve; it never reaches a client.
 */
const PROBE_TEXT = 'bind-failure probe';

/**
 Permission bits letting this process enter a directory while creating nothing
 in it, which is what makes `bind` fail with `EACCES`.
 */
const UNWRITABLE_DIRECTORY_MODE = 0o500;

/**
 Fixture directory name kept short: a Unix socket path longer than the platform
 limit fails with `EINVAL` before the permission check runs.
 */
const FIXTURE_DIRECTORY_NAME = 'bind-failure';

/**
 Permission bits restored before the fixture directory is removed.
 */
const OWNER_DIRECTORY_MODE = 0o700;

/**
 Directory holding the fixture, taken from the per-user runtime directory the
 server binds under; the system temporary directory is never used.
 */
const runtimeDirectory = process.env['XDG_RUNTIME_DIR'];

/**
 Why this suite cannot run here, or `false` when it can.
 */
const skipReason = (runtimeDirectory === undefined) || (runtimeDirectory === '')
  ? 'needs XDG_RUNTIME_DIR'
  : (process.getuid?.() === 0)
    ? 'file permissions do not stop root from binding'
    : false;

await describe({
  name: '',
  children: [
    describe({
      name: createOneShotSocketServer.name,
      children: [
        it({
          name: 'rejects when the socket file cannot be created',
          skip: skipReason,
          timeout: 15_000,
          fn: async () => {
            const fixtureDirectory = join(runtimeDirectory ?? '', FIXTURE_DIRECTORY_NAME,);
            await mkdir(fixtureDirectory, { recursive: true, },);
            await chmod(fixtureDirectory, UNWRITABLE_DIRECTORY_MODE,);

            process.env['XDG_RUNTIME_DIR'] = fixtureDirectory;
            try {
              await expect(createOneShotSocketServer(PROBE_TEXT,),)
                .rejects
                .toThrow('EACCES',);
            }
            finally {
              process.env['XDG_RUNTIME_DIR'] = runtimeDirectory;
              await chmod(fixtureDirectory, OWNER_DIRECTORY_MODE,);
              await rm(fixtureDirectory, { recursive: true, force: true, },);
            }
          },
        },),
      ],
    },),
  ],
},);
