/**
 The one scratch folder a run gets: the name it carries, the root it lives under, the folder the launcher makes with its `keep/`
 subfolder, the note the run's first prompt carries, and the check that refuses a folder which is not the one the launcher gives a
 run.

 The owner, asked when a run's scratch is deleted: "At run end, all but keep/ . Oh and I forgot to say all the dir names were
 extremely cryptic and shouldn't have been." So the name carries the run's own name and a word that says what the folder is:
 someone who sees only the name knows which run made it and that it is scratch.

 On 2026-11-13 `~/temp/agent` held two days of runs and subagents that nothing had removed: 190.5 GB as `du` counts it, of which
 68.2 GB (63.5 GiB) was its own data and 14.1 GB was shared between the copies inside it. That is the state this module and the
 emptying beside it exist to keep from coming back.

 @module
 */

import {
  chmod,
  lstat,
  mkdir,
  readlink,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  join,
  resolve,
} from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';

/**
 The word a run's scratch folder carries after the run's own name, so its name says what the folder is as well as which run made
 it.
 */
export const SCRATCH_FOLDER_END = '-scratch';

/**
 The subfolder of a run's scratch folder that survives the end of the run, which is where the report, the logs and the
 measurements a reader needs go.
 */
export const KEEP_FOLDER = 'keep';

/**
 The variable a run's own commands read the scratch folder from, so an agent learns the path from a command rather than from a
 path it built itself.
 */
export const SCRATCH_FOLDER_VARIABLE = 'PI_RUN_SCRATCH_FOLDER';

/**
 The mode of a run's scratch folder and of its `keep/` subfolder: what rule `ATH` asks for, because a folder anyone of the
 account may read is a folder another session's commands read too.
 */
export const SCRATCH_FOLDER_MODE = 0o700;

/**
 The parts of the scratch root below the home directory, which is where rule `ATH` puts a run's scratch.
 */
export const SCRATCH_ROOT_IN_HOME: readonly string[] = [
  'temp',
  'agent',
];

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The character that separates one folder from the next, which a run's name may never hold.
 */
const FOLDER_SEPARATOR = '/';

/**
 The character that ends a path in the kernel's own bookkeeping, which a run's name may never hold either.
 */
const NO_PATH_CHARACTER = '\0';

/**
 A folder name this package will not make or empty.
 */
export class ScratchFolderError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ScratchFolderError';
}

/**
 Reads the scratch root a run's folder goes under when no caller names one: `~/temp/agent`, as rule `ATH` puts it.

 @param home - the home directory the run's environment names

 @returns the scratch root's path

 @example
 ```ts
 const root = defaultScratchRoot({ home: '/home/a', },);
 ```
 */
export function defaultScratchRoot({ home, }: { readonly home: string },): string {
  return join(
    home,
    ...SCRATCH_ROOT_IN_HOME,
  );
}

/**
 Reads the name a run's scratch folder carries: the run's own name and the word that says what the folder is.

 @param runName - the run's name

 @returns the folder's name, without the root it lives under

 @example
 ```ts
 const name = scratchFolderNameOf('copy-route',);
 ```
 */
export function scratchFolderNameOf(runName: string,): string {
  return `${runName}${SCRATCH_FOLDER_END}`;
}

/**
 Reads where a run's scratch folder lives.

 @param scratchRoot - the folder every run's scratch lives under

 @param runName - the run's name

 @returns the run's scratch folder

 @example
 ```ts
 const folder = scratchFolderOf({ scratchRoot: '/home/a/temp/agent', runName: 'copy-route', },);
 ```
 */
export function scratchFolderOf({
  scratchRoot,
  runName,
}: {
  readonly scratchRoot: string;
  readonly runName: string;
},): string {
  return join(
    scratchRoot,
    scratchFolderNameOf(runName,),
  );
}

/**
 Reads whether a run's name can name a file at all, which a name holding a folder separator or the character that ends a path in
 the kernel cannot: such a name would put the folder outside the scratch root, or cut it in two.

 @param runName - the run's name

 @returns why the name cannot name a folder, or nothing when it can

 @example
 ```ts
 const problem = nameProblemOf('../../etc',);
 ```
 */
export function nameProblemOf(runName: string,): { readonly problem?: string } {
  if (runName === '') {
    return { problem: 'the run has no name, so it has no folder of its own', };
  }
  if (runName
    .includes(FOLDER_SEPARATOR,)
    || runName.includes(NO_PATH_CHARACTER,)) {
    return { problem: `the name ${JSON.stringify(runName,)} holds a folder separator or the character that ends a path, so it cannot name a folder of its own`, };
  }
  return {};
}

/**
 Says whether one path is the scratch folder the launcher gives one run: a folder directly under the scratch root, named after the
 run with the word that says what it is.

 Nothing here reads the disk: this is the check the emptying asks before it touches anything, so a caller that names a folder by
 hand, or a run whose name would reach out of the root, is refused rather than deleted.

 @param folder - the folder a caller named

 @param scratchRoot - the root every run's scratch lives directly under

 @param runName - the run the folder is said to belong to

 @returns why the folder is not that run's scratch folder, or nothing when it is

 @example
 ```ts
 const problem = scratchFolderProblemIn({ folder: '/home/a/temp/agent/a-scratch', scratchRoot: '/home/a/temp/agent', runName: 'a', },);
 ```
 */
export function scratchFolderProblemIn({
  folder,
  scratchRoot,
  runName,
}: {
  readonly folder: string;
  readonly scratchRoot: string;
  readonly runName: string;
},): { readonly problem?: string } {
  /**
   Why the run's name cannot name a folder at all.
   */
  const nameProblem = nameProblemOf(runName,);
  if (nameProblem.problem !== undefined) {
    return nameProblem;
  }
  /**
   The folder as the kernel reads it, with every `..` already folded.
   */
  const shown = resolve(folder,);
  /**
   The root as the kernel reads it.
   */
  const root = resolve(scratchRoot,);
  if (dirname(shown,) !== root) {
    return { problem: `${shown} is not directly under the scratch root ${root}`, };
  }
  /**
   The name the launcher gives that run's folder.
   */
  const expected = scratchFolderNameOf(runName,);
  if (basename(shown,) !== expected) {
    return { problem: `${shown} is not the folder a run named ${runName} gets, which is ${join(
      root,
      expected,
    )}`, };
  }
  return {};
}

/**
 Says whether a folder is there at all, so the launcher's line can say whether this run made it or found it.

 @param folder - the folder to look for

 @returns whether it is there

 @throws the error a read gave, other than a folder that is not there

 @example
 ```ts
 const there = await folderIsThere('/home/a/temp/agent/a-scratch',);
 ```
 */
async function folderIsThere(folder: string,): Promise<boolean> {
  try {
    await lstat(folder,);
    return true;
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
}

/**
 What making one run's scratch folder answered.
 */
export type MadeScratchFolder = {
  /**
   The run's scratch folder.
   */
  readonly folder: string;

  /**
   The `keep/` subfolder inside it, which survives the end of the run.
   */
  readonly keepFolder: string;

  /**
   Whether this start made the folder, or found it from an earlier run of the same name that is being resumed.
   */
  readonly made: boolean;
};

/**
 Says what is at one run's scratch folder's path that is not that run's folder, so a start refuses it rather than following it.

 Making a folder is the other half of emptying one: the emptying refuses a path that is a symbolic link, a file or anything else
 that is not a folder, and a maker that followed such a path would change the mode of a directory outside the scratch root and put a
 `keep/` inside it, if it is a link to a directory, and would fail with a raw `ENOENT` if the link points nowhere.

 @param scratchRoot - the root every run's scratch folder lives directly under

 @param runName - the run's name

 @returns what is there instead of the folder, with what to do about it, or nothing when the path may be made into the folder

 @example
 ```ts
 const obstacle = await scratchFolderObstacleIn({ scratchRoot: '/home/a/temp/agent', runName: 'copy-route', },);
 ```
 */
export async function scratchFolderObstacleIn({
  scratchRoot,
  runName,
}: {
  readonly scratchRoot: string;
  readonly runName: string;
},): Promise<{ readonly problem?: string }> {
  /**
   Why the run's name cannot name a folder, when it cannot.
   */
  const nameProblem = nameProblemOf(runName,);
  if (nameProblem.problem !== undefined) {
    return nameProblem;
  }
  /**
   The run's scratch folder.
   */
  const folder = scratchFolderOf({
    scratchRoot,
    runName,
  },);
  /**
   What is at that path, without following a symbolic link.
   */
  const atThePath = await lstatOrNothing(folder,);
  if (atThePath.shown === undefined) {
    return {};
  }
  /**
   What is at the path, which is there once the read above answered.
   */
  const { shown, } = atThePath;
  if (shown.isDirectory()) {
    return {};
  }
  /**
   Where the link points, when it is one that can be read, which a reader needs to know what to remove.
   */
  const target = shown.isSymbolicLink() ? await readlinkOrNothing(folder,) : '';
  return {
    problem: `${folder} is ${shown.isSymbolicLink() ? `a symbolic link${((target === '') ? '' : ` to ${target}`)}` : `not a folder (${shown.isFile() ? 'a file' : 'something else'})`}, so it is not the scratch folder this run would write in. Remove it, or empty it with empty-the-scratch-folder-of-a-run if it is an earlier run's, and start the run again; nothing of this start was made.`,
  };
}

/**
 Reads what is at one path, or answers nothing when nothing is there.

 @param path - the path to look at

 @returns what is there, without following a symbolic link, or nothing when nothing is

 @throws the error a read gave, other than a path that is not there

 @example
 ```ts
 const shown = await lstatOrNothing('/home/a/temp/agent/a-scratch');
 ```
 */
async function lstatOrNothing(path: string,): Promise<{ readonly shown?: Awaited<ReturnType<typeof lstat>> }> {
  try {
    return { shown: await lstat(path,), };
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return {};
    }
    throw error;
  }
}

/**
 Reads where one symbolic link points, or answers empty text when it cannot be read.

 @param path - the link

 @returns the path it names, or empty text

 @example
 ```ts
 const target = await readlinkOrNothing('/home/a/temp/agent/a-scratch');
 ```
 */
async function readlinkOrNothing(path: string,): Promise<string> {
  try {
    return await readlink(path,);
  }
  catch (error) {
    // A link whose target cannot be read is still a link, and the refusal above says so without naming where it points: what a
    // reader has to do is remove the link, which needs no target.
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return '';
    }
    throw error;
  }
}

/**
 Makes one run's scratch folder, with its `keep/` subfolder inside it, before the agent's first prompt names it.

 A folder an earlier run of the same name left is kept as it is: a resumed run gets the same folder again, with `keep/` as that
 earlier run left it.

 @param scratchRoot - the folder every run's scratch lives under, which is made when it is not there

 @param runName - the run's name

 @returns the folder, its `keep/` subfolder, and whether this start made the folder

 @throws ScratchFolderError when the run's name cannot name a folder, or when something that is not a folder is at the folder's
 path: a start refused that way has made nothing

 @example
 ```ts
 const made = await makeScratchFolder({ scratchRoot: '/home/a/temp/agent', runName: 'copy-route', },);
 ```
 */
export async function makeScratchFolder({
  scratchRoot,
  runName,
}: {
  readonly scratchRoot: string;
  readonly runName: string;
},): Promise<MadeScratchFolder> {
  /**
   What is at the folder's path instead of a folder, when something is: a link, a file, or a path the name cannot reach.
   */
  const obstacle = await scratchFolderObstacleIn({
    scratchRoot,
    runName,
  },);
  if (obstacle.problem !== undefined) {
    throw new ScratchFolderError(obstacle.problem,);
  }
  /**
   The run's scratch folder.
   */
  const folder = scratchFolderOf({
    scratchRoot,
    runName,
  },);
  /**
   Whether the folder was already there, which is what a resumed run finds.
   */
  const made = !await folderIsThere(folder,);
  await mkdir(
    scratchRoot,
    {
      recursive: true,
      mode: SCRATCH_FOLDER_MODE,
    },
  );
  await mkdir(
    folder,
    { recursive: true, },
  );
  // The mode of a made folder is the caller's, less the umask's, and rule `ATH` asks for a folder no other session reads.
  await chmod(
    folder,
    SCRATCH_FOLDER_MODE,
  );
  /**
   The folder inside it that survives the end of the run.
   */
  const keepFolder = join(
    folder,
    KEEP_FOLDER,
  );
  await mkdir(
    keepFolder,
    { recursive: true, },
  );
  await chmod(
    keepFolder,
    SCRATCH_FOLDER_MODE,
  );
  return {
    folder,
    keepFolder,
    made,
  };
}

/**
 The note every run's first prompt carries, so the agent learns where its scratch folder is from the prompt and from a command it
 runs, rather than from a path a brief asked it to build.

 @param folder - the run's scratch folder

 @returns the note, whole paragraphs, to put before what the brief says

 @example
 ```ts
 const note = scratchNoteOf({ folder: '/home/a/temp/agent/a-scratch', },);
 ```
 */
export function scratchNoteOf({ folder, }: { readonly folder: string },): string {
  return `${
    [
      `Scratch folder of this run: ${folder}`,
      'It is already there, with a `keep/` subfolder inside it. Everything you write outside your worktree goes in that folder.',
      'When this run ends, everything in it except `keep/` is deleted: put the report, the logs and the measurements a reader of the',
      `report needs in \`keep/\`, and nothing a build or a clone recreates. \`printenv ${
        SCRATCH_FOLDER_VARIABLE
      }\` prints this path.`,
    ].join('\n',)
  }\n\n`;
}
