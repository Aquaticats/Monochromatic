/**
 Tests for the ensure family through the built `./node` entry.

 Every case runs inside a disposable temporary directory; nothing touches
 the checkout. Permission cases are skipped for the superuser, whose
 access checks succeed whatever the mode bits say.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  access,
  chmod,
  constants,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  stat,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ensureDir,
  ensureFile,
  ensurePath,
} from '@monochromatic-dev/fs-path-core/node';

//region Fixture helpers

/**
 Disposable temporary directory.
 */
type TempDir = {
  /**
   Absolute path of the directory.
   */
  readonly path: string;

  /**
   Removes the directory tree after the test.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a disposable temporary directory.

 @returns directory that removes itself on dispose

 @example
 ```ts
 await using temp = await createTempDir();
 ```
 */
async function createTempDir(): Promise<TempDir> {
  /**
   Fresh directory under the system temporary root.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    'fs-path-ensure-',
  ),);
  return {
    path,
    [Symbol.asyncDispose](): Promise<void> {
      return rm(
        path,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 Runs a script in a child Node process started in one directory.

 The cases of a file run concurrently and share one working directory, so a
 case that needs a directory of its own asks a child process for it instead of
 moving the directory every other case resolves against.

 @param directory - Directory the child process starts in

 @param source - Module source the child runs

 @returns what the child wrote to its standard output

 @example
 ```ts
 const stdout = await runInDirectory({ directory: temp.path, source: 'console.log(1);' });
 ```
 */
async function runInDirectory({ directory, source, }: {
  readonly directory: string;
  readonly source: string;
},): Promise<string> {
  return await new Promise((resolve, reject,) => {
    execFile(
      process.execPath,
      [
        '--input-type=module',
        '-e',
        source,
      ],
      { cwd: directory, },
      (error, stdout, stderr,) => {
        if (error !== null) {
          reject(new Error(`child process in ${directory} failed: ${stderr}`,),);
          return;
        }
        resolve(stdout,);
      },
    );
  },);
}

/**
 Prefix a case's child process puts on the line carrying the message it caught,
 so logger output on the same stream cannot satisfy the assertion.
 */
const CAUGHT_MESSAGE_PREFIX = 'CAUGHT_MESSAGE:';

/**
 Built `./node` entry, resolved through the package specifier the cases import,
 so a child process runs the artifact under test rather than a rebuilt copy.
 */
const NODE_ENTRY_URL = import.meta.resolve('@monochromatic-dev/fs-path-core/node',);

/**
 Whether the current process is the superuser, for whom mode bits never
 deny access.
 */
const isSuperuser = ((typeof process.getuid) === 'function') && (process.getuid() === 0);

/**
 Mode with every permission bit cleared.
 */
const NO_PERMISSIONS = 0o000;

/**
 Mode granting the owner read, write, and traverse, which a permission case
 restores before the fixture is removed.
 */
const OWNER_DIRECTORY_MODE = 0o700;

/**
 Text a second writer puts into the file while `ensureFile` guarantees it.

 A non-empty body is what makes a truncating create visible: the guarantee
 under test is that this body survives the call.
 */
const CONCURRENT_CONTENT = 'written by the second writer while ensureFile ran';

/**
 Rounds of the concurrent-writer case.

 One round is enough to decide the fixed call, which never truncates, and the
 unbounded orderings are why a round repeats: the defect this case covers
 truncated the second writer's file in every round measured on the host that
 filed it, so a handful of rounds is the reproduction, not a sample.
 */
const CONCURRENT_ROUNDS = 20;

/**
 Runs an operation and returns what it threw, or `undefined` when it did not.

 @param operation - operation expected to throw

 @returns caught value

 @example
 ```ts
 const caught = await caughtFrom(() => ensureDir(filePath));
 ```
 */
async function caughtFrom(operation: () => Promise<unknown>,): Promise<unknown> {
  try {
    await operation();
    return undefined;
  }
  catch (error: unknown) {
    return error;
  }
}

//endregion Fixture helpers

await describe({
  name: '',
  children: [
    describe({
      name: ensureDir.name,
      children: [
        it({
          name: 'creates a missing directory tree and returns the path',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             Path two levels below the fixture, none of which exists yet.
             */
            const target = join(
              temp.path,
              'a',
              'b',
            );
            expect(await ensureDir(target,),).toBe(target,);
            expect((await stat(target,)).isDirectory(),).toBe(true,);
          },
        },),
        it({
          name: 'returns an existing accessible directory untouched',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             Directory created ahead of the call.
             */
            const target = join(
              temp.path,
              'existing',
            );
            await mkdir(target,);
            await writeFile(
              join(
                target,
                'keep.txt',
              ),
              'kept',
            );
            expect(await ensureDir(target,),).toBe(target,);
            expect(await readFile(
              join(
                target,
                'keep.txt',
              ),
              'utf8',
            ),).toBe('kept',);
          },
        },),
        it({
          name: 'throws when the path exists as a file',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             Regular file occupying the requested path.
             */
            const target = join(
              temp.path,
              'file.txt',
            );
            await writeFile(
              target,
              '',
            );
            /**
             Value thrown by the call.
             */
            const caught = await caughtFrom(() => ensureDir(target,),);
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toContain('not a directory',);
          },
        },),
        it({
          name: 'restores owner read and write access on an inaccessible directory',
          skip: isSuperuser ? 'the superuser passes every access check' : false,
          fn: async () => {
            await using temp = await createTempDir();
            /**
             Directory whose permissions are stripped before the call.
             */
            const target = join(
              temp.path,
              'locked',
            );
            await mkdir(target,);
            await chmod(
              target,
              NO_PERMISSIONS,
            );
            expect(await caughtFrom(() => access(
              target,
              constants.R_OK | constants.W_OK,
            ),),).toBeInstanceOf(Error,);
            expect(await ensureDir(target,),).toBe(target,);
            expect(await caughtFrom(() => access(
              target,
              constants.R_OK | constants.W_OK,
            ),),).toBeUndefined();
          },
        },),
      ],
    },),

    describe({
      name: ensureFile.name,
      children: [
        it({
          name: 'creates a missing file with its parent directories',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             File two directories below the fixture.
             */
            const target = join(
              temp.path,
              'nested',
              'deeper',
              'config.json',
            );
            expect(await ensureFile(target,),).toBe(target,);
            expect((await stat(target,)).isFile(),).toBe(true,);
            expect(await readFile(
              target,
              'utf8',
            ),).toBe('',);
          },
        },),
        it({
          name: 'creates a bare file name in the current directory',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             What the child process reported after ensuring the bare name.
             */
            const stdout = await runInDirectory({
              directory: temp.path,
              source: `const { ensureFile } = await import(${JSON.stringify(NODE_ENTRY_URL,)});
const created = await ensureFile('bare.txt');
console.log('CREATED:' + created);`,
            },);
            expect(stdout,).toContain('CREATED:bare.txt',);
            expect((await stat(join(
              temp.path,
              'bare.txt',
            ),)).isFile(),).toBe(true,);
          },
        },),
        it({
          name: 'keeps the content of a file a second writer creates during the call',
          fn: async () => {
            await using temp = await createTempDir();
            await Promise.all(Array.from(
              { length: CONCURRENT_ROUNDS, },
              async function runRound(_unusedValue: unknown, round: number,): Promise<void> {
                /**
                 Parent directory neither writer has created yet, so `ensureFile` has to create it too.
                 */
                const parent = join(
                  temp.path,
                  `race-${round}`,
                );
                /**
                 File the second writer creates between the existence check and the create.
                 */
                const target = join(
                  parent,
                  'appeared.txt',
                );
                /**
                 Second writer, started in the same tick so its create lands inside the call.
                 */
                const secondWriter = (async function writeContent(): Promise<void> {
                  await mkdir(
                    parent,
                    { recursive: true, },
                  );
                  await writeFile(
                    target,
                    CONCURRENT_CONTENT,
                  );
                })();
                await Promise.all([
                  secondWriter,
                  ensureFile(target,),
                ],);
                expect(await readFile(
                  target,
                  'utf8',
                ),).toBe(CONCURRENT_CONTENT,);
              },
            ),);
          },
        },),
        it({
          name: 'creates the target of a symbolic link whose target is absent',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             Link standing where the file is requested.
             */
            const linkPath = join(
              temp.path,
              'link.txt',
            );
            /**
             Path the link names but nothing occupies yet.
             */
            const linkTargetPath = join(
              temp.path,
              'link-target.txt',
            );
            await symlink(
              linkTargetPath,
              linkPath,
            );
            expect(await ensureFile(linkPath,),).toBe(linkPath,);
            expect((await stat(linkTargetPath,)).isFile(),).toBe(true,);
          },
        },),
        it({
          name: 'creates a file in a directory whose permissions deny access',
          skip: isSuperuser ? 'the superuser passes every access check' : false,
          fn: async () => {
            await using temp = await createTempDir();
            /**
             Directory whose permissions are stripped before the call.
             */
            const parent = join(
              temp.path,
              'locked',
            );
            await mkdir(parent,);
            await chmod(
              parent,
              NO_PERMISSIONS,
            );
            /**
             File inside the inaccessible directory, none of which exists yet.
             */
            const target = join(
              parent,
              'app.log',
            );
            expect(await ensureFile(target,),).toBe(target,);
            expect((await stat(target,)).isFile(),).toBe(true,);
          },
        },),
        it({
          name: 'names the denied file rather than a parent a bare name has none of',
          skip: isSuperuser ? 'the superuser passes every access check' : false,
          fn: async () => {
            await using temp = await createTempDir();
            /**
             Directory the bare name resolves against, which the child denies itself and restores.
             */
            const locked = join(
              temp.path,
              'locked',
            );
            await mkdir(locked,);
            /**
             What the child process reported after the denied create, which is nothing when it resolved.
             */
            const stdout = await runInDirectory({
              directory: locked,
              source: `const { chmod } = await import('node:fs/promises');
const { ensureFile } = await import(${JSON.stringify(NODE_ENTRY_URL,)});
const [ deniedMode, restoredMode ] = ${JSON.stringify([ NO_PERMISSIONS, OWNER_DIRECTORY_MODE, ],)};
const directory = process.cwd();
await chmod(directory, deniedMode);
try {
  await ensureFile('denied.txt');
  console.log('RESOLVED');
}
catch (error) {
  console.log(${JSON.stringify(CAUGHT_MESSAGE_PREFIX,)} + error.message);
}
await chmod(directory, restoredMode);`,
            },);
            /**
             Lines the child wrote with the message it caught, split out of whatever the logger wrote to the same stream.
             */
            const caughtLines = stdout.split(
              '\n',
            ).filter((line,) => line.startsWith(CAUGHT_MESSAGE_PREFIX,));
            expect(caughtLines.length,).toBe(1,);
            expect(caughtLines[0],).toContain('denied.txt',);
          },
        },),
        it({
          name: 'keeps the content of an existing accessible file',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             File written ahead of the call.
             */
            const target = join(
              temp.path,
              'present.txt',
            );
            await writeFile(
              target,
              'content stays',
            );
            expect(await ensureFile(target,),).toBe(target,);
            expect(await readFile(
              target,
              'utf8',
            ),).toBe('content stays',);
          },
        },),
        it({
          name: 'refuses a directory addressed with a query suffix, naming the path it checked',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             Directory occupying the path the stripped spelling names.
             */
            const target = join(
              temp.path,
              'folder',
            );
            await mkdir(target,);
            /**
             Spelling a caller passes, suffix included.
             */
            const spelling = `${target}?raw`;
            /**
             Value thrown by the call.
             */
            const caught = await caughtFrom(() => ensureFile(spelling,),);
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toContain('not a file',);
            expect((caught as Error).message,).toContain(target,);
          },
        },),
        it({
          name: 'keeps the content of the file a symbolic link names',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             File the link names, holding content the call must leave alone.
             */
            const linkTargetPath = join(
              temp.path,
              'link-target.txt',
            );
            /**
             Link standing where the file is requested.
             */
            const linkPath = join(
              temp.path,
              'link.txt',
            );
            await writeFile(
              linkTargetPath,
              'content stays',
            );
            await symlink(
              linkTargetPath,
              linkPath,
            );
            expect(await ensureFile(linkPath,),).toBe(linkPath,);
            expect(await readFile(
              linkTargetPath,
              'utf8',
            ),).toBe('content stays',);
          },
        },),
        it({
          name: 'throws when the path exists as a directory',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             Directory occupying the requested path.
             */
            const target = join(
              temp.path,
              'dir.txt',
            );
            await mkdir(target,);
            /**
             Value thrown by the call.
             */
            const caught = await caughtFrom(() => ensureFile(target,),);
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toContain('not a file',);
          },
        },),
        it({
          name: 'restores owner read and write access on an inaccessible file',
          skip: isSuperuser ? 'the superuser passes every access check' : false,
          fn: async () => {
            await using temp = await createTempDir();
            /**
             File whose permissions are stripped before the call.
             */
            const target = join(
              temp.path,
              'locked.txt',
            );
            await writeFile(
              target,
              'secret',
            );
            await chmod(
              target,
              NO_PERMISSIONS,
            );
            expect(await caughtFrom(() => access(
              target,
              constants.R_OK | constants.W_OK,
            ),),).toBeInstanceOf(Error,);
            expect(await ensureFile(target,),).toBe(target,);
            expect(await readFile(
              target,
              'utf8',
            ),).toBe('secret',);
            expect(await caughtFrom(() => access(
              target,
              constants.R_OK | constants.W_OK,
            ),),).toBeUndefined();
          },
        },),
      ],
    },),

    describe({
      name: ensurePath.name,
      children: [
        it({
          name: 'treats a path with an extension as a file',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             Extension-bearing path, expected to become a file.
             */
            const target = join(
              temp.path,
              'logs',
              'app.log',
            );
            expect(await ensurePath(target,),).toBe(target,);
            expect((await stat(target,)).isFile(),).toBe(true,);
          },
        },),
        it({
          name: 'treats a path without an extension as a directory',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             Extension-less path, expected to become a directory.
             */
            const target = join(
              temp.path,
              'logs',
              'archive',
            );
            expect(await ensurePath(target,),).toBe(target,);
            expect((await stat(target,)).isDirectory(),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
