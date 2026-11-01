import {
  mkdir,
  rename,
  utimes,
  writeFile,
} from 'node:fs/promises';
import { devNull, } from 'node:os';
import {
  dirname,
  join,
} from 'node:path';

import { citedHash, } from '../../dist/final/node/index.mjs';
import { spawnKeyless, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { REAL_GIT, } from '@monochromatic-dev/translation-repair-adjudication/ts/hermetic-git-run.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Cache account repository
// A THROWAWAY REPOSITORY the pre-launch cache check can read, built commit by
// commit with fixed committer times, so every time and every order it prints
// is known before it runs.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Git runs hermetic against user and system
// configuration, and the repository is removed when its scope ends.

/**
 One commit of the repository.

 @example
 ```ts
 const commit: RepoCommit = { hash: 'f6e93ed5f0000000000000000000000000000000', seconds: 1_788_000_000, subject: 'teach Tabby a longer nap', };
 ```
 */
export type RepoCommit = {
  /**
   Full object id.
   */
  readonly hash: string;

  /**
   Committer time in unix seconds.
   */
  readonly seconds: number;

  /**
   Subject line.
   */
  readonly subject: string;
};

/**
 The repository and what its history holds.

 @example
 ```ts
 await using repo = await makeCacheAccountRepo({ versions: 'two', },);
 ```
 */
type RepoHistory = {
  /**
   The commit setting the stage version, which opens the history.
   */
  readonly stageSet: RepoCommit;

  /**
   The commit editing a source file, which the pairing version's account names.
   */
  readonly napEdited: RepoCommit;

  /**
   The commit setting the pairing version, which only a repository of both
   versions holds.
   */
  readonly pairingSet: readonly RepoCommit[];

  /**
   The commit adding a source file nobody accounts for.
   */
  readonly purrAdded: RepoCommit;

  /**
   Time the newest record of a slice cache is given to fall between the
   pairing version's commit and the edit before it, in unix seconds.
   */
  readonly betweenSeconds: number;
};

/**
 The repository: its history, its top directory and its removal.

 @example
 ```ts
 await using repo = await makeCacheAccountRepo({ versions: 'two', },);
 ```
 */
type CacheAccountRepo = RepoHistory & AsyncDisposable & {
  /**
   Top directory of the repository, which is also its source's parent.
   */
  readonly path: string;
};

/**
 Milliseconds in a second.
 */
const MS_PER_SECOND = 1_000;

/**
 Seconds in a day.
 */
const DAY_SECONDS = 86_400;

/**
 Committer time of the first commit, 2026-09-01T10:00:00Z.
 */
const FIRST_COMMIT_SECONDS = Date.parse('2026-09-01T10:00:00Z',) / MS_PER_SECOND;

/**
 Commits one change at a fixed time, hermetic against user and system git
 configuration.

 @param cloneDir - repository directory

 @param day - days after the first commit's time

 @param subject - commit subject

 @returns The commit made

 @example
 ```ts
 const commit = await commitAt({ cloneDir, day: 1, subject: 'teach Tabby a longer nap', },);
 ```
 */
export async function commitAt(
  {
    cloneDir,
    day,
    subject,
  }: {
    readonly cloneDir: string;
    readonly day: number;
    readonly subject: string;
  },
): Promise<RepoCommit> {
  /**
   The commit's time in unix seconds.
   */
  const seconds = FIRST_COMMIT_SECONDS + (day * DAY_SECONDS);

  /**
   The same, as git reads a date.
   */
  const stamp = `${String(seconds,)} +0000`;

  /**
   Variables fixing identity, time and configuration.
   */
  const env = {
    GIT_CONFIG_GLOBAL: devNull,
    GIT_CONFIG_SYSTEM: devNull,
    GIT_AUTHOR_NAME: 'Tabby',
    GIT_AUTHOR_EMAIL: 'tabby@example.invalid',
    GIT_COMMITTER_NAME: 'Tabby',
    GIT_COMMITTER_EMAIL: 'tabby@example.invalid',
    GIT_AUTHOR_DATE: stamp,
    GIT_COMMITTER_DATE: stamp,
  };
  await spawnKeyless({
    file: REAL_GIT,
    args: [
      '-C',
      cloneDir,
      'add',
      '--all',
    ],
    extra: env,
  },);
  await spawnKeyless({
    file: REAL_GIT,
    args: [
      '-C',
      cloneDir,
      'commit',
      '--message',
      subject,
    ],
    extra: env,
  },);

  /**
   The new commit's id.
   */
  const { stdout, } = await spawnKeyless({
    file: REAL_GIT,
    args: [
      '-C',
      cloneDir,
      'rev-parse',
      'HEAD',
    ],
    extra: env,
  },);
  return {
    hash: stdout.trim(),
    seconds,
    subject,
  };
}

/**
 Writes a file under the repository, with its parent directories.

 @param cloneDir - repository directory

 @param path - path from the repository's top

 @param text - file's text

 @example
 ```ts
 await writeAtRepo({ cloneDir, path: 'src/nap.ts', text: 'export const NAP = 1;\n', },);
 ```
 */
export async function writeAtRepo(
  {
    cloneDir,
    path,
    text,
  }: {
    readonly cloneDir: string;
    readonly path: string;
    readonly text: string;
  },
): Promise<void> {
  /**
   Where the file goes.
   */
  const target = join(
    cloneDir,
    path,
  );
  await mkdir(
    dirname(target,),
    { recursive: true, },
  );
  await writeFile(
    target,
    text,
    'utf8',
  );
}

/**
 Writes one file into a repository and commits it at a fixed time, for a case
 that needs a history of its own beside the one the repository is built with.

 @param cloneDir - repository directory

 @param day - days after the first commit's time

 @param path - path from the repository's top

 @param text - file's text

 @param subject - commit subject

 @returns The commit made

 @example
 ```ts
 const commit = await commitFileAt({ cloneDir, day: 5, path: 'src/nap.ts', text: 'export const NAP = 3;\\n', subject: 'nap again', },);
 ```
 */
export async function commitFileAt(
  {
    cloneDir,
    day,
    path,
    text,
    subject,
  }: {
    readonly cloneDir: string;
    readonly day: number;
    readonly path: string;
    readonly text: string;
    readonly subject: string;
  },
): Promise<RepoCommit> {
  await writeAtRepo({
    cloneDir,
    path,
    text,
  },);
  return await commitAt({
    cloneDir,
    day,
    subject,
  },);
}

/**
 Moves one tracked file to another path and commits the move at a fixed time,
 the way the split moved the translation-repair sources into the family's
 packages: the content is unchanged, so git reads the commit as a rename.

 @param cloneDir - repository directory

 @param day - days after the first commit's time

 @param from - path from the repository's top the file leaves

 @param to - path from the repository's top the file arrives at

 @param subject - commit subject

 @returns The commit made

 @example
 ```ts
 const moved = await moveFileAt({ cloneDir, day: 6, from: 'src/nap.ts', to: 'package/nap/src/nap.ts', subject: 'move the nap', },);
 ```
 */
export async function moveFileAt(
  {
    cloneDir,
    day,
    from,
    to,
    subject,
  }: {
    readonly cloneDir: string;
    readonly day: number;
    readonly from: string;
    readonly to: string;
    readonly subject: string;
  },
): Promise<RepoCommit> {
  await mkdir(
    dirname(join(
      cloneDir,
      to,
    ),),
    { recursive: true, },
  );
  await rename(
    join(
      cloneDir,
      from,
    ),
    join(
      cloneDir,
      to,
    ),
  );
  return await commitAt({
    cloneDir,
    day,
    subject,
  },);
}

/**
 Builds the repository: the stage version set, a source file edited, the
 pairing version set by a file whose account names that edit, a source file
 added that no account names, and a test file added, which the check leaves
 out.

 @param versions - which cache versions the history sets: none, the stage
 version alone, or the stage version and then the pairing version; where the
 pairing version is not set the edit stays unaccounted

 @returns The repository, removed when its scope ends

 @example
 ```ts
 await using repo = await makeCacheAccountRepo({ versions: 'two', },);
 ```
 */
export async function makeCacheAccountRepo(
  { versions, }: { readonly versions: 'none' | 'one' | 'two'; },
): Promise<CacheAccountRepo> {
  return await scratchDirWith({
    prefix: 'cache-account-repo-',
    setup: async function seeded({ path: cloneDir, },): Promise<RepoHistory> {
      await spawnKeyless({
        file: REAL_GIT,
        args: [
          '-C',
          cloneDir,
          'init',
          '--quiet',
        ],
        extra: {
          GIT_CONFIG_GLOBAL: devNull,
          GIT_CONFIG_SYSTEM: devNull,
        },
      },);
      if (versions !== 'none') {
        await writeAtRepo({
          cloneDir,
          path: 'src/stage-cache-version.ts',
          text: 'export const STAGE_CACHE_VERSION = 1;\n',
        },);
      }
      await writeAtRepo({
        cloneDir,
        path: 'src/nap.ts',
        text: 'export const NAP = 1;\n',
      },);
      /**
       The commit setting the first version, or the first commit.
       */
      const stageSet = await commitAt({
        cloneDir,
        day: 0,
        subject: 'set the stage cache version',
      },);
      await writeAtRepo({
        cloneDir,
        path: 'src/nap.ts',
        text: 'export const NAP = 2;\n',
      },);
      /**
       The commit editing a source file.
       */
      const napEdited = await commitAt({
        cloneDir,
        day: 1,
        subject: 'teach Tabby a longer nap',
      },);
      if (versions === 'two') {
        await writeAtRepo({
          cloneDir,
          path: 'src/pairing-cache-version.ts',
          text: `// Commit ${citedHash({ hash: napEdited.hash, },)} lengthened the nap, which the pairing does not read.\nconst PAIRING_CACHE_VERSION = 4;\n`,
        },);
      }
      /**
       The commit setting the second version, when the history has one.
       */
      const pairingSet = (versions === 'two')
        ? [await commitAt({
          cloneDir,
          day: 2,
          subject: 'set the pairing cache version',
        },),]
        : [];
      await writeAtRepo({
        cloneDir,
        path: 'src/purr.ts',
        text: 'export const PURR = 1;\n',
      },);
      /**
       The commit adding a source file.
       */
      const purrAdded = await commitAt({
        cloneDir,
        day: 3,
        subject: 'teach Tabby to purr',
      },);
      await writeAtRepo({
        cloneDir,
        path: 'src/purr.test.ts',
        text: 'export const PURR_TEST_CACHE_VERSION = 7;\n',
      },);
      await commitAt({
        cloneDir,
        day: 4,
        subject: 'test the purr',
      },);
      return {
        stageSet,
        napEdited,
        pairingSet,
        purrAdded,
        betweenSeconds: napEdited.seconds + (DAY_SECONDS / 2),
      };
    },
  },);
}

/**
 Writes one slice-cache record of an entry under a runs directory and sets
 its modification time.

 @param runsDir - runs directory, made when absent

 @param seconds - modification time in unix seconds

 @returns The record's path

 @example
 ```ts
 const path = await writeSliceRecord({ runsDir, seconds: 1_788_000_000, },);
 ```
 */
export async function writeSliceRecord(
  {
    runsDir,
    seconds,
  }: {
    readonly runsDir: string;
    readonly seconds: number;
  },
): Promise<string> {
  /**
   The record's place in the entry's cache.
   */
  const path = join(
    runsDir,
    'slice-cache',
    'Tabby',
    '0-1-tabby.json',
  );
  await mkdir(
    dirname(path,),
    { recursive: true, },
  );
  await writeFile(
    path,
    '{}',
    'utf8',
  );
  await utimes(
    path,
    seconds,
    seconds,
  );
  return path;
}

//endregion Cache account repository
