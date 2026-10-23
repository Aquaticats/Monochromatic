/**
 Tests for the page-level consistency pass: when it asks the bench, how the
 ballots settle a repeated sentence and a refrain, what it leaves as it stands
 and why, what the floors refuse, what a resumed page reads back, and that no
 trouble stops a run from shipping.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { createHash, } from 'node:crypto';
import { chmodSync, } from 'node:fs';
import { writeFile, } from 'node:fs/promises';

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  RUN_LATE_JUDGES,
  RUN_TRANSLATORS,
} from '@monochromatic-dev/translation-repair-run';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  buildConsistencyMessages,
  CONSISTENCY_CACHE_VERSION,
} from '@monochromatic-dev/translation-repair-consistency';
import {
  type ArtifactConsistency,
  NO_CONSISTENCY,
  parseConsistency,
} from '@monochromatic-dev/translation-repair-artifact';
import {
  consistencyKey,
  runPassConsistency,
} from '../../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import {
  alterStored,
  everySeatQuotes,
  fourthAlreadyFollows,
  outcomesByGroup,
  sentencePage,
  sentenceQuoteSpansTheRefrain,
  sharedSlicePage,
  slotWithTheName,
  storedGathers,
  withBlankedBlocks,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-edge.test-fixture.ts';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import {
  archiveEnglishOf,
  ballotText,
  choosing,
  firstPlaceWins,
  otherSheetBehaviour,
  preparedCatPage,
  projectedCatPage,
  refrainAnsweredInOrder,
  otherSheetsSplittingRunOff,
  refrainChoosing,
  scriptedJudgeClient,
  type SeatBehaviour,
  type SeatScript,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';
import {
  REFRAIN_QUOTES,
  SENTENCE_QUOTES,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-quotes.test-fixture.ts';
import {
  candidateTextsOf,
  currentParagraphOf,
  finalistsOf,
  isRefrainSheet,
  sheetBlockOf,
  sheetKindOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-sheet-reading.test-fixture.ts';
import {
  type CatRun,
  candidateKindsOf,
  DIGEST,
  runCatPass,
} from './consistency-run.test-fixture.ts';

/**
 Logger the pass writes to, whose lines are not under test.
 */
const l = tagged({ tag: 'pass-consistency-test', },);

/**
 What the pass left of the page's second slice: the refrain carried to the
 first slice's pattern with its own slot.
 */
const SLICE_ONE_AFTER = '## Two\n\nThe smell of fish in the kitchen woke the kitten. So this cat must be a lazy little treasure.';

/**
 What the pass left of the third slice: the sentence carried from the first
 slice and the refrain carried to its pattern.
 */
const SLICE_TWO_AFTER = '## Three\n\nIn the evening, the sunny window put the kitten to sleep, and the fish snacks smiled too. '
  + 'So this cat must be a sweet little treasure.';

/**
 What the pass left of the fourth slice.
 */
const SLICE_THREE_AFTER = '## Four\n\nThe rain in the yard sent the kitten running home. So this cat must be a warm little treasure.';

/**
 Makes a directory unwritable until the scope that holds the result ends.

 @param path - directory to close to writing

 @returns Its path, with the means of opening it again on disposal

 @example
 ```ts
 using closed = readOnlyDirectory({ path: dir.path, },);
 ```
 */
function readOnlyDirectory({ path, }: { readonly path: string; },): Disposable & { readonly path: string; } {
  chmodSync(
    path,
    0o500,
  );
  return {
    path,
    [Symbol.dispose]: function reopen(): void {
      chmodSync(
        path,
        0o700,
      );
    },
  };
}

/**
 The texts the pass changed, by slice.

 @param consistency - the section

 @returns Slice index and text of every changed slice

 @example
 ```ts
 const changed = changedTexts({ consistency, },);
 ```
 */
function changedTexts(
  { consistency, }: { readonly consistency: ArtifactConsistency; },
): readonly (readonly [number, string])[] {
  return consistency.slices
    .map(function pair(slice,): readonly [number, string] {
      return [slice.sliceIndex, slice.text,];
    },);
}

/**
 Outcomes of a group's places, in order.

 @param consistency - the section

 @param group - position of the group on the page

 @returns One outcome per place

 @example
 ```ts
 const outcomes = outcomesOf({ consistency, group: 0, },);
 ```
 */
function outcomesOf(
  {
    consistency,
    group,
  }: {
    readonly consistency: ArtifactConsistency;
    readonly group: number;
  },
): readonly string[] {
  return nonNullishOrThrow(consistency.groups[group],)
    .occurrences
    .map(function outcomeOf(occurrence,): string {
      return occurrence.outcome;
    },);
}

//region Live group two
// The live case where a repeated sentence's first place carries an "also" that renders a word standing before the
// repeat: the source in this region is the live Chinese, the English the live English trimmed to the paragraph around it.

/**
 The words of the first place's English that render the repeated sentence, with the "also" the live ballots quoted.
 */
const GROUP_TWO_WANTED = 'She also wanted to stay by more people\'s side looking cute and bring sunshine to more people';

/**
 The words of the second place's English that render the repeated sentence.
 */
const GROUP_TWO_ACCOMPANY = 'She wanted to accompany more people with her cuteness and bring sunshine to their lives';

/**
 The live page of group two: the sentence ends the first section after "也是" and stands alone in the second.

 @returns Preparation of the page

 @example
 ```ts
 const prepared = groupTwoPage();
 ```
 */
function groupTwoPage(): ReturnType<typeof prepareDocumentPair> {
  return prepareDocumentPair({
    sourceText: '## 一\n\n她不是刻板印象当中懦弱的小孩，而是实实在在可以让大家依靠的「前辈」，更是值得信赖的「老师」，'
      + '也是她想以可爱的样子陪伴在更多人的身边，为更多人带来阳光。\n\n## 二\n\n她想以可爱的样子陪伴在更多人的身边，为更多人带来阳光。\n',
    targetText: '## One\n\nShe was not the stereotypical timid child;\nshe was a “senior” everyone could truly rely on,\nand,\n'
      + `even more,\na “teacher” worthy of trust.\n${GROUP_TWO_WANTED}.\n\n## Two\n\n${GROUP_TWO_ACCOMPANY}.\n`,
  },);
}

/**
 The live ballots of group two: four judges answer, two quoting with the closing full stop and two without, three
 choosing the first place and one the second; the other seats throw.

 @param seat - the seat asked

 @returns Behaviour of the seat

 @example
 ```ts
 const behave: SeatScript = groupTwoBallots;
 ```
 */
function groupTwoBallots({ seat, }: { readonly seat: RosterModelId; },): SeatBehaviour {
  /**
   Position of the seat among the late judges.
   */
  const position = RUN_LATE_JUDGES.indexOf(seat,);
  if (position > 3)
    return { kind: 'throws', };
  /**
   The closing mark two of the four live judges kept.
   */
  const stop = (position >= 2) ? '.' : '';
  return {
    kind: 'answers',
    text: ballotText({
      quotes: [
        { label: 'OCCURRENCE 1', quote: `${GROUP_TWO_WANTED}${stop}`, },
        { label: 'OCCURRENCE 2', quote: `${GROUP_TWO_ACCOMPANY}${stop}`, },
      ],
      choice: (position === 3) ? 'OCCURRENCE 2' : 'OCCURRENCE 1',
      keep: [],
    },),
  };
}

/**
 The writers and the bench of group two as scripted: the first two writers carry the chosen words without the "also",
 the third carries them as they stand, the rest return the paragraph unchanged; the judges choose the candidate that
 carries the chosen words without the "also".

 @param seat - the seat asked

 @param sheet - the sheet it was shown

 @returns Behaviour of the seat

 @example
 ```ts
 const behave: SeatScript = groupTwoWritersAndBench;
 ```
 */
function groupTwoWritersAndBench(
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
    /**
     Position of the seat among the translate stage's writers.
     */
    const position = RUN_TRANSLATORS.indexOf(seat,);
    // Only the first four writers answer, so the gather hears exactly them whatever the order the replies arrive in.
    if (position > 3)
      return { kind: 'throws', };
    /**
     The paragraph as it stands.
     */
    const current = currentParagraphOf({ sheet, },);
    /**
     The words the chosen rendering replaces here.
     */
    const words = sheetBlockOf({ sheet, label: 'THE WORDS TO REPLACE', },);
    /**
     The words the chosen place's English holds.
     */
    const chosen = sheetBlockOf({ sheet, label: 'CHOSEN RENDERING', },);
    return {
      kind: 'answers',
      text: JSON.stringify({
        translation: (position < 2)
          ? current.replace(words, chosen.replace(' also', '',),)
          : (position === 2)
            ? current.replace(words, chosen,)
            : current,
      },),
    };
  }
  if (kind === 'choice') {
    // Only the first four judges answer, so the gather hears exactly them.
    if (RUN_LATE_JUDGES.indexOf(seat,) > 3)
      return { kind: 'throws', };
    return choosing({
      choice: candidateTextsOf({ sheet, },)
        .find(function carriesTheWordsWithoutAlso({ current, text, },): boolean {
          return (!current) && text.includes('stay by',) && (!text.includes('also',));
        },)
        ?.label
        ?? '',
    },);
  }
  return otherSheetBehaviour({ sheet, },);
}

//endregion Live group two

//region Two places in one block
// The reviewer's case: one English paragraph holds two of the three places of a repeated sentence, so the pass carries to
// both in one rewrite of the paragraph.

/**
 The three English renderings of the repeated sentence, the first two in one paragraph of the first section.
 */
const TWO_IN_BLOCK_WORDS = [
  'The sunny window put the kitten to sleep',
  'The sun on the sill sent the kitten off to sleep',
  'The window sun made the kitten fall asleep',
] as const;

/**
 The page: the sentence stands twice in one paragraph of the first section and once in the second.

 @returns Preparation of the page

 @example
 ```ts
 const prepared = twoInBlockPage();
 ```
 */
function twoInBlockPage(): ReturnType<typeof prepareDocumentPair> {
  return prepareDocumentPair({
    sourceText: '## 一\n\n窗台上的阳光让小猫睡着了。厨房里很暖和。窗台上的阳光让小猫睡着了。\n\n## 二\n\n窗台上的阳光让小猫睡着了。\n',
    targetText: `## One\n\n${TWO_IN_BLOCK_WORDS[0]}. The kitchen was warm. ${TWO_IN_BLOCK_WORDS[1]}.\n\n## Two\n\n${TWO_IN_BLOCK_WORDS[2]}.\n`,
  },);
}

/**
 The ballots of the page: every judge quotes all three places and chooses the third.

 @param seat - the seat asked

 @returns Behaviour of the seat

 @example
 ```ts
 const behave: SeatScript = twoInBlockBallots;
 ```
 */
function twoInBlockBallots({ seat, }: { readonly seat: RosterModelId; },): SeatBehaviour {
  if (RUN_LATE_JUDGES.indexOf(seat,) > 3)
    return { kind: 'throws', };
  return {
    kind: 'answers',
    text: ballotText({
      quotes: TWO_IN_BLOCK_WORDS.map(function quoteOf(quote, at,): Record<string, string> {
        return { label: `OCCURRENCE ${String(at + 1,)}`, quote, };
      },),
      choice: 'OCCURRENCE 3',
      keep: [],
    },),
  };
}

/**
 The writers and the bench of the page: the first two writers put the chosen words in both places of the paragraph
 and also change the sentence between them, which the splice leaves alone; the judges choose a writer's paragraph.

 @param seat - the seat asked

 @param sheet - the sheet it was shown

 @returns Behaviour of the seat

 @example
 ```ts
 const behave: SeatScript = twoInBlockWritersAndBench;
 ```
 */
function twoInBlockWritersAndBench(
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
        translation: currentParagraphOf({ sheet, },)
          .replace(sheetBlockOf({ sheet, label: 'THE WORDS TO REPLACE 1', },), sheetBlockOf({ sheet, label: 'CHOSEN RENDERING', },),)
          .replace(sheetBlockOf({ sheet, label: 'THE WORDS TO REPLACE 2', },), sheetBlockOf({ sheet, label: 'CHOSEN RENDERING', },),)
          .replace('was warm', 'felt warm',),
      },),
    };
  }
  if (kind === 'choice') {
    if (RUN_LATE_JUDGES.indexOf(seat,) > 3)
      return { kind: 'throws', };
    return choosing({
      choice: candidateTextsOf({ sheet, },)
        .find(function carriesTheChangeBetween({ text, },): boolean {
          return text.includes('felt warm',);
        },)
        ?.label
        ?? '',
    },);
  }
  return otherSheetBehaviour({ sheet, },);
}

//endregion Two places in one block

//region A place across a paragraph boundary beside a place of the second paragraph
// The words of the first place run over the boundary between two paragraphs, so no block holds them and only a splice
// could carry them, and the chosen words hold no break, so the splice is unavailable (`consistency-block-breaks.ts`);
// the second place stands in the second paragraph, which a writer may then rewrite.

/**
 The three English renderings of the repeated sentence: the first runs over a paragraph boundary, the second ends the
 second paragraph.
 */
const CROSSING_WORDS = [
  'The sunny window put the kitten\n\nto sleep',
  'The sun on the sill sent the kitten off to sleep',
  'The window sun made the kitten fall asleep',
] as const;

/**
 The page: the sentence runs over a paragraph boundary and ends the next paragraph of the first section, and stands once
 in the second.

 @param afterFirst - what directly follows the words that run over the boundary, nothing by default; the opening curly
 single mark there is never a proven apostrophe, so it is a mark beside the words that makes the splice of that place
 unavailable, the chosen place holding none

 @returns Preparation of the page

 @example
 ```ts
 const prepared = crossingPage({ afterFirst: '‘', },);
 ```
 */
function crossingPage(
  { afterFirst = '', }: { readonly afterFirst?: string; } = {},
): ReturnType<typeof prepareDocumentPair> {
  return prepareDocumentPair({
    sourceText: '## 一\n\n窗台上的阳光让小猫睡着了。厨房里很暖和。窗台上的阳光让小猫睡着了。\n\n## 二\n\n窗台上的阳光让小猫睡着了。\n',
    targetText: `## One\n\n${CROSSING_WORDS[0]}${afterFirst}. The kitchen was warm. ${CROSSING_WORDS[1]}.\n\n## Two\n\n${CROSSING_WORDS[2]}.\n`,
  },);
}

/**
 The ballots of the page: every judge quotes all three places and chooses the third.

 @param seat - the seat asked

 @returns Behaviour of the seat

 @example
 ```ts
 const behave: SeatScript = crossingBallots;
 ```
 */
function crossingBallots({ seat, }: { readonly seat: RosterModelId; },): SeatBehaviour {
  if (RUN_LATE_JUDGES.indexOf(seat,) > 3)
    return { kind: 'throws', };
  return {
    kind: 'answers',
    text: ballotText({
      quotes: CROSSING_WORDS.map(function quoteOf(quote, at,): Record<string, string> {
        return { label: `OCCURRENCE ${String(at + 1,)}`, quote, };
      },),
      choice: 'OCCURRENCE 3',
      keep: [],
    },),
  };
}

/**
 The writers and the bench of the page: a writer that is asked puts the chosen words in the second paragraph and changes
 the warm sentence of it; the judges choose a writer's paragraph where one is on offer and the code splice otherwise.

 @param seat - the seat asked

 @param sheet - the sheet it was shown

 @returns Behaviour of the seat

 @example
 ```ts
 const behave: SeatScript = crossingWritersAndBench;
 ```
 */
function crossingWritersAndBench(
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
    /**
     The words to replace when the sheet numbers them, which it does for several receiving places.
     */
    const numbered = sheetBlockOf({ sheet, label: 'THE WORDS TO REPLACE 1', },);
    return {
      kind: 'answers',
      text: JSON.stringify({
        translation: currentParagraphOf({ sheet, },)
          .replace(
          (numbered === '') ? sheetBlockOf({ sheet, label: 'THE WORDS TO REPLACE', },) : numbered,
          sheetBlockOf({ sheet, label: 'CHOSEN RENDERING', },),
          )
          .replace('was warm', 'felt warm',),
      },),
    };
  }
  if (kind === 'choice') {
    if (RUN_LATE_JUDGES.indexOf(seat,) > 3)
      return { kind: 'throws', };
    /**
     The candidates the sheet offers.
     */
    const candidates = candidateTextsOf({ sheet, },);
    return choosing({
      choice: (candidates.find(function carriesTheChangeBefore({ text, },): boolean {
        return text.includes('felt warm',);
      },) ?? candidates.find(function isAlternative({ current, },): boolean {
        return !current;
      },))
        ?.label
        ?? '',
    },);
  }
  return otherSheetBehaviour({ sheet, },);
}

//endregion A place across a paragraph boundary beside a place of the second paragraph

//region Two places with the same words and different marks beside them
// The words `the cats` stand in two places, once with the opening curly single mark directly after them (never a proven
// apostrophe, so it stays a mark beside the words) and once with none, and the third place is worded otherwise: the places
// that receive the chosen words hold equal words and different marks.

/**
 The English of the page's first section: the chosen place, and the place whose words a single mark follows.
 */
const MARKED_FIRST_SECTION = 'The sunny window put the kitten to sleep. The kitchen was warm. Fill the cats‘ bowl.';

/**
 The English of the page's second section: the place whose words nothing follows.
 */
const MARKED_SECOND_SECTION = 'Then the cats nap.';

/**
 What every judge quotes for the three places of the page, in order.
 */
const MARKED_QUOTES = ['The sunny window put the kitten to sleep', 'the cats', 'the cats',] as const;

/**
 The page: the sentence stands twice in the first section and once in the second.

 @param first - English of the first section

 @param second - English of the second section

 @returns Preparation of the page

 @example
 ```ts
 const prepared = markedPage({ first: MARKED_FIRST_SECTION, second: MARKED_SECOND_SECTION, },);
 ```
 */
function markedPage(
  {
    first,
    second,
  }: {
    readonly first: string;
    readonly second: string;
  },
): ReturnType<typeof prepareDocumentPair> {
  return prepareDocumentPair({
    sourceText: '## 一\n\n窗台上的阳光让小猫睡着了。厨房里很暖和。窗台上的阳光让小猫睡着了。\n\n## 二\n\n窗台上的阳光让小猫睡着了。\n',
    targetText: `## One\n\n${first}\n\n## Two\n\n${second}\n`,
  },);
}

/**
 The ballots of the page: every judge quotes the three places and chooses one.

 @param choice - label every judge chooses

 @param quotes - what every judge quotes for the three places, in order

 @returns What each seat does when asked

 @example
 ```ts
 const behave = markedBallots({ choice: 'OCCURRENCE 3', quotes: MARKED_QUOTES, },);
 ```
 */
function markedBallots(
  {
    choice,
    quotes,
  }: {
    readonly choice: string;
    readonly quotes: readonly string[];
  },
): SeatScript {
  return function ballotOf({ seat, },): SeatBehaviour {
    if (RUN_LATE_JUDGES.indexOf(seat,) > 3)
      return { kind: 'throws', };
    return {
      kind: 'answers',
      text: ballotText({
        quotes: quotes.map(function quoteOf(quote, at,): Record<string, string> {
          return { label: `OCCURRENCE ${String(at + 1,)}`, quote, };
        },),
        choice,
        keep: [],
      },),
    };
  };
}

/**
 Runs the pass over the page with every judge choosing one place.

 @param choice - label every judge chooses

 @param first - English of the first section, the page's own by default

 @param second - English of the second section, the page's own by default

 @param quotes - what every judge quotes for the three places, in order, the page's own by default

 @returns The run

 @example
 ```ts
 const { consistency, } = await markedRun({ choice: 'OCCURRENCE 3', },);
 ```
 */
async function markedRun(
  {
    choice,
    first = MARKED_FIRST_SECTION,
    second = MARKED_SECOND_SECTION,
    quotes = MARKED_QUOTES,
  }: {
    readonly choice: string;
    readonly first?: string;
    readonly second?: string;
    readonly quotes?: readonly string[];
  },
): Promise<CatRun> {
  await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
  return await runCatPass({
    behave: markedBallots({ choice, quotes, },),
    entryCacheDir: dir.path,
    prepared: markedPage({ first, second, },),
  },);
}

/**
 The single marks the section records beside each place's words.

 @param consistency - the section

 @returns The mark before and the mark after of every place of the first group, `none` where there is none

 @example
 ```ts
 const marks = marksRecordedBy({ consistency, },); // [['none', 'none',], ['none', '‘',],]
 ```
 */
function marksRecordedBy({ consistency, }: { readonly consistency: ArtifactConsistency; },): readonly (readonly string[])[] {
  return nonNullishOrThrow(consistency.groups[0],)
    .occurrences
    .map(function marksOf({ rendering, },): readonly string[] {
      return (rendering.kind === 'located') ? [rendering.markBefore ?? 'none', rendering.markAfter ?? 'none',] : [rendering.kind,];
    },);
}

//endregion Two places with the same words and different marks beside them

//region Partly carried units
// A block whose receiving places are not all ones code can move the chosen words into: the opening curly single mark
// directly after the words of one place (never a proven apostrophe), which the chosen place does not hold, makes the
// splice unavailable there, and the places beside it in the same block are still carried by the splice.

/**
 The repeated sentence of the source, with the sentence about the kitchen that follows it.
 */
const PARTLY_SENTENCE = '窗台上的阳光让小猫睡着了。厨房里很暖和。';

/**
 The source of a page: the repeated sentence stands as often in the first section as the judges quote places in it, and
 once in the second section.

 @param places - how many places the first section holds

 @returns The Chinese source

 @example
 ```ts
 const sourceText = partlySourceOf({ places: 3, },);
 ```
 */
function partlySourceOf({ places, }: { readonly places: number; },): string {
  return `## 一\n\n${PARTLY_SENTENCE.repeat(places,)}\n\n## 二\n\n窗台上的阳光让小猫睡着了。\n`;
}

/**
 The words of the chosen place, which closes the second section.
 */
const PARTLY_CHOSEN = 'the kittens purr';

/**
 Runs the pass over a page of the first section's English with every judge choosing the last place.

 @param english - English of the first section

 @param quotes - what every judge quotes for the places of the first section, in order

 @param others - what the writers and the bench of candidates do, the page's own by default

 @returns The run, with the lines it logged

 @example
 ```ts
 const { run, lines, } = await partlyCarriedRun({ english: 'Fill the cats‘ bowl. Then the dogs nap.', quotes: ['the cats', 'the dogs',], },);
 ```
 */
async function partlyCarriedRun(
  {
    english,
    quotes,
    others = otherSheetBehaviour,
  }: {
    readonly english: string;
    readonly quotes: readonly string[];
    readonly others?: SeatScript;
  },
): Promise<{ readonly run: CatRun; readonly lines: readonly string[]; }> {
  await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
  /**
   The lines the pass logged.
   */
  const lines: string[] = [];
  /**
   The run.
   */
  const run = await runCatPass({
    behave: markedBallots({ choice: `OCCURRENCE ${String(quotes.length + 1,)}`, quotes: [...quotes, PARTLY_CHOSEN,], },),
    others,
    entryCacheDir: dir.path,
    prepared: prepareDocumentPair({
      sourceText: partlySourceOf({ places: quotes.length, },),
      targetText: `## One\n\n${english}\n\n## Two\n\n${PARTLY_CHOSEN}.\n`,
    },),
    lines,
  },);
  return { run, lines, };
}

/**
 What the pass recorded of a partly carried page, for the cases to compare.

 @param run - the run

 @returns The outcomes of the first group, the first section's text, the kinds of candidates each place's carry holds and
 the findings that name a partly carried unit

 @example
 ```ts
 const seen = partlySeenIn({ run, },);
 ```
 */
function partlySeenIn({ run, }: { readonly run: CatRun; },): {
  readonly outcomes: readonly string[];
  readonly firstSection: string;
  readonly kinds: unknown;
  readonly partly: readonly string[];
} {
  return {
    outcomes: outcomesOf({ consistency: run.consistency, group: 0, },),
    firstSection: run.consistency.slices
      .find(function isFirst({ sliceIndex, },): boolean {
        return sliceIndex === 0;
      },)
      ?.text
      ?? '',
    kinds: nonNullishOrThrow(run.consistency.groups[0],).carries?.map(function kindsOf({ candidates, },): unknown {
      return candidateKindsOf({ candidates, },);
    },),
    partly: run.consistency.findings.filter(function isPartly(finding,): boolean {
      return finding.includes('consistency-partly-carried',);
    },),
  };
}

/**
 Whether a logged line says the entry stopped or that an invariant of the pass fired.

 @param line - a line the pass logged

 @returns Whether it names the stop or the invariant

 @example
 ```ts
 const stopped = stopsTheEntry('error [runPassConsistency] stopping the entry',);
 ```
 */
function stopsTheEntry(line: string,): boolean {
  return line.includes('stopping the entry',) || line.includes('ConsistencyInvariantError',);
}

/**
 Fails unless the section reads back as the one the pass wrote.

 @param consistency - the section

 @example
 ```ts
 expectReadsBack({ consistency, },);
 ```
 */
function expectReadsBack({ consistency, }: { readonly consistency: ArtifactConsistency; },): void {
  expect(parseConsistency({
    value: structuredClone(consistency,),
    path: 'cat.consistency',
    generation: 17,
    sliceIndexes: new Set([0, 1,],),
  },),).toEqual(consistency,);
}

/**
 The writers and the bench of the page of a crossing place beside a block with two receiving places: a writer puts the
 chosen words in both places of the block and changes the warm sentence, and the judges choose the paragraph with the change.

 @param seat - the seat asked

 @param sheet - the sheet it was shown

 @returns Behaviour of the seat

 @example
 ```ts
 const behave: SeatScript = partlyWriters;
 ```
 */
function partlyWriters(
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
        translation: currentParagraphOf({ sheet, },)
          .replace(sheetBlockOf({ sheet, label: 'THE WORDS TO REPLACE 1', },), sheetBlockOf({ sheet, label: 'CHOSEN RENDERING', },),)
          .replace(sheetBlockOf({ sheet, label: 'THE WORDS TO REPLACE 2', },), sheetBlockOf({ sheet, label: 'CHOSEN RENDERING', },),)
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
 The writers of a page whose first place runs over an edge of the block beside it: a writer puts the chosen words in both
 places of the block and, where it is told to, also changes words of the block that another place holds, and the choice
 among candidates takes the first that is not the paragraph as the page has it now (the splice).

 @param change - the words a writer changes and what it writes in their place, none for a writer that changes no more
 than the receiving places

 @returns What each seat does when asked

 @example
 ```ts
 const behave: SeatScript = crossingWriters({ change: ['to sleep', 'to doze',], },);
 ```
 */
function crossingWriters(
  { change, }: { readonly change?: readonly [string, string]; },
): SeatScript {
  return function behaviourOf(
    {
      seat,
      sheet,
    },
  ): SeatBehaviour {
    if (sheetKindOf({ sheet, },) !== 'writer')
      return otherSheetBehaviour({ sheet, },);
    if (RUN_TRANSLATORS.indexOf(seat,) > 1)
      return { kind: 'throws', };
    /**
     The paragraph with the chosen words in both receiving places.
     */
    const carried = currentParagraphOf({ sheet, },)
      .replace(sheetBlockOf({ sheet, label: 'THE WORDS TO REPLACE 1', },), sheetBlockOf({ sheet, label: 'CHOSEN RENDERING', },),)
      .replace(sheetBlockOf({ sheet, label: 'THE WORDS TO REPLACE 2', },), sheetBlockOf({ sheet, label: 'CHOSEN RENDERING', },),);
    return {
      kind: 'answers',
      text: JSON.stringify({ translation: (change === undefined) ? carried : carried.replace(change[0], change[1],), },),
    };
  };
}

/**
 What the pass recorded of a page whose first place runs over an edge of a block and cannot be spliced, for the writers
 the page asks.

 @param english - English of the first section

 @param quotes - what every judge quotes for the places of the first section, in order

 @param change - the words a writer changes besides the receiving places' own, none for a writer that changes no more

 @returns The outcomes of the first group, the kinds of candidates each place's carry holds, and how many writers'
 paragraphs were withheld for the words of another place

 @example
 ```ts
 const seen = await crossingSeen({ english, quotes, change: ['to sleep', 'to doze',], });
 ```
 */
async function crossingSeen(
  {
    english,
    quotes,
    change,
  }: {
    readonly english: string;
    readonly quotes: readonly string[];
    readonly change?: readonly [string, string];
  },
): Promise<{
  readonly outcomes: readonly string[];
  readonly kinds: unknown;
  readonly withheld: number;
}> {
  const { run, } = await partlyCarriedRun({
    english,
    quotes,
    others: crossingWriters(((change === undefined) ? {} : { change, }),),
  },);
  expectReadsBack({ consistency: run.consistency, },);
  const { outcomes, kinds, } = partlySeenIn({ run, },);
  return {
    outcomes,
    kinds,
    withheld: run.consistency.findings.filter(function isWithheld(finding,): boolean {
      return finding.includes('consistency-candidate-refused',) && finding.includes('must stay as they are',);
    },).length,
  };
}

//endregion Partly carried units

//region Live group one
// The live refrain whose four places tied two and two: the source in this region is the live Chinese, the English the live
// English trimmed to the paragraph that holds the refrain.

/**
 The live page of group one: four sections, each opening on the refrain.

 @returns Preparation of the page

 @example
 ```ts
 const prepared = groupOnePage();
 ```
 */
function groupOnePage(): ReturnType<typeof prepareDocumentPair> {
  return prepareDocumentPair({
    sourceText: [
      '## 一\n\n所以她是个热爱生活的女孩吧，灯火下，她在最朴素的生活里怀揣着最遥远的梦想。\n',
      '## 二\n\n所以她是个毅力超群的女孩吧。你看星夜中，她在最虚弱的身体上承担着最坚实的压力。\n',
      '## 三\n\n所以她是个值得信赖的女孩吧。你看山海外，她在最复杂的情况里给出了最明确的方向。\n',
      '## 四\n\n所以她是个温柔细腻的女孩吧。你看头壳里，她在最狭小的空间中撑起了最完美的世界。\n',
    ].join('\n',),
    targetText: [
      '## One\n\nSo she must have been a girl who loved life.\nUnder the lights,\nin the plainest of lives,\n'
        + 'she held the most distant of dreams close to her heart.\n',
      '## Two\n\nSo she was a girl of extraordinary perseverance,\nI suppose.\nYou see,\namid the starry night,\n'
        + 'in the frailest of bodies,\nshe bore the most unyielding pressure.\n',
      '## Three\n\nSo she was a girl who could be trusted,\nwasn\'t she?\nYou see,\nfar across the mountains and seas,\n'
        + 'she gave the clearest directions in the most complicated of situations.\n',
      '## Four\n\nSo she was a gentle,\ntender-hearted girl,\nwasn\'t she?\nJust look:\ninside the headpiece,\n'
        + 'in the narrowest of spaces,\nshe held up the most perfect world.\n',
    ].join('\n',),
  },);
}

/**
 The quotes and slots of the four live judges of group one, written as they wrote them: a line break or a space, a
 straight or a curly apostrophe, a closing mark or none, and a slot with or without its "who" or "of".

 @type {readonly (readonly (readonly [string, string])[])[]}
 */
const GROUP_ONE_QUOTES: readonly (readonly (readonly [string, string,])[])[] = [
  [
    ['So she must have been a girl who loved life', 'loved life',],
    ['So she was a girl of extraordinary perseverance,\nI suppose', 'extraordinary perseverance',],
    ['So she was a girl who could be trusted,\nwasn\'t she', 'could be trusted',],
    ['So she was a gentle,\ntender-hearted girl,\nwasn\'t she', 'gentle,\ntender-hearted',],
  ],
  [
    ['So she must have been a girl who loved life', 'who loved life',],
    ['So she was a girl of extraordinary perseverance,\nI suppose', 'of extraordinary perseverance',],
    ['So she was a girl who could be trusted,\nwasn\'t she', 'who could be trusted',],
    ['So she was a gentle,\ntender-hearted girl,\nwasn\'t she', 'gentle,\ntender-hearted',],
  ],
  [
    ['So she must have been a girl who loved life.', 'loved life',],
    ['So she was a girl of extraordinary perseverance, I suppose.', 'of extraordinary perseverance',],
    ['So she was a girl who could be trusted, wasn\'t she?', 'who could be trusted',],
    ['So she was a gentle, tender-hearted girl, wasn\'t she?', 'gentle, tender-hearted',],
  ],
  [
    ['So she must have been a girl who loved life', 'loved life',],
    ['So she was a girl of extraordinary perseverance,\nI suppose', 'extraordinary perseverance',],
    ['So she was a girl who could be trusted,\nwasn’t she', 'could be trusted',],
    ['So she was a gentle,\ntender-hearted girl,\nwasn’t she', 'gentle, tender-hearted',],
  ],
];

/**
 The live ballots of group one: four judges answer, two choosing the third place and two the fourth, so the choices
 tie for first; the other seats throw.

 @param seat - the seat asked

 @returns Behaviour of the seat

 @example
 ```ts
 const behave: SeatScript = groupOneBallots;
 ```
 */
function groupOneBallots({ seat, }: { readonly seat: RosterModelId; },): SeatBehaviour {
  /**
   Position of the seat among the late judges.
   */
  const position = RUN_LATE_JUDGES.indexOf(seat,);
  if (position > 3)
    return { kind: 'throws', };
  return {
    kind: 'answers',
    text: JSON.stringify({
      assessment: 'The renderings differ in wording and force, not core meaning.',
      candidateQuotes: (GROUP_ONE_QUOTES[position] ?? []).map(function quoteOf([quote, slot,], at,) {
        return { label: `OCCURRENCE ${String(at + 1,)}`, quote, slot, };
      },),
      choice: (position < 2) ? 'OCCURRENCE 3' : 'OCCURRENCE 4',
      keep: [],
    },),
  };
}

/**
 The rewrites the writers of group one make: each receiving place's paragraph with its refrain put in the chosen
 frame ("So she was a girl who ..., wasn't she?"), the place's own slot kept.
 */
const GROUP_ONE_REWRITES: readonly (readonly [string, string, string])[] = [
  [
    'So she must have been a girl who loved life',
    'So she must have been a girl who loved life.',
    'So she was a girl who loved life, wasn\'t she?',
  ],
  [
    'So she was a girl of extraordinary perseverance,\nI suppose',
    'So she was a girl of extraordinary perseverance,\nI suppose.',
    'So she was a girl of extraordinary perseverance,\nwasn\'t she?',
  ],
  [
    'So she was a gentle,\ntender-hearted girl,\nwasn\'t she',
    'So she was a gentle,\ntender-hearted girl,\nwasn\'t she?',
    'So she was a girl who was gentle and tender-hearted, wasn\'t she?',
  ],
];

/**
 The run-off, the writers and the bench of group one as scripted: three of the first four late judges choose the third
 place in the run-off and one the fourth, every writer puts the receiving place's own slot in the chosen frame, and the
 judges choose the candidate that is not the paragraph as it stands.

 @param seat - the seat asked

 @param sheet - the sheet it was shown

 @returns Behaviour of the seat

 @example
 ```ts
 const behave: SeatScript = groupOneRunOffAndWriters;
 ```
 */
function groupOneRunOffAndWriters(
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
  if (kind === 'run-off') {
    /**
     Position of the seat among the late judges.
     */
    const position = RUN_LATE_JUDGES.indexOf(seat,);
    return (position > 3) ? { kind: 'throws', } : choosing({ choice: (position < 3) ? 'OCCURRENCE 3' : 'OCCURRENCE 4', },);
  }
  if (kind === 'writer') {
    /**
     The words the chosen rendering replaces here.
     */
    const words = sheetBlockOf({ sheet, label: 'THE WORDS TO REPLACE', },);
    /**
     The rewrite for the place whose words these are.
     */
    const rewrite = GROUP_ONE_REWRITES.find(function isThePlace([from,],): boolean {
      return from === words;
    },);
    return {
      kind: 'answers',
      text: JSON.stringify({
        translation: (rewrite === undefined)
          ? currentParagraphOf({ sheet, },)
          : currentParagraphOf({ sheet, },).replace(rewrite[1], rewrite[2],),
      },),
    };
  }
  return otherSheetBehaviour({ sheet, },);
}

//endregion Live group one

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runPassConsistency.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FINDS THE TWO GROUPS OF THE CAT PAGE, a repeated sentence and a refrain of four lines, in page '
            + 'order, so the cases here run over the shape the pass was built for',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, },);
            expect(run.consistency.groups.map(function repeatOf(group,): string {
              return group.repeat.kind;
            },),).toEqual(['maximal', 'refrain',],);
            expect(run.consistency.groups.map(function placedOf(group,): number {
              return group.occurrences.length;
            },),).toEqual([2, 4,],);
          },
        },),
        it({
          name: 'ASKS NOBODY AND CHANGES NOTHING for a page that repeats no wording',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const prepared = prepareDocumentPair({
              sourceText: '## 一\n\n小猫在窗台上打盹。\n\n## 二\n\n小鱼在水里游来游去。\n',
              targetText: '## One\n\nThe kitten naps on the sill.\n\n## Two\n\nThe fish swims about in the water.\n',
            },);
            const run = await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, prepared, },);
            expect(run.consistency,).toEqual(NO_CONSISTENCY,);
            expect(run.asked,).toEqual([],);
          },
        },),
        it({
          name: 'CARRIES A REPEATED SENTENCE AND A REFRAIN TO ONE RENDERING EACH where the archive stands in every '
            + 'slice, the live case\'s shape: four lines one way, two sentences one way',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, },);
            expect(changedTexts({ consistency: run.consistency, },),).toEqual([
              [1, SLICE_ONE_AFTER,],
              [2, SLICE_TWO_AFTER,],
              [3, SLICE_THREE_AFTER,],
            ],);
            expect([
              outcomesOf({ consistency: run.consistency, group: 0, },),
              outcomesOf({ consistency: run.consistency, group: 1, },),
            ],).toEqual([
              ['chosen', 'replaced',],
              ['chosen', 'replaced', 'replaced', 'replaced',],
            ],);
          },
        },),
        it({
          name: 'RECORDS A GROUP\'S WORDING, EVERY BALLOT, THE USABLE COUNT, THE QUORUM, THE WINNER AND EACH PLACE\'S '
            + 'OFFSETS AND LOCATED QUOTE, and the record reads back through the reader',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, },);
            const [sentence,] = run.consistency.groups;
            const group = nonNullishOrThrow(sentence,);
            expect(group.repeat,).toEqual({ kind: 'maximal', wording: '窗台上的阳光让小猫睡着了', },);
            expect(group.occurrences,).toEqual([
              {
                label: 'OCCURRENCE 1',
                sliceIndex: 0,
                startOffset: 6,
                endOffset: 18,
                line: 3,
                rendering: { kind: 'located', quote: 'The sunny window put the kitten to sleep', start: 8, },
                outcome: 'chosen',
              },
              {
                label: 'OCCURRENCE 2',
                sliceIndex: 2,
                startOffset: 85,
                endOffset: 97,
                line: 11,
                rendering: { kind: 'located', quote: 'the sunlight on the sill lulled the kitten to sleep', start: 26, },
                outcome: 'replaced',
              },
            ],);
            expect(group.unplaced,).toEqual([],);
            expect(group.verdict.kind,).toBe('asked',);
            if (group.verdict.kind === 'asked') {
              expect(group.verdict.usable,).toBe(group.verdict.ballots.length,);
              expect(group.verdict.winner,).toEqual({ kind: 'chosen', label: 'OCCURRENCE 1', },);
              // The ballots are listed in the order of a bench the prompt rotates, which moved when the house tense rule changed
              // the prompt, so the ballot of the first seat asked is found by its seat and not by its position.
              expect(group.verdict.ballots.find(function isFirstAsked(ballot,): boolean {
                return ballot.seat === run.asked[0];
              },),).toEqual({
                seat: run.asked[0],
                reason: 'The first rendering is the most faithful to the original, and the others differ in wording only.',
                quotes: SENTENCE_QUOTES,
                choice: 'OCCURRENCE 1',
                keep: [],
              },);
            }
            expect(parseConsistency({
              value: structuredClone(run.consistency,),
              path: 'cat.consistency',
              generation: 16,
              sliceIndexes: new Set([0, 1, 2, 3,],),
            },),).toEqual(run.consistency,);
          },
        },),
        it({
          name: 'RECORDS A REFRAIN\'S FRAME AND EACH LINE\'S SLOT OFFSETS AND SLOT',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, },);
            const group = nonNullishOrThrow(run.consistency.groups[1],);
            expect(group.repeat,).toEqual({ kind: 'refrain', prefix: '所以这只猫一定是', suffix: '的小宝贝', },);
            expect(group.occurrences[1],).toEqual({
              label: 'OCCURRENCE 2',
              sliceIndex: 1,
              startOffset: 56,
              endOffset: 70,
              line: 7,
              slotStartOffset: 64,
              slotEndOffset: 66,
              rendering: { kind: 'located', quote: 'So this cat is surely a lazy little treasure', start: 58, slot: 'lazy', },
              outcome: 'replaced',
            },);
          },
        },),
        it({
          name: 'PUTS THE SAME CARRIED WORDING IN THE SLICE WHERE THE LANES SHIPPED WORDING OTHER THAN THE ARCHIVE\'S, '
            + 'since the pass reads what each slice would ship and not what the archive holds',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const prepared = preparedCatPage();
            const archive = archiveEnglishOf({ prepared, },);
            const run = await runCatPass({
              behave: firstPlaceWins,
              entryCacheDir: dir.path,
              prepared,
              english: [
                archive[0] as string,
                '## Two\n\nThe smell of fish woke the kitten in the kitchen. So this cat is surely a lazy little treasure.',
                archive[2] as string,
                archive[3] as string,
              ],
            },);
            expect(changedTexts({ consistency: run.consistency, },)[0],).toEqual([
              1,
              '## Two\n\nThe smell of fish woke the kitten in the kitchen. So this cat must be a lazy little treasure.',
            ],);
            expect(outcomesOf({ consistency: run.consistency, group: 1, },),).toEqual([
              'chosen',
              'replaced',
              'replaced',
              'replaced',
            ],);
          },
        },),
        it({
          name: 'ASKS NO JUDGE ABOUT A GROUP WHOSE PLACES ALREADY READ ALIKE, each alone in a slice with one '
            + 'English, and records it consistent',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const prepared = preparedCatPage();
            const english = [...archiveEnglishOf({ prepared, },),];
            english[2] = english[0] as string;
            const run = await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, prepared, english, },);
            const group = nonNullishOrThrow(run.consistency.groups[0],);
            expect(group.verdict,).toEqual({ kind: 'not-asked', reason: 'consistent', },);
            expect(outcomesOf({ consistency: run.consistency, group: 0, },),).toEqual(['consistent', 'consistent',],);
            expect(run.sheets.every(function asksAboutFrame(sheet,): boolean {
              return sheet.includes('FRAME BEGINS WITH',);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'ASKS THE JUDGES ABOUT A GROUP WHOSE TWO PLACES SHARE ONE SLICE even where the slice\'s English is one '
            + 'text, since the slice may render the wording twice',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const prepared = prepareDocumentPair({
              sourceText: '窗台上的阳光让小猫睡着了。\n\n傍晚的时候，窗台上的阳光让小猫睡着了，小鱼干也笑了。\n',
              targetText: 'The sunny window put the kitten to sleep.\n\nIn the evening, the sunlight on the sill '
                + 'lulled the kitten to sleep, and the fish snacks smiled too.\n',
            },);
            const run = await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, prepared, },);
            expect(nonNullishOrThrow(run.consistency.groups[0],).verdict.kind,).toBe('asked',);
            expect(changedTexts({ consistency: run.consistency, },),).toEqual([
              [
                0,
                'The sunny window put the kitten to sleep.\n\nIn the evening, the sunny window put the '
                  + 'kitten to sleep, and the fish snacks smiled too.',
              ],
            ],);
          },
        },),
        it({
          name: 'ASKS NOBODY WHEN EVERY GROUP ALREADY READS ALIKE, reading no seat and recording the group consistent',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const prepared = prepareDocumentPair({
              sourceText: '## 一\n\n窗台上的阳光让小猫睡着了。\n\n## 二\n\n窗台上的阳光让小猫睡着了。\n',
              targetText: '## One\n\nThe sunny window put the kitten to sleep.\n\n## Two\n\nThe sunny window put the kitten to sleep.\n',
            },);
            const alike = '## One\n\nThe sunny window put the kitten to sleep.';
            const run = await runCatPass({
              behave: firstPlaceWins,
              entryCacheDir: dir.path,
              prepared,
              english: [alike, alike,],
            },);
            expect(run.asked,).toEqual([],);
            expect(run.consistency.slices,).toEqual([],);
            expect(nonNullishOrThrow(run.consistency.groups[0],).verdict,).toEqual({ kind: 'not-asked', reason: 'consistent', },);
          },
        },),
        it({
          name: 'RECORDS A PLACE THAT ALREADY CARRIES THE CHOSEN PATTERN AS CONSISTENT, and replaces the others',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const prepared = preparedCatPage();
            const archive = archiveEnglishOf({ prepared, },);
            const run = await runCatPass({
              behave: fourthAlreadyFollows,
              entryCacheDir: dir.path,
              prepared,
              english: [
                archive[0] as string,
                archive[1] as string,
                archive[2] as string,
                '## Four\n\nThe rain in the yard sent the kitten running home. So this cat must be a warm little treasure.',
              ],
            },);
            expect(outcomesOf({ consistency: run.consistency, group: 1, },),).toEqual([
              'chosen',
              'replaced',
              'replaced',
              'consistent',
            ],);
            expect(changedTexts({ consistency: run.consistency, },).map(function indexOf([index,],): number {
              return index;
            },),).toEqual([1, 2,],);
          },
        },),
        it({
          name: 'RECORDS AND SKIPS A GROUP WITH FEWER THAN TWO PLACED OCCURRENCES, listing the occurrence no slice holds '
            + 'as unplaced and leaving it as it stands',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const whole = preparedCatPage();
            // A preparation numbers its slices by position (`assertSliceIndexing`), so the slices that stay are renumbered:
            // the words of the dropped one stay in the page as text no slice holds.
            const prepared = {
              ...whole,
              slices: whole.slices.filter(function notThird(slice,): boolean {
                return slice.source.sliceIndex !== 2;
              },)
                .map(function byPosition(
                  slice,
                  position,
                ) {
                  return {
                    ...slice,
                    source: { ...slice.source, sliceIndex: position, },
                    target: { ...slice.target, sliceIndex: position, },
                  };
                },),
            };
            const run = await runCatPass({
              behave: function threePlaces({ sheet, },): SeatBehaviour {
                return isRefrainSheet({ sheet, },)
                  ? {
                    kind: 'answers',
                    text: ballotText({
                      quotes: [
                        REFRAIN_QUOTES[0] as Record<string, string>,
                        REFRAIN_QUOTES[1] as Record<string, string>,
                        { ...REFRAIN_QUOTES[3], label: 'OCCURRENCE 3', } as Record<string, string>,
                      ],
                      choice: 'OCCURRENCE 1',
                      keep: [],
                    },),
                  }
                  : firstPlaceWins({ sheet, },);
              },
              entryCacheDir: dir.path,
              prepared,
            },);
            const sentence = nonNullishOrThrow(run.consistency.groups[0],);
            expect(sentence.verdict,).toEqual({ kind: 'not-asked', reason: 'too-few-placed', },);
            expect(sentence.unplaced,).toEqual([
              { startOffset: 85, endOffset: 97, line: 11, reason: 'in-no-slice', },
            ],);
            expect(outcomesOf({ consistency: run.consistency, group: 0, },),).toEqual(['untouched',],);
            const refrain = nonNullishOrThrow(run.consistency.groups[1],);
            expect(refrain.occurrences.length,).toBe(3,);
            expect(refrain.unplaced.length,).toBe(1,);
            expect(changedTexts({ consistency: run.consistency, },),).toEqual([
              [1, SLICE_ONE_AFTER,],
              [2, SLICE_THREE_AFTER,],
            ],);
          },
        },),
        it({
          name: 'ASKS NOBODY ABOUT A GROUP WITH ENGLISH IN FEWER THAN TWO PLACES, since there is nothing to carry between them',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const prepared = preparedCatPage();
            const english = [...archiveEnglishOf({ prepared, },),];
            english[2] = '';
            const run = await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, prepared, english, },);
            expect(nonNullishOrThrow(run.consistency.groups[0],).verdict,).toEqual({
              kind: 'not-asked',
              reason: 'no-english',
            },);
            expect(outcomesOf({ consistency: run.consistency, group: 0, },),).toEqual(['untouched', 'untouched',],);
          },
        },),
        it({
          name: 'KEEPS A PLACE WHOSE CONTEXT CALLS FOR DIFFERENT WORDING when a strict majority list it, and still '
            + 'carries the other group to it',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: function keepsTheSecond({ sheet, },): SeatBehaviour {
                return isRefrainSheet({ sheet, },)
                  ? firstPlaceWins({ sheet, },)
                  : {
                    kind: 'answers',
                    text: ballotText({ quotes: SENTENCE_QUOTES, choice: 'OCCURRENCE 1', keep: ['OCCURRENCE 2',], },),
                  };
              },
              entryCacheDir: dir.path,
            },);
            expect(outcomesOf({ consistency: run.consistency, group: 0, },),).toEqual(['chosen', 'kept-context',],);
            expect(changedTexts({ consistency: run.consistency, },)[1],).toEqual([
              2,
              '## Three\n\nIn the evening, the sunlight on the sill lulled the kitten to sleep, and the fish snacks '
                + 'smiled too. So this cat must be a sweet little treasure.',
            ],);
          },
        },),
        it({
          name: 'RUNS OFF A TIE FOR FIRST between two places, four usable ballots splitting two and two: the bench is asked '
            + 'once more with a sheet showing only the tied places, and the place it names is carried to the others',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const lines: string[] = [];
            const run = await runCatPass({
              behave: refrainAnsweredInOrder({
                answers: [
                  refrainChoosing({ choice: 'OCCURRENCE 1', },),
                  refrainChoosing({ choice: 'OCCURRENCE 1', },),
                  refrainChoosing({ choice: 'OCCURRENCE 2', },),
                  refrainChoosing({ choice: 'OCCURRENCE 2', },),
                ],
              },),
              entryCacheDir: dir.path,
              lines,
            },);
            const refrain = nonNullishOrThrow(run.consistency.groups[1],);
            const reason = 'the choices reached no strict majority, so the finalists are OCCURRENCE 1 (2 ballots) and '
              + 'OCCURRENCE 2 (2 ballots), of 4 usable in all';
            expect({
              first: refrain.verdict.kind === 'asked'
                ? [
                  refrain.verdict.usable,
                  refrain.verdict.winner,
                  refrain.verdict.runOff?.finalists,
                  (refrain.verdict.runOff?.usable ?? 0) >= (refrain.verdict.runOff?.quorum ?? 1),
                ]
                : [],
              logged: lines.filter(function asksAgain(line,): boolean {
                return line.includes('asking the bench once more',);
              },),
              recorded: run.consistency.findings.filter(function isRunOffReason(finding,): boolean {
                return finding.includes('run-off asked',);
              },),
            },).toEqual({
              first: [
                4,
                { kind: 'chosen', label: 'OCCURRENCE 1', },
                ['OCCURRENCE 1', 'OCCURRENCE 2',],
                true,
              ],
              logged: [`info [runPassConsistency] [decideAskedGroup] asking the bench once more: ${reason}`,],
              recorded: [`group 2: run-off asked: ${reason}`,],
            },);
            // ONE LINE FOR THE RUN-OFF'S FINAL COUNT: how many of the choices heard were usable, the quorum, and the winner.
            expect(lines.filter(function countsRunOff(line,): boolean {
              return line.includes('the run-off counted',);
            },),).toEqual([
              'info [runPassConsistency] [decideAskedGroup] the run-off counted '
                + `${String(refrain.verdict.kind === 'asked' ? refrain.verdict.runOff?.usable : -1,)} usable of `
                + `${String(refrain.verdict.kind === 'asked' ? refrain.verdict.runOff?.ballots.length : -1,)} heard `
                + `(quorum ${String(refrain.verdict.kind === 'asked' ? refrain.verdict.runOff?.quorum : -1,)}): OCCURRENCE 1 won`,
            ],);
            const runOffSheets = run.sheets.filter(function isRunOff(sheet,): boolean {
              return sheetKindOf({ sheet, },) === 'run-off';
            },);
            expect(runOffSheets.length,).toBe(5,);
            expect(runOffSheets.every(function showsOnlyTheTied(sheet,): boolean {
              return sheet.includes('OCCURRENCE 1 ORIGINAL',) && sheet.includes('OCCURRENCE 2 ORIGINAL',)
                && (!sheet.includes('OCCURRENCE 3 ORIGINAL',)) && (!sheet.includes('OCCURRENCE 4 ORIGINAL',));
            },),).toBe(true,);
            expect(outcomesOf({ consistency: run.consistency, group: 1, },),).toEqual([
              'chosen',
              'replaced',
              'replaced',
              'replaced',
            ],);
          },
        },),
        it({
          name: 'LEAVES A GROUP AS IT STANDS ON A SECOND TIE, the run-off splitting four and four, recording the run-off, and '
            + 'a resume asks nobody and lands on the same page',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: refrainAnsweredInOrder({
                answers: [
                  refrainChoosing({ choice: 'OCCURRENCE 1', },),
                  refrainChoosing({ choice: 'OCCURRENCE 1', },),
                  refrainChoosing({ choice: 'OCCURRENCE 2', },),
                  refrainChoosing({ choice: 'OCCURRENCE 2', },),
                ],
              },),
              others: otherSheetsSplittingRunOff,
              entryCacheDir: dir.path,
            },);
            const refrain = nonNullishOrThrow(run.consistency.groups[1],);
            expect(refrain.verdict.kind === 'asked'
              ? [refrain.verdict.usable, refrain.verdict.winner, refrain.verdict.runOff?.usable,]
              : [],).toEqual([
                4,
                { kind: 'none', reason: 'run-off-tied', },
                4,
              ],);
            expect(outcomesOf({ consistency: run.consistency, group: 1, },),).toEqual([
              'untouched',
              'untouched',
              'untouched',
              'untouched',
            ],);
            expect(changedTexts({ consistency: run.consistency, },).map(function indexOf([index,],): number {
              return index;
            },),).toEqual([2,],);
            // The gathers reached their quorum with one seat short, so they were stored with their findings and a resume reads them back.
            const again = await runCatPass({
              behave: function throwsAlways(): SeatBehaviour {
                return { kind: 'throws', };
              },
              others: function throwsAlways(): SeatBehaviour {
                return { kind: 'throws', };
              },
              entryCacheDir: dir.path,
            },);
            expect(again.asked,).toEqual([],);
            expect(again.consistency,).toEqual(run.consistency,);
          },
        },),
        it({
          name: 'RUNS OFF THE TWO PLACES THAT TIED FOR FIRST WHEN NO PLACE HAS A MAJORITY, five usable ballots splitting two, '
            + 'two and one, and leaves the group as it stands when the run-off ties again',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: refrainAnsweredInOrder({
                answers: [
                  refrainChoosing({ choice: 'OCCURRENCE 1', },),
                  refrainChoosing({ choice: 'OCCURRENCE 1', },),
                  refrainChoosing({ choice: 'OCCURRENCE 2', },),
                  refrainChoosing({ choice: 'OCCURRENCE 2', },),
                  refrainChoosing({ choice: 'OCCURRENCE 3', },),
                ],
              },),
              others: otherSheetsSplittingRunOff,
              entryCacheDir: dir.path,
            },);
            const refrain = nonNullishOrThrow(run.consistency.groups[1],);
            expect(refrain.verdict.kind === 'asked'
              ? [refrain.verdict.usable, refrain.verdict.winner, refrain.verdict.runOff?.finalists,]
              : [],).toEqual([
                5,
                { kind: 'none', reason: 'run-off-tied', },
                ['OCCURRENCE 1', 'OCCURRENCE 2',],
              ],);
          },
        },),
        it({
          name: 'RUNS OFF A LONE LEADER WITH THE PLACES NEXT BEHIND IT: four usable ballots splitting two, one and one have no '
            + 'strict majority, so the bench is asked once more on a sheet showing the '
            + 'three places that drew ballots, and the place it names, one that led no ballot, is carried to the others',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const lines: string[] = [];
            const run = await runCatPass({
              behave: refrainAnsweredInOrder({
                answers: [
                  refrainChoosing({ choice: 'OCCURRENCE 1', },),
                  refrainChoosing({ choice: 'OCCURRENCE 1', },),
                  refrainChoosing({ choice: 'OCCURRENCE 2', },),
                  refrainChoosing({ choice: 'OCCURRENCE 3', },),
                ],
              },),
              others: function runOffNamesTheLastFinalist(call,): SeatBehaviour {
                return (sheetKindOf({ sheet: call.sheet, },) === 'run-off')
                  ? choosing({ choice: finalistsOf({ sheet: call.sheet, },).at(-1,) ?? '', },)
                  : otherSheetBehaviour({ sheet: call.sheet, },);
              },
              entryCacheDir: dir.path,
              lines,
            },);
            const refrain = nonNullishOrThrow(run.consistency.groups[1],);
            const reason = 'the choices reached no strict majority, so the finalists are OCCURRENCE 1 (2 ballots), '
              + 'OCCURRENCE 2 (1 ballot) and OCCURRENCE 3 (1 ballot), of 4 usable in all';
            expect({
              first: refrain.verdict.kind === 'asked'
                ? [refrain.verdict.usable, refrain.verdict.winner, refrain.verdict.runOff?.finalists,]
                : [],
              recorded: run.consistency.findings.filter(function isRunOffReason(finding,): boolean {
                return finding.includes('run-off asked',);
              },),
              logged: lines.filter(function asksAgain(line,): boolean {
                return line.includes('asking the bench once more',);
              },),
            },).toEqual({
              first: [
                4,
                { kind: 'chosen', label: 'OCCURRENCE 3', },
                ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3',],
              ],
              recorded: [`group 2: run-off asked: ${reason}`,],
              logged: [`info [runPassConsistency] [decideAskedGroup] asking the bench once more: ${reason}`,],
            },);
            const runOffSheets = run.sheets.filter(function isRunOff(sheet,): boolean {
              return sheetKindOf({ sheet, },) === 'run-off';
            },);
            expect(runOffSheets.length > 0,).toBe(true,);
            expect(runOffSheets.every(function showsOnlyTheFinalists(sheet,): boolean {
              return sheet.includes('OCCURRENCE 1 ORIGINAL',) && sheet.includes('OCCURRENCE 2 ORIGINAL',)
                && sheet.includes('OCCURRENCE 3 ORIGINAL',) && (!sheet.includes('OCCURRENCE 4 ORIGINAL',));
            },),).toBe(true,);
            expect(outcomesOf({ consistency: run.consistency, group: 1, },),).toEqual([
              'replaced',
              'replaced',
              'chosen',
              'replaced',
            ],);
          },
        },),
        it({
          name: 'LEAVES A GROUP AS IT STANDS ON A SECOND SPLIT OF A LONE LEADER\'S RUN-OFF, four usable ballots splitting two, one '
            + 'and one, then the run-off dividing the three places again, recording the run-off',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: refrainAnsweredInOrder({
                answers: [
                  refrainChoosing({ choice: 'OCCURRENCE 1', },),
                  refrainChoosing({ choice: 'OCCURRENCE 1', },),
                  refrainChoosing({ choice: 'OCCURRENCE 2', },),
                  refrainChoosing({ choice: 'OCCURRENCE 3', },),
                ],
              },),
              others: otherSheetsSplittingRunOff,
              entryCacheDir: dir.path,
            },);
            const refrain = nonNullishOrThrow(run.consistency.groups[1],);
            expect(refrain.verdict.kind === 'asked'
              ? [refrain.verdict.usable, refrain.verdict.winner, refrain.verdict.runOff?.finalists, refrain.verdict.runOff?.usable,]
              : [],).toEqual([
                4,
                { kind: 'none', reason: 'run-off-tied', },
                ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3',],
                4,
              ],);
            expect(outcomesOf({ consistency: run.consistency, group: 1, },),).toEqual([
              'untouched',
              'untouched',
              'untouched',
              'untouched',
            ],);
          },
        },),
        it({
          name: 'RUNS OFF EVERY PLACE NEXT BEHIND A LONE LEADER: five usable ballots splitting two, one, one and one name all four '
            + 'places',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: refrainAnsweredInOrder({
                answers: [
                  refrainChoosing({ choice: 'OCCURRENCE 1', },),
                  refrainChoosing({ choice: 'OCCURRENCE 1', },),
                  refrainChoosing({ choice: 'OCCURRENCE 2', },),
                  refrainChoosing({ choice: 'OCCURRENCE 3', },),
                  refrainChoosing({ choice: 'OCCURRENCE 4', },),
                ],
              },),
              others: otherSheetsSplittingRunOff,
              entryCacheDir: dir.path,
            },);
            const refrain = nonNullishOrThrow(run.consistency.groups[1],);
            expect(refrain.verdict.kind === 'asked'
              ? [refrain.verdict.usable, refrain.verdict.runOff?.finalists,]
              : [],).toEqual([
                5,
                ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3', 'OCCURRENCE 4',],
              ],);
          },
        },),
        it({
          name: 'LEAVES A GROUP AS IT STANDS BELOW THE QUORUM THE GATHER CLOSED ON, and stores nothing for a resume',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const behave = refrainAnsweredInOrder({
              answers: [
                refrainChoosing({ choice: 'OCCURRENCE 1', },),
                refrainChoosing({ choice: 'OCCURRENCE 1', },),
                refrainChoosing({ choice: 'OCCURRENCE 1', },),
              ],
            },);
            const first = await runCatPass({ behave, entryCacheDir: dir.path, },);
            const refrain = nonNullishOrThrow(first.consistency.groups[1],);
            expect(refrain.verdict.kind === 'asked'
              ? [refrain.verdict.usable, refrain.verdict.quorum, refrain.verdict.winner,]
              : [],).toEqual([
                3,
                4,
                { kind: 'none', reason: 'no-quorum', },
              ],);
            const again = await runCatPass({
              behave: refrainAnsweredInOrder({ answers: [], },),
              entryCacheDir: dir.path,
            },);
            // The repeated sentence's gather reached its quorum and was stored; the refrain's did not, so it is asked again.
            // The sentence's writers and judges of the third section are asked again as well: that paragraph holds a
            // refrain place, whose words the sheets list to keep, and the refrain located none this time.
            expect(again.asked.length,).toBe(30,);
            expect(again.sheets.map(function questionOf(sheet,): readonly [string, boolean] {
              return [sheetKindOf({ sheet, },), sheet.includes('FRAME BEGINS WITH',),];
            },),).toEqual([
              ...Array.from({ length: 20, }, (): readonly [string, boolean] => ['ballot', true,],),
              ...Array.from({ length: 5, }, (): readonly [string, boolean] => ['writer', false,],),
              ...Array.from({ length: 5, }, (): readonly [string, boolean] => ['choice', false,],),
            ],);
          },
        },),
        it({
          name: 'SHIPS THE PAGE UNCHANGED WHEN EVERY JUDGE THROWS, recording the groups it asked about and no slice changed',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: function throwsAlways(): SeatBehaviour {
                return { kind: 'throws', };
              },
              entryCacheDir: dir.path,
            },);
            expect(run.consistency.slices,).toEqual([],);
            expect(run.consistency.groups.map(function verdictOf(group,): unknown {
              return group.verdict.kind === 'asked' ? [group.verdict.usable, group.verdict.winner,] : group.verdict.kind;
            },),).toEqual([
              [0, { kind: 'none', reason: 'no-quorum', },],
              [0, { kind: 'none', reason: 'no-quorum', },],
            ],);
            expect(outcomesByGroup({ consistency: run.consistency, },).flat().every(function untouched(outcome,): boolean {
              return outcome === 'untouched';
            },),).toBe(true,);
          },
        },),
        it({
          name: 'LEAVES A PLACE UNLOCATED WHEN ITS QUOTE IS MISSING FROM, EMPTY OR AMBIGUOUS IN ITS ENGLISH, the group\'s '
            + 'other places and the other group carried all the same',
          fn: async () => {
            /**
             Runs the pass with a quote for the second place of the sentence.
             */
            async function withQuote(
              { quote, english, }: { readonly quote: string; readonly english?: readonly string[]; },
            ): Promise<CatRun> {
              await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
              return await runCatPass({
                behave: function quotesTheSecond({ sheet, },): SeatBehaviour {
                  return isRefrainSheet({ sheet, },)
                    ? firstPlaceWins({ sheet, },)
                    : {
                      kind: 'answers',
                      text: ballotText({
                        quotes: [SENTENCE_QUOTES[0] as Record<string, string>, { label: 'OCCURRENCE 2', quote, },],
                        choice: 'OCCURRENCE 1',
                        keep: [],
                      },),
                    };
                },
                entryCacheDir: dir.path,
                ...((english === undefined) ? {} : { english, }),
              },);
            }
            const prepared = preparedCatPage();
            const archive = archiveEnglishOf({ prepared, },);
            const twice = [
              archive[0] as string,
              archive[1] as string,
              '## Three\n\nIn the evening, the sunlight lulled the kitten to sleep, and the kitten to sleep again. '
                + 'So this cat has to be a sweet little darling.',
              archive[3] as string,
            ];
            const [missing, empty, ambiguous,] = [
              await withQuote({ quote: 'the lamp on the sill lulled the kitten to sleep', },),
              await withQuote({ quote: '', },),
              await withQuote({ quote: 'the kitten to sleep', english: twice, },),
            ];
            expect([missing, empty, ambiguous,].map(function placeOf(run,): unknown {
              const [, occurrence,] = nonNullishOrThrow(run.consistency.groups[0],).occurrences;
              return [occurrence?.outcome, occurrence?.rendering,];
            },),).toEqual([
              ['unlocated', { kind: 'unlocated', reason: 'not-found', },],
              ['unlocated', { kind: 'unlocated', reason: 'empty', },],
              ['unlocated', { kind: 'unlocated', reason: 'ambiguous', },],
            ],);
            expect(changedTexts({ consistency: missing.consistency, },).map(function indexOf([index,],): number {
              return index;
            },),).toEqual([1, 2, 3,],);
            expect(nonNullishOrThrow(missing.consistency.slices[1],).text,).toBe(
              '## Three\n\nIn the evening, the sunlight on the sill lulled the kitten to sleep, and the fish snacks '
                + 'smiled too. So this cat must be a sweet little treasure.',
            );
          },
        },),
        it({
          name: 'LEAVES THE GROUP AS IT STANDS WHEN THE PLACE A MAJORITY CHOSE HAS NO LOCATED QUOTE',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: function quotesNothingForTheFirst({ sheet, },): SeatBehaviour {
                return isRefrainSheet({ sheet, },)
                  ? firstPlaceWins({ sheet, },)
                  : {
                    kind: 'answers',
                    text: ballotText({
                      quotes: [SENTENCE_QUOTES[1] as Record<string, string>,],
                      choice: 'OCCURRENCE 1',
                      keep: [],
                    },),
                  };
              },
              entryCacheDir: dir.path,
            },);
            const sentence = nonNullishOrThrow(run.consistency.groups[0],);
            expect(sentence.verdict.kind === 'asked' ? sentence.verdict.winner : null,).toEqual({
              kind: 'none',
              reason: 'chosen-unlocated',
            },);
            expect(outcomesOf({ consistency: run.consistency, group: 0, },),).toEqual(['untouched', 'untouched',],);
          },
        },),
        it({
          name: 'LOCATES A FRAME\'S PLACE WITHOUT A SLOT WHEN ITS SLOT IS NOT IN ITS SPAN, leaves it to the writers since '
            + 'code cannot carry the pattern, and carries the frame to the others',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: function slotNotInQuote({ sheet, },): SeatBehaviour {
                return isRefrainSheet({ sheet, },)
                  ? {
                    kind: 'answers',
                    text: ballotText({
                      quotes: [
                        REFRAIN_QUOTES[0] as Record<string, string>,
                        { ...REFRAIN_QUOTES[1], slot: 'bored', } as Record<string, string>,
                        REFRAIN_QUOTES[2] as Record<string, string>,
                        REFRAIN_QUOTES[3] as Record<string, string>,
                      ],
                      choice: 'OCCURRENCE 1',
                      keep: [],
                    },),
                  }
                  : firstPlaceWins({ sheet, },);
              },
              entryCacheDir: dir.path,
            },);
            expect(outcomesOf({ consistency: run.consistency, group: 1, },),).toEqual([
              'chosen',
              'kept-current',
              'replaced',
              'replaced',
            ],);
            expect(nonNullishOrThrow(run.consistency.groups[1],).occurrences[1]?.rendering,).toEqual({
              kind: 'located',
              quote: 'So this cat is surely a lazy little treasure',
              start: 58,
            },);
            expect(nonNullishOrThrow(run.consistency.groups[1],).carries?.[0]?.decision,).toEqual({
              kind: 'none',
              reason: 'no-alternative',
            },);
          },
        },),
        it({
          name: 'LEAVES THE LATER OF TWO GROUPS\' REPLACEMENTS OUT WHEN THEIR WORDS OVERLAP IN ONE SLICE, recording '
            + 'overlapped, the earlier group winning',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: sentenceQuoteSpansTheRefrain,
              entryCacheDir: dir.path,
            },);
            expect(outcomesOf({ consistency: run.consistency, group: 1, },),).toEqual([
              'chosen',
              'replaced',
              'overlapped',
              'replaced',
            ],);
            expect(changedTexts({ consistency: run.consistency, },)[1],).toEqual([
              2,
              '## Three\n\nIn the evening, the sunny window put the kitten to sleep.',
            ],);
          },
        },),
        it({
          name: 'READS BACK THE BALLOTS OF A SETTLED GATHER ON A RESUME, asking nobody and landing on the same page',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const first = await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, },);
            const second = await runCatPass({
              behave: function throwsAlways(): SeatBehaviour {
                return { kind: 'throws', };
              },
              entryCacheDir: dir.path,
            },);
            expect(first.asked.length,).toBe(50,);
            expect(second.asked,).toEqual([],);
            expect(second.consistency,).toEqual(first.consistency,);
          },
        },),
        it({
          name: 'ASKS AGAIN WHEN THE SHEET HAS CHANGED, since the key holds the whole sheet: a slice\'s English changed '
            + 'between runs is a new question',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, },);
            const prepared = preparedCatPage();
            const archive = archiveEnglishOf({ prepared, },);
            const second = await runCatPass({
              behave: firstPlaceWins,
              entryCacheDir: dir.path,
              prepared,
              english: [
                archive[0] as string,
                archive[1] as string,
                archive[2] as string,
                '## Four\n\nThe rain in the yard sent the kitten running home. So this cat is certainly a warm little treasure box.',
              ],
            },);
            expect(second.asked.length,).toBe(5,);
          },
        },),
        it({
          name: 'SHOWS EVERY SHEET THE PAGE\'S DECLARED NAMES AND CITED REFERENCES, and a bare page neither',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const whole = preparedCatPage();
            const named = await runCatPass({
              behave: firstPlaceWins,
              entryCacheDir: dir.path,
              prepared: {
                ...whole,
                identityContext: '- name: ORIGINAL declares "咪咪", TRANSLATION declares "Mittens"',
                referenceContext: '- reference 1 https://example.org/mittens ("Mittens"): Mittens had a sister.',
              },
            },);
            await using other = await scratchDir({ prefix: 'pass-consistency-', },);
            const bare = await runCatPass({ behave: firstPlaceWins, entryCacheDir: other.path, },);
            expect([
              named.sheets.every(function carries(sheet,): boolean {
                return sheet.includes('TRANSLATION declares "Mittens"',) && sheet.includes('Mittens had a sister.',);
              },),
              bare.sheets.some(function carries(sheet,): boolean {
                return sheet.includes('Mittens',);
              },),
            ],).toEqual([true, false,],);
          },
        },),
        it({
          name: 'LEAVES A GROUP WHOSE HANDLING FAILED AS IT STANDS and says so in its verdict, without stopping the '
            + 'other groups or the pass',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const prepared = withBlankedBlocks({ prepared: preparedCatPage(), },);
            const run = await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, prepared, },);
            /**
             What each group's handling failed with: the offsets of its first place read other words in the block shown.
             */
            const detail = 'unreachable: the offsets of a place point at other words in the text shown than in the whole '
              + 'original (slice 0), and both are cut from the same offsets';
            expect(run.consistency.groups.map(function verdictOf(group,): unknown {
              return group.verdict;
            },),).toEqual([
              { kind: 'failed', detail, },
              { kind: 'failed', detail, },
            ],);
            expect(run.consistency.slices,).toEqual([],);
            expect(run.asked,).toEqual([],);
          },
        },),
        it({
          name: 'KEEPS THE WHOLE PAGE AS IT STOOD WHEN THE PASS CANNOT RUN, recording the trouble in its findings',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const prepared = preparedCatPage();
            const projected = projectedCatPage({ prepared, english: archiveEnglishOf({ prepared, },), });
            const asked: RosterModelId[] = [];
            const consistency = await runPassConsistency({
              client: scriptedJudgeClient({ behave: firstPlaceWins, asked, sheets: [], },),
              prepared,
              projected: { ...projected, comparison: projected.comparison.slice(0, 2,), },
              contestSlices: [],
              consolidateSlices: [],
              archiveDisputes: new Map(),
              entryCacheDir: dir.path,
              pipelineDigest: DIGEST,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              overlap: 2,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(consistency,).toEqual({
              groups: [],
              slices: [],
              findings: [
                'the consistency pass stopped (unreachable: slice 2 of the preparation has no comparison row, and the '
                  + 'comparison is derived from rows of that preparation), so no slice was changed',
              ],
            },);
            expect(asked,).toEqual([],);
          },
        },),
        it({
          name: 'RECORDS THE SEAT THE ROSTER ASKED, never a seat key the reply carries, and stores only a ballot\'s own fields',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: function repliesWithExtraKeys({ sheet, },): SeatBehaviour {
                const behaviour = firstPlaceWins({ sheet, },);
                return (behaviour.kind === 'answers')
                  ? {
                    kind: 'answers',
                    text: JSON.stringify({ ...JSON.parse(behaviour.text,), seat: 'imposter', note: 'extra', },),
                  }
                  : behaviour;
              },
              entryCacheDir: dir.path,
            },);
            const seats = run.consistency.groups.flatMap(function seatsOf(group,): readonly string[] {
              return (group.verdict.kind === 'asked')
                ? group.verdict.ballots.map(function seatOf(ballot,): string {
                  return ballot.seat;
                },)
                : [];
            },);
            expect(seats.every(function asked(seat,): boolean {
              return run.asked.some(function wasAsked(roster,): boolean {
                return roster === seat;
              },);
            },),).toBe(true,);
            expect(seats.length,).toBe(10,);
            const stored = (await storedGathers({ dir: dir.path, },)).filter(function isFirstRound({ parsed, },): boolean {
              return (parsed.record as { readonly kind?: string; }).kind === undefined;
            },);
            expect(stored.length,).toBe(2,);
            expect(stored.flatMap(function keysOf({ parsed, },): readonly string[] {
              const record = parsed.record as { readonly voices: readonly Record<string, unknown>[]; };
              return record.voices.map(function keys(voice,): string {
                return Object.keys(voice,).toSorted().join(',',);
              },);
            },).every(function plain(keys,): boolean {
              return keys === 'choice,keep,quotes,reason,seat';
            },),).toBe(true,);
          },
        },),
        it({
          name: 'ASKS AGAIN WHEN A STORED GATHER CANNOT BE ONE A FRESH GATHER MADE: a quorum of zero, fewer ballots than '
            + 'the quorum, or one seat heard twice',
          fn: async () => {
            const alterations: readonly (readonly [
              string,
              (record: Record<string, unknown>,) => unknown,
            ])[] = [
              ['quorum zero', function noQuorum(record,): unknown {
                return { ...record, quorum: 0, };
              },],
              ['too few ballots', function fewBallots(record,): unknown {
                return { ...record, voices: (record.voices as unknown[]).slice(0, 1,), };
              },],
              ['one seat twice', function twiceHeard(record,): unknown {
                const [first,] = record.voices as unknown[];
                return { ...record, voices: [first, first, first, first, first,], };
              },],
            ];
            const resumedAsked: Record<string, number> = {};
            for (const [label, change,] of alterations) {
              /* oxlint-disable no-await-in-loop -- three runs, each over its own directory */
              await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
              await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, },);
              await alterStored({ dir: dir.path, change, },);
              const again = await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, },);
              resumedAsked[label] = again.asked.length > 0 ? 1 : 0;
              /* oxlint-enable no-await-in-loop */
            }
            expect(resumedAsked,).toEqual({ 'quorum zero': 1, 'too few ballots': 1, 'one seat twice': 1, },);
          },
        },),
        it({
          name: 'ASKS AGAIN FOR A GATHER WHOSE STORED FILE IS CUT SHORT OR ANSWERS ANOTHER KEY, and resumes the other',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const first = await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, },);
            const [torn, foreign,] = (await storedGathers({ dir: dir.path, },)).filter(function isFirstRound({ parsed, },): boolean {
              return (parsed.record as { readonly kind?: string; }).kind === undefined;
            },);
            await writeFile(
              nonNullishOrThrow(torn,).path,
              JSON.stringify(torn?.parsed,).slice(0, 40,),
              'utf8',
            );
            await writeFile(
              nonNullishOrThrow(foreign,).path,
              JSON.stringify({ ...foreign?.parsed, cacheKey: 'another-question', },),
              'utf8',
            );
            const again = await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, },);
            // Only the two first-round gathers were damaged; the writers' and the choices' records resume.
            expect(again.sheets.length,).toBe(first.sheets.filter(function isBallotSheet(sheet,): boolean {
              return sheetKindOf({ sheet, },) === 'ballot';
            },).length,);
            expect(outcomesByGroup({ consistency: again.consistency, },),)
              .toEqual(outcomesByGroup({ consistency: first.consistency, },),);
          },
        },),
        it({
          name: 'CLOSES ON THE QUORUM OF THE SEATS A PROVIDER SERVES when most of the bench is dark, and carries the '
            + 'winner on that many ballots',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: firstPlaceWins,
              entryCacheDir: dir.path,
              dark: new Set(RUN_LATE_JUDGES.slice(3,),),
            },);
            const sentence = nonNullishOrThrow(run.consistency.groups[0],);
            expect(sentence.verdict.kind === 'asked'
              ? [sentence.verdict.quorum, sentence.verdict.usable >= 2, sentence.verdict.winner,]
              : [],).toEqual([2, true, { kind: 'chosen', label: 'OCCURRENCE 1', },],);
          },
        },),
        it({
          name: 'LEAVES A GROUP AS IT STANDS WHEN THE SEATS A PROVIDER SERVES ANSWER BELOW THE SHORT BENCH\'S QUORUM',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: refrainAnsweredInOrder({ answers: [refrainChoosing({ choice: 'OCCURRENCE 1', },),], },),
              entryCacheDir: dir.path,
              dark: new Set(RUN_LATE_JUDGES.slice(3,),),
            },);
            const refrain = nonNullishOrThrow(run.consistency.groups[1],);
            expect(refrain.verdict.kind === 'asked'
              ? [refrain.verdict.quorum, refrain.verdict.usable, refrain.verdict.winner,]
              : [],).toEqual([2, 1, { kind: 'none', reason: 'no-quorum', },],);
          },
        },),
        it({
          name: 'CARRIES THE SENTENCE INTO A PLACE WHOSE WORDS SIT INSIDE BALANCED EMPHASIS, replacing the emphasised '
            + 'words with the chosen ones when the quote holds both markers',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: everySeatQuotes({
                quotes: [
                  SENTENCE_QUOTES[0] as Record<string, string>,
                  { label: 'OCCURRENCE 2', quote: 'the **sunlight on the sill** lulled the kitten to sleep', },
                ],
                choice: 'OCCURRENCE 1',
                keep: [],
              },),
              entryCacheDir: dir.path,
              prepared: sentencePage(),
              english: [
                '## One\n\nThe sunny window put the kitten to sleep.',
                '## Two\n\nIn the evening, the **sunlight on the sill** lulled the kitten to sleep, and the fish snacks smiled too.',
              ],
            },);
            expect(changedTexts({ consistency: run.consistency, },),).toEqual([
              [1, '## Two\n\nIn the evening, the sunny window put the kitten to sleep, and the fish snacks smiled too.',],
            ],);
          },
        },),
        it({
          name: 'NEVER LEAVES A SLICE WITH UNBALANCED MARKS WHEN A QUOTE STARTS INSIDE EMPHASIS, A LINK, A CODE SPAN OR '
            + 'A FOOTNOTE AND ENDS OUTSIDE IT',
          fn: async () => {
            /**
             English of the second slice and the quote that cuts across its markup.
             */
            const cuts = [
              {
                english: '## Two\n\nIn the evening, the **sunlight on the sill** lulled the kitten to sleep, and the fish snacks smiled too.',
                quote: 'on the sill** lulled the kitten to sleep',
              },
              {
                english: '## Two\n\nIn the evening, [the sunlight on the sill](https://cats.example/sill) lulled the kitten to sleep, '
                  + 'and the fish snacks smiled too.',
                quote: 'sill](https://cats.example/sill) lulled the kitten to sleep',
              },
              {
                english: '## Two\n\nIn the evening, `the sunlight on the sill` lulled the kitten to sleep, and the fish snacks smiled too.',
                quote: 'sill` lulled the kitten to sleep',
              },
            ];
            const marks = ['**', '[', ']', '(', ')', '`',];
            const damaged: string[] = [];
            for (const cut of cuts) {
              /* oxlint-disable no-await-in-loop -- three runs, each over its own directory */
              await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
              const run = await runCatPass({
                behave: everySeatQuotes({
                  quotes: [
                    SENTENCE_QUOTES[0] as Record<string, string>,
                    { label: 'OCCURRENCE 2', quote: cut.quote, },
                  ],
                  choice: 'OCCURRENCE 1',
                  keep: [],
                },),
                entryCacheDir: dir.path,
                prepared: sentencePage(),
                english: ['## One\n\nThe sunny window put the kitten to sleep.', cut.english,],
              },);
              for (const [, text,] of changedTexts({ consistency: run.consistency, },)) {
                for (const mark of marks) {
                  const before = cut.english.split(mark,).length;
                  const after = text.split(mark,).length;
                  if ((before % 2) !== (after % 2))
                    damaged.push(`${mark} in ${text}`,);
                }
              }
              /* oxlint-enable no-await-in-loop */
            }
            expect(damaged,).toEqual([],);
          },
        },),
        it({
          name: 'NEVER STRANDS A FOOTNOTE DEFINITION OR CARRIES A FOOTNOTE MARK WITHOUT ITS DEFINITION when a quote holds one',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: everySeatQuotes({
                quotes: [
                  { label: 'OCCURRENCE 1', quote: 'The sunny window put the kitten to sleep[^1]', },
                  { label: 'OCCURRENCE 2', quote: 'the sunlight on the sill lulled the kitten to sleep', },
                ],
                choice: 'OCCURRENCE 1',
                keep: [],
              },),
              entryCacheDir: dir.path,
              prepared: sentencePage(),
              english: [
                '## One\n\nThe sunny window put the kitten to sleep[^1].\n\n[^1]: A note.',
                '## Two\n\nIn the evening, the sunlight on the sill lulled the kitten to sleep, and the fish snacks smiled too.',
              ],
            },);
            expect(changedTexts({ consistency: run.consistency, },).filter(function carriesMark([, text,],): boolean {
              return text.includes('[^1]',);
            },),).toEqual([],);
          },
        },),
        it({
          name: 'REPLACES ONE PLACE\'S WORDS WITH THE CHOSEN QUOTE WHEN THE CHOSEN QUOTE IS THE START OF ITS SLICE AND THE '
            + 'RECEIVING QUOTE IS THE END OF ITS OWN, closing punctuation staying where it stands',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: everySeatQuotes({
                quotes: [
                  { label: 'OCCURRENCE 1', quote: 'The sunny window put the kitten to sleep.', },
                  { label: 'OCCURRENCE 2', quote: 'the sunlight on the sill lulled the kitten to sleep, and the fish snacks smiled too.', },
                ],
                choice: 'OCCURRENCE 1',
                keep: [],
              },),
              entryCacheDir: dir.path,
              prepared: sharedSlicePage(),
            },);
            expect(changedTexts({ consistency: run.consistency, },),).toEqual([
              [0, 'The sunny window put the kitten to sleep.\n\nIn the evening, the sunny window put the kitten to sleep.',],
            ],);
          },
        },),
        it({
          name: 'LEAVES BOTH PLACES OF ONE SLICE UNLOCATED WHEN THEIR QUOTES OVERLAP, since one run of words cannot be '
            + 'two renderings, and changes nothing',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: everySeatQuotes({
                quotes: [
                  { label: 'OCCURRENCE 1', quote: 'The sunny window', },
                  { label: 'OCCURRENCE 2', quote: 'The sunny window put the kitten to sleep', },
                ],
                choice: 'OCCURRENCE 1',
                keep: [],
              },),
              entryCacheDir: dir.path,
              prepared: sharedSlicePage(),
            },);
            const sentence = nonNullishOrThrow(run.consistency.groups[0],);
            expect({
              renderings: sentence.occurrences.map(function renderingOf(occurrence,): unknown {
                return occurrence.rendering;
              },),
              winner: (sentence.verdict.kind === 'asked') ? sentence.verdict.winner : null,
              outcomes: outcomesOf({ consistency: run.consistency, group: 0, },),
              changed: run.consistency.slices,
            },).toEqual({
              renderings: [
                { kind: 'unlocated', reason: 'overlaps-another', },
                { kind: 'unlocated', reason: 'overlaps-another', },
              ],
              winner: { kind: 'none', reason: 'chosen-unlocated', },
              outcomes: ['untouched', 'untouched',],
              changed: [],
            },);
          },
        },),
        it({
          name: 'READS A REPEAT THAT CONTAINS AN HTML COMMENT IN ONE PLACE, the place\'s offsets covering the comment, and '
            + 'the record reads back through the reader',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const prepared = prepareDocumentPair({
              sourceText: '## 一\n\n窗台上的阳光让小猫睡着了。\n\n## 二\n\n傍晚，窗台上的阳光<!-- 注 -->让小猫睡着了，小鱼干也笑了。\n',
              targetText: '## One\n\nThe sunny window put the kitten to sleep.\n\n## Two\n\nIn the evening, the sunlight on the sill '
                + 'lulled the kitten to sleep, and the fish snacks smiled too.\n',
            },);
            const run = await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, prepared, },);
            const group = nonNullishOrThrow(run.consistency.groups[0],);
            expect(group.occurrences.map(function widthOf(occurrence,): number {
              return occurrence.endOffset - occurrence.startOffset;
            },),).toEqual([12, 22,],);
            expect(outcomesOf({ consistency: run.consistency, group: 0, },),).toEqual(['chosen', 'replaced',],);
            expect(parseConsistency({
              value: structuredClone(run.consistency,),
              path: 'cat.consistency',
              generation: 16,
              sliceIndexes: new Set([0, 1, 2, 3,],),
            },),).toEqual(run.consistency,);
          },
        },),
        it({
          name: 'FINDS NO GROUP IN A REPEAT BETWEEN STRAIGHT QUOTATION MARKS, asking nobody',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const prepared = prepareDocumentPair({
              sourceText: '## 一\n\n她说"窗台上的阳光让小猫睡着了"。\n\n## 二\n\n他喊"窗台上的阳光让小猫睡着了"！\n',
              targetText: '## One\n\nShe said "the sunny window put the kitten to sleep".\n\n## Two\n\nHe said "the sunlight on the sill lulled the kitten to sleep".\n',
            },);
            const run = await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, prepared, },);
            expect({ consistency: run.consistency, asked: run.asked, },).toEqual({ consistency: NO_CONSISTENCY, asked: [], },);
          },
        },),
        it({
          name: 'SHOWS EVERY SHEET THE TENSE CANDIDATES OF EACH SLICE\'S ENGLISH when the entry records a death, and a '
            + 'sheet of an entry that records none nothing of the kind',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const whole = preparedCatPage();
            const archive = archiveEnglishOf({ prepared: whole, },);
            const english = [
              '## One\n\nMittens is asleep in the sunny window. So this cat must be a chubby little treasure.',
              ...archive.slice(1,),
            ];
            const died = await runCatPass({
              behave: firstPlaceWins,
              entryCacheDir: dir.path,
              prepared: { ...whole, lifeEnded: true, subjectNames: ['Mittens',], },
              english,
            },);
            await using other = await scratchDir({ prefix: 'pass-consistency-', },);
            const living = await runCatPass({ behave: firstPlaceWins, entryCacheDir: other.path, prepared: whole, english, },);
            expect([
              died.sheets.length > 0,
              died.sheets.filter(function isBallotSheet(sheet,): boolean {
                return sheetKindOf({ sheet, },) === 'ballot';
              },).every(function showsBlock(sheet,): boolean {
                return sheet.includes('OCCURRENCE 1 TRANSLATION:\n- "Mittens is asleep in the sunny window." '
                  + '(matched "Mittens is", rule copula-or-auxiliary)',);
              },),
              living.sheets.filter(function isBallotSheet(sheet,): boolean {
                return sheetKindOf({ sheet, },) === 'ballot';
              },).some(function showsBlock(sheet,): boolean {
                return sheet.includes('TENSE CANDIDATES',);
              },),
            ],).toEqual([true, true, false,],);
          },
        },),
        it({
          name: 'ASKS AGAIN WHEN THE ENTRY\'S RECORDED DEATH OR ITS SUBJECT NAMES CHANGE, though the English shown is the same',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const whole = preparedCatPage();
            await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, prepared: whole, },);
            const living = await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, prepared: whole, },);
            const died = await runCatPass({
              behave: firstPlaceWins,
              entryCacheDir: dir.path,
              prepared: { ...whole, lifeEnded: true, subjectNames: ['Mittens',], },
            },);
            const diedAgain = await runCatPass({
              behave: firstPlaceWins,
              entryCacheDir: dir.path,
              prepared: { ...whole, lifeEnded: true, subjectNames: ['Mittens',], },
            },);
            const renamed = await runCatPass({
              behave: firstPlaceWins,
              entryCacheDir: dir.path,
              prepared: { ...whole, lifeEnded: true, subjectNames: ['Mittens', 'Tabby',], },
            },);
            expect([
              living.asked.length,
              died.asked.length > 0,
              diedAgain.asked.length,
              renamed.asked.length > 0,
            ],).toEqual([0, true, 0, true,],);
          },
        },),
        it({
          name: 'ASKS AGAIN WHEN THE DECLARED IDENTITY CHANGES, the recorded death date among it, since the sheet shows it',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const whole = preparedCatPage();
            const first = await runCatPass({
              behave: firstPlaceWins,
              entryCacheDir: dir.path,
              prepared: { ...whole, identityContext: '- died: 2025-02-26', lifeEnded: true, },
            },);
            const same = await runCatPass({
              behave: firstPlaceWins,
              entryCacheDir: dir.path,
              prepared: { ...whole, identityContext: '- died: 2025-02-26', lifeEnded: true, },
            },);
            const moved = await runCatPass({
              behave: firstPlaceWins,
              entryCacheDir: dir.path,
              prepared: { ...whole, identityContext: '- died: 2025-03-01', lifeEnded: true, },
            },);
            expect([first.asked.length > 0, same.asked.length, moved.asked.length > 0,],).toEqual([true, 0, true,],);
          },
        },),
        it({
          name: 'KEEPS THE KEY OF A SHEET FOR AN ENTRY THAT RECORDS NO DEATH AS ONE JSON VALUE OF THE VERSION, THE SHEET\'S '
            + 'DIGEST AND THE BENCH, and moves it for an entry that does by the death and the subject names',
          fn: async () => {
            const messages = buildConsistencyMessages({
              subject: {
                repeat: { kind: 'maximal', wording: '窗台上的阳光让小猫睡着了', },
                occurrences: [
                  {
                    sliceIndex: 0,
                    paragraph: '窗台上的阳光让小猫睡着了。',
                    markStart: 0,
                    markEnd: 12,
                    english: { kind: 'present', text: 'The sunny window put the kitten to sleep.', },
                  },
                ],
              },
            },);
            const modelIds = RUN_LATE_JUDGES.slice(0, 2,);
            const bare = createHash('sha256',)
              .update(
              JSON.stringify({
                version: CONSISTENCY_CACHE_VERSION,
                stage: 'ballots',
                sheet: hashContent({ content: JSON.stringify(messages,), },),
                roster: modelIds,
              },),
              'utf8',
              )
              .digest('hex',);
            const keys = {
              plain: consistencyKey({ stage: 'ballots', messages, modelIds, },),
              living: consistencyKey({ stage: 'ballots', messages, modelIds, lifeEnded: false, subjectNames: ['Mittens',], },),
              died: consistencyKey({ stage: 'ballots', messages, modelIds, lifeEnded: true, subjectNames: ['Mittens',], },),
              diedNamed: consistencyKey({
                stage: 'ballots',
                messages,
                modelIds,
                lifeEnded: true,
                subjectNames: ['Mittens', 'Tabby',],
              },),
              diedUnnamed: consistencyKey({ stage: 'ballots', messages, modelIds, lifeEnded: true, },),
            };
            expect([
              keys.plain === bare,
              keys.living === bare,
              keys.died !== bare,
              keys.diedNamed !== keys.died,
              keys.diedUnnamed !== keys.died,
              keys.diedUnnamed !== bare,
            ],).toEqual([true, true, true, true, true, true,],);
          },
        },),
        it({
          name: 'PASSES AN ABORT ON instead of recording it as trouble, since a stop is the caller\'s and not this pass\'s',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const stop = new AbortController();
            stop.abort(new Error('the entry was stopped',),);
            await expect(runCatPass({
              behave: firstPlaceWins,
              entryCacheDir: dir.path,
              signal: stop.signal,
            },),).rejects.toBe(stop.signal.reason,);
          },
        },),
      ],
    },),
    describe({
      name: `${runPassConsistency.name} logging a refusal`,
      concurrency: 1,
      children: [
        it({
          name: 'WITHHOLDS A SPLICE A FLOOR REFUSES FROM THE BENCH, keeping the slice\'s wording, recording kept-current '
            + 'with the floor\'s finding, and warning',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const whole = preparedCatPage();
            const prepared = { ...whole, declaredNames: ['Mittens',], };
            const archive = archiveEnglishOf({ prepared, },);
            const named = '## Two\n\nThe smell of fish in the kitchen woke the kitten. '
              + 'So this cat is surely Mittens, a lazy little treasure.';
            const { result: run, warned: warnings, } = await warnLinesDuring({
              run: async function passWithTheNameDropped(): Promise<CatRun> {
                return await runCatPass({
                  behave: slotWithTheName,
                  entryCacheDir: dir.path,
                  prepared,
                  english: [archive[0] as string, named, archive[2] as string, archive[3] as string,],
                },);
              },
            },);
            expect(outcomesOf({ consistency: run.consistency, group: 1, },),).toEqual([
              'chosen',
              'kept-current',
              'replaced',
              'replaced',
            ],);
            expect(changedTexts({ consistency: run.consistency, },).map(function indexOf([index,],): number {
              return index;
            },),).toEqual([2, 3,],);
            expect(run.consistency.findings.length,).toBe(1,);
            expect(run.consistency.findings[0]?.startsWith(
              'group 2: consistency-candidate-refused (splice of OCCURRENCE 2): ',
            ),).toBe(true,);
            expect(warnings.some(function namesThePlace(line,): boolean {
              return line.includes(
                'withheld the splice for OCCURRENCE 2 from the bench: the floors refuse its paragraph: ',
              );
            },),).toBe(true,);
            const carry = nonNullishOrThrow(run.consistency.groups[1],).carries?.[0];
            expect([carry?.label, carry?.decision, carry?.candidates.map(function kindOf({ kind, },): string {
              return kind;
            },),],).toEqual(['OCCURRENCE 2', { kind: 'none', reason: 'no-alternative', }, ['current',],],);
          },
        },),
        it({
          name: 'COUNTS THE BALLOTS IN HAND WHEN THE STORE REFUSES THEM, warning that a resume will ask again and '
            + 'leaving the page as the ballots decide',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const prepared = preparedCatPage();
            const archive = archiveEnglishOf({ prepared, },);
            // The first run opens the store, so the second finds it and can only fail on writing an answer.
            await runCatPass({ behave: firstPlaceWins, entryCacheDir: dir.path, prepared, },);
            using readOnly = readOnlyDirectory({ path: dir.path, },);
            const { result: run, warned: warnings, } = await warnLinesDuring({
              run: async function passWithAStoreThatRefuses(): Promise<CatRun> {
                return await runCatPass({
                  behave: firstPlaceWins,
                  entryCacheDir: dir.path,
                  prepared,
                  english: [
                    archive[0] as string,
                    archive[1] as string,
                    archive[2] as string,
                    '## Four\n\nThe rain in the yard sent the kitten running home soon. So this cat is certainly a warm little dumpling.',
                  ],
                },);
              },
            },);
            expect(readOnly.path,).toBe(dir.path,);
            expect(outcomesOf({ consistency: run.consistency, group: 1, },),).toEqual([
              'chosen',
              'replaced',
              'replaced',
              'replaced',
            ],);
            expect(warnings.some(function namesTheStore(line,): boolean {
              return line.includes('page-consistency: the answer could not be stored (',)
                && line.includes('); it is counted all the same',);
            },),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: `${runPassConsistency.name} carrying the chosen rendering by writing`,
      concurrency: 1,
      children: [
        it({
          name: 'CARRIES THE LIVE GROUP TWO BY WRITING: the writers drop the "also" that renders a word standing before the '
            + 'repeat, the splice carries it, and the bench chooses the paragraph without it',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: groupTwoBallots,
              others: groupTwoWritersAndBench,
              entryCacheDir: dir.path,
              prepared: groupTwoPage(),
            },);
            const group = nonNullishOrThrow(run.consistency.groups[0],);
            const carry = nonNullishOrThrow(group.carries?.[0],);
            const byKind = Object.fromEntries(carry.candidates.map(function entryOf({ kind, label, writers, text, },) {
              return [kind, { label, writers, text, }];
            },),);
            expect({
              winner: group.verdict.kind === 'asked' ? [group.verdict.usable, group.verdict.winner,] : null,
              outcomes: outcomesOf({ consistency: run.consistency, group: 0, },),
              changed: changedTexts({ consistency: run.consistency, },),
              candidateKinds: carry.candidates.map(function kindOf({ kind, },): string {
                return kind;
              },).toSorted(),
              current: byKind.current?.writers,
              splice: [byKind.splice?.writers, byKind.splice?.text,],
              written: [byKind.writer?.writers, byKind.writer?.text,],
              decision: carry.decision,
              chosenIsTheWriters: (carry.decision.kind === 'chosen') && (carry.decision.label === byKind.writer?.label),
              writerSheetsAsked: run.sheets.some(function isWriter(sheet,): boolean {
                return sheetKindOf({ sheet, },) === 'writer';
              },),
            },).toEqual({
              winner: [4, { kind: 'chosen', label: 'OCCURRENCE 1', },],
              outcomes: ['chosen', 'replaced',],
              changed: [[1, '## Two\n\nShe wanted to stay by more people\'s side looking cute and bring sunshine to more people.',],],
              candidateKinds: ['current', 'splice', 'writer',],
              current: [RUN_TRANSLATORS[3],],
              splice: [
                [RUN_TRANSLATORS[2],],
                'She also wanted to stay by more people\'s side looking cute and bring sunshine to more people.',
              ],
              written: [
                RUN_TRANSLATORS.slice(0, 2,),
                'She wanted to stay by more people\'s side looking cute and bring sunshine to more people.',
              ],
              decision: carry.decision,
              chosenIsTheWriters: true,
              writerSheetsAsked: true,
            },);
            expect(parseConsistency({
              value: structuredClone(run.consistency,),
              path: 'cat.consistency',
              generation: 16,
              sliceIndexes: new Set([0, 1,],),
            },),).toEqual(run.consistency,);
          },
        },),
        it({
          name: 'KEEPS THE PAGE\'S ENGLISH WHEN THE BENCH CHOOSES THE PARAGRAPH AS IT STANDS, though a writer and the code '
            + 'offered others, recording the choice and no replacement',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: groupTwoBallots,
              others: function benchKeepsCurrent(call,): SeatBehaviour {
                return (sheetKindOf({ sheet: call.sheet, },) === 'choice')
                  ? choosing({
                    choice: candidateTextsOf({ sheet: call.sheet, },)
                      .find(function isCurrent({ current, },): boolean {
                        return current;
                      },)
                      ?.label
                      ?? '',
                  },)
                  : groupTwoWritersAndBench(call,);
              },
              entryCacheDir: dir.path,
              prepared: groupTwoPage(),
            },);
            const carry = nonNullishOrThrow(run.consistency.groups[0]?.carries?.[0],);
            const current = carry.candidates.find(function isCurrent({ kind, },): boolean {
              return kind === 'current';
            },);
            expect({
              outcomes: outcomesOf({ consistency: run.consistency, group: 0, },),
              slices: run.consistency.slices,
              decision: carry.decision,
              currentLabel: current?.label,
            },).toEqual({
              outcomes: ['chosen', 'kept-current',],
              slices: [],
              decision: { kind: 'chosen', label: current?.label, },
              currentLabel: current?.label,
            },);
          },
        },),
        it({
          name: 'KEEPS THE PAGE\'S ENGLISH WHEN THE BENCH HAS NO MAJORITY AMONG THE CANDIDATES TWICE: the split is run off '
            + 'among the finalists, the run-off splits again, and the carry records why',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: groupTwoBallots,
              others: function benchSplits(call,): SeatBehaviour {
                if (sheetKindOf({ sheet: call.sheet, },) !== 'choice')
                  return groupTwoWritersAndBench(call,);
                /**
                 The candidates the sheet shows.
                 */
                const candidates = candidateTextsOf({ sheet: call.sheet, },);
                /**
                 Position of the seat among the late judges: the first two choose the first candidate and the next two the second.
                 */
                const position = RUN_LATE_JUDGES.indexOf(call.seat,);
                return (position >= 4) ? { kind: 'throws', } : choosing({ choice: candidates[position % 2]?.label ?? '', },);
              },
              entryCacheDir: dir.path,
              prepared: groupTwoPage(),
            },);
            const carry = nonNullishOrThrow(run.consistency.groups[0]?.carries?.[0],);
            expect({
              outcomes: outcomesOf({ consistency: run.consistency, group: 0, },),
              slices: run.consistency.slices,
              decision: carry.decision,
              usable: carry.usable,
              runOffFinalists: carry.runOff?.finalists,
              runOffUsable: carry.runOff?.usable,
            },).toEqual({
              outcomes: ['chosen', 'kept-current',],
              slices: [],
              decision: { kind: 'none', reason: 'no-majority', },
              usable: 4,
              runOffFinalists: ['CANDIDATE 1', 'CANDIDATE 2',],
              runOffUsable: 4,
            },);
          },
        },),
        it({
          name: 'CONTAINS A CARRY THAT FAILS: a writer that cannot be read is withheld with a finding, and the place keeps its '
            + 'English when no candidate but the paragraph as it stands remains',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const { result: run, warned: warnings, } = await warnLinesDuring({
              run: async function passWithBrokenWriters(): Promise<CatRun> {
                return await runCatPass({
                  behave: groupTwoBallots,
                  others: function writersReturnTheWrongThing(call,): SeatBehaviour {
                    if (sheetKindOf({ sheet: call.sheet, },) === 'writer')
                      return { kind: 'answers', text: JSON.stringify({ translation: '## A heading nobody asked for', },), };
                    return groupTwoWritersAndBench(call,);
                  },
                  entryCacheDir: dir.path,
                  prepared: groupTwoPage(),
                },);
              },
            },);
            const group = nonNullishOrThrow(run.consistency.groups[0],);
            const carry = nonNullishOrThrow(group.carries?.[0],);
            expect({
              outcomes: outcomesOf({ consistency: run.consistency, group: 0, },),
              slices: run.consistency.slices,
              kinds: carry.candidates.map(function kindOf({ kind, },): string {
                return kind;
              },).toSorted(),
              unreadable: run.consistency.findings.some(function isFinding(finding,): boolean {
                return finding.startsWith('group 1: consistency-candidate-',);
              },),
              warned: warnings.some(function namesTheSeat(line,): boolean {
                return line.includes('withheld',) && line.includes('from the bench',);
              },),
            },).toEqual({
              outcomes: ['chosen', 'kept-current',],
              slices: [],
              kinds: ['current', 'splice',],
              unreadable: true,
              warned: true,
            },);
          },
        },),
        it({
          name: 'CARRIES THE LIVE GROUP ONE BY RUNNING OFF THE TIE AND WRITING: the two places that tied for first are put to '
            + 'the bench once more on a sheet showing only them, and each receiving refrain keeps its own slot in the chosen '
            + 'frame',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: groupOneBallots,
              others: groupOneRunOffAndWriters,
              entryCacheDir: dir.path,
              prepared: groupOnePage(),
            },);
            const group = nonNullishOrThrow(run.consistency.groups[0],);
            const runOffSheets = run.sheets.filter(function isRunOff(sheet,): boolean {
              return sheetKindOf({ sheet, },) === 'run-off';
            },);
            const writerSheets = run.sheets.filter(function isWriter(sheet,): boolean {
              return sheetKindOf({ sheet, },) === 'writer';
            },);
            expect({
              first: group.verdict.kind === 'asked' ? [group.verdict.usable, group.verdict.winner,] : null,
              runOff: group.verdict.kind === 'asked'
                ? [group.verdict.runOff?.finalists, group.verdict.runOff?.usable, group.verdict.runOff?.quorum,]
                : null,
              finalistsShownOnly: runOffSheets.every(function showsOnlyTheTied(sheet,): boolean {
                return sheet.includes('OCCURRENCE 3 ORIGINAL',) && sheet.includes('OCCURRENCE 4 ORIGINAL',)
                  && (!sheet.includes('OCCURRENCE 1 ORIGINAL',)) && (!sheet.includes('OCCURRENCE 2 ORIGINAL',));
              },),
              outcomes: outcomesOf({ consistency: run.consistency, group: 0, },),
              changed: changedTexts({ consistency: run.consistency, },),
              carries: group.carries?.map(function carryOf({ label, candidates, decision, },): unknown {
                return [
                  label,
                  candidates.map(function kindOf({ kind, },): string {
                    return kind;
                  },).toSorted(),
                  decision.kind,
                ];
              },),
              slotsShown: writerSheets.map(function slotOf(sheet,): string {
                return sheetBlockOf({ sheet, label: 'THIS PLACE: SLOT IN THE ORIGINAL', },);
              },).filter(function distinct(slot, at, all,): boolean {
                return all.indexOf(slot,) === at;
              },).toSorted(),
            },).toEqual({
              first: [4, { kind: 'chosen', label: 'OCCURRENCE 3', },],
              runOff: [['OCCURRENCE 3', 'OCCURRENCE 4',], 4, 4,],
              finalistsShownOnly: true,
              outcomes: ['replaced', 'replaced', 'chosen', 'replaced',],
              changed: [
                // The writers' paragraphs are laid out as the page lays out a clause, one clause to a line.
                [0, '## One\n\nSo she was a girl who loved life,\nwasn\'t she?\nUnder the lights,\nin the plainest of lives,\n'
                  + 'she held the most distant of dreams close to her heart.',],
                [1, '## Two\n\nSo she was a girl of extraordinary perseverance,\nwasn\'t she?\nYou see,\namid the starry night,\n'
                  + 'in the frailest of bodies,\nshe bore the most unyielding pressure.',],
                [3, '## Four\n\nSo she was a girl who was gentle and tender-hearted,\nwasn\'t she?\nJust look:\n'
                  + 'inside the headpiece,\nin the narrowest of spaces,\nshe held up the most perfect world.',],
              ],
              carries: [
                ['OCCURRENCE 1', ['current', 'writer',], 'chosen',],
                ['OCCURRENCE 2', ['current', 'writer',], 'chosen',],
                ['OCCURRENCE 4', ['current', 'writer',], 'chosen',],
              ],
              slotsShown: ['热爱生活', '毅力超群', '温柔细腻',].toSorted(),
            },);
            expect(parseConsistency({
              value: structuredClone(run.consistency,),
              path: 'cat.consistency',
              generation: 16,
              sliceIndexes: new Set([0, 1, 2, 3,],),
            },),).toEqual(run.consistency,);
          },
        },),
        it({
          name: 'CARRIES TO TWO PLACES OF ONE PARAGRAPH IN ONE REWRITE: the writers are asked once for the paragraph, the bench '
            + 'chooses once among the current paragraph, the splice of both places and a writer\'s paragraph, and no place '
            + 'is left out as overlapping its own group',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: twoInBlockBallots,
              others: twoInBlockWritersAndBench,
              entryCacheDir: dir.path,
              prepared: twoInBlockPage(),
            },);
            const group = nonNullishOrThrow(run.consistency.groups[0],);
            /**
             The seat each sheet was shown to and the sheet, of the questions of one kind.
             */
            function askedAbout(
              { kind, }: { readonly kind: 'ballot' | 'writer' | 'choice'; },
            ): readonly (readonly [string, string])[] {
              return run.sheets
                .map(function pairOf(sheet, at,): readonly [string, string] {
                  return [run.asked[at] ?? '', sheet,];
                },)
                .filter(function isOfKind([, sheet,],): boolean {
                  return sheetKindOf({ sheet, },) === kind;
                },);
            }
            /**
             The distinct seats among pairs, in roster order of their names.
             */
            function seatsOf({ pairs, }: { readonly pairs: readonly (readonly [string, string])[]; },): readonly string[] {
              return [...new Set(pairs.map(function seatOf([seat,],): string {
                return seat;
              },),),].toSorted();
            }
            /**
             The distinct sheets among pairs.
             */
            function distinctSheetsOf({ pairs, }: { readonly pairs: readonly (readonly [string, string])[]; },): number {
              return new Set(pairs.map(function sheetOf([, sheet,],): string {
                return sheet;
              },),).size;
            }
            const writerPairs = askedAbout({ kind: 'writer', },);
            const choicePairs = askedAbout({ kind: 'choice', },);
            const currentBlock = `${TWO_IN_BLOCK_WORDS[0]}. The kitchen was warm. ${TWO_IN_BLOCK_WORDS[1]}.`;
            const writtenBlock = `${TWO_IN_BLOCK_WORDS[2]}.\nThe kitchen felt warm.\n${TWO_IN_BLOCK_WORDS[2]}.`;
            const splicedBlock = `${TWO_IN_BLOCK_WORDS[2]}. The kitchen was warm. ${TWO_IN_BLOCK_WORDS[2]}.`;
            // The choice asks its bench in an order the prompt rotates, and stops once the quorum is heard, so which of the
            // judges the script makes throw are asked moves with the sheet's text; the four that answer are always asked.
            const choiceSeats = seatsOf({ pairs: choicePairs, },);
            expect({
              outcomes: outcomesOf({ consistency: run.consistency, group: 0, },),
              changed: changedTexts({ consistency: run.consistency, },),
              labels: group.carries?.map(function labelOf({ label, },): string {
                return label;
              },),
              candidates: group.carries?.map(function candidatesOf({ candidates, },): readonly string[] {
                return candidates.map(function textOf({ kind, text, },): string {
                  return `${kind}: ${text}`;
                },).toSorted();
              },),
              decisions: group.carries?.map(function decisionOf({ decision, },): string {
                return decision.kind;
              },),
              sameChoice: JSON.stringify(group.carries?.[0]?.decision,) === JSON.stringify(group.carries?.[1]?.decision,),
              writerSeats: seatsOf({ pairs: writerPairs, }),
              writerSheets: distinctSheetsOf({ pairs: writerPairs, }),
              choiceSeatsOffBench: choiceSeats.filter(function offBench(seat,): boolean {
                return !RUN_LATE_JUDGES.includes(seat as RosterModelId,);
              },),
              answeringAsked: RUN_LATE_JUDGES.slice(0, 4,).every(function asked(seat,): boolean {
                return choiceSeats.includes(seat,);
              },),
              choiceSheets: distinctSheetsOf({ pairs: choicePairs, }),
              placesShown: writerPairs.every(function showsBoth([, sheet,],): boolean {
                return (sheetBlockOf({ sheet, label: 'THIS PLACE 1: ORIGINAL', },) !== '')
                  && (sheetBlockOf({ sheet, label: 'THIS PLACE 2: ORIGINAL', },) !== '')
                  && (sheetBlockOf({ sheet, label: 'CURRENT ENGLISH PARAGRAPH', },) === currentBlock);
              },),
            },).toEqual({
              outcomes: ['replaced', 'replaced', 'chosen',],
              changed: [[0, `## One\n\n${writtenBlock}`,],],
              labels: ['OCCURRENCE 1', 'OCCURRENCE 2',],
              candidates: [
                [`current: ${currentBlock}`, `splice: ${splicedBlock}`, `writer: ${writtenBlock}`,],
                [`current: ${currentBlock}`, `splice: ${splicedBlock}`, `writer: ${writtenBlock}`,],
              ],
              decisions: ['chosen', 'chosen',],
              sameChoice: true,
              writerSeats: [...RUN_TRANSLATORS,].toSorted(),
              writerSheets: 1,
              choiceSeatsOffBench: [],
              answeringAsked: true,
              choiceSheets: 1,
              placesShown: true,
            },);
            // The seats the script makes throw are the only findings: judges of the first round and of the choice, six
            // writers, each a lost voice, and the quorums the rest closed on.
            // Both rounds ask their bench in an order the prompt rotates (it moved when the house tense rule changed the prompt,
            // and when the sheets came to list their labels), and stop once the four that answer are heard, so which of the
            // judges the script makes throw are asked, and which of those are lost before the quorum closes, move with the
            // sheet. What holds whatever the order: each lost voice is a throwing judge the round asked, in roster order, and the
            // round's ratio counts every judge it asked.
            const ballotSeats = seatsOf({ pairs: askedAbout({ kind: 'ballot', },), },);
            /**
             The first finding of a round's ratio, which ends the voices it lost.
             */
            function ratioAt({ stage, }: { readonly stage: string; },): number {
              return run.consistency.findings.findIndex(function isRatio(finding,): boolean {
                return finding.startsWith(`group 1: stage-roster-incomplete (${stage} `,);
              },);
            }
            /**
             Whether findings are lost voices of throwing judges a round asked, in roster order.
             */
            function lostThrowingAsked(
              { findings, stage, asked, }: {
                readonly findings: readonly string[];
                readonly stage: string;
                readonly asked: readonly string[];
              },
            ): boolean {
              /**
               The lost voice each throwing judge the round asked would leave, in roster order.
               */
              const possible = RUN_LATE_JUDGES
                .slice(4,)
                .filter(function wasAsked(seat,): boolean {
                  return asked.includes(seat,);
                },)
                .map(function lostAt(seat,): string {
                  return `group 1: stage-voice-lost (${stage} ${seat})`;
                },);
              return (findings.length > 0)
                && findings.every(function inOrder(finding, at,): boolean {
                  return (possible.includes(finding,))
                    && ((at === 0) || (possible.indexOf(finding,) > possible.indexOf(findings[at - 1] ?? '',)));
                },);
            }
            const firstRatio = ratioAt({ stage: 'page-consistency', },);
            const choiceRatio = ratioAt({ stage: 'page-consistency-choice', },);
            const writersEnd = firstRatio + 8;
            expect({
              firstLost: lostThrowingAsked({
                findings: run.consistency.findings.slice(0, firstRatio,),
                stage: 'page-consistency',
                asked: ballotSeats,
              },),
              firstRatio: run.consistency.findings[firstRatio],
              choiceLost: lostThrowingAsked({
                findings: run.consistency.findings.slice(writersEnd, choiceRatio,),
                stage: 'page-consistency-choice',
                asked: choiceSeats,
              },),
              choiceRatio: run.consistency.findings[choiceRatio],
              total: run.consistency.findings.length,
            },).toEqual({
              firstLost: true,
              firstRatio: `group 1: stage-roster-incomplete (page-consistency 4/${ballotSeats.length})`,
              choiceLost: true,
              choiceRatio: `group 1: stage-roster-incomplete (page-consistency-choice 4/${choiceSeats.length})`,
              total: choiceRatio + 1,
            },);
            expect(run.consistency.findings.slice(firstRatio + 1, writersEnd,),).toEqual([
              'group 1: stage-voice-lost (page-consistency-writers hf:moonshotai/Kimi-K3)',
              'group 1: stage-voice-lost (page-consistency-writers minimax-m3)',
              'group 1: stage-voice-lost (page-consistency-writers gemma-4-26b-a4b-it)',
              'group 1: stage-voice-lost (page-consistency-writers glm-5.3)',
              'group 1: stage-voice-lost (page-consistency-writers deepseek-v4.1-flash)',
              'group 1: stage-voice-lost (page-consistency-writers google.gemma-4-e2b)',
              'group 1: stage-quorum-unmet (page-consistency-writers 2/8)',
            ],);
            expect(parseConsistency({
              value: structuredClone(run.consistency,),
              path: 'cat.consistency',
              generation: 16,
              sliceIndexes: new Set([0, 1,],),
            },),).toEqual(run.consistency,);
          },
        },),
        it({
          name: 'KEEPS A PLACE ACROSS A PARAGRAPH BOUNDARY WHEN THE CHOSEN WORDS HOLD NO BREAK, and asks the writers for the '
            + 'paragraph beside it: words that hold another number of block breaks have no splice, so no edit addresses the '
            + 'crossing words and the section the pass writes is one its own reader accepts',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: crossingBallots,
              others: crossingWritersAndBench,
              entryCacheDir: dir.path,
              prepared: crossingPage(),
            },);
            expect({
              outcomes: outcomesOf({ consistency: run.consistency, group: 0, },),
              changed: changedTexts({ consistency: run.consistency, },),
              writersAsked: run.sheets.some(function isWriterSheet(sheet,): boolean {
                return sheetKindOf({ sheet, },) === 'writer';
              },),
              notAsked: run.consistency.findings.filter(function isNotAsked(finding,): boolean {
                return finding.includes('consistency-writers-not-asked',);
              },),
            },).toEqual({
              outcomes: ['kept-current', 'replaced', 'chosen',],
              changed: [
                [
                  0,
                  `## One\n\n${CROSSING_WORDS[0]}.\nThe kitchen felt warm.\n${CROSSING_WORDS[2]}.`,
                ],
              ],
              writersAsked: true,
              notAsked: [
                'group 1: consistency-writers-not-asked (OCCURRENCE 1): the words that render the repeated wording lie in no '
                  + 'single block of the English, so no paragraph can be rewritten',
              ],
            },);
            // The reader refuses a section whose overlapped place names its own group, so reading it back is the proof.
            expect(parseConsistency({
              value: structuredClone(run.consistency,),
              path: 'cat.consistency',
              generation: 17,
              sliceIndexes: new Set([0, 1,],),
            },),).toEqual(run.consistency,);
          },
        },),
        it({
          name: 'ASKS THE WRITERS FOR THE SECOND PARAGRAPH WHEN THE PLACE THAT CROSSES INTO IT CANNOT BE SPLICED, the reviewer\'s '
            + 'case: a mark after the crossing words that the chosen place does not hold leaves that place no edit, so '
            + 'nothing addresses its words and the paragraph is rewritten for the place beside it, and the section is one its '
            + 'own reader accepts',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const run = await runCatPass({
              behave: crossingBallots,
              others: crossingWritersAndBench,
              entryCacheDir: dir.path,
              prepared: crossingPage({ afterFirst: '‘', },),
            },);
            expect({
              outcomes: outcomesOf({ consistency: run.consistency, group: 0, },),
              changed: changedTexts({ consistency: run.consistency, },),
              kinds: nonNullishOrThrow(run.consistency.groups[0],).carries?.map(function kindsOf({ candidates, },): unknown {
                return candidateKindsOf({ candidates, },);
              },),
              notAsked: run.consistency.findings.filter(function isNotAsked(finding,): boolean {
                return finding.includes('consistency-writers-not-asked',);
              },),
            },).toEqual({
              outcomes: ['kept-current', 'replaced', 'chosen',],
              changed: [
                [
                  0,
                  `## One\n\n${CROSSING_WORDS[0]}‘.\nThe kitchen felt warm.\n${CROSSING_WORDS[2]}.`,
                ],
              ],
              kinds: [['current',], ['current', 'splice', 'writer',],],
              notAsked: [
                'group 1: consistency-writers-not-asked (OCCURRENCE 1): the words that render the repeated wording lie in no '
                  + 'single block of the English, so no paragraph can be rewritten',
              ],
            },);
            expect(parseConsistency({
              value: structuredClone(run.consistency,),
              path: 'cat.consistency',
              generation: 17,
              sliceIndexes: new Set([0, 1,],),
            },),).toEqual(run.consistency,);
          },
        },),
        it({
          name: 'RECORDS THE MARK BESIDE THE WORDS OF TWO PLACES WHOSE WORDS ARE EQUAL AND WHOSE MARKS DIFFER, so that the section '
            + 'the pass writes is one its own reader accepts',
          fn: async () => {
            /**
             The pass over the page with every judge choosing the place whose words nothing follows.
             */
            const { consistency, } = await markedRun({ choice: 'OCCURRENCE 3', },);
            // The reader refuses a section whose recorded places contradict its own planner, so reading it back is the proof.
            expect(parseConsistency({
              value: structuredClone(consistency,),
              path: 'cat.consistency',
              generation: 17,
              sliceIndexes: new Set([0, 1,],),
            },),).toEqual(consistency,);
            // The first place of the first section holds other words and is replaced; the second holds the mark the
            // chosen place does not, and keeps its words in the same block.
            expect({
              outcomes: outcomesOf({ consistency, group: 0, },),
              marks: marksRecordedBy({ consistency, },),
            },).toEqual({
              outcomes: ['replaced', 'kept-current', 'chosen',],
              marks: [['none', 'none',], ['none', '‘',], ['none', 'none',],],
            },);
          },
        },),
        it({
          name: 'RECORDS THE MARKS OF THE PLACES THAT RECEIVE WORDS OF ANOTHER WORDING, one holding the mark the other does '
            + 'not, and the section is one its own reader accepts',
          fn: async () => {
            /**
             The pass over the page with every judge choosing the place of other words.
             */
            const { consistency, } = await markedRun({ choice: 'OCCURRENCE 1', },);
            expect({
              outcomes: outcomesOf({ consistency, group: 0, },),
              marks: marksRecordedBy({ consistency, },),
            },).toEqual({
              outcomes: ['chosen', 'kept-current', 'replaced',],
              marks: [['none', 'none',], ['none', '‘',], ['none', 'none',],],
            },);
            expect(parseConsistency({
              value: structuredClone(consistency,),
              path: 'cat.consistency',
              generation: 17,
              sliceIndexes: new Set([0, 1,],),
            },),).toEqual(consistency,);
          },
        },),
        it({
          name: 'CARRIES A PROVEN APOSTROPHE WITH THE WORDS INTO A PLACE THAT HOLDS NONE WHEN THE PAGE KEEPS IT ONE, a group '
            + 'whose places hold `parents’`, `folks’` and no mark: the recorded words hold the apostrophe and no mark beside '
            + 'them, the place that held none gains it, and the section the pass writes is one its generation seventeen '
            + 'reader accepts',
          fn: async () => {
            const { consistency, } = await markedRun({
              choice: 'OCCURRENCE 1',
              first: 'Fill the parents’ bowl. The kitchen was warm. Fill the folks’ bowl.',
              second: 'Then the family naps.',
              quotes: ['the parents', 'the folks', 'the family',],
            },);
            expect({
              outcomes: outcomesOf({ consistency, group: 0, },),
              changed: changedTexts({ consistency, },),
              quotes: nonNullishOrThrow(consistency.groups[0],).occurrences
                .map(function quoteOf({ rendering, },): string {
                  return (rendering.kind === 'located') ? rendering.quote : rendering.kind;
                },),
              marks: marksRecordedBy({ consistency, },),
            },).toEqual({
              outcomes: ['chosen', 'replaced', 'replaced',],
              changed: [
                [0, '## One\n\nFill the parents’ bowl. The kitchen was warm. Fill the parents’ bowl.',],
                [1, '## Two\n\nThen the parents’ naps.',],
              ],
              quotes: ['the parents’', 'the folks’', 'the family',],
              marks: [['none', 'none',], ['none', 'none',], ['none', 'none',],],
            },);
            expect(parseConsistency({
              value: structuredClone(consistency,),
              path: 'cat.consistency',
              generation: 17,
              sliceIndexes: new Set([0, 1,],),
            },),).toEqual(consistency,);
          },
        },),
        it({
          name: 'TAKES A PROVEN APOSTROPHE OUT WITH THE WORDS IT BELONGS TO, a group whose chosen place holds none: the places '
            + 'that held `parents’` and `folks’` read the chosen words and the section the pass writes is one its reader accepts',
          fn: async () => {
            const { consistency, } = await markedRun({
              choice: 'OCCURRENCE 3',
              first: 'Fill the parents’ bowl. The kitchen was warm. Fill the folks’ bowl.',
              second: 'Then the family naps.',
              quotes: ['the parents', 'the folks', 'the family',],
            },);
            expect({
              outcomes: outcomesOf({ consistency, group: 0, },),
              changed: changedTexts({ consistency, },),
            },).toEqual({
              outcomes: ['replaced', 'replaced', 'chosen',],
              changed: [[0, '## One\n\nFill the family bowl. The kitchen was warm. Fill the family bowl.',],],
            },);
            expect(parseConsistency({
              value: structuredClone(consistency,),
              path: 'cat.consistency',
              generation: 17,
              sliceIndexes: new Set([0, 1,],),
            },),).toEqual(consistency,);
          },
        },),
        it({
          name: 'LOCATES A PLACE WHOSE WORDS STAND TWICE IN ITS SLICE BY THE MARK THE JUDGES COPIED BESIDE THEM, the plural '
            + 'possessive of the review: the place is located once and its words hold the proven apostrophe, where the bare '
            + 'words were ambiguous',
          fn: async () => {
            const { consistency, } = await markedRun({
              choice: 'OCCURRENCE 1',
              first: 'The sunny window put the kitten to sleep. The kitchen was warm. Fill the cats\' bowl. Then the cats nap.',
              second: 'Then the kitten naps.',
              quotes: ['The sunny window put the kitten to sleep', 'the cats\'', 'the kitten naps',],
            },);
            expect({
              outcomes: outcomesOf({ consistency, group: 0, },),
              placed: nonNullishOrThrow(consistency.groups[0],).occurrences
                .map(function renderingOf({ rendering, },): unknown {
                  return rendering;
                },)
                .slice(1, 2,),
            },).toEqual({
              outcomes: ['chosen', 'replaced', 'replaced',],
              placed: [{ kind: 'located', quote: 'the cats\'', start: 77, },],
            },);
            expect(parseConsistency({
              value: structuredClone(consistency,),
              path: 'cat.consistency',
              generation: 17,
              sliceIndexes: new Set([0, 1,],),
            },),).toEqual(consistency,);
          },
        },),
        it({
          name: 'LOCATES NO PLACE WHOSE WORDS END IN EMPHASIS BEFORE A POSSESSIVE `\'s`, the reviewer\'s case: the apostrophe '
            + 'after markup joins the letter after it, so the quoted words stand inside a word and the page never reads '
            + '`cats\'s`; the place keeps its words, the place beside it that a splice moves cleanly is carried, and the '
            + 'section is one its own reader accepts',
          fn: async () => {
            const { consistency, } = await markedRun({
              choice: 'OCCURRENCE 1',
              first: 'Fill the cats\' bowl. The kitchen was warm. Then the dogs\' nap.',
              second: 'Fill the **cat**\'s bowl.',
              quotes: ['the cats', 'the dogs', 'the **cat**',],
            },);
            expect({
              outcomes: outcomesOf({ consistency, group: 0, },),
              changed: changedTexts({ consistency, },),
            },).toEqual({
              outcomes: ['chosen', 'replaced', 'unlocated',],
              changed: [[0, '## One\n\nFill the cats\' bowl. The kitchen was warm. Then the cats\' nap.',],],
            },);
            expect(parseConsistency({
              value: structuredClone(consistency,),
              path: 'cat.consistency',
              generation: 17,
              sliceIndexes: new Set([0, 1,],),
            },),).toEqual(consistency,);
          },
        },),
        it({
          name: 'CARRIES A PLURAL POSSESSIVE ON A PAGE WHOSE EARLIER `**cat**\'s` IS PART OF A WORD: the mark after emphasis joins '
            + 'the letter after it and is no possible opener, so the `\'` after `dogs` in a later section is proven, and the '
            + 'place is carried whichever of the two stands first; the section is one its own reader accepts',
          fn: async () => {
            const { consistency, } = await markedRun({
              choice: 'OCCURRENCE 1',
              first: 'Fill the cats\' bowl. The kitchen was warm. Fill the **cat**\'s bowl.',
              second: 'Then the dogs\' nap.',
              quotes: ['the cats', 'the **cat**', 'the dogs',],
            },);
            expect({
              outcomes: outcomesOf({ consistency, group: 0, },),
              changed: changedTexts({ consistency, },),
            },).toEqual({
              outcomes: ['chosen', 'unlocated', 'replaced',],
              changed: [[1, '## Two\n\nThen the cats\' nap.',],],
            },);
            expect(parseConsistency({
              value: structuredClone(consistency,),
              path: 'cat.consistency',
              generation: 17,
              sliceIndexes: new Set([0, 1,],),
            },),).toEqual(consistency,);
          },
        },),
        it({
          name: 'CARRIES STRAIGHT QUOTATION MARKS ONTO A CURLY PLACE AS CURLY ONES AND CURLY ONTO STRAIGHT, each loose mark '
            + 'in the form the receiving place wrote it, and the section the pass writes is one its own reader accepts',
          fn: async () => {
            const { consistency, } = await markedRun({
              choice: 'OCCURRENCE 1',
              first: 'Then "She was kind," she said. The kitchen was warm. Later “He was merry,” he said.',
              second: 'Next "He was merry," he said.',
              quotes: ['"She was kind," she said', '“He was merry,” he said', '"He was merry," he said',],
            },);
            expect({
              outcomes: outcomesOf({ consistency, group: 0, },),
              changed: changedTexts({ consistency, },),
            },).toEqual({
              outcomes: ['chosen', 'replaced', 'replaced',],
              changed: [
                [0, '## One\n\nThen "She was kind," she said. The kitchen was warm. Later “She was kind,” she said.',],
                [1, '## Two\n\nNext "She was kind," she said.',],
              ],
            },);
            expect(parseConsistency({
              value: structuredClone(consistency,),
              path: 'cat.consistency',
              generation: 17,
              sliceIndexes: new Set([0, 1,],),
            },),).toEqual(consistency,);
          },
        },),
        it({
          name: 'CARRIES THE PLACE OF A TABLE THAT CODE CAN MOVE THE WORDS INTO AND KEEPS THE ONE IT CANNOT, the reviewer\'s '
            + 'table: the second cell is replaced, the first is kept-current, the paragraph beside it is carried, and the '
            + 'section is one its own reader accepts',
          fn: async () => {
            const { run, lines, } = await partlyCarriedRun({
              english: '| h | h |\n|---|---|\n| Fill the cats‘ bowl | Then the dogs nap |\n\nLater the birds sing.',
              quotes: ['the cats', 'the dogs', 'the birds',],
            },);
            expect(partlySeenIn({ run, },),).toEqual({
              outcomes: ['kept-current', 'replaced', 'replaced', 'chosen',],
              firstSection: '## One\n\n| h | h |\n|---|---|\n| Fill the cats‘ bowl | Then the kittens purr nap |\n\n'
                + 'Later the kittens purr sing.',
              kinds: [['current', 'splice',], ['current', 'splice',], ['current', 'splice',],],
              partly: [
                'group 1: consistency-partly-carried (OCCURRENCE 1 and OCCURRENCE 2): the splice won, so code moved the '
                  + 'chosen words into OCCURRENCE 2 and left OCCURRENCE 1 as it was, since code cannot move them there',
              ],
            },);
            expect(lines.some(stopsTheEntry,),).toBe(false,);
            expectReadsBack({ consistency: run.consistency, },);
          },
        },),
        it({
          name: 'CARRIES THE PLACES OF A BLOCK THAT CODE CAN MOVE THE WORDS INTO WHETHER THE PLACE IT CANNOT MOVE THEM INTO '
            + 'COMES FIRST, IN THE MIDDLE OR LAST, each carried place replaced, that place kept-current, and every section '
            + 'one its own reader accepts',
          fn: async () => {
            /**
             The pages: the place that holds the apostrophe stands first, second and last.
             */
            const shapes = [
              {
                english: 'Fill the cats‘ bowl. Then the dogs nap. Next the birds sing.',
                quotes: ['the cats', 'the dogs', 'the birds',],
                outcomes: ['kept-current', 'replaced', 'replaced', 'chosen',],
                text: 'Fill the cats‘ bowl. Then the kittens purr nap. Next the kittens purr sing.',
              },
              {
                english: 'Then the dogs nap. Fill the cats‘ bowl. Next the birds sing.',
                quotes: ['the dogs', 'the cats', 'the birds',],
                outcomes: ['replaced', 'kept-current', 'replaced', 'chosen',],
                text: 'Then the kittens purr nap. Fill the cats‘ bowl. Next the kittens purr sing.',
              },
              {
                english: 'Then the dogs nap. Next the birds sing. Fill the cats‘ bowl.',
                quotes: ['the dogs', 'the birds', 'the cats',],
                outcomes: ['replaced', 'replaced', 'kept-current', 'chosen',],
                text: 'Then the kittens purr nap. Next the kittens purr sing. Fill the cats‘ bowl.',
              },
            ] as const;
            const seen = await Promise.all(shapes.map(async function runShape({ english, quotes, },) {
              const { run, } = await partlyCarriedRun({ english, quotes, },);
              expectReadsBack({ consistency: run.consistency, },);
              return partlySeenIn({ run, },);
            },),);
            expect(seen.map(function outcomesOfShape({ outcomes, },): readonly string[] {
              return outcomes;
            },),).toEqual(shapes.map(function expectedOutcomes({ outcomes, },): readonly string[] {
              return outcomes;
            },),);
            expect(seen.map(function textOfShape({ firstSection, },): string {
              return firstSection;
            },),).toEqual(shapes.map(function expectedText({ text, },): string {
              return `## One\n\n${text}`;
            },),);
            expect(seen.map(function kindsOfShape({ kinds, },): unknown {
              return kinds;
            },),).toEqual(shapes.map(function expectedKinds(): unknown {
              return [['current', 'splice',], ['current', 'splice',], ['current', 'splice',],];
            },),);
          },
        },),
        it({
          name: 'CARRIES EVERY PLACE OF A BLOCK BY THE SPLICE AND REPORTS NO PARTLY CARRIED UNIT WHEN CODE CAN MOVE THE WORDS '
            + 'INTO ALL OF THEM, the control: nothing about it changes',
          fn: async () => {
            const { run, } = await partlyCarriedRun({
              english: 'Fill the cats bowl. Then the dogs nap. Next the birds sing.',
              quotes: ['the cats', 'the dogs', 'the birds',],
            },);
            expect(partlySeenIn({ run, },),).toEqual({
              outcomes: ['replaced', 'replaced', 'replaced', 'chosen',],
              firstSection: '## One\n\nFill the kittens purr bowl. Then the kittens purr nap. Next the kittens purr sing.',
              kinds: [['current', 'splice',], ['current', 'splice',], ['current', 'splice',],],
              partly: [],
            },);
            expectReadsBack({ consistency: run.consistency, },);
          },
        },),
        it({
          name: 'OFFERS NO SPLICE FOR A BLOCK WHOSE EVERY RECEIVING PLACE IS ONE CODE CANNOT MOVE THE WORDS INTO, though the '
            + 'block beside it is carried: the places keep their words and the bench is not asked',
          fn: async () => {
            const { run, } = await partlyCarriedRun({
              english: 'Fill the cats‘ bowl. Then the dogs‘ nap.\n\nNext the birds sing.',
              quotes: ['the cats', 'the dogs', 'the birds',],
            },);
            expect(partlySeenIn({ run, },),).toEqual({
              outcomes: ['kept-current', 'kept-current', 'replaced', 'chosen',],
              firstSection: '## One\n\nFill the cats‘ bowl. Then the dogs‘ nap.\n\nNext the kittens purr sing.',
              kinds: [['current',], ['current',], ['current', 'splice',],],
              partly: [],
            },);
            expectReadsBack({ consistency: run.consistency, },);
          },
        },),
        it({
          name: 'KEEPS A SPLICE FROM MEETING A WRITER\'S PARAGRAPH OF THE SAME GROUP, the reviewer\'s question: a place that '
            + 'runs over a paragraph boundary and cannot be spliced no longer keeps the paragraph it runs into from the '
            + 'writers, and neither the splice nor the writer\'s paragraph of the partly carried block beside it meets the '
            + 'other, so the invariant of the pass does not fire',
          fn: async () => {
            /**
             The first section: a place that runs over a paragraph boundary and is followed by an apostrophe, and a paragraph
             that holds a place with an apostrophe after it and a place without.
             */
            const english = 'The sunny window put the kitten\n\nto sleep‘. The kitchen was warm. Fill the cats‘ bowl. '
              + 'Then the dogs nap.';
            const quotes = ['The sunny window put the kitten\n\nto sleep', 'the cats', 'the dogs',];
            const spliced = await partlyCarriedRun({ english, quotes, },);
            const written = await partlyCarriedRun({ english, quotes, others: partlyWriters, },);
            expect([spliced, written,].map(function seenOf({ run, },): unknown {
              return {
                seen: partlySeenIn({ run, },),
                writerSheets: run.sheets.some(function isWriterSheet(sheet,): boolean {
                  return sheetKindOf({ sheet, },) === 'writer';
                },),
              };
            },),).toEqual([
              {
                seen: {
                  outcomes: ['kept-current', 'kept-current', 'replaced', 'chosen',],
                  firstSection: '## One\n\nThe sunny window put the kitten\n\nto sleep‘. The kitchen was warm. Fill the cats‘ '
                    + 'bowl. Then the kittens purr nap.',
                  kinds: [['current',], ['current', 'splice',], ['current', 'splice',],],
                  partly: [
                    'group 1: consistency-partly-carried (OCCURRENCE 2 and OCCURRENCE 3): the splice won, so code moved the '
                      + 'chosen words into OCCURRENCE 3 and left OCCURRENCE 2 as it was, since code cannot move them there',
                  ],
                },
                writerSheets: true,
              },
              {
                seen: {
                  outcomes: ['kept-current', 'replaced', 'replaced', 'chosen',],
                  firstSection: '## One\n\nThe sunny window put the kitten\n\nto sleep‘.\nThe kitchen felt warm.\nFill the '
                    + 'kittens purr‘ bowl.\nThen the kittens purr nap.',
                  kinds: [['current',], ['current', 'splice', 'writer',], ['current', 'splice', 'writer',],],
                  partly: [],
                },
                writerSheets: true,
              },
            ],);
            [spliced, written,].forEach(function readsBack({ run, lines, },): void {
              expectReadsBack({ consistency: run.consistency, },);
              expect(lines.some(stopsTheEntry,),).toBe(false,);
            },);
          },
        },),
        it({
          name: 'WITHHOLDS A WRITER\'S PARAGRAPH THAT CHANGES THE WORDS OF A PLACE THAT RUNS INTO THE BLOCK AND CANNOT BE '
            + 'SPLICED, at the start of the block where the place crosses in and at its end where it crosses out, and '
            + 'keeps one that leaves them where they stand: the block with the withheld paragraphs keeps the outcomes the '
            + 'splice gives it',
          fn: async () => {
            /**
             The page where the first place crosses into the block of the other two, and the one where it crosses out.
             */
            const crossingIn = {
              english: 'The sunny window put the kitten\n\nto sleep‘. The kitchen was warm. Fill the cats‘ bowl. Then the dogs nap.',
              quotes: ['The sunny window put the kitten\n\nto sleep', 'the cats', 'the dogs',],
            };
            const crossingOut = {
              english: 'Fill the cats‘ bowl. Then the dogs nap. The sunny window put the kitten\n\nto sleep‘.',
              quotes: ['the cats', 'the dogs', 'The sunny window put the kitten\n\nto sleep',],
            };
            const seen = await Promise.all([
              crossingSeen({ ...crossingIn, change: ['to sleep', 'to doze',], },),
              crossingSeen({ ...crossingIn, },),
              crossingSeen({ ...crossingOut, change: ['put the kitten', 'sent the kitten',], },),
              crossingSeen({ ...crossingOut, },),
            ],);
            expect(seen.map(function withheldOf({ withheld, },): number {
              return withheld;
            },),).toEqual([2, 0, 2, 0,],);
            expect(seen.map(function kindsOf({ kinds, },): unknown {
              return kinds;
            },),).toEqual([
              [['current',], ['current', 'splice',], ['current', 'splice',],],
              [['current',], ['current', 'splice', 'writer',], ['current', 'splice', 'writer',],],
              [['current', 'splice',], ['current', 'splice',], ['current',],],
              [['current', 'splice', 'writer',], ['current', 'splice', 'writer',], ['current',],],
            ],);
            // The outcomes of the pages whose writers' paragraphs were withheld are the splice's, as the page with no
            // writers gives them.
            expect([0, 2,].map(function outcomesOfSeen(at,): readonly string[] {
              return nonNullishOrThrow(seen[at],).outcomes;
            },),).toEqual([
              ['kept-current', 'kept-current', 'replaced', 'chosen',],
              ['kept-current', 'replaced', 'kept-current', 'chosen',],
            ],);
          },
        },),
        it({
          name: 'GOES ON WITHOUT THE WRITERS WHEN THE SEAT READING FINDS THE WRITING BENCH UNREACHABLE: one error line names '
            + 'the bench and none says the entry stops, no writer is asked, the bench chooses between the English as it '
            + 'stands and the chosen words moved in by code, and the section records a finding for each receiving place',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-', },);
            const lines: string[] = [];
            const run = await runCatPass({
              behave: firstPlaceWins,
              entryCacheDir: dir.path,
              dryness: {
                neuralwatt: true,
                synthetic: true,
                bedrock: true,
                deepseek: true,
                xiaomi: true,
                hyper: true,
                openrouter: true,
              },
              lines,
            },);
            const why = 'the translators bench was not asked: translators 0 of 8 reachable, floor 2; '
              + 'no provider has named a return the pass can wait for';
            expect({
              errors: lines.filter(function isError(line,): boolean {
                return line.startsWith('error ',);
              },),
              stops: lines.filter(function saysStop(line,): boolean {
                return line.includes('stopping the entry',);
              },),
              writerSheets: run.sheets.filter(function isWriter(sheet,): boolean {
                return sheetKindOf({ sheet, },) === 'writer';
              },).length,
              findings: run.consistency.findings,
              kinds: run.consistency.groups.map(function kindsOf(group,): unknown {
                return group.carries?.map(function carryKinds({ candidates, },): unknown {
                  return candidateKindsOf({ candidates, },);
                },);
              },),
              outcomes: run.consistency.groups.map(function outcomesOfGroup(group,): unknown {
                return group.occurrences.map(function outcomeOf({ outcome, },): string {
                  return outcome;
                },);
              },),
              changed: changedTexts({ consistency: run.consistency, },),
            },).toEqual({
              errors: [
                'error [runPassConsistency] JUDGE SEATS phase=page consistency writing bench unreachable: '
                  + 'translators 0 of 8 reachable, floor 2; no provider has named a return the pass can wait for, so '
                  + 'the phase goes on without its writers',
              ],
              stops: [],
              writerSheets: 0,
              findings: [
                `group 1: consistency-writers-not-asked (OCCURRENCE 2): ${why}`,
                `group 2: consistency-writers-not-asked (OCCURRENCE 2): ${why}`,
                `group 2: consistency-writers-not-asked (OCCURRENCE 3): ${why}`,
                `group 2: consistency-writers-not-asked (OCCURRENCE 4): ${why}`,
              ],
              kinds: [[['current', 'splice',],], [['current', 'splice',], ['current', 'splice',], ['current', 'splice',],],],
              outcomes: [['chosen', 'replaced',], ['chosen', 'replaced', 'replaced', 'replaced',],],
              changed: [[1, SLICE_ONE_AFTER,], [2, SLICE_TWO_AFTER,], [3, SLICE_THREE_AFTER,],],
            },);
          },
        },),
      ],
    },),
  ],
},);
