import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import { fixtureGit, } from '../hermetic-git-run.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Slice census corpus fixture
// A THROWAWAY CORPUS-SHAPED REPOSITORY for the slice census's cases, holding
// invented cat pages only. The census reads `people/<id>/page.md` (the
// original) and `people/<id>/page.en.md` (the translation) at a commit.

/**
 Original page of three sections, in the Simplified Chinese the corpus uses.
 */
export const THREE_SECTION_SOURCE: string = [
  '---',
  'name: 小猫-whiskers',
  '---',
  '',
  '## 简介',
  '',
  '猫猫喜欢晒太阳。',
  '',
  '## 日常',
  '',
  '它每天在窗台上睡午觉，醒来就去找吃的。',
  '',
  '## 朋友们的话',
  '',
  '大家都说它是一只很温柔的猫。',
  '',
].join('\n',);

/**
 Translation carrying every section of the three-section original.
 */
export const THREE_SECTION_TARGET: string = [
  '---',
  'name: Whiskers',
  '---',
  '',
  '## Introduction',
  '',
  'Whiskers likes to sun herself.',
  '',
  '## Daily life',
  '',
  'She naps on the windowsill every day, and goes looking for food when she wakes.',
  '',
  '## What her friends say',
  '',
  'Everyone says she is a very gentle cat.',
  '',
].join('\n',);

/**
 Translation that stops after the first section, so two sections of the
 three-section original go unpaired.
 */
export const FIRST_SECTION_ONLY_TARGET: string = [
  '---',
  'name: Whiskers',
  '---',
  '',
  '## Introduction',
  '',
  'Whiskers likes to sun herself.',
  '',
].join('\n',);

/**
 Original page of one short section.
 */
export const ONE_BLOCK_SOURCE: string = [
  '---',
  'name: 小猫-whiskers',
  '---',
  '',
  '## 朋友们的话',
  '',
  '大家都说它是一只很温柔的猫。',
  '',
].join('\n',);

/**
 Translation of that section with one paragraph the original never wrote,
 which the census reads as a target-only block.
 */
export const ONE_BLOCK_PLUS_ADDED_TARGET: string = [
  '---',
  'name: Whiskers',
  '---',
  '',
  '## What her friends say',
  '',
  'Everyone says she is a very gentle cat.',
  '',
  'The kitten watched the garden birds all day.',
  '',
].join('\n',);

/**
 Original page whose front matter is not YAML a parser can read.
 */
export const UNREADABLE_FRONT_MATTER_SOURCE: string = [
  '---',
  'name: [小猫',
  '---',
  '',
  '## 简介',
  '',
  '猫猫喜欢晒太阳。',
  '',
].join('\n',);

/**
 Builds a throwaway repository holding the named files in one commit.

 @param files - file text by repository-relative path, written as given

 @returns Clone directory and the commit holding every file, removed on
 dispose

 @example
 ```ts
 await using corpus = await makeCensusCorpus({ files: { 'people/mochi/page.md': THREE_SECTION_SOURCE, }, },);
 ```
 */
export async function makeCensusCorpus(
  { files, }: { readonly files: Readonly<Record<string, string>>; },
): Promise<AsyncDisposable & {
  readonly cloneDir: string;
  readonly commitSha: string;
}> {
  // Fresh temp directory holding the throwaway repository.
  return await scratchDirWith({
    prefix: 'mochi-slice-census-',
    setup: async function seeded({ path: cloneDir, },): Promise<{
      readonly cloneDir: string;
      readonly commitSha: string;
    }> {
      await fixtureGit({
        cloneDir,
        args: ['init',],
      },);
      for (const [relPath, text,] of Object.entries(files,)) {
        /* oxlint-disable no-await-in-loop -- each file is written under a directory the previous one may have made */
        await mkdir(
          dirname(join(
            cloneDir,
            relPath,
          ),),
          { recursive: true, },
        );
        await writeFile(
          join(
            cloneDir,
            relPath,
          ),
          text,
          'utf8',
        );
        /* oxlint-enable no-await-in-loop */
      }
      await fixtureGit({
        cloneDir,
        args: [
          'add',
          '--all',
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
          'add the cat pages',
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
    },
  },);
}

//endregion Slice census corpus fixture
