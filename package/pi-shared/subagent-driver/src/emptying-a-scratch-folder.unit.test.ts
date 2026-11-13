/**
 Emptying one run's scratch folder: every entry deleted except `keep/`, every refusal named with its reason, and the lines a run's
 end carries.

 Every fixture here is a folder the test made under a root of its own, so no test reads or writes the machine's own scratch root.
 The adversarial cases are the ones a deletion must never get wrong: a link that points out of the folder, a link carrying the
 kept subfolder's name, a kept name that is a file, a name with spaces, newlines and a leading dash, a path that leaves the root,
 a folder that is a mount point, and a folder that cannot be read.

 @module
 */

import {
  chmod,
  lstat,
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
  stat,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { promisify, } from 'node:util';
import { execFile, } from 'node:child_process';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  ENTRIES_NAMED_IN_A_LINE,
  KEEP_FOLDER,
  type ScratchFolderEmptiness,
  emptyTheFolderItsRunStillOwns,
  emptyTheScratchFolderAtTheLowestDiskPriority,
  ownershipProblemOf,
  bytesTextOf,
  emptyTheFolderIn,
  leftWhereItIsIn,
  makeScratchFolder,
  mountPathsIn,
  scratchEmptyingLinesOf,
  scratchFolderOf,
} from '../dist/final/node/index.mjs';

/**
 The command that empties one folder, as a person or a launcher runs it.

 This is the built entry, so the test reaches the same wiring a run's end reaches rather than a function beside it.
 */
const EMPTIER = new URL('../dist/final/node/empty-the-scratch-folder-of-a-run.mjs', import.meta.url,).pathname;

/**
 `execFile` with a promise result, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);

/**
 A folder for one test, removed when the test ends.

 @returns the folder's path, and the disposer that removes it

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  const path = await mkdtemp(join(scratchRoot(), 'pi-scratch-emptying-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, {
        recursive: true,
        force: true,
      },);
    },
  };
}

/**
 One run's scratch folder, made by the launcher's own step under a root this test made.

 @param request - the folder the root lives in, and the run's name

 @returns the root, the folder and its `keep/` subfolder

 @example
 ```ts
 const scratch = await makeScratch({ base: folder.path, runName: 'a', });
 ```
 */
async function makeScratch({
  base,
  runName,
}: {
  readonly base: string;
  readonly runName: string;
}): Promise<{
  readonly scratchRoot: string;
  readonly folder: string;
  readonly keepFolder: string
}> {
  /**
   The root every run of this test lives under.
   */
  const root = join(
    base,
    'agent',
  );
  /**
   What making the folder answered.
   */
  const made = await makeScratchFolder({
    scratchRoot: root,
    runName,
  },);
  return {
    scratchRoot: root,
    folder: made.folder,
    keepFolder: made.keepFolder,
  };
}

/**
 Reads an answer as the folder that was emptied, so a test about what was removed does not carry a cast.

 @param emptying - what emptying the folder answered

 @returns the same answer, once it is known to be a folder that was emptied

 @throws Error when the folder was not emptied, which would make the test's own reading wrong

 @example
 ```ts
 const emptied = doneOf(await emptyWithoutTheMachine({ scratch, runName: 'a', }));
 ```
 */
function doneOf(emptying: ScratchFolderEmptiness,): Extract<ScratchFolderEmptiness, { emptied: 'done' }> {
  if (emptying.emptied !== 'done') {
    throw new Error(`expected a folder that was emptied, and the answer was "${emptying.emptied}"`,);
  }
  return emptying;
}

/**
 Everything a folder holds, by name, so a test says exactly what is left.

 @param folder - the folder to list

 @returns its entries' names, sorted

 @example
 ```ts
 const names = await namesInFolder('/home/a/temp/agent/a-scratch');
 ```
 */
async function namesInFolder(folder: string,): Promise<readonly string[]> {
  return (await readdir(folder,)).toSorted();
}

/**
 Empties one folder with the machine's own mounts and volume left out of it, which is what a test can decide itself.

 @param request - the folder, the root it must sit under, the run's name, and what a test decides about mounts and room

 @returns what emptying it answered

 @example
 ```ts
 const emptied = await emptyWithoutTheMachine({ scratch, runName: 'a', });
 ```
 */
async function emptyWithoutTheMachine({
  scratch,
  runName,
  leftBehindMounts = [],
  freeBytes = [1_000, 5_000],
}: {
  readonly scratch: { readonly scratchRoot: string; readonly folder: string };
  readonly runName: string;
  readonly leftBehindMounts?: readonly string[];
  readonly freeBytes?: readonly [number, number];
},): Promise<ScratchFolderEmptiness> {
  /**
   How many times the room was read, so the two readings answer differently.
   */
  const readings = { taken: 0, };
  return await emptyTheFolderIn({
    folder: scratch.folder,
    scratchRoot: scratch.scratchRoot,
    runName,
    mountsOf: async function mounts() {
      return leftBehindMounts;
    },
    freeBytesOf: async function room() {
      /**
       The reading this call answers with.
       */
      const answer = freeBytes[Math.min(
        readings.taken,
        freeBytes.length - 1,
      )];
      readings.taken += 1;
      return answer ?? 0;
    },
  },);
}

/**
 Puts a folder's mode back when the test ends, so a fixture the test made unreadable does not keep the disposer from removing it.

 @param path - the folder whose mode is put back

 @param mode - the mode to put back, which the folder had before the test took its permissions away

 @returns the folder's path, and the disposer that restores the mode

 @example
 ```ts
 await using restore = await restoredMode({ path: '/home/a/closed', mode: 0o700, },);
 ```
 */
async function restoredMode({
  path,
  mode,
}: {
  readonly path: string;
  readonly mode: number;
}): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  return {
    path,
    async [Symbol.asyncDispose]() {
      await chmod(path, mode,);
    },
  };
}

/**
 A process table in which one process works in one folder, which is how the proof reads the machine.

 @param request - the folder this test's process folders live in, and one process's working folder

 @returns the process folder

 @example
 ```ts
 const processFolder = await processTableWorkingIn({ base: folder.path, working: scratch.folder, });
 ```
 */
async function processTableWorkingIn({
  base,
  working,
}: {
  readonly base: string;
  readonly working: string;
}): Promise<string> {
  /**
   The folder a Linux machine keeps one entry per process in.
   */
  const processFolder = join(
    base,
    'proc',
  );
  await mkdir(processFolder, { recursive: true, },);
  await mkdir(join(
    processFolder,
    '7777',
  ), { recursive: true, },);
  await symlink(
    working,
    join(
      processFolder,
      '7777',
      'cwd',
    ),
  );
  return processFolder;
}

/**
 A folder holding a stand-in `systemctl`, so a test decides what the manager says about one unit without one being started.

 @param request - the folder the stand-in goes in, and the state it answers

 @returns the folder to put first on a `PATH`

 @example
 ```ts
 const bin = await managerSaying({ base: folder.path, state: 'active', });
 ```
 */
async function managerSaying({
  base,
  state,
}: {
  readonly base: string;
  readonly state: string;
}): Promise<string> {
  /**
   The folder the stand-in lives in.
   */
  const bin = join(
    base,
    'bin',
  );
  await mkdir(bin, { recursive: true, },);
  await writeFile(join(
    bin,
    'systemctl',
  ), `#!/bin/sh\nprintf '%s\\n' '${state}'\nexit 0\n`,);
  await chmod(join(
    bin,
    'systemctl',
  ), 0o700,);
  return bin;
}

await describe({
  name: '',
  children: [
    describe({
      name: emptyTheFolderIn.name,
      children: [
        it({
          name: 'deletes every entry except keep and keeps the folder with its kept subfolder inside it',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The run's scratch folder.
             */
            const scratch = await makeScratch({
              base: folder.path,
              runName: 'copy-route',
            },);
            await writeFile(join(
              scratch.folder,
              'log.txt',
            ), 'a log\n',);
            await mkdir(join(
              scratch.folder,
              'build',
              'deep',
            ), { recursive: true, },);
            await writeFile(join(
              scratch.folder,
              'build',
              'deep',
              'blob.bin',
            ), Buffer.alloc(
              1_000,
              7,
            ),);
            await writeFile(join(
              scratch.keepFolder,
              'report.md',
            ), 'kept\n',);
            /**
             What emptying it answered.
             */
            const emptied = await emptyWithoutTheMachine({
              scratch,
              runName: 'copy-route',
            },);
            expect(emptied).toMatchObject({
              emptied: 'done',
              removedFiles: 2,
              removedFolders: 2,
              removedBytes: 6 + 1_000,
              keepFiles: 1,
              keepBytes: 5,
              leftBehind: [],
            },);
            expect(await namesInFolder(scratch.folder,),).toEqual([KEEP_FOLDER,],);
            expect(await namesInFolder(scratch.keepFolder,),).toEqual(['report.md',],);
          },
        },),
        it({
          name: 'answers that there was no folder when the run ended before its launcher made one',
          fn: async () => {
            await using folder = await makeFolder();
            expect(await emptyTheFolderIn({
              folder: join(
                folder.path,
                'a-scratch',
              ),
              scratchRoot: folder.path,
              runName: 'a',
              mountsOf: async function mounts() {
                return [];
              },
              freeBytesOf: async function room() {
                return 0;
              },
            },),).toEqual({ emptied: 'no-folder', },);
          },
        },),
        it({
          name: 'refuses a folder that is not directly under the scratch root, and touches nothing outside the root',
          fn: async () => {
            await using folder = await makeFolder();
                        /**
             A folder one level down from the root, holding a file nobody may delete.
             */
            const outside = join(
              folder.path,
              'outside',
            );
            await mkdir(outside, { recursive: true, },);
            await writeFile(join(
              outside,
              'precious.txt',
            ), 'precious\n',);
            /**
             What emptying a folder that is not directly under the root answered.
             */
            const emptied = await emptyTheFolderIn({
              folder: join(
                outside,
                'a-scratch',
              ),
              scratchRoot: folder.path,
              runName: 'a',
              mountsOf: async function mounts() {
                return [];
              },
              freeBytesOf: async function room() {
                return 0;
              },
            },);
            expect(emptied.emptied,).toBe('refused',);
            expect((emptied as { readonly reason: string }).reason,).toContain(`is not directly under the scratch root`);
            expect(await readFile(join(
              outside,
              'precious.txt',
            ), 'utf8',),).toBe('precious\n',);
          },
        },),
        it({
          name: 'refuses a folder that is a symbolic link rather than following it out of the root',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder a link inside the root points at, which holds what nobody may lose.
             */
            const target = join(
              folder.path,
              'target',
            );
            await mkdir(target, { recursive: true, },);
            await writeFile(join(
              target,
              'precious.txt',
            ), 'precious\n',);
            /**
             The root, with a run's folder that is really a link to that target.
             */
            const root = join(
              folder.path,
              'agent',
            );
            await mkdir(root, { recursive: true, },);
            await symlink(
              target,
              join(
                root,
                'a-scratch',
              ),
            );
            /**
             What emptying it answered.
             */
            const emptied = await emptyTheFolderIn({
              folder: join(
                root,
                'a-scratch',
              ),
              scratchRoot: root,
              runName: 'a',
              mountsOf: async function mounts() {
                return [];
              },
              freeBytesOf: async function room() {
                return 0;
              },
            },);
            expect(emptied.emptied,).toBe('refused',);
            expect((emptied as { reason: string }).reason,).toContain('a symbolic link',);
            expect(await readFile(join(
              target,
              'precious.txt',
            ), 'utf8',),).toBe('precious\n',);
          },
        },),
        it({
          name: 'refuses a folder that is itself a mount point, and leaves everything inside it',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The run's scratch folder.
             */
            const scratch = await makeScratch({
              base: folder.path,
              runName: 'a',
            },);
            await writeFile(join(
              scratch.folder,
              'junk.txt',
            ), 'junk\n',);
            /**
             What emptying it answered when the kernel reports the folder itself as a mount.
             */
            const emptied = await emptyTheFolderIn({
              folder: scratch.folder,
              scratchRoot: scratch.scratchRoot,
              runName: 'a',
              mountsOf: async function mounts() {
                return [scratch.folder,];
              },
              freeBytesOf: async function room() {
                return 0;
              },
            },);
            expect(emptied.emptied,).toBe('refused',);
            expect((emptied as { reason: string }).reason,).toContain('it is a mount point',);
            expect(await namesInFolder(scratch.folder,),).toEqual([KEEP_FOLDER, 'junk.txt',].toSorted(),);
          },
        },),
        it({
          name: 'leaves a mount point inside the folder where it is, deletes the rest, and names that mount in the line',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The run's scratch folder.
             */
            const scratch = await makeScratch({
              base: folder.path,
              runName: 'a',
            },);
            /**
             The folder inside it that another filesystem stands at.
             */
            const mounted = join(
              scratch.folder,
              'another-filesystem',
            );
            await mkdir(mounted, { recursive: true, },);
            await writeFile(join(
              mounted,
              'someone-elses.txt',
            ), 'not yours\n',);
            await writeFile(join(
              scratch.folder,
              'junk.txt',
            ), 'junk\n',);
            /**
             What emptying it answered.
             */
            const emptied = await emptyWithoutTheMachine({
              scratch,
              runName: 'a',
              leftBehindMounts: [mounted,],
            },);
            expect(emptied.emptied,).toBe('done',);
            expect(doneOf(emptied,).leftBehind,).toHaveLength(1,);
            expect(await readFile(join(
              mounted,
              'someone-elses.txt',
            ), 'utf8',),).toBe('not yours\n',);
            expect(await namesInFolder(scratch.folder,),).toEqual([KEEP_FOLDER, 'another-filesystem',].toSorted(),);
            expect(scratchEmptyingLinesOf({
              runName: 'a',
              folder: scratch.folder,
              emptying: doneOf(emptied,),
            },).join('\n',),).toContain('it is a mount point',);
          },
        },),
        it({
          name: 'deletes a link that points out of the folder as a link, and leaves what it points at',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The run's scratch folder.
             */
            const scratch = await makeScratch({
              base: folder.path,
              runName: 'a',
            },);
            /**
             A folder and a file outside the run's scratch, which a link inside it names.
             */
            const outside = join(
              folder.path,
              'outside',
            );
            await mkdir(outside, { recursive: true, },);
            await writeFile(join(
              outside,
              'precious.txt',
            ), 'precious\n',);
            await symlink(
              outside,
              join(
                scratch.folder,
                'a-folder-link',
              ),
            );
            await symlink(
              join(
                outside,
                'precious.txt',
              ),
              join(
                scratch.folder,
                'a-file-link',
              ),
            );
            /**
             What emptying it answered.
             */
            const emptied = await emptyWithoutTheMachine({
              scratch,
              runName: 'a',
            },);
            expect(emptied).toMatchObject({
              emptied: 'done',
              removedFiles: 2,
              leftBehind: [],
            },);
            expect(await namesInFolder(scratch.folder,),).toEqual([KEEP_FOLDER,],);
            expect(await namesInFolder(outside,),).toEqual(['precious.txt',],);
          },
        },),
        it({
          name: 'leaves a link that carries the name of the kept subfolder as a link, and says so',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The run's scratch folder, without the kept subfolder a launcher makes: a link carries its name instead.
             */
            const root = join(
              folder.path,
              'agent',
            );
            /**
             The folder the link points at.
             */
            const outside = join(
              folder.path,
              'outside',
            );
            await mkdir(outside, { recursive: true, },);
            await writeFile(join(
              outside,
              'precious.txt',
            ), 'precious\n',);
            await mkdir(join(
              root,
              'a-scratch',
            ), { recursive: true, },);
            await symlink(
              outside,
              join(
                root,
                'a-scratch',
                KEEP_FOLDER,
              ),
            );
            await writeFile(join(
              root,
              'a-scratch',
              'junk.txt',
            ), 'junk\n',);
            /**
             What emptying it answered.
             */
            const emptied = await emptyWithoutTheMachine({
              scratch: {
                scratchRoot: root,
                folder: join(
                  root,
                  'a-scratch',
                ),
              },
              runName: 'a',
            },);
            expect(emptied.emptied,).toBe('done',);
            expect(doneOf(emptied,).leftBehind
              .map(function reasonOf(entry,) {
                return entry.reason;
              },)
              .join('\n',),).toContain('it is not a folder',);
            expect((await lstat(join(
              root,
              'a-scratch',
              KEEP_FOLDER,
            ))).isSymbolicLink(),).toBe(true,);
            expect(await namesInFolder(outside,),).toEqual(['precious.txt',],);
            expect(
              await namesInFolder(join(
                root,
                'a-scratch',
              ),),
            ).toEqual([KEEP_FOLDER,],);
          },
        },),
        it({
          name: 'leaves a kept name that is a file where it is, rather than deleting the one place a report would go',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The root, with a run's folder whose kept name is a file.
             */
            const root = join(
              folder.path,
              'agent',
            );
            await mkdir(join(
              root,
              'a-scratch',
            ), { recursive: true, },);
            await writeFile(join(
              root,
              'a-scratch',
              KEEP_FOLDER,
            ), 'not a folder\n',);
            await writeFile(join(
              root,
              'a-scratch',
              'junk.txt',
            ), 'junk\n',);
            /**
             What emptying it answered.
             */
            const emptied = await emptyWithoutTheMachine({
              scratch: {
                scratchRoot: root,
                folder: join(
                  root,
                  'a-scratch',
                ),
              },
              runName: 'a',
            },);
            expect(emptied).toMatchObject({
              emptied: 'done',
              keepFiles: 0,
              keepBytes: 0,
            },);
            expect(
              await namesInFolder(join(
                root,
                'a-scratch',
              ),),
            ).toEqual([KEEP_FOLDER,],);
            expect(await readFile(join(
              root,
              'a-scratch',
              KEEP_FOLDER,
            ), 'utf8',),).toBe('not a folder\n',);
          },
        },),
        it({
          name: 'empties the folder of a run whose name holds spaces, a newline and a leading dash',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The three names a shell would read differently, each one run's own.
             */
            const names = ['a name with spaces', 'a name\nwith a newline', '--a-leading-dash',];
            /* oxlint-disable eslint/no-await-in-loop -- one folder at a time, each made, filled, emptied and read before the next
               one, so a failure names the run name it belongs to; see doc/lint/sequential-awaits.md. */
            for (const runName of names) {
              /**
               The run's scratch folder.
               */
              const scratch = await makeScratch({
                base: folder.path,
                runName,
              },);
              expect(scratch.folder,).toBe(join(
                scratch.scratchRoot,
                `${runName}-scratch`,
              ),);
              await writeFile(join(
                scratch.folder,
                'junk.txt',
              ), 'junk\n',);
              expect(await emptyWithoutTheMachine({
                scratch,
                runName,
              },),).toMatchObject({
                emptied: 'done',
                removedFiles: 1,
              },);
              expect(await namesInFolder(scratch.folder,),).toEqual([KEEP_FOLDER,],);
            }
            /* oxlint-enable eslint/no-await-in-loop */
          },
        },),
        it({
          name: 'empties what it can of a folder holding a subfolder it may not read, and names that subfolder and why',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The run's scratch folder.
             */
            const scratch = await makeScratch({
              base: folder.path,
              runName: 'a',
            },);
            /**
             The subfolder no account, this one included, may read: a fixture left at mode zero, like `closed` under the fixture
             of `pi-runs/kimi-findings-lean-resume-and-progress-save-findings` on this machine.
             */
            const closed = join(
              scratch.folder,
              'closed',
            );
            await mkdir(closed, { recursive: true, },);
            await writeFile(join(
              closed,
              'a-fixture.txt',
            ), 'a fixture\n',);
            await writeFile(join(
              scratch.folder,
              'junk.txt',
            ), 'junk\n',);
            await chmod(closed, 0o000,);
            await using restore = await restoredMode({ path: closed, mode: 0o700, },);
            {
              /**
               What emptying it answered.
               */
              const emptied = await emptyWithoutTheMachine({
                scratch,
                runName: 'a',
              },);
              expect(emptied.emptied,).toBe('done',);
              expect(doneOf(emptied,).leftBehind
                .map(function pathOf(entry,) {
                  return [
                    entry.path,
                    entry.reason
                      .includes('EACCES',)
                      || entry.reason
                        .includes('ENOTEMPTY',),
                  ];
                },),).toEqual([
                  [closed, true,],
                  [closed, true,],
                ],);
              // The file it could remove is gone, and the folder it could not read is still there: the deletion finishes when it
              // meets what it cannot remove.
              expect(await namesInFolder(scratch.folder,),).toEqual([KEEP_FOLDER, 'closed',].toSorted(),);
            }
          },
        },),
        it({
          name: 'finishes when it meets an entry it may not delete, naming the entry and the folder it could not remove',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The run's scratch folder.
             */
            const scratch = await makeScratch({
              base: folder.path,
              runName: 'a',
            },);
            /**
             A subfolder this account may read and may not write, with a file inside it: the file cannot be deleted and the folder
             cannot be removed.
             */
            const sealed = join(
              scratch.folder,
              'sealed',
            );
            await mkdir(sealed, { recursive: true, },);
            await writeFile(join(
              sealed,
              'inside.txt',
            ), 'inside\n',);
            await writeFile(join(
              scratch.folder,
              'junk.txt',
            ), 'junk\n',);
            await chmod(sealed, 0o500,);
            await using restore = await restoredMode({ path: sealed, mode: 0o700, },);
            {
              /**
               What emptying it answered.
               */
              const emptied = await emptyWithoutTheMachine({
                scratch,
                runName: 'a',
              },);
              expect(emptied.emptied,).toBe('done',);
              expect(doneOf(emptied,).leftBehind
                .map(function pathOf(entry,) {
                  return entry.path;
                },),).toEqual([join(
                  sealed,
                  'inside.txt',
                ), sealed,],);
              expect(await namesInFolder(scratch.folder,),).toEqual([KEEP_FOLDER, 'sealed',].toSorted(),);
            }
          },
        },),
        it({
          name: 'says the room the volume gained from the two readings, and says so instead when it could not be read',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The run's scratch folder.
             */
            const scratch = await makeScratch({
              base: folder.path,
              runName: 'a',
            },);
            expect(await emptyWithoutTheMachine({
              scratch,
              runName: 'a',
              freeBytes: [1_000, 5_000,],
            },),).toMatchObject({
              emptied: 'done',
              freeBytesBefore: 1_000,
              freeBytesAfter: 5_000,
            },);
            /**
             What emptying it answered when the volume would not answer about its room.
             */
            const unreadable = await emptyTheFolderIn({
              folder: scratch.folder,
              scratchRoot: scratch.scratchRoot,
              runName: 'a',
              mountsOf: async function mounts() {
                return [];
              },
              freeBytesOf: async function room() {
                throw new Error('the volume would not say',);
              },
            },);
            expect(unreadable,).toMatchObject({
              emptied: 'done',
              freeSpaceProblem: 'the volume would not say',
            },);
            expect(scratchEmptyingLinesOf({
              runName: 'a',
              folder: scratch.folder,
              emptying: doneOf(unreadable,),
            },).join('\n',),).toContain('the room the volume gained could not be read (the volume would not say)',);
          },
        },),
      ],
    },),
    describe({
      name: mountPathsIn.name,
      children: [
        it({
          name: 'reads each mount point, with the escapes the kernel writes a space, a tab and a backslash as',
          fn: async () => {
            expect(mountPathsIn({
              shown: [
                '/dev/nvme0n1p2 / btrfs rw,relatime 0 0',
                '/dev/nvme0n1p1 /boot vfat rw 0 0',
                String.raw`tmpfs /mnt/a\040b\011c\134d tmpfs rw 0 0`,
                '',
              ].join('\n',),
            },),).toEqual([
              '/',
              '/boot',
              '/mnt/a b\tc\\d',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: ownershipProblemOf.name,
      children: [
        it({
          name: 'says nothing only when the manager says the unit has gone and no process works in the folder',
          fn: async () => {
            expect(ownershipProblemOf({
              runName: 'copy-route',
              unitState: 'inactive',
              processes: [],
              folder: '/home/a/temp/agent/copy-route-scratch',
              excludingPid: 1,
            },),).toEqual({},);
          },
        },),
        it({
          name: 'refuses while a run of that name is going, and refuses the same way when the manager answered nothing at all',
          fn: async () => {
            for (const unitState of ['active', 'activating', 'deactivating', 'reloading', 'refreshing',]) {
              expect(ownershipProblemOf({
                runName: 'copy-route',
                unitState,
                processes: [],
                folder: '/home/a/temp/agent/copy-route-scratch',
                excludingPid: 1,
              },).problem,).toContain('a run of that name is going now',);
            }
            expect(ownershipProblemOf({
              runName: 'copy-route',
              unitState: '',
              processes: [],
              folder: '/home/a/temp/agent/copy-route-scratch',
              excludingPid: 1,
            },).problem,).toContain('is in no state anybody could read',);
          },
        },),
        it({
          name: 'refuses while a process of the account works in the folder, and counts no process of the emptying itself',
          fn: async () => {
            expect(ownershipProblemOf({
              runName: 'copy-route',
              unitState: 'inactive',
              processes: [
                { pid: 9_001, folder: '/home/a/temp/agent/copy-route-scratch/build', command: 'bash', },
                { pid: 9_002, folder: '/elsewhere', command: 'node', },
              ],
              folder: '/home/a/temp/agent/copy-route-scratch',
              excludingPid: 1,
            },).problem,).toContain('a process still works in /home/a/temp/agent/copy-route-scratch/build (9001 bash)',);
            // The process doing the emptying is not the run's work, wherever it was started.
            expect(ownershipProblemOf({
              runName: 'copy-route',
              unitState: 'inactive',
              processes: [{ pid: 9_003, folder: '/home/a/temp/agent/copy-route-scratch', command: 'node', },],
              folder: '/home/a/temp/agent/copy-route-scratch',
              excludingPid: 9_003,
            },),).toEqual({},);
          },
        },),
      ],
    },),
    describe({
      name: emptyTheFolderItsRunStillOwns.name,
      children: [
        it({
          name: 'empties the folder when the run that ended is proven to still own it, and deletes nothing while a run of that name is going',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The run's scratch folder, with a file a new attempt would lose.
             */
            const scratch = await makeScratch({
              base: folder.path,
              runName: 'a',
            },);
            /**
             The file in it.
             */
            const sentinel = join(
              scratch.folder,
              'a-new-attempts-output.txt',
            );
            await writeFile(sentinel, 'the new attempt is writing this\n',);
            /**
             What the emptying answered while the manager said a run of that name was going.
             */
            const going = await emptyTheFolderItsRunStillOwns({
              folder: scratch.folder,
              scratchRoot: scratch.scratchRoot,
              runName: 'a',
              run: async function theManager(words: { readonly words: readonly string[] },) {
                return (words.words.includes('pi-a.service',))
                  ? { status: 0, stdout: 'activating\n', stderr: '', }
                  : { status: 0, stdout: 'inactive\n', stderr: '', };
              },
              processFolder: await processTableWorkingIn({
                base: folder.path,
                working: folder.path,
              },),
              mountsOf: async function mounts() {
                return [];
              },
              freeBytesOf: async function room() {
                return 0;
              },
            },);
            expect(going.emptied,).toBe('refused',);
            expect((going as { readonly reason: string }).reason,).toContain('a run of that name is going now',);
            expect(await namesInFolder(scratch.folder,),).toContain('a-new-attempts-output.txt',);
            /**
             What the emptying answered while a process worked in the folder instead.
             */
            const worked = await emptyTheFolderItsRunStillOwns({
              folder: scratch.folder,
              scratchRoot: scratch.scratchRoot,
              runName: 'a',
              run: async function theManager() {
                return { status: 0, stdout: 'inactive\n', stderr: '', };
              },
              processFolder: await processTableWorkingIn({
                base: join(folder.path, 'second',),
                working: join(
                  scratch.folder,
                  'build',
                ),
              },),
              mountsOf: async function mounts() {
                return [];
              },
              freeBytesOf: async function room() {
                return 0;
              },
            },);
            expect(worked.emptied,).toBe('refused',);
            expect((worked as { readonly reason: string }).reason,).toContain('a process still works in',);
            expect(await namesInFolder(scratch.folder,),).toContain('a-new-attempts-output.txt',);
            /**
             What the emptying answered with the unit gone and nothing working in the folder.
             */
            const emptied = await emptyTheFolderItsRunStillOwns({
              folder: scratch.folder,
              scratchRoot: scratch.scratchRoot,
              runName: 'a',
              run: async function theManager() {
                return { status: 0, stdout: 'inactive\n', stderr: '', };
              },
              processFolder: await processTableWorkingIn({
                base: join(folder.path, 'third',),
                working: folder.path,
              },),
              mountsOf: async function mounts() {
                return [];
              },
              freeBytesOf: async function room() {
                return 0;
              },
            },);
            expect(emptied.emptied,).toBe('done',);
            expect(await namesInFolder(scratch.folder,),).toEqual([KEEP_FOLDER,],);
          },
        },),
      ],
    },),
    describe({
      name: leftWhereItIsIn.name,
      children: [
        it({
          name: 'names a mount point and another filesystem as reasons an entry stays, and nothing for an entry of the folder own device',
          fn: async () => {
            expect(leftWhereItIsIn({
              path: '/home/a/temp/agent/a-scratch/mnt',
              mountPoints: new Set(['/home/a/temp/agent/a-scratch/mnt',],),
              device: 1,
              referenceDevice: 1,
            },).reason,).toContain('it is a mount point',);
            expect(leftWhereItIsIn({
              path: '/home/a/temp/agent/a-scratch/other',
              mountPoints: new Set<string>(),
              device: 7,
              referenceDevice: 1,
            },).reason,).toContain('it is on another filesystem (device 7)',);
            expect(leftWhereItIsIn({
              path: '/home/a/temp/agent/a-scratch/own',
              mountPoints: new Set<string>(),
              device: 1,
              referenceDevice: 1,
            },),).toEqual({},);
          },
        },),
      ],
    },),
    describe({
      name: bytesTextOf.name,
      children: [
        it({
          name: 'reads a size in the largest unit it reaches, and in bytes below a kibibyte',
          fn: async () => {
            expect(bytesTextOf(0,),).toBe('0 B',);
            expect(bytesTextOf(1_023,),).toBe('1023 B',);
            expect(bytesTextOf(1_024,),).toBe('1.0 KiB',);
            expect(bytesTextOf(1_572_864,),).toBe('1.5 MiB',);
            expect(bytesTextOf(2_147_483_648,),).toBe('2.0 GiB',);
          },
        },),
      ],
    },),
    describe({
      name: scratchEmptyingLinesOf.name,
      children: [
        it({
          name: 'says how much was removed, what keep holds, and how much room the volume gained, with the size figure named as a sum of file sizes',
          fn: async () => {
            /**
             The lines one emptied folder answers with.
             */
            const lines = scratchEmptyingLinesOf({
              runName: 'copy-route',
              folder: '/home/a/temp/agent/copy-route-scratch',
              emptying: {
                emptied: 'done',
                removedBytes: 1_536,
                removedFiles: 3,
                removedFolders: 1,
                keepBytes: 11,
                keepFiles: 1,
                leftBehind: [],
                freeBytesBefore: 1_000,
                freeBytesAfter: 4_096,
              },
            },);
            expect(lines,).toHaveLength(2,);
            expect(lines[0],).toContain('emptied /home/a/temp/agent/copy-route-scratch except keep/:',);
            expect(lines[0],).toContain('3 files and 1 folder removed',);
            expect(lines[0],).toContain('whose sizes read 1.5 KiB',);
            expect(lines[0],).toContain('a sum of the sizes of the files removed',);
            expect(lines[0],).toContain('keep/ holds 11 B in 1 file',);
            expect(lines[1],).toContain('so it gained 3.0 KiB',);
          },
        },),
        it({
          name: 'names the entries it could not remove and counts the rest, so a folder of a thousand of them still answers with lines a reader reads',
          fn: async () => {
            /**
             The lines for a folder holding more entries left behind than a line names.
             */
            const lines = scratchEmptyingLinesOf({
              runName: 'a',
              folder: '/home/a/temp/agent/a-scratch',
              emptying: {
                emptied: 'done',
                removedBytes: 0,
                removedFiles: 0,
                removedFolders: 0,
                keepBytes: 0,
                keepFiles: 0,
                leftBehind: Array.from(
                  { length: ENTRIES_NAMED_IN_A_LINE + 2, },
                  function entryAt(_unused, index,) {
                    return {
                      path: `/home/a/temp/agent/a-scratch/closed-${String(index,)}`,
                      reason: 'EACCES: permission denied',
                    };
                  },
                ),
                freeBytesBefore: 1_000,
                freeBytesAfter: 1_000,
              },
            },);
            expect(lines,).toHaveLength(2 + ENTRIES_NAMED_IN_A_LINE + 1,);
            expect(lines.at(-1,),).toBe(`scratch: a: and 2 more entries were left where they are`,);
            expect(lines[2],).toContain('left /home/a/temp/agent/a-scratch/closed-0 where it is: EACCES: permission denied',);
          },
        },),
        it({
          name: 'says a folder nothing had ended in, and a folder it refused, each in one line that names why',
          fn: async () => {
            expect(scratchEmptyingLinesOf({
              runName: 'a',
              folder: '/home/a/temp/agent/a-scratch',
              emptying: { emptied: 'no-folder', },
            },),).toEqual(['scratch: a: no folder to empty at /home/a/temp/agent/a-scratch',],);
            expect(scratchEmptyingLinesOf({
              runName: 'a',
              folder: '/home/a/temp/agent/a-scratch',
              emptying: {
                emptied: 'refused',
                reason: 'it is a mount point, so emptying it would cross into a filesystem this run did not make',
              },
            },),).toEqual([
              'scratch: a: refused to empty /home/a/temp/agent/a-scratch: it is a mount point, so emptying it would cross into a '
                + 'filesystem this run did not make',
            ],);
          },
        },),
        it({
          name: 'says the volume gained nothing in its own words when the two readings show no gain',
          fn: async () => {
            /**
             The lines for a deletion the volume shows no gain for.
             */
            const lines = scratchEmptyingLinesOf({
              runName: 'a',
              folder: '/home/a/temp/agent/a-scratch',
              emptying: {
                emptied: 'done',
                removedBytes: 0,
                removedFiles: 0,
                removedFolders: 0,
                keepBytes: 0,
                keepFiles: 0,
                leftBehind: [],
                freeBytesBefore: 1_000,
                freeBytesAfter: 1_000,
              },
            },);
            expect(lines[1],).toContain('so it gained nothing',);
            expect(lines[1],).toContain('another process wrote there while this ran',);
          },
        },),
      ],
    },),
    describe({
      name: emptyTheScratchFolderAtTheLowestDiskPriority.name,
      children: [
        it({
          name: 'names a deletion its own bound ended, with what it left, and never tries it again at full disk priority',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The run's scratch folder, holding what the bound left undone.
             */
            const scratch = await makeScratch({
              base: folder.path,
              runName: 'a',
            },);
            await mkdir(join(
              scratch.folder,
              'a-build-tree',
            ), { recursive: true, },);
            await writeFile(join(
              scratch.folder,
              'a-log.txt',
            ), 'a log\n',);
            await writeFile(join(
              scratch.folder,
              'a-build-tree',
              'an-object.bin',
            ), 'an object\n',);
            /**
             Every command the step asked for, so a second attempt is a second attempt.
             */
            const asked: string[] = [];
            /**
             What the step answered.
             */
            const lines = await emptyTheScratchFolderAtTheLowestDiskPriority({
              scratchRoot: scratch.scratchRoot,
              runName: 'a',
              run: async function boundKilled(request,) {
                asked.push(request.command,);
                return {
                  status: -1,
                  stdout: '',
                  stderr: '',
                  problem: `Command failed: ${request.command} --class idle -- ...`,
                  killedByItsOwnBound: true,
                };
              },
            },);
            expect(asked,).toEqual(['ionice',],);
            expect(lines.join('\n',),).toContain('it ran past the 1800 second bound this step gives one deletion',);
            expect(lines.join('\n',),).toContain('2 files and 2 folders are still in',);
            expect(lines.join('\n',),).toContain('the next end of a run of this name, or empty-the-scratch-folder-of-a-run, takes them',);
          },
        },),
        it({
          name: 'makes the deletion again without the disk class only when this machine has no program for it',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The run's scratch folder, which the second attempt empties.
             */
            const scratch = await makeScratch({
              base: folder.path,
              runName: 'a',
            },);
            /**
             Every command the step asked for.
             */
            const asked: string[] = [];
            /**
             What the step answered when the machine had no `ionice` at all.
             */
            const lines = await emptyTheScratchFolderAtTheLowestDiskPriority({
              scratchRoot: scratch.scratchRoot,
              runName: 'a',
              run: async function withoutTheProgram(request,) {
                asked.push(request.command,);
                if (request.command === 'ionice') {
                  return {
                    status: -1,
                    stdout: '',
                    stderr: '',
                    problem: 'spawn ionice ENOENT',
                    problemCode: 'ENOENT',
                  };
                }
                return {
                  status: 0,
                  stdout: `scratch: a: emptied ${scratch.folder} except keep/: 0 files and 0 folders removed\n`,
                  stderr: '',
                };
              },
            },);
            expect(asked,).toHaveLength(2,);
            expect(lines,).toEqual([`scratch: a: emptied ${scratch.folder} except keep/: 0 files and 0 folders removed`,],);
          },
        },),
      ],
    },),
    describe({
      name: 'the command empty-the-scratch-folder-of-a-run',
      children: [
        it({
          name: 'empties the folder the run name and root name, and answers what the run end reports',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The run's scratch folder.
             */
            const scratch = await makeScratch({
              base: folder.path,
              runName: 'a name with spaces',
            },);
            await writeFile(join(
              scratch.folder,
              'junk.txt',
            ), 'junk\n',);
            /**
             What the command printed and how it ended.
             */
            const ended = await execFileAsync(process.execPath, [
              EMPTIER,
              `--scratch-root=${scratch.scratchRoot}`,
              '--run-name=a name with spaces',
            ], { encoding: 'utf8', },);
            expect(ended.stdout,).toContain(`scratch: a name with spaces: emptied ${scratch.folder} except keep/:`,);
            expect(await namesInFolder(scratch.folder,),).toEqual([KEEP_FOLDER,],);
          },
        },),
        it({
          name: 'proves the run that ended still owns the folder inside its own process, so a run of that name going is caught as late as the deletion itself',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The run's scratch folder, with the file a new attempt wrote.
             */
            const scratch = await makeScratch({
              base: folder.path,
              runName: 'a',
            },);
            await writeFile(join(
              scratch.folder,
              'a-new-attempts-output.txt',
            ), 'the new attempt is writing this\n',);
            /**
             The manager the emptying asks, which says a run of that name is going.
             */
            const bin = await managerSaying({
              base: folder.path,
              state: 'activating',
            },);
            /**
             What the command printed and how it ended.
             */
            let ended: { readonly code?: number; readonly stdout?: string } = {};
            try {
              /**
               What the command printed when it refused.
               */
              const refused = await execFileAsync(process.execPath, [
                EMPTIER,
                `--scratch-root=${scratch.scratchRoot}`,
                '--run-name=a',
              ], {
                encoding: 'utf8',
                env: {
                  ...process.env,
                  PATH: `${bin}:${process.env.PATH ?? ''}`,
                },
              },);
              ended = {
                code: 0,
                stdout: refused.stdout,
              };
            }
            catch (error) {
              ended = error as { readonly code?: number; readonly stdout?: string };
            }
            expect(ended.code,).toBe(1,);
            expect(ended.stdout,).toContain('a run of that name is going now',);
            expect(await namesInFolder(scratch.folder,),).toContain('a-new-attempts-output.txt',);
          },
        },),
        it({
          name: 'refuses a folder that is a link and exits non-zero, saying why',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The root, and a link where a run's folder would be.
             */
            const root = join(
              folder.path,
              'agent',
            );
            /**
             What the link points at, which nobody may lose.
             */
            const target = join(
              folder.path,
              'target',
            );
            await mkdir(target, { recursive: true, },);
            await writeFile(join(
              target,
              'precious.txt',
            ), 'precious\n',);
            await mkdir(root, { recursive: true, },);
            await symlink(
              target,
              join(
                root,
                'a-scratch',
              ),
            );
            /**
             Why the command refused, as the failure Node reports carries it.
             */
            let caught: unknown;
            try {
              await execFileAsync(process.execPath, [
                EMPTIER,
                `--scratch-root=${root}`,
                '--run-name=a',
              ], { encoding: 'utf8', },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as { readonly code?: number }).code,).toBe(1,);
            expect((caught as { readonly stdout?: string }).stdout,).toContain('refused to empty',);
            expect((caught as { readonly stdout?: string }).stdout,).toContain('a symbolic link',);
            expect(await readFile(join(
              target,
              'precious.txt',
            ), 'utf8',),).toBe('precious\n',);
          },
        },),
      ],
    },),
    it({
      name: 'never follows a link that names the folder itself when it asks what the folder is',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The root, and a link at the run's own folder's place.
         */
        const root = join(
          folder.path,
          'agent',
        );
        await mkdir(root, { recursive: true, },);
        await symlink(
          folder.path,
          join(
            root,
            'a-scratch',
          ),
        );
        expect((await lstat(join(
          root,
          'a-scratch',
        ))).isSymbolicLink(),).toBe(true,);
        expect((await stat(join(
          root,
          'a-scratch',
        ))).isDirectory(),).toBe(true,);
        // The folder is a link, so the emptying refuses it: what it points at is not the run's scratch folder.
        expect(await emptyTheFolderIn({
          folder: join(
            root,
            'a-scratch',
          ),
          scratchRoot: root,
          runName: 'a',
          mountsOf: async function mounts() {
            return [];
          },
          freeBytesOf: async function room() {
            return 0;
          },
        },).then(function outcomeOf(answer,) {
          return answer.emptied;
        },),).toBe('refused',);
      },
    },),
  ],
},);
