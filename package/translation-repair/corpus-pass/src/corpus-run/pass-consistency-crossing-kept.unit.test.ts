/**
 Tests for the words of a place that must stay as they are and run over the edge of the block a writer rewrites: a
 writer's paragraph that changes the stretch of such a place's words inside the block is withheld whatever kind of
 place it is (a receiving place with no splice, the place a majority keeps for its context, the chosen place), the
 sheet the writers see lists the stretch, and a paragraph that leaves the stretch where it stands is kept.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

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
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  type ArtifactConsistency,
  parseConsistency,
} from '@monochromatic-dev/translation-repair-artifact';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  ballotText,
  choosing,
  otherSheetBehaviour,
  type SeatBehaviour,
  type SeatScript,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';
import {
  candidateTextsOf,
  currentParagraphOf,
  sheetKindOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-sheet-reading.test-fixture.ts';
import { runCatPass, } from './consistency-run.test-fixture.ts';

/**
 The repeated sentence of the source and the sentence about the kitchen between its repeats.
 */
const SENTENCE = '窗台上的阳光让小猫睡着了。';

/**
 The sentence about the kitchen.
 */
const WARM = '厨房里很暖和。';

/**
 The source of the page: the repeated sentence three times in the first section and once in the second.
 */
const SOURCE = `## 一\n\n${SENTENCE}${WARM}${SENTENCE}${WARM}${SENTENCE}\n\n## 二\n\n${SENTENCE}\n`;

/**
 The words of the first place, which end the first paragraph and run on into the second.
 */
const CROSSING_WORDS = 'The sunny window put the kitten\n\nto sleep';

/**
 The words of the second place, inside the second paragraph.
 */
const SECOND_WORDS = 'The sun on the sill sent the kitten off to sleep';

/**
 The words of the third place, which is the third paragraph.
 */
const THIRD_WORDS = 'The window sun made the kitten';

/**
 The words of the place that closes the second section.
 */
const FOURTH_WORDS = 'The window sun made the kitten fall asleep';

/**
 The first section's English where the first place is followed by an opening curly single mark the others do not hold,
 which no proven apostrophe makes of it, so the chosen words cannot be moved into it by code.
 */
const MARKED_ENGLISH = `${CROSSING_WORDS}‘ said she. The kitchen was warm. ${SECOND_WORDS}.\n\n${THIRD_WORDS}.`;

/**
 The first section's English where the first place is followed by nothing the others do not hold.
 */
const PLAIN_ENGLISH = `${CROSSING_WORDS} said she. The kitchen was warm. ${SECOND_WORDS}.\n\n${THIRD_WORDS}.`;

/**
 What the writers do to the paragraph they rewrite and what the choice among candidates takes.
 */
type Writing = {
  /**
   The words of the paragraph the writers change, and what they write instead.
   */
  readonly change: readonly [string, string];
};

/**
 What the pass recorded of one run.
 */
type Seen = {
  /**
   Outcomes of the places of the first group, in order.
   */
  readonly outcomes: readonly string[];

  /**
   The first section's text as the pass left it.
   */
  readonly firstSection: string;

  /**
   How many writers' paragraphs were withheld for the words of a place that runs over an edge of the block.
   */
  readonly withheld: number;

  /**
   The writer sheets that list a stretch at the start of the paragraph in the words to keep.
   */
  readonly sheetsListingTheStart: readonly string[];
};

/**
 What the writers and the bench of candidates do: a writer returns the paragraph it was shown with some words changed,
 and the choice among candidates takes the first candidate that holds the change.
 */
function writersChanging({ change, }: Writing,): SeatScript {
  return function behaviourOf(
    {
      seat,
      sheet,
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
        text: JSON.stringify({ translation: currentParagraphOf({ sheet, },).replace(change[0], change[1],), },),
      };
    }
    if (kind === 'choice') {
      if (RUN_LATE_JUDGES.indexOf(seat,) > 3)
        return { kind: 'throws', };
      return choosing({
        choice: candidateTextsOf({ sheet, },)
          .find(function carriesTheChange({ text, },): boolean {
            return text.includes(change[1],);
          },)
          ?.label
          ?? candidateTextsOf({ sheet, },)
            .find(function isAlternative({ current, },): boolean {
              return !current;
            },)
            ?.label
          ?? 'CANDIDATE 1',
      },);
    }
    return otherSheetBehaviour({ sheet, },);
  };
}

/**
 Runs the pass over the page whose first section is the given English, every judge quoting the four places and
 choosing one, and reports what the pass recorded.

 @param english - English of the first section
 @param choice - place every judge says the page should follow, from one
 @param keep - labels of the places every judge keeps for their surroundings
 @param writing - what the writers change

 @returns What the pass recorded

 @example
 ```ts
 const seen = await seenOn({ english: MARKED_ENGLISH, choice: 4, keep: ['OCCURRENCE 1',], writing, },);
 ```
 */
async function seenOn(
  {
    english,
    choice,
    keep,
    writing,
  }: {
    readonly english: string;
    readonly choice: number;
    readonly keep: readonly string[];
    readonly writing: Writing;
  },
): Promise<Seen> {
  await using dir = await scratchDir({ prefix: 'pass-consistency-crossing-kept-', },);
  /**
   What the judges quote for the four places, in order.
   */
  const quotes = [CROSSING_WORDS, SECOND_WORDS, THIRD_WORDS, FOURTH_WORDS,];
  const run = await runCatPass({
    behave: function ballotsOf({ seat, },): SeatBehaviour {
      if (RUN_LATE_JUDGES.indexOf(seat,) > 3)
        return { kind: 'throws', };
      return {
        kind: 'answers',
        text: ballotText({
          quotes: quotes.map(function quoteOf(quote, at,): Record<string, string> {
            return { label: `OCCURRENCE ${String(at + 1,)}`, quote, };
          },),
          choice: `OCCURRENCE ${String(choice,)}`,
          keep,
        },),
      };
    },
    others: writersChanging(writing,),
    entryCacheDir: dir.path,
    prepared: prepareDocumentPair({
      sourceText: SOURCE,
      targetText: `## One\n\n${english}\n\n## Two\n\n${FOURTH_WORDS}.\n`,
    },),
  },);
  /**
   The section the pass wrote.
   */
  const consistency: ArtifactConsistency = run.consistency;
  // The section reads back as the one the pass wrote, as the reader of the artifact reads it.
  expect(parseConsistency({
    value: structuredClone(consistency,),
    path: 'cat.consistency',
    generation: 17,
    sliceIndexes: new Set([0, 1,],),
  },),).toEqual(consistency,);
  return {
    outcomes: consistency.groups[0]
      ?.occurrences
      .map(function outcomeOf({ outcome, },): string {
        return outcome;
      },)
      ?? [],
    firstSection: consistency.slices
      .find(function isFirst({ sliceIndex, },): boolean {
        return sliceIndex === 0;
      },)
      ?.text
      ?? '',
    withheld: consistency.findings
      .filter(function refusesAStretch(finding,): boolean {
        return finding.includes('consistency-candidate-refused',)
          && finding.includes('the words of a place that runs over it and must stay as they are',);
      },)
      .length,
    sheetsListingTheStart: run.sheets.filter(function listsTheStart(sheet,): boolean {
      return (sheetKindOf({ sheet, },) === 'writer') && sheet.includes('WORDS OF OTHER PLACES TO KEEP EXACTLY',)
        && sheet.includes('(these are the FIRST words of the paragraph',);
    },),
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'a place whose words run into the block and must stay as they are',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WITHHOLDS A WRITER\'S PARAGRAPH THAT CHANGES THE STRETCH OF A RECEIVING PLACE WITH NO SPLICE, the control that '
            + 'also holds before the places the majority keeps and the chosen place were read',
          fn: async () => {
            const seen = await seenOn({
              english: MARKED_ENGLISH,
              choice: 4,
              keep: [],
              writing: { change: ['to sleep', 'to nap',], },
            },);
            expect({
              outcomes: seen.outcomes,
              withheld: seen.withheld,
              tailKept: seen.firstSection.includes('to sleep‘ said she.',),
              tailChanged: seen.firstSection.includes('to nap',),
            },).toEqual({
              outcomes: ['kept-current', 'replaced', 'replaced', 'chosen',],
              withheld: 2,
              tailKept: true,
              tailChanged: false,
            },);
          },
        },),
        it({
          name: 'WITHHOLDS A WRITER\'S PARAGRAPH THAT CHANGES THE STRETCH OF A PLACE THE MAJORITY KEEPS FOR ITS CONTEXT, so '
            + 'that the page holds the words the record says the place kept, and keeps one that leaves the stretch where it stands',
          fn: async () => {
            const changed = await seenOn({
              english: MARKED_ENGLISH,
              choice: 4,
              keep: ['OCCURRENCE 1',],
              writing: { change: ['to sleep', 'to nap',], },
            },);
            const left = await seenOn({
              english: MARKED_ENGLISH,
              choice: 4,
              keep: ['OCCURRENCE 1',],
              writing: { change: ['was warm', 'felt warm',], },
            },);
            expect({
              changed: {
                outcome: changed.outcomes[0],
                withheld: changed.withheld,
                tailKept: changed.firstSection.includes('to sleep‘ said she.',),
                tailChanged: changed.firstSection.includes('to nap',),
                listedForTheWriters: changed.sheetsListingTheStart.length > 0,
              },
              left: {
                outcome: left.outcomes[0],
                withheld: left.withheld,
                tailKept: left.firstSection.includes('to sleep‘ said she.',),
                otherChangeKept: left.firstSection.includes('felt warm',),
              },
            },).toEqual({
              changed: {
                outcome: 'kept-context',
                withheld: 2,
                tailKept: true,
                tailChanged: false,
                listedForTheWriters: true,
              },
              left: {
                outcome: 'kept-context',
                withheld: 0,
                tailKept: true,
                otherChangeKept: true,
              },
            },);
          },
        },),
        it({
          name: 'WITHHOLDS A WRITER\'S PARAGRAPH THAT CHANGES THE STRETCH OF THE CHOSEN PLACE, so that the page holds the words '
            + 'the record says the chosen place kept, and keeps one that leaves the stretch where it stands',
          fn: async () => {
            const changed = await seenOn({
              english: PLAIN_ENGLISH,
              choice: 1,
              keep: [],
              writing: { change: ['to sleep', 'to nap',], },
            },);
            const left = await seenOn({
              english: PLAIN_ENGLISH,
              choice: 1,
              keep: [],
              writing: { change: ['was warm', 'felt warm',], },
            },);
            expect({
              changed: {
                outcome: changed.outcomes[0],
                withheld: changed.withheld,
                tailChanged: changed.firstSection.includes('to nap',),
                listedForTheWriters: changed.sheetsListingTheStart.length > 0,
              },
              left: {
                outcome: left.outcomes[0],
                withheld: left.withheld,
                tailKept: left.firstSection.includes('to sleep said she.',),
                otherChangeKept: left.firstSection.includes('felt warm',),
              },
            },).toEqual({
              changed: {
                outcome: 'chosen',
                withheld: 2,
                tailChanged: false,
                listedForTheWriters: true,
              },
              left: {
                outcome: 'chosen',
                withheld: 0,
                tailKept: true,
                otherChangeKept: true,
              },
            },);
          },
        },),
      ],
    },),
  ],
},);
