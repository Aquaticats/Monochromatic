import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import { namingFixtureGit, } from '../archive-naming.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Corpus pass clone fixture
// A throwaway repository holding invented `people/<id>/page.md` and
// `people/<id>/page.en.md` pages at one commit, which the corpus pass reads
// as its corpus clone. Never the operator's clone.

/**
 One invented entry of a throwaway corpus clone.

 @example
 ```ts
 const entry: CorpusCloneEntry = { id: 'tabby', sourceText: '猫睡觉。\n', targetText: 'The cat naps.\n', };
 ```
 */
export type CorpusCloneEntry = {
  /**
   Entry id, the name of its directory under `people/`.
   */
  readonly id: string;

  /**
   Text of the original page.
   */
  readonly sourceText: string;

  /**
   Text of the archive's English page, absent to leave the pair incomplete.
   */
  readonly targetText?: string;

  /**
   Text of the entry's `info.yml`, absent to leave the entry without one.
   */
  readonly infoYml?: string;
};

/**
 Writes one file under a clone, making its directories.

 @param cloneDir - clone to write under

 @param relPath - path inside the clone

 @param text - file content

 @example
 ```ts
 await writeCloneFile({ cloneDir, relPath: 'people/tabby/page.md', text: '猫睡觉。\n', },);
 ```
 */
async function writeCloneFile(
  {
    cloneDir,
    relPath,
    text,
  }: {
    readonly cloneDir: string;
    readonly relPath: string;
    readonly text: string;
  },
): Promise<void> {
  await mkdir(
    join(
      cloneDir,
      dirname(relPath,),
    ),
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
}

/**
 Builds a throwaway repository whose one commit holds the entries' pages.

 @param entries - invented entries to commit, none for a corpus with no people

 @returns Clone directory and its one commit, removed when the scope ends

 @example
 ```ts
 await using clone = await makeCorpusPassClone({ entries: [{ id: 'tabby', sourceText: '猫睡觉。\n', targetText: 'The cat naps.\n', },], },);
 ```
 */
export async function makeCorpusPassClone(
  { entries, }: { readonly entries: readonly CorpusCloneEntry[]; },
): Promise<AsyncDisposable & {
  readonly cloneDir: string;
  readonly commitSha: string;
}> {
  return await scratchDirWith({
    prefix: 'corpus-pass-clone-',
    setup: async function committed({ path: cloneDir, },): Promise<{
      readonly cloneDir: string;
      readonly commitSha: string;
    }> {
      await namingFixtureGit({
        cloneDir,
        args: ['init',],
      },);

      // A commit needs one file even where the corpus holds no person.
      await writeCloneFile({
        cloneDir,
        relPath: 'README.md',
        text: 'An invented corpus of napping cats.\n',
      },);
      /* oxlint-disable no-await-in-loop -- a handful of invented files written one after another */
      for (const entry of entries) {
        await writeCloneFile({
          cloneDir,
          relPath: `people/${entry.id}/page.md`,
          text: entry.sourceText,
        },);
        if (entry.targetText !== undefined) {
          await writeCloneFile({
            cloneDir,
            relPath: `people/${entry.id}/page.en.md`,
            text: entry.targetText,
          },);
        }
        if (entry.infoYml !== undefined) {
          await writeCloneFile({
            cloneDir,
            relPath: `people/${entry.id}/info.yml`,
            text: entry.infoYml,
          },);
        }
      }
      /* oxlint-enable no-await-in-loop */
      await namingFixtureGit({
        cloneDir,
        args: [
          'add',
          '--all',
        ],
      },);
      await namingFixtureGit({
        cloneDir,
        args: [
          'commit',
          '--message',
          'record invented corpus',
        ],
      },);
      return {
        cloneDir,
        commitSha: await namingFixtureGit({
          cloneDir,
          args: [
            'rev-parse',
            'HEAD',
          ],
        },),
      };
    },
  },);
}

//endregion Corpus pass clone fixture
