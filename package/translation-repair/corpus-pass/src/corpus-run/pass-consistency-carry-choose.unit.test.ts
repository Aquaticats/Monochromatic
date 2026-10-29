/**
 Tests for the bench's choice among a block's candidates and what it comes to:
 the record of a carry that ended without a choice, once for each receiving
 place of the block, and, for a choice, the carry's records and the replacements
 the winner makes (the words of each receiving place for the splice, the block
 once for every receiving place for a writer's paragraph, nothing for the
 current paragraph). A block with no alternative to its own English is not put
 to the bench.

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
  carryOffersOf,
  ConsistencyInvariantError,
} from '@monochromatic-dev/translation-repair-consistency';
import {
  type CarryUnit,
  carryWithoutChoice,
  choose,
} from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  choosing,
  type SeatBehaviour,
  type SeatScript,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';
import { candidateTextsOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-sheet-reading.test-fixture.ts';
import {
  CARRY_REPEAT,
  carryScene,
  RECEIVING_ENGLISH,
  secondReceiverOf,
  SPLICED_PARAGRAPH,
} from './consistency-carry.test-fixture.ts';

/**
 What a writer wrote.
 */
const WRITTEN = 'Later the cat dozes in the sun.';

/**
 The three candidates: the current paragraph, the splice and one writer's paragraph.
 */
const OFFERS = carryOffersOf({
  current: RECEIVING_ENGLISH,
  splice: [SPLICED_PARAGRAPH,],
  written: [{ seat: 'hyper-cat', text: WRITTEN, },],
  lineStructured: false,
},);

/**
 Label of the writer's paragraph among the three candidates.
 */
const WRITER_LABEL = OFFERS.find(function isWriter({ kind, },): boolean {
  return kind === 'writer';
},)
  ?.label;

/**
 Judges that all choose the candidate holding a text.

 @param text - candidate's text

 @returns What each seat does

 @example
 ```ts
 const behave: SeatScript = allChoose({ text: WRITTEN, },);
 ```
 */
function allChoose({ text, }: { readonly text: string; },): SeatScript {
  return function choosesIt({ sheet, },): SeatBehaviour {
    return choosing({
      choice: candidateTextsOf({ sheet, },)
        .find(function holdsIt(candidate,): boolean {
          return candidate.text === text;
        },)
        ?.label ?? '',
    },);
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: carryWithoutChoice.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RECORDS A CARRY WITH NO BALLOTS, NO USABLE COUNT AND NO QUORUM under each label, the decision and the '
            + 'candidates as given',
          fn: async () => {
            expect(carryWithoutChoice({
              labels: ['OCCURRENCE 2', 'OCCURRENCE 3',],
              decision: { kind: 'failed', detail: 'Error', },
              candidates: [{ label: 'CANDIDATE 1', kind: 'current', writers: [], text: RECEIVING_ENGLISH, },],
            },),).toEqual([
              {
                label: 'OCCURRENCE 2',
                candidates: [{ label: 'CANDIDATE 1', kind: 'current', writers: [], text: RECEIVING_ENGLISH, },],
                ballots: [],
                usable: 0,
                quorum: 0,
                decision: { kind: 'failed', detail: 'Error', },
              },
              {
                label: 'OCCURRENCE 3',
                candidates: [{ label: 'CANDIDATE 1', kind: 'current', writers: [], text: RECEIVING_ENGLISH, },],
                ballots: [],
                usable: 0,
                quorum: 0,
                decision: { kind: 'failed', detail: 'Error', },
              },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: choose.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PUTS NO QUESTION TO THE BENCH WHEN ONLY THE CURRENT PARAGRAPH IS ON OFFER, recording no alternative and '
            + 'making no replacement',
          fn: async () => {
            const scene = carryScene({ behave: allChoose({ text: WRITTEN, }), },);
            const carried = await choose({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: scene.unit,
              offers: carryOffersOf({ current: RECEIVING_ENGLISH, splice: [], written: [], lineStructured: false, },),
              findings: ['consistency-writers-not-asked (OCCURRENCE 2): none',],
            },);
            expect({ carried, asked: scene.asked, },).toEqual({
              carried: {
                carries: [
                  {
                    label: 'OCCURRENCE 2',
                    candidates: [{ label: 'CANDIDATE 1', kind: 'current', writers: [], text: RECEIVING_ENGLISH, },],
                    ballots: [],
                    usable: 0,
                    quorum: 0,
                    decision: { kind: 'none', reason: 'no-alternative', },
                  },
                ],
                edits: [],
                findings: ['consistency-writers-not-asked (OCCURRENCE 2): none',],
              },
              asked: [],
            },);
          },
        },),
        it({
          name: 'REPLACES THE WORDS WHEN THE BENCH CHOOSES THE SPLICE, the block when it chooses a writer\'s paragraph, and '
            + 'nothing when it chooses the current paragraph',
          fn: async () => {
            const results = await Promise.all([RECEIVING_ENGLISH, SPLICED_PARAGRAPH, WRITTEN,].map(
              async function chooses(text,) {
                const scene = carryScene({ behave: allChoose({ text, }), },);
                const { carries, edits, } = await choose({
                  context: scene.context,
                  repeat: CARRY_REPEAT,
                  chosen: scene.chosen,
                  unit: scene.unit,
                  offers: OFFERS,
                  findings: [],
                },);
                return {
                  decided: OFFERS.find(function isNamed({ label, },): boolean {
                    return (carries[0]?.decision.kind === 'chosen') && (carries[0].decision.label === label);
                  },)?.kind,
                  edits,
                };
              },
            ),);
            expect(results,).toEqual([
              { decided: 'current', edits: [], },
              {
                decided: 'splice',
                edits: [{ kind: 'splice', placePositions: [1,], sliceIndex: 1, start: 6, end: 21, text: 'the cat naps', },],
              },
              {
                decided: 'writer',
                edits: [
                  { kind: 'block', placePositions: [1,], sliceIndex: 1, start: 0, end: RECEIVING_ENGLISH.length, text: WRITTEN, },
                ],
              },
            ],);
          },
        },),
        it({
          name: 'CARRIES EVERY RECEIVING PLACE OF A BLOCK IN ONE REPLACEMENT WHEN THE BENCH CHOOSES A WRITER\'S PARAGRAPH, and '
            + 'each place\'s own words when it chooses the splice, the record being one choice repeated under each label',
          fn: async () => {
            const scene = carryScene({ behave: allChoose({ text: WRITTEN, }), },);
            const unit: CarryUnit = {
              ...scene.unit,
              receivers: [scene.receiver, secondReceiverOf({ receiver: scene.receiver, },),],
            };
            const written = await choose({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit,
              offers: OFFERS,
              findings: [],
            },);
            const spliceScene = carryScene({ behave: allChoose({ text: 'Later the cat naps in the warm sun.', }), },);
            const spliceOffers = carryOffersOf({
              current: RECEIVING_ENGLISH,
              splice: ['Later the cat naps in the warm sun.',],
              written: [],
              lineStructured: false,
            },);
            const spliced = await choose({
              context: spliceScene.context,
              repeat: CARRY_REPEAT,
              chosen: spliceScene.chosen,
              unit,
              offers: spliceOffers,
              findings: [],
            },);
            expect({
              writtenEdits: written.edits,
              writtenLabels: written.carries.map(function labelOf({ label, },): string {
                return label;
              },),
              writtenDecisions: written.carries.map(function decisionOf({ decision, },) {
                return decision;
              },),
              splicedEdits: spliced.edits,
              asked: scene.asked.length,
            },).toEqual({
              writtenEdits: [
                { kind: 'block', placePositions: [1, 2,], sliceIndex: 1, start: 0, end: RECEIVING_ENGLISH.length, text: WRITTEN, },
              ],
              writtenLabels: ['OCCURRENCE 2', 'OCCURRENCE 3',],
              writtenDecisions: [
                { kind: 'chosen', label: WRITER_LABEL, },
                { kind: 'chosen', label: WRITER_LABEL, },
              ],
              splicedEdits: [
                { kind: 'splice', placePositions: [1,], sliceIndex: 1, start: 6, end: 21, text: 'the cat naps', },
                { kind: 'splice', placePositions: [2,], sliceIndex: 1, start: 25, end: 32, text: 'the warm sun', },
              ],
              asked: 3,
            },);
          },
        },),
        it({
          name: 'REFUSES A SPLICE THAT WON FOR A BLOCK NONE OF WHOSE PLACES HAS WORDS CODE CAN MOVE, since no splice is '
            + 'offered then',
          fn: async () => {
            const scene = carryScene({ behave: allChoose({ text: SPLICED_PARAGRAPH, }), },);
            const refusal = await rejectionOf(async function chooseUnspliceable(): Promise<unknown> {
              return await choose({
                context: scene.context,
                repeat: CARRY_REPEAT,
                chosen: scene.chosen,
                unit: {
                  ...scene.unit,
                  receivers: [{ ...scene.receiver, plan: { kind: 'unavailable', }, },],
                },
                offers: OFFERS,
                findings: [],
              },);
            },);
            expect(refusal,).toBeInstanceOf(ConsistencyInvariantError,);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: the splice won for OCCURRENCE 2, none of whose receiving places has '
                + 'words that code can move, and a splice is offered only when some receiving place of the block can be spliced',
            );
          },
        },),
        it({
          name: 'REPLACES THE WORDS OF THE PLACES CODE CAN MOVE WHEN THE SPLICE WINS FOR A BLOCK CARRIED IN PART, leaves the '
            + 'other place no replacement, records one choice under both labels and reports which place kept its words',
          fn: async () => {
            const scene = carryScene({ behave: allChoose({ text: 'Later the cat naps in the sun.', }), },);
            const unit: CarryUnit = {
              ...scene.unit,
              receivers: [
                { ...scene.receiver, plan: { kind: 'unavailable', }, },
                secondReceiverOf({ receiver: scene.receiver, },),
              ],
            };
            const offers = carryOffersOf({
              current: RECEIVING_ENGLISH,
              splice: ['Later the cat naps in the sun.',],
              written: [],
              lineStructured: false,
            },);
            const { carries, edits, findings, } = await choose({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit,
              offers,
              findings: [],
            },);
            expect({
              edits,
              labels: carries.map(function labelOf({ label, },): string {
                return label;
              },),
              decisions: carries.map(function decisionOf({ decision, },) {
                return decision;
              },),
              findings,
            },).toEqual({
              edits: [{ kind: 'splice', placePositions: [2,], sliceIndex: 1, start: 25, end: 32, text: 'the warm sun', },],
              labels: ['OCCURRENCE 2', 'OCCURRENCE 3',],
              decisions: [
                { kind: 'chosen', label: 'CANDIDATE 2', },
                { kind: 'chosen', label: 'CANDIDATE 2', },
              ],
              findings: [
                'consistency-partly-carried (OCCURRENCE 2 and OCCURRENCE 3): the splice won, so code moved the chosen words '
                  + 'into OCCURRENCE 3 and left OCCURRENCE 2 as it was, since code cannot move them there',
              ],
            },);
          },
        },),
        it({
          name: 'RECORDS EVERY CANDIDATE, EVERY BALLOT AND THE COUNT, with the quorum the gather closed on and the findings '
            + 'the candidates\' making and the asking reported',
          fn: async () => {
            const scene = carryScene({ behave: allChoose({ text: WRITTEN, }), },);
            const { carries, findings, } = await choose({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: scene.unit,
              offers: OFFERS,
              findings: ['consistency-candidate-refused (hyper-cat): none',],
            },);
            const [carry,] = carries;
            expect({
              labels: carry?.candidates.map(function labelOf({ label, },): string {
                return label;
              },),
              kinds: carry?.candidates.map(function kindOf({ kind, },): string {
                return kind;
              },).toSorted(),
              ballots: carry?.ballots.length,
              usable: carry?.usable,
              quorum: carry?.quorum,
              findings,
            },).toEqual({
              labels: ['CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 3',],
              kinds: ['current', 'splice', 'writer',],
              ballots: 3,
              usable: 3,
              quorum: 2,
              findings: ['consistency-candidate-refused (hyper-cat): none',],
            },);
          },
        },),
        it({
          name: 'MAKES NO REPLACEMENT WHEN THE CHOICES HAVE NO STRICT MAJORITY, recording that decision',
          fn: async () => {
            const scene = carryScene({
              behave: function splitsThreeWays({ seat, sheet, },): SeatBehaviour {
                const candidates = candidateTextsOf({ sheet, },);
                const at = scene.context.judgeModelIds.indexOf(seat,);
                return choosing({ choice: candidates[at % candidates.length]?.label ?? '', },);
              },
            },);
            const { carries, edits, } = await choose({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: scene.unit,
              offers: OFFERS,
              findings: [],
            },);
            expect({ decision: carries[0]?.decision, edits, },).toEqual({
              decision: { kind: 'none', reason: 'no-majority', },
              edits: [],
            },);
          },
        },),
      ],
    },),
  ],
},);
