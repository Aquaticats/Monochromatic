/**
 Tests for measuring one corpus entry after slicing.

 THE CENSUS IS THE INSTRUMENT EVERY SIZE CLAIM RESTS ON, and its most valuable
 column is the one that used to read zero for the wrong reason.
 `unpairedSourceSections` counts sections the aligner REFUSED to pair, and
 those are absent from `alignment.pairs` entirely rather than present with an
 empty side. An earlier counter walked the pairs, so it could only ever report
 zero, and zero read as "nothing went unpaired" instead of "this cannot see
 them". The "SEES the sections the aligner refused" case gives the census a page
 whose sections genuinely do not pair and requires a number greater than zero.

 THE PIN IS INJECTED, which is why any of this can be tested. `censusEntry`
 read `RUN_CORPUS_PIN` directly, so exercising it meant having the unlicensed
 corpus clone on disk and a suite that passed on one machine only. It now takes
 the pin the way `readAuditArguments` takes `argv`, and the cases in this file point
 it at a throwaway git repository built in a temp directory.

 FIXTURE CONTENT IS CAT-THEMED INVENTION mirroring corpus structure only:
 Simplified Chinese against English, one entry, committed once.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { devNull, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { spawnKeyless, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { CorpusReadError, } from '@monochromatic-dev/translation-repair-corpus-source';
import { censusEntry, } from '../../dist/final/node/index.mjs';
import { fixtureGit, REAL_GIT, } from '@monochromatic-dev/translation-repair-adjudication/ts/hermetic-git-run.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Slice census entry tests

/**
 Entry the throwaway clone carries.
 */
const ENTRY_ID = 'whiskers';

/**
 Original page: three sections, in the Simplified Chinese the corpus uses.
 */
const SOURCE_PAGE = [
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
 Translation carrying every section of the original.
 */
const FULL_TARGET_PAGE = [
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
 Original page with one section of three blocks, so a block pairing that
 declines the middle one changes what the slice carries.
 */
const BLOCKY_SOURCE_PAGE = [
  '---',
  'name: 小猫-whiskers',
  '---',
  '',
  '## 日常',
  '',
  '猫猫喜欢晒太阳。',
  '',
  '它每天在窗台上睡午觉。',
  '',
  '大家都说它是一只很温柔的猫。',
  '',
].join('\n',);

/**
 Translation of the same three blocks.
 */
const BLOCKY_TARGET_PAGE = [
  '---',
  'name: Whiskers',
  '---',
  '',
  '## Daily life',
  '',
  'Whiskers likes to sun herself.',
  '',
  'She naps on the windowsill every day.',
  '',
  'Everyone says she is a very gentle cat.',
  '',
].join('\n',);

/**
 Translation that stops after the first section, so two go unpaired.
 */
const SHORT_TARGET_PAGE = [
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
 Original page with one short section, whose translation has one block to
 pair with it.
 */
const ONE_BLOCK_SOURCE_PAGE = [
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
 Translation of that section's one block, 39 characters.
 */
const GENTLE_CAT_BLOCK = 'Everyone says she is a very gentle cat.';

/**
 Paragraph the translation adds that the original never wrote, 44
 characters.
 */
const ADDED_SHORT_BLOCK = 'The kitten watched the garden birds all day.';

/**
 Paragraph the translation adds that the original never wrote, 227
 characters.
 */
const ADDED_LONG_BLOCK = 'The kitten watched the garden birds all day, and when the sun went down behind the old stable it padded '
  + 'across the warm tiles to the kitchen door, sat beside the empty bowl, and told everyone who would listen how '
  + 'hungry it was.';

/**
 Builds the translation of the one-block section with a paragraph added
 after the block it renders.

 @param added - paragraph the translation adds

 @returns Translation page

 @example
 ```ts
 const page = onePlusAddedPage({ added: ADDED_SHORT_BLOCK, },);
 ```
 */
function onePlusAddedPage({ added, }: { readonly added: string; },): string {
  return [
    '---',
    'name: Whiskers',
    '---',
    '',
    '## What her friends say',
    '',
    GENTLE_CAT_BLOCK,
    '',
    added,
    '',
  ].join('\n',);
}

/**
 Row the census reads off a section whose recorded pairing places the
 original on the first paragraph and the added paragraph on nothing.

 @returns Block pairing recipe naming the heading and the one paragraph

 @example
 ```ts
 const recipe = recipeNamingOneBlock();
 ```
 */
function recipeNamingOneBlock(): {
  readonly blockPairings: ReadonlyMap<number, readonly { readonly source: number; readonly target: number; }[]>;
  readonly unrecorded: readonly ['sectionPairing',];
} {
  return {
    blockPairings: new Map([[
      0,
      [
        {
          source: 0,
          target: 0,
        },
        {
          source: 1,
          target: 1,
        },
      ],
    ],],),
    unrecorded: ['sectionPairing',],
  };
}

/**
 Builds a throwaway corpus-shaped repository holding one entry.

 @param targetPage - translation to commit beside the original, which decides
 how many sections pair

 @returns Pin naming the clone and its one commit, and an async disposer

 @example
 ```ts
 await using corpus = await throwawayCorpus({ targetPage: FULL_TARGET_PAGE, },);
 ```
 */
async function throwawayCorpus(
  {
    targetPage,
    sourcePage = SOURCE_PAGE,
  }: {
    readonly targetPage: string;
    readonly sourcePage?: string;
  },
): Promise<
  AsyncDisposable & {
    readonly pin: {
      readonly cloneDir: string;
      readonly commitSha: string;
    };
  }
  > {
  // Fresh temp directory holding the throwaway repository.
  return await scratchDirWith({
    prefix: 'whiskers-slice-census-',
    setup: async function seeded({ path: cloneDir, },): Promise<{
      readonly pin: {
        readonly cloneDir: string;
        readonly commitSha: string;
      };
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
          ENTRY_ID,
        ),
        { recursive: true, },
      );
      await writeFile(
        join(
          cloneDir,
          'people',
          ENTRY_ID,
          'page.md',
        ),
        sourcePage,
        'utf8',
      );
      await writeFile(
        join(
          cloneDir,
          'people',
          ENTRY_ID,
          'page.en.md',
        ),
        targetPage,
        'utf8',
      );
      await fixtureGit({
        cloneDir,
        args: [
          'add',
          `people/${ENTRY_ID}/page.md`,
          `people/${ENTRY_ID}/page.en.md`,
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
          'add whiskers',
          '--no-gpg-sign',
        ],
      },);

      /**
       Commit every read of this fixture's corpus pins to.
       */
      const commitSha = (await fixtureGit({
        cloneDir,
        args: [
          'rev-parse',
          'HEAD',
        ],
      },))
        .trim();

      return {
        pin: {
          cloneDir,
          commitSha,
        },
      };
    },
  },);
}

await describe({
  name: censusEntry.name,
  children: [
    it({
      name: 'MEASURES an entry whose translation carries every section',
      fn: async () => {
        await using corpus = await throwawayCorpus({ targetPage: FULL_TARGET_PAGE, },);

        /**
         What the census made of that entry.
         */
        const row = await censusEntry({
          entryId: ENTRY_ID,
          pin: corpus.pin,
        },);

        expect(row.entryId,).toBe(ENTRY_ID,);
        expect(row.sliceSourceChars.length,).toBeGreaterThan(0,);
        expect(row.sliceSourceChars.length,).toBe(row.sliceTargetChars.length,);
      },
    },),
    it({
      name: 'COUNTS one target-only block, at its own size, where the translation closes its last section with '
        + 'a short paragraph the original never wrote, and counts none where the two carry the same blocks',
      fn: async () => {
        await using corpus = await throwawayCorpus({
          targetPage: `${FULL_TARGET_PAGE}\nMeow.\n`,
        },);
        await using control = await throwawayCorpus({ targetPage: FULL_TARGET_PAGE, },);

        /**
         What the census reads off the control, whose translation carries
         every block of the original and no more.
         */
        const paired = {
          entryId: ENTRY_ID,
          carve: 'deterministic',
          pairingRefusal: '',
          sliceSourceChars: [
            15,
            26,
            24,
          ],
          unpairedSourceSections: 0,
          unpairedSourceChars: 0,
          unpairedTargetSections: 0,
          unpairedTargetChars: 0,
        };
        expect([
          await censusEntry({
            entryId: ENTRY_ID,
            pin: corpus.pin,
          },),
          await censusEntry({
            entryId: ENTRY_ID,
            pin: control.pin,
          },),
        ],).toEqual([
          {
            ...paired,
            // The added paragraph rides in the last slice, seven characters
            // with the blank line before it.
            sliceTargetChars: [
              47,
              94,
              71,
            ],
            targetOnlyBlocks: 1,
            targetOnlyChars: 5,
            targetOnlyBlockChars: [5,],
          },
          {
            ...paired,
            sliceTargetChars: [
              47,
              94,
              64,
            ],
            targetOnlyBlocks: 0,
            targetOnlyChars: 0,
            targetOnlyBlockChars: [],
          },
        ],);
      },
    },),
    it({
      name: 'COUNTS characters, not slices, so no slice is measured as empty',
      fn: async () => {
        await using corpus = await throwawayCorpus({ targetPage: FULL_TARGET_PAGE, },);

        /**
         What the census made of that entry.
         */
        const row = await censusEntry({
          entryId: ENTRY_ID,
          pin: corpus.pin,
        },);

        for (const chars of row.sliceSourceChars) {
          expect(chars,).toBeGreaterThan(0,);
        }
        for (const chars of row.sliceTargetChars) {
          expect(chars,).toBeGreaterThan(0,);
        }
      },
    },),
    it({
      name: 'SEES the sections the aligner refused, which a walk over pairs cannot',
      fn: async () => {
        // The defect this column exists after: a counter that walked
        // `alignment.pairs` could only report zero, because a refused section is
        // absent from the pairs entirely. Zero then read as "nothing went
        // unpaired" rather than as "this cannot see them".
        await using corpus = await throwawayCorpus({ targetPage: SHORT_TARGET_PAGE, },);

        /**
         What the census made of a half-translated entry.
         */
        const row = await censusEntry({
          entryId: ENTRY_ID,
          pin: corpus.pin,
        },);

        expect(row.unpairedSourceSections,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'REPORTS no unpaired sections when the translation carries them all',
      fn: async () => {
        // The positive control for the "SEES the sections the aligner refused"
        // case: a column that always reported a positive number would pass that
        // one and fail this.
        await using corpus = await throwawayCorpus({ targetPage: FULL_TARGET_PAGE, },);

        expect((await censusEntry({
          entryId: ENTRY_ID,
          pin: corpus.pin,
        },)).unpairedSourceSections,).toBe(0,);
      },
    },),
    it({
      name:
        'CARVES the sections through a supplied recipe and labels the row settled, since the slice sizes '
          + 'then describe the slicing the lanes judged rather than the deterministic baseline',
      fn: async () => {
        await using corpus = await throwawayCorpus({ targetPage: FULL_TARGET_PAGE, },);

        /**
         Deterministic carve, which pairs the three equal-shaped sections by
         index and leaves nothing unpaired.
         */
        const baseline = await censusEntry({
          entryId: ENTRY_ID,
          pin: corpus.pin,
        },);
        expect(baseline.carve,).toBe('deterministic',);
        expect(baseline.unpairedSourceSections,).toBe(0,);

        /**
         Carve through a recipe whose section round paired the outer two
         sections and left the middle one unclaimed.
         */
        const settled = await censusEntry({
          entryId: ENTRY_ID,
          pin: corpus.pin,
          recipe: {
            sectionPairing: [
              {
                source: 0,
                target: 0,
              },
              {
                source: 2,
                target: 2,
              },
            ],
            unrecorded: [],
          },
        },);
        expect(settled.carve,).toBe('settled-complete',);
        // The recipe has to move the accounting, or this passes for the wrong
        // reason.
        expect(settled.unpairedSourceSections,).toBe(1,);
        expect(settled.unpairedTargetSections,).toBe(1,);
      },
    },),
    it({
      name:
        'CARVES the blocks through a supplied recipe, so a block the roster declined leaves the slice, '
          + 'and labels a recipe with a defaulted half as partial',
      fn: async () => {
        await using corpus = await throwawayCorpus({
          sourcePage: BLOCKY_SOURCE_PAGE,
          targetPage: BLOCKY_TARGET_PAGE,
        },);

        /**
         Deterministic carve: one slice carrying all three blocks.
         */
        const baseline = await censusEntry({
          entryId: ENTRY_ID,
          pin: corpus.pin,
        },);

        /**
         Carve through a recipe that places every original of the four blocks
         (the heading and three paragraphs; the last two paragraphs merged
         into the third translation block) and so declines the last
         translation block, recorded without a section decider.

         A gap left unplaced on both sides is not a decline: since class one
         hundred twelve (mikaela15) it reads as a merge and stays in the slice,
         and a pairing leaving any original unplaced declines nothing.
         */
        const settled = await censusEntry({
          entryId: ENTRY_ID,
          pin: corpus.pin,
          recipe: {
            blockPairings: new Map([[
              0,
              [
                {
                  source: 0,
                  target: 0,
                },
                {
                  source: 1,
                  target: 1,
                },
                {
                  source: 2,
                  target: 2,
                },
                {
                  source: 3,
                  target: 2,
                },
              ],
            ],],),
            unrecorded: ['sectionPairing',],
          },
        },);
        expect(settled.carve,).toBe('settled-partial',);

        /**
         Translation characters the two carves put into slices.
         */
        const [baselineChars, settledChars,] = [
          baseline,
          settled,
        ].map(function totalTarget(row,): number {
          return row.sliceTargetChars
            .reduce(function add(
              sum,
              chars,
            ): number {
              return sum + chars;
            }, 0,);
        },);
        // The declined block has to leave the slice, or the recipe reached
        // subdivision as nothing.
        expect(settledChars,).toBeLessThan(baselineChars ?? 0,);
      },
    },),
    it({
      name: 'COUNTS the translation block a recorded pairing leaves unclaimed as target-only at its own size, '
        + 'which the deterministic aligner reads as no block left over',
      fn: async () => {
        await using corpus = await throwawayCorpus({
          sourcePage: BLOCKY_SOURCE_PAGE,
          targetPage: BLOCKY_TARGET_PAGE,
        },);

        expect(await censusEntry({
          entryId: ENTRY_ID,
          pin: corpus.pin,
          recipe: {
            blockPairings: new Map([[
              0,
              [
                {
                  source: 0,
                  target: 0,
                },
                {
                  source: 1,
                  target: 1,
                },
                {
                  source: 2,
                  target: 2,
                },
                {
                  source: 3,
                  target: 2,
                },
              ],
            ],],),
            unrecorded: ['sectionPairing',],
          },
        },),).toEqual({
          entryId: ENTRY_ID,
          carve: 'settled-partial',
          pairingRefusal: '',
          sliceSourceChars: [44,],
          sliceTargetChars: [84,],
          unpairedSourceSections: 0,
          unpairedSourceChars: 0,
          unpairedTargetSections: 0,
          unpairedTargetChars: 0,
          targetOnlyBlocks: 1,
          targetOnlyChars: 39,
          targetOnlyBlockChars: [39,],
        },);
      },
    },),
    it({
      name: 'SIZES an added paragraph by its own characters under a recorded pairing, for a short and for a long one',
      fn: async () => {
        await using shortCorpus = await throwawayCorpus({
          sourcePage: ONE_BLOCK_SOURCE_PAGE,
          targetPage: onePlusAddedPage({ added: ADDED_SHORT_BLOCK, },),
        },);
        await using longCorpus = await throwawayCorpus({
          sourcePage: ONE_BLOCK_SOURCE_PAGE,
          targetPage: onePlusAddedPage({ added: ADDED_LONG_BLOCK, },),
        },);

        /**
         Rows read under the recorded pairing, short and long.
         */
        const recorded = [
          await censusEntry({
            entryId: ENTRY_ID,
            pin: shortCorpus.pin,
            recipe: recipeNamingOneBlock(),
          },),
          await censusEntry({
            entryId: ENTRY_ID,
            pin: longCorpus.pin,
            recipe: recipeNamingOneBlock(),
          },),
        ].map(function addedColumn(row,): readonly number[] {
          return row.targetOnlyBlockChars;
        },);
        expect(recorded,).toEqual([
          [44,],
          [227,],
        ],);
      },
    },),
    it({
      name: 'ANSWERS a settled row set aside in the refusal\'s words where the recorded block pairing names a '
        + 'translation block the carved text lacks, and measures every section by the deterministic aligner',
      fn: async () => {
        await using corpus = await throwawayCorpus({
          sourcePage: BLOCKY_SOURCE_PAGE,
          targetPage: BLOCKY_TARGET_PAGE,
        },);

        expect(await censusEntry({
          entryId: ENTRY_ID,
          pin: corpus.pin,
          recipe: {
            blockPairings: new Map([[
              0,
              [{
                source: 0,
                target: 5,
              },],
            ],],),
            unrecorded: ['sectionPairing',],
          },
        },),).toEqual({
          entryId: ENTRY_ID,
          carve: 'settled-moved',
          pairingRefusal: 'the recorded block pairing does not fit the text carved (pairing names translation '
            + 'block 5, and there are 4), so every section\'s blocks were carved by the deterministic aligner',
          sliceSourceChars: [44,],
          sliceTargetChars: [125,],
          unpairedSourceSections: 0,
          unpairedSourceChars: 0,
          unpairedTargetSections: 0,
          unpairedTargetChars: 0,
          targetOnlyBlocks: 0,
          targetOnlyChars: 0,
          targetOnlyBlockChars: [],
        },);
      },
    },),
    it({
      name: 'REFUSES an entry the clone does not carry, rather than measuring nothing',
      fn: async () => {
        await using corpus = await throwawayCorpus({ targetPage: FULL_TARGET_PAGE, },);

        await expect(censusEntry({
          entryId: 'pepperbox',
          pin: corpus.pin,
        },),).rejects.toThrow(CorpusReadError,);
      },
    },),
  ],
},);

//endregion Slice census entry tests
