/**
 Reads of one install fixture's checkout: what a step wrote, what a commit holds, and what is left staged.

 These are the questions a test asks after an install ran, kept beside the fixture that makes the checkout
 (`stand-in-install-programs-for-tests.ts`) and apart from it, so the stand-in programs and the fixture stay one file of
 their own. The Git these reads run is `fixture-git-that-is-never-the-wrapper-for-tests.ts`, which owns that rule for
 every fixture of this package.
 */
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

import { fixtureGit } from './fixture-git-that-is-never-the-wrapper-for-tests.ts';

/**
 The commits one fixture checkout holds, newest first.

 @param repository - the checkout to read

 @returns every commit's subject

 @example
 ```ts
 await commitSubjects({ repository: fixture.repository });
 ```
 */
export async function commitSubjects({ repository }: { readonly repository: string; }): Promise<readonly string[]> {
  return (await fixtureGit({
    directory: repository,
    args: [
      'log',
      '--format=%s',
    ],
  })).split('\n')
    .filter(function hasText(line) {
      return line !== '';
    });
}

/**
 The text of one file as the last commit holds it, which is what a release commit of the lockfile has to name.

 @param repository - the checkout to read

 @param name - the file's name relative to the checkout

 @returns its text in `HEAD`

 @example
 ```ts
 await committedFileText({ repository: fixture.repository, name: 'toolchain.lock.json' });
 ```
 */
export async function committedFileText({
  repository,
  name,
}: {
  readonly repository: string;
  readonly name: string;
}): Promise<string> {
  return await fixtureGit({
    directory: repository,
    args: [
      'show',
      `HEAD:${name}`,
    ],
  });
}

/**
 The paths a commit made now would record from the index, which is how a test reads that nothing is left staged.

 @param repository - the checkout to read

 @returns the path names `git diff --cached` names, empty when nothing is staged

 @example
 ```ts
 await stagedPathNames({ repository: fixture.repository });
 ```
 */
export async function stagedPathNames({ repository }: { readonly repository: string; }): Promise<readonly string[]> {
  return (await fixtureGit({
    directory: repository,
    args: [
      'diff',
      '--cached',
      '--name-only',
    ],
  })).split('\n')
    .filter(function hasText(line) {
      return line !== '';
    });
}

/**
 The text of one file of a fixture checkout, which is how a test reads what a step wrote.

 @param repository - the checkout to read

 @param name - the file's name relative to the checkout

 @returns its text

 @example
 ```ts
 await fileTextOf({ repository: fixture.repository, name: 'toolchain.lock.json' });
 ```
 */
export async function fileTextOf({
  repository,
  name,
}: {
  readonly repository: string;
  readonly name: string;
}): Promise<string> {
  return await readFile(
    join(
      repository,
      name
    ),
    'utf8'
  );
}
