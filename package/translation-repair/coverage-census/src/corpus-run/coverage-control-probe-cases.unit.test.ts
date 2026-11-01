/**
 Tests for the passages the coverage control probe gathers: which entries it
 walks, how many passages it offers, and where each passage is said to sit,
 over a throwaway corpus.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { gatherCases, } from '../../dist/final/node/index.mjs';
import { makeProbeCorpus, } from '@monochromatic-dev/translation-repair-corpus-pass/ts/corpus-run/probes-b-built-command.test-fixture.ts';

/**
 Original of three sections against a translation of one: three section
 passages no matcher pairs.
 */
const THREE_SECTIONS = '## 一\n\n猫一。\n\n## 二\n\n猫二。\n\n## 三\n\n猫三。\n';

/**
 Translation of one section.
 */
const ONE_SECTION = '## One\n\nCat one.\n';

/**
 Original of one section holding three blocks against a translation holding
 one: two block passages.
 */
const THREE_BLOCKS = '## 一\n\n猫一。\n\n猫二。\n\n猫三。\n';

/**
 Gathers the passages of a corpus holding the given entries.

 @param files - corpus files by path

 @param onlyIds - entries asked for, empty for all

 @returns Where each case sits and what it asks about

 @example
 ```ts
 const gathered = await gatheredFrom({ files, onlyIds: [], },);
 ```
 */
async function gatheredFrom(
  {
    files,
    onlyIds,
  }: {
    readonly files: Readonly<Record<string, string>>;
    readonly onlyIds: readonly string[];
  },
): Promise<readonly { readonly where: string; readonly sourcePassage: string; }[]> {
  await using corpus = await makeProbeCorpus({ files, },);
  return (await gatherCases({
    onlyIds,
    pin: corpus.pin,
  },))
    .map(function plain({
      where,
      sourcePassage,
    },) {
      return {
        where,
        sourcePassage,
      };
    },);
}

await describe({
  name: gatherCases.name,
  children: [
    it({
      name: 'OFFERS a section passage for each section the matcher pairs with nothing, naming the entry and the section',
      fn: async () => {
        expect(await gatheredFrom({
          files: {
            'people/Mittens/page.md': THREE_SECTIONS,
            'people/Mittens/page.en.md': ONE_SECTION,
          },
          onlyIds: [],
        },),).toEqual([
          {
            where: 'Mittens section 0',
            sourcePassage: '## 一\n\n猫一。',
          },
          {
            where: 'Mittens section 1',
            sourcePassage: '## 二\n\n猫二。',
          },
          {
            where: 'Mittens section 2',
            sourcePassage: '## 三\n\n猫三。',
          },
        ],);
      },
    },),
    it({
      name: 'OFFERS a block passage for each block a paired section leaves unmatched, naming the entry, the pair and the block',
      fn: async () => {
        expect(await gatheredFrom({
          files: {
            'people/Tabby/page.md': THREE_BLOCKS,
            'people/Tabby/page.en.md': ONE_SECTION,
          },
          onlyIds: [],
        },),).toEqual([
          {
            where: 'Tabby pair 0 block 2',
            sourcePassage: '猫二。',
          },
          {
            where: 'Tabby pair 0 block 3',
            sourcePassage: '猫三。',
          },
        ],);
      },
    },),
    it({
      name: 'OFFERS nothing for an entry whose aligners pair every passage',
      fn: async () => {
        expect(await gatheredFrom({
          files: {
            'people/Whiskers/page.md': '## 一\n\n猫一。\n',
            'people/Whiskers/page.en.md': ONE_SECTION,
          },
          onlyIds: [],
        },),).toEqual([],);
      },
    },),
    it({
      name: 'STOPS at eight passages, cutting an entry short and never reading any later entry',
      fn: async () => {
        expect((await gatheredFrom({
          files: {
            'people/Cat1/page.md': THREE_SECTIONS,
            'people/Cat1/page.en.md': ONE_SECTION,
            'people/Cat2/page.md': THREE_SECTIONS,
            'people/Cat2/page.en.md': ONE_SECTION,
            'people/Cat3/page.md': THREE_SECTIONS,
            'people/Cat3/page.en.md': ONE_SECTION,
            'people/Cat4/page.md': THREE_SECTIONS,
          },
          onlyIds: [],
        },)).map(function whereOf({ where, },): string {
          return where;
        },),).toEqual([
          'Cat1 section 0',
          'Cat1 section 1',
          'Cat1 section 2',
          'Cat2 section 0',
          'Cat2 section 1',
          'Cat2 section 2',
          'Cat3 section 0',
          'Cat3 section 1',
        ],);
      },
    },),
    it({
      name: 'WALKS only the entries named, in the order the corpus lists them',
      fn: async () => {
        expect((await gatheredFrom({
          files: {
            'people/Cat1/page.md': THREE_BLOCKS,
            'people/Cat1/page.en.md': ONE_SECTION,
            'people/Cat2/page.md': THREE_BLOCKS,
            'people/Cat2/page.en.md': ONE_SECTION,
            'people/Cat3/page.md': THREE_BLOCKS,
            'people/Cat3/page.en.md': ONE_SECTION,
          },
          onlyIds: ['Cat3', 'Cat1',],
        },)).map(function whereOf({ where, },): string {
          return where;
        },),).toEqual([
          'Cat1 pair 0 block 2',
          'Cat1 pair 0 block 3',
          'Cat3 pair 0 block 2',
          'Cat3 pair 0 block 3',
        ],);
      },
    },),
    it({
      name: 'REFUSES an entry the corpus lacks, naming it',
      fn: async () => {
        await using corpus = await makeProbeCorpus({ files: { 'people/Cat1/page.md': 'x\n', }, },);
        try {
          await gatherCases({
            onlyIds: ['Nobody',],
            pin: corpus.pin,
          },);
          throw new Error('expected a refusal',);
        }
        catch (error) {
          expect(String(error,),).toBe(
            'StatedRefusalError: --only asks for "Nobody", which the corpus at the pin does not hold',
          );
        }
      },
    },),
    it({
      name: 'REFUSES an entry that lacks its translation, naming the page',
      fn: async () => {
        await using corpus = await makeProbeCorpus({ files: { 'people/Cat1/page.md': THREE_SECTIONS, }, },);
        try {
          await gatherCases({
            onlyIds: [],
            pin: corpus.pin,
          },);
          throw new Error('expected a refusal',);
        }
        catch (error) {
          expect(String(error,),).toBe(
            `CorpusReadError: corpus read failed for ${corpus.commitSha}:people/Cat1/page.en.md (missing-object); `
              + 'the commit has no such path: check the path, or pin a commit that has it.',
          );
        }
      },
    },),
  ],
},);
