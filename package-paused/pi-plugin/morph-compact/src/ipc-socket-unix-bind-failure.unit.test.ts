/**
 Tests that the one-shot Unix domain socket server reports a bind failure
 instead of discarding it ({@link createOneShotSocketServer}).

 A bind failure arrives on the server's asynchronous `error` event, so the
 failure can only leave the function through a rejected promise. While it was
 discarded, `launchWithLargeContext` could not fall through to its TCP tier and
 launched a child session pointed at a socket path nobody served.

 This suite runs in its own file because it points `XDG_RUNTIME_DIR` at a
 directory the fixture controls, and a suite in another file would bind its own
 socket into that directory while the variable is set. Its suites run one at a
 time for the same reason.

 @module
 */

import {
  chmod,
  mkdir,
  rm,
  writeFile,
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
 Permission bits restored before the fixture directory is removed.
 */
const OWNER_DIRECTORY_MODE = 0o700;

/**
 Fixture names kept short: a Unix socket path past the platform limit fails with
 `EINVAL` before the failure under test runs.
 */
const UNWRITABLE_DIRECTORY_NAME = 'bind-failure';

/**
 Regular file used as the parent of the socket path, so `bind` fails with
 `ENOTDIR`. This case needs no file permissions, so it also covers a run as
 root, where the unwritable-directory case cannot fail.
 */
const PARENT_FILE_NAME = 'pf';

/**
 Directory holding the fixtures, taken from the per-user runtime directory the
 server binds under; the system temporary directory is never used.
 */
const runtimeDirectory = process.env['XDG_RUNTIME_DIR'] ?? '';

if (runtimeDirectory === '') {
  // A skip here would let the defect come back unnoticed, and the package's
  // other socket suites already fail without this variable.
  throw new Error(
    'XDG_RUNTIME_DIR is not set, so the one-shot socket server cannot bind and this suite cannot prove bind failures are reported',
  );
}

/**
 Why the permission case cannot run here, or `false` when it can.
 */
const permissionCaseSkipReason = (process.getuid?.() === 0)
  ? 'file permissions do not stop root from binding'
  : false;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: createOneShotSocketServer.name,
      children: [
        it({
          name: 'rejects when the socket file cannot be created',
          skip: permissionCaseSkipReason,
          timeout: 15_000,
          fn: async () => {
            const fixtureDirectory = join(runtimeDirectory, UNWRITABLE_DIRECTORY_NAME,);
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
        it({
          name: 'rejects when the socket path has a file for a parent',
          timeout: 15_000,
          fn: async () => {
            const parentFilePath = join(runtimeDirectory, PARENT_FILE_NAME,);
            await writeFile(parentFilePath, '',);

            process.env['XDG_RUNTIME_DIR'] = join(parentFilePath, 'child',);
            try {
              await expect(createOneShotSocketServer(PROBE_TEXT,),)
                .rejects
                .toThrow('ENOTDIR',);
            }
            finally {
              process.env['XDG_RUNTIME_DIR'] = runtimeDirectory;
              await rm(parentFilePath, { force: true, },);
            }
          },
        },),
      ],
    },),
  ],
},);
