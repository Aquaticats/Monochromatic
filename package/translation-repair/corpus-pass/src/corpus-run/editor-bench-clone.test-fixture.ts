/**
 A throwaway corpus clone holding one invented entry, for the cases that run
 an editor runner's built command up to the point where it asks a model. The
 operator's own clone is never named: a case points the command at this one
 through the two variables the command reads its corpus from.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { namingFixtureGit, } from '../archive-naming.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Original page of the invented entry, one section.
 */
const SOURCE_PAGE = '## 窗台\n\n小猫在窗台上打盹。它的尾巴垂在地板上。\n';

/**
 English page of the invented entry, one section against one.
 */
const TARGET_PAGE = '## The windowsill\n\nThe kitten dozes on the windowsill. Its tail hangs to the floor.\n';

/**
 Fills a fresh directory with one entry's two pages and commits them once.

 @param path - directory to make a repository of

 @returns Commit the repository is pinned at

 @example
 ```ts
 const { commitSha, } = await seedBenchClone({ path: cloneDir, },);
 ```
 */
async function seedBenchClone({ path: cloneDir, }: { readonly path: string; },): Promise<{ readonly commitSha: string; }> {
  await namingFixtureGit({
    cloneDir,
    args: ['init',],
  },);
  await mkdir(
    join(
      cloneDir,
      'people',
      'mittens',
    ),
    { recursive: true, },
  );
  await writeFile(
    join(
      cloneDir,
      'people',
      'mittens',
      'page.md',
    ),
    SOURCE_PAGE,
    'utf8',
  );
  await writeFile(
    join(
      cloneDir,
      'people',
      'mittens',
      'page.en.md',
    ),
    TARGET_PAGE,
    'utf8',
  );
  await namingFixtureGit({
    cloneDir,
    args: [
      'add',
      '--',
      'people/mittens/page.md',
      'people/mittens/page.en.md',
    ],
  },);
  await namingFixtureGit({
    cloneDir,
    args: [
      'commit',
      '--message',
      'record the invented entry',
    ],
  },);
  return {
    commitSha: await namingFixtureGit({
      cloneDir,
      args: [
        'rev-parse',
        'HEAD',
      ],
    },),
  };
}

/**
 Builds a git repository holding one entry with both its pages, committed once.

 @returns Clone's path, the commit it is pinned at, and its disposer, which
 removes the whole repository

 @example
 ```ts
 await using clone = await makeBenchClone();
 const env = { TRANSLATION_REPAIR_CORPUS_CLONE_DIR: clone.path, TRANSLATION_REPAIR_CORPUS_COMMIT: clone.commitSha, };
 ```
 */
export async function makeBenchClone(): Promise<
  AsyncDisposable & {
    readonly path: string;
    readonly commitSha: string;
  }
  > {
  return await scratchDirWith({
    prefix: 'editor-bench-clone-',
    setup: seedBenchClone,
  },);
}
