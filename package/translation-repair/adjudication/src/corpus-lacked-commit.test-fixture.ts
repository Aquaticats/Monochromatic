/**
 Throwaway corpus clone for the cases that pin a commit the clone lacks.
 Fixture content is cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import {
  devNull,
} from 'node:os';
import { join, } from 'node:path';

import { spawnKeyless, } from './child-environment.test-fixture.ts';
import {
  fixtureGit,
  REAL_GIT,
} from './hermetic-git-run.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Clone lacking a commit
// ONE CLONE HOLDING ONE COMMIT, and a full hash that is in no clone: the pair
// every walker case needs to tell a page absent at a commit the clone holds
// from a commit the clone lacks.

/**
 Characters in a SHA-1 object id.
 */
const OBJECT_ID_LENGTH = 40;

/**
 A full hash no clone holds, so a pin naming it is a commit the clone lacks.
 */
export const LACKED_COMMIT_SHA: string = 'a'.repeat(OBJECT_ID_LENGTH,);

/**
 Entry carrying both pages and one picture at the held commit.
 */
export const PAIRED_ENTRY = 'mittens';

/**
 Entry carrying its source page only at the held commit.
 */
export const ONE_PAGE_ENTRY = 'biscuit';

/**
 Picture the paired entry carries.
 */
export const PAIRED_PICTURE = 'intro.webp';

/**
 Bytes the picture holds, standing in for an image nothing here decodes.
 */
const PICTURE_BYTES = new TextEncoder()
  .encode('paw',);

/**
 Writes the two entries into a fresh clone and commits them.

 @param path - empty scratch directory the clone is made in

 @returns Clone directory and the one commit it holds

 @example
 ```ts
 const seeded = await seedCloneHoldingOneCommit({ path: '/scratch/clone', },);
 ```
 */
async function seedCloneHoldingOneCommit({ path: cloneDir, }: { readonly path: string; },): Promise<{
  readonly cloneDir: string;
  readonly commitSha: string;
}> {
  await spawnKeyless({
    file: REAL_GIT,
    args: [
      'init',
      cloneDir,
    ],
    extra: {
      GIT_CONFIG_GLOBAL: devNull,
      GIT_CONFIG_SYSTEM: devNull,
    },
  },);
  await mkdir(
    join(
      cloneDir,
      'people',
      PAIRED_ENTRY,
      'photos',
    ),
    { recursive: true, },
  );
  await mkdir(
    join(
      cloneDir,
      'people',
      ONE_PAGE_ENTRY,
    ),
    { recursive: true, },
  );
  await writeFile(
    join(
      cloneDir,
      'people',
      PAIRED_ENTRY,
      'page.md',
    ),
    '猫猫喜欢晒太阳。\n',
    'utf8',
  );
  await writeFile(
    join(
      cloneDir,
      'people',
      PAIRED_ENTRY,
      'page.en.md',
    ),
    'The cat likes the sun.\n',
    'utf8',
  );
  await writeFile(
    join(
      cloneDir,
      'people',
      PAIRED_ENTRY,
      'photos',
      PAIRED_PICTURE,
    ),
    PICTURE_BYTES,
  );
  await writeFile(
    join(
      cloneDir,
      'people',
      ONE_PAGE_ENTRY,
      'page.md',
    ),
    '猫猫在窗台上睡觉。\n',
    'utf8',
  );
  await fixtureGit({
    cloneDir,
    args: [
      'add',
      'people',
    ],
  },);
  await fixtureGit({
    cloneDir,
    args: [
      '-c',
      'user.name=cat',
      '-c',
      'user.email=cat@example.org',
      'commit',
      '--message',
      'add the cats',
      '--no-gpg-sign',
    ],
  },);
  return {
    cloneDir,
    commitSha: (await fixtureGit({
      cloneDir,
      args: [
        'rev-parse',
        'HEAD',
      ],
    },))
      .trim(),
  };
}

/**
 Builds a throwaway clone with one commit holding one paired entry and one
 entry with a source page only, removed on dispose.

 @returns Clone directory and the one commit it holds

 @example
 ```ts
 await using clone = await makeCloneHoldingOneCommit();
 ```
 */
export async function makeCloneHoldingOneCommit(): Promise<
  AsyncDisposable & {
    readonly cloneDir: string;
    readonly commitSha: string;
  }
  > {
  return await scratchDirWith({
    prefix: 'translation-repair-lacked-commit-',
    setup: seedCloneHoldingOneCommit,
  },);
}

//endregion Clone lacking a commit
