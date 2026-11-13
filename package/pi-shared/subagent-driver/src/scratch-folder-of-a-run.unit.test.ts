/**
 The one scratch folder a run gets: the name it carries, the root it lives under, the check that refuses a folder which is not a
 run's own, and making the folder with its `keep/` subfolder.

 Every folder here is one the test made under a root of its own, so no test reads or writes the machine's own scratch root.

 @module
 */

import {
  chmod,
  lstat,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  stat,
  symlink,
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
  KEEP_FOLDER,
  SCRATCH_FOLDER_MODE,
  SCRATCH_FOLDER_VARIABLE,
  ScratchFolderError,
  defaultScratchRoot,
  makeScratchFolder,
  nameProblemOf,
  scratchFolderNameOf,
  scratchFolderObstacleIn,
  scratchFolderOf,
  scratchFolderProblemIn,
  scratchNoteOf,
} from '../dist/final/node/index.mjs';

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
  const path = await mkdtemp(join(scratchRoot(), 'pi-scratch-folder-',),);
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
 Reads the permissions of one path, without the bits a directory's own kind carries.

 @param path - the path to read

 @returns the permission bits, such as 448 for a folder only its owner may read

 @example
 ```ts
 const mode = await modeOf('/home/a/temp/agent/a-scratch');
 ```
 */
async function modeOf(path: string,): Promise<number> {
  return (await stat(path,)).mode & 0o777;
}

await describe({
  name: '',
  children: [
    describe({
      name: defaultScratchRoot.name,
      children: [
        it({
          name: 'puts a run scratch root at temp/agent under the home directory, which is where rule ATH puts it',
          fn: async () => {
            expect(defaultScratchRoot({ home: '/home/a', },),).toBe('/home/a/temp/agent',);
          },
        },),
      ],
    },),
    describe({
      name: scratchFolderNameOf.name,
      children: [
        it({
          name: 'names the folder after the run and the word that says the folder is scratch',
          fn: async () => {
            expect(scratchFolderNameOf('copy-route',),).toBe('copy-route-scratch',);
            expect(scratchFolderOf({
              scratchRoot: '/home/a/temp/agent',
              runName: 'copy-route',
            },),).toBe('/home/a/temp/agent/copy-route-scratch',);
          },
        },),
      ],
    },),
    describe({
      name: nameProblemOf.name,
      children: [
        it({
          name: 'refuses a name that holds a folder separator or the character that ends a path, which would put the folder outside the root',
          fn: async () => {
            expect(nameProblemOf('../etc',).problem,).toContain('cannot name a folder of its own',);
            expect(nameProblemOf('a/b',).problem,).toContain('cannot name a folder of its own',);
            expect(nameProblemOf('a\0b',).problem,).toContain('cannot name a folder of its own',);
            expect(nameProblemOf('',).problem,).toContain('no name',);
          },
        },),
        it({
          name: 'accepts a name with spaces, a newline or a leading dash, which a folder may carry',
          fn: async () => {
            for (const runName of ['a name', 'a\nname', '--odd', '..', 'a-scratch',]) {
              expect(nameProblemOf(runName,),).toEqual({},);
            }
          },
        },),
      ],
    },),
    describe({
      name: scratchFolderProblemIn.name,
      children: [
        it({
          name: 'accepts the one folder the launcher gives that run, directly under the scratch root',
          fn: async () => {
            expect(scratchFolderProblemIn({
              folder: '/home/a/temp/agent/copy-route-scratch',
              scratchRoot: '/home/a/temp/agent',
              runName: 'copy-route',
            },),).toEqual({},);
          },
        },),
        it({
          name: 'refuses a folder that is not directly under the scratch root, and names both folders',
          fn: async () => {
            /**
             The refusal for a folder one level deeper.
             */
            const deeper = scratchFolderProblemIn({
              folder: '/home/a/temp/agent/elsewhere/copy-route-scratch',
              scratchRoot: '/home/a/temp/agent',
              runName: 'copy-route',
            },);
            expect(deeper.problem,).toContain('is not directly under the scratch root /home/a/temp/agent',);
            /**
             The refusal for a path that leaves the root through `..`.
             */
            const escaped = scratchFolderProblemIn({
              folder: '/home/a/temp/agent/../copy-route-scratch',
              scratchRoot: '/home/a/temp/agent',
              runName: 'copy-route',
            },);
            expect(escaped.problem,).toContain('is not directly under the scratch root /home/a/temp/agent',);
          },
        },),
        it({
          name: 'refuses a folder whose name is not the one the launcher gives that run, and names the folder it would be',
          fn: async () => {
            /**
             The refusal for another run's folder.
             */
            const other = scratchFolderProblemIn({
              folder: '/home/a/temp/agent/another-run-scratch',
              scratchRoot: '/home/a/temp/agent',
              runName: 'copy-route',
            },);
            expect(other.problem,).toContain('is not the folder a run named copy-route gets',);
            expect(other.problem,).toContain('/home/a/temp/agent/copy-route-scratch',);
          },
        },),
        it({
          name: 'refuses a run name that would reach out of the root before it reads anything about the folder',
          fn: async () => {
            expect(scratchFolderProblemIn({
              folder: '/home/a/temp/agent/../../copy-route-scratch',
              scratchRoot: '/home/a/temp/agent',
              runName: '../copy-route',
            },).problem,).toContain('cannot name a folder of its own',);
          },
        },),
      ],
    },),
    describe({
      name: makeScratchFolder.name,
      children: [
        it({
          name: 'makes the folder and its keep subfolder with the mode rule ATH asks for, and says it made them',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The root this test's run lives under.
             */
            const root = join(
              folder.path,
              'agent',
            );
            /**
             What making the folder answered.
             */
            const made = await makeScratchFolder({
              scratchRoot: root,
              runName: 'copy-route',
            },);
            expect(made.made,).toBe(true,);
            expect(made.folder,).toBe(join(
              root,
              'copy-route-scratch',
            ),);
            expect(made.keepFolder,).toBe(join(
              root,
              'copy-route-scratch',
              KEEP_FOLDER,
            ),);
            expect(await modeOf(made.folder,),).toBe(SCRATCH_FOLDER_MODE,);
            expect(await modeOf(made.keepFolder,),).toBe(SCRATCH_FOLDER_MODE,);
            await writeFile(join(
              made.keepFolder,
              'report.md',
            ), 'kept\n',);
          },
        },),
        it({
          name: 'gives a resumed run the same folder again, with keep as the earlier run left it',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The root this test's run lives under.
             */
            const root = join(
              folder.path,
              'agent',
            );
            /**
             What the first run of that name made.
             */
            const first = await makeScratchFolder({
              scratchRoot: root,
              runName: 'copy-route',
            },);
            await writeFile(join(
              first.keepFolder,
              'report.md',
            ), 'the report\n',);
            /**
             What the resumed run of the same name found.
             */
            const second = await makeScratchFolder({
              scratchRoot: root,
              runName: 'copy-route',
            },);
            expect(second.made,).toBe(false,);
            expect(second.folder,).toBe(first.folder,);
            expect(await readFile(join(
              second.keepFolder,
              'report.md',
            ), 'utf8',),).toBe('the report\n',);
          },
        },),
        it({
          name: 'makes the scratch root itself when it is not there, and keeps a folder it finds',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The root this test's run lives under, which is two folders that do not exist yet.
             */
            const root = join(
              folder.path,
              'temp',
              'agent',
            );
            expect((await makeScratchFolder({
              scratchRoot: root,
              runName: 'a',
            },)).folder,).toBe(join(
              root,
              'a-scratch',
            ),);
            expect(await modeOf(root,),).toBe(SCRATCH_FOLDER_MODE,);
          },
        },),
        it({
          name: 'refuses a symbolic link at the folder path, leaves what it points at untouched, and puts no keep inside it',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The root this test's run lives under.
             */
            const root = join(
              folder.path,
              'agent',
            );
            /**
             The folder outside the root the link points at, holding a file and a mode of its own.
             */
            const target = join(
              folder.path,
              'a-user-directory',
            );
            await mkdir(target, { recursive: true, },);
            await writeFile(join(
              target,
              'precious.txt',
            ), 'do not touch\n',);
            await chmod(target, 0o755,);
            await mkdir(root, { recursive: true, },);
            await symlink(
              target,
              join(
                root,
                'a-scratch',
              ),
            );
            /**
             Why making the folder failed.
             */
            let caught: unknown;
            try {
              await makeScratchFolder({
                scratchRoot: root,
                runName: 'a',
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(ScratchFolderError,);
            expect((caught as Error).message,).toContain(`is a symbolic link to ${target}`,);
            expect((caught as Error).message,).toContain('Remove it, or empty it with empty-the-scratch-folder-of-a-run',);
            // The target keeps its own mode, its own file, and no keep/ of a run that is not its own.
            expect(await modeOf(target,),).toBe(0o755,);
            expect(await readFile(join(
              target,
              'precious.txt',
            ), 'utf8',),).toBe('do not touch\n',);
            await expect(
              lstat(join(
                target,
                'keep',
              ),),
            ).rejects.toThrow('ENOENT',);
            // And the path is still the link it was, rather than a folder this start made.
            expect((await lstat(join(
              root,
              'a-scratch',
            ))).isSymbolicLink(),).toBe(true,);
          },
        },),
        it({
          name: 'refuses a link that points nowhere with the same refusal, not with a raw error from making the folder',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The root this test's run lives under.
             */
            const root = join(
              folder.path,
              'agent',
            );
            await mkdir(root, { recursive: true, },);
            await symlink(
              join(
                folder.path,
                'no-such-folder',
              ),
              join(
                root,
                'a-scratch',
              ),
            );
            /**
             Why making the folder failed.
             */
            let caught: unknown;
            try {
              await makeScratchFolder({
                scratchRoot: root,
                runName: 'a',
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(ScratchFolderError,);
            expect((caught as { readonly code?: string }).code,).toBeUndefined();
            expect((caught as Error).message,).toContain('is a symbolic link to',);
          },
        },),
        it({
          name: 'refuses a file where the folder would be, and says what is there',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The root this test's run lives under, with a file where the run's folder would be.
             */
            const root = join(
              folder.path,
              'agent',
            );
            await mkdir(root, { recursive: true, },);
            await writeFile(join(
              root,
              'a-scratch',
            ), 'not a folder\n',);
            /**
             Why making the folder failed.
             */
            let caught: unknown;
            try {
              await makeScratchFolder({
                scratchRoot: root,
                runName: 'a',
              },);
            }
            catch (error) {
              caught = error;
            }
            expect((caught as Error).message,).toContain('is not a folder (a file)',);
            expect(await readFile(join(
              root,
              'a-scratch',
            ), 'utf8',),).toBe('not a folder\n',);
          },
        },),
        it({
          name: 'refuses a run name that would put the folder outside the root, before it makes anything',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             Why making that folder failed.
             */
            let caught: unknown;
            try {
              await makeScratchFolder({
                scratchRoot: folder.path,
                runName: '../escaped',
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(ScratchFolderError,);
            expect((caught as Error).message,).toContain('cannot name a folder of its own',);
          },
        },),
      ],
    },),
    describe({
      name: scratchFolderObstacleIn.name,
      children: [
        it({
          name: 'answers nothing for a path a run may be given, and for a folder an earlier run of that name left',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The root this test's runs live under.
             */
            const root = join(
              folder.path,
              'agent',
            );
            expect(await scratchFolderObstacleIn({
              scratchRoot: root,
              runName: 'a',
            },),).toEqual({},);
            await makeScratchFolder({
              scratchRoot: root,
              runName: 'a',
            },);
            expect(await scratchFolderObstacleIn({
              scratchRoot: root,
              runName: 'a',
            },),).toEqual({},);
          },
        },),
        it({
          name: 'names a symbolic link, and where it points, as what is at the path instead of the folder',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The root this test's runs live under.
             */
            const root = join(
              folder.path,
              'agent',
            );
            /**
             A folder outside the root, which the link points at.
             */
            const target = join(
              folder.path,
              'a-user-directory',
            );
            await mkdir(target, { recursive: true, },);
            await mkdir(root, { recursive: true, },);
            await symlink(
              target,
              join(
                root,
                'a-scratch',
              ),
            );
            /**
             What the check answered about the link.
             */
            const obstacle = await scratchFolderObstacleIn({
              scratchRoot: root,
              runName: 'a',
            },);
            expect(obstacle.problem,).toContain(`is a symbolic link to ${target}, so it is not the scratch folder this run would write in`,);
          },
        },),
        it({
          name: 'names a path a run name cannot reach, which is refused before the disk is read',
          fn: async () => {
            await using folder = await makeFolder();
            expect((await scratchFolderObstacleIn({
              scratchRoot: folder.path,
              runName: '../a',
            },)).problem,).toContain('cannot name a folder of its own',);
          },
        },),
      ],
    },),
    describe({
      name: scratchNoteOf.name,
      children: [
        it({
          name: 'names the folder, what keep is for, and the command that prints the same path, so the agent can miss neither',
          fn: async () => {
            /**
             The folder this note is about.
             */
            const folder = '/home/a/temp/agent/copy-route-scratch';
            /**
             The note a run's first prompt carries.
             */
            const note = scratchNoteOf({ folder, },);
            expect(note,).toContain(folder,);
            expect(note,).toContain(`\`${KEEP_FOLDER}/\``,);
            expect(note,).toContain(`printenv ${SCRATCH_FOLDER_VARIABLE}`,);
            expect(note,).toContain('Everything you write outside your worktree goes in that folder',);
            // The note is whole paragraphs and ends with a blank line, so it stands before what the brief says.
            expect(note.endsWith('\n\n',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
