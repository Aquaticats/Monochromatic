/**
 Behaviour of the backup copy: where a backup lands, what it keeps, and what it
 refuses to replace.
 
 Fixtures live under the harness scratch root, never the system temporary
 directory, and each test removes its own through `await using`.
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdir,
  mkdtempDisposable,
  readFile,
  readdir,
  stat,
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
  backupFolderName,
  backupPath,
  copyIntoBackup,
} from '../dist/final/node/index.mjs';

//region Fixtures
/**
 Moment that names every backup folder in this file, so an expected path is
 stated once instead of depending on when the suite runs.
 */
const FIXED_NOW_TEXT = '2026-05-14T06:17:11.030Z';

/**
 The moment above as a `Date`, handed to every call under test.
 */
const fixedNow = new Date(FIXED_NOW_TEXT,);

/**
 The folder name that moment must produce: the same ISO text with its colons
 removed.
 */
const FIXED_FOLDER_NAME = '2026-05-14T061711.030Z';

/**
 Creates the `bak` folder a backup root needs, which nothing creates for the
 caller.
 
 @param fixturePath - Folder the backup root is created in
 
 @returns Path of the empty backup root
 */
async function makeBackupRoot(fixturePath: string,): Promise<string> {
  const backupRoot = join(
    fixturePath,
    'bak',
  );
  await mkdir(backupRoot,);
  return backupRoot;
}

/**
 Reports why a test failed without asserting on the reason: used for a state the
 test reached by construction.
 
 @param reason - What the test expected to hold
 
 @throws Error carrying that reason, always
 */
function fail(reason: string,): never {
  throw new Error(reason,);
}
//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: backupFolderName.name,
      children: [
        it({
          name: 'removes every colon from the ISO timestamp so the name is one path segment',
          fn: async () => {
            expect(backupFolderName({ now: fixedNow, },),)
              .toBe(FIXED_FOLDER_NAME,);
          },
        },),
        it({
          name: 'keeps the date and time text around the removed colons',
          fn: async () => {
            const monthStart = new Date('2001-02-03T04:05:06.007Z',);
            expect(backupFolderName({ now: monthStart, },),)
              .toBe('2001-02-03T040506.007Z',);
          },
        },),
      ],
    },),
    describe({
      name: backupPath.name,
      children: [
        it({
          name: 'copies a file into the timestamped folder under its own name and preserves its timestamps',
          fn: async () => {
            await using fixture = await mkdtempDisposable(join(
              scratchRoot(),
              'backup-path-file-',
            ),);
            const fixturePath = fixture.path;
            const backupRoot = await makeBackupRoot(fixturePath,);
            const sourcePath = join(
              fixturePath,
              'note.txt',
            );
            await writeFile(
              sourcePath,
              'content before the edit',
            );
            /**
             Modification and access times planted on the source, far from the
             run's own clock, so a timestamp taken at copy time cannot match
             them by accident.
             */
            const mtime = new Date('2021-03-04T05:06:07.000Z',);
            const atime = new Date('2021-04-05T06:07:08.000Z',);
            await utimes(
              sourcePath,
              atime,
              mtime,
            );

            const destination = await backupPath({
              sourcePath,
              backupRoot,
              now: fixedNow,
            },);

            const expectedDestination = join(
              backupRoot,
              FIXED_FOLDER_NAME,
              'note.txt',
            );
            expect(destination,)
              .toBe(expectedDestination,);
            // Stat before reading: reading the copy would move its atime.
            const copied = await stat(destination,);
            expect(copied.mtime.toISOString(),)
              .toBe(mtime.toISOString(),);
            expect(copied.atime.toISOString(),)
              .toBe(atime.toISOString(),);
            const copiedText = await readFile(
              destination,
              'utf8',
            );
            expect(copiedText,)
              .toBe('content before the edit',);
          },
        },),
        it({
          name: 'copies a directory tree into the timestamped folder',
          fn: async () => {
            await using fixture = await mkdtempDisposable(join(
              scratchRoot(),
              'backup-path-directory-',
            ),);
            const fixturePath = fixture.path;
            const backupRoot = await makeBackupRoot(fixturePath,);
            const sourcePath = join(
              fixturePath,
              'notes',
            );
            await mkdir(
              join(
                sourcePath,
                'inner',
              ),
              { recursive: true, },
            );
            await writeFile(
              join(
                sourcePath,
                'inner',
                'deeper.txt',
              ),
              'nested content',
            );

            const destination = await backupPath({
              sourcePath,
              backupRoot,
              now: fixedNow,
            },);

            const expectedDestination = join(
              backupRoot,
              FIXED_FOLDER_NAME,
              'notes',
            );
            expect(destination,)
              .toBe(expectedDestination,);
            const copiedText = await readFile(
              join(
                destination,
                'inner',
                'deeper.txt',
              ),
              'utf8',
            );
            expect(copiedText,)
              .toBe('nested content',);
          },
        },),
        it({
          name: 'leaves a missing backup root missing instead of creating it',
          fn: async () => {
            await using fixture = await mkdtempDisposable(join(
              scratchRoot(),
              'backup-path-missing-root-',
            ),);
            const fixturePath = fixture.path;
            const sourcePath = join(
              fixturePath,
              'note.txt',
            );
            await writeFile(
              sourcePath,
              'content',
            );
            const backupRoot = join(
              fixturePath,
              'no-such-bak',
            );

            const attempt = backupPath({
              sourcePath,
              backupRoot,
              now: fixedNow,
            },);
            await expect(attempt,)
              .rejects
              .toThrow('ENOENT',);
            const backupRootStatus = stat(backupRoot,);
            await expect(backupRootStatus,)
              .rejects
              .toThrow('ENOENT',);
          },
        },),
      ],
    },),
    describe({
      name: copyIntoBackup.name,
      children: [
        it({
          name: 'rejects a copy onto an existing destination and leaves the existing copy intact',
          fn: async () => {
            await using fixture = await mkdtempDisposable(join(
              scratchRoot(),
              'backup-path-collision-',
            ),);
            const fixturePath = fixture.path;
            const sourcePath = join(
              fixturePath,
              'note.txt',
            );
            await writeFile(
              sourcePath,
              'content from this run',
            );
            const destination = join(
              fixturePath,
              'bak',
              FIXED_FOLDER_NAME,
              'note.txt',
            );
            const destinationDirectory = join(
              fixturePath,
              'bak',
              FIXED_FOLDER_NAME,
            );
            await mkdir(
              destinationDirectory,
              { recursive: true, },
            );
            /**
             The copy an earlier run left behind, which this run must not
             replace.
             */
            const earlierCopy = 'content from the earlier run';
            await writeFile(
              destination,
              earlierCopy,
            );

            const attempt = copyIntoBackup({
              sourcePath,
              destination,
            },);
            await expect(attempt,)
              .rejects
              .toHaveProperty('code', 'ERR_FS_CP_EEXIST',);
            const survivingText = await readFile(
              destination,
              'utf8',
            );
            expect(survivingText,)
              .toBe(earlierCopy,);
          },
        },),
        it({
          name: 'rejects a directory copy onto an existing destination directory and copies nothing inside it',
          fn: async () => {
            await using fixture = await mkdtempDisposable(join(
              scratchRoot(),
              'backup-path-collision-directory-',
            ),);
            const fixturePath = fixture.path;
            const sourcePath = join(
              fixturePath,
              'notes',
            );
            await mkdir(
              sourcePath,
              { recursive: true, },
            );
            await writeFile(
              join(
                sourcePath,
                'deeper.txt',
              ),
              'content from this run',
            );
            const destination = join(
              fixturePath,
              'bak',
              FIXED_FOLDER_NAME,
              'notes',
            );
            await mkdir(
              destination,
              { recursive: true, },
            );

            const attempt = copyIntoBackup({
              sourcePath,
              destination,
            },);
            await expect(attempt,)
              .rejects
              .toHaveProperty('code', 'ERR_FS_CP_EEXIST',);
            const arrived = stat(join(
              destination,
              'deeper.txt',
            ),);
            await expect(arrived,)
              .rejects
              .toThrow('ENOENT',);
          },
        },),
      ],
    },),
    describe({
      name: 'CLI entrypoint',
      children: [
        it({
          name: 'backs up a positional path into ./bak under the current working directory',
          fn: async () => {
            await using fixture = await mkdtempDisposable(join(
              scratchRoot(),
              'backup-path-cli-',
            ),);
            const fixturePath = fixture.path;
            await makeBackupRoot(fixturePath,);
            const sourceName = 'note.txt';
            await writeFile(
              join(
                fixturePath,
                sourceName,
              ),
              'content before the edit',
            );

            const entrypoint = join(
              import.meta.dirname,
              '..',
              'dist',
              'final',
              'node',
              'index.mjs',
            );
            const child = spawn(
              process.execPath,
              [
                entrypoint,
                sourceName,
              ],
              {
                cwd: fixturePath,
                stdio: 'pipe',
              },
            );
            /**
             Everything the program printed, collected so a failure reports what
             it said rather than only how it exited.
             */
            let stdout = '';
            let stderr = '';
            child.stdout.setEncoding(
              'utf8',
            );
            child.stderr.setEncoding(
              'utf8',
            );
            child.stdout.on(
              'data',
              (chunk: string,) => {
                stdout += chunk;
              },
            );
            child.stderr.on(
              'data',
              (chunk: string,) => {
                stderr += chunk;
              },
            );
            // `close` rather than `exit`: it fires once both output streams are
            // drained, so the text read below is complete.
            await once(child, 'close',);

            expect(stderr,)
              .toBe('',);
            expect(child.exitCode,)
              .toBe(0,);
            expect(stdout,)
              .toBe(`Backing up ${sourceName}\n`,);

            const backupRoot = join(
              fixturePath,
              'bak',
            );
            const backupFolders = await readdir(backupRoot,);
            expect(backupFolders,)
              .toHaveLength(1,);
            const backupFolder = backupFolders[0]
              ?? fail('the run created no timestamped backup folder',);
            const cliDestination = join(
              backupRoot,
              backupFolder,
              sourceName,
            );
            const copiedText = await readFile(
              cliDestination,
              'utf8',
            );
            expect(copiedText,)
              .toBe('content before the edit',);
          },
        },),
      ],
    },),
  ],
},);
