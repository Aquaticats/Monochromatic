/**
 Tests for what the page-level consistency pass does with apostrophes and
 quotation marks: a quotation mark is never taken for an apostrophe and deleted,
 an apostrophe is carried into a place that held none only where the page keeps
 it an apostrophe, an apostrophe that arrives is written in the form of the
 place it arrives in, and a splice whose words the floors always refuse leaves
 the paragraph beside it to the writers.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  RUN_LATE_JUDGES,
  RUN_TRANSLATORS,
} from '@monochromatic-dev/translation-repair-run';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  type ArtifactConsistency,
  parseConsistency,
} from '@monochromatic-dev/translation-repair-artifact';
import { runPassConsistency, } from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  ballotText,
  choosing,
  otherSheetBehaviour,
  type SeatBehaviour,
  type SeatScript,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';
import {
  type CatRun,
  candidateKindsOf,
  runCatPass,
} from './consistency-run.test-fixture.ts';
import {
  candidateTextsOf,
  currentParagraphOf,
  sheetBlockOf,
  sheetKindOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-sheet-reading.test-fixture.ts';

/**
 The repeated sentence of the source, with the sentence about the kitchen that follows it.
 */
const SENTENCE = '窗台上的阳光让小猫睡着了。厨房里很暖和。';

/**
 The ballots of a page: every judge quotes every place and chooses one.

 @param choice - label every judge chooses
 @param quotes - what every judge quotes for the places, in order
 @returns What each seat does when asked
 @example
 ```ts
 const behave = ballotsOf({ choice: 'OCCURRENCE 3', quotes: ['the cats', 'the dogs', 'the kittens',], },);
 ```
 */
function ballotsOf(
  {
    choice,
    quotes,
  }: {
    readonly choice: string;
    readonly quotes: readonly string[];
  },
): SeatScript {
  /**
   What every judge quotes, under the labels of the places.
   */
  const labelled = quotes.map(function labelOne(quote, at,): Record<string, string> {
    return { label: `OCCURRENCE ${String(at + 1,)}`, quote, };
  },);
  /**
   The seats that are silent, the late ones past the fourth.
   */
  const silent = new Set(RUN_LATE_JUDGES.slice(4,),);
  return function judgeSpeaks({ seat, },): SeatBehaviour {
    return silent.has(seat,)
      ? { kind: 'throws', }
      : { kind: 'answers', text: ballotText({ quotes: labelled, choice, keep: [], },), };
  };
}

/**
 Runs the pass over a page whose first section holds every place but the last and whose second section holds the last, every
 judge choosing one place.

 @param first - English of the first section
 @param second - English of the second section
 @param quotes - what every judge quotes for the places in order, the last in the second section
 @param choice - position of the place every judge chooses, counted from one
 @param others - what the writers and the bench of candidates do, the page's own by default
 @returns Run
 @example
 ```ts
 const run = await apostropheRun({ first: 'Fill the cats bowl.', second: 'Then the dogs nap.', quotes: ['the cats', 'the dogs',], choice: 2, },);
 ```
 */
async function apostropheRun(
  {
    first,
    second,
    quotes,
    choice,
    others = otherSheetBehaviour,
  }: {
    readonly first: string;
    readonly second: string;
    readonly quotes: readonly string[];
    readonly choice: number;
    readonly others?: SeatScript;
  },
): Promise<CatRun> {
  await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
  return await runCatPass({
    behave: ballotsOf({ choice: `OCCURRENCE ${String(choice,)}`, quotes, },),
    others,
    entryCacheDir: dir.path,
    prepared: prepareDocumentPair({
      sourceText: `## 一\n\n${SENTENCE.repeat(quotes.length - 1,)}\n\n## 二\n\n窗台上的阳光让小猫睡着了。\n`,
      targetText: `## One\n\n${first}\n\n## Two\n\n${second}\n`,
    },),
  },);
}

/**
 What the pass recorded of the first group.

 @param consistency - section
 @returns Outcomes of the places, texts of the slices the pass changed and findings that name a replacement left out
 @example
 ```ts
 const seen = seenIn({ consistency, },);
 ```
 */
function seenIn(
  { consistency, }: { readonly consistency: ArtifactConsistency; },
): {
  readonly outcomes: readonly string[];
  readonly changed: readonly (readonly [number, string])[];
  readonly leftOut: readonly string[];
} {
  return {
    outcomes: nonNullishOrThrow(consistency.groups[0],).occurrences
      .map(function outcomeOf({ outcome, },): string {
        return outcome;
      },),
    changed: consistency.slices.map(function pairOf({ sliceIndex, text, },): readonly [number, string] {
      return [sliceIndex, text,];
    },),
    leftOut: consistency.findings.filter(function isLeftOut(finding,): boolean {
      return finding.includes('was left out because the apostrophe it carries in',);
    },),
  };
}

/**
 The section as a reader of generation seventeen reads it back from a copy of the file.

 @param consistency - section the pass wrote
 @returns What the reader makes of it, which is the section itself when it reads back
 @throws {@link Error} when the reader refuses the section
 @example
 ```ts
 expect(readBack({ consistency, },),).toEqual(consistency,);
 ```
 */
function readBack({ consistency, }: { readonly consistency: ArtifactConsistency; },): ArtifactConsistency {
  return parseConsistency({
    path: 'cat.consistency',
    generation: 17,
    sliceIndexes: new Set([0, 1,],),
    value: structuredClone(consistency,),
  },);
}

/**
 The writers and the bench of a page whose first paragraph runs over a blank line: a writer puts the chosen words in each
 receiving place of its paragraph and changes the warm sentence, and the judges choose the paragraph with the change.

 @param seat - seat asked
 @param sheet - sheet it was shown
 @returns Behaviour of the seat
 @example
 ```ts
 const behave: SeatScript = warmWriters;
 ```
 */
function warmWriters(
  {
    seat,
    sheet,
  }: {
    readonly seat: RosterModelId;
    readonly before: number;
    readonly sheet: string;
  },
): SeatBehaviour {
  /**
   The question the sheet asks.
   */
  const kind = sheetKindOf({ sheet, },);
  if (kind === 'writer') {
    if (RUN_TRANSLATORS.indexOf(seat,) > 1)
      return { kind: 'throws', };
    return {
      kind: 'answers',
      text: JSON.stringify({
        translation: [1, 2, 3,].reduce(
          function replaced(paragraph: string, label,): string {
            /**
             The words the sheet asks to replace under this number, none when the sheet names no such place.
             */
            const words = sheetBlockOf({ sheet, label: `THE WORDS TO REPLACE ${String(label,)}`, },);
            return (words === '') ? paragraph : paragraph.replace(words, sheetBlockOf({ sheet, label: 'CHOSEN RENDERING', },),);
          },
          currentParagraphOf({ sheet, },),
        )
          .replace('was warm', 'felt warm',),
      },),
    };
  }
  if (kind === 'choice') {
    if (RUN_LATE_JUDGES.indexOf(seat,) > 3)
      return { kind: 'throws', };
    return choosing({
      choice: candidateTextsOf({ sheet, },)
        .find(function carriesTheChange({ text, },): boolean {
          return text.includes('felt warm',);
        },)
        ?.label
        ?? '',
    },);
  }
  return otherSheetBehaviour({ sheet, },);
}

/**
 Runs the pass over a page whose first place runs over a blank line and whose splice the floors always refuse, for the
 English that follows the first place's words.

 @param after - characters that follow the first place's words
 @returns Outcomes, kinds of candidates each place's carry holds and findings that name a paragraph no writer was asked about
 @example
 ```ts
 const seen = await residueSeen({ after: ' said she', },);
 ```
 */
async function residueSeen(
  { after, }: { readonly after: string; },
): Promise<{
  readonly outcomes: readonly string[];
  readonly kinds: unknown;
  readonly notAsked: readonly string[];
}> {
  const run = await apostropheRun({
    first: `The sunny window put the kitten\n\nto sleep${after}. The kitchen was warm. The sun on the sill sent the kitten off to sleep.`
      + '\n\nThe window sun made the kitten. The kitchen was warm.',
    second: 'The window sun made the kitten fall asleep.',
    quotes: [
      'The sunny window put the kitten\n\nto sleep',
      'The sun on the sill sent the kitten off to sleep',
      'The window sun made the kitten',
      'The window sun made the kitten fall asleep',
    ],
    choice: 4,
    others: warmWriters,
  },);
  expect(readBack({ consistency: run.consistency, },),).toEqual(run.consistency,);
  return {
    outcomes: seenIn({ consistency: run.consistency, },).outcomes,
    kinds: nonNullishOrThrow(run.consistency.groups[0],).carries?.map(function kindsOf({ candidates, },): unknown {
      return candidateKindsOf({ candidates, },);
    },),
    notAsked: run.consistency.findings.filter(function isNotAsked(finding,): boolean {
      return finding.includes('consistency-writers-not-asked',);
    },),
  };
}

/**
 The note the archive's translators put above a span of English they kept as the original.
 */
const TRANSLATORS_NOTE = '<!-- 这段话以下全部，原文都是英文，中文是反向翻译的 -->';

/**
 Runs the pass over a page of three sections whose middle one is a translators' note and a sealed span of the archive's own
 English, which belongs to no slice, between a section that holds the chosen place and a section that holds the receiving one.

 @param sealed - English the translators kept, which the page carries between the sections
 @returns Run, whose receiving place stands in the last section
 @example
 ```ts
 const run = await sealedRun({ sealed: 'She said ‘fill the bowls with water.', },);
 ```
 */
async function sealedRun({ sealed, }: { readonly sealed: string; },): Promise<CatRun> {
  await using dir = await scratchDir({ prefix: 'pass-consistency-sealed-', },);
  return await runCatPass({
    behave: ballotsOf({ choice: 'OCCURRENCE 1', quotes: ['the dogs', 'the kittens',], },),
    others: otherSheetBehaviour,
    entryCacheDir: dir.path,
    prepared: prepareDocumentPair({
      sourceText: '## 节1\n\n窗台上的阳光让小猫睡着了。\n\n## 节2\n\n别的话。\n\n## 节3\n\n窗台上的阳光让小猫睡着了。\n',
      targetText: `## Part 1\n\nThen the dogs now.\n\n## Part 2\n\n${TRANSLATORS_NOTE}\n\n${sealed}\n\n## Part 3\n\nThen feed the kittens’ now.\n`,
      sealArchiveOriginal: true,
    },),
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runPassConsistency.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS THE CLOSING QUOTATION MARK OF A QUOTATION THAT OPENED IN A SEALED SPAN BETWEEN THE SLICES: the opener stands '
            + 'in text no slice holds, the page that will be published holds it, so the place is `kept-current` and the page '
            + 'changes nowhere; the same page without the opener reads the mark as an apostrophe of the replaced words',
          fn: async () => {
            const opened = await sealedRun({ sealed: 'She said ‘fill the bowls with water.', },);
            const plain = await sealedRun({ sealed: 'She said fill the bowls with water.', },);
            expect([opened, plain,].map(function seen({ consistency, },): unknown {
              return seenIn({ consistency, },);
            },),).toEqual([
              { outcomes: ['chosen', 'kept-current',], changed: [], leftOut: [], },
              { outcomes: ['chosen', 'replaced',], changed: [[2, '## Part 3\n\nThen feed the dogs now.',],], leftOut: [], },
            ],);
          },
        },),
        it({
          name: 'CARRIES A PROVEN APOSTROPHE INTO A PLACE THAT HELD NONE WHERE THE PAGE KEEPS IT AN APOSTROPHE: the places that held '
            + '`folks’` and `the family` read the chosen `the parents’`, in either section, and the section reads back',
          fn: async () => {
            const { consistency, } = await apostropheRun({
              first: 'Fill the parents’ bowl. The kitchen was warm. Fill the folks’ bowl.',
              second: 'Then the family naps.',
              quotes: ['the parents', 'the folks', 'the family',],
              choice: 1,
            },);
            expect(seenIn({ consistency, },),).toEqual({
              outcomes: ['chosen', 'replaced', 'replaced',],
              changed: [
                [0, '## One\n\nFill the parents’ bowl. The kitchen was warm. Fill the parents’ bowl.',],
                [1, '## Two\n\nThen the parents’ naps.',],
              ],
              leftOut: [],
            },);
            expect(readBack({ consistency, },),).toEqual(consistency,);
          },
        },),
        it({
          name: 'LEAVES A PLACE AS IT WAS WHEN THE APOSTROPHE IT WOULD GAIN READS AS A QUOTATION MARK ON ITS PAGE: an opener stands '
            + 'before it, so the place is `kept-current`, a finding says so in plain words, the other places are carried, and '
            + 'the section reads back',
          fn: async () => {
            const { consistency, } = await apostropheRun({
              first: 'Fill the parents’ bowl. The kitchen was warm. Fill the folks’ bowl.',
              second: '‘Then the family naps.',
              quotes: ['the parents', 'the folks', 'the family',],
              choice: 1,
            },);
            expect(seenIn({ consistency, },),).toEqual({
              outcomes: ['chosen', 'replaced', 'kept-current',],
              changed: [[0, '## One\n\nFill the parents’ bowl. The kitchen was warm. Fill the parents’ bowl.',],],
              leftOut: [
                'slice 1: the consistency replacement of group 1 OCCURRENCE 3 was left out because the apostrophe it carries in '
                  + 'would read as a quotation mark there, since another quotation mark of the page could pair with it; the place '
                  + 'keeps its words',
              ],
            },);
            expect(readBack({ consistency, },),).toEqual(consistency,);
          },
        },),
        it({
          name: 'CARRIES THE APOSTROPHE INTO LATER PLACES WHEN THE ONLY OTHER MARK IS A PLURAL POSSESSIVE AFTER CLOSING EMPHASIS: the '
            + 'mark after `*kittens*` reads as a trailing edge mark nothing pairs with, so it blocks no later gain, while the place '
            + 'inside the emphasis keeps its words, since the apostrophe would stand twice',
          fn: async () => {
            const { consistency, } = await apostropheRun({
              first: 'Fill the dogs’ bowl. The *kittens*’ bowl is empty. A cat naps.',
              second: 'Then the family naps.',
              quotes: ['dogs', 'kittens', 'cat', 'family',],
              choice: 1,
            },);
            expect(seenIn({ consistency, },),).toEqual({
              outcomes: ['chosen', 'kept-current', 'replaced', 'replaced',],
              changed: [
                [0, '## One\n\nFill the dogs’ bowl. The *kittens*’ bowl is empty. A dogs’ naps.',],
                [1, '## Two\n\nThen the dogs’ naps.',],
              ],
              leftOut: [],
            },);
            expect(readBack({ consistency, },),).toEqual(consistency,);
          },
        },),
        it({
          name: 'LEAVES A PLACE AS IT WAS WHEN THE QUOTATION MARK THAT COULD PAIR WITH THE APOSTROPHE IT WOULD GAIN STANDS IN AN '
            + 'EARLIER SLICE: the opener stands in the first section after the places that give the mark, the gain is in the '
            + 'second, the place is `kept-current` with the finding, and the same page without the opener carries the mark',
          fn: async () => {
            const quotes = ['the parents', 'the folks', 'the family',];
            const opened = await apostropheRun({
              first: 'Fill the parents’ bowl. The kitchen was warm. Fill the folks’ bowl. She said ‘hello.',
              second: 'Then the family naps.',
              quotes,
              choice: 1,
            },);
            const plain = await apostropheRun({
              first: 'Fill the parents’ bowl. The kitchen was warm. Fill the folks’ bowl. She said hello.',
              second: 'Then the family naps.',
              quotes,
              choice: 1,
            },);
            expect([seenIn({ consistency: opened.consistency, },), seenIn({ consistency: plain.consistency, },),],).toEqual([
              {
                outcomes: ['chosen', 'replaced', 'kept-current',],
                changed: [
                  [
                    0,
                    '## One\n\nFill the parents’ bowl. The kitchen was warm. Fill the parents’ bowl. She said ‘hello.',
                  ],
                ],
                leftOut: [
                  'slice 1: the consistency replacement of group 1 OCCURRENCE 3 was left out because the apostrophe it carries in '
                    + 'would read as a quotation mark there, since another quotation mark of the page could pair with it; the place '
                    + 'keeps its words',
                ],
              },
              {
                outcomes: ['chosen', 'replaced', 'replaced',],
                changed: [
                  [0, '## One\n\nFill the parents’ bowl. The kitchen was warm. Fill the parents’ bowl. She said hello.',],
                  [1, '## Two\n\nThen the parents’ naps.',],
                ],
                leftOut: [],
              },
            ],);
            expect(readBack({ consistency: opened.consistency, },),).toEqual(opened.consistency,);
            expect(readBack({ consistency: plain.consistency, },),).toEqual(plain.consistency,);
          },
        },),
        it({
          name: 'KEEPS THE CLOSING QUOTATION MARK OF A QUOTATION THAT OPENED IN AN EARLIER SLICE, across a heading: the place is '
            + 'not taken for a plural possessive, its mark stays, and nothing of the page changes; the same page with no opener '
            + 'reads the mark as an apostrophe of the replaced words',
          fn: async () => {
            const quotes = ['the dogs', 'the kittens',];
            const opened = await apostropheRun({
              first: '‘Fill the dogs bowl.',
              second: 'Then feed the kittens’.',
              quotes,
              choice: 1,
            },);
            const plain = await apostropheRun({
              first: 'Fill the dogs bowl.',
              second: 'Then feed the kittens’.',
              quotes,
              choice: 1,
            },);
            expect([seenIn({ consistency: opened.consistency, },), seenIn({ consistency: plain.consistency, },),],).toEqual([
              { outcomes: ['chosen', 'kept-current',], changed: [], leftOut: [], },
              { outcomes: ['chosen', 'replaced',], changed: [[1, '## Two\n\nThen feed the dogs.',],], leftOut: [], },
            ],);
            expect(readBack({ consistency: opened.consistency, },),).toEqual(opened.consistency,);
            expect(readBack({ consistency: plain.consistency, },),).toEqual(plain.consistency,);
          },
        },),
        it({
          name: 'KEEPS THE CLOSING QUOTATION MARK OF A QUOTATION THAT OPENED IN AN EARLIER PARAGRAPH, the reviewer\'s B1: the '
            + 'place is not taken for a plural possessive, its mark stays, and nothing of the slice changes',
          fn: async () => {
            const { consistency, } = await apostropheRun({
              first: '‘Fill the bowls.\n\nThen feed the kittens’.',
              second: 'Then feed the dogs.',
              quotes: ['Then feed the kittens', 'Then feed the dogs',],
              choice: 2,
            },);
            expect(seenIn({ consistency, },),).toEqual({ outcomes: ['kept-current', 'chosen',], changed: [], leftOut: [], },);
            expect(readBack({ consistency, },),).toEqual(consistency,);
          },
        },),
        it({
          name: 'WRITES AN APOSTROPHE THAT ARRIVES IN THE FORM OF THE PLACE IT ARRIVES IN: a straight one onto a curly place '
            + 'stays curly, onto a place with no apostrophe it takes the form of the page\'s other apostrophes when they share '
            + 'one, and when they do not share one it still arrives curly, because the page that ships has its straight '
            + 'apostrophes curled by the archive typography',
          fn: async () => {
            const curly = await apostropheRun({
              first: 'Fill the kittens’ bowl. The kitchen was warm. Fill the pups’ bowl.',
              second: 'Then the dogs\' nap.',
              quotes: ['the kittens', 'the pups', 'the dogs',],
              choice: 3,
            },);
            const sharing = await apostropheRun({
              first: 'Fill the family bowl. The kitchen was warm. Fill the cat’s dish.',
              second: 'Then the dogs\' nap.',
              quotes: ['the family', 'the dogs',],
              choice: 2,
            },);
            const mixed = await apostropheRun({
              first: 'Fill the family bowl. The kitchen was warm. Fill the cat’s dish and the pup\'s bed.',
              second: 'Then the dogs\' nap.',
              quotes: ['the family', 'the dogs',],
              choice: 2,
            },);
            expect([curly, sharing, mixed,].map(function seen({ consistency, },): unknown {
              return seenIn({ consistency, },).changed;
            },),).toEqual([
              [[0, '## One\n\nFill the dogs’ bowl. The kitchen was warm. Fill the dogs’ bowl.',],],
              [[0, '## One\n\nFill the dogs’ bowl. The kitchen was warm. Fill the cat’s dish.',],],
              [[0, '## One\n\nFill the dogs’ bowl. The kitchen was warm. Fill the cat’s dish and the pup\'s bed.',],],
            ],);
            [curly, sharing, mixed,].forEach(function readsBack({ consistency, },): void {
              expect(readBack({ consistency, },),).toEqual(consistency,);
            },);
          },
        },),
        it({
          name: 'WRITES AN APOSTROPHE THAT ARRIVES IN THE FORM OF THE WHOLE PAGE, not of the receiving slice alone: the slice holds '
            + 'no apostrophe outside the words, the other slice writes its own curly, so a straight chosen one arrives curly, and '
            + 'it arrives curly also when the other slice writes both forms, because the page that ships has its straight '
            + 'apostrophes curled by the archive typography',
          fn: async () => {
            const curly = await apostropheRun({
              first: 'Fill the family bowl.',
              second: 'The cat’s dish waits. Then the dogs\' nap.',
              quotes: ['the family', 'the dogs',],
              choice: 2,
            },);
            const mixed = await apostropheRun({
              first: 'Fill the family bowl.',
              second: 'The cat’s dish and the pup\'s bed wait. Then the dogs\' nap.',
              quotes: ['the family', 'the dogs',],
              choice: 2,
            },);
            expect([curly, mixed,].map(function seen({ consistency, },): unknown {
              return seenIn({ consistency, },).changed;
            },),).toEqual([
              [[0, '## One\n\nFill the dogs’ bowl.',],],
              [[0, '## One\n\nFill the dogs’ bowl.',],],
            ],);
            [curly, mixed,].forEach(function readsBack({ consistency, },): void {
              expect(readBack({ consistency, },),).toEqual(consistency,);
            },);
          },
        },),
        it({
          name: 'KEEPS THE STRAIGHT FORM THE CHOSEN PLACE WROTE ON A PAGE WHOSE ARCHIVE KEEPS ITS STRAIGHT MARKS: the archive text '
            + 'holds no curly mark, so the typography of the shipped page curls nothing, and an apostrophe that arrives where the '
            + 'receiving words and the rest of the page hold none of their own is written as the chosen place wrote it; where the '
            + 'rest of the page holds only straight ones it arrives straight too',
          fn: async () => {
            const noOther = await apostropheRun({
              first: 'Fill the family bowl.',
              second: 'Then the dogs\' nap.',
              quotes: ['the family', 'the dogs',],
              choice: 2,
            },);
            const straightPage = await apostropheRun({
              first: 'Fill the family bowl. The kitchen was warm. Fill the cat\'s dish and the pup\'s bed.',
              second: 'Then the dogs\' nap.',
              quotes: ['the family', 'the dogs',],
              choice: 2,
            },);
            expect([noOther, straightPage,].map(function seen({ consistency, },): unknown {
              return seenIn({ consistency, },).changed;
            },),).toEqual([
              [[0, '## One\n\nFill the dogs\' bowl.',],],
              [[0, '## One\n\nFill the dogs\' bowl. The kitchen was warm. Fill the cat\'s dish and the pup\'s bed.',],],
            ],);
            [noOther, straightPage,].forEach(function readsBack({ consistency, },): void {
              expect(readBack({ consistency, },),).toEqual(consistency,);
            },);
          },
        },),
        it({
          name: 'LEAVES THE PARAGRAPH BESIDE A PLACE WHOSE SPLICE THE FLOORS ALWAYS REFUSE TO THE WRITERS, the reviewer\'s G3 '
            + 'residue: the chosen words hold no blank line and the receiving words hold one, with a plain space or a straight '
            + 'apostrophe after them, so the place has no splice and the next paragraph is offered to the writers',
          fn: async () => {
            const seen = await Promise.all([residueSeen({ after: ' said she', },), residueSeen({ after: '\' said she', },),],);
            expect(seen,).toEqual([
              {
                outcomes: ['kept-current', 'replaced', 'replaced', 'chosen',],
                kinds: [['current',], ['current', 'splice', 'writer',], ['current', 'splice', 'writer',],],
                notAsked: [
                  'group 1: consistency-writers-not-asked (OCCURRENCE 1): the words that render the repeated wording lie in no '
                    + 'single block of the English, so no paragraph can be rewritten',
                ],
              },
              {
                outcomes: ['kept-current', 'replaced', 'replaced', 'chosen',],
                kinds: [['current',], ['current', 'splice', 'writer',], ['current', 'splice', 'writer',],],
                notAsked: [
                  'group 1: consistency-writers-not-asked (OCCURRENCE 1): the words that render the repeated wording lie in no '
                    + 'single block of the English, so no paragraph can be rewritten',
                ],
              },
            ],);
          },
        },),
      ],
    },),
  ],
},);
