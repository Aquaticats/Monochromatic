/**
 Tests for collecting publish-time content check failures as defects the page
 ships with (the owner, 2026-09-27: a settled page ships with its defects
 reported; ledger E1).

 Only a check's own refusal becomes a defect. Anything else it throws is a
 fault in the code and must still stop the entry, or a bug would ship as a
 content report.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  archiveKeptDefects,
  defectsLine,
  pageWithoutBodyDefects,
  placeholderHeadingDefects,
  type PublishCheckStep,
  publishDefects,
  tenseSwapDefects,
  type WouldShipSource,
} from '../../dist/final/node/index.mjs';

/**
 Instant the injected clock reads, so no case waits on the real one.
 */
const PRINTED_AT = new Date('2026-10-22T08:10:11.123Z',);

/**
 Clock that reads the same instant every time it is asked.

 @returns Instant the line is printed at

 @example
 ```ts
 const line = defectsLine({ entryId, defects, attempt: 2, wallClock: fixedWallClock, },);
 ```
 */
function fixedWallClock(): Date {
  return PRINTED_AT;
}

/**
 One consolidation slice as an artifact records it, shipping the kind given.

 @param sliceIndex - slice the record answers

 @param kind - what it ships

 @param reason - why the archive keeps the slice, absent in a record written before the artifact said why

 @returns Slice record, carrying only what the defect reads beside its kind

 @example
 ```ts
 const slice = sliceShipping({ sliceIndex: 1, kind: 'archive', reason: 'slate-declined', },);
 ```
 */
function sliceShipping(
  {
    sliceIndex,
    kind,
    reason,
  }: {
    readonly sliceIndex: number;
    readonly kind: 'archive' | 'unchanged';
    readonly reason?: string;
  },
): Record<string, unknown> {
  return {
    sliceIndex,
    terminal: 'incumbent-only',
    shipped: (reason === undefined) ? { kind, } : {
      kind,
      reason,
    },
  };
}

/**
 Refusal the whisker check throws when the page fails it.

 @example
 ```ts
 throw new WhiskerRefusalError();
 ```
 */
class WhiskerRefusalError extends Error {
  /**
   Declares its message safe to repeat, as every check's refusal class must.
   */
  readonly messageNamesOnly: true = true;

  /**
   Names the check so the message is recognisable in a defect.
   */
  constructor() {
    super('entry WhiskerCat drops 1 whisker',);
    this.name = 'WhiskerRefusalError';
  }
}

/**
 Refusal a different check throws, standing in for a class the whisker step
 does not own.

 @example
 ```ts
 throw new TailRefusalError();
 ```
 */
class TailRefusalError extends Error {
  /**
   Declares its message safe to repeat, as every check's refusal class must.
   */
  readonly messageNamesOnly: true = true;

  /**
   Names the check so the message is recognisable.
   */
  constructor() {
    super('entry WhiskerCat drops 1 tail',);
    this.name = 'TailRefusalError';
  }
}

/**
 Check that passes.
 */
const PASSING: PublishCheckStep = {
  check: 'headings',
  refusal: WhiskerRefusalError,
  run: function passes(): void {
    // A CLEAN PAGE throws nothing.
  },
};

/**
 Check that fails with its own refusal.
 */
const REFUSING: PublishCheckStep = {
  check: 'destinations',
  refusal: WhiskerRefusalError,
  run: function refuses(): void {
    throw new WhiskerRefusalError();
  },
};

/**
 Second check failing with its own refusal, to show order is kept.
 */
const REFUSING_FRONT_MATTER: PublishCheckStep = {
  check: 'front-matter',
  refusal: TailRefusalError,
  run: function refuses(): void {
    throw new TailRefusalError();
  },
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: publishDefects.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COLLECTS NOTHING for checks that pass, the control the collection rests on',
          fn: async () => {
            expect(publishDefects({ steps: [PASSING,], },),).toStrictEqual([],);
          },
        },),
        it({
          name: 'HOLDS THE REFUSAL CLASS TO A MARKED ONE BY ITS TYPE, since a defect repeats the refusal\'s message',
          fn: async () => {
            /**
             Step naming a class that declares no safe message, which the compiler refuses.
             */
            const unmarked: PublishCheckStep = {
              check: 'headings',
              // @ts-expect-error a class that does not declare messageNamesOnly is no refusal a defect may repeat
              refusal: RangeError,
              run: function passes(): void {},
            };
            expect(unmarked.check,).toBe('headings',);
          },
        },),
        it({
          name: 'COLLECTS A CHECK\'S OWN REFUSAL as a defect naming the check and its message',
          fn: async () => {
            expect(publishDefects({
              steps: [
                PASSING,
                REFUSING,
              ],
            },),).toStrictEqual([{
              check: 'destinations',
              message: 'entry WhiskerCat drops 1 whisker',
            },],);
          },
        },),
        it({
          name: 'RUNS EVERY CHECK after one fails, and keeps their order',
          fn: async () => {
            expect(publishDefects({
              steps: [
                REFUSING_FRONT_MATTER,
                PASSING,
                REFUSING,
              ],
            },),).toStrictEqual([
              {
                check: 'front-matter',
                message: 'entry WhiskerCat drops 1 tail',
              },
              {
                check: 'destinations',
                message: 'entry WhiskerCat drops 1 whisker',
              },
            ],);
          },
        },),
        it({
          name: 'RETHROWS ANOTHER CHECK\'S REFUSAL, since the step owns only its own class',
          fn: async () => {
            expect(() => {
              publishDefects({
                steps: [{
                  check: 'headings',
                  refusal: WhiskerRefusalError,
                  run: function refusesWrongly(): void {
                    throw new TailRefusalError();
                  },
                },],
              },);
            },).toThrow(TailRefusalError,);
          },
        },),
        it({
          name: 'RETHROWS A FAULT IN THE CODE, so a bug never ships as a content report',
          fn: async () => {
            expect(() => {
              publishDefects({
                steps: [{
                  check: 'contributor-names',
                  refusal: WhiskerRefusalError,
                  run: function faults(): void {
                    throw new TypeError('whisker is not a function',);
                  },
                },],
              },);
            },).toThrow(TypeError,);
          },
        },),
      ],
    },),

    describe({
      name: defectsLine.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PRINTS NOTHING for a clean page',
          fn: async () => {
            expect(defectsLine({
              entryId: 'WhiskerCat',
              attempt: 2,
              wallClock: fixedWallClock,
              defects: [],
            },),).toBe('',);
          },
        },),
        it({
          name: 'NAMES THE ENTRY AND EVERY FAILED CHECK in the order they ran, after the clock time and the attempt',
          fn: async () => {
            expect(defectsLine({
              entryId: 'WhiskerCat',
              attempt: 2,
              wallClock: fixedWallClock,
              defects: [
                {
                  check: 'front-matter',
                  message: 'entry WhiskerCat drops 1 tail',
                },
                {
                  check: 'destinations',
                  message: 'entry WhiskerCat drops 1 whisker',
                },
              ],
            },),).toBe('DEFECTS WhiskerCat at=2026-10-22T08:10:11.123Z attempt=2 checks=front-matter,destinations',);
          },
        },),
      ],
    },),

    describe({
      name: archiveKeptDefects.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REPORTS NOTHING where the consolidation did not run or kept no archive, the control the report rests on',
          fn: async () => {
            expect(archiveKeptDefects({
              entryId: 'WhiskerCat',
              consolidation: { kind: 'not-run', } as WouldShipSource['consolidation'],
            },),).toStrictEqual([],);
            expect(archiveKeptDefects({
              entryId: 'WhiskerCat',
              consolidation: {
                kind: 'settled',
                slices: [sliceShipping({
                  sliceIndex: 0,
                  kind: 'unchanged',
                },),],
              } as unknown as WouldShipSource['consolidation'],
            },),).toStrictEqual([],);
          },
        },),
        it({
          name: 'NAMES EVERY SLICE THE ARCHIVE KEPT in one defect, saying where no wording passed the rule '
            + '(owner, 2026-09-27, "Keep archive, ship")',
          fn: async () => {
            expect(archiveKeptDefects({
              entryId: 'WhiskerCat',
              consolidation: {
                kind: 'settled',
                slices: [
                  sliceShipping({
                    sliceIndex: 1,
                    kind: 'archive',
                    reason: 'no-wording-passed',
                  },),
                  sliceShipping({
                    sliceIndex: 2,
                    kind: 'unchanged',
                  },),
                  sliceShipping({
                    sliceIndex: 4,
                    kind: 'archive',
                    reason: 'no-wording-passed',
                  },),
                ],
              } as unknown as WouldShipSource['consolidation'],
            },),).toStrictEqual([{
              check: 'no-valid-wording',
              message: 'entry WhiskerCat kept the archive at slice 1, slice 4, where no wording passed the '
                + 'deterministic publication rule',
            },],);
          },
        },),
        it({
          name: 'SAYS WHAT HAPPENED AT EACH SLICE, not that no wording passed where a final round that could not '
            + 'seat two judges, or a slate that declined, kept the archive (XingZ6022: sixteen slices, a translate '
            + 'wording had passed at each)',
          fn: async () => {
            expect(archiveKeptDefects({
              entryId: 'WhiskerCat',
              consolidation: {
                kind: 'settled',
                slices: [
                  sliceShipping({
                    sliceIndex: 14,
                    kind: 'archive',
                    reason: 'final-round-unseated',
                  },),
                  sliceShipping({
                    sliceIndex: 15,
                    kind: 'archive',
                    reason: 'slate-declined',
                  },),
                  sliceShipping({
                    sliceIndex: 16,
                    kind: 'archive',
                    reason: 'final-round-unseated',
                  },),
                  sliceShipping({
                    sliceIndex: 17,
                    kind: 'archive',
                    reason: 'no-wording-passed',
                  },),
                ],
              } as unknown as WouldShipSource['consolidation'],
            },),).toStrictEqual([{
              check: 'no-valid-wording',
              message: 'entry WhiskerCat kept the archive at slice 17, where no wording passed the deterministic '
                + 'publication rule; at slice 14, slice 16, where the final round could not seat two judges and no '
                + 'wording two judges chose passed the rule; at slice 15, where the slate judges declined every '
                + 'wording that passed the rule and none drew two ballots',
            },],);
          },
        },),
        it({
          name: 'DOES NOT CLAIM A REASON AN OLDER ARTIFACT NEVER RECORDED, and says so',
          fn: async () => {
            expect(archiveKeptDefects({
              entryId: 'WhiskerCat',
              consolidation: {
                kind: 'settled',
                slices: [sliceShipping({
                  sliceIndex: 3,
                  kind: 'archive',
                },),],
              } as unknown as WouldShipSource['consolidation'],
            },),).toStrictEqual([{
              check: 'no-valid-wording',
              message: 'entry WhiskerCat kept the archive at slice 3, where the artifact records no reason',
            },],);
          },
        },),
      ],
    },),

    describe({
      name: tenseSwapDefects.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES EVERY SLICE WHERE THE TRANSLATE TEXT SHIPPED OVER A STANDING THE TENSE DETECTOR FLAGGED, and '
            + 'says nothing for a translate winner that replaced a refused standing, an archive or a slice that '
            + 'settled on no change (owner, 2026-10-29, "Swap only where flagged")',
          fn: async () => {
            /**
             One consolidation slice shipping the translate lane winner for the reason given.

             @param sliceIndex - slice the record answers
             @param reason - why the winner shipped

             @returns Slice record, carrying only what the defect reads

             @example
             ```ts
             const slice = winnerSlice({ sliceIndex: 11, reason: 'present-tense-standing', });
             ```
             */
            function winnerSlice(
              {
                sliceIndex,
                reason,
              }: {
                readonly sliceIndex: number;
                readonly reason: string;
              },
            ): Record<string, unknown> {
              return {
                sliceIndex,
                terminal: 'slate-declined-standing',
                shipped: {
                  kind: 'translate-winner',
                  text: 'Mittens was a tabby.',
                  reason,
                },
              };
            }
            expect(tenseSwapDefects({
              entryId: 'WhiskerCat',
              consolidation: {
                kind: 'settled',
                slices: [
                  winnerSlice({ sliceIndex: 11, reason: 'present-tense-standing', },),
                  winnerSlice({ sliceIndex: 12, reason: 'refused-standing', },),
                  sliceShipping({ sliceIndex: 13, kind: 'archive', reason: 'slate-declined', },),
                  sliceShipping({ sliceIndex: 14, kind: 'unchanged', },),
                  winnerSlice({ sliceIndex: 16, reason: 'present-tense-standing', },),
                ],
              } as unknown as WouldShipSource['consolidation'],
            },),).toStrictEqual([{
              check: 'tense-swap',
              message: 'entry WhiskerCat shipped the translate lane text at slice 11, slice 16, in place of a standing '
                + 'the tense detector flagged, where the final round could not seat two judges',
            },],);
          },
        },),
        it({
          name: 'REPORTS NOTHING where the consolidation did not run or swapped no slice, the control the report rests on',
          fn: async () => {
            expect(tenseSwapDefects({
              entryId: 'WhiskerCat',
              consolidation: { kind: 'not-run', } as WouldShipSource['consolidation'],
            },),).toStrictEqual([],);
            expect(tenseSwapDefects({
              entryId: 'WhiskerCat',
              consolidation: {
                kind: 'settled',
                slices: [sliceShipping({ sliceIndex: 0, kind: 'unchanged', },),],
              } as unknown as WouldShipSource['consolidation'],
            },),).toStrictEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: pageWithoutBodyDefects.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES A PAGE WITH NO BLOCK WHILE THE ORIGINAL HOLDS SOME, whether the page is empty, whitespace, a '
            + 'byte order mark or front matter alone, saying how many blocks the original holds and nothing it quotes',
          fn: async () => {
            /**
             Byte order mark a page may open with.
             */
            const mark = String.fromCodePoint(0xFE_FF,);
            /**
             Original of two blocks.
             */
            const sourceText = '---\nname: 猫\n---\n\n## 简介\n\n一只虎斑猫。\n';
            for (const pageText of ['', '  \n\n', mark, '---\nname: Cat\n---\n', `${mark}---\nname: Cat\n---\n\n`,]) {
              expect(pageWithoutBodyDefects({
                entryId: 'WhiskerCat',
                sourceText,
                pageText,
              },),).toStrictEqual([{
                check: 'page-without-body',
                message: 'entry WhiskerCat ships a page with no body, where the original holds 2 blocks',
              },],);
            }
          },
        },),
        it({
          name: 'SAYS NOTHING for a page that holds a block, and nothing for an original that holds none, since '
            + 'no page can carry a body the original lacks',
          fn: async () => {
            expect(pageWithoutBodyDefects({
              entryId: 'WhiskerCat',
              sourceText: '## 简介\n\n一只虎斑猫。\n',
              pageText: '## Description\n',
            },),).toStrictEqual([],);
            expect(pageWithoutBodyDefects({
              entryId: 'WhiskerCat',
              sourceText: '---\nname: 猫\n---\n',
              pageText: '',
            },),).toStrictEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: placeholderHeadingDefects.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES A PAGE HEADING OF PLACEHOLDER MARKS ONLY WHERE THE ORIGINAL HEADING HAS WORDS: underscores, '
            + 'dashes, blanks, an empty heading, an ellipsis and the number of a series with a placeholder after '
            + 'it (XingZ6022: `### Part Nine: __` over `### 其九：空白`), saying the page heading and nothing it quotes '
            + '(owner, 2026-10-29, "Report it as a defect")',
          fn: async () => {
            for (const heading of [
              '### __',
              '### ——',
              '### ',
              '###',
              '### - - -',
              '### …',
              '### ＿＿',
              '### 　',
              '### Part Nine: __',
              '### Part Nine: ——',
            ]) {
              expect(placeholderHeadingDefects({
                entryId: 'WhiskerCat',
                sourceText: '### 其九：空白\n\n正文。\n',
                pageText: `${heading}\n\nBody.\n`,
              },),).toStrictEqual([{
                check: 'placeholder-heading',
                message: 'entry WhiskerCat ships a heading of placeholder marks only where the original heading '
                  + 'has words, at page heading 1',
              },],);
            }
          },
        },),
        it({
          name: 'SAYS NOTHING for a heading with words, a number with a title after it, a number alone, and a '
            + 'heading the original also leaves without words',
          fn: async () => {
            for (const heading of [
              '### Part Nine: Blank',
              '### Part Nine:',
              '### Blank',
              '### 9',
              '### 其九：空白',
            ]) {
              expect(placeholderHeadingDefects({
                entryId: 'WhiskerCat',
                sourceText: '### 其九：空白\n\n正文。\n',
                pageText: `${heading}\n\nBody.\n`,
              },),).toStrictEqual([],);
            }
            for (const original of ['### __', '### ——', '### ', '###', '### Part Nine: __',]) {
              expect(placeholderHeadingDefects({
                entryId: 'WhiskerCat',
                sourceText: `${original}\n\n正文。\n`,
                pageText: '### __\n\nBody.\n',
              },),).toStrictEqual([],);
            }
          },
        },),
        it({
          name: 'PAIRS HEADINGS BY POSITION where the original and the page hold as many, and names every page '
            + 'heading that has marks only where its original has words, not one the original also leaves blank',
          fn: async () => {
            expect(placeholderHeadingDefects({
              entryId: 'WhiskerCat',
              sourceText: '### 其一：猫\n\n正文。\n\n### 其二：狗\n\n正文。\n\n### 其三：——\n\n正文。\n\n### 其四：鸟\n\n正文。\n',
              pageText: '### Part One: Cat\n\nBody.\n\n### Part Two: __\n\nBody.\n\n### Part Three: __\n\nBody.\n\n'
                + '### Part Four: ——\n\nBody.\n',
            },),).toStrictEqual([{
              check: 'placeholder-heading',
              message: 'entry WhiskerCat ships 2 headings of placeholder marks only where the original headings '
                + 'have words, at page heading 2, page heading 4',
            },],);
          },
        },),
        it({
          name: 'COMPARES THE COUNTS where the original and the page hold different numbers of headings, since '
            + 'nothing is paired then: more placeholder headings on the page than in the original is a defect, '
            + 'as many or fewer is not',
          fn: async () => {
            expect(placeholderHeadingDefects({
              entryId: 'WhiskerCat',
              sourceText: '## 简介\n\n正文。\n\n## 生平\n\n正文。\n',
              pageText: '## Description\n\nBody.\n\n## __\n\nBody.\n\n## Life\n\nBody.\n',
            },),).toStrictEqual([{
              check: 'placeholder-heading',
              message: 'entry WhiskerCat ships 1 heading of placeholder marks only against 0 in the original, whose '
                + '2 headings are not paired with the page\'s 3, at page heading 2',
            },],);
            expect(placeholderHeadingDefects({
              entryId: 'WhiskerCat',
              sourceText: '## __\n\n正文。\n\n## 生平\n\n正文。\n',
              pageText: '## Description\n\nBody.\n\n## __\n\nBody.\n\n## Life\n\nBody.\n',
            },),).toStrictEqual([],);
          },
        },),
        it({
          name: 'SAYS NOTHING for a page and an original with no heading, and reads front matter as no heading',
          fn: async () => {
            expect(placeholderHeadingDefects({
              entryId: 'WhiskerCat',
              sourceText: '一只虎斑猫。\n',
              pageText: 'A tabby.\n',
            },),).toStrictEqual([],);
            expect(placeholderHeadingDefects({
              entryId: 'WhiskerCat',
              sourceText: '---\nname: 猫\n---\n\n## 简介\n\n正文。\n',
              pageText: '---\nname: Cat\n---\n\n## Description\n\nBody.\n',
            },),).toStrictEqual([],);
          },
        },),
      ],
    },),
  ],
},);
