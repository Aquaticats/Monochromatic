import { posix, } from 'node:path';

import spawn from 'nano-spawn';
import { childEnvironment, } from '@monochromatic-dev/translation-repair-typography/ts';

import { resolveGit, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Coverage census commit
// Ledger T8: what git says about the tree a census reads, apart from the
// entry so each question is tested against a throwaway repository: which
// commit the build came from and whether the files match it, and which files
// of the work tree have changed since an earlier census's commit.

/**
 The commit the package's files were built from, and whether they match it.

 @param packageDirectory - package directory, inside a git work tree

 @returns Nine-character hash and whether `git status` shows no change under the package

 @example
 ```ts
 const { head, clean, } = await packageCommit({ packageDirectory, },);
 ```
 */
export async function packageCommit({ packageDirectory, }: { readonly packageDirectory: string; },): Promise<{
  readonly head: string;
  readonly clean: boolean;
}> {
  /**
   Git to run.
   */
  const git = await resolveGit();
  /**
   The commit.
   */
  const { stdout: head, } = await spawn(
    git,
    [
      '-C',
      packageDirectory,
      'rev-parse',
      '--short=9',
      'HEAD',
    ],
    { env: childEnvironment({ parent: process.env, },), },
  );
  /**
   Changes under the package.
   */
  const { stdout: changes, } = await spawn(
    git,
    [
      '-C',
      packageDirectory,
      'status',
      '--porcelain',
      '--',
      '.',
    ],
    { env: childEnvironment({ parent: process.env, },), },
  );
  return {
    head: head.trim(),
    clean: changes.trim() === '',
  };
}

/**
 Names the files of the work tree that differ from an earlier commit, as a
 census names sources: relative to the package, so another package's file
 reads `../<package>/src/<file>.ts`, as its stretches do.

 THE WHOLE WORK TREE, since the census reads the sources of other packages
 the suite loads as well as the package's own, and matches their baseline
 stretches by line too. Asked about the package alone, git never named an
 edit to one of them (ledger B62). A reading asks after the sources it
 reads, so naming every changed file costs it nothing.

 THE TREE AS IT STANDS, uncommitted edits included, against that commit, since
 the census this reads beside was taken from the tree as it stands. A file
 added since that commit counts as changed; one never committed is not listed
 by git, and no census taken at that commit can hold a stretch in it.

 NUL-SEPARATED (`-z`), which git prints verbatim: read line by line, a name
 git quotes (one holding a double quote, a newline or a non-ASCII letter)
 came back quoted and matched no source (ledger B62).

 @param packageDirectory - package directory, inside a git work tree

 @param head - earlier commit, as a census records it

 @returns Package-relative paths, as a census names sources, of those that
 changed

 @example
 ```ts
 const edited = await sourcesEditedSince({ packageDirectory, head: baseline.head, },);
 ```
 */
export async function sourcesEditedSince(
  {
    packageDirectory,
    head,
  }: {
    readonly packageDirectory: string;
    readonly head: string;
  },
): Promise<ReadonlySet<string>> {
  /**
   Git to run.
   */
  const git = await resolveGit();
  /**
   Where the package sits under the work tree's root, ending in `/`, or
   empty at the root.
   */
  const { stdout: prefix, } = await spawn(
    git,
    [
      '-C',
      packageDirectory,
      'rev-parse',
      '--show-prefix',
    ],
    { env: childEnvironment({ parent: process.env, },), },
  );
  /**
   Changed paths relative to the work tree's root, each ended by NUL.
   */
  const { stdout: changed, } = await spawn(
    git,
    [
      '-C',
      packageDirectory,
      'diff',
      '--name-only',
      '-z',
      head,
    ],
    { env: childEnvironment({ parent: process.env, },), },
  );
  return new Set(changed
    .split('\0',)
    .filter(function named(path,): boolean {
      return path !== '';
    },)
    .map(function fromPackage(path,): string {
      return posix.relative(
        prefix,
        path,
      );
    },),);
}

//endregion Coverage census commit
