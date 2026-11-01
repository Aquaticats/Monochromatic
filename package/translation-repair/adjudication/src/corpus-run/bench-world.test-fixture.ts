import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract';
import { fixtureGit, } from '../hermetic-git-run.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Bench world
// EVERYTHING A BUILT BENCH OR CALIBRATION COMMAND READS FROM OUTSIDE ITS OWN
// PROCESS, made as throwaway places: a runs directory, a lookup cache
// directory and a corpus clone holding invented cat pages at one commit.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The commands draw their slices from the
// pinned corpus and keep their records in the runs directory, so a case that
// runs one names each place in the child's environment (the shared child
// fixture removes every `TRANSLATION_REPAIR_` variable first), and none of
// them can be the operator's own clone, cache or runs.

/**
 What the bench draw says of a corpus that yields no slice, when the clone a
 world made holds no entry: the whole message of its stated refusal.
 */
export const NO_ENTRY_NO_SLICE_REFUSAL: string =
  'the corpus at the pin yields no slice to sample: 0 entries are listed there and none could be sliced, so a '
    + 'bench drawn over it would compare on no work; check the clone and the commit this run reads';

/**
 One invented entry's two pages.

 @example
 ```ts
 const page: BenchPages = { source: '小猫打盹。\n', english: 'The kitten naps.\n', };
 ```
 */
export type BenchPages = {
  /**
   Original page, `page.md` in the clone.
   */
  readonly source: string;

  /**
   English page, `page.en.md` in the clone.
   */
  readonly english: string;
};

/**
 The places a built command reads, with the environment naming them.

 @example
 ```ts
 await using world = await benchWorld({ entries: { mittens: pages, }, },);
 ```
 */
type BenchWorld = AsyncDisposable & {
  /**
   Directory the command keeps its records in.
   */
  readonly runsDir: string;

  /**
   Environment variables naming every place, for the child.
   */
  readonly env: Readonly<Record<string, string>>;

  /**
   Pin naming the clone and the commit its pages were written at, for a case
   that reads the corpus in its own process.
   */
  readonly pin: CorpusPin;
};

/**
 One page of the clone.

 @example
 ```ts
 const file: PageFile = { relPath: 'people/mittens/page.md', text: '小猫打盹。\n', };
 ```
 */
type PageFile = {
  /**
   Path inside the clone.
   */
  readonly relPath: string;

  /**
   Whole contents.
   */
  readonly text: string;
};

/**
 Lists the pages the clone holds.

 @param entries - pages by entry id, both sides

 @param originalOnly - original pages of entries with no English page

 @returns Every page with its path in the clone

 @example
 ```ts
 const files = pagesOf({ entries, originalOnly, },);
 ```
 */
function pagesOf(
  {
    entries,
    originalOnly,
  }: {
    readonly entries: Readonly<Record<string, BenchPages>>;
    readonly originalOnly: Readonly<Record<string, string>>;
  },
): readonly PageFile[] {
  return [
    ...Object.entries(entries,)
      .flatMap(function bothPages([entryId, pages,]: ReadonlyEntry<string, BenchPages>,): readonly PageFile[] {
        return [
          {
            relPath: `people/${entryId}/page.md`,
            text: pages.source,
          },
          {
            relPath: `people/${entryId}/page.en.md`,
            text: pages.english,
          },
        ];
      },),
    ...Object.entries(originalOnly,)
      .map(function onePage([entryId, source,]: ReadonlyEntry<string, string>,): PageFile {
        return {
          relPath: `people/${entryId}/page.md`,
          text: source,
        };
      },),
  ];
}

/**
 Makes the runs directory, the cache directory and a clone holding the named
 entries at one commit.

 @param entries - pages by entry id, written under `people/<id>/` and
 committed together

 @param originalOnly - original pages of entries that have no English page,
 by entry id, committed with the rest

 @returns The places and the environment naming them, removed on scope exit

 @example
 ```ts
 await using world = await benchWorld({ entries: { mittens: pages, }, },);
 ```
 */
export async function benchWorld(
  {
    entries,
    originalOnly = {},
  }: {
    readonly entries: Readonly<Record<string, BenchPages>>;
    readonly originalOnly?: Readonly<Record<string, string>>;
  },
): Promise<BenchWorld> {
  return await scratchDirWith({
    prefix: 'bench-world-',
    setup: async function seeded({ path, },): Promise<{
      readonly runsDir: string;
      readonly env: Readonly<Record<string, string>>;
      readonly pin: CorpusPin;
    }> {
      /**
       Where the command keeps its records.
       */
      const runsDir = join(
        path,
        'runs',
      );

      /**
       Where the command would read lookups from.
       */
      const cacheDir = join(
        path,
        'cache',
      );

      /**
       The clone the command draws its slices from.
       */
      const cloneDir = join(
        path,
        'clone',
      );
      await Promise.all([
        mkdir(
          runsDir,
          { recursive: true, },
        ),
        mkdir(
          cacheDir,
          { recursive: true, },
        ),
        mkdir(
          cloneDir,
          { recursive: true, },
        ),
      ],);
      await fixtureGit({
        cloneDir,
        args: [
          'init',
          '--quiet',
          '--initial-branch',
          'main',
        ],
      },);

      /**
       Every page this clone holds, by its path in the clone.
       */
      const files = pagesOf({
        entries,
        originalOnly,
      },);
      await Promise.all(files.map(async function writeOne(file,): Promise<void> {
        /**
         Where this page lands in the clone.
         */
        const target = join(
          cloneDir,
          file.relPath,
        );
        await mkdir(
          dirname(target,),
          { recursive: true, },
        );
        await writeFile(
          target,
          file.text,
          'utf8',
        );
      },),);
      await fixtureGit({
        cloneDir,
        args: [
          'add',
          '--',
          ...files.map(function pathOf(file,): string {
            return file.relPath;
          },),
        ],
      },);
      await fixtureGit({
        cloneDir,
        args: [
          '-c',
          'user.name=Bench World',
          '-c',
          'user.email=bench-world@example.invalid',
          'commit',
          '--quiet',
          '--allow-empty',
          '--message',
          'cats',
        ],
      },);

      /**
       Commit the pages stand at.
       */
      const commitSha = await fixtureGit({
        cloneDir,
        args: [
          'rev-parse',
          'HEAD',
        ],
      },);

      return {
        runsDir,
        pin: {
          cloneDir,
          commitSha,
        },
        env: {
          TRANSLATION_REPAIR_RUNS_DIR: runsDir,
          TRANSLATION_REPAIR_LOOKUP_CACHE_DIR: cacheDir,
          TRANSLATION_REPAIR_CORPUS_CLONE_DIR: cloneDir,
          TRANSLATION_REPAIR_CORPUS_COMMIT: commitSha,
        },
      };
    },
  },);
}

//endregion Bench world
