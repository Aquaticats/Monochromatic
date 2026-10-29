/**
 Runs the page-level consistency pass over a cat page of four places and builds the page the publishing ships, for the cases that
 read what a reader of the shipped page would see: the pass's rows, the page-assembly guard over them, `shippableReplacements` and
 `spliceSlices`.

 The page: the first section holds three places written by a template, the second holds the fourth. Every judge quotes all four
 places and chooses the first, so the first place's words are the chosen ones and the others are the receiving places. The
 writers write nothing, so only a code splice can change a place.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { spliceSlices, } from '@monochromatic-dev/translation-repair-page-pass';
import { RUN_LATE_JUDGES, } from '@monochromatic-dev/translation-repair-run';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { NO_PAGE_ASSEMBLY, } from '@monochromatic-dev/translation-repair-artifact';
import {
  guardPageAssembly,
  shippableReplacements,
  type WouldShipSource,
} from '@monochromatic-dev/translation-repair-page-check';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  ballotText,
  otherSheetBehaviour,
  projectedCatPage,
  type SeatBehaviour,
  type SeatScript,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';
import { runCatPass, } from './consistency-run.test-fixture.ts';
import { sheetKindOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-sheet-reading.test-fixture.ts';

//region Consistency shipped page fixture

/**
 The repeated sentence of the source, with the sentence about the kitchen that follows it.
 */
const SENTENCE = '窗台上的阳光让小猫睡着了。厨房里很暖和。';

/**
 How many times the first section of the source repeats the sentence, once for each place the template writes.
 */
const SENTENCE_REPEATS = 3;

/**
 The digits that name the places a template writes, in the order of the places.
 */
const PLACE_DIGITS: readonly string[] = [
  '1',
  '2',
  '3',
];

/**
 How many places the judges quote: the three of the template and the fourth of the second section.
 */
const QUOTED_PLACES = 4;

/**
 The English of a template and the words each marked place quotes.
 */
type ParsedTemplate = {
  /**
   The English with the markers taken out.
   */
  readonly english: string;

  /**
   The words of places one, two and three, in that order.
   */
  readonly quotes: readonly string[];
};

/**
 Reads a template whose places are written between `⟦1` and `1⟧`, `⟦2` and `2⟧`, `⟦3` and `3⟧`.

 @param template - English of the first section with the markers

 @returns English and words each place quotes

 @example
 ```ts
 const { english, quotes, } = parsedTemplate({ template: 'The ⟦1dogs’1⟧ bowl.', },); // quotes ['dogs’']
 ```
 */
function parsedTemplate({ template, }: { readonly template: string; },): ParsedTemplate {
  /**
   The units of the template as whole characters.
   */
  const points = Array.from(template,);
  /**
   The pieces of English kept, mutated only inside this function.
   */
  const pieces: string[] = [];
  /**
   Where each open place started, mutated only inside this function.
   */
  const started = new Map<number, number>();
  /**
   The words of each place, by its number, mutated only inside this function.
   */
  const words = new Map<number, string>();
  /**
   How long the English is so far.
   */
  const length = { so: 0, };
  for (let index = 0; index < points.length; index += 1) {
    /**
     The character under the cursor.
     */
    const point = points[index] ?? '';
    if (point === '⟦') {
      started.set(
        Number(points[index + 1],),
        length.so,
      );
      index += 1;
      continue;
    }
    if (point === '⟧')
      continue;
    if (PLACE_DIGITS.includes(point,) && (points[index + 1] === '⟧')) {
      /**
       The English kept so far, which the place ends in.
       */
      const sofar = pieces.join('',);
      words.set(
        Number(point,),
        sofar.slice(
          started.get(Number(point,),) ?? 0,
          length.so,
        ),
      );
      continue;
    }
    pieces.push(point,);
    length.so += point.length;
  }
  return {
    english: pieces.join('',),
    quotes: PLACE_DIGITS.map(function wordsOf(digit,): string {
      return words.get(Number(digit,),) ?? '';
    },),
  };
}

/**
 What a scenario came to.
 */
type ShippedPage = {
  /**
   The page before the pass, as the archive wrote it.
   */
  readonly before: string;

  /**
   The page the publishing ships.
   */
  readonly shipped: string;

  /**
   The outcome of each place of the first group, in the order of the places.
   */
  readonly outcomes: readonly string[];
};

/**
 The ballots of the page: every judge quotes the four places and chooses the first, and the late judges are silent.

 @param quotes - what every judge quotes for the four places

 @returns What each seat does when asked

 @example
 ```ts
 const behave = ballotsOf({ quotes: ['dogs’', 'kittens', 'cat', 'kittens',], },);
 ```
 */
function ballotsOf({ quotes, }: { readonly quotes: readonly string[]; },): SeatScript {
  /**
   What every judge quotes, under the labels of the places.
   */
  const labelled = quotes.map(function labelOne(
    quote,
    at,
  ): Record<string, string> {
    return {
      label: `OCCURRENCE ${String(at + 1,)}`,
      quote,
    };
  },);
  /**
   The seats that are silent, the late ones past the fourth.
   */
  const silent = new Set(RUN_LATE_JUDGES.slice(QUOTED_PLACES,),);
  return function judgeSpeaks({ seat, },): SeatBehaviour {
    return silent.has(seat,)
      ? { kind: 'throws', }
      : {
        kind: 'answers',
        text: ballotText({
          quotes: labelled,
          choice: 'OCCURRENCE 1',
          keep: [],
        },),
      };
  };
}

/**
 The English of each slice of a preparation, in the order of the slices.

 @param slices - slices of the preparation

 @returns Target English of each slice

 @example
 ```ts
 const english = targetEnglishOf({ slices: prepared.slices, },);
 ```
 */
function targetEnglishOf(
  { slices, }: { readonly slices: ReturnType<typeof prepareDocumentPair>['slices']; },
): readonly string[] {
  return slices.map(function textOf({ target, },): string {
    return target.text;
  },);
}

/**
 What a seat does with a sheet that is not a first-round ballot when the writers are silent: a writer throws and every other
 sheet is answered as the fixture of the pass answers it.

 @param sheet - sheet the seat was shown

 @returns Behaviour of the seat

 @example
 ```ts
 const behaviour = writersSilent({ sheet, },);
 ```
 */
function writersSilent({ sheet, }: { readonly sheet: string; },): SeatBehaviour {
  return (sheetKindOf({ sheet, },) === 'writer')
    ? { kind: 'throws', }
    : otherSheetBehaviour({ sheet, },);
}

/**
 Runs the pass over a page and builds the page that ships.

 @param template - English of the first section, with the markers of places one to three

 @param secondSection - English of the second section, which holds the fourth place's words 'kittens'

 @param silentWriters - whether the writers throw, for the pages whose space the writer sheets show changed
 (`invisible-variants.ts` maps U+202F to a space and drops U+FEFF), where a writer that returns the paragraph as it stands
 would return a different paragraph

 @returns Page before, page shipped and outcomes of the first group

 @example
 ```ts
 const page = await shippedPageOf({ template: 'The ⟦1dogs’1⟧ bowl. The ⟦2kittens2⟧ bowl. A ⟦3cat3⟧ naps.', },);
 ```
 */
export async function shippedPageOf(
  {
    template,
    secondSection = 'The kittens nap.',
    silentWriters = false,
  }: {
    readonly template: string;
    readonly secondSection?: string;
    readonly silentWriters?: boolean;
  },
): Promise<ShippedPage> {
  /**
   The English and the words of the first three places.
   */
  const {
    english,
    quotes,
  } = parsedTemplate({ template, },);
  /**
   The preparation of the page.
   */
  const prepared = prepareDocumentPair({
    sourceText: `## 一\n\n${SENTENCE.repeat(SENTENCE_REPEATS,)}\n\n## 二\n\n窗台上的阳光让小猫睡着了。\n`,
    targetText: `## One\n\n${english}\n\n## Two\n\n${secondSection}\n`,
  },);
  /**
   The scratch directory of the entry cache, removed when the function ends.
   */
  await using dir = await scratchDir({ prefix: 'pass-consistency-shipped-page-', },);
  /**
   The pass over the page.
   */
  const run = await runCatPass({
    behave: ballotsOf({
      quotes: [
        ...quotes,
        'kittens',
      ],
    },),
    others: silentWriters ? writersSilent : otherSheetBehaviour,
    entryCacheDir: dir.path,
    prepared,
  },);
  /**
   What the pass made of the page.
   */
  const { consistency, } = run;
  /**
   The projection of the page before the pass.
   */
  const projected = projectedCatPage({
    prepared,
    english: targetEnglishOf({ slices: prepared.slices, },),
  },);
  /**
   The artifact as the stages compose it, before the guard.
   */
  const composed: WouldShipSource = {
    comparison: projected.comparison,
    laneSelection: {
      kind: 'contested',
      slices: [],
    },
    consolidation: {
      kind: 'settled',
      slices: [],
    },
    preparation: { archiveText: prepared.targetText, },
    pageAssembly: NO_PAGE_ASSEMBLY,
    consistency: {
      groups: [],
      slices: consistency.slices,
      findings: [],
    },
  };
  /**
   What the guard made of that page.
   */
  const pageAssembly = guardPageAssembly({
    artifact: composed,
    slices: prepared.slices,
    sourceText: prepared.sourceText,
    targetText: prepared.targetText,
    archiveOriginalSpans: prepared.archiveOriginalSpans ?? [],
  },);
  /**
   The first group of the pass, which holds the places of the template.
   */
  const { occurrences, } = nonNullishOrThrow(consistency.groups[0],);
  /**
   The outcome of each place of that group.
   */
  const outcomes = occurrences.map(function outcomeOf({ outcome, },): string {
    return outcome;
  },);
  return {
    before: prepared.targetText,
    shipped: spliceSlices({
      targetText: prepared.targetText,
      slices: prepared.slices,
      replacements: shippableReplacements({
        artifact: {
          ...composed,
          pageAssembly,
        },
      },),
    },),
    outcomes,
  };
}

//endregion Consistency shipped page fixture
