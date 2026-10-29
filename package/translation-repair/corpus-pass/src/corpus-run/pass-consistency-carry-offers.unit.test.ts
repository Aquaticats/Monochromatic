/**
 Tests for the texts offered for a block beside its current one and for what the
 two sheets of a block's carry show of the places: the chosen words moved into
 every receiving place of the block by code, where that is possible and every
 floor passes it, the blocks the writer bench writes for the places that pass
 the floors, and the chosen place's paragraph and slot, the receiving places'
 originals and the original blocks around them.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  blockStandingOf,
  ConsistencyInvariantError,
} from '@monochromatic-dev/translation-repair-consistency';
import {
  type CarryContext,
  type CarryReceiver,
  type CarryUnit,
  shownOf,
  splicedParagraphOf,
  unitsOf,
  writtenOffers,
} from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import type { SeatBehaviour, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';
import {
  sheetBlockOf,
  sheetKindOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-sheet-reading.test-fixture.ts';
import {
  CARRY_REPEAT,
  carryScene,
  RECEIVING_ENGLISH,
  secondReceiverOf,
  SPLICED_PARAGRAPH,
} from './consistency-carry.test-fixture.ts';

/**
 What a seat does when every writer returns a paragraph with the chosen words in it, and a judge is never asked.

 @param seat - seat asked

 @param sheet - sheet it was shown

 @returns Seat's reply

 @example
 ```ts
 const behave: SeatScript = writersAnswer;
 ```
 */
function writersAnswer({ seat, sheet, }: { readonly seat: string; readonly sheet: string; },): SeatBehaviour {
  return (sheetKindOf({ sheet, },) === 'writer')
    ? { kind: 'answers', text: JSON.stringify({ translation: `Later the cat dozes in the sun (${seat}).`, },), }
    : { kind: 'throws', };
}

/**
 The receiving place with another plan.

 @param receiver - receiving place

 @param plan - plan for it

 @returns Receiver

 @example
 ```ts
 const unavailable = withPlan({ receiver, plan: { kind: 'unavailable', }, },);
 ```
 */
function withPlan(
  {
    receiver,
    plan,
  }: {
    readonly receiver: CarryReceiver;
    readonly plan: CarryReceiver['plan'];
  },
): CarryReceiver {
  return { ...receiver, plan, };
}

/**
 The context with the floors told of a declared name.

 @param context - context

 @param declaredNames - names the front matter declares

 @returns Context

 @example
 ```ts
 const named = withDeclaredNames({ context, declaredNames: ['kitten',], },);
 ```
 */
function withDeclaredNames(
  {
    context,
    declaredNames,
  }: {
    readonly context: CarryContext;
    readonly declaredNames: readonly string[];
  },
): CarryContext {
  return { ...context, declaredNames, };
}

/**
 The seats of the scene's writer bench.

 @param context - what the carry reads

 @returns Writer bench, none when the writers are not asked

 @example
 ```ts
 const seats = writerSeatsOf({ context: scene.context, },);
 ```
 */
function writerSeatsOf({ context, }: { readonly context: CarryContext; },): readonly string[] {
  return (context.writers.kind === 'asked') ? context.writers.modelIds : [];
}

/**
 The first seat of the scene's writer bench.

 @param context - what the carry reads

 @returns Seat, empty when the writers are not asked

 @example
 ```ts
 const seat = firstWriterOf({ context: scene.context, },);
 ```
 */
function firstWriterOf({ context, }: { readonly context: CarryContext; },): string {
  return writerSeatsOf({ context, },)[0] ?? '';
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: shownOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHOWS THE CHOSEN PLACE\'S WORDS AND PARAGRAPH, the receiving places\' words in page order and the words '
            + 'to keep',
          fn: async () => {
            const scene = carryScene({ behave: writersAnswer, },);
            const unit: CarryUnit = {
              ...scene.unit,
              receivers: [scene.receiver, secondReceiverOf({ receiver: scene.receiver, },),],
              keeping: ['The cat naps',],
            };
            const shown = shownOf({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit,
            },);
            expect({
              chosenSpan: shown.chosen.span,
              chosenParagraph: shown.chosen.paragraph,
              chosenHasSlot: 'slot' in shown.chosen,
              receivingSpans: shown.receiving.map(function spanOf({ span, },): string {
                return span;
              },),
              keeping: shown.keeping,
            },).toEqual({
              chosenSpan: 'The cat naps',
              chosenParagraph: 'The cat naps. It is warm.',
              chosenHasSlot: false,
              receivingSpans: ['the kitten naps', 'the sun',],
              keeping: ['The cat naps',],
            },);
          },
        },),
        it({
          name: 'SHOWS THE CHOSEN PLACE\'S ENGLISH SLOT FOR A FRAME WHOSE SLOT WAS LOCATED, and none for a repeated run or '
            + 'a frame without one',
          fn: async () => {
            const scene = carryScene({ behave: writersAnswer, },);
            const slotted = {
              ...scene.chosen,
              rendering: {
                kind: 'located',
                quote: 'The cat naps',
                core: 'The cat naps',
                start: 0,
                slot: 'cat',
                slotStart: 4,
              },
            } as const;
            const refrain = { kind: 'refrain', prefix: '小猫', suffix: '打盹', } as const;
            const slots = [
              shownOf({ context: scene.context, repeat: refrain, chosen: slotted, unit: scene.unit, },).chosen,
              shownOf({ context: scene.context, repeat: CARRY_REPEAT, chosen: slotted, unit: scene.unit, },).chosen,
              shownOf({ context: scene.context, repeat: refrain, chosen: scene.chosen, unit: scene.unit, },).chosen,
            ].map(function slotOf(chosen,): string {
              return ('slot' in chosen) ? chosen.slot : 'none';
            },);
            expect(slots,).toEqual(['cat', 'none', 'none',],);
          },
        },),
        it({
          name: 'SHOWS THE ORIGINAL BLOCKS BEFORE AND AFTER THE BLOCKS THAT HOLD THE RECEIVING PLACES, and none where the '
            + 'slice has no body block around them',
          fn: async () => {
            const scene = carryScene({ behave: writersAnswer, },);
            const { place, } = scene.receiver;
            /**
             A block of the slice's original.
             */
            const nodeOf = (
              { id, text, startOffset, endOffset, }: {
                readonly id: string;
                readonly text: string;
                readonly startOffset: number;
                readonly endOffset: number;
              },
            ) => ({
              id,
              zone: 'body',
              kind: 'paragraph',
              text,
              startOffset,
              endOffset,
              contentHash: id,
            }) as const;
            const surrounded: CarryUnit = {
              ...scene.unit,
              receivers: [
                {
                  ...scene.receiver,
                  place: {
                    ...place,
                    slice: {
                      ...place.slice,
                      nodes: [
                        nodeOf({ id: 'a', text: '第一段', startOffset: 0, endOffset: 3, },),
                        nodeOf({
                          id: 'b',
                          text: place.slice.sourceText,
                          startOffset: place.slice.sourceStart,
                          endOffset: place.slice.sourceStart + place.slice.sourceText.length,
                        },),
                        nodeOf({ id: 'c', text: '第三段', startOffset: 900, endOffset: 903, },),
                      ],
                    },
                  },
                },
              ],
            };
            const around = shownOf({ context: scene.context, repeat: CARRY_REPEAT, chosen: scene.chosen, unit: surrounded, },);
            const alone = shownOf({ context: scene.context, repeat: CARRY_REPEAT, chosen: scene.chosen, unit: scene.unit, },);
            expect({
              before: around.before,
              after: around.after,
              aloneHasBefore: 'before' in alone,
              aloneHasAfter: 'after' in alone,
            },).toEqual({
              before: '第一段',
              after: '第三段',
              aloneHasBefore: false,
              aloneHasAfter: false,
            },);
          },
        },),
        it({
          name: 'SHOWS THE WHOLE ENGLISH OF THE CHOSEN PLACE\'S SLICE WHEN ITS WORDS LIE IN NO SINGLE BLOCK, never a bare '
            + 'span as a paragraph',
          fn: async () => {
            const scene = carryScene({ behave: writersAnswer, },);
            const english = 'The cat\n\nnaps. It is warm.';
            const crossing = {
              place: {
                ...scene.chosen.place,
                slice: { ...scene.chosen.place.slice, current: { kind: 'present', text: english, }, },
              },
              rendering: { kind: 'located', quote: 'cat\n\nnaps', core: 'cat\n\nnaps', start: 4, },
            } as const;
            const shown = shownOf({ context: scene.context, repeat: CARRY_REPEAT, chosen: crossing, unit: scene.unit, },);
            expect({ span: shown.chosen.span, paragraph: shown.chosen.paragraph, },).toEqual({
              span: 'cat\n\nnaps',
              paragraph: english,
            },);
          },
        },),
      ],
    },),
    describe({
      name: splicedParagraphOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'OFFERS THE PARAGRAPH WITH THE CHOSEN WORDS MOVED IN BY CODE when the plan replaces the words and every floor passes',
          fn: async () => {
            const { unit, context, } = carryScene({ behave: writersAnswer, },);
            expect(splicedParagraphOf({ unit, english: RECEIVING_ENGLISH, context, },),).toEqual({
              spliced: [SPLICED_PARAGRAPH,],
              findings: [],
            },);
          },
        },),
        it({
          name: 'MOVES THE CHOSEN WORDS INTO EVERY RECEIVING PLACE OF THE BLOCK IN ONE PARAGRAPH, each place its own plan',
          fn: async () => {
            const { unit, context, receiver, } = carryScene({ behave: writersAnswer, },);
            expect(splicedParagraphOf({
              unit: { ...unit, receivers: [receiver, secondReceiverOf({ receiver, },),], },
              english: RECEIVING_ENGLISH,
              context,
            },),).toEqual({
              spliced: ['Later the cat naps in the warm sun.',],
              findings: [],
            },);
          },
        },),
        it({
          name: 'OFFERS NOTHING WHEN THE WORDS CANNOT BE MOVED BY CODE INTO ANY RECEIVING PLACE OF THE BLOCK, whether the place '
            + 'already carries them or has no slot, and when every place of two is one code cannot move',
          fn: async () => {
            const { unit, context, receiver, } = carryScene({ behave: writersAnswer, },);
            expect([
              splicedParagraphOf({
                unit: { ...unit, receivers: [withPlan({ receiver, plan: { kind: 'same', }, },),], },
                english: RECEIVING_ENGLISH,
                context,
              },),
              splicedParagraphOf({
                unit: { ...unit, receivers: [withPlan({ receiver, plan: { kind: 'unavailable', }, },),], },
                english: RECEIVING_ENGLISH,
                context,
              },),
              splicedParagraphOf({
                unit: {
                  ...unit,
                  receivers: [
                    withPlan({ receiver, plan: { kind: 'unavailable', }, },),
                    withPlan({ receiver: secondReceiverOf({ receiver, },), plan: { kind: 'unavailable', }, },),
                  ],
                },
                english: RECEIVING_ENGLISH,
                context,
              },),
            ],).toEqual([
              { spliced: [], findings: [], },
              { spliced: [], findings: [], },
              { spliced: [], findings: [], },
            ],);
          },
        },),
        it({
          name: 'OFFERS THE PARAGRAPH SPLICED FOR THE PLACES CODE CAN MOVE THE WORDS INTO WHEN ANOTHER PLACE OF THE BLOCK '
            + 'CANNOT BE MOVED, whichever of the two places comes first, the other place keeping its words',
          fn: async () => {
            const { unit, context, receiver, } = carryScene({ behave: writersAnswer, },);
            /**
             The second receiving place, whose words stand later in the paragraph.
             */
            const second = secondReceiverOf({ receiver, },);
            expect([
              splicedParagraphOf({
                unit: { ...unit, receivers: [receiver, withPlan({ receiver: second, plan: { kind: 'unavailable', }, },),], },
                english: RECEIVING_ENGLISH,
                context,
              },),
              splicedParagraphOf({
                unit: { ...unit, receivers: [withPlan({ receiver, plan: { kind: 'unavailable', }, },), second,], },
                english: RECEIVING_ENGLISH,
                context,
              },),
            ],).toEqual([
              { spliced: [SPLICED_PARAGRAPH,], findings: [], },
              { spliced: ['Later the kitten naps in the warm sun.',], findings: [], },
            ],);
          },
        },),
        it({
          name: 'WITHHOLDS THE SPLICE A FLOOR REFUSES, with a finding naming the places and a warning',
          fn: async () => {
            const scene = carryScene({ behave: writersAnswer, },);
            const { spliced, findings, } = splicedParagraphOf({
              unit: scene.unit,
              english: RECEIVING_ENGLISH,
              context: withDeclaredNames({ context: scene.context, declaredNames: ['kitten',], },),
            },);
            expect({
              spliced,
              finding: findings.map(function head(finding,): string {
                return finding.slice(0, 'consistency-candidate-refused (splice of OCCURRENCE 2): '.length,);
              },),
              warned: scene.lines.map(function head(line,): string {
                return line.slice(
                  0,
                  'warn [splicedParagraphOf] withheld the splice for OCCURRENCE 2 from the bench: '.length,
                );
              },),
            },).toEqual({
              spliced: [],
              finding: ['consistency-candidate-refused (splice of OCCURRENCE 2): ',],
              warned: ['warn [splicedParagraphOf] withheld the splice for OCCURRENCE 2 from the bench: ',],
            },);
          },
        },),
        it({
          name: 'REFUSES TWO RECEIVING PLACES OF A BLOCK WHOSE WORDS OVERLAP, since the counting leaves such places unlocated '
            + 'before any carry',
          fn: async () => {
            const { unit, context, receiver, } = carryScene({ behave: writersAnswer, },);
            const overlapping: CarryReceiver = {
              ...secondReceiverOf({ receiver, },),
              plan: { kind: 'replace', start: 10, end: 25, text: 'the dog naps', },
            };
            const refusal = caught(function splicesOverlap(): unknown {
              return splicedParagraphOf({
                unit: { ...unit, receivers: [receiver, overlapping,], },
                english: RECEIVING_ENGLISH,
                context,
              },);
            },);
            expect(refusal,).toBeInstanceOf(ConsistencyInvariantError,);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: the receiving places OCCURRENCE 2 and OCCURRENCE 3 of one block have '
                + 'words that overlap, and places whose words overlap are left unlocated before a carry',
            );
          },
        },),
      ],
    },),
    describe({
      name: writtenOffers.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS THE WRITER BENCH FOR THE BLOCK AND KEEPS EACH PARAGRAPH THE FLOORS PASS, in roster order',
          fn: async () => {
            const scene = carryScene({ behave: writersAnswer, },);
            const { written, findings, } = await writtenOffers({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: scene.unit,
              english: RECEIVING_ENGLISH,
            },);
            expect({
              seats: written.map(function seatOf({ seat, },): string {
                return seat;
              },).toSorted(),
              texts: written.map(function textOf({ text, },): boolean {
                return text.startsWith('Later the cat dozes in the sun (',);
              },),
              findings,
              askedWriters: scene.sheets.every(function isWriterSheet(sheet,): boolean {
                return sheetKindOf({ sheet, },) === 'writer';
              },),
              chosenParagraphShown: scene.sheets.map(function paragraphOf(sheet,): string {
                return sheetBlockOf({ sheet, label: 'CHOSEN PLACE: ENGLISH PARAGRAPH', },);
              },),
            },).toEqual({
              seats: [...writerSeatsOf({ context: scene.context, },),].toSorted(),
              texts: [true, true, true,],
              findings: [],
              askedWriters: true,
              chosenParagraphShown: [
                'The cat naps. It is warm.',
                'The cat naps. It is warm.',
                'The cat naps. It is warm.',
              ],
            },);
          },
        },),
        it({
          name: 'SHOWS THE WRITERS EVERY RECEIVING PLACE OF THE BLOCK, each with its own words to replace and the one '
            + 'paragraph that holds them',
          fn: async () => {
            const scene = carryScene({ behave: writersAnswer, },);
            await writtenOffers({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: { ...scene.unit, receivers: [scene.receiver, secondReceiverOf({ receiver: scene.receiver, },),], },
              english: RECEIVING_ENGLISH,
            },);
            const [sheet,] = scene.sheets;
            expect({
              first: sheetBlockOf({ sheet: sheet ?? '', label: 'THE WORDS TO REPLACE 1', },),
              second: sheetBlockOf({ sheet: sheet ?? '', label: 'THE WORDS TO REPLACE 2', },),
              paragraph: sheetBlockOf({ sheet: sheet ?? '', label: 'CURRENT ENGLISH PARAGRAPH', },),
            },).toEqual({
              first: 'the kitten naps',
              second: 'the sun',
              paragraph: RECEIVING_ENGLISH,
            },);
          },
        },),
        it({
          name: 'WITHHOLDS A WRITER\'S PARAGRAPH THE FLOORS REFUSE, reporting it beside what the asking reported',
          fn: async () => {
            const scene = carryScene({
              behave: function oneAddsABlock({ seat, sheet, },): SeatBehaviour {
                return (seat === firstWriterOf({ context: scene.context, }))
                  ? { kind: 'answers', text: JSON.stringify({ translation: 'Later the cat dozes.\n\nAnd purrs.', },), }
                  : writersAnswer({ seat, sheet, },);
              },
            },);
            const { written, findings, } = await writtenOffers({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: scene.unit,
              english: RECEIVING_ENGLISH,
            },);
            expect({
              seats: written.map(function seatOf({ seat, },): string {
                return seat;
              },),
              refused: findings.filter(function isRefusal(finding,): boolean {
                return finding.startsWith(`consistency-candidate-refused (${firstWriterOf({ context: scene.context, })}): `,);
              },).length,
            },).toEqual({
              seats: writerSeatsOf({ context: scene.context, }).slice(1,).toSorted(),
              refused: 1,
            },);
          },
        },),
        it({
          name: 'WITHHOLDS A WRITER\'S PARAGRAPH THAT NO LONGER HOLDS THE WORDS OF THE CHOSEN PLACE standing in the block, and '
            + 'keeps one that holds them',
          fn: async () => {
            const scene = carryScene({
              behave: function oneDropsThem({ seat, },): SeatBehaviour {
                return { kind: 'answers', text: JSON.stringify({
                  translation: (seat === firstWriterOf({ context: scene.context, }))
                    ? 'Later the cat dozes in the sun.'
                    : 'Later the kitten naps in the sun.',
                }, ), };
              },
            },);
            const { written, findings, } = await writtenOffers({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: { ...scene.unit, keeping: ['kitten naps',], },
              english: RECEIVING_ENGLISH,
            },);
            expect({
              seats: written.map(function seatOf({ seat, },): string {
                return seat;
              },),
              findings,
            },).toEqual({
              seats: writerSeatsOf({ context: scene.context, }).slice(1,),
              findings: [
                `consistency-candidate-refused (${firstWriterOf({ context: scene.context, })}): the paragraph no longer holds `
                  + 'the words of another place that must stay as they are',
              ],
            },);
          },
        },),
        it({
          name: 'ASKS NO WRITER WHEN THE WRITERS ARE NOT ASKED, reporting the bench and why',
          fn: async () => {
            const scene = carryScene({
              behave: writersAnswer,
              writers: { kind: 'not-asked', why: 'the translators bench seats no model', },
            },);
            expect(await writtenOffers({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: scene.unit,
              english: RECEIVING_ENGLISH,
            },),).toEqual({
              written: [],
              findings: [
                'consistency-writers-not-asked (OCCURRENCE 2): the translators bench was not asked: '
                  + 'the translators bench seats no model',
              ],
            },);
            expect(scene.asked,).toEqual([],);
          },
        },),
        it({
          name: 'ASKS NO WRITER FOR WORDS THAT LIE IN NO SINGLE BLOCK, reporting that no paragraph can be rewritten',
          fn: async () => {
            const scene = carryScene({ behave: writersAnswer, },);
            expect(await writtenOffers({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: { ...scene.unit, writable: false, spliceOnlyIn: [], unwritableBecause: 'no-single-block', },
              english: RECEIVING_ENGLISH,
            },),).toEqual({
              written: [],
              findings: [
                'consistency-writers-not-asked (OCCURRENCE 2): the words that render the repeated wording lie in no single '
                  + 'block of the English, so no paragraph can be rewritten',
              ],
            },);
            expect(scene.asked,).toEqual([],);
          },
        },),
        it({
          name: 'ASKS NO WRITER FOR A PARAGRAPH THAT OVERLAPS THE WORDS OF ANOTHER RECEIVING PLACE OUTSIDE IT, reporting that '
            + 'overlap and not that the words lie in no single block',
          fn: async () => {
            const scene = carryScene({ behave: writersAnswer, },);
            expect(await writtenOffers({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: {
                ...scene.unit,
                block: { start: 6, end: RECEIVING_ENGLISH.length, text: RECEIVING_ENGLISH.slice(6,), },
                writable: false,
                spliceOnlyIn: [],
                unwritableBecause: 'overlaps-outside-receiver',
              },
              english: RECEIVING_ENGLISH,
            },),).toEqual({
              written: [],
              findings: [
                'consistency-writers-not-asked (OCCURRENCE 2): the paragraph that holds the words that render the repeated '
                  + 'wording overlaps the words of another receiving place of the group that lie outside it, so no paragraph '
                  + 'can be rewritten',
              ],
            },);
            expect(scene.asked,).toEqual([],);
          },
        },),
        it({
          name: 'PINS THE WORDING TO THE RECORDED REASON, NOT TO THE EXTENT OF THE BLOCK: a unit whose block is the '
            + 'whole English and which is marked overlapped gets the overlap wording, and one that records no reason and '
            + 'stands in no kind of block is refused',
          fn: async () => {
            const scene = carryScene({ behave: writersAnswer, },);
            const overlapped = await writtenOffers({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: { ...scene.unit, writable: false, spliceOnlyIn: [], unwritableBecause: 'overlaps-outside-receiver', },
              english: RECEIVING_ENGLISH,
            },);
            const unrecorded = await rejectionOf(async function offersWithoutReason(): Promise<unknown> {
              return await writtenOffers({
                context: scene.context,
                repeat: CARRY_REPEAT,
                chosen: scene.chosen,
                unit: { ...scene.unit, writable: false, spliceOnlyIn: [], },
                english: RECEIVING_ENGLISH,
              },);
            },);
            expect(overlapped.findings,).toEqual([
              'consistency-writers-not-asked (OCCURRENCE 2): the paragraph that holds the words that render the repeated '
                + 'wording overlaps the words of another receiving place of the group that lie outside it, so no paragraph '
                + 'can be rewritten',
            ],);
            expect(unrecorded,).toBeInstanceOf(ConsistencyInvariantError,);
            expect(String(unrecorded,),).toBe(
              'ConsistencyInvariantError: unreachable: OCCURRENCE 2 stands in a unit that is not writable and in no kind of '
                + 'block, and such a unit records why it is not writable',
            );
          },
        },),
        it({
          name: 'ASKS NO WRITER FOR WORDS IN A TABLE, A CODE BLOCK, A SETEXT HEADING OR A PREFORMATTED BLOCK, reporting which '
            + 'kind of block holds them and that no paragraph can be rewritten',
          fn: async () => {
            const scene = carryScene({ behave: writersAnswer, },);
            /**
             The findings of asking for a unit that stands in one kind of block.
             */
            async function findingsFor(
              { kind, }: { readonly kind: 'table' | 'code block' | 'setext heading' | 'preformatted block'; },
            ): Promise<readonly string[]> {
              return (await writtenOffers({
                context: scene.context,
                repeat: CARRY_REPEAT,
                chosen: scene.chosen,
                unit: { ...scene.unit, writable: false, spliceOnlyIn: [kind,], },
                english: RECEIVING_ENGLISH,
              },)).findings;
            }
            expect([
              await findingsFor({ kind: 'table', },),
              await findingsFor({ kind: 'code block', },),
              await findingsFor({ kind: 'setext heading', },),
              await findingsFor({ kind: 'preformatted block', },),
            ],).toEqual([
              [
                'consistency-writers-not-asked (OCCURRENCE 2): the words that render the repeated wording stand in a table, '
                  + 'so no paragraph can be rewritten',
              ],
              [
                'consistency-writers-not-asked (OCCURRENCE 2): the words that render the repeated wording stand in a code '
                  + 'block, so no paragraph can be rewritten',
              ],
              [
                'consistency-writers-not-asked (OCCURRENCE 2): the words that render the repeated wording stand in a setext '
                  + 'heading, so no paragraph can be rewritten',
              ],
              [
                'consistency-writers-not-asked (OCCURRENCE 2): the words that render the repeated wording stand in a '
                  + 'preformatted block, so no paragraph can be rewritten',
              ],
            ],);
            expect(scene.asked,).toEqual([],);
          },
        },),
        it({
          name: 'ASKS NO WRITER FOR WORDS THAT STAND IN A TABLE, A CODE BLOCK, A SETEXT HEADING (ALSO ONE WHOSE TEXT OPENS WITH A '
            + 'HASH) OR HTML TABLE MARKUP OF A REAL ENGLISH, and asks the writers for the words of a paragraph in the same '
            + 'chain',
          fn: async () => {
            /**
             What asking for a receiving place in one English comes to: the standing the English gives its words, the unit
             made from it, what the offers report and the writers asked.
             */
            async function offersFromEnglish(
              { english, }: { readonly english: string; },
            ): Promise<{
              readonly writable: boolean;
              readonly spliceOnlyIn: readonly string[];
              readonly findings: readonly string[];
              readonly written: number;
              readonly asked: number;
            }> {
              const scene = carryScene({ behave: writersAnswer, },);
              /**
               Where the receiving place's words start in the English.
               */
              const start = english.indexOf('the kitten naps',);
              /**
               The receiving place, standing in this English.
               */
              const receiver: CarryReceiver = {
                ...scene.receiver,
                place: {
                  ...scene.receiver.place,
                  slice: { ...scene.receiver.place.slice, current: { kind: 'present', text: english, }, },
                },
                rendering: { kind: 'located', quote: 'the kitten naps', core: 'the kitten naps', start, },
              };
              /**
               Where the words stand among the blocks of the English.
               */
              const { block, spliceOnlyIn, } = blockStandingOf({ english, start, end: start + 'the kitten naps'.length, },);
              /**
               The unit the receiving place makes.
               */
              const [unit,] = unitsOf({ held: [{ receiver, block, spliceOnlyIn, },], kept: [], },);
              const { written, findings, } = await writtenOffers({
                context: scene.context,
                repeat: CARRY_REPEAT,
                chosen: scene.chosen,
                unit: nonNullishOrThrow(unit,),
                english,
              },);
              return {
                writable: nonNullishOrThrow(unit,).writable,
                spliceOnlyIn: nonNullishOrThrow(unit,).spliceOnlyIn,
                findings,
                written: written.length,
                asked: scene.asked.length,
              };
            }
            /**
             The finding for words that stand in one kind of block.
             */
            function findingFor({ kind, }: { readonly kind: string; }): string {
              return 'consistency-writers-not-asked (OCCURRENCE 2): the words that render the repeated wording stand in a '
                + `${kind}, so no paragraph can be rewritten`;
            }
            expect([
              await offersFromEnglish({ english: '| a | b |\n| - | - |\n| the kitten naps | x |\n', },),
              await offersFromEnglish({ english: '```\nthe kitten naps\n```\n\nAfter.', },),
              await offersFromEnglish({ english: 'Later the kitten naps\n=====================\n\nAfter.', },),
              await offersFromEnglish({ english: '#later the kitten naps\n======================\n\nAfter.', },),
              await offersFromEnglish({ english: '<table><tr><td>the kitten naps</td></tr></table>\n\nAfter.', },),
              await offersFromEnglish({ english: RECEIVING_ENGLISH, },),
            ],).toEqual([
              { writable: false, spliceOnlyIn: ['table',], findings: [findingFor({ kind: 'table', },),], written: 0, asked: 0, },
              { writable: false, spliceOnlyIn: ['code block',], findings: [findingFor({ kind: 'code block', },),], written: 0, asked: 0, },
              {
                writable: false,
                spliceOnlyIn: ['setext heading',],
                findings: [findingFor({ kind: 'setext heading', },),],
                written: 0,
                asked: 0,
              },
              {
                writable: false,
                spliceOnlyIn: ['setext heading',],
                findings: [findingFor({ kind: 'setext heading', },),],
                written: 0,
                asked: 0,
              },
              { writable: false, spliceOnlyIn: ['table',], findings: [findingFor({ kind: 'table', },),], written: 0, asked: 0, },
              { writable: true, spliceOnlyIn: [], findings: [], written: 3, asked: 3, },
            ],);
          },
        },),
      ],
    },),
  ],
},);
