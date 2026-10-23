/**
 Tests for carrying the chosen rendering to the receiving places of one block by
 writing, not by splicing: the writers are asked for the block, the bench
 chooses among the current block, the code splice and each writer's block, and
 the winner makes the replacement; places that share a block are carried
 together, a place whose words lie in no single block is carried by the splice
 alone, a place whose carry goes wrong keeps its English with a record and a
 finding, and a caller's abort is not contained.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';

import { assertSlicesCoherent, } from '@monochromatic-dev/translation-repair-artifact';
import {
  applyConsistency,
  carryAll,
  type CarryReceiver,
  carryToUnit,
  type CarryUnit,
  keptWordsOf,
  settleAsked,
  settleCarried,
  type UnitCarried,
} from '../../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { firstPlaceBallot, } from '@monochromatic-dev/translation-repair-artifact/ts/corpus-run/consistency-group.test-fixture.ts';
import {
  choosing,
  type SeatBehaviour,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';
import {
  candidateTextsOf,
  sheetKindOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-sheet-reading.test-fixture.ts';
import {
  CARRY_REPEAT,
  carryScene,
  type CarryScene,
  RECEIVING_ENGLISH,
  SPLICED_PARAGRAPH,
} from './consistency-carry.test-fixture.ts';

/**
 What a writer wrote.
 */
const WRITTEN = 'Later the cat dozes in the sun.';

/**
 Writers that all write the same paragraph, and judges that all choose the candidate holding a text.

 @param choice - the text of the candidate every judge chooses

 @returns What each seat does

 @example
 ```ts
 const behave = writtenAndChosen({ choice: WRITTEN, },);
 ```
 */
function writtenAndChosen({ choice, }: { readonly choice: string; },) {
  return function behaves({ sheet, }: { readonly sheet: string; },): SeatBehaviour {
    if (sheetKindOf({ sheet, },) === 'writer')
      return { kind: 'answers', text: JSON.stringify({ translation: WRITTEN, },), };
    return choosing({
      choice: candidateTextsOf({ sheet, },)
        .find(function holdsIt(candidate,): boolean {
          return candidate.text === choice;
        },)
        ?.label ?? '',
    },);
  };
}

/**
 A writer that rewrites the second paragraph of the crossing page, and judges that choose a writer's paragraph where one
 is on offer and the first candidate other than the current paragraph otherwise.

 @param sheet - the sheet the seat was shown

 @returns What the seat does

 @example
 ```ts
 const scene = carryScene({ behave: writerOfSecondParagraph, },);
 ```
 */
function writerOfSecondParagraph({ sheet, }: { readonly sheet: string; },): SeatBehaviour {
  if (sheetKindOf({ sheet, },) === 'writer')
    return { kind: 'answers', text: JSON.stringify({ translation: 'naps in the sun. The cat purrs.', },), };
  /**
   The candidates the sheet offers.
   */
  const candidates = candidateTextsOf({ sheet, },);
  return choosing({
    choice: (candidates.find(function isWriters({ text, },): boolean {
      return text.includes('The cat purrs',);
    },) ?? candidates.find(function isAlternative({ current, },): boolean {
      return !current;
    },))
      ?.label ?? '',
  },);
}

/**
 Carries the chosen words to the scene's receiving block.

 @param scene - the scene

 @param unit - the receiving block, the scene's by default

 @returns How the carry ended

 @example
 ```ts
 const carried = await carriedIn({ scene: carryScene({ behave, },), },);
 ```
 */
async function carriedIn(
  {
    scene,
    unit = scene.unit,
  }: {
    readonly scene: CarryScene;
    readonly unit?: CarryUnit;
  },
): Promise<UnitCarried> {
  return await carryToUnit({
    context: scene.context,
    repeat: CARRY_REPEAT,
    chosen: scene.chosen,
    unit,
  },);
}

/**
 The scene's block with its one receiving place replaced.

 @param scene - the scene

 @param receiver - the receiving place, as a change to the scene's own

 @returns The unit holding that place alone

 @example
 ```ts
 const unit = unitOf({ scene, receiver: { ...scene.receiver, plan: { kind: 'unavailable', }, }, },);
 ```
 */
function unitOf(
  {
    scene,
    receiver,
  }: {
    readonly scene: CarryScene;
    readonly receiver: CarryReceiver;
  },
): CarryUnit {
  return {
    ...scene.unit,
    receivers: [receiver,],
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: carryToUnit.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CARRIES THE CHOSEN WORDS BY WRITING: the writers are asked first, then the judges choose among the current '
            + 'paragraph, the splice and the writers\' paragraph, and the writer\'s paragraph replaces the block',
          fn: async () => {
            const scene = carryScene({ behave: writtenAndChosen({ choice: WRITTEN, }), },);
            const { carries, edits, findings, } = await carriedIn({ scene, },);
            expect({
              labels: carries.map(function labelOf({ label, },): string {
                return label;
              },),
              kinds: carries[0]?.candidates.map(function kindOf({ kind, },): string {
                return kind;
              },).toSorted(),
              decision: carries[0]?.decision.kind,
              edits,
              findings,
              order: scene.sheets.map(function kindOf(sheet,): string {
                return sheetKindOf({ sheet, },);
              },).filter(function isFirstOfRun(kind, at, kinds,): boolean {
                return kind !== kinds[at - 1];
              },),
            },).toEqual({
              labels: ['OCCURRENCE 2',],
              kinds: ['current', 'splice', 'writer',],
              decision: 'chosen',
              edits: [{ kind: 'block', placePositions: [1,], sliceIndex: 1, start: 0, end: RECEIVING_ENGLISH.length, text: WRITTEN, },],
              findings: [],
              order: ['writer', 'choice',],
            },);
          },
        },),
        it({
          name: 'REPLACES ONLY THE WORDS WHEN THE BENCH CHOOSES THE SPLICE, and nothing when it chooses the current paragraph',
          fn: async () => {
            const spliced = await carriedIn({
              scene: carryScene({ behave: writtenAndChosen({ choice: SPLICED_PARAGRAPH, }), },),
            },);
            const kept = await carriedIn({
              scene: carryScene({ behave: writtenAndChosen({ choice: RECEIVING_ENGLISH, }), },),
            },);
            expect({ spliced: spliced.edits, kept: kept.edits, },).toEqual({
              spliced: [{ kind: 'splice', placePositions: [1,], sliceIndex: 1, start: 6, end: 21, text: 'the cat naps', },],
              kept: [],
            },);
          },
        },),
        it({
          name: 'PUTS NO QUESTION TO THE BENCH WHEN THE WRITERS ARE NOT ASKED AND THE SPLICE IS NOT POSSIBLE, so the place '
            + 'keeps its English with no alternative recorded and a finding naming the bench and why it was not asked',
          fn: async () => {
            const scene = carryScene({
              behave: writtenAndChosen({ choice: WRITTEN, }),
              writers: {
                kind: 'not-asked',
                why: 'translators 0 of 8 reachable, floor 2; no provider has named a return the pass can wait for',
              },
            },);
            const { carries, edits, findings, } = await carriedIn({
              scene,
              unit: unitOf({ scene, receiver: { ...scene.receiver, plan: { kind: 'unavailable', }, }, },),
            },);
            expect({ decisions: carries.map(function decisionOf({ decision, },) {
              return decision;
            },), edits, findings, asked: scene.asked, },).toEqual({
              decisions: [{ kind: 'none', reason: 'no-alternative', },],
              edits: [],
              findings: [
                'consistency-writers-not-asked (OCCURRENCE 2): the translators bench was not asked: '
                  + 'translators 0 of 8 reachable, floor 2; no provider has named a return the pass can wait for',
              ],
              asked: [],
            },);
          },
        },),
        it({
          name: 'CHOOSES BETWEEN THE CURRENT ENGLISH AND THE SPLICE WHEN THE WRITERS ARE NOT ASKED, and asks no writer',
          fn: async () => {
            const scene = carryScene({
              behave: writtenAndChosen({ choice: SPLICED_PARAGRAPH, }),
              writers: {
                kind: 'not-asked',
                why: 'translators 0 of 8 reachable, floor 2; no provider has named a return the pass can wait for',
              },
            },);
            const { carries, edits, findings, } = await carriedIn({ scene, },);
            expect({
              kinds: carries[0]?.candidates.map(function kindOf({ kind, },): string {
                return kind;
              },).toSorted(),
              edits,
              findings,
              kindsAsked: [...new Set(scene.sheets.map(function kindOf(sheet,): string {
                return sheetKindOf({ sheet, },);
              },),),],
            },).toEqual({
              kinds: ['current', 'splice',],
              edits: [{ kind: 'splice', placePositions: [1,], sliceIndex: 1, start: 6, end: 21, text: 'the cat naps', },],
              findings: [
                'consistency-writers-not-asked (OCCURRENCE 2): the translators bench was not asked: '
                  + 'translators 0 of 8 reachable, floor 2; no provider has named a return the pass can wait for',
              ],
              kindsAsked: ['choice',],
            },);
          },
        },),
        it({
          name: 'CONTAINS A CARRY THAT GOES WRONG: the places keep their English with a failed decision, a finding and an '
            + 'error line, whatever the failure was',
          fn: async () => {
            const scene = carryScene({ behave: writtenAndChosen({ choice: WRITTEN, }), },);
            const { carries, edits, findings, } = await carriedIn({
              scene,
              unit: unitOf({
                scene,
                receiver: {
                  ...scene.receiver,
                  place: {
                    ...scene.receiver.place,
                    slice: { ...scene.receiver.place.slice, current: { kind: 'absent', }, },
                  },
                },
              },),
            },);
            expect({
              decisions: carries.map(function kindOf({ decision, },): string {
                return decision.kind;
              },),
              edits,
              findings,
              logged: scene.lines.filter(function isError(line,): boolean {
                return line.startsWith('error [carryToUnit] the carry to OCCURRENCE 2 could not be made',);
              },).length,
            },).toEqual({
              decisions: ['failed',],
              edits: [],
              findings: ['consistency-carry-failed (OCCURRENCE 2): the place keeps its English',],
              logged: 1,
            },);
          },
        },),
        it({
          name: 'DOES NOT CONTAIN THE CALLER\'S ABORT: when the signal has fired the failure is thrown again',
          fn: async () => {
            const scene = carryScene({ behave: writtenAndChosen({ choice: WRITTEN, }), },);
            const refusal = await rejectionOf(async function carriesAfterAbort(): Promise<unknown> {
              return await carryToUnit({
                context: { ...scene.context, signal: AbortSignal.abort(new Error('the cat woke the run',),), },
                repeat: CARRY_REPEAT,
                chosen: scene.chosen,
                unit: unitOf({
                  scene,
                  receiver: {
                    ...scene.receiver,
                    place: {
                      ...scene.receiver.place,
                      slice: { ...scene.receiver.place.slice, current: { kind: 'absent', }, },
                    },
                  },
                },),
              },);
            },);
            expect(refusal,).toBeInstanceOf(Error,);
          },
        },),
      ],
    },),
    describe({
      name: carryAll.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'MAKES NO TWO EDITS OF ONE GROUP THAT OVERLAP WHEN ONE RECEIVING PLACE\'S WORDS CROSS A PARAGRAPH BOUNDARY AND '
            + 'ANOTHER STANDS IN THE SECOND PARAGRAPH: the second paragraph is not given to a writer, the crossing place is '
            + 'carried by its splice and the other, whose words hold another number of block breaks than the chosen words, '
            + 'keeps its words, the edits apply without the invariant throw, and the section is read back at '
            + 'generation 17',
          fn: async () => {
            const english = 'Later the kitten\n\nnaps in the sun. The kitten purrs.';
            const quotes = [
              { label: 'OCCURRENCE 1', quote: 'The cat\n\nnaps', },
              { label: 'OCCURRENCE 2', quote: 'the kitten\n\nnaps', },
              { label: 'OCCURRENCE 3', quote: 'The kitten purrs', },
            ];
            const scene = carryScene({ behave: writerOfSecondParagraph, },);
            /**
             The chosen place, whose words cross a paragraph boundary as the first receiving place's do, since words that
             hold another number of block breaks have no splice (`consistency-block-breaks.ts`).
             */
            const crossingChosen = {
              ...scene.chosen.place,
              slice: { ...scene.chosen.place.slice, current: { kind: 'present', text: 'The cat\n\nnaps. It is warm.', }, },
            } as const;
            /**
             A receiving place of the scene whose slice carries the crossing page's English.
             */
            const receivingPlace = {
              ...scene.receiver.place,
              slice: { ...scene.receiver.place.slice, current: { kind: 'present', text: english, }, },
            } as const;
            const decided = settleAsked({
              repeat: CARRY_REPEAT,
              places: [crossingChosen, receivingPlace, receivingPlace,],
              unplaced: [],
              asked: {
                quorum: 2,
                resumed: false,
                findings: [],
                voices: ['hyper-cat', 'bedrock-cat', 'synthetic-cat',].map(function ballotOf(seat,) {
                  return firstPlaceBallot({ seat, quotes, },);
                },),
              },
            },);
            const carried = await carryAll({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: nonNullishOrThrow(decided.chosen[0],),
              receivers: decided.receivers,
              kept: keptWordsOf({ decided, },),
            },);
            const edits = carried.flatMap(function editsOf({ edits: made, },) {
              return made;
            },)
              .toSorted(function inPageOrder(left, right,): number {
                return left.start - right.start;
              },);
            expect({
              kinds: edits.map(function kindOf({ kind, },): string {
                return kind;
              },),
              apart: edits.every(function endsBeforeNext(edit, at,): boolean {
                return (edits[at + 1]?.start ?? Infinity) >= edit.end;
              },),
              writersAsked: scene.sheets.some(function isWriterSheet(sheet,): boolean {
                return sheetKindOf({ sheet, },) === 'writer';
              },),
              findings: carried.flatMap(function findingsOf({ findings, },) {
                return findings;
              },),
            },).toEqual({
              kinds: ['splice',],
              apart: true,
              writersAsked: false,
              findings: [
                'consistency-writers-not-asked (OCCURRENCE 2): the words that render the repeated wording lie in no single '
                  + 'block of the English, so no paragraph can be rewritten',
                'consistency-writers-not-asked (OCCURRENCE 3): the paragraph that holds the words that render the repeated '
                  + 'wording overlaps the words of another receiving place of the group that lie outside it, so no paragraph '
                  + 'can be rewritten',
              ],
            },);
            const applied = applyConsistency({
              settled: [settleCarried({ decided, carried, },),],
              slices: [decided.places[0], decided.places[1],].map(function sliceOf(place,) {
                return nonNullishOrThrow(place,).slice;
              },),
              declared: [],
              declaredNames: [],
              l: levelCapturingLogger({ lines: [], },),
            },);
            expect(applied.groups.map(function outcomesRecorded({ occurrences, },): readonly string[] {
              return occurrences.map(function outcomeRecorded({ outcome, },): string {
                return outcome;
              },);
            },),).toEqual([['chosen', 'replaced', 'kept-current',],],);
            // The reader throws for a section it refuses, so returning is the acceptance.
            assertSlicesCoherent({ groups: applied.groups, slices: applied.slices, path: 'cat.consistency', generation: 17, },);
          },
        },),
        it({
          name: 'CARRIES THE WORDS THAT LIE IN NO SINGLE BLOCK BY THE SPLICE ALONE: the whole English of the slice is the '
            + 'block, no writer is asked, and the bench chooses between the English as it stands and the splice',
          fn: async () => {
            const scene = carryScene({ behave: writtenAndChosen({ choice: 'Later the cat naps in the sun.', }), },);
            const english = 'Later the kitten\n\nnaps in the sun.';
            const [carried,] = await carryAll({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              receivers: [
                {
                  ...scene.receiver,
                  place: {
                    ...scene.receiver.place,
                    slice: { ...scene.receiver.place.slice, current: { kind: 'present', text: english, }, },
                  },
                  rendering: { kind: 'located', quote: 'the kitten\n\nnaps', core: 'the kitten\n\nnaps', start: 6, },
                  plan: { kind: 'replace', start: 6, end: 22, text: 'the cat naps', },
                },
              ],
              kept: [],
            },);
            expect({
              candidates: carried?.carries[0]?.candidates,
              decision: carried?.carries[0]?.decision,
              edits: carried?.edits,
              findings: carried?.findings,
              kindsAsked: [...new Set(scene.sheets.map(function kindOf(sheet,): string {
                return sheetKindOf({ sheet, },);
              },),),],
            },).toEqual({
              candidates: [
                { label: 'CANDIDATE 1', kind: 'splice', writers: [], text: 'Later the cat naps in the sun.', },
                { label: 'CANDIDATE 2', kind: 'current', writers: [], text: english, },
              ],
              decision: { kind: 'chosen', label: 'CANDIDATE 1', },
              edits: [{ kind: 'splice', placePositions: [1,], sliceIndex: 1, start: 6, end: 22, text: 'the cat naps', },],
              findings: [
                'consistency-writers-not-asked (OCCURRENCE 2): the words that render the repeated wording lie in no single '
                  + 'block of the English, so no paragraph can be rewritten',
              ],
              kindsAsked: ['choice',],
            },);
          },
        },),
      ],
    },),
  ],
},);
