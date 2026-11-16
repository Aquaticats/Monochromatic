/**
 The checkout a command of this package was run in, found by walking up from the folder it was started in to the first folder a
 checkout marks, and then reading which repository that folder belongs to.

 Git itself cannot answer this here: this repository's Git wrapper refuses every command whose working folder is not the root of
 the repository (`require-root`), and a command of this package is run from the package's own folder, so asking git for the root
 from there fails. On 2026-11-11 that refusal ended every start of a batch: the launcher printed an uncaught error, and the batch
 kept only the last line of it. Walking up costs one read per folder and needs no git at all.

 A linked worktree marks itself with a file rather than a folder, and that file names the Git directory it was given, which is a
 folder inside the repository's own Git directory. Every command of this package therefore answers the repository, from the
 repository or from any of its worktrees: a worktree that answered itself would put a run's worktree inside a worktree, and a
 checkout default read in one place would name a different folder than the same default read in another.

 Where a run's unit is told which Git directories it may write, the files are not walked at all: `gitFoldersOf` asks Git itself,
 because a Git directory kept elsewhere (`git init --separate-git-dir`, a submodule) carries no `commondir` to read, and a layout
 that is guessed is a folder granted by mistake. A layout the walk meets and does not know is refused, never guessed.

 @module
 */

import {
  access,
  stat,
} from 'node:fs/promises';
import {
  dirname,
  isAbsolute,
  join,
  resolve,
  sep,
} from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import type { GitRunner, } from './worktree-of-a-run.ts';
import {
  NotAnOrdinaryFileError,
  readOrdinaryFile,
} from './read-ordinary-file.ts';

/**
 The name that marks the folder a checkout starts at, which is a folder in the main checkout and a file in a worktree made from it.
 */
export const CHECKOUT_MARKER: string = '.git';

/**
 The prefix of the line a worktree's marker file holds, which names the Git directory that worktree was given.
 */
const GITDIR_PREFIX = 'gitdir:';

/**
 The file inside a worktree's Git directory that names the Git directory every worktree of the repository shares.
 */
const COMMONDIR_FILE_NAME = 'commondir';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 No folder above the one a command was started in is a checkout, so nothing says where a run's worktree belongs.
 */
export class CheckoutNotFoundError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'CheckoutNotFoundError';
}

/**
 Says whether a path is there, without reading it.

 @param path - the path to look for

 @returns whether it is there

 @example
 ```ts
 const there = await isThere('/repo/.git');
 ```
 */
async function isThere(path: string,): Promise<boolean> {
  try {
    await access(path,);
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
 Finds the nearest folder that marks itself as a checkout, or the one above it, and so on to the filesystem root.

 Each step is one folder nearer the root, so this walks a path's length of folders and no more.

 @param folder - the folder being looked at

 @param from - the folder the command was started in, for the message

 @returns the nearest folder holding {@link CHECKOUT_MARKER}

 @throws CheckoutNotFoundError when no folder up to the filesystem root holds one

 @example
 ```ts
 const marked = await markedFolderOf({ folder: '/repo/package/x', from: '/repo/package/x' });
 ```
 */
async function markedFolderOf({
  folder,
  from,
}: {
  readonly folder: string;
  readonly from: string;
},): Promise<string> {
  if (await isThere(join(
    folder,
    CHECKOUT_MARKER,
  ),)) {
    return folder;
  }
  /**
   The folder one step nearer the filesystem root.
   */
  const above = dirname(folder,);
  if (above === folder) {
    throw new CheckoutNotFoundError(
      `no folder from ${from} up to ${folder} holds a ${CHECKOUT_MARKER}, so which checkout a run's worktree belongs in cannot be read. Run the launcher from inside the checkout, or name the checkout with --checkout-folder.`,
    );
  }
  return markedFolderOf({
    folder: above,
    from,
  },);
}

/**
 Reads the Git directory a worktree's marker file names.

 @param marker - the marker file

 @param markedFolder - the folder holding it, which a relative path in the file is read from

 @returns the Git directory's path

 @throws CheckoutNotFoundError when the file names no Git directory

 @example
 ```ts
 const gitDirectory = await gitDirectoryOf({ marker: '/repo/pi-a/.git', markedFolder: '/repo/pi-a', },);
 ```
 */
async function gitDirectoryOf({
  marker,
  markedFolder,
}: {
  readonly marker: string;
  readonly markedFolder: string;
},): Promise<string> {
  /**
   The line the marker file holds.
   */
  const named = (await ordinaryFileAt({ path: marker, },)).split('\n',)
    .map(function withoutTrailingSpace(line,) {
      return line.trim();
    },)
    .find(function namesAGitDirectory(line,) {
      return line.startsWith(GITDIR_PREFIX,);
    },);
  if (named === undefined) {
    throw new CheckoutNotFoundError(
      `${marker} is a file that names no ${GITDIR_PREFIX} path, so which repository ${markedFolder} belongs to cannot be read. Name the checkout with --checkout-folder.`,
    );
  }
  return resolve(
    markedFolder,
    named.slice(GITDIR_PREFIX.length,)
      .trim(),
  );
}

/**
 Reads a file Git wrote, which a named pipe or a device cannot make this process wait on.

 @param path - the file Git wrote

 @returns the file's text

 @throws CheckoutNotFoundError when the path is not an ordinary file, naming it and what it holds

 @throws the error the read gives, such as a missing file

 @example
 ```ts
 const text = await ordinaryFileAt({ path: '/repo/.git/worktrees/pi-a/commondir', });
 ```
 */
async function ordinaryFileAt({ path }: { readonly path: string },): Promise<string> {
  try {
    return await readOrdinaryFile(path,);
  }
  catch (error) {
    if (!(error instanceof NotAnOrdinaryFileError)) {
      throw error;
    }
    throw new CheckoutNotFoundError(
      `${path} is not an ordinary file, so what Git wrote there cannot be read without waiting for a writer that may never come. Replace it with the file Git writes, or name another checkout with --checkout-folder. ${String(error,)}`,
      { cause: error, },
    );
  }
}

/**
 Reads the Git directory every worktree of a repository shares, from the file Git writes for a linked worktree, and refuses a
 Git directory that names none.

 Why it refuses rather than answers: a Git directory with no `commondir` is a checkout whose Git directory is kept elsewhere
 (`git init --separate-git-dir`) or a submodule, and this walk has no way to name the repository those belong to; two folders
 above the Git directory is the layout of a linked worktree only, and answering it for the others names a folder nobody asked
 about. Where this walk is used, to find the repository a command runs in, the answer is the checkout the caller names instead.

 @param gitDirectory - the Git directory of the worktree, or of the repository itself

 @returns the shared Git directory's path

 @throws CheckoutNotFoundError when the Git directory names no shared directory, which is a layout this walk does not know

 @example
 ```ts
 const common = await commonDirectoryOf({ gitDirectory: '/repo/.git/worktrees/pi-a', },);
 ```
 */
async function commonDirectoryOf({ gitDirectory, }: { readonly gitDirectory: string },): Promise<string> {
  /**
   The file naming the shared Git directory, which only a linked worktree's Git directory has.
   */
  const named = join(
    gitDirectory,
    COMMONDIR_FILE_NAME,
  );
  if (!(await isThere(named,))) {
    throw new CheckoutNotFoundError(
      `${gitDirectory} names no ${COMMONDIR_FILE_NAME}, so it belongs to no linked worktree as this walk reads them: a checkout whose Git directory is kept elsewhere and a submodule both keep theirs without one. Name the checkout with --checkout-folder, or run the command from inside it.`,
    );
  }
  return resolve(
    gitDirectory,
    (await ordinaryFileAt({ path: named, },)).trim(),
  );
}

/**
 Says whether the marker of a checkout is the folder a repository holds, rather than the file a linked worktree holds.

 @param marker - the marker itself

 @param folder - the folder the marker was looked for in, for the message

 @returns whether the marker is a folder

 @throws CheckoutNotFoundError when the folder holds no marker

 @example
 ```ts
 const isAFolder = await markerIsAFolder({ marker: '/repo/.git', folder: '/repo', },);
 ```
 */
async function markerIsAFolder({
  marker,
  folder,
}: {
  readonly marker: string;
  readonly folder: string;
},): Promise<boolean> {
  try {
    return (await stat(marker,)).isDirectory();
  }
  catch (error) {
    if (errorCodeOf(error,) !== MISSING_PATH_CODE) {
      throw error;
    }
    throw new CheckoutNotFoundError(
      `${folder} holds no ${CHECKOUT_MARKER}, so the Git directories a run's commits write cannot be read. Run the command from inside the checkout, or name the checkout with --checkout-folder.`,
    );
  }
}

/**
 Reads the Git directory of the checkout at a folder from the files Git writes there, and the Git directory every worktree of the
 repository shares.

 This walk is what finds the repository a command was started in, where asking Git is the thing the `require-root` guard refuses.
 A unit's writable folders are asked of Git instead, by {@link gitFoldersOf}.

 @param folder - the checkout or the linked worktree to read, which must be its own root

 @returns the Git directory of that folder, and the one every worktree of its repository shares

 @throws CheckoutNotFoundError when the folder marks no checkout, or when the marker names a layout this walk does not know

 @example
 ```ts
 const folders = await gitFoldersTheFilesName({ folder: '/repo/.claude/worktrees/pi-a', },);
 ```
 */
export async function gitFoldersTheFilesName({ folder }: { readonly folder: string },): Promise<{
  readonly gitDirectory: string;
  readonly commonDirectory: string;
}> {
  /**
   The marker of the folder named, which is a folder in a repository and a file in a linked worktree.
   */
  const marker = join(
    folder,
    CHECKOUT_MARKER,
  );
  if (await markerIsAFolder({
    marker,
    folder,
  },)) {
    return {
      gitDirectory: marker,
      commonDirectory: marker,
    };
  }
  /**
   The Git directory this worktree was given, inside the repository's own.
   */
  const gitDirectory = await gitDirectoryOf({
    marker,
    markedFolder: folder,
  },);
  return {
    gitDirectory,
    commonDirectory: await commonDirectoryOf({ gitDirectory, },),
  };
}

/**
 A Git folder an answer named that a start cannot make writable, because it holds the checkout itself.

 @example
 ```ts
 throw new GitFolderAboveTheCheckoutError("the folder /repo is above the checkout",);
 ```
 */
export class GitFolderAboveTheCheckoutError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'GitFolderAboveTheCheckoutError';
}

/**
 A Git question about a checkout's folders that Git did not answer as this package needs it to.

 @example
 ```ts
 throw new GitFoldersUnreadableError("git rev-parse answered nothing",);
 ```
 */
export class GitFoldersUnreadableError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'GitFoldersUnreadableError';
}

/**
 Asks Git for one of a checkout's Git directories, and takes off only that answer's own record terminator.

 Why one question per path: Git prints a pathname as it is, and a pathname may hold a newline, so two answers in one stream cannot be
 told apart from one answer with a newline in it. A record terminator is the newline Git ends an answer with, and only that one is
 removed: trimming would eat a pathname's own leading or trailing space, and splitting would cut a pathname's own newline in two.

 @param git - how one git question is run

 @param folder - the checkout to ask about, which must be its own root

 @param word - the `git rev-parse` option that names this path, such as `--git-common-dir`

 @param what - what that path is, in the words a refusal prints

 @returns the path Git named, made absolute, with its own trailing spaces and newlines kept

 @throws GitFoldersUnreadableError when Git does not answer, or answers with something that is not one absolute path

 @example
 ```ts
 const gitDirectory = await gitPathFromAnswer({ git, folder: '/repo', word: '--absolute-git-dir', what: 'the Git directory of the checkout', });
 ```
 */
export async function gitPathFromAnswer({
  git,
  folder,
  word,
  what,
}: {
  /**
   How one git question is run.
   */
  readonly git: GitRunner;

  /**
   The checkout to ask about, which must be its own root.
   */
  readonly folder: string;

  /**
   The `git rev-parse` option that names this path.
   */
  readonly word: string;

  /**
   What that path is, in the words a refusal prints.
   */
  readonly what: string;
},): Promise<string> {
  /**
   What git answered about that one path.
   */
  const answered = await git({
    folder,
    words: [
      'rev-parse',
      '--path-format=absolute',
      word,
    ],
  },);
  if (answered.status !== 0) {
    throw new GitFoldersUnreadableError(
      `git rev-parse ${word} asked in ${folder} for ${what} answered ${String(answered.status,)}${((answered.problem ?? answered.stderr).trim() === '') ? '' : `: ${(answered.problem ?? answered.stderr).trim()}`}, so a run's unit cannot be told which folders its commits write. Name a checkout Git can describe with --checkout-folder.`,
    );
  }
  /**
   Git's answer without the single newline it ends an answer with, and without anything else.
   */
  const answeredPath = answered.stdout
    .endsWith('\n')
    ? answered.stdout
      .slice(
      0,
      -1,
      )
    : answered.stdout;
  if ((answeredPath === '') || (!isAbsolute(answeredPath,))) {
    throw new GitFoldersUnreadableError(
      `git rev-parse ${word} asked in ${folder} for ${what} answered ${JSON.stringify(answered.stdout,)}, which is not one absolute path, so a run's unit cannot be told which folders its commits write. Name a checkout Git can describe with --checkout-folder.`,
    );
  }
  return resolve(answeredPath,);
}

/**
 Asks Git for the Git directory of the checkout at a folder, and the Git directory every worktree of the repository shares.

 Why Git and not the files: a Git directory kept elsewhere (`git init --separate-git-dir`) and a submodule keep theirs without a
 `commondir`, so a walk that reads files either guesses and grants a folder nobody asked about, or refuses a layout Git knows.
 Git's own answers are the ones a run's commits, fetches and pushes write to.

 @param git - how one git question is run

 @param folder - the checkout to ask about, which must be its own root

 @returns the Git directory of that folder, and the one every worktree of its repository shares

 @throws GitFoldersUnreadableError when Git does not answer, or answers with something that is not an absolute path

 @throws GitFolderAboveTheCheckoutError when an answer is a folder that holds the checkout, which a start must not make writable

 @example
 ```ts
 const folders = await gitFoldersOf({ git, folder: '/repo', },);
 ```
 */
export async function gitFoldersOf({
  git,
  folder,
}: {
  /**
   How one git question is run.
   */
  readonly git: GitRunner;

  /**
   The checkout to ask about, which must be its own root.
   */
  readonly folder: string;
},): Promise<{
  readonly gitDirectory: string;
  readonly commonDirectory: string;
}> {
  /**
   The Git directory of the checkout, asked for on its own.

   One path per question, and only that question's own record terminator taken off: a Git pathname may hold a newline, so an
   answer split into lines turned one checkout into several and refused it. Asking twice costs one more `git rev-parse`, which
   answers from the checkout itself rather than from a walk of the files.
   */
  const gitDirectory = await gitPathFromAnswer({
    git,
    folder,
    word: '--absolute-git-dir',
    what: 'the Git directory of the checkout',
  },);
  /**
   The Git directory every worktree of the repository shares, asked for on its own for the same reason.
   */
  const commonDirectory = await gitPathFromAnswer({
    git,
    folder,
    word: '--git-common-dir',
    what: 'the Git directory every worktree of the repository shares',
  },);
  for (const answeredFolder of [
    gitDirectory,
    commonDirectory,
  ]) {
    /**
     Whether that folder holds the checkout, which would be granted writable and make the checkout writable with it.
     */
    const aboveTheCheckout = (answeredFolder !== folder)
      && (resolve(folder,)
        .startsWith(`${answeredFolder}${sep}`,));
    if (aboveTheCheckout) {
      throw new GitFolderAboveTheCheckoutError(
        `Git names ${answeredFolder} as a Git directory of ${folder}, and that folder holds the checkout, so a unit given it writable would make the checkout writable with it. Name the checkout with --checkout-folder, or unset GIT_DIR if it names a repository above it.`,
      );
    }
  }
  return {
    gitDirectory,
    commonDirectory,
  };
}

/**
 Finds the repository the checkout a command was started in belongs to.

 @param from - the folder the command was started in

 @returns the repository's folder, which is the folder holding the Git directory every worktree of it shares

 @throws CheckoutNotFoundError when no folder from there up to the filesystem root is a checkout

 @example
 ```ts
 const repository = await checkoutFolderOf({ from: '/repo/.claude/worktrees/pi-a/package/x', },);
 ```
 */
export async function checkoutFolderOf({ from }: { readonly from: string },): Promise<string> {
  /**
   The nearest folder that marks itself as a checkout, which is the worktree when a command is run inside one.
   */
  const markedFolder = await markedFolderOf({
    folder: resolve(from,),
    from,
  },);
  /**
   The marker itself, which is a folder in a repository and a file in a linked worktree.
   */
  const marker = join(
    markedFolder,
    CHECKOUT_MARKER,
  );
  if ((await stat(marker,)).isDirectory()) {
    return markedFolder;
  }
  /**
   The Git directory this worktree was given, inside the repository's own.
   */
  const gitDirectory = await gitDirectoryOf({
    marker,
    markedFolder,
  },);
  return dirname(await commonDirectoryOf({ gitDirectory, },),);
}
