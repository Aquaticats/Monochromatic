import { devNull, } from 'node:os';

import {
  commitAt,
  commitFileAt,
  type RepoCommit,
  writeAtRepo,
} from './cache-account-repo.test-fixture.ts';
import { spawnKeyless, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { REAL_GIT, } from '@monochromatic-dev/translation-repair-adjudication/ts/hermetic-git-run.test-fixture.ts';

//region Cache account merges
// MERGES IN THE THROWAWAY REPOSITORY of `cache-account-repo.test-fixture.ts`,
// for the cases where a cache version is set by a merge's resolution or only
// brought in by it (ledger B814).
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Git runs hermetic against user and system
// configuration.

/**
 One file written whole, with where it goes.

 @example
 ```ts
 const write: FileWrite = { path: 'src/nap.ts', text: 'export const NAP = 3;\n', };
 ```
 */
type FileWrite = {
  /**
   Path from the repository's top.
   */
  readonly path: string;

  /**
   File's text.
   */
  readonly text: string;
};

/**
 Runs one git command against the repository that makes no commit, hermetic
 against user and system configuration. A merge asks for an identity even when
 it stops before its commit, so the command carries the one every commit here
 is made under.

 @param cloneDir - repository directory

 @param args - arguments after the repository option

 @example
 ```ts
 await gitAt({ cloneDir, args: ['switch', '--create', 'side',], },);
 ```
 */
async function gitAt(
  {
    cloneDir,
    args,
  }: {
    readonly cloneDir: string;
    readonly args: readonly string[];
  },
): Promise<void> {
  await spawnKeyless({
    file: REAL_GIT,
    args: [
      '-C',
      cloneDir,
      ...args,
    ],
    extra: {
      GIT_CONFIG_GLOBAL: devNull,
      GIT_CONFIG_SYSTEM: devNull,
      GIT_AUTHOR_NAME: 'Tabby',
      GIT_AUTHOR_EMAIL: 'tabby@example.invalid',
      GIT_COMMITTER_NAME: 'Tabby',
      GIT_COMMITTER_EMAIL: 'tabby@example.invalid',
    },
  },);
}

/**
 Commits one file on a side branch, then merges the branch back with a merge
 commit, writing the resolution's files before the merge commits. A merge
 whose resolution writes a cache version neither side holds is how a value
 comes to stand in no ordinary commit: two branches that each took the next
 number, met by a merge that took the one after (ledger B814). With no
 resolution files the merge only brings the side branch in.

 @param cloneDir - repository directory

 @param day - days after the first commit's time for the side commit; the
 merge commits a day later

 @param side - file the side branch commits

 @param resolution - files the merge writes over what it merged, none for a
 plain merge

 @param subject - merge commit's subject

 @returns The side commit and the merge commit

 @example
 ```ts
 const { merge, } = await commitMergeAt({ cloneDir, day: 5, side, resolution: [], subject: 'merge the nap', },);
 ```
 */
export async function commitMergeAt(
  {
    cloneDir,
    day,
    side,
    resolution,
    subject,
  }: {
    readonly cloneDir: string;
    readonly day: number;
    readonly side: FileWrite;
    readonly resolution: readonly FileWrite[];
    readonly subject: string;
  },
): Promise<{
  readonly sideCommit: RepoCommit;
  readonly merge: RepoCommit;
}> {
  await gitAt({
    cloneDir,
    args: [
      'switch',
      '--quiet',
      '--create',
      'side',
    ],
  },);
  /**
   The side branch's one commit.
   */
  const sideCommit = await commitFileAt({
    cloneDir,
    day,
    path: side.path,
    text: side.text,
    subject: `${subject} (side)`,
  },);
  await gitAt({
    cloneDir,
    args: [
      'switch',
      '--quiet',
      '-',
    ],
  },);
  await gitAt({
    cloneDir,
    args: [
      'merge',
      '--quiet',
      '--no-commit',
      '--no-ff',
      'side',
    ],
  },);
  await Promise.all(resolution.map(async function written(write,): Promise<void> {
    await writeAtRepo({
      cloneDir,
      path: write.path,
      text: write.text,
    },);
  },),);
  /**
   The merge, concluded by the commit.
   */
  const merge = await commitAt({
    cloneDir,
    day: day + 1,
    subject,
  },);
  return {
    sideCommit,
    merge,
  };
}

//endregion Cache account merges
