/**
 Tests for what one entry contributes to the window trial: its two pages read
 from a scripted corpus, prepared, screened, and the flagged slices drawn with
 their controls.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CorpusReadError,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source';
import { drawEntry, } from '../../dist/final/node/index.mjs';
import {
  LACKED_COMMIT_SHA,
  makeCloneHoldingOneCommit,
  PAIRED_ENTRY,
} from '@monochromatic-dev/translation-repair-adjudication/ts/corpus-lacked-commit.test-fixture.ts';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';
import {
  CAT_PIN,
  CLEAN_SOURCE,
  CLEAN_TARGET,
  DEATH_LINE,
  FLAGGED_IDENTITY,
  FLAGGED_SOURCE,
  FLAGGED_TARGET,
} from './window-trial-probe-draw.test-fixture.ts';

/**
 Reader of a scripted corpus that records the paths it was asked for.

 @param pages - text of each path the corpus holds

 @param asked - array each path asked for lands in

 @returns Reader that answers a held path and refuses any other as git does

 @example
 ```ts
 const readPage = scriptedPages({ pages: new Map([['people/E/page.md', '猫',],],), asked: [], },);
 ```
 */
function scriptedPages(
  {
    pages,
    asked,
  }: {
    readonly pages: ReadonlyMap<string, string>;
    readonly asked: string[];
  },
): (input: { readonly pin: typeof CAT_PIN; readonly relPath: string; }) => Promise<string> {
  return function readPage(
    { relPath, }: { readonly pin: typeof CAT_PIN; readonly relPath: string; },
  ): Promise<string> {
    asked.push(relPath,);
    /**
     Text of the page, when the corpus holds it.
     */
    const held = pages.get(relPath,);
    if (held === undefined)
      return Promise.reject(new CorpusReadError({
        detail: `${relPath} at ${CAT_PIN.commitSha}`,
        cause: { stderr: `fatal: path '${relPath}' does not exist in '${CAT_PIN.commitSha}'`, },
        commit: 'held',
      },),);
    return Promise.resolve(held,);
  };
}

await describe({
  name: drawEntry.name,
  children: [
    it({
      name: 'DRAWS the flagged slice and one control, with every prepared slice, and reads the two pages of the entry',
      fn: async () => {
        const asked: string[] = [];
        const { logger, } = capturingLoggerPair();

        /**
         What the entry contributes.
         */
        const drawn = await drawEntry({
          entryId: 'Mittens',
          pin: CAT_PIN,
          readPage: scriptedPages({
            pages: new Map([
              [
                'people/Mittens/page.md',
                FLAGGED_SOURCE,
              ],
              [
                'people/Mittens/page.en.md',
                FLAGGED_TARGET,
              ],
            ],),
            asked,
          },),
          l: logger,
        },);

        expect(drawn.picks,).toEqual([
          {
            entryId: 'Mittens',
            sliceIndex: 1,
            sliceClass: 'untranslated',
          },
          {
            entryId: 'Mittens',
            sliceIndex: 2,
            sliceClass: 'control-unflagged',
          },
        ],);
        expect(drawn.slices.length,).toBe(3,);
        // THE INFO RECORD IS READ AFTER BOTH PAGES, so an entry with one side is skipped before it is asked for.
        expect(asked,).toEqual([
          'people/Mittens/page.md',
          'people/Mittens/page.en.md',
          'people/Mittens/info.yml',
        ],);
      },
    },),
    it({
      name: 'READS THE DEATH THE ENTRY\'S info.yml RECORDS and returns the page facts of the preparation it drew, the flag '
        + 'and the identity block naming the date, so the arms are judged on the sheet the pass asks',
      fn: async () => {
        const { logger, lines, } = capturingLoggerPair();

        /**
         What the entry contributes.
         */
        const drawn = await drawEntry({
          entryId: 'Mittens',
          pin: CAT_PIN,
          readPage: scriptedPages({
            pages: new Map([
              [
                'people/Mittens/page.md',
                FLAGGED_SOURCE,
              ],
              [
                'people/Mittens/page.en.md',
                FLAGGED_TARGET,
              ],
              [
                'people/Mittens/info.yml',
                'info:\n  died: 2025-02-26\n',
              ],
            ],),
            asked: [],
          },),
          l: logger,
        },);

        expect(drawn.sheetFacts,).toEqual({
          identityContext: `${DEATH_LINE}\n${FLAGGED_IDENTITY}`,
          lifeEnded: true,
        },);
        expect(drawn.picks.length,).toBe(2,);
        expect(lines,).toEqual([],);
      },
    },),
    it({
      name: 'DRAWS THE ENTRY WITH NO DEATH, and says so, when it has no info.yml at the pin',
      fn: async () => {
        const { logger, lines, } = capturingLoggerPair();

        /**
         What the entry contributes.
         */
        const drawn = await drawEntry({
          entryId: 'Mittens',
          pin: CAT_PIN,
          readPage: scriptedPages({
            pages: new Map([
              [
                'people/Mittens/page.md',
                FLAGGED_SOURCE,
              ],
              [
                'people/Mittens/page.en.md',
                FLAGGED_TARGET,
              ],
            ],),
            asked: [],
          },),
          l: logger,
        },);

        expect(drawn.sheetFacts,).toEqual({ identityContext: FLAGGED_IDENTITY, },);
        expect(drawn.picks.length,).toBe(2,);
        expect(lines,).toEqual([
          `ENTRY INFO entry=Mittens: no info.yml at the pin (corpus read failed for people/Mittens/info.yml at ${CAT_PIN.commitSha} `
            + '(missing-object); the commit has no such path: check the path, or pin a commit that has it.), '
            + 'so no death is recorded for it',
        ],);
      },
    },),
    it({
      name: 'DRAWS THE ENTRY WITH NO DEATH, and warns that the record was unreadable, when its info.yml is not YAML, '
        + 'since a run always ships and the arms of its slices are still bought',
      fn: async () => {
        const { logger, lines, } = capturingLoggerPair();

        /**
         What the entry contributes.
         */
        const drawn = await drawEntry({
          entryId: 'Mittens',
          pin: CAT_PIN,
          readPage: scriptedPages({
            pages: new Map([
              [
                'people/Mittens/page.md',
                FLAGGED_SOURCE,
              ],
              [
                'people/Mittens/page.en.md',
                FLAGGED_TARGET,
              ],
              [
                'people/Mittens/info.yml',
                'info: [\n',
              ],
            ],),
            asked: [],
          },),
          l: logger,
        },);

        expect(drawn.sheetFacts,).toEqual({ identityContext: FLAGGED_IDENTITY, },);
        expect(drawn.picks.length,).toBe(2,);
        expect(lines,).toEqual([
          'ENTRY INFO entry=Mittens: The info.yml of entry Mittens at the pin refused to parse at line 2 column 1 '
            + '(BAD_INDENT); corpus metadata parses upstream, so this signals corruption. No death is recorded for the entry, '
            + 'and it goes on as one whose info.yml records none',
        ],);
      },
    },),
    it({
      name: 'DRAWS NO PICK from an entry the screen flags nothing in, and still returns its prepared slices',
      fn: async () => {
        const { logger, } = capturingLoggerPair();

        /**
         What the entry contributes.
         */
        const drawn = await drawEntry({
          entryId: 'Mittens',
          pin: CAT_PIN,
          readPage: scriptedPages({
            pages: new Map([
              [
                'people/Mittens/page.md',
                CLEAN_SOURCE,
              ],
              [
                'people/Mittens/page.en.md',
                CLEAN_TARGET,
              ],
            ],),
            asked: [],
          },),
          l: logger,
        },);

        expect(drawn.picks,).toEqual([],);
        expect(drawn.slices.length,).toBe(2,);
        expect(drawn.sheetFacts,).toEqual({
          identityContext: 'NAMES AND LINKED TEXT THE ARCHIVE RENDERS ON THIS PAGE (render the same person or title the same way '
            + 'everywhere; a declared name inside a title takes its declared form):\n- 第一节 (heading): "Section one"\n'
            + '- 第二节 (heading): "Section two"',
        },);
      },
    },),
    it({
      name: 'STEPS PAST an entry with one side, drawing nothing and logging the skip',
      fn: async () => {
        const { logger, lines, } = capturingLoggerPair();

        /**
         What the entry contributes.
         */
        const drawn = await drawEntry({
          entryId: 'Mittens',
          pin: CAT_PIN,
          readPage: scriptedPages({
            pages: new Map([
              [
                'people/Mittens/page.md',
                CLEAN_SOURCE,
              ],
            ],),
            asked: [],
          },),
          l: logger,
        },);

        expect(drawn,).toEqual({
          picks: [],
          sheetFacts: {},
          slices: [],
        },);
        expect(lines,).toEqual([
          'Mittens: skipped, CorpusReadError: corpus read failed for people/Mittens/page.en.md at '
            + `${CAT_PIN.commitSha} (missing-object); the commit has no such path: check the path, or pin a `
            + 'commit that has it.',
        ],);
      },
    },),
    it({
      name: 'REJECTS with a corpus read that failed for any other reason, since a missing clone must not read as a missing page',
      fn: async () => {
        const { logger, lines, } = capturingLoggerPair();

        /**
         What the draw rejected with.
         */
        const refusal = await rejectionOf({
          promise: drawEntry({
            entryId: 'Mittens',
            pin: CAT_PIN,
            readPage: function unreadable(): Promise<string> {
              return Promise.reject(new CorpusReadError({
                detail: 'people/Mittens/page.md',
                cause: new Error('spawn failed',),
                commit: 'unasked',
              },),);
            },
            l: logger,
          },),
        },);

        expect(refusal,).toBeInstanceOf(CorpusReadError,);
        expect(String(refusal,),).toBe(
          'CorpusReadError: corpus read failed for people/Mittens/page.md (other); '
            + 'the read failed another way: run the same git read in the clone by hand to see why.',
        );
        expect(lines,).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES a pin whose commit the clone lacks, read through the real page reader, instead of '
        + 'skipping the entry as one with a single side',
      fn: async () => {
        await using clone = await makeCloneHoldingOneCommit();
        const { logger, lines, } = capturingLoggerPair();

        /**
         What the draw rejected with.
         */
        const refusal = await rejectionOf({
          promise: drawEntry({
            entryId: PAIRED_ENTRY,
            pin: {
              cloneDir: clone.cloneDir,
              commitSha: LACKED_COMMIT_SHA,
            },
            readPage: readCorpusFile,
            l: logger,
          },),
        },);

        expect(refusal,).toBeInstanceOf(CorpusReadError,);
        expect((refusal instanceof CorpusReadError) && refusal.kind,).toBe('missing-commit',);
        expect(String(refusal,),).toBe(
          `CorpusReadError: corpus read failed for ${LACKED_COMMIT_SHA}:people/${PAIRED_ENTRY}/page.md (missing-commit); `
            + 'the clone holds no commit by that revision: fetch it, or pin '
            + 'a commit the clone holds.',
        );
        expect(lines,).toEqual([],);
      },
    },),
  ],
},);
