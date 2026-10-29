/**
 Tests for what the page-level consistency pass keeps of one group's words while
 it carries another group's chosen rendering: the words another group located in
 a block a writer rewrites reach the writers and the judges, a writer's paragraph
 that loses them is withheld, the same whichever group the offending writer
 belongs to, and the judges are shown the words each receiving place replaces.

 The page is the cat page, whose repeated sentence and refrain share the third
 section: the sentence's second place and the refrain's chosen place stand in one
 paragraph.

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
  type ArtifactConsistency,
  parseConsistency,
} from '@monochromatic-dev/translation-repair-artifact';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  ballotText,
  firstPlaceWins,
  otherSheetBehaviour,
  type SeatBehaviour,
  type SeatScript,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';
import {
  candidateKindsOf,
  runCatPass,
} from './consistency-run.test-fixture.ts';
import {
  REFRAIN_QUOTES,
  SENTENCE_QUOTES,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-quotes.test-fixture.ts';
import {
  candidateTextsOf,
  currentParagraphOf,
  isRefrainSheet,
  sheetBlockOf,
  sheetKindOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-sheet-reading.test-fixture.ts';

/**
 The refrain words of the third section, which the refrain's judges choose.
 */
const CHOSEN_REFRAIN = 'So this cat has to be a sweet little darling';

/**
 The sentence words of the first section, which the sentence's judges choose.
 */
const CHOSEN_SENTENCE = 'The sunny window put the kitten to sleep';

/**
 What the pass leaves of the four sections when every writer's paragraph that loses another group's words is withheld
 and the bench takes the chosen words moved in by code.
 */
const SPLICED_SECTIONS: readonly (readonly [number, string])[] = [
  [0, `## One\n\n${CHOSEN_SENTENCE}. So this cat has to be a chubby little darling.`,],
  [1, '## Two\n\nThe smell of fish in the kitchen woke the kitten. So this cat has to be a lazy little darling.',],
  [
    2,
    `## Three\n\nIn the evening, the sunny window put the kitten to sleep, and the fish snacks smiled too. ${CHOSEN_REFRAIN}.`,
  ],
  [3, '## Four\n\nThe rain in the yard sent the kitten running home. So this cat has to be a warm little darling.',],
];

/**
 The first-round ballots: the sentence's judges choose its first place, the refrain's judges its third.

 @param call - call the seat was handed, of which only the sheet is read

 @returns Behaviour of the seat

 @example
 ```ts
 const behave: SeatScript = ballotsOfTwoGroups;
 ```
 */
function ballotsOfTwoGroups({ sheet, }: Parameters<SeatScript>[0],): SeatBehaviour {
  return isRefrainSheet({ sheet, },)
    ? { kind: 'answers', text: ballotText({ quotes: REFRAIN_QUOTES, choice: 'OCCURRENCE 3', keep: [], },), }
    : firstPlaceWins({ sheet, },);
}

/**
 The writers and the judges of the carry, where one group's writer also rewrites words of the other group.

 The writers of the group that does not offend return the paragraph as it stands. The offending writer of the sentence
 puts the chosen words in and rewrites the refrain words that stand beside them; the offending writer of the refrain
 rewrites the sentence words that stand in its paragraph. A judge takes a paragraph that holds the rewrite when one is
 offered and otherwise the first candidate that is not the paragraph as the page has it now.

 @param offender - group whose writers rewrite the other group's words

 @returns Script of the seats asked after the first round

 @example
 ```ts
 const others: SeatScript = writersAndJudges({ offender: 'sentence', });
 ```
 */
function writersAndJudges({ offender, }: { readonly offender: 'sentence' | 'refrain'; },): SeatScript {
  return function behave({ sheet, },): SeatBehaviour {
    /**
     The question the sheet asks.
     */
    const kind = sheetKindOf({ sheet, },);
    /**
     Whether the sheet is about the refrain.
     */
    const refrain = sheet.includes('FRAME BEGINS WITH',);
    if (kind === 'writer') {
      /**
       The paragraph as it stands.
       */
      const current = currentParagraphOf({ sheet, },);
      if (refrain) {
        return {
          kind: 'answers',
          text: JSON.stringify({
            translation: (offender === 'refrain')
              ? current.replace(CHOSEN_SENTENCE, 'The kitten sleeps in the sun',)
              : current,
          },),
        };
      }
      return {
        kind: 'answers',
        text: JSON.stringify({
          translation: (offender === 'sentence')
            ? current
              .replace(sheetBlockOf({ sheet, label: 'THE WORDS TO REPLACE', },), CHOSEN_SENTENCE,)
              .replace(CHOSEN_REFRAIN, 'So this cat is a darling',)
            : current,
        },),
      };
    }
    if (kind === 'choice') {
      /**
       The candidates the sheet offers.
       */
      const candidates = candidateTextsOf({ sheet, },);
      /**
       The paragraph that holds a rewrite of the other group's words, if one is offered.
       */
      const offending = candidates.find(function holdsRewrite({ text, },): boolean {
        return text.includes('is a darling',) || text.includes('The kitten sleeps in the sun',);
      },);
      /**
       The first candidate that is not the paragraph as the page has it now.
       */
      const alternative = candidates.find(function isAlternative({ current, },): boolean {
        return !current;
      },);
      return {
        kind: 'answers',
        text: JSON.stringify({
          assessment: 'It carries the chosen words.',
          choice: offending?.label ?? alternative?.label ?? 'CANDIDATE 1',
        },),
      };
    }
    return otherSheetBehaviour({ sheet, },);
  };
}

/**
 The first-round ballots where the sentence's second quote runs on into the refrain words the refrain's judges choose,
 with writers that return the paragraph as it stands and judges that take the first candidate that is not the paragraph
 as the page has it now.

 @param call - call the seat was handed, of which only the sheet is read

 @returns Behaviour of the seat

 @example
 ```ts
 const behave: SeatScript = sentenceRunsIntoChosenRefrain;
 ```
 */
function sentenceRunsIntoChosenRefrain({ sheet, }: Parameters<SeatScript>[0],): SeatBehaviour {
  /**
   The question the sheet asks.
   */
  const kind = sheetKindOf({ sheet, },);
  if (kind === 'writer') {
    return { kind: 'answers', text: JSON.stringify({ translation: currentParagraphOf({ sheet, },), },), };
  }
  if (kind === 'choice') {
    return {
      kind: 'answers',
      text: JSON.stringify({
        assessment: 'It carries the chosen words.',
        choice: candidateTextsOf({ sheet, },).find(function isAlternative({ current, },): boolean {
          return !current;
        },)?.label ?? 'CANDIDATE 1',
      },),
    };
  }
  if (kind !== 'ballot')
    return otherSheetBehaviour({ sheet, },);
  return isRefrainSheet({ sheet, },)
    ? { kind: 'answers', text: ballotText({ quotes: REFRAIN_QUOTES, choice: 'OCCURRENCE 3', keep: [], },), }
    : {
      kind: 'answers',
      text: ballotText({
        quotes: [
          SENTENCE_QUOTES[0] as Record<string, string>,
          {
            label: 'OCCURRENCE 2',
            quote: `the sunlight on the sill lulled the kitten to sleep, and the fish snacks smiled too. ${CHOSEN_REFRAIN}`,
          },
        ],
        choice: 'OCCURRENCE 1',
        keep: [],
      },),
    };
}

/**
 The kinds of candidate each receiving place of each group was offered, in page order, sorted within a place.

 @param consistency - section the pass recorded

 @returns One list of kinds per receiving place, group by group

 @example
 ```ts
 const kinds = offeredKindsOf({ consistency, },); // [[['current', 'splice',],],]
 ```
 */
function offeredKindsOf(
  { consistency, }: { readonly consistency: ArtifactConsistency; },
): readonly (readonly (readonly string[])[])[] {
  return consistency.groups
    .map(function groupKinds({ carries, },): readonly (readonly string[])[] {
      return (carries ?? []).map(function placeKinds({ candidates, },): readonly string[] {
        return candidateKindsOf({ candidates, },);
      },);
    },);
}

/**
 What the pass recorded for each place of each group.

 @param consistency - section the pass recorded

 @returns Outcome of every place, group by group

 @example
 ```ts
 const outcomes = outcomesOfGroups({ consistency, },); // [['chosen', 'replaced',],]
 ```
 */
function outcomesOfGroups({ consistency, }: { readonly consistency: ArtifactConsistency; },): readonly (readonly string[])[] {
  return consistency.groups
    .map(function outcomesOf({ occurrences, },): readonly string[] {
      return occurrences.map(function outcomeOf({ outcome, },): string {
        return outcome;
      },);
    },);
}

/**
 The findings that say a writer's paragraph was withheld, with the seat taken out so they read alike.

 @param consistency - section the pass recorded

 @returns Distinct findings, in the order they first appear

 @example
 ```ts
 const refused = refusalsOf({ consistency, },);
 ```
 */
function refusalsOf({ consistency, }: { readonly consistency: ArtifactConsistency; },): readonly string[] {
  return [
    ...new Set(
      consistency.findings
        .filter(function isRefusal(finding,): boolean {
          return finding.includes('consistency-candidate-refused',);
        },)
        .map(function withoutSeat(finding,): string {
          return `${finding.slice(0, finding.indexOf('(',) + 1,)}seat${finding.slice(finding.indexOf(')',),)}`;
        },),
    ),
  ];
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'runPassConsistency keeping the words of the other group',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WITHHOLDS A SENTENCE WRITER\'S PARAGRAPH THAT REWRITES THE REFRAIN WORDS STANDING BESIDE ITS PLACE, so the '
            + 'refrain\'s chosen words reach the page as the bench chose them',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-keeping-', },);
            const run = await runCatPass({
              behave: ballotsOfTwoGroups,
              others: writersAndJudges({ offender: 'sentence', }),
              entryCacheDir: dir.path,
            },);
            expect({
              slices: run.consistency.slices.map(function textOf({ sliceIndex, text, },): readonly [number, string] {
                return [sliceIndex, text,];
              },),
              outcomes: outcomesOfGroups({ consistency: run.consistency, },),
              offered: offeredKindsOf({ consistency: run.consistency, },),
              refusals: refusalsOf({ consistency: run.consistency, },),
            },).toEqual({
              slices: SPLICED_SECTIONS,
              outcomes: [['chosen', 'replaced',], ['replaced', 'replaced', 'chosen', 'replaced',],],
              offered: [
                [['current', 'splice',],],
                [['current', 'splice',], ['current', 'splice',], ['current', 'splice',],],
              ],
              refusals: [
                'group 1: consistency-candidate-refused (seat): the paragraph no longer holds the words of another place '
                  + 'that must stay as they are',
              ],
            },);
            expect(parseConsistency({
              value: structuredClone(run.consistency,),
              path: 'cat.consistency',
              generation: 17,
              sliceIndexes: new Set([0, 1, 2, 3,],),
            },),).toEqual(run.consistency,);
          },
        },),
        it({
          name: 'WITHHOLDS A REFRAIN WRITER\'S PARAGRAPH THAT REWRITES THE SENTENCE WORDS STANDING IN ITS PARAGRAPH, so the '
            + 'sentence\'s chosen words reach the page as the bench chose them',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-keeping-', },);
            const run = await runCatPass({
              behave: ballotsOfTwoGroups,
              others: writersAndJudges({ offender: 'refrain', }),
              entryCacheDir: dir.path,
            },);
            expect({
              slices: run.consistency.slices.map(function textOf({ sliceIndex, text, },): readonly [number, string] {
                return [sliceIndex, text,];
              },),
              outcomes: outcomesOfGroups({ consistency: run.consistency, },),
              offered: offeredKindsOf({ consistency: run.consistency, },),
              refusals: refusalsOf({ consistency: run.consistency, },),
            },).toEqual({
              slices: SPLICED_SECTIONS,
              outcomes: [['chosen', 'replaced',], ['replaced', 'replaced', 'chosen', 'replaced',],],
              offered: [
                [['current', 'splice',],],
                [['current', 'splice',], ['current', 'splice',], ['current', 'splice',],],
              ],
              refusals: [
                'group 2: consistency-candidate-refused (seat): the paragraph no longer holds the words of another place '
                  + 'that must stay as they are',
              ],
            },);
          },
        },),
        it({
          name: 'LEAVES OUT A SPLICE THAT WOULD CHANGE THE WORDS OF A PLACE ANOTHER GROUP KEEPS, recording overlapped for '
            + 'the place beside the chosen refrain place that stands alone in its section, with a finding naming both groups, '
            + 'and records a section its reader accepts',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-keeping-', },);
            const run = await runCatPass({
              behave: sentenceRunsIntoChosenRefrain,
              entryCacheDir: dir.path,
            },);
            expect({
              slices: run.consistency.slices.map(function textOf({ sliceIndex, text, },): readonly [number, string] {
                return [sliceIndex, text,];
              },),
              outcomes: outcomesOfGroups({ consistency: run.consistency, },),
              gaveWay: run.consistency.groups.map(function gaveWayOfGroup({ occurrences, },): readonly unknown[] {
                return occurrences.map(function gaveWayTo(occurrence,): unknown {
                  return occurrence.gaveWayTo ?? 'none';
                },);
              },),
              findings: run.consistency.findings,
            },).toEqual({
              slices: [SPLICED_SECTIONS[0], SPLICED_SECTIONS[1], SPLICED_SECTIONS[3],],
              outcomes: [['chosen', 'overlapped',], ['replaced', 'replaced', 'chosen', 'replaced',],],
              gaveWay: [['none', { group: 1, label: 'OCCURRENCE 3', },], ['none', 'none', 'none', 'none',],],
              findings: [
                'slice 2: the consistency replacement of group 1 OCCURRENCE 2 was left out because it changes the words that '
                  + 'group 2 OCCURRENCE 3 located',
              ],
            },);
            expect(parseConsistency({
              value: structuredClone(run.consistency,),
              path: 'cat.consistency',
              generation: 17,
              sliceIndexes: new Set([0, 1, 2, 3,],),
            },),).toEqual(run.consistency,);
          },
        },),
        it({
          name: 'SHOWS THE JUDGES WHO CHOOSE AMONG A BLOCK\'S CANDIDATES THE WORDS EACH RECEIVING PLACE REPLACES, as the writers '
            + 'of the candidates were shown them',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-keeping-', },);
            const run = await runCatPass({
              behave: ballotsOfTwoGroups,
              others: writersAndJudges({ offender: 'sentence', }),
              entryCacheDir: dir.path,
            },);
            expect([
              ...new Set(
                run.sheets
                  .filter(function isChoice(sheet,): boolean {
                    return sheetKindOf({ sheet, },) === 'choice';
                  },)
                  .map(function replacedOn(sheet,): string {
                    return sheetBlockOf({ sheet, label: 'THE WORDS TO REPLACE', },);
                  },),
              ),
            ].toSorted(),).toEqual([
              'So this cat is certainly a warm little dumpling',
              'So this cat is surely a lazy little treasure',
              'So this cat must be a chubby little treasure',
              'the sunlight on the sill lulled the kitten to sleep',
            ],);
          },
        },),
      ],
    },),
  ],
},);
