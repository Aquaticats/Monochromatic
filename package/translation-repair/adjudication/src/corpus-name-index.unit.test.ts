/**
 Guards class seventy-eight (2026-09-22): a person the source names by a Han
 handle reached the page as a transliteration of nothing (a romanization the
 handle's pinyin does not support) while the corpus
 itself declares that person's English name in their own entry's front
 matter, because the sheets' declared names come only from the page's own
 archive. Every entry's front matter is readable at the pin, so the names
 the corpus declares for the people this entry mentions join the identity
 context beside the page's own renderings.
 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  corpusNameLines,
  corpusNamesOf,
  CorpusReadError,
  readCorpusFile,
  readCorpusNames,
} from '@monochromatic-dev/translation-repair-corpus-source';
import {
  LACKED_COMMIT_SHA,
  makeCloneHoldingOneCommit,
  ONE_PAGE_ENTRY,
  PAIRED_ENTRY,
} from './corpus-lacked-commit.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';

/**
 Three invented entries: one declaring a Han name with an alias and an
 English rendering with two handles, one declaring a Han name whose English
 side carries only an alias, and one with no Han form to index.
 */
const ENTRIES = [
  {
    id: 'gum',
    sourceText: '---\nname: 奇妙的猫糖\ninfo:\n  alias: 猫糖\n---\n\n猫糖是一只爱吃鱼干的猫。\n',
    targetText: '---\nname: Magic Cat Candy\ninfo:\n  alias: Cat Candy, meow1219\n---\n\nCat Candy was a cat who loved dried fish.\n',
  },
  {
    id: 'slice',
    sourceText: '---\nname: 单鱼\ninfo:\n  alias: 单鱼\n---\n\n单鱼很温柔。\n',
    targetText: '---\ninfo:\n  alias: Danyu\n---\n\nDanyu was gentle.\n',
  },
  {
    id: 'latin',
    sourceText: '---\nname: Whiskers\n---\n\n它睡了。\n',
    targetText: '---\nname: Whiskers\n---\n\nIt slept.\n',
  },
  {
    id: 'kitty-one',
    sourceText: '---\nname: 猫猫\n---\n\n猫猫来了。\n',
    targetText: '---\nname: Kitty One\n---\n\nKitty One came.\n',
  },
  {
    id: 'kitty-two',
    sourceText: '---\ninfo:\n  alias: 猫猫\n---\n\n猫猫走了。\n',
    targetText: '---\nname: Kitty Two\n---\n\nKitty Two left.\n',
  },
] as const;

/**
 Heading the block carries.
 */
const HEADING = 'NAMES OF OTHER PEOPLE IN THIS ARCHIVE whose declared handle this text carries, as their own '
  + 'entries render them (where the text means the person, render them as their entry does, never a '
  + 'transliteration; where the handle is an ordinary word here, it is a word and stays translated):';

await describe({
  name: 'the corpus names the people an entry mentions as their own entries render them (class seventy-eight)',
  children: [
    it({
      name: 'INDEXES every Han name or alias an entry declares against the English forms its own translation '
        + 'declares, and DROPS a form two entries declare, since the corpus then cannot say which person a page means',
      fn: async () => {
        const names = corpusNamesOf({ entries: ENTRIES, },);
        expect(names,).toEqual([
          {
            source: '奇妙的猫糖',
            renderings: ['Magic Cat Candy', 'Cat Candy', 'meow1219',],
            entryId: 'gum',
          },
          {
            source: '猫糖',
            renderings: ['Magic Cat Candy', 'Cat Candy', 'meow1219',],
            entryId: 'gum',
          },
          {
            source: '单鱼',
            renderings: ['Danyu',],
            entryId: 'slice',
          },
        ],);
      },
    },),
    it({
      name: 'LINES the names a text carries, the longest form of one entry once, never the entry\'s own',
      fn: async () => {
        const names = corpusNamesOf({ entries: ENTRIES, },);
        const lines = corpusNameLines({
          text: '醒来之后，她听说奇妙的猫糖和单鱼一起去了鱼市。\n',
          names,
          ownId: 'slice',
        },);
        expect(lines,).toEqual([
          HEADING,
          '- 奇妙的猫糖 (entry gum): "Magic Cat Candy", "Cat Candy", "meow1219"',
        ],);
      },
    },),
    it({
      name: 'LINES the short alias when the text carries only it',
      fn: async () => {
        const names = corpusNamesOf({ entries: ENTRIES, },);
        const lines = corpusNameLines({
          text: '猫糖和单鱼都走了。\n',
          names,
          ownId: 'own',
        },);
        expect(lines,).toEqual([
          HEADING,
          '- 猫糖 (entry gum): "Magic Cat Candy", "Cat Candy", "meow1219"',
          '- 单鱼 (entry slice): "Danyu"',
        ],);
      },
    },),
    it({
      name: 'STAYS SILENT when the text names nobody the corpus declares, a shared form included',
      fn: async () => {
        const lines = corpusNameLines({
          text: '猫猫在门口犹豫。\n',
          names: corpusNamesOf({ entries: ENTRIES, },),
          ownId: 'own',
        },);
        expect(lines,).toEqual([],);
      },
    },),
    it({
      name: 'READS every entry at the pin through injected readers, stepping over a pair whose English page is absent',
      fn: async () => {
        /**
         Paths asked for, to prove both sides of every listed entry were read.
         */
        const asked: string[] = [];
        const names = await readCorpusNames({
          pin: { cloneDir: '/nonexistent', commitSha: 'deadbeef', },
          listPeople: async () => ['gum', 'missing',],
          readFile: async ({ relPath, },) => {
            asked.push(relPath,);
            if (relPath === 'people/gum/page.md')
              return ENTRIES[0].sourceText;
            if (relPath === 'people/gum/page.en.md')
              return ENTRIES[0].targetText;
            if (relPath === 'people/missing/page.md')
              return '---\nname: 影猫\n---\n';
            throw new CorpusReadError({
              detail: relPath,
              cause: { stderr: `fatal: path '${relPath}' does not exist in 'deadbeef'`, },
              commit: 'held',
            },);
          },
        },);
        expect(names,).toEqual([
          {
            source: '奇妙的猫糖',
            renderings: ['Magic Cat Candy', 'Cat Candy', 'meow1219',],
            entryId: 'gum',
          },
          {
            source: '猫糖',
            renderings: ['Magic Cat Candy', 'Cat Candy', 'meow1219',],
            entryId: 'gum',
          },
        ],);
        expect(asked.toSorted(),).toEqual([
          'people/gum/page.en.md',
          'people/gum/page.md',
          'people/missing/page.en.md',
          'people/missing/page.md',
        ],);
      },
    },),

    it({
      name: 'INDEXES nothing from an entry whose translation declares no forms, since the renderings '
        + 'are the point of the index',
      fn: async () => {
        expect(corpusNamesOf({
          entries: [
            ENTRIES[0],
            {
              id: 'bare',
              sourceText: '---\nname: 影猫\n---\n',
              targetText: '---\n---\n\nProse.\n',
            },
          ],
        },),).toEqual([
          {
            source: '奇妙的猫糖',
            renderings: ['Magic Cat Candy', 'Cat Candy', 'meow1219',],
            entryId: 'gum',
          },
          {
            source: '猫糖',
            renderings: ['Magic Cat Candy', 'Cat Candy', 'meow1219',],
            entryId: 'gum',
          },
        ],);
      },
    },),

    it({
      name: 'INDEXES no form one code point long or past the longest name, since neither names anyone '
        + 'this corpus would carry',
      fn: async () => {
        expect(corpusNamesOf({
          entries: [{
            id: 'sizes',
            sourceText: '---\nname: 影\ninfo:\n  alias: 影猫猫猫猫猫猫猫猫猫猫猫猫猫猫猫猫猫猫猫猫猫猫猫猫\n---\n',
            targetText: '---\nname: Shadow\ninfo:\n  alias: Long\n---\n',
          },],
        },),).toEqual([],);
      },
    },),

    it({
      name: 'RETHROWS a read failure that is no missing object, since only the absent side of a pair '
        + 'is stepped over',
      fn: async () => {
        /** Value caught from the read whose git failed for another reason. */
        let caught: unknown;
        try {
          await readCorpusNames({
            pin: { cloneDir: '/nonexistent', commitSha: 'deadbeef', },
            listPeople: async () => ['gum',],
            readFile: async ({ relPath, },) => {
              throw new CorpusReadError({
                detail: relPath,
                cause: { stderr: 'fatal: ambiguous argument', },
                commit: 'held',
              },);
            },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(CorpusReadError,);
        expect(String(caught,),).toBe(
          'CorpusReadError: corpus read failed for people/gum/page.md (other); '
            + 'the read failed another way: run the same git read in the clone by hand to see why.',
        );
        if (!(caught instanceof CorpusReadError))
          throw new Error('the read failure must reach the caller as a CorpusReadError',);
        expect(caught.kind,).toBe('other',);
      },
    },),
    it({
      name: 'RETHROWS the failure of the first listed entry when two entries are refused for another reason and '
        + 'the later one is refused first',
      fn: async () => {
        /**
         Opened by the later entry's read as it is refused, which the earlier
         entry's reads wait for.
         */
        const laterRefused = Promise.withResolvers<undefined>();
        /**
         Value caught from the walk over both entries.
         */
        const refusal = await rejectionOf({
          promise: readCorpusNames({
            pin: { cloneDir: '/nonexistent', commitSha: 'deadbeef', },
            listPeople: async () => ['gum', 'tabby',],
            readFile: async ({ relPath, },) => {
              /**
               What git says of a read it could not make.
               */
              const failure = new CorpusReadError({
                detail: relPath,
                cause: { stderr: 'fatal: ambiguous argument', },
                commit: 'held',
              },);
              if (relPath.startsWith('people/tabby/',)) {
                laterRefused.resolve(undefined,);
                throw failure;
              }
              await laterRefused.promise;
              throw failure;
            },
          },),
        },);
        expect(refusal,).toBeInstanceOf(CorpusReadError,);
        expect(String(refusal,),).toBe(
          'CorpusReadError: corpus read failed for people/gum/page.md (other); '
            + 'the read failed another way: run the same git read in the clone by hand to see why.',
        );
      },
    },),
    it({
      name: 'RETHROWS the failure of the original when both pages of an entry are refused for another reason '
        + 'and the archive page is refused first',
      fn: async () => {
        /**
         Opened by the archive read as it is refused, which the original's read
         waits for.
         */
        const archiveRefused = Promise.withResolvers<undefined>();
        /**
         Value caught from the walk over the entry.
         */
        const refusal = await rejectionOf({
          promise: readCorpusNames({
            pin: { cloneDir: '/nonexistent', commitSha: 'deadbeef', },
            listPeople: async () => ['gum',],
            readFile: async ({ relPath, },) => {
              /**
               What git says of a read it could not make.
               */
              const failure = new CorpusReadError({
                detail: relPath,
                cause: { stderr: 'fatal: ambiguous argument', },
                commit: 'held',
              },);
              if (relPath.endsWith('/page.en.md',)) {
                archiveRefused.resolve(undefined,);
                throw failure;
              }
              await archiveRefused.promise;
              throw failure;
            },
          },),
        },);
        expect(refusal,).toBeInstanceOf(CorpusReadError,);
        expect(String(refusal,),).toBe(
          'CorpusReadError: corpus read failed for people/gum/page.md (other); '
            + 'the read failed another way: run the same git read in the clone by hand to see why.',
        );
      },
    },),
    it({
      name: 'STEPS OVER an entry whose original is absent at the pin even when its archive page is refused for '
        + 'another reason first',
      fn: async () => {
        /**
         Opened by the archive read as it is refused, which the original's read
         waits for.
         */
        const archiveRefused = Promise.withResolvers<undefined>();
        expect(await readCorpusNames({
          pin: { cloneDir: '/nonexistent', commitSha: 'deadbeef', },
          listPeople: async () => ['gum',],
          readFile: async ({ relPath, },) => {
            if (relPath.endsWith('/page.en.md',)) {
              archiveRefused.resolve(undefined,);
              throw new CorpusReadError({
                detail: relPath,
                cause: { stderr: 'fatal: ambiguous argument', },
                commit: 'held',
              },);
            }
            await archiveRefused.promise;
            throw new CorpusReadError({
              detail: relPath,
              cause: { stderr: `fatal: path '${relPath}' does not exist in 'deadbeef'`, },
              commit: 'held',
            },);
          },
        },),).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES a pin whose commit the clone lacks, read through the real page reader, instead of '
        + 'indexing nothing as if every listed entry had one page',
      fn: async () => {
        await using clone = await makeCloneHoldingOneCommit();

        /**
         What the index of a commit the clone lacks refused with.
         */
        const refusal = await rejectionOf({
          promise: readCorpusNames({
            pin: {
              cloneDir: clone.cloneDir,
              commitSha: LACKED_COMMIT_SHA,
            },
            listPeople: async function listedByHand(): Promise<readonly string[]> {
              return [PAIRED_ENTRY, ONE_PAGE_ENTRY,];
            },
            readFile: readCorpusFile,
          },),
        },);

        expect(refusal,).toBeInstanceOf(CorpusReadError,);
        expect((refusal instanceof CorpusReadError) && refusal.kind,).toBe('missing-commit',);
        expect(String(refusal,),).toBe(
          `CorpusReadError: corpus read failed for ${LACKED_COMMIT_SHA}:people/${PAIRED_ENTRY}/page.md (missing-commit); `
            + 'the clone holds no commit by that revision: fetch it, or pin '
            + 'a commit the clone holds.',
        );
      },
    },),
    it({
      name: 'STEPS PAST an entry with one page at a commit the clone holds, indexing the entry that has both',
      fn: async () => {
        await using clone = await makeCloneHoldingOneCommit();
        expect(await readCorpusNames({
          pin: {
            cloneDir: clone.cloneDir,
            commitSha: clone.commitSha,
          },
          listPeople: async function listedByHand(): Promise<readonly string[]> {
            return [PAIRED_ENTRY, ONE_PAGE_ENTRY,];
          },
          readFile: readCorpusFile,
        },),).toEqual(corpusNamesOf({ entries: [], },),);
      },
    },),
  ],
},);
